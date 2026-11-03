//! App-local semantic wrappers over the real controller; test transports and
//! RAM disk are host fixtures, not native accessibility or persistence proof.
use capability_core::{Engine, Message, SEND};
use nieuw_os_catalog_client::Transport;
use nieuw_os_document_session::{
    volume::{Buffers, VolumeSession},
    EditMode,
};
use nieuw_os_input_client::{Binding, Session as Input};
use nieuw_os_materiaal::{
    controller::{Controller, Error as ControllerError},
    render::{SemanticId, SemanticNode, MAX_SEMANTIC_NODES},
    Scale, Viewport,
};
use nieuw_os_materiaal_app::{App, Error};
use nieuw_os_sdk_runtime::{
    Error as RuntimeError, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};
use nieuw_os_surface_protocol::{self as surface, client::Session as Surface, Geometry, Rectangle};
use std::{cell::RefCell, rc::Rc};
use volume_core::{self as volume, service::Service, table::Table, Disk, Status, SECTOR};

const APP: u64 = 1;
const VOL: u64 = 2;
const COMP: u64 = 3;
const POLICY: u64 = 4;
const DOC_EPOCH: u64 = 17;
const INPUT_EPOCH: u64 = 31;
const SURFACE_EPOCH: u64 = 11;

fn handle(raw: u64) -> Handle {
    Handle::from_raw(raw).unwrap()
}
fn send(raw: u64) -> SendEndpoint {
    SendEndpoint::from_handle(handle(raw))
}
fn receive(raw: u64) -> ReceiveEndpoint {
    ReceiveEndpoint::from_handle(handle(raw))
}
fn pair(engine: &mut Engine, server: u64) -> (u64, u64) {
    let root = engine.create_endpoint(server).unwrap();
    let app_send = engine.bootstrap_delegate(server, root, APP, SEND).unwrap();
    let app_receive = engine.create_endpoint(APP).unwrap();
    (app_send, app_receive)
}
fn geometry() -> Geometry {
    Geometry {
        width: 800,
        height: 600,
        assigned: Rectangle {
            x: 16,
            y: 48,
            width: 768,
            height: 536,
        },
        epoch: SURFACE_EPOCH,
    }
}

struct RamDisk {
    sectors: Vec<[u8; SECTOR]>,
}
impl Disk for RamDisk {
    fn read(&mut self, sector: u64, out: &mut [u8; SECTOR]) -> Result<(), Status> {
        *out = *self.sectors.get(sector as usize).ok_or(Status::Invalid)?;
        Ok(())
    }
    fn write(&mut self, sector: u64, bytes: &[u8; SECTOR]) -> Result<(), Status> {
        *self
            .sectors
            .get_mut(sector as usize)
            .ok_or(Status::Invalid)? = *bytes;
        Ok(())
    }
    fn flush(&mut self) -> Result<(), Status> {
        Ok(())
    }
}
struct VolumeHost {
    service: Service,
    disk: RamDisk,
    reply: Option<[u8; volume::wire::SIZE]>,
}
impl VolumeHost {
    fn new() -> Rc<RefCell<Self>> {
        let sectors = vec![[0; SECTOR]; volume::VOLUME_SECTORS as usize];
        let mut disk = RamDisk { sectors };
        volume::table::write_format_marker(&mut disk).unwrap();
        Rc::new(RefCell::new(Self {
            service: Service::new(DOC_EPOCH, Table::EMPTY, false),
            disk,
            reply: None,
        }))
    }
    fn recovered(&mut self) -> Table {
        let mut table = Table::EMPTY;
        volume::table::recover(&mut self.disk, &mut table).unwrap();
        table
    }
}
struct VolumePort(Rc<RefCell<VolumeHost>>);
impl Transport for VolumePort {
    fn send(&mut self, _: SendEndpoint, message: &Outgoing) -> Result<(), RuntimeError> {
        let message = Message::decode(message.as_bytes()).map_err(|_| RuntimeError::Kernel(-1))?;
        if message.tag != volume::upload::TAG || message.len as usize != volume::wire::SIZE {
            return Err(RuntimeError::Kernel(-1));
        }
        let request: &[u8; volume::wire::SIZE] = message.data[..volume::wire::SIZE]
            .try_into()
            .map_err(|_| RuntimeError::Kernel(-1))?;
        let mut host = self.0.borrow_mut();
        let VolumeHost {
            service,
            disk,
            reply,
        } = &mut *host;
        *reply = Some(service.handle_for(0, APP, request, disk));
        Ok(())
    }
    fn receive(&mut self, _: ReceiveEndpoint) -> Result<Received, RuntimeError> {
        let response = self
            .0
            .borrow_mut()
            .reply
            .take()
            .ok_or(RuntimeError::Kernel(-1))?;
        let mut message = Message::new();
        message.sender_pid = VOL;
        message.tag = volume::upload::TAG;
        message.len = volume::wire::SIZE as u32;
        message.data[..volume::wire::SIZE].copy_from_slice(&response);
        Received::from_bytes(message.encode(), u64::from(message.len))
            .map_err(|_| RuntimeError::Kernel(-1))
    }
    fn close(&mut self, _: Handle) -> Result<(), RuntimeError> {
        Ok(())
    }
    fn yield_now(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }
}

struct UnusedTransport;
impl Transport for UnusedTransport {
    fn send(&mut self, _: SendEndpoint, _: &Outgoing) -> Result<(), RuntimeError> {
        Err(RuntimeError::Kernel(-1))
    }
    fn receive(&mut self, _: ReceiveEndpoint) -> Result<Received, RuntimeError> {
        Err(RuntimeError::Kernel(-1))
    }
    fn close(&mut self, _: Handle) -> Result<(), RuntimeError> {
        Ok(())
    }
    fn yield_now(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }
}

struct SurfaceHost {
    receiver: surface::Receiver,
    reply: Option<Vec<u8>>,
}
struct SurfacePort(Rc<RefCell<SurfaceHost>>);
impl Transport for SurfacePort {
    fn send(&mut self, _: SendEndpoint, message: &Outgoing) -> Result<(), RuntimeError> {
        let message = Message::decode(message.as_bytes()).map_err(|_| RuntimeError::Kernel(-1))?;
        if message.tag != surface::TAG || message.len as usize > message.data.len() {
            return Err(RuntimeError::Kernel(-1));
        }
        let mut host = self.0.borrow_mut();
        match host
            .receiver
            .accept(APP, 1, &message.data[..message.len as usize])
            .map_err(|_| RuntimeError::Kernel(-1))?
        {
            surface::Action::Info(reply) => host.reply = Some(reply.bytes().to_vec()),
            _ => return Err(RuntimeError::Kernel(-1)),
        }
        Ok(())
    }
    fn receive(&mut self, _: ReceiveEndpoint) -> Result<Received, RuntimeError> {
        let bytes = self
            .0
            .borrow_mut()
            .reply
            .take()
            .ok_or(RuntimeError::Kernel(-1))?;
        let mut message = Message::new();
        message.sender_pid = COMP;
        message.tag = surface::TAG;
        message.len = bytes.len() as u32;
        message.data[..bytes.len()].copy_from_slice(&bytes);
        Received::from_bytes(message.encode(), u64::from(message.len))
            .map_err(|_| RuntimeError::Kernel(-1))
    }
    fn close(&mut self, _: Handle) -> Result<(), RuntimeError> {
        Ok(())
    }
    fn yield_now(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }
}

fn app<'a>(
    host: Rc<RefCell<VolumeHost>>,
    mode: EditMode,
    buffers: Buffers<'a>,
) -> App<'a, VolumePort, UnusedTransport, SurfacePort> {
    let mut engine = Engine::new();
    engine.register(APP).unwrap();
    engine.register(VOL).unwrap();
    engine.register(COMP).unwrap();
    engine.register(POLICY).unwrap();
    let volume_pair = pair(&mut engine, VOL);
    let input_pair = pair(&mut engine, POLICY);
    let surface_pair = pair(&mut engine, COMP);
    let volume = VolumeSession::new(
        VolumePort(host),
        DOC_EPOCH,
        VOL,
        0,
        send(volume_pair.0),
        receive(volume_pair.1),
        4,
    )
    .unwrap();
    let controller = Controller::new(
        volume,
        mode,
        buffers,
        Viewport::new(768, 536, Scale::Percent100, SURFACE_EPOCH).unwrap(),
        INPUT_EPOCH,
    )
    .unwrap();
    let input = Input::new(
        UnusedTransport,
        Binding {
            epoch: INPUT_EPOCH,
            service_pid: POLICY,
            client_id: 101,
            width: 800,
            height: 600,
            send: send(input_pair.0),
            receive: receive(input_pair.1),
        },
        4,
    )
    .unwrap();
    let surface_host = Rc::new(RefCell::new(SurfaceHost {
        receiver: surface::Receiver::new(APP, geometry(), 100).unwrap(),
        reply: None,
    }));
    let surface = Surface::new(
        SurfacePort(surface_host),
        COMP,
        SURFACE_EPOCH,
        send(surface_pair.0),
        receive(surface_pair.1),
        4,
    )
    .unwrap();
    App::new(controller, input, surface, geometry()).unwrap()
}

fn snapshot_generation(app: &App<'_, VolumePort, UnusedTransport, SurfacePort>) -> u64 {
    let mut nodes = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
    let snapshot = app.semantic_snapshot(&mut nodes).unwrap();
    assert!(snapshot.node_count <= MAX_SEMANTIC_NODES);
    snapshot.generation
}

#[test]
fn semantic_wrappers_bind_actions_to_snapshot_and_keep_edits_unsaved() {
    let host = VolumeHost::new();
    let (mut draft, mut original, mut staging) = ([0; 512], [0; 512], [0; 512]);
    let mut app = app(
        Rc::clone(&host),
        EditMode::ReadWrite,
        Buffers {
            draft: &mut draft,
            original: &mut original,
            staging: &mut staging,
        },
    );
    let material_generation = snapshot_generation(&app);
    assert_eq!(
        app.activate_semantic(SemanticId::Save, material_generation),
        Err(Error::Controller(ControllerError::InvalidInput))
    );
    assert_eq!(
        app.activate_semantic(SemanticId::NewDocument, material_generation - 1),
        Err(Error::Controller(ControllerError::StaleInput))
    );

    app.activate_semantic(SemanticId::NewDocument, material_generation)
        .unwrap();
    assert_eq!(
        app.set_semantic_value(SemanticId::Document, material_generation, "stale snapshot",),
        Err(Error::Controller(ControllerError::StaleInput))
    );
    let editor_generation = snapshot_generation(&app);
    app.set_semantic_value(
        SemanticId::Document,
        editor_generation,
        "App-local semantic edit",
    )
    .unwrap();
    assert_eq!(
        app.controller().view().contents,
        Some("App-local semantic edit")
    );
    assert!(app.controller().document().is_dirty());
    assert_eq!(host.borrow_mut().recovered(), Table::EMPTY);

    assert_eq!(
        app.activate_semantic(SemanticId::Save, editor_generation),
        Err(Error::Controller(ControllerError::StaleInput))
    );
    assert!(!app.controller().document().is_saving());
    assert_eq!(host.borrow_mut().recovered(), Table::EMPTY);
    let edited_generation = snapshot_generation(&app);
    assert_ne!(edited_generation, editor_generation);
    app.activate_semantic(SemanticId::Save, edited_generation)
        .unwrap();
    assert!(app.controller().document().is_saving());
    assert_eq!(host.borrow_mut().recovered(), Table::EMPTY);
    assert_eq!(app.evidence().errors, 4);
}

#[test]
fn semantic_wrappers_reject_read_only_and_nonexistent_actions() {
    let host = VolumeHost::new();
    let (mut draft, mut original, mut staging) = ([0; 512], [0; 512], [0; 512]);
    let mut app = app(
        host,
        EditMode::ReadOnly,
        Buffers {
            draft: &mut draft,
            original: &mut original,
            staging: &mut staging,
        },
    );
    let generation = snapshot_generation(&app);
    assert_eq!(
        app.activate_semantic(SemanticId::NewDocument, generation),
        Err(Error::Controller(ControllerError::InvalidInput))
    );
    assert_eq!(
        app.set_semantic_value(SemanticId::Document, generation, "forbidden"),
        Err(Error::Controller(ControllerError::InvalidInput))
    );
    assert_eq!(app.controller().document().draft(), None);
    assert_eq!(app.evidence().errors, 2);
}

//! R02 changed-region repaint: digest-proven RETAIN inside conservative
//! controller damage. Host fixtures (RAM disk, empty input queue, retained
//! receiver with a framebuffer); not native display or latency evidence.
use capability_core::{Engine, Message, SEND};
use nieuw_os_catalog_client::Transport;
use nieuw_os_document_session::{
    volume::{Buffers, VolumeSession},
    EditMode,
};
use nieuw_os_input_client::{Binding, Session as Input};
use nieuw_os_input_core::wire::{self as input_wire, Request as InputRequest};
use nieuw_os_materiaal::{
    controller::Controller,
    render::{Frame, SemanticId, SemanticNode, MAX_SEMANTIC_NODES},
    Scale, Viewport,
};
use nieuw_os_materiaal_app::{App, DAMAGE_TILES};
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

/// Answers every PEEK with Empty: the app stays input-responsive and is never
/// marked input-failed, so the retained path stays active.
struct EmptyInput(Option<Vec<u8>>);
impl Transport for EmptyInput {
    fn send(&mut self, _: SendEndpoint, message: &Outgoing) -> Result<(), RuntimeError> {
        let message = Message::decode(message.as_bytes()).map_err(|_| RuntimeError::Kernel(-1))?;
        let request = InputRequest::decode(&message.data[..message.len as usize], INPUT_EPOCH)
            .map_err(|_| RuntimeError::Kernel(-1))?;
        self.0 = Some(input_wire::response(request, 101, None).to_vec());
        Ok(())
    }
    fn receive(&mut self, _: ReceiveEndpoint) -> Result<Received, RuntimeError> {
        let bytes = self.0.take().ok_or(RuntimeError::Kernel(-1))?;
        let mut message = Message::new();
        message.sender_pid = POLICY;
        message.tag = input_wire::TAG;
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

/// Retained receiver with an 800x600 framebuffer; RETAIN leaves pixels alone.
struct SurfaceHost {
    receiver: surface::Receiver,
    reply: Option<Vec<u8>>,
    framebuffer: Vec<u32>,
    now: u64,
}
struct SurfacePort(Rc<RefCell<SurfaceHost>>);
impl Transport for SurfacePort {
    fn send(&mut self, _: SendEndpoint, message: &Outgoing) -> Result<(), RuntimeError> {
        let message = Message::decode(message.as_bytes()).map_err(|_| RuntimeError::Kernel(-1))?;
        let payload = &message.data[..message.len as usize];
        let mut host = self.0.borrow_mut();
        host.now += 1;
        let now = host.now;
        let SurfaceHost {
            receiver,
            reply,
            framebuffer,
            ..
        } = &mut *host;
        // A rejected RETAIN or blit is a test failure, never a silent skip.
        *reply = match receiver.accept(APP, now, payload).unwrap() {
            surface::Action::Staged => None,
            surface::Action::Info(r) | surface::Action::Retain(r) => Some(r.bytes().to_vec()),
            surface::Action::Commit(token) => {
                let r = receiver
                    .commit_with(token, now, |r, b| {
                        for (i, chunk) in b.as_chunks::<4>().0.iter().enumerate() {
                            let x = r.x + i as u32 % u32::from(r.width);
                            let y = r.y + i as u32 / u32::from(r.width);
                            framebuffer[(y * 800 + x) as usize] = u32::from_le_bytes(*chunk);
                        }
                        Ok::<(), ()>(())
                    })
                    .unwrap();
                assert!(r.status().is_ok(), "compositor rejected a tile");
                Some(r.bytes().to_vec())
            }
        };
        Ok(())
    }
    fn receive(&mut self, _: ReceiveEndpoint) -> Result<Received, RuntimeError> {
        let bytes = self
            .0
            .borrow_mut()
            .reply
            .take()
            .ok_or(RuntimeError::Kernel(-11))?;
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

type TestApp<'a> = App<'a, VolumePort, EmptyInput, SurfacePort>;

fn app<'a>(
    surface_host: &Rc<RefCell<SurfaceHost>>,
    mode: EditMode,
    buffers: Buffers<'a>,
    cache: &'a mut [u64],
) -> TestApp<'a> {
    let mut engine = Engine::new();
    for pid in [APP, VOL, COMP, POLICY] {
        engine.register(pid).unwrap();
    }
    let volume_pair = pair(&mut engine, VOL);
    let input_pair = pair(&mut engine, POLICY);
    let surface_pair = pair(&mut engine, COMP);
    let volume = VolumeSession::new(
        VolumePort(VolumeHost::new()),
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
        EmptyInput(None),
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
    let mut surface = Surface::new(
        SurfacePort(Rc::clone(surface_host)),
        COMP,
        SURFACE_EPOCH,
        send(surface_pair.0),
        receive(surface_pair.1),
        4,
    )
    .unwrap();
    let g = surface.geometry().unwrap();
    assert_eq!(g, geometry());
    let mut app = App::with_damage_cache(controller, input, surface, g, cache).unwrap();
    app.enable_retained().unwrap();
    app
}

fn surface_host() -> Rc<RefCell<SurfaceHost>> {
    Rc::new(RefCell::new(SurfaceHost {
        receiver: surface::Receiver::new_retained(APP, geometry(), 100).unwrap(),
        reply: None,
        framebuffer: vec![0; 800 * 600],
        now: 0,
    }))
}

fn finish_paint(app: &mut TestApp<'_>) {
    for _ in 0..geometry().tiles() + 1 {
        if !app.controller().view().dirty {
            return;
        }
        app.step().unwrap();
    }
    panic!("never completed the current frame");
}

fn generation(app: &TestApp<'_>) -> u64 {
    let mut nodes = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
    app.semantic_snapshot(&mut nodes).unwrap().generation
}

/// Global framebuffer index of a pixel inside the assigned surface.
fn at(x: u32, y: u32) -> usize {
    let g = geometry().assigned;
    ((g.y + y) * 800 + g.x + x) as usize
}

/// Every tile equals a fresh render of the current view; nothing outside the
/// assigned surface was written. `skip` excludes one deliberately mutated pixel.
fn assert_exact(app: &TestApp<'_>, host: &SurfaceHost, skip: Option<usize>) {
    let frame = Frame::new(app.controller().view());
    let g = geometry();
    let mut pixels = [0; 256];
    for tile in 0..g.tiles() {
        let r = g.rectangle(tile).unwrap();
        let drawn = frame.render_tile(r.x, r.y, &mut pixels).unwrap();
        for row in 0..drawn.height {
            for col in 0..drawn.width {
                let index = at(r.x + col, r.y + row);
                if Some(index) != skip {
                    assert_eq!(
                        host.framebuffer[index],
                        pixels[(row * drawn.width + col) as usize],
                        "stale or wrong pixel in tile {tile} at ({col},{row})"
                    );
                }
            }
        }
    }
    let a = g.assigned;
    for (index, pixel) in host.framebuffer.iter().enumerate() {
        let (x, y) = (index as u32 % 800, index as u32 / 800);
        if x < a.x || y < a.y || x >= a.x + u32::from(a.width) || y >= a.y + u32::from(a.height) {
            assert_eq!(*pixel, 0, "pixel outside the assigned surface changed");
        }
    }
}

fn editor<'a>(
    host: &Rc<RefCell<SurfaceHost>>,
    buffers: Buffers<'a>,
    cache: &'a mut [u64],
) -> TestApp<'a> {
    let mut app = app(host, EditMode::ReadWrite, buffers, cache);
    finish_paint(&mut app);
    assert_eq!(app.surface_counts().committed_tiles, geometry().tiles());
    let g = generation(&app);
    app.activate_semantic(SemanticId::NewDocument, g).unwrap();
    finish_paint(&mut app);
    app
}

#[test]
fn small_edit_under_full_damage_sends_only_changed_tiles() {
    let host = surface_host();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut cache = [0; DAMAGE_TILES];
    let buffers = Buffers {
        draft: &mut d,
        original: &mut o,
        staging: &mut s,
    };
    let mut app = editor(&host, buffers, &mut cache);
    assert_exact(&app, &host.borrow(), None);

    // Mutation control: corrupt one pixel of the bottom-left tile, far from
    // the edited text. If that tile were re-sent, the corruption would vanish.
    let a = geometry().assigned;
    // Bottom-left: untouched by the edit and not the last tile, which always
    // commits so the frame completes.
    let probe = at(3, u32::from(a.height) - 3);
    let original = host.borrow().framebuffer[probe];
    host.borrow_mut().framebuffer[probe] = original ^ 0x00ff_ffff;

    let before = app.surface_counts();
    let g = generation(&app);
    app.set_semantic_value(SemanticId::Document, g, "x")
        .unwrap();
    finish_paint(&mut app);
    assert_eq!(app.controller().view().contents, Some("x"));
    let after = app.surface_counts();
    let committed = after.committed_tiles - before.committed_tiles;
    let retained = after.retained_tiles - before.retained_tiles;
    assert_eq!(
        committed + retained,
        geometry().tiles(),
        "every tile is accounted for"
    );
    assert!(committed > 0, "the changed text tile must be sent");
    assert!(
        committed * 20 < geometry().tiles(),
        "a one-character edit must send <5% of tiles, sent {committed}"
    );
    // The untouched tile was retained byte-for-byte (corruption survives) and
    // every other tile, including the changed one, is exact.
    assert_eq!(host.borrow().framebuffer[probe], original ^ 0x00ff_ffff);
    assert_exact(&app, &host.borrow(), Some(probe));
    // The check itself is sensitive: without the skip it reports the mutation.
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        assert_exact(&app, &host.borrow(), None)
    }));
    assert!(caught.is_err(), "pixel oracle missed a corrupted tile");
}

#[test]
fn layout_change_and_superseded_partial_stay_exact() {
    let host = surface_host();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut cache = [0; DAMAGE_TILES];
    let buffers = Buffers {
        draft: &mut d,
        original: &mut o,
        staging: &mut s,
    };
    let mut app = editor(&host, buffers, &mut cache);

    // Interrupt a partial frame after its first real tile, then supersede it.
    // Tiles written since the base frame must be re-sent, never RETAINed
    // (the receiver would reject that and the step would fail).
    let g = generation(&app);
    app.set_semantic_value(SemanticId::Document, g, "first")
        .unwrap();
    let committed = app.surface_counts().committed_tiles;
    for _ in 0..geometry().tiles() {
        app.step().unwrap();
        if app.surface_counts().committed_tiles > committed {
            break;
        }
    }
    assert!(app.surface_counts().committed_tiles > committed);
    assert!(
        app.controller().view().dirty,
        "test must interrupt a partial frame"
    );
    let g = generation(&app);
    app.set_semantic_value(SemanticId::Document, g, "second")
        .unwrap();
    finish_paint(&mut app);
    assert_exact(&app, &host.borrow(), None);

    // A whole-screen layout change (editor -> list) repaints what differs.
    let g = generation(&app);
    app.activate_semantic(SemanticId::Back, g).unwrap();
    finish_paint(&mut app);
    for _ in 0..4 {
        if !app.controller().view().dirty {
            break;
        }
        finish_paint(&mut app);
    }
    assert_exact(&app, &host.borrow(), None);
}

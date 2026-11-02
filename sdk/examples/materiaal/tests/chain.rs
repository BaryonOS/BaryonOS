//! Host contract chain over real capability/input/volume engines. RAM sectors
//! and the recording display counterpart are explicitly NOT native evidence.
use capability_core::{Engine, Message, SEND};
use nieuw_os_catalog_client::Transport;
use nieuw_os_document_session::{
    volume::{Buffers, VolumeSession},
    EditMode,
};
use nieuw_os_input_client::{Binding, Session as Input};
use nieuw_os_input_core::{
    self as input,
    wire::{self as iw, Operation, Request},
};
use nieuw_os_materiaal::{
    controller::{Controller, Screen, Status},
    render::{Frame, Role, SemanticNode, MAX_SEMANTIC_NODES},
    Scale, Viewport,
};
use nieuw_os_materiaal_app::{bootstrap::Bootstrap, App, Error};
use nieuw_os_sdk_runtime::{
    Error as RuntimeError, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};
use nieuw_os_surface_protocol::{self as surface, client::Session as Surface, Geometry, Rectangle};
use std::{cell::RefCell, rc::Rc};
use volume_core::{
    self as volume,
    client::{Client, Transport as CoreTransport},
    service::Service,
    table::Table,
    Disk, Status as VolumeStatus, SECTOR,
};

const APP: u64 = 1;
const VOL: u64 = 2;
const COMP: u64 = 3;
const SOURCE: u64 = 4;
const POLICY: u64 = 5;
const INIT: u64 = 6;
const DOC_EPOCH: u64 = 17;
const INPUT_EPOCH: u64 = 31;
const SURFACE_EPOCH: u64 = 11;
fn handle(raw: u64) -> Handle {
    Handle::from_raw(raw).unwrap()
}
fn send(raw: u64) -> SendEndpoint {
    SendEndpoint::from_handle(handle(raw))
}
fn recv(raw: u64) -> ReceiveEndpoint {
    ReceiveEndpoint::from_handle(handle(raw))
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
#[derive(Clone, Copy)]
struct Pair {
    app_send: u64,
    app_receive: u64,
    ingress: u64,
    reply: u64,
    server: u64,
}
fn pair(e: &mut Engine, server: u64, ingress: Option<u64>) -> Pair {
    let ingress = ingress.unwrap_or_else(|| e.create_endpoint(server).unwrap());
    let app_send = e.bootstrap_delegate(server, ingress, APP, SEND).unwrap();
    let app_receive = e.create_endpoint(APP).unwrap();
    let reply = e
        .bootstrap_delegate(APP, app_receive, server, SEND)
        .unwrap();
    Pair {
        app_send,
        app_receive,
        ingress,
        reply,
        server,
    }
}
struct Ram {
    sectors: Vec<[u8; SECTOR]>,
    fail: bool,
}
impl Disk for Ram {
    fn read(&mut self, n: u64, out: &mut [u8; SECTOR]) -> Result<(), VolumeStatus> {
        *out = *self.sectors.get(n as usize).ok_or(VolumeStatus::Invalid)?;
        Ok(())
    }
    fn write(&mut self, n: u64, b: &[u8; SECTOR]) -> Result<(), VolumeStatus> {
        if self.fail {
            return Err(VolumeStatus::Io);
        }
        *self
            .sectors
            .get_mut(n as usize)
            .ok_or(VolumeStatus::Invalid)? = *b;
        Ok(())
    }
    fn flush(&mut self) -> Result<(), VolumeStatus> {
        if self.fail {
            Err(VolumeStatus::Io)
        } else {
            Ok(())
        }
    }
}
struct Host {
    engine: Box<Engine>,
    pairs: [Pair; 3],
    input: input::Service,
    surface: surface::Receiver,
    volume: Service,
    disk: Ram,
    roots: [u64; 4],
    source_send: u64,
    control_send: u64,
    source_id: u64,
    control_id: u64,
    requests: Vec<volume::wire::Header>,
    input_requests: usize,
    blits: usize,
    framebuffer: Vec<u32>,
    now: u64,
    drop_blit: bool,
    drop_commit: bool,
    drop_input: bool,
    fail_blit: bool,
    closed: Vec<u64>,
}
impl Host {
    fn new() -> Rc<RefCell<Self>> {
        let mut e = Box::new(Engine::new());
        for pid in 1..=6 {
            e.register(pid).unwrap();
        }
        let roots = core::array::from_fn(|_| e.create_endpoint(POLICY).unwrap());
        let source_send = e
            .bootstrap_delegate(POLICY, roots[0], SOURCE, SEND)
            .unwrap();
        let control_send = e.bootstrap_delegate(POLICY, roots[1], COMP, SEND).unwrap();
        let pairs = [
            pair(&mut e, VOL, None),
            pair(&mut e, POLICY, Some(roots[3])),
            pair(&mut e, COMP, None),
        ];
        let input = input::Service::new(
            input::Configuration {
                epoch: INPUT_EPOCH,
                source: input::Actor {
                    endpoint: recv(roots[0]),
                    pid: SOURCE,
                },
                compositor: input::Actor {
                    endpoint: recv(roots[1]),
                    pid: COMP,
                },
                supervisor: input::Actor {
                    endpoint: recv(roots[2]),
                    pid: INIT,
                },
                applications: recv(roots[3]),
                width: 800,
                height: 600,
            },
            &[input::ClientBinding { id: 101, pid: APP }],
        )
        .unwrap();
        let host = Rc::new(RefCell::new(Self {
            engine: e,
            pairs,
            input,
            surface: surface::Receiver::new(APP, geometry(), 100).unwrap(),
            volume: Service::new(DOC_EPOCH, Table::EMPTY, false),
            disk: Ram {
                sectors: vec![[0; SECTOR]; volume::VOLUME_SECTORS as usize],
                fail: false,
            },
            roots,
            source_send,
            control_send,
            source_id: 0,
            control_id: 0,
            requests: vec![],
            input_requests: 0,
            blits: 0,
            framebuffer: vec![0; 800 * 600],
            now: 0,
            drop_blit: false,
            drop_commit: false,
            drop_input: false,
            fail_blit: false,
            closed: vec![],
        }));
        {
            let mut h = host.borrow_mut();
            h.actor(true, Operation::Sync, 1, 0);
            h.actor(false, Operation::Focus, 101, 0);
        }
        host
    }
    fn actor(&mut self, source: bool, operation: Operation, arg0: u64, arg1: u64) {
        let (index, pid, channel, id) = if source {
            self.source_id += 1;
            (0, SOURCE, self.source_send, self.source_id)
        } else {
            self.control_id += 1;
            (1, COMP, self.control_send, self.control_id)
        };
        let mut m = Message::new();
        m.tag = iw::TAG;
        m.len = iw::SIZE as u32;
        m.data[..iw::SIZE].copy_from_slice(
            &Request {
                operation,
                id,
                epoch: INPUT_EPOCH,
                arg0,
                arg1,
            }
            .encode(),
        );
        self.engine.send(pid, channel, &m).unwrap();
        let actual = self.engine.receive(POLICY, self.roots[index]).unwrap();
        let received = Received::from_bytes(actual.encode(), actual.len as u64).unwrap();
        self.input
            .handle(recv(self.roots[index]), &received)
            .unwrap();
    }
    fn key(&mut self, byte: u8) {
        self.actor(true, Operation::Sample, 1, u64::from(byte));
    }
    fn packet(&mut self, buttons: u8, dx: i16, dy: i16) {
        assert!((-127..=127).contains(&dx) && (-127..=127).contains(&dy));
        let flags = 8 | buttons | if dx < 0 { 0x10 } else { 0 } | if dy < 0 { 0x20 } else { 0 };
        for byte in [flags, dx as u8, dy as u8] {
            self.actor(true, Operation::Sample, 0x21, u64::from(byte));
        }
    }
    fn mouse(&mut self, x: u32, y: u32, buttons: u8) {
        loop {
            let (cx, cy) = self.input.cursor();
            let dx = (x as i64 - cx as i64).clamp(-127, 127) as i16;
            let dy = (cy as i64 - y as i64).clamp(-127, 127) as i16;
            self.packet(buttons, dx, dy);
            if self.input.cursor() == (x, y) {
                break;
            }
        }
    }
    fn pump(&mut self, index: usize) {
        let p = self.pairs[index];
        let m = match self.engine.receive(p.server, p.ingress) {
            Ok(m) => m,
            Err(capability_core::Error::Again) => return,
            Err(e) => panic!("{e:?}"),
        };
        assert_eq!(m.cap_count, 0);
        let payload = &m.data[..m.len as usize];
        let response = match index {
            0 => {
                assert_eq!(m.tag, volume::upload::TAG);
                let body = payload.try_into().unwrap();
                let h = volume::wire::header(body);
                self.requests.push(h);
                let reply = self
                    .volume
                    .handle_for(0, m.sender_pid, body, &mut self.disk);
                if self.drop_commit && h.operation == volume::wire::COMMIT {
                    return;
                }
                Some(reply.to_vec())
            }
            1 => {
                assert_eq!(m.tag, iw::TAG);
                self.input_requests += 1;
                let received = Received::from_bytes(m.encode(), m.len as u64).unwrap();
                let prepared = match self.input.handle(recv(p.ingress), &received) {
                    Ok(r) => r,
                    Err(e) => self
                        .input
                        .error_reply(recv(p.ingress), &received, e)
                        .unwrap(),
                };
                assert!(self.input.reply_is_current(&prepared));
                if self.drop_input {
                    return;
                }
                Some(prepared.payload().to_vec())
            }
            2 => {
                assert_eq!(m.tag, surface::TAG);
                self.now += 1;
                match self
                    .surface
                    .accept(m.sender_pid, self.now, payload)
                    .unwrap()
                {
                    surface::Action::Staged => None,
                    surface::Action::Info(reply) => Some(reply.bytes().to_vec()),
                    surface::Action::Commit(token) => {
                        let fail = self.fail_blit;
                        let framebuffer = &mut self.framebuffer;
                        let blits = &mut self.blits;
                        let reply = self
                            .surface
                            .commit_with(token, self.now, |r, b| {
                                if fail {
                                    return Err(());
                                }
                                *blits += 1;
                                for (i, chunk) in b.chunks_exact(4).enumerate() {
                                    let x = r.x + i as u32 % u32::from(r.width);
                                    let y = r.y + i as u32 / u32::from(r.width);
                                    framebuffer[(y * 800 + x) as usize] =
                                        u32::from_le_bytes(chunk.try_into().unwrap());
                                }
                                Ok(())
                            })
                            .unwrap();
                        if self.drop_blit {
                            return;
                        }
                        Some(reply.bytes().to_vec())
                    }
                }
            }
            _ => unreachable!(),
        };
        if let Some(response) = response {
            let mut reply = Message::new();
            reply.tag = m.tag;
            reply.len = response.len() as u32;
            reply.data[..response.len()].copy_from_slice(&response);
            self.engine.send(p.server, p.reply, &reply).unwrap();
        }
    }
    fn contents(&mut self) -> Vec<u8> {
        struct Direct<'a>(&'a mut Service, &'a mut Ram);
        impl CoreTransport for Direct<'_> {
            fn call(
                &mut self,
                b: &[u8; volume::wire::SIZE],
            ) -> Result<[u8; volume::wire::SIZE], VolumeStatus> {
                Ok(self.0.handle(0, b, self.1))
            }
        }
        let mut c = Client::new(Direct(&mut self.volume, &mut self.disk), DOC_EPOCH, 900_000);
        let stat = c.stat(0).unwrap();
        let mut bytes = vec![0; stat.length];
        c.read(0, stat.revision, stat.length, |offset, chunk| {
            bytes[offset..offset + chunk.len()].copy_from_slice(chunk)
        })
        .unwrap();
        bytes
    }
}
struct Port(Rc<RefCell<Host>>, usize);
impl Transport for Port {
    fn send(&mut self, to: SendEndpoint, m: &Outgoing) -> Result<(), RuntimeError> {
        let mut h = self.0.borrow_mut();
        h.engine
            .send(
                APP,
                to.handle().raw(),
                &Message::decode(m.as_bytes()).unwrap(),
            )
            .map_err(|e| RuntimeError::Kernel(e.code()))?;
        h.pump(self.1);
        Ok(())
    }
    fn receive(&mut self, from: ReceiveEndpoint) -> Result<Received, RuntimeError> {
        let mut h = self.0.borrow_mut();
        let m = h
            .engine
            .receive(APP, from.handle().raw())
            .map_err(|e| RuntimeError::Kernel(e.code()))?;
        Received::from_bytes(m.encode(), m.len as u64)
    }
    fn close(&mut self, handle: Handle) -> Result<(), RuntimeError> {
        let mut h = self.0.borrow_mut();
        h.closed.push(handle.raw());
        h.engine
            .close(APP, handle.raw())
            .map_err(|e| RuntimeError::Kernel(e.code()))
    }
    fn yield_now(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }
}
fn configured_app<'a>(
    h: &Rc<RefCell<Host>>,
    buffers: Buffers<'a>,
    scale: Scale,
    mode: EditMode,
) -> App<'a, Port, Port, Port> {
    let pairs = h.borrow().pairs;
    let p = pairs[0];
    let volume = VolumeSession::new(
        Port(h.clone(), 0),
        DOC_EPOCH,
        VOL,
        0,
        send(p.app_send),
        recv(p.app_receive),
        4,
    )
    .unwrap();
    let mut controller = Controller::new(
        volume,
        mode,
        buffers,
        Viewport::new(768, 536, scale, SURFACE_EPOCH).unwrap(),
        INPUT_EPOCH,
    )
    .unwrap();
    controller.refresh().unwrap();
    let p = pairs[1];
    let input = Input::new(
        Port(h.clone(), 1),
        Binding {
            epoch: INPUT_EPOCH,
            service_pid: POLICY,
            client_id: 101,
            width: 800,
            height: 600,
            send: send(p.app_send),
            receive: recv(p.app_receive),
        },
        4,
    )
    .unwrap();
    let p = pairs[2];
    let mut surface = Surface::new(
        Port(h.clone(), 2),
        COMP,
        SURFACE_EPOCH,
        send(p.app_send),
        recv(p.app_receive),
        4,
    )
    .unwrap();
    let g = surface.geometry().unwrap();
    App::new(controller, input, surface, g).unwrap()
}
fn app<'a>(h: &Rc<RefCell<Host>>, buffers: Buffers<'a>) -> App<'a, Port, Port, Port> {
    configured_app(h, buffers, Scale::Percent100, EditMode::ReadWrite)
}
fn keys(h: &Rc<RefCell<Host>>, a: &mut App<'_, Port, Port, Port>, bytes: &[u8]) {
    for &b in bytes {
        h.borrow_mut().key(b);
        a.step().unwrap();
    }
}
fn new_text(h: &Rc<RefCell<Host>>, a: &mut App<'_, Port, Port, Port>) {
    keys(h, a, &[0x1c, 0x9c, 0x23, 0xa3, 0x17, 0x97]);
    assert_eq!(a.controller().view().contents, Some("hi"));
}
fn finish_paint(a: &mut App<'_, Port, Port, Port>) {
    for _ in 0..geometry().tiles() + 1 {
        if !a.controller().view().dirty {
            return;
        }
        a.step().unwrap();
    }
    panic!("never completed current frame")
}

fn assert_semantic_node(
    app: &App<'_, Port, Port, Port>,
    role: Role,
    label: &str,
    value: Option<&str>,
    focused: bool,
    enabled: bool,
    read_only: bool,
) {
    let mut nodes = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
    let count = app.semantics(&mut nodes).unwrap();
    let node = nodes[..count]
        .iter()
        .find(|node| node.role == role && node.label == label)
        .unwrap();
    assert_eq!(node.value, value);
    assert_eq!(node.focused, focused);
    assert_eq!(node.enabled, enabled);
    assert_eq!(node.read_only, read_only);
}

fn undersized_semantics_buffer_is_atomic(app: &App<'_, Port, Port, Port>) {
    // The renderer needs exactly the visited node count, not the maximum.
    let mut full = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
    let count = app.semantics(&mut full).unwrap();
    let mut storage = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
    let nodes = &mut storage[..count - 1];
    assert_eq!(
        app.semantics(nodes),
        Err(Error::Accessibility(
            nieuw_os_materiaal::render::Error::Buffer
        ))
    );
    assert!(nodes.iter().all(|node| node.label.is_empty()));
}

fn complete_chain() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 65536], [0; 65536], [0; 65536]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    assert_semantic_node(&a, Role::Heading, "Materiaal", None, false, true, false);
    assert_semantic_node(&a, Role::Button, "Nieuw document", None, true, true, false);
    undersized_semantics_buffer_is_atomic(&a);
    finish_paint(&mut a);
    assert!(a.evidence().ready);
    assert_eq!(a.evidence().frames, 1);
    assert_eq!(a.evidence().tiles, geometry().tiles());
    assert_eq!(
        h.borrow().surface.completed_frame(),
        Some(a.controller().view().generation)
    );
    {
        let r = geometry().assigned;
        let host = h.borrow();
        let fb = &host.framebuffer;
        assert!(fb[(r.y * 800 + r.x) as usize] != 0);
        assert_eq!(fb[0], 0);
    }
    new_text(&h, &mut a);
    assert_semantic_node(
        &a,
        Role::TextEditor,
        "Document",
        Some("hi"),
        true,
        true,
        false,
    );
    assert_eq!(a.evidence().save_acks, 0);
    assert_eq!(a.stop_clean(), Err(Error::Unsaved));
    keys(&h, &mut a, &[0x1d, 0x1f, 0x9f, 0x9d]);
    for _ in 0..3 {
        a.step().unwrap();
    }
    assert_eq!(a.evidence().save_acks, 1);
    assert_eq!(a.evidence().saved_revision, 1);
    assert_eq!(a.evidence().saved_bytes, 2);
    assert_eq!(
        a.evidence().saved_checksum,
        volume::checksum_update(volume::FNV_OFFSET, b"hi")
    );
    assert_semantic_node(
        &a,
        Role::Status,
        "Op dit apparaat opgeslagen.",
        None,
        false,
        true,
        false,
    );
    assert_eq!(h.borrow_mut().contents(), b"hi");
    let mut table = Table::EMPTY;
    volume::table::recover(&mut h.borrow_mut().disk, &mut table).unwrap();
    assert_eq!(table.entries[0].revision, 1);
    keys(
        &h,
        &mut a,
        &[0x01, 0x81, 0x0f, 0x8f, 0x0f, 0x8f, 0x1c, 0x9c],
    );
    assert_eq!(a.controller().view().screen, Screen::Editor);
    assert_eq!(a.controller().view().contents, Some("hi"));
    assert_semantic_node(
        &a,
        Role::TextEditor,
        "Document",
        Some("hi"),
        true,
        true,
        false,
    );
    finish_paint(&mut a);
    a.stop_clean().unwrap();
    assert_eq!(h.borrow().closed.len(), 6);
}
fn lost_surface_reply() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    new_text(&h, &mut a);
    let before = a.evidence();
    h.borrow_mut().drop_blit = true;
    assert!(matches!(a.step(), Err(Error::Surface(_))));
    assert!(a.controller().view().dirty);
    assert_eq!(a.controller().view().contents, Some("hi"));
    assert!(!a.is_visible());
    assert_eq!(a.evidence().tiles, before.tiles);
    assert_eq!(a.evidence().frames, 0);
    assert_eq!(a.evidence().save_acks, 0);
    let blits = h.borrow().blits;
    for _ in 0..5 {
        a.step().unwrap();
    }
    assert_eq!(h.borrow().blits, blits);
}
fn lost_commit_reply() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    new_text(&h, &mut a);
    h.borrow_mut().drop_commit = true;
    keys(&h, &mut a, &[0x1d, 0x1f, 0x9f, 0x9d]);
    assert_eq!(a.evidence().save_acks, 0);
    assert!(a.controller().document().is_uncertain());
    assert_eq!(a.controller().view().contents, Some("hi"));
    assert_eq!(a.evidence().saved_revision, 0);
    assert_eq!(h.borrow_mut().contents(), b"hi");
    assert!(matches!(a.controller().view().status, Status::Failed(_)));
}
fn input_loss_preserves_draft() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    new_text(&h, &mut a);
    h.borrow_mut().drop_input = true;
    assert!(matches!(a.step(), Err(Error::Input(_))));
    assert!(a.input_failed());
    assert_eq!(a.controller().view().contents, Some("hi"));
    assert!(a.controller().document().is_dirty());
    finish_paint(&mut a);
    assert!(!a.evidence().ready);
    assert_eq!(a.evidence().save_acks, 0);
}
fn outside_release_no_click() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    new_text(&h, &mut a);
    let r = Frame::new(a.controller().view()).layout().save;
    let (x, y) = (r.x + 26, r.y + 58);
    h.borrow_mut().mouse(x, y, 1);
    a.step().unwrap();
    h.borrow_mut().mouse(2, 2, 1);
    a.step().unwrap();
    h.borrow_mut().mouse(x, y, 1);
    a.step().unwrap();
    h.borrow_mut().mouse(x, y, 0);
    a.step().unwrap();
    assert!(!h
        .borrow()
        .requests
        .iter()
        .any(|r| r.operation == volume::wire::BEGIN));
    h.borrow_mut().mouse(x, y, 1);
    a.step().unwrap();
    h.borrow_mut().mouse(x, y, 0);
    a.step().unwrap();
    assert_eq!(
        h.borrow()
            .requests
            .iter()
            .filter(|r| r.operation == volume::wire::BEGIN)
            .count(),
        1
    );
}
fn cancelled_save_no_commit() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    new_text(&h, &mut a);
    // Both S and Enter (focused Cancel) are queued in one bounded batch: BEGIN is
    // followed by the real ABORT before save_step can send its first chunk.
    for b in [0x1d, 0x1f, 0x9d, 0x1c] {
        h.borrow_mut().key(b);
    }
    a.step().unwrap();
    keys(&h, &mut a, &[0x9f, 0x9c]);
    assert_eq!(a.evidence().save_acks, 0);
    assert!(a.controller().document().is_dirty());
    assert!(!a.controller().document().is_saving());
    assert!(h
        .borrow()
        .requests
        .iter()
        .any(|r| r.operation == volume::wire::ABORT));
    assert!(!h
        .borrow()
        .requests
        .iter()
        .any(|r| r.operation == volume::wire::COMMIT));
}
macro_rules! scenario {
    ($name:ident,$f:ident) => {
        #[test]
        fn $name() {
            std::thread::Builder::new()
                .stack_size(16 * 1024 * 1024)
                .spawn($f)
                .unwrap()
                .join()
                .unwrap();
        }
    };
}
scenario!(
    real_protocol_chain_edit_save_reopen_and_separate_surface,
    complete_chain
);
scenario!(
    lost_surface_ack_does_not_present_or_drop_draft,
    lost_surface_reply
);
scenario!(lost_document_commit_never_claims_saved, lost_commit_reply);
scenario!(
    input_channel_failure_retains_unsaved_draft,
    input_loss_preserves_draft
);
scenario!(
    global_outside_release_and_held_reentry_never_click,
    outside_release_no_click
);
scenario!(
    cancel_uses_real_abort_and_never_commits,
    cancelled_save_no_commit
);

fn negative_blit() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    h.borrow_mut().fail_blit = true;
    assert!(matches!(a.step(), Err(Error::Surface(_))));
    assert_eq!(h.borrow().blits, 0);
    assert_eq!(a.evidence().tiles, 0);
    assert!(!a.evidence().ready);
    assert!(a.controller().view().dirty);
}
fn change_midframe() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    for _ in 0..13 {
        a.step().unwrap();
    }
    let old = a.controller().view().generation;
    new_text(&h, &mut a);
    assert!(a.controller().view().generation > old);
    assert_eq!(a.evidence().frames, 0);
    assert!(a.controller().view().dirty);
    finish_paint(&mut a);
    assert_eq!(a.evidence().frames, 1);
    assert_eq!(
        h.borrow().surface.completed_frame(),
        Some(a.controller().view().generation)
    );
    assert!(a.evidence().tiles > geometry().tiles());
}
fn input_loss_after_ready() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    finish_paint(&mut a);
    assert!(a.evidence().ready);
    h.borrow_mut().drop_input = true;
    assert!(a.step().is_err());
    assert!(!a.evidence().ready);
    assert_eq!(a.evidence().frames, 1);
}
fn surface_loss_after_ready() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    finish_paint(&mut a);
    new_text(&h, &mut a);
    h.borrow_mut().fail_blit = true;
    assert!(a.step().is_err());
    assert!(!a.evidence().ready);
    assert!(!a.is_visible());
    assert_eq!(a.evidence().frames, 1);
}
fn scale200() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = configured_app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
        Scale::Percent200,
        EditMode::ReadWrite,
    );
    let new = Frame::new(a.controller().view())
        .layout()
        .new
        .physical(Scale::Percent200);
    let (x, y) = (16 + new.x + 8, 48 + new.y + 8);
    h.borrow_mut().mouse(x, y, 0);
    a.step().unwrap();
    h.borrow_mut().mouse(x, y, 1);
    a.step().unwrap();
    h.borrow_mut().mouse(x, y, 0);
    a.step().unwrap();
    assert_eq!(a.controller().view().screen, Screen::Editor);
    keys(&h, &mut a, &[0x23, 0xa3]);
    assert_eq!(a.controller().view().contents, Some("h"));
    finish_paint(&mut a);
    assert!(a.evidence().ready);
}
fn readonly_cannot_create() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = configured_app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
        Scale::Percent100,
        EditMode::ReadOnly,
    );
    keys(
        &h,
        &mut a,
        &[0x1c, 0x9c, 0x23, 0xa3, 0x1d, 0x1f, 0x9f, 0x9d],
    );
    assert_eq!(a.controller().view().screen, Screen::Material);
    assert!(!h
        .borrow()
        .requests
        .iter()
        .any(|r| r.operation == volume::wire::BEGIN));
    assert_eq!(a.evidence().save_acks, 0);
    a.stop_clean().unwrap();
}
fn small_buffer_limit() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 1], [0; 1], [0; 1]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    keys(&h, &mut a, &[0x1c, 0x9c, 0x23, 0xa3, 0x17, 0x97]);
    assert_eq!(a.controller().view().contents, Some("h"));
    assert!(a.evidence().errors > 0);
    assert_eq!(a.evidence().save_acks, 0);
    assert!(a.controller().document().is_dirty());
}
fn focus_loss_suppresses_held_shortcut() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    new_text(&h, &mut a);
    keys(&h, &mut a, &[0x1d]);
    {
        let mut host = h.borrow_mut();
        host.actor(false, Operation::Focus, 0, 0);
        host.actor(false, Operation::Focus, 101, 0);
        host.key(0x1d);
        host.key(0x1f);
    }
    a.step().unwrap();
    assert!(!h
        .borrow()
        .requests
        .iter()
        .any(|r| r.operation == volume::wire::BEGIN));
    assert_eq!(a.controller().view().contents, Some("his"));
    assert_eq!(a.evidence().save_acks, 0);
    keys(&h, &mut a, &[0x9d, 0x9f]);
}
fn cancel_after_append() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    new_text(&h, &mut a);
    {
        let mut host = h.borrow_mut();
        for b in [0x1d, 0x1f, 0x9f, 0x9d] {
            host.key(b);
        }
    }
    a.step().unwrap();
    assert!(a.controller().document().is_saving());
    h.borrow_mut().key(0x1c);
    a.step().unwrap();
    assert!(!a.controller().document().is_saving());
    assert_eq!(a.evidence().save_acks, 0);
    assert!(h
        .borrow()
        .requests
        .iter()
        .any(|r| r.operation == volume::wire::APPEND));
    assert!(h
        .borrow()
        .requests
        .iter()
        .any(|r| r.operation == volume::wire::ABORT));
    assert!(!h
        .borrow()
        .requests
        .iter()
        .any(|r| r.operation == volume::wire::COMMIT));
}
fn disk_failure_retains_draft() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    new_text(&h, &mut a);
    h.borrow_mut().disk.fail = true;
    keys(&h, &mut a, &[0x1d, 0x1f, 0x9f, 0x9d]);
    assert_eq!(a.evidence().save_acks, 0);
    assert_eq!(a.controller().view().contents, Some("hi"));
    assert!(a.controller().document().is_dirty());
    assert!(a.evidence().errors > 0);
}
fn stop_after_no_work() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    a.stop_clean().unwrap();
    a.stop_clean().unwrap();
    let host = h.borrow();
    assert_eq!(host.closed.len(), 6);
    assert!(host
        .requests
        .iter()
        .all(|r| r.operation != volume::wire::COMMIT));
    assert_eq!(a.evidence().save_acks, 0);
}
scenario!(negative_display_completion_never_presents, negative_blit);
scenario!(generation_change_abandons_partial_frame, change_midframe);
scenario!(
    input_failure_revokes_current_ready_flag,
    input_loss_after_ready
);
scenario!(
    surface_failure_revokes_current_ready_flag,
    surface_loss_after_ready
);
scenario!(
    scale200_uses_actual_assigned_global_pointer_coordinates,
    scale200
);
scenario!(
    readonly_ui_never_publishes_create_or_save,
    readonly_cannot_create
);
scenario!(
    bounded_draft_rejects_large_edit_without_destroying_old_bytes,
    small_buffer_limit
);
scenario!(
    focus_regrant_does_not_reuse_held_control,
    focus_loss_suppresses_held_shortcut
);
scenario!(
    cancel_between_real_append_and_commit_preserves_draft,
    cancel_after_append
);
scenario!(
    storage_io_failure_never_claims_saved,
    disk_failure_retains_draft
);
scenario!(
    clean_stop_closes_each_owner_once_without_commit,
    stop_after_no_work
);

fn startup() -> [u64; 128] {
    let mut w = [0; 128];
    for (at, v) in [
        (32, 17),
        (72, 2),
        (80, 3),
        (88, 4),
        (96, 5),
        (168, 2),
        (176, 1),
        (184, 6),
        (544, 7),
        (552, 8),
        (560, 3),
        (568, 11),
        (576, 9),
        (584, 10),
        (592, 5),
        (600, 31),
        (608, 101),
        (616, 1),
    ] {
        w[at / 8] = if matches!(at, 72 | 80 | 88 | 96 | 544 | 552 | 576 | 584) {
            v | (1 << 32)
        } else {
            v
        };
    }
    w
}
#[test]
fn bootstrap_rejects_aliases_invalid_handles_actors_epochs_and_scale() {
    let base = startup();
    assert!(Bootstrap::read(|at| base[at as usize / 8]).is_some());
    for (at, v) in [
        (72, 3),
        (80, 0),
        (552, 7),
        (584, 8),
        (168, 0),
        (176, 2),
        (560, 6),
        (592, 3),
        (32, 0),
        (568, 0),
        (600, 0),
        (608, 0),
        (616, 0),
        (616, 3),
        (112, 2),
    ] {
        let mut w = base;
        w[at / 8] = if matches!(at, 72 | 552 | 584) {
            v | (1 << 32)
        } else {
            v
        };
        assert!(
            Bootstrap::read(|at| w[at as usize / 8]).is_none(),
            "offset{at} value{v}"
        );
    }
    let mut w = base;
    w[616 / 8] = 2;
    w[112 / 8] = 1;
    let b = Bootstrap::read(|at| w[at as usize / 8]).unwrap();
    assert_eq!(b.scale, Scale::Percent200);
    assert!(b.read_only);
}

fn independent_ready_requires_current_complete_frame() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    finish_paint(&mut a);
    assert!(a.evidence().ready);
    let old = a.controller().view().generation;
    assert_eq!(a.evidence().presented_generation, old);
    h.borrow_mut().key(0x1c);
    a.step().unwrap();
    assert!(a.controller().view().dirty);
    assert!(a.controller().view().generation > old);
    assert_eq!(a.evidence().frames, 1);
    assert!(
        !a.evidence().ready,
        "Ready remained true while current generation has only its first tile ACK"
    );
    assert_eq!(a.evidence().presented_generation, old);
    finish_paint(&mut a);
    assert!(a.evidence().ready);
    assert_eq!(
        a.evidence().presented_generation,
        a.controller().view().generation
    );
}
scenario!(
    independent_current_ready_is_not_stale_after_edit,
    independent_ready_requires_current_complete_frame
);

fn diagnostic_f12_only_actual_press() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    assert_eq!(a.evidence().f12_presses, 0);
    keys(&h, &mut a, &[0x57, 0xd7, 0x58, 0x58, 0xd8]);
    assert_eq!(a.evidence().f12_presses, 1);
    keys(&h, &mut a, &[0x58, 0xd8]);
    assert_eq!(a.evidence().f12_presses, 2);
    assert_eq!(a.evidence().save_acks, 0);
    assert_eq!(a.controller().view().screen, Screen::Material);
}
scenario!(
    diagnostic_f12_counts_press_without_repeat_release_or_action,
    diagnostic_f12_only_actual_press
);

fn interactive_batch_reduces_idle_input_requests() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    a.step().unwrap(); // Acknowledge the real initial focus/cancel offer.
    let requests = h.borrow().input_requests;
    let tiles = a.evidence().tiles;
    a.step().unwrap();
    assert_eq!(h.borrow().input_requests - requests, 1);
    assert_eq!(a.evidence().tiles - tiles, 1, "old step contract changed");

    let requests = h.borrow().input_requests;
    let tiles = a.evidence().tiles;
    a.step_interactive().unwrap();
    assert_eq!(h.borrow().input_requests - requests, 1);
    assert_eq!(a.evidence().tiles - tiles, 4);
    assert_eq!(a.evidence().frames, 0);
    assert!(!a.evidence().ready);
}

fn finish_interactive_paint(a: &mut App<'_, Port, Port, Port>) {
    for _ in 0..geometry().tiles() {
        if !a.controller().view().dirty {
            return;
        }
        a.step_interactive().unwrap();
    }
    panic!("interactive frame never completed");
}

fn interactive_ready_requires_complete_current_generation() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    finish_interactive_paint(&mut a);
    assert!(a.evidence().ready);
    assert_eq!(a.evidence().frames, 1);
    assert_eq!(a.evidence().tiles, geometry().tiles());
    assert_eq!(h.borrow().blits as u64, geometry().tiles());
    let old = a.evidence().presented_generation;
    let blits = h.borrow().blits;
    a.step_interactive().unwrap();
    assert_eq!(h.borrow().blits, blits, "clean frames must not repaint");

    h.borrow_mut().key(0x1c);
    let tiles = a.evidence().tiles;
    a.step_interactive().unwrap();
    assert!(a.controller().view().generation > old);
    assert_eq!(a.evidence().tiles - tiles, 4);
    assert!(!a.evidence().ready);
    assert_eq!(a.evidence().frames, 1);
    assert_eq!(a.evidence().presented_generation, old);
    finish_interactive_paint(&mut a);
    assert!(a.evidence().ready);
    assert_eq!(a.evidence().frames, 2);
    assert_eq!(
        a.evidence().presented_generation,
        a.controller().view().generation
    );
    assert_eq!(
        h.borrow().surface.completed_frame(),
        Some(a.evidence().presented_generation)
    );
}

fn interactive_missing_ack_stops_the_batch() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    new_text(&h, &mut a);
    let tiles = a.evidence().tiles;
    let blits = h.borrow().blits;
    h.borrow_mut().drop_blit = true;
    assert!(matches!(a.step_interactive(), Err(Error::Surface(_))));
    assert_eq!(
        h.borrow().blits - blits,
        1,
        "batch continued after lost ACK"
    );
    assert_eq!(a.evidence().tiles, tiles);
    assert_eq!(a.evidence().frames, 0);
    assert!(!a.evidence().ready);
    assert!(a.controller().view().dirty);
    assert_eq!(a.controller().view().contents, Some("hi"));
    let blits = h.borrow().blits;
    a.step_interactive().unwrap();
    assert_eq!(h.borrow().blits, blits);
}

fn interactive_input_batch_remains_four_offers() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    a.step().unwrap();
    for scan in [0x1e, 0x9e, 0x30, 0xb0, 0x2e, 0xae] {
        h.borrow_mut().key(scan);
    }
    let before = a.evidence();
    let requests = h.borrow().input_requests;
    a.step_interactive().unwrap();
    assert_eq!(a.evidence().keys - before.keys, 4);
    assert_eq!(a.evidence().input_acks - before.input_acks, 4);
    assert_eq!(h.borrow().input_requests - requests, 8);
    assert_eq!(a.evidence().tiles - before.tiles, 4);
    let before = a.evidence();
    a.step_interactive().unwrap();
    assert_eq!(a.evidence().keys - before.keys, 2);
    assert_eq!(a.evidence().input_acks - before.input_acks, 2);
}

fn interactive_input_failure_preserves_draft_and_renders_bounded_feedback() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    new_text(&h, &mut a);
    h.borrow_mut().drop_input = true;
    let before = a.evidence();
    assert!(matches!(a.step_interactive(), Err(Error::Input(_))));
    assert!(a.input_failed());
    assert!(!a.evidence().ready);
    assert_eq!(a.evidence().tiles - before.tiles, 4);
    assert_eq!(a.evidence().save_acks, before.save_acks);
    assert_eq!(a.controller().view().contents, Some("hi"));
    assert!(a.controller().document().is_dirty());
}

scenario!(
    interactive_four_tiles_use_one_idle_peek,
    interactive_batch_reduces_idle_input_requests
);
scenario!(
    interactive_batch_never_certifies_partial_generation,
    interactive_ready_requires_complete_current_generation
);
scenario!(
    interactive_lost_tile_ack_ends_batch_and_preserves_draft,
    interactive_missing_ack_stops_the_batch
);
scenario!(
    interactive_input_still_processes_at_most_four_offers,
    interactive_input_batch_remains_four_offers
);
scenario!(
    interactive_input_failure_keeps_draft_and_bounded_paint,
    interactive_input_failure_preserves_draft_and_renders_bounded_feedback
);

// Replacement services use genuinely fresh registered principals and endpoint
// objects. No old session is cloned or granted aliases to its active channels.
#[derive(Clone, Copy)]
struct Replacement {
    pids: [u64; 3], // compositor, source, policy
    input_epoch: u64,
    geometry: Geometry,
}
fn replacement_actor(h: &mut Host, r: Replacement, source: bool, op: Operation, a: u64) {
    let (index, pid, channel, id) = if source {
        h.source_id += 1;
        (0, r.pids[1], h.source_send, h.source_id)
    } else {
        h.control_id += 1;
        (1, r.pids[0], h.control_send, h.control_id)
    };
    let mut m = Message::new();
    m.tag = iw::TAG;
    m.len = iw::SIZE as u32;
    m.data[..iw::SIZE].copy_from_slice(
        &Request {
            operation: op,
            id,
            epoch: r.input_epoch,
            arg0: a,
            arg1: 0,
        }
        .encode(),
    );
    h.engine.send(pid, channel, &m).unwrap();
    let actual = h.engine.receive(r.pids[2], h.roots[index]).unwrap();
    let actual = Received::from_bytes(actual.encode(), actual.len as u64).unwrap();
    h.input.handle(recv(h.roots[index]), &actual).unwrap();
}
fn replace_services(
    h: &Rc<RefCell<Host>>,
    old: [u64; 3],
    next: u64,
    input_epoch: u64,
    mut g: Geometry,
) -> Replacement {
    let mut h = h.borrow_mut();
    for pid in old {
        h.engine.exit(pid).unwrap();
    }
    let pids = [next, next + 1, next + 2];
    for pid in pids {
        h.engine.register(pid).unwrap();
    }
    let roots = core::array::from_fn(|_| h.engine.create_endpoint(pids[2]).unwrap());
    h.source_send = h
        .engine
        .bootstrap_delegate(pids[2], roots[0], pids[1], SEND)
        .unwrap();
    h.control_send = h
        .engine
        .bootstrap_delegate(pids[2], roots[1], pids[0], SEND)
        .unwrap();
    let input_pair = pair(&mut h.engine, pids[2], Some(roots[3]));
    let surface_pair = pair(&mut h.engine, pids[0], None);
    h.pairs[1] = input_pair;
    h.pairs[2] = surface_pair;
    h.roots = roots;
    h.source_id = 0;
    h.control_id = 0;
    h.input = input::Service::new(
        input::Configuration {
            epoch: input_epoch,
            source: input::Actor {
                endpoint: recv(roots[0]),
                pid: pids[1],
            },
            compositor: input::Actor {
                endpoint: recv(roots[1]),
                pid: pids[0],
            },
            supervisor: input::Actor {
                endpoint: recv(roots[2]),
                pid: INIT,
            },
            applications: recv(roots[3]),
            width: 800,
            height: 600,
        },
        &[input::ClientBinding { id: 101, pid: APP }],
    )
    .unwrap();
    g.epoch = SURFACE_EPOCH + next;
    h.surface = surface::Receiver::new(APP, g, 100).unwrap();
    let r = Replacement {
        pids,
        input_epoch,
        geometry: g,
    };
    replacement_actor(&mut h, r, true, Operation::Sync, 1);
    replacement_actor(&mut h, r, false, Operation::Focus, 101);
    assert!(h.engine.check_invariants());
    r
}
fn replacement_channels(
    h: &Rc<RefCell<Host>>,
    r: Replacement,
    epoch: u64,
) -> (Input<Port>, Surface<Port>, Geometry) {
    let pairs = h.borrow().pairs;
    let input = Input::new(
        Port(h.clone(), 1),
        Binding {
            epoch,
            service_pid: r.pids[2],
            client_id: 101,
            width: 800,
            height: 600,
            send: send(pairs[1].app_send),
            receive: recv(pairs[1].app_receive),
        },
        4,
    )
    .unwrap();
    let mut surface = Surface::new(
        Port(h.clone(), 2),
        r.pids[0],
        r.geometry.epoch,
        send(pairs[2].app_send),
        recv(pairs[2].app_receive),
        4,
    )
    .unwrap();
    let geometry = surface.geometry().unwrap();
    (input, surface, geometry)
}
fn assert_graphics_closed(h: &Host, pairs: [Pair; 3]) {
    for p in &pairs[1..] {
        assert!(h.engine.check_endpoint(APP, p.app_send, SEND).is_err());
        assert!(h
            .engine
            .check_endpoint(APP, p.app_receive, capability_core::RECV)
            .is_err());
    }
}
fn stale_wire_epochs_fail_on_new_channels(h: &Rc<RefCell<Host>>, r: Replacement) {
    let mut h = h.borrow_mut();
    let mut m = Message::new();
    m.tag = iw::TAG;
    m.len = iw::SIZE as u32;
    m.data[..iw::SIZE].copy_from_slice(
        &Request {
            operation: Operation::Peek,
            id: 1,
            epoch: INPUT_EPOCH,
            arg0: 0,
            arg1: 0,
        }
        .encode(),
    );
    let p = h.pairs[1];
    h.engine.send(APP, p.app_send, &m).unwrap();
    let actual = h.engine.receive(r.pids[2], p.ingress).unwrap();
    let actual = Received::from_bytes(actual.encode(), actual.len as u64).unwrap();
    assert!(matches!(
        h.input.handle(recv(p.ingress), &actual),
        Err(input::Error::WrongEpoch)
    ));
    let mut m = Message::new();
    m.tag = surface::TAG;
    m.len = surface::HEADER as u32;
    m.data[..surface::HEADER].copy_from_slice(
        &surface::Header {
            operation: surface::Operation::Info,
            request: 1,
            epoch: SURFACE_EPOCH,
            frame: 0,
            tile: 0,
            arg0: 0,
            arg1: 0,
        }
        .encode(),
    );
    let p = h.pairs[2];
    h.engine.send(APP, p.app_send, &m).unwrap();
    let actual = h.engine.receive(r.pids[0], p.ingress).unwrap();
    let now = h.now;
    assert!(matches!(
        h.surface
            .accept(actual.sender_pid, now, &actual.data[..actual.len as usize]),
        Err(surface::Error::Epoch)
    ));
}
fn finish_rebound(a: &mut App<'_, Port, Port, Port>) {
    // A live upload can produce APPEND and COMMIT UI invalidations in the first
    // two steps. Budget those transitions in addition to one entire new frame.
    for _ in 0..geometry().tiles() + 16 {
        a.step().unwrap();
        if a.evidence().ready {
            return;
        }
    }
    panic!("replacement never acknowledged the complete current frame");
}
fn rebind_preserves_document_and_pending_upload() {
    for saving in [false, true] {
        let h = Host::new();
        let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
        let mut a = app(
            &h,
            Buffers {
                draft: &mut d,
                original: &mut o,
                staging: &mut s,
            },
        );
        new_text(&h, &mut a);
        keys(&h, &mut a, &[0x4b, 0xcb]); // nontrivial caret retained across replacement
        finish_rebound(&mut a);
        assert!(a.evidence().ready);
        if saving {
            let generation = a.controller().view().generation;
            a.activate_semantic(nieuw_os_materiaal::render::SemanticId::Save, generation)
                .unwrap();
        }
        assert_eq!(a.controller().document().is_saving(), saving);
        let before = a.controller().view();
        let caret = before.caret;
        let anchor = before.anchor;
        let draft = a.controller().document().draft().unwrap();
        let object = draft.object;
        let revision = draft.loaded_revision;
        let name = draft.name.to_vec();
        let old = h.borrow().pairs;
        let requests = h.borrow().requests.len();
        a.detach_presentation().unwrap();
        assert!(!a.evidence().ready);
        assert_graphics_closed(&h.borrow(), old);
        let r = replace_services(&h, [COMP, SOURCE, POLICY], 7, INPUT_EPOCH + 1, geometry());
        stale_wire_epochs_fail_on_new_channels(&h, r);
        let (input, surface, g) = replacement_channels(&h, r, r.input_epoch);
        a.rebind_presentation(input, surface, g).unwrap();
        let view = a.controller().view();
        let draft = a.controller().document().draft().unwrap();
        assert_eq!(view.contents, Some("hi"));
        assert_eq!((view.caret, view.anchor), (caret, anchor));
        assert_eq!(
            (draft.object, draft.loaded_revision, draft.name),
            (object, revision, name.as_slice())
        );
        assert!(a.controller().document().is_dirty());
        assert_eq!(a.controller().document().is_saving(), saving);
        assert!(!a.controller().document().is_closed());
        assert!(!a.evidence().ready);
        assert_eq!(a.evidence().presented_generation, 0);
        assert_eq!(h.borrow().requests.len(), requests);
        assert_eq!(h.borrow().pairs[0].app_send, old[0].app_send);
        assert_eq!(h.borrow().pairs[0].app_receive, old[0].app_receive);
        a.step().unwrap();
        assert!(!a.evidence().ready);
        if !saving {
            let generation = a.controller().view().generation;
            a.activate_semantic(nieuw_os_materiaal::render::SemanticId::Save, generation)
                .unwrap();
        }
        finish_rebound(&mut a);
        assert!(a.evidence().ready);
        assert_eq!(
            h.borrow().surface.completed_frame(),
            Some(a.controller().view().generation)
        );
        assert_eq!(a.evidence().save_acks, 1);
        assert_eq!(h.borrow_mut().contents(), b"hi");
        let h = h.borrow();
        assert_eq!(
            h.requests
                .iter()
                .filter(|r| r.operation == volume::wire::BEGIN)
                .count(),
            1
        );
        assert_eq!(
            h.requests
                .iter()
                .filter(|r| r.operation == volume::wire::COMMIT)
                .count(),
            1
        );
        assert!(h.engine.check_invariants());
    }
}
scenario!(
    fresh_presentation_preserves_draft_caret_and_active_document_upload,
    rebind_preserves_document_and_pending_upload
);

fn rejected_rebind_closes_only_new_channels_and_preserves_draft() {
    for failure in 0..3 {
        let h = Host::new();
        let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
        let mut a = configured_app(
            &h,
            Buffers {
                draft: &mut d,
                original: &mut o,
                staging: &mut s,
            },
            Scale::Percent200,
            EditMode::ReadWrite,
        );
        new_text(&h, &mut a);
        let caret = a.controller().view().caret;
        a.detach_presentation().unwrap();
        let mut g = geometry();
        if failure == 2 {
            g.assigned.width = 320;
            g.assigned.height = 240;
        }
        let r = replace_services(&h, [COMP, SOURCE, POLICY], 7, INPUT_EPOCH + 1, g);
        let new = h.borrow().pairs;
        let (input, surface, mut actual) = replacement_channels(
            &h,
            r,
            if failure == 0 {
                INPUT_EPOCH
            } else {
                r.input_epoch
            },
        );
        if failure == 1 {
            actual.epoch = SURFACE_EPOCH;
        }
        assert!(matches!(
            a.rebind_presentation(input, surface, actual),
            Err(Error::Binding)
        ));
        assert_graphics_closed(&h.borrow(), new);
        assert_eq!(a.controller().view().contents, Some("hi"));
        assert_eq!(a.controller().view().caret, caret);
        assert!(a.controller().document().is_dirty());
        assert!(!a.evidence().ready);
        assert!(!a.controller().document().is_closed());
        let r = replace_services(&h, r.pids, 10, INPUT_EPOCH + 2, geometry());
        let (input, surface, g) = replacement_channels(&h, r, r.input_epoch);
        a.rebind_presentation(input, surface, g).unwrap();
        finish_rebound(&mut a);
        assert!(a.evidence().ready);
        assert_eq!(a.controller().view().contents, Some("hi"));
    }
}
scenario!(
    stale_or_small_rebind_closes_new_grants_without_losing_document,
    rejected_rebind_closes_only_new_channels_and_preserves_draft
);

fn lost_final_replacement_ack_never_certifies_preserved_document() {
    let h = Host::new();
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut a = app(
        &h,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    new_text(&h, &mut a);
    finish_paint(&mut a);
    let frames = a.evidence().frames;
    let tiles = a.evidence().tiles;
    a.detach_presentation().unwrap();
    let r = replace_services(&h, [COMP, SOURCE, POLICY], 7, INPUT_EPOCH + 1, geometry());
    let (input, surface, g) = replacement_channels(&h, r, r.input_epoch);
    a.rebind_presentation(input, surface, g).unwrap();
    for _ in 0..geometry().tiles() - 1 {
        a.step().unwrap();
        assert!(!a.evidence().ready);
    }
    assert_eq!(a.evidence().tiles - tiles, geometry().tiles() - 1);
    assert_eq!(h.borrow().surface.completed_frame(), None);
    h.borrow_mut().drop_blit = true;
    assert!(matches!(a.step(), Err(Error::Surface(_))));
    // The new Receiver committed every pixel, but the app did not receive the
    // last acknowledgement. A complete framebuffer is not a confirmed frame.
    assert_eq!(
        h.borrow().surface.completed_frame(),
        Some(a.controller().view().generation)
    );
    assert!(!a.evidence().ready);
    assert_eq!(a.evidence().presented_generation, 0);
    assert_eq!(a.evidence().frames, frames);
    assert_eq!(a.evidence().tiles - tiles, geometry().tiles() - 1);
    assert_eq!(a.controller().view().contents, Some("hi"));
    assert!(a.controller().document().is_dirty());
    assert_eq!(a.evidence().save_acks, 0);
}
scenario!(
    lost_final_replacement_frame_ack_preserves_draft_without_ready,
    lost_final_replacement_ack_never_certifies_preserved_document
);

fn externally_retired_presentation_detaches_without_changing_document_ownership() {
    for input_already_closed in [false, true] {
        let h = Host::new();
        let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
        let mut a = app(
            &h,
            Buffers {
                draft: &mut d,
                original: &mut o,
                staging: &mut s,
            },
        );
        new_text(&h, &mut a);
        finish_paint(&mut a);
        if input_already_closed {
            h.borrow_mut().drop_input = true;
            assert!(matches!(a.step(), Err(Error::Input(_))));
        }
        let old = h.borrow().pairs;
        let requests = h.borrow().requests.len();
        let caret = a.controller().view().caret;
        {
            let mut host = h.borrow_mut();
            for pair in &old[1..] {
                for handle in [pair.app_send, pair.app_receive] {
                    match host.engine.close(APP, handle) {
                        Ok(()) | Err(capability_core::Error::BadHandle) => {}
                        other => panic!("unexpected manifest retirement: {other:?}"),
                    }
                }
            }
        }
        a.detach_presentation().unwrap();
        assert!(!a.evidence().ready);
        assert_eq!(a.controller().view().contents, Some("hi"));
        assert_eq!(a.controller().view().caret, caret);
        assert!(a.controller().document().is_dirty());
        assert!(!a.controller().document().is_closed());
        assert_eq!(a.evidence().save_acks, 0);
        let host = h.borrow();
        assert_eq!(host.requests.len(), requests);
        assert!(host
            .engine
            .check_endpoint(APP, old[0].app_send, SEND)
            .is_ok());
        assert!(host
            .engine
            .check_endpoint(APP, old[0].app_receive, capability_core::RECV)
            .is_ok());
        assert_graphics_closed(&host, old);
        assert!(host.engine.check_invariants());
    }
}
scenario!(
    retired_reply_leaves_preserve_document_and_allow_both_owner_pause,
    externally_retired_presentation_detaches_without_changing_document_ownership
);

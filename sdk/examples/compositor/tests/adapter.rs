//! Host fault/codec tests, with real capability-engine sender metadata. The
//! display transport below ALWAYS fails writes; it proves no native rendering.
use capability_core::{Engine, Message, SEND};
use nieuw_os_display_client::{
    wire as display, DisplayClient, DisplaySource, InputSource, Transport,
};
use nieuw_os_native_compositor::{
    boot::Bootstrap,
    chrome::Chrome,
    control::{self, Configuration, Coordinator, Operation, Request, State, Status},
    surface::{Fatal, Surface},
};
use nieuw_os_native_input_service::Ipc;
use nieuw_os_sdk_runtime::{
    Capability, Error, Handle, Outgoing, ReceiveEndpoint, Received, Rights, SendEndpoint,
};
use nieuw_os_surface_protocol::{self as sur, Header};
use std::{cell::RefCell, rc::Rc};
fn handle(raw: u64) -> Handle {
    Handle::from_raw(raw).unwrap()
}
fn recv(raw: u64) -> ReceiveEndpoint {
    ReceiveEndpoint::from_handle(handle(raw))
}
fn send(raw: u64) -> SendEndpoint {
    SendEndpoint::from_handle(handle(raw))
}
fn number(b: &[u8], i: usize) -> u64 {
    u64::from_le_bytes(b[i..i + 8].try_into().unwrap())
}
struct Port {
    pid: u64,
    engine: Rc<RefCell<Box<Engine>>>,
    sends: usize,
    reads: usize,
    closed: Vec<u64>,
}
impl Ipc for Port {
    fn send(&mut self, e: SendEndpoint, m: &Outgoing) -> Result<(), Error> {
        self.sends += 1;
        self.engine
            .borrow_mut()
            .send(
                self.pid,
                e.handle().raw(),
                &Message::decode(m.as_bytes()).unwrap(),
            )
            .map_err(|e| Error::Kernel(e.code()))
    }
    fn receive(&mut self, e: ReceiveEndpoint) -> Result<Received, Error> {
        self.reads += 1;
        let m = self
            .engine
            .borrow_mut()
            .receive(self.pid, e.handle().raw())
            .map_err(|e| Error::Kernel(e.code()))?;
        Received::from_bytes(m.encode(), u64::from(m.len))
    }
    fn close(&mut self, h: Handle) -> Result<(), Error> {
        self.closed.push(h.raw());
        self.engine
            .borrow_mut()
            .close(self.pid, h.raw())
            .map_err(|e| Error::Kernel(e.code()))
    }
}
struct Model {
    engine: Rc<RefCell<Box<Engine>>>,
    ports: Vec<Port>,
    surface: u64,
    app_send: u64,
    app_reply: u64,
    comp_reply: u64,
    control: u64,
    init_send: u64,
    focus_send: u64,
    policy_recv: u64,
    focus_recv: u64,
    policy_send: u64,
    registry_recv: u64,
    registry_send: u64,
}
impl Model {
    fn new() -> Self {
        let mut e = Box::new(Engine::new());
        for p in [1, 3, 5, 7, 9] {
            e.register(p).unwrap();
        }
        let surface = e.create_endpoint(5).unwrap();
        let app_send = e.bootstrap_delegate(5, surface, 3, SEND).unwrap();
        let app_reply = e.create_endpoint(3).unwrap();
        let comp_reply = e.bootstrap_delegate(3, app_reply, 5, SEND).unwrap();
        let control = e.create_endpoint(5).unwrap();
        let init_send = e.bootstrap_delegate(5, control, 1, SEND).unwrap();
        let policy_recv = e.create_endpoint(7).unwrap();
        let focus_send = e.bootstrap_delegate(7, policy_recv, 5, SEND).unwrap();
        let focus_recv = e.create_endpoint(5).unwrap();
        let policy_send = e.bootstrap_delegate(5, focus_recv, 7, SEND).unwrap();
        let registry_recv = e.create_endpoint(1).unwrap();
        let registry_send = e.bootstrap_delegate(1, registry_recv, 5, SEND).unwrap();
        let engine = Rc::new(RefCell::new(e));
        let ports = [1, 3, 5, 7, 9]
            .into_iter()
            .map(|pid| Port {
                pid,
                engine: engine.clone(),
                sends: 0,
                reads: 0,
                closed: vec![],
            })
            .collect();
        Self {
            engine,
            ports,
            surface,
            app_send,
            app_reply,
            comp_reply,
            control,
            init_send,
            focus_send,
            policy_recv,
            focus_recv,
            policy_send,
            registry_recv,
            registry_send,
        }
    }
    fn coordinator(&self) -> Coordinator {
        Coordinator::new(Configuration {
            init_pid: 1,
            surface_epoch: 11,
            generation: 2,
            policy_pid: 7,
            policy_epoch: 70,
            client_id: 101,
            send: send(self.focus_send),
            receive: recv(self.focus_recv),
        })
        .unwrap()
    }
    fn control(&mut self, operation: Operation, id: u64) -> Received {
        let m = Outgoing::new(
            control::TAG,
            &Request {
                operation,
                id,
                epoch: 11,
                generation: 2,
            }
            .encode(),
        )
        .unwrap();
        self.ports[0].send(send(self.init_send), &m).unwrap();
        self.ports[2].receive(recv(self.control)).unwrap()
    }
    fn ack(&mut self, status: u16) {
        let msg = self.ports[3].receive(recv(self.policy_recv)).unwrap();
        let mut b = msg.payload().to_vec();
        b[6..8].copy_from_slice(
            &(0x8000 | nieuw_os_input_core::wire::Operation::Focus as u16).to_le_bytes(),
        );
        b[24..].fill(0);
        b[24..26].copy_from_slice(&status.to_le_bytes());
        self.ports[3]
            .send(
                send(self.policy_send),
                &Outgoing::new(nieuw_os_input_core::wire::TAG, &b).unwrap(),
            )
            .unwrap();
    }
    fn surface(&self) -> (Surface<FailDisplay>, Rc<RefCell<Vec<display::Blit>>>) {
        let calls = Rc::new(RefCell::new(vec![]));
        let mut d = DisplayClient::new(
            FailDisplay(calls.clone()),
            DisplaySource::from_handle(handle(0x100000001)),
        );
        let g = d.geometry().unwrap();
        (
            Surface::new(d, g, 11, 3, recv(self.surface), send(self.comp_reply)).unwrap(),
            calls,
        )
    }
    fn push(&mut self, b: &[u8]) {
        self.ports[1]
            .send(send(self.app_send), &Outgoing::new(sur::TAG, b).unwrap())
            .unwrap();
    }
    fn stage(&mut self, s: &mut Surface<FailDisplay>, pixels: &[u8; 1024]) {
        self.push(&begin());
        s.step(0, &mut self.ports[2], || Ok(0)).unwrap();
        for (i, part) in pixels.chunks(sur::CHUNK).enumerate() {
            let mut b = h(sur::Operation::Chunk).encode().to_vec();
            b[40..44].copy_from_slice(&((i * sur::CHUNK) as u32).to_le_bytes());
            b[44..48].copy_from_slice(&(part.len() as u32).to_le_bytes());
            b.extend(part);
            self.push(&b);
            s.step(0, &mut self.ports[2], || Ok(0)).unwrap();
        }
    }
}
fn g() -> display::Geometry {
    display::Geometry {
        width: 800,
        height: 600,
        epoch: 55,
    }
}
struct FailDisplay(Rc<RefCell<Vec<display::Blit>>>);
impl Transport for FailDisplay {
    fn display_info(&mut self, _: DisplaySource, out: &mut [u8; 32]) -> Result<u64, Error> {
        *out = g().encode().unwrap();
        Ok(32)
    }
    fn display_blit(&mut self, _: DisplaySource, d: &[u8; 32], p: &[u8]) -> Result<u64, Error> {
        let d = display::Blit::decode(d).unwrap();
        d.validate(g(), p.len()).unwrap();
        display::validate_pixels(p).unwrap();
        self.0.borrow_mut().push(d);
        Err(Error::Kernel(-6))
    }
    fn input_reset_begin(&mut self, _: InputSource) -> Result<u64, Error> {
        panic!("no raw input in compositor")
    }
    fn input_reset_poll(&mut self, _: InputSource, _: u64, _: &mut [u8; 32]) -> Result<u64, Error> {
        panic!("no raw input")
    }
    fn input_read(&mut self, _: InputSource, _: &mut [u8; 32]) -> Result<u64, Error> {
        panic!("no raw input")
    }
}
fn h(operation: sur::Operation) -> Header {
    Header {
        operation,
        request: 1,
        epoch: 11,
        frame: 1,
        tile: 0,
        arg0: 0,
        arg1: 0,
    }
}
fn begin() -> Vec<u8> {
    let mut b = h(sur::Operation::Begin).encode().to_vec();
    b[40..44].copy_from_slice(&0u32.to_le_bytes());
    b[44..48].copy_from_slice(&0u32.to_le_bytes());
    b.extend(32u16.to_le_bytes());
    b.extend(8u16.to_le_bytes());
    b.extend(1024u32.to_le_bytes());
    b
}
fn boot() -> [u64; 128] {
    let mut w = [0; 128];
    for (o, v) in [
        (32, nieuw_os_native_compositor::boot::MAGIC),
        (40, 2),
        (48, 0x100000001),
        (56, 0x100000002),
        (64, 0x100000003),
        (72, 3),
        (80, 11),
        (88, 0x100000004),
        (96, 0x100000005),
        (104, 7),
        (112, 70),
        (120, 0x100000006),
        (128, 0x100000007),
        (136, 1),
        (144, 101),
    ] {
        w[o / 8] = v;
    }
    w
}
#[test]
fn bootstrap_bindings_reserved_tail_and_epoch_domains() {
    let w = boot();
    let b = Bootstrap::decode(&w).unwrap();
    let ready = b.ready(55).unwrap();
    assert_eq!(number(&ready.as_bytes()[64..96], 8), 11);
    assert_eq!(number(&ready.as_bytes()[64..96], 16), 2);
    assert_eq!(number(&ready.as_bytes()[64..96], 24), 55);
    assert!(b.ready(0).is_err());
    for at in [40, 72, 80, 104, 112, 136, 144] {
        let mut bad = w;
        bad[at / 8] = 0;
        assert!(Bootstrap::decode(&bad).is_err());
    }
    let mut bad = w;
    bad[19] = 1;
    assert!(Bootstrap::decode(&bad).is_err());
}
#[test]
fn trusted_tiles_cover_only_border_and_wipe_only_app() {
    let c = Chrome::new(g()).unwrap();
    let r = c.geometry(11).unwrap().assigned;
    let mut coverage = vec![0u8; 800 * 600];
    for t in c.tiles() {
        let mut b = [0xa5; 1024];
        let n = c.render(t, State::Focused, &mut b).unwrap();
        assert!(b[..n].chunks_exact(4).all(|p| p[3] == 0));
        assert!(b[n..].iter().all(|x| *x == 0xa5));
        for y in t.y..t.y + u32::from(t.height) {
            for x in t.x..t.x + u32::from(t.width) {
                coverage[(y * 800 + x) as usize] += 1;
            }
        }
    }
    for t in c.wipe_tiles() {
        let mut b = [0xa5; 1024];
        let n = c.render_wipe(t, &mut b).unwrap();
        assert!(b[n..].iter().all(|x| *x == 0xa5));
        for y in t.y..t.y + u32::from(t.height) {
            for x in t.x..t.x + u32::from(t.width) {
                assert!(
                    x >= r.x
                        && y >= r.y
                        && x < r.x + u32::from(r.width)
                        && y < r.y + u32::from(r.height)
                );
                coverage[(y * 800 + x) as usize] += 1;
            }
        }
    }
    assert!(coverage.iter().all(|x| *x == 1));
    let mut b = [0xa5; 1024];
    assert!(c
        .render(
            display::Rectangle {
                x: 16,
                y: 48,
                width: 32,
                height: 8
            },
            State::Inactive,
            &mut b
        )
        .is_err());
    assert_eq!(b, [0xa5; 1024]);
    assert!(Chrome::new(display::Geometry { width: 351, ..g() }).is_err());
}
fn control_wait_impl() {
    let mut m = Model::new();
    let mut c = m.coordinator();
    let msg = m.control(Operation::FocusStart, 1);
    c.accept(0, &msg).unwrap();
    for now in 0..100 {
        assert!(!c.step(now, &mut m.ports[2]).unwrap());
    }
    assert_eq!(c.counts(), (1, 0));
    assert_eq!(c.state(), State::Inactive);
    assert_eq!(c.step(201, &mut m.ports[2]), Err(Status::Uncertain));
    assert!(c.is_terminal());
    assert_eq!(c.counts(), (1, 0));
}
fn control_ack_impl() {
    let mut m = Model::new();
    let mut c = m.coordinator();
    let msg = m.control(Operation::FocusStart, 1);
    c.accept(0, &msg).unwrap();
    c.step(0, &mut m.ports[2]).unwrap();
    m.ack(0);
    assert!(c.step(1, &mut m.ports[2]).unwrap());
    assert_eq!(c.state(), State::Focused);
    assert_eq!(c.counts(), (1, 1));
    let filler = Outgoing::new(1, &[]).unwrap();
    for _ in 0..16 {
        m.ports[2].send(send(m.registry_send), &filler).unwrap();
    }
    c.flush(2, send(m.registry_send), &mut m.ports[2]).unwrap();
    assert!(c.is_busy());
    for _ in 0..16 {
        m.ports[0].receive(recv(m.registry_recv)).unwrap();
    }
    c.flush(3, send(m.registry_send), &mut m.ports[2]).unwrap();
    let reply = m.ports[0].receive(recv(m.registry_recv)).unwrap();
    assert_eq!(reply.sender(), 5);
    assert_eq!(number(reply.payload(), 32), 0);
    assert_eq!(number(reply.payload(), 40), 1);
    assert_eq!(c.accept(4, &msg), Err(Status::Replay));
}
fn bad_ack_impl() {
    let mut m = Model::new();
    let mut c = m.coordinator();
    let msg = m.control(Operation::FocusStart, 1);
    c.accept(0, &msg).unwrap();
    c.step(0, &mut m.ports[2]).unwrap();
    m.ack(2);
    assert_eq!(c.step(1, &mut m.ports[2]), Err(Status::Input));
    assert_eq!(c.state(), State::Inactive);
    assert_eq!(c.counts(), (1, 0));
    assert!(m.ports[0]
        .receive(recv(m.registry_recv))
        .unwrap_err()
        .is_again());
}
fn failed_blit_impl() {
    let mut m = Model::new();
    let (mut s, calls) = m.surface();
    m.stage(&mut s, &[0; 1024]);
    m.push(&h(sur::Operation::Commit).encode());
    assert_eq!(s.step(0, &mut m.ports[2], || Ok(0)), Err(Fatal::Display));
    let reply = m.ports[1].receive(recv(m.app_reply)).unwrap();
    assert_eq!(
        sur::Reply::decode(reply.payload(), h(sur::Operation::Commit))
            .unwrap()
            .status(),
        Err(sur::Error::Blit)
    );
    assert_eq!(calls.borrow().len(), 1);
    assert_eq!(s.counts().committed, 0);
    assert_eq!(s.counts().pixels, 0);
    assert_eq!(s.completed_frame(), None);
}
fn fresh_expiry_impl() {
    let mut m = Model::new();
    let (mut s, calls) = m.surface();
    m.stage(&mut s, &[0; 1024]);
    m.push(&h(sur::Operation::Commit).encode());
    s.step(0, &mut m.ports[2], || Ok(201)).unwrap();
    let reply = m.ports[1].receive(recv(m.app_reply)).unwrap();
    assert_eq!(
        sur::Reply::decode(reply.payload(), h(sur::Operation::Commit))
            .unwrap()
            .status(),
        Err(sur::Error::Timeout)
    );
    assert!(calls.borrow().is_empty());
}
fn malformed_pixels_impl() {
    let mut m = Model::new();
    let (mut s, calls) = m.surface();
    let mut b = [0; 1024];
    b[1023] = 1;
    m.stage(&mut s, &b);
    m.push(&h(sur::Operation::Commit).encode());
    s.step(0, &mut m.ports[2], || Ok(0)).unwrap();
    let reply = m.ports[1].receive(recv(m.app_reply)).unwrap();
    assert_eq!(
        sur::Reply::decode(reply.payload(), h(sur::Operation::Commit))
            .unwrap()
            .status(),
        Err(sur::Error::Pixels)
    );
    assert!(calls.borrow().is_empty());
}
fn retire_impl() {
    let mut m = Model::new();
    let (mut s, calls) = m.surface();
    m.stage(&mut s, &[0; 1024]);
    s.retire(&mut m.ports[2]);
    let reads = m.ports[2].reads;
    s.step(300, &mut m.ports[2], || panic!()).unwrap();
    assert_eq!(m.ports[2].reads, reads);
    assert_eq!(m.ports[2].closed.len(), 2);
    s.retire(&mut m.ports[2]);
    assert_eq!(m.ports[2].closed.len(), 2);
    let c = Chrome::new(g()).unwrap();
    assert_eq!(
        s.clear(c, c.wipe_tiles().next().unwrap(), &mut [0; 1024]),
        Err(Fatal::Display)
    );
    assert_eq!(s.counts().cleared, 0);
    assert_eq!(calls.borrow().len(), 1);
}
fn wrong_sender_and_caps_impl() {
    let mut m = Model::new();
    let (mut s, calls) = m.surface();
    let grant = m
        .engine
        .borrow_mut()
        .bootstrap_delegate(5, m.surface, 9, SEND)
        .unwrap();
    m.ports[4]
        .send(send(grant), &Outgoing::new(sur::TAG, &begin()).unwrap())
        .unwrap();
    s.step(0, &mut m.ports[2], || Ok(0)).unwrap();
    assert_eq!(s.counts().rejected, 1);
    assert!(calls.borrow().is_empty());
    let cap = m.engine.borrow_mut().create_endpoint(3).unwrap();
    let mut msg = Outgoing::new(sur::TAG, &begin()).unwrap();
    msg.push_cap(Capability {
        handle: handle(cap),
        rights: Rights::SEND,
    })
    .unwrap();
    m.ports[1].send(send(m.app_send), &msg).unwrap();
    s.step(0, &mut m.ports[2], || Ok(0)).unwrap();
    assert_eq!(m.ports[2].closed.len(), 1);
    assert!(calls.borrow().is_empty());
    assert_eq!(s.counts().committed, 0);
}
macro_rules! scenario {
    ($name:ident,$body:ident) => {
        #[test]
        fn $name() {
            std::thread::Builder::new()
                .stack_size(16 * 1024 * 1024)
                .spawn($body)
                .unwrap()
                .join()
                .unwrap();
        }
    };
}
scenario!(
    focus_publication_is_not_replayed_and_deadline_never_focuses,
    control_wait_impl
);
scenario!(
    actual_sender_ack_then_registry_backpressure_preserves_reply,
    control_ack_impl
);
scenario!(negative_focus_ack_never_reports_focused, bad_ack_impl);
scenario!(
    real_adapter_display_error_never_positive_ack,
    failed_blit_impl
);
scenario!(
    fresh_commit_clock_expires_before_display_write,
    fresh_expiry_impl
);
scenario!(
    noncanonical_pixels_never_reach_display,
    malformed_pixels_impl
);
scenario!(
    retire_closes_routes_once_no_receive_and_failed_clear_not_counted,
    retire_impl
);
scenario!(
    real_engine_wrong_sender_and_transferred_caps_do_not_blit,
    wrong_sender_and_caps_impl
);
fn control_binding_impl() {
    let mut m = Model::new();
    let mut c = m.coordinator();
    let grant = m
        .engine
        .borrow_mut()
        .bootstrap_delegate(5, m.control, 9, SEND)
        .unwrap();
    let good = Request {
        operation: Operation::FocusStart,
        id: 1,
        epoch: 11,
        generation: 2,
    };
    m.ports[4]
        .send(
            send(grant),
            &Outgoing::new(control::TAG, &good.encode()).unwrap(),
        )
        .unwrap();
    let bad = m.ports[2].receive(recv(m.control)).unwrap();
    assert_eq!(c.accept(0, &bad), Err(Status::Access));
    let mut old = good;
    old.generation = 1;
    m.ports[0]
        .send(
            send(m.init_send),
            &Outgoing::new(control::TAG, &old.encode()).unwrap(),
        )
        .unwrap();
    let bad = m.ports[2].receive(recv(m.control)).unwrap();
    assert_eq!(c.accept(0, &bad), Err(Status::Binding));
    assert_eq!(c.counts(), (0, 0));
    assert_eq!(c.state(), State::Inactive);
}
fn retired_focus0_impl() {
    let mut m = Model::new();
    let mut c = m.coordinator();
    let msg = m.control(Operation::Retire, 1);
    assert!(c.accept(0, &msg).unwrap());
    c.step(0, &mut m.ports[2]).unwrap();
    let wire = m.ports[3].receive(recv(m.policy_recv)).unwrap();
    let request = nieuw_os_input_core::wire::Request::decode(wire.payload(), 70).unwrap();
    assert_eq!(
        request.operation,
        nieuw_os_input_core::wire::Operation::Focus
    );
    assert_eq!(request.arg0, 0);
    assert_eq!(c.state(), State::Inactive);
    assert_eq!(c.counts(), (1, 0));
    assert_eq!(c.step(201, &mut m.ports[2]), Err(Status::Uncertain));
    assert!(m.ports[0]
        .receive(recv(m.registry_recv))
        .unwrap_err()
        .is_again());
}
fn retire_reply_deadline_starts_after_wipe_impl() {
    let mut m = Model::new();
    let mut c = m.coordinator();
    let msg = m.control(Operation::Retire, 1);
    assert!(c.accept(0, &msg).unwrap());
    c.step(0, &mut m.ports[2]).unwrap();
    m.ack(0);
    assert!(c.step(1, &mut m.ports[2]).unwrap());
    assert_eq!(c.state(), State::Retired);

    // Even after a long wipe, no reply can be sent before completion.
    c.flush(500, send(m.registry_send), &mut m.ports[2])
        .unwrap();
    assert!(m.ports[0]
        .receive(recv(m.registry_recv))
        .unwrap_err()
        .is_again());
    let filler = Outgoing::new(1, &[]).unwrap();
    for _ in 0..16 {
        m.ports[2].send(send(m.registry_send), &filler).unwrap();
    }
    c.wipe_complete(500);
    c.flush(500, send(m.registry_send), &mut m.ports[2])
        .unwrap();
    // Repeated completion calls must not extend the deadline.
    c.wipe_complete(600);
    c.flush(700, send(m.registry_send), &mut m.ports[2])
        .unwrap();
    assert_eq!(
        c.flush(701, send(m.registry_send), &mut m.ports[2]),
        Err(Status::Uncertain)
    );
    assert!(c.is_terminal());

    let mut m = Model::new();
    let mut c = m.coordinator();
    let msg = m.control(Operation::Retire, 1);
    assert!(c.accept(0, &msg).unwrap());
    c.step(0, &mut m.ports[2]).unwrap();
    m.ack(0);
    assert!(c.step(1, &mut m.ports[2]).unwrap());
    c.wipe_complete(500);
    c.flush(500, send(m.registry_send), &mut m.ports[2])
        .unwrap();
    let reply = m.ports[0].receive(recv(m.registry_recv)).unwrap();
    assert_eq!(number(reply.payload(), 32), 0);
    assert_eq!(number(reply.payload(), 40), State::Retired as u64);
}
fn wrong_policy_impl() {
    let mut m = Model::new();
    let mut c = m.coordinator();
    let msg = m.control(Operation::FocusStart, 1);
    c.accept(0, &msg).unwrap();
    c.step(0, &mut m.ports[2]).unwrap();
    let request = m.ports[3].receive(recv(m.policy_recv)).unwrap();
    let mut bytes = request.payload().to_vec();
    bytes[6..8].copy_from_slice(&(0x8000 | 16u16).to_le_bytes());
    bytes[24..].fill(0);
    let attacker = m
        .engine
        .borrow_mut()
        .bootstrap_delegate(5, m.focus_recv, 9, SEND)
        .unwrap();
    m.ports[4]
        .send(
            send(attacker),
            &Outgoing::new(nieuw_os_input_core::wire::TAG, &bytes).unwrap(),
        )
        .unwrap();
    assert_eq!(c.step(1, &mut m.ports[2]), Err(Status::Uncertain));
    assert_eq!(c.counts(), (1, 0));
    assert_eq!(c.state(), State::Inactive);
}
scenario!(
    control_requires_actual_init_pid_and_current_generation,
    control_binding_impl
);
scenario!(
    retire_requests_real_unfocus_and_no_ack_before_reply,
    retired_focus0_impl
);
scenario!(
    retire_reply_deadline_starts_after_wipe,
    retire_reply_deadline_starts_after_wipe_impl
);
scenario!(
    canonical_focus_reply_from_wrong_kernel_sender_is_terminal,
    wrong_policy_impl
);

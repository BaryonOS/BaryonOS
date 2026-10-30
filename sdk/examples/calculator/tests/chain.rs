//! Actual capability/INP1/SUR1 engines in a host process. Recording RAM pixels
//! replace the display counterpart ONLY here, never in calculator-app.
use capability_core::{Engine, Message, RECV, SEND};
use nieuw_os_calculator::{
    app::{App, Error},
    bootstrap::Bootstrap,
    decimal::SCALE,
};
use nieuw_os_input_client::{Binding, Session as Input, Transport};
use nieuw_os_input_core::{
    self as input,
    wire::{self as iw, Operation, Request},
};
use nieuw_os_sdk_runtime::{
    Error as RuntimeError, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};
use nieuw_os_surface_protocol::{self as surface, client::Session as Surface, Geometry, Rectangle};
use std::{cell::RefCell, rc::Rc};
const APP: u64 = 11;
const COMP: u64 = 12;
const SOURCE: u64 = 13;
const POLICY: u64 = 14;
const INIT: u64 = 15;
const IE: u64 = 31;
const SE: u64 = 17;
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
        epoch: SE,
    }
}
#[derive(Clone, Copy)]
struct Pair {
    send: u64,
    receive: u64,
    ingress: u64,
    reply: u64,
    server: u64,
}
fn pair(e: &mut Engine, server: u64, ingress: Option<u64>) -> Pair {
    let ingress = ingress.unwrap_or_else(|| e.create_endpoint(server).unwrap());
    let tx = e.bootstrap_delegate(server, ingress, APP, SEND).unwrap();
    let root = e.create_endpoint(APP).unwrap();
    let rx = e.derive(APP, root, RECV).unwrap();
    let reply = e.bootstrap_delegate(APP, root, server, SEND).unwrap();
    e.close(APP, root).unwrap();
    Pair {
        send: tx,
        receive: rx,
        ingress,
        reply,
        server,
    }
}
struct Host {
    engine: Box<Engine>,
    pairs: [Pair; 2],
    input: input::Service,
    surface: surface::Receiver,
    roots: [u64; 4],
    source_send: u64,
    control_send: u64,
    source_id: u64,
    control_id: u64,
    now: u64,
    blits: usize,
    framebuffer: Vec<u32>,
    closed: Vec<u64>,
    drop_input: bool,
    drop_blit: bool,
    fail_blit: bool,
    fail_close: bool,
}
impl Host {
    fn new() -> Rc<RefCell<Self>> {
        let mut e = Box::new(Engine::new());
        for pid in [APP, COMP, SOURCE, POLICY, INIT] {
            e.register(pid).unwrap();
        }
        let roots = core::array::from_fn(|_| e.create_endpoint(POLICY).unwrap());
        let source_send = e
            .bootstrap_delegate(POLICY, roots[0], SOURCE, SEND)
            .unwrap();
        let control_send = e.bootstrap_delegate(POLICY, roots[1], COMP, SEND).unwrap();
        let pairs = [
            pair(&mut e, POLICY, Some(roots[3])),
            pair(&mut e, COMP, None),
        ];
        let input = input::Service::new(
            input::Configuration {
                epoch: IE,
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
        let h = Rc::new(RefCell::new(Self {
            engine: e,
            pairs,
            input,
            surface: surface::Receiver::new(APP, geometry(), 100).unwrap(),
            roots,
            source_send,
            control_send,
            source_id: 0,
            control_id: 0,
            now: 0,
            blits: 0,
            framebuffer: vec![0; 800 * 600],
            closed: vec![],
            drop_input: false,
            drop_blit: false,
            fail_blit: false,
            fail_close: false,
        }));
        {
            let mut host = h.borrow_mut();
            host.actor(true, Operation::Sync, 1, 0);
            host.actor(false, Operation::Focus, 101, 0);
        }
        h
    }
    fn actor(&mut self, source: bool, op: Operation, a: u64, b: u64) {
        let (index, pid, channel, id) = if source {
            self.source_id += 1;
            (0, SOURCE, self.source_send, self.source_id)
        } else {
            self.control_id += 1;
            (1, COMP, self.control_send, self.control_id)
        };
        let mut message = Message::new();
        message.tag = iw::TAG;
        message.len = iw::SIZE as u32;
        message.data[..iw::SIZE].copy_from_slice(
            &Request {
                operation: op,
                id,
                epoch: IE,
                arg0: a,
                arg1: b,
            }
            .encode(),
        );
        self.engine.send(pid, channel, &message).unwrap();
        let actual = self.engine.receive(POLICY, self.roots[index]).unwrap();
        let received = Received::from_bytes(actual.encode(), actual.len as u64).unwrap();
        self.input
            .handle(recv(self.roots[index]), &received)
            .unwrap();
    }
    fn key(&mut self, byte: u8) {
        self.actor(true, Operation::Sample, 1, u64::from(byte));
    }
    fn mouse(&mut self, x: u32, y: u32, buttons: u8) {
        loop {
            let (cx, cy) = self.input.cursor();
            let dx = (x as i64 - cx as i64).clamp(-127, 127) as i16;
            let dy = (cy as i64 - y as i64).clamp(-127, 127) as i16;
            let flags = 8 | buttons | if dx < 0 { 0x10 } else { 0 } | if dy < 0 { 0x20 } else { 0 };
            for byte in [flags, dx as u8, dy as u8] {
                self.actor(true, Operation::Sample, 0x21, u64::from(byte));
            }
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
        let response = if index == 0 {
            assert_eq!(m.tag, iw::TAG);
            let received = Received::from_bytes(m.encode(), m.len as u64).unwrap();
            let reply = self.input.handle(recv(p.ingress), &received).unwrap();
            assert!(self.input.reply_is_current(&reply));
            if self.drop_input {
                return;
            }
            Some(reply.payload().to_vec())
        } else {
            assert_eq!(m.tag, surface::TAG);
            self.now += 1;
            match self
                .surface
                .accept(m.sender_pid, self.now, &m.data[..m.len as usize])
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
                        .commit_with(token, self.now, |rect, bytes| {
                            if fail {
                                return Err(());
                            }
                            *blits += 1;
                            for (i, chunk) in bytes.chunks_exact(4).enumerate() {
                                let x = rect.x + i as u32 % u32::from(rect.width);
                                let y = rect.y + i as u32 / u32::from(rect.width);
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
        };
        if let Some(response) = response {
            let mut reply = Message::new();
            reply.tag = m.tag;
            reply.len = response.len() as u32;
            reply.data[..response.len()].copy_from_slice(&response);
            self.engine.send(p.server, p.reply, &reply).unwrap();
        }
    }
}
struct Port(Rc<RefCell<Host>>, usize);
impl Transport for Port {
    fn send(&mut self, to: SendEndpoint, message: &Outgoing) -> Result<(), RuntimeError> {
        let mut h = self.0.borrow_mut();
        h.engine
            .send(
                APP,
                to.handle().raw(),
                &Message::decode(message.as_bytes()).unwrap(),
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
            .map_err(|e| RuntimeError::Kernel(e.code()))?;
        if h.fail_close && h.closed.len() == 1 {
            Err(RuntimeError::InvalidReturn)
        } else {
            Ok(())
        }
    }
    fn yield_now(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }
}
fn app(h: &Rc<RefCell<Host>>, scale: u32) -> App<Port, Port> {
    let p = h.borrow().pairs[0];
    let input = Input::new(
        Port(h.clone(), 0),
        Binding {
            epoch: IE,
            service_pid: POLICY,
            client_id: 101,
            width: 800,
            height: 600,
            send: send(p.send),
            receive: recv(p.receive),
        },
        8,
    )
    .unwrap();
    let p = h.borrow().pairs[1];
    let mut surface = Surface::new(
        Port(h.clone(), 1),
        COMP,
        SE,
        send(p.send),
        recv(p.receive),
        8,
    )
    .unwrap();
    let geometry = surface.geometry().unwrap();
    App::new(input, surface, geometry, scale).unwrap()
}
fn finish(a: &mut App<Port, Port>) {
    for _ in 0..geometry().tiles() + 1 {
        a.step().unwrap();
        if a.evidence().ready {
            return;
        }
    }
    panic!("no complete current frame")
}
fn keys(h: &Rc<RefCell<Host>>, a: &mut App<Port, Port>, bytes: &[u8]) {
    for byte in bytes {
        h.borrow_mut().key(*byte);
        a.step().unwrap();
    }
}
#[test]
fn actual_typed_service_chain_computes_from_real_scan_bytes_and_commits_only_current_frames() {
    for scale in [1, 2] {
        let h = Host::new();
        let mut a = app(&h, scale);
        finish(&mut a);
        assert_eq!(a.evidence().frames, 1);
        assert_eq!(a.evidence().tiles, geometry().tiles());
        assert_eq!(
            h.borrow().surface.completed_frame(),
            Some(a.model().generation())
        );
        keys(
            &h,
            &mut a,
            &[0x03, 0x83, 0x2a, 0x0d, 0x8d, 0xaa, 0x04, 0x84, 0x1c, 0x9c],
        );
        assert_eq!(a.model().expression(), "2+3");
        assert_eq!(a.model().answer().unwrap().value.raw(), 5 * SCALE);
        assert!(!a.evidence().ready);
        finish(&mut a);
        let host = h.borrow();
        assert_eq!(host.framebuffer[0], 0);
        assert_ne!(host.framebuffer[(48 * 800 + 16) as usize], 0);
        assert_eq!(host.engine.stats().block_objects, 0);
        assert_eq!(host.engine.stats().entropy_objects, 0);
        assert_eq!(host.engine.stats().network_objects, 0);
        assert!(host.engine.check_invariants());
        drop(host);
        a.stop_clean().unwrap();
        assert_eq!(h.borrow().closed.len(), 4);
    }
}
#[test]
fn actual_outside_mouse_cancel_and_focus_generation_cannot_activate_a_captured_button() {
    let h = Host::new();
    let mut a = app(&h, 1);
    finish(&mut a);
    let layout = nieuw_os_calculator::render::Layout::new(a.viewport());
    let button = layout.button(0);
    let (x, y) = (
        16 + button.x + button.width / 2,
        48 + button.y + button.height / 2,
    );
    h.borrow_mut().mouse(x, y, 0);
    a.step().unwrap();
    h.borrow_mut().mouse(x, y, 1);
    a.step().unwrap();
    h.borrow_mut().mouse(0, 0, 1);
    for _ in 0..10 {
        a.step().unwrap();
    }
    h.borrow_mut().mouse(x, y, 0);
    for _ in 0..10 {
        a.step().unwrap();
    }
    assert_eq!(a.model().expression(), "");
    h.borrow_mut().mouse(x, y, 1);
    for _ in 0..10 {
        a.step().unwrap();
    }
    h.borrow_mut().actor(false, Operation::Focus, 0, 0);
    a.step().unwrap();
    assert!(!a.model().active());
    h.borrow_mut().actor(false, Operation::Focus, 101, 0);
    h.borrow_mut().mouse(x, y, 0);
    for _ in 0..10 {
        a.step().unwrap();
    }
    assert_eq!(a.model().expression(), "");
    // The old button release was correctly suppressed by the input service's
    // focus cancellation. A new genuine modifier offer establishes local
    // focus without editing or activating a calculator control.
    keys(&h, &mut a, &[0x2a, 0xaa]);
    assert!(a.model().active());
    h.borrow_mut().mouse(x, y, 1);
    h.borrow_mut().mouse(x, y, 0);
    for _ in 0..10 {
        a.step().unwrap();
    }
    assert_eq!(a.model().expression(), "7");
}
#[test]
fn lost_or_failed_real_surface_reply_never_certifies_rendered_state() {
    for failed in [false, true] {
        let h = Host::new();
        let mut a = app(&h, 1);
        if failed {
            h.borrow_mut().fail_blit = true;
        } else {
            h.borrow_mut().drop_blit = true;
        }
        assert!(matches!(a.step(), Err(Error::Surface(_))));
        assert!(!a.evidence().ready);
        assert_eq!(a.evidence().frames, 0);
        assert_eq!(a.evidence().tiles, 0);
        a.stop_clean().unwrap();
        assert_eq!(h.borrow().closed.len(), 4);
    }
}
#[test]
fn input_failure_is_visible_but_cannot_be_a_fake_ready_and_stop_attempts_every_owner() {
    let h = Host::new();
    let mut a = app(&h, 1);
    h.borrow_mut().drop_input = true;
    assert!(matches!(a.step(), Err(Error::Input(_))));
    assert_eq!(
        a.model().notice(),
        nieuw_os_calculator::model::Notice::InputLost
    );
    assert!(!a.evidence().ready);
    h.borrow_mut().fail_close = true;
    a.stop_clean().unwrap();
    assert_eq!(h.borrow().closed.len(), 4);
    let h = Host::new();
    let mut a = app(&h, 1);
    h.borrow_mut().fail_close = true;
    assert!(matches!(a.stop_clean(), Err(Error::Cleanup(_))));
    assert_eq!(h.borrow().closed.len(), 4);
}
#[test]
fn bootstrap_rejects_aliases_hidden_document_metadata_and_unbound_principals() {
    let mut words = [0u64; 128];
    for (i, at) in [88, 96, 544, 552, 576, 584].into_iter().enumerate() {
        words[at / 8] = (1 << 32) | (i as u64 + 1);
    }
    for (i, at) in [176, 184, 560, 592].into_iter().enumerate() {
        words[at / 8] = 100 + i as u64;
    }
    for at in [568, 600, 608, 616] {
        words[at / 8] = 1;
    }
    assert!(Bootstrap::read(|at| words[at as usize / 8]).is_some());
    for (at, value) in [
        (552, words[544 / 8]),
        (592, words[184 / 8]),
        (568, 0),
        (608, 0),
        (616, 3),
        (80, 99),
        (168, 22),
    ] {
        let mut wrong = words;
        wrong[at / 8] = value;
        assert!(
            Bootstrap::read(|at| wrong[at as usize / 8]).is_none(),
            "{at}"
        );
    }
}

#[test]
fn startup_stop_closes_only_the_four_untransferred_service_leaves_even_on_failure() {
    let host = Host::new();
    let mut h = host.borrow_mut();
    let registry_root = h.engine.create_endpoint(INIT).unwrap();
    let registry = h
        .engine
        .bootstrap_delegate(INIT, registry_root, APP, SEND)
        .unwrap();
    let control_root = h.engine.create_endpoint(APP).unwrap();
    let control = h.engine.derive(APP, control_root, RECV).unwrap();
    h.engine.close(APP, control_root).unwrap();
    let [input, surface] = h.pairs;
    let config = Bootstrap {
        registry: send(registry),
        control: recv(control),
        surface_send: send(surface.send),
        surface_receive: recv(surface.receive),
        input_send: send(input.send),
        input_receive: recv(input.receive),
        self_pid: APP,
        init_pid: INIT,
        compositor_pid: COMP,
        policy_pid: POLICY,
        surface_epoch: SE,
        input_epoch: IE,
        input_client: 101,
        scale: 1,
    };
    let mut attempted = vec![];
    let result = config.close_unstarted(|handle| {
        attempted.push(handle.raw());
        h.engine.close(APP, handle.raw()).unwrap();
        if attempted.len() == 1 {
            Err(RuntimeError::InvalidReturn)
        } else {
            Ok(())
        }
    });
    assert_eq!(result, Err(RuntimeError::InvalidReturn));
    assert_eq!(
        attempted,
        [input.send, input.receive, surface.send, surface.receive]
    );
    assert!(h.engine.check_endpoint(APP, registry, SEND).is_ok());
    assert!(h.engine.check_endpoint(APP, control, RECV).is_ok());
    assert!(h.engine.check_invariants());
}

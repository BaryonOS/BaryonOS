//! Actual capability/INP1/SUR1 engines in a host process. Recording RAM pixels
//! replace the display counterpart ONLY here, never in calculator-app.
use capability_core::{Engine, Message, RECV, SEND};
use nieuw_os_calculator::{
    app::{App, Error},
    bootstrap::Bootstrap,
    decimal::SCALE,
};
use nieuw_os_input_client::{clipboard as clip, Binding, Session as Input, Transport};
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ClipboardFault {
    #[default]
    None,
    FocusBeforeAck,
    ReceiveClosed,
    AckSendClosed,
    AckReplyClosed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClipboardTrace {
    Requested(u64),
    ReplyQueued,
    Received,
    AckAttempted,
    FocusCanceled,
    AckStatus(u64),
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
    actors: [u64; 3],
    input_epoch: u64,
    surface_epoch: u64,
    now: u64,
    blits: usize,
    framebuffer: Vec<u32>,
    closed: Vec<u64>,
    drop_input: bool,
    drop_blit: bool,
    drop_final_blit: bool,
    fail_blit: bool,
    fail_close: bool,
    clipboard_fault: ClipboardFault,
    clipboard_trace: Vec<ClipboardTrace>,
    clipboard_sequence: Option<u64>,
    clipboard_reply_queued: bool,
    clipboard_ack_reply_queued: bool,
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
            actors: [COMP, SOURCE, POLICY],
            input_epoch: IE,
            surface_epoch: SE,
            now: 0,
            blits: 0,
            framebuffer: vec![0; 800 * 600],
            closed: vec![],
            drop_input: false,
            drop_blit: false,
            drop_final_blit: false,
            fail_blit: false,
            fail_close: false,
            clipboard_fault: ClipboardFault::None,
            clipboard_trace: vec![],
            clipboard_sequence: None,
            clipboard_reply_queued: false,
            clipboard_ack_reply_queued: false,
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
            (0, self.actors[1], self.source_send, self.source_id)
        } else {
            self.control_id += 1;
            (1, self.actors[0], self.control_send, self.control_id)
        };
        let mut message = Message::new();
        message.tag = iw::TAG;
        message.len = iw::SIZE as u32;
        message.data[..iw::SIZE].copy_from_slice(
            &Request {
                operation: op,
                id,
                epoch: self.input_epoch,
                arg0: a,
                arg1: b,
            }
            .encode(),
        );
        self.engine.send(pid, channel, &message).unwrap();
        let actual = self
            .engine
            .receive(self.actors[2], self.roots[index])
            .unwrap();
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
        let response = if index == 0 && m.tag == clip::TAG {
            // A tiny host-only clipboard counterpart, not the native broker.
            // The actual input Service still supplies/ACKs the real shortcut;
            // CLP shares its exact app pair and Engine-authenticated peer.
            assert_eq!(m.sender_pid, APP);
            let request = clip::Request::decode(&m.data[..m.len as usize]).unwrap();
            assert_eq!(request.header.operation, clip::Operation::Paste);
            assert_eq!(request.header.epoch, self.input_epoch);
            self.clipboard_sequence = Some(request.header.sequence);
            self.clipboard_trace
                .push(ClipboardTrace::Requested(request.header.sequence));
            let reply = clip::Reply {
                header: request.header,
                status: clip::Status::Ok,
                text: "6/2",
            }
            .outgoing()
            .unwrap();
            let response = Message::decode(reply.as_bytes()).unwrap();
            Some(response.data[..response.len as usize].to_vec())
        } else if index == 0 {
            assert_eq!(m.tag, iw::TAG);
            let request = Request::decode(&m.data[..m.len as usize], self.input_epoch).unwrap();
            let received = Received::from_bytes(m.encode(), m.len as u64).unwrap();
            let reply = match self.input.handle(recv(p.ingress), &received) {
                Ok(reply) => reply,
                Err(error) => self
                    .input
                    .error_reply(recv(p.ingress), &received, error)
                    .unwrap(),
            };
            assert!(self.input.reply_is_current(&reply));
            if request.operation == Operation::Ack && self.clipboard_sequence == Some(request.arg0)
            {
                let status = u64::from_le_bytes(reply.payload()[24..32].try_into().unwrap());
                self.clipboard_trace.push(ClipboardTrace::AckStatus(status));
                self.clipboard_ack_reply_queued = true;
            }
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
                surface::Action::Retain(reply) => {
                    if self.drop_blit
                        || self.drop_final_blit && self.surface.completed_frame().is_some()
                    {
                        return;
                    }
                    Some(reply.bytes().to_vec())
                }
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
                    if self.drop_blit
                        || self.drop_final_blit && self.surface.completed_frame().is_some()
                    {
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
            if m.tag == clip::TAG {
                self.clipboard_reply_queued = true;
                self.clipboard_trace.push(ClipboardTrace::ReplyQueued);
            }
        }
    }
    /// Host lifecycle plumbing only: real Engine exits invalidate every old
    /// service principal before strictly larger PIDs and fresh leaf handles
    /// are installed. This is not the native kernel's 0x53 implementation.
    fn replace_services(&mut self) {
        let [comp, source, policy] = self.actors;
        for pid in [comp, source, policy] {
            self.engine.exit(pid).unwrap();
        }
        self.surface.retire();
        self.actors = [policy + 2, policy + 3, policy + 4];
        let [comp, source, policy] = self.actors;
        assert!(comp > INIT);
        for pid in [comp, source, policy] {
            self.engine.register(pid).unwrap();
        }
        self.input_epoch += 1;
        self.surface_epoch += 1;
        self.roots = core::array::from_fn(|_| self.engine.create_endpoint(policy).unwrap());
        self.source_send = self
            .engine
            .bootstrap_delegate(policy, self.roots[0], source, SEND)
            .unwrap();
        self.control_send = self
            .engine
            .bootstrap_delegate(policy, self.roots[1], comp, SEND)
            .unwrap();
        self.pairs = [
            pair(&mut self.engine, policy, Some(self.roots[3])),
            pair(&mut self.engine, comp, None),
        ];
        self.input = input::Service::new(
            input::Configuration {
                epoch: self.input_epoch,
                source: input::Actor {
                    endpoint: recv(self.roots[0]),
                    pid: source,
                },
                compositor: input::Actor {
                    endpoint: recv(self.roots[1]),
                    pid: comp,
                },
                supervisor: input::Actor {
                    endpoint: recv(self.roots[2]),
                    pid: INIT,
                },
                applications: recv(self.roots[3]),
                width: 800,
                height: 600,
            },
            &[input::ClientBinding { id: 101, pid: APP }],
        )
        .unwrap();
        self.surface = surface::Receiver::new(
            APP,
            Geometry {
                epoch: self.surface_epoch,
                ..geometry()
            },
            100,
        )
        .unwrap();
        self.source_id = 0;
        self.control_id = 0;
        self.actor(true, Operation::Sync, 1, 0);
        self.actor(false, Operation::Focus, 101, 0);
        assert!(self.engine.check_invariants());
    }
}
struct Port(Rc<RefCell<Host>>, usize);
impl Transport for Port {
    fn send(&mut self, to: SendEndpoint, message: &Outgoing) -> Result<(), RuntimeError> {
        let mut h = self.0.borrow_mut();
        let message = Message::decode(message.as_bytes()).unwrap();
        if self.1 == 0 && message.tag == iw::TAG {
            let request =
                Request::decode(&message.data[..message.len as usize], h.input_epoch).unwrap();
            if request.operation == Operation::Ack && h.clipboard_sequence == Some(request.arg0) {
                h.clipboard_trace.push(ClipboardTrace::AckAttempted);
                match h.clipboard_fault {
                    ClipboardFault::FocusBeforeAck => {
                        // Runs after clipboard_paste returned its valid Text,
                        // before publishing ACK: real compositor-authorized
                        // Service focus cancellation supersedes that offer.
                        h.actor(false, Operation::Focus, 0, 0);
                        h.clipboard_trace.push(ClipboardTrace::FocusCanceled);
                    }
                    ClipboardFault::AckSendClosed => {
                        h.engine.close(APP, to.handle().raw()).unwrap();
                    }
                    _ => {}
                }
            }
        }
        h.engine
            .send(APP, to.handle().raw(), &message)
            .map_err(|e| RuntimeError::Kernel(e.code()))?;
        h.pump(self.1);
        Ok(())
    }
    fn receive(&mut self, from: ReceiveEndpoint) -> Result<Received, RuntimeError> {
        let mut h = self.0.borrow_mut();
        if self.1 == 0
            && ((h.clipboard_fault == ClipboardFault::ReceiveClosed && h.clipboard_reply_queued)
                || (h.clipboard_fault == ClipboardFault::AckReplyClosed
                    && h.clipboard_ack_reply_queued))
        {
            // Actual capability closure supplies BadHandle; no fabricated
            // successful reply or protocol ACK reaches the app.
            h.engine.close(APP, from.handle().raw()).unwrap();
        }
        let m = h
            .engine
            .receive(APP, from.handle().raw())
            .map_err(|e| RuntimeError::Kernel(e.code()))?;
        let received = Received::from_bytes(m.encode(), m.len as u64)?;
        if m.tag == clip::TAG {
            assert_eq!(m.sender_pid, h.actors[2]);
            h.clipboard_reply_queued = false;
            h.clipboard_trace.push(ClipboardTrace::Received);
        }
        Ok(received)
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
fn clipboard_ready(h: &Rc<RefCell<Host>>, enabled: bool) -> App<Port, Port> {
    let mut a = app(h, 1);
    if enabled {
        a.enable_clipboard();
    }
    finish(&mut a);
    keys(h, &mut a, &[0x03, 0x83]); // existing synthetic expression "2"
    keys(h, &mut a, &[0x1d, 0x1e, 0x9e]); // held Ctrl, then Ctrl+A selects it
    finish(&mut a);
    assert_eq!(a.model().expression(), "2");
    assert_eq!(a.model().selection(), 0..1);
    assert_eq!(a.model().evaluations(), 0);
    assert_eq!(a.model().failures(), 0);
    assert!(a.model().answer().is_none());
    assert!(a.evidence().ready);
    a
}

fn assert_retained_calculator_pixels(a: &App<Port, Port>, h: &Host, scale: u32) {
    use nieuw_os_calculator::render::{render_tile, Rect, Viewport};
    let g = geometry();
    let viewport = Viewport::new(
        u32::from(g.assigned.width),
        u32::from(g.assigned.height),
        scale,
    )
    .unwrap();
    let mut pixels = [0; 256];
    for tile in 0..g.tiles() {
        let r = g.rectangle(tile).unwrap();
        render_tile(
            a.model(),
            viewport,
            Rect {
                x: r.x,
                y: r.y,
                width: u32::from(r.width),
                height: u32::from(r.height),
            },
            &mut pixels,
        )
        .unwrap();
        for row in 0..u32::from(r.height) {
            for col in 0..u32::from(r.width) {
                let at = ((g.assigned.y + r.y + row) * 800 + g.assigned.x + r.x + col) as usize;
                assert_eq!(
                    h.framebuffer[at],
                    pixels[(row * u32::from(r.width) + col) as usize],
                    "retained calculator differs at tile {tile}, ({col},{row})"
                );
            }
        }
    }
    for (at, pixel) in h.framebuffer.iter().enumerate() {
        let (x, y) = (at as u32 % 800, at as u32 / 800);
        let r = g.assigned;
        if x < r.x || y < r.y || x >= r.x + u32::from(r.width) || y >= r.y + u32::from(r.height) {
            assert_eq!(*pixel, 0, "calculator paint escaped its assigned surface");
        }
    }
}

#[test]
fn retained_calculator_exact_pixels_work_reduction_and_partial_supersede() {
    for scale in [1, 2] {
        let h = Host::new();
        h.borrow_mut().surface = surface::Receiver::new_retained(APP, geometry(), 100).unwrap();
        let mut a = app(&h, scale);
        a.enable_retained().unwrap();
        finish(&mut a);
        assert_eq!(a.surface_counts().committed_tiles, geometry().tiles());
        assert_eq!(a.surface_counts().retained_tiles, 0);
        let before = a.surface_counts();
        keys(&h, &mut a, &[0x03, 0x83]);
        finish(&mut a);
        assert_eq!(a.model().expression(), "2");
        assert_eq!(a.model().evaluations(), 0);
        let after = a.surface_counts();
        assert!(after.retained_tiles > before.retained_tiles);
        let old_full_sends: u64 = (0..geometry().tiles())
            .map(|tile| {
                geometry()
                    .rectangle(tile)
                    .unwrap()
                    .bytes()
                    .div_ceil(surface::CHUNK) as u64
                    + 2
            })
            .sum();
        assert!(
            (after.sent_messages - before.sent_messages) * 10 < old_full_sends,
            "calculator edit must remove at least 90% of old full-grid SUR sends"
        );
        assert_retained_calculator_pixels(&a, &h.borrow(), scale);

        let frames = a.evidence().frames;
        let committed = a.surface_counts().committed_tiles;
        h.borrow_mut().key(0x04); // Append "3", stop after the first changed tile.
        for _ in 0..geometry().tiles() {
            a.step().unwrap();
            if a.surface_counts().committed_tiles > committed {
                break;
            }
        }
        assert!(a.surface_counts().committed_tiles > committed);
        assert_eq!(a.evidence().frames, frames);
        keys(&h, &mut a, &[0x84, 0x05, 0x85]); // Supersede with "4".
        finish(&mut a);
        assert_eq!(a.model().expression(), "234");
        assert_eq!(
            h.borrow().surface.completed_frame(),
            Some(a.model().generation())
        );
        assert_retained_calculator_pixels(&a, &h.borrow(), scale);
        assert!(h.borrow().engine.check_invariants());
    }
}

#[test]
fn lost_retained_reply_never_presents_or_drops_calculator_work() {
    let h = Host::new();
    h.borrow_mut().surface = surface::Receiver::new_retained(APP, geometry(), 100).unwrap();
    let mut a = app(&h, 1);
    a.enable_retained().unwrap();
    finish(&mut a);
    let frames = a.evidence().frames;
    let blits = h.borrow().blits;
    h.borrow_mut().drop_blit = true;
    h.borrow_mut().key(0x03);
    assert!(matches!(a.step(), Err(Error::Surface(_))));
    assert_eq!(
        h.borrow().blits,
        blits,
        "the lost reply must be Retain, not a blit"
    );
    assert_eq!(a.model().expression(), "2");
    assert_eq!(a.model().evaluations(), 0);
    assert_eq!(a.evidence().frames, frames);
    assert!(!a.evidence().ready);
    assert!(!a.can_idle());
    assert!(h.borrow().engine.check_invariants());
}

#[test]
fn enabled_clipboard_pastes_only_after_real_input_ack_and_never_evaluates_text() {
    for enabled in [false, true] {
        let h = Host::new();
        let mut a = clipboard_ready(&h, enabled);
        let acks = a.evidence().input_acks;
        h.borrow_mut().key(0x2f); // real Ctrl+V Press
        a.step().unwrap();
        assert_eq!(a.evidence().input_acks, acks + 1);
        if enabled {
            assert_eq!(a.model().expression(), "6/2");
            let sequence = h.borrow().clipboard_sequence.unwrap();
            assert_eq!(
                h.borrow().clipboard_trace,
                [
                    ClipboardTrace::Requested(sequence),
                    ClipboardTrace::ReplyQueued,
                    ClipboardTrace::Received,
                    ClipboardTrace::AckAttempted,
                    ClipboardTrace::AckStatus(0),
                ]
            );
            assert!(!a.evidence().ready);
            finish(&mut a);
            assert!(a.evidence().ready);
            assert_eq!(
                h.borrow().surface.completed_frame(),
                Some(a.model().generation())
            );
        } else {
            assert_eq!(a.model().expression(), "2");
            assert!(h.borrow().clipboard_trace.is_empty());
        }
        assert_eq!(a.model().evaluations(), 0);
        assert_eq!(a.model().failures(), 0);
        assert!(a.model().answer().is_none());
        a.stop_clean().unwrap();
        assert!(h.borrow().engine.check_invariants());
    }
}

#[test]
fn received_paste_is_discarded_when_real_focus_cancellation_supersedes_its_ack() {
    let h = Host::new();
    let mut a = clipboard_ready(&h, true);
    let superseded = a.evidence().superseded;
    h.borrow_mut().clipboard_fault = ClipboardFault::FocusBeforeAck;
    h.borrow_mut().key(0x2f);
    a.step().unwrap();
    assert_eq!(a.model().expression(), "2");
    assert_eq!(a.model().selection(), 0..1);
    assert_eq!(a.model().evaluations(), 0);
    assert_eq!(a.model().failures(), 0);
    assert!(a.model().answer().is_none());
    assert!(!a.model().active());
    assert_eq!(a.evidence().superseded, superseded + 1);
    let sequence = h.borrow().clipboard_sequence.unwrap();
    assert_eq!(
        h.borrow().clipboard_trace,
        [
            ClipboardTrace::Requested(sequence),
            ClipboardTrace::ReplyQueued,
            ClipboardTrace::Received,
            ClipboardTrace::AckAttempted,
            ClipboardTrace::FocusCanceled,
            ClipboardTrace::AckStatus(input::Error::StaleAck as u64),
        ]
    );
    assert!(h.borrow().engine.check_invariants());
    a.stop_clean().unwrap();
}

#[test]
fn clipboard_or_later_ack_transport_failure_keeps_selection_and_cannot_fake_ready_or_ack() {
    for fault in [
        ClipboardFault::ReceiveClosed,
        ClipboardFault::AckSendClosed,
        ClipboardFault::AckReplyClosed,
    ] {
        let h = Host::new();
        let mut a = clipboard_ready(&h, true);
        let acks = a.evidence().input_acks;
        let errors = a.evidence().errors;
        h.borrow_mut().clipboard_fault = fault;
        h.borrow_mut().key(0x2f);
        assert!(matches!(a.step(), Err(Error::Input(_))));
        assert_eq!(a.model().expression(), "2");
        assert_eq!(a.model().selection(), 0..1);
        assert_eq!(a.model().evaluations(), 0);
        assert_eq!(a.model().failures(), 0);
        assert!(a.model().answer().is_none());
        assert_eq!(a.evidence().input_acks, acks);
        assert_eq!(a.evidence().errors, errors + 1);
        assert_eq!(
            a.model().notice(),
            nieuw_os_calculator::model::Notice::InputLost
        );
        let trace = h.borrow().clipboard_trace.clone();
        assert!(trace.contains(&ClipboardTrace::ReplyQueued));
        if fault == ClipboardFault::ReceiveClosed {
            assert!(!trace.contains(&ClipboardTrace::Received));
            assert!(!trace.contains(&ClipboardTrace::AckAttempted));
        } else {
            assert!(trace.contains(&ClipboardTrace::Received));
            assert!(trace.contains(&ClipboardTrace::AckAttempted));
        }
        assert_eq!(
            trace.contains(&ClipboardTrace::AckStatus(0)),
            fault == ClipboardFault::AckReplyClosed
        );
        // Even after every tile of the error state has genuinely committed,
        // failed input cannot become Ready or silently retry the old gesture.
        for _ in 0..geometry().tiles().div_ceil(8) + 1 {
            a.step().unwrap();
            assert!(!a.evidence().ready);
            assert!(!a.can_idle());
        }
        assert_eq!(
            h.borrow().surface.completed_frame(),
            Some(a.model().generation())
        );
        assert_eq!(h.borrow().clipboard_trace, trace);
        assert_eq!(a.evidence().input_acks, acks);
        assert_eq!(a.model().expression(), "2");
        assert!(h.borrow().engine.check_invariants());
        a.stop_clean().unwrap();
    }
}
fn replacement_sessions(
    h: &Rc<RefCell<Host>>,
    input_epoch: u64,
) -> (Input<Port>, Surface<Port>, Geometry) {
    let host = h.borrow();
    let [p, s] = host.pairs;
    let [comp, _, policy] = host.actors;
    let surface_epoch = host.surface_epoch;
    drop(host);
    let input = Input::new(
        Port(h.clone(), 0),
        Binding {
            epoch: input_epoch,
            service_pid: policy,
            client_id: 101,
            width: 800,
            height: 600,
            send: send(p.send),
            receive: recv(p.receive),
        },
        8,
    )
    .unwrap();
    let mut surface = Surface::new(
        Port(h.clone(), 1),
        comp,
        surface_epoch,
        send(s.send),
        recv(s.receive),
        8,
    )
    .unwrap();
    let geometry = surface.geometry().unwrap();
    (input, surface, geometry)
}
fn computed_work(h: &Rc<RefCell<Host>>, a: &mut App<Port, Port>) {
    finish(a);
    // A real failed evaluation, reset, successful expression, then move its
    // caret left: preservation is tested with nonzero counters and a caret
    // different from the expression end, using authenticated scan events.
    keys(h, a, &[0x1c, 0x9c, 0x2e, 0xae]);
    keys(
        h,
        a,
        &[
            0x03, 0x83, 0x2a, 0x0d, 0x8d, 0xaa, 0x04, 0x84, 0x1c, 0x9c, 0xe0, 0x4b, 0xe0, 0xcb,
        ],
    );
    finish(a);
    assert_computed_work(a);
}
fn assert_computed_work(a: &App<Port, Port>) {
    assert_eq!(a.model().expression(), "2+3");
    assert_eq!(a.model().caret(), 2);
    assert_eq!(a.model().selection(), 2..2);
    assert_eq!(a.model().answer().unwrap().value.raw(), 5 * SCALE);
    assert!(a.model().current());
    assert_eq!(a.model().evaluations(), 1);
    assert_eq!(a.model().failures(), 1);
}
fn assert_closed_pairs(h: &Host, pairs: [Pair; 2]) {
    for pair in pairs {
        assert_eq!(
            h.engine.check_endpoint(APP, pair.send, SEND),
            Err(capability_core::Error::BadHandle)
        );
        assert_eq!(
            h.engine.check_endpoint(APP, pair.receive, RECV),
            Err(capability_core::Error::BadHandle)
        );
    }
}

#[test]
fn real_service_replacement_preserves_work_and_requires_new_complete_frame_acknowledgements() {
    for drop_ack in [false, true] {
        let h = Host::new();
        let mut a = app(&h, 1);
        computed_work(&h, &mut a);
        let evidence = a.evidence();
        let old_pairs = h.borrow().pairs;
        a.detach_presentation().unwrap();
        assert!(!a.can_idle());
        assert!(!a.evidence().ready);
        assert_computed_work(&a);
        {
            let mut host = h.borrow_mut();
            host.replace_services();
            assert_closed_pairs(&host, old_pairs);
            for pid in [COMP, SOURCE, POLICY] {
                assert_eq!(
                    host.engine.send(pid, old_pairs[0].send, &Message::new()),
                    Err(capability_core::Error::BadHandle)
                );
            }
            for pair in host.pairs {
                assert!(host.engine.check_endpoint(APP, pair.send, SEND).is_ok());
                assert!(host.engine.check_endpoint(APP, pair.receive, RECV).is_ok());
            }
            // Old epochs still fail even when carried over a genuinely fresh
            // kernel-authenticated app channel to the replacement service.
            let p = host.pairs[0];
            let mut stale = Message::new();
            stale.tag = iw::TAG;
            stale.len = iw::SIZE as u32;
            stale.data[..iw::SIZE].copy_from_slice(
                &Request {
                    operation: Operation::Peek,
                    id: 900,
                    epoch: IE,
                    arg0: 0,
                    arg1: 0,
                }
                .encode(),
            );
            host.engine.send(APP, p.send, &stale).unwrap();
            let actual = host.engine.receive(p.server, p.ingress).unwrap();
            let received = Received::from_bytes(actual.encode(), actual.len as u64).unwrap();
            assert_eq!(received.sender(), APP);
            assert!(matches!(
                host.input.handle(recv(p.ingress), &received),
                Err(input::Error::WrongEpoch)
            ));
            let p = host.pairs[1];
            let payload = surface::Header {
                operation: surface::Operation::Info,
                request: 900,
                epoch: SE,
                frame: 0,
                tile: 0,
                arg0: 0,
                arg1: 0,
            }
            .encode();
            stale.tag = surface::TAG;
            stale.len = payload.len() as u32;
            stale.data[..payload.len()].copy_from_slice(&payload);
            host.engine.send(APP, p.send, &stale).unwrap();
            let actual = host.engine.receive(p.server, p.ingress).unwrap();
            let now = host.now;
            assert!(matches!(
                host.surface
                    .accept(actual.sender_pid, now, &actual.data[..actual.len as usize]),
                Err(surface::Error::Epoch)
            ));
            assert_eq!(host.surface.completed_frame(), None);
        }
        let epoch = h.borrow().input_epoch;
        let (input, surface, geometry) = replacement_sessions(&h, epoch);
        a.rebind_presentation(input, surface, geometry).unwrap();
        assert!(!a.can_idle());
        assert_computed_work(&a);
        assert!(!a.evidence().ready);
        assert_eq!(a.evidence().presented_generation, 0);
        assert_eq!(a.evidence().frames, evidence.frames);
        h.borrow_mut().drop_final_blit = drop_ack;
        if drop_ack {
            let blits = h.borrow().blits;
            let mut failed = false;
            for _ in 0..geometry.tiles().div_ceil(8) + 1 {
                match a.step() {
                    Ok(()) => assert!(!a.evidence().ready),
                    Err(Error::Surface(_)) => {
                        failed = true;
                        break;
                    }
                    other => panic!("unexpected recovery result: {other:?}"),
                }
            }
            assert!(failed);
            assert_eq!(h.borrow().blits, blits + geometry.tiles() as usize);
            assert!(h.borrow().surface.completed_frame().is_some());
            assert!(!a.evidence().ready);
            assert_eq!(a.evidence().frames, evidence.frames);
            assert_eq!(a.evidence().tiles, evidence.tiles + geometry.tiles() - 1);
            assert_eq!(a.evidence().presented_generation, 0);
        } else {
            a.step().unwrap();
            assert!(!a.evidence().ready);
            assert_eq!(a.evidence().frames, evidence.frames);
            finish(&mut a);
            assert!(a.evidence().ready);
            assert_eq!(a.evidence().frames, evidence.frames + 1);
            assert_eq!(a.evidence().presented_generation, a.model().generation());
            assert_eq!(
                h.borrow().surface.completed_frame(),
                Some(a.model().generation())
            );
        }
        assert_computed_work(&a);
        a.stop_clean().unwrap();
        assert!(h.borrow().engine.check_invariants());
    }
}

#[test]
fn rejected_rebind_epochs_and_small_scaled_geometry_close_fresh_channels_and_allow_retry() {
    for invalid in 0..3 {
        let h = Host::new();
        let mut a = app(&h, 2);
        computed_work(&h, &mut a);
        a.detach_presentation().unwrap();
        h.borrow_mut().replace_services();
        if invalid == 2 {
            let epoch = h.borrow().surface_epoch;
            h.borrow_mut().surface = surface::Receiver::new(
                APP,
                Geometry {
                    assigned: Rectangle {
                        width: 320,
                        height: 240,
                        ..geometry().assigned
                    },
                    epoch,
                    ..geometry()
                },
                100,
            )
            .unwrap();
        }
        let epoch = if invalid == 0 {
            IE
        } else {
            h.borrow().input_epoch
        };
        let (input, surface, mut geometry) = replacement_sessions(&h, epoch);
        if invalid == 1 {
            geometry.epoch = SE;
        }
        let fresh = h.borrow().pairs;
        assert_eq!(
            a.rebind_presentation(input, surface, geometry),
            Err(Error::Binding)
        );
        assert_closed_pairs(&h.borrow(), fresh);
        assert_computed_work(&a);
        assert!(!a.evidence().ready);
        assert_eq!(h.borrow().surface.completed_frame(), None);

        // Rejected sessions transfer no ownership into App. A later wholly
        // fresh service group can still bind and render the preserved work.
        h.borrow_mut().replace_services();
        let epoch = h.borrow().input_epoch;
        let (input, surface, geometry) = replacement_sessions(&h, epoch);
        a.rebind_presentation(input, surface, geometry).unwrap();
        assert!(!a.evidence().ready);
        finish(&mut a);
        assert!(a.evidence().ready);
        assert_computed_work(&a);
        a.stop_clean().unwrap();
        assert!(h.borrow().engine.check_invariants());
    }
}

#[test]
fn supervisor_retirement_detaches_both_owners_when_one_session_already_failed() {
    for input_already_closed in [false, true] {
        let h = Host::new();
        let mut a = app(&h, 1);
        computed_work(&h, &mut a);
        if input_already_closed {
            h.borrow_mut().drop_input = true;
            assert!(matches!(a.step(), Err(Error::Input(_))));
        }
        let pairs = h.borrow().pairs;
        {
            let mut host = h.borrow_mut();
            for pair in pairs {
                for handle in [pair.send, pair.receive] {
                    match host.engine.close(APP, handle) {
                        Ok(()) | Err(capability_core::Error::BadHandle) => {}
                        other => panic!("unexpected manifest retirement: {other:?}"),
                    }
                }
            }
        }
        a.detach_presentation().unwrap();
        assert!(!a.evidence().ready);
        assert_computed_work(&a);
        assert_closed_pairs(&h.borrow(), pairs);
        assert!(h.borrow().engine.check_invariants());
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
        assert!(!a.can_idle());
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
    assert!(!a.can_idle());
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

#[test]
fn idle_requires_current_frame_and_empty_batch_and_rechecks_late_input() {
    let h = Host::new();
    let mut a = app(&h, 1);
    assert!(!a.can_idle());
    a.step().unwrap();
    assert!(!a.evidence().ready);
    assert!(!a.can_idle());
    finish(&mut a);
    assert!(a.can_idle());

    let acks = a.evidence().input_acks;
    for byte in [0x2a, 0xaa, 0x36, 0xb6] {
        h.borrow_mut().key(byte);
    }
    a.step().unwrap();
    assert_eq!(a.evidence().input_acks - acks, 4);
    assert!(!a.can_idle());
    finish(&mut a);
    assert!(a.can_idle());

    let endpoint = {
        let mut host = h.borrow_mut();
        let root = host.engine.create_endpoint(APP).unwrap();
        let control = host.engine.derive(APP, root, RECV).unwrap();
        host.engine.close(APP, root).unwrap();
        recv(control)
    };
    let poll = nieuw_os_calculator::idle::Poll::new(40, endpoint).unwrap();
    let keys = a.evidence().keys;
    h.borrow_mut().key(0x03);
    assert_eq!(poll.deadline(), 41);
    a.step().unwrap();
    assert_eq!(a.evidence().keys, keys + 1);
    assert_eq!(a.model().expression(), "2");
    assert!(!a.can_idle());
    finish(&mut a);
    assert!(a.can_idle());
    a.stop_clean().unwrap();
    assert!(!a.can_idle());
    h.borrow_mut()
        .engine
        .close(APP, endpoint.handle().raw())
        .unwrap();
}

//! Host protocol/authority tests through real capability and input-service
//! engines. They are not native keyboard, mouse or compositor execution.
use capability_core::{Cap, Engine, Message, SEND, TRANSFER};
use nieuw_os_input_client::{
    AckOutcome, Binding, Delivery, Error, Event, Peek, Session, Transport,
};
use nieuw_os_input_core::{
    wire::{self, Operation, Request},
    Actor, ClientBinding, Configuration, Service,
};
use nieuw_os_sdk_runtime::{
    Error as RuntimeError, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};
use std::{cell::RefCell, rc::Rc};
use ui_core::{KeyAction, PhysicalKey};

const SERVER: u64 = 10;
const CLIENT: u64 = 4;
const EPOCH: u64 = 7;
const ID: u64 = 101;
fn h(raw: u64) -> Handle {
    Handle::from_raw(raw).unwrap()
}
fn recv(raw: u64) -> ReceiveEndpoint {
    ReceiveEndpoint::from_handle(h(raw))
}
#[derive(Clone, Copy, Default)]
enum Fault {
    #[default]
    None,
    DropReply,
    WrongSender,
    WrongTag,
    Short,
    Caps,
    Mutate(usize, u8),
    UnknownSend,
    Receive,
    Yield,
}
struct Model {
    engine: Box<Engine>,
    service: Service,
    roots: [u64; 4],
    source_send: u64,
    control_send: u64,
    reply_root: u64,
    reply_send: u64,
    hostile_send: u64,
    app_send: u64,
    transferable: u64,
    source_id: u64,
    control_id: u64,
    fault: Fault,
    attempts: usize,
    publications: usize,
    requests: usize,
    receive_calls: usize,
    yields: usize,
    closed: Vec<u64>,
    queue_full: bool,
    defer_reply: bool,
    pending: Option<Message>,
    received_cap: Option<u64>,
    request_ids: Vec<u64>,
    close_failure: Option<u64>,
}
impl Model {
    fn new() -> Rc<RefCell<Self>> {
        let mut engine = Box::new(Engine::new());
        for pid in [1, 2, 3, CLIENT, 9, SERVER] {
            engine.register(pid).unwrap();
        }
        let roots = core::array::from_fn(|_| engine.create_endpoint(SERVER).unwrap());
        let source_send = engine
            .bootstrap_delegate(SERVER, roots[0], 1, SEND)
            .unwrap();
        let control_send = engine
            .bootstrap_delegate(SERVER, roots[1], 2, SEND)
            .unwrap();
        let app_send = engine
            .bootstrap_delegate(SERVER, roots[3], CLIENT, SEND)
            .unwrap();
        let reply_root = engine.create_endpoint(CLIENT).unwrap();
        let reply_send = engine
            .bootstrap_delegate(CLIENT, reply_root, SERVER, SEND)
            .unwrap();
        let hostile_send = engine
            .bootstrap_delegate(CLIENT, reply_root, 9, SEND)
            .unwrap();
        let root = engine.create_endpoint(1).unwrap();
        let transferable = engine
            .bootstrap_delegate(1, root, SERVER, SEND | TRANSFER)
            .unwrap();
        let service = Service::new(
            Configuration {
                epoch: EPOCH,
                source: Actor {
                    endpoint: recv(roots[0]),
                    pid: 1,
                },
                compositor: Actor {
                    endpoint: recv(roots[1]),
                    pid: 2,
                },
                supervisor: Actor {
                    endpoint: recv(roots[2]),
                    pid: 3,
                },
                applications: recv(roots[3]),
                width: 1280,
                height: 720,
            },
            &[ClientBinding {
                id: ID,
                pid: CLIENT,
            }],
        )
        .unwrap();
        Rc::new(RefCell::new(Self {
            engine,
            service,
            roots,
            source_send,
            control_send,
            reply_root,
            reply_send,
            hostile_send,
            app_send,
            transferable,
            source_id: 0,
            control_id: 0,
            fault: Fault::None,
            attempts: 0,
            publications: 0,
            requests: 0,
            receive_calls: 0,
            yields: 0,
            closed: Vec::new(),
            queue_full: false,
            defer_reply: false,
            pending: None,
            received_cap: None,
            request_ids: Vec::new(),
            close_failure: None,
        }))
    }
    fn binding(&self) -> Binding {
        Binding {
            epoch: EPOCH,
            service_pid: SERVER,
            client_id: ID,
            width: 1280,
            height: 720,
            send: SendEndpoint::from_handle(h(self.app_send)),
            receive: recv(self.reply_root),
        }
    }
    fn control(&mut self, owner: u64) {
        self.control_id += 1;
        self.actor(
            1,
            2,
            self.control_send,
            Request {
                operation: Operation::Focus,
                id: self.control_id,
                epoch: EPOCH,
                arg0: owner,
                arg1: 0,
            },
        );
    }
    fn sync(&mut self) {
        self.source_id += 1;
        self.actor(
            0,
            1,
            self.source_send,
            Request {
                operation: Operation::Sync,
                id: self.source_id,
                epoch: EPOCH,
                arg0: 1,
                arg1: 0,
            },
        );
    }
    fn sample(&mut self, status: u8, byte: u8) {
        self.source_id += 1;
        self.actor(
            0,
            1,
            self.source_send,
            Request {
                operation: Operation::Sample,
                id: self.source_id,
                epoch: EPOCH,
                arg0: status as u64,
                arg1: byte as u64,
            },
        );
    }
    fn actor(&mut self, role: usize, pid: u64, send: u64, request: Request) {
        let mut message = Message::new();
        message.tag = wire::TAG;
        message.len = wire::SIZE as u32;
        message.data[..wire::SIZE].copy_from_slice(&request.encode());
        self.engine.send(pid, send, &message).unwrap();
        let actual = self.engine.receive(SERVER, self.roots[role]).unwrap();
        let received = Received::from_bytes(actual.encode(), actual.len as u64).unwrap();
        let reply = self
            .service
            .handle(recv(self.roots[role]), &received)
            .unwrap();
        assert!(self.service.reply_is_current(&reply));
    }
    fn key(&mut self) {
        self.sync();
        self.control(ID);
        self.sample(1, 0x1e);
    }
    fn deliver(&mut self, request: Message) {
        self.requests += 1;
        let incoming = Received::from_bytes(request.encode(), request.len as u64).unwrap();
        let decoded = Request::decode(incoming.payload(), EPOCH).unwrap();
        self.request_ids.push(decoded.id);
        let prepared = match self.service.handle(recv(self.roots[3]), &incoming) {
            Ok(reply) => reply,
            Err(error) => self
                .service
                .error_reply(recv(self.roots[3]), &incoming, error)
                .unwrap(),
        };
        assert!(self.service.reply_is_current(&prepared));
        if matches!(self.fault, Fault::DropReply) {
            return;
        }
        let mut response = Message::new();
        response.tag = wire::TAG;
        response.len = wire::SIZE as u32;
        response.data[..wire::SIZE].copy_from_slice(prepared.payload());
        match self.fault {
            Fault::WrongTag => response.tag ^= 1,
            Fault::Short => response.len -= 1,
            Fault::Mutate(at, value) => response.data[at] = value,
            Fault::Caps => {
                response.cap_count = 1;
                response.caps[0] = Cap {
                    handle: self.transferable,
                    rights: SEND,
                };
            }
            _ => {}
        }
        if self.defer_reply {
            self.pending = Some(response);
        } else {
            self.reply(response);
        }
    }
    fn reply(&mut self, response: Message) {
        if matches!(self.fault, Fault::WrongSender) {
            self.engine.send(9, self.hostile_send, &response).unwrap();
        } else {
            self.engine
                .send(SERVER, self.reply_send, &response)
                .unwrap();
        }
    }
}
struct Port(Rc<RefCell<Model>>);
impl Transport for Port {
    fn send(&mut self, endpoint: SendEndpoint, message: &Outgoing) -> Result<(), RuntimeError> {
        let mut m = self.0.borrow_mut();
        m.attempts += 1;
        let request = Message::decode(message.as_bytes()).unwrap();
        m.engine
            .send(CLIENT, endpoint.handle().raw(), &request)
            .map_err(|e| RuntimeError::Kernel(e.code()))?;
        m.publications += 1;
        let root = m.roots[3];
        let actual = m.engine.receive(SERVER, root).unwrap();
        m.deliver(actual);
        if matches!(m.fault, Fault::UnknownSend) {
            Err(RuntimeError::InvalidReturn)
        } else {
            Ok(())
        }
    }
    fn receive(&mut self, endpoint: ReceiveEndpoint) -> Result<Received, RuntimeError> {
        let mut m = self.0.borrow_mut();
        m.receive_calls += 1;
        if matches!(m.fault, Fault::Receive) {
            return Err(RuntimeError::InvalidReturn);
        }
        let actual = m
            .engine
            .receive(CLIENT, endpoint.handle().raw())
            .map_err(|e| RuntimeError::Kernel(e.code()))?;
        let result = Received::from_bytes(actual.encode(), actual.len as u64)?;
        m.received_cap = result.cap(0).map(|cap| cap.handle.raw());
        Ok(result)
    }
    fn yield_now(&mut self) -> Result<(), RuntimeError> {
        let mut m = self.0.borrow_mut();
        m.yields += 1;
        if matches!(m.fault, Fault::Yield) {
            return Err(RuntimeError::InvalidReturn);
        }
        if m.queue_full {
            let root = m.roots[3];
            while m.engine.receive(SERVER, root).is_ok() {}
            m.queue_full = false;
        }
        if let Some(reply) = m.pending.take() {
            m.reply(reply);
        }
        Ok(())
    }
    fn close(&mut self, handle: Handle) -> Result<(), RuntimeError> {
        let mut m = self.0.borrow_mut();
        m.closed.push(handle.raw());
        m.engine
            .close(CLIENT, handle.raw())
            .map_err(|e| RuntimeError::Kernel(e.code()))?;
        if m.close_failure == Some(handle.raw()) {
            Err(RuntimeError::InvalidReturn)
        } else {
            Ok(())
        }
    }
}
fn open_session(model: &Rc<RefCell<Model>>) -> Session<Port> {
    Session::new(Port(model.clone()), model.borrow().binding(), 4).unwrap()
}
fn offer(session: &mut Session<Port>) -> nieuw_os_input_client::Offer {
    let Peek::Offer(offer) = session.peek().unwrap() else {
        panic!("expected actual event");
    };
    offer
}

#[test]
fn actual_key_offer_once_then_exact_ack_and_empty() {
    let model = Model::new();
    model.borrow_mut().key();
    let mut session = open_session(&model);
    let first = offer(&mut session);
    assert!(
        matches!(first.event(), Event::Key(key) if key.key == PhysicalKey::A && key.action == KeyAction::Press)
    );
    assert_eq!(first.epoch(), EPOCH);
    assert!(first.sequence() > 0 && first.generation() > 0);
    assert_eq!(session.peek(), Err(Error::OutstandingOffer));
    assert_eq!(model.borrow().requests, 1);
    assert_eq!(session.ack(&first), Ok(AckOutcome::Acknowledged));
    assert_eq!(session.ack(&first), Err(Error::WrongOffer));
    assert_eq!(session.peek(), Ok(Peek::Empty));
    assert_eq!(model.borrow().request_ids, [1, 2, 3]);
    session.close().unwrap();
    session.close().unwrap();
    assert_eq!(model.borrow().closed.len(), 2);
}

#[test]
fn focus_cancel_supersedes_old_ack_without_consuming_the_new_cancel() {
    let model = Model::new();
    model.borrow_mut().key();
    let mut session = open_session(&model);
    let first = offer(&mut session);
    model.borrow_mut().control(0);
    assert_eq!(session.ack(&first), Ok(AckOutcome::Superseded));
    assert!(!session.is_closed());
    let cancel = offer(&mut session);
    assert!(matches!(
        cancel.event(),
        Event::Cancel(nieuw_os_input_client::CancelReason::FocusLost)
    ));
    assert!(cancel.sequence() > first.sequence());
    assert_eq!(session.ack(&cancel), Ok(AckOutcome::Acknowledged));
    assert_eq!(session.peek(), Ok(Peek::Empty));
}

#[test]
fn actual_mouse_global_coordinates_checked_buttons_and_cancel_watermark() {
    let model = Model::new();
    {
        let mut m = model.borrow_mut();
        m.sync();
        m.control(ID);
        for byte in [9, 7, 3] {
            m.sample(0x21, byte);
        }
    }
    let mut session = open_session(&model);
    let mouse = offer(&mut session);
    assert!(
        matches!(mouse.event(), Event::Mouse { input, x: 7, y: 0 } if input.buttons.left() && input.changed.left() && input.dx == 7 && input.dy == 3)
    );
    session.ack(&mouse).unwrap();
    model.borrow_mut().sync();
    // A through-watermark is not fresh focus authority. An older positive
    // watermark remains canonical and is interpreted by the UI controller.
    model.borrow_mut().fault = Fault::Mutate(56, 1);
    let cancel = offer(&mut session);
    assert!(matches!(cancel.event(), Event::Cancel(_)));
    assert_eq!(cancel.generation(), 1);
}

#[test]
fn lost_peek_or_ack_reply_closes_without_republishing_or_false_ack() {
    for ack in [false, true] {
        let model = Model::new();
        model.borrow_mut().key();
        let mut session = open_session(&model);
        let first = if ack { Some(offer(&mut session)) } else { None };
        model.borrow_mut().fault = Fault::DropReply;
        let result = if let Some(first) = first {
            session.ack(&first).map(|_| ())
        } else {
            session.peek().map(|_| ())
        };
        assert!(matches!(
            result,
            Err(Error::Interrupted {
                delivery: Delivery::Enqueued,
                ..
            })
        ));
        assert!(session.is_closed());
        let count = model.borrow().publications;
        assert_eq!(session.peek(), Err(Error::Closed));
        assert_eq!(model.borrow().publications, count);
        assert_eq!(model.borrow().closed.len(), 2);
        assert_eq!(model.borrow().service.queued(), usize::from(!ack));
    }
}

#[test]
fn only_real_send_eagain_retries_before_enqueue_and_receive_eagain_never_resends() {
    let model = Model::new();
    model.borrow_mut().key();
    {
        let mut m = model.borrow_mut();
        let send = m.app_send;
        for _ in 0..16 {
            m.engine.send(CLIENT, send, &Message::new()).unwrap();
        }
        m.queue_full = true;
        m.defer_reply = true;
    }
    let mut session = open_session(&model);
    offer(&mut session);
    let m = model.borrow();
    assert_eq!((m.attempts, m.publications, m.requests), (2, 1, 1));
    assert_eq!(m.request_ids, [1]);
    assert_eq!(m.yields, 2);
}

#[test]
fn unknown_send_or_failed_receive_cannot_be_retried_as_no_publication() {
    for fault in [Fault::UnknownSend, Fault::Receive, Fault::Yield] {
        let model = Model::new();
        model.borrow_mut().key();
        model.borrow_mut().fault = fault;
        if matches!(fault, Fault::Yield) {
            model.borrow_mut().defer_reply = true;
        }
        let mut session = open_session(&model);
        let result = session.peek();
        assert!(matches!(
            result,
            Err(Error::Interrupted {
                delivery: Delivery::Enqueued | Delivery::Unknown,
                ..
            })
        ));
        assert_eq!(model.borrow().attempts, 1);
        assert_eq!(model.borrow().publications, 1);
        assert_eq!(model.borrow().closed.len(), 2);
    }
}

#[test]
fn hostile_envelopes_and_noncanonical_key_or_header_fields_never_offer_input() {
    let mutations = [
        Fault::WrongSender,
        Fault::WrongTag,
        Fault::Short,
        Fault::Mutate(0, b'X'),
        Fault::Mutate(4, 2),
        Fault::Mutate(6, Operation::Ack as u8),
        Fault::Mutate(8, 2),
        Fault::Mutate(16, 8),
        Fault::Mutate(32, 9),
        Fault::Mutate(24, 19),
        Fault::Mutate(24, 1),
        Fault::Mutate(40, 0),
        Fault::Mutate(41, 0),
        Fault::Mutate(42, 255),
        Fault::Mutate(43, 1),
        Fault::Mutate(45, 0x80),
        Fault::Mutate(46, 1),
        Fault::Mutate(48, 0),
        Fault::Mutate(56, 0),
        Fault::Mutate(64, 1),
        Fault::Mutate(95, 1),
    ];
    for fault in mutations {
        let model = Model::new();
        model.borrow_mut().key();
        model.borrow_mut().fault = fault;
        let mut session = open_session(&model);
        assert!(matches!(
            session.peek(),
            Err(Error::Interrupted {
                delivery: Delivery::Enqueued,
                protocol: Some(_),
                ..
            })
        ));
        assert!(session.is_closed());
        assert_eq!(model.borrow().closed.len(), 2);
    }
}

#[test]
fn unexpected_actual_transferred_cap_is_closed_once_with_owned_channels_even_when_close_fails() {
    let model = Model::new();
    model.borrow_mut().key();
    {
        let mut m = model.borrow_mut();
        m.fault = Fault::Caps;
        m.close_failure = Some(m.app_send);
    }
    let mut session = open_session(&model);
    assert!(matches!(
        session.peek(),
        Err(Error::Interrupted {
            cleanup: Some(_),
            ..
        })
    ));
    session.close().unwrap();
    let m = model.borrow();
    let extra = m.received_cap.unwrap();
    assert_eq!(m.closed.len(), 3);
    assert_eq!(
        m.closed.iter().filter(|handle| **handle == extra).count(),
        1
    );
    assert!(m.engine.check_endpoint(CLIENT, extra, SEND).is_err());
    assert!(m.engine.check_invariants());
}

#[test]
fn malformed_ack_is_not_acknowledged_and_reserved_mouse_bits_or_outside_gop_are_rejected() {
    let model = Model::new();
    model.borrow_mut().key();
    let mut session = open_session(&model);
    let first = offer(&mut session);
    model.borrow_mut().fault = Fault::Mutate(24, 1);
    assert!(matches!(
        session.ack(&first),
        Err(Error::Interrupted {
            protocol: Some(_),
            ..
        })
    ));
    for (at, value) in [(41, 4), (42, 8), (43, 8), (65, 5), (69, 3), (45, 2)] {
        let model = Model::new();
        {
            let mut m = model.borrow_mut();
            m.sync();
            m.control(ID);
            for b in [9, 7, 3] {
                m.sample(0x21, b);
            }
            m.fault = Fault::Mutate(at, value);
        }
        let mut session = open_session(&model);
        assert!(matches!(
            session.peek(),
            Err(Error::Interrupted {
                protocol: Some(_),
                ..
            })
        ));
    }
}

#[test]
fn replayed_event_sequence_or_older_key_generation_cannot_be_offered_again() {
    for offset in [48, 56] {
        let model = Model::new();
        model.borrow_mut().key();
        let mut session = open_session(&model);
        let first = offer(&mut session);
        session.ack(&first).unwrap();
        model.borrow_mut().sample(1, 0x1e);
        model.borrow_mut().fault = Fault::Mutate(offset, 1);
        assert!(matches!(
            session.peek(),
            Err(Error::Interrupted {
                protocol: Some(_),
                ..
            })
        ));
        assert!(session.is_closed());
    }
}

#[test]
fn canonical_service_denial_is_not_empty_and_closes_the_uncertain_binding() {
    let model = Model::new();
    {
        let mut m = model.borrow_mut();
        // A preexisting request history is a genuine incompatible service
        // binding; construct it through real app-PID/ingress metadata.
        let send = m.app_send;
        m.actor(
            3,
            CLIENT,
            send,
            Request {
                operation: Operation::Peek,
                id: 100,
                epoch: EPOCH,
                arg0: 0,
                arg1: 0,
            },
        );
    }
    let mut session = open_session(&model);
    assert_eq!(
        session.peek(),
        Err(Error::Service {
            status: nieuw_os_input_client::ServiceError::StaleRequest,
            cleanup: None,
        })
    );
    assert!(session.is_closed());
    assert_eq!(model.borrow().closed.len(), 2);
}

#[test]
fn owned_pair_drops_once_while_invalid_construction_never_takes_ownership() {
    let model = Model::new();
    let mut binding = model.borrow().binding();
    binding.epoch = 0;
    assert!(matches!(
        Session::new(Port(model.clone()), binding, 4),
        Err(Error::InvalidBinding)
    ));
    assert_eq!(model.borrow().closed.len(), 0);
    {
        let mut session = open_session(&model);
        assert_eq!(session.peek(), Ok(Peek::Empty));
    }
    let m = model.borrow();
    assert_eq!(m.closed, [m.app_send, m.reply_root]);
}

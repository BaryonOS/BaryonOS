//! Actual capability and input engines with a small test-only CLP responder.
//! This proves the client contract, not native clipboard broker execution.
use capability_core::{Cap, Engine, Message, RECV, SEND, TRANSFER};
use nieuw_os_input_client::{
    clipboard::{self as clip, Header, Operation, Reply, Request, Status},
    AckOutcome, Binding, Delivery, Error as InputError, Event, Offer, Peek, Session, Transport,
};
use nieuw_os_input_core::{
    self as input,
    wire::{self as iw, Operation as InputOperation, Request as InputRequest},
};
use nieuw_os_sdk_runtime::{
    Error as RuntimeError, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};
use std::{cell::RefCell, rc::Rc};
use ui_core::{KeyAction, PhysicalKey};

const APP: u64 = 4;
const HOSTILE: u64 = 9;
const SERVER: u64 = 10;
const EPOCH: u64 = 37;
const CLIENT: u64 = 101;
fn handle(raw: u64) -> Handle {
    Handle::from_raw(raw).unwrap()
}
fn recv(raw: u64) -> ReceiveEndpoint {
    ReceiveEndpoint::from_handle(handle(raw))
}
fn send(raw: u64) -> SendEndpoint {
    SendEndpoint::from_handle(handle(raw))
}

#[derive(Clone, Copy, Default)]
enum Fault {
    #[default]
    None,
    WrongSender,
    WrongTag,
    Mutate(usize, u8),
    Short,
    ExtraByte,
    Caps,
    DropReply,
    UnknownSend,
    Receive,
    AgainOnce,
}
struct Host {
    engine: Box<Engine>,
    input: input::Service,
    roots: [u64; 4],
    source_send: u64,
    app_send: u64,
    app_receive: u64,
    reply: u64,
    hostile_reply: u64,
    transferable: u64,
    source_request: u64,
    fault: Fault,
    semantic: Status,
    text: String,
    request_ids: Vec<u64>,
    clip_requests: usize,
    clip_attempts: usize,
    yields: usize,
    closed: Vec<u64>,
    transferred: Option<u64>,
}
impl Host {
    fn new() -> Rc<RefCell<Self>> {
        let mut engine = Box::new(Engine::new());
        for pid in [1, 2, 3, APP, HOSTILE, SERVER] {
            engine.register(pid).unwrap();
        }
        let roots = core::array::from_fn(|_| engine.create_endpoint(SERVER).unwrap());
        let source_send = engine
            .bootstrap_delegate(SERVER, roots[0], 1, SEND)
            .unwrap();
        let focus_send = engine
            .bootstrap_delegate(SERVER, roots[1], 2, SEND)
            .unwrap();
        let app_send = engine
            .bootstrap_delegate(SERVER, roots[3], APP, SEND)
            .unwrap();
        let root = engine.create_endpoint(APP).unwrap();
        let app_receive = engine.derive(APP, root, RECV).unwrap();
        let reply = engine.bootstrap_delegate(APP, root, SERVER, SEND).unwrap();
        let hostile_reply = engine.bootstrap_delegate(APP, root, HOSTILE, SEND).unwrap();
        engine.close(APP, root).unwrap();
        let root = engine.create_endpoint(1).unwrap();
        let transferable = engine
            .bootstrap_delegate(1, root, SERVER, SEND | TRANSFER)
            .unwrap();
        let input = input::Service::new(
            input::Configuration {
                epoch: EPOCH,
                source: input::Actor {
                    endpoint: recv(roots[0]),
                    pid: 1,
                },
                compositor: input::Actor {
                    endpoint: recv(roots[1]),
                    pid: 2,
                },
                supervisor: input::Actor {
                    endpoint: recv(roots[2]),
                    pid: 3,
                },
                applications: recv(roots[3]),
                width: 1280,
                height: 720,
            },
            &[input::ClientBinding {
                id: CLIENT,
                pid: APP,
            }],
        )
        .unwrap();
        let mut host = Self {
            engine,
            input,
            roots,
            source_send,
            app_send,
            app_receive,
            reply,
            hostile_reply,
            transferable,
            source_request: 0,
            fault: Fault::None,
            semantic: Status::Ok,
            text: String::new(),
            request_ids: vec![],
            clip_requests: 0,
            clip_attempts: 0,
            yields: 0,
            closed: vec![],
            transferred: None,
        };
        host.source(InputOperation::Sync, 1, 0);
        host.actor(
            1,
            2,
            focus_send,
            InputRequest {
                operation: InputOperation::Focus,
                id: 1,
                epoch: EPOCH,
                arg0: CLIENT,
                arg1: 0,
            },
        );
        Rc::new(RefCell::new(host))
    }
    fn actor(&mut self, role: usize, pid: u64, endpoint: u64, request: InputRequest) {
        let mut message = Message::new();
        message.tag = iw::TAG;
        message.len = iw::SIZE as u32;
        message.data[..iw::SIZE].copy_from_slice(&request.encode());
        self.engine.send(pid, endpoint, &message).unwrap();
        let actual = self.engine.receive(SERVER, self.roots[role]).unwrap();
        let incoming = Received::from_bytes(actual.encode(), actual.len as u64).unwrap();
        let response = self
            .input
            .handle(recv(self.roots[role]), &incoming)
            .unwrap();
        assert!(self.input.reply_is_current(&response));
    }
    fn source(&mut self, operation: InputOperation, arg0: u64, arg1: u64) {
        self.source_request += 1;
        self.actor(
            0,
            1,
            self.source_send,
            InputRequest {
                operation,
                id: self.source_request,
                epoch: EPOCH,
                arg0,
                arg1,
            },
        );
    }
    fn key(&mut self, scan: u8) {
        self.source(InputOperation::Sample, 1, u64::from(scan));
    }
    fn binding(&self) -> Binding {
        Binding {
            epoch: EPOCH,
            service_pid: SERVER,
            client_id: CLIENT,
            width: 1280,
            height: 720,
            send: send(self.app_send),
            receive: recv(self.app_receive),
        }
    }
    fn respond(&mut self, actual: Message) {
        assert_eq!(actual.sender_pid, APP);
        assert_eq!(actual.cap_count, 0);
        let mut response = Message::new();
        if actual.tag == iw::TAG {
            let request = InputRequest::decode(&actual.data[..actual.len as usize], EPOCH).unwrap();
            self.request_ids.push(request.id);
            let incoming = Received::from_bytes(actual.encode(), actual.len as u64).unwrap();
            let reply = self.input.handle(recv(self.roots[3]), &incoming).unwrap();
            response.tag = iw::TAG;
            response.len = iw::SIZE as u32;
            response.data[..iw::SIZE].copy_from_slice(reply.payload());
            self.engine.send(SERVER, self.reply, &response).unwrap();
            return;
        }
        assert_eq!(actual.tag, clip::TAG);
        self.clip_requests += 1;
        let request = Request::decode(&actual.data[..actual.len as usize]).unwrap();
        self.request_ids.push(request.header.request);
        let mut status = self.semantic;
        if status == Status::Ok && request.header.operation == Operation::Copy {
            self.text = request.text.to_owned();
        }
        if status == Status::Ok
            && request.header.operation == Operation::Paste
            && self.text.is_empty()
        {
            status = Status::Empty;
        }
        let text = if status == Status::Ok && request.header.operation == Operation::Paste {
            self.text.as_str()
        } else {
            ""
        };
        response.tag = clip::TAG;
        response.len = Reply {
            header: request.header,
            status,
            text,
        }
        .encode(&mut response.data)
        .unwrap() as u32;
        match self.fault {
            Fault::DropReply => return,
            Fault::WrongTag => response.tag ^= 1,
            Fault::Mutate(at, value) => response.data[at] = value,
            Fault::Short => response.len -= 1,
            Fault::ExtraByte => response.len += 1,
            Fault::Caps => {
                response.cap_count = 1;
                response.caps[0] = Cap {
                    handle: self.transferable,
                    rights: SEND,
                };
            }
            _ => {}
        }
        if matches!(self.fault, Fault::WrongSender) {
            self.engine
                .send(HOSTILE, self.hostile_reply, &response)
                .unwrap();
        } else {
            self.engine.send(SERVER, self.reply, &response).unwrap();
        }
    }
}
struct Port(Rc<RefCell<Host>>);
impl Transport for Port {
    fn send(&mut self, endpoint: SendEndpoint, outgoing: &Outgoing) -> Result<(), RuntimeError> {
        let mut host = self.0.borrow_mut();
        let request = Message::decode(outgoing.as_bytes()).unwrap();
        let clipboard = request.tag == clip::TAG;
        if clipboard {
            host.clip_attempts += 1;
            if matches!(host.fault, Fault::AgainOnce) && host.clip_attempts == 1 {
                return Err(RuntimeError::Kernel(capability_core::Error::Again.code()));
            }
        }
        host.engine
            .send(APP, endpoint.handle().raw(), &request)
            .map_err(|e| RuntimeError::Kernel(e.code()))?;
        let ingress = host.roots[3];
        let actual = host.engine.receive(SERVER, ingress).unwrap();
        host.respond(actual);
        if clipboard && matches!(host.fault, Fault::UnknownSend) {
            Err(RuntimeError::InvalidReturn)
        } else {
            Ok(())
        }
    }
    fn receive(&mut self, endpoint: ReceiveEndpoint) -> Result<Received, RuntimeError> {
        let mut host = self.0.borrow_mut();
        if matches!(host.fault, Fault::Receive) && host.clip_requests != 0 {
            return Err(RuntimeError::InvalidReturn);
        }
        let actual = host
            .engine
            .receive(APP, endpoint.handle().raw())
            .map_err(|e| RuntimeError::Kernel(e.code()))?;
        let message = Received::from_bytes(actual.encode(), actual.len as u64)?;
        host.transferred = message.cap(0).map(|cap| cap.handle.raw());
        Ok(message)
    }
    fn yield_now(&mut self) -> Result<(), RuntimeError> {
        self.0.borrow_mut().yields += 1;
        Ok(())
    }
    fn close(&mut self, handle: Handle) -> Result<(), RuntimeError> {
        let mut host = self.0.borrow_mut();
        host.closed.push(handle.raw());
        host.engine
            .close(APP, handle.raw())
            .map_err(|e| RuntimeError::Kernel(e.code()))
    }
}
fn session(host: &Rc<RefCell<Host>>) -> Session<Port> {
    Session::new(Port(host.clone()), host.borrow().binding(), 4).unwrap()
}
fn offer(session: &mut Session<Port>) -> Offer {
    match session.peek().unwrap() {
        Peek::Offer(offer) => offer,
        Peek::Empty => panic!("expected actual input event"),
    }
}
fn shortcut(host: &Rc<RefCell<Host>>, session: &mut Session<Port>, scan: u8) -> Offer {
    host.borrow_mut().key(0x1d);
    let control = offer(session);
    assert!(matches!(control.event(), Event::Key(k) if k.key == PhysicalKey::LeftControl));
    assert_eq!(session.ack(&control), Ok(AckOutcome::Acknowledged));
    host.borrow_mut().key(scan);
    let result = offer(session);
    assert!(
        matches!(result.event(), Event::Key(k) if k.action == KeyAction::Press && k.modifiers.control())
    );
    result
}
fn assert_protocol_failure(
    result: Result<(), clip::Error>,
    session: &Session<Port>,
    host: &Rc<RefCell<Host>>,
) {
    assert!(matches!(
        result,
        Err(clip::Error::Input(InputError::Interrupted { .. }))
    ));
    assert!(session.is_closed());
    assert!(session.outstanding().is_none());
    assert!(host.borrow().engine.check_invariants());
}

#[test]
fn copy_and_paste_share_actual_input_request_ids_and_leave_offer_for_normal_ack() {
    let host = Host::new();
    let mut session = session(&host);
    let copy = shortcut(&host, &mut session, 0x2e);
    let text = "Material: café e\u{301} — 日本語\n\0";
    session.clipboard_copy(&copy, text).unwrap();
    assert_eq!(session.outstanding(), Some(copy));
    assert_eq!(session.clipboard_copy(&copy, text), Err(clip::Error::Used));
    assert_eq!(host.borrow().clip_requests, 1);
    assert_eq!(session.ack(&copy), Ok(AckOutcome::Acknowledged));
    assert!(session.clipboard_copy(&copy, "old offer").is_err());
    host.borrow_mut().key(0xae);
    let release = offer(&mut session);
    session.ack(&release).unwrap();
    host.borrow_mut().key(0x2f);
    let paste = offer(&mut session);
    let received = session.clipboard_paste(&paste).unwrap();
    assert_eq!(received.as_str(), text);
    assert_eq!(received.len(), text.len());
    assert!(!received.is_empty());
    assert!(session.clipboard_paste(&paste).is_err());
    assert_eq!(session.ack(&paste), Ok(AckOutcome::Acknowledged));
    assert!(!session.is_closed());
    assert_eq!(host.borrow().clip_requests, 2);
    assert_eq!(
        host.borrow().request_ids,
        (1..=host.borrow().request_ids.len() as u64).collect::<Vec<_>>()
    );
}

#[test]
fn only_exact_unmodified_control_press_offer_can_publish_clipboard_work() {
    for modifier in [None, Some(0x2a), Some(0x38), Some(0x5b)] {
        let host = Host::new();
        let mut session = session(&host);
        if let Some(scan) = modifier {
            if scan == 0x5b {
                host.borrow_mut().key(0xe0);
            }
            host.borrow_mut().key(scan);
            let extra = offer(&mut session);
            session.ack(&extra).unwrap();
        }
        let key = if modifier.is_none() {
            host.borrow_mut().key(0x2e);
            offer(&mut session)
        } else {
            shortcut(&host, &mut session, 0x2e)
        };
        let sent = host.borrow().request_ids.len();
        assert!(session.clipboard_copy(&key, "synthetic").is_err());
        assert_eq!(host.borrow().request_ids.len(), sent);
        assert_eq!(host.borrow().clip_requests, 0);
        assert!(!session.is_closed());
        session.ack(&key).unwrap();
    }
    for scan in [0x2e, 0xae] {
        let host = Host::new();
        let mut session = session(&host);
        let first = shortcut(&host, &mut session, 0x2e);
        session.ack(&first).unwrap();
        host.borrow_mut().key(scan);
        let repeat_or_release = offer(&mut session);
        assert!(matches!(repeat_or_release.event(), Event::Key(k) if k.action != KeyAction::Press));
        assert!(session
            .clipboard_copy(&repeat_or_release, "synthetic")
            .is_err());
        assert_eq!(host.borrow().clip_requests, 0);
    }
    let host = Host::new();
    let mut session = session(&host);
    let copy = shortcut(&host, &mut session, 0x2e);
    assert!(session.clipboard_paste(&copy).is_err());
    assert_eq!(host.borrow().clip_requests, 0);
}

#[test]
fn local_selection_bounds_preserve_bytes_and_do_not_publish_invalid_requests() {
    let host = Host::new();
    let mut session = session(&host);
    let key = shortcut(&host, &mut session, 0x2e);
    assert_eq!(session.clipboard_copy(&key, ""), Err(clip::Error::Empty));
    assert_eq!(
        session.clipboard_copy(&key, &"x".repeat(clip::MAX_TEXT + 1)),
        Err(clip::Error::TooLong)
    );
    assert_eq!(host.borrow().clip_requests, 0);
    let full = "é".repeat(clip::MAX_TEXT / 2);
    session.clipboard_copy(&key, &full).unwrap();
    assert_eq!(host.borrow().text, full);
    assert_eq!(host.borrow().clip_requests, 1);
    assert!(!session.is_closed());
}

#[test]
fn semantic_refusals_are_one_use_but_keep_transport_open_and_allow_exact_input_ack() {
    for status in [
        Status::Denied,
        Status::Empty,
        Status::TooLong,
        Status::InvalidText,
        Status::Used,
    ] {
        let host = Host::new();
        let mut session = session(&host);
        let key = shortcut(&host, &mut session, 0x2f);
        host.borrow_mut().semantic = status;
        let expected = match status {
            Status::Denied => clip::Error::Denied,
            Status::Empty => clip::Error::Empty,
            Status::TooLong => clip::Error::TooLong,
            Status::InvalidText => clip::Error::InvalidText,
            Status::Used => clip::Error::Used,
            Status::Ok => unreachable!(),
        };
        assert_eq!(session.clipboard_paste(&key).err(), Some(expected));
        assert!(!session.is_closed());
        assert_eq!(session.outstanding(), Some(key));
        assert_eq!(session.clipboard_paste(&key).err(), Some(clip::Error::Used));
        assert_eq!(host.borrow().clip_requests, 1);
        assert_eq!(session.ack(&key), Ok(AckOutcome::Acknowledged));
    }
}

#[test]
fn wrong_peer_tag_operation_request_epoch_sequence_generation_and_padding_close_the_pair() {
    for fault in [
        Fault::WrongSender,
        Fault::WrongTag,
        Fault::Mutate(6, 2),
        Fault::Mutate(7, 0),
        Fault::Mutate(8, 200),
        Fault::Mutate(16, 200),
        Fault::Mutate(24, 200),
        Fault::Mutate(32, 200),
        Fault::Mutate(40, 193),
        Fault::Mutate(42, 6),
        Fault::Mutate(44, 1),
        Fault::Mutate(63, 1),
        Fault::Short,
        Fault::ExtraByte,
    ] {
        let host = Host::new();
        let mut session = session(&host);
        let key = shortcut(&host, &mut session, 0x2e);
        host.borrow_mut().fault = fault;
        let result = session.clipboard_copy(&key, "synthetic");
        assert_protocol_failure(result, &session, &host);
        assert_eq!(host.borrow().clip_requests, 1);
        assert_eq!(host.borrow().closed.len(), 2);
    }
}

#[test]
fn unexpected_transferred_capability_is_actually_closed_before_session_failure() {
    let host = Host::new();
    let mut session = session(&host);
    let key = shortcut(&host, &mut session, 0x2e);
    host.borrow_mut().fault = Fault::Caps;
    let result = session.clipboard_copy(&key, "synthetic");
    assert_protocol_failure(result, &session, &host);
    let host = host.borrow();
    let cap = host.transferred.unwrap();
    assert!(host.closed.contains(&cap));
    assert_eq!(host.closed.len(), 3);
    assert_eq!(
        host.engine.check_endpoint(APP, cap, SEND),
        Err(capability_core::Error::BadHandle)
    );
}

#[test]
fn invalid_paste_text_or_empty_success_never_yields_owned_plaintext() {
    for (fault, semantic) in [
        (Fault::Mutate(64, 0xff), Status::Ok),
        (Fault::Mutate(40, 193), Status::Ok),
        (Fault::ExtraByte, Status::Ok),
        (Fault::Mutate(42, 0), Status::Denied),
    ] {
        let host = Host::new();
        let mut session = session(&host);
        let key = shortcut(&host, &mut session, 0x2f);
        {
            let mut broker = host.borrow_mut();
            broker.text = "a".to_owned();
            broker.semantic = semantic;
            broker.fault = fault;
        }
        let result = session.clipboard_paste(&key).map(|_| ());
        assert_protocol_failure(result, &session, &host);
        assert_eq!(host.borrow().clip_requests, 1);
        assert_eq!(host.borrow().closed.len(), 2);
    }
}

#[test]
fn right_control_shortcut_is_an_actual_eligible_offer() {
    let host = Host::new();
    let mut session = session(&host);
    host.borrow_mut().key(0xe0);
    host.borrow_mut().key(0x1d);
    let control = offer(&mut session);
    assert!(matches!(control.event(), Event::Key(key) if key.key == PhysicalKey::RightControl));
    session.ack(&control).unwrap();
    host.borrow_mut().key(0x2e);
    let copy = offer(&mut session);
    session.clipboard_copy(&copy, "synthetic").unwrap();
    assert_eq!(host.borrow().clip_requests, 1);
    assert_eq!(session.ack(&copy), Ok(AckOutcome::Acknowledged));
}

#[test]
fn uncertain_or_lost_reply_closes_without_republishing_or_automatic_replay() {
    for fault in [Fault::UnknownSend, Fault::DropReply, Fault::Receive] {
        let host = Host::new();
        let mut session = session(&host);
        let key = shortcut(&host, &mut session, 0x2e);
        host.borrow_mut().fault = fault;
        let error = session.clipboard_copy(&key, "synthetic").unwrap_err();
        assert!(matches!(
            error,
            clip::Error::Input(InputError::Interrupted {
                delivery: Delivery::Unknown | Delivery::Enqueued,
                ..
            })
        ));
        assert!(session.is_closed());
        assert_eq!(host.borrow().clip_requests, 1);
        assert!(session.clipboard_copy(&key, "must not replay").is_err());
        assert_eq!(host.borrow().clip_requests, 1);
        assert_eq!(host.borrow().closed.len(), 2);
    }
    let host = Host::new();
    let mut session = session(&host);
    let key = shortcut(&host, &mut session, 0x2e);
    host.borrow_mut().fault = Fault::AgainOnce;
    session.clipboard_copy(&key, "synthetic").unwrap();
    assert_eq!(host.borrow().clip_attempts, 2);
    assert_eq!(host.borrow().clip_requests, 1);
    assert!(!session.is_closed());
}

#[test]
fn wire_rejects_noncanonical_lengths_utf8_headers_and_success_bodies() {
    let header = Header {
        operation: Operation::Copy,
        request: 1,
        epoch: 2,
        sequence: 3,
        generation: 4,
    };
    let mut canonical = [0; clip::MAX_PAYLOAD];
    let length = Request { header, text: "é" }
        .encode(&mut canonical)
        .unwrap();
    assert_eq!(Request::decode(&canonical[..length]).unwrap().text, "é");
    assert_eq!(length, clip::HEADER + 2);
    for mutation in [
        (0, b'X'),
        (4, 2),
        (6, 3),
        (7, 0x80),
        (8, 0),
        (16, 0),
        (24, 0),
        (32, 0),
        (40, 193),
        (42, 1),
        (44, 1),
        (64, 0xff),
    ] {
        let mut bad = canonical;
        bad[mutation.0] = mutation.1;
        assert!(Request::decode(&bad[..length]).is_err());
    }
    assert!(Request::decode(&canonical[..clip::HEADER - 1]).is_err());
    assert!(Request::decode(&[0; clip::MAX_PAYLOAD + 1]).is_err());
    assert!(Request { header, text: "" }.encode(&mut canonical).is_err());
    assert!(Reply {
        header,
        status: Status::Ok,
        text: "extra"
    }
    .encode(&mut canonical)
    .is_err());
    let paste = Header {
        operation: Operation::Paste,
        ..header
    };
    assert!(Request {
        header: paste,
        text: "extra"
    }
    .encode(&mut canonical)
    .is_err());
    assert!(Reply {
        header: paste,
        status: Status::Ok,
        text: ""
    }
    .encode(&mut canonical)
    .is_err());
    assert!(Reply {
        header: paste,
        status: Status::Denied,
        text: "extra"
    }
    .encode(&mut canonical)
    .is_err());
}

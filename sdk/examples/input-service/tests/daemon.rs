//! Host adapter tests over the real capability engine. Raw fixtures below test
//! only source sequencing; they never count as native PS/2/driver evidence.
use capability_core::{Engine, Message, SEND};
use nieuw_os_display_client::{
    wire::{Reset, Sample},
    Error as RawError, ResetProgress,
};
use nieuw_os_input_core::{
    wire::{self, Operation, Request},
    Actor, ClientBinding, Configuration,
};
use nieuw_os_native_input_service::hid::{
    Error as HidError, Progress as HidProgress, Source as HidSource,
};
use nieuw_os_native_input_service::{
    policy::{Client, Policy, Replies},
    source::{self, Progress, RawInput, Source},
    Ipc,
};
use nieuw_os_sdk_runtime::{
    Capability, Error, Handle, Outgoing, ReceiveEndpoint, Received, Rights, SendEndpoint,
};
use nieuw_os_usb_core::{
    boot_interface::Speed,
    boot_session::{CompletionStatus::*, Error as SessionError, Lifetime, Session},
};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

const POLICY: u64 = 10;
const EPOCH: u64 = 70;
fn recv(raw: u64) -> ReceiveEndpoint {
    ReceiveEndpoint::from_handle(Handle::from_raw(raw).unwrap())
}
fn send(raw: u64) -> SendEndpoint {
    SendEndpoint::from_handle(Handle::from_raw(raw).unwrap())
}
fn number(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}
fn request(op: Operation, id: u64, arg0: u64) -> Outgoing {
    Outgoing::new(
        wire::TAG,
        &Request {
            operation: op,
            id,
            epoch: EPOCH,
            arg0,
            arg1: 0,
        }
        .encode(),
    )
    .unwrap()
}
struct Port {
    pid: u64,
    engine: Rc<RefCell<Box<Engine>>>,
    sends: usize,
    closed: Vec<u64>,
}
impl Ipc for Port {
    fn send(&mut self, endpoint: SendEndpoint, message: &Outgoing) -> Result<(), Error> {
        self.sends += 1;
        self.engine
            .borrow_mut()
            .send(
                self.pid,
                endpoint.handle().raw(),
                &Message::decode(message.as_bytes()).unwrap(),
            )
            .map_err(|e| Error::Kernel(e.code()))
    }
    fn receive(&mut self, endpoint: ReceiveEndpoint) -> Result<Received, Error> {
        let message = self
            .engine
            .borrow_mut()
            .receive(self.pid, endpoint.handle().raw())
            .map_err(|e| Error::Kernel(e.code()))?;
        Received::from_bytes(message.encode(), u64::from(message.len))
    }
    fn close(&mut self, handle: Handle) -> Result<(), Error> {
        self.closed.push(handle.raw());
        self.engine
            .borrow_mut()
            .close(self.pid, handle.raw())
            .map_err(|e| Error::Kernel(e.code()))
    }
}
struct Model {
    engine: Rc<RefCell<Box<Engine>>>,
    policy: Policy,
    ingress: [u64; 4],
    requests: [u64; 5],
    responses: [u64; 5],
    reply_send: [u64; 5],
    ports: Vec<Port>,
    policy_io: Port,
}
impl Model {
    fn new() -> Self {
        let mut engine = Box::new(Engine::new());
        for pid in [1, 2, 3, 4, 5, 6, POLICY] {
            engine.register(pid).unwrap();
        }
        let ingress = core::array::from_fn(|_| engine.create_endpoint(POLICY).unwrap());
        let requests = core::array::from_fn(|i| {
            engine
                .bootstrap_delegate(POLICY, ingress[i.min(3)], i as u64 + 1, SEND)
                .unwrap()
        });
        let responses = core::array::from_fn(|i| engine.create_endpoint(i as u64 + 1).unwrap());
        let reply_send = core::array::from_fn(|i| {
            engine
                .bootstrap_delegate(i as u64 + 1, responses[i], POLICY, SEND)
                .unwrap()
        });
        let config = Configuration {
            epoch: EPOCH,
            source: Actor {
                endpoint: recv(ingress[0]),
                pid: 1,
            },
            compositor: Actor {
                endpoint: recv(ingress[1]),
                pid: 2,
            },
            supervisor: Actor {
                endpoint: recv(ingress[2]),
                pid: 3,
            },
            applications: recv(ingress[3]),
            width: 800,
            height: 600,
        };
        let policy = Policy::new(
            config,
            Replies {
                source: send(reply_send[0]),
                compositor: send(reply_send[1]),
                supervisor: send(reply_send[2]),
            },
            &[
                Client {
                    binding: ClientBinding { id: 101, pid: 4 },
                    reply: send(reply_send[3]),
                },
                Client {
                    binding: ClientBinding { id: 202, pid: 5 },
                    reply: send(reply_send[4]),
                },
            ],
        )
        .unwrap();
        let engine = Rc::new(RefCell::new(engine));
        let ports = (1..=5)
            .map(|pid| Port {
                pid,
                engine: Rc::clone(&engine),
                sends: 0,
                closed: Vec::new(),
            })
            .collect();
        let policy_io = Port {
            pid: POLICY,
            engine: Rc::clone(&engine),
            sends: 0,
            closed: Vec::new(),
        };
        Self {
            engine,
            policy,
            ingress,
            requests,
            responses,
            reply_send,
            ports,
            policy_io,
        }
    }
    fn step(&mut self) {
        self.policy.step(&mut self.policy_io).unwrap();
    }
    fn call(&mut self, index: usize, message: Outgoing) -> Received {
        self.ports[index]
            .send(send(self.requests[index]), &message)
            .unwrap();
        self.step();
        self.ports[index]
            .receive(recv(self.responses[index]))
            .unwrap()
    }
    fn start(&mut self) {
        assert_eq!(
            number(self.call(0, request(Operation::Sync, 1, 1)).payload(), 24),
            0
        );
        assert_eq!(
            number(
                self.call(1, request(Operation::Focus, 1, 101)).payload(),
                24
            ),
            0
        );
    }
    fn sample(&mut self, id: u64, status: u8, byte: u8) -> Received {
        let message = Outgoing::new(
            wire::TAG,
            &Request {
                operation: Operation::Sample,
                id,
                epoch: EPOCH,
                arg0: status.into(),
                arg1: byte.into(),
            }
            .encode(),
        )
        .unwrap();
        self.call(0, message)
    }
    fn hid_source(&self) -> HidSource {
        HidSource::new(source::Configuration {
            epoch: EPOCH,
            policy_pid: POLICY,
            request: send(self.requests[0]),
            response: recv(self.responses[0]),
        })
        .unwrap()
    }
    fn hid_exchange(&mut self, source: &mut HidSource, now: u64, operation: Operation) {
        assert_eq!(
            source.step(now, &mut self.ports[0]),
            Ok(HidProgress::Pending)
        );
        self.step();
        assert_eq!(
            source.step(now + 1, &mut self.ports[0]),
            Ok(HidProgress::Acknowledged(operation))
        );
    }
}

fn hid_session(protocol: u8, generation: u64) -> Session {
    let config = [
        9, 2, 34, 0, 1, 1, 0, 0x80, 50, 9, 4, 0, 0, 1, 3, 1, protocol, 0, 9, 0x21, 0x11, 1, 0, 1,
        0x22, 63, 0, 7, 5, 0x81, 3, 8, 0, 10,
    ];
    let mut session = Session::new(
        Lifetime {
            device: 9,
            generation,
        },
        &config,
        0,
        Speed::Full,
    )
    .unwrap();
    let protocol = session.begin_protocol().unwrap();
    session
        .complete_protocol(protocol.token, Success, 0)
        .unwrap();
    session
}

fn hid_completion_publication_requires_sync_and_matching_valid_transport_work_impl() {
    let mut m = Model::new();
    let mut source = m.hid_source();
    let mut session = hid_session(1, 1);
    let input = session.begin_input().unwrap();
    let bytes = [0, 0, 4, 0, 0, 0, 0, 0];
    assert_eq!(
        source.complete_input(1, &mut session, input.token, Success, &bytes),
        Err(HidError::NotSynchronized)
    );
    source.synchronize(1).unwrap();
    assert_eq!(
        source.complete_input(1, &mut session, input.token, Success, &bytes),
        Err(HidError::Pending)
    );
    m.hid_exchange(&mut source, 1, Operation::Sync);
    m.call(1, request(Operation::Focus, 1, 101));
    source
        .complete_input(3, &mut session, input.token, Success, &bytes)
        .unwrap();
    m.hid_exchange(&mut source, 3, Operation::HidKeyboard);
    let event = m.call(3, request(Operation::Peek, 1, 0));
    assert_eq!(event.payload()[40], 1);
    let sends = m.ports[0].sends;
    assert_eq!(
        source.complete_input(5, &mut session, input.token, Success, &bytes),
        Err(HidError::Session(SessionError::UnexpectedCompletion))
    );
    for (status, data) in [
        (Failed, &bytes[..]),
        (Success, &[][..]),
        (Success, &[0; 9][..]),
    ] {
        let input = session.begin_input().unwrap();
        assert!(matches!(
            source.complete_input(5, &mut session, input.token, status, data),
            Err(HidError::Session(_))
        ));
        assert_eq!(source.step(5, &mut m.ports[0]), Ok(HidProgress::Idle));
    }
    assert_eq!(m.ports[0].sends, sends);
    let input = session.begin_input().unwrap();
    source
        .complete_input(6, &mut session, input.token, Success, &[0; 8])
        .unwrap();
    m.hid_exchange(&mut source, 6, Operation::HidKeyboard); // No skipped source ID.
}

fn hid_real_ipc_backpressure_retains_the_original_request_and_deadline_impl() {
    let mut m = Model::new();
    let mut source = m.hid_source();
    for _ in 0..16 {
        m.engine
            .borrow_mut()
            .send(1, m.requests[0], &Message::new())
            .unwrap();
    }
    source.synchronize(1).unwrap();
    assert_eq!(source.step(1, &mut m.ports[0]), Ok(HidProgress::Pending));
    assert_eq!(source.synchronize(2), Err(HidError::Pending));
    for _ in 0..16 {
        m.engine.borrow_mut().receive(POLICY, m.ingress[0]).unwrap();
    }
    m.hid_exchange(&mut source, 3, Operation::Sync);
    assert_eq!(m.ports[0].sends, 2); // One EAGAIN, then the original request once.
    assert_eq!(source.step(5, &mut m.ports[0]), Ok(HidProgress::Idle));
    source.synchronize(6).unwrap();
    assert_eq!(
        source.step(206, &mut m.ports[0]),
        Err(HidError::Fatal(source::Fatal::Deadline))
    );
    assert_eq!(m.ports[0].sends, 2);
}

fn hid_detach_invalidates_pending_usb_immediately_and_rejects_reattach_replay_impl() {
    let mut m = Model::new();
    let mut source = m.hid_source();
    source.synchronize(1).unwrap();
    m.hid_exchange(&mut source, 1, Operation::Sync);
    m.call(1, request(Operation::Focus, 1, 101));
    let mut old = hid_session(1, 1);
    let first = old.begin_input().unwrap();
    source
        .complete_input(3, &mut old, first.token, Success, &[0, 0, 4, 0, 0, 0, 0, 0])
        .unwrap();
    let late = old.begin_input().unwrap();
    assert_eq!(source.detach(3, &mut old), Err(HidError::Pending));
    assert_eq!(
        old.complete_input(late.token, Success, &[0; 8]),
        Err(SessionError::UnexpectedCompletion)
    );
    m.hid_exchange(&mut source, 3, Operation::HidKeyboard);
    source.detach(5, &mut old).unwrap();
    let mut new = hid_session(1, 2);
    let input = new.begin_input().unwrap();
    assert_eq!(
        source.complete_input(5, &mut new, input.token, Success, &[0; 8]),
        Err(HidError::Pending)
    );
    m.hid_exchange(&mut source, 5, Operation::HidDetach);
    assert_eq!(
        source.complete_input(7, &mut new, first.token, Success, &[0; 8]),
        Err(HidError::Session(SessionError::UnexpectedCompletion))
    );
    source
        .complete_input(7, &mut new, input.token, Success, &[0; 8])
        .unwrap();
    m.hid_exchange(&mut source, 7, Operation::HidKeyboard);
    let cancel = m.call(3, request(Operation::Peek, 1, 0));
    assert_eq!(&cancel.payload()[40..42], &[3, 2]);
}

fn hid_wrong_actual_reply_sender_cannot_acknowledge_sync_or_enable_publication_impl() {
    let mut m = Model::new();
    let mut source = m.hid_source();
    source.synchronize(1).unwrap();
    assert_eq!(source.step(1, &mut m.ports[0]), Ok(HidProgress::Pending));
    let forged_send = m
        .engine
        .borrow_mut()
        .bootstrap_delegate(1, m.responses[0], 2, SEND)
        .unwrap();
    let ack = wire::response(
        Request {
            operation: Operation::Sync,
            id: 1,
            epoch: EPOCH,
            arg0: wire::HID_PROFILE,
            arg1: 0,
        },
        0,
        None,
    );
    m.ports[1]
        .send(send(forged_send), &Outgoing::new(wire::TAG, &ack).unwrap())
        .unwrap();
    assert_eq!(
        source.step(2, &mut m.ports[0]),
        Err(HidError::Fatal(source::Fatal::Protocol))
    );
    assert_eq!(
        source.synchronize(3),
        Err(HidError::Fatal(source::Fatal::Protocol))
    );
}

fn hid_mouse_session_publishes_exact_completed_bytes_through_policy_impl() {
    let mut m = Model::new();
    let mut source = m.hid_source();
    source.synchronize(1).unwrap();
    m.hid_exchange(&mut source, 1, Operation::Sync);
    m.call(1, request(Operation::Focus, 1, 101));
    let mut mouse = hid_session(2, 1);
    let input = mouse.begin_input().unwrap();
    source
        .complete_input(3, &mut mouse, input.token, Success, &[1, 5, 7])
        .unwrap();
    m.hid_exchange(&mut source, 3, Operation::HidMouse);
    let event = m.call(3, request(Operation::Peek, 1, 0));
    assert_eq!(&event.payload()[40..44], &[2, 0, 1, 1]);
    assert_eq!(
        i16::from_le_bytes(event.payload()[44..46].try_into().unwrap()),
        5
    );
    assert_eq!(
        i16::from_le_bytes(event.payload()[46..48].try_into().unwrap()),
        -7
    );
}

fn hid_replayed_ack_remote_error_and_clock_reversal_fail_closed_impl() {
    for variant in 0..3 {
        let mut m = Model::new();
        let mut source = m.hid_source();
        source.synchronize(5).unwrap();
        if variant == 2 {
            assert_eq!(
                source.step(4, &mut m.ports[0]),
                Err(HidError::Fatal(source::Fatal::Clock))
            );
            continue;
        }
        assert_eq!(source.step(5, &mut m.ports[0]), Ok(HidProgress::Pending));
        let request = Request {
            operation: Operation::Sync,
            id: if variant == 0 { 2 } else { 1 },
            epoch: EPOCH,
            arg0: wire::HID_PROFILE,
            arg1: 0,
        };
        let ack = if variant == 0 {
            wire::response(request, 0, None)
        } else {
            wire::failure(request, 0, nieuw_os_input_core::Error::StreamLost)
        };
        m.policy_io
            .send(
                send(m.reply_send[0]),
                &Outgoing::new(wire::TAG, &ack).unwrap(),
            )
            .unwrap();
        let expected = if variant == 0 {
            source::Fatal::Protocol
        } else {
            source::Fatal::Remote(12)
        };
        assert_eq!(
            source.step(6, &mut m.ports[0]),
            Err(HidError::Fatal(expected))
        );
        assert_eq!(
            source.step(7, &mut m.ports[0]),
            Err(HidError::Fatal(source::Fatal::Protocol))
        );
    }
}

fn hid_unexpected_reply_capability_is_closed_before_terminal_rejection_impl() {
    let mut m = Model::new();
    let mut source = m.hid_source();
    source.synchronize(1).unwrap();
    assert_eq!(source.step(1, &mut m.ports[0]), Ok(HidProgress::Pending));
    let bytes = wire::response(
        Request {
            operation: Operation::Sync,
            id: 1,
            epoch: EPOCH,
            arg0: wire::HID_PROFILE,
            arg1: 0,
        },
        0,
        None,
    );
    let mut outgoing = Outgoing::new(wire::TAG, &bytes).unwrap();
    outgoing
        .push_cap(Capability {
            handle: Handle::from_raw(m.ingress[0]).unwrap(),
            rights: Rights::SEND,
        })
        .unwrap();
    m.policy_io.send(send(m.reply_send[0]), &outgoing).unwrap();
    assert_eq!(
        source.step(2, &mut m.ports[0]),
        Err(HidError::Fatal(source::Fatal::Protocol))
    );
    assert_eq!(m.ports[0].closed.len(), 1);
    assert_eq!(
        m.engine.borrow_mut().close(1, m.ports[0].closed[0]),
        Err(capability_core::Error::BadHandle)
    );
}
#[derive(Default)]
struct Raw {
    resets: u64,
    reads: u64,
    samples: VecDeque<Result<Option<Sample>, RawError>>,
}
impl RawInput for Raw {
    fn begin_reset(&mut self) -> Result<u64, RawError> {
        self.resets += 1;
        Ok(self.resets)
    }
    fn poll_reset(&mut self) -> Result<ResetProgress, RawError> {
        Ok(ResetProgress::Ready(Reset {
            ticket: self.resets,
            epoch: self.resets,
            discarded: 0,
            operations: 30,
        }))
    }
    fn read(&mut self) -> Result<Option<Sample>, RawError> {
        self.reads += 1;
        self.samples.pop_front().unwrap_or(Ok(None))
    }
}
fn source(model: &Model) -> Source {
    Source::new(source::Configuration {
        epoch: EPOCH,
        policy_pid: POLICY,
        request: send(model.requests[0]),
        response: recv(model.responses[0]),
    })
    .unwrap()
}
fn ready(model: &mut Model, source: &mut Source, raw: &mut Raw) {
    assert_eq!(
        source.step(0, raw, &mut model.ports[0]),
        Ok(Progress::Pending)
    );
    assert_eq!(
        source.step(1, raw, &mut model.ports[0]),
        Ok(Progress::Pending)
    );
    assert_eq!(
        source.step(2, raw, &mut model.ports[0]),
        Ok(Progress::Pending)
    );
    model.step();
    assert_eq!(
        source.step(3, raw, &mut model.ports[0]),
        Ok(Progress::Ready { raw_epoch: 1 })
    );
}
fn no_request_replay_while_real_reply_waits_impl() {
    let mut m = Model::new();
    let mut s = source(&m);
    let mut raw = Raw::default();
    ready(&mut m, &mut s, &mut raw);
    raw.samples.push_back(Ok(Some(Sample {
        epoch: 1,
        sequence: 1,
        status: 1,
        byte: 0x1e,
    })));
    assert_eq!(s.step(4, &mut raw, &mut m.ports[0]), Ok(Progress::Pending));
    assert_eq!(s.step(5, &mut raw, &mut m.ports[0]), Ok(Progress::Pending));
    let sends = m.ports[0].sends;
    for now in 6..40 {
        assert_eq!(
            s.step(now, &mut raw, &mut m.ports[0]),
            Ok(Progress::Pending)
        );
    }
    assert_eq!(m.ports[0].sends, sends);
    assert_eq!(raw.reads, 1);
    m.step();
    assert_eq!(
        s.step(40, &mut raw, &mut m.ports[0]),
        Ok(Progress::SampleAccepted)
    );
    assert_eq!(m.policy.counts().handled, 2);
    assert_eq!(s.counts().samples_acked, 1);
}
fn real_send_again_retains_same_source_request_without_reading_samples_impl() {
    let mut m = Model::new();
    let mut s = source(&m);
    let mut raw = Raw::default();
    let filler = Outgoing::new(1, &[]).unwrap();
    for _ in 0..16 {
        m.ports[0].send(send(m.requests[0]), &filler).unwrap();
    }
    assert!(m.ports[0]
        .send(send(m.requests[0]), &filler)
        .unwrap_err()
        .is_again());
    s.step(0, &mut raw, &mut m.ports[0]).unwrap();
    s.step(1, &mut raw, &mut m.ports[0]).unwrap();
    assert_eq!(s.step(2, &mut raw, &mut m.ports[0]), Ok(Progress::Pending));
    assert_eq!(s.counts().send_again, 1);
    assert_eq!(raw.reads, 0);
    for _ in 0..16 {
        m.step();
    }
    s.step(3, &mut raw, &mut m.ports[0]).unwrap();
    m.step();
    assert_eq!(
        s.step(4, &mut raw, &mut m.ports[0]),
        Ok(Progress::Ready { raw_epoch: 1 })
    );
    assert_eq!(m.policy.counts().handled, 1);
    assert_eq!(s.counts().sync_acked, 1);
}
fn physical_error_ack_requires_real_reset_and_monotone_inp1_ids_impl() {
    let mut m = Model::new();
    let mut s = source(&m);
    let mut raw = Raw::default();
    ready(&mut m, &mut s, &mut raw);
    raw.samples.push_back(Ok(Some(Sample {
        epoch: 1,
        sequence: 1,
        status: 0x81,
        byte: 0x1e,
    })));
    s.step(4, &mut raw, &mut m.ports[0]).unwrap();
    s.step(5, &mut raw, &mut m.ports[0]).unwrap();
    m.step();
    assert_eq!(
        s.step(6, &mut raw, &mut m.ports[0]),
        Ok(Progress::Recovering { reason: 13 })
    );
    assert!(!m.policy.core().is_online());
    s.step(7, &mut raw, &mut m.ports[0]).unwrap();
    s.step(8, &mut raw, &mut m.ports[0]).unwrap();
    s.step(9, &mut raw, &mut m.ports[0]).unwrap();
    m.step();
    assert_eq!(
        s.step(10, &mut raw, &mut m.ports[0]),
        Ok(Progress::Ready { raw_epoch: 2 })
    );
    assert_eq!(s.counts().sync_acked, 2);
    assert_eq!(m.policy.counts().handled, 2); // Error sample was not a successful handle.
    assert_eq!(m.policy.counts().rejected, 1);
}
fn raw_loss_and_native_clock_deadline_are_terminal_without_fake_sync_impl() {
    let mut m = Model::new();
    let mut s = source(&m);
    let mut raw = Raw::default();
    ready(&mut m, &mut s, &mut raw);
    raw.samples.push_back(Err(RawError::SequenceLost));
    let sends = m.ports[0].sends;
    assert_eq!(
        s.step(4, &mut raw, &mut m.ports[0]),
        Err(source::Fatal::Raw(RawError::SequenceLost))
    );
    assert!(s.step(5, &mut raw, &mut m.ports[0]).is_err());
    assert_eq!(m.ports[0].sends, sends);
    assert_eq!(raw.resets, 1);
    let mut s = source(&m);
    let mut raw = Raw::default();
    s.step(0, &mut raw, &mut m.ports[0]).unwrap();
    s.step(1, &mut raw, &mut m.ports[0]).unwrap();
    assert_eq!(
        s.step(202, &mut raw, &mut m.ports[0]),
        Err(source::Fatal::Deadline)
    );
}
fn pending_peek_survives_backpressure_and_focus_replaces_stale_reply_impl() {
    let mut m = Model::new();
    m.start();
    m.sample(2, 1, 0x1e);
    assert_eq!(m.policy.core().queued(), 1);
    let filler = Outgoing::new(1, &[]).unwrap();
    for _ in 0..16 {
        m.policy_io.send(send(m.reply_send[3]), &filler).unwrap();
    }
    m.ports[3]
        .send(send(m.requests[3]), &request(Operation::Peek, 1, 0))
        .unwrap();
    m.step();
    assert_eq!(m.policy.core().queued(), 1);
    assert_eq!(m.policy.counts().reply_again, 1);
    let response = m.call(1, request(Operation::Focus, 2, 202));
    assert_eq!(number(response.payload(), 24), 0);
    assert_eq!(m.policy.core().focus().unwrap().owner().identity(), 202);
    assert_eq!(m.policy.counts().stale, 1);
    for _ in 0..16 {
        m.ports[3].receive(recv(m.responses[3])).unwrap();
    }
    m.step();
    let negative = m.ports[3].receive(recv(m.responses[3])).unwrap();
    assert_eq!(negative.sender(), POLICY);
    assert_eq!(number(negative.payload(), 24), 8);
    assert_eq!(number(negative.payload(), 8), 1);
    let cancel = m.call(3, request(Operation::Peek, 2, 0));
    assert_eq!(cancel.payload()[40], 3);
}
fn pipelined_app_cannot_overwrite_pending_or_mutate_protected_routes_impl() {
    let mut m = Model::new();
    m.start();
    let filler = Outgoing::new(1, &[]).unwrap();
    for _ in 0..16 {
        m.policy_io.send(send(m.reply_send[3]), &filler).unwrap();
    }
    m.ports[3]
        .send(send(m.requests[3]), &request(Operation::Peek, 1, 0))
        .unwrap();
    m.step();
    let handled = m.policy.counts().handled;
    m.ports[3]
        .send(send(m.requests[3]), &request(Operation::Focus, 2, 202))
        .unwrap();
    m.call(2, request(Operation::SourceDead, 1, 0));
    assert_eq!(m.policy.counts().pipelined, 1);
    assert_eq!(m.policy.counts().handled, handled + 1);
    assert!(!m.policy.core().is_online());
    for _ in 0..16 {
        m.ports[3].receive(recv(m.responses[3])).unwrap();
    }
    m.step();
    let original = m.ports[3].receive(recv(m.responses[3])).unwrap();
    assert_eq!(number(original.payload(), 8), 1);
}
fn policy_state_is_bounded_and_checks_all_four_actual_recv_rights_impl() {
    let mut m = Model::new();
    m.step();
    assert!(m.policy.receive_rights_checked());
    assert!(std::mem::size_of::<Policy>() < 16384);
    assert!(std::mem::size_of::<Source>() < 1024);
}
fn actual_wrong_sender_on_source_ingress_cannot_initialize_input_impl() {
    let mut m = Model::new();
    let grant = m
        .engine
        .borrow_mut()
        .bootstrap_delegate(POLICY, m.ingress[0], 4, SEND)
        .unwrap();
    m.ports[3]
        .send(send(grant), &request(Operation::Sync, 1, 1))
        .unwrap();
    m.step();
    assert!(!m.policy.core().is_online());
    assert_eq!(m.policy.counts().handled, 0);
    assert_eq!(m.policy.counts().rejected, 1);
}
fn unexpected_actual_delegated_caps_are_closed_once_without_core_mutation_impl() {
    let mut m = Model::new();
    let mut outgoing = request(Operation::Peek, 1, 0);
    for _ in 0..2 {
        let raw = m.engine.borrow_mut().create_endpoint(4).unwrap();
        outgoing
            .push_cap(Capability {
                handle: Handle::from_raw(raw).unwrap(),
                rights: Rights::SEND,
            })
            .unwrap();
    }
    m.ports[3].send(send(m.requests[3]), &outgoing).unwrap();
    m.step();
    assert_eq!(m.policy_io.closed.len(), 2);
    assert_ne!(m.policy_io.closed[0], m.policy_io.closed[1]);
    assert_eq!(m.policy.counts().handled, 0);
    assert_eq!(m.policy.counts().rejected, 1);
    for &raw in &m.policy_io.closed {
        assert_eq!(
            m.engine.borrow_mut().close(POLICY, raw),
            Err(capability_core::Error::BadHandle)
        );
    }
    // Reusing ID1 is valid: rejected delegated caps never reached core.handle.
    let reply = m.call(3, request(Operation::Peek, 1, 0));
    assert_eq!(number(reply.payload(), 24), 1);
}
fn canonical_ack_from_wrong_kernel_sender_cannot_report_source_ready_impl() {
    let mut m = Model::new();
    let mut s = source(&m);
    let mut raw = Raw::default();
    s.step(0, &mut raw, &mut m.ports[0]).unwrap();
    s.step(1, &mut raw, &mut m.ports[0]).unwrap();
    s.step(2, &mut raw, &mut m.ports[0]).unwrap();
    let grant = m
        .engine
        .borrow_mut()
        .bootstrap_delegate(1, m.responses[0], 6, SEND)
        .unwrap();
    let mut body = Request {
        operation: Operation::Sync,
        id: 1,
        epoch: EPOCH,
        arg0: 0,
        arg1: 0,
    }
    .encode();
    body[6..8].copy_from_slice(&(wire::REPLY | Operation::Sync as u16).to_le_bytes());
    let outgoing = Outgoing::new(wire::TAG, &body).unwrap();
    m.engine
        .borrow_mut()
        .send(6, grant, &Message::decode(outgoing.as_bytes()).unwrap())
        .unwrap();
    assert_eq!(
        s.step(3, &mut raw, &mut m.ports[0]),
        Err(source::Fatal::Protocol)
    );
    assert_eq!(s.counts().sync_acked, 0);
    assert_eq!(raw.reads, 0);
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
    no_request_replay_while_real_reply_waits,
    no_request_replay_while_real_reply_waits_impl
);
scenario!(
    real_send_again_retains_same_source_request_without_reading_samples,
    real_send_again_retains_same_source_request_without_reading_samples_impl
);
scenario!(
    physical_error_ack_requires_real_reset_and_monotone_inp1_ids,
    physical_error_ack_requires_real_reset_and_monotone_inp1_ids_impl
);
scenario!(
    raw_loss_and_native_clock_deadline_are_terminal_without_fake_sync,
    raw_loss_and_native_clock_deadline_are_terminal_without_fake_sync_impl
);
scenario!(
    pending_peek_survives_backpressure_and_focus_replaces_stale_reply,
    pending_peek_survives_backpressure_and_focus_replaces_stale_reply_impl
);
scenario!(
    pipelined_app_cannot_overwrite_pending_or_mutate_protected_routes,
    pipelined_app_cannot_overwrite_pending_or_mutate_protected_routes_impl
);
scenario!(
    policy_state_is_bounded_and_checks_all_four_actual_recv_rights,
    policy_state_is_bounded_and_checks_all_four_actual_recv_rights_impl
);
scenario!(
    actual_wrong_sender_on_source_ingress_cannot_initialize_input,
    actual_wrong_sender_on_source_ingress_cannot_initialize_input_impl
);
scenario!(
    unexpected_actual_delegated_caps_are_closed_once_without_core_mutation,
    unexpected_actual_delegated_caps_are_closed_once_without_core_mutation_impl
);
scenario!(
    canonical_ack_from_wrong_kernel_sender_cannot_report_source_ready,
    canonical_ack_from_wrong_kernel_sender_cannot_report_source_ready_impl
);
scenario!(
    hid_completion_publication_requires_sync_and_matching_valid_transport_work,
    hid_completion_publication_requires_sync_and_matching_valid_transport_work_impl
);
scenario!(
    hid_real_ipc_backpressure_retains_the_original_request_and_deadline,
    hid_real_ipc_backpressure_retains_the_original_request_and_deadline_impl
);
scenario!(
    hid_detach_invalidates_pending_usb_immediately_and_rejects_reattach_replay,
    hid_detach_invalidates_pending_usb_immediately_and_rejects_reattach_replay_impl
);
scenario!(
    hid_wrong_actual_reply_sender_cannot_acknowledge_sync_or_enable_publication,
    hid_wrong_actual_reply_sender_cannot_acknowledge_sync_or_enable_publication_impl
);
scenario!(
    hid_mouse_session_publishes_exact_completed_bytes_through_policy,
    hid_mouse_session_publishes_exact_completed_bytes_through_policy_impl
);
scenario!(
    hid_replayed_ack_remote_error_and_clock_reversal_fail_closed,
    hid_replayed_ack_remote_error_and_clock_reversal_fail_closed_impl
);
scenario!(
    hid_unexpected_reply_capability_is_closed_before_terminal_rejection,
    hid_unexpected_reply_capability_is_closed_before_terminal_rejection_impl
);

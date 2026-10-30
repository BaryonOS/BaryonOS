//! Host capability/IPC model, not a kernel driver or native input execution.
use capability_core::{Cap, Engine, Message, SEND, TRANSFER};
use nieuw_os_input_core::{
    wire::{self, Operation, Request},
    Actor, ClientBinding, Configuration, Error, Reply, Route, Service, QUEUE_CAPACITY,
};
use nieuw_os_sdk_runtime::{Handle, ReceiveEndpoint, Received};

const SERVICE: u64 = 10;
const EPOCH: u64 = 7;

struct Model {
    engine: Box<Engine>,
    service: Service,
    roots: [u64; 4],
    source_send: u64,
    control_send: u64,
    lifecycle_send: u64,
    app_send: [u64; 3],
    reply_roots: [u64; 2],
    reply_send: [u64; 2],
    source_id: u64,
    control_id: u64,
    lifecycle_id: u64,
    app_ids: [u64; 2],
}

fn endpoint(raw: u64) -> ReceiveEndpoint {
    ReceiveEndpoint::from_handle(Handle::from_raw(raw).unwrap())
}
fn number(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}
fn request(operation: Operation, id: u64, arg0: u64, arg1: u64) -> Request {
    Request {
        operation,
        id,
        epoch: EPOCH,
        arg0,
        arg1,
    }
}

impl Model {
    fn new() -> Self {
        let mut engine = Box::new(Engine::new());
        for pid in [1, 2, 3, 4, 5, 6, SERVICE] {
            engine.register(pid).unwrap();
        }
        let roots = core::array::from_fn(|_| engine.create_endpoint(SERVICE).unwrap());
        let source_send = engine
            .bootstrap_delegate(SERVICE, roots[0], 1, SEND)
            .unwrap();
        let control_send = engine
            .bootstrap_delegate(SERVICE, roots[1], 2, SEND)
            .unwrap();
        let lifecycle_send = engine
            .bootstrap_delegate(SERVICE, roots[2], 3, SEND)
            .unwrap();
        let app_send = core::array::from_fn(|index| {
            engine
                .bootstrap_delegate(SERVICE, roots[3], 4 + index as u64, SEND)
                .unwrap()
        });
        let reply_roots =
            core::array::from_fn(|index| engine.create_endpoint(4 + index as u64).unwrap());
        let reply_send = core::array::from_fn(|index| {
            engine
                .bootstrap_delegate(4 + index as u64, reply_roots[index], SERVICE, SEND)
                .unwrap()
        });
        let configuration = Configuration {
            epoch: EPOCH,
            source: Actor {
                endpoint: endpoint(roots[0]),
                pid: 1,
            },
            compositor: Actor {
                endpoint: endpoint(roots[1]),
                pid: 2,
            },
            supervisor: Actor {
                endpoint: endpoint(roots[2]),
                pid: 3,
            },
            applications: endpoint(roots[3]),
            width: 800,
            height: 600,
        };
        let service = Service::new(
            configuration,
            &[
                ClientBinding { id: 101, pid: 4 },
                ClientBinding { id: 202, pid: 5 },
            ],
        )
        .unwrap();
        Self {
            engine,
            service,
            roots,
            source_send,
            control_send,
            lifecycle_send,
            app_send,
            reply_roots,
            reply_send,
            source_id: 0,
            control_id: 0,
            lifecycle_id: 0,
            app_ids: [0; 2],
        }
    }
    fn envelope(&mut self, pid: u64, send: u64, root: u64, payload: &[u8]) -> Received {
        let mut message = Message::new();
        message.tag = wire::TAG;
        message.len = payload.len() as u32;
        message.data[..payload.len()].copy_from_slice(payload);
        self.engine.send(pid, send, &message).unwrap();
        let delivered = self.engine.receive(SERVICE, root).unwrap();
        assert_eq!(delivered.sender_pid, pid);
        Received::from_bytes(delivered.encode(), u64::from(delivered.len)).unwrap()
    }
    fn call(&mut self, role: usize, pid: u64, send: u64, request: Request) -> Result<Reply, Error> {
        let received = self.envelope(pid, send, self.roots[role], &request.encode());
        self.service.handle(endpoint(self.roots[role]), &received)
    }
    fn sync(&mut self) -> Result<Reply, Error> {
        self.source_id += 1;
        self.call(
            0,
            1,
            self.source_send,
            request(Operation::Sync, self.source_id, 1, 0),
        )
    }
    fn sample(&mut self, status: u8, byte: u8) -> Result<Reply, Error> {
        self.source_id += 1;
        self.call(
            0,
            1,
            self.source_send,
            request(
                Operation::Sample,
                self.source_id,
                u64::from(status),
                u64::from(byte),
            ),
        )
    }
    fn key(&mut self, byte: u8) {
        self.sample(1, byte).unwrap();
    }
    fn mouse(&mut self, bytes: [u8; 3]) {
        for byte in bytes {
            self.sample(0x21, byte).unwrap();
        }
    }
    fn focus(&mut self, id: u64) -> Result<Reply, Error> {
        self.control_id += 1;
        self.call(
            1,
            2,
            self.control_send,
            request(Operation::Focus, self.control_id, id, 0),
        )
    }
    fn death(&mut self, operation: Operation, id: u64) -> Result<Reply, Error> {
        self.lifecycle_id += 1;
        self.call(
            2,
            3,
            self.lifecycle_send,
            request(operation, self.lifecycle_id, id, 0),
        )
    }
    fn app(&mut self, index: usize, operation: Operation, value: u64) -> Result<Reply, Error> {
        self.app_ids[index] += 1;
        self.call(
            3,
            4 + index as u64,
            self.app_send[index],
            request(operation, self.app_ids[index], value, 0),
        )
    }
    fn peek(&mut self, index: usize) -> Reply {
        self.app(index, Operation::Peek, 0).unwrap()
    }
    fn ack(&mut self, index: usize, reply: &Reply) {
        self.app(index, Operation::Ack, number(reply.payload(), 48))
            .unwrap();
    }
    fn start(&mut self) {
        self.sync().unwrap();
        self.focus(101).unwrap();
    }
}

#[test]
fn fixed_wire_is_canonical_and_every_short_or_extra_input_is_rejected() {
    let expected = request(Operation::Sync, 9, 1, 0);
    let mut bytes = [0u8; 96];
    bytes[..8].copy_from_slice(&[b'I', b'N', b'P', b'1', 1, 0, 1, 0]);
    bytes[8] = 9;
    bytes[16] = 7;
    bytes[24] = 1;
    assert_eq!(expected.encode(), bytes);
    assert_eq!(Request::decode(&bytes, EPOCH), Ok(expected));
    for length in 0..wire::SIZE {
        assert_eq!(
            Request::decode(&bytes[..length], EPOCH),
            Err(Error::Malformed)
        );
    }
    let mut long = bytes.to_vec();
    long.push(0);
    assert_eq!(Request::decode(&long, EPOCH), Err(Error::Malformed));
    for offset in [0, 4, 6, 40, 71, 95] {
        let mut malformed = bytes;
        malformed[offset] ^= 0x40;
        assert_eq!(Request::decode(&malformed, EPOCH), Err(Error::Malformed));
    }
    assert_eq!(Request::decode(&bytes, EPOCH + 1), Err(Error::WrongEpoch));
    for bad in [
        request(Operation::Sync, 1, 0, 0),
        request(Operation::Sample, 1, 256, 0),
        request(Operation::Peek, 1, 101, 0),
        request(Operation::Ack, 1, 0, 0),
    ] {
        assert_eq!(Request::decode(&bad.encode(), EPOCH), Err(Error::Malformed));
    }
}

#[test]
fn authority_engine_sender_and_ingress_cannot_be_replaced_by_application_payload() {
    let mut m = Model::new();
    m.start();
    for operation in [
        Operation::Sync,
        Operation::Sample,
        Operation::Focus,
        Operation::ClientDead,
    ] {
        let (a, b) = match operation {
            Operation::Sync => (1, 0),
            Operation::Sample => (1, 0x1e),
            _ => (101, 0),
        };
        let result = m.call(3, 4, m.app_send[0], request(operation, 1, a, b));
        assert_eq!(result, Err(Error::WrongRole));
    }
    // Even a mistakenly delegated protected endpoint still checks kernel PID.
    let misdelegated = m
        .engine
        .bootstrap_delegate(SERVICE, m.roots[0], 4, SEND)
        .unwrap();
    assert_eq!(
        m.call(0, 4, misdelegated, request(Operation::Sample, 2, 1, 0x1e)),
        Err(Error::Access)
    );
    // A third process sharing application SEND cannot claim either binding.
    assert_eq!(
        m.call(3, 6, m.app_send[2], request(Operation::Peek, 1, 0, 0)),
        Err(Error::Access)
    );
    let mut forged = Message::new();
    forged.sender_pid = 1;
    assert!(m.engine.send(4, m.app_send[0], &forged).is_err());
    assert_eq!(m.service.queued(), 0);
    m.key(0x1e);
    let reply = m.peek(0);
    assert_eq!(reply.route(), Route::Client(101));
    assert_eq!(&reply.payload()[40..46], &[1, 1, 29, 0, 0, 0]); // Physical A press.
    assert_eq!(number(reply.payload(), 32), 101);
}

#[test]
fn negative_reply_stays_on_authenticated_route_with_no_success_or_event_payload() {
    let mut m = Model::new();
    let received = m.envelope(
        4,
        m.app_send[0],
        m.roots[3],
        &request(Operation::Focus, 1, 202, 0).encode(),
    );
    assert_eq!(
        m.service.handle(endpoint(m.roots[3]), &received),
        Err(Error::WrongRole)
    );
    let rejected = m
        .service
        .error_reply(endpoint(m.roots[3]), &received, Error::WrongRole)
        .unwrap();
    assert_eq!(rejected.route(), Route::Client(101));
    assert_eq!(number(rejected.payload(), 24), 7);
    assert_eq!(number(rejected.payload(), 32), 101);
    assert!(rejected.payload()[40..].iter().all(|byte| *byte == 0));
    let intruder = m.envelope(
        6,
        m.app_send[2],
        m.roots[3],
        &request(Operation::Peek, 1, 0, 0).encode(),
    );
    assert_eq!(
        m.service
            .error_reply(endpoint(m.roots[3]), &intruder, Error::Access),
        Err(Error::Access)
    );
}

#[test]
fn focus_change_invalidates_prepared_reply_and_suppresses_old_held_key() {
    let mut m = Model::new();
    m.start();
    m.key(0x1e);
    let old = m.peek(0);
    assert!(m.service.reply_is_current(&old));
    m.focus(202).unwrap();
    assert!(!m.service.reply_is_current(&old));
    let cancel = m.peek(0);
    assert_eq!(&cancel.payload()[40..42], &[3, 1]);
    assert_eq!(number(cancel.payload(), 56), number(old.payload(), 56));
    m.key(0x1e); // physical repeat remains owned by the old focus
    assert_eq!(number(m.peek(1).payload(), 24), 1);
    m.key(0x9e);
    m.key(0x1e);
    let fresh = m.peek(1);
    assert_eq!(fresh.payload()[40], 1);
    assert!(number(fresh.payload(), 56) > number(old.payload(), 56));
    m.ack(0, &cancel); // acknowledging A's cancel cannot remove B's key
    assert_eq!(m.service.queued(), 1);
}

#[test]
fn offered_input_is_not_consumed_by_real_ipc_queue_backpressure() {
    let mut m = Model::new();
    m.start();
    m.key(0x1e);
    let reply = m.peek(0);
    for _ in 0..16 {
        m.engine
            .send(SERVICE, m.reply_send[0], &Message::new())
            .unwrap();
    }
    let mut message = Message::new();
    message.tag = wire::TAG;
    message.len = wire::SIZE as u32;
    message.data[..wire::SIZE].copy_from_slice(reply.payload());
    assert_eq!(
        m.engine.send(SERVICE, m.reply_send[0], &message),
        Err(capability_core::Error::Again)
    );
    assert_eq!(m.service.queued(), 1);
    let again = m.peek(0);
    assert_eq!(number(again.payload(), 48), number(reply.payload(), 48));
    for _ in 0..16 {
        m.engine.receive(4, m.reply_roots[0]).unwrap();
    }
    assert!(m.service.reply_is_current(&again));
    m.engine.send(SERVICE, m.reply_send[0], &message).unwrap();
    let delivered = m.engine.receive(4, m.reply_roots[0]).unwrap();
    assert_eq!(delivered.sender_pid, SERVICE);
    assert_eq!(&delivered.data[..wire::SIZE], reply.payload());
    m.ack(0, &again);
    assert_eq!(m.service.queued(), 0);
}

#[test]
fn application_counter_exhaustion_stops_only_its_own_requests() {
    let mut m = Model::new();
    m.start();
    m.key(0x1e);
    m.call(
        3,
        4,
        m.app_send[0],
        request(Operation::Peek, u64::MAX, 0, 0),
    )
    .unwrap();
    assert_eq!(m.app(0, Operation::Peek, 0), Err(Error::StaleRequest));
    assert!(!m.service.is_halted());
    assert!(m.service.is_online());
    m.focus(202).unwrap();
    m.key(0x9e);
    m.key(0x1e);
    let other = m.peek(1);
    assert_eq!(other.route(), Route::Client(202));
    assert_eq!(other.payload()[40], 1);
}

#[test]
fn empty_is_an_observation_and_does_not_hide_a_later_cancellation() {
    let mut m = Model::new();
    m.start();
    let empty = m.peek(0);
    assert_eq!(number(empty.payload(), 24), 1);
    m.key(0x1e);
    m.focus(202).unwrap();
    // Empty says nothing about present focus or authorization. Only actual
    // event/cancel replies carry a generation/sequence requiring invalidation.
    assert!(m.service.reply_is_current(&empty));
    let cancel = m.peek(0);
    assert_eq!(cancel.payload()[40], 3);
    assert!(m.service.reply_is_current(&cancel));
}

#[test]
fn stale_ack_cannot_remove_a_new_cancel_or_event() {
    let mut m = Model::new();
    m.start();
    m.key(0x1e);
    let old = m.peek(0);
    m.focus(202).unwrap();
    assert_eq!(
        m.app(0, Operation::Ack, number(old.payload(), 48)),
        Err(Error::StaleAck)
    );
    let cancel = m.peek(0);
    m.ack(0, &cancel);
    assert_eq!(
        m.app(0, Operation::Ack, number(cancel.payload(), 48)),
        Err(Error::StaleAck)
    );
    m.key(0x9e);
    m.key(0x1e);
    let b = m.peek(1);
    assert_eq!(
        m.app(0, Operation::Ack, number(b.payload(), 48)),
        Err(Error::StaleAck)
    );
    assert_eq!(m.service.queued(), 1);
}

#[test]
fn queue_overflow_has_priority_cancellation_and_no_stuck_repeat() {
    let mut m = Model::new();
    m.start();
    for _ in 0..QUEUE_CAPACITY {
        m.key(0x1e);
    }
    assert_eq!(m.sample(1, 0x1e), Err(Error::QueueFull));
    assert_eq!(m.service.queued(), 0);
    assert!(m.service.is_online());
    let cancel = m.peek(0);
    assert_eq!(&cancel.payload()[40..42], &[3, 3]);
    m.key(0x1e);
    assert_eq!(m.service.queued(), 0);
    m.ack(0, &cancel);
    m.key(0x9e);
    m.key(0x1e);
    assert_eq!(&m.peek(0).payload()[40..42], &[1, 1]);
}

#[test]
fn source_gap_and_controller_fault_require_explicit_authenticated_resync() {
    let mut m = Model::new();
    m.start();
    m.key(0x2a);
    m.source_id += 1; // one byte was lost before the next received sample
    assert_eq!(m.sample(1, 0x1e), Err(Error::StreamLost));
    assert!(!m.service.is_online());
    assert_eq!(m.sample(1, 0x1e), Err(Error::NotSynchronized));
    let before = m.peek(0);
    assert_eq!(&before.payload()[40..42], &[3, 2]);
    m.sync().unwrap();
    assert!(!m.service.reply_is_current(&before));
    let cancel = m.peek(0);
    m.ack(0, &cancel);
    m.key(0x2a); // reset decoder sees make; known old held Shift stays suppressed
    m.key(0x1e);
    let typed = m.peek(0);
    assert_eq!(&typed.payload()[44..46], &[0, 0]);
    assert_eq!(m.sample(0xc1, 0x1e), Err(Error::ControllerFault));
    assert!(!m.service.is_online());
}

#[test]
fn held_without_focus_survives_resync_and_real_break_rearms_immediately() {
    for repeat in [false, true] {
        let mut m = Model::new();
        m.sync().unwrap();
        m.key(0x1d);
        m.focus(101).unwrap();
        m.sync().unwrap();
        let cancel = m.peek(0);
        m.ack(0, &cancel);
        if repeat {
            m.key(0x1d);
        }
        m.key(0x1e);
        let typed = m.peek(0);
        assert_eq!(&typed.payload()[44..46], &[0, 0]);
        m.ack(0, &typed);
        m.key(0x9d);
        m.key(0x1d);
        let control = m.peek(0);
        assert_eq!(control.payload()[42], 28);
        assert_eq!(&control.payload()[44..46], &[4, 0]);
    }
}

#[test]
fn ps2_mouse_bytes_translate_real_deltas_buttons_and_bounded_coordinates() {
    let mut m = Model::new();
    m.start();
    m.mouse([0x29, 5, 254]); // right 5, down 2, left button pressed
    let mouse = m.peek(0);
    assert_eq!(&mouse.payload()[40..44], &[2, 0, 1, 1]);
    assert_eq!(
        i16::from_le_bytes(mouse.payload()[44..46].try_into().unwrap()),
        5
    );
    assert_eq!(
        i16::from_le_bytes(mouse.payload()[46..48].try_into().unwrap()),
        -2
    );
    assert_eq!(m.service.cursor(), (5, 2));
    m.ack(0, &mouse);
    m.focus(202).unwrap();
    m.mouse([0x09, 3, 0]); // left stays physically held across focus
    let next = m.peek(1);
    assert_eq!(&next.payload()[42..44], &[0, 0]);
    assert_eq!(m.service.cursor(), (8, 2));
}

#[test]
fn client_death_removes_focus_and_cannot_be_reversed_by_replayed_requests() {
    let mut m = Model::new();
    m.start();
    m.key(0x1e);
    let old = m.peek(0);
    m.death(Operation::ClientDead, 101).unwrap();
    assert_eq!(m.service.focus(), None);
    assert!(!m.service.reply_is_current(&old));
    assert_eq!(m.app(0, Operation::Peek, 0), Err(Error::ActorDead));
    assert_eq!(m.focus(101), Err(Error::ActorDead));
    m.focus(202).unwrap();
    m.key(0x1e);
    assert_eq!(number(m.peek(1).payload(), 24), 1);
    m.key(0x9e);
    m.key(0x1e);
    assert_eq!(m.peek(1).payload()[40], 1);
}

#[test]
fn source_and_compositor_death_are_permanent_for_the_bootstrap_lifetime() {
    for operation in [Operation::SourceDead, Operation::CompositorDead] {
        let mut m = Model::new();
        m.start();
        m.key(0x1e);
        let old = m.peek(0);
        m.death(operation, 0).unwrap();
        assert!(!m.service.reply_is_current(&old));
        assert_eq!(m.service.focus(), None);
        let cancel = m.peek(0);
        assert_eq!(cancel.payload()[40], 3);
        if operation == Operation::SourceDead {
            assert_eq!(m.sync(), Err(Error::ActorDead));
        } else {
            assert_eq!(m.focus(101), Err(Error::ActorDead));
        }
    }
}

#[test]
fn replayed_focus_or_lifecycle_and_old_epoch_cannot_modify_current_authority() {
    let mut m = Model::new();
    m.start();
    m.focus(202).unwrap();
    let current = m.service.focus();
    assert_eq!(
        m.call(1, 2, m.control_send, request(Operation::Focus, 1, 101, 0)),
        Err(Error::StaleRequest)
    );
    assert_eq!(m.service.focus(), current);
    let wrong_epoch = Request {
        epoch: EPOCH - 1,
        ..request(Operation::Sample, m.source_id + 1, 1, 0x1e)
    };
    assert_eq!(
        m.call(0, 1, m.source_send, wrong_epoch),
        Err(Error::WrongEpoch)
    );
    m.key(0x1e);
    assert_eq!(m.service.queued(), 1);
    m.death(Operation::ClientDead, 101).unwrap();
    assert_eq!(
        m.call(
            2,
            3,
            m.lifecycle_send,
            request(Operation::ClientDead, 1, 202, 0)
        ),
        Err(Error::StaleRequest)
    );
    assert_eq!(m.service.focus(), current);
    let reply = m.peek(1);
    assert!(m.service.reply_is_current(&reply));
}

#[test]
fn malformed_mouse_packet_stops_stream_and_resync_does_not_inherit_modifier() {
    let mut m = Model::new();
    m.start();
    m.key(0x2a);
    assert_eq!(m.sample(0x21, 0), Err(Error::DecodeFault));
    assert!(!m.service.is_online());
    m.sync().unwrap();
    let cancel = m.peek(0);
    m.ack(0, &cancel);
    m.key(0x1e);
    let typed = m.peek(0);
    assert_eq!(&typed.payload()[44..46], &[0, 0]);
}

#[test]
fn unexpected_delegated_caps_are_rejected_and_adapter_can_close_them_without_leak() {
    let mut m = Model::new();
    let transferable = m
        .engine
        .derive(4, m.reply_roots[0], SEND | TRANSFER)
        .unwrap();
    let before = m.engine.stats().handles;
    let mut message = Message::new();
    message.tag = wire::TAG;
    message.len = wire::SIZE as u32;
    message.data[..wire::SIZE].copy_from_slice(&request(Operation::Peek, 1, 0, 0).encode());
    message.cap_count = 1;
    message.caps[0] = Cap {
        handle: transferable,
        rights: SEND,
    };
    m.engine.send(4, m.app_send[0], &message).unwrap();
    let delivered = m.engine.receive(SERVICE, m.roots[3]).unwrap();
    let received = Received::from_bytes(delivered.encode(), u64::from(delivered.len)).unwrap();
    assert_eq!(
        m.service.handle(endpoint(m.roots[3]), &received),
        Err(Error::UnexpectedCaps)
    );
    for index in 0..received.cap_count() {
        m.engine
            .close(SERVICE, received.cap(index).unwrap().handle.raw())
            .unwrap();
    }
    assert_eq!(m.engine.stats().handles, before);
    assert!(m.engine.check_invariants());
}

#[test]
fn malformed_or_stale_samples_do_not_inject_extra_input_and_memory_is_bounded() {
    let mut m = Model::new();
    m.start();
    m.key(0x1e);
    let same = request(Operation::Sample, m.source_id, 1, 0x1e);
    assert_eq!(m.call(0, 1, m.source_send, same), Err(Error::StaleRequest));
    assert_eq!(m.service.queued(), 1);
    let mut bad = same.encode();
    bad[95] = 1;
    let received = m.envelope(1, m.source_send, m.roots[0], &bad);
    assert_eq!(
        m.service.handle(endpoint(m.roots[0]), &received),
        Err(Error::Malformed)
    );
    assert_eq!(m.service.queued(), 1);
    println!("bounded Service bytes={}", core::mem::size_of::<Service>());
    assert!(core::mem::size_of::<Service>() <= 8192);
    assert_eq!(
        m.call(
            0,
            1,
            m.source_send,
            request(Operation::Sample, u64::MAX, 1, 0x1e)
        ),
        Err(Error::CounterExhausted)
    );
    assert!(m.service.is_halted());
    assert_eq!(m.service.focus(), None);
}

#[test]
fn configuration_rejects_aliased_roles_duplicate_clients_and_zero_geometry() {
    let m = Model::new();
    let endpoint0 = endpoint(m.roots[0]);
    let good = Configuration {
        epoch: EPOCH,
        source: Actor {
            endpoint: endpoint0,
            pid: 1,
        },
        compositor: Actor {
            endpoint: endpoint(m.roots[1]),
            pid: 2,
        },
        supervisor: Actor {
            endpoint: endpoint(m.roots[2]),
            pid: 3,
        },
        applications: endpoint(m.roots[3]),
        width: 800,
        height: 600,
    };
    for c in [
        Configuration { epoch: 0, ..good },
        Configuration { width: 0, ..good },
        Configuration {
            applications: endpoint0,
            ..good
        },
    ] {
        assert!(matches!(
            Service::new(c, &[ClientBinding { id: 101, pid: 4 }]),
            Err(Error::Configuration)
        ));
    }
    assert!(matches!(
        Service::new(
            good,
            &[
                ClientBinding { id: 101, pid: 4 },
                ClientBinding { id: 101, pid: 5 }
            ]
        ),
        Err(Error::Configuration)
    ));
    assert!(matches!(
        Service::new(
            good,
            &[
                ClientBinding { id: 101, pid: 4 },
                ClientBinding { id: 202, pid: 4 }
            ]
        ),
        Err(Error::Configuration)
    ));
}

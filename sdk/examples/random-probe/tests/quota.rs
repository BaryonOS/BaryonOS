//! Deterministic host scheduling/queue oracles, not native timing or entropy evidence.
use capability_core::{Engine, Error, Message, QUEUE_CAPACITY, SEND};
use keyring_core::{random, wire as service_wire, EntropySource, Service, Status};
use nieuw_os_native_random_probe::{contract, wire};
use nieuw_os_sdk_runtime::Received;

struct TestSource(u8);
impl EntropySource for TestSource {
    fn read(&mut self, out: &mut [u8]) -> Result<(), Status> {
        for byte in out {
            self.0 = self.0.wrapping_add(1);
            *byte = self.0;
        }
        Ok(())
    }
}
fn ready() -> (Service, TestSource) {
    let mut source = TestSource(0);
    let mut service = Service::new();
    service.start(&mut source).unwrap();
    (service, source)
}
fn message(data: [u8; wire::SIZE]) -> Message {
    Message {
        tag: wire::TAG,
        len: wire::SIZE as u32,
        data,
        ..Message::new()
    }
}
fn reply(service: &mut Service, source: &mut TestSource, id: u64, now: u64) -> Message {
    let request = wire::request(wire::RANDOM, id, 1);
    let header = service_wire::Header::decode(&request).unwrap();
    let mut bytes = [0; random::MAX_BYTES];
    let data = match service.random_at(0, id, 1, now, source, &mut bytes) {
        Ok(1) => {
            let mut body = [0; 17];
            body[..16].copy_from_slice(&request[16..32]);
            body[16] = bytes[0];
            service_wire::reply(&header, Status::Ok, &body)
        }
        Ok(_) => panic!("unexpected RANDOM length"),
        Err(status) => service_wire::reply(&header, status, &[]),
    };
    message(data)
}
fn parsed(message: Message, id: u64, count: usize) -> Option<wire::Reply> {
    let received = Received::from_bytes(message.encode(), wire::SIZE as u64).unwrap();
    wire::reply(&received, 2, &wire::request(wire::RANDOM, id, 1), count)
}

#[test]
fn four_bounded_batches_preserve_fifo_counters_and_real_quota_denial() {
    assert_eq!(contract::WINDOW_TICKS, random::WINDOW_TICKS);
    assert_eq!(random::WINDOW_CALLS, 64);
    const { assert!(contract::QUOTA_BATCH <= QUEUE_CAPACITY) };
    assert_eq!(64 % contract::QUOTA_BATCH, 0);
    let mut engine = Engine::new();
    for pid in [2, 5] {
        engine.register(pid).unwrap();
    }
    let ingress = engine.create_endpoint(2).unwrap();
    let send = engine.bootstrap_delegate(2, ingress, 5, SEND).unwrap();
    let receive = engine.create_endpoint(5).unwrap();
    let respond = engine.bootstrap_delegate(5, receive, 2, SEND).unwrap();
    let (mut service, mut source) = ready();
    let mut now = 131;
    for first in (1000..1064).step_by(contract::QUOTA_BATCH) {
        // Worst queue interleaving: the entire request batch is enqueued
        // before service execution, and all replies precede client reads.
        for id in first..first + contract::QUOTA_BATCH as u64 {
            engine
                .send(5, send, &message(wire::request(wire::RANDOM, id, 1)))
                .unwrap();
        }
        for id in first..first + contract::QUOTA_BATCH as u64 {
            let request = engine.receive(2, ingress).unwrap();
            assert_eq!(request.sender_pid, 5);
            assert_eq!(request.data, wire::request(wire::RANDOM, id, 1));
            let response = reply(&mut service, &mut source, id, now);
            engine.send(2, respond, &response).unwrap();
            now += 1;
        }
        for id in first..first + contract::QUOTA_BATCH as u64 {
            let response = parsed(engine.receive(5, receive).unwrap(), id, 1).unwrap();
            assert_eq!((response.status, response.length), (0, 1));
        }
        assert_eq!(engine.receive(2, ingress), Err(Error::Again));
        assert_eq!(engine.receive(5, receive), Err(Error::Again));
        assert!(engine.check_invariants());
    }
    let counters = service.random_counters();
    assert_eq!(
        (counters.calls, counters.bytes, counters.seeds),
        (64, 64, 1)
    );
    let response = reply(&mut service, &mut source, 1064, now);
    engine.send(2, respond, &response).unwrap();
    assert_eq!(
        parsed(engine.receive(5, receive).unwrap(), 1064, 0)
            .unwrap()
            .status,
        Status::Quota as i32
    );
    assert_eq!(service.random_counters(), counters);
    assert!(engine.check_invariants());
}

#[test]
fn ci_114_tick_roundtrip_burst_must_not_be_misreported_as_quota_denial() {
    let (mut service, mut source) = ready();
    for index in 0..64 {
        let response = reply(
            &mut service,
            &mut source,
            1000 + index,
            131 + index * 114 / 63,
        );
        assert_eq!(
            service_wire::Header::decode(&response.data).unwrap().status,
            0
        );
    }
    let mut response = reply(&mut service, &mut source, 1064, 247);
    response.sender_pid = 2;
    assert_eq!(
        service_wire::Header::decode(&response.data).unwrap().status,
        0
    );
    assert!(parsed(response, 1064, 0).is_none());
    let accepted = parsed(response, 1064, 1).unwrap();
    assert_eq!((accepted.status, accepted.length), (0, 1));
}

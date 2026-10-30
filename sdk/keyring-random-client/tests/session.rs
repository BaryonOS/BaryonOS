use keyring_core::{wire, EntropySource, Service, Status};
use nieuw_os_keyring_random_client::{
    Delivery, Failure, Outcome, ServiceStatus, Session, Transport,
};
use nieuw_os_sdk_runtime::{
    Capability, Error, Handle, Outgoing, ReceiveEndpoint, Received, Rights, SendEndpoint,
};
use std::{cell::RefCell, rc::Rc};

fn handle(slot: u64) -> Handle {
    Handle::from_raw((1 << 32) | slot).unwrap()
}
struct Source(u64);
impl EntropySource for Source {
    fn read(&mut self, output: &mut [u8]) -> Result<(), Status> {
        for b in output {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            *b = self.0 as u8;
        }
        Ok(())
    }
}
struct Shared {
    service: Service,
    source: Source,
    now: u64,
    sent: usize,
    receives: usize,
    yields: usize,
    pending: Option<[u8; 256]>,
    reply: Option<[u8; 320]>,
    closed: Vec<Handle>,
    send_again: bool,
    receive_again: bool,
    send_error: Option<Error>,
    receive_error: Option<Error>,
    yield_error: bool,
    close_error: Option<Handle>,
    service_status: Option<Status>,
}
#[derive(Clone)]
struct Mock(Rc<RefCell<Shared>>);
impl Transport for Mock {
    fn send(&mut self, _: SendEndpoint, message: &Outgoing) -> Result<(), Error> {
        let mut s = self.0.borrow_mut();
        s.sent += 1;
        if s.send_again {
            return Err(Error::Kernel(-8));
        }
        if let Some(error) = s.send_error {
            return Err(error);
        }
        let bytes: [u8; 256] = message.as_bytes()[64..].try_into().unwrap();
        let header = wire::Header::decode(&bytes).unwrap();
        // This is the real service wire parser, not a duplicate fake response.
        let length = wire::random_length(&header, &bytes[16..32]).unwrap();
        assert!((1..=128).contains(&length));
        s.pending = Some(bytes);
        Ok(())
    }
    fn receive(&mut self, _: ReceiveEndpoint) -> Result<Received, Error> {
        let mut s = self.0.borrow_mut();
        s.receives += 1;
        if s.receive_again {
            return Err(Error::Kernel(-8));
        }
        if let Some(error) = s.receive_error {
            return Err(error);
        }
        if let Some(reply) = s.reply.take() {
            return Received::from_bytes(
                reply,
                u32::from_le_bytes(reply[16..20].try_into().unwrap()) as u64,
            );
        }
        let request = s.pending.take().unwrap();
        let header = wire::Header::decode(&request).unwrap();
        let now = s.now;
        s.now += 100;
        let reply = if let Some(status) = s.service_status {
            wire::reply(&header, status, &[])
        } else {
            let Shared {
                service, source, ..
            } = &mut *s;
            wire::dispatch_random(service, source, 0, now, &header, &request[16..32])
        };
        let mut message = *Outgoing::new(wire::TAG, &reply).unwrap().as_bytes();
        message[..8].copy_from_slice(&7u64.to_le_bytes());
        Received::from_bytes(message, 256)
    }
    fn yield_now(&mut self) -> Result<(), Error> {
        let mut s = self.0.borrow_mut();
        s.yields += 1;
        if s.yield_error {
            Err(Error::Kernel(-9))
        } else {
            Ok(())
        }
    }
    fn close(&mut self, handle: Handle) -> Result<(), Error> {
        let mut s = self.0.borrow_mut();
        s.closed.push(handle);
        if s.close_error == Some(handle) {
            Err(Error::Kernel(-3))
        } else {
            Ok(())
        }
    }
}
fn setup(budget: usize) -> (Session<Mock>, Rc<RefCell<Shared>>) {
    let mut service = Service::new();
    let mut source = Source(0x31415926);
    service.start(&mut source).unwrap();
    let shared = Rc::new(RefCell::new(Shared {
        service,
        source,
        now: 0,
        sent: 0,
        receives: 0,
        yields: 0,
        pending: None,
        reply: None,
        closed: vec![],
        send_again: false,
        receive_again: false,
        send_error: None,
        receive_error: None,
        yield_error: false,
        close_error: None,
        service_status: None,
    }));
    let session = Session::new(
        Mock(shared.clone()),
        SendEndpoint::from_handle(handle(1)),
        ReceiveEndpoint::from_handle(handle(2)),
        7,
        budget,
    )
    .unwrap();
    (session, shared)
}
fn fixture() -> [u8; 320] {
    let request = wire::random_request(1, 32).unwrap();
    let header = wire::Header::decode(&request).unwrap();
    let mut body = [0u8; 48];
    body[..16].copy_from_slice(&request[16..32]);
    body[16..].fill(0x53);
    let reply = wire::reply(&header, Status::Ok, &body);
    let mut bytes = *Outgoing::new(wire::TAG, &reply).unwrap().as_bytes();
    bytes[..8].copy_from_slice(&7u64.to_le_bytes());
    bytes
}

#[test]
fn every_client_length_roundtrips_through_real_core_and_never_resends_after_queue() {
    let (mut session, shared) = setup(4);
    for length in 1..=128 {
        session.begin(length).unwrap();
        assert!(session.poll().unwrap().is_none());
        let Some(Outcome::Bytes(bytes)) = session.poll().unwrap() else {
            panic!("no output")
        };
        assert_eq!(bytes.as_slice().len(), length);
        assert_eq!(format!("{bytes:?}"), "RandomBytes(<redacted>)");
    }
    let s = shared.borrow();
    assert_eq!(s.sent, 128);
    assert_eq!(s.receives, 128);
    assert_eq!(s.service.random_counters().calls, 128);
    assert_eq!(s.service.random_counters().bytes, 8256);
    drop(s);
    drop(session);
    assert_eq!(shared.borrow().closed, vec![handle(1), handle(2)]);
}
#[test]
fn invalid_length_busy_idle_closed_do_not_publish() {
    let (mut session, shared) = setup(4);
    assert_eq!(session.poll().unwrap_err().failure, Failure::Idle);
    for length in [0, 129, 224, usize::MAX] {
        assert_eq!(
            session.begin(length).unwrap_err().failure,
            Failure::InvalidLength
        );
    }
    session.begin(32).unwrap();
    assert_eq!(session.begin(64).unwrap_err().failure, Failure::Busy);
    assert_eq!(shared.borrow().sent, 0);
    session.cancel().unwrap();
    assert_eq!(session.poll().unwrap_err().failure, Failure::Closed);
    assert_eq!(session.begin(32).unwrap_err().failure, Failure::Closed);
    session.close().unwrap();
    assert_eq!(shared.borrow().closed.len(), 2);
}
#[test]
fn canonical_not_ready_and_quota_return_no_bytes_allow_explicit_new_requests() {
    let (mut session, shared) = setup(4);
    for (status, expected) in [
        (Status::NotReady, ServiceStatus::NotReady),
        (Status::Quota, ServiceStatus::Quota),
    ] {
        shared.borrow_mut().service_status = Some(status);
        session.begin(32).unwrap();
        session.poll().unwrap();
        assert!(
            matches!(session.poll().unwrap(),Some(Outcome::Service(actual)) if actual==expected)
        );
        assert!(!session.is_closed());
    }
    assert_eq!(shared.borrow().sent, 2);
    shared.borrow_mut().service_status = None;
    session.begin(32).unwrap();
    session.poll().unwrap();
    assert!(matches!(session.poll().unwrap(), Some(Outcome::Bytes(_))));
}
#[test]
fn degraded_and_unknown_status_close_lease_without_fallback() {
    for status in [
        Status::Degraded,
        Status::Invalid,
        Status::Entropy,
        Status::Auth,
        Status::BadHandle,
        Status::Revoked,
    ] {
        let (mut session, shared) = setup(4);
        shared.borrow_mut().service_status = Some(status);
        session.begin(32).unwrap();
        session.poll().unwrap();
        let result = session.poll();
        if status == Status::Degraded {
            assert!(matches!(
                result,
                Ok(Some(Outcome::Service(ServiceStatus::Degraded)))
            ));
        } else {
            assert_eq!(result.unwrap_err().failure, Failure::Protocol);
        }
        assert!(session.is_closed());
        assert_eq!(shared.borrow().sent, 1);
        assert_eq!(shared.borrow().closed.len(), 2);
    }
}
#[test]
fn malformed_responses_terminally_reject_sender_tag_op_index_id_status_prefix_count_and_padding() {
    // All modifications preserve the outer Received format, reaching client validation.
    let offsets = [0, 8, 64, 65, 66, 68, 76, 80, 84, 86, 88, 144];
    for offset in offsets {
        let (mut session, shared) = setup(4);
        let mut reply = fixture();
        reply[offset] ^= 1;
        shared.borrow_mut().reply = Some(reply);
        session.begin(32).unwrap();
        session.poll().unwrap();
        let error = session.poll().unwrap_err();
        assert_eq!(error.failure, Failure::Protocol, "offset {offset}");
        assert_eq!(error.delivery, Delivery::Enqueued);
        assert!(session.is_closed());
    }
    for length in [0u32, 16, 32, 255] {
        let (mut session, shared) = setup(4);
        let mut reply = fixture();
        reply[16..20].copy_from_slice(&length.to_le_bytes());
        reply[64 + length as usize..].fill(0);
        shared.borrow_mut().reply = Some(reply);
        session.begin(32).unwrap();
        session.poll().unwrap();
        assert!(session.poll().is_err());
        assert!(session.is_closed());
    }
}
#[test]
fn unexpected_caps_are_closed_even_when_reply_is_rejected_and_one_close_fails() {
    let (mut session, shared) = setup(4);
    let fixture = fixture();
    let mut outgoing = Outgoing::new(wire::TAG, &fixture[64..]).unwrap();
    for slot in [3, 4] {
        outgoing
            .push_cap(Capability {
                handle: handle(slot),
                rights: Rights::SEND,
            })
            .unwrap();
    }
    let mut reply = *outgoing.as_bytes();
    reply[..8].copy_from_slice(&7u64.to_le_bytes());
    shared.borrow_mut().reply = Some(reply);
    shared.borrow_mut().close_error = Some(handle(3));
    session.begin(32).unwrap();
    session.poll().unwrap();
    assert!(session.poll().is_err());
    assert_eq!(
        shared.borrow().closed,
        vec![handle(3), handle(4), handle(1), handle(2)]
    );
}
#[test]
fn eagain_is_bounded_and_post_enqueue_timeout_never_resends() {
    for queued in [false, true] {
        let (mut session, shared) = setup(3);
        shared.borrow_mut().send_again = !queued;
        shared.borrow_mut().receive_again = queued;
        session.begin(32).unwrap();
        for _ in 0..3 {
            assert!(session.poll().unwrap().is_none());
        }
        let error = session.poll().unwrap_err();
        assert_eq!(error.failure, Failure::Timeout);
        assert_eq!(
            error.delivery,
            if queued {
                Delivery::Enqueued
            } else {
                Delivery::NotEnqueued
            }
        );
        assert_eq!(shared.borrow().sent, if queued { 1 } else { 3 });
        assert!(session.is_closed());
    }
}
#[test]
fn transport_ambiguity_yield_failure_and_cancel_are_terminal_with_both_closes() {
    let (mut session, shared) = setup(4);
    shared.borrow_mut().send_error = Some(Error::InvalidReturn);
    session.begin(32).unwrap();
    assert_eq!(session.poll().unwrap_err().delivery, Delivery::Unknown);
    assert!(session.is_closed());
    let (mut session, shared) = setup(4);
    shared.borrow_mut().send_again = true;
    shared.borrow_mut().yield_error = true;
    session.begin(32).unwrap();
    assert_eq!(session.poll().unwrap_err().delivery, Delivery::NotEnqueued);
    let (mut session, shared) = setup(4);
    shared.borrow_mut().receive_error = Some(Error::Kernel(-3));
    session.begin(32).unwrap();
    session.poll().unwrap();
    assert_eq!(session.poll().unwrap_err().delivery, Delivery::Enqueued);
    for queued in [false, true] {
        let (mut session, shared) = setup(4);
        session.begin(32).unwrap();
        if queued {
            session.poll().unwrap();
        }
        shared.borrow_mut().close_error = Some(handle(1));
        assert_eq!(session.cancel(), Err(Error::Kernel(-3)));
        assert_eq!(shared.borrow().closed, vec![handle(1), handle(2)]);
        assert!(session.is_closed());
    }
}
#[test]
fn invalid_config_is_rejected() {
    let (_, shared) = setup(4);
    for (pid, budget) in [(0, 4), (7, 0), (7, 1_000_001)] {
        assert!(matches!(
            Session::new(
                Mock(shared.clone()),
                SendEndpoint::from_handle(handle(1)),
                ReceiveEndpoint::from_handle(handle(2)),
                pid,
                budget
            ),
            Err(Failure::InvalidConfig)
        ));
    }
    assert!(matches!(
        Session::new(
            Mock(shared),
            SendEndpoint::from_handle(handle(1)),
            ReceiveEndpoint::from_handle(handle(1)),
            7,
            4
        ),
        Err(Failure::InvalidConfig)
    ));
}

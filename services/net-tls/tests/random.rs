use core::num::NonZeroU64;
use nieuw_os_keyring_random_client::{Delivery, Failure as SessionFailure, Session, Transport};
use nieuw_os_net_tls::random::{Failure, KeyServiceRandom, MonotonicClock};
use nieuw_os_sdk_runtime::{Error, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint};
use rustls::crypto::SecureRandom;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct Observed {
    sends: usize,
    ids: Vec<u64>,
    closed: Vec<Handle>,
}
#[derive(Clone, Copy)]
enum Reply {
    Bytes,
    WrongSender,
    WrongId,
    Status(i32),
    Again,
    UnknownSend,
}
struct Fixture {
    observed: Arc<Mutex<Observed>>,
    reply: Reply,
    id: u64,
    length: usize,
    cleanup_error: bool,
}
impl Transport for Fixture {
    fn send(&mut self, _: SendEndpoint, message: &Outgoing) -> Result<(), Error> {
        let payload = &message.as_bytes()[64..320];
        self.id = u64::from_le_bytes(payload[4..12].try_into().unwrap());
        self.length = usize::from(u16::from_le_bytes(payload[22..24].try_into().unwrap()));
        let mut observed = self.observed.lock().unwrap();
        observed.sends += 1;
        observed.ids.push(self.id);
        if matches!(self.reply, Reply::UnknownSend) {
            return Err(Error::Kernel(-9));
        }
        Ok(())
    }
    fn receive(&mut self, _: ReceiveEndpoint) -> Result<Received, Error> {
        if matches!(self.reply, Reply::Again) {
            return Err(Error::Kernel(-8));
        }
        let mut payload = [0; 256];
        payload[0] = 0x8a;
        let id = self.id + u64::from(matches!(self.reply, Reply::WrongId));
        payload[4..12].copy_from_slice(&id.to_le_bytes());
        if let Reply::Status(status) = self.reply {
            payload[12..16].copy_from_slice(&status.to_le_bytes());
        } else {
            payload[2..4].copy_from_slice(&((16 + self.length) as u16).to_le_bytes());
            payload[16..20].copy_from_slice(b"RND1");
            payload[20..22].copy_from_slice(&1u16.to_le_bytes());
            payload[22..24].copy_from_slice(&(self.length as u16).to_le_bytes());
            payload[32..32 + self.length].fill(0xa5);
        }
        let mut envelope = *Outgoing::new(0x4b45_5931, &payload).unwrap().as_bytes();
        let sender: u64 = if matches!(self.reply, Reply::WrongSender) {
            8
        } else {
            7
        };
        envelope[..8].copy_from_slice(&sender.to_le_bytes());
        Received::from_bytes(envelope, 256)
    }
    fn yield_now(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn close(&mut self, handle: Handle) -> Result<(), Error> {
        self.observed.lock().unwrap().closed.push(handle);
        if self.cleanup_error {
            Err(Error::Kernel(-9))
        } else {
            Ok(())
        }
    }
}
struct Clock {
    ticks: Vec<Result<u64, Error>>,
    index: usize,
}
impl MonotonicClock for Clock {
    fn ticks(&mut self) -> Result<u64, Error> {
        let value = self.ticks[self.index.min(self.ticks.len() - 1)];
        self.index += 1;
        value
    }
}
fn rng(
    reply: Reply,
    ticks: &[Result<u64, Error>],
    cleanup_error: bool,
) -> (KeyServiceRandom<Fixture, Clock>, Arc<Mutex<Observed>>) {
    let observed = Arc::new(Mutex::new(Observed::default()));
    let handle = |slot: u64| Handle::from_raw((1u64 << 32) | slot).unwrap();
    let session = Session::new(
        Fixture {
            observed: observed.clone(),
            reply,
            id: 0,
            length: 0,
            cleanup_error,
        },
        SendEndpoint::from_handle(handle(1)),
        ReceiveEndpoint::from_handle(handle(2)),
        7,
        8,
    )
    .unwrap();
    (
        KeyServiceRandom::new(
            session,
            Clock {
                ticks: ticks.to_vec(),
                index: 0,
            },
            NonZeroU64::new(10).unwrap(),
        ),
        observed,
    )
}

#[test]
fn bounded_synthetic_reply_uses_the_real_sdk_lease_and_fresh_ids() {
    let (rng, observed) = rng(Reply::Bytes, &[Ok(0)], false);
    for length in [1, 32, 64, 128] {
        let mut output = vec![0x5a; length];
        SecureRandom::fill(&rng, &mut output).unwrap();
        assert_eq!(output, vec![0xa5; length]);
    }
    assert_eq!(observed.lock().unwrap().ids, [1, 2, 3, 4]);
    assert_eq!(format!("{rng:?}"), "KeyServiceRandom(<redacted>)");
}

#[test]
fn invalid_length_does_not_chunk_publish_or_destroy_a_valid_lease() {
    let (rng, observed) = rng(Reply::Bytes, &[Ok(0)], false);
    let mut output = [0x5a; 129];
    assert_eq!(
        rng.try_fill(&mut output).unwrap_err().failure,
        Failure::InvalidLength
    );
    assert_eq!(output, [0; 129]);
    rng.try_fill(&mut []).unwrap();
    assert_eq!(observed.lock().unwrap().sends, 0);
    rng.try_fill(&mut [0; 32]).unwrap();
}

#[test]
fn unknown_publication_is_never_retried_and_preserves_delivery_and_cleanup() {
    let (rng, observed) = rng(Reply::UnknownSend, &[Ok(0)], true);
    let mut output = [0x5a; 32];
    let error = rng.try_fill(&mut output).unwrap_err();
    let Failure::Session(error) = error.failure else {
        panic!("lost SDK error")
    };
    assert_eq!(error.delivery, Delivery::Unknown);
    assert_eq!(error.cleanup, Some(Error::Kernel(-9)));
    assert_eq!(output, [0; 32]);
    assert!(SecureRandom::fill(&rng, &mut output).is_err());
    assert_eq!(observed.lock().unwrap().sends, 1);
    assert_eq!(observed.lock().unwrap().closed.len(), 2);
}

#[test]
fn wrong_sender_and_request_id_never_publish_random_output() {
    for reply in [Reply::WrongSender, Reply::WrongId] {
        let (rng, observed) = rng(reply, &[Ok(0)], false);
        let mut output = [0x5a; 32];
        assert!(matches!(
            rng.try_fill(&mut output).unwrap_err().failure,
            Failure::Session(error) if error.failure == SessionFailure::Protocol
        ));
        assert_eq!(output, [0; 32]);
        assert_eq!(observed.lock().unwrap().closed.len(), 2);
    }
}

#[test]
fn service_denials_are_errors_without_rng_fallback() {
    for status in [-1, -2, -7] {
        let (rng, observed) = rng(Reply::Status(status), &[Ok(0)], false);
        let mut output = [0x5a; 32];
        assert!(matches!(
            rng.try_fill(&mut output).unwrap_err().failure,
            Failure::Service(_)
        ));
        assert_eq!(output, [0; 32]);
        assert_eq!(observed.lock().unwrap().sends, 1);
        assert_eq!(observed.lock().unwrap().closed.len(), 2);
    }
}

#[test]
fn deadline_including_a_late_complete_reply_denies_output_and_retires_lease() {
    for ticks in [
        vec![Ok(0), Ok(10)],
        vec![Ok(0), Ok(0), Ok(0), Ok(0), Ok(10)],
    ] {
        let (rng, observed) = rng(Reply::Bytes, &ticks, false);
        let mut output = [0x5a; 32];
        assert_eq!(
            rng.try_fill(&mut output).unwrap_err().failure,
            Failure::Deadline
        );
        assert_eq!(output, [0; 32]);
        assert_eq!(observed.lock().unwrap().closed.len(), 2);
        assert!(rng.try_fill(&mut output).is_err());
    }
}

#[test]
fn rollback_overflow_and_clock_failure_deny_before_publication() {
    for (ticks, failure) in [
        (vec![Ok(5), Ok(4)], Failure::ClockWentBackwards),
        (vec![Ok(u64::MAX)], Failure::DeadlineOverflow),
        (
            vec![Err(Error::Kernel(-9))],
            Failure::Clock(Error::Kernel(-9)),
        ),
    ] {
        let (rng, observed) = rng(Reply::Bytes, &ticks, false);
        let mut output = [0x5a; 32];
        assert_eq!(rng.try_fill(&mut output).unwrap_err().failure, failure);
        assert_eq!(output, [0; 32]);
        assert_eq!(observed.lock().unwrap().sends, 0);
        assert_eq!(observed.lock().unwrap().closed.len(), 2);
    }
}

#[test]
fn frozen_monotonic_clock_still_has_the_sdk_poll_budget() {
    let (rng, observed) = rng(Reply::Again, &[Ok(0)], false);
    let mut output = [0x5a; 32];
    assert!(matches!(
        rng.try_fill(&mut output).unwrap_err().failure,
        Failure::Session(error) if error.failure == SessionFailure::Timeout
    ));
    assert_eq!(output, [0; 32]);
    assert_eq!(observed.lock().unwrap().sends, 1);
    assert_eq!(observed.lock().unwrap().closed.len(), 2);
}

#[test]
fn rollback_between_requests_also_retires_the_lease() {
    let (rng, observed) = rng(
        Reply::Bytes,
        &[Ok(9), Ok(9), Ok(9), Ok(9), Ok(9), Ok(8)],
        false,
    );
    rng.try_fill(&mut [0; 32]).unwrap();
    let mut output = [0x5a; 32];
    assert_eq!(
        rng.try_fill(&mut output).unwrap_err().failure,
        Failure::ClockWentBackwards
    );
    assert_eq!(output, [0; 32]);
    assert_eq!(observed.lock().unwrap().sends, 1);
    assert_eq!(observed.lock().unwrap().closed.len(), 2);
}

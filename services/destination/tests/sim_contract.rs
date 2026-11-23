//! P12 (8.2 R2, revision 4): the simulated disk alone against the broker
//! contract of 2.7, properties 1-5, under both reset models. No service runs
//! here; each test drives `SimDisk` through the `Broker` trait.
//!
//! Not modelled by `SimDisk`, so not tested (stated, not faked); the claim
//! that the simulated disk meets the broker contract is therefore partial
//! for properties 4 and 5:
//! - property 4, the deadline itself: `SimDisk` has no clock-based deadline.
//!   A request's result and the time it stays pending are injected per
//!   request (`Fault::code`, `Fault::pending`), and a test may keep a poll
//!   pending past `TIMEOUT` on purpose (N26 (e), (f)) to exercise the
//!   service's own bound. What is tested is that the pending time runs from
//!   admission and that a -16 or -13 completion resets the device, which is
//!   reused only after the reset is acknowledged.
//! - property 5, persistence: quarantine (-13) and offline (-14) are
//!   injected per submission (`Fault::submit`), not a device state, and the
//!   reset always acknowledges after `RESET_TIMEOUT`; "until reboot" holds
//!   only while a test's hook keeps injecting. What is tested is that such
//!   a submission is not admitted and changes nothing.
mod common;

use common::*;
use destination_core::{
    device::{Broker, Request, POLL_BOUND, RESET_TIMEOUT, SUBMIT_BOUND},
    SECTOR,
};

type Snapshot = (Vec<[u8; SECTOR]>, Vec<(u64, [u8; SECTOR])>, usize, usize);

/// Device state a submission could change: durable sectors, the volatile
/// cache, and the admitted write and flush totals.
fn snapshot(disk: &SimDisk) -> Snapshot {
    (
        disk.durable.clone(),
        disk.cache.clone(),
        disk.writes,
        disk.flushes,
    )
}

fn admitted(disk: &SimDisk) -> usize {
    disk.log
        .iter()
        .filter(|event| matches!(event, Event::Submit(_)))
        .count()
}

/// Poll until the result is delivered; returns it and the tick it came at.
fn result(disk: &mut SimDisk, buffer: &mut [u8; SECTOR]) -> (i64, u64) {
    for _ in 0..10_000 {
        let code = disk.poll(buffer);
        if code != -8 {
            return (code, disk.tick);
        }
    }
    panic!("request never answered");
}

fn pending(ticks: u64) -> Fault {
    Fault {
        pending: ticks,
        ..Fault::default()
    }
}

/// Property 1: at most one request of the device is admitted at a time.
#[test]
fn p12_1_one_request_admitted() {
    both_reset_models(|drops| {
        let mut disk = SimDisk::new(SECTORS);
        disk.fault(|op, _| (op == Op::Flush).then(|| pending(5)));
        let data = [1; SECTOR];
        assert_eq!(disk.submit(Request::Flush), 0, "drops {drops}");
        assert_eq!(disk.submit(Request::Write(20, &data)), -8);
        assert_eq!(disk.submit(Request::Read(20)), -8);
        assert_eq!(admitted(&disk), 1);
        assert_eq!(result(&mut disk, &mut [0; SECTOR]).0, 0);
        assert_eq!(disk.submit(Request::Write(20, &data)), 0);
        assert_eq!(admitted(&disk), 2);
        assert_eq!(result(&mut disk, &mut [0; SECTOR]).0, 0);
    });
}

/// Property 2: a submission returns -8 and changes nothing while a request
/// is pending, while its result is ready but unconsumed, while one whose
/// poll bound the service has given up on is still held, and while the
/// device is being reset.
#[test]
fn p12_2_refused_submission_changes_nothing() {
    both_reset_models(|drops| {
        let context = format!("drops {drops}");
        let mut disk = SimDisk::new(SECTORS);
        // Pending, then ready but unconsumed.
        disk.fault(|op, c| (op == Op::Write(20) && c.writes == 0).then(|| pending(10)));
        disk.arm();
        assert_eq!(disk.submit(Request::Write(20, &[1; SECTOR])), 0);
        let before = snapshot(&disk);
        for _ in 0..20 {
            assert_eq!(
                disk.submit(Request::Write(21, &[2; SECTOR])),
                -8,
                "{context}"
            );
        }
        assert_eq!(snapshot(&disk), before, "{context}: pending or unconsumed");
        assert_eq!(admitted(&disk), 1);
        assert_eq!(result(&mut disk, &mut [0; SECTOR]).0, 0);
        assert_eq!(disk.visible()[20], [1; SECTOR]);

        // Given up on: the request stays pending past the service's poll
        // bound; the next submission is still refused for longer than the
        // submission bound and changes nothing; the result is then delivered.
        disk.fault(|op, _| (op == Op::Write(22)).then(|| pending(POLL_BOUND + SUBMIT_BOUND + 50)));
        assert_eq!(disk.submit(Request::Write(22, &[3; SECTOR])), 0);
        for _ in 0..POLL_BOUND {
            assert_eq!(disk.poll(&mut [0; SECTOR]), -8);
        }
        let before = snapshot(&disk);
        for _ in 0..=SUBMIT_BOUND {
            assert_eq!(disk.submit(Request::Flush), -8, "{context}: given up");
        }
        assert_eq!(snapshot(&disk), before, "{context}: given up");
        assert_eq!(result(&mut disk, &mut [0; SECTOR]).0, 0);

        // While the device is being reset after a timed-out request.
        disk.fault(|op, _| (op == Op::Write(23)).then(|| Fault::code(-16)));
        assert_eq!(disk.submit(Request::Write(23, &[4; SECTOR])), 0);
        assert_eq!(result(&mut disk, &mut [0; SECTOR]).0, -16);
        disk.fault = None;
        let before = snapshot(&disk);
        let mut refused = 0;
        while disk.submit(Request::Write(24, &[5; SECTOR])) == -8 {
            refused += 1;
            assert_eq!(snapshot(&disk), before, "{context}: resetting");
        }
        assert!(refused >= RESET_TIMEOUT - 1, "{context}: refused {refused}");
        assert_eq!(result(&mut disk, &mut [0; SECTOR]).0, 0);
    });
}

/// Property 3: each admitted request is answered exactly once, also when a
/// reset ends it: -8 while pending, then its result once; a further poll
/// does not return it again (it gets -1, the M3 broker's `WrongTicket`).
#[test]
fn p12_3_answered_exactly_once() {
    both_reset_models(|drops| {
        let context = format!("drops {drops}");
        let mut disk = SimDisk::new(SECTORS);
        disk.fault(|op, _| match op {
            Op::Write(20) => Some(pending(3)),
            Op::Write(21) => Some(Fault::code(-16)),
            Op::Read(22) => Some(Fault::code(-13)),
            _ => None,
        });
        assert_eq!(disk.submit(Request::Write(20, &[1; SECTOR])), 0);
        assert_eq!(disk.poll(&mut [0; SECTOR]), -8, "{context}: pending");
        assert_eq!(result(&mut disk, &mut [0; SECTOR]).0, 0);
        assert_eq!(disk.poll(&mut [0; SECTOR]), -1, "{context}: second poll");

        // A read result once; a second poll leaves the buffer unchanged.
        assert_eq!(disk.submit(Request::Read(20)), 0);
        let mut buffer = [0; SECTOR];
        assert_eq!(result(&mut disk, &mut buffer).0, 0);
        assert_eq!(buffer, [1; SECTOR]);
        let mut again = [7; SECTOR];
        assert_eq!(disk.poll(&mut again), -1, "{context}: second read poll");
        assert_eq!(again, [7; SECTOR]);

        // Ended by a reset: a mutating request -16, a read -13, each once.
        for (request, code) in [
            (Request::Write(21, &[2; SECTOR]), -16),
            (Request::Read(22), -13),
        ] {
            let mut code_seen = -8;
            while code_seen == -8 {
                code_seen = disk.submit(request);
            }
            assert_eq!(code_seen, 0, "{context}");
            assert_eq!(result(&mut disk, &mut [0; SECTOR]).0, code, "{context}");
            assert_eq!(disk.poll(&mut [0; SECTOR]), -1, "{context}: after reset");
        }
        // The reset model decides only what happens to the unflushed cache.
        assert_eq!(drops, disk.dropped > 0, "{context}: model exercised");
        assert_eq!(disk.visible()[20] == [1; SECTOR], !drops, "{context}");
    });
}

/// Property 4, observable part: a request's pending time runs from its
/// admission, not from its first submission attempt; after a -16 or -13
/// completion the device is reset and admits nothing before the reset is
/// acknowledged. (The deadline itself is not modelled; module comment.)
#[test]
fn p12_4_admission_time_and_reset_fence() {
    both_reset_models(|drops| {
        let context = format!("drops {drops}");
        let mut disk = SimDisk::new(SECTORS);
        disk.fault(|op, c| match (op, c.flushes) {
            (Op::Flush, 0) => Some(Fault {
                again: Some(Some(RESET_TIMEOUT)),
                pending: 40,
                ..Fault::default()
            }),
            (Op::Flush, 1) => Some(Fault::code(-16)),
            _ => None,
        });
        disk.arm();
        let first_attempt = disk.tick + 1;
        while disk.submit(Request::Flush) == -8 {}
        let admission = disk.tick;
        assert!(admission >= first_attempt + RESET_TIMEOUT - 1, "{context}");
        let (code, at) = result(&mut disk, &mut [0; SECTOR]);
        assert_eq!((code, at), (0, admission + 40), "{context}: from admission");

        assert_eq!(disk.submit(Request::Flush), 0);
        let (code, failed_at) = result(&mut disk, &mut [0; SECTOR]);
        assert_eq!(code, -16);
        while disk.submit(Request::Read(16)) == -8 {}
        assert!(disk.tick >= failed_at + RESET_TIMEOUT, "{context}: fence");
        let ack = disk.log.iter().rposition(|e| *e == Event::ResetAck);
        let read = disk
            .log
            .iter()
            .rposition(|e| *e == Event::Submit(Op::Read(16)));
        assert!(ack.unwrap() < read.unwrap(), "{context}: ack before reuse");
        assert_eq!(result(&mut disk, &mut [0; SECTOR]).0, 0);
    });
}

/// Property 5, observable part: a submission answered -13 (quarantined) or
/// -14 (offline) is not admitted and changes nothing, for as long as it is
/// injected. (Persistence is not modelled; module comment.)
#[test]
fn p12_5_quarantined_or_offline_not_admitted() {
    both_reset_models(|drops| {
        for code in [-13, -14] {
            let context = format!("drops {drops} code {code}");
            let mut disk = SimDisk::new(SECTORS);
            disk.fault(move |_, _| {
                Some(Fault {
                    submit: code,
                    ..Fault::default()
                })
            });
            let before = snapshot(&disk);
            for request in [
                Request::Write(20, &[1; SECTOR]),
                Request::Flush,
                Request::Read(20),
            ] {
                for _ in 0..3 {
                    assert_eq!(disk.submit(request), code, "{context}");
                }
            }
            assert_eq!(snapshot(&disk), before, "{context}");
            assert_eq!(admitted(&disk), 0, "{context}");
            assert_eq!(
                disk.poll(&mut [0; SECTOR]),
                -1,
                "{context}: nothing pending"
            );
        }
    });
}

//! P12 (8.2 R2, revision 4; revision 5 Q4, Q7): the simulated disk alone
//! against the broker contract of 2.7, properties 1-5, under both reset
//! models. No service runs here; each test drives `SimDisk` through the
//! `Broker` trait.
//!
//! Modelled by `SimDisk` and tested here by its own behaviour, not by
//! injected results:
//! - property 4: in the opt-in mode `Deadline::Enforced` the model answers a
//!   request whose device result is ready only after admission + `TIMEOUT`
//!   itself, -16 for WRITE and FLUSH and -13 for READ, and resets the device,
//!   which admits nothing until the reset is acknowledged
//!   (`p12_4_deadline_answers_and_resets`), also when the first poll comes
//!   after that result is ready (`p12_4_deadline_decided_by_ready_time`). The
//!   deadline is evaluated at the poll, so the reset starts at the poll that
//!   consumes that result. The mode is off by default because N26 (e), N26 (f)
//!   and the "given up on" case of property 2 keep a request pending past the
//!   deadline on purpose (2.7, host models); they opt out with
//!   `Deadline::Unchecked`. Every other test runs under `Deadline::Tripwire`,
//!   which fails the test on such a request (review 9, OPT9-7).
//! - property 5: a reset that is not acknowledged quarantines the device
//!   (-13), and a failed re-initialisation after an acknowledged reset takes
//!   it offline (-14); either is device state that refuses every submission
//!   until a simulated reboot (`SimDisk::rebooted`), with the reset caused by
//!   the model's own deadline or by a -16 completion
//!   (`p12_5_quarantine_and_offline_until_reboot`). The state is entered at
//!   the first submission at or after the end of the reset interval. A
//!   submission answered -13 or -14 by an injected fault is held the same
//!   way. The device's own reset injected with `Fault::again` (N26 (d))
//!   always ends ready.
//!
//! Still chosen by the test: per request, a device result other than the
//! deadline's and how long the request stays pending; per disk, how a reset
//! ends after a -16 or -13 result (`SimDisk::reset_end`, default
//! acknowledged). The P12 tests that poll
//! with no request pending on purpose, and the Q6 kill test in `broker.rs`,
//! set `stray_polls_allowed`; every other test fails on such a poll when its
//! `SimDisk` is dropped (2.7 poll rule, Q6).
mod common;

use common::*;
use destination_core::{
    device::{Broker, Request, POLL_BOUND, RESET_TIMEOUT, SUBMIT_BOUND, TIMEOUT},
    SECTOR,
};

type Snapshot = (
    Vec<[u8; SECTOR]>,
    Vec<(u64, [u8; SECTOR])>,
    usize,
    usize,
    BrokerState,
);

/// Device state a submission or poll could change: durable sectors, the
/// volatile cache, the admitted write and flush totals, and the broker state
/// (pending request, reset, quarantine or offline). Not the test
/// instruments: the tick, the log, the stray-poll count, the fault-hook
/// `counts`, the power-cut flag `crashed` and the non-vacuity counter
/// `dropped`.
fn snapshot(disk: &SimDisk) -> Snapshot {
    (
        disk.durable.clone(),
        disk.cache.clone(),
        disk.writes,
        disk.flushes,
        disk.broker_state(),
    )
}

/// A further poll (property 3): -1, and it changes nothing, neither the
/// device state nor the caller's buffer.
fn further_poll(disk: &mut SimDisk, context: &str) {
    let before = snapshot(disk);
    let mut buffer = [7; SECTOR];
    assert_eq!(disk.poll(&mut buffer), -1, "{context}");
    assert_eq!(buffer, [7; SECTOR], "{context}: buffer unchanged");
    assert_eq!(snapshot(disk), before, "{context}: device state unchanged");
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
        // This breaks property 4 on purpose (2.7, host models).
        disk.deadline = Deadline::Unchecked;
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
        disk.deadline = Deadline::Tripwire;

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
/// does not return it again (it gets -1, the M3 broker's `WrongTicket`) and
/// changes nothing: a device-state snapshot before and after each further
/// poll is equal, and the caller's buffer is unchanged (Q7).
#[test]
fn p12_3_answered_exactly_once() {
    both_reset_models(|drops| {
        let context = format!("drops {drops}");
        let mut disk = SimDisk::new(SECTORS);
        // Further polls with no request pending are this test's subject.
        disk.stray_polls_allowed = true;
        disk.fault(|op, _| match op {
            Op::Write(20) => Some(pending(3)),
            Op::Write(21) => Some(Fault::code(-16)),
            Op::Read(22) => Some(Fault::code(-13)),
            _ => None,
        });
        assert_eq!(disk.submit(Request::Write(20, &[1; SECTOR])), 0);
        assert_eq!(disk.poll(&mut [0; SECTOR]), -8, "{context}: pending");
        assert_eq!(result(&mut disk, &mut [0; SECTOR]).0, 0);
        further_poll(&mut disk, &format!("{context}: second poll"));

        // A read result once; a second poll leaves the buffer unchanged.
        assert_eq!(disk.submit(Request::Read(20)), 0);
        let mut buffer = [0; SECTOR];
        assert_eq!(result(&mut disk, &mut buffer).0, 0);
        assert_eq!(buffer, [1; SECTOR]);
        further_poll(&mut disk, &format!("{context}: second read poll"));

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
            further_poll(&mut disk, &format!("{context}: after reset {code}"));
        }
        assert_eq!(disk.stray_polls, 4, "{context}: the four further polls");
        // The reset model decides only what happens to the unflushed cache.
        assert_eq!(drops, disk.dropped > 0, "{context}: model exercised");
        assert_eq!(disk.visible()[20] == [1; SECTOR], !drops, "{context}");
    });
}

/// Property 4 with an injected result: a request's pending time runs from its
/// admission, not from its first submission attempt; after a -16 or -13
/// completion the device is reset and admits nothing before the reset is
/// acknowledged. The model's own deadline: `p12_4_deadline_answers_and_resets`.
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

/// Property 5 with an injected submission result: a submission answered -13
/// (quarantined) or -14 (offline) is not admitted, changes nothing on the
/// medium, and is held as device state: every later submission gets the same
/// code, also once the fault hook no longer returns it, until a simulated
/// reboot. A hook that then asks for the device's own reset (`Fault::again`)
/// changes nothing either: no reset starts and the cache is kept (RV1-O2).
#[test]
fn p12_5_quarantined_or_offline_not_admitted() {
    both_reset_models(|drops| {
        for code in [-13, -14] {
            let context = format!("drops {drops} code {code}");
            let mut disk = SimDisk::new(SECTORS);
            // Polls with nothing pending on purpose (below).
            disk.stray_polls_allowed = true;
            // A cached write, so a refused submission that dropped it shows.
            assert_eq!(disk.submit(Request::Write(21, &[2; SECTOR])), 0);
            assert_eq!(result(&mut disk, &mut [0; SECTOR]).0, 0);
            disk.fault(move |_, _| {
                Some(Fault {
                    submit: code,
                    ..Fault::default()
                })
            });
            let medium = |disk: &SimDisk| (disk.durable.clone(), disk.cache.clone());
            let before = medium(&disk);
            assert_eq!(disk.submit(Request::Flush), code, "{context}");
            assert_eq!(disk.down(), Some(code), "{context}: held");
            disk.fault(|_, _| {
                Some(Fault {
                    again: Some(Some(RESET_TIMEOUT)),
                    ..Fault::default()
                })
            });
            let held = snapshot(&disk);
            for request in [
                Request::Write(20, &[1; SECTOR]),
                Request::Flush,
                Request::Read(20),
            ] {
                for _ in 0..3 {
                    assert_eq!(disk.submit(request), code, "{context}");
                }
            }
            assert_eq!(snapshot(&disk), held, "{context}");
            assert_eq!(medium(&disk), before, "{context}");
            assert_eq!(admitted(&disk), 1, "{context}: only the cached write");
            assert_eq!(
                disk.poll(&mut [0; SECTOR]),
                -1,
                "{context}: nothing pending"
            );
            let mut disk = disk.rebooted();
            assert_eq!(disk.submit(Request::Read(20)), 0, "{context}: rebooted");
            assert_eq!(result(&mut disk, &mut [0; SECTOR]).0, 0);
        }
    });
}

/// Property 4 by the model's own deadline (opt-in `Deadline::Enforced`, Q4):
/// a request still pending at admission + `TIMEOUT` is answered by the
/// broker at that tick, -16 for WRITE and FLUSH and -13 for READ; a read
/// fills no buffer and (a model choice, not property 4) nothing the request
/// carried lands; a further poll returns -1 and changes nothing (property 3
/// for a request ended by the model's own reset); the device is reset (the reset
/// model decides the unflushed cache) and admits nothing before the reset is
/// acknowledged. The deadline runs from admission, not from the first
/// submission attempt.
/// Kills: a deadline that does not answer (the request stays pending for
/// 100 x `TIMEOUT` and `result` panics); a deadline measured from the first
/// attempt (answered early); the other kind's code; reuse before the ack.
#[test]
fn p12_4_deadline_answers_and_resets() {
    both_reset_models(|drops| {
        let data = [9; SECTOR];
        for (name, request, op, code) in [
            ("write", Request::Write(30, &data), Op::Write(30), -16),
            ("flush", Request::Flush, Op::Flush, -16),
            ("read", Request::Read(20), Op::Read(20), -13),
        ] {
            let context = format!("drops {drops} {name}");
            let mut disk = SimDisk::new(SECTORS);
            disk.deadline = Deadline::Enforced;
            // One further poll on purpose (property 3, below).
            disk.stray_polls_allowed = true;
            // A completed, unflushed write that the reset keeps or drops.
            assert_eq!(disk.submit(Request::Write(20, &[1; SECTOR])), 0);
            assert_eq!(result(&mut disk, &mut [0; SECTOR]).0, 0);
            disk.fault(move |o, _| (o == op).then(|| pending(100 * TIMEOUT)));
            assert_eq!(disk.submit(request), 0, "{context}");
            let admission = disk.tick;
            let mut buffer = [7; SECTOR];
            let (got, at) = result(&mut disk, &mut buffer);
            assert_eq!((got, at), (code, admission + TIMEOUT), "{context}");
            assert_eq!(buffer, [7; SECTOR], "{context}: no read data");
            further_poll(&mut disk, &format!("{context}: after deadline"));
            assert_eq!(disk.stray_polls, 1, "{context}");
            disk.fault = None;
            let before = snapshot(&disk);
            while disk.submit(Request::Read(21)) == -8 {
                assert_eq!(snapshot(&disk), before, "{context}: resetting");
            }
            assert!(disk.tick >= at + RESET_TIMEOUT, "{context}: fence");
            let ack = disk.log.iter().rposition(|e| *e == Event::ResetAck);
            let reuse = disk
                .log
                .iter()
                .rposition(|e| *e == Event::Submit(Op::Read(21)));
            assert!(ack.unwrap() < reuse.unwrap(), "{context}: ack first");
            assert_eq!(result(&mut disk, &mut [0; SECTOR]).0, 0);
            assert_eq!(drops, disk.dropped > 0, "{context}: model exercised");
            assert_eq!(disk.visible()[20] == [1; SECTOR], !drops, "{context}");
            assert_eq!(disk.visible()[30], [0; SECTOR], "{context}: not landed");
        }

        // From admission: the flush first waits out the device's own reset.
        let mut disk = SimDisk::new(SECTORS);
        disk.deadline = Deadline::Enforced;
        disk.fault(|op, _| {
            (op == Op::Flush).then(|| Fault {
                again: Some(Some(RESET_TIMEOUT)),
                pending: 100 * TIMEOUT,
                ..Fault::default()
            })
        });
        let first_attempt = disk.tick + 1;
        while disk.submit(Request::Flush) == -8 {}
        let admission = disk.tick;
        assert!(
            admission >= first_attempt + RESET_TIMEOUT - 1,
            "drops {drops}"
        );
        let (code, at) = result(&mut disk, &mut [0; SECTOR]);
        assert_eq!((code, at), (-16, admission + TIMEOUT), "drops {drops}");
    });
}

/// Property 5 as device state (Q4): after a reset that is not acknowledged
/// the device is quarantined and every submission gets -13; after a failed
/// re-initialisation it is offline and every submission gets -14; both hold
/// for far longer than any service bound, change nothing, and end only with
/// a simulated reboot. The reset is caused by the model's own deadline or by
/// a -16 completion.
/// Kills: a model that admits a submission after quarantine or offline
/// before the simulated reboot; one that ends either state by itself.
#[test]
fn p12_5_quarantine_and_offline_until_reboot() {
    both_reset_models(|drops| {
        for (end, code) in [(ResetEnd::Timeout, -13), (ResetEnd::ReinitFails, -14)] {
            for by_deadline in [false, true] {
                let context = format!("drops {drops} {end:?} deadline {by_deadline}");
                let mut disk = SimDisk::new(SECTORS);
                // Polls with nothing pending on purpose (below).
                disk.stray_polls_allowed = true;
                disk.reset_end = end;
                if by_deadline {
                    disk.deadline = Deadline::Enforced;
                }
                for request in [
                    Request::Write(21, &[2; SECTOR]),
                    Request::Flush,
                    Request::Write(20, &[1; SECTOR]),
                ] {
                    assert_eq!(disk.submit(request), 0, "{context}");
                    assert_eq!(result(&mut disk, &mut [0; SECTOR]).0, 0);
                }
                disk.fault(move |op, _| {
                    (op == Op::Write(22)).then(|| {
                        if by_deadline {
                            pending(100 * TIMEOUT)
                        } else {
                            Fault::code(-16)
                        }
                    })
                });
                assert_eq!(disk.submit(Request::Write(22, &[3; SECTOR])), 0);
                let (got, failed_at) = result(&mut disk, &mut [0; SECTOR]);
                assert_eq!(got, -16, "{context}");
                disk.fault = None;
                let admitted_before = admitted(&disk);
                let medium = (disk.durable.clone(), disk.cache.clone());
                while disk.submit(Request::Flush) == -8 {
                    assert!(disk.tick < failed_at + RESET_TIMEOUT, "{context}");
                }
                assert!(
                    disk.tick >= failed_at + RESET_TIMEOUT,
                    "{context}: reset first"
                );
                assert_eq!(disk.down(), Some(code), "{context}");
                // A refused submission does not start the device's own reset.
                disk.fault(|_, _| {
                    Some(Fault {
                        again: Some(Some(RESET_TIMEOUT)),
                        ..Fault::default()
                    })
                });
                let held = snapshot(&disk);
                for _ in 0..3 * TIMEOUT {
                    for request in [
                        Request::Write(23, &[4; SECTOR]),
                        Request::Flush,
                        Request::Read(21),
                    ] {
                        assert_eq!(disk.submit(request), code, "{context}");
                    }
                    assert_eq!(disk.poll(&mut [0; SECTOR]), -1, "{context}");
                }
                assert_eq!(snapshot(&disk), held, "{context}: changes nothing");
                assert_eq!((disk.durable.clone(), disk.cache.clone()), medium);
                assert_eq!(admitted(&disk), admitted_before, "{context}");
                assert_eq!(
                    disk.log.contains(&Event::ResetAck),
                    end == ResetEnd::ReinitFails,
                    "{context}: acknowledged only before a failed re-initialisation"
                );
                assert_eq!(drops, disk.dropped > 0, "{context}: model exercised");
                assert_eq!(disk.visible()[20] == [1; SECTOR], !drops, "{context}");

                // The simulated reboot keeps the durable image only.
                let mut disk = disk.rebooted();
                assert_eq!(disk.submit(Request::Read(21)), 0, "{context}: rebooted");
                let mut buffer = [0; SECTOR];
                assert_eq!(result(&mut disk, &mut buffer).0, 0);
                assert_eq!(buffer, [2; SECTOR], "{context}");
                assert_eq!(disk.visible()[20], [0; SECTOR], "{context}");
            }
        }
    });
}

/// Property 4 (RV1-R1): the deadline is decided by when the device result is
/// ready, not by when the caller polls. Each request is first polled long
/// after its result is ready, the ticks advanced by refused submissions. A
/// result ready exactly at admission + `TIMEOUT` is the device's; one ready a
/// few ticks later is answered by the deadline and resets the device.
/// Kills: a deadline decided at the poll tick (the late result is delivered
/// as the device's 0); a boundary that counts as overdue (-16 or -13 there).
#[test]
fn p12_4_deadline_decided_by_ready_time() {
    both_reset_models(|drops| {
        for (name, request, op, code) in [
            ("flush", Request::Flush, Op::Flush, -16),
            ("read", Request::Read(20), Op::Read(20), -13),
        ] {
            for (extra, want) in [(0, 0), (5, code)] {
                let context = format!("drops {drops} {name} +{extra}");
                let mut disk = SimDisk::new(SECTORS);
                disk.deadline = Deadline::Enforced;
                assert_eq!(disk.submit(Request::Write(20, &[1; SECTOR])), 0);
                assert_eq!(result(&mut disk, &mut [0; SECTOR]).0, 0);
                disk.fault(move |o, _| (o == op).then(|| pending(TIMEOUT + extra)));
                assert_eq!(disk.submit(request), 0, "{context}");
                let admission = disk.tick;
                while disk.tick < admission + 2 * TIMEOUT {
                    assert_eq!(disk.submit(Request::Read(21)), -8, "{context}");
                }
                let mut buffer = [7; SECTOR];
                assert_eq!(disk.poll(&mut buffer), want, "{context}");
                let effect = want == 0;
                let data = if effect && op == Op::Read(20) { 1 } else { 7 };
                assert_eq!(buffer, [data; SECTOR], "{context}: read data");
                let flushed = effect && op == Op::Flush;
                assert_eq!(disk.durable[20] == [1; SECTOR], flushed, "{context}");
                // Only the deadline's result resets: nothing is admitted yet.
                let next = disk.submit(Request::Read(21));
                assert_eq!(next == -8, !effect, "{context}: reset fence");
                let lost = !effect && drops;
                assert_eq!(disk.dropped > 0, lost, "{context}: model exercised");
            }
        }
    });
}

/// The default mode fails the test on the same sparse poll (RV1-R1).
#[test]
#[should_panic(expected = "pending past TIMEOUT")]
fn deadline_tripwire_flags_overdue_sparse_poll() {
    let mut disk = SimDisk::new(SECTORS);
    disk.fault(|op, _| (op == Op::Flush).then(|| pending(TIMEOUT + 5)));
    assert_eq!(disk.submit(Request::Flush), 0);
    while disk.tick < 2 * TIMEOUT {
        assert_eq!(disk.submit(Request::Read(21)), -8);
    }
    disk.poll(&mut [0; SECTOR]);
}

/// OPT9-7: in the default mode a request whose device result is ready only
/// after admission + `TIMEOUT` fails the test, so the default-off deadline
/// differs from an always-on one only in the tests that opt out explicitly
/// (N26 (e), N26 (f), the "given up on" case of property 2). The boundary, a
/// result ready exactly at the deadline being the device's, is pinned without
/// a panic in `p12_4_deadline_decided_by_ready_time`.
#[test]
#[should_panic(expected = "pending past TIMEOUT")]
fn deadline_tripwire_flags_overdue_request() {
    let mut disk = SimDisk::new(SECTORS);
    disk.fault(|op, c| (op == Op::Flush).then(|| pending(TIMEOUT + c.flushes as u64)));
    assert_eq!(disk.submit(Request::Flush), 0);
    assert_eq!(result(&mut disk, &mut [0; SECTOR]).0, 0);
    assert_eq!(disk.submit(Request::Flush), 0);
    result(&mut disk, &mut [0; SECTOR]);
}

/// Q6: the stray-poll counter is not vacuous. A poll after the result was
/// delivered counts, and dropping the disk fails the test.
#[test]
#[should_panic(expected = "stray poll")]
fn stray_poll_tripwire() {
    let mut disk = SimDisk::new(SECTORS);
    assert_eq!(disk.submit(Request::Flush), 0);
    assert_eq!(result(&mut disk, &mut [0; SECTOR]).0, 0);
    assert_eq!(disk.poll(&mut [0; SECTOR]), -1);
}

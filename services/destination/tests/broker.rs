//! Kernel -8 and the service's own wait bounds (spec 2.7): N26; the
//! "Device results" mapping of 2.5 (revision 3): N33; the SEAL content check
//! of 2.7 step 3 (O17 (a)): N37 (a), (b).
//! `SimDisk` advances its clock by one tick per submission and per poll.
//! N26 runs under both reset models (section 6, revision 3); only (d)
//! reaches a reset that drops cached data, and (a), (b), (c), (e) and (f)
//! assert that none was dropped (review F2).
mod common;

use common::*;
use destination_core::{
    catalog::Record,
    device::{POLL_BOUND, RESET_TIMEOUT, SUBMIT_BOUND, TIMEOUT},
    service::{Export, Outcome, State},
    wire, Status, DATA_START, SECTOR,
};

/// Slack for the ticks spent on the requests around the bounded one.
const SLACK: u64 = 30;

fn forever() -> Fault {
    Fault {
        again: Some(None),
        ..Fault::default()
    }
}

/// N26 (a): -8 forever to the mount FLUSH: failed mount after one bound.
/// Kills: a resubmission loop without the bound (the call budget panics).
#[test]
fn n26a_mount_flush_again_forever() {
    both_reset_models(|_| {
        let mut disk = SimDisk::new(SECTORS);
        disk.fault(|op, _| (op == Op::Flush).then(forever));
        let mut h = H::boot(disk);
        assert_eq!(h.svc.state(), State::Failed { no_device: false });
        assert!(h.disk.tick <= SUBMIT_BOUND + SLACK);
        assert_eq!(h.info().0, Status::Io);
        let start = h.disk.tick;
        assert_eq!(h.unbind(1), Status::Io);
        assert_eq!(h.disk.tick, start, "UNBIND issues no device request");
        assert_eq!(h.disk.writes, 0);
        // F2: no reset that drops the cache is reached, so both models agree.
        assert_eq!(h.disk.dropped, 0);
    });
}

/// N26 (b): -8 forever from the barrier-2 flush on: SEAL `Unknown` after one
/// bound, the remount FLUSH exceeds the bound too: failed mount.
#[test]
fn n26b_barrier2_again_forever() {
    both_reset_models(|_| {
        let mut h = H::formatted();
        assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
        let bytes = archive(1, 1, 300, 1);
        let entry = h.write_all(1, &bytes);
        h.disk.arm();
        h.disk
            .fault(|op, c| (op == Op::Flush && c.flushes == 1).then(forever));
        let start = h.disk.tick;
        assert_eq!(h.seal(1, entry, bytes.len() as u64).status, Status::Unknown);
        assert_eq!(h.svc.state(), State::Failed { no_device: false });
        let mutations = h.mutations();
        assert_eq!(h.unbind(1), Status::Io);
        let elapsed = h.disk.tick - start;
        assert!(
            elapsed <= 2 * SUBMIT_BOUND + SLACK,
            "UNBIND served after {elapsed} ticks"
        );
        assert_eq!(h.info().0, Status::Io);
        assert_eq!(h.mutations(), mutations, "no device write follows");
        // F2: no reset that drops the cache is reached, so both models agree.
        assert_eq!(h.disk.dropped, 0);
    });
}

/// N26 (c): -8 forever to an APPEND write: APPEND `Io` after one bound,
/// writing entry discarded, safe path.
#[test]
fn n26c_append_write_again_forever() {
    both_reset_models(|_| {
        let mut h = H::formatted();
        assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
        assert_eq!(h.bind(2, wire::BIND_EXPORT, 0), Status::Ok);
        let entry = wire::header(&h.create(1, b"x").reply.unwrap()).entry;
        h.disk
            .fault(|op, _| matches!(op, Op::Write(_)).then(forever));
        let start = h.disk.tick;
        let mut offset = 0;
        let status = loop {
            let outcome = h.append(1, entry, offset, &[1; 160]);
            if outcome.status != Status::Ok {
                break outcome.status;
            }
            offset += 160;
        };
        assert_eq!(status, Status::Io);
        assert!(h.svc.safe_path());
        assert_eq!(h.unbind(1), Status::Ok);
        assert!(h.disk.tick - start <= SUBMIT_BOUND + SLACK);
        assert_eq!(h.create(2, b"y").status, Status::Access);
        // F2: no reset that drops the cache is reached, so both models agree.
        assert_eq!(h.disk.dropped, 0);
    });
}

/// N26 (d): -8 on submission for `RESET_TIMEOUT` ticks (disk 1's own reset)
/// and a poll pending for `TIMEOUT` - 1 ticks both complete normally: no
/// `Io`, no safe path. Under the cache-dropping reset model the reset loses
/// the unflushed SEAL tail write, so the step-3 readback refuses the extent
/// (`Corrupt`, nothing listed) and barrier 2 is never reached; the pending
/// poll half is then covered by the cache-kept model only; no catalog
/// sector is written (revision 4).
/// Kills: a bound shorter than one legitimate reset; the submission bound
/// applied to the pending poll; a SEAL that seals without the readback
/// (dropping model lists an entry that fails VERIFY).
#[test]
fn n26d_legitimate_waits_complete() {
    both_reset_models(|drops| {
        let mut h = H::formatted();
        assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
        let bytes = archive(1, 1, 300, 1);
        let entry = h.write_all(1, &bytes);
        h.disk.arm();
        h.disk.fault(|op, c| match (op, c.flushes) {
            (Op::Flush, 0) => Some(Fault {
                again: Some(Some(RESET_TIMEOUT)),
                ..Fault::default()
            }),
            (Op::Flush, 1) => Some(Fault {
                pending: TIMEOUT - 1,
                ..Fault::default()
            }),
            _ => None,
        });
        let sealed = h.seal(1, entry, bytes.len() as u64).status;
        assert!(!h.svc.safe_path());
        if drops {
            assert!(h.disk.dropped > 0, "the reset dropped the tail write");
            assert_eq!(sealed, Status::Corrupt);
            assert_eq!(h.disk.counts.catalog_writes, 0, "no catalog write");
            assert_eq!(h.list_ids(0).1, Vec::<u64>::new());
        } else {
            assert_eq!(sealed, Status::Ok);
            assert_eq!(h.list_ids(0).1, vec![entry]);
        }
    });
}

/// N26 (e): a completion poll pending beyond `TIMEOUT` + 2 x `RESET_TIMEOUT`
/// on a write counts as `Unknown` and starts the safe path.
/// Kills: a poll loop without a bound (the call budget panics).
#[test]
fn n26e_poll_beyond_bound_is_unknown() {
    both_reset_models(|_| {
        let mut h = H::formatted();
        assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
        let entry = wire::header(&h.create(1, b"x").reply.unwrap()).entry;
        h.disk.fault(|op, _| {
            matches!(op, Op::Write(_)).then_some(Fault {
                pending: 100 * POLL_BOUND,
                ..Fault::default()
            })
        });
        let start = h.disk.tick;
        let mut offset = 0;
        let status = loop {
            let outcome = h.append(1, entry, offset, &[1; 160]);
            if outcome.status != Status::Ok {
                break outcome.status;
            }
            offset += 160;
        };
        assert_eq!(status, Status::Unknown);
        assert!(h.disk.tick - start <= POLL_BOUND + SLACK);
        assert!(h.svc.safe_path());
        // F2: no reset that drops the cache is reached, so both models agree.
        assert_eq!(h.disk.dropped, 0);
    });
}

/// Kernel -14 and -13 on submission of a destination write (2.5, 5.3,
/// revision 3): both `Io`, both start the safe path.
#[test]
fn kernel_offline_on_write() {
    for code in [-14, -13] {
        let mut h = H::formatted();
        assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
        let entry = wire::header(&h.create(1, b"x").reply.unwrap()).entry;
        h.disk.fault(move |op, _| {
            matches!(op, Op::Write(_)).then_some(Fault {
                submit: code,
                ..Fault::default()
            })
        });
        let mut offset = 0;
        let status = loop {
            let outcome = h.append(1, entry, offset, &[1; 160]);
            if outcome.status != Status::Ok {
                break outcome.status;
            }
            offset += 160;
        };
        assert_eq!(status, Status::Io, "submission {code}");
        assert!(h.svc.safe_path());
    }
}

/// N26 (f), O9 (review R2): the barrier-2 FLUSH stays pending past the poll
/// bound, so SEAL answers `Unknown`; the broker still holds that request, so
/// the remount FLUSH is refused with -8 until the submission bound and the
/// instance enters the failed-mount state with the outcome *unknown*.
/// Kills: a remount that treats the refused FLUSH as done, or a submission
/// loop without a bound after an expired poll.
#[test]
fn n26f_remount_after_expired_poll_bound() {
    both_reset_models(|_| {
        let mut h = H::formatted();
        assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
        let bytes = archive(1, 1, 300, 1);
        let entry = h.write_all(1, &bytes);
        h.disk.arm();
        h.disk.fault(|op, c| {
            (op == Op::Flush && c.flushes == 1).then_some(Fault {
                pending: 100 * POLL_BOUND,
                ..Fault::default()
            })
        });
        let start = h.disk.tick;
        assert_eq!(h.seal(1, entry, bytes.len() as u64).status, Status::Unknown);
        assert!(h.disk.tick - start <= POLL_BOUND + SUBMIT_BOUND + 2 * SLACK);
        assert!(h.disk.log.contains(&Event::Refused(Op::Flush)));
        assert_eq!(h.svc.state(), State::Failed { no_device: false });
        assert!(h.svc.safe_path());
        let mutations = h.mutations();
        assert_eq!(h.info().0, Status::Io);
        assert_eq!(h.list_ids(1), (Status::Unknown, vec![]));
        assert_eq!(h.mutations(), mutations, "no device write follows");
        // F2: no reset that drops the cache is reached, so both models agree.
        assert_eq!(h.disk.dropped, 0);
    });
}

/// The broker results of N33: (name, fault). Submission -13, -14 and another
/// error (-1, a refused `prepare`), then completion -12, -13, -16 and -14.
fn n33_results() -> [(&'static str, Fault); 7] {
    let submit = |code| Fault {
        submit: code,
        ..Fault::default()
    };
    [
        ("submit -13", submit(-13)),
        ("submit -14", submit(-14)),
        ("submit -1", submit(-1)),
        ("complete -12", Fault::code(-12)),
        ("complete -13", Fault::code(-13)),
        ("complete -16", Fault::code(-16)),
        ("complete -14", Fault::code(-14)),
    ]
}

/// Inject `fault` into the first request for which `hit` holds.
fn once(h: &mut H, fault: Fault, hit: impl Fn(Op, &Counts) -> bool + 'static) {
    h.disk.arm();
    let mut fired = false;
    h.disk.fault(move |op, c| {
        let now = !fired && hit(op, c);
        fired |= now;
        now.then_some(fault)
    });
}

/// N33 (reads): READ, VERIFY and the FORMAT scan of a blank instance, for
/// every broker result. `Io` except completion -13 `Timeout`; never
/// `NoDevice`, never `Corrupt`; no safe path; the instance stays mounted
/// (blank for FORMAT) and writes nothing.
/// Kills: a -14 that replies `NoDevice` from READ, VERIFY or FORMAT; a read
/// failure that VERIFY answers `Corrupt` or that starts the safe path; a read
/// whose completion -13 is not `Timeout`.
#[test]
fn n33_device_results_reads() {
    for (name, fault) in n33_results() {
        let expected = if name == "complete -13" {
            Status::Timeout
        } else {
            Status::Io
        };
        for site in ["READ", "VERIFY", "FORMAT scan"] {
            let context = format!("{site} {name}");
            let mut h = if site == "FORMAT scan" {
                H::blank()
            } else {
                let mut h = H::formatted();
                let e1 = h.export(1, &archive(1, 1, 700, 1));
                let kind = if site == "READ" {
                    wire::BIND_RESTORE
                } else {
                    wire::BIND_VERIFY
                };
                assert_eq!(h.bind(2, kind, e1), Status::Ok);
                h
            };
            let state = h.svc.state();
            let writes = h.disk.writes;
            once(&mut h, fault, |op, _| matches!(op, Op::Read(_)));
            let status = match site {
                "READ" => {
                    let mut read = req(wire::READ);
                    read.entry = 1;
                    read.aux = 10;
                    h.portal(2, read, &[]).status
                }
                "VERIFY" => h.verify_entry(2, 1),
                _ => h.control(req(wire::FORMAT), &[]).status,
            };
            assert_eq!(status, expected, "{context}");
            assert!(!h.svc.safe_path(), "{context}: no safe path");
            assert_eq!(h.svc.state(), state, "{context}: state unchanged");
            assert_eq!(h.disk.writes, writes, "{context}: nothing written");
        }
    }
}

/// N33 (writes and flushes): an APPEND write, the SEAL tail write and
/// barrier 1, for every broker result. `Io` for the submission results and
/// completion -12, `Unknown` for completion -13, -16 and -14; never
/// `NoDevice` or `Timeout`; the safe path starts in every case and the
/// writing entry is discarded.
/// Kills: a write whose completion -13 replies `Timeout`; a -14 that replies
/// `NoDevice`; a completion -14 or -16 answered `Io`.
#[test]
fn n33_device_results_writes() {
    // 1100 bytes: SEAL writes a partial last sector itself.
    let bytes = archive(2, 1, 1020, 9);
    for (name, fault) in n33_results() {
        let expected = match name {
            "complete -13" | "complete -16" | "complete -14" => Status::Unknown,
            _ => Status::Io,
        };
        for site in ["APPEND write", "SEAL tail write", "SEAL barrier 1"] {
            let context = format!("{site} {name}");
            let mut h = H::formatted();
            assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
            assert_eq!(h.bind(2, wire::BIND_EXPORT, 0), Status::Ok);
            let status = if site == "APPEND write" {
                let entry = wire::header(&h.create(1, b"x").reply.unwrap()).entry;
                once(&mut h, fault, |op, _| matches!(op, Op::Write(_)));
                let mut offset = 0;
                loop {
                    let outcome = h.append(1, entry, offset, &[1; 160]);
                    if outcome.status != Status::Ok {
                        break outcome.status;
                    }
                    offset += 160;
                }
            } else {
                let entry = h.write_all(1, &bytes);
                let tail = site == "SEAL tail write";
                once(&mut h, fault, move |op, _| {
                    if tail {
                        matches!(op, Op::Write(_))
                    } else {
                        op == Op::Flush
                    }
                });
                h.seal(1, entry, bytes.len() as u64).status
            };
            assert_eq!(status, expected, "{context}");
            assert!(h.svc.safe_path(), "{context}: safe path");
            assert_eq!(h.disk.counts.catalog_writes, 0, "{context}");
            assert_eq!(h.create(2, b"y").status, Status::Access, "{context}");
            assert_eq!(h.list_ids(0), (Status::Ok, vec![]), "{context}");
            assert_eq!(h.info().0, Status::Ok, "{context}: still mounted");
        }
    }
}

/// N33 / N14: kernel -14 to a mount, a remount or FORMAT is recorded for the
/// failed-mount state, which INFO reports as `NoDevice` -14, while the
/// request that met it replies per "Device results" (2.5); any other result
/// there reports `Io` -12.
/// Kills: a failed-mount state that loses the -14 at the boot mount (FLUSH
/// or read, submission or completion), at the remount after a SEAL catalog
/// failure, or at a FORMAT write or flush.
#[test]
fn n33_no_device_reaches_failed_mount_state() {
    let submit14 = Fault {
        submit: -14,
        ..Fault::default()
    };
    // Boot mount: FLUSH or a read of sectors 0-15, submission or completion.
    for (name, fault, read) in [
        ("flush submit", submit14, false),
        ("flush complete", Fault::code(-14), false),
        ("read submit", submit14, true),
        ("read complete", Fault::code(-14), true),
        ("flush complete -12", Fault::code(-12), false),
    ] {
        let mut disk = SimDisk::new(SECTORS);
        let mut fired = false;
        disk.fault(move |op, _| {
            let hit = !fired && (op == Op::Read(5) && read || op == Op::Flush && !read);
            fired |= hit;
            hit.then_some(fault)
        });
        let mut h = H::boot(disk);
        let no_device = fault.submit == -14 || fault.code == -14;
        assert_eq!(h.svc.state(), State::Failed { no_device }, "mount {name}");
        let info = if no_device {
            Status::NoDevice
        } else {
            Status::Io
        };
        assert_eq!(h.info().0, info, "mount {name}");
        assert_eq!(h.list_ids(0).0, Status::Io, "mount {name}");
    }
    // Remount after a SEAL catalog failure (barrier 2 -12): its FLUSH meets -14.
    for fault in [submit14, Fault::code(-14)] {
        let mut h = H::formatted();
        assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
        let bytes = archive(1, 1, 300, 1);
        let entry = h.write_all(1, &bytes);
        h.disk.arm();
        h.disk.fault(move |op, c| match (op, c.flushes) {
            (Op::Flush, 1) => Some(Fault::code(-12)),
            (Op::Flush, 2) => Some(fault),
            _ => None,
        });
        assert_eq!(h.seal(1, entry, bytes.len() as u64).status, Status::Unknown);
        assert_eq!(h.svc.state(), State::Failed { no_device: true });
        assert_eq!(h.info().0, Status::NoDevice);
        assert_eq!(h.list_ids(1), (Status::Unknown, vec![]));
    }
    // FORMAT: the second write of copy A, or barrier flush, meets -14.
    for (fault, reply) in [(submit14, Status::Io), (Fault::code(-14), Status::Unknown)] {
        for flush in [false, true] {
            let mut h = H::blank();
            once(&mut h, fault, move |op, c| {
                if flush {
                    op == Op::Flush
                } else {
                    matches!(op, Op::Write(_)) && c.writes == 1
                }
            });
            assert_eq!(h.control(req(wire::FORMAT), &[]).status, reply);
            assert_eq!(h.svc.state(), State::Failed { no_device: true });
            assert_eq!(h.info().0, Status::NoDevice);
        }
    }
}

/// Image after E1 with `stale` already in the free space at `next_free`, as
/// a discarded write leaves it (2.6), cold-booted. Returns the harness, E1
/// and the first free sector.
fn with_stale(stale: &[u8]) -> (H, u64, u64) {
    let mut h = H::formatted();
    let e1 = h.export(1, &archive(1, 2, 700, 3));
    let first = h.svc.catalog().next_free;
    let mut image = h.disk.durable.clone();
    for (index, chunk) in stale.chunks(SECTOR).enumerate() {
        image[first as usize + index][..chunk.len()].copy_from_slice(chunk);
    }
    (H::cold(image), e1, first)
}

/// Export `bytes` on lane 2; while its APPEND writes are completed but not
/// flushed, a READ or VERIFY on lane 3 completes with -13 (`Timeout`) and
/// resets the device. Returns the SEAL outcome.
fn seal_after_read_timeout(h: &mut H, e1: u64, bytes: &[u8], verify: bool) -> Outcome {
    assert_eq!(h.bind(2, wire::BIND_EXPORT, 0), Status::Ok);
    let kind = if verify {
        wire::BIND_VERIFY
    } else {
        wire::BIND_RESTORE
    };
    assert_eq!(h.bind(3, kind, e1), Status::Ok);
    let entry = h.write_all(2, bytes);
    assert!(!h.disk.cache.is_empty(), "unflushed APPEND writes");
    once(h, Fault::code(-13), |op, _| matches!(op, Op::Read(_)));
    let status = if verify {
        h.verify_entry(3, e1)
    } else {
        let mut read = req(wire::READ);
        read.entry = e1;
        read.aux = 10;
        h.portal(3, read, &[]).status
    };
    assert_eq!(status, Status::Timeout);
    assert!(!h.svc.safe_path(), "a read failure starts no safe path");
    h.disk.fault = None;
    h.disk.arm();
    h.seal(2, entry, bytes.len() as u64)
}

/// Every listed entry verifies.
fn listed_entries_verify(h: &mut H) {
    assert_eq!(h.unbind(3), Status::Ok);
    for id in h.list_ids(0).1 {
        assert_eq!(h.bind(3, wire::BIND_VERIFY, id), Status::Ok);
        assert_eq!(h.verify_entry(3, id), Status::Ok, "listed entry {id}");
        assert_eq!(h.unbind(3), Status::Ok);
    }
}

/// N33 (review 4 OP-6): a READ or VERIFY times out and resets the device while
/// an export lane holds completed, unflushed APPEND writes, under both reset
/// models. The free sectors held different bytes before (a discarded,
/// non-archive write of the same length), so a stale extent cannot pass the
/// readback by accident. Cache kept: SEAL `Ok` and the entry verifies. Cache
/// dropped: SEAL `Corrupt` -18 at the step-3 readback, nothing listed, no
/// catalog write. Both: never a listed entry that fails VERIFY.
/// Kills: a SEAL that seals without the step-3 readback (the dropping model
/// lists an entry that fails VERIFY).
#[test]
fn n33_read_timeout_with_unflushed_append() {
    both_reset_models(|drops| {
        for verify in [false, true] {
            let context = format!("drops {drops} verify {verify}");
            let bytes = archive(2, 1, 1020, 5);
            let mut stale = bytes.clone();
            stale[100] ^= 0x5a;
            let (mut h, e1, first) = with_stale(&stale);
            assert_ne!(h.disk.durable[first as usize][..], bytes[..SECTOR]);
            let sealed = seal_after_read_timeout(&mut h, e1, &bytes, verify);
            assert_eq!(drops, h.disk.dropped > 0, "{context}: model exercised");
            if drops {
                assert_eq!(sealed.status, Status::Corrupt, "{context}");
                assert_eq!(h.disk.counts.catalog_writes, 0, "{context}");
                assert_eq!(h.list_ids(0).1, vec![e1], "{context}");
            } else {
                assert_eq!(sealed.status, Status::Ok, "{context}");
                assert_eq!(h.list_ids(0).1.len(), 2, "{context}");
            }
            assert!(!h.svc.safe_path(), "{context}");
            listed_entries_verify(&mut h);
        }
    });
}

fn trailer(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(bytes[bytes.len() - 8..].try_into().unwrap())
}

/// N37 under O17 (a): SEAL `Corrupt` -18, no catalog write, nothing listed
/// but E1, the writing entry discarded, no safe path, and a later CREATE on
/// a new binding of `lane` runs.
fn refused_without_latch(h: &mut H, sealed: &Outcome, e1: u64, lane: usize, context: &str) {
    assert_eq!(sealed.status, Status::Corrupt, "{context}");
    assert_eq!(
        h.disk.counts.catalog_writes, 0,
        "{context}: no catalog write"
    );
    assert_eq!(h.list_ids(0).1, vec![e1], "{context}: nothing listed");
    assert_eq!(h.info().0, Status::Ok, "{context}: INFO");
    assert_eq!(
        h.svc.binding(lane).unwrap().export,
        Export::NotSealed,
        "{context}: discarded"
    );
    assert!(!h.svc.safe_path(), "{context}: no safe path");
    assert_eq!(h.unbind(lane), Status::Ok);
    assert_eq!(h.bind(lane, wire::BIND_EXPORT, 0), Status::Ok);
    assert_eq!(
        h.create(lane, b"next").status,
        Status::Ok,
        "{context}: CREATE"
    );
}

/// SEAL `Ok` and its `aux` is the trailer of the archive the portal sent.
fn sealed_as_sent(h: &mut H, sealed: &Outcome, bytes: &[u8], context: &str) {
    assert_eq!(sealed.status, Status::Ok, "{context}");
    let aux = wire::header(&sealed.reply.unwrap()).aux;
    assert_eq!(aux, trailer(bytes), "{context}: aux is the sent trailer");
    assert_eq!(h.list_ids(0).1.len(), 2, "{context}");
    assert!(!h.svc.safe_path(), "{context}");
}

/// N33 (iii) and N37 (a), read-timeout variant (O17 (a)): the free space
/// holds a *valid* archive of exactly the same, sector-aligned length (no
/// tail write at SEAL), and a READ or VERIFY on lane 3 times out and resets
/// the device while lane 2's APPEND writes are unflushed. Cache dropped: the
/// readback passes `archive::verify` over the stale archive, and only the
/// content check refuses it (`Corrupt`, nothing listed, no catalog write,
/// no safe path, a later CREATE runs). Cache kept: SEAL `Ok`, `aux` equals
/// the sent trailer, the entry verifies.
/// Kills: a SEAL without the content check (the cache-dropped model lists
/// the stale archive, the earlier recorded outcome).
#[test]
fn n37a_read_timeout_over_stale_valid_archive() {
    both_reset_models(|drops| {
        for verify in [false, true] {
            let context = format!("drops {drops} verify {verify}");
            let bytes = archive(2, 1, 944, 5);
            let stale = archive(7, 3, 944, 77);
            assert_eq!((bytes.len(), stale.len()), (1024, 1024));
            let (mut h, e1, _) = with_stale(&stale);
            let sealed = seal_after_read_timeout(&mut h, e1, &bytes, verify);
            assert_eq!(drops, h.disk.dropped > 0, "{context}: model exercised");
            if drops {
                refused_without_latch(&mut h, &sealed, e1, 2, &context);
            } else {
                sealed_as_sent(&mut h, &sealed, &bytes, &context);
            }
            listed_entries_verify(&mut h);
        }
    });
}

/// N37 (a), barrier-1 variant (as N26 (d)): disk 1's own reset during the
/// barrier-1 submission, over a stale valid archive of the same length,
/// here not sector-aligned (under the dropping model every unflushed write
/// of the entry, the SEAL tail included, is lost). Outcomes as above.
/// Kills: a SEAL without the content check.
#[test]
fn n37a_own_reset_at_barrier1_over_stale_valid_archive() {
    both_reset_models(|drops| {
        let context = format!("drops {drops}");
        let bytes = archive(2, 1, 1000, 5);
        let stale = archive(7, 3, 1000, 77);
        assert_eq!((bytes.len(), stale.len()), (1080, 1080));
        let (mut h, e1, _) = with_stale(&stale);
        assert_eq!(h.bind(2, wire::BIND_EXPORT, 0), Status::Ok);
        let entry = h.write_all(2, &bytes);
        h.disk.arm();
        h.disk.fault(|op, c| {
            (op == Op::Flush && c.flushes == 0).then(|| Fault {
                again: Some(Some(RESET_TIMEOUT)),
                ..Fault::default()
            })
        });
        let sealed = h.seal(2, entry, bytes.len() as u64);
        h.disk.fault = None;
        assert_eq!(drops, h.disk.dropped > 0, "{context}: model exercised");
        if drops {
            refused_without_latch(&mut h, &sealed, e1, 2, &context);
        } else {
            sealed_as_sent(&mut h, &sealed, &bytes, &context);
        }
        listed_entries_verify(&mut h);
    });
}

/// N37 (b): without a reset, the disk reports success for every write of
/// the entry and for barrier 1 but stores none of them, so the stale valid
/// archive stays whole: SEAL `Corrupt`, nothing listed, no catalog write,
/// no safe path, a later CREATE runs. No reset occurs, so both models agree.
/// Kills: a SEAL without the content check (it lists the stale archive).
#[test]
fn n37b_unstored_entry_writes_over_stale_valid_archive() {
    both_reset_models(|drops| {
        let context = format!("drops {drops}");
        let bytes = archive(2, 1, 1000, 5);
        let stale = archive(7, 3, 1000, 77);
        let (mut h, e1, first) = with_stale(&stale);
        assert_eq!(h.bind(2, wire::BIND_EXPORT, 0), Status::Ok);
        h.disk.arm();
        h.disk.fault(|op, _| {
            matches!(op, Op::Write(sector) if sector >= DATA_START).then(|| Fault {
                silent: true,
                ..Fault::default()
            })
        });
        let entry = h.write_all(2, &bytes);
        let sealed = h.seal(2, entry, bytes.len() as u64);
        h.disk.fault = None;
        assert_eq!(h.disk.counts.entry_writes, 3, "{context}: writes answered");
        let record = Record {
            first,
            length: bytes.len() as u64,
            ..Record::EMPTY
        };
        assert_eq!(extent(&h.disk.visible(), &record), stale, "{context}");
        assert_eq!(h.disk.dropped, 0, "{context}: no reset");
        refused_without_latch(&mut h, &sealed, e1, 2, &context);
        listed_entries_verify(&mut h);
    });
}

/// O17 (a): the content digest covers only APPENDs answered `Ok`; a refused
/// APPEND (`Stale`, `Full`) appends nothing, so SEAL still matches.
/// Kills: a digest updated before the offset check or before the scope-end
/// check (SEAL `Corrupt`).
#[test]
fn n37_digest_covers_only_appends_answered_ok() {
    let mut h = H::boot(SimDisk::new(24));
    assert_eq!(h.control(req(wire::FORMAT), &[]).status, Status::Ok);
    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    // 4080 bytes: the eight free sectors; one more APPEND of 160 is `Full`.
    let bytes = archive(1, 1, 4000, 1);
    let entry = wire::header(&h.create(1, b"x").reply.unwrap()).entry;
    let (head, rest) = bytes.split_at(4000);
    for (index, chunk) in head.chunks(wire::DATA).enumerate() {
        let offset = (index * wire::DATA) as u64;
        assert_eq!(h.append(1, entry, offset, chunk).status, Status::Ok);
        if index == 3 {
            assert_eq!(h.append(1, entry, 0, &[9; 160]).status, Status::Stale);
        }
    }
    assert_eq!(h.append(1, entry, 4000, &[9; 160]).status, Status::Full);
    assert_eq!(h.append(1, entry, 4000, rest).status, Status::Ok);
    let sealed = h.seal(1, entry, bytes.len() as u64);
    assert_eq!(sealed.status, Status::Ok);
    assert_eq!(wire::header(&sealed.reply.unwrap()).aux, trailer(&bytes));
}

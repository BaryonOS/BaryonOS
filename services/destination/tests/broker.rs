//! Kernel -8 and the service's own wait bounds (spec 2.7): N26.
//! `SimDisk` advances its clock by one tick per submission and per poll.
mod common;

use common::*;
use destination_core::{
    device::{POLL_BOUND, RESET_TIMEOUT, SUBMIT_BOUND, TIMEOUT},
    service::State,
    wire, Status,
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
}

/// N26 (b): -8 forever from the barrier-2 flush on: SEAL `Unknown` after one
/// bound, the remount FLUSH exceeds the bound too: failed mount.
#[test]
fn n26b_barrier2_again_forever() {
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
}

/// N26 (c): -8 forever to an APPEND write: APPEND `Io` after one bound,
/// writing entry discarded, safe path.
#[test]
fn n26c_append_write_again_forever() {
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
}

/// N26 (d): -8 on submission for `RESET_TIMEOUT` ticks (disk 1's own reset)
/// and a poll pending for `TIMEOUT` - 1 ticks both complete normally.
/// Kills: a bound shorter than one legitimate reset; the submission bound
/// applied to the pending poll.
#[test]
fn n26d_legitimate_waits_complete() {
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
    assert_eq!(h.seal(1, entry, bytes.len() as u64).status, Status::Ok);
    assert!(!h.svc.safe_path());
    assert_eq!(h.list_ids(0).1, vec![entry]);
}

/// N26 (e): a completion poll pending beyond `TIMEOUT` + 2 x `RESET_TIMEOUT`
/// on a write counts as `Unknown` and starts the safe path.
/// Kills: a poll loop without a bound (the call budget panics).
#[test]
fn n26e_poll_beyond_bound_is_unknown() {
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
}

/// Kernel -14 and -13 on submission of a destination write (5.3): `NoDevice`
/// and `Io` replies; both start the safe path.
#[test]
fn kernel_offline_on_write() {
    for (code, reply) in [(-14, Status::NoDevice), (-13, Status::Io)] {
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
        assert_eq!(status, reply);
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
}

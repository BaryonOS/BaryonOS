//! Flush order and cache loss (spec 2.7, section 6): N11, N12, N13, N24, N25;
//! remount against the committed catalog (O16): N35.
//! N13 and N24 pass through a device reset and run under both reset models
//! (cache kept, cache dropped; section 6, revision 3).
//! Flush-order and cache-loss acceptance is host-only (section 6); every
//! image is built from the durable sectors plus a subset of the unflushed
//! cache, then mounted by a new instance (a cold boot).
mod common;

use common::*;
use destination_core::{
    catalog::{self, Catalog, Record},
    device::{Broker, Request},
    service::{Env, Export, Meta, Outcome, State},
    wire::{self, Header},
    Status, SECTOR,
};

/// Cold boot of `image`: it must mount, list `E1` or `E1, E2`, and every
/// listed entry must be byte-identical to what was exported and verify.
fn check_cold(image: Vec<[u8; SECTOR]>, exported: &[(u64, Vec<u8>)], context: &str) -> Vec<u64> {
    let mut h = H::cold(image.clone());
    assert_eq!(h.svc.state(), State::Mounted, "{context}: cold boot mounts");
    let (status, ids) = h.list_ids(0);
    assert_eq!(status, Status::Ok);
    let e1 = exported[0].0;
    assert!(
        ids == vec![e1] || ids == exported.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        "{context}: LIST {ids:?}"
    );
    for (id, bytes) in exported.iter().filter(|(id, _)| ids.contains(id)) {
        let record = *h.svc.catalog().find(*id).unwrap();
        assert_eq!(
            &extent(&image, &record),
            bytes,
            "{context}: entry {id} byte-identical"
        );
        assert_eq!(h.bind(3, wire::BIND_VERIFY, *id), Status::Ok);
        assert_eq!(
            h.verify_entry(3, *id),
            Status::Ok,
            "{context}: entry {id} verifies"
        );
        assert_eq!(h.unbind(3), Status::Ok);
    }
    ids
}

fn with_e1() -> (H, u64, Vec<u8>) {
    let mut h = H::formatted();
    let bytes = archive(1, 2, 700, 3);
    let e1 = h.export(1, &bytes);
    (h, e1, bytes)
}

/// N11: power cut after k destination writes during APPEND, cache loss.
/// Expected after cold boot: LIST = {E1}, E1 byte-identical.
/// Kills: an APPEND that writes into E1's extent (byte identity after cold
/// boot) or the catalog area (every submitted write lies at or beyond the
/// committed `next_free`; one damaged catalog copy alone would still mount,
/// see also `n4_export_never_touches_sealed_extent`).
#[test]
fn n11_power_cut_during_append() {
    let e2_bytes = archive(2, 1, 1800, 7);
    for k in 0..=4 {
        let (mut h, e1, e1_bytes) = with_e1();
        assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
        let next_free = h.svc.catalog().next_free;
        let logged = h.disk.log.len();
        h.disk.arm();
        h.disk.crash_at = Some(k);
        let entry = wire::header(&h.create(1, b"two").reply.unwrap()).entry;
        for (index, chunk) in e2_bytes.chunks(wire::DATA).enumerate() {
            if h.append(1, entry, (index * wire::DATA) as u64, chunk)
                .status
                != Status::Ok
            {
                break;
            }
        }
        for event in &h.disk.log[logged..] {
            if let Event::Submit(Op::Write(sector)) = event {
                assert!(*sector >= next_free, "N11 k={k}: write to {sector}");
            }
        }
        for image in h.disk.crash_images() {
            let ids = check_cold(image, &[(e1, e1_bytes.clone())], &format!("N11 k={k}"));
            assert_eq!(ids, vec![e1]);
        }
    }
}

/// N12: power cut at every mutating request of SEAL (last-sector write,
/// barrier 1, the eight catalog writes, barriers 2 and 3), with every subset
/// of the unflushed cache surviving.
/// Expected: LIST = {E1} or {E1, E2}; a listed E2 verifies; never a listed E2
/// that fails, never `Corrupt`.
/// Kills: a SEAL without barrier 1 (E2 listed while its data sectors are
/// lost) or without barrier 2 (both copies torn: `Corrupt`).
#[test]
fn n12_power_cut_during_seal() {
    // 1100 bytes: two full sectors written by APPEND and a partial last one.
    let e2_bytes = archive(2, 1, 1020, 9);
    let mut seen = [false; 2];
    for k in 0..12 {
        let (mut h, e1, e1_bytes) = with_e1();
        assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
        let entry = h.write_all(1, &e2_bytes);
        h.disk.arm();
        h.disk.crash_at = Some(k);
        let status = h.seal(1, entry, e2_bytes.len() as u64).status;
        let exported = [(e1, e1_bytes.clone()), (entry, e2_bytes.clone())];
        for image in h.disk.crash_images() {
            let ids = check_cold(image, &exported, &format!("N12 k={k} ({status:?})"));
            seen[ids.len() - 1] = true;
        }
    }
    assert_eq!(seen, [true, true], "both outcomes occur");
}

/// N25: power cut during FORMAT at every write prefix of copy A, every
/// subset of the cache surviving, before copy B.
/// Kills: a FORMAT failure that leaves the instance a blank candidate (a
/// repeated FORMAT writes again), see `n25_format_write_failure`.
#[test]
fn n25_power_cut_during_format() {
    for k in 0..=4 {
        let mut h = H::blank();
        h.disk.arm();
        h.disk.crash_at = Some(k);
        h.control(req(wire::FORMAT), &[]);
        let cache: Vec<u64> = h.disk.cache.iter().map(|(s, _)| *s).collect();
        for mask in 0u32..(1 << cache.len()) {
            let image = h.disk.crash_subset(|index| mask & (1 << index) != 0);
            let kept = |sector: u64| {
                cache
                    .iter()
                    .enumerate()
                    .any(|(index, s)| *s == sector && mask & (1 << index) != 0)
            };
            let mut cold = H::cold(image);
            match (kept(0), kept(3)) {
                (false, false) => {
                    assert_eq!(cold.svc.state(), State::Blank, "k={k} mask={mask}");
                    assert_eq!(cold.control(req(wire::FORMAT), &[]).status, Status::Ok);
                }
                (true, true) => {
                    assert_eq!(cold.svc.state(), State::Mounted);
                    let catalog = cold.svc.catalog();
                    assert_eq!((catalog.sequence, catalog.count), (1, 0));
                }
                _ => {
                    assert_eq!(cold.svc.state(), State::Corrupt, "k={k} mask={mask}");
                    assert_eq!(cold.info().0, Status::Corrupt);
                    assert_eq!(cold.control(req(wire::FORMAT), &[]).status, Status::Corrupt);
                    assert_eq!(cold.disk.writes, 0);
                    // External recovery: the owner re-zeroes the whole scope.
                    let mut again = H::blank();
                    assert_eq!(again.control(req(wire::FORMAT), &[]).status, Status::Ok);
                }
            }
        }
    }
}

#[test]
fn n25_format_write_failure() {
    for (code, reply) in [(-12, Status::Io), (-16, Status::Unknown)] {
        for at in 0..10 {
            let mut h = H::blank();
            h.disk.arm();
            // Mutating requests of FORMAT: 4 writes, flush, 4 writes, flush.
            h.disk
                .fault(move |op, c| (op.mutating() && c.mutating == at).then(|| Fault::code(code)));
            assert_eq!(
                h.control(req(wire::FORMAT), &[]).status,
                reply,
                "code {code} at {at}"
            );
            assert_eq!(h.svc.state(), State::Failed { no_device: false });
            h.disk.fault = None;
            let mutations = h.mutations();
            assert_eq!(h.info().0, Status::Io);
            assert_eq!(h.control(req(wire::FORMAT), &[]).status, Status::Io);
            assert_eq!(h.mutations(), mutations, "no further device request");
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Point {
    Step4(usize),
    Barrier2,
    Step5(usize),
    Barrier3,
}

/// Lanes for N13: X export (the SEAL), Y export (bound, no CREATE), lane 3
/// verify or restore of E1. Revision 2 lists four bound operation lanes but
/// 2.8 allows three, so the verify and restore lanes run in separate passes.
fn n13_case(point: Point, code: i64, lands: bool, remount_ok: bool, restore: bool) -> usize {
    let context =
        format!("{point:?} code {code} lands {lands} remount {remount_ok} restore {restore}");
    let (mut h, e1, e1_bytes) = with_e1();
    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    assert_eq!(h.bind(2, wire::BIND_EXPORT, 0), Status::Ok);
    let third = if restore {
        wire::BIND_RESTORE
    } else {
        wire::BIND_VERIFY
    };
    assert_eq!(h.bind(3, third, e1), Status::Ok);
    let e2_bytes = archive(2, 1, 1020, 5);
    let e2 = h.write_all(1, &e2_bytes);
    h.disk.arm();
    let mut fired = false;
    h.disk.fault(move |op, c| {
        let hit = match (point, op) {
            (Point::Step4(i), Op::Write(s)) => s < 8 && c.catalog_writes == i,
            (Point::Step5(i), Op::Write(s)) => s < 8 && c.catalog_writes == 4 + i,
            (Point::Barrier2, Op::Flush) => c.flushes == 1,
            (Point::Barrier3, Op::Flush) => c.flushes == 2,
            _ => false,
        };
        if hit && !fired {
            fired = true;
            return Some(Fault {
                code,
                lands,
                ..Fault::default()
            });
        }
        // The first flush after the fault is the remount FLUSH.
        (fired && op == Op::Flush && !remount_ok).then(|| Fault::code(-12))
    });
    assert_eq!(
        h.seal(1, e2, e2_bytes.len() as u64).status,
        Status::Unknown,
        "{context}"
    );
    assert!(h.svc.safe_path());
    h.disk.fault = None;
    let mutations = h.mutations();
    let x_list = h.list_on(1, PORTAL);
    let create = |h: &mut H, lane| h.create(lane, b"n").status;
    assert_eq!(
        create(&mut h, 1),
        Status::Access,
        "{context}: X CREATE (check 6)"
    );
    assert_eq!(
        h.append(1, e2, e2_bytes.len() as u64, b"x").status,
        Status::Access
    );
    assert_eq!(h.seal(1, e2, e2_bytes.len() as u64).status, Status::Access);
    let mut abort = req(wire::ABORT);
    abort.entry = e2;
    assert_eq!(
        h.portal(1, abort, &[]).status,
        Status::Access,
        "{context}: X ABORT"
    );
    assert_eq!(
        create(&mut h, 3),
        Status::Access,
        "{context}: lane 3 CREATE (check 6)"
    );
    let third_op = if restore {
        h.read_entry(3, e1, e1_bytes.len() as u64)
            .map(|_| ())
            .err()
            .unwrap_or(Status::Ok)
    } else {
        h.verify_entry(3, e1)
    };
    let mut bind = req(wire::BIND);
    bind.count = 2;
    bind.name_length = wire::BIND_EXPORT;
    bind.aux = 100;
    if remount_ok {
        assert_eq!(h.svc.state(), State::Mounted, "{context}");
        let cold = H::cold(h.disk.durable.clone())
            .svc
            .catalog()
            .find(e2)
            .is_some();
        let sealed = h.svc.catalog().find(e2).is_some();
        assert_eq!(
            sealed, cold,
            "{context}: resolution matches the cold-boot catalog"
        );
        let expected: Vec<u64> = if sealed { vec![e2] } else { vec![] };
        assert_eq!(x_list.0, Status::Ok);
        assert_eq!(
            x_list.1.iter().map(|r| r.id).collect::<Vec<_>>(),
            expected,
            "{context}"
        );
        assert_eq!(
            create(&mut h, 2),
            Status::Access,
            "{context}: Y CREATE (safe path)"
        );
        assert_eq!(third_op, Status::Ok, "{context}: lane 3 operation runs");
        assert_eq!(h.list_ids(3), (Status::Ok, vec![e1]));
        assert_eq!(h.info().0, Status::Ok);
        let mut ids = vec![e1];
        if sealed {
            ids.push(e2);
        }
        assert_eq!(h.list_ids(0), (Status::Ok, ids));
        assert_eq!(h.control(bind, &TEST_ID).status, Status::Busy, "BIND runs");
        if matches!(point, Point::Step5(_) | Point::Barrier3) {
            assert!(
                sealed,
                "{context}: barrier 2 completed, the target copy is durable"
            );
        }
        if point == Point::Step4(0) && !lands {
            assert!(!sealed, "{context}");
        }
    } else {
        assert_eq!(
            h.svc.state(),
            State::Failed { no_device: false },
            "{context}"
        );
        assert_eq!(x_list.0, Status::Unknown, "{context}: X LIST");
        assert!(x_list.1.is_empty());
        assert_eq!(create(&mut h, 2), Status::Io, "{context}: Y CREATE");
        assert_eq!(third_op, Status::Io);
        assert_eq!(h.list_ids(3).0, Status::Io);
        assert_eq!(h.info().0, Status::Io);
        assert_eq!(h.list_ids(0).0, Status::Io);
        assert_eq!(h.control(bind, &TEST_ID).status, Status::Io);
        // Next boot: only that boot's catalog decides.
        check_cold(
            h.disk.visible(),
            &[(e1, e1_bytes.clone()), (e2, e2_bytes.clone())],
            &context,
        );
    }
    assert_eq!(
        h.mutations(),
        mutations,
        "{context}: no device write or FLUSH retry"
    );
    h.disk.dropped
}

/// N13: `Unknown` at SEAL for every device result at steps 4-5.
/// Kills: a safe-path trigger that fires only on `Unknown` (Y's CREATE runs
/// after an `Io` at barrier 2); check 7 before check 6 (X's CREATE returns
/// `Io` in the failed-mount state); a binding limit applied only after
/// resolution.
#[test]
fn n13_unknown_at_seal() {
    let points = [
        Point::Step4(0),
        Point::Step4(3),
        Point::Barrier2,
        Point::Step5(0),
        Point::Step5(3),
        Point::Barrier3,
    ];
    both_reset_models(|drops| {
        let mut dropped = 0;
        for point in points {
            for code in [-12, -13, -16] {
                for lands in [false, true] {
                    for remount_ok in [true, false] {
                        for restore in [false, true] {
                            dropped += n13_case(point, code, lands, remount_ok, restore);
                        }
                    }
                }
            }
        }
        assert_eq!(drops, dropped > 0, "the dropping model lost cached writes");
    });
}

/// SEAL whose step-4 write of sector `catalog_write` times out and lands
/// late in `mode`. Returns the harness after SEAL and the IDs.
fn n24_seal(catalog_write: usize, mode: Late) -> (H, u64, Vec<u8>, u64, Vec<u8>) {
    let (mut h, e1, e1_bytes) = with_e1();
    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    assert_eq!(h.bind(2, wire::BIND_EXPORT, 0), Status::Ok);
    let e2_bytes = archive(2, 1, 1020, 5);
    let e2 = h.write_all(1, &e2_bytes);
    h.disk.arm();
    h.disk.fault(move |op, c| {
        (matches!(op, Op::Write(s) if s < 8) && c.catalog_writes == catalog_write).then_some(
            Fault {
                code: -16,
                late: Some(mode),
                ..Fault::default()
            },
        )
    });
    h.disk.log.clear();
    assert_eq!(h.seal(1, e2, e2_bytes.len() as u64).status, Status::Unknown);
    h.disk.fault = None;
    (h, e1, e1_bytes, e2, e2_bytes)
}

fn position(log: &[Event], wanted: impl Fn(&Event) -> bool) -> usize {
    log.iter().position(wanted).expect("event present")
}

/// N24 (a): the timed-out catalog write lands before the fence (reset
/// acknowledgement). No remount FLUSH, no resolution and no request before
/// the fence; afterwards CREATE is `Access` and the export lane's LIST
/// matches the cold-boot catalog.
/// Kills: a remount FLUSH issued, or a request received, before the fence
/// (seen in the event order: FLUSH or SEAL's reply before the late landing).
#[test]
fn n24a_late_write_before_fence() {
    both_reset_models(|drops| {
        let mut dropped = 0;
        for write in [0, 3] {
            let (mut h, _, _, e2, _) = n24_seal(write, Late::AtResetAck);
            dropped += h.disk.dropped;
            let log = h.disk.log.clone();
            let timeout = position(&log, |e| matches!(e, Event::Complete(Op::Write(_), -16)));
            let ack = position(&log, |e| *e == Event::ResetAck);
            let landed = position(&log, |e| matches!(e, Event::LateLand(_)));
            let remount = timeout + position(&log[timeout..], |e| *e == Event::Submit(Op::Flush));
            let handled = position(&log, |e| matches!(e, Event::Handled(wire::SEAL, _)));
            assert!(
                timeout < ack && ack < landed && landed < remount && remount < handled,
                "{log:?}"
            );
            assert!(!log[timeout..remount]
                .iter()
                .any(|e| matches!(e, Event::Submit(_))));
            assert_eq!(h.create(2, b"y").status, Status::Access);
            let listed = !h.list_on(1, PORTAL).1.is_empty();
            let cold = H::cold(h.disk.durable.clone())
                .svc
                .catalog()
                .find(e2)
                .is_some();
            assert_eq!(listed, cold, "resolution matches the cold-boot catalog");
        }
        assert_eq!(drops, dropped > 0, "the dropping model lost cached writes");
    });
}

/// N24 (b): the late write lands after the remount FLUSH. Outside the
/// assumed device model (O9): recorded, not a pass. Required only: no sealed
/// extent written, never a listed entry that fails VERIFY, and LIST after a
/// cold boot {E1} or {E1, E2} (or `Corrupt` with E1 byte-identical).
#[test]
fn n24b_late_write_after_remount_flush_recorded() {
    both_reset_models(|drops| {
        let mut dropped = 0;
        for write in [0, 3] {
            let (h, e1, e1_bytes, e2, e2_bytes) = n24_seal(write, Late::AfterNextFlush);
            dropped += h.disk.dropped;
            let resolved = h.svc.binding(1).unwrap().export;
            let e1_record = *H::cold(h.disk.durable.clone())
                .svc
                .catalog()
                .find(e1)
                .unwrap();
            for image in [h.disk.durable.clone(), h.disk.visible()] {
                assert_eq!(extent(&image, &e1_record), e1_bytes);
                let cold = H::cold(image.clone());
                if cold.svc.state() == State::Corrupt {
                    eprintln!("N24(b) write {write}: Corrupt after cold boot (fail closed)");
                    continue;
                }
                let ids = check_cold(
                    image,
                    &[(e1, e1_bytes.clone()), (e2, e2_bytes.clone())],
                    "N24(b)",
                );
                eprintln!(
                    "N24(b) write {write}: in-boot {resolved:?}, cold boot {ids:?} (recorded)"
                );
            }
        }
        assert_eq!(drops, dropped > 0, "the dropping model lost cached writes");
    });
}

/// N24 (c): after a remount that resolved E2 as not sealed, CREATE and APPEND
/// on another lane are refused before the late write lands, so it lands on
/// no other entry's data. (The cache is empty at this reset, so the two
/// reset models do not differ here.)
/// Kills: admitting CREATE after the resolution in the same boot.
#[test]
fn n24c_no_reuse_after_not_sealed() {
    both_reset_models(|_| {
        let (mut h, _, _, e2, _) = n24_seal(0, Late::Manual);
        assert_eq!(h.svc.catalog().find(e2), None, "resolved as not sealed");
        let writes = h.disk.writes;
        assert_eq!(h.create(2, b"y").status, Status::Access);
        assert_eq!(h.append(2, e2, 0, b"y").status, Status::Access);
        assert_eq!(h.disk.writes, writes, "no device write");
        assert!(h.disk.has_late());
        h.disk.land_late();
        let next_free = h.svc.catalog().next_free;
        for (sector, _) in &h.disk.cache {
            assert!(*sector < 8 || *sector >= next_free);
        }
    });
}

/// N24 (d): an entry write during APPEND times out and completes late, or
/// fails with `Io` while its data still reaches the medium.
/// Kills: a safe-path trigger that ignores a device `Io` on an entry write,
/// or admits CREATE after the reset in the same boot.
#[test]
fn n24d_entry_write_failure() {
    both_reset_models(|drops| {
        for (code, late, reply) in [
            (-16, Some(Late::Manual), Status::Unknown),
            (-12, None, Status::Io),
        ] {
            let (mut h, _, _) = with_e1();
            let next_free = h.svc.catalog().next_free;
            assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
            assert_eq!(h.bind(2, wire::BIND_EXPORT, 0), Status::Ok);
            let entry = wire::header(&h.create(1, b"x").reply.unwrap()).entry;
            // The second entry write fails: the first is cached and unflushed,
            // so the dropping reset model has something to lose at a timeout.
            h.disk.arm();
            h.disk.fault(move |op, c| {
                (matches!(op, Op::Write(_)) && c.writes >= 1).then_some(Fault {
                    code,
                    lands: true,
                    late,
                    ..Fault::default()
                })
            });
            let chunk = [3u8; 160];
            let mut offset = 0;
            let status = loop {
                let outcome = h.append(1, entry, offset, &chunk);
                if outcome.status != Status::Ok {
                    break outcome.status;
                }
                offset += 160;
            };
            assert_eq!(status, reply);
            h.disk.fault = None;
            assert!(h.svc.safe_path());
            assert_eq!(
                h.append(1, entry, offset, &chunk).status,
                Status::Access,
                "entry discarded"
            );
            assert_eq!(h.create(2, b"y").status, Status::Access);
            h.disk.land_late();
            for (sector, _) in &h.disk.cache {
                assert!(
                    *sector >= next_free,
                    "late entry write only in unallocated sectors"
                );
            }
            assert_eq!(h.list_ids(0).1.len(), 1);
            assert_eq!(drops && code == -16, h.disk.dropped > 0, "model exercised");
        }
    });
}

/// O1 (review R2): a device failure on the SEAL last-sector write or on
/// barrier 1 is a safe-path trigger (2.7): SEAL answers the device result,
/// the writing entry is discarded, nothing is listed and CREATE on another
/// bound export lane is `Access` for the rest of the boot.
/// Kills: `device_failure` replaced by `discard` (no latch) at either site.
#[test]
fn o1_seal_step3_failure_latches_safe_path() {
    // 1100 bytes: the last sector is partial, so SEAL writes it itself.
    let bytes = archive(2, 1, 1020, 9);
    assert_ne!(bytes.len() % SECTOR, 0);
    for (site, is_tail) in [("tail write", true), ("barrier 1", false)] {
        for (code, reply) in [(-12, Status::Io), (-16, Status::Unknown)] {
            let context = format!("{site} {code}");
            let (mut h, e1, _) = with_e1();
            assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
            assert_eq!(h.bind(2, wire::BIND_EXPORT, 0), Status::Ok);
            let entry = h.write_all(1, &bytes);
            h.disk.arm();
            h.disk.fault(move |op, c| {
                let hit = if is_tail {
                    matches!(op, Op::Write(_)) && c.writes == 0
                } else {
                    op == Op::Flush && c.flushes == 0
                };
                hit.then(|| Fault::code(code))
            });
            assert_eq!(
                h.seal(1, entry, bytes.len() as u64).status,
                reply,
                "{context}"
            );
            h.disk.fault = None;
            assert_eq!(h.disk.counts.catalog_writes, 0, "{context}");
            assert_eq!(
                h.seal(1, entry, bytes.len() as u64).status,
                Status::Access,
                "{context}: writing entry discarded"
            );
            assert_eq!(h.create(2, b"y").status, Status::Access, "{context}");
            assert!(h.svc.safe_path(), "{context}: latched");
            assert_eq!(h.list_ids(0).1, vec![e1], "{context}");
        }
    }
}

/// Wraps the disk as a device whose sectors 0-15 read as `area` once a write
/// or FLUSH has failed: the remount after a SEAL catalog failure then reads
/// a layout that contradicts the committed catalog (a changed device, O16).
struct Swapped<'a> {
    disk: &'a mut SimDisk,
    area: &'a [[u8; SECTOR]],
    swapped: &'a mut bool,
    last: Option<Op>,
}

impl Broker for Swapped<'_> {
    fn submit(&mut self, request: Request<'_>) -> i64 {
        self.last = Some(match request {
            Request::Read(sector) => Op::Read(sector),
            Request::Write(sector, _) => Op::Write(sector),
            Request::Flush => Op::Flush,
        });
        self.disk.submit(request)
    }
    fn poll(&mut self, buffer: &mut [u8; SECTOR]) -> i64 {
        let code = self.disk.poll(buffer);
        match self.last {
            Some(Op::Read(sector)) if code == 0 && *self.swapped && sector < 16 => {
                *buffer = self.area[sector as usize];
            }
            Some(op) if op.mutating() && code != 0 && code != -8 => *self.swapped = true,
            _ => {}
        }
        code
    }
    fn now(&mut self) -> u64 {
        self.disk.now()
    }
}

fn via(
    h: &mut H,
    area: &[[u8; SECTOR]],
    swapped: &mut bool,
    lane: usize,
    sender: u64,
    header: Header,
) -> Outcome {
    let mut header = header;
    h.next_id[lane] += 1;
    header.request_id = h.next_id[lane];
    header.epoch = h.generation[lane];
    let body = wire::encode(&header, &[]);
    let mut broker = Swapped {
        disk: &mut h.disk,
        area,
        swapped,
        last: None,
    };
    let mut env = Env {
        broker: &mut broker,
        verifier: &mut h.verifier,
        entropy: &mut h.entropy,
    };
    let meta = Meta {
        lane,
        sender_pid: sender,
        cap_count: 0,
    };
    h.svc.handle(meta, &body, &mut env)
}

/// E1 sealed; export lane 1 has written E2 and is about to SEAL; verify lane
/// 2 is bound to E1. Returns the harness, E1, E2's provisional ID and length.
fn n35_setup() -> (H, u64, u64, u64) {
    let (mut h, e1, _) = with_e1();
    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    assert_eq!(h.bind(2, wire::BIND_VERIFY, e1), Status::Ok);
    let bytes = archive(2, 1, 300, 2);
    let entry = h.write_all(1, &bytes);
    (h, e1, entry, bytes.len() as u64)
}

/// N35 (O16; 2.6, 2.7): the SEAL step-4 catalog write fails and the remount
/// then reads (a) all zero in sectors 0-15; (b) a valid catalog with another
/// destination ID that lists the provisional entry ID; (c) any other valid
/// catalog that is neither the committed copy nor that copy plus exactly the
/// provisional record (E1 changed; the new record under another ID; two new
/// records; the new record at copy sequence + 2; the committed records under
/// another destination ID; the committed records at copy sequence + 1);
/// (d) a non-blank layout that is no valid catalog.
/// (a)-(c): failed-mount state, INFO `Io`, export-lane LIST `Unknown` with
/// zero records, VERIFY and FORMAT `Io`; (d): `Corrupt` for all, never a
/// record. In every case no device write follows the failed one and the
/// binding never resolves as sealed.
/// Kills: a remount that ends blank (FORMAT `Access` or a FORMAT that
/// writes); a remount that mounts another catalog and resolves the binding
/// ((b) as sealed: LIST returns a record); dropping the provisional-ID or
/// the copy-sequence clause of the comparison; weakening the committed-copy
/// equality to the records, or ignoring its ID or sequence. Dropping only
/// its count clause is an equivalent mutant: a second new record would need
/// a sealing sequence above the copy's, which no valid copy has.
#[test]
fn n35_remount_contradicting_committed_catalog() {
    // The catalog a successful SEAL commits, from an identical run.
    let (mut ok, _, entry, length) = n35_setup();
    let committed = *ok.svc.catalog();
    assert_eq!(ok.seal(1, entry, length).status, Status::Ok);
    let sealed = *ok.svc.catalog();
    assert!(catalog::extends_by_one(&committed, &sealed));
    assert_eq!(sealed.records[committed.count].id, entry);

    let mut other_id = sealed;
    other_id.id[0] ^= 1;
    // The committed records under another ID or at a later copy sequence:
    // only the exact committed copy resolves the binding as not sealed.
    let mut committed_other_id = committed;
    committed_other_id.id[0] ^= 1;
    let mut committed_later = committed;
    committed_later.sequence += 1;
    let mut e1_changed = sealed;
    e1_changed.records[0].trailer ^= 1;
    let mut other_entry = sealed;
    other_entry.records[committed.count].id = entry + 1;
    other_entry.next_entry = entry + 2;
    let mut sequence_skipped = sealed;
    sequence_skipped.sequence += 1;
    let mut two_more = sealed;
    two_more.records[sealed.count] = Record {
        id: entry + 1,
        first: sealed.next_free,
        length: 80,
        trailer: 0,
        sealed_sequence: sealed.sequence + 1,
        label_length: 1,
        label: [0; 32],
    };
    two_more.records[sealed.count].label[0] = b'z';
    two_more.count += 1;
    two_more.sequence += 1;
    two_more.next_free += 1;
    two_more.next_entry += 1;
    let area_of = |c: &Catalog| {
        let bytes = c.encode();
        assert!(
            catalog::decode(&bytes, SECTORS).is_some(),
            "crafted catalog valid"
        );
        let mut area = vec![[0; SECTOR]; 16];
        put_copy(&mut area, 0, &bytes);
        put_copy(&mut area, 1, &bytes);
        area
    };
    let mut foreign = vec![[0; SECTOR]; 16];
    foreign[0] = [0xa5; SECTOR];
    let cases = [
        ("(a) all zero", vec![[0; SECTOR]; 16], false),
        ("(b) other destination ID", area_of(&other_id), false),
        ("(c) E1 changed", area_of(&e1_changed), false),
        (
            "(c) new record under another ID",
            area_of(&other_entry),
            false,
        ),
        ("(c) two new records", area_of(&two_more), false),
        ("(c) copy sequence + 2", area_of(&sequence_skipped), false),
        (
            "(c) committed records, other destination ID",
            area_of(&committed_other_id),
            false,
        ),
        (
            "(c) committed records at copy sequence + 1",
            area_of(&committed_later),
            false,
        ),
        ("(d) not a valid catalog", foreign, true),
    ];
    for (name, area, corrupt) in cases {
        let (mut h, e1, entry, length) = n35_setup();
        let mut swapped = false;
        h.disk.arm();
        h.disk.fault(|op, c| {
            (matches!(op, Op::Write(sector) if sector < 8) && c.catalog_writes == 0)
                .then(|| Fault::code(-12))
        });
        let mut seal = req(wire::SEAL);
        seal.entry = entry;
        seal.offset = length;
        assert_eq!(
            via(&mut h, &area, &mut swapped, 1, PORTAL, seal).status,
            Status::Unknown,
            "{name}"
        );
        h.disk.fault = None;
        assert!(swapped, "{name}: the step-4 write failed");
        let failed = position(
            &h.disk.log,
            |event| matches!(event, Event::Complete(Op::Write(sector), code) if *sector < 8 && *code != 0),
        );
        assert!(
            !h.disk.log[failed..]
                .iter()
                .any(|event| matches!(event, Event::Submit(Op::Write(_)))),
            "{name}: no write after the failed one"
        );
        let (state, refusal) = if corrupt {
            (State::Corrupt, Status::Corrupt)
        } else {
            (State::Failed { no_device: false }, Status::Io)
        };
        assert_eq!(h.svc.state(), state, "{name}");
        assert_eq!(h.svc.binding(1).unwrap().export, Export::Unknown, "{name}");
        let mutations = h.mutations();
        let info = via(&mut h, &area, &mut swapped, 0, DIALOG, req(wire::INFO));
        assert_eq!(info.status, refusal, "{name}: INFO");
        let list = via(&mut h, &area, &mut swapped, 1, PORTAL, req(wire::LIST));
        let listed = if corrupt {
            Status::Corrupt
        } else {
            Status::Unknown
        };
        assert_eq!(list.status, listed, "{name}: export LIST");
        assert_eq!(
            wire::header(&list.reply.unwrap()).length,
            0,
            "{name}: no record"
        );
        let mut verify = req(wire::VERIFY);
        verify.entry = e1;
        let verified = via(&mut h, &area, &mut swapped, 2, PORTAL, verify).status;
        assert_eq!(verified, refusal, "{name}: VERIFY");
        let format = via(&mut h, &area, &mut swapped, 0, DIALOG, req(wire::FORMAT)).status;
        assert_eq!(format, refusal, "{name}: FORMAT");
        assert_eq!(h.mutations(), mutations, "{name}: no device write");
        assert_eq!(h.svc.binding(1).unwrap().export, Export::Unknown, "{name}");
    }
}

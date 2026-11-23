//! Check order and authorization rows: N3, N17, N21, N22, N28, N31.
//! Expected permissions are written out here by hand from the "Permitted
//! on" column of 2.3, not taken from the library.
mod common;

use common::*;
use destination_core::{
    service::{Meta, Outcome, State},
    wire::{self, Header},
    Status,
};

/// Send `header` on `lane` with an explicit request ID and epoch.
fn raw(h: &mut H, lane: usize, sender: u64, header: Header, data: &[u8], caps: u32) -> Outcome {
    let mut header = header;
    header.length = data.len() as u32;
    let body = wire::encode(&header, data);
    h.send(
        Meta {
            lane,
            sender_pid: sender,
            cap_count: caps,
        },
        &body,
    )
}

/// True if a request with this ID on this lane is now `Stale`, i.e. the
/// high-water mark reached it.
fn consumed(h: &mut H, lane: usize, request_id: u64) -> bool {
    let mut info = req(wire::INFO);
    info.request_id = request_id;
    info.epoch = h.generation[lane];
    let sender = if lane == 0 { DIALOG } else { PORTAL };
    h.next_id[lane] = h.next_id[lane].max(request_id);
    raw(h, lane, sender, info, &[], 0).status == Status::Stale
}

fn device_ops(h: &H) -> usize {
    h.disk
        .log
        .iter()
        .filter(|e| matches!(e, Event::Submit(_) | Event::Refused(_)))
        .count()
}

/// Mounted with one sealed entry E1; lane 1 bound for export (generation 1),
/// lane 2 unbound. With `failed`, lane 3's SEAL fails at barrier 2 and the
/// remount FLUSH fails: the instance is in the failed-mount state.
fn scenario(failed: bool) -> (H, u64) {
    scenario_with(failed, false)
}

/// As `scenario`, optionally with lane 2 bound for restore of E1.
fn scenario_with(failed: bool, restore: bool) -> (H, u64) {
    let mut h = H::formatted();
    let e1 = h.export(1, &archive(1, 1, 300, 1));
    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    if restore {
        assert_eq!(h.bind(2, wire::BIND_RESTORE, e1), Status::Ok);
    }
    if failed {
        assert_eq!(h.bind(3, wire::BIND_EXPORT, 0), Status::Ok);
        let bytes = archive(2, 1, 200, 2);
        let entry = h.write_all(3, &bytes);
        h.disk.arm();
        // Flushes after arm: barrier 1 (0), barrier 2 (1), remount (2).
        h.disk
            .fault(|op, c| (op == Op::Flush && c.flushes >= 1).then(|| Fault::code(-12)));
        assert_eq!(h.seal(3, entry, bytes.len() as u64).status, Status::Unknown);
        assert_eq!(h.svc.state(), State::Failed { no_device: false });
        h.disk.fault = None;
    }
    (h, e1)
}

/// N28: every pair of checks 1-7 violated at once on an operation lane; the
/// earlier check decides. Kills: a swap of any two checks whose outcomes
/// differ in status, reply or high-water mark (see the spec's list; swaps
/// among checks 3, 4, 6 and the safe-path row are equivalent mutants).
#[test]
fn n28_check_order_operation_lane() {
    let expected = |check: usize| match check {
        1 | 2 => Status::Invalid,
        3 | 4 | 6 => Status::Access,
        5 => Status::Stale,
        _ => Status::Io,
    };
    for first in 1..=7 {
        for second in first..=7 {
            let (mut h, e1) = scenario(first == 7 || second == 7);
            let violated = |c: usize| c == first || c == second;
            let lane = if violated(3) { 2 } else { 1 };
            let sender = if violated(4) { OTHER } else { PORTAL };
            let mut header = if violated(6) {
                let mut verify = req(wire::VERIFY);
                verify.entry = e1;
                verify
            } else {
                req(wire::INFO)
            };
            let request_id = h.next_id[1] + 1;
            header.request_id = request_id;
            header.epoch = if violated(5) { 99 } else { h.generation[1] };
            let mut body = wire::encode(&header, &[]);
            if violated(2) {
                body[0] = 9;
            }
            let ops = device_ops(&h);
            let denials = h.svc.denials;
            let outcome = h.send(
                Meta {
                    lane,
                    sender_pid: sender,
                    cap_count: u32::from(violated(1)),
                },
                &body,
            );
            let want = expected(first);
            assert_eq!(outcome.status, want, "checks {first} and {second}");
            assert_eq!(
                outcome.reply.is_some(),
                sender == PORTAL,
                "reply {first}/{second}"
            );
            assert_eq!(outcome.close_caps, violated(1));
            assert_eq!(device_ops(&h), ops, "no device I/O {first}/{second}");
            if first < 7 {
                assert_eq!(h.svc.denials, denials + 1);
                assert!(
                    !consumed(&mut h, 1, request_id),
                    "high-water {first}/{second}"
                );
            }
        }
    }
}

/// N28 on the control lane, including check 6 against check 7 in the
/// failed-mount, `Corrupt` and blank states.
#[test]
fn n28_check_order_control_lane() {
    let states: Vec<(&str, H, Status)> = vec![
        ("blank", H::blank(), Status::NotFound),
        (
            "failed",
            {
                let mut disk = SimDisk::new(SECTORS);
                disk.fault(|op, _| (op == Op::Flush).then(|| Fault::code(-12)));
                H::boot(disk)
            },
            Status::Io,
        ),
        (
            "corrupt",
            {
                let mut image = vec![[0; 512]; SECTORS as usize];
                image[2][9] = 1;
                H::cold(image)
            },
            Status::Corrupt,
        ),
    ];
    for (name, mut h, state_status) in states {
        h.disk.fault = None;
        for (check, violation) in [(1, "caps"), (2, "version"), (5, "epoch"), (6, "op")] {
            let mut header = if check == 6 {
                let mut create = req(wire::CREATE);
                create.name_length = 1;
                create
            } else {
                req(wire::LIST)
            };
            let request_id = h.next_id[0] + 1;
            header.request_id = request_id;
            header.epoch = u64::from(check == 5);
            header.length = u32::from(check == 6);
            let data: &[u8] = if check == 6 { b"x" } else { &[] };
            let mut body = wire::encode(&header, data);
            if check == 2 {
                body[0] = 9;
            }
            let outcome = h.send(
                Meta {
                    lane: 0,
                    sender_pid: DIALOG,
                    cap_count: u32::from(check == 1),
                },
                &body,
            );
            let want = match check {
                1 | 2 => Status::Invalid,
                5 => Status::Stale,
                _ => Status::Access,
            };
            assert_eq!(
                outcome.status, want,
                "{name}: check {check} ({violation}) before 7"
            );
            assert!(!consumed(&mut h, 0, request_id));
        }
        // Check 7 alone answers the state's code and consumes the ID.
        let request_id = h.next_id[0] + 1;
        let mut list = req(wire::LIST);
        list.request_id = request_id;
        assert_eq!(
            raw(&mut h, 0, DIALOG, list, &[], 0).status,
            state_status,
            "{name}"
        );
        assert!(consumed(&mut h, 0, request_id));
    }
}

/// N17: stale grants, replays and header rules.
/// Kills: a BIND that accepts a generation equal to the lane maximum; a
/// validator that advances the high-water mark on a request rejected by
/// checks 1-6; a validator that does not advance it on a check-7 refusal.
#[test]
fn n17_replay_and_rebind() {
    let mut h = H::formatted();
    let bytes = archive(1, 1, 400, 4);
    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    let entry = h.write_all(1, &bytes);
    // Duplicated APPEND offset: Stale, nothing appended.
    let writes = h.disk.writes;
    assert_eq!(h.append(1, entry, 0, b"again").status, Status::Stale);
    assert_eq!(h.disk.writes, writes);
    assert_eq!(h.seal(1, entry, bytes.len() as u64).status, Status::Ok);
    // Second SEAL: Access at check 6, no second entry.
    assert_eq!(h.seal(1, entry, bytes.len() as u64).status, Status::Access);
    assert_eq!(h.list_ids(0).1, vec![entry]);

    // Rebind with g+1; a request with epoch g is Stale.
    let old = h.generation[1];
    assert_eq!(h.unbind(1), Status::Ok);
    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    let mut info = req(wire::INFO);
    info.request_id = 1;
    info.epoch = old;
    assert_eq!(raw(&mut h, 1, PORTAL, info, &[], 0).status, Status::Stale);

    // Unbound lane: any well-formed request is Access.
    assert_eq!(h.portal(2, req(wire::INFO), &[]).status, Status::Access);

    // Control lane: epoch != 0 and non-increasing request IDs are Stale.
    let mut info = req(wire::INFO);
    info.request_id = h.next_id[0] + 1;
    info.epoch = 1;
    assert_eq!(raw(&mut h, 0, DIALOG, info, &[], 0).status, Status::Stale);
    info.epoch = 0;
    info.request_id = h.next_id[0];
    assert_eq!(raw(&mut h, 0, DIALOG, info, &[], 0).status, Status::Stale);
    info.request_id = 0;
    assert_eq!(raw(&mut h, 0, DIALOG, info, &[], 0).status, Status::Stale);

    // CREATE with length != name_length.
    let mut create = req(wire::CREATE);
    create.name_length = 3;
    assert_eq!(h.portal(1, create, b"ab").status, Status::Invalid);

    // A rejection by checks 1-6 does not advance the mark.
    let next = h.next_id[1] + 1;
    let mut bad = req(wire::INFO);
    bad.request_id = next;
    bad.epoch = h.generation[1];
    bad.count = 1; // INFO requires count 0: check 2
    assert_eq!(raw(&mut h, 1, PORTAL, bad, &[], 0).status, Status::Invalid);
    let mut good = req(wire::INFO);
    good.request_id = next;
    good.epoch = h.generation[1];
    assert_eq!(raw(&mut h, 1, PORTAL, good, &[], 0).status, Status::Ok);
}

#[test]
fn n17_rebind_with_equal_or_lower_generation_while_append_queued() {
    for lower in [false, true] {
        let mut h = H::formatted();
        assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
        assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Busy);
        assert_eq!(h.unbind(1), Status::Ok);
        assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
        let g = h.generation[1];
        let entry = wire::header(&h.create(1, b"x").reply.unwrap()).entry;
        // An APPEND of binding g, queued on the lane.
        let mut queued = req(wire::APPEND);
        queued.request_id = h.next_id[1] + 1;
        queued.epoch = g;
        queued.entry = entry;
        assert_eq!(h.unbind(1), Status::Ok);
        let again = if lower { g - 1 } else { g };
        assert_eq!(
            h.bind_with(1, wire::BIND_EXPORT, 0, again, &TEST_ID),
            Status::Stale
        );
        assert!(h.svc.binding(1).is_none(), "lane stays unbound");
        let writes = h.disk.writes;
        assert_eq!(
            raw(&mut h, 1, PORTAL, queued, b"data", 0).status,
            Status::Access
        );
        assert_eq!(h.disk.writes, writes);
        let (_, _, data) = h.info();
        assert_eq!(
            &data[24..32],
            &g.to_le_bytes(),
            "INFO reports the unchanged maximum"
        );
    }
}

#[test]
fn n17_check7_refusal_advances_mark_in_safe_path() {
    let mut h = H::formatted();
    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    let entry = wire::header(&h.create(1, b"x").reply.unwrap()).entry;
    h.disk
        .fault(|op, _| matches!(op, Op::Write(_)).then(|| Fault::code(-12)));
    let chunk = [7u8; 160];
    let mut offset = 0;
    loop {
        let outcome = h.append(1, entry, offset, &chunk);
        if outcome.status != Status::Ok {
            assert_eq!(outcome.status, Status::Io);
            break;
        }
        offset += 160;
    }
    h.disk.fault = None;
    assert!(h.svc.safe_path());
    assert_eq!(h.bind(2, wire::BIND_EXPORT, 0), Status::Ok);
    let mut create = req(wire::CREATE);
    create.name_length = 1;
    create.request_id = 1;
    create.epoch = h.generation[2];
    assert_eq!(
        raw(&mut h, 2, PORTAL, create, b"y", 0).status,
        Status::Access
    );
    let mut info = req(wire::INFO);
    info.request_id = 1;
    info.epoch = h.generation[2];
    assert_eq!(raw(&mut h, 2, PORTAL, info, &[], 0).status, Status::Stale);
}

/// Lane kinds for N31.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Control,
    Unbound,
    Export,
    Verify,
    Restore,
}

/// Hand-written "Permitted on" table (2.3), checked at check 6.
fn allowed(kind: Kind, op: u16) -> bool {
    match kind {
        Kind::Control => matches!(op, 1 | 2 | 6 | 9 | 10 | 11),
        Kind::Unbound => false,
        Kind::Export => matches!(op, 1..=3),
        Kind::Verify => matches!(op, 1 | 2 | 7),
        Kind::Restore => matches!(op, 1 | 2 | 8),
    }
}

/// A well-formed request for `op` naming `entry` where an entry is needed.
fn request_for(op: u16, entry: u64) -> (Header, Vec<u8>) {
    let mut h = req(op);
    let mut data = Vec::new();
    match op {
        wire::CREATE => {
            h.name_length = 1;
            data = b"l".to_vec();
        }
        wire::APPEND => {
            h.entry = entry;
            data = b"d".to_vec();
        }
        wire::SEAL | wire::ABORT | wire::VERIFY => h.entry = entry,
        wire::READ => {
            h.entry = entry;
            h.aux = 1;
        }
        wire::BIND => {
            h.name_length = 2;
            h.entry = entry;
            h.count = 2;
            h.aux = 50;
            data = TEST_ID.to_vec();
        }
        wire::UNBIND => h.count = 2,
        _ => {}
    }
    (h, data)
}

/// N31: every operation on every lane kind; permitted operations pass check
/// 6 (they consume their request ID), all others are refused with `Access`
/// before it. Kills: any flipped entry of the permission table.
#[test]
fn n31_operation_by_lane() {
    for kind in [
        Kind::Control,
        Kind::Unbound,
        Kind::Export,
        Kind::Verify,
        Kind::Restore,
    ] {
        for op in 1..=11u16 {
            let mut h = H::formatted();
            let e1 = h.export(1, &archive(1, 1, 200, 1));
            let lane = match kind {
                Kind::Control => 0,
                _ => 1,
            };
            match kind {
                Kind::Export => assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok),
                Kind::Verify => assert_eq!(h.bind(1, wire::BIND_VERIFY, e1), Status::Ok),
                Kind::Restore => assert_eq!(h.bind(1, wire::BIND_RESTORE, e1), Status::Ok),
                _ => {}
            }
            let (mut header, data) = request_for(op, e1);
            let request_id = h.next_id[lane] + 1;
            header.request_id = request_id;
            header.epoch = h.generation[lane];
            let sender = if lane == 0 { DIALOG } else { PORTAL };
            let status = raw(&mut h, lane, sender, header, &data, 0).status;
            let passed = kind != Kind::Unbound && consumed(&mut h, lane, request_id);
            assert_eq!(passed, allowed(kind, op), "{kind:?} op {op} ({status:?})");
            if !allowed(kind, op) {
                assert_eq!(status, Status::Access, "{kind:?} op {op}");
            }
        }
    }
}

/// N31: export-binding life cycle, LIST scoping and wrong entries.
#[test]
fn n31_binding_limits_and_list_scope() {
    let mut h = H::formatted();
    let e1 = h.export(1, &archive(1, 1, 200, 1));
    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    assert_eq!(h.bind(2, wire::BIND_VERIFY, e1), Status::Ok);
    assert_eq!(h.bind(3, wire::BIND_RESTORE, e1), Status::Ok);
    assert_eq!(
        h.list_ids(1),
        (Status::Ok, vec![]),
        "export lane: none before seal"
    );
    let bytes = archive(2, 1, 250, 2);
    let e2 = h.write_all(1, &bytes);
    assert_eq!(h.list_ids(1).1, Vec::<u64>::new());
    // Wrong entries on operation lanes.
    assert_eq!(h.seal(1, e2 + 1, bytes.len() as u64).status, Status::Access);
    assert_eq!(
        h.append(1, e1, bytes.len() as u64, b"x").status,
        Status::Access
    );
    let mut abort = req(wire::ABORT);
    abort.entry = e1;
    assert_eq!(h.portal(1, abort, &[]).status, Status::Access);
    assert_eq!(h.verify_entry(2, e2), Status::Access);
    let mut read = req(wire::READ);
    read.entry = e2;
    read.aux = 1;
    assert_eq!(h.portal(3, read, &[]).status, Status::Access);
    // Control ABORT of a non-writing entry.
    let mut abort = req(wire::ABORT);
    abort.entry = e1;
    assert_eq!(h.control(abort, &[]).status, Status::NotFound);
    assert_eq!(h.seal(1, e2, bytes.len() as u64).status, Status::Ok);
    assert_eq!(
        h.list_ids(1).1,
        vec![e2],
        "export lane: its own entry once sealed"
    );
    assert_eq!(
        h.list_ids(2).1,
        vec![e1],
        "verify lane: the bound entry only"
    );
    assert_eq!(h.list_ids(3).1, vec![e1]);
    assert_eq!(h.list_ids(0).1, vec![e1, e2]);
    // Second CREATE after SEAL.
    assert_eq!(h.create(1, b"again").status, Status::Access);
    // Second CREATE after ABORT.
    assert_eq!(h.unbind(1), Status::Ok);
    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    let e3 = wire::header(&h.create(1, b"three").reply.unwrap()).entry;
    let mut abort = req(wire::ABORT);
    abort.entry = e3;
    assert_eq!(h.portal(1, abort, &[]).status, Status::Ok);
    assert_eq!(h.create(1, b"again").status, Status::Access);
    assert_eq!(h.list_ids(1).1, Vec::<u64>::new());
}

/// N31 (revision 2): the entry check is check 6, so a wrong-entry READ in the
/// failed-mount state is `Access`, not `Io`.
#[test]
fn n31_wrong_entry_in_failed_mount_is_access() {
    let (mut h, e1) = scenario_with(true, true);
    let read = |entry: u64| {
        let mut r = req(wire::READ);
        r.entry = entry;
        r.aux = 1;
        r
    };
    assert_eq!(h.portal(2, read(e1 + 1), &[]).status, Status::Access);
    assert_eq!(h.portal(2, read(e1), &[]).status, Status::Io);
    assert_eq!(h.unbind(2), Status::Io, "failed mount: UNBIND answered Io");
}

/// N22: the dialog uses a lane bound to the portal with its own root.
/// Kills: a service that skips the sender check, or replies to the portal
/// for a non-portal sender.
#[test]
fn n22_dialog_on_bound_lane() {
    let mut h = H::formatted();
    let e1 = h.export(1, &archive(1, 1, 200, 1));
    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    assert_eq!(h.bind(2, wire::BIND_RESTORE, e1), Status::Ok);
    let entry = wire::header(&h.create(1, b"x").reply.unwrap()).entry;
    let mut read = req(wire::READ);
    read.entry = e1;
    read.aux = 10;
    let mut append = req(wire::APPEND);
    append.entry = entry;
    for (lane, header, data) in [
        (1, req(wire::INFO), &b""[..]),
        (1, append, &b"data"[..]),
        (2, read, &b""[..]),
    ] {
        let ops = device_ops(&h);
        let denials = h.svc.denials;
        let outcome = h.on(lane, DIALOG, header, data);
        assert_eq!(outcome.status, Status::Access);
        assert!(outcome.reply.is_none(), "no reply to a non-portal sender");
        assert_eq!(h.svc.denials, denials + 1);
        assert_eq!(device_ops(&h), ops);
    }
    // The portal on an unbound lane is still answered.
    assert!(h.portal(3, req(wire::INFO), &[]).reply.is_some());
}

/// N3: scope escape within the destination (restore and verify lanes).
/// Kills: a READ bound check that compares only `offset` with the length, or
/// uses wrapping addition.
#[test]
fn n3_read_bounds() {
    let mut h = H::formatted();
    let bytes = archive(1, 1, 300, 1);
    let e1 = h.export(1, &bytes);
    let e2 = h.export(1, &archive(2, 1, 100, 2));
    let length = bytes.len() as u64;
    assert_eq!(h.bind(1, wire::BIND_RESTORE, e1), Status::Ok);
    assert_eq!(h.bind(2, wire::BIND_VERIFY, e1), Status::Ok);
    let read = |entry: u64, offset: u64, aux: u64| {
        let mut r = req(wire::READ);
        r.entry = entry;
        r.offset = offset;
        r.aux = aux;
        r
    };
    assert_eq!(h.read_entry(1, e1, length).unwrap(), bytes);
    assert_eq!(h.portal(1, read(e2, 0, 10), &[]).status, Status::Access);
    for (offset, aux) in [
        (length, 1),
        (length - 5, 10),
        (u64::MAX - 3, 10),
        (length + 1, 1),
    ] {
        let outcome = h.portal(1, read(e1, offset, aux), &[]);
        assert_eq!(outcome.status, Status::Range, "offset {offset} aux {aux}");
        assert!(wire::data(&outcome.reply.unwrap()).is_empty(), "no data");
    }
    let mut append = req(wire::APPEND);
    append.entry = e1;
    assert_eq!(h.portal(2, append, b"x").status, Status::Access);
}

/// N21: a second writer.
#[test]
fn n21_second_writer_busy() {
    let mut h = H::formatted();
    assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
    assert_eq!(h.bind(2, wire::BIND_EXPORT, 0), Status::Ok);
    assert_eq!(h.create(1, b"one").status, Status::Ok);
    assert_eq!(h.create(2, b"two").status, Status::Busy);
    // The refused CREATE did not use up lane 2's single CREATE.
    assert_eq!(h.unbind(1), Status::Ok);
    assert_eq!(h.create(2, b"two").status, Status::Ok);
}

/// O3 (review R2): N28 pairs that the loops above cannot separate.
/// (1,2) with a bad operation: check 1 gives `Invalid`, check 2 would give
/// `NoSys`. (3,5) and (4,5) through a stale `request_id` instead of the
/// epoch. Control lane (5,6): epoch 1 with a CREATE gives `Stale`.
/// Kills: check 1 after check 2 (M19a); the `request_id` half of check 5
/// before checks 3/4 (M19c); control-lane check 5 after check 6 (M19g).
#[test]
fn o3_check_order_extra_pairs() {
    let (mut h, _) = scenario(false);
    let fresh = |h: &H, lane: usize| h.next_id[lane] + 1;

    // (1,2): transferred capability and operation 99.
    let request_id = fresh(&h, 1);
    let mut header = req(wire::INFO);
    header.request_id = request_id;
    header.epoch = h.generation[1];
    let mut body = wire::encode(&header, &[]);
    body[2..4].copy_from_slice(&99u16.to_le_bytes());
    let meta = Meta {
        lane: 1,
        sender_pid: PORTAL,
        cap_count: 1,
    };
    let outcome = h.send(meta, &body);
    assert_eq!(outcome.status, Status::Invalid, "check 1 before 2");
    assert!(outcome.close_caps);
    assert!(!consumed(&mut h, 1, request_id));

    // (3,5) and (4,5): request_id 0 (Stale at check 5) on an unbound lane
    // and from another sender.
    for (lane, sender) in [(2, PORTAL), (1, OTHER)] {
        let mut info = req(wire::INFO);
        info.epoch = h.generation[lane];
        let outcome = raw(&mut h, lane, sender, info, &[], 0);
        assert_eq!(
            outcome.status,
            Status::Access,
            "lane {lane} sender {sender}"
        );
        assert_eq!(outcome.reply.is_some(), sender == PORTAL);
    }
    let request_id = fresh(&h, 1);
    assert!(!consumed(&mut h, 1, request_id));

    // Control lane (5,6): epoch 1 and a CREATE, which the control lane
    // never permits.
    let request_id = fresh(&h, 0);
    let mut create = req(wire::CREATE);
    create.name_length = 1;
    create.request_id = request_id;
    create.epoch = 1;
    let outcome = raw(&mut h, 0, DIALOG, create, b"x", 0);
    assert_eq!(outcome.status, Status::Stale, "control check 5 before 6");
    assert!(!consumed(&mut h, 0, request_id));
}

fn set32(body: &mut [u8; wire::SIZE], at: usize, value: u32) {
    body[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

fn set64(body: &mut [u8; wire::SIZE], at: usize, value: u64) {
    body[at..at + 8].copy_from_slice(&value.to_le_bytes());
}

// Header field offsets of 2.2.
const ENTRY: usize = 24;
const OFFSET: usize = 32;
const LENGTH: usize = 40;
const STATUS: usize = 44;
const NAME: usize = 48;
const COUNT: usize = 52;
const AUX: usize = 56;

type Edit = (&'static str, fn(&mut [u8; wire::SIZE]));
/// Operation, baseline body, violations (`Invalid`), allowed "any" values.
type Case = (&'static str, [u8; wire::SIZE], Vec<Edit>, Vec<Edit>);

/// O2 (review R2): check 2, one field rule violated at a time per operation
/// (field-rule table of 2.2). Each baseline passes; each violation is
/// `Invalid`; the "any" fields accept any value. Kills: dropping the
/// trailing-data-zero rule (V2), a non-zero `aux` allowed everywhere (V3), a
/// non-zero `status` (V4), non-zero bytes 4..8 (V5), the BIND entry rule (V6),
/// and any other single field rule.
#[test]
fn o2_check2_field_rules() {
    let base = |operation: u16, setup: fn(&mut Header), data: &[u8]| {
        let mut header = req(operation);
        header.request_id = 1;
        header.length = data.len() as u32;
        setup(&mut header);
        wire::encode(&header, data)
    };
    let common: [Edit; 4] = [
        ("bytes 4..8", |b| b[4] = 1),
        ("status", |b| set32(b, STATUS, 1)),
        ("trailing data", |b| b[223] = 1),
        ("length 161", |b| set32(b, LENGTH, 161)),
    ];
    let no_aux: Edit = ("aux", |b| set64(b, AUX, 1));
    let entry_one: Edit = ("entry != 0", |b| set64(b, ENTRY, 1));
    let entry_zero: Edit = ("entry 0", |b| set64(b, ENTRY, 0));
    let offset: Edit = ("offset", |b| set64(b, OFFSET, 1));
    let length_one: Edit = ("length 1", |b| set32(b, LENGTH, 1));
    let name: Edit = ("name_length", |b| set32(b, NAME, 1));
    let count: Edit = ("count", |b| set32(b, COUNT, 1));
    let zero: fn(&mut Header) = |_| {};
    let simple = vec![entry_one, offset, length_one, name, count, no_aux];
    let cases: Vec<Case> = vec![
        ("INFO", base(wire::INFO, zero, &[]), simple.clone(), vec![]),
        ("FORMAT", base(wire::FORMAT, zero, &[]), simple, vec![]),
        (
            "LIST",
            base(wire::LIST, |h| h.count = 15, &[]),
            vec![
                entry_one,
                offset,
                length_one,
                name,
                ("count 16", |b| set32(b, COUNT, 16)),
                no_aux,
            ],
            vec![],
        ),
        (
            "CREATE",
            base(wire::CREATE, |h| h.name_length = 1, b"x"),
            vec![
                entry_one,
                offset,
                ("length 0", |b| {
                    set32(b, LENGTH, 0);
                    set32(b, NAME, 0);
                    b[64] = 0;
                }),
                ("length 33", |b| {
                    set32(b, LENGTH, 33);
                    set32(b, NAME, 33);
                }),
                ("name_length != length", |b| set32(b, NAME, 2)),
                count,
                no_aux,
            ],
            vec![],
        ),
        (
            "APPEND",
            base(wire::APPEND, |h| h.entry = 1, b"x"),
            vec![
                entry_zero,
                ("length 0", |b| {
                    set32(b, LENGTH, 0);
                    b[64] = 0;
                }),
                name,
                count,
                no_aux,
            ],
            vec![("offset any", |b| set64(b, OFFSET, u64::MAX))],
        ),
        (
            "SEAL",
            base(wire::SEAL, |h| h.entry = 1, &[]),
            vec![entry_zero, length_one, name, count, no_aux],
            vec![("offset any", |b| set64(b, OFFSET, 7))],
        ),
        (
            "ABORT",
            base(wire::ABORT, |h| h.entry = 1, &[]),
            vec![entry_zero, offset, length_one, name, count, no_aux],
            vec![],
        ),
        (
            "VERIFY",
            base(wire::VERIFY, |h| h.entry = 1, &[]),
            vec![entry_zero, offset, length_one, name, count, no_aux],
            vec![],
        ),
        (
            "READ",
            base(
                wire::READ,
                |h| {
                    h.entry = 1;
                    h.aux = 160;
                },
                &[],
            ),
            vec![
                entry_zero,
                length_one,
                name,
                count,
                ("aux 0", |b| set64(b, AUX, 0)),
                ("aux 161", |b| set64(b, AUX, 161)),
            ],
            vec![("offset any", |b| set64(b, OFFSET, 9))],
        ),
        (
            "BIND export",
            base(
                wire::BIND,
                |h| {
                    h.name_length = wire::BIND_EXPORT;
                    h.count = 1;
                },
                &[0; 16],
            ),
            vec![
                ("export entry != 0", |b| set64(b, ENTRY, 1)),
                offset,
                ("length 15", |b| set32(b, LENGTH, 15)),
                ("name_length 0", |b| set32(b, NAME, 0)),
                ("name_length 4", |b| set32(b, NAME, 4)),
                ("count 0", |b| set32(b, COUNT, 0)),
                ("count 4", |b| set32(b, COUNT, 4)),
            ],
            vec![("aux any", |b| set64(b, AUX, u64::MAX))],
        ),
        (
            "BIND verify",
            base(
                wire::BIND,
                |h| {
                    h.name_length = wire::BIND_VERIFY;
                    h.entry = 1;
                    h.count = 3;
                },
                &[0; 16],
            ),
            vec![
                ("verify entry 0", |b| set64(b, ENTRY, 0)),
                ("name_length 0", |b| set32(b, NAME, 0)),
                ("name_length 4", |b| set32(b, NAME, 4)),
            ],
            vec![],
        ),
        (
            "UNBIND",
            base(wire::UNBIND, |h| h.count = 1, &[]),
            vec![
                entry_one,
                offset,
                length_one,
                name,
                ("count 0", |b| set32(b, COUNT, 0)),
                ("count 4", |b| set32(b, COUNT, 4)),
                no_aux,
            ],
            vec![],
        ),
    ];
    for (op, body, violations, allowed) in cases {
        assert!(wire::validate(&body).is_ok(), "{op}: baseline");
        for (rule, edit) in common.iter().chain(&violations) {
            let mut bad = body;
            edit(&mut bad);
            assert_eq!(
                wire::validate(&bad).err(),
                Some(Status::Invalid),
                "{op}: {rule}"
            );
        }
        for (rule, edit) in &allowed {
            let mut ok = body;
            edit(&mut ok);
            assert!(wire::validate(&ok).is_ok(), "{op}: {rule}");
        }
    }

    // Through the service: a check-2 refusal leaves the high-water mark.
    let (mut h, _) = scenario(false);
    for (rule, edit) in common.iter().chain([&no_aux]) {
        let request_id = h.next_id[0] + 1;
        let mut info = req(wire::INFO);
        info.request_id = request_id;
        let mut body = wire::encode(&info, &[]);
        edit(&mut body);
        let meta = Meta {
            lane: 0,
            sender_pid: DIALOG,
            cap_count: 0,
        };
        assert_eq!(h.send(meta, &body).status, Status::Invalid, "{rule}");
        assert!(!consumed(&mut h, 0, request_id), "{rule}: high-water");
    }
}

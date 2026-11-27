//! A03-T2c-2 native acceptance: the task-grant broker is the sole client of a
//! read-only volume domain 0 and the only process that can cause a volume
//! READ. Four processes: broker (slot 0), owner app (slot 1), volume service
//! (slot 2, the unchanged volume-docs ELF) and a hostile helper (slot 3, same
//! client ELF as the owner). Apps hold endpoint rights only; the volume holds a
//! BLOCK_READ-only grant and runs with the read-only startup word set.
//!
//! FIXTURES, as in `task_grant.rs`: the bootstrap grant words in the broker's
//! DATA page stand in for the trusted permission dialog (omitted by
//! `test-task-grant-volume-denied`), and the kernel writes each client's exit
//! PID into the broker's private page in place of a product exit
//! notification. Domain 1 of the volume names the helper's PID with its send
//! handle closed (the `nas.rs` pattern), so no process can reach it.
//! This module replaces `system` via a path attribute in m3.rs.
use super::*;
use capability_core::{BlockScope, BLOCK_FLUSH, BLOCK_READ, BLOCK_WRITE, RECV, SEND};

const BROKER: usize = 0;
const OWNER: usize = 1;
const VOLUME: usize = 2;
const HELPER: usize = 3;
const TIMEOUT_TICKS: u64 = 20_000;
const DENIED: bool = cfg!(feature = "test-task-grant-volume-denied");

// Wire values shared with sdk/examples/task-grant-broker/src/lib.rs.
const OP_READ: u64 = 1;
const OP_WRITE: u64 = 2;
const OP_CLOSE: u64 = 3;
const OP_RETIRE: u64 = 4;
const OK: u64 = 1;
const NO: u64 = 2;
const GO: u64 = 4;
const RESUME: u64 = 5;
/// Startup STATs of objects 1 and 2 (object 2 is absent from the seed).
const STARTUP_CALLS: u64 = 2;
/// Broker words, see sdk/examples/task-grant-broker/src/broker.rs.
const BROKER_READY: u64 = 48;
const KERNEL_ACK: u64 = 416;
const VOLUME_LOG: u64 = 520;

#[derive(Clone, Copy)]
struct ClientEvidence {
    exited: bool,
    pid: u64,
    code: u64,
    statuses: [u64; 10],
    content: [u64; 2],
}
const NONE: ClientEvidence = ClientEvidence {
    exited: false,
    pid: 0,
    code: 0,
    statuses: [0; 10],
    content: [0; 2],
};
struct Evidence {
    clients: [ClientEvidence; 2],
    /// Volume read_sectors and rejected (its NotFound STAT of absent object 2)
    /// and the kernel block broker's completed reads when the broker finished
    /// its STATs.
    baseline: Option<(u64, u64, u64)>,
    kernel_tables: usize,
}
static mut EVIDENCE: Evidence = Evidence {
    clients: [NONE; 2],
    baseline: None,
    kernel_tables: 0,
};

fn evidence() -> &'static mut Evidence {
    // SAFETY: KERNEL held, IF=0; serialized kernel service-stack access and
    // the borrow ends before scheduling.
    unsafe { &mut *ptr::addr_of_mut!(EVIDENCE) }
}
fn fail(reason: &[u8]) -> ! {
    serial::write(b"TASKGRANT:FAIL:");
    serial::write(reason);
    serial::write(b"\n");
    serial::exit(0x11)
}
fn require(ok: bool, reason: &[u8]) {
    if !ok {
        fail(reason);
    }
}
fn put(s: &State, slot: usize, offset: u64, value: u64) {
    write_word(s.processes[slot].physical(DATA + offset), value);
}
fn word(s: &State, slot: usize, offset: u64) -> u64 {
    read_word(s.processes[slot].physical(DATA + offset))
}
pub(super) fn image_elf(_: u32) -> Option<&'static [u8]> {
    None
}
fn channel(e: &mut capability_core::Engine, owner: u64, sender: u64) -> (u64, u64) {
    let root = e
        .create_endpoint(owner)
        .unwrap_or_else(|_| fail(b"ENDPOINT"));
    let receive = e
        .derive(owner, root, RECV)
        .unwrap_or_else(|_| fail(b"ENDPOINT_RECV"));
    let send = e
        .bootstrap_delegate(owner, root, sender, SEND)
        .unwrap_or_else(|_| fail(b"ENDPOINT_SEND"));
    e.close(owner, root)
        .unwrap_or_else(|_| fail(b"ENDPOINT_ROOT"));
    (receive, send)
}
/// First 16 bytes of `volume_disk.fixture(1, 1, 300)` (seed object 1), as the
/// owner stores them: two little-endian words.
const fn content_words() -> [u64; 2] {
    let mut words = [0u64; 2];
    let mut i = 0;
    while i < 16 {
        let byte = ((31 + 7 + 13 * i + i / 251) & 0xff) as u64;
        words[i / 8] |= byte << ((i % 8) * 8);
        i += 1;
    }
    words
}

pub(super) fn manifest(s: &mut State) {
    require(
        s.amount == MAX
            && s.processes[..4].iter().all(|p| p.status != Status::Dead)
            && s.processes[4..].iter().all(|p| p.status == Status::Dead),
        b"MANIFEST_STATE",
    );
    let [broker, owner, volume, helper] =
        [BROKER, OWNER, VOLUME, HELPER].map(|i| s.processes[i].pid);
    let e = m4::engine();
    for pid in [broker, owner, volume, helper] {
        e.register(pid).unwrap_or_else(|_| fail(b"REGISTER"));
    }

    // One request endpoint owned by the broker; both clients may only send.
    let root = e
        .create_endpoint(broker)
        .unwrap_or_else(|_| fail(b"REQUEST_ENDPOINT"));
    let request_recv = e
        .derive(broker, root, RECV)
        .unwrap_or_else(|_| fail(b"REQUEST_RECV"));
    let request_send = [owner, helper].map(|client| {
        e.bootstrap_delegate(broker, root, client, SEND)
            .unwrap_or_else(|_| fail(b"REQUEST_SEND"))
    });
    e.close(broker, root)
        .unwrap_or_else(|_| fail(b"REQUEST_ROOT"));
    // One private reply endpoint per client; only the broker may send to it.
    let [(owner_reply, owner_reply_send), (helper_reply, helper_reply_send)] =
        [owner, helper].map(|client| channel(e, client, broker));

    // Volume domain 0: the broker alone. Domain 1: the helper's PID with its
    // send handle closed, so the second ingress stays unreachable.
    let (volume_ingress, broker_volume) = channel(e, volume, broker);
    let (broker_volume_reply, volume_reply_send) = channel(e, broker, volume);
    let (volume_other, unused_send) = channel(e, volume, helper);
    e.close(helper, unused_send)
        .unwrap_or_else(|_| fail(b"DOMAIN1_SEND"));
    let (unused_reply, volume_other_reply) = channel(e, helper, volume);
    e.close(helper, unused_reply)
        .unwrap_or_else(|_| fail(b"DOMAIN1_REPLY"));
    let block = e
        .create_block_device_with_rights(
            volume,
            BlockScope {
                device: 0,
                first_sector: 4,
                sector_count: 580,
            },
            BLOCK_READ,
        )
        .unwrap_or_else(|_| fail(b"BLOCK_GRANT"));
    require(
        e.check_block(volume, block, BLOCK_READ).is_ok()
            && e.check_block(volume, block, BLOCK_WRITE).is_err()
            && e.check_block(volume, block, BLOCK_FLUSH).is_err(),
        b"VOLUME_READ_ONLY",
    );
    // Apps: exactly their request send and reply receive, nothing else.
    for (pid, send, reply) in [
        (owner, request_send[0], owner_reply),
        (helper, request_send[1], helper_reply),
    ] {
        require(
            e.process_usage(pid).map(|usage| usage.handles) == Ok(2)
                && e.check_endpoint(pid, send, SEND).is_ok()
                && e.check_endpoint(pid, send, RECV).is_err()
                && e.check_endpoint(pid, reply, RECV).is_ok()
                && e.check_endpoint(pid, reply, SEND).is_err(),
            b"CLIENT_RIGHTS",
        );
        for handle in [send, reply] {
            require(
                e.check_block(pid, handle, BLOCK_READ).is_err()
                    && e.check_entropy(pid, handle).is_err()
                    && e.check_launch(pid, handle).is_err()
                    && e.check_network(pid, handle).is_err(),
                b"CLIENT_AUTHORITY",
            );
        }
    }
    for handle in [
        request_recv,
        owner_reply_send,
        helper_reply_send,
        broker_volume,
        broker_volume_reply,
    ] {
        require(
            e.check_block(broker, handle, BLOCK_READ).is_err(),
            b"BROKER_AUTHORITY",
        );
    }
    require(e.check_invariants(), b"MANIFEST_ENGINE");

    for slot in [BROKER, OWNER, HELPER] {
        put(s, slot, 32, 1);
        put(s, slot, 40, 0);
        for offset in (96..640).step_by(8) {
            put(s, slot, offset, 0);
        }
    }
    for (offset, value) in [
        (0, 0),
        (BROKER_READY, 0),
        (56, request_recv),
        (64, owner_reply_send),
        (72, helper_reply_send),
        (80, broker_volume),
        (88, broker_volume_reply),
        (176, owner),
        (184, helper),
        (408, volume),
    ] {
        put(s, BROKER, offset, value);
    }
    if !DENIED {
        // FIXTURE standing in for the trusted dialog: owner, object 1,
        // revision 1 (the seed's revision), READ only, until close; and the
        // same decision again, applied after the owner's close.
        for base in [96, 360] {
            for (index, value) in [1, owner, 1, 1, 1, 1].into_iter().enumerate() {
                put(s, BROKER, base + index as u64 * 8, value);
            }
        }
    }
    for (slot, role, reply, send) in [
        (OWNER, 1, owner_reply, request_send[0]),
        (HELPER, 2, helper_reply, request_send[1]),
    ] {
        put(s, slot, 0, role);
        put(s, slot, 56, reply);
        put(s, slot, 64, send);
        put(s, slot, 176, broker);
        // The value both clients put in the claimed actor field.
        put(s, slot, 184, owner);
    }
    for (offset, value) in [
        (32, 1),
        (48, block),
        (56, volume_ingress),
        (80, volume_other),
        (88, volume_reply_send),
        (72, volume_other_reply),
        (120, 1),
        (160, broker),
        (168, helper),
        (176, volume),
    ] {
        put(s, VOLUME, offset, value);
    }
    let releasable: usize = s.processes[..4]
        .iter()
        .map(|p| {
            let paging = p.paging.as_ref().unwrap_or_else(|| fail(b"ADDRESS_SPACE"));
            p.count + p.kernel_frames.len() + paging.owned_table_count()
        })
        .sum();
    s.free_baseline = s
        .free_count()
        .checked_add(releasable)
        .unwrap_or_else(|| fail(b"FRAME_BASELINE"));
    *evidence() = Evidence {
        clients: [NONE; 2],
        baseline: None,
        kernel_tables: s.kernel.owned_table_count(),
    };
    serial::write(b"TASKGRANT:VOLUME_READ_ONLY:PASS\nTASKGRANT:NO_VOLUME_HANDLE:PASS\n");
    serial::write(b"TASKGRANT:MANIFEST:PASS\n");
}

pub(super) fn observe_exit(s: &mut State, slot: usize, _counter: u64) {
    let index = match slot {
        OWNER => 0,
        HELPER => 1,
        _ => {
            print_number(b"TASKGRANT:EXIT_SLOT:", slot as u64);
            print_number(b"TASKGRANT:EXIT_WORD40:", word(s, slot, 40));
            fail(b"UNEXPECTED_EXIT")
        }
    };
    require(!evidence().clients[index].exited, b"DUPLICATE_EXIT");
    let mut statuses = [0u64; 10];
    for (i, status) in statuses.iter_mut().enumerate() {
        *status = word(s, slot, 192 + i as u64 * 8);
    }
    evidence().clients[index] = ClientEvidence {
        exited: true,
        // Captured here: the slot's PID is cleared once exit cleanup runs.
        pid: s.processes[slot].pid,
        code: s.last_event.code,
        statuses,
        content: [word(s, slot, 320), word(s, slot, 328)],
    };
    // FIXTURE lifecycle notice into the broker's private page.
    let notices = word(s, BROKER, 448);
    require(notices < 2, b"EXIT_NOTICES");
    put(s, BROKER, 456 + notices * 8, s.processes[slot].pid);
    put(s, BROKER, 448, notices + 1);
}

pub(super) fn observe_fault(_: &mut State, slot: usize, _: &[u64; 22], _: u64) {
    print_number(b"TASKGRANT:FAULT_SLOT:", slot as u64);
    fail(b"PROCESS_FAULT")
}

fn entry(pid: u64, op: u64, object: u64, status: u64) -> u64 {
    (pid << 32) | (op << 24) | (object << 16) | status
}

pub(super) fn poll(s: &mut State) -> Option<*const u64> {
    m5::broker_tick(s);
    require(!s.unexpected && !s.fault, b"UNEXPECTED_EVENT");
    if s.ticks >= TIMEOUT_TICKS {
        for slot in 0..4 {
            print_number(b"TASKGRANT:TIMEOUT_WORD40:", word(s, slot, 40));
        }
        print_number(b"TASKGRANT:TIMEOUT_VOLUME192:", word(s, VOLUME, 192));
        fail(b"TIMEOUT");
    }
    require(word(s, VOLUME, 192) != 9, b"VOLUME_STARTUP");
    let done = word(s, BROKER, 40);
    if done > 1 {
        print_number(b"TASKGRANT:BROKER_ERROR:", done);
        fail(b"BROKER_ERROR");
    }
    // The broker STATs first and serves no request before this snapshot.
    if word(s, BROKER, BROKER_READY) == 1 && evidence().baseline.is_none() {
        require(word(s, VOLUME, 192) == 1, b"VOLUME_NOT_SERVING");
        let (published, _, _) = m5::broker_summary();
        evidence().baseline = Some((word(s, VOLUME, 344), word(s, VOLUME, 352), published[0]));
        put(s, BROKER, KERNEL_ACK, 1);
    }
    let [owner_evidence, helper_evidence] = evidence().clients;
    if !owner_evidence.exited || !helper_evidence.exited || done != 1 {
        return None;
    }
    let Some((read_sectors, rejected, startup_reads)) = evidence().baseline else {
        fail(b"NO_BASELINE");
    };
    require(
        owner_evidence.code == 0 && helper_evidence.code == 0,
        b"CLIENT_EXIT_CODE",
    );

    let owner = owner_evidence.pid;
    let helper = helper_evidence.pid;
    let granted = if DENIED { NO } else { OK };
    // The broker's own decision log, in serving order (see task_grant.rs).
    let expected = [
        entry(owner, OP_READ, 1, granted),
        entry(owner, OP_WRITE, 1, NO),
        entry(owner, OP_READ, 2, NO),
        entry(helper, OP_READ, 1, NO),
        entry(helper, OP_CLOSE, 1, NO),
        entry(owner, OP_READ, 1, granted),
        entry(owner, OP_CLOSE, 1, granted),
        entry(owner, OP_READ, 1, NO),
        entry(owner, OP_READ, 1, granted),
        entry(helper, OP_RETIRE, 1, NO),
        entry(owner, OP_READ, 1, granted),
        entry(helper, OP_READ, 1, NO),
    ];
    let mut calls = 0u64;
    for (index, value) in expected.into_iter().enumerate() {
        let actual = word(s, BROKER, 256 + index as u64 * 8);
        if actual != value {
            print_number(b"TASKGRANT:LOG_INDEX:", index as u64);
            print_number(b"TASKGRANT:LOG_ACTUAL:", actual);
            print_number(b"TASKGRANT:LOG_EXPECTED:", value);
            fail(b"BROKER_LOG");
        }
        // Volume calls caused by this decision: some only for an authorised
        // READ, none for any denial, CLOSE or WRITE.
        let caused = word(s, BROKER, VOLUME_LOG + index as u64 * 8);
        let read = (value >> 24) & 0xff == OP_READ && value & 0xffff == OK;
        if (caused != 0) != read {
            print_number(b"TASKGRANT:VOLUME_LOG_INDEX:", index as u64);
            print_number(b"TASKGRANT:VOLUME_LOG_CALLS:", caused);
            fail(b"VOLUME_REQUESTS");
        }
        calls += caused;
    }
    require(
        word(s, BROKER, 424) == STARTUP_CALLS
            && word(s, BROKER, 432) == STARTUP_CALLS + calls
            && (calls == 0) == DENIED,
        b"VOLUME_CALLS",
    );
    let (read_ok, denied, close_ok, fixtures, held) = if DENIED {
        (0, 12, 0, 0, 0)
    } else {
        (4, 7, 1, 2, 1)
    };
    require(
        word(s, BROKER, 192) == read_ok
            && word(s, BROKER, 200) == denied
            && word(s, BROKER, 208) == close_ok
            && word(s, BROKER, 216) == 4
            && word(s, BROKER, 224) == 0
            && word(s, BROKER, 144) == fixtures,
        b"BROKER_COUNTERS",
    );
    require(word(s, BROKER, 160) == held, b"FORGED_RETIRE");
    require(
        word(s, BROKER, 152) == 1
            && word(s, BROKER, 232) == held
            && word(s, BROKER, 240) == held
            && word(s, BROKER, 248) == 0
            && word(s, BROKER, 448) == 2
            && word(s, BROKER, 456) == owner
            && word(s, BROKER, 464) == helper,
        b"EXIT_RETIREMENT",
    );
    require(
        owner_evidence.statuses
            == [
                granted, NO, NO, RESUME, granted, granted, NO, granted, RESUME, granted,
            ]
            && helper_evidence.statuses == [GO, NO, NO, GO, NO, GO, NO, 0, 0, 0],
        b"CLIENT_STATUSES",
    );
    let content = if DENIED { [0, 0] } else { content_words() };
    require(
        owner_evidence.content == content && helper_evidence.content == [0, 0],
        b"CONTENT",
    );

    // Volume-side evidence beyond the broker's own claims: the volume's
    // counters (ring 3) and the kernel block broker's own read count.
    let sectors = word(s, VOLUME, 344);
    print_number(b"TASKGRANT:VOLUME_STARTUP_READ_SECTORS:", read_sectors);
    print_number(b"TASKGRANT:VOLUME_READ_SECTORS:", sectors);
    print_number(b"TASKGRANT:VOLUME_CALLS:", word(s, BROKER, 432));
    print_number(b"TASKGRANT:VOLUME_REJECTED:", word(s, VOLUME, 352));
    // Printed before any verdict so a failing run still shows the kernel count.
    let (published, completed, block_idle) = m5::broker_summary();
    print_number(b"TASKGRANT:STARTUP_DISK_READS:", startup_reads);
    print_number(b"TASKGRANT:DISK_READS:", published[0]);
    require(
        word(s, VOLUME, 192) == 1
            && word(s, VOLUME, 376) == 0
            && word(s, VOLUME, 352) == rejected
            && word(s, VOLUME, 304) == 0
            && word(s, VOLUME, 328) == 0
            && word(s, VOLUME, 336) == 0
            && if DENIED {
                sectors == read_sectors
            } else {
                sectors > read_sectors
            },
        b"VOLUME_STATE",
    );
    require(
        block_idle && published == completed && startup_reads > 0 && published[1..] == [0, 0],
        b"BLOCK_READ_ONLY",
    );
    // Kernel-counted: no disk read at all after startup without a grant.
    require(
        if DENIED {
            published[0] == startup_reads
        } else {
            published[0] > startup_reads
        },
        b"DISK_READS_AFTER_STARTUP",
    );

    if DENIED {
        serial::write(b"TASKGRANT:DENIED:PASS\nTASKGRANT:HELPER_DENIED:PASS\n");
    } else {
        serial::write(
            b"TASKGRANT:VOLUME_READ_OK:PASS\nTASKGRANT:OWNER_READ:PASS\n\
TASKGRANT:OWNER_LIMITS:PASS\nTASKGRANT:UNKNOWN_OBJECT_DENIED:PASS\n\
TASKGRANT:FORGED_ACTOR_DENIED:PASS\nTASKGRANT:HELPER_DENIED:PASS\n\
TASKGRANT:CLOSE_REVOKES:PASS\nTASKGRANT:FORGED_RETIRE_DENIED:PASS\n\
TASKGRANT:EXIT_RETIRES:PASS\n",
        );
    }
    serial::write(b"TASKGRANT:DENIED_NO_VOLUME_REQUEST:PASS\n");

    for slot in [BROKER, VOLUME] {
        s.processes[slot].status = Status::Dying;
        s.cleanup(slot);
    }
    let stats = m4::engine().stats();
    require(
        stats.processes == 0
            && stats.handles == 0
            && stats.nodes == 0
            && stats.endpoints == 0
            && stats.queued_messages == 0
            && stats.block_objects == 0
            && m4::engine().check_invariants(),
        b"ENGINE_CLEANUP",
    );
    let grown = s
        .kernel
        .owned_table_count()
        .checked_sub(evidence().kernel_tables)
        .unwrap_or_else(|| fail(b"KERNEL_TABLES"));
    require(
        s.free_count().checked_add(grown) == Some(s.free_baseline),
        b"FRAME_CLEANUP",
    );
    serial::write(b"TASKGRANT:CLEANUP:PASS\n");
    serial::write(if DENIED {
        b"TASKGRANT:CASE:VOLUME_DENIED:PASS\n"
    } else {
        b"TASKGRANT:CASE:VOLUME:PASS\n"
    });
    serial::write(b"TASKGRANT:PASS\nM1:PASS\n");
    serial::exit(0x10)
}

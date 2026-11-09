//! Opt-in native RANDOM acceptance observer only. Bootstrap creates the grants;
//! this module handles explicit test rebind/revoke phases and checks actual
//! service ingress/entropy-output facts. No packet, key or random generation.
use super::*;
use capability_core::{
    Engine, Error as CapError, PROCESS_EXITED, RECV as CAP_RECV, SEND as CAP_SEND,
};
#[path = "../../../../sdk/examples/random-probe/src/contract.rs"]
#[allow(dead_code)]
mod contract;
use contract as c;

pub(super) struct Binding {
    pub case: u64,
    pub key_pid: u64,
    pub probe_pid: u64,
    pub random_root: u64,
    pub send_ancestor: u64,
    pub send_leaf: u64,
    pub reply_receive: u64,
}
struct Evidence {
    binding: Option<Binding>,
    rebind: bool,
    revoked: bool,
    snapshot: Option<[u64; 24]>,
    exit_tick: u64,
    legacy_start: Option<u64>,
    app_pid: u64,
    terminal_tick: Option<u64>,
    app_status: Option<[u64; 3]>,
    app_exit_tick: u64,
}
static mut EVIDENCE: Evidence = Evidence {
    binding: None,
    rebind: false,
    revoked: false,
    snapshot: None,
    exit_tick: 0,
    legacy_start: None,
    app_pid: 0,
    terminal_tick: None,
    app_status: None,
    app_exit_tick: 0,
};
fn evidence() -> &'static mut Evidence {
    // SAFETY: One CPU, IF=0; callbacks are sequential, with no arena borrow
    // retained across a syscall, scheduling boundary or second accessor call.
    unsafe { &mut *ptr::addr_of_mut!(EVIDENCE) }
}
fn require(ok: bool, why: &[u8]) {
    if !ok {
        serial::write(b"SYSTEM:RANDOM:FAIL:");
        serial::write(why);
        serial::write(b"\n");
        serial::exit(0x11)
    }
}
fn slot(s: &State, pid: u64) -> Option<usize> {
    s.processes
        .iter()
        .position(|p| p.pid == pid && p.status != Status::Dead)
}
fn read(s: &State, slot: usize, at: u64) -> u64 {
    read_word(s.processes[slot].physical(DATA + at))
}
fn put(s: &State, slot: usize, at: u64, value: u64) {
    write_word(s.processes[slot].physical(DATA + at), value);
}
fn legacy_progress(s: &State, key: usize, baseline: &mut Option<u64>) -> bool {
    require(read(s, key, 376) == 0, b"LEGACY_POLL_ERRORS");
    let now = read(s, key, 368);
    let Some(start) = *baseline else {
        *baseline = Some(now);
        return false;
    };
    now.checked_sub(start).is_some_and(|delta| delta >= 2)
}
fn terminal(s: &State, app_pid: u64, when: &mut Option<u64>) {
    if when.is_some() {
        return;
    }
    let app = slot(s, app_pid).expect("live legacy app holds its sole session");
    *when = Some(s.ticks);
    put(s, app, c::APP_RELEASE, 1);
}

/// Called after root-owned bootstrap has registered all five initial PIDs,
/// installed only two probe leaves and populated the startup descriptors.
pub(super) fn bind(s: &State, e: &mut Engine, binding: Binding) {
    require(
        binding.case <= c::WRONG_PID && evidence().binding.is_none(),
        b"BIND_ONCE",
    );
    let p = slot(s, binding.probe_pid).unwrap_or_else(|| {
        require(false, b"PROBE_PID");
        0
    });
    let key = slot(s, binding.key_pid).unwrap_or_else(|| {
        require(false, b"KEY_PID");
        0
    });
    require(
        p != key && binding.probe_pid != 0 && binding.key_pid != 0,
        b"DISTINCT_PID",
    );
    require(
        e.available_slots(binding.probe_pid) == capability_core::MAX_SLOTS - 3,
        b"PROBE_ONLY_TWO_CAPS",
    );
    require(
        e.check_endpoint(binding.probe_pid, binding.send_leaf, CAP_SEND)
            .is_ok()
            && e.check_endpoint(binding.probe_pid, binding.reply_receive, CAP_RECV)
                .is_ok()
            && e.check_endpoint(binding.key_pid, binding.random_root, CAP_RECV)
                .is_ok(),
        b"BOUND_ENDPOINTS",
    );
    require(
        read(s, p, c::SEND) == binding.send_leaf
            && read(s, p, c::RECEIVE) == binding.reply_receive
            && read(s, p, c::KEY_PID) == binding.key_pid
            && read(s, p, c::OWN_PID) == binding.probe_pid
            && read(s, p, c::CASE) == binding.case,
        b"PROBE_STARTUP",
    );
    let caller = if binding.case == c::WRONG_PID {
        s.processes[3].pid
    } else {
        binding.probe_pid
    };
    require(
        read(s, key, 120) == u64::from_le_bytes(*b"NOSRNG01")
            && read(s, key, 144) == caller
            && read(s, key, 152) == 0,
        b"SERVICE_STARTUP",
    );
    let app = 3;
    for (at, value) in [
        (c::APP_MAGIC_AT, c::APP_MAGIC),
        (c::APP_RELEASE, 0),
        (c::APP_STATUS_ACK, 0),
        (c::APP_STATUS_BEGIN, 0),
        (c::APP_STATUS_END, 0),
    ] {
        put(s, app, at, value);
    }
    let ev = evidence();
    ev.app_pid = s.processes[app].pid;
    ev.binding = Some(binding);
}
pub(super) fn is_probe(s: &State, slot: usize) -> bool {
    evidence()
        .binding
        .as_ref()
        .is_some_and(|b| s.processes[slot].pid == b.probe_pid)
}
pub(super) fn poll(s: &State) {
    let ev = evidence();
    if ev.snapshot.is_some() {
        return;
    }
    let b = ev
        .binding
        .as_mut()
        .expect("random fixture bound before scheduling");
    let Some(p) = slot(s, b.probe_pid) else {
        return;
    };
    let key = slot(s, b.key_pid).expect("live Key Service");
    let phase = read(s, p, c::REQUEST_PHASE);
    if phase == 0 || read(s, p, c::ACK_PHASE) == phase {
        return;
    }
    let e = m4::engine();
    match phase {
        c::REBIND => {
            require(
                b.case == c::RAW && !ev.rebind && !ev.revoked,
                b"REBIND_ORDER",
            );
            require(
                e.check_endpoint(b.probe_pid, b.send_leaf, CAP_SEND) == Err(CapError::BadHandle),
                b"OLD_SEND_CLOSED",
            );
            require(
                e.available_slots(b.probe_pid) == capability_core::MAX_SLOTS - 2,
                b"UNIQUE_REPLY",
            );
            let new = e
                .bootstrap_delegate(b.key_pid, b.send_ancestor, b.probe_pid, CAP_SEND)
                .expect("bounded SEND rebind");
            require(new != b.send_leaf, b"NEW_SEND_GENERATION");
            b.send_leaf = new;
            ev.rebind = true;
            put(s, p, c::NEW_SEND, new);
        }
        c::CLOSED_REPLY => {
            require(
                b.case == c::RAW && ev.rebind && !ev.revoked,
                b"CLOSED_REPLY_ORDER",
            );
            if read(s, key, 320) != 1 || read(s, key, 328) != 0 || read(s, key, 352) != 1 {
                return;
            }
            require(
                read(s, key, 360) == 0 && read(s, key, 192) == 1,
                b"LEGACY_SURVIVES_CLOSED_REPLY",
            );
            require(
                e.check_endpoint(b.probe_pid, b.reply_receive, CAP_RECV)
                    == Err(CapError::BadHandle),
                b"REPLY_CLOSED",
            );
            require(
                read(s, key, 272) == 90 && read(s, key, 296) == 110,
                b"RAW_TERMINAL_SOURCE",
            );
            terminal(s, ev.app_pid, &mut ev.terminal_tick);
            if !legacy_progress(s, key, &mut ev.legacy_start) {
                return;
            }
        }
        c::REVOKE => {
            require(
                b.case == c::RAW
                    && ev.rebind
                    && !ev.revoked
                    && read(s, p, c::ACK_PHASE) == c::CLOSED_REPLY,
                b"REVOKE_ORDER",
            );
            e.revoke(b.key_pid, b.send_ancestor)
                .expect("fixture SEND subtree revoke");
            require(
                e.check_endpoint(b.probe_pid, b.send_leaf, CAP_SEND) == Err(CapError::Revoked)
                    && e.check_endpoint(b.key_pid, b.random_root, CAP_RECV).is_ok(),
                b"REVOKE_ISOLATION",
            );
            ev.revoked = true;
        }
        c::SETTLE => {
            require(
                matches!(b.case, c::SDK_CANCEL | c::SDK_UNCERTAIN),
                b"SETTLE_CASE",
            );
            let calls = if b.case == c::SDK_CANCEL { 5 } else { 1 };
            if read(s, key, 296) != calls
                || read(s, key, 272) != calls
                || read(s, key, 328) != 0
                || read(s, key, 344) + read(s, key, 352) != calls
            {
                return;
            }
            require(
                e.available_slots(b.probe_pid) == capability_core::MAX_SLOTS - 1,
                b"SDK_RETIRED_BOTH_HANDLES",
            );
            require(
                read(s, key, 344) + read(s, key, 352) == calls
                    && read(s, key, 352) <= 1
                    && read(s, key, 320) == read(s, key, 352)
                    && read(s, key, 360) == 0
                    && read(s, key, 192) == 1,
                b"SETTLED_REPLY_BOUNDARY",
            );
            terminal(s, ev.app_pid, &mut ev.terminal_tick);
            if !legacy_progress(s, key, &mut ev.legacy_start) {
                return;
            }
        }
        c::WRONG_PID_CONSUMED => {
            require(b.case == c::WRONG_PID, b"WRONG_PID_CASE");
            if read(s, key, 296) != 1 || read(s, key, 304) != 1 {
                return;
            }
            require(
                read(s, key, 272) == 0 && read(s, key, 280) == 0 && read(s, key, 288) == 0,
                b"WRONG_PID_NO_RNG",
            );
            terminal(s, ev.app_pid, &mut ev.terminal_tick);
            if !legacy_progress(s, key, &mut ev.legacy_start) {
                return;
            }
        }
        _ => require(false, b"UNKNOWN_PHASE"),
    }
    if matches!(phase, c::CLOSED_REPLY | c::SETTLE | c::WRONG_PID_CONSUMED) {
        // The app owns the sole legacy KEY session. Do not release the probe
        // until its new STATUS roundtrip really completed after terminal RNG
        // handling; a polling heartbeat alone is not the acceptance event.
        let status_ack = ev
            .app_status
            .map(|status| status[0])
            .or_else(|| slot(s, ev.app_pid).map(|app| read(s, app, c::APP_STATUS_ACK)));
        if status_ack != Some(1) {
            return;
        }
    }
    require(e.check_invariants(), b"PHASE_ENGINE");
    put(s, p, c::ACK_PHASE, phase);
}
/// Root system observer calls this before cleanup, and skips its own old
/// init/app worker interpretation if true. PID, not a reusable slot, identifies
/// the probe. No page is read after the process has been cleaned/reused.
pub(super) fn observe_exit(s: &State, slot: usize) -> bool {
    if s.processes[slot].pid == evidence().app_pid {
        if s.last_event.code == 0 && read(s, slot, c::RESULT) == 1 {
            require(s.last_event.pid == s.processes[slot].pid, b"APP_EXIT_PID");
            let status = [
                read(s, slot, c::APP_STATUS_ACK),
                read(s, slot, c::APP_STATUS_BEGIN),
                read(s, slot, c::APP_STATUS_END),
            ];
            let ev = evidence();
            require(ev.app_status.is_none(), b"SINGLE_APP_STATUS");
            ev.app_status = Some(status);
            ev.app_exit_tick = s.ticks;
        }
        return false; // Existing system observer still checks all old evidence.
    }
    if !is_probe(s, slot) {
        return false;
    }
    if s.last_event.pid != s.processes[slot].pid
        || s.last_event.code != 0
        || read(s, slot, c::RESULT) != 1
    {
        print_number(b"SYSTEM:RANDOM_ERROR_STEP:", read(s, slot, c::STEP));
        print_number(b"SYSTEM:RANDOM_ERROR_EXIT:", s.last_event.code);
        print_number(
            b"SYSTEM:RANDOM_ERROR_PHASE:",
            read(s, slot, c::REQUEST_PHASE),
        );
        if evidence()
            .binding
            .as_ref()
            .is_some_and(|b| b.case == c::RAW)
        {
            for (label, at) in c::RAW_DIAGNOSTICS {
                print_number(label, read(s, slot, at));
            }
        }
        require(false, b"PROBE_RESULT");
    }
    let mut words = [0; 24];
    for (i, at) in c::COUNTERS.iter().enumerate() {
        words[i] = read(s, slot, *at);
    }
    let ev = evidence();
    require(ev.snapshot.is_none(), b"SINGLE_EXIT");
    ev.snapshot = Some(words);
    ev.exit_tick = s.ticks;
    m4::engine().process_died(s.processes[slot].pid, PROCESS_EXITED, 0);
    true
}
pub(super) fn finished() -> bool {
    let ev = evidence();
    ev.snapshot.is_some() && ev.app_status.is_some()
}
pub(super) fn check_and_print(s: &State) {
    let ev = evidence();
    let b = ev.binding.as_ref().expect("bound random fixture");
    let a = ev.snapshot.expect("actual random exit");
    let key = slot(s, b.key_pid).expect("service must remain live until final check");
    let status = ev.app_status.expect("actual legacy status RPC");
    let terminal = ev.terminal_tick.expect("actual RNG terminal observation");
    require(
        status[0] == 1
            && terminal <= status[1]
            && status[1] <= status[2]
            && status[2] <= ev.app_exit_tick
            && ev.app_exit_tick <= s.ticks,
        b"LEGACY_RPC_AFTER_RNG",
    );
    print_number(b"SYSTEM:RANDOM_LEGACY_STATUS:", status[0]);
    print_number(b"SYSTEM:RANDOM_TERMINAL_TICKS:", terminal);
    print_number(b"SYSTEM:RANDOM_STATUS_BEGIN_TICKS:", status[1]);
    print_number(b"SYSTEM:RANDOM_STATUS_END_TICKS:", status[2]);
    let mut service = [0; 12];
    for (i, at) in c::SERVICE_WORDS.iter().enumerate() {
        service[i] = read(s, key, *at);
    }
    require(
        a[19] <= a[20] && a[20] <= ev.exit_tick && ev.exit_tick <= s.ticks && a[6] >= 2,
        b"CLOCK_OBSERVATIONS",
    );
    match b.case {
        c::RAW => {
            require(
                a[0..6] == [89, 4737, 5, 9, 9, 1]
                    && a[7..12] == [110, 1, 1, 1, 1]
                    && a[21] == 109
                    && a[22] == 6
                    && ev.rebind
                    && ev.revoked,
                b"RAW_EVIDENCE",
            );
            require(
                service == [90, 4769, 3, 110, 0, 9, 1, 0, 9, 109, 1, 0],
                b"RAW_SERVICE_EVIDENCE",
            );
        }
        c::SDK_CANCEL | c::SDK_UNCERTAIN => {
            let (cancel, unknown, calls, bytes, output, positive) = if b.case == c::SDK_CANCEL {
                (1, 0, 5, 257, 225, 4)
            } else {
                (0, 1, 1, 64, 0, 0)
            };
            require(
                a[7] == calls
                    && a[12..19] == [output, calls, 2, cancel, unknown, 1, 0]
                    && a[23] == positive,
                b"SDK_EVIDENCE",
            );
            require(
                service[0..6] == [calls, bytes, 1, calls, 0, 0]
                    && service[7..9] == [0, 0]
                    && service[9] + service[10] == calls
                    && service[10] <= 1
                    && service[6] == service[10]
                    && service[11] == 0,
                b"SDK_SERVICE_EVIDENCE",
            );
        }
        c::WRONG_PID => {
            require(
                a[7] == 1 && a[18] == 1 && a[21] == 0 && a[20] - a[19] >= c::NO_REPLY_TICKS,
                b"PID_EVIDENCE",
            );
            require(
                service == [0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0],
                b"PID_CONSUMED_AND_REJECTED",
            );
        }
        _ => require(false, b"FINAL_CASE"),
    }
    require(read(s, key, 192) == 1, b"KEY_STILL_READY");
    let legacy_after = read(s, key, 368)
        .checked_sub(ev.legacy_start.expect("terminal RNG observation"))
        .unwrap_or(0);
    require(
        legacy_after >= 2 && read(s, key, 376) == 0,
        b"LEGACY_PROGRESS_AFTER_RNG",
    );
    print_number(b"SYSTEM:RANDOM_LEGACY_POLLS_AFTER:", legacy_after);
    let (entropy, _, idle) = keyring::broker_summary();
    require(
        idle && entropy == 1024 + service[2] * 256 && read(s, key, 264) == entropy,
        b"ACTUAL_ENTROPY_SEEDS",
    );
    for (label, value) in [
        (&b"SYSTEM:RANDOM_PROBE_PID:"[..], b.probe_pid),
        (b"SYSTEM:RANDOM_KEY_PID:", b.key_pid),
        (b"SYSTEM:RANDOM_APP_PID:", ev.app_pid),
        (
            b"SYSTEM:RANDOM_EXPECTED_CALLER_PID:",
            if b.case == c::WRONG_PID {
                ev.app_pid
            } else {
                b.probe_pid
            },
        ),
        (b"SYSTEM:RANDOM_CASE:", b.case),
        (b"SYSTEM:RANDOM_SUCCESSES:", a[0]),
        (b"SYSTEM:RANDOM_BYTES:", a[1]),
        (b"SYSTEM:RANDOM_VARIABLES:", a[2]),
        (b"SYSTEM:RANDOM_LEGACY_DENIED:", a[3]),
        (b"SYSTEM:RANDOM_INVALID_DENIED:", a[4]),
        (b"SYSTEM:RANDOM_QUOTA_DENIED:", a[5]),
        (b"SYSTEM:RANDOM_CLOCK_READS:", a[6]),
        (b"SYSTEM:RANDOM_ENQUEUED:", a[7]),
        (b"SYSTEM:RANDOM_REBINDS:", a[8]),
        (b"SYSTEM:RANDOM_STALE_DENIED:", a[9]),
        (b"SYSTEM:RANDOM_REPLAY_DENIED:", a[10]),
        (b"SYSTEM:RANDOM_REVOKED_DENIED:", a[11]),
        (b"SYSTEM:RANDOM_SDK_BYTES:", a[12]),
        (b"SYSTEM:RANDOM_SDK_SENDS:", a[13]),
        (b"SYSTEM:RANDOM_SDK_CLOSES:", a[14]),
        (b"SYSTEM:RANDOM_CANCELLED:", a[15]),
        (b"SYSTEM:RANDOM_UNKNOWN:", a[16]),
        (b"SYSTEM:RANDOM_NO_RETRY:", a[17]),
        (b"SYSTEM:RANDOM_PID_DENIED:", a[18]),
        (b"SYSTEM:RANDOM_START_TICKS:", a[19]),
        (b"SYSTEM:RANDOM_END_TICKS:", a[20]),
        (b"SYSTEM:RANDOM_REPLIES:", a[21]),
        (b"SYSTEM:RANDOM_CAP_DENIED:", a[22]),
        (b"SYSTEM:RANDOM_SDK_POSITIVES:", a[23]),
        (b"SYSTEM:RANDOM_SERVICE_CALLS:", service[0]),
        (b"SYSTEM:RANDOM_SERVICE_BYTES:", service[1]),
        (b"SYSTEM:RANDOM_SEEDS:", service[2]),
        (b"SYSTEM:RANDOM_INGRESS_SEEN:", service[3]),
        (b"SYSTEM:RANDOM_BAD_CALLER:", service[4]),
        (b"SYSTEM:RANDOM_INVALID_OPCODE:", service[5]),
        (b"SYSTEM:RANDOM_RETIRED:", service[6]),
        (b"SYSTEM:RANDOM_PENDING:", service[7]),
        (b"SYSTEM:RANDOM_INVALID_REQUEST:", service[8]),
        (b"SYSTEM:RANDOM_REPLY_ENQUEUED:", service[9]),
        (b"SYSTEM:RANDOM_REPLY_CLOSED:", service[10]),
        (b"SYSTEM:RANDOM_REPLY_TIMEOUT:", service[11]),
    ] {
        print_number(label, value);
    }
    serial::write(b"SYSTEM:RANDOM:PASS\n");
}

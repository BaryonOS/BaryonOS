//! B01-LOCK (ADR-KEYRING-LOCK-20261126, issue #564): syscall 0x92 LOCK ends the
//! Key Service instance through the kernel's process teardown (KL3) and scrubs
//! the unused service stack (KL4). Only this path sets LOCKED (KL1), and only
//! once the exit-path scrub has read the whole stack back as zero (`latch`;
//! project-owner decision of 26 November 2026: F1 holds only after the stack
//! and buffer erasure is complete). An exit or fault of the instance stays
//! `UNEXPECTED_EXIT`. SUSPEND (KL2) and UNLOCK are separate changes.
use super::*;
#[cfg(feature = "test-system-lock")]
use capability_core::Message;

#[cfg(feature = "native-smp-m3")]
compile_error!("LOCK does not handle a quarantined teardown yet (B01-KEYRING-LOCK §6)");
#[cfg(all(feature = "test-system-lock", feature = "test-system-s3"))]
compile_error!("the LOCK fixture runs on the S5 power flow, not the S3 flow");

/// Key Service startup words that `finish` reads after the instance is gone.
/// Counters and handles only; no key material lives in the startup area.
const KEY_WORDS: [u64; 12] = [48, 192, 224, 232, 240, 248, 256, 384, 392, 400, 408, 416];
/// Usable service stack below `arch::service_stack()`; guard pages bound it.
const STACK_BYTES: u64 = 32768;
#[cfg(feature = "test-system-lock")]
const TAG: u64 = 0x4b45_5931;
/// Labelled residue canary (A10). It travels only in the request id of one
/// KEY1 administrative request; the Key Service echoes it in its reply.
#[cfg(feature = "test-system-lock")]
const CANARY: [u8; 8] = *b"B01LKCNY";
/// A6 app startup words (system app `after_lock`): the kernel's probe and go
/// flags, then the app's send and receive codes on its KEY1 endpoint pair.
#[cfg(feature = "test-system-lock")]
pub(super) const APP_PROBE: u64 = 824;
#[cfg(feature = "test-system-lock")]
const APP_GO: u64 = 832;
#[cfg(feature = "test-system-lock")]
const APP_CODES: [u64; 2] = [840, 848];
/// A6: teardown revoked the app's SEND handle to the dead instance (-10), and
/// no reply is queued for it (-8).
#[cfg(feature = "test-system-lock")]
const APP_REFUSED: [u64; 2] = [10, 8];

// Read and written by `m3_restore` (arch.rs) only when this module is built.
#[no_mangle]
static mut m3_scrub_pending: u8 = 0;
#[no_mangle]
static mut m3_scrubs: u64 = 0;
#[no_mangle]
static mut m3_scrub_residue: u64 = 0;

struct LockState {
    /// The instance is gone (teardown ran); LOCK is irreversible from here.
    gone: bool,
    /// LOCKED: `gone` and the exit-path stack scrub read back zero (F1).
    locked: bool,
    locks: u64,
    log: [i64; 8],
    logged: usize,
    dropped: u64,
    words: [u64; KEY_WORDS.len()],
    entropy: (usize, usize),
    /// Canary copies on the whole service stack at LOCK and one tick later.
    canary: (u64, u64),
    #[cfg(feature = "test-system-lock")]
    owned: [u64; FRAME_COUNT.div_ceil(64)],
    #[cfg(feature = "test-system-lock")]
    frames: u64,
    /// A6: the app's send and receive codes after LOCK, as magnitudes.
    #[cfg(feature = "test-system-lock")]
    app: [u64; 2],
    #[cfg(feature = "test-system-lock")]
    fixture: Fixture,
}
#[cfg(feature = "test-system-lock")]
#[derive(Clone, Copy)]
struct Fixture {
    stage: u8,
    holds: u64,
    deadline: u64,
    done: bool,
}
static mut LOCK_STATE: LockState = LockState {
    gone: false,
    locked: false,
    locks: 0,
    log: [0; 8],
    logged: 0,
    dropped: 0,
    words: [0; KEY_WORDS.len()],
    entropy: (0, 0),
    canary: (0, 0),
    #[cfg(feature = "test-system-lock")]
    owned: [0; FRAME_COUNT.div_ceil(64)],
    #[cfg(feature = "test-system-lock")]
    frames: 0,
    #[cfg(feature = "test-system-lock")]
    app: [0; 2],
    #[cfg(feature = "test-system-lock")]
    fixture: Fixture {
        stage: 0,
        holds: 0,
        deadline: 0,
        done: false,
    },
};
fn state() -> &'static mut LockState {
    // SAFETY: One CPU (native-smp-m3 is excluded), IF=0, KERNEL held. Each
    // borrow ends before any call that takes another one (`word` → `gone`,
    // `lock`, `live`, `erased`), and before a syscall or scheduling.
    unsafe { &mut *ptr::addr_of_mut!(LOCK_STATE) }
}

pub(super) fn locked() -> bool {
    state().locked
}
pub(super) fn gone() -> bool {
    state().gone
}
/// F1 for SUSPEND (KL2, H05-T5d): LOCKED, every requested scrub ran and read
/// back zero, and the entropy broker holds no bytes. Frame zeroing (`cleanup`'s
/// owner assert) and an empty queue from the instance (-8) hold before LOCK.
pub(crate) fn erased() -> bool {
    let (scrubs, residue, pending) = scrub_result();
    let st = state();
    kernel_core::keylock::erased(kernel_core::keylock::Erasure {
        locked: st.locked,
        locks: st.locks,
        scrubs,
        residue,
        scrub_pending: pending != 0,
        broker_quiet: keyring::broker_quiet(),
    })
}
/// Every event: after a LOCK, LOCKED latches only once `m3_restore` scrubbed the
/// whole service stack and read it back as zero; a LOCK without that halts.
pub(super) fn latch() {
    let st = state();
    // A ring-3 0x92 reaches this in its own kernel entry, before m3_restore has
    // run the scrub; LOCKED (and so `erased`) stays false until a later entry.
    if !st.gone || st.locked || scrub_result().2 != 0 {
        return;
    }
    #[cfg(feature = "test-system-lock")]
    {
        // A10: no canary copy is left anywhere on the service stack.
        st.canary.1 = count();
        require(st.canary.1 == 0, b"LOCK_CANARY_RESIDUE");
    }
    require(scrub_result() == (1, 0, 0), b"LOCK_STACK_RESIDUE");
    st.locked = true;
    serial::write(b"SYSTEM:LOCK:LOCKED\n");
}
/// `word(KEY, ..)` after LOCK: the value the instance held at teardown.
pub(super) fn key_word(offset: u64) -> u64 {
    let index = KEY_WORDS.iter().position(|known| *known == offset);
    state().words[index.unwrap_or_else(|| fail(b"LOCK_WORD"))]
}

/// Syscall 0x92 LOCK(power handle, 0, 0, 0, 0, 0), init only. The reserved
/// arguments (-1) and the POWER authority (-6, -4) are checked before any
/// state; then an earlier LOCK (-14, irreversible) and the side-effect-free retry
/// preconditions (-8): no entropy request pending or answered, and no message
/// from the instance still queued anywhere.
pub(crate) fn lock(s: &mut State, slot: usize, a: [u64; 6]) -> i64 {
    let code = if a[1..].iter().any(|value| *value != 0) {
        -1
    } else if let Err(error) = m4::engine().check_power(s.processes[slot].pid, a[0]) {
        error.code()
    } else if state().gone {
        -14
    } else if !keyring::broker_quiet() || m4::engine().queued_from(s.processes[KEY].pid) != 0 {
        -8
    } else {
        0
    };
    if code != 0 {
        let st = state();
        if st.logged < st.log.len() {
            st.log[st.logged] = code;
            st.logged += 1;
        } else {
            st.dropped = st.dropped.saturating_add(1);
        }
        return code;
    }
    teardown(s);
    0
}

fn teardown(s: &mut State) {
    require(
        !matches!(s.processes[KEY].status, Status::Dead | Status::Dying),
        b"LOCK_NO_INSTANCE",
    );
    // Locals first: `word` reads `gone()`, so no LOCK_STATE borrow may span it.
    let canary = count();
    let words = KEY_WORDS.map(|offset| word(s, KEY, offset));
    #[cfg(feature = "test-system-lock")]
    record_frames(s);
    let before = m4::engine().stats().entropy_objects;
    // KL3: the teardown `finish` uses; `cleanup` asserts that no frame or page
    // table still names the instance as owner and zeroes every freed frame.
    s.processes[KEY].status = Status::Dying;
    s.cleanup(KEY);
    let after = m4::engine().stats().entropy_objects;
    require(
        s.processes[KEY].status == Status::Dead && after < before,
        b"LOCK_TEARDOWN",
    );
    let st = state();
    st.canary.0 = canary;
    st.words = words;
    st.entropy = (before, after);
    // KL4: `m3_restore` zeroes and verifies the whole service stack before the
    // next ring-3 instruction. The noscrub red control omits only this request.
    #[cfg(not(feature = "test-system-lock-noscrub"))]
    // SAFETY: One CPU, IF=0; m3_restore reads it only after this handler.
    unsafe {
        ptr::addr_of_mut!(m3_scrub_pending).write(1)
    };
    st.gone = true;
    st.locks += 1;
    #[cfg(feature = "test-system-lock")]
    check_frames();
}

/// Canary copies on the whole service stack, at any byte offset; 0 outside
/// the fixture.
#[inline(never)]
fn count() -> u64 {
    #[cfg(feature = "test-system-lock")]
    {
        let top = arch::service_stack();
        let mut found = 0;
        let mut at = top - STACK_BYTES;
        while at + 8 <= top {
            // SAFETY: inside the mapped service stack; [u8; 8] has alignment 1.
            if unsafe { ptr::read_volatile(at as *const [u8; 8]) } == CANARY {
                found += 1;
            }
            at += 1;
        }
        found
    }
    #[cfg(not(feature = "test-system-lock"))]
    0
}

/// The exit-path scrub ran once and read back zero (`m3_restore`).
fn scrub_result() -> (u64, u64, u8) {
    // SAFETY: One CPU, IF=0; m3_restore writes these only outside Rust.
    unsafe {
        (
            ptr::addr_of!(m3_scrubs).read(),
            ptr::addr_of!(m3_scrub_residue).read(),
            ptr::addr_of!(m3_scrub_pending).read(),
        )
    }
}

/// A4 (test only): every frame the instance owned, its kernel stack and its
/// page tables, recorded before teardown and read back as zero after it.
#[cfg(feature = "test-system-lock")]
fn record_frames(s: &State) {
    let st = state();
    st.owned.fill(0);
    let pid = s.processes[KEY].pid;
    let table = Owner::PageTable(u16::try_from(KEY + 1).unwrap());
    for index in 0..FRAME_COUNT {
        let owner = s.frames.owner(index as u64 * PAGE).unwrap();
        if matches!(owner, Owner::User(owner_pid, _) if owner_pid == pid) || owner == table {
            st.owned[index / 64] |= 1 << (index % 64);
        }
    }
    for pa in s.processes[KEY].kernel_frames {
        let index = (pa / PAGE) as usize;
        st.owned[index / 64] |= 1 << (index % 64);
    }
}
#[cfg(feature = "test-system-lock")]
fn check_frames() {
    let st = state();
    st.frames = 0;
    for index in 0..FRAME_COUNT {
        if st.owned[index / 64] & (1 << (index % 64)) == 0 {
            continue;
        }
        let zero = paging::physical_read(index as u64 * PAGE, |page| {
            (0..PAGE as usize)
                // SAFETY: physical_read maps one whole frame for this closure.
                .all(|offset| unsafe { page.add(offset).read_volatile() } == 0)
        });
        require(zero, b"LOCK_FRAME_RESIDUE");
        st.frames += 1;
    }
    require(st.frames != 0, b"LOCK_NO_FRAMES");
}

/// Test only: from the first grant on, init's grant is held (-8, not logged)
/// until the fixture is done; that includes the storage gate's -8 while the
/// fixture's own entropy request keeps the broker busy.
#[cfg(feature = "test-system-lock")]
pub(super) fn hold(code: i64) -> bool {
    let fixture = &mut state().fixture;
    let held = !fixture.done && (code == 0 || code == -8 && fixture.holds != 0);
    if held {
        fixture.holds += 1;
    }
    held
}

#[cfg(feature = "test-system-lock")]
fn live(s: &State) -> bool {
    !state().gone && !matches!(s.processes[KEY].status, Status::Dead | Status::Dying)
}

/// Labelled native fixture (TEST profile only), driven by the real 0x92
/// handler on init's behalf once the app is done and init retries its S5
/// grant: A9 authority, A2 pending entropy, A3 queued reply, A4/A10 LOCK, A6
/// clients after LOCK (init's ingress and the app's own KEY1 endpoint) and the
/// irreversible -14. The app parks after its DONE until the kernel's go word.
#[cfg(feature = "test-system-lock")]
pub(super) fn fixture(s: &mut State) {
    // A copy: `lock`, `live` and `erased` take their own LOCK_STATE borrows.
    let f = state().fixture;
    if f.done || f.holds == 0 {
        return;
    }
    let power = word(s, INIT, 48);
    let grant = [power, 0, 0, 0, 0, 0];
    match f.stage {
        #[cfg(feature = "test-system-lock-exit")]
        0 => {
            // A5 red control: the instance exits on its own; that must end in
            // UNEXPECTED_EXIT and never count as LOCKED.
            put(s, KEY, 960, 1);
            state().fixture.stage = 9;
        }
        #[cfg(not(feature = "test-system-lock-exit"))]
        0 => {
            let image = word(s, INIT, 56);
            let forged = power + (1 << 32);
            require(
                lock(s, INIT, [power, 1, 0, 0, 0, 0]) == -1
                    && lock(s, INIT, [image, 0, 0, 0, 0, 0]) == -6
                    && lock(s, INIT, [forged, 0, 0, 0, 0, 0]) == -4
                    && live(s),
                b"LOCK_AUTHORITY",
            );
            require(keyring::broker_quiet(), b"LOCK_BROKER_BUSY");
            let request = [word(s, KEY, 48), 256, 0, 0, 0, 0];
            let (ticket, _) = keyring::syscall(s, KEY, 0x60, request);
            require(ticket >= 0, b"LOCK_ENTROPY_REQUEST");
            require(
                lock(s, INIT, grant) == -8 && live(s) && !keyring::broker_quiet(),
                b"LOCK_ENTROPY_PENDING",
            );
            state().fixture.deadline = s.ticks + 4096;
            state().fixture.stage = 1;
        }
        1 => {
            // Answered, then reclaimed after its lease; never latched (-14).
            if !keyring::broker_quiet() {
                require(s.ticks < f.deadline, b"LOCK_BROKER_STUCK");
                return;
            }
            require(keyring::broker_summary().2, b"LOCK_ENTROPY_LATCH");
            require(admin_request(s), b"LOCK_CANARY_SEND");
            state().fixture.deadline = s.ticks + 4096;
            state().fixture.stage = 2;
        }
        2 => {
            if m4::engine().queued_from(s.processes[KEY].pid) == 0 {
                require(s.ticks < f.deadline, b"LOCK_NO_REPLY");
                return;
            }
            require(lock(s, INIT, grant) == -8 && live(s), b"LOCK_REPLY_QUEUED");
            require(
                drain_reply(s) && m4::engine().queued_from(s.processes[KEY].pid) == 0,
                b"LOCK_CANARY_REPLY",
            );
            state().fixture.stage = 3;
        }
        3 => {
            // A later tick, so the drain's frames are no longer live. The
            // instance is gone, but LOCKED waits for the exit-path scrub.
            require(lock(s, INIT, grant) == 0, b"LOCK_GRANT");
            // The ring-3 0x92 order: the handler, then `poll` → `latch` in the
            // same kernel entry, before m3_restore scrubs. It must defer.
            latch();
            require(state().gone && !state().locked && !erased(), b"LOCK_GRANT");
            require(!admin_request(s) && !drain_reply(s), b"LOCK_CLIENT_SERVED");
            require(lock(s, INIT, grant) == -14, b"LOCK_NOT_IRREVERSIBLE");
            require(state().canary.0 >= 1, b"LOCK_CANARY_ABSENT");
            state().fixture.stage = 4;
        }
        4 => {
            // One event later `latch` found the stack scrubbed (A10, F1).
            require(state().locked && erased(), b"LOCK_NOT_ERASED");
            put(s, APP, APP_GO, 1);
            state().fixture.deadline = s.ticks + 4096;
            state().fixture.stage = 5;
        }
        5 => {
            // A6: the app asked the dead instance to unseal its record.
            if evidence().app.is_none() {
                require(s.ticks < f.deadline, b"LOCK_APP_STUCK");
                return;
            }
            require(state().app == APP_REFUSED, b"LOCK_APP_SERVED");
            state().fixture.done = true;
        }
        _ => {}
    }
}

/// A6, at the app's exit: its codes on the KEY1 endpoint pair after LOCK.
#[cfg(feature = "test-system-lock")]
pub(super) fn app_exit(s: &State) {
    state().app = APP_CODES.map(|at| (word(s, APP, at) as i64).unsigned_abs());
}

/// One KEY1 administrative request from init with an unknown opcode and the
/// canary as request id; the Key Service refuses it and echoes the id.
#[cfg(feature = "test-system-lock")]
#[inline(never)]
fn admin_request(s: &State) -> bool {
    let mut bytes = [0u8; 320];
    bytes[8..16].copy_from_slice(&TAG.to_le_bytes());
    bytes[16..20].copy_from_slice(&256u32.to_le_bytes());
    bytes[64] = 0xee;
    bytes[68..76].copy_from_slice(&CANARY);
    let sent = match Message::decode(&bytes) {
        Ok(mut message) => {
            let sent = m4::engine()
                .send(s.processes[INIT].pid, word(s, INIT, 104), &message)
                .is_ok();
            message.data.fill(0);
            core::hint::black_box(&message);
            sent
        }
        Err(_) => false,
    };
    bytes.fill(0);
    core::hint::black_box(&bytes);
    sent
}
#[cfg(feature = "test-system-lock")]
#[inline(never)]
fn drain_reply(s: &State) -> bool {
    let Ok(mut reply) = m4::engine().receive(s.processes[INIT].pid, word(s, INIT, 112)) else {
        return false;
    };
    // A refusal: the echoed request id and a non-zero status.
    let ok = reply.tag == TAG
        && reply.len == 256
        && reply.data[0] == 0xee
        && reply.data[4..12] == CANARY
        && reply.data[12..16] != [0; 4];
    reply.data.fill(0);
    core::hint::black_box(&reply);
    ok
}

/// `finish`: LOCK evidence. The fixture's refusal log is exact.
pub(super) fn check_and_print() {
    let erased = erased();
    let st = state();
    require(!st.gone || erased, b"LOCK_NOT_ERASED");
    #[cfg(feature = "test-system-lock")]
    require(
        st.fixture.done
            && st.locks == 1
            && st.locked
            && scrub_result() == (1, 0, 0)
            && st.app == APP_REFUSED
            && st.log[..st.logged] == [-1, -6, -4, -8, -8, -14]
            && st.dropped == 0,
        b"LOCK_EVIDENCE",
    );
    for (label, value) in [
        (&b"SYSTEM:LOCK_COUNT:"[..], st.locks),
        (b"SYSTEM:LOCK_REFUSALS:", st.logged as u64),
        (b"SYSTEM:LOCK_STACK_SCRUBS:", scrub_result().0),
        (b"SYSTEM:LOCK_STACK_RESIDUE:", scrub_result().1),
        (b"SYSTEM:LOCK_CANARY_BEFORE:", st.canary.0),
        (b"SYSTEM:LOCK_CANARY_AFTER:", st.canary.1),
        (b"SYSTEM:LOCK_ENTROPY_OBJECTS_BEFORE:", st.entropy.0 as u64),
        (b"SYSTEM:LOCK_ENTROPY_OBJECTS_AFTER:", st.entropy.1 as u64),
    ] {
        print_number(label, value);
    }
    #[cfg(feature = "test-system-lock")]
    {
        print_number(b"SYSTEM:LOCK_FRAMES_ZEROED:", st.frames);
        print_number(b"SYSTEM:LOCK_GRANT_HOLDS:", st.fixture.holds);
        print_number(b"SYSTEM:LOCK_APP_SEND:", st.app[0]);
        print_number(b"SYSTEM:LOCK_APP_RECEIVE:", st.app[1]);
        serial::write(b"SYSTEM:LOCK:PASS\n");
    }
}

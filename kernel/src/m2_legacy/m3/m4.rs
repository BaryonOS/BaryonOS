//! M4 syscall boundary and synthetic ring-3 evidence. The authority engine never
//! receives user pointers. All copies use prevalidated fixed-size byte plans.
use super::*;
#[cfg(feature = "native-smp-m4")]
use crate::m2::{ap, smp};
use capability_core::{
    AuthoritySnapshot, Engine, Error, Message, SnapshotLease, SnapshotLock, RECV, SEND,
};
#[cfg(all(
    not(feature = "milestone-m5"),
    not(feature = "native-qsim-job"),
    not(feature = "native-sdk-app"),
    not(feature = "native-supervisor"),
    not(feature = "native-keyring"),
    not(feature = "native-display-probe"),
    not(feature = "native-fpu")
))]
use capability_core::{Cap, ALL_RIGHTS};
#[cfg(feature = "native-smp-m4")]
use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use kernel_core::user::{CopyPlan, PageAccess};
#[path = "snapshot_wipe.rs"]
mod snapshot_wipe;
#[cfg(all(
    not(feature = "milestone-m5"),
    not(feature = "native-qsim-job"),
    not(feature = "native-sdk-app"),
    not(feature = "native-supervisor"),
    not(feature = "native-keyring"),
    not(feature = "native-display-probe"),
    not(feature = "native-fpu")
))]
const MSG: u64 = 0x40011000;
#[cfg(all(
    not(feature = "milestone-m5"),
    not(feature = "native-qsim-job"),
    not(feature = "native-sdk-app"),
    not(feature = "native-supervisor"),
    not(feature = "native-keyring"),
    not(feature = "native-display-probe"),
    not(feature = "native-fpu")
))]
const OUT: u64 = 0x40012000;
#[cfg(all(
    not(feature = "milestone-m5"),
    not(feature = "native-qsim-job"),
    not(feature = "native-sdk-app"),
    not(feature = "native-supervisor"),
    not(feature = "native-keyring"),
    not(feature = "native-display-probe"),
    not(feature = "native-fpu")
))]
const ROWS: u64 = DATA + 0x100;
static mut ENGINE: Engine = Engine::new();
#[cfg(all(
    not(feature = "milestone-m5"),
    not(feature = "native-qsim-job"),
    not(feature = "native-sdk-app"),
    not(feature = "native-supervisor"),
    not(feature = "native-keyring"),
    not(feature = "native-display-probe"),
    not(feature = "native-fpu")
))]
struct Evidence {
    calls: u64,
    success: u64,
    errors: [u64; 12],
    send: [u64; 3],
    recv: [u64; 3],
    again: [u64; 3],
    revoke: [u64; 3],
    close: [u64; 3],
    derive: [u64; 3],
    hook: bool,
    started: bool,
    survivor: u64,
    expected_exit: bool,
    expected_fault: bool,
}
#[cfg(all(
    not(feature = "milestone-m5"),
    not(feature = "native-qsim-job"),
    not(feature = "native-sdk-app"),
    not(feature = "native-supervisor"),
    not(feature = "native-keyring"),
    not(feature = "native-display-probe"),
    not(feature = "native-fpu")
))]
const EMPTY_EVIDENCE: Evidence = Evidence {
    calls: 0,
    success: 0,
    errors: [0; 12],
    send: [0; 3],
    recv: [0; 3],
    again: [0; 3],
    revoke: [0; 3],
    close: [0; 3],
    derive: [0; 3],
    hook: false,
    started: false,
    survivor: 0,
    expected_exit: false,
    expected_fault: false,
};
#[cfg(all(
    not(feature = "milestone-m5"),
    not(feature = "native-qsim-job"),
    not(feature = "native-sdk-app"),
    not(feature = "native-supervisor"),
    not(feature = "native-keyring"),
    not(feature = "native-display-probe"),
    not(feature = "native-fpu")
))]
static mut EVIDENCE: Evidence = EMPTY_EVIDENCE;
#[cfg(all(
    not(feature = "milestone-m5"),
    not(feature = "native-qsim-job"),
    not(feature = "native-sdk-app"),
    not(feature = "native-supervisor"),
    not(feature = "native-keyring"),
    not(feature = "native-display-probe"),
    not(feature = "native-fpu")
))]
static mut LIFECYCLE_ROUNDS: u8 = 0;
#[cfg(feature = "native-smp-m4")]
const QUIESCE_NONE: u8 = 0;
#[cfg(feature = "native-smp-m4")]
const QUIESCE_ROUND_RESET: u8 = 1;
#[cfg(feature = "native-smp-m4")]
const QUIESCE_FINAL_CLEANUP: u8 = 2;
#[cfg(feature = "native-smp-m4")]
const QUIESCE_POLLS: usize = 100_000_000;
#[cfg(feature = "native-smp-m4")]
static QUIESCE_ACTION: AtomicU8 = AtomicU8::new(QUIESCE_NONE);
#[cfg(feature = "native-smp-m4")]
static QUIESCED_CPUS: AtomicUsize = AtomicUsize::new(0);

#[cfg(feature = "native-smp-m4")]
pub(super) fn quiescence_requested() -> bool {
    QUIESCE_ACTION.load(Ordering::Acquire) != QUIESCE_NONE
}

#[cfg(feature = "native-smp-m4")]
pub(super) fn mark_quiesced(cpu: usize) {
    assert!(cpu < super::SMP_CPUS);
    QUIESCED_CPUS.fetch_or(1 << cpu, Ordering::AcqRel);
}

#[cfg(feature = "native-smp-m4")]
fn request_quiescence(action: u8) {
    let bootstrap = smp::bootstrap_index();
    assert_eq!(super::arch::current_cpu_index(), bootstrap);
    assert_ne!(action, QUIESCE_NONE);
    QUIESCED_CPUS.store(1 << bootstrap, Ordering::Relaxed);
    if QUIESCE_ACTION
        .compare_exchange(QUIESCE_NONE, action, Ordering::Release, Ordering::Relaxed)
        .is_err()
    {
        serial::fail(b"M4:SMP:QUIESCE_ALREADY_ACTIVE");
    }
    smp::schedule_aps();
}

#[cfg(feature = "native-smp-m4")]
pub(super) fn wait_for_quiescence() {
    let cpu_count = ap::cpu_count();
    let expected = (1 << cpu_count) - 1;
    for _ in 0..QUIESCE_POLLS {
        if QUIESCED_CPUS.load(Ordering::Acquire) == expected {
            return;
        }
        core::hint::spin_loop();
    }
    serial::fail(b"M4:SMP:QUIESCE_TIMEOUT");
}
pub(super) fn engine() -> &'static mut Engine {
    // SAFETY: Setup uses the BSP before AP release; afterward M3Guard serializes
    // all engine access across CPUs with IF clear. No nested calls or borrows.
    unsafe { &mut *ptr::addr_of_mut!(ENGINE) }
}
static SNAPSHOT_LOCK: SnapshotLock = SnapshotLock::new();
static mut AUTHORITY_SNAPSHOT: AuthoritySnapshot = AuthoritySnapshot::new();

pub(super) struct AuthorityObservation {
    snapshot: &'static mut AuthoritySnapshot,
    // Declared after storage so release happens only after Drop's message wipe.
    _lease: SnapshotLease<'static>,
}
fn require_snapshot_if0() {
    let flags: u64;
    // SAFETY: CPL0 read-only flags observation, stack restored before returning.
    // No user pointer or memory reference crosses this check.
    unsafe {
        asm!("pushfq", "pop {}", out(reg) flags, options(preserves_flags));
    }
    assert_eq!(flags & (1 << 9), 0, "authority observation requires IF=0");
}
pub(super) fn observe_authority() -> AuthorityObservation {
    require_snapshot_if0();
    let lease = SNAPSHOT_LOCK
        .try_claim()
        .expect("nested authority observation");
    // SAFETY: Claim precedes constructing the unique static-buffer reference.
    // M3Guard serializes engine snapshots across CPUs; IF is clear. The snapshot
    // lease prevents nested access to this static buffer until its message wipe.
    // The live Engine is distinct storage;
    // its short borrow ends here, before dispatch or any scheduling.
    let snapshot = unsafe { &mut *ptr::addr_of_mut!(AUTHORITY_SNAPSHOT) };
    engine().snapshot_into(snapshot);
    AuthorityObservation {
        snapshot,
        _lease: lease,
    }
}
impl AuthorityObservation {
    pub(super) fn matches_current(&self) -> bool {
        require_snapshot_if0();
        self.snapshot.matches(engine())
    }
}
impl Drop for AuthorityObservation {
    fn drop(&mut self) {
        require_snapshot_if0();
        for message in self.snapshot.messages_mut() {
            snapshot_wipe::wipe_message(message);
        }
    }
}
#[cfg(all(
    not(feature = "milestone-m5"),
    not(feature = "native-qsim-job"),
    not(feature = "native-sdk-app"),
    not(feature = "native-supervisor"),
    not(feature = "native-keyring"),
    not(feature = "native-display-probe"),
    not(feature = "native-fpu")
))]
fn evidence() -> &'static mut Evidence {
    // SAFETY: Same serialized service-stack lifetime as Engine; no IRQ mutates it.
    unsafe { &mut *ptr::addr_of_mut!(EVIDENCE) }
}
pub(super) fn plan(
    state: &State,
    slot: usize,
    pointer: u64,
    length: u64,
    output: bool,
) -> Result<CopyPlan, Error> {
    let p = &state.processes[slot];
    user::copy_plan(pointer, length, p.pid, output, |va| {
        let (physical, writable, _, user) = p.paging.as_ref()?.permissions(va)?;
        let page = physical & !(PAGE - 1);
        let Owner::User(owner, _) = state.frames.owner(page).ok()? else {
            return None;
        };
        Some(PageAccess {
            physical: page,
            owner,
            user,
            writable,
        })
    })
    .map_err(|_| Error::Fault)
}
pub(super) fn read_plan(plan: &CopyPlan) -> [u8; 320] {
    let mut bytes = [0; 320];
    let mut at = 0;
    for chunk in &plan.chunks[..plan.count] {
        paging::physical_read(chunk.physical, |source| {
            for i in 0..chunk.length {
                // SAFETY: Every byte belongs to the validated caller-owned page;
                // a temporary RO/NX scratch alias cannot violate text W^X.
                bytes[at + i] = unsafe { source.add(chunk.offset + i).read_volatile() };
            }
        });
        at += chunk.length;
    }
    bytes
}
pub(super) fn write_plan(plan: &CopyPlan, bytes: &[u8; 320]) {
    let mut at = 0;
    for chunk in &plan.chunks[..plan.count] {
        paging::physical(chunk.physical, |dest| {
            for i in 0..chunk.length {
                // SAFETY: All output pages were validated writable/owned before
                // engine commit. Setup is BSP-only before AP release; runtime
                // M3 events hold M3Guard with IF clear. No shared-memory/DMA API.
                unsafe {
                    dest.add(chunk.offset + i).write_volatile(bytes[at + i]);
                }
            }
        });
        at += chunk.length;
    }
}
fn dispatch(
    state: &mut State,
    slot: usize,
    number: u64,
    args: [u64; 6],
    output_before: &mut Option<(CopyPlan, [u8; 320])>,
) -> Result<u64, Error> {
    let used = match number {
        0x10 | 0x12 => 1,
        0x11 => 2,
        0x20 | 0x21 => 3,
        _ => return Err(Error::NoSys),
    };
    if args[used..].iter().any(|a| *a != 0) {
        return Err(Error::Invalid);
    }
    let pid = state.processes[slot].pid;
    match number {
        0x10 => engine().close(pid, args[0]).map(|_| 0),
        0x11 => engine().derive(pid, args[0], args[1]),
        0x12 => engine().revoke(pid, args[0]).map(|_| 0),
        0x20 | 0x21 => {
            if args[2] != 320 {
                return Err(Error::MessageSize);
            }
            engine().check_endpoint(pid, args[0], if number == 0x20 { SEND } else { RECV })?;
            if number == 0x21 {
                *output_before = plan(state, slot, args[1], 320, false)
                    .or_else(|_| {
                        plan(
                            state,
                            slot,
                            args[1],
                            core::cmp::min(320, PAGE - (args[1] & (PAGE - 1))),
                            false,
                        )
                    })
                    .ok()
                    .map(|p| {
                        let bytes = read_plan(&p);
                        (p, bytes)
                    });
            }
            let copy = plan(state, slot, args[1], 320, number == 0x21)?;
            if number == 0x20 {
                let snapshot = read_plan(&copy);
                // Deliberate rewrite exists only in the snapshot-test binary.
                #[cfg(feature = "test-m4-snapshot")]
                if slot == 0 && !evidence().hook {
                    let writable = plan(state, slot, args[1], 320, true)?;
                    let mut changed = snapshot;
                    changed[16..20].copy_from_slice(&257u32.to_le_bytes());
                    changed[64..72].fill(0xcc);
                    write_plan(&writable, &changed);
                    evidence().hook = true;
                }
                let message = Message::decode(&snapshot)?;
                engine().send(pid, args[0], &message)?;
                #[cfg(all(feature = "milestone-m5", not(feature = "milestone-m6a")))]
                super::m5::sent(state, slot, &message);
                #[cfg(all(feature = "milestone-m6a", not(feature = "native-system")))]
                super::m5::sent(state, slot, args[0], &message);
                Ok(0)
            } else {
                // Full output validation precedes all node/slot/queue commits.
                let message = engine().receive(pid, args[0])?;
                let length = u64::from(message.len);
                let bytes = message.encode();
                write_plan(&copy, &bytes);
                assert_eq!(read_plan(&copy), bytes, "complete receive output mismatch");
                assert!(bytes[64 + length as usize..].iter().all(|byte| *byte == 0));
                #[cfg(all(feature = "milestone-m6a", not(feature = "native-system")))]
                super::m5::received(state, slot, args[0], &message);
                Ok(length)
            }
        }
        _ => unreachable!(),
    }
}
pub fn syscall(state: &mut State, slot: usize, number: u64, args: [u64; 6]) -> (i64, u64) {
    let before = observe_authority();
    // On cross-page failure retain evidence for every readable output-prefix
    // byte, including a read-only second page. Nothing is copied to output yet.
    let mut output_before = None;
    let result = dispatch(state, slot, number, args, &mut output_before);
    let unchanged = result.is_ok() || before.matches_current();
    // Do not retain snapshot contents/lease through evidence observers or the
    // scheduler. Drop observably wipes every Message before releasing storage.
    drop(before);
    assert!(unchanged, "failed syscall changed authority/queue state");
    #[cfg(all(
        not(feature = "milestone-m5"),
        not(feature = "native-qsim-job"),
        not(feature = "native-sdk-app"),
        not(feature = "native-supervisor"),
        not(feature = "native-keyring"),
        not(feature = "native-display-probe"),
        not(feature = "native-fpu")
    ))]
    let e = evidence();
    #[cfg(all(
        not(feature = "milestone-m5"),
        not(feature = "native-qsim-job"),
        not(feature = "native-sdk-app"),
        not(feature = "native-supervisor"),
        not(feature = "native-keyring"),
        not(feature = "native-display-probe"),
        not(feature = "native-fpu")
    ))]
    {
        e.calls += 1;
    }
    match result {
        Ok(value) => {
            #[cfg(feature = "milestone-m5")]
            super::m5::cap_event(slot, number);
            #[cfg(all(
                not(feature = "milestone-m5"),
                not(feature = "native-qsim-job"),
                not(feature = "native-sdk-app"),
                not(feature = "native-supervisor"),
                not(feature = "native-keyring"),
                not(feature = "native-display-probe"),
                not(feature = "native-fpu")
            ))]
            {
                e.success += 1;
                match number {
                    0x20 => e.send[slot] += 1,
                    0x21 => e.recv[slot] += 1,
                    0x12 => e.revoke[slot] += 1,
                    0x10 => e.close[slot] += 1,
                    0x11 => e.derive[slot] += 1,
                    _ => {}
                }
            }
            (0, value)
        }
        Err(error) => {
            if let Some((p, bytes)) = output_before {
                assert_eq!(
                    read_plan(&p),
                    bytes,
                    "failed receive partially wrote user output"
                );
            }
            #[cfg(all(
                not(feature = "milestone-m5"),
                not(feature = "native-qsim-job"),
                not(feature = "native-sdk-app"),
                not(feature = "native-supervisor"),
                not(feature = "native-keyring"),
                not(feature = "native-display-probe"),
                not(feature = "native-fpu")
            ))]
            {
                let index = (-error.code()) as usize;
                e.errors[index] += 1;
                if number == 0x21 && error == Error::Again {
                    e.again[slot] += 1;
                }
            }
            (error.code(), 0)
        }
    }
}
pub fn process_exit(pid: u64) {
    engine().exit(pid).expect("registered process cleanup");
    assert!(engine().check_invariants());
}
#[cfg(all(
    not(feature = "milestone-m5"),
    not(feature = "native-qsim-job"),
    not(feature = "native-sdk-app"),
    not(feature = "native-supervisor"),
    not(feature = "native-keyring"),
    not(feature = "native-display-probe"),
    not(feature = "native-fpu")
))]
mod acceptance {
    use super::*;
    pub fn observe_exit(state: &mut State, slot: usize, _counter: u64) {
        let expected = case() == 8 && state.round == 1 && slot == 0 && state.last_event.code == 0;
        state.unexpected |= !expected;
        evidence().expected_exit |= expected;
        state.observer_survivor(slot);
    }
    pub fn observe_fault(state: &mut State, slot: usize, context: &[u64; 22], _counter: u64) {
        let expected =
            case() == 8 && state.round == 2 && slot == 1 && context[15] == 6 && context[16] == 0;
        state.unexpected |= !expected;
        evidence().expected_fault |= expected;
        state.observer_survivor(slot);
    }
    fn word(state: &State, slot: usize, offset: u64) -> u64 {
        read_word(state.processes[slot].physical(DATA + offset))
    }
    fn put(state: &State, slot: usize, offset: u64, value: u64) {
        write_word(state.processes[slot].physical(DATA + offset), value);
    }
    struct Script {
        slot: usize,
        count: usize,
    }
    impl Script {
        fn new(slot: usize) -> Self {
            Self { slot, count: 0 }
        }
        fn row(
            &mut self,
            state: &State,
            op: u64,
            args: [u64; 6],
            rax: i64,
            rdx: u64,
            flags: u64,
        ) -> usize {
            assert!(self.count < 32);
            let i = self.count;
            let row = [
                op, args[0], args[1], args[2], args[3], args[4], args[5], rax as u64, rdx, flags,
            ];
            for (j, value) in row.into_iter().enumerate() {
                write_word(
                    state.processes[self.slot].physical(ROWS + i as u64 * 80 + j as u64 * 8),
                    value,
                );
            }
            self.count += 1;
            i
        }
        fn call(
            &mut self,
            s: &State,
            op: u64,
            a: u64,
            b: u64,
            c: u64,
            rax: i64,
            rdx: u64,
            flags: u64,
        ) -> usize {
            self.row(s, op, [a, b, c, 0, 0, 0], rax, rdx, flags)
        }
        fn wait(&mut self, s: &State, value: u64) {
            self.call(s, 0x104, DATA + 96, value, 0, 0, 0, 0);
        }
        fn write(&mut self, s: &State, address: u64, value: u64) {
            self.call(s, 0x100, address, value, 0, 0, 0, 0);
        }
        fn check(&mut self, s: &State, address: u64, value: u64) {
            self.call(s, 0x102, address, value, 0, 0, 0, 0);
        }
        fn finish(self, s: &State) {
            put(s, self.slot, 88, self.count as u64);
            put(s, self.slot, 0, 1);
        }
    }
    fn message(state: &State, slot: usize, caps: bool, handle: u64) {
        let mut m = Message::new();
        m.tag = 0x4455;
        m.len = 4;
        m.data[..4].copy_from_slice(b"ABCD");
        if caps {
            m.cap_count = 1;
            m.caps[0] = Cap { handle, rights: 13 };
        }
        let bytes = m.encode();
        let copy = plan(state, slot, MSG, 320, true).unwrap();
        write_plan(&copy, &bytes);
    }
    pub fn manifest(state: &mut State) {
        assert_eq!(state.amount, 3);
        assert_eq!(engine().stats().processes, 0);
        // SAFETY: No active syscall; test evidence has one service-stack owner.
        unsafe {
            ptr::addr_of_mut!(EVIDENCE).write(EMPTY_EVIDENCE);
        }
        if state.round == 0 {
            // SAFETY: BSP-only setup precedes AP release; later lifecycle rounds
            // are serialized by M3Guard with IF clear. Reset only before round one.
            unsafe {
                ptr::addr_of_mut!(LIFECYCLE_ROUNDS).write(0);
            }
        }
        let mut roots = [0; 3];
        for (slot, root) in roots.iter_mut().enumerate() {
            let pid = state.processes[slot].pid;
            engine().register(pid).unwrap();
            *root = engine().create_endpoint(pid).unwrap();
            put(state, slot, 32, slot as u64);
            put(state, slot, 56, *root);
            put(state, slot, 96, 0);
        }
        for slot in 0..3 {
            let next = (slot + 1) % 3;
            let cap = engine()
                .bootstrap_delegate(
                    state.processes[next].pid,
                    roots[next],
                    state.processes[slot].pid,
                    ALL_RIGHTS & !RECV,
                )
                .unwrap();
            put(state, slot, 64, cap);
            message(state, slot, false, 0);
            let pa = state.processes[slot].physical(OUT);
            paging::physical(pa, |bytes| {
                // SAFETY: Exclusive owned RW page before user start. Poison proves
                // receive overwrites all 320 output bytes, including canonical zeros.
                unsafe {
                    ptr::write_bytes(bytes, 0xa5, PAGE as usize);
                }
            });
        }
        if case() == 4 {
            let foreign = engine().create_endpoint(state.processes[1].pid).unwrap();
            put(state, 0, 72, foreign);
        }
        build_scripts(state, roots);
        assert!(engine().check_invariants());
    }
    pub fn poll(state: &mut State) -> Option<*const u64> {
        for p in &state.processes[..state.amount] {
            if p.status != Status::Dead {
                assert_eq!(read_word(p.physical(DATA + 16)), p.pattern);
            }
        }
        if state.unexpected {
            if state.survivor_slot < state.amount
                && state.processes[state.survivor_slot].status != Status::Dead
                && word(state, state.survivor_slot, 8) <= state.survivor_start + 8
            {
                return None;
            }
            serial::write(b"M4:FAIL:USER_FIXTURE\n");
            serial::exit(0x11);
        }
        barriers(state);
        if !scripts_done(state) {
            return None;
        }
        let e = evidence();
        let survivor = if state.processes[2].status != Status::Dead {
            2
        } else {
            1
        };
        if !e.started {
            e.started = true;
            e.survivor = word(state, survivor, 8);
            return None;
        }
        if word(state, survivor, 8) <= e.survivor + 8 {
            return None;
        }
        validate_evidence(state);
        if case() == 8 && state.round < 2 {
            #[cfg(feature = "native-smp-m4")]
            {
                if super::arch::current_cpu_index() != smp::bootstrap_index() {
                    return None;
                }
                request_quiescence(QUIESCE_ROUND_RESET);
                return None;
            }
            #[cfg(not(feature = "native-smp-m4"))]
            {
                cleanup_all(state);
                state.round += 1;
                state.setup();
                return None;
            }
        }
        #[cfg(feature = "native-smp-m4")]
        {
            if super::arch::current_cpu_index() != smp::bootstrap_index() {
                return None;
            }
            request_quiescence(QUIESCE_FINAL_CLEANUP);
            return None;
        }
        #[cfg(not(feature = "native-smp-m4"))]
        {
            serial::write(case_marker());
            serial::write(b"M4:SURVIVOR_PROGRESS:PASS\n");
            cleanup_all(state);
            serial::write(b"M4:CLEANUP:PASS\n");
            serial::write(ks_marker());
            serial::write(b"M4:PASS\nM1:PASS\n");
            serial::exit(0x10)
        }
    }
    #[cfg(feature = "native-smp-m4")]
    pub(super) fn finish_quiescence(state: &mut State) -> *const u64 {
        let action = QUIESCE_ACTION.load(Ordering::Acquire);
        let cpu_count = ap::cpu_count();
        let expected = (1 << cpu_count) - 1;
        assert!(action != QUIESCE_NONE);
        assert_eq!(QUIESCED_CPUS.load(Ordering::Acquire), expected);
        assert!(state.cpu_current[..cpu_count]
            .iter()
            .all(|owner| *owner == MAX));
        serial::write(b"M4:SMP:QUIESCED:PASS\n");

        match action {
            QUIESCE_ROUND_RESET => {
                cleanup_all(state);
                serial::write(b"M4:SMP:ROUND_CLEANUP:PASS\n");
                state.round += 1;
                state.setup();
                QUIESCED_CPUS.store(0, Ordering::Relaxed);
                QUIESCE_ACTION.store(QUIESCE_NONE, Ordering::Release);
                serial::write(b"M4:SMP:ROUND_RESET:PASS\n");
                smp::schedule_aps();
                state
                    .select_smp(smp::bootstrap_index())
                    .unwrap_or_else(|| serial::fail(b"M4:SMP:NO_BOOT_PROCESS"))
            }
            QUIESCE_FINAL_CLEANUP => {
                // The lifecycle result is emitted only after the last cleanup.
                if case() != 8 {
                    serial::write(case_marker());
                    serial::write(b"M4:SURVIVOR_PROGRESS:PASS\n");
                }
                cleanup_all(state);
                serial::write(b"M4:SMP:ROUND_CLEANUP:PASS\n");
                if case() == 8 {
                    serial::write(case_marker());
                    serial::write(b"M4:SURVIVOR_PROGRESS:PASS\n");
                }
                serial::write(b"M4:CLEANUP:PASS\n");
                serial::write(ks_marker());
                serial::write(b"M4:PASS\nM1:PASS\n");
                state.complete_round()
            }
            _ => serial::fail(b"M4:SMP:QUIESCE_ACTION"),
        }
    }
    fn cleanup_all(state: &mut State) {
        for slot in 0..state.amount {
            if state.processes[slot].status != Status::Dead {
                state.processes[slot].status = Status::Dying;
                state.cleanup(slot);
            }
        }
        let stats = engine().stats();
        assert_eq!(stats.processes, 0);
        assert_eq!(stats.handles, 0);
        assert_eq!(stats.nodes, 0);
        assert_eq!(stats.endpoints, 0);
        assert_eq!(stats.queued_messages, 0);
        assert!(engine().check_invariants());
    }
    fn case() -> u8 {
        if cfg!(feature = "test-m4-copy") {
            1
        } else if cfg!(feature = "test-m4-wire") {
            2
        } else if cfg!(feature = "test-m4-snapshot") {
            3
        } else if cfg!(feature = "test-m4-handles") {
            4
        } else if cfg!(feature = "test-m4-rights") {
            5
        } else if cfg!(feature = "test-m4-revoke") {
            6
        } else if cfg!(feature = "test-m4-quotas") {
            7
        } else if cfg!(feature = "test-m4-lifecycle") {
            8
        } else {
            0
        }
    }
    fn case_marker() -> &'static [u8] {
        match case() {
            0 => b"M4:EXCHANGE:PASS\n",
            1 => b"M4:CASE:COPY:PASS\n",
            2 => b"M4:CASE:WIRE:PASS\n",
            3 => b"M4:CASE:SNAPSHOT:PASS\n",
            4 => b"M4:CASE:HANDLES:PASS\n",
            5 => b"M4:CASE:RIGHTS:PASS\n",
            6 => b"M4:CASE:REVOKE:PASS\n",
            7 => b"M4:CASE:QUOTAS:PASS\n",
            8 => b"M4:CASE:LIFECYCLE:PASS\n",
            _ => unreachable!(),
        }
    }
    fn ks_marker() -> &'static [u8] {
        match case() {
            0 => b"",
            1 => b"KS-09:PASS\n",
            2 => b"KS-10:PASS\n",
            3 => b"KS-11:PASS\n",
            4 => b"KS-12:PASS\n",
            5 => b"KS-13:PASS\n",
            6 => b"KS-14:PASS\n",
            7 => b"KS-15:PASS\n",
            8 => b"KS-16:PASS\n",
            _ => unreachable!(),
        }
    }
    fn build_scripts(s: &State, roots: [u64; 3]) {
        let mut a = Script::new(0);
        let mut b = Script::new(1);
        let mut c = Script::new(2);
        let main = roots[0];
        let aux = word(s, 0, 64);
        match case() {
            0 => {
                a.call(s, 0x20, aux, MSG, 320, 0, 0, 0);
                b.call(s, 0x21, roots[1], OUT, 320, 0, 4, 256);
                b.check(s, OUT + 64, 0x44434241);
                b.call(s, 0x20, word(s, 1, 64), MSG, 320, 0, 0, 0);
                c.call(s, 0x21, roots[2], OUT, 320, 0, 4, 256);
                c.check(s, OUT + 64, 0x44434241);
            }
            1 => {
                let original = read_plan(&plan(s, 0, MSG, 320, false).unwrap());
                write_plan(&plan(s, 0, MSG + 1, 320, true).unwrap(), &original);
                a.call(s, 0x20, main, MSG + 1, 320, 0, 0, 0);
                for pointer in [
                    0,
                    text_start(),
                    0x0001000040000000,
                    u64::MAX - 100,
                    STACK_END - 160,
                    OUT + PAGE - 160,
                    HEAP + 65536 - 160,
                ] {
                    a.call(s, 0x21, main, pointer, 320, -3, 0, 0);
                }
                a.call(s, 0x21, main, OUT + 1, 320, 0, 4, 0);
                a.check(s, OUT + 65, 0x44434241);
                for pointer in [
                    0,
                    text_start(),
                    0x0001000040000000,
                    u64::MAX - 100,
                    STACK_END - 160,
                    HEAP + 65536 - 160,
                ] {
                    a.call(s, 0x20, main, pointer, 320, -3, 0, 0);
                }
                a.call(s, 0x20, main, MSG + 1, 320, 0, 0, 0);
                a.call(s, 0x21, main, OUT + 1, 320, 0, 4, 0);
                a.call(s, 0x20, main, 0x40013000, 320, 0, 0, 0);
                a.call(s, 0x21, main, OUT + 1, 320, 0, 0, 0);
                a.check(s, OUT + 65, 0);
            }
            2 => {
                for size in [0, 319, 321, u64::MAX] {
                    a.call(s, 0x20, 0, 0, size, -11, 0, 0);
                    a.call(s, 0x21, 0, 0, size, -11, 0, 0);
                }
                a.write(s, MSG + 16, 257);
                a.call(s, 0x20, main, MSG, 320, -11, 0, 0);
                a.write(s, MSG + 16, (3u64 << 32) | 4);
                a.call(s, 0x20, main, MSG, 320, -11, 0, 0);
                a.write(s, MSG + 16, 4);
                for (address, value) in [(MSG, 1), (MSG + 24, 1), (MSG + 32, 1), (MSG + 72, 1)] {
                    a.write(s, address, value);
                    a.call(s, 0x20, main, MSG, 320, -1, 0, 0);
                    a.write(s, address, 0);
                }
                let valid = read_plan(&plan(s, 0, MSG, 320, false).unwrap());
                write_plan(&plan(s, 0, OUT + 1, 320, true).unwrap(), &valid);
                a.call(s, 0x20, main, OUT + 1, 320, 0, 0, 0);
                a.call(s, 0x21, main, MSG + 1, 320, 0, 4, 0);
                a.check(s, MSG + 65, 0x44434241);
                a.row(s, 0x21, [0, 0, 320, 1, 0, 0], -1, 0, 0);
            }
            3 => {
                a.call(s, 0x20, main, MSG, 320, 0, 0, 0);
                a.call(s, 0x21, main, OUT, 320, 0, 4, 0);
                a.check(s, OUT + 8, 0x4455);
                a.check(s, OUT + 16, 4);
                a.check(s, OUT + 64, 0x44434241);
                a.check(s, MSG + 16, 257);
                a.check(s, MSG + 64, 0xcccccccccccccccc);
                for number in [0x30, 0x31, 0x32] {
                    a.call(s, number, 0, 0, 0, -2, 0, 0);
                }
            }
            4 => {
                for handle in [0, word(s, 0, 72), (9u64 << 32) | 250] {
                    a.call(s, 0x10, handle, 0, 0, -4, 0, 0);
                }
                let derived = a.call(s, 0x11, main, 1, 0, 0, 0, 64);
                a.call(s, 0x10, derived as u64, 0, 0, 0, 0, 1);
                a.call(s, 0x20, derived as u64, MSG, 320, -4, 0, 1);
                let fresh = a.call(s, 0x11, main, 1, 0, 0, 0, 64);
                a.call(s, 0x20, derived as u64, MSG, 320, -4, 0, 1);
                a.call(s, 0x20, fresh as u64, MSG, 320, 0, 0, 1);
                a.call(s, 0x21, main, OUT, 320, 0, 4, 0);
                // Numeric slot1 collides with the caller's existing own root only.
                a.call(s, 0x20, main, MSG, 320, 0, 0, 0);
                a.call(s, 0x21, main, OUT, 320, 0, 4, 0);
                a.check(s, OUT, s.processes[0].pid);
            }
            5 => {
                let no_rights = a.call(s, 0x11, main, 0, 0, 0, 0, 64);
                a.call(s, 0x11, no_rights as u64, 0, 0, -5, 0, 1);
                a.call(s, 0x20, no_rights as u64, MSG, 320, -5, 0, 1);
                a.call(s, 0x11, main, 32, 0, -1, 0, 0);
                let send_only = a.call(s, 0x11, main, 1, 0, 0, 0, 64);
                a.call(s, 0x11, send_only as u64, 31, 0, -5, 0, 1);
                let limited = a.call(s, 0x11, main, 5, 0, 0, 0, 64);
                a.call(s, 0x11, limited as u64, 31, 0, -5, 0, 1);
                // Transfer needs TRANSFER and must not carry RECV.
                a.write(s, MSG + 16, (1u64 << 32) | 4);
                a.call(s, 0x100, MSG + 32, send_only as u64, 0, 0, 0, 2);
                a.write(s, MSG + 40, 1);
                a.call(s, 0x20, main, MSG, 320, -5, 0, 0);
                a.write(s, MSG + 32, main);
                a.write(s, MSG + 40, 31);
                a.call(s, 0x20, main, MSG, 320, -5, 0, 0);
                a.write(s, MSG + 40, 13);
                a.call(s, 0x20, main, MSG, 320, 0, 0, 0);
                a.call(s, 0x21, main, OUT, 320, 0, 4, 0);
            }
            6 => {
                message(s, 0, true, main);
                message(s, 1, true, 0);
                a.call(s, 0x20, aux, MSG, 320, 0, 0, 0);
                a.call(s, 0x12, main, 0, 0, 0, 0, 0);
                a.wait(s, 1);
                a.call(s, 0x20, aux, MSG, 320, 0, 0, 0);
                a.wait(s, 2);
                a.call(s, 0x12, main, 0, 0, 0, 0, 0);
                b.wait(s, 1);
                b.call(s, 0x21, roots[1], OUT, 320, -8, 0, 0);
                b.wait(s, 2);
                b.call(s, 0x21, roots[1], OUT, 320, 0, 4, 0);
                let bh = b.call(s, 0x103, OUT + 32, 0, 0, 0, 0, 64);
                b.call(s, 0x100, MSG + 32, bh as u64, 0, 0, 0, 2);
                b.call(s, 0x20, word(s, 1, 64), MSG, 320, 0, 0, 0);
                b.wait(s, 3);
                b.call(s, 0x20, bh as u64, MSG, 320, -10, 0, 1);
                b.check(s, OUT + 64, 0x44434241);
                c.call(s, 0x21, roots[2], OUT, 320, 0, 4, 256);
                let ch = c.call(s, 0x103, OUT + 32, 0, 0, 0, 0, 64);
                c.wait(s, 3);
                c.call(s, 0x20, ch as u64, MSG, 320, -10, 0, 1);
                c.check(s, OUT + 64, 0x44434241);
            }
            7 => {
                let mut chain = [0usize; 8];
                for i in 0..8 {
                    chain[i] = a.call(
                        s,
                        0x11,
                        if i == 0 { main } else { chain[i - 1] as u64 },
                        31,
                        0,
                        0,
                        0,
                        64 | if i == 0 { 0 } else { 1 },
                    );
                }
                a.call(s, 0x11, chain[7] as u64, 31, 0, -7, 0, 1);
                for result in chain {
                    a.call(s, 0x10, result as u64, 0, 0, 0, 0, 1);
                }
                let last = a.call(s, 0x11, main, 1, 0, 0, 0, 64 | (253u64 << 32));
                a.call(s, 0x11, main, 1, 0, -7, 0, 0);
                a.call(s, 0x10, last as u64, 0, 0, 0, 0, 1);
                a.call(s, 0x11, main, 1, 0, 0, 0, 64);
                a.call(s, 0x20, main, MSG, 320, 0, 0, 16u64 << 32);
                a.call(s, 0x20, main, MSG, 320, -8, 0, 0);
                a.call(s, 0x21, main, OUT, 320, 0, 4, 16u64 << 32);
                a.call(s, 0x21, main, OUT, 320, -8, 0, 0);
            }
            8 => match s.round {
                0 => {
                    a.call(s, 0x20, main, MSG, 320, 0, 0, 0);
                    a.call(s, 0x10, main, 0, 0, 0, 0, 0);
                    c.wait(s, 1);
                    c.call(s, 0x20, word(s, 2, 64), MSG, 320, -9, 0, 0);
                }
                1 => {
                    message(s, 0, true, main);
                    a.call(s, 0x20, aux, MSG, 320, 0, 0, 0);
                    a.call(s, 2, 0, 0, 0, 0, 0, 0);
                    b.wait(s, 1);
                    b.call(s, 0x21, roots[1], OUT, 320, -8, 0, 0);
                }
                2 => {
                    message(s, 0, true, main);
                    a.call(s, 0x20, aux, MSG, 320, 0, 0, 0);
                    a.wait(s, 1);
                    a.call(s, 0x20, aux, MSG, 320, -10, 0, 0);
                    b.wait(s, 1);
                    b.call(s, 0x105, 0, 0, 0, 0, 0, 0);
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
        a.finish(s);
        // Non-participating peers remain private yield-workers rather than claiming
        // script success. Their increasing counters demonstrate survivor progress.
        if b.count != 0 {
            b.finish(s);
        } else {
            put(s, 1, 0, 0);
        }
        if c.count != 0 {
            c.finish(s);
        } else {
            put(s, 2, 0, 0);
        }
    }
    fn barriers(s: &State) {
        let e = evidence();
        if case() == 6 {
            if e.revoke[0] >= 1 {
                put(s, 1, 96, 1);
            }
            if e.again[1] >= 1 {
                put(s, 0, 96, 1);
            }
            if e.send[0] >= 2 {
                put(s, 1, 96, 2);
            }
            if e.recv[2] >= 1 {
                put(s, 0, 96, 2);
            }
            if e.revoke[0] >= 2 {
                put(s, 1, 96, 3);
                put(s, 2, 96, 3);
            }
        }
        if case() == 8 {
            match s.round {
                0 => {
                    if e.close[0] >= 1 {
                        put(s, 2, 96, 1);
                    }
                }
                1 => {
                    if s.processes[0].status == Status::Dead && e.expected_exit {
                        put(s, 1, 96, 1);
                    }
                }
                2 => {
                    if e.send[0] >= 1 && s.processes[1].status != Status::Dead {
                        put(s, 1, 96, 1);
                    }
                    if s.processes[1].status == Status::Dead && e.expected_fault {
                        put(s, 0, 96, 1);
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    fn done(s: &State, slot: usize) -> bool {
        s.processes[slot].status != Status::Dead && word(s, slot, 48) == 1
    }
    fn scripts_done(s: &State) -> bool {
        match case() {
            0 | 6 => done(s, 0) && done(s, 1) && done(s, 2),
            8 => match s.round {
                0 => done(s, 0) && done(s, 2),
                1 => s.processes[0].status == Status::Dead && done(s, 1),
                2 => s.processes[1].status == Status::Dead && done(s, 0),
                _ => false,
            },
            _ => done(s, 0),
        }
    }
    fn validate_evidence(s: &State) {
        let e = evidence();
        assert!(e.calls > 0 && e.success > 0);
        assert!(engine().check_invariants());
        match case() {
            0 => {
                assert!(e.send[0] >= 1 && e.recv[1] >= 1 && e.send[1] >= 1 && e.recv[2] >= 1);
                assert_eq!(read_word(s.processes[1].physical(OUT + 64)), 0x44434241);
                assert_eq!(read_word(s.processes[2].physical(OUT + 64)), 0x44434241);
            }
            1 => {
                assert!(e.errors[3] >= 13 && e.send[0] >= 3 && e.recv[0] >= 3);
                assert_eq!(read_word(s.processes[0].physical(OUT + 65)), 0);
            }
            2 => {
                assert!(e.errors[11] >= 10 && e.errors[1] >= 5 && e.send[0] >= 1 && e.recv[0] >= 1);
            }
            3 => {
                assert!(e.hook && e.errors[2] >= 3 && e.send[0] >= 1 && e.recv[0] >= 1);
                assert_eq!(read_word(s.processes[0].physical(OUT + 64)), 0x44434241);
                assert_eq!(read_word(s.processes[0].physical(MSG + 16)), 257);
            }
            4 => {
                assert!(
                    e.errors[4] >= 5
                        && e.derive[0] >= 2
                        && e.close[0] >= 1
                        && e.send[0] >= 2
                        && e.recv[0] >= 2
                );
            }
            5 => {
                assert!(e.errors[5] >= 6 && e.errors[1] >= 1 && e.send[0] >= 1 && e.recv[0] >= 1);
            }
            6 => {
                assert!(
                    e.revoke[0] >= 2
                        && e.send[0] >= 2
                        && e.recv[1] >= 1
                        && e.send[1] >= 1
                        && e.recv[2] >= 1
                        && e.errors[10] >= 2
                        && e.again[1] >= 1
                );
                for slot in [1, 2] {
                    assert_eq!(read_word(s.processes[slot].physical(OUT + 64)), 0x44434241);
                }
            }
            7 => {
                assert!(
                    e.derive[0] >= 262
                        && e.errors[7] >= 2
                        && e.errors[8] >= 2
                        && e.close[0] >= 9
                        && e.send[0] >= 16
                        && e.recv[0] >= 16
                );
            }
            8 => {
                match s.round {
                    0 => assert!(e.close[0] >= 1 && e.errors[9] >= 1),
                    1 => assert!(e.expected_exit && e.again[1] >= 1),
                    2 => assert!(e.expected_fault && e.errors[10] >= 1),
                    _ => unreachable!(),
                }
                // SAFETY: Serialized evidence accumulator, one bit per completed real
                // close/exit/fault round. A fresh manifest cannot fabricate prior bits.
                unsafe {
                    let bits = ptr::addr_of!(LIFECYCLE_ROUNDS).read() | (1 << s.round);
                    ptr::addr_of_mut!(LIFECYCLE_ROUNDS).write(bits);
                    if s.round == 2 {
                        assert_eq!(bits, 7);
                    }
                }
            }
            _ => unreachable!(),
        }
        assert_eq!(engine().stats().queued_messages, 0);
    }
}
#[cfg(feature = "native-smp-m4")]
pub(super) fn finish_quiescence(state: &mut State) -> *const u64 {
    acceptance::finish_quiescence(state)
}
#[cfg(all(
    not(feature = "milestone-m5"),
    not(feature = "native-qsim-job"),
    not(feature = "native-sdk-app"),
    not(feature = "native-supervisor"),
    not(feature = "native-keyring"),
    not(feature = "native-display-probe"),
    not(feature = "native-fpu")
))]
pub(super) use acceptance::{manifest, observe_exit, observe_fault, poll};

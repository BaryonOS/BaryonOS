//! Separate observer for the opt-in native-memory architecture fixture. No
//! allocation, mapping, quota or fault policy in this file affects other apps.
//! The app's evidence is checked against live kernel mappings/owners and actual
//! exception/exit records before cleanup, not accepted as a synthetic success.
use super::*;
#[path = "../../../../sdk/examples/memory/src/contract.rs"]
mod contract;
use contract::*;

struct Evidence {
    baseline: usize,
    kernel_tables: usize,
    clean: u64,
    pages: u64,
    zero: u64,
    denied: u64,
    quota: u64,
    collections: u64,
    alloc_oom: u64,
    private: u64,
    faults: u64,
    exits: u64,
    reuse: u64,
    oom: u64,
    retries: u64,
    retry_zero: u64,
    survivor_start: u64,
    started: bool,
    oom_survivor: usize,
    retry_requested: bool,
    retry_base: u64,
    wipe_frames: [u64; kernel_core::USER_QUOTA],
    wipe_count: usize,
    wiped_bytes: u64,
}
static mut EVIDENCE: Evidence = Evidence {
    baseline: 0,
    kernel_tables: 0,
    clean: 0,
    pages: 0,
    zero: 0,
    denied: 0,
    quota: 0,
    collections: 0,
    alloc_oom: 0,
    private: 0,
    faults: 0,
    exits: 0,
    reuse: 0,
    oom: 0,
    retries: 0,
    retry_zero: 0,
    survivor_start: 0,
    started: false,
    oom_survivor: MAX,
    retry_requested: false,
    retry_base: 0,
    wipe_frames: [0; kernel_core::USER_QUOTA],
    wipe_count: 0,
    wiped_bytes: 0,
};
fn evidence() -> &'static mut Evidence {
    // SAFETY: KERNEL held, IF=0; fixture callbacks are sequential. Callers hold
    // only one arena reference and never reenter this accessor while it is live.
    unsafe { &mut *ptr::addr_of_mut!(EVIDENCE) }
}
fn require(ok: bool, why: &[u8]) {
    if !ok {
        serial::write(b"MEMORY:FAIL:");
        serial::write(why);
        serial::write(b"\n");
        serial::exit(0x11);
    }
}
fn word(s: &State, slot: usize, offset: u64) -> u64 {
    read_word(s.processes[slot].physical(DATA + offset))
}
fn put(s: &State, slot: usize, offset: u64, value: u64) {
    write_word(s.processes[slot].physical(DATA + offset), value);
}
fn case() -> u64 {
    if cfg!(feature = "native-memory-oom") {
        4
    } else if cfg!(feature = "native-memory-peer") {
        3
    } else if cfg!(feature = "native-memory-guard") {
        2
    } else if cfg!(feature = "native-memory-nx") {
        1
    } else {
        0
    }
}
fn configure(s: &State, slot: usize, generation: u64) {
    put(s, slot, GENERATION, generation);
    put(s, slot, SLOT, slot as u64);
    put(s, slot, CASE, if generation == 2 { 0 } else { case() });
    put(s, slot, INITIAL_MAPPINGS, s.processes[slot].count as u64);
}
pub(super) fn manifest(s: &State) {
    let count = if case() == 4 { MAX } else { 2 };
    require(
        s.amount == count && s.next_pid == count as u64 + 1,
        b"MANIFEST",
    );
    let e = evidence();
    e.baseline = s.free_count()
        + s.processes[..count]
            .iter()
            .map(|p| {
                p.count + p.kernel_frames.len() + p.paging.as_ref().unwrap().owned_table_count()
            })
            .sum::<usize>();
    e.kernel_tables = s.kernel.owned_table_count();
    for slot in 0..count {
        require(s.processes[slot].pid == slot as u64 + 1, b"INITIAL_PID");
        require(s.processes[slot].heap_pages == 16, b"INITIAL_HEAP");
        configure(s, slot, 1);
    }
    serial::write(b"MEMORY:MANIFEST:PASS\n");
}
fn collect(s: &State, slot: usize) {
    let p = &s.processes[slot];
    let pages = word(s, slot, PAGES);
    let suffix = HEAP + (16 + pages) * PAGE;
    require(
        word(s, slot, CLEAN) == 1 && word(s, slot, DONE) == 1,
        b"APP_COMPLETION",
    );
    require(word(s, slot, ARG_DENIED) == 7, b"ARGUMENT_REFUSALS");
    require(
        word(s, slot, COLLECTIONS) == 1 && word(s, slot, ALLOC_OOM) == 1,
        b"ALLOCATOR_COLLECTIONS",
    );
    require(
        word(s, slot, ZERO_BYTES) == pages * PAGE && pages > 0,
        b"ALL_NEW_BYTES_ZERO",
    );
    require(
        word(s, slot, PRIVATE) == 1
            && word(s, slot, PATTERN)
                == (p.pattern ^ 0x7b5d_e031_9184_6621 ^ word(s, slot, GENERATION)),
        b"PRIVATE_PATTERN",
    );
    require(
        word(s, slot, SUFFIX_END) == suffix && p.heap_pages == 16 + pages,
        b"SUFFIX_ACCOUNTING",
    );
    require(
        word(s, slot, STARTUP_PAGES) == word(s, slot, INITIAL_MAPPINGS)
            && p.count as u64 == word(s, slot, INITIAL_MAPPINGS) + pages,
        b"MAPPING_BUDGET",
    );
    require(
        p.count <= kernel_core::USER_QUOTA && p.count <= MAPS,
        b"UNCHANGED_QUOTA",
    );
    require(
        read_word(p.physical(HEAP)) == word(s, slot, PATTERN),
        b"OLD_HEAP_PRESERVED",
    );
    let tables = p.paging.as_ref().unwrap();
    require(tables.leaf(suffix) == 0, b"UNMAPPED_SUFFIX_GUARD");
    let mut actual_heap_pages = 0;
    for m in p
        .mappings()
        .iter()
        .filter(|m| (HEAP..suffix).contains(&m.va))
    {
        actual_heap_pages += 1;
        require(
            m.kind == Kind::Data && m.writable && !m.executable,
            b"HEAP_MAP_RIGHTS",
        );
        require(
            s.frames.owner(m.pa) == Ok(Owner::User(p.pid, Kind::Data)),
            b"HEAP_OWNER",
        );
        require(
            tables.permissions(m.va) == Some((m.pa, true, false, true)),
            b"HEAP_PTE_RW_NX_USER",
        );
        let identity = s.kernel.leaf(m.pa);
        require(
            identity & ADDRESS == m.pa
                && identity & 4 == 0
                && identity & 2 != 0
                && identity & NX != 0,
            b"SUPERVISOR_ALIAS",
        );
        require(
            s.frames
                .alias(m.pa)
                .is_ok_and(|a| a.total == 2 && a.writable == 2 && a.executable == 0),
            b"HEAP_ALIAS_COUNTS",
        );
    }
    require(actual_heap_pages == p.heap_pages, b"HEAP_MAP_COUNT");
    let e = evidence();
    require(e.wipe_count == 0, b"PREVIOUS_CLEANUP_OBSERVED");
    for m in p
        .mappings()
        .iter()
        .filter(|m| (HEAP..suffix).contains(&m.va))
    {
        e.wipe_frames[e.wipe_count] = m.pa;
        e.wipe_count += 1;
    }
    if case() == 4 {
        require(
            word(s, slot, OOM) + word(s, slot, QUOTA_DENIED) == 1,
            b"REAL_RESOURCE_STOP",
        );
        if slot == e.oom_survivor {
            require(e.retry_requested && word(s, slot, RETRY) == 1, b"OOM_RETRY");
            require(
                word(s, slot, RETRY_BASE) == e.retry_base
                    && word(s, slot, RETRY_ZERO_BYTES) == PAGE,
                b"OOM_NO_SUFFIX_ADVANCE",
            );
        } else {
            require(word(s, slot, RETRY) == 0, b"NO_SPURIOUS_RETRY");
        }
    } else {
        require(
            pages >= 514 && p.count == kernel_core::USER_QUOTA,
            b"GROWTH_PAST_PT_AND_QUOTA_EDGE",
        );
        require(
            word(s, slot, QUOTA_DENIED) == 2 && word(s, slot, OOM) == 0,
            b"QUOTA_NO_ADVANCE",
        );
    }
    e.clean += 1;
    e.pages += pages;
    e.zero += word(s, slot, ZERO_BYTES);
    e.denied += word(s, slot, ARG_DENIED);
    e.quota += word(s, slot, QUOTA_DENIED);
    e.collections += 1;
    e.alloc_oom += 1;
    e.private += 1;
    e.oom += word(s, slot, OOM);
    e.retries += word(s, slot, RETRY);
    e.retry_zero += word(s, slot, RETRY_ZERO_BYTES);
}
fn verify_cleanup(s: &State) {
    let e = evidence();
    // The terminating process has been cleaned up; no allocation/yield occurs
    // between that return and this poll callback. These are diagnostic physical
    // numbers only, never references or mappings retained into a live process.
    for &pa in &e.wipe_frames[..e.wipe_count] {
        require(s.frames.owner(pa) == Ok(Owner::Free), b"CLEANUP_FREE_OWNER");
        require(
            s.frames
                .alias(pa)
                .is_ok_and(|a| a.total == 0 && a.writable == 0 && a.executable == 0)
                && s.kernel.leaf(pa) == 0,
            b"CLEANUP_NO_ALIAS",
        );
        let zero = paging::physical(pa, |bytes| {
            (0..PAGE as usize).step_by(8).all(|offset| {
                // SAFETY: Exclusive kernel scratch access to the exact admitted
                // free page; no allocator or scheduler can reuse it under IF=0.
                unsafe { bytes.add(offset).cast::<u64>().read_volatile() == 0 }
            })
        });
        require(zero, b"CLEANUP_PHYSICAL_ZERO");
    }
    e.wiped_bytes += e.wipe_count as u64 * PAGE;
    e.wipe_frames[..e.wipe_count].fill(0);
    e.wipe_count = 0;
}
pub(super) fn observe_fault(s: &State, slot: usize, c: &[u64; 22], _: u64) {
    require(case() > 0 && case() < 4 && slot == 0, b"FAULT_CASE");
    let e = evidence();
    require(
        e.faults == 0 && e.reuse == 0 && e.started,
        b"SINGLE_EXPECTED_FAULT",
    );
    let target = word(s, slot, TARGET);
    let (code, rip) = if case() == 1 {
        (21, target)
    } else {
        (if case() == 2 { 6 } else { 5 }, word(s, slot, FAULT_RIP))
    };
    require(
        s.last_event.vector == 14
            && c[16] == code
            && s.last_event.cr2 == target
            && s.last_event.rip == rip,
        b"EXACT_NATIVE_PAGE_FAULT",
    );
    if case() == 2 {
        require(target == word(s, slot, SUFFIX_END), b"GUARD_ADDRESS");
    }
    if case() == 3 {
        let peer = &s.processes[1];
        require(
            peer.status == Status::Ready && target == peer.physical(HEAP + 65536),
            b"LIVE_PEER_TARGET",
        );
        require(
            s.frames.owner(target) == Ok(Owner::User(peer.pid, Kind::Data)),
            b"PEER_OWNER",
        );
    }
    e.faults = 1;
    e.survivor_start = word(s, 1, HEARTBEAT);
    // End the unique global evidence borrow before collect obtains it.
    collect(s, slot);
}
pub(super) fn observe_exit(s: &State, slot: usize, _: u64) {
    if s.last_event.code != 0 {
        print_number(b"MEMORY:APP_FAILURE_SLOT:", slot as u64);
        print_number(b"MEMORY:APP_FAILURE_STEP:", word(s, slot, FAILURE));
    }
    require(s.last_event.code == 0, b"APP_EXIT");
    collect(s, slot);
    let e = evidence();
    e.exits += 1;
    if case() != 4 && slot == 0 {
        if e.reuse == 0 {
            e.survivor_start = word(s, 1, HEARTBEAT);
        } else {
            put(s, 1, CONTROL, 1);
        }
    }
}
pub(super) fn poll(s: &mut State) {
    verify_cleanup(s);
    require(
        s.ticks < 10000 && s.syscall_calls < 2_000_000,
        b"BOUNDED_RUN",
    );
    let e = evidence();
    if case() == 4 {
        if !e.started
            && s.processes[..MAX]
                .iter()
                .enumerate()
                .all(|(i, p)| p.status != Status::Dead && word(s, i, DONE) == 1)
        {
            e.oom_survivor = (0..MAX).find(|i| word(s, *i, OOM) == 1).unwrap_or(MAX);
            require(e.oom_survivor < MAX, b"ACTUAL_PHYSICAL_OOM_REQUIRED");
            e.started = true;
            for slot in 0..MAX {
                if slot != e.oom_survivor {
                    put(s, slot, CONTROL, 1);
                }
            }
        }
        if e.started
            && !e.retry_requested
            && (0..MAX).all(|i| i == e.oom_survivor || s.processes[i].status == Status::Dead)
        {
            e.retry_base = word(s, e.oom_survivor, SUFFIX_END);
            e.retry_requested = true;
            put(s, e.oom_survivor, CONTROL, 3);
        }
    } else {
        if !e.started && word(s, 0, DONE) == 1 && word(s, 1, DONE) == 1 {
            e.started = true;
            if case() == 0 {
                put(s, 0, CONTROL, 1);
            } else {
                let target = if case() == 1 {
                    HEAP + 65536
                } else if case() == 2 {
                    word(s, 0, SUFFIX_END)
                } else {
                    s.processes[1].physical(HEAP + 65536)
                };
                put(s, 0, TARGET, target);
                put(s, 0, CONTROL, 2);
            }
        }
        if e.started
            && e.reuse == 0
            && s.processes[0].status == Status::Dead
            && word(s, 1, HEARTBEAT) > e.survivor_start + 4
        {
            require(
                s.processes[1].status == Status::Ready,
                b"LIVE_SURVIVOR_PROGRESS",
            );
            s.spawn_process(0);
            require(s.processes[0].pid == 3, b"FRESH_PID_ON_SLOT_REUSE");
            configure(s, 0, 2);
            e.reuse = 1;
        }
        if e.reuse == 1 && s.processes[0].status != Status::Dead && word(s, 0, DONE) == 1 {
            put(s, 0, CONTROL, 1);
        }
    }
    if s.processes[..s.amount]
        .iter()
        .all(|p| p.status == Status::Dead)
    {
        let lives = if case() == 4 { 16 } else { 3 };
        let faults = u64::from(case() > 0 && case() < 4);
        require(
            e.clean == lives && e.exits == lives - faults && e.faults == faults,
            b"LIFECYCLE_COUNTS",
        );
        require(e.reuse == u64::from(case() != 4), b"SLOT_REUSE_COUNT");
        require(
            e.denied == lives * 7
                && e.collections == lives
                && e.alloc_oom == lives
                && e.private == lives,
            b"REAL_APP_CHECK_COUNTS",
        );
        require(
            e.zero == e.pages * PAGE && s.timer_interrupts >= 2,
            b"ZERO_AND_TIMER_COUNTS",
        );
        require(
            e.wipe_count == 0 && e.wiped_bytes == e.zero + e.clean * 65536,
            b"ALL_HEAP_BYTES_WIPED_ON_CLEANUP",
        );
        require(
            e.retries == u64::from(case() == 4)
                && e.retry_zero == if case() == 4 { PAGE } else { 0 },
            b"OOM_RECOVERY_COUNTS",
        );
        require(case() != 4 || e.oom >= 1, b"OOM_OBSERVED");
        let retained = s
            .kernel
            .owned_table_count()
            .checked_sub(e.kernel_tables)
            .unwrap();
        require(
            s.free_count() + retained == e.baseline && heap::clean_boundary(),
            b"FRAME_AND_TRANSACTION_RECOVERY",
        );
        for (name, value) in [
            (b"MEMORY:CLEAN_STARTS:".as_slice(), e.clean),
            (b"MEMORY:GROWN_PAGES:", e.pages),
            (b"MEMORY:ZERO_BYTES:", e.zero),
            (b"MEMORY:ARG_DENIED:", e.denied),
            (b"MEMORY:QUOTA_DENIED:", e.quota),
            (b"MEMORY:PRIVATE_CHECKS:", e.private),
            (b"MEMORY:COLLECTION_RUNS:", e.collections),
            (b"MEMORY:ALLOC_OOM_DENIED:", e.alloc_oom),
            (b"MEMORY:FAULTS:", e.faults),
            (b"MEMORY:EXITS:", e.exits),
            (b"MEMORY:SLOT_REUSES:", e.reuse),
            (b"MEMORY:OOM_FAILURES:", e.oom),
            (b"MEMORY:OOM_RECOVERIES:", e.retries),
            (b"MEMORY:RETRY_ZERO_BYTES:", e.retry_zero),
            (b"MEMORY:TIMER_IRQS:", s.timer_interrupts),
            (b"MEMORY:LIVE_PROCESSES:", 0),
            (b"MEMORY:FRAMES_RECOVERED:", 1),
            (b"MEMORY:KERNEL_TABLES_RETAINED:", retained as u64),
            (b"MEMORY:CLEANUP_ZERO_BYTES:", e.wiped_bytes),
        ] {
            print_number(name, value);
        }
        serial::write(b"MEMORY:GROWTH:PASS\nMEMORY:ISOLATION:PASS\n");
        if case() == 4 {
            serial::write(b"MEMORY:OOM:PASS\n");
        }
        serial::write(b"MEMORY:CLEANUP:PASS\n");
        serial::write(match case() {
            0 => b"MEMORY:CASE:NORMAL:PASS\n",
            1 => b"MEMORY:CASE:NX:PASS\n",
            2 => b"MEMORY:CASE:GUARD:PASS\n",
            3 => b"MEMORY:CASE:PEER:PASS\n",
            _ => b"MEMORY:CASE:OOM:PASS\n",
        });
        serial::write(b"M1:PASS\n");
        serial::exit(0x10);
    }
}

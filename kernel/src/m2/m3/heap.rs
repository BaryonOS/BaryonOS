//! Fallible own-process heap growth. No shrink, delegation, physical mapping or
//! executable memory. Existing process lifetime, copy plans and pinned DMA are
//! unchanged: old mappings never move and cleanup still detaches brokers first.
//!
//! OSDev: Page Frame Allocation, W001461, revision30500: virtual contiguity does
//! not require physical contiguity. Nieuw OS retains its existing frame owner,
//! W^X and USER_QUOTA model; this adds no host allocator or alternate kernel.
use super::super::paging::MapPlan;
use super::*;
use kernel_core::{Error, USER_QUOTA};

const INITIAL_PAGES: u64 = 16;
const LIMIT: u64 = HEAP + USER_QUOTA as u64 * PAGE;
const ENOMEM: i64 = -16; // -9 is the existing Closed error, never repurposed.

struct Transaction {
    frames: [u64; USER_QUOTA],
    count: usize,
    kernel: MapPlan,
    user: MapPlan,
}
static mut TRANSACTION: Transaction = Transaction {
    frames: [0; USER_QUOTA],
    count: 0,
    kernel: MapPlan::new(),
    user: MapPlan::new(),
};

fn code(error: Error) -> i64 {
    match error {
        Error::Exhausted => ENOMEM,
        Error::Quota => -7,
        Error::Range => -1,
        _ => -5,
    }
}
fn erase(t: &mut Transaction) {
    t.frames[..t.count].fill(0);
    t.count = 0;
    t.kernel.clear();
    t.user.clear();
}
fn rollback(s: &mut State, t: &mut Transaction, pid: u64) {
    for &pa in &t.frames[..t.count] {
        // No PTE/alias has been published. Releasing this exclusively reserved
        // frame also zeroes it, exactly as normal process cleanup does.
        s.frames
            .free(pa, Owner::User(pid, Kind::Data), paging::zero_dynamic)
            .expect("unpublished owned heap reservation");
    }
    erase(t);
}

pub(super) fn clean_boundary() -> bool {
    // SAFETY: Called only with IF=0 outside grow(), notably on process cleanup.
    // No active transaction/arena borrow can coexist with process termination.
    let t = unsafe { &*ptr::addr_of!(TRANSACTION) };
    t.count == 0 && t.frames.iter().all(|pa| *pa == 0) && t.kernel.is_clear() && t.user.is_clear()
}

pub(super) fn syscall(s: &mut State, slot: usize, a: [u64; 6]) -> (i64, u64) {
    match grow(s, slot, a) {
        Ok(base) => (0, base),
        Err(error) => (error, 0),
    }
}
fn grow(s: &mut State, slot: usize, a: [u64; 6]) -> Result<u64, i64> {
    if a[0] == 0 || a[1..].iter().any(|word| *word != 0) || slot >= s.amount {
        return Err(-1);
    }
    let p = &s.processes[slot];
    if p.status != Status::Running
        || p.pid == 0
        || p.paging.is_none()
        || p.heap_pages < INITIAL_PAGES
    {
        return Err(-5);
    }
    let bytes = a[0].checked_mul(PAGE).ok_or(-1)?;
    let base = p
        .heap_pages
        .checked_mul(PAGE)
        .and_then(|bytes| HEAP.checked_add(bytes))
        .ok_or(-1)?;
    let end = base.checked_add(bytes).ok_or(-1)?;
    let pages = usize::try_from(a[0]).map_err(|_| -1)?;
    if end > LIMIT
        || p.count
            .checked_add(pages)
            .is_none_or(|count| count > MAPS || count > USER_QUOTA)
    {
        return Err(-7);
    }
    let pid = p.pid;
    #[cfg(feature = "native-memory")]
    memory_fixture::observe_pressure(s);
    // SAFETY: The sole call path is the single-CPU IF=0 event loop. No yield,
    // callback into user code or nested heap transaction occurs before this
    // exclusive arena borrow ends. The ~20KiB arena is permanent supervisor
    // metadata, not a local on the 16KiB service stack. No user pointers reside
    // here, and every success/failure clears all active reservation metadata.
    let t = unsafe { &mut *ptr::addr_of_mut!(TRANSACTION) };
    assert_eq!(t.count, 0);
    // Reserve and zero physical pages without publishing mappings. Allocation
    // remains fallible, including actual physical OOM and per-owner quota.
    if let Err(error) = s.frames.allocate_many(
        Owner::User(pid, Kind::Data),
        &mut t.frames[..pages],
        paging::zero_dynamic,
    ) {
        erase(t);
        return Err(code(error));
    }
    t.count = pages;
    let preflight = (|| {
        for (i, &pa) in t.frames[..t.count].iter().enumerate() {
            // Frames::allocate_many scans free frames in ascending order, so the reserved
            // physical list is strictly ordered. The planner checks that too.
            // Both aliases are RW/NX to this same owned Data frame, never text.
            if s.frames.owner(pa).map_err(code)? != Owner::User(pid, Kind::Data) {
                return Err(-5);
            }
            s.kernel
                .preflight_private_rw(s.frames, &mut t.kernel, pa, pa, false)
                .map_err(code)?;
            s.processes[slot]
                .paging
                .as_ref()
                .ok_or(-5)?
                .preflight_private_rw(s.frames, &mut t.user, base + i as u64 * PAGE, pa, true)
                .map_err(code)?;
        }
        let tables = t.kernel.tables().checked_add(t.user.tables()).ok_or(-7)?;
        if s.free_count() < tables {
            return Err(ENOMEM);
        }
        Ok(())
    })();
    if let Err(error) = preflight {
        rollback(s, t, pid);
        return Err(error);
    }

    // Commit under IF=0: both complete mapping plans have validated unique,
    // canonical, empty destinations, parent flags/owners and metadata bounds.
    // Remaining free frames cover the EXACT deduplicated table demand. No code
    // below allocates anything except those tables. Fresh data frames have zero
    // aliases; two RW/NX aliases cannot overflow counters or violate W^X. Thus
    // existing map/append assertions are internal invariants, not OOM paths.
    let kernel_tables = s.kernel.owned_table_count();
    let user_tables = s.processes[slot]
        .paging
        .as_ref()
        .unwrap()
        .owned_table_count();
    for (i, &pa) in t.frames[..t.count].iter().enumerate() {
        let va = base + i as u64 * PAGE;
        s.kernel.map(s.frames, pa, pa, true, false);
        let p = &mut s.processes[slot];
        p.paging
            .as_mut()
            .unwrap()
            .map_user(s.frames, va, pa, true, false);
        p.append_mapping(Mapping {
            va,
            pa,
            kind: Kind::Data,
            writable: true,
            executable: false,
        });
    }
    // Check the planner against the real mapping path, not only its own model.
    assert_eq!(
        s.kernel.owned_table_count(),
        kernel_tables + t.kernel.tables()
    );
    assert_eq!(
        s.processes[slot]
            .paging
            .as_ref()
            .unwrap()
            .owned_table_count(),
        user_tables + t.user.tables()
    );
    s.processes[slot].heap_pages += a[0];
    // User CR3 is inactive throughout the transaction; select reloads CR3 before
    // user return (PCID/PGE disabled). No old mapping or copy-plan target changed.
    erase(t);
    Ok(base)
}

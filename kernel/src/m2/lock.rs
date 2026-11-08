//! K06 step 5a (native-smp only): IRQ-safe ticket lock with per-CPU lockdep.
//!
//! Interrupt and lock contract (spec §4.1, §4.3): `lock_irqsave` saves
//! RFLAGS.IF, masks interrupts, records the level in this CPU's lockdep set
//! (a violation stops the VM), then spins. The guard releases the lock, drops
//! the level (LIFO only, audit F2; otherwise the VM stops) and only then
//! restores IF. Lockdep and the time-bounded spin with its STUCK marker are
//! active in every native-smp build, release included (audit F5, DG-1). Step
//! 5b: waiters for KERNEL/ASPACE spin with a `sti; pause; cli` window (APs;
//! since 5c-2 every CPU whose sources are all gated, BSP included). Step 5c-2
//! (BT-4): releasing the last lock back to IF=1 honours a deferred 0xe8.
//! Step 5b-2 (ADR-075): also built for M3, where one CPU owns a single lockdep
//! word and `KERNEL` serializes every M3 event (K-1).
//! Audit F3/F4 (ADR-077): KERNEL/ASPACE are refused inside an interrupt
//! handler (LD-1) and their window opens only outside one (LD-2); two locks of
//! one level only through `lock_pair`, by ascending key (LD-3/LD-4).
//! Audit F15 (ADR-079, IQ-2): in native-smp builds a lock records the CPU
//! index of its holder, so the BSP can refuse to quarantine a CPU that holds it.
#[cfg(feature = "native-smp")]
use super::smp;
use core::{
    arch::asm,
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU64, AtomicU8, Ordering},
};
use smp_lock::{level, Guard, Lockdep, Order, TicketLock};

/// One-CPU profiles (M3): the same interface as `smp`, without GS anchors.
/// Every lock use runs with IF=0 on the only CPU, so one state is exact. The
/// M3 entry path does not count IDT nesting (no GS anchor): depth stays 0.
#[cfg(not(feature = "native-smp"))]
mod smp {
    use super::LockdepCell;
    use core::sync::atomic::AtomicU64;
    static LOCKDEP: AtomicU64 = AtomicU64::new(0);
    static STATE: LockdepCell = LockdepCell::new();
    pub fn lockdep() -> &'static AtomicU64 {
        &LOCKDEP
    }
    pub fn with_lockdep<R>(f: impl FnOnce(&mut smp_lock::Lockdep) -> R) -> R {
        // SAFETY: the only CPU, IF=0 inside every lock operation, and `f`
        // (a lockdep method) never re-enters.
        unsafe { STATE.update(&LOCKDEP, f) }
    }
    pub fn irq_depth() -> u32 {
        0
    }
    pub fn irq_ready() -> bool {
        false
    }
    pub fn resched_on_release() {}
    pub fn lockdep_order(_: smp_lock::Order) -> ! {
        crate::serial::fail(b"LOCKDEP_ORDER")
    }
    pub fn lockdep_lifo(_: smp_lock::Order) -> ! {
        crate::serial::fail(b"LOCKDEP_LIFO")
    }
    pub fn lockdep_in_handler(_: u8) -> ! {
        crate::serial::fail(b"LOCKDEP_IN_HANDLER")
    }
    pub fn stuck_deadline() -> u64 {
        0
    }
    pub fn past(_: u64) -> bool {
        false
    }
    pub fn lock_stuck(_: u8) -> ! {
        crate::serial::fail(b"LOCK_STUCK")
    }
    pub fn cpu_index() -> usize {
        0
    }
}

/// One CPU's full lockdep state (keys and pair counts, audit F4). Its held
/// levels are mirrored into an `AtomicU64` that other CPUs and the preempt
/// handler read.
pub struct LockdepCell(UnsafeCell<Lockdep>);

// SAFETY: a cell is only touched through `update`, whose contract confines
// it to its own CPU with IF=0; other CPUs read only the atomic mirror.
unsafe impl Sync for LockdepCell {}

impl LockdepCell {
    pub const fn new() -> Self {
        Self(UnsafeCell::new(Lockdep::new()))
    }

    /// Run `f` on the state, then mirror its held levels into `mirror`.
    ///
    /// # Safety
    /// Only the CPU that owns this cell, with IF=0, and `f` must not call
    /// `update` again: then this is the only live reference to the state.
    pub unsafe fn update<R>(&self, mirror: &AtomicU64, f: impl FnOnce(&mut Lockdep) -> R) -> R {
        // SAFETY: exclusive per the caller's contract above; the cell holds
        // an initialized, aligned `Lockdep` for its whole 'static life.
        let dep = unsafe { &mut *self.0.get() };
        let result = f(dep);
        mirror.store(dep.held_mask(), Ordering::Relaxed);
        result
    }
}

/// K-1 (ADR-075): the one kernel section. Every mutation of M3 `STATE`, the
/// frame allocator, paging, IPC queues, quota and the supervisor holds it.
pub static KERNEL: IrqLock<()> = IrqLock::new(level::KERNEL, ());

/// True while this CPU holds (or awaits) `KERNEL`: its lockdep bit.
pub fn kernel_held() -> bool {
    smp::lockdep().load(Ordering::Relaxed) & (1 << level::KERNEL) != 0
}

pub struct IrqLock<T> {
    level: u8,
    /// LD-3 order within a level (ASPACE: pid, RUNQ: CPU index).
    key: u32,
    inner: TicketLock<T>,
    /// IQ-2: CPU index of the holder, `FREE` otherwise. Written by the holder
    /// only, outside the ticket logic; read by the BSP before an INIT.
    owner: AtomicU8,
}

const FREE: u8 = u8::MAX;

impl<T> IrqLock<T> {
    /// A lock that is never paired with another of its level (key 0).
    pub const fn new(level: u8, value: T) -> Self {
        Self::keyed(level, 0, value)
    }

    /// A lock that `lock_pair` may take together with another of its level.
    pub const fn keyed(level: u8, key: u32, value: T) -> Self {
        Self {
            level,
            key,
            inner: TicketLock::new(value),
            owner: AtomicU8::new(FREE),
        }
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    /// IQ-2: the CPU index (BSP 0, AP slot + 1) that holds this lock now.
    pub fn owner(&self) -> Option<u8> {
        Some(self.owner.load(Ordering::Acquire)).filter(|&cpu| cpu != FREE)
    }

    pub fn lock_irqsave(&self) -> IrqGuard<'_, T> {
        self.take(false)
    }

    /// Terminal paths only (DG-2: NMI, #DF, panic, a stuck lock): take the
    /// lock if it is free right now, without lockdep and without touching IF.
    /// The caller stops the VM afterwards, so no lock order can follow.
    #[cfg(feature = "native-smp")]
    pub fn try_terminal(&self) -> Option<Guard<'_, T>> {
        self.inner.try_lock()
    }

    /// LD-3: take `self` and `other` (same level) by ascending key, whatever
    /// the argument order, so two CPUs pairing the same locks cannot deadlock.
    /// Returns (lower key, higher key): bind it as `let (low, high) = ...` so
    /// `high` drops first (LD-4); dropping the tuple whole releases `low`
    /// first and stops with the LIFO marker. One lock twice, equal keys or
    /// different levels have no order and stop with the order marker.
    pub fn lock_pair<'a>(&'a self, other: &'a Self) -> (IrqGuard<'a, T>, IrqGuard<'a, T>) {
        if self.level != other.level || self.key == other.key {
            smp::lockdep_order(Order {
                held: self.level,
                wanted: other.level,
            });
        }
        let (low, high) = if self.key < other.key {
            (self, other)
        } else {
            (other, self)
        };
        let low = low.take(false);
        (low, high.take(true))
    }

    fn take(&self, paired: bool) -> IrqGuard<'_, T> {
        let flags: u64;
        // SAFETY: CPL0; reads RFLAGS through the stack (balanced push/pop) and
        // masks maskable interrupts on this CPU only. No other memory is touched.
        unsafe { asm!("pushfq", "pop {}", "cli", out(reg) flags) };
        // LD-1 (audit F3): a handler never takes KERNEL/ASPACE, so the window
        // below can never open inside one.
        let in_handler = smp::irq_depth() != 0;
        if in_handler && self.level <= level::ASPACE {
            smp::lockdep_in_handler(self.level);
        }
        let recorded = tracked(self.level).then(|| {
            smp::with_lockdep(|dep| {
                if paired {
                    dep.acquire_paired(self.level, self.key)
                } else {
                    dep.acquire(self.level, self.key)
                }
            })
        });
        if let Some(Err(order)) = recorded {
            smp::lockdep_order(order);
        }
        // Step 5b (spec §4.3): a waiter for KERNEL/ASPACE opens a short
        // interrupt window between polls, so it can still acknowledge a
        // shootdown from the holder. Safe because lockdep (above) proved every
        // held level is below the handler levels (RUNQ and up) and LD-1 keeps
        // handlers off KERNEL/ASPACE; LD-2 opens it only outside a handler.
        // Step 5c-2 (BT-2/S3): on every CPU, BSP included, once all of its
        // unmasked sources have IDT gates (`irq_ready`). `test-smp-spin-irq-off`
        // is the deaf control.
        let window = self.level <= level::ASPACE
            && !in_handler
            && !cfg!(feature = "test-smp-spin-irq-off")
            && smp::irq_ready();
        let relax = || {
            if window {
                // SAFETY: CPL0 on a CPU whose IDT covers every unmasked
                // source; STI's shadow covers PAUSE, so a pending interrupt is
                // taken only before CLI. No lock this CPU holds is taken by a
                // handler, and the saved IF in `flags` is unchanged.
                unsafe { asm!("sti", "pause", "cli") };
            }
        };
        // DG-1 (audit F5): every native-smp image, release included, bounds the
        // wait in time; the deadline starts at the first contended poll. The
        // one-CPU profiles never have a waiter and spin unbounded.
        let guard = if cfg!(feature = "native-smp") {
            let mut deadline = None;
            self.inner.lock_with(|| {
                relax();
                let limit = *deadline.get_or_insert_with(smp::stuck_deadline);
                if smp::past(limit) {
                    smp::lock_stuck(self.level);
                }
            })
        } else {
            self.inner.lock_with(relax)
        };
        self.owner.store(smp::cpu_index() as u8, Ordering::Release);
        IrqGuard {
            guard: Some(guard),
            owner: &self.owner,
            level: self.level,
            key: self.key,
            interrupts: flags & (1 << 9) != 0,
        }
    }
}

/// Levels the lockdep word records (`smp_lock::Lockdep` holds 0..=63).
/// shortcut: SERIAL (70, audit F7) is the top level and a leaf (`write_raw`
/// takes no lock) and is held only with IF=0, so it is not recorded; a
/// re-entry would show up as `SMP:LOCK:STUCK:70` (DG-1). Widen `Lockdep`
/// when a level above 63 needs ordering.
const fn tracked(level: u8) -> bool {
    level < 64
}

/// `!Send` through the inner guard: never carried across a context switch.
pub struct IrqGuard<'a, T> {
    guard: Option<Guard<'a, T>>,
    owner: &'a AtomicU8,
    level: u8,
    key: u32,
    interrupts: bool,
}

impl<T> Deref for IrqGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.guard.as_ref().expect("live guard")
    }
}

impl<T> DerefMut for IrqGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.guard.as_mut().expect("live guard")
    }
}

impl<T> Drop for IrqGuard<'_, T> {
    fn drop(&mut self) {
        self.owner.store(FREE, Ordering::Release);
        drop(self.guard.take());
        if tracked(self.level) {
            if let Err(order) = smp::with_lockdep(|dep| dep.release(self.level, self.key)) {
                smp::lockdep_lifo(order);
            }
        }
        if self.interrupts {
            // BT-4: the last lock is gone and IF=1 returns outside a handler,
            // the next safe place for a reschedule deferred meanwhile.
            if smp::lockdep().load(Ordering::Relaxed) == 0 && smp::irq_depth() == 0 {
                smp::resched_on_release();
            }
            // SAFETY: restores the IF=1 this CPU had before `lock_irqsave`,
            // after the lock and its lockdep level are released.
            unsafe { asm!("sti", options(nomem, nostack)) };
        }
    }
}

#[cfg(feature = "native-smp")]
static OUTER: IrqLock<u32> = IrqLock::new(level::KERNEL, 0);
#[cfg(feature = "native-smp")]
static INNER: IrqLock<u32> = IrqLock::new(level::FRAMES, 0);

/// BSP, after `smp::start`, IF=0: the allowed order nests and fully unwinds;
/// the negative profile takes the reverse order and must stop with the marker.
#[cfg(feature = "native-smp")]
pub fn selftest() {
    {
        let mut outer = OUTER.lock_irqsave();
        let mut inner = INNER.lock_irqsave();
        *outer += 1;
        *inner += 1;
    }
    let flags: u64;
    // SAFETY: as in `lock_irqsave`, without changing IF.
    unsafe { asm!("pushfq", "pop {}", out(reg) flags) };
    let unwound = smp::lockdep().load(Ordering::Relaxed) == 0;
    if !unwound || flags & (1 << 9) != 0 || *OUTER.lock_irqsave() != 1 || *INNER.lock_irqsave() != 1
    {
        crate::serial::fail(b"SMP_LOCKDEP_SELFTEST");
    }
    if cfg!(feature = "test-smp-lockdep-order") {
        let _inner = INNER.lock_irqsave();
        let _outer = OUTER.lock_irqsave();
        crate::serial::fail(b"SMP_LOCKDEP_ORDER_MISSED");
    }
    crate::serial::write(b"SMP:LOCKDEP:PASS\n");
}

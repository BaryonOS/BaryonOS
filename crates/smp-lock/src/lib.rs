#![cfg_attr(not(test), no_std)]
#![deny(unsafe_op_in_unsafe_fn)]
//! K06 step 5a: FIFO ticket spinlock and lock-order checking (lockdep).
//!
//! Architecture-neutral: interrupt masking (`irqsave`) and per-CPU placement
//! of [`Lockdep`] live in the kernel; this crate holds only the parts that can
//! be stress-tested on the host. See the K06 SMP spec §4.

use core::{
    cell::UnsafeCell,
    hint::spin_loop,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU32, Ordering},
};

pub mod sched;

/// Fixed lock levels (outer -> inner). A CPU may only take a lock whose level
/// is strictly higher than every lock it already holds.
pub mod level {
    pub const KERNEL: u8 = 10;
    pub const ASPACE: u8 = 20;
    pub const FRAMES: u8 = 30;
    pub const RUNQ: u8 = 40;
    pub const SHOOTDOWN: u8 = 50;
    pub const ICR: u8 = 60;
    pub const SERIAL: u8 = 70;
}

/// The bounded spin gave up: the lock was not granted within `limit` polls.
/// The caller's ticket stays consumed, so the lock is permanently unusable;
/// only a terminal stop may follow.
#[derive(Debug, PartialEq, Eq)]
pub struct Stuck;

/// FIFO ticket lock (Mellor-Crummey & Scott 1991): no starvation under TCG.
pub struct TicketLock<T> {
    next: AtomicU32,
    owner: AtomicU32,
    value: UnsafeCell<T>,
}

// SAFETY: a `Guard` (the only path to `&mut T`) exists for at most one holder at
// a time: ticket `t` is served only while `owner == t`, and `owner` advances
// (Release) only when that guard drops. T: Send because the value moves
// between CPUs through the lock.
unsafe impl<T: Send> Sync for TicketLock<T> {}

impl<T> TicketLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            next: AtomicU32::new(0),
            owner: AtomicU32::new(0),
            value: UnsafeCell::new(value),
        }
    }

    /// Unbounded spin (release builds, after lockdep proved the order).
    pub fn lock(&self) -> Guard<'_, T> {
        self.lock_with(|| {})
    }

    /// Unbounded spin, calling `relax` between polls (step 5b: the kernel's
    /// `sti; pause; cli` window for KERNEL/ASPACE, spec §4.3).
    pub fn lock_with(&self, mut relax: impl FnMut()) -> Guard<'_, T> {
        let ticket = self.next.fetch_add(1, Ordering::Relaxed);
        while self.owner.load(Ordering::Acquire) != ticket {
            relax();
            spin_loop();
        }
        Guard::new(self)
    }

    /// Spin at most `limit` polls, calling `relax` between polls (e.g. the
    /// kernel's `sti; pause; cli` window, spec §4.3).
    pub fn lock_bounded(&self, limit: u32, mut relax: impl FnMut()) -> Result<Guard<'_, T>, Stuck> {
        let ticket = self.next.fetch_add(1, Ordering::Relaxed);
        for _ in 0..limit {
            if self.owner.load(Ordering::Acquire) == ticket {
                return Ok(Guard::new(self));
            }
            relax();
            spin_loop();
        }
        Err(Stuck)
    }

    /// Take the lock only if it is free right now (terminal NMI/#DF paths).
    pub fn try_lock(&self) -> Option<Guard<'_, T>> {
        let owner = self.owner.load(Ordering::Acquire);
        self.next
            .compare_exchange(
                owner,
                owner.wrapping_add(1),
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .ok()
            .map(|_| Guard::new(self))
    }
}

/// Exclusive access; `!Send` so it cannot be carried across a context switch
/// to another CPU (INV-7, compile-time part).
///
/// ```compile_fail
/// fn send<T: Send>(_: T) {}
/// let lock = smp_lock::TicketLock::new(0u32);
/// send(lock.lock());
/// ```
pub struct Guard<'a, T> {
    lock: &'a TicketLock<T>,
    _not_send: PhantomData<*const ()>,
}

impl<'a, T> Guard<'a, T> {
    fn new(lock: &'a TicketLock<T>) -> Self {
        Self {
            lock,
            _not_send: PhantomData,
        }
    }
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: aliasing — this guard is the sole holder (ticket served);
        // alignment — UnsafeCell<T> has T's layout; lifetime — bound to 'a and
        // to the guard, which releases the lock only on drop.
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: as in `deref`; `&mut self` makes this the only live borrow.
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock.owner.fetch_add(1, Ordering::Release);
    }
}

/// Lock-order violation: `wanted` is not above the highest held level.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Order {
    pub held: u8,
    pub wanted: u8,
}

/// Per-CPU set of held lock levels (bit per level, levels 0..=63). Touched
/// only by its own CPU with interrupts masked.
///
/// shortcut: one bit per level, so two RUNQ locks (spec §4.2, ascending CPU
/// index) cannot both be recorded; extend with a nesting count in step 5d.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Lockdep {
    held: u64,
}

impl Lockdep {
    pub const fn new() -> Self {
        Self { held: 0 }
    }
    pub const fn from_bits(held: u64) -> Self {
        Self { held }
    }
    pub const fn bits(self) -> u64 {
        self.held
    }

    fn highest(self) -> Option<u8> {
        (self.held != 0).then(|| 63 - self.held.leading_zeros() as u8)
    }

    /// Record `level` before spinning on the lock.
    pub fn acquire(&mut self, level: u8) -> Result<(), Order> {
        assert!(level < 64);
        match self.highest() {
            Some(held) if held >= level => Err(Order {
                held,
                wanted: level,
            }),
            _ => {
                self.held |= 1 << level;
                Ok(())
            }
        }
    }

    /// Forget `level`; releasing a level that is not held is a kernel bug.
    /// Release must be LIFO (audit F2): each guard restores its own saved IF,
    /// so dropping an outer guard first would re-enable interrupts while an
    /// inner (handler-level) lock is still held. `Err` leaves the set unchanged.
    pub fn release(&mut self, level: u8) -> Result<(), Order> {
        assert!(
            level < 64 && self.held & (1 << level) != 0,
            "lockdep: release of unheld level"
        );
        match self.highest() {
            Some(held) if held != level => Err(Order {
                held,
                wanted: level,
            }),
            _ => {
                self.held &= !(1 << level);
                Ok(())
            }
        }
    }

    /// At a context switch only the levels in `allowed` (the RUNQ hand-over,
    /// spec §6.3) may be held; returns the offending set otherwise.
    pub fn check_switch(self, allowed: u64) -> Result<(), u64> {
        match self.held & !allowed {
            0 => Ok(()),
            extra => Err(extra),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::Arc, thread, vec::Vec};

    #[test]
    fn mutual_exclusion_under_contention() {
        const THREADS: usize = 8;
        const ROUNDS: u64 = 20_000;
        let lock = Arc::new(TicketLock::new((0u64, 0u32)));
        let workers: Vec<_> = (0..THREADS)
            .map(|_| {
                let lock = Arc::clone(&lock);
                thread::spawn(move || {
                    for round in 0..ROUNDS {
                        let mut guard = lock.lock();
                        guard.1 += 1;
                        assert_eq!(guard.1, 1, "two holders inside the lock");
                        // Split read/write widens the window a broken lock would lose.
                        let value = guard.0;
                        if round % 64 == 0 {
                            thread::yield_now();
                        }
                        guard.0 = value + 1;
                        guard.1 -= 1;
                    }
                })
            })
            .collect();
        workers.into_iter().for_each(|w| w.join().unwrap());
        assert_eq!(lock.lock().0, THREADS as u64 * ROUNDS);
    }

    #[test]
    fn tickets_are_served_in_fifo_order() {
        let lock = Arc::new(TicketLock::new(Vec::new()));
        let first = lock.lock();
        let waiters: Vec<_> = (0..4u32)
            .map(|id| {
                let lock = Arc::clone(&lock);
                let before = lock.next.load(Ordering::Relaxed);
                let spawned = Arc::clone(&lock);
                let handle = thread::spawn(move || spawned.lock().push(id));
                // Each waiter has taken its ticket before the next one starts.
                while lock.next.load(Ordering::Relaxed) == before {
                    thread::yield_now();
                }
                handle
            })
            .collect();
        drop(first);
        waiters.into_iter().for_each(|w| w.join().unwrap());
        assert_eq!(*lock.lock(), [0, 1, 2, 3]);
    }

    #[test]
    fn try_lock_and_bounded_spin() {
        let lock = TicketLock::new(7);
        let held = lock.try_lock().expect("free lock");
        assert!(lock.try_lock().is_none());
        let mut polls = 0;
        assert_eq!(lock.lock_bounded(5, || polls += 1).err(), Some(Stuck));
        assert_eq!(polls, 5);
        drop(held);
        // The Stuck caller's ticket is consumed: the lock is never granted again.
        assert!(lock.lock_bounded(100, || {}).is_err());
        assert!(lock.try_lock().is_none());

        let fresh = TicketLock::new(1);
        assert_eq!(*fresh.lock_bounded(1, || {}).unwrap(), 1);
        assert_eq!(*fresh.try_lock().unwrap(), 1);
    }

    #[test]
    fn lock_with_relaxes_while_waiting() {
        use std::sync::atomic::AtomicU64;
        let lock = Arc::new(TicketLock::new(0u32));
        let relaxed = Arc::new(AtomicU64::new(0));
        let held = lock.lock();
        let waiter = {
            let (lock, relaxed) = (Arc::clone(&lock), Arc::clone(&relaxed));
            thread::spawn(move || {
                *lock.lock_with(|| {
                    relaxed.fetch_add(1, Ordering::Relaxed);
                }) += 1;
            })
        };
        // The holder releases only after the waiter has demonstrably relaxed.
        while relaxed.load(Ordering::Relaxed) == 0 {
            thread::yield_now();
        }
        assert_eq!(*held, 0, "waiter got in while the lock was held");
        drop(held);
        waiter.join().unwrap();
        assert_eq!(*lock.lock(), 1);
        // Uncontended: granted at once, relax never runs.
        let mut calls = 0;
        drop(TicketLock::new(()).lock_with(|| calls += 1));
        assert_eq!(calls, 0);
    }

    #[test]
    fn ticket_counter_wraps() {
        let lock = TicketLock::new(0);
        lock.next.store(u32::MAX, Ordering::Relaxed);
        lock.owner.store(u32::MAX, Ordering::Relaxed);
        drop(lock.lock());
        drop(lock.try_lock().expect("wrapped to 0"));
        assert_eq!(lock.owner.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn lockdep_enforces_ascending_order() {
        let mut dep = Lockdep::new();
        dep.acquire(level::KERNEL).unwrap();
        dep.acquire(level::FRAMES).unwrap();
        assert_eq!(
            dep.acquire(level::ASPACE),
            Err(Order {
                held: level::FRAMES,
                wanted: level::ASPACE
            })
        );
        assert_eq!(
            dep.acquire(level::FRAMES),
            Err(Order {
                held: 30,
                wanted: 30
            })
        );
        dep.release(level::FRAMES).unwrap();
        dep.acquire(level::ASPACE).unwrap();
        dep.release(level::ASPACE).unwrap();
        dep.release(level::KERNEL).unwrap();
        assert_eq!(dep, Lockdep::new());

        let mut reverse = Lockdep::new();
        reverse.acquire(level::FRAMES).unwrap();
        assert_eq!(
            reverse.acquire(level::KERNEL),
            Err(Order {
                held: 30,
                wanted: 10
            })
        );
        // A rejected acquire leaves the held set unchanged.
        assert_eq!(reverse.bits(), 1 << level::FRAMES);
    }

    #[test]
    fn lockdep_switch_allows_only_the_handover() {
        let runq = 1u64 << level::RUNQ;
        let mut dep = Lockdep::new();
        assert_eq!(dep.check_switch(runq), Ok(()));
        dep.acquire(level::RUNQ).unwrap();
        assert_eq!(dep.check_switch(runq), Ok(()));
        assert_eq!(dep.check_switch(0), Err(runq));
        let mut kernel = Lockdep::new();
        kernel.acquire(level::KERNEL).unwrap();
        assert_eq!(kernel.check_switch(runq), Err(1 << level::KERNEL));
    }

    #[test]
    #[should_panic(expected = "unheld")]
    fn lockdep_rejects_unbalanced_release() {
        let _ = Lockdep::new().release(level::KERNEL);
    }

    #[test]
    fn lockdep_rejects_out_of_order_release() {
        // F2: KERNEL (IF=1 saved) then RUNQ; dropping KERNEL first would `sti`
        // with RUNQ held, letting a tick deadlock on this CPU's own RUNQ.
        let mut dep = Lockdep::new();
        dep.acquire(level::KERNEL).unwrap();
        dep.acquire(level::RUNQ).unwrap();
        let both = dep.bits();
        assert_eq!(
            dep.release(level::KERNEL),
            Err(Order {
                held: level::RUNQ,
                wanted: level::KERNEL
            })
        );
        assert_eq!(dep.bits(), both);
        dep.release(level::RUNQ).unwrap();
        dep.release(level::KERNEL).unwrap();
        assert_eq!(dep, Lockdep::new());
    }
}

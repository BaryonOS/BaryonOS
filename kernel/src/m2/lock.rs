//! K06 step 5a (native-smp only): IRQ-safe ticket lock with per-CPU lockdep.
//!
//! Interrupt and lock contract (spec §4.1, §4.3): `lock_irqsave` saves
//! RFLAGS.IF, masks interrupts, records the level in this CPU's lockdep set
//! (a violation stops the VM), then spins. The guard releases the lock, drops
//! the level and only then restores IF. Lockdep is active in every native-smp
//! build; the bounded spin with its STUCK marker only in debug builds.
use super::smp;
use core::{
    arch::asm,
    ops::{Deref, DerefMut},
    sync::atomic::Ordering,
};
use smp_lock::{level, Guard, Lockdep, TicketLock};

pub struct IrqLock<T> {
    level: u8,
    name: &'static [u8],
    inner: TicketLock<T>,
}

impl<T> IrqLock<T> {
    pub const fn new(level: u8, name: &'static [u8], value: T) -> Self {
        Self {
            level,
            name,
            inner: TicketLock::new(value),
        }
    }

    pub fn lock_irqsave(&self) -> IrqGuard<'_, T> {
        let flags: u64;
        // SAFETY: CPL0; reads RFLAGS through the stack (balanced push/pop) and
        // masks maskable interrupts on this CPU only. No other memory is touched.
        unsafe { asm!("pushfq", "pop {}", "cli", out(reg) flags) };
        let held = smp::lockdep();
        let mut dep = Lockdep::from_bits(held.load(Ordering::Relaxed));
        if let Err(order) = dep.acquire(self.level) {
            smp::lockdep_order(order);
        }
        held.store(dep.bits(), Ordering::Relaxed);
        let guard = if cfg!(debug_assertions) {
            self.inner
                .lock_bounded(1 << 24, || {})
                .unwrap_or_else(|_| smp::lock_stuck(self.name))
        } else {
            self.inner.lock()
        };
        IrqGuard {
            guard: Some(guard),
            level: self.level,
            interrupts: flags & (1 << 9) != 0,
        }
    }
}

/// `!Send` through the inner guard: never carried across a context switch.
pub struct IrqGuard<'a, T> {
    guard: Option<Guard<'a, T>>,
    level: u8,
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
        drop(self.guard.take());
        let held = smp::lockdep();
        let mut dep = Lockdep::from_bits(held.load(Ordering::Relaxed));
        dep.release(self.level);
        held.store(dep.bits(), Ordering::Relaxed);
        if self.interrupts {
            // SAFETY: restores the IF=1 this CPU had before `lock_irqsave`,
            // after the lock and its lockdep level are released.
            unsafe { asm!("sti", options(nomem, nostack)) };
        }
    }
}

static OUTER: IrqLock<u32> = IrqLock::new(level::KERNEL, b"KERNEL", 0);
static INNER: IrqLock<u32> = IrqLock::new(level::FRAMES, b"FRAMES", 0);

/// BSP, after `smp::start`, IF=0: the allowed order nests and fully unwinds;
/// the negative profile takes the reverse order and must stop with the marker.
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

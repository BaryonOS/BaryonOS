#![forbid(unsafe_code)]
//! Explicitly qualified clocks exposed by the native runtime.
use core::sync::atomic::{AtomicBool, Ordering};

use crate::{Error, WaitCondition, IPC_WAIT_HORIZON_TICKS};

/// Nominal ticks per second in the supported single-CPU QEMU profile.
pub const MONOTONIC_HZ: u64 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WallClockQuality {
    /// Read from unauthenticated firmware/RTC state; never certificate time.
    UnauthenticatedRtc,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WallClockReading {
    pub unix_seconds: i64,
    pub quality: WallClockQuality,
}

impl WallClockReading {
    pub const fn unauthenticated_rtc(unix_seconds: i64) -> Self {
        Self {
            unix_seconds,
            quality: WallClockQuality::UnauthenticatedRtc,
        }
    }
}

/// One-shot cancellation flag that can be shared with a concurrent caller.
///
/// Cancellation is observed between monotone waits; the current ABI cannot
/// interrupt a wait already in the kernel.
pub struct TimerCancellation(AtomicBool);

impl TimerCancellation {
    pub const fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

impl Default for TimerCancellation {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
/// Absolute monotonic deadline plus its shared cancellation flag.
pub struct TimerRegistration<'a> {
    deadline: u64,
    cancellation: &'a TimerCancellation,
}

impl<'a> TimerRegistration<'a> {
    pub const fn at(deadline: u64, cancellation: &'a TimerCancellation) -> Self {
        Self {
            deadline,
            cancellation,
        }
    }

    pub const fn deadline(self) -> u64 {
        self.deadline
    }

    pub fn is_cancelled(self) -> bool {
        self.cancellation.is_cancelled()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitOutcome {
    Ready(u16),
    TimedOut,
    Cancelled,
}

pub trait WaitDriver {
    fn monotonic_ticks(&mut self) -> Result<u64, Error>;
    fn ipc_wait(&mut self, conditions: &[WaitCondition], deadline: u64) -> Result<u16, Error>;
}

fn chunk_deadline(now: u64, absolute_deadline: u64) -> u64 {
    absolute_deadline.min(now.saturating_add(CANCEL_POLL_TICKS.min(IPC_WAIT_HORIZON_TICKS)))
}

/// Maximum interval between cancellation checks for long waits.
///
/// The current clock is nominally 100 Hz, so this is roughly one second in
/// the supported profile. It is not a real-time guarantee.
pub const CANCEL_POLL_TICKS: u64 = MONOTONIC_HZ;

pub fn wait_until_with(
    driver: &mut impl WaitDriver,
    conditions: &[WaitCondition],
    timer: &TimerRegistration<'_>,
) -> Result<WaitOutcome, Error> {
    if timer.is_cancelled() {
        return Ok(WaitOutcome::Cancelled);
    }
    loop {
        let now = driver.monotonic_ticks()?;
        if now >= timer.deadline() {
            return Ok(WaitOutcome::TimedOut);
        }
        let next = chunk_deadline(now, timer.deadline());
        let ready = driver.ipc_wait(conditions, next)?;
        if timer.is_cancelled() {
            return Ok(WaitOutcome::Cancelled);
        }
        if ready != 0 {
            return Ok(WaitOutcome::Ready(ready));
        }
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
struct NativeDriver;

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
impl WaitDriver for NativeDriver {
    fn monotonic_ticks(&mut self) -> Result<u64, Error> {
        crate::native::monotonic_ticks()
    }

    fn ipc_wait(&mut self, conditions: &[WaitCondition], deadline: u64) -> Result<u16, Error> {
        crate::native::ipc_wait(conditions, deadline)
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub fn now_monotonic() -> Result<u64, Error> {
    crate::native::monotonic_ticks()
}

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub fn now_wall() -> Result<Option<WallClockReading>, Error> {
    match crate::native::wall_clock_utc() {
        Ok(reading) => Ok(Some(reading)),
        Err(Error::Kernel(-2)) => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub fn sleep_until(timer: &TimerRegistration<'_>) -> Result<WaitOutcome, Error> {
    wait_until_with(&mut NativeDriver, &[], timer)
}

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub fn wait_until(
    conditions: &[WaitCondition],
    timer: &TimerRegistration<'_>,
) -> Result<WaitOutcome, Error> {
    wait_until_with(&mut NativeDriver, conditions, timer)
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use crate::{Handle, ReceiveEndpoint, SendEndpoint};
    use std::vec;
    use std::vec::Vec;

    #[test]
    fn wall_clock_values_always_carry_untrusted_quality() {
        assert_eq!(
            WallClockReading::unauthenticated_rtc(-1),
            WallClockReading {
                unix_seconds: -1,
                quality: WallClockQuality::UnauthenticatedRtc,
            }
        );
    }

    #[test]
    fn timer_registration_accepts_boot_deadline_and_supports_shared_cancellation() {
        let cancellation = TimerCancellation::new();
        let timer = TimerRegistration::at(10, &cancellation);
        assert_eq!(timer.deadline(), 10);
        assert!(!timer.is_cancelled());
        cancellation.cancel();
        assert!(timer.is_cancelled());
        assert_eq!(TimerRegistration::at(0, &cancellation).deadline(), 0);
    }

    #[test]
    fn deadline_at_boot_is_immediately_expired_without_registering_a_wait() {
        let cancellation = TimerCancellation::new();
        let timer = TimerRegistration::at(0, &cancellation);
        let mut driver = FakeDriver {
            now: vec![0],
            ..Default::default()
        };
        assert_eq!(
            wait_until_with(&mut driver, &[], &timer).unwrap(),
            WaitOutcome::TimedOut
        );
        assert!(driver.deadlines.is_empty());
    }

    #[test]
    fn chunking_is_bounded_by_wait_horizon() {
        assert_eq!(chunk_deadline(100, 105), 105);
        assert_eq!(chunk_deadline(100, 100 + CANCEL_POLL_TICKS + 50), 200);
    }

    #[derive(Default)]
    struct FakeDriver<'a> {
        now: Vec<u64>,
        waits: Vec<u16>,
        deadlines: Vec<u64>,
        calls: usize,
        cancel_on_wait: Option<&'a TimerCancellation>,
    }

    impl WaitDriver for FakeDriver<'_> {
        fn monotonic_ticks(&mut self) -> Result<u64, Error> {
            let tick = *self
                .now
                .get(self.calls)
                .or_else(|| self.now.last())
                .ok_or(Error::InvalidReturn)?;
            Ok(tick)
        }

        fn ipc_wait(&mut self, _: &[WaitCondition], deadline: u64) -> Result<u16, Error> {
            self.deadlines.push(deadline);
            if let Some(cancellation) = self.cancel_on_wait {
                cancellation.cancel();
            }
            let value = *self
                .waits
                .get(self.calls)
                .or_else(|| self.waits.last())
                .ok_or(Error::InvalidReturn)?;
            self.calls += 1;
            Ok(value)
        }
    }

    fn recv(raw: u64) -> ReceiveEndpoint {
        ReceiveEndpoint::from_handle(Handle::from_raw(raw).unwrap())
    }

    fn send(raw: u64) -> SendEndpoint {
        SendEndpoint::from_handle(Handle::from_raw(raw).unwrap())
    }

    #[test]
    fn wait_until_returns_ready_mask_before_deadline() {
        let mut driver = FakeDriver {
            now: vec![10],
            waits: vec![0b11],
            ..Default::default()
        };
        let cancellation = TimerCancellation::new();
        let timer = TimerRegistration::at(500, &cancellation);
        let conditions = [
            WaitCondition::receive(recv(0x1_0000_0001)),
            WaitCondition::send(send(0x1_0000_0002)),
        ];
        assert_eq!(
            wait_until_with(&mut driver, &conditions, &timer).unwrap(),
            WaitOutcome::Ready(0b11)
        );
        assert_eq!(driver.deadlines, vec![10 + CANCEL_POLL_TICKS]);
    }

    #[test]
    fn wait_until_loops_until_timeout_across_horizon_chunks() {
        let mut driver = FakeDriver {
            now: vec![0, 1000, 1500, 2000],
            waits: vec![0, 0, 0],
            ..Default::default()
        };
        let cancellation = TimerCancellation::new();
        let timer = TimerRegistration::at(2000, &cancellation);
        assert_eq!(
            wait_until_with(&mut driver, &[], &timer).unwrap(),
            WaitOutcome::TimedOut
        );
        assert_eq!(driver.deadlines, vec![100, 1100, 1600]);
    }

    #[test]
    fn cancelled_timer_does_not_wait() {
        let cancellation = TimerCancellation::new();
        cancellation.cancel();
        let timer = TimerRegistration::at(2000, &cancellation);
        let mut driver = FakeDriver {
            now: vec![1],
            waits: vec![1],
            ..Default::default()
        };
        assert_eq!(
            wait_until_with(&mut driver, &[], &timer).unwrap(),
            WaitOutcome::Cancelled
        );
        assert!(driver.deadlines.is_empty());
    }

    #[test]
    fn cancellation_during_wait_is_observed_before_returning_ready() {
        let cancellation = TimerCancellation::new();
        let mut driver = FakeDriver {
            now: vec![1],
            waits: vec![1],
            cancel_on_wait: Some(&cancellation),
            ..Default::default()
        };
        let timer = TimerRegistration::at(1000, &cancellation);
        assert_eq!(
            wait_until_with(&mut driver, &[], &timer).unwrap(),
            WaitOutcome::Cancelled
        );
    }
}

#![forbid(unsafe_code)]
//! Explicitly qualified clocks exposed by the native runtime.
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

#[cfg(test)]
mod tests {
    use super::*;

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
}

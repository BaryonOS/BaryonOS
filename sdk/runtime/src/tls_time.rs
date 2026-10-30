#![forbid(unsafe_code)]
//! Admission boundary for TLS certificate time, not an authenticated time source.
//!
//! No currently implemented clock can produce [`CertificateTime`]. Callers
//! must propagate [`TimeUnavailable`] before certificate validation; they must
//! not substitute RTC, monotonic ticks, host time or a build timestamp.
//!
//! A wall-clock reading is not itself certificate time:
//! ```compile_fail
//! use nieuw_os_sdk_runtime::{clock::WallClockReading, tls_time::CertificateTime};
//! let time: CertificateTime = WallClockReading::unauthenticated_rtc(1_793_361_600);
//! ```
//!
//! Raw timestamps cannot construct a certificate-time value:
//! ```compile_fail
//! use nieuw_os_sdk_runtime::tls_time::CertificateTime;
//! let time = CertificateTime { unix_seconds: 1_793_361_600 };
//! ```
//!
//! Boot-monotonic ticks cannot be supplied as calendar time:
//! ```compile_fail
//! use nieuw_os_sdk_runtime::tls_time::certificate_time;
//! let ticks: u64 = 100;
//! let time = certificate_time(Some(ticks));
//! ```

use crate::clock::{WallClockQuality, WallClockReading};

/// Calendar seconds admitted for a future TLS verifier, not certificate approval.
///
/// There is deliberately no public constructor, parser, `Default` or numeric
/// conversion. A future authenticated provider requires a separately reviewed
/// authority, freshness, uncertainty and rollback contract before this type
/// can be issued. Values cannot be minted by safe callers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CertificateTime {
    unix_seconds: u64,
}

impl CertificateTime {
    pub const fn unix_seconds(self) -> u64 {
        self.unix_seconds
    }
}

/// A TLS caller must surface this failure rather than skip date verification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeUnavailable {
    MissingSource,
    UnauthenticatedRtc,
}

/// Fail closed for every currently available clock source.
///
/// A plausible, stable or nondecreasing RTC timestamp is still unauthenticated.
/// The exhaustive quality match intentionally requires a new decision if another
/// clock quality is introduced; it must not silently become trusted.
pub const fn certificate_time(
    reading: Option<WallClockReading>,
) -> Result<CertificateTime, TimeUnavailable> {
    match reading {
        None => Err(TimeUnavailable::MissingSource),
        Some(reading) => match reading.quality {
            WallClockQuality::UnauthenticatedRtc => Err(TimeUnavailable::UnauthenticatedRtc),
        },
    }
}

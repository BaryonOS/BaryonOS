use core::time::Duration;
use nieuw_os_sdk_runtime::{
    clock::WallClockReading,
    tls_time::{certificate_time, TimeUnavailable},
    Error as RuntimeError,
};
use rustls::{pki_types::UnixTime, time_provider::TimeProvider};

/// No RTC, monotonic, host-clock or build-time fallback is permitted.
pub fn checked_time(reading: Option<WallClockReading>) -> Result<UnixTime, TimeUnavailable> {
    let time = certificate_time(reading)?;
    Ok(UnixTime::since_unix_epoch(Duration::from_secs(
        time.unix_seconds(),
    )))
}

#[derive(Debug)]
pub struct CertificateClock {
    read: fn() -> Result<Option<WallClockReading>, RuntimeError>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Source(RuntimeError),
    Unavailable(TimeUnavailable),
}

impl CertificateClock {
    pub const fn new(read: fn() -> Result<Option<WallClockReading>, RuntimeError>) -> Self {
        Self { read }
    }

    /// A future TLS consumer must propagate this error before validation.
    pub fn checked_now(&self) -> Result<UnixTime, Error> {
        checked_time((self.read)().map_err(Error::Source)?).map_err(Error::Unavailable)
    }
}

impl TimeProvider for CertificateClock {
    fn current_time(&self) -> Option<UnixTime> {
        self.checked_now().ok()
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub fn native_reading() -> Result<Option<WallClockReading>, RuntimeError> {
    nieuw_os_sdk_runtime::native::wall_clock_utc().map(Some)
}

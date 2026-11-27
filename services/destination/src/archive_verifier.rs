//! The production archive verifier (spec 2.3, "Archive verifier"; route step
//! R5 precondition): [`VolumeVerifier`] runs `volume_core::archive::verify`,
//! the same check the portal runs before a restore, so that the service seals
//! and verifies exactly the bytes the portal accepts. No other archive
//! acceptance exists outside test code.
//!
//! Built only with the `volume-verifier` feature, so the core itself still
//! does not depend on `volume-core`.

use crate::service::{Source, Summary, Verifier};
use crate::Status;
use volume_core::archive;
use volume_core::MAX_LENGTH;

/// `volume_core::Status` for a service status. Both enums use the same codes
/// (spec 2.4); `volume_core::Status` is a superset.
fn to_volume(status: Status) -> volume_core::Status {
    volume_core::Status::from_code(status as i64)
}

/// Service status for a verifier result. Codes the service does not define
/// (only `Quota`, which `verify` never returns) become `Corrupt`, so an
/// unexpected result never accepts an entry.
fn from_volume(status: volume_core::Status) -> Status {
    match status {
        volume_core::Status::Ok => Status::Ok,
        volume_core::Status::Invalid => Status::Invalid,
        volume_core::Status::NoSys => Status::NoSys,
        volume_core::Status::Range => Status::Range,
        volume_core::Status::Access => Status::Access,
        volume_core::Status::Stale => Status::Stale,
        volume_core::Status::Io => Status::Io,
        volume_core::Status::Timeout => Status::Timeout,
        volume_core::Status::NoDevice => Status::NoDevice,
        volume_core::Status::Protocol => Status::Protocol,
        volume_core::Status::Unknown => Status::Unknown,
        volume_core::Status::Full => Status::Full,
        volume_core::Status::Corrupt => Status::Corrupt,
        volume_core::Status::NotFound => Status::NotFound,
        volume_core::Status::Busy => Status::Busy,
        volume_core::Status::Quota => Status::Corrupt,
        // Lane 0 only (H05-T5c): terminal device loss, never retried.
        volume_core::Status::Quarantined => Status::NoDevice,
    }
}

/// Adapts the service's extent stream to `volume_core::archive::Source` and
/// keeps the first source error, so that it is returned unchanged (2.3).
struct Stream<'a> {
    source: &'a mut dyn Source,
    error: Option<Status>,
}

impl archive::Source for Stream<'_> {
    fn get(&mut self, buffer: &mut [u8]) -> Result<usize, volume_core::Status> {
        match self.source.get(buffer) {
            Ok(count) => Ok(count),
            Err(status) => {
                self.error.get_or_insert(status);
                Err(to_volume(status))
            }
        }
    }
}

/// `volume_core::archive::verify` with its 64 KiB scratch buffer. The native
/// service holds one instance for its lifetime; it carries no state between
/// calls.
pub struct VolumeVerifier {
    scratch: [u8; MAX_LENGTH],
}

impl VolumeVerifier {
    pub const fn new() -> Self {
        Self {
            scratch: [0; MAX_LENGTH],
        }
    }
}

impl Default for VolumeVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl Verifier for VolumeVerifier {
    fn verify(&mut self, source: &mut dyn Source) -> Result<Summary, Status> {
        let mut stream = Stream {
            source,
            error: None,
        };
        let result = archive::verify(&mut stream, &mut self.scratch);
        // A source error ends the walk; whatever `verify` made of it, the
        // caller sees the source's own status (2.3).
        if let Some(status) = stream.error {
            return Err(status);
        }
        match result {
            Ok(summary) => Ok(Summary {
                generation: summary.generation,
                objects: summary.objects as u64,
            }),
            // `verify` never answers Ok as an error; keep it fail-closed.
            Err(volume_core::Status::Ok) => Err(Status::Corrupt),
            Err(status) => Err(from_volume(status)),
        }
    }
}

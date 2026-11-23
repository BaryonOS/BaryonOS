#![no_std]
#![forbid(unsafe_code)]
//! `SettingsMedia` over the M6e v6 volume IPC: settings slot `i` is volume
//! object `i` of the caller's own domain. The service authorizes every request
//! by the kernel-bound endpoint; this adapter holds no block or other rights.
//!
//! Integrity remains settings-core's SHA-256 plus the volume's FNV sums; both
//! detect accidental damage only (no authentication or rollback protection).

use nieuw_os_settings_core::{Error, MediaError, Settings, SettingsMedia, Theme, SLOT_LEN};
use volume_core::{
    checksum_update,
    client::{Client, Transport},
    Status, FNV_OFFSET, PER_DOMAIN,
};

/// Volume object names of the two A/B slots.
pub const NAMES: [&[u8]; 2] = [b"settings-a", b"settings-b"];

/// Report word 496 codes; tools/run_settings.py and the README use the same table.
pub const fn code(error: Error) -> u64 {
    match error {
        Error::InvalidValue => 1,
        Error::RecoveryRequired => 2,
        Error::NotInitialized => 3,
        Error::NotBlank => 4,
        Error::Corrupt => 5,
        Error::UnsupportedVersion => 6,
        Error::NoPrevious => 7,
        Error::GenerationExhausted => 8,
        Error::Media(MediaError::Io) => 9,
        Error::Media(MediaError::Full) => 10,
        Error::ReadbackMismatch => 11,
    }
}

/// Report word 488: the 8-byte canonical settings encoding as a little-endian u64.
pub fn encode(settings: Settings) -> u64 {
    let theme = match settings.theme() {
        Theme::System => 0,
        Theme::Light => 1,
        Theme::Dark => 2,
    };
    theme
        | u64::from(settings.reduced_motion()) << 8
        | u64::from(settings.text_scale_percent()) << 16
}

/// Per-domain fixture values, distinct so the host can tell the domains apart.
pub fn first(domain: usize) -> Settings {
    let (theme, scale, motion) = if domain == 0 {
        (Theme::Light, 100, false)
    } else {
        (Theme::System, 125, false)
    };
    Settings::new(theme, scale, motion).unwrap_or_else(|_| unreachable!())
}

pub fn second(domain: usize) -> Settings {
    let (theme, scale, motion) = if domain == 0 {
        (Theme::Dark, 150, true)
    } else {
        (Theme::Light, 200, true)
    };
    Settings::new(theme, scale, motion).unwrap_or_else(|_| unreachable!())
}

pub struct VolumeMedia<T: Transport> {
    pub client: Client<T>,
    domain: usize,
    /// Set by any write that did not end in an acknowledged COMMIT, including
    /// an `Unknown` exchange; never cleared, so `flush` cannot claim durability.
    poisoned: bool,
}

impl<T: Transport> VolumeMedia<T> {
    pub fn new(client: Client<T>, domain: usize) -> Self {
        Self {
            client,
            domain,
            poisoned: false,
        }
    }

    pub fn object(&self, slot: usize) -> u64 {
        (self.domain * PER_DOMAIN + slot) as u64
    }

    fn slot(slot: usize) -> Result<(), MediaError> {
        if slot < NAMES.len() {
            Ok(())
        } else {
            Err(MediaError::Io)
        }
    }
}

impl<T: Transport> SettingsMedia for VolumeMedia<T> {
    /// STAT then READ. An absent object (never written or deleted) reads as a
    /// blank slot: deleting both objects therefore makes the store look
    /// uninitialized again. Any other length than one slot is refused.
    fn read(&mut self, slot: usize, output: &mut [u8; SLOT_LEN]) -> Result<(), MediaError> {
        Self::slot(slot)?;
        let object = self.object(slot);
        let stat = match self.client.stat(object) {
            Ok(stat) => stat,
            Err((Status::NotFound, _)) => {
                output.fill(0);
                return Ok(());
            }
            Err(_) => return Err(MediaError::Io),
        };
        // The reported length is untrusted service output.
        if stat.length != SLOT_LEN {
            return Err(MediaError::Io);
        }
        let mut buffer = [0; SLOT_LEN];
        let read = self
            .client
            .read(object, stat.revision, SLOT_LEN, |offset, bytes| {
                // The client bounds offset + len by the requested length.
                buffer[offset..offset + bytes.len()].copy_from_slice(bytes);
            });
        if read != Ok(SLOT_LEN) || checksum_update(FNV_OFFSET, &buffer) != stat.content_sum {
            return Err(MediaError::Io);
        }
        *output = buffer;
        Ok(())
    }

    /// BEGIN, one APPEND and COMMIT against the revision STAT reported. Any
    /// failure before an acknowledged COMMIT aborts the staged upload.
    fn write(&mut self, slot: usize, data: &[u8; SLOT_LEN]) -> Result<(), MediaError> {
        Self::slot(slot)?;
        if self.poisoned {
            return Err(MediaError::Io);
        }
        self.poisoned = true;
        let object = self.object(slot);
        let expected = match self.client.stat(object) {
            Ok(stat) => stat.revision,
            Err((Status::NotFound, revision)) => revision,
            Err(_) => return Err(MediaError::Io),
        };
        let begin = self.client.next_request_id();
        let result =
            self.client
                .write(object, expected, NAMES[slot], SLOT_LEN, |offset, buffer| {
                    buffer.copy_from_slice(&data[offset..offset + buffer.len()]);
                });
        match result {
            Ok((revision, _)) if Some(revision) == expected.checked_add(1) => {
                self.poisoned = false;
                Ok(())
            }
            Ok(_) | Err(Status::Unknown) => Err(MediaError::Io),
            Err(status) => {
                // Best effort; after a failed COMMIT the session is already gone.
                let _ = self.client.abort(object, expected, begin);
                Err(if matches!(status, Status::Full | Status::Quota) {
                    MediaError::Full
                } else {
                    MediaError::Io
                })
            }
        }
    }

    /// Each acknowledged COMMIT is already flushed by the service, so flush
    /// succeeds exactly when no write since start failed or stayed uncertain.
    fn flush(&mut self) -> Result<(), MediaError> {
        if self.poisoned {
            Err(MediaError::Io)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests;

#![no_std]
#![forbid(unsafe_code)]

//! Bounded app-management protocol and streaming artifact transaction core.
//!
//! This crate does not create kernel authority. Package identities are
//! descriptors; the issuer/key lifecycle needed for durable publisher
//! verification is not implemented. Launch grants must arrive as
//! kernel-validated capabilities from the policy broker.

#[cfg(test)]
extern crate std;

pub mod catalog;
pub mod protocol;
pub mod store;

use app_package_core::SourceVerifiedExecutable;

pub const IDENTITY_LEN: usize = 144;
pub const MAX_PACKAGE_LEN: usize = app_package_core::MANIFEST_LEN + app_package_core::MAX_IMAGE_LEN;

/// Canonical package identity, including the selected source and its current
/// generation. A decoded identity is untrusted metadata, not launch authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackageIdentity {
    app_id: [u8; app_package_core::APP_ID_LEN],
    source_id: [u8; 32],
    source_generation: u32,
    version: u64,
    image_length: u32,
    image_sha256: [u8; 32],
}

impl PackageIdentity {
    pub fn from_verified(package: &SourceVerifiedExecutable<'_>) -> Self {
        let manifest = package.executable().manifest();
        Self {
            app_id: *manifest.app_id.as_bytes(),
            source_id: *package.source_id().as_bytes(),
            source_generation: package.source_generation(),
            version: manifest.version,
            image_length: manifest.image_length,
            image_sha256: manifest.image_sha256,
        }
    }

    pub const fn app_id(&self) -> &[u8; app_package_core::APP_ID_LEN] {
        &self.app_id
    }

    pub const fn source_id(&self) -> &[u8; 32] {
        &self.source_id
    }

    pub const fn source_generation(&self) -> u32 {
        self.source_generation
    }

    pub const fn version(&self) -> u64 {
        self.version
    }

    pub const fn image_length(&self) -> u32 {
        self.image_length
    }

    pub const fn image_sha256(&self) -> &[u8; 32] {
        &self.image_sha256
    }

    pub(crate) fn encode(&self, output: &mut [u8]) {
        debug_assert_eq!(output.len(), IDENTITY_LEN);
        output[..64].copy_from_slice(&self.app_id);
        output[64..96].copy_from_slice(&self.source_id);
        output[96..100].copy_from_slice(&self.source_generation.to_le_bytes());
        output[100..108].copy_from_slice(&self.version.to_le_bytes());
        output[108..112].copy_from_slice(&self.image_length.to_le_bytes());
        output[112..144].copy_from_slice(&self.image_sha256);
    }

    pub(crate) fn decode(input: &[u8]) -> Result<Self, protocol::ProtocolError> {
        if input.len() != IDENTITY_LEN {
            return Err(protocol::ProtocolError::InvalidLength);
        }
        let mut app_id = [0; app_package_core::APP_ID_LEN];
        app_id.copy_from_slice(&input[..64]);
        let mut source_id = [0; 32];
        source_id.copy_from_slice(&input[64..96]);
        let source_generation = u32::from_le_bytes(input[96..100].try_into().unwrap());
        let version = u64::from_le_bytes(input[100..108].try_into().unwrap());
        let image_length = u32::from_le_bytes(input[108..112].try_into().unwrap());
        let mut image_sha256 = [0; 32];
        image_sha256.copy_from_slice(&input[112..144]);
        if !valid_app_id(&app_id)
            || source_id.iter().all(|byte| *byte == 0)
            || source_generation == 0
            || version == 0
            || image_length == 0
            || image_length as usize > app_package_core::MAX_IMAGE_LEN
        {
            return Err(protocol::ProtocolError::InvalidIdentity);
        }
        Ok(Self {
            app_id,
            source_id,
            source_generation,
            version,
            image_length,
            image_sha256,
        })
    }
}

fn valid_app_id(bytes: &[u8; app_package_core::APP_ID_LEN]) -> bool {
    let length = match bytes.iter().position(|byte| *byte == 0) {
        Some(0) | None => return false,
        Some(length) => length,
    };
    if bytes[length..].iter().any(|byte| *byte != 0) {
        return false;
    }
    let mut segment_start = true;
    let mut previous = 0;
    let mut has_separator = false;
    for (index, byte) in bytes[..length].iter().copied().enumerate() {
        if byte == b'.' {
            if segment_start || previous == b'-' {
                return false;
            }
            segment_start = true;
            previous = byte;
            has_separator = true;
            continue;
        }
        let alphanumeric = byte.is_ascii_lowercase() || byte.is_ascii_digit();
        if !(alphanumeric || byte == b'-' && !segment_start) {
            return false;
        }
        if index + 1 == length && byte == b'-' {
            return false;
        }
        segment_start = false;
        previous = byte;
    }
    has_separator && !segment_start
}

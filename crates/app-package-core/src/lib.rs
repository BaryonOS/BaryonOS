#![no_std]
#![forbid(unsafe_code)]

//! Canonical verifier for a single-image Nieuw OS application package.
//!
//! A package contains one signed manifest followed by exact application-image
//! bytes. Executable validation reuses the kernel's user-ELF policy; physical
//! mapping and process launch remain the downstream loader's responsibility.
//! It has no install hooks, dependencies, or capability declarations. A
//! successful verification establishes publisher-key provenance and image
//! integrity only; it does not establish that the program is benign or
//! authorize it to access any object, device, network, or service.

use ed25519_dalek::{Signature as EdSignature, VerifyingKey as EdKey};
use ml_dsa::{
    EncodedSignature, EncodedVerifyingKey, MlDsa65, Signature as PqSignature, VerifyingKey as PqKey,
};
use sha2::{Digest, Sha256};

pub const MANIFEST_MAGIC: [u8; 8] = *b"NOSAPP01";
pub const FORMAT_VERSION: u32 = 1;
pub const MANIFEST_LEN: usize = 3584;
pub const SIGNED_LEN: usize = 144;
pub const ED25519_SIGNATURE_LEN: usize = 64;
pub const MLDSA65_SIGNATURE_LEN: usize = 3309;
pub const ED25519_PUBLIC_LEN: usize = 32;
pub const MLDSA65_PUBLIC_LEN: usize = 1952;
pub const APP_ID_LEN: usize = 64;
pub const MAX_IMAGE_LEN: usize = 2 * 1024 * 1024;
pub const MAX_SOURCES: usize = 8;
const SIGNATURES_END: usize = SIGNED_LEN + ED25519_SIGNATURE_LEN + MLDSA65_SIGNATURE_LEN;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reject {
    Format,
    UnsupportedAbi,
    UnknownKey,
    Signature,
    Digest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutableReject {
    Package(Reject),
    Image(kernel_core::Error),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceId([u8; 32]);

impl SourceId {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrustReject {
    InvalidSourceId,
    InvalidKeys,
    DuplicateSource,
    DuplicateKeys,
    Full,
    UnknownSource,
    RevokedSource,
    StaleSourceGeneration,
    GenerationExhausted,
    Executable(ExecutableReject),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppId([u8; APP_ID_LEN]);

impl AppId {
    pub fn as_bytes(&self) -> &[u8; APP_ID_LEN] {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        let length = self
            .0
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(APP_ID_LEN);
        core::str::from_utf8(&self.0[..length]).expect("AppId is validated ASCII")
    }
}

/// `major` occupies the high 16 bits and `minor` the low 16 bits.
pub const fn abi_version(major: u16, minor: u16) -> u32 {
    ((major as u32) << 16) | minor as u32
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Manifest {
    pub app_id: AppId,
    pub abi_min: u32,
    pub abi_max: u32,
    pub version: u64,
    pub image_length: u32,
    pub image_sha256: [u8; 32],
    pub ed25519_key_id: [u8; 8],
    pub mldsa65_key_id: [u8; 8],
}

impl Manifest {
    /// Decode the canonical, fixed-size manifest; signatures are checked later.
    pub fn decode(bytes: &[u8; MANIFEST_LEN]) -> Result<Self, Reject> {
        if bytes[..8] != MANIFEST_MAGIC {
            return Err(Reject::Format);
        }
        let u32_at =
            |offset: usize| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        let manifest = Self {
            app_id: AppId(bytes[12..12 + APP_ID_LEN].try_into().unwrap()),
            abi_min: u32_at(76),
            abi_max: u32_at(80),
            version: u64::from_le_bytes(bytes[84..92].try_into().unwrap()),
            image_length: u32_at(92),
            image_sha256: bytes[96..128].try_into().unwrap(),
            ed25519_key_id: bytes[128..136].try_into().unwrap(),
            mldsa65_key_id: bytes[136..144].try_into().unwrap(),
        };
        if u32_at(8) != FORMAT_VERSION
            || !valid_app_id(&manifest.app_id.0)
            || manifest.abi_min >> 16 == 0
            || manifest.abi_max >> 16 == 0
            || manifest.abi_min > manifest.abi_max
            || manifest.version == 0
            || manifest.image_length == 0
            || manifest.image_length as usize > MAX_IMAGE_LEN
            || bytes[SIGNATURES_END..].iter().any(|byte| *byte != 0)
        {
            return Err(Reject::Format);
        }
        Ok(manifest)
    }

    /// The exact canonical bytes covered by both signatures.
    fn signed_bytes(&self) -> [u8; SIGNED_LEN] {
        let mut bytes = [0u8; SIGNED_LEN];
        bytes[..8].copy_from_slice(&MANIFEST_MAGIC);
        bytes[8..12].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
        bytes[12..12 + APP_ID_LEN].copy_from_slice(&self.app_id.0);
        bytes[76..80].copy_from_slice(&self.abi_min.to_le_bytes());
        bytes[80..84].copy_from_slice(&self.abi_max.to_le_bytes());
        bytes[84..92].copy_from_slice(&self.version.to_le_bytes());
        bytes[92..96].copy_from_slice(&self.image_length.to_le_bytes());
        bytes[96..128].copy_from_slice(&self.image_sha256);
        bytes[128..136].copy_from_slice(&self.ed25519_key_id);
        bytes[136..144].copy_from_slice(&self.mldsa65_key_id);
        bytes
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrustAnchors {
    pub ed25519: [u8; ED25519_PUBLIC_LEN],
    pub mldsa65: [u8; MLDSA65_PUBLIC_LEN],
}

impl TrustAnchors {
    pub fn ed25519_key_id(&self) -> [u8; 8] {
        key_id(&self.ed25519)
    }

    pub fn mldsa65_key_id(&self) -> [u8; 8] {
        key_id(&self.mldsa65)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceStatus {
    Active,
    Revoked,
}

/// Public source metadata only; neither a trust token nor a storage encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceRecord {
    pub id: SourceId,
    pub anchors: TrustAnchors,
    pub generation: u32,
    pub state: SourceStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotError {
    BufferTooSmall,
    Full,
    InvalidSourceId,
    InvalidKeys,
    DuplicateSource,
    DuplicateKeys,
    InvalidGeneration,
}

pub struct VerifiedPackage<'a> {
    manifest: Manifest,
    image: &'a [u8],
}

impl VerifiedPackage<'_> {
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn image(&self) -> &[u8] {
        self.image
    }
}

pub struct VerifiedExecutable<'a> {
    package: VerifiedPackage<'a>,
    load_plan: kernel_core::user::Image,
}

impl VerifiedExecutable<'_> {
    pub fn manifest(&self) -> &Manifest {
        self.package.manifest()
    }

    pub fn image(&self) -> &[u8] {
        self.package.image()
    }

    pub fn load_plan(&self) -> &kernel_core::user::Image {
        &self.load_plan
    }
}

pub struct SourceVerifiedExecutable<'a> {
    source_id: SourceId,
    source_generation: u32,
    executable: VerifiedExecutable<'a>,
}

impl SourceVerifiedExecutable<'_> {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn source_generation(&self) -> u32 {
        self.source_generation
    }

    pub fn executable(&self) -> &VerifiedExecutable<'_> {
        &self.executable
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourceState {
    Empty,
    Active,
    Revoked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SourceSlot {
    id: SourceId,
    anchors: TrustAnchors,
    generation: u32,
    state: SourceState,
}

const EMPTY_SOURCE: SourceSlot = SourceSlot {
    id: SourceId([0; 32]),
    anchors: TrustAnchors {
        ed25519: [0; ED25519_PUBLIC_LEN],
        mldsa65: [0; MLDSA65_PUBLIC_LEN],
    },
    generation: 0,
    state: SourceState::Empty,
};

/// Bounded source policy. Mutations must be exposed only through a
/// user-authorized source manager; persistence and capability checks belong
/// to that service, not this in-memory core.
pub struct SourceRegistry {
    sources: [SourceSlot; MAX_SOURCES],
}

impl SourceRegistry {
    pub const fn new() -> Self {
        Self {
            sources: [EMPTY_SOURCE; MAX_SOURCES],
        }
    }

    /// Copies all occupied slots in order, including tombstones. A short
    /// buffer is left unchanged; bytes after the returned count are untouched.
    /// The records establish neither authenticity, freshness nor durability.
    pub fn export_snapshot(&self, out: &mut [SourceRecord]) -> Result<usize, SnapshotError> {
        let count = self
            .sources
            .iter()
            .filter(|source| source.state != SourceState::Empty)
            .count();
        if out.len() < count {
            return Err(SnapshotError::BufferTooSmall);
        }
        let records = self.sources.iter().filter_map(|source| {
            let state = match source.state {
                SourceState::Empty => return None,
                SourceState::Active => SourceStatus::Active,
                SourceState::Revoked => SourceStatus::Revoked,
            };
            Some(SourceRecord {
                id: source.id,
                anchors: source.anchors,
                generation: source.generation,
                state,
            })
        });
        for (destination, record) in out.iter_mut().zip(records) {
            *destination = record;
        }
        Ok(count)
    }

    /// Reconstructs a structural candidate, not an authorized active registry.
    /// The trusted service must keep it private until authentication, domain,
    /// freshness/rollback and atomic-publication checks have succeeded.
    /// No existing registry is replaced and no partial candidate is returned.
    pub fn from_snapshot(records: &[SourceRecord]) -> Result<Self, SnapshotError> {
        if records.len() > MAX_SOURCES {
            return Err(SnapshotError::Full);
        }
        for (index, record) in records.iter().enumerate() {
            if record.id.0.iter().all(|byte| *byte == 0) {
                return Err(SnapshotError::InvalidSourceId);
            }
            if !valid_anchors(&record.anchors) {
                return Err(SnapshotError::InvalidKeys);
            }
            let minimum = match record.state {
                SourceStatus::Active => 1,
                SourceStatus::Revoked => 2,
            };
            if record.generation < minimum {
                return Err(SnapshotError::InvalidGeneration);
            }
            if records[..index].iter().any(|other| other.id == record.id) {
                return Err(SnapshotError::DuplicateSource);
            }
            if records[..index]
                .iter()
                .any(|other| other.anchors == record.anchors)
            {
                return Err(SnapshotError::DuplicateKeys);
            }
        }
        let mut candidate = Self::new();
        for (slot, record) in candidate.sources.iter_mut().zip(records) {
            *slot = SourceSlot {
                id: record.id,
                anchors: record.anchors,
                generation: record.generation,
                state: match record.state {
                    SourceStatus::Active => SourceState::Active,
                    SourceStatus::Revoked => SourceState::Revoked,
                },
            };
        }
        Ok(candidate)
    }

    /// Add one explicitly selected source. Revoked slots remain tombstones and
    /// cannot be silently reused under the same identity.
    pub fn add(&mut self, id: SourceId, anchors: TrustAnchors) -> Result<u32, TrustReject> {
        if id.0.iter().all(|byte| *byte == 0) {
            return Err(TrustReject::InvalidSourceId);
        }
        if !valid_anchors(&anchors) {
            return Err(TrustReject::InvalidKeys);
        }
        if self
            .sources
            .iter()
            .any(|source| source.state != SourceState::Empty && source.id == id)
        {
            return Err(TrustReject::DuplicateSource);
        }
        if self
            .sources
            .iter()
            .any(|source| source.state != SourceState::Empty && source.anchors == anchors)
        {
            return Err(TrustReject::DuplicateKeys);
        }
        let slot = self
            .sources
            .iter_mut()
            .find(|source| source.state == SourceState::Empty)
            .ok_or(TrustReject::Full)?;
        *slot = SourceSlot {
            id,
            anchors,
            generation: 1,
            state: SourceState::Active,
        };
        Ok(1)
    }

    /// Replace a source's key pair only against the generation the caller
    /// inspected. The service must authorize this trust change separately.
    pub fn rotate(
        &mut self,
        id: SourceId,
        expected_generation: u32,
        anchors: TrustAnchors,
    ) -> Result<u32, TrustReject> {
        let index = self.source_index(id)?;
        let source = self.sources[index];
        if source.state == SourceState::Revoked {
            return Err(TrustReject::RevokedSource);
        }
        if source.generation != expected_generation {
            return Err(TrustReject::StaleSourceGeneration);
        }
        if !valid_anchors(&anchors) {
            return Err(TrustReject::InvalidKeys);
        }
        if source.anchors == anchors {
            return Err(TrustReject::DuplicateKeys);
        }
        if self.sources.iter().enumerate().any(|(other, entry)| {
            other != index && entry.state != SourceState::Empty && entry.anchors == anchors
        }) {
            return Err(TrustReject::DuplicateKeys);
        }
        let generation = source
            .generation
            .checked_add(1)
            .ok_or(TrustReject::GenerationExhausted)?;
        self.sources[index].anchors = anchors;
        self.sources[index].generation = generation;
        Ok(generation)
    }

    /// Revoke future package verification for this source. Existing installed
    /// apps are not removed by this policy operation.
    pub fn revoke(&mut self, id: SourceId, expected_generation: u32) -> Result<u32, TrustReject> {
        let index = self.source_index(id)?;
        let source = self.sources[index];
        if source.state == SourceState::Revoked {
            return Err(TrustReject::RevokedSource);
        }
        if source.generation != expected_generation {
            return Err(TrustReject::StaleSourceGeneration);
        }
        let generation = source
            .generation
            .checked_add(1)
            .ok_or(TrustReject::GenerationExhausted)?;
        self.sources[index].generation = generation;
        self.sources[index].state = SourceState::Revoked;
        Ok(generation)
    }

    pub fn verify_executable<'a>(
        &self,
        id: SourceId,
        expected_generation: u32,
        package: &'a [u8],
        current_abi: u32,
    ) -> Result<SourceVerifiedExecutable<'a>, TrustReject> {
        let source = self.sources[self.source_index(id)?];
        if source.state == SourceState::Revoked {
            return Err(TrustReject::RevokedSource);
        }
        if source.generation != expected_generation {
            return Err(TrustReject::StaleSourceGeneration);
        }
        let executable = verify_executable_package(package, &source.anchors, current_abi)
            .map_err(TrustReject::Executable)?;
        Ok(SourceVerifiedExecutable {
            source_id: id,
            source_generation: source.generation,
            executable,
        })
    }

    fn source_index(&self, id: SourceId) -> Result<usize, TrustReject> {
        self.sources
            .iter()
            .position(|source| source.state != SourceState::Empty && source.id == id)
            .ok_or(TrustReject::UnknownSource)
    }
}

impl Default for SourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Verify exact framing, current ABI compatibility, both source-selected
/// signatures, and the complete image digest before exposing the image bytes.
pub fn verify_package<'a>(
    package: &'a [u8],
    anchors: &TrustAnchors,
    current_abi: u32,
) -> Result<VerifiedPackage<'a>, Reject> {
    let manifest_end = MANIFEST_LEN;
    let manifest_bytes: &[u8; MANIFEST_LEN] = package
        .get(..manifest_end)
        .ok_or(Reject::Format)?
        .try_into()
        .map_err(|_| Reject::Format)?;
    let manifest = Manifest::decode(manifest_bytes)?;
    if current_abi < manifest.abi_min || current_abi > manifest.abi_max {
        return Err(Reject::UnsupportedAbi);
    }
    let image_end = manifest_end
        .checked_add(manifest.image_length as usize)
        .ok_or(Reject::Format)?;
    if package.len() != image_end {
        return Err(Reject::Format);
    }
    if manifest.ed25519_key_id != anchors.ed25519_key_id()
        || manifest.mldsa65_key_id != anchors.mldsa65_key_id()
    {
        return Err(Reject::UnknownKey);
    }
    let image = &package[manifest_end..image_end];
    let digest = Sha256::digest(image);
    if digest[..] != manifest.image_sha256[..] {
        return Err(Reject::Digest);
    }
    let message = manifest.signed_bytes();
    if message[..] != manifest_bytes[..SIGNED_LEN] {
        return Err(Reject::Format);
    }
    let ed_key = EdKey::from_bytes(&anchors.ed25519).map_err(|_| Reject::UnknownKey)?;
    let ed_signature = EdSignature::from_bytes(
        manifest_bytes[SIGNED_LEN..SIGNED_LEN + ED25519_SIGNATURE_LEN]
            .try_into()
            .unwrap(),
    );
    if ed_key.verify_strict(&message, &ed_signature).is_err() {
        return Err(Reject::Signature);
    }
    let encoded_key = EncodedVerifyingKey::<MlDsa65>::try_from(&anchors.mldsa65[..])
        .map_err(|_| Reject::UnknownKey)?;
    let pq_key = PqKey::<MlDsa65>::decode(&encoded_key);
    let signature_start = SIGNED_LEN + ED25519_SIGNATURE_LEN;
    let encoded_signature = EncodedSignature::<MlDsa65>::try_from(
        &manifest_bytes[signature_start..signature_start + MLDSA65_SIGNATURE_LEN],
    )
    .map_err(|_| Reject::Signature)?;
    let Some(pq_signature) = PqSignature::<MlDsa65>::decode(&encoded_signature) else {
        return Err(Reject::Signature);
    };
    if !pq_key.verify_with_context(&message, b"", &pq_signature) {
        return Err(Reject::Signature);
    }

    Ok(VerifiedPackage { manifest, image })
}

fn valid_anchors(anchors: &TrustAnchors) -> bool {
    EdKey::from_bytes(&anchors.ed25519).is_ok()
        && EncodedVerifyingKey::<MlDsa65>::try_from(&anchors.mldsa65[..]).is_ok()
}

/// Verify package provenance and apply the same user-ELF policy used by the
/// kernel before a verified image can be considered launchable.
pub fn verify_executable_package<'a>(
    package: &'a [u8],
    anchors: &TrustAnchors,
    current_abi: u32,
) -> Result<VerifiedExecutable<'a>, ExecutableReject> {
    let package =
        verify_package(package, anchors, current_abi).map_err(ExecutableReject::Package)?;
    let load_plan = kernel_core::user::parse(package.image()).map_err(ExecutableReject::Image)?;
    Ok(VerifiedExecutable { package, load_plan })
}

fn key_id(public: &[u8]) -> [u8; 8] {
    let digest = Sha256::digest(public);
    digest[..8].try_into().unwrap()
}

/// Checks canonical, zero-padded app-ID syntax, not publisher trust or grants.
pub fn valid_app_id(bytes: &[u8; APP_ID_LEN]) -> bool {
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

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    #[test]
    fn abi_version_encodes_major_and_minor_in_separate_halves() {
        assert_eq!(abi_version(0, 0), 0);
        assert_eq!(abi_version(1, 0), 0x0001_0000);
        assert_eq!(abi_version(0, 1), 1);
        assert_eq!(abi_version(u16::MAX, u16::MAX), u32::MAX);
        assert_eq!(abi_version(0x1234, 0xABCD), 0x1234_ABCD);
    }

    #[test]
    fn app_id_syntax_preserves_canonical_and_padding_boundaries() {
        for (text, expected) in [
            ("org.example.app", true),
            ("0.a", true),
            ("org.some-app", true),
            ("", false),
            ("app", false),
            (".app", false),
            ("app.", false),
            ("org..app", false),
            ("org.-app", false),
            ("org.app-", false),
            ("org-.app", false),
            ("Org.app", false),
            ("org_app.test", false),
            ("org/app.test", false),
        ] {
            let mut bytes = [0; APP_ID_LEN];
            bytes[..text.len()].copy_from_slice(text.as_bytes());
            assert_eq!(valid_app_id(&bytes), expected, "{text}");
        }
        let mut maximum = [b'a'; APP_ID_LEN];
        maximum[1] = b'.';
        assert!(!valid_app_id(&maximum));
        maximum[APP_ID_LEN - 1] = 0;
        assert!(valid_app_id(&maximum));
        for index in 3..APP_ID_LEN {
            let mut bytes = [0; APP_ID_LEN];
            bytes[..3].copy_from_slice(b"a.b");
            bytes[index] = 1;
            assert!(!valid_app_id(&bytes));
        }
    }
    use ed25519_dalek::{Signer, SigningKey as EdSigningKey};
    use ml_dsa::{MlDsa65, SigningKey as PqSigningKey};
    use std::{vec, vec::Vec};

    struct TestKeys {
        ed25519: EdSigningKey,
        mldsa65: PqSigningKey<MlDsa65>,
    }

    impl TestKeys {
        fn new(ed_seed: [u8; 32], pq_seed: [u8; 32]) -> Self {
            Self {
                ed25519: EdSigningKey::from_bytes(&ed_seed),
                mldsa65: PqSigningKey::<MlDsa65>::from_seed(&pq_seed.into()),
            }
        }

        fn anchors(&self) -> TrustAnchors {
            let encoded = self.mldsa65.expanded_key().verifying_key().encode();
            let mut mldsa65 = [0u8; MLDSA65_PUBLIC_LEN];
            mldsa65.copy_from_slice(&encoded);
            TrustAnchors {
                ed25519: self.ed25519.verifying_key().to_bytes(),
                mldsa65,
            }
        }

        fn package(&self, image: &[u8], version: u64) -> Vec<u8> {
            let anchors = self.anchors();
            let mut app_id = [0u8; APP_ID_LEN];
            app_id[..17].copy_from_slice(b"org.example.notes");
            let manifest = Manifest {
                app_id: AppId(app_id),
                abi_min: abi_version(1, 0),
                abi_max: abi_version(1, 9),
                version,
                image_length: image.len() as u32,
                image_sha256: Sha256::digest(image).into(),
                ed25519_key_id: anchors.ed25519_key_id(),
                mldsa65_key_id: anchors.mldsa65_key_id(),
            };
            let signed = manifest.signed_bytes();
            let mut bytes = vec![0u8; MANIFEST_LEN + image.len()];
            bytes[..SIGNED_LEN].copy_from_slice(&signed);
            let ed_signature = self.ed25519.sign(&signed);
            bytes[SIGNED_LEN..SIGNED_LEN + ED25519_SIGNATURE_LEN]
                .copy_from_slice(&ed_signature.to_bytes());
            let pq_signature = self
                .mldsa65
                .expanded_key()
                .sign_deterministic(&signed, b"")
                .unwrap()
                .encode();
            let signature_start = SIGNED_LEN + ED25519_SIGNATURE_LEN;
            bytes[signature_start..signature_start + MLDSA65_SIGNATURE_LEN]
                .copy_from_slice(&pq_signature);
            bytes[MANIFEST_LEN..].copy_from_slice(image);
            bytes
        }
    }

    fn image() -> Vec<u8> {
        (0..4096u32).map(|i| (i % 251) as u8).collect()
    }

    fn valid_user_elf() -> Vec<u8> {
        let mut bytes = vec![0; 0x2010];
        bytes[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
        bytes[16..18].copy_from_slice(&2u16.to_le_bytes());
        bytes[18..20].copy_from_slice(&62u16.to_le_bytes());
        bytes[20..24].copy_from_slice(&1u32.to_le_bytes());
        bytes[24..32].copy_from_slice(&0x4000_0000u64.to_le_bytes());
        bytes[32..40].copy_from_slice(&64u64.to_le_bytes());
        bytes[52..54].copy_from_slice(&64u16.to_le_bytes());
        bytes[54..56].copy_from_slice(&56u16.to_le_bytes());
        bytes[56..58].copy_from_slice(&2u16.to_le_bytes());
        for (index, (flags, offset, address, memory_size)) in [
            (5u32, 0x1000u64, 0x4000_0000u64, 16u64),
            (6u32, 0x2000u64, 0x4000_1000u64, 128u64),
        ]
        .into_iter()
        .enumerate()
        {
            let header = 64 + index * 56;
            bytes[header..header + 4].copy_from_slice(&1u32.to_le_bytes());
            bytes[header + 4..header + 8].copy_from_slice(&flags.to_le_bytes());
            bytes[header + 8..header + 16].copy_from_slice(&offset.to_le_bytes());
            bytes[header + 16..header + 24].copy_from_slice(&address.to_le_bytes());
            bytes[header + 32..header + 40].copy_from_slice(&16u64.to_le_bytes());
            bytes[header + 40..header + 48].copy_from_slice(&memory_size.to_le_bytes());
            bytes[header + 48..header + 56].copy_from_slice(&4096u64.to_le_bytes());
        }
        bytes
    }

    fn keys() -> TestKeys {
        TestKeys::new([7; 32], [9; 32])
    }

    fn snapshot_record(tag: u8, generation: u32, state: SourceStatus) -> SourceRecord {
        SourceRecord {
            id: SourceId::from_bytes([tag; 32]),
            anchors: TestKeys::new([tag; 32], [tag; 32]).anchors(),
            generation,
            state,
        }
    }

    fn export(registry: &SourceRegistry) -> Vec<SourceRecord> {
        let sentinel = snapshot_record(99, 1, SourceStatus::Active);
        let mut out = vec![sentinel; MAX_SOURCES];
        let count = registry.export_snapshot(&mut out).unwrap();
        out.truncate(count);
        out
    }

    #[test]
    fn snapshots_roundtrip_empty_full_and_mixed_records_in_slot_order() {
        assert!(export(&SourceRegistry::from_snapshot(&[]).unwrap()).is_empty());
        let mut registry = SourceRegistry::new();
        for tag in 1..=MAX_SOURCES as u8 {
            registry
                .add(
                    SourceId::from_bytes([tag; 32]),
                    TestKeys::new([tag; 32], [tag; 32]).anchors(),
                )
                .unwrap();
        }
        registry
            .rotate(
                SourceId::from_bytes([1; 32]),
                1,
                TestKeys::new([20; 32], [20; 32]).anchors(),
            )
            .unwrap();
        registry.revoke(SourceId::from_bytes([2; 32]), 1).unwrap();
        let records = export(&registry);
        assert_eq!(records.len(), MAX_SOURCES);
        assert_eq!(records[0].generation, 2);
        assert_eq!(records[1].state, SourceStatus::Revoked);
        let mut restored = SourceRegistry::from_snapshot(&records).unwrap();
        assert_eq!(export(&restored), records);
        assert_eq!(
            restored.add(SourceId::from_bytes([2; 32]), records[1].anchors),
            Err(TrustReject::DuplicateSource)
        );
        assert_eq!(
            restored.add(SourceId::from_bytes([99; 32]), records[1].anchors),
            Err(TrustReject::DuplicateKeys)
        );
        assert_eq!(
            restored.add(
                SourceId::from_bytes([99; 32]),
                TestKeys::new([99; 32], [99; 32]).anchors()
            ),
            Err(TrustReject::Full)
        );
        assert_eq!(
            registry.rotate(records[0].id, 2, records[2].anchors),
            restored.rotate(records[0].id, 2, records[2].anchors)
        );
        assert_eq!(
            registry.revoke(records[1].id, 2),
            restored.revoke(records[1].id, 2)
        );
        let next = TestKeys::new([30; 32], [30; 32]).anchors();
        assert_eq!(
            registry.rotate(records[0].id, 2, next),
            restored.rotate(records[0].id, 2, next)
        );
        assert_eq!(export(&registry), export(&restored));
    }

    #[test]
    fn snapshot_export_is_all_or_nothing_and_preserves_unused_output() {
        let records = [
            snapshot_record(1, 1, SourceStatus::Active),
            snapshot_record(2, 2, SourceStatus::Revoked),
        ];
        let registry = SourceRegistry::from_snapshot(&records).unwrap();
        let sentinel = snapshot_record(99, 1, SourceStatus::Active);
        assert_eq!(
            registry.export_snapshot(&mut []),
            Err(SnapshotError::BufferTooSmall)
        );
        let mut small = [sentinel; 1];
        assert_eq!(
            registry.export_snapshot(&mut small),
            Err(SnapshotError::BufferTooSmall)
        );
        assert_eq!(small, [sentinel]);
        let mut large = [sentinel; 3];
        assert_eq!(registry.export_snapshot(&mut large), Ok(2));
        assert_eq!(large[..2], records);
        assert_eq!(large[2], sentinel);
        assert_eq!(export(&registry), records);
        assert_eq!(SourceRegistry::new().export_snapshot(&mut []), Ok(0));
    }

    #[test]
    fn snapshot_reconstruction_rejects_complete_invalid_input_without_mutation() {
        let first = snapshot_record(1, 1, SourceStatus::Active);
        let second = snapshot_record(2, 2, SourceStatus::Revoked);
        let registry = SourceRegistry::from_snapshot(&[first, second]).unwrap();
        let mut invalid_id = second;
        invalid_id.id = SourceId::from_bytes([0; 32]);
        let mut invalid_keys = second;
        invalid_keys.anchors.ed25519 = (0..=u8::MAX)
            .map(|byte| [byte; ED25519_PUBLIC_LEN])
            .find(|key| EdKey::from_bytes(key).is_err())
            .unwrap();
        let mut duplicate_id = second;
        duplicate_id.id = first.id;
        let mut duplicate_keys = second;
        duplicate_keys.anchors = first.anchors;
        let mut zero_generation = second;
        zero_generation.generation = 0;
        let mut early_revoke = second;
        early_revoke.generation = 1;
        let mut zero_active = first;
        zero_active.generation = 0;
        for (invalid, expected) in [
            (invalid_id, SnapshotError::InvalidSourceId),
            (invalid_keys, SnapshotError::InvalidKeys),
            (duplicate_id, SnapshotError::DuplicateSource),
            (duplicate_keys, SnapshotError::DuplicateKeys),
            (zero_generation, SnapshotError::InvalidGeneration),
            (early_revoke, SnapshotError::InvalidGeneration),
            (zero_active, SnapshotError::InvalidGeneration),
        ] {
            assert!(matches!(
                SourceRegistry::from_snapshot(&[first, invalid]),
                Err(error) if error == expected
            ));
            assert_eq!(export(&registry), [first, second]);
        }
        assert!(matches!(
            SourceRegistry::from_snapshot(&[first; MAX_SOURCES + 1]),
            Err(SnapshotError::Full)
        ));
    }

    #[test]
    fn snapshots_preserve_max_generations_and_revocation_without_wraparound() {
        let records = [
            snapshot_record(1, u32::MAX, SourceStatus::Active),
            snapshot_record(2, u32::MAX, SourceStatus::Revoked),
        ];
        let mut registry = SourceRegistry::from_snapshot(&records).unwrap();
        let next = snapshot_record(3, 1, SourceStatus::Active).anchors;
        assert_eq!(
            registry.rotate(records[0].id, u32::MAX, next),
            Err(TrustReject::GenerationExhausted)
        );
        assert_eq!(
            registry.revoke(records[0].id, u32::MAX),
            Err(TrustReject::GenerationExhausted)
        );
        assert_eq!(
            registry.rotate(records[1].id, u32::MAX, next),
            Err(TrustReject::RevokedSource)
        );
        assert_eq!(
            registry.revoke(records[1].id, u32::MAX),
            Err(TrustReject::RevokedSource)
        );
        assert_eq!(
            registry.revoke(records[0].id, u32::MAX - 1),
            Err(TrustReject::StaleSourceGeneration)
        );
        assert_eq!(export(&registry), records);
    }

    #[test]
    fn snapshots_keep_existing_keypair_policy_and_returned_rotation_state() {
        let first = snapshot_record(1, 1, SourceStatus::Active);
        let mut shared_ed = snapshot_record(2, 1, SourceStatus::Active);
        shared_ed.anchors.ed25519 = first.anchors.ed25519;
        let mut shared_pq = snapshot_record(3, 1, SourceStatus::Active);
        shared_pq.anchors.mldsa65 = first.anchors.mldsa65;
        let mut registry = SourceRegistry::new();
        for record in [first, shared_ed, shared_pq] {
            registry.add(record.id, record.anchors).unwrap();
        }
        let records = export(&registry);
        assert_eq!(
            export(&SourceRegistry::from_snapshot(&records).unwrap()),
            records
        );
        let next = snapshot_record(4, 1, SourceStatus::Active).anchors;
        assert_eq!(registry.rotate(first.id, 1, next), Ok(2));
        assert_eq!(registry.rotate(first.id, 2, first.anchors), Ok(3));
        let returned = export(&registry);
        assert_eq!(returned[0].anchors, first.anchors);
        assert_eq!(returned[0].generation, 3);
        assert_eq!(
            export(&SourceRegistry::from_snapshot(&returned).unwrap()),
            returned
        );
    }

    #[test]
    fn reconstructed_registry_keeps_verification_and_stale_generation_checks() {
        let keys = keys();
        let id = SourceId::from_bytes([1; 32]);
        let records = [SourceRecord {
            id,
            anchors: keys.anchors(),
            generation: 7,
            state: SourceStatus::Active,
        }];
        let mut registry = SourceRegistry::from_snapshot(&records).unwrap();
        let image = valid_user_elf();
        let package = keys.package(&image, 1);
        let verified = registry
            .verify_executable(id, 7, &package, abi_version(1, 3))
            .unwrap();
        assert_eq!(verified.source_generation(), 7);
        assert_eq!(verified.executable().image(), image);
        assert!(matches!(
            registry.verify_executable(id, 6, &package, abi_version(1, 3)),
            Err(TrustReject::StaleSourceGeneration)
        ));
        registry.revoke(id, 7).unwrap();
        let revoked = SourceRegistry::from_snapshot(&export(&registry)).unwrap();
        assert!(matches!(
            revoked.verify_executable(id, 8, &package, abi_version(1, 3)),
            Err(TrustReject::RevokedSource)
        ));
        // A previously returned value is not proof of current source authority.
        assert_eq!(verified.source_generation(), 7);
    }

    #[test]
    fn verifies_exact_image_and_source_selected_hybrid_signatures() {
        let keys = keys();
        let package = keys.package(&image(), 1);
        let verified = verify_package(&package, &keys.anchors(), abi_version(1, 9)).unwrap();
        assert_eq!(verified.manifest().app_id.as_str(), "org.example.notes");
        assert_eq!(verified.manifest().version, 1);
        assert_eq!(verified.image(), image());
    }

    #[test]
    fn verified_executable_uses_kernel_user_elf_policy() {
        let keys = keys();
        let image = valid_user_elf();
        let package = keys.package(&image, 1);
        let verified =
            verify_executable_package(&package, &keys.anchors(), abi_version(1, 3)).unwrap();
        assert_eq!(verified.image(), image);
        assert_eq!(verified.manifest().version, 1);
        assert_eq!(verified.load_plan().entry, 0x4000_0000);
        assert_eq!(verified.load_plan().count, 2);
        assert_eq!(verified.load_plan().pages, 2);
    }

    #[test]
    fn source_registry_selects_and_revokes_independent_sources() {
        let first = keys();
        let second = TestKeys::new([1; 32], [2; 32]);
        let first_id = SourceId::from_bytes([1; 32]);
        let second_id = SourceId::from_bytes([2; 32]);
        let mut registry = SourceRegistry::new();
        assert_eq!(registry.add(first_id, first.anchors()), Ok(1));
        assert_eq!(registry.add(second_id, second.anchors()), Ok(1));

        let image = valid_user_elf();
        let first_package = first.package(&image, 1);
        let second_package = second.package(&image, 1);
        let verified = registry
            .verify_executable(first_id, 1, &first_package, abi_version(1, 3))
            .unwrap();
        assert_eq!(verified.source_id(), first_id);
        assert_eq!(verified.source_generation(), 1);
        assert!(matches!(
            registry.verify_executable(first_id, 1, &second_package, abi_version(1, 3)),
            Err(TrustReject::Executable(ExecutableReject::Package(
                Reject::UnknownKey
            )))
        ));

        assert_eq!(registry.revoke(first_id, 1), Ok(2));
        assert!(matches!(
            registry.verify_executable(first_id, 2, &first_package, abi_version(1, 3)),
            Err(TrustReject::RevokedSource)
        ));
        assert_eq!(
            registry
                .verify_executable(second_id, 1, &second_package, abi_version(1, 3))
                .unwrap()
                .source_id(),
            second_id
        );
        assert_eq!(
            registry.add(first_id, first.anchors()),
            Err(TrustReject::DuplicateSource)
        );
    }

    #[test]
    fn source_rotation_is_generation_checked_and_rejects_old_keys() {
        let original = keys();
        let rotated = TestKeys::new([3; 32], [4; 32]);
        let id = SourceId::from_bytes([9; 32]);
        let mut registry = SourceRegistry::new();
        registry.add(id, original.anchors()).unwrap();

        assert_eq!(
            registry.rotate(id, 1, original.anchors()),
            Err(TrustReject::DuplicateKeys)
        );
        assert_eq!(
            registry.rotate(id, 0, rotated.anchors()),
            Err(TrustReject::StaleSourceGeneration)
        );
        assert_eq!(registry.rotate(id, 1, rotated.anchors()), Ok(2));
        let image = valid_user_elf();
        let old_package = original.package(&image, 1);
        let new_package = rotated.package(&image, 2);
        assert!(matches!(
            registry.verify_executable(id, 2, &old_package, abi_version(1, 3)),
            Err(TrustReject::Executable(ExecutableReject::Package(
                Reject::UnknownKey
            )))
        ));
        assert_eq!(
            registry
                .verify_executable(id, 2, &new_package, abi_version(1, 3))
                .unwrap()
                .executable()
                .manifest()
                .version,
            2
        );
    }

    #[test]
    fn source_registry_rejects_invalid_duplicates_and_capacity_overflow() {
        let first = keys();
        let mut registry = SourceRegistry::new();
        assert_eq!(
            registry.add(SourceId::from_bytes([0; 32]), first.anchors()),
            Err(TrustReject::InvalidSourceId)
        );
        let first_id = SourceId::from_bytes([1; 32]);
        assert_eq!(registry.add(first_id, first.anchors()), Ok(1));
        assert_eq!(
            registry.add(SourceId::from_bytes([2; 32]), first.anchors()),
            Err(TrustReject::DuplicateKeys)
        );

        for slot in 1..MAX_SOURCES {
            let id = SourceId::from_bytes([slot as u8 + 1; 32]);
            let keys = TestKeys::new([slot as u8 + 20; 32], [slot as u8 + 40; 32]);
            assert_eq!(registry.add(id, keys.anchors()), Ok(1));
        }
        let overflow_keys = TestKeys::new([101; 32], [102; 32]);
        assert_eq!(
            registry.add(SourceId::from_bytes([99; 32]), overflow_keys.anchors()),
            Err(TrustReject::Full)
        );
    }

    #[test]
    fn signed_package_with_invalid_user_elf_is_not_launchable() {
        let keys = keys();
        let mut image = valid_user_elf();
        image[68..72].copy_from_slice(&7u32.to_le_bytes());
        let package = keys.package(&image, 1);
        assert!(verify_package(&package, &keys.anchors(), abi_version(1, 3)).is_ok());
        assert!(matches!(
            verify_executable_package(&package, &keys.anchors(), abi_version(1, 3)),
            Err(ExecutableReject::Image(kernel_core::Error::Permission))
        ));
    }

    #[test]
    fn rejects_wrong_keys_signature_digest_and_non_exact_framing() {
        let keys = keys();
        let package = keys.package(&image(), 1);
        let other_keys = TestKeys::new([1; 32], [2; 32]);
        assert_eq!(
            verify_package(&package, &other_keys.anchors(), abi_version(1, 9)).err(),
            Some(Reject::UnknownKey)
        );

        let mut changed_manifest = package.clone();
        changed_manifest[20] ^= 1;
        assert_eq!(
            verify_package(&changed_manifest, &keys.anchors(), abi_version(1, 9)).err(),
            Some(Reject::Signature)
        );
        for offset in [SIGNED_LEN + 3, SIGNED_LEN + ED25519_SIGNATURE_LEN + 100] {
            let mut changed_signature = package.clone();
            changed_signature[offset] ^= 1;
            assert_eq!(
                verify_package(&changed_signature, &keys.anchors(), abi_version(1, 9)).err(),
                Some(Reject::Signature),
                "signature byte {offset}"
            );
        }
        let mut changed_image = package.clone();
        *changed_image.last_mut().unwrap() ^= 1;
        assert_eq!(
            verify_package(&changed_image, &keys.anchors(), abi_version(1, 9)).err(),
            Some(Reject::Digest)
        );
        let mut trailing = package.clone();
        trailing.push(0);
        assert_eq!(
            verify_package(&trailing, &keys.anchors(), abi_version(1, 9)).err(),
            Some(Reject::Format)
        );
        assert_eq!(
            verify_package(
                &package[..MANIFEST_LEN - 1],
                &keys.anchors(),
                abi_version(1, 9)
            )
            .err(),
            Some(Reject::Format)
        );
    }

    #[test]
    fn rejects_unsupported_abi_and_noncanonical_manifests_before_signatures() {
        let keys = keys();
        let package = keys.package(&image(), 1);
        assert_eq!(
            verify_package(&package, &keys.anchors(), abi_version(2, 0)).err(),
            Some(Reject::UnsupportedAbi)
        );

        for (offset, value) in [(12usize, b'.'), (12, b'A'), (78, 0), (8, 2)] {
            let mut malformed = package.clone();
            malformed[offset] = value;
            assert_eq!(
                verify_package(&malformed, &keys.anchors(), abi_version(1, 9)).err(),
                Some(Reject::Format),
                "offset {offset}"
            );
        }
        let mut noncanonical_id = package;
        noncanonical_id[12 + 18] = b'x';
        assert_eq!(
            verify_package(&noncanonical_id, &keys.anchors(), abi_version(1, 9)).err(),
            Some(Reject::Format)
        );
    }
}

#![no_std]
#![forbid(unsafe_code)]

//! Canonical verifier for a single-image Nieuw OS application package.
//!
//! A package contains one signed manifest followed by exact application-image
//! bytes. The downstream image loader is responsible for executable validation.
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

fn key_id(public: &[u8]) -> [u8; 8] {
    let digest = Sha256::digest(public);
    digest[..8].try_into().unwrap()
}

fn valid_app_id(bytes: &[u8; APP_ID_LEN]) -> bool {
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

    fn keys() -> TestKeys {
        TestKeys::new([7; 32], [9; 32])
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

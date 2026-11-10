#![no_std]
#![forbid(unsafe_code)]
//! B03 update core (ADR-049): the signed manifest format, hybrid
//! Ed25519 + ML-DSA-65 verification, test anchors behind an opt-in feature,
//! the slot store
//! layout, the loader's slot selection policy with a version floor, and the
//! two-phase install plan. Shared by the UEFI loader, the ring-3 update
//! service and the host signing tool so all three agree byte for byte.
//! No cryptographic primitive is implemented here.
use ed25519_dalek::{Signature as EdSignature, SigningKey as EdSigningKey, VerifyingKey as EdKey};
use ml_dsa::{
    EncodedSignature, EncodedVerifyingKey, MlDsa65, Signature as PqSignature,
    SigningKey as PqSigningKey, VerifyingKey as PqKey,
};
use sha2::{Digest, Sha256};

pub mod policy;
#[cfg(feature = "test-signing-anchors")]
pub mod test_policy;

pub const SECTOR: usize = 512;
pub const MANIFEST_MAGIC: [u8; 8] = *b"NOSMAN01";
pub const MANIFEST_LEN: usize = 3500;
pub const MANIFEST_SECTORS: u32 = 7;
/// Canonical signed region: everything before the signatures.
pub const SIGNED_LEN: usize = 84;
pub const ED25519_SIGNATURE: usize = 64;
pub const MLDSA65_SIGNATURE: usize = 3309;
pub const ED25519_PUBLIC: usize = 32;
pub const MLDSA65_PUBLIC: usize = 1952;
pub const TARGET_KERNEL: u32 = 1;
pub const FORMAT_VERSION: u32 = 1;
/// Largest image a slot holds: 2 MiB = 4096 sectors (a kernel image with
/// its embedded ring-3 programs is about 1.5 MiB).
pub const IMAGE_CAPACITY_SECTORS: u32 = 4096;
pub const MAX_IMAGE_LEN: u32 = IMAGE_CAPACITY_SECTORS * SECTOR as u32;

pub const STORE_MAGIC: [u8; 8] = *b"NOSSLOT1";
/// Fixed slot table (sectors). The header repeats it; decoders check equality.
pub const HEADER_SECTOR: u32 = 0;
pub const SLOT_A_MANIFEST: u32 = 1;
pub const SLOT_A_IMAGE: u32 = SLOT_A_MANIFEST + MANIFEST_SECTORS;
pub const SLOT_B_MANIFEST: u32 = SLOT_A_IMAGE + IMAGE_CAPACITY_SECTORS;
pub const SLOT_B_IMAGE: u32 = SLOT_B_MANIFEST + MANIFEST_SECTORS;
pub const INBOX_MANIFEST: u32 = SLOT_B_IMAGE + IMAGE_CAPACITY_SECTORS;
pub const INBOX_IMAGE: u32 = INBOX_MANIFEST + MANIFEST_SECTORS;
pub const STORE_SECTORS: u32 = INBOX_IMAGE + IMAGE_CAPACITY_SECTORS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reject {
    Format,
    UnknownKey,
    Signature,
    Digest,
    Downgrade,
    Read,
    KeyRevoked,
    KeyExpired,
    KeyNotYetValid,
    KeyPolicy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Manifest {
    pub target: u32,
    pub image_length: u32,
    pub version: u64,
    pub min_version: u64,
    pub image_sha256: [u8; 32],
    pub ed25519_key_id: [u8; 8],
    pub mldsa_key_id: [u8; 8],
}

impl Manifest {
    /// Canonical signed bytes (84), independent of any signature.
    pub fn signed_bytes(&self) -> [u8; SIGNED_LEN] {
        let mut b = [0u8; SIGNED_LEN];
        b[..8].copy_from_slice(&MANIFEST_MAGIC);
        b[8..12].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
        b[12..16].copy_from_slice(&self.target.to_le_bytes());
        b[16..20].copy_from_slice(&self.image_length.to_le_bytes());
        b[20..28].copy_from_slice(&self.version.to_le_bytes());
        b[28..36].copy_from_slice(&self.min_version.to_le_bytes());
        b[36..68].copy_from_slice(&self.image_sha256);
        b[68..76].copy_from_slice(&self.ed25519_key_id);
        b[76..84].copy_from_slice(&self.mldsa_key_id);
        b
    }
    /// Structural decode only: magic, versions, bounds and canonical zero tail.
    pub fn decode(bytes: &[u8; MANIFEST_LEN]) -> Result<Self, Reject> {
        if bytes[..8] != MANIFEST_MAGIC {
            return Err(Reject::Format);
        }
        let u32_at = |at: usize| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        let u64_at = |at: usize| u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap());
        if u32_at(8) != FORMAT_VERSION {
            return Err(Reject::Format);
        }
        let manifest = Self {
            target: u32_at(12),
            image_length: u32_at(16),
            version: u64_at(20),
            min_version: u64_at(28),
            image_sha256: bytes[36..68].try_into().unwrap(),
            ed25519_key_id: bytes[68..76].try_into().unwrap(),
            mldsa_key_id: bytes[76..84].try_into().unwrap(),
        };
        if manifest.target != TARGET_KERNEL
            || manifest.image_length == 0
            || manifest.image_length > MAX_IMAGE_LEN
            || manifest.version == 0
            || manifest.min_version > manifest.version
            || bytes[SIGNED_LEN + ED25519_SIGNATURE + MLDSA65_SIGNATURE..]
                .iter()
                .any(|b| *b != 0)
        {
            return Err(Reject::Format);
        }
        Ok(manifest)
    }
}

/// Pinned public keys. Key IDs are the first 8 bytes of SHA-256(public key).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrustAnchors {
    pub ed25519: [u8; ED25519_PUBLIC],
    pub mldsa65: [u8; MLDSA65_PUBLIC],
}

pub fn key_id(public: &[u8]) -> [u8; 8] {
    let digest = sha256(public);
    digest[..8].try_into().unwrap()
}

/// SHA-256 of arbitrary bytes (the one digest this format uses).
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

impl TrustAnchors {
    pub fn ed25519_key_id(&self) -> [u8; 8] {
        key_id(&self.ed25519)
    }
    pub fn mldsa_key_id(&self) -> [u8; 8] {
        key_id(&self.mldsa65)
    }
}

/// Cryptographic verification against one key pair, without lifecycle policy.
/// Boot/install callers must use `policy::TrustPolicy::verify_manifest` instead.
/// Both signatures cover the same canonical bytes.
pub fn verify_manifest(
    bytes: &[u8; MANIFEST_LEN],
    anchors: &TrustAnchors,
) -> Result<Manifest, Reject> {
    let manifest = Manifest::decode(bytes)?;
    if manifest.ed25519_key_id != anchors.ed25519_key_id()
        || manifest.mldsa_key_id != anchors.mldsa_key_id()
    {
        return Err(Reject::UnknownKey);
    }
    let message = manifest.signed_bytes();
    if message[..] != bytes[..SIGNED_LEN] {
        return Err(Reject::Format);
    }
    let ed_key = EdKey::from_bytes(&anchors.ed25519).map_err(|_| Reject::UnknownKey)?;
    let ed_sig = EdSignature::from_bytes(
        bytes[SIGNED_LEN..SIGNED_LEN + ED25519_SIGNATURE]
            .try_into()
            .unwrap(),
    );
    if ed_key.verify_strict(&message, &ed_sig).is_err() {
        return Err(Reject::Signature);
    }
    let encoded_key = EncodedVerifyingKey::<MlDsa65>::try_from(&anchors.mldsa65[..])
        .map_err(|_| Reject::UnknownKey)?;
    let pq_key = PqKey::<MlDsa65>::decode(&encoded_key);
    let start = SIGNED_LEN + ED25519_SIGNATURE;
    let encoded_sig =
        EncodedSignature::<MlDsa65>::try_from(&bytes[start..start + MLDSA65_SIGNATURE])
            .map_err(|_| Reject::Signature)?;
    let Some(pq_sig) = PqSignature::<MlDsa65>::decode(&encoded_sig) else {
        return Err(Reject::Signature);
    };
    if !pq_key.verify_with_context(&message, b"", &pq_sig) {
        return Err(Reject::Signature);
    }
    Ok(manifest)
}

/// Image binding: exact length and SHA-256 as recorded in the manifest.
pub fn verify_image(manifest: &Manifest, image: &[u8]) -> Result<(), Reject> {
    if image.len() != manifest.image_length as usize {
        return Err(Reject::Digest);
    }
    let digest = Sha256::digest(image);
    if digest[..] != manifest.image_sha256[..] {
        return Err(Reject::Digest);
    }
    Ok(())
}

/// Incremental SHA-256 for callers that read an image sector by sector.
pub struct ImageDigest {
    hasher: Sha256,
    remaining: usize,
}
impl ImageDigest {
    pub fn new(manifest: &Manifest) -> Self {
        Self {
            hasher: Sha256::new(),
            remaining: manifest.image_length as usize,
        }
    }
    /// Feed the next chunk; bytes beyond the image length are ignored (sector padding).
    pub fn update(&mut self, chunk: &[u8]) {
        let take = core::cmp::min(chunk.len(), self.remaining);
        self.hasher.update(&chunk[..take]);
        self.remaining -= take;
    }
    pub fn finish(self, manifest: &Manifest) -> Result<(), Reject> {
        if self.remaining != 0 {
            return Err(Reject::Digest);
        }
        let digest = self.hasher.finalize();
        if digest[..] != manifest.image_sha256[..] {
            return Err(Reject::Digest);
        }
        Ok(())
    }
}

/// Signing (host tool and tests). Both signatures cover the canonical bytes.
pub struct SigningKeys {
    pub ed25519: EdSigningKey,
    pub mldsa65: PqSigningKey<MlDsa65>,
}
impl SigningKeys {
    pub fn from_seeds(ed25519_seed: &[u8; 32], mldsa_seed: &[u8; 32]) -> Self {
        Self {
            ed25519: EdSigningKey::from_bytes(ed25519_seed),
            mldsa65: PqSigningKey::<MlDsa65>::from_seed(&(*mldsa_seed).into()),
        }
    }
    pub fn anchors(&self) -> TrustAnchors {
        let encoded = self.mldsa65.expanded_key().verifying_key().encode();
        let mut mldsa65 = [0u8; MLDSA65_PUBLIC];
        mldsa65.copy_from_slice(&encoded);
        TrustAnchors {
            ed25519: self.ed25519.verifying_key().to_bytes(),
            mldsa65,
        }
    }
    pub fn sign(&self, manifest: &Manifest) -> Result<[u8; MANIFEST_LEN], Reject> {
        use ed25519_dalek::Signer;
        let anchors = self.anchors();
        if manifest.ed25519_key_id != anchors.ed25519_key_id()
            || manifest.mldsa_key_id != anchors.mldsa_key_id()
        {
            return Err(Reject::UnknownKey);
        }
        let message = manifest.signed_bytes();
        let mut out = [0u8; MANIFEST_LEN];
        out[..SIGNED_LEN].copy_from_slice(&message);
        let ed = self.ed25519.sign(&message);
        out[SIGNED_LEN..SIGNED_LEN + ED25519_SIGNATURE].copy_from_slice(&ed.to_bytes());
        let pq = self
            .mldsa65
            .expanded_key()
            .sign_deterministic(&message, b"")
            .map_err(|_| Reject::Signature)?;
        let start = SIGNED_LEN + ED25519_SIGNATURE;
        out[start..start + MLDSA65_SIGNATURE].copy_from_slice(&pq.encode());
        Ok(out)
    }
}

/// Build the manifest for an image under the given anchors.
pub fn manifest_for(
    image: &[u8],
    version: u64,
    min_version: u64,
    anchors: &TrustAnchors,
) -> Result<Manifest, Reject> {
    if image.is_empty()
        || image.len() > MAX_IMAGE_LEN as usize
        || version == 0
        || min_version > version
    {
        return Err(Reject::Format);
    }
    let digest = Sha256::digest(image);
    Ok(Manifest {
        target: TARGET_KERNEL,
        image_length: image.len() as u32,
        version,
        min_version,
        image_sha256: digest.into(),
        ed25519_key_id: anchors.ed25519_key_id(),
        mldsa_key_id: anchors.mldsa_key_id(),
    })
}

/// Slot store header (sector 0). The table is fixed; the header repeats it so
/// an independent reader can check the layout it assumes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub active: u8,
    pub generation: u64,
}

impl Header {
    pub fn encode(&self) -> [u8; SECTOR] {
        let mut b = [0u8; SECTOR];
        b[..8].copy_from_slice(&STORE_MAGIC);
        b[8..12].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
        b[12..16].copy_from_slice(&u32::from(self.active).to_le_bytes());
        b[16..24].copy_from_slice(&self.generation.to_le_bytes());
        for (i, value) in [
            SLOT_A_MANIFEST,
            SLOT_A_IMAGE,
            IMAGE_CAPACITY_SECTORS,
            SLOT_B_MANIFEST,
            SLOT_B_IMAGE,
            IMAGE_CAPACITY_SECTORS,
            INBOX_MANIFEST,
            INBOX_IMAGE,
            IMAGE_CAPACITY_SECTORS,
        ]
        .into_iter()
        .enumerate()
        {
            b[24 + i * 4..28 + i * 4].copy_from_slice(&value.to_le_bytes());
        }
        b
    }
    pub fn decode(sector: &[u8; SECTOR]) -> Result<Self, Reject> {
        if sector[..8] != STORE_MAGIC {
            return Err(Reject::Format);
        }
        let u32_at = |at: usize| u32::from_le_bytes(sector[at..at + 4].try_into().unwrap());
        if u32_at(8) != FORMAT_VERSION {
            return Err(Reject::Format);
        }
        let active = u32_at(12);
        if active > 1 {
            return Err(Reject::Format);
        }
        let generation = u64::from_le_bytes(sector[16..24].try_into().unwrap());
        let header = Self {
            active: active as u8,
            generation,
        };
        // The table and the tail must match this crate's constants exactly.
        if header.encode()[24..] != sector[24..] {
            return Err(Reject::Format);
        }
        Ok(header)
    }
}

pub const fn manifest_sector(slot: u8) -> u32 {
    if slot == 0 {
        SLOT_A_MANIFEST
    } else {
        SLOT_B_MANIFEST
    }
}
pub const fn image_sector(slot: u8) -> u32 {
    if slot == 0 {
        SLOT_A_IMAGE
    } else {
        SLOT_B_IMAGE
    }
}
pub const fn other(slot: u8) -> u8 {
    1 - slot
}
pub const fn image_sectors(length: u32) -> u32 {
    length.div_ceil(SECTOR as u32)
}

/// Outcome of inspecting one slot: a verified manifest or the first rejection.
pub type SlotResult = Result<Manifest, Reject>;

/// Loader policy: the header's active slot first, then the other; a verified
/// manifest below the floor is a downgrade. Returns the chosen slot, its
/// manifest and the per-slot rejection reasons for the serial log.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choice {
    pub slot: u8,
    pub manifest: Manifest,
}

pub fn choose(
    header: &Header,
    slots: [SlotResult; 2],
    floor: u64,
) -> (Option<Choice>, [Option<Reject>; 2]) {
    // Fields are public; only `Header::decode` enforces this. Fail closed.
    if header.active > 1 {
        return (None, [Some(Reject::Format); 2]);
    }
    let mut reasons = [None; 2];
    let mut chosen = None;
    for candidate in [header.active, other(header.active)] {
        match slots[usize::from(candidate)] {
            Ok(manifest) if manifest.version < floor => {
                reasons[usize::from(candidate)] = Some(Reject::Downgrade);
            }
            Ok(manifest) => {
                if chosen.is_none() {
                    chosen = Some(Choice {
                        slot: candidate,
                        manifest,
                    });
                }
            }
            Err(reject) => reasons[usize::from(candidate)] = Some(reject),
        }
    }
    (chosen, reasons)
}

/// The update service's decision for an inbox bundle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    /// Install into this slot (the inactive one) and then switch the pointer.
    Install {
        target: u8,
        version: u64,
    },
    UpToDate,
    Rejected(Reject),
}

pub fn decide(header: &Header, current_version: u64, floor: u64, inbox: SlotResult) -> Decision {
    match inbox {
        _ if header.active > 1 => Decision::Rejected(Reject::Format),
        Err(reject) => Decision::Rejected(reject),
        Ok(manifest) if manifest.version < floor || manifest.version < manifest.min_version => {
            Decision::Rejected(Reject::Downgrade)
        }
        Ok(manifest) if manifest.version <= current_version => Decision::UpToDate,
        Ok(manifest) => Decision::Install {
            target: other(header.active),
            version: manifest.version,
        },
    }
}

/// The header to publish after a completed install: pointer to the target,
/// generation + 1. Written last, after image and manifest are flushed.
pub fn header_after_install(header: &Header, target: u8) -> Header {
    Header {
        active: target,
        generation: header.generation.wrapping_add(1),
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use hex_literal::hex;

    fn keys() -> SigningKeys {
        SigningKeys::from_seeds(&[7u8; 32], &[9u8; 32])
    }
    fn image() -> std::vec::Vec<u8> {
        (0..70_000u32).map(|i| (i % 251) as u8).collect()
    }
    fn signed(version: u64) -> ([u8; MANIFEST_LEN], Manifest, TrustAnchors) {
        let keys = keys();
        let anchors = keys.anchors();
        let manifest = manifest_for(&image(), version, 1, &anchors).unwrap();
        (keys.sign(&manifest).unwrap(), manifest, anchors)
    }

    #[test]
    fn ed25519_rfc8032_test_vector_1_verifies_with_the_same_api() {
        let public = hex!("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a");
        let signature = hex!(
            "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e06522490155"
            "5fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b"
        );
        let key = EdKey::from_bytes(&public).unwrap();
        assert!(key
            .verify_strict(b"", &EdSignature::from_bytes(&signature))
            .is_ok());
        assert!(key
            .verify_strict(b"x", &EdSignature::from_bytes(&signature))
            .is_err());
        let seed = hex!("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60");
        assert_eq!(
            EdSigningKey::from_bytes(&seed).verifying_key().to_bytes(),
            public
        );
    }

    #[test]
    fn mldsa65_sizes_match_fips_204_and_signing_is_deterministic() {
        let keys = keys();
        let anchors = keys.anchors();
        assert_eq!(anchors.mldsa65.len(), 1952);
        let sig = keys
            .mldsa65
            .expanded_key()
            .sign_deterministic(b"m", b"")
            .unwrap()
            .encode();
        assert_eq!(sig.len(), 3309);
        let again = keys
            .mldsa65
            .expanded_key()
            .sign_deterministic(b"m", b"")
            .unwrap()
            .encode();
        assert_eq!(sig, again);
    }

    #[test]
    fn manifest_round_trips_and_both_signatures_are_required() {
        let (bytes, manifest, anchors) = signed(3);
        assert_eq!(verify_manifest(&bytes, &anchors), Ok(manifest));
        assert_eq!(Manifest::decode(&bytes), Ok(manifest));
        assert_eq!(verify_image(&manifest, &image()), Ok(()));
        // Tamper with each region: signed bytes, Ed25519, ML-DSA, padding.
        for (position, expected) in [
            (20usize, Reject::Signature),
            (40, Reject::Signature),
            (SIGNED_LEN + 3, Reject::Signature),
            (SIGNED_LEN + ED25519_SIGNATURE + 100, Reject::Signature),
            (
                SIGNED_LEN + ED25519_SIGNATURE + MLDSA65_SIGNATURE - 1,
                Reject::Signature,
            ),
            (MANIFEST_LEN - 1, Reject::Format),
        ] {
            let mut bad = bytes;
            bad[position] ^= 0x01;
            assert_eq!(
                verify_manifest(&bad, &anchors),
                Err(expected),
                "position {position}"
            );
        }
        // A manifest signed by other keys is unknown here, even if well-formed.
        let others = SigningKeys::from_seeds(&[1u8; 32], &[2u8; 32]);
        let foreign = manifest_for(&image(), 3, 1, &others.anchors()).unwrap();
        let foreign_bytes = others.sign(&foreign).unwrap();
        assert_eq!(
            verify_manifest(&foreign_bytes, &anchors),
            Err(Reject::UnknownKey)
        );
        // Mixed: our Ed25519 key id with foreign ML-DSA signature bytes.
        let mut mixed = bytes;
        let start = SIGNED_LEN + ED25519_SIGNATURE;
        mixed[start..start + MLDSA65_SIGNATURE]
            .copy_from_slice(&foreign_bytes[start..start + MLDSA65_SIGNATURE]);
        assert_eq!(verify_manifest(&mixed, &anchors), Err(Reject::Signature));
        // Image binding.
        let mut changed = image();
        changed[12345] ^= 0x80;
        assert_eq!(verify_image(&manifest, &changed), Err(Reject::Digest));
        assert_eq!(
            verify_image(&manifest, &image()[..69_999]),
            Err(Reject::Digest)
        );
        let mut incremental = ImageDigest::new(&manifest);
        for chunk in image().chunks(SECTOR) {
            let mut sector = [0u8; SECTOR];
            sector[..chunk.len()].copy_from_slice(chunk);
            incremental.update(&sector);
        }
        assert_eq!(incremental.finish(&manifest), Ok(()));
    }

    #[test]
    fn structural_rejections_come_before_any_signature_check() {
        let (bytes, _, anchors) = signed(2);
        let mut magic = bytes;
        magic[0] = b'X';
        assert_eq!(verify_manifest(&magic, &anchors), Err(Reject::Format));
        let mut target = bytes;
        target[12] = 2;
        assert_eq!(verify_manifest(&target, &anchors), Err(Reject::Format));
        let mut zero_version = bytes;
        zero_version[20..28].fill(0);
        assert_eq!(
            verify_manifest(&zero_version, &anchors),
            Err(Reject::Format)
        );
        let mut too_big = bytes;
        too_big[16..20].copy_from_slice(&(MAX_IMAGE_LEN + 1).to_le_bytes());
        assert_eq!(verify_manifest(&too_big, &anchors), Err(Reject::Format));
        let mut floor_above = bytes;
        floor_above[28..36].copy_from_slice(&9u64.to_le_bytes());
        assert_eq!(verify_manifest(&floor_above, &anchors), Err(Reject::Format));
        assert!(manifest_for(&[], 1, 0, &anchors).is_err());
        assert!(manifest_for(&[1], 0, 0, &anchors).is_err());
    }

    #[test]
    fn header_round_trips_and_pins_the_fixed_table() {
        let header = Header {
            active: 1,
            generation: 7,
        };
        let bytes = header.encode();
        assert_eq!(Header::decode(&bytes), Ok(header));
        let mut bad_active = bytes;
        bad_active[12] = 2;
        assert_eq!(Header::decode(&bad_active), Err(Reject::Format));
        let mut moved_table = bytes;
        moved_table[28] ^= 1;
        assert_eq!(Header::decode(&moved_table), Err(Reject::Format));
        let mut tail = bytes;
        tail[511] = 1;
        assert_eq!(Header::decode(&tail), Err(Reject::Format));
        assert_eq!(SLOT_A_IMAGE, 8);
        assert_eq!(SLOT_B_MANIFEST, 4104);
        assert_eq!(INBOX_IMAGE, 8214);
        assert_eq!(STORE_SECTORS, 12310);
        assert_eq!(image_sectors(1), 1);
        assert_eq!(image_sectors(70_000), 137);
        assert_eq!(
            header_after_install(&header, 0),
            Header {
                active: 0,
                generation: 8
            }
        );
    }

    #[test]
    fn selection_prefers_active_falls_back_and_enforces_the_floor() {
        let (_, v1, _) = signed(1);
        let (_, v2, _) = signed(2);
        let header_a = Header {
            active: 0,
            generation: 1,
        };
        let header_b = Header {
            active: 1,
            generation: 2,
        };
        // Normal: active A with v1 and B empty.
        let (choice, reasons) = choose(&header_a, [Ok(v1), Err(Reject::Format)], 0);
        assert_eq!(choice.map(|c| (c.slot, c.manifest.version)), Some((0, 1)));
        assert_eq!(reasons, [None, Some(Reject::Format)]);
        // After install: active B v2.
        let (choice, _) = choose(&header_b, [Ok(v1), Ok(v2)], 2);
        assert_eq!(choice.map(|c| c.slot), Some(1));
        // Downgrade attack: pointer back to A while the floor is 2.
        let (choice, reasons) = choose(&header_a, [Ok(v1), Ok(v2)], 2);
        assert_eq!(choice.map(|c| (c.slot, c.manifest.version)), Some((1, 2)));
        assert_eq!(reasons, [Some(Reject::Downgrade), None]);
        // Bad signature in the active slot: fall back to the other one.
        let (choice, reasons) = choose(&header_b, [Ok(v1), Err(Reject::Signature)], 1);
        assert_eq!(choice.map(|c| c.slot), Some(0));
        assert_eq!(reasons, [None, Some(Reject::Signature)]);
        // Nothing valid.
        let (choice, reasons) = choose(&header_b, [Err(Reject::Digest), Err(Reject::Signature)], 0);
        assert_eq!(choice, None);
        assert_eq!(reasons, [Some(Reject::Digest), Some(Reject::Signature)]);
        // Both below floor.
        let (choice, reasons) = choose(&header_a, [Ok(v1), Ok(v2)], 3);
        assert_eq!(choice, None);
        assert_eq!(reasons, [Some(Reject::Downgrade), Some(Reject::Downgrade)]);
    }

    /// `Header` fields are public; only `decode` enforces `active <= 1`. A
    /// hand-built header must fail closed instead of panicking (`1 - slot`) or,
    /// without overflow checks, yielding install target 255.
    #[test]
    fn out_of_range_active_slot_fails_closed() {
        let (_, v2, _) = signed(2);
        for active in [2, u8::MAX] {
            let header = Header {
                active,
                generation: 1,
            };
            assert_eq!(
                choose(&header, [Ok(v2), Ok(v2)], 0),
                (None, [Some(Reject::Format); 2])
            );
            assert_eq!(
                decide(&header, 1, 1, Ok(v2)),
                Decision::Rejected(Reject::Format)
            );
        }
    }

    #[test]
    fn install_decisions_never_target_the_active_slot_or_go_backwards() {
        let (_, v2, _) = signed(2);
        let header = Header {
            active: 0,
            generation: 1,
        };
        assert_eq!(
            decide(&header, 1, 1, Ok(v2)),
            Decision::Install {
                target: 1,
                version: 2
            }
        );
        assert_eq!(decide(&header, 2, 2, Ok(v2)), Decision::UpToDate);
        assert_eq!(
            decide(&header, 3, 3, Ok(v2)),
            Decision::Rejected(Reject::Downgrade)
        );
        assert_eq!(
            decide(&header, 1, 1, Err(Reject::Signature)),
            Decision::Rejected(Reject::Signature)
        );
        let header_b = Header {
            active: 1,
            generation: 5,
        };
        assert_eq!(
            decide(&header_b, 1, 0, Ok(v2)),
            Decision::Install {
                target: 0,
                version: 2
            }
        );
    }
}

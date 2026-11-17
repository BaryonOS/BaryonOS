//! Synthetic-key B03 policy and image-binding experiment. This module has no
//! production ingress or enrolment path. Callers supply an independently pinned
//! TEST root, retain the committed policy and floor, and validate native ELF
//! loadability separately before activation. No heap allocation is required.
#[cfg(feature = "test-policy-envelope-probe")]
pub mod probe;

use crate::policy::{Signer, MAX_SIGNERS};
use crate::{
    key_id, sha256, verify_hybrid, Manifest, Reject, TrustAnchors, ED25519_PUBLIC,
    ED25519_SIGNATURE, MANIFEST_LEN, MLDSA65_PUBLIC, MLDSA65_SIGNATURE,
};
use sha2::{Digest, Sha256};

pub const POLICY_MAGIC: [u8; 8] = *b"B03POL01";
pub const BINDING_MAGIC: [u8; 8] = *b"B03BND01";
pub const EXPERIMENTAL_TARGET: [u8; 16] = *b"B03-QEMU-TEST01\0";
pub const POLICY_FORMAT_VERSION: u16 = 1;
pub const POLICY_HEADER_LEN: usize = 112;
pub const SIGNER_ENTRY_LEN: usize = 2008;
pub const HYBRID_SIGNATURE_LEN: usize = ED25519_SIGNATURE + MLDSA65_SIGNATURE;
pub const MAX_POLICY_BODY_LEN: usize = POLICY_HEADER_LEN + MAX_SIGNERS * SIGNER_ENTRY_LEN;
pub const MAX_POLICY_ENVELOPE_LEN: usize = MAX_POLICY_BODY_LEN + HYBRID_SIGNATURE_LEN;
pub const BINDING_MESSAGE_LEN: usize = 88;
pub const BINDING_ENVELOPE_LEN: usize = BINDING_MESSAGE_LEN + HYBRID_SIGNATURE_LEN;
const PUBLIC_PAIR_LEN: usize = ED25519_PUBLIC + MLDSA65_PUBLIC;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyReject {
    Format,
    Target,
    Root,
    Role,
    SignerTable,
    Successor,
    Revocation,
    Binding,
    Core(Reject),
}

impl From<Reject> for PolicyReject {
    fn from(value: Reject) -> Self {
        Self::Core(value)
    }
}

/// Constructible only after canonical parsing and both root signatures succeed.
/// Borrowing keeps the authenticated body immutable for this object's lifetime.
#[derive(Clone, Copy, Debug)]
pub struct VerifiedPolicy<'a> {
    body: &'a [u8],
    digest: [u8; 32],
}

impl VerifiedPolicy<'_> {
    pub fn version(&self) -> u64 {
        u64_at(self.body, 32)
    }

    pub fn digest(&self) -> [u8; 32] {
        self.digest
    }

    pub fn predecessor_digest(&self) -> [u8; 32] {
        self.body[48..80].try_into().unwrap()
    }

    pub fn minimum_image_version(&self) -> u64 {
        u64_at(self.body, 40)
    }

    pub fn signer_count(&self) -> usize {
        u16::from_le_bytes(self.body[10..12].try_into().unwrap()) as usize
    }

    /// Return one authenticated table entry, including a possible tombstone.
    /// This lookup alone does not authenticate or authorize any image.
    pub fn signer(&self, index: usize) -> Option<Signer> {
        if index >= self.signer_count() {
            return None;
        }
        let entry = entry(self.body, index);
        Some(Signer {
            anchors: TrustAnchors {
                ed25519: entry[..ED25519_PUBLIC].try_into().unwrap(),
                mldsa65: entry[ED25519_PUBLIC..PUBLIC_PAIR_LEN].try_into().unwrap(),
            },
            first_version: u64_at(entry, PUBLIC_PAIR_LEN),
            last_version: u64_at(entry, PUBLIC_PAIR_LEN + 8),
            revoked: entry[PUBLIC_PAIR_LEN + 16] == 1,
        })
    }

    /// Select the complete pair; neither key identifier independently selects
    /// authority. Revocation and release-window checks are performed on use.
    pub fn signer_for(
        &self,
        ed25519_id: &[u8; 8],
        mldsa_id: &[u8; 8],
    ) -> Result<Signer, PolicyReject> {
        for index in 0..self.signer_count() {
            let bytes = entry(self.body, index);
            if key_id(&bytes[..ED25519_PUBLIC]) == *ed25519_id
                && key_id(&bytes[ED25519_PUBLIC..PUBLIC_PAIR_LEN]) == *mldsa_id
            {
                return Ok(self.signer(index).unwrap());
            }
        }
        Err(Reject::UnknownKey.into())
    }

    /// Authenticate the unchanged manifest and its binding under the same pair,
    /// then check the policy window, revocation, image floor and exact image.
    /// This establishes cryptographic/digest policy validity, NOT ELF loadability.
    /// Native callers must also run the existing ELF loader checks before any
    /// activation and enforce the transition's running-version/persistence rules.
    pub fn verify_bound_image(
        &self,
        binding: &[u8],
        manifest_bytes: &[u8; MANIFEST_LEN],
        image: &[u8],
        floor: u64,
    ) -> Result<Manifest, PolicyReject> {
        if binding.len() != BINDING_ENVELOPE_LEN || binding[..8] != BINDING_MAGIC {
            return Err(PolicyReject::Format);
        }
        if binding[8..24] != EXPERIMENTAL_TARGET {
            return Err(PolicyReject::Target);
        }
        if binding[24..56] != self.digest || binding[56..88] != sha256(manifest_bytes) {
            return Err(PolicyReject::Binding);
        }
        let decoded = Manifest::decode(manifest_bytes)?;
        let signer = self.signer_for(&decoded.ed25519_key_id, &decoded.mldsa_key_id)?;
        if signer.revoked {
            return Err(Reject::KeyRevoked.into());
        }
        let manifest = crate::verify_manifest(manifest_bytes, &signer.anchors)?;
        if manifest.version < signer.first_version {
            return Err(Reject::KeyNotYetValid.into());
        }
        if manifest.version > signer.last_version {
            return Err(Reject::KeyExpired.into());
        }
        if manifest.version < core::cmp::max(floor, self.minimum_image_version()) {
            return Err(Reject::Downgrade.into());
        }
        verify_hybrid(
            &binding[..BINDING_MESSAGE_LEN],
            &binding[BINDING_MESSAGE_LEN..],
            &signer.anchors,
        )?;
        crate::verify_image(&manifest, image)?;
        Ok(manifest)
    }
}

/// The caller must obtain this root from independently trusted TEST state,
/// never from the envelope being authenticated. A zero predecessor is meaningful
/// only for an explicitly pinned initial fixture; it grants no enrolment right.
pub fn verify_policy<'a>(
    envelope: &'a [u8],
    root: &TrustAnchors,
) -> Result<VerifiedPolicy<'a>, PolicyReject> {
    let body_len = validate_structure(envelope, root)?;
    let body = &envelope[..body_len];
    verify_hybrid(body, &envelope[body_len..], root)?;
    Ok(VerifiedPolicy {
        body,
        digest: sha256(body),
    })
}

/// Validate a new transition within one fixed root lineage. This does not
/// perform persistence or handle an idempotent retry of an already committed
/// operation; the native anchor protocol must do both using its complete identity.
pub fn validate_successor(
    current: &VerifiedPolicy<'_>,
    next: &VerifiedPolicy<'_>,
) -> Result<(), PolicyReject> {
    if current.body[80..112] != next.body[80..112] {
        return Err(PolicyReject::Root);
    }
    if next.version() <= current.version() || next.predecessor_digest() != current.digest() {
        return Err(PolicyReject::Successor);
    }
    for index in 0..current.signer_count() {
        let old = entry(current.body, index);
        if old[PUBLIC_PAIR_LEN + 16] != 1 {
            continue;
        }
        let retained = (0..next.signer_count()).any(|index| {
            let candidate = entry(next.body, index);
            candidate[..PUBLIC_PAIR_LEN] == old[..PUBLIC_PAIR_LEN]
                && candidate[PUBLIC_PAIR_LEN + 16] == 1
        });
        if !retained {
            return Err(PolicyReject::Revocation);
        }
    }
    Ok(())
}

/// Root identity commits to both complete public keys and the root-role domain.
pub fn root_identifier(root: &TrustAnchors) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"B03ROOT1");
    digest.update(root.ed25519);
    digest.update(root.mldsa65);
    digest.finalize().into()
}

fn envelope_len(count: usize) -> Result<usize, PolicyReject> {
    if !(1..=MAX_SIGNERS).contains(&count) {
        return Err(PolicyReject::Format);
    }
    count
        .checked_mul(SIGNER_ENTRY_LEN)
        .and_then(|entries| entries.checked_add(POLICY_HEADER_LEN))
        .and_then(|body| body.checked_add(HYBRID_SIGNATURE_LEN))
        .ok_or(PolicyReject::Format)
}

fn validate_structure(envelope: &[u8], root: &TrustAnchors) -> Result<usize, PolicyReject> {
    if envelope.len() < POLICY_HEADER_LEN + SIGNER_ENTRY_LEN + HYBRID_SIGNATURE_LEN
        || envelope.len() > MAX_POLICY_ENVELOPE_LEN
    {
        return Err(PolicyReject::Format);
    }
    if envelope[..8] != POLICY_MAGIC
        || u16::from_le_bytes(envelope[8..10].try_into().unwrap()) != POLICY_FORMAT_VERSION
    {
        return Err(PolicyReject::Format);
    }
    let count = u16::from_le_bytes(envelope[10..12].try_into().unwrap()) as usize;
    let expected_len = envelope_len(count)?;
    let body_len = expected_len - HYBRID_SIGNATURE_LEN;
    if envelope.len() != expected_len
        || u32::from_le_bytes(envelope[12..16].try_into().unwrap()) as usize != body_len
        || u64_at(envelope, 32) == 0
        || u64_at(envelope, 40) == 0
    {
        return Err(PolicyReject::Format);
    }
    if envelope[16..32] != EXPERIMENTAL_TARGET {
        return Err(PolicyReject::Target);
    }
    if envelope[80..112] != root_identifier(root) {
        return Err(PolicyReject::Root);
    }
    let body = &envelope[..body_len];
    for index in 0..count {
        let signer = entry(body, index);
        if u64_at(signer, PUBLIC_PAIR_LEN) == 0
            || u64_at(signer, PUBLIC_PAIR_LEN + 8) < u64_at(signer, PUBLIC_PAIR_LEN)
            || signer[PUBLIC_PAIR_LEN + 16] > 1
            || signer[PUBLIC_PAIR_LEN + 17..].iter().any(|byte| *byte != 0)
        {
            return Err(PolicyReject::SignerTable);
        }
        if signer[..ED25519_PUBLIC] == root.ed25519
            || signer[ED25519_PUBLIC..PUBLIC_PAIR_LEN] == root.mldsa65
        {
            return Err(PolicyReject::Role);
        }
        if index > 0 && entry(body, index - 1)[..PUBLIC_PAIR_LEN] >= signer[..PUBLIC_PAIR_LEN] {
            return Err(PolicyReject::SignerTable);
        }
        let ed_id = key_id(&signer[..ED25519_PUBLIC]);
        let pq_id = key_id(&signer[ED25519_PUBLIC..PUBLIC_PAIR_LEN]);
        for previous in 0..index {
            let earlier = entry(body, previous);
            if key_id(&earlier[..ED25519_PUBLIC]) == ed_id
                && key_id(&earlier[ED25519_PUBLIC..PUBLIC_PAIR_LEN]) == pq_id
            {
                return Err(PolicyReject::SignerTable);
            }
            if signer[PUBLIC_PAIR_LEN + 16] != earlier[PUBLIC_PAIR_LEN + 16]
                && (signer[..ED25519_PUBLIC] == earlier[..ED25519_PUBLIC]
                    || signer[ED25519_PUBLIC..PUBLIC_PAIR_LEN]
                        == earlier[ED25519_PUBLIC..PUBLIC_PAIR_LEN])
            {
                return Err(PolicyReject::Revocation);
            }
        }
    }
    Ok(body_len)
}

fn entry(body: &[u8], index: usize) -> &[u8] {
    let start = POLICY_HEADER_LEN + SIGNER_ENTRY_LEN * index;
    &body[start..start + SIGNER_ENTRY_LEN]
}

fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

/// Explicit host-only helpers for generating public synthetic TEST fixtures.
/// They are absent from the native loader/service targets and are not a signing
/// service, key-custody mechanism or production release tool.
#[cfg(all(
    feature = "test-signing-anchors",
    not(any(target_os = "none", target_os = "uefi"))
))]
pub mod fixtures {
    use super::*;
    use crate::SigningKeys;
    use ed25519_dalek::Signer as _;

    /// Encode an already sorted table into caller-owned storage, returning the
    /// exact envelope length. Unused output capacity is not part of the envelope.
    pub fn encode_policy(
        version: u64,
        minimum_image_version: u64,
        predecessor_digest: &[u8; 32],
        signers: &[Signer],
        root: &SigningKeys,
        out: &mut [u8],
    ) -> Result<usize, PolicyReject> {
        let length = envelope_len(signers.len())?;
        let bytes = out.get_mut(..length).ok_or(PolicyReject::Format)?;
        bytes.fill(0);
        let body_len = length - HYBRID_SIGNATURE_LEN;
        bytes[..8].copy_from_slice(&POLICY_MAGIC);
        bytes[8..10].copy_from_slice(&POLICY_FORMAT_VERSION.to_le_bytes());
        bytes[10..12].copy_from_slice(&(signers.len() as u16).to_le_bytes());
        bytes[12..16].copy_from_slice(&(body_len as u32).to_le_bytes());
        bytes[16..32].copy_from_slice(&EXPERIMENTAL_TARGET);
        bytes[32..40].copy_from_slice(&version.to_le_bytes());
        bytes[40..48].copy_from_slice(&minimum_image_version.to_le_bytes());
        bytes[48..80].copy_from_slice(predecessor_digest);
        let anchors = root.anchors();
        bytes[80..112].copy_from_slice(&root_identifier(&anchors));
        for (index, signer) in signers.iter().enumerate() {
            let offset = POLICY_HEADER_LEN + index * SIGNER_ENTRY_LEN;
            let encoded = &mut bytes[offset..offset + SIGNER_ENTRY_LEN];
            encoded[..ED25519_PUBLIC].copy_from_slice(&signer.anchors.ed25519);
            encoded[ED25519_PUBLIC..PUBLIC_PAIR_LEN].copy_from_slice(&signer.anchors.mldsa65);
            encoded[PUBLIC_PAIR_LEN..PUBLIC_PAIR_LEN + 8]
                .copy_from_slice(&signer.first_version.to_le_bytes());
            encoded[PUBLIC_PAIR_LEN + 8..PUBLIC_PAIR_LEN + 16]
                .copy_from_slice(&signer.last_version.to_le_bytes());
            encoded[PUBLIC_PAIR_LEN + 16] = u8::from(signer.revoked);
        }
        validate_structure(bytes, &anchors)?;
        let (body, signatures) = bytes.split_at_mut(body_len);
        sign_message(body, root, signatures)?;
        Ok(length)
    }

    /// Bind the exact signed manifest to a policy body digest. The helper checks
    /// that the manifest is authenticated by the supplied complete release pair.
    pub fn sign_binding(
        policy_digest: &[u8; 32],
        manifest_bytes: &[u8; MANIFEST_LEN],
        release: &SigningKeys,
    ) -> Result<[u8; BINDING_ENVELOPE_LEN], PolicyReject> {
        crate::verify_manifest(manifest_bytes, &release.anchors())?;
        let mut out = [0u8; BINDING_ENVELOPE_LEN];
        out[..8].copy_from_slice(&BINDING_MAGIC);
        out[8..24].copy_from_slice(&EXPERIMENTAL_TARGET);
        out[24..56].copy_from_slice(policy_digest);
        out[56..88].copy_from_slice(&sha256(manifest_bytes));
        let (message, signatures) = out.split_at_mut(BINDING_MESSAGE_LEN);
        sign_message(message, release, signatures)?;
        Ok(out)
    }

    fn sign_message(
        message: &[u8],
        keys: &SigningKeys,
        signatures: &mut [u8],
    ) -> Result<(), PolicyReject> {
        signatures[..ED25519_SIGNATURE].copy_from_slice(&keys.ed25519.sign(message).to_bytes());
        let pq = keys
            .mldsa65
            .expanded_key()
            .sign_deterministic(message, b"")
            .map_err(|_| Reject::Signature)?;
        signatures[ED25519_SIGNATURE..].copy_from_slice(&pq.encode());
        Ok(())
    }
}

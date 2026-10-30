//! Pinned signing authority, not a manifest-supplied trust store. Validity is
//! measured in release versions, never unauthenticated wall-clock time.
use crate::{Manifest, Reject, TrustAnchors, MANIFEST_LEN};

pub const MAX_SIGNERS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Signer {
    pub anchors: TrustAnchors,
    pub first_version: u64,
    pub last_version: u64,
    pub revoked: bool,
}

pub struct TrustPolicy<'a> {
    signers: &'a [Signer],
}

impl<'a> TrustPolicy<'a> {
    /// Validate the complete pinned table, including unrelated entries, before
    /// accepting any signer. Duplicate ID pairs are ambiguous and fail closed.
    pub fn new(signers: &'a [Signer]) -> Result<Self, Reject> {
        if signers.is_empty() || signers.len() > MAX_SIGNERS {
            return Err(Reject::KeyPolicy);
        }
        for (index, signer) in signers.iter().enumerate() {
            if signer.first_version == 0 || signer.last_version < signer.first_version {
                return Err(Reject::KeyPolicy);
            }
            for previous in &signers[..index] {
                if previous.anchors.ed25519_key_id() == signer.anchors.ed25519_key_id()
                    && previous.anchors.mldsa_key_id() == signer.anchors.mldsa_key_id()
                {
                    return Err(Reject::KeyPolicy);
                }
            }
        }
        Ok(Self { signers })
    }

    pub fn verify_manifest(&self, bytes: &[u8; MANIFEST_LEN]) -> Result<Manifest, Reject> {
        let manifest = Manifest::decode(bytes)?;
        let signer = self
            .signers
            .iter()
            .find(|signer| {
                manifest.ed25519_key_id == signer.anchors.ed25519_key_id()
                    && manifest.mldsa_key_id == signer.anchors.mldsa_key_id()
            })
            .ok_or(Reject::UnknownKey)?;
        if signer.revoked {
            return Err(Reject::KeyRevoked);
        }
        // Authenticate the release version before using it as a policy input.
        let manifest = crate::verify_manifest(bytes, &signer.anchors)?;
        if manifest.version < signer.first_version {
            return Err(Reject::KeyNotYetValid);
        }
        if manifest.version > signer.last_version {
            return Err(Reject::KeyExpired);
        }
        Ok(manifest)
    }
}

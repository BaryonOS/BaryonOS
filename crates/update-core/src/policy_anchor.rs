//! Canonical TEST-only anchor and transition identity for ADR-B03K3.
//!
//! This module defines bytes and fail-closed decisions, not a persistent backend.
//! Callers still need an authorised loader-only ingress, durable atomic compare-and-swap,
//! independent readback and native cut/replay evidence.

use crate::policy_envelope::EXPERIMENTAL_TARGET;
use crate::MAX_IMAGE_LEN;
use sha2::{Digest, Sha256};

pub const ANCHOR_MAGIC: [u8; 8] = *b"B03ANC01";
pub const ANCHOR_FORMAT_VERSION: u16 = 1;
pub const ANCHOR_LEN: usize = 120;
pub const TRANSITION_DOMAIN: [u8; 8] = *b"B03TXN01";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorReject {
    Format,
    Target,
    Revision,
    Policy,
    Floor,
    Operation,
    Predecessor,
    Conflict,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    A = 0,
    B = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitState {
    Predecessor,
    Successor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolicyAnchor {
    revision: u64,
    policy_version: u64,
    image_floor: u64,
    policy_digest: [u8; 32],
    last_transition: [u8; 32],
}

impl PolicyAnchor {
    /// Construct the one explicitly pinned synthetic bootstrap state.
    pub fn test_bootstrap(
        policy_version: u64,
        policy_digest: [u8; 32],
        image_floor: u64,
    ) -> Result<Self, AnchorReject> {
        if policy_version == 0 || image_floor == 0 || policy_digest == [0; 32] {
            return Err(AnchorReject::Format);
        }
        Ok(Self {
            revision: 1,
            policy_version,
            image_floor,
            policy_digest,
            last_transition: [0; 32],
        })
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn policy_version(&self) -> u64 {
        self.policy_version
    }

    pub fn image_floor(&self) -> u64 {
        self.image_floor
    }

    pub fn policy_digest(&self) -> [u8; 32] {
        self.policy_digest
    }

    pub fn last_transition(&self) -> [u8; 32] {
        self.last_transition
    }

    pub fn successor(
        &self,
        policy_version: u64,
        policy_digest: [u8; 32],
        image_floor: u64,
        transition: [u8; 32],
    ) -> Result<Self, AnchorReject> {
        let revision = self.revision.checked_add(1).ok_or(AnchorReject::Revision)?;
        if policy_version <= self.policy_version || policy_digest == [0; 32] {
            return Err(AnchorReject::Policy);
        }
        if image_floor < self.image_floor || image_floor == 0 {
            return Err(AnchorReject::Floor);
        }
        if transition == [0; 32] {
            return Err(AnchorReject::Operation);
        }
        Ok(Self {
            revision,
            policy_version,
            image_floor,
            policy_digest,
            last_transition: transition,
        })
    }

    pub fn encode(&self) -> [u8; ANCHOR_LEN] {
        let mut out = [0u8; ANCHOR_LEN];
        out[..8].copy_from_slice(&ANCHOR_MAGIC);
        out[8..10].copy_from_slice(&ANCHOR_FORMAT_VERSION.to_le_bytes());
        out[16..32].copy_from_slice(&EXPERIMENTAL_TARGET);
        out[32..40].copy_from_slice(&self.revision.to_le_bytes());
        out[40..48].copy_from_slice(&self.policy_version.to_le_bytes());
        out[48..56].copy_from_slice(&self.image_floor.to_le_bytes());
        out[56..88].copy_from_slice(&self.policy_digest);
        out[88..120].copy_from_slice(&self.last_transition);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, AnchorReject> {
        if bytes.len() != ANCHOR_LEN
            || bytes[..8] != ANCHOR_MAGIC
            || u16::from_le_bytes(bytes[8..10].try_into().unwrap()) != ANCHOR_FORMAT_VERSION
            || bytes[10..16].iter().any(|byte| *byte != 0)
        {
            return Err(AnchorReject::Format);
        }
        if bytes[16..32] != EXPERIMENTAL_TARGET {
            return Err(AnchorReject::Target);
        }
        let anchor = Self {
            revision: u64_at(bytes, 32),
            policy_version: u64_at(bytes, 40),
            image_floor: u64_at(bytes, 48),
            policy_digest: bytes[56..88].try_into().unwrap(),
            last_transition: bytes[88..120].try_into().unwrap(),
        };
        if anchor.revision == 0
            || anchor.policy_version == 0
            || anchor.image_floor == 0
            || anchor.policy_digest == [0; 32]
            || (anchor.revision == 1 && anchor.last_transition != [0; 32])
            || (anchor.revision > 1 && anchor.last_transition == [0; 32])
        {
            return Err(AnchorReject::Format);
        }
        Ok(anchor)
    }

    fn is_successor_of(&self, expected: &Self) -> bool {
        expected.revision.checked_add(1) == Some(self.revision)
            && self.policy_version > expected.policy_version
            && self.image_floor >= expected.image_floor
            && self.last_transition != [0; 32]
    }
}

/// Hash the complete operation tuple; a policy or slot number alone is insufficient.
#[allow(clippy::too_many_arguments)]
pub fn transition_identity(
    expected: &PolicyAnchor,
    candidate_policy_version: u64,
    candidate_policy_digest: [u8; 32],
    candidate_image_version: u64,
    manifest_digest: [u8; 32],
    image_digest: [u8; 32],
    image_length: u32,
    destination: Slot,
) -> Result<[u8; 32], AnchorReject> {
    if candidate_policy_version <= expected.policy_version
        || candidate_policy_digest == [0; 32]
        || candidate_image_version < expected.image_floor
        || manifest_digest == [0; 32]
        || image_digest == [0; 32]
        || image_length == 0
        || image_length > MAX_IMAGE_LEN
    {
        return Err(AnchorReject::Operation);
    }
    let mut digest = Sha256::new();
    digest.update(TRANSITION_DOMAIN);
    digest.update(expected.encode());
    digest.update(EXPERIMENTAL_TARGET);
    digest.update(candidate_policy_version.to_le_bytes());
    digest.update(candidate_policy_digest);
    digest.update(candidate_image_version.to_le_bytes());
    digest.update(manifest_digest);
    digest.update(image_digest);
    digest.update(image_length.to_le_bytes());
    digest.update([destination as u8]);
    Ok(digest.finalize().into())
}

/// Pure oracle for an atomic compare-and-swap backend. A mismatch never returns a write.
pub fn compare_and_swap(
    current: &PolicyAnchor,
    expected: &PolicyAnchor,
    successor: &PolicyAnchor,
) -> Result<PolicyAnchor, AnchorReject> {
    if !successor.is_successor_of(expected) {
        return Err(AnchorReject::Policy);
    }
    if current != expected {
        return Err(AnchorReject::Predecessor);
    }
    Ok(*successor)
}

/// Classify independent durable readback; every third valid value is a conflict.
pub fn classify_readback(
    expected: &PolicyAnchor,
    successor: &PolicyAnchor,
    bytes: &[u8],
) -> Result<CommitState, AnchorReject> {
    if !successor.is_successor_of(expected) {
        return Err(AnchorReject::Policy);
    }
    let observed = PolicyAnchor::decode(bytes)?;
    if observed == *expected {
        Ok(CommitState::Predecessor)
    } else if observed == *successor {
        Ok(CommitState::Successor)
    } else {
        Err(AnchorReject::Conflict)
    }
}

fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (PolicyAnchor, PolicyAnchor) {
        let current = PolicyAnchor::test_bootstrap(1, [1; 32], 2).unwrap();
        let operation =
            transition_identity(&current, 2, [2; 32], 3, [3; 32], [4; 32], 64, Slot::B).unwrap();
        let next = current.successor(2, [2; 32], 3, operation).unwrap();
        (current, next)
    }

    #[test]
    fn canonical_anchor_round_trips() {
        let (current, next) = fixture();
        assert_eq!(PolicyAnchor::decode(&current.encode()), Ok(current));
        assert_eq!(PolicyAnchor::decode(&next.encode()), Ok(next));
        assert_eq!(&current.encode()[..8], b"B03ANC01");
        assert_eq!(current.last_transition(), [0; 32]);
        assert_ne!(next.last_transition(), [0; 32]);
    }

    #[test]
    fn compare_and_readback_require_exact_complete_anchors() {
        let (current, next) = fixture();
        assert_eq!(compare_and_swap(&current, &current, &next), Ok(next));
        assert_eq!(
            compare_and_swap(&next, &current, &next),
            Err(AnchorReject::Predecessor)
        );
        assert_eq!(
            classify_readback(&current, &next, &current.encode()),
            Ok(CommitState::Predecessor)
        );
        assert_eq!(
            classify_readback(&current, &next, &next.encode()),
            Ok(CommitState::Successor)
        );
        let conflicting = current.successor(3, [9; 32], 3, [8; 32]).unwrap();
        assert_eq!(
            classify_readback(&current, &next, &conflicting.encode()),
            Err(AnchorReject::Conflict)
        );
    }

    #[test]
    fn malformed_uncertain_and_rollback_states_fail_closed() {
        let (current, next) = fixture();
        assert_eq!(
            PolicyAnchor::decode(&next.encode()[..ANCHOR_LEN - 1]),
            Err(AnchorReject::Format)
        );
        let mut torn = next.encode();
        torn[10] = 1;
        assert_eq!(PolicyAnchor::decode(&torn), Err(AnchorReject::Format));
        assert_eq!(
            current.successor(1, [2; 32], 3, [5; 32]),
            Err(AnchorReject::Policy)
        );
        assert_eq!(
            current.successor(2, [2; 32], 1, [5; 32]),
            Err(AnchorReject::Floor)
        );
        let mut maximum = next.encode();
        maximum[32..40].copy_from_slice(&u64::MAX.to_le_bytes());
        let maximum = PolicyAnchor::decode(&maximum).unwrap();
        assert_eq!(
            maximum.successor(2, [2; 32], 2, [5; 32]),
            Err(AnchorReject::Revision)
        );
    }

    #[test]
    fn transition_identity_binds_every_operation_field() {
        let (current, _) = fixture();
        let base =
            transition_identity(&current, 2, [2; 32], 3, [3; 32], [4; 32], 64, Slot::A).unwrap();
        let changed_slot =
            transition_identity(&current, 2, [2; 32], 3, [3; 32], [4; 32], 64, Slot::B).unwrap();
        let changed_image =
            transition_identity(&current, 2, [2; 32], 4, [3; 32], [4; 32], 64, Slot::A).unwrap();
        assert_ne!(base, changed_slot);
        assert_ne!(base, changed_image);
    }
}

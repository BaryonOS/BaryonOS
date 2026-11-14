//! Read-only native authentication probe for public synthetic TEST vectors.
//!
//! The loader and ring-3 service each execute this function. There is no I/O,
//! mutable authority, ingress capability, signing key, policy commit or ELF
//! activation here. The 64-byte payload is opaque test data, not a kernel.
use super::{
    validate_successor, verify_policy, PolicyReject, VerifiedPolicy, BINDING_ENVELOPE_LEN,
    BINDING_MESSAGE_LEN, POLICY_HEADER_LEN, SIGNER_ENTRY_LEN,
};
use crate::{Reject, TrustAnchors, ED25519_SIGNATURE, MANIFEST_LEN};

const POLICY: &[u8] = include_bytes!("probe-fixtures/policy.bin");
// Reuse authenticated window fixtures for lineage checks. This keeps the
// embedded probe inside the existing two-MiB kernel slot bound.
const SUCCESSOR: &[u8] = include_bytes!("probe-fixtures/policy-premature.bin");
const WRONG_LINEAGE: &[u8] = include_bytes!("probe-fixtures/policy-revoked.bin");
const MANIFEST: &[u8; MANIFEST_LEN] = include_bytes!("probe-fixtures/manifest.bin");
const PAYLOAD: &[u8; 64] = include_bytes!("probe-fixtures/payload.bin");
const BINDING: &[u8; BINDING_ENVELOPE_LEN] = include_bytes!("probe-fixtures/binding.bin");
const BODY_LEN: usize = POLICY_HEADER_LEN + SIGNER_ENTRY_LEN;
const POLICY_LEN: usize = BODY_LEN + crate::ED25519_SIGNATURE + crate::MLDSA65_SIGNATURE;

pub const CASE_COUNT: u64 = 26;
pub const EXPECTED_MASK: u64 = (1 << CASE_COUNT) - 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProbeReport {
    pub cases: u64,
    pub passed_mask: u64,
}

impl ProbeReport {
    fn record(&mut self, passed: bool) {
        if passed {
            self.passed_mask |= 1 << self.cases;
        }
        self.cases += 1;
    }

    pub fn passed(&self) -> bool {
        self.cases == CASE_COUNT && self.passed_mask == EXPECTED_MASK
    }
}

enum PolicyMutation {
    Flip(usize),
    RootEd,
    RootPq,
    Trailing,
}

#[inline(never)]
fn changed_policy(root: &TrustAnchors, change: PolicyMutation, expected: PolicyReject) -> bool {
    let mut bytes = [0u8; POLICY_LEN + 1];
    bytes[..POLICY_LEN].copy_from_slice(POLICY);
    let mut len = POLICY_LEN;
    match change {
        PolicyMutation::Flip(offset) => bytes[offset] ^= 1,
        PolicyMutation::RootEd => bytes[112..144].copy_from_slice(&root.ed25519),
        PolicyMutation::RootPq => bytes[144..2096].copy_from_slice(&root.mldsa65),
        PolicyMutation::Trailing => len += 1,
    }
    verify_policy(&bytes[..len], root).err() == Some(expected)
}

#[inline(never)]
fn changed_binding(policy: &VerifiedPolicy<'_>, offset: usize, expected: PolicyReject) -> bool {
    let mut bytes = *BINDING;
    bytes[offset] ^= 1;
    policy
        .verify_bound_image(&bytes, MANIFEST, PAYLOAD, 1)
        .err()
        == Some(expected)
}

#[inline(never)]
fn window_case(root: &TrustAnchors, policy: &[u8], binding: &[u8], expected: Reject) -> bool {
    match verify_policy(policy, root) {
        Ok(policy) => {
            policy
                .verify_bound_image(binding, MANIFEST, PAYLOAD, 1)
                .err()
                == Some(PolicyReject::Core(expected))
        }
        Err(_) => false,
    }
}

/// Each bit is set only after that case actually executes and returns its exact
/// expected result. An unexpected baseline failure returns an incomplete report.
#[inline(never)]
pub fn run() -> ProbeReport {
    let root = TrustAnchors {
        ed25519: *include_bytes!("probe-fixtures/root-ed25519.bin"),
        mldsa65: *include_bytes!("probe-fixtures/root-mldsa65.bin"),
    };
    let mut report = ProbeReport {
        cases: 0,
        passed_mask: 0,
    };
    let policy = verify_policy(POLICY, &root);
    report.record(policy.is_ok()); // 0: both root signatures
    let Ok(policy) = policy else { return report };
    report.record(
        policy
            .verify_bound_image(BINDING, MANIFEST, PAYLOAD, 1)
            .is_ok_and(|manifest| manifest.version == 2 && manifest.image_length == 64),
    ); // 1
    report.record(
        verify_policy(SUCCESSOR, &root)
            .is_ok_and(|next| validate_successor(&policy, &next).is_ok()),
    ); // 2
    let signature = PolicyReject::Core(Reject::Signature);
    report.record(changed_policy(
        &root,
        PolicyMutation::Flip(BODY_LEN),
        signature,
    )); // 3: Ed
    report.record(changed_policy(
        &root,
        PolicyMutation::Flip(BODY_LEN + ED25519_SIGNATURE),
        signature,
    )); // 4: PQ
    report.record(changed_policy(
        &root,
        PolicyMutation::Flip(16),
        PolicyReject::Target,
    )); // 5
    report.record(
        verify_policy(&POLICY[..POLICY.len() - 1], &root).err() == Some(PolicyReject::Format),
    ); // 6
    report.record(changed_policy(
        &root,
        PolicyMutation::Flip(12),
        PolicyReject::Format,
    )); // 7
    report.record(changed_policy(
        &root,
        PolicyMutation::Flip(80),
        PolicyReject::Root,
    )); // 8
    report.record(changed_policy(
        &root,
        PolicyMutation::RootEd,
        PolicyReject::Role,
    )); // 9
    report.record(changed_policy(
        &root,
        PolicyMutation::RootPq,
        PolicyReject::Role,
    )); // 10
    report.record(verify_policy(WRONG_LINEAGE, &root).is_ok_and(|next| {
        validate_successor(&policy, &next).err() == Some(PolicyReject::Successor)
    })); // 11
    report.record(changed_binding(&policy, BINDING_MESSAGE_LEN, signature)); // 12: Ed
    report.record(changed_binding(
        &policy,
        BINDING_MESSAGE_LEN + ED25519_SIGNATURE,
        signature,
    )); // 13: PQ
    report.record(changed_binding(&policy, 24, PolicyReject::Binding)); // 14: policy digest
    report.record(changed_binding(&policy, 56, PolicyReject::Binding)); // 15: manifest digest
    report.record(changed_binding(&policy, 8, PolicyReject::Target)); // 16
    report.record(
        policy
            .verify_bound_image(&BINDING[..BINDING.len() - 1], MANIFEST, PAYLOAD, 1)
            .err()
            == Some(PolicyReject::Format),
    ); // 17
    let mut changed_image = *PAYLOAD;
    changed_image[0] ^= 1;
    report.record(
        policy
            .verify_bound_image(BINDING, MANIFEST, &changed_image, 1)
            .err()
            == Some(PolicyReject::Core(Reject::Digest)),
    ); // 18
    report.record(
        policy
            .verify_bound_image(BINDING, MANIFEST, PAYLOAD, 3)
            .err()
            == Some(PolicyReject::Core(Reject::Downgrade)),
    ); // 19
    report.record(window_case(
        &root,
        include_bytes!("probe-fixtures/policy-revoked.bin"),
        include_bytes!("probe-fixtures/binding-revoked.bin"),
        Reject::KeyRevoked,
    )); // 20
    report.record(window_case(
        &root,
        include_bytes!("probe-fixtures/policy-premature.bin"),
        include_bytes!("probe-fixtures/binding-premature.bin"),
        Reject::KeyNotYetValid,
    )); // 21
    report.record(window_case(
        &root,
        include_bytes!("probe-fixtures/policy-expired.bin"),
        include_bytes!("probe-fixtures/binding-expired.bin"),
        Reject::KeyExpired,
    )); // 22
    report.record(changed_policy(
        &root,
        PolicyMutation::Trailing,
        PolicyReject::Format,
    )); // 23
    report.record(verify_policy(MANIFEST, &root).err() == Some(PolicyReject::Format)); // 24: format domains
    report.record(
        policy
            .verify_bound_image(POLICY, MANIFEST, PAYLOAD, 1)
            .err()
            == Some(PolicyReject::Format),
    ); // 25
    report
}

#[cfg(test)]
mod tests {
    #[test]
    fn immutable_native_vector_cases_match_the_exact_report() {
        let report = super::run();
        assert_eq!(report.cases, 26);
        assert_eq!(report.passed_mask, 67_108_863);
        assert!(report.passed());
    }
}

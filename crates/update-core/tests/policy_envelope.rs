//! Public synthetic seeds and byte fixtures; none are production authority.
use ed25519_dalek::Signer as _;
use std::sync::OnceLock;
use update_core::policy::{Signer, MAX_SIGNERS};
use update_core::policy_envelope::{fixtures::*, *};
use update_core::{
    manifest_for, sha256, Reject, SigningKeys, TrustAnchors, ED25519_SIGNATURE, MANIFEST_LEN,
    MLDSA65_SIGNATURE, SIGNED_LEN,
};

const PAIR_LEN: usize = 1984;

struct Fixture {
    root: SigningKeys,
    releases: Vec<SigningKeys>,
    policy: Vec<u8>,
    image: Vec<u8>,
    manifest: [u8; MANIFEST_LEN],
    binding: [u8; BINDING_ENVELOPE_LEN],
}

fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let root = SigningKeys::from_seeds(&[0x71; 32], &[0x72; 32]);
        let releases: Vec<_> = (0..8)
            .map(|index| SigningKeys::from_seeds(&[0x11 + index; 32], &[0x31 + index; 32]))
            .collect();
        let policy = encode(&root, vec![signer(&releases[0])], 1, 1, &[0; 32]);
        // Crypto/digest unit evidence deliberately does not assert ELF loadability.
        let image = b"public synthetic TEST image; ELF is a separate native check".to_vec();
        let manifest = releases[0]
            .sign(&manifest_for(&image, 5, 1, &releases[0].anchors()).unwrap())
            .unwrap();
        let verified = verify_policy(&policy, &root.anchors()).unwrap();
        let binding = sign_binding(&verified.digest(), &manifest, &releases[0]).unwrap();
        Fixture {
            root,
            releases,
            policy,
            image,
            manifest,
            binding,
        }
    })
}

fn signer(keys: &SigningKeys) -> Signer {
    Signer {
        anchors: keys.anchors(),
        first_version: 1,
        last_version: 10,
        revoked: false,
    }
}

fn sort(signers: &mut [Signer]) {
    signers.sort_unstable_by(|a, b| {
        a.anchors
            .ed25519
            .cmp(&b.anchors.ed25519)
            .then(a.anchors.mldsa65.cmp(&b.anchors.mldsa65))
    });
}

fn encode(
    root: &SigningKeys,
    mut signers: Vec<Signer>,
    version: u64,
    minimum: u64,
    parent: &[u8; 32],
) -> Vec<u8> {
    sort(&mut signers);
    let mut bytes = vec![0; MAX_POLICY_ENVELOPE_LEN];
    let length = encode_policy(version, minimum, parent, &signers, root, &mut bytes).unwrap();
    bytes.truncate(length);
    bytes
}

fn sign_message(message: &[u8], keys: &SigningKeys) -> [u8; HYBRID_SIGNATURE_LEN] {
    let mut signatures = [0; HYBRID_SIGNATURE_LEN];
    signatures[..ED25519_SIGNATURE].copy_from_slice(&keys.ed25519.sign(message).to_bytes());
    signatures[ED25519_SIGNATURE..].copy_from_slice(
        &keys
            .mldsa65
            .expanded_key()
            .sign_deterministic(message, b"")
            .unwrap()
            .encode(),
    );
    signatures
}

fn resign_policy(bytes: &mut [u8], keys: &SigningKeys) {
    let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let signatures = sign_message(&bytes[..length], keys);
    bytes[length..].copy_from_slice(&signatures);
}

fn resign_binding(bytes: &mut [u8; BINDING_ENVELOPE_LEN], keys: &SigningKeys) {
    let signatures = sign_message(&bytes[..BINDING_MESSAGE_LEN], keys);
    bytes[BINDING_MESSAGE_LEN..].copy_from_slice(&signatures);
}

fn assert_policy_rejected(bytes: &[u8], root: &TrustAnchors, reason: PolicyReject) {
    assert_eq!(verify_policy(bytes, root).unwrap_err(), reason);
}

#[test]
fn canonical_fixture_round_trips_and_identity_is_only_the_body() {
    let f = fixture();
    let policy = verify_policy(&f.policy, &f.root.anchors()).unwrap();
    assert_eq!(&f.policy[..8], b"B03POL01");
    assert_eq!(&f.policy[16..32], b"B03-QEMU-TEST01\0");
    assert_eq!(f.policy.len(), 112 + 2008 + 64 + 3309);
    assert_eq!(policy.version(), 1);
    assert_eq!(policy.minimum_image_version(), 1);
    assert_eq!(policy.predecessor_digest(), [0; 32]);
    assert_eq!(policy.digest(), sha256(&f.policy[..112 + 2008]));
    assert_ne!(policy.digest(), sha256(&f.policy));
    assert_eq!(policy.signer_count(), 1);
    assert_eq!(policy.signer(0), Some(signer(&f.releases[0])));
    assert_eq!(policy.signer(1), None);
    assert_eq!(policy.signer(usize::MAX), None);
    let anchors = f.releases[0].anchors();
    assert_eq!(
        policy.signer_for(&anchors.ed25519_key_id(), &anchors.mldsa_key_id()),
        Ok(signer(&f.releases[0]))
    );
    let manifest = policy
        .verify_bound_image(&f.binding, &f.manifest, &f.image, 5)
        .unwrap();
    assert_eq!(manifest.version, 5);
    assert_eq!(f.binding.len(), 88 + 64 + 3309);
    assert_eq!(&f.binding[56..88], &sha256(&f.manifest));
    let mut independently_encoded_root = Vec::from(&b"B03ROOT1"[..]);
    independently_encoded_root.extend_from_slice(&f.root.anchors().ed25519);
    independently_encoded_root.extend_from_slice(&f.root.anchors().mldsa65);
    assert_eq!(
        root_identifier(&f.root.anchors()),
        sha256(&independently_encoded_root)
    );
}

#[test]
fn every_wrong_envelope_length_and_declared_size_fails_before_crypto() {
    let f = fixture();
    let root = f.root.anchors();
    verify_policy(&f.policy, &root).unwrap();
    let mut bytes = vec![0; MAX_POLICY_ENVELOPE_LEN + 1];
    bytes[..f.policy.len()].copy_from_slice(&f.policy);
    for length in 0..=bytes.len() {
        if length != f.policy.len() {
            assert_policy_rejected(&bytes[..length], &root, PolicyReject::Format);
        }
    }
    for declared in [
        0,
        111,
        112,
        2119,
        2121,
        MAX_POLICY_BODY_LEN as u32,
        u32::MAX,
    ] {
        let mut altered = f.policy.clone();
        altered[12..16].copy_from_slice(&declared.to_le_bytes());
        assert_policy_rejected(&altered, &root, PolicyReject::Format);
    }
    for count in [0u16, 9, u16::MAX] {
        let mut altered = f.policy.clone();
        altered[10..12].copy_from_slice(&count.to_le_bytes());
        assert_policy_rejected(&altered, &root, PolicyReject::Format);
    }
}

#[test]
fn all_header_prefix_fields_are_checked_and_zero_versions_are_refused() {
    let f = fixture();
    let root = f.root.anchors();
    verify_policy(&f.policy, &root).unwrap();
    for offset in 0..10 {
        let mut bytes = f.policy.clone();
        bytes[offset] ^= 0x80;
        assert_policy_rejected(&bytes, &root, PolicyReject::Format);
    }
    for offset in 16..32 {
        let mut bytes = f.policy.clone();
        bytes[offset] ^= 1;
        assert_policy_rejected(&bytes, &root, PolicyReject::Target);
    }
    for offset in 80..112 {
        let mut bytes = f.policy.clone();
        bytes[offset] ^= 1;
        assert_policy_rejected(&bytes, &root, PolicyReject::Root);
    }
    for offset in [32, 40] {
        let mut bytes = f.policy.clone();
        bytes[offset..offset + 8].fill(0);
        assert_policy_rejected(&bytes, &root, PolicyReject::Format);
    }
}

#[test]
fn both_root_signatures_fixed_root_and_role_separation_are_required() {
    let f = fixture();
    let root = f.root.anchors();
    verify_policy(&f.policy, &root).unwrap();
    let body_len = 112 + 2008;
    for offset in [body_len, body_len + ED25519_SIGNATURE] {
        let mut bytes = f.policy.clone();
        bytes[offset] ^= 1;
        assert_policy_rejected(&bytes, &root, PolicyReject::Core(Reject::Signature));
    }
    let mut changed_body = f.policy.clone();
    changed_body[32] = 3; // Still structurally valid, but not root-authenticated.
    assert_policy_rejected(&changed_body, &root, PolicyReject::Core(Reject::Signature));
    assert_policy_rejected(&f.policy, &f.releases[0].anchors(), PolicyReject::Root);
    let mut release_signed_policy = f.policy.clone();
    resign_policy(&mut release_signed_policy, &f.releases[0]);
    assert_policy_rejected(
        &release_signed_policy,
        &root,
        PolicyReject::Core(Reject::Signature),
    );
    let mixed = SigningKeys::from_seeds(&[0x71; 32], &[0x74; 32]);
    let mut mixed_signed_policy = f.policy.clone();
    resign_policy(&mut mixed_signed_policy, &mixed);
    assert_policy_rejected(
        &mixed_signed_policy,
        &root,
        PolicyReject::Core(Reject::Signature),
    );
    let mixed_control = encode(&mixed, vec![signer(&f.releases[0])], 1, 1, &[0; 32]);
    verify_policy(&mixed_control, &mixed.anchors()).unwrap();
    for (start, component) in [(112, &root.ed25519[..]), (112 + 32, &root.mldsa65[..])] {
        let mut bytes = f.policy.clone();
        bytes[start..start + component.len()].copy_from_slice(component);
        resign_policy(&mut bytes, &f.root);
        assert_policy_rejected(&bytes, &root, PolicyReject::Role);
    }
}

#[test]
fn windows_flags_reserved_bytes_sorting_and_duplicate_pairs_fail_closed() {
    let f = fixture();
    let root = f.root.anchors();
    verify_policy(&f.policy, &root).unwrap();
    for (offset, value) in [(112 + PAIR_LEN, 0u64), (112 + PAIR_LEN + 8, 0u64)] {
        let mut bytes = f.policy.clone();
        bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        resign_policy(&mut bytes, &f.root);
        assert_policy_rejected(&bytes, &root, PolicyReject::SignerTable);
    }
    for offset in 112 + PAIR_LEN + 16..112 + SIGNER_ENTRY_LEN {
        let mut bytes = f.policy.clone();
        bytes[offset] = if offset == 112 + PAIR_LEN + 16 { 2 } else { 1 };
        resign_policy(&mut bytes, &f.root);
        assert_policy_rejected(&bytes, &root, PolicyReject::SignerTable);
    }
    let valid = encode(
        &f.root,
        vec![signer(&f.releases[0]), signer(&f.releases[1])],
        1,
        1,
        &[0; 32],
    );
    verify_policy(&valid, &root).unwrap();
    let mut duplicate = valid.clone();
    duplicate.copy_within(112..112 + SIGNER_ENTRY_LEN, 112 + SIGNER_ENTRY_LEN);
    resign_policy(&mut duplicate, &f.root);
    assert_policy_rejected(&duplicate, &root, PolicyReject::SignerTable);
    let mut reversed = valid;
    for index in 0..SIGNER_ENTRY_LEN {
        reversed.swap(112 + index, 112 + SIGNER_ENTRY_LEN + index);
    }
    resign_policy(&mut reversed, &f.root);
    assert_policy_rejected(&reversed, &root, PolicyReject::SignerTable);
}

#[test]
fn full_public_keys_determine_order_and_revocation_blocks_either_shared_component() {
    let f = fixture();
    let root = f.root.anchors();
    for shared_ed in [true, false] {
        let first = signer(&f.releases[0]);
        let mut second = signer(&f.releases[1]);
        if shared_ed {
            second.anchors.ed25519 = first.anchors.ed25519;
        } else {
            second.anchors.mldsa65 = first.anchors.mldsa65;
        }
        let valid = encode(&f.root, vec![first, second], 1, 1, &[0; 32]);
        verify_policy(&valid, &root).unwrap();
        for tombstone in [0, 1] {
            let mut bytes = valid.clone();
            bytes[112 + tombstone * SIGNER_ENTRY_LEN + PAIR_LEN + 16] = 1;
            resign_policy(&mut bytes, &f.root);
            assert_policy_rejected(&bytes, &root, PolicyReject::Revocation);
        }
        if shared_ed {
            let mut reversed = valid;
            for index in 0..SIGNER_ENTRY_LEN {
                reversed.swap(112 + index, 112 + SIGNER_ENTRY_LEN + index);
            }
            resign_policy(&mut reversed, &f.root);
            assert_policy_rejected(&reversed, &root, PolicyReject::SignerTable);
        }
    }
}

#[test]
fn bounded_maximum_table_accepts_eight_and_rejects_exhaustion() {
    let f = fixture();
    let mut signers: Vec<_> = f.releases.iter().map(signer).collect();
    assert_eq!(signers.len(), MAX_SIGNERS);
    let bytes = encode(&f.root, signers.clone(), 1, 1, &[0; 32]);
    assert_eq!(bytes.len(), 19549);
    let policy = verify_policy(&bytes, &f.root.anchors()).unwrap();
    assert_eq!(policy.signer_count(), 8);
    assert!(policy.signer(7).is_some());
    assert!(policy.signer(8).is_none());
    signers.push(signers[0]);
    let mut out = [0; MAX_POLICY_ENVELOPE_LEN];
    assert_eq!(
        encode_policy(2, 1, &policy.digest(), &signers, &f.root, &mut out),
        Err(PolicyReject::Format)
    );
    assert_eq!(
        encode_policy(1, 1, &[0; 32], &[], &f.root, &mut out),
        Err(PolicyReject::Format)
    );
    assert_eq!(
        encode_policy(1, 1, &[0; 32], &signers[..1], &f.root, &mut out[..100]),
        Err(PolicyReject::Format)
    );
}

#[test]
fn successors_require_exact_parent_strict_version_fixed_root_and_permanent_tombstones() {
    let f = fixture();
    let root = f.root.anchors();
    let active = signer(&f.releases[0]);
    let mut revoked = signer(&f.releases[1]);
    revoked.revoked = true;
    let current_bytes = encode(&f.root, vec![active, revoked], 3, 1, &[0; 32]);
    let current = verify_policy(&current_bytes, &root).unwrap();
    let good = encode(
        &f.root,
        vec![signer(&f.releases[2]), revoked],
        7,
        2,
        &current.digest(),
    );
    validate_successor(&current, &verify_policy(&good, &root).unwrap()).unwrap();
    for version in [1, 3] {
        let bytes = encode(
            &f.root,
            vec![active, revoked],
            version,
            2,
            &current.digest(),
        );
        assert_eq!(
            validate_successor(&current, &verify_policy(&bytes, &root).unwrap()),
            Err(PolicyReject::Successor)
        );
    }
    for parent in [[0; 32], [0x44; 32]] {
        let bytes = encode(&f.root, vec![active, revoked], 4, 1, &parent);
        assert_eq!(
            validate_successor(&current, &verify_policy(&bytes, &root).unwrap()),
            Err(PolicyReject::Successor)
        );
    }
    for table in [vec![active], vec![active, signer(&f.releases[1])]] {
        let bytes = encode(&f.root, table, 4, 1, &current.digest());
        assert_eq!(
            validate_successor(&current, &verify_policy(&bytes, &root).unwrap()),
            Err(PolicyReject::Revocation)
        );
    }
    let other_root = SigningKeys::from_seeds(&[0x73; 32], &[0x74; 32]);
    let other = encode(&other_root, vec![active, revoked], 4, 1, &current.digest());
    let other_verified = verify_policy(&other, &other_root.anchors()).unwrap();
    assert_eq!(
        validate_successor(&current, &other_verified),
        Err(PolicyReject::Root)
    );
    let maximum_bytes = encode(&f.root, vec![active], u64::MAX, 1, &current.digest());
    let maximum = verify_policy(&maximum_bytes, &root).unwrap();
    let wrapped_bytes = encode(&f.root, vec![active], 1, 1, &maximum.digest());
    assert_eq!(
        validate_successor(&maximum, &verify_policy(&wrapped_bytes, &root).unwrap()),
        Err(PolicyReject::Successor)
    );
}

#[test]
fn binding_lengths_prefix_target_policy_and_exact_manifest_are_enforced() {
    let f = fixture();
    let policy = verify_policy(&f.policy, &f.root.anchors()).unwrap();
    policy
        .verify_bound_image(&f.binding, &f.manifest, &f.image, 1)
        .unwrap();
    let mut padded = f.binding.to_vec();
    padded.push(0);
    for length in 0..=padded.len() {
        if length != BINDING_ENVELOPE_LEN {
            assert_eq!(
                policy.verify_bound_image(&padded[..length], &f.manifest, &f.image, 1),
                Err(PolicyReject::Format)
            );
        }
    }
    for (range, reason) in [
        (0..8, PolicyReject::Format),
        (8..24, PolicyReject::Target),
        (24..88, PolicyReject::Binding),
    ] {
        for offset in range {
            let mut bytes = f.binding;
            bytes[offset] ^= 1;
            assert_eq!(
                policy.verify_bound_image(&bytes, &f.manifest, &f.image, 1),
                Err(reason)
            );
        }
    }
    let next_bytes = encode(
        &f.root,
        vec![signer(&f.releases[0])],
        2,
        1,
        &policy.digest(),
    );
    let next = verify_policy(&next_bytes, &f.root.anchors()).unwrap();
    validate_successor(&policy, &next).unwrap();
    assert_eq!(
        next.verify_bound_image(&f.binding, &f.manifest, &f.image, 1),
        Err(PolicyReject::Binding)
    );
    let rebound = sign_binding(&next.digest(), &f.manifest, &f.releases[0]).unwrap();
    next.verify_bound_image(&rebound, &f.manifest, &f.image, 1)
        .unwrap();
    let different_manifest = f.releases[0]
        .sign(&manifest_for(&f.image, 6, 1, &f.releases[0].anchors()).unwrap())
        .unwrap();
    assert_eq!(
        policy.verify_bound_image(&f.binding, &different_manifest, &f.image, 1),
        Err(PolicyReject::Binding)
    );
    let different_binding =
        sign_binding(&policy.digest(), &different_manifest, &f.releases[0]).unwrap();
    policy
        .verify_bound_image(&different_binding, &different_manifest, &f.image, 1)
        .unwrap();
}

#[test]
fn both_binding_signatures_and_the_complete_manifest_selected_pair_are_required() {
    let f = fixture();
    let bytes = encode(
        &f.root,
        vec![signer(&f.releases[0]), signer(&f.releases[1])],
        1,
        1,
        &[0; 32],
    );
    let policy = verify_policy(&bytes, &f.root.anchors()).unwrap();
    let binding = sign_binding(&policy.digest(), &f.manifest, &f.releases[0]).unwrap();
    policy
        .verify_bound_image(&binding, &f.manifest, &f.image, 1)
        .unwrap();
    for offset in [BINDING_MESSAGE_LEN, BINDING_MESSAGE_LEN + ED25519_SIGNATURE] {
        let mut bad = binding;
        bad[offset] ^= 1;
        assert_eq!(
            policy.verify_bound_image(&bad, &f.manifest, &f.image, 1),
            Err(PolicyReject::Core(Reject::Signature))
        );
    }
    let mixed = SigningKeys::from_seeds(&[0x11; 32], &[0x32; 32]);
    for wrong_pair in [&f.releases[1], &mixed, &f.root] {
        let mut bad = binding;
        resign_binding(&mut bad, wrong_pair);
        assert_eq!(
            policy.verify_bound_image(&bad, &f.manifest, &f.image, 1),
            Err(PolicyReject::Core(Reject::Signature))
        );
    }
    // The second pair really is valid under this same policy; it cannot sign
    // the first pair's manifest binding merely because both pairs are listed.
    let other_manifest = f.releases[1]
        .sign(&manifest_for(&f.image, 5, 1, &f.releases[1].anchors()).unwrap())
        .unwrap();
    let other_binding = sign_binding(&policy.digest(), &other_manifest, &f.releases[1]).unwrap();
    policy
        .verify_bound_image(&other_binding, &other_manifest, &f.image, 1)
        .unwrap();
    assert_eq!(
        sign_binding(&policy.digest(), &f.manifest, &f.releases[1]),
        Err(PolicyReject::Core(Reject::UnknownKey))
    );
    let mixed_manifest = mixed
        .sign(&manifest_for(&f.image, 5, 1, &mixed.anchors()).unwrap())
        .unwrap();
    let mixed_binding = sign_binding(&policy.digest(), &mixed_manifest, &mixed).unwrap();
    assert_eq!(
        policy.verify_bound_image(&mixed_binding, &mixed_manifest, &f.image, 1),
        Err(PolicyReject::Core(Reject::UnknownKey))
    );
}

#[test]
fn manifest_authentication_and_exact_image_are_required_even_with_a_valid_binding() {
    let f = fixture();
    let policy = verify_policy(&f.policy, &f.root.anchors()).unwrap();
    policy
        .verify_bound_image(&f.binding, &f.manifest, &f.image, 1)
        .unwrap();
    for offset in [SIGNED_LEN, SIGNED_LEN + ED25519_SIGNATURE, 20] {
        let mut manifest = f.manifest;
        manifest[offset] ^= 1;
        let mut binding = f.binding;
        binding[56..88].copy_from_slice(&sha256(&manifest));
        resign_binding(&mut binding, &f.releases[0]);
        assert_eq!(
            policy.verify_bound_image(&binding, &manifest, &f.image, 1),
            Err(PolicyReject::Core(Reject::Signature))
        );
    }
    let mut image = f.image.clone();
    image[0] ^= 1;
    assert_eq!(
        policy.verify_bound_image(&f.binding, &f.manifest, &image, 1),
        Err(PolicyReject::Core(Reject::Digest))
    );
    assert_eq!(
        policy.verify_bound_image(&f.binding, &f.manifest, &f.image[..f.image.len() - 1], 1),
        Err(PolicyReject::Core(Reject::Digest))
    );
    let mut long_image = f.image.clone();
    long_image.push(0);
    assert_eq!(
        policy.verify_bound_image(&f.binding, &f.manifest, &long_image, 1),
        Err(PolicyReject::Core(Reject::Digest))
    );
}

#[test]
fn authenticated_release_windows_revocation_and_both_floors_are_enforced() {
    let f = fixture();
    let root = f.root.anchors();
    for (first, last, revoked, minimum, floor, expected) in [
        (5, 5, false, 5, 5, Ok(())),
        (6, 10, false, 1, 1, Err(Reject::KeyNotYetValid)),
        (1, 4, false, 1, 1, Err(Reject::KeyExpired)),
        (1, 10, true, 1, 1, Err(Reject::KeyRevoked)),
        (1, 10, false, 6, 1, Err(Reject::Downgrade)),
        (1, 10, false, 1, 6, Err(Reject::Downgrade)),
    ] {
        let mut authority = signer(&f.releases[0]);
        authority.first_version = first;
        authority.last_version = last;
        authority.revoked = revoked;
        let bytes = encode(&f.root, vec![authority], 1, minimum, &[0; 32]);
        let policy = verify_policy(&bytes, &root).unwrap();
        let binding = sign_binding(&policy.digest(), &f.manifest, &f.releases[0]).unwrap();
        assert_eq!(
            policy
                .verify_bound_image(&binding, &f.manifest, &f.image, floor)
                .map(|_| ()),
            expected.map_err(PolicyReject::Core)
        );
    }
}

#[test]
fn legacy_manifest_and_other_domains_never_authorize_the_new_format() {
    let f = fixture();
    let root = f.root.anchors();
    let policy = verify_policy(&f.policy, &root).unwrap();
    policy
        .verify_bound_image(&f.binding, &f.manifest, &f.image, 1)
        .unwrap();
    assert_policy_rejected(&f.manifest, &root, PolicyReject::Format);
    assert_policy_rejected(&f.binding, &root, PolicyReject::Format);
    assert_eq!(
        policy.verify_bound_image(&f.policy, &f.manifest, &f.image, 1),
        Err(PolicyReject::Format)
    );
    let mut binding = f.binding;
    // This uses the correct release pair, isolating message-domain separation
    // from the independent root/release role check below.
    binding[BINDING_MESSAGE_LEN..]
        .copy_from_slice(&f.manifest[SIGNED_LEN..SIGNED_LEN + HYBRID_SIGNATURE_LEN]);
    assert_eq!(
        policy.verify_bound_image(&binding, &f.manifest, &f.image, 1),
        Err(PolicyReject::Core(Reject::Signature))
    );
    let policy_body = POLICY_HEADER_LEN + SIGNER_ENTRY_LEN;
    binding[BINDING_MESSAGE_LEN..].copy_from_slice(&f.policy[policy_body..]);
    assert_eq!(
        policy.verify_bound_image(&binding, &f.manifest, &f.image, 1),
        Err(PolicyReject::Core(Reject::Signature))
    );
    assert_eq!(HYBRID_SIGNATURE_LEN, ED25519_SIGNATURE + MLDSA65_SIGNATURE);
}

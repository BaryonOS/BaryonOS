use update_core::{
    manifest_for,
    policy::{Signer, TrustPolicy, MAX_SIGNERS},
    test_policy, verify_manifest, Manifest, Reject, SigningKeys, MANIFEST_LEN,
};

fn keys(index: usize) -> SigningKeys {
    let seeds: [(&[u8; 32], &[u8; 32]); 4] = [
        (
            include_bytes!("../../../tools/update-keys/TEST-ed25519.seed"),
            include_bytes!("../../../tools/update-keys/TEST-mldsa65.seed"),
        ),
        (
            include_bytes!("../../../tools/update-keys/TEST-next-ed25519.seed"),
            include_bytes!("../../../tools/update-keys/TEST-next-mldsa65.seed"),
        ),
        (
            include_bytes!("../../../tools/update-keys/TEST-revoked-ed25519.seed"),
            include_bytes!("../../../tools/update-keys/TEST-revoked-mldsa65.seed"),
        ),
        (
            include_bytes!("../../../tools/update-keys/TEST-expired-ed25519.seed"),
            include_bytes!("../../../tools/update-keys/TEST-expired-mldsa65.seed"),
        ),
    ];
    SigningKeys::from_seeds(seeds[index].0, seeds[index].1)
}

fn signed(index: usize, version: u64) -> ([u8; MANIFEST_LEN], Manifest) {
    let keys = keys(index);
    let manifest = manifest_for(b"synthetic test image", version, 0, &keys.anchors()).unwrap();
    (keys.sign(&manifest).unwrap(), manifest)
}

#[test]
fn rotation_accepts_current_and_successor_with_both_signatures() {
    for (index, version) in [(0, 1), (0, 2), (1, 2), (1, u64::MAX)] {
        let (bytes, manifest) = signed(index, version);
        assert_eq!(test_policy::verify_manifest(&bytes), Ok(manifest));
        for offset in [84, 148] {
            let mut corrupted = bytes;
            corrupted[offset] ^= 1;
            assert_eq!(
                test_policy::verify_manifest(&corrupted),
                Err(Reject::Signature)
            );
        }
    }
}

#[test]
fn revocation_and_release_version_validity_are_distinct() {
    for version in [1, 2, u64::MAX] {
        let (bytes, _) = signed(2, version);
        // Cryptographic validity is deliberately insufficient authority.
        assert!(verify_manifest(&bytes, &keys(2).anchors()).is_ok());
        assert_eq!(
            test_policy::verify_manifest(&bytes),
            Err(Reject::KeyRevoked)
        );
    }
    let (early, _) = signed(1, 1);
    assert_eq!(
        test_policy::verify_manifest(&early),
        Err(Reject::KeyNotYetValid)
    );
    let (last, manifest) = signed(3, 1);
    assert_eq!(test_policy::verify_manifest(&last), Ok(manifest));
    for version in [2, u64::MAX] {
        let (expired, _) = signed(3, version);
        assert_eq!(
            test_policy::verify_manifest(&expired),
            Err(Reject::KeyExpired)
        );
    }
}

#[test]
fn version_tampering_cannot_move_a_signature_into_the_valid_window() {
    let (mut bytes, _) = signed(1, 1);
    bytes[20..28].copy_from_slice(&2u64.to_le_bytes());
    assert_eq!(test_policy::verify_manifest(&bytes), Err(Reject::Signature));
    let (mut bytes, _) = signed(3, 2);
    bytes[20..28].copy_from_slice(&1u64.to_le_bytes());
    assert_eq!(test_policy::verify_manifest(&bytes), Err(Reject::Signature));
}

#[test]
fn key_pair_cannot_be_mixed_or_selected_from_untrusted_data() {
    let (mut bytes, _) = signed(0, 2);
    bytes[76..84].copy_from_slice(&test_policy::SIGNERS[1].anchors.mldsa_key_id());
    assert_eq!(
        test_policy::verify_manifest(&bytes),
        Err(Reject::UnknownKey)
    );
    let foreign = SigningKeys::from_seeds(&[51; 32], &[52; 32]);
    let manifest = manifest_for(b"synthetic", 2, 0, &foreign.anchors()).unwrap();
    let bytes = foreign.sign(&manifest).unwrap();
    assert_eq!(
        test_policy::verify_manifest(&bytes),
        Err(Reject::UnknownKey)
    );
}

#[test]
fn invalid_policy_and_duplicate_pairs_fail_closed_independent_of_order() {
    assert!(matches!(TrustPolicy::new(&[]), Err(Reject::KeyPolicy)));
    let baseline = test_policy::SIGNERS[0];
    let oversized = [baseline; MAX_SIGNERS + 1];
    assert!(matches!(
        TrustPolicy::new(&oversized),
        Err(Reject::KeyPolicy)
    ));
    for invalid in [
        Signer {
            first_version: 0,
            ..baseline
        },
        Signer {
            first_version: 3,
            last_version: 2,
            ..baseline
        },
    ] {
        assert!(matches!(
            TrustPolicy::new(&[test_policy::SIGNERS[1], invalid]),
            Err(Reject::KeyPolicy)
        ));
    }
    for duplicate in [
        baseline,
        Signer {
            revoked: true,
            ..baseline
        },
        Signer {
            first_version: 8,
            ..baseline
        },
    ] {
        assert!(matches!(
            TrustPolicy::new(&[baseline, duplicate]),
            Err(Reject::KeyPolicy)
        ));
        assert!(matches!(
            TrustPolicy::new(&[duplicate, baseline]),
            Err(Reject::KeyPolicy)
        ));
    }
}

#[test]
fn rejected_authority_never_becomes_an_install_or_boot_choice() {
    let (valid, _) = signed(0, 1);
    let current = test_policy::verify_manifest(&valid).unwrap();
    let header = update_core::Header {
        active: 0,
        generation: 1,
    };
    for (index, version, rejection) in [
        (2, 2, Reject::KeyRevoked),
        (3, 2, Reject::KeyExpired),
        (1, 1, Reject::KeyNotYetValid),
    ] {
        let (bytes, _) = signed(index, version);
        let candidate = test_policy::verify_manifest(&bytes);
        assert_eq!(
            update_core::decide(&header, 1, 1, candidate),
            update_core::Decision::Rejected(rejection)
        );
        let (choice, reasons) = update_core::choose(&header, [candidate, Ok(current)], 1);
        assert_eq!(choice.unwrap().slot, 1);
        assert_eq!(reasons[0], Some(rejection));
        assert!(update_core::choose(&header, [candidate, candidate], 1)
            .0
            .is_none());
    }
}

//! Reproduce public TEST vectors. No production key input or entropy is accepted.
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, env, fs, path::PathBuf};
use update_core::policy_envelope::{fixtures, verify_policy, MAX_POLICY_ENVELOPE_LEN};
use update_core::{manifest_for, policy::Signer, SigningKeys};

fn main() {
    let mode = env::args().nth(1).unwrap_or_else(|| "--check".into());
    assert!(
        matches!(mode.as_str(), "--check" | "--write"),
        "use --check or --write"
    );
    assert_eq!(env::args().count(), 2, "use exactly one explicit mode");
    let root = SigningKeys::from_seeds(&[0x71; 32], &[0x72; 32]);
    let release = SigningKeys::from_seeds(&[0x81; 32], &[0x82; 32]);
    let anchors = root.anchors();
    let signer = Signer {
        anchors: release.anchors(),
        first_version: 1,
        last_version: 4,
        revoked: false,
    };
    let payload: Vec<u8> = (0..64).map(|i| (i * 3 + 1) as u8).collect();
    let manifest = release
        .sign(&manifest_for(&payload, 2, 1, &signer.anchors).unwrap())
        .unwrap();
    let mut files = BTreeMap::<String, Vec<u8>>::new();
    files.insert("root-ed25519.bin".into(), anchors.ed25519.to_vec());
    files.insert("root-mldsa65.bin".into(), anchors.mldsa65.to_vec());
    files.insert("payload.bin".into(), payload);
    files.insert("manifest.bin".into(), manifest.to_vec());
    let mut bytes = [0u8; MAX_POLICY_ENVELOPE_LEN];
    let len = fixtures::encode_policy(1, 1, &[0; 32], &[signer], &root, &mut bytes).unwrap();
    let policy = verify_policy(&bytes[..len], &anchors).unwrap();
    let digest = policy.digest();
    files.insert(
        "binding.bin".into(),
        fixtures::sign_binding(&digest, &manifest, &release)
            .unwrap()
            .to_vec(),
    );
    files.insert("policy.bin".into(), bytes[..len].to_vec());
    for (name, version, parent, entry) in [
        (
            "policy-revoked",
            2,
            [0; 32],
            Signer {
                revoked: true,
                ..signer
            },
        ),
        (
            "policy-premature",
            2,
            digest,
            Signer {
                first_version: 3,
                ..signer
            },
        ),
        (
            "policy-expired",
            2,
            digest,
            Signer {
                last_version: 1,
                ..signer
            },
        ),
    ] {
        let len =
            fixtures::encode_policy(version, 1, &parent, &[entry], &root, &mut bytes).unwrap();
        let policy = verify_policy(&bytes[..len], &anchors).unwrap();
        if let Some(suffix) = name.strip_prefix("policy-") {
            files.insert(
                format!("binding-{suffix}.bin"),
                fixtures::sign_binding(&policy.digest(), &manifest, &release)
                    .unwrap()
                    .to_vec(),
            );
        }
        files.insert(format!("{name}.bin"), bytes[..len].to_vec());
    }
    let directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/policy_envelope/probe-fixtures");
    let hashes = files
        .iter()
        .map(|(name, bytes)| format!("{:x}  {name}\n", Sha256::digest(bytes)))
        .collect::<String>();
    files.insert("SHA256SUMS".into(), hashes.into_bytes());
    if mode == "--write" {
        fs::create_dir_all(&directory).unwrap();
        for (name, bytes) in &files {
            fs::write(directory.join(name), bytes).unwrap();
        }
    } else {
        let actual = fs::read_dir(&directory)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            actual,
            files.keys().cloned().collect(),
            "unexpected or missing fixture"
        );
        for (name, expected) in &files {
            assert_eq!(
                &fs::read(directory.join(name)).unwrap(),
                expected,
                "fixture mismatch: {name}"
            );
        }
    }
    println!("B03 public TEST vectors: {} files {}", files.len(), mode);
}

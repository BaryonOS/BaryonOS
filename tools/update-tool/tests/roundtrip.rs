//! The tool's keygen/sign/verify/pack round trip, run as a subprocess so the
//! command line itself is under test. Uses a scratch directory, never the
//! repository's TEST keys.
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};
use update_core::{Header, MANIFEST_LEN, SECTOR, SLOT_A_IMAGE, SLOT_A_MANIFEST, STORE_SECTORS};

fn tool(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_update-tool"))
        .args(args)
        .output()
        .expect("spawn update-tool")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "nieuw-os-update-tool-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn keygen_sign_verify_pack_round_trip() {
    let dir = scratch("roundtrip");
    let keys = dir.join("keys");
    let out = tool(&[
        "keygen",
        "--dir",
        keys.to_str().unwrap(),
        "--label",
        "TEST-scratch",
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for name in ["TEST-scratch-ed25519.seed", "TEST-scratch-mldsa65.seed"] {
            assert_eq!(
                fs::metadata(keys.join(name)).unwrap().permissions().mode() & 0o077,
                0
            );
        }
        for name in [
            "TEST-scratch-ed25519.pub",
            "TEST-scratch-mldsa65.pub",
            "TEST-scratch-anchors.json",
        ] {
            assert_eq!(
                fs::metadata(keys.join(name)).unwrap().permissions().mode() & 0o133,
                0
            );
        }
    }
    // Refuses to overwrite an existing label.
    let again = tool(&[
        "keygen",
        "--dir",
        keys.to_str().unwrap(),
        "--label",
        "TEST-scratch",
    ]);
    assert!(!again.status.success());
    assert_eq!(
        fs::read(keys.join("TEST-scratch-ed25519.pub"))
            .unwrap()
            .len(),
        32
    );
    assert_eq!(
        fs::read(keys.join("TEST-scratch-mldsa65.pub"))
            .unwrap()
            .len(),
        1952
    );

    let image = dir.join("image.bin");
    fs::write(
        &image,
        (0..5000u32).map(|i| (i % 253) as u8).collect::<Vec<_>>(),
    )
    .unwrap();
    let manifest = dir.join("manifest.bin");
    let common = ["--keys", keys.to_str().unwrap(), "--label", "TEST-scratch"];
    let out = tool(&[
        "sign",
        common[0],
        common[1],
        common[2],
        common[3],
        "--image",
        image.to_str().unwrap(),
        "--version",
        "7",
        "--min-version",
        "3",
        "--out",
        manifest.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(fs::read(&manifest).unwrap().len(), MANIFEST_LEN);
    let out = tool(&[
        "verify",
        common[0],
        common[1],
        common[2],
        common[3],
        "--manifest",
        manifest.to_str().unwrap(),
        "--image",
        image.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout)
        .contains("OK version=7 min_version=3 image_length=5000"));

    // One flipped byte in the ML-DSA signature region is a SIGNATURE rejection;
    // one flipped image byte is a DIGEST rejection; a foreign key is UNKNOWN_KEY.
    let mut tampered = fs::read(&manifest).unwrap();
    tampered[200] ^= 1;
    let bad = dir.join("tampered.bin");
    fs::write(&bad, &tampered).unwrap();
    let out = tool(&[
        "verify",
        common[0],
        common[1],
        common[2],
        common[3],
        "--manifest",
        bad.to_str().unwrap(),
    ]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("REJECTED:SIGNATURE"));
    let mut wrong_image = fs::read(&image).unwrap();
    wrong_image[10] ^= 0x80;
    let wrong = dir.join("wrong.bin");
    fs::write(&wrong, &wrong_image).unwrap();
    let out = tool(&[
        "verify",
        common[0],
        common[1],
        common[2],
        common[3],
        "--manifest",
        manifest.to_str().unwrap(),
        "--image",
        wrong.to_str().unwrap(),
    ]);
    assert!(String::from_utf8_lossy(&out.stderr).contains("REJECTED:DIGEST"));
    let other = dir.join("other");
    assert!(tool(&[
        "keygen",
        "--dir",
        other.to_str().unwrap(),
        "--label",
        "TEST-other"
    ])
    .status
    .success());
    let out = tool(&[
        "verify",
        "--keys",
        other.to_str().unwrap(),
        "--label",
        "TEST-other",
        "--manifest",
        manifest.to_str().unwrap(),
    ]);
    assert!(String::from_utf8_lossy(&out.stderr).contains("REJECTED:UNKNOWN_KEY"));

    // Pack places header, manifest and image at the fixed table positions.
    let store = dir.join("store.img");
    let out = tool(&[
        "pack",
        "--out",
        store.to_str().unwrap(),
        "--active",
        "A",
        "--generation",
        "4",
        "--slot-a-manifest",
        manifest.to_str().unwrap(),
        "--slot-a-image",
        image.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let bytes = fs::read(&store).unwrap();
    assert_eq!(bytes.len(), STORE_SECTORS as usize * SECTOR);
    let header = Header::decode(bytes[..SECTOR].try_into().unwrap()).unwrap();
    assert_eq!(
        header,
        Header {
            active: 0,
            generation: 4
        }
    );
    let at = SLOT_A_MANIFEST as usize * SECTOR;
    assert_eq!(
        &bytes[at..at + MANIFEST_LEN],
        fs::read(&manifest).unwrap().as_slice()
    );
    let at = SLOT_A_IMAGE as usize * SECTOR;
    assert_eq!(&bytes[at..at + 5000], fs::read(&image).unwrap().as_slice());
    let out = tool(&["header", "--active", "B", "--generation", "9"]);
    let expected: String = Header {
        active: 1,
        generation: 9,
    }
    .encode()
    .iter()
    .map(|b| format!("{b:02x}"))
    .collect();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), expected);
    // A wrong-sized image for the manifest is refused by pack.
    let out = tool(&[
        "pack",
        "--out",
        store.to_str().unwrap(),
        "--active",
        "B",
        "--slot-b-manifest",
        manifest.to_str().unwrap(),
        "--slot-b-image",
        wrong.to_str().unwrap(),
    ]);
    assert!(!out.status.success());
    fs::remove_dir_all(&dir).unwrap();
}

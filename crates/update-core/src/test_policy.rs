//! Public QEMU fixture anchors. These are never a release trust policy.
use crate::{policy::Signer, policy::TrustPolicy, Manifest, Reject, TrustAnchors, MANIFEST_LEN};

pub const SIGNERS: [Signer; 4] = [
    Signer {
        anchors: TrustAnchors {
            ed25519: *include_bytes!("../../../tools/update-keys/TEST-ed25519.pub"),
            mldsa65: *include_bytes!("../../../tools/update-keys/TEST-mldsa65.pub"),
        },
        first_version: 1,
        last_version: u64::MAX,
        revoked: false,
    },
    Signer {
        anchors: TrustAnchors {
            ed25519: *include_bytes!("../../../tools/update-keys/TEST-next-ed25519.pub"),
            mldsa65: *include_bytes!("../../../tools/update-keys/TEST-next-mldsa65.pub"),
        },
        first_version: 2,
        last_version: u64::MAX,
        revoked: false,
    },
    Signer {
        anchors: TrustAnchors {
            ed25519: *include_bytes!("../../../tools/update-keys/TEST-revoked-ed25519.pub"),
            mldsa65: *include_bytes!("../../../tools/update-keys/TEST-revoked-mldsa65.pub"),
        },
        first_version: 1,
        last_version: u64::MAX,
        revoked: true,
    },
    Signer {
        anchors: TrustAnchors {
            ed25519: *include_bytes!("../../../tools/update-keys/TEST-expired-ed25519.pub"),
            mldsa65: *include_bytes!("../../../tools/update-keys/TEST-expired-mldsa65.pub"),
        },
        first_version: 1,
        last_version: 1,
        revoked: false,
    },
];

pub fn verify_manifest(bytes: &[u8; MANIFEST_LEN]) -> Result<Manifest, Reject> {
    TrustPolicy::new(&SIGNERS)?.verify_manifest(bytes)
}

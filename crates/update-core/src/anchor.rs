//! Cold-freshness anchor chain (M1 protected slot, ADR-NET-ISSUER-20261125;
//! contract in `docs/implementation/COLD-FRESHNESS-ANCHOR.md`).
//!
//! HOST MODEL ONLY: no kernel anchor service exists yet and nothing here has
//! run against a TPM (S5). Each anchor domain is one `TPM_NT_EXTEND` NV index
//! whose value is a hash chain; `NV_Extend` sets it to `H(old ‖ data)`
//! (TPM 2.0 Part 3, 31.7). The kernel derives the extend event from the
//! domain of the caller's capability, so a holder cannot extend for another
//! domain. Every preimage field has a fixed length.

use sha2::{Digest as _, Sha256};

pub type Digest = [u8; 32];

pub const EVENT_EXT: &[u8] = b"BARYON-ANCHOR-EXT-V1";
pub const EVENT_GENESIS: &[u8] = b"BARYON-ANCHOR-GENESIS-V1";
pub const EVENT_INSTANCE: &[u8] = b"BARYON-ANCHOR-INSTANCE-V1";

fn sha(parts: &[&[u8]]) -> Digest {
    let mut d = Sha256::new();
    for p in parts {
        d.update(p);
    }
    d.finalize().into()
}

/// `E(S) = SHA-256("BARYON-ANCHOR-EXT-V1" ‖ dom ‖ D(S))`, 20 + 1 + 32 bytes.
pub fn extend_event(domain: u8, store: &Digest) -> Digest {
    sha(&[EVENT_EXT, &[domain], store])
}

/// The value `NV_Extend(data = event)` leaves on an index holding `prev`.
pub fn chain_step(prev: &Digest, event: &Digest) -> Digest {
    sha(&[prev, event])
}

/// Chain value once the commit of store digest `store` on top of `prev` is
/// anchored.
pub fn chain_after(domain: u8, prev: &Digest, store: &Digest) -> Digest {
    chain_step(prev, &extend_event(domain, store))
}

/// `instance = SHA-256("BARYON-ANCHOR-INSTANCE-V1" ‖ ENROLL digest ‖ dom ‖
/// provisioning nonce)`, 25 + 32 + 1 + 32 bytes. The ENROLL digest and the
/// nonce come from the signed enrolment or recovery record (via BootInfo).
pub fn instance(enroll: &Digest, domain: u8, nonce: &[u8; 32]) -> Digest {
    sha(&[EVENT_INSTANCE, enroll, &[domain], nonce])
}

/// Data of the single provisioning extend: `SHA-256("BARYON-ANCHOR-GENESIS-V1"
/// ‖ dom ‖ instance)`, 24 + 1 + 32 bytes.
pub fn genesis_event(domain: u8, instance: &Digest) -> Digest {
    sha(&[EVENT_GENESIS, &[domain], instance])
}

/// Chain value right after provisioning: the first extend of a freshly
/// defined index starts from the all-zero value (unverified, U21).
pub fn genesis_chain(domain: u8, instance: &Digest) -> Digest {
    chain_step(&[0; 32], &genesis_event(domain, instance))
}

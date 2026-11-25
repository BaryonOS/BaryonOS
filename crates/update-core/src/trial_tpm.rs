//! REL-05 TPM constants (B03-REL05 step S2, ADR-B03-REL05-TPM-20261125).
//!
//! Frozen values from `docs/implementation/B03-REL05-TRIAL-BOOT-TPM.md`
//! section 6.1 and `docs/implementation/COLD-FRESHNESS-ANCHOR.md`: the phase
//! PCR, its extend events, the NV handles, the NV attributes and the
//! `authPolicy` digests. Loader and kernel compare against these literals and
//! compute no hash at run time. `tests/trial_tpm_constants.rs` is the host tool:
//! it recomputes every digest with `sha2` and fails on any drift. Native
//! acceptance by swtpm is step S5.

/// Phase PCR (non-resettable; 16 and 23 are resettable from locality 0).
pub const PCR_PHASE: u32 = 13;

/// Event data the loader passes to `HashLogExtendEvent` (U2: the firmware
/// hashes it, so the PCR receives `SHA-256(event)`).
pub const EVENT_OPEN: &[u8] = b"BARYON-TRIAL-OPEN-V1";
pub const EVENT_CLOSED: &[u8] = b"BARYON-TRIAL-CLOSED-V1";
pub const EVENT_SEAL: &[u8] = b"BARYON-TRIAL-SEAL-V1";

/// `SHA-256(EVENT_*)`, the digest extended into `PCR_PHASE`.
pub const DIGEST_OPEN: [u8; 32] =
    h("9171e0beb82964796fce198ea6be3082542e47ac93413b260ca738025b10587e");
pub const DIGEST_CLOSED: [u8; 32] =
    h("045f890d159cf579eac941ebf1a97ed642a3c1a1867effeaa417079b1a5221b2");
pub const DIGEST_SEAL: [u8; 32] =
    h("cc801f70ad6aa65b1f619d2fe3f5dc94734e2436fca944e9cb97d2adf51776b3");

/// `PCR_PHASE` at loader start (all zero).
pub const PCR_ZERO: [u8; 32] = [0; 32];
/// `P_OPEN = SHA-256(0³² ‖ DIGEST_OPEN)`: the only value under which `CONFIRM`
/// is writable.
pub const P_OPEN: [u8; 32] = h("501c196abc9e732ec3471edd882639bf7368b168fd5580b67547f4dc1a3696c0");
/// `SHA-256(0³² ‖ DIGEST_CLOSED)`: phase value after a non-trial handover.
pub const P_CLOSED: [u8; 32] =
    h("6c3bbe55b4efb9b75ad347f6c2e1d14a23235dfda8027599614e8ef0faae57a2");

// TPM 2.0 Part 2 constants used below.
pub const TPM_ALG_SHA256: u16 = 0x000B;
pub const TPM_CC_NV_INCREMENT: u32 = 0x0000_0134;
pub const TPM_CC_NV_WRITE: u32 = 0x0000_0137;
pub const TPM_CC_POLICY_COMMAND_CODE: u32 = 0x0000_016C;
pub const TPM_CC_POLICY_PCR: u32 = 0x0000_017F;
/// `TPML_PCR_SELECTION` {count 1, SHA-256, sizeofSelect 3, PCR 13}.
pub const PCR_SELECTION_PHASE: [u8; 10] = [0, 0, 0, 1, 0x00, 0x0B, 3, 0x00, 0x20, 0x00];

// TPMA_NV bits (Part 2, 13.4).
pub const TPMA_NV_AUTHWRITE: u32 = 1 << 2;
pub const TPMA_NV_POLICYWRITE: u32 = 1 << 3;
pub const TPM_NT_COUNTER: u32 = 1 << 4;
pub const TPMA_NV_WRITEALL: u32 = 1 << 12;
pub const TPMA_NV_AUTHREAD: u32 = 1 << 18;
pub const TPMA_NV_NO_DA: u32 = 1 << 25;
/// Set by the TPM after the first write; masked out before comparison.
pub const TPMA_NV_WRITTEN: u32 = 1 << 29;

/// Ordinary, policy-written update records (`TRIAL`, `CONFIRM`, `FLOOR`).
pub const ATTR_UPDATE_ORDINARY: u32 =
    TPMA_NV_POLICYWRITE | TPMA_NV_WRITEALL | TPMA_NV_AUTHREAD | TPMA_NV_NO_DA;
/// Policy-incremented counter `GEN`.
pub const ATTR_UPDATE_COUNTER: u32 =
    TPM_NT_COUNTER | TPMA_NV_POLICYWRITE | TPMA_NV_AUTHREAD | TPMA_NV_NO_DA;
/// Cold-freshness anchor counter (empty `authValue`, not phase-bound).
pub const ATTR_ANCHOR_COUNTER: u32 =
    TPM_NT_COUNTER | TPMA_NV_AUTHWRITE | TPMA_NV_AUTHREAD | TPMA_NV_NO_DA;
/// Cold-freshness anchor slot.
pub const ATTR_ANCHOR_SLOT: u32 =
    TPMA_NV_AUTHWRITE | TPMA_NV_WRITEALL | TPMA_NV_AUTHREAD | TPMA_NV_NO_DA;

/// `PolicyPCR(13 = 0³²) + PolicyCommandCode(NV_Write)`; shared by `TRIAL` and
/// `FLOOR` (both loader-only, so sharing grants no other writer).
pub const POLICY_LOADER_WRITE: [u8; 32] =
    h("088d4acf1b7ad7d6a53a44d97668736524219be4e7148b45be4dcc3d00462816");
/// `PolicyPCR(13 = P_OPEN) + PolicyCommandCode(NV_Write)` for `CONFIRM`.
pub const POLICY_CONFIRM_WRITE: [u8; 32] =
    h("3388962f1cf962f3b6397ea05edaa3161e98bf935ae15f1248f19ef2efb33129");
/// `PolicyPCR(13 = 0³²) + PolicyCommandCode(NV_Increment)` for `GEN`.
pub const POLICY_LOADER_INCREMENT: [u8; 32] =
    h("e4f2860805e292f3712a85c063722fb7ad6f475889cef9c59667aba424db1bd1");

/// Expected NV public area (`TPMS_NV_PUBLIC`, `nameAlg` SHA-256).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NvSpec {
    pub handle: u32,
    pub attributes: u32,
    /// Empty for the anchor indices (`AUTHWRITE` with empty `authValue`).
    pub auth_policy: Option<[u8; 32]>,
    pub size: u16,
}

/// Largest encoded `TPMS_NV_PUBLIC` here: 4 + 2 + 4 + 2 + 32 + 2.
pub const NV_PUBLIC_MAX: usize = 46;

impl NvSpec {
    /// Big-endian `TPMS_NV_PUBLIC` as passed to `NV_DefineSpace` (WRITTEN clear).
    pub fn encode_public(&self) -> ([u8; NV_PUBLIC_MAX], usize) {
        let mut out = [0u8; NV_PUBLIC_MAX];
        out[0..4].copy_from_slice(&self.handle.to_be_bytes());
        out[4..6].copy_from_slice(&TPM_ALG_SHA256.to_be_bytes());
        out[6..10].copy_from_slice(&self.attributes.to_be_bytes());
        let mut at = 10;
        match self.auth_policy {
            Some(p) => {
                out[at..at + 2].copy_from_slice(&32u16.to_be_bytes());
                out[at + 2..at + 34].copy_from_slice(&p);
                at += 34;
            }
            None => {
                out[at..at + 2].copy_from_slice(&0u16.to_be_bytes());
                at += 2;
            }
        }
        out[at..at + 2].copy_from_slice(&self.size.to_be_bytes());
        (out, at + 2)
    }

    /// Fail-closed public-area check after `NV_ReadPublic` (section 6.1):
    /// everything must match except the TPM-owned `WRITTEN` bit.
    pub fn matches(&self, name_alg: u16, attributes: u32, auth_policy: &[u8], size: u16) -> bool {
        let policy_ok = match &self.auth_policy {
            Some(p) => auth_policy == p.as_slice(),
            None => auth_policy.is_empty(),
        };
        name_alg == TPM_ALG_SHA256
            && attributes & !TPMA_NV_WRITTEN == self.attributes
            && policy_ok
            && size == self.size
    }
}

pub const NV_TRIAL: NvSpec = NvSpec {
    handle: 0x0100_B300,
    attributes: ATTR_UPDATE_ORDINARY,
    auth_policy: Some(POLICY_LOADER_WRITE),
    size: 64,
};
pub const NV_CONFIRM: NvSpec = NvSpec {
    handle: 0x0100_B301,
    attributes: ATTR_UPDATE_ORDINARY,
    auth_policy: Some(POLICY_CONFIRM_WRITE),
    size: 64,
};
pub const NV_FLOOR: NvSpec = NvSpec {
    handle: 0x0100_B302,
    attributes: ATTR_UPDATE_ORDINARY,
    auth_policy: Some(POLICY_LOADER_WRITE),
    size: 64,
};
pub const NV_GEN: NvSpec = NvSpec {
    handle: 0x0100_B303,
    attributes: ATTR_UPDATE_COUNTER,
    auth_policy: Some(POLICY_LOADER_INCREMENT),
    size: 8,
};

/// Anchor domains 0..=7 (domain 0 = `AnchorDomain::Network`, the only one the
/// kernel exposes in this step).
pub const ANCHOR_DOMAINS: u8 = 8;
pub const ANCHOR_BASE: u32 = 0x0100_B310;

/// Counter and slot spec of anchor domain `d`, or `None` outside 0..8.
pub const fn anchor_domain(d: u8) -> Option<(NvSpec, NvSpec)> {
    if d >= ANCHOR_DOMAINS {
        return None;
    }
    let counter = ANCHOR_BASE + 2 * d as u32;
    Some((
        NvSpec {
            handle: counter,
            attributes: ATTR_ANCHOR_COUNTER,
            auth_policy: None,
            size: 8,
        },
        NvSpec {
            handle: counter + 1,
            attributes: ATTR_ANCHOR_SLOT,
            auth_policy: None,
            size: 64,
        },
    ))
}

/// Compile-time hex decoder for the frozen digests.
const fn h(s: &str) -> [u8; 32] {
    const fn nib(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            _ => panic!("hex"),
        }
    }
    let b = s.as_bytes();
    assert!(b.len() == 64);
    let mut out = [0u8; 32];
    let mut i = 0;
    while i < 32 {
        out[i] = (nib(b[2 * i]) << 4) | nib(b[2 * i + 1]);
        i += 1;
    }
    out
}

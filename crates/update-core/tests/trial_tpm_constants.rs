//! S2 host tool: recompute every frozen REL-05 TPM constant with `sha2`
//! (TPM 2.0 Part 3: PolicyPCR 23.7, PolicyCommandCode 23.11) and fail on drift.
//! The NV names were also computed independently with Python `hashlib`;
//! swtpm must report the same names in S5.
use sha2::{Digest, Sha256};
use update_core::trial_tpm::*;

fn sha(parts: &[&[u8]]) -> [u8; 32] {
    let mut d = Sha256::new();
    for p in parts {
        d.update(p);
    }
    d.finalize().into()
}

fn extend(pcr: [u8; 32], event: &[u8]) -> [u8; 32] {
    sha(&[&pcr, &sha(&[event])])
}

fn policy(pcr_value: [u8; 32], cc: u32) -> [u8; 32] {
    let d1 = sha(&[
        &[0u8; 32],
        &TPM_CC_POLICY_PCR.to_be_bytes(),
        &PCR_SELECTION_PHASE,
        &sha(&[&pcr_value]),
    ]);
    sha(&[
        &d1,
        &TPM_CC_POLICY_COMMAND_CODE.to_be_bytes(),
        &cc.to_be_bytes(),
    ])
}

fn name(spec: &NvSpec) -> String {
    let (buf, len) = spec.encode_public();
    let mut n = TPM_ALG_SHA256.to_be_bytes().to_vec();
    n.extend_from_slice(&sha(&[&buf[..len]]));
    n.iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn phase_values() {
    assert_eq!(DIGEST_OPEN, sha(&[EVENT_OPEN]));
    assert_eq!(DIGEST_CLOSED, sha(&[EVENT_CLOSED]));
    assert_eq!(DIGEST_SEAL, sha(&[EVENT_SEAL]));
    assert_eq!(P_OPEN, extend(PCR_ZERO, EVENT_OPEN));
    assert_eq!(P_CLOSED, extend(PCR_ZERO, EVENT_CLOSED));
    // The kernel's SEAL must close the CONFIRM window.
    assert_ne!(extend(P_OPEN, EVENT_SEAL), P_OPEN);
    assert_ne!(P_OPEN, P_CLOSED);
}

#[test]
fn pcr_selection_is_pcr13_sha256() {
    let sel = &PCR_SELECTION_PHASE;
    assert_eq!(u32::from_be_bytes(sel[0..4].try_into().unwrap()), 1);
    assert_eq!(
        u16::from_be_bytes(sel[4..6].try_into().unwrap()),
        TPM_ALG_SHA256
    );
    assert_eq!(sel[6], 3);
    let bits = u32::from(sel[7]) | (u32::from(sel[8]) << 8) | (u32::from(sel[9]) << 16);
    assert_eq!(bits, 1 << PCR_PHASE);
}

#[test]
fn policy_digests() {
    assert_eq!(POLICY_LOADER_WRITE, policy(PCR_ZERO, TPM_CC_NV_WRITE));
    assert_eq!(POLICY_CONFIRM_WRITE, policy(P_OPEN, TPM_CC_NV_WRITE));
    assert_eq!(
        POLICY_LOADER_INCREMENT,
        policy(PCR_ZERO, TPM_CC_NV_INCREMENT)
    );
    // The kernel phase (P_OPEN) satisfies no loader policy, and vice versa.
    for p in [POLICY_LOADER_WRITE, POLICY_LOADER_INCREMENT] {
        assert_ne!(p, policy(P_OPEN, TPM_CC_NV_WRITE));
        assert_ne!(p, policy(P_OPEN, TPM_CC_NV_INCREMENT));
    }
    assert_ne!(POLICY_CONFIRM_WRITE, policy(PCR_ZERO, TPM_CC_NV_WRITE));
    assert_eq!(POLICY_LOADER_READ, policy(PCR_ZERO, TPM_CC_NV_READ));
    assert_ne!(POLICY_LOADER_READ, policy(P_OPEN, TPM_CC_NV_READ));
    assert_ne!(POLICY_LOADER_READ, policy(P_CLOSED, TPM_CC_NV_READ));
    assert_ne!(POLICY_CONFIRM_WRITE, policy(P_CLOSED, TPM_CC_NV_WRITE));
}

#[test]
fn attributes_match_spec_6_1() {
    // PPWRITE, OWNERWRITE, POLICY_DELETE, WRITEDEFINE, WRITE_STCLEAR, ORDERLY, READ_STCLEAR.
    let forbidden: u32 =
        (1 << 0) | (1 << 1) | (1 << 10) | (1 << 13) | (1 << 14) | (1 << 26) | (1 << 31);
    for spec in [NV_TRIAL, NV_CONFIRM, NV_FLOOR, NV_GEN, NV_LREC] {
        assert_eq!(spec.attributes & TPMA_NV_AUTHWRITE, 0, "{:#x}", spec.handle);
        assert_ne!(spec.attributes & TPMA_NV_POLICYWRITE, 0);
        assert_eq!(spec.attributes & forbidden, 0);
    }
    for a in (0..ANCHOR_DOMAINS).map(|d| anchor_domain(d).unwrap()) {
        assert_eq!(a.attributes & forbidden, 0);
        assert_eq!(a.attributes & (0xf << 4), TPM_NT_EXTEND);
        assert_eq!(a.size, 32, "extend index holds one SHA-256 digest");
    }
    assert_eq!(ATTR_UPDATE_ORDINARY, 0x0204_1008);
    assert_eq!(ATTR_UPDATE_COUNTER, 0x0204_0018);
    assert_eq!(ATTR_ANCHOR_CHAIN, 0x0204_0044);
    // HAUTH: owner-written, policy-read only (no AUTHREAD, no OWNERREAD, no
    // POLICY_DELETE), ordinary, 64 bytes; swtpm accepted 0x02081002.
    assert_eq!(ATTR_HAUTH, 0x0208_1002);
    assert_eq!(ATTR_HAUTH & (TPMA_NV_AUTHREAD | (1 << 17) | (1 << 10)), 0);
    assert_eq!(ATTR_HAUTH & (0xf << 4), 0);
    assert_eq!(NV_HAUTH.size, 64);
}

#[test]
fn nv_names_frozen() {
    let expect = [
        (
            NV_TRIAL,
            "000b9e8b10f8d7144551838b922be39c6147beeaa56c7c8fff64b59aa6f98ee6eb8f",
        ),
        (
            NV_CONFIRM,
            "000ba3a467a8867f40431a7e7d89740fe735e0a53cc02852395926d43b3ce7588642",
        ),
        (
            NV_FLOOR,
            "000bbf1bb71997212d93d900850521e9e2b1fb9aee10dfce4bd1aa518f9f2355b2f1",
        ),
        (
            NV_GEN,
            "000b959b5113b338f9129a3715e8ab4329a0fd0790db8721fbedf57244585c13c91c",
        ),
        (
            NV_LREC,
            "000bad14c18b2e6f42a9e51a1cdfb8d6ef84d0dba6fcfd0598852bde1504447c4782",
        ),
    ];
    for (spec, n) in expect {
        assert_eq!(name(&spec), n, "{:#x}", spec.handle);
    }
    // The Name hashes the current attributes, WRITTEN included, so the
    // anchor index has a different Name after its genesis extend.
    let a = anchor_domain(0).unwrap();
    assert_eq!(
        name(&a),
        "000b4626434643c485b98e9e38ab863cccd989faa890fe6067f7fd820ca3a2c84db9"
    );
    let written = NvSpec {
        attributes: a.attributes | TPMA_NV_WRITTEN,
        ..a
    };
    assert_eq!(
        name(&written),
        "000b442b22e4b54d200b29adf9ce099a389754078300340bdcb3764d0b51608d1f6d"
    );
    // HAUTH: the loader compares the Name before writing the secret and on
    // every boot (squatting). Both forms observed on swtpm 0.10.1
    // (docs/verification/b03-rel05-squat-probe-20261125, N1).
    assert_eq!(
        name(&NV_HAUTH),
        "000b8bc449e1e502ccff31629cdf815c711e7ef01c335d2a45695e402373a21fd496"
    );
    let written = NvSpec {
        attributes: NV_HAUTH.attributes | TPMA_NV_WRITTEN,
        ..NV_HAUTH
    };
    assert_eq!(
        name(&written),
        "000bbd7dcf06efd2bc90ae362d1a85da21c406bc9b64944632c26e2f6f513f739ce2"
    );
    // LREC (U26): trusted only with the written Name; an unwritten index with
    // the expected public area is what a run-time definer could leave behind.
    let written = NvSpec {
        attributes: NV_LREC.attributes | TPMA_NV_WRITTEN,
        ..NV_LREC
    };
    assert_eq!(
        name(&written),
        "000b282efe987c36f93dc5464cfc70b668f196f74e7bcf62ace9c1a3757ef81f7a60"
    );
}

#[test]
fn anchor_handles() {
    let handle = |d| anchor_domain(d).map(|a| a.handle);
    assert_eq!(handle(0), Some(0x0100_B310));
    assert_eq!(handle(1), Some(0x0100_B311));
    assert_eq!(handle(7), Some(0x0100_B317));
    assert_eq!(handle(ANCHOR_DOMAINS), None);
    let update = [
        NV_TRIAL.handle,
        NV_CONFIRM.handle,
        NV_FLOOR.handle,
        NV_GEN.handle,
        NV_HAUTH.handle,
    ];
    assert_eq!(NV_HAUTH.handle, 0x0100_B304);
    for d in 0..ANCHOR_DOMAINS {
        let h = anchor_domain(d).unwrap().handle;
        assert!((0x0100_B300..=0x0100_B31F).contains(&h));
        assert!(!update.contains(&h));
    }
}

#[test]
fn matches_is_fail_closed() {
    let s = NV_CONFIRM;
    let p = s.auth_policy.unwrap();
    assert!(s.matches(TPM_ALG_SHA256, s.attributes, &p, 64));
    assert!(s.matches(TPM_ALG_SHA256, s.attributes | TPMA_NV_WRITTEN, &p, 64));
    assert!(!s.matches(0x0004, s.attributes, &p, 64));
    assert!(!s.matches(TPM_ALG_SHA256, s.attributes | TPMA_NV_AUTHWRITE, &p, 64));
    assert!(!s.matches(TPM_ALG_SHA256, s.attributes, &POLICY_LOADER_WRITE, 64));
    assert!(!s.matches(TPM_ALG_SHA256, s.attributes, &p[..31], 64));
    assert!(!s.matches(TPM_ALG_SHA256, s.attributes, &[], 64));
    assert!(!s.matches(TPM_ALG_SHA256, s.attributes, &p, 63));
    let a = anchor_domain(0).unwrap();
    assert!(a.matches(TPM_ALG_SHA256, a.attributes, &[], 32));
    assert!(a.matches(TPM_ALG_SHA256, a.attributes | TPMA_NV_WRITTEN, &[], 32));
    assert!(!a.matches(TPM_ALG_SHA256, a.attributes, &p, 32));
    assert!(!a.matches(TPM_ALG_SHA256, a.attributes, &[], 8));
    // A counter or ordinary index squatting the handle is refused.
    let squat = a.attributes & !(0xf << 4);
    assert!(!a.matches(TPM_ALG_SHA256, squat | TPM_NT_COUNTER, &[], 32));
    assert!(!a.matches(TPM_ALG_SHA256, squat, &[], 32));
}

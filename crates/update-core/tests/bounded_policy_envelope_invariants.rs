//! B05-T4: bounded mutation enumeration over the committed B03 policy envelope.
//!
//! Input: the committed public TEST vector `probe-fixtures/policy.bin` (one
//! signer, 5,493 bytes) and its pinned TEST root. No key material is generated
//! or embedded here. Bound: every byte position x up to four replacement values
//! {0x00, 0xff, b^0x01, b^0x80} (deduplicated, never equal to the original),
//! every truncation 0..len, and 1..=16 appended bytes of four values. Each case
//! must yield the exact typed reject derived from `validate_structure` and
//! `verify_hybrid`; only the unmodified bytes may verify. This is bounded
//! enumeration in the debug profile, not a proof and not an external audit.
//! Mutation control: in `verify_hybrid` (src/lib.rs), change the condition
//! `if !pq_key.verify_with_context(message, b"", &pq_sig)` to
//! `if !pq_key.verify_with_context(message, b"", &pq_sig) && false`, so an
//! invalid ML-DSA signature is never rejected;
//! `single_byte_mutations_get_the_exact_reject` must then fail on ML-DSA
//! signature-region cases.
#![cfg(feature = "experimental-policy-test")]

use std::thread;
use update_core::policy_envelope::{
    validate_successor, verify_policy, PolicyReject, HYBRID_SIGNATURE_LEN, POLICY_HEADER_LEN,
    SIGNER_ENTRY_LEN,
};
use update_core::{Reject, TrustAnchors, ED25519_PUBLIC, MLDSA65_PUBLIC};

const FIXTURES: &str = "src/policy_envelope/probe-fixtures";
const POLICY: &[u8] = include_bytes!("../src/policy_envelope/probe-fixtures/policy.bin");
const PREMATURE: &[u8] =
    include_bytes!("../src/policy_envelope/probe-fixtures/policy-premature.bin");
const EXPIRED: &[u8] = include_bytes!("../src/policy_envelope/probe-fixtures/policy-expired.bin");
const REVOKED: &[u8] = include_bytes!("../src/policy_envelope/probe-fixtures/policy-revoked.bin");
const ROOT_ED: &[u8] = include_bytes!("../src/policy_envelope/probe-fixtures/root-ed25519.bin");
const ROOT_PQ: &[u8] = include_bytes!("../src/policy_envelope/probe-fixtures/root-mldsa65.bin");
const PAIR: usize = ED25519_PUBLIC + MLDSA65_PUBLIC;
const ENTRY: usize = POLICY_HEADER_LEN;
const BODY: usize = POLICY_HEADER_LEN + SIGNER_ENTRY_LEN;
const SIGNATURE: PolicyReject = PolicyReject::Core(Reject::Signature);

fn root() -> TrustAnchors {
    TrustAnchors {
        ed25519: ROOT_ED.try_into().unwrap(),
        mldsa65: ROOT_PQ.try_into().unwrap(),
    }
}

fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

/// Independent statement of the reject a single changed byte must produce.
fn expected(mutated: &[u8], position: usize) -> PolicyReject {
    match position {
        0..=15 => PolicyReject::Format, // magic, format version, count, body length
        16..=31 => PolicyReject::Target,
        32..=47 if u64_at(mutated, position & !7) == 0 => PolicyReject::Format,
        80..=111 => PolicyReject::Root,
        p if (ENTRY..BODY).contains(&p) && p >= ENTRY + PAIR => {
            let first = u64_at(mutated, ENTRY + PAIR);
            let last = u64_at(mutated, ENTRY + PAIR + 8);
            let revoked = mutated[ENTRY + PAIR + 16];
            let padding = mutated[ENTRY + PAIR + 17..BODY].iter().any(|b| *b != 0);
            if first == 0 || last < first || revoked > 1 || padding {
                PolicyReject::SignerTable
            } else {
                SIGNATURE
            }
        }
        // Version, floor, predecessor, signer keys and both signature legs.
        _ => SIGNATURE,
    }
}

fn values(original: u8) -> Vec<u8> {
    let mut out = Vec::new();
    for value in [0x00, 0xff, original ^ 0x01, original ^ 0x80] {
        if value != original && !out.contains(&value) {
            out.push(value);
        }
    }
    out
}

#[test]
fn fixture_layout_and_original_verifies() {
    assert_eq!(POLICY.len(), BODY + HYBRID_SIGNATURE_LEN, "{FIXTURES}");
    let verified = verify_policy(POLICY, &root()).expect("unmodified TEST vector verifies");
    assert_eq!(verified.version(), 1);
    assert_eq!(verified.signer_count(), 1);
}

#[test]
fn single_byte_mutations_get_the_exact_reject() {
    let anchors = root();
    let workers = thread::available_parallelism().map_or(1, |n| n.get().min(8));
    let chunk = POLICY.len().div_ceil(workers);
    let counts: Vec<[usize; 6]> = thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|worker| {
                let anchors = &anchors;
                scope.spawn(move || {
                    let mut counts = [0usize; 6];
                    let mut bytes = POLICY.to_vec();
                    let end = POLICY.len().min((worker + 1) * chunk);
                    for position in worker * chunk..end {
                        for value in values(POLICY[position]) {
                            bytes[position] = value;
                            let want = expected(&bytes, position);
                            let got = verify_policy(&bytes, anchors).map(|_| ());
                            assert_eq!(got, Err(want), "position {position} value {value:#04x}");
                            counts[match want {
                                PolicyReject::Format => 0,
                                PolicyReject::Target => 1,
                                PolicyReject::Root => 2,
                                PolicyReject::SignerTable => 3,
                                _ if position >= BODY => 4,
                                _ => 5,
                            }] += 1;
                        }
                        bytes[position] = POLICY[position];
                    }
                    counts
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut total = [0usize; 6];
    for c in counts {
        for (t, n) in total.iter_mut().zip(c) {
            *t += n;
        }
    }
    assert_eq!(total.iter().sum::<usize>(), 21_760, "{total:?}");
    // Every reject class must actually be reached, or the oracle is vacuous.
    assert!(total.iter().all(|n| *n > 0), "{total:?}");
    println!(
        "cases={} format={} target={} root={} signer_table={} signature(sig region)={} signature(body)={}",
        total.iter().sum::<usize>(),
        total[0],
        total[1],
        total[2],
        total[3],
        total[4],
        total[5]
    );
}

#[test]
fn every_truncation_and_append_is_a_format_reject() {
    let anchors = root();
    for length in 0..POLICY.len() {
        assert_eq!(
            verify_policy(&POLICY[..length], &anchors).map(|_| ()),
            Err(PolicyReject::Format),
            "length {length}"
        );
    }
    for extra in 1..=16 {
        for value in [0x00, 0x01, 0x80, 0xff] {
            let mut bytes = POLICY.to_vec();
            bytes.resize(POLICY.len() + extra, value);
            assert_eq!(
                verify_policy(&bytes, &anchors).map(|_| ()),
                Err(PolicyReject::Format),
                "append {extra} x {value:#04x}"
            );
        }
    }
}

#[test]
fn any_other_root_is_rejected_before_signature_work() {
    let anchors = root();
    for position in 0..ED25519_PUBLIC + MLDSA65_PUBLIC {
        let mut wrong = TrustAnchors { ..anchors };
        if position < ED25519_PUBLIC {
            wrong.ed25519[position] ^= 1;
        } else {
            wrong.mldsa65[position - ED25519_PUBLIC] ^= 1;
        }
        assert_eq!(
            verify_policy(POLICY, &wrong).map(|_| ()),
            Err(PolicyReject::Root),
            "root byte {position}"
        );
    }
}

/// `verify_policy` has no version floor; lineage, stale and replayed versions
/// are enforced by `validate_successor`. The oracle models all three of its
/// conditions in order: equal root identifier (body 80..112, else `Root`), a
/// strictly greater version with predecessor digest equal to the current
/// digest (else `Successor`), and every revoked current signer pair retained
/// as revoked in the next table (else `Revocation`). Bound: only the 16 ordered
/// pairs of the four committed fixtures. They share one root and only `policy`
/// precedes a linked successor with no revoked signers, so only the `Ok` and
/// `Successor` outcomes are reachable here; `Root` and `Revocation` rejects are
/// covered by the existing `tests/policy_envelope.rs`.
#[test]
fn successor_accepts_only_strictly_newer_linked_policies() {
    let anchors = root();
    let all = [
        ("policy", POLICY),
        ("premature", PREMATURE),
        ("expired", EXPIRED),
        ("revoked", REVOKED),
    ];
    let signers = |bytes: &[u8]| -> Vec<(Vec<u8>, u8)> {
        let count = u16::from_le_bytes(bytes[10..12].try_into().unwrap()) as usize;
        (0..count)
            .map(|i| {
                let e = &bytes[ENTRY + i * SIGNER_ENTRY_LEN..];
                (e[..PAIR].to_vec(), e[PAIR + 16])
            })
            .collect()
    };
    for (current_name, current_bytes) in all {
        for (next_name, next_bytes) in all {
            let current = verify_policy(current_bytes, &anchors).unwrap();
            let next = verify_policy(next_bytes, &anchors).unwrap();
            let next_signers = signers(next_bytes);
            let want = if current_bytes[80..112] != next_bytes[80..112] {
                Err(PolicyReject::Root)
            } else if next.version() <= current.version()
                || next.predecessor_digest() != current.digest()
            {
                Err(PolicyReject::Successor)
            } else if signers(current_bytes)
                .iter()
                .filter(|(_, revoked)| *revoked == 1)
                .any(|(pair, _)| !next_signers.contains(&(pair.clone(), 1)))
            {
                Err(PolicyReject::Revocation)
            } else {
                Ok(())
            };
            assert_eq!(
                validate_successor(&current, &next),
                want,
                "{current_name} -> {next_name}"
            );
        }
    }
    // The fixture set must contain both accepted and stale/unlinked transitions.
    let p = verify_policy(POLICY, &anchors).unwrap();
    assert_eq!(
        validate_successor(&p, &verify_policy(PREMATURE, &anchors).unwrap()),
        Ok(())
    );
    assert_eq!(
        validate_successor(&verify_policy(PREMATURE, &anchors).unwrap(), &p),
        Err(PolicyReject::Successor)
    );
    assert_eq!(
        validate_successor(&p, &verify_policy(REVOKED, &anchors).unwrap()),
        Err(PolicyReject::Successor)
    );
}

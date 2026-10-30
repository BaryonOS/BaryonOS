//! Host verification of the actual B02 codecs. Deterministic entropy is a test
//! fixture, never an entropy claim. These tests do not execute a native disk or
//! supply a trusted cold-boot rollback anchor.

use chacha20poly1305::{
    aead::{AeadInPlace, KeyInit},
    Key, XChaCha20Poly1305, XNonce,
};
use hkdf::Hkdf;
use keyring_core::{
    persistent::{SnapshotInfo, PERSISTENT_BLOB, PERSISTENT_HEADER, PERSISTENT_PLAIN},
    records::{RecordContext, RecordKind},
    EntropySource, Service, State, Status, DOMAINS, KEYS_PER_DOMAIN, KEY_BYTES, MAX_PLAINTEXT,
    MAX_SEALED, NONCE_BYTES, RESEED_INTERVAL, SECRET_BYTES,
};
use sha2::Sha256;

const VOLUME: [u8; 16] = *b"test-volume-0001";
const SECRET: [u8; SECRET_BYTES] = [0x35; SECRET_BYTES];

struct Fake {
    state: u64,
    fail: bool,
}

impl EntropySource for Fake {
    fn read(&mut self, out: &mut [u8]) -> Result<(), Status> {
        if self.fail {
            return Err(Status::Entropy);
        }
        for byte in out {
            self.state ^= self.state << 13;
            self.state ^= self.state >> 7;
            self.state ^= self.state << 17;
            *byte = (self.state >> 24) as u8;
        }
        Ok(())
    }
}

fn ready(seed: u64) -> (Service, Fake) {
    let mut source = Fake {
        state: seed,
        fail: false,
    };
    let mut service = Service::new();
    service.start(&mut source).unwrap();
    (service, source)
}

fn context(length: usize) -> RecordContext {
    RecordContext {
        volume: VOLUME,
        domain: 0,
        object: 42,
        revision: 9,
        chunk: 3,
        length: length as u16,
        kind: RecordKind::Content,
        schema: 1,
    }
}

fn export(service: &mut Service, source: &mut Fake) -> ([u8; PERSISTENT_BLOB], SnapshotInfo) {
    let mut blob = [0u8; PERSISTENT_BLOB];
    let info = service
        .export_persistent(&VOLUME, &SECRET, source, &mut blob)
        .unwrap();
    (blob, info)
}

#[test]
fn records_round_trip_every_supported_length_with_fresh_nonces_and_untouched_tails() {
    let (mut service, mut source) = ready(1);
    let key = service.generate(0, &mut source).unwrap();
    let plaintext = core::array::from_fn::<_, MAX_PLAINTEXT, _>(|i| i as u8);
    let mut previous_nonce = [0; NONCE_BYTES];
    for length in 0..=MAX_PLAINTEXT {
        let mut sealed = [0xa5; MAX_SEALED];
        let written = service
            .seal_record(
                0,
                key,
                context(length),
                &plaintext[..length],
                &mut source,
                &mut sealed,
            )
            .unwrap();
        assert_eq!(written, NONCE_BYTES + length + 16);
        assert!(sealed[written..].iter().all(|byte| *byte == 0xa5));
        assert_ne!(sealed[..NONCE_BYTES], previous_nonce);
        previous_nonce.copy_from_slice(&sealed[..NONCE_BYTES]);
        let mut out = [0x77; MAX_PLAINTEXT];
        assert_eq!(
            service.unseal_record(
                0,
                key,
                context(length),
                &sealed[..written],
                &mut source,
                &mut out
            ),
            Ok(length)
        );
        assert_eq!(out[..length], plaintext[..length]);
        assert!(out[length..].iter().all(|byte| *byte == 0x77));
    }
}

#[test]
fn records_authenticate_every_identity_field_and_key_handle() {
    let (mut service, mut source) = ready(2);
    service.generate(0, &mut source).unwrap();
    service.generate(0, &mut source).unwrap();
    service.generate(1, &mut source).unwrap();
    let expected = context(4);
    let mut sealed = [0; MAX_SEALED];
    let length = service
        .seal_record(0, 0, expected, b"data", &mut source, &mut sealed)
        .unwrap();
    let alternatives = [
        RecordContext {
            volume: [7; 16],
            ..expected
        },
        RecordContext {
            object: 43,
            ..expected
        },
        RecordContext {
            revision: 10,
            ..expected
        },
        RecordContext {
            chunk: 4,
            ..expected
        },
        RecordContext {
            domain: 1,
            ..expected
        },
        RecordContext {
            kind: RecordKind::Name,
            ..expected
        },
        RecordContext {
            schema: 2,
            ..expected
        },
    ];
    for changed in alternatives {
        let mut out = [0x88; MAX_PLAINTEXT];
        assert_eq!(
            service.unseal_record(
                usize::from(changed.domain),
                0,
                changed,
                &sealed[..length],
                &mut source,
                &mut out
            ),
            Err(Status::Auth)
        );
        assert_eq!(out, [0x88; MAX_PLAINTEXT]);
    }
    let mut out = [0x88; MAX_PLAINTEXT];
    assert_eq!(
        service.unseal_record(0, 1, expected, &sealed[..length], &mut source, &mut out),
        Err(Status::Auth)
    );
    assert_eq!(out, [0x88; MAX_PLAINTEXT]);
}

#[test]
fn equally_sized_names_metadata_and_content_are_not_interchangeable() {
    let (mut service, mut source) = ready(37);
    service.generate(0, &mut source).unwrap();
    let kinds = [
        RecordKind::Content,
        RecordKind::Name,
        RecordKind::ObjectMetadata,
        RecordKind::VolumeManifest,
        RecordKind::SearchIndex,
    ];
    for kind in kinds {
        let ctx = RecordContext { kind, ..context(4) };
        let mut sealed = [0; MAX_SEALED];
        let length = service
            .seal_record(0, 0, ctx, b"name", &mut source, &mut sealed)
            .unwrap();
        for other_kind in kinds {
            let expected = RecordContext {
                kind: other_kind,
                ..ctx
            };
            let mut out = [0x19; MAX_PLAINTEXT];
            let result =
                service.unseal_record(0, 0, expected, &sealed[..length], &mut source, &mut out);
            if other_kind == kind {
                assert_eq!(result, Ok(4));
                assert_eq!(&out[..4], b"name");
            } else {
                assert_eq!(result, Err(Status::Auth));
                assert_eq!(out, [0x19; MAX_PLAINTEXT]);
            }
        }
    }
    for code in 0..=u8::MAX {
        assert_eq!(
            RecordKind::from_code(code).is_some(),
            (1..=5).contains(&code)
        );
    }
}

#[test]
fn every_record_bitflip_is_rejected_without_plaintext_or_namespace_changes() {
    let (mut service, mut source) = ready(3);
    service.generate(0, &mut source).unwrap();
    let mut sealed = [0; MAX_SEALED];
    let length = service
        .seal_record(
            0,
            0,
            context(MAX_PLAINTEXT),
            &[5; MAX_PLAINTEXT],
            &mut source,
            &mut sealed,
        )
        .unwrap();
    let before = service.snapshot_info();
    for bit in 0..length * 8 {
        let mut changed = sealed;
        changed[bit / 8] ^= 1 << (bit % 8);
        let mut out = [0x98; MAX_PLAINTEXT];
        assert_eq!(
            service.unseal_record(
                0,
                0,
                context(MAX_PLAINTEXT),
                &changed[..length],
                &mut source,
                &mut out
            ),
            Err(Status::Auth),
            "bit {bit}"
        );
        assert_eq!(out, [0x98; MAX_PLAINTEXT]);
        assert_eq!(service.snapshot_info(), before);
    }
}

#[test]
fn record_lengths_and_caller_domains_are_checked_before_accounting() {
    let (mut service, mut source) = ready(4);
    service.generate(0, &mut source).unwrap();
    let mut out = [0x91; MAX_SEALED];
    let counters = service.counters;
    let before = service.snapshot_info();
    for (domain, ctx, bytes) in [
        (DOMAINS, context(1), &b"x"[..]),
        (1, context(1), &b"x"[..]),
        (
            0,
            RecordContext {
                domain: 255,
                ..context(1)
            },
            &b"x"[..],
        ),
        (0, context(MAX_PLAINTEXT + 1), &b"x"[..]),
        (0, context(2), &b"x"[..]),
        (
            0,
            RecordContext {
                schema: 0,
                ..context(1)
            },
            &b"x"[..],
        ),
    ] {
        assert_eq!(
            service.seal_record(domain, 0, ctx, bytes, &mut source, &mut out),
            Err(Status::Invalid)
        );
        assert_eq!(out, [0x91; MAX_SEALED]);
    }
    assert_eq!(service.counters, counters);
    assert_eq!(service.snapshot_info(), before);
}

#[test]
fn records_reject_all_truncations_extra_bytes_and_declared_length_changes() {
    let (mut service, mut source) = ready(5);
    service.generate(0, &mut source).unwrap();
    let mut sealed = [0; MAX_SEALED];
    let length = service
        .seal_record(0, 0, context(4), b"data", &mut source, &mut sealed)
        .unwrap();
    for shortened in 0..length {
        let mut out = [0x81; MAX_PLAINTEXT];
        assert_eq!(
            service.unseal_record(
                0,
                0,
                context(4),
                &sealed[..shortened],
                &mut source,
                &mut out
            ),
            Err(Status::Invalid)
        );
        assert_eq!(out, [0x81; MAX_PLAINTEXT]);
    }
    for changed in [context(3), context(5)] {
        let mut out = [0x81; MAX_PLAINTEXT];
        assert_eq!(
            service.unseal_record(0, 0, changed, &sealed[..length], &mut source, &mut out),
            Err(Status::Invalid)
        );
    }
    let mut out = [0x81; MAX_PLAINTEXT];
    assert_eq!(
        service.unseal_record(
            0,
            0,
            context(4),
            &sealed[..length + 1],
            &mut source,
            &mut out
        ),
        Err(Status::Invalid)
    );
    assert_eq!(out, [0x81; MAX_PLAINTEXT]);
}

#[test]
fn record_revision_replay_requires_a_trusted_current_context() {
    let (mut service, mut source) = ready(6);
    service.generate(0, &mut source).unwrap();
    let mut sealed = [0; MAX_SEALED];
    let length = service
        .seal_record(0, 0, context(4), b"old!", &mut source, &mut sealed)
        .unwrap();
    let mut out = [0x42; MAX_PLAINTEXT];
    assert_eq!(
        service.unseal_record(
            0,
            0,
            RecordContext {
                revision: 10,
                ..context(4)
            },
            &sealed[..length],
            &mut source,
            &mut out
        ),
        Err(Status::Auth)
    );
    assert_eq!(out, [0x42; MAX_PLAINTEXT]);
    // Replaying BOTH the old manifest context and ciphertext remains possible;
    // the codec deliberately does not claim whole-volume freshness.
    assert_eq!(
        service.unseal_record(0, 0, context(4), &sealed[..length], &mut source, &mut out),
        Ok(4)
    );
    assert_eq!(&out[..4], b"old!");
}

#[test]
fn legacy_and_record_ciphertexts_are_not_interchangeable() {
    let (mut service, mut source) = ready(7);
    service.generate(0, &mut source).unwrap();
    let mut sealed = [0; MAX_SEALED];
    let length = service
        .seal(0, 0, b"data", &mut source, &mut sealed)
        .unwrap();
    let mut out = [0x32; MAX_PLAINTEXT];
    assert_eq!(
        service.unseal_record(0, 0, context(4), &sealed[..length], &mut source, &mut out),
        Err(Status::Auth)
    );
    let length = service
        .seal_record(0, 0, context(4), b"data", &mut source, &mut sealed)
        .unwrap();
    assert_eq!(
        service.unseal(0, 0, &sealed[..length], &mut source, &mut out),
        Err(Status::Auth)
    );
    assert_eq!(out, [0x32; MAX_PLAINTEXT]);
}

#[test]
fn record_operations_obey_key_revocation_and_entropy_failure() {
    let (mut service, mut source) = ready(8);
    service.generate(0, &mut source).unwrap();
    let mut out = [0x13; MAX_SEALED];
    // generate consumed operation 1; reach operation 63 before failing reseed.
    for _ in 1..RESEED_INTERVAL - 1 {
        service
            .seal_record(0, 0, context(1), b"x", &mut source, &mut out)
            .unwrap();
    }
    source.fail = true;
    out.fill(0x13);
    let before = service.snapshot_info();
    assert_eq!(
        service.seal_record(0, 0, context(1), b"x", &mut source, &mut out),
        Err(Status::Degraded)
    );
    assert_eq!(out, [0x13; MAX_SEALED]);
    assert_eq!(service.snapshot_info(), before);
    assert_eq!(service.state(), State::Degraded);

    let (mut service, mut source) = ready(9);
    service.generate(0, &mut source).unwrap();
    service.revoke(0, 0).unwrap();
    assert_eq!(
        service.seal_record(0, 0, context(1), b"x", &mut source, &mut out),
        Err(Status::Revoked)
    );
    assert_eq!(out, [0x13; MAX_SEALED]);
    assert_eq!(
        service.seal_record(0, 16, context(1), b"x", &mut source, &mut out),
        Err(Status::BadHandle)
    );
}

#[test]
fn persistent_snapshot_preserves_exact_allocations_revocations_and_record_keys() {
    let (mut original, mut source) = ready(10);
    original.generate(0, &mut source).unwrap();
    original.generate(0, &mut source).unwrap();
    original.generate(1, &mut source).unwrap();
    original.revoke(0, 1).unwrap();
    let mut sealed = [0; MAX_SEALED];
    let length = original
        .seal_record(0, 0, context(4), b"data", &mut source, &mut sealed)
        .unwrap();
    let (blob, info) = export(&mut original, &mut source);
    assert_eq!(
        info,
        SnapshotInfo {
            generation: 4,
            created: [2, 1],
            revoked: [2, 0]
        }
    );
    let (mut restored, mut other) = ready(11);
    assert_eq!(
        restored.restore_persistent(&VOLUME, &SECRET, &blob, 4, &mut other),
        Ok(info)
    );
    assert_eq!(restored.generate(0, &mut other), Ok(2));
    assert_eq!(restored.generate(1, &mut other), Ok(1));
    let mut out = [0; MAX_PLAINTEXT];
    assert_eq!(
        restored.unseal_record(0, 0, context(4), &sealed[..length], &mut other, &mut out),
        Ok(4)
    );
    assert_eq!(&out[..4], b"data");
    assert_eq!(
        restored.seal_record(0, 1, context(1), b"x", &mut other, &mut sealed),
        Err(Status::Revoked)
    );
}

#[test]
fn old_persistent_snapshot_cannot_undo_live_allocations_or_revocations() {
    let (mut service, mut source) = ready(12);
    service.generate(0, &mut source).unwrap();
    let (old, _) = export(&mut service, &mut source);
    service.generate(0, &mut source).unwrap();
    service.revoke(0, 0).unwrap();
    let before = service.snapshot_info();
    assert_eq!(
        service.restore_persistent(&VOLUME, &SECRET, &old, 0, &mut source),
        Ok(before)
    );
    assert_eq!(service.generate(0, &mut source), Ok(2));
    let mut out = [0; MAX_SEALED];
    assert_eq!(
        service.seal_record(0, 0, context(1), b"x", &mut source, &mut out),
        Err(Status::Revoked)
    );
}

#[test]
fn merging_divergent_revocations_creates_a_new_generation_without_resurrection() {
    let (mut left, mut source) = ready(13);
    left.generate(0, &mut source).unwrap();
    left.generate(0, &mut source).unwrap();
    let (initial, _) = export(&mut left, &mut source);
    let (mut right, mut other) = ready(14);
    right
        .restore_persistent(&VOLUME, &SECRET, &initial, 0, &mut other)
        .unwrap();
    left.revoke(0, 0).unwrap();
    right.revoke(0, 1).unwrap();
    let (right_blob, _) = export(&mut right, &mut other);
    let merged = left
        .restore_persistent(&VOLUME, &SECRET, &right_blob, 0, &mut source)
        .unwrap();
    assert_eq!(merged.generation, 4);
    assert_eq!(merged.created, [2, 0]);
    assert_eq!(merged.revoked, [3, 0]);
    let (merged_blob, _) = export(&mut left, &mut source);
    assert_eq!(
        right.restore_persistent(&VOLUME, &SECRET, &merged_blob, 4, &mut other),
        Ok(merged)
    );
    assert_eq!(
        right.restore_persistent(&VOLUME, &SECRET, &initial, 0, &mut other),
        Ok(merged)
    );
}

#[test]
fn importing_an_older_divergent_branch_advances_the_current_generation() {
    let (mut current, mut source) = ready(31);
    current.generate(0, &mut source).unwrap();
    current.generate(0, &mut source).unwrap();
    let (initial, _) = export(&mut current, &mut source);
    let (mut old_branch, mut other) = ready(32);
    old_branch
        .restore_persistent(&VOLUME, &SECRET, &initial, 0, &mut other)
        .unwrap();
    old_branch.revoke(0, 1).unwrap();
    let (older, old_info) = export(&mut old_branch, &mut other);
    assert_eq!(old_info.generation, 3);
    current.generate(1, &mut source).unwrap();
    current.generate(1, &mut source).unwrap();
    current.revoke(0, 0).unwrap();
    assert_eq!(current.key_generation(), 5);
    let merged = current
        .restore_persistent(&VOLUME, &SECRET, &older, 0, &mut source)
        .unwrap();
    assert_eq!(merged.generation, 6);
    assert_eq!(merged.created, [2, 2]);
    assert_eq!(merged.revoked, [3, 0]);
}

#[test]
fn fully_allocated_and_revoked_domain_survives_persistent_restore() {
    let (mut original, mut source) = ready(33);
    for index in 0..KEYS_PER_DOMAIN {
        assert_eq!(original.generate(1, &mut source), Ok(index as u8));
        original.revoke(1, index as u8).unwrap();
    }
    let (blob, info) = export(&mut original, &mut source);
    assert_eq!(info.created, [0, 16]);
    assert_eq!(info.revoked, [0, u16::MAX]);
    let (mut restored, mut other) = ready(34);
    assert_eq!(
        restored.restore_persistent(&VOLUME, &SECRET, &blob, 0, &mut other),
        Ok(info)
    );
    assert_eq!(restored.generate(1, &mut other), Err(Status::Quota));
    assert_eq!(restored.generate(0, &mut other), Ok(0));
}

#[test]
fn persistence_requires_the_external_generation_floor_to_reject_cold_replay() {
    let (mut service, mut source) = ready(15);
    service.generate(0, &mut source).unwrap();
    let (old, info) = export(&mut service, &mut source);
    service.revoke(0, 0).unwrap();
    let (mut cold, mut other) = ready(16);
    let before = cold.snapshot_info();
    assert_eq!(
        cold.restore_persistent(&VOLUME, &SECRET, &old, 2, &mut other),
        Err(Status::Auth)
    );
    assert_eq!(cold.snapshot_info(), before);
    assert_eq!(
        cold.restore_persistent(&VOLUME, &SECRET, &old, 1, &mut other),
        Ok(info)
    );
    let (mut unanchored, mut another) = ready(17);
    assert_eq!(
        unanchored.restore_persistent(&VOLUME, &SECRET, &old, 0, &mut another),
        Ok(info),
        "without a trusted floor, old authentic state is indistinguishable on cold start"
    );
}

#[test]
fn persistent_nonce_is_fresh_and_allocation_metadata_is_encrypted() {
    let (mut service, mut source) = ready(18);
    service.generate(0, &mut source).unwrap();
    let (first, info) = export(&mut service, &mut source);
    let (second, again) = export(&mut service, &mut source);
    assert_eq!(info, again);
    assert_eq!(first[..PERSISTENT_HEADER], second[..PERSISTENT_HEADER]);
    assert_ne!(first[PERSISTENT_HEADER..], second[PERSISTENT_HEADER..]);
    assert_eq!(&first[..8], b"NOSKEY01");
    // Only format/suite/volume are outside the AEAD body. Allocations,
    // revocations, generation and the master are all inside that body.
    assert_eq!(PERSISTENT_PLAIN, KEY_BYTES + 8 + DOMAINS * 4);
}

#[test]
fn every_persistent_bitflip_and_wrong_secret_preserve_the_namespace() {
    let (mut original, mut source) = ready(19);
    original.generate(1, &mut source).unwrap();
    let (blob, _) = export(&mut original, &mut source);
    let (mut recipient, mut other) = ready(20);
    let before = recipient.snapshot_info();
    assert_eq!(
        recipient.restore_persistent(&VOLUME, &[0x36; SECRET_BYTES], &blob, 0, &mut other),
        Err(Status::Auth)
    );
    for bit in 0..blob.len() * 8 {
        let mut changed = blob;
        changed[bit / 8] ^= 1 << (bit % 8);
        assert_eq!(
            recipient.restore_persistent(&VOLUME, &SECRET, &changed, 0, &mut other),
            Err(Status::Auth),
            "bit {bit}"
        );
        assert_eq!(recipient.snapshot_info(), before);
    }
    // Failed authentication did not bind or consume allocation authority.
    assert_eq!(recipient.generate(0, &mut other), Ok(0));
}

#[test]
fn persistent_lengths_and_volume_are_exactly_validated() {
    let (mut original, mut source) = ready(21);
    let (blob, _) = export(&mut original, &mut source);
    let (mut recipient, mut other) = ready(22);
    let before = recipient.snapshot_info();
    for length in 0..PERSISTENT_BLOB {
        assert_eq!(
            recipient.restore_persistent(&VOLUME, &SECRET, &blob[..length], 0, &mut other),
            Err(Status::Invalid)
        );
    }
    let mut extra = blob.to_vec();
    extra.push(0);
    assert_eq!(
        recipient.restore_persistent(&VOLUME, &SECRET, &extra, 0, &mut other),
        Err(Status::Invalid)
    );
    assert_eq!(
        recipient.restore_persistent(&[9; 16], &SECRET, &blob, 0, &mut other),
        Err(Status::Auth)
    );
    assert_eq!(recipient.snapshot_info(), before);
}

#[test]
fn persistent_empty_snapshot_still_binds_volume_and_master_lineage() {
    let (mut original, mut source) = ready(23);
    let (blob, info) = export(&mut original, &mut source);
    let (mut recipient, mut other) = ready(24);
    recipient
        .restore_persistent(&VOLUME, &SECRET, &blob, 0, &mut other)
        .unwrap();
    let (mut foreign, mut third) = ready(25);
    let (foreign_blob, _) = export(&mut foreign, &mut third);
    assert_eq!(
        recipient.restore_persistent(&VOLUME, &SECRET, &foreign_blob, 0, &mut other),
        Err(Status::Invalid)
    );
    let mut out = [0x65; PERSISTENT_BLOB];
    assert_eq!(
        recipient.export_persistent(&[0x32; 16], &SECRET, &mut other, &mut out),
        Err(Status::Invalid)
    );
    assert_eq!(out, [0x65; PERSISTENT_BLOB]);
    assert_eq!(recipient.snapshot_info(), info);
    // The old recovery operation cannot bypass the new lineage binding.
    let mut recovery_secret = [0; SECRET_BYTES];
    let mut recovery_blob = [0; keyring_core::RECOVERY_BLOB];
    foreign
        .export_recovery(&mut third, &mut recovery_secret, &mut recovery_blob)
        .unwrap();
    assert_eq!(
        recipient.restore(&recovery_secret, &recovery_blob, &mut other),
        Err(Status::Invalid)
    );
    assert_eq!(recipient.snapshot_info(), info);
}

/// Encode an authenticated fixture under the caller's TEST secret so schema
/// validation is exercised separately from AEAD integrity rejection.
fn authenticated_fixture(
    blob: &[u8; PERSISTENT_BLOB],
    change: impl FnOnce(&mut [u8]),
) -> [u8; PERSISTENT_BLOB] {
    let hk = Hkdf::<Sha256>::new(Some(b"nieuw-os persistent key-state v1"), &SECRET);
    let mut wrap = [0u8; KEY_BYTES];
    hk.expand(b"wrap", &mut wrap).unwrap();
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&wrap));
    let body = PERSISTENT_HEADER + NONCE_BYTES;
    let end = body + PERSISTENT_PLAIN;
    let mut plain = blob[body..end].to_vec();
    cipher
        .decrypt_in_place_detached(
            XNonce::from_slice(&blob[PERSISTENT_HEADER..body]),
            &blob[..PERSISTENT_HEADER],
            &mut plain,
            chacha20poly1305::Tag::from_slice(&blob[end..]),
        )
        .unwrap();
    change(&mut plain);
    let tag = cipher
        .encrypt_in_place_detached(
            XNonce::from_slice(&blob[PERSISTENT_HEADER..body]),
            &blob[..PERSISTENT_HEADER],
            &mut plain,
        )
        .unwrap();
    let mut changed = *blob;
    changed[body..end].copy_from_slice(&plain);
    changed[end..].copy_from_slice(&tag);
    changed
}

#[test]
fn authenticated_invalid_allocations_and_unallocated_revocations_fail_atomically() {
    let (mut original, mut source) = ready(26);
    let (blob, _) = export(&mut original, &mut source);
    let (mut recipient, mut other) = ready(27);
    let before = recipient.snapshot_info();
    for domain in 0..DOMAINS {
        for created in [KEYS_PER_DOMAIN as u16 + 1, u16::MAX] {
            let changed = authenticated_fixture(&blob, |plain| {
                let offset = KEY_BYTES + 8 + domain * 4;
                plain[offset..offset + 2].copy_from_slice(&created.to_le_bytes());
            });
            assert_eq!(
                recipient.restore_persistent(&VOLUME, &SECRET, &changed, 0, &mut other),
                Err(Status::Invalid)
            );
            assert_eq!(recipient.snapshot_info(), before);
        }
        let changed = authenticated_fixture(&blob, |plain| {
            let offset = KEY_BYTES + 8 + domain * 4 + 2;
            plain[offset..offset + 2].copy_from_slice(&1u16.to_le_bytes());
        });
        assert_eq!(
            recipient.restore_persistent(&VOLUME, &SECRET, &changed, 0, &mut other),
            Err(Status::Invalid)
        );
        assert_eq!(recipient.snapshot_info(), before);
    }
}

#[test]
fn generation_overflow_never_partially_allocates_or_revokes() {
    let (mut original, mut source) = ready(28);
    original.generate(0, &mut source).unwrap();
    let (blob, _) = export(&mut original, &mut source);
    let max = authenticated_fixture(&blob, |plain| {
        plain[KEY_BYTES..KEY_BYTES + 8].copy_from_slice(&u64::MAX.to_le_bytes())
    });
    let (mut recipient, mut other) = ready(29);
    recipient
        .restore_persistent(&VOLUME, &SECRET, &max, 0, &mut other)
        .unwrap();
    let before = recipient.snapshot_info();
    assert_eq!(recipient.generate(0, &mut other), Err(Status::Quota));
    assert_eq!(recipient.revoke(0, 0), Err(Status::Quota));
    assert_eq!(recipient.snapshot_info(), before);
}

#[test]
fn union_generation_overflow_does_not_commit_half_of_an_import() {
    let (mut original, mut source) = ready(35);
    original.generate(0, &mut source).unwrap();
    original.generate(0, &mut source).unwrap();
    let (blob, _) = export(&mut original, &mut source);
    let fixture = |revoked: u16| {
        authenticated_fixture(&blob, |plain| {
            plain[KEY_BYTES..KEY_BYTES + 8].copy_from_slice(&u64::MAX.to_le_bytes());
            plain[KEY_BYTES + 10..KEY_BYTES + 12].copy_from_slice(&revoked.to_le_bytes());
        })
    };
    let (mut recipient, mut other) = ready(36);
    recipient
        .restore_persistent(&VOLUME, &SECRET, &fixture(1), 0, &mut other)
        .unwrap();
    let before = recipient.snapshot_info();
    assert_eq!(
        recipient.restore_persistent(&VOLUME, &SECRET, &fixture(2), 0, &mut other),
        Err(Status::Quota)
    );
    assert_eq!(recipient.snapshot_info(), before);
}

#[test]
fn persistent_calls_do_not_bypass_service_readiness_or_mutate_failed_output() {
    let (mut original, mut source) = ready(30);
    let (blob, _) = export(&mut original, &mut source);
    let mut service = Service::new();
    let mut out = [0x72; PERSISTENT_BLOB];
    assert_eq!(
        service.export_persistent(&VOLUME, &SECRET, &mut source, &mut out),
        Err(Status::NotReady)
    );
    assert_eq!(out, [0x72; PERSISTENT_BLOB]);
    assert_eq!(
        service.restore_persistent(&VOLUME, &SECRET, &blob, 0, &mut source),
        Err(Status::NotReady)
    );
    assert_eq!(service.state(), State::Starting);
    assert_eq!(service.snapshot_info().created, [0; DOMAINS]);
}

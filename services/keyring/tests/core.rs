//! Host tests for keyring-core: standard test vectors for the pinned
//! primitives, the SP 800-90B health tests, and the service lifecycle with a
//! deterministic fake entropy source. A host test proves the model, not the
//! native run.
use chacha20poly1305::aead::{AeadInPlace, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use hex_literal::hex;
use hkdf::Hkdf;
use keyring_core::health::{Health, HealthFailure, APT_CUTOFF, APT_WINDOW, RCT_CUTOFF};
use keyring_core::wire::{self, Header};
use keyring_core::{
    Drbg, EntropySource, Service, State, Status, KEYS_PER_DOMAIN, MAX_PLAINTEXT, MAX_SEALED,
    RECOVERY_BLOB, RESEED_ENTROPY, RESEED_INTERVAL, SECRET_BYTES, STARTUP_ENTROPY,
};
use sha2::Sha256;

/// Deterministic, non-repeating byte stream (a counter through a small
/// mixer). This is a TEST source: it exercises code paths, it is not entropy.
struct Fake {
    state: u64,
    fail_after: Option<u64>,
    reads: u64,
    constant: Option<u8>,
}
impl Fake {
    fn new() -> Self {
        Self {
            state: 0x9e37_79b9_7f4a_7c15,
            fail_after: None,
            reads: 0,
            constant: None,
        }
    }
}
impl EntropySource for Fake {
    fn read(&mut self, out: &mut [u8]) -> Result<(), Status> {
        if self.fail_after.is_some_and(|n| self.reads >= n) {
            return Err(Status::Entropy);
        }
        self.reads += 1;
        for byte in out.iter_mut() {
            if let Some(c) = self.constant {
                *byte = c;
                continue;
            }
            self.state ^= self.state << 13;
            self.state ^= self.state >> 7;
            self.state ^= self.state << 17;
            *byte = (self.state >> 24) as u8;
        }
        Ok(())
    }
}

fn ready() -> (Service, Fake) {
    let mut source = Fake::new();
    let mut service = Service::new();
    service.start(&mut source).unwrap();
    assert_eq!(service.state(), State::Ready);
    (service, source)
}

/// Raw test bytes spanning multiple broker requests, including request borders.
struct Scripted {
    bytes: std::vec::Vec<u8>,
    offset: usize,
}
impl EntropySource for Scripted {
    fn read(&mut self, out: &mut [u8]) -> Result<(), Status> {
        let end = self.offset + out.len();
        let bytes = self.bytes.get(self.offset..end).ok_or(Status::Entropy)?;
        out.copy_from_slice(bytes);
        self.offset = end;
        Ok(())
    }
}

fn healthy_test_bytes(length: usize) -> std::vec::Vec<u8> {
    (0..length).map(|i| i as u8).collect()
}

fn reach_reseed(service: &mut Service, source: &mut impl EntropySource) -> Result<u8, Status> {
    // Even quota-denied calls count against the entropy budget. This reaches
    // one reseed without depending on key allocation beyond the first 16.
    let mut result = Err(Status::Invalid);
    for _ in 0..RESEED_INTERVAL {
        result = service.generate(0, source);
    }
    result
}

#[test]
fn hkdf_sha256_rfc5869_test_case_1() {
    let ikm = hex!("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b");
    let salt = hex!("000102030405060708090a0b0c");
    let info = hex!("f0f1f2f3f4f5f6f7f8f9");
    let hk = Hkdf::<Sha256>::new(Some(&salt), &ikm);
    let mut okm = [0u8; 42];
    hk.expand(&info, &mut okm).unwrap();
    assert_eq!(
        okm,
        hex!(
            "3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865"
        )
    );
}

#[test]
fn xchacha20poly1305_draft_irtf_cfrg_xchacha_03_a_3_1() {
    let key = hex!("808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f");
    let nonce = hex!("404142434445464748494a4b4c4d4e4f5051525354555657");
    let aad = hex!("50515253c0c1c2c3c4c5c6c7");
    let plaintext = b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.";
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
    let mut out = [0u8; 114];
    out.copy_from_slice(plaintext);
    let tag = cipher
        .encrypt_in_place_detached(XNonce::from_slice(&nonce), &aad, &mut out)
        .unwrap();
    assert_eq!(
        out[..],
        hex!(
            "bd6d179d3e83d43b9576579493c0e939572a1700252bfaccbed2902c21396cbb"
            "731c7f1b0b4aa6440bf3a82f4eda7e39ae64c6708c54c216cb96b72e1213b452"
            "2f8c9ba40db5d945b11b69b982c1bb9e3f3fac2bc369488f76b2383565d3fff9"
            "21f9664c97637da9768812f615c68b13b52e"
        )[..]
    );
    assert_eq!(tag[..], hex!("c0875924c1c7987947deafd8780acf49")[..]);
}

#[test]
fn drbg_is_deterministic_per_seed_erases_keys_and_never_repeats_blocks() {
    let seed = [7u8; STARTUP_ENTROPY];
    let mut a = Drbg::from_entropy(&seed);
    let mut b = Drbg::from_entropy(&seed);
    let mut x = [0u8; 64];
    let mut y = [0u8; 64];
    a.fill(&mut x);
    b.fill(&mut y);
    assert_eq!(x, y);
    let mut z = [0u8; 64];
    a.fill(&mut z);
    assert_ne!(x, z, "fast key erasure must change the key between calls");
    assert_eq!(a.generated(), 2);
    let mut c = Drbg::from_entropy(&[8u8; STARTUP_ENTROPY]);
    let mut w = [0u8; 64];
    c.fill(&mut w);
    assert_ne!(x, w);
    b.reseed(&[1u8; 256]);
    let mut v = [0u8; 64];
    b.fill(&mut v);
    assert_ne!(v, z);
}

#[test]
fn health_tests_pass_varied_input_and_catch_stuck_or_biased_sources() {
    let mut health = Health::new();
    let mut fake = Fake::new();
    let mut raw = [0u8; 4096];
    fake.read(&mut raw).unwrap();
    assert_eq!(health.feed(&raw), Ok(()));
    assert_eq!(health.samples, 4096);
    // Exactly RCT_CUTOFF identical bytes fail; one fewer passes.
    let mut h = Health::new();
    assert_eq!(h.feed(&[0x5a; (RCT_CUTOFF - 1) as usize]), Ok(()));
    assert_eq!(
        Health::new().feed(&[0x5a; RCT_CUTOFF as usize]),
        Err(HealthFailure::Repetition)
    );
    assert_eq!(
        Health::new().feed(&[0x5a; 256]),
        Err(HealthFailure::Repetition)
    );
    // Adaptive proportion: the window's first value appearing APT_CUTOFF times.
    let mut biased = [0u8; APT_WINDOW as usize];
    for (i, byte) in biased.iter_mut().enumerate() {
        *byte = if i % 8 == 0 {
            0x11
        } else {
            (i % 251) as u8 + 1
        };
    }
    // 512 / 8 = 64 occurrences of 0x11 >= 62, with no run of 6.
    assert_eq!(Health::new().feed(&biased), Err(HealthFailure::Proportion));
    let mut fine = biased;
    for (i, byte) in fine.iter_mut().enumerate() {
        if i % 8 == 0 && i >= 8 * (APT_CUTOFF as usize - 2) {
            *byte = 0x22;
        }
    }
    assert_eq!(Health::new().feed(&fine), Ok(()));
}

#[test]
fn service_blocks_without_source_or_with_stuck_source_and_never_makes_keys() {
    let mut absent = Fake::new();
    absent.fail_after = Some(0);
    let mut service = Service::new();
    assert_eq!(service.start(&mut absent), Err(Status::Entropy));
    assert_eq!(service.state(), State::Blocked);
    assert_eq!(service.reason(), HealthFailure::Source);
    assert_eq!(service.generate(0, &mut absent), Err(Status::NotReady));
    assert_eq!(service.start(&mut absent), Err(Status::Invalid));

    let mut stuck = Fake::new();
    stuck.constant = Some(0x5a);
    let mut service = Service::new();
    assert_eq!(service.start(&mut stuck), Err(Status::Entropy));
    assert_eq!(service.state(), State::Blocked);
    assert_eq!(service.reason(), HealthFailure::Repetition);
    assert_eq!(service.generate(0, &mut stuck), Err(Status::NotReady));
    assert_eq!(service.keys_created(0), 0);
    let mut out = [0u8; MAX_SEALED];
    assert_eq!(
        service.seal(0, 0, b"x", &mut stuck, &mut out),
        Err(Status::NotReady)
    );
}

#[test]
fn seal_unseal_round_trip_binds_domain_index_and_rejects_tampering() {
    let (mut service, mut source) = ready();
    let k0 = service.generate(0, &mut source).unwrap();
    let k1 = service.generate(0, &mut source).unwrap();
    assert_eq!((k0, k1), (0, 1));
    let mut sealed = [0u8; MAX_SEALED];
    let len = service
        .seal(0, k0, b"hello keyring", &mut source, &mut sealed)
        .unwrap();
    assert_eq!(len, 24 + 13 + 16);
    let mut plain = [0u8; MAX_PLAINTEXT];
    let n = service
        .unseal(0, k0, &sealed[..len], &mut source, &mut plain)
        .unwrap();
    assert_eq!(&plain[..n], b"hello keyring");
    // Wrong key of the same domain, other domain, and tampering all fail.
    assert_eq!(
        service.unseal(0, k1, &sealed[..len], &mut source, &mut plain),
        Err(Status::Auth)
    );
    let other = service.generate(1, &mut source).unwrap();
    assert_eq!(
        service.unseal(1, other, &sealed[..len], &mut source, &mut plain),
        Err(Status::Auth)
    );
    for position in [0usize, 30, len - 1] {
        let mut bad = sealed;
        bad[position] ^= 0x01;
        let mut out = [0u8; MAX_PLAINTEXT];
        assert_eq!(
            service.unseal(0, k0, &bad[..len], &mut source, &mut out),
            Err(Status::Auth)
        );
        assert!(out.iter().all(|b| *b == 0), "no partial plaintext");
    }
    assert_eq!(
        service.unseal(0, k0, &sealed[..len - 1], &mut source, &mut plain),
        Err(Status::Auth)
    );
    assert_eq!(
        service.unseal(0, k0, &sealed[..39], &mut source, &mut plain),
        Err(Status::Invalid)
    );
    assert_eq!(service.counters.auth_failures, 6);
    // Handles are per domain and bounded.
    assert_eq!(
        service.unseal(0, 7, &sealed[..len], &mut source, &mut plain),
        Err(Status::BadHandle)
    );
    assert_eq!(service.generate(2, &mut source), Err(Status::Invalid));
    let mut big = [0u8; MAX_SEALED];
    assert_eq!(
        service.seal(0, k0, &[0u8; MAX_PLAINTEXT + 1], &mut source, &mut big),
        Err(Status::Invalid)
    );
}

#[test]
fn revoke_and_quota_are_permanent_per_domain() {
    let (mut service, mut source) = ready();
    let k = service.generate(0, &mut source).unwrap();
    assert_eq!(service.revoke(0, k), Ok(()));
    assert_eq!(service.revoke(0, k), Err(Status::Revoked));
    assert_eq!(service.revoke(0, 5), Err(Status::BadHandle));
    let mut out = [0u8; MAX_SEALED];
    assert_eq!(
        service.seal(0, k, b"x", &mut source, &mut out),
        Err(Status::Revoked)
    );
    for _ in 1..KEYS_PER_DOMAIN {
        service.generate(0, &mut source).unwrap();
    }
    assert_eq!(service.generate(0, &mut source), Err(Status::Quota));
    assert_eq!(service.keys_created(0), KEYS_PER_DOMAIN as u16);
    assert_eq!(
        service.generate(1, &mut source),
        Ok(0),
        "domains are independent"
    );
}

#[test]
fn recovery_restores_keys_and_revocations_into_another_instance() {
    let (mut a, mut source) = ready();
    let k0 = a.generate(0, &mut source).unwrap();
    let k1 = a.generate(0, &mut source).unwrap();
    a.revoke(0, k1).unwrap();
    let mut sealed = [0u8; MAX_SEALED];
    let len = a
        .seal(0, k0, b"survives", &mut source, &mut sealed)
        .unwrap();
    let mut secret = [0u8; SECRET_BYTES];
    let mut blob = [0u8; RECOVERY_BLOB];
    a.export_recovery(&mut source, &mut secret, &mut blob)
        .unwrap();

    let mut other = Fake::new();
    other.state ^= 0xdead_beef;
    let mut b = Service::new();
    b.start(&mut other).unwrap();
    let mut plain = [0u8; MAX_PLAINTEXT];
    assert_eq!(
        b.unseal(0, k0, &sealed[..len], &mut other, &mut plain),
        Err(Status::BadHandle),
        "a fresh instance has no keys to try"
    );
    let mut wrong = secret;
    wrong[3] ^= 0x80;
    assert_eq!(b.restore(&wrong, &blob, &mut other), Err(Status::Auth));
    let mut bad_blob = blob;
    bad_blob[40] ^= 1;
    assert_eq!(b.restore(&secret, &bad_blob, &mut other), Err(Status::Auth));
    assert_eq!(
        b.unseal(0, k0, &sealed[..len], &mut other, &mut plain),
        Err(Status::BadHandle)
    );
    assert_eq!(b.restore(&secret, &blob, &mut other), Ok(2));
    let n = b
        .unseal(0, k0, &sealed[..len], &mut other, &mut plain)
        .unwrap();
    assert_eq!(&plain[..n], b"survives");
    let mut out = [0u8; MAX_SEALED];
    assert_eq!(
        b.seal(0, k1, b"x", &mut other, &mut out),
        Err(Status::Revoked),
        "revocation state travels with the recovery blob"
    );
    assert_eq!(b.counters.restores, 1);
}

#[test]
fn older_recovery_preserves_revocations_and_cannot_reissue_indices() {
    let (mut service, mut source) = ready();
    let k0 = service.generate(0, &mut source).unwrap();
    let k1 = service.generate(0, &mut source).unwrap();
    let mut sealed = [0u8; MAX_SEALED];
    let length = service
        .seal(0, k0, b"revoked after export", &mut source, &mut sealed)
        .unwrap();
    service.revoke(0, k1).unwrap();
    let mut secret = [0u8; SECRET_BYTES];
    let mut blob = [0u8; RECOVERY_BLOB];
    service
        .export_recovery(&mut source, &mut secret, &mut blob)
        .unwrap();
    service.revoke(0, k0).unwrap();
    let issued_after_export = service.generate(0, &mut source).unwrap();
    assert_eq!(issued_after_export, 2);

    assert_eq!(service.restore(&secret, &blob, &mut source), Ok(2));
    let mut plain = [0u8; MAX_PLAINTEXT];
    assert_eq!(
        service.unseal(0, k0, &sealed[..length], &mut source, &mut plain),
        Err(Status::Revoked),
        "the authenticated older blob must not undo a known revocation"
    );
    assert_eq!(
        service.seal(0, k1, b"x", &mut source, &mut sealed),
        Err(Status::Revoked),
        "the recovered revocation must remain set too"
    );
    assert_eq!(service.generate(0, &mut source), Err(Status::Quota));
    assert_eq!(service.keys_created(0), KEYS_PER_DOMAIN as u16);

    // Re-exporting the restored state must carry the union to a fresh service.
    service
        .export_recovery(&mut source, &mut secret, &mut blob)
        .unwrap();
    let (mut fresh, mut other) = ready();
    fresh.restore(&secret, &blob, &mut other).unwrap();
    assert_eq!(
        fresh.seal(0, k0, b"x", &mut other, &mut sealed),
        Err(Status::Revoked)
    );
    assert_eq!(
        fresh.seal(0, k1, b"x", &mut other, &mut sealed),
        Err(Status::Revoked)
    );
}

#[test]
fn used_service_rejects_another_master_without_losing_revocation_history() {
    let (mut a, mut source) = ready();
    let key = a.generate(0, &mut source).unwrap();
    let mut secret_a = [0u8; SECRET_BYTES];
    let mut blob_a = [0u8; RECOVERY_BLOB];
    a.export_recovery(&mut source, &mut secret_a, &mut blob_a)
        .unwrap();

    let mut different = Fake::new();
    different.state ^= 0xdead_beef;
    let mut c = Service::new();
    c.start(&mut different).unwrap();
    let mut secret_c = [0u8; SECRET_BYTES];
    let mut blob_c = [0u8; RECOVERY_BLOB];
    c.export_recovery(&mut different, &mut secret_c, &mut blob_c)
        .unwrap();

    let mut out = [0u8; MAX_SEALED];
    assert_eq!(
        a.restore(&secret_c, &blob_c, &mut source),
        Err(Status::Invalid),
        "issuing a key binds the service to its master lineage"
    );
    a.revoke(0, key).unwrap();
    assert_eq!(a.restore(&secret_a, &blob_a, &mut source), Ok(2));
    assert_eq!(
        a.seal(0, key, b"x", &mut source, &mut out),
        Err(Status::Revoked)
    );

    // A fresh service can import A, but cannot then switch to C and back to
    // A to erase an in-memory revocation that its old blob does not contain.
    let (mut b, mut other) = ready();
    b.restore(&secret_a, &blob_a, &mut other).unwrap();
    b.revoke(0, key).unwrap();
    assert_eq!(
        b.restore(&secret_c, &blob_c, &mut other),
        Err(Status::Invalid)
    );
    assert_eq!(
        b.counters.restores, 1,
        "rejected imports do not mutate lineage"
    );
    b.restore(&secret_a, &blob_a, &mut other).unwrap();
    assert_eq!(
        b.seal(0, key, b"x", &mut other, &mut out),
        Err(Status::Revoked)
    );
}

#[test]
fn continuous_apt_spans_two_reseeds_and_blocks_further_key_operations() {
    let mut bytes = healthy_test_bytes(STARTUP_ENTROPY);
    for _ in 0..RESEED_ENTROPY / 8 * 2 {
        bytes.extend_from_slice(&[17, 1, 2, 3, 4, 5, 6, 7]);
    }
    let mut source = Scripted { bytes, offset: 0 };
    let mut service = Service::new();
    service.start(&mut source).unwrap();
    reach_reseed(&mut service, &mut source).ok();
    assert_eq!(service.state(), State::Ready, "32 occurrences are below 62");
    assert_eq!(service.counters.reseeds, 1);
    assert_eq!(
        reach_reseed(&mut service, &mut source),
        Err(Status::Degraded)
    );
    assert_eq!(service.reason(), HealthFailure::Proportion);
    assert_eq!(
        service.counters.reseeds, 1,
        "failing bytes never reseed the DRBG"
    );
    assert_eq!(service.generate(1, &mut source), Err(Status::Degraded));
}

#[test]
fn continuous_rct_spans_startup_and_first_reseed() {
    let mut bytes = healthy_test_bytes(STARTUP_ENTROPY + RESEED_ENTROPY);
    bytes[STARTUP_ENTROPY - 3..STARTUP_ENTROPY + 3].fill(17);
    let mut source = Scripted { bytes, offset: 0 };
    let mut service = Service::new();
    service.start(&mut source).unwrap();
    assert_eq!(
        reach_reseed(&mut service, &mut source),
        Err(Status::Degraded)
    );
    assert_eq!(service.reason(), HealthFailure::Repetition);
    assert_eq!(service.counters.reseeds, 0);
}

#[test]
fn continuous_rct_spans_consecutive_reseeds() {
    let border = STARTUP_ENTROPY + RESEED_ENTROPY;
    let mut bytes = healthy_test_bytes(border + RESEED_ENTROPY);
    bytes[border - 3..border + 3].fill(17);
    let mut source = Scripted { bytes, offset: 0 };
    let mut service = Service::new();
    service.start(&mut source).unwrap();
    reach_reseed(&mut service, &mut source).ok();
    assert_eq!(service.state(), State::Ready);
    assert_eq!(
        reach_reseed(&mut service, &mut source),
        Err(Status::Degraded)
    );
    assert_eq!(service.reason(), HealthFailure::Repetition);
    assert_eq!(service.counters.reseeds, 1);
}

#[test]
fn reseed_after_budget_and_degradation_when_the_source_fails_later() {
    let (mut service, mut source) = ready();
    for _ in 0..RESEED_INTERVAL - 1 {
        service.generate(1, &mut source).ok();
    }
    assert_eq!(service.counters.reseeds, 0);
    let _ = service.generate(1, &mut source);
    assert_eq!(service.counters.reseeds, 1);
    assert_eq!(service.state(), State::Ready);
    source.fail_after = Some(source.reads);
    for _ in 0..RESEED_INTERVAL {
        let _ = service.revoke(1, 0);
        let _ = service.generate(0, &mut source);
    }
    assert_eq!(service.state(), State::Degraded);
    assert_eq!(service.reason(), HealthFailure::Source);
    assert_eq!(service.generate(0, &mut source), Err(Status::Degraded));
}

#[test]
fn wire_headers_are_canonical_and_bounded() {
    let body = [9u8; 5];
    let bytes = wire::request(wire::SEAL, 3, 77, &body).unwrap();
    let header = Header::decode(&bytes).unwrap();
    assert_eq!(
        (
            header.op,
            header.index,
            header.length,
            header.request,
            header.status
        ),
        (wire::SEAL, 3, 5, 77, 0)
    );
    assert_eq!(Header::decode(&bytes[..255]), None);
    let mut trailing = bytes;
    trailing[200] = 1;
    assert_eq!(Header::decode(&trailing), None);
    let mut long = bytes;
    long[2..4].copy_from_slice(&(wire::BODY as u16 + 1).to_le_bytes());
    assert_eq!(Header::decode(&long), None);
    assert!(wire::request(wire::SEAL, 0, 1, &[0u8; wire::BODY + 1]).is_none());
    let reply = wire::reply(&header, Status::Auth, &[1, 2, 3]);
    let decoded = Header::decode(&reply).unwrap();
    assert_eq!(decoded.op, wire::SEAL | wire::REPLY);
    assert_eq!(decoded.status, Status::Auth.code());
    assert_eq!(decoded.length, 0, "error replies carry no body");
    assert_eq!(wire::body_bounds(wire::RESTORE), Some((108, 108)));
    assert_eq!(wire::body_bounds(wire::UNSEAL), Some((40, MAX_SEALED)));
    assert_eq!(wire::body_bounds(0x42), None);
    assert_eq!(Status::from_code(-5), Status::Auth);
    assert_eq!(Status::from_code(99), Status::Invalid);
}

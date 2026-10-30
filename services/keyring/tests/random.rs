//! Algorithm/IPC policy models with a deterministic TEST source, never proof
//! of native entropy quality, kernel delivery, TLS or physical randomness.
use chacha20_rng::ChaCha20Rng;
use keyring_core::{
    random,
    wire::{self, Header},
    EntropySource, Service, State, Status, MAX_PLAINTEXT, MAX_SEALED, RECOVERY_BLOB, SECRET_BYTES,
};
use rand_core_rng::{Rng, SeedableRng};

struct Source {
    value: u64,
    reads: usize,
    fail: bool,
    stuck: bool,
}
impl Source {
    fn new() -> Self {
        Self {
            value: 0x9e37_79b9_7f4a_7c15,
            reads: 0,
            fail: false,
            stuck: false,
        }
    }
}
impl EntropySource for Source {
    fn read(&mut self, out: &mut [u8]) -> Result<(), Status> {
        self.reads += 1;
        if self.fail {
            return Err(Status::Entropy);
        }
        for byte in out {
            self.value ^= self.value << 13;
            self.value ^= self.value >> 7;
            self.value ^= self.value << 17;
            *byte = if self.stuck {
                7
            } else {
                (self.value >> 24) as u8
            };
        }
        Ok(())
    }
}
fn ready() -> (Service, Source) {
    let mut source = Source::new();
    let mut service = Service::new();
    service.start(&mut source).unwrap();
    (service, source)
}

#[test]
fn maintained_rng_matches_published_chacha20_zero_key_stream() {
    // eSTREAM/RFC 7539 all-zero key, nonce and initial counter test vector.
    // This verifies the selected maintained RNG API/byte order, not entropy.
    let mut rng = ChaCha20Rng::from_seed([0; 32]);
    let mut bytes = [0; 64];
    rng.fill_bytes(&mut bytes);
    assert_eq!(
        bytes,
        hex_literal::hex!(
            "76b8e0ada0f13d90405d6ae55386bd28bdd219b8a08ded1aa836efcc8b770dc7
         da41597c5157488d7724e03fb8d84a376a43b8f41518a11cc387b669b2ee6586"
        )
    );
}

#[test]
fn identical_healthy_source_bytes_are_separated_by_principal() {
    let (mut a, mut source_a) = ready();
    let (mut b, mut source_b) = ready();
    let mut out_a = [0; random::MAX_BYTES];
    let mut out_b = out_a;
    a.random_at(0, 1, 128, 0, &mut source_a, &mut out_a)
        .unwrap();
    b.random_at(1, 1, 128, 0, &mut source_b, &mut out_b)
        .unwrap();
    assert_eq!(source_a.value, source_b.value); // identical raw bytes consumed
    assert_ne!(out_a[..128], out_b[..128]);
}

#[test]
fn readiness_has_no_fallback_and_never_modifies_failed_output() {
    let mut service = Service::new();
    let mut source = Source::new();
    let mut out = [0xa5; random::MAX_BYTES];
    assert_eq!(
        service.random_at(0, 1, 32, 0, &mut source, &mut out),
        Err(Status::NotReady)
    );
    assert_eq!(source.reads, 0);
    source.fail = true;
    assert_eq!(service.start(&mut source), Err(Status::Entropy));
    assert_eq!(
        service.random_at(0, 1, 32, 0, &mut source, &mut out),
        Err(Status::NotReady)
    );
    assert_eq!(out, [0xa5; random::MAX_BYTES]);
    assert_eq!(service.random_counters(), random::Counters::default());
}

#[test]
fn all_lengths_preserve_tails_and_domains_and_requests_are_distinct() {
    let (mut service, mut source) = ready();
    let mut previous = [0u8; random::MAX_BYTES];
    for length in 1..=random::MAX_BYTES {
        let mut out = [0xa5; random::MAX_BYTES];
        let request = length as u64;
        assert_eq!(
            service.random_at(0, request, length, request * 100, &mut source, &mut out),
            Ok(length)
        );
        assert_eq!(&out[length..], &[0xa5; random::MAX_BYTES][length..]);
        assert_ne!(out[..length], previous[..length]);
        previous = out;
    }
    let mut domain1 = [0; random::MAX_BYTES];
    service
        .random_at(1, 1, 64, 22_500, &mut source, &mut domain1)
        .unwrap();
    assert_ne!(previous[..64], domain1[..64]);
    assert_eq!(service.keys_created(0), 0);
    assert_eq!(service.keys_created(1), 0);
}

#[test]
fn bad_domains_lengths_replays_and_clocks_do_not_advance_a_generator() {
    let (mut service, mut source) = ready();
    let (mut control, mut control_source) = ready();
    let (mut a, mut b) = ([0; random::MAX_BYTES], [0; random::MAX_BYTES]);
    service
        .random_at(2, 1, 64, 10, &mut source, &mut a)
        .unwrap();
    control
        .random_at(2, 1, 64, 10, &mut control_source, &mut b)
        .unwrap();
    assert_eq!(a, b); // deterministic TEST sources match; this is not entropy evidence
    let calls = service.random_counters();
    let reads = source.reads;
    for (domain, request, length, now) in [
        (4, 2, 32, 10),
        (0, 0, 32, 10),
        (2, 1, 32, 10),
        (2, 2, 0, 10),
        (2, 2, 225, 10),
        (2, 2, 32, 9),
        (2, 2, 32, u64::MAX),
        (2, 2, 32, u64::MAX - 99),
    ] {
        a.fill(0xa5);
        assert_eq!(
            service.random_at(domain, request, length, now, &mut source, &mut a),
            Err(Status::Invalid)
        );
        assert_eq!(a, [0xa5; random::MAX_BYTES]);
        assert_eq!(service.random_counters(), calls);
        assert_eq!(source.reads, reads);
    }
    service
        .random_at(2, 2, 64, 10, &mut source, &mut a)
        .unwrap();
    control
        .random_at(2, 2, 64, 10, &mut control_source, &mut b)
        .unwrap();
    assert_eq!(a[..64], b[..64]);
}

#[test]
fn real_tick_window_and_reseed_call_budget_are_separate() {
    let (mut service, mut source) = ready();
    let mut out = [0u8; random::MAX_BYTES];
    for request in 1..=64 {
        service
            .random_at(0, request, 1, 0, &mut source, &mut out)
            .unwrap();
    }
    assert_eq!(source.reads, 5); // four startup blocks and one independent RNG seed
    let counters = service.random_counters();
    out.fill(0xa5);
    for now in [0, 99] {
        assert_eq!(
            service.random_at(0, 65, 1, now, &mut source, &mut out),
            Err(Status::Quota)
        );
        assert_eq!(out, [0xa5; random::MAX_BYTES]);
        assert_eq!(service.random_counters(), counters);
    }
    service
        .random_at(0, 65, 1, 100, &mut source, &mut out)
        .unwrap();
    assert_eq!(source.reads, 6);
    assert_eq!(service.random_counters().seeds, 2);
    // Another domain has its own quota and RNG; the authoritative clock is global.
    assert_eq!(
        service.random_at(1, 1, 1, 99, &mut source, &mut out),
        Err(Status::Invalid)
    );
    service
        .random_at(1, 1, 1, 100, &mut source, &mut out)
        .unwrap();
}

#[test]
fn byte_quota_is_checked_before_reading_entropy_and_output() {
    let (mut service, mut source) = ready();
    let mut out = [0; random::MAX_BYTES];
    for request in 1..=36 {
        service
            .random_at(0, request, 224, 0, &mut source, &mut out)
            .unwrap();
    }
    assert_eq!(service.random_counters().bytes, 8064);
    assert_eq!(service.random_counters().seeds, 2);
    let reads = source.reads;
    out.fill(0xa5);
    assert_eq!(
        service.random_at(0, 37, 224, 99, &mut source, &mut out),
        Err(Status::Quota)
    );
    assert_eq!(out, [0xa5; random::MAX_BYTES]);
    assert_eq!(source.reads, reads);
    service
        .random_at(0, 37, 224, 100, &mut source, &mut out)
        .unwrap();
    assert_eq!(service.random_counters().seeds, 3);
}

#[test]
fn seed_failures_degrade_all_random_domains_without_revoking_confirmed_keys() {
    let (mut service, mut source) = ready();
    let key = service.generate(0, &mut source).unwrap();
    let mut sealed = [0u8; MAX_SEALED];
    let n = service
        .seal(0, key, b"confirmed", &mut source, &mut sealed)
        .unwrap();
    let (mut secret, mut recovery) = ([0u8; SECRET_BYTES], [0u8; RECOVERY_BLOB]);
    service
        .export_recovery(&mut source, &mut secret, &mut recovery)
        .unwrap();
    let generation = service.key_generation();
    let mut out = [0xa5; random::MAX_BYTES];
    source.fail = true;
    assert_eq!(
        service.random_at(0, 1, 32, 0, &mut source, &mut out),
        Err(Status::Degraded)
    );
    assert_eq!(out, [0xa5; random::MAX_BYTES]);
    assert_eq!(service.state(), State::Degraded);
    assert_eq!(service.keys_created(0), 1);
    assert_eq!(service.key_generation(), generation);
    assert_eq!(service.counters.revoked, 0);
    let reads = source.reads;
    assert_eq!(
        service.random_at(1, 1, 32, 100, &mut source, &mut out),
        Err(Status::Degraded)
    );
    assert_eq!(source.reads, reads);
    // Independently recover the already-confirmed ciphertext using its prior
    // external recovery secret. RANDOM failure did not reset the master.
    let (mut restored, mut other) = ready();
    restored.restore(&secret, &recovery, &mut other).unwrap();
    let mut plain = [0u8; MAX_PLAINTEXT];
    assert_eq!(
        restored.unseal(0, key, &sealed[..n], &mut other, &mut plain),
        Ok(9)
    );
    assert_eq!(&plain[..9], b"confirmed");
}

#[test]
fn required_reseed_runs_health_checks_not_just_source_success() {
    let (mut service, mut source) = ready();
    let mut out = [0; random::MAX_BYTES];
    for request in 1..=64 {
        service
            .random_at(0, request, 1, 0, &mut source, &mut out)
            .unwrap();
    }
    source.stuck = true;
    out.fill(0xa5);
    assert_eq!(
        service.random_at(0, 65, 1, 100, &mut source, &mut out),
        Err(Status::Degraded)
    );
    assert_eq!(out, [0xa5; random::MAX_BYTES]);
    assert_eq!(service.random_counters().seeds, 1);
}

#[test]
fn wire_maximum_fits_exactly_and_each_legacy_opcode_is_denied_on_random_ingress() {
    let (mut service, mut source) = ready();
    let request = wire::random_request(1, 224).unwrap();
    let header = Header::decode(&request).unwrap();
    let reply = wire::dispatch_random(&mut service, &mut source, 2, 0, &header, &request[16..32]);
    let answer = Header::decode(&reply).unwrap();
    assert_eq!(answer.length, 240);
    assert_eq!(answer.op, wire::RANDOM | wire::REPLY);
    assert_eq!(&reply[16..32], &request[16..32]);
    assert_eq!(
        wire::HEADER + wire::RANDOM_PREFIX + random::MAX_BYTES,
        wire::SIZE
    );
    let counters = service.random_counters();
    let reads = source.reads;
    for op in 1..=9 {
        let header = Header {
            op,
            index: 0,
            length: 0,
            request: 2,
            status: 0,
        };
        let denied = wire::dispatch_random(&mut service, &mut source, 2, 0, &header, &[]);
        let answer = Header::decode(&denied).unwrap();
        assert_eq!(answer.status, Status::Invalid.code());
        assert_eq!(answer.length, 0);
        assert!(denied[16..].iter().all(|x| *x == 0));
    }
    assert_eq!(service.random_counters(), counters);
    assert_eq!(source.reads, reads);
}

#[test]
fn random_wire_rejects_noncanonical_context_before_any_generation() {
    let (mut service, mut source) = ready();
    let original = wire::random_request(1, 32).unwrap();
    let reads = source.reads;
    for offset in [0, 1, 2, 12, 16, 20, 22, 23, 24, 31] {
        let mut bad = original;
        if offset == 22 {
            bad[offset] = 0;
        } else {
            bad[offset] ^= 0x80;
        }
        if let Some(header) = Header::decode(&bad) {
            let len = usize::from(header.length);
            let reply =
                wire::dispatch_random(&mut service, &mut source, 2, 0, &header, &bad[16..16 + len]);
            assert_ne!(Header::decode(&reply).unwrap().status, 0, "offset{offset}");
        }
    }
    assert_eq!(source.reads, reads);
    assert_eq!(service.random_counters(), random::Counters::default());
    assert!(wire::random_request(0, 32).is_err());
    assert!(wire::random_request(1, 0).is_err());
    assert!(wire::random_request(1, 225).is_err());
}

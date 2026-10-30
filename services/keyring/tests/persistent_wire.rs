//! Host tests for the persistence-only KEY1 dispatcher used by the native
//! administrative ingress. Deterministic entropy is a test fixture, not an
//! entropy claim; no native execution or trusted rollback anchor is proven.

use keyring_core::{
    persistent::{SnapshotInfo, PERSISTENT_BLOB},
    records::{RecordContext, RecordKind},
    wire::{self, Header},
    EntropySource, Service, Status, MAX_PLAINTEXT, MAX_SEALED, SECRET_BYTES,
};

const VOLUME: [u8; wire::VOLUME_ID] = *b"wire-volume-0001";
const SECRET: [u8; SECRET_BYTES] = [0x5a; SECRET_BYTES];

struct TestEntropy(u64);
impl EntropySource for TestEntropy {
    fn read(&mut self, out: &mut [u8]) -> Result<(), Status> {
        for byte in out {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            *byte = (self.0 >> 24) as u8;
        }
        Ok(())
    }
}

fn ready(seed: u64) -> (Service, TestEntropy) {
    let mut service = Service::new();
    let mut source = TestEntropy(seed);
    service.start(&mut source).unwrap();
    (service, source)
}

fn admin(
    service: &mut Service,
    source: &mut TestEntropy,
    request: &[u8; wire::SIZE],
) -> (Header, [u8; wire::SIZE]) {
    let header = Header::decode(request).unwrap();
    let reply = wire::dispatch_persistent(
        service,
        source,
        &header,
        &request[wire::HEADER..wire::HEADER + usize::from(header.length)],
    );
    (Header::decode(&reply).unwrap(), reply)
}

fn assert_denied(request: &[u8; wire::SIZE], reply: &(Header, [u8; wire::SIZE]), status: Status) {
    let sent = Header::decode(request).unwrap();
    assert_eq!(reply.0.op, sent.op | wire::REPLY);
    assert_eq!(reply.0.index, sent.index);
    assert_eq!(reply.0.request, sent.request);
    assert_eq!(reply.0.status, status.code());
    assert_eq!(reply.0.length, 0);
    assert_eq!(reply.1[wire::HEADER..], [0; wire::BODY]);
}

fn context() -> RecordContext {
    RecordContext {
        volume: VOLUME,
        domain: 0,
        object: 1,
        revision: 1,
        chunk: 0,
        length: MAX_PLAINTEXT as u16,
        kind: RecordKind::Content,
        schema: 1,
    }
}

fn export(
    service: &mut Service,
    source: &mut TestEntropy,
) -> ([u8; PERSISTENT_BLOB], SnapshotInfo) {
    let request = wire::export_persistent_request(7, &VOLUME, &SECRET).unwrap();
    let (header, reply) = admin(service, source, &request);
    assert_eq!(header.op, wire::EXPORT_PERSISTENT | wire::REPLY);
    assert_eq!(header.status, Status::Ok.code());
    assert_eq!(usize::from(header.length), wire::EXPORT_PERSISTENT_REPLY);
    let mut blob = [0; PERSISTENT_BLOB];
    blob.copy_from_slice(&reply[wire::HEADER..wire::HEADER + PERSISTENT_BLOB]);
    let info = wire::decode_snapshot_info(
        &reply[wire::HEADER + PERSISTENT_BLOB..wire::HEADER + wire::EXPORT_PERSISTENT_REPLY],
    )
    .unwrap();
    (blob, info)
}

#[test]
fn snapshot_info_round_trips_and_rejects_other_lengths() {
    let info = SnapshotInfo {
        generation: 0x0102_0304_0506_0708,
        created: [16, 3],
        revoked: [0xffff, 4],
    };
    let encoded = wire::encode_snapshot_info(info);
    assert_eq!(wire::decode_snapshot_info(&encoded), Some(info));
    assert_eq!(wire::decode_snapshot_info(&encoded[1..]), None);
    assert_eq!(
        wire::decode_snapshot_info(&[0; wire::SNAPSHOT_INFO + 1]),
        None
    );
    assert_eq!(wire::body_bounds(wire::EXPORT_PERSISTENT), Some((48, 48)));
    assert_eq!(
        wire::body_bounds(wire::RESTORE_PERSISTENT),
        Some((176, 176))
    );
    assert_eq!(wire::EXPORT_PERSISTENT_REPLY, 136);
}

#[test]
fn export_restores_exact_state_and_record_keys_into_a_cold_instance() {
    let (mut live, mut source) = ready(1);
    assert_eq!(live.generate(0, &mut source), Ok(0));
    live.revoke(0, 0).unwrap();
    assert_eq!(live.generate(0, &mut source), Ok(1));
    let mut sealed = [0; MAX_SEALED];
    let plaintext = [0x42u8; MAX_PLAINTEXT];
    let length = live
        .seal_record(0, 1, context(), &plaintext, &mut source, &mut sealed)
        .unwrap();
    let (blob, info) = export(&mut live, &mut source);
    assert_eq!(
        info,
        SnapshotInfo {
            generation: 3,
            created: [2, 0],
            revoked: [1, 0]
        }
    );
    assert_eq!(info, live.snapshot_info());
    assert_eq!(&blob[..8], b"NOSKEY01");
    assert_eq!(blob[12..28], VOLUME);

    let (mut cold, mut other) = ready(2);
    let request = wire::restore_persistent_request(9, &VOLUME, &SECRET, 0, &blob).unwrap();
    let (header, reply) = admin(&mut cold, &mut other, &request);
    assert_eq!(header.status, Status::Ok.code());
    assert_eq!(usize::from(header.length), wire::SNAPSHOT_INFO);
    assert_eq!(
        wire::decode_snapshot_info(&reply[wire::HEADER..wire::HEADER + wire::SNAPSHOT_INFO]),
        Some(info)
    );
    assert_eq!(cold.counters.restores, 1);
    let mut out = [0; MAX_PLAINTEXT];
    assert_eq!(
        cold.unseal_record(0, 1, context(), &sealed[..length], &mut other, &mut out),
        Ok(MAX_PLAINTEXT)
    );
    assert_eq!(out, plaintext);
    assert_eq!(
        cold.unseal_record(0, 0, context(), &sealed[..length], &mut other, &mut out),
        Err(Status::Revoked)
    );
    assert_eq!(cold.generate(0, &mut other), Ok(2));
    // Re-importing the identical current state is idempotent for the live instance.
    let again =
        wire::restore_persistent_request(10, &VOLUME, &SECRET, info.generation, &blob).unwrap();
    let (header, _) = admin(&mut live, &mut source, &again);
    assert_eq!(header.status, Status::Ok.code());
    assert_eq!(live.snapshot_info(), info);
}

#[test]
fn wrong_secret_floor_and_foreign_volume_are_refused_without_namespace_changes() {
    let (mut live, mut source) = ready(3);
    live.generate(0, &mut source).unwrap();
    let (blob, info) = export(&mut live, &mut source);
    let (mut cold, mut other) = ready(4);
    let before = cold.snapshot_info();
    let mut wrong = SECRET;
    wrong[0] ^= 1;
    let request = wire::restore_persistent_request(1, &VOLUME, &wrong, 0, &blob).unwrap();
    let reply = admin(&mut cold, &mut other, &request);
    assert_denied(&request, &reply, Status::Auth);
    assert_eq!(cold.counters.auth_failures, 1);
    let request =
        wire::restore_persistent_request(2, &VOLUME, &SECRET, info.generation + 1, &blob).unwrap();
    let reply = admin(&mut cold, &mut other, &request);
    assert_denied(&request, &reply, Status::Auth);
    // The floor refusal is not an AEAD failure; the counter stays at one.
    assert_eq!(cold.counters.auth_failures, 1);
    let other_volume = *b"wire-volume-0002";
    let request = wire::restore_persistent_request(3, &other_volume, &SECRET, 0, &blob).unwrap();
    let reply = admin(&mut cold, &mut other, &request);
    assert_denied(&request, &reply, Status::Auth);
    assert_eq!(cold.snapshot_info(), before);
    assert_eq!(cold.counters.restores, 0);
    let request =
        wire::restore_persistent_request(4, &VOLUME, &SECRET, info.generation, &blob).unwrap();
    let (header, _) = admin(&mut cold, &mut other, &request);
    assert_eq!(header.status, Status::Ok.code());
    // A volume-bound instance refuses an export under another identity.
    let request = wire::export_persistent_request(5, &other_volume, &SECRET).unwrap();
    let reply = admin(&mut cold, &mut other, &request);
    assert_denied(&request, &reply, Status::Invalid);
}

#[test]
fn persistence_dispatch_refuses_every_other_opcode_and_malformed_request() {
    let (mut service, mut source) = ready(5);
    service.generate(0, &mut source).unwrap();
    let (blob, _) = export(&mut service, &mut source);
    let before = service.snapshot_info();
    let sealed = [0u8; 40];
    let ctx = wire::encode_record_context(context()).unwrap();
    let mut record_body = [0u8; wire::RECORD_CONTEXT + MAX_SEALED];
    record_body[..wire::RECORD_CONTEXT].copy_from_slice(&ctx);
    let legacy = [
        wire::request(wire::STATUS, 0, 1, &[]).unwrap(),
        wire::request(wire::GENERATE, 0, 2, &[]).unwrap(),
        wire::request(wire::SEAL, 0, 3, b"x").unwrap(),
        wire::request(wire::UNSEAL, 0, 4, &sealed).unwrap(),
        wire::request(wire::REVOKE, 0, 5, &[]).unwrap(),
        wire::request(wire::EXPORT_RECOVERY, 0, 6, &[]).unwrap(),
        wire::request(
            wire::RESTORE,
            0,
            7,
            &[0; SECRET_BYTES + keyring_core::RECOVERY_BLOB],
        )
        .unwrap(),
        wire::request(wire::UNSEAL_RECORD, 0, 8, &record_body).unwrap(),
        wire::random_request(9, 16).unwrap(),
        wire::request(13, 0, 10, &[]).unwrap(),
    ];
    for request in legacy {
        let reply = admin(&mut service, &mut source, &request);
        assert_denied(&request, &reply, Status::Invalid);
    }
    let good = wire::restore_persistent_request(11, &VOLUME, &SECRET, 0, &blob).unwrap();
    let header = Header::decode(&good).unwrap();
    let body = &good[wire::HEADER..wire::HEADER + usize::from(header.length)];
    for (index, status, cut) in [(1, 0, 0), (0, 1, 0), (0, 0, 1)] {
        let mut changed = header;
        changed.index = index;
        changed.status = status;
        let used = &body[..body.len() - cut];
        changed.length = used.len() as u16;
        let reply = wire::dispatch_persistent(&mut service, &mut source, &changed, used);
        assert_eq!(
            Header::decode(&reply).unwrap().status,
            Status::Invalid.code()
        );
        assert_eq!(reply[wire::HEADER..], [0; wire::BODY]);
    }
    let mut long = [0u8; wire::RESTORE_PERSISTENT_BODY + 1];
    long[..body.len()].copy_from_slice(body);
    let mut changed = header;
    changed.length = long.len() as u16;
    let reply = wire::dispatch_persistent(&mut service, &mut source, &changed, &long);
    assert_eq!(
        Header::decode(&reply).unwrap().status,
        Status::Invalid.code()
    );
    let short_export = wire::request(wire::EXPORT_PERSISTENT, 0, 12, &[0; 47]).unwrap();
    let reply = admin(&mut service, &mut source, &short_export);
    assert_denied(&short_export, &reply, Status::Invalid);
    assert_eq!(service.snapshot_info(), before);
    assert_eq!(service.counters.restores, 0);

    // The record and RANDOM dispatchers never reach the persistence codec.
    for request in [
        wire::export_persistent_request(13, &VOLUME, &SECRET).unwrap(),
        wire::restore_persistent_request(14, &VOLUME, &SECRET, 0, &blob).unwrap(),
    ] {
        let header = Header::decode(&request).unwrap();
        let body = &request[wire::HEADER..wire::HEADER + usize::from(header.length)];
        let reply = wire::dispatch_record(&mut service, &mut source, 0, &header, body);
        assert_eq!(
            Header::decode(&reply).unwrap().status,
            Status::Invalid.code()
        );
        let reply = wire::dispatch_random(&mut service, &mut source, 0, 1, &header, body);
        assert_eq!(
            Header::decode(&reply).unwrap().status,
            Status::Invalid.code()
        );
    }
    assert_eq!(service.snapshot_info(), before);
}

#[test]
fn persistence_respects_service_readiness() {
    let mut blocked = Service::new();
    let mut source = TestEntropy(6);
    let request = wire::export_persistent_request(1, &VOLUME, &SECRET).unwrap();
    let reply = admin(&mut blocked, &mut source, &request);
    assert_denied(&request, &reply, Status::NotReady);
    let (mut live, mut live_source) = ready(7);
    let (blob, _) = export(&mut live, &mut live_source);
    let request = wire::restore_persistent_request(2, &VOLUME, &SECRET, 0, &blob).unwrap();
    let reply = admin(&mut blocked, &mut source, &request);
    assert_denied(&request, &reply, Status::NotReady);
}

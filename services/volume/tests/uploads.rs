//! v7 upload identity over the real volume Service. The disk is the existing
//! host-only memory/cache model; this does not prove native lifecycle wiring.
mod common;
use common::SimDisk;
use volume_core::{
    service::Service,
    table::Table,
    upload,
    wire::{self, Header},
    Status, PER_DOMAIN,
};

const EPOCH: u64 = 7;

fn begin(object: u64, name: &[u8], length: usize) -> Header {
    let mut h = Header::new(wire::BEGIN, 2, EPOCH, object);
    h.length = name.len() as u32;
    h.name_length = h.length;
    h.aux = length as u64;
    h
}
fn call(
    s: &mut Service,
    disk: &mut SimDisk,
    domain: usize,
    principal: u64,
    h: &Header,
    data: &[u8],
    token: u32,
) -> (Status, u32) {
    let response = s.handle_for(domain, principal, &upload::encode(h, data, token), disk);
    let reply = upload::validate_reply(h, token, &response, EPOCH).unwrap();
    (
        Status::from_code(i64::from(reply.status)),
        upload::token(&response),
    )
}
fn chunk(h: &Header, operation: u16, offset: u64, length: u32) -> Header {
    let mut next = Header::new(operation, h.request_id, EPOCH, h.object);
    next.revision = h.revision;
    next.aux = offset;
    next.length = length;
    next
}

#[test]
fn same_id_and_revision_cannot_replace_another_editor_or_principal() {
    let (mut s, mut disk) = (Service::new(EPOCH, Table::EMPTY, false), SimDisk::new());
    let a = begin(0, b"NameA", 4);
    let b = begin(0, b"NameB", 4);
    let (status, token) = call(&mut s, &mut disk, 0, 10, &a, b"NameA", 0);
    assert_eq!(status, Status::Ok);
    assert_ne!(token, 0);
    assert_eq!(
        call(&mut s, &mut disk, 0, 10, &b, b"NameB", 0).0,
        Status::Busy
    );
    assert_eq!(
        call(&mut s, &mut disk, 0, 11, &b, b"NameB", 0).0,
        Status::Busy
    );
    let before = disk.mutating_ops;
    for operation in [wire::APPEND, wire::COMMIT, wire::ABORT] {
        let h = chunk(
            &a,
            operation,
            if operation == wire::COMMIT { 4 } else { 0 },
            if operation == wire::APPEND { 4 } else { 0 },
        );
        let data = if operation == wire::APPEND {
            &b"BBBB"[..]
        } else {
            &[][..]
        };
        assert_eq!(
            call(&mut s, &mut disk, 0, 11, &h, data, token).0,
            Status::Access
        );
        assert_eq!(
            call(&mut s, &mut disk, 0, 10, &h, data, token + 1).0,
            Status::Access
        );
    }
    assert_eq!(disk.mutating_ops, before);
    assert_eq!(
        call(
            &mut s,
            &mut disk,
            0,
            10,
            &chunk(&a, wire::APPEND, 0, 4),
            b"AAAA",
            token
        )
        .0,
        Status::Ok
    );
    assert_eq!(
        call(
            &mut s,
            &mut disk,
            0,
            10,
            &chunk(&a, wire::COMMIT, 4, 0),
            &[],
            token
        )
        .0,
        Status::Ok
    );
    let entry = s.table().entries[0];
    assert_eq!(&entry.current.name[..entry.current.name_length], b"NameA");
    assert_eq!(
        entry.current.content_sum,
        volume_core::checksum_update(volume_core::FNV_OFFSET, b"AAAA")
    );
}

#[test]
fn stale_token_cannot_abort_or_append_a_later_upload_with_identical_id() {
    let (mut s, mut disk) = (Service::new(EPOCH, Table::EMPTY, false), SimDisk::new());
    let a = begin(0, b"First", 4);
    let (_, old) = call(&mut s, &mut disk, 0, 10, &a, b"First", 0);
    assert_eq!(
        call(
            &mut s,
            &mut disk,
            0,
            10,
            &chunk(&a, wire::ABORT, 0, 0),
            &[],
            old
        )
        .0,
        Status::Ok
    );
    let b = begin(0, b"Second", 4);
    let (_, new) = call(&mut s, &mut disk, 0, 10, &b, b"Second", 0);
    assert!(new > old);
    assert_eq!(
        call(
            &mut s,
            &mut disk,
            0,
            10,
            &chunk(&a, wire::APPEND, 0, 4),
            b"AAAA",
            old
        )
        .0,
        Status::Access
    );
    assert_eq!(
        call(
            &mut s,
            &mut disk,
            0,
            10,
            &chunk(&a, wire::ABORT, 0, 0),
            &[],
            old
        )
        .0,
        Status::Access
    );
    assert_eq!(
        call(
            &mut s,
            &mut disk,
            0,
            10,
            &chunk(&b, wire::APPEND, 0, 4),
            b"BBBB",
            new
        )
        .0,
        Status::Ok
    );
    assert_eq!(
        call(
            &mut s,
            &mut disk,
            0,
            10,
            &chunk(&b, wire::COMMIT, 4, 0),
            &[],
            new
        )
        .0,
        Status::Ok
    );
    assert_eq!(&s.table().entries[0].current.name[..6], b"Second");
}

#[test]
fn legacy_and_token_bound_uploads_cannot_take_over_each_other() {
    let (mut s, mut disk) = (Service::new(EPOCH, Table::EMPTY, false), SimDisk::new());
    let a = begin(0, b"Token", 4);
    let (_, token) = call(&mut s, &mut disk, 0, 10, &a, b"Token", 0);
    for operation in [wire::BEGIN, wire::APPEND, wire::COMMIT, wire::ABORT] {
        let (h, data) = match operation {
            wire::BEGIN => (a, &b"Token"[..]),
            wire::APPEND => (chunk(&a, operation, 0, 4), &b"BBBB"[..]),
            wire::COMMIT => (chunk(&a, operation, 4, 0), &[][..]),
            _ => (chunk(&a, operation, 0, 0), &[][..]),
        };
        let reply = s.handle(0, &wire::encode(&h, data), &mut disk);
        assert_eq!(wire::header(&reply).status, Status::Busy as i32);
    }
    assert_eq!(
        call(
            &mut s,
            &mut disk,
            0,
            10,
            &chunk(&a, wire::ABORT, 0, 0),
            &[],
            token
        )
        .0,
        Status::Ok
    );
    assert_eq!(
        wire::header(&s.handle(0, &wire::encode(&a, b"Token"), &mut disk)).status,
        0
    );
    assert_eq!(
        call(&mut s, &mut disk, 0, 10, &a, b"Token", 0).0,
        Status::Busy
    );
    let response = s.handle(
        0,
        &wire::encode(&chunk(&a, wire::ABORT, 0, 0), &[]),
        &mut disk,
    );
    assert_eq!(wire::header(&response).status, 0);
    assert_eq!(
        call(&mut s, &mut disk, 0, 10, &a, b"Token", 0).0,
        Status::Ok
    );
}

#[test]
fn trusted_detach_releases_only_owner_and_never_reuses_old_token() {
    let (mut s, mut disk) = (Service::new(EPOCH, Table::EMPTY, false), SimDisk::new());
    let a = begin(0, b"First", 4);
    let (_, old) = call(&mut s, &mut disk, 0, 10, &a, b"First", 0);
    assert_eq!(s.detach(11), 0);
    assert!(s.session(0).is_some());
    assert_eq!(s.detach(10), 1);
    assert_eq!(s.detach(10), 0);
    assert_eq!(s.table(), &Table::EMPTY);
    let (_, new) = call(&mut s, &mut disk, 0, 11, &a, b"First", 0);
    assert!(new > old);
    assert_eq!(
        call(
            &mut s,
            &mut disk,
            0,
            10,
            &chunk(&a, wire::ABORT, 0, 0),
            &[],
            old
        )
        .0,
        Status::Access
    );
    assert_eq!(
        call(
            &mut s,
            &mut disk,
            0,
            11,
            &chunk(&a, wire::ABORT, 0, 0),
            &[],
            new
        )
        .0,
        Status::Ok
    );
}

#[test]
fn wrong_domain_and_malformed_token_envelopes_never_touch_staging() {
    let (mut s, mut disk) = (Service::new(EPOCH, Table::EMPTY, false), SimDisk::new());
    let a = begin(0, b"First", 4);
    let (_, token) = call(&mut s, &mut disk, 0, 10, &a, b"First", 0);
    let mut foreign = chunk(&a, wire::ABORT, 0, 0);
    foreign.object = PER_DOMAIN as u64;
    assert_eq!(
        call(&mut s, &mut disk, 1, 10, &foreign, &[], token).0,
        Status::Access
    );
    let before = *s.session(0).unwrap();
    for body in [
        upload::encode(&a, b"First", token),
        upload::encode(&chunk(&a, wire::ABORT, 0, 0), &[], 0),
    ] {
        assert!(upload::validate_request(&body, EPOCH, 0).is_err());
        let response = s.handle_for(0, 10, &body, &mut disk);
        assert_eq!(wire::header(&response).status, Status::Invalid as i32);
    }
    assert_eq!(*s.session(0).unwrap(), before);
    let valid = upload::encode(&chunk(&a, wire::ABORT, 0, 0), &[], token);
    assert_eq!(
        wire::header(&s.handle_for(0, 0, &valid, &mut disk)).status,
        Status::Access as i32
    );
    assert_eq!(*s.session(0).unwrap(), before);
}

#[test]
fn reply_token_mismatch_and_zero_begin_token_are_protocol_errors() {
    let a = begin(0, b"First", 0);
    let mut reply = Header::new(wire::BEGIN, a.request_id, EPOCH, 0);
    reply.aux = 0;
    let body = upload::encode(&reply, &[], 0);
    assert_eq!(
        upload::validate_reply(&a, 0, &body, EPOCH),
        Err(Status::Protocol)
    );
    let abort = chunk(&a, wire::ABORT, 0, 0);
    let body = upload::encode(&abort, &[], 8);
    assert_eq!(
        upload::validate_reply(&abort, 7, &body, EPOCH),
        Err(Status::Protocol)
    );
}

#[test]
fn uncertainty_precedes_legacy_guard_and_cannot_resume_token_upload() {
    let (mut s, mut disk) = (Service::new(EPOCH, Table::EMPTY, false), SimDisk::new());
    let a = begin(0, b"Staged", 4);
    let (_, token) = call(&mut s, &mut disk, 0, 10, &a, b"Staged", 0);
    // A different legitimate legacy operation can fail in its table publish
    // while this staged upload remains. Unknown must dominate both protocols.
    let mut collection = Header::new(wire::COLLECTION, 3, EPOCH, 0);
    collection.aux = 1;
    collection.length = 1;
    collection.name_length = 1;
    disk.fail_once_at = Some(disk.mutating_ops);
    let failed = s.handle(0, &wire::encode(&collection, b"C"), &mut disk);
    assert_eq!(wire::header(&failed).status, Status::Unknown as i32);
    assert!(s.uncertain());
    let abort = chunk(&a, wire::ABORT, 0, 0);
    let legacy = s.handle(0, &wire::encode(&abort, &[]), &mut disk);
    assert_eq!(wire::header(&legacy).status, Status::Unknown as i32);
    assert_eq!(
        call(&mut s, &mut disk, 0, 10, &abort, &[], token).0,
        Status::Unknown
    );
}

#[test]
fn domain_counters_are_independent_and_equal_tokens_keep_distinct_owners() {
    let (mut s, mut disk) = (Service::new(EPOCH, Table::EMPTY, false), SimDisk::new());
    let a = begin(0, b"DomainA", 4);
    let b = begin(PER_DOMAIN as u64, b"DomainB", 4);
    let (_, token_a) = call(&mut s, &mut disk, 0, 10, &a, b"DomainA", 0);
    let (_, token_b) = call(&mut s, &mut disk, 1, 11, &b, b"DomainB", 0);
    assert_eq!((token_a, token_b), (1, 1));
    for (domain, principal, header) in [(0, 11, a), (1, 10, b)] {
        assert_eq!(
            call(
                &mut s,
                &mut disk,
                domain,
                principal,
                &chunk(&header, wire::APPEND, 0, 4),
                b"data",
                1
            )
            .0,
            Status::Access
        );
        assert_eq!(
            call(
                &mut s,
                &mut disk,
                domain,
                principal,
                &chunk(&header, wire::COMMIT, 4, 0),
                &[],
                1
            )
            .0,
            Status::Access
        );
        assert_eq!(
            call(
                &mut s,
                &mut disk,
                domain,
                principal,
                &chunk(&header, wire::ABORT, 0, 0),
                &[],
                1
            )
            .0,
            Status::Access
        );
    }
    assert_eq!(s.detach(11), 1);
    let (_, second_b) = call(&mut s, &mut disk, 1, 11, &b, b"DomainB", 0);
    assert_eq!(second_b, 2);
    assert_eq!(s.detach(11), 1);
    assert_eq!(s.detach(10), 1);
    let (_, second_a) = call(&mut s, &mut disk, 0, 10, &a, b"DomainA", 0);
    assert_eq!(second_a, 2);
}

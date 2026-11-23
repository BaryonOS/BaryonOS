//! Host tests of L2 INSTALL, INSTALL_COLLECTION and PROVENANCE_STAT
//! (docs/implementation/S09-L2-PROVENANCE.md section 8, tests 1-6 and 12)
//! through the real in-process service and client.
mod common;

use common::{pattern, recover, Local, SimDisk};
use volume_core::{
    archive::{self, Sink},
    client::{self, Client, Provenance},
    service::{Service, Staged},
    table::{self, Table},
    upload,
    wire::{self, Header, Manifest, ManifestVersion, SubOp},
    Status, CHUNK, FNV_OFFSET, MAX_LENGTH, NAME, PER_DOMAIN, QUOTA, SECTOR,
};

const EPOCH: u64 = 9;

fn fresh() -> (Service, SimDisk) {
    (Service::new(EPOCH, Table::EMPTY, false), SimDisk::new())
}

fn client<'a>(service: &'a mut Service, disk: &'a mut SimDisk, domain: usize) -> Client<Local<'a>> {
    Client::new(
        Local {
            service,
            disk,
            domain,
        },
        EPOCH,
        500 + 1000 * domain as u64,
    )
}

fn status(h: &Header) -> Status {
    Status::from_code(i64::from(h.status))
}

fn call(s: &mut Service, d: &mut SimDisk, domain: usize, h: &Header, data: &[u8]) -> Header {
    wire::header(&s.handle(domain, &wire::encode(h, data), d))
}

/// v7 request; returns the reply header and the reply token.
fn call7(
    s: &mut Service,
    d: &mut SimDisk,
    domain: usize,
    principal: u64,
    h: &Header,
    data: &[u8],
    token: u32,
) -> (Header, u32) {
    let body = s.handle_for(domain, principal, &upload::encode(h, data, token), d);
    (wire::header(&body), upload::token(&body))
}

fn version(origin: u64, length: usize, name: &[u8]) -> ManifestVersion {
    let mut out = ManifestVersion {
        origin,
        length,
        name_length: name.len(),
        name: [0; NAME],
    };
    out.name[..name.len()].copy_from_slice(name);
    out
}

/// Versions with origins `top, top - 1, ..`; object origins around them.
fn manifest(lengths: &[usize], top: u64) -> Manifest {
    let mut m = Manifest {
        origin_revision: top + 4,
        origin_created: 3,
        origin_modified: 11,
        count: lengths.len(),
        ..Manifest::default()
    };
    for (index, length) in lengths.iter().enumerate() {
        let name = format!("Versie{index}");
        m.versions[index] = version(top - index as u64, *length, name.as_bytes());
    }
    m
}

fn byte(object: u64, index: usize, offset: usize) -> u8 {
    pattern(object, 100 + index as u64, offset)
}

fn fill(object: u64) -> impl FnMut(usize, usize, &mut [u8]) {
    move |index, offset, buffer| {
        for (at, b) in buffer.iter_mut().enumerate() {
            *b = byte(object, index, offset + at);
        }
    }
}

fn install(
    s: &mut Service,
    d: &mut SimDisk,
    object: u64,
    m: &Manifest,
    meta: (u32, u16, u8),
) -> Result<u64, Status> {
    client(s, d, object as usize / PER_DOMAIN)
        .install(object, m, meta, fill(object))
        .map(|(revision, _)| revision)
}

fn begin_header(id: u64, object: u64, m: &Manifest, aux: u64) -> (Header, Vec<u8>) {
    let mut h = Header::new(wire::INSTALL_BEGIN, id, EPOCH, object);
    h.count = m.count as u32;
    h.aux = aux;
    h.length = m.bytes() as u32;
    let mut data = vec![0; m.bytes()];
    m.encode(&mut data);
    (h, data)
}

fn append_header(id: u64, object: u64, index: u32, offset: usize, length: usize) -> Header {
    let mut h = Header::new(wire::INSTALL_APPEND, id, EPOCH, object);
    h.count = index;
    h.aux = offset as u64;
    h.length = length as u32;
    h
}

fn commit_header(id: u64, object: u64, n: u32, total: usize) -> Header {
    let mut h = Header::new(wire::INSTALL_COMMIT, id, EPOCH, object);
    h.count = n;
    h.aux = total as u64;
    h
}

fn content(object: u64, index: usize, offset: usize, length: usize) -> Vec<u8> {
    (offset..offset + length)
        .map(|at| byte(object, index, at))
        .collect()
}

fn read_all(
    c: &mut Client<Local<'_>>,
    object: u64,
    revision: u64,
    selector: usize,
    length: usize,
) -> Vec<u8> {
    let mut out = vec![0; length];
    c.read_version(object, revision, selector, length, |offset, chunk| {
        out[offset..offset + chunk.len()].copy_from_slice(chunk)
    })
    .unwrap();
    out
}

fn provenance(s: &mut Service, d: &mut SimDisk, object: u64) -> Provenance {
    let revision = s.table().entries[object as usize].revision;
    client(s, d, object as usize / PER_DOMAIN)
        .provenance(object, revision)
        .unwrap()
}

// --- 1. Request shape -------------------------------------------------------

fn shape(h: &Header, data: &[u8], domain: usize) -> Result<Header, Status> {
    wire::validate_request(&wire::encode(h, data), EPOCH, domain)
}

#[test]
fn install_requests_are_validated_field_by_field() {
    let good = manifest(&[100, 0, 9], 7);
    let (h, data) = begin_header(1, 2, &good, wire::meta(1, 2, 0));
    assert!(shape(&h, &data, 0).is_ok());
    assert_eq!(shape(&h, &data, 1), Err(Status::Access), "foreign slot");
    let refused =
        |h: Header, data: &[u8]| assert_eq!(shape(&h, data, 0), Err(Status::Invalid), "{h:?}");
    refused(Header { revision: 1, ..h }, &data);
    refused(
        Header {
            name_length: 1,
            ..h
        },
        &data,
    );
    refused(Header { aux: 1 << 56, ..h }, &data);
    refused(
        Header {
            aux: wire::meta(0, 0, 5),
            ..h
        },
        &data,
    );
    refused(Header { count: 2, ..h }, &data);
    for (index, edit) in [
        // Non-zero name padding of record 0, an empty and an invalid name.
        (24 + 13 + 7, 1u8),
        (24 + 12, 0),
        (24 + 13, b'/'),
        // Header name length > NAME.
        (24 + 12, 33),
    ] {
        let mut bad = data.clone();
        bad[index] = edit;
        refused(h, &bad);
    }
    let mut cases = Vec::new();
    let mut m = good;
    m.origin_revision = 0;
    cases.push(m);
    let mut m = good;
    m.origin_created = 0;
    cases.push(m);
    let mut m = good;
    m.origin_modified = 0;
    cases.push(m);
    let mut m = good;
    m.origin_created = 12; // after modified
    cases.push(m);
    let mut m = good;
    m.versions[1].origin = 5; // gap: 7, 5
    cases.push(m);
    let mut m = good;
    m.versions[0].origin = 11; // equal to origin_revision: valid
    m.versions[1].origin = 10;
    m.versions[2].origin = 9;
    m.origin_revision = 11;
    assert!(m.valid());
    m.origin_revision = 10; // first version above origin_revision
    cases.push(m);
    let low = manifest(&[1, 1, 1], 2); // origin_0 < n (origins 2, 1, 0)
    cases.push(low);
    let mut m = good;
    m.versions[2].length = MAX_LENGTH + 1;
    cases.push(m);
    for m in cases {
        assert!(!m.valid());
        let mut data = vec![0; m.bytes()];
        m.encode(&mut data);
        refused(h, &data);
    }
    // Σ overflow: every length u32::MAX (each above MAX_LENGTH).
    let mut m = good;
    for v in &mut m.versions[..3] {
        v.length = u32::MAX as usize;
    }
    let mut huge = vec![0; m.bytes()];
    m.encode(&mut huge);
    assert!(!m.valid());
    refused(h, &huge);
    // 4 versions do not fit DATA; count 4 or 0 is refused.
    refused(Header { count: 4, ..h }, &data);
    refused(
        Header {
            count: 0,
            length: 24,
            ..h
        },
        &data[..24],
    );

    let append = append_header(1, 2, 0, 0, 5);
    assert!(shape(&append, &[1; 5], 0).is_ok());
    refused(Header { count: 3, ..append }, &[1; 5]);
    refused(
        Header {
            revision: 1,
            ..append
        },
        &[1; 5],
    );
    refused(
        Header {
            name_length: 1,
            ..append
        },
        &[1; 5],
    );
    refused(
        Header {
            length: 0,
            ..append
        },
        &[],
    );
    refused(
        Header {
            aux: MAX_LENGTH as u64,
            ..append
        },
        &[1; 5],
    );
    let commit = commit_header(1, 2, 3, 109);
    assert!(shape(&commit, &[], 0).is_ok());
    refused(Header { count: 0, ..commit }, &[]);
    refused(Header { count: 4, ..commit }, &[]);
    refused(
        Header {
            revision: 1,
            ..commit
        },
        &[],
    );
    refused(
        Header {
            aux: 3 * MAX_LENGTH as u64 + 1,
            ..commit
        },
        &[],
    );

    let mut collection = Header::new(wire::INSTALL_COLLECTION, 1, EPOCH, 0);
    collection.aux = 2;
    collection.name_length = 4;
    collection.length = 12;
    let mut data = b"Werk".to_vec();
    data.extend_from_slice(&5u64.to_le_bytes());
    assert!(shape(&collection, &data, 0).is_ok());
    let mut zero = b"Werk".to_vec();
    zero.extend_from_slice(&[0; 8]);
    refused(collection, &zero);
    refused(
        Header {
            length: 11,
            ..collection
        },
        &data[..11],
    );
    refused(
        Header {
            object: 1,
            ..collection
        },
        &data,
    );
    refused(
        Header {
            aux: 0,
            ..collection
        },
        &data,
    );
    refused(
        Header {
            aux: 5,
            ..collection
        },
        &data,
    );
    refused(
        Header {
            revision: 1,
            ..collection
        },
        &data,
    );
    refused(
        Header {
            count: 1,
            ..collection
        },
        &data,
    );
    refused(
        Header {
            name_length: 0,
            length: 8,
            ..collection
        },
        &data[4..],
    );
    let mut slash = data.clone();
    slash[0] = b'/';
    refused(collection, &slash);

    let mut stat = Header::new(wire::PROVENANCE_STAT, 1, EPOCH, 3);
    stat.revision = 2;
    assert!(shape(&stat, &[], 0).is_ok());
    refused(
        Header {
            revision: 0,
            ..stat
        },
        &[],
    );
    refused(Header { aux: 1, ..stat }, &[]);
    refused(
        Header {
            name_length: 1,
            ..stat
        },
        &[],
    );
    refused(Header { count: 2, ..stat }, &[]);
    refused(
        Header {
            count: 1,
            aux: 1,
            ..stat
        },
        &[],
    ); // collection selector needs the base object
    let collection_stat = Header {
        object: 0,
        count: 1,
        aux: 4,
        ..stat
    };
    assert!(shape(&collection_stat, &[], 0).is_ok());
    refused(
        Header {
            aux: 0,
            ..collection_stat
        },
        &[],
    );
    refused(
        Header {
            aux: 5,
            ..collection_stat
        },
        &[],
    );
    assert_eq!(
        shape(&Header::new(wire::LAST + 1, 1, EPOCH, 0), &[], 0),
        Err(Status::NoSys)
    );
}

// --- 2. Sessions --------------------------------------------------------------

#[test]
fn staged_session_size_is_pinned() {
    let staged = core::mem::size_of::<Staged>();
    let service = core::mem::size_of::<Service>();
    eprintln!("Staged={staged} Service={service}");
    assert!(staged <= 1600, "Staged grew to {staged}");
    // The native service runs on a 64 KiB stack (16 pages, image fixture).
    assert!(service <= 24 * 1024, "Service grew to {service}");
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Upload,
    Install,
}

/// Stage one session in domain 0, slot 1, untokened (v6) or tokened (v7,
/// principal 10). Returns (request id, token).
fn stage(s: &mut Service, d: &mut SimDisk, kind: Kind, tokened: bool) -> (u64, u32) {
    let (h, data) = match kind {
        Kind::Upload => {
            let mut h = Header::new(wire::BEGIN, 41, EPOCH, 1);
            h.aux = 600;
            h.length = 4;
            h.name_length = 4;
            (h, b"Naam".to_vec())
        }
        Kind::Install => begin_header(41, 1, &manifest(&[600], 5), 0),
    };
    let token = if tokened {
        let (reply, token) = call7(s, d, 0, 10, &h, &data, 0);
        assert_eq!(status(&reply), Status::Ok);
        assert_ne!(token, 0);
        token
    } else {
        assert_eq!(status(&call(s, d, 0, &h, &data)), Status::Ok);
        0
    };
    (41, token)
}

#[test]
fn session_matrix_every_cell() {
    let up_begin = || {
        let mut h = Header::new(wire::BEGIN, 77, EPOCH, 2);
        h.aux = 10;
        h.length = 1;
        h.name_length = 1;
        (h, b"N".to_vec())
    };
    let in_begin = || begin_header(77, 2, &manifest(&[10], 3), 0);
    let continuation = |op: u16, kind: Kind| -> (Header, Vec<u8>) {
        match op {
            wire::APPEND => {
                let mut h = Header::new(wire::APPEND, 41, EPOCH, 1);
                h.length = 1;
                (h, vec![1])
            }
            wire::COMMIT => {
                let mut h = Header::new(wire::COMMIT, 41, EPOCH, 1);
                h.aux = 600;
                (h, vec![])
            }
            wire::ABORT => (Header::new(wire::ABORT, 41, EPOCH, 1), vec![]),
            wire::INSTALL_APPEND => (append_header(41, 1, 0, 0, 1), vec![1]),
            _ => {
                let _ = kind;
                (commit_header(41, 1, 1, 600), vec![])
            }
        }
    };
    // (staged, tokened) -> expected statuses.
    for staged in [None, Some(Kind::Upload), Some(Kind::Install)] {
        for tokened in [false, true] {
            if staged.is_none() && tokened {
                continue;
            }
            let setup = |s: &mut Service, d: &mut SimDisk| match staged {
                Some(kind) => stage(s, d, kind, tokened).1,
                None => 0,
            };
            let label = format!("{staged:?} tokened={tokened}");
            // v6 BEGIN.
            let (mut s, mut d) = fresh();
            setup(&mut s, &mut d);
            let (h, data) = up_begin();
            let expected = match (staged, tokened) {
                (None, _) | (Some(Kind::Upload), false) => Status::Ok,
                _ => Status::Busy,
            };
            assert_eq!(
                status(&call(&mut s, &mut d, 0, &h, &data)),
                expected,
                "v6 BEGIN {label}"
            );
            if staged == Some(Kind::Install) {
                assert!(
                    matches!(s.staged(0), Some(Staged::Install(_))),
                    "never taken {label}"
                );
            }
            // v6 INSTALL_BEGIN.
            let (mut s, mut d) = fresh();
            setup(&mut s, &mut d);
            let (h, data) = in_begin();
            let expected = if staged.is_none() {
                Status::Ok
            } else {
                Status::Busy
            };
            assert_eq!(
                status(&call(&mut s, &mut d, 0, &h, &data)),
                expected,
                "v6 INSTALL_BEGIN {label}"
            );
            // v7 BEGIN / INSTALL_BEGIN.
            for (h, data) in [up_begin(), in_begin()] {
                let (mut s, mut d) = fresh();
                setup(&mut s, &mut d);
                let (reply, token) = call7(&mut s, &mut d, 0, 11, &h, &data, 0);
                let expected = if staged.is_none() {
                    Status::Ok
                } else {
                    Status::Busy
                };
                assert_eq!(status(&reply), expected, "v7 {} {label}", h.operation);
                assert_eq!(token != 0, staged.is_none());
            }
            // v6 continuations.
            for op in [
                wire::APPEND,
                wire::COMMIT,
                wire::ABORT,
                wire::INSTALL_APPEND,
                wire::INSTALL_COMMIT,
            ] {
                let (mut s, mut d) = fresh();
                setup(&mut s, &mut d);
                let (h, data) = continuation(op, Kind::Upload);
                let install_op = matches!(op, wire::INSTALL_APPEND | wire::INSTALL_COMMIT);
                let expected = match (staged, tokened) {
                    (_, true) => Status::Busy,
                    (None, _) => Status::Invalid,
                    (Some(Kind::Upload), false) if install_op => Status::Invalid,
                    (Some(Kind::Upload), false) if op == wire::COMMIT => Status::Range,
                    (Some(Kind::Install), false) if op == wire::APPEND || op == wire::COMMIT => {
                        Status::Invalid
                    }
                    (Some(Kind::Install), false) if op == wire::INSTALL_COMMIT => Status::Range,
                    _ => Status::Ok,
                };
                assert_eq!(
                    status(&call(&mut s, &mut d, 0, &h, &data)),
                    expected,
                    "v6 op {op} {label}"
                );
            }
            // v7 continuations: matching principal and token only.
            for op in [
                wire::APPEND,
                wire::COMMIT,
                wire::ABORT,
                wire::INSTALL_APPEND,
                wire::INSTALL_COMMIT,
            ] {
                for (principal, token_delta) in [(10, 0), (12, 0), (10, 1)] {
                    let (mut s, mut d) = fresh();
                    let token = setup(&mut s, &mut d);
                    let (h, data) = continuation(op, Kind::Upload);
                    let sent = if token == 0 { 1 } else { token + token_delta };
                    let (reply, _) = call7(&mut s, &mut d, 0, principal, &h, &data, sent);
                    let bound = tokened && principal == 10 && token_delta == 0;
                    let install_op = matches!(op, wire::INSTALL_APPEND | wire::INSTALL_COMMIT);
                    let expected = if !bound {
                        Status::Access
                    } else {
                        match (staged, op) {
                            (Some(Kind::Upload), _) if install_op => Status::Invalid,
                            (Some(Kind::Install), wire::APPEND | wire::COMMIT) => Status::Invalid,
                            (Some(Kind::Upload), wire::COMMIT) => Status::Range,
                            (Some(Kind::Install), wire::INSTALL_COMMIT) => Status::Range,
                            _ => Status::Ok,
                        }
                    };
                    assert_eq!(
                        status(&reply),
                        expected,
                        "v7 op {op} principal {principal} {label}"
                    );
                }
            }
        }
    }
}

#[test]
fn install_tokens_share_the_upload_counter_and_are_required_in_v7() {
    let (mut s, mut d) = fresh();
    let (_, first) = stage(&mut s, &mut d, Kind::Upload, true);
    let mut abort = Header::new(wire::ABORT, 41, EPOCH, 1);
    abort.revision = 0;
    assert_eq!(
        status(&call7(&mut s, &mut d, 0, 10, &abort, &[], first).0),
        Status::Ok
    );
    let (_, second) = stage(&mut s, &mut d, Kind::Install, true);
    assert_eq!(second, first + 1, "same per-domain counter, never reused");
    // Token 0 on an install continuation is malformed in v7.
    let (h, data) = (append_header(41, 1, 0, 0, 1), vec![1]);
    assert_eq!(
        status(&call7(&mut s, &mut d, 0, 10, &h, &data, 0).0),
        Status::Invalid
    );
    // A successful INSTALL_BEGIN reply without a token is a protocol error.
    let m = manifest(&[1], 1);
    let (h, _) = begin_header(5, 3, &m, 0);
    let reply = wire::encode(
        &Header {
            status: 0,
            aux: 1,
            count: 1,
            ..Header::new(wire::INSTALL_BEGIN, 5, EPOCH, 3)
        },
        &[],
    );
    assert_eq!(
        upload::validate_reply(&h, 0, &upload::wrap(reply, 0), EPOCH),
        Err(Status::Protocol)
    );
    assert!(upload::validate_reply(&h, 0, &upload::wrap(reply, 9), EPOCH).is_ok());
}

#[test]
fn read_only_refuses_install_operations_and_allows_provenance_stat() {
    let (mut s, mut d) = fresh();
    install(&mut s, &mut d, 1, &manifest(&[10], 4), (0, 0, 0)).unwrap();
    let table = *s.table();
    let mut s = Service::new(EPOCH, table, true);
    let (h, data) = begin_header(3, 2, &manifest(&[1], 1), 0);
    assert_eq!(status(&call(&mut s, &mut d, 0, &h, &data)), Status::Access);
    for h in [append_header(3, 2, 0, 0, 1), commit_header(3, 2, 1, 1)] {
        let data = vec![1; h.length as usize];
        assert_eq!(status(&call(&mut s, &mut d, 0, &h, &data)), Status::Access);
    }
    let mut collection = Header::new(wire::INSTALL_COLLECTION, 3, EPOCH, 0);
    collection.aux = 1;
    collection.name_length = 1;
    collection.length = 9;
    let mut data = b"W".to_vec();
    data.extend_from_slice(&1u64.to_le_bytes());
    assert_eq!(
        status(&call(&mut s, &mut d, 0, &collection, &data)),
        Status::Access
    );
    let p = provenance(&mut s, &mut d, 1);
    assert_eq!((p.installed, p.origin_revision, p.versions[0]), (1, 8, 4));
    assert_eq!(s.table(), &table);
}

#[test]
fn first_commit_installs_three_versions_exactly_on_the_rule_r_bound() {
    let mut d = SimDisk::new();
    table::write_format_marker(&mut d).unwrap();
    volume_core::Disk::flush(&mut d).unwrap();
    let mut s = Service::recover(EPOCH, false, &mut d).unwrap();
    let m = manifest(&[CHUNK + 1, 0, 3], 20);
    assert_eq!(install(&mut s, &mut d, 2, &m, (5, 6, 0)), Ok(3));
    let t = recover(d.settled()).unwrap();
    assert_eq!(
        (t.generation, t.clocks[0], t.format),
        (1, 1, table::VERSION_V3)
    );
    let e = &t.entries[2];
    assert_eq!(
        e.revision,
        t.generation + 2,
        "revision = generation + n - 1"
    );
    assert_eq!((e.created, e.modified, e.installed), (1, 1, 3));
    // A fresh instance mounts it and serves it.
    let mut s = Service::recover(EPOCH, true, &mut d).unwrap();
    let mut c = client(&mut s, &mut d, 0);
    let stat = c.stat(2).unwrap();
    assert_eq!(
        (stat.revision, stat.version_revision, stat.history_count),
        (3, 3, 2)
    );
    assert_eq!(
        read_all(&mut c, 2, 3, 0, CHUNK + 1),
        content(2, 0, 0, CHUNK + 1)
    );
    assert_eq!(read_all(&mut c, 2, 3, 2, 3), content(2, 2, 0, 3));
}

// --- 3. Refusals ---------------------------------------------------------------

#[test]
fn install_begin_refuses_used_slots_absent_collections_and_capacity() {
    let (mut s, mut d) = fresh();
    let mut c = client(&mut s, &mut d, 0);
    let (r1, _) = c.write(1, 0, b"A", 1, |_, b| b.fill(1)).unwrap();
    let (r2, _) = c.write(2, 0, b"B", 1, |_, b| b.fill(1)).unwrap();
    c.delete(2, r2).unwrap();
    let (r3, _) = c.write(3, 0, b"C", 1, |_, b| b.fill(1)).unwrap();
    let r3 = c.delete(3, r3).unwrap();
    c.purge(3, r3).unwrap();
    let _ = r1;
    let m = manifest(&[5], 2);
    for object in [1, 2, 3] {
        // present, tombstoned, purged: revision != 0, refused at BEGIN.
        let (h, data) = begin_header(9, object, &m, 0);
        assert_eq!(
            status(&call(&mut s, &mut d, 0, &h, &data)),
            Status::Stale,
            "slot {object}"
        );
        assert!(s.staged(0).is_none(), "slot {object}");
        assert_eq!(
            install(&mut s, &mut d, object, &m, (0, 0, 0)),
            Err(Status::Stale),
            "slot {object}"
        );
    }
    // Absent collection: Range before anything is reserved.
    let allocations = s.counters.chunk_allocations;
    let (h, data) = begin_header(9, 4, &m, wire::meta(0, 0, 2));
    assert_eq!(status(&call(&mut s, &mut d, 0, &h, &data)), Status::Range);
    assert!(s.staged(0).is_none());
    assert_eq!(s.counters.chunk_allocations, allocations);

    // Σ chunks above MAX_CHUNKS for one object: 3 x 13 = 39 fits QUOTA.
    let (mut s, mut d) = fresh();
    let big = manifest(&[13 * CHUNK; 3], 9);
    assert_eq!(install(&mut s, &mut d, 0, &big, (0, 0, 0)), Ok(3));
    assert_eq!(s.table().domain_chunks(0), 39);
    let (mut s, mut d) = fresh();
    let over = manifest(&[14 * CHUNK; 3], 9);
    assert_eq!(
        install(&mut s, &mut d, 0, &over, (0, 0, 0)),
        Err(Status::Quota)
    );
    // Full: the other domain holds 40 chunks, 24 remain for 39.
    let (mut s, mut d) = fresh();
    let mut other = client(&mut s, &mut d, 1);
    for slot in 0..3 {
        let length = if slot < 2 { MAX_LENGTH } else { 8 * CHUNK };
        other
            .write(8 + slot, 0, b"Vol", length, |_, b| b.fill(2))
            .unwrap();
    }
    assert_eq!(s.table().domain_chunks(1), QUOTA);
    assert_eq!(
        install(&mut s, &mut d, 0, &big, (0, 0, 0)),
        Err(Status::Full)
    );
    assert!(s.staged(0).is_none());
}

#[test]
fn staged_chunks_of_another_domain_are_never_handed_out() {
    let (mut s, mut d) = fresh();
    // Domain 1 stages a 16-chunk upload (not committed).
    let mut other = client(&mut s, &mut d, 1);
    other
        .write_partial(8, 0, b"Lang", MAX_LENGTH, 10, |_, b| b.fill(3))
        .unwrap();
    let Some(Staged::Upload(foreign)) = s.staged(1).copied() else {
        panic!("staged upload");
    };
    let foreign = foreign.chunks[..foreign.chunk_count].to_vec();
    let m = manifest(&[5 * CHUNK, 5 * CHUNK, 5 * CHUNK], 3);
    let (h, data) = begin_header(9, 0, &m, 0);
    let reply = call(&mut s, &mut d, 0, &h, &data);
    assert_eq!(status(&reply), Status::Ok);
    assert!(client::install_begun(&m, &reply));
    let Some(Staged::Install(mine)) = s.staged(0).copied() else {
        panic!("staged install");
    };
    let mine = &mine.chunks[..mine.chunk_count];
    assert_eq!(mine.len(), 15);
    assert!(mine.iter().all(|chunk| !foreign.contains(chunk)));
    // Each version starts at its own fresh chunk, nothing shared.
    let mut sorted = mine.to_vec();
    sorted.dedup();
    assert_eq!(sorted.len(), 15);
}

#[test]
fn continuation_refusals_and_zero_length_versions() {
    let (mut s, mut d) = fresh();
    let m = manifest(&[0, 600, 0], 3);
    let (h, data) = begin_header(9, 1, &m, 0);
    let reply = call(&mut s, &mut d, 0, &h, &data);
    assert_eq!(
        (status(&reply), reply.aux, reply.count),
        (Status::Ok, 600, 1)
    );
    let send = |s: &mut Service, d: &mut SimDisk, h: Header| {
        let data = content(1, h.count as usize, h.aux as usize, h.length as usize);
        status(&call(s, d, 0, &h, &data))
    };
    // Version 0 is empty, so version 1 is the target.
    assert_eq!(
        send(&mut s, &mut d, append_header(9, 1, 0, 0, 10)),
        Status::Range,
        "wrong count"
    );
    assert_eq!(
        send(&mut s, &mut d, append_header(9, 1, 2, 0, 10)),
        Status::Range,
        "wrong count"
    );
    assert_eq!(
        send(&mut s, &mut d, append_header(9, 1, 1, 5, 10)),
        Status::Range,
        "out of order"
    );
    assert_eq!(
        send(&mut s, &mut d, append_header(9, 1, 1, 0, 160)),
        Status::Ok
    );
    assert_eq!(
        send(&mut s, &mut d, append_header(9, 1, 1, 160, 160)),
        Status::Ok
    );
    // COMMIT before completion; count/aux mismatch; upload ops on an install.
    assert_eq!(
        status(&call(&mut s, &mut d, 0, &commit_header(9, 1, 3, 600), &[])),
        Status::Range
    );
    assert_eq!(
        status(&call(&mut s, &mut d, 0, &commit_header(9, 1, 2, 600), &[])),
        Status::Invalid
    );
    assert_eq!(
        status(&call(&mut s, &mut d, 0, &commit_header(9, 1, 3, 599), &[])),
        Status::Invalid
    );
    let mut upload_commit = Header::new(wire::COMMIT, 9, EPOCH, 1);
    upload_commit.aux = 600;
    assert_eq!(
        status(&call(&mut s, &mut d, 0, &upload_commit, &[])),
        Status::Invalid
    );
    assert_eq!(
        status(&call(&mut s, &mut d, 0, &commit_header(8, 1, 3, 600), &[])),
        Status::Invalid,
        "other request id"
    );
    assert_eq!(
        send(&mut s, &mut d, append_header(9, 1, 1, 320, 160)),
        Status::Ok
    );
    assert_eq!(
        send(&mut s, &mut d, append_header(9, 1, 1, 480, 121)),
        Status::Range,
        "overrun"
    );
    assert_eq!(
        send(&mut s, &mut d, append_header(9, 1, 1, 480, 120)),
        Status::Ok
    );
    assert_eq!(
        send(&mut s, &mut d, append_header(9, 1, 1, 600, 1)),
        Status::Range,
        "complete"
    );
    let reply = call(&mut s, &mut d, 0, &commit_header(9, 1, 3, 600), &[]);
    assert_eq!(
        (status(&reply), reply.revision, reply.aux),
        (Status::Ok, 3, 600)
    );
    let e = s.table().entries[1];
    assert_eq!(
        (e.current.length, e.history[0].length, e.history[1].length),
        (0, 600, 0)
    );
    assert_eq!(e.current.content_sum, FNV_OFFSET);
    let mut c = client(&mut s, &mut d, 0);
    assert_eq!(read_all(&mut c, 1, 3, 1, 600), content(1, 1, 0, 600));
    // INSTALL_COMMIT of an upload session.
    let (mut s, mut d) = fresh();
    stage(&mut s, &mut d, Kind::Upload, false);
    assert_eq!(
        status(&call(&mut s, &mut d, 0, &commit_header(41, 1, 1, 600), &[])),
        Status::Invalid
    );
}

#[test]
fn absent_collection_at_commit_is_range_without_a_table_write() {
    let (mut s, mut d) = fresh();
    let mut c = client(&mut s, &mut d, 0);
    c.collection(0, 3, 0, b"Map").unwrap();
    let m = manifest(&[7], 2);
    let id = c.install_begin(4, &m, (0, 0, 3)).unwrap();
    c.install_stream(4, id, &m, 0..1, &mut fill(4)).unwrap();
    // The collection disappears while the install is staged.
    c.collection(0, 3, 1, b"").unwrap();
    let writes = s.counters.table_writes;
    let before = *s.table();
    assert_eq!(
        status(&call(&mut s, &mut d, 0, &commit_header(id, 4, 1, 7), &[])),
        Status::Range
    );
    assert_eq!(s.counters.table_writes, writes);
    assert_eq!(s.table(), &before);
    assert!(s.staged(0).is_none(), "chunks released");
}

// --- 4./5. Normal use and the provenance lifecycle --------------------------

#[test]
fn normal_use_after_install_and_provenance_travel_with_versions() {
    let (mut s, mut d) = fresh();
    let mut c = client(&mut s, &mut d, 0);
    c.collection(0, 1, 0, b"Werk").unwrap();
    let m = manifest(&[700, CHUNK + 3, 0], 30);
    let (revision, _) = c.install(5, &m, (7, 2, 1), fill(5)).unwrap();
    assert_eq!(revision, 3);
    let (generation, clock) = c.domain_stat(0).unwrap();
    assert_eq!(clock, 2, "clock + 1 per install");
    assert!(revision <= generation + 2 && revision <= clock + 2);
    let stat = c.stat(5).unwrap(); // the unchanged client validates it
    assert_eq!(
        (
            stat.revision,
            stat.version_revision,
            stat.created,
            stat.modified
        ),
        (3, 3, 2, 2)
    );
    assert_eq!((stat.tags, stat.kind, stat.collection), (7, 2, 1));
    assert_eq!(stat.history.map(|h| h.revision), [2, 1]);
    // READ of every version, twice in alternating order (read cache keys).
    for _ in 0..2 {
        for (selector, length) in [(0, 700), (1, CHUNK + 3), (2, 0), (1, CHUNK + 3), (0, 700)] {
            assert_eq!(
                read_all(&mut c, 5, 3, selector, length),
                content(5, selector, 0, length)
            );
        }
    }
    for selector in 1..=2 {
        let named = c.version_stat(5, 3, selector).unwrap();
        assert_eq!(named.name(), format!("Versie{selector}").as_bytes());
    }
    let p = c.provenance(5, 3).unwrap();
    assert_eq!(
        (
            p.installed,
            p.origin_revision,
            p.origin_created,
            p.origin_modified
        ),
        (3, 34, 3, 11)
    );
    assert_eq!((p.version_count, p.versions), (3, [30, 29, 28]));

    // WRITE: the replaced current version keeps its origin in history.
    let (r4, _) = c.write(5, 3, b"Nieuw", 4, |_, b| b.fill(9)).unwrap();
    assert_eq!(r4, 4);
    let p = c.provenance(5, 4).unwrap();
    assert_eq!(
        (p.origin_revision, p.version_count, p.versions),
        (34, 3, [0, 30, 29])
    );
    // RESTORE each history slot: origins move, never copied.
    let r5 = c.restore(5, 4, 2).unwrap(); // version 2 (origin 29)
    assert_eq!(
        read_all(&mut c, 5, r5, 0, CHUNK + 3),
        content(5, 1, 0, CHUNK + 3)
    );
    let e = s.table().entries[5];
    assert_eq!(
        [e.current.origin, e.history[0].origin, e.history[1].origin],
        [29, 0, 30]
    );
    let mut c = client(&mut s, &mut d, 0);
    let r6 = c.restore(5, r5, 2).unwrap(); // origin 30 back to current
    assert_eq!(read_all(&mut c, 5, r6, 0, 700), content(5, 0, 0, 700));
    assert_eq!(read_all(&mut c, 5, r6, 2, 4), vec![9; 4]);
    let e = s.table().entries[5];
    assert_eq!(
        [e.current.origin, e.history[0].origin, e.history[1].origin],
        [30, 29, 0]
    );
    // RENAME and META leave provenance unchanged.
    let mut c = client(&mut s, &mut d, 0);
    let r7 = c.rename(5, r6, b"Anders").unwrap();
    let r8 = c.meta(5, r7, 1, 1, 0).unwrap();
    let p = c.provenance(5, r8).unwrap();
    assert_eq!((p.installed, p.origin_revision, p.versions[0]), (3, 34, 30));
    let t = recover(d.settled()).unwrap();
    assert_eq!(t.entries[5], s.table().entries[5]);
    assert_eq!(t.format, table::VERSION_V3);
}

fn installed_three(s: &mut Service, d: &mut SimDisk) {
    install(s, d, 1, &manifest(&[11, 12, 13], 40), (0, 0, 0)).unwrap();
}

#[test]
fn object_provenance_refuses_a_wrong_revision_and_a_never_used_slot() {
    let (mut s, mut d) = fresh();
    installed_three(&mut s, &mut d);
    let revision = s.table().entries[1].revision;
    assert_eq!(s.table().entries[2].revision, 0);
    let mut c = client(&mut s, &mut d, 0);
    // Error replies carry no revision (4.4).
    assert_eq!(
        c.provenance(1, revision + 1).unwrap_err(),
        (Status::Stale, 0)
    );
    assert_eq!(c.provenance(1, revision).unwrap().revision, revision);
    assert_eq!(c.provenance(2, 1).unwrap_err(), (Status::NotFound, 0));
}

#[test]
fn lifecycle_delete_restore_undelete_hides_versions_behind_the_gap() {
    let (mut s, mut d) = fresh();
    installed_three(&mut s, &mut d);
    let mut c = client(&mut s, &mut d, 0);
    let r4 = c.delete(1, 3).unwrap();
    // Absent: object origins only, no version origin (R2).
    let p = c.provenance(1, r4).unwrap();
    assert_eq!(
        (p.installed, p.origin_revision, p.version_count),
        (3, 44, 0)
    );
    let e = s.table().entries[1];
    assert_eq!([e.history[0].origin, e.history[1].origin], [40, 39]);
    // RESTORE of the absent entry (undelete): object origins unchanged.
    let mut c = client(&mut s, &mut d, 0);
    let r5 = c.restore(1, r4, 1).unwrap();
    let p = c.provenance(1, r5).unwrap();
    assert_eq!(
        (p.origin_revision, p.origin_created, p.origin_modified),
        (44, 3, 11)
    );
    // Current revision 5 follows history revision 2 with a gap: hidden.
    assert_eq!((p.version_count, p.versions[0]), (1, 40));
    assert_eq!(c.version_stat(1, r5, 1).unwrap_err().0, Status::NotFound);
    assert_eq!(s.table().entries[1].installed, 3);
}

#[test]
fn lifecycle_recreate_zeroes_object_origins_and_old_versions_keep_theirs() {
    let (mut s, mut d) = fresh();
    installed_three(&mut s, &mut d);
    let mut c = client(&mut s, &mut d, 0);
    let r4 = c.delete(1, 3).unwrap();
    // COMMIT on the absent entry re-creates the object.
    let (r5, _) = c.write(1, r4, b"Opnieuw", 2, |_, b| b.fill(4)).unwrap();
    let e = s.table().entries[1];
    assert_eq!(
        (e.origin_revision, e.origin_created, e.origin_modified),
        (0, 0, 0)
    );
    assert_eq!(e.installed, 3, "installed is never cleared");
    assert_eq!([e.current.origin, e.history[0].origin], [0, 40]);
    let mut c = client(&mut s, &mut d, 0);
    let p = c.provenance(1, r5).unwrap();
    assert_eq!(
        (
            p.installed,
            p.origin_revision,
            p.version_count,
            p.versions[0]
        ),
        (3, 0, 1, 0)
    );
    // RESTORE of the installed-era history version: its origin travels while
    // the object origins stay zero.
    let r6 = c.restore(1, r5, 1).unwrap();
    let e = s.table().entries[1];
    assert_eq!((e.current.origin, e.origin_revision), (40, 0));
    let mut c = client(&mut s, &mut d, 0);
    let p = c.provenance(1, r6).unwrap();
    assert_eq!((p.origin_revision, p.versions[0]), (0, 40));
    assert_eq!(read_all(&mut c, 1, r6, 0, 11), content(1, 0, 0, 11));
    recover(d.settled()).unwrap();
}

#[test]
fn lifecycle_purge_and_transact() {
    // PURGE of a present entry drops history origins only.
    let (mut s, mut d) = fresh();
    installed_three(&mut s, &mut d);
    let mut c = client(&mut s, &mut d, 0);
    let r4 = c.purge(1, 3).unwrap();
    let e = s.table().entries[1];
    assert_eq!(
        (e.origin_revision, e.current.origin, e.history_count),
        (44, 40, 0)
    );
    // DELETE then PURGE of the absent entry ends the incarnation.
    let mut c = client(&mut s, &mut d, 0);
    let r5 = c.delete(1, r4).unwrap();
    c.purge(1, r5).unwrap();
    let e = s.table().entries[1];
    assert_eq!(
        (e.origin_revision, e.origin_created, e.installed),
        (0, 0, 3)
    );
    assert!(e.history.iter().all(|v| v.origin == 0));
    recover(d.settled()).unwrap();

    // TRANSACT DELETE keeps origins; TRANSACT PURGE of the absent entry zeroes.
    let (mut s, mut d) = fresh();
    installed_three(&mut s, &mut d);
    let sub = |op, expected| SubOp {
        index: 1,
        op,
        collection: 0,
        tags: 0,
        expected,
        kind: 0,
    };
    let mut c = client(&mut s, &mut d, 0);
    c.transact(0, &[sub(wire::SUB_DELETE, 3)]).unwrap();
    let e = s.table().entries[1];
    assert_eq!((e.origin_revision, e.history[0].origin), (44, 40));
    let mut c = client(&mut s, &mut d, 0);
    c.transact(0, &[sub(wire::SUB_META, 0)]).unwrap_err();
    c.transact(0, &[sub(wire::SUB_PURGE, 4)]).unwrap();
    let e = s.table().entries[1];
    assert_eq!((e.origin_revision, e.installed, e.history_count), (0, 3, 0));
    // TRANSACT PURGE of a present entry keeps object origins.
    let (mut s, mut d) = fresh();
    installed_three(&mut s, &mut d);
    let mut c = client(&mut s, &mut d, 0);
    c.transact(0, &[sub(wire::SUB_PURGE, 3)]).unwrap();
    assert_eq!(s.table().entries[1].origin_revision, 44);
    recover(d.settled()).unwrap();
}

#[test]
fn collection_install_provenance_and_delete() {
    let (mut s, mut d) = fresh();
    let mut c = client(&mut s, &mut d, 0);
    assert_eq!(c.install_collection(0, 2, b"Foto's", 17), Ok(1));
    assert_eq!(
        c.install_collection(0, 2, b"Weer", 18).unwrap_err().0,
        Status::Stale
    );
    c.collection(0, 3, 0, b"Gewoon").unwrap();
    assert_eq!(
        c.install_collection(0, 3, b"Weer", 18).unwrap_err().0,
        Status::Stale
    );
    assert_eq!(c.collection_provenance(0, 2, 1), Ok(17));
    assert_eq!(c.collection_provenance(0, 3, 1), Ok(0));
    assert_eq!(
        c.collection_provenance(0, 2, 2).unwrap_err().0,
        Status::Stale
    );
    assert_eq!(
        c.collection_provenance(0, 4, 1).unwrap_err().0,
        Status::NotFound
    );
    let info = c.collection_stat(0, 2).unwrap();
    assert_eq!(
        (info.revision, info.present, info.name()),
        (1, true, "Foto's".as_bytes())
    );
    // Rename keeps the origin, delete zeroes it.
    let r2 = c.collection(0, 2, 1, b"Fotos").unwrap();
    assert_eq!(c.collection_provenance(0, 2, r2), Ok(17));
    let r3 = c.collection(0, 2, r2, b"").unwrap();
    assert_eq!(c.collection_provenance(0, 2, r3), Ok(0));
    let t = recover(d.settled()).unwrap();
    assert_eq!(t.collection(0, 2).origin_revision, 0);
    assert_eq!(t.format, table::VERSION_V3);
}

// --- 6. Cross-domain ------------------------------------------------------------

#[test]
fn installs_leave_the_other_domain_unobservable_and_foreign_selectors_are_refused() {
    let (mut s, mut d) = fresh();
    let mut other = client(&mut s, &mut d, 1);
    let (r, _) = other.write(9, 0, b"Ander", 3, |_, b| b.fill(1)).unwrap();
    let before = (
        other.domain_stat(1).unwrap(),
        other.provenance(9, r).unwrap(),
        other.stat(9).unwrap(),
    );
    install(&mut s, &mut d, 1, &manifest(&[3, 3], 9), (0, 0, 0)).unwrap();
    client(&mut s, &mut d, 0)
        .install_collection(0, 1, b"Eigen", 4)
        .unwrap();
    let mut other = client(&mut s, &mut d, 1);
    let after = (
        other.domain_stat(1).unwrap(),
        other.provenance(9, r).unwrap(),
        other.stat(9).unwrap(),
    );
    assert_eq!(before, after);
    assert_eq!(other.provenance(1, 2).unwrap_err().0, Status::Access);
    assert_eq!(
        other.collection_provenance(0, 1, 1).unwrap_err().0,
        Status::Access
    );
    let (h, data) = begin_header(3, 2, &manifest(&[1], 1), 0);
    assert_eq!(status(&call(&mut s, &mut d, 1, &h, &data)), Status::Access);
    assert_eq!(other_install_collection(&mut s, &mut d), Status::Access);
}

fn other_install_collection(s: &mut Service, d: &mut SimDisk) -> Status {
    let mut h = Header::new(wire::INSTALL_COLLECTION, 3, EPOCH, 0);
    h.aux = 2;
    h.name_length = 1;
    h.length = 9;
    let mut data = b"X".to_vec();
    data.extend_from_slice(&1u64.to_le_bytes());
    status(&call(s, d, 1, &h, &data))
}

// --- 7. Io, SHUTDOWN, detach (no table write) -----------------------------------

#[test]
fn io_during_install_is_a_certain_abort_and_releases_the_chunks() {
    // Mutating disk ops of a 600-byte single-version install on a fresh
    // disk: marker write + flush (0, 1), one full sector (2), the partial
    // sector at the completing APPEND (3), the commit FLUSH (4), table...
    for (fail, what) in [(2, "content"), (3, "partial sector"), (4, "commit flush")] {
        let (mut s, mut d) = fresh();
        d.fail_once_at = Some(fail);
        let m = manifest(&[600], 6);
        assert_eq!(
            install(&mut s, &mut d, 1, &m, (0, 0, 0)),
            Err(Status::Io),
            "{what}"
        );
        assert!(s.staged(0).is_none(), "{what}");
        assert_eq!(s.counters.table_writes, 0, "{what}");
        assert!(!s.uncertain());
        assert_eq!(s.table(), &Table::EMPTY);
        // The chunks are free again: the same lowest chunk is reused.
        assert_eq!(install(&mut s, &mut d, 1, &m, (0, 0, 0)), Ok(1), "{what}");
        assert_eq!(s.table().entries[1].current.chunks[0], 0, "{what}");
        let t = recover(d.settled()).unwrap();
        assert_eq!(t.entries[1], s.table().entries[1]);
    }
}

#[test]
fn shutdown_and_detach_drop_staged_installs() {
    for tokened in [false, true] {
        let (mut s, mut d) = fresh();
        stage(&mut s, &mut d, Kind::Install, tokened);
        let detached = s.detach(10);
        assert_eq!(
            detached,
            usize::from(tokened),
            "detach only tokened sessions of the principal"
        );
        assert_eq!(s.staged(0).is_some(), !tokened);
        if !tokened {
            assert_eq!(s.detach(11), 0);
        }
        let (mut s, mut d) = fresh();
        stage(&mut s, &mut d, Kind::Install, tokened);
        client(&mut s, &mut d, 0).shutdown(0).unwrap();
        assert!(s.staged(0).is_none());
        assert_eq!(s.counters.table_writes, 0);
    }
}

// --- 12. Export -------------------------------------------------------------------

struct Bytes(Vec<u8>);
impl Sink for Bytes {
    fn put(&mut self, bytes: &[u8]) -> Result<(), Status> {
        self.0.extend_from_slice(bytes);
        Ok(())
    }
}

#[test]
fn export_v2_refuses_with_range_inside_the_excess_window_then_exports_live_values() {
    let (mut s, mut d) = fresh();
    install(&mut s, &mut d, 2, &manifest(&[5, 6, 7], 12), (0, 0, 0)).unwrap();
    // clock 1, revision 3: exceeds by 2 = n - 1.
    let mut c = client(&mut s, &mut d, 0);
    assert_eq!(
        archive::export_v2(&mut c, 0, &mut Bytes(Vec::new())).unwrap_err(),
        Status::Range
    );
    c.write(0, 0, b"Een", 1, |_, b| b.fill(1)).unwrap();
    assert_eq!(
        archive::export_v2(&mut c, 0, &mut Bytes(Vec::new())).unwrap_err(),
        Status::Range
    );
    c.write(3, 0, b"Twee", 1, |_, b| b.fill(1)).unwrap();
    // n - 1 = 2 further commits close the window.
    let mut sink = Bytes(Vec::new());
    let exported = archive::export_v2(&mut c, 0, &mut sink).unwrap();
    let o = exported.archive.object(2).unwrap();
    assert_eq!(
        (o.revision, o.version_count),
        (3, 3),
        "live destination values"
    );
    assert_eq!(o.versions.map(|v| v.revision), [3, 2, 1]);
    let verified = archive::verify_v2(
        &mut Reader {
            bytes: &sink.0,
            at: 0,
        },
        &mut scratch(),
    )
    .unwrap();
    assert_eq!(verified, exported.archive);
}

#[test]
fn v1_export_of_an_installed_domain() {
    let (mut s, mut d) = fresh();
    install(&mut s, &mut d, 2, &manifest(&[300, 6], 12), (1, 0, 0)).unwrap();
    let mut sink = Bytes(Vec::new());
    let summary = archive::export(&mut client(&mut s, &mut d, 0), 0, &mut sink).unwrap();
    let verified = archive::verify(
        &mut Reader {
            bytes: &sink.0,
            at: 0,
        },
        &mut scratch(),
    )
    .unwrap();
    assert_eq!(summary, verified);
    assert_eq!(summary.objects, 1);
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl archive::Source for Reader<'_> {
    fn get(&mut self, buffer: &mut [u8]) -> Result<usize, Status> {
        let amount = buffer.len().min(self.bytes.len() - self.at);
        buffer[..amount].copy_from_slice(&self.bytes[self.at..self.at + amount]);
        self.at += amount;
        Ok(amount)
    }
}

fn scratch() -> Box<[u8; MAX_LENGTH]> {
    vec![0u8; MAX_LENGTH].into_boxed_slice().try_into().unwrap()
}

#[test]
fn provenance_reply_validation() {
    let request = |count: u32| {
        let mut h = Header::new(
            wire::PROVENANCE_STAT,
            4,
            EPOCH,
            if count == 1 { 0 } else { 2 },
        );
        h.revision = 3;
        h.count = count;
        h.aux = u64::from(count);
        h
    };
    let reply = |req: &Header, revision: u64, aux: u64, count: u32, versions: u32, data: &[u8]| {
        let mut h = Header::new(wire::PROVENANCE_STAT, 4, EPOCH, req.object);
        h.revision = revision;
        h.aux = aux;
        h.count = count;
        h.length = data.len() as u32;
        h.name_length = versions;
        wire::encode(&h, data)
    };
    let check = |req: &Header, body| client::validate_reply(req, &body, EPOCH).is_ok();
    let object = request(0);
    let mut origins = vec![0u8; 24];
    origins[..8].copy_from_slice(&9u64.to_le_bytes());
    origins[8..16].copy_from_slice(&2u64.to_le_bytes());
    origins[16..24].copy_from_slice(&5u64.to_le_bytes());
    let mut with = origins.clone();
    with.extend_from_slice(&8u64.to_le_bytes());
    assert!(check(&object, reply(&object, 3, 0, 0, 0, &[])));
    assert!(check(&object, reply(&object, 3, 0, 2, 0, &origins)));
    assert!(check(&object, reply(&object, 3, 0, 2, 1, &with)));
    assert!(
        !check(&object, reply(&object, 4, 0, 2, 1, &with)),
        "revision echo"
    );
    assert!(
        !check(&object, reply(&object, 3, 0, 4, 1, &with)),
        "count > 3"
    );
    assert!(
        !check(&object, reply(&object, 3, 0, 2, 0, &with)),
        "length vs versions"
    );
    assert!(
        !check(&object, reply(&object, 3, 0, 2, 4, &with)),
        "versions > 3"
    );
    assert!(
        !check(&object, reply(&object, 3, 0, 0, 0, &origins)),
        "count 0 carries data"
    );
    assert!(!check(&object, reply(&object, 3, 1, 2, 0, &origins)), "aux");
    let mut bad = origins.clone();
    bad[8..16].copy_from_slice(&6u64.to_le_bytes()); // created > modified
    assert!(!check(&object, reply(&object, 3, 0, 2, 0, &bad)));
    let mut bad = origins.clone();
    bad[..8].fill(0); // revision 0 with created != 0
    assert!(!check(&object, reply(&object, 3, 0, 2, 0, &bad)));
    let mut bad = origins.clone();
    bad[8..16].fill(0); // revision != 0 with created 0 (created <= modified holds)
    assert!(!check(&object, reply(&object, 3, 0, 2, 0, &bad)));
    let mut bad = vec![0u8; 24];
    bad[8] = 1; // revision 0, created != 0, modified 0
    assert!(!check(&object, reply(&object, 3, 0, 2, 0, &bad)));
    // 17 versions with a matching length would overrun `versions[..3]`.
    let mut many = origins.clone();
    many.resize(24 + 8 * 17, 0);
    assert!(
        !check(&object, reply(&object, 3, 0, 1, 17, &many)),
        "versions > 3 with matching length"
    );
    let mut ended = vec![0u8; 24];
    assert!(check(&object, reply(&object, 3, 0, 2, 0, &ended)));
    ended[16] = 1; // modified without revision
    assert!(!check(&object, reply(&object, 3, 0, 2, 0, &ended)));
    let coll = request(1);
    assert!(check(&coll, reply(&coll, 3, 7, 1, 0, &[])));
    assert!(check(&coll, reply(&coll, 3, 0, 0, 0, &[])));
    assert!(!check(&coll, reply(&coll, 3, 0, 1, 0, &[])));
    assert!(!check(&coll, reply(&coll, 3, 7, 0, 0, &[])));
    assert!(!check(&coll, reply(&coll, 2, 7, 1, 0, &[])));
    // Install replies: revision n, echoed aux.
    let commit = commit_header(4, 2, 3, 30);
    let ok = |h: &Header, revision, aux, count| {
        let mut r = Header::new(h.operation, h.request_id, EPOCH, h.object);
        (r.revision, r.aux, r.count) = (revision, aux, count);
        client::validate_reply(h, &wire::encode(&r, &[]), EPOCH).is_ok()
    };
    assert!(ok(&commit, 3, 30, 0));
    assert!(!ok(&commit, 4, 30, 0));
    assert!(!ok(&commit, 3, 31, 0));
    let append = append_header(4, 2, 1, 10, 5);
    assert!(ok(&append, 0, 15, 1));
    assert!(!ok(&append, 0, 15, 0));
    assert!(!ok(&append, 1, 15, 1));
    assert!(!ok(&append, 0, 14, 1), "received offset");
    assert!(!ok(&append, 0, 16, 1), "received offset");
    let mut coll_install = Header::new(wire::INSTALL_COLLECTION, 4, EPOCH, 0);
    coll_install.aux = 2;
    assert!(ok(&coll_install, 1, 2, 0));
    assert!(!ok(&coll_install, 2, 2, 0));
    let m = manifest(&[CHUNK + 1, 5], 4);
    let (begin, _) = begin_header(4, 2, &m, 0);
    assert!(ok(&begin, 0, CHUNK as u64 + 6, 3));
    let r = Header {
        aux: CHUNK as u64 + 6,
        count: 3,
        ..Header::new(wire::INSTALL_BEGIN, 4, EPOCH, 2)
    };
    assert!(client::install_begun(&m, &r));
    assert!(!client::install_begun(&m, &Header { count: 2, ..r }));
    assert!(!client::install_begun(&m, &Header { aux: 1, ..r }));
    assert!(!ok(&begin, 1, CHUNK as u64 + 6, 3));
    let _ = SECTOR;
}

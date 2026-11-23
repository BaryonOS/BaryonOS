//! Host tests (U05-T3 S1) against the real volume-core service and an
//! in-memory disk. Malformed replies are injected below `Client`, so they
//! exercise `validate_reply` as the primary check (lead decision B1).
extern crate std;
use std::string::String;
use std::vec;
use std::vec::Vec;

use super::*;
use volume_core::{service::Service, table::Table, wire, Disk, SECTOR, VOLUME_SECTORS};

const EPOCH: u64 = 7;
const VECTORS: &str = include_str!("../vectors.tsv");

struct Memory(Vec<[u8; SECTOR]>);

impl Disk for Memory {
    fn read(&mut self, sector: u64, buffer: &mut [u8; SECTOR]) -> Result<(), Status> {
        *buffer = *self.0.get(sector as usize).ok_or(Status::Range)?;
        Ok(())
    }
    fn write(&mut self, sector: u64, buffer: &[u8; SECTOR]) -> Result<(), Status> {
        *self.0.get_mut(sector as usize).ok_or(Status::Range)? = *buffer;
        Ok(())
    }
    fn flush(&mut self) -> Result<(), Status> {
        Ok(())
    }
}

/// The service as seen from one kernel-bound domain, with fault injection.
struct Local {
    service: Service,
    disk: Memory,
    domain: usize,
    calls: usize,
    /// Fail the request with this call number (0-based) without sending it.
    fail_at: Option<(usize, Status)>,
    /// Rewrite every LIST reply after the service produced it.
    tamper: Option<fn(&mut [u8; wire::SIZE])>,
}

impl Transport for Local {
    fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
        let call = self.calls;
        self.calls += 1;
        if let Some((at, status)) = self.fail_at {
            if at == call {
                return Err(status);
            }
        }
        let mut reply = self.service.handle(self.domain, request, &mut self.disk);
        if let Some(tamper) = self.tamper {
            if wire::header(request).operation == wire::LIST {
                tamper(&mut reply);
            }
        }
        Ok(reply)
    }
}

fn local(domain: usize) -> Local {
    Local {
        service: Service::new(EPOCH, Table::EMPTY, false),
        disk: Memory(vec![[0; SECTOR]; VOLUME_SECTORS as usize]),
        domain,
        calls: 0,
        fail_at: None,
        tamper: None,
    }
}

fn search(local: Local, domain: usize) -> Search<Local> {
    Search::new(Client::new(local, EPOCH, 1), domain).unwrap()
}

/// Same service and disk, seen from another domain.
fn rebind(search: Search<Local>, domain: usize) -> Search<Local> {
    let mut local = search.client.transport;
    local.domain = domain;
    Search::new(Client::new(local, EPOCH, 90_000), domain).unwrap()
}

fn generation(search: &Search<Local>) -> u64 {
    search
        .client
        .transport
        .service
        .table()
        .domain_generation(search.domain)
}

fn hex(field: &str) -> Vec<u8> {
    (0..field.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&field[i..i + 2], 16).unwrap())
        .collect()
}

fn hexlist(field: &str) -> Vec<Vec<u8>> {
    if field == "-" {
        return Vec::new();
    }
    field.split(',').map(hex).collect()
}

/// Rows of `kind`, without trailing `#` comment fields.
fn rows(kind: &str) -> Vec<Vec<&'static str>> {
    VECTORS
        .split('\n')
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            line.split('\t')
                .filter(|field| !field.starts_with('#'))
                .collect::<Vec<_>>()
        })
        .filter(|fields| fields[0] == kind)
        .map(|fields| fields[1..].to_vec())
        .collect()
}

fn write(search: &mut Search<Local>, object: u64, expected: u64, name: &[u8]) -> u64 {
    let content = [0x5a; 3];
    let (revision, _) = search
        .client
        .write(object, expected, name, content.len(), |offset, buffer| {
            buffer.copy_from_slice(&content[offset..offset + buffer.len()])
        })
        .unwrap();
    revision
}

/// Domain 0 holds the vector record set, built through the real service.
fn fixture() -> Search<Local> {
    let mut search = search(local(0), 0);
    for row in rows("label") {
        let index: usize = row[0].parse().unwrap();
        assert_eq!(search.client.collection(0, index, 0, &hex(row[1])), Ok(1));
    }
    for row in rows("record") {
        let object: u64 = row[0].parse().unwrap();
        let revision: u64 = row[1].parse().unwrap();
        let collection: u8 = row[3].parse().unwrap();
        let mut current = write(&mut search, object, 0, &hex(row[2]));
        if collection != 0 {
            current = search
                .client
                .meta(object, current, 0, 0, collection)
                .unwrap();
        }
        assert_eq!(current, revision, "record {object}");
    }
    search.rebuild().unwrap();
    search
}

fn objects(hits: &Hits) -> Vec<u64> {
    let mut found: Vec<u64> = hits.iter().map(|hit| hit.object).collect();
    found.sort_unstable();
    found
}

fn find(search: &Search<Local>, query: &str) -> Vec<(u64, u64)> {
    let mut found: Vec<(u64, u64)> = search
        .query(query.as_bytes())
        .unwrap()
        .iter()
        .map(|hit| (hit.object, hit.revision))
        .collect();
    found.sort_unstable();
    found
}

#[test]
fn n8_whitespace_set_is_rust_is_whitespace() {
    let listed: Vec<u32> = rows("ws")[0][0]
        .split(',')
        .map(|c| u32::from_str_radix(c, 16).unwrap())
        .collect();
    let rust: Vec<u32> = (0..=0x10ffff)
        .filter_map(char::from_u32)
        .filter(|c| c.is_whitespace())
        .map(u32::from)
        .collect();
    assert_eq!(listed, rust);
}

#[test]
fn n8_tokeniser_vectors_are_split_whitespace() {
    for row in rows("token") {
        let field = String::from_utf8(hex(row[0])).unwrap();
        let tokens: Vec<Vec<u8>> = field
            .split_whitespace()
            .map(|t| t.as_bytes().to_vec())
            .collect();
        assert_eq!(tokens, hexlist(row[1]), "{field:?}");
    }
}

#[test]
fn n8_query_vectors_and_digest_match_the_oracle() {
    let search = fixture();
    assert_eq!(search.generation, Some(generation(&search)));
    let mut value = DIGEST_START;
    for row in rows("query") {
        let query = hex(row[0]);
        let hits = search.query(&query).unwrap();
        let wanted: Vec<u64> = if row[1] == "-" {
            Vec::new()
        } else {
            row[1].split(',').map(|o| o.parse().unwrap()).collect()
        };
        assert_eq!(objects(&hits), wanted, "{:?}", String::from_utf8(query));
        value = digest(value, &hits);
    }
    assert_eq!(
        value,
        u64::from_str_radix(rows("digest")[0][0], 16).unwrap()
    );
}

#[test]
fn b4_sixteen_token_name_and_label_fill_t() {
    let search = fixture();
    // Record 3: 16 name tokens plus the 16-token label of collection 2.
    assert_eq!(find(&search, "q F a p"), vec![(3, 2)]);
    assert_eq!(find(&search, "F p"), vec![(3, 2)]);
}

#[test]
fn b1_collection_4_is_accepted_and_indexed() {
    let search = fixture();
    assert_eq!(find(&search, "archive"), vec![(2, 2)]);
    let entry = wire::ListEntry {
        object: 2,
        revision: 2,
        collection: COLLECTIONS as u8,
        ..wire::ListEntry::EMPTY
    };
    assert_eq!(check_entry(0, &entry), Ok(2));
}

#[test]
fn b1_check_entry_is_defence_in_depth() {
    let good = wire::ListEntry {
        object: 9,
        revision: 1,
        name_length: NAME,
        ..wire::ListEntry::EMPTY
    };
    assert_eq!(check_entry(1, &good), Ok(1));
    let bad = |change: fn(&mut wire::ListEntry)| {
        let mut entry = good;
        change(&mut entry);
        check_entry(1, &entry)
    };
    let protocol = Err(Error::Volume(Status::Protocol));
    assert_eq!(bad(|e| e.collection = COLLECTIONS as u8 + 1), protocol);
    assert_eq!(bad(|e| e.name_length = NAME + 1), protocol);
    assert_eq!(bad(|e| e.name_length = 255), protocol);
    assert_eq!(bad(|e| e.revision = 0), protocol);
    assert_eq!(bad(|e| e.object = 1), Err(Error::Foreign));
    assert_eq!(bad(|e| e.object = 16), Err(Error::Foreign));
    assert_eq!(bad(|e| e.object = 255), Err(Error::Foreign));
}

/// N1-N4: every malformed LIST reply stops in `Client` as `Protocol`; the
/// index keeps answering its last complete snapshot and sends no extra pages.
#[test]
fn n1_to_n4_malformed_list_replies_are_protocol_and_publish_nothing() {
    let tampers: [fn(&mut [u8; wire::SIZE]); 8] = [
        |r| r[64 + 1] = NAME as u8 + 1,                  // N1 name_length 33
        |r| r[64 + 1] = 255,                             // N1 name_length 255
        |r| r[64] = PER_DOMAIN as u8,                    // N2 other domain
        |r| r[64] = 16,                                  // N2 beyond OBJECTS
        |r| r[64 + 8..64 + 16].copy_from_slice(&[0; 8]), // N3 revision 0
        |r| r[64 + 2] = COLLECTIONS as u8 + 1,           // N3 collection 5
        |r| {
            // N4 duplicate object: the second entry repeats the first.
            let first: [u8; wire::LIST_ENTRY] = r[64..64 + wire::LIST_ENTRY].try_into().unwrap();
            r[64 + wire::LIST_ENTRY..64 + 2 * wire::LIST_ENTRY].copy_from_slice(&first);
        },
        |r| r[64 + 16 + NAME - 1] = b'x', // nonzero name tail
    ];
    for (case, tamper) in tampers.into_iter().enumerate() {
        let mut search = fixture();
        let before = find(&search, "meeting");
        search.rename(4, 1, b"after").unwrap();
        search.client.transport.tamper = Some(tamper);
        assert_eq!(
            search.rebuild(),
            Err(Error::Volume(Status::Protocol)),
            "case {case}"
        );
        assert_eq!(find(&search, "meeting"), before, "case {case}");
        assert_eq!(find(&search, "after"), vec![(4, 2)], "case {case}");
        search.client.transport.tamper = None;
        assert_eq!(search.rebuild(), Ok(()), "case {case}");
        assert_eq!(search.generation, Some(generation(&search)));
    }
}

/// N5 (B2): `Unknown`/`Timeout` at any request of a rebuild or a mutation
/// revokes the grant; the same epoch is refused again and nothing more is sent.
#[test]
fn n5_unknown_or_timeout_revokes_for_the_whole_epoch() {
    let mut probe = fixture();
    probe.rename(0, 2, b"probe").unwrap();
    let start = probe.client.transport.calls;
    probe.rebuild().unwrap();
    let requests = probe.client.transport.calls - start;
    assert!(requests >= 2 + COLLECTIONS, "LIST, all STATs, LIST again");
    for status in [Status::Unknown, Status::Timeout] {
        for position in 0..requests {
            let mut search = fixture();
            search.rename(0, 2, b"probe").unwrap();
            let at = search.client.transport.calls + position;
            search.client.transport.fail_at = Some((at, status));
            assert_eq!(search.rebuild(), Err(Error::Volume(status)));
            assert_revoked(&mut search);
        }
        let mut search = fixture();
        let at = search.client.transport.calls;
        search.client.transport.fail_at = Some((at, status));
        assert_eq!(search.rename(0, 2, b"x"), Err(Error::Volume(status)));
        assert_revoked(&mut search);
    }
}

fn assert_revoked(search: &mut Search<Local>) {
    let denied = Err(Error::Index(IndexError::Denied));
    search.client.transport.fail_at = None;
    assert_eq!(search.query(b"meeting").map(|_| ()), denied);
    assert_eq!(search.regrant(), denied);
    let calls = search.client.transport.calls;
    assert_eq!(search.rebuild(), denied);
    assert_eq!(search.rename(1, 1, b"y").map(|_| ()), denied);
    assert_eq!(search.delete(1, 1).map(|_| ()), denied);
    assert_eq!(search.collection(1, 1, b"z").map(|_| ()), denied);
    assert_eq!(search.query(b"meeting").map(|_| ()), denied);
    assert_eq!(
        search.client.transport.calls, calls,
        "no request after revoke"
    );
}

#[test]
fn n6_query_bounds_are_explicit_errors() {
    let search = fixture();
    let error = |query: &[u8]| search.query(query).map(|_| ());
    assert_eq!(error(b""), Err(Error::Index(IndexError::EmptyQuery)));
    assert_eq!(
        error("\u{3000} \t".as_bytes()),
        Err(Error::Index(IndexError::EmptyQuery))
    );
    assert_eq!(
        error(&[b'a'; TEXT + 1]),
        Err(Error::Index(IndexError::TextFull))
    );
    assert_eq!(error(b"a\xff"), Err(Error::Index(IndexError::InvalidUtf8)));
    // T = 32 one-byte tokens need 63 bytes; a 33rd needs 65 > B, so with
    // B = 64 the byte bound always trips first and TokensFull is unreachable.
    let tokens = |n: usize| vec!["a"; n].join(" ").into_bytes();
    assert_eq!(tokens(TOKENS).len(), 2 * TOKENS - 1);
    assert_eq!(error(&tokens(TOKENS)), Ok(()));
    assert_eq!(
        error(&tokens(TOKENS + 1)),
        Err(Error::Index(IndexError::TextFull))
    );
}

#[test]
fn queries_wait_for_a_first_rebuild() {
    let search = search(local(0), 0);
    assert_eq!(search.query(b"a").map(|_| ()), Err(Error::RebuildRequired));
}

/// m9: the index takes the revision RENAME and DELETE returned.
#[test]
fn n11_rename_and_delete_use_returned_revisions() {
    let mut search = fixture();
    assert_eq!(search.rename(0, 2, b"Agenda"), Ok(3));
    assert_eq!(find(&search, "meeting"), vec![]);
    assert_eq!(find(&search, "agenda work"), vec![(0, 3)]);
    assert_eq!(search.delete(1, 1), Ok(2));
    assert_eq!(find(&search, "budget"), vec![]);
    // An older revision cannot overwrite either change.
    let stale = |object, revision, name: &'static [u8]| Document {
        object,
        revision,
        name,
        collections: b"",
        metadata: b"",
        content: b"",
    };
    let grant = search.grant;
    assert_eq!(
        search.index.update(grant, stale(0, 2, b"Meeting")),
        Err(IndexError::StaleRevision)
    );
    assert_eq!(
        search.index.update(grant, stale(1, 1, b"budget")),
        Err(IndexError::StaleRevision)
    );
    // A later full rebuild agrees: LIST no longer reports the deleted object.
    search.rebuild().unwrap();
    assert_eq!(search.generation, Some(generation(&search)));
    assert_eq!(find(&search, "budget"), vec![]);
    assert_eq!(find(&search, "agenda"), vec![(0, 3)]);
}

#[test]
fn refused_mutation_keeps_the_index_answering() {
    let mut search = fixture();
    assert!(matches!(search.rename(0, 1, b"x"), Err(Error::Volume(_))));
    assert!(matches!(search.delete(1, 9), Err(Error::Volume(_))));
    assert!(matches!(
        search.collection(1, 9, b"x"),
        Err(Error::Volume(_))
    ));
    assert_eq!(find(&search, "meeting"), vec![(0, 2)]);
    assert!(!search.dirty);
}

#[test]
fn foreign_objects_are_refused_before_any_request() {
    let mut search = fixture();
    let calls = search.client.transport.calls;
    assert_eq!(search.rename(8, 1, b"x"), Err(Error::Foreign));
    assert_eq!(search.delete(15, 1), Err(Error::Foreign));
    assert_eq!(search.meta(16, 1, 0, 0, 1), Err(Error::Foreign));
    assert_eq!(search.write(9, 0, b"x", b"y"), Err(Error::Foreign));
    assert_eq!(search.client.transport.calls, calls);
}

/// B3: COLLECTION and META force a full rebuild; the old label gives 0 hits.
#[test]
fn b3_collection_rename_and_meta_rebuild_labels() {
    let mut search = fixture();
    assert_eq!(search.collection(1, 1, b"Home"), Ok(2));
    assert_eq!(find(&search, "work"), vec![]);
    assert_eq!(find(&search, "home meeting"), vec![(0, 2)]);
    // A non-empty collection cannot be deleted (Busy); moving the member out
    // drops the label from its record.
    assert_eq!(
        search.collection(4, 1, b""),
        Err(Error::Volume(Status::Busy))
    );
    assert_eq!(search.meta(2, 2, 0, 0, 0), Ok(3));
    assert_eq!(find(&search, "archive"), vec![]);
    assert_eq!(find(&search, "café"), vec![(2, 3)]);
    assert_eq!(search.meta(1, 1, 0, 0, 1), Ok(2));
    assert_eq!(find(&search, "home"), vec![(0, 2), (1, 2)]);
    // Rename keeps the label from the last applied snapshot.
    assert_eq!(search.rename(1, 2, b"Taxes"), Ok(3));
    assert_eq!(find(&search, "home taxes"), vec![(1, 3)]);
    assert_eq!(search.generation, Some(generation(&search) - 1));
}

#[test]
fn b3_failed_forced_rebuild_blocks_queries_until_rebuilt() {
    let mut search = fixture();
    // META succeeds (one request); the first request of its rebuild fails.
    let at = search.client.transport.calls + 1;
    search.client.transport.fail_at = Some((at, Status::Io));
    assert_eq!(search.meta(1, 1, 0, 0, 1), Err(Error::Volume(Status::Io)));
    assert_eq!(
        search.query(b"work").map(|_| ()),
        Err(Error::RebuildRequired)
    );
    search.client.transport.fail_at = None;
    // A successful incremental rename must not clear the pending rebuild.
    assert_eq!(search.rename(5, 1, b"other"), Ok(2));
    assert_eq!(
        search.query(b"work").map(|_| ()),
        Err(Error::RebuildRequired)
    );
    search.rebuild().unwrap();
    assert_eq!(find(&search, "work"), vec![(0, 2), (1, 2)]);
}

/// I6: a snapshot no newer than the last applied one is refused.
#[test]
fn b3_stale_generation_is_refused_and_keeps_the_index() {
    let mut search = fixture();
    assert_eq!(search.rebuild(), Err(Error::StaleGeneration));
    // Model a replayed older snapshot: the last applied one is newer.
    let newer = generation(&search) + 2;
    search.generation = Some(newer);
    search.rename(0, 2, b"zulu").unwrap();
    assert!(generation(&search) < newer);
    assert_eq!(search.rebuild(), Err(Error::StaleGeneration));
    assert_eq!(find(&search, "zulu"), vec![(0, 3)]);
}

/// N7: a full domain of PER_DOMAIN objects with deletes and a re-add stays
/// within E = PER_DOMAIN, because tombstones reuse their object's slot.
#[test]
fn n7_full_domain_with_tombstones_and_re_add() {
    let mut search = fixture();
    write(&mut search, 6, 0, b"six");
    write(&mut search, 7, 0, b"seven");
    search.rebuild().unwrap();
    assert_eq!(find(&search, "seven"), vec![(7, 1)]);
    assert_eq!(search.delete(6, 1), Ok(2));
    assert_eq!(search.delete(7, 1), Ok(2));
    search.rebuild().unwrap();
    let revision = write(&mut search, 6, 2, b"six again");
    assert!(revision > 2);
    search.rebuild().unwrap();
    assert_eq!(find(&search, "six"), vec![(6, revision)]);
    assert_eq!(find(&search, "seven"), vec![]);
    assert_eq!(find(&search, "kelvin\u{212a}"), vec![(5, 1)]);
}

/// I2/N9 (host): each domain's adapter only ever sees its own LIST.
#[test]
fn domains_see_only_their_own_records() {
    let mut a = fixture();
    a.rename(0, 2, b"alpha0").unwrap();
    let mut b = rebind(a, 1);
    write(&mut b, 8, 0, b"beta1");
    b.rebuild().unwrap();
    assert_eq!(find(&b, "beta1"), vec![(8, 1)]);
    assert_eq!(find(&b, "alpha0"), vec![]);
    assert_eq!(find(&b, "meeting"), vec![]);
    assert!(matches!(b.client.stat(0), Err((Status::Access, _))));
    let mut a = rebind(b, 0);
    a.rebuild().unwrap();
    assert_eq!(find(&a, "alpha0"), vec![(0, 3)]);
    assert_eq!(find(&a, "beta1"), vec![]);
}

#[test]
fn digest_is_order_independent_and_revision_bound() {
    let search = fixture();
    let all = search.query(b"meeting").unwrap();
    assert_ne!(digest(DIGEST_START, &all), DIGEST_START);
    let none = search.query(b"nothing").unwrap();
    assert_eq!(digest(DIGEST_START, &none), DIGEST_START);
}

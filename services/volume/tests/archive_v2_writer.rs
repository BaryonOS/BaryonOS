//! Host-only tests of the S09 archive v2 writer (`archive::export_v2`) over
//! the real in-process volume service.
//!
//! Differential check: the archives exported from a fixed scenario are
//! committed under `tests/fixtures/archive-v2-written/` together with a
//! manifest built from the source *table* (not from the writer's output).
//! This test requires byte equality with the committed files, and
//! `tests/test_volume_archive_v2_written.py` requires the independent Python
//! reader (`tools/volume_archive.py` `parse_v2`) to accept every archive with
//! exactly the manifest's values. Regenerate after a deliberate change:
//! `NOS_WRITE_FIXTURES=1 cargo +1.100.0 test --locked -p volume-core --test archive_v2_writer`.
mod common;

use common::{fill, Local, SimDisk};
use volume_core::{
    archive::{self, ArchiveV2, DiskSink, DiskSource, ExportV2, Sink, Source},
    client::{Client, Transport},
    service::Service,
    table::{Entry, Table},
    wire, Status, COLLECTIONS, MAX_LENGTH, PER_DOMAIN,
};

const EPOCH: u64 = 6;

struct Bytes(Vec<u8>);
impl Sink for Bytes {
    fn put(&mut self, bytes: &[u8]) -> Result<(), Status> {
        self.0.extend_from_slice(bytes);
        Ok(())
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl Source for Reader<'_> {
    fn get(&mut self, buffer: &mut [u8]) -> Result<usize, Status> {
        let amount = buffer.len().min(self.bytes.len() - self.at).min(77);
        buffer[..amount].copy_from_slice(&self.bytes[self.at..self.at + amount]);
        self.at += amount;
        Ok(amount)
    }
}
fn reader(bytes: &[u8]) -> Reader<'_> {
    Reader { bytes, at: 0 }
}

fn scratch() -> Box<[u8; MAX_LENGTH]> {
    vec![0u8; MAX_LENGTH].into_boxed_slice().try_into().unwrap()
}

fn client<'a>(service: &'a mut Service, disk: &'a mut SimDisk, domain: usize) -> Client<Local<'a>> {
    Client::new(
        Local {
            service,
            disk,
            domain,
        },
        EPOCH,
        100 + 1000 * domain as u64,
    )
}

/// History, renames, META, deletes, purges, a RESTORE, collections (one
/// deleted) in domain 1, interleaved with commits of domain 0, so the global
/// generation differs from either domain's clock and generation.
fn scenario() -> (Service, SimDisk) {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let o = |slot: usize| (PER_DOMAIN + slot) as u64;
    {
        let mut a = client(&mut service, &mut disk, 0);
        a.write(0, 0, b"Nul", 40, fill(0, 1)).unwrap();
        a.collection(0, 2, 0, b"Thuis").unwrap();
    }
    {
        let mut c = client(&mut service, &mut disk, 1);
        c.collection(1, 1, 0, b"Werk").unwrap();
        c.collection(1, 2, 0, b"Oud").unwrap();
        c.collection(1, 3, 0, b"Weg").unwrap();
        c.collection(1, 3, 1, b"").unwrap();
        // Slot 0: three consecutive versions, then META (no version).
        c.write(o(0), 0, b"Plan", 10, fill(o(0), 1)).unwrap();
        c.write(o(0), 1, b"Plan v2", 300, fill(o(0), 2)).unwrap();
        c.write(o(0), 2, b"Plan v3", 5000, fill(o(0), 3)).unwrap();
        c.meta(o(0), 3, 0x8001, 7, 1).unwrap();
        // Slot 1: RENAME then COMMIT leaves a gap; the older version is omitted.
        c.write(o(1), 0, b"a", 20, fill(o(1), 1)).unwrap();
        c.rename(o(1), 1, b"b").unwrap();
        c.write(o(1), 2, b"c", 21, fill(o(1), 3)).unwrap();
        // Slot 2: deleted (tombstone): a content-free record only.
        c.write(o(2), 0, b"weg", 30, fill(o(2), 1)).unwrap();
        c.delete(o(2), 1).unwrap();
        // Slot 3: deleted and purged.
        c.write(o(3), 0, b"gewist", 31, fill(o(3), 1)).unwrap();
        c.delete(o(3), 1).unwrap();
        c.purge(o(3), 2).unwrap();
        // Slot 4: DELETE then COMMIT: the deleted version is never exported.
        c.write(o(4), 0, b"plan.txt", 32, fill(o(4), 1)).unwrap();
        c.delete(o(4), 1).unwrap();
        c.write(o(4), 2, b"nieuw", 33, fill(o(4), 3)).unwrap();
        // Slot 5: RESTORE keeps a consecutive chain.
        c.write(o(5), 0, b"r1", 34, fill(o(5), 1)).unwrap();
        c.write(o(5), 1, b"r2", 35, fill(o(5), 2)).unwrap();
        c.restore(o(5), 2, 1).unwrap();
        // Slot 6: in collection 2, empty content.
        c.write(o(6), 0, b"leeg", 0, fill(o(6), 1)).unwrap();
        c.meta(o(6), 1, 0, 0, 2).unwrap();
    }
    client(&mut service, &mut disk, 0)
        .write(1, 0, b"Een", 41, fill(1, 1))
        .unwrap();
    (service, disk)
}

fn export(service: &mut Service, disk: &mut SimDisk, domain: usize) -> (ExportV2, Vec<u8>) {
    let mut sink = Bytes(Vec::new());
    let result = archive::export_v2(&mut client(service, disk, domain), domain, &mut sink).unwrap();
    (result, sink.0)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Versions L1 may carry: current first, then history while each step is 1.
fn prefix(entry: &Entry) -> Vec<&volume_core::table::Version> {
    let mut out = vec![&entry.current];
    for version in &entry.history[..entry.history_count] {
        if version.revision + 1 != out.last().unwrap().revision {
            break;
        }
        out.push(version);
    }
    out
}

/// The manifest lines (format of `tests/test_volume_archive_v2_fixtures.py`)
/// expected for an export of `domain`, derived from the source table only.
fn expected(name: &str, table: &Table, domain: usize, length: usize) -> Vec<String> {
    let entries = &table.entries[domain * PER_DOMAIN..(domain + 1) * PER_DOMAIN];
    let mut versions = Vec::new();
    for (slot, entry) in entries.iter().enumerate() {
        if entry.present {
            versions.push((slot, prefix(entry)));
        }
    }
    let content: usize = versions
        .iter()
        .flat_map(|(_, v)| v.iter().map(|v| v.length))
        .sum();
    let mut out = vec![format!(
        "{name} archive 0 1 {} {} {content} {length}",
        table.domain_generation(domain),
        table.clocks[domain]
    )];
    for index in 1..=COLLECTIONS {
        let c = table.collection(domain, index);
        if c.revision != 0 {
            let name_hex = if c.present { hex(c.name()) } else { "-".into() };
            out.push(format!(
                "{name} collection {index} {} {} {name_hex}",
                u8::from(c.present),
                c.revision
            ));
        }
    }
    for (slot, entry) in entries.iter().enumerate() {
        if entry.revision == 0 {
            continue;
        }
        let kept = if entry.present { prefix(entry) } else { vec![] };
        out.push(format!(
            "{name} object {slot} {} {} {} {} {} {} {} {}",
            u8::from(entry.present),
            entry.collection,
            entry.kind,
            entry.tags,
            entry.revision,
            entry.created,
            entry.modified,
            kept.len()
        ));
        for (i, v) in kept.iter().enumerate() {
            out.push(format!(
                "{name} version {slot} {i} {} {} {} {}",
                v.revision,
                v.length,
                v.content_sum,
                hex(v.name())
            ));
        }
    }
    out
}

fn fixture_dir() -> String {
    format!(
        "{}/tests/fixtures/archive-v2-written",
        env!("CARGO_MANIFEST_DIR")
    )
}

#[test]
fn exported_archives_verify_and_match_the_committed_differential_fixtures() {
    let (mut service, mut disk) = scenario();
    let table = *service.table();
    assert_ne!(table.generation, table.domain_generation(1));
    let mut manifest = Vec::new();
    let mut files = Vec::new();
    for domain in 0..2 {
        let name = format!("domain{domain}.bin");
        let (result, bytes) = export(&mut service, &mut disk, domain);
        let verified = archive::verify_v2(&mut reader(&bytes), &mut scratch()).unwrap();
        assert_eq!(verified, result.archive);
        assert!(verified.accounted && !verified.tombstones);
        assert_eq!(bytes.len() as u64, {
            let a = &verified;
            let versions: usize = a.objects[..a.object_count]
                .iter()
                .map(|o| o.version_count)
                .sum();
            (archive::HEADER_V2
                + archive::TRAILER
                + a.collection_count * archive::COLLECTION_RECORD_V2
                + a.object_count * archive::OBJECT_RECORD_V2
                + versions * archive::VERSION_RECORD_V2) as u64
                + a.content_bytes
        });
        assert!(bytes.len() as u64 <= archive::MAX_ARCHIVE_V2);
        manifest.extend(expected(&name, &table, domain, bytes.len()));
        files.push((name, bytes));
    }
    // Slot 1 (RENAME gap) and slot 4 (DELETE gap) each lose one version.
    let (result, _) = export(&mut service, &mut disk, 1);
    assert_eq!(result.omitted, 2);
    assert_eq!(service.table(), &table, "export mutates nothing");
    let text = manifest.join("\n") + "\n";
    let dir = fixture_dir();
    if std::env::var_os("NOS_WRITE_FIXTURES").is_some() {
        std::fs::create_dir_all(&dir).unwrap();
        for (name, bytes) in &files {
            std::fs::write(format!("{dir}/{name}"), bytes).unwrap();
        }
        std::fs::write(format!("{dir}/manifest.txt"), &text).unwrap();
    }
    for (name, bytes) in &files {
        assert_eq!(
            &std::fs::read(format!("{dir}/{name}")).unwrap(),
            bytes,
            "{name}"
        );
    }
    assert_eq!(
        std::fs::read_to_string(format!("{dir}/manifest.txt")).unwrap(),
        text
    );
}

#[test]
fn round_trip_into_a_fresh_domain_of_a_fresh_volume() {
    let (mut service, mut disk) = scenario();
    let (result, bytes) = export(&mut service, &mut disk, 1);
    let plan = result.archive;
    let mut target_disk = SimDisk::new();
    let mut target = Service::new(EPOCH, Table::EMPTY, false);
    for domain in 0..2 {
        let mut d = client(&mut target, &mut target_disk, domain);
        let mut recorded = ArchiveV2::default();
        let report = archive::restore_native(
            || reader(&bytes),
            &mut scratch(),
            &mut d,
            domain,
            &mut recorded,
        )
        .unwrap();
        assert_eq!(recorded, plan, "source values reported unchanged");
        assert_eq!(report.skipped, 0, "the writer exports the prefix only");
        assert_eq!(report.restored, 0b0111_0011);
        for o in plan.objects[..plan.object_count]
            .iter()
            .filter(|o| o.present)
        {
            let source_object = (PER_DOMAIN + o.slot) as u64;
            let object = (domain * PER_DOMAIN + o.slot) as u64;
            let stat = d.stat(object).unwrap();
            assert_eq!(
                (stat.tags, stat.kind, stat.collection),
                (o.tags, o.kind, o.collection)
            );
            assert_eq!(stat.history_count + 1, o.version_count);
            let source_stat = client(&mut service, &mut disk, 1)
                .stat(source_object)
                .unwrap();
            for (selector, version) in o.versions[..o.version_count].iter().enumerate() {
                let name = if selector == 0 {
                    stat.name[..stat.name_length].to_vec()
                } else {
                    d.version_stat(object, stat.revision, selector)
                        .unwrap()
                        .name()
                        .to_vec()
                };
                assert_eq!(name, version.name());
                let read = |c: &mut Client<Local<'_>>, object, revision| {
                    let mut out = vec![0; version.length];
                    c.read_version(object, revision, selector, version.length, |at, chunk| {
                        out[at..at + chunk.len()].copy_from_slice(chunk)
                    })
                    .unwrap();
                    out
                };
                let source = read(
                    &mut client(&mut service, &mut disk, 1),
                    source_object,
                    source_stat.revision,
                );
                assert_eq!(read(&mut d, object, stat.revision), source);
            }
        }
        // Collections: present ones recreated with their names; deleted not.
        for c in &plan.collections[..plan.collection_count] {
            let info = d.collection_stat(domain, c.index).unwrap();
            assert_eq!((info.present, info.name()), (c.present, c.name()));
        }
    }
}

/// Old service: refuses the new operations with NoSys before reading state.
struct Old<'a>(Local<'a>);
impl Transport for Old<'_> {
    fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
        let header = wire::header(request);
        if header.operation >= wire::DOMAIN_STAT {
            return Ok(wire::error_reply(&header, EPOCH, Status::NoSys, 0));
        }
        self.0.call(request)
    }
}

#[test]
fn an_old_service_gets_no_bytes_written() {
    let (mut service, mut disk) = scenario();
    let mut old = Client::new(
        Old(Local {
            service: &mut service,
            disk: &mut disk,
            domain: 1,
        }),
        EPOCH,
        5,
    );
    let mut sink = Bytes(Vec::new());
    assert_eq!(
        archive::export_v2(&mut old, 1, &mut sink),
        Err(Status::NoSys)
    );
    assert!(sink.0.is_empty());
}

/// Another writer of the same domain commits right before request `at`.
struct Racing<'a> {
    inner: Local<'a>,
    calls: usize,
    at: usize,
}
impl Transport for Racing<'_> {
    fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
        self.calls += 1;
        if self.calls == self.at {
            let object = PER_DOMAIN as u64;
            let revision = self.inner.service.table().entries[object as usize].revision;
            client(self.inner.service, self.inner.disk, 1)
                .write(object, revision, b"Plan v4", 7, fill(object, 9))
                .unwrap();
        }
        self.inner.call(request)
    }
}

#[test]
fn a_concurrent_own_commit_discards_the_export() {
    let (mut service, mut disk) = scenario();
    let counting = Racing {
        inner: Local {
            service: &mut service,
            disk: &mut disk,
            domain: 1,
        },
        calls: 0,
        at: 0,
    };
    let mut c = Client::new(counting, EPOCH, 5);
    archive::export_v2(&mut c, 1, &mut Bytes(Vec::new())).unwrap();
    let total = c.transport.calls;
    for at in 1..=total {
        let (mut service, mut disk) = scenario();
        let racing = Racing {
            inner: Local {
                service: &mut service,
                disk: &mut disk,
                domain: 1,
            },
            calls: 0,
            at,
        };
        let mut sink = Bytes(Vec::new());
        let result = archive::export_v2(&mut Client::new(racing, EPOCH, 5), 1, &mut sink);
        if at == 1 {
            // Before the first DOMAIN_STAT: one consistent later state.
            result.unwrap();
            archive::verify_v2(&mut reader(&sink.0), &mut scratch()).unwrap();
        } else {
            assert_eq!(
                result.unwrap_err(),
                Status::Stale,
                "commit before call {at}"
            );
            assert!(archive::verify_v2(&mut reader(&sink.0), &mut scratch()).is_err());
        }
    }
}

/// v1 and v2 archives of one domain, each written by `DiskSink` to its own
/// range, read back through the same `DiskSource` type: owned or borrowed,
/// every stored byte and nothing behind it.
#[test]
fn disk_source_reads_v1_and_v2_archives_of_one_domain() {
    let (mut service, mut disk) = scenario();
    let (v2, v2_bytes) = export(&mut service, &mut disk, 1);
    let mut v1_sink = Bytes(Vec::new());
    let v1 = archive::export(&mut client(&mut service, &mut disk, 1), 1, &mut v1_sink).unwrap();
    let v1_bytes = v1_sink.0;
    assert_eq!(v1_bytes[8], 1);
    assert_eq!(v2_bytes[8], 2);
    for bytes in [&v1_bytes, &v2_bytes] {
        let mut range = common::SimDisk::new();
        let capacity = (bytes.len() / volume_core::SECTOR + 3) as u64;
        let mut sink = DiskSink::new(&mut range, capacity);
        sink.put(bytes).unwrap();
        assert_eq!(sink.finish(), Ok(bytes.len() as u64));
        let mut read = Vec::new();
        let mut source = DiskSource::new(&mut range, capacity);
        let mut piece = [0; 333];
        loop {
            match source.get(&mut piece).unwrap() {
                0 => break,
                amount => read.extend_from_slice(&piece[..amount]),
            }
        }
        assert_eq!(&read, bytes);
    }
    let mut range = common::SimDisk::new();
    let capacity = (v1_bytes.len() / volume_core::SECTOR + 1) as u64;
    let mut sink = DiskSink::new(&mut range, capacity);
    sink.put(&v1_bytes).unwrap();
    sink.finish().unwrap();
    assert_eq!(
        archive::verify(&mut DiskSource::new(range, capacity), &mut scratch()),
        Ok(v1)
    );
    let mut range = common::SimDisk::new();
    let capacity = (v2_bytes.len() / volume_core::SECTOR + 1) as u64;
    let mut sink = DiskSink::new(&mut range, capacity);
    sink.put(&v2_bytes).unwrap();
    sink.finish().unwrap();
    assert_eq!(
        archive::verify_v2(&mut DiskSource::new(range, capacity), &mut scratch()),
        Ok(v2.archive)
    );
}

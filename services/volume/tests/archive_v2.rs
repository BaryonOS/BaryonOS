//! Host-only S09 archive v2 tests: the Rust reader against the fixtures and
//! manifest produced by the independent Python oracle
//! (`tests/test_volume_archive_v2_fixtures.py`), and L1 `restore_native`
//! through the real in-process volume service. No v2 encoder exists here.
mod common;

use common::{recover, Local, SimDisk};
use volume_core::{
    archive::{self, ArchiveV2, DiskSource, NativeReport, Source, MAX_ARCHIVE_V2},
    checksum_update,
    client::{Client, Transport},
    service::Service,
    table::Table,
    upload, wire, Disk, Status, CHUNK, FNV_OFFSET, MAX_LENGTH, PER_DOMAIN, SECTOR,
};

const EPOCH: u64 = 6;
const DOMAIN: usize = 1;

fn fixture(name: &str) -> Vec<u8> {
    let path = format!(
        "{}/tests/fixtures/archive-v2/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read(&path).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// Serves the archive in uneven pieces to exercise reassembly.
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

fn verify(bytes: &[u8]) -> Result<ArchiveV2, Status> {
    archive::verify_v2(&mut reader(bytes), &mut scratch())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// The manifest lines the Python oracle wrote, rebuilt from the Rust reader.
fn lines(name: &str, a: &ArchiveV2) -> Vec<String> {
    let mut out = vec![format!(
        "{name} archive {} {} {} {} {} {}",
        u8::from(a.tombstones),
        u8::from(a.accounted),
        a.generation,
        a.clock,
        a.content_bytes,
        fixture(name).len()
    )];
    for c in &a.collections[..a.collection_count] {
        let name_hex = if c.name_length == 0 {
            "-".into()
        } else {
            hex(c.name())
        };
        out.push(format!(
            "{name} collection {} {} {} {name_hex}",
            c.index,
            u8::from(c.present),
            c.revision
        ));
    }
    for o in &a.objects[..a.object_count] {
        out.push(format!(
            "{name} object {} {} {} {} {} {} {} {} {}",
            o.slot,
            u8::from(o.present),
            o.collection,
            o.kind,
            o.tags,
            o.revision,
            o.created,
            o.modified,
            o.version_count
        ));
        for (i, v) in o.versions[..o.version_count].iter().enumerate() {
            out.push(format!(
                "{name} version {} {i} {} {} {} {}",
                o.slot,
                v.revision,
                v.length,
                v.content_sum,
                hex(v.name())
            ));
        }
    }
    out
}

#[test]
fn reader_agrees_with_the_oracle_manifest() {
    let manifest = String::from_utf8(fixture("manifest.txt")).unwrap();
    let mut names: Vec<&str> = manifest
        .lines()
        .map(|l| l.split(' ').next().unwrap())
        .collect();
    names.dedup();
    let mut refused = 0;
    for name in names {
        let expected: Vec<&str> = manifest
            .lines()
            .filter(|l| l.split(' ').next() == Some(name))
            .collect();
        let bytes = fixture(name);
        if expected[0].starts_with(&format!("{name} refused ")) {
            assert_eq!(verify(&bytes), Err(Status::Corrupt), "{name}");
            refused += 1;
            continue;
        }
        let parsed = verify(&bytes).unwrap_or_else(|s| panic!("{name}: {s:?}"));
        assert_eq!(lines(name, &parsed), expected, "{name}");
        // Content is handed out only after its checksum matched the record.
        let mut seen = 0;
        archive::walk_v2(&mut reader(&bytes), &mut scratch(), |o, i, content| {
            assert_eq!(content.len(), o.versions[i].length);
            assert_eq!(
                checksum_update(FNV_OFFSET, content),
                o.versions[i].content_sum
            );
            seen += 1;
            Ok(())
        })
        .unwrap();
        let versions: usize = parsed.objects[..parsed.object_count]
            .iter()
            .map(|o| o.version_count)
            .sum();
        assert_eq!(seen, versions, "{name}");
    }
    let listed = manifest.lines().filter(|l| l.contains(" refused ")).count();
    assert_eq!(refused, listed);
    assert!(refused >= 60, "{refused}");
    // v1 stays v1: the v1 reader accepts it, the v2 reader refuses it, and
    // the v1 reader refuses every v2 archive.
    assert!(archive::verify(&mut reader(&fixture("v1.bin")), &mut scratch()).is_ok());
    assert_eq!(
        archive::verify(&mut reader(&fixture("tombstones.bin")), &mut scratch()),
        Err(Status::Corrupt)
    );
}

#[test]
fn every_bit_flip_truncation_and_trailing_byte_is_refused() {
    for name in ["tombstones.bin", "restore.bin"] {
        let bytes = fixture(name);
        for end in 0..bytes.len() {
            assert!(verify(&bytes[..end]).is_err(), "{name} truncated at {end}");
        }
        let mut longer = bytes.clone();
        longer.push(0);
        assert_eq!(verify(&longer), Err(Status::Corrupt));
    }
    let bytes = fixture("tombstones.bin");
    for at in 0..bytes.len() {
        for bit in 0..8 {
            let mut data = bytes.clone();
            data[at] ^= 1 << bit;
            assert!(verify(&data).is_err(), "flip at {at} bit {bit}");
        }
    }
}

/// Counts BEGINs (fresh writes) and all mutating requests, and can fail the
/// n-th BEGIN, or every COLLECTION, like a dead link.
struct Watch<T> {
    inner: T,
    begins: usize,
    mutations: usize,
    fail_begin: Option<usize>,
    fail_collection: bool,
}
impl<T: Transport> Transport for Watch<T> {
    fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
        let operation = wire::header(request).operation;
        if [
            wire::BEGIN,
            wire::APPEND,
            wire::COMMIT,
            wire::ABORT,
            wire::DELETE,
            wire::RENAME,
            wire::RESTORE,
            wire::PURGE,
            wire::META,
            wire::COLLECTION,
            wire::TRANSACT,
        ]
        .contains(&operation)
        {
            self.mutations += 1;
        }
        if operation == wire::COLLECTION && self.fail_collection {
            return Err(Status::Io);
        }
        if operation == wire::BEGIN {
            self.begins += 1;
            if self.fail_begin == Some(self.begins) {
                return Err(Status::Io);
            }
        }
        self.inner.call(request)
    }
}

fn client<'a>(
    service: &'a mut Service,
    disk: &'a mut SimDisk,
    domain: usize,
    fail_begin: Option<usize>,
) -> Client<Watch<Local<'a>>> {
    Client::new(
        Watch {
            inner: Local {
                service,
                disk,
                domain,
            },
            begins: 0,
            mutations: 0,
            fail_begin,
            fail_collection: false,
        },
        EPOCH,
        100 + 1000 * domain as u64,
    )
}

fn restore<T: Transport>(
    bytes: &[u8],
    c: &mut Client<T>,
) -> Result<(NativeReport, ArchiveV2), (Status, NativeReport)> {
    let mut source = ArchiveV2::default();
    archive::restore_native(|| reader(bytes), &mut scratch(), c, DOMAIN, &mut source)
        .map(|report| (report, source))
}

fn read_version<T: Transport>(
    c: &mut Client<T>,
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

/// Every version's content in archive order (current first), per slot.
fn contents(bytes: &[u8]) -> Vec<Vec<Vec<u8>>> {
    let mut out = vec![Vec::new(); PER_DOMAIN];
    archive::walk_v2(&mut reader(bytes), &mut scratch(), |o, _, content| {
        out[o.slot].push(content.to_vec());
        Ok(())
    })
    .unwrap();
    out
}

#[test]
fn restore_native_replays_history_into_a_fresh_domain() {
    let bytes = fixture("restore.bin");
    let source = verify(&bytes).unwrap();
    let content = contents(&bytes);
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN, None);
    let (report, recorded) = restore(&bytes, &mut c).unwrap();
    assert_eq!(recorded, source);
    // Slots 1, 3 and 4 are present; slot 7 is a content-free record.
    assert_eq!(report.restored, 0b0001_1010);
    assert_eq!(report.writes, 6);
    // Six WRITEs, two COLLECTIONs, two METAs.
    assert_eq!(report.committed, 10);
    // Every chain in restore.bin is consecutive: the full history is replayed.
    assert_eq!(report.skipped, 0);
    assert_eq!(c.transport.begins, 6);

    // Collections: present ones at their own index, the deleted one not replayed.
    for collection in &source.collections[..source.collection_count] {
        let info = c.collection_stat(DOMAIN, collection.index).unwrap();
        assert_eq!(info.present, collection.present);
        assert_eq!(info.name(), collection.name());
        if collection.present {
            assert_eq!(info.revision, 1, "fresh destination revision");
            assert_ne!(info.revision, collection.revision);
        } else {
            assert_eq!(info.revision, 0, "deleted collection not replayed");
        }
    }
    assert_eq!(c.collection_stat(DOMAIN, 1).unwrap().revision, 0);

    for (slot, archived) in content.iter().enumerate() {
        let object = (DOMAIN * PER_DOMAIN + slot) as u64;
        let Some(o) = source.object(slot).filter(|o| o.present) else {
            // Absent, never written: content-free records are not replayed.
            assert_eq!(c.stat(object).unwrap_err(), (Status::NotFound, 0));
            continue;
        };
        let stat = c.stat(object).unwrap();
        let current = &o.versions[0];
        assert_eq!(&stat.name[..stat.name_length], current.name());
        assert_eq!(stat.length, current.length);
        assert_eq!(stat.content_sum, current.content_sum);
        assert_eq!(
            (stat.tags, stat.kind, stat.collection),
            (o.tags, o.kind, o.collection)
        );
        assert_eq!(stat.history_count, o.version_count - 1);
        // One WRITE per version plus META when metadata is set: fresh values,
        // never the recorded source revisions or stamps.
        let meta = u64::from(o.tags != 0 || o.kind != 0 || o.collection != 0);
        assert_eq!(stat.revision, o.version_count as u64 + meta);
        assert_ne!(stat.revision, o.revision);
        assert_ne!(stat.version_revision, current.revision);
        assert!(stat.created < o.created && stat.modified < o.modified);
        assert!(stat.modified <= 2 * report.writes as u64);
        assert_eq!(
            read_version(&mut c, object, stat.revision, 0, stat.length),
            archived[0]
        );
        // History newest first, exactly the archive's order and content.
        for (i, history) in stat.history[..stat.history_count].iter().enumerate() {
            let recorded = &o.versions[i + 1];
            assert_eq!(history.length, recorded.length);
            assert_eq!(history.content_sum, recorded.content_sum);
            assert_ne!(history.revision, recorded.revision);
            assert_eq!(
                read_version(&mut c, object, stat.revision, i + 1, history.length),
                archived[i + 1]
            );
        }
    }
    // The durable table carries the history names (VERSION_STAT also does).
    let table = recover(disk.settled()).unwrap();
    for o in source.objects[..source.object_count]
        .iter()
        .filter(|o| o.present)
    {
        let entry = &table.entries[DOMAIN * PER_DOMAIN + o.slot];
        let names: Vec<&[u8]> = core::iter::once(entry.current.name())
            .chain(
                entry.history[..entry.history_count]
                    .iter()
                    .map(|v| v.name()),
            )
            .collect();
        let recorded: Vec<&[u8]> = o.versions[..o.version_count]
            .iter()
            .map(|v| v.name())
            .collect();
        assert_eq!(names, recorded);
        assert!(entry.history[..entry.history_count]
            .windows(2)
            .all(|w| w[0].revision > w[1].revision));
    }
    // The domain is now used: a second restore is refused, nothing merged.
    let mut c = client(&mut service, &mut disk, DOMAIN, None);
    assert_eq!(restore(&bytes, &mut c).unwrap_err().0, Status::Stale);
    assert_eq!(c.transport.begins, 0);
}

#[test]
fn restore_native_leaves_the_other_domain_alone() {
    let bytes = fixture("present-only.bin");
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut other = client(&mut service, &mut disk, 0, None);
    let (revision, _) = other.write(0, 0, b"Eigen", 3, |_, b| b.fill(9)).unwrap();
    let mut c = client(&mut service, &mut disk, DOMAIN, None);
    // Usage of domain 0 does not make domain 1 used.
    assert!(archive::never_used(&mut c, DOMAIN).unwrap());
    restore(&bytes, &mut c).unwrap();
    let mut other = client(&mut service, &mut disk, 0, None);
    assert_eq!(other.stat(0).unwrap().revision, revision);
    assert!(!archive::never_used(&mut other, 0).unwrap());
}

fn refused_without_writes(
    bytes: &[u8],
    status: Status,
    prepare: impl FnOnce(&mut Client<Watch<Local<'_>>>),
) {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN, None);
    prepare(&mut c);
    // Baseline after `prepare`: its deliberate writes are not the restore's.
    let mutations = c.transport.mutations;
    let before = archive::never_used(&mut c, DOMAIN).unwrap();
    let (got, report) = restore(bytes, &mut c).unwrap_err();
    assert_eq!(got, status);
    // Only `skipped` (known once the archive is verified) may be non-zero.
    assert_eq!(
        report,
        NativeReport {
            skipped: report.skipped,
            ..NativeReport::default()
        }
    );
    assert_eq!(
        c.transport.mutations, mutations,
        "no mutating request (BEGIN, COLLECTION, META, ...) was sent"
    );
    // A refusal leaves the domain exactly as it was.
    assert_eq!(archive::never_used(&mut c, DOMAIN).unwrap(), before);
}

#[test]
fn restore_native_refuses_a_used_domain() {
    let bytes = fixture("present-only.bin");
    let object = (DOMAIN * PER_DOMAIN + 5) as u64;
    // Written, deleted and purged: absent again, but not never-used.
    refused_without_writes(&bytes, Status::Stale, |c| {
        let (revision, _) = c.write(object, 0, b"Weg", 4, |_, b| b.fill(1)).unwrap();
        let revision = c.delete(object, revision).unwrap();
        c.purge(object, revision).unwrap();
        assert_eq!(c.stat(object).unwrap_err().0, Status::NotFound);
        assert_ne!(c.stat(object).unwrap_err().1, 0);
    });
    // A collection created and deleted again.
    refused_without_writes(&bytes, Status::Stale, |c| {
        let revision = c.collection(DOMAIN, 3, 0, b"Kort").unwrap();
        c.collection(DOMAIN, 3, revision, b"").unwrap();
        assert!(!c.collection_stat(DOMAIN, 3).unwrap().present);
    });
    // A present object.
    refused_without_writes(&bytes, Status::Stale, |c| {
        c.write(object, 0, b"Er", 1, |_, b| b.fill(1)).unwrap();
    });
}

#[test]
fn restore_native_refuses_tombstones_corruption_and_v1() {
    // R2: an archive with TOMBSTONES set is refused even into a fresh domain.
    for (name, status) in [
        ("tombstones.bin", Status::Invalid),
        ("s9-generation.bin", Status::Corrupt),
        ("v1.bin", Status::Corrupt),
        ("over-quota.bin", Status::Quota),
        ("full.bin", Status::Full),
    ] {
        refused_without_writes(&fixture(name), status, |c| {
            assert!(archive::never_used(c, DOMAIN).unwrap())
        });
    }
    // ACCOUNTED content-free records and deleted collections are accepted but not replayed.
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN, None);
    let (report, recorded) = restore(&fixture("accounted.bin"), &mut c).unwrap();
    assert_eq!(report.restored, 0b0000_0101);
    assert!(recorded.accounted);
    assert_eq!(
        c.stat((DOMAIN * PER_DOMAIN + 6) as u64).unwrap_err(),
        (Status::NotFound, 0)
    );
    assert_eq!(c.collection_stat(DOMAIN, 2).unwrap().revision, 0);
}

#[test]
fn restore_native_checks_the_domain_quota_before_the_first_write() {
    let bytes = fixture("over-quota.bin");
    let source = verify(&bytes).unwrap();
    let needed: usize = source.objects[..source.object_count]
        .iter()
        .flat_map(|o| o.versions[..o.version_count].iter())
        .map(|v| v.length.div_ceil(CHUNK))
        .sum();
    assert_eq!(needed, volume_core::QUOTA + 1);
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN, None);
    let (status, report) = restore(&bytes, &mut c).unwrap_err();
    assert_eq!(
        (status, report.writes, c.transport.begins),
        (Status::Quota, 0, 0)
    );
    // Nothing was committed, so the domain is still never used.
    assert!(archive::never_used(&mut c, DOMAIN).unwrap());
}

fn check_retry_refused(service: &mut Service, disk: &mut SimDisk, bytes: &[u8]) {
    let mut c = client(service, disk, DOMAIN, None);
    assert!(!archive::never_used(&mut c, DOMAIN).unwrap());
    let (status, report) = restore(bytes, &mut c).unwrap_err();
    assert_eq!(
        (status, report.committed, c.transport.begins),
        (Status::Stale, 0, 0)
    );
    assert!(!archive::never_used(&mut c, DOMAIN).unwrap());
}

#[test]
fn a_failed_replay_leaves_the_domain_used_and_a_retry_is_refused() {
    let bytes = fixture("restore.bin");
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN, Some(3));
    let (status, report) = restore(&bytes, &mut c).unwrap_err();
    assert_eq!(
        (status, report.writes, report.committed, report.restored),
        (Status::Io, 2, 2, 0)
    );
    check_retry_refused(&mut service, &mut disk, &bytes);
}

#[test]
fn a_slot_without_metadata_is_reported_restored_when_a_collection_fails() {
    // restore.bin: slot 3 needs one WRITE and no META; slots 1 and 4 need META.
    let bytes = fixture("restore.bin");
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN, None);
    c.transport.fail_collection = true;
    let (status, report) = restore(&bytes, &mut c).unwrap_err();
    assert_eq!(
        (status, report.writes, report.committed, report.restored),
        (Status::Io, 6, 6, 1 << 3)
    );
}

#[test]
fn a_slot_without_metadata_is_reported_restored_when_a_later_write_fails() {
    // restore.bin replays 6 versions, current versions last in slot order
    // 1, 3, 4: the 6th BEGIN (slot 4) fails after slot 3's only WRITE.
    let bytes = fixture("restore.bin");
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN, Some(6));
    let (status, report) = restore(&bytes, &mut c).unwrap_err();
    assert_eq!(
        (status, report.writes, report.committed, report.restored),
        (Status::Io, 5, 5, 1 << 3)
    );
}

#[test]
fn replay_beyond_the_guaranteed_free_chunks_is_refused_before_any_write() {
    // The volume-wide free count is not observable; every other domain stays
    // within QUOTA, so only CHUNKS - QUOTA = 24 chunks are guaranteed free.
    // full.bin needs 25: refused with Full before the first BEGIN, even with
    // the other domain empty, and the domain stays never used.
    let bytes = fixture("full.bin");
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN, None);
    let (status, report) = restore(&bytes, &mut c).unwrap_err();
    assert_eq!(
        (status, report.writes, report.restored),
        (Status::Full, 0, 0)
    );
    assert_eq!(archive::never_used(&mut c, DOMAIN), Ok(true));
}

#[test]
fn replay_within_the_guaranteed_free_chunks_succeeds_beside_a_full_domain() {
    // The other domain holds its whole quota (40 of 64 chunks).
    let bytes = fixture("restore.bin");
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut other = client(&mut service, &mut disk, 0, None);
    for (object, length) in [(0u64, MAX_LENGTH), (1, MAX_LENGTH), (2, 8 * CHUNK)] {
        other
            .write(object, 0, b"Vol", length, |_, b| b.fill(3))
            .unwrap();
    }
    let mut c = client(&mut service, &mut disk, DOMAIN, None);
    restore(&bytes, &mut c).unwrap();
    assert_eq!(archive::never_used(&mut c, DOMAIN), Ok(false));
}

/// A source that changes between passes: the replay must not act on bytes
/// the verify pass did not see.
#[test]
fn replay_passes_must_match_the_verified_archive() {
    let good = fixture("present-only.bin");
    let other = fixture("restore.bin");
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN, None);
    let mut opened = 0;
    let (status, report) = archive::restore_native(
        || {
            opened += 1;
            reader(if opened == 1 { &good } else { &other })
        },
        &mut scratch(),
        &mut c,
        DOMAIN,
        &mut ArchiveV2::default(),
    )
    .unwrap_err();
    assert_eq!((status, report.committed), (Status::Corrupt, 0));
    // The mismatch is found before the first BEGIN: the domain is untouched.
    assert_eq!(archive::never_used(&mut c, DOMAIN), Ok(true));
}

#[test]
fn a_refusal_reports_the_skipped_versions_of_the_verified_archive() {
    // delete-gap.bin: 6 versions follow a revision gap. A refused restore
    // (here an already used domain) reports them like a replay would.
    let bytes = fixture("delete-gap.bin");
    let object = (DOMAIN * PER_DOMAIN + 5) as u64;
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN, None);
    c.write(object, 0, b"Er", 1, |_, b| b.fill(1)).unwrap();
    let (status, report) = restore(&bytes, &mut c).unwrap_err();
    assert_eq!(
        (status, report.writes, report.committed, report.skipped),
        (Status::Stale, 0, 0, 6)
    );
}

#[test]
fn a_pass_differing_after_its_records_is_corrupt_after_its_writes() {
    // Later passes carry one trailing byte: every object record still
    // matches, so the first pass (depth 2: slot 1's oldest version) writes
    // before the end of the pass is found to differ.
    let good = fixture("restore.bin");
    let mut longer = good.clone();
    longer.push(0);
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN, None);
    let mut opened = 0;
    let (status, report) = archive::restore_native(
        || {
            opened += 1;
            reader(if opened == 1 { &good } else { &longer })
        },
        &mut scratch(),
        &mut c,
        DOMAIN,
        &mut ArchiveV2::default(),
    )
    .unwrap_err();
    assert_eq!(
        (status, report.writes, report.committed, report.restored),
        (Status::Corrupt, 1, 1, 0)
    );
    assert_eq!(archive::never_used(&mut c, DOMAIN), Ok(false));
}

/// Forwards every request but malforms the reply to the first COMMIT.
struct MalformedCommit<'a> {
    inner: Local<'a>,
    done: bool,
}
impl Transport for MalformedCommit<'_> {
    fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
        let mut reply = self.inner.call(request)?;
        if !self.done && wire::header(request).operation == wire::COMMIT {
            self.done = true;
            reply[2] ^= 1; // the reply's operation no longer matches
        }
        Ok(reply)
    }
}

#[test]
fn a_malformed_reply_to_a_landed_commit_is_a_lower_bound() {
    let bytes = fixture("restore.bin");
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = Client::new(
        MalformedCommit {
            inner: Local {
                service: &mut service,
                disk: &mut disk,
                domain: DOMAIN,
            },
            done: false,
        },
        EPOCH,
        100,
    );
    let (status, report) = restore(&bytes, &mut c).unwrap_err();
    assert_eq!(
        (status, report.writes, report.committed, report.restored),
        (Status::Protocol, 0, 0, 0)
    );
    // Yet the first WRITE (slot 1's oldest version) did commit.
    let object = (DOMAIN * PER_DOMAIN + 1) as u64;
    assert_eq!(c.stat(object).unwrap().revision, 1);
    assert_eq!(archive::never_used(&mut c, DOMAIN), Ok(false));
}

#[test]
fn a_staged_upload_in_the_destination_makes_the_first_begin_busy() {
    // Scenario B: a token-bound upload staged (not committed) in the
    // destination domain. The domain is still never used; the replay's first
    // mutation is a BEGIN, refused with Busy, so nothing commits.
    let bytes = fixture("present-only.bin");
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut begin = wire::Header::new(wire::BEGIN, 2, EPOCH, (DOMAIN * PER_DOMAIN + 3) as u64);
    begin.length = 4;
    begin.name_length = 4;
    begin.aux = 10;
    let response = service.handle_for(DOMAIN, 77, &upload::encode(&begin, b"Half", 0), &mut disk);
    let reply = upload::validate_reply(&begin, 0, &response, EPOCH).unwrap();
    assert_eq!(Status::from_code(i64::from(reply.status)), Status::Ok);
    let mut c = client(&mut service, &mut disk, DOMAIN, None);
    assert_eq!(archive::never_used(&mut c, DOMAIN), Ok(true));
    let (status, report) = restore(&bytes, &mut c).unwrap_err();
    assert_eq!(
        (status, report.writes, report.committed, report.restored),
        (Status::Busy, 0, 0, 0)
    );
    assert_eq!(archive::never_used(&mut c, DOMAIN), Ok(true));
    // Once the staged upload is gone the same restore succeeds.
    assert_eq!(service.detach(77), 1);
    let mut c = client(&mut service, &mut disk, DOMAIN, None);
    assert_eq!(restore(&bytes, &mut c).unwrap().0.restored, 0b0000_0101);
}

#[test]
fn a_collection_only_restore_is_not_refused_by_a_staged_upload() {
    // Pins the documented exception: with no version to replay there is no
    // BEGIN, COLLECTION is not refused by a staged upload, so the restore
    // succeeds and makes the domain used; the staged upload still commits.
    let bytes = fixture("collection-only.bin");
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let object = (DOMAIN * PER_DOMAIN + 3) as u64;
    let mut begin = wire::Header::new(wire::BEGIN, 2, EPOCH, object);
    begin.length = 4;
    begin.name_length = 4;
    begin.aux = 4;
    let response = service.handle_for(DOMAIN, 77, &upload::encode(&begin, b"Half", 0), &mut disk);
    let token = upload::token(&response);
    assert_ne!(token, 0);
    let mut c = client(&mut service, &mut disk, DOMAIN, None);
    assert_eq!(archive::never_used(&mut c, DOMAIN), Ok(true));
    let (report, _) = restore(&bytes, &mut c).unwrap();
    assert_eq!(
        (report.writes, report.committed, c.transport.begins),
        (0, 2, 0)
    );
    assert_eq!(archive::never_used(&mut c, DOMAIN), Ok(false));
    assert!(service.session(DOMAIN).is_some());
    let mut call = |operation, aux, length, data: &[u8]| {
        let mut h = wire::Header::new(operation, begin.request_id, EPOCH, object);
        h.aux = aux;
        h.length = length;
        let response = service.handle_for(DOMAIN, 77, &upload::encode(&h, data, token), &mut disk);
        let reply = upload::validate_reply(&h, token, &response, EPOCH).unwrap();
        (Status::from_code(i64::from(reply.status)), reply.revision)
    };
    assert_eq!(call(wire::APPEND, 0, 4, b"Rest").0, Status::Ok);
    assert_eq!(call(wire::COMMIT, 4, 0, &[]), (Status::Ok, 1));
    let mut c = client(&mut service, &mut disk, DOMAIN, None);
    assert_eq!(c.collection_stat(DOMAIN, 3).unwrap().name(), b"Leeg");
}

/// Fill the other domain to its quota: 40 committed chunks, or 32 committed
/// plus 8 staged in an open upload session.
fn fill_other_domain(service: &mut Service, disk: &mut SimDisk, staged: bool) {
    let mut other = client(service, disk, 0, None);
    other
        .write(0, 0, b"Vol", MAX_LENGTH, |_, b| b.fill(3))
        .unwrap();
    other
        .write(1, 0, b"Vol", MAX_LENGTH, |_, b| b.fill(3))
        .unwrap();
    if staged {
        other
            .write_partial(2, 0, b"Half", 8 * CHUNK, CHUNK, |_, b| b.fill(4))
            .unwrap();
        assert!(service.session(0).is_some());
    } else {
        other
            .write(2, 0, b"Vol", 8 * CHUNK, |_, b| b.fill(3))
            .unwrap();
    }
}

#[test]
fn exactly_the_guaranteed_free_chunks_restore_beside_a_full_domain() {
    assert_eq!(archive::GUARANTEED_FREE, 24);
    let bytes = fixture("free-24.bin");
    for staged in [false, true] {
        let mut disk = SimDisk::new();
        let mut service = Service::new(EPOCH, Table::EMPTY, false);
        fill_other_domain(&mut service, &mut disk, staged);
        let mut c = client(&mut service, &mut disk, DOMAIN, None);
        let (report, _) =
            restore(&bytes, &mut c).unwrap_or_else(|e| panic!("staged={staged}: {e:?}"));
        assert_eq!((report.restored, report.writes), (0xff, 24));
        assert_eq!(
            service.table().domain_chunks(DOMAIN),
            archive::GUARANTEED_FREE
        );
    }
}

#[test]
fn accepted_boundaries_restore() {
    let bytes = fixture("boundary.bin");
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN, None);
    restore(&bytes, &mut c).unwrap();
    let stat = c.stat((DOMAIN * PER_DOMAIN + 7) as u64).unwrap();
    assert_eq!(
        (stat.length, stat.name_length, stat.collection),
        (MAX_LENGTH, 32, 4)
    );
    assert_eq!(c.collection_stat(DOMAIN, 4).unwrap().name_length, 32);
}

#[test]
fn versions_after_a_revision_gap_are_not_replayed() {
    // Owner decision (19 November 2026): versions that may predate a DELETE
    // are refused. Only the prefix with revision steps of exactly 1 replays.
    let bytes = fixture("delete-gap.bin");
    let source = verify(&bytes).unwrap();
    let content = contents(&bytes);
    let prefix = |slot| archive::replayable(source.object(slot).unwrap());
    assert_eq!((prefix(0), prefix(2), prefix(4), prefix(7)), (1, 2, 1, 1));
    assert_eq!(
        archive::replayable(verify(&fixture("restore.bin")).unwrap().object(1).unwrap()),
        3
    );
    // Counting the skipped versions would exceed the guaranteed free chunks.
    let all: usize = source.objects[..source.object_count]
        .iter()
        .flat_map(|o| o.versions[..o.version_count].iter())
        .map(|v| v.length.div_ceil(CHUNK))
        .sum();
    assert!(all > archive::GUARANTEED_FREE, "{all}");

    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN, None);
    let (report, _) = restore(&bytes, &mut c).unwrap();
    assert_eq!((report.writes, report.skipped), (7, 6));
    assert_eq!(report.restored, 0b1111_0101);
    // Seven one-chunk versions replayed; nothing of the skipped ones stored.
    assert_eq!(service.table().domain_chunks(DOMAIN), 7);
    let mut c = client(&mut service, &mut disk, DOMAIN, None);
    let mut readable = Vec::new();
    for (slot, archived) in content.iter().enumerate() {
        let object = (DOMAIN * PER_DOMAIN + slot) as u64;
        let Ok(stat) = c.stat(object) else { continue };
        let o = source.object(slot).unwrap();
        let replayed = archive::replayable(o);
        assert_eq!(stat.history_count, replayed - 1, "slot {slot}");
        for selector in 0..replayed {
            let length = if selector == 0 {
                stat.length
            } else {
                stat.history[selector - 1].length
            };
            let got = read_version(&mut c, object, stat.revision, selector, length);
            assert_eq!(got, archived[selector], "slot {slot} version {selector}");
            readable.push(got);
        }
    }
    assert_eq!(readable.len(), 7);
    for skipped in content
        .iter()
        .flat_map(|versions| versions.iter().filter(|v| v.len() > CHUNK))
    {
        assert!(!readable.contains(skipped));
    }
    // The durable table holds exactly the replayed prefixes, names included.
    let table = recover(disk.settled()).unwrap();
    for o in source.objects[..source.object_count].iter() {
        let entry = &table.entries[DOMAIN * PER_DOMAIN + o.slot];
        let names: Vec<&[u8]> = core::iter::once(entry.current.name())
            .chain(
                entry.history[..entry.history_count]
                    .iter()
                    .map(|v| v.name()),
            )
            .collect();
        let recorded: Vec<&[u8]> = o.versions[..archive::replayable(o)]
            .iter()
            .map(|v| v.name())
            .collect();
        assert_eq!(names, recorded, "slot {}", o.slot);
        assert!(entry.versions().all(|v| v.length <= CHUNK));
    }
}

/// Loses the reply to the `nth` request of `operation` (1-based) like a dead
/// link; with `forward` the request still reached the service and committed.
/// `skew` then alters the first STAT or COLLECTION_STAT reply after that loss.
struct Lossy<'a> {
    inner: Local<'a>,
    operation: u16,
    nth: usize,
    forward: bool,
    seen: usize,
    skew: Option<fn(&mut [u8; wire::SIZE])>,
}
impl Transport for Lossy<'_> {
    fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
        let operation = wire::header(request).operation;
        if self.seen >= self.nth && matches!(operation, wire::STAT | wire::COLLECTION_STAT) {
            if let Some(skew) = self.skew.take() {
                let mut reply = self.inner.call(request)?;
                skew(&mut reply);
                return Ok(reply);
            }
        }
        if operation == self.operation {
            self.seen += 1;
            if self.seen == self.nth {
                if self.forward {
                    self.inner.call(request)?;
                }
                return Err(Status::Unknown);
            }
        }
        self.inner.call(request)
    }
}

fn lossy_restore(
    operation: u16,
    nth: usize,
    forward: bool,
    skew: Option<fn(&mut [u8; wire::SIZE])>,
) -> Result<NativeReport, (Status, NativeReport)> {
    let bytes = fixture("restore.bin");
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = Client::new(
        Lossy {
            inner: Local {
                service: &mut service,
                disk: &mut disk,
                domain: DOMAIN,
            },
            operation,
            nth,
            forward,
            seen: 0,
            skew,
        },
        EPOCH,
        100,
    );
    let result = restore(&bytes, &mut c).map(|(report, _)| report);
    assert!(c.transport.seen >= nth, "fault injected");
    assert!(c.transport.skew.is_none(), "skew applied");
    result
}

#[test]
fn a_lost_write_reply_is_resolved_by_stat() {
    // The first COMMIT landed: the replay continues and counts it.
    let report = lossy_restore(wire::COMMIT, 1, true, None).unwrap();
    assert_eq!(
        (report.restored, report.writes, report.committed),
        (0b0001_1010, 6, 10)
    );
    // The first COMMIT never arrived: STAT shows no new revision, so the
    // outcome stays Unknown and nothing is claimed.
    let (status, report) = lossy_restore(wire::COMMIT, 1, false, None).unwrap_err();
    assert_eq!(
        (status, report.writes, report.committed),
        (Status::Unknown, 0, 0)
    );
    // A later lost COMMIT that did not land: earlier WRITEs are counted.
    let (status, report) = lossy_restore(wire::COMMIT, 3, false, None).unwrap_err();
    assert_eq!((status, report.committed), (Status::Unknown, 2));
}

#[test]
fn a_lost_collection_or_meta_reply_is_resolved_by_reading_back() {
    for operation in [wire::COLLECTION, wire::META] {
        let report = lossy_restore(operation, 1, true, None).unwrap();
        assert_eq!((report.restored, report.committed), (0b0001_1010, 10));
    }
    // Completed slots stay reported: metadata-free slot 3 after its WRITE,
    // slot 1 after its META.
    let (status, report) = lossy_restore(wire::COLLECTION, 1, false, None).unwrap_err();
    assert_eq!(
        (status, report.committed, report.restored),
        (Status::Unknown, 6, 1 << 3)
    );
    let (status, report) = lossy_restore(wire::META, 2, false, None).unwrap_err();
    assert_eq!(
        (status, report.committed, report.restored),
        (Status::Unknown, 9, 0b0000_1010)
    );
}

// Reply byte offsets: revision 32, length (aux) 56, name 64, STAT content sum 96.
fn bump_revision(reply: &mut [u8; wire::SIZE]) {
    reply[32] += 1;
}
fn skew_name(reply: &mut [u8; wire::SIZE]) {
    reply[64] ^= 1;
}
fn skew_length(reply: &mut [u8; wire::SIZE]) {
    reply[56] += 1;
}
fn skew_content_sum(reply: &mut [u8; wire::SIZE]) {
    reply[96] ^= 1;
}
// STAT data starts at 64: tags at data 64, kind 68, collection 70.
fn skew_tags(reply: &mut [u8; wire::SIZE]) {
    reply[128] ^= 1;
}
fn skew_kind(reply: &mut [u8; wire::SIZE]) {
    reply[132] ^= 1;
}
fn skew_collection(reply: &mut [u8; wire::SIZE]) {
    reply[134] ^= 1;
}
// A well-formed COLLECTION_STAT reply for a deleted collection: count
// (offset 52) 0 and no name (length 40, name length 48, data from 64). The
// reply validation requires an absent collection to have no name, so the
// `present` check cannot be separated from the name check; this pins that an
// absent collection is never claimed.
fn clear_present(reply: &mut [u8; wire::SIZE]) {
    reply[40..44].fill(0);
    reply[48..56].fill(0);
    reply[64..].fill(0);
}

#[test]
fn a_landed_commit_that_stat_shows_differently_is_not_claimed() {
    // Each field of the exact-match rule on its own: one differing field
    // means the landed revision is not demonstrably ours.
    let skews: [fn(&mut [u8; wire::SIZE]); 4] =
        [bump_revision, skew_name, skew_length, skew_content_sum];
    for skew in skews {
        let (status, report) = lossy_restore(wire::COMMIT, 1, true, Some(skew)).unwrap_err();
        assert_eq!(
            (status, report.writes, report.committed),
            (Status::Unknown, 0, 0)
        );
    }
    let skews: [fn(&mut [u8; wire::SIZE]); 3] = [bump_revision, skew_name, clear_present];
    for skew in skews {
        let (status, report) = lossy_restore(wire::COLLECTION, 1, true, Some(skew)).unwrap_err();
        assert_eq!(
            (status, report.committed, report.restored),
            (Status::Unknown, 6, 1 << 3)
        );
    }
    // META: 6 WRITEs and 2 COLLECTIONs confirmed, slot 1's META not claimed.
    let skews: [fn(&mut [u8; wire::SIZE]); 4] =
        [bump_revision, skew_tags, skew_kind, skew_collection];
    for skew in skews {
        let (status, report) = lossy_restore(wire::META, 1, true, Some(skew)).unwrap_err();
        assert_eq!(
            (status, report.committed, report.restored),
            (Status::Unknown, 8, 1 << 3)
        );
    }
}

/// An archive range on a disk: the bytes from sector 0, the rest of the last
/// sector and `spare` further sectors filled with 0xA5. Reads are counted, and
/// a read outside the range panics.
#[derive(Clone)]
struct Range {
    sectors: Vec<[u8; SECTOR]>,
    reads: usize,
}
impl Range {
    fn new(bytes: &[u8], spare: usize) -> Self {
        let mut sectors = vec![[0xA5; SECTOR]; bytes.len().div_ceil(SECTOR) + spare];
        for (at, byte) in bytes.iter().enumerate() {
            sectors[at / SECTOR][at % SECTOR] = *byte;
        }
        Self { sectors, reads: 0 }
    }
    fn capacity(&self) -> u64 {
        self.sectors.len() as u64
    }
}
impl Disk for Range {
    fn read(&mut self, sector: u64, buffer: &mut [u8; SECTOR]) -> Result<(), Status> {
        self.reads += 1;
        *buffer = self.sectors[sector as usize];
        Ok(())
    }
    fn write(&mut self, _: u64, _: &[u8; SECTOR]) -> Result<(), Status> {
        unreachable!("a source never writes")
    }
    fn flush(&mut self) -> Result<(), Status> {
        unreachable!("a source never flushes")
    }
}

/// Every byte `source` yields, read in uneven pieces.
fn drain(source: &mut impl Source) -> Result<Vec<u8>, Status> {
    let mut out = Vec::new();
    let mut piece = [0; 77];
    loop {
        match source.get(&mut piece)? {
            0 => return Ok(out),
            amount => out.extend_from_slice(&piece[..amount]),
        }
    }
}

#[test]
fn disk_source_reads_every_v2_fixture_exactly_as_stored() {
    let mut names: Vec<String> = std::fs::read_dir(format!(
        "{}/tests/fixtures/archive-v2",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
    .map(|entry| entry.unwrap().file_name().into_string().unwrap())
    .filter(|name| name.ends_with(".bin"))
    .collect();
    names.sort();
    assert!(names.len() > 60, "fixtures missing: {}", names.len());
    let mut valid = 0;
    for name in &names {
        let mut bytes = fixture(name);
        if name == "trailing-byte.bin" {
            // A stream refuses a byte behind the trailer; on a disk that byte
            // lies behind the stored length and is padding, as in format v1.
            assert_eq!(verify(&bytes), Err(Status::Corrupt));
            bytes.pop();
        }
        let expected = verify(&bytes);
        // Exactly the archive's sectors, and with spare 0xA5 sectors behind it.
        for spare in [0, 2] {
            let mut range = Range::new(&bytes, spare);
            let capacity = range.capacity();
            let got =
                archive::verify_v2(&mut DiskSource::new(&mut range, capacity), &mut scratch());
            assert_eq!(got, expected, "{name}, {spare} spare sectors");
            if expected.is_ok() {
                // The computed length is the archive's: no padding byte leaks.
                let read = drain(&mut DiskSource::new(&mut range, capacity)).unwrap();
                assert_eq!(read, bytes, "{name}");
            }
        }
        valid += usize::from(expected.is_ok());
    }
    assert!(valid >= 5, "too few valid fixtures: {valid}");
}

#[test]
fn disk_source_refuses_a_short_range_and_an_unknown_version_after_one_read() {
    let bytes = fixture("restore.bin");
    assert!(verify(&bytes).is_ok());
    // One sector short of the stored length.
    let mut range = Range::new(&bytes, 0);
    let short = range.capacity() - 1;
    assert_eq!(
        drain(&mut DiskSource::new(&mut range, short)),
        Err(Status::Corrupt)
    );
    assert_eq!(range.reads, 1);
    // Format versions other than 1 and 2 are refused on the header sector alone.
    for version in [0u16, 3, 0xFFFF] {
        let mut forged = bytes.clone();
        forged[8..10].copy_from_slice(&version.to_le_bytes());
        let mut range = Range::new(&forged, 0);
        let capacity = range.capacity();
        assert_eq!(
            drain(&mut DiskSource::new(&mut range, capacity)),
            Err(Status::Corrupt),
            "version {version}"
        );
        assert_eq!(range.reads, 1, "version {version}");
    }
    // A v2 header claiming more than the largest v2 archive is refused on the
    // header sector, even on a range that could hold it (the walkers' own
    // header checks would refuse it too; a raw reader sees this bound alone).
    let mut forged = bytes.clone();
    forged[24..32].copy_from_slice(&MAX_ARCHIVE_V2.to_le_bytes());
    let mut range = Range::new(&forged, 0);
    let capacity = MAX_ARCHIVE_V2.div_ceil(SECTOR as u64) * 2;
    assert_eq!(
        drain(&mut DiskSource::new(&mut range, capacity)),
        Err(Status::Corrupt)
    );
    assert_eq!(range.reads, 1);
}

/// `restore_native_l2` opens the archive twice; an owning `DiskSource` over a
/// fresh handle per open gives the same result as the in-memory reader.
#[test]
fn restore_native_l2_reads_an_archive_range_through_owned_sources() {
    let bytes = fixture("restore.bin");
    let range = Range::new(&bytes, 1);
    let capacity = range.capacity();
    let mut results = Vec::new();
    for from_disk in [false, true] {
        let mut disk = SimDisk::new();
        let mut service = Service::new(EPOCH, Table::EMPTY, false);
        let mut c = client(&mut service, &mut disk, DOMAIN, None);
        let mut recorded = ArchiveV2::default();
        let mut opens = 0;
        let report = if from_disk {
            archive::restore_native_l2(
                || {
                    opens += 1;
                    DiskSource::new(range.clone(), capacity)
                },
                &mut scratch(),
                &mut c,
                DOMAIN,
                &mut recorded,
            )
        } else {
            archive::restore_native_l2(
                || reader(&bytes),
                &mut scratch(),
                &mut c,
                DOMAIN,
                &mut recorded,
            )
        }
        .unwrap();
        if from_disk {
            assert_eq!(opens, 2);
        }
        results.push((report, recorded));
    }
    assert_eq!(results[0], results[1]);
    assert_ne!(results[1].0.committed, 0);
}

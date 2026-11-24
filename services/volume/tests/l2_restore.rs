//! Host tests of L2 `restore_native_l2` (docs/implementation/S09-L2-PROVENANCE.md
//! section 8, tests 7-11) over the Python-oracle archive v2 fixtures through
//! the real in-process volume service.
mod common;

use common::{recover, Local, SimDisk};
use volume_core::{
    archive::{self, ArchiveV2, L2Report, Source},
    client::{Client, Transport},
    service::Service,
    table::{self, Table},
    upload, wire, Status, MAX_LENGTH, PER_DOMAIN,
};

const EPOCH: u64 = 6;
const DOMAIN: usize = 1;

/// Relative to the package directory (cargo's test working directory), so a
/// scratch copy of the tree (tools/mutate_volume.py) reads its own fixtures.
fn fixture(name: &str) -> Vec<u8> {
    let path = format!("tests/fixtures/archive-v2/{name}");
    std::fs::read(&path).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// Serves the archive in uneven pieces; `flip` corrupts one byte on every
/// open after the first (the verification pass reads clean bytes).
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
    flip: Option<usize>,
}
impl Source for Reader<'_> {
    fn get(&mut self, buffer: &mut [u8]) -> Result<usize, Status> {
        let amount = buffer.len().min(self.bytes.len() - self.at).min(77);
        buffer[..amount].copy_from_slice(&self.bytes[self.at..self.at + amount]);
        if let Some(flip) = self.flip {
            if (self.at..self.at + amount).contains(&flip) {
                buffer[flip - self.at] ^= 0x01;
            }
        }
        self.at += amount;
        Ok(amount)
    }
}

fn scratch() -> Box<[u8; MAX_LENGTH]> {
    vec![0u8; MAX_LENGTH].into_boxed_slice().try_into().unwrap()
}

/// Counts L2 writes; answers like an older service (`NoSys`), drops the
/// link (`Unknown`) at one operation, or neither.
struct Watch<T> {
    inner: T,
    installs: usize,
    aborts: usize,
    old_service: bool,
    unknown_at: Option<u16>,
    malformed_at: Option<u16>,
    refuse_at: Option<u16>,
    abort_error: Option<Status>,
    last_install: Option<wire::Header>,
}
impl<T: Transport> Transport for Watch<T> {
    fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
        let h = wire::header(request);
        let l2 = (wire::INSTALL_BEGIN..=wire::PROVENANCE_STAT).contains(&h.operation);
        if l2 && h.operation != wire::PROVENANCE_STAT {
            self.installs += 1;
            self.last_install = Some(h);
        }
        if h.operation == wire::ABORT {
            self.aborts += 1;
            if let Some(status) = self.abort_error {
                return Err(status);
            }
        }
        if self.refuse_at == Some(h.operation) {
            let reply = wire::Header {
                status: Status::Range as i32,
                ..wire::Header::new(h.operation, h.request_id, h.epoch, h.object)
            };
            return Ok(wire::encode(&reply, &[]));
        }
        if l2 && self.old_service {
            let reply = wire::Header {
                status: Status::NoSys as i32,
                ..wire::Header::new(h.operation, h.request_id, h.epoch, h.object)
            };
            return Ok(wire::encode(&reply, &[]));
        }
        if self.unknown_at == Some(h.operation) {
            // The request is delivered, the reply is lost.
            let _ = self.inner.call(request);
            return Err(Status::Unknown);
        }
        let mut reply = self.inner.call(request)?;
        if self.malformed_at == Some(h.operation) {
            reply[2] ^= 1; // the operation no longer matches the request
        }
        Ok(reply)
    }
}

fn client<'a>(
    service: &'a mut Service,
    disk: &'a mut SimDisk,
    domain: usize,
) -> Client<Watch<Local<'a>>> {
    Client::new(
        Watch {
            inner: Local {
                service,
                disk,
                domain,
            },
            installs: 0,
            aborts: 0,
            old_service: false,
            unknown_at: None,
            malformed_at: None,
            refuse_at: None,
            abort_error: None,
            last_install: None,
        },
        EPOCH,
        100 + 1000 * domain as u64,
    )
}

fn restore<T: Transport>(
    bytes: &[u8],
    c: &mut Client<T>,
    domain: usize,
    flip: Option<usize>,
) -> Result<(L2Report, ArchiveV2), (Status, L2Report)> {
    let mut source = ArchiveV2::default();
    let mut opens = 0;
    archive::restore_native_l2(
        || {
            opens += 1;
            Reader {
                bytes,
                at: 0,
                flip: if opens > 1 { flip } else { None },
            }
        },
        &mut scratch(),
        c,
        domain,
        &mut source,
    )
    .map(|report| (report, source))
}

fn contents(bytes: &[u8]) -> Vec<Vec<Vec<u8>>> {
    let mut out = vec![Vec::new(); PER_DOMAIN];
    archive::walk_v2(
        &mut Reader {
            bytes,
            at: 0,
            flip: None,
        },
        &mut scratch(),
        |o, _, content| {
            out[o.slot].push(content.to_vec());
            Ok(())
        },
    )
    .unwrap();
    out
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

/// Every present object and collection carries the archive values as
/// provenance beside live destination revisions; content is identical.
fn check_restored<T: Transport>(
    c: &mut Client<T>,
    bytes: &[u8],
    report: &L2Report,
    source: &ArchiveV2,
    domain: usize,
) {
    let content = contents(bytes);
    let mut commits = 0;
    for collection in &source.collections[..source.collection_count] {
        let info = c.collection_stat(domain, collection.index).unwrap();
        assert_eq!(info.present, collection.present);
        if collection.present {
            commits += 1;
            assert_eq!(info.revision, 1, "live revision, not the archive one");
            assert_eq!(info.name(), collection.name());
            assert_eq!(
                c.collection_provenance(domain, collection.index, 1),
                Ok(collection.revision)
            );
            assert_ne!(report.collections & (1 << (collection.index - 1)), 0);
        }
    }
    for object in &source.objects[..source.object_count] {
        let target = (domain * PER_DOMAIN + object.slot) as u64;
        if !object.present {
            assert_eq!(
                c.stat(target).unwrap_err(),
                (Status::NotFound, 0),
                "absent objects are not replayed"
            );
            assert_eq!(report.restored & (1 << object.slot), 0);
            continue;
        }
        commits += 1;
        let n = archive::replayable(object);
        let stat = c.stat(target).unwrap();
        assert_eq!(stat.revision, n as u64, "rule R: revision n");
        assert_eq!(report.revisions[object.slot], n as u64);
        assert_eq!(
            (stat.tags, stat.kind, stat.collection),
            (object.tags, object.kind, object.collection)
        );
        assert_eq!(stat.created, stat.modified);
        assert_eq!(stat.history_count, n - 1);
        let p = c.provenance(target, stat.revision).unwrap();
        assert_eq!(
            (
                p.installed,
                p.origin_revision,
                p.origin_created,
                p.origin_modified
            ),
            (n, object.revision, object.created, object.modified)
        );
        assert_eq!(p.version_count, n);
        for (selector, version) in object.versions[..n].iter().enumerate() {
            assert_eq!(
                p.versions[selector], version.revision,
                "version origin {selector}"
            );
            assert_eq!(
                read_version(c, target, stat.revision, selector, version.length),
                content[object.slot][selector]
            );
            if selector > 0 {
                assert_eq!(
                    c.version_stat(target, stat.revision, selector)
                        .unwrap()
                        .name(),
                    version.name()
                );
            }
        }
    }
    assert_eq!(report.committed, commits);
    let (_, clock) = c.domain_stat(domain).unwrap();
    assert_eq!(clock, commits as u64, "one clock tick per commit");
}

#[test]
fn restore_installs_every_present_object_with_its_provenance() {
    let bytes = fixture("restore.bin");
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN);
    let (report, source) = restore(&bytes, &mut c, DOMAIN, None).unwrap();
    assert_eq!(report.skipped, 0);
    assert_ne!(report.restored, 0);
    check_restored(&mut c, &bytes, &report, &source, DOMAIN);
    // A fresh instance mounts the v3 table and serves the same values.
    let t = recover(disk.settled()).unwrap();
    assert_eq!(t.format, table::VERSION_V3);
    let mut service = Service::new(EPOCH, t, true);
    let mut c = client(&mut service, &mut disk, DOMAIN);
    check_restored(&mut c, &bytes, &report, &source, DOMAIN);
}

#[test]
fn versions_behind_a_revision_gap_are_skipped() {
    let bytes = fixture("delete-gap.bin");
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN);
    let (report, source) = restore(&bytes, &mut c, DOMAIN, None).unwrap();
    let expected: usize = source.objects[..source.object_count]
        .iter()
        .filter(|o| o.present)
        .map(|o| o.version_count - archive::replayable(o))
        .sum();
    assert!(expected > 0, "fixture has a gap");
    assert_eq!(report.skipped, expected);
    check_restored(&mut c, &bytes, &report, &source, DOMAIN);
}

#[test]
fn restore_leaves_the_other_domain_alone() {
    let bytes = fixture("restore.bin");
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut other = client(&mut service, &mut disk, 0);
    let (revision, _) = other.write(3, 0, b"Eigen", 5, |_, b| b.fill(7)).unwrap();
    let before = (
        other.domain_stat(0).unwrap(),
        other.stat(3).unwrap(),
        other.provenance(3, revision).unwrap(),
    );
    restore(
        &bytes,
        &mut client(&mut service, &mut disk, DOMAIN),
        DOMAIN,
        None,
    )
    .unwrap();
    let mut other = client(&mut service, &mut disk, 0);
    let after = (
        other.domain_stat(0).unwrap(),
        other.stat(3).unwrap(),
        other.provenance(3, revision).unwrap(),
    );
    assert_eq!(before, after);
}

fn refused_without_writes(
    bytes: &[u8],
    status: Status,
    prepare: impl FnOnce(&mut Client<Watch<Local<'_>>>),
) {
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN);
    prepare(&mut c);
    let installs = c.transport.installs;
    let before = archive::never_used(&mut c, DOMAIN).unwrap();
    let (got, report) = restore(bytes, &mut c, DOMAIN, None).unwrap_err();
    assert_eq!(got, status);
    assert_eq!(report, L2Report::default());
    assert_eq!(c.transport.installs, installs, "no write was started");
    assert_eq!(archive::never_used(&mut c, DOMAIN).unwrap(), before);
}

#[test]
fn refusals_before_the_first_write() {
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
    // R3: a used domain (written, deleted, purged) is refused.
    let object = (DOMAIN * PER_DOMAIN + 5) as u64;
    refused_without_writes(&fixture("present-only.bin"), Status::Stale, |c| {
        let (revision, _) = c.write(object, 0, b"Weg", 4, |_, b| b.fill(1)).unwrap();
        let revision = c.delete(object, revision).unwrap();
        c.purge(object, revision).unwrap();
    });
    // Out-of-range destination domain.
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN);
    assert_eq!(
        restore(&fixture("restore.bin"), &mut c, 7, None)
            .unwrap_err()
            .0,
        Status::Invalid
    );
}

#[test]
fn an_older_service_answers_nosys_and_nothing_is_committed() {
    let bytes = fixture("restore.bin");
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN);
    c.transport.old_service = true;
    let (status, report) = restore(&bytes, &mut c, DOMAIN, None).unwrap_err();
    assert_eq!(
        (status, report.committed, c.transport.installs),
        (Status::NoSys, 0, 1)
    );
    assert!(archive::never_used(&mut c, DOMAIN).unwrap());
    assert_eq!(service.counters.table_writes, 0);
}

#[test]
fn a_staged_session_refuses_restore_before_any_collection_commit() {
    let bytes = fixture("restore.bin");
    let plan = archive::verify_v2(
        &mut Reader {
            bytes: &bytes,
            at: 0,
            flip: None,
        },
        &mut scratch(),
    )
    .unwrap();
    assert!(plan.collections[..plan.collection_count]
        .iter()
        .any(|collection| collection.present));
    let manifest = archive::l2_manifest(
        plan.objects[..plan.object_count]
            .iter()
            .find(|object| object.present)
            .unwrap(),
    );
    for operation in [wire::BEGIN, wire::INSTALL_BEGIN] {
        for tokened in [false, true] {
            let mut disk = SimDisk::new();
            let mut service = Service::new(EPOCH, Table::EMPTY, false);
            let object = (DOMAIN * PER_DOMAIN + 5) as u64;
            let mut begin = wire::Header::new(operation, 41, EPOCH, object);
            let data = if operation == wire::BEGIN {
                begin.aux = 600;
                begin.name_length = 4;
                begin.length = 4;
                b"Naam".to_vec()
            } else {
                begin.count = manifest.count as u32;
                begin.length = manifest.bytes() as u32;
                let mut data = vec![0; manifest.bytes()];
                manifest.encode(&mut data);
                data
            };
            let reply = if tokened {
                service.handle_for(DOMAIN, 17, &upload::encode(&begin, &data, 0), &mut disk)
            } else {
                service.handle(DOMAIN, &wire::encode(&begin, &data), &mut disk)
            };
            assert_eq!(wire::header(&reply).status, Status::Ok as i32);
            let token = if tokened { upload::token(&reply) } else { 0 };
            let writes = service.counters.table_writes;
            let mut c = client(&mut service, &mut disk, DOMAIN);
            let (status, report) = restore(&bytes, &mut c, DOMAIN, None).unwrap_err();
            assert_eq!(status, Status::Busy);
            assert_eq!(report.committed, 0, "refusal must not strand collections");
            assert_eq!((report.collections, report.restored), (0, 0));
            assert!(archive::never_used(&mut c, DOMAIN).unwrap());
            assert_eq!(service.counters.table_writes, writes);
            assert!(
                service.staged(DOMAIN).is_some(),
                "preserve the prior session"
            );

            // Only its original owner/session tuple can clear the staged work.
            let abort = wire::Header::new(wire::ABORT, 41, EPOCH, object);
            let reply = if tokened {
                service.handle_for(DOMAIN, 17, &upload::encode(&abort, &[], token), &mut disk)
            } else {
                service.handle(DOMAIN, &wire::encode(&abort, &[]), &mut disk)
            };
            assert_eq!(wire::header(&reply).status, Status::Ok as i32);
            assert!(service.staged(DOMAIN).is_none());
            let mut c = client(&mut service, &mut disk, DOMAIN);
            let (report, source) = restore(&bytes, &mut c, DOMAIN, None).unwrap();
            check_restored(&mut c, &bytes, &report, &source, DOMAIN);
        }
    }
}

/// Archive offset just past the content of version `index` of `slot`.
fn content_end(bytes: &[u8], slot: usize, index: usize) -> usize {
    let at = std::rc::Rc::new(std::cell::Cell::new(0));
    struct Counting<'a>(&'a [u8], std::rc::Rc<std::cell::Cell<usize>>);
    impl Source for Counting<'_> {
        fn get(&mut self, buffer: &mut [u8]) -> Result<usize, Status> {
            let at = self.1.get();
            let amount = buffer.len().min(self.0.len() - at);
            buffer[..amount].copy_from_slice(&self.0[at..at + amount]);
            self.1.set(at + amount);
            Ok(amount)
        }
    }
    let mut end = None;
    archive::walk_v2(
        &mut Counting(bytes, at.clone()),
        &mut scratch(),
        |o, i, content| {
            if (o.slot, i) == (slot, index) {
                let here = at.get();
                assert_eq!(&bytes[here - content.len()..here], content);
                end = Some(here);
            }
            Ok(())
        },
    )
    .unwrap();
    end.unwrap()
}

#[test]
fn a_content_mismatch_on_the_second_read_aborts_and_stops() {
    let bytes = fixture("restore.bin");
    let source = archive::verify_v2(
        &mut Reader {
            bytes: &bytes,
            at: 0,
            flip: None,
        },
        &mut scratch(),
    )
    .unwrap();
    // A history version (index 1) of a multi-version object: BEGIN and the
    // first stream have happened when the mismatch is read.
    let target = source.objects[..source.object_count]
        .iter()
        .find(|o| o.present && archive::replayable(o) >= 2 && o.versions[1].length > 0)
        .expect("fixture has a multi-version object");
    let flip = Some(content_end(&bytes, target.slot, 1) - 1);
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN);
    let (status, report) = restore(&bytes, &mut c, DOMAIN, flip).unwrap_err();
    assert_eq!(status, Status::Corrupt);
    assert_eq!(report.restored & (1 << target.slot), 0);
    assert_eq!(c.transport.aborts, 1, "the staged install is aborted");
    // Collections and the objects before it stand; no resume.
    let before = source.objects[..source.object_count]
        .iter()
        .filter(|o| o.present && o.slot < target.slot)
        .count();
    assert_eq!(
        report.committed,
        report.collections.count_ones() as usize + before
    );
    assert!(report.committed > 0);
    assert!(service.staged(DOMAIN).is_none());
    let t = recover(disk.settled()).unwrap();
    assert_eq!(t.entries[DOMAIN * PER_DOMAIN + target.slot].revision, 0);
    // Rerun into the same domain is refused; a fresh domain succeeds.
    let mut c = client(&mut service, &mut disk, DOMAIN);
    let (status, report) = restore(&bytes, &mut c, DOMAIN, None).unwrap_err();
    assert_eq!(
        (status, report.committed, c.transport.installs),
        (Status::Stale, 0, 0)
    );
    let mut fresh = client(&mut service, &mut disk, 0);
    let (report, source) = restore(&bytes, &mut fresh, 0, None).unwrap();
    check_restored(&mut fresh, &bytes, &report, &source, 0);
}

#[test]
fn source_uncertainty_after_begin_keeps_definite_cleanup_semantics() {
    struct FailingSource<'a> {
        inner: Reader<'a>,
        fail: Option<usize>,
        status: Status,
    }
    impl Source for FailingSource<'_> {
        fn get(&mut self, buffer: &mut [u8]) -> Result<usize, Status> {
            if self.fail.is_some_and(|at| {
                (self.inner.at..self.inner.at + buffer.len().min(77)).contains(&at)
            }) {
                return Err(self.status);
            }
            self.inner.get(buffer)
        }
    }
    let bytes = fixture("restore.bin");
    let source = archive::verify_v2(
        &mut Reader {
            bytes: &bytes,
            at: 0,
            flip: None,
        },
        &mut scratch(),
    )
    .unwrap();
    let target = source.objects[..source.object_count]
        .iter()
        .find(|o| o.present && archive::replayable(o) >= 2 && o.versions[1].length > 0)
        .unwrap();
    let fail_at = content_end(&bytes, target.slot, 1) - 1;
    for failure in [Status::Unknown, Status::Protocol] {
        for cleanup_error in [None, Some(Status::Io)] {
            let mut disk = SimDisk::new();
            let mut service = Service::new(EPOCH, Table::EMPTY, false);
            let mut c = client(&mut service, &mut disk, DOMAIN);
            c.transport.abort_error = cleanup_error;
            let mut opens = 0;
            let (status, report) = archive::restore_native_l2(
                || {
                    opens += 1;
                    FailingSource {
                        inner: Reader {
                            bytes: &bytes,
                            at: 0,
                            flip: None,
                        },
                        fail: if opens > 1 { Some(fail_at) } else { None },
                        status: failure,
                    }
                },
                &mut scratch(),
                &mut c,
                DOMAIN,
                &mut ArchiveV2::default(),
            )
            .unwrap_err();
            assert_eq!(status, failure, "source error remains the returned cause");
            assert_eq!(report.restored & (1 << target.slot), 0);
            assert_eq!(c.transport.aborts, 1, "no INSTALL RPC was uncertain");
            if let Some(cleanup_status) = cleanup_error {
                let pending = c.unconfirmed_install().unwrap();
                assert_eq!(
                    (pending.phase, pending.status, pending.begin_confirmed),
                    (
                        volume_core::client::InstallPhase::Abort,
                        cleanup_status,
                        true
                    )
                );
                assert!(c.transport.inner.service.staged(DOMAIN).is_some());
            } else {
                assert!(c.unconfirmed_install().is_none());
                assert!(c.transport.inner.service.staged(DOMAIN).is_none());
            }
        }
    }
}

#[test]
fn a_different_archive_on_the_second_read_installs_nothing() {
    // Verified plan: present-only.bin. The second read serves another valid
    // archive sharing slot 0 (delete-gap.bin): refused before INSTALL_BEGIN.
    let (plan, other) = (fixture("present-only.bin"), fixture("delete-gap.bin"));
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN);
    let mut opens = 0;
    let mut source = ArchiveV2::default();
    let (status, report) = archive::restore_native_l2(
        || {
            opens += 1;
            let bytes = if opens == 1 { &plan[..] } else { &other[..] };
            Reader {
                bytes,
                at: 0,
                flip: None,
            }
        },
        &mut scratch(),
        &mut c,
        DOMAIN,
        &mut source,
    )
    .unwrap_err();
    assert_eq!((status, report.restored), (Status::Corrupt, 0));
    assert_eq!(report.committed, report.collections.count_ones() as usize);
    assert!(service.staged(DOMAIN).is_none());
    let t = recover(disk.settled()).unwrap();
    assert!(
        t.entries.iter().all(|e| e.revision == 0),
        "no object installed"
    );
}

#[test]
fn an_unknown_commit_outcome_stops_without_abort() {
    let bytes = fixture("restore.bin");
    let mut disk = SimDisk::new();
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = client(&mut service, &mut disk, DOMAIN);
    c.transport.unknown_at = Some(wire::INSTALL_COMMIT);
    let (status, report) = restore(&bytes, &mut c, DOMAIN, None).unwrap_err();
    assert_eq!(status, Status::Unknown);
    assert_eq!(report.restored, 0, "the outcome is not claimed");
    let pending = c.unconfirmed_install().unwrap();
    assert_eq!(pending.phase, volume_core::client::InstallPhase::Commit);
    assert!(pending.begin_confirmed);
    assert_eq!(c.transport.aborts, 0);
    let installs = c.transport.installs;
    assert_eq!(
        restore(&bytes, &mut c, DOMAIN, None).unwrap_err().0,
        Status::Unknown
    );
    assert_eq!(
        c.transport.installs, installs,
        "no replay of the uncertain COMMIT"
    );
    assert_eq!(c.unconfirmed_install(), Some(pending));
    // The delivered commit happened; nothing was aborted or retried.
    assert!(service.staged(DOMAIN).is_none());
    let t = recover(disk.settled()).unwrap();
    assert_eq!(t.entries.iter().filter(|e| e.installed != 0).count(), 1);
}

#[test]
fn a_lost_append_retains_restore_recovery_before_another_write() {
    for malformed in [false, true] {
        let bytes = fixture("present-only.bin");
        let mut disk = SimDisk::new();
        let mut service = Service::new(EPOCH, Table::EMPTY, false);
        let mut c = client(&mut service, &mut disk, DOMAIN);
        if malformed {
            c.transport.malformed_at = Some(wire::INSTALL_APPEND);
        } else {
            c.transport.unknown_at = Some(wire::INSTALL_APPEND);
        }
        let expected = if malformed {
            Status::Protocol
        } else {
            Status::Unknown
        };
        let (status, report) = restore(&bytes, &mut c, DOMAIN, None).unwrap_err();
        assert_eq!(status, expected);
        let pending = c
            .unconfirmed_install()
            .expect("restore must retain the confirmed session on lost APPEND");
        assert_eq!(
            (
                pending.epoch,
                pending.status,
                pending.phase,
                pending.begin_confirmed
            ),
            (
                EPOCH,
                expected,
                volume_core::client::InstallPhase::Append,
                true
            )
        );
        assert_eq!(
            report.restored & (1 << (pending.object as usize % PER_DOMAIN)),
            0,
            "the uncertain object is not reported as installed"
        );
        assert_eq!(c.transport.aborts, 0);
        assert!(c.transport.inner.service.staged(DOMAIN).is_some());
        let installs = c.transport.installs;
        assert_eq!(
            restore(&bytes, &mut c, DOMAIN, None).unwrap_err().0,
            expected
        );
        assert_eq!(
            c.transport.installs, installs,
            "recovery blocks every restore write"
        );
        let wrong = volume_core::client::UnconfirmedInstall {
            request_id: pending.request_id + 1,
            ..pending
        };
        assert!(!c.acknowledge_install_recovery(wrong));
        assert_eq!(c.unconfirmed_install(), Some(pending));
        c.transport.unknown_at = None;
        c.transport.malformed_at = None;
        c.abort(pending.object, 0, pending.request_id).unwrap();
        assert!(c.acknowledge_install_recovery(pending));
        assert!(!c.acknowledge_install_recovery(pending));
        // Earlier confirmed collection/object commits still prevent same-domain resume.
        assert!(report.committed > 0);
        assert_eq!(
            restore(&bytes, &mut c, DOMAIN, None).unwrap_err().0,
            Status::Stale
        );
        let mut fresh = client(&mut service, &mut disk, 0);
        let (report, source) = restore(&bytes, &mut fresh, 0, None).unwrap();
        check_restored(&mut fresh, &bytes, &report, &source, 0);
    }
}

#[test]
fn failed_restore_cleanup_preserves_the_confirmed_id_and_original_error() {
    for operation in [wire::INSTALL_APPEND, wire::INSTALL_COMMIT] {
        for cleanup_error in [
            None,
            Some(Status::Io),
            Some(Status::Unknown),
            Some(Status::Protocol),
            Some(Status::Invalid),
        ] {
            let bytes = fixture("present-only.bin");
            let mut disk = SimDisk::new();
            let mut service = Service::new(EPOCH, Table::EMPTY, false);
            let mut c = client(&mut service, &mut disk, DOMAIN);
            c.transport.refuse_at = Some(operation);
            c.transport.abort_error = cleanup_error;
            let (status, report) = restore(&bytes, &mut c, DOMAIN, None).unwrap_err();
            assert_eq!(
                status,
                Status::Range,
                "cleanup does not replace the original refusal"
            );
            assert_eq!(c.transport.aborts, 1);
            if let Some(status) = cleanup_error {
                let sent = c.transport.last_install.unwrap();
                let pending = c
                    .unconfirmed_install()
                    .expect("failed cleanup retains the staged session");
                assert_eq!(
                    (
                        pending.object,
                        pending.request_id,
                        pending.phase,
                        pending.status,
                        pending.begin_confirmed
                    ),
                    (
                        sent.object,
                        sent.request_id,
                        volume_core::client::InstallPhase::Abort,
                        status,
                        true
                    )
                );
                assert!(c.transport.inner.service.staged(DOMAIN).is_some());
                let installs = c.transport.installs;
                assert_eq!(restore(&bytes, &mut c, DOMAIN, None).unwrap_err().0, status);
                assert_eq!(c.transport.installs, installs);
                c.transport.abort_error = None;
                c.abort(pending.object, 0, pending.request_id).unwrap();
                assert_eq!(c.unconfirmed_install(), Some(pending));
                assert!(c.acknowledge_install_recovery(pending));
            } else {
                assert!(c.unconfirmed_install().is_none());
                assert!(c.transport.inner.service.staged(DOMAIN).is_none());
            }
            c.transport.refuse_at = None;
            assert!(report.committed > 0);
            assert_eq!(
                restore(&bytes, &mut c, DOMAIN, None).unwrap_err().0,
                Status::Stale
            );
            let mut fresh = client(&mut service, &mut disk, 0);
            let (report, source) = restore(&bytes, &mut fresh, 0, None).unwrap();
            check_restored(&mut fresh, &bytes, &report, &source, 0);
        }
    }
}

#[test]
fn an_unconfirmed_begin_retains_recovery_and_blocks_another_restore() {
    for malformed in [false, true] {
        let bytes = fixture("present-only.bin");
        let mut disk = SimDisk::new();
        let mut service = Service::new(EPOCH, Table::EMPTY, false);
        let mut c = client(&mut service, &mut disk, DOMAIN);
        if malformed {
            c.transport.malformed_at = Some(wire::INSTALL_BEGIN);
        } else {
            c.transport.unknown_at = Some(wire::INSTALL_BEGIN);
        }
        let expected = if malformed {
            Status::Protocol
        } else {
            Status::Unknown
        };
        let (status, report) = restore(&bytes, &mut c, DOMAIN, None).unwrap_err();
        assert_eq!(status, expected);
        assert_eq!(report.restored, 0);
        assert_eq!(report.committed, report.collections.count_ones() as usize);
        assert!(
            report.committed > 0,
            "fixture has a collection before its object"
        );
        let pending = c.unconfirmed_install().unwrap();
        assert_eq!(pending.phase, volume_core::client::InstallPhase::Begin);
        assert!(!pending.begin_confirmed);
        assert_eq!((pending.epoch, pending.status), (EPOCH, expected));
        assert_eq!(
            c.transport.aborts, 0,
            "unconfirmed identity is not ownership"
        );
        assert!(c.transport.inner.service.staged(DOMAIN).is_some());
        let installs = c.transport.installs;
        assert_eq!(
            restore(&bytes, &mut c, DOMAIN, None).unwrap_err().0,
            expected
        );
        assert_eq!(c.transport.installs, installs, "retry starts no write");
        let wrong = volume_core::client::UnconfirmedInstall {
            request_id: pending.request_id + 1,
            ..pending
        };
        assert!(!c.acknowledge_install_recovery(wrong));
        assert_eq!(c.unconfirmed_install(), Some(pending));

        // The fixture independently knows this fresh domain had no prior owner.
        // A real caller without that proof must reconcile through trusted recovery.
        c.transport.malformed_at = None;
        c.transport.unknown_at = None;
        c.abort(pending.object, 0, pending.request_id).unwrap();
        assert!(c.acknowledge_install_recovery(pending));
        // The confirmed collection commit still prevents resuming this domain.
        assert_eq!(
            restore(&bytes, &mut c, DOMAIN, None).unwrap_err().0,
            Status::Stale
        );
        let mut fresh = client(&mut service, &mut disk, 0);
        let (report, source) = restore(&bytes, &mut fresh, 0, None).unwrap();
        check_restored(&mut fresh, &bytes, &report, &source, 0);
    }
}

#[test]
fn malformed_commit_replies_leave_only_confirmed_commits_in_the_report() {
    for operation in [wire::INSTALL_COLLECTION, wire::INSTALL_COMMIT] {
        for malformed in [false, true] {
            let bytes = fixture("restore.bin");
            let mut disk = SimDisk::new();
            let mut service = Service::new(EPOCH, Table::EMPTY, false);
            let mut c = client(&mut service, &mut disk, DOMAIN);
            if malformed {
                c.transport.malformed_at = Some(operation);
            } else {
                c.transport.unknown_at = Some(operation);
            }
            let expected = if malformed {
                Status::Protocol
            } else {
                Status::Unknown
            };
            let (status, report) = restore(&bytes, &mut c, DOMAIN, None).unwrap_err();
            assert_eq!(status, expected);
            assert_eq!(report.restored, 0, "the uncertain object is not claimed");
            let sent = c.transport.last_install.unwrap();
            let pending = c.unconfirmed_install().unwrap();
            assert_eq!(
                (
                    pending.epoch,
                    pending.object,
                    pending.request_id,
                    pending.status
                ),
                (sent.epoch, sent.object, sent.request_id, expected)
            );
            assert_eq!(
                pending.phase,
                if operation == wire::INSTALL_COLLECTION {
                    volume_core::client::InstallPhase::Collection {
                        index: sent.aux as usize,
                    }
                } else {
                    volume_core::client::InstallPhase::Commit
                }
            );
            assert_eq!(pending.begin_confirmed, operation == wire::INSTALL_COMMIT);
            assert_eq!(
                c.transport.aborts, 0,
                "uncertain durable changes are never aborted automatically"
            );
            let installs = c.transport.installs;
            assert_eq!(
                restore(&bytes, &mut c, DOMAIN, None).unwrap_err().0,
                expected
            );
            assert_eq!(c.transport.installs, installs);
            assert_eq!(archive::never_used(&mut c, DOMAIN), Ok(false));
            assert_eq!(c.unconfirmed_install(), Some(pending));
            let t = recover(c.transport.inner.disk.settled()).unwrap();
            let committed = t.entries.iter().filter(|e| e.installed != 0).count()
                + t.collections.iter().filter(|c| c.present).count();
            assert_eq!(
                committed,
                report.committed + 1,
                "the rejected reply committed"
            );
            // Reconcile the durable state before explicitly acknowledging; no resume.
            assert!(c.acknowledge_install_recovery(pending));
            c.transport.malformed_at = None;
            c.transport.unknown_at = None;
            assert_eq!(
                restore(&bytes, &mut c, DOMAIN, None).unwrap_err().0,
                Status::Stale
            );
        }
    }
}

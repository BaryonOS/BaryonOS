#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
// The portal builds (archive-portal: v1; l2-portal: S09 phase 3) keep only
// their S09 portal path so app text stays within the 64 KiB startup
// boundary; the fixture code they leave out is unused.
#![cfg_attr(
    any(feature = "archive-portal", feature = "l2-portal"),
    allow(dead_code, unused_imports)
)]
//! Native M6e document app. Two instances run in separate address spaces,
//! each with only a send/reply endpoint pair to the volume service. All
//! results are app-reported evidence; the host disk oracle checks the image.

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!("volume-app is only for the Nieuw OS x86_64-unknown-none target");
#[cfg(all(feature = "archive-portal", feature = "l2-portal"))]
compile_error!("archive-portal and l2-portal are separate portal builds");

#[cfg_attr(
    not(any(feature = "archive-portal", feature = "l2-portal")),
    allow(dead_code)
)]
mod disk;
mod support;

use disk::tick;
#[cfg(any(feature = "archive-portal", feature = "l2-portal"))]
use disk::NativeDisk;
use nieuw_os_sdk_runtime::{native, BlockDevice, Handle, Outgoing, ReceiveEndpoint, SendEndpoint};
#[cfg(feature = "archive-portal")]
use volume_core::archive::Report;
#[cfg(any(feature = "archive-portal", feature = "l2-portal"))]
use volume_core::archive::{self, DiskSink, DiskSource};
#[cfg(feature = "l2-portal")]
use volume_core::archive::{ArchiveV2, L2Report};
use volume_core::{
    checksum_update,
    client::{Client, HistoryInfo, Stat, Transport},
    wire::{self, Header, ListEntry, Manifest, SubOp},
    Status, CHUNK, COLLECTIONS, FNV_OFFSET, MAX_LENGTH, PER_DOMAIN,
};
#[cfg(feature = "archive-portal")]
use volume_core::{ustar, Disk, SECTOR};

const LIMIT: usize = 400_000;

// Case selectors written by the kernel manifest (startup word 112).
const NORMAL: u64 = 0;
const RECOVER: u64 = 1;
const CUT_DATA: u64 = 2;
const CUT_TABLE: u64 = 3;
const RECOVER_CUT: u64 = 4;
const CORRUPT: u64 = 5;
const CUT_TXN: u64 = 6;
// S09 L2 install cases (never 7..=19: those are the archive-portal cases).
const INSTALL: u64 = 20;
const INSTALL_RECOVER: u64 = 21;
const INSTALL_CUT: u64 = 22;

// S09 L2 INSTALL fixture, mirrored by INSTALL_* in tools/run_volume.py (the
// host oracle; tests/test_volume_runner.py checks both copies agree). Domain 0
// installs object index 0 with these versions, current first: version i has
// revision n - i, content byte(object, n - i, ..) and origin INSTALL_ORIGIN_0 - i.
// INSTALL_COLLECTION is installed first, so the object's clock is 2 (rule R).
const INSTALL_VERSIONS: [(&str, usize); 3] = [
    ("Versie drie", 5000),
    ("Versie twee", 300),
    ("Versie een", 0),
];
/// Object origin: (origin_revision, origin_created, origin_modified).
const INSTALL_ORIGIN: (u64, u64, u64) = (40, 7, 31);
const INSTALL_ORIGIN_0: u64 = 40;
/// (tags, kind, collection) of the installed object.
const INSTALL_META: (u32, u16, u8) = (0x5a, 2, 1);
/// (index, name, origin) of the collection INSTALL_COLLECTION creates.
const INSTALL_COLLECTION: (usize, &str, u64) = (1, "Import", 12);
// S09 archive cases: slot 1 is the backup portal, slot 2 an ordinary app.
#[cfg(feature = "archive-portal")]
const ARCHIVE_EXPORT: u64 = 7;
#[cfg(feature = "archive-portal")]
const ARCHIVE_RESTORE: u64 = 8;
#[cfg(feature = "archive-portal")]
const ARCHIVE_CORRUPT: u64 = 9;
#[cfg(feature = "archive-portal")]
const ARCHIVE_CONFLICT: u64 = 10;
#[cfg(feature = "archive-portal")]
const ARCHIVE_FULL: u64 = 11;
#[cfg(feature = "archive-portal")]
const ARCHIVE_USTAR: u64 = 12;
/// S09 second-disk flows of their own (the kernel maps the other archive-disk
/// cases onto the flows above): the archive disk is absent, and the power cut
/// during an export over an older archive.
#[cfg(feature = "archive-portal")]
const ARCHIVE_ABSENT: u64 = 15;
#[cfg(feature = "archive-portal")]
const ARCHIVE_CUT: u64 = 18;
/// S09 O13 N38 slice 1 (a, b, c, e): the portal's whole device-1 scope is one
/// kernel write-latch entry, and the kernel turns the completion of the second
/// device-1 WRITE into a device error (labelled, simulated fault).
#[cfg(feature = "archive-portal")]
const ARCHIVE_LATCH: u64 = 31;
/// S09 O13 N38 slice 2 (f): syscall 0x44 on test-scripted kernel latch
/// entries [0, 10) and [10, 20); the archive handle's scope is [0, 15) and the
/// kernel adds three handles (words 576, 584, 592). No injected fault.
#[cfg(feature = "archive-portal")]
const ARCHIVE_LATCH_0X44: u64 = 32;
// S09 phase 3 L2 cases (docs/implementation/S09-L2-NATIVE-EXPORT.md section
// 4): slot 1 is the backup portal on the device-1 archive disk.
#[cfg(feature = "l2-portal")]
const L2_EXPORT: u64 = 25;
#[cfg(feature = "l2-portal")]
const L2_RESTORE: u64 = 26;
#[cfg(feature = "l2-portal")]
const L2_CORRUPT: u64 = 27;
#[cfg(feature = "l2-portal")]
const L2_USED: u64 = 28;
// A power cut inside the L2 restore (29) and its cold read-only recovery (30).
#[cfg(feature = "l2-portal")]
const L2_RESTORE_CUT: u64 = 29;
#[cfg(feature = "l2-portal")]
const RECOVER_L2_CUT: u64 = 30;
/// Evidence words of the L2 cases (spec section 5.2), read by the kernel.
#[cfg(feature = "l2-portal")]
const L2_WORDS: L2Words = L2Words {
    restored: 520,
    collections: 528,
    committed: 536,
    skipped: 544,
    mismatches: 552,
    length: 560,
    status: 568,
};
#[cfg(feature = "l2-portal")]
struct L2Words {
    restored: u64,
    collections: u64,
    committed: u64,
    /// `L2Report.skipped`, or `ExportV2.omitted` for the export case.
    skipped: u64,
    mismatches: u64,
    length: u64,
    status: u64,
}

/// Startup word with the portal's archive range capability (slot 1, archive
/// cases only); the range size must equal the kernel manifest's scope.
#[cfg(any(feature = "archive-portal", feature = "l2-portal"))]
const ARCHIVE_HANDLE: u64 = 504;
#[cfg(any(feature = "archive-portal", feature = "l2-portal"))]
const ARCHIVE_SECTORS: u64 = 1040;
/// Scope of the full-target case, below the 180 sectors the export needs.
#[cfg(feature = "archive-portal")]
const FULL_SECTORS: u64 = 100;
/// The ustar case stores its archive in the lower half of the range and the
/// ustar stream from this sector; tools/run_volume.py uses the same split.
#[cfg(feature = "archive-portal")]
const USTAR_FIRST: u64 = ARCHIVE_SECTORS / 2;

/// The upper half of the archive range as a disk of its own, so the ustar
/// stream cannot reach the archive it is read from.
#[cfg(feature = "archive-portal")]
struct Shifted(NativeDisk);

#[cfg(feature = "archive-portal")]
impl Disk for Shifted {
    fn read(&mut self, sector: u64, buffer: &mut [u8; SECTOR]) -> Result<(), Status> {
        self.0.read(sector + USTAR_FIRST, buffer)
    }

    fn write(&mut self, sector: u64, buffer: &[u8; SECTOR]) -> Result<(), Status> {
        self.0.write(sector + USTAR_FIRST, buffer)
    }

    fn flush(&mut self) -> Result<(), Status> {
        self.0.flush()
    }
}
/// Private, zeroed 64 KiB RW/NX heap the kernel maps for every process
/// before entry; the portal's only use is the archive content scratch.
#[cfg(any(feature = "archive-portal", feature = "l2-portal"))]
const HEAP: usize = 0x6000_0000;
#[cfg(any(feature = "archive-portal", feature = "l2-portal"))]
const _: () = assert!(MAX_LENGTH == 0x10000);

fn fail(step: u64) -> ! {
    support::put(240, step);
    support::put(40, 0x100 + step);
    native::exit(1)
}

fn check(condition: bool, step: u64) {
    if !condition {
        fail(step);
    }
}

/// Deterministic fixture content; tools/volume_disk.py implements the same
/// function independently to verify the image.
fn byte(object: u64, revision: u64, offset: usize) -> u8 {
    (object as usize * 31 + revision as usize * 7 + offset * 13 + offset / 251) as u8
}

fn fill(object: u64, revision: u64) -> impl FnMut(usize, &mut [u8]) {
    move |offset, buffer| {
        for (index, value) in buffer.iter_mut().enumerate() {
            *value = byte(object, revision, offset + index);
        }
    }
}

struct Native {
    send: SendEndpoint,
    receive: ReceiveEndpoint,
    service: u64,
}

impl Transport for Native {
    fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
        let message = Outgoing::new(wire::TAG, request).map_err(|_| Status::Protocol)?;
        let mut sent = false;
        for _ in 0..LIMIT {
            tick();
            match native::send(self.send, &message) {
                Ok(()) => {
                    sent = true;
                    break;
                }
                Err(error) if error.is_again() => {
                    let _ = native::yield_now();
                }
                Err(_) => return Err(Status::Protocol),
            }
        }
        if !sent {
            return Err(Status::Timeout);
        }
        for _ in 0..LIMIT {
            tick();
            match native::receive(self.receive) {
                Ok(reply) => {
                    let caps = reply.cap_count() != 0;
                    if caps {
                        let _ = native::close_received_caps(&reply);
                    }
                    if caps
                        || reply.sender() != self.service
                        || reply.tag() != wire::TAG
                        || reply.payload().len() != wire::SIZE
                    {
                        // A queued message may already have been processed:
                        // never resend, report the exchange as uncertain.
                        return Err(Status::Unknown);
                    }
                    let mut body = [0; wire::SIZE];
                    body.copy_from_slice(reply.payload());
                    return Ok(body);
                }
                Err(error) if error.is_again() => {
                    let _ = native::yield_now();
                }
                Err(_) => return Err(Status::Unknown),
            }
        }
        Err(Status::Unknown)
    }
}

struct App {
    client: Client<Native>,
    domain: usize,
    verified_bytes: u64,
    verified_docs: u64,
    commits: u64,
}

impl App {
    fn object(&self, index: u64) -> u64 {
        (self.domain * PER_DOMAIN) as u64 + index
    }

    fn write(&mut self, index: u64, expected: u64, name: &[u8], length: usize, step: u64) -> u64 {
        let object = self.object(index);
        match self
            .client
            .write(object, expected, name, length, fill(object, expected + 1))
        {
            Ok((revision, id)) => {
                check(revision == expected + 1, step);
                self.commits += 1;
                let stat = self.stat(index, step + 1);
                check(
                    stat.revision == revision && stat.length == length && stat.request_id == id,
                    step + 2,
                );
                check(&stat.name[..stat.name_length] == name, step + 3);
                revision
            }
            Err(_) => fail(step),
        }
    }

    fn stat(&mut self, index: u64, step: u64) -> Stat {
        self.client
            .stat(self.object(index))
            .unwrap_or_else(|_| fail(step))
    }

    /// Read every byte of one retained version through bounded READ replies
    /// and compare it against the fixture for `content` and the length and
    /// checksum that STAT reported for it.
    fn verify_version(
        &mut self,
        index: u64,
        revision: u64,
        selector: usize,
        version: HistoryInfo,
        content: u64,
        step: u64,
    ) {
        let (length, sum_expected) = (version.length, version.content_sum);
        let object = self.object(index);
        let mut sum = FNV_OFFSET;
        let mut mismatch = false;
        let read = self
            .client
            .read_version(object, revision, selector, length, |offset, bytes| {
                for (index, value) in bytes.iter().enumerate() {
                    mismatch |= *value != byte(object, content, offset + index);
                }
                sum = checksum_update(sum, bytes);
            });
        check(read == Ok(length) && !mismatch && sum == sum_expected, step);
        self.verified_docs += 1;
        self.verified_bytes += length as u64;
    }

    fn verify(&mut self, index: u64, content: u64, step: u64) -> Stat {
        let stat = self.stat(index, step);
        let current = HistoryInfo {
            revision: stat.version_revision,
            length: stat.length,
            content_sum: stat.content_sum,
        };
        self.verify_version(index, stat.revision, 0, current, content, step + 1);
        stat
    }

    fn list(&mut self, filter: usize, step: u64) -> ([ListEntry; PER_DOMAIN], usize) {
        let mut entries = [ListEntry::EMPTY; PER_DOMAIN];
        let (count, _) = self
            .client
            .list_filtered(self.domain, filter, &mut entries)
            .unwrap_or_else(|_| fail(step));
        (entries, count)
    }

    /// Objects (by domain index) listed under `filter`, as a bit mask.
    fn listed(&mut self, filter: usize, step: u64) -> u8 {
        let (entries, count) = self.list(filter, step);
        entries[..count].iter().fold(0u8, |mask, entry| {
            mask | 1 << (entry.object as usize % PER_DOMAIN)
        })
    }

    fn raw(&mut self, header: &Header, data: &[u8]) -> Header {
        match self.client.exchange(header, data) {
            Ok((reply, _)) => reply,
            Err(_) => fail(900),
        }
    }

    /// Requests for the other domain, out-of-range objects and malformed
    /// bytes must be refused by the service before any state is consulted.
    fn boundaries(&mut self, step: u64) {
        let epoch = self.client.epoch;
        let foreign = ((1 - self.domain) * PER_DOMAIN) as u64;
        let id = self.client.next_request_id() + 10_000;
        let mut foreign_meta = Header::new(wire::META, id + 6, epoch, foreign);
        foreign_meta.revision = 1;
        let mut foreign_collection = Header::new(wire::COLLECTION, id + 7, epoch, foreign);
        foreign_collection.aux = 1;
        let mut foreign_restore = Header::new(wire::RESTORE, id + 8, epoch, foreign);
        foreign_restore.aux = 1;
        for (offset, mut header) in [
            Header::new(wire::STAT, id, epoch, foreign),
            Header::new(wire::LIST, id + 1, epoch, foreign),
            Header::new(wire::DELETE, id + 2, epoch, foreign + 1),
            foreign_meta,
            foreign_collection,
            foreign_restore,
        ]
        .into_iter()
        .enumerate()
        {
            if header.operation == wire::DELETE {
                header.revision = 1;
            }
            let reply = self.raw(&header, &[]);
            check(reply.status == Status::Access as i32, step + offset as u64);
        }
        let reply = self.raw(&Header::new(wire::STAT, id + 3, epoch, 16), &[]);
        check(reply.status == Status::Range as i32, step + 10);
        let mut write = Header::new(wire::BEGIN, id + 4, epoch, foreign);
        write.aux = 10;
        write.length = 1;
        write.name_length = 1;
        check(
            self.raw(&write, b"x").status == Status::Access as i32,
            step + 11,
        );
        // Malformed: version 4 body; the reply must be a canonical Invalid.
        let mut body = wire::encode(&Header::new(wire::STAT, id + 5, epoch, self.object(0)), &[]);
        body[0] = 4;
        match self.client.transport.call(&body) {
            Ok(reply) => {
                let h = wire::header(&reply);
                check(
                    h.status == Status::Invalid as i32 && h.revision == 0,
                    step + 12,
                );
                check(reply[64..].iter().all(|b| *b == 0), step + 13);
            }
            Err(_) => fail(step + 12),
        }
        let mut read = Header::new(wire::READ, id + 9, epoch, self.object(0));
        read.count = wire::DATA as u32 + 1;
        check(
            self.raw(&read, &[]).status == Status::Invalid as i32,
            step + 14,
        );
        // A transaction naming one object twice is structurally invalid.
        let mut txn = Header::new(wire::TRANSACT, id + 10, epoch, self.object(0));
        txn.count = 2;
        txn.length = 2 * wire::SUBOP as u32;
        let mut data = [0u8; 2 * wire::SUBOP];
        data[1] = wire::SUB_PURGE;
        data[wire::SUBOP + 1] = wire::SUB_PURGE;
        check(
            self.raw(&txn, &data).status == Status::Invalid as i32,
            step + 15,
        );
    }

    fn wait_gate(&self, value: u64) {
        while support::word(464) < value {
            tick();
            let _ = native::yield_now();
        }
    }

    fn phase(&self, value: u64) {
        support::put(456, value);
    }

    /// Domain digest over objects (metadata, content checksum, history depth)
    /// and collections; the host oracle recomputes it from the raw image.
    fn digest(&mut self, step: u64) -> u64 {
        let mut digest = FNV_OFFSET;
        for index in 0..PER_DOMAIN as u64 {
            let object = self.object(index);
            match self.client.stat(object) {
                Ok(stat) => {
                    digest = checksum_update(digest, &[index as u8, 1]);
                    digest = checksum_update(digest, &stat.revision.to_le_bytes());
                    digest = checksum_update(digest, &(stat.length as u64).to_le_bytes());
                    digest = checksum_update(digest, &stat.content_sum.to_le_bytes());
                    digest = checksum_update(digest, &stat.name[..stat.name_length]);
                    digest = checksum_update(digest, &[stat.collection, stat.history_count as u8]);
                    digest = checksum_update(digest, &stat.tags.to_le_bytes());
                    digest = checksum_update(digest, &stat.kind.to_le_bytes());
                }
                Err((Status::NotFound, revision)) => {
                    digest = checksum_update(digest, &[index as u8, 0]);
                    digest = checksum_update(digest, &revision.to_le_bytes());
                }
                Err(_) => fail(step),
            }
        }
        // Collections are observable through LIST filters only; their
        // membership mask per index enters the digest.
        for collection in 1..=COLLECTIONS {
            let mask = self.listed(collection + 1, step + 1);
            digest = checksum_update(digest, &[0xc0 | collection as u8, mask]);
        }
        digest
    }

    fn report(&mut self, step: u64) {
        let digest = self.digest(step);
        support::put(400, self.verified_docs);
        support::put(408, self.verified_bytes);
        support::put(416, self.commits);
        support::put(448, digest);
    }

    /// COLLECTION_STAT of `domain`'s collection `index`: present with `name`
    /// at `revision`, or (`None`) absent at `revision` (0 = never created).
    fn collection_is(
        &mut self,
        domain: usize,
        index: usize,
        name: Option<&str>,
        revision: u64,
        step: u64,
    ) {
        let info = self
            .client
            .collection_stat(domain, index)
            .unwrap_or_else(|_| fail(step));
        let named = match name {
            Some(name) => info.present && info.name() == name.as_bytes(),
            None => !info.present && info.name_length == 0,
        };
        check(
            named && info.revision == revision && (revision == 0) == (info.request_id == 0),
            step + 1,
        );
    }

    fn meta(
        &mut self,
        index: u64,
        expected: u64,
        tags: u32,
        kind: u16,
        collection: u8,
        step: u64,
    ) -> u64 {
        let revision = self
            .client
            .meta(self.object(index), expected, tags, kind, collection)
            .unwrap_or_else(|_| fail(step));
        check(revision == expected + 1, step + 1);
        revision
    }
}

fn sub(index: u8, op: u8, collection: u8, tags: u32, expected: u64) -> SubOp {
    SubOp {
        index,
        op,
        collection,
        tags,
        expected,
        kind: 0,
    }
}

/// Expected final state of the normal fixture, re-verified by the cold
/// recovery: (index, revision, content revision, length, name, collection,
/// history depth).
type Final = (u64, u64, u64, usize, &'static str, u8, usize);
const FINAL_A: &[Final] = &[
    (1, 3, 1, 129, "Notitie ✓", 2, 0),
    (2, 2, 1, 0, "Leeg", 1, 0),
    (3, 42, 38, 5 * CHUNK, "Concept", 0, 0),
    (4, 4, 1, 5000, "Herstart", 0, 0),
    (5, 1, 1, MAX_LENGTH, "Archief", 0, 0),
];
const FINAL_B: &[Final] = &[
    (0, 2, 1, 4 * CHUNK, "Foto-index", 1, 0),
    (1, 1, 1, 300, "Lijst", 0, 0),
    (2, 21, 20, CHUNK, "Logboek", 0, 0),
    (3, 1, 1, MAX_LENGTH, "Groot 1", 0, 0),
    (4, 1, 1, MAX_LENGTH, "Groot 2", 0, 0),
];

fn normal_a(app: &mut App) {
    let (_, count) = app.list(0, 10);
    check(count == 0, 11);
    // S05: documents far beyond the old 128-byte record, streamed in 160-byte pieces.
    let revision = app.write(0, 0, "Jaarverslag 2026".as_bytes(), MAX_LENGTH, 20);
    check(revision == 1, 24);
    app.verify(0, 1, 25);
    app.write(1, 0, b"Notitie", 129, 30);
    app.verify(1, 1, 35);
    app.write(2, 0, b"Leeg", 0, 40);
    app.verify(2, 1, 45);
    check(
        app.client.rename(app.object(1), 1, "Notitie ✓".as_bytes()) == Ok(2),
        50,
    );
    check(
        app.client.rename(app.object(1), 1, b"Oud") == Err(Status::Stale),
        51,
    );
    app.verify(1, 1, 52);
    // S04: 40 overwrites alternate 4 and 5 chunks; old chunks are reused.
    let mut revision = 0;
    for round in 0..40 {
        let length = if round % 2 == 0 {
            3 * CHUNK + 17
        } else {
            5 * CHUNK
        };
        revision = app.write(3, revision, b"Concept", length, 60);
        app.verify(3, revision, 65);
    }
    check(revision == 40, 69);
    // S06 history: the two previous versions are readable and restorable.
    let stat = app.stat(3, 200);
    check(stat.history_count == 2, 201);
    check(
        stat.history[0].revision == 39 && stat.history[1].revision == 38,
        202,
    );
    for (selector, content) in [(1usize, 39u64), (2, 38)] {
        let info = stat.history[selector - 1];
        app.verify_version(3, 40, selector, info, content, 203);
    }
    check(app.client.restore(app.object(3), 40, 2) == Ok(41), 205);
    let restored = app.verify(3, 38, 206);
    check(
        restored.version_revision == 41 && restored.length == 5 * CHUNK,
        208,
    );
    check(
        restored.history[0].revision == 40 && restored.history[1].revision == 39,
        209,
    );
    // Abort and an abandoned session replaced by a new BEGIN.
    let object = app.object(4);
    let id = app
        .client
        .write_partial(object, 0, b"Halfweg", 5000, 1000, fill(object, 1))
        .unwrap_or_else(|_| fail(70));
    check(app.client.abort(object, 0, id).is_ok(), 71);
    check(app.client.stat(object) == Err((Status::NotFound, 0)), 72);
    let old = app
        .client
        .write_partial(object, 0, b"Halfweg", 5000, 480, fill(object, 1))
        .unwrap_or_else(|_| fail(73));
    app.write(4, 0, b"Herstart", 5000, 74);
    let mut stale = Header::new(wire::APPEND, old, app.client.epoch, object);
    stale.aux = 480;
    stale.length = 1;
    check(app.raw(&stale, &[1]).status == Status::Invalid as i32, 78);
    app.verify(4, 1, 79);
    app.boundaries(80);
    // S06 collections and metadata.
    check(app.client.collection(0, 1, 0, b"Werk") == Ok(1), 100);
    check(
        app.client.collection(0, 2, 0, "Privé".as_bytes()) == Ok(1),
        101,
    );
    check(
        app.client.collection(0, 1, 0, b"Oud") == Err((Status::Stale, 0)),
        102,
    );
    // Collections are readable: name, presence and CAS revision.
    app.collection_is(0, 1, Some("Werk"), 1, 90);
    app.collection_is(0, 2, Some("Privé"), 1, 92);
    app.collection_is(0, 3, None, 0, 94);
    check(
        app.client.meta(app.object(0), 1, 1, 1, 4) == Err((Status::Range, 0)),
        103,
    );
    app.meta(0, 1, 1, 1, 1, 104);
    app.meta(1, 2, 0, 2, 2, 106);
    // A transaction with one stale precondition changes nothing ...
    let failing = [
        sub(2, wire::SUB_META, 1, 0, 1),
        sub(4, wire::SUB_META, 1, 0, 9),
    ];
    check(app.client.transact(0, &failing) == Err(Status::Stale), 108);
    check(
        app.stat(2, 109).collection == 0 && app.stat(4, 110).revision == 1,
        111,
    );
    // ... and a valid one moves both documents in a single commit.
    let moving = [
        sub(2, wire::SUB_META, 1, 0, 1),
        sub(4, wire::SUB_META, 1, 0, 1),
    ];
    check(app.client.transact(0, &moving).is_ok(), 112);
    app.commits += 1;
    check(
        app.stat(2, 113).revision == 2 && app.stat(4, 114).collection == 1,
        115,
    );
    check(app.listed(2, 116) == 0b10101, 117);
    check(app.listed(3, 118) == 0b10, 119);
    check(app.listed(1, 120) == 0b1000, 121);
    check(
        app.client.collection(0, 1, 1, b"") == Err((Status::Busy, 0)),
        122,
    );
    // Undelete: delete keeps the content as history; RESTORE brings it back.
    check(app.client.delete(app.object(4), 2) == Ok(3), 125);
    check(
        app.client.stat(app.object(4)) == Err((Status::NotFound, 3)),
        126,
    );
    check(app.client.restore(app.object(4), 3, 1) == Ok(4), 127);
    let back = app.verify(4, 1, 128);
    // The undeleted version left the history; nothing is held twice.
    check(back.collection == 0 && back.history_count == 0, 130);
    let (_, count) = app.list(0, 131);
    check(count == 5, 132);
    app.phase(1);
    app.wait_gate(1);
    // Quota counts retained history: the archive does not fit until the
    // older drafts are purged.
    let object = app.object(5);
    check(
        app.client
            .write(object, 0, b"Archief", MAX_LENGTH, fill(object, 1))
            == Err(Status::Quota),
        140,
    );
    check(app.client.purge(app.object(3), 41) == Ok(42), 141);
    app.write(5, 0, b"Archief", MAX_LENGTH, 142);
    let object = app.object(6);
    check(
        app.client.write(object, 0, b"Te veel", 1, fill(object, 1)) == Err(Status::Quota),
        146,
    );
    check(app.client.stat(object) == Err((Status::NotFound, 0)), 147);
    support::put(424, 1);
    app.phase(2);
    app.wait_gate(2);
    // Release 16 chunks for domain B: delete keeps history, purge frees it.
    check(app.client.delete(app.object(0), 2) == Ok(3), 150);
    check(app.client.purge(app.object(0), 3) == Ok(4), 151);
    check(
        app.client.stat(app.object(0)) == Err((Status::NotFound, 4)),
        152,
    );
    app.phase(3);
    verify_final(app, FINAL_A, 160);
    app.report(180);
}

fn normal_b(app: &mut App) {
    let (_, count) = app.list(0, 10);
    check(count == 0, 11);
    app.write(0, 0, b"Foto-index", 4 * CHUNK, 20);
    app.verify(0, 1, 25);
    app.write(1, 0, b"Lijst", 300, 30);
    app.verify(1, 1, 35);
    let mut revision = 0;
    for round in 0..20 {
        let length = if round % 2 == 0 { 2 * CHUNK + 1 } else { CHUNK };
        revision = app.write(2, revision, b"Logboek", length, 40);
        app.verify(2, revision, 45);
    }
    let stat = app.stat(2, 46);
    let info = stat.history[0];
    app.verify_version(2, 20, 1, info, 19, 47);
    check(app.client.purge(app.object(2), 20) == Ok(21), 48);
    check(app.stat(2, 49).history_count == 0, 50);
    app.boundaries(60);
    check(app.client.collection(1, 1, 0, b"Fotos") == Ok(1), 80);
    app.collection_is(1, 1, Some("Fotos"), 1, 82);
    app.meta(0, 1, 3, 7, 1, 81);
    app.phase(1);
    app.wait_gate(2);
    // Domain A holds 40 chunks and B 6: 18 free. The first 16-chunk upload
    // fits; the second exceeds free space while B stays under its quota.
    app.write(3, 0, b"Groot 1", MAX_LENGTH, 90);
    let object = app.object(4);
    check(
        app.client
            .write(object, 0, b"Groot 2", MAX_LENGTH, fill(object, 1))
            == Err(Status::Full),
        95,
    );
    support::put(432, 1);
    app.phase(2);
    app.wait_gate(3);
    app.write(4, 0, b"Groot 2", MAX_LENGTH, 100);
    app.verify(4, 1, 105);
    verify_final(app, FINAL_B, 160);
    app.report(180);
}

fn verify_final(app: &mut App, expected: &[Final], step: u64) {
    for (index, revision, content, length, name, collection, history) in expected {
        let stat = app.verify(*index, *content, step);
        check(
            stat.revision == *revision && stat.length == *length,
            step + 2,
        );
        check(&stat.name[..stat.name_length] == name.as_bytes(), step + 3);
        check(
            stat.collection == *collection && stat.history_count == *history,
            step + 4,
        );
        for selector in 1..=stat.history_count {
            let info = stat.history[selector - 1];
            app.verify_version(*index, stat.revision, selector, info, *content, step + 5);
        }
    }
    let (_, count) = app.list(0, step + 6);
    check(count == expected.len(), step + 7);
}

fn recover(app: &mut App) {
    let expected = if app.domain == 0 { FINAL_A } else { FINAL_B };
    verify_final(app, expected, 20);
    if app.domain == 0 {
        check(
            app.client.stat(app.object(0)) == Err((Status::NotFound, 4)),
            40,
        );
        check(app.listed(2, 41) == 0b100 && app.listed(3, 42) == 0b10, 43);
    } else {
        check(app.listed(2, 41) == 0b1, 43);
    }
    // The cold recovery instance is read-only: every mutation is refused.
    let object = app.object(7);
    check(
        app.client.write(object, 0, b"Nee", 1, fill(object, 1)) == Err(Status::Access),
        44,
    );
    let first = app.object(expected[0].0);
    let revision = expected[0].1;
    check(
        app.client.rename(first, revision, b"Nee") == Err(Status::Access),
        45,
    );
    check(
        app.client.delete(first, revision) == Err((Status::Access, 0)),
        46,
    );
    check(
        app.client.meta(first, revision, 0, 0, 0) == Err((Status::Access, 0)),
        47,
    );
    check(
        app.client.collection(app.domain, 3, 0, b"Nee") == Err((Status::Access, 0)),
        48,
    );
    check(
        app.client.purge(first, revision) == Err((Status::Access, 0)),
        49,
    );
    check(
        app.client.restore(first, revision, 1) == Err((Status::Access, 0)),
        60,
    );
    check(
        app.client
            .transact(app.domain, &[sub(0, wire::SUB_PURGE, 0, 0, 1)])
            == Err(Status::Access),
        61,
    );
    // Reading collections stays allowed on the read-only mount.
    if app.domain == 0 {
        app.collection_is(0, 1, Some("Werk"), 1, 62);
        app.collection_is(0, 2, Some("Privé"), 1, 64);
    } else {
        app.collection_is(1, 1, Some("Fotos"), 1, 62);
        app.collection_is(1, 2, None, 0, 64);
    }
    app.report(50);
}

fn cut(app: &mut App, case: u64) {
    if app.domain == 1 {
        let (_, count) = app.list(0, 10);
        check(count == 0, 11);
        return;
    }
    app.write(0, 0, b"Voor de crash", 9000, 20);
    app.verify(0, 1, 25);
    if case == CUT_TXN {
        app.write(1, 0, b"Tweede", 300, 30);
        check(app.client.collection(0, 1, 0, b"Werk") == Ok(1), 35);
        app.phase(1);
        // The kernel kills the VM in the middle of this transaction's table.
        let moving = [
            sub(0, wire::SUB_META, 1, 5, 1),
            sub(1, wire::SUB_META, 1, 5, 1),
        ];
        let _ = app.client.transact(0, &moving);
        fail(36);
    }
    app.phase(1);
    // The kernel kills the VM in the middle of this second commit.
    let object = app.object(0);
    let _ = app
        .client
        .write(object, 1, b"Tijdens de crash", MAX_LENGTH, fill(object, 2));
    fail(30);
}

/// Report what the cold instance finds; the host oracle decides whether it
/// is exactly the state before the interrupted commit.
fn recover_cut(app: &mut App) {
    if app.domain == 1 {
        let (_, count) = app.list(0, 10);
        check(count == 0, 11);
        app.report(12);
        return;
    }
    // One build serves every parent: object 0 never-used or at the install's
    // revision n is the install-cut parent (the other parents leave 1 or 2).
    let first = match app.client.stat(app.object(0)) {
        Ok(stat) if stat.revision != INSTALL_VERSIONS.len() as u64 => stat,
        Ok(_) => return recover_install(app, true),
        Err((Status::NotFound, 0)) => return recover_install(app, false),
        Err(_) => fail(20),
    };
    check(first.revision == 1 || first.revision == 2, 21);
    let length = if first.revision == 1 {
        9000
    } else {
        MAX_LENGTH
    };
    let stat = app.verify(0, first.revision, 22);
    check(stat.length == length, 24);
    support::put(480, stat.revision);
    support::put(488, u64::from(stat.collection));
    if let Ok(second) = app.client.stat(app.object(1)) {
        app.verify(1, 1, 26);
        support::put(496, second.revision);
    }
    let object = app.object(2);
    check(
        app.client.write(object, 0, b"Nee", 1, fill(object, 1)) == Err(Status::Access),
        28,
    );
    app.report(30);
}

fn install_manifest() -> Manifest {
    let (origin_revision, origin_created, origin_modified) = INSTALL_ORIGIN;
    let mut manifest = Manifest {
        origin_revision,
        origin_created,
        origin_modified,
        count: INSTALL_VERSIONS.len(),
        ..Manifest::default()
    };
    for (index, (version, (name, length))) in manifest
        .versions
        .iter_mut()
        .zip(INSTALL_VERSIONS)
        .enumerate()
    {
        version.origin = INSTALL_ORIGIN_0 - index as u64;
        version.length = length;
        version.name_length = name.len();
        version.name[..name.len()].copy_from_slice(name.as_bytes());
    }
    manifest
}

/// The installed collection, present at revision 1 with its fixture origin.
fn install_collection_is(app: &mut App, step: u64) {
    let (index, name, origin) = INSTALL_COLLECTION;
    app.collection_is(0, index, Some(name), 1, step);
    check(
        app.client.collection_provenance(0, index, 1) == Ok(origin),
        step + 2,
    );
}

/// The complete installed state of object 0: STAT, VERSION_STAT for every
/// version, every version's bytes, PROVENANCE_STAT and the collection.
fn installed(app: &mut App, step: u64) {
    let n = INSTALL_VERSIONS.len() as u64;
    let object = app.object(0);
    let stat = app.stat(0, step);
    let (_, clock) = app.client.domain_stat(0).unwrap_or_else(|_| fail(step + 1));
    let (tags, kind, collection) = INSTALL_META;
    check(
        stat.revision == n
            && stat.history_count == INSTALL_VERSIONS.len() - 1
            && (stat.tags, stat.kind, stat.collection) == (tags, kind, collection)
            && stat.created == clock
            && stat.modified == clock
            && stat.name[..stat.name_length] == *INSTALL_VERSIONS[0].0.as_bytes(),
        step + 2,
    );
    for (selector, (name, length)) in INSTALL_VERSIONS.into_iter().enumerate() {
        let revision = n - selector as u64;
        let info = if selector == 0 {
            HistoryInfo {
                revision: stat.version_revision,
                length: stat.length,
                content_sum: stat.content_sum,
            }
        } else {
            stat.history[selector - 1]
        };
        check(info.revision == revision && info.length == length, step + 3);
        let version = app
            .client
            .version_stat(object, n, selector)
            .unwrap_or_else(|_| fail(step + 4));
        check(
            if selector == 0 {
                version.present && version.created == clock && version.modified == clock
            } else {
                version.version_revision == revision && version.name() == name.as_bytes()
            },
            step + 5,
        );
        app.verify_version(0, n, selector, info, revision, step + 6);
    }
    let provenance = app
        .client
        .provenance(object, n)
        .unwrap_or_else(|_| fail(step + 7));
    check(
        provenance.revision == n
            && provenance.installed == INSTALL_VERSIONS.len()
            && (
                provenance.origin_revision,
                provenance.origin_created,
                provenance.origin_modified,
            ) == INSTALL_ORIGIN
            && provenance.version_count == INSTALL_VERSIONS.len()
            && (0..INSTALL_VERSIONS.len())
                .all(|i| provenance.versions[i] == INSTALL_ORIGIN_0 - i as u64),
        step + 8,
    );
    install_collection_is(app, step + 10);
    support::put(480, stat.revision);
    support::put(488, u64::from(stat.collection));
    support::put(496, provenance.installed as u64);
}

/// S09 L2: INSTALL_COLLECTION, then INSTALL of the fixture into never-used
/// slots (install, install-cut), or their cold read-only re-check.
fn install_flow(app: &mut App, case: u64) {
    if app.domain == 1 {
        let (_, count) = app.list(0, 10);
        check(count == 0, 11);
        if case != INSTALL_CUT {
            app.report(12);
        }
        return;
    }
    let manifest = install_manifest();
    if case == INSTALL_RECOVER {
        installed(app, 200);
        check(
            app.client
                .install_begin(app.object(1), &manifest, INSTALL_META)
                == Err(Status::Access),
            230,
        );
        check(
            app.client.install_collection(0, 2, b"Nee", 1) == Err((Status::Access, 0)),
            231,
        );
        app.report(240);
        return;
    }
    let (index, name, origin) = INSTALL_COLLECTION;
    check(
        app.client
            .install_collection(0, index, name.as_bytes(), origin)
            == Ok(1),
        200,
    );
    app.commits += 1;
    // Never-used slot, absent collection: Range before any reservation.
    let (tags, kind, _) = INSTALL_META;
    check(
        app.client
            .install_begin(app.object(1), &manifest, (tags, kind, 2))
            == Err(Status::Range),
        201,
    );
    if case == INSTALL_CUT {
        // Only after the collection commit is durable: the kernel then
        // kills the VM inside the INSTALL_COMMIT table write below.
        app.phase(1);
    }
    let object = app.object(0);
    let n = INSTALL_VERSIONS.len() as u64;
    let result = app.client.install(
        object,
        &manifest,
        INSTALL_META,
        |version, offset, buffer| {
            for (index, value) in buffer.iter_mut().enumerate() {
                *value = byte(object, n - version as u64, offset + index);
            }
        },
    );
    check(case != INSTALL_CUT, 202);
    check(matches!(result, Ok((revision, _)) if revision == n), 203);
    app.commits += 1;
    installed(app, 210);
    // The slot is used now: Stale, with no table write (kernel INSTALL_RESULT).
    check(
        app.client.install_begin(object, &manifest, INSTALL_META) == Err(Status::Stale),
        232,
    );
    app.report(240);
}

/// recover-cut after install-cut: exactly never-used or fully installed.
fn recover_install(app: &mut App, complete: bool) {
    if complete {
        installed(app, 300);
    } else {
        install_collection_is(app, 320);
    }
    check(
        app.client
            .install_begin(app.object(1), &install_manifest(), INSTALL_META)
            == Err(Status::Access),
        330,
    );
    app.report(340);
}

/// The portal's verify-then-restore flow (spec: the trailer follows the last
/// object, so restore may only run on bytes that `verify` accepted).
#[cfg(feature = "archive-portal")]
fn verified_restore(
    disk: &mut NativeDisk,
    scratch: &mut [u8; MAX_LENGTH],
    app: &mut App,
) -> Result<Report, (Status, Report)> {
    archive::verify(&mut DiskSource::new(&mut *disk, ARCHIVE_SECTORS), scratch)
        .map_err(|status| (status, Report::default()))?;
    archive::restore(
        &mut DiskSource::new(disk, ARCHIVE_SECTORS),
        scratch,
        &mut app.client,
        0,
    )
}

/// Submits one block request again while the broker answers -8 (its own
/// transient state), yielding in between; any other answer is returned.
#[cfg(feature = "archive-portal")]
fn submit(
    mut call: impl FnMut() -> Result<u64, nieuw_os_sdk_runtime::Error>,
) -> Result<u64, nieuw_os_sdk_runtime::Error> {
    for _ in 0..disk::POLL_LIMIT {
        match call() {
            Err(error) if error.is_again() => {
                let _ = native::yield_now();
            }
            result => return result,
        }
    }
    Err(nieuw_os_sdk_runtime::Error::Kernel(-13))
}

/// S09 backup portal: the only process with a capability on the archive
/// range, plus a volume client bound to domain 0 like any app.
#[cfg(feature = "archive-portal")]
fn portal(app: &mut App, case: u64) {
    let device = BlockDevice::from_handle(
        Handle::from_raw(support::word(ARCHIVE_HANDLE)).unwrap_or_else(|_| fail(700)),
    );
    // The capability ends at its scope: the first sector past it is refused.
    let sectors = if case == ARCHIVE_FULL {
        FULL_SECTORS
    } else {
        ARCHIVE_SECTORS
    };
    check(native::block_read(device, sectors, 0, 128).is_err(), 701);
    // SAFETY: the kernel maps HEAP..HEAP+64 KiB private, zeroed RW/NX for this
    // process before entry and nothing else here uses it; this is the only
    // reference, held for the rest of the portal run on one thread.
    let scratch = unsafe { &mut *(HEAP as *mut [u8; MAX_LENGTH]) };
    let mut disk = NativeDisk { device };
    // The broker serves one block request at a time across both disks (others
    // get Busy), so
    // archive I/O must not overlap service I/O: slot 2 waits for phase 2,
    // and every case reaches a READY service before its first archive read.
    app.phase(1);
    match case {
        ARCHIVE_EXPORT => {
            // The source is the recovered normal fixture, mounted read-only.
            verify_final(app, FINAL_A, 702);
            let mut sink = DiskSink::new(&mut disk, ARCHIVE_SECTORS);
            let summary =
                archive::export(&mut app.client, 0, &mut sink).unwrap_or_else(|_| fail(710));
            sink.finish().unwrap_or_else(|_| fail(711));
            check(
                summary.objects == FINAL_A.len() && summary.collections == 2,
                712,
            );
            // Read the stored bytes back through the same capability.
            let stored = archive::verify(&mut DiskSource::new(&mut disk, ARCHIVE_SECTORS), scratch)
                .unwrap_or_else(|_| fail(713));
            check(stored == summary, 714);
            let object = app.object(7);
            check(
                app.client.write(object, 0, b"Nee", 1, fill(object, 1)) == Err(Status::Access),
                715,
            );
        }
        ARCHIVE_RESTORE => {
            // Source loss: a fresh volume whose domain starts empty.
            check(app.list(0, 720).1 == 0, 721);
            let report = verified_restore(&mut disk, scratch, app).unwrap_or_else(|_| fail(723));
            let all = FINAL_A.iter().fold(0u8, |mask, item| mask | 1 << item.0);
            check(
                report
                    == Report {
                        restored: all,
                        ..Report::default()
                    },
                724,
            );
            for (index, _, content, length, name, collection, _) in FINAL_A {
                let stat = app.verify(*index, *content, 730);
                check(
                    stat.length == *length
                        && &stat.name[..stat.name_length] == name.as_bytes()
                        && stat.collection == *collection
                        && stat.history_count == 0,
                    735,
                );
            }
            check(app.list(0, 736).1 == FINAL_A.len(), 737);
        }
        ARCHIVE_CORRUPT => {
            // Corrupt archive: the same flow as restore refuses it at verify,
            // so restore never runs and nothing is restored.
            check(app.list(0, 738).1 == 0, 739);
            check(
                verified_restore(&mut disk, scratch, app)
                    == Err((Status::Corrupt, Report::default())),
                740,
            );
            check(app.list(0, 741).1 == 0, 742);
        }
        ARCHIVE_CONFLICT => {
            // Restore into a domain that already holds data. Slot 3 is taken
            // under another name and slot 6 uses the name of archive slot 4:
            // both conflicts stay untouched. Collection 2 holds another name,
            // so archive slot 1 lands outside a collection (kind kept);
            // collection 1 is equal, so archive slot 2 reuses it.
            check(app.list(0, 770).1 == 0, 771);
            app.write(3, 0, b"Bezet", 77, 772);
            app.write(6, 0, b"Herstart", 33, 776);
            check(app.client.collection(0, 1, 0, b"Werk") == Ok(1), 780);
            check(app.client.collection(0, 2, 0, b"Anders") == Ok(1), 781);
            let report = verified_restore(&mut disk, scratch, app).unwrap_or_else(|_| fail(782));
            check(
                report
                    == Report {
                        restored: 1 << 1 | 1 << 2 | 1 << 5,
                        conflicts: 1 << 3 | 1 << 4,
                        uncollected: 1 << 1,
                        bare: 0,
                    },
                783,
            );
            for (index, name, length, step) in [(3, "Bezet", 77, 784), (6, "Herstart", 33, 787)] {
                let stat = app.verify(index, 1, step);
                check(
                    stat.revision == 1
                        && stat.length == length
                        && &stat.name[..stat.name_length] == name.as_bytes()
                        && stat.collection == 0,
                    step + 2,
                );
            }
            let missing = app.object(4);
            check(
                matches!(app.client.stat(missing), Err((Status::NotFound, 0))),
                790,
            );
            for (index, revision, collection, kind) in [(1, 2, 0, 2), (2, 2, 1, 0), (5, 1, 0, 0)] {
                let &(_, _, content, length, name, _, _) = FINAL_A
                    .iter()
                    .find(|item| item.0 == index)
                    .unwrap_or_else(|| fail(791));
                let stat = app.verify(index, content, 792);
                check(
                    stat.revision == revision
                        && stat.length == length
                        && &stat.name[..stat.name_length] == name.as_bytes()
                        && stat.collection == collection
                        && stat.kind == kind
                        && stat.history_count == 0,
                    794,
                );
            }
            app.collection_is(0, 1, Some("Werk"), 1, 795);
            app.collection_is(0, 2, Some("Anders"), 1, 797);
            check(app.list(0, 799).1 == 5, 800);
        }
        ARCHIVE_FULL => {
            // Same read-only source as export, but the target is too small.
            verify_final(app, FINAL_A, 802);
            let mut sink = DiskSink::new(&mut disk, FULL_SECTORS);
            check(
                archive::export(&mut app.client, 0, &mut sink) == Err(Status::Full),
                810,
            );
            // No finish: a failed export is never padded or flushed.
            // The kernel also refuses a write past the cut scope.
            check(
                native::block_write(device, FULL_SECTORS, &[0; 512]).is_err(),
                811,
            );
            // The unfinished range is not an archive: its header claims more
            // than the range holds.
            check(
                archive::verify(&mut DiskSource::new(&mut disk, FULL_SECTORS), scratch)
                    == Err(Status::Corrupt),
                812,
            );
            // The source volume still answers; the host checks its bytes.
            check(app.list(0, 813).1 == FINAL_A.len(), 814);
            let object = app.object(7);
            check(
                app.client.write(object, 0, b"Nee", 1, fill(object, 1)) == Err(Status::Access),
                815,
            );
        }
        ARCHIVE_USTAR => {
            // D6(a) user export: archive the read-only fixture into the lower
            // half, then convert that stored archive into the upper half. It
            // reuses the portal's existing `unsafe` HEAP scratch cast above and
            // adds no further `unsafe`.
            verify_final(app, FINAL_A, 820);
            let mut sink = DiskSink::new(&mut disk, USTAR_FIRST);
            let summary =
                archive::export(&mut app.client, 0, &mut sink).unwrap_or_else(|_| fail(830));
            sink.finish().unwrap_or_else(|_| fail(831));
            let (checked, names) =
                ustar::check(&mut DiskSource::new(&mut disk, USTAR_FIRST), scratch)
                    .unwrap_or_else(|_| fail(832));
            check(checked == summary && !names.refused(), 833);
            let mut target = Shifted(NativeDisk { device });
            let mut out = DiskSink::new(&mut target, ARCHIVE_SECTORS - USTAR_FIRST);
            // Reasons are judged again, so a caller cannot clear a refusal: a
            // reserved name gives Invalid before the first byte reaches `out`.
            let mut refused = names;
            refused.object[0].bytes[..3].copy_from_slice(b"CON");
            refused.object[0].length = 3;
            check(
                ustar::from_archive(
                    &mut DiskSource::new(&mut disk, USTAR_FIRST),
                    scratch,
                    &refused,
                    &mut out,
                ) == Err(Status::Invalid),
                834,
            );
            check(
                ustar::from_archive(
                    &mut DiskSource::new(&mut disk, USTAR_FIRST),
                    scratch,
                    &names,
                    &mut out,
                ) == Ok(summary),
                835,
            );
            let length = out.finish().unwrap_or_else(|_| fail(836));
            check(length > 0 && length.is_multiple_of(SECTOR as u64), 837);
            // The host reads the stream with Python tarfile at this length.
            support::put(512, length);
        }
        ARCHIVE_ABSENT => {
            // The kernel granted a range of a disk that is not there: the
            // broker answers NoDevice, so nothing is written or finalized.
            verify_final(app, FINAL_A, 840);
            let mut sink = DiskSink::new(&mut disk, ARCHIVE_SECTORS);
            check(
                archive::export(&mut app.client, 0, &mut sink) == Err(Status::NoDevice),
                850,
            );
            check(
                archive::verify(&mut DiskSource::new(&mut disk, ARCHIVE_SECTORS), scratch)
                    == Err(Status::NoDevice),
                851,
            );
            check(app.list(0, 852).1 == FINAL_A.len(), 853);
            let object = app.object(7);
            check(
                app.client.write(object, 0, b"Nee", 1, fill(object, 1)) == Err(Status::Access),
                854,
            );
        }
        ARCHIVE_CUT => {
            // Change the domain so the new archive differs from the older one
            // on the disk from its header on, then export over it. The kernel
            // stops the guest after a fixed number of archive writes.
            verify_final(app, FINAL_A, 860);
            // A new collection needs no content chunk (the fixture is near
            // full) and changes the header's collection count and generation.
            check(app.client.collection(0, 3, 0, b"Nieuw") == Ok(1), 870);
            let mut sink = DiskSink::new(&mut disk, ARCHIVE_SECTORS);
            let _ = archive::export(&mut app.client, 0, &mut sink);
            // Not reached when the cut happens.
            fail(879);
        }
        ARCHIVE_LATCH => {
            use nieuw_os_sdk_runtime::Error::Kernel;
            // A fresh domain; the service is READY before any device-1 request.
            check(app.list(0, 880).1 == 0, 881);
            let written = [0xa5; SECTOR];
            let refused = [0x5a; SECTOR];
            // (a) A device-1 WRITE answered Ok.
            check(disk.write(1, &written).is_ok(), 882);
            // (b) The kernel fails this WRITE's completion. Without any poll,
            // the next WRITE gets -8 while it is in flight and -5 once the
            // Policy latched the entry; only then is its ticket polled.
            let ticket = native::block_write(device, 2, &written).unwrap_or_else(|_| fail(883));
            check(
                submit(|| native::block_write(device, 3, &refused)) == Err(Kernel(-5)),
                884,
            );
            check(
                native::block_wait(device, ticket, &mut [], disk::POLL_LIMIT) == Err(Kernel(-16)),
                885,
            );
            // (c) READ and FLUSH stay admitted; -8 only while the reset runs.
            let ticket =
                submit(|| native::block_read(device, 1, 0, 128)).unwrap_or_else(|_| fail(886));
            let mut back = [0; 128];
            check(
                native::block_wait(device, ticket, &mut back, disk::POLL_LIMIT).is_ok(),
                887,
            );
            check(back == written[..128], 888);
            let ticket = submit(|| native::block_flush(device)).unwrap_or_else(|_| fail(889));
            check(
                native::block_wait(device, ticket, &mut [], disk::POLL_LIMIT).is_ok(),
                890,
            );
            check(
                native::block_write(device, 4, &refused) == Err(Kernel(-5)),
                891,
            );
            // (e) A device-0 WRITE (one service commit) is admitted.
            app.write(0, 0, b"Na", 1, 892);
        }
        ARCHIVE_LATCH_0X44 => {
            use nieuw_os_sdk_runtime::Error::Kernel;
            let handle = |offset| {
                BlockDevice::from_handle(
                    Handle::from_raw(support::word(offset)).unwrap_or_else(|_| fail(940)),
                )
            };
            // READ|WRITE on [15, 25) (part of [10, 20) only) and on [25, 35)
            // (no entry); READ only on the archive object's own [0, 15).
            let (partial, none, read_only) = (handle(576), handle(584), handle(592));
            check(app.list(0, 941).1 == 0, 942);
            let (first, refused, last) = ([0xa5; SECTOR], [0x5a; SECTOR], [0x3c; SECTOR]);
            // (f3) No entry, or only part of one: -5. (f2) READ only, although
            // the scope wholly contains both open entries: refused.
            for (device, step) in [(none, 943), (partial, 944), (read_only, 945)] {
                check(native::block_latch(device) == Err(Kernel(-5)), step);
            }
            // Nothing was latched: a WRITE into each entry is admitted.
            check(disk.write(3, &first).is_ok(), 946);
            check(disk.write(12, &first).is_ok(), 947);
            // (f1, f4) [0, 15) wholly contains [0, 10), which then refuses
            // WRITEs, and only part of [10, 20), which stays open.
            check(native::block_latch(device) == Ok(0), 948);
            check(
                native::block_write(device, 3, &refused) == Err(Kernel(-5)),
                949,
            );
            check(disk.write(12, &last).is_ok(), 950);
        }
        _ => fail(769),
    }
    app.phase(2);
    app.report(750);
}

/// Writable source step S1-S5 of the export case (spec section 3.2), so that
/// archive origins differ from the values a destination assigns. S1 and S2
/// read the current revision first; S3 and S4 expect the collection
/// revisions of the `archive-export` parent (1, 1, absent) and fail closed
/// otherwise. The host oracle derives the source state from that image.
#[cfg(feature = "l2-portal")]
fn l2_source_step(app: &mut App) {
    // S1: two consecutive zero-length WRITEs to object 2 ("Leeg"): no chunk.
    let stat = app.stat(2, 900);
    let mut name = [0; volume_core::NAME];
    let length = stat.name_length;
    name[..length].copy_from_slice(&stat.name[..length]);
    check(stat.length == 0, 901);
    let revision = app.write(2, stat.revision, &name[..length], 0, 902);
    app.write(2, revision, &name[..length], 0, 906);
    // S2: META on object 5 (tags change only).
    let stat = app.stat(5, 910);
    app.meta(
        5,
        stat.revision,
        stat.tags ^ 1,
        stat.kind,
        stat.collection,
        911,
    );
    app.commits += 1;
    // S3: rename collections 1 and 2; S4: create collection 3, rename it.
    // Steps start at 916: `meta` above reports 911 and 912 (`step + 1`).
    for (index, expected, new_name, step) in [
        (1, 1, &b"Werkmap"[..], 916),
        (2, 1, &b"Thuis"[..], 917),
        (3, 0, &b"Nieuw"[..], 918),
        (3, 1, &b"Nieuwer"[..], 919),
    ] {
        check(
            app.client.collection(0, index, expected, new_name) == Ok(expected + 1),
            step,
        );
        app.commits += 1;
    }
    // S5: a zero-length WRITE, then RESTORE of the older version, on objects 1
    // and 5: each gets a two-version chain, so no archive origin equals the
    // live value a destination assigns (discrimination invariant, spec
    // section 2). Chunk-neutral. `write` reports up to `step + 4` here.
    for (index, step) in [(1, 920), (5, 930)] {
        let stat = app.stat(index, step);
        let length = stat.name_length;
        name = [0; volume_core::NAME];
        name[..length].copy_from_slice(&stat.name[..length]);
        let revision = app.write(index, stat.revision, &name[..length], 0, step + 1);
        check(
            app.client.restore(app.object(index), revision, 1) == Ok(revision + 1),
            step + 5,
        );
        app.commits += 1;
    }
}

/// Provenance of the restored domain against the verified plan: the number
/// of objects and collections whose installed origins differ from the
/// archive values (0 expected). An absent archive slot must stay never-used,
/// exactly `(NotFound, 0)` as in the host oracle. Content, metadata and
/// deleted collections are host-checked. With `partial` (an interrupted
/// restore, case 30), a planned object or collection may instead be exactly
/// never-used, as `archive::never_used` defines it; the returned masks name
/// the fully installed objects (by slot) and collections (bit index - 1).
#[cfg(feature = "l2-portal")]
fn l2_mismatches(app: &mut App, plan: &ArchiveV2, partial: bool) -> (u64, u64, u64) {
    let (mut mismatches, mut restored, mut collections) = (0, 0, 0);
    for c in plan.collections[..plan.collection_count]
        .iter()
        .filter(|c| c.present)
    {
        if partial
            && app
                .client
                .collection_stat(0, c.index)
                .is_ok_and(|stat| stat.revision == 0)
        {
            continue;
        }
        let origin = app.client.collection_provenance(0, c.index, 1);
        mismatches += u64::from(origin != Ok(c.revision));
        collections |= 1 << (c.index - 1);
    }
    for o in &plan.objects[..plan.object_count] {
        let object = app.object(o.slot as u64);
        let n = archive::replayable(o);
        let never_used =
            |app: &mut App| matches!(app.client.stat(object), Err((Status::NotFound, 0)));
        if n == 0 {
            mismatches += u64::from(!never_used(app));
            continue;
        }
        if partial && never_used(app) {
            continue;
        }
        restored |= 1 << o.slot;
        let equal = match app.client.provenance(object, n as u64) {
            Ok(p) => {
                p.installed == n
                    && p.version_count == n
                    && (p.origin_revision, p.origin_created, p.origin_modified)
                        == (o.revision, o.created, o.modified)
                    && (0..n).all(|i| p.versions[i] == o.versions[i].revision)
            }
            Err(_) => false,
        };
        mismatches += u64::from(!equal);
    }
    (mismatches, restored, collections)
}

#[cfg(feature = "l2-portal")]
fn l2_report(report: &L2Report, result: Status) {
    support::put(L2_WORDS.restored, u64::from(report.restored));
    support::put(L2_WORDS.collections, u64::from(report.collections));
    support::put(L2_WORDS.committed, report.committed as u64);
    support::put(L2_WORDS.skipped, report.skipped as u64);
    support::put(L2_WORDS.status, result as i32 as u64);
}

/// S09 phase 3 backup portal on the device-1 archive disk: v2 export with
/// a writable source step (25), cold L2 restore (26), a corrupt archive
/// (27) and a used domain after a cold boot (28). Each case runs in its own
/// non-inlined function, so its `ArchiveV2` copies leave the stack before
/// the next phase and never stay in the frame of `run` (spec section 5.2).
#[cfg(feature = "l2-portal")]
#[inline(never)]
fn l2_portal(app: &mut App, case: u64) {
    let device = BlockDevice::from_handle(
        Handle::from_raw(support::word(ARCHIVE_HANDLE)).unwrap_or_else(|_| fail(700)),
    );
    // The capability ends at its scope: the first sector past it is refused.
    check(
        native::block_read(device, ARCHIVE_SECTORS, 0, 128).is_err(),
        701,
    );
    // SAFETY: the kernel maps HEAP..HEAP+64 KiB private, zeroed RW/NX for this
    // process before entry and nothing else here uses it; this is the only
    // reference, held for the rest of the portal run on one thread.
    let scratch = unsafe { &mut *(HEAP as *mut [u8; MAX_LENGTH]) };
    // Archive I/O never overlaps another app's service I/O: slot 2 waits for
    // phase 2 (kernel gate), as in the v1 portal.
    app.phase(1);
    if case != L2_EXPORT {
        // A READ-only grant: the capability check refuses the write with
        // capability-core `Error::Access` (-5), before any broker state.
        check(
            native::block_write(device, 0, &[0; 512])
                == Err(nieuw_os_sdk_runtime::Error::Kernel(-5)),
            702,
        );
    }
    match case {
        L2_EXPORT => l2_export(app, device, scratch),
        // 29 runs the intact restore; the kernel cuts the power inside it.
        L2_RESTORE | L2_RESTORE_CUT | L2_CORRUPT => {
            l2_restore(app, device, scratch, case != L2_CORRUPT)
        }
        L2_USED | RECOVER_L2_CUT => l2_used(app, device, scratch, case == RECOVER_L2_CUT),
        _ => fail(769),
    }
    app.phase(2);
    app.report(750);
}

/// Case 25: S1-S4, then a v2 export to the device-1 disk, read back and
/// verified through the same capability.
#[cfg(feature = "l2-portal")]
#[inline(never)]
fn l2_export(app: &mut App, device: BlockDevice, scratch: &mut [u8; MAX_LENGTH]) {
    l2_source_step(app);
    let mut disk = NativeDisk { device };
    let mut sink = DiskSink::new(&mut disk, ARCHIVE_SECTORS);
    let export = archive::export_v2(&mut app.client, 0, &mut sink).unwrap_or_else(|_| fail(710));
    let length = sink.finish().unwrap_or_else(|_| fail(711));
    let plan = archive::verify_v2(&mut DiskSource::new(&mut disk, ARCHIVE_SECTORS), scratch)
        .unwrap_or_else(|_| fail(712));
    check(plan == export.archive, 713);
    support::put(L2_WORDS.skipped, export.omitted as u64);
    support::put(L2_WORDS.length, length);
}

/// Cases 26 and 27: a cold L2 restore into a fresh volume (source loss);
/// a damaged archive is refused at verification, before any service request.
#[cfg(feature = "l2-portal")]
#[inline(never)]
fn l2_restore(app: &mut App, device: BlockDevice, scratch: &mut [u8; MAX_LENGTH], intact: bool) {
    check(app.list(0, 720).1 == 0, 721);
    let mut plan = ArchiveV2::default();
    let result = archive::restore_native_l2(
        || DiskSource::new(NativeDisk { device }, ARCHIVE_SECTORS),
        scratch,
        &mut app.client,
        0,
        &mut plan,
    );
    if intact {
        let report = result.unwrap_or_else(|_| fail(722));
        l2_report(&report, Status::Ok);
        app.commits += report.committed as u64;
        support::put(L2_WORDS.mismatches, l2_mismatches(app, &plan, false).0);
    } else {
        let (status, report) = result.err().unwrap_or_else(|| fail(723));
        check(status == Status::Corrupt && report.committed == 0, 724);
        l2_report(&report, status);
        check(app.list(0, 725).1 == 0, 726);
    }
}

/// Case 28: the restored domain after a cold boot, both disks read-only.
/// Provenance still matches, and a second restore is refused as `Stale`.
/// Case 30 (`partial`): the domain a power cut interrupted mid-restore; each
/// planned object and collection is fully installed or exactly never-used,
/// and the observed masks go to words 520/528 (the kernel and host check them).
#[cfg(feature = "l2-portal")]
#[inline(never)]
fn l2_used(app: &mut App, device: BlockDevice, scratch: &mut [u8; MAX_LENGTH], partial: bool) {
    // Reach a READY service before the first archive read: the broker serves
    // one block request at a time across both disks, so an archive read
    // during the service's recovery fails that recovery (as in `l2_restore`).
    let listed = app.list(0, 728).1;
    check(partial || listed == 5, 729);
    let plan = archive::verify_v2(
        &mut DiskSource::new(NativeDisk { device }, ARCHIVE_SECTORS),
        scratch,
    )
    .unwrap_or_else(|_| fail(730));
    let (mismatches, restored, collections) = l2_mismatches(app, &plan, partial);
    check(restored.count_ones() as usize == listed, 734);
    support::put(L2_WORDS.mismatches, mismatches);
    let object = app.object(7);
    check(
        app.client.write(object, 0, b"Nee", 1, fill(object, 1)) == Err(Status::Access),
        731,
    );
    let (status, report) = archive::restore_native_l2(
        || DiskSource::new(NativeDisk { device }, ARCHIVE_SECTORS),
        scratch,
        &mut app.client,
        0,
        &mut ArchiveV2::default(),
    )
    .err()
    .unwrap_or_else(|| fail(732));
    check(status == Status::Stale && report.committed == 0, 733);
    l2_report(&report, status);
    if partial {
        // After l2_report: the refused restore's empty masks must not hide them.
        support::put(L2_WORDS.restored, restored);
        support::put(L2_WORDS.collections, collections);
    }
}

fn run() {
    check(native::check_abi().is_ok(), 1);
    let slot = support::word(0);
    check((1..=2).contains(&slot), 2);
    let case = support::word(112);
    if case == CORRUPT {
        // The service refuses to start; this app must not reach any document.
        support::put(40, 1);
        native::exit(0);
    }
    let send =
        SendEndpoint::from_handle(Handle::from_raw(support::word(64)).unwrap_or_else(|_| fail(3)));
    let receive = ReceiveEndpoint::from_handle(
        Handle::from_raw(support::word(56)).unwrap_or_else(|_| fail(4)),
    );
    // Apps hold endpoint handles only. Block calls with the own endpoint
    // handle or a forged handle number must be refused by the kernel broker.
    let own = BlockDevice::from_handle(send.handle());
    check(native::block_read(own, 0, 0, 128).is_err(), 6);
    check(native::block_flush(own).is_err(), 7);
    let forged = Handle::from_raw((7 << 32) | 200).unwrap_or_else(|_| fail(8));
    check(
        native::block_read(BlockDevice::from_handle(forged), 0, 0, 128).is_err(),
        9,
    );
    let transport = Native {
        send,
        receive,
        service: support::word(176),
    };
    let domain = (slot - 1) as usize;
    let mut app = App {
        client: Client::new(transport, support::word(32), 1 + slot * 1_000_000),
        domain,
        verified_bytes: 0,
        verified_docs: 0,
        commits: 0,
    };
    match case {
        #[cfg(not(any(feature = "archive-portal", feature = "l2-portal")))]
        NORMAL if domain == 0 => normal_a(&mut app),
        #[cfg(not(any(feature = "archive-portal", feature = "l2-portal")))]
        NORMAL => normal_b(&mut app),
        #[cfg(not(any(feature = "archive-portal", feature = "l2-portal")))]
        RECOVER => recover(&mut app),
        #[cfg(not(any(feature = "archive-portal", feature = "l2-portal")))]
        CUT_DATA | CUT_TABLE | CUT_TXN => cut(&mut app, case),
        #[cfg(not(any(feature = "archive-portal", feature = "l2-portal")))]
        RECOVER_CUT => recover_cut(&mut app),
        #[cfg(not(any(feature = "archive-portal", feature = "l2-portal")))]
        INSTALL | INSTALL_RECOVER | INSTALL_CUT => install_flow(&mut app, case),
        #[cfg(feature = "archive-portal")]
        ARCHIVE_EXPORT | ARCHIVE_RESTORE | ARCHIVE_CORRUPT | ARCHIVE_CONFLICT | ARCHIVE_FULL
        | ARCHIVE_USTAR | ARCHIVE_ABSENT | ARCHIVE_CUT | ARCHIVE_LATCH | ARCHIVE_LATCH_0X44
            if domain == 0 =>
        {
            portal(&mut app, case)
        }
        // The ordinary app holds no archive authority; it reports its domain
        // only after the portal finished its archive I/O (kernel gate 2).
        #[cfg(feature = "archive-portal")]
        ARCHIVE_EXPORT | ARCHIVE_RESTORE | ARCHIVE_CORRUPT | ARCHIVE_CONFLICT | ARCHIVE_FULL
        | ARCHIVE_USTAR | ARCHIVE_ABSENT | ARCHIVE_CUT | ARCHIVE_LATCH | ARCHIVE_LATCH_0X44 => {
            app.phase(1);
            app.wait_gate(2);
            app.report(760)
        }
        #[cfg(feature = "l2-portal")]
        L2_EXPORT | L2_RESTORE | L2_CORRUPT | L2_USED | L2_RESTORE_CUT | RECOVER_L2_CUT
            if domain == 0 =>
        {
            l2_portal(&mut app, case)
        }
        #[cfg(feature = "l2-portal")]
        L2_EXPORT | L2_RESTORE | L2_CORRUPT | L2_USED | L2_RESTORE_CUT | RECOVER_L2_CUT => {
            app.phase(1);
            app.wait_gate(2);
            app.report(760)
        }
        _ => fail(5),
    }
    support::put(40, 1);
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    run();
    native::exit(0)
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(999)
}

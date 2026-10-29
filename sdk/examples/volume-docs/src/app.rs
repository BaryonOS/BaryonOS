#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Native M6e document app. Two instances run in separate address spaces,
//! each with only a send/reply endpoint pair to the volume service. All
//! results are app-reported evidence; the host disk oracle checks the image.

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!("volume-app is only for the Nieuw OS x86_64-unknown-none target");

mod support;

use nieuw_os_sdk_runtime::{native, BlockDevice, Handle, Outgoing, ReceiveEndpoint, SendEndpoint};
use volume_core::{
    checksum_update,
    client::{Client, HistoryInfo, Stat, Transport},
    wire::{self, Header, ListEntry, SubOp},
    Status, CHUNK, COLLECTIONS, FNV_OFFSET, MAX_LENGTH, PER_DOMAIN,
};

const LIMIT: usize = 400_000;

// Case selectors written by the kernel manifest (startup word 112).
const NORMAL: u64 = 0;
const RECOVER: u64 = 1;
const CUT_DATA: u64 = 2;
const CUT_TABLE: u64 = 3;
const RECOVER_CUT: u64 = 4;
const CORRUPT: u64 = 5;
const CUT_TXN: u64 = 6;

fn tick() {
    support::put(8, support::word(8).wrapping_add(1));
}

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
    let first = app.stat(0, 20);
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
        NORMAL if domain == 0 => normal_a(&mut app),
        NORMAL => normal_b(&mut app),
        RECOVER => recover(&mut app),
        CUT_DATA | CUT_TABLE | CUT_TXN => cut(&mut app, case),
        RECOVER_CUT => recover_cut(&mut app),
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

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
    client::{Client, Stat, Transport},
    wire::{self, Header, ListEntry},
    Status, CHUNK, FNV_OFFSET, MAX_LENGTH, NAME, PER_DOMAIN,
};

const LIMIT: usize = 400_000;

// Case selectors written by the kernel manifest (startup word 112).
const NORMAL: u64 = 0;
const RECOVER: u64 = 1;
const CUT_DATA: u64 = 2;
const CUT_TABLE: u64 = 3;
const RECOVER_CUT: u64 = 4;
const CORRUPT: u64 = 5;

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

    /// Read every byte through bounded READ replies and compare against the
    /// fixture for `content_revision`, plus the service-reported checksum.
    fn verify(&mut self, index: u64, content_revision: u64, step: u64) -> Stat {
        let object = self.object(index);
        let stat = self.stat(index, step);
        let mut sum = FNV_OFFSET;
        let mut mismatch = false;
        let read = self
            .client
            .read(object, stat.revision, stat.length, |offset, bytes| {
                for (index, value) in bytes.iter().enumerate() {
                    mismatch |= *value != byte(object, content_revision, offset + index);
                }
                sum = checksum_update(sum, bytes);
            });
        check(
            read == Ok(stat.length) && !mismatch && sum == stat.content_sum,
            step + 1,
        );
        self.verified_docs += 1;
        self.verified_bytes += stat.length as u64;
        stat
    }

    fn list(&mut self, step: u64) -> ([ListEntry; PER_DOMAIN], usize) {
        let mut entries = [ListEntry {
            object: 0,
            revision: 0,
            length: 0,
            name_length: 0,
            name: [0; NAME],
        }; PER_DOMAIN];
        let (count, _) = self
            .client
            .list(self.domain, &mut entries)
            .unwrap_or_else(|_| fail(step));
        (entries, count)
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
        for (offset, mut header) in [
            Header::new(wire::STAT, id, epoch, foreign),
            Header::new(wire::LIST, id + 1, epoch, foreign),
            Header::new(wire::DELETE, id + 2, epoch, foreign + 1),
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
        check(reply.status == Status::Range as i32, step + 3);
        let mut write = Header::new(wire::BEGIN, id + 4, epoch, foreign);
        write.aux = 10;
        write.length = 1;
        write.name_length = 1;
        let reply = self.raw(&write, b"x");
        check(reply.status == Status::Access as i32, step + 4);
        // Malformed: version 4 body; the reply must be a canonical Invalid.
        let mut body = wire::encode(&Header::new(wire::STAT, id + 5, epoch, self.object(0)), &[]);
        body[0] = 4;
        match self.client.transport.call(&body) {
            Ok(reply) => {
                let h = wire::header(&reply);
                check(
                    h.status == Status::Invalid as i32 && h.revision == 0,
                    step + 5,
                );
                check(reply[64..].iter().all(|b| *b == 0), step + 6);
            }
            Err(_) => fail(step + 5),
        }
        let mut read = Header::new(wire::READ, id + 6, epoch, self.object(0));
        read.count = wire::DATA as u32 + 1;
        let reply = self.raw(&read, &[]);
        check(reply.status == Status::Invalid as i32, step + 7);
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

    fn digest(&mut self, step: u64) -> u64 {
        // Domain digest over every object's metadata and content checksum;
        // the host oracle recomputes it from the raw disk image.
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
                }
                Err((Status::NotFound, revision)) => {
                    digest = checksum_update(digest, &[index as u8, 0]);
                    digest = checksum_update(digest, &revision.to_le_bytes());
                }
                Err(_) => fail(step),
            }
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
}

// Final state of the normal fixture, verified again by the cold recovery.
// (object index, revision, content revision, length, name)
const FINAL_A: &[(u64, u64, u64, usize, &str)] = &[
    (1, 2, 1, 129, "Notitie ✓"),
    (2, 1, 1, 0, "Leeg"),
    (3, 40, 40, 5 * CHUNK, "Concept"),
    (4, 1, 1, 5000, "Herstart"),
    (5, 1, 1, MAX_LENGTH, "Archief"),
];
const FINAL_B: &[(u64, u64, u64, usize, &str)] = &[
    (0, 1, 1, 4 * CHUNK, "Foto-index"),
    (1, 1, 1, 300, "Lijst"),
    (2, 20, 20, CHUNK, "Logboek"),
    (3, 1, 1, MAX_LENGTH, "Groot 1"),
    (4, 1, 1, MAX_LENGTH, "Groot 2"),
];

fn normal_a(app: &mut App) {
    let (_, count) = app.list(10);
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
    // S04: 40 overwrites alternate 3 and 5 chunks; old chunks are reused.
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
    let (entries, count) = app.list(90);
    check(count == 5, 91);
    for (index, entry) in entries[..count].iter().enumerate() {
        check(entry.object == app.object(index as u64), 92);
    }
    app.phase(1);
    app.wait_gate(1);
    // Fill exactly to the 40-chunk quota (16 + 1 + 0 + 5 + 2 + 16), then one byte more.
    app.write(5, 0, b"Archief", MAX_LENGTH, 100);
    let object = app.object(6);
    check(
        app.client.write(object, 0, b"Te veel", 1, fill(object, 1)) == Err(Status::Quota),
        105,
    );
    check(app.client.stat(object) == Err((Status::NotFound, 0)), 106);
    support::put(424, 1);
    app.phase(2);
    app.wait_gate(2);
    // Release 16 chunks; domain B then reuses them.
    check(app.client.delete(app.object(0), 1) == Ok(2), 110);
    check(
        app.client.stat(app.object(0)) == Err((Status::NotFound, 2)),
        111,
    );
    app.phase(3);
    verify_final(app, FINAL_A, 120);
    app.report(130);
}

fn normal_b(app: &mut App) {
    let (_, count) = app.list(10);
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
    app.boundaries(50);
    app.phase(1);
    app.wait_gate(2);
    // Domain A holds 40 chunks and B 6: 18 free. The first 16-chunk upload
    // fits; the second exceeds free space while B stays under its quota.
    app.write(3, 0, b"Groot 1", MAX_LENGTH, 60);
    let object = app.object(4);
    check(
        app.client
            .write(object, 0, b"Groot 2", MAX_LENGTH, fill(object, 1))
            == Err(Status::Full),
        65,
    );
    support::put(432, 1);
    app.phase(2);
    app.wait_gate(3);
    app.write(4, 0, b"Groot 2", MAX_LENGTH, 70);
    app.verify(4, 1, 75);
    verify_final(app, FINAL_B, 120);
    app.report(130);
}

fn verify_final(app: &mut App, expected: &[(u64, u64, u64, usize, &str)], step: u64) {
    for (index, revision, content, length, name) in expected {
        let stat = app.verify(*index, *content, step);
        check(
            stat.revision == *revision && stat.length == *length,
            step + 2,
        );
        check(&stat.name[..stat.name_length] == name.as_bytes(), step + 3);
    }
    let (_, count) = app.list(step + 4);
    check(count == expected.len(), step + 5);
}

fn recover(app: &mut App) {
    let expected = if app.domain == 0 { FINAL_A } else { FINAL_B };
    verify_final(app, expected, 20);
    if app.domain == 0 {
        check(
            app.client.stat(app.object(0)) == Err((Status::NotFound, 2)),
            40,
        );
    }
    // The cold recovery instance is read-only: mutations are refused.
    let object = app.object(7);
    check(
        app.client.write(object, 0, b"Nee", 1, fill(object, 1)) == Err(Status::Access),
        41,
    );
    let first = app.object(expected[0].0);
    check(
        app.client.rename(first, expected[0].1, b"Nee") == Err(Status::Access),
        42,
    );
    check(
        app.client.delete(first, expected[0].1) == Err((Status::Access, 0)),
        43,
    );
    app.report(50);
}

fn cut(app: &mut App) {
    if app.domain == 1 {
        let (_, count) = app.list(10);
        check(count == 0, 11);
        return;
    }
    app.write(0, 0, b"Voor de crash", 9000, 20);
    app.verify(0, 1, 25);
    support::put(456, 1);
    // The kernel kills the VM in the middle of this second commit.
    let object = app.object(0);
    let _ = app
        .client
        .write(object, 1, b"Tijdens de crash", MAX_LENGTH, fill(object, 2));
    fail(30);
}

fn recover_cut(app: &mut App) {
    if app.domain == 1 {
        let (_, count) = app.list(10);
        check(count == 0, 11);
        app.report(12);
        return;
    }
    let stat = app.stat(0, 20);
    check(stat.revision == 1 || stat.revision == 2, 21);
    let stat = app.verify(0, stat.revision, 22);
    let expected_length = if stat.revision == 1 { 9000 } else { MAX_LENGTH };
    check(stat.length == expected_length, 24);
    support::put(480, stat.revision);
    let object = app.object(1);
    check(
        app.client.write(object, 0, b"Nee", 1, fill(object, 1)) == Err(Status::Access),
        25,
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
        CUT_DATA | CUT_TABLE => cut(&mut app),
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

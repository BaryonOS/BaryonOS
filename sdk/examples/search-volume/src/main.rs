#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Native U05-T3 search app (step S2: cross-build only, not yet wired into a
//! kernel case). Two instances, one per volume domain, each hold only a
//! send/reply endpoint pair to the unchanged volume service. Each builds the
//! search index of its own domain from LIST and COLLECTION_STAT replies. All
//! results are app-reported; tools/run_search.py recomputes them from the raw
//! image with an independent oracle.

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!("search-app is only for the Nieuw OS x86_64-unknown-none target");

// Byte-identical to the settings-volume copy: the fixed M6c startup page shim.
#[path = "../../volume-docs/src/support.rs"]
mod support;

use nieuw_os_sdk_runtime::{native, BlockDevice, Handle, Outgoing, ReceiveEndpoint, SendEndpoint};
use nieuw_os_search_volume_example::{digest, Error, Hits, IndexError, Search, DIGEST_START};
use volume_core::{
    client::{Client, Transport},
    wire::{self, ListEntry},
    Status, PER_DOMAIN,
};

const LIMIT: usize = 400_000;
/// Case selector written by the kernel manifest (startup word 112).
const SEARCH: u64 = 25;
/// Content length of every fixture object; the bytes follow the volume
/// fixture formula, which tools/volume_disk.py checks independently.
const CONTENT: usize = 16;
/// Every bit of startup word 496 (one per negative check); run_search.py
/// requires exactly this value.
const ALL_NEGATIVES: u64 = (1 << 12) - 1;

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

fn tick() {
    support::put(8, support::word(8).wrapping_add(1));
}

struct Native {
    send: SendEndpoint,
    receive: ReceiveEndpoint,
    service: u64,
}

// Same transport as the settings app (#436): never resend after a send.
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
                        // Never resend: the request may already have been processed.
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

/// Volume fixture byte (volume-docs app.rs `byte`, tools/volume_disk.py).
fn content(object: u64) -> [u8; CONTENT] {
    let mut out = [0; CONTENT];
    for (offset, value) in out.iter_mut().enumerate() {
        // A new object is written at revision 1.
        *value = (object as usize * 31 + 7 + offset * 13 + offset / 251) as u8;
    }
    out
}

/// The hits equal `want` as a set of `(object, revision)`.
fn same(hits: &Hits, want: &[(u64, u64)]) -> bool {
    hits.len() == want.len()
        && want
            .iter()
            .all(|w| hits.iter().any(|h| (h.object, h.revision) == *w))
}

fn query(search: &Search<Native>, text: &[u8], step: u64) -> Hits {
    search.query(text).unwrap_or_else(|_| fail(step))
}

fn expect(search: &Search<Native>, text: &[u8], want: &[(u64, u64)], step: u64) {
    check(same(&query(search, text, step), want), step);
}

fn run() {
    check(native::check_abi().is_ok(), 1);
    let slot = support::word(0);
    check((1..=2).contains(&slot), 2);
    check(support::word(112) == SEARCH, 5);
    let send =
        SendEndpoint::from_handle(Handle::from_raw(support::word(64)).unwrap_or_else(|_| fail(3)));
    let receive = ReceiveEndpoint::from_handle(
        Handle::from_raw(support::word(56)).unwrap_or_else(|_| fail(4)),
    );
    let mut negatives = 0u64;
    // Apps hold endpoint handles only. Block calls with the own endpoint
    // handle or a forged handle number must be refused by the kernel broker
    // (finish() requires an observed device denial per app slot).
    let own = BlockDevice::from_handle(send.handle());
    let forged = Handle::from_raw((7 << 32) | 200).unwrap_or_else(|_| fail(8));
    if native::block_read(own, 0, 0, 128).is_err()
        && native::block_flush(own).is_err()
        && native::block_read(BlockDevice::from_handle(forged), 0, 0, 128).is_err()
    {
        negatives |= 1 << 0;
    }
    let transport = Native {
        send,
        receive,
        service: support::word(176),
    };
    let domain = (slot - 1) as usize;
    let other = 1 - domain;
    let mut client = Client::new(transport, support::word(32), 1 + slot * 1_000_000);
    // N10: the service refuses the other domain by endpoint binding. These two
    // requests are the only ones run_search.py expects in SERVICE:REJECTED.
    if matches!(
        client.stat((other * PER_DOMAIN) as u64),
        Err((Status::Access, _))
    ) {
        negatives |= 1 << 1;
    }
    if client.list(other, &mut [ListEntry::EMPTY; PER_DOMAIN]) == Err(Status::Access) {
        negatives |= 1 << 2;
    }
    let (own_token, other_token): (&[u8], &[u8]) = if domain == 0 {
        (b"alpha", b"beta")
    } else {
        (b"beta", b"alpha")
    };
    let base = (domain * PER_DOMAIN) as u64;
    let (notes, draft, budget) = (base, base + 1, base + 2);
    let mut search = Search::new(client, domain).unwrap_or_else(|_| fail(10));

    // Fixtures. Every forced operation rebuilds from LIST + COLLECTION_STAT.
    check(search.collection(1, 0, b"Projects") == Ok(1), 11);
    let names: [(u64, &[u8], &[u8]); 3] = [
        (notes, b"alpha notes", b"beta notes"),
        (draft, b"alpha draft", b"beta draft"),
        (budget, b"alpha budget", b"beta budget"),
    ];
    for (object, a, b) in names {
        let name = if domain == 0 { a } else { b };
        check(search.write(object, 0, name, &content(object)) == Ok(1), 12);
    }
    check(search.meta(notes, 1, 0, 0, 1) == Ok(2), 13);
    expect(
        &search,
        own_token,
        &[(notes, 2), (draft, 1), (budget, 1)],
        14,
    );
    expect(&search, b"PROJECTS", &[(notes, 2)], 15);

    // B3: a collection rename drops the old label from the index.
    check(search.collection(1, 1, b"Archive") == Ok(2), 16);
    expect(&search, b"archive", &[(notes, 2)], 17);
    if query(&search, b"projects", 18).is_empty() {
        negatives |= 1 << 3;
    }

    // m9: RENAME and DELETE update the index with the returned revision.
    let final_name: &[u8] = if domain == 0 {
        b"alpha final"
    } else {
        b"beta final"
    };
    check(search.rename(draft, 1, final_name) == Ok(2), 19);
    expect(&search, b"final", &[(draft, 2)], 20);
    if query(&search, b"draft", 21).is_empty() {
        negatives |= 1 << 4;
    }
    check(search.delete(budget, 1) == Ok(2), 22);
    // N11: the deleted object is not found.
    if query(&search, b"budget", 23).is_empty() {
        negatives |= 1 << 5;
    }
    // N11: a stale expected revision is refused by the service; the index
    // stays usable and unchanged.
    if search.rename(draft, 1, b"stale") == Err(Error::Volume(Status::Stale))
        && same(&query(&search, b"final", 24), &[(draft, 2)])
    {
        negatives |= 1 << 6;
    }
    // N9: the other domain's token gives no hit.
    if query(&search, other_token, 25).is_empty() {
        negatives |= 1 << 7;
    }
    // N12: with B = 64 a query fails on bytes before tokens (TokensFull is
    // unreachable); an empty query and invalid UTF-8 are explicit errors.
    if search.query(&[b'a'; 65]) == Err(Error::Index(IndexError::TextFull)) {
        negatives |= 1 << 8;
    }
    if search.query(b" ") == Err(Error::Index(IndexError::EmptyQuery))
        && search.query(&[0xff]) == Err(Error::Index(IndexError::InvalidUtf8))
    {
        negatives |= 1 << 9;
    }
    // A foreign object is refused before any request is sent.
    if search.rename((other * PER_DOMAIN) as u64, 1, b"x") == Err(Error::Foreign) {
        negatives |= 1 << 10;
    }
    // Incremental rename/delete leave the full-snapshot generation unchanged.
    // First refresh that snapshot, then prove another rebuild is stale.
    check(search.rebuild().is_ok(), 27);
    if search.rebuild() == Err(Error::StaleGeneration)
        && same(&query(&search, b"final", 26), &[(draft, 2)])
    {
        negatives |= 1 << 11;
    }

    // Report words, recomputable from the after-image alone (run_search.py
    // QUERIES lists the same set in the same order).
    let own_hits = query(&search, own_token, 30);
    check(same(&own_hits, &[(notes, 2), (draft, 2)]), 31);
    let mut value = DIGEST_START;
    for text in [
        own_token,
        b"notes",
        b"final",
        b"draft",
        b"budget",
        b"archive",
        b"projects",
        other_token,
    ] {
        value = digest(value, &query(&search, text, 32));
    }
    let other_hits = query(&search, other_token, 33);
    support::put(400, own_hits.len() as u64);
    support::put(448, value);
    support::put(480, own_hits.len() as u64);
    support::put(488, other_hits.len() as u64);
    support::put(496, negatives);
    check(negatives == ALL_NEGATIVES, 34);
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    run();
    support::put(40, 1);
    native::exit(0)
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(999)
}

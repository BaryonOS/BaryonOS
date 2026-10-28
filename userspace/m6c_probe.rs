#![no_std]
#![no_main]
#[path = "../services/catalog/src/user_service.rs"]
mod catalog_service;
#[path = "../services/catalog/src/journal.rs"]
mod journal;
#[path = "m6_runtime.rs"]
mod rt;
#[path = "../services/catalog/src/wire.rs"]
mod wire;
use rt::Message;

fn value(object: u64, revision: u64) -> [u8; 128] {
    let mut bytes = [0; 128];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = (0x31 + object * 29 + revision * 7 + index as u64 * 3) as u8;
    }
    bytes
}
fn name(object: u64, revision: u64) -> &'static [u8] {
    let old: [&[u8]; 4] = [
        "Café A".as_bytes(),
        b"Werk A",
        "Café B".as_bytes(),
        b"Werk B",
    ];
    let new: [&[u8]; 4] = [
        "Café hernoemd A".as_bytes(),
        b"Werk hernoemd A",
        "Café hernoemd B".as_bytes(),
        b"Werk hernoemd B",
    ];
    if revision < 2 {
        old[object as usize]
    } else {
        new[object as usize]
    }
}
fn body(message: &Message) -> [u8; wire::SIZE] {
    let mut bytes = [0; wire::SIZE];
    bytes.copy_from_slice(&message.bytes[64..64 + wire::SIZE]);
    bytes
}
fn request(
    id: u64,
    object: u64,
    revision: u64,
    operation: u16,
    name: Option<&[u8]>,
    bytes: Option<&[u8]>,
) -> Message {
    let mut message = Message::new();
    message.payload(
        wire::TAG,
        &wire::request(id, rt::word(32), object, revision, operation, name, bytes),
    );
    message
}
fn rpc(message: &Message) -> [u8; wire::SIZE] {
    rt::send(rt::word(64), message);
    for _ in 0..rt::LIMIT {
        if let Some(mut response) = rt::receive(rt::word(56)) {
            let caps = response.close_caps();
            if caps
                || response.sender() != rt::word(176)
                || response.tag() != wire::TAG
                || response.length() != wire::SIZE
            {
                rt::fail(30);
            }
            let bytes = body(&response);
            wire::validate_reply(&bytes, wire::header(&body(message)), rt::word(32))
                .unwrap_or_else(|_| rt::fail(32));
            return bytes;
        }
        rt::yield_now();
    }
    rt::fail(31)
}
fn transact(message: &Message, expected: wire::Status) -> wire::Header {
    let bytes = rpc(message);
    let header = wire::header(&bytes);
    if header.status != expected as i32 {
        rt::fail(33);
    }
    header
}
fn get(id: u64, object: u64) -> (u64, bool) {
    let message = request(id, object, 0, wire::GET, None, None);
    let bytes = rpc(&message);
    let h = wire::header(&bytes);
    match h.status {
        0 => {
            let data_revision = if h.revision == 2 { 1 } else { h.revision };
            if h.length != 128
                || bytes[64..192] != value(object, data_revision)
                || &bytes[192..192 + h.name_length as usize] != name(object, h.revision)
            {
                rt::fail(35);
            }
            (h.revision, true)
        }
        -19 => (h.revision, false),
        _ => rt::fail(36),
    }
}
fn check_get(id: u64, object: u64, revision: u64, present: bool) {
    if get(id, object) != (revision, present) {
        rt::fail(37);
    }
}
fn list(id: u64, domain: u64) -> wire::Catalog {
    let message = request(id, domain * 2, 0, wire::LIST, None, None);
    let bytes = rpc(&message);
    wire::decode_catalog(&bytes, wire::header(&body(&message)), rt::word(32))
        .unwrap_or_else(|_| rt::fail(43))
}
fn check_list(id: u64, domain: u64, revisions: [u64; 2], present: [bool; 2]) {
    let catalog = list(id, domain);
    if catalog.generation != revisions[0] + revisions[1]
        || catalog.count != usize::from(present[0]) + usize::from(present[1])
    {
        rt::fail(44);
    }
    let mut index = 0;
    for slot in 0..2 {
        if present[slot] {
            let entry = catalog.entries[index];
            let object = domain * 2 + slot as u64;
            if entry.object != object
                || entry.revision != revisions[slot]
                || entry.length != 128
                || &entry.name[..entry.name_length] != name(object, revisions[slot])
            {
                rt::fail(45);
            }
            index += 1;
        }
    }
}
fn put(id: u64, object: u64, old: u64) {
    let next = value(object, old + 1);
    let message = request(
        id,
        object,
        old,
        wire::PUT,
        Some(name(object, old + 1)),
        Some(&next),
    );
    let h = transact(&message, wire::Status::Ok);
    if h.revision != old + 1 {
        rt::fail(38);
    }
}
fn rename(id: u64, object: u64, old: u64, expected: wire::Status) {
    let message = request(id, object, old, wire::RENAME, Some(name(object, 2)), None);
    let h = transact(&message, expected);
    if expected == wire::Status::Ok && h.revision != old + 1 {
        rt::fail(46);
    }
}
fn authority_probes(object: u64) {
    let send = rt::word(64);
    let receive = rt::word(56);
    rt::expect(rt::call(0x40, [send, 0, 0, 128, 0, 0]), -6);
    rt::expect(rt::call(0x42, [send, 0, rt::SECTOR, 512, 0, 0]), -6);
    rt::expect(rt::call(0x43, [send, 0, 0, 0, 0, 0]), -6);
    rt::expect(rt::call(0x40, [0, 0, 0, 128, 0, 0]), -4);
    rt::expect(rt::call(0x21, [send, rt::OUT, 320, 0, 0, 0]), -5);
    rt::expect(rt::call(0x11, [send, 1, 0, 0, 0, 0]), -5);
    let message = request(900, object, 0, wire::GET, None, None);
    rt::expect(rt::send_once(receive, &message), -5);
}
fn negative_requests(domain: u64) {
    let object = domain * 2;
    let other = (1 - domain) * 2;
    for fault in 0..32 {
        let id = 100 + fault;
        let mut bytes = wire::request(id, rt::word(32), object, 0, wire::GET, None, None);
        let expected = match fault {
            0 => {
                bytes[0] = 3;
                wire::Status::Invalid
            }
            1 => {
                bytes[2] = 9;
                wire::Status::NoSys
            }
            2 => {
                wire::put64(&mut bytes, 8, 0);
                wire::Status::Invalid
            }
            3 => {
                wire::put64(&mut bytes, 16, 0);
                wire::Status::Stale
            }
            4 => {
                wire::put64(&mut bytes, 24, other);
                wire::Status::Access
            }
            5 => {
                bytes[56] = 1;
                wire::Status::Invalid
            }
            6 => {
                wire::put32(&mut bytes, 40, 129);
                wire::Status::Range
            }
            7 => {
                bytes[64] = 1;
                wire::Status::Invalid
            }
            8 => {
                bytes = wire::request(
                    id,
                    rt::word(32),
                    object,
                    1,
                    wire::PUT,
                    Some(name(object, 1)),
                    Some(&value(object, 2)),
                );
                wire::Status::Stale
            }
            9 => {
                bytes = wire::request(id, rt::word(32), object, 0, wire::DELETE, None, None);
                wire::Status::NotFound
            }
            12 => {
                bytes = wire::request(
                    id,
                    rt::word(32),
                    other,
                    0,
                    wire::PUT,
                    Some(name(other, 1)),
                    Some(&value(other, 1)),
                );
                wire::Status::Access
            }
            13 => {
                bytes = wire::request(id, rt::word(32), other, 0, wire::DELETE, None, None);
                wire::Status::Access
            }
            14 => {
                bytes = wire::request(
                    id,
                    rt::word(32),
                    other,
                    0,
                    wire::RENAME,
                    Some(name(other, 2)),
                    None,
                );
                wire::Status::Access
            }
            15 => {
                bytes = wire::request(id, rt::word(32), other, 0, wire::LIST, None, None);
                wire::Status::Access
            }
            16 => {
                bytes = wire::request(id, rt::word(32), object, 0, wire::PUT, Some(&[0xff]), None);
                wire::Status::Invalid
            }
            17 => {
                bytes = wire::request(
                    id,
                    rt::word(32),
                    object,
                    0,
                    wire::PUT,
                    Some(&[b'x'; 33]),
                    None,
                );
                wire::Status::Range
            }
            18 => {
                bytes[223] = 1;
                wire::Status::Invalid
            }
            19 => {
                bytes = wire::request(id, rt::word(32), object + 1, 0, wire::LIST, None, None);
                wire::Status::Invalid
            }
            20 => {
                bytes = wire::request(id, rt::word(32), object, 1, wire::LIST, None, None);
                wire::Status::Invalid
            }
            21 => {
                bytes = wire::request(id, rt::word(32), object, 0, wire::LIST, None, None);
                wire::put32(&mut bytes, 52, 1);
                wire::Status::Invalid
            }
            22 => {
                bytes = wire::request(
                    id,
                    rt::word(32),
                    object,
                    0,
                    wire::RENAME,
                    Some(name(object, 2)),
                    None,
                );
                wire::Status::NotFound
            }
            23 => {
                bytes = wire::request(
                    id,
                    rt::word(32),
                    object,
                    0,
                    wire::PUT,
                    Some(b"bad/name"),
                    None,
                );
                wire::Status::Invalid
            }
            24 => {
                bytes = wire::request(
                    id,
                    rt::word(32),
                    object,
                    0,
                    wire::PUT,
                    Some("bad\u{202e}name".as_bytes()),
                    None,
                );
                wire::Status::Invalid
            }
            25 => {
                bytes = wire::request(
                    id,
                    rt::word(32),
                    object,
                    0,
                    wire::RENAME,
                    Some(name(object, 2)),
                    Some(b"x"),
                );
                wire::Status::Invalid
            }
            26 => {
                bytes = wire::request(
                    id,
                    rt::word(32),
                    object,
                    0,
                    wire::GET,
                    Some(name(object, 1)),
                    None,
                );
                wire::Status::Invalid
            }
            27 => {
                bytes = wire::request(id, rt::word(32), object, 0, wire::PUT, None, Some(b"x"));
                wire::Status::Invalid
            }
            28 => {
                bytes = wire::request(id, rt::word(32), object, 0, wire::PUT, Some(&[0xc3]), None);
                wire::Status::Invalid
            }
            29 => {
                wire::put32(&mut bytes, 44, (-5i32) as u32);
                wire::Status::Invalid
            }
            30 => {
                bytes[4] = 1;
                wire::Status::Invalid
            }
            31 => {
                bytes = wire::request(id, rt::word(32), object, 0, wire::LIST, None, Some(b"x"));
                wire::Status::Invalid
            }
            _ => wire::Status::Invalid,
        };
        let mut message = Message::new();
        message.payload(wire::TAG, &bytes);
        if fault == 10 {
            wire::put32(&mut message.bytes, 16, 223);
        }
        if fault == 11 {
            wire::put32(&mut message.bytes, 16, 225);
        }
        transact(&message, expected);
    }
}
// The fixture barrier ensures both foreign documents actually exist. These
// denials therefore exercise names/content that must remain undisclosed, not
// merely empty slots whose zero payload would mask a metadata disclosure.
fn cross_domain_populated(domain: u64) {
    let other = (1 - domain) * 2;
    for (index, operation) in [wire::GET, wire::PUT, wire::DELETE, wire::RENAME, wire::LIST]
        .into_iter()
        .enumerate()
    {
        let revision = if matches!(operation, wire::PUT | wire::DELETE | wire::RENAME) {
            1
        } else {
            0
        };
        let new_value = value(other, 2);
        let label = if matches!(operation, wire::PUT | wire::RENAME) {
            Some(name(other, 2))
        } else {
            None
        };
        let payload = if operation == wire::PUT {
            Some(&new_value[..])
        } else {
            None
        };
        let message = request(
            300 + index as u64,
            other,
            revision,
            operation,
            label,
            payload,
        );
        transact(&message, wire::Status::Access);
    }
}
fn uncertain_requests(domain: u64) {
    let object = domain * 2;
    for (index, operation) in [wire::GET, wire::LIST, wire::PUT, wire::DELETE, wire::RENAME]
        .into_iter()
        .enumerate()
    {
        let revision = if matches!(operation, wire::PUT | wire::DELETE | wire::RENAME) {
            1
        } else {
            0
        };
        let new_value = value(object, 2);
        let label = if matches!(operation, wire::PUT | wire::RENAME) {
            Some(name(object, 2))
        } else {
            None
        };
        let payload = if operation == wire::PUT {
            Some(&new_value[..])
        } else {
            None
        };
        let message = request(
            400 + index as u64,
            object,
            revision,
            operation,
            label,
            payload,
        );
        // validate_reply in rpc verifies the empty canonical error payload,
        // zero revision and lack of names/catalog entries, as well as identity.
        transact(&message, wire::Status::Unknown);
    }
    // Authorization and structural validation still precede the closed view.
    // In particular the other domain never receives Unknown as an existence
    // oracle or sees its peer's prior acknowledged name or content.
    cross_domain_populated(domain);
    let mut bytes = wire::request(410, rt::word(32), object, 0, wire::GET, None, None);
    bytes[4] = 1;
    let mut malformed = Message::new();
    malformed.payload(wire::TAG, &bytes);
    transact(&malformed, wire::Status::Invalid);
}
fn wait(offset: u64) {
    for _ in 0..rt::LIMIT {
        if rt::word(offset) == 1 {
            return;
        }
        rt::yield_now();
    }
    rt::fail(39)
}
fn finish(object: u64) -> ! {
    let old = rt::word(64);
    rt::expect(rt::call(0x10, [old, 0, 0, 0, 0, 0]), 0);
    let message = request(999, object, 0, wire::GET, None, None);
    rt::expect(rt::send_once(old, &message), -4);
    rt::put(40, 1);
    rt::worker()
}
fn client() -> ! {
    let case = rt::word(112);
    let domain = rt::word(0) - 1;
    let object = domain * 2;
    if matches!(case, 4 | 7) || (matches!(case, 5 | 6) && domain == 1) {
        rt::worker();
    }
    wait(256);
    authority_probes(object);
    if (8..=15).contains(&case) {
        if domain == 0 {
            put(10, object, 0);
            rename(11, object, 1, wire::Status::Unknown);
        } else {
            wait(280);
        }
        uncertain_requests(domain);
        finish(object);
    }
    if case == 1 {
        let first = get(1, object);
        let second = get(2, object + 1);
        check_list(3, domain, [first.0, second.0], [first.1, second.1]);
        finish(object);
    }
    if matches!(case, 5 | 6) {
        put(10, object, 0);
        rename(11, object, 1, wire::Status::Ok);
        rt::fail(47);
    }
    if case == 2 {
        negative_requests(domain);
    }
    check_list(1, domain, [0, 0], [false, false]);
    check_get(2, object, 0, false);
    if case == 3 {
        for revision in 0..16 {
            put(1000 + domain * 100 + revision, object, revision);
        }
        // Test-only barrier after all 32 genuine ACK receives; no new rights.
        wait(264);
        let next = value(object, 17);
        let message = request(
            2000 + domain,
            object,
            16,
            wire::PUT,
            Some(name(object, 17)),
            Some(&next),
        );
        transact(&message, wire::Status::Full);
        let rename_full = request(
            2100,
            object,
            16,
            wire::RENAME,
            Some(b"Mag niet overschrijven"),
            None,
        );
        transact(&rename_full, wire::Status::Full);
        check_get(2101, object, 16, true);
        check_list(2102, domain, [16, 0], [true, false]);
        check_get(2103, object + 1, 0, false);
    } else {
        put(3, object, 0);
        put(4, object + 1, 0);
        if case == 2 {
            wait(272);
            cross_domain_populated(domain);
        }
        check_list(5, domain, [1, 1], [true, true]);
        check_get(6, object, 1, true);
        rename(7, object, 1, wire::Status::Ok);
        rename(8, object, 1, wire::Status::Stale);
        check_get(9, object, 2, true);
        check_list(10, domain, [2, 1], [true, true]);
        let message = request(11, object + 1, 1, wire::DELETE, None, None);
        if transact(&message, wire::Status::Ok).revision != 2 {
            rt::fail(40);
        }
        check_list(12, domain, [2, 2], [true, false]);
        check_get(13, object + 1, 2, false);
        put(14, object, 2);
        check_get(15, object, 3, true);
        check_list(16, domain, [3, 2], [true, false]);
    }
    finish(object)
}
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    if rt::word(24) != 0 || rt::word(16) == 0 || rt::call(0, [0; 6]) != (0, (1 << 32) | 2) {
        rt::fail(41);
    }
    match rt::word(0) {
        0 => catalog_service::run(),
        1 | 2 => client(),
        _ => rt::fail(42),
    }
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    rt::fail(0xff)
}

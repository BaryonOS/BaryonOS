#![no_std]
#![no_main]
#[path = "../services/objects/src/journal.rs"]
mod journal;
#[path = "../services/objects/src/user_service.rs"]
mod object_service;
#[path = "m6_runtime.rs"]
mod rt;
#[path = "../services/objects/src/wire.rs"]
mod wire;
use rt::Message;

fn value(object: u64, revision: u64) -> [u8; 128] {
    let mut bytes = [0; 128];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = (0x31 + object * 29 + revision * 7 + index as u64 * 3) as u8;
    }
    bytes
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
    bytes: Option<&[u8; 128]>,
) -> Message {
    let mut message = Message::new();
    message.payload(
        wire::TAG,
        &wire::request(
            id,
            rt::word(32),
            object,
            revision,
            operation,
            bytes.map(|bytes| &bytes[..]),
        ),
    );
    message
}
fn rpc(message: &Message) -> Message {
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
            return response;
        }
        rt::yield_now();
    }
    rt::fail(31)
}
fn transact(message: &Message, expected: wire::Status) -> (wire::Header, [u8; 128]) {
    let response = rpc(message);
    let bytes = body(&response);
    let header = wire::validate_reply(&bytes, wire::header(&body(message)), rt::word(32))
        .unwrap_or_else(|_| rt::fail(32));
    if header.status != expected as i32 {
        rt::fail(33);
    }
    let mut result = [0; 128];
    result.copy_from_slice(&bytes[64..192]);
    (header, result)
}
fn get(id: u64, object: u64) -> (u64, bool) {
    let message = request(id, object, 0, wire::GET, None);
    let response = rpc(&message);
    let bytes = body(&response);
    let header = wire::validate_reply(&bytes, wire::header(&body(&message)), rt::word(32))
        .unwrap_or_else(|_| rt::fail(34));
    match header.status {
        0 => {
            if header.length != 128 || bytes[64..192] != value(object, header.revision) {
                rt::fail(35);
            }
            (header.revision, true)
        }
        -19 => (header.revision, false),
        _ => rt::fail(36),
    }
}
fn check_get(id: u64, object: u64, revision: u64, present: bool) {
    if get(id, object) != (revision, present) {
        rt::fail(37);
    }
}
fn put(id: u64, object: u64, old: u64) {
    let next = value(object, old + 1);
    let message = request(id, object, old, wire::PUT, Some(&next));
    let (header, _) = transact(&message, wire::Status::Ok);
    if header.revision != old + 1 {
        rt::fail(38);
    }
}
fn authority_probes(object: u64) {
    let send = rt::word(64);
    let receive = rt::word(56);
    // Real syscalls must reject endpoint handles where a typed block capability
    // is required. Neither application receives any DMA/MMIO/block authority.
    rt::expect(rt::call(0x40, [send, 0, 0, 128, 0, 0]), -6);
    rt::expect(rt::call(0x42, [send, 0, rt::SECTOR, 512, 0, 0]), -6);
    rt::expect(rt::call(0x43, [send, 0, 0, 0, 0, 0]), -6);
    rt::expect(rt::call(0x40, [0, 0, 0, 128, 0, 0]), -4);
    rt::expect(rt::call(0x21, [send, rt::OUT, 320, 0, 0, 0]), -5);
    rt::expect(rt::call(0x11, [send, 1, 0, 0, 0, 0]), -5);
    let message = request(900, object, 0, wire::GET, None);
    rt::expect(rt::send_once(receive, &message), -5);
}
fn negative_requests(domain: u64) {
    let object = domain * 2;
    for fault in 0..14 {
        let mut bytes = wire::request(100 + fault, rt::word(32), object, 0, wire::GET, None);
        let expected = match fault {
            0 => {
                bytes[0] = 0;
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
                wire::put64(&mut bytes, 24, (1 - domain) * 2);
                wire::Status::Access
            }
            5 => {
                bytes[48] = 1;
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
                    108,
                    rt::word(32),
                    object,
                    1,
                    wire::PUT,
                    Some(&value(object, 2)),
                );
                wire::Status::Stale
            }
            9 => {
                bytes = wire::request(109, rt::word(32), object, 0, wire::DELETE, None);
                wire::Status::NotFound
            }
            12 => {
                let other = (1 - domain) * 2;
                bytes = wire::request(
                    112,
                    rt::word(32),
                    other,
                    0,
                    wire::PUT,
                    Some(&value(other, 1)),
                );
                wire::Status::Access
            }
            13 => {
                bytes = wire::request(113, rt::word(32), (1 - domain) * 2, 0, wire::DELETE, None);
                wire::Status::Access
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
        let _ = transact(&message, expected);
    }
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
    // A formerly valid own SEND handle is closed; the same integer must no
    // longer reach the service, even while its sibling reply endpoint lives.
    let old = rt::word(64);
    rt::expect(rt::call(0x10, [old, 0, 0, 0, 0, 0]), 0);
    let message = request(999, object, 0, wire::GET, None);
    rt::expect(rt::send_once(old, &message), -4);
    rt::put(40, 1);
    rt::worker()
}
fn uncertain_client(domain: u64, object: u64) -> ! {
    if domain == 0 {
        // Only this first PUT may reach the device. The peer waits until the
        // observer has seen its real Unknown reply delivered over IPC.
        let next = value(object, 1);
        let message = request(10, object, 0, wire::PUT, Some(&next));
        let _ = transact(&message, wire::Status::Unknown);
    }
    wait(264);
    let next = value(object, 1);
    for (id, operation, bytes) in [
        (20, wire::GET, None),
        (21, wire::PUT, Some(&next)),
        (22, wire::DELETE, None),
        // Identical to the first domain-0 PUT: no automatic or explicit replay
        // may publish another write in an uncertain service instance.
        (10, wire::PUT, Some(&next)),
    ] {
        let message = request(id, object, 0, operation, bytes);
        let (header, returned) = transact(&message, wire::Status::Unknown);
        if header.revision != 0 || header.length != 0 || returned != [0; 128] {
            rt::fail(43);
        }
    }
    // Validation still precedes the uncertain-state guard. Domain denial must
    // not expose an object, revision, payload or perform any block syscall.
    let message = request(23, (1 - domain) * 2, 0, wire::GET, None);
    let _ = transact(&message, wire::Status::Access);
    finish(object)
}
fn client() -> ! {
    let case = rt::word(112);
    let domain = rt::word(0) - 1;
    let object = domain * 2;
    if case == 4 {
        rt::worker();
    }
    wait(256);
    authority_probes(object);
    if matches!(case, 7 | 8) {
        uncertain_client(domain, object);
    }
    if case == 1 {
        let _ = get(1, object);
        let _ = get(2, object + 1);
        finish(object);
    }
    check_get(1, object, 0, false);
    check_get(2, object + 1, 0, false);
    if case == 2 {
        negative_requests(domain);
    }
    if case == 3 {
        for revision in 0..16 {
            put(1000 + domain * 100 + revision, object, revision);
        }
        // Kernel test barrier follows 32 real mutation ACK receives. It does
        // not grant authority and cannot cause an I/O completion or service ACK.
        wait(264);
        let next = value(object, 17);
        let message = request(2000 + domain, object, 16, wire::PUT, Some(&next));
        let _ = transact(&message, wire::Status::Full);
        check_get(2100, object, 16, true);
        check_get(2101, object + 1, 0, false);
    } else {
        put(10, object, 0);
        put(11, object + 1, 0);
        put(12, object, 1);
        let message = request(13, object + 1, 1, wire::DELETE, None);
        let (header, _) = transact(&message, wire::Status::Ok);
        if header.revision != 2 {
            rt::fail(40);
        }
        check_get(20, object, 2, true);
        check_get(21, object + 1, 2, false);
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
        0 => object_service::run(),
        1 | 2 => client(),
        _ => rt::fail(42),
    }
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    rt::fail(0xff)
}

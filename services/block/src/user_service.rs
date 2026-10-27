//! Actual ring3 service/front-end. Device bytes come exclusively from block_poll;
//! no disk pattern, seed or expected response is present in this program.
use crate::{rt, wire};
use rt::Message;

fn kernel_slice(ticket: u64, length: usize, returned: u64) -> Result<[u8; 128], wire::Status> {
    let m = rt::output();
    if returned != length as u64 || m.sender() != 0 || m.tag() != ticket || m.length() != length
        || m.cap_count() != 0 || wire::get64(&m.bytes, 24) != 0
        || m.bytes[32..64].iter().any(|b| *b != 0)
        || m.bytes[64 + length..].iter().any(|b| *b != 0) {
        return Err(wire::Status::DeviceProtocol);
    }
    let mut bytes = [0; 128];
    bytes[..length].copy_from_slice(&m.bytes[64..64 + length]);
    Ok(bytes)
}

fn device_read(sector: u64, offset: u64, length: u32) -> Result<[u8; 128], wire::Status> {
    let cap = rt::word(48);
    let first = rt::word(200);
    let relative = sector.checked_sub(first).filter(|s| *s < 4).ok_or(wire::Status::Access)?;
    // Only READ is replayed, once, after the kernel's bounded reset/fault path.
    // Reset-timeout/offline cases deliberately have no successful-data claim.
    let attempts = if matches!(rt::word(112), 12 | 17) { 1 } else { 2 };
    let mut last = wire::Status::Io;
    for _ in 0..attempts {
        let mut submission = None;
        for _ in 0..rt::LIMIT {
            let (code, ticket) = rt::call(0x40, [cap, relative, offset, u64::from(length), 0, 0]);
            if code == 0 { submission = Some(ticket); break; }
            if ticket != 0 { rt::fail(20); }
            if code == -8 { rt::yield_now(); continue; }
            last = wire::Status::from_code(code); break;
        }
        let Some(ticket) = submission else { continue; };
        // Explicit test-only pending-read crash. The kernel owns quiescence and
        // restart; this is never selected by an app request/opcode.
        if rt::word(184) == 2 { rt::crash(); }
        for _ in 0..rt::LIMIT {
            let (code, value) = rt::call(0x41, [cap, ticket, rt::OUT, 320, 0, 0]);
            if code == 0 { return kernel_slice(ticket, length as usize, value); }
            if value != 0 { rt::fail(21); }
            if code == -8 { rt::yield_now(); continue; }
            last = wire::Status::from_code(code); break;
        }
    }
    Err(last)
}

fn device_preflight() {
    let cap = rt::word(48);
    let request_root = rt::word(56);
    rt::expect(rt::call(0x40, [request_root, 0, 0, 128, 0, 0]), -6);
    rt::expect(rt::call(0x40, [0, 0, 0, 128, 0, 0]), -4);
    rt::expect(rt::call(0x40, [cap, 4, 0, 128, 0, 0]), -5);
    for length in [0, 129] { rt::expect(rt::call(0x40, [cap, 0, 0, length, 0, 0]), -11); }
    for offset in [512, u64::MAX] { rt::expect(rt::call(0x40, [cap, 0, offset, 1, 0, 0]), -1); }
    rt::expect(rt::call(0x11, [cap, 1, 0, 0, 0, 0]), -5);
    let (code, ticket) = rt::call(0x40, [cap, 0, 0, 128, 0, 0]);
    if code != 0 { rt::fail(22); }
    rt::expect(rt::call(0x41, [cap, ticket, rt::OUT, 319, 0, 0]), -11);
    rt::expect(rt::call(0x41, [cap, ticket ^ (1 << 63), rt::OUT, 320, 0, 0]), -1);
    rt::expect(rt::call(0x41, [cap, ticket, 0, 320, 0, 0]), -3);
    rt::expect(rt::call(0x41, [cap, ticket, 0x40013000, 320, 0, 0]), -3);
    for _ in 0..rt::LIMIT {
        let (code, value) = rt::call(0x41, [cap, ticket, rt::OUT, 320, 0, 0]);
        if code == 0 {
            if kernel_slice(ticket, 128, value).is_err() { rt::fail(23); }
            rt::expect(rt::call(0x41, [cap, ticket, rt::OUT, 320, 0, 0]), -1);
            return;
        }
        if (code, value) != (-8, 0) { rt::fail(24); }
        rt::yield_now();
    }
    rt::fail(25)
}

fn announce(epoch: u64) {
    let mut handles = [0; 2];
    for (i, root) in [rt::word(56), rt::word(80)].into_iter().enumerate() {
        let (code, handle) = rt::call(0x11, [root, 9, 0, 0, 0, 0]);
        if code != 0 { rt::fail(26); }
        handles[i] = handle;
    }
    let mut payload = [0; 32];
    wire::put64(&mut payload, 0, 1); wire::put64(&mut payload, 8, epoch);
    // PID is supplied by the kernel Message.sender_pid, not this announcement.
    wire::put64(&mut payload, 24, 2);
    let mut notice = Message::new(); notice.payload(wire::ANNOUNCE, &payload);
    notice.add_caps(&handles, 9);
    rt::send(rt::word(72), &notice);
    for handle in handles { rt::expect(rt::call(0x10, [handle, 0, 0, 0, 0, 0]), 0); }
}

pub fn run() -> ! {
    let epoch = rt::word(32);
    if epoch == 0 || rt::word(48) == 0 { rt::fail(27); }
    announce(epoch);
    if rt::word(112) == 2 { device_preflight(); }
    let roots = [rt::word(56), rt::word(80)];
    loop {
        if rt::word(184) == 1 { rt::crash(); }
        for (scope, endpoint) in roots.iter().enumerate() {
            let Some(mut message) = rt::receive(*endpoint) else { continue; };
            let unexpected_caps = message.close_caps();
            // Reply endpoint is fixed to the bound client; do not send unsolicited
            // answers to that client for a different, unauthorized sender.
            if message.sender() != rt::word(160) { continue; }
            let mut payload = [0; wire::SIZE];
            payload.copy_from_slice(&message.bytes[64..64 + wire::SIZE]);
            let header = wire::header(&payload);
            let request = if unexpected_caps || message.tag() != wire::TAG || message.length() != wire::SIZE {
                Err(wire::Status::Invalid)
            } else { wire::validate_request(&payload, epoch) };
            let (status, bytes, length) = match request {
                Err(error) => (error, [0; 128], 0),
                Ok(valid) => match device_read(rt::word(96 + scope as u64 * 8), valid.offset, valid.length) {
                    Ok(data) => (wire::Status::Ok, data, valid.length as usize),
                    Err(error) => (error, [0; 128], 0),
                },
            };
            let response = match wire::reply(header, epoch, status, &bytes[..length]) {
                Ok(bytes) => bytes,
                Err(_) => rt::fail(28),
            };
            let mut answer = Message::new(); answer.payload(wire::TAG, &response);
            rt::send(rt::word(88), &answer);
        }
        rt::yield_now();
    }
}

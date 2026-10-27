#![no_std]
#![no_main]

#[path = "../services/block/src/wire.rs"]
mod wire;
#[path = "m5_runtime.rs"]
mod rt;
#[path = "../services/block/src/user_service.rs"]
mod block_service;

use rt::Message;

struct Connection { caps: [u64; 2], epoch: u64, service_pid: u64, next_id: u64 }
impl Connection {
    fn new() -> Self { Self { caps: [0; 2], epoch: 0, service_pid: 0, next_id: 1 } }
    fn install(&mut self, mut notice: Message) {
        let p = &notice.bytes[64..96];
        let epoch = wire::get64(p, 8); let pid = wire::get64(p, 16);
        let valid = notice.sender() == rt::word(168) && notice.tag() == wire::REBIND
            && notice.length() == 32 && notice.cap_count() == 2
            && wire::get64(p, 0) == 1 && wire::get64(p, 24) == 2
            && epoch == self.epoch + 1 && pid > self.service_pid
            && (self.epoch != 0 || pid == rt::word(176))
            && notice.rights(0) == 1 && notice.rights(1) == 1;
        if !valid { notice.close_caps(); rt::fail(30); }
        let fresh = [notice.cap(0), notice.cap(1)];
        if self.epoch != 0 {
            let payload = wire::request(0, self.epoch, 0, 1, wire::READ);
            let mut stale = Message::new(); stale.payload(wire::TAG, &payload);
            rt::expect(rt::send_once(self.caps[0], &stale), -10);
            for handle in self.caps { rt::expect(rt::call(0x10, [handle, 0, 0, 0, 0, 0]), 0); }
        }
        self.caps = fresh; self.epoch = epoch; self.service_pid = pid;
        rt::put(136, epoch);
    }
    fn try_rebind(&mut self) -> bool {
        if let Some(notice) = rt::receive(rt::word(80)) { self.install(notice); true } else { false }
    }
    fn wait_rebind(&mut self) {
        for _ in 0..rt::LIMIT { if self.try_rebind() { return; } rt::yield_now(); }
        rt::fail(31)
    }
}

enum Rpc { Reply(Message), Rebound }
fn rpc(conn: &mut Connection, scope: usize, message: &Message) -> Rpc {
    for _ in 0..rt::LIMIT {
        let sent = rt::send_once(conn.caps[scope], message);
        match sent {
            (0, 0) => break,
            (-8, 0) => { if conn.try_rebind() { return Rpc::Rebound; } rt::yield_now(); },
            (-10, 0) if rt::word(112) == 13 => { conn.wait_rebind(); return Rpc::Rebound; },
            _ => rt::fail(32),
        }
    }
    rt::put(144, rt::word(144) + 1);
    for _ in 0..rt::LIMIT {
        if conn.try_rebind() { return Rpc::Rebound; }
        if let Some(mut response) = rt::receive(rt::word(56)) {
            let caps = response.close_caps();
            if caps || response.sender() != conn.service_pid || response.tag() != wire::TAG
                || response.length() != wire::SIZE { rt::fail(33); }
            return Rpc::Reply(response);
        }
        rt::yield_now();
    }
    rt::fail(34)
}
fn answer_payload(message: &Message) -> [u8; wire::SIZE] {
    let mut bytes = [0; wire::SIZE]; bytes.copy_from_slice(&message.bytes[64..64 + wire::SIZE]); bytes
}
fn positive_reads(conn: &mut Connection) {
    'again: loop {
        let mut sectors = [[0u8; 512]; 2];
        for (scope, sector) in sectors.iter_mut().enumerate() {
            for part in 0..4 {
                let id = conn.next_id; conn.next_id += 1;
                let payload = wire::request(id, conn.epoch, (part * 128) as u64, 128, wire::READ);
                let request = wire::header(&payload);
                let mut message = Message::new(); message.payload(wire::TAG, &payload);
                let reply = match rpc(conn, scope, &message) { Rpc::Rebound => continue 'again, Rpc::Reply(m) => m };
                let bytes = answer_payload(&reply);
                let h = match wire::validate_reply(&bytes, request, conn.epoch) { Ok(h) => h, Err(_) => rt::fail(35) };
                if h.status != 0 { rt::fail(36); }
                sector[part * 128..part * 128 + 128].copy_from_slice(&bytes[48..176]);
            }
        }
        rt::put(120, wire::checksum_update(wire::FNV_OFFSET, &sectors[0]));
        rt::put(128, wire::checksum_update(wire::FNV_OFFSET, &sectors[1]));
        rt::put(192, if conn.epoch == 1 { 1 } else { 4 });
        return;
    }
}
fn negative_reply(conn: &mut Connection, mut message: Message, expected: wire::Status) {
    let request = wire::header(&answer_payload(&message));
    let source_caps = if message.cap_count() != 0 { message.cap(0) } else { 0 };
    let reply = match rpc(conn, 0, &message) { Rpc::Reply(m) => m, Rpc::Rebound => rt::fail(37) };
    if source_caps != 0 { rt::expect(rt::call(0x10, [source_caps, 0, 0, 0, 0, 0]), 0); }
    // Dropping a local send-message copy does not close any transferred receiver
    // authority. The service must explicitly close its actual received handles.
    wire::put32(&mut message.bytes, 20, 0);
    let bytes = answer_payload(&reply);
    let h = match wire::validate_reply(&bytes, request, conn.epoch) { Ok(h) => h, Err(_) => rt::fail(38) };
    if h.status != expected as i32 || h.length != 0 || bytes[48..].iter().any(|b| *b != 0) { rt::fail(39); }
    rt::put(152, rt::word(152) + 1);
}
fn service_errors(conn: &mut Connection) {
    // Each malformed request is still a canonical outer Message320.
    for fault in 0..13 {
        let id = conn.next_id; conn.next_id += 1;
        let mut payload = wire::request(id, conn.epoch, 0, 128, wire::READ);
        let expected = match fault {
            0 => { payload[0] = 2; wire::Status::Invalid },
            1 => { payload[2] = wire::WRITE as u8; wire::Status::NoSys },
            2 => { payload[4] = 1; wire::Status::Invalid },
            3 => { payload[40] = 1; wire::Status::Invalid },
            4 => { wire::put32(&mut payload, 32, 0); wire::Status::Range },
            5 => { wire::put32(&mut payload, 32, 129); wire::Status::Range },
            6 => { wire::put64(&mut payload, 24, u64::MAX); wire::Status::Range },
            7 => { wire::put64(&mut payload, 24, 511); wire::put32(&mut payload, 32, 2); wire::Status::Range },
            8 => { wire::put64(&mut payload, 16, conn.epoch - 1); wire::Status::Stale },
            9 => { payload[48] = 1; wire::Status::Invalid },
            _ => wire::Status::Invalid,
        };
        let mut message = Message::new(); message.payload(wire::TAG, &payload);
        if fault == 10 { wire::put32(&mut message.bytes, 16, 191); }
        if fault == 11 { wire::put32(&mut message.bytes, 16, 193); }
        if fault == 12 {
            let (code, handle) = rt::call(0x11, [rt::word(56), 9, 0, 0, 0, 0]);
            if code != 0 { rt::fail(40); }
            message.add_caps(&[handle, handle], 1);
        }
        negative_reply(conn, message, expected);
    }
    rt::put(192, 2);
}
fn client_device_rights() {
    rt::expect(rt::call(0x40, [rt::word(56), 0, 0, 128, 0, 0]), -6);
    rt::expect(rt::call(0x40, [0, 0, 0, 128, 0, 0]), -4);
    rt::expect(rt::call(0x40, [rt::word(232), 0, 0, 128, 0, 0]), -5);
    let (code, result) = rt::call(0x40, [rt::word(224), 0, 0, 128, 0, 0]);
    if !matches!(code, -4 | -5 | -6) || result != 0 { rt::fail(41); }
    rt::expect(rt::call(0x10, [rt::word(232), 0, 0, 0, 0, 0]), 0);
}
fn client() -> ! {
    let mut conn = Connection::new(); conn.wait_rebind();
    let case = rt::word(112);
    if case == 1 { service_errors(&mut conn); }
    if case == 2 { client_device_rights(); }
    if matches!(case, 12 | 17) {
        let payload = wire::request(1, conn.epoch, 0, 128, wire::READ);
        let mut message = Message::new(); message.payload(wire::TAG, &payload);
        negative_reply(&mut conn, message, if case == 12 { wire::Status::Timeout } else { wire::Status::NoDevice });
    } else {
        positive_reads(&mut conn);
        if conn.epoch != if case == 13 { 4 } else { 1 } { rt::fail(42); }
    }
    rt::put(192, 5); rt::put(40, 1); rt::worker()
}
fn supervisor() -> ! {
    let target = if rt::word(112) == 13 { 4 } else { 1 };
    let mut epoch = 0; let mut service_pid = 0;
    loop {
        let Some(mut notice) = rt::receive(rt::word(56)) else { rt::yield_now(); continue; };
        let p = &notice.bytes[64..96];
        let next = wire::get64(p, 8); let sender = notice.sender();
        let valid = notice.tag() == wire::ANNOUNCE && notice.length() == 32 && notice.cap_count() == 2
            && wire::get64(p, 0) == 1 && wire::get64(p, 16) == 0 && wire::get64(p, 24) == 2
            && sender != rt::word(160) && sender != rt::word(168) && sender > service_pid
            && (epoch != 0 || sender == rt::word(176)) && next == epoch + 1
            && notice.rights(0) == 9 && notice.rights(1) == 9;
        if !valid { notice.close_caps(); rt::fail(43); }
        let handles = [notice.cap(0), notice.cap(1)];
        let mut payload = [0; 32]; payload.copy_from_slice(p); wire::put64(&mut payload, 16, sender);
        let mut forward = Message::new(); forward.payload(wire::REBIND, &payload); forward.add_caps(&handles, 1);
        rt::send(rt::word(64), &forward);
        for handle in handles { rt::expect(rt::call(0x10, [handle, 0, 0, 0, 0, 0]), 0); }
        epoch = next; service_pid = sender; rt::put(136, epoch);
        if epoch == target { rt::put(40, 1); }
    }
}
fn attacker() -> ! {
    match rt::word(112) {
        14 | 15 => {
            let target = rt::word(if rt::word(112) == 14 { 208 } else { 216 });
            // SAFETY: Explicit isolated negative test, raw assembly deliberately
            // attempts a forbidden supervisor read. No Rust reference/UB slice.
            unsafe { core::arch::asm!("mov rax, [{target}]", target = in(reg) target, out("rax") _, options(nostack)); }
        }
        16 => {
            // SAFETY: Deliberately forbidden CPL3 PCI-config port access. A #GP
            // must kill only this synthetic attacker, never authorize the I/O.
            unsafe { core::arch::asm!("out dx, eax", in("dx") 0xcf8u16, in("eax") 0u32, options(nomem, nostack)); }
        }
        _ => rt::fail(44),
    }
    rt::fail(45)
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    if rt::word(24) != 0 || rt::word(16) == 0 { rt::fail(46); }
    if rt::call(0, [0; 6]) != (0, (1 << 32) | 1) { rt::fail(47); }
    match rt::word(0) { 0 => block_service::run(), 1 => client(), 2 => supervisor(), 3 => attacker(), _ => rt::fail(48) }
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { rt::fail(0xff) }

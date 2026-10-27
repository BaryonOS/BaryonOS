//! Scalar runtime for the single-thread M5 ELF. Fixed private user buffers are
//! accessed with volatile bytes; kernel IPC never retains their pointers.
use core::arch::{asm, global_asm};

pub const DATA: u64 = 0x40010000;
pub const MSG: u64 = 0x40011000;
pub const OUT: u64 = 0x40012000;
pub const LIMIT: usize = 1_000_000;

global_asm!(r#"
 .section .m5.header,"aw",@progbits
 .balign 8
 .global m5_startup
m5_startup:
 .zero 1024
 .section .m5.buffers,"aw",@progbits
 .balign 4096
 .global m5_message
m5_message:
 .zero 4096
 .global m5_output
m5_output:
 .zero 4096
 .section .rodata.m5,"a",@progbits
 .quad 0
"#);

pub fn word(offset: u64) -> u64 {
    // SAFETY: All callers use aligned startup offsets below1024. This private
    // RW ELF page exists for the entire process, with no Rust reference alias.
    unsafe { ((DATA + offset) as *const u64).read_volatile() }
}
pub fn put(offset: u64, value: u64) {
    // SAFETY: Same fixed private startup page; one user thread, aligned u64.
    unsafe { ((DATA + offset) as *mut u64).write_volatile(value); }
}
pub fn call(number: u64, args: [u64; 6]) -> (i64, u64) {
    put(8, word(8).wrapping_add(1));
    let mut primary = number;
    let mut secondary = args[2];
    // SAFETY: Own int80 ABI preserves all GPRs except RAX/RDX and sanitizes
    // flags. Arguments are integers, including deliberate invalid-pointer
    // tests; this wrapper constructs no borrowed slice from those integers.
    // Omit nomem: the syscall may copy bytes into/out of our private buffers.
    unsafe {
        asm!("int 0x80", inout("rax") primary, in("rdi") args[0], in("rsi") args[1],
            inout("rdx") secondary, in("r10") args[3], in("r8") args[4], in("r9") args[5],
            options(nostack));
    }
    (primary as i64, secondary)
}
pub fn yield_now() {
    if call(1, [0; 6]) != (0, 0) { fail(1); }
}
pub fn fail(code: u64) -> ! {
    put(40, 0x100 + code); put(240, code);
    call(2, [0xe5, 0, 0, 0, 0, 0]);
    loop { core::hint::spin_loop(); }
}
pub fn crash() -> ! {
    // SAFETY: Explicit synthetic userfault in the fixed M5 crash test only.
    unsafe { asm!("ud2", options(noreturn)); }
}
pub fn worker() -> ! { loop { yield_now(); } }

#[derive(Clone, Copy)]
pub struct Message { pub bytes: [u8; 320] }
impl Message {
    pub const fn new() -> Self { Self { bytes: [0; 320] } }
    pub fn sender(&self) -> u64 { crate::wire::get64(&self.bytes, 0) }
    pub fn tag(&self) -> u64 { crate::wire::get64(&self.bytes, 8) }
    pub fn length(&self) -> usize { crate::wire::get32(&self.bytes, 16) as usize }
    pub fn cap_count(&self) -> usize { crate::wire::get32(&self.bytes, 20) as usize }
    pub fn cap(&self, index: usize) -> u64 { crate::wire::get64(&self.bytes, 32 + index * 16) }
    pub fn rights(&self, index: usize) -> u64 { crate::wire::get64(&self.bytes, 40 + index * 16) }
    pub fn payload(&mut self, tag: u64, bytes: &[u8]) {
        if bytes.len() > 256 { fail(2); }
        crate::wire::put64(&mut self.bytes, 8, tag);
        crate::wire::put32(&mut self.bytes, 16, bytes.len() as u32);
        self.bytes[64..64 + bytes.len()].copy_from_slice(bytes);
    }
    pub fn add_caps(&mut self, caps: &[u64], rights: u64) {
        if caps.len() > 2 { fail(3); }
        crate::wire::put32(&mut self.bytes, 20, caps.len() as u32);
        for (i, cap) in caps.iter().enumerate() {
            crate::wire::put64(&mut self.bytes, 32 + i * 16, *cap);
            crate::wire::put64(&mut self.bytes, 40 + i * 16, rights);
        }
    }
    pub fn close_caps(&mut self) -> bool {
        let count = core::cmp::min(self.cap_count(), 2);
        for i in 0..count { let _ = call(0x10, [self.cap(i), 0, 0, 0, 0, 0]); }
        crate::wire::put32(&mut self.bytes, 20, 0);
        count != 0
    }
}
pub fn write_message(message: &Message) {
    for (i, byte) in message.bytes.iter().enumerate() {
        // SAFETY: MSG is a private, mapped RW page; exactly320 in-page bytes.
        unsafe { (MSG as *mut u8).add(i).write_volatile(*byte); }
    }
}
pub fn output() -> Message {
    let mut message = Message::new();
    for (i, byte) in message.bytes.iter_mut().enumerate() {
        // SAFETY: OUT is a private mapped RW page, filled synchronously by the
        // preceding validated kernel copy. No DMA targets this user page.
        *byte = unsafe { (OUT as *const u8).add(i).read_volatile() };
    }
    message
}
pub fn send_once(endpoint: u64, message: &Message) -> (i64, u64) {
    write_message(message); call(0x20, [endpoint, MSG, 320, 0, 0, 0])
}
pub fn send(endpoint: u64, message: &Message) {
    for _ in 0..LIMIT {
        match send_once(endpoint, message) {
            (0, 0) => return,
            (-8, 0) => yield_now(),
            _ => fail(4),
        }
    }
    fail(5)
}
pub fn receive(endpoint: u64) -> Option<Message> {
    match call(0x21, [endpoint, OUT, 320, 0, 0, 0]) {
        (0, length) => { let m = output(); if m.length() as u64 != length { fail(6); } Some(m) }
        (-8, 0) => None,
        _ => fail(7),
    }
}
pub fn expect(result: (i64, u64), error: i64) {
    if result != (error, 0) { fail(8); }
}

// Compiler memory intrinsics. Their C caller contracts provide valid ranges;
// explicit assembly avoids optimizer-created recursive intrinsic calls.
#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    // SAFETY: Caller guarantees readable/writable nonoverlapping ranges of n
    // bytes. rep movsb touches exactly those bytes and needs no extra alignment.
    unsafe { asm!("cld", "rep movsb", inout("rdi") dst => _, inout("rsi") src => _, inout("rcx") n => _, options(nostack)); }
    dst
}
#[no_mangle]
pub unsafe extern "C" fn memset(dst: *mut u8, value: i32, n: usize) -> *mut u8 {
    // SAFETY: Caller guarantees n writable bytes; byte stores touch only them.
    unsafe { asm!("cld", "rep stosb", inout("rdi") dst => _, in("al") value as u8, inout("rcx") n => _, options(nostack)); }
    dst
}
#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if n == 0 { return dst; }
    if dst as usize <= src as usize {
        // SAFETY: Forward byte order preserves overlapping ranges when dst<=src.
        unsafe { asm!("cld", "rep movsb", inout("rdi") dst => _, inout("rsi") src => _, inout("rcx") n => _, options(nostack)); }
    } else {
        // SAFETY: Valid n-byte ranges and n>0 make the final-byte offsets valid;
        // backward order preserves overlap, and DF is restored before return.
        unsafe { asm!("std", "rep movsb", "cld", inout("rdi") dst.add(n - 1) => _, inout("rsi") src.add(n - 1) => _, inout("rcx") n => _, options(nostack)); }
    }
    dst
}
#[no_mangle]
pub unsafe extern "C" fn memcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    for i in 0..n {
        // SAFETY: Caller guarantees readable n-byte ranges; volatile reads avoid
        // the optimizer turning this implementation back into a memcmp call.
        let (x, y) = unsafe { (a.add(i).read_volatile(), b.add(i).read_volatile()) };
        if x != y { return i32::from(x) - i32::from(y); }
    }
    0
}

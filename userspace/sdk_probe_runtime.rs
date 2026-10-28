//! Opt-in SDK bridge for the unchanged M6c service/client acceptance roles.
//! Normal IPC uses SDK-owned stack/borrowed buffers. Raw malformed-ABI probes,
//! block I/O, startup metadata and deliberate test faults retain their exact
//! old runtime path. This is a test adapter, not an app startup specification.
use nieuw_os_sdk_runtime::{native, Error, Handle, Outgoing, ReceiveEndpoint, SendEndpoint};

#[allow(dead_code)]
#[path = "m6_runtime.rs"]
mod fixture;
pub use fixture::{expect, fail, put, word, LIMIT, OUT, SECTOR};

fn before_sdk_call() {
    put(8, word(8).wrapping_add(1));
}

fn after_sdk_call() {
    // Same test-only, kernel-selected service-crash boundary as m6_runtime.
    // Never turn a requested fault into a later successful SDK result.
    if word(0) == 0 && word(184) == 1 {
        fixture::crash();
    }
}

fn registers(result: Result<u64, Error>) -> (i64, u64) {
    after_sdk_call();
    match result {
        Ok(value) => (0, value),
        Err(Error::Kernel(code)) => (code, 0),
        // A runtime representation/return mismatch is a failed probe. It is
        // never relabelled as an expected kernel errno or a successful syscall.
        Err(_) => fail(90),
    }
}

pub fn call(number: u64, args: [u64; 6]) -> (i64, u64) {
    // Only canonical, ordinary operations use typed wrappers here. Unused
    // arguments and invalid numeric handles must still reach the actual kernel
    // when the probe is deliberately checking its ABI rejection behavior.
    if number == 0 && args == [0; 6] {
        before_sdk_call();
        return registers(native::abi_info());
    }
    if number == 1 && args == [0; 6] {
        before_sdk_call();
        return registers(native::yield_now().map(|()| 0));
    }
    if number == 0x10 && args[1..].iter().all(|argument| *argument == 0) {
        if let Ok(handle) = Handle::from_raw(args[0]) {
            before_sdk_call();
            return registers(native::close(handle).map(|()| 0));
        }
    }
    fixture::call(number, args)
}

pub fn yield_now() {
    if call(1, [0; 6]) != (0, 0) {
        fail(1);
    }
}

pub fn worker() -> ! {
    loop {
        yield_now();
    }
}

#[derive(Clone, Copy)]
pub struct Message {
    pub bytes: [u8; 320],
}

impl Message {
    pub const fn new() -> Self {
        Self { bytes: [0; 320] }
    }
    pub fn sender(&self) -> u64 {
        crate::wire::get64(&self.bytes, 0)
    }
    pub fn tag(&self) -> u64 {
        crate::wire::get64(&self.bytes, 8)
    }
    pub fn length(&self) -> usize {
        crate::wire::get32(&self.bytes, 16) as usize
    }
    pub fn cap_count(&self) -> usize {
        crate::wire::get32(&self.bytes, 20) as usize
    }
    pub fn payload(&mut self, tag: u64, bytes: &[u8]) {
        if bytes.len() > 256 {
            fail(2);
        }
        crate::wire::put64(&mut self.bytes, 8, tag);
        crate::wire::put32(&mut self.bytes, 16, bytes.len() as u32);
        self.bytes[64..64 + bytes.len()].copy_from_slice(bytes);
    }
    pub fn close_caps(&mut self) -> bool {
        let count = core::cmp::min(self.cap_count(), 2);
        for index in 0..count {
            let handle = crate::wire::get64(&self.bytes, 32 + index * 16);
            let _ = call(0x10, [handle, 0, 0, 0, 0, 0]);
        }
        // Preserve the established service probe's had_caps result and exact
        // cleanup semantics. Protocol clients use the original SDK envelope.
        crate::wire::put32(&mut self.bytes, 20, 0);
        count != 0
    }
}

pub fn output() -> Message {
    // Block completions have sender 0 and stay on the original block ABI path;
    // they are not IPC receives and must not be parsed as an SDK peer message.
    Message {
        bytes: fixture::output().bytes,
    }
}

pub fn send_once(endpoint: u64, message: &Message) -> (i64, u64) {
    let handle = Handle::from_raw(endpoint);
    let outgoing = Outgoing::from_bytes(message.bytes);
    match (handle, outgoing) {
        (Ok(handle), Ok(outgoing)) => {
            before_sdk_call();
            registers(native::send(SendEndpoint::from_handle(handle), &outgoing).map(|()| 0))
        }
        _ => {
            // Deliberate malformed envelope/handle probes reach the real kernel.
            // Never count a local SDK rejection as a negative kernel result.
            fixture::send_once(
                endpoint,
                &fixture::Message {
                    bytes: message.bytes,
                },
            )
        }
    }
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
    let handle = match Handle::from_raw(endpoint) {
        Ok(handle) => handle,
        Err(_) => {
            return fixture::receive(endpoint).map(|message| Message {
                bytes: message.bytes,
            })
        }
    };
    before_sdk_call();
    let received = native::receive(ReceiveEndpoint::from_handle(handle));
    after_sdk_call();
    match received {
        Ok(message) => Some(Message {
            bytes: *message.as_bytes(),
        }),
        Err(error) if error.is_again() => None,
        Err(_) => fail(7),
    }
}

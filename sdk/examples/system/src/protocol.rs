//! Small FR-I01 coordination protocol; no service/content authority is encoded.
use nieuw_os_sdk_runtime::{
    native, Error, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};
pub const TAG: u64 = 0x5359_5354_454d_0001;
pub const READY: u8 = 1;
pub const GO: u8 = 2;
pub const DONE: u8 = 3;
pub const WORKER_READY: u8 = 4;
pub const LIMIT: usize = 400_000;
pub fn tick() {
    super::support::put(8, super::support::word(8).wrapping_add(1));
}
pub fn handle(at: u64) -> Handle {
    Handle::from_raw(super::support::word(at)).unwrap_or_else(|_| fail(90))
}
pub fn fail(step: u64) -> ! {
    super::support::put(240, step);
    super::support::put(40, 0x100 + step);
    native::exit(1)
}
pub fn check(ok: bool, step: u64) {
    if !ok {
        fail(step);
    }
}
pub fn send(to: SendEndpoint, op: u8) {
    let message = Outgoing::new(TAG, &[op]).unwrap_or_else(|_| fail(91));
    for _ in 0..LIMIT {
        tick();
        match native::send(to, &message) {
            Ok(()) => return,
            Err(e) if e.is_again() => {
                let _ = native::yield_now();
            }
            Err(_) => fail(92),
        }
    }
    fail(93)
}
pub fn receive(from: ReceiveEndpoint) -> Option<Received> {
    tick();
    match native::receive(from) {
        Ok(message) => {
            if message.cap_count() != 0 {
                let _ = native::close_received_caps(&message);
                fail(94);
            }
            if message.tag() != TAG || message.payload().len() != 1 {
                fail(95);
            }
            Some(message)
        }
        Err(e) if e.is_again() => {
            let _ = native::yield_now();
            None
        }
        Err(_) => fail(96),
    }
}
pub fn code<T>(result: Result<T, Error>) -> i64 {
    match result {
        Ok(_) => 0,
        Err(Error::Kernel(c)) => c,
        Err(_) => 1,
    }
}

//! Small FR-I01 coordination protocol; no service/content authority is encoded.
use nieuw_os_sdk_runtime::{
    native, Error, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};
use volume_core::{
    client::Transport,
    wire::{self as vw},
    Status as VolumeStatus,
};
pub const TAG: u64 = 0x5359_5354_454d_0001;
pub const READY: u8 = 1;
pub const GO: u8 = 2;
pub const DONE: u8 = 3;
pub const WORKER_READY: u8 = 4;
/// App → init: keys allocated and a record sealed in memory; nothing of it is
/// on disk yet. Init must durably persist the key state before replying.
pub const KEYS_READY: u8 = 5;
/// Init → app: key state is committed; the app may now commit ciphertext.
pub const PERSISTED: u8 = 6;
pub const LIMIT: usize = 400_000;
/// Synthetic volume identity of this acceptance profile. Record contexts and
/// the persistent key-state header are bound to it; a product adapter must
/// bind to the authenticated identity of the real volume instead.
pub const VOLUME_ID: [u8; 16] = *b"NOS-SYSTEM-VOL01";
/// Domain-0 object that holds the sealed record (owned by the app).
pub const SEALED_OBJECT: u64 = 1;
pub const SEALED_NAME: &[u8] = b"verzegeld";
/// First domain-1 object: the encrypted key-state snapshot (owned by init).
pub const KEYSTATE_OBJECT: u64 = 8;
pub const KEYSTATE_NAME: &[u8] = b"sleutelstatus";
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
pub fn bump(offset: u64) {
    super::support::put(offset, super::support::word(offset) + 1);
}
/// Deterministic plaintext of the persisted record; the host oracle recomputes
/// it to show that the committed bytes are not this plaintext.
pub fn record_byte(offset: usize) -> u8 {
    (11 + offset * 17 + offset / 97) as u8
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
/// One bounded request/reply over a fixed-size wire. The reply must come from
/// the addressed peer, carry the expected tag and no capability.
pub fn exchange<const N: usize>(
    send: SendEndpoint,
    receive: ReceiveEndpoint,
    peer: u64,
    tag: u64,
    request: &[u8; N],
) -> [u8; N] {
    let out = Outgoing::new(tag, request).unwrap_or_else(|_| fail(80));
    let mut sent = false;
    for _ in 0..LIMIT {
        tick();
        match native::send(send, &out) {
            Ok(()) => {
                sent = true;
                break;
            }
            Err(e) if e.is_again() => {
                let _ = native::yield_now();
            }
            Err(_) => fail(81),
        }
    }
    check(sent, 82);
    for _ in 0..LIMIT {
        tick();
        match native::receive(receive) {
            Ok(reply) => {
                if reply.cap_count() != 0 {
                    let _ = native::close_received_caps(&reply);
                    fail(83);
                }
                check(
                    reply.sender() == peer && reply.tag() == tag && reply.payload().len() == N,
                    84,
                );
                let mut bytes = [0; N];
                bytes.copy_from_slice(reply.payload());
                return bytes;
            }
            Err(e) if e.is_again() => {
                let _ = native::yield_now();
            }
            Err(_) => fail(85),
        }
    }
    fail(86)
}
/// Legacy v6 volume transport over one granted endpoint pair.
pub struct VolumeTransport {
    pub send: SendEndpoint,
    pub receive: ReceiveEndpoint,
    pub peer: u64,
}
impl Transport for VolumeTransport {
    fn call(&mut self, request: &[u8; vw::SIZE]) -> Result<[u8; vw::SIZE], VolumeStatus> {
        Ok(exchange(
            self.send,
            self.receive,
            self.peer,
            vw::TAG,
            request,
        ))
    }
}
pub fn code<T>(result: Result<T, Error>) -> i64 {
    match result {
        Ok(_) => 0,
        Err(Error::Kernel(c)) => c,
        Err(_) => 1,
    }
}

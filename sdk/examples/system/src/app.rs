#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Actual independent IPC clients for volume and Key Service. The disk test
//! data is deliberately plaintext; this is not B02 encrypted storage evidence.
#[allow(dead_code)]
mod protocol;
mod support;
use keyring_core::{wire as kw, Status as KeyStatus};
use nieuw_os_sdk_runtime::{
    native, BlockDevice, EntropySource, LaunchImage, Outgoing, ReceiveEndpoint, SendEndpoint,
};
use protocol::*;
use volume_core::{
    checksum_update,
    client::{Client, Transport},
    wire as vw, Status, FNV_OFFSET, PER_DOMAIN,
};

fn exchange<const N: usize>(
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
struct Volume {
    send: SendEndpoint,
    receive: ReceiveEndpoint,
    peer: u64,
}
impl Transport for Volume {
    fn call(&mut self, request: &[u8; vw::SIZE]) -> Result<[u8; vw::SIZE], Status> {
        Ok(exchange(
            self.send,
            self.receive,
            self.peer,
            vw::TAG,
            request,
        ))
    }
}
struct Keys {
    send: SendEndpoint,
    receive: ReceiveEndpoint,
    peer: u64,
    next: u64,
}
impl Keys {
    fn call(
        &mut self,
        op: u8,
        index: u8,
        body: &[u8],
        expected: KeyStatus,
    ) -> (kw::Header, [u8; kw::SIZE]) {
        let id = self.next;
        self.next += 1;
        let request = kw::request(op, index, id, body).unwrap_or_else(|| fail(70));
        let reply = exchange(self.send, self.receive, self.peer, kw::TAG, &request);
        let h = kw::Header::decode(&reply).unwrap_or_else(|| fail(71));
        check(
            h.op == (op | kw::REPLY)
                && h.index == index
                && h.request == id
                && h.status == expected.code(),
            72,
        );
        check(expected == KeyStatus::Ok || h.length == 0, 73);
        (h, reply)
    }
}
fn byte(offset: usize) -> u8 {
    (7 + offset * 13 + offset / 251) as u8
}
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    main()
}

#[inline(never)]
fn main() -> ! {
    check(native::check_abi().is_ok(), 1);
    let mut keys = Keys {
        send: SendEndpoint::from_handle(handle(64)),
        receive: ReceiveEndpoint::from_handle(handle(56)),
        peer: support::word(160),
        next: 1,
    };
    let mut volume = Client::new(
        Volume {
            send: SendEndpoint::from_handle(handle(80)),
            receive: ReceiveEndpoint::from_handle(handle(72)),
            peer: support::word(168),
        },
        1,
        1,
    );
    let registry = SendEndpoint::from_handle(handle(88));
    let control = ReceiveEndpoint::from_handle(handle(96));
    let recover = support::word(112) == 1;
    support::put(240, 2);
    let (h, status) = keys.call(kw::STATUS, 0, &[], KeyStatus::Ok);
    check(h.length == 14 && status[kw::HEADER] == 1, 2);
    let before = volume.stat(0);
    if recover {
        check(matches!(before,Ok(s) if s.revision==1 && s.length==4096), 3);
    } else {
        check(before == Err((Status::NotFound, 0)), 3);
    }
    check(
        volume.stat(PER_DOMAIN as u64) == Err((Status::Access, 0)),
        4,
    );
    // Every request crosses the real syscall boundary; endpoint handles cannot
    // be reinterpreted as device, entropy or process-launch authority.
    check(
        code(native::block_read(
            BlockDevice::from_handle(handle(80)),
            0,
            0,
            1,
        )) == -6,
        5,
    );
    check(
        code(native::entropy_request(
            EntropySource::from_handle(handle(64)),
            1,
        )) == -6,
        6,
    );
    check(
        code(native::spawn(LaunchImage::from_handle(handle(64)), 0, &[])) == -6,
        7,
    );
    support::put(408, 3);
    support::put(448, 2);
    send(registry, READY);
    let mut go = false;
    for _ in 0..LIMIT {
        if let Some(m) = receive(control) {
            check(m.sender() == support::word(184) && m.payload() == [GO], 8);
            go = true;
            break;
        }
    }
    check(go, 9);
    support::put(240, 10);
    if !recover {
        check(
            volume
                .write(0, 0, b"systeemproef", 4096, |offset, bytes| {
                    for (i, b) in bytes.iter_mut().enumerate() {
                        *b = byte(offset + i);
                    }
                })
                .is_ok_and(|(revision, _)| revision == 1),
            10,
        );
    }
    let stat = volume.stat(0).unwrap_or_else(|_| fail(11));
    check(
        stat.revision == 1
            && stat.length == 4096
            && &stat.name[..stat.name_length] == b"systeemproef",
        12,
    );
    let mut digest = FNV_OFFSET;
    let mut valid = true;
    check(
        volume.read(0, 1, 4096, |offset, bytes| {
            for (i, b) in bytes.iter().enumerate() {
                valid &= *b == byte(offset + i);
            }
            digest = checksum_update(digest, bytes);
        }) == Ok(4096),
        13,
    );
    check(valid && digest == stat.content_sum, 14);
    support::put(240, 15);
    let (h, created) = keys.call(kw::GENERATE, 0, &[], KeyStatus::Ok);
    check(h.length == 1, 15);
    let index = created[kw::HEADER];
    let plain = b"Nieuw OS: echte lokale cryptografie via ring-3 IPC.";
    let (h, sealed) = keys.call(kw::SEAL, index, plain, KeyStatus::Ok);
    let length = usize::from(h.length);
    check(length == plain.len() + 40, 16);
    let mut blob = [0; keyring_core::MAX_SEALED];
    blob[..length].copy_from_slice(&sealed[kw::HEADER..kw::HEADER + length]);
    let (h, opened) = keys.call(kw::UNSEAL, index, &blob[..length], KeyStatus::Ok);
    check(
        usize::from(h.length) == plain.len()
            && &opened[kw::HEADER..kw::HEADER + plain.len()] == plain,
        17,
    );
    blob[length - 1] ^= 1;
    let _ = keys.call(kw::UNSEAL, index, &blob[..length], KeyStatus::Auth);
    blob[length - 1] ^= 1;
    let _ = keys.call(kw::REVOKE, index, &[], KeyStatus::Ok);
    let _ = keys.call(kw::UNSEAL, index, &blob[..length], KeyStatus::Revoked);
    for (at, value) in [
        (400, 1),
        (416, 1),
        (424, 4096),
        (432, digest),
        (440, 4),
        (40, 1),
    ] {
        support::put(at, value);
    }
    send(registry, DONE);
    native::exit(0)
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(99)
}

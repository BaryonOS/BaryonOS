#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Actual independent IPC clients for volume and Key Service. The disk test
//! data is deliberately plaintext; this is not B02 encrypted storage evidence.
#[allow(dead_code)]
mod protocol;
mod support;
use keyring_core::{
    records::{RecordContext, RecordKind},
    wire as kw, Status as KeyStatus, MAX_PLAINTEXT, MAX_SEALED, NONCE_BYTES, TAG_BYTES,
};
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
    fn reply(&self, request: &[u8; kw::SIZE], expected: KeyStatus) -> (kw::Header, [u8; kw::SIZE]) {
        let sent = kw::Header::decode(request).unwrap_or_else(|| fail(70));
        let reply = exchange(self.send, self.receive, self.peer, kw::TAG, request);
        let h = kw::Header::decode(&reply).unwrap_or_else(|| fail(71));
        check(
            h.op == (sent.op | kw::REPLY)
                && h.index == sent.index
                && h.request == sent.request
                && h.status == expected.code(),
            72,
        );
        check(expected == KeyStatus::Ok || h.length == 0, 73);
        (h, reply)
    }

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
        self.reply(&request, expected)
    }

    fn record(
        &mut self,
        op: u8,
        index: u8,
        context: RecordContext,
        data: &[u8],
        expected: KeyStatus,
    ) -> (kw::Header, [u8; kw::SIZE]) {
        let id = self.next;
        self.next += 1;
        let request = kw::record_request(op, index, id, context, data).unwrap_or_else(|_| fail(74));
        self.reply(&request, expected)
    }
}

const RECORD_SEAL: u64 = 456;
const RECORD_UNSEAL: u64 = 464;
const RECORD_CONTEXT_DENIED: u64 = 472;
const RECORD_REVOKED_DENIED: u64 = 480;

fn bump(offset: u64) {
    support::put(offset, support::word(offset) + 1);
}

/// Exercise the real ring-3 record protocol against the real Key Service.
/// These are in-memory records under a synthetic volume identity, not encrypted
/// disk storage: the independent 4096-byte volume fixture stays unchanged.
#[inline(never)]
fn records_before_revoke(keys: &mut Keys, index: u8) -> (RecordContext, [u8; MAX_SEALED]) {
    let context = RecordContext {
        volume: *b"native-record-v1",
        domain: 0,
        object: 0,
        revision: 1,
        chunk: 3,
        length: MAX_PLAINTEXT as u16,
        kind: RecordKind::Content,
        schema: 1,
    };
    let plaintext = core::array::from_fn::<_, MAX_PLAINTEXT, _>(|i| byte(i + 0x4000));
    let (header, reply) = keys.record(kw::SEAL_RECORD, index, context, &plaintext, KeyStatus::Ok);
    check(usize::from(header.length) == MAX_SEALED, 30);
    let mut sealed = [0; MAX_SEALED];
    sealed.copy_from_slice(&reply[kw::HEADER..kw::HEADER + MAX_SEALED]);
    bump(RECORD_SEAL);
    let (header, reply) = keys.record(kw::UNSEAL_RECORD, index, context, &sealed, KeyStatus::Ok);
    check(
        usize::from(header.length) == plaintext.len()
            && reply[kw::HEADER..kw::HEADER + plaintext.len()] == plaintext,
        31,
    );
    bump(RECORD_UNSEAL);

    let mut corrupted = sealed;
    corrupted[MAX_SEALED - 1] ^= 1;
    keys.record(
        kw::UNSEAL_RECORD,
        index,
        context,
        &corrupted,
        KeyStatus::Auth,
    );
    bump(RECORD_CONTEXT_DENIED);
    for changed in [
        RecordContext {
            volume: [0x53; 16],
            ..context
        },
        RecordContext {
            object: 1,
            ..context
        },
        RecordContext {
            revision: 2,
            ..context
        },
        RecordContext {
            chunk: 4,
            ..context
        },
        RecordContext {
            kind: RecordKind::Name,
            ..context
        },
        RecordContext {
            schema: 2,
            ..context
        },
    ] {
        keys.record(kw::UNSEAL_RECORD, index, changed, &sealed, KeyStatus::Auth);
        bump(RECORD_CONTEXT_DENIED);
    }

    // Keep the complete tag but remove the last ciphertext byte. The changed
    // expected length is structurally valid; authentication must reject it.
    let shorter_context = RecordContext {
        length: context.length - 1,
        ..context
    };
    let shorter_end = NONCE_BYTES + MAX_PLAINTEXT - 1;
    let mut shortened = [0; MAX_SEALED - 1];
    shortened[..shorter_end].copy_from_slice(&sealed[..shorter_end]);
    shortened[shorter_end..].copy_from_slice(&sealed[MAX_SEALED - TAG_BYTES..]);
    keys.record(
        kw::UNSEAL_RECORD,
        index,
        shorter_context,
        &shortened,
        KeyStatus::Auth,
    );
    bump(RECORD_CONTEXT_DENIED);

    // Malformed requests cannot be made by the checked record builder, so use
    // the bounded generic KEY1 builder to exercise the native service parser.
    let mut body = [0; kw::RECORD_CONTEXT + MAX_SEALED];
    body[..kw::RECORD_CONTEXT]
        .copy_from_slice(&kw::encode_record_context(context).unwrap_or_else(|_| fail(32)));
    body[kw::RECORD_CONTEXT..].copy_from_slice(&sealed);
    body[13] = 1;
    keys.call(kw::UNSEAL_RECORD, index, &body, KeyStatus::Invalid);
    bump(RECORD_CONTEXT_DENIED);
    body[..kw::RECORD_CONTEXT]
        .copy_from_slice(&kw::encode_record_context(shorter_context).unwrap_or_else(|_| fail(33)));
    keys.call(kw::UNSEAL_RECORD, index, &body, KeyStatus::Invalid);
    bump(RECORD_CONTEXT_DENIED);
    (context, sealed)
}

fn records_after_revoke(
    keys: &mut Keys,
    index: u8,
    context: RecordContext,
    sealed: &[u8; MAX_SEALED],
) {
    keys.record(
        kw::UNSEAL_RECORD,
        index,
        context,
        sealed,
        KeyStatus::Revoked,
    );
    bump(RECORD_REVOKED_DENIED);
    keys.record(
        kw::SEAL_RECORD,
        index,
        context,
        &[0; MAX_PLAINTEXT],
        KeyStatus::Revoked,
    );
    bump(RECORD_REVOKED_DENIED);
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
    let (context, record) = records_before_revoke(&mut keys, index);
    let _ = keys.call(kw::REVOKE, index, &[], KeyStatus::Ok);
    let _ = keys.call(kw::UNSEAL, index, &blob[..length], KeyStatus::Revoked);
    records_after_revoke(&mut keys, index, context, &record);
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

#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Actual independent IPC clients for volume and Key Service. The disk test
//! data is deliberately plaintext; this is not B02 encrypted storage evidence.
#[allow(dead_code)]
mod protocol;
#[allow(dead_code)]
#[path = "../../random-probe/src/contract.rs"]
mod random_contract;
mod support;
use keyring_core::{
    persistent::PERSISTENT_BLOB,
    records::{RecordContext, RecordKind},
    wire as kw, Status as KeyStatus, MAX_PLAINTEXT, MAX_SEALED, NONCE_BYTES, SECRET_BYTES,
    TAG_BYTES,
};
use nieuw_os_catalog_client::NativeTransport;
use nieuw_os_document_session::{
    volume::{Buffers, DocumentSession, Error as DocumentError, SaveProgress, VolumeSession},
    EditMode, SaveOutcome,
};
use nieuw_os_keyring_client::{
    KeyIndex, NativeTransport as NativeKeyTransport, Operation as KeyOperation, Poll as KeyPoll,
    Reply as KeyReply, Session as KeySession, State as KeyState,
};
use nieuw_os_sdk_runtime::{
    native, BlockDevice, EntropySource, LaunchImage, ReceiveEndpoint, SendEndpoint,
};
use protocol::*;
use volume_core::{checksum_update, client::Client, wire as vw, Status, FNV_OFFSET, PER_DOMAIN};

/// Only this trusted negative-test fixture borrows the session's numeric
/// handles. Every raw call is serialized between completed typed exchanges;
/// it never owns/closes the channels or replaces native denials with SDK errors.
struct RawKeyProbe {
    send: SendEndpoint,
    receive: ReceiveEndpoint,
    peer: u64,
    next: u64,
}
impl RawKeyProbe {
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
// B02 persistence evidence (kernel system.rs PERSIST_WORDS).
const RECORD_PERSISTED: u64 = 528;
const RECORD_RESTORED_UNSEAL: u64 = 536;
const RECORD_RESTORED_REVOKED_DENIED: u64 = 544;
const RECORD_RESTORED_TAMPER_DENIED: u64 = 552;
const RECORD_ADMIN_DENIED: u64 = 560;
const RECORD_OBJECT_REVISION: u64 = 568;

/// Context of the persisted record: bound to this volume identity, the exact
/// domain-0 object and the revision that the commit below creates.
fn sealed_context() -> RecordContext {
    RecordContext {
        volume: VOLUME_ID,
        domain: 0,
        object: SEALED_OBJECT,
        revision: 1,
        chunk: 0,
        length: MAX_PLAINTEXT as u16,
        kind: RecordKind::Content,
        schema: 1,
    }
}

fn record_plaintext() -> [u8; MAX_PLAINTEXT] {
    core::array::from_fn(record_byte)
}

/// Persistence opcodes must be unreachable from an ordinary domain endpoint,
/// whatever secret or volume the app claims. Both exact-length requests are
/// refused as Invalid before any cryptographic processing.
fn admin_ops_denied_on_domain_endpoint(probe: &mut RawKeyProbe) {
    let secret = [0x11u8; SECRET_BYTES];
    let blob = [0u8; PERSISTENT_BLOB];
    let id = probe.next;
    probe.next += 2;
    let export = kw::export_persistent_request(id, &VOLUME_ID, &secret).unwrap_or_else(|| fail(44));
    probe.reply(&export, KeyStatus::Invalid);
    bump(RECORD_ADMIN_DENIED);
    let restore = kw::restore_persistent_request(id + 1, &VOLUME_ID, &secret, 0, &blob)
        .unwrap_or_else(|| fail(45));
    probe.reply(&restore, KeyStatus::Invalid);
    bump(RECORD_ADMIN_DENIED);
}

/// Normal run: allocate a second key and seal one record in memory. Init must
/// durably persist the key state before this app commits any ciphertext, so
/// the sealed bytes stay in memory until PERSISTED arrives.
#[inline(never)]
fn allocate_and_seal(
    keys: &mut KeySession<NativeKeyTransport>,
    probe: &mut RawKeyProbe,
    registry: SendEndpoint,
    control: ReceiveEndpoint,
    sealed: &mut [u8; MAX_SEALED],
) {
    let key = match keys.exchange(KeyOperation::Generate) {
        Ok(KeyReply::Generated(key)) => key,
        _ => fail(40),
    };
    check(key.raw() == 1, 41);
    let plaintext = record_plaintext();
    match keys.exchange(KeyOperation::SealRecord {
        key,
        context: sealed_context(),
        plaintext: &plaintext,
    }) {
        Ok(KeyReply::Sealed(reply)) => {
            check(reply.as_bytes().len() == MAX_SEALED, 42);
            sealed.copy_from_slice(reply.as_bytes());
        }
        _ => fail(42),
    }
    check(!keys.is_busy(), 43);
    bump(RECORD_PERSISTED);
    admin_ops_denied_on_domain_endpoint(probe);
    send(registry, KEYS_READY);
    let mut persisted = false;
    for _ in 0..LIMIT {
        if let Some(m) = receive(control) {
            check(
                m.sender() == support::word(184) && m.payload() == [PERSISTED],
                46,
            );
            persisted = true;
            break;
        }
    }
    check(persisted, 47);
}

/// Cold restart: the ciphertext read back from the volume decrypts under the
/// restored key state, the pre-restart revocation still holds and a modified
/// record is refused.
#[inline(never)]
fn restored_record(
    keys: &mut KeySession<NativeKeyTransport>,
    probe: &mut RawKeyProbe,
    sealed: &[u8; MAX_SEALED],
) {
    let context = sealed_context();
    let key = KeyIndex::new(1).unwrap_or_else(|_| fail(52));
    match keys.exchange(KeyOperation::UnsealRecord {
        key,
        context,
        sealed,
    }) {
        Ok(KeyReply::Plaintext(reply)) => check(reply.as_bytes() == record_plaintext(), 53),
        _ => fail(53),
    }
    check(!keys.is_busy(), 54);
    bump(RECORD_RESTORED_UNSEAL);
    probe.record(kw::UNSEAL_RECORD, 0, context, sealed, KeyStatus::Revoked);
    bump(RECORD_RESTORED_REVOKED_DENIED);
    let mut corrupted = *sealed;
    corrupted[MAX_SEALED - 1] ^= 1;
    probe.record(kw::UNSEAL_RECORD, 1, context, &corrupted, KeyStatus::Auth);
    bump(RECORD_RESTORED_TAMPER_DENIED);
}
/// Exercise the real ring-3 record protocol against the real Key Service.
/// These are in-memory records under a synthetic volume identity, not encrypted
/// disk storage: the independent 4096-byte volume fixture stays unchanged.
#[inline(never)]
fn records_before_revoke(
    keys: &mut KeySession<NativeKeyTransport>,
    probe: &mut RawKeyProbe,
    key: KeyIndex,
) -> (RecordContext, [u8; MAX_SEALED]) {
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
    let mut sealed = [0; MAX_SEALED];
    match keys.exchange(KeyOperation::SealRecord {
        key,
        context,
        plaintext: &plaintext,
    }) {
        Ok(KeyReply::Sealed(reply)) => {
            check(reply.as_bytes().len() == MAX_SEALED, 30);
            sealed.copy_from_slice(reply.as_bytes());
        }
        _ => fail(30),
    }
    bump(RECORD_SEAL);
    match keys.exchange(KeyOperation::UnsealRecord {
        key,
        context,
        sealed: &sealed,
    }) {
        Ok(KeyReply::Plaintext(reply)) => check(reply.as_bytes() == plaintext, 31),
        _ => fail(31),
    }
    bump(RECORD_UNSEAL);

    // Keep all real raw negative requests: the service's native boundary,
    // rather than local client checks, must reject these exact cases.
    check(!keys.is_busy(), 34);
    let index = key.raw();
    let mut corrupted = sealed;
    corrupted[MAX_SEALED - 1] ^= 1;
    probe.record(
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
        probe.record(kw::UNSEAL_RECORD, index, changed, &sealed, KeyStatus::Auth);
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
    probe.record(
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
    probe.call(kw::UNSEAL_RECORD, index, &body, KeyStatus::Invalid);
    bump(RECORD_CONTEXT_DENIED);
    body[..kw::RECORD_CONTEXT]
        .copy_from_slice(&kw::encode_record_context(shorter_context).unwrap_or_else(|_| fail(33)));
    probe.call(kw::UNSEAL_RECORD, index, &body, KeyStatus::Invalid);
    bump(RECORD_CONTEXT_DENIED);
    (context, sealed)
}

fn records_after_revoke(
    keys: &mut RawKeyProbe,
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

/// Opt-in fault-isolation acceptance only. This uses the same sole live KEY
/// session; no reply alias, new authority, cached STATUS or host result.
fn status_after_random_terminal(keys: &mut KeySession<NativeKeyTransport>) {
    use random_contract as r;
    let magic = support::word(r::APP_MAGIC_AT);
    if magic == 0 {
        return;
    } // Every old profile retains its old ABI/flow.
    check(magic == r::APP_MAGIC, 90);
    let start = native::monotonic_ticks().unwrap_or_else(|_| fail(91));
    let end = start
        .checked_add(r::REQUEST_TICKS)
        .unwrap_or_else(|| fail(92));
    let mut last = start;
    while support::word(r::APP_RELEASE) != 1 {
        let now = native::monotonic_ticks().unwrap_or_else(|_| fail(91));
        check(now >= last && now < end, 93);
        last = now;
        native::yield_now().unwrap_or_else(|_| fail(94));
    }
    check(!keys.is_busy() && !keys.is_closed(), 95);
    let begin = native::monotonic_ticks().unwrap_or_else(|_| fail(91));
    check(begin >= last, 97);
    support::put(r::APP_STATUS_BEGIN, begin);
    let deadline = begin
        .checked_add(r::REQUEST_TICKS)
        .unwrap_or_else(|| fail(92));
    keys.begin(KeyOperation::Status)
        .unwrap_or_else(|_| fail(96));
    last = begin;
    loop {
        let now = native::monotonic_ticks().unwrap_or_else(|_| fail(91));
        if now < last || now >= deadline {
            let _ = keys.close();
            fail(93);
        }
        last = now;
        match keys.poll() {
            Ok(KeyPoll::Pending { .. }) => {}
            Ok(KeyPoll::Ready(KeyReply::Status(status))) => {
                check(status.state == KeyState::Ready, 96);
                break;
            }
            _ => fail(96),
        }
    }
    let done = native::monotonic_ticks().unwrap_or_else(|_| fail(91));
    check(done >= last, 97);
    support::put(r::APP_STATUS_END, done);
    support::put(r::APP_STATUS_ACK, 1);
}

/// Three 4-KiB caller buffers live only on the 64-KiB userspace stack. This
/// ordinary native SDK client owns the volume endpoints until explicit close.
/// It has no disk/Key-Service secrets, host shortcut or private success path.
#[inline(never)]
fn document(recover: bool, sealed: &mut [u8; MAX_SEALED]) -> u64 {
    let mut draft = [0; 4096];
    let mut original = [0; 4096];
    let mut staging = [0; 4096];
    let session = VolumeSession::new(
        NativeTransport,
        1,
        support::word(168),
        0,
        SendEndpoint::from_handle(handle(80)),
        ReceiveEndpoint::from_handle(handle(72)),
        LIMIT,
    )
    .unwrap_or_else(|_| fail(50));
    let mut document = DocumentSession::new(
        session,
        if recover {
            EditMode::ReadOnly
        } else {
            EditMode::ReadWrite
        },
        Buffers {
            draft: &mut draft,
            original: &mut original,
            staging: &mut staging,
        },
    )
    .unwrap_or_else(|_| fail(51));
    if !recover {
        check(document.create(0, b"systeemproef").is_ok(), 52);
        // Build the real fixture through the public edit API, without a fourth
        // document-sized buffer or aliased access to the borrowed draft.
        let mut source = [0; vw::DATA];
        for offset in (0..4096).step_by(source.len()) {
            let count = source.len().min(4096 - offset);
            for (index, value) in source[..count].iter_mut().enumerate() {
                *value = byte(offset + index);
            }
            check(
                document
                    .replace_range(offset..offset, &source[..count])
                    .is_ok(),
                53,
            );
        }
        check(document.is_dirty(), 54);
        check(document.begin_save().is_ok() && document.is_saving(), 55);
        // One APPEND is 160 bytes, below the service's 512-byte sector buffer:
        // cancellation releases the staged token before any content WRITE.
        check(
            document.save_next()
                == Ok(SaveProgress::Writing {
                    completed: vw::DATA,
                    total: 4096,
                }),
            56,
        );
        check(document.cancel_save().is_ok() && !document.is_saving(), 57);
        check(document.is_dirty() && !document.is_uncertain(), 58);
        check(
            document
                .draft()
                .is_some_and(|loaded| loaded.loaded_revision == 0),
            59,
        );
        // The successful final BEGIN uses request ID 3, preserving the existing
        // on-disk request identity and the final fixture's exact disk bytes.
        check(
            document.save() == Ok(SaveOutcome::Saved { revision: 1 }),
            60,
        );
        check(
            !document.is_dirty() && !document.is_saving() && !document.is_uncertain(),
            61,
        );
    }
    check(document.open(0).is_ok(), 62);
    let digest = {
        let loaded = document.draft().unwrap_or_else(|| fail(63));
        check(
            loaded.object == 0
                && loaded.loaded_revision == 1
                && loaded.name == b"systeemproef"
                && loaded.contents.len() == 4096,
            64,
        );
        check(
            loaded
                .contents
                .iter()
                .enumerate()
                .all(|(offset, value)| *value == byte(offset)),
            65,
        );
        checksum_update(FNV_OFFSET, loaded.contents)
    };
    let metadata = document.metadata().unwrap_or_else(|_| fail(66));
    check(
        metadata.revision == 1 && metadata.length == 4096 && metadata.content_sum == digest,
        67,
    );
    if recover {
        check(
            document.replace_contents(b"read-only must retain the recovered document")
                == Err(DocumentError::ReadOnly),
            68,
        );
        check(
            document.save() == Err(DocumentError::ReadOnly) && !document.is_dirty(),
            69,
        );
        // B02: the sealed record committed by the previous run is an ordinary
        // document; its bytes come back for the restored-key proof.
        check(document.open(SEALED_OBJECT).is_ok(), 76);
        let loaded = document.draft().unwrap_or_else(|| fail(77));
        check(
            loaded.object == SEALED_OBJECT
                && loaded.loaded_revision == 1
                && loaded.name == SEALED_NAME
                && loaded.contents.len() == MAX_SEALED,
            77,
        );
        sealed.copy_from_slice(loaded.contents);
        support::put(RECORD_OBJECT_REVISION, loaded.loaded_revision);
    } else {
        // B02: commit the sealed record only now, after init confirmed that the
        // key state is durable. The document path is the same as for plaintext.
        check(document.create(SEALED_OBJECT, SEALED_NAME).is_ok(), 70);
        check(document.replace_range(0..0, sealed).is_ok(), 71);
        check(
            document.save() == Ok(SaveOutcome::Saved { revision: 1 }),
            72,
        );
        let metadata = document.metadata().unwrap_or_else(|_| fail(73));
        check(
            metadata.revision == 1
                && metadata.length == MAX_SEALED
                && metadata.content_sum == checksum_update(FNV_OFFSET, sealed),
            73,
        );
        support::put(RECORD_OBJECT_REVISION, metadata.revision);
    }
    check(document.close().is_ok() && document.is_closed(), 75);
    digest
}
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    main()
}

#[inline(never)]
fn main() -> ! {
    check(native::check_abi().is_ok(), 1);
    #[cfg(feature = "native-clock")]
    {
        use nieuw_os_sdk_runtime::clock::WallClockQuality;

        let monotonic_before = native::monotonic_ticks().unwrap_or_else(|_| fail(21));
        let first = native::wall_clock_utc().unwrap_or_else(|_| fail(22));
        let second = native::wall_clock_utc().unwrap_or_else(|_| fail(23));
        let monotonic_after = native::monotonic_ticks().unwrap_or_else(|_| fail(26));
        check(
            first.quality == WallClockQuality::UnauthenticatedRtc
                && second.quality == WallClockQuality::UnauthenticatedRtc
                && second.unix_seconds >= first.unix_seconds
                && monotonic_after >= monotonic_before,
            24,
        );
        support::put(488, first.unix_seconds as u64);
        support::put(496, second.unix_seconds as u64);
        support::put(512, monotonic_before);
        support::put(520, monotonic_after);

        let mut number = 0x73u64;
        let mut secondary = 0u64;
        // SAFETY: This negative syscall probe supplies only scalar values (no
        // pointers) to the clock ABI and uses the documented int80 registers.
        unsafe {
            core::arch::asm!(
                "int 0x80",
                inout("rax") number,
                in("rdi") 1u64,
                in("rsi") 0u64,
                inout("rdx") secondary,
                in("r10") 0u64,
                in("r8") 0u64,
                in("r9") 0u64,
                options(nostack),
            );
        }
        check(number as i64 == -22 && secondary == 0, 25);
        support::put(504, 1);
    }
    let mut keys = KeySession::new(
        NativeKeyTransport,
        support::word(160),
        0,
        SendEndpoint::from_handle(handle(64)),
        ReceiveEndpoint::from_handle(handle(56)),
        LIMIT,
    )
    .unwrap_or_else(|_| fail(10));
    let mut probe = RawKeyProbe {
        send: SendEndpoint::from_handle(handle(64)),
        receive: ReceiveEndpoint::from_handle(handle(56)),
        peer: support::word(160),
        // Disjoint correlation IDs make raw fixture traffic recognizable.
        // They grant no authority and never overlap a pending typed request.
        next: 1 << 63,
    };
    let registry = SendEndpoint::from_handle(handle(88));
    let control = ReceiveEndpoint::from_handle(handle(96));
    let recover = support::word(112) == 1;
    support::put(240, 2);
    match keys.exchange(KeyOperation::Status) {
        Ok(KeyReply::Status(status)) => check(status.state == KeyState::Ready, 2),
        _ => fail(2),
    }
    {
        // Startup-only v6 probe preserves the real cross-domain rejection.
        // It ends before the v7 DocumentSession becomes the sole owner of
        // this endpoint pair; no two active sessions share these handles.
        let mut probe = Client::new(
            VolumeTransport {
                send: SendEndpoint::from_handle(handle(80)),
                receive: ReceiveEndpoint::from_handle(handle(72)),
                peer: support::word(168),
            },
            1,
            1,
        );
        let before = probe.stat(0);
        if recover {
            check(matches!(before,Ok(s) if s.revision==1 && s.length==4096), 3);
        } else {
            check(before == Err((Status::NotFound, 0)), 3);
        }
        check(probe.stat(PER_DOMAIN as u64) == Err((Status::Access, 0)), 4);
    }
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
    support::put(240, 15);
    let key = match keys.exchange(KeyOperation::Generate) {
        Ok(KeyReply::Generated(key)) => key,
        _ => fail(15),
    };
    // A cold restart restores the exact allocation count: the two indices of
    // the previous run are never reissued.
    check(key.raw() == if recover { 2 } else { 0 }, 15);
    let plain = b"Nieuw OS: echte lokale cryptografie via ring-3 IPC.";
    let mut blob = [0; MAX_SEALED];
    let length = match keys.exchange(KeyOperation::Seal {
        key,
        plaintext: plain,
    }) {
        Ok(KeyReply::Sealed(reply)) => {
            let length = reply.as_bytes().len();
            check(length == plain.len() + NONCE_BYTES + TAG_BYTES, 16);
            blob[..length].copy_from_slice(reply.as_bytes());
            length
        }
        _ => fail(16),
    };
    match keys.exchange(KeyOperation::Unseal {
        key,
        sealed: &blob[..length],
    }) {
        Ok(KeyReply::Plaintext(reply)) => check(reply.as_bytes() == plain, 17),
        _ => fail(17),
    }
    check(!keys.is_busy(), 18);
    blob[length - 1] ^= 1;
    let _ = probe.call(kw::UNSEAL, key.raw(), &blob[..length], KeyStatus::Auth);
    blob[length - 1] ^= 1;
    let (context, record) = records_before_revoke(&mut keys, &mut probe, key);
    check(
        matches!(
            keys.exchange(KeyOperation::Revoke(key)),
            Ok(KeyReply::Revoked)
        ),
        19,
    );
    check(!keys.is_busy(), 20);
    let _ = probe.call(kw::UNSEAL, key.raw(), &blob[..length], KeyStatus::Revoked);
    records_after_revoke(&mut probe, key.raw(), context, &record);
    status_after_random_terminal(&mut keys);
    support::put(240, 22);
    let mut sealed = [0; MAX_SEALED];
    if recover {
        admin_ops_denied_on_domain_endpoint(&mut probe);
    } else {
        allocate_and_seal(&mut keys, &mut probe, registry, control, &mut sealed);
    }
    support::put(240, 10);
    let digest = document(recover, &mut sealed);
    if recover {
        support::put(240, 23);
        restored_record(&mut keys, &mut probe, &sealed);
    }
    check(keys.close().is_ok() && keys.is_closed(), 21);
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

#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Ring-3 dependency handshake, actual fault/restart/kill lifecycle and the
//! B02 recovery agent: init alone holds the Key Service's administrative
//! ingress and the second volume domain. It persists the encrypted key state
//! before the app may commit ciphertext, and restores it after a cold start.
#[allow(dead_code)]
mod protocol;
mod support;
use keyring_core::{
    persistent::{SnapshotInfo, PERSISTENT_BLOB},
    random::clear_bytes,
    wire as kw, Status as KeyStatus, SECRET_BYTES,
};
use nieuw_os_sdk_runtime::{
    native, Capability, Error, Handle, LaunchImage, ProcessStatus, ReceiveEndpoint, Rights,
    SendEndpoint,
};
use protocol::*;
use volume_core::{checksum_update, client::Client, FNV_OFFSET};

// Evidence words read by the kernel (system.rs WORDS, init slot).
const EV_EXPORTS: u64 = 432;
const EV_RESTORES: u64 = 440;
const EV_WRONG_SECRET_DENIED: u64 = 448;
const EV_FLOOR_DENIED: u64 = 456;
const EV_ADMIN_OP_DENIED: u64 = 464;
const EV_KEYSTATE_BYTES: u64 = 472;
const EV_GENERATION: u64 = 480;

/// Synthetic acceptance secret of this profile's recovery agent. It is test
/// fixture data for the disposable VM image only: a product recovery agent
/// obtains the user's secret under an explicit administrative capability, and
/// the Key Service never stores any copy of it.
const RECOVERY_SECRET: [u8; SECRET_BYTES] = *b"synthetic-system-recovery-secret";

struct KeyAdmin {
    send: SendEndpoint,
    receive: ReceiveEndpoint,
    peer: u64,
    next: u64,
}

impl KeyAdmin {
    /// One administrative exchange. This caller-owned request copy holding the
    /// secret is wiped afterwards; the SDK wipes its own envelopes on drop.
    fn call(&mut self, request: &mut [u8; kw::SIZE], expected: KeyStatus) -> [u8; kw::SIZE] {
        let sent = kw::Header::decode(request).unwrap_or_else(|| fail(30));
        let reply = exchange(self.send, self.receive, self.peer, kw::TAG, request);
        clear_bytes(request);
        let header = kw::Header::decode(&reply).unwrap_or_else(|| fail(31));
        check(
            header.op == (sent.op | kw::REPLY)
                && header.index == sent.index
                && header.request == sent.request
                && header.status == expected.code(),
            32,
        );
        check(expected == KeyStatus::Ok || header.length == 0, 33);
        reply
    }

    fn id(&mut self) -> u64 {
        self.next += 1;
        self.next
    }

    fn export(&mut self) -> ([u8; PERSISTENT_BLOB], SnapshotInfo) {
        let id = self.id();
        let mut request = kw::export_persistent_request(id, &VOLUME_ID, &RECOVERY_SECRET)
            .unwrap_or_else(|| fail(34));
        let reply = self.call(&mut request, KeyStatus::Ok);
        let header = kw::Header::decode(&reply).unwrap_or_else(|| fail(34));
        check(
            usize::from(header.length) == kw::EXPORT_PERSISTENT_REPLY,
            34,
        );
        let mut blob = [0; PERSISTENT_BLOB];
        blob.copy_from_slice(&reply[kw::HEADER..kw::HEADER + PERSISTENT_BLOB]);
        let info = kw::decode_snapshot_info(
            &reply[kw::HEADER + PERSISTENT_BLOB..kw::HEADER + kw::EXPORT_PERSISTENT_REPLY],
        )
        .unwrap_or_else(|| fail(35));
        bump(EV_EXPORTS);
        (blob, info)
    }

    fn restore(
        &mut self,
        secret: &[u8; SECRET_BYTES],
        minimum_generation: u64,
        blob: &[u8; PERSISTENT_BLOB],
        expected: KeyStatus,
    ) -> Option<SnapshotInfo> {
        let id = self.id();
        let mut request =
            kw::restore_persistent_request(id, &VOLUME_ID, secret, minimum_generation, blob)
                .unwrap_or_else(|| fail(36));
        let reply = self.call(&mut request, expected);
        if expected != KeyStatus::Ok {
            return None;
        }
        let header = kw::Header::decode(&reply).unwrap_or_else(|| fail(37));
        check(usize::from(header.length) == kw::SNAPSHOT_INFO, 37);
        let info = kw::decode_snapshot_info(&reply[kw::HEADER..kw::HEADER + kw::SNAPSHOT_INFO])
            .unwrap_or_else(|| fail(37));
        bump(EV_RESTORES);
        Some(info)
    }

    /// The administrative ingress serves persistence only: a key operation
    /// sent there is refused before any state changes.
    fn legacy_denied(&mut self) {
        let id = self.id();
        let mut request = kw::request(kw::GENERATE, 0, id, &[]).unwrap_or_else(|| fail(38));
        self.call(&mut request, KeyStatus::Invalid);
        bump(EV_ADMIN_OP_DENIED);
    }

    fn negatives(&mut self, blob: &[u8; PERSISTENT_BLOB], floor: u64) {
        let mut wrong = RECOVERY_SECRET;
        wrong[0] ^= 1;
        self.restore(&wrong, 0, blob, KeyStatus::Auth);
        clear_bytes(&mut wrong);
        bump(EV_WRONG_SECRET_DENIED);
        // An independently trusted generation floor above the snapshot refuses
        // an authentic but stale snapshot. This profile has no such anchor on
        // disk; the floor here only demonstrates the mechanism.
        self.restore(&RECOVERY_SECRET, floor, blob, KeyStatus::Auth);
        bump(EV_FLOOR_DENIED);
        self.legacy_denied();
    }
}

fn admin() -> KeyAdmin {
    KeyAdmin {
        send: SendEndpoint::from_handle(handle(104)),
        receive: ReceiveEndpoint::from_handle(handle(112)),
        peer: support::word(168),
        next: 0x4000,
    }
}

fn volume() -> Client<VolumeTransport> {
    Client::new(
        VolumeTransport {
            send: SendEndpoint::from_handle(handle(88)),
            receive: ReceiveEndpoint::from_handle(handle(96)),
            peer: support::word(184),
        },
        1,
        0x200,
    )
}

fn expected_state(info: SnapshotInfo) {
    // Two domain-0 keys exist, index 0 is revoked; domain 1 was never used.
    check(info.created == [2, 0] && info.revoked == [1, 0], 20);
}

/// Normal run: export after the app's allocations, refuse the negative cases,
/// durably commit the snapshot under domain 1, then release the app.
#[inline(never)]
fn persist_key_state() {
    let mut keys = admin();
    let (blob, info) = keys.export();
    expected_state(info);
    keys.negatives(&blob, info.generation + 1);
    let mut store = volume();
    let (revision, _) = store
        .write(
            KEYSTATE_OBJECT,
            0,
            KEYSTATE_NAME,
            PERSISTENT_BLOB,
            |offset, buffer| {
                buffer.copy_from_slice(&blob[offset..offset + buffer.len()]);
            },
        )
        .unwrap_or_else(|_| fail(21));
    check(revision == 1, 21);
    let stat = store.stat(KEYSTATE_OBJECT).unwrap_or_else(|_| fail(22));
    check(
        stat.revision == 1
            && stat.length == PERSISTENT_BLOB
            && stat.content_sum == checksum_update(FNV_OFFSET, &blob),
        22,
    );
    support::put(EV_KEYSTATE_BYTES, PERSISTENT_BLOB as u64);
    // Re-importing the committed current state is idempotent for the live
    // service: same namespace, same generation.
    let again = keys
        .restore(&RECOVERY_SECRET, info.generation, &blob, KeyStatus::Ok)
        .unwrap_or_else(|| fail(23));
    check(again == info, 23);
    support::put(EV_GENERATION, info.generation);
}

/// Cold start on the unchanged read-only image: read the committed snapshot,
/// refuse the negative cases, then restore the exact key state.
#[inline(never)]
fn restore_key_state() {
    let mut store = volume();
    let stat = store.stat(KEYSTATE_OBJECT).unwrap_or_else(|_| fail(24));
    check(
        stat.revision == 1
            && stat.length == PERSISTENT_BLOB
            && &stat.name[..stat.name_length] == KEYSTATE_NAME,
        24,
    );
    let mut blob = [0; PERSISTENT_BLOB];
    let read = store
        .read(KEYSTATE_OBJECT, 1, PERSISTENT_BLOB, |offset, data| {
            blob[offset..offset + data.len()].copy_from_slice(data);
        })
        .unwrap_or_else(|_| fail(25));
    check(
        read == PERSISTENT_BLOB && stat.content_sum == checksum_update(FNV_OFFSET, &blob),
        25,
    );
    support::put(EV_KEYSTATE_BYTES, PERSISTENT_BLOB as u64);
    let mut keys = admin();
    keys.negatives(&blob, u64::MAX);
    let info = keys
        .restore(&RECOVERY_SECRET, 0, &blob, KeyStatus::Ok)
        .unwrap_or_else(|| fail(26));
    expected_state(info);
    support::put(EV_GENERATION, info.generation);
}

/// H05 (ADR-076): orderly poweroff with init's non-delegable POWER handle.
/// The kernel logs each refusal itself and decides; init only sequences.
#[inline(never)]
fn power_off_sequence(power: Handle, image: LaunchImage) -> ! {
    // Delegation is refused by type before any effect.
    let grant = Capability {
        handle: power,
        rights: Rights::POWER,
    };
    check(native::spawn(image, 0, &[grant]).is_err(), 50);
    check(native::derive(power, Rights::POWER).is_err(), 51);
    // The volume still holds BLOCK authority: refused, nothing happens.
    check(native::power_off(power) == Error::Kernel(-8), 52);
    // Final flush; on Ok the volume closes its block capability. A refused
    // flush (Unknown) leaves the decision to the kernel gate below.
    let _ = volume().shutdown(volume_core::PER_DOMAIN as u64);
    // Authority is checked before the gate: wrong type, forged generation.
    check(native::power_off(image.handle()) == Error::Kernel(-6), 53);
    let forged = Handle::from_raw(power.raw() + (1 << 32)).unwrap_or_else(|_| fail(54));
    check(native::power_off(forged) == Error::Kernel(-4), 54);
    for _ in 0..LIMIT * 8 {
        match native::power_off(power) {
            Error::Kernel(-8) => {
                let _ = native::yield_now();
            }
            // Not durable this boot: stay up, the next cold start recovers.
            Error::Kernel(-16) => native::exit(0),
            _ => fail(55),
        }
    }
    fail(56)
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    main()
}

#[inline(never)]
fn main() -> ! {
    check(native::check_abi().is_ok(), 1);
    let image = LaunchImage::from_handle(handle(56));
    let registry = ReceiveEndpoint::from_handle(handle(64));
    let grant = Capability {
        handle: handle(72),
        rights: Rights::SEND,
    };
    let app_control = SendEndpoint::from_handle(handle(80));
    let app = support::word(160);
    let recover = support::word(120) == 1;
    // The app reports READY only after real STATUS/STAT round trips to both
    // boot services. Init does not read another process's evidence memory.
    let mut ready = false;
    for _ in 0..LIMIT {
        if let Some(m) = receive(registry) {
            check(m.sender() == app && m.payload() == [READY], 2);
            ready = true;
            break;
        }
    }
    check(ready, 3);
    support::put(240, 4);
    let failed = native::spawn(image, 1, &[grant]).unwrap_or_else(|_| fail(4));
    let mut faulted = false;
    for _ in 0..LIMIT {
        tick();
        match native::status(failed) {
            Ok(ProcessStatus::Faulted(6)) => {
                faulted = true;
                break;
            }
            Ok(ProcessStatus::Running) => {
                let _ = native::yield_now();
            }
            _ => fail(5),
        }
    }
    check(faulted, 6);
    check(native::close(failed.handle()).is_ok(), 7);
    support::put(240, 8);
    let worker = native::spawn(image, 0, &[grant]).unwrap_or_else(|_| fail(8));
    let mut restarted = false;
    for _ in 0..LIMIT {
        if let Some(m) = receive(registry) {
            check(m.sender() != app && m.payload() == [WORKER_READY], 9);
            restarted = true;
            break;
        }
    }
    check(restarted, 10);
    if recover {
        // The key state must be back before the app performs any key work.
        support::put(240, 17);
        restore_key_state();
    }
    send(app_control, GO);
    support::put(240, 11);
    if !recover {
        let mut allocated = false;
        for _ in 0..LIMIT {
            if let Some(m) = receive(registry) {
                check(m.sender() == app && m.payload() == [KEYS_READY], 18);
                allocated = true;
                break;
            }
        }
        check(allocated, 19);
        support::put(240, 20);
        persist_key_state();
        send(app_control, PERSISTED);
        support::put(240, 27);
    }
    let mut done = false;
    for _ in 0..LIMIT {
        if let Some(m) = receive(registry) {
            check(m.sender() == app && m.payload() == [DONE], 11);
            done = true;
            break;
        }
    }
    check(done, 12);
    check(native::status(worker) == Ok(ProcessStatus::Running), 13);
    check(native::kill(worker).is_ok(), 14);
    check(native::status(worker) == Ok(ProcessStatus::Killed), 15);
    check(native::close(worker.handle()).is_ok(), 16);
    for (at, value) in [(400, 2), (408, 1), (416, 1), (424, 1), (40, 1)] {
        support::put(at, value);
    }
    if support::word(48) != 0 {
        power_off_sequence(handle(48), image);
    }
    native::exit(0)
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(99)
}

#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Ring-3 dependency handshake, actual fault/restart/kill lifecycle and the
//! B02 recovery agent: init alone holds the Key Service's administrative
//! ingress and the second volume domain. It persists the encrypted key state
//! before the app may commit ciphertext, and restores it after a cold start.
// H05-T5d: the LOCK caller is TEST code for the S3 profiles only.
#[cfg(all(feature = "lock-caller", not(feature = "test-system-s3")))]
compile_error!(
    "lock-caller (LOCK 0x92) exists only in the S3 TEST init; enable feature test-system-s3"
);
#[allow(dead_code)]
mod protocol;
mod support;
use keyring_core::{
    persistent::{SnapshotInfo, PERSISTENT_BLOB},
    random::clear_bytes,
    wire as kw, Status as KeyStatus, SECRET_BYTES,
};
use nieuw_os_sdk_runtime::{
    native, BlockDevice, Capability, EntropySource, Error, Handle, LaunchImage, Outgoing,
    ProcessStatus, ReceiveEndpoint, Rights, SendEndpoint,
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
// ADR-B02-KEY1-RELAY-20261123 (test profile): startup words from the kernel's
// relay manifest and the relay evidence words it reads at init's exit.
const RELAY_COMMIT_RECEIVE: u64 = 192;
const RELAY_COMMIT_REPLY: u64 = 200;
const RELAY_CASE: u64 = 208;
const EV_RELAY_RECEIVED: u64 = 760;
const EV_RELAY_COMMITTED: u64 = 768;
const EV_RELAY_REFUSED: u64 = 776;
const EV_RELAY_WITHHELD: u64 = 784;
const EV_RELAY_GENERATION: u64 = 792;
const EV_RELAY_SUM: u64 = 800;
const EV_RELAY_REVISION: u64 = 808;
const EV_RELAY_BEGUN: u64 = 816;
/// Bound of one relay phase (KEYS_READY, then DONE) in 100 Hz ticks.
const RELAY_PHASE_TICKS: u64 = 6000;
/// Committed key-state header: format, version, suite, volume id, reserved.
const KEYSTATE_HEADER: [u8; 32] = {
    let mut header = [0u8; 32];
    let magic = *b"NOSKEY01";
    let mut i = 0;
    while i < 16 {
        if i < 8 {
            header[i] = magic[i];
        }
        header[12 + i] = VOLUME_ID[i];
        i += 1;
    }
    header[8] = 1;
    header[10] = 1;
    header
};

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

/// Relay mode: supply the external secret once, before any domain traffic,
/// then show that a second supply and a key opcode on the admin ingress are
/// refused. Init makes no other call into the Key Service after this.
#[inline(never)]
fn supply_secret() {
    let mut keys = admin();
    let id = keys.id();
    let mut request = kw::supply_secret_request(id, &RECOVERY_SECRET);
    keys.call(&mut request, KeyStatus::Ok);
    let mut wrong = RECOVERY_SECRET;
    wrong[0] ^= 1;
    let id = keys.id();
    let mut request = kw::supply_secret_request(id, &wrong);
    clear_bytes(&mut wrong);
    keys.call(&mut request, KeyStatus::Invalid);
    keys.legacy_denied();
}

/// Init's side of the key-state commit ingress. The kernel endpoint object
/// is the authority; the sender check below is defense in depth only.
struct Relay {
    receive: ReceiveEndpoint,
    reply: SendEndpoint,
    key: u64,
    case: u64,
    store: Client<VolumeTransport>,
    revision: u64,
}

impl Relay {
    fn reply(&self, message: &[u8; kw::SIZE]) {
        let out = Outgoing::new(kw::TAG, message).unwrap_or_else(|_| fail(100));
        for _ in 0..LIMIT {
            tick();
            match native::send(self.reply, &out) {
                Ok(()) => return,
                Err(e) if e.is_again() => {
                    let _ = native::yield_now();
                }
                Err(_) => fail(101),
            }
        }
        fail(102)
    }

    /// At most one commit request per call; never blocks when none is queued.
    fn poll(&mut self) {
        tick();
        let message = match native::receive(self.receive) {
            Ok(message) => message,
            Err(e) if e.is_again() => return,
            Err(_) => fail(103),
        };
        bump(EV_RELAY_RECEIVED);
        let payload = message.payload();
        let header = kw::Header::decode(payload).unwrap_or(kw::Header {
            op: kw::COMMIT_KEYSTATE,
            index: 0,
            length: 0,
            request: 0,
            status: 0,
        });
        let request = if message.cap_count() != 0 {
            let _ = native::close_received_caps(&message);
            None
        } else if message.sender() != self.key || message.tag() != kw::TAG {
            None
        } else {
            kw::decode_commit_request(payload).filter(|(_, blob)| blob[..32] == KEYSTATE_HEADER)
        };
        // Refused before any volume call: certainly not committed.
        let Some((generation, blob)) = request else {
            bump(EV_RELAY_REFUSED);
            self.reply(&kw::commit_reply(&header, false));
            return;
        };
        if support::word(EV_RELAY_RECEIVED) > 3 {
            match self.case {
                2 => {
                    bump(EV_RELAY_REFUSED);
                    self.reply(&kw::commit_reply(&header, false));
                }
                3 => bump(EV_RELAY_WITHHELD),
                _ => fail(104),
            }
            return;
        }
        // From here NOT_COMMITTED is never sent: every write error after this
        // point is reported as Degraded, which the service maps to Unknown.
        bump(EV_RELAY_BEGUN);
        let written = self.store.write(
            KEYSTATE_OBJECT,
            self.revision,
            KEYSTATE_NAME,
            PERSISTENT_BLOB,
            |offset, buffer| buffer.copy_from_slice(&blob[offset..offset + buffer.len()]),
        );
        match written {
            Ok((revision, _)) if revision == self.revision + 1 => {
                self.revision = revision;
                bump(EV_RELAY_COMMITTED);
                support::put(EV_RELAY_GENERATION, generation);
                support::put(EV_RELAY_SUM, checksum_update(FNV_OFFSET, &blob));
                support::put(EV_RELAY_REVISION, revision);
                self.reply(&kw::commit_reply(&header, true));
            }
            _ => self.reply(&kw::reply(&header, KeyStatus::Degraded, &[])),
        }
    }

    /// Serve commits while waiting for one registry message, tick-bounded.
    fn wait(&mut self, registry: ReceiveEndpoint, app: u64, expected: u8, step: u64) {
        let start = native::monotonic_ticks().unwrap_or_else(|_| fail(105));
        let deadline = start
            .checked_add(RELAY_PHASE_TICKS)
            .unwrap_or_else(|| fail(105));
        loop {
            self.poll();
            if let Some(m) = receive(registry) {
                check(m.sender() == app && m.payload() == [expected], step);
                return;
            }
            let now = native::monotonic_ticks().unwrap_or_else(|_| fail(105));
            check(now < deadline, step + 1);
        }
    }
}

/// After GO: commits are served while the app works; at KEYS_READY the
/// committed snapshot on disk must match the last acknowledged commit.
#[inline(never)]
fn relay_sequence(registry: ReceiveEndpoint, app: u64, app_control: SendEndpoint, case: u64) {
    let mut relay = Relay {
        receive: ReceiveEndpoint::from_handle(handle(RELAY_COMMIT_RECEIVE)),
        reply: SendEndpoint::from_handle(handle(RELAY_COMMIT_REPLY)),
        key: support::word(168),
        case,
        store: volume(),
        revision: 0,
    };
    relay.wait(registry, app, KEYS_READY, 18);
    support::put(240, 20);
    let stat = relay
        .store
        .stat(KEYSTATE_OBJECT)
        .unwrap_or_else(|_| fail(22));
    check(
        stat.revision == relay.revision
            && stat.revision == support::word(EV_RELAY_COMMITTED)
            && stat.length == PERSISTENT_BLOB
            && stat.content_sum == support::word(EV_RELAY_SUM),
        22,
    );
    support::put(EV_KEYSTATE_BYTES, PERSISTENT_BLOB as u64);
    support::put(EV_GENERATION, support::word(EV_RELAY_GENERATION));
    send(app_control, PERSISTED);
    support::put(240, 27);
    relay.wait(registry, app, DONE, 11);
}

/// Legacy flow: export and commit at KEYS_READY (normal), then wait for DONE.
#[inline(never)]
fn persist_and_wait(registry: ReceiveEndpoint, app: u64, app_control: SendEndpoint, recover: bool) {
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
    let cold = support::word(576);
    if cold == 1 || cold == 4 {
        // Withholding means this recovery agent never submits the fixture secret;
        // it submits a zero substitute, not a claim of physical key erasure.
        let secret = if cold == 4 {
            [0; SECRET_BYTES]
        } else {
            RECOVERY_SECRET
        };
        keys.restore(&secret, 0, &blob, KeyStatus::Auth);
        support::put(584, 1);
        native::exit(0);
    }
    if cold == 0 {
        keys.negatives(&blob, u64::MAX);
    }
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
    // test-system-s3 kernels set word 592 (2: the held-completion fixture,
    // 3: the A7 LOCK-busy fixture); every other profile skips SUSPEND.
    let word = support::word(592);
    let held = word == 2;
    let busy = word == 3;
    let s3 = word == 1 || held || busy;
    if s3 {
        // KL2: the instance is not LOCKED yet (-25, before the storage gate).
        check(native::suspend(power) == Err(Error::Kernel(-25)), 70);
    }
    // Final flush; on Ok the volume closes its block capability. A refused
    // flush (Unknown) leaves the decision to the kernel gate below.
    let _ = volume().shutdown(volume_core::PER_DOMAIN as u64);
    // Authority is checked before the gate: wrong type, forged generation.
    check(native::power_off(image.handle()) == Error::Kernel(-6), 53);
    let forged = Handle::from_raw(power.raw() + (1 << 32)).unwrap_or_else(|_| fail(54));
    check(native::power_off(forged) == Error::Kernel(-4), 54);
    if s3 {
        suspend_sequence(power, image, forged, held, busy);
    }
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

/// H05-T4 (ADR-H05-S3-SYSTEM-20261123): authority refusals, then the one
/// granted suspend, proof of ticks after the wake, then the one-shot refusal.
/// The kernel logs each SUSPEND result itself; init only sequences.
#[inline(never)]
fn suspend_sequence(power: Handle, image: LaunchImage, forged: Handle, held: bool, busy: bool) {
    check(
        native::suspend(image.handle()) == Err(Error::Kernel(-6)),
        71,
    );
    check(native::suspend(forged) == Err(Error::Kernel(-4)), 72);
    let (mut number, mut secondary) = (0x91u64, 7u64);
    // SAFETY: Scalar-only negative probe: a nonzero second argument with the
    // documented int80 registers; no pointer reaches the kernel.
    unsafe {
        core::arch::asm!(
            "int 0x80",
            inout("rax") number,
            in("rdi") power.raw(),
            in("rsi") 1u64,
            inout("rdx") secondary,
            in("r10") 0u64,
            in("r8") 0u64,
            in("r9") 0u64,
            options(nostack),
        );
    }
    check(number as i64 == -1 && secondary == 0, 73);
    // H05-T5d (KL2): LOCK ends the Key Service instance before any sleep.
    #[cfg(feature = "lock-caller")]
    let locked = lock_sequence(power, busy);
    #[cfg(not(feature = "lock-caller"))]
    let locked = {
        let _ = busy;
        check(false, 81);
        false
    };
    if !locked {
        // A7: LOCK stayed busy, so SUSPEND refuses (-25) and init returns to
        // S5 once the kernel has delivered its own requests (word 640).
        check(native::suspend(power) == Err(Error::Kernel(-25)), 83);
        for _ in 0..LIMIT {
            if support::word(640) == 1 {
                return;
            }
            let _ = native::yield_now();
        }
        fail(84);
    }
    // Ring-3 stack memory must survive the sleep unchanged.
    let mut pattern = [0u64; 8];
    for (index, slot) in pattern.iter_mut().enumerate() {
        let value = (slot as *mut u64 as u64) ^ 0x5a5a_5a5a_0000_0000 ^ index as u64;
        // SAFETY: Aligned write into this frame's own live local array.
        unsafe { core::ptr::write_volatile(slot, value) };
    }
    // Held-completion fixture: the volume broker still holds the completed
    // SHUTDOWN flush, so the kernel refuses once (-8) before any I/O.
    if held {
        check(native::suspend(power) == Err(Error::Kernel(-8)), 80);
    }
    // Storage is closed after SHUTDOWN Ok, so the next request is granted.
    // No -25 retry: m3_restore finished LOCK's stack scrub before this
    // process ran again, so KL2 sees LOCKED and erased (fail closed).
    check(native::suspend(power) == Ok(()), 74);
    for (index, slot) in pattern.iter_mut().enumerate() {
        let expected = (slot as *mut u64 as u64) ^ 0x5a5a_5a5a_0000_0000 ^ index as u64;
        // SAFETY: Aligned read of the same live local array.
        check(unsafe { core::ptr::read_volatile(slot) } == expected, 75);
    }
    // The kernel sets word 616 on the first timer interrupt after the wake.
    let mut ticked = false;
    for _ in 0..LIMIT {
        if support::word(616) == 1 {
            ticked = true;
            break;
        }
        let _ = native::yield_now();
    }
    check(ticked, 76);
    check(native::suspend(power) == Err(Error::Kernel(-14)), 77);
    // Devices are lost: well-formed entropy and block requests are refused
    // with -14 before any authority check. Init holds no entropy or block
    // capability and the storage gate closed every block object, so its own
    // POWER handle (-6 before the wake) is the only handle it can present.
    // H05-T5b: a test-system-s3-reinit kernel (word 624) re-initialised
    // virtio-rng before word 616, so entropy reaches the authority check.
    let entropy = if support::word(624) == 1 { -6 } else { -14 };
    check(
        native::entropy_request(EntropySource::from_handle(power), 1)
            == Err(Error::Kernel(entropy)),
        78,
    );
    // H05-T5c: a test-system-s3-blk-reinit kernel sets word 632 to the
    // expected magnitude: 6 (authority, after the validated virtio-blk
    // re-init) or 24 (lane 0 quarantined after a refused re-init).
    let block = match support::word(632) {
        0 => -14,
        magnitude => -(magnitude as i64),
    };
    check(
        native::block_read(BlockDevice::from_handle(power), 0, 0, 1) == Err(Error::Kernel(block)),
        79,
    );
}

/// A7: init's LOCK attempts while the kernel keeps the broker busy.
#[cfg(feature = "lock-caller")]
const LOCK_BUSY_ATTEMPTS: usize = 8;

/// H05-T5d (ADR-KEYRING-LOCK-20261126): LOCK 0x92 with the POWER handle. -8
/// (an entropy request or a reply of the instance still in flight) is retried
/// with a bounded number of yields; the A7 profile expects exactly
/// LOCK_BUSY_ATTEMPTS refusals and no grant. Anything else fails.
#[cfg(feature = "lock-caller")]
#[inline(never)]
fn lock_sequence(power: Handle, busy: bool) -> bool {
    let attempts = if busy { LOCK_BUSY_ATTEMPTS } else { LIMIT };
    for _ in 0..attempts {
        // rdx is also argument 2 (context[12]), so it must enter as 0.
        let (mut number, mut secondary) = (0x92u64, 0u64);
        // SAFETY: Scalar-only call with the documented int80 registers and
        // zero reserved arguments; no pointer reaches the kernel.
        unsafe {
            core::arch::asm!(
                "int 0x80",
                inout("rax") number,
                in("rdi") power.raw(),
                in("rsi") 0u64,
                inout("rdx") secondary,
                in("r10") 0u64,
                in("r8") 0u64,
                in("r9") 0u64,
                options(nostack),
            );
        }
        check(secondary == 0, 82);
        match number as i64 {
            0 if !busy => return true,
            -8 => {
                let _ = native::yield_now();
            }
            _ => fail(82),
        }
    }
    check(busy, 82);
    false
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
    let relay = support::word(RELAY_CASE);
    if relay != 0 {
        // The Key Service gates domain traffic on this supply, and the app's
        // READY follows a Key Service STATUS: supply strictly first.
        supply_secret();
    }
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
    if relay != 0 {
        relay_sequence(registry, app, app_control, relay);
    } else {
        persist_and_wait(registry, app, app_control, recover);
    }
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

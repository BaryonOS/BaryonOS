//! Standalone host tests of the actual kernel wipe and actual capability types.
#![deny(unsafe_op_in_unsafe_fn)]
#[path = "../../kernel/src/m2/m3/snapshot_wipe.rs"]
mod snapshot_wipe;
use capability_core::{AuthoritySnapshot, Cap, Engine, Message, SnapshotLock};
use core::{mem, ptr};
use std::ffi::c_void;
unsafe extern "C" {
    fn mmap(
        address: *mut c_void,
        len: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: isize,
    ) -> *mut c_void;
    fn mprotect(address: *mut c_void, len: usize, prot: i32) -> i32;
    fn munmap(address: *mut c_void, len: usize) -> i32;
}
struct Guarded(*mut u8);
impl Guarded {
    fn new() -> Self {
        // SAFETY: Private anonymous host test RAM; only its middle page is
        // accessible. The object is never shared with kernel/host hardware.
        let base = unsafe { mmap(ptr::null_mut(), 12288, 0, 2 | 0x20, -1, 0) };
        assert_ne!(base as usize, usize::MAX);
        assert_eq!(
            unsafe { mprotect(base.cast::<u8>().add(4096).cast(), 4096, 3) },
            0
        );
        Self(base.cast())
    }
    fn start(&self) -> *mut u8 {
        unsafe { self.0.add(4096) }
    }
    fn end(&self) -> *mut u8 {
        unsafe { self.0.add(8192) }
    }
}
impl Drop for Guarded {
    fn drop(&mut self) {
        assert_eq!(unsafe { munmap(self.0.cast(), 12288) }, 0);
    }
}
fn marker(seed: u64) -> Message {
    let mut m = Message::new();
    m.sender_pid = seed | 1;
    m.tag = seed.rotate_left(7) | 1;
    m.len = 0xffff_ffff;
    m.cap_count = 0x1234_5678;
    m.reserved = !seed;
    m.caps = [Cap {
        handle: seed | 0x0101,
        rights: !seed,
    }; 2];
    for (i, byte) in m.data.iter_mut().enumerate() {
        *byte = (i as u8).wrapping_mul(31) ^ seed as u8;
    }
    m
}
fn assert_erased(m: &Message) {
    assert_eq!(*m, Message::new());
    // SAFETY: The actual wipe initialized EVERY byte of the complete object,
    // including any padding; reading the resulting byte values is now valid.
    let bytes = unsafe {
        core::slice::from_raw_parts(ptr::from_ref(m).cast::<u8>(), mem::size_of::<Message>())
    };
    assert!(bytes.iter().all(|byte| *byte == 0));
}
#[test]
fn complete_message_headers_descriptors_and_every_payload_byte_zero() {
    assert_eq!(mem::size_of::<Message>(), 320);
    assert!(mem::align_of::<Message>() >= 8);
    for seed in [0, 1, u64::MAX, 0x1234_5678_9abc_def0] {
        let mut m = marker(seed);
        snapshot_wipe::wipe_message(&mut m);
        assert_erased(&m);
    }
}
#[test]
fn exact_guard_page_start_and_end_without_reading_old_padding() {
    let pages = Guarded::new();
    for pointer in [pages.start(), unsafe {
        pages.end().sub(mem::size_of::<Message>())
    }] {
        let message = pointer.cast::<Message>();
        // SAFETY: Middle page contains a full, correctly aligned Message at
        // either exact boundary. Initialize fields, then lend it exclusively.
        unsafe {
            message.write(marker(0xdead_beef));
            snapshot_wipe::wipe_message(&mut *message);
            assert_erased(&*message);
        }
    }
}
#[test]
fn every_valid_alignment_canaries_and_adjacent_message_are_preserved() {
    // This harness lends u64-backed storage. A future stronger Message
    // alignment must fail before any cast, even if the production wipe remains
    // valid for properly allocated Messages with that stronger alignment.
    assert_eq!(mem::align_of::<Message>(), mem::align_of::<u64>());
    let mut storage = [0x0bad_f00d_1234_5678u64; 64];
    for prefix in 1..16 {
        storage.fill(0x0bad_f00d_1234_5678);
        // SAFETY: u64 storage is aligned for Message (checked above), and this
        // 320-byte region fits with a nonempty canary prefix and suffix.
        let message = unsafe { storage.as_mut_ptr().add(prefix).cast::<Message>() };
        unsafe {
            message.write(marker(prefix as u64));
            snapshot_wipe::wipe_message(&mut *message);
            assert_erased(&*message);
        }
        assert!(storage[..prefix]
            .iter()
            .all(|word| *word == 0x0bad_f00d_1234_5678));
        assert!(storage[prefix + 40..]
            .iter()
            .all(|word| *word == 0x0bad_f00d_1234_5678));
    }
    let untouched = marker(99);
    let mut pair = [marker(7), untouched];
    snapshot_wipe::wipe_message(&mut pair[0]);
    assert_erased(&pair[0]);
    assert_eq!(pair[1], untouched);
}
#[test]
fn all_1024_snapshot_messages_are_erased_while_exclusive_lease_is_held() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let engine = Engine::new();
            let mut snapshot = AuthoritySnapshot::new();
            engine.snapshot_into(&mut snapshot);
            let lock = SnapshotLock::new();
            let lease = lock.try_claim().unwrap();
            let mut count = 0;
            for message in snapshot.messages_mut() {
                *message = marker(count);
                snapshot_wipe::wipe_message(message);
                assert_erased(message);
                assert!(lock.try_claim().is_none());
                count += 1;
            }
            assert_eq!(count, 1024);
            assert!(snapshot
                .messages_mut()
                .all(|message| *message == Message::new()));
            assert!(snapshot.matches(&engine));
            drop(lease);
            assert!(lock.try_claim().is_some());
        })
        .unwrap()
        .join()
        .unwrap();
}

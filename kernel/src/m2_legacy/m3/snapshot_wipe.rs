//! Observable erasure of an exclusively owned snapshot Message. No live Engine,
//! mapping, capability, hash, allocator or user pointer is involved.
use capability_core::{Cap, Message};
use core::{mem, ptr};

const WORDS: usize = 40;
const _: () = assert!(mem::size_of::<Cap>() == 2 * mem::size_of::<u64>());
const _: () = assert!(mem::size_of::<Message>() == WORDS * mem::size_of::<u64>());
const _: () = assert!(mem::align_of::<Message>() >= mem::align_of::<u64>());
// With the exact primitive schema below, this also rejects hidden padding.
const _: () = assert!(
    mem::size_of::<Message>()
        == 3 * mem::size_of::<u64>() + 2 * mem::size_of::<u32>() + 2 * mem::size_of::<Cap>() + 256
);

// Compile-time structural obligation for the unsafe all-zero representation.
// Copy alone would also admit NonZero values/references/enums. Exhaustive
// Message AND Cap patterns and exact primitive types must remain valid when
// either schema changes. This function pointer is never called or stored.
#[deny(unused_variables)]
const _: fn(&mut Message) = |Message {
                                 sender_pid,
                                 tag,
                                 len,
                                 cap_count,
                                 reserved,
                                 caps,
                                 data,
                             }| {
    // By-value type checks cannot silently use a wrapper's DerefMut coercion.
    let _: u64 = *sender_pid;
    let _: u64 = *tag;
    let _: u32 = *len;
    let _: u32 = *cap_count;
    let _: u64 = *reserved;
    let _: [u8; 256] = *data;
    let capabilities: [Cap; 2] = *caps;
    for Cap { handle, rights } in capabilities {
        let _: u64 = handle;
        let _: u64 = rights;
    }
};

#[inline(always)]
pub(super) fn wipe_message(message: &mut Message) {
    let words = ptr::from_mut(message).cast::<u64>();
    for index in 0..WORDS {
        // SAFETY: The unique initialized Message borrow covers all 320 bytes.
        // Its alignment and complete size are checked above. Every index is
        // 0..40, so each aligned eight-byte write stays within this object.
        // The exhaustive primitive schema proves all-zero is valid, including
        // during these partial writes. No padding or old content is read; any
        // padding is initialized too. There are no aliases, callbacks or reads
        // until this synchronous wipe finishes. Volatile stores preserve the
        // observable erase before the caller may release its snapshot lease.
        unsafe {
            words.add(index).write_volatile(0);
        }
    }
}

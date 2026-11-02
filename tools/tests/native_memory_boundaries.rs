//! Standalone Linux/x86-64 test of the actual kernel intrinsics, including
//! inaccessible guard pages. No host disk/device access or external crates.
#![deny(unsafe_op_in_unsafe_fn)]
#[path = "../../kernel/src/memory.rs"]
mod memory;
use core::{arch::asm, ptr};
use std::ffi::c_void;

unsafe extern "C" {
    fn mmap(
        address: *mut c_void,
        length: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: isize,
    ) -> *mut c_void;
    fn mprotect(address: *mut c_void, length: usize, prot: i32) -> i32;
    fn munmap(address: *mut c_void, length: usize) -> i32;
}
struct Guarded {
    base: *mut u8,
}
impl Guarded {
    fn new() -> Self {
        // SAFETY: Anonymous private host-test allocation; only the middle page
        // is writable. Both adjacent pages deliberately remain inaccessible.
        let base = unsafe { mmap(ptr::null_mut(), 3 * 4096, 0, 2 | 0x20, -1, 0) };
        assert_ne!(base as usize, usize::MAX);
        assert_eq!(
            unsafe { mprotect(base.cast::<u8>().add(4096).cast(), 4096, 3) },
            0
        );
        Self { base: base.cast() }
    }
    fn start(&self) -> *mut u8 {
        unsafe { self.base.add(4096) }
    }
    fn end(&self) -> *mut u8 {
        unsafe { self.base.add(8192) }
    }
}
impl Drop for Guarded {
    fn drop(&mut self) {
        assert_eq!(unsafe { munmap(self.base.cast(), 3 * 4096) }, 0);
    }
}
fn fill(p: *mut u8, n: usize, seed: usize) {
    for i in 0..n {
        unsafe {
            p.add(i).write_volatile((i.wrapping_mul(37) ^ seed) as u8);
        }
    }
}
fn equal(a: *const u8, b: *const u8, n: usize) {
    for i in 0..n {
        assert_eq!(
            unsafe { a.add(i).read_volatile() },
            unsafe { b.add(i).read_volatile() },
            "byte {i} length {n}"
        );
    }
}
fn oracle(a: &[u8], b: &[u8]) -> i32 {
    for (&x, &y) in a.iter().zip(b) {
        if x != y {
            return i32::from(x) - i32::from(y);
        }
    }
    0
}
#[test]
fn copy_all_alignments_lengths_canaries_and_original_return_pointer() {
    let mut source = [0; 1048];
    let mut output = [0; 1048];
    fill(source.as_mut_ptr(), source.len(), 11);
    for sa in 0..8 {
        for da in 0..8 {
            for n in 0..=1025 {
                output.fill(0xa5);
                let src = unsafe { source.as_ptr().add(8 + sa) };
                let dst = unsafe { output.as_mut_ptr().add(8 + da) };
                assert_eq!(unsafe { memory::memcpy(dst, src, n) }, dst);
                equal(src, dst, n);
                assert!(output[..8 + da].iter().all(|&x| x == 0xa5));
                assert!(output[8 + da + n..].iter().all(|&x| x == 0xa5));
                assert_eq!(unsafe { memory::memcmp(src, dst, n) }, 0);
            }
        }
    }
}
#[test]
fn guard_pages_exact_ends_starts_empty_inaccessible_and_read_only_source() {
    let a = Guarded::new();
    let b = Guarded::new();
    fill(a.start(), 4096, 51);
    assert_eq!(unsafe { mprotect(a.start().cast(), 4096, 1) }, 0);
    for n in 0..=1025 {
        let src = unsafe { a.end().sub(n) };
        let dst = unsafe { b.end().sub(n) };
        unsafe {
            memory::memcpy(dst, src, n);
        }
        equal(src, dst, n);
        assert_eq!(unsafe { memory::memcmp(src, dst, n) }, 0);
        assert_eq!(unsafe { memory::memcmp(dst, src, n) }, 0);
        for alignment in 0..8 {
            let other = unsafe { b.start().add(alignment) };
            unsafe {
                memory::memcpy(other, src, n);
            }
            equal(src, other, n);
            assert_eq!(unsafe { memory::memcmp(src, other, n) }, 0);
            assert_eq!(unsafe { memory::memcmp(other, src, n) }, 0);
        }
    }
    // Both arguments are genuinely inaccessible. Zero-length must not load or
    // store, and can never form an unconditional full-word access.
    assert_eq!(unsafe { memory::memcpy(b.end(), a.end(), 0) }, b.end());
    assert_eq!(unsafe { memory::memcmp(a.end(), b.end(), 0) }, 0);
    unsafe {
        memory::memcpy(b.start(), a.start(), 1025);
    }
    equal(a.start(), b.start(), 1025);
}
#[test]
fn comparison_first_byte_unsigned_lexicographic_and_word_tail_boundaries() {
    let mut a = [0; 1048];
    let mut b = [0; 1048];
    for aa in 0..8 {
        for ba in 0..8 {
            for n in [1, 2, 7, 8, 9, 15, 16, 17, 31, 32, 33, 255, 256, 257, 1025] {
                for first in 0..n {
                    for (x, y) in [(0x80, 0x7f), (0x00, 0xff), (0xff, 0), (1, 2)] {
                        a.fill(0x55);
                        b.fill(0x55);
                        a[aa + first] = x;
                        b[ba + first] = y;
                        if first + 1 < n {
                            a[aa + first + 1] = y;
                            b[ba + first + 1] = x;
                        }
                        let expected = oracle(&a[aa..aa + n], &b[ba..ba + n]);
                        let got =
                            unsafe { memory::memcmp(a.as_ptr().add(aa), b.as_ptr().add(ba), n) };
                        assert_eq!(got, expected, "align {aa}/{ba} n={n} first={first}");
                    }
                }
            }
        }
    }
}
#[test]
fn copy_compare_df_is_cleared_before_return_even_from_deliberate_std_trampoline() {
    let source = [0x73u8; 40];
    let mut output = [0; 40];
    for n in [0, 1, 7, 8, 9, 32, 33] {
        let (flags, returned): (u64, *mut u8);
        // SAFETY: All pointers/ranges valid. STD, the deliberate non-ABI entry,
        // CALL, flag capture and CLD stay in ONE asm block; Rust only resumes
        // with DF=0. Explicit C ABI clobbers model all call-modified registers.
        unsafe {
            asm!("std", "call {routine}", "pushfq", "pop r8", "cld",
                routine = sym memory::memcpy,
                in("rdi") output.as_mut_ptr(), in("rsi") source.as_ptr(), in("rdx") n,
                lateout("rax") returned, lateout("r8") flags, clobber_abi("C"));
        }
        assert_eq!(returned, output.as_mut_ptr());
        assert_eq!(flags & (1 << 10), 0);
        equal(source.as_ptr(), output.as_ptr(), n);
        let (flags, compared): (u64, i32);
        unsafe {
            asm!("std", "call {routine}", "pushfq", "pop r8", "cld",
                routine = sym memory::memcmp,
                in("rdi") source.as_ptr(), in("rsi") source.as_ptr(), in("rdx") n,
                lateout("eax") compared, lateout("r8") flags, clobber_abi("C"));
        }
        assert_eq!(compared, 0);
        assert_eq!(flags & (1 << 10), 0);
    }
}
#[test]
fn existing_memmove_overlap_behavior_is_preserved() {
    for n in 0..=128 {
        for gap in 0..16 {
            for backwards in [false, true] {
                let mut actual = [0u8; 160];
                fill(actual.as_mut_ptr(), actual.len(), 3);
                let before = actual;
                let (from, to) = if backwards { (0, gap) } else { (gap, 0) };
                let returned = unsafe {
                    memory::memmove(actual.as_mut_ptr().add(to), actual.as_ptr().add(from), n)
                };
                assert_eq!(returned, unsafe { actual.as_mut_ptr().add(to) });
                for i in 0..actual.len() {
                    let expected = if (to..to + n).contains(&i) {
                        before[from + i - to]
                    } else {
                        before[i]
                    };
                    assert_eq!(actual[i], expected);
                }
            }
        }
    }
}

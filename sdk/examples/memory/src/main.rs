#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Actual ring-3 SDK growth and maintained allocator. The harness never supplies
//! application heap, allocation success, returned bytes or generated answers.
extern crate alloc;

mod contract;
#[path = "../../system/src/support.rs"]
mod support;

use alloc::{boxed::Box, string::String, vec::Vec};
use contract::*;
use core::{alloc::Layout, arch::asm, arch::global_asm, panic::PanicInfo};
use nieuw_os_heap_allocator::{AllocError, Arena, UserAllocator};
use nieuw_os_sdk_runtime::{memory, native, Error};

#[global_allocator]
static ALLOCATOR: UserAllocator = UserAllocator::new();

global_asm!(
    r#"
.section .text.entry,"ax",@progbits
.global _start
_start:
    cld
    // The kernel enters at STACK_END-8, exactly the SysV C-entry alignment.
    // Tail-jump to the noreturn C function: an extra CALL would shift RSP by8.
    mov rax, rsp
    and eax, 15
    cmp eax, 8
    jne .Lmemory_bad_entry_alignment
    jmp memory_main
.Lmemory_bad_entry_alignment:
    ud2
.section .text.memory_faults,"ax",@progbits
.global memory_guard_fault
memory_guard_fault:
    mov byte ptr [rdi], 1
    ud2
.global memory_peer_fault
memory_peer_fault:
    mov rax, qword ptr [rdi]
    ud2
"#
);
extern "C" {
    fn memory_guard_fault(address: u64) -> !;
    fn memory_peer_fault(address: u64) -> !;
}

fn fail(step: u64) -> ! {
    support::put(FAILURE, step);
    native::exit(1)
}
fn check(ok: bool, step: u64) {
    if !ok {
        fail(step);
    }
}
fn bump(field: u64, amount: u64) {
    support::put(
        field,
        support::word(field)
            .checked_add(amount)
            .unwrap_or_else(|| fail(90)),
    );
}
fn heartbeat() {
    bump(HEARTBEAT, 1);
    native::yield_now().unwrap_or_else(|_| fail(91));
}
fn settle_ticks() {
    let start = native::monotonic_ticks().unwrap_or_else(|_| fail(92));
    while native::monotonic_ticks()
        .unwrap_or_else(|_| fail(92))
        .saturating_sub(start)
        < 2
    {
        verify_private();
        heartbeat();
    }
}
fn pattern() -> u64 {
    support::word(16) ^ 0x7b5d_e031_9184_6621 ^ support::word(GENERATION)
}

/// Test-only raw register call for deliberately noncanonical Sys4 arguments.
/// The normal growth path below always uses the public typed SDK wrapper.
fn raw_growth(args: [u64; 6]) -> (i64, u64) {
    let mut rax = 4u64;
    let mut rdx = args[2];
    // SAFETY: Scalar int80 ABI1.7 preserves other GPRs; no reference or unchecked
    // pointer is supplied, and no nomem flag hides kernel-visible writes.
    unsafe {
        asm!("int 0x80",inout("rax") rax,in("rdi") args[0],in("rsi") args[1],
            inout("rdx") rdx,in("r10") args[3],in("r8") args[4],in("r9") args[5],options(nostack));
    }
    (rax as i64, rdx)
}
fn argument_errors() {
    for args in [[0; 6], [u64::MAX, 0, 0, 0, 0, 0]] {
        check(raw_growth(args) == (-1, 0), 10);
        bump(ARG_DENIED, 1);
    }
    for index in 1..6 {
        let mut args = [0; 6];
        args[0] = 1;
        args[index] = 1;
        check(raw_growth(args) == (-1, 0), 11);
        bump(ARG_DENIED, 1);
    }
}
fn inspect_new(block: &memory::OwnedHeapPages, expected: u64) {
    check(block.base_address() == expected, 20);
    for offset in (0..block.len_bytes()).step_by(8) {
        // SAFETY: Successful SDK extent is exclusively owned, aligned, mapped
        // for this lifetime and not yet transferred to the allocator.
        check(
            unsafe {
                block
                    .as_ptr()
                    .add(offset as usize)
                    .cast::<u64>()
                    .read_volatile()
            } == 0,
            21,
        );
    }
    bump(ZERO_BYTES, block.len_bytes());
    bump(PAGES, block.page_count());
    for offset in (0..block.len_bytes()).step_by(memory::PAGE_BYTES as usize) {
        // SAFETY: Same exclusively owned fresh page, outside any allocator.
        unsafe {
            block
                .as_ptr()
                .add(offset as usize)
                .cast::<u64>()
                .write_volatile(pattern());
        }
    }
    support::put(SUFFIX_END, expected + block.len_bytes());
}
fn collections(block: memory::OwnedHeapPages) {
    let arena =
        Arena::checked(block.as_ptr(), block.len_bytes() as usize).unwrap_or_else(|_| fail(30));
    // SAFETY: Exclusive fresh mapped private RW/NX extent, no retained Rust
    // references or aliases. Ownership transfers for the entire process life.
    unsafe {
        ALLOCATOR.initialize_raw(arena).unwrap_or_else(|_| fail(31));
    }
    {
        let mut numbers = Vec::new();
        numbers
            .try_reserve_exact(16384)
            .unwrap_or_else(|_| fail(32));
        for i in 0..16384u64 {
            numbers.push(i ^ pattern());
        }
        check(
            numbers
                .iter()
                .enumerate()
                .all(|(i, x)| *x == (i as u64 ^ pattern())),
            33,
        );
        let object = Box::new([pattern(); 128]);
        check(object.iter().all(|x| *x == pattern()), 34);
        let mut text = String::new();
        text.try_reserve(4096).unwrap_or_else(|_| fail(35));
        for _ in 0..128 {
            text.push_str("Nieuw OS eigen geheugen. ");
        }
        check(text.len() == 3200 && text.starts_with("Nieuw OS"), 36);
        // Ordinary allocations must actually inhabit the newly grown extent.
        for address in [
            numbers.as_ptr() as u64,
            object.as_ptr() as u64,
            text.as_ptr() as u64,
        ] {
            check(
                (block.base_address()..block.base_address() + block.len_bytes()).contains(&address),
                37,
            );
        }
        heartbeat();
        check(
            numbers[16001] == (16001 ^ pattern()) && object[127] == pattern(),
            38,
        );
    }
    check(ALLOCATOR.usage().live_allocations == 0, 39);
    let retained_layout = Layout::from_size_align(64, 8).unwrap();
    let retained = ALLOCATOR
        .try_allocate(retained_layout)
        .unwrap_or_else(|_| fail(42));
    // SAFETY: Fresh private allocator block; this test owns its full extent.
    unsafe { retained.as_ptr().write_bytes(0x5d, retained_layout.size()) };
    let held_layout = Layout::from_size_align(64 * 1024, 8).unwrap();
    let held = ALLOCATOR
        .try_allocate(held_layout)
        .unwrap_or_else(|_| fail(43));
    let request_bytes = ALLOCATOR
        .usage()
        .engine_free
        .checked_add(32 * 1024)
        .unwrap_or_else(|| fail(44));
    let request = Layout::from_size_align(request_bytes, 8).unwrap_or_else(|_| fail(45));
    let mut reclaim_attempts = 0;
    let recovered = ALLOCATOR
        .try_allocate_with_reclaim_once(request, || {
            reclaim_attempts += 1;
            // SAFETY: This callback runs only after a failed allocation and
            // releases this exact live block once, outside the allocator lock.
            unsafe { ALLOCATOR.deallocate(held, held_layout) };
        })
        .unwrap_or_else(|_| fail(46));
    check(reclaim_attempts == 1, 47);
    check(
        unsafe {
            core::slice::from_raw_parts(retained.as_ptr(), retained_layout.size())
        }
        .iter()
        .all(|byte| *byte == 0x5d),
        48,
    );
    check(
        (block.base_address()..block.base_address() + block.len_bytes())
            .contains(&(recovered.as_ptr() as u64)),
        49,
    );
    unsafe {
        ALLOCATOR.deallocate(recovered, request);
        ALLOCATOR.deallocate(retained, retained_layout);
    }
    check(ALLOCATOR.usage().live_allocations == 0, 56);
    check(
        matches!(
            ALLOCATOR.try_allocate(Layout::from_size_align(16 * 1024 * 1024, 8).unwrap()),
            Err(AllocError::OutOfMemory)
        ),
        40,
    );
    check(ALLOCATOR.usage().live_allocations == 0, 41);
    support::put(COLLECTIONS, 1);
    support::put(ALLOC_OOM, 1);
}
fn grow_checked(pages: u64) -> memory::OwnedHeapPages {
    let expected = support::word(SUFFIX_END);
    let block = native::heap_grow(pages).unwrap_or_else(|_| fail(22));
    inspect_new(&block, expected);
    block
}
fn normal_growth() {
    argument_errors();
    collections(grow_checked(513));
    check(
        matches!(
            native::heap_grow(memory::MAX_PROCESS_PAGES),
            Err(Error::Kernel(-7))
        ),
        50,
    );
    bump(QUOTA_DENIED, 1);
    let _next = grow_checked(1);
    let initial = support::word(INITIAL_MAPPINGS);
    let remaining = memory::MAX_PROCESS_PAGES
        .checked_sub(initial + support::word(PAGES))
        .unwrap_or_else(|| fail(51));
    check(remaining > 0, 52);
    let _last = grow_checked(remaining);
    check(matches!(native::heap_grow(1), Err(Error::Kernel(-7))), 53);
    bump(QUOTA_DENIED, 1);
    verify_private();
    settle_ticks();
    support::put(DONE, 1);
}
fn verify_private() {
    // SAFETY: Initial private heap never moved or changed ownership, and is not
    // used by the allocator. Distinct process pattern detects accidental sharing.
    check(
        unsafe { (memory::HEAP_BASE as *const u64).read_volatile() } == pattern(),
        54,
    );
    // The page immediately following the allocator arena is independently kept
    // as raw private storage; the allocator cannot overwrite this sentinel.
    let after_allocator = memory::HEAP_BASE
        + memory::INITIAL_HEAP_BYTES
        + if support::word(CASE) == 4 { 64 } else { 513 } * memory::PAGE_BYTES;
    if support::word(SUFFIX_END) > after_allocator {
        // SAFETY: This address was returned by a later successful heap growth.
        check(
            unsafe { (after_allocator as *const u64).read_volatile() } == pattern(),
            55,
        );
    }
    support::put(PRIVATE, 1);
}
fn oom_growth() {
    argument_errors();
    let mut initialized = false;
    loop {
        let next = support::word(SUFFIX_END);
        match native::heap_grow(64) {
            Ok(block) => {
                inspect_new(&block, next);
                if !initialized {
                    collections(block);
                    initialized = true;
                }
                verify_private();
                heartbeat();
            }
            Err(Error::Kernel(-7)) => {
                bump(QUOTA_DENIED, 1);
                break;
            }
            Err(Error::Kernel(memory::OUT_OF_MEMORY)) => {
                support::put(OOM, 1);
                break;
            }
            _ => fail(60),
        }
    }
    check(initialized, 61);
    verify_private();
    settle_ticks();
    support::put(DONE, 1);
    loop {
        match support::word(CONTROL) {
            1 => native::exit(0),
            3 => {
                check(support::word(OOM) == 1, 62);
                let old_end = support::word(SUFFIX_END);
                let block = grow_checked(1);
                support::put(RETRY_BASE, block.base_address());
                support::put(RETRY_ZERO_BYTES, block.len_bytes());
                check(
                    block.base_address() == old_end && block.len_bytes() == memory::PAGE_BYTES,
                    63,
                );
                support::put(RETRY, 1);
                verify_private();
                native::exit(0)
            }
            _ => heartbeat(),
        }
    }
}

#[no_mangle]
pub extern "C" fn memory_main() -> ! {
    check(native::abi_info() == Ok((1 << 32) | 7), 1);
    let generation = support::word(GENERATION);
    check((1..=2).contains(&generation), 2);
    check(
        support::word(SLOT) < if support::word(CASE) == 4 { 16 } else { 2 },
        4,
    );
    // SAFETY: Initial kernel-supplied private heap is mapped zeroed and unused.
    for offset in (0..memory::INITIAL_HEAP_BYTES).step_by(8) {
        check(
            unsafe { ((memory::HEAP_BASE + offset) as *const u64).read_volatile() } == 0,
            3,
        );
    }
    support::put(CLEAN, 1);
    support::put(STARTUP_PAGES, support::word(INITIAL_MAPPINGS));
    support::put(PATTERN, pattern());
    support::put(SUFFIX_END, memory::HEAP_BASE + memory::INITIAL_HEAP_BYTES);
    // SAFETY: Retain a sentinel in the existing heap, outside the new allocator.
    unsafe {
        (memory::HEAP_BASE as *mut u64).write_volatile(pattern());
    }
    if support::word(CASE) == 4 {
        oom_growth();
    }
    normal_growth();
    loop {
        verify_private();
        match support::word(CONTROL) {
            1 => native::exit(0),
            2 => {
                let target = support::word(TARGET);
                match support::word(CASE) {
                    1 => {
                        // SAFETY: Deliberate native NX negative test: target is
                        // this app's new RW/NX page. A real CPU #PF is required.
                        unsafe {
                            asm!("jmp rax",in("rax") target,options(noreturn));
                        }
                    }
                    2 => {
                        support::put(FAULT_RIP, memory_guard_fault as *const () as u64);
                        // SAFETY: Explicit fault fixture, root supplies the first
                        // unmapped suffix page. The process must be terminated.
                        unsafe {
                            memory_guard_fault(target);
                        }
                    }
                    3 => {
                        support::put(FAULT_RIP, memory_peer_fault as *const () as u64);
                        // SAFETY: Explicit isolation negative: a live peer's
                        // physical page is supervisor-only, never app authority.
                        unsafe {
                            memory_peer_fault(target);
                        }
                    }
                    _ => fail(70),
                }
            }
            _ => heartbeat(),
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    fail(99)
}

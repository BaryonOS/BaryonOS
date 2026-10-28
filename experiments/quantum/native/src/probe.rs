//! Synthetic ring-3 port probe. The kernel remains owner of the native verdict.
#![no_std]
#![no_main]

use core::{
    alloc::{GlobalAlloc, Layout},
    arch::{asm, global_asm},
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};
use qsim_native::{execute, Abort, Control, Error, ExecuteError, Gate, HEAP_BYTES};

const DATA: usize = 0x4001_0000;
const HEAP: usize = 0x6000_0000;

global_asm!(
    r#"
    .section .qsim.header,"aw",@progbits
    .balign 8
    .global qsim_startup
qsim_startup:
    .zero 512
"#
);

struct ProcessArena(AtomicUsize);

// SAFETY: The existing single-thread process contract maps the full private
// 64-KiB heap RW/NX. Checked monotonically increasing offsets never overlap;
// base alignment is 64 KiB. No allocation is reset or reused during this
// finite probe. Deallocation intentionally retains bytes until process exit.
unsafe impl GlobalAlloc for ProcessArena {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut current = self.0.load(Ordering::Relaxed);
        loop {
            let Some((start, end)) = qsim_native::heap_budget::allocation_range(
                current,
                layout.size(),
                layout.align(),
                HEAP_BYTES,
            ) else {
                return ptr::null_mut();
            };
            match self
                .0
                .compare_exchange_weak(current, end, Ordering::Relaxed, Ordering::Relaxed)
            {
                Ok(_) => return (HEAP + start) as *mut u8,
                Err(actual) => current = actual,
            }
        }
    }

    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {}
}

#[global_allocator]
static ARENA: ProcessArena = ProcessArena(AtomicUsize::new(0));

fn read(offset: usize) -> u64 {
    // SAFETY: Fixed aligned offsets in our initialized private 512-byte report.
    unsafe { ((DATA + offset) as *const u64).read_volatile() }
}
fn write(offset: usize, value: u64) {
    // SAFETY: Same mapped private report, no retained reference or shared page.
    unsafe { ((DATA + offset) as *mut u64).write_volatile(value) }
}
fn syscall(number: u64) -> (i64, u64) {
    let mut primary = number;
    let secondary: u64;
    // SAFETY: Existing int80 ABI; only abi_info/yield are used, with all
    // unused arguments zero. The kernel sanitizes flags and preserves GPRs.
    unsafe {
        asm!("int 0x80", inout("rax") primary, in("rdi") 0_u64,
            in("rsi") 0_u64, inout("rdx") 0_u64 => secondary,
            in("r10") 0_u64, in("r8") 0_u64, in("r9") 0_u64,
            options(nostack));
    }
    (primary as i64, secondary)
}
fn yield_now() {
    if syscall(1) != (0, 0) {
        fail(1);
    }
}
fn worker() -> ! {
    loop {
        yield_now();
    }
}
fn fail(code: u64) -> ! {
    write(32, 0x100 + code);
    // Avoid recursively invoking a failed yield wrapper.
    loop {
        let _ = syscall(1);
    }
}

struct NativeControl {
    calls: u64,
    previous: u64,
    gap: u64,
    stop: Option<(u64, Abort)>,
}
impl NativeControl {
    const fn new(stop: Option<(u64, Abort)>) -> Self {
        Self {
            calls: 0,
            previous: 0,
            gap: 0,
            stop,
        }
    }
}
impl Control for NativeControl {
    fn checkpoint(&mut self, work: u64) -> Result<(), Abort> {
        if work < self.previous {
            fail(2);
        }
        self.gap = self.gap.max(work - self.previous);
        self.previous = work;
        self.calls += 1;
        write(8, read(8) + 1);
        write(80, read(80).max(self.gap));
        if let Some((stop, reason)) = self.stop {
            if self.calls >= stop {
                return Err(reason);
            }
        }
        yield_now();
        Ok(())
    }
}

fn passed(bit: u64) {
    write(40, read(40) | bit);
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    let pattern = read(16);
    let (status, abi) = syscall(0);
    if status != 0 || abi >> 32 != 1 || read(0) != 0 {
        fail(3);
    }
    for offset in (32..128).step_by(8) {
        write(offset, 0);
    }
    let bell = [
        Gate::H { target: 0 },
        Gate::Cx {
            control: 0,
            target: 1,
        },
    ];
    let result =
        execute(2, &bell, 4096, 0, &mut NativeControl::new(None)).unwrap_or_else(|_| fail(4));
    if result.bins != 4
        || result.shots_completed != 4096
        || result.counts[..4] != [2082, 0, 0, 2014]
        || result.counts[4..].iter().any(|count| *count != 0)
    {
        fail(5);
    }
    write(48, u64::from(result.counts[0]));
    write(56, u64::from(result.counts[3]));
    write(64, u64::from(result.shots_completed));
    passed(1);

    let six = core::array::from_fn::<_, 6, _>(|target| Gate::X {
        target: target as u64,
    });
    let result =
        execute(6, &six, 4096, u64::MAX, &mut NativeControl::new(None)).unwrap_or_else(|_| fail(6));
    if result.bins != 64
        || result.counts[63] != 4096
        || result.counts[..63].iter().any(|count| *count != 0)
    {
        fail(7);
    }
    write(88, u64::from(result.counts[63]));
    passed(2);

    let gates = [Gate::H { target: 0 }; 64];
    for (stop, reason, bit) in [
        (1, Abort::Cancelled, 4),
        (8, Abort::Cancelled, 8),
        (80, Abort::DeadlineExceeded, 16),
    ] {
        let mut control = NativeControl::new(Some((stop, reason)));
        if execute(6, &gates, 4096, 0, &mut control)
            != Err(Error::Execution(ExecuteError::Aborted(reason)))
            || control.calls != stop
        {
            fail(8);
        }
        passed(bit);
    }

    let mut control = NativeControl::new(None);
    if execute(7, &[], 1, 0, &mut control) != Err(Error::QubitLimit)
        || execute(1, &[Gate::X { target: 0 }; 65], 1, 0, &mut control) != Err(Error::GateLimit)
        || execute(1, &[], 4097, 0, &mut control) != Err(Error::ShotLimit)
        || !matches!(
            execute(6, &[Gate::X { target: 6 }], 1, 0, &mut control),
            Err(Error::Invalid(_))
        )
        || control.calls != 0
    {
        fail(9);
    }
    passed(32);

    let used = ARENA.0.load(Ordering::Relaxed);
    if used == 0 || used > HEAP_BYTES {
        fail(10);
    }
    // SAFETY: A bounded negative allocation request. Existing allocations use
    // nonzero arena space, so requesting all 64 KiB must return null unchanged.
    let denied = unsafe { ARENA.alloc(Layout::from_size_align(HEAP_BYTES, 8).unwrap()) };
    if !denied.is_null() || ARENA.0.load(Ordering::Relaxed) != used {
        fail(11);
    }
    write(72, used as u64);
    // SAFETY: Reserve the remaining private arena without dereferencing it.
    // Nothing is freed/reset; a subsequent core allocation must fail cleanly.
    let rest = unsafe { ARENA.alloc(Layout::from_size_align(HEAP_BYTES - used, 1).unwrap()) };
    if rest != (HEAP + used) as *mut u8
        || ARENA.0.load(Ordering::Relaxed) != HEAP_BYTES
        || execute(1, &[], 1, 0, &mut NativeControl::new(None))
            != Err(Error::Execution(ExecuteError::ResourceExhausted))
    {
        fail(13);
    }
    write(96, ARENA.0.load(Ordering::Relaxed) as u64);
    write(104, 1);
    passed(64);
    if read(16) != pattern || read(80) > 64 * 7 || read(40) != 127 {
        fail(12);
    }
    write(32, 1);
    worker()
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    fail(0xff)
}

// Compiler intrinsics: exact scalar byte ranges, no host libc or SIMD.
#[no_mangle]
unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, size: usize) -> *mut u8 {
    // SAFETY: C caller provides readable/writable nonoverlapping `size` bytes.
    unsafe {
        asm!("cld", "rep movsb", inout("rdi") dst => _, inout("rsi") src => _,
            inout("rcx") size => _, options(nostack));
    }
    dst
}

#[no_mangle]
unsafe extern "C" fn memset(dst: *mut u8, byte: i32, size: usize) -> *mut u8 {
    // SAFETY: C caller provides writable `size` bytes; no alignment assumption.
    unsafe {
        asm!("cld", "rep stosb", inout("rdi") dst => _, in("al") byte as u8,
            inout("rcx") size => _, options(nostack));
    }
    dst
}

#[no_mangle]
unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, size: usize) -> *mut u8 {
    if dst as usize <= src as usize {
        // SAFETY: Forward copying preserves overlapping ranges in this order.
        unsafe {
            asm!("cld", "rep movsb", inout("rdi") dst => _, inout("rsi") src => _,
                inout("rcx") size => _, options(nostack));
        }
    } else {
        for offset in (0..size).rev() {
            // SAFETY: Caller provides valid ranges; descending copies preserve
            // overlap without setting DF across a scheduler interrupt.
            unsafe {
                dst.add(offset)
                    .write_volatile(src.add(offset).read_volatile())
            };
        }
    }
    dst
}

#[no_mangle]
unsafe extern "C" fn memcmp(a: *const u8, b: *const u8, size: usize) -> i32 {
    for offset in 0..size {
        // SAFETY: C caller provides both readable ranges. Volatile reads avoid
        // an optimizer rewrite into a recursively invoked memcmp intrinsic.
        let (left, right) =
            unsafe { (a.add(offset).read_volatile(), b.add(offset).read_volatile()) };
        if left != right {
            return i32::from(left) - i32::from(right);
        }
    }
    0
}

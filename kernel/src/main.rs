#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(feature = "framebuffer")]
mod framebuffer;
#[cfg(feature = "graphics-g0")]
mod graphics;
#[cfg(feature = "milestone-m2")]
mod m2;
mod memory;
mod serial;
#[cfg(all(feature = "graphics-g0", feature = "milestone-m3"))]
compile_error!("graphics-g0 is an IF0 bootstrap, not a ring-3/compositor profile");
#[cfg(all(
    feature = "native-rtc",
    any(feature = "milestone-m3", feature = "framebuffer")
))]
compile_error!("native-rtc exclusively owns CMOS in the single-CPU IF0 bootstrap");
#[cfg(all(
    feature = "native-smp",
    any(
        all(feature = "milestone-m3", not(feature = "native-smp-m3")),
        feature = "framebuffer",
        feature = "native-rtc",
        feature = "graphics-g0"
    )
))]
compile_error!("native-smp is an IF0 M2 bring-up profile; single-CPU subsystems stay excluded");
// Service profiles remain single-CPU unless an explicit SMP fixture admits them.
#[cfg(all(
    feature = "native-smp-m3",
    any(
        feature = "native-fpu",
        all(
            any(feature = "ipc-wait", feature = "milestone-m4"),
            not(any(feature = "native-smp-m3-ipc", feature = "native-smp-m4-quotas"))
        ),
        feature = "native-supervisor"
    )
))]
compile_error!("SMP services require an explicit integration profile");

use boot_protocol::{BootInfo, ADDRESS_LIMIT, PAGE_SIZE};
use core::{arch::asm, ptr};

static mut BSS_SENTINEL: [u64; 8] = [0; 8];
static mut DATA_SENTINEL: u64 = 0x4e4f_534b_4552_4e31;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "sysv64" fn _start(info_pointer: *const BootInfo) -> ! {
    serial::init();
    serial::write(b"KERNEL:ENTRY\n");
    let address = info_pointer as u64;
    if address == 0 || !address.is_multiple_of(PAGE_SIZE) || address >= ADDRESS_LIMIT {
        serial::fail(b"BOOTINFO_POINTER");
    }
    // SAFETY: Loader-to-kernel ABI guarantees this page is mapped and initialized;
    // range checks precede the copy. M1 trusts its own loader/firmware, not user pointers.
    let info = unsafe { info_pointer.read() };
    #[cfg(feature = "framebuffer")]
    let valid_header = info.validate_graphics_header(address);
    #[cfg(not(feature = "framebuffer"))]
    let valid_header = info.validate_header(address);
    if valid_header.is_err() {
        serial::fail(b"BOOTINFO_HEADER");
    }
    #[cfg(feature = "framebuffer")]
    framebuffer::capture(&info);
    let stack: u64;
    let flags: u64;
    // SAFETY: CPL0; read current stack and flags without changing them. Stack is
    // the dedicated mapped allocation installed by the assembly handoff.
    unsafe {
        asm!("mov {}, rsp", out(reg) stack, options(nomem, nostack, preserves_flags));
    }
    // SAFETY: Own writable stack has sufficient space; push/pop balances RSP.
    unsafe {
        asm!("pushfq", "pop {}", out(reg) flags, options(preserves_flags));
    }
    if stack < info.stack_start
        || stack >= info.stack_start + info.stack_len
        || flags & (1 << 9) != 0
    {
        serial::fail(b"CPU_STATE");
    }
    // SAFETY: Header bounds/overlap validation established a separate retained
    // page allocation under 1 GiB, len <= 1 MiB. Loader ABI supplies mapping/readability.
    // No firmware or allocator will mutate it after successful EBS.
    let map = unsafe {
        core::slice::from_raw_parts(
            info.memory_map_addr as *const u8,
            info.memory_map_len as usize,
        )
    };
    #[cfg(feature = "framebuffer")]
    let valid_map = boot_protocol::graphics::validate_graphics_memory_map(&info, map);
    #[cfg(not(feature = "framebuffer"))]
    let valid_map = boot_protocol::validate_memory_map(&info, map);
    if valid_map.is_err() {
        serial::fail(b"MEMORY_MAP");
    }
    for i in 0..8 {
        // SAFETY: Static in own loaded .bss, valid element index; volatile read
        // prevents compile-time replacement and actually verifies loader zeroing.
        if unsafe {
            ptr::addr_of!(BSS_SENTINEL)
                .cast::<u64>()
                .add(i)
                .read_volatile()
        } != 0
        {
            serial::fail(b"BSS_NOT_ZERO");
        }
    }
    // SAFETY: Static in own loaded .data; volatile read verifies file-backed initialization.
    if unsafe { ptr::addr_of!(DATA_SENTINEL).read_volatile() } != 0x4e4f_534b_4552_4e31 {
        serial::fail(b"DATA_INVALID");
    }
    serial::write(b"BOOTINFO:OK\n");
    #[cfg(feature = "test-panic")]
    panic!("test-only kernel panic");
    #[cfg(all(feature = "test-hang", not(feature = "test-panic")))]
    loop {
        core::hint::spin_loop();
    }
    #[cfg(not(any(feature = "test-panic", feature = "test-hang")))]
    {
        #[cfg(feature = "milestone-m2")]
        m2::start(info, map);
        #[cfg(not(feature = "milestone-m2"))]
        {
            serial::write(b"M1:PASS\n");
            serial::exit(0x10)
        }
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    #[cfg(feature = "milestone-m2")]
    if let Some(location) = info.location() {
        let mut digits = [b'0'; 10];
        let mut n = location.line();
        for digit in digits.iter_mut().rev() {
            *digit = b'0' + (n % 10) as u8;
            n /= 10;
        }
        // DG-2: a terminal line, whole and without waiting for SERIAL.
        let mut line = serial::Line::new();
        serial::write_terminal(
            line.push(b"M2:PANIC_LINE:")
                .push(&digits)
                .push(b"\n")
                .as_bytes(),
        );
    }
    #[cfg(not(feature = "milestone-m2"))]
    let _ = info;
    serial::fail(b"KERNEL_PANIC")
}

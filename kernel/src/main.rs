#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]

mod memory;
mod serial;
#[cfg(feature = "milestone-m2")]
mod m2;

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
    if info.validate_header(address).is_err() {
        serial::fail(b"BOOTINFO_HEADER");
    }
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
    if boot_protocol::validate_memory_map(&info, map).is_err() {
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
    #[cfg(feature="milestone-m2")]
    if let Some(location)=info.location() {
        let mut digits=[b'0';10];
        let mut n=location.line();
        for digit in digits.iter_mut().rev() { *digit=b'0'+(n%10) as u8; n/=10; }
        serial::write(b"M2:PANIC_LINE:"); serial::write(&digits); serial::write(b"\n");
    }
    #[cfg(not(feature="milestone-m2"))]
    let _=info;
    serial::fail(b"KERNEL_PANIC")
}

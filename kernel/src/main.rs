#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(feature = "native-dispi-probe")]
mod dispi;
#[cfg(feature = "framebuffer")]
#[cfg_attr(feature = "native-dispi-probe", allow(dead_code))]
mod framebuffer;
#[cfg(feature = "graphics-g0")]
#[cfg_attr(feature = "native-dispi-probe", allow(dead_code))]
mod graphics;
#[cfg(all(feature = "milestone-m2", not(feature = "native-smp-legacy")))]
mod m2;
#[cfg(feature = "native-smp-legacy")]
#[path = "m2_legacy/mod.rs"]
mod m2;
mod memory;
mod serial;
#[cfg(all(
    feature = "native-xhci-probe",
    any(
        feature = "milestone-m3",
        feature = "native-smp",
        feature = "native-smp-legacy",
        feature = "framebuffer",
        feature = "native-rtc",
        feature = "native-acpi-s5",
        feature = "native-acpi-s3"
    )
))]
compile_error!("native-xhci-probe exclusively owns the single-CPU IF0 DMA fixture");
#[cfg(all(feature = "test-xhci-dma-fault", feature = "test-xhci-no-dmar"))]
compile_error!("xHCI negative cases require separate boots");
#[cfg(all(
    feature = "native-usb-hid-probe",
    any(feature = "test-xhci-dma-fault", feature = "test-xhci-no-dmar")
))]
compile_error!("USB HID and legacy xHCI negative cases require separate boots");
#[cfg(all(
    feature = "native-usb-hid-reattach",
    feature = "test-usb-hid-readonly-fault"
))]
compile_error!("USB reattach and the read-only DMA fault require separate boots");
#[cfg(all(
    feature = "native-virtio-snd-probe",
    any(
        feature = "milestone-m3",
        feature = "framebuffer",
        feature = "native-smp",
        feature = "native-rtc",
        feature = "native-acpi-s5",
        feature = "native-acpi-s3",
        feature = "native-xhci-probe",
        feature = "native-dispi-probe"
    )
))]
compile_error!("native-virtio-snd-probe exclusively owns the single-CPU IF0 audio DMA fixture");
#[cfg(all(feature = "test-snd-fence-reinit", feature = "test-snd-quarantine"))]
compile_error!("virtio-snd fence cases require separate boots");
#[cfg(all(
    feature = "native-dispi-probe",
    any(
        feature = "milestone-m3",
        feature = "native-smp",
        feature = "native-smp-legacy",
        feature = "native-rtc",
        feature = "native-acpi-s5",
        feature = "native-xhci-probe",
        feature = "display-input"
    )
))]
compile_error!("native-dispi-probe exclusively owns the Bochs DISPI ports in the IF0 bootstrap");
#[cfg(all(feature = "graphics-g0", feature = "milestone-m3"))]
compile_error!("graphics-g0 is an IF0 bootstrap, not a ring-3/compositor profile");
#[cfg(all(
    feature = "native-rtc",
    any(feature = "milestone-m3", feature = "framebuffer")
))]
compile_error!("native-rtc exclusively owns CMOS in the single-CPU IF0 bootstrap");
#[cfg(all(
    feature = "native-acpi-s5",
    any(
        feature = "milestone-m3",
        feature = "framebuffer",
        feature = "native-smp",
        feature = "native-rtc"
    )
))]
compile_error!("native-acpi-s5 ends the single-CPU IF0 bootstrap; no other subsystem may run");
#[cfg(all(
    feature = "test-system-net-link",
    feature = "native-net-service",
    not(feature = "test-net-service-link")
))]
compile_error!("the link test module has a ring-3 service consumer only in test-net-service-link");
#[cfg(all(
    feature = "native-acpi-s3",
    any(
        feature = "milestone-m3",
        feature = "framebuffer",
        feature = "native-smp",
        feature = "native-rtc",
        feature = "native-acpi-s5",
        feature = "native-system-power"
    )
))]
compile_error!("native-acpi-s3 resumes only the single-CPU IF0 bootstrap; no device, AP or service state survives sleep");
#[cfg(all(
    feature = "native-system-power",
    any(
        feature = "native-acpi-s5",
        feature = "native-smp",
        feature = "framebuffer",
        feature = "native-system-net",
        feature = "native-system-random",
        feature = "native-system-images",
        feature = "native-system-clock",
        feature = "native-system-nas-probe",
        feature = "native-cap-digest-bench",
        feature = "test-system-recover"
    )
))]
compile_error!("native-system-power gates S5 on exactly the base native-system brokers");
#[cfg(all(
    feature = "test-system-s3",
    any(
        feature = "test-system-power-flush-error",
        feature = "test-system-power-flush-timeout",
        feature = "fpu-context",
        feature = "native-smp-m3",
        feature = "native-acpi-s3"
    )
))]
compile_error!("test-system-s3 suspends only the granted base native-system-power profile: no fault fixture, live FPU state or GS anchor survives the wake");
// Explicit, not only through the native-system-power guard: the wake resets
// every device, and only the block and entropy brokers refuse after it.
#[cfg(all(
    feature = "test-system-s3",
    any(
        feature = "native-system-net",
        feature = "native-system-clock",
        feature = "native-system-nas-probe",
        feature = "native-system-random",
        feature = "native-system-display-input",
        feature = "native-window-session"
    )
))]
compile_error!("test-system-s3 excludes the network, clock, NAS, random, display-input and window-session system profiles");
#[cfg(all(
    feature = "test-system-s3-held-completion",
    feature = "test-system-s3-reinit"
))]
compile_error!("the S3 held-completion and re-init fixtures require separate boots");
#[cfg(all(
    feature = "native-smp",
    not(feature = "native-smp-legacy"),
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
            not(any(
                feature = "native-smp-m3-ipc",
                feature = "native-smp-m4-quotas",
                feature = "native-smp-supervisor"
            ))
        ),
        all(feature = "native-supervisor", not(feature = "native-smp-supervisor"))
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
    #[cfg(all(feature = "framebuffer", feature = "native-smp-legacy"))]
    let info = {
        let mut header = info;
        // ABI 1.2 ends at byte 128, where its graphics extension starts.
        // Only the larger ABI 1.3 header carries these SMP fields.
        header.clear_absent_legacy_graphics_smp_fields();
        header
    };
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

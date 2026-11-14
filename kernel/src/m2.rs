//! M2 bootstrap owns all state; single CPU, IF clear, no allocator/scheduler locks.
//! Copies boot input before any frame reuse. The old loader allocations are retained.
mod acpi;
mod arch;
#[cfg(any(feature = "native-smp", feature = "milestone-m3"))]
pub(crate) mod lock;
#[cfg(feature = "milestone-m3")]
mod m3;
pub(crate) mod paging;
#[cfg(any(feature = "native-acpi-s5", feature = "native-system-power"))]
pub(crate) mod power;
#[cfg(any(feature = "native-rtc", feature = "native-system-clock"))]
mod rtc;
#[cfg(feature = "native-smp")]
pub(crate) mod smp;
use crate::serial;
use boot_protocol::{BootInfo, MemoryMap, MAP_LIMIT};
use core::{arch::asm, mem::MaybeUninit, ptr};
use kernel_core::{Frames, Kind, Owner, Range, PAGE, USER_QUOTA};
pub const FRAME_COUNT: usize = 65536; // 256-MiB fixed QEMU profile; higher RAM retained.
static mut FRAMES: Frames<FRAME_COUNT> = Frames::new();
static mut INFO: MaybeUninit<BootInfo> = MaybeUninit::uninit();
static mut MAP: [u8; MAP_LIMIT as usize] = [0; MAP_LIMIT as usize];
static mut QUOTA_FRAMES: [u64; USER_QUOTA] = [0; USER_QUOTA];
#[repr(align(4096))]
struct Probe([u8; 4096]);
static mut NX_PROBE: Probe = Probe([0xc3; 4096]);
extern "C" {
    static __kernel_start: u8;
    static __text_end: u8;
    static __rodata_end: u8;
    static __kernel_end: u8;
}
pub fn text_start() -> u64 {
    ptr::addr_of!(__kernel_start) as u64
}
pub fn nx_probe() -> u64 {
    // SAFETY: Forms only a raw address into a permanent static; no read, write,
    // reference or concurrent access is performed by this address calculation.
    unsafe { ptr::addr_of!(NX_PROBE.0) as u64 }
}

pub fn zero(address: u64) {
    assert_eq!(address % PAGE, 0);
    assert!(address != 0 && address < PAGE * FRAME_COUNT as u64);
    // SAFETY: Frame allocator supplies a validated normal ConventionalMemory frame
    // with exclusive Free/old owner and no aliases on free. Firmware identitymaps
    // candidate RAM until CR3 switch; post-switch caller must retain its mapping.
    // Raw writes do not create aliased references; exactly one whole aligned page.
    unsafe {
        ptr::write_bytes(address as *mut u8, 0, PAGE as usize);
    }
}
pub fn start(info: BootInfo, map: &[u8]) -> ! {
    // SAFETY: Single bootstrap owner. Static buffers are inside retained kernel
    // image, distinct from source boot allocations. Length was validated by M1.
    unsafe {
        ptr::addr_of_mut!(INFO).write(MaybeUninit::new(info));
        ptr::copy_nonoverlapping(map.as_ptr(), ptr::addr_of_mut!(MAP).cast::<u8>(), map.len());
        let stack = arch::stack_top();
        // Discard loader stack only after boot inputs have been copied. The own
        // stack is 16-KiB and both guards will be absent under the new CR3.
        asm!("mov rsp, {stack}","xor rbp,rbp","call {entry}",stack=in(reg) stack,entry=sym run,options(noreturn));
    }
}
extern "C" fn run() -> ! {
    arch::install();
    // SAFETY: INFO and MAP were initialized once before stack transfer and are
    // now immutable. FRAMES has this one exclusive bootstrap borrow, IF=0.
    let (info, map_bytes, frames) = unsafe {
        let info = (&*ptr::addr_of!(INFO)).assume_init_ref();
        (
            info,
            core::slice::from_raw_parts(
                ptr::addr_of!(MAP).cast::<u8>(),
                info.memory_map_len as usize,
            ),
            &mut *ptr::addr_of_mut!(FRAMES),
        )
    };
    #[cfg(feature = "framebuffer")]
    boot_protocol::graphics::validate_graphics_memory_map(info, map_bytes)
        .expect("copied graphics map");
    #[cfg(not(feature = "framebuffer"))]
    boot_protocol::validate_memory_map(info, map_bytes).expect("copied map");
    let map = MemoryMap::new(map_bytes, info.descriptor_size, info.descriptor_version).unwrap();
    let retained = info.retained_ranges().unwrap().map(|r| Range {
        start: r.start,
        end: r.end,
    });
    for i in 0..map.len() {
        let d = map.descriptor(i).unwrap();
        let range = Range::pages(d.physical_start, d.number_of_pages).expect("descriptor range");
        frames.admit(range, d.memory_type, d.attributes, &retained);
    }
    frames
        .retain_kernel(Range {
            start: info.kernel_reserved_start,
            end: info.kernel_reserved_start + info.kernel_reserved_len,
        })
        .unwrap();
    check_map_rejections();
    for range in &retained {
        for page in (range.start..range.end).step_by(PAGE as usize) {
            if page < PAGE * FRAME_COUNT as u64 {
                assert_ne!(frames.owner(page).unwrap(), Owner::Free);
            }
        }
    }
    serial::write(b"KS-01:PASS\n");
    check_frames(frames);
    serial::write(b"KS-02:PASS\n");
    // First-fit before any table allocation: the lowest free frame is the SIPI page.
    #[cfg(feature = "native-smp")]
    let trampoline = smp::reserve_trampoline(frames);
    let mut paging = paging::Paging::new(frames);
    let text_end = ptr::addr_of!(__text_end) as u64;
    let rodata_end = ptr::addr_of!(__rodata_end) as u64;
    let kernel_end = ptr::addr_of!(__kernel_end) as u64;
    for page in (text_start()..kernel_end).step_by(PAGE as usize) {
        #[cfg(feature = "native-smp")]
        if smp::is_guard(page) {
            continue;
        }
        if !arch::is_guard(page) {
            paging.map(frames, page, page, page >= rodata_end, page < text_end);
        }
    }
    for range in &retained[1..] {
        for page in (range.start..range.end).step_by(PAGE as usize) {
            paging.map(
                frames,
                page,
                page,
                page >= info.stack_start && page < info.stack_start + info.stack_len,
                false,
            );
        }
    }
    // Creates the shared scratch PT; `begin_scratch` unmaps it again.
    #[cfg(any(feature = "milestone-m3", feature = "native-smp"))]
    paging.map(frames, paging::SCRATCH, nx_probe(), true, false);
    #[cfg(feature = "framebuffer")]
    crate::framebuffer::map(&mut paging, frames, &map);
    // Needs the firmware identity map: may zero a new PT and writes the SIPI page.
    #[cfg(feature = "native-smp")]
    let smp_prepared = smp::prepare(frames, &mut paging, trampoline);
    paging.prepare_firmware_scratch(frames);
    paging.map_tables(frames);
    arch::protection();
    arch::activate(paging.root);
    let _discovery = acpi::discover(info, map_bytes, &mut paging);
    paging.audit(frames);
    assert_eq!(paging.leaf(text_start()) & (2 | (1 << 63)), 0);
    assert_eq!(paging.leaf(nx_probe()) & (2 | (1 << 63)), 2 | (1 << 63));
    assert_eq!(
        paging.leaf(arch::stack_top() - PAGE) & (2 | (1 << 63)),
        2 | (1 << 63)
    );
    for guard in arch::guard_pages() {
        assert_eq!(paging.leaf(guard), 0);
    }
    #[cfg(feature = "native-smp")]
    for guard in smp::guard_pages() {
        assert_eq!(paging.leaf(guard), 0);
    }
    serial::write(b"KS-03:PASS\n");
    #[cfg(feature = "native-smp")]
    smp::start(smp_prepared, &_discovery.madt, frames, &mut paging);
    #[cfg(feature = "native-smp")]
    smp::margins();
    #[cfg(feature = "native-smp")]
    lock::selftest();
    #[cfg(feature = "native-rtc")]
    rtc::run();
    if cfg!(feature = "test-m2-ro") {
        // SAFETY: Deliberate fault probe through an integer address; no Rust
        // reference is created to immutable text. Hardware WP must reject store.
        unsafe {
            asm!("mov byte ptr [{0}], 0",in(reg) text_start(),options(nostack));
        }
        serial::fail(b"M2_RO_DID_NOT_FAULT");
    }
    if cfg!(feature = "test-m2-nx") {
        // SAFETY: Deliberate execution probe; valid mapped data begins with ret,
        // but hardware NX must fault BEFORE any data is executed.
        unsafe {
            asm!("call {0}",in(reg) nx_probe());
        }
        serial::fail(b"M2_NX_DID_NOT_FAULT");
    }
    if cfg!(feature = "test-m2-absent") {
        // SAFETY: Test-only deliberate unmapped access uses assembly, avoiding a
        // Rust null dereference (which would be UB independently of hardware).
        unsafe {
            asm!("mov rax, [0]",out("rax") _,options(nostack));
        }
        serial::fail(b"M2_ABSENT_DID_NOT_FAULT");
    }
    if cfg!(feature = "test-m2-kernel-fault") {
        // SAFETY: Test-only architected #UD; own IDT must stop with raw exit35.
        unsafe {
            asm!("ud2", options(noreturn));
        }
    }
    if cfg!(feature = "test-m2-double-fault") {
        arch::double_fault_probe();
    }
    #[cfg(feature = "graphics-g0")]
    crate::graphics::run();
    #[cfg(feature = "milestone-m3")]
    {
        serial::write(b"M2:PASS\n");
        m3::start(frames, paging);
    }
    #[cfg(all(not(feature = "milestone-m3"), not(feature = "graphics-g0")))]
    {
        serial::write(b"M2:PASS\nM1:PASS\n");
        #[cfg(feature = "native-acpi-s5")]
        power::off();
        serial::exit(0x10)
    }
}

fn check_map_rejections() {
    let mut bytes = [0u8; 96];
    bytes[..4].copy_from_slice(&7u32.to_le_bytes());
    bytes[8..16].copy_from_slice(&PAGE.to_le_bytes());
    bytes[24..32].copy_from_slice(&2u64.to_le_bytes());
    let (first, second) = bytes.split_at_mut(48);
    second.copy_from_slice(first);
    assert!(MemoryMap::new(&bytes, 48, 1).is_err());
    bytes[24..32].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(MemoryMap::new(&bytes[..48], 48, 1).is_err());
    let mut small = Frames::<8>::new();
    small.admit(
        Range::pages(0, 8).unwrap(),
        7,
        8,
        &[Range::pages(2 * PAGE, 1).unwrap()],
    );
    let mut seen = [false; 8];
    while let Ok(address) = small.allocate(Owner::Kernel, |_| {}) {
        assert_ne!(address, 2 * PAGE);
        let i = (address / PAGE) as usize;
        assert!(!seen[i]);
        seen[i] = true;
    }
    assert_eq!(seen.iter().filter(|x| **x).count(), 6);
}
fn check_frames(frames: &mut Frames<FRAME_COUNT>) {
    let old = Owner::User(1, Kind::Data);
    let new = Owner::User(2, Kind::Data);
    let address = frames.allocate(old, zero).unwrap();
    // SAFETY: Exclusive allocated frame, firmware identitymapped; no aliases or
    // Rust references. Full-page pattern tests actual zeroing across ownership.
    unsafe {
        ptr::write_bytes(address as *mut u8, 0xa5, PAGE as usize);
    }
    assert!(frames
        .free(address, new, |_| panic!("wrong-owner zero"))
        .is_err());
    frames.free(address, old, zero).unwrap();
    assert!(is_zero(address));
    let again = frames.allocate(new, zero).unwrap();
    assert_eq!(again, address);
    assert!(is_zero(again));
    frames.free(again, new, zero).unwrap();
    // SAFETY: Exclusive scratch, static rather than 16-KiB stack allocation.
    let quota = unsafe { &mut *ptr::addr_of_mut!(QUOTA_FRAMES) };
    let mut previous = 0;
    for slot in quota.iter_mut() {
        *slot = frames.allocate(old, zero).unwrap();
        assert!(is_zero(*slot));
        // First-fit plus ownership assertion checks every success is unique.
        assert_eq!(frames.owner(*slot).unwrap(), old);
        assert!(*slot != 0 && *slot > previous);
        previous = *slot;
    }
    assert_eq!(
        frames.allocate(old, |_| panic!("quota must precede mutation")),
        Err(kernel_core::Error::Quota)
    );
    for &address in quota.iter() {
        frames.free(address, old, zero).unwrap();
    }
}
fn is_zero(address: u64) -> bool {
    for i in 0..PAGE as usize {
        // SAFETY: Exclusive allocated/freed-yet-unreused page with firmware
        // identity mapping; volatile makes this a physical zeroing observation.
        if unsafe { (address as *const u8).add(i).read_volatile() } != 0 {
            return false;
        }
    }
    true
}

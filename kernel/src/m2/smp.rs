use core::{
    arch::asm,
    hint::spin_loop,
    sync::atomic::{compiler_fence, fence, AtomicBool, AtomicU32, AtomicU64, AtomicU8, Ordering},
};

use boot_protocol::smp::{Info, MAX_CPUS};
use kernel_core::smp::{CpuSet, State};

use super::{ap, paging::Paging, FRAME_COUNT};
use crate::serial;

const PRESENT_WRITE_NX: u64 = 1 | 2 | (1 << 63);

const APIC_BASE_MSR: u32 = 0x1b;
const APIC_ENABLE: u64 = 1 << 11;
const APIC_X2_ENABLE: u64 = 1 << 10;
const APIC_ID: u64 = 0x20;
const APIC_SPURIOUS: u64 = 0xf0;
const APIC_ICR_LOW: u64 = 0x300;
const APIC_ICR_HIGH: u64 = 0x310;
const ICR_DELIVERY_PENDING: u32 = 1 << 12;
const IPI_INIT_ASSERT: u32 = 0x0000_c500;
const IPI_INIT_DEASSERT: u32 = 0x0000_8500;
const IPI_STARTUP: u32 = 0x0000_0600;
const MAX_IPI_POLLS: usize = 1_000_000;
const MAX_AP_POLLS: usize = 100_000_000;
const INIT_DELAY_SPINS: usize = 20_000_000;
const WORK_ITEMS: u32 = 64;

const AP_DISCOVERED: u8 = 0;
const AP_ONLINE: u8 = 1;
const AP_FAILED: u8 = 2;
const AP_STOPPED: u8 = 3;
const AP_ENTERED: u8 = 4;

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct BootstrapData {
    pub apic_ids: [u32; MAX_CPUS],
    pub stack_tops: [u64; MAX_CPUS],
    pub cr3: u64,
}

impl BootstrapData {
    pub const EMPTY: Self = Self {
        apic_ids: [u32::MAX; MAX_CPUS],
        stack_tops: [0; MAX_CPUS],
        cr3: 0,
    };
}

static APIC_WINDOW: AtomicU64 = AtomicU64::new(0);
static AP_STATES: [AtomicU8; MAX_CPUS] = [const { AtomicU8::new(AP_DISCOVERED) }; MAX_CPUS];
static REQUESTS: [AtomicU32; MAX_CPUS] = [const { AtomicU32::new(0) }; MAX_CPUS];
static ACKS: [AtomicU32; MAX_CPUS] = [const { AtomicU32::new(0) }; MAX_CPUS];
static STOP: AtomicBool = AtomicBool::new(false);

pub fn prepare(info: &Info, paging: &mut Paging, frames: &mut kernel_core::Frames<FRAME_COUNT>) {
    info.validate().expect("SMP topology");
    let count = usize::from(info.cpu_count);
    assert!(count >= 2, "SMP acceptance requires at least two CPUs");
    let mut cpus =
        CpuSet::new(&info.apic_ids[..count], info.bootstrap_apic_id).expect("CPU inventory");
    assert_eq!(cpus.online_count(), 1);
    for index in 0..count {
        if index != cpus.bootstrap_index() {
            assert_eq!(cpus.state(info.apic_ids[index]), Some(State::Discovered));
        }
    }

    let apic_window = paging
        .map_local_apic(frames, info.local_apic_base)
        .expect("local APIC UC mapping");
    APIC_WINDOW.store(apic_window, Ordering::Release);
    ap::initialize(&info.apic_ids[..count], paging.root);
    for index in 0..count {
        let (start, top) = ap::stack_bounds(index);
        assert!(
            start.is_multiple_of(kernel_core::PAGE),
            "AP stack alignment"
        );
        assert!(top.is_multiple_of(kernel_core::PAGE), "AP stack alignment");
        assert_eq!(top - start, 16 * 1024, "AP stack bounds");
        let allocation_start = start
            .checked_sub(kernel_core::PAGE)
            .expect("AP lower guard bounds");
        let allocation_end = top
            .checked_add(kernel_core::PAGE)
            .expect("AP upper guard bounds");
        for previous in 0..index {
            let (previous_start, previous_top) = ap::stack_bounds(previous);
            assert!(
                allocation_end <= previous_start - kernel_core::PAGE
                    || allocation_start >= previous_top + kernel_core::PAGE,
                "AP stack allocations overlap"
            );
        }
        assert_eq!(paging.leaf(allocation_start), 0, "AP lower guard is mapped");
        assert_eq!(paging.leaf(top), 0, "AP upper guard is mapped");
        for address in (start..top).step_by(kernel_core::PAGE as usize) {
            let entry = paging.leaf(address);
            assert_eq!(
                entry & (0x000f_ffff_ffff_f000 | PRESENT_WRITE_NX),
                address | PRESENT_WRITE_NX,
                "AP stack page is not identity-mapped RW/NX"
            );
        }
    }
    ap::copy_trampoline(info.trampoline_page, ap_entry as *const () as u64);

    // Keep the topology check's mutable state local to bootstrap; CPU transitions
    // are recorded separately with atomics once APs start executing.
    for index in 0..count {
        if index != cpus.bootstrap_index() {
            cpus.begin_start(info.apic_ids[index])
                .expect("AP start transition");
        }
    }
}

pub fn start_and_test(info: &Info) {
    if info.cpu_count < 2 {
        serial::fail(b"SMP:NEEDS_TWO_CPUS");
    }
    // SAFETY: CPUID is available on the supported x86_64 target and has no
    // memory effects.
    if unsafe { core::arch::x86_64::__cpuid(1) }.edx & (1 << 9) == 0 {
        serial::fail(b"SMP:APIC_UNAVAILABLE");
    }
    let window = APIC_WINDOW.load(Ordering::Acquire);
    if window == 0 {
        serial::fail(b"SMP:APIC_UNMAPPED");
    }
    enable_xapic(info.local_apic_base);
    if apic_read(window, APIC_ID) >> 24 != info.bootstrap_apic_id {
        serial::fail(b"SMP:BSP_ID_MISMATCH");
    }
    apic_write(
        window,
        APIC_SPURIOUS,
        (apic_read(window, APIC_SPURIOUS) & 0x3ff) | 0x100,
    );

    let bootstrap = info
        .bootstrap_index()
        .unwrap_or_else(|_| serial::fail(b"SMP:BSP_MISSING"));
    let mut cpus = CpuSet::new(
        &info.apic_ids[..usize::from(info.cpu_count)],
        info.bootstrap_apic_id,
    )
    .unwrap_or_else(|_| serial::fail(b"SMP:CPUSET"));
    for index in 0..usize::from(info.cpu_count) {
        if index == bootstrap {
            continue;
        }
        let id = info.apic_ids[index];
        let vector = (info.trampoline_page >> 12) as u8;
        cpus.begin_start(id)
            .unwrap_or_else(|_| serial::fail(b"SMP:CPUSET_START"));
        start_ap(window, id, vector, index);
        wait_for_state(index, AP_ONLINE);
        cpus.mark_online(id)
            .unwrap_or_else(|_| serial::fail(b"SMP:CPUSET_ONLINE"));
    }

    let online = 1 + AP_STATES[..usize::from(info.cpu_count)]
        .iter()
        .filter(|state| state.load(Ordering::Acquire) == AP_ONLINE)
        .count();
    if online != usize::from(info.cpu_count) {
        serial::fail(b"SMP:ONLINE_COUNT");
    }
    for (index, apic_id) in info.apic_ids[..usize::from(info.cpu_count)]
        .iter()
        .copied()
        .enumerate()
    {
        serial::write(b"SMP:CPU:");
        write_decimal(index as u64);
        serial::write(b":APIC:");
        write_decimal(u64::from(apic_id));
        serial::write(b":ONLINE\n");
    }
    serial::write(b"SMP:CPUS:");
    write_decimal(online as u64);
    serial::write(b"\n");

    for (index, request) in REQUESTS.iter().enumerate() {
        request.store(0, Ordering::Relaxed);
        ACKS[index].store(0, Ordering::Relaxed);
    }
    for round in 1..=WORK_ITEMS {
        for index in 0..usize::from(info.cpu_count) {
            if index == bootstrap {
                continue;
            }
            REQUESTS[index].store(round, Ordering::Release);
            let mut polls = 0;
            while ACKS[index].load(Ordering::Acquire) != round {
                if AP_STATES[index].load(Ordering::Acquire) != AP_ONLINE || polls == MAX_AP_POLLS {
                    serial::fail(b"SMP:WORK_TIMEOUT");
                }
                polls += 1;
                spin_loop();
            }
        }
    }
    for index in 0..usize::from(info.cpu_count) {
        if index != bootstrap && ACKS[index].load(Ordering::Acquire) != WORK_ITEMS {
            serial::fail(b"SMP:WORK_COUNT");
        }
    }
    serial::write(b"SMP:WORKERS:PASS\n");
    STOP.store(true, Ordering::Release);
    for index in 0..usize::from(info.cpu_count) {
        if index != bootstrap {
            wait_for_state(index, AP_STOPPED);
        }
    }
}

extern "C" fn ap_entry(apic_id: u32, index: u32) -> ! {
    let index = index as usize;
    if index >= MAX_CPUS {
        terminal_halt();
    }
    AP_STATES[index].store(AP_ENTERED, Ordering::Release);
    serial::write(b"SMP:AP:ENTRY\n");
    let Some(expected_id) = ap::cpu_id(index) else {
        AP_STATES[index].store(AP_FAILED, Ordering::Release);
        terminal_halt();
    };
    if expected_id != apic_id {
        AP_STATES[index].store(AP_FAILED, Ordering::Release);
        terminal_halt();
    }
    super::arch::install_ap(index, ap::stack_top(index));
    AP_STATES[index].store(AP_ONLINE, Ordering::Release);

    loop {
        if STOP.load(Ordering::Acquire) {
            AP_STATES[index].store(AP_STOPPED, Ordering::Release);
            terminal_halt();
        }
        let request = REQUESTS[index].load(Ordering::Acquire);
        if request > ACKS[index].load(Ordering::Relaxed) {
            ACKS[index].store(request, Ordering::Release);
        }
        spin_loop();
    }
}

fn terminal_halt() -> ! {
    loop {
        // SAFETY: Terminal AP state; keep IF clear and prevent accidental work
        // after any startup identity/state mismatch.
        unsafe {
            asm!("cli", "hlt", options(nomem, nostack));
        }
    }
}

fn enable_xapic(expected_base: u64) {
    let (low, high): (u32, u32);
    // SAFETY: CPL0 read of IA32_APIC_BASE; no memory is accessed by RDMSR.
    unsafe {
        asm!("rdmsr", in("ecx") APIC_BASE_MSR, out("eax") low, out("edx") high, options(nostack));
    }
    let mut value = (u64::from(high) << 32) | u64::from(low);
    let base = value & 0x000f_ffff_ffff_f000;
    if base != expected_base || value & APIC_X2_ENABLE != 0 {
        serial::fail(b"SMP:XAPIC_MODE_MISMATCH");
    }
    value |= APIC_ENABLE;
    // SAFETY: BSP enables the firmware-selected xAPIC base after mapping it UC.
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") APIC_BASE_MSR,
            in("eax") value as u32,
            in("edx") (value >> 32) as u32,
            options(nostack)
        );
    }
}

fn start_ap(window: u64, apic_id: u32, vector: u8, index: usize) {
    if apic_id > u8::MAX as u32 {
        serial::fail(b"SMP:XAPIC_ID_RANGE");
    }
    send_ipi(window, apic_id, IPI_INIT_ASSERT);
    delay(INIT_DELAY_SPINS);
    send_ipi(window, apic_id, IPI_INIT_DEASSERT);
    delay(INIT_DELAY_SPINS);
    send_ipi(window, apic_id, IPI_STARTUP | u32::from(vector));
    delay(INIT_DELAY_SPINS / 20);
    if AP_STATES[index].load(Ordering::Acquire) != AP_ONLINE {
        send_ipi(window, apic_id, IPI_STARTUP | u32::from(vector));
    }
}

fn write_decimal(mut value: u64) {
    let mut bytes = [0u8; 20];
    let mut cursor = bytes.len();
    loop {
        cursor -= 1;
        bytes[cursor] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    serial::write(&bytes[cursor..]);
}

fn send_ipi(window: u64, destination: u32, command: u32) {
    let mut polls = 0;
    while apic_read(window, APIC_ICR_LOW) & ICR_DELIVERY_PENDING != 0 {
        if polls == MAX_IPI_POLLS {
            serial::fail(b"SMP:IPI_BUSY_TIMEOUT");
        }
        polls += 1;
        spin_loop();
    }
    apic_write(window, APIC_ICR_HIGH, destination << 24);
    apic_write(window, APIC_ICR_LOW, command);
    polls = 0;
    while apic_read(window, APIC_ICR_LOW) & ICR_DELIVERY_PENDING != 0 {
        if polls == MAX_IPI_POLLS {
            serial::fail(b"SMP:IPI_SEND_TIMEOUT");
        }
        polls += 1;
        spin_loop();
    }
}

fn wait_for_state(index: usize, expected: u8) {
    for _ in 0..MAX_AP_POLLS {
        let state = AP_STATES[index].load(Ordering::Acquire);
        if state == expected {
            return;
        }
        if state == AP_FAILED {
            serial::fail(b"SMP:AP_FAILED");
        }
        spin_loop();
    }
    serial::write(b"SMP:AP_STATE:");
    write_decimal(u64::from(AP_STATES[index].load(Ordering::Acquire)));
    serial::write(b"\n");
    serial::fail(b"SMP:AP_START_TIMEOUT");
}

fn delay(spins: usize) {
    for _ in 0..spins {
        spin_loop();
    }
}

fn apic_read(window: u64, offset: u64) -> u32 {
    assert!(offset < 4096 && offset.is_multiple_of(16));
    compiler_fence(Ordering::SeqCst);
    // SAFETY: The validated UC APIC page is mapped at this fixed supervisor VA.
    let value = unsafe { ((window + offset) as *const u32).read_volatile() };
    fence(Ordering::SeqCst);
    value
}

fn apic_write(window: u64, offset: u64, value: u32) {
    assert!(offset < 4096 && offset.is_multiple_of(16));
    compiler_fence(Ordering::SeqCst);
    // SAFETY: The validated UC APIC page is mapped at this fixed supervisor VA.
    unsafe {
        ((window + offset) as *mut u32).write_volatile(value);
    }
    fence(Ordering::SeqCst);
}

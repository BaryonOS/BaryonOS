//! Ring-3 experiment. The opt-in SMP profile serializes shared M3 mutations;
//! each CPU retains its own running-process identity and entry stack.
use super::{
    arch,
    paging::{self, Paging},
    text_start, FRAME_COUNT,
};
use crate::serial;
#[cfg(feature = "native-smp-m3-lock-contention")]
use core::sync::atomic::AtomicU64;
use core::{
    arch::asm,
    mem::{replace, MaybeUninit},
    ptr,
};
#[cfg(feature = "native-smp-m3")]
use core::{
    hint::spin_loop,
    sync::atomic::{AtomicBool, Ordering},
};
#[cfg(feature = "native-memory")]
use kernel_core::pressure::{Level as MemoryPressure, ReclaimBudget};
use kernel_core::{user, Frames, Kind, Owner, PAGE};
#[cfg(feature = "display-input")]
mod display;
#[cfg(feature = "native-display-probe")]
mod display_fixture;
#[cfg(feature = "fpu-context")]
mod fpu;
#[cfg(feature = "process-memory")]
mod heap;
#[cfg(feature = "native-memory")]
mod memory_fixture;
#[cfg(feature = "display-input")]
mod ps2;
#[cfg(feature = "ipc-wait")]
mod wait;
#[cfg(all(
    feature = "native-memory",
    any(
        feature = "milestone-m4",
        feature = "native-fpu",
        feature = "native-qsim"
    )
))]
compile_error!("native-memory is a separate M3 architecture acceptance profile");
#[cfg(feature = "native-memory")]
const _: () = assert!(
    (cfg!(feature = "native-memory-nx") as u8)
        + (cfg!(feature = "native-memory-guard") as u8)
        + (cfg!(feature = "native-memory-peer") as u8)
        + (cfg!(feature = "native-memory-oom") as u8)
        <= 1,
    "choose one native-memory fault case"
);

#[cfg(all(
    feature = "native-fpu",
    any(
        all(feature = "milestone-m4", not(feature = "test-fpu-wait")),
        feature = "native-qsim"
    )
))]
compile_error!("native-fpu is a separate M3 acceptance profile");
#[cfg(feature = "native-cap-digest-bench")]
mod cap_bench;
#[cfg(all(
    feature = "native-system-materiaal",
    not(feature = "native-window-session")
))]
mod desktop;
#[cfg(feature = "native-keyring")]
#[cfg_attr(feature = "native-system", allow(dead_code))]
mod keyring;
#[cfg(feature = "milestone-m4")]
mod m4;
#[cfg(feature = "native-system-net")]
mod network;
#[cfg(feature = "native-qsim")]
mod qsim;
#[cfg(feature = "native-qsim-job")]
mod qsim_job;
#[cfg(feature = "native-sdk-app")]
#[cfg_attr(feature = "native-ipc-wait", path = "m3/wait_fixture.rs")]
mod sdk_app;
#[cfg(feature = "native-supervisor")]
#[cfg_attr(feature = "native-system", allow(dead_code))]
mod supervisor;
#[cfg(feature = "native-system")]
#[cfg_attr(feature = "native-system-materiaal", allow(dead_code))]
#[cfg_attr(feature = "native-system-nas-probe", path = "m3/nas.rs")]
#[cfg_attr(feature = "native-net-service", path = "m3/network_service.rs")]
mod system;
#[cfg(feature = "native-window-session")]
mod window_session;
#[cfg(all(
    feature = "native-system-nas-probe",
    any(
        feature = "native-system-materiaal",
        feature = "native-system-net",
        feature = "native-system-random",
        feature = "native-system-images",
        feature = "native-system-clock",
        feature = "native-cap-digest-bench"
    )
))]
compile_error!("the local NAS probe requires its own diagnostic process manifest");
#[cfg(all(
    feature = "native-window-session",
    any(
        feature = "native-system-materiaal-probe",
        feature = "native-system-session-preview"
    )
))]
compile_error!("window session and legacy desktop manifests are separate profiles");
#[cfg(feature = "native-tpm-crb")]
mod tpm_fixture;
#[cfg(any(
    all(feature = "native-system-random", feature = "native-system-net"),
    all(feature = "native-system-random", feature = "native-system-images"),
    all(feature = "native-system-images", feature = "native-system-net")
))]
compile_error!(
    "choose one fifth-process acceptance probe; this does not define the product service manifest"
);
// These diagnostic manifests assign overlapping process roles. Reject their
// union instead of silently selecting one fixture and suppressing another.
#[cfg(any(
    all(
        feature = "native-system-materiaal",
        any(
            feature = "native-system-random",
            feature = "native-system-images",
            feature = "native-system-net",
            feature = "native-cap-digest-bench"
        )
    ),
    all(
        feature = "native-cap-digest-bench",
        any(
            feature = "native-system-random",
            feature = "native-system-images",
            feature = "native-system-net"
        )
    )
))]
compile_error!("choose one diagnostic process manifest; product services are not excluded");
#[cfg(feature = "native-system-images")]
mod image_fixture;
#[cfg(feature = "native-system-random")]
mod random_fixture;
#[cfg(all(
    feature = "native-keyring",
    not(feature = "native-system"),
    any(
        feature = "milestone-m5",
        feature = "graphics-g0",
        feature = "native-qsim",
        feature = "native-qsim-job",
        feature = "native-sdk-app",
        feature = "native-supervisor"
    )
))]
compile_error!("native-keyring is a separate opt-in M4 profile");
#[cfg(all(
    feature = "native-supervisor",
    not(feature = "native-system"),
    any(
        feature = "milestone-m5",
        feature = "graphics-g0",
        feature = "native-qsim",
        feature = "native-qsim-job",
        feature = "native-sdk-app"
    )
))]
compile_error!("native-supervisor is a separate opt-in M4 profile");
#[cfg(all(
    feature = "native-qsim",
    any(
        feature = "milestone-m4",
        feature = "graphics-g0",
        feature = "native-qsim-job"
    )
))]
compile_error!("native-qsim is a separate opt-in M3 profile");
#[cfg(all(
    feature = "native-qsim-job",
    any(
        feature = "milestone-m5",
        feature = "graphics-g0",
        feature = "native-qsim"
    )
))]
compile_error!("native-qsim-job is a separate opt-in M4 profile");
#[cfg(all(
    feature = "native-sdk-app",
    any(
        feature = "milestone-m5",
        feature = "graphics-g0",
        feature = "native-qsim",
        feature = "native-qsim-job"
    )
))]
compile_error!("native-sdk-app is a separate opt-in M4 profile");
#[cfg(all(feature = "milestone-m5", not(feature = "milestone-m6a")))]
mod m5;
#[cfg(all(
    feature = "milestone-m6a",
    not(feature = "milestone-m6b"),
    not(feature = "native-volume"),
    not(feature = "native-update")
))]
#[path = "m3/m6.rs"]
mod m5;
#[cfg(feature = "native-volume")]
#[cfg_attr(feature = "native-system", allow(dead_code))]
#[path = "m3/volume.rs"]
mod m5;
#[cfg(all(feature = "native-volume", feature = "milestone-m6b"))]
compile_error!("native-volume is a separate opt-in M6a-based profile");
#[cfg(feature = "native-update")]
#[path = "m3/update.rs"]
mod m5;
#[cfg(all(
    feature = "native-update",
    any(feature = "milestone-m6b", feature = "native-volume")
))]
compile_error!("native-update is a separate opt-in M6a-based profile");
#[cfg(all(feature = "milestone-m6b", not(feature = "milestone-m6c")))]
#[path = "m3/m6b.rs"]
mod m5;
#[cfg(feature = "milestone-m6c")]
#[path = "m3/m6c.rs"]
mod m5;
#[cfg(all(
    not(feature = "milestone-m4"),
    not(feature = "native-qsim"),
    not(feature = "native-fpu"),
    not(feature = "native-memory")
))]
const ELF: &[u8] = include_bytes!(env!("NOS_USER_ELF"));
#[cfg(feature = "native-qsim")]
const ELF: &[u8] = include_bytes!(env!("NOS_QSIM_ELF"));
#[cfg(feature = "native-qsim-job")]
const ELF: &[u8] = include_bytes!(env!("NOS_QSIM_JOB_ELF"));
#[cfg(feature = "native-sdk-app")]
const SDK_SERVICE_ELF: &[u8] = include_bytes!(env!("NOS_SDK_SERVICE_ELF"));
#[cfg(feature = "native-sdk-app")]
const SDK_APP_ELF: &[u8] = include_bytes!(env!("NOS_SDK_APP_ELF"));
#[cfg(all(feature = "native-supervisor", not(feature = "native-system")))]
const SUPERVISOR_ELF: &[u8] = include_bytes!(env!("NOS_SUPERVISOR_ELF"));
#[cfg(all(feature = "native-supervisor", not(feature = "native-system")))]
const WORKER_ELF: &[u8] = include_bytes!(env!("NOS_WORKER_ELF"));
#[cfg(all(feature = "native-supervisor", not(feature = "native-system")))]
const CLIENT_ELF: &[u8] = include_bytes!(env!("NOS_CLIENT_ELF"));
#[cfg(feature = "native-keyring")]
#[cfg_attr(feature = "native-net-service", allow(dead_code))]
const KEYRING_SERVICE_ELF: &[u8] = include_bytes!(env!("NOS_KEYRING_SERVICE_ELF"));
#[cfg(all(feature = "native-keyring", not(feature = "native-system")))]
const KEYRING_APP_ELF: &[u8] = include_bytes!(env!("NOS_KEYRING_APP_ELF"));
#[cfg(all(
    feature = "milestone-m4",
    not(feature = "milestone-m5"),
    not(feature = "native-qsim-job"),
    not(feature = "native-sdk-app"),
    not(feature = "native-supervisor"),
    not(feature = "native-keyring"),
    not(feature = "native-display-probe"),
    not(feature = "native-fpu")
))]
const ELF: &[u8] = include_bytes!(env!("NOS_IPC_ELF"));
#[cfg(all(feature = "milestone-m5", not(feature = "milestone-m6a")))]
const ELF: &[u8] = include_bytes!(env!("NOS_M5_ELF"));
#[cfg(all(
    feature = "milestone-m6a",
    not(feature = "milestone-m6b"),
    not(feature = "native-volume"),
    not(feature = "native-update")
))]
const ELF: &[u8] = include_bytes!(env!("NOS_M6_ELF"));
#[cfg(feature = "native-update")]
const UPDATE_SERVICE_ELF: &[u8] = include_bytes!(env!("NOS_UPDATE_SERVICE_ELF"));
#[cfg(feature = "native-volume")]
#[cfg_attr(feature = "native-net-service", allow(dead_code))]
const VOLUME_SERVICE_ELF: &[u8] = include_bytes!(env!("NOS_VOLUME_SERVICE_ELF"));
#[cfg(all(feature = "native-volume", not(feature = "native-system")))]
const VOLUME_APP_ELF: &[u8] = include_bytes!(env!("NOS_VOLUME_APP_ELF"));
#[cfg(all(feature = "milestone-m6b", not(feature = "milestone-m6c")))]
const ELF: &[u8] = include_bytes!(env!("NOS_M6B_ELF"));
#[cfg(feature = "milestone-m6c")]
const ELF: &[u8] = include_bytes!(env!("NOS_M6C_ELF"));
#[cfg(all(feature = "native-system", not(feature = "native-system-materiaal")))]
#[cfg_attr(feature = "native-net-service", allow(dead_code))]
const SYSTEM_INIT_ELF: &[u8] = include_bytes!(env!("NOS_SYSTEM_INIT_ELF"));
#[cfg(all(feature = "native-system", not(feature = "native-system-materiaal")))]
#[cfg_attr(feature = "native-net-service", allow(dead_code))]
const SYSTEM_APP_ELF: &[u8] = include_bytes!(env!("NOS_SYSTEM_APP_ELF"));
#[cfg(all(
    feature = "native-system",
    not(feature = "native-system-materiaal"),
    not(feature = "native-system-nas-probe")
))]
#[cfg_attr(feature = "native-net-service", allow(dead_code))]
const SYSTEM_WORKER_ELF: &[u8] = include_bytes!(env!("NOS_SYSTEM_WORKER_ELF"));
#[cfg(feature = "native-system-random")]
const RANDOM_PROBE_ELF: &[u8] = include_bytes!(env!("NOS_RANDOM_PROBE_ELF"));
#[cfg(feature = "native-system-images")]
const IMAGE_PROBE_ELF: &[u8] = include_bytes!(env!("NOS_IMAGE_PROBE_ELF"));
#[cfg(all(feature = "native-system-net", not(feature = "native-net-service")))]
const NET_PROBE_ELF: &[u8] = include_bytes!(env!("NOS_NET_PROBE_ELF"));
#[cfg(feature = "native-net-service")]
const NETWORK_SERVICE_ELF: &[u8] = include_bytes!(env!("NOS_NETWORK_SERVICE_ELF"));
#[cfg(feature = "native-net-service")]
const NETWORK_CLIENT_ELF: &[u8] = include_bytes!(env!("NOS_NETWORK_CLIENT_ELF"));
const DATA: u64 = 0x40010000;
const HEAP: u64 = 0x60000000;
// B03: ML-DSA-65 verification in the ring-3 update service needs about
// 200 KiB of stack; this profile maps 512 KiB instead of the usual 64 KiB.
// The unmapped guard below the stack stays.
const STACK: u64 = if cfg!(feature = "native-update") {
    0x7ff7f000
} else {
    0x7ffef000
};
const STACK_END: u64 = 0x7ffff000;
const MAX: usize = 16;
#[cfg(feature = "native-smp-m3")]
const SMP_CPUS: usize = boot_protocol::smp::MAX_CPUS;
#[cfg(all(feature = "native-smp-m3", feature = "fpu-context"))]
compile_error!("native-smp-m3 excludes FPU state until it is modeled per CPU");
const MAPS: usize = 2048;
const ADDRESS: u64 = 0x000f_ffff_ffff_f000;
const NX: u64 = 1 << 63;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Dead,
    Ready,
    Running,
    Dying,
    #[cfg(feature = "ipc-wait")]
    Waiting,
}
#[derive(Clone, Copy)]
struct Mapping {
    va: u64,
    pa: u64,
    kind: Kind,
    writable: bool,
    executable: bool,
}
const EMPTY_MAP: Mapping = Mapping {
    va: 0,
    pa: 0,
    kind: Kind::Data,
    writable: false,
    executable: false,
};
static mut USER_MAPS: [[Mapping; MAPS]; MAX] = [[EMPTY_MAP; MAPS]; MAX];
struct Process {
    pid: u64,
    status: Status,
    paging: Option<Paging>,
    map_slot: usize,
    count: usize,
    kernel_frames: [u64; 4],
    kernel_top: u64,
    context: [u64; 22],
    selected: u64,
    pattern: u64,
    #[cfg(feature = "ipc-wait")]
    wait: Option<wait::WaitSet>,
    #[cfg(feature = "process-memory")]
    heap_pages: u64,
}
impl Process {
    const fn new() -> Self {
        Self {
            pid: 0,
            status: Status::Dead,
            paging: None,
            map_slot: MAX,
            count: 0,
            kernel_frames: [0; 4],
            kernel_top: 0,
            context: [0; 22],
            selected: 0,
            pattern: 0,
            #[cfg(feature = "ipc-wait")]
            wait: None,
            #[cfg(feature = "process-memory")]
            heap_pages: 0,
        }
    }
    fn physical(&self, va: u64) -> u64 {
        let page = va & !(PAGE - 1);
        self.mappings()
            .iter()
            .find(|m| m.va == page)
            .expect("user page")
            .pa
            + (va & (PAGE - 1))
    }
    fn mappings(&self) -> &[Mapping] {
        assert!(self.map_slot < MAX && self.count <= MAPS);
        // SAFETY: Each live Process uniquely owns one fixed arena slot. Moves
        // preserve that slot; reload begins only after cleanup drops all borrows.
        // The arena is initialized typed storage inside retained kernel memory.
        unsafe {
            core::slice::from_raw_parts(
                ptr::addr_of!(USER_MAPS)
                    .cast::<Mapping>()
                    .add(self.map_slot * MAPS),
                self.count,
            )
        }
    }
    fn append_mapping(&mut self, mapping: Mapping) {
        assert!(self.map_slot < MAX && self.count < MAPS);
        // SAFETY: Exclusive Process borrow, distinct owned arena index and no
        // retained mapping slice across this mutation or any scheduling point.
        unsafe {
            ptr::addr_of_mut!(USER_MAPS)
                .cast::<Mapping>()
                .add(self.map_slot * MAPS + self.count)
                .write(mapping);
        }
        self.count += 1;
    }
}
struct State {
    frames: &'static mut Frames<FRAME_COUNT>,
    kernel: Paging,
    processes: [Process; MAX],
    current: usize,
    #[cfg(feature = "native-smp-m3")]
    cpu_current: [usize; SMP_CPUS],
    #[cfg(feature = "native-smp-m3")]
    round_robin_cursor: usize,
    #[cfg(feature = "native-smp-m3")]
    cpu_selections: [u64; SMP_CPUS],
    #[cfg(feature = "native-smp-m3-tlb-reuse")]
    tlb_reuse_phase: u8,
    #[cfg(feature = "native-smp-m3-tlb-reuse")]
    tlb_reuse_old_pid: u64,
    #[cfg(feature = "native-smp-m3-tlb-reuse")]
    tlb_reuse_cpu: usize,
    #[cfg(feature = "native-smp-m3-tlb-reuse")]
    tlb_reuse_pa: u64,
    amount: usize,
    next_pid: u64,
    round: u8,
    ticks: u64,
    #[cfg(feature = "native-system-clock")]
    system_clock_calls: u64,
    selections: u64,
    fault: bool,
    exited: bool,
    survivor_start: u64,
    free_baseline: usize,
    timer_interrupts: u64,
    max_syscall_cycles: u64,
    syscall_calls: u64,
    survivor_slot: usize,
    unexpected: bool,
    last_event: EventRecord,
    last_tick: u64,
    max_tick_gap: u64,
    had_syscall_df_entry: bool,
    #[cfg(feature = "ipc-wait")]
    wait_metrics: wait::Metrics,
}
#[derive(Clone, Copy)]
#[cfg_attr(
    any(feature = "milestone-m4", feature = "native-fpu"),
    allow(dead_code)
)]
struct EventRecord {
    pid: u64,
    code: u64,
    vector: u64,
    rip: u64,
    cr2: u64,
}
static mut STATE: MaybeUninit<State> = MaybeUninit::uninit();
#[cfg(feature = "native-smp-m3")]
static M3_LOCKED: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "native-smp-m3-lock-contention")]
static M3_LOCK_CONTENTIONS: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "native-smp-m3")]
const M3_LOCK_POLLS: usize = 100_000_000;
#[cfg(feature = "native-smp-m3")]
struct M3Guard;
#[cfg(feature = "native-smp-m3")]
impl M3Guard {
    fn acquire() -> Self {
        for _ in 0..M3_LOCK_POLLS {
            if M3_LOCKED
                .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return Self;
            }
            #[cfg(feature = "native-smp-m3-lock-contention")]
            M3_LOCK_CONTENTIONS.fetch_add(1, Ordering::Relaxed);
            spin_loop();
        }
        serial::fail(b"M3:SMP:LOCK_TIMEOUT")
    }
}
#[cfg(feature = "native-smp-m3")]
impl Drop for M3Guard {
    fn drop(&mut self) {
        M3_LOCKED.store(false, Ordering::Release);
    }
}
#[no_mangle]
static mut m3_service_stack: u64 = 0;
#[no_mangle]
static mut m3_idle_active: u8 = 0;
#[no_mangle]
static mut m3_idle_wakeups: u64 = 0;
extern "C" {
    fn m3_timer();
    fn m3_syscall();
    #[cfg(feature = "native-smp-m3")]
    fn m3_schedule_ipi();
    fn m3_restore();
}

#[cfg(feature = "native-display-probe")]
fn process_elf(slot: usize) -> &'static [u8] {
    match slot {
        0 => include_bytes!(env!("NOS_DISPLAY_PROBE_ELF")),
        1 => include_bytes!(env!("NOS_INPUT_PROBE_ELF")),
        2 => include_bytes!(env!("NOS_DISPLAY_INTRUDER_ELF")),
        _ => panic!("display/input acceptance has three processes"),
    }
}
#[cfg(feature = "native-fpu")]
fn process_elf(slot: usize) -> &'static [u8] {
    match slot {
        0 => include_bytes!(env!("NOS_FPU_A_ELF")),
        1 => include_bytes!(env!("NOS_FPU_B_ELF")),
        _ => panic!("FPU acceptance has two processes"),
    }
}
#[cfg(feature = "native-sdk-app")]
fn process_elf(slot: usize) -> &'static [u8] {
    match slot {
        0 => SDK_SERVICE_ELF,
        1 => SDK_APP_ELF,
        _ => panic!("native SDK profile has only service and app"),
    }
}
#[cfg(feature = "native-update")]
fn process_elf(slot: usize) -> &'static [u8] {
    match slot {
        0 => UPDATE_SERVICE_ELF,
        _ => panic!("native update profile boots only the update service"),
    }
}
#[cfg(all(feature = "native-volume", not(feature = "native-system")))]
fn process_elf(slot: usize) -> &'static [u8] {
    match slot {
        0 => VOLUME_SERVICE_ELF,
        1 | 2 => VOLUME_APP_ELF,
        _ => panic!("native volume profile has one service and two apps"),
    }
}
#[cfg(all(feature = "native-supervisor", not(feature = "native-system")))]
fn process_elf(slot: usize) -> &'static [u8] {
    match slot {
        0 => SUPERVISOR_ELF,
        _ => {
            panic!("native supervisor profile boots only the supervisor; children spawn at runtime")
        }
    }
}
#[cfg(all(feature = "native-keyring", not(feature = "native-system")))]
fn process_elf(slot: usize) -> &'static [u8] {
    match slot {
        0 | 1 => KEYRING_SERVICE_ELF,
        2 => KEYRING_APP_ELF,
        _ => panic!("native keyring profile has two services and one app"),
    }
}
#[cfg(feature = "native-system")]
fn process_elf(slot: usize) -> &'static [u8] {
    #[cfg(feature = "native-window-session")]
    return window_session::process_elf(slot);
    #[cfg(all(
        feature = "native-system-materiaal",
        not(feature = "native-window-session")
    ))]
    return desktop::process_elf(slot);
    #[cfg(feature = "native-net-service")]
    return system::process_elf(slot);
    #[cfg(not(any(feature = "native-system-materiaal", feature = "native-net-service")))]
    match slot {
        0 => SYSTEM_INIT_ELF,
        1 => KEYRING_SERVICE_ELF,
        2 => VOLUME_SERVICE_ELF,
        3 => SYSTEM_APP_ELF,
        #[cfg(feature = "native-system-net")]
        4 => NET_PROBE_ELF,
        #[cfg(feature = "native-system-random")]
        4 => RANDOM_PROBE_ELF,
        #[cfg(feature = "native-system-images")]
        4 => IMAGE_PROBE_ELF,
        _ => panic!("system workers are launched by init"),
    }
}
#[cfg(feature = "native-memory")]
fn process_elf(_: usize) -> &'static [u8] {
    include_bytes!(env!("NOS_MEMORY_ELF"))
}
#[cfg(not(any(
    feature = "native-display-probe",
    feature = "native-memory",
    feature = "native-fpu",
    feature = "native-sdk-app",
    feature = "native-volume",
    feature = "native-update",
    feature = "native-supervisor",
    feature = "native-keyring"
)))]
fn process_elf(_slot: usize) -> &'static [u8] {
    ELF
}

pub fn start(frames: &'static mut Frames<FRAME_COUNT>, mut kernel: Paging) -> ! {
    arch::reset_debug();
    let image = user::parse(process_elf(0)).expect("embedded user ELF");
    assert_eq!(image.entry, 0x40000000);
    kernel.begin_scratch(frames);
    #[cfg(feature = "native-tpm-crb")]
    tpm_fixture::prepare(frames, &mut kernel);
    #[cfg(all(feature = "milestone-m5", not(feature = "native-net-service")))]
    m5::prepare(frames, &mut kernel);
    #[cfg(feature = "native-keyring")]
    keyring::prepare(frames, &mut kernel);
    #[cfg(feature = "native-system-net")]
    network::prepare(frames, &mut kernel);
    // SAFETY: One CPU, IF=0; permanent state constructed once. The service stack
    // is the retained guarded MAIN stack; no interrupted frame is stored there.
    unsafe {
        ptr::addr_of_mut!(m3_service_stack).write(arch::service_stack());
        // Initialize in place: the fixed process table is deliberately larger
        // than a kernel stack and must never be materialized as a stack local.
        let state = ptr::addr_of_mut!(STATE).cast::<State>();
        ptr::addr_of_mut!((*state).frames).write(frames);
        ptr::addr_of_mut!((*state).kernel).write(kernel);
        for i in 0..MAX {
            ptr::addr_of_mut!((*state).processes)
                .cast::<Process>()
                .add(i)
                .write(Process::new());
        }
        ptr::addr_of_mut!((*state).current).write(MAX);
        #[cfg(feature = "native-smp-m3")]
        {
            ptr::addr_of_mut!((*state).cpu_current).write([MAX; SMP_CPUS]);
            ptr::addr_of_mut!((*state).round_robin_cursor).write(0);
            ptr::addr_of_mut!((*state).cpu_selections).write([0; SMP_CPUS]);
        }
        #[cfg(feature = "native-smp-m3-tlb-reuse")]
        {
            ptr::addr_of_mut!((*state).tlb_reuse_phase).write(0);
            ptr::addr_of_mut!((*state).tlb_reuse_old_pid).write(0);
            ptr::addr_of_mut!((*state).tlb_reuse_cpu).write(MAX);
            ptr::addr_of_mut!((*state).tlb_reuse_pa).write(0);
        }
        ptr::addr_of_mut!((*state).amount).write(0);
        ptr::addr_of_mut!((*state).next_pid).write(1);
        ptr::addr_of_mut!((*state).round).write(0);
        ptr::addr_of_mut!((*state).ticks).write(0);
        #[cfg(feature = "native-system-clock")]
        ptr::addr_of_mut!((*state).system_clock_calls).write(0);
        ptr::addr_of_mut!((*state).selections).write(0);
        ptr::addr_of_mut!((*state).fault).write(false);
        ptr::addr_of_mut!((*state).exited).write(false);
        ptr::addr_of_mut!((*state).survivor_start).write(0);
        ptr::addr_of_mut!((*state).free_baseline).write(0);
        ptr::addr_of_mut!((*state).timer_interrupts).write(0);
        ptr::addr_of_mut!((*state).max_syscall_cycles).write(0);
        ptr::addr_of_mut!((*state).syscall_calls).write(0);
        ptr::addr_of_mut!((*state).survivor_slot).write(1);
        ptr::addr_of_mut!((*state).unexpected).write(false);
        ptr::addr_of_mut!((*state).last_event).write(EventRecord {
            pid: 0,
            code: 0,
            vector: 0,
            rip: 0,
            cr2: 0,
        });
        ptr::addr_of_mut!((*state).last_tick).write(0);
        ptr::addr_of_mut!((*state).max_tick_gap).write(0);
        ptr::addr_of_mut!((*state).had_syscall_df_entry).write(false);
        #[cfg(feature = "ipc-wait")]
        ptr::addr_of_mut!((*state).wait_metrics).write(wait::Metrics::EMPTY);
        arch::user_gate(32, m3_timer as *const () as u64, false);
        arch::user_gate(128, m3_syscall as *const () as u64, true);
        #[cfg(feature = "native-smp-m3")]
        arch::user_gate(0xf1, m3_schedule_ipi as *const () as u64, false);
        let state = (&mut *ptr::addr_of_mut!(STATE)).assume_init_mut();
        #[cfg(feature = "display-input")]
        {
            display::prepare();
            ps2::prepare();
        }
        state.setup();
        #[cfg(feature = "native-smp-m3")]
        {
            let bootstrap = super::smp::bootstrap_index();
            arch::set_cpu_local(bootstrap, arch::service_stack(), true);
            super::smp::release_to_m3();
            super::smp::wait_for_m3_ready();
            #[cfg(feature = "native-smp-m3-tlb-reuse")]
            assert_eq!(
                super::ap::cpu_count(),
                2,
                "native TLB-reuse acceptance requires exactly two CPUs"
            );
            #[cfg(feature = "native-smp-m4")]
            assert_eq!(
                super::ap::cpu_count(),
                2,
                "native M4 SMP acceptance requires exactly two CPUs"
            );
        }
        #[cfg(feature = "native-cap-digest-bench")]
        cap_bench::run(m4::engine());
        #[cfg(not(feature = "native-cap-digest-bench"))]
        {
            timer_init();
            #[cfg(feature = "native-smp-m4")]
            let next = {
                let _guard = M3Guard::acquire();
                super::smp::schedule_aps();
                state
                    .select_smp(super::smp::bootstrap_index())
                    .unwrap_or_else(|| serial::fail(b"M3:SMP:NO_BOOT_PROCESS"))
            };
            #[cfg(feature = "native-smp-m3-lock-contention")]
            let next = {
                let _guard = M3Guard::acquire();
                super::smp::schedule_aps();
                let mut observed = false;
                for _ in 0..M3_LOCK_POLLS {
                    if M3_LOCK_CONTENTIONS.load(Ordering::Relaxed) != 0 {
                        observed = true;
                        break;
                    }
                    spin_loop();
                }
                if !observed {
                    serial::fail(b"M3:SMP:CONTENTION_TIMEOUT");
                }
                state
                    .select_smp(super::smp::bootstrap_index())
                    .unwrap_or_else(|| serial::fail(b"M3:SMP:NO_BOOT_PROCESS"))
            };
            #[cfg(all(
                feature = "native-smp-m3",
                not(any(feature = "native-smp-m4", feature = "native-smp-m3-lock-contention"))
            ))]
            let next = state
                .select_smp(super::smp::bootstrap_index())
                .unwrap_or_else(|| serial::fail(b"M3:SMP:NO_BOOT_PROCESS"));
            #[cfg(not(feature = "native-smp-m3"))]
            let next = state.select();
            asm!("mov rax, {context}","jmp {restore}",context=in(reg) next,restore=sym m3_restore,options(noreturn));
        }
    }
}

impl State {
    fn setup(&mut self) {
        self.current = MAX;
        #[cfg(feature = "native-smp-m3")]
        {
            self.cpu_current = [MAX; SMP_CPUS];
            self.round_robin_cursor = 0;
            self.cpu_selections = [0; SMP_CPUS];
        }
        #[cfg(feature = "native-smp-m3-tlb-reuse")]
        {
            self.tlb_reuse_phase = 0;
            self.tlb_reuse_old_pid = 0;
            self.tlb_reuse_cpu = MAX;
            self.tlb_reuse_pa = 0;
        }
        self.ticks = 0;
        self.selections = 0;
        self.fault = false;
        self.exited = false;
        self.survivor_slot = 1;
        self.unexpected = false;
        self.amount = if cfg!(feature = "native-memory-oom") {
            MAX
        } else if cfg!(feature = "native-memory") || cfg!(feature = "native-fpu") {
            2
        } else if cfg!(feature = "native-supervisor") {
            // The whole table is schedulable; only the supervisor boots now.
            MAX
        } else if cfg!(feature = "native-keyring") {
            3
        } else if cfg!(feature = "native-update") {
            // One ring-3 update service; the loader and the host oracle are
            // the other two independent parties of the acceptance.
            1
        } else if cfg!(any(
            feature = "native-qsim",
            feature = "native-qsim-job",
            feature = "native-sdk-app"
        )) {
            2
        } else if cfg!(feature = "milestone-m5") && (14..=16).contains(&m5_case()) {
            4
        } else if cfg!(feature = "milestone-m4") {
            3
        } else if mode() != 0 || self.round >= 2 {
            2
        } else {
            16
        };
        let initial = if cfg!(feature = "native-net-service") {
            3
        } else if cfg!(any(
            feature = "native-system-materiaal-probe",
            feature = "native-window-session"
        )) {
            8
        } else if cfg!(feature = "native-system-materiaal") {
            7
        } else if cfg!(any(
            feature = "native-system-net",
            feature = "native-system-random",
            feature = "native-system-images"
        )) {
            5
        } else if cfg!(feature = "native-system") {
            4
        } else if cfg!(feature = "native-supervisor") {
            1
        } else {
            self.amount
        };
        for slot in 0..initial {
            self.spawn_process(slot);
        }

        let target = if cfg!(feature = "milestone-m5")
            || cfg!(feature = "native-qsim")
            || cfg!(feature = "native-qsim-job")
            || cfg!(feature = "native-sdk-app")
            || cfg!(feature = "native-supervisor")
            || cfg!(feature = "native-keyring")
        {
            0
        } else if mode() == 5 {
            text_start()
        } else {
            self.processes[1].physical(DATA + 16)
        };
        write_word(self.processes[0].physical(DATA + 24), target);
        self.survivor_start = if self.processes[1].status == Status::Dead {
            0
        } else {
            read_word(self.processes[1].physical(DATA + 8))
        };
        self.audit();
        if cfg!(feature = "test-m3-large-image") {
            serial::write(b"M3:LARGE_IMAGE:PASS\n");
        }
        #[cfg(all(
            feature = "milestone-m4",
            not(feature = "milestone-m5"),
            not(feature = "native-qsim-job"),
            not(feature = "native-sdk-app"),
            not(feature = "native-supervisor"),
            not(feature = "native-keyring"),
            not(feature = "native-display-probe"),
            not(feature = "native-fpu")
        ))]
        m4::manifest(self);
        #[cfg(feature = "native-sdk-app")]
        sdk_app::manifest(self);
        #[cfg(all(feature = "native-supervisor", not(feature = "native-system")))]
        supervisor::manifest(self);
        #[cfg(all(feature = "native-keyring", not(feature = "native-system")))]
        keyring::manifest(self);
        #[cfg(all(feature = "milestone-m5", not(feature = "native-system")))]
        m5::manifest(self);
        #[cfg(all(feature = "native-system", not(feature = "native-system-materiaal")))]
        system::manifest(self);
        #[cfg(all(
            feature = "native-system-materiaal",
            not(feature = "native-window-session")
        ))]
        desktop::manifest(self);
        #[cfg(feature = "native-window-session")]
        window_session::manifest(self);
        #[cfg(feature = "native-qsim")]
        qsim::manifest(self);
        #[cfg(feature = "native-qsim-job")]
        qsim_job::manifest(self);
        #[cfg(feature = "native-fpu")]
        fpu::fixture::manifest(self);
        #[cfg(feature = "native-memory")]
        memory_fixture::manifest(self);
        #[cfg(feature = "native-display-probe")]
        display_fixture::manifest(self);
    }
    fn spawn_process(&mut self, slot: usize) {
        self.spawn_process_from(slot, process_elf(slot));
    }
    /// Build one process from a validated embedded ELF into a dead slot. The
    /// caller has checked capacity; allocation failure here is a kernel bug.
    fn spawn_process_from(&mut self, slot: usize, elf: &'static [u8]) {
        assert!(self.processes[slot].status == Status::Dead);
        #[cfg(feature = "fpu-context")]
        fpu::initialize(slot);
        let (pid, next) = user::issue_pid(self.next_pid).expect("PID exhaustion");
        self.next_pid = next;
        let mut p = Process::new();
        p.map_slot = slot;
        p.pid = pid;
        p.pattern = 0x4141000000000000 | u64::from(pid);
        p.paging = Some(Paging::new_user(
            self.frames,
            &self.kernel,
            u16::try_from(slot + 1).unwrap(),
        ));
        let image = user::parse(elf).unwrap();
        for segment in &image.segments[..image.count] {
            let start = segment.va & !(PAGE - 1);
            let end = (segment.va + segment.memory_size + PAGE - 1) & !(PAGE - 1);
            for va in (start..end).step_by(PAGE as usize) {
                let kind = if segment.executable {
                    Kind::Text
                } else {
                    Kind::Data
                };
                let pa = self.add_user(
                    &mut p,
                    va,
                    kind,
                    segment.writable,
                    segment.executable,
                    false,
                );
                let first = core::cmp::max(va, segment.va);
                let last = core::cmp::min(va + PAGE, segment.va + segment.file_size);
                if first < last {
                    let offset = (segment.file_offset + first - segment.va) as usize;
                    let length = (last - first) as usize;
                    paging::physical(pa, |dest| {
                        // SAFETY: Entire frame exclusively allocated and not executable yet;
                        // validated ELF source/file range, nonoverlapping kernel input.
                        unsafe {
                            ptr::copy_nonoverlapping(
                                elf.as_ptr().add(offset),
                                dest.add((first - va) as usize),
                                length,
                            );
                        }
                    });
                }
                let zero_start = core::cmp::max(va, segment.va + segment.file_size);
                let zero_end = core::cmp::min(va + PAGE, segment.va + segment.memory_size);
                if zero_start < zero_end {
                    paging::physical(pa, |bytes| {
                        for offset in zero_start - va..zero_end - va {
                            // SAFETY: Exclusive newly allocated frame, not yet
                            // executable. Observe actual ELF BSS zeroing bytes.
                            assert_eq!(unsafe { bytes.add(offset as usize).read_volatile() }, 0);
                        }
                    });
                }
                if segment.executable {
                    self.kernel.identity_readonly(self.frames, pa);
                }
                p.paging.as_mut().unwrap().map_user(
                    self.frames,
                    va,
                    pa,
                    segment.writable,
                    segment.executable,
                );
            }
        }
        for va in (HEAP..HEAP + 65536).step_by(PAGE as usize) {
            self.add_user(&mut p, va, Kind::Data, true, false, true);
        }
        #[cfg(feature = "process-memory")]
        {
            p.heap_pages = 16;
        }
        for va in (STACK..STACK_END).step_by(PAGE as usize) {
            self.add_user(&mut p, va, Kind::Stack, true, false, true);
        }
        let lower = 0x30000000 + slot as u64 * 0x6000;
        assert_eq!(self.kernel.leaf(lower), 0);
        assert_eq!(self.kernel.leaf(lower + 0x5000), 0);
        for i in 0..4 {
            let pa = self
                .frames
                .allocate(Owner::Kernel, paging::zero_dynamic)
                .unwrap();
            p.kernel_frames[i] = pa;
            self.kernel.map(self.frames, pa, pa, true, false);
            self.kernel
                .map(self.frames, lower + PAGE * (i as u64 + 1), pa, true, false);
        }
        p.kernel_top = lower + 0x5000;
        let own_mode = if cfg!(feature = "milestone-m4") {
            0
        } else if mode() == 17 {
            2
        } else if mode() == 15 {
            if slot == 1 {
                15
            } else {
                0
            }
        } else if slot == 0 {
            if mode() == 0 && self.round >= 2 {
                2
            } else {
                mode()
            }
        } else {
            0
        };
        write_word(p.physical(DATA), own_mode);
        write_word(p.physical(DATA + 8), 0);
        write_word(p.physical(DATA + 16), p.pattern);
        write_word(p.physical(DATA + 24), 0);
        p.context[17] = image.entry;
        p.context[18] = 0x23;
        p.context[19] = 0x202;
        p.context[20] = STACK_END - 8;
        p.context[21] = 0x1b;
        assert_eq!(read_word(p.physical(STACK_END - 8)), 0);
        p.status = Status::Ready;
        if cfg!(feature = "test-m3-large-image") {
            assert_eq!(p.count, 67);
        }
        self.processes[slot] = p;
    }
    fn add_user(
        &mut self,
        p: &mut Process,
        va: u64,
        kind: Kind,
        writable: bool,
        executable: bool,
        map_now: bool,
    ) -> u64 {
        assert!(p.count < MAPS);
        let pa = self
            .frames
            .allocate(Owner::User(p.pid, kind), paging::zero_dynamic)
            .unwrap();
        self.kernel.map(self.frames, pa, pa, true, false);
        p.append_mapping(Mapping {
            va,
            pa,
            kind,
            writable,
            executable,
        });
        if map_now {
            p.paging
                .as_mut()
                .unwrap()
                .map_user(self.frames, va, pa, writable, executable);
        }
        pa
    }
    fn audit_process(&self, i: usize) {
        let p = &self.processes[i];
        assert!(!matches!(p.status, Status::Dead));
        let tables = p.paging.as_ref().unwrap();
        assert_ne!(tables.root, self.kernel.root);
        assert!(!tables.permissions(text_start()).unwrap().3);
        assert_eq!(tables.leaf(0), 0);
        assert_eq!(tables.leaf(STACK - PAGE), 0);
        assert_eq!(tables.leaf(STACK_END), 0);
        for m in p.mappings() {
            assert_eq!(self.frames.owner(m.pa), Ok(Owner::User(p.pid, m.kind)));
            let leaf = tables.leaf(m.va);
            assert_eq!(leaf & ADDRESS, m.pa);
            assert_ne!(leaf & 4, 0);
            assert_eq!(leaf & 2 != 0, m.writable);
            assert_eq!(leaf & NX == 0, m.executable);
            assert_eq!(
                tables.permissions(m.va).unwrap(),
                (m.pa, m.writable, m.executable, true)
            );
            let alias = self.frames.alias(m.pa).unwrap();
            assert!(alias.executable == 0 || alias.writable == 0);
            let identity = self.kernel.leaf(m.pa);
            assert_eq!(identity & 4, 0);
            assert_ne!(identity & NX, 0);
            // Executable frames never retain a writable kernel alias. Data
            // aliases stay writable for controlled copying/zeroing even when
            // that process's user mapping is read-only.
            assert_eq!(identity & 2 == 0, m.executable);
            for other in self.processes[..i]
                .iter()
                .filter(|other| other.status != Status::Dead)
            {
                assert!(other.mappings().iter().all(|x| x.pa != m.pa));
            }
        }
        // Legacy fixtures use a user-visible canary. The window-session
        // profile also audits preserved apps after arbitrary user execution:
        // their writable startup words are data, never a kernel invariant.
        #[cfg(not(feature = "native-window-session"))]
        assert_eq!(read_word(p.physical(DATA + 16)), p.pattern);
    }
    fn audit(&self) {
        for i in 0..self.amount {
            if self.processes[i].status != Status::Dead {
                self.audit_process(i);
            }
        }
    }
    fn progress_observers(&mut self) -> Option<*const u64> {
        #[allow(unused_variables)] // M3-only profiles have no observer module.
        let state = self;
        #[cfg(all(
            feature = "milestone-m4",
            not(feature = "milestone-m5"),
            not(feature = "native-qsim-job"),
            not(feature = "native-sdk-app"),
            not(feature = "native-supervisor"),
            not(feature = "native-keyring"),
            not(feature = "native-display-probe"),
            not(feature = "native-fpu")
        ))]
        if let Some(next) = m4::poll(state) {
            return Some(next);
        }
        #[cfg(all(feature = "native-supervisor", not(feature = "native-system")))]
        if let Some(next) = supervisor::poll(state) {
            return Some(next);
        }
        #[cfg(all(feature = "native-keyring", not(feature = "native-system")))]
        if let Some(next) = keyring::poll(state) {
            return Some(next);
        }
        #[cfg(all(feature = "milestone-m5", not(feature = "native-system")))]
        if let Some(next) = m5::poll(state) {
            return Some(next);
        }
        #[cfg(all(
            feature = "native-system-materiaal",
            not(feature = "native-window-session")
        ))]
        desktop::poll(state);
        #[cfg(feature = "native-window-session")]
        window_session::poll(state);
        #[cfg(all(feature = "native-system", not(feature = "native-system-materiaal")))]
        if let Some(next) = system::poll(state) {
            return Some(next);
        }
        #[cfg(feature = "native-sdk-app")]
        if let Some(next) = sdk_app::poll(state) {
            return Some(next);
        }
        #[cfg(feature = "native-qsim")]
        if let Some(next) = qsim::poll(state) {
            return Some(next);
        }
        #[cfg(feature = "native-qsim-job")]
        if let Some(next) = qsim_job::poll(state) {
            return Some(next);
        }
        #[cfg(feature = "native-fpu")]
        fpu::fixture::poll(state);
        #[cfg(feature = "native-memory")]
        memory_fixture::poll(state);
        #[cfg(feature = "native-display-probe")]
        display_fixture::poll(state);
        None
    }
    #[cfg_attr(feature = "native-smp-m3", allow(dead_code))]
    fn select(&mut self) -> *const u64 {
        'selection: loop {
            #[cfg(feature = "ipc-wait")]
            wait::refresh(self);
            #[cfg(feature = "native-tpm-crb")]
            tpm_fixture::poll(self.ticks);
            let start = if self.current < self.amount {
                (self.current + 1) % self.amount
            } else {
                0
            };
            for offset in 0..self.amount {
                let slot = (start + offset) % self.amount;
                if self.processes[slot].status == Status::Ready {
                    let p = &self.processes[slot];
                    let c = &p.context;
                    let valid =
                        user::valid_return_layout(c[17], c[20], c[18], c[21], STACK..STACK_END)
                            && p.paging
                                .as_ref()
                                .unwrap()
                                .permissions(c[17])
                                .is_some_and(|x| x.2 && x.3)
                            && p.paging
                                .as_ref()
                                .unwrap()
                                .permissions(c[20])
                                .is_some_and(|x| x.1 && !x.2 && x.3);
                    if !valid {
                        // Invalid saved user state is a process failure, never a
                        // kernel assertion or an unchecked iretq to hostile state.
                        #[cfg(not(feature = "native-window-session"))]
                        let expected = mode() == 16
                            && slot == 0
                            && c[20] == 0
                            && c[18] == 0x23
                            && c[21] == 0x1b
                            && (0x40000000..0x60000000).contains(&c[17])
                            && p.paging
                                .as_ref()
                                .unwrap()
                                .permissions(c[17])
                                .is_some_and(|x| x.2 && x.3);
                        self.last_event = EventRecord {
                            pid: p.pid,
                            code: 0,
                            vector: 256,
                            rip: c[17],
                            cr2: 0,
                        };
                        #[cfg(feature = "native-window-session")]
                        window_session::observe_invalid_return(self, slot);
                        #[cfg(not(feature = "native-window-session"))]
                        {
                            self.unexpected |= !expected;
                            if expected {
                                serial::write(b"M3:INVALID_RSP:PASS\n");
                            }
                            self.observer_survivor(slot);
                        }
                        self.fault = true;
                        self.processes[slot].status = Status::Dying;
                        self.cleanup(slot);
                        continue 'selection;
                    }
                    self.current = slot;
                    self.selections += 1;
                    let p = &mut self.processes[slot];
                    p.status = Status::Running;
                    p.selected += 1;
                    if !cfg!(feature = "native-qsim")
                        && !cfg!(feature = "native-qsim-job")
                        && !cfg!(feature = "native-sdk-app")
                        && !cfg!(feature = "native-fpu")
                        && !cfg!(feature = "native-memory")
                        && mode() == 0
                        && self.round < 2
                        && !self.fault
                        && !self.exited
                    {
                        assert_eq!(slot, ((self.selections - 1) % 16) as usize);
                    }
                    let c = &mut p.context;
                    c[19] = user::return_flags(c[19]);
                    arch::process_stack(p.kernel_top);
                    #[cfg(feature = "fpu-context")]
                    fpu::select(slot);
                    arch::activate(p.paging.as_ref().unwrap().root);
                    return c.as_ptr();
                }
            }
            #[cfg(feature = "ipc-wait")]
            if self.processes[..self.amount]
                .iter()
                .any(|p| p.status == Status::Waiting)
            {
                if let Some(next) = self.wait_idle_once() {
                    return next;
                }
                continue;
            }
            self.idle()
        }
    }
    #[cfg(feature = "native-smp-m3")]
    fn select_smp(&mut self, cpu: usize) -> Option<*const u64> {
        assert!(cpu < SMP_CPUS && cpu < super::ap::cpu_count());
        assert!(self.amount > 0 && self.amount <= MAX);
        assert_eq!(self.cpu_current[cpu], MAX);
        #[cfg(feature = "ipc-wait")]
        wait::refresh(self);
        #[cfg(feature = "native-tpm-crb")]
        tpm_fixture::poll(self.ticks);

        for offset in 0..self.amount {
            let slot = (self.round_robin_cursor + offset) % self.amount;
            if self.processes[slot].status != Status::Ready {
                continue;
            }
            #[cfg(feature = "native-smp-m4")]
            if cpu != if slot == 1 { 1 } else { 0 } {
                continue;
            }
            #[cfg(feature = "native-smp-m3-tlb-reuse")]
            if cpu != (if slot == 0 { 1 } else { 0 }) {
                continue;
            }
            let valid = {
                let process = &self.processes[slot];
                let context = &process.context;
                user::valid_return_layout(
                    context[17],
                    context[20],
                    context[18],
                    context[21],
                    STACK..STACK_END,
                ) && process
                    .paging
                    .as_ref()
                    .unwrap()
                    .permissions(context[17])
                    .is_some_and(|permissions| permissions.2 && permissions.3)
                    && process
                        .paging
                        .as_ref()
                        .unwrap()
                        .permissions(context[20])
                        .is_some_and(|permissions| permissions.1 && !permissions.2 && permissions.3)
            };
            if !valid {
                let context = self.processes[slot].context;
                #[cfg(not(feature = "native-window-session"))]
                let expected = mode() == 16
                    && slot == 0
                    && context[20] == 0
                    && context[18] == 0x23
                    && context[21] == 0x1b
                    && (0x40000000..0x60000000).contains(&context[17])
                    && self.processes[slot]
                        .paging
                        .as_ref()
                        .unwrap()
                        .permissions(context[17])
                        .is_some_and(|permissions| permissions.2 && permissions.3);
                self.last_event = EventRecord {
                    pid: self.processes[slot].pid,
                    code: 0,
                    vector: 256,
                    rip: context[17],
                    cr2: 0,
                };
                #[cfg(feature = "native-window-session")]
                window_session::observe_invalid_return(self, slot);
                #[cfg(not(feature = "native-window-session"))]
                {
                    self.unexpected |= !expected;
                    if expected {
                        serial::write(b"M3:INVALID_RSP:PASS\n");
                    }
                    self.observer_survivor(slot);
                }
                self.fault = true;
                self.processes[slot].status = Status::Dying;
                self.cleanup(slot);
                continue;
            }

            assert!((0..SMP_CPUS).all(|other| other == cpu || self.cpu_current[other] != slot));
            self.round_robin_cursor = (slot + 1) % self.amount;
            self.cpu_current[cpu] = slot;
            self.cpu_selections[cpu] = self.cpu_selections[cpu].saturating_add(1);
            self.selections += 1;
            let process = &mut self.processes[slot];
            process.status = Status::Running;
            process.selected += 1;
            let context = &mut process.context;
            context[19] = user::return_flags(context[19]);
            arch::process_stack(process.kernel_top);
            #[cfg(feature = "fpu-context")]
            fpu::select(slot);
            arch::activate(process.paging.as_ref().unwrap().root);
            return Some(context.as_ptr());
        }
        None
    }

    fn select_after_event(&mut self, cpu: usize) -> *const u64 {
        #[cfg(feature = "native-smp-m3")]
        {
            self.select_smp(cpu).unwrap_or(ptr::null())
        }
        #[cfg(not(feature = "native-smp-m3"))]
        {
            let _ = cpu;
            self.select()
        }
    }
    #[cfg(feature = "ipc-wait")]
    fn wait_idle_once(&mut self) -> Option<*const u64> {
        arch::activate(self.kernel.root);
        arch::process_stack(arch::service_stack());
        self.current = MAX;
        // SAFETY: Single CPU, IF=0 and the permanent MAIN service stack/root.
        // The existing CPL0 IRQ0 stub touches only its raw counter and saved
        // interrupt frame, never STATE/Engine or Rust references. No driver IRQ
        // is enabled by this feature. No user-memory reference is retained.
        unsafe {
            ptr::addr_of_mut!(m3_idle_wakeups).write(0);
            ptr::addr_of_mut!(m3_idle_active).write(1);
            // STI's interrupt shadow closes the check-to-sleep race. CLI runs
            // before any Rust state access; omit nomem to keep the barrier.
            asm!("sti", "hlt", "cli", options(nostack));
        }
        // SAFETY: IF=0 again. Clear the idle-only route before scheduler/user
        // resumption and observe the actual IRQ0 count once, without inferring
        // missed interrupts or elapsed wall time from RDTSC.
        let interrupts = unsafe {
            ptr::addr_of_mut!(m3_idle_active).write(0);
            ptr::addr_of!(m3_idle_wakeups).read_volatile()
        };
        self.ticks = self.ticks.saturating_add(interrupts);
        self.timer_interrupts = self.timer_interrupts.saturating_add(interrupts);
        self.wait_metrics.idle_irqs = self.wait_metrics.idle_irqs.saturating_add(interrupts);
        if interrupts != 0 {
            let now = clock();
            if self.last_tick != 0 {
                self.max_tick_gap = self.max_tick_gap.max(now.saturating_sub(self.last_tick));
            }
            self.last_tick = now;
        }
        // Even a wake with zero PIT delta must poll actual brokers/observers;
        // it cannot complete a timer-only wait before the absolute deadline.
        self.progress_observers()
    }
    #[cfg_attr(feature = "native-smp-m3", allow(dead_code))]
    fn idle(&mut self) -> ! {
        arch::activate(self.kernel.root);
        arch::process_stack(arch::service_stack());
        self.current = MAX;
        let free = self.free_count();
        // SAFETY: IF=0, permanent service stack/root. The only admitted CPL0 IRQ
        // while idle is IRQ0, whose assembly path touches only these two raw
        // counters, sends EOI and restores the CPU frame. It never borrows STATE.
        unsafe {
            ptr::addr_of_mut!(m3_idle_wakeups).write(0);
            ptr::addr_of_mut!(m3_idle_active).write(1);
        }
        loop {
            // SAFETY: STI shadow makes HLT await a real IRQ atomically. IRQ0
            // returns to CLI before Rust accesses its exclusively borrowed state.
            unsafe {
                asm!("sti", "hlt", "cli", options(nostack));
            }
            // SAFETY: IF=0; volatile observation of assembly-updated diagnostics.
            let wakes = unsafe { ptr::addr_of!(m3_idle_wakeups).read_volatile() };
            if mode() == 17 {
                assert_eq!(self.free_count(), free);
                assert!(self.processes[..self.amount]
                    .iter()
                    .all(|p| p.status == Status::Dead));
                if wakes >= 8 {
                    // SAFETY: IF=0; disable the special idle-only IRQ route before
                    // leaving idle, with no live user process or active user root.
                    unsafe {
                        ptr::addr_of_mut!(m3_idle_active).write(0);
                    }
                    print_number(b"M3:IDLE_TIMER_IRQS:", wakes);
                    serial::write(b"M3:IDLE:PASS\n");
                    self.complete_round();
                }
            }
        }
    }
    fn cleanup(&mut self, slot: usize) {
        #[cfg(feature = "native-smp-m3")]
        {
            let current_cpu = arch::current_cpu_index();
            if !self
                .cpu_current
                .iter()
                .enumerate()
                .all(|(cpu, running)| cpu == current_cpu || *running != slot)
            {
                serial::fail(b"SMP:CLEANUP_REMOTE_ACTIVE");
            }
        }
        #[cfg(feature = "process-memory")]
        assert!(
            heap::clean_boundary(),
            "heap reservations cannot outlive a syscall"
        );
        let mut p = replace(&mut self.processes[slot], Process::new());
        assert!(p.status == Status::Dying || p.status == Status::Dead);
        let mut address = p.paging.take().unwrap();
        #[cfg(feature = "fpu-context")]
        fpu::clear(slot);
        #[cfg(feature = "display-input")]
        {
            display::detach(p.pid);
            ps2::detach(p.pid);
        }
        #[cfg(feature = "milestone-m5")]
        m5::detach(p.pid);
        #[cfg(feature = "native-keyring")]
        keyring::detach(p.pid);
        #[cfg(feature = "native-system-net")]
        network::detach(p.pid);
        #[cfg(feature = "milestone-m4")]
        m4::process_exit(p.pid);
        let free_before = self.free_count();
        let total_released = p.count + p.kernel_frames.len() + address.owned_table_count();
        #[cfg(feature = "native-smp-m3-tlb-reuse")]
        let mut held_frames = 0;
        for m in p.mappings() {
            address.unmap(self.frames, m.va);
            self.kernel.unmap(self.frames, m.pa);
            #[cfg(feature = "native-smp-m3-tlb-reuse")]
            if slot == 0 && self.tlb_reuse_phase == 0 && m.va == ((DATA + 8) & !(PAGE - 1)) {
                assert_eq!(self.tlb_reuse_pa, 0);
                self.tlb_reuse_pa = m.pa;
                held_frames += 1;
                continue;
            }
            self.frames
                .free(m.pa, Owner::User(p.pid, m.kind), paging::zero_dynamic)
                .unwrap();
        }
        #[cfg(feature = "native-smp-m3-tlb-reuse")]
        assert_eq!(
            held_frames,
            if slot == 0 && self.tlb_reuse_phase == 0 {
                1
            } else {
                0
            }
        );
        let lower = p.kernel_top - 0x5000;
        for (i, pa) in p.kernel_frames.into_iter().enumerate() {
            self.kernel
                .unmap(self.frames, lower + PAGE * (i as u64 + 1));
            self.kernel.unmap(self.frames, pa);
            self.frames
                .free(pa, Owner::Kernel, paging::zero_dynamic)
                .unwrap();
        }
        address.destroy(self.frames);
        #[cfg(feature = "native-smp-m3-tlb-reuse")]
        let released = total_released - held_frames;
        #[cfg(not(feature = "native-smp-m3-tlb-reuse"))]
        let released = total_released;
        assert_eq!(self.free_count(), free_before + released);
        #[cfg(feature = "native-memory")]
        memory_fixture::observe_cleanup(self, slot, released, free_before, self.free_count());
        for index in 0..FRAME_COUNT {
            let owner = self.frames.owner(index as u64 * PAGE).unwrap();
            #[cfg(feature = "native-smp-m3-tlb-reuse")]
            let held_old_data =
                index as u64 * PAGE == self.tlb_reuse_pa && self.tlb_reuse_phase == 0;
            #[cfg(feature = "native-smp-m3-tlb-reuse")]
            assert!(!matches!(owner, Owner::User(pid, _) if pid == p.pid) || held_old_data);
            #[cfg(not(feature = "native-smp-m3-tlb-reuse"))]
            assert!(!matches!(owner,Owner::User(pid,_) if pid==p.pid));
            assert_ne!(owner, Owner::PageTable(u16::try_from(slot + 1).unwrap()));
        }
    }
    fn complete_round(&mut self) -> ! {
        #[cfg(feature = "native-smp-m3")]
        {
            for cpu in 0..super::ap::cpu_count() {
                if self.cpu_selections[cpu] == 0 {
                    serial::write(b"M3:SMP:CPU_WITHOUT_PROCESS\n");
                    serial::exit(0x11);
                }
                print_number(b"M3:SMP:CPU:", cpu as u64);
                print_number(b"M3:SMP:SCHEDULES:", self.cpu_selections[cpu]);
            }
            #[cfg(feature = "native-smp-m3-lock-contention")]
            print_number(
                b"M3:SMP:LOCK:CONTENDED:",
                M3_LOCK_CONTENTIONS.load(Ordering::Relaxed),
            );
            serial::write(b"M3:SMP:SCHEDULER:PASS\n");
            serial::exit(0x10);
        }
        #[cfg(not(feature = "native-smp-m3"))]
        {
            for slot in 0..self.amount {
                if self.processes[slot].status != Status::Dead {
                    self.processes[slot].status = Status::Dying;
                    self.cleanup(slot);
                }
            }
            if self.unexpected {
                serial::write(b"M3:FAIL:UNEXPECTED_USER_EVENT\n");
                serial::exit(0x11);
            }
            if mode() == 0 {
                if self.round == 0 {
                    serial::write(b"KS-04:PASS\n");
                    self.free_baseline = self.free_count();
                } else {
                    assert_eq!(self.free_count(), self.free_baseline);
                }
                if self.round == 1 {
                    serial::write(b"KS-07:PASS\n");
                }
                self.round += 1;
                if self.round < 6 {
                    self.setup();
                    let next = self.select();
                    // SAFETY: Full verified static user context; permanent kernel CR3
                    // and service stack remain retained through iretq.
                    unsafe {
                        asm!("mov rax, {context}","jmp {restore}",context=in(reg) next,restore=sym m3_restore,options(noreturn));
                    }
                }
            }
            print_number(b"M3:TIMER_IRQS:", self.timer_interrupts);
            print_number(b"M3:SYSCALL_CALLS:", self.syscall_calls);
            print_number(b"M3:MAX_SYSCALL_CYCLES:", self.max_syscall_cycles);
            print_number(b"M3:MAX_TIMER_GAP_CYCLES:", self.max_tick_gap);
            serial::write(b"M3:CLEANUP:PASS\n");
            if (3..=14).contains(&mode()) {
                serial::write(if mode() <= 5 {
                    b"KS-05:PASS\n"
                } else {
                    b"KS-06:PASS\n"
                });
            }
            assert!(self.had_syscall_df_entry);
            serial::write(b"KS-08:PASS\nM3:SYSCALL_DF_ENTRY:PASS\nM3:PASS\nM1:PASS\n");
            serial::exit(0x10)
        }
    }
    fn free_count(&self) -> usize {
        self.frames.availability().free
    }
    fn observer_exit(&mut self, slot: usize, counter: u64) {
        #[cfg(feature = "native-display-probe")]
        {
            display_fixture::observe_exit(self, slot, counter);
            return;
        }
        #[cfg(feature = "native-memory")]
        {
            memory_fixture::observe_exit(self, slot, counter);
            return;
        }
        #[cfg(feature = "native-fpu")]
        {
            fpu::fixture::observe_exit(self, slot, counter);
            return;
        }
        #[cfg(feature = "native-window-session")]
        {
            window_session::observe_exit(self, slot, counter);
            return;
        }
        #[cfg(all(
            feature = "native-system-materiaal",
            not(feature = "native-window-session")
        ))]
        {
            desktop::observe_exit(self, slot, counter);
            return;
        }
        #[cfg(all(feature = "native-system", not(feature = "native-system-materiaal")))]
        {
            system::observe_exit(self, slot, counter);
            return;
        }
        #[cfg(feature = "native-sdk-app")]
        {
            sdk_app::observe_exit(self, slot, counter);
            return;
        }
        #[cfg(all(feature = "native-supervisor", not(feature = "native-system")))]
        {
            supervisor::observe_exit(self, slot, counter);
            return;
        }
        #[cfg(all(feature = "native-keyring", not(feature = "native-system")))]
        {
            keyring::observe_exit(self, slot, counter);
            return;
        }
        #[cfg(feature = "native-qsim-job")]
        {
            qsim_job::observe_exit(self, slot, counter);
            return;
        }
        #[cfg(all(feature = "milestone-m5", not(feature = "native-system")))]
        {
            let _ = counter;
            m5::observe_exit(self, slot);
            return;
        }
        #[cfg(all(
            feature = "milestone-m4",
            not(feature = "milestone-m5"),
            not(feature = "native-qsim-job"),
            not(feature = "native-sdk-app"),
            not(feature = "native-supervisor"),
            not(feature = "native-keyring"),
            not(feature = "native-display-probe"),
            not(feature = "native-fpu")
        ))]
        {
            m4::observe_exit(self, slot, counter);
            return;
        }
        #[cfg(all(
            not(feature = "milestone-m4"),
            not(feature = "native-fpu"),
            not(feature = "native-memory")
        ))]
        {
            let record = self.last_event;
            let expected = if mode() == 17 {
                record.code == 0 && counter >= 8
            } else if mode() == 15 {
                slot == 1 && record.code == 7 && counter == 1
            } else {
                (mode() == 2 || mode() == 0 && self.round >= 2)
                    && slot == 0
                    && record.code == 0
                    && counter >= 8
            };
            self.unexpected |= !expected;
            if expected {
                serial::write(if mode() == 15 {
                    b"M3:EXIT_NONZERO_B:PASS\n"
                } else {
                    b"M3:EXIT:PASS\n"
                });
            }
            self.observer_survivor(slot);
        }
    }
    fn observer_fault(&mut self, slot: usize, context: &[u64; 22], counter: u64) {
        #[cfg(feature = "native-display-probe")]
        {
            display_fixture::observe_fault(self, slot, context, counter);
            return;
        }
        #[cfg(feature = "native-memory")]
        {
            memory_fixture::observe_fault(self, slot, context, counter);
            return;
        }
        #[cfg(feature = "native-fpu")]
        {
            fpu::fixture::observe_fault(self, slot, context, counter);
            return;
        }
        #[cfg(feature = "native-window-session")]
        {
            window_session::observe_fault(self, slot, context, counter);
            return;
        }
        #[cfg(all(
            feature = "native-system-materiaal",
            not(feature = "native-window-session")
        ))]
        {
            desktop::observe_fault(self, slot, context, counter);
            return;
        }
        #[cfg(all(feature = "native-system", not(feature = "native-system-materiaal")))]
        {
            system::observe_fault(self, slot, context, counter);
            return;
        }
        #[cfg(feature = "native-sdk-app")]
        {
            sdk_app::observe_fault(self, slot, context, counter);
            return;
        }
        #[cfg(all(feature = "native-supervisor", not(feature = "native-system")))]
        {
            supervisor::observe_fault(self, slot, context, counter);
            return;
        }
        #[cfg(all(feature = "native-keyring", not(feature = "native-system")))]
        {
            keyring::observe_fault(self, slot, context, counter);
            return;
        }
        #[cfg(feature = "native-qsim-job")]
        {
            qsim_job::observe_fault(self, slot, context, counter);
            return;
        }
        #[cfg(all(feature = "milestone-m5", not(feature = "native-system")))]
        {
            let _ = counter;
            m5::observe_fault(self, slot, context);
            return;
        }
        #[cfg(all(
            feature = "milestone-m4",
            not(feature = "milestone-m5"),
            not(feature = "native-qsim-job"),
            not(feature = "native-sdk-app"),
            not(feature = "native-supervisor"),
            not(feature = "native-keyring"),
            not(feature = "native-display-probe"),
            not(feature = "native-fpu")
        ))]
        {
            m4::observe_fault(self, slot, context, counter);
            return;
        }
        #[cfg(all(
            not(feature = "milestone-m4"),
            not(feature = "native-fpu"),
            not(feature = "native-memory")
        ))]
        {
            if slot != 0 || self.fault {
                self.unexpected = true;
                self.observer_survivor(slot);
                return;
            }
            let record = self.last_event;
            let expected = match mode() {
                3 => Some((14, 5, read_word(self.processes[0].physical(DATA + 24)))),
                4 | 5 => Some((14, 7, read_word(self.processes[0].physical(DATA + 24)))),
                6 => Some((14, 21, STACK)),
                7 => Some((14, 21, HEAP)),
                8 => Some((14, 6, 0x7ffee000)),
                9 => Some((14, 6, 0x7ffff000)),
                10 => Some((13, 0, 0)),
                11 => Some((7, 0, 0)),
                12 | 13 | 14 => Some((6, 0, 0)),
                _ => None,
            };
            let valid = slot == 0
                && !self.fault
                && counter > 0
                && expected.is_some_and(|e| {
                    record.vector == e.0
                        && context[16] == e.1
                        && (record.vector != 14 || record.cr2 == e.2)
                        && (!(mode() == 6 || mode() == 7) || record.rip == e.2)
                });
            self.unexpected |= !valid;
            if valid {
                serial::write(fault_marker());
            }
            self.observer_survivor(slot);
        }
    }
    #[cfg(not(feature = "native-window-session"))]
    fn observer_survivor(&mut self, departing: usize) {
        self.survivor_slot = (0..self.amount)
            .find(|i| {
                *i != departing
                    && matches!(self.processes[*i].status, Status::Ready | Status::Running)
            })
            .unwrap_or(MAX);
        self.survivor_start = if self.survivor_slot < MAX {
            read_word(self.processes[self.survivor_slot].physical(DATA + 8))
        } else {
            0
        };
    }
}

#[cfg(feature = "native-smp-m3")]
pub(super) fn start_ap(index: usize) -> ! {
    arch::set_cpu_local(index, super::ap::stack_top(index), false);
    super::smp::mark_m3_ready(index);
    m3_cpu_idle()
}

#[cfg(feature = "native-smp-m3")]
#[no_mangle]
extern "C" fn m3_cpu_idle() -> ! {
    loop {
        // SAFETY: This CPU is idle on its permanent M3 service stack and has
        // kernel GS/CR3 installed. The STI-HLT pair atomically waits for an IPI.
        unsafe {
            asm!("sti", "hlt", "cli", options(nostack));
        }
    }
}

#[no_mangle]
extern "C" fn m3_event(source: *const u64) -> *const u64 {
    #[cfg(feature = "native-smp-m3")]
    {
        let next = {
            let _guard = M3Guard::acquire();
            // SAFETY: The SMP lock is held, interrupts are disabled on this
            // entry, and the mutable state borrow ends before the lock is released.
            let state = unsafe { (&mut *ptr::addr_of_mut!(STATE)).assume_init_mut() };
            m3_event_locked(state, source)
        };
        return next;
    }
    #[cfg(not(feature = "native-smp-m3"))]
    {
        // SAFETY: The ordinary M3 profile has one CPU and interrupt-gate entry.
        let state = unsafe { (&mut *ptr::addr_of_mut!(STATE)).assume_init_mut() };
        m3_event_locked(state, source)
    }
}

fn m3_event_locked(state: &mut State, source: *const u64) -> *const u64 {
    let begin = clock();
    #[cfg(feature = "native-smp-m3")]
    {
        let cpu = arch::current_cpu_index();
        // SAFETY: The assembly entry supplied a live normalized interrupt frame
        // on this CPU's kernel stack; these two words are within the saved prefix.
        let vector = unsafe { source.add(15).read() };
        let cs = unsafe { source.add(18).read() };
        if cs == 8 {
            assert_eq!(
                state.cpu_current[cpu], MAX,
                "kernel interrupt while process runs"
            );
            if vector == 32 {
                assert_eq!(cpu, super::smp::bootstrap_index(), "PIT interrupt on AP");
                out8(0x20, 0x20);
                state.ticks = state.ticks.saturating_add(1);
                state.timer_interrupts = state.timer_interrupts.saturating_add(1);
                if state.last_tick != 0 {
                    state.max_tick_gap = state
                        .max_tick_gap
                        .max(begin.saturating_sub(state.last_tick));
                }
                state.last_tick = begin;
                super::smp::schedule_aps();
            } else if vector == 0xf1 {
                assert_ne!(cpu, super::smp::bootstrap_index(), "schedule IPI on BSP");
                super::smp::acknowledge_schedule_ipi();
            } else {
                serial::write(b"M3:FAIL:UNEXPECTED_KERNEL_INTERRUPT\n");
                serial::exit(0x11);
            }
            return state.select_smp(cpu).unwrap_or(source);
        }
    }
    #[cfg(feature = "native-smp-m3")]
    let cpu = arch::current_cpu_index();
    #[cfg(not(feature = "native-smp-m3"))]
    let cpu = 0;
    #[cfg(feature = "native-smp-m3")]
    let current = state.cpu_current[cpu];
    #[cfg(not(feature = "native-smp-m3"))]
    let current = state.current;
    assert!(current < state.amount);
    let mut context = [0u64; 22];
    // SAFETY: Live normalized full ring-3 frame; kernel-origin faults stop before
    // reading optional user RSP/SS fields. RSP frame source is on mapped stack.
    unsafe {
        if source.add(18).read() != 0x23 {
            serial::write(b"M3:FAIL:KERNEL_EXCEPTION\n");
            serial::exit(0x11);
        }
        ptr::copy_nonoverlapping(source, context.as_mut_ptr(), 22);
    }
    arch::activate(state.kernel.root);
    state.processes[current].context = context;
    let vector = context[15];
    #[cfg(feature = "native-fpu")]
    fpu::fixture::entry(current, vector);
    let mut terminate = false;
    if vector == 32 {
        out8(0x20, 0x20);
        state.ticks = if cfg!(feature = "monotonic-clock") {
            state.ticks.saturating_add(1)
        } else {
            state.ticks + 1
        };
        state.timer_interrupts += 1;
        if state.last_tick != 0 {
            state.max_tick_gap =
                core::cmp::max(state.max_tick_gap, begin.saturating_sub(state.last_tick));
        }
        state.last_tick = begin;
        #[cfg(feature = "native-smp-m3")]
        {
            assert_eq!(cpu, super::smp::bootstrap_index(), "PIT interrupt on AP");
            super::smp::schedule_aps();
        }
    } else if vector == 0xf1 {
        #[cfg(feature = "native-smp-m3")]
        super::smp::acknowledge_schedule_ipi();
    } else if vector == 128 {
        state.had_syscall_df_entry |= context[19] & (1 << 10) != 0;
        let number = context[14];
        let args = [
            context[8],
            context[9],
            context[12],
            context[5],
            context[7],
            context[6],
        ];
        #[cfg(all(feature = "native-supervisor", not(feature = "native-system")))]
        let supervisor_call = matches!(number, 0x30 | 0x50..=0x52);
        #[cfg(all(feature = "native-keyring", not(feature = "native-system")))]
        let supervisor_call = matches!(number, 0x60 | 0x61);
        #[cfg(feature = "native-system")]
        let supervisor_call = matches!(number, 0x30 | 0x50..=0x52 | 0x60 | 0x61)
            || cfg!(feature = "native-system-net") && matches!(number, 0x70..=0x72)
            || cfg!(feature = "native-window-session") && number == 0x53;
        #[cfg(feature = "native-system-clock")]
        let supervisor_call = supervisor_call || number == 0x73;
        #[cfg(not(any(feature = "native-supervisor", feature = "native-keyring")))]
        #[allow(unused_variables)]
        let supervisor_call = false;
        #[cfg(feature = "display-input")]
        let supervisor_call = supervisor_call || matches!(number, 0x80..=0x84);
        #[cfg(feature = "native-supervisor")]
        if matches!(number, 0x30 | 0x50..=0x52) {
            let (rax, rdx) = supervisor::syscall(state, current, number, args);
            state.processes[current].context[14] = rax as u64;
            state.processes[current].context[12] = rdx;
        }
        #[cfg(feature = "native-window-session")]
        if number == 0x53 {
            let (rax, rdx) = window_session::restart(state, current, args);
            state.processes[current].context[14] = rax as u64;
            state.processes[current].context[12] = rdx;
        }
        #[cfg(feature = "native-system-clock")]
        if number == 0x73 {
            let (rax, rdx) = if args.iter().all(|value| *value == 0) {
                match super::rtc::read_current_utc() {
                    Ok(seconds) => {
                        state.system_clock_calls = state.system_clock_calls.saturating_add(1);
                        (0, seconds as u64)
                    }
                    Err(()) => (-5, 0),
                }
            } else {
                (-22, 0)
            };
            state.processes[current].context[14] = rax as u64;
            state.processes[current].context[12] = rdx;
        }
        #[cfg(feature = "native-keyring")]
        if matches!(number, 0x60 | 0x61) {
            let (rax, rdx) = keyring::syscall(state, current, number, args);
            state.processes[current].context[14] = rax as u64;
            state.processes[current].context[12] = rdx;
        }
        #[cfg(feature = "display-input")]
        if matches!(number, 0x80..=0x84) {
            let (rax, rdx) = if number <= 0x81 {
                display::syscall(state, current, number, args)
            } else {
                ps2::syscall(state, current, number, args)
            };
            state.processes[current].context[14] = rax as u64;
            state.processes[current].context[12] = rdx;
        }
        #[cfg(feature = "native-system-net")]
        if matches!(number, 0x70..=0x72) {
            let (rax, rdx) = network::syscall(state, current, number, args);
            state.processes[current].context[14] = rax as u64;
            state.processes[current].context[12] = rdx;
        }
        #[cfg(feature = "ipc-wait")]
        let supervisor_call = supervisor_call || number == 0x22;
        #[cfg(feature = "ipc-wait")]
        if number == 0x22 {
            let (rax, rdx) = wait::syscall(state, current, args);
            state.processes[current].context[14] = rax as u64;
            state.processes[current].context[12] = rdx;
        }
        #[cfg(feature = "milestone-m4")]
        if number >= 0x10
            && !supervisor_call
            && !(cfg!(feature = "milestone-m5")
                && (matches!(number, 0x40 | 0x41)
                    || cfg!(feature = "milestone-m6a") && matches!(number, 0x42 | 0x43)))
        {
            let (rax, rdx) = m4::syscall(state, current, number, args);
            #[cfg(feature = "native-smp-m4")]
            if rax == 0 && matches!(number, 0x20 | 0x21) {
                let operation: &[u8] = if number == 0x20 {
                    b"M4:SMP:SEND:"
                } else {
                    b"M4:SMP:RECV:"
                };
                print_number(operation, ((current as u64) << 8) | cpu as u64);
            }
            #[cfg(feature = "native-smp-m4-revoke")]
            if number == 0x12 && rax == 0 {
                print_number(b"M4:SMP:REVOKE:", ((current as u64) << 8) | cpu as u64);
            }
            #[cfg(feature = "native-smp-m4-revoke")]
            if matches!(number, 0x20 | 0x21) && rax == -10 {
                print_number(b"M4:SMP:REVOKED:", ((current as u64) << 8) | cpu as u64);
            }
            #[cfg(feature = "milestone-m6b")]
            m5::ipc_result(current, number, rax);
            state.processes[current].context[14] = rax as u64;
            state.processes[current].context[12] = rdx;
        }
        #[cfg(feature = "milestone-m5")]
        if (matches!(number, 0x40 | 0x41)
            || cfg!(feature = "milestone-m6a") && matches!(number, 0x42 | 0x43))
        {
            let (rax, rdx) = m5::syscall(state, current, number, args);
            state.processes[current].context[14] = rax as u64;
            state.processes[current].context[12] = rdx;
        }
        #[cfg(feature = "process-memory")]
        if number == 4 {
            let (status, base) = heap::syscall(state, current, args);
            state.processes[current].context[14] = status as u64;
            state.processes[current].context[12] = base;
        }
        if (number < 0x10 || !cfg!(feature = "milestone-m4"))
            && !(cfg!(feature = "process-memory") && number == 4)
        {
            let p = &mut state.processes[current];
            p.context[14] = 0;
            p.context[12] = 0;
            if (number <= 1 && args.iter().any(|x| *x != 0))
                || (number == 2 && args[1..].iter().any(|x| *x != 0))
                || (number == 3
                    && cfg!(feature = "monotonic-clock")
                    && args.iter().any(|x| *x != 0))
            {
                p.context[14] = (-1i64) as u64;
            } else {
                match number {
                    0 => {
                        p.context[12] = (1 << 32)
                            | if cfg!(feature = "native-window-session") {
                                11
                            } else if cfg!(feature = "ipc-wait") {
                                10
                            } else if cfg!(feature = "native-system-clock") {
                                9
                            } else if cfg!(feature = "display-input") {
                                8
                            } else if cfg!(feature = "process-memory") {
                                7
                            } else if cfg!(feature = "monotonic-clock") {
                                6
                            } else if cfg!(feature = "native-system-net") {
                                5
                            } else if cfg!(feature = "native-keyring") {
                                4
                            } else if cfg!(feature = "native-supervisor") {
                                3
                            } else if cfg!(feature = "milestone-m6a") {
                                2
                            } else {
                                u64::from(cfg!(feature = "milestone-m5"))
                            }
                    }
                    1 => {}
                    3 if cfg!(feature = "monotonic-clock") => {
                        // Boot-monotonic PIT IRQ count, nominally 100 Hz.
                        // No walltime, suspend accounting or certificate clock.
                        p.context[12] = state.ticks;
                    }
                    2 => {
                        // Generic syscall semantics: every process may exit with any code.
                        // The code is retained metadata; fixture expectations are separate.
                        state.last_event = EventRecord {
                            pid: p.pid,
                            code: args[0],
                            vector: 128,
                            rip: context[17],
                            cr2: 0,
                        };
                        terminate = true;
                        state.exited = true;
                    }
                    _ => p.context[14] = (-2i64) as u64,
                }
            }
            if terminate {
                let counter = read_word(state.processes[current].physical(DATA + 8));
                state.observer_exit(current, counter);
            }
        }
        state.syscall_calls += 1;
    } else {
        let cr2: u64;
        // SAFETY: CPL0 fault metadata, never dereference/log arbitrary contents.
        unsafe {
            asm!("mov {}, cr2",out(reg) cr2,options(nomem,nostack));
        }
        // Generic fault semantics likewise have no hard-coded process identity.
        state.last_event = EventRecord {
            pid: state.processes[current].pid,
            code: context[16],
            vector,
            rip: context[17],
            cr2,
        };
        let counter = read_word(state.processes[current].physical(DATA + 8));
        state.observer_fault(current, &context, counter);
        state.fault = true;
        terminate = true;
    }
    if terminate {
        assert_eq!(state.last_event.pid, state.processes[current].pid);
    }
    if terminate {
        state.processes[current].status = Status::Dying;
        #[cfg(feature = "native-smp-m3-tlb-reuse")]
        if current == 0 {
            match state.tlb_reuse_phase {
                0 => {
                    assert_eq!(cpu, 1);
                    state.tlb_reuse_old_pid = state.last_event.pid;
                    state.tlb_reuse_cpu = cpu;
                }
                1 => {
                    assert_eq!(cpu, state.tlb_reuse_cpu);
                    assert_ne!(state.last_event.pid, state.tlb_reuse_old_pid);
                    let new_data = state.processes[current].physical(DATA + 8) & !(PAGE - 1);
                    assert_ne!(new_data, state.tlb_reuse_pa);
                    assert!(read_word(state.processes[current].physical(DATA + 8)) >= 8);
                }
                _ => serial::fail(b"M3:SMP:TLB_REUSE:PHASE"),
            }
        }
        state.cleanup(current);
        #[cfg(feature = "native-smp-m3-tlb-reuse")]
        if current == 0 {
            match state.tlb_reuse_phase {
                0 => {
                    state.spawn_process(current);
                    let replacement = &state.processes[current];
                    assert_ne!(replacement.pid, state.tlb_reuse_old_pid);
                    let data = replacement.physical(DATA + 8);
                    assert_ne!(data & !(PAGE - 1), state.tlb_reuse_pa);
                    assert_eq!(read_word(data), 0);
                    assert_eq!(
                        read_word(replacement.physical(DATA + 16)),
                        replacement.pattern
                    );
                    state.tlb_reuse_phase = 1;
                }
                1 => {
                    let held = state.tlb_reuse_pa;
                    let old_pid = state.tlb_reuse_old_pid;
                    state
                        .frames
                        .free(held, Owner::User(old_pid, Kind::Data), paging::zero_dynamic)
                        .unwrap();
                    state.tlb_reuse_pa = 0;
                    state.tlb_reuse_phase = 2;
                }
                _ => serial::fail(b"M3:SMP:TLB_REUSE:PHASE"),
            }
        }
        #[cfg(feature = "native-smp-m3-exit")]
        print_number(b"M3:SMP:EXIT:", ((current as u64) << 8) | cpu as u64);
        #[cfg(feature = "native-smp-m3-tlb-reuse")]
        if current == 0 && state.tlb_reuse_phase == 2 {
            serial::write(b"M3:SMP:TLB_REUSE:PASS\n");
        }
    } else {
        #[cfg(feature = "ipc-wait")]
        {
            state.processes[current].status = if state.processes[current].wait.is_some() {
                Status::Waiting
            } else {
                Status::Ready
            };
        }
        #[cfg(not(feature = "ipc-wait"))]
        {
            state.processes[current].status = Status::Ready;
        }
    }
    #[cfg(feature = "native-smp-m3")]
    {
        assert_eq!(state.cpu_current[cpu], current);
        state.cpu_current[cpu] = MAX;
    }
    if let Some(next) = state.progress_observers() {
        return next;
    }
    #[cfg(all(
        not(feature = "milestone-m4"),
        not(feature = "native-fpu"),
        not(feature = "native-memory")
    ))]
    if state.fault || state.exited {
        if state.survivor_slot == MAX && mode() != 17 {
            serial::write(b"M3:FAIL:NO_TEST_SURVIVOR\n");
            serial::exit(0x11);
        }
        if state.survivor_slot == MAX {
            if vector == 128 {
                state.max_syscall_cycles =
                    core::cmp::max(state.max_syscall_cycles, clock().saturating_sub(begin));
            }
            return state.select_after_event(cpu);
        }
        let survivor = &state.processes[state.survivor_slot];
        assert_eq!(read_word(survivor.physical(DATA + 16)), survivor.pattern);
        #[cfg(feature = "native-smp-m3-tlb-reuse")]
        let reuse_complete = state.tlb_reuse_phase == 2;
        #[cfg(not(feature = "native-smp-m3-tlb-reuse"))]
        let reuse_complete = true;
        if state.ticks >= 4
            && read_word(survivor.physical(DATA + 8)) > state.survivor_start + 8
            && reuse_complete
        {
            if !state.unexpected {
                serial::write(b"M3:SURVIVOR_PROGRESS:PASS\n");
            }
            state.complete_round();
        }
    } else if mode() == 0 && state.round < 2 {
        for p in &state.processes[..state.amount] {
            assert_eq!(read_word(p.physical(DATA + 16)), p.pattern);
        }
        if state.ticks >= 64 {
            assert!(state.processes[..16]
                .iter()
                .all(|p| p.selected >= 4 && read_word(p.physical(DATA + 8)) > 0));
            state.complete_round();
        }
    }
    let next = state.select_after_event(cpu);
    if vector == 128 {
        state.max_syscall_cycles =
            core::cmp::max(state.max_syscall_cycles, clock().saturating_sub(begin));
    }
    next
}

fn read_word(pa: u64) -> u64 {
    let page = pa & !(PAGE - 1);
    let offset = (pa & (PAGE - 1)) as usize;
    assert!(offset <= PAGE as usize - 8);
    paging::physical(page, |p| {
        let mut bytes = [0; 8];
        for (i, byte) in bytes.iter_mut().enumerate() {
            // SAFETY: Validated owned in-page bytes; bytewise observation also
            // supports the deliberately misaligned IPC output evidence.
            *byte = unsafe { p.add(offset + i).read_volatile() };
        }
        u64::from_le_bytes(bytes)
    })
}
fn write_word(pa: u64, value: u64) {
    let page = pa & !(PAGE - 1);
    let offset = (pa & (PAGE - 1)) as usize;
    assert!(offset <= PAGE as usize - 8);
    paging::physical(page, |p| {
        for (i, byte) in value.to_le_bytes().into_iter().enumerate() {
            // SAFETY: Exclusive validated in-page byte; no alignment or Rust
            // representation assumption is made about the addressed field.
            unsafe {
                p.add(offset + i).write_volatile(byte);
            }
        }
    });
}
fn out8(port: u16, value: u8) {
    // SAFETY: Fixed QEMU PIC/PIT only, CPL0; no host device or firmware calls.
    unsafe {
        asm!("out dx,al",in("dx") port,in("al") value,options(nomem,nostack,preserves_flags));
    }
}
fn timer_init() {
    out8(0x20, 0x11);
    out8(0xa0, 0x11);
    out8(0x21, 0x20);
    out8(0xa1, 0x28);
    out8(0x21, 4);
    out8(0xa1, 2);
    out8(0x21, 1);
    out8(0xa1, 1);
    out8(0x21, 0xfe);
    out8(0xa1, 0xff);
    out8(0x43, 0x36);
    out8(0x40, 11932u16 as u8);
    out8(0x40, (11932u16 >> 8) as u8);
}
fn clock() -> u64 {
    let (lo, hi): (u32, u32);
    // SAFETY: Scalar timestamp observation. TCG guest cycles are diagnostic, not
    // a host latency/realtime guarantee. No lost-tick count is inferred.
    unsafe {
        asm!("rdtsc",out("eax") lo,out("edx") hi,options(nomem,nostack));
    }
    (u64::from(hi) << 32) | u64::from(lo)
}
fn print_number(label: &[u8], mut value: u64) {
    let mut bytes = [b'0'; 20];
    for byte in bytes.iter_mut().rev() {
        *byte = b'0' + (value % 10) as u8;
        value /= 10;
    }
    serial::write(label);
    serial::write(&bytes);
    serial::write(b"\n");
}
fn mode() -> u64 {
    if cfg!(feature = "milestone-m4") {
        18
    } else if cfg!(feature = "test-m3-peer-read") {
        3
    } else if cfg!(feature = "test-m3-peer-write") {
        4
    } else if cfg!(feature = "test-m3-kernel-write") {
        5
    } else if cfg!(feature = "test-m3-stack-exec") {
        6
    } else if cfg!(feature = "test-m3-heap-exec") {
        7
    } else if cfg!(feature = "test-m3-guard-low") {
        8
    } else if cfg!(feature = "test-m3-guard-high") {
        9
    } else if cfg!(feature = "test-m3-cli") {
        10
    } else if cfg!(feature = "test-m3-x87") {
        11
    } else if cfg!(feature = "test-m3-xmm") {
        12
    } else if cfg!(feature = "test-m3-avx") {
        13
    } else if cfg!(feature = "test-m3-ud") {
        14
    } else if cfg!(feature = "test-m3-exit") {
        2
    } else if cfg!(feature = "test-m3-b-exit") {
        15
    } else if cfg!(feature = "test-m3-invalid-rsp") {
        16
    } else if cfg!(feature = "test-m3-idle") {
        17
    } else {
        0
    }
}
fn m5_case() -> u8 {
    if cfg!(feature = "test-m5-service-errors") {
        1
    } else if cfg!(feature = "test-m5-device-rights") {
        2
    } else if cfg!(feature = "test-m5-async") {
        3
    } else if cfg!(feature = "test-m5-bad-descriptor") {
        4
    } else if cfg!(feature = "test-m5-used-id") {
        5
    } else if cfg!(feature = "test-m5-used-index") {
        6
    } else if cfg!(feature = "test-m5-used-short") {
        7
    } else if cfg!(feature = "test-m5-used-long") {
        8
    } else if cfg!(feature = "test-m5-used-status") {
        9
    } else if cfg!(feature = "test-m5-io-error") {
        10
    } else if cfg!(feature = "test-m5-timeout") {
        11
    } else if cfg!(feature = "test-m5-reset-timeout") {
        12
    } else if cfg!(feature = "test-m5-restart") {
        13
    } else if cfg!(feature = "test-m5-cpu-mmio") {
        14
    } else if cfg!(feature = "test-m5-cpu-dma") {
        15
    } else if cfg!(feature = "test-m5-cpu-pci") {
        16
    } else if cfg!(feature = "test-m5-no-device") {
        17
    } else {
        0
    }
}
#[cfg(not(feature = "milestone-m4"))]
#[cfg_attr(
    any(feature = "native-fpu", feature = "native-memory"),
    allow(dead_code)
)]
fn fault_marker() -> &'static [u8] {
    match mode() {
        3 => b"M3:FAULT:PEER_READ:PASS\n",
        4 => b"M3:FAULT:PEER_WRITE:PASS\n",
        5 => b"M3:FAULT:KERNEL_WRITE:PASS\n",
        6 => b"M3:FAULT:STACK_EXEC:PASS\n",
        7 => b"M3:FAULT:HEAP_EXEC:PASS\n",
        8 => b"M3:FAULT:GUARD_LOW:PASS\n",
        9 => b"M3:FAULT:GUARD_HIGH:PASS\n",
        10 => b"M3:FAULT:CLI:PASS\n",
        11 => b"M3:FAULT:X87:PASS\n",
        12 => b"M3:FAULT:XMM:PASS\n",
        13 => b"M3:FAULT:AVX:PASS\n",
        14 => b"M3:FAULT:UD:PASS\n",
        _ => b"M3:FAIL:UNEXPECTED_FAULT\n",
    }
}

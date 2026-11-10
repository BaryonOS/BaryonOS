//! Single-CPU ring-3 experiment. Each entry copies all GPR/iret state before
//! switching to a permanent service stack and kernel CR3. No user reference
//! survives a scheduling point. Every mutation holds `lock::KERNEL` (ADR-075,
//! K-1), taken at each entry and released only right before the user resume.
#[cfg(feature = "native-smp-m3")]
use super::smp;
use super::{
    arch,
    lock::{self, IrqGuard},
    paging::{self, Paging},
    text_start, FRAME_COUNT,
};
use crate::serial;
#[cfg(feature = "native-smp-m3")]
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use core::{
    arch::asm,
    mem::{replace, MaybeUninit},
    ptr,
};
use kernel_core::{user, Frames, Kind, Owner, PAGE};
#[cfg(feature = "native-smp-m3")]
use smp_lock::sched;
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
#[cfg(feature = "native-smp-m3-kill")]
mod smp_lifecycle;
#[cfg(feature = "native-smp-m3")]
mod smp_recovery;
#[cfg(feature = "native-smp-m3-unmap")]
mod smp_unmap;
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
#[cfg(any(feature = "native-qsim", feature = "native-smp-m3-soak"))]
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
    any(not(feature = "milestone-m4"), feature = "native-smp-m3-ipc"),
    not(feature = "native-qsim"),
    not(feature = "native-fpu"),
    not(feature = "native-memory")
))]
const ELF: &[u8] = include_bytes!(env!("NOS_USER_ELF"));
#[cfg(feature = "native-qsim")]
const ELF: &[u8] = include_bytes!(env!("NOS_QSIM_ELF"));
#[cfg(feature = "native-smp-m3-soak")]
const SOAK_QSIM_ELF: &[u8] = include_bytes!(env!("NOS_QSIM_ELF"));
#[cfg(feature = "native-smp-m3-soak")]
const SOAK_IPC_ELF: &[u8] = include_bytes!(env!("NOS_IPC_ELF"));
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
    not(feature = "native-smp-m3-ipc"),
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
#[cfg(feature = "native-net-service")]
const NET_SERVICE_PROCESSES: usize = system::PROCESSES;
#[cfg(not(feature = "native-net-service"))]
const NET_SERVICE_PROCESSES: usize = 0;
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
// The opt-in load fixture needs sixteen busy workers plus a separate IPC
// pair on eight CPUs. Product and quota manifests keep their sixteen slots.
const MAX: usize = if cfg!(feature = "native-smp-m3-load") {
    18
} else {
    16
};
/// R3-6: stable arena keys; slot reuse occurs only after all active CPUs leave.
#[cfg(feature = "native-smp-m3")]
static ASPACE: [lock::IrqLock<()>; MAX] = {
    let mut locks = [const { lock::IrqLock::keyed(smp_lock::level::ASPACE, 0, ()) }; MAX];
    let mut index = 0;
    while index < MAX {
        locks[index] = lock::IrqLock::keyed(smp_lock::level::ASPACE, index as u32, ());
        index += 1;
    }
    locks
};
#[cfg(feature = "native-smp-m3")]
static RUNQ: [lock::IrqLock<sched::RunQueue<MAX>>; smp::MAX_CPUS] = {
    let mut queues = [const { lock::IrqLock::keyed(smp_lock::level::RUNQ, 0, sched::RunQueue::new()) };
        smp::MAX_CPUS];
    let mut index = 0;
    while index < smp::MAX_CPUS {
        queues[index] =
            lock::IrqLock::keyed(smp_lock::level::RUNQ, index as u32, sched::RunQueue::new());
        index += 1;
    }
    queues
};
#[cfg(feature = "native-smp-m3")]
static SCHED_STATE: [AtomicU32; MAX] = [const { AtomicU32::new(sched::DEAD) }; MAX];
#[cfg(feature = "native-smp-m3")]
static QUEUED: [AtomicBool; MAX] = [const { AtomicBool::new(false) }; MAX];
#[cfg(feature = "native-smp-m3")]
static ON_CPU: [AtomicBool; MAX] = [const { AtomicBool::new(false) }; MAX];
const MAPS: usize = 2048;
const ADDRESS: u64 = 0x000f_ffff_ffff_f000;
const NX: u64 = 1 << 63;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Dead,
    Ready,
    Running,
    Dying,
    #[cfg(any(feature = "ipc-wait", feature = "native-smp-m3"))]
    Waiting,
}
#[derive(Clone, Copy)]
struct Mapping {
    va: u64,
    pa: u64,
    kind: Kind,
    writable: bool,
    executable: bool,
    #[cfg(feature = "native-smp-m3")]
    retired_alias: u64,
}
const EMPTY_MAP: Mapping = Mapping {
    va: 0,
    pa: 0,
    kind: Kind::Data,
    writable: false,
    executable: false,
    #[cfg(feature = "native-smp-m3")]
    retired_alias: 0,
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
    /// R3-2: the only CPU that runs this process (no migration in 5d-2).
    #[cfg(feature = "native-smp-m3")]
    home: usize,
    /// INV-4: CPU index bits that may cache this process's translations.
    #[cfg(feature = "native-smp-m3")]
    active: u64,
    #[cfg(feature = "native-smp-m3")]
    kill_wait: Option<(u64, u64)>,
    #[cfg(feature = "native-smp-m3")]
    quarantined: bool,
    #[cfg(feature = "native-smp-m3")]
    retired_stack: [u64; 8],
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
            #[cfg(feature = "native-smp-m3")]
            home: 0,
            #[cfg(feature = "native-smp-m3")]
            active: 0,
            #[cfg(feature = "native-smp-m3")]
            kill_wait: None,
            #[cfg(feature = "native-smp-m3")]
            quarantined: false,
            #[cfg(feature = "native-smp-m3")]
            retired_stack: [0; 8],
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
    /// Step 5d-2: M3 ticks taken from CPL3 and process switches, per CPU.
    #[cfg(feature = "native-smp-m3")]
    cpu_ticks: [u64; smp::MAX_CPUS],
    #[cfg(feature = "native-smp-m3")]
    cpu_switches: [u64; smp::MAX_CPUS],
    /// Periodic timer/0xe8 generations acknowledged by each real selection.
    #[cfg(feature = "native-smp-m3")]
    cpu_quantum: [sched::Quantum; smp::MAX_CPUS],
    #[cfg(feature = "native-smp-m3-migrate")]
    migration_stage: u8,
    #[cfg(feature = "native-smp-m3-migrate")]
    migration_progress: u64,
    #[cfg(feature = "native-smp-m3-load")]
    load_started_ms: u64,
}
#[derive(Clone, Copy)]
#[cfg_attr(
    any(
        feature = "milestone-m4",
        feature = "native-fpu",
        feature = "native-smp-m3"
    ),
    allow(dead_code)
)]
struct EventRecord {
    pid: u64,
    code: u64,
    vector: u64,
    rip: u64,
    cr2: u64,
}
/// Only reached while holding `lock::KERNEL` (K-1, K-4).
static mut STATE: MaybeUninit<State> = MaybeUninit::uninit();
#[cfg(feature = "native-smp-m3-soak")]
static mut SOAK_IPC_READY: bool = false;
/// The `KERNEL` guard of the section in progress. A slot instead of a local,
/// because `complete_round` resumes user mode from deep inside `m3_event`
/// without unwinding it. Only the holder touches it, so KERNEL makes it exclusive.
static mut KERNEL_GUARD: Option<IrqGuard<'static, ()>> = None;
/// Step 5d-2: `State.current` of each CPU while it is outside KERNEL. The
/// holder of KERNEL swaps its own entry in (`load_current`) and out
/// (`kernel_leave`), so inside KERNEL `current` always names its process.
#[cfg(feature = "native-smp-m3")]
static mut CPU_CURRENT: [usize; smp::MAX_CPUS] = [MAX; smp::MAX_CPUS];
/// Set once `m3_go` released the APs (shortcut: no spawn after that point,
/// because a spawn would then need kernel-half rounds; K05 on SMP adds it).
#[cfg(feature = "native-smp-m3")]
static mut M3_LIVE: bool = false;
#[cfg(feature = "native-smp-m3")]
fn cpu() -> usize {
    smp::cpu_index()
}
/// R3-4: TSS.RSP0 of the executing CPU.
fn set_rsp0(top: u64) {
    #[cfg(feature = "native-smp-m3")]
    smp::rsp0(top);
    #[cfg(not(feature = "native-smp-m3"))]
    arch::process_stack(top);
}
/// The executing CPU's permanent M3 service stack.
fn service_stack() -> u64 {
    #[cfg(feature = "native-smp-m3")]
    return smp::m3_stack();
    #[cfg(not(feature = "native-smp-m3"))]
    arch::service_stack()
}
/// KERNEL just taken by an M3 entry: park if the M3 phase ended, otherwise
/// swap this CPU's `current` in and fold the BSP's CPL0 IRQ0 ticks.
#[cfg(feature = "native-smp-m3")]
fn load_current() -> &'static mut State {
    if smp::m3_stopping() {
        // Nothing of STATE is touched: the stopping CPU owns it now.
        kernel_leave_raw();
        smp::m3_park();
    }
    // SAFETY: KERNEL held (K-1); STATE was initialized before M3_GO.
    let state = unsafe { (&mut *ptr::addr_of_mut!(STATE)).assume_init_mut() };
    let cpu = cpu();
    // SAFETY: KERNEL held; only its holder touches CPU_CURRENT and the counter.
    unsafe {
        state.current = ptr::addr_of!(CPU_CURRENT[cpu]).read();
        if cpu == 0 {
            let irqs = ptr::addr_of!(m3_idle_wakeups).read_volatile();
            ptr::addr_of_mut!(m3_idle_wakeups).write(0);
            state.ticks = state.ticks.saturating_add(irqs);
            state.timer_interrupts = state.timer_interrupts.saturating_add(irqs);
        }
    }
    state
}
fn kernel_enter() {
    #[cfg(feature = "native-smp-m3")]
    smp::m3_entry_admit();
    let guard = lock::KERNEL.lock_irqsave();
    // SAFETY: this CPU now holds KERNEL, the slot's only protection; the last
    // holder emptied it in `kernel_leave` before releasing the lock.
    let stale = unsafe { ptr::addr_of_mut!(KERNEL_GUARD).replace(Some(guard)) };
    assert!(stale.is_none(), "KERNEL guard slot not empty");
}
fn kernel_leave() {
    #[cfg(feature = "native-smp-m3")]
    // SAFETY: KERNEL held and STATE initialized (every caller runs after
    // `start` built it); only the holder touches CPU_CURRENT.
    unsafe {
        let current = (*ptr::addr_of!(STATE)).assume_init_ref().current;
        ptr::addr_of_mut!(CPU_CURRENT[cpu()]).write(current);
    }
    kernel_leave_raw();
}
fn kernel_leave_raw() {
    // SAFETY: as in `kernel_enter`; the guard is moved out, then dropped
    // (lock released, lockdep popped, saved IF=0 kept) after the slot is empty.
    let guard = unsafe { ptr::addr_of_mut!(KERNEL_GUARD).replace(None) };
    assert!(guard.is_some(), "KERNEL not entered");
    drop(guard);
    #[cfg(feature = "native-smp-m3")]
    smp::m3_entry_left();
}
/// Leave the kernel section (INV-7: no lock across the switch) and resume
/// `next`, a full verified user context, through `m3_restore`.
fn resume(next: *const u64) -> ! {
    kernel_leave();
    // SAFETY: Full verified static user context; permanent kernel CR3 and
    // service stack remain retained through iretq. IF stays 0 until iretq.
    unsafe {
        asm!("mov rax, {context}","jmp {restore}",context=in(reg) next,restore=sym m3_restore,options(noreturn));
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
    fn m3_ap_timer();
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
#[cfg(feature = "native-smp-m3-soak")]
fn process_elf(slot: usize) -> &'static [u8] {
    match slot {
        0..=3 => SOAK_IPC_ELF,
        4..=7 => SOAK_QSIM_ELF,
        _ => ELF,
    }
}
#[cfg(not(any(
    feature = "native-display-probe",
    feature = "native-memory",
    feature = "native-fpu",
    feature = "native-sdk-app",
    feature = "native-volume",
    feature = "native-update",
    feature = "native-supervisor",
    feature = "native-keyring",
    feature = "native-smp-m3-soak"
)))]
fn process_elf(_slot: usize) -> &'static [u8] {
    ELF
}

// Out of line: its frame must not sit under the SMP self-tests on MAIN (5d-2).
#[inline(never)]
#[cfg_attr(feature = "native-smp-m3", allow(unused_mut))]
pub fn start(frames: &'static mut Frames<FRAME_COUNT>, mut kernel: Paging) -> ! {
    arch::reset_debug();
    let image = user::parse(process_elf(0)).expect("embedded user ELF");
    assert_eq!(image.entry, 0x40000000);
    kernel_enter();
    // native-smp-m3: `smp::start` already recorded every CPU's slot (K-2).
    #[cfg(not(feature = "native-smp-m3"))]
    kernel.begin_scratch(frames);
    #[cfg(feature = "native-tpm-crb")]
    tpm_fixture::prepare(frames, &mut kernel);
    #[cfg(all(feature = "milestone-m5", not(feature = "native-net-service")))]
    m5::prepare(frames, &mut kernel);
    #[cfg(feature = "native-keyring")]
    keyring::prepare(frames, &mut kernel);
    #[cfg(feature = "native-system-net")]
    network::prepare(frames, &mut kernel);
    // SAFETY: KERNEL held, IF=0; permanent state constructed once. The service stack
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
        #[cfg(feature = "native-smp-m3")]
        {
            ptr::addr_of_mut!((*state).cpu_ticks).write([0; smp::MAX_CPUS]);
            ptr::addr_of_mut!((*state).cpu_switches).write([0; smp::MAX_CPUS]);
            ptr::addr_of_mut!((*state).cpu_quantum).write([sched::Quantum::new(); smp::MAX_CPUS]);
        }
        #[cfg(feature = "native-smp-m3-migrate")]
        {
            ptr::addr_of_mut!((*state).migration_stage).write(0);
            ptr::addr_of_mut!((*state).migration_progress).write(0);
        }
        arch::user_gate(32, m3_timer as *const () as u64, false);
        arch::user_gate(128, m3_syscall as *const () as u64, true);
        let state = (&mut *ptr::addr_of_mut!(STATE)).assume_init_mut();
        #[cfg(feature = "display-input")]
        {
            display::prepare();
            ps2::prepare();
        }
        state.setup();
        #[cfg(feature = "native-smp-m3")]
        {
            // Cleanup balance: every frame a process owns comes back; the
            // kernel tables its spawn added stay (like after M3 round 0).
            let owned: usize = state.processes[..state.amount]
                .iter()
                .filter(|p| p.status != Status::Dead)
                .map(|p| {
                    p.count + p.kernel_frames.len() + p.paging.as_ref().unwrap().owned_table_count()
                })
                .sum();
            state.free_baseline = state.free_count() + owned;
            smp::m3_go(m3_ap_timer as *const () as u64);
            #[cfg(feature = "native-smp-m3-load")]
            ptr::addr_of_mut!((*state).load_started_ms).write(smp::m3_time_ms());
            ptr::addr_of_mut!(M3_LIVE).write(true);
        }
        #[cfg(feature = "native-cap-digest-bench")]
        cap_bench::run(m4::engine());
        #[cfg(not(feature = "native-cap-digest-bench"))]
        {
            timer_init();
            resume(state.select());
        }
    }
}

impl State {
    fn setup(&mut self) {
        self.current = MAX;
        self.ticks = 0;
        self.selections = 0;
        self.fault = false;
        self.exited = false;
        self.survivor_slot = 1;
        self.unexpected = false;
        self.amount = if cfg!(any(
            feature = "native-smp-m4-quotas",
            feature = "native-smp-qsim"
        )) {
            4
        } else if cfg!(feature = "native-smp-m3-ipc") {
            MAX
        } else if cfg!(feature = "native-memory-oom") {
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
            NET_SERVICE_PROCESSES
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
        #[cfg(feature = "native-smp-m3-unmap")]
        smp_unmap::manifest(self);
        #[cfg(feature = "native-smp-m3-soak")]
        m4::manifest(self);
        #[cfg(feature = "native-smp-m3-ipc")]
        if !cfg!(feature = "native-smp-m3-soak") {
            smp_ipc_manifest(self);
        }
        #[cfg(feature = "native-smp-m3-soak")]
        unsafe {
            SOAK_IPC_READY = false;
        }
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
        if !cfg!(feature = "native-smp-m3-ipc") {
            m4::manifest(self);
        }
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
        #[cfg(any(feature = "native-qsim", feature = "native-smp-m3-soak"))]
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
        #[cfg(feature = "native-smp-m3")]
        let _address = ASPACE[slot].lock_irqsave();
        #[cfg(feature = "fpu-context")]
        fpu::initialize(slot);
        let (pid, next) = user::issue_pid(self.next_pid).expect("PID exhaustion");
        self.next_pid = next;
        let mut p = Process::new();
        p.map_slot = slot;
        p.pid = pid;
        #[cfg(feature = "native-smp-m3")]
        {
            p.home = self.pick_home();
        }
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
        #[cfg(feature = "native-smp-m3")]
        let smp_faulter = cfg!(feature = "test-smp-m3-fault") && slot == 2 && p.home == 2;
        #[cfg(not(feature = "native-smp-m3"))]
        let smp_faulter = false;
        let own_mode = if cfg!(feature = "native-smp-m3-soak") && (4..8).contains(&slot) {
            0
        } else if cfg!(feature = "native-smp-m3-ipc") {
            let ipc_left = if cfg!(feature = "native-smp-m3-soak") {
                8
            } else {
                1
            };
            if slot == ipc_left || slot == ipc_left + 1 {
                if cfg!(feature = "native-smp-m3-soak") {
                    19
                } else {
                    20
                }
            } else {
                19
            }
        } else if cfg!(feature = "milestone-m4") {
            0
        } else if smp_faulter {
            // 5d-2 negative: this process executes ud2 on AP2 (K03 under SMP).
            14
        } else if cfg!(feature = "native-smp-m3") && !cfg!(feature = "native-smp-qsim") {
            // R3-3: every SMP process is an FPU canary with its own values.
            19
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
        #[cfg(feature = "native-smp-m3-ipc")]
        write_word(p.physical(DATA + 152), (1u64 << 32) | 10);
        #[cfg(feature = "native-smp-m3-migrate")]
        if slot == 1 {
            write_word(p.physical(DATA + 136), 1);
        }
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
        #[cfg(feature = "native-smp-m3")]
        {
            // Publish every new shared kernel alias before making the task
            // dispatchable. Boot construction precedes the M3 go barrier.
            if unsafe { ptr::addr_of!(M3_LIVE).read() } {
                kernel_round();
            }
            assert!(!ON_CPU[slot].load(Ordering::Acquire));
            assert!(!QUEUED[slot].load(Ordering::Acquire));
            SCHED_STATE[slot].store(sched::READY, Ordering::Release);
            self.enqueue(slot);
        }
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
            #[cfg(feature = "native-smp-m3")]
            retired_alias: 0,
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
        #[cfg(feature = "native-smp-m3")]
        if self.processes[i].quarantined {
            smp_recovery::audit_quarantined(self, i);
            return;
        }
        #[cfg(feature = "native-smp-m3")]
        let _address = ASPACE[i].lock_irqsave();
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
        #[cfg(feature = "native-smp-m3")]
        if smp::m3_recovery_requested() {
            return Some(smp_recovery::recover());
        }
        #[allow(unused_variables)] // M3-only profiles have no observer module.
        let state = self;
        #[cfg(feature = "native-smp-m3-unmap")]
        smp_unmap::poll(state);
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
        if !cfg!(feature = "native-smp-m3-ipc") || cfg!(feature = "native-smp-m3-soak") {
            if let Some(next) = m4::poll(state) {
                return Some(next);
            }
        }
        #[cfg(feature = "native-smp-m3-soak")]
        if m4::soak_done() && unsafe { !SOAK_IPC_READY } {
            smp_ipc_manifest(state);
            write_word(state.processes[8].physical(DATA), 20);
            write_word(state.processes[9].physical(DATA), 20);
            // SAFETY: KERNEL/IF=0 serializes setup and every observer.
            unsafe { SOAK_IPC_READY = true };
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
        #[cfg(any(feature = "native-qsim", feature = "native-smp-m3-soak"))]
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
    fn select(&mut self) -> *const u64 {
        self.select_with_continuation(None)
    }

    /// A candidate is only the just-descheduled caller of an ordinary syscall.
    /// All observers, kill/wait refresh and the existing return validator still
    /// run; the final event-generation test is made here, with IF=0.
    fn select_with_continuation(&mut self, mut continuation: Option<usize>) -> *const u64 {
        #[cfg(not(feature = "native-smp-m3"))]
        let _ = &mut continuation;
        debug_assert!(lock::kernel_held(), "K-1: mutation without KERNEL");
        'selection: loop {
            #[cfg(feature = "native-smp-m3")]
            if smp::m3_recovery_requested() {
                return smp_recovery::recover();
            }
            #[cfg(feature = "native-smp-m3")]
            self.refresh_kills();
            #[cfg(feature = "ipc-wait")]
            wait::refresh(self);
            #[cfg(feature = "native-tpm-crb")]
            tpm_fixture::poll(self.ticks);
            #[cfg(not(feature = "native-smp-m3"))]
            let start = if self.current < self.amount {
                (self.current + 1) % self.amount
            } else {
                0
            };
            for _offset in 0..self.amount {
                #[cfg(not(feature = "native-smp-m3"))]
                let slot = (start + _offset) % self.amount;
                #[cfg(not(feature = "native-smp-m3"))]
                let continuing = false;
                #[cfg(feature = "native-smp-m3")]
                let (slot, continuing) = {
                    let candidate = continuation.take().filter(|&slot| {
                        slot < self.amount
                            && slot == self.current
                            && self.processes[slot].status == Status::Ready
                            && self.processes[slot].home == cpu()
                            && !self.processes[slot].quarantined
                            && self.processes[slot].kill_wait.is_none()
                            && !smp::m3_stopping()
                            && !smp::m3_recovery_requested()
                            && SCHED_STATE[slot].load(Ordering::Acquire) == sched::READY
                    });
                    let mut queue = RUNQ[cpu()].lock_irqsave();
                    // RUNQ does not open an IF window. Re-read generations
                    // after all earlier KERNEL/ASPACE wait windows and refreshes.
                    let candidate = candidate.filter(|_| {
                        self.cpu_quantum[cpu()].can_continue(smp::m3_schedule_events())
                    });
                    let selected = if let Some(slot) = candidate {
                        assert!(
                            queue.remove(slot as u32, &QUEUED),
                            "continuation missing from its home queue"
                        );
                        Some((slot as u32, true))
                    } else {
                        queue.pop(&QUEUED).map(|slot| (slot, false))
                    };
                    drop(queue);
                    let Some((slot, continuing)) = selected else {
                        break;
                    };
                    let slot = slot as usize;
                    assert_eq!(self.processes[slot].home, cpu());
                    if self.processes[slot].status == Status::Dying {
                        sched::kill_commit(&SCHED_STATE[slot], &ON_CPU[slot])
                            .expect("retired process still owns its stack");
                        self.cleanup(slot);
                        if let Some(next) = self.progress_observers() {
                            return next;
                        }
                        continue 'selection;
                    }
                    (slot, continuing)
                };
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
                    #[cfg(feature = "native-smp-m3")]
                    {
                        sched::dispatch(&SCHED_STATE[slot], cpu()).expect("SMP double dispatch");
                        assert!(sched::claim_stack(&ON_CPU[slot]), "SMP stack already owned");
                        if self.current != slot {
                            self.cpu_switches[cpu()] += 1;
                        }
                    }
                    self.current = slot;
                    if !continuing {
                        self.selections += 1;
                        #[cfg(feature = "native-smp-m3")]
                        self.cpu_quantum[cpu()].record_dispatch(smp::m3_schedule_events());
                    }
                    let p = &mut self.processes[slot];
                    p.status = Status::Running;
                    if !continuing {
                        p.selected += 1;
                    }
                    if !cfg!(feature = "native-qsim")
                        && !cfg!(feature = "native-qsim-job")
                        && !cfg!(feature = "native-sdk-app")
                        && !cfg!(feature = "native-fpu")
                        && !cfg!(feature = "native-memory")
                        && !cfg!(feature = "native-smp-m3")
                        && mode() == 0
                        && self.round < 2
                        && !self.fault
                        && !self.exited
                    {
                        assert_eq!(slot, ((self.selections - 1) % 16) as usize);
                    }
                    let c = &mut p.context;
                    c[19] = user::return_flags(c[19]);
                    set_rsp0(p.kernel_top);
                    #[cfg(feature = "fpu-context")]
                    fpu::select(slot);
                    // INV-4 (spec §6.3 step 5a): the bit precedes the CR3 load.
                    #[cfg(feature = "native-smp-m3")]
                    {
                        p.active |= 1 << cpu();
                    }
                    arch::activate(p.paging.as_ref().unwrap().root);
                    #[cfg(feature = "native-smp-m3-unmap")]
                    smp_unmap::before_dispatch(slot);
                    return c.as_ptr();
                }
            }
            #[cfg(all(feature = "ipc-wait", not(feature = "native-smp-m3")))]
            if self.processes[..self.amount]
                .iter()
                .any(|p| p.status == Status::Waiting)
            {
                if let Some(next) = self.wait_idle_once() {
                    return next;
                }
                continue;
            }
            #[cfg(feature = "native-smp-m3")]
            smp_idle();
            #[cfg(not(feature = "native-smp-m3"))]
            self.idle()
        }
    }
    #[cfg(feature = "ipc-wait")]
    fn wait_idle_once(&mut self) -> Option<*const u64> {
        arch::activate(self.kernel.root);
        arch::process_stack(arch::service_stack());
        self.current = MAX;
        // SAFETY: KERNEL held, IF=0 and the permanent MAIN service stack/root.
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
        set_rsp0(service_stack());
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
        debug_assert!(lock::kernel_held(), "K-1: mutation without KERNEL");
        #[cfg(feature = "native-smp-m3")]
        if self.processes[slot].quarantined {
            return;
        }
        #[cfg(feature = "native-smp-m3")]
        {
            assert!(!QUEUED[slot].load(Ordering::Acquire));
            assert!(!ON_CPU[slot].load(Ordering::Acquire));
            assert_eq!(self.processes[slot].active, 0);
            if SCHED_STATE[slot].load(Ordering::Acquire) != sched::DEAD {
                if SCHED_STATE[slot].load(Ordering::Acquire) != sched::DEAD_PENDING {
                    sched::kill_request(&SCHED_STATE[slot]).expect("retire live process");
                }
                sched::kill_commit(&SCHED_STATE[slot], &ON_CPU[slot])
                    .expect("retire quiescent process");
            }
        }
        #[cfg(feature = "native-smp-m3")]
        let _address = ASPACE[slot].lock_irqsave();
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
        retire_authority(slot, p.pid);
        let free_before = self.free_count();
        let released = p.count + p.kernel_frames.len() + address.owned_table_count();
        // Step 5d-2 (INV-1/INV-4): only this CPU may still cache the user
        // half; every CPU may cache the kernel-half aliases, so on SMP all
        // PTEs go first, then one kernel round, then the frees.
        #[cfg(feature = "native-smp-m3")]
        if p.active & !(1 << cpu()) != 0 {
            serial::write(b"SMP:M3:REMOTE_ACTIVE\n");
            serial::fail(b"SMP_M3_REMOTE_ACTIVE");
        }
        let defer = cfg!(feature = "native-smp-m3");
        for index in 0..p.count {
            let m = p.mappings()[index];
            address.unmap(self.frames, m.va);
            #[cfg(feature = "native-smp-m3")]
            {
                let entry = self.kernel.clear_leaf(m.pa);
                // SAFETY: KERNEL+ASPACE held; the mapping was copied above,
                // so no shared arena borrow survives this exclusive write.
                unsafe {
                    ptr::addr_of_mut!(USER_MAPS[p.map_slot][index].retired_alias).write(entry)
                };
            }
            #[cfg(not(feature = "native-smp-m3"))]
            self.kernel.unmap(self.frames, m.pa);
            if !defer {
                self.frames
                    .free(m.pa, Owner::User(p.pid, m.kind), paging::zero_dynamic)
                    .unwrap();
            }
        }
        let lower = p.kernel_top - 0x5000;
        for (i, pa) in p.kernel_frames.into_iter().enumerate() {
            #[cfg(feature = "native-smp-m3")]
            {
                p.retired_stack[i] = self.kernel.clear_leaf(lower + PAGE * (i as u64 + 1));
                p.retired_stack[i + 4] = self.kernel.clear_leaf(pa);
            }
            #[cfg(not(feature = "native-smp-m3"))]
            {
                self.kernel
                    .unmap(self.frames, lower + PAGE * (i as u64 + 1));
                self.kernel.unmap(self.frames, pa);
            }
            if !defer {
                self.frames
                    .free(pa, Owner::Kernel, paging::zero_dynamic)
                    .unwrap();
            }
        }
        #[cfg(feature = "native-smp-m3")]
        {
            if !kernel_round() {
                // PTEs are gone, but alias counts, owner records, arena and
                // tables remain intact. No quarantined frame can be reused.
                p.paging = Some(address);
                p.quarantined = true;
                self.processes[slot] = p;
                return;
            }
            for m in p.mappings() {
                paging::release_leaf(self.frames, m.retired_alias);
                self.frames
                    .free(m.pa, Owner::User(p.pid, m.kind), paging::zero_dynamic)
                    .unwrap();
            }
            for entry in p.retired_stack {
                paging::release_leaf(self.frames, entry);
            }
            for pa in p.kernel_frames {
                self.frames
                    .free(pa, Owner::Kernel, paging::zero_dynamic)
                    .unwrap();
            }
        }
        address.destroy(self.frames);
        assert_eq!(self.free_count(), free_before + released);
        for index in 0..FRAME_COUNT {
            let owner = self.frames.owner(index as u64 * PAGE).unwrap();
            assert!(!matches!(owner,Owner::User(pid,_) if pid==p.pid));
            assert_ne!(owner, Owner::PageTable(u16::try_from(slot + 1).unwrap()));
        }
    }
    fn complete_round(&mut self) -> ! {
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
                resume(self.select());
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
    fn free_count(&self) -> usize {
        (0..FRAME_COUNT)
            .filter(|i| self.frames.owner(*i as u64 * PAGE) == Ok(Owner::Free))
            .count()
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
    #[cfg_attr(feature = "native-smp-m3", allow(dead_code))]
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

/// The only Rust entry for M3 events (K-1): KERNEL from the first shared
/// access until the stub resumes `next`.
#[no_mangle]
extern "C" fn m3_event(source: *const u64) -> *const u64 {
    // SAFETY: Entry stub has moved to MAIN service stack with IF=0; `source` is
    // this CPU's mapped interrupted frame (22 initialized words). Checked before
    // KERNEL so a kernel-origin exception inside a held section keeps its
    // marker instead of tripping lockdep; nothing shared is read here.
    if unsafe { source.add(18).read() } != 0x23 {
        serial::write(b"M3:FAIL:KERNEL_EXCEPTION\n");
        serial::exit(0x11);
    }
    kernel_enter();
    #[cfg(feature = "native-smp-m3")]
    load_current();
    let next = event(source);
    kernel_leave();
    next
}
fn event(source: *const u64) -> *const u64 {
    let begin = clock();
    // SAFETY: KERNEL held (K-1), so this is the only live STATE borrow. Source
    // is the exclusively owned mapped per-process kernel stack, 22 initialized
    // words; all source bytes are copied before free.
    let state = unsafe { (&mut *ptr::addr_of_mut!(STATE)).assume_init_mut() };
    let current = state.current;
    assert!(current < state.amount);
    let mut context = [0u64; 22];
    // SAFETY: Live normalized full ring-3 frame (CS checked in `m3_event`).
    // RSP frame source is on mapped stack.
    unsafe {
        ptr::copy_nonoverlapping(source, context.as_mut_ptr(), 22);
    }
    arch::activate(state.kernel.root);
    // INV-4 (spec §6.3 step 5c): cleared only after the CR3 load.
    #[cfg(feature = "native-smp-m3-migrate")]
    let entry_active = state.processes[current].active;
    #[cfg(feature = "native-smp-m3")]
    {
        assert_eq!(state.processes[current].active, 1 << cpu());
        state.processes[current].active &= !(1 << cpu());
    }
    state.processes[current].context = context;
    #[cfg(feature = "native-smp-m3")]
    {
        // The entry stub completed FXSAVE and moved to the permanent service
        // stack before entering Rust. Publish quiescence after the CR3 switch.
        assert!(ON_CPU[current].swap(false, Ordering::Release));
        // SAFETY: KERNEL held; no CPU may restore this context until dispatch.
        unsafe { ptr::addr_of_mut!(CPU_CURRENT[cpu()]).write(MAX) };
    }
    // The fixture's user loop checks its private pattern on every iteration.
    // Sample the process that actually entered, before cleanup can free DATA.
    // Scanning every peer here would serialize O(processes) scratch remaps on
    // every entry; all live processes are also checked after the final stop.
    #[cfg(all(
        feature = "native-smp-m3",
        not(any(
            feature = "native-supervisor",
            feature = "native-smp-m4-quotas",
            feature = "native-smp-qsim"
        ))
    ))]
    {
        let p = &state.processes[current];
        if p.status != Status::Dead && !p.quarantined {
            let _address = ASPACE[current].lock_irqsave();
            assert_eq!(read_word(p.physical(DATA + 16)), p.pattern);
        }
    }
    let vector = context[15];
    #[cfg(feature = "native-smp-m3")]
    if state.processes[current].status == Status::Dying {
        // The entry stub saved FPU/GPR state before taking KERNEL. Remote
        // kill is acknowledged only after this CPU left the user CR3.
        if vector == 32 {
            out8(0x20, 0x20);
        } else if matches!(vector, 0xd0 | 0xe8) {
            smp::eoi();
        }
        #[cfg(feature = "native-smp-m3-kill")]
        smp_lifecycle::entry(state, current, vector);
        state.cleanup(current);
        #[cfg(feature = "native-smp-m3-kill")]
        smp_lifecycle::retired(state, current);
        return state.select();
    }
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
            state.cpu_ticks[0] += 1;
        }
    } else if cfg!(feature = "native-smp-m3") && vector == 0xd0 {
        // An AP's M3 tick from CPL3 (the CPL0 case never reaches Rust).
        #[cfg(feature = "native-smp-m3")]
        {
            smp::eoi();
            state.cpu_ticks[cpu()] += 1;
        }
    } else if cfg!(feature = "native-smp-m3") && vector == 0xe8 {
        #[cfg(feature = "native-smp-m3")]
        {
            smp::m3_record_reschedule();
            smp::eoi();
        }
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
            #[cfg(feature = "native-smp-m3-ipc")]
            assert!(m4::engine().check_invariants());
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
                #[cfg(feature = "test-smp-m3-canary")]
                if current == 1 && state.migration_stage == 1 && args[0] == 0xe0 {
                    serial::write(b"SMP:MIGRATE:CANARY:CORRUPT:CONTROL\n");
                    serial::exit(0x10);
                }
                #[cfg(all(feature = "native-smp-m3", not(feature = "native-supervisor")))]
                {
                    let mut line = serial::Line::new();
                    line.push(b"SMP:M3:EXIT:PID=")
                        .decimal(state.last_event.pid)
                        .push(b":CPU=")
                        .decimal(cpu() as u64)
                        .push(b":VECTOR=")
                        .decimal(vector)
                        .push(b":CODE=")
                        .decimal(state.last_event.code)
                        .push(b"\n");
                    serial::write(line.as_bytes());
                    #[cfg(feature = "native-smp-m3-ipc")]
                    if matches!(current, 1 | 2) {
                        let _address = ASPACE[current].lock_irqsave();
                        print_decimal(
                            b"SMP:IPC:LAST_SYSCALL_RESULT=",
                            read_word(state.processes[current].physical(DATA + 144)),
                        );
                        print_decimal(
                            b"SMP:IPC:PROBE_STAGE=",
                            read_word(state.processes[current].physical(DATA + 160)),
                        );
                        let mut registers = serial::Line::new();
                        registers
                            .push(b"SMP:IPC:GPR:RBX=")
                            .decimal(context[11])
                            .push(b":RBP=")
                            .decimal(context[10])
                            .push(b":R12=")
                            .decimal(context[3])
                            .push(b":R13=")
                            .decimal(context[2])
                            .push(b":R14=")
                            .decimal(context[1])
                            .push(b":R15=")
                            .decimal(context[0])
                            .push(b"\n");
                        serial::write(registers.as_bytes());
                    }
                }
                let counter = {
                    #[cfg(feature = "native-smp-m3")]
                    let _address = ASPACE[current].lock_irqsave();
                    read_word(state.processes[current].physical(DATA + 8))
                };
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
        let counter = {
            #[cfg(feature = "native-smp-m3")]
            let _address = ASPACE[current].lock_irqsave();
            read_word(state.processes[current].physical(DATA + 8))
        };
        #[cfg(all(feature = "native-smp-m3", not(feature = "native-supervisor")))]
        {
            #[cfg(feature = "native-smp-m3-unmap")]
            let expected_unmap =
                smp_unmap::expected_fault(state, current, vector, context[16], cr2);
            #[cfg(not(feature = "native-smp-m3-unmap"))]
            let expected_unmap = false;
            if !expected_unmap {
                smp_fault(state, current, vector, context[16]);
            }
        }
        #[cfg(any(not(feature = "native-smp-m3"), feature = "native-supervisor"))]
        state.observer_fault(current, &context, counter);
        #[cfg(feature = "native-smp-m3")]
        let _ = counter;
        state.fault = true;
        terminate = true;
    }
    if terminate {
        assert_eq!(state.last_event.pid, state.processes[current].pid);
    }
    if terminate {
        state.processes[current].status = Status::Dying;
        state.cleanup(current);
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
        #[cfg(feature = "native-smp-m3")]
        if state.processes[current].kill_wait.is_some() {
            state.processes[current].status = Status::Waiting;
        }
        #[cfg(feature = "native-smp-m3")]
        if state.processes[current].status == Status::Waiting {
            sched::block_prepare(&SCHED_STATE[current], cpu()).expect("block running process");
            assert!(!sched::block_commit(&SCHED_STATE[current]).expect("commit blocked process"));
        } else {
            sched::yield_cpu(&SCHED_STATE[current], cpu()).expect("yield running process");
            state.enqueue(current);
        }
    }
    #[cfg(feature = "native-smp-m3-migrate")]
    if vector == 128 && context[14] == 1 {
        state.migrate_fixture(current, entry_active);
    }
    if let Some(next) = state.progress_observers() {
        return next;
    }
    #[cfg(all(
        feature = "native-smp-m3",
        not(any(
            feature = "native-supervisor",
            feature = "native-smp-m4-quotas",
            feature = "native-smp-qsim"
        ))
    ))]
    smp_progress(state);
    #[cfg(all(
        not(feature = "milestone-m4"),
        not(feature = "native-fpu"),
        not(feature = "native-memory"),
        not(feature = "native-smp-m3")
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
            return state.select();
        }
        let survivor = &state.processes[state.survivor_slot];
        assert_eq!(read_word(survivor.physical(DATA + 16)), survivor.pattern);
        if state.ticks >= 4 && read_word(survivor.physical(DATA + 8)) > state.survivor_start + 8 {
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
    #[cfg(feature = "native-smp-m3")]
    let next = {
        // Use the original syscall number in the saved entry context, not its
        // result in process.context[14]. Yield/exit/block never continue. A
        // candidate is consumed and revalidated after kill/wait refresh above.
        let candidate = (vector == 128
            && !matches!(context[14], 1 | 2)
            && !terminate
            && state.current == current
            && state.processes[current].status == Status::Ready
            && state.processes[current].home == cpu()
            && state.processes[current].kill_wait.is_none())
        .then_some(current);
        state.select_with_continuation(candidate)
    };
    #[cfg(not(feature = "native-smp-m3"))]
    let next = state.select();
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
    any(
        feature = "native-fpu",
        feature = "native-memory",
        feature = "native-smp-m3"
    ),
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

// ---- K06 step 5d-2 (ADR-081): the base M3 manifest on every online CPU ----

/// Revoke authority once even when physical storage remains quarantined.
fn retire_authority(_slot: usize, _pid: u64) {
    assert!(lock::kernel_held());
    #[cfg(feature = "display-input")]
    {
        display::detach(_pid);
        ps2::detach(_pid);
    }
    #[cfg(feature = "milestone-m5")]
    m5::detach(_pid);
    #[cfg(feature = "native-keyring")]
    keyring::detach(_pid);
    #[cfg(feature = "native-system-net")]
    network::detach(_pid);
    #[cfg(feature = "milestone-m4")]
    if !cfg!(feature = "native-smp-m3-ipc")
        || cfg!(feature = "native-smp-m3-soak") && matches!(_slot, 8 | 9)
        || !cfg!(feature = "native-smp-m3-soak") && matches!(_slot, 1 | 2)
    {
        m4::process_exit(_pid);
    }
}

/// Finish a diagnostic workload before its existing single-CPU cleanup
/// oracle runs. No STATE reference is used while KERNEL is released.
#[cfg(any(feature = "native-smp-m4-quotas", feature = "native-smp-qsim"))]
fn smp_stop_for_cleanup() -> &'static mut State {
    assert!(lock::kernel_held());
    smp::m3_fairness_report();
    smp::m3_stop_begin();
    kernel_leave();
    let stopped = smp::m3_stop_wait();
    kernel_enter();
    if !stopped {
        serial::fail(b"SMP_M3_STOP_TIMEOUT");
    }
    // SAFETY: KERNEL held and all peer CPUs parked permanently on kernel
    // CR3; this fresh borrow is the only live access to initialized STATE.
    let state = unsafe { (&mut *ptr::addr_of_mut!(STATE)).assume_init_mut() };
    arch::activate(state.kernel.root);
    for index in 0..4 {
        assert!(state.cpu_ticks[index] > 0 && state.cpu_switches[index] >= 1);
        let mut line = serial::Line::new();
        line.push(b"SMP:M3:CPU=")
            .decimal(index as u64)
            .push(b":USER_TICKS=")
            .decimal(state.cpu_ticks[index])
            .push(b":SWITCHES=")
            .decimal(state.cpu_switches[index])
            .push(b"\n");
        serial::write(line.as_bytes());
    }
    for queue in &RUNQ {
        while queue.lock_irqsave().pop(&QUEUED).is_some() {}
    }
    for slot in 0..state.amount {
        state.processes[slot].active = 0;
        ON_CPU[slot].store(false, Ordering::Release);
    }
    state
}

#[cfg(feature = "native-smp-m3")]
impl State {
    #[cfg(feature = "native-smp-m3-migrate")]
    fn migrate_fixture(&mut self, slot: usize, entry_active: u64) {
        if slot != 1
            || self.processes[slot].status != Status::Ready
            || self.processes[slot].selected < 16
        {
            return;
        }
        assert!(
            smp::online_mask() & 7 == 7,
            "migration requires CPUs 0, 1 and 2"
        );
        let source = self.processes[slot].home;
        if self.migration_stage == 0 && self.cpu_switches[source] < 1000 {
            return;
        }
        let progress = {
            let _address = ASPACE[slot].lock_irqsave();
            read_word(self.processes[slot].physical(DATA + 8))
        };
        if self.migration_stage != 0 && progress.saturating_sub(self.migration_progress) < 2 {
            return;
        }
        if matches!((self.migration_stage, source), (1, 2) | (2, 0)) {
            assert_eq!(entry_active, 1 << source);
            let mut line = serial::Line::new();
            line.push(b"SMP:MIGRATE:ACTIVE:CPU=")
                .decimal(source as u64)
                .push(b":MASK=")
                .decimal(entry_active)
                .push(b"\n");
            serial::write(line.as_bytes());
        }
        let target = match (self.migration_stage, source) {
            (0, 1) => 2,
            (1, 2) => 0,
            (2, 0) => {
                // A real user entry after the second restore has checked the
                // full GPR and x87/SSE canaries in probe mode 19.
                self.migration_stage = 3;
                serial::write(b"SMP:MIGRATE:CANARY:PASS\n");
                return;
            }
            _ => return,
        };
        assert_eq!(SCHED_STATE[slot].load(Ordering::Acquire), sched::READY);
        assert!(!ON_CPU[slot].load(Ordering::Acquire));
        assert_eq!(self.processes[slot].active, 0);
        // SAFETY: all CPU_CURRENT publications are serialized by KERNEL.
        for owner in unsafe { &*ptr::addr_of!(CPU_CURRENT) } {
            assert_ne!(*owner, slot);
        }
        let context = self.processes[slot].context;
        let fpu_before = [
            fpu::image_qword(slot, 32),
            fpu::image_qword(slot, 160),
            fpu::image_qword(slot, 400),
        ];
        let (mut low, mut high) = RUNQ[source].lock_pair(&RUNQ[target]);
        let (from, to) = if source < target {
            (&mut *low, &mut *high)
        } else {
            (&mut *high, &mut *low)
        };
        let mut retained = [0u32; MAX];
        let mut count = 0;
        let mut found = false;
        while let Some(ready) = from.pop(&QUEUED) {
            if ready as usize == slot {
                assert!(!found);
                found = true;
            } else {
                retained[count] = ready;
                count += 1;
            }
        }
        assert!(found, "migration must remove the source queue owner");
        for ready in &retained[..count] {
            from.push(*ready, &QUEUED).unwrap();
        }
        self.processes[slot].home = target;
        to.push(slot as u32, &QUEUED)
            .expect("migration destination has exclusive ownership");
        drop(high);
        drop(low);
        assert_eq!(self.processes[slot].context, context);
        assert_eq!(
            [
                fpu::image_qword(slot, 32),
                fpu::image_qword(slot, 160),
                fpu::image_qword(slot, 400)
            ],
            fpu_before
        );
        #[cfg(feature = "test-smp-m3-canary")]
        if self.migration_stage == 0 {
            self.processes[slot].context[11] ^= 1 << 48;
        }
        self.migration_stage += 1;
        self.migration_progress = progress;
        let mut line = serial::Line::new();
        line.push(b"SMP:MIGRATE:FROM=")
            .decimal(source as u64)
            .push(b":TO=")
            .decimal(target as u64)
            .push(b"\n");
        serial::write(line.as_bytes());
        assert!(smp::m3_reschedule(target));
    }

    fn enqueue(&mut self, slot: usize) {
        assert!(lock::kernel_held());
        assert!(!ON_CPU[slot].load(Ordering::Acquire));
        RUNQ[self.processes[slot].home]
            .lock_irqsave()
            .push(slot as u32, &QUEUED)
            .expect("single RUNQ ownership");
    }

    fn wake_ready(&mut self, slot: usize) {
        assert!(sched::wake(&SCHED_STATE[slot]).expect("wake blocked process"));
        self.processes[slot].status = Status::Ready;
        self.enqueue(slot);
    }

    /// R3-2 / spec §6.2: the online CPU with the fewest live processes, the
    /// lowest index on a tie. Called under KERNEL during setup only.
    /// A remote kill blocks its caller until the target acknowledges on entry.
    /// The normal scheduling path releases KERNEL, allowing the target to enter.
    #[cfg(any(
        feature = "native-supervisor",
        feature = "native-smp-m3-kill",
        feature = "test-smp-m3-recovery"
    ))]
    fn retire_process(&mut self, target: usize, requester: Option<usize>) {
        const KILL_ACK_MS: u64 = 100;
        assert!(lock::kernel_held());
        let previous = sched::kill_request(&SCHED_STATE[target]).expect("kill live process");
        self.processes[target].status = Status::Dying;
        if !QUEUED[target].load(Ordering::Acquire) && !ON_CPU[target].load(Ordering::Acquire) {
            self.cleanup(target);
            return;
        }
        assert!(previous == sched::READY || previous >= sched::running(0));
        if let Some(slot) = requester {
            assert_ne!(slot, target);
            self.processes[slot].kill_wait = Some((
                self.processes[target].pid,
                smp::m3_time_ms().saturating_add(KILL_ACK_MS),
            ));
        }
        if !smp::m3_reschedule(self.processes[target].home) {
            serial::fail(b"SMP_M3_KILL_IPI");
        }
    }

    fn refresh_kills(&mut self) {
        for slot in 0..self.amount {
            let Some((target, deadline)) = self.processes[slot].kill_wait else {
                continue;
            };
            if self.processes[..self.amount]
                .iter()
                .any(|p| p.pid == target && p.status != Status::Dead && !p.quarantined)
            {
                if smp::m3_time_ms() >= deadline {
                    smp::m3_request_recovery();
                    return;
                }
                continue;
            }
            self.processes[slot].kill_wait = None;
            self.wake_ready(slot);
            let home = self.processes[slot].home;
            if home != cpu() && !smp::m3_reschedule(home) {
                serial::fail(b"SMP_M3_KILL_WAKE");
            }
        }
    }

    fn pick_home(&self) -> usize {
        let online = smp::online_mask();
        (0..smp::MAX_CPUS)
            .filter(|cpu| online & (1 << cpu) != 0)
            .min_by_key(|&cpu| {
                let load = self
                    .processes
                    .iter()
                    .filter(|p| p.status != Status::Dead && p.home == cpu)
                    .count();
                (load, cpu)
            })
            .expect("the BSP is always online")
    }
}

/// A full kernel-half round to every other online CPU (spec §5 item 6). The
/// caller has cleared the PTEs and frees the frames only after it returns.
#[cfg(feature = "native-smp-m3")]
fn kernel_round() -> bool {
    // More than 32 pages: every target reloads CR3 (5e, PS-2).
    if smp::shootdown_kernel(0, 512).is_err() {
        serial::write(b"SMP:M3:SHOOTDOWN_TIMEOUT\n");
        smp::m3_request_recovery();
        return false;
    }
    true
}

/// AP entry after M3_GO (from its home loop, which is never resumed).
#[cfg(feature = "native-smp-m3")]
pub fn ap_main() -> ! {
    kernel_enter();
    let state = load_current();
    resume(state.select())
}

/// KERNEL held, no process of this CPU is ready. Idle with KERNEL released
/// (INV-7), so the other CPUs keep scheduling, then look again. Callers
/// hold a `&mut State` they never use again (this function diverges).
#[cfg(feature = "native-smp-m3")]
fn smp_idle() -> ! {
    loop {
        {
            // SAFETY: KERNEL held (K-1); the only live use of STATE.
            let state = unsafe { (&mut *ptr::addr_of_mut!(STATE)).assume_init_mut() };
            state.current = MAX;
            arch::activate(state.kernel.root);
            set_rsp0(service_stack());
        }
        kernel_leave();
        // SAFETY: no lock held, kernel CR3, this CPU's service stack. Every
        // unmasked source has a gate (BSP: PIT IRQ0 counted by m2_dispatch;
        // AP: the M3 0xd0 stub EOIs), STI's shadow covers HLT.
        unsafe { asm!("sti", "hlt", "cli", options(nostack)) };
        kernel_enter();
        let state = load_current();
        #[cfg(feature = "ipc-wait")]
        {
            state.wait_metrics.idle_irqs = state.wait_metrics.idle_irqs.saturating_add(1);
        }
        state.refresh_kills();
        #[cfg(feature = "ipc-wait")]
        wait::refresh(state);
        if let Some(next) = state.progress_observers() {
            resume(next);
        }
        if !RUNQ[cpu()].lock_irqsave().is_empty() {
            resume(state.select());
        }
    }
}

/// Two separately owned endpoint queues; only SEND crosses to the peer.
#[cfg(feature = "native-smp-m3-ipc")]
fn smp_ipc_manifest(state: &mut State) {
    use capability_core::{Message, RECV, SEND};
    let first = if cfg!(feature = "native-smp-m3-soak") {
        8
    } else {
        1
    };
    let second = first + 1;
    assert!(
        smp::online_mask().count_ones() >= 2 || cfg!(feature = "native-smp-m3-load"),
        "IPC requires distinct CPUs outside the one-CPU load regression"
    );
    if smp::online_mask().count_ones() >= 2 {
        assert_ne!(state.processes[first].home, state.processes[second].home);
    }
    // Only the IPC pair needs capability-engine membership. The other load
    // workers exercise ring-3 CPU demand without changing engine quotas.
    for p in &state.processes[first..=second] {
        m4::engine().register(p.pid).unwrap();
    }
    let roots = [
        m4::engine()
            .create_endpoint(state.processes[first].pid)
            .unwrap(),
        m4::engine()
            .create_endpoint(state.processes[second].pid)
            .unwrap(),
    ];
    // The bootstrap updates both spaces, so acquire them by the established
    // stable key order; release in reverse order before any scheduling.
    let (_low, _high) = ASPACE[first].lock_pair(&ASPACE[second]);
    for (role, slot) in [first, second].into_iter().enumerate() {
        let peer = 1 - role;
        let send = m4::engine()
            .bootstrap_delegate(
                state.processes[first + peer].pid,
                roots[peer],
                state.processes[slot].pid,
                SEND,
            )
            .unwrap();
        for (offset, value) in [
            (64, send),
            (72, roots[role]),
            (80, role as u64),
            (
                88,
                if cfg!(feature = "native-smp-m3-load") {
                    u64::MAX - 1
                } else {
                    10_000
                },
            ),
            (96, 0),
            (104, 0),
            (112, 0),
            (120, 1),
            (128, 0),
        ] {
            write_word(state.processes[slot].physical(DATA + offset), value);
        }
        let mut message = Message::new();
        message.len = 8;
        let plan = m4::plan(state, slot, HEAP, 320, true).unwrap();
        m4::write_plan(&plan, &message.encode());
        write_word(state.processes[slot].physical(HEAP + 512), roots[role]);
        write_word(state.processes[slot].physical(HEAP + 520), RECV);
    }
    assert!(m4::engine().check_invariants());
}

#[cfg(feature = "native-smp-m3-ipc")]
fn smp_ipc_done(state: &State) -> bool {
    #[cfg(feature = "native-smp-m3-soak")]
    if unsafe { !SOAK_IPC_READY } {
        return false;
    }
    // Engine mutations are audited at the completed syscall/cleanup/bootstrap
    // boundaries. This observer still reads actual per-process IPC progress.
    let first = if cfg!(feature = "native-smp-m3-soak") {
        8
    } else {
        1
    };
    [first, first + 1].into_iter().all(|slot| {
        let _address = ASPACE[slot].lock_irqsave();
        let p = &state.processes[slot];
        let rounds = read_word(p.physical(DATA + 96));
        if cfg!(feature = "native-smp-m3-load") {
            // Duration controls this profile. Both endpoints keep validating
            // sequence numbers throughout the load rather than going idle
            // after a fixed round count.
            rounds > 0
        } else {
            assert!(rounds <= 10_000);
            rounds == 10_000 && read_word(p.physical(DATA + 104)) == 1
        }
    }) && (smp::online_mask().count_ones() == 1 || state.wait_metrics.remote_wakes > 0)
}

/// An app fault in the SMP manifest. Only the deliberate negative case (ud2
/// in slot 2 on AP2) is expected; it stops just that process (K03).
#[cfg(feature = "native-smp-m3")]
fn smp_fault(state: &mut State, slot: usize, vector: u64, error: u64) {
    // `int3` from CPL3 is #BP, or #GP with the IDT selector 3*8+2 when gate 3
    // is DPL0 (the M2 exception gates are).
    if vector == 3 || vector == 13 && error == 3 * 8 + 2 {
        // The FPU canary saw another process's or a lost x87/SSE state.
        let mut line = serial::Line::new();
        line.push(b"SMP:M3:FPU:CORRUPT")
            .push(if cfg!(feature = "test-smp-m3-fpu-skip") {
                b":CONTROL"
            } else {
                b""
            })
            .push(b"\n");
        serial::write(line.as_bytes());
        if cfg!(feature = "test-smp-m3-fpu-skip") {
            serial::exit(0x10);
        }
    }
    let expected = cfg!(feature = "test-smp-m3-fault")
        && slot == 2
        && vector == 6
        && cpu() == 2
        && state.processes[slot].home == 2;
    state.unexpected |= !expected;
    if !expected {
        let mut line = serial::Line::new();
        line.push(b"SMP:M3:FAULT:VECTOR=")
            .decimal(vector)
            .push(b":CPU=")
            .decimal(cpu() as u64)
            .push(b"\n");
        serial::write(line.as_bytes());
    }
    if expected {
        print_decimal(b"SMP:M3:FAULT:UD:CPU=", cpu() as u64);
    }
}

/// Every event, KERNEL held: once each online CPU ran ring 3 (M3 ticks from
/// CPL3) and switched processes at least 4 times, every live process made
/// progress with its pattern intact and (negative case) the fault happened,
/// stop the other CPUs, check cleanup balance and end the VM.
#[cfg(feature = "native-smp-m3")]
fn smp_progress(state: &mut State) {
    #[cfg(feature = "test-smp-m3-recovery")]
    let recovery_done = smp_recovery::poll(state);
    #[cfg(not(feature = "test-smp-m3-recovery"))]
    let recovery_done = true;
    #[cfg(feature = "native-smp-m3-kill")]
    let lifecycle_done = smp_lifecycle::poll(state);
    #[cfg(not(feature = "native-smp-m3-kill"))]
    let lifecycle_done = true;
    // Spec 08 §4: the FPU canary must survive 1000 switches on every CPU.
    const MIN_SWITCHES: u64 = 1000;
    let live = |p: &&Process| p.status != Status::Dead && !p.quarantined;
    let online = smp::online_mask();
    let cpus = || (0..smp::MAX_CPUS).filter(move |cpu| online & (1 << cpu) != 0);
    #[cfg(feature = "native-smp-m3-ipc")]
    let ipc_done = smp_ipc_done(state);
    #[cfg(not(feature = "native-smp-m3-ipc"))]
    let ipc_done = true;
    #[cfg(feature = "native-smp-m3-soak")]
    let k07_done = m4::soak_done() && qsim::soak_done();
    #[cfg(not(feature = "native-smp-m3-soak"))]
    let k07_done = true;
    let done = ipc_done
        && lifecycle_done
        && recovery_done
        && k07_done
        && {
            #[cfg(feature = "native-smp-m3-load")]
            {
                smp::m3_time_ms().saturating_sub(state.load_started_ms)
                    >= if cfg!(feature = "native-smp-m3-soak") {
                        600_000
                    } else {
                        30_000
                    }
            }
            #[cfg(not(feature = "native-smp-m3-load"))]
            {
                true
            }
        }
        && {
            #[cfg(feature = "native-smp-m3-migrate")]
            {
                state.migration_stage == 3
            }
            #[cfg(not(feature = "native-smp-m3-migrate"))]
            {
                true
            }
        }
        && state.ticks >= 64
        && cpus().all(|cpu| state.cpu_ticks[cpu] > 0 && state.cpu_switches[cpu] >= MIN_SWITCHES)
        && state.processes[..state.amount]
            .iter()
            .enumerate()
            .filter(|(_, p)| live(p))
            .all(|(slot, p)| {
                let _address = ASPACE[slot].lock_irqsave();
                p.selected >= 4
                    && read_word(p.physical(DATA + 8)) > 0
                    // Loop progress precedes FPU initialization. Do not stop
                    // a live canary in its first, timer-interrupted setup.
                    // The soak's M4/qsim slots are not this canary fixture.
                    && (cfg!(feature = "native-smp-m3-soak") && slot < 8
                        || read_word(p.physical(DATA + 128)) == 1)
            })
        && (!cfg!(feature = "test-smp-m3-fault") || state.fault);
    if !done && !state.unexpected {
        return;
    }
    #[cfg(feature = "native-smp-m3-load")]
    {
        let mut line = serial::Line::new();
        line.push(b"SMP:LOAD:WORKERS=")
            .decimal((state.amount - 2) as u64)
            .push(b":ELAPSED_MS=")
            .decimal(smp::m3_time_ms().saturating_sub(state.load_started_ms))
            .push(b"\n");
        serial::write(line.as_bytes());
    }
    #[cfg(any(feature = "native-smp-m3-ipc", feature = "native-smp-m3-migrate"))]
    smp::m3_fairness_report();
    #[cfg(feature = "native-smp-m3-ipc")]
    let received_reschedules = smp::m3_received_reschedules();
    #[cfg(feature = "test-smp-m3-canary")]
    serial::fail(b"SMP_MIGRATE_CANARY_NOT_DETECTED");
    // The flag goes up under KERNEL, so this CPU is the only stopper. It
    // then releases KERNEL so the others can park at their next entry; they
    // touch nothing of STATE before parking (`load_current`).
    smp::m3_stop_begin();
    kernel_leave();
    let stopped = smp::m3_stop_wait();
    kernel_enter();
    if !stopped {
        serial::write(b"SMP:M3:STOP:TIMEOUT\n");
        serial::fail(b"SMP_M3_STOP_TIMEOUT");
    }
    // SAFETY: KERNEL held again and every other CPU is halted for good.
    let state = unsafe { (&mut *ptr::addr_of_mut!(STATE)).assume_init_mut() };
    if state.unexpected {
        serial::write(b"M3:FAIL:UNEXPECTED_USER_EVENT\n");
        serial::exit(0x11);
    }
    arch::activate(state.kernel.root);
    // Every peer is quiescent. Retain a complete final isolation scan before
    // any cleanup, in addition to each process's own user and entry checks.
    for (slot, p) in state.processes[..state.amount].iter().enumerate() {
        if p.status != Status::Dead && !p.quarantined {
            let _address = ASPACE[slot].lock_irqsave();
            assert_eq!(read_word(p.physical(DATA + 16)), p.pattern);
        }
    }
    #[cfg(feature = "native-smp-m3-ipc")]
    {
        assert!(m4::engine().check_invariants());
        assert!(smp_ipc_done(state));
        serial::write(b"SMP:IPC:PASS\n");
        print_decimal(b"SMP:IPC:REMOTE_WAKEUPS=", state.wait_metrics.remote_wakes);
        assert!(online.count_ones() == 1 || received_reschedules > 0);
        print_decimal(b"SMP:IPC:RESCHEDULE_IPIS=", received_reschedules);
    }
    for cpu in cpus() {
        let mut line = serial::Line::new();
        line.push(b"SMP:M3:CPU=")
            .decimal(cpu as u64)
            .push(b":USER_TICKS=")
            .decimal(state.cpu_ticks[cpu])
            .push(b":SWITCHES=")
            .decimal(state.cpu_switches[cpu])
            .push(b"\n");
        serial::write(line.as_bytes());
    }
    print_decimal(b"SMP:SCHED:CPUS=", u64::from(online.count_ones()));
    smp_fpu_evidence(state);
    for queue in &RUNQ {
        while queue.lock_irqsave().pop(&QUEUED).is_some() {}
    }
    for slot in 0..state.amount {
        if state.processes[slot].status != Status::Dead {
            // The parked CPUs loaded the kernel CR3 before halting.
            state.processes[slot].active = 0;
            ON_CPU[slot].store(false, Ordering::Release);
            state.processes[slot].status = Status::Dying;
            state.cleanup(slot);
        }
    }
    #[cfg(feature = "test-smp-m3-recovery")]
    let retained: usize = (0..state.amount)
        .filter(|slot| state.processes[*slot].quarantined)
        .map(|slot| smp_recovery::audit_quarantined(state, slot))
        .sum();
    #[cfg(not(feature = "test-smp-m3-recovery"))]
    let retained = 0;
    assert_eq!(state.free_count() + retained, state.free_baseline);
    serial::write(b"SMP:M3:CLEANUP:PASS\n");
    if cfg!(feature = "test-smp-m3-fault") {
        serial::write(b"SMP:M3:K03:PASS\n");
    }
    print_decimal(b"SMP:M3:PASS:CPUS=", u64::from(online.count_ones()));
    serial::write(b"M1:PASS\n");
    serial::exit(0x10)
}

#[cfg(feature = "native-smp-m3")]
fn print_decimal(label: &[u8], value: u64) {
    let mut line = serial::Line::new();
    line.push(label).decimal(value).push(b"\n");
    serial::write(line.as_bytes());
}

/// R3-3 evidence after the stop, before cleanup: each live canary's last
/// FXSAVE image (written by its own CPU) holds its own ST0, XMM0 and XMM15;
/// every CPU saved and restored images.
#[cfg(feature = "native-smp-m3")]
fn smp_fpu_evidence(state: &State) {
    // ST0's 64-bit significand: the pattern (bit 62 set) normalized by one.
    const ST0: usize = 32;
    const XMM0: usize = 160;
    const XMM15: usize = 160 + 15 * 16;
    let mut checked = 0;
    for (slot, p) in state.processes[..state.amount].iter().enumerate() {
        if p.status == Status::Dead
            || p.quarantined
            || cfg!(feature = "native-smp-m3-soak") && slot < 8
        {
            continue;
        }
        if fpu::image_qword(slot, XMM0) != p.pattern
            || fpu::image_qword(slot, XMM15) != !p.pattern
            || fpu::image_qword(slot, ST0) != p.pattern << 1
        {
            let header = fpu::image_qword(slot, 0);
            print_decimal(b"SMP:M3:FPU:IMAGE:SLOT=", slot as u64);
            print_decimal(b"SMP:M3:FPU:IMAGE:PATTERN=", p.pattern);
            print_decimal(b"SMP:M3:FPU:IMAGE:ST0=", fpu::image_qword(slot, ST0));
            print_decimal(b"SMP:M3:FPU:IMAGE:XMM0=", fpu::image_qword(slot, XMM0));
            print_decimal(b"SMP:M3:FPU:IMAGE:XMM15=", fpu::image_qword(slot, XMM15));
            print_decimal(b"SMP:M3:FPU:IMAGE:TOP=", (header >> 27) & 7);
            print_decimal(b"SMP:M3:FPU:IMAGE:TAG=", (header >> 32) & 0xff);
            print_decimal(b"SMP:M3:FPU:IMAGE:SAVED_RIP=", p.context[17]);
            serial::write(b"SMP:M3:FPU:IMAGE_MISMATCH\n");
            serial::fail(b"SMP_M3_FPU_IMAGE");
        }
        checked += 1;
    }
    let (saves, restores) = fpu::counts();
    assert!(saves > 0 && restores > 0);
    print_decimal(b"SMP:M3:FPU:PASS:PROCESSES=", checked);
}

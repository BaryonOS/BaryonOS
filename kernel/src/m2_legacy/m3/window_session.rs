//! U01 fixed isolated window-service group. User init owns readiness, pause,
//! rebind and restart policy. The kernel owns only bounded process/capability
//! construction and teardown; it never renders, authenticates or restarts from
//! an observer. All callbacks run on one CPU with IF=0.
use super::*;
use capability_core::{
    BlockScope, Engine, Error, BLOCK_FLUSH, BLOCK_READ, BLOCK_WRITE, PROCESS_EXITED,
    PROCESS_FAULTED, PROCESS_KILLED, RECV, SEND,
};
#[path = "window_retirement.rs"]
mod retirement;

const INIT: usize = 0;
const KEY: usize = 1;
const VOLUME: usize = 2;
const MATERIAL: usize = 3;
const COMP: usize = 4;
const SOURCE: usize = 5;
const POLICY: usize = 6;
const CALC: usize = 7;
const APPS: [usize; 2] = [MATERIAL, CALC];
const GROUP: [usize; 3] = [COMP, SOURCE, POLICY];
const GROUP_IMAGE: u32 = 0x5749_4e31;
const SURFACE_EPOCH: u64 = 0x5355_5200_0000_0001;
const INPUT_EPOCH: u64 = 0x494e_5000_0000_0001;
// Includes a temporary endpoint root during construction. No budget depends on
// reusing nodes/frames reclaimed by teardown.
const GROUP_NODES: usize = 48;
const GROUP_ENDPOINTS: usize = 12;
const SPAWN_FRAME_MARGIN: usize = 64;

#[derive(Clone, Copy)]
struct Runtime {
    generation: u64,
    surface_epoch: u64,
    input_epoch: u64,
    registry_root: u64,
    group_launch: u64,
    compositor_process: u64,
    compositor_control: u64,
    presentation_retired: bool,
    // Actual kernel-issued identities. Startup pages are writable user data
    // and cannot choose what the kernel closes or whom it terminates.
    pids: [u64; 3],
    app_graphics: [[u64; 4]; 2],
    baseline: usize,
    kernel_tables: usize,
    init_done: bool,
    stop_tick: u64,
    sample_tick: u64,
    last_mask: u64,
    last_recovery: u64,
    restarts: u64,
    #[cfg(feature = "native-window-session-probe")]
    app_generations: [u64; 2],
    #[cfg(feature = "native-window-session-probe")]
    compositor_commits: u64,
    #[cfg(feature = "native-window-session-probe")]
    compositor_sample_tick: u64,
    #[cfg(feature = "native-window-session-probe")]
    compositor_activity: [u64; 6],
    #[cfg(feature = "native-window-session-probe")]
    input_progress: [[u64; 2]; 2],
}
static mut RUNTIME: Runtime = Runtime {
    generation: 1,
    surface_epoch: SURFACE_EPOCH,
    input_epoch: INPUT_EPOCH,
    registry_root: 0,
    group_launch: 0,
    compositor_process: 0,
    compositor_control: 0,
    presentation_retired: false,
    pids: [0; 3],
    app_graphics: [[0; 4]; 2],
    baseline: 0,
    kernel_tables: 0,
    init_done: false,
    stop_tick: 0,
    sample_tick: u64::MAX,
    last_mask: u64::MAX,
    last_recovery: u64::MAX,
    restarts: 0,
    #[cfg(feature = "native-window-session-probe")]
    app_generations: [u64::MAX; 2],
    #[cfg(feature = "native-window-session-probe")]
    compositor_commits: u64::MAX,
    #[cfg(feature = "native-window-session-probe")]
    compositor_sample_tick: u64::MAX,
    #[cfg(feature = "native-window-session-probe")]
    compositor_activity: [u64::MAX; 6],
    #[cfg(feature = "native-window-session-probe")]
    input_progress: [[u64::MAX; 2]; 2],
};
fn runtime() -> &'static mut Runtime {
    // SAFETY: Single CPU, IF=0, nonreentrant scheduler/syscall callbacks. Every
    // borrow ends before cleanup, a nested accessor or scheduling. No user
    // pointer/reference is retained in this kernel-owned bookkeeping.
    unsafe { &mut *ptr::addr_of_mut!(RUNTIME) }
}
fn fail(reason: &[u8]) -> ! {
    serial::write(b"WINDOW:FAIL:");
    serial::write(reason);
    serial::write(b"\n");
    serial::exit(0x11)
}
fn require(ok: bool, reason: &[u8]) {
    if !ok {
        fail(reason)
    }
}
fn put(s: &State, slot: usize, offset: u64, value: u64) {
    write_word(s.processes[slot].physical(DATA + offset), value);
}
fn word(s: &State, slot: usize, offset: u64) -> u64 {
    read_word(s.processes[slot].physical(DATA + offset))
}
pub(super) fn process_elf(slot: usize) -> &'static [u8] {
    match slot {
        INIT => include_bytes!(env!("NOS_WINDOW_INIT_ELF")),
        KEY => KEYRING_SERVICE_ELF,
        VOLUME => VOLUME_SERVICE_ELF,
        MATERIAL => include_bytes!(env!("NOS_MATERIAAL_ELF")),
        COMP => include_bytes!(env!("NOS_WINDOW_COMPOSITOR_ELF")),
        SOURCE => include_bytes!(env!("NOS_INPUT_SOURCE_ELF")),
        POLICY => include_bytes!(env!("NOS_INPUT_POLICY_ELF")),
        CALC => include_bytes!(env!("NOS_CALCULATOR_ELF")),
        _ => panic!("window manifest image outside fixed roles"),
    }
}
fn channel(e: &mut Engine, owner: u64, sender: u64) -> (u64, u64) {
    let root = e
        .create_endpoint(owner)
        .expect("preflighted window endpoint");
    let rx = e.derive(owner, root, RECV).unwrap();
    let tx = e.bootstrap_delegate(owner, root, sender, SEND).unwrap();
    e.close(owner, root).unwrap();
    (rx, tx)
}
fn close_recorded(e: &mut Engine, pid: u64, handle: u64) {
    match e.close(pid, handle) {
        Ok(()) | Err(Error::BadHandle) => {}
        Err(_) => fail(b"RECORDED_HANDLE_CLOSE"),
    }
}

pub(super) fn manifest(s: &mut State) {
    require(
        s.amount == MAX && s.processes[..8].iter().all(|p| p.status != Status::Dead),
        b"BOOT_PROCESSES",
    );
    let [init, key, volume, app, _, _, _, calc] = core::array::from_fn(|i| s.processes[i].pid);
    let e = m4::engine();
    for p in &s.processes[..8] {
        e.register(p.pid).unwrap();
    }
    let registry_root = e.create_endpoint(init).unwrap();
    let registry = e.derive(init, registry_root, RECV).unwrap();
    // Retain the manifest root only in init for replacement group delegations.
    let app_registry = [app, calc].map(|pid| {
        e.bootstrap_delegate(init, registry_root, pid, SEND)
            .unwrap()
    });
    let (material_control, init_material) = channel(e, app, init);
    let (calc_control, init_calc) = channel(e, calc, init);
    let (doc_reply, volume_reply) = channel(e, app, volume);
    let (doc_ingress, app_document) = channel(e, volume, app);
    let root = e.create_endpoint(volume).unwrap();
    let other_doc = e.derive(volume, root, RECV).unwrap();
    e.close(volume, root).unwrap();
    let readonly = cfg!(feature = "test-system-recover");
    let block = e
        .create_block_device_with_rights(
            volume,
            BlockScope {
                device: 0,
                first_sector: 4,
                sector_count: 580,
            },
            if readonly {
                BLOCK_READ
            } else {
                BLOCK_READ | BLOCK_WRITE | BLOCK_FLUSH
            },
        )
        .unwrap();
    let entropy = e.create_entropy(key).unwrap();
    // Unconnected legacy key endpoints stay wholly within the key principal.
    let unused_key_ingress = |e: &mut Engine| {
        let root = e.create_endpoint(key).unwrap();
        let rx = e.derive(key, root, RECV).unwrap();
        e.close(key, root).unwrap();
        rx
    };
    let key_ingress = unused_key_ingress(e);
    let key_other = unused_key_ingress(e);
    let unused_reply = e.create_endpoint(key).unwrap();
    let _unused_receive = e.derive(key, unused_reply, RECV).unwrap();
    let key_reply = e.derive(key, unused_reply, SEND).unwrap();
    e.close(key, unused_reply).unwrap();
    for (at, value) in [
        (32, 1),
        (48, entropy),
        (56, key_ingress),
        (80, key_other),
        (88, key_reply),
        (160, app),
        (176, key),
    ] {
        put(s, KEY, at, value);
    }
    for (at, value) in [
        (32, 1),
        (48, block),
        (56, doc_ingress),
        (80, other_doc),
        (88, volume_reply),
        (72, volume_reply),
        (120, u64::from(readonly)),
        (160, app),
        (168, init),
        (176, volume),
    ] {
        put(s, VOLUME, at, value);
    }
    for (index, slot) in APPS.into_iter().enumerate() {
        for (at, value) in [
            (32, 1),
            (88, app_registry[index]),
            (
                96,
                if index == 0 {
                    material_control
                } else {
                    calc_control
                },
            ),
            (176, s.processes[slot].pid),
            (184, init),
            (648, 0),
            (656, 1),
            (
                664,
                u64::from(cfg!(feature = "native-window-session-probe")),
            ),
        ] {
            put(s, slot, at, value);
        }
    }
    for (at, value) in [
        (72, doc_reply),
        (80, app_document),
        (112, u64::from(readonly)),
        (168, volume),
    ] {
        put(s, MATERIAL, at, value);
    }
    let launch = e.create_launch(init, GROUP_IMAGE).unwrap();
    let process = e.create_process(init, s.processes[COMP].pid).unwrap();
    runtime().registry_root = registry_root;
    runtime().group_launch = launch;
    runtime().compositor_process = process;
    for (at, value) in [
        (32, u64::from_le_bytes(*b"WININIT1")),
        (40, 1),
        (48, registry),
        (56, init_material),
        (64, init_calc),
        (80, app),
        (88, calc),
        (136, launch),
        (144, process),
        (152, registry_root),
    ] {
        put(s, INIT, at, value);
    }
    construct_group(s);
    require(m4::engine().check_invariants(), b"MANIFEST_ENGINE");
    runtime().baseline = s.free_count()
        + s.processes[..8]
            .iter()
            .map(|p| {
                p.count + p.kernel_frames.len() + p.paging.as_ref().unwrap().owned_table_count()
            })
            .sum::<usize>();
    runtime().kernel_tables = s.kernel.owned_table_count();
    serial::write(b"WINDOW:MANIFEST:PASS\n");
}

/// All group leaves are created from manifest roots. Only the compositor has
/// display/raw-policy access; apps share the ingress object but have separate
/// leaf handles and private reply endpoints. Real sender PIDs select clients.
fn construct_group(s: &State) {
    let r = *runtime();
    let init = s.processes[INIT].pid;
    let [comp, source, policy] = GROUP.map(|slot| s.processes[slot].pid);
    let apps = APPS.map(|slot| s.processes[slot].pid);
    let e = m4::engine();
    let registry = [comp, source, policy].map(|pid| {
        e.bootstrap_delegate(init, r.registry_root, pid, SEND)
            .unwrap()
    });
    let root = e.create_endpoint(comp).unwrap();
    let ingress = e.derive(comp, root, RECV).unwrap();
    let control = e.bootstrap_delegate(comp, root, init, SEND).unwrap();
    let app_sends = apps.map(|pid| {
        [
            e.bootstrap_delegate(comp, root, pid, SEND).unwrap(),
            e.bootstrap_delegate(comp, root, pid, SEND).unwrap(),
        ]
    });
    e.close(comp, root).unwrap();
    let app_replies = apps.map(|pid| [channel(e, pid, comp), channel(e, pid, comp)]);
    let (source_rx, source_tx) = channel(e, policy, source);
    let (source_reply_rx, source_reply_tx) = channel(e, source, policy);
    let (focus_rx, focus_tx) = channel(e, policy, comp);
    let (focus_reply_rx, focus_reply_tx) = channel(e, comp, policy);
    let (lifecycle_rx, unused_lifecycle_tx) = channel(e, policy, init);
    // The role contract reserves this RX; init has no lifecycle RPC. Avoid
    // leaking an invalidated SEND leaf in init on every replacement.
    e.close(init, unused_lifecycle_tx).unwrap();
    let (lowlevel_rx, lowlevel_tx) = channel(e, policy, comp);
    let (lowlevel_reply_rx, lowlevel_reply_tx) = channel(e, comp, policy);
    let display = e.create_display(comp).unwrap();
    let input = e.create_input(source).unwrap();
    let scale = if cfg!(feature = "native-window-session-hidpi") {
        2
    } else {
        1
    };
    let geometry = crate::framebuffer::descriptor();
    for (index, slot) in APPS.into_iter().enumerate() {
        let handles = [
            app_sends[index][0],
            app_replies[index][0].0,
            app_sends[index][1],
            app_replies[index][1].0,
        ];
        for (at, value) in [
            (544, handles[0]),
            (552, handles[1]),
            (560, comp),
            (568, r.surface_epoch),
            (576, handles[2]),
            (584, handles[3]),
            (592, comp),
            (600, r.input_epoch),
            (608, index as u64 + 1),
            (616, scale),
            (648, 0),
            (656, 1),
            (
                664,
                u64::from(cfg!(feature = "native-window-session-probe")),
            ),
        ] {
            put(s, slot, at, value);
        }
        runtime().app_graphics[index] = handles;
        for handle in handles {
            require(
                e.check_display(apps[index], handle).is_err()
                    && e.check_input(apps[index], handle).is_err()
                    && e.check_entropy(apps[index], handle).is_err()
                    && e.check_network(apps[index], handle).is_err()
                    && e.check_block(apps[index], handle, BLOCK_READ).is_err()
                    && e.check_launch(apps[index], handle).is_err(),
                b"APP_AUTHORITY",
            );
        }
        let at = 160 + index as u64 * 64;
        for (offset, value) in [
            (0, apps[index]),
            (8, app_replies[index][0].1),
            (16, app_replies[index][1].1),
            (24, r.surface_epoch),
            (32, r.input_epoch),
            (40, index as u64 + 1),
        ] {
            put(s, COMP, at + offset, value);
        }
    }
    for (at, value) in [
        (32, u64::from_le_bytes(*b"WINCOMP1")),
        (40, r.generation),
        (48, display),
        (56, ingress),
        (64, focus_tx),
        (72, focus_reply_rx),
        (80, policy),
        (88, r.input_epoch),
        (96, lowlevel_tx),
        (104, lowlevel_reply_rx),
        (112, registry[0]),
        (120, init),
        (128, comp),
        (136, scale),
    ] {
        put(s, COMP, at, value);
    }
    for (at, value) in [
        (32, u64::from_le_bytes(*b"NOSISRC1")),
        (40, r.generation),
        (48, input),
        (56, source_tx),
        (64, source_reply_rx),
        (72, policy),
        (80, r.input_epoch),
        (88, registry[1]),
        (96, init),
    ] {
        put(s, SOURCE, at, value);
    }
    for (at, value) in [
        (32, u64::from_le_bytes(*b"NOSIPOL1")),
        (40, r.generation),
        (48, source_rx),
        (56, focus_rx),
        (64, lifecycle_rx),
        (72, lowlevel_rx),
        (80, source),
        (88, comp),
        (96, init),
        (104, r.input_epoch),
        (112, u64::from(geometry.width)),
        (120, u64::from(geometry.height)),
        (128, source_reply_tx),
        (136, focus_reply_tx),
        (144, registry[2]),
        (152, registry[2]),
        (160, 1),
        (168, 100),
        (176, comp),
        (184, lowlevel_reply_tx),
    ] {
        put(s, POLICY, at, value);
    }
    for (at, value) in [
        (40, r.generation),
        (72, control),
        (96, comp),
        (104, source),
        (112, policy),
        (120, r.surface_epoch),
        (128, r.input_epoch),
        (144, r.compositor_process),
    ] {
        put(s, INIT, at, value);
    }
    runtime().pids = [comp, source, policy];
    runtime().compositor_control = control;
}

/// Only the exact manifest init + its typed group Launch and dead current
/// compositor Process can request replacement. Old/stale/foreign caps never
/// select a PID, slot, image, root or app channel. All rejection is read-only.
pub(super) fn restart(s: &mut State, slot: usize, a: [u64; 6]) -> (i64, u64) {
    match restart_checked(s, slot, a) {
        Ok(cap) => (0, cap),
        Err(code) => (code, 0),
    }
}
fn restart_checked(s: &mut State, slot: usize, a: [u64; 6]) -> Result<u64, i64> {
    let r = *runtime();
    let init = s.processes[INIT].pid;
    if slot != INIT || s.processes[INIT].status == Status::Dead || r.init_done {
        return Err(-5);
    }
    let authority = retirement::Authority {
        init,
        launch: r.group_launch,
        image: GROUP_IMAGE,
        process: r.compositor_process,
        compositor: r.pids[0],
        apps: core::array::from_fn(|i| (s.processes[APPS[i]].pid, r.app_graphics[i])),
    };
    let e = m4::engine();
    let phase = retirement::authorize(e, init, &authority, a)?;
    if s.processes[COMP].status != Status::Dead {
        return Err(-9);
    }
    if APPS
        .iter()
        .any(|i| matches!(s.processes[*i].status, Status::Dead | Status::Dying))
        || [SOURCE, POLICY].iter().enumerate().any(|(i, slot)| {
            s.processes[*slot].status == Status::Dead || s.processes[*slot].pid != r.pids[i + 1]
        })
    {
        return Err(-9);
    }
    if phase == retirement::Phase::Retire {
        retirement::retire(e, &authority)?;
        runtime().presentation_retired = true;
        print_number(b"WINDOW:PRESENTATION_RETIRED:", r.generation);
        return Ok(0);
    }
    // Init issues this phase only after both real pause acknowledgements.
    // Requiring the prior retirement prevents a blocked old RPC from keeping
    // its caller from ever reaching the control handler.
    if !r.presentation_retired {
        return Err(-9);
    }
    e.check_delegation(init, r.registry_root, SEND)
        .map_err(|v| v.code())?;
    let generation = r.generation.checked_add(1).ok_or(-7)?;
    let surface_epoch = r.surface_epoch.checked_add(1).ok_or(-7)?;
    let input_epoch = r.input_epoch.checked_add(1).ok_or(-7)?;
    let mut next = s.next_pid;
    let mut frames = 0usize;
    for role in GROUP {
        let (_, after) = user::issue_pid(next).map_err(|_| -7)?;
        next = after;
        let parsed = user::parse(process_elf(role)).unwrap_or_else(|_| fail(b"EMBEDDED_IMAGE"));
        frames = frames
            .checked_add(parsed.pages + SPAWN_FRAME_MARGIN)
            .ok_or(-7)?;
    }
    if s.free_count() < frames
        || e.available_processes() < 3
        || e.available_process_objects() < 1
        || e.available_endpoints() < GROUP_ENDPOINTS
        || e.available_nodes() < GROUP_NODES
        || e.available_slots(init) < 3
        || !e.can_create_display_after_exit(r.pids[0])
        || !display::can_bind_fresh()
        || !e.can_create_input_after_exit(r.pids[1])
        || APPS.iter().enumerate().any(|(i, slot)| {
            let pid = s.processes[*slot].pid;
            e.available_slots(pid) < 4
                || !e.can_create_owned_endpoints_after_closing(pid, &r.app_graphics[i], 2, &r.pids)
        })
    {
        return Err(-7);
    }
    // Commit. Fresh identities/epochs are written only after all old group
    // objects are closed. Ordinary allocation failures after complete preflight
    // are kernel bugs and stop the VM; no partial success escapes to init.
    for role in [SOURCE, POLICY] {
        let pid = s.processes[role].pid;
        m4::engine().process_died(pid, PROCESS_KILLED, 0);
        s.processes[role].status = Status::Dying;
        s.cleanup(role);
    }
    let e = m4::engine();
    for (i, role) in APPS.into_iter().enumerate() {
        for handle in r.app_graphics[i] {
            close_recorded(e, s.processes[role].pid, handle);
        }
    }
    close_recorded(e, init, r.compositor_control);
    close_recorded(e, init, r.compositor_process);
    for role in GROUP {
        s.spawn_process_from(role, process_elf(role));
        e.register(s.processes[role].pid).unwrap();
    }
    let process = e.create_process(init, s.processes[COMP].pid).unwrap();
    runtime().generation = generation;
    runtime().surface_epoch = surface_epoch;
    runtime().input_epoch = input_epoch;
    runtime().compositor_process = process;
    runtime().presentation_retired = false;
    #[cfg(feature = "native-window-session-probe")]
    {
        runtime().app_generations = [u64::MAX; 2];
        runtime().compositor_commits = u64::MAX;
        runtime().compositor_sample_tick = u64::MAX;
        runtime().compositor_activity = [u64::MAX; 6];
    }
    runtime().restarts = r
        .restarts
        .checked_add(1)
        .unwrap_or_else(|| fail(b"RESTART_COUNTER"));
    construct_group(s);
    require(m4::engine().check_invariants(), b"RESTART_ENGINE");
    s.audit();
    print_number(b"WINDOW:RESTART_GENERATION:", generation);
    print_number(b"WINDOW:COMPOSITOR_PID:", s.processes[COMP].pid);
    Ok(process)
}

pub(super) fn observe_exit(s: &State, slot: usize, _: u64) {
    print_number(b"WINDOW:EXIT_SLOT:", slot as u64);
    print_number(b"WINDOW:EXIT_PID:", s.processes[slot].pid);
    print_number(b"WINDOW:EXIT_CODE:", s.last_event.code);
    print_number(b"WINDOW:EXIT_STEP:", word(s, slot, 240));
    #[cfg(feature = "native-window-session-probe")]
    if let Some(index) = APPS.iter().position(|candidate| *candidate == slot) {
        // Preserve the final complete app-reported snapshot in the serial
        // transcript before cleanup destroys its private pages.
        runtime().app_generations[index] = u64::MAX;
        snapshots(s);
    }
    m4::engine().process_died(s.processes[slot].pid, PROCESS_EXITED, s.last_event.code);
    match slot {
        INIT => {
            require(
                s.last_event.code == 0 && word(s, INIT, 504) == 1,
                b"INIT_EXIT",
            );
            runtime().init_done = true;
            runtime().stop_tick = s.ticks;
        }
        MATERIAL | CALC => require(s.last_event.code == 0, b"APP_EXIT"),
        COMP => {} // Only init may decide whether/when to replace this group.
        _ => fail(b"CRITICAL_SERVICE_EXIT"),
    }
}
pub(super) fn observe_fault(s: &State, slot: usize, context: &[u64; 22], _: u64) {
    print_number(b"WINDOW:FAULT_SLOT:", slot as u64);
    print_number(b"WINDOW:FAULT_PID:", s.processes[slot].pid);
    print_number(b"WINDOW:FAULT_VECTOR:", context[15]);
    print_number(b"WINDOW:FAULT_RIP:", context[17]);
    // ProcessStatus::Faulted carries the vector, not the exception error code.
    m4::engine().process_died(s.processes[slot].pid, PROCESS_FAULTED, context[15]);
    require(slot == COMP, b"CRITICAL_SERVICE_FAULT");
}
/// Kernel return-layout validation rejected saved user registers before iret.
/// Classify this as a general-protection failure (13) for the typed process
/// ABI, while the distinct diagnostic identifies that no CPU exception ran.
pub(super) fn observe_invalid_return(s: &State, slot: usize) {
    print_number(b"WINDOW:INVALID_RETURN_SLOT:", slot as u64);
    print_number(b"WINDOW:INVALID_RETURN_PID:", s.processes[slot].pid);
    print_number(b"WINDOW:INVALID_RETURN_RIP:", s.last_event.rip);
    m4::engine().process_died(s.processes[slot].pid, PROCESS_FAULTED, 13);
    require(slot == COMP, b"CRITICAL_INVALID_RETURN");
}

/// Diagnostic observations are app-reported counters, never authority, a
/// framebuffer oracle, or evidence that a task has succeeded. No text/keys are
/// copied. The normal product profile omits these detailed synthetic-test logs.
#[cfg(feature = "native-window-session-probe")]
fn snapshots(s: &State) {
    for (index, (slot, generation_at, prefix, fields)) in [
        (
            MATERIAL,
            624,
            b"WINDOW:MATERIAL:".as_slice(),
            &[
                (400, b"READY".as_slice()),
                (624, b"GEN"),
                (408, b"FRAMES"),
                (480, b"SAVE_ACKS"),
                (440, b"REVISION"),
                (448, b"SAVED_BYTES"),
                (456, b"SAVED_SUM"),
                (504, b"SCREEN"),
                (512, b"DIRTY"),
                (520, b"DRAFT_BYTES"),
                (528, b"DRAFT_SUM"),
                (536, b"OBJECT"),
                (632, b"SCALE"),
                (472, b"ERRORS"),
                (672, b"PROBES"),
            ][..],
        ),
        (
            CALC,
            456,
            b"WINDOW:CALCULATOR:".as_slice(),
            &[
                (400, b"READY".as_slice()),
                (456, b"GEN"),
                (408, b"FRAMES"),
                (464, b"EVALUATIONS"),
                (472, b"FAILURES"),
                (480, b"EXPRESSION_BYTES"),
                (680, b"EXPRESSION_SUM"),
                (488, b"CURRENT"),
                (504, b"CARET"),
                (512, b"ACTIVE"),
                (520, b"ANSWER_LOW"),
                (528, b"ANSWER_HIGH"),
                (536, b"HAS_ANSWER"),
                (632, b"SCALE"),
                (448, b"ERRORS"),
                (672, b"PROBES"),
            ][..],
        ),
    ]
    .into_iter()
    .enumerate()
    {
        if s.processes[slot].status == Status::Dead {
            continue;
        }
        let progress = [
            word(s, slot, 424),
            word(s, slot, if index == 0 { 464 } else { 440 }),
        ];
        if progress != runtime().input_progress[index] {
            runtime().input_progress[index] = progress;
            for (label, value) in [
                (b"KEYS".as_slice(), progress[0]),
                (b"INPUT_ACKS".as_slice(), progress[1]),
            ] {
                serial::write(prefix);
                serial::write(label);
                print_number(b":", value);
            }
        }
        if word(s, slot, 400) != 1 {
            continue;
        }
        let generation = word(s, slot, generation_at);
        if generation == runtime().app_generations[index] {
            continue;
        }
        runtime().app_generations[index] = generation;
        serial::write(prefix);
        serial::write(b"SNAPSHOT:BEGIN\n");
        let r = *runtime();
        for (label, value) in [
            (b"PID".as_slice(), s.processes[slot].pid),
            (b"SERVICE_GENERATION", r.generation),
            (b"SURFACE_EPOCH", r.surface_epoch),
            (b"INPUT_EPOCH", r.input_epoch),
        ] {
            serial::write(prefix);
            serial::write(label);
            print_number(b":", value);
        }
        for &(at, label) in fields {
            serial::write(prefix);
            serial::write(label);
            print_number(b":", word(s, slot, at));
        }
        serial::write(prefix);
        serial::write(b"SNAPSHOT:END\n");
    }
    if s.processes[COMP].status != Status::Dead && word(s, COMP, 400) == 1 {
        let commits = word(s, COMP, 416);
        let activity = [
            word(s, COMP, 440),
            word(s, COMP, 448),
            word(s, COMP, 456),
            word(s, COMP, 496),
            word(s, COMP, 504),
            word(s, COMP, 512),
        ];
        let previous = *runtime();
        if previous.compositor_sample_tick == u64::MAX
            || activity != previous.compositor_activity
            || (commits != previous.compositor_commits
                && s.ticks.saturating_sub(previous.compositor_sample_tick) >= 100)
        {
            runtime().compositor_commits = commits;
            runtime().compositor_sample_tick = s.ticks;
            runtime().compositor_activity = activity;
            serial::write(b"WINDOW:COMPOSITOR:SNAPSHOT:BEGIN\n");
            print_number(b"WINDOW:COMPOSITOR:GENERATION:", runtime().generation);
            print_number(b"WINDOW:COMPOSITOR:PID:", s.processes[COMP].pid);
            for (at, label) in [
                (416, b"WINDOW:COMPOSITOR:COMMITS:".as_slice()),
                (424, b"WINDOW:COMPOSITOR:INPUT:"),
                (432, b"WINDOW:COMPOSITOR:REJECTED:"),
                (440, b"WINDOW:COMPOSITOR:FOCUSED:"),
                (448, b"WINDOW:COMPOSITOR:MOVES:"),
                (456, b"WINDOW:COMPOSITOR:SWITCHES:"),
                (464, b"WINDOW:COMPOSITOR:MATERIAL:"),
                (480, b"WINDOW:COMPOSITOR:CALCULATOR:"),
                (488, b"WINDOW:COMPOSITOR:FATAL:"),
                (496, b"WINDOW:COMPOSITOR:POINTER_X:"),
                (504, b"WINDOW:COMPOSITOR:POINTER_Y:"),
                (512, b"WINDOW:COMPOSITOR:BUTTONS:"),
            ] {
                print_number(label, word(s, COMP, at));
            }
            serial::write(b"WINDOW:COMPOSITOR:SNAPSHOT:END\n");
        }
    }
}
pub(super) fn poll(s: &mut State) {
    keyring::broker_tick(s);
    m5::broker_tick(s);
    require(!s.unexpected, b"UNEXPECTED_EVENT");
    let r = *runtime();
    if r.sample_tick != s.ticks {
        runtime().sample_tick = s.ticks;
        #[cfg(feature = "native-window-session-probe")]
        snapshots(s);
        if s.processes[INIT].status != Status::Dead {
            let mask = word(s, INIT, 400);
            let completed = word(s, INIT, 480);
            if mask != r.last_mask || completed != r.last_recovery {
                runtime().last_mask = mask;
                runtime().last_recovery = completed;
                serial::write(b"WINDOW:INIT:SNAPSHOT:BEGIN\n");
                print_number(b"WINDOW:INIT_PID:", s.processes[INIT].pid);
                for (label, value) in [
                    (b"WINDOW:INITMASK:".as_slice(), mask),
                    (b"WINDOW:GENERATION:", word(s, INIT, 40)),
                    (b"WINDOW:RECOVERIES:", completed),
                    (b"WINDOW:REBIND_ACKS:", word(s, INIT, 432)),
                ] {
                    print_number(label, value);
                }
                serial::write(b"WINDOW:INIT:SNAPSHOT:END\n");
            }
        }
    }
    if !r.init_done {
        return;
    }
    if APPS
        .iter()
        .any(|slot| s.processes[*slot].status != Status::Dead)
    {
        require(
            s.ticks.saturating_sub(r.stop_tick) < 300,
            b"APP_STOP_TIMEOUT",
        );
        return;
    }
    #[cfg(feature = "native-window-session-probe")]
    {
        runtime().compositor_sample_tick = u64::MAX;
        snapshots(s);
    }
    let (_, _, entropy_idle) = keyring::broker_summary();
    let (submitted, completed, block_idle) = m5::broker_summary();
    require(
        entropy_idle && block_idle && submitted == completed,
        b"BROKERS_NOT_IDLE",
    );
    for role in [COMP, SOURCE, POLICY, KEY, VOLUME] {
        if s.processes[role].status != Status::Dead {
            let pid = s.processes[role].pid;
            m4::engine().process_died(pid, PROCESS_KILLED, 0);
            s.processes[role].status = Status::Dying;
            s.cleanup(role);
        }
    }
    let stats = m4::engine().stats();
    require(
        stats.processes == 0
            && stats.handles == 0
            && stats.queued_messages == 0
            && m4::engine().check_invariants(),
        b"ENGINE_CLEANUP",
    );
    let grown = s
        .kernel
        .owned_table_count()
        .checked_sub(r.kernel_tables)
        .unwrap();
    require(
        s.free_count().checked_add(grown) == Some(r.baseline),
        b"FRAME_CLEANUP",
    );
    print_number(b"WINDOW:RESTARTS:", r.restarts);
    print_number(b"WINDOW:LIVE_PROCESSES:", stats.processes as u64);
    print_number(b"WINDOW:LIVE_HANDLES:", stats.handles as u64);
    serial::write(b"WINDOW:CLEANUP:PASS\nWINDOW:CASE:PASS\nM1:PASS\n");
    serial::exit(0x10)
}

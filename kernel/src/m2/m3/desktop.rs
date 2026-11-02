//! Fixed development composition of real isolated GUI services. No rendering,
//! input decoding, document editing or storage policy is implemented here.
//! The observer reads private result words only while their process is alive.
use super::*;
#[cfg(feature = "native-system-materiaal-probe")]
#[path = "desktop_gate.rs"]
mod gate;
use capability_core::{BlockScope, BLOCK_FLUSH, BLOCK_READ, BLOCK_WRITE, RECV, SEND};

const INIT: usize = 0;
const KEY: usize = 1;
const VOLUME: usize = 2;
const APP: usize = 3;
const COMP: usize = 4;
const SOURCE: usize = 5;
const POLICY: usize = 6;
const CONTROL: usize = 7;
const SURFACE_EPOCH: u64 = 0x5355_5200_0000_0001;
const INPUT_EPOCH: u64 = 0x494e_5000_0000_0001;
#[cfg(feature = "native-system-materiaal-probe")]
const SNAPSHOT_WORDS: [u64; 18] = [
    400, 624, 408, 416, 424, 432, 464, 480, 440, 448, 456, 504, 512, 520, 528, 536, 632, 472,
];
#[cfg(feature = "native-system-materiaal-probe")]
const SNAPSHOT_NAMES: [&[u8]; 18] = [
    b"READY",
    b"PRESENTED_GEN",
    b"FRAMES",
    b"TILES",
    b"KEYS",
    b"MOUSE",
    b"INPUT_ACKS",
    b"SAVE_ACKS",
    b"SAVED_REVISION",
    b"SAVED_BYTES",
    b"SAVED_SUM",
    b"SCREEN",
    b"DRAFT_DIRTY",
    b"DRAFT_BYTES",
    b"DRAFT_SUM",
    b"OBJECT",
    b"SCALE",
    b"ERRORS",
];
struct Observation {
    ready: u64,
    #[cfg(feature = "native-system-materiaal-probe")]
    generation: u64,
    baseline: usize,
    kernel_tables: usize,
    #[cfg(feature = "native-system-materiaal-probe")]
    stop_triggered: bool,
    controller_done: bool,
    init_done: bool,
    app_done: bool,
    #[cfg(feature = "native-system-materiaal-probe")]
    gate: gate::Gate,
    sample_tick: u64,
}
static mut OBSERVATION: Observation = Observation {
    ready: 0,
    #[cfg(feature = "native-system-materiaal-probe")]
    generation: 0,
    baseline: 0,
    kernel_tables: 0,
    #[cfg(feature = "native-system-materiaal-probe")]
    stop_triggered: false,
    controller_done: false,
    init_done: false,
    app_done: false,
    #[cfg(feature = "native-system-materiaal-probe")]
    gate: gate::Gate::new(),
    sample_tick: u64::MAX,
};
fn observation() -> &'static mut Observation {
    // SAFETY: Single CPU, IF=0, nonreentrant scheduler callbacks. No borrow is
    // held over cleanup/scheduling or a nested call to this accessor.
    unsafe { &mut *ptr::addr_of_mut!(OBSERVATION) }
}
fn fail(reason: &[u8]) -> ! {
    serial::write(b"MATERIAAL:FAIL:");
    serial::write(reason);
    serial::write(b"\n");
    serial::exit(0x11)
}
fn require(ok: bool, reason: &[u8]) {
    if !ok {
        fail(reason);
    }
}
fn put(s: &State, slot: usize, at: u64, value: u64) {
    write_word(s.processes[slot].physical(DATA + at), value);
}
fn word(s: &State, slot: usize, at: u64) -> u64 {
    read_word(s.processes[slot].physical(DATA + at))
}
fn metadata(s: &State, slot: usize) -> [u64; 128] {
    let physical = s.processes[slot].physical(DATA);
    let mut snapshot = [0u64; 128];
    // DATA is a fixed page-aligned, mapped private startup region. This sole
    // CPU has IF=0: the process cannot mutate it during this bounded copy. The
    // scratch mapping and all raw access end inside this callback; no pointer
    // or reference escapes it or survives scheduling/cleanup.
    paging::physical(physical, |page| {
        for (index, value) in snapshot.iter_mut().enumerate() {
            // SAFETY: 128 aligned u64s occupy 1024 bytes within this live 4KiB
            // frame. Read-only volatile access; no concurrent user execution.
            *value = unsafe { page.add(index * 8).cast::<u64>().read_volatile() };
        }
    });
    snapshot
}
pub(super) fn process_elf(slot: usize) -> &'static [u8] {
    match slot {
        INIT => include_bytes!(env!("NOS_DESKTOP_INIT_ELF")),
        KEY => KEYRING_SERVICE_ELF,
        VOLUME => VOLUME_SERVICE_ELF,
        APP => include_bytes!(env!("NOS_MATERIAAL_ELF")),
        COMP => include_bytes!(env!("NOS_COMPOSITOR_ELF")),
        SOURCE => include_bytes!(env!("NOS_INPUT_SOURCE_ELF")),
        POLICY => include_bytes!(env!("NOS_INPUT_POLICY_ELF")),
        #[cfg(feature = "native-system-materiaal-probe")]
        CONTROL => include_bytes!(env!("NOS_DESKTOP_CONTROLLER_ELF")),
        _ => panic!("desktop bootstrap image outside fixed manifest"),
    }
}
fn channel(e: &mut capability_core::Engine, owner: u64, sender: u64) -> (u64, u64) {
    let root = e.create_endpoint(owner).expect("bounded GUI endpoint");
    let receive = e.derive(owner, root, RECV).unwrap();
    let send = e.bootstrap_delegate(owner, root, sender, SEND).unwrap();
    e.close(owner, root).unwrap();
    (receive, send)
}
pub(super) fn manifest(s: &mut State) {
    let amount = if cfg!(feature = "native-system-materiaal-probe") {
        8
    } else {
        7
    };
    require(
        s.amount == MAX
            && s.processes[..amount]
                .iter()
                .all(|p| p.status != Status::Dead),
        b"BOOT_PROCESSES",
    );
    let [init, key, volume, app, comp, source, policy] =
        core::array::from_fn(|i| s.processes[i].pid);
    let e = m4::engine();
    for process in &s.processes[..amount] {
        e.register(process.pid).unwrap();
    }
    let registry_root = e.create_endpoint(init).unwrap();
    let registry = e.derive(init, registry_root, RECV).unwrap();
    let registry_send: [u64; 4] = [app, comp, source, policy].map(|pid| {
        e.bootstrap_delegate(init, registry_root, pid, SEND)
            .unwrap()
    });
    e.close(init, registry_root).unwrap();
    let (app_control, init_app) = channel(e, app, init);
    let (comp_control, init_comp) = channel(e, comp, init);
    let (doc_reply, volume_reply) = channel(e, app, volume);
    let (doc_ingress, app_document) = channel(e, volume, app);
    let root = e.create_endpoint(volume).unwrap();
    let other_doc = e.derive(volume, root, RECV).unwrap();
    e.close(volume, root).unwrap();
    let other_reply = volume_reply;
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
    // Two unconnected legacy key ingress endpoints. The GUI app has no key
    // service capability, raw entropy, block, launch or network authority.
    let unused_key_ingress = |engine: &mut capability_core::Engine| {
        let root = engine.create_endpoint(key).unwrap();
        let rx = engine.derive(key, root, RECV).unwrap();
        engine.close(key, root).unwrap();
        rx
    };
    let key_ingress = unused_key_ingress(e);
    let key_other = unused_key_ingress(e);
    let unused_reply = e.create_endpoint(key).unwrap();
    // Keep a real receive leaf alive: SEND alone cannot keep an endpoint open.
    // This unused fixed reply stays inside the key principal; no GUI process
    // can request key operations, and no key reply is sent during this probe.
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
        (72, other_reply),
        (120, u64::from(readonly)),
        (160, app),
        (168, init),
        (176, volume),
    ] {
        put(s, VOLUME, at, value);
    }
    let (surface_rx, app_surface) = channel(e, comp, app);
    let (surface_reply_rx, surface_reply_tx) = channel(e, app, comp);
    let (source_rx, source_tx) = channel(e, policy, source);
    let (source_reply_rx, source_reply_tx) = channel(e, source, policy);
    let (focus_rx, focus_tx) = channel(e, policy, comp);
    let (focus_reply_rx, focus_reply_tx) = channel(e, comp, policy);
    let (lifecycle_rx, _) = channel(e, policy, init);
    let (apps_rx, apps_tx) = channel(e, policy, app);
    let (input_reply_rx, input_reply_tx) = channel(e, app, policy);
    let display = e.create_display(comp).unwrap();
    let input = e.create_input(source).unwrap();
    let geometry = crate::framebuffer::descriptor();
    for (at, value) in [
        (32, 1),
        (72, doc_reply),
        (80, app_document),
        (88, registry_send[0]),
        (96, app_control),
        (112, u64::from(readonly)),
        (168, volume),
        (176, app),
        (184, init),
        (544, app_surface),
        (552, surface_reply_rx),
        (560, comp),
        (568, SURFACE_EPOCH),
        (576, apps_tx),
        (584, input_reply_rx),
        (592, policy),
        (600, INPUT_EPOCH),
        (608, 1),
        // Public VM navigation only; no authentication or new capability.
        (648, u64::from(cfg!(feature = "native-system-session-preview"))),
        (
            616,
            if cfg!(feature = "test-system-materiaal-scale200") {
                2
            } else {
                1
            },
        ),
    ] {
        put(s, APP, at, value);
    }
    for (at, value) in [
        (32, u64::from_le_bytes(*b"NOSCOMP1")),
        (40, 1),
        (48, display),
        (56, surface_rx),
        (64, surface_reply_tx),
        (72, app),
        (80, SURFACE_EPOCH),
        (88, focus_tx),
        (96, focus_reply_rx),
        (104, policy),
        (112, INPUT_EPOCH),
        (120, comp_control),
        (128, registry_send[1]),
        (136, init),
        (144, 1),
    ] {
        put(s, COMP, at, value);
    }
    for (at, value) in [
        (32, u64::from_le_bytes(*b"NOSISRC1")),
        (40, 1),
        (48, input),
        (56, source_tx),
        (64, source_reply_rx),
        (72, policy),
        (80, INPUT_EPOCH),
        (88, registry_send[2]),
        (96, init),
    ] {
        put(s, SOURCE, at, value);
    }
    for (at, value) in [
        (32, u64::from_le_bytes(*b"NOSIPOL1")),
        (40, 1),
        (48, source_rx),
        (56, focus_rx),
        (64, lifecycle_rx),
        (72, apps_rx),
        (80, source),
        (88, comp),
        (96, init),
        (104, INPUT_EPOCH),
        (112, u64::from(geometry.width)),
        (120, u64::from(geometry.height)),
        (128, source_reply_tx),
        (136, focus_reply_tx),
        (144, registry_send[3]),
        (152, registry_send[3]),
        (160, 1),
        (168, 1),
        (176, app),
        (184, input_reply_tx),
    ] {
        put(s, POLICY, at, value);
    }
    for (at, value) in [
        (32, u64::from_le_bytes(*b"NOSDINI1")),
        (40, 1),
        (48, registry),
        (56, init_app),
        (64, init_comp),
        (80, app),
        (88, comp),
        (96, source),
        (104, policy),
        (112, SURFACE_EPOCH),
        (120, INPUT_EPOCH),
        (128, 1),
        (136, 1),
    ] {
        put(s, INIT, at, value);
    }
    #[cfg(feature = "native-system-materiaal-probe")]
    {
        let controller = s.processes[CONTROL].pid;
        let (init_control, controller_send) = channel(e, init, controller);
        let (controller_reply, init_reply) = channel(e, controller, init);
        for (at, value) in [(72, init_control), (144, init_reply), (152, controller)] {
            put(s, INIT, at, value);
        }
        for (at, value) in [
            (32, u64::from_le_bytes(*b"NOSDCTL1")),
            (40, 1),
            (48, controller_send),
            (56, controller_reply),
            (64, init),
            (72, SURFACE_EPOCH),
            (80, controller),
        ] {
            put(s, CONTROL, at, value);
        }
    }
    for (slot, handles) in [(
        APP,
        [
            app_document,
            doc_reply,
            app_surface,
            surface_reply_rx,
            apps_tx,
            input_reply_rx,
            registry_send[0],
            app_control,
        ],
    )] {
        for handle in handles {
            let pid = s.processes[slot].pid;
            require(
                e.check_display(pid, handle).is_err()
                    && e.check_input(pid, handle).is_err()
                    && e.check_entropy(pid, handle).is_err()
                    && e.check_network(pid, handle).is_err()
                    && e.check_block(pid, handle, BLOCK_READ).is_err()
                    && e.check_launch(pid, handle).is_err(),
                b"APP_AUTHORITY",
            );
        }
    }
    require(e.check_invariants(), b"ENGINE_MANIFEST");
    observation().baseline = s.free_count()
        + s.processes[..amount]
            .iter()
            .map(|p| {
                p.count + p.kernel_frames.len() + p.paging.as_ref().unwrap().owned_table_count()
            })
            .sum::<usize>();
    observation().kernel_tables = s.kernel.owned_table_count();
    serial::write(b"MATERIAAL:MANIFEST:PASS\n");
}
pub(super) fn observe_exit(s: &State, slot: usize, _: u64) {
    print_number(b"MATERIAAL:EXIT_SLOT:", slot as u64);
    print_number(b"MATERIAAL:EXIT_CODE:", s.last_event.code);
    print_number(b"MATERIAAL:EXIT_STEP:", word(s, slot, 240));
    require(s.last_event.code == 0, b"PROCESS_EXIT");
    match slot {
        APP => observation().app_done = true,
        INIT => {
            require(
                word(s, INIT, 480) == 7
                    && word(s, INIT, 448) == 1
                    && word(s, INIT, 464) == 1
                    && word(s, INIT, 472) == 1
                    && word(s, INIT, 512) == 1,
                b"INIT_STOP_PROOF",
            );
            observation().init_done = true;
        }
        CONTROL => {
            require(word(s, CONTROL, 456) == 1, b"CONTROL_RESULT");
            observation().controller_done = true;
        }
        _ => fail(b"CRITICAL_SERVICE_EXIT"),
    }
}
pub(super) fn observe_fault(s: &State, slot: usize, context: &[u64; 22], _: u64) {
    print_number(b"MATERIAAL:FAULT_SLOT:", slot as u64);
    print_number(b"MATERIAAL:FAULT_VECTOR:", context[15]);
    print_number(b"MATERIAAL:FAULT_RIP:", context[17]);
    print_number(b"MATERIAAL:FAULT_STEP:", word(s, slot, 240));
    fail(b"NATIVE_FAULT")
}
pub(super) fn poll(s: &mut State) {
    keyring::broker_tick(s);
    m5::broker_tick(s);
    require(!s.unexpected, b"UNEXPECTED_EVENT");
    #[cfg(feature = "native-system-materiaal-probe")]
    if s.ticks >= 20_000 {
        for slot in 0..8 {
            if s.processes[slot].status != Status::Dead {
                print_number(b"MATERIAAL:TIMEOUT_SLOT:", slot as u64);
                for offset in [
                    40, 192, 240, 400, 408, 416, 424, 432, 440, 472, 480, 488, 496,
                ] {
                    print_number(b"MATERIAAL:AT:", offset);
                    print_number(b"MATERIAAL:VALUE:", word(s, slot, offset));
                }
            }
        }
        fail(b"TIMEOUT");
    }
    // Observer-only cadence. Device brokers still run on every kernel entry;
    // security checks, IPC and rendering retain their original semantics.
    if observation().sample_tick != s.ticks {
        observation().sample_tick = s.ticks;
        if observation().ready != 15 && s.processes[INIT].status != Status::Dead {
            let init = metadata(s, INIT);
            for (bit, (at, label)) in [
                (400, &b"MATERIAAL:SOURCE:READY\n"[..]),
                (408, b"MATERIAAL:POLICY:READY\n"),
                (416, b"MATERIAAL:COMPOSITOR:READY\n"),
                (432, b"MATERIAAL:FOCUS:READY\n"),
            ]
            .iter()
            .enumerate()
            {
                if init[*at as usize / 8] == 1 && observation().ready & (1 << bit) == 0 {
                    observation().ready |= 1 << bit;
                    serial::write(label);
                }
            }
        }
        #[cfg(feature = "native-system-materiaal-probe")]
        if s.processes[APP].status != Status::Dead {
            let app = metadata(s, APP);
            if app[400 / 8] == 1 {
                let generation = app[624 / 8];
                if generation > observation().generation {
                    observation().generation = generation;
                    serial::write(b"MATERIAAL:SNAPSHOT:BEGIN\n");
                    for (at, label) in SNAPSHOT_WORDS.into_iter().zip(SNAPSHOT_NAMES) {
                        serial::write(b"MATERIAAL:");
                        serial::write(label);
                        print_number(b":", app[at as usize / 8]);
                    }
                    serial::write(b"MATERIAAL:SNAPSHOT:END\n");
                }
                if !observation().stop_triggered && observation().ready == 15 {
                    let frame = gate::Frame {
                        generation,
                        screen: app[504 / 8],
                        dirty: app[512 / 8],
                        save_acks: app[480 / 8],
                        revision: app[440 / 8],
                        saved_bytes: app[448 / 8],
                        saved_sum: app[456 / 8],
                        draft_bytes: app[520 / 8],
                        draft_sum: app[528 / 8],
                        object: app[536 / 8],
                        f12: app[640 / 8],
                    };
                    let mut sum = 0xcbf29ce484222325u64;
                    for byte in b"nieuw os werkt" {
                        sum = (sum ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
                    }
                    if observation().gate.observe(
                        frame,
                        cfg!(feature = "test-system-recover"),
                        14,
                        sum,
                    ) {
                        observation().stop_triggered = true;
                        print_number(b"MATERIAAL:F12_BARRIER:", app[640 / 8]);
                        put(s, CONTROL, 400, 1);
                        serial::write(b"MATERIAAL:CONTROLLER:RELEASED\n");
                    }
                }
            }
        }
    }
    #[cfg(feature = "native-system-materiaal-probe")]
    if observation().app_done && observation().init_done && observation().controller_done {
        require(
            word(s, KEY, 192) == 1 && word(s, VOLUME, 192) == 1,
            b"SERVICE_READY",
        );
        let screen = display::counts();
        let input = ps2::counts();
        require(
            screen[0] > 0 && screen[1] > 0 && input[0] > 0 && input[1] > 0 && input[2] > 0,
            b"REAL_DEVICE_PROGRESS",
        );
        let (_, _, entropy_idle) = keyring::broker_summary();
        let (submitted, completed, block_idle) = m5::broker_summary();
        require(
            entropy_idle && block_idle && submitted == completed,
            b"BROKERS_NOT_IDLE",
        );
        require(submitted[0] > 0, b"NO_DISK_READ");
        if !cfg!(feature = "test-system-recover") {
            require(submitted[1] > 0 && submitted[2] > 0, b"NO_DURABLE_WRITE");
        }
        for slot in [KEY, VOLUME, COMP, SOURCE, POLICY] {
            s.processes[slot].status = Status::Dying;
            s.cleanup(slot);
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
            .checked_sub(observation().kernel_tables)
            .unwrap();
        require(
            s.free_count().checked_add(grown) == Some(observation().baseline),
            b"FRAME_CLEANUP",
        );
        for (label, value) in [
            (&b"MATERIAAL:DISK_READS:"[..], submitted[0]),
            (b"MATERIAAL:DISK_WRITES:", submitted[1]),
            (b"MATERIAAL:DISK_FLUSHES:", submitted[2]),
            (b"MATERIAAL:LIVE_PROCESSES:", stats.processes as u64),
            (b"MATERIAAL:LIVE_HANDLES:", stats.handles as u64),
            (b"MATERIAAL:FRAMES_RECOVERED:", 1),
        ] {
            print_number(label, value);
        }
        serial::write(b"MATERIAAL:CLEANUP:PASS\nMATERIAAL:CASE:PASS\nM1:PASS\n");
        serial::exit(0x10)
    }
}

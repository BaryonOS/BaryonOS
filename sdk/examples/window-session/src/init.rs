#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#[path = "../../system/src/support.rs"]
mod support;
use nieuw_os_sdk_runtime::{
    native,
    process::{ChildProcess, LaunchImage, ProcessStatus},
    Handle, Outgoing, ReceiveEndpoint, SendEndpoint,
};
use nieuw_os_window_session::{
    boot::CONTROL,
    recovery::{ready_bit, Group, Recovery},
};
#[derive(Clone, Copy)]
struct Boot {
    generation: u64,
    registry: ReceiveEndpoint,
    apps: [SendEndpoint; 2],
    comp: SendEndpoint,
    pids: [u64; 5],
    surface_epoch: u64,
    input_epoch: u64,
    launch: LaunchImage,
    process: ChildProcess,
}
impl Boot {
    fn group(self) -> Group {
        Group {
            generation: self.generation,
            surface_epoch: self.surface_epoch,
            input_epoch: self.input_epoch,
        }
    }
}
fn fail(code: u64) -> ! {
    support::put(488, code);
    native::exit(code)
}
fn handle(at: u64) -> Handle {
    Handle::from_raw(support::word(at)).unwrap_or_else(|_| fail(1))
}
fn read() -> Boot {
    if support::word(32) != u64::from_le_bytes(*b"WININIT1") {
        fail(2);
    }
    let boot = Boot {
        generation: support::word(40),
        registry: ReceiveEndpoint::from_handle(handle(48)),
        apps: [
            SendEndpoint::from_handle(handle(56)),
            SendEndpoint::from_handle(handle(64)),
        ],
        comp: SendEndpoint::from_handle(handle(72)),
        pids: core::array::from_fn(|i| support::word(80 + i as u64 * 8)),
        surface_epoch: support::word(120),
        input_epoch: support::word(128),
        launch: LaunchImage::from_handle(handle(136)),
        process: ChildProcess::from_handle(handle(144)),
    };
    if boot.generation == 0
        || boot.surface_epoch == 0
        || boot.input_epoch == 0
        || boot.pids.contains(&0)
        || boot
            .pids
            .iter()
            .enumerate()
            .any(|(i, p)| boot.pids[..i].contains(p))
    {
        fail(3);
    }
    boot
}
fn tick() -> u64 {
    native::monotonic_ticks().unwrap_or_else(|_| fail(4))
}
fn send(to: SendEndpoint, op: u8) {
    let m = Outgoing::new(CONTROL, &[op]).unwrap_or_else(|_| fail(5));
    let start = tick();
    loop {
        match native::send(to, &m) {
            Ok(()) => return,
            Err(e) if e.is_again() && tick() - start < 200 => {
                let _ = native::yield_now();
            }
            Err(_) => fail(6),
        }
    }
}
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    run()
}
#[inline(never)]
fn run() -> ! {
    native::check_abi().unwrap_or_else(|_| fail(7));
    let mut boot = read();
    let mut mask = 0u8;
    let mut running = false;
    let mut pause = 0u8;
    let mut recovering = false;
    let mut recovery: Option<Recovery> = None;
    let mut rebound = 0u8;
    let mut stopping = false;
    let mut stopped = 0u8;
    let mut start = tick();
    let mut restarts = 0u64;
    let mut rejected = 0u64;
    loop {
        for _ in 0..16 {
            let m = match native::receive(boot.registry) {
                Ok(m) => m,
                Err(e) if e.is_again() => break,
                Err(_) => fail(8),
            };
            if m.cap_count() != 0 {
                let _ = native::close_received_caps(&m);
                rejected += 1;
                continue;
            }
            let index = boot.pids.iter().position(|p| *p == m.sender());
            if m.tag() == CONTROL && m.payload().len() == 1 {
                match (index, m.payload()[0]) {
                    (Some(i), 1) if i < 2 && !running && !recovering => mask |= 1 << i,
                    (Some(i), 7)
                        if i < 2 && recovery.as_ref().is_some_and(Recovery::awaiting_pause) =>
                    {
                        pause |= 1 << i
                    }
                    (Some(i), 8)
                        if i < 2
                            && recovery.as_ref().is_some_and(Recovery::awaiting_rebind_ack) =>
                    {
                        rebound |= 1 << i
                    }
                    (Some(i), 3) if i < 2 && stopping => {
                        stopped |= 1 << i;
                        if i == 0 {
                            send(boot.apps[1], 5);
                        }
                    }
                    (Some(i), 6) if i < 2 && stopping => {
                        stopping = false;
                        stopped = 0;
                        support::put(496, 1);
                    }
                    (Some(2), 9) if running && !recovering && !stopping => {
                        stopping = true;
                        stopped = 0;
                        send(boot.apps[0], 5);
                    }
                    _ => rejected += 1,
                }
                continue;
            }
            // Old readiness cannot prefill the replacement mask while waiting
            // for app pause or rebind acknowledgements.
            if recovering && !recovery.as_ref().is_some_and(Recovery::awaiting_ready) {
                rejected += 1;
                continue;
            }
            let Some(bit) = ready_bit(boot.group(), index, m.tag(), m.payload()) else {
                rejected += 1;
                continue;
            };
            mask |= bit;
        }
        if !running && !recovering && mask == 31 {
            for to in boot.apps {
                send(to, 2);
            }
            running = true;
            support::put(440, 1);
        }
        if running && !recovering && !stopping {
            match native::status(boot.process).unwrap_or_else(|_| fail(9)) {
                ProcessStatus::Running => {}
                ProcessStatus::Faulted(_) | ProcessStatus::Exited(_) => {
                    // App-owned reply endpoints survive peer death. Retire
                    // their exact old presentation leaves before waiting for
                    // app control: an in-flight RPC must return to its owner.
                    native::retire_service_group_presentation(boot.launch, boot.process)
                        .unwrap_or_else(|_| fail(17));
                    recovering = true;
                    recovery = Some(Recovery::begin(boot.group()).unwrap_or_else(|_| fail(18)));
                    mask = 3;
                    pause = 0;
                    rebound = 0;
                    start = tick();
                    for to in boot.apps {
                        send(to, 7);
                    }
                }
                ProcessStatus::Killed => fail(10),
            }
        }
        if recovery
            .as_ref()
            .is_some_and(|state| state.can_restart(pause))
        {
            let process = native::restart_service_group(boot.launch, boot.process)
                .unwrap_or_else(|_| fail(11));
            let updated = read();
            if updated.process != process
                || updated.generation <= boot.generation
                || updated.pids[..2] != boot.pids[..2]
                || updated.pids[2..].iter().any(|p| boot.pids[2..].contains(p))
                || updated.surface_epoch == boot.surface_epoch
                || updated.input_epoch == boot.input_epoch
            {
                fail(12);
            }
            recovery
                .as_mut()
                .unwrap_or_else(|| fail(18))
                .installed(updated.group(), pause)
                .unwrap_or_else(|_| fail(18));
            boot = updated;
            mask = 3;
            pause = 0;
            rebound = 0;
            restarts = restarts.checked_add(1).unwrap_or_else(|| fail(14));
        }
        if recovery
            .as_ref()
            .is_some_and(|state| state.can_rebind(boot.group(), mask, pause))
        {
            recovery
                .as_mut()
                .unwrap_or_else(|| fail(18))
                .rebind_sent(boot.group(), mask, pause)
                .unwrap_or_else(|_| fail(18));
            for to in boot.apps {
                send(to, 8);
            }
            pause = 4;
        }
        if recovery
            .as_ref()
            .is_some_and(|state| state.complete(rebound))
        {
            recovering = false;
            recovery = None;
            support::put(480, restarts);
        }
        if recovering && tick().saturating_sub(start) > 6000 {
            fail(15);
        }
        if stopping && stopped == 3 {
            send(boot.comp, 5);
            support::put(504, 1);
            native::exit(0);
        }
        for (at, v) in [
            (400, u64::from(mask)),
            (408, restarts),
            (416, rejected),
            (424, u64::from(recovering)),
            (432, u64::from(rebound)),
        ] {
            support::put(at, v);
        }
        native::yield_now().unwrap_or_else(|_| fail(16));
    }
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(99)
}

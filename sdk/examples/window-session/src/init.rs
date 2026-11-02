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
use nieuw_os_window_session::boot::{CONTROL, READY};
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
fn word(b: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(b[at..at + 8].try_into().unwrap())
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
                    (Some(i), 7) if i < 2 && recovering => pause |= 1 << i,
                    (Some(i), 8) if i < 2 && recovering => rebound |= 1 << i,
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
            let b = m.payload();
            if m.tag() != READY
                || b.len() != 32
                || &b[..4] != b"IPD1"
                || b[4..6] != 1u16.to_le_bytes()
                || word(b, 16) != boot.generation
            {
                rejected += 1;
                continue;
            }
            let role = u16::from_le_bytes([b[6], b[7]]);
            let (expected, epoch, bit) = match role {
                1 => (3, boot.input_epoch, 8),
                2 => (4, boot.input_epoch, 16),
                3 => (2, boot.surface_epoch, 4),
                _ => {
                    rejected += 1;
                    continue;
                }
            };
            if (role == 2 && word(b, 24) != 0) || (role != 2 && word(b, 24) == 0) {
                rejected += 1;
                continue;
            }
            if index != Some(expected) || word(b, 8) != epoch {
                rejected += 1;
                continue;
            }
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
        if recovering && pause == 3 {
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
            boot = updated;
            mask = 3;
            pause = 0;
            rebound = 0;
            restarts = restarts.checked_add(1).unwrap_or_else(|| fail(14));
        }
        if recovering && mask == 31 && pause == 0 {
            for to in boot.apps {
                send(to, 8);
            }
            pause = 4;
        }
        if recovering && pause == 4 && rebound == 3 {
            recovering = false;
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

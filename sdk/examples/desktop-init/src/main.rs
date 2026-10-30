#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#[path = "../../system/src/support.rs"]
mod support;
use nieuw_os_native_desktop_init::{
    boot::Bootstrap,
    supervisor::{Phase, Supervisor},
};
use nieuw_os_native_input_service::{
    source::wait::{self, Action, Mode},
    NativeIpc,
};
use nieuw_os_sdk_runtime::native;
fn fail(code: u64, config: Option<&Bootstrap>) -> ! {
    support::put(488, code);
    if let Some(config) = config {
        for handle in config.handles().into_iter().flatten() {
            let _ = native::close(handle);
        }
    }
    native::exit(code)
}
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    run()
}
#[inline(never)]
fn run() -> ! {
    let mode = native::abi_info()
        .ok()
        .and_then(Mode::from_abi)
        .unwrap_or_else(|| fail(90, None));
    let words = core::array::from_fn(|i| support::word(i as u64 * 8));
    let config = Bootstrap::decode(&words).unwrap_or_else(|_| fail(91, None));
    let now = native::monotonic_ticks().unwrap_or_else(|_| fail(92, Some(&config)));
    let mut supervisor = Supervisor::new(config, now).unwrap_or_else(|_| fail(93, Some(&config)));
    let mut io = NativeIpc;
    loop {
        let now = native::monotonic_ticks().unwrap_or_else(|_| fail(94, Some(&config)));
        let result = supervisor.step(now, &mut io);
        let c = supervisor.counts();
        for (o, v) in [
            (400, c.source_ready),
            (408, c.policy_ready),
            (416, c.compositor_ready),
            (424, c.app_startup),
            (432, c.focus_ack),
            (448, c.stop_sent),
            (456, c.refused),
            (464, c.retire_ack),
            (472, c.controller_replies),
            (488, supervisor.fatal().map_or(0, |f| f as u64)),
            (496, c.rejected),
            (504, c.closed_caps),
            (512, c.app_done),
            (520, c.registry_receives),
            (528, c.ready_tick),
            (536, supervisor.start_tick()),
        ] {
            support::put(o, v);
        }
        // Publish coherent diagnostic readiness last; the real IPC send has
        // already completed. This does not claim current renderer health.
        support::put(480, supervisor.phase() as u64);
        support::put(440, c.go_sent);
        match result {
            Ok(Phase::Stopped) => {
                for handle in config.handles().into_iter().flatten() {
                    let _ = native::close(handle);
                }
                native::exit(0)
            }
            Ok(Phase::Failed) => fail(supervisor.fatal().map_or(98, |e| e as u64), Some(&config)),
            Err(error) => fail(error as u64, Some(&config)),
            _ => {}
        }
        // ABI 1.8 keeps the original loop. On 1.10 only a quiescent Running
        // supervisor parks; clock/wait/mask faults fail closed, never yield.
        let action = if mode == Mode::Wait {
            supervisor
                .wait_plan(now)
                .unwrap_or_else(|_| fail(94, Some(&config)))
        } else {
            Action::Run
        };
        match action {
            Action::Run => {
                if native::yield_now().is_err() {
                    fail(95, Some(&config));
                }
            }
            Action::Park(plan) => match wait::park(&plan) {
                Ok(()) => {}
                Err(wait::ParkError::Mask) => fail(97, Some(&config)),
                Err(wait::ParkError::Clock(_)) => fail(94, Some(&config)),
                Err(_) => fail(96, Some(&config)),
            },
        }
    }
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(99, None)
}

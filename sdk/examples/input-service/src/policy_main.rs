#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#[path = "../../system/src/support.rs"]
mod support;
use nieuw_os_native_input_service::{
    boot::{self, PolicyBoot},
    policy::Policy,
    Ipc, NativeIpc,
};
use nieuw_os_sdk_runtime::native;

fn words() -> [u64; 128] {
    core::array::from_fn(|index| support::word(index as u64 * 8))
}
fn close_all(boot: &PolicyBoot) {
    let mut handles = [None; 16];
    let c = boot.configuration;
    for (index, handle) in [
        c.source.endpoint.handle(),
        c.compositor.endpoint.handle(),
        c.supervisor.endpoint.handle(),
        c.applications.handle(),
        boot.replies.source.handle(),
        boot.replies.compositor.handle(),
        boot.replies.supervisor.handle(),
        boot.registry.handle(),
    ]
    .into_iter()
    .enumerate()
    {
        handles[index] = Some(handle);
    }
    for (index, client) in boot.clients[..boot.client_count].iter().enumerate() {
        handles[8 + index] = Some(client.reply.handle());
    }
    for (index, handle) in handles.iter().enumerate() {
        if let Some(handle) = handle {
            if !handles[..index].contains(&Some(*handle)) {
                let _ = native::close(*handle);
            }
        }
    }
}
fn fail(code: u64, boot: Option<&PolicyBoot>) -> ! {
    support::put(480, code);
    if let Some(boot) = boot {
        close_all(boot);
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
    let boot = PolicyBoot::decode(&words()).unwrap_or_else(|_| fail(1, None));
    if native::abi_info() != Ok((1 << 32) | 8) {
        fail(2, Some(&boot));
    }
    let mut policy = Policy::new(
        boot.configuration,
        boot.replies,
        &boot.clients[..boot.client_count],
    )
    .unwrap_or_else(|_| fail(3, Some(&boot)));
    let ready = boot::ready(2, boot.configuration.epoch, boot.generation, 0)
        .unwrap_or_else(|_| fail(4, Some(&boot)));
    let mut registered = false;
    let mut ready_started = None;
    let mut io = NativeIpc;
    loop {
        if policy.step(&mut io).is_err() {
            policy.shutdown();
            fail(5, Some(&boot));
        }
        if !registered && policy.receive_rights_checked() {
            let now = native::monotonic_ticks().unwrap_or_else(|_| fail(8, Some(&boot)));
            let start = *ready_started.get_or_insert(now);
            if now.saturating_sub(start) > nieuw_os_native_input_service::source::EXCHANGE_TICKS {
                policy.shutdown();
                fail(9, Some(&boot));
            }
            match io.send(boot.registry, &ready) {
                Ok(()) => {
                    registered = true;
                    support::put(472, 1);
                }
                Err(error) if error.is_again() => {}
                Err(_) => {
                    policy.shutdown();
                    fail(6, Some(&boot));
                }
            }
        }
        let c = policy.counts();
        for (offset, value) in [
            (400, c.received),
            (408, c.handled),
            (416, c.rejected),
            (424, c.pipelined),
            (432, c.reply_again),
            (440, c.sent),
            (448, c.stale),
            (456, c.retired),
            (464, u64::from(c.recv_checked)),
        ] {
            support::put(offset, value);
        }
        if native::yield_now().is_err() {
            policy.shutdown();
            fail(7, Some(&boot));
        }
    }
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    support::put(480, 99);
    native::exit(99)
}

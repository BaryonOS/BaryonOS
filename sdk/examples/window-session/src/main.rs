#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#[path = "../../system/src/support.rs"]
mod support;
use core::cell::UnsafeCell;
use nieuw_os_display_client::{wire::Rectangle as NativeRect, DisplayClient, NativeTransport};
use nieuw_os_graphics::Rect;
use nieuw_os_input_client::{Binding, Event, Peek, Session as Input};
use nieuw_os_input_core::{
    wire::{self, Operation, Request},
    CancelReason,
};
use nieuw_os_native_input_service::{protocol::command_reply, Ipc, NativeIpc};
use nieuw_os_sdk_runtime::{native, Outgoing, SendEndpoint};
use nieuw_os_surface_protocol::{self as sur, Action, Receiver};
use nieuw_os_window_session::{
    boot::{Bootstrap, CONTROL},
    input::Queue,
    scene::{Damage, Scene, MAX_PIXELS},
};
use ui_core::{KeyAction, PhysicalKey};

struct Backing(UnsafeCell<[u32; MAX_PIXELS]>);
// SAFETY: One thread acquires this private allocation once in nonreturning
// entry. There is no shared mapping or callback that can reenter this code.
unsafe impl Sync for Backing {}
static BACKING: Backing = Backing(UnsafeCell::new([0; MAX_PIXELS]));
fn fail(code: u64) -> ! {
    support::put(488, code);
    native::exit(code)
}
fn tick() -> u64 {
    native::monotonic_ticks().unwrap_or_else(|_| fail(2))
}
fn send(to: SendEndpoint, message: &Outgoing) {
    let start = tick();
    loop {
        match native::send(to, message) {
            Ok(()) => return,
            Err(e) if e.is_again() && tick().saturating_sub(start) < 200 => {
                let _ = native::yield_now();
            }
            Err(_) => fail(3),
        }
    }
}
fn present(
    scene: &Scene,
    backing: &[u32],
    display: &mut DisplayClient<NativeTransport>,
    rect: Rect,
) -> Result<(), ()> {
    if rect.x < 0 || rect.y < 0 || rect.width > 32 || rect.height > 8 {
        return Err(());
    }
    let mut colors = [0u32; 256];
    scene.render(rect, &mut colors, backing).map_err(|_| ())?;
    let length = (rect.width * rect.height * 4) as usize;
    let mut bytes = [0u8; 1024];
    for (p, b) in colors.iter().zip(bytes[..length].chunks_exact_mut(4)) {
        b.copy_from_slice(&p.to_le_bytes());
    }
    display
        .blit(
            NativeRect {
                x: rect.x as u32,
                y: rect.y as u32,
                width: rect.width as u16,
                height: rect.height as u16,
            },
            &bytes[..length],
        )
        .map_err(|_| ())
}
fn repaint(
    scene: &Scene,
    backing: &[u32],
    display: &mut DisplayClient<NativeTransport>,
    damage: &mut Damage,
    count: usize,
) -> Result<(), ()> {
    for _ in 0..count {
        let Some(rect) = damage.next_tile() else {
            break;
        };
        present(scene, backing, display, rect)?;
    }
    Ok(())
}
fn focus(boot: &Bootstrap) {
    let request = Request {
        operation: Operation::Focus,
        id: 1,
        epoch: boot.policy_epoch,
        arg0: 100,
        arg1: 0,
    };
    send(
        boot.focus,
        &Outgoing::new(wire::TAG, &request.encode()).unwrap_or_else(|_| fail(4)),
    );
    let start = tick();
    loop {
        match native::receive(boot.focus_reply) {
            Ok(reply) => {
                if command_reply(&reply, boot.policy_pid, request) != Ok(0) {
                    fail(5);
                }
                return;
            }
            Err(e) if e.is_again() && tick().saturating_sub(start) < 200 => {
                let _ = native::yield_now();
            }
            Err(_) => fail(6),
        }
    }
}
fn mark_focus(scene: &Scene, damage: &mut Damage) {
    for index in 0..2 {
        damage.mark(scene.paint_bounds(index));
    }
    damage.mark(scene.dock());
}
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    run()
}
#[inline(never)]
fn run() -> ! {
    native::check_abi().unwrap_or_else(|_| fail(1));
    let words = core::array::from_fn(|i| support::word(i as u64 * 8));
    let boot = Bootstrap::decode(&words).unwrap_or_else(|_| fail(7));
    let mut display = DisplayClient::new(NativeTransport, boot.display);
    let geometry = display.geometry().unwrap_or_else(|_| fail(8));
    let mut scene = Scene::new(
        geometry.width,
        geometry.height,
        boot.scale,
        boot.apps.map(|a| a.surface_epoch),
    )
    .unwrap_or_else(|_| fail(9));
    // SAFETY: Sole access for this process lifetime; no other reference to the
    // static exists. Apps cannot map this userspace process's backing store.
    let backing = unsafe { &mut *BACKING.0.get() };
    backing.fill(0xf4f6f8);
    let mut surfaces = boot.apps.map(|app| {
        let index = usize::from(app.pid == boot.apps[1].pid);
        Receiver::new(app.pid, scene.windows[index].geometry, 200).unwrap_or_else(|_| fail(10))
    });
    let mut queues = boot
        .apps
        .map(|app| Queue::new(app.pid, app.client, app.input_epoch).unwrap_or_else(|_| fail(11)));
    let mut input = Input::new(
        nieuw_os_input_client::NativeTransport,
        Binding {
            epoch: boot.policy_epoch,
            service_pid: boot.policy_pid,
            client_id: 100,
            width: geometry.width,
            height: geometry.height,
            send: boot.input,
            receive: boot.input_reply,
        },
        100_000,
    )
    .unwrap_or_else(|_| fail(12));
    let mut damage = Damage::new(geometry.width, geometry.height).unwrap_or_else(|_| fail(36));
    damage.mark(Rect::new(0, 0, geometry.width, geometry.height));
    // Complete the initial trusted frame before declaring compositor readiness.
    while let Some(rect) = damage.next_tile() {
        present(&scene, backing, &mut display, rect).unwrap_or_else(|_| fail(13));
        if (rect.y as u32).is_multiple_of(64) {
            let _ = native::yield_now();
        }
    }
    focus(&boot);
    let ready = boot.ready(geometry.epoch).unwrap_or_else(|_| fail(14));
    send(boot.registry, &ready);
    support::put(472, 1);
    let mut io = NativeIpc;
    let mut pending: [Option<(SendEndpoint, Outgoing, u64)>; 2] = core::array::from_fn(|_| None);
    let mut received = 0u64;
    let mut commits = 0u64;
    let mut rejected = 0u64;
    let mut input_events = 0u64;
    loop {
        for _ in 0..4 {
            let offer = match input.peek() {
                Ok(Peek::Empty) => break,
                Ok(Peek::Offer(o)) => o,
                Err(_) => fail(15),
            };
            input_events = input_events.checked_add(1).unwrap_or_else(|| fail(16));
            let previous = scene.focused;
            let event = offer.event();
            match event {
                Event::Key(key) if key.key == PhysicalKey::F6 => {
                    if key.action == KeyAction::Press {
                        scene.cycle().unwrap_or_else(|_| fail(17));
                    }
                }
                #[cfg(feature = "fault-injection")]
                Event::Key(key) if key.key == PhysicalKey::F12 => {
                    if key.action == KeyAction::Press {
                        // SAFETY: Deliberate CPL3 fault only in the explicitly
                        // selected acceptance executable; no host instruction.
                        unsafe {
                            core::arch::asm!("ud2", options(noreturn));
                        }
                    }
                }
                Event::Key(key) if key.key == PhysicalKey::F10 => {
                    if key.action == KeyAction::Press {
                        send(
                            boot.registry,
                            &Outgoing::new(CONTROL, &[9]).unwrap_or_else(|_| fail(18)),
                        );
                    }
                }
                Event::Key(_) => queues[scene.focused]
                    .push(event)
                    .unwrap_or_else(|_| fail(19)),
                Event::Cancel(reason) => {
                    scene.cancel_pointer();
                    for q in &mut queues {
                        q.cancel(reason).unwrap_or_else(|_| fail(20));
                    }
                }
                Event::Mouse { input: mouse, x, y } => {
                    damage.mark(scene.pointer_bounds());
                    let old_frames = [scene.paint_bounds(0), scene.paint_bounds(1)];
                    let recipient = scene
                        .mouse(
                            x,
                            y,
                            mouse.changed.left() && mouse.buttons.left(),
                            mouse.buttons.left(),
                        )
                        .unwrap_or_else(|_| fail(21));
                    damage.mark(scene.pointer_bounds());
                    for index in 0..2 {
                        if old_frames[index] != scene.paint_bounds(index) {
                            damage.mark(old_frames[index]);
                            damage.mark(scene.paint_bounds(index));
                        }
                    }
                    if let Some(index) = recipient {
                        let b = scene.windows[index].body;
                        // Translate only into the immutable assigned app-local
                        // rectangle. Window movement never widens its bounds.
                        queues[index]
                            .push(Event::Mouse {
                                input: mouse,
                                x: x - b.x as u32,
                                y: y - b.y as u32,
                            })
                            .unwrap_or_else(|_| fail(22));
                    } else {
                        // Leaving an app body must cancel an app-local press
                        // or selection drag; do not retain an invisible grab.
                        queues[previous]
                            .cancel(CancelReason::Resync)
                            .unwrap_or_else(|_| fail(22));
                    }
                }
            }
            if scene.focused != previous {
                queues[previous]
                    .cancel(CancelReason::FocusLost)
                    .unwrap_or_else(|_| fail(23));
                mark_focus(&scene, &mut damage);
            }
            input.ack(&offer).unwrap_or_else(|_| fail(24));
        }
        repaint(&scene, backing, &mut display, &mut damage, 32).unwrap_or_else(|_| fail(25));
        for index in 0..2 {
            if let Some((to, mut message, start)) = pending[index].take() {
                let expired = tick().saturating_sub(start) > 200;
                if !expired
                    && u64::from_le_bytes(message.as_bytes()[8..16].try_into().unwrap())
                        == wire::TAG
                {
                    let mut prepared = [0u8; wire::SIZE];
                    prepared.copy_from_slice(&message.as_bytes()[64..64 + wire::SIZE]);
                    let reply = queues[index]
                        .refresh(&prepared)
                        .unwrap_or_else(|_| fail(34));
                    message = Outgoing::new(wire::TAG, &reply).unwrap_or_else(|_| fail(35));
                }
                let failed = if expired {
                    true
                } else {
                    match io.send(to, &message) {
                        Ok(()) => false,
                        Err(e) if e.is_again() => {
                            pending[index] = Some((to, message, start));
                            continue;
                        }
                        Err(_) => true,
                    }
                };
                if failed {
                    // One unread reply cannot block the other app's staged
                    // tile or input ACK. Only this window loses its channels.
                    surfaces[index].retire();
                    queues[index].retire();
                    let _ = native::close(boot.apps[index].surface_reply.handle());
                    let _ = native::close(boot.apps[index].input_reply.handle());
                    rejected += 1;
                }
            }
        }
        for _ in 0..16 {
            let message = match io.receive(boot.ingress) {
                Ok(m) => m,
                Err(e) if e.is_again() => break,
                Err(_) => fail(28),
            };
            received = received.checked_add(1).unwrap_or_else(|| fail(29));
            if message.cap_count() != 0 {
                let _ = native::close_received_caps(&message);
                rejected += 1;
                continue;
            }
            if message.sender() == boot.init_pid
                && message.tag() == CONTROL
                && message.payload() == [5]
            {
                native::exit(0);
            }
            let Some(index) = boot.apps.iter().position(|a| a.pid == message.sender()) else {
                rejected += 1;
                continue;
            };
            if pending[index].is_some() {
                rejected += 1;
                continue;
            }
            let (tag, payload, to) = if message.tag() == sur::TAG {
                let now = tick();
                let reply = match surfaces[index].accept(message.sender(), now, message.payload()) {
                    Ok(Action::Staged) => continue,
                    Ok(Action::Info(reply)) => reply,
                    Ok(Action::Commit(token)) => {
                        let result = surfaces[index]
                            .commit_with(token, tick(), |rect, pixels| {
                                let damaged = scene
                                    .copy_tile(index, rect, pixels, backing)
                                    .map_err(|_| ())?;
                                present(&scene, backing, &mut display, damaged)
                            })
                            .unwrap_or_else(|_| fail(30));
                        if result.status().is_ok() {
                            commits += 1;
                        } else {
                            rejected += 1;
                        }
                        result
                    }
                    Err(error) => {
                        rejected += 1;
                        let Some(reply) =
                            surfaces[index].reject(message.sender(), message.payload(), error)
                        else {
                            continue;
                        };
                        reply
                    }
                };
                (
                    sur::TAG,
                    Outgoing::new(sur::TAG, reply.bytes()).unwrap_or_else(|_| fail(31)),
                    boot.apps[index].surface_reply,
                )
            } else if message.tag() == wire::TAG {
                let reply = match queues[index].handle(message.sender(), message.payload()) {
                    Ok(r) => r,
                    Err(_) => {
                        rejected += 1;
                        continue;
                    }
                };
                (
                    wire::TAG,
                    Outgoing::new(wire::TAG, &reply).unwrap_or_else(|_| fail(32)),
                    boot.apps[index].input_reply,
                )
            } else {
                rejected += 1;
                continue;
            };
            let _ = tag;
            pending[index] = Some((to, payload, tick()));
        }
        for (at, v) in [
            (400, 1),
            (408, received),
            (416, commits),
            (424, input_events),
            (432, rejected),
            (440, scene.focused as u64),
            (448, scene.moves),
            (456, scene.switches),
            (464, surfaces[0].completed_frame().unwrap_or(0)),
            (480, surfaces[1].completed_frame().unwrap_or(0)),
        ] {
            support::put(at, v);
        }
        native::yield_now().unwrap_or_else(|_| fail(33));
    }
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(99)
}

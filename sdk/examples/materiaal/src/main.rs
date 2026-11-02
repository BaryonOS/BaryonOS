#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Real CPL3 adapter. Only IPC endpoints are owned here; the compositor owns
//! display, the isolated source owns raw PS/2, the volume service owns disk.
#[path = "../../system/src/support.rs"]
mod support;
use core::{arch::global_asm, cell::UnsafeCell};
use nieuw_os_catalog_client::NativeTransport;
use nieuw_os_document_session::{
    volume::{Buffers, VolumeSession},
    EditMode,
};
use nieuw_os_input_client::{Binding, Session as Input};
use nieuw_os_materiaal::{controller::Controller, Viewport};
use nieuw_os_materiaal_app::{bootstrap::Bootstrap, App, Evidence, Observation};
use nieuw_os_sdk_runtime::{native, Outgoing, Received, SendEndpoint};
use nieuw_os_surface_protocol::client::Session as Surface;

const TAG: u64 = 0x5359_5354_454d_0001;
const POLL: usize = 100_000;
const CAPACITY: usize = 64 * 1024;
struct CallerBuffers(UnsafeCell<[[u8; CAPACITY]; 3]>);
// SAFETY: Single application thread, acquired once in the nonreturning entry.
// Kernel preemption switches private address spaces and never reenters Rust.
unsafe impl Sync for CallerBuffers {}
#[no_mangle]
static materiaal_document_buffers: CallerBuffers =
    CallerBuffers(UnsafeCell::new([[0; CAPACITY]; 3]));

global_asm!(
    r#"
.section .text.entry,"ax",@progbits
.global _start
_start:
    cld
    // Kernel C-entry RSP is STACK_END-8. Do not push another return address.
    mov rax, rsp
    and eax, 15
    cmp eax, 8
    jne .Lmateriaal_bad_entry_alignment
    jmp materiaal_main
.Lmateriaal_bad_entry_alignment:
    ud2
"#
);

fn fail(code: u64) -> ! {
    support::put(240, code);
    native::exit(1)
}
fn tick() {
    let next = support::word(8).checked_add(1).unwrap_or_else(|| fail(98));
    support::put(8, next);
}
fn send(to: SendEndpoint, operation: u8) {
    let outgoing = Outgoing::new(TAG, &[operation]).unwrap_or_else(|_| fail(90));
    for _ in 0..POLL {
        tick();
        match native::send(to, &outgoing) {
            Ok(()) => return,
            Err(e) if e.is_again() => {
                let _ = native::yield_now();
            }
            Err(_) => fail(91),
        }
    }
    fail(92)
}
fn control(config: &Bootstrap) -> Option<Received> {
    match native::receive(config.control_receive) {
        Ok(message) => {
            if message.cap_count() != 0 {
                let _ = native::close_received_caps(&message);
                fail(93);
            }
            if message.sender() != config.init_pid
                || message.tag() != TAG
                || message.payload().len() != 1
            {
                fail(94);
            }
            Some(message)
        }
        Err(e) if e.is_again() => None,
        Err(_) => fail(95),
    }
}
fn publish(e: Evidence, observation: Observation) {
    // Single CPU: the kernel may observe at a preemption boundary. Publish
    // Ready last, so a partially updated group cannot certify a new snapshot.
    support::put(400, 0);
    for (offset, value) in [
        (408, e.frames),
        (416, e.tiles),
        (424, e.keys),
        (432, e.mice),
        (440, e.saved_revision),
        (448, e.saved_bytes),
        (456, e.saved_checksum),
        (464, e.input_acks),
        (472, e.errors),
        (480, e.save_acks),
        (488, e.superseded),
        (504, observation.screen),
        (512, u64::from(observation.dirty)),
        (520, observation.draft_length),
        (528, observation.draft_checksum),
        (536, observation.object),
        (624, e.presented_generation),
        (632, observation.scale),
        (640, e.f12_presses),
    ] {
        support::put(offset, value);
    }
    support::put(400, u64::from(e.ready));
}

#[no_mangle]
pub extern "C" fn materiaal_main() -> ! {
    native::check_abi().unwrap_or_else(|_| fail(1));
    let config = Bootstrap::read(support::word).unwrap_or_else(|| fail(2));
    // READY here is bootstrap/ELF readiness, not rendered UI or a saved file.
    send(config.registry_send, 1);
    let mut go = false;
    for _ in 0..POLL {
        tick();
        if let Some(message) = control(&config) {
            if message.payload() != [2] {
                fail(3);
            }
            go = true;
            break;
        }
        let _ = native::yield_now();
    }
    if !go {
        fail(4);
    }
    let mut surface = Surface::new(
        NativeTransport,
        config.compositor_pid,
        config.surface_epoch,
        config.surface_send,
        config.surface_receive,
        POLL,
    )
    .unwrap_or_else(|_| fail(5));
    let geometry = surface.geometry().unwrap_or_else(|_| fail(6));
    let viewport = Viewport::new(
        u32::from(geometry.assigned.width),
        u32::from(geometry.assigned.height),
        config.scale,
        geometry.epoch,
    )
    .unwrap_or_else(|_| fail(7));
    let input = Input::new(
        NativeTransport,
        Binding {
            epoch: config.input_epoch,
            service_pid: config.policy_pid,
            client_id: config.input_client,
            width: geometry.width,
            height: geometry.height,
            send: config.input_send,
            receive: config.input_receive,
        },
        POLL,
    )
    .unwrap_or_else(|_| fail(8));
    let volume = VolumeSession::new(
        NativeTransport,
        config.document_epoch,
        config.volume_pid,
        0,
        config.document_send,
        config.document_receive,
        POLL,
    )
    .unwrap_or_else(|_| fail(9));
    // SAFETY: This is the sole access to this static. The nonreturning entry
    // splits it once into three disjoint borrowed arrays, retained by the sole
    // document owner. No references to kernel/foreign memory or host allocation.
    let buffers = unsafe { &mut *materiaal_document_buffers.0.get() };
    let [draft, original, staging] = buffers;
    let mut controller = Controller::new(
        volume,
        if config.read_only {
            EditMode::ReadOnly
        } else {
            EditMode::ReadWrite
        },
        Buffers {
            draft,
            original,
            staging,
        },
        viewport,
        config.input_epoch,
    )
    .unwrap_or_else(|_| fail(10));
    let session_preview = support::word(648) == 1;
    if session_preview {
        controller.enable_vm_desktop().unwrap_or_else(|_| fail(14));
    }
    // A list failure is a genuine controller error screen, never an empty list
    // substituted for unavailable storage. Keep the application alive.
    let _ = controller.refresh();
    // Lives for the nonreturning entry; one digest per tile of the surface.
    let mut damage = [0; nieuw_os_materiaal_app::DAMAGE_TILES];
    let mut app = App::with_damage_cache(controller, input, surface, geometry, &mut damage)
        .unwrap_or_else(|_| fail(11));
    loop {
        tick();
        let _ = if session_preview {
            app.step_interactive()
        } else {
            app.step()
        };
        publish(app.evidence(), app.observation());
        if let Some(message) = control(&config) {
            // STOP is an explicit supervisor request, never an implicit save or
            // discard. A dirty/busy document refuses; all three owners close
            // before DONE. Full OS session/shutdown policy is a separate task.
            if message.payload() != [5] {
                fail(12);
            }
            match app.stop_clean() {
                Ok(()) => {
                    send(config.registry_send, 3);
                    native::exit(0);
                }
                Err(nieuw_os_materiaal_app::Error::Unsaved) => {
                    support::put(496, 1);
                    send(config.registry_send, 6);
                }
                Err(_) => fail(13),
            }
        }
        let _ = native::yield_now();
    }
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(99)
}

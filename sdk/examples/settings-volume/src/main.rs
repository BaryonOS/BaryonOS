#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Native U05-N1 settings app. Two instances (one per volume domain) hold only
//! a send/reply endpoint pair to the volume service and persist typed
//! settings-core records as two volume objects of their own domain. All
//! results are app-reported; tools/run_settings.py checks the raw image.

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!("settings-app is only for the Nieuw OS x86_64-unknown-none target");

mod support;

use nieuw_os_sdk_runtime::{native, BlockDevice, Handle, Outgoing, ReceiveEndpoint, SendEndpoint};
use nieuw_os_settings_core::{Error, Settings, SettingsStore, Theme};
use nieuw_os_settings_volume_example::{code, encode, first, second, VolumeMedia};
use volume_core::{
    client::{Client, Transport},
    wire, Status,
};

const LIMIT: usize = 400_000;
// Case selectors written by the kernel manifest (startup word 112).
const SETTINGS: u64 = 23;
const SETTINGS_RECOVER: u64 = 24;

fn fail(step: u64) -> ! {
    support::put(240, step);
    support::put(40, 0x100 + step);
    native::exit(1)
}

fn check(condition: bool, step: u64) {
    if !condition {
        fail(step);
    }
}

fn tick() {
    support::put(8, support::word(8).wrapping_add(1));
}

struct Native {
    send: SendEndpoint,
    receive: ReceiveEndpoint,
    service: u64,
}

impl Transport for Native {
    fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
        let message = Outgoing::new(wire::TAG, request).map_err(|_| Status::Protocol)?;
        let mut sent = false;
        for _ in 0..LIMIT {
            tick();
            match native::send(self.send, &message) {
                Ok(()) => {
                    sent = true;
                    break;
                }
                Err(error) if error.is_again() => {
                    let _ = native::yield_now();
                }
                Err(_) => return Err(Status::Protocol),
            }
        }
        if !sent {
            return Err(Status::Timeout);
        }
        for _ in 0..LIMIT {
            tick();
            match native::receive(self.receive) {
                Ok(reply) => {
                    let caps = reply.cap_count() != 0;
                    if caps {
                        let _ = native::close_received_caps(&reply);
                    }
                    if caps
                        || reply.sender() != self.service
                        || reply.tag() != wire::TAG
                        || reply.payload().len() != wire::SIZE
                    {
                        // Never resend: the request may already have been processed.
                        return Err(Status::Unknown);
                    }
                    let mut body = [0; wire::SIZE];
                    body.copy_from_slice(reply.payload());
                    return Ok(body);
                }
                Err(error) if error.is_again() => {
                    let _ = native::yield_now();
                }
                Err(_) => return Err(Status::Unknown),
            }
        }
        Err(Status::Unknown)
    }
}

fn report(generation: u64, value: u64, status: u64) {
    support::put(480, generation);
    support::put(488, value);
    support::put(496, status);
}

/// Writable case: initialize then set on blank storage; on an existing
/// record verify the second value and revert to the first after the restart.
fn settings(media: &mut VolumeMedia<Native>, domain: usize) {
    let mut store = SettingsStore::new();
    match store.recover(media) {
        Err(Error::NotInitialized) => {
            check(store.initialize(media, first(domain)) == Ok(1), 20);
            check(store.set(media, second(domain)) == Ok(2), 21);
        }
        Ok(generation) => {
            check(store.get() == Ok(second(domain)), 22);
            check(store.revert(media) == Ok(generation + 1), 23);
            check(store.get() == Ok(first(domain)), 24);
        }
        Err(_) => fail(25),
    }
    let generation = store.generation().unwrap_or_else(|_| fail(26));
    // Out-of-range values cannot be constructed, so nothing reaches storage.
    check(Settings::new(Theme::Dark, 301, false).is_err(), 27);
    check(Settings::new(Theme::Dark, 74, false).is_err(), 28);
    check(store.generation() == Ok(generation), 29);
    // A fresh store over the same objects must see exactly the committed state.
    let mut cold = SettingsStore::new();
    check(cold.recover(media) == Ok(generation), 30);
    let value = cold.get().unwrap_or_else(|_| fail(31));
    check(Ok(value) == store.get(), 32);
    report(generation, encode(value), 0);
}

/// Read-only case: recovery only; a refusal is the reported result, not a failure.
fn settings_recover(media: &mut VolumeMedia<Native>) {
    let mut store = SettingsStore::new();
    match store.recover(media) {
        Ok(generation) => {
            let value = store.get().unwrap_or_else(|_| fail(40));
            report(generation, encode(value), 0);
        }
        Err(error) => report(0, 0, code(error)),
    }
}

fn run() {
    check(native::check_abi().is_ok(), 1);
    let slot = support::word(0);
    check((1..=2).contains(&slot), 2);
    let case = support::word(112);
    check(matches!(case, SETTINGS | SETTINGS_RECOVER), 5);
    let send =
        SendEndpoint::from_handle(Handle::from_raw(support::word(64)).unwrap_or_else(|_| fail(3)));
    let receive = ReceiveEndpoint::from_handle(
        Handle::from_raw(support::word(56)).unwrap_or_else(|_| fail(4)),
    );
    // Apps hold endpoint handles only. Block calls with the own endpoint
    // handle or a forged handle number must be refused by the kernel broker.
    let own = BlockDevice::from_handle(send.handle());
    check(native::block_read(own, 0, 0, 128).is_err(), 6);
    check(native::block_flush(own).is_err(), 7);
    let forged = Handle::from_raw((7 << 32) | 200).unwrap_or_else(|_| fail(8));
    check(
        native::block_read(BlockDevice::from_handle(forged), 0, 0, 128).is_err(),
        9,
    );
    let transport = Native {
        send,
        receive,
        service: support::word(176),
    };
    let domain = (slot - 1) as usize;
    let client = Client::new(transport, support::word(32), 1 + slot * 1_000_000);
    let mut media = VolumeMedia::new(client, domain);
    if domain == 1 {
        // The other domain's settings object is refused by the service.
        check(matches!(media.client.stat(0), Err((Status::Access, _))), 10);
    }
    if case == SETTINGS {
        settings(&mut media, domain);
    } else {
        settings_recover(&mut media);
    }
    support::put(40, 1);
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    run();
    native::exit(0)
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    fail(999)
}

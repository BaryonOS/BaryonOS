#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Minimal userspace catalog service for ordinary SDK-app acceptance.

mod support;

use catalog_core::wire::{self, Catalog, CatalogEntry, Status};
use nieuw_os_sdk_runtime::{native, Handle, Outgoing, ReceiveEndpoint, SendEndpoint};

const NAME: &[u8] = b"Voorbeeld";
const VALUE: &[u8] = b"Nieuw OS SDK";

fn tick() {
    support::put(8, support::word(8).wrapping_add(1));
}

fn receive(endpoint: ReceiveEndpoint, peer: u64) -> Result<nieuw_os_sdk_runtime::Received, ()> {
    for _ in 0..100_000 {
        tick();
        match native::receive(endpoint) {
            Ok(message) => {
                if message.cap_count() != 0
                    || message.sender() != peer
                    || message.tag() != wire::TAG
                    || message.payload().len() != wire::SIZE
                {
                    let _ = native::close_received_caps(&message);
                    return Err(());
                }
                return Ok(message);
            }
            Err(error) if error.is_again() => native::yield_now().map_err(|_| ())?,
            Err(_) => return Err(()),
        }
    }
    Err(())
}

fn send(endpoint: SendEndpoint, payload: &[u8; wire::SIZE]) -> Result<(), ()> {
    let message = Outgoing::new(wire::TAG, payload).map_err(|_| ())?;
    for _ in 0..100_000 {
        tick();
        match native::send(endpoint, &message) {
            Ok(()) => return Ok(()),
            Err(error) if error.is_again() => native::yield_now().map_err(|_| ())?,
            Err(_) => return Err(()),
        }
    }
    Err(())
}

fn catalog() -> Catalog {
    let mut name = [0; wire::NAME];
    name[..NAME.len()].copy_from_slice(NAME);
    Catalog {
        generation: 1,
        count: 1,
        entries: [
            CatalogEntry {
                object: 0,
                revision: 1,
                length: VALUE.len(),
                name_length: NAME.len(),
                name,
            },
            CatalogEntry::EMPTY,
        ],
    }
}

fn run() -> Result<(), ()> {
    native::check_abi().map_err(|_| ())?;
    let epoch = support::word(32);
    let peer = support::word(176);
    if epoch == 0 || peer == 0 {
        return Err(());
    }
    let receive_endpoint =
        ReceiveEndpoint::from_handle(Handle::from_raw(support::word(56)).map_err(|_| ())?);
    let send_endpoint =
        SendEndpoint::from_handle(Handle::from_raw(support::word(64)).map_err(|_| ())?);
    let listing = catalog();
    let mut requests = 0u64;
    let mut replies = 0u64;

    for expected in [wire::LIST, wire::GET] {
        let message = receive(receive_endpoint, peer)?;
        let mut body = [0; wire::SIZE];
        body.copy_from_slice(message.payload());
        let header = wire::validate_request(&body, epoch, 0).map_err(|_| ())?;
        if header.operation != expected {
            return Err(());
        }
        let response = match expected {
            wire::LIST => wire::catalog_reply(header, epoch, Status::Ok, Some(&listing)),
            wire::GET if header.object == 0 => {
                wire::reply(header, epoch, 1, Status::Ok, Some(NAME), Some(VALUE))
            }
            _ => return Err(()),
        };
        requests += 1;
        send(send_endpoint, &response)?;
        replies += 1;
    }

    support::put(240, requests);
    support::put(248, replies);
    support::put(40, 1);
    loop {
        tick();
        native::yield_now().map_err(|_| ())?;
    }
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    if run().is_err() {
        support::put(40, 0x101);
    }
    loop {
        let _ = native::yield_now();
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    support::put(40, 0x1ff);
    loop {
        let _ = native::yield_now();
    }
}

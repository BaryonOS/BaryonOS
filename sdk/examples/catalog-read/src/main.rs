#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Native read-only example using the same B01 client as other catalog apps.
//! The existing M6c bootstrap must provide explicitly granted client endpoints.

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!("catalog-read is only for the Nieuw OS x86_64-unknown-none target");

mod support;
use nieuw_os_catalog_client::{NativeTransport, Operation, Reply, Session};
use nieuw_os_sdk_runtime::{native, Handle, ReceiveEndpoint, SendEndpoint};

fn run() -> Result<(), ()> {
    native::check_abi().map_err(|_| ())?;
    let role = support::word(0);
    if !(1..=2).contains(&role) {
        return Err(());
    }
    let send = SendEndpoint::from_handle(Handle::from_raw(support::word(64)).map_err(|_| ())?);
    let receive =
        ReceiveEndpoint::from_handle(Handle::from_raw(support::word(56)).map_err(|_| ())?);
    let mut catalog = Session::new(
        NativeTransport,
        support::word(32),
        support::word(176),
        (role - 1) as usize,
        send,
        receive,
        100_000,
    )
    .map_err(|_| ())?;
    let listing = match catalog.exchange(Operation::List).map_err(|_| ())? {
        Reply::Catalog(listing) => listing,
        _ => return Err(()),
    };
    let mut total_bytes = 0u64;
    for entry in &listing.entries[..listing.count] {
        let document = match catalog
            .exchange(Operation::Get {
                object: entry.object,
            })
            .map_err(|_| ())?
        {
            Reply::Document(document) => document,
            // A concurrent deletion is an explicit typed NotFound outcome.
            // It does not count as successfully reading the earlier snapshot.
            Reply::NotFound { .. } => return Err(()),
            _ => return Err(()),
        };
        if document.revision != entry.revision
            || document.name() != &entry.name[..entry.name_length]
            || document.value().len() != entry.length
        {
            return Err(());
        }
        total_bytes += document.value().len() as u64;
    }
    catalog.close().map_err(|_| ())?;
    // Local diagnostics only, never a kernel-issued acceptance marker.
    support::put(240, listing.count as u64);
    support::put(248, total_bytes);
    Ok(())
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    match run() {
        Ok(()) => native::exit(0),
        Err(_) => native::exit(1),
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    native::exit(2)
}

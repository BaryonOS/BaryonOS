#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Native M6e volume service. It owns exactly one scoped block capability
//! and two kernel-bound ingress endpoints; the endpoint a request arrives on
//! is its only domain authority. All volume policy lives in `volume-core`.

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!("volume-service is only for the Nieuw OS x86_64-unknown-none target");

mod support;

use nieuw_os_sdk_runtime::{
    block::READ_MAX, native, BlockDevice, Error, Handle, Outgoing, ReceiveEndpoint, SendEndpoint,
    WaitCondition, IPC_WAIT_HORIZON_TICKS,
};
use volume_core::{service::Service, upload, wire, Disk, Status, SECTOR};

const POLL_LIMIT: usize = 200_000;

fn tick() {
    support::put(8, support::word(8).wrapping_add(1));
}

fn status(error: Error) -> Status {
    match error {
        Error::Kernel(code) => Status::from_code(code),
        _ => Status::Protocol,
    }
}

struct NativeDisk {
    device: BlockDevice,
}

impl Disk for NativeDisk {
    fn read(&mut self, sector: u64, buffer: &mut [u8; SECTOR]) -> Result<(), Status> {
        for part in 0..SECTOR / READ_MAX {
            tick();
            let ticket = native::block_read(
                self.device,
                sector,
                (part * READ_MAX) as u64,
                READ_MAX as u64,
            )
            .map_err(status)?;
            native::block_wait(
                self.device,
                ticket,
                &mut buffer[part * READ_MAX..(part + 1) * READ_MAX],
                POLL_LIMIT,
            )
            .map_err(status)?;
        }
        Ok(())
    }
    fn write(&mut self, sector: u64, buffer: &[u8; SECTOR]) -> Result<(), Status> {
        tick();
        let ticket = native::block_write(self.device, sector, buffer).map_err(status)?;
        native::block_wait(self.device, ticket, &mut [], POLL_LIMIT).map_err(status)
    }
    fn flush(&mut self) -> Result<(), Status> {
        tick();
        let ticket = native::block_flush(self.device).map_err(status)?;
        native::block_wait(self.device, ticket, &mut [], POLL_LIMIT).map_err(status)
    }
}

fn evidence(service: &Service) {
    let c = service.counters;
    support::put(128, service.table().generation);
    for (offset, value) in [
        (304, c.commits),
        (312, c.chunk_allocations),
        (320, c.data_sectors),
        (328, c.table_writes),
        (336, c.flushes),
        (344, c.read_sectors),
        (352, c.rejected),
        (360, c.aborted_sessions),
        (368, c.unknown),
        (384, c.transactions),
        (392, c.restores),
    ] {
        support::put(offset, value);
    }
    support::put(376, u64::from(service.uncertain()));
}

fn startup_error(code: Status) -> ! {
    support::put(232, code as i32 as i64 as u64);
    support::put(192, 9);
    // Keep running without serving: the kernel observes the refusal.
    loop {
        tick();
        let _ = native::yield_now();
    }
}

fn handle(raw: u64) -> Handle {
    Handle::from_raw(raw).unwrap_or_else(|_| startup_error(Status::Invalid))
}

fn send(endpoint: SendEndpoint, tag: u64, body: &[u8; wire::SIZE]) {
    let Ok(message) = Outgoing::new(tag, body) else {
        startup_error(Status::Protocol);
    };
    for _ in 0..POLL_LIMIT {
        tick();
        match native::send(endpoint, &message) {
            Ok(()) => return,
            Err(error) if error.is_again() => {
                let _ = native::yield_now();
            }
            Err(_) => return,
        }
    }
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    if native::check_abi().is_err() {
        startup_error(Status::NoSys);
    }
    let idle_wait =
        native::abi_info().unwrap_or_else(|_| startup_error(Status::NoSys)) == (1 << 32) | 10;
    let epoch = support::word(32);
    // Explicit mount mode from the bootstrap; the block capability separately
    // enforces the same boundary (READ only) for read-only mounts.
    let read_only = match support::word(120) {
        0 => false,
        1 => true,
        _ => startup_error(Status::Invalid),
    };
    let device = BlockDevice::from_handle(handle(support::word(48)));
    let roots = [
        ReceiveEndpoint::from_handle(handle(support::word(56))),
        ReceiveEndpoint::from_handle(handle(support::word(80))),
    ];
    let conditions = roots.map(WaitCondition::receive);
    let replies = [
        SendEndpoint::from_handle(handle(support::word(88))),
        SendEndpoint::from_handle(handle(support::word(72))),
    ];
    let clients = [support::word(160), support::word(168)];
    if epoch == 0 || clients.contains(&0) || clients[0] == clients[1] {
        startup_error(Status::Invalid);
    }
    let mut disk = NativeDisk { device };
    let mut service = match Service::recover(epoch, read_only, &mut disk) {
        Ok(service) => service,
        Err(error) => startup_error(error),
    };
    evidence(&service);
    support::put(232, 0);
    support::put(192, 1);
    loop {
        let mut received_any = false;
        for domain in 0..2 {
            tick();
            let message = match native::receive(roots[domain]) {
                Ok(message) => {
                    received_any = true;
                    message
                }
                Err(error) if idle_wait && !error.is_again() => startup_error(status(error)),
                Err(_) => continue,
            };
            let had_caps = message.cap_count() != 0;
            if had_caps {
                let _ = native::close_received_caps(&message);
            }
            if message.sender() != clients[domain] {
                continue;
            }
            let mut body = [0; wire::SIZE];
            let payload = message.payload();
            let token_bound = message.tag() == upload::TAG;
            let reply = if had_caps
                || !matches!(message.tag(), wire::TAG | upload::TAG)
                || payload.len() != wire::SIZE
            {
                body[..payload.len().min(wire::SIZE)]
                    .copy_from_slice(&payload[..payload.len().min(wire::SIZE)]);
                service.counters.rejected += 1;
                let error = wire::error_reply(&wire::header(&body), epoch, Status::Invalid, 0);
                if token_bound {
                    upload::wrap(error, upload::token(&body))
                } else {
                    error
                }
            } else {
                body.copy_from_slice(payload);
                if token_bound {
                    // Sender is kernel-delivered envelope metadata, already
                    // matched to this ingress's authorized client above.
                    service.handle_for(domain, message.sender(), &body, &mut disk)
                } else {
                    service.handle(domain, &body, &mut disk)
                }
            };
            evidence(&service);
            send(
                replies[domain],
                if token_bound { upload::TAG } else { wire::TAG },
                &reply,
            );
        }
        if idle_wait && !received_any {
            // All disk operations and replies above completed synchronously.
            let deadline = native::monotonic_ticks()
                .ok()
                .and_then(|now| now.checked_add(IPC_WAIT_HORIZON_TICKS))
                .unwrap_or_else(|| startup_error(Status::Protocol));
            if let Err(error) = native::ipc_wait(&conditions, deadline) {
                startup_error(status(error));
            }
        } else {
            let _ = native::yield_now();
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    support::put(232, Status::Protocol as i32 as i64 as u64);
    native::exit(3)
}

//! Block access through the per-device broker (spec 2.7, "Meaning of kernel
//! -8" and "Time source"), and the only two write paths of 2.7 step 2.
//!
//! The broker is the kernel's: a submission (syscalls 0x40, 0x42, 0x43) and a
//! completion poll (0x41), plus the boot-monotonic tick count (syscall 3).
//! The service is the device's only client under K5 and consumes the result
//! of every request before it submits the next, so at most one request is
//! ever outstanding.
use crate::{catalog::COPY_SECTORS, Status, DATA_START, SECTOR};

/// Kernel return codes the core interprets.
pub const AGAIN: i64 = -8;
pub const KERNEL_TIMEOUT: i64 = -13;
pub const KERNEL_NO_DEVICE: i64 = -14;
pub const KERNEL_IO: i64 = -12;
pub const KERNEL_UNKNOWN: i64 = -16;

/// Broker request timeout and reset timeout in ticks (`volume.rs:24-25`).
pub const TIMEOUT: u64 = 500;
pub const RESET_TIMEOUT: u64 = 10;
/// A submission that keeps returning -8 for longer than this is a device `Io`.
pub const SUBMIT_BOUND: u64 = 2 * RESET_TIMEOUT;
/// A completion poll that keeps returning -8 for longer than this after the
/// submission is `Unknown` (write, flush) or `Timeout` (read).
pub const POLL_BOUND: u64 = TIMEOUT + 2 * RESET_TIMEOUT;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request<'a> {
    Read(u64),
    Write(u64, &'a [u8; SECTOR]),
    Flush,
}

impl Request<'_> {
    pub fn mutating(&self) -> bool {
        !matches!(self, Self::Read(_))
    }
}

/// Kernel block broker for the destination scope. Sectors are scope-relative.
pub trait Broker {
    /// Submit one request: 0 accepted, -8 not admitted now, other negative
    /// values are kernel errors (-14 absent or offline, -13 quarantined).
    fn submit(&mut self, request: Request<'_>) -> i64;
    /// Poll the outstanding request: -8 still pending, 0 success (a read has
    /// filled `buffer`), other negative values are the device result.
    fn poll(&mut self, buffer: &mut [u8; SECTOR]) -> i64;
    /// Boot-monotonic tick count (nominally 100 Hz).
    fn now(&mut self) -> u64;
}

/// Device result of a failed request, as the reply statuses of 2.5 use it.
pub fn submit_error(code: i64) -> Status {
    match code {
        KERNEL_NO_DEVICE => Status::NoDevice,
        // A submission refused for a quarantined (offline, -13) device is a
        // device failure answered `Io` (5.3).
        _ => Status::Io,
    }
}

pub fn poll_error(code: i64) -> Status {
    match code {
        KERNEL_NO_DEVICE => Status::NoDevice,
        KERNEL_TIMEOUT => Status::Timeout,
        KERNEL_UNKNOWN => Status::Unknown,
        _ => Status::Io,
    }
}

/// Run one request to completion within the bounds of 2.7.
pub fn run<B: Broker>(
    broker: &mut B,
    request: Request<'_>,
    buffer: &mut [u8; SECTOR],
) -> Result<(), Status> {
    let start = broker.now();
    loop {
        match broker.submit(request) {
            0 => break,
            AGAIN => {
                if broker.now().saturating_sub(start) > SUBMIT_BOUND {
                    return Err(Status::Io);
                }
            }
            code => return Err(submit_error(code)),
        }
    }
    let submitted = broker.now();
    loop {
        match broker.poll(buffer) {
            0 => return Ok(()),
            AGAIN => {
                if broker.now().saturating_sub(submitted) > POLL_BOUND {
                    return Err(if request.mutating() {
                        Status::Unknown
                    } else {
                        Status::Timeout
                    });
                }
            }
            code => return Err(poll_error(code)),
        }
    }
}

pub fn read<B: Broker>(
    broker: &mut B,
    sector: u64,
    buffer: &mut [u8; SECTOR],
) -> Result<(), Status> {
    run(broker, Request::Read(sector), buffer)
}

pub fn flush<B: Broker>(broker: &mut B) -> Result<(), Status> {
    run(broker, Request::Flush, &mut [0; SECTOR])
}

/// Entry write path: only `max(16, next_free) <= sector < scope end`, with
/// `next_free` the committed value. It never relies on catalog validation to
/// keep sectors 0-15 out of reach. A refused write never reaches the device.
pub fn write_entry<B: Broker>(
    broker: &mut B,
    committed_next_free: u64,
    scope_sectors: u64,
    sector: u64,
    data: &[u8; SECTOR],
) -> Result<(), Status> {
    if sector < DATA_START.max(committed_next_free) || sector >= scope_sectors {
        return Err(Status::Access);
    }
    run(broker, Request::Write(sector, data), &mut [0; SECTOR])
}

/// Catalog write path: only the eight catalog sectors 0-7.
pub fn write_catalog<B: Broker>(
    broker: &mut B,
    sector: u64,
    data: &[u8; SECTOR],
) -> Result<(), Status> {
    if sector >= 2 * COPY_SECTORS {
        return Err(Status::Access);
    }
    run(broker, Request::Write(sector, data), &mut [0; SECTOR])
}

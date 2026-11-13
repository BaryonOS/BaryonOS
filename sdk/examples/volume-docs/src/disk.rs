//! A scoped block capability as a `volume_core::Disk`, shared by the volume
//! service and the S09 archive portal. The kernel broker enforces the scope
//! and rights; this adapter only splits sector reads into broker-sized parts.
use nieuw_os_sdk_runtime::{block::READ_MAX, native, BlockDevice, Error};
use volume_core::{Disk, Status, SECTOR};

use crate::support;

pub const POLL_LIMIT: usize = 200_000;

pub fn tick() {
    support::put(8, support::word(8).wrapping_add(1));
}

pub fn status(error: Error) -> Status {
    match error {
        Error::Kernel(code) => Status::from_code(code),
        _ => Status::Protocol,
    }
}

pub struct NativeDisk {
    pub device: BlockDevice,
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

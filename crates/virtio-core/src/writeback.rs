//! Pure plans for the first M6a WRITE/FLUSH step; not a transport or authority.
//!
//! These helpers do not publish descriptors, copy a write payload, acknowledge
//! FEATURES_OK or manage an outstanding request. The caller must authorize each
//! operation, copy exactly one initialized sector into its pinned bounce page,
//! and preserve all exposed buffers until completion or acknowledged reset.
//! WRITE completion is not durability, and reset does not undo a partial write.

use crate::{
    Completion, Descriptor, DmaPool, Error, ACCESS_PLATFORM, NEXT, READ_ONLY, VERSION_1, WRITE,
};

/// VIRTIO_BLK_F_FLUSH (feature bit 9), not the FLUSH request opcode (4).
pub const FLUSH: u64 = 1 << 9;

/// Validated feature proposal for this strictly writable, flush-capable profile.
///
/// Construction only checks an offered feature snapshot. The transport still
/// has to write these bits, set FEATURES_OK and verify device acceptance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Features {
    accepted_bits: u64,
}

impl Features {
    pub fn negotiate(offered: u64) -> Result<Self, Error> {
        if offered & (VERSION_1 | FLUSH) != VERSION_1 | FLUSH
            || offered & (READ_ONLY | ACCESS_PLATFORM) != 0
        {
            return Err(Error::Unsupported);
        }
        Ok(Self {
            accepted_bits: VERSION_1 | FLUSH,
        })
    }

    pub const fn accepted_bits(self) -> u64 {
        self.accepted_bits
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    /// Exactly one complete 512-byte sector; no partial-sector read/modify/write.
    Write { sector: u64 },
    /// Flush the device cache; sector zero and no data descriptor.
    Flush,
}

/// Immutable checked request bytes and the exact active descriptor chain.
///
/// Head is always descriptor 0. The full five-page pool is checked even for
/// FLUSH, which does not expose the bounce page. No userspace address or length
/// is accepted. A valid plan does not prove frame ownership, pinning or IOMMU
/// protection; those remain properties of the future transport implementation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plan {
    operation: Operation,
    header: [u8; 16],
    descriptors: [Descriptor; 3],
    count: usize,
}

impl Plan {
    pub fn new(
        _features: Features,
        operation: Operation,
        pool: DmaPool,
        ram_end: u64,
        capacity: u64,
    ) -> Result<Self, Error> {
        pool.validate(ram_end)?;
        let (kind, sector, count) = match operation {
            Operation::Write { sector } => {
                if sector >= capacity {
                    return Err(Error::Range);
                }
                (1u32, sector, 3)
            }
            Operation::Flush => (4, 0, 2),
        };
        let mut header = [0; 16];
        header[..4].copy_from_slice(&kind.to_le_bytes());
        header[8..].copy_from_slice(&sector.to_le_bytes());
        let status = Descriptor {
            address: pool.status_address()?,
            length: 1,
            flags: WRITE,
            next: 0,
        };
        let mut descriptors = [
            Descriptor {
                address: pool.request,
                length: 16,
                flags: NEXT,
                next: 1,
            },
            status,
            Descriptor {
                address: 0,
                length: 0,
                flags: 0,
                next: 0,
            },
        ];
        if matches!(operation, Operation::Write { .. }) {
            descriptors[1] = Descriptor {
                address: pool.bounce,
                length: 512,
                flags: NEXT,
                next: 2,
            };
            descriptors[2] = status;
        }
        Ok(Self {
            operation,
            header,
            descriptors,
            count,
        })
    }

    pub const fn operation(&self) -> Operation {
        self.operation
    }

    pub const fn header(&self) -> &[u8; 16] {
        &self.header
    }

    /// Only the active chain: three descriptors for WRITE, two for FLUSH.
    pub fn descriptors(&self) -> &[Descriptor] {
        &self.descriptors[..self.count]
    }

    /// Check the entire draft before the transport publishes any descriptor.
    /// Extra descriptors, opcode/sector changes and nonzero reserved bytes fail.
    pub fn validate(&self, header: &[u8; 16], descriptors: &[Descriptor]) -> Result<(), Error> {
        if header != &self.header || descriptors != self.descriptors() {
            return Err(Error::Protocol);
        }
        Ok(())
    }

    /// Only status is device-writable in either chain, so used length is one.
    /// Pending ignores stale ring metadata. A terminal device error never
    /// authorizes data exposure or proves that the write had no side effects.
    pub fn validate_used(
        &self,
        old_idx: u16,
        new_idx: u16,
        used_id: u32,
        used_len: u32,
        status: u8,
    ) -> Result<Completion, Error> {
        let delta = new_idx.wrapping_sub(old_idx);
        if delta == 0 {
            return Err(Error::Pending);
        }
        if delta != 1 || used_id != 0 || used_len != 1 {
            return Err(Error::Protocol);
        }
        match status {
            0 => Ok(Completion::Success),
            1 => Ok(Completion::Io),
            2 => Ok(Completion::Unsupported),
            _ => Err(Error::Protocol),
        }
    }
}

#[cfg(test)]
mod tests;

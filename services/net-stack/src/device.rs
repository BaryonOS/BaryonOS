use crate::Error;
use smoltcp::{
    phy::{self, DeviceCapabilities, Medium},
    time::Instant,
};

pub const FRAME_BYTES: usize = 1514;

#[derive(Clone, Copy, Debug)]
pub struct Transmission<'a> {
    pub id: u64,
    pub bytes: &'a [u8],
}

/// Backpressure remains in effect until explicit matching completion. All data
/// buffers are ordinary user memory; the kernel driver owns its DMA bounce pool.
pub struct PacketDevice<'a> {
    rx: &'a mut [u8; FRAME_BYTES],
    tx: &'a mut [u8; FRAME_BYTES],
    rx_len: usize,
    tx_len: usize,
    generation: u64,
}

impl<'a> PacketDevice<'a> {
    pub fn new(rx: &'a mut [u8; FRAME_BYTES], tx: &'a mut [u8; FRAME_BYTES]) -> Self {
        Self {
            rx,
            tx,
            rx_len: 0,
            tx_len: 0,
            generation: 0,
        }
    }

    pub fn can_receive(&self) -> bool {
        self.rx_len == 0
    }

    pub(crate) fn discard_receive(&mut self) {
        self.rx_len = 0;
    }

    pub fn push(&mut self, frame: &[u8]) -> Result<(), Error> {
        if !(14..=FRAME_BYTES).contains(&frame.len()) {
            return Err(Error::InvalidFrame);
        }
        if !self.can_receive() {
            return Err(Error::Busy);
        }
        self.rx[..frame.len()].copy_from_slice(frame);
        self.rx_len = frame.len();
        Ok(())
    }

    pub fn transmission(&self) -> Option<Transmission<'_>> {
        (self.tx_len != 0).then(|| Transmission {
            id: self.generation,
            bytes: &self.tx[..self.tx_len],
        })
    }

    pub(crate) fn queue_control(&mut self, frame: &[u8]) -> Result<u64, Error> {
        if !(14..=FRAME_BYTES).contains(&frame.len()) {
            return Err(Error::InvalidFrame);
        }
        if self.tx_len != 0 {
            return Err(Error::Busy);
        }
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(Error::IdentifierExhausted)?;
        self.tx[..frame.len()].copy_from_slice(frame);
        self.generation = generation;
        self.tx_len = frame.len();
        Ok(generation)
    }

    pub fn complete(&mut self, id: u64) -> Result<(), Error> {
        if self.tx_len == 0 || id == 0 || self.generation != id {
            return Err(Error::StaleTransmission);
        }
        self.tx_len = 0;
        Ok(())
    }
}

pub struct Rx<'a> {
    bytes: &'a [u8],
}
pub struct Tx<'a> {
    bytes: &'a mut [u8; FRAME_BYTES],
    length: &'a mut usize,
    generation: &'a mut u64,
}

impl phy::RxToken for Rx<'_> {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(self.bytes)
    }
}

impl phy::TxToken for Tx<'_> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        // Trusted smoltcp must honor advertised MTU. Fail before publishing on a
        // violated upstream invariant; never hand out an out-of-bounds slice.
        assert!((14..=FRAME_BYTES).contains(&len));
        assert_eq!(*self.length, 0);
        let generation = self
            .generation
            .checked_add(1)
            .expect("TX identity exhausted");
        let result = f(&mut self.bytes[..len]);
        *self.generation = generation;
        *self.length = len;
        result
    }
}

impl phy::Device for PacketDevice<'_> {
    type RxToken<'b>
        = Rx<'b>
    where
        Self: 'b;
    type TxToken<'b>
        = Tx<'b>
    where
        Self: 'b;

    fn receive(&mut self, _: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        if self.rx_len == 0 || self.tx_len != 0 || self.generation == u64::MAX {
            return None;
        }
        let length = self.rx_len;
        self.rx_len = 0;
        Some((
            Rx {
                bytes: &self.rx[..length],
            },
            Tx {
                bytes: self.tx,
                length: &mut self.tx_len,
                generation: &mut self.generation,
            },
        ))
    }

    fn transmit(&mut self, _: Instant) -> Option<Self::TxToken<'_>> {
        if self.tx_len != 0 || self.generation == u64::MAX {
            return None;
        }
        Some(Tx {
            bytes: self.tx,
            length: &mut self.tx_len,
            generation: &mut self.generation,
        })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut capabilities = DeviceCapabilities::default();
        capabilities.medium = Medium::Ethernet;
        capabilities.max_transmission_unit = FRAME_BYTES;
        capabilities.max_burst_size = Some(1);
        capabilities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhausted_identity_cannot_publish_control_or_wrap_to_an_old_id() {
        let mut rx = [0; FRAME_BYTES];
        let mut tx = [0xa5; FRAME_BYTES];
        let mut device = PacketDevice::new(&mut rx, &mut tx);
        device.generation = u64::MAX;
        assert_eq!(
            device.queue_control(&[7; 60]),
            Err(Error::IdentifierExhausted)
        );
        assert!(device.transmission().is_none());
        assert_eq!(device.tx, &[0xa5; FRAME_BYTES]);
        assert_eq!(device.complete(u64::MAX), Err(Error::StaleTransmission));
    }
}

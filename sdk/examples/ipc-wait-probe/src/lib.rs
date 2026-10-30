#![no_std]
#![forbid(unsafe_code)]

use nieuw_os_sdk_runtime::{Error, Outgoing, Received};

pub const TAG: u64 = 0x5741_4954_5052_0001;
pub const QUEUE_CAPACITY: u32 = 16;
pub const PHASE_TICKS: u64 = 200;
pub const SETTLE_TICKS: u64 = 40;
pub const EMPTY: u8 = 1;
pub const DATA: u8 = 2;
pub const IDLE: u8 = 3;
pub const IDLE_ACK: u8 = 4;
pub const FULL: u8 = 5;
pub const FULL_ACK: u8 = 6;
pub const ITEM: u8 = 7;
pub const DRAINED: u8 = 8;
pub const REVOKE: u8 = 9;
pub const REVOKE_ACK: u8 = 10;
pub const REVOKED: u8 = 11;
pub const SELF: u8 = 12;
pub const NEGATIVES_BIT: u64 = 1 << 0;
pub const IMMEDIATE_BIT: u64 = 1 << 1;
pub const EMPTY_BIT: u64 = 1 << 2;
pub const IDLE_BIT: u64 = 1 << 3;
pub const FULL_BIT: u64 = 1 << 4;
pub const REVOKE_BIT: u64 = 1 << 5;
pub const ROLE_ZERO_MASK: u64 = 0x3f;
pub const ROLE_ONE_MASK: u64 = 0x3c;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Packet {
    pub kind: u8,
    pub sequence: u32,
    pub at: u64,
}

impl Packet {
    pub const fn new(kind: u8, sequence: u32, at: u64) -> Self {
        Self { kind, sequence, at }
    }

    pub fn outgoing(self) -> Result<Outgoing, Error> {
        let mut bytes = [0; 16];
        bytes[0] = self.kind;
        bytes[4..8].copy_from_slice(&self.sequence.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.at.to_le_bytes());
        Outgoing::new(TAG, &bytes)
    }

    pub fn received(message: &Received, sender: u64) -> Result<Self, Error> {
        let bytes = message.payload();
        if sender == 0
            || message.sender() != sender
            || message.tag() != TAG
            || message.cap_count() != 0
            || bytes.len() != 16
            || !(EMPTY..=SELF).contains(&bytes[0])
            || bytes[1..4] != [0; 3]
        {
            return Err(Error::InvalidMessage);
        }
        Ok(Self {
            kind: bytes[0],
            sequence: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            at: u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
        })
    }
}

pub fn deadline(now: u64, ticks: u64) -> Option<u64> {
    now.checked_add(ticks)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn incoming(packet: Packet) -> Received {
        let mut bytes = *packet.outgoing().unwrap().as_bytes();
        bytes[..8].copy_from_slice(&47u64.to_le_bytes());
        Received::from_bytes(bytes, 16).unwrap()
    }

    #[test]
    fn every_phase_keeps_exact_payload_and_sender() {
        for kind in EMPTY..=SELF {
            let packet = Packet::new(kind, u32::MAX, 0x1234_5678_9abc_def0);
            assert_eq!(Packet::received(&incoming(packet), 47), Ok(packet));
        }
    }

    #[test]
    fn wrong_sender_or_envelope_cannot_be_an_ack() {
        let packet = Packet::new(DATA, 0, 0);
        assert_eq!(
            Packet::received(&incoming(packet), 48),
            Err(Error::InvalidMessage)
        );
        for (offset, value) in [(8, 0), (64, 0), (65, 1)] {
            let mut bytes = *incoming(packet).as_bytes();
            bytes[offset] = value;
            let message = Received::from_bytes(bytes, 16).unwrap();
            assert_eq!(Packet::received(&message, 47), Err(Error::InvalidMessage));
        }
        let mut bytes = *incoming(packet).as_bytes();
        bytes[16..20].copy_from_slice(&15u32.to_le_bytes());
        let message = Received::from_bytes(bytes, 15).unwrap();
        assert_eq!(Packet::received(&message, 47), Err(Error::InvalidMessage));
    }

    #[test]
    fn absolute_deadlines_do_not_wrap() {
        assert_eq!(deadline(50, PHASE_TICKS), Some(250));
        assert_eq!(
            deadline(u64::MAX - PHASE_TICKS, PHASE_TICKS),
            Some(u64::MAX)
        );
        assert_eq!(deadline(u64::MAX, 1), None);
    }
}

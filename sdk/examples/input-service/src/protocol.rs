//! Exact command reply checks for the source side of INP1. Does not decode
//! desktop input on the host or create any Received envelope/capability.
use nieuw_os_input_core::wire::{self, Request};
use nieuw_os_sdk_runtime::Received;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invalid;
fn word(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
pub fn command_reply(message: &Received, peer: u64, request: Request) -> Result<u16, Invalid> {
    let bytes = message.payload();
    if message.sender() != peer
        || message.cap_count() != 0
        || message.tag() != wire::TAG
        || bytes.len() != wire::SIZE
        || &bytes[..4] != b"INP1"
        || u16::from_le_bytes([bytes[4], bytes[5]]) != wire::VERSION
        || u16::from_le_bytes([bytes[6], bytes[7]]) != (request.operation as u16 | wire::REPLY)
        || word(bytes, 8) != request.id
        || word(bytes, 16) != request.epoch
        || word(bytes, 32) != 0
        || bytes[40..].iter().any(|byte| *byte != 0)
    {
        return Err(Invalid);
    }
    let status = word(bytes, 24);
    if status == 0 || (2..=18).contains(&status) {
        Ok(status as u16)
    } else {
        Err(Invalid)
    }
}

//! Fixed coordination codecs only. PID authority comes from real kernel IPC.
use crate::boot::Bootstrap;
use nieuw_os_sdk_runtime::{Outgoing, Received};
pub const SYSTEM: u64 = 0x5359_5354_454d_0001;
pub const READY: u8 = 1;
pub const GO: u8 = 2;
pub const DONE: u8 = 3;
pub const STOP: u8 = 5;
pub const STOP_REFUSED: u8 = 6;
pub const IPD: u64 = u64::from_le_bytes(*b"IPDREAD1");
pub const CMP: u64 = u64::from_le_bytes(*b"CMPCTRL1");
pub const CONTROL: u64 = u64::from_le_bytes(*b"DINITCTL");
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invalid;
fn n(b: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(b[o..o + 8].try_into().unwrap())
}
pub fn ready(config: &Bootstrap, m: &Received) -> Result<u8, Invalid> {
    let b = m.payload();
    if m.cap_count() != 0
        || m.tag() != IPD
        || b.len() != 32
        || &b[..4] != b"IPD1"
        || b[4..6] != 1u16.to_le_bytes()
        || n(b, 16) != config.generation
    {
        return Err(Invalid);
    }
    let role = u16::from_le_bytes([b[6], b[7]]);
    let epoch = n(b, 8);
    let raw = n(b, 24);
    match (m.sender(), role) {
        (p, 1) if p == config.source_pid && epoch == config.policy_epoch && raw > 0 => Ok(1),
        (p, 2) if p == config.policy_pid && epoch == config.policy_epoch && raw == 0 => Ok(2),
        (p, 3)
            if p == config.compositor_pid
                && epoch == config.surface_epoch
                && raw == config.native_display_epoch =>
        {
            Ok(4)
        }
        _ => Err(Invalid),
    }
}
pub fn system(m: &Received, peer: u64) -> Result<u8, Invalid> {
    if m.sender() != peer || m.cap_count() != 0 || m.tag() != SYSTEM || m.payload().len() != 1 {
        return Err(Invalid);
    }
    Ok(m.payload()[0])
}
pub fn app(operation: u8) -> Result<Outgoing, Invalid> {
    Outgoing::new(SYSTEM, &[operation]).map_err(|_| Invalid)
}
pub fn compositor(config: &Bootstrap, id: u64, retire: bool) -> Result<Outgoing, Invalid> {
    let mut b = [0; 48];
    b[..4].copy_from_slice(b"CMP1");
    b[4..6].copy_from_slice(&1u16.to_le_bytes());
    b[6..8].copy_from_slice(&(if retire { 3u16 } else { 1u16 }).to_le_bytes());
    for (o, v) in [(8, id), (16, config.surface_epoch), (24, config.generation)] {
        b[o..o + 8].copy_from_slice(&v.to_le_bytes());
    }
    Outgoing::new(CMP, &b).map_err(|_| Invalid)
}
pub fn compositor_reply(
    config: &Bootstrap,
    m: &Received,
    id: u64,
    retire: bool,
) -> Result<(), Invalid> {
    let b = m.payload();
    if m.sender() != config.compositor_pid
        || m.cap_count() != 0
        || m.tag() != CMP
        || b.len() != 48
        || &b[..4] != b"CMP1"
        || b[4..6] != 1u16.to_le_bytes()
        || u16::from_le_bytes([b[6], b[7]]) != (0x8000 | if retire { 3 } else { 1 })
        || n(b, 8) != id
        || n(b, 16) != config.surface_epoch
        || n(b, 24) != config.generation
        || n(b, 32) != 0
        || n(b, 40) != if retire { 2 } else { 1 }
    {
        return Err(Invalid);
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StopRequest {
    pub id: u64,
    pub epoch: u64,
    pub generation: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum StopStatus {
    Stopped = 0,
    Refused = 1,
    Failed = 2,
}
impl StopRequest {
    pub fn encode(self) -> [u8; 48] {
        let mut b = [0; 48];
        b[..4].copy_from_slice(b"DIN1");
        b[4..6].copy_from_slice(&1u16.to_le_bytes());
        b[6..8].copy_from_slice(&1u16.to_le_bytes());
        for (o, v) in [(8, self.id), (16, self.epoch), (24, self.generation)] {
            b[o..o + 8].copy_from_slice(&v.to_le_bytes());
        }
        b
    }
    pub fn decode(config: &Bootstrap, m: &Received) -> Result<Self, Invalid> {
        let controller = config.controller.ok_or(Invalid)?;
        let b = m.payload();
        if m.sender() != controller.pid
            || m.tag() != CONTROL
            || m.cap_count() != 0
            || b.len() != 48
            || &b[..4] != b"DIN1"
            || b[4..6] != 1u16.to_le_bytes()
            || b[6..8] != 1u16.to_le_bytes()
            || b[32..].iter().any(|x| *x != 0)
        {
            return Err(Invalid);
        }
        let result = Self {
            id: n(b, 8),
            epoch: n(b, 16),
            generation: n(b, 24),
        };
        if result.id == 0
            || result.epoch != config.surface_epoch
            || result.generation != config.generation
        {
            return Err(Invalid);
        }
        Ok(result)
    }
    pub fn reply(self, status: StopStatus) -> Result<Outgoing, Invalid> {
        let mut b = self.encode();
        b[6..8].copy_from_slice(&0x8001u16.to_le_bytes());
        b[32..40].copy_from_slice(&(status as u64).to_le_bytes());
        Outgoing::new(CONTROL, &b).map_err(|_| Invalid)
    }
}

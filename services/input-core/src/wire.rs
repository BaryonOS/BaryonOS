//! Fixed INP1 payload; no application-controlled field grants authority.
use crate::{CancelReason, Error, Event};

pub const SIZE: usize = 96;
pub const TAG: u64 = 0x4e49_4e50_5554_0001;
pub const VERSION: u16 = 1;
pub const REPLY: u16 = 0x8000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Operation {
    Sync = 1,
    Sample = 2,
    Focus = 16,
    ClientDead = 32,
    SourceDead = 33,
    CompositorDead = 34,
    Peek = 48,
    Ack = 49,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub operation: Operation,
    pub id: u64,
    pub epoch: u64,
    pub arg0: u64,
    pub arg1: u64,
}

fn get64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
fn put64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

impl Request {
    pub fn encode(self) -> [u8; SIZE] {
        let mut out = [0; SIZE];
        out[..4].copy_from_slice(b"INP1");
        out[4..6].copy_from_slice(&VERSION.to_le_bytes());
        out[6..8].copy_from_slice(&(self.operation as u16).to_le_bytes());
        put64(&mut out, 8, self.id);
        put64(&mut out, 16, self.epoch);
        put64(&mut out, 24, self.arg0);
        put64(&mut out, 32, self.arg1);
        out
    }

    pub fn decode(bytes: &[u8], epoch: u64) -> Result<Self, Error> {
        if bytes.len() != SIZE
            || &bytes[..4] != b"INP1"
            || u16::from_le_bytes([bytes[4], bytes[5]]) != VERSION
            || bytes[40..].iter().any(|b| *b != 0)
        {
            return Err(Error::Malformed);
        }
        let operation = match u16::from_le_bytes([bytes[6], bytes[7]]) {
            1 => Operation::Sync,
            2 => Operation::Sample,
            16 => Operation::Focus,
            32 => Operation::ClientDead,
            33 => Operation::SourceDead,
            34 => Operation::CompositorDead,
            48 => Operation::Peek,
            49 => Operation::Ack,
            _ => return Err(Error::Malformed),
        };
        let request = Self {
            operation,
            id: get64(bytes, 8),
            epoch: get64(bytes, 16),
            arg0: get64(bytes, 24),
            arg1: get64(bytes, 32),
        };
        if request.epoch != epoch {
            return Err(Error::WrongEpoch);
        }
        if request.id == 0 {
            return Err(Error::Malformed);
        }
        let valid = match operation {
            Operation::Sync => request.arg0 == 1 && request.arg1 == 0,
            Operation::Sample => request.arg0 <= u8::MAX as u64 && request.arg1 <= u8::MAX as u64,
            Operation::Focus => request.arg1 == 0,
            Operation::ClientDead | Operation::Ack => request.arg0 != 0 && request.arg1 == 0,
            Operation::SourceDead | Operation::CompositorDead | Operation::Peek => {
                request.arg0 == 0 && request.arg1 == 0
            }
        };
        if !valid {
            return Err(Error::Malformed);
        }
        Ok(request)
    }
}

/// Canonical successful/empty response. A failed `handle` returns an Error;
/// the adapter must expose it as an error, never substitute a success reply.
pub(crate) fn response(
    request: Request,
    client: u64,
    event: Option<(u64, u64, Event)>,
) -> [u8; SIZE] {
    let mut bytes = request.encode();
    bytes[6..8].copy_from_slice(&((request.operation as u16) | REPLY).to_le_bytes());
    put64(
        &mut bytes,
        24,
        u64::from(request.operation == Operation::Peek && event.is_none()),
    );
    put64(&mut bytes, 32, client);
    if let Some((sequence, generation, event)) = event {
        put64(&mut bytes, 48, sequence);
        put64(&mut bytes, 56, generation);
        match event {
            Event::Key(key) => {
                bytes[40] = 1;
                bytes[41] = match key.action {
                    ui_core::KeyAction::Press => 1,
                    ui_core::KeyAction::Repeat => 2,
                    ui_core::KeyAction::Release => 3,
                    ui_core::KeyAction::Pulse => 4,
                };
                bytes[42] = key.key as u8;
                let m = key.modifiers;
                let flags = [
                    m.left_shift,
                    m.right_shift,
                    m.left_control,
                    m.right_control,
                    m.left_alt,
                    m.right_alt,
                    m.left_super,
                    m.right_super,
                    m.caps_lock,
                    m.num_lock,
                    m.scroll_lock,
                ];
                let mut bits = 0u16;
                for (index, enabled) in flags.iter().enumerate() {
                    bits |= u16::from(*enabled) << index;
                }
                bytes[44..46].copy_from_slice(&bits.to_le_bytes());
            }
            Event::Mouse { input, x, y } => {
                bytes[40] = 2;
                bytes[41] = u8::from(input.x_overflow) | (u8::from(input.y_overflow) << 1);
                bytes[42] = input.buttons.bits();
                bytes[43] = input.changed.bits();
                bytes[44..46].copy_from_slice(&input.dx.to_le_bytes());
                bytes[46..48].copy_from_slice(&input.dy.to_le_bytes());
                bytes[64..68].copy_from_slice(&x.to_le_bytes());
                bytes[68..72].copy_from_slice(&y.to_le_bytes());
            }
            Event::Cancel(reason) => {
                bytes[40] = 3;
                bytes[41] = match reason {
                    CancelReason::FocusLost => 1,
                    CancelReason::SourceLost => 2,
                    CancelReason::QueueFull => 3,
                    CancelReason::Resync => 4,
                    CancelReason::SourceDead => 5,
                    CancelReason::CompositorDead => 6,
                };
            }
        }
    }
    bytes
}

pub(crate) fn failure(request: Request, client: u64, error: Error) -> [u8; SIZE] {
    let mut bytes = response(request, client, None);
    put64(&mut bytes, 24, u64::from(error as u16));
    bytes
}

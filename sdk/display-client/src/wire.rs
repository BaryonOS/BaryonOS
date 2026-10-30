//! Canonical fixed 32-byte hardware records, never a GUI or drawing language.
pub const SIZE: usize = 32;
pub const TILE_WIDTH: u16 = 32;
pub const TILE_HEIGHT: u16 = 8;
pub const MAX_PIXELS: usize = TILE_WIDTH as usize * TILE_HEIGHT as usize;
pub const MAX_BYTES: usize = MAX_PIXELS * 4;
pub const FORMAT: u32 = 1;
pub const INPUT_PROFILE: u32 = 1;
pub const INPUT_OPERATIONS: u32 = 8192;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invalid;
fn u16at(b: &[u8; SIZE], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}
fn u32at(b: &[u8; SIZE], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn u64at(b: &[u8; SIZE], o: usize) -> u64 {
    u64::from_le_bytes(b[o..o + 8].try_into().unwrap())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Geometry {
    pub width: u32,
    pub height: u32,
    pub epoch: u64,
}
impl Geometry {
    pub fn validate(self) -> Result<(), Invalid> {
        if self.width == 0
            || self.width > 1920
            || self.height == 0
            || self.height > 1080
            || self.epoch == 0
        {
            Err(Invalid)
        } else {
            Ok(())
        }
    }
    pub fn encode(self) -> Result<[u8; SIZE], Invalid> {
        self.validate()?;
        let mut b = [0; SIZE];
        b[0..4].copy_from_slice(&self.width.to_le_bytes());
        b[4..8].copy_from_slice(&self.height.to_le_bytes());
        b[8..16].copy_from_slice(&self.epoch.to_le_bytes());
        b[16..18].copy_from_slice(&TILE_WIDTH.to_le_bytes());
        b[18..20].copy_from_slice(&TILE_HEIGHT.to_le_bytes());
        b[20..24].copy_from_slice(&FORMAT.to_le_bytes());
        Ok(b)
    }
    pub fn decode(b: &[u8; SIZE]) -> Result<Self, Invalid> {
        if u16at(b, 16) != TILE_WIDTH
            || u16at(b, 18) != TILE_HEIGHT
            || u32at(b, 20) != FORMAT
            || b[24..].iter().any(|x| *x != 0)
        {
            return Err(Invalid);
        }
        let g = Self {
            width: u32at(b, 0),
            height: u32at(b, 4),
            epoch: u64at(b, 8),
        };
        g.validate()?;
        Ok(g)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rectangle {
    pub x: u32,
    pub y: u32,
    pub width: u16,
    pub height: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Blit {
    pub epoch: u64,
    pub rectangle: Rectangle,
}
impl Blit {
    pub fn byte_len(self) -> Result<usize, Invalid> {
        let r = self.rectangle;
        if r.width == 0
            || r.height == 0
            || r.width > TILE_WIDTH
            || r.height > TILE_HEIGHT
            || self.epoch == 0
        {
            return Err(Invalid);
        }
        usize::from(r.width)
            .checked_mul(usize::from(r.height))
            .and_then(|n| n.checked_mul(4))
            .ok_or(Invalid)
    }
    pub fn validate(self, g: Geometry, bytes: usize) -> Result<(), Invalid> {
        g.validate()?;
        let r = self.rectangle;
        if self.epoch != g.epoch
            || self.byte_len()? != bytes
            || r.x
                .checked_add(u32::from(r.width))
                .is_none_or(|end| end > g.width)
            || r.y
                .checked_add(u32::from(r.height))
                .is_none_or(|end| end > g.height)
        {
            Err(Invalid)
        } else {
            Ok(())
        }
    }
    pub fn encode(self) -> Result<[u8; SIZE], Invalid> {
        let bytes = self.byte_len()?;
        let r = self.rectangle;
        let mut b = [0; SIZE];
        b[0..8].copy_from_slice(&self.epoch.to_le_bytes());
        b[8..12].copy_from_slice(&r.x.to_le_bytes());
        b[12..16].copy_from_slice(&r.y.to_le_bytes());
        b[16..18].copy_from_slice(&r.width.to_le_bytes());
        b[18..20].copy_from_slice(&r.height.to_le_bytes());
        b[24..28].copy_from_slice(&(bytes as u32).to_le_bytes());
        Ok(b)
    }
    pub fn decode(b: &[u8; SIZE]) -> Result<Self, Invalid> {
        if b[20..24].iter().chain(b[28..].iter()).any(|v| *v != 0) {
            return Err(Invalid);
        }
        let value = Self {
            epoch: u64at(b, 0),
            rectangle: Rectangle {
                x: u32at(b, 8),
                y: u32at(b, 12),
                width: u16at(b, 16),
                height: u16at(b, 18),
            },
        };
        if value.byte_len()? != u32at(b, 24) as usize {
            return Err(Invalid);
        }
        Ok(value)
    }
}
pub fn validate_pixels(bytes: &[u8]) -> Result<(), Invalid> {
    if bytes.is_empty()
        || bytes.len() > MAX_BYTES
        || !bytes.len().is_multiple_of(4)
        || bytes.chunks_exact(4).any(|p| p[3] != 0)
    {
        Err(Invalid)
    } else {
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reset {
    pub ticket: u64,
    pub epoch: u64,
    pub discarded: u32,
    pub operations: u32,
}
impl Reset {
    pub fn encode(self) -> Result<[u8; SIZE], Invalid> {
        if self.ticket == 0
            || self.epoch == 0
            || self.operations == 0
            || self.operations > INPUT_OPERATIONS
            || self.discarded > self.operations
        {
            return Err(Invalid);
        }
        let mut b = [0; SIZE];
        b[0..8].copy_from_slice(&self.ticket.to_le_bytes());
        b[8..16].copy_from_slice(&self.epoch.to_le_bytes());
        b[16..20].copy_from_slice(&self.discarded.to_le_bytes());
        b[20..24].copy_from_slice(&self.operations.to_le_bytes());
        b[24..28].copy_from_slice(&INPUT_PROFILE.to_le_bytes());
        Ok(b)
    }
    pub fn decode(b: &[u8; SIZE]) -> Result<Self, Invalid> {
        if u32at(b, 24) != INPUT_PROFILE || u32at(b, 28) != 0 {
            return Err(Invalid);
        }
        let r = Self {
            ticket: u64at(b, 0),
            epoch: u64at(b, 8),
            discarded: u32at(b, 16),
            operations: u32at(b, 20),
        };
        r.encode()?;
        Ok(r)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sample {
    pub epoch: u64,
    pub sequence: u64,
    pub status: u8,
    pub byte: u8,
}
impl Sample {
    pub const fn needs_reset(self) -> bool {
        self.status & 0xc0 != 0
    }
    pub const fn is_mouse(self) -> bool {
        self.status & 0x20 != 0
    }
    pub fn encode(self) -> Result<[u8; SIZE], Invalid> {
        if self.epoch == 0 || self.sequence == 0 || self.status & 1 == 0 {
            return Err(Invalid);
        }
        let mut b = [0; SIZE];
        b[0..8].copy_from_slice(&self.epoch.to_le_bytes());
        b[8..16].copy_from_slice(&self.sequence.to_le_bytes());
        b[16] = self.status;
        b[17] = self.byte;
        Ok(b)
    }
    pub fn decode(b: &[u8; SIZE]) -> Result<Self, Invalid> {
        if b[18..].iter().any(|v| *v != 0) {
            return Err(Invalid);
        }
        let sample = Self {
            epoch: u64at(b, 0),
            sequence: u64at(b, 8),
            status: b[16],
            byte: b[17],
        };
        sample.encode()?;
        Ok(sample)
    }
}

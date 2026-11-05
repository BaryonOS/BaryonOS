//! Native Materiaal front-end building blocks, without device access.
//!
//! The adapter authenticates input and owns display capabilities. This crate
//! owns document interaction and produces bounded private-RAM tiles. Host
//! checks do not demonstrate a native compositor or complete product.
#![no_std]
#![forbid(unsafe_code)]

pub mod controller;
pub mod design;
pub mod desktop;
pub mod render;
pub mod session;
pub mod text;
pub mod workspace;

pub const TILE_WIDTH: u32 = 32;
pub const TILE_HEIGHT: u32 = 8;
pub const TILE_PIXELS: usize = (TILE_WIDTH * TILE_HEIGHT) as usize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Scale {
    Percent100,
    Percent200,
}
impl Scale {
    pub const fn factor(self) -> u32 {
        match self {
            Self::Percent100 => 1,
            Self::Percent200 => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeometryError {
    Invalid,
    TooSmall,
}

/// Public display geometry, never a physical address or hardware stride.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Viewport {
    width: u32,
    height: u32,
    scale: Scale,
    epoch: u64,
}
impl Viewport {
    pub fn new(width: u32, height: u32, scale: Scale, epoch: u64) -> Result<Self, GeometryError> {
        if epoch == 0 || width > ui_core::MAX_WIDTH || height > ui_core::MAX_HEIGHT {
            return Err(GeometryError::Invalid);
        }
        if width / scale.factor() < 320 || height / scale.factor() < 240 {
            return Err(GeometryError::TooSmall);
        }
        Ok(Self {
            width,
            height,
            scale,
            epoch,
        })
    }
    pub const fn width(self) -> u32 {
        self.width
    }
    pub const fn height(self) -> u32 {
        self.height
    }
    pub const fn scale(self) -> Scale {
        self.scale
    }
    pub const fn epoch(self) -> u64 {
        self.epoch
    }
    pub const fn logical_width(self) -> u32 {
        self.width / self.scale.factor()
    }
    pub const fn logical_height(self) -> u32 {
        self.height / self.scale.factor()
    }
    pub const fn bounds(self) -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: self.width,
            height: self.height,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
impl Rect {
    pub fn contains(self, x: u32, y: u32) -> bool {
        u64::from(x) >= u64::from(self.x)
            && u64::from(y) >= u64::from(self.y)
            && u64::from(x) < u64::from(self.x) + u64::from(self.width)
            && u64::from(y) < u64::from(self.y) + u64::from(self.height)
    }
    pub fn intersects(self, other: Self) -> bool {
        self.width != 0
            && self.height != 0
            && other.width != 0
            && other.height != 0
            && u64::from(self.x) < u64::from(other.x) + u64::from(other.width)
            && u64::from(other.x) < u64::from(self.x) + u64::from(self.width)
            && u64::from(self.y) < u64::from(other.y) + u64::from(other.height)
            && u64::from(other.y) < u64::from(self.y) + u64::from(self.height)
    }
    pub fn physical(self, scale: Scale) -> Self {
        let factor = scale.factor();
        Self {
            x: self.x.saturating_mul(factor),
            y: self.y.saturating_mul(factor),
            width: self.width.saturating_mul(factor),
            height: self.height.saturating_mul(factor),
        }
    }
}

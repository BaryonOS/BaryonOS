//! Bounded scalar rendering into caller-owned pixels. No device, IPC, allocator,
//! floating point, runtime font parser or document/storage dependency.
#![no_std]
#![forbid(unsafe_code)]
pub mod damage;

#[cfg(not(feature = "inter-atlas"))]
mod atlas;
// Generated compact tables; the generator, not rustfmt, owns their layout.
#[cfg(feature = "inter-atlas")]
#[rustfmt::skip]
#[path = "atlas_inter.rs"]
mod atlas;
pub mod icon;
pub mod mark;
mod mark_geometry;
pub mod text;
pub mod theme;

pub use icon::Icon;
pub use text::{Font, Size, Weight};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
impl Rect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
    pub fn contains(self, x: i32, y: i32) -> bool {
        i64::from(x) >= i64::from(self.x)
            && i64::from(y) >= i64::from(self.y)
            && i64::from(x) < i64::from(self.x) + i64::from(self.width)
            && i64::from(y) < i64::from(self.y) + i64::from(self.height)
    }
    pub fn intersects(self, other: Self) -> bool {
        self.width != 0
            && self.height != 0
            && other.width != 0
            && other.height != 0
            && i64::from(self.x) < i64::from(other.x) + i64::from(other.width)
            && i64::from(other.x) < i64::from(self.x) + i64::from(self.width)
            && i64::from(self.y) < i64::from(other.y) + i64::from(other.height)
            && i64::from(other.y) < i64::from(self.y) + i64::from(self.height)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Geometry,
    Buffer,
}

pub struct Canvas<'a> {
    rect: Rect,
    pixels: &'a mut [u32],
}
impl<'a> Canvas<'a> {
    pub fn new(rect: Rect, pixels: &'a mut [u32]) -> Result<Self, Error> {
        if rect.width == 0
            || rect.height == 0
            || rect.width > 8192
            || rect.height > 8192
            || i64::from(rect.x) + i64::from(rect.width) > i64::from(i32::MAX)
            || i64::from(rect.y) + i64::from(rect.height) > i64::from(i32::MAX)
        {
            return Err(Error::Geometry);
        }
        let count = usize::try_from(u64::from(rect.width) * u64::from(rect.height))
            .map_err(|_| Error::Geometry)?;
        if pixels.len() < count {
            return Err(Error::Buffer);
        }
        Ok(Self {
            rect,
            pixels: &mut pixels[..count],
        })
    }
    pub const fn bounds(&self) -> Rect {
        self.rect
    }
    pub fn clear(&mut self, color: u32) {
        self.pixels.fill(color & 0x00ff_ffff);
    }
    pub fn pixel(&mut self, x: i32, y: i32, color: u32) {
        self.blend_pixel(x, y, color, 255);
    }
    pub fn blend_pixel(&mut self, x: i32, y: i32, color: u32, coverage: u8) {
        if coverage == 0 || !self.rect.contains(x, y) {
            return;
        }
        let index = ((i64::from(y) - i64::from(self.rect.y)) * i64::from(self.rect.width)
            + i64::from(x)
            - i64::from(self.rect.x)) as usize;
        self.pixels[index] = blend(color, self.pixels[index], coverage);
    }
    fn clipped(&self, r: Rect) -> Option<(i32, i32, i32, i32)> {
        if !self.rect.intersects(r) {
            return None;
        }
        let left = i64::from(r.x).max(i64::from(self.rect.x));
        let top = i64::from(r.y).max(i64::from(self.rect.y));
        let right = (i64::from(r.x) + i64::from(r.width))
            .min(i64::from(self.rect.x) + i64::from(self.rect.width));
        let bottom = (i64::from(r.y) + i64::from(r.height))
            .min(i64::from(self.rect.y) + i64::from(self.rect.height));
        Some((
            i32::try_from(left).ok()?,
            i32::try_from(top).ok()?,
            i32::try_from(right).ok()?,
            i32::try_from(bottom).ok()?,
        ))
    }
    pub fn fill_rect(&mut self, r: Rect, color: u32) {
        if let Some((l, t, right, bottom)) = self.clipped(r) {
            for y in t..bottom {
                for x in l..right {
                    self.pixel(x, y, color);
                }
            }
        }
    }
    /// Bounded distance falloff outside a surface; no backdrop read or blur.
    pub fn soft_shadow(
        &mut self,
        r: Rect,
        spread: u32,
        color: u32,
        opacity: u8,
    ) -> Result<(), Error> {
        if spread == 0 || spread > 64 || r.width == 0 || r.height == 0 {
            return Err(Error::Geometry);
        }
        let left = i64::from(r.x) - i64::from(spread);
        let top = i64::from(r.y) - i64::from(spread);
        let right = i64::from(r.x) + i64::from(r.width);
        let bottom = i64::from(r.y) + i64::from(r.height);
        let x0 = left.max(i64::from(self.rect.x));
        let y0 = top.max(i64::from(self.rect.y));
        let x1 =
            (right + i64::from(spread)).min(i64::from(self.rect.x) + i64::from(self.rect.width));
        let y1 =
            (bottom + i64::from(spread)).min(i64::from(self.rect.y) + i64::from(self.rect.height));
        let extent = i64::from(spread).pow(2);
        for y in y0..y1 {
            for x in x0..x1 {
                if r.contains(x as i32, y as i32) {
                    continue;
                }
                let dx = (i64::from(r.x) - x).max(x - right + 1).max(0);
                let dy = (i64::from(r.y) - y).max(y - bottom + 1).max(0);
                if dx >= i64::from(spread) || dy >= i64::from(spread) {
                    continue;
                }
                let distance = dx * dx + dy * dy;
                if distance < extent {
                    let alpha = (i64::from(opacity) * (extent - distance) / extent) as u8;
                    self.blend_pixel(x as i32, y as i32, color, alpha);
                }
            }
        }
        Ok(())
    }
    pub fn rounded_rect(&mut self, r: Rect, radius: u32, color: u32) {
        self.rounded(r, radius, 0, color, 255);
    }
    /// Frosted surface: `color` over what is already drawn, at `alpha`/255.
    pub fn translucent_rounded_rect(&mut self, r: Rect, radius: u32, color: u32, alpha: u8) {
        self.rounded(r, radius, 0, color, alpha);
    }
    pub fn stroke_rounded_rect(&mut self, r: Rect, radius: u32, stroke: u32, color: u32) {
        if stroke == 0 {
            return;
        }
        self.rounded(r, radius, stroke, color, 255);
    }
    fn rounded(&mut self, r: Rect, radius: u32, stroke: u32, color: u32, alpha: u8) {
        if r.width > 1_048_576 || r.height > 1_048_576 {
            return;
        }
        let Some((l, t, right, bottom)) = self.clipped(r) else {
            return;
        };
        let radius = radius.min(r.width / 2).min(r.height / 2);
        for y in t..bottom {
            for x in l..right {
                let mut coverage = 0;
                for sy in [1, 3] {
                    for sx in [1, 3] {
                        let px = i64::from(x) * 4 + sx;
                        let py = i64::from(y) * 4 + sy;
                        let outer = rounded_sample(r, radius, px, py);
                        let inner = if stroke > 0
                            && r.width > stroke.saturating_mul(2)
                            && r.height > stroke.saturating_mul(2)
                        {
                            let inset = stroke.min(i32::MAX as u32) as i32;
                            rounded_sample(
                                Rect::new(
                                    r.x.saturating_add(inset),
                                    r.y.saturating_add(inset),
                                    r.width - 2 * stroke,
                                    r.height - 2 * stroke,
                                ),
                                radius.saturating_sub(stroke),
                                px,
                                py,
                            )
                        } else {
                            false
                        };
                        if outer && (stroke == 0 || !inner) {
                            coverage += 1;
                        }
                    }
                }
                self.blend_pixel(x, y, color, (coverage * u32::from(alpha) / 4) as u8);
            }
        }
    }
    /// Integer antialiased segment. Geometry is clipped before pixel access.
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, thickness: u32, color: u32) {
        if thickness == 0
            || thickness > 128
            || (i64::from(x1) - i64::from(x0)).abs() > 65536
            || (i64::from(y1) - i64::from(y0)).abs() > 65536
        {
            return;
        }
        let pad = (thickness / 2 + 2) as i32;
        let left = x0.min(x1).saturating_sub(pad);
        let top = y0.min(y1).saturating_sub(pad);
        let right = x0.max(x1).saturating_add(pad);
        let bottom = y0.max(y1).saturating_add(pad);
        let width = (i64::from(right) - i64::from(left)).clamp(0, i64::from(u32::MAX)) as u32;
        let height = (i64::from(bottom) - i64::from(top)).clamp(0, i64::from(u32::MAX)) as u32;
        let Some((l, t, r, b)) = self.clipped(Rect::new(left, top, width, height)) else {
            return;
        };
        let ax = i128::from(x0) * 4 + 2;
        let ay = i128::from(y0) * 4 + 2;
        let dx = (i128::from(x1) - i128::from(x0)) * 4;
        let dy = (i128::from(y1) - i128::from(y0)) * 4;
        let length = dx * dx + dy * dy;
        let radius = i128::from(thickness) * 2;
        for y in t..b {
            for x in l..r {
                let mut coverage = 0;
                for sy in [1, 3] {
                    for sx in [1, 3] {
                        let px = i128::from(x) * 4 + sx - ax;
                        let py = i128::from(y) * 4 + sy - ay;
                        let dot = px * dx + py * dy;
                        let inside = if length == 0 || dot <= 0 {
                            px * px + py * py <= radius * radius
                        } else if dot >= length {
                            let bx = px - dx;
                            let by = py - dy;
                            bx * bx + by * by <= radius * radius
                        } else {
                            let cross = px * dy - py * dx;
                            cross * cross <= radius * radius * length
                        };
                        if inside {
                            coverage += 1;
                        }
                    }
                }
                self.blend_pixel(x, y, color, (coverage * 255 / 4) as u8);
            }
        }
    }
    pub fn ellipse(&mut self, r: Rect, color: u32) {
        if r.width > 1_048_576 || r.height > 1_048_576 {
            return;
        }
        let Some((l, t, right, bottom)) = self.clipped(r) else {
            return;
        };
        let rx = i128::from(r.width) * 2;
        let ry = i128::from(r.height) * 2;
        let cx = i128::from(r.x) * 4 + rx;
        let cy = i128::from(r.y) * 4 + ry;
        for y in t..bottom {
            for x in l..right {
                let mut coverage = 0;
                for sy in [1, 3] {
                    for sx in [1, 3] {
                        let dx = i128::from(x) * 4 + sx - cx;
                        let dy = i128::from(y) * 4 + sy - cy;
                        if dx * dx * ry * ry + dy * dy * rx * rx <= rx * rx * ry * ry {
                            coverage += 1;
                        }
                    }
                }
                self.blend_pixel(x, y, color, (coverage * 255 / 4) as u8);
            }
        }
    }
    pub fn text(&mut self, x: i32, y: i32, value: &str, font: Font, color: u32) -> u32 {
        text::draw(self, x, y, value, font, color)
    }
    pub fn icon(&mut self, r: Rect, icon: Icon, color: u32, stroke: u32) {
        icon::draw(self, r, icon, color, stroke);
    }
}
fn rounded_sample(r: Rect, radius: u32, x: i64, y: i64) -> bool {
    let left = i64::from(r.x) * 4;
    let top = i64::from(r.y) * 4;
    let right = left + i64::from(r.width) * 4;
    let bottom = top + i64::from(r.height) * 4;
    if x < left || y < top || x >= right || y >= bottom {
        return false;
    }
    let radius = i64::from(radius) * 4;
    if radius == 0 {
        return true;
    }
    let cx = x.clamp(left + radius, right - radius);
    let cy = y.clamp(top + radius, bottom - radius);
    let dx = x - cx;
    let dy = y - cy;
    dx * dx + dy * dy <= radius * radius
}
pub fn blend(front: u32, back: u32, alpha: u8) -> u32 {
    let a = u32::from(alpha);
    let mut result = 0;
    for shift in [0, 8, 16] {
        let c = (((front >> shift) & 255) * a + ((back >> shift) & 255) * (255 - a) + 127) / 255;
        result |= c << shift;
    }
    result
}
#[derive(Clone, Copy)]
struct FontRecord {
    first: u16,
    size: u8,
    height: u8,
}
#[derive(Clone, Copy)]
struct Glyph {
    offset: u32,
    rows: u32,
    left: i16,
    top: i16,
    width: u8,
    height: u8,
    advance: u16,
}

#![no_std]
#![forbid(unsafe_code)]
//! Bounded software raster and Set-1 input for the graphical bootstrap.
//! No hardware access, allocation, app authority or persistence lives here.

mod font;
mod input;
pub use input::{EditResult, KeyCode, KeyEvent, Keyboard, Modifiers, TextLine};

pub const MAX_WIDTH: u32 = 1920;
pub const MAX_HEIGHT: u32 = 1080;
pub const MAX_STRIDE: u32 = 4096;
pub const MAX_ROW_BYTES: usize = MAX_WIDTH as usize * 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { Geometry, Buffer, Row }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat { Rgb, Bgr }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color { pub red: u8, pub green: u8, pub blue: u8 }
impl Color {
    pub const fn new(red: u8, green: u8, blue: u8) -> Self { Self { red, green, blue } }
    pub const fn encode(self, format: PixelFormat) -> [u8; 4] {
        match format {
            PixelFormat::Rgb => [self.red, self.green, self.blue, 0],
            PixelFormat::Bgr => [self.blue, self.green, self.red, 0],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Geometry { width: u32, height: u32, stride: u32, bytes: usize, format: PixelFormat }
impl Geometry {
    pub fn new(width: u32, height: u32, stride: u32, buffer_len: usize, format: PixelFormat) -> Result<Self, Error> {
        if width == 0 || height == 0 || width > MAX_WIDTH || height > MAX_HEIGHT || stride < width || stride > MAX_STRIDE {
            return Err(Error::Geometry);
        }
        let bytes = (stride as usize).checked_mul(height as usize).and_then(|n| n.checked_mul(4)).ok_or(Error::Geometry)?;
        if buffer_len < bytes { return Err(Error::Buffer); }
        Ok(Self { width, height, stride, bytes, format })
    }
    pub const fn width(self) -> u32 { self.width }
    pub const fn height(self) -> u32 { self.height }
    pub const fn stride(self) -> u32 { self.stride }
    pub const fn required_bytes(self) -> usize { self.bytes }
    pub const fn format(self) -> PixelFormat { self.format }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect { pub x: i32, pub y: i32, pub width: u32, pub height: u32 }

/// Only initialized private RAM slices belong here, never a MMIO/raw framebuffer.
pub struct Canvas<'a> { bytes: &'a mut [u8], geometry: Geometry }
impl<'a> Canvas<'a> {
    pub fn new(bytes: &'a mut [u8], geometry: Geometry) -> Result<Self, Error> {
        if bytes.len() < geometry.bytes { return Err(Error::Buffer); }
        Ok(Self { bytes: &mut bytes[..geometry.bytes], geometry })
    }
    pub const fn geometry(&self) -> Geometry { self.geometry }
    pub fn pixel(&mut self, x: i32, y: i32, color: Color) -> bool {
        if x < 0 || y < 0 || x as u32 >= self.geometry.width || y as u32 >= self.geometry.height { return false; }
        let at = ((y as usize * self.geometry.stride as usize) + x as usize) * 4;
        self.bytes[at..at+4].copy_from_slice(&color.encode(self.geometry.format));
        true
    }
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        // i64 bounds make negative origins and u32::MAX extents safe to clip.
        let left = i64::from(rect.x).max(0).min(i64::from(self.geometry.width));
        let top = i64::from(rect.y).max(0).min(i64::from(self.geometry.height));
        let right = (i64::from(rect.x)+i64::from(rect.width)).max(0).min(i64::from(self.geometry.width));
        let bottom = (i64::from(rect.y)+i64::from(rect.height)).max(0).min(i64::from(self.geometry.height));
        for y in top..bottom { for x in left..right { self.pixel(x as i32, y as i32, color); } }
    }
    pub fn text(&mut self, x: i32, y: i32, text: &[u8], color: Color, scale: u8) {
        if !(1..=4).contains(&scale) { return; }
        for (index, byte) in text.iter().copied().enumerate() {
            let Some(offset) = index.checked_mul(6*usize::from(scale)).and_then(|n| i64::try_from(n).ok()) else { break; };
            let origin = i64::from(x)+offset;
            if origin > i64::from(i32::MAX) { break; }
            let glyph = font::glyph(byte);
            for (row, bits) in glyph.iter().copied().enumerate() {
                for col in 0..5 { if bits & (1 << (4-col)) != 0 {
                    let px = origin+(col*usize::from(scale)) as i64;
                    let py = i64::from(y)+(row*usize::from(scale)) as i64;
                    if let (Ok(px), Ok(py)) = (i32::try_from(px), i32::try_from(py)) {
                        self.fill_rect(Rect { x: px, y: py, width: u32::from(scale), height: u32::from(scale) }, color);
                    }
                } }
            }
        }
    }
}

pub const BACKGROUND: Color = Color::new(0xf4,0xf5,0xf1);
pub const SURFACE: Color = Color::new(0xff,0xff,0xff);
pub const TEXT: Color = Color::new(0x16,0x2b,0x2b);
pub const MUTED: Color = Color::new(0x5d,0x6a,0x67);
pub const ACCENT: Color = Color::new(0x0a,0x5e,0x55);

fn label_pixel(text: &[u8], x: u32, y: u32, origin_x: u32, origin_y: u32, scale: u32) -> bool {
    let (Some(dx),Some(dy)) = (x.checked_sub(origin_x),y.checked_sub(origin_y)) else { return false; };
    if dy >= 7*scale { return false; }
    let cell = dx/(6*scale); let column = (dx%(6*scale))/scale;
    let Some(byte) = text.get(cell as usize).copied() else { return false; };
    column < 5 && font::glyph(byte)[(dy/scale) as usize] & (1 << (4-column)) != 0
}

/// One bounded row of the diagnostic scene; no framebuffer pointer is accepted.
/// Bytes after the visible width (including stride padding) remain untouched.
pub fn render_debug_row<const N: usize>(geometry: Geometry, line: &TextLine<N>, y: u32, row: &mut [u8]) -> Result<(), Error> {
    if y >= geometry.height { return Err(Error::Row); }
    let length = geometry.width as usize*4;
    if row.len() < length { return Err(Error::Buffer); }
    for x in 0..geometry.width {
        let mut color = BACKGROUND;
        if x < 12 { color = ACCENT; }
        if (32..geometry.width.saturating_sub(32)).contains(&x) && (140..244).contains(&y) { color = SURFACE; }
        if label_pixel(b"MATERIAAL",x,y,40,36,4) { color = TEXT; }
        if label_pixel(b"GRAFISCHE BASIS - GEEN DESKTOP",x,y,40,88,2) { color = MUTED; }
        if label_pixel(b"TYPE EEN REGEL",x,y,48,156,2) { color = MUTED; }
        if label_pixel(line.as_bytes(),x,y,48,194,2) { color = TEXT; }
        let cursor = 48_u64 + line.len() as u64*12;
        if u64::from(x) >= cursor && u64::from(x) < cursor+2 && (192..212).contains(&y) { color = ACCENT; }
        if label_pixel(b"ALLEEN RAM - NIET OPGESLAGEN",x,y,40,280,2) { color = MUTED; }
        if label_pixel(b"ESC SLUIT DE PROEF",x,y,40,316,2) { color = MUTED; }
        let at=x as usize*4;row[at..at+4].copy_from_slice(&color.encode(geometry.format));
    }
    Ok(())
}
pub fn draw_material_debug<const N: usize>(canvas: &mut Canvas<'_>, line: &TextLine<N>) {
    let geometry=canvas.geometry;
    for y in 0..geometry.height {
        let start=y as usize*geometry.stride as usize*4;
        render_debug_row(geometry,line,y,&mut canvas.bytes[start..start+geometry.width as usize*4]).unwrap();
    }
}

#[cfg(test)] mod tests;

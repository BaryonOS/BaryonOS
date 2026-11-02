//! Fixed trusted border only. The app's assigned pixels are never chrome.
use crate::{control::State, design as d};
use nieuw_os_display_client::wire::{Geometry as NativeGeometry, Rectangle as Tile};
use nieuw_os_surface_protocol::{Geometry, Rectangle};
use noto_sans_mono_bitmap::{get_raster, FontWeight, RasterHeight};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invalid;
#[derive(Clone, Copy)]
pub struct Chrome {
    width: u32,
    height: u32,
    assigned: Rectangle,
}
impl Chrome {
    pub fn new(native: NativeGeometry) -> Result<Self, Invalid> {
        native.validate().map_err(|_| Invalid)?;
        let width = native
            .width
            .checked_sub(32)
            .filter(|x| *x >= 320)
            .ok_or(Invalid)?;
        let height = native
            .height
            .checked_sub(64)
            .filter(|x| *x >= 240)
            .ok_or(Invalid)?;
        Ok(Self {
            width: native.width,
            height: native.height,
            assigned: Rectangle {
                x: 16,
                y: 48,
                width: u16::try_from(width).map_err(|_| Invalid)?,
                height: u16::try_from(height).map_err(|_| Invalid)?,
            },
        })
    }
    pub fn geometry(self, epoch: u64) -> Result<Geometry, Invalid> {
        let g = Geometry {
            width: self.width,
            height: self.height,
            assigned: self.assigned,
            epoch,
        };
        g.validate().map_err(|_| Invalid)?;
        Ok(g)
    }
    pub fn tiles(self) -> Tiles {
        Tiles {
            chrome: self,
            strip: 0,
            app: false,
            x: 0,
            y: 0,
        }
    }
    pub fn wipe_tiles(self) -> Tiles {
        Tiles {
            chrome: self,
            strip: 0,
            app: true,
            x: 0,
            y: 0,
        }
    }
    pub fn render_wipe(self, tile: Tile, out: &mut [u8; 1024]) -> Result<usize, Invalid> {
        let r = self.assigned;
        if tile.width == 0
            || tile.width > 32
            || tile.height == 0
            || tile.height > 8
            || tile.x < r.x
            || tile.y < r.y
            || u64::from(tile.x) + u64::from(tile.width) > u64::from(r.x) + u64::from(r.width)
            || u64::from(tile.y) + u64::from(tile.height) > u64::from(r.y) + u64::from(r.height)
        {
            return Err(Invalid);
        }
        let len = usize::from(tile.width) * usize::from(tile.height) * 4;
        for p in out[..len].chunks_exact_mut(4) {
            p.copy_from_slice(&d::BACKGROUND.to_le_bytes());
        }
        Ok(len)
    }
    fn strip(self, index: u8) -> Option<(u32, u32, u32, u32)> {
        match index {
            0 => Some((0, 0, self.width, 48)),
            1 => Some((0, self.height - 16, self.width, 16)),
            2 => Some((0, 48, 16, self.height - 64)),
            3 => Some((self.width - 16, 48, 16, self.height - 64)),
            _ => None,
        }
    }
    /// Entire destination is validated before output mutation; unused bytes
    /// remain unchanged. Canonical B,G,R,0 source bytes, no framebuffer stride.
    pub fn render(self, tile: Tile, state: State, out: &mut [u8; 1024]) -> Result<usize, Invalid> {
        let endx = u64::from(tile.x) + u64::from(tile.width);
        let endy = u64::from(tile.y) + u64::from(tile.height);
        if tile.width == 0
            || tile.width > 32
            || tile.height == 0
            || tile.height > 8
            || endx > u64::from(self.width)
            || endy > u64::from(self.height)
            || !(0..4).any(|i| {
                self.strip(i).is_some_and(|(x, y, w, h)| {
                    tile.x >= x
                        && tile.y >= y
                        && endx <= u64::from(x + w)
                        && endy <= u64::from(y + h)
                })
            })
        {
            return Err(Invalid);
        }
        let text = match state {
            State::Inactive => "Invoer nog niet actief",
            State::Focused => "Invoer actief",
            State::Retired => "Werkruimte gesloten",
        };
        for y in 0..u32::from(tile.height) {
            for x in 0..u32::from(tile.width) {
                let gx = tile.x + x;
                let gy = tile.y + y;
                let mut color = if gy < 48 { d::SURFACE } else { d::BACKGROUND };
                if gy == 47
                    || gy == self.height - 16
                    || (gy >= 48 && gy < self.height - 16 && (gx == 15 || gx == self.width - 16))
                {
                    color = d::BORDER;
                }
                if gy < 48 {
                    color = glyph("Nieuw OS", 24, 6, gx, gy, color, d::TEXT);
                    color = glyph(
                        text,
                        24,
                        27,
                        gx,
                        gy,
                        color,
                        if state == State::Focused {
                            d::ACCENT
                        } else {
                            d::MUTED
                        },
                    );
                }
                let offset = (y as usize * usize::from(tile.width) + x as usize) * 4;
                out[offset..offset + 4].copy_from_slice(&color.to_le_bytes());
            }
        }
        Ok(usize::from(tile.width) * usize::from(tile.height) * 4)
    }
}
fn blend(background: u32, color: u32, alpha: u8) -> u32 {
    let a = u32::from(alpha);
    let mut out = 0;
    for shift in [0, 8, 16] {
        let c = ((background >> shift) & 255) * (255 - a) + ((color >> shift) & 255) * a;
        out |= (c / 255) << shift;
    }
    out
}
fn glyph(text: &str, x: u32, y: u32, px: u32, py: u32, background: u32, color: u32) -> u32 {
    if px < x || py < y || py >= y + 16 {
        return background;
    }
    let mut left = x;
    for ch in text.chars() {
        if let Some(raster) = get_raster(ch, FontWeight::Regular, RasterHeight::Size16) {
            let width = raster.width() as u32;
            if px >= left && px < left + width {
                return blend(
                    background,
                    color,
                    raster.raster()[(py - y) as usize][(px - left) as usize],
                );
            }
            left += width;
        }
        if left > px {
            break;
        }
    }
    background
}
pub struct Tiles {
    chrome: Chrome,
    strip: u8,
    app: bool,
    x: u32,
    y: u32,
}
impl Iterator for Tiles {
    type Item = Tile;
    fn next(&mut self) -> Option<Tile> {
        loop {
            let (x, y, width, height) = if self.app {
                if self.strip != 0 {
                    return None;
                }
                let r = self.chrome.assigned;
                (r.x, r.y, u32::from(r.width), u32::from(r.height))
            } else {
                self.chrome.strip(self.strip)?
            };
            if self.y >= height {
                self.strip += 1;
                self.x = 0;
                self.y = 0;
                continue;
            }
            let tile = Tile {
                x: x + self.x,
                y: y + self.y,
                width: 32.min(width - self.x) as u16,
                height: 8.min(height - self.y) as u16,
            };
            self.x += 32;
            if self.x >= width {
                self.x = 0;
                self.y += 8;
            }
            return Some(tile);
        }
    }
}

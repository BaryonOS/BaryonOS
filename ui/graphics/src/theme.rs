//! Shared calm, high-contrast system colors and an original procedural backdrop.
//! Consumers may pass different palettes; these do not confer trusted UI rights.
use crate::{blend, Canvas, Rect};
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub desktop_start: u32,
    pub desktop_end: u32,
    pub glow: u32,
    pub surface: u32,
    pub background: u32,
    pub text: u32,
    pub muted: u32,
    pub accent: u32,
    pub accent_soft: u32,
    pub border: u32,
}
pub const MATERIAAL: Palette = Palette {
    desktop_start: 0x18242f,
    desktop_end: 0x344850,
    glow: 0xc1b5a0,
    surface: 0xffffff,
    background: 0xf2f1ec,
    text: 0x20262d,
    muted: 0x5c6570,
    accent: 0x14655e,
    accent_soft: 0xe3ede8,
    border: 0xd9ddda,
};
pub fn wallpaper(canvas: &mut Canvas<'_>, screen: Rect, p: Palette) {
    if screen.width == 0 || screen.height == 0 {
        return;
    }
    let b = canvas.bounds();
    for dy in 0..b.height {
        for dx in 0..b.width {
            let x = b.x.saturating_add(dx as i32);
            let y = b.y.saturating_add(dy as i32);
            if !screen.contains(x, y) {
                continue;
            }
            let nx = ((i64::from(x) - i64::from(screen.x)) * 1024 / i64::from(screen.width)) as u32;
            let ny =
                ((i64::from(y) - i64::from(screen.y)) * 1024 / i64::from(screen.height)) as u32;
            let base = blend(
                p.desktop_end,
                p.desktop_start,
                ((nx * 75 + ny * 110) / 1024) as u8,
            );
            let dist = u64::from(nx.abs_diff(790)).pow(2) + u64::from(ny.abs_diff(160)).pow(2);
            let haze = (700_000u64.saturating_sub(dist) * 68 / 700_000) as u8;
            let curve = i64::from(ny) * 2 + i64::from(nx) * i64::from(nx) / 1024 - 1450;
            let ribbon = (180i64.saturating_sub(curve.abs()).max(0) * 25 / 180) as u8;
            canvas.pixel(x, y, blend(0xa6afa5, blend(p.glow, base, haze), ribbon));
        }
    }
}

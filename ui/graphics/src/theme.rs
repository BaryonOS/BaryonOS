//! Shared warm paper, ink and iris colors. The desktop wallpaper is an aurora.
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
pub const DIVIDER: u32 = 0xdeddd6;
pub const SHELL_TEXT: u32 = 0xf0eee8;
pub const SHELL_MUTED: u32 = 0xaaa9b2;
/// Soft radial light: full `max` at the centre, fading to 0 at `radius²`.
fn glow(nx: u32, ny: u32, cx: u32, cy: u32, radius2: u64, max: u64) -> u8 {
    let d = u64::from(nx.abs_diff(cx)).pow(2) + u64::from(ny.abs_diff(cy)).pow(2);
    (radius2.saturating_sub(d) * max / radius2) as u8
}
/// Aurora: deep diagonal base, iris and teal light, two thin luminous ribbons
/// and a gentle vignette. Integer-only so every tile renders identically.
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
            let mut c = blend(
                p.desktop_end,
                p.desktop_start,
                ((nx * 90 + ny * 130) / 1024) as u8,
            );
            c = blend(p.glow, c, glow(nx, ny, 860, 140, 420_000, 190));
            c = blend(0x1fa58f, c, glow(nx, ny, 120, 1000, 520_000, 150));
            c = blend(0x6b4fd8, c, glow(nx, ny, 560, 1180, 300_000, 90));
            // Ribbon centre follows a shallow arc across the screen.
            let arc = 300 + i64::from(nx.abs_diff(420)).pow(2) / 900 + i64::from(nx) / 5;
            let off = (i64::from(ny) - arc).abs();
            c = blend(
                0x7fe6cf,
                c,
                (60i64.saturating_sub(off).max(0) * 70 / 60) as u8,
            );
            let off = (i64::from(ny) - arc - 70).abs();
            c = blend(
                0xa59cff,
                c,
                (40i64.saturating_sub(off).max(0) * 34 / 40) as u8,
            );
            let edge = u64::from(nx.abs_diff(512)).pow(2) + u64::from(ny.abs_diff(470)).pow(2);
            c = blend(
                0x020a12,
                c,
                (edge.saturating_sub(180_000) / 4_000).min(110) as u8,
            );
            canvas.pixel(x, y, c);
        }
    }
}

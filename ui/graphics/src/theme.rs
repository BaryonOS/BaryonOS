//! Shared warm paper, ink and iris colors. The desktop wallpaper is a horizon.
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
/// Soft radial light: `max` at the centre, fading to 0 at `radius2` (squared).
fn glow(x: i64, y: i64, cx: i64, cy: i64, radius2: i64, max: i64) -> u8 {
    let d = (x - cx).pow(2) + (y - cy).pow(2);
    ((radius2 - d).max(0) * max / radius2) as u8
}
/// Horizon: a dark sky over the lit rim of a planet, with atmosphere light,
/// an iris haze and sparse stars. Integer-only, so every tile is identical.
pub fn wallpaper(canvas: &mut Canvas<'_>, screen: Rect, p: Palette) {
    if screen.width == 0 || screen.height == 0 {
        return;
    }
    let (w, h) = (i64::from(screen.width), i64::from(screen.height));
    let (cx, cy, radius) = (w / 2, h * 12 / 5, h * 7 / 4);
    let rim = (h / 360).max(1);
    let b = canvas.bounds();
    for dy in 0..b.height {
        for dx in 0..b.width {
            let x = b.x.saturating_add(dx as i32);
            let y = b.y.saturating_add(dy as i32);
            if !screen.contains(x, y) {
                continue;
            }
            let (px, py) = (
                i64::from(x) - i64::from(screen.x),
                i64::from(y) - i64::from(screen.y),
            );
            // Distance in 1/16 px keeps the thin rim line antialiased.
            let fine = ((px - cx).pow(2) * 256 + (py - cy).pow(2) * 256)
                .unsigned_abs()
                .isqrt() as i64
                - radius * 16;
            let d = fine / 16;
            let c = if d > 0 {
                // Sky: near-black above, the deep token blue towards the horizon.
                let mut c = blend(
                    p.desktop_start,
                    0x01040a,
                    (255 - (d * 255 / h).min(255)) as u8,
                );
                c = blend(p.glow, c, glow(px, py, w / 6, h * 3 / 5, (h * h) / 3, 120));
                c = blend(0x1fa58f, c, ((h / 3 - d).max(0) * 170 / (h / 3)) as u8);
                c = blend(0x7fe6cf, c, ((h / 40 - d).max(0) * 150 / (h / 40)) as u8);
                let hash = ((px * 73_856_093) ^ (py * 19_349_663)).rem_euclid(1_543);
                if hash == 0 && d > h / 4 {
                    c = blend(0xffffff, c, (90 + (px + py).rem_euclid(120)) as u8);
                }
                c
            } else {
                // Planet: almost black, faintly lit just under the rim.
                blend(
                    0x0d3036,
                    0x02060b,
                    ((h / 7 + d).max(0) * 200 / (h / 7)) as u8,
                )
            };
            let line = ((rim * 32 - fine.abs()).max(0) * 255 / (rim * 32)) as u8;
            canvas.pixel(x, y, blend(0xe6fff8, c, line));
        }
    }
}

//! Shared warm paper, ink and iris colors. The native desktop is flat graphite.
//! Consumers may pass different palettes; these do not confer trusted UI rights.
use crate::{Canvas, Rect};
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
    desktop_start: 0x17191f,
    desktop_end: 0x17191f,
    glow: 0x17191f,
    surface: 0xffffff,
    background: 0xf7f6f2,
    text: 0x202229,
    muted: 0x65656d,
    accent: 0x5449cf,
    accent_soft: 0xeeecfb,
    border: 0x8b8b86,
};
pub const DIVIDER: u32 = 0xdeddd6;
pub const SHELL_TEXT: u32 = 0xf0eee8;
pub const SHELL_MUTED: u32 = 0xaaa9b2;
pub fn wallpaper(canvas: &mut Canvas<'_>, screen: Rect, p: Palette) {
    canvas.fill_rect(screen, p.desktop_start);
}

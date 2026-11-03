//! Shared semantic roles, parsed at compile time from the product token source.
//! Colors and effects never confer compositor authority.
use crate::{Canvas, Rect};
pub const TOKEN_SOURCE: &str = include_str!("../../prototype/tokens.css");

const fn value_start(name: &[u8]) -> usize {
    let source = TOKEN_SOURCE.as_bytes();
    let mut at = 0;
    while at + name.len() < source.len() {
        let mut index = 0;
        while index < name.len() && source[at + index] == name[index] {
            index += 1;
        }
        if index == name.len() && source[at + index] == b':' {
            at += index + 1;
            while at < source.len() && (source[at] == b' ' || source[at] == b'\n') {
                at += 1;
            }
            return at;
        }
        at += 1;
    }
    panic!("Missing product token")
}
const fn digit(byte: u8) -> u32 {
    match byte {
        b'0'..=b'9' => (byte - b'0') as u32,
        b'A'..=b'F' => (byte - b'A' + 10) as u32,
        b'a'..=b'f' => (byte - b'a' + 10) as u32,
        _ => panic!("Invalid color token"),
    }
}
pub const fn color(name: &[u8]) -> u32 {
    let source = TOKEN_SOURCE.as_bytes();
    let at = value_start(name);
    assert!(source[at] == b'#');
    let mut result = 0;
    let mut i = 1;
    while i <= 6 {
        result = result * 16 + digit(source[at + i]);
        i += 1;
    }
    assert!(source[at + 7] == b';');
    result
}
pub const fn pixels(name: &[u8]) -> u32 {
    let source = TOKEN_SOURCE.as_bytes();
    let mut at = value_start(name);
    let mut result = 0;
    while source[at] >= b'0' && source[at] <= b'9' {
        result = result * 10 + (source[at] - b'0') as u32;
        at += 1;
    }
    assert!(source[at] == b'p' && source[at + 1] == b'x' && source[at + 2] == b';');
    result
}
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
    desktop_start: color(b"--desktop-start"),
    desktop_end: color(b"--desktop-end"),
    glow: color(b"--desktop-glow"),
    surface: color(b"--surface"),
    background: color(b"--background"),
    text: color(b"--text"),
    muted: color(b"--muted"),
    accent: color(b"--accent"),
    accent_soft: color(b"--accent-soft"),
    border: color(b"--border"),
};
pub const HIGH_CONTRAST: Palette = Palette {
    desktop_start: 0,
    desktop_end: 0,
    glow: 0,
    surface: 0,
    background: 0,
    text: 0xffffff,
    muted: 0xb8b8b8,
    accent: 0xffff00,
    accent_soft: 0x00ffff,
    border: 0xffffff,
};
pub const HIGH_CONTRAST_DANGER: u32 = 0xffb4b4;
pub const DIVIDER: u32 = color(b"--line");
pub const SHELL_TEXT: u32 = color(b"--desktop-text");
pub const SHELL_MUTED: u32 = color(b"--shell-muted");
pub const SHELL_ACTIVE: u32 = color(b"--shell-active");
pub const SHELL_LINE: u32 = color(b"--shell-line");
pub const SHADOW: u32 = color(b"--shadow");
pub const DANGER: u32 = color(b"--danger");
pub const WARNING: u32 = color(b"--warning");
pub const WARNING_BACKGROUND: u32 = color(b"--warning-bg");
pub fn wallpaper(canvas: &mut Canvas<'_>, screen: Rect, p: Palette) {
    let bounds = canvas.bounds();
    if !bounds.intersects(screen) {
        return;
    }
    let top = bounds.y.max(screen.y);
    let bottom = (i64::from(bounds.y) + i64::from(bounds.height))
        .min(i64::from(screen.y) + i64::from(screen.height));
    for y in i64::from(top)..bottom {
        let progress =
            ((y - i64::from(screen.y)) * 255 / i64::from(screen.height.max(1))).clamp(0, 255) as u8;
        canvas.fill_rect(
            Rect::new(screen.x, y as i32, screen.width, 1),
            crate::blend(p.desktop_end, p.desktop_start, progress),
        );
    }
}

//! Compile-time bridge to the one existing Materiaal token source. A missing
//! or malformed token is a compilation error, never a different native theme.
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
    panic!("Missing Materiaal token")
}
const fn digit(byte: u8) -> u32 {
    match byte {
        b'0'..=b'9' => (byte - b'0') as u32,
        b'A'..=b'F' => (byte - b'A' + 10) as u32,
        b'a'..=b'f' => (byte - b'a' + 10) as u32,
        _ => panic!("Invalid color token"),
    }
}
const fn color(name: &[u8]) -> u32 {
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
const fn pixels(name: &[u8]) -> u32 {
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
pub const BACKGROUND: u32 = color(b"--background");
pub const SURFACE: u32 = color(b"--surface");
pub const TEXT: u32 = color(b"--text");
pub const MUTED: u32 = color(b"--muted");
pub const ACCENT: u32 = color(b"--accent");
pub const DANGER: u32 = color(b"--danger");
pub const WARNING: u32 = color(b"--warning");
pub const WARNING_BACKGROUND: u32 = color(b"--warning-bg");
pub const BORDER: u32 = color(b"--border");
pub const LINE: u32 = color(b"--line");
pub const ACCENT_SOFT: u32 = color(b"--accent-soft");
pub const SPACE: u32 = pixels(b"--space-1");
pub const PAD: u32 = pixels(b"--space-6");
pub const CONTROL_RADIUS: u32 = pixels(b"--radius-control");
pub const SURFACE_RADIUS: u32 = pixels(b"--radius-surface");

// These are the existing PRODUCT-SPEC typography/control contracts, not a
// separate color/token table. No animation is introduced in this slice.
pub const LINE_HEIGHT: u32 = 24;
pub const CONTROL_HEIGHT: u32 = 44;
pub const FOCUS_WIDTH: u32 = 3;
pub const FOCUS_OFFSET: u32 = 3;

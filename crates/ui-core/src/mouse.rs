//! Standard PS/2 device-ID-0 stream packets. Controller/command routing is external.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MouseButtons(u8);
impl MouseButtons {
    pub const fn left(self) -> bool {
        self.0 & 1 != 0
    }
    pub const fn right(self) -> bool {
        self.0 & 2 != 0
    }
    pub const fn middle(self) -> bool {
        self.0 & 4 != 0
    }
    pub const fn bits(self) -> u8 {
        self.0
    }
    pub(crate) const fn from_bits(bits: u8) -> Self {
        Self(bits & 7)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MouseEvent {
    /// Positive X is right; positive Y is up, as on the PS/2 wire.
    pub dx: i16,
    pub dy: i16,
    pub buttons: MouseButtons,
    pub changed: MouseButtons,
    /// An overflow axis has zero displacement; its lost movement is not guessed.
    pub x_overflow: bool,
    pub y_overflow: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseError {
    MissingHeader,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ps2Mouse {
    packet: [u8; 3],
    received: u8,
    buttons: MouseButtons,
}
impl Ps2Mouse {
    pub const fn new() -> Self {
        Self {
            packet: [0; 3],
            received: 0,
            buttons: MouseButtons(0),
        }
    }
    /// Known byte loss or command traffic requires an explicit transport reset.
    /// The caller also cancels client button state; no invented release is emitted.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    pub fn feed(&mut self, byte: u8) -> Result<Option<MouseEvent>, MouseError> {
        if self.received == 0 && byte & 8 == 0 {
            return Err(MouseError::MissingHeader);
        }
        self.packet[self.received as usize] = byte;
        self.received += 1;
        if self.received != 3 {
            return Ok(None);
        }
        self.received = 0;
        let [flags, x, y] = self.packet;
        let buttons = MouseButtons::from_bits(flags);
        let changed = MouseButtons::from_bits(buttons.0 ^ self.buttons.0);
        self.buttons = buttons;
        Ok(Some(MouseEvent {
            dx: if flags & 0x40 != 0 {
                0
            } else {
                i16::from(x) - if flags & 0x10 != 0 { 256 } else { 0 }
            },
            dy: if flags & 0x80 != 0 {
                0
            } else {
                i16::from(y) - if flags & 0x20 != 0 { 256 } else { 0 }
            },
            buttons,
            changed,
            x_overflow: flags & 0x40 != 0,
            y_overflow: flags & 0x80 != 0,
        }))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cursor {
    width: u32,
    height: u32,
    x: u32,
    y: u32,
}
impl Cursor {
    pub const fn new(width: u32, height: u32) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        Some(Self {
            width,
            height,
            x: 0,
            y: 0,
        })
    }
    pub const fn position(self) -> (u32, u32) {
        (self.x, self.y)
    }
    pub fn apply(&mut self, event: MouseEvent) -> (u32, u32) {
        self.x =
            (i64::from(self.x) + i64::from(event.dx)).clamp(0, i64::from(self.width - 1)) as u32;
        self.y =
            (i64::from(self.y) - i64::from(event.dy)).clamp(0, i64::from(self.height - 1)) as u32;
        self.position()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn packet(mouse: &mut Ps2Mouse, bytes: [u8; 3]) -> MouseEvent {
        assert_eq!(mouse.feed(bytes[0]), Ok(None));
        assert_eq!(mouse.feed(bytes[1]), Ok(None));
        mouse.feed(bytes[2]).unwrap().unwrap()
    }
    #[test]
    fn nine_bit_signed_extremes_are_not_truncated_to_i8() {
        let mut m = Ps2Mouse::new();
        for (flags, x, y, expected) in [
            (0x08, 255, 128, (255, 128)),
            (0x38, 0, 255, (-256, -1)),
            (0x18, 128, 0, (-128, 0)),
            (0x28, 0, 1, (0, -255)),
        ] {
            let e = packet(&mut m, [flags, x, y]);
            assert_eq!((e.dx, e.dy), expected);
        }
    }
    #[test]
    fn button_transitions_survive_overflow_without_guessed_movement() {
        let mut m = Ps2Mouse::new();
        let e = packet(&mut m, [0x4f, 255, 5]);
        assert_eq!(
            (e.dx, e.dy, e.buttons.bits(), e.changed.bits()),
            (0, 5, 7, 7)
        );
        assert!(e.x_overflow && !e.y_overflow);
        let e = packet(&mut m, [0x8b, 2, 255]);
        assert_eq!(
            (e.dx, e.dy, e.buttons.bits(), e.changed.bits()),
            (2, 0, 3, 4)
        );
        assert!(!e.x_overflow && e.y_overflow);
        assert_eq!(packet(&mut m, [0xc8, 255, 255]).changed.bits(), 3);
        assert_eq!(packet(&mut m, [0x08, 0, 0]).changed.bits(), 0);
    }
    #[test]
    fn split_packets_payload_header_bits_and_explicit_resynchronization() {
        let mut m = Ps2Mouse::new();
        for b in [0, 1, 7, 0x80] {
            assert_eq!(m.feed(b), Err(MouseError::MissingHeader));
        }
        assert_eq!(m.feed(0x08), Ok(None));
        assert_eq!(m.feed(0x08), Ok(None)); // payload bit 3 is not a new header
        assert_eq!(m.feed(0x08).unwrap().unwrap().dx, 8);
        m.feed(0x09).unwrap(); // interrupted before the rest of this packet
        m.reset();
        let e = packet(&mut m, [0x0a, 4, 5]);
        assert!(!e.buttons.left() && e.buttons.right() && !e.buttons.middle());
        assert_eq!((e.dx, e.dy, e.changed.bits()), (4, 5, 2));
    }
    #[test]
    fn cursor_clamps_and_inverts_y_without_unsigned_wrap() {
        assert!(Cursor::new(0, 1).is_none());
        assert!(Cursor::new(1, 0).is_none());
        let mut m = Ps2Mouse::new();
        let mut c = Cursor::new(10, 20).unwrap();
        assert_eq!(c.apply(packet(&mut m, [0x28, 255, 1])), (9, 19));
        assert_eq!(c.apply(packet(&mut m, [0x18, 0, 255])), (0, 0));
        let mut large = Cursor::new(u32::MAX, u32::MAX).unwrap();
        large.x = u32::MAX - 2;
        large.y = u32::MAX - 2;
        assert_eq!(
            large.apply(packet(&mut m, [0x28, 255, 1])),
            (u32::MAX - 1, u32::MAX - 1)
        );
    }
}

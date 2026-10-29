//! Physical Set-1 events. Layout, text, device commands and hardware I/O are separate.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PhysicalKey {
    Escape,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Digit0,
    Minus,
    Equal,
    Backspace,
    Tab,
    Q,
    W,
    E,
    R,
    T,
    Y,
    U,
    I,
    O,
    P,
    LeftBracket,
    RightBracket,
    Enter,
    LeftControl,
    A,
    S,
    D,
    F,
    G,
    H,
    J,
    K,
    L,
    Semicolon,
    Quote,
    Backquote,
    LeftShift,
    Backslash,
    Z,
    X,
    C,
    V,
    B,
    N,
    M,
    Comma,
    Period,
    Slash,
    RightShift,
    KeypadMultiply,
    LeftAlt,
    Space,
    CapsLock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    NumLock,
    ScrollLock,
    Keypad7,
    Keypad8,
    Keypad9,
    KeypadSubtract,
    Keypad4,
    Keypad5,
    Keypad6,
    KeypadAdd,
    Keypad1,
    Keypad2,
    Keypad3,
    Keypad0,
    KeypadDecimal,
    IsoBackslash,
    F11,
    F12,
    KeypadEnter,
    RightControl,
    KeypadDivide,
    PrintScreen,
    RightAlt,
    Home,
    Up,
    PageUp,
    Left,
    Right,
    End,
    Down,
    PageDown,
    Insert,
    Delete,
    LeftSuper,
    RightSuper,
    Pause,
    Break,
    SysRq,
    Menu,
}
pub(crate) const KEY_COUNT: usize = PhysicalKey::Menu as usize + 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyAction {
    Press,
    Repeat,
    Release,
    /// Pause has a make sequence and no break sequence; it never remains held.
    Pulse,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KeyboardModifiers {
    pub left_shift: bool,
    pub right_shift: bool,
    pub left_control: bool,
    pub right_control: bool,
    pub left_alt: bool,
    pub right_alt: bool,
    pub left_super: bool,
    pub right_super: bool,
    pub caps_lock: bool,
    pub num_lock: bool,
    pub scroll_lock: bool,
}
impl KeyboardModifiers {
    pub const fn shift(self) -> bool {
        self.left_shift || self.right_shift
    }
    pub const fn control(self) -> bool {
        self.left_control || self.right_control
    }
    pub const fn alt(self) -> bool {
        self.left_alt || self.right_alt
    }
    pub const fn super_key(self) -> bool {
        self.left_super || self.right_super
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyboardEvent {
    pub key: PhysicalKey,
    pub action: KeyAction,
    /// State after this event. Locks are device-wide; held modifiers can be
    /// filtered by InputRouter to the current focus owner.
    pub modifiers: KeyboardModifiers,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyboardError {
    InvalidByte,
    UnsupportedScan,
    MalformedPrefix,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Prefix {
    #[default]
    None,
    Extended,
    Pause(u8),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Set1Keyboard {
    prefix: Prefix,
    down: [bool; KEY_COUNT],
    caps: bool,
    num: bool,
    scroll: bool,
}
impl Default for Set1Keyboard {
    fn default() -> Self {
        Self::new()
    }
}
impl Set1Keyboard {
    pub const fn new() -> Self {
        Self {
            prefix: Prefix::None,
            down: [false; KEY_COUNT],
            caps: false,
            num: false,
            scroll: false,
        }
    }
    /// Call together with router reset and client input-cancel notification on
    /// byte loss/parity/timeout. No synthetic keypress or release is generated.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    pub fn modifiers(&self) -> KeyboardModifiers {
        use PhysicalKey::*;
        KeyboardModifiers {
            left_shift: self.down[LeftShift as usize],
            right_shift: self.down[RightShift as usize],
            left_control: self.down[LeftControl as usize],
            right_control: self.down[RightControl as usize],
            left_alt: self.down[LeftAlt as usize],
            right_alt: self.down[RightAlt as usize],
            left_super: self.down[LeftSuper as usize],
            right_super: self.down[RightSuper as usize],
            caps_lock: self.caps,
            num_lock: self.num,
            scroll_lock: self.scroll,
        }
    }
    fn error(&mut self, error: KeyboardError) -> Result<Option<KeyboardEvent>, KeyboardError> {
        self.reset();
        Err(error)
    }
    pub fn feed(&mut self, byte: u8) -> Result<Option<KeyboardEvent>, KeyboardError> {
        if let Prefix::Pause(index) = self.prefix {
            const TAIL: [u8; 5] = [0x1d, 0x45, 0xe1, 0x9d, 0xc5];
            if byte != TAIL[index as usize] {
                return self.error(KeyboardError::MalformedPrefix);
            }
            if index == 4 {
                self.prefix = Prefix::None;
                return Ok(Some(KeyboardEvent {
                    key: PhysicalKey::Pause,
                    action: KeyAction::Pulse,
                    modifiers: self.modifiers(),
                }));
            }
            self.prefix = Prefix::Pause(index + 1);
            return Ok(None);
        }
        if byte == 0xe0 || byte == 0xe1 {
            if self.prefix != Prefix::None {
                return self.error(KeyboardError::MalformedPrefix);
            }
            self.prefix = if byte == 0xe0 {
                Prefix::Extended
            } else {
                Prefix::Pause(0)
            };
            return Ok(None);
        }
        if byte == 0 || byte == 0xff {
            return self.error(KeyboardError::InvalidByte);
        }
        let extended = core::mem::replace(&mut self.prefix, Prefix::None) == Prefix::Extended;
        // Full PrintScreen sequences contain fake E0 Shift make/break bytes.
        // Their short forms under Shift/Ctrl use E0 37/B7 directly.
        if extended && matches!(byte, 0x2a | 0xaa) {
            return Ok(None);
        }
        let Some(key) = physical(extended, byte & 0x7f) else {
            return self.error(KeyboardError::UnsupportedScan);
        };
        let held = &mut self.down[key as usize];
        let action = if byte & 0x80 == 0 {
            let action = if *held {
                KeyAction::Repeat
            } else {
                KeyAction::Press
            };
            *held = true;
            action
        } else {
            // Lost/foreign releases never acquire focus or create an action.
            if !*held {
                return Ok(None);
            }
            *held = false;
            KeyAction::Release
        };
        if action == KeyAction::Press {
            match key {
                PhysicalKey::CapsLock => self.caps = !self.caps,
                PhysicalKey::NumLock => self.num = !self.num,
                PhysicalKey::ScrollLock => self.scroll = !self.scroll,
                _ => {}
            }
        }
        Ok(Some(KeyboardEvent {
            key,
            action,
            modifiers: self.modifiers(),
        }))
    }
}

fn physical(extended: bool, scan: u8) -> Option<PhysicalKey> {
    use PhysicalKey::*;
    Some(if extended {
        match scan {
            0x1c => KeypadEnter,
            0x1d => RightControl,
            0x35 => KeypadDivide,
            0x37 => PrintScreen,
            0x38 => RightAlt,
            0x46 => Break,
            0x47 => Home,
            0x48 => Up,
            0x49 => PageUp,
            0x4b => Left,
            0x4d => Right,
            0x4f => End,
            0x50 => Down,
            0x51 => PageDown,
            0x52 => Insert,
            0x53 => Delete,
            0x5b => LeftSuper,
            0x5c => RightSuper,
            0x5d => Menu,
            _ => return None,
        }
    } else {
        const FIRST: [PhysicalKey; 83] = [
            Escape,
            Digit1,
            Digit2,
            Digit3,
            Digit4,
            Digit5,
            Digit6,
            Digit7,
            Digit8,
            Digit9,
            Digit0,
            Minus,
            Equal,
            Backspace,
            Tab,
            Q,
            W,
            E,
            R,
            T,
            Y,
            U,
            I,
            O,
            P,
            LeftBracket,
            RightBracket,
            Enter,
            LeftControl,
            A,
            S,
            D,
            F,
            G,
            H,
            J,
            K,
            L,
            Semicolon,
            Quote,
            Backquote,
            LeftShift,
            Backslash,
            Z,
            X,
            C,
            V,
            B,
            N,
            M,
            Comma,
            Period,
            Slash,
            RightShift,
            KeypadMultiply,
            LeftAlt,
            Space,
            CapsLock,
            F1,
            F2,
            F3,
            F4,
            F5,
            F6,
            F7,
            F8,
            F9,
            F10,
            NumLock,
            ScrollLock,
            Keypad7,
            Keypad8,
            Keypad9,
            KeypadSubtract,
            Keypad4,
            Keypad5,
            Keypad6,
            KeypadAdd,
            Keypad1,
            Keypad2,
            Keypad3,
            Keypad0,
            KeypadDecimal,
        ];
        match scan {
            1..=0x53 => FIRST[scan as usize - 1],
            0x54 => SysRq,
            0x56 => IsoBackslash,
            0x57 => F11,
            0x58 => F12,
            _ => return None,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn make_repeat_break_and_distinct_extended_keys() {
        let mut k = Set1Keyboard::new();
        for (byte, action) in [
            (0x1e, KeyAction::Press),
            (0x1e, KeyAction::Repeat),
            (0x9e, KeyAction::Release),
        ] {
            let e = k.feed(byte).unwrap().unwrap();
            assert_eq!((e.key, e.action), (PhysicalKey::A, action));
        }
        assert_eq!(k.feed(0x9e), Ok(None));
        for (scan, normal, extended) in [
            (0x1c, PhysicalKey::Enter, PhysicalKey::KeypadEnter),
            (0x48, PhysicalKey::Keypad8, PhysicalKey::Up),
        ] {
            assert_eq!(k.feed(scan).unwrap().unwrap().key, normal);
            assert_eq!(k.feed(0xe0), Ok(None));
            assert_eq!(k.feed(scan).unwrap().unwrap().key, extended);
        }
    }
    #[test]
    fn both_modifier_sides_release_independently_and_locks_ignore_repeat() {
        let mut k = Set1Keyboard::new();
        for byte in [
            0x2a, 0x36, 0x1d, 0xe0, 0x1d, 0x38, 0xe0, 0x38, 0xe0, 0x5b, 0xe0, 0x5c,
        ] {
            k.feed(byte).unwrap();
        }
        assert!(
            k.modifiers().shift()
                && k.modifiers().control()
                && k.modifiers().alt()
                && k.modifiers().super_key()
        );
        for byte in [0xaa, 0x9d, 0xb8, 0xe0, 0xdb] {
            k.feed(byte).unwrap();
        }
        let m = k.modifiers();
        assert!(
            !m.left_shift
                && m.right_shift
                && !m.left_control
                && m.right_control
                && !m.left_alt
                && m.right_alt
                && !m.left_super
                && m.right_super
        );
        for byte in [0xb6, 0xe0, 0x9d, 0xe0, 0xb8, 0xe0, 0xdc] {
            k.feed(byte).unwrap();
        }
        assert_eq!(k.modifiers(), KeyboardModifiers::default());
        for scan in [0x3a, 0x45, 0x46] {
            k.feed(scan).unwrap();
            k.feed(scan).unwrap();
            k.feed(scan | 0x80).unwrap();
        }
        assert!(k.modifiers().caps_lock && k.modifiers().num_lock && k.modifiers().scroll_lock);
        for scan in [0x3a, 0x45, 0x46] {
            k.feed(scan).unwrap();
        }
        assert!(!k.modifiers().caps_lock && !k.modifiers().num_lock && !k.modifiers().scroll_lock);
    }
    #[test]
    fn printscreen_fake_shifts_and_pause_are_bounded() {
        let mut k = Set1Keyboard::new();
        let mut events = [None; 2];
        let mut n = 0;
        for b in [0xe0, 0x2a, 0xe0, 0x37, 0xe0, 0xb7, 0xe0, 0xaa] {
            if let Some(e) = k.feed(b).unwrap() {
                events[n] = Some(e);
                n += 1;
            }
        }
        assert_eq!(n, 2);
        assert_eq!(events[0].unwrap().key, PhysicalKey::PrintScreen);
        assert_eq!(events[1].unwrap().action, KeyAction::Release);
        assert_eq!(k.modifiers(), KeyboardModifiers::default());
        for b in [0xe1, 0x1d, 0x45, 0xe1, 0x9d] {
            assert_eq!(k.feed(b), Ok(None));
        }
        let event = k.feed(0xc5).unwrap().unwrap();
        assert_eq!(
            (event.key, event.action),
            (PhysicalKey::Pause, KeyAction::Pulse)
        );
        assert_eq!(k.modifiers(), KeyboardModifiers::default());
        assert_eq!(k.feed(0x1e).unwrap().unwrap().action, KeyAction::Press);
    }
    #[test]
    fn malformed_input_resets_pressed_state_instead_of_sticking_modifiers() {
        for bytes in [
            &[0xe0, 0xe0][..],
            &[0xe1, 0x1d, 0x44],
            &[0xe0, 0x7f],
            &[0xff],
            &[0x00],
            &[0xfa],
        ] {
            let mut k = Set1Keyboard::new();
            k.feed(0x2a).unwrap();
            let mut failed = false;
            for b in bytes {
                failed |= k.feed(*b).is_err();
            }
            assert!(failed);
            assert_eq!(k.modifiers(), KeyboardModifiers::default());
            assert_eq!(k.feed(0x1e).unwrap().unwrap().action, KeyAction::Press);
        }
    }
    #[test]
    fn all_standard_key_positions_emit_unique_physical_events_and_release() {
        let mut seen = [false; KEY_COUNT];
        for extended in [false, true] {
            for scan in 1..0x80 {
                let Some(key) = physical(extended, scan) else {
                    continue;
                };
                assert!(!seen[key as usize], "duplicate physical mapping");
                seen[key as usize] = true;
                let mut k = Set1Keyboard::new();
                if extended {
                    k.feed(0xe0).unwrap();
                }
                assert_eq!(k.feed(scan).unwrap().unwrap().key, key);
                if extended {
                    k.feed(0xe0).unwrap();
                }
                assert_eq!(
                    k.feed(scan | 0x80).unwrap().unwrap().action,
                    KeyAction::Release
                );
            }
        }
        assert_eq!(seen.iter().filter(|s| **s).count(), KEY_COUNT - 1); // Pause is a sequence.
    }
}

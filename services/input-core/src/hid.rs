//! Boot-HID reports to the existing UI event types. No device authority or I/O.
//! Uses HID 1.11 Appendix B and the USB HID Usage Tables Keyboard/Keypad page.
//! Only keys represented by the current PhysicalKey contract are emitted.

use nieuw_os_usb_core::boot::{self, KeyboardChanges, KeyboardState, MouseReport};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, MouseButtons, MouseEvent, PhysicalKey};

/// Six releases, eight modifier transitions and six presses per boot report.
pub struct Keys {
    events: [Option<KeyboardEvent>; 20],
    len: usize,
}
impl Keys {
    pub fn events(&self) -> impl Iterator<Item = KeyboardEvent> + '_ {
        self.events[..self.len].iter().flatten().copied()
    }
    fn push(&mut self, key: PhysicalKey, action: KeyAction, modifiers: KeyboardModifiers) {
        self.events[self.len] = Some(KeyboardEvent {
            key,
            action,
            modifiers,
        });
        self.len += 1;
    }
}

pub struct Keyboard {
    state: KeyboardState,
    modifiers: u8,
    locks: KeyboardModifiers,
}
impl Keyboard {
    pub const fn new() -> Self {
        Self {
            state: KeyboardState::new(),
            modifiers: 0,
            locks: KeyboardModifiers {
                left_shift: false,
                right_shift: false,
                left_control: false,
                right_control: false,
                left_alt: false,
                right_alt: false,
                left_super: false,
                right_super: false,
                caps_lock: false,
                num_lock: false,
                scroll_lock: false,
            },
        }
    }

    pub fn update(&mut self, bytes: &[u8]) -> Result<Keys, boot::Error> {
        let mut report: [u8; 8] = bytes.try_into().map_err(|_| boot::Error::Length)?;
        // US backslash and non-US hash occupy the same PhysicalKey position.
        // Normalize that alias before duplicate validation, never emit two
        // independent owners for one physical key. Unsupported usages stay inert.
        for usage in &mut report[2..] {
            if *usage == 0x32 {
                *usage = 0x31;
            }
        }
        let changes = self.state.update(&report)?;
        Ok(self.events(changes))
    }

    pub fn detach(&mut self) -> Keys {
        let changes = self.state.clear();
        self.events(changes)
    }

    fn modifiers(&self) -> KeyboardModifiers {
        KeyboardModifiers {
            left_control: self.modifiers & 1 != 0,
            left_shift: self.modifiers & 2 != 0,
            left_alt: self.modifiers & 4 != 0,
            left_super: self.modifiers & 8 != 0,
            right_control: self.modifiers & 16 != 0,
            right_shift: self.modifiers & 32 != 0,
            right_alt: self.modifiers & 64 != 0,
            right_super: self.modifiers & 128 != 0,
            ..self.locks
        }
    }

    fn events(&mut self, changes: KeyboardChanges) -> Keys {
        let mut keys = Keys {
            events: [None; 20],
            len: 0,
        };
        for usage in changes.released() {
            if let Some(key) = physical(*usage) {
                keys.push(key, KeyAction::Release, self.modifiers());
            }
        }
        for (index, key) in [
            PhysicalKey::LeftControl,
            PhysicalKey::LeftShift,
            PhysicalKey::LeftAlt,
            PhysicalKey::LeftSuper,
            PhysicalKey::RightControl,
            PhysicalKey::RightShift,
            PhysicalKey::RightAlt,
            PhysicalKey::RightSuper,
        ]
        .into_iter()
        .enumerate()
        {
            let bit = 1 << index;
            let action = if changes.modifiers_released & bit != 0 {
                self.modifiers &= !bit;
                Some(KeyAction::Release)
            } else if changes.modifiers_pressed & bit != 0 {
                self.modifiers |= bit;
                Some(KeyAction::Press)
            } else {
                None
            };
            if let Some(action) = action {
                keys.push(key, action, self.modifiers());
            }
        }
        for usage in changes.pressed() {
            if let Some(key) = physical(*usage) {
                match key {
                    PhysicalKey::CapsLock => self.locks.caps_lock ^= true,
                    PhysicalKey::NumLock => self.locks.num_lock ^= true,
                    PhysicalKey::ScrollLock => self.locks.scroll_lock ^= true,
                    _ => {}
                }
                keys.push(key, KeyAction::Press, self.modifiers());
            }
        }
        keys
    }
}

pub struct Mouse {
    buttons: u8,
}
impl Mouse {
    pub const fn new() -> Self {
        Self { buttons: 0 }
    }
    pub fn update(&mut self, bytes: &[u8]) -> Result<MouseEvent, boot::Error> {
        let report = MouseReport::parse(bytes)?;
        let changed = report.buttons ^ self.buttons;
        self.buttons = report.buttons;
        Ok(MouseEvent {
            dx: i16::from(report.dx),
            // USB screen Y grows down; the shared event contract grows up.
            dy: -i16::from(report.dy),
            buttons: MouseButtons::checked(report.buttons).unwrap(),
            changed: MouseButtons::checked(changed).unwrap(),
            x_overflow: false,
            y_overflow: false,
        })
    }
    pub fn detach(&mut self) -> MouseEvent {
        self.update(&[0; 3]).unwrap()
    }
}

fn physical(usage: u8) -> Option<PhysicalKey> {
    use PhysicalKey::*;
    const LETTERS: [PhysicalKey; 26] = [
        A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    ];
    const DIGITS: [PhysicalKey; 10] = [
        Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9, Digit0,
    ];
    const FUNCTIONS: [PhysicalKey; 12] = [F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12];
    Some(match usage {
        0x04..=0x1d => LETTERS[usize::from(usage - 4)],
        0x1e..=0x27 => DIGITS[usize::from(usage - 0x1e)],
        0x28 => Enter,
        0x29 => Escape,
        0x2a => Backspace,
        0x2b => Tab,
        0x2c => Space,
        0x2d => Minus,
        0x2e => Equal,
        0x2f => LeftBracket,
        0x30 => RightBracket,
        0x31 => Backslash,
        0x33 => Semicolon,
        0x34 => Quote,
        0x35 => Backquote,
        0x36 => Comma,
        0x37 => Period,
        0x38 => Slash,
        0x39 => CapsLock,
        0x3a..=0x45 => FUNCTIONS[usize::from(usage - 0x3a)],
        0x46 => PrintScreen,
        0x47 => ScrollLock,
        0x48 => Pause,
        0x49 => Insert,
        0x4a => Home,
        0x4b => PageUp,
        0x4c => Delete,
        0x4d => End,
        0x4e => PageDown,
        0x4f => Right,
        0x50 => Left,
        0x51 => Down,
        0x52 => Up,
        0x53 => NumLock,
        0x54 => KeypadDivide,
        0x55 => KeypadMultiply,
        0x56 => KeypadSubtract,
        0x57 => KeypadAdd,
        0x58 => KeypadEnter,
        0x59 => Keypad1,
        0x5a => Keypad2,
        0x5b => Keypad3,
        0x5c => Keypad4,
        0x5d => Keypad5,
        0x5e => Keypad6,
        0x5f => Keypad7,
        0x60 => Keypad8,
        0x61 => Keypad9,
        0x62 => Keypad0,
        0x63 => KeypadDecimal,
        0x64 => IsoBackslash,
        0x65 => Menu,
        _ => return None,
    })
}

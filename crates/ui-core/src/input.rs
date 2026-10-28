#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub caps_lock: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCode {
    Character(u8),
    Backspace,
    Enter,
    Escape,
    Tab,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    Delete,
    LeftShift,
    RightShift,
    LeftControl,
    RightControl,
    LeftAlt,
    RightAlt,
    CapsLock,
    PrintScreen,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub pressed: bool,
    pub modifiers: Modifiers,
    pub text: Option<u8>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Keyboard {
    extended: bool,
    pause_left: u8,
    shifts: [bool; 2],
    controls: [bool; 2],
    alts: [bool; 2],
    caps: bool,
    caps_down: bool,
}
impl Keyboard {
    pub const fn new() -> Self {
        Self {
            extended: false,
            pause_left: 0,
            shifts: [false; 2],
            controls: [false; 2],
            alts: [false; 2],
            caps: false,
            caps_down: false,
        }
    }
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    pub fn feed(&mut self, byte: u8) -> Option<KeyEvent> {
        if self.pause_left > 0 {
            self.pause_left -= 1;
            return None;
        }
        if byte == 0xe1 {
            self.extended = false;
            self.pause_left = 5;
            return None;
        }
        if byte == 0xe0 {
            self.extended = true;
            return None;
        }
        let extended = core::mem::replace(&mut self.extended, false);
        let pressed = byte & 0x80 == 0;
        let scan = byte & 0x7f;
        let code = match (extended, scan) {
            (false, 0x2a) => {
                self.shifts[0] = pressed;
                KeyCode::LeftShift
            }
            (false, 0x36) => {
                self.shifts[1] = pressed;
                KeyCode::RightShift
            }
            (false, 0x1d) => {
                self.controls[0] = pressed;
                KeyCode::LeftControl
            }
            (true, 0x1d) => {
                self.controls[1] = pressed;
                KeyCode::RightControl
            }
            (false, 0x38) => {
                self.alts[0] = pressed;
                KeyCode::LeftAlt
            }
            (true, 0x38) => {
                self.alts[1] = pressed;
                KeyCode::RightAlt
            }
            (false, 0x3a) => {
                if pressed && !self.caps_down {
                    self.caps = !self.caps;
                }
                self.caps_down = pressed;
                KeyCode::CapsLock
            }
            (true, 0x48) => KeyCode::Up,
            (true, 0x50) => KeyCode::Down,
            (true, 0x4b) => KeyCode::Left,
            (true, 0x4d) => KeyCode::Right,
            (true, 0x47) => KeyCode::Home,
            (true, 0x4f) => KeyCode::End,
            (true, 0x53) => KeyCode::Delete,
            (true, 0x37) => KeyCode::PrintScreen,
            (_, 0x1c) => KeyCode::Enter,
            (false, 0x01) => KeyCode::Escape,
            (false, 0x0e) => KeyCode::Backspace,
            (false, 0x0f) => KeyCode::Tab,
            (false, _) => KeyCode::Character(base(scan)?),
            _ => return None,
        };
        let modifiers = Modifiers {
            shift: self.shifts.contains(&true),
            control: self.controls.contains(&true),
            alt: self.alts.contains(&true),
            caps_lock: self.caps,
        };
        let text = if pressed && !modifiers.control && !modifiers.alt {
            if let KeyCode::Character(b) = code {
                Some(if b.is_ascii_lowercase() {
                    if modifiers.shift ^ modifiers.caps_lock {
                        b.to_ascii_uppercase()
                    } else {
                        b
                    }
                } else if modifiers.shift {
                    shifted(b)
                } else {
                    b
                })
            } else {
                None
            }
        } else {
            None
        };
        Some(KeyEvent {
            code,
            pressed,
            modifiers,
            text,
        })
    }
}
fn base(scan: u8) -> Option<u8> {
    Some(match scan {
        0x02..=0x0b => b"1234567890"[(scan - 2) as usize],
        0x0c => b'-',
        0x0d => b'=',
        0x10..=0x19 => b"qwertyuiop"[(scan - 0x10) as usize],
        0x1a => b'[',
        0x1b => b']',
        0x1e..=0x26 => b"asdfghjkl"[(scan - 0x1e) as usize],
        0x27 => b';',
        0x28 => b'\'',
        0x29 => b'`',
        0x2b => b'\\',
        0x2c..=0x32 => b"zxcvbnm"[(scan - 0x2c) as usize],
        0x33 => b',',
        0x34 => b'.',
        0x35 => b'/',
        0x39 => b' ',
        _ => return None,
    })
}
fn shifted(b: u8) -> u8 {
    match b {
        b'1' => b'!',
        b'2' => b'@',
        b'3' => b'#',
        b'4' => b'$',
        b'5' => b'%',
        b'6' => b'^',
        b'7' => b'&',
        b'8' => b'*',
        b'9' => b'(',
        b'0' => b')',
        b'-' => b'_',
        b'=' => b'+',
        b'[' => b'{',
        b']' => b'}',
        b';' => b':',
        b'\'' => b'"',
        b'`' => b'~',
        b'\\' => b'|',
        b',' => b'<',
        b'.' => b'>',
        b'/' => b'?',
        _ => b,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditResult {
    Unchanged,
    Changed,
    Full,
    Submit,
}
pub struct TextLine<const N: usize> {
    bytes: [u8; N],
    length: usize,
}
impl<const N: usize> Default for TextLine<N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<const N: usize> TextLine<N> {
    pub const fn new() -> Self {
        Self {
            bytes: [0; N],
            length: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.length
    }
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.length]
    }
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(self.as_bytes()).unwrap()
    }
    pub fn apply(&mut self, event: KeyEvent) -> EditResult {
        if !event.pressed || event.modifiers.control || event.modifiers.alt {
            return EditResult::Unchanged;
        }
        match event.code {
            KeyCode::Backspace if self.length > 0 => {
                self.length -= 1;
                self.bytes[self.length] = 0;
                EditResult::Changed
            }
            KeyCode::Enter => EditResult::Submit,
            KeyCode::Character(_) => {
                let Some(b) = event.text.filter(|b| (32..=126).contains(b)) else {
                    return EditResult::Unchanged;
                };
                if self.length == N {
                    return EditResult::Full;
                }
                self.bytes[self.length] = b;
                self.length += 1;
                EditResult::Changed
            }
            _ => EditResult::Unchanged,
        }
    }
}

#![forbid(unsafe_code)]

//! USB HID 1.11 Appendix B boot reports, not the general report protocol.
//!
//! The caller must have selected boot protocol on a verified HID interface and
//! pass exactly the completed transfer bytes. This module does not perform I/O,
//! identify a device, authorize input, or publish events to the OS.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Length,
    DuplicateKey(u8),
    DisplacementOutOfRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyboardStatus {
    Rollover,
    PostFail,
    Undefined,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyboardReport {
    modifiers: u8,
    keys: Result<[u8; 6], KeyboardStatus>,
}

impl KeyboardReport {
    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        let report: &[u8; 8] = bytes.try_into().map_err(|_| Error::Length)?;
        let keys = [
            report[2], report[3], report[4], report[5], report[6], report[7],
        ];
        let status = keys.iter().find_map(|key| match key {
            1 => Some(KeyboardStatus::Rollover),
            2 => Some(KeyboardStatus::PostFail),
            3 => Some(KeyboardStatus::Undefined),
            _ => None,
        });
        if status.is_none() {
            for (index, key) in keys.iter().enumerate() {
                if *key != 0 && keys[..index].contains(key) {
                    return Err(Error::DuplicateKey(*key));
                }
            }
        }
        // Byte 1 is OEM-reserved and ignored, not required to be zero (B.1).
        Ok(Self {
            modifiers: report[0],
            keys: status.map_or(Ok(keys), Err),
        })
    }

    pub const fn modifiers(self) -> u8 {
        self.modifiers
    }

    /// Usage IDs, not text or PS/2 scan codes. Zero denotes an empty slot.
    pub const fn keys(self) -> Result<[u8; 6], KeyboardStatus> {
        self.keys
    }
}

/// Changes in device-reported state; these values confer no input authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KeyboardChanges {
    pub modifiers_pressed: u8,
    pub modifiers_released: u8,
    pub status: Option<KeyboardStatus>,
    pressed: [u8; 6],
    pressed_len: usize,
    released: [u8; 6],
    released_len: usize,
}

impl KeyboardChanges {
    pub fn pressed(&self) -> &[u8] {
        &self.pressed[..self.pressed_len]
    }

    pub fn released(&self) -> &[u8] {
        &self.released[..self.released_len]
    }
}

/// One device/interface lifetime. No repeat timing, layout, or global state.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KeyboardState {
    modifiers: u8,
    keys: [u8; 6],
}

impl KeyboardState {
    pub const fn new() -> Self {
        Self {
            modifiers: 0,
            keys: [0; 6],
        }
    }

    /// Malformed reports leave state unchanged. A keyboard-reported error
    /// preserves the last known key set but still updates modifiers (App. C).
    pub fn update(&mut self, bytes: &[u8]) -> Result<KeyboardChanges, Error> {
        let report = KeyboardReport::parse(bytes)?;
        Ok(self.apply(report))
    }

    fn apply(&mut self, report: KeyboardReport) -> KeyboardChanges {
        let mut changes = KeyboardChanges {
            modifiers_pressed: report.modifiers & !self.modifiers,
            modifiers_released: self.modifiers & !report.modifiers,
            ..KeyboardChanges::default()
        };
        self.modifiers = report.modifiers;
        match report.keys {
            Err(status) => changes.status = Some(status),
            Ok(keys) => {
                for key in self.keys {
                    if key != 0 && !keys.contains(&key) {
                        changes.released[changes.released_len] = key;
                        changes.released_len += 1;
                    }
                }
                for key in keys {
                    if key != 0 && !self.keys.contains(&key) {
                        changes.pressed[changes.pressed_len] = key;
                        changes.pressed_len += 1;
                    }
                }
                self.keys = keys;
            }
        }
        changes
    }

    /// Releases the last known state, e.g. when its owning device detaches.
    /// Callers must still enforce their own device lifetime/epoch boundaries.
    pub fn clear(&mut self) -> KeyboardChanges {
        self.apply(KeyboardReport {
            modifiers: 0,
            keys: Ok([0; 6]),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MouseReport {
    /// Left, right, middle in bits 0..=2; OEM bits are not extra buttons.
    pub buttons: u8,
    pub dx: i8,
    pub dy: i8,
}

impl MouseReport {
    /// Boot reports are 3..=8 bytes; ignore OEM extensions, including wheels.
    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        if !(3..=8).contains(&bytes.len()) {
            return Err(Error::Length);
        }
        let dx = i8::from_ne_bytes([bytes[1]]);
        let dy = i8::from_ne_bytes([bytes[2]]);
        if dx == i8::MIN || dy == i8::MIN {
            return Err(Error::DisplacementOutOfRange);
        }
        Ok(Self {
            buttons: bytes[0] & 7,
            dx,
            dy,
        })
    }
}

/// Changes in one interface's boot-mouse state, without input authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MouseChanges {
    pub buttons_pressed: u8,
    pub buttons_released: u8,
    pub dx: i8,
    pub dy: i8,
}

/// Button state for one device/interface lifetime. Movement is not retained.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MouseState {
    buttons: u8,
}

impl MouseState {
    pub const fn new() -> Self {
        Self { buttons: 0 }
    }

    /// Malformed reports leave the prior button state unchanged.
    pub fn update(&mut self, bytes: &[u8]) -> Result<MouseChanges, Error> {
        let report = MouseReport::parse(bytes)?;
        let changes = MouseChanges {
            buttons_pressed: report.buttons & !self.buttons,
            buttons_released: self.buttons & !report.buttons,
            dx: report.dx,
            dy: report.dy,
        };
        self.buttons = report.buttons;
        Ok(changes)
    }

    /// Releases held buttons on detach or reset. The owner must reject stale
    /// completions and enforce device lifetime/epoch boundaries separately.
    pub fn clear(&mut self) -> MouseChanges {
        let changes = MouseChanges {
            buttons_released: self.buttons,
            ..MouseChanges::default()
        };
        self.buttons = 0;
        changes
    }
}

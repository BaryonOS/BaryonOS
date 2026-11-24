//! Optional firmware-facts extension of the BootInfo page (H06-T3, HW-INV).
//!
//! Only the `hw-inventory` loader writes it; every other loader leaves these
//! bytes zero, which never validates. The loader records the UEFI global
//! `SecureBoot` variable before ExitBootServices; the kernel reads it only in
//! the read-only `native-hw-inventory` profile. It is an observation for an
//! inventory record, never evidence of a verified boot chain.
use crate::PAGE_SIZE;

/// Byte offset within the BootInfo page, after the 64-byte ACPI extension.
pub const FACTS_OFFSET: u64 = 256;
pub const FACTS_SIZE: usize = 16;
pub const FACTS_MAGIC: [u8; 8] = *b"NOSFWF\0\0";
pub const FACTS_VERSION: u16 = 1;

const _: () = assert!(FACTS_OFFSET + FACTS_SIZE as u64 <= PAGE_SIZE);

/// The loader's observation of the UEFI `SecureBoot` variable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecureBoot {
    /// Present, one byte, value 0.
    Disabled,
    /// Present, one byte, value 1.
    Enabled,
    /// GetVariable reported EFI_NOT_FOUND: the state is unknown, not disabled.
    Absent,
    /// Any other status, size or value: not reported as a state.
    Unreadable,
}

impl SecureBoot {
    const fn code(self) -> u8 {
        match self {
            Self::Disabled => 0,
            Self::Enabled => 1,
            Self::Absent => 2,
            Self::Unreadable => 3,
        }
    }

    const fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::Disabled),
            1 => Some(Self::Enabled),
            2 => Some(Self::Absent),
            3 => Some(Self::Unreadable),
            _ => None,
        }
    }

    /// Classifies one GetVariable result. `found` is false only for
    /// EFI_NOT_FOUND; `value` holds the bytes the firmware reported, which
    /// must be exactly one byte of 0 or 1.
    pub fn from_variable(found: bool, success: bool, value: &[u8]) -> Self {
        match (found, success, value) {
            (false, _, _) => Self::Absent,
            (true, true, [0]) => Self::Disabled,
            (true, true, [1]) => Self::Enabled,
            _ => Self::Unreadable,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// Missing, foreign or malformed extension (including the all-zero page).
    Handoff,
}

/// Encodes the extension written by the `hw-inventory` loader.
pub fn encode(secure_boot: SecureBoot) -> [u8; FACTS_SIZE] {
    let mut bytes = [0u8; FACTS_SIZE];
    bytes[..8].copy_from_slice(&FACTS_MAGIC);
    bytes[8..10].copy_from_slice(&FACTS_VERSION.to_le_bytes());
    bytes[10..12].copy_from_slice(&(FACTS_SIZE as u16).to_le_bytes());
    bytes[12] = secure_boot.code();
    bytes
}

/// Validates a copied extension; reserved bytes must be zero.
pub fn decode(bytes: &[u8; FACTS_SIZE]) -> Result<SecureBoot, Error> {
    if bytes[..8] != FACTS_MAGIC
        || u16::from_le_bytes([bytes[8], bytes[9]]) != FACTS_VERSION
        || u16::from_le_bytes([bytes[10], bytes[11]]) as usize != FACTS_SIZE
        || bytes[13..] != [0; 3]
    {
        return Err(Error::Handoff);
    }
    SecureBoot::from_code(bytes[12]).ok_or(Error::Handoff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_state() {
        for state in [
            SecureBoot::Disabled,
            SecureBoot::Enabled,
            SecureBoot::Absent,
            SecureBoot::Unreadable,
        ] {
            assert_eq!(decode(&encode(state)), Ok(state));
        }
    }

    #[test]
    fn rejects_zero_page_foreign_and_malformed_extensions() {
        assert_eq!(decode(&[0; FACTS_SIZE]), Err(Error::Handoff));
        let good = encode(SecureBoot::Enabled);
        for (index, value) in [(0, b'X'), (8, 2), (10, 15), (12, 4), (13, 1), (15, 1)] {
            let mut bad = good;
            bad[index] = value;
            assert_eq!(decode(&bad), Err(Error::Handoff), "byte {index}");
        }
    }

    #[test]
    fn classifies_variable_results_without_guessing() {
        assert_eq!(
            SecureBoot::from_variable(false, false, &[]),
            SecureBoot::Absent
        );
        assert_eq!(
            SecureBoot::from_variable(true, true, &[0]),
            SecureBoot::Disabled
        );
        assert_eq!(
            SecureBoot::from_variable(true, true, &[1]),
            SecureBoot::Enabled
        );
        for value in [&[][..], &[2], &[1, 0], &[0xff]] {
            assert_eq!(
                SecureBoot::from_variable(true, true, value),
                SecureBoot::Unreadable
            );
        }
        assert_eq!(
            SecureBoot::from_variable(true, false, &[1]),
            SecureBoot::Unreadable
        );
    }
}

//! Allocator-free core of the S09 backup destination service (route step R2
//! of `docs/implementation/S09-DESTINATION-SERVICE.md`, revision 2).
//!
//! The core is pure: it issues no system call. Block I/O goes through the
//! [`device::Broker`] trait (submission, completion poll and a boot-monotonic
//! tick count), and kernel message metadata arrives as [`service::Meta`].
//! Archive verification is abstracted behind [`service::Verifier`] so that the
//! real `volume_core::archive::verify` can be plugged in by the native service
//! (route step R5) without this crate depending on `volume-core`.
//!
//! FNV-64 detects accidental damage only; it is not authentication, and the
//! destination ID is an identifier, not a secret (spec 2.6, 5.4).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

pub mod catalog;
pub mod device;
pub mod service;
pub mod wire;

pub const SECTOR: usize = 512;
/// First sector of the entry area; sectors 0-15 are catalog and reserved.
pub const DATA_START: u64 = 16;
/// Sealed entries per destination (spec 2.8).
pub const MAX_ENTRIES: usize = 16;
/// Label bytes (spec 2.3, `valid_name`).
pub const LABEL: usize = 32;
/// Smallest and largest v1 archive in bytes (spec 2.6, 2.8).
pub const MIN_ARCHIVE: u64 = 80;
pub const MAX_ARCHIVE: u64 = 525_328;
/// Destination ID length (spec 2.6).
pub const ID: usize = 16;

/// Service statuses; the values are those of `volume_core::Status` (spec 2.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    Ok = 0,
    Invalid = -1,
    NoSys = -2,
    Range = -3,
    Access = -5,
    Stale = -10,
    Io = -12,
    Timeout = -13,
    NoDevice = -14,
    Protocol = -15,
    Unknown = -16,
    Full = -17,
    Corrupt = -18,
    NotFound = -19,
    Busy = -21,
}

pub const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

pub fn fnv64(mut value: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        value = (value ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
    }
    value
}

/// Label rule shared with the volume service (`catalog_core::wire::valid_name`).
pub fn valid_name(bytes: &[u8]) -> bool {
    if bytes.is_empty() || bytes.len() > LABEL {
        return false;
    }
    let Ok(name) = core::str::from_utf8(bytes) else {
        return false;
    };
    name != "."
        && name != ".."
        && name.chars().all(|ch| {
            !ch.is_control()
                && ch != '/'
                && ch != '\\'
                && !matches!(ch, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
}

pub fn get32(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}
pub fn get64(bytes: &[u8], at: usize) -> u64 {
    let mut word = [0; 8];
    word.copy_from_slice(&bytes[at..at + 8]);
    u64::from_le_bytes(word)
}
pub fn put32(bytes: &mut [u8], at: usize, value: u32) {
    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
pub fn put64(bytes: &mut [u8], at: usize, value: u64) {
    bytes[at..at + 8].copy_from_slice(&value.to_le_bytes());
}

/// Sectors occupied by an entry of `length` bytes.
pub const fn sectors_for(length: u64) -> u64 {
    length.div_ceil(SECTOR as u64)
}

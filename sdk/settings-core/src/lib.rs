#![no_std]
#![forbid(unsafe_code)]

//! Typed appearance settings and an integrity-only A/B persistence model.
//! No permissions, authentication, encryption, rollback resistance or UI.

use sha2::{Digest, Sha256};

pub const SCHEMA_VERSION: u32 = 1;
pub const SLOT_LEN: usize = 96;
pub const MIN_TEXT_SCALE_PERCENT: u16 = 75;
pub const MAX_TEXT_SCALE_PERCENT: u16 = 300;
const BODY_LEN: usize = SLOT_LEN - 32;
const MAGIC: &[u8; 8] = b"NOSSET01";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Theme {
    System,
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Settings {
    theme: Theme,
    text_scale_percent: u16,
    reduced_motion: bool,
}

impl Settings {
    /// This first schema deliberately contains only appearance/accessibility.
    pub const fn new(
        theme: Theme,
        text_scale_percent: u16,
        reduced_motion: bool,
    ) -> Result<Self, Error> {
        if text_scale_percent < MIN_TEXT_SCALE_PERCENT
            || text_scale_percent > MAX_TEXT_SCALE_PERCENT
        {
            return Err(Error::InvalidValue);
        }
        Ok(Self {
            theme,
            text_scale_percent,
            reduced_motion,
        })
    }

    pub const fn theme(self) -> Theme {
        self.theme
    }

    pub const fn text_scale_percent(self) -> u16 {
        self.text_scale_percent
    }

    pub const fn reduced_motion(self) -> bool {
        self.reduced_motion
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaError {
    Io,
    Full,
}

/// Exactly two fixed slots, owned exclusively by this settings owner.
/// Reads fill a complete slot or fail. Writes may be partial, or complete even
/// when returning an error. Successful flush promises prior writes are durable;
/// the actual adapter must establish that promise. Use the same medium for
/// recovery and mutation; caller identity, authorization and locking are external.
pub trait SettingsMedia {
    fn read(&mut self, slot: usize, output: &mut [u8; SLOT_LEN]) -> Result<(), MediaError>;
    fn write(&mut self, slot: usize, data: &[u8; SLOT_LEN]) -> Result<(), MediaError>;
    fn flush(&mut self) -> Result<(), MediaError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidValue,
    RecoveryRequired,
    NotInitialized,
    NotBlank,
    Corrupt,
    UnsupportedVersion,
    NoPrevious,
    GenerationExhausted,
    Media(MediaError),
    ReadbackMismatch,
}

#[derive(Clone, Copy)]
struct Record {
    generation: u64,
    current: Settings,
    previous: Option<Settings>,
}

#[derive(Clone, Copy)]
struct State {
    slot: usize,
    record: Record,
}

enum Probe {
    Blank,
    Bad,
    Good(Record),
}

/// No current value is available until explicit initialization or recovery.
/// Any failed initialize/recover/set/revert retires the in-memory state.
/// Discard returned values after an error; they are historical observations,
/// not proof of what the medium currently contains.
#[derive(Default)]
pub struct SettingsStore {
    state: Option<State>,
}

impl SettingsStore {
    pub const fn new() -> Self {
        Self { state: None }
    }

    pub fn get(&self) -> Result<Settings, Error> {
        Ok(self.state.ok_or(Error::RecoveryRequired)?.record.current)
    }

    pub fn generation(&self) -> Result<u64, Error> {
        Ok(self.state.ok_or(Error::RecoveryRequired)?.record.generation)
    }

    /// Explicit initialization is permitted only when both complete slots are
    /// zero. Corrupt or previously initialized storage is never reformatted.
    pub fn initialize<M: SettingsMedia>(
        &mut self,
        media: &mut M,
        value: Settings,
    ) -> Result<u64, Error> {
        self.state = None;
        let first = read_slot(media, 0)?;
        let second = read_slot(media, 1)?;
        if first.iter().chain(second.iter()).any(|byte| *byte != 0) {
            return Err(Error::NotBlank);
        }
        let record = Record {
            generation: 1,
            current: value,
            previous: None,
        };
        persist(media, 0, record)?;
        self.state = Some(State { slot: 0, record });
        Ok(1)
    }

    /// Selects the highest valid generation; read errors and checksum-valid
    /// unknown versions abort instead of silently choosing an older slot.
    /// The selected copy is published only after a successful flush.
    pub fn recover<M: SettingsMedia>(&mut self, media: &mut M) -> Result<u64, Error> {
        self.state = None;
        let first = probe(&read_slot(media, 0)?)?;
        let second = probe(&read_slot(media, 1)?)?;
        let (slot, record) = match (first, second) {
            (Probe::Blank, Probe::Blank) => return Err(Error::NotInitialized),
            (Probe::Good(a), Probe::Good(b)) => {
                if a.generation == b.generation {
                    return Err(Error::Corrupt);
                }
                if a.generation > b.generation {
                    (0, a)
                } else {
                    (1, b)
                }
            }
            (Probe::Good(a), _) => (0, a),
            (_, Probe::Good(b)) => (1, b),
            _ => return Err(Error::Corrupt),
        };
        // A selected record may only be cached after an earlier failed flush.
        // The next commit overwrites the other slot, so establish durability of
        // the selected copy first; on failure stay retired.
        let selected = read_slot(media, slot)?;
        media.flush().map_err(Error::Media)?;
        if read_slot(media, slot)? != selected {
            return Err(Error::ReadbackMismatch);
        }
        self.state = Some(State { slot, record });
        Ok(record.generation)
    }

    /// Always commits a new generation, even when the value is unchanged.
    pub fn set<M: SettingsMedia>(&mut self, media: &mut M, value: Settings) -> Result<u64, Error> {
        let state = self.state.take().ok_or(Error::RecoveryRequired)?;
        self.commit(media, state, value)
    }

    /// One-level revert persists the previous committed value at a NEW
    /// generation. Repeated revert toggles the last two committed values.
    pub fn revert<M: SettingsMedia>(&mut self, media: &mut M) -> Result<u64, Error> {
        let state = self.state.take().ok_or(Error::RecoveryRequired)?;
        let previous = state.record.previous.ok_or(Error::NoPrevious)?;
        self.commit(media, state, previous)
    }

    fn commit<M: SettingsMedia>(
        &mut self,
        media: &mut M,
        state: State,
        value: Settings,
    ) -> Result<u64, Error> {
        let generation = state
            .record
            .generation
            .checked_add(1)
            .ok_or(Error::GenerationExhausted)?;
        let record = Record {
            generation,
            current: value,
            previous: Some(state.record.current),
        };
        let slot = 1 - state.slot;
        persist(media, slot, record)?;
        self.state = Some(State { slot, record });
        Ok(generation)
    }
}

fn read_slot<M: SettingsMedia>(media: &mut M, slot: usize) -> Result<[u8; SLOT_LEN], Error> {
    let mut bytes = [0; SLOT_LEN];
    media.read(slot, &mut bytes).map_err(Error::Media)?;
    Ok(bytes)
}

fn persist<M: SettingsMedia>(media: &mut M, slot: usize, record: Record) -> Result<(), Error> {
    let bytes = encode(record);
    media.write(slot, &bytes).map_err(Error::Media)?;
    media.flush().map_err(Error::Media)?;
    if read_slot(media, slot)? != bytes {
        return Err(Error::ReadbackMismatch);
    }
    Ok(())
}

fn encode_settings(settings: Settings) -> [u8; 8] {
    let mut bytes = [0; 8];
    bytes[0] = match settings.theme {
        Theme::System => 0,
        Theme::Light => 1,
        Theme::Dark => 2,
    };
    bytes[1] = u8::from(settings.reduced_motion);
    bytes[2..4].copy_from_slice(&settings.text_scale_percent.to_le_bytes());
    bytes
}

fn decode_settings(bytes: &[u8]) -> Option<Settings> {
    let theme = match bytes[0] {
        0 => Theme::System,
        1 => Theme::Light,
        2 => Theme::Dark,
        _ => return None,
    };
    if bytes[1] > 1 || bytes[4..].iter().any(|byte| *byte != 0) {
        return None;
    }
    Settings::new(
        theme,
        u16::from_le_bytes([bytes[2], bytes[3]]),
        bytes[1] != 0,
    )
    .ok()
}

fn encode(record: Record) -> [u8; SLOT_LEN] {
    let mut bytes = [0; SLOT_LEN];
    bytes[..8].copy_from_slice(MAGIC);
    bytes[8..12].copy_from_slice(&SCHEMA_VERSION.to_le_bytes());
    bytes[12] = u8::from(record.previous.is_some());
    bytes[16..24].copy_from_slice(&record.generation.to_le_bytes());
    bytes[24..32].copy_from_slice(&encode_settings(record.current));
    if let Some(previous) = record.previous {
        bytes[32..40].copy_from_slice(&encode_settings(previous));
    }
    let digest = Sha256::digest(&bytes[..BODY_LEN]);
    bytes[BODY_LEN..].copy_from_slice(&digest);
    bytes
}

fn probe(bytes: &[u8; SLOT_LEN]) -> Result<Probe, Error> {
    if bytes.iter().all(|byte| *byte == 0) {
        return Ok(Probe::Blank);
    }
    if bytes[..8] != MAGIC[..] || bytes[BODY_LEN..] != Sha256::digest(&bytes[..BODY_LEN])[..] {
        return Ok(Probe::Bad);
    }
    let version = u32::from_le_bytes(bytes[8..12].try_into().map_err(|_| Error::Corrupt)?);
    if version != SCHEMA_VERSION {
        return Err(Error::UnsupportedVersion);
    }
    let generation = u64::from_le_bytes(bytes[16..24].try_into().map_err(|_| Error::Corrupt)?);
    if generation == 0
        || bytes[12] > 1
        || bytes[13..16]
            .iter()
            .chain(bytes[40..BODY_LEN].iter())
            .any(|byte| *byte != 0)
    {
        return Ok(Probe::Bad);
    }
    let Some(current) = decode_settings(&bytes[24..32]) else {
        return Ok(Probe::Bad);
    };
    let previous = if bytes[12] == 1 {
        let Some(previous) = decode_settings(&bytes[32..40]) else {
            return Ok(Probe::Bad);
        };
        Some(previous)
    } else {
        if bytes[32..40].iter().any(|byte| *byte != 0) {
            return Ok(Probe::Bad);
        }
        None
    };
    Ok(Probe::Good(Record {
        generation,
        current,
        previous,
    }))
}

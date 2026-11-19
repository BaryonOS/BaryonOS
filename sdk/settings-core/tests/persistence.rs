use nieuw_os_settings_core::{
    Error, MediaError, Settings, SettingsMedia, SettingsStore, Theme, MAX_TEXT_SCALE_PERCENT,
    MIN_TEXT_SCALE_PERCENT, SCHEMA_VERSION, SLOT_LEN,
};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq)]
enum Event {
    Read(usize),
    Write(usize),
    Flush,
}

#[derive(Clone)]
struct Memory {
    slots: [[u8; SLOT_LEN]; 2],
    events: Vec<Event>,
    read_error: Option<usize>,
    write_error: Option<(usize, MediaError)>,
    fail_flush: bool,
    drop_write: bool,
    wrong_readback: bool,
}

impl Memory {
    fn blank() -> Self {
        Self {
            slots: [[0; SLOT_LEN]; 2],
            events: Vec::new(),
            read_error: None,
            write_error: None,
            fail_flush: false,
            drop_write: false,
            wrong_readback: false,
        }
    }
}

impl SettingsMedia for Memory {
    fn read(&mut self, slot: usize, output: &mut [u8; SLOT_LEN]) -> Result<(), MediaError> {
        self.events.push(Event::Read(slot));
        if self.read_error == Some(slot) {
            return Err(MediaError::Io);
        }
        *output = self.slots[slot];
        if self.wrong_readback {
            output[0] ^= 1;
        }
        Ok(())
    }
    fn write(&mut self, slot: usize, data: &[u8; SLOT_LEN]) -> Result<(), MediaError> {
        self.events.push(Event::Write(slot));
        if let Some((limit, error)) = self.write_error {
            let limit = limit.min(SLOT_LEN);
            self.slots[slot][..limit].copy_from_slice(&data[..limit]);
            return Err(error);
        }
        if !self.drop_write {
            self.slots[slot] = *data;
        }
        Ok(())
    }
    fn flush(&mut self) -> Result<(), MediaError> {
        self.events.push(Event::Flush);
        if self.fail_flush {
            Err(MediaError::Io)
        } else {
            Ok(())
        }
    }
}

fn original() -> Settings {
    Settings::new(Theme::System, 100, false).unwrap()
}
fn changed() -> Settings {
    Settings::new(Theme::Dark, 150, true).unwrap()
}

fn initialized() -> (Memory, SettingsStore) {
    let mut media = Memory::blank();
    let mut store = SettingsStore::new();
    assert_eq!(store.initialize(&mut media, original()), Ok(1));
    media.events.clear();
    (media, store)
}

fn updated() -> (Memory, SettingsStore) {
    let (mut media, mut store) = initialized();
    assert_eq!(store.set(&mut media, changed()), Ok(2));
    media.events.clear();
    (media, store)
}

fn rechecksum(bytes: &mut [u8; SLOT_LEN]) {
    let digest = Sha256::digest(&bytes[..SLOT_LEN - 32]);
    bytes[SLOT_LEN - 32..].copy_from_slice(&digest);
}

fn unknown(store: &mut SettingsStore, media: &mut Memory) {
    assert_eq!(store.get(), Err(Error::RecoveryRequired));
    assert_eq!(store.generation(), Err(Error::RecoveryRequired));
    let events = media.events.len();
    assert_eq!(store.set(media, original()), Err(Error::RecoveryRequired));
    assert_eq!(store.revert(media), Err(Error::RecoveryRequired));
    assert_eq!(media.events.len(), events);
}

#[test]
fn typed_values_validate_the_declared_profile() {
    for scale in [
        0,
        MIN_TEXT_SCALE_PERCENT - 1,
        MAX_TEXT_SCALE_PERCENT + 1,
        u16::MAX,
    ] {
        assert_eq!(
            Settings::new(Theme::Light, scale, false),
            Err(Error::InvalidValue)
        );
    }
    for scale in [MIN_TEXT_SCALE_PERCENT, 100, MAX_TEXT_SCALE_PERCENT] {
        let settings = Settings::new(Theme::Light, scale, true).unwrap();
        assert_eq!(settings.theme(), Theme::Light);
        assert_eq!(settings.text_scale_percent(), scale);
        assert!(settings.reduced_motion());
    }
}

#[test]
fn explicit_initialization_requires_entirely_blank_storage() {
    let mut media = Memory::blank();
    let mut store = SettingsStore::new();
    unknown(&mut store, &mut media);
    assert_eq!(store.recover(&mut media), Err(Error::NotInitialized));
    assert_eq!(store.get(), Err(Error::RecoveryRequired));
    assert!(media
        .events
        .iter()
        .all(|event| matches!(event, Event::Read(_))));
    media.events.clear();
    assert_eq!(store.initialize(&mut media, original()), Ok(1));
    assert_eq!(
        media.events,
        [
            Event::Read(0),
            Event::Read(1),
            Event::Write(0),
            Event::Flush,
            Event::Read(0)
        ]
    );
    assert_eq!(store.get(), Ok(original()));
    let saved = media.slots;
    assert_eq!(
        store.initialize(&mut media, changed()),
        Err(Error::NotBlank)
    );
    assert_eq!(media.slots, saved);
    unknown(&mut store, &mut media);
    assert_eq!(store.recover(&mut media), Ok(1));
    let mut media = Memory::blank();
    media.slots[1][SLOT_LEN - 1] = 1; // Zero header is not sufficient to call storage blank.
    assert_eq!(
        store.initialize(&mut media, original()),
        Err(Error::NotBlank)
    );
    assert_eq!(store.recover(&mut media), Err(Error::Corrupt));
}

#[test]
fn canonical_slots_use_write_flush_readback_and_preserve_the_active_copy() {
    let (mut media, mut store) = initialized();
    let slot = media.slots[0];
    assert_eq!(&slot[..8], b"NOSSET01");
    assert_eq!(&slot[8..12], &SCHEMA_VERSION.to_le_bytes());
    assert_eq!(&slot[12..16], &[0; 4]);
    assert_eq!(&slot[16..24], &1u64.to_le_bytes());
    assert_eq!(&slot[24..32], &[0, 0, 100, 0, 0, 0, 0, 0]);
    assert!(slot[32..64].iter().all(|byte| *byte == 0));
    assert_eq!(&slot[64..], &Sha256::digest(&slot[..64])[..]);
    assert_eq!(store.set(&mut media, changed()), Ok(2));
    assert_eq!(
        media.events,
        [Event::Write(1), Event::Flush, Event::Read(1)]
    );
    assert_eq!(media.slots[0], slot);
    assert_eq!(&media.slots[1][24..32], &[2, 1, 150, 0, 0, 0, 0, 0]);
    assert_eq!(&media.slots[1][32..40], &slot[24..32]);
}

#[test]
fn reload_and_revert_always_produce_new_committed_generations() {
    let (mut media, _) = updated();
    let mut store = SettingsStore::new();
    assert_eq!(store.recover(&mut media), Ok(2));
    assert_eq!(store.get(), Ok(changed()));
    assert_eq!(store.revert(&mut media), Ok(3));
    assert_eq!(store.get(), Ok(original()));
    let mut reloaded = SettingsStore::new();
    assert_eq!(reloaded.recover(&mut media), Ok(3));
    assert_eq!(reloaded.get(), Ok(original()));
    assert_eq!(reloaded.revert(&mut media), Ok(4));
    assert_eq!(reloaded.get(), Ok(changed()));
    assert_eq!(reloaded.set(&mut media, changed()), Ok(5));
    assert_eq!(reloaded.revert(&mut media), Ok(6));
    assert_eq!(reloaded.get(), Ok(changed()));
}

#[test]
fn missing_previous_value_fails_without_mutation_or_current_state_claim() {
    let (mut media, mut store) = initialized();
    assert_eq!(store.revert(&mut media), Err(Error::NoPrevious));
    assert!(media.events.is_empty());
    unknown(&mut store, &mut media);
    assert_eq!(store.recover(&mut media), Ok(1));
}

#[test]
fn every_torn_write_and_complete_write_error_requires_recovery() {
    let (baseline, _) = updated();
    let mut complete = baseline.clone();
    let mut preview = SettingsStore::new();
    assert_eq!(preview.recover(&mut complete), Ok(2));
    assert_eq!(preview.set(&mut complete, original()), Ok(3));
    // Every partial prefix differs from the intended new record, including
    // when the old inactive slot already contains a checksum-valid record.
    assert_ne!(
        complete.slots[0][SLOT_LEN - 1],
        baseline.slots[0][SLOT_LEN - 1]
    );
    for cut in 0..=SLOT_LEN {
        let mut media = baseline.clone();
        let mut store = SettingsStore::new();
        assert_eq!(store.recover(&mut media), Ok(2));
        media.write_error = Some((cut, MediaError::Io));
        assert_eq!(
            store.set(&mut media, original()),
            Err(Error::Media(MediaError::Io)),
            "cut {cut}"
        );
        assert_eq!(media.slots[1], baseline.slots[1]);
        unknown(&mut store, &mut media);
        media.write_error = None;
        if cut == SLOT_LEN {
            assert_eq!(store.recover(&mut media), Ok(3));
            assert_eq!(store.get(), Ok(original()));
        } else {
            assert_eq!(store.recover(&mut media), Ok(2), "cut {cut}");
            assert_eq!(store.get(), Ok(changed()));
        }
    }
}

#[test]
fn initialization_errors_never_publish_defaults_or_silently_reformat() {
    for cut in [0, 20, SLOT_LEN] {
        let mut media = Memory::blank();
        let mut store = SettingsStore::new();
        media.write_error = Some((cut, MediaError::Full));
        assert_eq!(
            store.initialize(&mut media, changed()),
            Err(Error::Media(MediaError::Full))
        );
        unknown(&mut store, &mut media);
        media.write_error = None;
        match cut {
            0 => assert_eq!(store.recover(&mut media), Err(Error::NotInitialized)),
            SLOT_LEN => {
                assert_eq!(store.recover(&mut media), Ok(1));
                assert_eq!(store.get(), Ok(changed()));
            }
            _ => {
                assert_eq!(store.recover(&mut media), Err(Error::Corrupt));
                assert_eq!(
                    store.initialize(&mut media, original()),
                    Err(Error::NotBlank)
                );
            }
        }
    }
}

#[test]
fn full_flush_readback_and_missing_write_failures_never_acknowledge() {
    for mode in 0..5 {
        let (mut media, mut store) = initialized();
        let expected = match mode {
            0 => {
                media.write_error = Some((0, MediaError::Full));
                Error::Media(MediaError::Full)
            }
            1 => {
                media.fail_flush = true;
                Error::Media(MediaError::Io)
            }
            2 => {
                media.read_error = Some(1);
                Error::Media(MediaError::Io)
            }
            3 => {
                media.drop_write = true;
                Error::ReadbackMismatch
            }
            _ => {
                media.wrong_readback = true;
                Error::ReadbackMismatch
            }
        };
        assert_eq!(store.set(&mut media, changed()), Err(expected));
        unknown(&mut store, &mut media);
        media.write_error = None;
        media.fail_flush = false;
        media.read_error = None;
        media.drop_write = false;
        media.wrong_readback = false;
        let expected_generation = if mode == 0 || mode == 3 { 1 } else { 2 };
        assert_eq!(store.recover(&mut media), Ok(expected_generation));
        assert_eq!(
            store.get(),
            Ok(if expected_generation == 1 {
                original()
            } else {
                changed()
            })
        );
    }
}

#[test]
fn revert_failure_also_requires_recovery_without_replaying_old_ack() {
    let (mut media, mut store) = updated();
    media.write_error = Some((SLOT_LEN, MediaError::Full));
    assert_eq!(
        store.revert(&mut media),
        Err(Error::Media(MediaError::Full))
    );
    unknown(&mut store, &mut media);
    media.write_error = None;
    assert_eq!(store.recover(&mut media), Ok(3));
    assert_eq!(store.get(), Ok(original()));
    assert_eq!(store.revert(&mut media), Ok(4));
    assert_eq!(store.get(), Ok(changed()));
}

#[test]
fn corrupt_inactive_fallback_is_distinct_from_any_read_error() {
    let (mut media, mut store) = updated();
    media.slots[1][64] ^= 1;
    assert_eq!(store.recover(&mut media), Ok(1));
    assert_eq!(store.get(), Ok(original()));
    for slot in 0..2 {
        media.read_error = Some(slot);
        assert_eq!(store.recover(&mut media), Err(Error::Media(MediaError::Io)));
        unknown(&mut store, &mut media);
        media.read_error = None;
    }
    for slot in 0..2 {
        let mut blank = Memory::blank();
        blank.read_error = Some(slot);
        assert_eq!(
            store.initialize(&mut blank, original()),
            Err(Error::Media(MediaError::Io))
        );
        assert!(blank
            .events
            .iter()
            .all(|event| matches!(event, Event::Read(_))));
    }
}

#[test]
fn both_corrupt_and_equal_generation_copies_are_rejected() {
    let (mut media, mut store) = updated();
    for slot in &mut media.slots {
        slot[64] ^= 1;
    }
    assert_eq!(store.recover(&mut media), Err(Error::Corrupt));
    unknown(&mut store, &mut media);
    let saved = media.slots;
    assert_eq!(
        store.initialize(&mut media, original()),
        Err(Error::NotBlank)
    );
    assert_eq!(media.slots, saved);
    let (mut media, mut store) = initialized();
    media.slots[1] = media.slots[0];
    assert_eq!(store.recover(&mut media), Err(Error::Corrupt));
    unknown(&mut store, &mut media);
}

#[test]
fn checksum_valid_unknown_versions_abort_even_with_another_good_slot() {
    for slot in 0..2 {
        for version in [0u32, SCHEMA_VERSION + 1, u32::MAX] {
            let (mut media, mut store) = updated();
            media.slots[slot][8..12].copy_from_slice(&version.to_le_bytes());
            rechecksum(&mut media.slots[slot]);
            assert_eq!(store.recover(&mut media), Err(Error::UnsupportedVersion));
            unknown(&mut store, &mut media);
        }
    }
    let (mut media, mut store) = updated();
    media.slots[1][8] = 2; // Without a valid checksum this remains a corrupt slot, not a trusted future schema.
    assert_eq!(store.recover(&mut media), Ok(1));
}

#[test]
fn rejects_checksum_valid_invalid_settings_flags_padding_and_generation() {
    let (baseline, _) = initialized();
    for (offset, value) in [
        (0, b'X'),
        (12, 2),
        (13, 1),
        (24, 3),
        (25, 2),
        (26, 74),
        (28, 1),
        (32, 1),
        (40, 1),
        (63, 1),
    ] {
        let mut bad = baseline.slots[0];
        bad[offset] = value;
        rechecksum(&mut bad);
        let mut media = baseline.clone();
        media.slots = [bad, bad];
        assert_eq!(
            SettingsStore::new().recover(&mut media),
            Err(Error::Corrupt),
            "offset {offset}"
        );
    }
    let mut media = baseline;
    media.slots[0][16..24].fill(0);
    rechecksum(&mut media.slots[0]);
    assert_eq!(
        SettingsStore::new().recover(&mut media),
        Err(Error::Corrupt)
    );
    let (mut media, _) = updated();
    media.slots[1][36] = 1; // Reserved bytes in a present previous value are checked too.
    rechecksum(&mut media.slots[1]);
    media.slots[0] = media.slots[1];
    assert_eq!(
        SettingsStore::new().recover(&mut media),
        Err(Error::Corrupt)
    );
}

#[test]
fn generation_exhaustion_never_wraps_or_writes() {
    let (mut media, mut store) = updated();
    media.slots[1][16..24].copy_from_slice(&u64::MAX.to_le_bytes());
    rechecksum(&mut media.slots[1]);
    for revert in [false, true] {
        assert_eq!(store.recover(&mut media), Ok(u64::MAX));
        media.events.clear();
        let result = if revert {
            store.revert(&mut media)
        } else {
            store.set(&mut media, original())
        };
        assert_eq!(result, Err(Error::GenerationExhausted));
        assert!(media.events.is_empty());
        unknown(&mut store, &mut media);
    }
}

/// Separate volatile cache and durable image: writes reach only the cache and
/// a successful flush copies the cache to the durable image.
struct Cached {
    cache: [[u8; SLOT_LEN]; 2],
    durable: [[u8; SLOT_LEN]; 2],
    fail_flush: bool,
}

impl Cached {
    fn power_loss(&mut self) {
        self.cache = self.durable;
    }
}

impl SettingsMedia for Cached {
    fn read(&mut self, slot: usize, output: &mut [u8; SLOT_LEN]) -> Result<(), MediaError> {
        *output = self.cache[slot];
        Ok(())
    }
    fn write(&mut self, slot: usize, data: &[u8; SLOT_LEN]) -> Result<(), MediaError> {
        self.cache[slot] = *data;
        Ok(())
    }
    fn flush(&mut self) -> Result<(), MediaError> {
        if self.fail_flush {
            return Err(MediaError::Io);
        }
        self.durable = self.cache;
        Ok(())
    }
}

#[test]
fn recovery_requires_a_durability_barrier_before_publishing_a_cached_record() {
    let mut media = Cached {
        cache: [[0; SLOT_LEN]; 2],
        durable: [[0; SLOT_LEN]; 2],
        fail_flush: false,
    };
    let mut store = SettingsStore::new();
    assert_eq!(store.initialize(&mut media, original()), Ok(1));
    // Generation 2 reaches only the cache: readable, checksum-valid, not durable.
    media.fail_flush = true;
    assert_eq!(
        store.set(&mut media, changed()),
        Err(Error::Media(MediaError::Io))
    );
    let mut recovered = SettingsStore::new();
    assert_eq!(
        recovered.recover(&mut media),
        Err(Error::Media(MediaError::Io))
    );
    assert_eq!(recovered.get(), Err(Error::RecoveryRequired));
    assert_eq!(
        recovered.set(&mut media, original()),
        Err(Error::RecoveryRequired)
    );
    // Without the barrier the acknowledged generation 1 is still the durable copy.
    let durable = media.durable;
    media.power_loss();
    assert_eq!(media.cache, durable);
    assert_eq!(
        recovered.recover(&mut media),
        Err(Error::Media(MediaError::Io))
    );
    media.fail_flush = false;
    assert_eq!(recovered.recover(&mut media), Ok(1));
    assert_eq!(recovered.get(), Ok(original()));

    // Successful-flush control: a cached generation becomes durable on recovery.
    media.fail_flush = true;
    assert!(recovered.set(&mut media, changed()).is_err());
    media.fail_flush = false;
    let mut control = SettingsStore::new();
    assert_eq!(control.recover(&mut media), Ok(2));
    media.power_loss();
    let mut after = SettingsStore::new();
    assert_eq!(after.recover(&mut media), Ok(2));
    assert_eq!(after.get(), Ok(changed()));
}

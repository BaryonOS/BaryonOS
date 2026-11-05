use nieuw_os_usb_core::boot::{
    Error, KeyboardChanges, KeyboardReport, KeyboardState, KeyboardStatus, MouseReport,
};

fn report(modifiers: u8, keys: [u8; 6]) -> [u8; 8] {
    [
        modifiers, 0, keys[0], keys[1], keys[2], keys[3], keys[4], keys[5],
    ]
}

fn no_keys(changes: &KeyboardChanges) {
    assert!(changes.pressed().is_empty());
    assert!(changes.released().is_empty());
}

#[test]
fn keyboard_report_is_exact_and_preserves_every_modifier_and_oem_value() {
    for length in 0..=256 {
        let bytes = vec![0; length];
        assert_eq!(
            KeyboardReport::parse(&bytes).is_ok(),
            length == 8,
            "length {length}"
        );
    }
    for modifiers in 0..=u8::MAX {
        for oem in 0..=u8::MAX {
            let mut bytes = report(modifiers, [4, 5, 6, 7, 8, 255]);
            bytes[1] = oem;
            let parsed = KeyboardReport::parse(&bytes).unwrap();
            assert_eq!(parsed.modifiers(), modifiers);
            assert_eq!(parsed.keys(), Ok([4, 5, 6, 7, 8, 255]));
        }
    }
}

#[test]
fn all_keyboard_usages_and_slots_are_classified_without_a_legacy_cutoff() {
    for slot in 0..6 {
        for usage in 0..=u8::MAX {
            let mut keys = [0; 6];
            keys[slot] = usage;
            let parsed = KeyboardReport::parse(&report(0xa5, keys)).unwrap();
            let expected = match usage {
                1 => Err(KeyboardStatus::Rollover),
                2 => Err(KeyboardStatus::PostFail),
                3 => Err(KeyboardStatus::Undefined),
                _ => Ok(keys),
            };
            assert_eq!(parsed.keys(), expected);
        }
    }
}

#[test]
fn duplicate_nonzero_usages_fail_without_changing_prior_state() {
    for usage in 4..=u8::MAX {
        for first in 0..6 {
            for second in first + 1..6 {
                let mut state = KeyboardState::new();
                state.update(&report(3, [4, 5, 0, 0, 0, 0])).unwrap();
                let old = state;
                let mut keys = [0; 6];
                keys[first] = usage;
                keys[second] = usage;
                assert_eq!(
                    state.update(&report(0, keys)),
                    Err(Error::DuplicateKey(usage))
                );
                assert_eq!(state, old);
            }
        }
    }
}

#[test]
fn unordered_array_is_a_set_not_make_break_or_typematic() {
    let mut state = KeyboardState::new();
    let initial = state.update(&report(0x81, [4, 5, 6, 7, 8, 9])).unwrap();
    assert_eq!(initial.pressed(), &[4, 5, 6, 7, 8, 9]);
    assert_eq!(initial.modifiers_pressed, 0x81);
    for rotation in 0..6 {
        let mut keys = [4, 5, 6, 7, 8, 9];
        keys.rotate_left(rotation);
        let repeated = state.update(&report(0x81, keys)).unwrap();
        no_keys(&repeated);
        assert_eq!(repeated.modifiers_pressed, 0);
        assert_eq!(repeated.modifiers_released, 0);
        assert_eq!(repeated.status, None);
    }
    let replaced = state.update(&report(2, [10, 11, 12, 13, 14, 15])).unwrap();
    assert_eq!(replaced.pressed(), &[10, 11, 12, 13, 14, 15]);
    assert_eq!(replaced.released(), &[9, 4, 5, 6, 7, 8]);
    assert_eq!(replaced.modifiers_pressed, 2);
    assert_eq!(replaced.modifiers_released, 0x81);
}

#[test]
fn phantom_reports_keep_prior_keys_but_modifiers_continue_and_recovery_releases() {
    for (usage, status) in [
        (1, KeyboardStatus::Rollover),
        (2, KeyboardStatus::PostFail),
        (3, KeyboardStatus::Undefined),
    ] {
        let mut state = KeyboardState::new();
        state.update(&report(2, [4, 5, 0, 0, 0, 0])).unwrap();
        let phantom = state.update(&report(0x40, [usage; 6])).unwrap();
        no_keys(&phantom);
        assert_eq!(phantom.status, Some(status));
        assert_eq!(phantom.modifiers_pressed, 0x40);
        assert_eq!(phantom.modifiers_released, 2);
        let recovery = state.update(&report(0x40, [5, 6, 0, 0, 0, 0])).unwrap();
        assert_eq!(recovery.released(), &[4]);
        assert_eq!(recovery.pressed(), &[6]);
        assert_eq!(recovery.status, None);
        let cleared = state.clear();
        assert_eq!(cleared.released(), &[5, 6]);
        assert_eq!(cleared.modifiers_released, 0x40);
        assert!(cleared.pressed().is_empty());
        assert_eq!(state.clear(), KeyboardChanges::default());
        assert_eq!(state, KeyboardState::new());
    }
}

#[test]
fn truncated_or_extended_keyboard_transfers_cannot_clear_held_keys() {
    for length in 0..=256 {
        if length == 8 {
            continue;
        }
        let mut state = KeyboardState::new();
        state.update(&report(0xff, [4, 5, 6, 7, 8, 9])).unwrap();
        let old = state;
        assert_eq!(state.update(&vec![0; length]), Err(Error::Length));
        assert_eq!(state, old);
    }
}

#[test]
fn mouse_is_bounded_ignores_oem_and_decodes_signed_displacements_exactly() {
    for length in 0..=256 {
        let mut bytes = vec![0x7f; length];
        if length != 0 {
            bytes[0] = 0xff;
        }
        let expected = if (3..=8).contains(&length) {
            Ok(MouseReport {
                buttons: 7,
                dx: 127,
                dy: 127,
            })
        } else {
            Err(Error::Length)
        };
        assert_eq!(MouseReport::parse(&bytes), expected);
    }
    for buttons in 0..=u8::MAX {
        for raw in 0..=u8::MAX {
            for axis in 1..=2 {
                let mut bytes = [buttons, 0, 0, 255, 255, 255, 255, 255];
                bytes[axis] = raw;
                let expected = if raw == 128 {
                    Err(Error::DisplacementOutOfRange)
                } else {
                    Ok(MouseReport {
                        buttons: buttons & 7,
                        dx: if axis == 1 { raw as i8 } else { 0 },
                        dy: if axis == 2 { raw as i8 } else { 0 },
                    })
                };
                assert_eq!(MouseReport::parse(&bytes), expected);
            }
        }
    }
}

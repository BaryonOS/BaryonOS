//! Pure policy regressions used by the real native init. These tests do not
//! replace the actual syscall, process identities or native frame acceptance.
use nieuw_os_window_session::{
    boot::READY,
    recovery::{ready_bit, Group, Recovery},
};

fn old() -> Group {
    Group {
        generation: 1,
        surface_epoch: 101,
        input_epoch: 201,
    }
}
fn fresh() -> Group {
    Group {
        generation: 2,
        surface_epoch: 102,
        input_epoch: 202,
    }
}
fn ready(group: Group, role: u16) -> [u8; 32] {
    let mut bytes = [0; 32];
    bytes[..4].copy_from_slice(b"IPD1");
    bytes[4..6].copy_from_slice(&1u16.to_le_bytes());
    bytes[6..8].copy_from_slice(&role.to_le_bytes());
    let epoch = if role == 3 {
        group.surface_epoch
    } else {
        group.input_epoch
    };
    bytes[8..16].copy_from_slice(&epoch.to_le_bytes());
    bytes[16..24].copy_from_slice(&group.generation.to_le_bytes());
    bytes[24..].copy_from_slice(&u64::from(role != 2).to_le_bytes());
    bytes
}

#[test]
fn complete_old_ready_mask_never_bypasses_both_pause_acks_or_actual_installation() {
    let mut recovery = Recovery::begin(old()).unwrap();
    assert!(recovery.awaiting_pause());
    for pause in [0, 1, 2, 4, 7, u8::MAX] {
        let before = recovery;
        assert!(!recovery.can_restart(pause));
        assert!(!recovery.can_rebind(old(), 31, pause));
        assert!(!recovery.can_rebind(fresh(), 31, pause));
        assert!(recovery.installed(fresh(), pause).is_err());
        assert!(recovery.rebind_sent(old(), 31, pause).is_err());
        assert_eq!(recovery, before);
        assert!(!recovery.complete(3));
    }
    assert!(recovery.can_restart(3));
    assert!(!recovery.can_rebind(old(), 31, 3));
    assert!(!recovery.can_rebind(fresh(), 31, 3));
    assert!(recovery.rebind_sent(fresh(), 31, 3).is_err());
    recovery.installed(fresh(), 3).unwrap();
    assert!(recovery.awaiting_ready());
    assert!(!recovery.awaiting_pause());
    assert!(!recovery.can_restart(3));
    assert!(!recovery.can_rebind(old(), 31, 0));
}

#[test]
fn fresh_installation_rejects_invalid_or_reused_group_values_without_advancing_stage() {
    for invalid in [
        Group {
            generation: 0,
            ..old()
        },
        Group {
            surface_epoch: 0,
            ..old()
        },
        Group {
            input_epoch: 0,
            ..old()
        },
    ] {
        assert!(Recovery::begin(invalid).is_err());
    }
    for invalid in [
        old(),
        Group {
            generation: 0,
            ..fresh()
        },
        Group {
            surface_epoch: 0,
            ..fresh()
        },
        Group {
            input_epoch: 0,
            ..fresh()
        },
        Group {
            surface_epoch: old().surface_epoch,
            ..fresh()
        },
        Group {
            input_epoch: old().input_epoch,
            ..fresh()
        },
    ] {
        let mut recovery = Recovery::begin(old()).unwrap();
        let before = recovery;
        assert!(recovery.installed(invalid, 3).is_err());
        assert_eq!(recovery, before);
        assert!(recovery.can_restart(3));
    }
}

#[test]
fn rebind_requires_all_three_current_service_roles_and_then_both_new_app_acks() {
    let mut recovery = Recovery::begin(old()).unwrap();
    recovery.installed(fresh(), 3).unwrap();
    let mut mask = 3;
    for (role, index) in [(3, 2), (1, 3), (2, 4)] {
        assert!(!recovery.can_rebind(fresh(), mask, 0));
        assert!(!recovery.complete(3));
        mask |= ready_bit(fresh(), Some(index), READY, &ready(fresh(), role)).unwrap();
    }
    assert_eq!(mask, 31);
    assert!(recovery.can_rebind(fresh(), mask, 0));
    for pause in [1, 2, 3, 4, u8::MAX] {
        assert!(recovery.rebind_sent(fresh(), mask, pause).is_err());
    }
    for missing in [3, 7, 11, 15, 19, 23, 27, 30] {
        assert!(recovery.rebind_sent(fresh(), missing, 0).is_err());
    }
    let before = recovery;
    assert!(recovery.installed(fresh(), 3).is_err());
    assert_eq!(recovery, before);
    recovery.rebind_sent(fresh(), mask, 0).unwrap();
    assert!(recovery.awaiting_rebind_ack());
    assert!(!recovery.awaiting_ready());
    assert!(!recovery.can_restart(3));
    assert!(!recovery.can_rebind(fresh(), mask, 0));
    assert!(recovery.rebind_sent(fresh(), mask, 0).is_err());
    assert!(recovery.installed(fresh(), 3).is_err());
    for incomplete in [0, 1, 2, 4, 7, u8::MAX] {
        assert!(!recovery.complete(incomplete));
    }
    assert!(recovery.complete(3));
}

#[test]
fn old_or_misbound_ready_messages_cannot_supply_fresh_service_bits() {
    for (role, index, bit) in [(1, 3, 8), (2, 4, 16), (3, 2, 4)] {
        let bytes = ready(fresh(), role);
        assert_eq!(ready_bit(fresh(), Some(index), READY, &bytes), Some(bit));
        assert_eq!(
            ready_bit(fresh(), Some(index), READY, &ready(old(), role)),
            None
        );
        assert_eq!(ready_bit(fresh(), Some(index), READY ^ 1, &bytes), None);
        for wrong_sender in [None, Some(0), Some(1), Some(2), Some(3), Some(4), Some(5)] {
            if wrong_sender != Some(index) {
                assert_eq!(ready_bit(fresh(), wrong_sender, READY, &bytes), None);
            }
        }
        for length in [0, 4, 16, 24, 31] {
            assert_eq!(
                ready_bit(fresh(), Some(index), READY, &bytes[..length]),
                None
            );
        }
        let mut long = bytes.to_vec();
        long.push(0);
        assert_eq!(ready_bit(fresh(), Some(index), READY, &long), None);
        for (at, replacement) in [(0, b'X'), (4, 2), (6, 0), (6, 4)] {
            let mut wrong = bytes;
            wrong[at] = replacement;
            assert_eq!(ready_bit(fresh(), Some(index), READY, &wrong), None);
        }
        for at in [8, 16] {
            let mut wrong = bytes;
            wrong[at..at + 8].copy_from_slice(&0u64.to_le_bytes());
            assert_eq!(ready_bit(fresh(), Some(index), READY, &wrong), None);
        }
        let mut wrong_epoch = bytes;
        let stale_epoch = if role == 3 {
            old().surface_epoch
        } else {
            old().input_epoch
        };
        wrong_epoch[8..16].copy_from_slice(&stale_epoch.to_le_bytes());
        assert_eq!(ready_bit(fresh(), Some(index), READY, &wrong_epoch), None);
        let mut wrong_generation = bytes;
        wrong_generation[16..24].copy_from_slice(&old().generation.to_le_bytes());
        assert_eq!(
            ready_bit(fresh(), Some(index), READY, &wrong_generation),
            None
        );
        let mut wrong = bytes;
        wrong[24..].copy_from_slice(&u64::from(role == 2).to_le_bytes());
        assert_eq!(ready_bit(fresh(), Some(index), READY, &wrong), None);
    }
}

#[test]
fn ready_validator_rejects_invalid_current_group_even_with_matching_payload() {
    for current in [
        Group {
            generation: 0,
            ..fresh()
        },
        Group {
            surface_epoch: 0,
            ..fresh()
        },
        Group {
            input_epoch: 0,
            ..fresh()
        },
    ] {
        for (role, index) in [(1, 3), (2, 4), (3, 2)] {
            assert_eq!(
                ready_bit(current, Some(index), READY, &ready(current, role)),
                None
            );
        }
    }
}

#[test]
fn another_recovery_starts_at_pause_and_rejects_every_previous_ready_generation() {
    let mut previous = Recovery::begin(old()).unwrap();
    previous.installed(fresh(), 3).unwrap();
    previous.rebind_sent(fresh(), 31, 0).unwrap();
    assert!(previous.complete(3));
    let current = Group {
        generation: 3,
        surface_epoch: 103,
        input_epoch: 203,
    };
    let mut again = Recovery::begin(fresh()).unwrap();
    assert!(again.awaiting_pause());
    assert!(!again.complete(3));
    assert!(!again.can_rebind(fresh(), 31, 0));
    assert!(!again.can_rebind(current, 31, 0));
    again.installed(current, 3).unwrap();
    for (role, index) in [(1, 3), (2, 4), (3, 2)] {
        for stale in [old(), fresh()] {
            assert_eq!(
                ready_bit(current, Some(index), READY, &ready(stale, role)),
                None
            );
        }
    }
    assert!(!again.can_rebind(current, 3, 0));
    assert!(again.can_rebind(current, 31, 0));
}

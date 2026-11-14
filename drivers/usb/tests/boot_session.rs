use nieuw_os_usb_core::{
    boot::{self, KeyboardChanges, MouseChanges},
    boot_interface::Speed,
    boot_session::{Changes, CompletionStatus::*, Error, Lifetime, Session},
};

fn configuration(protocol: u8, interface: u8, packet: u8) -> [u8; 34] {
    [
        9, 2, 34, 0, 1, 1, 0, 0x80, 50, 9, 4, interface, 0, 1, 3, 1, protocol, 0, 9, 0x21, 0x11, 1,
        0, 1, 0x22, 63, 0, 7, 5, 0x81, 3, packet, 0, 10,
    ]
}

fn session(protocol: u8, device: u64, generation: u64, interface: u8, packet: u8) -> Session {
    Session::new(
        Lifetime { device, generation },
        &configuration(protocol, interface, packet),
        interface,
        Speed::Full,
    )
    .unwrap()
}

fn ready(session: &mut Session) {
    let control = session.begin_protocol().unwrap();
    session
        .complete_protocol(control.token, Success, 0)
        .unwrap();
}

fn keyboard(changes: Changes) -> KeyboardChanges {
    match changes {
        Changes::Keyboard(changes) => changes,
        Changes::Mouse(_) => panic!("expected keyboard changes"),
    }
}

fn mouse(changes: Changes) -> MouseChanges {
    match changes {
        Changes::Mouse(changes) => changes,
        Changes::Keyboard(_) => panic!("expected mouse changes"),
    }
}

const HELD: [u8; 8] = [3, 0, 4, 5, 0, 0, 0, 0];

#[test]
fn only_a_matching_successful_zero_data_protocol_completion_enables_input() {
    let mut session = session(1, 9, 1, 7, 8);
    assert_eq!(session.begin_input(), Err(Error::NotReady));
    let control = session.begin_protocol().unwrap();
    assert_eq!(control.request.setup(), [0x21, 0x0b, 0, 0, 7, 0, 0, 0]);
    assert_eq!(session.begin_protocol(), Err(Error::Pending));
    assert_eq!(session.begin_input(), Err(Error::Pending));
    assert_eq!(
        session.complete_protocol(control.token, Failed, 0),
        Err(Error::TransferFailed)
    );
    assert_eq!(session.begin_input(), Err(Error::NotReady));
    assert_eq!(
        session.complete_protocol(control.token, Success, 0),
        Err(Error::UnexpectedCompletion)
    );
    for length in [1, 8, usize::MAX] {
        let control = session.begin_protocol().unwrap();
        assert_eq!(
            session.complete_protocol(control.token, Success, length),
            Err(Error::Length)
        );
        assert_eq!(session.begin_input(), Err(Error::NotReady));
    }
    ready(&mut session);
    assert_eq!(session.begin_protocol(), Err(Error::AlreadyReady));
    let input = session.begin_input().unwrap();
    assert_eq!((input.endpoint, input.max_length), (0x81, 8));
    assert_eq!(session.begin_input(), Err(Error::Pending));
}

#[test]
fn keyboard_transitions_and_reset_release_the_selected_state_once() {
    let mut session = session(1, 9, 1, 0, 8);
    ready(&mut session);
    let first = session.begin_input().unwrap();
    let changes = keyboard(session.complete_input(first.token, Success, &HELD).unwrap());
    assert_eq!(changes.pressed(), &[4, 5]);
    assert_eq!(changes.modifiers_pressed, 3);
    assert_eq!(
        session.complete_input(first.token, Success, &HELD),
        Err(Error::UnexpectedCompletion)
    );
    let second = session.begin_input().unwrap();
    let changes = keyboard(
        session
            .complete_input(second.token, Success, &[2, 0, 5, 6, 0, 0, 0, 0])
            .unwrap(),
    );
    assert_eq!(changes.pressed(), &[6]);
    assert_eq!(changes.released(), &[4]);
    assert_eq!(changes.modifiers_released, 1);
    let changes = keyboard(session.reset().unwrap());
    assert_eq!(changes.released(), &[5, 6]);
    assert_eq!(changes.modifiers_released, 2);
    assert_eq!(
        keyboard(session.reset().unwrap()),
        KeyboardChanges::default()
    );
    assert_eq!(session.begin_input(), Err(Error::NotReady));
    ready(&mut session);
    let input = session.begin_input().unwrap();
    assert_eq!(
        keyboard(session.complete_input(input.token, Success, &HELD).unwrap()).pressed(),
        &[4, 5]
    );
}

#[test]
fn mouse_oem_bytes_are_bounded_and_detach_releases_buttons_without_movement() {
    let mut session = session(2, 9, 1, 0, 4);
    ready(&mut session);
    let input = session.begin_input().unwrap();
    assert_eq!(input.max_length, 4);
    assert_eq!(
        mouse(
            session
                .complete_input(input.token, Success, &[0x81, 5, 0xfd, 0xff])
                .unwrap()
        ),
        MouseChanges {
            buttons_pressed: 1,
            dx: 5,
            dy: -3,
            ..MouseChanges::default()
        }
    );
    let input = session.begin_input().unwrap();
    assert_eq!(
        mouse(
            session
                .complete_input(input.token, Success, &[6, 0, 0])
                .unwrap()
        ),
        MouseChanges {
            buttons_pressed: 6,
            buttons_released: 1,
            ..MouseChanges::default()
        }
    );
    let pending = session.begin_input().unwrap();
    assert_eq!(
        mouse(session.detach()),
        MouseChanges {
            buttons_released: 6,
            ..MouseChanges::default()
        }
    );
    assert_eq!(mouse(session.detach()), MouseChanges::default());
    assert_eq!(
        session.complete_input(pending.token, Success, &[7, 1, 1]),
        Err(Error::UnexpectedCompletion)
    );
    assert_eq!(session.reset(), Err(Error::Detached));
    assert_eq!(session.begin_input(), Err(Error::Detached));
    assert_eq!(session.begin_protocol(), Err(Error::Detached));
}

#[test]
fn wrong_device_generation_interface_and_kind_cannot_consume_pending_work() {
    let mut current = session(1, 9, 1, 0, 8);
    let control = current.begin_protocol().unwrap();
    for (device, generation, interface) in [(10, 1, 0), (9, 2, 0), (9, 1, 1)] {
        let mut other = session(1, device, generation, interface, 8);
        let foreign = other.begin_protocol().unwrap();
        assert_eq!(
            current.complete_protocol(foreign.token, Success, 0),
            Err(Error::UnexpectedCompletion)
        );
    }
    current
        .complete_protocol(control.token, Success, 0)
        .unwrap();
    let input = current.begin_input().unwrap();
    assert_eq!(
        current.complete_input(control.token, Success, &HELD),
        Err(Error::UnexpectedCompletion)
    );
    assert_eq!(
        current.complete_protocol(input.token, Success, 0),
        Err(Error::UnexpectedCompletion)
    );
    assert_eq!(
        keyboard(current.complete_input(input.token, Success, &HELD).unwrap()).pressed(),
        &[4, 5]
    );
}

#[test]
fn reset_during_control_rejects_the_old_completion_without_losing_the_new_one() {
    let mut session = session(1, 9, 1, 0, 8);
    let old = session.begin_protocol().unwrap();
    session.reset().unwrap();
    let new = session.begin_protocol().unwrap();
    assert_ne!(old.token, new.token);
    assert_eq!(
        session.complete_protocol(old.token, Success, 0),
        Err(Error::UnexpectedCompletion)
    );
    session.complete_protocol(new.token, Success, 0).unwrap();
    assert!(session.begin_input().is_ok());
}

#[test]
fn reset_with_held_keys_and_pending_input_cannot_revive_old_input() {
    let mut session = session(1, 9, 1, 0, 8);
    ready(&mut session);
    let input = session.begin_input().unwrap();
    session.complete_input(input.token, Success, &HELD).unwrap();
    let late = session.begin_input().unwrap();
    let released = keyboard(session.reset().unwrap());
    assert_eq!(released.released(), &[4, 5]);
    assert_eq!(released.modifiers_released, 3);
    let protocol = session.begin_protocol().unwrap();
    assert_eq!(
        session.complete_input(late.token, Success, &HELD),
        Err(Error::UnexpectedCompletion)
    );
    session
        .complete_protocol(protocol.token, Success, 0)
        .unwrap();
    let new = session.begin_input().unwrap();
    assert_eq!(
        session.complete_input(late.token, Success, &HELD),
        Err(Error::UnexpectedCompletion)
    );
    assert_eq!(
        keyboard(session.complete_input(new.token, Success, &[0; 8]).unwrap()),
        KeyboardChanges::default()
    );
}

#[test]
fn reattaching_at_the_same_device_identity_requires_a_new_generation() {
    let mut old = session(1, 9, 1, 0, 8);
    let old_control = old.begin_protocol().unwrap();
    old.complete_protocol(old_control.token, Success, 0)
        .unwrap();
    let old_input = old.begin_input().unwrap();
    old.detach();
    let mut new = session(1, 9, 2, 0, 8);
    let new_control = new.begin_protocol().unwrap();
    assert_eq!(
        new.complete_protocol(old_control.token, Success, 0),
        Err(Error::UnexpectedCompletion)
    );
    new.complete_protocol(new_control.token, Success, 0)
        .unwrap();
    let new_input = new.begin_input().unwrap();
    assert_eq!(
        new.complete_input(old_input.token, Success, &HELD),
        Err(Error::UnexpectedCompletion)
    );
    assert_eq!(
        keyboard(
            new.complete_input(new_input.token, Success, &[0; 8])
                .unwrap()
        ),
        KeyboardChanges::default()
    );
}

#[test]
fn detach_during_protocol_is_terminal_even_after_a_late_success() {
    let mut session = session(1, 9, 1, 0, 8);
    let control = session.begin_protocol().unwrap();
    assert_eq!(keyboard(session.detach()), KeyboardChanges::default());
    assert_eq!(
        session.complete_protocol(control.token, Success, 0),
        Err(Error::UnexpectedCompletion)
    );
    assert_eq!(session.begin_input(), Err(Error::Detached));
    assert_eq!(session.reset(), Err(Error::Detached));
}

#[test]
fn matching_failed_or_malformed_reports_are_consumed_without_mutating_held_keys() {
    let mut session = session(1, 9, 1, 0, 8);
    ready(&mut session);
    let input = session.begin_input().unwrap();
    session.complete_input(input.token, Success, &HELD).unwrap();
    for (status, bytes, error) in [
        (Failed, &[0; 8][..], Error::TransferFailed),
        (Success, &[][..], Error::Report(boot::Error::Length)),
        (Success, &[0; 7][..], Error::Report(boot::Error::Length)),
        (Success, &[0; 9][..], Error::Length),
        (
            Success,
            &[0, 0, 4, 4, 0, 0, 0, 0][..],
            Error::Report(boot::Error::DuplicateKey(4)),
        ),
    ] {
        let invalid = session.begin_input().unwrap();
        assert_eq!(
            session.complete_input(invalid.token, status, bytes),
            Err(error)
        );
        let next = session.begin_input().unwrap();
        assert_eq!(
            session.complete_input(invalid.token, Success, &[0; 8]),
            Err(Error::UnexpectedCompletion)
        );
        assert_eq!(
            keyboard(session.complete_input(next.token, Success, &HELD).unwrap()),
            KeyboardChanges::default()
        );
    }
    let released = keyboard(session.detach());
    assert_eq!(released.released(), &[4, 5]);
    assert_eq!(released.modifiers_released, 3);
    assert_eq!(keyboard(session.detach()), KeyboardChanges::default());
}

#[test]
fn mouse_actual_length_cannot_exceed_its_endpoint_even_with_valid_boot_bytes() {
    let mut session = session(2, 9, 1, 0, 3);
    ready(&mut session);
    let input = session.begin_input().unwrap();
    session
        .complete_input(input.token, Success, &[1, 0, 0])
        .unwrap();
    for bytes in [&[0; 2][..], &[0; 4][..], &[0, 128, 0][..]] {
        let input = session.begin_input().unwrap();
        assert!(session.complete_input(input.token, Success, bytes).is_err());
    }
    assert_eq!(mouse(session.detach()).buttons_released, 1);
}

#[test]
fn construction_reuses_descriptor_validation_and_bounds_the_single_packet_profile() {
    let lifetime = Lifetime {
        device: 9,
        generation: 1,
    };
    for (protocol, minimum) in [(1, 8), (2, 3)] {
        for packet in 1..=64 {
            let mut bytes = configuration(protocol, 0, packet);
            bytes[25..27].copy_from_slice(&u16::MAX.to_le_bytes());
            let result = Session::new(lifetime, &bytes, 0, Speed::Full);
            if packet < minimum {
                assert!(matches!(result, Err(Error::PacketTooSmall)));
            } else {
                let mut session = result.unwrap();
                ready(&mut session);
                assert_eq!(
                    session.begin_input().unwrap().max_length,
                    usize::from(packet.min(8))
                );
            }
        }
    }
    let mut malformed = configuration(1, 0, 8);
    malformed[19] = 0xff; // A framed descriptor is not a HID descriptor.
    assert!(matches!(
        Session::new(lifetime, &malformed, 0, Speed::Full),
        Err(Error::Binding(_))
    ));
    assert!(matches!(
        Session::new(lifetime, &configuration(1, 0, 9), 0, Speed::Low),
        Err(Error::Binding(_))
    ));
}

#[test]
fn an_independent_session_can_complete_while_another_is_pending_or_reset() {
    let mut first = session(1, 9, 1, 0, 8);
    let mut second = session(2, 10, 1, 0, 8);
    let pending = first.begin_protocol().unwrap();
    ready(&mut second);
    let input = second.begin_input().unwrap();
    first.reset().unwrap();
    assert_eq!(
        first.complete_protocol(pending.token, Success, 0),
        Err(Error::UnexpectedCompletion)
    );
    assert_eq!(
        mouse(
            second
                .complete_input(input.token, Success, &[1, 2, 3])
                .unwrap()
        )
        .buttons_pressed,
        1
    );
    ready(&mut first);
    assert!(first.begin_input().is_ok());
    assert!(second.begin_input().is_ok());
}

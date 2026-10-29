//! Public-API regressions for controller resets while physical keys remain held.
use ui_core::{FocusOwner, InputRouter, KeyboardEvent, Routed, Set1Keyboard};

fn key(
    router: &mut InputRouter,
    keyboard: &mut Set1Keyboard,
    byte: u8,
) -> Option<Routed<KeyboardEvent>> {
    keyboard
        .feed(byte)
        .unwrap()
        .and_then(|event| router.route_key(event))
}

#[test]
fn held_without_focus_cannot_become_new_press_after_decoder_reset() {
    let mut router = InputRouter::new();
    let mut keyboard = Set1Keyboard::new();
    assert!(key(&mut router, &mut keyboard, 0x1e).is_none());
    router.set_focus(FocusOwner::new(1)).unwrap();
    router.reset_input().unwrap();
    keyboard.reset();
    // This is still a typematic make from the physical key pressed with no
    // target. Decoder reset must not let it acquire the current focus owner.
    assert!(key(&mut router, &mut keyboard, 0x1e).is_none());
    assert!(key(&mut router, &mut keyboard, 0x9e).is_none());
    assert!(key(&mut router, &mut keyboard, 0x1e).is_some());
}

#[test]
fn release_directly_after_decoder_reset_allows_first_fresh_press() {
    let mut router = InputRouter::new();
    let mut keyboard = Set1Keyboard::new();
    router.set_focus(FocusOwner::new(1)).unwrap();
    let old = key(&mut router, &mut keyboard, 0x1e).unwrap();
    router.reset_input().unwrap();
    keyboard.reset();
    assert!(!router.is_current(old.token()));
    // There is no intervening repeat to teach the reset decoder that A is
    // still held. Its real break must nevertheless clear the router marker.
    assert!(key(&mut router, &mut keyboard, 0x9e).is_none());
    let fresh = key(&mut router, &mut keyboard, 0x1e)
        .expect("first new physical press after a real release must work");
    assert!(router.is_current(fresh.token()));
}

#[test]
fn nofocus_modifier_repeat_cannot_create_chord_after_reset() {
    let mut router = InputRouter::new();
    let mut keyboard = Set1Keyboard::new();
    assert!(key(&mut router, &mut keyboard, 0x1d).is_none());
    router.set_focus(FocusOwner::new(1)).unwrap();
    router.reset_input().unwrap();
    keyboard.reset();
    assert!(key(&mut router, &mut keyboard, 0x1d).is_none());
    assert!(keyboard.modifiers().control());
    let typed = key(&mut router, &mut keyboard, 0x1e).unwrap();
    assert!(!typed.event().modifiers.control());
}

#[test]
fn nofocus_break_after_reset_clears_physical_marker_without_repeat() {
    let mut router = InputRouter::new();
    let mut keyboard = Set1Keyboard::new();
    assert!(key(&mut router, &mut keyboard, 0x1d).is_none());
    router.set_focus(FocusOwner::new(1)).unwrap();
    router.reset_input().unwrap();
    keyboard.reset();
    assert!(key(&mut router, &mut keyboard, 0x9d).is_none());
    let fresh = key(&mut router, &mut keyboard, 0x1d).unwrap();
    assert!(fresh.event().modifiers.control());
}

#[test]
fn orphan_breaks_and_pause_pulses_never_create_held_ownership() {
    let mut router = InputRouter::new();
    let mut keyboard = Set1Keyboard::new();
    router.set_focus(FocusOwner::new(1)).unwrap();
    assert!(key(&mut router, &mut keyboard, 0x9e).is_none());
    assert!(key(&mut router, &mut keyboard, 0x9d).is_none());
    for _ in 0..2 {
        for byte in [0xe1, 0x1d, 0x45, 0xe1, 0x9d] {
            assert!(key(&mut router, &mut keyboard, byte).is_none());
        }
        let pause = key(&mut router, &mut keyboard, 0xc5).unwrap();
        assert_eq!(pause.event().action, ui_core::KeyAction::Pulse);
    }
    assert!(!key(&mut router, &mut keyboard, 0x1e)
        .unwrap()
        .event()
        .modifiers
        .control());
}

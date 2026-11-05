//! Host routing regression, bound to the native main-loop wiring below.
use nieuw_os_window_session::{
    clipboard::{ClearAction, ClearChord},
    scene::{Damage, Scene},
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

fn assert_product_wiring() {
    let main = include_str!("../src/main.rs");
    let expected = r#"
                let clear = match event {
                    Event::Key(key) if !palette_key || key.action == KeyAction::Release => {
                        clear_chord.key(key)
                    }
                    _ => ClearAction::Pass,
                };
                if !effect.consumed {
                    match event {
                        Event::Key(_) if clear != ClearAction::Pass => {
                            if clear == ClearAction::Clear {
                                clipboard.clear(&queues).unwrap_or_else(|_| fail(39));
                            }
                        }
"#;
    assert!(
        main.contains(expected.trim_matches('\n')),
        "native main must drain physical releases and keep modal dispatch before clipboard clear"
    );
    assert!(main.contains("scene\n                            .switcher_key(key, &mut damage)"));
}

fn route(
    scene: &mut Scene,
    damage: &mut Damage,
    clear: &mut ClearChord,
    event: KeyboardEvent,
) -> ClearAction {
    let owned =
        scene.switcher_key(event, damage).unwrap() || scene.palette_key(event, damage).unwrap();
    let action = if !owned || event.action == KeyAction::Release {
        clear.key(event)
    } else {
        ClearAction::Pass
    };
    if owned {
        ClearAction::Consumed
    } else {
        action
    }
}

#[test]
fn switcher_release_finishes_existing_clear_without_authorizing_a_new_clear() {
    assert_product_wiring();
    for existing_clear in [false, true] {
        let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
        let mut damage = Damage::new(1280, 720).unwrap();
        let mut clear = ClearChord::default();
        let normal = KeyboardModifiers::default();
        let alt = KeyboardModifiers {
            left_alt: true,
            ..normal
        };
        let chord = KeyboardModifiers {
            left_control: true,
            left_shift: true,
            ..normal
        };
        let mut send = |key, action, modifiers| {
            route(
                &mut scene,
                &mut damage,
                &mut clear,
                KeyboardEvent {
                    key,
                    action,
                    modifiers,
                },
            )
        };
        if existing_clear {
            assert_eq!(
                send(PhysicalKey::Delete, KeyAction::Press, chord),
                ClearAction::Clear
            );
        }
        assert_eq!(
            send(PhysicalKey::Tab, KeyAction::Press, alt),
            ClearAction::Consumed
        );
        for action in [KeyAction::Press, KeyAction::Repeat, KeyAction::Release] {
            assert_eq!(
                send(PhysicalKey::Delete, action, chord),
                ClearAction::Consumed,
                "modal input must not execute a clipboard clear"
            );
        }
        send(PhysicalKey::LeftAlt, KeyAction::Release, normal);
        assert_eq!(
            send(PhysicalKey::Delete, KeyAction::Press, normal),
            ClearAction::Pass,
            "ordinary Delete must not inherit the previous clear gesture"
        );
        assert_eq!(
            send(PhysicalKey::Delete, KeyAction::Release, normal),
            ClearAction::Pass
        );
        assert_eq!(
            send(PhysicalKey::Delete, KeyAction::Press, chord),
            ClearAction::Clear,
            "a fresh non-modal clear remains available"
        );
    }
}

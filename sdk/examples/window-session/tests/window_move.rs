//! Host contracts; native runtime acceptance is a separate gate.
use nieuw_os_graphics::Rect;
use nieuw_os_window_session::scene::{Damage, Invalid, Scene, MAX_PIXELS};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

const DISPLAYS: [(u32, u32, u32); 5] = [
    (800, 500, 1),
    (1280, 720, 1),
    (1920, 1080, 1),
    (1600, 1000, 2),
    (1920, 1080, 2),
];

fn event(key: PhysicalKey, action: KeyAction, alt: bool, shift: bool) -> KeyboardEvent {
    KeyboardEvent {
        key,
        action,
        modifiers: KeyboardModifiers {
            left_alt: alt,
            left_shift: shift,
            ..Default::default()
        },
    }
}
fn tap(scene: &mut Scene, damage: &mut Damage, key: PhysicalKey, alt: bool, shift: bool) {
    for action in [KeyAction::Press, KeyAction::Release] {
        assert!(scene
            .palette_key(event(key, action, alt, shift), damage)
            .unwrap());
    }
}
fn repaint(scene: &Scene, damage: &mut Damage, screen: &mut [u32], backing: &[u32]) {
    while let Some(tile) = damage.next_tile() {
        let mut pixels = [0; 256];
        scene.render(tile, &mut pixels, backing).unwrap();
        for row in 0..tile.height as usize {
            let start = (tile.y as usize + row) * scene.width as usize + tile.x as usize;
            screen[start..start + tile.width as usize].copy_from_slice(
                &pixels[row * tile.width as usize..(row + 1) * tile.width as usize],
            );
        }
    }
    let mut reference = vec![0; screen.len()];
    scene
        .render_reference(
            Rect::new(0, 0, scene.width, scene.height),
            &mut reference,
            backing,
        )
        .unwrap();
    assert_eq!(
        screen, reference,
        "Damage and optimized raster must match every reference pixel"
    );
}

#[test]
fn keyboard_move_preserves_buffers_contracts_and_repaints_old_new_and_cancelled_frames() {
    for (w, h, s) in DISPLAYS {
        let mut scene = Scene::new(w, h, s, [17, 29]).unwrap();
        let contracts = scene.windows.map(|window| (window.geometry, window.offset));
        let origin = scene.windows[0].body;
        let mut damage = Damage::new(w, h).unwrap();
        let mut backing = vec![0xabcdef; MAX_PIXELS];
        backing[..scene.windows[1].offset].fill(0xc0ffee);
        let mut screen = vec![0; (w * h) as usize];
        damage.mark(Rect::new(0, 0, w, h));
        repaint(&scene, &mut damage, &mut screen, &backing);
        let original = screen.clone();
        tap(&mut scene, &mut damage, PhysicalKey::F7, true, false);
        assert!(scene.keyboard_move_is_active() && scene.overlay_is_open());
        assert!(!scene.palette_open());
        assert_eq!(scene.cycle(), Err(Invalid));
        assert_eq!(scene.minimize(0, &mut damage), Err(Invalid));
        repaint(&scene, &mut damage, &mut screen, &backing);
        for (key, shift) in [(PhysicalKey::Right, false), (PhysicalKey::Up, true)] {
            tap(&mut scene, &mut damage, key, false, shift);
            repaint(&scene, &mut damage, &mut screen, &backing);
        }
        assert_eq!(scene.windows[0].body.x, origin.x + 16 * s as i32);
        assert_eq!(scene.windows[0].body.y, origin.y - s as i32);
        assert_eq!(scene.moves, 2);
        for key in [
            PhysicalKey::A,
            PhysicalKey::F6,
            PhysicalKey::F8,
            PhysicalKey::F9,
        ] {
            tap(&mut scene, &mut damage, key, false, false);
        }
        assert_eq!(scene.focused, 0);
        assert_eq!(scene.moves, 2);
        tap(&mut scene, &mut damage, PhysicalKey::Escape, false, false);
        repaint(&scene, &mut damage, &mut screen, &backing);
        assert_eq!(screen, original);
        assert!(!scene.overlay_is_open());
        assert_eq!(scene.windows[0].body, origin);
        tap(&mut scene, &mut damage, PhysicalKey::F7, true, false);
        tap(&mut scene, &mut damage, PhysicalKey::Right, false, false);
        tap(&mut scene, &mut damage, PhysicalKey::Enter, false, false);
        repaint(&scene, &mut damage, &mut screen, &backing);
        assert!(!scene.overlay_is_open());
        assert_eq!(scene.windows[0].body.x, origin.x + 16 * s as i32);
        assert_eq!(
            scene.windows.map(|window| (window.geometry, window.offset)),
            contracts
        );
        assert!(backing[..scene.windows[1].offset]
            .iter()
            .all(|pixel| *pixel == 0xc0ffee));
        assert!(backing[scene.windows[1].offset..]
            .iter()
            .all(|pixel| *pixel == 0xabcdef));
    }
}

#[test]
fn movement_is_bounded_repeatable_and_counter_failure_is_atomic() {
    for (w, h, s) in DISPLAYS {
        let mut scene = Scene::new(w, h, s, [1, 2]).unwrap();
        let origin = scene.windows[0].body;
        let mut damage = Damage::new(w, h).unwrap();
        tap(&mut scene, &mut damage, PhysicalKey::F7, true, false);
        for key in [
            PhysicalKey::Left,
            PhysicalKey::Up,
            PhysicalKey::Right,
            PhysicalKey::Down,
        ] {
            for _ in 0..200 {
                assert!(scene
                    .palette_key(event(key, KeyAction::Repeat, false, false), &mut damage)
                    .unwrap());
            }
            let b = scene.windows[0].body;
            assert!(b.x >= 0 && b.x + b.width as i32 <= w as i32);
            assert!(b.y >= 94 * s as i32 && b.y + b.height as i32 <= (h - 72 * s) as i32);
        }
        let before = scene.windows[0].body;
        scene.moves = u64::MAX;
        assert_eq!(
            scene.palette_key(
                event(PhysicalKey::Left, KeyAction::Press, false, false),
                &mut damage
            ),
            Err(Invalid)
        );
        assert_eq!(scene.windows[0].body, before);
        tap(&mut scene, &mut damage, PhysicalKey::Escape, false, false);
        assert_eq!(
            scene.windows[0].body, origin,
            "Rollback must work even when counters are exhausted"
        );
    }
}

#[test]
fn input_loss_and_pointer_click_cancel_without_clickthrough_or_stale_key_delivery() {
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    let mut damage = Damage::new(1280, 720).unwrap();
    let origin = scene.windows[0].body;
    for pointer in [false, true] {
        tap(&mut scene, &mut damage, PhysicalKey::F7, true, false);
        tap(&mut scene, &mut damage, PhysicalKey::Right, false, false);
        scene
            .palette_key(
                event(PhysicalKey::A, KeyAction::Press, false, false),
                &mut damage,
            )
            .unwrap();
        if pointer {
            assert_eq!(
                scene.mouse_with_damage(100, 200, true, true, &mut damage),
                Ok(None)
            );
            assert_eq!(
                scene.mouse_with_damage(100, 200, false, false, &mut damage),
                Ok(None)
            );
            assert_eq!(
                scene.mouse_with_damage(100, 200, false, false, &mut damage),
                Ok(Some(0))
            );
        } else {
            scene.cancel_palette_input(&mut damage);
        }
        assert_eq!(scene.windows[0].body, origin);
        assert!(!scene.keyboard_move_is_active());
        for action in [KeyAction::Repeat, KeyAction::Release] {
            assert!(scene
                .palette_key(event(PhysicalKey::A, action, false, false), &mut damage)
                .unwrap());
        }
        assert!(!scene
            .palette_key(
                event(PhysicalKey::A, KeyAction::Press, false, false),
                &mut damage
            )
            .unwrap());
    }
}

#[test]
fn menu_shortcut_is_discoverable_and_hidden_windows_cannot_start_movement() {
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    let mut damage = Damage::new(1280, 720).unwrap();
    tap(&mut scene, &mut damage, PhysicalKey::Space, true, false);
    assert!(scene.window_menu_is_open());
    tap(&mut scene, &mut damage, PhysicalKey::F7, true, false);
    assert!(scene.keyboard_move_is_active());
    assert!(!scene.window_menu_is_open());
    tap(&mut scene, &mut damage, PhysicalKey::Escape, false, false);
    scene.minimize(0, &mut damage).unwrap();
    scene.minimize(1, &mut damage).unwrap();
    tap(&mut scene, &mut damage, PhysicalKey::F7, true, false);
    assert!(!scene.keyboard_move_is_active());
}

#[test]
fn shortcut_rejects_extra_modifiers_and_repeat_cannot_restart_the_origin() {
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    let origin = scene.windows[0].body;
    let mut damage = Damage::new(1280, 720).unwrap();
    for modifier in 0..3 {
        let mut key = event(PhysicalKey::F7, KeyAction::Press, true, false);
        match modifier {
            0 => key.modifiers.left_control = true,
            1 => key.modifiers.left_shift = true,
            _ => key.modifiers.left_super = true,
        }
        assert!(!scene.palette_key(key, &mut damage).unwrap());
        assert!(!scene.keyboard_move_is_active());
    }
    tap(&mut scene, &mut damage, PhysicalKey::F7, true, false);
    tap(&mut scene, &mut damage, PhysicalKey::Right, false, false);
    for action in [KeyAction::Repeat, KeyAction::Press, KeyAction::Release] {
        assert!(scene
            .palette_key(event(PhysicalKey::F7, action, true, false), &mut damage)
            .unwrap());
    }
    tap(&mut scene, &mut damage, PhysicalKey::Escape, false, false);
    assert_eq!(scene.windows[0].body, origin);
}

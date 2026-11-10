//! Real scene pixels and input policy on the host; not native execution.
use nieuw_os_graphics::{Font, Rect, Size, Weight};
use nieuw_os_window_session::{
    clipboard::Notice,
    scene::{Damage, Scene, MAX_PIXELS},
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

fn tap(scene: &mut Scene, damage: &mut Damage, key: PhysicalKey, alt: bool) {
    for action in [KeyAction::Press, KeyAction::Release] {
        assert!(scene
            .palette_key(
                KeyboardEvent {
                    key,
                    action,
                    modifiers: KeyboardModifiers {
                        left_alt: alt,
                        ..Default::default()
                    },
                },
                damage
            )
            .unwrap());
    }
}
fn click(scene: &mut Scene, damage: &mut Damage, r: Rect) {
    for (down, held) in [(true, true), (false, false)] {
        assert_eq!(
            scene.mouse_with_damage((r.x + 2) as u32, (r.y + 2) as u32, down, held, damage),
            Ok(None)
        );
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
        "Panel changes must damage every composited pixel"
    );
}

#[test]
fn all_real_notice_states_keyboard_focus_and_dismissal_repaint_every_pixel() {
    for (w, h, s) in [
        (800, 500, 1),
        (1280, 720, 1),
        (1920, 1080, 1),
        (1600, 1000, 2),
        (1920, 1080, 2),
    ] {
        let mut scene = Scene::new(w, h, s, [17, 29]).unwrap();
        let contracts = scene
            .windows
            .map(|window| (window.body, window.geometry, window.offset));
        let mut damage = Damage::new(w, h).unwrap();
        let mut backing = vec![0xabcdef; MAX_PIXELS];
        backing[..scene.windows[1].offset].fill(0xc0ffee);
        let mut screen = vec![0; (w * h) as usize];
        damage.mark(Rect::new(0, 0, w, h));
        repaint(&scene, &mut damage, &mut screen, &backing);
        let original = screen.clone();
        tap(&mut scene, &mut damage, PhysicalKey::N, true);
        assert!(scene.notifications_are_open() && !scene.notification_is_present());
        repaint(&scene, &mut damage, &mut screen, &backing);
        tap(&mut scene, &mut damage, PhysicalKey::Enter, false);
        repaint(&scene, &mut damage, &mut screen, &backing);
        assert_eq!(screen, original);
        for notice in [
            Notice::Copied,
            Notice::Offered,
            Notice::Cleared,
            Notice::Empty,
            Notice::Denied,
        ] {
            assert!(scene.set_clipboard_notice(notice));
            damage.mark(scene.clipboard_notice_bounds());
            assert!(scene.notification_is_present());
            tap(&mut scene, &mut damage, PhysicalKey::N, true);
            assert!(!scene.palette_open());
            repaint(&scene, &mut damage, &mut screen, &backing);
            for key in [PhysicalKey::Tab, PhysicalKey::Up] {
                tap(&mut scene, &mut damage, key, false);
                repaint(&scene, &mut damage, &mut screen, &backing);
            }
            tap(&mut scene, &mut damage, PhysicalKey::Delete, false);
            assert!(!scene.notification_is_present());
            assert!(
                !scene.set_clipboard_notice(notice),
                "Polling the same status must not resurrect a dismissed message"
            );
            repaint(&scene, &mut damage, &mut screen, &backing);
            tap(&mut scene, &mut damage, PhysicalKey::Enter, false);
            repaint(&scene, &mut damage, &mut screen, &backing);
            assert!(!scene.overlay_is_open());
        }
        assert_eq!(
            scene
                .windows
                .map(|window| (window.body, window.geometry, window.offset)),
            contracts
        );
        assert_eq!((scene.focused, scene.moves, scene.switches), (0, 0, 0));
        assert!(backing[..scene.windows[1].offset]
            .iter()
            .all(|pixel| *pixel == 0xc0ffee));
    }
}

#[test]
fn mouse_affordances_and_input_cancellation_cannot_click_or_type_through() {
    for (w, h, s) in [
        (800, 500, 1),
        (1280, 720, 1),
        (1920, 1080, 1),
        (1600, 1000, 2),
        (1920, 1080, 2),
    ] {
        let mut scene = Scene::new(w, h, s, [1, 2]).unwrap();
        let mut damage = Damage::new(w, h).unwrap();
        let mut backing = vec![0xabcdef; MAX_PIXELS];
        backing[..scene.windows[1].offset].fill(0xc0ffee);
        let mut screen = vec![0; (w * h) as usize];
        scene.set_clipboard_notice(Notice::Copied);
        damage.mark(Rect::new(0, 0, w, h));
        repaint(&scene, &mut damage, &mut screen, &backing);
        let button = scene.clipboard_notice_bounds();
        click(&mut scene, &mut damage, button);
        assert!(scene.notifications_are_open());
        repaint(&scene, &mut damage, &mut screen, &backing);
        for key in [
            PhysicalKey::A,
            PhysicalKey::F6,
            PhysicalKey::F7,
            PhysicalKey::F8,
            PhysicalKey::F9,
        ] {
            tap(&mut scene, &mut damage, key, false);
        }
        assert!(!scene.keyboard_move_is_active());
        assert_eq!((scene.focused, scene.moves, scene.switches), (0, 0, 0));
        let dismiss = scene.notification_dismiss_button();
        click(&mut scene, &mut damage, dismiss);
        assert!(!scene.notification_is_present());
        repaint(&scene, &mut damage, &mut screen, &backing);
        let close = scene.notification_close_button();
        click(&mut scene, &mut damage, close);
        assert!(!scene.notifications_are_open());
        repaint(&scene, &mut damage, &mut screen, &backing);
        click(&mut scene, &mut damage, button);
        repaint(&scene, &mut damage, &mut screen, &backing);
        let (x, y) = (100 * s, 200 * s);
        assert_eq!(
            scene.mouse_with_damage(x, y, true, true, &mut damage),
            Ok(None)
        );
        assert!(!scene.notifications_are_open());
        repaint(&scene, &mut damage, &mut screen, &backing);
        assert_eq!(
            scene.mouse_with_damage(x, y, false, false, &mut damage),
            Ok(None)
        );
        repaint(&scene, &mut damage, &mut screen, &backing);
        assert_eq!(
            scene.mouse_with_damage(x, y, false, false, &mut damage),
            Ok(Some(0))
        );
        tap(&mut scene, &mut damage, PhysicalKey::N, true);
        repaint(&scene, &mut damage, &mut screen, &backing);
        assert!(scene
            .palette_key(
                KeyboardEvent {
                    key: PhysicalKey::A,
                    action: KeyAction::Press,
                    modifiers: Default::default()
                },
                &mut damage,
            )
            .unwrap());
        scene.cancel_palette_input(&mut damage);
        repaint(&scene, &mut damage, &mut screen, &backing);
        for action in [KeyAction::Repeat, KeyAction::Release] {
            assert!(scene
                .palette_key(
                    KeyboardEvent {
                        key: PhysicalKey::A,
                        action,
                        modifiers: Default::default()
                    },
                    &mut damage,
                )
                .unwrap());
        }
        assert!(!scene.notifications_are_open());
    }
}

#[test]
fn panel_controls_labels_and_modifier_scope_are_bounded_on_every_display() {
    for (w, h, s) in [
        (800, 500, 1),
        (1280, 720, 1),
        (1600, 1000, 2),
        (1920, 1080, 2),
    ] {
        let mut scene = Scene::new(w, h, s, [1, 2]).unwrap();
        let mut damage = Damage::new(w, h).unwrap();
        let r = scene.notifications_paint_bounds();
        assert!(
            r.x >= 0
                && r.y >= 0
                && r.x + r.width as i32 <= w as i32
                && r.y + r.height as i32 <= h as i32
        );
        let small = Font::new(Size::Small, Weight::Regular, s == 2);
        assert!(small.measure("Tab kiezen · Delete verwijderen · Esc terug") <= 392 * s);
        for shift in [false, true] {
            let key = KeyboardEvent {
                key: PhysicalKey::N,
                action: KeyAction::Press,
                modifiers: KeyboardModifiers {
                    left_alt: true,
                    left_control: !shift,
                    left_shift: shift,
                    ..Default::default()
                },
            };
            assert!(!scene.palette_key(key, &mut damage).unwrap());
            assert!(!scene.notifications_are_open());
        }
        tap(&mut scene, &mut damage, PhysicalKey::F7, true);
        tap(&mut scene, &mut damage, PhysicalKey::N, true);
        assert!(scene.keyboard_move_is_active() && !scene.notifications_are_open());
        tap(&mut scene, &mut damage, PhysicalKey::Escape, false);
    }
}

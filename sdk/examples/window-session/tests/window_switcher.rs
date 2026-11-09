//! Shell input and whole-frame damage contracts; these are host tests.
use nieuw_os_graphics::Rect;
use nieuw_os_window_session::{
    scene::{Damage, Scene, MAX_PIXELS},
    shell::Page,
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey, Set1Keyboard};

const DISPLAYS: [(u32, u32, u32); 5] = [
    (800, 500, 1),
    (1280, 720, 1),
    (1920, 1080, 1),
    (1600, 1000, 2),
    (1920, 1080, 2),
];

fn key(physical: PhysicalKey, action: KeyAction, alt: bool) -> KeyboardEvent {
    KeyboardEvent {
        key: physical,
        action,
        modifiers: KeyboardModifiers {
            left_alt: alt,
            ..KeyboardModifiers::default()
        },
    }
}

fn route(scene: &mut Scene, damage: &mut Damage, byte: u8, keyboard: &mut Set1Keyboard) -> bool {
    let event = keyboard.feed(byte).unwrap().unwrap();
    scene.switcher_key(event, damage).unwrap()
}

fn repaint(scene: &Scene, damage: &mut Damage, frame: &mut [u32], backing: &[u32]) {
    while let Some(tile) = damage.next_tile() {
        let mut pixels = [0xdead_beef; 258];
        scene.render(tile, &mut pixels, backing).unwrap();
        let mut reference = [0xdead_beef; 258];
        scene
            .render_reference(tile, &mut reference, backing)
            .unwrap();
        assert_eq!(pixels, reference, "fast/reference differ at {tile:?}");
        let width = tile.width as usize;
        for row in 0..tile.height as usize {
            let start = (tile.y as usize + row) * scene.width as usize + tile.x as usize;
            frame[start..start + width].copy_from_slice(&pixels[row * width..(row + 1) * width]);
        }
    }
    let mut reference = vec![0; frame.len()];
    scene
        .render_reference(
            Rect::new(0, 0, scene.width, scene.height),
            &mut reference,
            backing,
        )
        .unwrap();
    assert_eq!(frame, reference, "incremental frame missed damage");
}

#[test]
fn keyboard_preview_cycle_cancel_commit_and_tail_releases_repaint_every_pixel() {
    let backing: Vec<_> = (0..MAX_PIXELS)
        .map(|i| (i as u32 * 13) & 0x00ff_ffff)
        .collect();
    for (width, height, scale) in DISPLAYS {
        let mut scene = Scene::new(width, height, scale, [1, 2]).unwrap();
        scene.pointer = (width - 20, 50 * scale);
        let windows = scene.windows;
        let mut damage = Damage::new(width, height).unwrap();
        let mut frame = vec![0; (width * height) as usize];
        damage.mark(Rect::new(0, 0, width, height));
        repaint(&scene, &mut damage, &mut frame, &backing);
        let baseline = frame.clone();
        let mut keyboard = Set1Keyboard::new();
        assert!(!route(&mut scene, &mut damage, 0x38, &mut keyboard)); // Alt.
        assert!(route(&mut scene, &mut damage, 0x0f, &mut keyboard)); // Tab.
        assert_eq!(scene.switcher_selected(), Some(1));
        assert_eq!(scene.focused, 0);
        repaint(&scene, &mut damage, &mut frame, &backing);
        assert_ne!(frame, baseline);
        assert!(route(&mut scene, &mut damage, 0x0f, &mut keyboard)); // Repeat.
        assert_eq!(scene.switcher_selected(), Some(1), "repeat must not cycle");
        assert!(route(&mut scene, &mut damage, 0x8f, &mut keyboard));
        assert!(route(&mut scene, &mut damage, 0x0f, &mut keyboard));
        assert_eq!(scene.switcher_selected(), Some(0));
        repaint(&scene, &mut damage, &mut frame, &backing);
        assert!(route(&mut scene, &mut damage, 0x01, &mut keyboard)); // Escape.
        assert!(!scene.switcher_is_open());
        repaint(&scene, &mut damage, &mut frame, &backing);
        assert_eq!(frame, baseline);
        for byte in [0x81, 0x8f, 0xb8] {
            assert!(route(&mut scene, &mut damage, byte, &mut keyboard));
        }
        for byte in [0x38, 0x0f, 0x8f, 0xb8] {
            let consumed = route(&mut scene, &mut damage, byte, &mut keyboard);
            assert_eq!(consumed, byte != 0x38);
        }
        assert_eq!(scene.focused, 1);
        assert_eq!(scene.switches, 1);
        repaint(&scene, &mut damage, &mut frame, &backing);
        for (before, after) in windows.iter().zip(scene.windows) {
            assert_eq!(before.body, after.body);
            assert_eq!(before.geometry, after.geometry);
            assert_eq!(before.offset, after.offset);
        }
    }
}

#[test]
fn mouse_selection_outside_cancel_and_input_loss_do_not_deliver_stale_releases() {
    let backing = vec![0x1a3245; MAX_PIXELS];
    for (width, height, scale) in DISPLAYS {
        let mut scene = Scene::new(width, height, scale, [1, 2]).unwrap();
        let mut damage = Damage::new(width, height).unwrap();
        let mut frame = vec![0; (width * height) as usize];
        damage.mark(Rect::new(0, 0, width, height));
        repaint(&scene, &mut damage, &mut frame, &backing);
        for selection in [None, Some(1), Some(0)] {
            scene
                .switcher_key(key(PhysicalKey::Tab, KeyAction::Press, true), &mut damage)
                .unwrap();
            repaint(&scene, &mut damage, &mut frame, &backing);
            let focus = scene.focused;
            let point = selection.map_or((width - 5, height - 5), |index| {
                let r = scene.switcher_card(index);
                ((r.x + 5) as u32, (r.y + 5) as u32)
            });
            assert_eq!(
                scene.mouse_with_damage(point.0, point.1, true, true, &mut damage),
                Ok(None)
            );
            assert!(!scene.switcher_is_open());
            assert_eq!(scene.focused, selection.unwrap_or(focus));
            repaint(&scene, &mut damage, &mut frame, &backing);
            let body = scene.windows[scene.focused].body;
            assert_eq!(
                scene.mouse_with_damage(
                    body.x as u32 + 5,
                    body.y as u32 + 5,
                    false,
                    false,
                    &mut damage
                ),
                Ok(None)
            );
            repaint(&scene, &mut damage, &mut frame, &backing);
            scene
                .switcher_key(key(PhysicalKey::Tab, KeyAction::Release, true), &mut damage)
                .unwrap();
            scene
                .switcher_key(
                    key(PhysicalKey::LeftAlt, KeyAction::Release, false),
                    &mut damage,
                )
                .unwrap();
        }
        scene
            .switcher_key(key(PhysicalKey::Tab, KeyAction::Press, true), &mut damage)
            .unwrap();
        let focus = scene.focused;
        repaint(&scene, &mut damage, &mut frame, &backing);
        scene.cancel_palette_input(&mut damage);
        assert_eq!(scene.focused, focus);
        assert!(!scene.switcher_is_open());
        repaint(&scene, &mut damage, &mut frame, &backing);
    }
}

#[test]
fn previews_sample_real_owned_backing_at_both_scales_and_reject_short_buffers() {
    let mut backing = vec![0x112233; MAX_PIXELS];
    for (width, height, scale) in DISPLAYS {
        let mut scene = Scene::new(width, height, scale, [1, 2]).unwrap();
        scene.pointer = (0, 0);
        let mut damage = Damage::new(width, height).unwrap();
        scene
            .switcher_key(key(PhysicalKey::Tab, KeyAction::Press, true), &mut damage)
            .unwrap();
        let bounds = scene.switcher_bounds();
        assert!(bounds.x >= 0 && bounds.y >= 0);
        assert!(bounds.x as u32 + bounds.width <= width);
        assert!(bounds.y as u32 + bounds.height <= height);
        for index in 0..2 {
            let window = scene.windows[index];
            for y in 0..window.body.height as usize {
                for x in 0..window.body.width as usize {
                    backing[window.offset + y * window.body.width as usize + x] =
                        ((x as u32 * 7) ^ (y as u32 * 21) ^ (index as u32 * 0x808080)) & 0xffffff;
                }
            }
        }
        for index in 0..2 {
            let window = scene.windows[index];
            let preview = scene.switcher_preview(index).unwrap();
            let mut pixels = vec![0xdeadbeef; (preview.width * preview.height) as usize + 4];
            scene.render(preview, &mut pixels, &backing).unwrap();
            for y in 0..preview.height {
                for x in 0..preview.width {
                    let at = window.offset
                        + (y * window.body.height / preview.height * window.body.width
                            + x * window.body.width / preview.width)
                            as usize;
                    assert_eq!(pixels[(y * preview.width + x) as usize], backing[at]);
                }
            }
            assert_eq!(
                &pixels[(preview.width * preview.height) as usize..],
                &[0xdeadbeef; 4]
            );
            assert!(scene.render(preview, &mut pixels, &backing[..10]).is_err());
            assert!(scene
                .render_reference(preview, &mut pixels, &backing[..10])
                .is_err());
            let changed = window.offset;
            backing[changed] ^= 0xffffff;
            scene.render(preview, &mut pixels, &backing).unwrap();
            assert_eq!(pixels[0], backing[changed], "preview must use live data");
        }
        assert_eq!(scene.switcher_card(2), Rect::default());
        assert!(scene.switcher_preview(2).is_err());
    }
}

#[test]
fn exclusive_keys_modifiers_repeat_ownership_and_failed_commit_are_bounded() {
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    let mut damage = Damage::new(1280, 720).unwrap();
    for mut modifiers in [
        KeyboardModifiers {
            left_control: true,
            left_alt: true,
            ..KeyboardModifiers::default()
        },
        KeyboardModifiers {
            left_super: true,
            left_alt: true,
            ..KeyboardModifiers::default()
        },
        KeyboardModifiers::default(),
    ] {
        assert!(!scene
            .switcher_key(
                KeyboardEvent {
                    key: PhysicalKey::Tab,
                    action: KeyAction::Press,
                    modifiers
                },
                &mut damage
            )
            .unwrap());
        modifiers.left_alt = true;
        assert!(!scene
            .switcher_key(
                KeyboardEvent {
                    key: PhysicalKey::Tab,
                    action: KeyAction::Repeat,
                    modifiers
                },
                &mut damage
            )
            .unwrap());
    }

    scene.palette_open();
    scene
        .switcher_key(key(PhysicalKey::Tab, KeyAction::Press, true), &mut damage)
        .unwrap();
    assert!(!scene.palette_is_open());
    assert!(!scene.palette_open());
    for physical in [
        PhysicalKey::X,
        PhysicalKey::F6,
        PhysicalKey::Delete,
        PhysicalKey::K,
    ] {
        for action in [KeyAction::Press, KeyAction::Repeat, KeyAction::Release] {
            assert!(scene
                .switcher_key(key(physical, action, true), &mut damage)
                .unwrap());
        }
    }
    scene.switches = u64::MAX;
    assert!(scene
        .switcher_key(key(PhysicalKey::Enter, KeyAction::Press, true), &mut damage)
        .is_err());
    assert_eq!(scene.focused, 0);
    assert!(scene.switcher_is_open());
    scene.switches = 0;
    scene
        .switcher_key(key(PhysicalKey::Enter, KeyAction::Press, true), &mut damage)
        .unwrap();
    assert_eq!(scene.focused, 1);
    for action in [KeyAction::Repeat, KeyAction::Release] {
        assert!(scene
            .switcher_key(key(PhysicalKey::Enter, action, true), &mut damage)
            .unwrap());
    }
    scene
        .switcher_key(
            key(PhysicalKey::LeftAlt, KeyAction::Release, false),
            &mut damage,
        )
        .unwrap();
    assert!(!scene
        .switcher_key(key(PhysicalKey::X, KeyAction::Press, false), &mut damage)
        .unwrap());
}

#[test]
fn both_alt_keys_shift_navigation_and_dock_hover_cancel_are_not_sticky() {
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    let mut damage = Damage::new(1280, 720).unwrap();
    let both = KeyboardModifiers {
        left_alt: true,
        right_alt: true,
        left_shift: true,
        ..KeyboardModifiers::default()
    };
    scene
        .switcher_key(
            KeyboardEvent {
                key: PhysicalKey::Tab,
                action: KeyAction::Press,
                modifiers: both,
            },
            &mut damage,
        )
        .unwrap();
    assert_eq!(scene.switcher_selected(), Some(1));
    scene
        .switcher_key(
            KeyboardEvent {
                key: PhysicalKey::LeftAlt,
                action: KeyAction::Release,
                modifiers: KeyboardModifiers {
                    right_alt: true,
                    ..KeyboardModifiers::default()
                },
            },
            &mut damage,
        )
        .unwrap();
    assert!(scene.switcher_is_open());
    assert_eq!(scene.focused, 0);
    scene
        .switcher_key(
            key(PhysicalKey::RightAlt, KeyAction::Release, false),
            &mut damage,
        )
        .unwrap();
    assert_eq!(scene.focused, 1);
    assert!(!scene.switcher_is_open());
    scene.cancel_palette_input(&mut damage);
    let dock = scene.task_button(0);
    scene.pointer = ((dock.x + 5) as u32, (dock.y + 5) as u32);
    let backing = vec![0x334455; MAX_PIXELS];
    let mut frame = vec![0; (scene.width * scene.height) as usize];
    damage.mark(Rect::new(0, 0, scene.width, scene.height));
    repaint(&scene, &mut damage, &mut frame, &backing);
    scene
        .switcher_key(key(PhysicalKey::Tab, KeyAction::Press, true), &mut damage)
        .unwrap();
    assert_eq!(scene.hovered_task(), None);
    repaint(&scene, &mut damage, &mut frame, &backing);
    scene.cancel_palette_input(&mut damage);
    assert_eq!(scene.hovered_task(), Some(0));
    repaint(&scene, &mut damage, &mut frame, &backing);
}

#[test]
fn switcher_never_opens_on_blocking_shell_pages() {
    for page in [Page::Lock, Page::Auth, Page::Settings] {
        let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
        let mut damage = Damage::new(1280, 720).unwrap();
        scene.shell.open(page);
        assert!(scene.shell.blocks_apps());
        assert!(!scene
            .switcher_key(key(PhysicalKey::Tab, KeyAction::Press, true), &mut damage)
            .unwrap());
        assert!(!scene.switcher_is_open());
        assert_eq!(scene.focused, 0);
        assert_eq!(scene.shell.page(), page);
    }
}

#[test]
fn committing_switcher_reveals_a_minimized_window_for_both_input_routes() {
    for keyboard_commit in [true, false] {
        let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
        let mut damage = Damage::new(1280, 720).unwrap();
        scene.shell.hidden[1] = true;
        assert!(scene
            .switcher_key(key(PhysicalKey::Tab, KeyAction::Press, true), &mut damage)
            .unwrap());
        assert_eq!(scene.switcher_selected(), Some(1));
        assert!(scene.shell.hidden[1]);
        if keyboard_commit {
            assert!(scene
                .switcher_key(key(PhysicalKey::Enter, KeyAction::Press, true), &mut damage)
                .unwrap());
        } else {
            let card = scene.switcher_card(1);
            assert_eq!(
                scene.mouse_with_damage(
                    (card.x + 5) as u32,
                    (card.y + 5) as u32,
                    true,
                    true,
                    &mut damage
                ),
                Ok(None)
            );
        }
        assert_eq!(scene.focused, 1);
        assert!(!scene.shell.hidden[1]);
        assert!(!scene.switcher_is_open());
    }
}


#[test]
fn trusted_shell_shortcuts_pass_through_an_open_switcher_without_focus_commit() {
    for (physical, super_key) in [
        (PhysicalKey::F3, false),
        (PhysicalKey::Menu, false),
        (PhysicalKey::F10, false),
        (PhysicalKey::F12, false),
        (PhysicalKey::L, true),
    ] {
        let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
        let mut damage = Damage::new(1280, 720).unwrap();
        assert!(scene
            .switcher_key(key(PhysicalKey::Tab, KeyAction::Press, true), &mut damage)
            .unwrap());
        assert_eq!(scene.switcher_selected(), Some(1));
        let shortcut = KeyboardEvent {
            key: physical,
            action: KeyAction::Press,
            modifiers: KeyboardModifiers {
                left_alt: true,
                left_super: super_key,
                ..KeyboardModifiers::default()
            },
        };
        assert!(!scene.switcher_key(shortcut, &mut damage).unwrap());
        assert!(!scene.switcher_is_open());
        assert_eq!(scene.focused, 0);
        assert_eq!(scene.switches, 0);
        assert!(scene
            .switcher_key(key(PhysicalKey::Tab, KeyAction::Release, true), &mut damage)
            .unwrap());
        assert!(scene
            .switcher_key(key(PhysicalKey::LeftAlt, KeyAction::Release, false), &mut damage)
            .unwrap());
        assert!(!scene
            .switcher_key(key(physical, KeyAction::Release, false), &mut damage)
            .unwrap());
    }
}

#[test]
fn switcher_surround_click_releases_capture_before_next_app_press() {
    for (width, height, scale) in DISPLAYS {
        let mut scene = Scene::new(width, height, scale, [1, 2]).unwrap();
        let mut damage = Damage::new(width, height).unwrap();
        assert!(scene
            .switcher_key(key(PhysicalKey::Tab, KeyAction::Press, true), &mut damage)
            .unwrap());
        let surround = scene.switcher_bounds();
        let x = surround.x as u32 + 6 * scale;
        let y = surround.y as u32 + 6 * scale;
        for index in 0..2 {
            assert!(!scene.switcher_card(index).contains(x as i32, y as i32));
        }
        assert_eq!(scene.mouse_with_damage(x, y, true, true, &mut damage), Ok(None));
        assert!(scene.switcher_is_open());
        assert_eq!(scene.mouse_with_damage(x, y, false, false, &mut damage), Ok(None));
        assert!(scene.switcher_is_open());
        assert!(scene
            .switcher_key(key(PhysicalKey::LeftAlt, KeyAction::Release, false), &mut damage)
            .unwrap());
        assert!(!scene.switcher_is_open());
        assert_eq!(scene.focused, 1);
        let body = scene.windows[1].body;
        assert_eq!(
            scene.mouse_with_damage(
                (body.x + 8) as u32,
                (body.y + 8) as u32,
                true,
                true,
                &mut damage,
            ),
            Ok(Some(1)),
            "surround release must not swallow the next application click"
        );
    }
}

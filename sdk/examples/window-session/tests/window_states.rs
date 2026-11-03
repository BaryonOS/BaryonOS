//! Host contracts for the native shell; these tests do not execute a VM.
use nieuw_os_graphics::{theme, Rect};
use nieuw_os_surface_protocol::Rectangle;
use nieuw_os_window_session::scene::{
    Damage, Invalid, Scene, WindowState, WorkspaceMode, MAX_PIXELS,
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey, Set1Keyboard};

const DISPLAYS: [(u32, u32, u32); 5] = [
    (800, 500, 1),
    (1280, 720, 1),
    (1920, 1080, 1),
    (1600, 1000, 2),
    (1920, 1080, 2),
];

fn key(key: PhysicalKey, action: KeyAction) -> KeyboardEvent {
    KeyboardEvent {
        key,
        action,
        modifiers: KeyboardModifiers::default(),
    }
}

fn open_menu(scene: &mut Scene, damage: &mut Damage) {
    let mut keyboard = Set1Keyboard::new();
    for byte in [0x38, 0x39, 0xb9, 0xb8] {
        let event = keyboard.feed(byte).unwrap().unwrap();
        let consumed = scene.palette_key(event, damage).unwrap();
        assert_eq!(
            consumed,
            matches!(byte, 0x39 | 0xb9) || (byte == 0xb8 && scene.window_menu_is_open())
        );
    }
}

fn tap(scene: &mut Scene, damage: &mut Damage, physical: PhysicalKey) {
    for action in [KeyAction::Press, KeyAction::Release] {
        assert!(scene.palette_key(key(physical, action), damage).unwrap());
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
    if let Some(at) = screen.iter().zip(&reference).position(|(a, b)| a != b) {
        panic!(
            "{}x{} scale {}: damage differs at ({},{}): {:x} != {:x}",
            scene.width,
            scene.height,
            scene.scale,
            at % scene.width as usize,
            at / scene.width as usize,
            screen[at],
            reference[at]
        );
    }
}

#[test]
fn menu_and_title_buttons_expose_exact_physical_native_coordinates() {
    for (w, h, s, buttons, menus) in [
        (
            800,
            500,
            1,
            [(436, 79), (700, 105)],
            [(436, 112), (488, 138)],
        ),
        (
            1280,
            720,
            1,
            [(836, 79), (1180, 105)],
            [(836, 112), (968, 138)],
        ),
        (
            1920,
            1080,
            1,
            [(916, 79), (1820, 105)],
            [(916, 112), (1608, 138)],
        ),
        (
            1600,
            1000,
            2,
            [(872, 158), (1400, 210)],
            [(872, 224), (976, 276)],
        ),
        (
            1920,
            1080,
            2,
            [(1032, 158), (1720, 210)],
            [(1032, 224), (1296, 276)],
        ),
    ] {
        let mut scene = Scene::new(w, h, s, [1, 2]).unwrap();
        let mut damage = Damage::new(w, h).unwrap();
        for index in 0..2 {
            let (x, y) = buttons[index];
            assert_eq!(
                scene.window_menu_button(index),
                Rect::new(x, y, 28 * s, 28 * s)
            );
            assert_eq!(
                scene.minimize_button(index),
                Rect::new(x + 36 * s as i32, y, 28 * s, 28 * s)
            );
            scene.select(index).unwrap();
            open_menu(&mut scene, &mut damage);
            assert_eq!(
                scene.window_menu_bounds(),
                Rect::new(menus[index].0, menus[index].1, 300 * s, 184 * s)
            );
            tap(&mut scene, &mut damage, PhysicalKey::Escape);
        }
    }
}

#[test]
fn failed_focus_switch_cannot_partly_minimize_or_restore_a_window() {
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    let mut damage = Damage::new(1280, 720).unwrap();
    scene.switches = u64::MAX;
    assert_eq!(scene.minimize(0, &mut damage), Err(Invalid));
    assert_eq!(scene.focused, 0);
    assert_eq!(scene.window_state(0), Some(WindowState::Normal));
    assert!(!damage.has_pending());
    scene.minimize(1, &mut damage).unwrap();
    assert_eq!(scene.select(1), Err(Invalid));
    assert_eq!(scene.focused, 0);
    assert_eq!(scene.window_state(1), Some(WindowState::Minimized));
}

#[test]
fn minimize_restore_and_invalid_indices_preserve_surface_contracts_and_backing() {
    for (w, h, s) in DISPLAYS {
        let mut scene = Scene::new(w, h, s, [17, 29]).unwrap();
        let contracts = scene
            .windows
            .map(|window| (window.body, window.geometry, window.offset));
        let mut damage = Damage::new(w, h).unwrap();
        let mut backing = vec![0xabcdef; MAX_PIXELS];
        assert_eq!(scene.window_state(2), None);
        assert_eq!(scene.minimize(2, &mut damage), Err(Invalid));
        assert_eq!(scene.select(2), Err(Invalid));
        assert!(!damage.has_pending());
        assert_eq!(scene.minimize_button(2), Rect::default());
        assert_eq!(scene.window_menu_button(2), Rect::default());
        scene.minimize(0, &mut damage).unwrap();
        assert_eq!(scene.window_state(0), Some(WindowState::Minimized));
        assert_eq!(scene.focused, 1);
        assert!(scene.focused_visible());
        assert!(!scene.visible(0));
        scene.minimize(1, &mut damage).unwrap();
        assert!(!scene.focused_visible());
        assert!(!scene.visible(1));
        scene.toggle_focus(&mut damage);
        assert!(!scene.focused_visible());
        scene.arrange(&mut damage);
        assert!(!scene.focused_visible(), "arranging is not restoring");

        let tile = Rectangle {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        assert!(scene
            .copy_tile(0, tile, &[0x21, 0x43, 0x65, 0], &mut backing)
            .is_ok());
        assert_eq!(backing[0], 0x654321);
        assert_eq!(backing[scene.windows[1].offset], 0xabcdef);
        assert_eq!(
            scene.copy_tile(0, tile, &[0, 0, 0, 1], &mut backing),
            Err(Invalid)
        );
        assert_eq!(backing[0], 0x654321);
        scene.select(scene.focused).unwrap();
        assert!(
            scene.focused_visible(),
            "select restores even the same index"
        );
        scene.cycle().unwrap();
        assert_eq!(scene.focused, 0);
        assert!(scene.visible(0));
        assert!(scene.visible(1));
        assert_eq!(
            scene
                .windows
                .map(|window| (window.body, window.geometry, window.offset)),
            contracts
        );
        assert_eq!(backing[0], 0x654321);
    }
}

#[test]
fn alt_f9_owns_repeat_and_release_and_no_plain_f9_is_intercepted() {
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    let mut damage = Damage::new(1280, 720).unwrap();
    assert!(!scene
        .palette_key(key(PhysicalKey::F9, KeyAction::Press), &mut damage)
        .unwrap());
    let mut keyboard = Set1Keyboard::new();
    for byte in [0x38, 0x43, 0x43, 0xc3, 0xb8] {
        let event = keyboard.feed(byte).unwrap().unwrap();
        assert_eq!(
            scene.palette_key(event, &mut damage).unwrap(),
            matches!(byte, 0x43 | 0xc3)
        );
    }
    assert_eq!(scene.focused, 1);
    assert_eq!(scene.window_state(0), Some(WindowState::Minimized));
    assert_eq!(scene.window_state(1), Some(WindowState::Normal));
    for byte in [0x38, 0x43, 0xc3, 0xb8] {
        let event = keyboard.feed(byte).unwrap().unwrap();
        scene.palette_key(event, &mut damage).unwrap();
    }
    assert!(!scene.focused_visible());
    open_menu(&mut scene, &mut damage);
    assert!(
        !scene.window_menu_is_open(),
        "no menu for a nonexistent visible focus"
    );
    scene.cycle().unwrap();
    assert!(scene.focused_visible());
}

#[test]
fn modal_keyboard_navigation_cancellation_and_owned_keys_do_not_leak() {
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    let mut damage = Damage::new(1280, 720).unwrap();
    open_menu(&mut scene, &mut damage);
    assert!(scene.window_menu_is_open());
    assert!(!scene.palette_open(), "one modal overlay at a time");
    for action in [KeyAction::Press, KeyAction::Repeat, KeyAction::Release] {
        assert!(scene
            .palette_key(key(PhysicalKey::F7, action), &mut damage)
            .unwrap());
    }
    assert_eq!(scene.workspace_mode(), WorkspaceMode::Work);
    assert!(scene
        .palette_key(key(PhysicalKey::A, KeyAction::Press), &mut damage)
        .unwrap());
    tap(&mut scene, &mut damage, PhysicalKey::Escape);
    assert!(!scene.window_menu_is_open());
    for action in [KeyAction::Repeat, KeyAction::Release] {
        assert!(scene
            .palette_key(key(PhysicalKey::A, action), &mut damage)
            .unwrap());
    }
    assert!(!scene
        .palette_key(key(PhysicalKey::A, KeyAction::Press), &mut damage)
        .unwrap());

    open_menu(&mut scene, &mut damage);
    tap(&mut scene, &mut damage, PhysicalKey::Up);
    for action in [KeyAction::Press, KeyAction::Repeat, KeyAction::Release] {
        assert!(scene
            .palette_key(key(PhysicalKey::Enter, action), &mut damage)
            .unwrap());
    }
    assert_eq!(scene.workspace_mode(), WorkspaceMode::Focus);
    assert!(!scene.visible(1));
    open_menu(&mut scene, &mut damage);
    let mut backtab = Set1Keyboard::new();
    for byte in [0x2a, 0x0f, 0x8f, 0xaa] {
        let event = backtab.feed(byte).unwrap().unwrap();
        assert!(scene.palette_key(event, &mut damage).unwrap());
    }
    tap(&mut scene, &mut damage, PhysicalKey::Enter);
    assert_eq!(scene.workspace_mode(), WorkspaceMode::Work);

    open_menu(&mut scene, &mut damage);
    assert!(scene
        .palette_key(key(PhysicalKey::A, KeyAction::Press), &mut damage)
        .unwrap());
    scene.cancel_palette_input(&mut damage);
    assert!(!scene.window_menu_is_open());
    for action in [KeyAction::Repeat, KeyAction::Release] {
        assert!(scene
            .palette_key(key(PhysicalKey::A, action), &mut damage)
            .unwrap());
    }
    assert!(!scene
        .palette_key(key(PhysicalKey::A, KeyAction::Press), &mut damage)
        .unwrap());
}

#[test]
fn hidden_windows_never_receive_pointer_and_modal_dismiss_has_no_clickthrough() {
    for (w, h, s) in DISPLAYS {
        let mut scene = Scene::new(w, h, s, [1, 2]).unwrap();
        let mut damage = Damage::new(w, h).unwrap();
        scene.minimize(0, &mut damage).unwrap();
        scene.minimize(1, &mut damage).unwrap();
        for index in 0..2 {
            let body = scene.windows[index].body;
            for (down, held) in [(true, true), (false, true), (false, false)] {
                assert_eq!(
                    scene.mouse_with_damage(
                        (body.x + 8) as u32,
                        (body.y + 8) as u32,
                        down,
                        held,
                        &mut damage
                    ),
                    Ok(None)
                );
            }
        }
        let task = scene.task_button(0);
        click(&mut scene, &mut damage, task);
        assert!(scene.focused_visible());
        let menu = scene.window_menu_button(0);
        click(&mut scene, &mut damage, menu);
        assert!(scene.window_menu_is_open());
        let body = scene.windows[0].body;
        let (x, y) = (
            body.x as u32 + 10,
            (body.y + body.height as i32 - 10) as u32,
        );
        assert!(!scene.window_menu_bounds().contains(x as i32, y as i32));
        for (down, held) in [(true, true), (false, true), (false, false)] {
            assert_eq!(
                scene.mouse_with_damage(x, y, down, held, &mut damage),
                Ok(None)
            );
        }
        assert!(!scene.window_menu_is_open());
        assert_eq!(scene.mouse(x, y, true, true), Ok(Some(0)));
        scene.cancel_pointer();
        let frame = scene.frame(0);
        let old_body = scene.windows[0].body;
        scene
            .mouse((frame.x + 30) as u32, (frame.y + 10) as u32, true, true)
            .unwrap();
        scene
            .mouse((frame.x + 50) as u32, (frame.y + 30) as u32, false, true)
            .unwrap();
        assert_eq!(
            scene.moves, 1,
            "original native title drag coordinates still work"
        );
        assert_ne!(scene.windows[0].body, old_body);
        scene.mouse(0, 0, false, false).unwrap();
        let minimize = scene.minimize_button(0);
        click(&mut scene, &mut damage, minimize);
        assert!(!scene.focused_visible());
    }
}

#[test]
fn hidden_focused_body_cannot_take_the_opaque_shortcut_or_require_hidden_backing() {
    for (w, h, s) in DISPLAYS {
        let mut scene = Scene::new(w, h, s, [1, 2]).unwrap();
        let mut damage = Damage::new(w, h).unwrap();
        scene.minimize(0, &mut damage).unwrap();
        scene.minimize(1, &mut damage).unwrap();
        scene.pointer = (0, 0);
        for index in 0..2 {
            let b = scene.windows[index].body;
            let r = Rect::new(b.x + 16, b.y + 24, 32, 8);
            let mut actual = [0x87654321; 260];
            let mut expected = actual;
            scene.render(r, &mut actual, &[]).unwrap();
            scene.render_reference(r, &mut expected, &[]).unwrap();
            assert_eq!(actual, expected);
            assert_eq!(&actual[256..], &[0x87654321; 4]);
            assert!(actual[..256].iter().all(|pixel| *pixel != 0xabcdef));
        }
    }
}

#[test]
fn full_and_incremental_pixels_match_for_every_window_transition_and_menu_shadow() {
    let backing: Vec<_> = (0..MAX_PIXELS)
        .map(|index| (index as u32).wrapping_mul(0x045d9f3b) & 0x00ff_ffff)
        .collect();
    for (w, h, s) in DISPLAYS {
        let mut scene = Scene::new(w, h, s, [17, 29]).unwrap();
        let mut damage = Damage::new(w, h).unwrap();
        let mut screen = vec![0; (w * h) as usize];
        damage.mark(Rect::new(0, 0, w, h));
        repaint(&scene, &mut damage, &mut screen, &backing);
        open_menu(&mut scene, &mut damage);
        let bounds = scene.window_menu_bounds();
        assert!(bounds.x >= 12 * s as i32);
        assert!(bounds.x + bounds.width as i32 + 12 * s as i32 <= w as i32);
        assert!(bounds.y + bounds.height as i32 + 12 * s as i32 <= scene.dock().y);
        repaint(&scene, &mut damage, &mut screen, &backing);
        tap(&mut scene, &mut damage, PhysicalKey::Down);
        repaint(&scene, &mut damage, &mut screen, &backing);
        tap(&mut scene, &mut damage, PhysicalKey::Enter);
        repaint(&scene, &mut damage, &mut screen, &backing);
        assert!(!scene.window_menu_is_open());
        open_menu(&mut scene, &mut damage);
        repaint(&scene, &mut damage, &mut screen, &backing);
        tap(&mut scene, &mut damage, PhysicalKey::Tab);
        tap(&mut scene, &mut damage, PhysicalKey::Tab);
        tap(&mut scene, &mut damage, PhysicalKey::Enter);
        assert_eq!(scene.workspace_mode(), WorkspaceMode::Focus);
        repaint(&scene, &mut damage, &mut screen, &backing);
        open_menu(&mut scene, &mut damage);
        repaint(&scene, &mut damage, &mut screen, &backing);
        tap(&mut scene, &mut damage, PhysicalKey::Enter);
        assert_eq!(scene.focused, 1);
        repaint(&scene, &mut damage, &mut screen, &backing);
        let button = scene.minimize_button(1);
        click(&mut scene, &mut damage, button);
        assert!(!scene.focused_visible());
        repaint(&scene, &mut damage, &mut screen, &backing);
        let dock = scene.task_button(0);
        let marker = |dy| {
            screen[((dock.y + dy * s as i32) as u32 * w + (dock.x + 12 * s as i32) as u32) as usize]
        };
        assert_eq!(marker(36), theme::SHELL_MUTED);
        assert_ne!(marker(40), theme::MATERIAAL.accent, "not an active marker");
        click(&mut scene, &mut damage, dock);
        repaint(&scene, &mut damage, &mut screen, &backing);
        let search = scene.command_button();
        click(&mut scene, &mut damage, search);
        repaint(&scene, &mut damage, &mut screen, &backing);
        tap(&mut scene, &mut damage, PhysicalKey::Down);
        tap(&mut scene, &mut damage, PhysicalKey::Enter);
        assert_eq!(scene.focused, 1);
        assert_eq!(scene.window_state(1), Some(WindowState::Normal));
        repaint(&scene, &mut damage, &mut screen, &backing);
        let menu_button = scene.window_menu_button(1);
        click(&mut scene, &mut damage, menu_button);
        repaint(&scene, &mut damage, &mut screen, &backing);
        let task = scene.task_button(0);
        scene
            .mouse_with_damage(
                (task.x + 2) as u32,
                (task.y + 2) as u32,
                false,
                false,
                &mut damage,
            )
            .unwrap();
        repaint(&scene, &mut damage, &mut screen, &backing);
        scene.cancel_palette_input(&mut damage);
        repaint(&scene, &mut damage, &mut screen, &backing);
        let frame = scene.frame(1);
        scene
            .mouse_with_damage(
                (frame.x + 30) as u32,
                (frame.y + 10) as u32,
                true,
                true,
                &mut damage,
            )
            .unwrap();
        scene
            .mouse_with_damage(w - 10, h - 10, false, true, &mut damage)
            .unwrap();
        scene
            .mouse_with_damage(w - 10, h - 10, false, false, &mut damage)
            .unwrap();
        repaint(&scene, &mut damage, &mut screen, &backing);
        let menu_button = scene.window_menu_button(1);
        click(&mut scene, &mut damage, menu_button);
        repaint(&scene, &mut damage, &mut screen, &backing);
        let bounds = scene.window_menu_bounds();
        let arrange_row = Rect::new(
            bounds.x + 8 * s as i32,
            bounds.y + 52 * s as i32,
            284 * s,
            40 * s,
        );
        click(&mut scene, &mut damage, arrange_row);
        assert_eq!(scene.workspace_mode(), WorkspaceMode::Work);
        assert_eq!(scene.windows[1].body.y, 140 * s as i32);
        repaint(&scene, &mut damage, &mut screen, &backing);
        open_menu(&mut scene, &mut damage);
        repaint(&scene, &mut damage, &mut screen, &backing);
        let task = scene.task_button(0);
        click(&mut scene, &mut damage, task);
        assert_eq!(
            scene.focused, 1,
            "outside click dismisses without selecting the dock"
        );
        repaint(&scene, &mut damage, &mut screen, &backing);
        open_menu(&mut scene, &mut damage);
        repaint(&scene, &mut damage, &mut screen, &backing);
        tap(&mut scene, &mut damage, PhysicalKey::Escape);
        repaint(&scene, &mut damage, &mut screen, &backing);
    }
}

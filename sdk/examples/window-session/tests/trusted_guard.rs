//! Veld overlays and shortcuts stay below main's trusted shell. Host model of
//! the compositor routing order: Scene::palette_key, then Shell::key.
use nieuw_os_window_session::scene::{trusted_shell_key, Damage, Scene, WorkspaceMode};
use nieuw_os_window_session::shell::Page;
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

const DISPLAYS: [(u32, u32, u32); 2] = [(1280, 720, 1), (1920, 1080, 2)];

fn chord(key: PhysicalKey, modifiers: KeyboardModifiers) -> KeyboardEvent {
    KeyboardEvent {
        key,
        action: KeyAction::Press,
        modifiers,
    }
}
fn plain(key: PhysicalKey) -> KeyboardEvent {
    chord(key, KeyboardModifiers::default())
}
fn alt(key: PhysicalKey) -> KeyboardEvent {
    chord(
        key,
        KeyboardModifiers {
            left_alt: true,
            ..Default::default()
        },
    )
}
fn shift(key: PhysicalKey) -> KeyboardEvent {
    chord(
        key,
        KeyboardModifiers {
            left_shift: true,
            ..Default::default()
        },
    )
}
fn ctrl(key: PhysicalKey) -> KeyboardEvent {
    chord(
        key,
        KeyboardModifiers {
            left_control: true,
            ..Default::default()
        },
    )
}
fn super_l() -> KeyboardEvent {
    chord(
        PhysicalKey::L,
        KeyboardModifiers {
            left_super: true,
            ..Default::default()
        },
    )
}
fn tap(scene: &mut Scene, damage: &mut Damage, key: KeyboardEvent) -> bool {
    let consumed = scene.palette_key(key, damage).unwrap();
    let mut release = key;
    release.action = KeyAction::Release;
    scene.palette_key(release, damage).unwrap();
    consumed
}
fn center(r: nieuw_os_graphics::Rect) -> (u32, u32) {
    (
        (r.x + r.width as i32 / 2) as u32,
        (r.y + r.height as i32 / 2) as u32,
    )
}
/// Every Veld overlay, opened through its real shortcut or pointer route.
fn open_overlay(scene: &mut Scene, damage: &mut Damage, overlay: usize) {
    match overlay {
        0 => assert!(tap(scene, damage, ctrl(PhysicalKey::K))),
        1 => assert!(tap(scene, damage, alt(PhysicalKey::Space))),
        2 => assert!(tap(scene, damage, alt(PhysicalKey::N))),
        3 => assert!(tap(scene, damage, alt(PhysicalKey::F7))),
        _ => {
            let (x, y) = center(scene.clipboard_notice_bounds());
            scene.mouse_with_damage(x, y, true, true, damage).unwrap();
        }
    }
    assert!(scene.overlay_is_open(), "overlay {overlay}");
}

#[test]
fn trusted_shell_keys_close_every_veld_overlay_and_reach_the_shell() {
    let trusted = [
        (super_l(), Some(Page::Lock)),
        (plain(PhysicalKey::F3), Some(Page::Search)),
        (plain(PhysicalKey::Menu), Some(Page::Session)),
        (plain(PhysicalKey::F10), None),
        (plain(PhysicalKey::F12), None),
    ];
    for (w, h, s) in DISPLAYS {
        for overlay in 0..5 {
            for (key, page) in trusted {
                assert!(trusted_shell_key(key));
                let mut scene = Scene::new(w, h, s, [1, 2]).unwrap();
                let mut damage = Damage::new(w, h).unwrap();
                let origin = scene.windows.map(|window| window.body);
                open_overlay(&mut scene, &mut damage, overlay);
                if overlay == 3 {
                    assert!(tap(&mut scene, &mut damage, plain(PhysicalKey::Right)));
                    assert_ne!(scene.windows.map(|window| window.body), origin);
                }
                // An overlay may never answer Ok(true) for a trusted key, and
                // its repeat is not owned either.
                assert!(!scene.palette_key(key, &mut damage).unwrap());
                let mut repeat = key;
                repeat.action = KeyAction::Repeat;
                assert!(!scene.palette_key(repeat, &mut damage).unwrap());
                assert!(!scene.overlay_is_open());
                assert_eq!(scene.windows.map(|window| window.body), origin);
                let effect = scene.shell.key(key, w, h, s);
                match page {
                    Some(page) => {
                        assert!(effect.consumed);
                        assert_eq!(scene.shell.page(), page);
                        assert!(scene.shell.blocks_apps());
                    }
                    // F10/F12 stay unconsumed for main.rs' native routes.
                    None => assert!(!effect.consumed),
                }
            }
        }
    }
}

#[test]
fn veld_shortcuts_and_pointer_do_nothing_under_lock_or_any_cover() {
    let shortcuts = [
        alt(PhysicalKey::N),
        alt(PhysicalKey::F7),
        alt(PhysicalKey::F9),
        alt(PhysicalKey::Space),
        plain(PhysicalKey::F7),
        plain(PhysicalKey::F8),
        ctrl(PhysicalKey::K),
        shift(PhysicalKey::F10),
    ];
    for (w, h, s) in DISPLAYS {
        for page in [
            Page::Lock,
            Page::Users,
            Page::Auth,
            Page::Unlocking,
            Page::Search,
            Page::Settings,
            Page::Notifications,
            Page::Session,
            Page::Capability,
        ] {
            let mut scene = Scene::new(w, h, s, [1, 2]).unwrap();
            let mut damage = Damage::new(w, h).unwrap();
            scene.shell.open(page);
            let bodies = scene.windows.map(|window| window.body);
            for key in shortcuts {
                assert!(
                    !scene.palette_key(key, &mut damage).unwrap(),
                    "{page:?} {key:?}"
                );
                let effect = scene.shell.key(key, w, h, s);
                assert!(effect.consumed, "the cover owns {key:?}");
                scene.shell.open(page);
            }
            assert!(!scene.overlay_is_open());
            assert_eq!(scene.workspace_mode(), WorkspaceMode::Work);
            assert_eq!(scene.shell.hidden, [false; 2]);
            assert_eq!(scene.windows.map(|window| window.body), bodies);
            assert_eq!(scene.focused, 0);
            // Pointer: no Veld control, title button or app body reacts, and
            // the compositor never routes a covered pointer to the scene.
            for target in [
                scene.clipboard_notice_bounds(),
                scene.window_menu_button(0),
                scene.minimize_button(1),
                scene.windows[1].body,
            ] {
                let (x, y) = center(target);
                assert!(!scene.desktop_control_at(x, y));
                assert_eq!(
                    scene.mouse_with_damage(x, y, true, true, &mut damage),
                    Ok(None)
                );
                assert_eq!(
                    scene.mouse_with_damage(x, y, false, false, &mut damage),
                    Ok(None)
                );
            }
            assert!(!scene.overlay_is_open());
            assert_eq!(scene.shell.hidden, [false; 2]);
            assert_eq!(scene.focused, 0);
            assert_eq!(scene.hovered_task(), None);
        }
    }
}

#[test]
fn shift_f10_stays_with_an_open_overlay_while_plain_f10_reaches_the_shutdown_route() {
    for (w, h, s) in DISPLAYS {
        let mut scene = Scene::new(w, h, s, [1, 2]).unwrap();
        let mut damage = Damage::new(w, h).unwrap();
        open_overlay(&mut scene, &mut damage, 3);
        assert!(tap(&mut scene, &mut damage, plain(PhysicalKey::Right)));
        let moved = scene.windows[0].body;
        assert!(!trusted_shell_key(shift(PhysicalKey::F10)));
        assert!(scene
            .palette_key(shift(PhysicalKey::F10), &mut damage)
            .unwrap());
        assert!(scene.keyboard_move_is_active());
        assert_eq!(scene.windows[0].body, moved);
        assert!(!scene
            .palette_key(plain(PhysicalKey::F10), &mut damage)
            .unwrap());
        assert!(!scene.keyboard_move_is_active());
        assert_ne!(scene.windows[0].body, moved, "cancel restores the origin");
        // Under the lock screen Shift+F10 is the cover's key, not the app's.
        scene.shell.lock();
        assert!(!scene
            .palette_key(shift(PhysicalKey::F10), &mut damage)
            .unwrap());
        assert!(scene.shell.key(shift(PhysicalKey::F10), w, h, s).consumed);
    }
}

#[test]
fn overlay_pointer_is_released_at_lock_and_cover() {
    for (w, h, s) in DISPLAYS {
        for (target, guard) in [
            (0, true),
            (1, true),
            (2, true),
            (0, false),
            (1, false),
            (2, false),
        ] {
            let mut scene = Scene::new(w, h, s, [1, 2]).unwrap();
            let mut damage = Damage::new(w, h).unwrap();
            // A press swallowed by a Veld control is still held...
            let (x, y) = center(match target {
                0 => scene.minimize_button(0),
                1 => scene.window_menu_button(0),
                _ => scene.clipboard_notice_bounds(),
            });
            assert_eq!(
                scene.mouse_with_damage(x, y, true, true, &mut damage),
                Ok(None)
            );
            // ...when Super+L locks. The compositor closes every overlay as
            // the cover starts; the release then goes to the shell.
            // Without the key guard (any other cover start), the compositor's
            // cover transition alone must release the swallowed press.
            if guard {
                assert!(!scene.palette_key(super_l(), &mut damage).unwrap());
                assert!(scene.shell.key(super_l(), w, h, s).consumed);
            } else {
                scene.shell.lock();
            }
            assert!(scene.shell.blocks_apps());
            scene.cancel_palette_input(&mut damage);
            scene.cancel_pointer();
            assert!(!scene.overlay_is_open());
            scene.shell.mouse((x, y), false, false, w, h, s);
            scene.shell.open(Page::Desktop);
            // The first click after unlock reaches its app instead of being
            // swallowed by a stale overlay press.
            scene.shell.hidden = [false; 2];
            let (bx, by) = center(scene.windows[1].body);
            assert_eq!(
                scene.mouse_with_damage(bx, by, true, true, &mut damage),
                Ok(Some(1)),
                "target {target} guard {guard}"
            );
        }
    }
}

#[test]
fn trusted_guard_keys_are_exactly_the_shell_and_native_routes() {
    for key in [
        super_l(),
        plain(PhysicalKey::F3),
        plain(PhysicalKey::Menu),
        plain(PhysicalKey::F10),
        plain(PhysicalKey::F12),
    ] {
        assert!(trusted_shell_key(key));
    }
    for key in [
        plain(PhysicalKey::L),
        shift(PhysicalKey::F10),
        alt(PhysicalKey::N),
        plain(PhysicalKey::F7),
        ctrl(PhysicalKey::K),
    ] {
        assert!(!trusted_shell_key(key));
    }
}

/// Compositor order for a key: Alt+Tab switcher, then the palette route.
fn dispatch_key(scene: &mut Scene, damage: &mut Damage, key: KeyboardEvent) -> bool {
    scene.switcher_key(key, damage).unwrap() || scene.palette_key(key, damage).unwrap()
}

#[test]
fn alt_tab_during_keyboard_move_rolls_back_and_never_reaches_a_failure_path() {
    for (w, h, s) in DISPLAYS {
        let mut scene = Scene::new(w, h, s, [1, 2]).unwrap();
        let mut damage = Damage::new(w, h).unwrap();
        let origin = scene.windows.map(|window| window.body);
        assert!(dispatch_key(&mut scene, &mut damage, alt(PhysicalKey::F7)));
        assert!(dispatch_key(
            &mut scene,
            &mut damage,
            plain(PhysicalKey::Right)
        ));
        assert_ne!(scene.windows[0].body, origin[0]);
        let mut tab = alt(PhysicalKey::Tab);
        assert!(scene.switcher_key(tab, &mut damage).unwrap());
        assert!(scene.switcher_is_open());
        assert!(!scene.keyboard_move_is_active());
        assert_eq!(scene.windows.map(|window| window.body), origin);
        // Releasing Alt commits the selection; `select` must not be rejected
        // (the compositor treats Err as a fatal failure).
        tab.key = PhysicalKey::LeftAlt;
        tab.action = KeyAction::Release;
        tab.modifiers = KeyboardModifiers::default();
        assert!(scene.switcher_key(tab, &mut damage).is_ok());
        assert!(!scene.switcher_is_open());
        assert!(!scene.overlay_is_open());
    }
}

/// Combined routing as in the compositor: Veld owns the packet or the shell.
fn dispatch_mouse(
    scene: &mut Scene,
    damage: &mut Damage,
    p: (u32, u32),
    down: bool,
    held: bool,
) -> Option<usize> {
    let (w, h, s) = (scene.width, scene.height, scene.scale);
    if scene.veld_owns_pointer(p.0, p.1) {
        return scene
            .mouse_with_damage(p.0, p.1, down, held, damage)
            .unwrap();
    }
    let effect = scene.shell.mouse(p, down, held, w, h, s);
    if effect.consumed {
        scene.update_pointer(p.0, p.1, damage);
        return None;
    }
    scene
        .mouse_with_damage(p.0, p.1, down, held, damage)
        .unwrap()
}

#[test]
fn dismissed_overlay_over_the_dock_owns_its_button_up_and_not_the_next_press() {
    for (w, h, s) in DISPLAYS {
        let mut scene = Scene::new(w, h, s, [1, 2]).unwrap();
        let mut damage = Damage::new(w, h).unwrap();
        assert!(tap(&mut scene, &mut damage, ctrl(PhysicalKey::K)));
        // Dismiss the palette with a click on the dock, outside the overlay.
        let dock = center(scene.dock());
        assert_eq!(
            dispatch_mouse(&mut scene, &mut damage, dock, true, true),
            None
        );
        assert!(!scene.overlay_is_open());
        assert_eq!(
            dispatch_mouse(&mut scene, &mut damage, dock, false, false),
            None
        );
        // The next packet combines movement with a new press and reaches the app.
        let body = center(scene.windows[1].body);
        assert_eq!(
            dispatch_mouse(&mut scene, &mut damage, body, true, true),
            Some(1)
        );
        // A trusted cover keeps priority over a pending Veld pointer.
        assert!(tap(&mut scene, &mut damage, ctrl(PhysicalKey::K)));
        dispatch_mouse(&mut scene, &mut damage, dock, true, true);
        scene.shell.lock();
        scene.cancel_palette_input(&mut damage);
        assert!(!scene.veld_owns_pointer(dock.0, dock.1));
    }
}

use nieuw_os_window_session::scene::{Damage, Scene};

#[test]
fn retirement_moves_focus_only_when_needed_and_never_routes_back() {
    for dead in 0..2 {
        let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
        scene.select(dead).unwrap();
        let mut damage = Damage::new(1280, 720).unwrap();
        scene.retire_leaf(dead, &mut damage).unwrap();
        assert_eq!(scene.focused, 1 - dead);
        assert!(scene.focused_visible());
        scene.retire_leaf(dead, &mut damage).unwrap();
        scene.cycle().unwrap();
        assert_eq!(scene.focused, 1 - dead);
        // Explicit navigation can reveal the fallback, never the app input.
        scene.select(dead).unwrap();
        assert_eq!(scene.focused, dead);
        assert!(!scene.focused_visible());
        scene.cycle().unwrap();
        assert_eq!(scene.focused, 1 - dead);
        scene.retire_leaf(1 - dead, &mut damage).unwrap();
        assert!(!scene.focused_visible());
    }
}

#[test]
fn retired_body_is_rendered_without_stale_app_pixels_at_each_scale() {
    use nieuw_os_graphics::Rect;
    use nieuw_os_window_session::scene::MAX_PIXELS;

    for (width, height, scale) in [(800, 500, 1), (1600, 1000, 2)] {
        for dead in 0..2 {
            let mut scene = Scene::new(width, height, scale, [1, 2]).unwrap();
            let mut damage = Damage::new(width, height).unwrap();
            let backing = vec![0x00fa_00fa; MAX_PIXELS];
            scene.retire_leaf(dead, &mut damage).unwrap();
            // Restrict the comparison to a body tile. The reference path must
            // agree even when the focused window is retired as well.
            scene.retire_leaf(1 - dead, &mut damage).unwrap();
            let body = scene.windows[scene.focused].body;
            let rect = Rect::new(
                body.x + 20 * scale as i32,
                body.y + 20 * scale as i32,
                32 * scale,
                16 * scale,
            );
            let mut fast = vec![0; (rect.width * rect.height) as usize];
            let mut reference = fast.clone();
            scene.render(rect, &mut fast, &backing).unwrap();
            scene
                .render_reference(rect, &mut reference, &backing)
                .unwrap();
            assert_eq!(fast, reference);
            assert!(fast.iter().all(|pixel| *pixel != 0x00fa_00fa));
        }
    }
}

#[test]
fn retired_leaf_rejects_pointer_while_peer_remains_interactive() {
    for dead in 0..2 {
        let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
        let mut damage = Damage::new(1280, 720).unwrap();
        scene.retire_leaf(dead, &mut damage).unwrap();
        let peer = 1 - dead;
        let body = scene.windows[peer].body;
        let x = (body.x + 8) as u32;
        let y = (body.y + body.height as i32 - 8) as u32;
        assert_eq!(scene.mouse(x, y, true, true), Ok(Some(peer)));
        assert_eq!(scene.focused, peer);
        scene.retire_leaf(peer, &mut damage).unwrap();
        assert_eq!(scene.mouse(x, y, true, true), Ok(None));
    }
}

#[test]
fn switcher_does_not_preview_retired_app_pixels() {
    use nieuw_os_window_session::scene::MAX_PIXELS;
    use ui_core::Set1Keyboard;

    for dead in 0..2 {
        let mut scene = Scene::new(800, 500, 1, [1, 2]).unwrap();
        let mut damage = Damage::new(800, 500).unwrap();
        scene.retire_leaf(dead, &mut damage).unwrap();
        let mut keyboard = Set1Keyboard::new();
        for byte in [0x38, 0x0f] {
            let event = keyboard.feed(byte).unwrap().unwrap();
            scene.switcher_key(event, &mut damage).unwrap();
        }
        let rect = scene.switcher_preview(dead).unwrap();
        let mut pixels = vec![0; (rect.width * rect.height) as usize];
        scene
            .render_reference(rect, &mut pixels, &vec![0x00fa_00fa; MAX_PIXELS])
            .unwrap();
        assert!(pixels.iter().all(|pixel| *pixel != 0x00fa_00fa));
    }
}

#[test]
fn keyboard_switcher_reveals_minimized_fallback_without_changing_workspace_mode() {
    use nieuw_os_window_session::scene::WorkspaceMode;
    use ui_core::Set1Keyboard;

    let mut scene = Scene::new(800, 500, 1, [1, 2]).unwrap();
    let mut damage = Damage::new(800, 500).unwrap();
    scene.minimize(1, &mut damage).unwrap();
    scene.toggle_focus(&mut damage);
    scene.retire_leaf(1, &mut damage).unwrap();
    assert_eq!(scene.focused, 0);
    assert_eq!(scene.workspace_mode(), WorkspaceMode::Focus);
    let mut keyboard = Set1Keyboard::new();
    for byte in [0x38, 0x0f, 0x8f, 0xb8] {
        let event = keyboard.feed(byte).unwrap().unwrap();
        scene.switcher_key(event, &mut damage).unwrap();
    }
    assert_eq!(scene.focused, 1);
    assert!(scene.visible(1));
    assert!(!scene.focused_visible());
    assert_eq!(scene.workspace_mode(), WorkspaceMode::Focus);
}

#[test]
fn retiring_the_other_leaf_preserves_the_healthy_window_drag() {
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    let mut damage = Damage::new(1280, 720).unwrap();
    scene.select(1).unwrap();
    let frame = scene.frame(1);
    let x = (frame.x + 80) as u32;
    let y = (frame.y + 15) as u32;
    assert_eq!(scene.mouse(x, y, true, true), Ok(None));
    scene.retire_leaf(0, &mut damage).unwrap();
    let before = scene.windows[1].body;
    assert_eq!(scene.mouse(x - 30, y + 20, false, true), Ok(None));
    assert_ne!(scene.windows[1].body, before);
}

#[test]
fn fallback_text_fits_both_compact_native_scales_with_text_contrast() {
    use nieuw_os_graphics::{theme, Font, Size, Weight};
    fn luminance(color: u32) -> f64 {
        let linear = |shift: u32| {
            let value = f64::from((color >> shift) & 255) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(16) + 0.7152 * linear(8) + 0.0722 * linear(0)
    }
    let palette = theme::MATERIAAL;
    let a = luminance(palette.text);
    let b = luminance(palette.background);
    assert!((a.max(b) + 0.05) / (a.min(b) + 0.05) >= 4.5);
    for (width, height, scale) in [(800, 500, 1), (1600, 1000, 2)] {
        let scene = Scene::new(width, height, scale, [1, 2]).unwrap();
        let font = Font::new(Size::Body, Weight::Semibold, scale == 2);
        for window in scene.windows {
            assert!(font.measure("Dit venster reageert niet") + 32 * scale <= window.body.width);
            assert!(font.height() + 40 * scale <= window.body.height);
        }
    }
}

#[test]
fn retiring_focused_leaf_does_not_restore_a_minimized_peer() {
    use nieuw_os_window_session::scene::WindowState;
    let mut scene = Scene::new(800, 500, 1, [1, 2]).unwrap();
    let mut damage = Damage::new(800, 500).unwrap();
    scene.minimize(1, &mut damage).unwrap();
    scene.retire_leaf(0, &mut damage).unwrap();
    assert_eq!(scene.window_state(1), Some(WindowState::Minimized));
    assert!(!scene.focused_visible());
    scene.select(1).unwrap();
    assert!(scene.focused_visible());
}

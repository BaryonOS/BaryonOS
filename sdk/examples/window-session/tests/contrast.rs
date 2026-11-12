//! WCAG 2.x contrast of main's trusted dock/header roles with the Veld
//! additions, measured against backgrounds sampled from the actual raster.
use nieuw_os_graphics::{theme, Rect};
use nieuw_os_window_session::scene::{Scene, HEADER_TEXT, MAX_PIXELS};
use nieuw_os_window_session::shell::{DOCK_ACCENT, DOCK_ICON, DOCK_MARKER, DOCK_TEXT};

fn luminance(color: u32) -> f64 {
    let linear = |shift: u32| {
        let c = f64::from((color >> shift) & 255) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(16) + 0.7152 * linear(8) + 0.0722 * linear(0)
}
fn contrast(a: u32, b: u32) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[test]
fn dock_and_header_roles_meet_wcag_on_the_actual_dark_desktop() {
    let backing = vec![0; MAX_PIXELS];
    for (w, h, s) in [(1280, 720, 1), (1920, 1080, 2), (800, 500, 1)] {
        let mut scene = Scene::new(w, h, s, [1, 2]).unwrap();
        scene.shell.hidden[1] = true;
        let hover = scene.task_button(1);
        scene.pointer = (
            (hover.x + hover.width as i32 - 8) as u32,
            (hover.y + 4) as u32,
        );
        assert_eq!(scene.hovered_task(), Some(1));
        let mut pixels = vec![0; (w * h) as usize];
        scene
            .render_reference(Rect::new(0, 0, w, h), &mut pixels, &backing)
            .unwrap();
        let at = |x: i32, y: i32| pixels[(y as u32 * w + x as u32) as usize];
        let active = scene.task_button(0);
        let bar = at(w as i32 - 4 * s as i32, h as i32 - 4 * s as i32);
        let hovered = at(hover.x + 6 * s as i32, hover.y + 40 * s as i32);
        let focused = at(active.x + 6 * s as i32, active.y + 40 * s as i32);
        let header = at(4, 4);
        // The sampled pixels are the real dock/header, not the wallpaper.
        assert_ne!(bar, hovered);
        assert_ne!(bar, focused);
        for background in [bar, hovered, focused] {
            assert!(
                contrast(DOCK_TEXT, background) >= 4.5,
                "text {background:06x}"
            );
            // Icons, minimized marker and focus underline: non-text 3:1.
            for role in [DOCK_ICON, DOCK_MARKER, DOCK_ACCENT] {
                assert!(
                    contrast(role, background) >= 3.0,
                    "{role:06x} on {background:06x}"
                );
            }
        }
        // The minimized marker is actually painted on the hovered button.
        let marker = at(hover.x + 16 * s as i32, hover.y + 36 * s as i32);
        assert_eq!(marker, DOCK_MARKER);
        // Veld text inside main's header (notice/placeholder/move hint).
        assert!(contrast(HEADER_TEXT, header) >= 4.5);
        // Shared shell roles also hold over the darker header itself.
        for role in [theme::SHELL_TEXT, theme::SHELL_MUTED] {
            assert!(contrast(role, header) >= 4.5);
        }
    }
}

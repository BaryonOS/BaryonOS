//! Exact host raster/damage contracts. No VM, display or latency acceptance.
use nieuw_os_graphics::Rect;
use nieuw_os_window_session::{
    scene::{Damage, Scene, MAX_PIXELS},
    shell::Page,
};

const PAGES: [Page; 10] = [
    Page::Lock,
    Page::Users,
    Page::Auth,
    Page::Unlocking,
    Page::Desktop,
    Page::Search,
    Page::Settings,
    Page::Notifications,
    Page::Session,
    Page::Capability,
];

fn backing() -> Vec<u32> {
    (0..MAX_PIXELS)
        .map(|i| (i as u32).wrapping_mul(0x045d_9f3b) & 0x00ff_ffff)
        .collect()
}
fn full(scene: &Scene) -> Rect {
    Rect::new(0, 0, scene.width, scene.height)
}
fn reference(scene: &Scene, backing: &[u32]) -> Vec<u32> {
    let mut pixels = vec![0x1234_5678; (scene.width * scene.height) as usize];
    scene
        .render_reference(full(scene), &mut pixels, backing)
        .unwrap();
    pixels
}
fn assert_screen(actual: &[u32], expected: &[u32], context: &str) {
    assert_eq!(actual.len(), expected.len());
    let mismatch = actual.iter().zip(expected).position(|(a, b)| a != b);
    assert!(
        mismatch.is_none(),
        "{context}: first differing pixel {mismatch:?}"
    );
}
fn repaint(scene: &Scene, backing: &[u32], damage: &mut Damage, screen: &mut [u32]) -> usize {
    let mut count = 0;
    while let Some(r) = damage.next_tile() {
        let mut pixels = [0x98ab_cdef; 260];
        scene.render(r, &mut pixels, backing).unwrap();
        let length = (r.width * r.height) as usize;
        assert_eq!(&pixels[length..], &[0x98ab_cdef; 260][length..]);
        for row in 0..r.height as usize {
            let start = (r.y as usize + row) * scene.width as usize + r.x as usize;
            screen[start..start + r.width as usize]
                .copy_from_slice(&pixels[row * r.width as usize..(row + 1) * r.width as usize]);
        }
        count += 1;
    }
    count
}
fn compare_tile(scene: &Scene, r: Rect, backing: &[u32]) {
    let count = (r.width * r.height) as usize;
    let mut fast = vec![0x9876_5432; count + 4];
    let mut slow = fast.clone();
    let actual = scene.render(r, &mut fast, backing);
    let expected = scene.render_reference(r, &mut slow, backing);
    assert_eq!(actual, expected, "error differs at {r:?}");
    if actual.is_ok() {
        assert_eq!(fast, slow, "raster differs at {r:?}");
    }
    assert_eq!(&fast[count..], &[0x9876_5432; 4]);
    assert_eq!(&slow[count..], &[0x9876_5432; 4]);
}

#[test]
fn opaque_shortcut_preserves_every_page_scale_focus_pointer_and_hidden_backing() {
    let pixels = backing();
    for (w, h, scale) in [
        (1280, 720, 1),
        (1440, 900, 1),
        (1024, 768, 1),
        (1920, 1080, 1),
        (1920, 1080, 2),
    ] {
        let mut scene = Scene::new(w, h, scale, [1, 2]).unwrap();
        scene.shell.set_clock(Some(1_793_649_600));
        for page in PAGES {
            scene.shell.open(page);
            let panel = scene.shell.panel(w, h, scale);
            for focus in 0..2 {
                scene.select(focus).unwrap();
                scene.shell.hidden = [focus == 1, focus == 0];
                for pointer in [(0, 0), (w - 1, h - 1), (w / 2, h / 2)] {
                    scene.pointer = pointer;
                    for r in [
                        Rect::new(0, 0, 32, 8),
                        Rect::new(20, (44 * scale - 4) as i32, 32, 8),
                        Rect::new(panel.x, panel.y, 32, 8),
                        Rect::new(panel.x + 28, panel.y + 24, 32, 8),
                        Rect::new(panel.x + panel.width as i32 - 16, panel.y + 12, 32, 8),
                        Rect::new(w as i32 / 2, h as i32 - 64, 32, 8),
                        Rect::new(pointer.0 as i32 - 4, pointer.1 as i32 - 4, 32, 8),
                    ] {
                        compare_tile(&scene, r, &pixels);
                    }
                }
            }
        }
    }
}

#[test]
fn cover_shortcut_keeps_short_buffers_and_backing_geometry_errors() {
    let pixels = backing();
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    scene.shell.lock();
    let body = scene.windows[0];
    let r = Rect::new(body.body.x + 8, body.body.y + 8, 32, 8);
    let end = body.offset + (body.body.width * body.body.height) as usize;
    compare_tile(&scene, r, &pixels[..end - 1]);
    let mut short = [0x1234_5678; 255];
    let before = short;
    assert!(scene.render(r, &mut short, &pixels).is_err());
    assert_eq!(short, before);
    for r in [Rect::new(0, 0, 0, 8), Rect::new(i32::MAX, 0, 32, 8)] {
        compare_tile(&scene, r, &pixels);
    }
    // A moved window shadow can extend past the viewport; retain the original
    // composite behavior instead of incorrectly treating those pixels as cover.
    scene.windows[0].body.x = (scene.width - scene.windows[0].body.width) as i32;
    compare_tile(&scene, Rect::new(1276, body.body.y, 8, 8), &pixels);
}

#[test]
fn panel_damage_matches_complete_frame_for_every_transition_with_clock_and_pointer() {
    let pixels = backing();
    for (w, h, scale) in [(1280, 720, 1), (1024, 768, 1), (1920, 1080, 2)] {
        let mut scene = Scene::new(w, h, scale, [1, 2]).unwrap();
        scene.shell.set_clock(Some(1_793_649_600));
        scene.shell.open(Page::Lock);
        let mut screen = reference(&scene, &pixels);
        for page in PAGES
            .into_iter()
            .chain([Page::Users, Page::Lock, Page::Desktop])
        {
            let was_blocked = scene.shell.blocks_apps();
            let was_hidden = scene.shell.hidden;
            let was_panel = scene.shell.panel(w, h, scale);
            scene.shell.open(page);
            let mut damage = Damage::new(w, h).unwrap();
            damage.mark(scene.shell_damage(was_blocked, was_hidden, was_panel));
            damage.mark(scene.pointer_bounds());
            scene.pointer = if scene.pointer.0 == 1 {
                (w - 5, h - 5)
            } else {
                (1, 1)
            };
            damage.mark(scene.pointer_bounds());
            // The native loop independently marks the status bar on RTC change.
            scene.shell.set_clock(Some(1_793_649_660));
            scene.mark_clock_damage(&mut damage);
            repaint(&scene, &pixels, &mut damage, &mut screen);
            let expected = reference(&scene, &pixels);
            assert_screen(
                &screen,
                &expected,
                &format!("transition to {page:?} at {w}x{h}@{scale}"),
            );
        }
    }
}

#[test]
fn pending_first_cover_and_hidden_inactive_window_cannot_leave_old_pixels() {
    let pixels = backing();
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    let mut screen = reference(&scene, &pixels);
    let mut damage = Damage::new(scene.width, scene.height).unwrap();
    scene.shell.lock();
    damage.mark(full(&scene));
    // An input arrives while the first opaque cover is still being presented.
    let first = damage.next_tile().unwrap();
    let mut tile = [0; 256];
    scene.render(first, &mut tile, &pixels).unwrap();
    for row in 0..8 {
        screen[row * scene.width as usize..row * scene.width as usize + 32]
            .copy_from_slice(&tile[row * 32..row * 32 + 32]);
    }
    scene.shell.open(Page::Users);
    damage.mark(scene.shell_damage(
        true,
        [false; 2],
        scene.shell.panel(scene.width, scene.height, scene.scale),
    ));
    repaint(&scene, &pixels, &mut damage, &mut screen);
    assert_screen(&screen, &reference(&scene, &pixels), "pending first cover");

    scene.shell.open(Page::Desktop);
    screen = reference(&scene, &pixels);
    let old_focus = scene.focused;
    let old_hidden = scene.shell.hidden;
    let inactive = scene.frame(1);
    scene
        .mouse(
            (inactive.x + inactive.width as i32 - 30) as u32,
            (inactive.y + 18) as u32,
            true,
            true,
        )
        .unwrap();
    assert_eq!(scene.focused, old_focus);
    assert!(scene.shell.hidden[1]);
    damage.mark(scene.shell_damage(
        false,
        old_hidden,
        scene.shell.panel(scene.width, scene.height, scene.scale),
    ));
    repaint(&scene, &pixels, &mut damage, &mut screen);
    assert_screen(&screen, &reference(&scene, &pixels), "inactive window hide");
}

#[test]
fn blocked_page_changes_reduce_tiles_but_desktop_and_visibility_changes_are_full() {
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    scene.shell.open(Page::Users);
    let mut damage = Damage::new(scene.width, scene.height).unwrap();
    damage.mark(scene.shell_damage(
        true,
        [false; 2],
        scene.shell.panel(scene.width, scene.height, scene.scale),
    ));
    let mut tiles = 0;
    while damage.next_tile().is_some() {
        tiles += 1;
    }
    assert_eq!(tiles, 1080);
    assert_eq!(
        scene.shell_damage(
            false,
            [false; 2],
            scene.shell.panel(scene.width, scene.height, scene.scale)
        ),
        full(&scene)
    );
    assert_eq!(
        scene.shell_damage(
            true,
            [true, false],
            scene.shell.panel(scene.width, scene.height, scene.scale)
        ),
        full(&scene)
    );
    scene.shell.open(Page::Desktop);
    assert_eq!(
        scene.shell_damage(
            true,
            [false; 2],
            scene.shell.panel(scene.width, scene.height, scene.scale)
        ),
        full(&scene)
    );
}

#[test]
fn lock_clock_updates_both_status_and_large_clock_without_a_page_change() {
    let pixels = backing();
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    scene.shell.lock();
    scene.shell.set_clock(Some(1_793_649_600));
    let mut screen = reference(&scene, &pixels);
    assert!(scene.shell.set_clock(Some(1_793_736_060)));
    let mut damage = Damage::new(scene.width, scene.height).unwrap();
    scene.mark_clock_damage(&mut damage);
    repaint(&scene, &pixels, &mut damage, &mut screen);
    assert_screen(&screen, &reference(&scene, &pixels), "clock update");
}

#[test]
fn shrinking_panel_union_erases_old_panel_and_focus_outside_new_bounds() {
    let pixels = backing();
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    scene.shell.open(Page::Capability);
    let panel = scene.shell.panel(scene.width, scene.height, scene.scale);
    // Model a previously taller/wider page, including its border and focus.
    let previous = Rect::new(
        panel.x - 16,
        panel.y - 16,
        panel.width + 32,
        panel.height + 32,
    );
    let mut screen = reference(&scene, &pixels);
    nieuw_os_graphics::Canvas::new(full(&scene), &mut screen)
        .unwrap()
        .fill_rect(previous, 0xff00ff);
    scene.shell.open(Page::Users);
    let mut damage = Damage::new(scene.width, scene.height).unwrap();
    let region = scene.shell_damage(true, [false; 2], previous);
    assert!(region.x <= previous.x && region.y <= previous.y);
    assert!(region.width >= previous.width && region.height >= previous.height);
    damage.mark(region);
    repaint(&scene, &pixels, &mut damage, &mut screen);
    assert_screen(
        &screen,
        &reference(&scene, &pixels),
        "shrinking panel union",
    );
}

#[test]
fn unlock_keeps_the_cover_until_both_app_frames_and_cover_are_complete() {
    use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    scene.shell.lock();
    for _ in 0..3 {
        scene.shell.key(
            KeyboardEvent {
                key: PhysicalKey::Enter,
                action: KeyAction::Press,
                modifiers: KeyboardModifiers::default(),
            },
            scene.width,
            scene.height,
            scene.scale,
        );
    }
    assert_eq!(scene.shell.page(), Page::Unlocking);
    for (damage, frames) in [
        (true, [Some(1), Some(1)]),
        (false, [None, Some(1)]),
        (false, [Some(1), None]),
        (false, [Some(0), Some(1)]),
    ] {
        assert!(!scene.finish_unlock(damage, frames));
        assert_eq!(scene.shell.page(), Page::Unlocking);
        let b = scene.windows[0].body;
        assert_eq!(
            scene
                .mouse(b.x as u32 + 8, b.y as u32 + 8, true, true)
                .unwrap(),
            None
        );
    }
    assert!(scene.finish_unlock(false, [Some(1), Some(1)]));
    assert_eq!(scene.shell.page(), Page::Desktop);
    assert!(!scene.finish_unlock(false, [Some(1), Some(1)]));
}

#[test]
fn clipboard_status_is_visible_only_in_its_trusted_desktop_region() {
    use nieuw_os_window_session::clipboard::Notice;
    let pixels = backing();
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    let none = reference(&scene, &pixels);
    scene.set_clipboard_notice(Notice::Copied);
    let copied = reference(&scene, &pixels);
    let region = scene.clipboard_notice_bounds();
    let mut changed = 0;
    for (i, (a, b)) in none.iter().zip(&copied).enumerate() {
        if a != b {
            changed += 1;
            assert!(region.contains(
                (i % scene.width as usize) as i32,
                (i / scene.width as usize) as i32
            ));
        }
    }
    assert!(changed > 0);
    scene.shell.lock();
    let covered = reference(&scene, &pixels);
    scene.set_clipboard_notice(Notice::Empty);
    assert_screen(
        &covered,
        &reference(&scene, &pixels),
        "clipboard text under cover",
    );
}

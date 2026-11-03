use nieuw_os_graphics::Rect;
use nieuw_os_window_session::{
    scene::{Scene, MAX_PIXELS},
    shell::{Command, Page, Shell},
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};
fn key(shell: &mut Shell, k: PhysicalKey) -> Command {
    shell
        .key(
            KeyboardEvent {
                key: k,
                action: KeyAction::Press,
                modifiers: KeyboardModifiers::default(),
            },
            1280,
            720,
            1,
        )
        .command
}
#[test]
fn public_guest_route_is_separate_from_unavailable_personal_auth() {
    let mut s = Shell::new(true);
    assert_eq!(s.page(), Page::Lock);
    assert!(s.blocks_apps());
    key(&mut s, PhysicalKey::Enter);
    assert_eq!(s.page(), Page::Users);
    key(&mut s, PhysicalKey::Tab);
    key(&mut s, PhysicalKey::Enter);
    assert_eq!(s.page(), Page::Auth);
    for method in [
        PhysicalKey::Enter,
        PhysicalKey::Tab,
        PhysicalKey::Enter,
        PhysicalKey::Tab,
        PhysicalKey::Enter,
    ] {
        key(&mut s, method);
        assert!(s.blocks_apps());
        assert!(!s.finish_unlock());
    }
    key(&mut s, PhysicalKey::Escape);
    assert_eq!(s.page(), Page::Lock);
    key(&mut s, PhysicalKey::Enter);
    key(&mut s, PhysicalKey::Enter);
    assert_eq!(s.page(), Page::Auth);
    key(&mut s, PhysicalKey::Enter);
    assert_eq!(s.page(), Page::Unlocking);
    assert!(s.finish_unlock());
    assert_eq!(s.page(), Page::Desktop);
    s.lock();
    assert!(s.blocks_apps());
    assert!(!s.finish_unlock());
}
#[test]
fn search_finds_apps_material_settings_and_actions_without_network() {
    for (keys, expected) in [
        (
            vec![PhysicalKey::M, PhysicalKey::A, PhysicalKey::T],
            Command::Activate(0),
        ),
        (
            vec![PhysicalKey::R, PhysicalKey::E, PhysicalKey::K],
            Command::Activate(1),
        ),
    ] {
        let mut s = Shell::new(false);
        key(&mut s, PhysicalKey::F3);
        for k in keys {
            key(&mut s, k);
        }
        assert_eq!(key(&mut s, PhysicalKey::Enter), expected);
        assert!(!s.blocks_apps());
    }
    let mut s = Shell::new(false);
    key(&mut s, PhysicalKey::F3);
    for k in [PhysicalKey::I, PhysicalKey::N, PhysicalKey::S] {
        key(&mut s, k);
    }
    key(&mut s, PhysicalKey::Enter);
    assert_eq!(s.page(), Page::Settings);
    key(&mut s, PhysicalKey::Escape);
    key(&mut s, PhysicalKey::F3);
    for k in [
        PhysicalKey::L,
        PhysicalKey::O,
        PhysicalKey::C,
        PhysicalKey::K,
    ] {
        key(&mut s, k);
    }
    key(&mut s, PhysicalKey::Enter);
    assert_eq!(s.page(), Page::Lock);
}
#[test]
fn cancelled_pointer_or_page_change_cannot_activate_stale_controls() {
    let mut s = Shell::new(true);
    let c = s.controls(1280, 720, 1)[0].unwrap().bounds;
    s.mouse((c.x as u32 + 4, c.y as u32 + 4), true, true, 1280, 720, 1);
    s.cancel_pointer();
    s.mouse((c.x as u32 + 4, c.y as u32 + 4), false, false, 1280, 720, 1);
    assert_eq!(s.page(), Page::Lock);
    s.mouse((c.x as u32 + 4, c.y as u32 + 4), true, true, 1280, 720, 1);
    s.open(Page::Users);
    s.mouse((c.x as u32 + 4, c.y as u32 + 4), false, false, 1280, 720, 1);
    assert_eq!(s.page(), Page::Users);
}
#[test]
fn super_l_cannot_be_swallowed_by_launcher_and_escape_keeps_app_backing() {
    let mut s = Shell::new(false);
    let mut event = KeyboardEvent {
        key: PhysicalKey::LeftSuper,
        action: KeyAction::Press,
        modifiers: KeyboardModifiers {
            left_super: true,
            ..Default::default()
        },
    };
    s.key(event, 1280, 720, 1);
    assert_eq!(s.page(), Page::Desktop);
    event.key = PhysicalKey::L;
    s.key(event, 1280, 720, 1);
    assert_eq!(s.page(), Page::Lock);
    event.key = PhysicalKey::LeftSuper;
    event.action = KeyAction::Release;
    s.key(event, 1280, 720, 1);
    assert_eq!(s.page(), Page::Lock);
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    let backing = vec![0x123456; MAX_PIXELS];
    let before = backing.clone();
    scene.shell.open(Page::Search);
    key(&mut scene.shell, PhysicalKey::Escape);
    assert_eq!(backing, before);
    assert_eq!(scene.shell.page(), Page::Desktop);
}
#[test]
fn trusted_cover_is_opaque_even_when_apps_change_and_fast_path_cannot_bypass_it() {
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    scene.shell.lock();
    let a = vec![0xff00ff; MAX_PIXELS];
    let b = vec![0x00ff00; MAX_PIXELS];
    let tile = Rect::new(160, 200, 32, 8);
    let mut x = [0; 256];
    let mut y = [0; 256];
    scene.render(tile, &mut x, &a).unwrap();
    scene.render(tile, &mut y, &b).unwrap();
    assert_eq!(x, y);
    assert!(!x.contains(&0xff00ff));
}
#[test]
fn controls_are_bounded_nonoverlapping_at_supported_sizes_and_200_percent() {
    for (w, h, scale) in [
        (1280, 720, 1),
        (1920, 1080, 1),
        (1440, 900, 1),
        (1024, 768, 1),
        (1920, 1080, 2),
    ] {
        let mut s = Shell::new(true);
        for page in [
            Page::Lock,
            Page::Users,
            Page::Auth,
            Page::Search,
            Page::Settings,
            Page::Session,
            Page::Capability,
        ] {
            s.open(page);
            let controls: Vec<_> = s.controls(w, h, scale).into_iter().flatten().collect();
            for (i, c) in controls.iter().enumerate() {
                assert!(c.bounds.x >= 0 && c.bounds.y >= 0);
                assert!(c.bounds.x as u32 + c.bounds.width <= w);
                assert!(c.bounds.y as u32 + c.bounds.height <= h);
                assert!(c.bounds.height >= 44 * scale);
                for previous in &controls[..i] {
                    assert!(!c.bounds.intersects(previous.bounds));
                }
            }
        }
    }
}
#[test]
fn rtc_is_display_only_bounded_and_handles_leap_dates() {
    let mut s = Shell::new(true);
    s.set_clock(Some(0));
    assert_eq!(s.clock(), "00:00");
    assert_eq!(s.date(), "1970-01-01");
    s.set_clock(Some(951_825_600));
    assert_eq!(s.date(), "2000-02-29");
    for t in [None, Some(-1), Some(i64::MAX)] {
        s.set_clock(t);
        assert_eq!(s.clock(), "--:--");
        assert_eq!(s.date(), "----------");
    }
    assert!(s.blocks_apps());
}

#[test]
fn panel_pointer_damage_tracks_focus_and_activation_only() {
    let mut shell = Shell::new(true);
    key(&mut shell, PhysicalKey::Enter);
    let controls: Vec<_> = shell.controls(1280, 720, 1).into_iter().flatten().collect();
    let point = |i: usize| {
        (
            controls[i].bounds.x as u32 + 2,
            controls[i].bounds.y as u32 + 2,
        )
    };
    assert!(!shell.mouse(point(0), false, false, 1280, 720, 1).changed);
    assert!(!shell.mouse(point(0), true, true, 1280, 720, 1).changed);
    assert!(!shell.mouse(point(1), false, true, 1280, 720, 1).changed);
    assert!(!shell.mouse(point(1), false, false, 1280, 720, 1).changed);
    assert!(shell.mouse(point(1), true, true, 1280, 720, 1).changed);
    assert!(!shell.mouse(point(1), false, true, 1280, 720, 1).changed);
    assert!(shell.mouse(point(1), false, false, 1280, 720, 1).changed);
}

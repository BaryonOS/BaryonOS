//! Native renderer contracts and opt-in host previews. No VM is executed here.
use nieuw_os_graphics::Rect;
use nieuw_os_surface_protocol::Rectangle;
use nieuw_os_window_session::scene::{Damage, Scene, MAX_PIXELS};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey, Set1Keyboard};

fn damage_covers(damage: &mut Damage, bounds: Rect) {
    let tiles: Vec<_> = core::iter::from_fn(|| damage.next_tile()).collect();
    for y in bounds.y..bounds.y + bounds.height as i32 {
        for x in bounds.x..bounds.x + bounds.width as i32 {
            assert!(
                tiles.iter().any(|tile| tile.contains(x, y)),
                "missing ({x},{y})"
            );
        }
    }
}

#[test]
fn palette_does_not_swallow_trusted_lock_search_or_session_shortcuts() {
    use nieuw_os_window_session::shell::Page;
    for (width, height, scale) in [(1280, 720, 1), (1920, 1080, 2)] {
        for (physical, super_key, expected) in [
            (PhysicalKey::L, true, Page::Lock),
            (PhysicalKey::F3, false, Page::Search),
            (PhysicalKey::Menu, false, Page::Session),
        ] {
            let mut scene = Scene::new(width, height, scale, [1, 2]).unwrap();
            let mut damage = Damage::new(width, height).unwrap();
            scene.palette_open();
            let key = KeyboardEvent {
                key: physical,
                action: KeyAction::Press,
                modifiers: KeyboardModifiers {
                    left_super: super_key,
                    ..KeyboardModifiers::default()
                },
            };
            assert!(!scene.palette_key(key, &mut damage).unwrap());
            assert!(!scene.palette_is_open());
            let effect = scene.shell.key(key, width, height, scale);
            assert!(effect.consumed);
            assert_eq!(scene.shell.page(), expected);
            assert!(scene.shell.blocks_apps());
            damage_covers(&mut damage, scene.palette_bounds());
        }
    }
}

#[test]
fn palette_cannot_open_over_trusted_pages_and_can_reveal_a_hidden_app() {
    use nieuw_os_window_session::shell::Page;
    for (width, height, scale) in [(1280, 720, 1), (1920, 1080, 2)] {
        let mut scene = Scene::new(width, height, scale, [1, 2]).unwrap();
        let mut damage = Damage::new(width, height).unwrap();
        let mut keyboard = Set1Keyboard::new();
        keyboard.feed(0x1d).unwrap();
        let open = keyboard.feed(0x25).unwrap().unwrap();
        for page in [Page::Lock, Page::Search, Page::Settings, Page::Session] {
            scene.shell.open(page);
            assert!(!scene.palette_key(open, &mut damage).unwrap());
            assert!(!scene.palette_is_open());
        }
        scene.shell.open(Page::Desktop);
        scene.shell.hidden[1] = true;
        assert!(scene.palette_key(open, &mut damage).unwrap());
        scene.palette_next(false);
        let enter = keyboard.feed(0x1c).unwrap().unwrap();
        assert!(scene.palette_key(enter, &mut damage).unwrap());
        assert_eq!(scene.focused, 1);
        assert!(!scene.shell.hidden[1]);
        assert!(!scene.palette_is_open());
        assert!(scene
            .palette_key(keyboard.feed(0xa5).unwrap().unwrap(), &mut damage)
            .unwrap());
        scene.cancel_palette_input(&mut damage);
        assert!(!scene
            .palette_key(keyboard.feed(0x9c).unwrap().unwrap(), &mut damage)
            .unwrap());
    }
}

#[test]
fn shell_consumed_pointer_updates_damage_actual_dock_hover_pixels() {
    for (width, height, scale) in [(1280, 720, 1), (1920, 1080, 2)] {
        let mut scene = Scene::new(width, height, scale, [1, 2]).unwrap();
        let backing = vec![0x424242; MAX_PIXELS];
        let dock = scene.dock();
        let mut before = vec![0; (dock.width * dock.height) as usize];
        let mut after = before.clone();
        scene.render(dock, &mut before, &backing).unwrap();
        let button = scene.task_button(1);
        let mut damage = Damage::new(width, height).unwrap();
        scene.update_pointer((button.x + 12) as u32, (button.y + 12) as u32, &mut damage);
        scene.render(dock, &mut after, &backing).unwrap();
        assert_eq!(scene.hovered_task(), Some(1));
        assert_eq!(scene.focused, 0);
        let tiles: Vec<_> = core::iter::from_fn(|| damage.next_tile()).collect();
        let mut changed = 0;
        for (index, (a, b)) in before.iter().zip(&after).enumerate() {
            if a != b {
                changed += 1;
                let x = dock.x + (index % dock.width as usize) as i32;
                let y = dock.y + (index / dock.width as usize) as i32;
                assert!(tiles.iter().any(|tile| tile.contains(x, y)));
            }
        }
        assert!(changed > 100 * scale * scale);
        scene.update_pointer(0, 0, &mut damage);
        damage_covers(&mut damage, button);
        scene.render(dock, &mut before, &backing).unwrap();
        assert_ne!(before, after);
    }
}

#[test]
fn native_palette_key_route_opens_filters_selects_and_consumes_releases() {
    for (width, height, scale) in [(1280, 720, 1), (1920, 1080, 2)] {
        let mut scene = Scene::new(width, height, scale, [1, 2]).unwrap();
        let mut damage = Damage::new(width, height).unwrap();
        let mut keyboard = Set1Keyboard::new();
        for byte in [0x1d, 0x25, 0xa5, 0x9d, 0x13, 0x93, 0x12, 0x92, 0x25, 0xa5] {
            let key = keyboard.feed(byte).unwrap().unwrap();
            let consumed = scene.palette_key(key, &mut damage).unwrap();
            assert_eq!(consumed, byte != 0x1d);
        }
        assert!(scene.palette_is_open());
        assert_eq!(scene.palette_query(), "rek");
        assert_eq!(scene.palette_selected(), Some(1));
        assert_eq!(scene.focused, 0);
        damage_covers(&mut damage, scene.palette_bounds());
        let enter = keyboard.feed(0x1c).unwrap().unwrap();
        assert!(scene.palette_key(enter, &mut damage).unwrap());
        assert!(!scene.palette_is_open());
        assert_eq!(scene.focused, 1);
        assert!(scene
            .palette_key(keyboard.feed(0x9c).unwrap().unwrap(), &mut damage)
            .unwrap());
        assert!(!scene
            .palette_key(keyboard.feed(0x1e).unwrap().unwrap(), &mut damage)
            .unwrap());
        damage_covers(&mut damage, scene.palette_bounds());
    }
}

#[test]
fn palette_navigation_empty_results_escape_and_mouse_do_not_reach_apps() {
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    let mut damage = Damage::new(1280, 720).unwrap();
    let key = |key| KeyboardEvent {
        key,
        action: KeyAction::Press,
        modifiers: KeyboardModifiers::default(),
    };
    scene.palette_open();
    assert!(scene
        .palette_key(key(PhysicalKey::Tab), &mut damage)
        .unwrap());
    assert_eq!(scene.palette_selected(), Some(1));
    assert!(scene
        .palette_key(key(PhysicalKey::Up), &mut damage)
        .unwrap());
    assert_eq!(scene.palette_selected(), Some(0));
    assert!(scene.palette_key(key(PhysicalKey::Z), &mut damage).unwrap());
    assert_eq!(scene.palette_selected(), None);
    assert!(scene
        .palette_key(key(PhysicalKey::Enter), &mut damage)
        .unwrap());
    assert!(scene.palette_is_open());
    assert_eq!(scene.focused, 0);
    assert!(scene
        .palette_key(key(PhysicalKey::Backspace), &mut damage)
        .unwrap());
    assert_eq!(scene.palette_query(), "");
    let bounds = scene.palette_bounds();
    assert_eq!(
        scene.mouse_with_damage(
            (bounds.x + 30) as u32,
            (bounds.y + 160) as u32,
            true,
            true,
            &mut damage
        ),
        Ok(None)
    );
    assert_eq!(scene.focused, 1);
    assert!(!scene.palette_is_open());
    damage_covers(&mut damage, bounds);
    scene.palette_open();
    assert!(scene
        .palette_key(key(PhysicalKey::Escape), &mut damage)
        .unwrap());
    assert!(!scene.palette_is_open());
    assert_eq!(scene.focused, 1);
    scene.cancel_palette_input(&mut damage);
    assert!(!scene
        .palette_key(key(PhysicalKey::Enter), &mut damage)
        .unwrap());
}

#[test]
fn native_mouse_damage_covers_entire_old_and_new_hovered_dock_buttons() {
    for (width, height, scale) in [(1280, 720, 1), (1920, 1080, 2)] {
        let mut scene = Scene::new(width, height, scale, [1, 2]).unwrap();
        let mut damage = Damage::new(width, height).unwrap();
        for index in [1, 0] {
            let old = scene.hovered_task();
            let bounds = scene.task_button(index);
            assert_eq!(
                scene.mouse_with_damage(
                    (bounds.x + 2) as u32,
                    (bounds.y + 2) as u32,
                    false,
                    false,
                    &mut damage
                ),
                Ok(None)
            );
            assert_eq!(scene.focused, 0, "hover must not select an app");
            let tiles: Vec<_> = core::iter::from_fn(|| damage.next_tile()).collect();
            for rect in [old.map(|i| scene.task_button(i)), Some(bounds)]
                .into_iter()
                .flatten()
            {
                for y in rect.y..rect.y + rect.height as i32 {
                    for x in rect.x..rect.x + rect.width as i32 {
                        assert!(tiles.iter().any(|tile| tile.contains(x, y)));
                    }
                }
            }
        }
        let old = scene.task_button(0);
        scene
            .mouse_with_damage(0, 0, false, false, &mut damage)
            .unwrap();
        assert_eq!(scene.hovered_task(), None);
        damage_covers(&mut damage, old);
    }
}

#[test]
fn palette_filters_only_real_apps_and_cannot_create_or_select_a_placeholder() {
    for (width, height, scale) in [(1280, 720, 1), (1920, 1080, 2)] {
        let mut scene = Scene::new(width, height, scale, [1, 2]).unwrap();
        let body = scene.windows.map(|window| window.body);
        assert!(!scene.palette_insert(b'r'));
        assert_eq!(scene.palette_selected(), None);
        assert!(scene.palette_open());
        assert!(!scene.palette_open());
        assert_eq!(scene.palette_selected(), Some(0));
        assert!(scene.palette_next(false));
        assert_eq!(scene.palette_selected(), Some(1));
        assert!(scene.palette_next(true));
        assert_eq!(scene.palette_selected(), Some(0));
        for byte in b"ReKeN" {
            assert!(scene.palette_insert(*byte));
        }
        assert_eq!(scene.palette_query(), "ReKeN");
        assert_eq!(scene.palette_selected(), Some(1));
        assert!(!scene.palette_next(false));
        assert_eq!(scene.focused, 0, "query never grants or changes focus");
        let bounds = scene.palette_bounds();
        assert_eq!(
            scene.palette_hit(
                (bounds.x + 30 * scale as i32) as u32,
                (bounds.y + 112 * scale as i32) as u32
            ),
            Some(1)
        );
        assert_eq!(scene.palette_hit(0, 0), None);
        assert!(scene.palette_insert(b'x'));
        assert_eq!(scene.palette_selected(), None);
        assert_eq!(
            scene.palette_hit(
                (bounds.x + 30 * scale as i32) as u32,
                (bounds.y + 112 * scale as i32) as u32
            ),
            None
        );
        assert!(scene.palette_backspace());
        assert_eq!(scene.palette_selected(), Some(1));
        assert!(scene.palette_close());
        assert!(!scene.palette_close());
        assert_eq!(scene.palette_selected(), None);
        assert_eq!(scene.palette_query(), "");
        assert_eq!(scene.windows.map(|window| window.body), body);
        assert_eq!(
            scene.palette_bounds(),
            bounds,
            "close still identifies damaged pixels"
        );
        assert!(scene.palette_open());
        for _ in 0..32 {
            assert!(scene.palette_insert(b'a'));
        }
        assert!(!scene.palette_insert(b'a'));
        assert!(!scene.palette_insert(0xff));
        assert!(!scene.palette_insert(b'\n'));
        assert_eq!(scene.palette_query().len(), 32);
    }
}

#[test]
fn task_buttons_are_the_actual_click_regions_and_palette_is_not_skipped_by_fast_tiles() {
    let backing = vec![0xabcdef; MAX_PIXELS];
    for (width, height, scale) in [(1280, 720, 1), (1920, 1080, 2)] {
        let mut scene = Scene::new(width, height, scale, [1, 2]).unwrap();
        for index in [1, 0] {
            let r = scene.task_button(index);
            assert!(scene.dock().contains(r.x, r.y));
            assert_eq!(
                scene.mouse((r.x + 2) as u32, (r.y + 2) as u32, true, true),
                Ok(None)
            );
            assert_eq!(scene.focused, index);
        }
        assert_eq!(scene.task_button(2), Rect::default());
        scene.pointer = (0, 0);
        scene.palette_open();
        for byte in b"mat" {
            scene.palette_insert(*byte);
        }
        let bounds = scene.palette_bounds();
        for y in (0..bounds.height as i32).step_by(7) {
            for x in (0..bounds.width as i32).step_by(31) {
                let tile = Rect::new(bounds.x + x, bounds.y + y, 32, 8);
                let mut actual = [0xdeadbeef; 258];
                let mut reference = actual;
                scene.render(tile, &mut actual, &backing).unwrap();
                scene
                    .render_reference(tile, &mut reference, &backing)
                    .unwrap();
                assert_eq!(actual, reference);
                assert_eq!(&actual[256..], &[0xdeadbeef; 2]);
            }
        }
    }
}

fn read_ppm(path: &std::path::Path, width: u32, height: u32) -> Vec<u8> {
    let data = std::fs::read(path).unwrap();
    let header = format!("P6\n{width} {height}\n255\n");
    assert!(data.starts_with(header.as_bytes()));
    assert_eq!(data.len(), header.len() + (width * height * 3) as usize);
    data[header.len()..].to_vec()
}
fn write_ppm(path: &std::path::Path, width: u32, height: u32, pixels: &[u32]) {
    use std::io::Write;
    let mut file = std::fs::File::create_new(path).unwrap();
    write!(file, "P6\n{width} {height}\n255\n").unwrap();
    for pixel in pixels {
        file.write_all(&[(pixel >> 16) as u8, (pixel >> 8) as u8, *pixel as u8])
            .unwrap();
    }
    file.sync_all().unwrap();
}

#[test]
fn host_preview_composes_actual_rust_app_rasters_with_the_actual_scene() {
    let Some(directory) = std::env::var_os("OS_WORKSPACE_PREVIEW_DIRECTORY") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    let calculator =
        std::path::PathBuf::from(std::env::var_os("OS_WORKSPACE_CALCULATOR_PPM").unwrap());
    let calc = read_ppm(&calculator, 320, 484);
    let mut sheet = vec![0; 2560 * 1440];
    for (panel, phase) in ["initial", "dirty", "palette", "palette-filtered"]
        .into_iter()
        .enumerate()
    {
        let material = read_ppm(
            &directory.join("material").join(if phase == "initial" {
                "880x510-initial.ppm"
            } else {
                "880x510-dirty.ppm"
            }),
            880,
            510,
        );
        let mut backing = vec![0; MAX_PIXELS];
        let mut scene = Scene::new(1280, 720, 1, [11, 12]).unwrap();
        scene.pointer = (1240, 60);
        for (index, rgb) in [&material, &calc].into_iter().enumerate() {
            let window = scene.windows[index];
            let mut bytes =
                Vec::with_capacity((window.body.width * window.body.height * 4) as usize);
            for pixel in rgb.chunks_exact(3) {
                bytes.extend_from_slice(&[pixel[2], pixel[1], pixel[0], 0]);
            }
            scene
                .copy_tile(
                    index,
                    Rectangle {
                        x: 0,
                        y: 0,
                        width: window.body.width as u16,
                        height: window.body.height as u16,
                    },
                    &bytes,
                    &mut backing,
                )
                .unwrap();
        }
        if phase.starts_with("palette") {
            scene.palette_open();
            if phase == "palette-filtered" {
                for byte in b"rek" {
                    scene.palette_insert(*byte);
                }
            }
        }
        let mut pixels = vec![0; 1280 * 720];
        scene
            .render(Rect::new(0, 0, 1280, 720), &mut pixels, &backing)
            .unwrap();
        write_ppm(
            &directory.join(format!("workspace-{phase}.ppm")),
            1280,
            720,
            &pixels,
        );
        let (left, top) = ((panel % 2) * 1280, (panel / 2) * 720);
        for row in 0..720 {
            sheet[(top + row) * 2560 + left..(top + row) * 2560 + left + 1280]
                .copy_from_slice(&pixels[row * 1280..(row + 1) * 1280]);
        }
    }
    write_ppm(
        &directory.join("host-contact-sheet.ppm"),
        2560,
        1440,
        &sheet,
    );
}

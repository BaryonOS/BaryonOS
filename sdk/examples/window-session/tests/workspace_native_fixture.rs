//! Shell-only references. Zero app buffers are not application raster evidence.
use nieuw_os_graphics::Rect;
use nieuw_os_window_session::scene::{Damage, Scene, MAX_PIXELS};
use std::io::Write;
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

fn tap(scene: &mut Scene, damage: &mut Damage, key: PhysicalKey, alt: bool, shift: bool) {
    for action in [KeyAction::Press, KeyAction::Release] {
        assert!(scene
            .palette_key(
                KeyboardEvent {
                    key,
                    action,
                    modifiers: KeyboardModifiers {
                        left_alt: alt,
                        left_shift: shift,
                        ..Default::default()
                    },
                },
                damage,
            )
            .unwrap());
    }
}

#[test]
fn render_workspace_shell_oracles() {
    let Some(directory) = std::env::var_os("OS_WINDOW_FIXTURE_DIRECTORY") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    let backing = vec![0; MAX_PIXELS];
    let scale = std::env::var("OS_WINDOW_SHELL_SCALE")
        .unwrap_or_else(|_| "1".into())
        .parse::<u32>()
        .unwrap();
    let (width, height) = match scale {
        1 => (1280, 720),
        2 => (1920, 1080),
        _ => panic!("unsupported native fixture scale"),
    };
    let mut scene = Scene::new(width, height, scale, [1, 2]).unwrap();
    scene.pointer = (width - 20, 50 * scale);
    let mut damage = Damage::new(width, height).unwrap();
    for phase in [
        "focus-material",
        "focus-calculator",
        "work",
        "search",
        "selected",
        "window-menu",
        "window-menu-next",
        "window-minimized",
        "window-empty",
        "window-restored",
        "move-open",
        "move-coarse",
        "move-fine",
        "move-committed",
        "move-cancel-open",
        "move-cancel-step",
        "move-cancelled",
        "move-arranged",
        "notice-copied",
        "notice-dismissed",
        "notice-empty-mouse",
        "notice-closed",
    ] {
        match phase {
            "focus-material" => scene.toggle_focus(&mut damage),
            "focus-calculator" => scene.select(1).unwrap(),
            "work" => scene.toggle_focus(&mut damage),
            "search" => {
                scene.palette_open();
                for byte in b"finder" {
                    scene.palette_insert(*byte);
                }
            }
            "selected" => {
                scene.select(0).unwrap();
                scene.palette_close();
            }
            "window-menu" => {
                for (key, action, alt) in [
                    (PhysicalKey::Space, KeyAction::Press, true),
                    (PhysicalKey::Space, KeyAction::Release, true),
                ] {
                    scene
                        .palette_key(
                            KeyboardEvent {
                                key,
                                action,
                                modifiers: KeyboardModifiers {
                                    left_alt: alt,
                                    ..Default::default()
                                },
                            },
                            &mut damage,
                        )
                        .unwrap();
                }
                assert!(scene.window_menu_is_open());
            }
            "window-menu-next" => {
                for action in [KeyAction::Press, KeyAction::Release] {
                    scene
                        .palette_key(
                            KeyboardEvent {
                                key: PhysicalKey::Down,
                                action,
                                modifiers: Default::default(),
                            },
                            &mut damage,
                        )
                        .unwrap();
                }
            }
            "window-minimized" => {
                for action in [KeyAction::Press, KeyAction::Release] {
                    scene
                        .palette_key(
                            KeyboardEvent {
                                key: PhysicalKey::Escape,
                                action,
                                modifiers: Default::default(),
                            },
                            &mut damage,
                        )
                        .unwrap();
                }
                scene.minimize(0, &mut damage).unwrap();
            }
            "window-empty" => scene.minimize(1, &mut damage).unwrap(),
            "window-restored" => scene.select(0).unwrap(),
            "move-open" => {
                if std::env::var("OS_WINDOW_SHELL_CLIPBOARD").as_deref() == Ok("1") {
                    scene.set_clipboard_notice(nieuw_os_window_session::clipboard::Notice::Empty);
                }
                tap(&mut scene, &mut damage, PhysicalKey::F7, true, false);
            }
            "move-coarse" => {
                tap(&mut scene, &mut damage, PhysicalKey::Right, false, false);
                tap(&mut scene, &mut damage, PhysicalKey::Down, false, false);
            }
            "move-fine" => {
                tap(&mut scene, &mut damage, PhysicalKey::Right, false, true);
                tap(&mut scene, &mut damage, PhysicalKey::Up, false, true);
            }
            "move-committed" => tap(&mut scene, &mut damage, PhysicalKey::Enter, false, false),
            "move-cancel-open" => tap(&mut scene, &mut damage, PhysicalKey::F7, true, false),
            "move-cancel-step" => tap(&mut scene, &mut damage, PhysicalKey::Left, false, false),
            "move-cancelled" => tap(&mut scene, &mut damage, PhysicalKey::Escape, false, false),
            "move-arranged" => tap(&mut scene, &mut damage, PhysicalKey::F8, false, false),
            "notice-copied" => {
                scene.set_clipboard_notice(nieuw_os_window_session::clipboard::Notice::Copied);
                tap(&mut scene, &mut damage, PhysicalKey::N, true, false);
            }
            "notice-dismissed" => tap(&mut scene, &mut damage, PhysicalKey::Delete, false, false),
            "notice-empty-mouse" => {
                tap(&mut scene, &mut damage, PhysicalKey::Escape, false, false);
                let button = scene.clipboard_notice_bounds();
                for down in [true, false] {
                    scene
                        .mouse_with_damage(
                            (button.x + 2) as u32,
                            (button.y + 2) as u32,
                            down,
                            down,
                            &mut damage,
                        )
                        .unwrap();
                }
                scene
                    .mouse_with_damage(width - 20, 50 * scale, false, false, &mut damage)
                    .unwrap();
            }
            "notice-closed" => {
                let button = scene.notification_close_button();
                for down in [true, false] {
                    scene
                        .mouse_with_damage(
                            (button.x + 2) as u32,
                            (button.y + 2) as u32,
                            down,
                            down,
                            &mut damage,
                        )
                        .unwrap();
                }
                scene
                    .mouse_with_damage(width - 20, 50 * scale, false, false, &mut damage)
                    .unwrap();
            }
            _ => unreachable!(),
        }
        let mut pixels = vec![0; (width * height) as usize];
        scene
            .render_reference(Rect::new(0, 0, width, height), &mut pixels, &backing)
            .unwrap();
        let mut file =
            std::fs::File::create_new(directory.join(format!("workspace-{phase}.ppm"))).unwrap();
        write!(file, "P6\n{width} {height}\n255\n").unwrap();
        for pixel in pixels {
            file.write_all(&[(pixel >> 16) as u8, (pixel >> 8) as u8, pixel as u8])
                .unwrap();
        }
        file.sync_all().unwrap();
    }
}

//! Shell-only references. Zero app buffers are not application raster evidence.
use nieuw_os_graphics::Rect;
use nieuw_os_window_session::scene::{Damage, Scene, MAX_PIXELS};
use std::io::Write;
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

#[test]
fn render_workspace_shell_oracles() {
    let Some(directory) = std::env::var_os("OS_WINDOW_FIXTURE_DIRECTORY") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    let backing = vec![0; MAX_PIXELS];
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    scene.pointer = (1260, 50);
    let mut damage = Damage::new(1280, 720).unwrap();
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
            _ => unreachable!(),
        }
        let mut pixels = vec![0; 1280 * 720];
        scene
            .render_reference(Rect::new(0, 0, 1280, 720), &mut pixels, &backing)
            .unwrap();
        let mut file =
            std::fs::File::create_new(directory.join(format!("workspace-{phase}.ppm"))).unwrap();
        file.write_all(b"P6\n1280 720\n255\n").unwrap();
        for pixel in pixels {
            file.write_all(&[(pixel >> 16) as u8, (pixel >> 8) as u8, pixel as u8])
                .unwrap();
        }
        file.sync_all().unwrap();
    }
}

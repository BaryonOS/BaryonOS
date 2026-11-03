//! Host shell oracle composed from captured, real native app pixels.
use nieuw_os_window_session::scene::{Damage, Scene, MAX_PIXELS};
use std::{io::Write, path::Path};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

#[test]
fn generate_switcher_from_native_app_backing_when_requested() {
    let Some(directory) = std::env::var_os("OS_SWITCHER_FIXTURE_DIRECTORY") else {
        return;
    };
    let directory = Path::new(&directory);
    let scale: u32 = std::env::var("OS_SWITCHER_FIXTURE_SCALE")
        .unwrap()
        .parse()
        .unwrap();
    let (width, height) = match scale {
        1 => (1280, 720),
        2 => (1920, 1080),
        _ => panic!("unsupported native profile"),
    };
    let bytes = std::fs::read(directory.join("backing.bin")).unwrap();
    assert_eq!(bytes.len(), MAX_PIXELS * 4);
    let backing: Vec<u32> = bytes
        .chunks_exact(4)
        .map(|p| u32::from_le_bytes(p.try_into().unwrap()))
        .collect();
    assert!(backing.iter().all(|color| color >> 24 == 0));
    let mut scene = Scene::new(width, height, scale, [1, 2]).unwrap();
    let mut damage = Damage::new(width, height).unwrap();
    let key = |physical| KeyboardEvent {
        key: physical,
        action: KeyAction::Press,
        modifiers: KeyboardModifiers {
            left_alt: true,
            ..KeyboardModifiers::default()
        },
    };
    scene
        .switcher_key(key(PhysicalKey::Tab), &mut damage)
        .unwrap();
    for selected in [1, 0] {
        if selected == 0 {
            scene
                .switcher_key(key(PhysicalKey::Left), &mut damage)
                .unwrap();
        }
        assert_eq!(scene.switcher_selected(), Some(selected));
        let r = scene.switcher_bounds();
        let mut pixels = vec![0; (r.width * r.height) as usize];
        scene.render_reference(r, &mut pixels, &backing).unwrap();
        let mut file =
            std::fs::File::create(directory.join(format!("switcher-{selected}.ppm"))).unwrap();
        write!(file, "P6\n{} {}\n255\n", r.width, r.height).unwrap();
        for color in pixels {
            file.write_all(&[(color >> 16) as u8, (color >> 8) as u8, color as u8])
                .unwrap();
        }
        file.sync_all().unwrap();
    }
}

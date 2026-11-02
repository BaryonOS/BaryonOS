//! Native renderer contracts and opt-in host previews. No VM is executed here.
use nieuw_os_graphics::Rect;
use nieuw_os_surface_protocol::Rectangle;
use nieuw_os_window_session::scene::{Scene, MAX_PIXELS};

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

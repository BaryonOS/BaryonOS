//! Exact raster contracts for the native compositor's opaque-body shortcut.
//! The reference is the unchanged full composite path, not a timing benchmark.
use nieuw_os_graphics::Rect;
use nieuw_os_window_session::scene::{Scene, MAX_PIXELS};

fn backing() -> Vec<u32> {
    (0..MAX_PIXELS)
        .map(|i| (i as u32).wrapping_mul(0x045d_9f3b) ^ ((i as u32) >> 8) | 0xab00_0000)
        .collect()
}

fn compare(scene: &Scene, rectangle: Rect, backing: &[u32]) {
    let count = (rectangle.width * rectangle.height) as usize;
    let mut fast = vec![0x8765_4321; count + 4];
    let mut reference = fast.clone();
    let actual = scene.render(rectangle, &mut fast, backing);
    let expected = scene.render_reference(rectangle, &mut reference, backing);
    assert_eq!(actual, expected, "error differs for {rectangle:?}");
    if actual.is_ok() {
        assert_eq!(fast, reference, "pixels differ for {rectangle:?}");
    }
    assert_eq!(&fast[count..], &[0x8765_4321; 4]);
    assert_eq!(&reference[count..], &[0x8765_4321; 4]);
}

#[test]
fn opaque_tiles_match_composite_at_both_scales_focuses_and_overlaps() {
    let pixels = backing();
    for (width, height, scale) in [(1280, 720, 1), (1920, 1080, 2)] {
        for focus in 0..2 {
            for moved in [false, true] {
                let mut scene = Scene::new(width, height, scale, [1, 2]).unwrap();
                if moved {
                    // Overlap the bodies and put their lower rows near the
                    // dock, within the same bounds used by title dragging.
                    scene.windows[1].body.x = scene.windows[0].body.x + 40 * scale as i32;
                    for window in &mut scene.windows {
                        window.body.y = (height - window.body.height - 72 * scale) as i32;
                    }
                }
                scene.select(focus).unwrap();
                let body = scene.windows[focus].body;
                for pointer_on_body in [false, true] {
                    scene.pointer = if pointer_on_body {
                        (body.x as u32 + 20 * scale, body.y as u32 + 20 * scale)
                    } else {
                        (0, 0)
                    };
                    // Interior fast tiles, partial tiles, and each body edge.
                    let rectangles = [
                        Rect::new(body.x + 8, body.y + 8, 32, 8),
                        Rect::new(body.x + 32, body.y + 24, 17, 5),
                        Rect::new(
                            body.x + body.width as i32 - 13,
                            body.y + body.height as i32 - 5,
                            13,
                            5,
                        ),
                        Rect::new(body.x - 1, body.y + 8, 32, 8),
                        Rect::new(body.x + body.width as i32 - 16, body.y + 8, 32, 8),
                        Rect::new(body.x + 32, body.y - 4, 32, 8),
                        Rect::new(body.x + 32, body.y + body.height as i32 - 4, 32, 8),
                        Rect::new(
                            scene.pointer.0 as i32 - 4,
                            scene.pointer.1 as i32 - 4,
                            32,
                            8,
                        ),
                        Rect::new(scene.dock().x, scene.dock().y, 32, 8),
                    ];
                    for rectangle in rectangles {
                        compare(&scene, rectangle, &pixels);
                    }
                    // Exercise every sub-tile position over a patch including
                    // the cursor and, when moved, another window underneath.
                    for y in (0..40).step_by(7) {
                        for x in (0..80).step_by(13) {
                            compare(&scene, Rect::new(body.x + x, body.y + y, 32, 8), &pixels);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn optimized_path_preserves_short_backing_buffer_and_geometry_errors() {
    let pixels = backing();
    let mut scene = Scene::new(1280, 720, 1, [1, 2]).unwrap();
    scene.pointer = (0, 0);
    for focus in 0..2 {
        scene.select(focus).unwrap();
        let window = scene.windows[focus];
        let rectangle = Rect::new(window.body.x + 32, window.body.y + 32, 32, 8);
        let end = window.offset + (window.body.width * window.body.height) as usize;
        compare(&scene, rectangle, &pixels[..end - 1]);
        let mut short = [0; 255];
        let mut reference = short;
        assert!(scene.render(rectangle, &mut short, &pixels).is_err());
        assert_eq!(
            scene.render(rectangle, &mut short, &pixels),
            scene.render_reference(rectangle, &mut reference, &pixels)
        );
        for invalid in [
            Rect::new(0, 0, 0, 8),
            Rect::new(0, 0, 32, 0),
            Rect::new(i32::MAX, 0, 32, 8),
        ] {
            compare(&scene, invalid, &pixels);
        }
    }
}

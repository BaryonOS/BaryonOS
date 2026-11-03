use nieuw_os_graphics::{Canvas, Font, Icon, Rect, Size, Weight};
#[test]
fn buffer_and_geometry_fail_before_writing() {
    let mut pixels = [0xdead_beef; 255];
    assert!(Canvas::new(Rect::new(0, 0, 32, 8), &mut pixels).is_err());
    assert_eq!(pixels, [0xdead_beef; 255]);
    assert!(Canvas::new(Rect::new(i32::MAX, 0, 1, 1), &mut pixels).is_err());
}
fn scene(c: &mut Canvas<'_>) {
    nieuw_os_graphics::theme::wallpaper(
        c,
        Rect::new(0, 0, 64, 64),
        nieuw_os_graphics::theme::MATERIAAL,
    );
    c.soft_shadow(Rect::new(3, 3, 50, 34), 12, 0x202229, 40)
        .unwrap();
    c.rounded_rect(Rect::new(3, 3, 50, 34), 9, 0x16394c);
    c.stroke_rounded_rect(Rect::new(2, 2, 52, 36), 10, 2, 0x91d4c2);
    c.ellipse(Rect::new(-5, 15, 24, 30), 0xe9b889);
    c.line(-15, 40, 61, 4, 2, 0x4b85bc);
    c.icon(Rect::new(32, 12, 24, 24), Icon::Material, 0xffffff, 1);
    c.text(
        2,
        40,
        "Materiaal Café",
        Font::new(Size::Body, Weight::Regular, false),
        0x102030,
    );
}
#[test]
fn tile_rendering_matches_one_scene_without_changing_guard() {
    let mut full = [0; 64 * 64];
    scene(&mut Canvas::new(Rect::new(0, 0, 64, 64), &mut full).unwrap());
    let mut tiles = [0; 64 * 64];
    for y in (0..64).step_by(8) {
        for x in (0..64).step_by(32) {
            let mut tile = [0xabad1dea; 258];
            scene(&mut Canvas::new(Rect::new(x, y, 32, 8), &mut tile[..256]).unwrap());
            assert_eq!(&tile[256..], &[0xabad1dea; 2]);
            for row in 0..8 {
                tiles[((y + row) * 64 + x) as usize..((y + row) * 64 + x + 32) as usize]
                    .copy_from_slice(&tile[row as usize * 32..row as usize * 32 + 32]);
            }
        }
    }
    assert_eq!(full, tiles);
    assert!(full.iter().all(|p| p >> 24 == 0));
}
#[test]
fn extreme_coordinates_and_empty_shapes_are_safe() {
    let mut pixels = [0x123456; 256];
    let mut c = Canvas::new(Rect::new(0, 0, 32, 8), &mut pixels).unwrap();
    c.rounded_rect(
        Rect::new(i32::MIN, i32::MIN, u32::MAX, u32::MAX),
        u32::MAX,
        1,
    );
    c.stroke_rounded_rect(
        Rect::new(i32::MIN, i32::MIN, u32::MAX, u32::MAX),
        u32::MAX,
        u32::MAX,
        1,
    );
    c.ellipse(Rect::new(i32::MIN, i32::MIN, u32::MAX, u32::MAX), 1);
    c.line(i32::MIN, i32::MIN, i32::MAX, i32::MAX, 128, 1);
    c.soft_shadow(Rect::new(i32::MIN, i32::MIN, 1, 1), 64, 1, 255)
        .unwrap();
    c.soft_shadow(
        Rect::new(i32::MAX, i32::MAX, u32::MAX, u32::MAX),
        64,
        1,
        255,
    )
    .unwrap();
    assert!(pixels.iter().all(|p| *p == 0x123456));
}
#[test]
fn invalid_shadow_is_atomic_and_coverage_stays_outside_surface() {
    let mut pixels = [0xffffff; 64 * 64];
    let mut c = Canvas::new(Rect::new(0, 0, 64, 64), &mut pixels).unwrap();
    for (r, spread) in [
        (Rect::new(10, 10, 32, 32), 0),
        (Rect::new(10, 10, 32, 32), 65),
        (Rect::new(10, 10, 0, 32), 12),
    ] {
        assert!(c.soft_shadow(r, spread, 0, 255).is_err());
    }
    assert!(pixels.iter().all(|p| *p == 0xffffff));
    let mut c = Canvas::new(Rect::new(0, 0, 64, 64), &mut pixels).unwrap();
    c.soft_shadow(Rect::new(16, 16, 32, 32), 12, 0, 64).unwrap();
    for y in 0..64 {
        for x in 0..64 {
            let p = pixels[y * 64 + x];
            if (16..48).contains(&x) && (16..48).contains(&y)
                || !(4..60).contains(&x)
                || !(4..60).contains(&y)
            {
                assert_eq!(p, 0xffffff);
            }
        }
    }
    assert!(pixels[20 * 64 + 15] < pixels[20 * 64 + 7]);
    assert!(pixels[20 * 64 + 7] < 0xffffff);
}
#[test]
fn glyphs_and_vectors_have_real_intermediate_coverage() {
    let mut pixels = [0; 64 * 64];
    let mut c = Canvas::new(Rect::new(0, 0, 64, 64), &mut pixels).unwrap();
    c.text(
        0,
        0,
        "Ag",
        Font::new(Size::Title, Weight::Semibold, false),
        0xffffff,
    );
    c.icon(Rect::new(30, 28, 28, 28), Icon::Search, 0xffffff, 2);
    assert!(pixels.iter().any(|p| *p > 0 && *p < 0xffffff));
    assert!(pixels.iter().filter(|p| **p != 0).count() > 100);
}

#[test]
fn shared_shell_text_meets_contrast_through_the_entire_static_gradient() {
    use nieuw_os_graphics::{blend, theme};
    fn luminance(color: u32) -> f64 {
        let linear = |shift: u32| {
            let c = f64::from((color >> shift) & 255u32) / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(16) + 0.7152 * linear(8) + 0.0722 * linear(0)
    }
    let p = theme::MATERIAAL;
    for alpha in 0..=255 {
        let background = blend(p.desktop_end, p.desktop_start, alpha);
        for foreground in [theme::SHELL_TEXT, theme::SHELL_MUTED] {
            let a = luminance(foreground);
            let b = luminance(background);
            assert!((a.max(b) + 0.05) / (a.min(b) + 0.05) >= 4.5);
        }
    }
}

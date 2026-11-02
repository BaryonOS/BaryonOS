use nieuw_os_graphics::{Canvas, Font, Icon, Rect, Size, Weight};
#[test]
fn buffer_and_geometry_fail_before_writing() {
    let mut pixels = [0xdead_beef; 255];
    assert!(Canvas::new(Rect::new(0, 0, 32, 8), &mut pixels).is_err());
    assert_eq!(pixels, [0xdead_beef; 255]);
    assert!(Canvas::new(Rect::new(i32::MAX, 0, 1, 1), &mut pixels).is_err());
}
fn scene(c: &mut Canvas<'_>) {
    c.clear(0xf0f4f8);
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
    assert!(pixels.iter().all(|p| *p == 0x123456));
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

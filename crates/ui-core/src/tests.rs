extern crate std;
use super::*;
use std::vec;

#[test]
fn geometry_rejects_zero_oversize_stride_and_short_buffers() {
    for (w, h, s, n) in [
        (0, 1, 1, 4),
        (1, 0, 1, 4),
        (1921, 1, 1921, 10000),
        (1, 1081, 1, 10000),
        (10, 10, 9, 400),
        (1, 1, u32::MAX, usize::MAX),
        (u32::MAX, u32::MAX, u32::MAX, usize::MAX),
        (2, 2, 3, 23),
    ] {
        assert!(Geometry::new(w, h, s, n, PixelFormat::Rgb).is_err());
    }
    let g = Geometry::new(2, 2, 3, 24, PixelFormat::Rgb).unwrap();
    assert_eq!(
        (g.width(), g.height(), g.stride(), g.required_bytes()),
        (2, 2, 3, 24)
    );
    assert!(Canvas::new(&mut [0; 23], g).is_err());
}
#[test]
fn clipping_preserves_row_padding_and_outer_canaries_in_both_formats() {
    for format in [PixelFormat::Rgb, PixelFormat::Bgr] {
        let g = Geometry::new(3, 2, 5, 40, format).unwrap();
        let mut memory = [0xa5; 56];
        let mut c = Canvas::new(&mut memory[8..48], g).unwrap();
        c.fill_rect(
            Rect {
                x: i32::MIN,
                y: i32::MIN,
                width: u32::MAX,
                height: u32::MAX,
            },
            Color::new(1, 2, 3),
        );
        assert!(!c.pixel(-1, 0, TEXT));
        assert!(!c.pixel(3, 0, TEXT));
        assert!(!c.pixel(0, 2, TEXT));
        assert_eq!(&memory[..8], &[0xa5; 8]);
        assert_eq!(&memory[48..], &[0xa5; 8]);
        for y in 0..2 {
            for x in 0..5 {
                let p = &memory[8 + (y * 5 + x) * 4..12 + (y * 5 + x) * 4];
                let expected = if x < 3 {
                    Color::new(1, 2, 3).encode(format)
                } else {
                    [0xa5; 4]
                };
                assert_eq!(p, &expected);
            }
        }
    }
}
#[test]
fn empty_and_offscreen_rects_leave_canvas_untouched() {
    let g = Geometry::new(2, 2, 2, 16, PixelFormat::Rgb).unwrap();
    let mut bytes = [0x77; 16];
    let mut c = Canvas::new(&mut bytes, g).unwrap();
    for r in [
        Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 2,
        },
        Rect {
            x: 3,
            y: 0,
            width: u32::MAX,
            height: u32::MAX,
        },
        Rect {
            x: -9,
            y: -9,
            width: 2,
            height: 2,
        },
    ] {
        c.fill_rect(r, TEXT);
    }
    assert_eq!(bytes, [0x77; 16]);
}
#[test]
fn original_glyph_has_expected_pixels_and_clips_safely() {
    let g = Geometry::new(5, 7, 5, 140, PixelFormat::Rgb).unwrap();
    let mut bytes = [0; 140];
    Canvas::new(&mut bytes, g)
        .unwrap()
        .text(0, 0, b"A", Color::new(255, 255, 255), 1);
    // Independent geometric landmarks: apex 01110 and full crossbar 11111.
    assert_eq!(&bytes[..4], &[0; 4]);
    assert_eq!(&bytes[4..8], &[255, 255, 255, 0]);
    for x in 0..5 {
        assert_eq!(
            &bytes[(3 * 5 + x) * 4..(3 * 5 + x + 1) * 4],
            &[255, 255, 255, 0]
        );
    }
    let before = bytes;
    Canvas::new(&mut bytes, g)
        .unwrap()
        .text(i32::MAX, i32::MAX, b"AAA", TEXT, 4);
    assert_eq!(bytes, before);
}
#[test]
fn scene_row_and_canvas_keep_padding_and_typed_input_changes_only_input_area() {
    let g = Geometry::new(640, 360, 644, 644 * 360 * 4, PixelFormat::Bgr).unwrap();
    let mut pixels = vec![0xa5; g.required_bytes() + 16];
    let mut line = TextLine::<32>::new();
    draw_material_debug(&mut Canvas::new(&mut pixels, g).unwrap(), &line);
    for y in 0..360 {
        assert!(pixels[(y * 644 + 640) * 4..(y + 1) * 644 * 4]
            .iter()
            .all(|v| *v == 0xa5));
    }
    assert_eq!(&pixels[g.required_bytes()..], &[0xa5; 16]);
    let before = pixels.clone();
    line.apply(Keyboard::new().feed(0x1e).unwrap());
    draw_material_debug(&mut Canvas::new(&mut pixels, g).unwrap(), &line);
    let mut changed = false;
    for y in 0..360 {
        for x in 0..640 {
            let at = (y * 644 + x) * 4;
            if before[at..at + 4] != pixels[at..at + 4] {
                changed = true;
                assert!((48..72).contains(&x));
                assert!((192..212).contains(&y));
            }
        }
    }
    assert!(changed);
    let mut row = [0xcc; MAX_ROW_BYTES + 8];
    let before = row;
    assert_eq!(render_debug_row(g, &line, 360, &mut row), Err(Error::Row));
    assert_eq!(row, before);
    assert_eq!(
        render_debug_row(g, &line, 0, &mut [0; 1]),
        Err(Error::Buffer)
    );
    render_debug_row(g, &line, 0, &mut row).unwrap();
    assert!(row[640 * 4..].iter().all(|v| *v == 0xcc));
}
#[test]
fn release_modifier_chords_and_caps_repeat_do_not_insert_unexpected_text() {
    let mut k = Keyboard::new();
    let mut line = TextLine::<16>::new();
    for b in [
        0x2a, 0x1e, 0x9e, 0xaa, 0x1e, 0x9e, 0x3a, 0x3a, 0xba, 0x1e, 0x9e, 0x2a, 0x1e, 0x9e, 0xaa,
    ] {
        if let Some(e) = k.feed(b) {
            line.apply(e);
        }
    }
    assert_eq!(line.as_str(), "AaAa");
    for b in [0x1d, 0x1e, 0x9e, 0x9d, 0x38, 0x1e, 0x9e, 0xb8] {
        if let Some(e) = k.feed(b) {
            line.apply(e);
        }
    }
    assert_eq!(line.as_str(), "AaAa");
}
#[test]
fn extended_keys_pause_and_printscreen_do_not_leave_fake_shift_pressed() {
    let mut k = Keyboard::new();
    let mut line = TextLine::<8>::new();
    for b in [
        0xe0, 0x2a, 0xe0, 0x37, 0xe0, 0xb7, 0xe0, 0xaa, 0xe1, 0x1d, 0x45, 0xe1, 0x9d, 0xc5, 0x1e,
    ] {
        if let Some(e) = k.feed(b) {
            line.apply(e);
        }
    }
    assert_eq!(line.as_str(), "a");
    assert_eq!(k.feed(0xe0), None);
    assert_eq!(k.feed(0x48).unwrap().code, KeyCode::Up);
    k.feed(0xe0);
    let up = k.feed(0xc8).unwrap();
    assert!(!up.pressed);
    assert_eq!(up.text, None);
    k.feed(0xe0);
    k.feed(0x1d);
    assert!(k.feed(0x1e).unwrap().modifiers.control);
    k.feed(0xe0);
    k.feed(0x9d);
    assert!(!k.feed(0x1e).unwrap().modifiers.control);
}
#[test]
fn both_modifier_sides_are_tracked_independently_and_reset_is_safe() {
    let mut k = Keyboard::new();
    for b in [0x2a, 0x36, 0xaa] {
        k.feed(b);
    }
    assert_eq!(k.feed(0x1e).unwrap().text, Some(b'A'));
    k.reset();
    assert_eq!(k.feed(0x1e).unwrap().text, Some(b'a'));
}
#[test]
fn text_limit_backspace_submit_and_zero_capacity_are_explicit() {
    let mut k = Keyboard::new();
    let a = k.feed(0x1e).unwrap();
    let mut line = TextLine::<2>::new();
    assert_eq!(line.apply(a), EditResult::Changed);
    assert_eq!(line.apply(a), EditResult::Changed);
    assert_eq!(line.apply(a), EditResult::Full);
    assert_eq!(line.as_str(), "aa");
    assert_eq!(line.apply(k.feed(0x0e).unwrap()), EditResult::Changed);
    assert_eq!(line.as_str(), "a");
    assert_eq!(line.apply(k.feed(0x1c).unwrap()), EditResult::Submit);
    assert_eq!(line.as_str(), "a");
    assert_eq!(line.apply(k.feed(0x9e).unwrap()), EditResult::Unchanged);
    assert_eq!(TextLine::<0>::new().apply(a), EditResult::Full);
    for text in [0, 0x1b, 0x7f, 0x80, 0xff] {
        assert_eq!(
            line.apply(KeyEvent {
                text: Some(text),
                ..a
            }),
            EditResult::Unchanged
        );
    }
}
#[test]
fn arbitrary_scancode_streams_never_overflow_or_emit_control_sequences() {
    for first in 0..=255 {
        let mut k = Keyboard::new();
        let mut line = TextLine::<31>::new();
        k.feed(first);
        for second in 0..=255 {
            if let Some(event) = k.feed(second) {
                line.apply(event);
            }
            assert!(line.len() <= 31);
            assert!(line.as_bytes().iter().all(|b| (32..=126).contains(b)));
        }
    }
}

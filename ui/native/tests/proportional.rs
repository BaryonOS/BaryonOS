use nieuw_os_graphics::{Font, Size, Weight};
use nieuw_os_materiaal::text::{Proportional, VisualPosition};

#[test]
fn proportional_caret_accumulates_the_same_unrounded_advance_as_renderer() {
    for double in [false, true] {
        let font = Font::new(Size::Body, Weight::Regular, double);
        let metrics = Proportional::new(font, 1200);
        assert!(font.advance_64('W') > font.advance_64('i'));
        let value = "Wi café Ω fallback";
        let mut expected = 0u64;
        for (byte, ch) in value.char_indices() {
            let p = metrics.position(value, byte);
            assert_eq!(
                p,
                VisualPosition {
                    row: 0,
                    x_64: expected
                }
            );
            assert_eq!(metrics.byte_at(value, p), byte);
            expected += u64::from(font.advance_64(ch));
        }
        assert_eq!(metrics.position(value, value.len()).x_64, expected);
        assert_eq!(font.measure(value), expected.div_ceil(64) as u32);
    }
}

#[test]
fn wrap_hits_clamp_to_original_utf8_boundaries_and_visual_line_ends() {
    for double in [false, true] {
        let font = Font::new(Size::Body, Weight::Regular, double);
        for width in [1, 23, 48, 112, 400] {
            let metrics = Proportional::new(font, width);
            let value = "Wiiii café\tWW\r\né\n\nlast";
            let end = metrics.position(value, value.len());
            for row in 0..=end.row + 2 {
                for x in [0, 1, 25 * 64, 200 * 64, u64::MAX] {
                    let byte = metrics.byte_at(value, VisualPosition { row, x_64: x });
                    assert!(value.is_char_boundary(byte));
                    assert!(byte <= value.len());
                }
            }
            for (byte, ch) in value.char_indices() {
                if matches!(ch, '\r' | '\n') {
                    continue;
                }
                let p = metrics.position(value, byte);
                assert_eq!(metrics.byte_at(value, p), byte);
                let start = metrics.line_boundary(value, byte, false);
                let stop = metrics.line_boundary(value, byte, true);
                assert!(start <= byte && byte <= stop);
            }
            assert_eq!(
                metrics.byte_at(
                    value,
                    VisualPosition {
                        row: usize::MAX,
                        x_64: 0
                    }
                ),
                value.len()
            );
        }
    }
}

#[test]
fn crlf_is_one_break_and_tabs_use_four_space_stops() {
    let font = Font::new(Size::Body, Weight::Regular, false);
    let metrics = Proportional::new(font, 1200);
    let tab = u64::from(font.advance_64(' ')) * 4;
    assert_eq!(metrics.position("i\tX", 2).x_64, tab);
    assert_eq!(
        metrics.position("a\r\nb", 3),
        VisualPosition { row: 1, x_64: 0 }
    );
    assert_eq!(
        metrics.byte_at(
            "a\nb",
            VisualPosition {
                row: 0,
                x_64: u64::MAX
            }
        ),
        1
    );
    assert_eq!(metrics.position("", usize::MAX), VisualPosition::default());
}

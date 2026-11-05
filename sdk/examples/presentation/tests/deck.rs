use nieuw_os_graphics::theme::MATERIAAL;
use nieuw_os_presentation::{Deck, Error, HEIGHT, MAX_SLIDES, WIDTH};

const SOURCE: &str = include_str!("../fixtures/deck.txt");

#[test]
fn import_select_next_previous_preserve_the_exact_utf8_source() {
    let source = "Eerste titel\nCaf\u{e9}, originele tekst.\n---\nTweede titel\n漢字";
    let mut deck = Deck::parse(source).unwrap();
    assert_eq!(deck.count(), 2);
    assert_eq!(deck.selected(), 0);
    assert!(!deck.previous_slide());
    assert_eq!(deck.slide().body, "Caf\u{e9}, originele tekst.");
    assert!(deck.next_slide());
    assert_eq!(deck.slide().title, "Tweede titel");
    assert!(!deck.next_slide());
    assert_eq!(deck.select(2), Err(Error::Selection));
    assert_eq!(deck.selected(), 1);
    assert!(deck.previous_slide());
    assert_eq!(deck.source().as_ptr(), source.as_ptr());
    assert_eq!(deck.source(), source);
}

#[test]
fn real_shared_font_and_canvas_render_both_pages_at_both_scales() {
    let mut deck = Deck::parse(SOURCE).unwrap();
    for scale in [1, 2] {
        let width = WIDTH * scale;
        let mut pixels = vec![0xdeadbeef; WIDTH * HEIGHT * scale * scale + 1];
        deck.select(0).unwrap();
        deck.render(&mut pixels, scale).unwrap();
        assert_eq!(pixels[0], MATERIAAL.surface);
        assert_eq!(pixels[(88 * scale) * width + 32 * scale], MATERIAAL.accent);
        assert_eq!(*pixels.last().unwrap(), 0xdeadbeef);
        let first = pixels.clone();
        deck.next_slide();
        deck.render(&mut pixels, scale).unwrap();
        assert_ne!(pixels, first);
        assert!(
            pixels
                .iter()
                .filter(|pixel| **pixel != MATERIAAL.surface)
                .count()
                > 500
        );
        deck.previous_slide();
        deck.render(&mut pixels, scale).unwrap();
        assert_eq!(pixels, first);
    }
}

#[test]
fn geometry_and_complete_text_preflight_fail_before_modifying_pixels() {
    let mut pixels = vec![0x123456; WIDTH * HEIGHT];
    let deck = Deck::parse(SOURCE).unwrap();
    assert_eq!(deck.render(&mut pixels[..100], 1), Err(Error::Buffer));
    assert_eq!(deck.render(&mut pixels, 0), Err(Error::Scale));
    assert_eq!(deck.render(&mut pixels, usize::MAX), Err(Error::Scale));
    let wide = "W".repeat(256);
    let deck = Deck::parse(&wide).unwrap();
    assert_eq!(deck.render(&mut pixels, 1), Err(Error::TextOverflow));
    let tall = format!("Title\n{}", ["line"; 12].join("\n"));
    let deck = Deck::parse(&tall).unwrap();
    assert_eq!(deck.render(&mut pixels, 1), Err(Error::TextOverflow));
    assert!(pixels.iter().all(|pixel| *pixel == 0x123456));
}

#[test]
fn malformed_or_oversized_decks_are_never_silently_truncated() {
    for source in [
        "",
        "\nBody",
        "---",
        "Title\n---\n",
        "Title\r\nBody",
        "Title\0Body",
        "Title\tBody",
    ] {
        assert!(Deck::parse(source).is_err(), "{source:?}");
    }
    assert!(matches!(
        Deck::parse(&"a".repeat(32 * 1024 + 1)),
        Err(Error::InputLimit)
    ));
    assert!(matches!(
        Deck::parse(&"a".repeat(257)),
        Err(Error::ContentLimit)
    ));
    assert!(matches!(
        Deck::parse(&format!("Title\n{}", "b".repeat(513))),
        Err(Error::ContentLimit)
    ));
    assert_eq!(
        Deck::parse(&["Title"; MAX_SLIDES].join("\n---\n"))
            .unwrap()
            .count(),
        MAX_SLIDES
    );
    assert!(matches!(
        Deck::parse(&["Title"; MAX_SLIDES + 1].join("\n---\n")),
        Err(Error::SlideLimit)
    ));
}

#[test]
fn missing_glyph_is_visible_and_source_bytes_remain_unchanged() {
    let unknown = Deck::parse("漢").unwrap();
    let replacement = Deck::parse("\u{fffd}").unwrap();
    let mut first = vec![0; WIDTH * HEIGHT];
    let mut second = first.clone();
    unknown.render(&mut first, 1).unwrap();
    replacement.render(&mut second, 1).unwrap();
    assert_eq!(first, second);
    assert_eq!(unknown.source(), "漢");
}

use nieuw_os_calculator::{
    decimal::{Kind, SCALE},
    model::{Command, Model, Notice},
    render::{self, Layout, Rect, Viewport},
};
use nieuw_os_graphics::{Font, Size, Weight};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};
fn type_text(model: &mut Model, text: &str) {
    for byte in text.bytes() {
        model.command(Command::Insert(byte));
    }
}
#[test]
fn clipboard_paste_rejects_whole_invalid_values_without_partial_edit_or_evaluation() {
    use nieuw_os_calculator::model::PasteError;
    let mut model = Model::new();
    type_text(&mut model, "2+3");
    model.command(Command::Evaluate);
    model.command(Command::SelectAll);
    let before = (
        model.expression().to_owned(),
        model.caret(),
        model.selection(),
        model.answer(),
        model.current(),
        model.evaluations(),
    );
    for (value, error) in [
        ("123x456", PasteError::Unsupported),
        ("12\n3", PasteError::Unsupported),
        ("é", PasteError::Unsupported),
        ("", PasteError::Empty),
    ] {
        assert_eq!(model.paste_clipboard(value), Err(error));
        assert_eq!(
            (
                model.expression().to_owned(),
                model.caret(),
                model.selection(),
                model.answer(),
                model.current(),
                model.evaluations()
            ),
            before
        );
    }
    assert_eq!(model.clipboard_selection(), "2+3");
    model.paste_clipboard("6/2").unwrap();
    assert_eq!(model.expression(), "6/2");
    assert_eq!(model.caret(), 3);
    assert!(model.selection().is_empty());
    assert!(!model.current());
    assert_eq!(model.evaluations(), 1, "pasting never evaluates");
    assert_eq!(
        model.answer(),
        before.3,
        "old answer retained, marked stale"
    );
    assert_eq!(
        model.clipboard_selection(),
        "",
        "no whole-expression or answer fallback"
    );
}
#[test]
fn clipboard_paste_capacity_is_checked_before_replacing_selection_and_requires_field_focus() {
    use nieuw_os_calculator::model::PasteError;
    let mut model = Model::new();
    type_text(&mut model, &"1".repeat(256));
    model.command(Command::Left(true));
    let selection = model.selection();
    assert_eq!(model.paste_clipboard("22"), Err(PasteError::TooLong));
    assert_eq!(model.expression(), "1".repeat(256));
    assert_eq!(model.selection(), selection);
    model.paste_clipboard("2").unwrap();
    assert_eq!(model.expression(), format!("{}2", "1".repeat(255)));
    model.command(Command::SelectAll);
    model.command(Command::Tab(false));
    assert_eq!(model.clipboard_selection(), "");
    assert_eq!(model.paste_clipboard("3"), Err(PasteError::Inactive));
    assert_eq!(model.expression().len(), 256);
}
fn key(model: &mut Model, key: PhysicalKey, shift: bool) {
    model.keyboard(KeyboardEvent {
        key,
        action: KeyAction::Press,
        modifiers: KeyboardModifiers {
            left_shift: shift,
            ..Default::default()
        },
    });
}
#[test]
fn error_keeps_expression_caret_and_previous_result_until_explicit_reset() {
    let mut m = Model::new();
    type_text(&mut m, "1+2");
    m.command(Command::Evaluate);
    assert_eq!(m.answer().unwrap().value.raw(), 3 * SCALE);
    m.command(Command::SelectAll);
    type_text(&mut m, "7/0");
    let caret = m.caret();
    m.command(Command::Evaluate);
    assert_eq!(m.expression(), "7/0");
    assert_eq!(m.caret(), caret);
    assert!(!m.current());
    assert_eq!(m.answer().unwrap().value.raw(), 3 * SCALE);
    assert!(matches!(m.notice(),Notice::Arithmetic(e)if e.kind==Kind::DivideByZero));
    m.command(Command::Backspace);
    type_text(&mut m, "2");
    m.command(Command::Evaluate);
    assert_eq!(m.answer().unwrap().value.raw(), 3_500_000_000);
    m.command(Command::Cancel);
    assert_eq!(m.expression(), "");
    assert!(!m.current());
    assert!(m.answer().is_some());
    m.command(Command::Reset);
    assert!(m.answer().is_none());
    assert_eq!(m.caret(), 0);
}
#[test]
fn long_input_selection_and_editing_are_atomic_and_recoverable() {
    let mut m = Model::new();
    type_text(&mut m, &"1".repeat(256));
    m.command(Command::Insert(b'2'));
    assert_eq!(m.expression(), "1".repeat(256));
    assert_eq!(m.caret(), 256);
    assert_eq!(m.notice(), Notice::TooLong);
    m.command(Command::SelectAll);
    m.command(Command::Insert(b'7'));
    assert_eq!(m.expression(), "7");
    assert_eq!(m.notice(), Notice::None);
    type_text(&mut m, "+42");
    m.command(Command::Home(false));
    m.command(Command::Right(false));
    m.command(Command::Right(true));
    m.command(Command::Delete);
    assert_eq!(m.expression(), "742");
    m.command(Command::Home(false));
    m.command(Command::Delete);
    assert_eq!(m.expression(), "42");
    m.command(Command::End(false));
    m.command(Command::Left(true));
    m.command(Command::Insert(b'3'));
    assert_eq!(m.expression(), "43");
}
#[test]
fn real_keyboard_semantics_include_modifiers_keypad_focus_and_press_only_actions() {
    let mut m = Model::new();
    key(&mut m, PhysicalKey::Digit2, false);
    key(&mut m, PhysicalKey::Equal, true);
    key(&mut m, PhysicalKey::Digit3, false);
    key(&mut m, PhysicalKey::Enter, false);
    assert_eq!(m.expression(), "2+3");
    assert_eq!(m.answer().unwrap().value.raw(), 5 * SCALE);
    let calls = m.evaluations();
    for action in [KeyAction::Release, KeyAction::Repeat] {
        m.keyboard(KeyboardEvent {
            key: PhysicalKey::Enter,
            action,
            modifiers: Default::default(),
        });
    }
    assert_eq!(m.evaluations(), calls);
    key(&mut m, PhysicalKey::Tab, true);
    assert_eq!(m.focus(), 20);
    key(&mut m, PhysicalKey::Tab, false);
    assert_eq!(m.focus(), 0);
    m.command(Command::Reset);
    m.keyboard(KeyboardEvent {
        key: PhysicalKey::Keypad7,
        action: KeyAction::Press,
        modifiers: KeyboardModifiers {
            num_lock: true,
            ..Default::default()
        },
    });
    assert_eq!(m.expression(), "7");
    m.keyboard(KeyboardEvent {
        key: PhysicalKey::A,
        action: KeyAction::Press,
        modifiers: KeyboardModifiers {
            left_control: true,
            ..Default::default()
        },
    });
    key(&mut m, PhysicalKey::Digit4, false);
    assert_eq!(m.expression(), "4");
}
#[test]
fn pointer_capture_does_not_activate_after_outside_cancel_or_focus_loss() {
    let mut m = Model::new();
    m.pointer(Some(0), true, false);
    m.cancel_pointer();
    m.pointer(Some(0), false, true);
    assert_eq!(m.expression(), "");
    assert_eq!(m.hover(), Some(0));
    m.pointer(Some(1), true, false);
    m.cancel_focus();
    m.pointer(Some(1), false, true);
    assert_eq!(m.expression(), "");
    m.pointer(Some(2), true, false);
    m.pointer(Some(2), false, true);
    assert_eq!(m.expression(), "9");
    m.pointer(Some(usize::MAX), true, false);
    m.pointer(None, false, true);
    assert_eq!(m.expression(), "9");
    m.command(Command::Reset);
    type_text(&mut m, "12345");
    m.caret_at(1, true, true);
    m.caret_at(4, false, true);
    m.caret_at(4, false, false);
    assert_eq!(m.selection(), 1..4);
    m.command(Command::Insert(b'9'));
    assert_eq!(m.expression(), "195");
}
#[test]
fn every_control_is_inside_both_scales_and_hit_testing_never_clamps() {
    for (w, h, s) in [
        (320, 240, 1),
        (640, 608, 2),
        (768, 536, 1),
        (640, 480, 2),
        (768, 536, 2),
        (1920, 1080, 2),
    ] {
        let v = Viewport::new(w, h, s).unwrap();
        let l = Layout::new(v);
        for i in 0..20 {
            let b = l.button(i);
            assert!(b.width >= 27 && b.height >= 16);
            assert!(b.x + b.width <= w / s && b.y + b.height <= h / s);
            assert_eq!(l.hit(b.x + b.width / 2, b.y + b.height / 2), Some(i));
        }
        assert_eq!(l.hit(u32::MAX, u32::MAX), None);
    }
    assert!(Viewport::new(319, 240, 1).is_none());
    assert!(Viewport::new(639, 480, 2).is_none());
    assert!(Viewport::new(800, 600, 0).is_none());
}
#[test]
fn renderer_is_bounded_tracks_real_state_and_scrolls_long_input_at_both_scales() {
    for scale in [1, 2] {
        let v = Viewport::new(320 * scale, 240 * scale, scale).unwrap();
        let mut m = Model::new();
        let mut before = 0u64;
        let mut after = 0u64;
        let checksum = |m: &Model| {
            let mut sum = 0u64;
            for y in (0..v.height()).step_by(8) {
                for x in (0..v.width()).step_by(32) {
                    let r = Rect {
                        x,
                        y,
                        width: 32.min(v.width() - x),
                        height: 8.min(v.height() - y),
                    };
                    let mut pixels = [0xfeed_beef; 258];
                    render::render_tile(m, v, r, &mut pixels[1..257]).unwrap();
                    assert_eq!(pixels[0], 0xfeed_beef);
                    assert_eq!(pixels[257], 0xfeed_beef);
                    for p in &pixels[1..1 + (r.width * r.height) as usize] {
                        assert_eq!(p >> 24, 0);
                        sum = sum.wrapping_mul(1099511628211) ^ u64::from(*p);
                    }
                }
            }
            sum
        };
        before = before.wrapping_add(checksum(&m));
        type_text(&mut m, "(12+3)*4");
        m.command(Command::Evaluate);
        after = after.wrapping_add(checksum(&m));
        assert_ne!(before, after);
        type_text(&mut m, &"+1".repeat(100));
        let l = Layout::new(v);
        let start = render::expression_start(&m, l, scale);
        assert!(start > 0 && start < m.caret());
        let _ = checksum(&m);
        let mut pixels = [0xabcdef; 256];
        assert!(render::render_tile(
            &m,
            v,
            Rect {
                x: v.width(),
                y: 0,
                width: 1,
                height: 1
            },
            &mut pixels
        )
        .is_err());
        assert!(pixels.iter().all(|p| *p == 0xabcdef));
    }
}

fn image(model: &Model, view: Viewport) -> Vec<u32> {
    let mut pixels = vec![0; (view.width() * view.height()) as usize];
    for y in (0..view.height()).step_by(8) {
        for x in (0..view.width()).step_by(32) {
            let rect = Rect {
                x,
                y,
                width: 32.min(view.width() - x),
                height: 8.min(view.height() - y),
            };
            let mut tile = [0; 256];
            render::render_tile(model, view, rect, &mut tile).unwrap();
            for row in 0..rect.height {
                let from = (row * rect.width) as usize;
                let to = ((rect.y + row) * view.width() + rect.x) as usize;
                pixels[to..to + rect.width as usize]
                    .copy_from_slice(&tile[from..from + rect.width as usize]);
            }
        }
    }
    pixels
}

#[test]
fn proportional_formula_hit_testing_and_caret_follow_exact_visible_advances() {
    for scale in [1, 2] {
        let view = Viewport::new(320 * scale, 484 * scale, scale).unwrap();
        let layout = Layout::new(view);
        let font = Font::new(Size::Body, Weight::Regular, scale == 2);
        assert_ne!(font.advance('.'), font.advance('1'));
        let mut model = Model::new();
        type_text(&mut model, "1.1+(8/2)");
        let mut advance = 0u64;
        for caret in 0..=model.expression().len() {
            model.command(Command::Home(false));
            for _ in 0..caret {
                model.command(Command::Right(false));
            }
            assert_eq!(render::expression_start(&model, layout, scale), 0);
            let physical = (layout.expression.x + 8) * scale + (advance / 64) as u32;
            let logical = (physical + scale / 2) / scale;
            assert_eq!(
                render::expression_caret_at(&model, layout, scale, logical),
                caret
            );
            if caret < model.expression().len() {
                advance += u64::from(font.advance(model.expression().as_bytes()[caret] as char));
            }
        }
        assert_eq!(render::expression_caret_at(&model, layout, scale, 0), 0);
        assert_eq!(
            render::expression_caret_at(&model, layout, scale, u32::MAX),
            model.expression().len()
        );
        model.command(Command::Reset);
        type_text(&mut model, &"1.".repeat(128));
        let start = render::expression_start(&model, layout, scale);
        assert!(start > 0 && start < model.caret());
        assert_eq!(
            render::expression_caret_at(&model, layout, scale, layout.expression.x),
            start
        );
        assert_eq!(
            render::expression_caret_at(&model, layout, scale, u32::MAX),
            256
        );
        model.command(Command::Home(false));
        let end = render::expression_caret_at(&model, layout, scale, u32::MAX);
        assert!(end > 0 && end < 256);
    }
}

#[test]
fn selected_narrow_glyph_and_caret_use_same_pixel_edges_at_both_scales() {
    for scale in [1, 2] {
        let view = Viewport::new(320 * scale, 484 * scale, scale).unwrap();
        let layout = Layout::new(view);
        let font = Font::new(Size::Body, Weight::Regular, scale == 2);
        let mut model = Model::new();
        type_text(&mut model, "1.1");
        model.command(Command::Home(false));
        model.command(Command::Right(false));
        model.command(Command::Right(true));
        assert_eq!(model.selection(), 1..2);
        let pixels = image(&model, view);
        let origin = (layout.expression.x + 8) * scale;
        let left = origin + font.advance('1') / 64;
        let right = origin + (font.advance('1') + font.advance('.')) / 64;
        let row = (layout.expression.y + 4) * scale;
        for x in left..right {
            assert_eq!(
                pixels[(row * view.width() + x) as usize],
                render::design::ACCENT_SOFT
            );
        }
        assert_eq!(
            pixels[(row * view.width() + right) as usize],
            render::design::TEXT
        );
        assert_eq!(
            pixels[(row * view.width() + left - 1) as usize],
            render::design::BACKGROUND
        );
    }
}

#[test]
fn longest_signed_answer_fits_with_actual_body_raster_at_both_scene_sizes() {
    use nieuw_os_graphics::{Canvas, Rect as GraphicRect};
    for (width, height, scale) in [(320, 484, 1), (640, 608, 2)] {
        let view = Viewport::new(width, height, scale).unwrap();
        let layout = Layout::new(view);
        let mut model = Model::new();
        type_text(&mut model, "-9999999999,999999999");
        model.command(Command::Evaluate);
        let answer = model.answer().unwrap().value.format();
        let body = Font::new(Size::Body, Weight::Semibold, scale == 2);
        let title = Font::new(Size::Title, Weight::Semibold, scale == 2);
        assert!(title.measure(answer.as_str()) > layout.width * scale);
        assert!(body.measure(answer.as_str()) <= layout.width * scale);
        let pixels = image(&model, view);
        let region = GraphicRect::new(
            (layout.x * scale) as i32,
            (layout.result_y * scale) as i32,
            layout.width * scale,
            body.height(),
        );
        let mut expected =
            vec![render::design::BACKGROUND; (region.width * region.height) as usize];
        Canvas::new(region, &mut expected).unwrap().text(
            region.x + region.width as i32 - body.measure(answer.as_str()) as i32,
            region.y,
            answer.as_str(),
            body,
            render::design::TEXT,
        );
        for row in 0..region.height {
            let actual = ((region.y as u32 + row) * width + region.x as u32) as usize;
            let from = (row * region.width) as usize;
            assert_eq!(
                &pixels[actual..actual + region.width as usize],
                &expected[from..from + region.width as usize]
            );
        }
    }
}

#[test]
fn paper_calculator_host_fixtures_use_the_real_bounded_renderer() {
    use std::io::Write;
    let Some(directory) = std::env::var_os("OS_CALCULATOR_RENDER_FIXTURE_DIRECTORY") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    for (width, height, scale) in [(320, 484, 1), (640, 608, 2), (640, 968, 2), (320, 240, 1)] {
        let view = Viewport::new(width, height, scale).unwrap();
        let mut model = Model::new();
        type_text(&mut model, "12,5 * (8 - 3)");
        model.command(Command::Evaluate);
        let pixels = image(&model, view);
        let path = directory.join(format!("calculator-{width}x{height}-s{scale}.ppm"));
        let mut file = std::fs::File::create_new(path).unwrap();
        write!(file, "P6\n{width} {height}\n255\n").unwrap();
        for color in pixels {
            file.write_all(&[(color >> 16) as u8, (color >> 8) as u8, color as u8])
                .unwrap();
        }
        file.sync_all().unwrap();
    }
}

fn assert_damage_covers_changes(before: &Model, after: &Model, view: Viewport) {
    let damage = after.paint_damage(view);
    for y in (0..view.height()).step_by(8) {
        for x in (0..view.width()).step_by(32) {
            let tile = Rect {
                x,
                y,
                width: 32.min(view.width() - x),
                height: 8.min(view.height() - y),
            };
            if !damage.intersects(x, y, tile.width, tile.height) {
                let mut old = [0; 256];
                let mut new = [0; 256];
                render::render_tile(before, view, tile, &mut old).unwrap();
                render::render_tile(after, view, tile, &mut new).unwrap();
                assert!(old == new, "changed pixels outside damage at {tile:?}");
            }
        }
    }
}

#[test]
fn damage_covers_selection_scroll_results_notices_and_each_button_transition() {
    for (width, height, scale) in [(320, 240, 1), (640, 480, 2), (768, 536, 1)] {
        let view = Viewport::new(width, height, scale).unwrap();
        let mut model = Model::new();
        assert!(model.presented(model.generation()));
        for command in [
            Command::Insert(b'2'),
            Command::Evaluate,
            Command::Insert(b'3'),
            Command::Home(false),
            Command::Right(true),
            Command::End(true),
            Command::Delete,
            Command::Evaluate,
            Command::Reset,
            Command::Tab(false),
            Command::Activate,
            Command::Tab(false),
            Command::Tab(true),
            Command::Tab(true),
        ] {
            let before = model.clone();
            model.command(command);
            assert_damage_covers_changes(&before, &model, view);
            assert!(model.presented(model.generation()));
        }
        for index in 0..20 {
            for (down, up) in [(false, false), (true, false), (false, true)] {
                let before = model.clone();
                model.pointer(Some(index), down, up);
                assert_damage_covers_changes(&before, &model, view);
                assert!(model.presented(model.generation()));
            }
        }
        for transition in 0..8 {
            let before = model.clone();
            match transition {
                0 => type_text(&mut model, &"1+".repeat(70)),
                1 => model.command(Command::Home(false)),
                2 => model.command(Command::End(true)),
                3 => model.cancel_focus(),
                4 => model.focus_gained(),
                5 => model.clipboard_notice(Notice::ClipboardUnavailable),
                6 => model.input_lost(),
                _ => model.command(Command::Delete),
            }
            assert_damage_covers_changes(&before, &model, view);
            assert!(model.presented(model.generation()));
        }
    }
}

#[test]
fn superseded_frames_and_rebind_cannot_acknowledge_away_pending_damage() {
    for scale in [1, 2] {
        let view = Viewport::new(320 * scale, 240 * scale, scale).unwrap();
        let mut model = Model::new();
        assert!(model.presented(model.generation()));
        let base = model.clone();
        model.pointer(Some(0), false, false);
        let partial = model.clone();
        let old_generation = model.generation();
        let button = Layout::new(view).button(0);
        assert!(model.paint_damage(view).intersects(
            button.x * scale,
            button.y * scale,
            button.width * scale,
            button.height * scale
        ));
        model.pointer(None, false, false);
        model.command(Command::Insert(b'8'));
        assert!(!model.presented(old_generation));
        assert!(!model.presented(model.generation() + 1));
        assert_damage_covers_changes(&base, &model, view);
        assert_damage_covers_changes(&partial, &model, view);
        assert!(model.paint_damage(view).intersects(
            button.x * scale,
            button.y * scale,
            button.width * scale,
            button.height * scale
        ));
        assert!(!model.paint_damage(view).intersects(0, 0, 1, 1));
        assert!(model.presented(model.generation()));
        assert!(!model
            .paint_damage(view)
            .intersects(0, 0, view.width(), view.height()));
        let previous = model.generation();
        model.force_full_repaint();
        assert!(!model.presented(previous));
        let full = model.paint_damage(view);
        assert!(full.intersects(0, 0, 1, 1));
        assert!(full.intersects(view.width() - 1, view.height() - 1, 1, 1));
        assert!(!full.intersects(view.width(), 0, 1, 1));
        assert!(!full.intersects(u32::MAX, u32::MAX, u32::MAX, u32::MAX));
        assert!(!full.intersects(0, 0, 0, 1));
        assert!(model.presented(model.generation()));
    }
}

#[test]
fn expression_hit_testing_uses_the_rendered_scroll_origin_at_both_scales() {
    for scale in [1, 2] {
        let view = Viewport::new(320 * scale, 240 * scale, scale).unwrap();
        let layout = Layout::new(view);
        let mut model = Model::new();
        type_text(&mut model, &"1234567890".repeat(12));
        let step = render::character_width(scale);
        let left = layout.expression.x + 8;
        let start = render::expression_start(&model, layout, scale);
        assert!(start > 0);
        for column in 0..5 {
            assert_eq!(
                render::expression_caret_at(&model, layout, scale, left + column * step),
                start + column as usize
            );
        }

        assert_eq!(render::expression_caret_at(&model, layout, scale, 0), start);
        assert_eq!(
            render::expression_caret_at(&model, layout, scale, left + step - 1),
            start + 1
        );
        assert_eq!(
            render::expression_caret_at(&model, layout, scale, u32::MAX),
            model.expression().len()
        );
        model.command(Command::Home(false));
        assert_eq!(render::expression_start(&model, layout, scale), 0);
        assert_eq!(render::expression_caret_at(&model, layout, scale, left), 0);
        model.command(Command::Reset);
        assert_eq!(
            render::expression_caret_at(&model, layout, scale, u32::MAX),
            0
        );
    }
}

#[test]
fn reverted_partial_scroll_and_noop_focus_regain_still_repaint_changed_pixels() {
    for scale in [1, 2] {
        let view = Viewport::new(320 * scale, 240 * scale, scale).unwrap();
        let mut model = Model::new();
        type_text(&mut model, &"1234567890".repeat(12));
        assert!(model.presented(model.generation()));
        model.command(Command::Home(false));
        let partial = model.clone();
        let superseded = model.generation();
        model.command(Command::End(false));
        assert!(!model.presented(superseded));
        assert_damage_covers_changes(&partial, &model, view);
        assert!(model.presented(model.generation()));
        for pointer in [true, false] {
            model.cancel_focus();
            assert!(model.presented(model.generation()));
            let inactive = model.clone();
            if pointer {
                model.pointer(None, false, false);
            } else {
                model.command(Command::Delete);
            }
            assert!(model.active());
            assert!(model.generation() > inactive.generation());
            assert_damage_covers_changes(&inactive, &model, view);
            assert!(model.presented(model.generation()));
        }
    }
}

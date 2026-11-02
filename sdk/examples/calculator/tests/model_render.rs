use nieuw_os_calculator::{
    decimal::{Kind, SCALE},
    model::{Command, Model, Notice},
    render::{self, Layout, Rect, Viewport},
};
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

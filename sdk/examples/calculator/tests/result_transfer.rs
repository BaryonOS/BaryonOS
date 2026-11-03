use nieuw_os_calculator::{
    model::{Command, Model, PasteError},
    render::{self, Rect, Viewport},
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

fn calculate(model: &mut Model, expression: &str) {
    model.command(Command::SelectAll);
    model.paste_clipboard(expression).unwrap();
    model.command(Command::Evaluate);
}

fn select_result(model: &mut Model, action: KeyAction) {
    model.keyboard(KeyboardEvent {
        key: PhysicalKey::R,
        action,
        modifiers: KeyboardModifiers {
            left_control: true,
            ..Default::default()
        },
    });
}

#[test]
fn current_result_requires_explicit_selection_and_copies_decimal_text_not_the_formula() {
    let mut model = Model::new();
    calculate(&mut model, "(12+3)/2");
    assert_eq!(model.clipboard_selection(), "");
    select_result(&mut model, KeyAction::Press);
    assert!(model.result_selected());
    assert_eq!(model.clipboard_selection(), "7,5");
    assert_eq!(model.expression(), "(12+3)/2");
    assert_eq!(model.evaluations(), 1);
    assert_eq!(model.paste_clipboard("9"), Err(PasteError::Inactive));
    assert_eq!(model.expression(), "(12+3)/2");
    model.command(Command::SelectAll);
    assert!(!model.result_selected());
    assert_eq!(model.clipboard_selection(), "(12+3)/2");
}

#[test]
fn stale_error_reset_and_focus_loss_never_leave_an_answer_copy_fallback() {
    let mut model = Model::new();
    model.command(Command::SelectResult);
    assert!(!model.result_selected());
    calculate(&mut model, "2+3");
    model.command(Command::SelectResult);
    assert_eq!(model.clipboard_selection(), "5");
    model.command(Command::Insert(b'4'));
    assert!(!model.result_selected());
    assert!(!model.current());
    model.command(Command::SelectResult);
    assert_eq!(model.clipboard_selection(), "");
    calculate(&mut model, "1/0");
    model.command(Command::SelectResult);
    assert_eq!(model.clipboard_selection(), "");
    calculate(&mut model, "7/2");
    model.command(Command::SelectResult);
    model.cancel_focus();
    assert!(!model.result_selected());
    assert_eq!(model.clipboard_selection(), "");
    model.focus_gained();
    assert_eq!(model.clipboard_selection(), "");
    model.command(Command::SelectResult);
    model.command(Command::Reset);
    assert_eq!(model.clipboard_selection(), "");
    assert!(model.answer().is_none());
}

#[test]
fn result_selection_cancels_pending_button_and_does_not_follow_repeat_or_modified_shortcuts() {
    let mut model = Model::new();
    calculate(&mut model, "9/2");
    for action in [KeyAction::Repeat, KeyAction::Release] {
        select_result(&mut model, action);
        assert!(!model.result_selected());
    }
    for modifiers in [
        KeyboardModifiers {
            left_control: true,
            left_shift: true,
            ..Default::default()
        },
        KeyboardModifiers {
            left_control: true,
            left_alt: true,
            ..Default::default()
        },
        KeyboardModifiers {
            left_control: true,
            left_super: true,
            ..Default::default()
        },
    ] {
        model.keyboard(KeyboardEvent {
            key: PhysicalKey::R,
            action: KeyAction::Press,
            modifiers,
        });
        assert!(!model.result_selected());
    }
    model.pointer(Some(0), true, false);
    select_result(&mut model, KeyAction::Press);
    assert!(model.result_selected());
    assert_eq!(model.pressed(), None);
    model.pointer(Some(0), false, true);
    assert_eq!(model.expression(), "9/2");
    assert_eq!(model.clipboard_selection(), "4,5");
    model.caret_at(1, true, true);
    assert!(!model.result_selected());
    assert_eq!(model.clipboard_selection(), "");
}

#[test]
fn negative_and_rounded_result_text_matches_the_displayed_decimal_value() {
    for (expression, expected) in [
        ("-7/2", "-3,5"),
        ("1/3", "0,333333333"),
        ("0", "0"),
        ("9999999999,999999999", "9999999999,999999999"),
    ] {
        let mut model = Model::new();
        calculate(&mut model, expression);
        model.command(Command::SelectResult);
        assert_eq!(model.clipboard_selection(), expected);
        assert_eq!(
            model.clipboard_selection(),
            model.answer().unwrap().value.format().as_str()
        );
        assert_eq!(model.evaluations(), 1);
    }
}

#[test]
fn complete_result_selection_and_clear_pixels_are_inside_retained_damage_at_both_scales() {
    for (width, height, scale) in [(320, 240, 1), (320, 484, 1), (640, 608, 2)] {
        let view = Viewport::new(width, height, scale).unwrap();
        let mut model = Model::new();
        calculate(&mut model, "7/2");
        for command in [Command::SelectResult, Command::Home(false)] {
            let before = model.clone();
            assert!(model.presented(model.generation()));
            model.command(command);
            let damage = model.paint_damage(view);
            let mut count = 0;
            for y in (0..height).step_by(8) {
                for x in (0..width).step_by(32) {
                    let rect = Rect {
                        x,
                        y,
                        width: (width - x).min(32),
                        height: (height - y).min(8),
                    };
                    let mut old = [0; 256];
                    let mut new = [0; 256];
                    render::render_tile(&before, view, rect, &mut old).unwrap();
                    render::render_tile(&model, view, rect, &mut new).unwrap();
                    for at in 0..(rect.width * rect.height) as usize {
                        if old[at] != new[at] {
                            count += 1;
                            assert!(damage.intersects(
                                x + at as u32 % rect.width,
                                y + at as u32 / rect.width,
                                1,
                                1
                            ));
                        }
                    }
                }
            }
            assert!(count > 0);
        }
    }
}

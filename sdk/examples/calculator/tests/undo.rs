use nieuw_os_calculator::{
    decimal::SCALE,
    model::{Command, Model, PasteError, UNDO_CAPACITY},
    render::{self, Rect, Viewport},
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

fn text(model: &mut Model, value: &str) {
    for byte in value.bytes() {
        model.command(Command::Insert(byte));
    }
}

fn shortcut(model: &mut Model, key: PhysicalKey, shift: bool, action: KeyAction) {
    model.keyboard(KeyboardEvent {
        key,
        action,
        modifiers: KeyboardModifiers {
            left_control: true,
            left_shift: shift,
            ..Default::default()
        },
    });
}

#[test]
fn keyboard_undo_redo_restores_selection_without_reusing_or_evaluating_answers() {
    let mut model = Model::new();
    text(&mut model, "12+3");
    model.command(Command::Evaluate);
    let answer = model.answer();
    model.command(Command::Home(false));
    model.command(Command::Right(false));
    model.command(Command::Right(true));
    model.command(Command::Insert(b'8'));
    assert_eq!(model.expression(), "18+3");
    shortcut(&mut model, PhysicalKey::Z, false, KeyAction::Press);
    assert_eq!(model.expression(), "12+3");
    assert_eq!(model.selection(), 1..2);
    assert_eq!(model.caret(), 2);
    assert!(!model.current(), "undo is text restoration, not evaluation");
    assert_eq!(model.answer(), answer);
    assert_eq!(model.evaluations(), 1);
    shortcut(&mut model, PhysicalKey::Z, true, KeyAction::Press);
    assert_eq!(model.expression(), "18+3");
    assert_eq!(model.selection(), 2..2);
    assert_eq!(model.evaluations(), 1);
    shortcut(&mut model, PhysicalKey::Z, false, KeyAction::Press);
    shortcut(&mut model, PhysicalKey::Y, false, KeyAction::Press);
    assert_eq!(model.expression(), "18+3");
    model.command(Command::Evaluate);
    assert_eq!(model.answer().unwrap().value.raw(), 21 * SCALE);
}

#[test]
fn whole_paste_is_one_transaction_and_rejected_paste_preserves_redo() {
    let mut model = Model::new();
    text(&mut model, "9+2");
    model.command(Command::SelectAll);
    model.paste_clipboard("123,45 / (2+3)").unwrap();
    model.command(Command::Undo);
    assert_eq!(model.expression(), "9+2");
    assert_eq!(model.selection(), 0..3);
    for value in ["", "4\n5", "é", &"1".repeat(257)] {
        assert!(model.paste_clipboard(value).is_err());
        assert!(model.can_redo());
        assert_eq!(model.expression(), "9+2");
    }
    model.command(Command::Redo);
    assert_eq!(model.expression(), "123,45 / (2+3)");
    model.command(Command::Undo);
    model.command(Command::Insert(b'7'));
    assert!(!model.can_redo(), "a new edit abandons the old redo branch");
    model.command(Command::Redo);
    assert_eq!(model.expression(), "7");
}

#[test]
fn deletion_clear_and_escape_can_be_reversed_but_empty_edits_do_not_fill_history() {
    let mut model = Model::new();
    model.command(Command::Backspace);
    model.command(Command::Delete);
    model.command(Command::Reset);
    model.command(Command::Cancel);
    assert!(!model.can_undo());
    text(&mut model, "42");
    for command in [
        Command::Backspace,
        Command::Delete,
        Command::Reset,
        Command::Cancel,
    ] {
        model.command(Command::Home(false));
        if command == Command::Backspace {
            model.command(Command::End(false));
        }
        let caret = model.caret();
        model.command(command);
        assert_ne!(model.expression(), "42");
        model.command(Command::Undo);
        assert_eq!(model.expression(), "42");
        assert_eq!(model.caret(), caret);
        model.command(Command::Redo);
        assert_ne!(model.expression(), "42");
        model.command(Command::Undo);
    }
}

#[test]
fn history_is_bounded_and_repeated_roundtrips_preserve_the_entire_available_tail() {
    let mut model = Model::new();
    text(&mut model, &"1".repeat(UNDO_CAPACITY + 9));
    for _ in 0..3 {
        for _ in 0..UNDO_CAPACITY {
            assert!(model.can_undo());
            model.command(Command::Undo);
        }
        assert_eq!(model.expression(), "1".repeat(9));
        assert!(!model.can_undo());
        model.command(Command::Undo);
        assert_eq!(model.expression(), "1".repeat(9));
        for _ in 0..UNDO_CAPACITY {
            assert!(model.can_redo());
            model.command(Command::Redo);
        }
        assert!(!model.can_redo());
        assert_eq!(model.expression(), "1".repeat(UNDO_CAPACITY + 9));
    }
}

#[test]
fn navigation_evaluation_rejected_input_and_focus_loss_do_not_create_edits() {
    let mut model = Model::new();
    model.paste_clipboard("7/0").unwrap();
    model.command(Command::Evaluate);
    model.command(Command::SelectAll);
    model.command(Command::Home(false));
    model.command(Command::Insert(b'x'));
    model.cancel_focus();
    assert_eq!(model.paste_clipboard("8"), Err(PasteError::Inactive));
    model.focus_gained();
    model.command(Command::Undo);
    assert_eq!(model.expression(), "");
    assert!(!model.can_undo());
    assert_eq!(model.failures(), 1);
    model.command(Command::Redo);
    assert_eq!(model.expression(), "7/0");
    assert_eq!(model.failures(), 1, "redo does not rerun arithmetic");
}

#[test]
fn repeats_releases_alt_super_and_shift_y_cannot_consume_history() {
    let mut model = Model::new();
    text(&mut model, "12");
    for action in [KeyAction::Repeat, KeyAction::Release] {
        shortcut(&mut model, PhysicalKey::Z, false, action);
        assert_eq!(model.expression(), "12");
    }
    for modifiers in [
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
            key: PhysicalKey::Z,
            action: KeyAction::Press,
            modifiers,
        });
        assert_eq!(model.expression(), "12");
    }
    shortcut(&mut model, PhysicalKey::Z, false, KeyAction::Press);
    assert_eq!(model.expression(), "1");
    shortcut(&mut model, PhysicalKey::Y, true, KeyAction::Press);
    assert_eq!(model.expression(), "1");
    for action in [KeyAction::Repeat, KeyAction::Release] {
        shortcut(&mut model, PhysicalKey::Y, false, action);
        assert_eq!(model.expression(), "1");
    }
}

#[test]
fn undo_from_a_button_cancels_the_old_pointer_release_and_returns_to_the_formula() {
    let mut model = Model::new();
    text(&mut model, "12");
    model.pointer(Some(0), true, false);
    assert_eq!(model.pressed(), Some(0));
    shortcut(&mut model, PhysicalKey::Z, false, KeyAction::Press);
    assert_eq!(model.expression(), "1");
    assert_eq!(model.focus(), 0);
    assert_eq!(model.pressed(), None);
    model.pointer(Some(0), false, true);
    assert_eq!(
        model.expression(),
        "1",
        "stale pointer release cannot add a digit"
    );
}

fn frame(model: &Model, view: Viewport) -> Vec<u32> {
    let mut image = vec![0; (view.width() * view.height()) as usize];
    for y in (0..view.height()).step_by(8) {
        for x in (0..view.width()).step_by(32) {
            let rect = Rect {
                x,
                y,
                width: (view.width() - x).min(32),
                height: (view.height() - y).min(8),
            };
            let mut tile = [0; 256];
            render::render_tile(model, view, rect, &mut tile).unwrap();
            for row in 0..rect.height {
                let at = ((y + row) * view.width() + x) as usize;
                let from = (row * rect.width) as usize;
                image[at..at + rect.width as usize]
                    .copy_from_slice(&tile[from..from + rect.width as usize]);
            }
        }
    }
    image
}

#[test]
fn retained_damage_covers_undo_redo_scroll_selection_and_history_hints_at_both_scales() {
    for (width, height, scale) in [(320, 240, 1), (320, 484, 1), (640, 608, 2)] {
        let view = Viewport::new(width, height, scale).unwrap();
        let mut model = Model::new();
        model.paste_clipboard(&"1234567890".repeat(12)).unwrap();
        model.command(Command::Home(false));
        model.command(Command::Right(true));
        model.paste_clipboard("9").unwrap();
        for command in [Command::Undo, Command::Undo, Command::Redo, Command::Redo] {
            let before = frame(&model, view);
            assert!(model.presented(model.generation()));
            model.command(command);
            let after = frame(&model, view);
            let damage = model.paint_damage(view);
            for (at, (old, new)) in before.iter().zip(&after).enumerate() {
                if old != new {
                    assert!(
                        damage.intersects(at as u32 % width, at as u32 / width, 1, 1),
                        "missing damage for {command:?} at {at}, scale {scale}"
                    );
                }
            }
            assert_ne!(before, after);
        }
    }
}

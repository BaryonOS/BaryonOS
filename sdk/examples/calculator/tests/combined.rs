//! Recall (#184), undo/redo (#186) and result transfer (#192) on one model.
use nieuw_os_calculator::{
    decimal::SCALE,
    model::{Command, Model},
    render::{self, Rect, Viewport},
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

fn text(model: &mut Model, value: &str) {
    model.command(Command::SelectAll);
    model.paste_clipboard(value).unwrap();
}

fn calculate(model: &mut Model, value: &str) {
    text(model, value);
    model.command(Command::Evaluate);
    assert!(model.current());
}

fn key(model: &mut Model, key: PhysicalKey, control: bool) {
    model.keyboard(KeyboardEvent {
        key,
        action: KeyAction::Press,
        modifiers: KeyboardModifiers {
            left_control: control,
            ..Default::default()
        },
    });
}

#[test]
fn undo_while_recalling_returns_to_the_exact_unfinished_draft_and_redo_to_the_recall() {
    let mut model = Model::new();
    calculate(&mut model, "2+3");
    text(&mut model, "9/2");
    model.command(Command::Left(true));
    let (selection, caret) = (model.selection(), model.caret());
    key(&mut model, PhysicalKey::Up, false);
    assert_eq!(model.expression(), "2+3");
    assert!(model.recalling());
    key(&mut model, PhysicalKey::Z, true);
    assert!(!model.recalling(), "undo always leaves recall");
    assert_eq!(model.expression(), "9/2");
    assert_eq!((model.selection(), model.caret()), (selection, caret));
    key(&mut model, PhysicalKey::Y, true);
    assert_eq!(model.expression(), "2+3");
    // Down no longer browses: the draft is not restored a second time.
    key(&mut model, PhysicalKey::Down, false);
    assert_eq!(model.expression(), "2+3");
    key(&mut model, PhysicalKey::Z, true);
    assert_eq!(model.expression(), "9/2");
    assert_eq!(model.evaluations(), 1);
    assert!(!model.current());
    assert_eq!(model.answer().unwrap().value.raw(), 5 * SCALE);
}

#[test]
fn editing_or_evaluating_a_recalled_formula_keeps_the_draft_reachable_by_undo() {
    let mut model = Model::new();
    calculate(&mut model, "2+3");
    text(&mut model, "42");
    key(&mut model, PhysicalKey::Up, false);
    model.command(Command::Insert(b'4'));
    assert_eq!(model.expression(), "2+34");
    model.command(Command::Undo);
    assert_eq!(model.expression(), "2+3");
    model.command(Command::Undo);
    assert_eq!(model.expression(), "42");

    let mut model = Model::new();
    calculate(&mut model, "4*5");
    text(&mut model, "7-");
    key(&mut model, PhysicalKey::Up, false);
    model.command(Command::Evaluate);
    assert_eq!(model.answer().unwrap().value.raw(), 20 * SCALE);
    assert_eq!(model.recent_count(), 1, "adjacent duplicate is not added");
    model.command(Command::Undo);
    assert_eq!(model.expression(), "7-");
    assert!(!model.current());
}

#[test]
fn redo_while_recalling_keeps_an_existing_redo_branch() {
    let mut model = Model::new();
    calculate(&mut model, "2+3");
    text(&mut model, "1");
    model.command(Command::Insert(b'2'));
    assert_eq!(model.expression(), "12");
    key(&mut model, PhysicalKey::Z, true);
    assert_eq!(model.expression(), "1");
    assert!(model.can_redo());
    key(&mut model, PhysicalKey::Up, false);
    assert_eq!(model.expression(), "2+3");
    key(&mut model, PhysicalKey::Y, true);
    assert!(!model.recalling());
    assert_eq!(model.expression(), "12", "redo continues from the draft");
    key(&mut model, PhysicalKey::Z, true);
    assert_eq!(model.expression(), "1");
}

#[test]
fn redo_while_recalling_without_redo_branch_still_leaves_recall_visibly() {
    let mut model = Model::new();
    calculate(&mut model, "2+3");
    text(&mut model, "1");
    key(&mut model, PhysicalKey::Up, false);
    assert!(model.recalling());
    let generation = model.generation();
    key(&mut model, PhysicalKey::Y, true);
    assert!(!model.recalling());
    assert_eq!(model.expression(), "2+3");
    assert!(model.generation() > generation);
    assert_ne!(model.status(), "Omlaag: volgende of eigen invoer");
}

#[test]
fn recall_and_undo_drop_result_selection_and_a_stale_result_cannot_be_selected() {
    let mut model = Model::new();
    calculate(&mut model, "7/2");
    key(&mut model, PhysicalKey::R, true);
    assert!(model.result_selected());
    assert_eq!(model.clipboard_selection(), "3,5");
    assert_eq!(model.status(), "Resultaat: Ctrl+C om te kopiëren");
    key(&mut model, PhysicalKey::Up, false);
    assert!(!model.result_selected());
    assert_eq!(model.expression(), "7/2");
    key(&mut model, PhysicalKey::R, true);
    assert!(!model.result_selected(), "recall marks the answer stale");

    let mut model = Model::new();
    calculate(&mut model, "7/2");
    key(&mut model, PhysicalKey::R, true);
    key(&mut model, PhysicalKey::Z, true);
    assert!(!model.result_selected());
    assert!(!model.current());
    key(&mut model, PhysicalKey::R, true);
    assert!(!model.result_selected());
    assert_eq!(model.clipboard_selection(), "");
}

#[test]
fn one_status_line_prefers_the_most_relevant_hint() {
    let mut model = Model::new();
    assert_eq!(model.status(), "");
    text(&mut model, "2+3");
    assert_eq!(model.status(), "Ctrl+Z: ongedaan");
    model.command(Command::Evaluate);
    assert_eq!(model.status(), "Klik resultaat of Ctrl+R: selecteren");
    model.command(Command::SelectResult);
    assert_eq!(model.status(), "Resultaat: Ctrl+C om te kopiëren");
    model.command(Command::Cancel);
    assert_eq!(model.status(), "Omhoog: eerdere berekening");
    model.command(Command::RecallOlder);
    assert_eq!(model.status(), "Omlaag: volgende of eigen invoer");
    model.command(Command::RecallNewer);
    assert_eq!(model.status(), "Omhoog: eerdere berekening");
    model.command(Command::Undo);
    assert_eq!(model.expression(), "2+3");
    assert_eq!(model.status(), "Ctrl+Z: ongedaan | Ctrl+Y: opnieuw");
    calculate(&mut model, "1/3");
    assert_eq!(model.status(), "Afgerond op 9 decimalen.");
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
fn combined_status_and_formula_changes_stay_inside_retained_damage_at_both_scales() {
    for (width, height, scale) in [(320, 240, 1), (320, 484, 1), (640, 608, 2)] {
        let view = Viewport::new(width, height, scale).unwrap();
        let mut model = Model::new();
        text(&mut model, "2+3");
        for command in [
            Command::Evaluate,
            Command::SelectResult,
            Command::Cancel,
            Command::RecallOlder,
            Command::Undo,
            Command::Redo,
            Command::RecallNewer,
            Command::Insert(b'1'),
            Command::RecallOlder,
            Command::RecallNewer,
            Command::Undo,
        ] {
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
        }
    }
}

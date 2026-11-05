use nieuw_os_calculator::{
    decimal::SCALE,
    model::{Command, Model, RECENT_CAPACITY},
    render::{self, Rect, Viewport},
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

fn text(model: &mut Model, value: &str) {
    model.command(Command::SelectAll);
    model.paste_clipboard(value).unwrap();
}

fn key(model: &mut Model, key: PhysicalKey) {
    model.keyboard(KeyboardEvent {
        key,
        action: KeyAction::Press,
        modifiers: KeyboardModifiers::default(),
    });
}

fn calculate(model: &mut Model, value: &str) {
    text(model, value);
    model.command(Command::Evaluate);
    assert!(model.current());
}

#[test]
fn arrows_recall_successful_calculations_and_return_to_the_exact_unfinished_selection() {
    let mut model = Model::new();
    calculate(&mut model, "12+3");
    calculate(&mut model, "4*5");
    text(&mut model, "6/2+");
    model.command(Command::Left(true));
    let selection = model.selection();
    let caret = model.caret();
    let evaluations = model.evaluations();
    key(&mut model, PhysicalKey::Up);
    assert_eq!(model.expression(), "4*5");
    assert!(!model.current());
    assert!(model.recalling());
    key(&mut model, PhysicalKey::Up);
    assert_eq!(model.expression(), "12+3");
    key(&mut model, PhysicalKey::Up);
    assert_eq!(model.expression(), "12+3", "oldest entry does not wrap");
    key(&mut model, PhysicalKey::Down);
    assert_eq!(model.expression(), "4*5");
    key(&mut model, PhysicalKey::Down);
    assert_eq!(model.expression(), "6/2+");
    assert_eq!(model.selection(), selection);
    assert_eq!(model.caret(), caret);
    assert!(!model.recalling());
    assert_eq!(model.evaluations(), evaluations);
    assert_eq!(model.answer().unwrap().value.raw(), 20 * SCALE);
    assert!(
        !model.current(),
        "the recalled formula never inherits a cached answer"
    );
}

#[test]
fn editing_recalled_work_becomes_a_new_draft_and_explicit_enter_makes_a_new_calculation() {
    let mut model = Model::new();
    calculate(&mut model, "2+3");
    text(&mut model, "42");
    key(&mut model, PhysicalKey::Up);
    model.command(Command::Insert(b'4'));
    assert_eq!(model.expression(), "2+34");
    assert!(!model.recalling());
    key(&mut model, PhysicalKey::Up);
    assert_eq!(model.expression(), "2+3");
    key(&mut model, PhysicalKey::Down);
    assert_eq!(
        model.expression(),
        "2+34",
        "the edited draft replaces the old draft"
    );
    key(&mut model, PhysicalKey::Enter);
    assert_eq!(model.answer().unwrap().value.raw(), 36 * SCALE);
    assert_eq!(model.evaluations(), 2);
    assert_eq!(model.recent_count(), 2);
}

#[test]
fn recent_calculations_are_bounded_success_only_and_adjacent_duplicates_are_not_added() {
    let mut model = Model::new();
    for n in 1..=RECENT_CAPACITY + 3 {
        calculate(&mut model, &n.to_string());
        model.command(Command::Evaluate);
    }
    assert_eq!(model.recent_count(), RECENT_CAPACITY);
    text(&mut model, "1/0");
    model.command(Command::Evaluate);
    assert_eq!(model.recent_count(), RECENT_CAPACITY);
    for n in (4..=RECENT_CAPACITY + 3).rev() {
        key(&mut model, PhysicalKey::Up);
        assert_eq!(model.expression(), n.to_string());
    }
    key(&mut model, PhysicalKey::Up);
    assert_eq!(model.expression(), "4");
    assert_eq!(model.failures(), 1);
}

#[test]
fn empty_history_button_focus_and_modified_or_released_arrows_do_not_replace_input() {
    let mut model = Model::new();
    text(&mut model, "12");
    for key in [PhysicalKey::Up, PhysicalKey::Down] {
        model.keyboard(KeyboardEvent {
            key,
            action: KeyAction::Press,
            modifiers: Default::default(),
        });
    }
    assert_eq!(model.expression(), "12");
    calculate(&mut model, "2+3");
    text(&mut model, "42");
    model.command(Command::Tab(false));
    key(&mut model, PhysicalKey::Up);
    assert_eq!(model.expression(), "42");
    assert!(!model.recalling());
    model.command(Command::Tab(true));
    for modifiers in [
        KeyboardModifiers {
            left_shift: true,
            ..Default::default()
        },
        KeyboardModifiers {
            left_control: true,
            ..Default::default()
        },
        KeyboardModifiers {
            left_alt: true,
            ..Default::default()
        },
        KeyboardModifiers {
            left_super: true,
            ..Default::default()
        },
    ] {
        model.keyboard(KeyboardEvent {
            key: PhysicalKey::Up,
            action: KeyAction::Press,
            modifiers,
        });
        assert_eq!(model.expression(), "42");
    }
    for action in [KeyAction::Repeat, KeyAction::Release] {
        model.keyboard(KeyboardEvent {
            key: PhysicalKey::Up,
            action,
            modifiers: Default::default(),
        });
        assert_eq!(model.expression(), "42");
    }
}

#[test]
fn focus_loss_preserves_the_unfinished_draft_but_clear_exits_recall() {
    let mut model = Model::new();
    calculate(&mut model, "1+2");
    text(&mut model, "9*");
    key(&mut model, PhysicalKey::Up);
    model.cancel_focus();
    model.focus_gained();
    key(&mut model, PhysicalKey::Down);
    assert_eq!(model.expression(), "9*");
    key(&mut model, PhysicalKey::Up);
    model.command(Command::Cancel);
    key(&mut model, PhysicalKey::Down);
    assert_eq!(model.expression(), "");
    assert!(!model.recalling());
    assert_eq!(model.recent_count(), 1);
}

#[test]
fn recalled_selection_scroll_and_status_are_covered_by_retained_damage_at_both_scales() {
    for (width, height, scale) in [(320, 240, 1), (320, 484, 1), (640, 608, 2)] {
        let view = Viewport::new(width, height, scale).unwrap();
        let mut model = Model::new();
        calculate(&mut model, "12+3");
        calculate(&mut model, "4*5");
        text(&mut model, &"1234567890".repeat(12));
        model.command(Command::Left(true));
        for keycode in [
            PhysicalKey::Up,
            PhysicalKey::Up,
            PhysicalKey::Down,
            PhysicalKey::Down,
        ] {
            let before = model.clone();
            assert!(model.presented(model.generation()));
            key(&mut model, keycode);
            let damage = model.paint_damage(view);
            let mut differences = 0;
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
                            differences += 1;
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
            assert!(
                differences > 0,
                "recall must repaint real formula/status pixels"
            );
        }
    }
}

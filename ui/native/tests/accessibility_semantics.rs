//! U04 host slice: semantic-tree checks on the real controller at 100 % and
//! 200 %. Private-RAM host evidence only; no screen reader, TTS, native QEMU
//! run or human test, and no U04 acceptance.
#[allow(dead_code)]
mod common;
use common::*;
use nieuw_os_document_session::{volume::Buffers, EditMode};
use nieuw_os_input_core::Event;
use nieuw_os_materiaal::{
    controller::{Controller, DocumentTool, InputEnvelope, Screen, Status},
    render::{Frame, Role, SemanticActions, SemanticId, SemanticNode, MAX_SEMANTIC_NODES},
    Scale, Viewport,
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

/// Same logical layout (960 x 540) at both rendered scales.
const VIEWPORTS: [(u32, u32, Scale); 2] = [
    (960, 540, Scale::Percent100),
    (1920, 1080, Scale::Percent200),
];

fn run(f: fn()) {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap();
}

#[derive(Clone, Debug)]
struct Node {
    id: SemanticId,
    role: Role,
    label: String,
    enabled: bool,
    focused: bool,
    actions: SemanticActions,
    read_only: bool,
}

fn nodes(ui: &Controller<'_, Port>) -> Vec<Node> {
    let mut raw = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
    let count = Frame::new(ui.view()).semantics(&mut raw).unwrap();
    raw[..count]
        .iter()
        .map(|n| Node {
            id: n.id,
            role: n.role,
            label: n.label.to_owned(),
            enabled: n.enabled,
            focused: n.focused,
            actions: n.actions,
            read_only: n.read_only,
        })
        .collect()
}

fn is_control(n: &Node) -> bool {
    n.actions.activate
        || n.actions.set_value
        || matches!(n.role, Role::Button | Role::ListItem | Role::TextEditor)
}

fn key(ui: &mut Controller<'_, Port>, seq: &mut u64, physical: PhysicalKey, shift: bool) {
    *seq += 1;
    let modifiers = KeyboardModifiers {
        left_shift: shift,
        ..Default::default()
    };
    ui.deliver(InputEnvelope {
        epoch: INPUT_EPOCH,
        sequence: *seq,
        generation: 1,
        event: Event::Key(KeyboardEvent {
            key: physical,
            action: KeyAction::Press,
            modifiers,
        }),
    })
    .unwrap();
}

fn focused(ui: &Controller<'_, Port>) -> SemanticId {
    let focused: Vec<_> = nodes(ui).into_iter().filter(|n| n.focused).collect();
    assert_eq!(focused.len(), 1, "exactly one focused node: {focused:?}");
    focused[0].id
}

/// Labels, single focus, and a Tab / Shift-Tab cycle that wraps without a
/// trap. With `strict`, Tab must also follow semantic (reading) order.
fn check_screen(ui: &mut Controller<'_, Port>, seq: &mut u64, name: &str, strict: bool) {
    let tree = nodes(ui);
    for node in tree.iter().filter(|n| n.enabled && is_control(n)) {
        assert!(
            !node.label.trim().is_empty(),
            "{name}: enabled control {:?} has no label",
            node.id
        );
    }
    let start = focused(ui);
    // Semantic order of the controls keyboard focus can reach.
    let order: Vec<SemanticId> = tree
        .iter()
        .filter(|n| n.enabled && (n.actions.activate || n.actions.set_value))
        .map(|n| n.id)
        .collect();
    let at = order
        .iter()
        .position(|id| *id == start)
        .unwrap_or_else(|| panic!("{name}: focus {start:?} not a control in {order:?}"));
    let screen = ui.view().screen;
    let mut forward = Vec::new();
    for _ in 0..order.len() {
        key(ui, seq, PhysicalKey::Tab, false);
        assert_eq!(ui.view().screen, screen, "{name}: Tab left the screen");
        forward.push(focused(ui));
    }
    // Wrap: after one full cycle focus is back at its start, every enabled
    // control was reached exactly once, and nothing else took focus.
    assert_eq!(forward.last(), Some(&start), "{name}: Tab did not wrap");
    let mut visited = forward.clone();
    visited.sort_by_key(|id| format!("{id:?}"));
    visited.dedup();
    let mut controls = order.clone();
    controls.sort_by_key(|id| format!("{id:?}"));
    assert_eq!(
        visited, controls,
        "{name}: Tab cycle {forward:?} vs controls {order:?}"
    );
    let mut backward = Vec::new();
    for _ in 0..order.len() {
        key(ui, seq, PhysicalKey::Tab, true);
        assert_eq!(
            ui.view().screen,
            screen,
            "{name}: Shift-Tab left the screen"
        );
        backward.push(focused(ui));
    }
    let mut reverse: Vec<_> = forward[..order.len() - 1].to_vec();
    reverse.reverse();
    reverse.push(start);
    assert_eq!(
        backward, reverse,
        "{name}: Shift-Tab is not the reverse of Tab"
    );
    if strict {
        let expected: Vec<_> = (1..=order.len())
            .map(|i| order[(at + i) % order.len()])
            .collect();
        assert_eq!(
            forward, expected,
            "{name}: Tab order differs from semantic order"
        );
    }
}

fn seeded() -> std::rc::Rc<std::cell::RefCell<Host>> {
    let host = Host::new();
    host.borrow_mut().seed_named(0, b"Reis", b"Earlier");
    host.borrow_mut().seed_named(0, b"Reis", b"Current");
    host.borrow_mut().seed_named(1, b"Dagboek", b"Notities");
    host
}

#[derive(Clone, Copy)]
enum Target {
    Material,
    Editor,
    Rename,
    Organize,
    History,
}

/// Drive the real controller to `target` at each scale and run the checks.
fn screen(target: Target, strict: bool) {
    for (w, h, scale) in VIEWPORTS {
        let host = seeded();
        let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
        let mut ui = controller(
            &host,
            EditMode::ReadWrite,
            Buffers {
                draft: &mut d,
                original: &mut o,
                staging: &mut s,
            },
        );
        ui.resize(Viewport::new(w, h, scale, 2).unwrap()).unwrap();
        ui.refresh().unwrap();
        assert_eq!(ui.view().screen, Screen::Material);
        let mut seq = 0;
        let (name, expected) = match target {
            Target::Material => ("material", Screen::Material),
            Target::Editor => ("editor", Screen::Editor),
            Target::Rename => ("rename", Screen::Rename),
            Target::Organize => ("organize", Screen::Organize),
            Target::History => ("history", Screen::History),
        };
        if !matches!(target, Target::Material) {
            ui.open(0).unwrap();
        }
        match target {
            Target::Rename => ui.begin_rename().unwrap(),
            Target::Organize => ui.begin_organize().unwrap(),
            Target::History => ui.begin_history().unwrap(),
            _ => {}
        }
        assert_eq!(ui.view().screen, expected);
        check_screen(&mut ui, &mut seq, &format!("{name} {scale:?}"), strict);
    }
}

#[test]
fn material_screen_labels_focus_and_wrapping_tab_cycle() {
    run(|| screen(Target::Material, false));
}
#[test]
fn material_screen_tab_order_matches_semantic_order() {
    run(|| screen(Target::Material, true));
}
#[test]
fn editor_screen_labels_focus_and_wrapping_tab_cycle() {
    run(|| screen(Target::Editor, false));
}
#[test]
fn editor_screen_tab_order_matches_semantic_order() {
    run(|| screen(Target::Editor, true));
}
#[test]
fn rename_screen_labels_focus_and_wrapping_tab_cycle() {
    run(|| screen(Target::Rename, false));
}
#[test]
fn rename_screen_tab_order_matches_semantic_order() {
    run(|| screen(Target::Rename, true));
}
#[test]
fn organize_screen_labels_focus_and_wrapping_tab_cycle() {
    run(|| screen(Target::Organize, false));
}
#[test]
fn organize_screen_tab_order_matches_semantic_order() {
    run(|| screen(Target::Organize, true));
}
#[test]
fn history_screen_labels_focus_and_wrapping_tab_cycle() {
    run(|| screen(Target::History, false));
}
#[test]
fn history_screen_tab_order_matches_semantic_order() {
    run(|| screen(Target::History, true));
}

fn document(ui: &Controller<'_, Port>) -> Node {
    nodes(ui)
        .into_iter()
        .find(|n| n.id == SemanticId::Document)
        .unwrap()
}

#[test]
fn read_only_text_editor_reports_read_only_and_offers_no_value_action() {
    run(|| {
        for (w, h, scale) in VIEWPORTS {
            for read_tool in [false, true] {
                let host = seeded();
                let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
                let mut ui = controller(
                    &host,
                    if read_tool {
                        EditMode::ReadWrite
                    } else {
                        EditMode::ReadOnly
                    },
                    Buffers {
                        draft: &mut d,
                        original: &mut o,
                        staging: &mut s,
                    },
                );
                ui.resize(Viewport::new(w, h, scale, 2).unwrap()).unwrap();
                ui.open(0).unwrap();
                if read_tool {
                    let editable = document(&ui);
                    assert_eq!(editable.role, Role::TextEditor);
                    assert!(!editable.read_only);
                    assert!(editable.actions.set_value);
                    ui.switch_tool().unwrap();
                    assert_eq!(ui.view().tool, DocumentTool::Read);
                }
                let node = document(&ui);
                assert_eq!(node.role, Role::TextEditor, "{scale:?} {read_tool}");
                assert!(node.read_only, "{scale:?} read tool {read_tool}");
                assert!(!node.actions.set_value, "{scale:?} read tool {read_tool}");
                assert!(!node.label.trim().is_empty());
            }
        }
    });
}

#[test]
fn status_node_text_changes_when_a_save_fails() {
    run(|| {
        for (w, h, scale) in VIEWPORTS {
            let host = seeded();
            let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
            let mut ui = controller(
                &host,
                EditMode::ReadWrite,
                Buffers {
                    draft: &mut d,
                    original: &mut o,
                    staging: &mut s,
                },
            );
            ui.resize(Viewport::new(w, h, scale, 2).unwrap()).unwrap();
            ui.open(0).unwrap();
            ui.insert("Mine ").unwrap();
            let status = |ui: &Controller<'_, Port>| {
                nodes(ui)
                    .into_iter()
                    .find(|n| n.id == SemanticId::Status)
                    .map(|n| (n.role, n.label))
                    .unwrap()
            };
            let (role, before) = status(&ui);
            assert_eq!(role, Role::Status);
            assert!(!before.trim().is_empty());
            host.borrow_mut().disk.fail = true;
            assert!(ui
                .save()
                .and_then(|()| {
                    while !ui.advance_save()? {}
                    Ok(())
                })
                .is_err());
            assert!(matches!(ui.view().status, Status::Failed(_)));
            let (_, after) = status(&ui);
            assert!(!after.trim().is_empty());
            assert_ne!(before, after, "{scale:?}: status text unchanged on error");
        }
    });
}

/// Reading order of two control rectangles: controls whose vertical extents
/// overlap share a row band and read left to right; otherwise the higher one
/// reads first.
fn reads_before(a: &SemanticNode<'_>, b: &SemanticNode<'_>) -> bool {
    let (a, b) = (a.bounds, b.bounds);
    if a.y < b.y + b.height && b.y < a.y + a.height {
        a.x < b.x
    } else {
        a.y < b.y
    }
}

/// A node Tab can land on when enabled; a read-only text editor is one.
fn focusable(role: Role, actions: SemanticActions) -> bool {
    actions.activate
        || actions.set_value
        || matches!(role, Role::Button | Role::ListItem | Role::TextEditor)
}

fn controls<'a>(raw: &'a [SemanticNode<'a>]) -> Vec<&'a SemanticNode<'a>> {
    raw.iter()
        .filter(|n| focusable(n.role, n.actions))
        .collect()
}

/// Semantic order equals visual reading order, no two controls overlap, and
/// every Tab / Shift-Tab step moves to the neighbouring enabled control of
/// the semantic tree (a scrolling list may continue past its last row).
fn reading_order_guard(ui: &mut Controller<'_, Port>, seq: &mut u64, name: &str) {
    let mut raw = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
    let count = Frame::new(ui.view()).semantics(&mut raw).unwrap();
    let all = controls(&raw[..count]);
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            assert!(
                !reads_before(b, a),
                "{name}: {:?} {:?} is read before {:?} {:?} but follows it semantically",
                b.id,
                b.bounds,
                a.id,
                a.bounds
            );
            let (x, y) = (a.bounds, b.bounds);
            assert!(
                !(x.x < y.x + y.width
                    && y.x < x.x + x.width
                    && x.y < y.y + y.height
                    && y.y < x.y + x.height),
                "{name}: {:?} {x:?} overlaps {:?} {y:?}",
                a.id,
                b.id
            );
        }
    }
    let screen = ui.view().screen;
    for reverse in [false, true] {
        for _ in 0..24 {
            let tree = nodes(ui);
            let order: Vec<SemanticId> = tree
                .iter()
                .filter(|n| n.enabled && focusable(n.role, n.actions))
                .map(|n| n.id)
                .collect();
            let at = order
                .iter()
                .position(|id| *id == focused(ui))
                .unwrap_or_else(|| panic!("{name}: focus not on an enabled control"));
            key(ui, seq, PhysicalKey::Tab, reverse);
            assert_eq!(ui.view().screen, screen, "{name}: Tab left the screen");
            let next = focused(ui);
            let expected = order[if reverse {
                (at + order.len() - 1) % order.len()
            } else {
                (at + 1) % order.len()
            }];
            // Moving into or along a scrolling list may bring an off-screen
            // row into view; that row is not in the tree taken before Tab.
            let row = |id: &SemanticId| matches!(id, SemanticId::Object(_));
            let scrolls = row(&next) && (row(&expected) || row(&order[at]));
            assert!(
                next == expected || (scrolls && !order.contains(&next)),
                "{name}: Tab{} from {:?} went to {next:?}, semantic order says {expected:?}",
                if reverse { " (reverse)" } else { "" },
                order[at]
            );
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum State {
    Material,
    MaterialPaged,
    Editor,
    EditorSaving,
    Rename,
    Organize,
    Filter,
    ConfirmClose,
    /// Rename / Organize with an empty form, so Confirm is disabled.
    RenameEmpty,
    OrganizeEmpty,
    History,
}

/// Controls each state must still expose, disabled, in the semantic tree.
fn disabled_in(state: State, mode: EditMode) -> &'static [SemanticId] {
    match (state, mode) {
        (State::EditorSaving, _) => &[SemanticId::DocumentTool],
        (State::RenameEmpty | State::OrganizeEmpty, _) => &[SemanticId::FormConfirm],
        (State::Editor, EditMode::ReadOnly) => &[SemanticId::Save],
        _ => &[],
    }
}

fn guard_state(w: u32, h: u32, scale: Scale, state: State) {
    guard_state_in(w, h, scale, state, EditMode::ReadWrite);
}

fn guard_state_in(w: u32, h: u32, scale: Scale, state: State, mode: EditMode) {
    let host = Host::new();
    for i in 0..7u64 {
        host.borrow_mut().seed_named(i, b"Reis", b"Earlier");
    }
    host.borrow_mut().seed_named(0, b"Reis", b"Current");
    let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
    let mut ui = controller(
        &host,
        mode,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    let factor = scale.factor();
    ui.resize(Viewport::new(w * factor, h * factor, scale, 2).unwrap())
        .unwrap();
    ui.refresh().unwrap();
    match state {
        State::Material => {}
        State::MaterialPaged => {
            let generation = ui.view().generation;
            ui.activate_semantic(SemanticId::NextPage, generation)
                .unwrap();
        }
        State::Filter => {
            let generation = ui.view().generation;
            ui.activate_semantic(SemanticId::CollectionFilter, generation)
                .unwrap();
            assert_eq!(ui.view().screen, Screen::Filter);
        }
        _ => {
            ui.open(0).unwrap();
            match state {
                State::EditorSaving => {
                    ui.insert("Mine ").unwrap();
                    ui.save().unwrap();
                    assert!(ui.view().saving);
                }
                State::Rename => ui.begin_rename().unwrap(),
                State::Organize => ui.begin_organize().unwrap(),
                State::RenameEmpty => {
                    ui.begin_rename().unwrap();
                    ui.set_form_value("").unwrap();
                }
                State::OrganizeEmpty => {
                    ui.begin_organize().unwrap();
                    ui.set_form_value("").unwrap();
                }
                State::History => ui.begin_history().unwrap(),
                State::ConfirmClose => {
                    ui.insert("Mine ").unwrap();
                    ui.request_back().unwrap();
                    assert_eq!(ui.view().screen, Screen::ConfirmClose);
                }
                _ => {}
            }
        }
    }
    let name = format!("{state:?} {mode:?} {w}x{h} {scale:?}");
    let tree = nodes(&ui);
    for id in disabled_in(state, mode) {
        let node = tree
            .iter()
            .find(|n| n.id == *id)
            .unwrap_or_else(|| panic!("{name}: disabled {id:?} missing from the semantic tree"));
        assert!(!node.enabled, "{name}: {id:?} should be disabled");
    }
    let mut seq = 0;
    reading_order_guard(&mut ui, &mut seq, &name);
}

/// Widths below 480, 480..559, below 700, 700..783 and from 784 (with compact
/// heights down to 240), each at 100 % and 200 %, for every native app screen
/// except History, plus read-only Material and Editor. History is still read
/// Restore (right) before Cancel (left); see the ignored test below.
#[test]
fn semantic_order_is_reading_order_and_tab_follows_it_on_every_screen() {
    run(|| {
        for (w, h) in [
            (400, 540),
            (520, 540),
            (600, 540),
            (760, 540),
            (760, 300),
            (960, 540),
            (880, 300),
            (880, 287),
            (880, 240),
        ] {
            for scale in [Scale::Percent100, Scale::Percent200] {
                for state in [
                    State::Material,
                    State::MaterialPaged,
                    State::Editor,
                    State::EditorSaving,
                    State::Rename,
                    State::Organize,
                    State::RenameEmpty,
                    State::OrganizeEmpty,
                    State::Filter,
                    State::ConfirmClose,
                ] {
                    guard_state(w, h, scale, state);
                }
                for state in [State::Material, State::Editor] {
                    guard_state_in(w, h, scale, state, EditMode::ReadOnly);
                }
            }
        }
    });
}

#[test]
#[ignore = "follow-up: the History dialog reads Version, Restore, Cancel, not \
            its visual order; out of the U04 ruling's scope and relied on by \
            tools/window_library.py:77 (Alt+H, Tab, Enter) and \
            ui/native/tests/library_native_fixture.rs:191-192, so it must \
            change together with those scripts"]
fn history_dialog_semantic_order_is_reading_order() {
    run(|| {
        for scale in [Scale::Percent100, Scale::Percent200] {
            guard_state(960, 540, scale, State::History);
        }
    });
}

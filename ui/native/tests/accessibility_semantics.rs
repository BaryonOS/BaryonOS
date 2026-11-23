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
#[ignore = "U04 defect: Tab visits list items before search and filter (controller.rs:1543-1571); the semantic tree lists search and filter first (render.rs:1243 before 1317)"]
fn material_screen_tab_order_matches_semantic_order() {
    run(|| screen(Target::Material, true));
}
#[test]
fn editor_screen_labels_focus_and_wrapping_tab_cycle() {
    run(|| screen(Target::Editor, false));
}
#[test]
#[ignore = "U04 defect: Tab order Back, Document, Save, Rename, Organize, History, Tool, Menu (controller.rs:1572-1601); the semantic tree lists Rename, Organize, History first and Document last (render.rs:1332)"]
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
#[ignore = "U04 defect: Tab visits Cancel before the collection slots (controller.rs:1603-1618); the semantic tree lists the slots before Cancel (render.rs:1463, 1544)"]
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

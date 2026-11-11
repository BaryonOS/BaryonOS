#[allow(dead_code)]
mod common;

use common::*;
use nieuw_os_document_session::{volume::Buffers, EditMode};
use nieuw_os_input_core::{CancelReason, Event};
use nieuw_os_materiaal::{
    controller::{Controller, DocumentTool, Error, InputEnvelope, Screen, Status},
    render::{Frame, Layout, SemanticId, SemanticNode, MAX_SEMANTIC_NODES},
    Rect, Scale, Viewport, TILE_PIXELS,
};
use std::{cell::RefCell, rc::Rc};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, MouseButtons, MouseEvent, PhysicalKey};

fn scenario(
    mode: EditMode,
    test: impl FnOnce(&Rc<RefCell<Host>>, &mut Controller<'_, Port>) + Send + 'static,
) {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let host = Host::new();
            host.borrow_mut()
                .seed(0, b"Een echt document\nTweede regel");
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
            ui.open(0).unwrap();
            test(&host, &mut ui);
        })
        .unwrap()
        .join()
        .unwrap();
}

fn key(key: PhysicalKey, shift: bool, control: bool) -> Event {
    Event::Key(KeyboardEvent {
        key,
        action: KeyAction::Press,
        modifiers: KeyboardModifiers {
            left_shift: shift,
            left_control: control,
            ..Default::default()
        },
    })
}

fn send(ui: &mut Controller<'_, Port>, sequence: &mut u64, event: Event) {
    *sequence += 1;
    ui.deliver(InputEnvelope {
        epoch: INPUT_EPOCH,
        sequence: *sequence,
        generation: 1,
        event,
    })
    .unwrap();
}

fn mouse(x: u32, y: u32, buttons: u8, changed: u8) -> Event {
    Event::Mouse {
        x,
        y,
        input: MouseEvent {
            dx: 0,
            dy: 0,
            buttons: MouseButtons::checked(buttons).unwrap(),
            changed: MouseButtons::checked(changed).unwrap(),
            x_overflow: false,
            y_overflow: false,
        },
    }
}

fn click(ui: &mut Controller<'_, Port>, sequence: &mut u64, rect: Rect) {
    let (x, y) = (rect.x + rect.width / 2, rect.y + rect.height / 2);
    send(ui, sequence, mouse(x, y, 1, 1));
    send(ui, sequence, mouse(x, y, 0, 1));
}

fn state(ui: &Controller<'_, Port>) -> (String, usize, usize, usize, Status, bool, u64) {
    let v = ui.view();
    (
        v.contents.unwrap().to_owned(),
        v.caret,
        v.anchor,
        v.scroll,
        v.status,
        v.document_dirty,
        ui.document().draft().unwrap().object,
    )
}

fn activate(ui: &mut Controller<'_, Port>, id: SemanticId) {
    ui.activate_semantic(id, ui.view().generation).unwrap();
}

#[test]
fn genuine_menu_keys_pointer_and_named_button_open_without_storage_or_selection_changes() {
    scenario(EditMode::ReadWrite, |host, ui| {
        ui.insert("Mijn draft. ").unwrap();
        let mut sequence = 0;
        send(ui, &mut sequence, key(PhysicalKey::Left, true, false));
        let before = state(ui);
        let requests = host.borrow().requests.len();
        for trigger in [
            key(PhysicalKey::Menu, false, false),
            key(PhysicalKey::F10, true, false),
        ] {
            send(ui, &mut sequence, trigger);
            assert!(ui.view().document_menu.is_some());
            assert_eq!(state(ui), before);
            send(ui, &mut sequence, key(PhysicalKey::Escape, false, false));
            assert_eq!(state(ui), before);
        }
        send(ui, &mut sequence, key(PhysicalKey::F10, false, false));
        assert!(ui.view().document_menu.is_none());
        for action in [KeyAction::Release, KeyAction::Repeat] {
            send(
                ui,
                &mut sequence,
                Event::Key(KeyboardEvent {
                    key: PhysicalKey::Menu,
                    action,
                    modifiers: Default::default(),
                }),
            );
            assert!(ui.view().document_menu.is_none());
        }
        let editor = Layout::new(ui.view().viewport).editor;
        send(ui, &mut sequence, mouse(editor.x + 70, editor.y + 20, 2, 2));
        send(ui, &mut sequence, mouse(editor.x + 70, editor.y + 20, 0, 2));
        let menu = ui.view().document_menu.unwrap();
        assert_eq!(
            (menu.bounds.x, menu.bounds.y),
            (editor.x + 70, editor.y + 20)
        );
        assert_eq!(state(ui), before);
        activate(ui, SemanticId::CancelDocumentMenu);
        for (w, h, scale) in [
            (320, 240, Scale::Percent100),
            (480, 260, Scale::Percent100),
            (880, 510, Scale::Percent100),
            (1120, 660, Scale::Percent200),
        ] {
            ui.resize(Viewport::new(w, h, scale, 2).unwrap()).unwrap();
            let layout = Layout::new(ui.view().viewport);
            for other in [layout.back, layout.tool, layout.save, layout.editor] {
                assert!(!layout.context_menu.intersects(other));
            }
            let mut nodes = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
            let count = Frame::new(ui.view()).semantics(&mut nodes).unwrap();
            let button = nodes[..count]
                .iter()
                .find(|n| n.id == SemanticId::DocumentMenu)
                .unwrap();
            assert_eq!(button.label, "Documentacties openen");
            click(ui, &mut sequence, button.bounds);
            assert!(ui.view().document_menu.is_some());
            let menu = ui.view().document_menu.unwrap();
            assert!(menu.bounds.x >= 8 && menu.bounds.y >= 8);
            assert!(menu.bounds.x + menu.bounds.width + 8 <= w / scale.factor());
            assert!(menu.bounds.y + menu.bounds.height + 8 <= h / scale.factor());
            activate(ui, SemanticId::CancelDocumentMenu);
        }
        assert_eq!(host.borrow().requests.len(), requests);
    });
}

#[test]
fn modal_keys_do_not_edit_save_switch_or_authorize_clipboard() {
    scenario(EditMode::ReadWrite, |host, ui| {
        ui.insert("draft ").unwrap();
        let mut sequence = 0;
        send(ui, &mut sequence, key(PhysicalKey::A, false, true));
        let before = state(ui);
        let requests = host.borrow().requests.len();
        send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
        for event in [
            key(PhysicalKey::B, false, false),
            key(PhysicalKey::Backspace, false, false),
            key(PhysicalKey::Delete, false, false),
            key(PhysicalKey::F4, false, false),
            key(PhysicalKey::S, false, true),
            key(PhysicalKey::C, false, true),
            key(PhysicalKey::V, false, true),
            key(PhysicalKey::A, false, true),
        ] {
            send(ui, &mut sequence, event);
        }
        assert!(ui.clipboard_selection().is_none());
        ui.paste_clipboard("geen recht").unwrap();
        ui.clipboard_feedback(Status::ClipboardCopied).unwrap();
        assert_eq!(ui.insert("geen edit"), Err(Error::InvalidInput));
        assert_eq!(state(ui), before);
        assert_eq!(ui.view().tool, DocumentTool::Write);
        assert_eq!(host.borrow().requests.len(), requests);
        send(ui, &mut sequence, key(PhysicalKey::Tab, true, false));
        assert_eq!(ui.view().document_menu.unwrap().selected, 3);
        send(ui, &mut sequence, key(PhysicalKey::Enter, false, false));
        assert!(ui.view().document_menu.is_none());
        assert_eq!(ui.clipboard_selection(), Some(before.0.as_str()));
        assert_eq!(state(ui), before);
    });
}

#[test]
fn select_all_and_switch_use_the_same_real_document_without_rpc() {
    scenario(EditMode::ReadWrite, |host, ui| {
        ui.insert("echt ").unwrap();
        let requests = host.borrow().requests.len();
        let before = state(ui);
        let mut sequence = 0;
        send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
        send(ui, &mut sequence, key(PhysicalKey::Enter, false, false));
        assert_eq!(ui.clipboard_selection(), Some(before.0.as_str()));
        let selected = state(ui);
        send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
        send(ui, &mut sequence, key(PhysicalKey::Down, false, false));
        send(ui, &mut sequence, key(PhysicalKey::Enter, false, false));
        assert_eq!(ui.view().tool, DocumentTool::Read);
        assert_eq!(state(ui), selected);
        assert_eq!(ui.insert("nee"), Err(Error::InvalidInput));
        ui.paste_clipboard("nee").unwrap();
        send(ui, &mut sequence, key(PhysicalKey::F10, true, false));
        activate(ui, SemanticId::DocumentTool);
        assert_eq!(ui.view().tool, DocumentTool::Write);
        assert_eq!(state(ui), selected);
        assert_eq!(host.borrow().requests.len(), requests);
    });
}

#[test]
fn menu_semantics_are_modal_generation_bound_and_readonly_save_stays_disabled() {
    scenario(EditMode::ReadOnly, |host, ui| {
        let mut sequence = 0;
        let old_generation = ui.view().generation;
        let requests = host.borrow().requests.len();
        send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
        assert_eq!(
            ui.activate_semantic(SemanticId::DocumentTool, old_generation),
            Err(Error::StaleInput)
        );
        let mut nodes = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
        let count = Frame::new(ui.view()).semantics(&mut nodes).unwrap();
        assert_eq!(count, 7);
        assert_eq!(nodes[..count].iter().filter(|n| n.focused).count(), 1);
        for id in [
            SemanticId::Back,
            SemanticId::Document,
            SemanticId::DocumentMenu,
        ] {
            assert!(!nodes[..count].iter().any(|n| n.id == id));
        }
        assert!(nodes[..count]
            .iter()
            .all(|n| !n.actions.set_value && n.value.is_none()));
        let save = nodes[..count]
            .iter()
            .find(|n| n.id == SemanticId::Save)
            .unwrap();
        assert!(!save.enabled && !save.actions.activate);
        for id in [
            SemanticId::Back,
            SemanticId::Document,
            SemanticId::DocumentMenu,
        ] {
            assert_eq!(
                ui.activate_semantic(id, ui.view().generation),
                Err(Error::InvalidInput)
            );
        }
        assert_eq!(
            ui.activate_semantic(SemanticId::Save, ui.view().generation),
            Err(Error::InvalidInput)
        );
        assert_eq!(
            ui.set_semantic_value(SemanticId::Document, ui.view().generation, "nee"),
            Err(Error::InvalidInput)
        );
        send(ui, &mut sequence, key(PhysicalKey::Tab, false, false));
        assert_eq!(ui.view().document_menu.unwrap().selected, 1);
        send(ui, &mut sequence, key(PhysicalKey::Right, false, false));
        assert_eq!(ui.view().document_menu.unwrap().selected, 3);
        send(ui, &mut sequence, key(PhysicalKey::Up, false, false));
        assert_eq!(ui.view().document_menu.unwrap().selected, 1);
        activate(ui, SemanticId::DocumentTool);
        assert!(!ui.view().editable && !ui.view().can_save);
        send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
        activate(ui, SemanticId::DocumentTool);
        assert_eq!(ui.view().tool, DocumentTool::Write);
        assert!(!ui.view().editable && !ui.view().can_save);
        assert_eq!(host.borrow().requests.len(), requests);
    });
}

#[test]
fn real_menu_save_commits_and_is_not_available_during_save_or_close_confirmation() {
    scenario(EditMode::ReadWrite, |host, ui| {
        ui.insert("Bewaar ").unwrap();
        let expected = ui.view().contents.unwrap().as_bytes().to_vec();
        let mut sequence = 0;
        send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
        activate(ui, SemanticId::Save);
        assert!(ui.view().saving && ui.view().document_menu.is_none());
        send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
        send(ui, &mut sequence, key(PhysicalKey::F10, true, false));
        assert!(ui.view().document_menu.is_none());
        while !ui.advance_save().unwrap() {}
        assert!(matches!(ui.view().status, Status::Saved { .. }));
        assert_eq!(host.borrow_mut().bytes(0), expected);
        ui.insert("nieuw ").unwrap();
        ui.request_back().unwrap();
        assert_eq!(ui.view().screen, Screen::ConfirmClose);
        send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
        send(ui, &mut sequence, key(PhysicalKey::F10, true, false));
        assert!(ui.view().document_menu.is_none());
    });
}

#[test]
fn pointer_press_release_must_match_and_outside_click_never_reaches_editor_or_save() {
    scenario(EditMode::ReadWrite, |host, ui| {
        ui.insert("draft ").unwrap();
        let before = state(ui);
        let requests = host.borrow().requests.len();
        let mut sequence = 0;
        send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
        let menu = ui.view().document_menu.unwrap();
        let first = menu.item(0);
        let second = menu.item(1);
        send(ui, &mut sequence, mouse(first.x + 12, first.y + 12, 1, 1));
        send(ui, &mut sequence, mouse(second.x + 12, second.y + 12, 0, 1));
        assert!(ui.view().document_menu.is_some());
        assert_eq!(state(ui), before);
        let save = Layout::new(ui.view().viewport).save;
        click(ui, &mut sequence, save);
        assert!(ui.view().document_menu.is_none());
        assert_eq!(state(ui), before);
        assert_eq!(host.borrow().requests.len(), requests);
        send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
        let menu = ui.view().document_menu.unwrap();
        let cancel = menu.item(3);
        click(ui, &mut sequence, cancel);
        assert!(ui.view().document_menu.is_none());
        assert_eq!(state(ui), before);
        send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
        let menu = ui.view().document_menu.unwrap();
        let item = menu.item(1);
        send(ui, &mut sequence, mouse(item.x + 12, item.y + 12, 1, 1));
        ui.cancel_pointer().unwrap();
        send(ui, &mut sequence, mouse(item.x + 12, item.y + 12, 0, 1));
        assert!(ui.view().document_menu.is_some());
        assert_eq!(ui.view().tool, DocumentTool::Write);
        assert_eq!(state(ui), before);
    });
}

#[test]
fn resize_cancel_rebind_and_new_focus_generation_cancel_pending_menu_clicks() {
    scenario(EditMode::ReadWrite, |_, ui| {
        let mut sequence = 0;
        for cancellation in 0..4 {
            let v = ui.view().viewport;
            let generation = match cancellation {
                0 | 1 => 1,
                2 => 2,
                _ => 3,
            };
            sequence += 1;
            ui.deliver(InputEnvelope {
                epoch: INPUT_EPOCH,
                sequence,
                generation,
                event: key(PhysicalKey::Menu, false, false),
            })
            .unwrap();
            let item = ui.view().document_menu.unwrap().item(1).physical(v.scale());
            sequence += 1;
            ui.deliver(InputEnvelope {
                epoch: INPUT_EPOCH,
                sequence,
                generation,
                event: mouse(item.x + 10, item.y + 10, 1, 1),
            })
            .unwrap();
            let before = state(ui);
            match cancellation {
                0 => ui
                    .resize(Viewport::new(880, 510, Scale::Percent100, 2).unwrap())
                    .unwrap(),
                1 => {
                    sequence += 1;
                    ui.deliver(InputEnvelope {
                        epoch: INPUT_EPOCH,
                        sequence,
                        generation,
                        event: Event::Cancel(CancelReason::FocusLost),
                    })
                    .unwrap();
                }
                2 => {
                    sequence += 1;
                    ui.deliver(InputEnvelope {
                        epoch: INPUT_EPOCH,
                        sequence,
                        generation: 3,
                        event: mouse(item.x + 10, item.y + 10, 0, 1),
                    })
                    .unwrap();
                }
                _ => ui.rebind_input(INPUT_EPOCH + 1).unwrap(),
            }
            assert!(ui.view().document_menu.is_none());
            let after = state(ui);
            assert_eq!(
                (&after.0, after.1, after.2, after.3, after.5, after.6),
                (&before.0, before.1, before.2, before.3, before.5, before.6)
            );
            assert_eq!(ui.view().tool, DocumentTool::Write);
            if cancellation == 1 {
                // The next input generation must be newer than the cancellation.
                sequence += 1;
                ui.deliver(InputEnvelope {
                    epoch: INPUT_EPOCH,
                    sequence,
                    generation: 2,
                    event: key(PhysicalKey::Space, false, true),
                })
                .unwrap();
            }
            if cancellation == 3 {
                let old = InputEnvelope {
                    epoch: INPUT_EPOCH,
                    sequence: sequence + 1,
                    generation: 3,
                    event: mouse(item.x + 10, item.y + 10, 0, 1),
                };
                assert_eq!(ui.deliver(old), Err(Error::InputEpoch));
                ui.deliver(InputEnvelope {
                    epoch: INPUT_EPOCH + 1,
                    sequence: 1,
                    generation: 1,
                    ..old
                })
                .unwrap();
                assert!(ui.view().document_menu.is_none());
                assert_eq!(ui.view().tool, DocumentTool::Write);
            }
        }
    });
}

fn snapshot(ui: &Controller<'_, Port>) -> Vec<Vec<u32>> {
    let frame = Frame::new(ui.view());
    frame
        .tiles()
        .map(|rect| {
            let mut pixels = [0x1234_5678; TILE_PIXELS + 3];
            let r = frame.render_tile(rect.x, rect.y, &mut pixels).unwrap();
            let count = (r.width * r.height) as usize;
            assert!(pixels[count..].iter().all(|p| *p == 0x1234_5678));
            pixels[..count].to_vec()
        })
        .collect()
}

#[test]
fn full_repaint_covers_menu_shadow_and_restores_every_pixel_at_both_native_scales() {
    scenario(EditMode::ReadWrite, |_, ui| {
        ui.set_compositor_cursor(true).unwrap();
        let mut sequence = 0;
        for (width, height, scale) in [
            (320, 240, Scale::Percent100),
            (480, 260, Scale::Percent100),
            (880, 510, Scale::Percent100),
            (1120, 660, Scale::Percent200),
        ] {
            ui.resize(Viewport::new(width, height, scale, 2).unwrap())
                .unwrap();
            let before = snapshot(ui);
            assert!(ui.presented(ui.view().generation, ui.view().viewport.epoch()));
            let original = state(ui);
            send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
            let open = snapshot(ui);
            assert_ne!(before, open);
            let menu = ui.view().document_menu.unwrap().bounds;
            let shadow = Rect {
                x: menu.x - 8,
                y: menu.y - 8,
                width: menu.width + 16,
                height: menu.height + 16,
            }
            .physical(scale);
            for tile in Frame::new(ui.view())
                .tiles()
                .filter(|t| t.intersects(shadow))
            {
                assert!(ui
                    .paint_damage()
                    .intersects(tile.x, tile.y, tile.width, tile.height));
            }
            assert!(ui.presented(ui.view().generation, ui.view().viewport.epoch()));
            send(ui, &mut sequence, key(PhysicalKey::Escape, false, false));
            for tile in Frame::new(ui.view())
                .tiles()
                .filter(|t| t.intersects(shadow))
            {
                assert!(ui
                    .paint_damage()
                    .intersects(tile.x, tile.y, tile.width, tile.height));
            }
            assert_eq!(
                snapshot(ui),
                before,
                "dismiss must repaint all former shadow pixels"
            );
            assert_eq!(state(ui), original);
        }
    });
}

#[test]
fn menu_geometry_and_named_actions_are_stable_for_native_integration() {
    scenario(EditMode::ReadWrite, |_, ui| {
        let mut sequence = 0;
        for (width, height, scale, button, keyboard_menu, button_menu) in [
            (
                880,
                510,
                Scale::Percent100,
                Rect {
                    x: 812,
                    y: 68,
                    width: 44,
                    height: 44,
                },
                Rect {
                    x: 44,
                    y: 200,
                    width: 248,
                    height: 208,
                },
                Rect {
                    x: 624,
                    y: 112,
                    width: 248,
                    height: 208,
                },
            ),
            (
                1120,
                660,
                Scale::Percent200,
                Rect {
                    x: 984,
                    y: 136,
                    width: 88,
                    height: 88,
                },
                Rect {
                    x: 88,
                    y: 228,
                    width: 496,
                    height: 416,
                },
                Rect {
                    x: 608,
                    y: 224,
                    width: 496,
                    height: 416,
                },
            ),
        ] {
            ui.resize(Viewport::new(width, height, scale, 2).unwrap())
                .unwrap();
            assert_eq!(
                Layout::new(ui.view().viewport).context_menu.physical(scale),
                button
            );
            send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
            assert_eq!(
                ui.view().document_menu.unwrap().bounds.physical(scale),
                keyboard_menu
            );
            activate(ui, SemanticId::CancelDocumentMenu);
            click(ui, &mut sequence, button);
            assert_eq!(
                ui.view().document_menu.unwrap().bounds.physical(scale),
                button_menu
            );
            let mut nodes = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
            let count = Frame::new(ui.view()).semantics(&mut nodes).unwrap();
            for (index, id, label) in [
                (0, SemanticId::SelectAll, "Alles selecteren"),
                (1, SemanticId::DocumentTool, "Lezen"),
                (2, SemanticId::Save, "Opslaan"),
                (3, SemanticId::CancelDocumentMenu, "Annuleren"),
            ] {
                let node = nodes[..count].iter().find(|n| n.id == id).unwrap();
                assert_eq!(node.label, label);
                assert_eq!(
                    node.bounds,
                    Rect {
                        x: button_menu.x + 10 * scale.factor(),
                        y: button_menu.y + (10 + 48 * index) * scale.factor(),
                        width: 228 * scale.factor(),
                        height: 44 * scale.factor(),
                    }
                );
            }
            activate(ui, SemanticId::CancelDocumentMenu);
        }
    });
}

#[test]
fn long_titles_never_paint_over_the_context_button_or_its_focus_ring() {
    scenario(EditMode::ReadWrite, |_, ui| {
        for (width, height, scale) in [
            (320, 240, Scale::Percent100),
            (480, 260, Scale::Percent100),
            (880, 510, Scale::Percent100),
            (1120, 660, Scale::Percent200),
        ] {
            ui.resize(Viewport::new(width, height, scale, 2).unwrap())
                .unwrap();
            let mut empty = ui.view();
            empty.title = b"";
            let mut long = empty;
            long.title = b"WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW";
            let button = Layout::new(empty.viewport).context_menu.physical(scale);
            let clear = Frame::new(empty);
            let titled = Frame::new(long);
            for tile in clear.tiles().filter(|t| t.intersects(button)) {
                let (mut a, mut b) = ([0; TILE_PIXELS], [0; TILE_PIXELS]);
                let r = clear.render_tile(tile.x, tile.y, &mut a).unwrap();
                titled.render_tile(tile.x, tile.y, &mut b).unwrap();
                for y in r.y..r.y + r.height {
                    for x in r.x..r.x + r.width {
                        if button.contains(x, y) {
                            let index = ((y - r.y) * r.width + x - r.x) as usize;
                            assert_eq!(a[index], b[index]);
                        }
                    }
                }
            }
        }
    });
}

#[test]
fn desktop_overlay_and_close_never_leave_live_menu_actions() {
    scenario(EditMode::ReadWrite, |_, ui| {
        let mut sequence = 0;
        send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
        ui.enable_vm_desktop().unwrap();
        assert!(ui.view().document_menu.is_none());
        send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
        send(ui, &mut sequence, key(PhysicalKey::F10, true, false));
        assert!(ui.view().document_menu.is_none());
        assert_eq!(
            ui.activate_semantic(SemanticId::SelectAll, ui.view().generation),
            Err(Error::InvalidInput)
        );
    });
    scenario(EditMode::ReadWrite, |_, ui| {
        let mut sequence = 0;
        send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
        let generation = ui.view().generation;
        ui.close_clean().unwrap();
        assert!(ui.view().document_menu.is_none());
        assert_eq!(
            ui.activate_semantic(SemanticId::DocumentTool, generation),
            Err(Error::StaleInput)
        );
        send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
        assert!(ui.view().document_menu.is_none());
    });
}

#[test]
fn scrolled_selection_survives_menu_dismissal_and_resync_never_replays_a_press() {
    scenario(EditMode::ReadWrite, |_, ui| {
        let lines = "Een lange echte regel\n".repeat(90);
        ui.set_semantic_value(SemanticId::Document, ui.view().generation, &lines)
            .unwrap();
        let mut sequence = 0;
        send(ui, &mut sequence, key(PhysicalKey::Left, true, false));
        assert!(ui.view().scroll > 0);
        let before = state(ui);
        send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
        send(ui, &mut sequence, key(PhysicalKey::Escape, false, false));
        assert_eq!(state(ui), before);
        send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
        let item = ui.view().document_menu.unwrap().item(1);
        send(ui, &mut sequence, mouse(item.x + 10, item.y + 10, 1, 1));
        send(ui, &mut sequence, Event::Cancel(CancelReason::Resync));
        assert!(ui.view().document_menu.is_none());
        sequence += 1;
        ui.deliver(InputEnvelope {
            epoch: INPUT_EPOCH,
            sequence,
            generation: 2,
            event: mouse(item.x + 10, item.y + 10, 0, 1),
        })
        .unwrap();
        assert_eq!(ui.view().tool, DocumentTool::Write);
        let after = state(ui);
        assert_eq!(
            (&after.0, after.1, after.2, after.3, after.5, after.6),
            (&before.0, before.1, before.2, before.3, before.5, before.6)
        );
        assert_eq!(after.4, Status::InputLost);
    });
}

fn key_action(key: PhysicalKey, action: KeyAction) -> Event {
    Event::Key(KeyboardEvent {
        key,
        action,
        modifiers: KeyboardModifiers::default(),
    })
}

#[test]
fn held_enter_on_select_all_does_not_replace_the_draft_through_repeat_and_release() {
    scenario(EditMode::ReadWrite, |_, ui| {
        let mut sequence = 0;
        let before = state(ui);
        send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
        assert_eq!(ui.view().document_menu.unwrap().selected, 0);
        send(
            ui,
            &mut sequence,
            key_action(PhysicalKey::Enter, KeyAction::Press),
        );
        assert!(ui.view().document_menu.is_none());
        for _ in 0..3 {
            send(
                ui,
                &mut sequence,
                key_action(PhysicalKey::Enter, KeyAction::Repeat),
            );
        }
        send(
            ui,
            &mut sequence,
            key_action(PhysicalKey::Enter, KeyAction::Release),
        );
        let after = state(ui);
        assert_eq!(after.0, before.0, "the draft text is preserved");
        assert!(!after.5, "the draft stays clean");
        // After release a genuine new Enter press edits again.
        send(
            ui,
            &mut sequence,
            key_action(PhysicalKey::Enter, KeyAction::Press),
        );
        assert_eq!(state(ui).0, "\n");
    });
}

#[test]
fn backspace_held_through_menu_dismissal_does_not_delete_until_a_new_press() {
    scenario(EditMode::ReadWrite, |_, ui| {
        let mut sequence = 0;
        send(ui, &mut sequence, key(PhysicalKey::End, false, true));
        let before = state(ui);
        send(ui, &mut sequence, key(PhysicalKey::Menu, false, false));
        send(
            ui,
            &mut sequence,
            key_action(PhysicalKey::Backspace, KeyAction::Press),
        );
        send(ui, &mut sequence, key(PhysicalKey::Escape, false, false));
        assert!(ui.view().document_menu.is_none());
        send(
            ui,
            &mut sequence,
            key_action(PhysicalKey::Backspace, KeyAction::Repeat),
        );
        send(
            ui,
            &mut sequence,
            key_action(PhysicalKey::Backspace, KeyAction::Repeat),
        );
        send(
            ui,
            &mut sequence,
            key_action(PhysicalKey::Backspace, KeyAction::Release),
        );
        let after = state(ui);
        assert_eq!(
            (&after.0, after.1, after.5),
            (&before.0, before.1, before.5)
        );
        send(
            ui,
            &mut sequence,
            key_action(PhysicalKey::Backspace, KeyAction::Press),
        );
        assert_eq!(state(ui).0.len(), before.0.len() - 1);
    });
}

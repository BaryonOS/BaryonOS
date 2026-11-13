#[allow(dead_code)]
mod common;
use common::*;
use nieuw_os_document_session::{volume::Buffers, EditMode};
use nieuw_os_input_core::{CancelReason, Event};
use nieuw_os_materiaal::{
    controller::{DocumentTool, Error, InputEnvelope, Screen},
    render::{Frame, Layout, SemanticId, SemanticNode, MAX_SEMANTIC_NODES},
    Scale, Viewport,
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

fn run(test: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(test)
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn tools_borrow_the_same_object_and_never_implicitly_save_or_reopen() {
    run(|| {
        for mode in [EditMode::ReadWrite, EditMode::ReadOnly] {
            let host = Host::new();
            host.borrow_mut().seed(0, b"Original");
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
            if mode == EditMode::ReadWrite {
                ui.insert("Mijn draft. ").unwrap();
            }
            let before = ui.view();
            let expected = before.contents.unwrap().to_owned();
            let identity = ui.document().draft().unwrap().object;
            let state = (
                before.caret,
                before.anchor,
                before.scroll,
                before.status,
                before.document_dirty,
            );
            let requests = host.borrow().requests.len();
            ui.switch_tool().unwrap();
            assert_eq!(ui.view().tool, DocumentTool::Read);
            assert!(!ui.view().editable);
            assert!(!ui.view().can_save);
            assert_eq!(ui.save(), Err(Error::InvalidInput));
            assert_eq!(ui.begin_organize(), Err(Error::InvalidInput));
            assert_eq!(ui.insert("mag niet"), Err(Error::InvalidInput));
            ui.paste_clipboard("mag niet").unwrap();
            assert_eq!(
                ui.set_semantic_value(SemanticId::Document, ui.view().generation, "mag niet"),
                Err(Error::InvalidInput)
            );
            assert_eq!(ui.view().contents, Some(expected.as_str()));
            let view = ui.view();
            let mut nodes = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
            let count = Frame::new(view).semantics(&mut nodes).unwrap();
            let document = nodes[..count]
                .iter()
                .find(|n| n.id == SemanticId::Document)
                .unwrap();
            assert!(document.read_only);
            assert!(!document.actions.set_value);
            ui.switch_tool().unwrap();
            let view = ui.view();
            assert_eq!(view.editable, mode == EditMode::ReadWrite);
            assert_eq!(
                (
                    view.caret,
                    view.anchor,
                    view.scroll,
                    view.status,
                    view.document_dirty
                ),
                state
            );
            assert_eq!(ui.document().draft().unwrap().object, identity);
            assert_eq!(
                host.borrow().requests.len(),
                requests,
                "tool changes must not issue storage RPCs"
            );
            assert_eq!(host.borrow_mut().bytes(0), b"Original");
        }
    });
}

#[test]
fn keyboard_tool_selection_copy_and_close_dialog_preserve_draft() {
    run(|| {
        let host = Host::new();
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
        ui.create().unwrap();
        ui.insert("Een echt document").unwrap();
        let mut sequence = 0;
        let mut deliver = |ui: &mut nieuw_os_materiaal::controller::Controller<'_, Port>, event| {
            sequence += 1;
            ui.deliver(InputEnvelope {
                epoch: INPUT_EPOCH,
                sequence,
                generation: 1,
                event,
            })
            .unwrap();
        };
        let key = |key, action, control| {
            Event::Key(KeyboardEvent {
                key,
                action,
                modifiers: KeyboardModifiers {
                    left_control: control,
                    ..Default::default()
                },
            })
        };
        deliver(&mut ui, key(PhysicalKey::F4, KeyAction::Press, false));
        deliver(&mut ui, key(PhysicalKey::F4, KeyAction::Repeat, false));
        deliver(&mut ui, key(PhysicalKey::F4, KeyAction::Release, false));
        assert_eq!(ui.view().tool, DocumentTool::Read);
        assert!(!ui.view().can_save);
        let requests_before_save = host.borrow().requests.len();
        deliver(&mut ui, key(PhysicalKey::S, KeyAction::Press, true));
        assert_eq!(host.borrow().requests.len(), requests_before_save);
        assert!(ui.view().document_dirty);
        deliver(&mut ui, key(PhysicalKey::A, KeyAction::Press, true));
        assert_eq!(ui.clipboard_selection(), Some("Een echt document"));
        for physical in [
            PhysicalKey::X,
            PhysicalKey::Delete,
            PhysicalKey::Backspace,
            PhysicalKey::Enter,
        ] {
            deliver(&mut ui, key(physical, KeyAction::Press, false));
        }
        assert_eq!(ui.view().contents, Some("Een echt document"));
        let selection = (ui.view().anchor, ui.view().caret);
        deliver(&mut ui, Event::Cancel(CancelReason::FocusLost));
        assert_eq!(ui.view().tool, DocumentTool::Read);
        assert_eq!((ui.view().anchor, ui.view().caret), selection);
        ui.request_back().unwrap();
        assert_eq!(ui.view().screen, Screen::ConfirmClose);
        assert!(!ui.view().can_save);
        let generation = ui.view().generation;
        assert_eq!(
            ui.activate_semantic(SemanticId::SaveAndBack, generation),
            Err(Error::InvalidInput)
        );
        assert_eq!(ui.view().screen, Screen::ConfirmClose);
        assert!(ui.view().document_dirty);
        assert_eq!(host.borrow().requests.len(), requests_before_save);
        assert_eq!(ui.switch_tool(), Err(Error::InvalidInput));
        ui.activate_semantic(SemanticId::KeepEditing, ui.view().generation)
            .unwrap();
        assert_eq!(ui.view().tool, DocumentTool::Read);
        ui.activate_semantic(SemanticId::DocumentTool, ui.view().generation)
            .unwrap();
        assert_eq!(ui.view().tool, DocumentTool::Write);
        ui.save().unwrap();
        assert_eq!(ui.switch_tool(), Err(Error::InvalidInput));
        while !ui.advance_save().unwrap() {}
        assert_eq!(host.borrow_mut().bytes(0), b"Een echt document");
        ui.request_back().unwrap();
        assert!(
            ui.view().editable,
            "reading must not disable creating the next object"
        );
    });
}

#[test]
fn tool_control_is_disjoint_and_semantic_at_both_native_scales() {
    for (width, height, scale) in [
        (320, 240, Scale::Percent100),
        (400, 300, Scale::Percent100),
        (880, 510, Scale::Percent100),
        (1120, 660, Scale::Percent200),
    ] {
        let viewport = Viewport::new(width, height, scale, 1).unwrap();
        let layout = Layout::new(viewport);
        assert!(!layout.tool.intersects(layout.back));
        assert!(!layout.tool.intersects(layout.save));
        assert!(layout.tool.width > 0);
    }
}

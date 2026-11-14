//! Real capability/volume-service contract checks. RAM sectors are host proof.
#[allow(dead_code)]
mod common;
use common::*;
use nieuw_os_document_session::{volume::Buffers, EditMode};
use nieuw_os_input_core::Event;
use nieuw_os_materiaal::{
    controller::{Controller, DocumentTool, Error, Focus, InputEnvelope, Screen, Status},
    render::{Frame, Layout, SemanticId, SemanticNode, MAX_SEMANTIC_NODES},
    Scale, Viewport,
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey, Ps2Mouse};
use volume_core::wire;

fn run(f: fn()) {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap();
}
fn send(ui: &mut Controller<'_, Port>, seq: &mut u64, event: Event) {
    *seq += 1;
    ui.deliver(InputEnvelope {
        epoch: INPUT_EPOCH,
        sequence: *seq,
        generation: 1,
        event,
    })
    .unwrap();
}
fn key(ui: &mut Controller<'_, Port>, seq: &mut u64, physical: PhysicalKey) {
    send(
        ui,
        seq,
        Event::Key(KeyboardEvent {
            key: physical,
            action: KeyAction::Press,
            modifiers: KeyboardModifiers::default(),
        }),
    );
}
fn saved(ui: &mut Controller<'_, Port>, contents: &str) {
    ui.create().unwrap();
    ui.insert(contents).unwrap();
    ui.save().unwrap();
    while !ui.advance_save().unwrap() {}
}
fn mutations(host: &Host) -> usize {
    host.requests
        .iter()
        .filter(|h| {
            matches!(
                h.operation,
                wire::BEGIN
                    | wire::COMMIT
                    | wire::COLLECTION
                    | wire::META
                    | wire::RESTORE
                    | wire::RENAME
            )
        })
        .count()
}

#[test]
fn cancelled_names_and_stale_semantic_actions_preserve_document_and_disk() {
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
        saved(&mut ui, "Werk blijft bewaard.");
        let before = mutations(&host.borrow());
        ui.begin_rename().unwrap();
        let generation = ui.view().generation;
        ui.set_semantic_value(SemanticId::FormInput, generation, "Reisnotities")
            .unwrap();
        let mut seq = 0;
        key(&mut ui, &mut seq, PhysicalKey::Escape);
        assert_eq!(ui.view().screen, Screen::Editor);
        assert_eq!(ui.view().title, b"Nieuw document");
        assert_eq!(ui.view().contents, Some("Werk blijft bewaard."));
        assert_eq!(
            ui.activate_semantic(SemanticId::FormConfirm, generation),
            Err(Error::StaleInput)
        );
        assert_eq!(mutations(&host.borrow()), before);
        ui.begin_rename().unwrap();
        ui.set_form_value("Reisnotities").unwrap();
        ui.confirm_form().unwrap();
        assert!(ui.document().is_dirty());
        assert_eq!(mutations(&host.borrow()), before);
    });
}

#[test]
fn a_held_confirm_button_cannot_cross_into_a_different_workspace_dialog() {
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
        saved(&mut ui, "Draft");
        ui.begin_rename().unwrap();
        let old = Layout::for_screen(ui.view().viewport, Screen::Rename).form_confirm;
        let mut mouse = Ps2Mouse::new();
        let mut seq = 0;
        let mut down = None;
        for b in [0x09, 0, 0] {
            if let Some(event) = mouse.feed(b).unwrap() {
                down = Some(event);
            }
        }
        send(
            &mut ui,
            &mut seq,
            Event::Mouse {
                input: down.unwrap(),
                x: old.x + 8,
                y: old.y + 8,
            },
        );
        key(&mut ui, &mut seq, PhysicalKey::Escape);
        ui.begin_organize().unwrap();
        ui.set_form_value("Reis").unwrap();
        let new = Layout::for_screen(ui.view().viewport, Screen::Organize).form_confirm;
        let before = mutations(&host.borrow());
        let mut up = None;
        for b in [0x08, 0, 0] {
            if let Some(event) = mouse.feed(b).unwrap() {
                up = Some(event);
            }
        }
        send(
            &mut ui,
            &mut seq,
            Event::Mouse {
                input: up.unwrap(),
                x: new.x + 8,
                y: new.y + 8,
            },
        );
        assert_eq!(mutations(&host.borrow()), before);
        assert_eq!(ui.view().screen, Screen::Organize);
    });
}

#[test]
fn collection_success_followed_by_stale_object_failure_is_truthful_and_nonreplaying() {
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
        saved(&mut ui, "Original");
        ui.begin_organize().unwrap();
        ui.set_form_value("Reis").unwrap();
        host.borrow_mut()
            .seed_named(0, b"Nieuw document", b"Concurrent werk");
        assert!(ui.confirm_form().is_err());
        assert_eq!(ui.view().status, Status::CollectionSavedAssignmentFailed);
        assert!(!ui.view().can_save);
        let table = host.borrow_mut().recover();
        assert!(table.collections[0].present);
        assert_eq!(table.entries[0].collection, 0);
        assert_eq!(host.borrow_mut().bytes(0), b"Concurrent werk");
        let before = mutations(&host.borrow());
        assert!(ui.confirm_form().is_err());
        assert_eq!(mutations(&host.borrow()), before);
    });
}

#[test]
fn filtered_creation_uses_the_entire_domain_and_search_reads_real_names_only() {
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
        saved(&mut ui, "One");
        ui.begin_rename().unwrap();
        ui.set_form_value("Reisnotities").unwrap();
        ui.confirm_form().unwrap();
        ui.save().unwrap();
        while !ui.advance_save().unwrap() {}
        ui.request_back().unwrap();
        let before = mutations(&host.borrow());
        ui.set_search_query("REIS").unwrap();
        assert_eq!(ui.view().objects.len(), 1);
        ui.set_search_query("onbekend").unwrap();
        assert!(ui.view().objects.is_empty());
        assert_eq!(mutations(&host.borrow()), before);
        ui.create().unwrap();
        assert_eq!(ui.document().draft().unwrap().object, 1);
        assert_eq!(host.borrow_mut().bytes(0), b"One");
    });
}

#[test]
fn history_readonly_preview_does_not_advertise_or_perform_a_restore() {
    run(|| {
        let host = Host::new();
        host.borrow_mut().seed_named(0, b"Reis", b"Earlier");
        host.borrow_mut().seed_named(0, b"Reis", b"Current");
        let (mut d, mut o, mut s) = ([0; 4096], [0; 4096], [0; 4096]);
        let mut ui = controller(
            &host,
            EditMode::ReadOnly,
            Buffers {
                draft: &mut d,
                original: &mut o,
                staging: &mut s,
            },
        );
        ui.open(0).unwrap();
        ui.begin_history().unwrap();
        assert_eq!(ui.view().workspace.preview(), Some("Earlier"));
        let before = mutations(&host.borrow());
        let mut nodes = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
        let count = Frame::new(ui.view()).semantics(&mut nodes).unwrap();
        assert!(nodes[..count]
            .iter()
            .any(|n| n.id == SemanticId::RestoreHistory && !n.enabled && !n.actions.activate));
        assert!(ui.restore_selected_history().is_err());
        assert_eq!(mutations(&host.borrow()), before);
    });
}

#[test]
fn read_tool_history_preview_cannot_restore_through_semantics_or_direct_calls() {
    run(|| {
        let host = Host::new();
        host.borrow_mut().seed_named(0, b"Reis", b"Earlier");
        host.borrow_mut().seed_named(0, b"Reis", b"Current");
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
        ui.open(0).unwrap();
        ui.switch_tool().unwrap();
        assert_eq!(ui.view().tool, DocumentTool::Read);
        ui.begin_history().unwrap();
        assert_eq!(ui.view().screen, Screen::History);
        assert_eq!(ui.view().workspace.preview(), Some("Earlier"));
        assert!(!ui.view().can_save);

        let before = mutations(&host.borrow());
        let mut nodes = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
        let count = Frame::new(ui.view()).semantics(&mut nodes).unwrap();
        assert!(nodes[..count]
            .iter()
            .any(|node| node.id == SemanticId::RestoreHistory
                && !node.enabled
                && !node.actions.activate));
        let generation = ui.view().generation;
        assert_eq!(
            ui.activate_semantic(SemanticId::RestoreHistory, generation),
            Err(Error::InvalidInput)
        );
        assert_eq!(ui.restore_selected_history(), Err(Error::InvalidInput));
        assert_eq!(ui.view().screen, Screen::History);
        assert_eq!(ui.view().tool, DocumentTool::Read);
        assert_eq!(mutations(&host.borrow()), before);
        assert_eq!(host.borrow_mut().bytes(0), b"Current");
    });
}

#[test]
fn new_workspace_controls_stay_visible_and_tab_focus_has_a_semantic_node() {
    run(|| {
        for (w, h, scale) in [
            (320, 240, Scale::Percent100),
            (768, 536, Scale::Percent100),
            (1120, 660, Scale::Percent200),
            (1920, 1080, Scale::Percent200),
        ] {
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
            ui.resize(Viewport::new(w, h, scale, 2).unwrap()).unwrap();
            saved(&mut ui, "Original");
            ui.insert(" more").unwrap();
            ui.save().unwrap();
            while !ui.advance_save().unwrap() {}
            let mut seq = 0;
            for kind in 0..3 {
                match kind {
                    0 => ui.begin_rename().unwrap(),
                    1 => ui.begin_organize().unwrap(),
                    _ => ui.begin_history().unwrap(),
                }
                for _ in 0..12 {
                    let mut nodes = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
                    let count = Frame::new(ui.view()).semantics(&mut nodes).unwrap();
                    let controls: Vec<_> = nodes[..count]
                        .iter()
                        .filter(|n| n.actions.activate || n.actions.set_value)
                        .collect();
                    assert!(
                        controls.iter().any(|n| n.focused),
                        "missing focused node at {w}x{h} {:?}",
                        ui.view().focus
                    );
                    for node in &controls {
                        assert!(node.bounds.x + node.bounds.width <= w);
                        assert!(node.bounds.y + node.bounds.height <= h);
                    }
                    for (index, a) in controls.iter().enumerate() {
                        for b in &controls[index + 1..] {
                            assert!(
                                !a.bounds.intersects(b.bounds),
                                "overlap {:?}/{:?} {w}x{h}",
                                a.id,
                                b.id
                            );
                        }
                    }
                    key(&mut ui, &mut seq, PhysicalKey::Tab);
                }
                key(&mut ui, &mut seq, PhysicalKey::Escape);
                assert_eq!(ui.view().screen, Screen::Editor);
            }
            assert_eq!(ui.view().focus, Focus::Document);
        }
    });
}

#[test]
fn history_page_keys_expose_all_wrapped_preview_rows_without_editing() {
    run(|| {
        let host = Host::new();
        let before =
            "Eerste regel\nTweede regel\nDerde regel\nVierde regel\nVijfde regel\nZesde regel";
        host.borrow_mut().seed_named(0, b"Reis", before.as_bytes());
        host.borrow_mut().seed_named(0, b"Reis", b"Current");
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
        ui.resize(Viewport::new(640, 480, Scale::Percent200, 2).unwrap())
            .unwrap();
        ui.open(0).unwrap();
        ui.begin_history().unwrap();
        let count = mutations(&host.borrow());
        let mut seq = 0;
        key(&mut ui, &mut seq, PhysicalKey::PageDown);
        assert!(ui.view().workspace.preview_scroll() > 0);
        assert_eq!(ui.view().contents, Some("Current"));
        for _ in 0..20 {
            key(&mut ui, &mut seq, PhysicalKey::PageDown);
        }
        assert!(ui.view().workspace.preview_scroll() >= 4);
        for _ in 0..20 {
            key(&mut ui, &mut seq, PhysicalKey::PageUp);
        }
        assert_eq!(ui.view().workspace.preview_scroll(), 0);
        assert_eq!(mutations(&host.borrow()), count);
    });
}

#[test]
fn verified_shortened_history_preview_can_restore_the_complete_large_document() {
    run(|| {
        let host = Host::new();
        let previous = "Lange reisnotities. Café bij het water.\n".repeat(100);
        host.borrow_mut()
            .seed_named(0, b"Reis", previous.as_bytes());
        host.borrow_mut().seed_named(0, b"Reis", b"Current");
        let (mut d, mut o, mut s) = ([0; 8192], [0; 8192], [0; 8192]);
        let mut ui = controller(
            &host,
            EditMode::ReadWrite,
            Buffers {
                draft: &mut d,
                original: &mut o,
                staging: &mut s,
            },
        );
        ui.open(0).unwrap();
        ui.begin_history().unwrap();
        assert!(ui.view().workspace.preview_truncated());
        assert_eq!(ui.view().workspace.preview_total(), previous.len());
        assert!(previous.starts_with(ui.view().workspace.preview().unwrap()));
        assert!(ui.view().workspace.preview().unwrap().len() <= 2048);
        ui.restore_selected_history().unwrap();
        assert_eq!(ui.view().contents, Some(previous.as_str()));
        assert_eq!(host.borrow_mut().bytes(0), previous.as_bytes());
    });
}

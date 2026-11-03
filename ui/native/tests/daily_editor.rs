mod common;

use common::*;
use nieuw_os_document_session::{volume::Buffers, EditMode};
use nieuw_os_input_core::Event;
use nieuw_os_materiaal::{
    controller::{InputEnvelope, Screen, Status},
    render::SemanticId,
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

fn key(sequence: u64, physical: PhysicalKey, shift: bool) -> InputEnvelope {
    InputEnvelope {
        epoch: INPUT_EPOCH,
        sequence,
        generation: 1,
        event: Event::Key(KeyboardEvent {
            key: physical,
            action: KeyAction::Press,
            modifiers: KeyboardModifiers {
                left_shift: shift,
                ..KeyboardModifiers::default()
            },
        }),
    }
}

fn editor_selection_save_close_and_fresh_reopen() {
    let host = Host::new();
    host.borrow_mut().seed(0, "Dag wereld\nCafé ☕".as_bytes());
    let (mut d, mut o, mut s) = ([0; 65536], [0; 65536], [0; 65536]);
    let mut ui = controller(
        &host,
        EditMode::ReadWrite,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    ui.refresh().unwrap();
    ui.activate_semantic(SemanticId::Object(0), ui.view().generation)
        .unwrap();
    for (i, physical) in [PhysicalKey::Down, PhysicalKey::End, PhysicalKey::Left]
        .into_iter()
        .enumerate()
    {
        ui.deliver(key(i as u64 + 1, physical, physical == PhysicalKey::Left))
            .unwrap();
    }
    assert_eq!(ui.clipboard_selection(), Some("☕"));
    for (i, physical) in [
        PhysicalKey::T,
        PhysicalKey::H,
        PhysicalKey::E,
        PhysicalKey::E,
    ]
    .into_iter()
    .enumerate()
    {
        ui.deliver(key(i as u64 + 4, physical, false)).unwrap();
    }
    let expected = "Dag wereld\nCafé thee";
    assert_eq!(ui.view().contents, Some(expected));
    ui.request_back().unwrap();
    assert_eq!(ui.view().screen, Screen::ConfirmClose);
    assert!(ui.close_clean().is_err());
    ui.activate_semantic(SemanticId::KeepEditing, ui.view().generation)
        .unwrap();
    assert_eq!(ui.view().contents, Some(expected));
    ui.request_back().unwrap();
    ui.activate_semantic(SemanticId::SaveAndBack, ui.view().generation)
        .unwrap();
    while !ui.advance_save().unwrap() {}
    assert_eq!(ui.view().screen, Screen::Material);
    assert_eq!(host.borrow_mut().bytes(0), expected.as_bytes());
    ui.close_clean().unwrap();
    assert_eq!(host.borrow().closed.len(), 2);
    drop(ui);
    d.fill(0);
    o.fill(0);
    s.fill(0);
    let restarted = Host::new();
    {
        let mut next = restarted.borrow_mut();
        next.disk.sectors = host.borrow().disk.sectors.clone();
        let table = next.recover();
        next.service = volume_core::service::Service::new(EPOCH, table, false);
    }
    let mut next = controller(
        &restarted,
        EditMode::ReadWrite,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    next.refresh().unwrap();
    next.activate_semantic(SemanticId::Object(0), next.view().generation)
        .unwrap();
    assert_eq!(next.view().contents, Some(expected));
    assert!(!next.document().is_dirty());
    next.close_clean().unwrap();
    assert_eq!(restarted.borrow().closed.len(), 2);
}

fn large_editor_failure_close_and_explicit_recovery() {
    for fault in 0..3 {
        let host = Host::new();
        host.borrow_mut().seed(0, b"Original");
        let (mut d, mut o, mut s) = ([0; 65536], [0; 65536], [0; 65536]);
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
        let text = "é".repeat(32768);
        ui.set_semantic_value(SemanticId::Document, ui.view().generation, &text)
            .unwrap();
        match fault {
            0 => host.borrow_mut().disk.fail = true,
            1 => {
                let root = host.borrow().request_root;
                host.borrow_mut().engine.revoke(2, root).unwrap();
            }
            _ => host
                .borrow_mut()
                .seed(0, b"Replacement from another writer"),
        }
        let result = ui.save().and_then(|()| {
            while !ui.advance_save()? {}
            Ok(())
        });
        assert!(result.is_err());
        assert_eq!(ui.view().contents, Some(text.as_str()));
        assert!(!matches!(ui.view().status, Status::Saved { .. }));
        assert!(ui.close_clean().is_err());
        ui.request_back().unwrap();
        assert_eq!(ui.view().screen, Screen::ConfirmClose);
        ui.activate_semantic(SemanticId::KeepEditing, ui.view().generation)
            .unwrap();
        assert_eq!(ui.view().contents, Some(text.as_str()));
        if fault == 2 {
            let requests = host.borrow().requests.len();
            assert!(ui.save().is_err());
            assert_eq!(host.borrow().requests.len(), requests);
            ui.request_back().unwrap();
            ui.activate_semantic(SemanticId::Discard, ui.view().generation)
                .unwrap();
            ui.activate_semantic(SemanticId::Object(0), ui.view().generation)
                .unwrap();
            assert_eq!(ui.view().contents, Some("Replacement from another writer"));
            assert!(!ui.document().is_conflicted());
            ui.close_clean().unwrap();
        } else {
            assert_ne!(ui.view().status, Status::Ready);
        }
    }
}

#[test]
fn daily_editor_selection_save_close_reopen() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(editor_selection_save_close_and_fresh_reopen)
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn daily_editor_large_draft_failed_save_revoke_and_stale_revision() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(large_editor_failure_close_and_explicit_recovery)
        .unwrap()
        .join()
        .unwrap();
}

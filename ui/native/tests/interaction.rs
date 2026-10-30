mod common;
use common::*;
use nieuw_os_document_session::{
    volume::{Buffers, Error as StorageError},
    EditMode,
};
use nieuw_os_input_core::{CancelReason, Event};
use nieuw_os_materiaal::{
    controller::{Error, Focus, InputEnvelope, Screen, Status},
    design,
    render::{Frame, Role, SemanticNode, MAX_SEMANTIC_NODES},
    text, Scale, Viewport, TILE_PIXELS,
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey, Ps2Mouse, Set1Keyboard};
use volume_core::wire;

fn envelope(sequence: u64, generation: u64, event: Event) -> InputEnvelope {
    InputEnvelope {
        epoch: INPUT_EPOCH,
        sequence,
        generation,
        event,
    }
}
fn key(key: PhysicalKey) -> Event {
    Event::Key(KeyboardEvent {
        key,
        action: KeyAction::Press,
        modifiers: KeyboardModifiers::default(),
    })
}
fn control(key: PhysicalKey) -> Event {
    Event::Key(KeyboardEvent {
        key,
        action: KeyAction::Press,
        modifiers: KeyboardModifiers {
            left_control: true,
            ..KeyboardModifiers::default()
        },
    })
}

fn real_ps2_edit_stream_commit_list_and_reopen_impl() {
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
    ui.refresh().unwrap();
    assert!(ui.view().objects.is_empty());
    ui.create().unwrap();
    let mut keyboard = Set1Keyboard::new();
    let mut sequence = 0;
    for byte in [0x23, 0xa3, 0x17, 0x97] {
        if let Some(event) = keyboard.feed(byte).unwrap() {
            sequence += 1;
            ui.deliver(envelope(sequence, 1, Event::Key(event)))
                .unwrap();
        }
    }
    ui.insert("\nCafé op dit apparaat.").unwrap();
    assert_eq!(ui.view().contents, Some("hi\nCafé op dit apparaat."));
    assert!(ui.document().is_dirty());
    sequence += 1;
    ui.deliver(envelope(sequence, 1, control(PhysicalKey::S)))
        .unwrap();
    assert!(matches!(
        ui.view().status,
        Status::Saving { completed: 0, .. }
    ));
    assert!(!ui.advance_save().unwrap());
    assert!(matches!(ui.view().status,Status::Saving{completed,..} if completed>0));
    assert!(ui.advance_save().unwrap());
    assert_eq!(ui.view().status, Status::Saved { revision: 1 });
    ui.request_back().unwrap();
    assert_eq!(ui.view().screen, Screen::Material);
    assert_eq!(ui.view().objects.len(), 1);
    ui.open(0).unwrap();
    assert_eq!(ui.view().contents, Some("hi\nCafé op dit apparaat."));
    let mut h = host.borrow_mut();
    assert_eq!(h.bytes(0), "hi\nCafé op dit apparaat.".as_bytes());
    assert_eq!(h.recover().entries[0].revision, 1);
    assert!(h.requests.iter().all(|r| r.epoch == EPOCH));
}

fn abort_retains_draft_and_does_not_publish_an_unsaved_revision_impl() {
    let host = Host::new();
    host.borrow_mut().seed(0, b"Original");
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
    ui.insert(&"Nieuw ".repeat(100)).unwrap();
    let expected = ui.view().contents.unwrap().to_owned();
    ui.save().unwrap();
    ui.advance_save().unwrap();
    assert_eq!(
        ui.insert("verboden"),
        Err(Error::Storage(StorageError::Busy))
    );
    ui.cancel_save().unwrap();
    assert_eq!(ui.view().contents, Some(expected.as_str()));
    assert!(ui.document().is_dirty());
    assert_eq!(ui.view().status, Status::Cancelled);
    let mut h = host.borrow_mut();
    assert!(h.requests.iter().any(|r| r.operation == wire::ABORT));
    assert!(!h.requests.iter().any(|r| r.operation == wire::COMMIT));
    assert_eq!(h.recover().entries[0].revision, 1);
    assert_eq!(h.bytes(0), b"Original");
}

fn lost_commit_never_becomes_saved_or_automatic_replay_impl() {
    let host = Host::new();
    host.borrow_mut().seed(0, b"Old");
    let (mut d, mut o, mut s) = ([0; 1024], [0; 1024], [0; 1024]);
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
    ui.insert("Keep ").unwrap();
    ui.save().unwrap();
    ui.advance_save().unwrap();
    host.borrow_mut().drop_commit = true;
    assert!(ui.advance_save().is_err());
    assert!(ui.document().is_uncertain());
    assert!(ui.document().is_closed());
    assert!(ui.document().is_dirty());
    assert_eq!(
        ui.view().status,
        Status::Failed(Error::Storage(StorageError::Uncertain))
    );
    assert!(!ui.view().can_save);
    let count = host.borrow().requests.len();
    assert_eq!(ui.save(), Err(Error::Storage(StorageError::Uncertain)));
    assert_eq!(count, host.borrow().requests.len());
    assert_eq!(ui.view().contents, Some("Keep Old"));
    // Returning local bytes to the originally loaded value must not turn an
    // unknown commit into a newly confirmed save.
    ui.deliver(envelope(1, 1, control(PhysicalKey::A))).unwrap();
    ui.insert("Old").unwrap();
    assert!(!ui.document().is_dirty());
    assert_eq!(
        ui.view().status,
        Status::Failed(Error::Storage(StorageError::Uncertain))
    );
    assert_eq!(ui.save(), Err(Error::Storage(StorageError::Uncertain)));
    assert_eq!(count, host.borrow().requests.len());
    let mut h = host.borrow_mut();
    assert_eq!(h.closed.len(), 2);
    assert_eq!(h.recover().entries[0].revision, 2);
    assert_eq!(h.bytes(0), b"Keep Old");
}

fn conflict_revocation_and_io_error_keep_unconfirmed_work_visible_impl() {
    for fault in 0..3 {
        let host = Host::new();
        host.borrow_mut().seed(0, b"Original");
        let (mut d, mut o, mut s) = ([0; 1024], [0; 1024], [0; 1024]);
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
        ui.insert("Mine ").unwrap();
        match fault {
            0 => host.borrow_mut().seed(0, b"Their version"),
            1 => {
                let root = host.borrow().request_root;
                host.borrow_mut().engine.revoke(2, root).unwrap();
            }
            _ => host.borrow_mut().disk.fail = true,
        }
        let result = ui.save().and_then(|()| {
            while !ui.advance_save()? {}
            Ok(())
        });
        assert!(result.is_err());
        assert_eq!(ui.view().contents, Some("Mine Original"));
        assert!(ui.document().is_dirty());
        assert!(!matches!(ui.view().status, Status::Saved { .. }));
        if fault == 0 {
            assert!(ui.document().is_conflicted());
        }
        if fault == 1 {
            assert!(ui.document().is_closed());
        }
    }
}

fn wrong_reply_identity_cannot_populate_a_fake_object_list_impl() {
    let host = Host::new();
    host.borrow_mut().seed(0, b"Data");
    host.borrow_mut().wrong_peer = true;
    let (mut d, mut o, mut s) = ([0; 512], [0; 512], [0; 512]);
    let mut ui = controller(
        &host,
        EditMode::ReadWrite,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    assert!(ui.refresh().is_err());
    assert!(!ui.view().list_loaded);
    assert!(ui.view().objects.is_empty());
    assert_eq!(host.borrow().closed.len(), 2);
}

fn dirty_close_defaults_to_cancel_and_only_explicit_discard_loses_edits_impl() {
    let host = Host::new();
    host.borrow_mut().seed(0, b"Original");
    let (mut d, mut o, mut s) = ([0; 512], [0; 512], [0; 512]);
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
    ui.insert("New ").unwrap();
    ui.request_back().unwrap();
    assert_eq!(ui.view().screen, Screen::ConfirmClose);
    assert_eq!(ui.view().focus, Focus::KeepEditing);
    ui.deliver(envelope(1, 1, key(PhysicalKey::Escape)))
        .unwrap();
    assert_eq!(ui.view().screen, Screen::Editor);
    assert_eq!(ui.view().contents, Some("New Original"));
    ui.request_back().unwrap();
    ui.deliver(envelope(2, 1, key(PhysicalKey::Tab))).unwrap();
    assert_eq!(ui.view().focus, Focus::Discard);
    ui.deliver(envelope(3, 1, key(PhysicalKey::Enter))).unwrap();
    assert_eq!(ui.view().screen, Screen::Material);
    assert_eq!(host.borrow_mut().bytes(0), b"Original");
    assert!(!host
        .borrow()
        .requests
        .iter()
        .any(|r| r.operation == wire::COMMIT));
}

fn replay_cancel_and_old_focus_cannot_trigger_a_new_edit_impl() {
    let host = Host::new();
    let (mut d, mut o, mut s) = ([0; 512], [0; 512], [0; 512]);
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
    ui.deliver(envelope(1, 3, key(PhysicalKey::A))).unwrap();
    assert_eq!(ui.view().contents, Some("a"));
    assert_eq!(
        ui.deliver(envelope(1, 3, key(PhysicalKey::B))),
        Err(Error::StaleInput)
    );
    ui.deliver(envelope(2, 3, Event::Cancel(CancelReason::FocusLost)))
        .unwrap();
    assert_eq!(
        ui.deliver(envelope(3, 3, key(PhysicalKey::B))),
        Err(Error::StaleInput)
    );
    ui.deliver(envelope(4, 4, key(PhysicalKey::C))).unwrap();
    assert_eq!(ui.view().contents, Some("ac"));
    let mut wrong = envelope(5, 4, key(PhysicalKey::D));
    wrong.epoch += 1;
    assert_eq!(ui.deliver(wrong), Err(Error::InputEpoch));
    assert_eq!(ui.view().contents, Some("ac"));
}

fn mouse_click_uses_real_ps2_buttons_and_release_must_match_impl() {
    let host = Host::new();
    let (mut d, mut o, mut s) = ([0; 512], [0; 512], [0; 512]);
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
    let mut mouse = Ps2Mouse::new();
    let mut sequence = 0;
    for (flags, x, y) in [(0x09, 40, 90), (0x08, 40, 90)] {
        let mut event = None;
        for byte in [flags, 0, 0] {
            event = mouse.feed(byte).unwrap();
        }
        sequence += 1;
        ui.deliver(envelope(
            sequence,
            1,
            Event::Mouse {
                input: event.unwrap(),
                x,
                y,
            },
        ))
        .unwrap();
    }
    assert_eq!(ui.view().screen, Screen::Editor);
    assert!(ui.document().is_dirty());
    // Press Save, cancel input ownership, then release on Save: no BEGIN.
    let save = Frame::new(ui.view()).layout().save;
    for (flags, cancel) in [(0x09, false), (0x08, true)] {
        let mut event = None;
        for byte in [flags, 0, 0] {
            event = mouse.feed(byte).unwrap();
        }
        if cancel {
            sequence += 1;
            ui.deliver(envelope(sequence, 1, Event::Cancel(CancelReason::Resync)))
                .unwrap();
        }
        sequence += 1;
        let result = ui.deliver(envelope(
            sequence,
            if cancel { 2 } else { 1 },
            Event::Mouse {
                input: event.unwrap(),
                x: save.x + 10,
                y: save.y + 10,
            },
        ));
        assert!(result.is_ok());
    }
    assert!(!host
        .borrow()
        .requests
        .iter()
        .any(|r| r.operation == wire::BEGIN));
}

fn readonly_and_unsupported_utf8_do_not_corrupt_bytes_impl() {
    for invalid in [false, true] {
        let host = Host::new();
        host.borrow_mut()
            .seed(0, if invalid { &[0xff, b'a'] } else { b"Text" });
        let (mut d, mut o, mut s) = ([0; 512], [0; 512], [0; 512]);
        let mut ui = controller(
            &host,
            EditMode::ReadOnly,
            Buffers {
                draft: &mut d,
                original: &mut o,
                staging: &mut s,
            },
        );
        if invalid {
            assert_eq!(ui.open(0), Err(Error::InvalidUtf8));
            assert_eq!(ui.document().draft().unwrap().contents, &[0xff, b'a']);
        } else {
            ui.open(0).unwrap();
            assert_eq!(ui.insert("x"), Err(Error::Storage(StorageError::ReadOnly)));
            assert_eq!(ui.view().contents, Some("Text"));
            assert_eq!(ui.save(), Err(Error::Storage(StorageError::ReadOnly)));
        }
        assert!(!ui.view().can_save);
    }
}

fn explicit_cleanup_refuses_dirty_and_never_reconfirms_closed_work_impl() {
    let host = Host::new();
    let (mut d, mut o, mut s) = ([0; 512], [0; 512], [0; 512]);
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
    assert_eq!(
        ui.close_clean(),
        Err(Error::Storage(StorageError::UnsavedChanges))
    );
    assert!(host.borrow().closed.is_empty());
    ui.insert("Keep").unwrap();
    ui.save().unwrap();
    while !ui.advance_save().unwrap() {}
    ui.close_clean().unwrap();
    assert_eq!(host.borrow().closed.len(), 2);
    assert_eq!(ui.save(), Err(Error::Storage(StorageError::Closed)));
    assert!(!matches!(ui.view().status, Status::Saved { .. }));
    assert_eq!(ui.view().contents, Some("Keep"));
}

fn bounded_buffers_full_input_and_64kib_document_are_not_silent_truncation_impl() {
    for size in [8, volume_core::MAX_LENGTH] {
        let host = Host::new();
        let (mut d, mut o, mut s) = (vec![0; size], vec![0; size], vec![0; size]);
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
        let content = "é".repeat(size / 2);
        ui.insert(&content).unwrap();
        assert_eq!(
            ui.insert("x"),
            Err(Error::Storage(StorageError::ContentTooLarge))
        );
        assert_eq!(ui.view().contents, Some(content.as_str()));
        ui.save().unwrap();
        while !ui.advance_save().unwrap() {}
        ui.open(0).unwrap();
        assert_eq!(ui.view().contents, Some(content.as_str()));
    }
}

fn utf8_navigation_wrap_crlf_tabs_and_range_boundaries_are_checked_impl() {
    let text = "aé\tZ\r\nb";
    assert_eq!(text::previous(text, 3), 1);
    assert_eq!(text::next(text, 1), 3);
    assert_eq!(text::next(text, 2), 3);
    assert_eq!(text::boundary(text, usize::MAX), text.len());
    assert_eq!(text::position(text, 6, 8).row, 0);
    assert_eq!(text::position(text, 7, 8).row, 1);
    assert_eq!(text::selection(7, 1), 1..7);
    for columns in 4..40 {
        for byte in 0..=text.len() {
            let at = text::boundary(text, byte);
            let p = text::position(text, at, columns);
            let back = text::byte_at(text, p, columns);
            assert!(text.is_char_boundary(back));
        }
    }
}

fn real_raster_tiles_have_guards_clipping_lowercase_and_semantic_focus_impl() {
    for scale in [Scale::Percent100, Scale::Percent200] {
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
        ui.insert("Café\nabc XYZ\nSelectie blijft UTF-8.").unwrap();
        ui.resize(Viewport::new(803, 603, scale, 4).unwrap())
            .unwrap();
        let frame = Frame::new(ui.view());
        let mut count = 0;
        let mut nonbackground = 0;
        for tile in frame.tiles() {
            let mut pixels = [0xdead_beef; TILE_PIXELS + 2];
            let rendered = frame.render_tile(tile.x, tile.y, &mut pixels[1..]).unwrap();
            assert_eq!(rendered, tile);
            let length = (tile.width * tile.height) as usize;
            assert_eq!(pixels[0], 0xdead_beef);
            assert!(pixels[length + 1..].iter().all(|p| *p == 0xdead_beef));
            assert!(pixels[1..=length].iter().all(|p| p & 0xff00_0000 == 0));
            nonbackground += pixels[1..=length]
                .iter()
                .filter(|p| **p != design::BACKGROUND)
                .count();
            count += length;
        }
        assert_eq!(count, 803 * 603);
        assert!(nonbackground > 1000);
        let mut nodes = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
        let n = frame.semantics(&mut nodes).unwrap();
        assert_eq!(nodes[..n].iter().filter(|n| n.focused).count(), 1);
        let editor = nodes[..n]
            .iter()
            .find(|n| n.role == Role::TextEditor)
            .unwrap();
        assert_eq!(editor.value, Some("Café\nabc XYZ\nSelectie blijft UTF-8."));
        assert_eq!(
            editor.selection,
            Some((
                "Café\nabc XYZ\nSelectie blijft UTF-8.".len(),
                "Café\nabc XYZ\nSelectie blijft UTF-8.".len()
            ))
        );
    }
    assert_ne!(
        noto_sans_mono_bitmap::get_raster(
            'a',
            noto_sans_mono_bitmap::FontWeight::Regular,
            noto_sans_mono_bitmap::RasterHeight::Size16
        )
        .unwrap()
        .raster(),
        noto_sans_mono_bitmap::get_raster(
            'A',
            noto_sans_mono_bitmap::FontWeight::Regular,
            noto_sans_mono_bitmap::RasterHeight::Size16
        )
        .unwrap()
        .raster()
    );
}

fn display_epoch_stale_frame_and_failed_present_keep_repaint_pending_impl() {
    let host = Host::new();
    let (mut d, mut o, mut s) = ([0; 512], [0; 512], [0; 512]);
    let mut ui = controller(
        &host,
        EditMode::ReadWrite,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    let generation = ui.view().generation;
    assert!(!ui.presented(generation, 2));
    assert!(ui.view().dirty);
    ui.resize(Viewport::new(640, 480, Scale::Percent200, 2).unwrap())
        .unwrap();
    assert!(!ui.presented(generation, 1));
    assert!(!ui.presented(generation, 2));
    let generation = ui.view().generation;
    assert!(ui.presented(generation, 2));
    assert!(Frame::new(ui.view()).tiles().next().is_none());
    ui.refresh().unwrap();
    assert!(ui.view().dirty);
}

// The existing capability/volume host model is large by value. Match the
// SDK harness stack, without changing the native process-stack contract.
macro_rules! scenario {
    ($name:ident,$implementation:ident) => {
        #[test]
        fn $name() {
            std::thread::Builder::new()
                .stack_size(16 * 1024 * 1024)
                .spawn($implementation)
                .unwrap()
                .join()
                .unwrap();
        }
    };
}
scenario!(
    real_ps2_edit_stream_commit_list_and_reopen,
    real_ps2_edit_stream_commit_list_and_reopen_impl
);
scenario!(
    abort_retains_draft_and_does_not_publish_an_unsaved_revision,
    abort_retains_draft_and_does_not_publish_an_unsaved_revision_impl
);
scenario!(
    lost_commit_never_becomes_saved_or_automatic_replay,
    lost_commit_never_becomes_saved_or_automatic_replay_impl
);
scenario!(
    conflict_revocation_and_io_error_keep_unconfirmed_work_visible,
    conflict_revocation_and_io_error_keep_unconfirmed_work_visible_impl
);
scenario!(
    wrong_reply_identity_cannot_populate_a_fake_object_list,
    wrong_reply_identity_cannot_populate_a_fake_object_list_impl
);
scenario!(
    dirty_close_defaults_to_cancel_and_only_explicit_discard_loses_edits,
    dirty_close_defaults_to_cancel_and_only_explicit_discard_loses_edits_impl
);
scenario!(
    replay_cancel_and_old_focus_cannot_trigger_a_new_edit,
    replay_cancel_and_old_focus_cannot_trigger_a_new_edit_impl
);
scenario!(
    mouse_click_uses_real_ps2_buttons_and_release_must_match,
    mouse_click_uses_real_ps2_buttons_and_release_must_match_impl
);
scenario!(
    readonly_and_unsupported_utf8_do_not_corrupt_bytes,
    readonly_and_unsupported_utf8_do_not_corrupt_bytes_impl
);
scenario!(
    bounded_buffers_full_input_and_64kib_document_are_not_silent_truncation,
    bounded_buffers_full_input_and_64kib_document_are_not_silent_truncation_impl
);
scenario!(
    utf8_navigation_wrap_crlf_tabs_and_range_boundaries_are_checked,
    utf8_navigation_wrap_crlf_tabs_and_range_boundaries_are_checked_impl
);
scenario!(
    real_raster_tiles_have_guards_clipping_lowercase_and_semantic_focus,
    real_raster_tiles_have_guards_clipping_lowercase_and_semantic_focus_impl
);
scenario!(
    display_epoch_stale_frame_and_failed_present_keep_repaint_pending,
    display_epoch_stale_frame_and_failed_present_keep_repaint_pending_impl
);
scenario!(
    explicit_cleanup_refuses_dirty_and_never_reconfirms_closed_work,
    explicit_cleanup_refuses_dirty_and_never_reconfirms_closed_work_impl
);

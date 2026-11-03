mod common;
use common::*;
use nieuw_os_document_session::{volume::Buffers, EditMode};
use nieuw_os_input_core::Event;
use nieuw_os_materiaal::{
    controller::{Error, Focus, InputEnvelope, Screen, Status, MAX_SEARCH_BYTES},
    render::{Frame, SemanticId, SemanticNode, MAX_SEMANTIC_NODES},
    Scale, Viewport,
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

fn input(sequence: u64, key: PhysicalKey, control: bool) -> InputEnvelope {
    InputEnvelope {
        epoch: INPUT_EPOCH,
        sequence,
        generation: 1,
        event: Event::Key(KeyboardEvent {
            key,
            action: KeyAction::Press,
            modifiers: KeyboardModifiers {
                left_control: control,
                ..KeyboardModifiers::default()
            },
        }),
    }
}

fn named_document_save_search_reopen_and_rename() {
    let host = Host::new();
    host.borrow_mut()
        .seed_named(0, b"Other", b"Keep other bytes");
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
    ui.create().unwrap();
    let contents = "Café\nExact dezelfde inhoud na heropenen.";
    ui.set_semantic_value(
        SemanticId::DocumentName,
        ui.view().generation,
        "Reisnotities",
    )
    .unwrap();
    ui.set_semantic_value(SemanticId::Document, ui.view().generation, contents)
        .unwrap();
    ui.save().unwrap();
    while !ui.advance_save().unwrap() {}
    ui.request_back().unwrap();
    let calls = host.borrow().requests.len();
    ui.set_semantic_value(SemanticId::Search, ui.view().generation, "REIS")
        .unwrap();
    assert_eq!(host.borrow().requests.len(), calls);
    assert_eq!(ui.view().objects.len(), 1);
    assert_eq!(ui.view().objects[0].object, 1);
    ui.deliver(input(1, PhysicalKey::Enter, false)).unwrap();
    assert_eq!(ui.view().contents, Some(contents));
    assert_eq!(ui.view().title, b"Reisnotities");
    ui.deliver(input(2, PhysicalKey::F2, false)).unwrap();
    assert_eq!(ui.view().focus, Focus::DocumentName);
    assert_eq!(ui.view().field_anchor, 0);
    assert_eq!(ui.view().field_caret, b"Reisnotities".len());
    ui.deliver(input(3, PhysicalKey::A, false)).unwrap();
    assert_eq!(ui.view().title, b"a");
    assert!(ui.view().document_dirty);
    ui.deliver(input(4, PhysicalKey::S, true)).unwrap();
    while !ui.advance_save().unwrap() {}
    ui.request_back().unwrap();
    assert!(ui.view().objects.is_empty());
    ui.deliver(input(5, PhysicalKey::Escape, false)).unwrap();
    assert!(ui.view().query.is_empty());
    assert_eq!(ui.view().objects.len(), 2);
    ui.set_semantic_value(SemanticId::Search, ui.view().generation, "a")
        .unwrap();
    ui.deliver(input(6, PhysicalKey::Enter, false)).unwrap();
    assert_eq!(ui.view().contents, Some(contents));
    assert_eq!(ui.view().title, b"a");
    ui.close_clean().unwrap();
    assert_eq!(host.borrow().closed.len(), 2);
    let mut h = host.borrow_mut();
    assert_eq!(h.bytes(1), contents.as_bytes());
    assert_eq!(h.bytes(0), b"Keep other bytes");
    assert_eq!(h.recover().entries[1].revision, 2);
}

fn filtered_create_uses_all_slots_and_readonly_search_remains_available() {
    let host = Host::new();
    host.borrow_mut().seed_named(0, b"Caf\xc3\xa9", b"One");
    host.borrow_mut().seed_named(1, b"Elsewhere", b"Two");
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
    ui.set_semantic_value(SemanticId::Search, ui.view().generation, "CAFÉ")
        .unwrap();
    assert_eq!(ui.view().objects.len(), 1);
    ui.create().unwrap();
    assert_eq!(ui.document().draft().unwrap().object, 2);
    assert_eq!(host.borrow_mut().bytes(0), b"One");
    assert_eq!(host.borrow_mut().bytes(1), b"Two");

    let other = Host::new();
    other.borrow_mut().seed_named(0, b"Read only", b"Content");
    let (mut d, mut o, mut s) = ([0; 512], [0; 512], [0; 512]);
    let mut readonly = controller(
        &other,
        EditMode::ReadOnly,
        Buffers {
            draft: &mut d,
            original: &mut o,
            staging: &mut s,
        },
    );
    readonly.refresh().unwrap();
    readonly
        .set_semantic_value(SemanticId::Search, readonly.view().generation, "ONLY")
        .unwrap();
    readonly
        .deliver(input(1, PhysicalKey::Enter, false))
        .unwrap();
    let before = other.borrow().requests.len();
    assert_eq!(
        readonly.set_semantic_value(
            SemanticId::DocumentName,
            readonly.view().generation,
            "Forbidden"
        ),
        Err(Error::InvalidInput)
    );
    assert_eq!(other.borrow().requests.len(), before);
    assert_eq!(readonly.view().title, b"Read only");
}

fn invalid_stale_and_failed_renames_do_not_publish() {
    let host = Host::new();
    host.borrow_mut().seed_named(0, b"Original", b"Contents");
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
    let old = ui.view().generation;
    ui.open(0).unwrap();
    assert_eq!(
        ui.set_semantic_value(SemanticId::DocumentName, old, "Stale"),
        Err(Error::StaleInput)
    );
    for name in [
        "",
        "..",
        "Bad/name",
        "Bad\nname",
        "\u{202e}name",
        &"x".repeat(33),
    ] {
        assert!(ui
            .set_semantic_value(SemanticId::DocumentName, ui.view().generation, name)
            .is_err());
        assert_eq!(ui.view().title, b"Original");
    }
    ui.set_semantic_value(SemanticId::DocumentName, ui.view().generation, "Changed")
        .unwrap();
    ui.request_back().unwrap();
    assert_eq!(ui.view().screen, Screen::ConfirmClose);
    ui.activate_semantic(SemanticId::Discard, ui.view().generation)
        .unwrap();
    ui.open(0).unwrap();
    assert_eq!(ui.view().title, b"Original");
    ui.set_semantic_value(
        SemanticId::DocumentName,
        ui.view().generation,
        "Unconfirmed",
    )
    .unwrap();
    ui.save().unwrap();
    assert_eq!(
        ui.set_semantic_value(SemanticId::DocumentName, ui.view().generation, "Busy"),
        Err(Error::InvalidInput)
    );
    host.borrow_mut().disk.fail = true;
    while matches!(ui.advance_save(), Ok(false)) {}
    assert!(matches!(ui.view().status, Status::Failed(_)));
    assert_eq!(ui.view().title, b"Unconfirmed");
    assert!(ui.view().document_dirty);
    host.borrow_mut().disk.fail = false;
    assert_eq!(host.borrow_mut().recover().entries[0].name(), b"Original");
}

fn search_keyboard_semantics_and_bounds_at_both_scales() {
    for scale in [Scale::Percent100, Scale::Percent200] {
        let host = Host::new();
        host.borrow_mut().seed_named(0, b"Caf\xc3\xa9", b"Content");
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
        ui.resize(Viewport::new(320 * scale.factor(), 240 * scale.factor(), scale, 2).unwrap())
            .unwrap();
        ui.deliver(input(1, PhysicalKey::F, true)).unwrap();
        assert_eq!(ui.view().focus, Focus::Search);
        ui.deliver(input(2, PhysicalKey::C, false)).unwrap();
        assert_eq!(ui.view().query, "c");
        assert_eq!(ui.view().objects.len(), 1);
        let stale = ui.view().generation;
        ui.set_semantic_value(SemanticId::Search, stale, "Café")
            .unwrap();
        assert_eq!(
            ui.set_semantic_value(SemanticId::Search, stale, "wrong"),
            Err(Error::StaleInput)
        );
        ui.deliver(input(3, PhysicalKey::Backspace, false)).unwrap();
        assert_eq!(ui.view().query, "Caf");
        let generation = ui.view().generation;
        assert!(ui
            .set_semantic_value(
                SemanticId::Search,
                generation,
                &"x".repeat(MAX_SEARCH_BYTES + 1)
            )
            .is_err());
        assert_eq!(ui.view().query, "Caf");
        let frame = Frame::new(ui.view());
        let mut nodes = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
        let count = frame.semantics(&mut nodes).unwrap();
        let node = nodes[..count]
            .iter()
            .find(|node| node.id == SemanticId::Search)
            .unwrap();
        assert!(node.actions.set_value && node.focused);
        assert!(node.bounds.x + node.bounds.width <= 320 * scale.factor());
        assert!(node.bounds.y + node.bounds.height <= 240 * scale.factor());
        let mut tile = [0; 256];
        frame
            .render_tile(node.bounds.x, node.bounds.y, &mut tile)
            .unwrap();
    }
}

macro_rules! scenario {
    ($name:ident) => {
        #[test]
        fn $name() {
            std::thread::Builder::new()
                .stack_size(16 * 1024 * 1024)
                .spawn(super::$name)
                .unwrap()
                .join()
                .unwrap();
        }
    };
}

fn revoked_save_keeps_name_and_contents_unpublished() {
    let host = Host::new();
    host.borrow_mut().seed(0, b"Original");
    let (mut draft, mut original, mut staging) = ([0; 128], [0; 128], [0; 128]);
    let mut ui = controller(
        &host,
        EditMode::ReadWrite,
        Buffers {
            draft: &mut draft,
            original: &mut original,
            staging: &mut staging,
        },
    );
    ui.refresh().unwrap();
    ui.open(0).unwrap();
    ui.set_semantic_value(
        SemanticId::DocumentName,
        ui.view().generation,
        "Private draft",
    )
    .unwrap();
    let root = host.borrow().request_root;
    host.borrow_mut().engine.revoke(2, root).unwrap();
    assert!(ui.save().is_err());
    assert!(ui.document().is_dirty());
    assert!(!matches!(ui.view().status, Status::Saved { .. }));
    let mut h = host.borrow_mut();
    assert_eq!(h.recover().entries[0].name(), b"Document");
    assert_eq!(h.bytes(0), b"Original");
}

mod scenarios {
    scenario!(named_document_save_search_reopen_and_rename);
    scenario!(filtered_create_uses_all_slots_and_readonly_search_remains_available);
    scenario!(invalid_stale_and_failed_renames_do_not_publish);
    scenario!(search_keyboard_semantics_and_bounds_at_both_scales);
    scenario!(revoked_save_keeps_name_and_contents_unpublished);
}

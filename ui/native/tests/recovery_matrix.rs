//! U05-R1: Materiaal save-recovery matrix (UX-SR06/07/08/11), host level only.
//!
//! Drives the real controller and the v7 volume service through the shared
//! `common` harness, without editing it. It covers only the failure classes that
//! `interaction.rs` does not already assert:
//!
//! - `Service(Quota)`: domain 0 is pre-filled to its 40-chunk quota, so BEGIN of
//!   the next revision is refused.
//! - `Service(Full)`: domain 1 (through a local domain-1 transport over the
//!   public `Host::service`/`Host::disk`) and domain 0 together leave 7 of 64
//!   chunks free while the draft needs 8.
//! - `Protocol` at BEGIN: `Host::wrong_peer` forges the reply sender.
//! - `Protocol` at COMMIT: the same forgery on the COMMIT reply; the session
//!   must report `Uncertain`, never Saved, and offer no retry.
//! - `Interrupted`: the request endpoint is revoked, so the BEGIN send fails in
//!   the transport (not a service deny; `Service(Access)` belongs to U03-T3
//!   part 2 and is only listed in the ignored copy test).
//!
//! Per row, at 100 % and 200 %: draft bytes, caret and selection anchor are
//! unchanged; exactly one semantic node is focused and it is enabled; the Status
//! node text equals the drawn `status_label` and its word-wrapped lines fit the
//! status rectangle; nothing shows Saved without a COMMIT ACK; no request is
//! replayed. Focus is checked through semantic nodes, never by control index.
//!
//! Dropped: none of the requested rows. The status line is word-wrapped
//! (`Painter::wrapped`), not elided, so "fits its elision width" is checked as
//! "all wrapped lines fit inside `Layout::status`" with the renderer's font.
//! The matrix uses 800x600 @ 100 % and 1600x900 @ 200 % (both 800-wide logical
//! layouts with two status lines). Known defects, as ignored tests that fail
//! today and route to the `render.rs` owner / UX writing: distinct target copy
//! (UX-SR07) and the compact 640x480 @ 200 % status rect clipping the second
//! line of the generic failure label.
mod common;
use common::*;
use nieuw_os_document_session::{
    volume::{Buffers, Error as StorageError},
    EditMode,
};
use nieuw_os_graphics::{Font, Size, Weight};
use nieuw_os_input_core::Event;
use nieuw_os_materiaal::{
    controller::{Error, InputEnvelope, Status},
    design,
    render::{status_label, Frame, Layout, Role, SemanticId, SemanticNode, MAX_SEMANTIC_NODES},
    Scale, Viewport,
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};
use volume_core::{
    client::{Client, Transport as CoreTransport},
    service::Service,
    wire, Status as VolumeStatus, CHUNK, MAX_LENGTH,
};

const GENERIC: &str = "Handeling mislukt. Je tekst blijft open.";

#[derive(Clone, Copy, Debug, PartialEq)]
enum Fault {
    Quota,
    Full,
    ProtocolAtBegin,
    ProtocolAtCommit,
    Interrupted,
}

/// Domain-1 writer over the shared service, mirroring the private domain-0
/// `Direct` in `common` so another app can consume volume capacity.
struct OtherDomain<'a, D>(&'a mut Service, &'a mut D);
impl<D: volume_core::Disk> CoreTransport for OtherDomain<'_, D> {
    fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], VolumeStatus> {
        Ok(self.0.handle(1, request, self.1))
    }
}

fn shift_left() -> Event {
    Event::Key(KeyboardEvent {
        key: PhysicalKey::Left,
        action: KeyAction::Press,
        modifiers: KeyboardModifiers {
            left_shift: true,
            ..KeyboardModifiers::default()
        },
    })
}

fn commits(host: &std::rc::Rc<std::cell::RefCell<Host>>) -> usize {
    host.borrow()
        .requests
        .iter()
        .filter(|request| request.operation == wire::COMMIT)
        .count()
}

/// Lines `Painter::wrapped` draws for `label` within `width` logical pixels.
fn wrapped_lines(label: &str, width: u32, scale: Scale) -> u32 {
    let font = Font::new(Size::Body, Weight::Regular, scale == Scale::Percent200);
    let measure = |s: &str| font.measure(s).div_ceil(scale.factor());
    let (mut rest, mut lines) = (label, 0);
    while !rest.is_empty() {
        let mut take = rest.len();
        let mut space = None;
        for (index, ch) in rest.char_indices() {
            if ch.is_whitespace() {
                space = Some(index);
            }
            if measure(&rest[..index + ch.len_utf8()]) > width {
                take = space.filter(|i| *i > 0).unwrap_or(index.max(ch.len_utf8()));
                break;
            }
        }
        assert!(
            measure(rest[..take].trim_end()) <= width,
            "unbreakable status word overflows: {label:?}"
        );
        rest = rest[take..].trim_start_matches(char::is_whitespace);
        lines += 1;
    }
    lines
}

fn expected_status(fault: Fault, status: Status) -> bool {
    match fault {
        Fault::Quota => {
            status == Status::Failed(Error::Storage(StorageError::Service(VolumeStatus::Quota)))
        }
        Fault::Full => {
            status == Status::Failed(Error::Storage(StorageError::Service(VolumeStatus::Full)))
        }
        Fault::ProtocolAtBegin => matches!(
            status,
            Status::Failed(Error::Storage(StorageError::Protocol { .. }))
        ),
        Fault::ProtocolAtCommit => {
            status == Status::Failed(Error::Storage(StorageError::Uncertain))
        }
        Fault::Interrupted => matches!(
            status,
            Status::Failed(Error::Storage(StorageError::Interrupted { .. }))
        ),
    }
}

fn recovery_row(fault: Fault) {
    let host = Host::new();
    host.borrow_mut().seed(0, b"Original");
    match fault {
        // 16 + 16 + 7 + 1 (object 0) = 40 chunks: the quota is exactly used.
        Fault::Quota => {
            let mut h = host.borrow_mut();
            h.seed(1, &vec![b'q'; MAX_LENGTH]);
            h.seed(2, &vec![b'q'; MAX_LENGTH]);
            h.seed(3, &vec![b'q'; 7 * CHUNK]);
        }
        // Domain 1 uses 40, domain 0 uses 17: 7 chunks free, 8 needed below.
        Fault::Full => {
            let mut h = host.borrow_mut();
            h.seed(1, &vec![b'f'; MAX_LENGTH]);
            let h = &mut *h;
            let mut other = Client::new(OtherDomain(&mut h.service, &mut h.disk), EPOCH, 500_000);
            for (object, length) in [(8, MAX_LENGTH), (9, MAX_LENGTH), (10, 8 * CHUNK)] {
                let bytes = vec![b'o'; length];
                other
                    .write(object, 0, b"Other", length, |offset, out| {
                        out.copy_from_slice(&bytes[offset..offset + out.len()])
                    })
                    .unwrap();
            }
        }
        _ => {}
    }
    let (mut d, mut o, mut s) = (
        vec![0; MAX_LENGTH],
        vec![0; MAX_LENGTH],
        vec![0; MAX_LENGTH],
    );
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
    let prefix = if fault == Fault::Full {
        "a".repeat(7 * CHUNK + 100)
    } else {
        "Mine ".to_owned()
    };
    ui.insert(&prefix).unwrap();
    for sequence in 1..=2 {
        ui.deliver(InputEnvelope {
            epoch: INPUT_EPOCH,
            sequence,
            generation: 1,
            event: shift_left(),
        })
        .unwrap();
    }
    let draft = ui.view().contents.unwrap().to_owned();
    let (caret, anchor) = (ui.view().caret, ui.view().anchor);
    assert_eq!(draft, format!("{prefix}Original"));
    assert_eq!((caret + 2, anchor), (prefix.len(), prefix.len()));

    match fault {
        Fault::ProtocolAtBegin => host.borrow_mut().wrong_peer = true,
        Fault::Interrupted => {
            let root = host.borrow().request_root;
            host.borrow_mut().engine.revoke(2, root).unwrap();
        }
        _ => {}
    }
    let result = ui.save().and_then(|()| {
        loop {
            if fault == Fault::ProtocolAtCommit && commits(&host) == 0 {
                // The next exchange after the last APPEND is COMMIT.
                let appended = host
                    .borrow()
                    .requests
                    .iter()
                    .filter(|r| r.operation == wire::APPEND)
                    .count();
                if appended > 0 {
                    host.borrow_mut().wrong_peer = true;
                }
            }
            if ui.advance_save()? {
                return Ok(());
            }
        }
    });
    assert!(result.is_err(), "{fault:?} must not complete a save");
    let status = ui.view().status;
    assert!(expected_status(fault, status), "{fault:?} -> {status:?}");
    let requests = host.borrow().requests.len();
    let saved = status_label(Status::Saved { revision: 2 });

    for (width, height, scale) in [
        (800, 600, Scale::Percent100),
        (1600, 900, Scale::Percent200),
    ] {
        ui.resize(Viewport::new(width, height, scale, 1).unwrap())
            .unwrap();
        let view = ui.view();
        assert_eq!(view.contents, Some(draft.as_str()), "{fault:?} {scale:?}");
        assert_eq!(
            (view.caret, view.anchor),
            (caret, anchor),
            "{fault:?} {scale:?}"
        );
        assert!(ui.document().is_dirty() && !ui.document().is_saving());
        assert_eq!(view.status, status, "{fault:?} {scale:?}: status changed");

        let frame = Frame::new(ui.view());
        let mut nodes = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
        let count = frame.semantics(&mut nodes).unwrap();
        let nodes = &nodes[..count];
        let focused: Vec<_> = nodes.iter().filter(|node| node.focused).collect();
        assert_eq!(focused.len(), 1, "{fault:?} {scale:?}: {focused:?}");
        assert!(
            focused[0].enabled,
            "{fault:?} {scale:?}: focus on disabled {focused:?}"
        );
        let save = nodes.iter().find(|n| n.id == SemanticId::Save).unwrap();
        assert_eq!(save.enabled, ui.view().can_save);
        assert_eq!(save.actions.activate, ui.view().can_save);

        let status_node = nodes.iter().find(|n| n.id == SemanticId::Status).unwrap();
        assert_eq!(status_node.role, Role::Status);
        assert!(!status_node.label.is_empty());
        assert_eq!(status_node.label, status_label(ui.view().status));
        assert_ne!(status_node.label, saved);
        assert!(nodes
            .iter()
            .all(|n| n.label != saved && n.value != Some(saved)));

        let layout = frame.layout();
        let status_width = ui.view().viewport.logical_width() - 2 * design::PAD;
        let lines = wrapped_lines(status_node.label, status_width, scale);
        assert!(
            lines * design::LINE_HEIGHT <= layout.status.height,
            "{fault:?} {scale:?}: {lines} status lines exceed {}",
            layout.status.height
        );
        assert_eq!(status_node.bounds, layout.status.physical(scale));

        if fault == Fault::ProtocolAtCommit {
            assert!(ui.document().is_uncertain() && !ui.view().can_save);
            assert!(nodes.iter().all(|n| !n.label.contains("Opnieuw")));
            assert!(!save.enabled && !save.focused);
        }
        if matches!(fault, Fault::ProtocolAtBegin | Fault::Interrupted) {
            assert!(ui.document().is_closed() && !ui.view().can_save);
        }
        assert_eq!(host.borrow().requests.len(), requests, "no replay");
    }
    if fault == Fault::ProtocolAtCommit {
        assert_eq!(ui.save(), Err(Error::Storage(StorageError::Uncertain)));
        assert_eq!(host.borrow().requests.len(), requests);
    }

    let mut h = host.borrow_mut();
    h.wrong_peer = false;
    if fault == Fault::ProtocolAtCommit {
        // The service applied COMMIT, but its reply was not authenticated:
        // the UI correctly reports Uncertain rather than Saved.
        assert_eq!(h.recover().entries[0].revision, 2);
    } else {
        assert_eq!(h.recover().entries[0].revision, 1);
        assert_eq!(h.bytes(0), b"Original");
        assert!(!h.requests.iter().any(|r| r.operation == wire::COMMIT));
    }
}

macro_rules! scenario {
    ($name:ident, $fault:expr) => {
        #[test]
        fn $name() {
            std::thread::Builder::new()
                .stack_size(16 * 1024 * 1024)
                .spawn(|| recovery_row($fault))
                .unwrap()
                .join()
                .unwrap();
        }
    };
}
scenario!(quota_keeps_draft_focus_and_truthful_status, Fault::Quota);
scenario!(full_keeps_draft_focus_and_truthful_status, Fault::Full);
scenario!(
    protocol_at_begin_closes_without_saved,
    Fault::ProtocolAtBegin
);
scenario!(
    protocol_at_commit_is_uncertain_without_retry,
    Fault::ProtocolAtCommit
);
scenario!(interrupted_send_closes_without_saved, Fault::Interrupted);

fn failed(error: StorageError) -> &'static str {
    status_label(Status::Failed(Error::Storage(error)))
}

#[test]
#[ignore = "U05 defect: status_label maps Service(Full) and Service(Quota) to generic copy (render.rs:1750); UX-SR07 needs distinct Full/Quota copy"]
fn sr07_full_and_quota_have_distinct_copy() {
    let full = failed(StorageError::Service(VolumeStatus::Full));
    let quota = failed(StorageError::Service(VolumeStatus::Quota));
    assert_ne!(full, GENERIC, "Full shows generic copy");
    assert_ne!(quota, GENERIC, "Quota shows generic copy");
    assert_ne!(full, quota, "Full and Quota are indistinguishable");
}

#[test]
#[ignore = "U05 defect: status_label maps a known Service(Io) error to generic copy (render.rs:1750); UX-SR07 needs known-I/O copy"]
fn sr07_known_io_error_has_specific_copy() {
    let io = failed(StorageError::Service(VolumeStatus::Io));
    assert_ne!(io, GENERIC, "known I/O shows generic copy");
    assert_ne!(io, failed(StorageError::Service(VolumeStatus::Full)));
}

#[test]
#[ignore = "U05 defect: status_label maps Service(Access) to generic copy (render.rs:1750); deny copy is planned in U03-T3 part 2"]
fn sr07_deny_has_specific_copy() {
    let deny = failed(StorageError::Service(VolumeStatus::Access));
    assert_ne!(deny, GENERIC, "deny shows generic copy");
    assert_ne!(deny, failed(StorageError::Service(VolumeStatus::Io)));
}

#[test]
#[ignore = "U05 defect: at 640x480 @ 200 % the compact status rect holds one line, but the generic failure label wraps to two and the second is drawn below the viewport (render.rs status layout)"]
fn sr11_compact_200_percent_failure_status_is_not_clipped() {
    let viewport = Viewport::new(640, 480, Scale::Percent200, 1).unwrap();
    let status = Layout::new(viewport).status;
    let width = viewport.logical_width() - 2 * design::PAD;
    for label in [
        GENERIC,
        failed(StorageError::Uncertain),
        failed(StorageError::Conflict),
    ] {
        let lines = wrapped_lines(label, width, Scale::Percent200);
        assert!(
            lines * design::LINE_HEIGHT <= status.height,
            "{label:?}: {lines} lines in a {} px status rect",
            status.height
        );
    }
}

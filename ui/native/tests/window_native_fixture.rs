//! Host oracle for actual window-body QMP comparisons. No native execution,
//! compositor, authentication, persistence or physical-device claim is made.
#[allow(dead_code)]
mod common;
use common::{controller, Host, Port, INPUT_EPOCH};
use nieuw_os_document_session::{volume::Buffers, EditMode};
use nieuw_os_input_core::{CancelReason, Event};
use nieuw_os_materiaal::{
    controller::{Controller, Focus, InputEnvelope, Screen, Status},
    render::Frame,
    Scale, Viewport, TILE_HEIGHT, TILE_PIXELS, TILE_WIDTH,
};
use std::{io::Write, path::Path};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, MouseButtons, MouseEvent, PhysicalKey};

const TEXT: &str = "nieuw os werkt";
const PROFILES: [(u32, u32, Scale); 2] = [
    (880, 510, Scale::Percent100),
    (1120, 660, Scale::Percent200),
];
fn key(
    ui: &mut Controller<'_, Port>,
    epoch: u64,
    sequence: &mut u64,
    generation: u64,
    physical: PhysicalKey,
    control: bool,
) {
    modified_key(
        ui,
        epoch,
        sequence,
        generation,
        physical,
        KeyboardModifiers {
            left_control: control,
            ..KeyboardModifiers::default()
        },
    );
}
fn modified_key(
    ui: &mut Controller<'_, Port>,
    epoch: u64,
    sequence: &mut u64,
    generation: u64,
    physical: PhysicalKey,
    modifiers: KeyboardModifiers,
) {
    for action in [KeyAction::Press, KeyAction::Release] {
        *sequence += 1;
        ui.deliver(InputEnvelope {
            epoch,
            sequence: *sequence,
            generation,
            event: Event::Key(KeyboardEvent {
                key: physical,
                action,
                modifiers,
            }),
        })
        .unwrap();
    }
}
fn text(ui: &mut Controller<'_, Port>, sequence: &mut u64) {
    for physical in [
        PhysicalKey::N,
        PhysicalKey::I,
        PhysicalKey::E,
        PhysicalKey::U,
        PhysicalKey::W,
        PhysicalKey::Space,
        PhysicalKey::O,
        PhysicalKey::S,
        PhysicalKey::Space,
        PhysicalKey::W,
        PhysicalKey::E,
        PhysicalKey::R,
        PhysicalKey::K,
        PhysicalKey::T,
    ] {
        key(ui, INPUT_EPOCH, sequence, 1, physical, false);
    }
    assert_eq!(ui.view().contents, Some(TEXT));
}
fn snapshot(ui: &Controller<'_, Port>, phase: &str, directory: Option<&Path>) {
    assert_eq!(ui.view().cursor, None);
    snapshot_frame(ui, phase, directory);
}
fn snapshot_frame(ui: &Controller<'_, Port>, phase: &str, directory: Option<&Path>) {
    let view = ui.view();
    assert!(view.desktop.is_none());
    assert!(!view.draw_pointer);
    let (width, height) = (view.viewport.width(), view.viewport.height());
    let frame = Frame::new(view);
    let mut pixels = vec![0; (width * height * 3) as usize];
    let mut tile = [0; TILE_PIXELS];
    for y in (0..height).step_by(TILE_HEIGHT as usize) {
        for x in (0..width).step_by(TILE_WIDTH as usize) {
            let rect = frame.render_tile(x, y, &mut tile).unwrap();
            for row in 0..rect.height {
                for column in 0..rect.width {
                    let pixel = tile[(row * rect.width + column) as usize];
                    let offset = (((rect.y + row) * width + rect.x + column) * 3) as usize;
                    pixels[offset..offset + 3].copy_from_slice(&[
                        (pixel >> 16) as u8,
                        (pixel >> 8) as u8,
                        pixel as u8,
                    ]);
                }
            }
        }
    }
    if let Some(directory) = directory {
        let mut file =
            std::fs::File::create_new(directory.join(format!("{width}x{height}-{phase}.ppm")))
                .unwrap();
        write!(file, "P6\n{width} {height}\n255\n").unwrap();
        file.write_all(&pixels).unwrap();
        file.sync_all().unwrap();
    }
}
fn run(operation: impl FnOnce() + Send + 'static) {
    // The real bounded Engine/Volume host structures are deliberately large;
    // this host stack never changes the native user stack/arena quota.
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(operation)
        .unwrap()
        .join()
        .unwrap();
}
#[test]
fn render_native_window_material_walkthrough_oracle() {
    run(|| {
        let directory =
            std::env::var_os("OS_WINDOW_FIXTURE_DIRECTORY").map(std::path::PathBuf::from);
        for (width, height, scale) in PROFILES {
            let host = Host::new();
            let (mut draft, mut original, mut staging) = ([0; 4096], [0; 4096], [0; 4096]);
            let mut ui = controller(
                &host,
                EditMode::ReadWrite,
                Buffers {
                    draft: &mut draft,
                    original: &mut original,
                    staging: &mut staging,
                },
            );
            ui.resize(Viewport::new(width, height, scale, 1).unwrap())
                .unwrap();
            ui.set_compositor_cursor(true).unwrap();
            ui.refresh().unwrap();
            let mut sequence = 0;
            assert_eq!(ui.view().screen, Screen::Material);
            assert!(ui.view().objects.is_empty());
            snapshot(&ui, "initial", directory.as_deref());
            key(
                &mut ui,
                INPUT_EPOCH,
                &mut sequence,
                1,
                PhysicalKey::Enter,
                false,
            );
            assert_eq!(ui.view().screen, Screen::Editor);
            assert_eq!(ui.view().contents, Some(""));
            snapshot(&ui, "editor", directory.as_deref());
            text(&mut ui, &mut sequence);
            assert!(ui.document().is_dirty());
            snapshot(&ui, "dirty", directory.as_deref());
            key(
                &mut ui,
                INPUT_EPOCH,
                &mut sequence,
                1,
                PhysicalKey::F4,
                false,
            );
            assert!(!ui.view().editable);
            snapshot(&ui, "read-tool", directory.as_deref());
            for physical in [PhysicalKey::X, PhysicalKey::Delete, PhysicalKey::Backspace] {
                key(&mut ui, INPUT_EPOCH, &mut sequence, 1, physical, false);
            }
            assert_eq!(ui.view().contents, Some(TEXT));
            snapshot(&ui, "read-tool-blocked", directory.as_deref());
            key(
                &mut ui,
                INPUT_EPOCH,
                &mut sequence,
                1,
                PhysicalKey::F4,
                false,
            );
            assert!(ui.view().editable);
            snapshot(&ui, "write-tool-restored", directory.as_deref());
            modified_key(
                &mut ui,
                INPUT_EPOCH,
                &mut sequence,
                1,
                PhysicalKey::F10,
                KeyboardModifiers {
                    left_shift: true,
                    ..KeyboardModifiers::default()
                },
            );
            assert!(ui.view().document_menu.is_some());
            key(
                &mut ui,
                INPUT_EPOCH,
                &mut sequence,
                1,
                PhysicalKey::X,
                false,
            );
            assert_eq!(ui.view().contents, Some(TEXT));
            snapshot(&ui, "document-menu", directory.as_deref());
            key(
                &mut ui,
                INPUT_EPOCH,
                &mut sequence,
                1,
                PhysicalKey::Down,
                false,
            );
            snapshot(&ui, "document-menu-next", directory.as_deref());
            key(
                &mut ui,
                INPUT_EPOCH,
                &mut sequence,
                1,
                PhysicalKey::Enter,
                false,
            );
            assert!(ui.view().document_menu.is_none());
            assert!(!ui.view().editable);
            snapshot(&ui, "document-menu-read", directory.as_deref());
            for physical in [PhysicalKey::X, PhysicalKey::Delete, PhysicalKey::Backspace] {
                key(&mut ui, INPUT_EPOCH, &mut sequence, 1, physical, false);
            }
            assert_eq!(ui.view().contents, Some(TEXT));
            snapshot(&ui, "document-menu-blocked", directory.as_deref());
            modified_key(
                &mut ui,
                INPUT_EPOCH,
                &mut sequence,
                1,
                PhysicalKey::F10,
                KeyboardModifiers {
                    left_shift: true,
                    ..KeyboardModifiers::default()
                },
            );
            key(
                &mut ui,
                INPUT_EPOCH,
                &mut sequence,
                1,
                PhysicalKey::Escape,
                false,
            );
            assert!(ui.view().document_menu.is_none());
            assert!(!ui.view().editable);
            snapshot(&ui, "document-menu-cancelled", directory.as_deref());
            key(
                &mut ui,
                INPUT_EPOCH,
                &mut sequence,
                1,
                PhysicalKey::F4,
                false,
            );
            assert!(ui.view().editable);
            snapshot(&ui, "document-menu-write", directory.as_deref());
            key(&mut ui, INPUT_EPOCH, &mut sequence, 1, PhysicalKey::S, true);
            assert!(ui.document().is_saving());
            let mut completed = false;
            for _ in 0..8 {
                if ui.advance_save().unwrap() {
                    completed = true;
                    break;
                }
            }
            assert!(completed);
            assert_eq!(ui.view().status, Status::Saved { revision: 1 });
            assert!(!ui.document().is_dirty());
            assert_eq!(host.borrow_mut().bytes(0), TEXT.as_bytes());
            snapshot(&ui, "saved", directory.as_deref());
            key(
                &mut ui,
                INPUT_EPOCH,
                &mut sequence,
                1,
                PhysicalKey::Escape,
                false,
            );
            assert_eq!(ui.view().screen, Screen::Material);
            assert_eq!(ui.view().objects.len(), 1);
            snapshot(&ui, "list", directory.as_deref());
            for physical in [PhysicalKey::Tab, PhysicalKey::Tab, PhysicalKey::Enter] {
                key(&mut ui, INPUT_EPOCH, &mut sequence, 1, physical, false);
            }
            assert_eq!(ui.view().screen, Screen::Editor);
            assert_eq!(ui.view().contents, Some(TEXT));
            assert_eq!(ui.view().focus, Focus::Document);
            snapshot(&ui, "reopened", directory.as_deref());
            assert_eq!(host.borrow_mut().recover().entries[0].revision, 1);
            sequence += 1;
            ui.deliver(InputEnvelope {
                epoch: INPUT_EPOCH,
                sequence,
                generation: 1,
                event: Event::Cancel(CancelReason::FocusLost),
            })
            .unwrap();
            snapshot(&ui, "reopened-window-restored", directory.as_deref());
            let (x, y) = if scale == Scale::Percent100 {
                (834, 90)
            } else {
                (1028, 180)
            };
            for buttons in [1, 0] {
                sequence += 1;
                ui.deliver(InputEnvelope {
                    epoch: INPUT_EPOCH,
                    sequence,
                    generation: 2,
                    event: Event::Mouse {
                        x,
                        y,
                        input: MouseEvent {
                            dx: 0,
                            dy: 0,
                            buttons: MouseButtons::checked(buttons).unwrap(),
                            changed: MouseButtons::checked(1).unwrap(),
                            x_overflow: false,
                            y_overflow: false,
                        },
                    },
                })
                .unwrap();
            }
            assert!(ui.view().cursor.is_some());
            assert!(ui.view().document_menu.is_some());
            let tile_x = x / TILE_WIDTH * TILE_WIDTH;
            let tile_y = y / TILE_HEIGHT * TILE_HEIGHT;
            let mut hovered = [0; TILE_PIXELS];
            Frame::new(ui.view())
                .render_tile(tile_x, tile_y, &mut hovered)
                .unwrap();
            let status = ui.view().status;
            let focus = ui.view().focus;
            let cursor = ui.view().cursor;
            sequence += 1;
            assert_eq!(
                ui.deliver(InputEnvelope {
                    epoch: INPUT_EPOCH,
                    sequence,
                    generation: 1,
                    event: Event::PointerLeave,
                }),
                Err(nieuw_os_materiaal::controller::Error::StaleInput)
            );
            assert_eq!(ui.view().cursor, cursor);
            assert!(ui.view().document_menu.is_some());
            sequence += 1;
            ui.deliver(InputEnvelope {
                epoch: INPUT_EPOCH,
                sequence,
                generation: 2,
                event: Event::PointerLeave,
            })
            .unwrap();
            assert!(ui.view().document_menu.is_some());
            assert_eq!(ui.view().status, status);
            assert_eq!(ui.view().focus, focus);
            assert_eq!(ui.view().cursor, None);
            assert_eq!(
                ui.deliver(InputEnvelope {
                    epoch: INPUT_EPOCH,
                    sequence,
                    generation: 2,
                    event: Event::PointerLeave,
                }),
                Err(nieuw_os_materiaal::controller::Error::StaleInput)
            );
            let mut left = [0; TILE_PIXELS];
            Frame::new(ui.view())
                .render_tile(tile_x, tile_y, &mut left)
                .unwrap();
            assert_ne!(
                hovered, left,
                "Pointer-leave must change actual hover pixels"
            );
            snapshot(&ui, "document-menu-mouse", directory.as_deref());
            key(
                &mut ui,
                INPUT_EPOCH,
                &mut sequence,
                2,
                PhysicalKey::Escape,
                false,
            );
            assert!(ui.view().document_menu.is_none());
            assert_eq!(ui.view().contents, Some(TEXT));
            assert!(!ui.document().is_dirty());
            snapshot(&ui, "document-menu-mouse-cancelled", directory.as_deref());
        }
    });
}
#[test]
fn render_window_draft_focus_loss_and_input_rebind_oracle() {
    run(|| {
        let directory =
            std::env::var_os("OS_WINDOW_FIXTURE_DIRECTORY").map(std::path::PathBuf::from);
        for (width, height, scale) in PROFILES {
            let host = Host::new();
            let (mut draft, mut original, mut staging) = ([0; 4096], [0; 4096], [0; 4096]);
            let mut ui = controller(
                &host,
                EditMode::ReadWrite,
                Buffers {
                    draft: &mut draft,
                    original: &mut original,
                    staging: &mut staging,
                },
            );
            ui.resize(Viewport::new(width, height, scale, 1).unwrap())
                .unwrap();
            ui.set_compositor_cursor(true).unwrap();
            ui.refresh().unwrap();
            let mut sequence = 0;
            key(
                &mut ui,
                INPUT_EPOCH,
                &mut sequence,
                1,
                PhysicalKey::Enter,
                false,
            );
            text(&mut ui, &mut sequence);
            let caret = ui.view().caret;
            let anchor = ui.view().anchor;
            sequence += 1;
            ui.deliver(InputEnvelope {
                epoch: INPUT_EPOCH,
                sequence,
                generation: 1,
                event: Event::Cancel(CancelReason::FocusLost),
            })
            .unwrap();
            assert_eq!(ui.view().status, Status::InputLost);
            snapshot(&ui, "dirty-focus-lost", directory.as_deref());
            ui.rebind_input(INPUT_EPOCH + 1).unwrap();
            assert_eq!(ui.view().status, Status::InputLost);
            snapshot(&ui, "dirty-rebound", directory.as_deref());
            let mut sequence = 0;
            // Right at EOF restores validated input without changing the draft.
            key(
                &mut ui,
                INPUT_EPOCH + 1,
                &mut sequence,
                1,
                PhysicalKey::Right,
                false,
            );
            assert_eq!(ui.view().contents, Some(TEXT));
            assert_eq!((ui.view().caret, ui.view().anchor), (caret, anchor));
            assert!(ui.document().is_dirty());
            assert!(!ui.document().is_closed());
            assert_eq!(ui.view().status, Status::Dirty);
            snapshot(&ui, "dirty-recovered", directory.as_deref());
            assert!(host
                .borrow_mut()
                .recover()
                .entries
                .iter()
                .all(|entry| entry.revision == 0));
        }
    });
}

#[test]
fn render_window_clipboard_oracles() {
    run(|| {
        let directory =
            std::env::var_os("OS_WINDOW_FIXTURE_DIRECTORY").map(std::path::PathBuf::from);
        for (width, height, scale) in PROFILES {
            let host = Host::new();
            let (mut draft, mut original, mut staging) = ([0; 4096], [0; 4096], [0; 4096]);
            let mut ui = controller(
                &host,
                EditMode::ReadWrite,
                Buffers {
                    draft: &mut draft,
                    original: &mut original,
                    staging: &mut staging,
                },
            );
            ui.resize(Viewport::new(width, height, scale, 1).unwrap())
                .unwrap();
            ui.set_compositor_cursor(true).unwrap();
            ui.refresh().unwrap();
            ui.create().unwrap();
            let mut sequence = 0;
            text(&mut ui, &mut sequence);
            key(&mut ui, INPUT_EPOCH, &mut sequence, 1, PhysicalKey::A, true);
            assert_eq!(ui.clipboard_selection(), Some(TEXT));
            ui.clipboard_feedback(Status::ClipboardCopied).unwrap();
            snapshot(&ui, "clipboard-copied", directory.as_deref());
            key(
                &mut ui,
                INPUT_EPOCH,
                &mut sequence,
                1,
                PhysicalKey::Digit3,
                false,
            );
            key(&mut ui, INPUT_EPOCH, &mut sequence, 1, PhysicalKey::A, true);
            ui.clipboard_feedback(Status::ClipboardCopied).unwrap();
            snapshot(&ui, "clipboard-numeric", directory.as_deref());
            key(&mut ui, INPUT_EPOCH, &mut sequence, 1, PhysicalKey::V, true);
            ui.paste_clipboard("3+4").unwrap();
            assert_eq!(ui.view().contents, Some("3+4"));
            snapshot(&ui, "clipboard-back", directory.as_deref());
            ui.clipboard_feedback(Status::ClipboardEmpty).unwrap();
            snapshot(&ui, "clipboard-empty", directory.as_deref());
            key(&mut ui, INPUT_EPOCH, &mut sequence, 1, PhysicalKey::A, true);
            text(&mut ui, &mut sequence);
            ui.clipboard_feedback(Status::ClipboardEmpty).unwrap();
            snapshot(&ui, "clipboard-recovered-empty", directory.as_deref());
        }
    });
}

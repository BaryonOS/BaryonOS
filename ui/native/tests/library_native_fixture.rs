//! Source-bound host app-body oracles, never native or durable-disk evidence.
#[allow(dead_code)]
mod common;
use common::{controller, Host, Port, INPUT_EPOCH};
use nieuw_os_document_session::{volume::Buffers, EditMode};
use nieuw_os_input_core::Event;
use nieuw_os_materiaal::{
    controller::{Controller, InputEnvelope},
    render::Frame,
    Scale, Viewport, TILE_HEIGHT, TILE_PIXELS, TILE_WIDTH,
};
use std::{io::Write, path::Path};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

const ORIGINAL: &str = "een week aan de kust\n\nwe reizen met de trein en fietsen door de duinen\nhet huisje ligt vlak bij zee";
const EDITED: &str = "een week aan de kust\n\nwe vertrekken op zaterdag en nemen de fiets mee\nhet huisje ligt vlak bij zee";

fn key(
    ui: &mut Controller<'_, Port>,
    sequence: &mut u64,
    key: PhysicalKey,
    control: bool,
    shift: bool,
    alt: bool,
) {
    for action in [KeyAction::Press, KeyAction::Release] {
        *sequence += 1;
        ui.deliver(InputEnvelope {
            epoch: INPUT_EPOCH,
            sequence: *sequence,
            generation: 1,
            event: Event::Key(KeyboardEvent {
                key,
                action,
                modifiers: KeyboardModifiers {
                    left_control: control,
                    left_shift: shift,
                    left_alt: alt,
                    ..KeyboardModifiers::default()
                },
            }),
        })
        .unwrap();
    }
}
fn plain(ui: &mut Controller<'_, Port>, sequence: &mut u64, physical: PhysicalKey) {
    key(ui, sequence, physical, false, false, false);
}
fn ctrl(ui: &mut Controller<'_, Port>, sequence: &mut u64, physical: PhysicalKey) {
    key(ui, sequence, physical, true, false, false);
}
fn text(ui: &mut Controller<'_, Port>, sequence: &mut u64, value: &str) {
    for ch in value.chars() {
        let physical = match ch.to_ascii_lowercase() {
            'a' => PhysicalKey::A,
            'b' => PhysicalKey::B,
            'c' => PhysicalKey::C,
            'd' => PhysicalKey::D,
            'e' => PhysicalKey::E,
            'f' => PhysicalKey::F,
            'g' => PhysicalKey::G,
            'h' => PhysicalKey::H,
            'i' => PhysicalKey::I,
            'j' => PhysicalKey::J,
            'k' => PhysicalKey::K,
            'l' => PhysicalKey::L,
            'm' => PhysicalKey::M,
            'n' => PhysicalKey::N,
            'o' => PhysicalKey::O,
            'p' => PhysicalKey::P,
            'q' => PhysicalKey::Q,
            'r' => PhysicalKey::R,
            's' => PhysicalKey::S,
            't' => PhysicalKey::T,
            'u' => PhysicalKey::U,
            'v' => PhysicalKey::V,
            'w' => PhysicalKey::W,
            'x' => PhysicalKey::X,
            'y' => PhysicalKey::Y,
            'z' => PhysicalKey::Z,
            ' ' => PhysicalKey::Space,
            '\n' => PhysicalKey::Enter,
            _ => panic!("unsupported test input"),
        };
        key(
            ui,
            sequence,
            physical,
            false,
            ch.is_ascii_uppercase(),
            false,
        );
    }
}
fn save(ui: &mut Controller<'_, Port>, sequence: &mut u64) {
    ctrl(ui, sequence, PhysicalKey::S);
    assert!(ui.document().is_saving());
    let mut complete = false;
    for _ in 0..8 {
        if ui.advance_save().unwrap() {
            complete = true;
            break;
        }
    }
    assert!(complete);
    assert!(!ui.document().is_dirty());
}
fn snapshot(ui: &Controller<'_, Port>, name: &str, directory: Option<&Path>) {
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
            std::fs::File::create_new(directory.join(format!("{width}x{height}-{name}.ppm")))
                .unwrap();
        write!(file, "P6\n{width} {height}\n255\n").unwrap();
        file.write_all(&pixels).unwrap();
        file.sync_all().unwrap();
    }
}

#[test]
fn native_library_walkthrough_and_cold_controller_oracles() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let directory =
                std::env::var_os("OS_WINDOW_FIXTURE_DIRECTORY").map(std::path::PathBuf::from);
            for (width, height, scale) in [
                (880, 510, Scale::Percent100),
                (1120, 660, Scale::Percent200),
            ] {
                let host = Host::new();
                let (mut draft, mut original, mut staging) = ([0; 4096], [0; 4096], [0; 4096]);
                let viewport = Viewport::new(width, height, scale, 1).unwrap();
                let mut ui = controller(
                    &host,
                    EditMode::ReadWrite,
                    Buffers {
                        draft: &mut draft,
                        original: &mut original,
                        staging: &mut staging,
                    },
                );
                ui.resize(viewport).unwrap();
                ui.set_compositor_cursor(true).unwrap();
                ui.refresh().unwrap();
                let mut seq = 0;
                snapshot(&ui, "library-empty", directory.as_deref());
                plain(&mut ui, &mut seq, PhysicalKey::Enter);
                plain(&mut ui, &mut seq, PhysicalKey::F2);
                text(&mut ui, &mut seq, "Reisnotities");
                plain(&mut ui, &mut seq, PhysicalKey::Enter);
                text(&mut ui, &mut seq, ORIGINAL);
                assert_eq!(ui.view().title, b"Reisnotities");
                assert_eq!(ui.view().contents, Some(ORIGINAL));
                snapshot(&ui, "library-named-draft", directory.as_deref());
                save(&mut ui, &mut seq);
                snapshot(&ui, "library-saved", directory.as_deref());
                ctrl(&mut ui, &mut seq, PhysicalKey::G);
                text(&mut ui, &mut seq, "Reis");
                plain(&mut ui, &mut seq, PhysicalKey::Enter);
                snapshot(&ui, "library-organized", directory.as_deref());
                ctrl(&mut ui, &mut seq, PhysicalKey::A);
                text(&mut ui, &mut seq, EDITED);
                assert_eq!(ui.view().contents, Some(EDITED));
                snapshot(&ui, "library-edited", directory.as_deref());
                save(&mut ui, &mut seq);
                snapshot(&ui, "library-revision-saved", directory.as_deref());
                key(&mut ui, &mut seq, PhysicalKey::H, false, false, true);
                snapshot(&ui, "library-history", directory.as_deref());
                plain(&mut ui, &mut seq, PhysicalKey::Tab);
                plain(&mut ui, &mut seq, PhysicalKey::Enter);
                assert_eq!(ui.view().contents, Some(ORIGINAL));
                assert!(!ui.document().is_dirty());
                snapshot(&ui, "library-restored", directory.as_deref());
                plain(&mut ui, &mut seq, PhysicalKey::Escape);
                ctrl(&mut ui, &mut seq, PhysicalKey::F);
                text(&mut ui, &mut seq, "reis");
                snapshot(&ui, "library-search", directory.as_deref());
                plain(&mut ui, &mut seq, PhysicalKey::Enter);
                snapshot(&ui, "library-opened", directory.as_deref());
                assert_eq!(ui.view().contents, Some(ORIGINAL));
                assert_eq!(host.borrow_mut().bytes(0), ORIGINAL.as_bytes());
                drop(ui);
                // Fresh controller state over the SAME real service/table, not a
                // seeded UI list. Actual cold VM/disk proof is the runner's job.
                let mut ui = controller(
                    &host,
                    EditMode::ReadWrite,
                    Buffers {
                        draft: &mut draft,
                        original: &mut original,
                        staging: &mut staging,
                    },
                );
                ui.resize(viewport).unwrap();
                ui.set_compositor_cursor(true).unwrap();
                ui.refresh().unwrap();
                let mut seq = 0;
                snapshot(&ui, "library-cold-list", directory.as_deref());
                ctrl(&mut ui, &mut seq, PhysicalKey::F);
                text(&mut ui, &mut seq, "reis");
                snapshot(&ui, "library-cold-search", directory.as_deref());
                plain(&mut ui, &mut seq, PhysicalKey::Enter);
                assert_eq!(ui.view().contents, Some(ORIGINAL));
                snapshot(&ui, "library-cold-opened", directory.as_deref());
                key(&mut ui, &mut seq, PhysicalKey::H, false, false, true);
                snapshot(&ui, "library-cold-history", directory.as_deref());
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

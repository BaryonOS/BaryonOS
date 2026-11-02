//! Host-rendered oracle for the real QMP route. This is not native evidence.
#[allow(dead_code)]
mod common;

use common::{controller, Host, INPUT_EPOCH};
use nieuw_os_document_session::{volume::Buffers, EditMode};
use nieuw_os_input_core::Event;
use nieuw_os_materiaal::{
    controller::{Controller, InputEnvelope},
    desktop::Page,
    render::Frame,
    Scale, Viewport, TILE_HEIGHT, TILE_PIXELS, TILE_WIDTH,
};
use std::{io::Write, path::Path};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

fn key(ui: &mut Controller<'_, common::Port>, sequence: &mut u64, physical: PhysicalKey) {
    for action in [KeyAction::Press, KeyAction::Release] {
        *sequence += 1;
        ui.deliver(InputEnvelope {
            epoch: INPUT_EPOCH,
            sequence: *sequence,
            generation: 1,
            event: Event::Key(KeyboardEvent {
                key: physical,
                action,
                modifiers: KeyboardModifiers::default(),
            }),
        })
        .unwrap();
    }
}

fn snapshot(ui: &Controller<'_, common::Port>, name: &str, page: Page, output: Option<&Path>) {
    let view = ui.view();
    assert_eq!(view.desktop.unwrap().page(), page);
    assert_eq!(view.cursor, None);
    let frame = Frame::new(view);
    let width = view.viewport.width();
    let height = view.viewport.height();
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
    if let Some(directory) = output {
        let mut file = std::fs::File::create_new(directory.join(format!("{name}.ppm"))).unwrap();
        write!(file, "P6\n{width} {height}\n255\n").unwrap();
        file.write_all(&pixels).unwrap();
        file.sync_all().unwrap();
    }
}

#[test]
fn render_native_session_walkthrough_oracle() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let directory =
                std::env::var_os("OS_SESSION_FIXTURE_DIRECTORY").map(std::path::PathBuf::from);
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
            ui.resize(Viewport::new(768, 536, Scale::Percent100, 1).unwrap())
                .unwrap();
            ui.enable_vm_desktop().unwrap();
            ui.refresh().unwrap();
            let mut sequence = 0;
            snapshot(&ui, "accounts", Page::Accounts, directory.as_deref());
            key(&mut ui, &mut sequence, PhysicalKey::Tab);
            key(&mut ui, &mut sequence, PhysicalKey::Enter);
            for physical in [
                PhysicalKey::T,
                PhysicalKey::E,
                PhysicalKey::S,
                PhysicalKey::T,
            ] {
                key(&mut ui, &mut sequence, physical);
            }
            let desktop = ui.view().desktop.unwrap();
            assert!(!desktop.is_guest());
            assert_eq!(desktop.password_len(), 4);
            assert!(!desktop.unavailable());
            snapshot(&ui, "personal-login", Page::Login, directory.as_deref());
            key(&mut ui, &mut sequence, PhysicalKey::Enter);
            let desktop = ui.view().desktop.unwrap();
            assert!(!desktop.is_guest());
            assert_eq!(desktop.password_len(), 0);
            assert!(desktop.unavailable());
            snapshot(&ui, "denied", Page::Login, directory.as_deref());
            key(&mut ui, &mut sequence, PhysicalKey::Escape);
            snapshot(&ui, "accounts-return", Page::Accounts, directory.as_deref());
            key(&mut ui, &mut sequence, PhysicalKey::Enter);
            assert!(ui.view().desktop.unwrap().is_guest());
            snapshot(&ui, "guest-login", Page::Login, directory.as_deref());
            key(&mut ui, &mut sequence, PhysicalKey::Enter);
            snapshot(&ui, "desktop", Page::Home, directory.as_deref());
            key(&mut ui, &mut sequence, PhysicalKey::Tab);
            key(&mut ui, &mut sequence, PhysicalKey::Tab);
            key(&mut ui, &mut sequence, PhysicalKey::Enter);
            for physical in [PhysicalKey::M, PhysicalKey::A, PhysicalKey::T] {
                key(&mut ui, &mut sequence, physical);
            }
            assert_eq!(ui.view().desktop.unwrap().query(), "mat");
            snapshot(&ui, "search", Page::Search, directory.as_deref());
            key(&mut ui, &mut sequence, PhysicalKey::Enter);
            snapshot(&ui, "material", Page::Material, directory.as_deref());
            assert!(ui.view().objects.is_empty());
        })
        .unwrap()
        .join()
        .unwrap();
}

/// Renderer/layout checks at the intended desktop scales. Generated files are
/// host fixtures; only the root QEMU harness can establish native execution.
#[test]
fn render_desktop_foundation_at_reference_scales() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let directory =
                std::env::var_os("OS_FOUNDATION_FIXTURE_DIRECTORY").map(std::path::PathBuf::from);
            for (width, height, scale) in [
                (1280, 720, Scale::Percent100),
                (1920, 1080, Scale::Percent200),
            ] {
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
                ui.enable_vm_desktop().unwrap();
                ui.refresh().unwrap();
                let mut sequence = 0;
                snapshot(
                    &ui,
                    &format!("{width}-accounts"),
                    Page::Accounts,
                    directory.as_deref(),
                );
                key(&mut ui, &mut sequence, PhysicalKey::Enter);
                snapshot(
                    &ui,
                    &format!("{width}-login"),
                    Page::Login,
                    directory.as_deref(),
                );
                key(&mut ui, &mut sequence, PhysicalKey::Enter);
                snapshot(
                    &ui,
                    &format!("{width}-desktop"),
                    Page::Home,
                    directory.as_deref(),
                );
                key(&mut ui, &mut sequence, PhysicalKey::Enter);
                snapshot(
                    &ui,
                    &format!("{width}-material"),
                    Page::Material,
                    directory.as_deref(),
                );
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

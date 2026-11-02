//! Source-bound host compositions of the real shell, app and compositor code.
//! Ordered RAM storage and fixed display-only UTC are synthetic fixtures.
//! This does not establish native authentication, QEMU execution or hardware.
#[allow(dead_code)]
mod common;

use common::{controller, Host};
use nieuw_os_calculator::{
    model::{Command as CalcCommand, Model},
    render as calculator,
};
use nieuw_os_document_session::{volume::Buffers, EditMode};
use nieuw_os_graphics::{Canvas, Font, Rect, Size, Weight};
use nieuw_os_materiaal::{render::Frame, Scale, Viewport, TILE_HEIGHT, TILE_PIXELS, TILE_WIDTH};
use nieuw_os_window_session::{
    scene::{Scene, MAX_PIXELS},
    shell::{Page, Shell},
};
use std::{
    io::Write,
    path::{Path, PathBuf},
};
use ui_core::{KeyAction, KeyboardEvent, KeyboardModifiers, PhysicalKey};

// 2 November 2026, 14:32 UTC. Public display fixture, never auth freshness.
const FIXTURE_UTC: i64 = 1_793_629_920;
const DOCUMENT: &str = "Ruimte voor goed werk\n\nVandaag\nEen rustige werkplek helpt om aandacht bij de inhoud te houden.\nDit document blijft hetzelfde materiaal, ook als je een ander\ngereedschap kiest. Je werk heeft een herkenbare plek.\n\nOnze afspraken\n- Bewaar wijzigingen als een nieuwe revisie.\n- Geef alleen toegang tot het gekozen materiaal.\n- Houd zichtbaar waar de lokale kopie staat.\n- Een onderbreking mag geen werk laten verdwijnen.\n\nVolgende stap\nLees de tekst terug, maak een berekening en hervat daarna hier.\nDe geschiedenis blijft bij dit document.\n";
const PROFILES: [(u32, u32, u32); 5] = [
    (1280, 720, 1),
    (1920, 1080, 1),
    (1440, 900, 1),
    (1024, 768, 1),
    (1920, 1080, 2),
];

fn press(shell: &mut Shell, key: PhysicalKey, width: u32, height: u32, scale: u32) {
    for action in [KeyAction::Press, KeyAction::Release] {
        shell.key(
            KeyboardEvent {
                key,
                action,
                modifiers: KeyboardModifiers::default(),
            },
            width,
            height,
            scale,
        );
    }
}
fn guest_desktop(scene: &mut Scene) {
    let (w, h, s) = (scene.width, scene.height, scene.scale);
    scene.shell = Shell::new(true);
    scene.shell.set_clock(Some(FIXTURE_UTC));
    press(&mut scene.shell, PhysicalKey::Enter, w, h, s);
    assert_eq!(scene.shell.page(), Page::Users);
    press(&mut scene.shell, PhysicalKey::Enter, w, h, s);
    assert_eq!(scene.shell.page(), Page::Auth);
    press(&mut scene.shell, PhysicalKey::Enter, w, h, s);
    assert_eq!(scene.shell.page(), Page::Unlocking);
    assert!(scene.shell.finish_unlock());
    assert_eq!(scene.shell.page(), Page::Desktop);
}
fn transfer(scene: &Scene, index: usize, rect: Rect, pixels: &[u32], backing: &mut [u32]) {
    let mut bytes = [0u8; TILE_PIXELS * 4];
    let (x, y, width, height) = (rect.x as u32, rect.y as u32, rect.width, rect.height);
    let count = (width * height) as usize;
    for (color, out) in pixels[..count]
        .iter()
        .zip(bytes[..count * 4].chunks_exact_mut(4))
    {
        out.copy_from_slice(&color.to_le_bytes());
    }
    let mut rectangle = scene.windows[index].geometry.assigned;
    rectangle.x = x;
    rectangle.y = y;
    rectangle.width = width as u16;
    rectangle.height = height as u16;
    scene
        .copy_tile(index, rectangle, &bytes[..count * 4], backing)
        .unwrap();
}
fn app_backing(scene: &Scene) -> Vec<u32> {
    let host = Host::new();
    {
        let mut h = host.borrow_mut();
        h.seed_named(0, b"Werknotities", DOCUMENT.as_bytes());
        h.seed_named(
            1,
            b"Ideeen voor morgen",
            b"Een tweede synthetisch document.",
        );
        h.seed_named(2, b"Reisplan", b"Een lokaal bewaard reisplan.");
    }
    let mut backing = vec![0; MAX_PIXELS];
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
    let material = scene.windows[0].body;
    ui.resize(
        Viewport::new(
            material.width,
            material.height,
            if scene.scale == 2 {
                Scale::Percent200
            } else {
                Scale::Percent100
            },
            1,
        )
        .unwrap(),
    )
    .unwrap();
    ui.set_compositor_cursor(true).unwrap();
    ui.refresh().unwrap();
    assert_eq!(ui.view().objects.len(), 3);
    ui.open(0).unwrap();
    assert_eq!(ui.view().contents, Some(DOCUMENT));
    let frame = Frame::new(ui.view());
    let mut tile = [0u32; TILE_PIXELS];
    for y in (0..material.height).step_by(TILE_HEIGHT as usize) {
        for x in (0..material.width).step_by(TILE_WIDTH as usize) {
            let rect = frame.render_tile(x, y, &mut tile).unwrap();
            transfer(
                scene,
                0,
                Rect::new(rect.x as i32, rect.y as i32, rect.width, rect.height),
                &tile,
                &mut backing,
            );
        }
    }
    let calculator_body = scene.windows[1].body;
    let view =
        calculator::Viewport::new(calculator_body.width, calculator_body.height, scene.scale)
            .unwrap();
    let mut model = Model::new();
    for byte in b"1250*0,21" {
        model.command(CalcCommand::Insert(*byte));
    }
    model.command(CalcCommand::Evaluate);
    assert!(model.answer().is_some());
    model.cancel_focus();
    for y in (0..calculator_body.height).step_by(TILE_HEIGHT as usize) {
        for x in (0..calculator_body.width).step_by(TILE_WIDTH as usize) {
            let rect = calculator::Rect {
                x,
                y,
                width: TILE_WIDTH.min(calculator_body.width - x),
                height: TILE_HEIGHT.min(calculator_body.height - y),
            };
            calculator::render_tile(&model, view, rect, &mut tile).unwrap();
            transfer(
                scene,
                1,
                Rect::new(x as i32, y as i32, rect.width, rect.height),
                &tile,
                &mut backing,
            );
        }
    }
    backing
}
fn write_ppm(path: &Path, width: u32, height: u32, pixels: &[u32]) {
    assert_eq!(pixels.len(), (width * height) as usize);
    let mut file = std::fs::File::create_new(path).unwrap();
    write!(file, "P6\n{width} {height}\n255\n").unwrap();
    for row in pixels.chunks(width as usize) {
        let mut rgb = Vec::with_capacity(width as usize * 3);
        for pixel in row {
            rgb.extend_from_slice(&[(pixel >> 16) as u8, (pixel >> 8) as u8, *pixel as u8]);
        }
        file.write_all(&rgb).unwrap();
    }
    file.sync_all().unwrap();
}
fn render(scene: &Scene, backing: &[u32]) -> Vec<u32> {
    let mut pixels = vec![0; (scene.width * scene.height) as usize];
    scene
        .render(
            Rect::new(0, 0, scene.width, scene.height),
            &mut pixels,
            backing,
        )
        .unwrap();
    pixels
}
fn snapshot(scene: &Scene, backing: &[u32], label: &str, directory: Option<&Path>) -> Vec<u32> {
    let pixels = render(scene, backing);
    if let Some(directory) = directory {
        write_ppm(
            &directory.join(format!(
                "{}x{}-s{}-{label}.ppm",
                scene.width, scene.height, scene.scale
            )),
            scene.width,
            scene.height,
            &pixels,
        );
    }
    for control in scene
        .shell
        .controls(scene.width, scene.height, scene.scale)
        .iter()
        .flatten()
    {
        let b = control.bounds;
        assert!(
            b.x >= 0 && b.y >= 44 * scene.scale as i32,
            "{label}: control above safe content"
        );
        assert!(
            i64::from(b.x) + i64::from(b.width) <= i64::from(scene.width),
            "{label}: horizontal control clipping"
        );
        assert!(
            i64::from(b.y) + i64::from(b.height) <= i64::from(scene.height - 44 * scene.scale),
            "{label}: control/footer overlap"
        );
    }
    pixels
}
fn board(images: &[(String, Vec<u32>)], directory: &Path) {
    let source = std::process::Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .unwrap_or_else(|| "unknown".to_owned());
    let source = format!(
        "Hostbron: {} · fixture gebruikt dezelfde Rust-renderers",
        source.trim()
    );
    let (width, height) = (1352, 1800);
    let mut pixels = vec![0xf2f1ec; (width * height) as usize];
    for (index, (_, image)) in images.iter().enumerate() {
        let (x, y) = (
            24 + (index as u32 % 2) * 664,
            104 + (index as u32 / 2) * 416,
        );
        for dy in 0..360u32 {
            for dx in 0..640u32 {
                pixels[((y + dy) * width + x + dx) as usize] =
                    image[(dy * 2 * 1280 + dx * 2) as usize];
            }
        }
    }
    {
        let mut c = Canvas::new(Rect::new(0, 0, width, height), &mut pixels).unwrap();
        let title = Font::new(Size::Title, Weight::Semibold, false);
        let body = Font::new(Size::Body, Weight::Regular, false);
        c.text(24, 20, "Nieuw OS · sessieroute", title, 0x20262d);
        c.text(
            24,
            58,
            "Hostrender · echte Rust-renderers · synthetische gegevens · geen accountauthenticatie",
            body,
            0x5c6570,
        );
        for (index, (name, _)) in images.iter().enumerate() {
            let (x, y) = (
                24 + (index as i32 % 2) * 664,
                104 + (index as i32 / 2) * 416,
            );
            c.text(x, y + 370, name, body, 0x20262d);
        }
        c.text(24, 1770, &source, body, 0x5c6570);
    }
    write_ppm(
        &directory.join("session-board-host.ppm"),
        width,
        height,
        &pixels,
    );
}
#[test]
fn render_eight_session_states_at_reference_resolutions() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let directory = std::env::var_os("OS_SESSION_BOARD_DIRECTORY").map(PathBuf::from);
            if let Some(path) = &directory {
                assert!(path.is_dir());
            }
            for (width, height, scale) in PROFILES {
                let mut scene = Scene::new(width, height, scale, [1, 2]).unwrap();
                let backing = app_backing(&scene);
                let mut images = Vec::new();
                scene.shell = Shell::new(true);
                scene.shell.set_clock(Some(FIXTURE_UTC));
                images.push((
                    "1 · Vergrendelscherm".to_owned(),
                    snapshot(&scene, &backing, "lock", directory.as_deref()),
                ));
                press(&mut scene.shell, PhysicalKey::Enter, width, height, scale);
                assert_eq!(scene.shell.page(), Page::Users);
                images.push((
                    "2 · Gebruiker kiezen".to_owned(),
                    snapshot(&scene, &backing, "users", directory.as_deref()),
                ));
                press(&mut scene.shell, PhysicalKey::Tab, width, height, scale);
                press(&mut scene.shell, PhysicalKey::Enter, width, height, scale);
                assert_eq!(scene.shell.page(), Page::Auth);
                images.push((
                    "3 · Authenticatiemethoden (onbeschikbaar)".to_owned(),
                    snapshot(&scene, &backing, "auth", directory.as_deref()),
                ));
                guest_desktop(&mut scene);
                scene.shell.hidden = [true, true];
                images.push((
                    "4 · Lege werkruimte".to_owned(),
                    snapshot(&scene, &backing, "empty-desktop", directory.as_deref()),
                ));
                scene.shell.hidden = [false, false];
                images.push((
                    "5 · Materiaal en Rekenmachine".to_owned(),
                    snapshot(&scene, &backing, "two-windows", directory.as_deref()),
                ));
                press(&mut scene.shell, PhysicalKey::F3, width, height, scale);
                assert_eq!(scene.shell.page(), Page::Search);
                images.push((
                    "6 · Zoeken en starten".to_owned(),
                    snapshot(&scene, &backing, "search", directory.as_deref()),
                ));
                scene.shell.open(Page::Settings);
                images.push((
                    "7 · Snelle instellingen".to_owned(),
                    snapshot(&scene, &backing, "settings", directory.as_deref()),
                ));
                scene.shell.open(Page::Capability);
                images.push((
                    "8 · Systeemaanvraag (geen grant)".to_owned(),
                    snapshot(&scene, &backing, "capability", directory.as_deref()),
                ));
                if (width, height, scale) == PROFILES[0] {
                    if let Some(path) = &directory {
                        board(&images, path);
                    }
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

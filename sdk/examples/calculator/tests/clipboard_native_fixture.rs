//! Exact calculator-body reference images, using the real model and renderer.
//! This generator is host-only and performs no native clipboard exchange.
use nieuw_os_calculator::{
    model::{Command, Model, Notice},
    render::{render_tile, Rect, Viewport},
};
use std::{io::Write, path::Path};
fn snapshot(model: &Model, view: Viewport, phase: &str, directory: Option<&Path>) {
    let (width, height) = (view.width(), view.height());
    let mut pixels = vec![0u8; (width * height * 3) as usize];
    for y in (0..height).step_by(8) {
        for x in (0..width).step_by(32) {
            let rect = Rect {
                x,
                y,
                width: 32.min(width - x),
                height: 8.min(height - y),
            };
            let mut tile = [0; 256];
            render_tile(model, view, rect, &mut tile).unwrap();
            for row in 0..rect.height {
                for col in 0..rect.width {
                    let p = tile[(row * rect.width + col) as usize];
                    let at = (((y + row) * width + x + col) * 3) as usize;
                    pixels[at..at + 3].copy_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, p as u8]);
                }
            }
        }
    }
    if let Some(directory) = directory {
        let mut file = std::fs::File::create_new(
            directory.join(format!("calculator-{width}x{height}-{phase}.ppm")),
        )
        .unwrap();
        write!(file, "P6\n{width} {height}\n255\n").unwrap();
        file.write_all(&pixels).unwrap();
        file.sync_all().unwrap();
    }
}
#[test]
fn render_native_clipboard_calculator_oracles() {
    let directory = std::env::var_os("OS_WINDOW_FIXTURE_DIRECTORY").map(std::path::PathBuf::from);
    for (width, height, scale) in [(320, 484, 1), (640, 608, 2)] {
        let view = Viewport::new(width, height, scale).unwrap();
        let mut model = Model::new();
        assert!(model.paste_clipboard("nieuw os werkt").is_err());
        assert_eq!(model.expression(), "");
        snapshot(&model, view, "clipboard-unsupported", directory.as_deref());
        model.paste_clipboard("3").unwrap();
        snapshot(&model, view, "clipboard-pasted", directory.as_deref());
        for b in b"+4" {
            model.command(Command::Insert(*b));
        }
        model.command(Command::SelectAll);
        model.clipboard_notice(Notice::ClipboardCopied);
        assert_eq!(model.clipboard_selection(), "3+4");
        snapshot(
            &model,
            view,
            "clipboard-calculator-copied",
            directory.as_deref(),
        );
    }
}

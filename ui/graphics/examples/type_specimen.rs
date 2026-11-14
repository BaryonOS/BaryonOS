//! Host-only visual check: draws fixed specimen lines with every `Font` record
//! through the real `Canvas::text` path into binary PPM files. Run explicitly:
//! `cargo run --example type_specimen [--features inter-atlas] -- OUT_DIR`.
//! Synthetic host images, not native or QEMU evidence.
use nieuw_os_graphics::{Canvas, Font, Rect, Size, Weight};
use std::io::Write;

const LINES: [&str; 5] = [
    "Bestanden · Thuis-NAS verbonden",
    "Versie 0.9.3 — 13 nov. 2026 09:41",
    "Dit apparaat 1.234,56 MB",
    "AaBbIiLl1|0Oo àéïõü €",
    "Instellingen",
];
// (background, text) light then dark, 0xRRGGBB.
const THEMES: [(u32, u32); 2] = [(0xffffff, 0x141a23), (0x0b0e14, 0xe8ecf2)];

fn main() -> std::io::Result<()> {
    let out = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("usage: type_specimen OUT_DIR"),
    );
    let atlas = if cfg!(feature = "inter-atlas") {
        "inter"
    } else {
        "open-sans"
    };
    let mut fonts = Vec::new();
    for double in [false, true] {
        let scale = if double { "200" } else { "100" };
        for (size, weight, name) in [
            (Size::Small, Weight::Regular, "small"),
            (Size::Body, Weight::Regular, "body-regular"),
            (Size::Body, Weight::Semibold, "body-semibold"),
            (Size::Title, Weight::Semibold, "title"),
        ] {
            fonts.push((format!("{name}-{scale}"), Font::new(size, weight, double)));
        }
        fonts.push((format!("display-{scale}"), Font::display(double)));
    }
    for (name, font) in fonts {
        let pad = font.em() as i32 / 2 + 8;
        let column = LINES.iter().map(|l| font.measure(l)).max().unwrap() + 2 * pad as u32;
        let height = font.height() * LINES.len() as u32 + 2 * pad as u32;
        let width = column * THEMES.len() as u32;
        let mut pixels = vec![0u32; (width * height) as usize];
        let mut canvas = Canvas::new(Rect::new(0, 0, width, height), &mut pixels).unwrap();
        for (i, (background, ink)) in THEMES.into_iter().enumerate() {
            let x = (column * i as u32) as i32;
            canvas.fill_rect(Rect::new(x, 0, column, height), background);
            for (row, line) in LINES.iter().enumerate() {
                let y = pad + (font.height() * row as u32) as i32;
                canvas.text(x + pad, y, line, font, ink);
            }
        }
        let mut file = std::fs::File::create(out.join(format!("{atlas}-{name}.ppm")))?;
        write!(file, "P6\n{width} {height}\n255\n")?;
        let bytes: Vec<u8> = pixels
            .iter()
            .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
            .collect();
        file.write_all(&bytes)?;
    }
    Ok(())
}

//! Host-only reference/preview driver. Never linked into calculator-app.
use nieuw_os_calculator::{
    decimal,
    model::{Command, Model},
    render::{render_tile, Rect, Viewport},
};
use std::{
    io::{self, BufRead, Write},
    path::Path,
};
fn preview(path: &Path, scale: u32) {
    let view = Viewport::new(768, 536, scale).expect("supported preview viewport");
    let mut model = Model::new();
    for byte in b"12,5 * (8 - 3)" {
        model.command(Command::Insert(*byte));
    }
    model.command(Command::Evaluate);
    let mut pixels = vec![0u32; (view.width() * view.height()) as usize];
    for y in (0..view.height()).step_by(8) {
        for x in (0..view.width()).step_by(32) {
            let rect = Rect {
                x,
                y,
                width: 32.min(view.width() - x),
                height: 8.min(view.height() - y),
            };
            let mut tile = [0; 256];
            render_tile(&model, view, rect, &mut tile).unwrap();
            for (dy, row) in tile[..(rect.width * rect.height) as usize]
                .chunks_exact(rect.width as usize)
                .enumerate()
            {
                let at = ((y + dy as u32) * view.width() + x) as usize;
                pixels[at..at + row.len()].copy_from_slice(row);
            }
        }
    }
    let mut output = std::fs::File::create(path).unwrap();
    write!(output, "P6\n{} {}\n255\n", view.width(), view.height()).unwrap();
    for p in pixels {
        output
            .write_all(&[(p >> 16) as u8, (p >> 8) as u8, p as u8])
            .unwrap();
    }
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 4 && args[1] == "--preview" {
        preview(Path::new(&args[2]), args[3].parse().unwrap());
        return;
    }
    assert_eq!(
        args.len(),
        1,
        "host driver expects stdin or --preview FILE SCALE"
    );
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        match decimal::evaluate(&line) {
            Ok(result) => println!("{} {}", result.value.raw(), u8::from(result.rounded)),
            Err(error) => println!("ERR {:?} {}", error.kind, error.position),
        }
    }
}

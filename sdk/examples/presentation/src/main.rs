//! Host-only preview. Uses the same native raster library, not a web view.
use nieuw_os_presentation::{Deck, HEIGHT, MAX_INPUT, WIDTH};
use std::io::{self, Read, Write};

fn run() -> Result<(), String> {
    let usage = "Gebruik: slide-preview --slide NUMMER [--scale 1|2] < deck.txt > slide.ppm";
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--slide")) {
        return Err(usage.into());
    }
    let selected = args
        .next()
        .and_then(|a| a.into_string().ok())
        .and_then(|a| a.parse::<usize>().ok())
        .filter(|n| *n != 0)
        .ok_or(usage)?;
    let scale = match args.next() {
        None => 1,
        Some(flag) if flag == "--scale" => match args.next().as_deref() {
            Some(value) if value == "1" => 1,
            Some(value) if value == "2" => 2,
            _ => return Err(usage.into()),
        },
        _ => return Err(usage.into()),
    };
    if args.next().is_some() {
        return Err(usage.into());
    }
    let mut input = Vec::new();
    input
        .try_reserve_exact(MAX_INPUT + 1)
        .map_err(|_| "Onvoldoende geheugen.")?;
    io::stdin()
        .lock()
        .take((MAX_INPUT + 1) as u64)
        .read_to_end(&mut input)
        .map_err(|e| format!("Deck lezen mislukt: {e}"))?;
    if input.len() > MAX_INPUT {
        return Err("Deck is te groot.".into());
    }
    let source = std::str::from_utf8(&input).map_err(|_| "Deck is geen UTF-8.")?;
    let mut deck = Deck::parse(source).map_err(|e| format!("Deck ongeldig: {e:?}"))?;
    deck.select(selected - 1)
        .map_err(|e| format!("Dia selecteren mislukt: {e:?}"))?;
    let count = WIDTH * HEIGHT * scale * scale;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(count)
        .map_err(|_| "Onvoldoende geheugen.")?;
    pixels.resize(count, 0);
    deck.render(&mut pixels, scale)
        .map_err(|e| format!("Dia renderen mislukt: {e:?}"))?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(count * 3 + 32)
        .map_err(|_| "Onvoldoende geheugen.")?;
    write!(output, "P6\n{} {}\n255\n", WIDTH * scale, HEIGHT * scale)
        .map_err(|e| format!("PPM-header mislukt: {e}"))?;
    for pixel in pixels {
        output.extend_from_slice(&[(pixel >> 16) as u8, (pixel >> 8) as u8, pixel as u8]);
    }
    io::stdout()
        .lock()
        .write_all(&output)
        .map_err(|e| format!("Voorbeeld schrijven mislukt: {e}"))
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

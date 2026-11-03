//! Host-only conversion consumer. Not a native Nieuw OS executable.
use nieuw_os_image_export::{convert, MAX_INPUT_BYTES, MAX_OUTPUT_BYTES};
use std::io::{self, Read, Write};

fn run() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);
    let usage = "Gebruik: image-bmp --background RRGGBB < invoer.png > uitvoer.bmp";
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--background")) {
        return Err(usage.into());
    }
    let background = args
        .next()
        .and_then(|arg| arg.into_string().ok())
        .filter(|value| value.len() == 6 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .and_then(|value| u32::from_str_radix(&value, 16).ok())
        .ok_or(usage)?;
    if args.next().is_some() {
        return Err(usage.into());
    }
    let mut input = Vec::new();
    input
        .try_reserve_exact(MAX_INPUT_BYTES + 1)
        .map_err(|_| "Onvoldoende geheugen voor invoer.")?;
    io::stdin()
        .lock()
        .take((MAX_INPUT_BYTES + 1) as u64)
        .read_to_end(&mut input)
        .map_err(|error| format!("Afbeelding lezen mislukt: {error}"))?;
    let output = convert(&input, background, MAX_OUTPUT_BYTES)
        .map_err(|error| format!("Afbeelding exporteren mislukt: {error:?}"))?;
    io::stdout()
        .lock()
        .write_all(&output)
        .map_err(|error| format!("BMP schrijven mislukt: {error}"))
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

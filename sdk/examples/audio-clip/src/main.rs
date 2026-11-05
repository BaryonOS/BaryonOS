//! Host-only PCM file consumer. Native playback remains a separate audio chain.
use nieuw_os_audio_clip::{Wave, MAX_INPUT_BYTES};
use std::io::{self, Read, Write};

fn number(value: Option<std::ffi::OsString>) -> Option<usize> {
    let text = value?.into_string().ok()?;
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

fn run() -> Result<(), String> {
    let usage =
        "Gebruik: wav-clip --start-frame N --frames N [--drop-metadata] < bron.wav > fragment.wav";
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--start-frame")) {
        return Err(usage.into());
    }
    let start = number(args.next()).ok_or(usage)?;
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--frames")) {
        return Err(usage.into());
    }
    let frames = number(args.next()).filter(|n| *n != 0).ok_or(usage)?;
    let end = start
        .checked_add(frames)
        .ok_or("Framebereik buiten bereik.")?;
    let drop_metadata = match args.next() {
        None => false,
        Some(flag) if flag == "--drop-metadata" => true,
        _ => return Err(usage.into()),
    };
    if args.next().is_some() {
        return Err(usage.into());
    }
    let mut input = Vec::new();
    input
        .try_reserve_exact(MAX_INPUT_BYTES + 1)
        .map_err(|_| "Onvoldoende geheugen.")?;
    io::stdin()
        .lock()
        .take((MAX_INPUT_BYTES + 1) as u64)
        .read_to_end(&mut input)
        .map_err(|error| format!("WAV lezen mislukt: {error}"))?;
    let wave = Wave::parse(&input).map_err(|error| format!("WAV ongeldig: {error:?}"))?;
    let output = wave
        .clip(start..end, drop_metadata, MAX_INPUT_BYTES)
        .map_err(|error| format!("Fragment maken mislukt: {error:?}"))?;
    let mut out = io::stdout().lock();
    // Explicit flush: the exit-time flush of the line buffer ignores errors.
    out.write_all(&output)
        .and_then(|()| out.flush())
        .map_err(|error| format!("Fragment schrijven mislukt: {error}"))
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

//! Host-only conversion consumer, not native save authorization.
use nieuw_os_document_export::{export, MAX_INPUT_BYTES, MAX_OUTPUT_BYTES};
use std::io::{self, Read, Write};

fn run() -> Result<(), String> {
    if std::env::args_os().len() != 1 {
        return Err("Gebruik: text-docx < tekst.txt > document.docx".into());
    }
    let mut input = Vec::new();
    input
        .try_reserve_exact(MAX_INPUT_BYTES + 1)
        .map_err(|_| "Onvoldoende geheugen.")?;
    io::stdin()
        .lock()
        .take((MAX_INPUT_BYTES + 1) as u64)
        .read_to_end(&mut input)
        .map_err(|error| format!("Tekst lezen mislukt: {error}"))?;
    let document = export(&input, MAX_OUTPUT_BYTES)
        .map_err(|error| format!("DOCX exporteren mislukt: {error:?}"))?;
    io::stdout()
        .lock()
        .write_all(&document)
        .map_err(|error| format!("DOCX schrijven mislukt: {error}"))
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

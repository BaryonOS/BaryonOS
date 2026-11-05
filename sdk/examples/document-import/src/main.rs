use nieuw_os_document_import::{import_docx, MAX_PACKAGE_BYTES};
use std::io::{self, Read, Write};

fn main() {
    if let Err(message) = run() {
        eprintln!("docx-text: {message}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 1 || args[0] != "--plain-text" {
        return Err("gebruik: docx-text --plain-text < bron.docx > tekst.txt; verliest opmaak, importeert geen tabellen/velden/afbeeldingen/header- of footertekst".into());
    }
    let mut input = Vec::new();
    input
        .try_reserve_exact(MAX_PACKAGE_BYTES + 1)
        .map_err(|e| format!("inputgeheugen: {e}"))?;
    io::stdin()
        .lock()
        .take((MAX_PACKAGE_BYTES + 1) as u64)
        .read_to_end(&mut input)
        .map_err(|e| format!("lezen: {e}"))?;
    let text = import_docx(&input).map_err(|e| format!("import geweigerd: {e:?}"))?;
    let mut out = io::stdout().lock();
    // Explicit flush: the exit-time flush of the line buffer ignores errors.
    out.write_all(text.as_bytes())
        .and_then(|()| out.flush())
        .map_err(|e| format!("schrijven: {e}"))
}

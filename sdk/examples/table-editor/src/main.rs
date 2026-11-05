//! Host-only file interoperability consumer, not a native Nieuw OS executable.
use std::io::{self, Read, Write};

use nieuw_os_table_editor::{Limits, Table};

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|value| {
            value
                .into_string()
                .map_err(|_| "Argument is geen geldige UTF-8-tekst.".to_string())
        })
        .collect::<Result<_, _>>()?;
    if !args.len().is_multiple_of(4) || args.chunks(4).any(|edit| edit[0] != "--set") {
        return Err(
            "Gebruik: table-csv [--set RIJ KOLOM TEKST]...; indices beginnen bij 1.".into(),
        );
    }
    let limits = Limits::default();
    let mut input = Vec::new();
    io::stdin()
        .lock()
        .take((limits.input_bytes + 1) as u64)
        .read_to_end(&mut input)
        .map_err(|error| format!("CSV lezen mislukt: {error}"))?;
    let mut table =
        Table::import(&input, limits).map_err(|error| format!("CSV import mislukt: {error:?}"))?;
    for edit in args.chunks(4) {
        let index = |value: &str| {
            value
                .parse::<usize>()
                .ok()
                .and_then(|index| index.checked_sub(1))
                .ok_or_else(|| "Rij en kolom moeten positieve gehele getallen zijn.".to_string())
        };
        table
            .set_cell(index(&edit[1])?, index(&edit[2])?, &edit[3])
            .map_err(|error| format!("Cel bewerken mislukt: {error:?}"))?;
    }
    let output = table
        .export()
        .map_err(|error| format!("CSV export mislukt: {error:?}"))?;
    io::stdout()
        .lock()
        .write_all(&output)
        .map_err(|error| format!("CSV schrijven mislukt: {error}"))
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

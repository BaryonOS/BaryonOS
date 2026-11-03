//! Host-only numeric spreadsheet consumer, not a native app executable.
use nieuw_os_spreadsheet::{Address, Sheet};
use std::io::{self, Write};

fn run() -> Result<(), String> {
    let usage = "Gebruik: sheet-calc --set A1 2 --set B1 '=A1*3' --get B1 [--set ... --get ...]";
    let mut args = std::env::args_os().skip(1);
    let mut sheet = Sheet::default();
    let mut output = String::new();
    let mut actions = 0;
    while let Some(action) = args.next() {
        actions += 1;
        if actions > 256 {
            return Err("Maximaal 256 acties.".into());
        }
        let address_text = args
            .next()
            .and_then(|a| a.into_string().ok())
            .ok_or(usage)?;
        let address = Address::parse(&address_text).map_err(|e| format!("{e:?}"))?;
        if action == "--set" {
            let input = args
                .next()
                .and_then(|a| a.into_string().ok())
                .ok_or(usage)?;
            sheet
                .set(address, &input)
                .map_err(|e| format!("Cel instellen mislukt: {e:?}"))?;
        } else if action == "--get" {
            let value = sheet
                .value(address)
                .map_err(|e| format!("Herberekenen mislukt: {e:?}"))?;
            let text = value.value.format();
            output
                .try_reserve(text.as_str().len() + 16)
                .map_err(|_| "Onvoldoende geheugen.")?;
            output.push_str(text.as_str());
            if value.rounded {
                output.push_str(" [afgerond]");
            }
            output.push('\n');
        } else {
            return Err(usage.into());
        }
    }
    if output.is_empty() {
        return Err(usage.into());
    }
    io::stdout()
        .lock()
        .write_all(output.as_bytes())
        .map_err(|e| format!("Resultaat schrijven mislukt: {e}"))
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

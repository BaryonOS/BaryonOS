#![cfg(feature = "host-cli")]
use std::process::Command;

#[test]
fn real_process_reads_recalculates_and_exposes_rounding() {
    let output = Command::new(env!("CARGO_BIN_EXE_sheet-calc"))
        .args([
            "--set", "A1", "2", "--set", "B1", "=A1*3", "--get", "B1", "--set", "A1", "4", "--get",
            "B1", "--set", "A1", "=1/3", "--get", "B1",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output.stderr);
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, b"6\n12\n0,999999999 [afgerond]\n");
}

#[test]
fn usage_and_cell_errors_have_no_success_shaped_output() {
    for args in [
        vec![],
        vec!["--get", "A1"],
        vec!["--set", "A1"],
        vec!["--set", "a1", "2"],
        vec!["--bad", "A1"],
        vec!["--set", "A1", "2"],
        vec!["--set", "A1", "=A1", "--get", "A1"],
        vec![
            "--set", "A1", "2", "--get", "A1", "--set", "B1", "=1/0", "--get", "B1",
        ],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_sheet-calc"))
            .args(&args)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn error_messages_show_a1_addresses_without_internal_debug_coordinates() {
    for (args, expected) in [
        (vec!["--set", "A1", "=A1+1", "--get", "A1"], "Cel A1: circulaire celverwijzing"),
        (vec!["--get", "B1"], "Cel B1: verwezen cel ontbreekt"),
        (vec!["--get", "a1"], "Ongeldig adres 'a1'"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_sheet-calc"))
            .args(&args)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains(expected), "got: {error}");
        assert!(!error.contains("Address {"), "got: {error}");
    }
}

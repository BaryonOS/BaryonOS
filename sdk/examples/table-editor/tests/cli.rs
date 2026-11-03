#![cfg(feature = "host-cli")]

use std::{
    io::Write,
    process::{Command, Stdio},
};

fn run(input: &[u8], args: &[&str]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_table-csv"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let write = child.stdin.take().unwrap().write_all(input);
    let output = child.wait_with_output().unwrap();
    if let Err(error) = write {
        assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
        assert!(!output.status.success(), "stdin was not fully consumed");
    }
    output
}

#[test]
fn real_process_import_edit_export_reopen() {
    let result = run(
        "Naam,Waarde\nZoë,1\n".as_bytes(),
        &["--set", "2", "2", "€ 42,50"],
    );
    assert!(result.status.success(), "{:?}", result.stderr);
    assert!(result.stderr.is_empty());
    assert_eq!(
        result.stdout,
        "Naam,Waarde\r\nZoë,\"€ 42,50\"\r\n".as_bytes()
    );
    let reopened = run(&result.stdout, &[]);
    assert!(reopened.status.success());
    assert_eq!(reopened.stdout, result.stdout);
}

#[test]
fn malformed_csv_arguments_and_late_failed_edits_produce_no_success_output() {
    for (input, args) in [
        (b"a,b\nc".as_slice(), vec![]),
        (b"a,b", vec!["--set", "0", "1", "x"]),
        (b"a,b", vec!["--set", "2", "1", "x"]),
        (b"a,b", vec!["--set", "1", "1", "x", "--set", "1", "3", "y"]),
        (b"a,b", vec!["--set", "1"]),
        (b"a,b", vec!["--unknown", "1", "1", "x"]),
        (b"a,b", vec!["--set", "184467440737095516160", "1", "x"]),
    ] {
        let result = run(input, &args);
        assert!(!result.status.success(), "{args:?}");
        assert!(result.stdout.is_empty(), "{args:?}");
        assert!(!result.stderr.is_empty(), "{args:?}");
    }
}

#[test]
fn rejected_arguments_close_stdin_without_consuming_a_large_input() {
    let result = run(&vec![b'a'; 128 * 1024], &["--unknown"]);
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert!(!result.stderr.is_empty());
}

#[cfg(unix)]
#[test]
fn non_utf8_host_argument_is_an_explicit_failure_not_a_panic() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let output = Command::new(env!("CARGO_BIN_EXE_table-csv"))
        .arg(OsString::from_vec(vec![0xff]))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr).unwrap().contains("UTF-8"));
}

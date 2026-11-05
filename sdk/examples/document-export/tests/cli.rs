#![cfg(feature = "host-cli")]
use std::{
    io::Write,
    process::{Command, Stdio},
};

fn run(input: &[u8], args: &[&str]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_text-docx"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn actual_cli_produces_the_document_and_errors_never_write_success_output() {
    let input = "document\nCaf\u{e9} & 漢字\t😀";
    let output = run(input.as_bytes(), &[]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        output.stdout,
        nieuw_os_document_export::export(input.as_bytes(), usize::MAX).unwrap()
    );
    for input in [b"bad\0".as_slice(), b"CR\r\n", &[0xff]] {
        let output = run(input, &[]);
        assert!(!output.status.success());
        assert!(!output.stderr.is_empty());
        assert!(output.stdout.is_empty());
    }
    let output = run(b"", &["--anything"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}

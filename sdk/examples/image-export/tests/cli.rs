#![cfg(feature = "host-cli")]
use std::{
    io::Write,
    process::{Command, Stdio},
};

const PNG: &[u8] = include_bytes!("../../../../services/image-codec/fixtures/alpha.png");

fn run(input: &[u8], args: &[&str]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_image-bmp"))
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
fn real_host_process_produces_bmp_with_explicit_background() {
    let result = run(PNG, &["--background", "ffffff"]);
    assert!(result.status.success(), "{:?}", result.stderr);
    assert!(result.stderr.is_empty());
    assert_eq!(
        result.stdout,
        nieuw_os_image_export::convert(PNG, 0xffffff, usize::MAX).unwrap()
    );
    assert_eq!(&result.stdout[..2], b"BM");
}

#[test]
fn invalid_input_and_missing_or_invalid_explicit_conversion_choice_never_produce_output() {
    for args in [
        vec![],
        vec!["--background"],
        vec!["--background", "fff"],
        vec!["--background", "zzzzzz"],
        vec!["--background", "+12345"],
        vec!["--background", "000000", "extra"],
    ] {
        let result = run(PNG, &args);
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
        assert!(!result.stderr.is_empty());
    }

    let result = run(b"not an image", &["--background", "000000"]);
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(!result.stderr.is_empty());
}

#[test]
fn rejected_background_closes_stdin_before_large_input_without_test_panic() {
    let result = run(&vec![b'a'; 128 * 1024], &["--background", "zzzzzz"]);
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert!(!result.stderr.is_empty());
}

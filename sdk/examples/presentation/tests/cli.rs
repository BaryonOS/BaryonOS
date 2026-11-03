#![cfg(feature = "host-cli")]
use std::{
    io::Write,
    process::{Command, Stdio},
};

fn run(source: &[u8], args: &[&str]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_slide-preview"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let write = child.stdin.take().unwrap().write_all(source);
    let output = child.wait_with_output().unwrap();
    if let Err(error) = write {
        assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
        assert!(!output.status.success(), "stdin was not fully consumed");
    }
    output
}

#[test]
fn real_process_emits_different_full_ppm_frames_for_selected_pages() {
    let source = include_bytes!("../fixtures/deck.txt");
    let first = run(source, &["--slide", "1"]);
    let second = run(source, &["--slide", "2"]);
    assert!(first.status.success() && second.status.success());
    assert!(first.stderr.is_empty() && second.stderr.is_empty());
    assert!(first.stdout.starts_with(b"P6\n640 360\n255\n"));
    assert_eq!(
        first.stdout.len(),
        b"P6\n640 360\n255\n".len() + 640 * 360 * 3
    );
    assert_ne!(first.stdout, second.stdout);
}

#[test]
fn rejected_scale_closes_stdin_before_large_input_without_test_panic() {
    let output = run(&vec![b'a'; 128 * 1024], &["--slide", "1", "--scale", "3"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn invalid_selection_options_utf8_and_overflow_have_no_success_output() {
    for args in [
        vec![],
        vec!["--slide", "0"],
        vec!["--slide", "3"],
        vec!["--slide", "1", "--scale", "3"],
    ] {
        let output = run(b"Title", &args);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
    for source in [vec![0xff], b"Title\r\nBody".to_vec(), vec![b'W'; 256]] {
        let output = run(&source, &["--slide", "1"]);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

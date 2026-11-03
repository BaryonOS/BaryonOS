# Terminal core

`nieuw-os-terminal-core` is an allocation-free, `no_std` SDK primitive for
editing a UTF-8 command line, parsing a deliberately limited argv syntax, and
validating bounded Message320 terminal-I/O payloads. It supports quotes and
escaping, but rejects unquoted shell operators and expansion syntax instead
of implying support for pipelines, redirection, variables, or command
substitution.

The stdin/stdout payload uses a fixed tag, versioned header, job ID, monotone
sequence, and bounded byte chunk. A receiver binds the expected kernel-reported
peer, job, direction, tag, and capability-free message; EOF closes that
direction. Job IDs and parsed arguments are labels/data, not authority. The
process's exit result must come from the kernel process-status API, not an
untrusted payload.

This crate does not connect the payload to actual process endpoints, launch
programs, render a terminal, or receive keyboard input. A native terminal
application and its authorized image/bootstrap remain separate work.

Run its host tests from the repository root:

```sh
cargo test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-terminal-core
```

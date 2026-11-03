# Terminal core

`nieuw-os-terminal-core` is an allocation-free, `no_std` SDK primitive for
editing a UTF-8 command line and parsing a deliberately limited argv syntax.
It supports quotes and escaping, but rejects unquoted shell operators and
expansion syntax instead of implying support for pipelines, redirection,
variables, or command substitution.

This crate does not launch programs, perform terminal rendering or keyboard
input, define stdin/stdout IPC, or create capabilities. A native terminal
application and a process-I/O contract remain separate work; parsed arguments
grant no process or resource authority.

Run its host tests from the repository root:

```sh
cargo test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-terminal-core
```

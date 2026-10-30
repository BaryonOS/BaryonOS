# Verification

Basis `ca45a7dabe6a040921de03bdc2fad3e5045a75ba`, Rust 1.90.0, crate-local
lockfile, own `target/`. Logs and exit codes live next to this file;
`SHA256SUMS` binds sources, lockfile and logs (`sha256sum -c SHA256SUMS`
from `drivers/virtio-snd`).

```sh
cargo +1.90.0 fmt --check
cargo +1.90.0 clippy --offline --locked --all-targets -- -D warnings
cargo +1.90.0 test --offline --locked
cargo +1.90.0 build --offline --locked --release --target x86_64-unknown-none
```

Evidence boundary: host unit tests and no_std crosscompile. No QEMU guest,
audio output, transport, physical hardware or human audit. The 6 prototype
tests are carried over (annotated in `src/tests.rs`) and extended with
feature, range/overflow, multi-record and reader-bounds negatives.

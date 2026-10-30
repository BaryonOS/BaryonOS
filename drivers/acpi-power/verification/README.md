# Standalone ACPI parser verification

## Basis and scope

- Source base: `6daadbba37d09b6795b419febf3e576bf55a093`.
- Rust toolchain: `1.90.0`; Cargo ran offline against the existing local
  toolchain/cache.
- Scope: only `drivers/acpi-power/**`; no kernel, BootInfo, build-profile,
  manifest, or shared release evidence was changed.
- Test inputs are synthetic host fixtures. No firmware tables were read from a
  guest, no QEMU run was performed, and no physical hardware was tested.
- This crate does not constitute H05 acceptance.

## Commands and results

Run from the repository root with the selected Rust toolchain available:

| Check | Exact command | Exit | Output |
|---|---|---:|---|
| Rust formatting | `cargo +1.90.0 fmt --manifest-path drivers/acpi-power/Cargo.toml -- --check` | 0 | `fmt.log` |
| Unit and negative tests | `cargo +1.90.0 test --offline --locked --manifest-path drivers/acpi-power/Cargo.toml` | 0; 11 passed | `tests.log` |
| Clippy | `cargo +1.90.0 clippy --offline --locked --manifest-path drivers/acpi-power/Cargo.toml --all-targets -- -D warnings` | 0 | `clippy.log` |
| `no_std` release build | `cargo +1.90.0 build --target x86_64-unknown-none --release --offline --locked --manifest-path drivers/acpi-power/Cargo.toml` | 0 | `build.log` |

Each command's numeric exit status is stored in its matching `.exit` file.
`SHA256SUMS` binds the crate source, manifest, lockfile, this contract, logs,
status files, and the generated release `.rlib`. The build artifact is local
and ignored by Git; its hash is recorded for reproducibility, not as a shipped
kernel artifact.

## Still open

The loader-to-BootInfo ACPI snapshot, exact versioned ABI layout, kernel
independent table/range/reservation validation, trusted broker integration,
QEMU-profile-bound poweroff action, guest shutdown and native negative cases
remain release-owner work. The snapshot metadata must include table kind,
offset/length and original firmware source address, and must preserve existing
BootInfo v1.1 and graphics v1.2 layouts. Ring-3 parser metadata is not
authority; the kernel must independently validate the handoff and never accept
a serialized `S5Plan`, port, or sleep type from the shutdown caller.

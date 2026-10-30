# Standalone ACPI parser verification

## Basis and scope

- Source base for this follow-up: `50e5954ceb958b08178f94a77fff02e9fa952feb`.
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
| Unit and negative tests | `cargo +1.90.0 test --offline --locked --manifest-path drivers/acpi-power/Cargo.toml` | 0; 22 passed | `tests.log` |
| Clippy | `cargo +1.90.0 clippy --offline --locked --manifest-path drivers/acpi-power/Cargo.toml --all-targets -- -D warnings` | 0 | `clippy.log` |
| `no_std` release build | `cargo +1.90.0 build --target x86_64-unknown-none --release --offline --locked --manifest-path drivers/acpi-power/Cargo.toml` | 0 | `build.log` |

Each command's numeric exit status is stored in its matching `.exit` file.
`SHA256SUMS` binds the crate source, manifest, lockfile, this contract, logs,
status files, and the generated release `.rlib`. The build artifact is local
and ignored by Git; its hash is recorded for reproducibility, not as a shipped
kernel artifact.

## Package-length regression

The new test
`validates_all_pkg_length_widths_and_rejects_reserved_lead_bits` failed before
the fix: a two-byte package-length lead with bit 4 set produced an accepted
S5 plan instead of `MalformedAml`. The captured pre-fix run is
`pkg-length-before.log` with exit 101 in `pkg-length-before.exit`. Its exact
command was
`cargo +1.90.0 test --offline --locked --manifest-path drivers/acpi-power/Cargo.toml validates_all_pkg_length_widths_and_rejects_reserved_lead_bits`.
That expected red result is not a successful acceptance run. The updated
parser explicitly rejects nonzero unused lead bits in multi-byte lengths.

The four new tests also cover one through three continuation bytes, truncated
length encodings, undersized/oversized packages, every proper prefix of eight
AML object fixtures both alone and after a valid S5, and every proper prefix
of both input tables. No panic is intercepted: one would fail the host test.
This is bounded fixture coverage, not exhaustive fuzzing or native proof.

## PM1 GAS no-fallback matrix

The next two tests independently mutate every interpreted GAS field for PM1a
and PM1b with valid legacy addresses retained. Invalid address spaces, widths,
offsets, access sizes, overflowing 16-bit I/O addresses and `u64::MAX` must
fail rather than fall back to those legacy fields. Extended register ranges
that overlap by either byte are rejected. An incorrect nonzero `X_DSDT`
address also fails even when the legacy DSDT address matches. This follow-up
adds negative coverage without changing the public API or register-selection
behavior.

## QEMU S5 object-shape correction

The QEMU `v10.2.1` table builder places `_S5_` inside literal `Scope(\)` and
uses four package elements (types A/B, reserved zero/zero). The original
parser skipped this scope and accepted only two fields. The new bounded
root-scope traversal and four-element support address that concrete
compatibility blocker without general AML execution. The public QEMU source
URL and exact source SHA-256 are in the crate README; no upstream code was
copied.

Five tests cover that shape, nonzero/truncated reserved fields, scope-boundary
escapes, resuming after scopes, duplicate definitions across scopes, and the
eight-level fixed-stack bound. Existing non-root-scope byte-pattern rejection
remains. A full guest-firmware DSDT, loader handoff and native poweroff remain
unverified; a synthetic QEMU-shaped fixture is not a native pass.

## Still open

The loader-to-BootInfo ACPI snapshot, exact versioned ABI layout, kernel
independent table/range/reservation validation, trusted broker integration,
QEMU-profile-bound poweroff action, guest shutdown and native negative cases
remain release-owner work. The snapshot metadata must include table kind,
offset/length and original firmware source address, and must preserve existing
BootInfo v1.1 and graphics v1.2 layouts. Ring-3 parser metadata is not
authority; the kernel must independently validate the handoff and never accept
a serialized `S5Plan`, port, or sleep type from the shutdown caller.

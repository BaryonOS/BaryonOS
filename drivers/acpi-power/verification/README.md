# Standalone ACPI parser verification

## Basis and scope

- Source base for this follow-up: `4af90fcdf0a7136307428523657fa23a05d45a11`.
- Rust toolchain: `1.90.0`; Cargo ran offline against the existing local
  toolchain/cache.
- Scope: only `drivers/acpi-power/**`; no kernel, BootInfo, build-profile,
  manifest, or shared release evidence was changed.
- The default suite uses synthetic host fixtures. An additional explicit
  host test uses complete tables read from a separate QEMU/OVMF firmware
  guest, with negative mutations of those same bytes. This is not a
  Nieuw-OS guest run. No physical hardware was tested.
- This crate does not constitute H05 acceptance.

## Commands and results

Run from the repository root with the selected Rust toolchain available:

| Check | Exact command | Exit | Output |
|---|---|---:|---|
| Rust formatting | `cargo +1.90.0 fmt --manifest-path drivers/acpi-power/Cargo.toml -- --check` | 0 | `fmt.log` |
| Unit and negative tests | `cargo +1.90.0 test --offline --locked --manifest-path drivers/acpi-power/Cargo.toml` | 0; 22 passed, 1 explicitly ignored | `tests.log` |
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
separate gates; the full guest-firmware DSDT host test below now passes.
A synthetic QEMU-shaped fixture is not a native pass.

## Complete QEMU/OVMF firmware tables

The private firmware-only capture used QEMU 10.2.1 (Debian
`1:10.2.1+ds-1ubuntu3.3`), `pc-q35-8.2`, `Nehalem-v1`, TCG, one CPU and
8 GiB. It deliberately omitted the release disk, NIC, virtio devices,
Nieuw-OS loader/kernel and visible display. Only a private copy of OVMF
VARS was writable. There were no host disks, host devices or network
backends. These deviations make it a firmware diagnostic, not VM-FR.

`qemu-firmware/report.json` records the exact launch command, firmware
hashes, running/paused QMP responses, addresses, lengths, checksums-by-hash
and elapsed capture time (20.146 s). `qemu-firmware/qmp-trace.json` records
the actual QMP commands and responses: start, stop, bounded memory reads,
table reads and `quit`. QEMU exited 0 after QMP quit, **not guest shutdown**.
The host waited 20 seconds to collect this diagnostic snapshot; that is
not a validated boot-readiness condition or measured boot/poweroff latency.
`qemu-firmware/tool-hashes.txt` binds the wrapper and underlying executable.

| Selected table | Firmware address (decimal) | Bytes | SHA-256 |
|---|---:|---:|---|
| FADT | 2142736384 | 244 | `410716dfb169eaba296ed3c336028843b3cf9fce6ca7c179bb242199cfec9d1a` |
| DSDT | 2142740480 | 8344 | `95bb7cd45de3e16a85206f9f50cb6597b3b8dd97ec4afef3e3004ef3170430e5` |

The bounded diagnostic searched saved low guest memory (1 MiB) and the top
32 MiB of below-4G RAM, checked the RSDP checksums/length, then followed the
XSDT and FADT DSDT pointer. Each selected SDT had its signature, length
(36 bytes through 1 MiB) and checksum checked before parsing. This host
debugger discovery is **not** the UEFI configuration-table handoff required
of the real loader, and authenticates no kernel snapshot. A legacy-style
16-byte candidate stride initially missed OVMF's ACPI2 structure at
`2142756884`, immediately after a 20-byte legacy RSDP. The diagnostic was
changed to inspect every signature position; no loader alignment rule or
parser API was changed or inferred from this observation.

Two preliminary diagnostic attempts failed: missing explicit QEMU ROM
paths, then the candidate-stride mistake above. They are not successful
gates. Their original reports and memory snapshots remain in this session's
`files/h05-firmware-4af90fc/` and `files/h05-fw2/`, outside Git.
The successful capture and raw table files remain in
`files/h05-fw3/`; generated firmware bytes and broad memory dumps are not
committed. The one-off capture source is `files/h05_firmware_probe.py`;
its hash is recorded with the tool hashes. This is not a shared runner.

The test obtains no port-I/O authority and makes no hardware call. It checks
the complete original tables produce PM1a `0x604`, S5 type 0 and no PM1b.
It then rejects every proper table prefix, wrong signatures/checksums,
`u32::MAX` declared lengths, a checksummed `u64::MAX` PM1a GAS address, a
mismatched DSDT source address and checksummed AML suffixes containing an
unsupported opcode, a truncated scope and a duplicate S5 declaration.
No parser panic is caught or converted into success.

To repeat against the retained fixture, set these four variables explicitly
(paths may be relocated; addresses are decimal and describe this capture):

```sh
export NOS_ACPI_FADT=/home/simon/.copilot/session-state/b6c80d1bf5744c2e81c568572bf7c83d/files/h05-fw3/fadt.bin
export NOS_ACPI_DSDT=/home/simon/.copilot/session-state/b6c80d1bf5744c2e81c568572bf7c83d/files/h05-fw3/dsdt.bin
export NOS_ACPI_FADT_ADDRESS=2142736384 NOS_ACPI_DSDT_ADDRESS=2142740480
cargo +1.90.0 test --offline --locked --manifest-path drivers/acpi-power/Cargo.toml -- --ignored --exact tests::validates_external_qemu_tables_and_rejects_mutations
```

Result: exit 0, one external-fixture test passed; log
`qemu-firmware-test.log` and numeric status `qemu-firmware-test.exit`.
This opt-in test is intentionally ignored by the normal synthetic suite.
Missing input is a failure when explicitly invoked. Fixture checksums and
source addresses are test inputs, not caller authority for a shutdown.

No product or rubric row closes: proposed total 34% to 34%, hardware 1/12
to 1/12. The manager owns central status/assessment. The remaining major
VM-FR device gap identified read-only is virtio-snd/audio; it has no second
active writer reservation here. The next H05 integration remains with the
release owner; current main has assigned syscall `0x73` to native clock,
so the historical shutdown-slot suggestion must not be reused unchecked.

## Still open

The loader-to-BootInfo ACPI snapshot, exact versioned ABI layout, kernel
independent table/range/reservation validation, trusted broker integration,
QEMU-profile-bound poweroff action, guest shutdown and native negative cases
remain release-owner work. The snapshot metadata must include table kind,
offset/length and original firmware source address, and must preserve existing
BootInfo v1.1 and graphics v1.2 layouts. Ring-3 parser metadata is not
authority; the kernel must independently validate the handoff and never accept
a serialized `S5Plan`, port, or sleep type from the shutdown caller.

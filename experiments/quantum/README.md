# Q01 — local quantum simulator

This host experiment executes `qsim-v0` circuits locally with a custom
statevector engine. The engine uses `no_std + alloc`, has no external
dependencies, and forbids `unsafe`. This is not a port to the project's
own OS, a physical quantum processor, quantum acceleration, or a cryptographic
implementation.

## Running

From the project root, with the project Rust toolchain on `PATH`:

```powershell
cargo +1.90.0 run --manifest-path experiments/quantum/Cargo.toml --locked -p qsim-cli
```

The program reads one complete JSON document from stdin through EOF and writes
one JSON response to stdout. To run the Bell fixture in PowerShell:

```powershell
Get-Content -Raw experiments/quantum/fixtures/bell8.json | cargo +1.90.0 run --manifest-path experiments/quantum/Cargo.toml --locked -p qsim-cli
```

The following process environment can be used with the project-local Windows
toolchain:

```powershell
$env:CARGO_HOME = "$env:LOCALAPPDATA\nieuw-os\cargo"
$env:RUSTUP_HOME = "$env:LOCALAPPDATA\nieuw-os\rustup"
$env:PATH = "$env:CARGO_HOME\bin;$env:PATH"
```

The `nieuw-os` directory components are existing local toolchain
identifiers and remain unchanged. These assignments affect only the current
shell. After the first dependency fetch, a build works with `--offline`.
On Windows, the built program is stored at
`experiments/quantum/target/debug/qsim-cli.exe`. It opens no files,
network ports, NAS connection, or cloud endpoint; stdin and stdout are its only
user interface.

## Protocol and limits

Required fields are `schema`, `job_id`, `qubits`,
`gates`, `shots`, and `seed`. The schema is exactly
`qsim-v0`. A job ID contains 1–64 ASCII letters, digits, `_`, or
`-`. A seed consists of exactly 16 lowercase hexadecimal characters.
Gates are limited to `h`, `x`, `z`, and `s` with
`target`, or `cx` with both `control` and `target`.
Unknown, missing, or duplicate fields; `control` on single-qubit gates;
negative or fractional integers; trailing JSON; and excessively deep or
malformed data are rejected.

| Resource | Limit |
|---|---:|
| Request | 262,144 bytes; the reader stops after limit + 1 |
| Qubits | 1–12 |
| Gates | 0–4,096; bounded during deserialisation |
| Shots | 1–65,536 |
| Work budget | `(gates + 1) × 2^qubits + shots × (qubits + 1) ≤ 20,000,000` |
| CLI execution deadline | 10 seconds, monotonic clock after validation |
| Test process including I/O | 30 seconds |

Qubit 0 is the least-significant bit; strings display
`q[n-1]…q[0]`. Output contains only non-zero counts in lexicographic
order. All measurements occur at the end of the circuit. Reset, pulse, noise,
and mid-circuit measurements are not supported.

SplitMix64 provides reproducible sampling: it is **not a cryptographic RNG and
must never be used for keys or system entropy**. Source and applicable
permission are recorded in [THIRD-PARTY.md](THIRD-PARTY.md).

Exit status: 0 success; 2 input/schema; 3 quota/memory; 4 cancellation/deadline;
5 numerical/internal error. Error responses contain fixed field names and never
raw input. An invalid job ID, or one that has not yet been read reliably, is
reported as `null`.

## Checks and actual evidence

Run from the project root:

```powershell
python experiments/quantum/verify.py
```

The script also locates the project-local Windows toolchain, installs nothing,
uses Cargo offline, checks formatting, compiles and runs the Rust tests, checks
Clippy with no allowed warnings, and compiles the core for
`x86_64-unknown-none`. It then runs the five separate fixtures with a
30-second limit per process. Raw logs, output, source and binary hashes, and
exact commands are stored under `verification/`; that directory is a
locally generated artifact.

These checks actually passed on 27 October 2026 with Rust 1.90.0 and the Windows
MSVC host toolchain:

- 25 Rust tests passed. The tests cover Q1-01 through Q1-17 from the
  specification, in some cases as grouped tests. Q1-16 did not run with the
  network physically blocked; the narrower offline and dependency checks are
  described below.
- Bell8, seed 0: exactly `00:5, 11:3`; Bell4096: exactly
  `00:2082, 11:2014`.
- Identity: only `000`; X on qubit 1: only `10`.
- The maximum job ran with 12 qubits, 4,096 gates, and 65,536 shots. Measured
  total engine-buffer capacity, including the gate buffer, was **212,992
  bytes**. This excludes input, the JSON parser, allocator overhead, the stack,
  and the host runtime; it is not a measurement of total process memory.
- Cancellation before execution, during gates, and during sampling returns an
  error without a partially completed result. The deadline is tested
  deterministically through `Control`.
- Phase, norm, reversible gates, and erroneous/NaN/infinite internal amplitudes
  were tested directly; histogram tests alone would miss phase errors.
- Contract audit Q01V corrected a rounding edge case: the CDF endpoint is set
  to 1 from the last positive probability so a trailing zero mass cannot
  receive a rounding remainder. A fixed unit test fails with the previous
  implementation and passes after the correction. An additional test checks
  the limits of 256 basis indices and 64 shots between checkpoints separately.
- Standalone fixture execution and offline builds pass. The dependency tree
  contains no network client. The host was not physically disconnected from the
  network and no OS network sandbox was configured for this check.

The engine reserves job buffers fallibly and returns `ResourceExhausted`
when a reservation fails. The host adapter and Serde additionally use bounded
regular Rust/parser allocations; global host-memory exhaustion has not been
tested as a recoverable process-error scenario. The 10-second core deadline
does not interrupt blocked stdin or stdout. No IPC cancellation object or
project-OS floating-point/context-save ABI exists yet. These limits belong to
the next userspace integration step, not to a kernel port of the simulator.

## Design and recovery

`ValidatedJob` has private fields. Targets, shifts, allocation sizes, and
work calculations are validated before execution. The core has no clock,
environment, files, or network access. The caller supplies `Control`;
checkpoints occur before allocation, before each gate, after at most 256 basis
indices, and after at most 64 shots. A cancelled job produces no histogram.

The quantum workspace is independent and remains outside the root kernel
workspace. Removing this experiment directory, or reverting only its future
Git change, removes the experiment without changing OS, NAS, or host settings.
The command above recreates build artifacts and checks.

## Next step

Obtain an independent code review and then connect a bounded userspace service
to IPC cancellation, a correct floating-point context, and concrete scheduler
and memory quotas once the project's own OS supports them. Physical QPU drivers
and post-quantum cryptography remain separate tasks.

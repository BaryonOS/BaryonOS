# Q0 complete: build specification for local quantum execution
Research date: 26 October 2026. Status: **selected specification for Q1**, ready for implementation. This document closes the Q0 decision questions for the first simulator. Reopen these decisions only when a problem has been demonstrated, not through another general research cycle.

## 1. Recorded choices

| Component | Decision |
|---|---|
| First result | A local command-line simulator on the development host, plus a reusable computation library |
| Implementation | Safe Rust; small custom statevector engine; no `unsafe`, Python runtime, GPU framework or external quantum service |
| Library | `qsim-core`: `#![no_std]` with `alloc`; no files, network, clocks or environment access |
| Host program | `qsim-cli`: `std`, one JSON request through stdin, one JSON response through stdout |
| Format | Small custom **job protocol qsim-v0**; no new general-purpose programming language or OpenQASM/QIR compatibility claim |
| Gates | H, X, Z, S, CX; initialisation always `\|0…0⟩`; complete final measurement only |
| Outside Q1 | Mid-circuit measurement, reset, classical conditionals, arbitrary matrix input, pulses, noise model, hardware control |
| Computation | Complex amplitudes with two `f64` fields; no fast-math or hardware-dependent acceleration paths |
| Final objective | Same engine behind a userspace service on the custom OS, later an interchangeable local QPU backend |
| Privacy | No network code, cloud credentials, telemetry, model download or NAS access in simulator/runtime |

OpenQASM covers substantially more language and timing functionality and does not itself define the complete execution environment. QIR focuses on interoperable hybrid programs. Both remain possible future adapters, but either would broaden the small initial experiment unnecessarily (OpenQASM Contributors, n.d.; QIR Alliance, n.d.). The selected JSON job protocol is an auditable internal boundary, not a competitor to those standards.

Q1 initially runs as a host process. It may run on the development computer; this does not cause the new operating system to inherit the host kernel. A genuine port to the custom operating system and a genuine QPU run remain separate outcomes.

## 2. Boundaries before allocation and execution
All maxima below are selected project budgets, not hardware claims.

| Field/resource | Exact boundary |
|---|---|
| UTF-8 request | Maximum 262,144 bytes; stop reading after limit + 1 |
| Schema | Exactly `qsim-v0` |
| `job_id` | 1–64 ASCII characters from `A-Z a-z 0-9 _ -` |
| `qubits` | Integer 1–12 |
| `gates` | Array of 0–4,096 elements |
| `shots` | Integer 1–65,536 |
| `seed` | Required: exactly 16 lowercase hexadecimal characters; unsigned 64-bit simulator seed |
| Work budget | `(gate_count + 1) * (1 << qubits) + shots * (qubits + 1) <= 20_000_000`; all calculations are checked |
| Q1 concurrency | One job per CLI process; no internal queue or worker pool |
| Test-harness timeout | 30 seconds for one test process; actual benchmark results determine later whether adjustment is necessary |

JSON is deserialised into strong types. Unknown fields, missing fields, duplicate keys, unknown gate variants, negative or fractional integers, unknown schemas, and trailing data are rejected. Use `deny_unknown_fields`, not `flatten`, and do not use an untyped intermediate `Value` that can conceal duplicate keys. Bound the gate array to 4,096 entries during deserialisation; do not first populate an unbounded `Vec`. Retain the parser recursion limit and add a depth test. The request contains no filenames, shell commands, or URLs.

At 12 qubits, the amplitude vector contains 4,096 elements × 16 bytes = 65,536 bytes. The CDF and counter array require 32,768 and 16,384 bytes respectively: 114,688 bytes in total, excluding the gate buffer, input, allocator, and host runtime. Target at most 512 KiB for engine-owned job buffers. Measure this value; do not present it as total process RAM. Reserve bounded buffers with fallible allocation and return `RESOURCE_EXHAUSTED` on failure; untrusted input must not reach a panic path.

## 3. Computational contract
Amplitudes are stored in ascending basis-index order. Qubit `q` is bit `q` of the index; qubit 0 is the least significant bit. A result string displays `q[n-1] … q[0]`. This choice is consistent with the documented Qiskit indexing convention but does not require a Qiskit dependency (IBM, n.d.).

`Complex { re: f64, im: f64 }`; initial vector: amplitude 0 = (1, 0), all others = (0, 0). User-supplied initial amplitudes are not accepted.

For every gate on `q`, take pairs `i` for which bit `q = 0` and `j = i | (1 << q)`:
- X: swap `a[i]` and `a[j]`.
- H: the new values are `(a[i]+a[j])/√2` and `(a[i]-a[j])/√2`. Use `core::f64::consts::FRAC_1_SQRT_2`; calculate both from the old values.
- Z: multiply only `a[j]` by −1.
- S: multiply only `a[j]` by `i`: `(re, im) → (-im, re)`.
- CX: control and target must differ and both be less than `qubits`. Swap only pairs with control bit 1 and target bit 0; never swap the same two amplitudes twice.

After all gates, calculate `p[k] = re² + im²` in ascending index order. Non-finite or negative values and `|sum(p)-1| > 1e-9` produce `NUMERICAL_ERROR`. For sampling, divide the probabilities by that sum; this corrects only rounding noise within the permitted margin. Build an ascending cumulative distribution and set the last element to exactly 1.0. Samples select the first index for which `CDF[k] > u`; a zero-mass CDF entry is therefore never selected.

This repeatedly samples the same final state, which is equivalent to preparing the state again and measuring at the end for this restricted set of unitary circuits. It is not an implementation of mid-circuit measurement or a physical quantum-randomness source.

### Deterministic sampler
Use SplitMix64 only for reproducible simulation (Vigna, 2015):
- The state starts with the hexadecimal `seed`.
- For each draw, first add `0x9e3779b97f4a7c15`, then apply the published xor-shift/multiply mix with `0xbf58476d1ce4e5b9`, `0x94d049bb133111eb`, and shifts 30, 27, and 31.
- All integer additions and multiplications wrap modulo 2^64.
- Convert the upper 53 bits of the result to `u = (word >> 11) * 2^-53`, so `0 <= u < 1`.
- Perform exactly one draw per shot. There are no hidden additional draws.

This is **not a cryptographic RNG**, a quantum RNG, or a source for operating-system, NAS, or update keys. Retain the source attribution and applicable permission for the algorithm implementation used. Kernel cryptography must not import this module.

## 4. Exact job and result format
All fields in this example are required. Gates contain only the fields shown. Gate order is execution order. A final measurement is implicit.

```json
{
  "schema": "qsim-v0",
  "job_id": "bell-8",
  "qubits": 2,
  "gates": [
    {"op": "h", "target": 0},
    {"op": "cx", "control": 0, "target": 1}
  ],
  "shots": 8,
  "seed": "0000000000000000"
}
```

Selected success format, with the histogram in lexicographic bit-string order and without zero values:

```json
{
  "schema": "qsim-v0",
  "job_id": "bell-8",
  "backend": "local-statevector-v0",
  "status": "completed",
  "qubits": 2,
  "shots_completed": 8,
  "counts": {"00": 5, "11": 3},
  "seed": "0000000000000000"
}
```

For X/Z/S/H: `{"op":"x","target":0}`, analogously for the other letters. CX is named exactly `cx`, not `cnot`; v0 has no aliases. Retain the schema and backend version in results. By default, output contains no amplitudes, input dump, timestamp, hostname, or machine ID.

Error format:

```json
{
  "schema": "qsim-v0",
  "job_id": null,
  "backend": "local-statevector-v0",
  "status": "error",
  "error": {"code": "INVALID_REQUEST", "field": "qubits"}
}
```

Use `job_id: null` until the ID has been validated reliably. After validation, the valid ID may be returned. `field` is a fixed field name, not raw input. Error codes are `INVALID_REQUEST`, `UNSUPPORTED_SCHEMA`, `RESOURCE_EXHAUSTED`, `NUMERICAL_ERROR`, `CANCELLED`, `DEADLINE_EXCEEDED`, and `INTERNAL_ERROR`. Input/version errors: exit 2; budget/memory: exit 3; cancellation/deadline: exit 4; numerical/internal error: exit 5. Success: exit 0.

An early end to stdin without complete JSON is `INVALID_REQUEST`. Logs containing technical details go to stderr without circuit or user content. There is no server port.

## 5. Cancellation, deadlines, and the backend boundary
`qsim-core` provides one synchronous operation on validated input, with a caller-supplied control:

```rust
pub trait Control {
    fn checkpoint(&mut self, work_done: u64) -> Result<(), Abort>;
}
pub enum Abort { Cancelled, DeadlineExceeded }
pub fn execute(job: &ValidatedJob, control: &mut impl Control)
    -> Result<JobResult, ExecuteError>;
```

The core has no wall clock or global signal handler. A checkpoint occurs before allocation, before every gate, during loops after at most 256 basis indices, before sampling, and after at most 64 shots. The caller compares monotonic time with its deadline or signals cancellation. Checkpoints make cooperative interruption testable; they provide no real-time guarantee. Test both cancellation before execution and cancellation during execution. After cancellation, do not return a partial histogram as a completed result.

The CLI uses a fixed monotonic execution deadline of 10 seconds after receipt and validation of the bounded request; tests may use a direct `Control` stub for deterministic deadline cases. This deadline does not interrupt a blocked stdin read or stdout write. The encompassing test harness limits the entire process, including input/output, to 30 seconds. Q1 need not add a cross-platform Ctrl+C library: process termination remains possible, while cooperative cancellation is demonstrated through the core API. The future userspace service connects the control to its IPC cancellation object, I/O deadlines, and scheduler quota.

The backend boundary to be implemented later provides `capabilities`, `execute`, and `cancel`, with schema/backend version and resource boundaries. Q1 implements only the local simulator. Do not load a backend name or endpoint from user JSON. Q2/Q3 introduce explicitly registered local hardware adapters with their own controls, driver permissions, and reset/isolation evidence.

## 6. Files and dependencies to create for Q1
Under `experiments/quantum/`:
- `Cargo.toml`: separate workspace, not a dependency of the kernel or boot code.
- `qsim-core/src/{lib,complex,gate,validate,statevector,sampler}.rs`.
- `qsim-core/tests/{gates,validation,cancellation,fixtures}.rs`.
- `qsim-cli/src/main.rs`: bounded stdin/typed JSON, core call, result encoding.
- `qsim-cli/tests/cli.rs`: exit status, limits, stdout schema.
- `fixtures/{identity,x1,bell8,bell4096,invalid-target}.json`.
- `README.md`: commands, exact gates, limits, seed warning, and evidence status.

Core: zero external dependencies. JSON is used only in the host adapter. Proposed explicit parser pins based on the maintainer documentation consulted are `serde = "=1.0.229"` with `derive` and `serde_json = "=1.0.154"`. The latest documentation that was opened reports these versions; the exact serde page was accessible, but the direct serde_json version page was not. Registry fetching and compatibility checks have **not yet been performed**. The implementation agent tries these pins once, records Cargo.lock, and reports an actual resolution failure. Only when unavailability has been demonstrated may a verified available version be recorded in DEPENDENCIES.md; there is no silent unpinned upgrade. This dependency check does not block the dependency-free core.

Use the project's Rust toolchain from BOOT-AND-TOOLCHAIN.md once it has been established. The quantum experiment compiles for the host target; the later userspace port to the custom operating system also requires a correct floating-point/context-save ABI. Do not place the simulator in the kernel to bypass that requirement.

After adding the files:
```bash
cargo test --manifest-path experiments/quantum/Cargo.toml --workspace --locked
cargo clippy --manifest-path experiments/quantum/Cargo.toml --workspace --all-targets --locked -- -D warnings
cargo run --manifest-path experiments/quantum/Cargo.toml --locked -p qsim-cli < experiments/quantum/fixtures/bell8.json
```
These are execution commands for the future implementation, **not project tests that have already passed**.

## 7. Concrete acceptance tests

| ID | Test | Required result |
|---|---|---|
| Q1-01 | No gates, n=3 | Only `000`, count = shots |
| Q1-02 | X on q1, n=2 | Only `10`; detects reversed bit order |
| Q1-03 | H followed by H | Returns to the initial vector; complex difference per amplitude <= 1e-12 |
| Q1-04 | X/X, Z/Z, S/S/S/S, CX/CX | Returns to the initial vector for several fixed basis and superposition states |
| Q1-05 | H(q0), S(q0), n=1 | Amplitudes (1/√2, i/√2); tests phase, not only the histogram |
| Q1-06 | H(q0), CX(q0,q1), n=2 | Only indices 0 and 3 with probability 0.5 ± 1e-12; norm 1 ± 1e-12 |
| Q1-07 | Bell8, seed 0 | Exactly `00:5,11:3`; result shown above |
| Q1-08 | Bell4096, seed 0 | Exactly `00:2082,11:2014`; no other strings |
| Q1-09 | Repeat a validated job | Same histogram and schema; no timestamp fields |
| Q1-10 | q=0/13, target>=n, CX control=target, unknown gate | Rejected before state-vector allocation |
| Q1-11 | 4,097 gates, 65,537 shots, >256 KiB, overflow input | Bounded error; no panic, hang, or truncated "success" |
| Q1-12 | Duplicate key, extra field, invalid seed, fractional qubits, trailing JSON | Specific error/exit 2 |
| Q1-13 | Cancelled before or during gate execution/sampling | `CANCELLED`, no completed or partial result |
| Q1-14 | Deadline reached through test `Control` | `DEADLINE_EXCEEDED`, deterministic test without sleeping |
| Q1-15 | Invalid/non-finite internal amplitudes injected in a unit test | `NUMERICAL_ERROR`; do not normalise as if the state were valid |
| Q1-16 | Offline execution after the build | Bell fixture passes without a network; dependency tree contains no network client |
| Q1-17 | Valid maximum input | Finishes within the host-test budget or returns an explicit deadline error; no unbounded allocation |

State-vector tests use analytical states; sampler tests use fixed fixtures so that CI does not fail randomly. An additional sampling experiment with varying seeds may examine statistics but does not replace the fixed regression tests. Performance remains unknown until actual implementation and measurement.

**Specification check performed:** an independent Python reference calculation of the published SplitMix64 mix and Bell distribution produced the stated 8-shot and 4,096-shot histograms for seed 0 on 26 October 2026. The vector-memory calculation (114,688 bytes) and maximum work budget (17,633,280 units) were also checked arithmetically. No Rust simulator was built or tested; the Python calculation validates fixtures and is not evidence of the future implementation.

## 8. Decided PQC policy for later system integration
PQC is a separate security component and is not required to build the Q1 simulation.

| Use | Decided starting point | Implementation boundary |
|---|---|---|
| New shared secrets in a suitable transport protocol | ML-KEM-768 as the intended PQ component | No custom handshake or hybrid protocol; use a final standardised protocol binding and maintained library during the networking phase |
| Signed operating-system/package metadata | ML-DSA-65 as the intended custom update profile | Test trust anchors, canonical metadata, rollback policy, key rotation, and recovery together |
| Symmetric data protection | AES-256 as a candidate within an existing authenticated storage/message format | Do not improvise the mode, nonce/tweak policy, authentication, or key lifecycle |
| Algorithm migration | Explicit algorithm ID, format version, and restricted allowlist | No insecure negotiated fallback; permit old versions only under an explicit migration policy |
| Entropy/keys | Trustworthy system entropy and a controlled key lifecycle | Never use the simulator seed or SplitMix64 |

The parameter choices are project decisions, not certification or a claim of suitability for every future device. NIST FIPS 203 and 204 describe a KEM and digital signatures respectively; SP 800-227 provides recommendations for applying KEMs. The FIPS 203 and 204 publication pages identify errata, which must be handled explicitly during cryptographic integration (NIST, 2024a, 2024b; Alagic et al., 2025). An unchanged classical firmware chain of trust does not become post-quantum secure merely by adding a PQ signature to a later package.

Q1 implements **none** of these cryptographic primitives. Writing a new cryptographic implementation for originality is outside the scope; maintained, reviewed libraries are selected in the concrete integration task.

## 9. What Q0 closes
The first simulator representation, gates, bit order, boundaries, output, sampling, error codes, cancellation model, file layout, and test fixtures have been decided. Q1 can be implemented directly from this specification. Specific physical QPU drivers, real-time control electronics, and evidence for a hybrid system port remain Q2/Q3 tasks; they do not block Q1 or the first operating-system boot.

## References (APA 7)
Alagic, G., Barker, E., Chen, L., Moody, D., Robinson, A., Silberg, H., & Waller, N. (2025). *Recommendations for key-encapsulation mechanisms* (NIST Special Publication 800-227). National Institute of Standards and Technology. https://doi.org/10.6028/NIST.SP.800-227

IBM. (n.d.). *Bit-ordering in the Qiskit SDK*. Accessed 26 October 2026, at https://quantum.cloud.ibm.com/docs/en/guides/bit-ordering

National Institute of Standards and Technology. (2024a). *Module-lattice-based key-encapsulation mechanism standard* (FIPS 203). https://doi.org/10.6028/NIST.FIPS.203 — current errata reference: https://csrc.nist.gov/pubs/fips/203/final

National Institute of Standards and Technology. (2024b). *Module-lattice-based digital signature standard* (FIPS 204). https://doi.org/10.6028/NIST.FIPS.204 — current errata reference: https://csrc.nist.gov/pubs/fips/204/final

OpenQASM Contributors. (n.d.). *Introduction: OpenQASM live specification*. Accessed 26 October 2026, at https://openqasm.com/intro.html

QIR Alliance. (n.d.). *QIR Alliance*. Accessed 26 October 2026, at https://www.qir-alliance.org/

Serde Contributors. (n.d.-a). *Container attributes*. Accessed 26 October 2026, at https://serde.rs/container-attrs.html

Serde Contributors. (n.d.-b). *Serde 1.0.229*. https://docs.rs/serde/1.0.229/serde/

Serde JSON Contributors. (n.d.). *Serde JSON*. Accessed 26 October 2026, at https://docs.rs/serde_json/latest/serde_json/

Vigna, S. (2015). *SplitMix* [Source code]. https://prng.di.unimi.it/splitmix.c

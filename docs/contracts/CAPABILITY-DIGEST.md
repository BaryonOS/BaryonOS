# Capability digest: production selects the originally measured calculation

CAP-DIGEST-TCG-2 restores `Engine::state_digest()` to the complete original
calculation from commit
`3f20f087a7f0ddf4757af8d3ea110dd0c044a72c`. This is the production choice
following the focused native comparison. The historical investigation and
measurement protocol are in [CAP-DIAGNOSTIC-DIGEST.md](CAP-DIAGNOSTIC-DIGEST.md);
the choice below supersedes the earlier temporary selection of the zero-run variant.

The production hash once again processes every typed word with the same
`h = (h XOR word) * 0x100000001b3 modulo 2^64`, starting from
`0xcbf29ce484222325`. Each message is serialised to all 320 bytes with the
existing `Message::encode()` and processed as 40 little-endian words.
All fields, generations, invalid logical states, unused queues, unused
capabilities and payload bytes beyond `len` continue to participate.
Compiler padding was not part of this typed stream and is still not read.
The fixed stream contains **74,575 words**. The historical contract incorrectly
summed this formula to 74,669; this corrects that documentation error without
removing any field or operation from the stream:

```text
1 + 16*(2 + 256*4) + 1024*12 + 64*(8 + 16*44)
  + 4*9 + 8*7 + 16*9 + 11*6 = 74575
```

No cache, dirty flag, partial hash, live-field selection, expansion of rights,
change in error order, ABI change or removed syscall check. `stats()` continues
to use the same complete digest. All kernel checks before and after every
IPC operation remain mandatory, including when the operation returns EAGAIN.
The digest is diagnostic, not a cryptographic hash, authentication or proof
of collision freedom.

## Actual measurement and limited inference

The integrator actually executed `native-cap-digest-bench` on commit
`48b51878c03b5b73449f563735c7854b24926fc4`, source fingerprint v3
`d54262733b9e2aca9b6ec1385d2c4ba7f0087d91cabab96d3f2a722edffca60d`.
Profile: pc-q35-8.2, qemu64, one CPU, single-thread TCG, 256 MiB RAM,
QEMU 10.0.13 (Debian 1:10.0.13+ds-0+deb13u1). The actual four-process
manifest graph contained 21 handles, 29 nodes and 9 endpoints. User apps
and the PIT timer did not run during measurement. All three variants
received the same Engine and 128 measured calls, divided into eight rotated
rounds of sixteen calls, with four warm-up calls per variant.

| Variant | Sum of eight TSC batches | Smallest batch | Largest batch |
| --- | ---: | ---: | ---: |
| Original reference | 36,998,312 | 4,203,914 | 5,101,902 |
| Directly typed message words | 43,931,758 | 5,070,624 | 6,485,388 |
| Zero-run composition | 77,735,944 | 8,745,048 | 10,404,420 |

All hash comparisons and complete before/after Stats comparisons passed.
The digest was `0x31b7de1c7a15245b` every time. Raw QEMU exit was 33 without
timeout; total runner duration was 3.68513638 seconds. The disposable disk
had the same hash before and after. This report was directly inspected:

```text
/workspace/nieuw-os-worktrees/root-materiaal/build/cap-bench/cap-bench-report.json
SHA256 8b3619dd4c33c652eb59ef338b68bd64b4f80984acf05d36c82dcbf7d0ad6b3f
kernel SHA256 27c787f021e1e81050b80821907efc2b9fbe7bbf3545caa91961b59fba71eca2
serial SHA256 d6a61a5c557d1e78614d373fba4773a0fcf84d828edd4348cbdc9dfcd56e7145
```

The current H05 replay based on
`1864c283a8fe0e732044e361981b9c01d342e00b` exercised the newer four-process
manifest with 25 handles, 35 nodes and 11 endpoints. All three implementations
produced `0x77cd1386b23a996b`; raw QEMU exit was 33, guest duration was
4.490098334 seconds and the disposable disk hash remained
`49c40c316234e2e4e4c8dd681a057b1b36611648896dcacba1794263b711c266`.
The result is bound to source fingerprint v3
`2859165807d303b8dbe2f977899b96700c7eccbc8bc2348f055fe5aed8d37394`
and stored locally in `build/pr218-final-cap-bench/cap-bench-report.json`.
It does not replace physical-hardware evidence.

These **unserialised RDTSC** differences are coarse TCG batch diagnostics,
including loop, function-pointer, black_box and parity-calculation work.
They do not measure physical CPU cycles or exact per-call latency. The
original calculation was the fastest of these three variants in this one
bounded test. No universal ranking for physical hardware, KVM, other CPUs
or other Engine states follows. Nor is this GUI, app, disk, entropy,
boot-latency, security or release acceptance.

The earlier host ratio of approximately 2.48 for the zero-run variant did
not predict this result. The integrated GUI run on
`1fd879dfdde88890a68f0a23901c69acd464b582` failed with raw exit 35 at the guest
deadline: one complete frame and 1,932 tiles. According to the integrator,
the first capture with 1,608 tiles appeared after approximately 170.07 seconds,
compared with approximately 106 seconds previously. The exact failed run and
its limitations are recorded in the historical contract. No native optimisation
gain is claimed from that host measurement.

## Diagnostic alternatives remain verifiable

The opt-in crate feature `diagnostic-bench` retains:

- `diagnostic_reference_digest()`: the literally preserved old reference.
- `diagnostic_direct_digest()`: all typed words, without zero detection.
- `diagnostic_zero_digest()`: the complete earlier zero-run calculation.

The zero-run calculation was moved unchanged out of the production method.
All five existing differential test groups now also compare this explicit
zero-run variant with the reference and production hash. Wire-bit tests
additionally check the direct message calculation. Private-corruption tests
continue to examine inactive, dead and invalid indices, nonzero trailing
bytes and unused capabilities. The manifest test compares the complete
Engine and Stats before and after.

Without the feature and outside `cfg(test)`, the reference, direct and
zero-run alternatives are not built. Symbol inspection checks this in the
production rlib. The explicitly started historical host microbenchmark
still compares reference with zero-run through `diagnostic_zero_digest()`;
it does not become a meaningless reference-versus-reference comparison
because the production choice changes.

The integrator must point the ZERO function pointer in the existing native
measurement harness to `Engine::diagnostic_zero_digest()`. After this
production change, the name ZERO must no longer refer to `Engine::state_digest`.
This component commit changes no kernel hook, module wiring, runner or GUI
observer; those files remain owned by the integrator.

## Component checks and resumption

The production `state_digest` body is compared literally with the source
from 3f20; SHA256 of the identical body is
`29ca84c1172b4c8dffd6f4aa36270ce23fa349075963d287fd0c9b08628cb7ed`.
The `stats()` body remains identical to 952e9ace, as does the relocated
zero-run body. This guards the exact stream; tests alone do not prove
collision freedom or complete security.

Dedicated logs are under `build/cap-reference-checks/`. Reproducible checks
using Rust 1.90.0, offline/locked, with a dedicated target directory:

```text
cargo +1.90.0 fmt -p capability-core -- --check
cargo +1.90.0 test --offline --locked -p capability-core
cargo +1.90.0 test --offline --locked -p capability-core --features diagnostic-bench
cargo +1.90.0 clippy --offline --locked -p capability-core --all-targets -- -D warnings
cargo +1.90.0 clippy --offline --locked -p capability-core --features diagnostic-bench --all-targets -- -D warnings
cargo +1.90.0 build --offline --locked --release -p capability-core --target x86_64-unknown-none
cargo +1.90.0 build --offline --locked --release -p capability-core --features diagnostic-bench --target x86_64-unknown-none
```

Both scalar-none configurations also receive Clippy with `-D warnings`.
Scalar flags are code-model=kernel, relocation-model=static, no-redzone,
SSE/SSE2/SSE3/SSSE3/SSE4.1/SSE4.2/AVX/AVX2 disabled and explicit path remapping.
All 47 original core tests remain part of both test sets.
Executed on the frozen component source: **52 standard tests and 53 tests with
diagnostic-bench passed**, each with zero failures and one explicit host
microbenchmark ignored. All nine fmt, test, Clippy and build checks described
above had exit status 0. The production rlib contained no DiagnosticHasher,
reference, direct or zero-alternative symbols.

```text
componentbron v3 ea2db2786e74508e84e42d7166194869ce550ee01243a1e4b564b92112e9801d
lib.rs SHA256 932f48e14bfb9cc545064a5d5d6dd23e7b87f500c58f90d65745dfe11f391da5
default none-rlib SHA256 cfdc9f5aa6db892f6726973f281f97e8e5413741c00f3e332d79daac7408104e
diagnostic none-rlib SHA256 2cbe91f0bee7ba2da82fab4bf1218d9c608369c4cf15b753ed3c4efa5f496bca
build/cap-reference-checks/report.json SHA256 e050cc6b3b2735bebb1745a5179c8f4058f80abf38258abc0e2e7e97f00b7015
```

The fingerprint concerns this dedicated component worktree, not root's next
integrated candidate. A read-only source review by a second agent confirmed
the literal bodies and cfg isolation without a concrete blocker; this is
neither an independent human audit nor a native test.

The integrated native GUI and relevant regressions must be rerun on the
new exact commit and image. This production choice signs off no GUI criterion
and does not increase the release percentage.

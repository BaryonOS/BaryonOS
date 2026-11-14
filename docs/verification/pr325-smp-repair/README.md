# PR #325: legacy SMP scheduling and conflict repair

Date: 14 November 2026. Incoming PR head: `0d47a6ee29f60270936331111ef542c2a038a36f`.
Integrated main: `2eb7b942cac10db727a184819812634300a8e778`.

## Observed failure and diagnosis

GitHub Actions run [43594595263](https://github.com/BaryonOS/BaryonOS/actions/runs/43594595263),
job [132446375822](https://github.com/BaryonOS/BaryonOS/actions/runs/43594595263/job/132446375822),
failed the final `m4-revoke-two` stage with raw QEMU exit 35 and
`M4:FAIL:USER_FIXTURE` after 14.203 seconds. It did not time out.
The serial prefix contains one successful send, one revoke and then a rejected
send by process 0. The original result, serial output and artifact receipt are
preserved byte-for-byte as text with SHA-256 digests in [evidence.json](evidence.json).

The legacy M3 scheduler used one scan cursor for all CPUs. In the M4 fixture,
process 1 is pinned to CPU 1; processes 0 and 2 are pinned to CPU 0. Every
selection of process 1 set the shared cursor to 2. In the allowed repeated
interleaving CPU 1 selects process 1, then CPU 0 selects a process, CPU 0
always selected process 2. Process 0 could remain ready indefinitely.
This is a deterministic counterexample derived from the actual selector,
not a separately executed host test.

That starvation can prevent process 0's second send, exhaust process 1's
32,768-attempt wait bound, and cause process 1's cleanup to revoke the send
capability held by process 0. The resulting rejected send matches the observed
prefix. However, the original serial output omits the first unexpected exit's
row and attempt count, so this precise causal sequence in the original CI run
is not established. An unchanged local baseline passed once on QEMU 10.0.13;
the failed CI run used QEMU 8.2.2.

## Change and preserved contracts

Only five cursor uses in `kernel/src/m2_legacy/m3.rs` change: the field becomes
an array per CPU, both initialization paths reset the complete array, and
selection reads and advances the calling CPU's cursor. A pinned peer can no
longer overwrite another CPU's scan position. Ready checks, affinity,
return-layout validation, process cleanup and the assertion against simultaneous
execution on two CPUs remain unchanged. No retry bound, timeout, capability
semantics, workflow or failure oracle is weakened. The single-CPU selector is
unchanged. Two read-only AI reviews found no blocking issue; these are not an
independent human audit.

The two conflicts in `docs/PROJECT-STATE.md` and `docs/progress.json` retain the
complete S09 handoff, D6 approval and pending conditions, together with the
current main checkpoints for merge hold and coverage evidence. All original
PR files outside those two documents are preserved. The main refresh during
verification imported PR #302 unchanged. The criterion register remains 41/100.

## Executed verification

| Check | Result |
| --- | --- |
| Targeted `native-smp-m4-revoke` build and QEMU run | Passed; raw exit 33 |
| `python3 -B -u tools/test_smp.py`, final integrated source | All 15 native profiles and 23 host tests passed |
| `python3 -B tools/preflight.py --strict --firmware "$OS_OVMF_CODE"` | Exit 0 |
| `python3 -B tools/build.py` | Exit 0 |
| `python3 -B tools/run.py --smoke` | Exit 0 |
| `cargo +1.90.0 test --locked -p capability-core -p kernel-core -p volume-core` | 223 passed; one pre-existing ignored test |
| Volume Python harness | 48 passed |
| IPC artifact checks against the built ELF | 13 passed |
| `cargo +1.90.0 fmt --all -- --check` | Exit 0 |

The final native matrix and standard gate ran after integrating main `2eb7b942`.
Earlier host tests' Rust and test inputs are unchanged by that refresh. The
initial IPC artifact invocation was blocked before tests because the ELF did
not exist; the later invocation against the actual built ELF passed all 13
tests. The earlier blocked invocation is retained in the evidence.

Local native environment: Rust 1.90.0, QEMU 10.0.13, `pc-q35-8.2`, TCG,
qemu64 CPU, 256 MiB and OVMF 4M. This is not the identical QEMU build used in CI.
Final source-v3 fingerprint:
`4ed35695957d83b8c3bef5194888f6f4c901fdf2670695fb59a120367423bb13`.
The [generated matrix receipt](../sm-smp/smp-rendezvous-20261114T130006Z.json) and the evidence
bundle preserve each native result and serial transcript. Recorded local
`source_commit` fields identify the checkout parent during integration; the
fingerprint binds the tested source including the uncommitted integration.

These checks establish the bounded experimental scheduler behavior exercised
by the existing profiles. They add no OS acceptance criterion. GitHub Actions
on the published head remain the CI confirmation; this local report does not
claim an unobserved remote pass. S09 remains open and the PR is not merged.

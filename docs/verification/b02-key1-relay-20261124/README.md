# B02-T4 native KEY1 key-state commit relay evidence (24 November 2026)

Task: B02-T4, native KEY1 key-state commit ingress under
ADR-B02-KEY1-RELAY-20261123 (TEST profile only). This directory holds the
native runs and host checks at one frozen source. Volume images and binaries
are not copied. `SHA256SUMS` covers every other file here. Design, pinned
values and their derivation: `docs/implementation/B02-SECURE-RECORDS.md`,
section FR-B02R.

## Source and environment

| Item | Value |
|---|---|
| Source commit (recorded by every `result.json`) | `d4db823e6d80afee937f5d5af71cef96dfe098ee` (branch `claude/b02-key1-relay-native-20261124`, base `006fb93c`) |
| Source fingerprint (`source_sha256`, v3) | `e6d90fbf49e2…` (full value in each `result.json`) |
| Rust toolchain | `1.100.0` (`rust-toolchain.toml`; `tools/build.py` invokes `cargo +1.100.0 build --locked`) |
| QEMU | 10.2.1 (Debian 1:10.2.1+ds-1ubuntu3.3), `-machine pc-q35-8.2`, `-accel tcg,thread=single`, OVMF |
| Isolation | `NOS_BUILD_DIR=<worktree>/build/relay-20261124` (inside the checkout because `run_system` requires that), `NOS_TARGET_DIR`/`CARGO_TARGET_DIR` in a scratch directory outside the worktree; one QEMU guest at a time |

## Re-run after review follow-up (49b1557d)

Review commit `49b1557d` widened the answered-commit latency window to
`0..1499` ticks in `kernel/src/m2/m3/system.rs` and `tools/run_system.py` (a
fast KVM host can finish every commit within one tick). The full gate
`python3 -B -u tools/test_system.py` was re-run at
`49b1557d2f5c068b55b65a24dc15473f4c69d028` (fingerprint `b435ddc59b26…`): exit 0.
`0-relay`, `1-relay-not-committed`, `2-relay-timeout`, `3-red-relay-leak` and
`5-gate/system-gate.json` are from that run, with the same measured values
(answered commits max 5 ticks, timeout 1500, leak raw 35 with the disk
unchanged, wrong-case control 6/6 rejected). `tests/test_system_harness` (59
tests) and root/kernel `fmt --check` were re-run at that commit (exit 0).
`4-control-wrong-case` (12/12 including relabelled logs), `6-b01`,
`7-regression-normal` and `host/` remain from `d4db823e`; the follow-up does
not change the keyring ELF or any host crate.

## Commands and raw exit codes

`steps.tsv` lists every step with its exit code and duration. Native steps, in order:

| Step | Command | Exit |
|---|---|---|
| plain normal regression | `python3 -B tools/build.py --kernel-feature native-system`; `python3 -B -u tools/run_system.py --case normal` | 0; 0 (QEMU raw 33) |
| normal commit | `… --kernel-feature test-system-keystate-relay`; `run_system.py --case relay` | 0; 0 (raw 33) |
| `NOT_COMMITTED` → `NotReady` | `… test-system-keystate-relay-not-committed`; `--case relay-not-committed` | 0; 0 (raw 33) |
| timeout latches `Degraded` | `… test-system-keystate-relay-timeout`; `--case relay-timeout` | 0; 0 (raw 33) |
| app with commit SEND (red) | `… test-system-keystate-relay-leak`; `--case relay-leak` | 0; 0 (red case expected; raw 35) |
| FR-I01 gate incl. relay stages, red case and wrong-case control | `python3 -B -u tools/test_system.py` | 0 |
| B01 regression | `python3 -B -u tools/test_keyring.py` | 0 (7 stages) |

Host checks (`host/host-*.txt`):

| Command | Exit |
|---|---|
| `cargo +1.100.0 fmt --all -- --check`; same with `--manifest-path sdk/Cargo.toml` | 0; 0 |
| `cargo +1.100.0 test --locked -p keyring-core` (unit tests incl. 7 relay tests; every `services/keyring/tests/*.rs` suite unchanged) | 0 |
| `cargo +1.100.0 clippy --locked -p keyring-core --all-targets -- -D warnings` | 0 |
| native clippy of `nieuw-os-keyring-example` and `nieuw-os-system-example` (`--features …/native --target x86_64-unknown-none -D warnings`, soft crypto backend cfgs) | 0 |
| `python3 -B -m unittest tests.test_system_harness` (59 tests) | 0 |
| `python3 -B -m unittest discover -s tools/tests -p 'test_*.py'` with `GIT_EDITOR` unset (990 tests, 15 skipped) | 0 |
| `python3 -m compileall -q tools tests` | 0 |
| `python3 -B tools/agent_gate.py --job native-system.yml#native-system --list` and `--job ci.yml#host-quality --list` | 1 and 1 (see NOT RUN) |

Because the `agent_gate.py` prefilter could not resolve either job, the `run:`
steps of both jobs were enumerated from the workflow files and the applicable
ones were run individually at the same source (`steps.tsv`, `host/`):

| Command (from the job) | Exit |
|---|---|
| `python3 -B tools/preflight.py --strict` | 0 |
| `python3 -B -u tools/test_system.py` immediately followed by `python3 -B tools/capture_system.py --destination docs/verification/full-release-<unique>` (capture revalidates the gate JSON, which now carries `relay_stages`, `relay_red` and `relay_wrong_case`; the output directory was moved out of the tree) | 0; 0 |
| `cargo +1.100.0 test --locked` and `clippy … --all-targets -D warnings` for the 19 root packages of `host-quality` | 0; 0 |
| `cargo +1.100.0 fmt --all -- --check` (the kernel is a root workspace member) and `fmt --manifest-path kernel/Cargo.toml` | 0; 0 |
| `cargo +1.100.0 test --manifest-path sdk/Cargo.toml --locked`; `clippy --manifest-path sdk/Cargo.toml --locked --all-targets -- -D warnings`; `test --manifest-path sdk/keyring-client/Cargo.toml` | 0; 0; 0 |
| `tests/` patterns of `host-quality`: `test_ci_evidence*`, `test_ci_workflows`, `test_security_alerts_summary`, `test_tokenguard*`, `test_document_session_*`, `test_volume_*`, `test_system_*`, `test_common`, `test_preflight`, `test_build_paths`, `test_guest_harness`, `test_m1_artifact_binding`, `test_m6c_*` (one run per pattern) | 0 each |
| `python3 tools/progress.py --check` | 0 |

An unfiltered `python3 -B -m unittest discover -s tests` aborts on a module-name
collision with `tools/test_policy_envelopes.py`; it is not part of any job and
is not reported as run.

### Startup-word isolation

The relay paths run only when startup words that only the relay kernel writes
are nonzero: Key Service words 496/504/512, init word 208 and app word 752.
`grep -rnE 'put\w*\([^)]*\b(496|504|512|208|752)\b' kernel/src/m2 kernel/src/m2_legacy`
outside `m3/system.rs` finds three writes, none into these ELFs:
`keyring.rs` writes 512 into slot 3 (`KEYRING_APP_ELF`), `volume.rs` writes 504
into slot 1 (`VOLUME_APP_ELF`) and `m5.rs` writes 208 into an M5 process. The
other profiles that load the Key Service ELF (`desktop.rs`,
`window_session.rs`) write only words 32–176 into it, and `nas.rs` writes only
words 32–184 and the replay range 256–335 into its processes. Inside `system.rs` every relay write is under
`cfg(feature = "test-system-keystate-relay")`.

With the session's `GIT_EDITOR=true` set, `tools/tests` reports 2 failures and
9 errors in git-driven pipeline tests; with it unset all pass. This is an
environment effect, unrelated to this change (`host/host-tools-tests.txt` is
the clean run).

## Measured values

| Case | Requests | Ok | NOT_COMMITTED | Unknown | App statuses | Max commit ticks | Disk |
|---|---|---|---|---|---|---|---|
| `relay` (`0-relay`) | 3 | 3 | 0 | 0 | 0, 0 | 5 | object 8 rev 3, versions [3, 2, 1]; volume gen 5, 5 chunks, clocks [2, 3] |
| `relay-not-committed` (`1-…`) | 5 | 3 | 2 | 0 | 1, 1 (`NotReady`) | 5 | identical shape; the refused commits wrote nothing |
| `relay-timeout` (`2-…`) | 4 | 3 | 0 | 1 | 7, 7 (`Degraded`) | 1500 | identical shape |
| `relay-leak` (`3-red-relay-leak`) | — | — | — | — | — | — | `SYSTEM:FAIL:APP_COMMIT_HANDLE` only, no `MANIFEST:PASS`, raw 35, image unchanged (`before_sha256 == after_sha256`) |

All three passing cases also show `SECRET_SUPPLIED 1`, `SECRET_REFUSED 1`,
`LAST_GENERATION 3`, init `COMMITTED 3`, `BEGUN 3`, `REVISION 3`,
`GENERATION 3`, and `SUM` equal to the FNV checksum of the current object 8
version on disk (random nonces make `SUM` differ per run). Run durations:
4.6 s, 4.6 s, 19.6 s and 1.8 s; the `test_system.py` relay stages gave the same
counters (max ticks 5, 5, 1500). The plain normal profile still passes with its
unchanged oracle (`7-regression-normal`: object 8 revision 1, volume gen 3).

## Wrong-case control

`4-control-wrong-case/cross-evaluation.json` (from `5-gate/system-gate.json`):
every relay log is evaluated against each other relay case, twice. As
recorded, each is rejected by its case marker. Relabelled, with the case marker
and `KEYSTATE_RELAY_CASE` rewritten to the claimed case, each is still rejected
by the counters (`KEYSTATE_RELAY_REQUESTS: expected 5, found 3`, …). 12
rejections, 0 acceptances.

## NOT RUN

- `tools/agent_gate.py` local prefilter for `native-system.yml#native-system`
  and `ci.yml#host-quality`: the tool exits 1 at plan resolution
  ("uses step needs an explicit local adapter" for the pinned
  `actions/checkout`), before any step runs. The applicable commands were run
  individually above instead; this is not hosted acceptance.
- Hosted CI (no push in this task). The `agent_gate.py` resolution failure
  exists at base `006fb93c` too (this change does not touch `.github/**` or
  `tools/agent_gate.py`); a local adapter needs a lead integration request.
- Native steps of `native-system.yml` other than `test_system.py` and
  `capture_system.py` (IPC wait, TPM, system clock, network, net-link, FPU, TCP,
  memory), and the other profiles that embed the changed ELFs (`run_random`,
  `run_images`, Materiaal, NAS, desktop, window session, cap-bench). The relay
  paths are gated on startup words that no other profile writes (see above);
  this is code reading, not a run.
- `host-quality` steps unrelated to the changed files: actionlint download,
  coverage tooling, quantum, Node prototype and NAS tests, the other SDK
  manifests, M6c build, clean-tree check.
- Physical hardware; human audit.

## Limits

- TEST profile only; the secret is the compile-time test constant in init.
- No cold restart in the relay profiles: rollback anchoring and the trusted
  generation floor stay with B02-T3, and the RESTORE custody path for a held
  secret is not exercised natively.
- One writer, one instance, one volume identity (`NOS-SYSTEM-VOL01`).
- QEMU TCG timing; tick values are not real-time guarantees.
- B02 stays open; this evidence grants no progress credit by itself.

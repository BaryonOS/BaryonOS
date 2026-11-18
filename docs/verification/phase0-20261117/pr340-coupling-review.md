# P0-340-THREADS — bounded read-only review

**Decision:** all four coupling advisories are nonblocking at `90816de39460b76e35cd626721d4dac9bde905e2`. No concrete maintainability or security defect was found in these four symbols that justifies blocking integration. Refactoring solely to reduce the reported metrics is not recommended.

**Scope and authority:** `/tmp/baryonos-phase0-20261117/pr340`, branch `feat/s09-ustar-name-check`; the initial and final working tree were clean. This report is the only write. No tracked files, ownership records, comments, threads, runners, Git refs or remote state were changed. Broader S09 ownership and D1–D5 are outside this review.

**Method:** root, services and docs instructions, Ponytail, ECC verification and OSDev guidance were read. Hosted Graphify recall, rank, find, callers and tests queries succeeded, but the index was `b718f6bafdfc4927d3b7acfe786e67dcb8f88eec`, not the PR head. In particular, it did not contain the two new negative tests. Bounded reads of the actual PR definitions and current callers therefore supplied the review evidence; no local graph was created.

## Advisory decisions

| Location at the reviewed head | Advisory | Decision and concrete rationale |
|---|---|---|
| `services/volume/src/ustar.rs:119`, `from_archive` | fan-out 7 | **Nonblocking.** This short orchestration function connects the checks needed for one export. The public object count is bounded before slicing (125–127); `judge` recalculates reasons instead of trusting caller-controlled bits (128–132); each parsed entry and the collection names are compared before its first write (134–143); the final object count and full archive walk must succeed before the two terminator blocks (163–167). The helper calls implement these responsibilities without granting authority or duplicating validation. Every sink failure propagates. Splitting the function merely to lower fan-out would obscure the write order. |
| `services/volume/tests/archive.rs:995`, helper `ustar` | Ca·Ce 15 | **Nonblocking.** The seven-line test helper deliberately centralises the documented two-pass sequence. Both readers borrow the same immutable input slice, and `and_then` prevents conversion after a failed check. It returns the actual `Result` and output bytes for callers to assert, rather than manufacturing success. Separate negative tests also call `from_archive` directly with modified `Names`, so this convenience helper does not hide the second-pass boundary. Reuse by multiple tests is not a production dependency problem. |
| `services/volume/tests/archive.rs:1296`, `ustar_export_refuses_the_pax_entry_name_at_the_root` | fan-out 7 | **Nonblocking.** The test uses normal archive-building helpers to exercise a real overwrite/collision condition: root `PaxHeader`, case-folded root `paxheader`, and a used collection `PAXHEADER`. Its local counting sink verifies **zero calls to `put`**, including after refusal bits are cleared (1305–1328). A positive case accepts `Docs/PaxHeader` (1311–1315). The calls support explicit fixture setup, the positional rule and the no-write invariant. The production guard is visible at `ustar.rs:286–298`; the metric identifies no omitted validation. |
| `services/volume/tests/archive.rs:1500`, `ustar_export_judges_names_again_and_refuses_another_source` | fan-out 6 | **Nonblocking.** The direct converter closure avoids relying on the common two-pass helper. It exercises cleared single-name and pairwise refusal bits, an oversized object count, renamed collection/object, and fewer/more expected objects (1507–1544). These are distinct counterexamples to trusting public `Names`. Later mismatches assert the error status without demanding an empty buffer, consistent with the documented partial-output contract. Keeping these boundary cases together makes the invariant easier to inspect. |

## Security checks and evidence limits

The reviewed counterexamples were caller-edited `Names`, an excessive count, substituted slots/names/collections, truncated/corrupt input, and errors after earlier writes. Static source inspection shows the expected refusal paths. The supporting parser bounds header counts, collection indices, names and content lengths before slicing or calling the exporter (`archive.rs:278–363`). `In::exact` rejects source-reported lengths above the provided buffer (`archive.rs:232–243`); end-of-stream length is also checked (`372–377`). The exporter uses borrowed Rust slices and has no `unsafe`, raw-pointer, IPC, DMA, capability, updater or key-handling operation. Those other subsystems were not independently audited.

The current non-test-file caller is the `ARCHIVE_USTAR` test case at `sdk/examples/volume-docs/src/app.rs:1016–1058`: it checks the archive summary, exercises a caller-edited reserved name, requires the real conversion to return `Ok(summary)`, and only then finishes the sink. This bounded inspection found no new production authority or capability path.

Two limits must remain explicit:

- `Names` binds the slots, names, collections and object count. It does **not** bind the complete first-pass content or metadata, and this review does not claim byte-identical source pinning. The second archive is independently validated. That matches `S09-BACKUP-ARCHIVE.md:361–370`.
- A second-pass error can leave valid-looking partial tar bytes. `services/volume/tests/archive.rs:1548–1563` demonstrates that Python accepts a stream without its two terminator blocks; callers must discard output on every error. The API and specification explicitly retain this requirement. Future production integration must preserve it.

The root-position PaxHeader case is not representable by the single-name shared fixture alone, because it depends on position and collection use. The explicit archive test supplies that evidence. No pax-unaware reader, Windows/macOS extraction, physical hardware or independent human audit was run. Existing platform limits such as 8.3 aliases and destination path length remain documented. Absence of a finding in this bounded AI review is not proof of security or full S09 acceptance.

## Executed checks

| Command or query | Result |
|---|---|
| `rtk git status --short --branch` (initial and final) | exit 0; clean `feat/s09-ustar-name-check` |
| `rtk git rev-parse HEAD` | exit 0; exact head above |
| `rtk git remote get-url origin` | exit 0; `https://github.com/BaryonOS/BaryonOS.git` |
| Hosted Graphify `recall`, `graphify_rank_files`, three `graphify_find` queries, `graphify_callers`, `graphify_tests_for` | successful tool results; index mismatch recorded above; empty/partial PR coverage was not treated as absence |
| `rtk git diff b718f6bafdfc4927d3b7acfe786e67dcb8f88eec HEAD -- services/volume/src/ustar.rs services/volume/tests/archive.rs` | exit 0; filtered diff used for orientation, complete relevant current bodies read separately |
| `rg -n 'from_archive\|ustar::check' --glob '*.rs' --glob '!build/**' --glob '!target/**' .` | exit 0; current call sites identified |
| `rg -n 'unsafe\|capabilit\|secret\|key\|DMA\|dma\|syscall\|ipc\|IPC' services/volume/src/ustar.rs` | exit 1; no matches; source bodies were also inspected |
| Bounded `nl -ba ... \| sed -n ...` source reads | exit 0; `ustar.rs:1–400`, test ranges `30–145`, `975–1045`, `1280–1390`, `1430–1570`, parser `24–82`, `208–247`, `245–430`, native caller `990–1080` |
| `sha256sum build/tokenguard/run-j2nmuqcn.log services/volume/src/ustar.rs services/volume/tests/archive.rs` | exit 0; raw-log SHA matches the native receipt |
| `rg -n 'Ran [0-9]+ tests\|test result:\|Clippy\|clippy\|stage\|passed\|PASS' build/tokenguard/run-j2nmuqcn.log` | exit 0; all 15 native stages report passed, final `M6e gate passed=True stages=15` |

Three navigation attempts returned exit 2: a nonrecursive `rtk grep` with the nonexistent `services/volume-app` path, an `rg` command including nonexistent `kernel/src/volume.rs`, and a historical-evidence lookup including nonexistent `USTAR-NAMES-20261115.md`. Corrected bounded source searches resolved the relevant caller and definitions; these were navigation errors, not test failures. Searches for direct converter calls under `kernel` and nested SDK `AGENTS.md` returned exit 1 (no matches).

Observed SHA-256 values:

- `services/volume/src/ustar.rs`: `dd3ae91021369b6dd334f8ce016d877fece681ccff5ce807dfc4021d9db11716`
- `services/volume/tests/archive.rs`: `15919204d5c4aee041a1e0b898520473e1ef66c6ead4d6d12341cdd510aa511a`
- Native raw log: `b11e49562231ecbb135d7cd9ddcb9a683b5db8a35eaf825d4c652b29b8918407`

The existing source-bound receipt `/tmp/baryonos-phase0-20261117/repo/docs/verification/phase0-20261117/pr340-native-receipt.json` records this exact source head, `PATH=/snap/rustup/current/bin:$PATH CARGO_BUILD_JOBS=2 python3 -B tools/test_volume.py`, exit 0, 151.164 seconds, QEMU 10.2.1, and 15 passed stages. Its raw log is `/tmp/baryonos-phase0-20261117/pr340/build/tokenguard/run-j2nmuqcn.log`. Earlier host evidence reports 93 Rust and 53 Python tests and the corrected exact freestanding Clippy invocation; those suites were **NOT RUN** by this reviewer, as instructed. No new concern justified repeating the passed suites.

**Remediation direction:** preserve the existing validations, failure propagation and explicit boundary tests; no metric-driven refactor is required. **Blocker:** none from these four advisories. **Next step:** the lead can use these nonblocking decisions in its PR340 integration assessment; this report neither merges the PR nor closes review threads.

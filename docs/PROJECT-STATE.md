# Current project status

## NEXT AGENT HANDOFF — PR #210 Veld review repairs — 10 November 2026

**Source:** the local candidate is based directly on current `main`,
`342194f215297c52a5578c7d1b7f18492150e61d`, after the #257, #297, #300
and #259 squash merges. Its replayed implementation commit is
`7c97ce9e2237f1ddbf83827627ec97968d6913cb` (tree
`809ff681e2203c34edd1d02fe4fb8793091be42e`). Nothing from this repair round
has been published or treated as hosted evidence.

**Review repairs:** the two native matrix profiles now keep distinct evidence
artifacts and only the 100% profile requires the Windows bundle that it builds.
The 200% result field can be accepted only by the complete workspace route,
never by the switcher part gate. The compositor activity sample covers each of
its ten counters exactly once. INP1 documents payload-free `PointerLeave`; the
measured Materiaal RX code and budget are recorded as 157,161 bytes in 39 pages
under the fixed 160-KiB cap, with five LOAD segments and no W+X.

**Local evidence:** 27 runner tests, 22 workflow-contract tests and all 110
window-session Rust tests passed. Python compilation, warning-free formatting,
strict preflight, the progress consistency check and `git diff --check` passed.
The default build and TCG smoke passed (`M1:PASS`, raw exit 33), and both
`native-window-session-probe` and `native-window-session-hidpi-probe` built and
were inspected successfully. Independent read-only review found no remaining
code, evidence or documentation blocker in the repair set.

**Acceptance:** **39/100 → 39/100**. The prior bounded local 100% and 200%
runtime attempts timed out and are not positive evidence. The final published
head still requires both complete hosted 1800-second native matrix gates.

**Next:** finish this replay on the actual new `main`, publish with an exact
lease, and evaluate only current-head hosted checks and review. Squash-merge
with the expected head only after both native profiles are green.

## Historical checkpoint — PR #299 ChaCha20 production unification — 10 November 2026

**Source:** this local candidate is based directly on the actual PR #297
squash `2d17a3870c0f22254af0a749d3f29128fe2e11ae` (tree
`563b3ec57355ca124672afe94de79808f9e4fc70`), which follows the PR #257
squash. Its semantic donor is local commit
`5ba29a843d7087ff637fb2856677cfd17da6f74a` on reviewed #297 candidate
`3cb1e16d0f5f1a91784d0761ec0b5c5d7f60606f`. This replay has not been
published or treated as hosted evidence.

**Implementation:** production now resolves one pinned `chacha20=0.10.2` for
the DRBG, maintained RNG and AEAD closure. The direct pinned `cipher=0.5.2`
dependency enables its `zeroize` feature so the stream wrapper and buffered
keystream implement `ZeroizeOnDrop`; compile-time assertions retain this
boundary. `chacha20=0.9.1` remains only as the independent dev-test alias
`chacha20-legacy` for the byte-exact old DRBG oracle. Product builders use the
single portable `chacha20_backend="soft"` selector.

**Compatibility and provenance:** the new implementation reproduces the old
DRBG bytes and cold-restores a fixed pre-migration persisted blob, including
decryption of its existing record key. All four locks and crate checksums are
contract-tested; the three consumer product locks contain 0.10.2 but not
0.9.1. The reviewed upstream crates are `no_std`, require Rust 1.85, use MIT
or Apache-2.0 and have no build script.

**Local evidence:** all 71 keyring tests and four crypto-contract tests passed.
Warning-denying Clippy, formatting, strict preflight, progress consistency and
`git diff --check` passed. Locked offline fetches, consumer tests and Clippy,
and the keyring, SDK-client, example, system and random-probe freestanding
release builds passed. Independent read-only review found no security,
compatibility, lockgraph or provenance blocker.

**Acceptance:** **39/100 → 39/100**. No native B01, RANDOM, FR-I01 or B03 gate,
QEMU run, physical entropy result or new product criterion is claimed.

**Next:** finish this replay on the actual #297 squash, publish with an exact
lease and evaluate only current-head hosted checks and review. Squash-merge
with the expected head only after they are green.

## Historical checkpoint — PR #257 fail-closed publication inventory — 10 November 2026

**Source:** exact reviewed head `590d9c3b589435fff6bf794c85908bbb37a98382`
was squash-merged as `1e3062fc1258ba6d098e70bcb13d64de1fbd90a2`;
the trees are byte-identical at `17696fb42e386366db39257517908acf80ae03fc`.
The audit remains a local evidence tool and never grants publication clearance
by itself.

**Implementation:** the three-phase scanner binds the exact advertised Git
tips, Git history and metadata, Actions inventory, scanner identity, retained
bytes and resource receipts to one immutable snapshot. Git replacement refs
are disabled, public ref discovery receives no inherited credentials, and the
private route exposes its token only to a transient askpass process. Retained
state uses reconciled O(1) accounting with bounded atomic replacements. A
final audit-success record is now invalidated before every retry and is written
only after the command deadline and persistent usage ledger have closed
successfully; a failing retry cannot leave an earlier success active. The
canonical repository binding is now `BaryonOS/BaryonOS` at immutable repository
ID `1435549142`; both historical owner/name routes are accepted only through a
GitHub redirect to that exact ID and current metadata.

**Validation:** all 86 publication-audit unit tests pass, including real Git
replacement-ref isolation, exact advertised-HEAD binding, retained-file
tampering, repository-transfer binding, deadline and state-quota failures,
failed re-finalization and the ordered success commit. The live public API
confirms the canonical name, clone URL and unchanged repository ID; both old
routes return the expected ID redirect. Python bytecode compilation, the strict
preflight and the staged-diff check pass. No live GitHub/Gitleaks inventory was
executed for this integration, so there is no claim that the repository is
clear for publication and no historical receipt is promoted to current evidence.

**Acceptance:** **39/100 → 39/100**. This strengthens audit tooling and its
fail-closed evidence boundary but completes no OS product criterion and adds
no native runtime, physical-hardware or human acceptance.

At merge time 45 hosted checks were green and the native-window job was still
in progress; that unfinished job is not positive evidence. Any real publication
audit must run against its exact advertised source tip and must keep
`publication_clearance` false.

## NEXT AGENT HANDOFF — PR #218 H05 step-2 replay after #269 — 10 November 2026

**Source:** local branch `codex/pr218-final-20261110`, based directly on
`main` `1864c283a8fe0e732044e361981b9c01d342e00b` (tree
`f5ed6ad39f32505b8215d9c65e8dc91582d56dc1`). The semantic donor is PR
#218 head `f1901125b34a2543571a47b937c992d18aeda7cc`; the bounded local port was
replayed from `ae5bf09b12db83ce198772e8f788ce83d46cec22`. The already merged PR
#213 ACPI/FADT/DSDT/S5 implementation was not duplicated. No remote branch,
review thread or pull request was mutated.

**Implementation:** the port adds the sole non-delegable POWER capability,
volume IPC v6 `SHUTDOWN`, syscall `0x90`, the final storage/keyring gate,
bounded fail-closed refusal evidence, SDK/example support and three native
POWER profiles. The former default-to-POWER selector branch now rejects every
unknown or composite selector before state changes; a full-engine atomicity
test covers `POWER|TRANSFER`, an unknown high bit and `u64::MAX`. Current
main's newer keyring manifest is retained. Its capability benchmark expectations
were corrected from 21/29/9 to the observed 25 handles, 35 nodes and 11
endpoints instead of reverting the new administrative channels.

**Integration:** #243's fail-closed `qemu_accel()` selector remains the only
way to opt into KVM; the POWER runner no longer hard-codes TCG. The evidence
verifier retains the KVM pilot and both H05 jobs, for 18 archive workflows and
20 preservation calls. The English documentation from #269 remains English.
Its progress register and generated README/SVG remain byte-unchanged at 39/100.

**Current replay validation:** capability-core passed 87 tests with one ignored
benchmark; volume-core passed all 72 tests; the SDK workspace passed 116 tests.
Warning-denying Clippy and formatting passed for the affected Rust workspaces.
The 19 system-harness, eight CI-evidence, 21 CI-workflow, ten KVM-selector,
34 system-capture and 18 cap-benchmark harness tests passed. Pinned actionlint
1.7.12, the staged-diff check and the progress consistency check also passed.
The current candidate source fingerprint is
`2859165807d303b8dbe2f977899b96700c7eccbc8bc2348f055fe5aed8d37394`.

**Native evidence:** strict preflight, the default build and the TCG smoke
passed (`M1:PASS`, raw exit 33). All three POWER profiles built with three
ELF load segments. The source-bound six-stage POWER matrix passed: normal S5
exited raw 0; flush-error and flush-timeout exited raw 33 without S5; all three
cold recoveries exited raw 33 with unchanged recovery-image hashes. Its report
is `build/pr218-final-power-gate/power-system-gate.json`. The native capability
benchmark passed raw 33 with digest `0x77cd1386b23a996b` for all variants and
an unchanged disk; its report is
`build/pr218-final-cap-bench/cap-bench-report.json`. The standalone #213 path
passed five real S5 exits and two corrupt-DSDT rejections.

The separately requested `smp-lock` suite is not green evidence. With no QEMU
or competing heavy task, `mutual_exclusion_under_contention` still consumed
approximately four CPUs for more than three minutes. The candidate suite was
interrupted with exit 130. The exact test was then repeated in a clean detached
worktree at parent `1864c283a8fe0e732044e361981b9c01d342e00b`; it reproduced
the same ten-thread, approximately 397% CPU hang and hit the bounded 180-second
timeout with exit 124. The replay changes neither `crates/smp-lock` nor
`kernel/src/m2/lock.rs`, so this is a pre-existing parent-baseline failure,
not passing evidence and not a POWER-port regression.

**Acceptance:** **39/100 → 39/100**. This is QEMU/TCG and host evidence for a
bounded H05 substep. It does not establish complete coordinated production
shutdown, S3 sleep/wake, thermal/battery behavior, independent human security
review or physical-hardware support; H05 remains open.

**Next:** review the final diff and identities, create the single local noreply
commit and publish only through the parent integration flow. Current-head
hosted checks and review remain required before any guarded squash merge.

## NEXT AGENT HANDOFF — Issue #292 target-flag repair — 10 November 2026

**Source:** PR #295 local integration commit
`16d2fb3d6c3d2e66b66888fcf5577b316a9f580a`, rebased onto the #287 squash
base `e1eefaf8e0ab061c90e9fefe89ef16f80917a343`. The artifact-store workflow
uses additive Cargo configuration so the freestanding target retains its code
model, relocation model, linker arguments, and scalar CPU-feature restrictions.
`.cargo/config.toml`, kernel runtime behavior, capabilities, and dependencies
are unchanged.

**Local integration evidence:** twelve workflow contract tests, nine
artifact-store contract tests, warning-denying Clippy, formatting, the
freestanding library build, strict preflight, the standard build, QEMU smoke,
and the progress check passed on tree
`2c04a809d890e7eb8e4f6041281aeda41f9e1c7b`. The verbose compiler invocation
retained the required code model, relocation model, no-red-zone setting, four
linker arguments, and scalar target-feature restrictions. Source hashes,
commands, log hashes, and evidence limits are recorded in
[`verification/pr295-main-integration-20261110.json`](verification/pr295-main-integration-20261110.json).
The original focused record remains in
[`verification/artifact-store-target-flags.md`](verification/artifact-store-target-flags.md).

**Acceptance:** **39/100 → 39/100**. This CI correction closes no OS product
criterion and is not native runtime evidence.

**Next:** publish with an expected old-head lease. Require current-head hosted
review and checks before a guarded squash. Issue #292 remains subject to the
repository's normal issue lifecycle.

## Historical checkpoint — PR #167 switcher review repair — 10 November 2026

**Proposal:** VELD-08 remains a bounded Alt+Tab subgate. It does not accept the
complete native session, recovery, accessibility, or interaction-budget criteria.

**Implementation:** PR #167 now includes current `main` `5456296` and binds each
tested key edge to an acknowledged pointer marker with an exact input-counter
delta. The native work-frame reference additionally requires `SHELL_READY` and
two consecutive byte-identical full-screen captures. No capability, kernel ABI,
dependency, storage right, or default gate was changed.

**Test evidence:** `python3 -B tools/test_window_session.py` passed 23 host tests;
`python3 tools/progress.py --check` and `git diff --check` passed. Strict preflight
reported missing Rust, QEMU, and OVMF; the build and smoke commands consequently
failed before execution. Exact source, commands, exit codes, and evidence limits
are recorded in
[`verification/pr167-switcher-review-fix-20261110.json`](verification/pr167-switcher-review-fix-20261110.json).

**Unknown:** The final PR head still requires current-head CodeRabbit/CodeQL and
required checks, including native switcher runs at 100% and 200%. Historical
native receipts do not verify this repair. **Progress remains 39/100.**

**Next:** evaluate only the final published head checks, resolve the verified
review thread, and squash-merge with an expected head SHA when all required
gates are green.

## H05 native ACPI S5 integration — PR #213 (9 November 2026)

The opt-in `native-acpi-s5` kernel profile connects the existing
`drivers/acpi-power` parser to bounded FADT/DSDT discovery and ACPI S5 port
writes in the fixed QEMU/OVMF configuration. Invalid firmware performs no PM1 control-port writes; serial diagnostics
and debug-exit I/O remain active. The change adds a reset-sensitive QEMU oracle,
five positive and two negative native runs, and the `native-power.yml`
workflow. Source-bound original evidence is retained in
[`verification/h05-acpi-s5/`](verification/h05-acpi-s5/).

CI evidence now retains 17 tracked workflow files and 18 preservation
calls, alongside the English ADR-076 and verification record. The 22 green
workflow results on the former PR head `9d88326` predate integration with
main and are not evidence for the new merge commit. No physical-hardware,
service-initiated power-off, S3 sleep/wake, independently traced negative
PM1 writes or complete ACPI support is claimed.
**The current acceptance score remains 39/100; H05 is still open.**

**Historical handoff (PR #213):** Main `04c8b224e3062400288a353056b75abdd3bcbba5`
and the prior PR source `9d88326be637adccfeccc743f31b3c67185e16a8`
were integrated at `16d8346a2968f0315334e8190eb66e8103b14cf6`;
`ee047c512c8637f57b72528515c564f9092f164a` fixes the CI evidence counts.
The SMP legacy profiles and 17-workflow/18-call evidence contract are retained.
No local Rust, QEMU, strict preflight, build or smoke gate ran in this connector
session. Confirm all applicable CI and security-review results on the latest
source, including native-power, host-quality and qemu-smoke, before squash merge.
Physical-hardware evidence remains missing, so H05 stays open.

## PR #167 — VELD-08 window-switcher integration — 9 November 2026

**Source:** Existing PR #167 at pre-integration head `fdaef7acce36d1865a79406314a4c1159de172fd`, merged in the current integration branch with `main` at `316d4f10470efe16bfdf314304a410f8b1150833`. The implementation preserves compositor-owned window thumbnails, focus-on-confirmation, bounded input handling and explicit opt-in 1x/2x native probes. The latest corrections retain trusted-shell routing, remove minimized-window hiding after selection, require complete app frames before the native assertion, preserve key-edge baselines, and release pointer capture after the full outside-click gesture.

**Evidence:** Local integration commit `09f982c348a9ba98baded4f3401defb2a38cfd0b` passed `tools/progress.py --check`, strict preflight, `tools/test_window_session.py`, the complete window-session host suite, format and warning-denying Clippy, `tools/build.py`, and `tools/run.py --smoke`. Native switcher runs at scales 1 and 2 each passed all 18 phases with raw QEMU exit 33, `switcher_gate=true` and intentionally `native_gate=false`; both reports bind source fingerprint `3e669760446ce83a47d45cf4e5e2d3584860731d9043c53e8a7bc17a6fe56dae` to that commit. GitHub email privacy required a metadata-only rewrite to publication commit `b25a7d10b50240f00b44569ffafba65ae0a064d2`; both commits have identical tree `f1cb8de749cb58b7209b77f7c5271e52f0ca726f`. Compact evidence is stored in [`veld-switcher-20261103/current-main-integration-20261109.json`](verification/veld-switcher-20261103/current-main-integration-20261109.json).

**Evidence boundary:** GitHub reported 47 checks on pre-integration head `fdaef7acce36d1865a79406314a4c1159de172fd`: 46 successful and one neutral (`Graphify Formal Verification`). CodeRabbit completed successfully and no unresolved review thread was present. Those GitHub results do **not** verify publication commit `b25a7d10b50240f00b44569ffafba65ae0a064d2`; current-head CI and review remain required after publication. Historical two-scale receipts remain bound to `537fee48c1c31155a51a997cd7a2236b159e9ac7`. No physical-device, human usability or independent security acceptance is claimed.

**Progress:** **39/100 → 39/100.** A successful switcher subgate does not close the complete native integration, accessibility or recovery criteria.

**Historical handoff (PR #167):** Publish the integrated source without rewriting history. Confirm current-head CodeRabbit, CodeQL and required GitHub checks, then squash-merge with the expected head SHA.

## PR #154 — reviewed K06 and experimental M3/M4 SMP integration — 9 November 2026

**Source:** existing PR #154 `simonvanieperen-studious-spoon` at tested source `fcaa2ef44b7bdfcabbc6368149105f978cade27a` against `main` `540f7286cea57f9fe49c55b896aa9a42fd4b4b35`. K06 MADT (up to eight CPUs) remains the default `native-smp` profile. The distinct SMP Info/M3-M4 (up to four CPUs) remains opt-in under `native-smp-legacy` and its M3/M4 profiles, without changing the default boot ABI. Duplicate UEFI ACPI 2.0 configuration entries are rejected; per-CPU safety and bounded memory-pressure recovery assertions are retained.

**Evidence on source `fcaa2ef44b7bdfcabbc6368149105f978cade27a`:** GitHub Actions [K06 native SMP and experimental M3/M4 SMP](https://github.com/simonvanieperen/BaryonOS/actions/runs/42667770606) completed successfully, including the full 15-stage experimental native and negative matrix. [Native system integration](https://github.com/simonvanieperen/BaryonOS/actions/runs/42667770871) completed successfully, including normal/NX/guard/peer/OOM physical memory tests and actual recovery. [CI](https://github.com/simonvanieperen/BaryonOS/actions/runs/42667770942) passed `qemu-smoke`, but `host-quality` failed a 16-versus-17 evidence invocation count because the new experimental SMP evidence job is now included. The isolated contract-test correction follows in this PR checkpoint. Earlier checks are not acceptance evidence for this later documentation/test HEAD.

**Acceptance:** **39/100 → 39/100.** K06 remains open: the experimental M3/M4 profile is still a separate opt-in implementation and does not establish the full shared concurrent production scheduling/address-space model or physical hardware support. B01 positive native management recovery also remains open. No safety or product acceptance point is added by CI success alone.

**NEXT AGENT HANDOFF:** verify fresh `host-quality`, `qemu-smoke`, K06 and experimental SMP, native-system and CodeRabbit/CodeQL checks after the evidence-test and status commit. Run or confirm `python3 tools/progress.py --check`, `python3 tools/preflight.py --strict`, `python3 tools/build.py`, and `python3 tools/run.py --smoke` on the final source. Resolve valid remaining security review findings; then squash-merge this existing PR with expected HEAD SHA. Do not grant authority, accept K06 or suppress negative cases without supporting evidence.

## PR #154 — K06 and experimental M3/M4 SMP integration checkpoint — 9 November 2026

**Source:** existing PR #154 at `0ac938095908ecaf8e42218823bea97367160a65`, integrated against main `2ed4c087746021da3c8b96511af3209a35d0f441`. The conflicting ACPI MADT/K06 and SMP Info/M3-M4 paths are retained as **separate opt-in build profiles** instead of an unconditional ours/theirs replacement. K06 stays the default `native-smp` profile (up to eight CPUs); the historical SMP Info ABI and its native M3/M4 probes are isolated under `native-smp-legacy` and the dependent `native-smp-m3`/`native-smp-m4` features (up to four CPUs). Stable boot ABI/graphics and legacy SMP wire layouts are feature-gated separately; the legacy test runner has a distinct module name.

**Verification boundary:** source-level integration only. The branch implementation is not yet verified by Cargo, `tools/progress.py --check`, strict preflight, the native build, QEMU smoke, full SMP matrix or the current-head GitHub Actions checks. Historical PR evidence and the older K06 tests do **not** establish merged-runtime acceptance. No criterion is newly accepted: **39/100 → 39/100**. Do not squash-merge before required checks pass and review findings are resolved.

**NEXT AGENT HANDOFF:** run the standard host, formatting, preflight, build and QEMU smoke gates on the actual integrated PR SHA, then `python3 -B -u tools/test_smp.py` and all current K06 `tools/run_smp.py` cases. Repair any compiler, ABI, oracle, or negative-case failures; check CodeRabbit review and GitHub `host-quality`/`qemu-smoke` on the latest head before squash-merge using an expected SHA. Preserve the separate B01 native positive recovery test as open.



## PR #214 — Horizon desktop integration — 9 November 2026

**Source:** PR #214 (`ui/desktop-2030`), integrating main `c663f0b6386203e17e28d52c2c79da161205ee26`
while retaining PR #155's keyring authorisation fix and 39/100 assessment.
The Horizon implementation remains host-rendered; no native UI acceptance is
claimed. Existing CI workflows and capability boundaries are unchanged.

**Fixes:** the SDK protocol-chain semantic heading now expects the visible
`Nebula` label; the graphics wallpaper matches rustfmt; the Orbit dock avoids
integer underflow on tiny widths and has a focused regression test. Previous
positive fade guards and focus-ring behavior are preserved. Desktop-shell
documentation distinguishes the planned running-app indicator from the
implemented view, where the dock is not rendered while Nebula is open.

**Evidence:** on prior head `d54b174005338d82fa5222a7eee770a373fe5c80`, GitHub
`host-quality` failed on the old heading assertion; `native-display-input`
failed on rustfmt, and `native-window-session` failed in the same
protocol-chain test. Prior `qemu-smoke` passed, but is not current-head
evidence. No local Rust, progress, preflight, build or QEMU commands ran in
this GitHub-connector environment. The integrated PR HEAD requires fresh CI.
Progress stays **39/100**; no criterion is newly accepted. Native UX and
user-task observations remain open.

**NEXT AGENT HANDOFF:** verify `python3 tools/progress.py --check`,
`python3 tools/preflight.py --strict`, `python3 tools/build.py`, and
`python3 tools/run.py --smoke` on the integrated head; review CodeRabbit
and the required GitHub `host-quality`/`qemu-smoke` statuses before
squash-merge. The separate B01 positive native recovery test remains open
under PR #155's checkpoint.

## PR #155 keyring recovery authorization — 9 November 2026

**Source:** PR #155 security code at `ef391def0a4bcd825f4664ab21de844ccd4f7ee4` was integrated with main `9554d677c0ab8dd9acfce47f9abe43e22c0215c8` on the existing PR branch. The current main CI workflows and concise documentation hierarchy were preserved. Ordinary applications can no longer export or restore keyring recovery material; only a separately authenticated management ingress can dispatch these operations. No new native management capability was granted.

**Acceptance:** **41/100 → 39/100.** B01 is open because the earlier positive native recovery test used ordinary app authority. The historical five-case native QEMU gate and positive management host tests do not establish native positive management recovery. Checks on the combined HEAD have **not yet been executed or verified**. No other accepted criterion changed.

**NEXT AGENT HANDOFF:** run `python3 tools/progress.py --check`, `python3 tools/preflight.py --strict`, `python3 tools/build.py`, `python3 tools/run.py --smoke`, and `python3 -B -u tools/test_keyring.py` on the combined PR HEAD. Review CodeRabbit and required CI results before squash merge. Physical-hardware and human-security audits remain outstanding.

## PR #181 — PCI device lifecycle policy checkpoint — 9 November 2026

The new `kernel-core` device execution lease registry models PCI function/generation binding, bounded config and BAR access, directional DMA grants, interrupt teardown, quarantine and release. It is a host-testable policy model only; no production kernel driver adapter is connected. Bus-master enablement is test-only until a kernel-issued, enforced DMA/IOMMU-containment proof exists. Live DMA frames are not pinned against the generic allocator's free path. Do not treat this as native PCI, DMA/IOMMU or physical-hardware acceptance.

The CI-blocking rustfmt regression was corrected on PR [#181](https://github.com/simonvanieperen/BaryonOS/pull/181) in `edb5ac77e3dac7e511c664e04e52d009175b6e23`; current `main` was integrated into the PR at `157182c068c62533bf06e4e8a1c994e7e72d2842` without replacing its two source-file changes. The current PR-head CI still must be evaluated; historical tests from earlier source commits are not proof for this checkpoint. Progress: **41/100 → 41/100**; no new criterion is accepted. Follow-up: add enforced DMA pinning/containment, then a fail-closed native adapter and negative runtime tests before exposing drivers to this model.

## PR #284 refresh — 9 November 2026

Dependabot coverage is extended to the remaining independent Cargo workspaces. The separate SDK entry from #285 is retained; its directory is excluded from the grouped entry to prevent duplicates. Progress remains 41/100. Local preflight, build and QEMU smoke passed (exit 0 each); all 11 workflow/Dependabot contract tests passed in WSL with the Windows worktree Git directory explicitly selected.

## PR #285 refresh — 9 November 2026

SDK monthly Dependabot coverage is retained on current main. The current documentation hierarchy is preserved. Progress remains 41/100; no new product acceptance. Local `python tools/preflight.py --strict`, `python tools/build.py`, and `python tools/run.py --smoke` passed (exit 0 each; QEMU raw exit 33, M1:PASS). No physical-hardware acceptance is claimed.

## NEXT AGENT HANDOFF — documentation hierarchy and governance — 9 November 2026

**Source and scope:** base `main` `f704f078213170e72292a5a491f0ee83bd588955`, branch `docs/restructure-project-docs-20261109`. The active README, contribution guidance, security policy, licensing documentation, and status layers have been reorganized so that each information type has one primary source. Historical content was preserved under `docs/history/` before the active files were shortened.

**Progress:** **41/100 → 41/100**. This is documentation and governance work; no new product criterion is accepted and no native OS functionality is changed.

**Open repository administration:** GitHub Wiki pages, GitHub Milestone objects, and a real private vulnerability-reporting channel require repository-level actions that are not exposed by the current connector. They are tracked in issues [#277](https://github.com/simonvanieperen/BaryonOS/issues/277), [#276](https://github.com/simonvanieperen/BaryonOS/issues/276), and [#278](https://github.com/simonvanieperen/BaryonOS/issues/278), respectively. [ROADMAP.md](ROADMAP.md) remains the authoritative milestone source until the Milestone objects exist.

**Resume:** review this branch through CI and automated review. Confirm that README, `tools/progress.py`, `docs/progress.json`, `docs/PROGRESS.md`, and `docs/assets/progress.svg` remain consistent with `python3 tools/progress.py --check`.

## PR #185 — USB HID control setup encoding — 9 November 2026

**Source:** Existing PR #185, branch `copilot/driver-usb-hid-control` at `ad2d6d63c24d95174c083e2cbbc6ff7da263f09e`, integrated against `main` `75fbaa2dfdd9e14c0b2d64ad78e6e081bc283592`. The merge resolution retains the current main tree and the USB module/test changes. All 21 CI workflow files changed in the historical PR already match main; outdated README, project-status and progress snapshots are not reinstated. The PR's source-bound SMP validation receipt is retained as historical evidence, not as verification of this integration.

**Result:** `drivers/usb/src/hid_control.rs` encodes HID 1.11 class/interface `SET_PROTOCOL(boot)` and `SET_IDLE(report ID 0)` setup packets. Its exhaustive host tests cover interface numbers and idle durations. No USB transport, device identity, control completion or new authority is implemented; the caller must validate actual transfer completion.

**Verification boundary:** The previous branch reported passing targeted USB host tests and historical QEMU/SMP runs. This integration has **not** executed `cargo +1.90.0 test --locked --manifest-path drivers/usb/Cargo.toml`, strict Clippy, formatting, `python3 tools/preflight.py --strict`, `python3 tools/build.py` or `python3 tools/run.py --smoke` in the current session; the connected execution environment lacks Rust/QEMU and the project's RTK/Ponytail tooling. Hosted CI and review on the new PR head require separate confirmation. The earlier SMP receipt under `docs/verification/` is bound to its recorded source and does not demonstrate HID hardware support.

**Progress:** **41/100 → 41/100**; no fixed acceptance criterion is newly satisfied.

**Next:** confirm the current PR-head CI/review gate and squash-merge this existing PR; implement and test the HID control-transfer transport and completion boundary separately.


## Authoritative sources

| Question | Primary source |
|---|---|
| What is Baryon OS? | [README.md](../README.md) |
| What is the current technical state? | This file |
| How is the 0–100 score calculated? | [PROGRESS.md](PROGRESS.md) and [progress.json](progress.json) |
| What must ultimately be achieved? | [ROADMAP.md](ROADMAP.md) and [FULL-TEST-RELEASE.md](FULL-TEST-RELEASE.md) |
| What concrete work is open? | GitHub Issues and [WORK-QUEUE.md](WORK-QUEUE.md) where still applicable |
| Which architecture decisions apply? | [DECISIONS.md](DECISIONS.md) and `docs/implementation/` |
| How is the system built and tested? | [DEVELOPMENT.md](DEVELOPMENT.md) and [TEST-STRATEGY.md](TEST-STRATEGY.md) |
| Where is source-bound test evidence stored? | [verification/](verification/) |
| Where is historical status stored? | [history/](history/) |
| How can someone contribute? | [CONTRIBUTING.md](../CONTRIBUTING.md) |
| How should a vulnerability be reported? | [SECURITY.md](../SECURITY.md) |
| Which software license applies? | [LICENSE](../LICENSE), [LICENSING.md](../LICENSING.md), and [MPL-2.0-FILES.txt](../MPL-2.0-FILES.txt) |

## Current development state

The fixed progress score is **39/100**. Detailed accepted and open criteria are maintained only in [PROGRESS.md](PROGRESS.md) and [progress.json](progress.json). Recent source-bound implementation and CI evidence remains available under [verification/](verification/).

The former checkpoint sequence is preserved in [PROJECT-STATE-through-2026-11-09.md](history/PROJECT-STATE-through-2026-11-09.md). That snapshot is historical context, not a second current-status source.

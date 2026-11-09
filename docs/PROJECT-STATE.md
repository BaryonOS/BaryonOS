# Current project status

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

**NEXT AGENT HANDOFF (PR #213):** Main `04c8b224e3062400288a353056b75abdd3bcbba5`
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

**Evidence boundary:** GitHub reported 47 checks on `fdaef7acce36d1865a79406314a4c1159de172fd`: 46 successful and one neutral (`Graphify Formal Verification`). CodeRabbit completed successfully and no unresolved review thread was present. These results do **not** verify the current merge with `316d4f10470efe16bfdf314304a410f8b1150833`. Historical two-scale receipts remain bound to `537fee48c1c31155a51a997cd7a2236b159e9ac7`; no physical-device, human usability or independent security acceptance is claimed.

**Progress:** **39/100 → 39/100.** A successful switcher subgate does not close the complete native integration, accessibility or recovery criteria.

**NEXT AGENT HANDOFF (PR #167):** Run `python3 tools/progress.py --check`, `python3 tools/preflight.py --strict`, `python3 tools/build.py`, `python3 tools/run.py --smoke`, the targeted window-session host tests and the 1x/2x native switcher gate on the exact integrated head. Confirm current-head CodeRabbit, CodeQL and required GitHub checks before squash merge.

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

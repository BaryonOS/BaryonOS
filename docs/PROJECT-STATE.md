# Current project status

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

The fixed progress score is **41/100**. Detailed accepted and open criteria are maintained only in [PROGRESS.md](PROGRESS.md) and [progress.json](progress.json). Recent source-bound implementation and CI evidence remains available under [verification/](verification/).

The former checkpoint sequence is preserved in [PROJECT-STATE-through-2026-11-09.md](history/PROJECT-STATE-through-2026-11-09.md). That snapshot is historical context, not a second current-status source.

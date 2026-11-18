# SMP guide translation — 17 November 2026

Task P0-274-SMP translates only the first-party prose in `docs/implementation/K06-SMP.md` to professional English. Source: main `95d59ad52761c1f8f0201d934556902b90ce63cb`; branch `codex/phase0-smp-guide-english`.

Invariant: preserve every SMP requirement, timing/liveness bound, security boundary, status, numeric literal, command, identifier and evidence limitation. The existing English appendix remains byte-identical. The lead additionally owns this receipt, the top handoff and the unchanged-score assessment; no code, workflow or criterion changes are included.

Independent AI semantic review passes on final SHA-256 `3dba5104d2d57158534c72ae35929751fdb653c1034eff8327e7d13ce26643ad`. The sole review finding was an ambiguous English reading of `461.816` bytes; a local clarification preserves that literal and identifies its period as a thousands separator. Reversing only the clarification reproduces the first reviewed hash.

Preservation assertions and whitespace checks return 0: 365 inline spans, 773 numeric tokens, two URLs, the command block, all 41 previous heading anchors and the English appendix are retained. Exact hashes and raw-report references are in [verification.json](verification.json). The acceptance-report destination `docs/verification/k06-smp-sched/native-report.json` is absent on the original source and current checkout; this change supplies no replacement evidence or K06 acceptance.

Required local checks: progress generation/check, source/hash preservation and `git diff --check`; required current-head hosted checks remain a separate merge gate. Native execution, the complete 74×50 matrix, hardware and human audit are NOT RUN by this documentation task. Progress remains 41/100, issue #274 remains open, and final baseline B/governance phases 1–6 remain unfinished. Eligible Phase 0 squash merge follows the explicit owner authority after all gates; no auto-merge or bypass.

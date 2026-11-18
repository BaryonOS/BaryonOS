# CI guide translation — 17 November 2026

Task P0-274-CI translates the remaining Dutch prose in `docs/CI.md` to professional English. Source: main `b718f6bafdfc4927d3b7acfe786e67dcb8f88eec`; branch `codex/phase0-ci-guide-english`. This is a bounded contribution to issue #274, which remains open.

Invariant: preserve commands, versions, gates, dated evidence, accepted requirements and historical limitations. The write set is the CI guide, this evidence directory, the lead-owned top project handoff and the existing progress assessment/generated views. No runtime, workflow, hook or product configuration changes are included.

An independent AI source comparison found one unsupported integration-date label. Removing that label was the only subsequent guide change; the final hash reproduces the reviewed version when that one label is restored. The final review found no remaining semantic blocker. This is not a human audit or a new verification of the historical CI results.

Preservation checks passed (exit 0): 47 inline-code occurrences, one fenced block, five Markdown link targets, nine URLs, all nine previous heading anchors, and all 82 original numeric tokens. All five local link targets exist. Explicit old anchor IDs preserve existing incoming links. The existing CI workflow contract suite passes: 26 tests, exit 0. Full raw command output and its exit-code sidecar remain in the task worktree under `build/tokenguard/run-7gwopruy.{log,json}`; compact hashes are recorded in [verification.json](verification.json).

The progress assessment remains 41/100. Translation provides no new native or product acceptance. Native tests were not repeated for this documentation-only change; current-head required hosted CI is separate and must pass before merge readiness. Full repository translation, live Wiki consistency, final integrated security assessment and baseline B remain outstanding. No merge or auto-merge is authorized by this change.

Resume: verify this branch's remote head and required hosted checks. Preserve the separate integration checkpoint and all per-workstream handoffs; do not interpret this task as release of Orbitaal or S09 ownership.

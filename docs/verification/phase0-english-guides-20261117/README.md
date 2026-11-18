# Native guide translations — 17 November 2026

Task P0-274-GUIDES / P0-274-IPC-CONTRACT translates three remaining first-party guides into professional English: `docs/implementation/NATIVE-MATERIAL-LIBRARY.md`, `sdk/examples/ipc-wait-probe/README.md` and `docs/contracts/NATIVE-IPC-WAIT.md`. Source comparison: `cece3041187e1b7d08ec7708c36b84c728d6e2d2`. The earlier CI-guide translation and its separate receipt remain unchanged in this PR.

Invariant: preserve technical requirements, permissions, uncertainty, dated execution claims, code blocks, identifiers, numeric literals and stable UI/protocol/wire-spec text. Explicit old heading IDs preserve incoming links. Lead integration also owns this receipt, the top handoff and the unchanged-score assessment; workers changed only their three assigned documents.

Independent AI semantic comparison found no blocker on the exact hashes in [verification.json](verification.json). Source-preservation and whitespace checks pass (exit 0). Earlier auxiliary extraction heuristics rejected English grammatical forms; the reports preserve those failures, diagnostics and corrected checks. No product test or acceptance requirement was weakened. Full raw records are retained in the durable Phase 0 handoff archive, with their hashes in this receipt.

Integration must preserve main `95d59ad52761c1f8f0201d934556902b90ce63cb`, the earlier CI guide and all workstream handoffs. Required commands are the existing workflow-contract suite, progress generation/check, source/hash preservation and `git diff --check`, followed by required hosted checks on the integrated head. Native tests are NOT RUN for this documentation-only change. Translation provides no new runtime, security, hardware or human-audit evidence.

Issue #274 remains open for actual repository-wide coverage; progress remains 41/100. Final baseline B and governance phases 1–6 remain outstanding. Merge eligibility follows the explicit Phase 0 authority and all current-head gates; auto-merge remains prohibited.

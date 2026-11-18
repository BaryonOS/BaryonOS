---
name: development-loop
description: Execute one bounded BaryonOS task with shared ownership, risk-based verification and a compact continuation checkpoint.
---

# Shared development loop

Root `AGENTS.md` is policy. This procedure is shared by Claude Code, Codex and
GitHub Copilot; client adapters add tool constraints, never authority.

1. Orient once: branch, full HEAD, short status, current `docs/STATUS.md` and the
   latest relevant `docs/PROJECT-STATE.md` checkpoint. Preserve dirty work. Read
   applicable directory instructions and only the contracts relevant to the task.
2. Claim one existing actionable task in the checkpoint or existing task ledger.
   Check ownership, locks and open PRs. Use a separate worktree for another writer;
   no shared files or Cargo target directories have two concurrent writers.
3. Recall relevant paths with hosted Graphify, then find definitions, callers and
   tests. Compare indexed commit with HEAD; use focused source reads for differences.
   If unavailable, report it and read the owning source; do not build a local graph.
4. Before editing, record goal, invariant, exact write set, acceptance commands and
   evidence limit in the existing task record. Reuse the existing implementation.
   Do not start another planning phase for already approved work.
5. Make the smallest complete change. Preserve callers, validation, error handling,
   security and accessibility. No unrelated cleanup or speculative abstraction.
6. Run targeted tests selected by changed behaviour and risk. Security boundaries
   need meaningful negative tests. Reuse source-bound evidence when relevant code,
   dependencies, build profile and environment remain valid, and explain that reuse.
   Repeat native/integration gates when affected or required for acceptance, not
   routinely for unchanged code. Mandatory hosted checks and protections remain.
7. Review the complete diff once; repair concrete failures and rerun affected tests.
   After two identical failures without new diagnostics, change approach. Default:
   one lead, at most one additional reviewer when risk warrants it. A second AI is
   not a human audit. No optional review quota retries or continuous CI polling.
8. Checkpoint coherent work: exact source, commands, raw exit codes, local log paths,
   NOT RUN reasons, blocker and executable next step. Update the existing progress
   assessment without awarding unaccepted criteria. Commit, push and open/update a
   draft PR under the current mandate. Manual squash needs mandatory checks and
   current owner authorization; never auto-merge or bypass protection.
9. Record one durable lesson in shared Graphify memory when useful; changing task
   state belongs in existing checkpoints, never AGENTS.md. Continue the next disjoint
   eligible task. External blockers do not stop independent implementation. At a hard
   limit leave a compact checkpoint; no work continues after the session ends.

Use one suitable output wrapper per command, selectively. Preserve raw logs locally;
trivial changes need no separate evidence package. Expand checks for actual risk or
failure. Host, simulation, native QEMU, hardware, client runtime parity and human audit
are separate evidence levels. Governance does not require zero open issues/findings
and does not release the issue-pipeline execution hold.

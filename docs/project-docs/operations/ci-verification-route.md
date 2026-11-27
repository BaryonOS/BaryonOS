---
title: CI verification route
type: operation
created: 2026-11-26
updated: 2026-11-26
sources:
  - code: .github/workflows/merge-hold-recheck.yml
  - code: tools/merge_hold_recheck.py
verified_at: c44134c6
---

# CI verification route

This operation page directs CI investigations to their canonical owners. It defines no status, policy, acceptance evidence, gate result, or merge authority; exact-head verification, merge-hold freshness, the Security Floor, and existing gates remain governed by those sources.

| Need | Canonical source |
|---|---|
| Gate selection and exact-head verification | [CI contract](../../CI-CONTRACT.md) |
| CI operation and re-check interpretation | [CI operations](../../CI.md) |
| Evidence collection and retention | [CI evidence](../../CI-EVIDENCE.md) |
| Source-bound audit behind PR #552 | [Stale-green audit evidence](../../verification/ci-audit-stale-green-20261126/README.md) |
| Re-check event definition | [`.github/workflows/merge-hold-recheck.yml:1`](../../../.github/workflows/merge-hold-recheck.yml) |
| Re-check implementation | [`tools/merge_hold_recheck.py:1`](../../../tools/merge_hold_recheck.py) |

> ⚠️ This navigation page and a stale green label are not acceptance evidence. Confirm the current pull-request head, latest applicable gate attempts, merge-hold freshness, and source-bound evidence in the canonical sources before relying on a result.

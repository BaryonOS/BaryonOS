# Issue 274 helper CodeQL corrections — 18 November 2026

Implementation source `159eda463c70dc538aec3b4c04ae4527b1c6c7e2` replaces the incomplete HTML regular expressions with the existing `marked` lexer and reads Wiki content only from regular blobs in one frozen Git commit. It also changes the sole remaining generated-build documentation link into an explicit local path. No CodeQL query, workflow, finding or validation rule is disabled.

[Checks](checks.json) record the exact helper hashes, both failing restored-source controls, three focused Python regressions, Node syntax, the complete 406-test tools suite and the real local Wiki collector. The final collector is bound to this commit and Wiki commit `4a01b8b2ad2695da509a512ea688007c964f56da`; it reports 7,667 local links, zero unresolved targets and zero unsupported targets. The complete tools run used the same executable helper/test blobs immediately before the documentation-only link correction; it is not relabelled as a full-tree run on the implementation commit.

Independent read-only review passed the exact `collect.cjs` and `check.py` blobs. Hosted current-head CodeQL remains mandatory and pending; this host evidence does not complete issue 274, validate all language/UI content, or establish a native/security acceptance claim. Score remains 41/100.

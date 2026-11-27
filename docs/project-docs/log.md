# Work log

Append-only. Newest entries at the bottom.

---

## [2026-11-26] setup | maintained documentation scaffolded

- Categories: concepts - architecture - subsystems - decisions - reference - operations - history
- Baseline commit: `c44134c6`
- Created: `docs/project-docs/{CLAUDE,index,log}.md`, `raw/`, four
  `.claude/commands/docs-*` files and `.claude/scripts/docs-lint.py`.
- Started in grow mode with [[ci-verification-route]] as the first maintained area.
- Preserved `docs/wiki/` as the public explanatory source and all existing canonical
  status, decision and evidence files in place.
- Added a root documentation delegation and kept the vendored linter scoped to
  `docs/project-docs`, because the repository's existing `docs/` tree is not a single
  frontmatter-based wiki.

## [2026-11-27] structure | made the documentation layer model-independent

- Made `docs/project-docs/AGENTS.md` the canonical maintenance contract.
- Reduced `CLAUDE.md` files and `/docs-*` commands to compatibility adapters.
- Added [[documentation-workflow]] as the direct procedure for Claude, Codex, Copilot,
  Ollama, OpenCode and other agents.
- Kept the grow-on-use model and existing canonical documents in place; no project facts
  or historical records were migrated or duplicated.
- Separated unclassified product-change discovery from explicit page-source freshness,
  so documentation and adapter sources cannot be hidden by the product-code filter.
- Kept unverified freshness open for judgement instead of treating it as clean or stale.

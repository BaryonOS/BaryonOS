# Changes

## 2.0.1 — 19 November 2026

- Excluded W001473 ("Monotasking Systems") from the active corpus under
  ADR-W001473-PROVENANCE-20261119 (issue #283): its text was ported from the old
  OS Wiki in 2006 and every revision predates the OSDev CC0 start date, so no
  reuse grant is established. Evidence:
  `docs/verification/w001473-provenance-20261119/`.
- Added `scripts/exclude_sources.py`, which removes a record's section, recomputes
  later byte and line positions, updates the indexes and manifest, and appends the
  reason to `references/exclusions.json`. 3,131 source records remain.
- Revision metadata and Git history are unchanged; earlier commits still contain
  the excluded text.

## 2.0.0 — 28 October 2026

- Complete Codex/ChatGPT Agent Skill with YAML frontmatter and agents/openai.yaml.
- Seven task-oriented workflows alongside the 24 existing thematic source routes.
- All 3,132 source records retained as text in 16 directly readable Markdown corpus files.
- Thousands of separate source files and the large SQLite database replaced by a
  compact, populated source register with byte and line positions and a Python-standard-library search tool.
- Source links updated and checked again; original source IDs retained.
- Installation paths, use without a shell, test boundaries, source-injection protection and
  truthful progress/agent reporting explicitly documented.
- No installation, repository change, cloud import or public publication performed.

This is a repackaging with expanded agent instructions, not a new wiki snapshot.

# BaryonOS maintained project-documentation contract

This directory is a lightweight, model-independent navigation layer for Claude, Codex,
Copilot, Ollama, OpenCode and other agents. It is a linked set of Markdown pages, not a
large memory file, a replacement knowledge base or a separate site.

The repository and directory-level `AGENTS.md` files define how work is performed. This
file defines how the growing documentation layer is read and maintained. Client-specific
files are compatibility adapters only.

## Authority and scope

The reviewable public explanatory Wiki source remains in [`docs/wiki/`](../wiki/), with
its repository guide in [`docs/WIKI.md`](../WIKI.md). Canonical contracts, current
status, decisions, task state and version-bound evidence remain in their existing owning
files. This layer links to those sources; it does not copy or replace them.

Hosted Graphify memory can help an agent recall context, but it is neither the canonical
project record nor proof that a fact is current. Verify consequential claims against the
owning repository source.

> ⚠️ The existing hook discovers the setup through `docs/CLAUDE.md` and invokes
> `.claude/scripts/docs-lint.py`. Those paths are compatibility surfaces, not Claude-only
> authority. Keep the linter default at `docs/project-docs`; otherwise it interprets the
> heterogeneous canonical `docs/` tree as maintained pages.

## Token-efficient reading route

1. Read [`index.md`](index.md).
2. Open only the pages relevant to the task, normally one to three.
3. Follow their explicit links to canonical sources only when needed.
4. If the route is missing, report the gap instead of loading the entire documentation
   tree. Add a focused page when work actually touches that area.

Do not create a growing `MEMORY.md`. A single file forces every agent to spend tokens on
unrelated history. Page-level `verified_at` values permit incremental checks against only
the sources a page covers.

## Language and page categories

Maintained pages are written in English. Frontmatter keys, paths and link targets remain
unchanged regardless of the conversation language.

| Access pattern | Categories |
|---|---|
| Exploratory | `concepts` `architecture` `subsystems` `decisions` |
| Lookup | `reference` `operations` |
| Historical | `history` |

Exploratory pages need useful `[[links]]`. Lookup pages should remain short and direct.
Decide the category before adding a page.

## Layout

```text
docs/project-docs/
├── AGENTS.md       model-independent contract
├── CLAUDE.md       compatibility adapter
├── index.md        small table of contents and reading order
├── log.md          append-only maintenance log
├── raw/            source material that exists neither in code nor canonical docs
├── concepts/
├── architecture/
├── subsystems/
├── decisions/
├── reference/
├── operations/
└── history/
```

Empty category directories are optional. Add them only when a focused page is needed.

## Maintained page format

```yaml
---
title: Page title
type: concept | architecture | subsystem | decision | reference | operation | history
created: YYYY-MM-DD
updated: YYYY-MM-DD
sources:
  - code: path/from/repository/root
  - raw: YYYY-MM-DD_source-name.md
  - ext: https://primary.example/source
verified_at: <short-or-full-commit-sha>
---
```

Rules:

- Link another maintained page as `[[page-name]]`.
- Every factual claim needs an owning source. Mark unsupported claims `(unverified)`.
- Point to code or canonical documents; do not paste them into a second source of truth.
- Write every operational trap as its own `> ⚠️` paragraph.
- Change `updated` only when the body changes. Move `verified_at` after actually checking
  the page against its named sources.
- One fact lives on one page. Other pages link to it.
- Never record secret values, private endpoints, personal host paths, access tokens,
  production key material or unpublished security-report details.

## Incremental maintenance

The layer grows with project work; it is not backfilled speculatively.

- When work finishes in an uncovered area, add one focused page or extend the nearest
  page. Document what the source does, why it is shaped that way and the traps encountered.
- Do not document unrelated areas merely to make the tree look complete.
- Before editing an area, read the relevant maintained page's `> ⚠️` blocks.
- After a source changes, compare from the page's `verified_at` commit and read only the
  affected diff. Correct the prose or move only `verified_at` when the prose remains true.
- Run the linter after documentation changes.

> ⚠️ Only one session should synchronise this documentation layer at a time. Concurrent
> sessions otherwise edit `index.md` and append to `log.md` together.

## Code-change filter

For documentation freshness, product code is the repository excluding documentation and
client adapters:

```text
code = repository - { docs, .agents, .claude, .codex, AGENTS.md, CLAUDE.md }
```

This is a judgement filter, not a version-control exclusion. All documentation and
adapters remain committed with the repository.

> ⚠️ A documentation-only commit still moves `HEAD`. Counting it as a product-code
> change creates false staleness and teaches agents to ignore the freshness signal.

## Common operations

The client-neutral procedure is in [[documentation-workflow]]. Clients that support the
existing `/docs-sync`, `/docs-lint`, `/docs-query` and `/docs-status` aliases may use
them; every other agent can perform the same steps directly from that page.

## Growth mode

docs-mode: grow

release-commands:

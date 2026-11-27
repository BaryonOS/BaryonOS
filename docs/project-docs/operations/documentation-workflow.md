---
title: Model-independent documentation workflow
type: operation
created: 2026-11-27
updated: 2026-11-27
sources:
  - code: AGENTS.md
  - code: docs/AGENTS.md
  - code: .claude/scripts/docs-lint.py
verified_at: 2c88aed6
---

# Model-independent documentation workflow

This procedure works for any agent that can read repository files and run Git and
Python. It does not depend on a specific model, chat client, plugin or hosted memory.

Clients without automatic instruction-file discovery should receive this starter text:

> Read the root `AGENTS.md` and every applicable directory `AGENTS.md`. For project
> context, start at `docs/project-docs/index.md`, load only the relevant maintained
> pages, and verify consequential claims against their owning sources.

## Start or answer a project question

1. Read [`docs/project-docs/index.md`](../index.md).
2. Select only the relevant maintained pages, normally one to three.
3. Follow their links to canonical contracts, status, decisions or evidence as needed.
4. State that the route is missing when the index has no relevant page. Search the
   owning source only when the task requires it, and add a focused page after the work.

> ⚠️ Do not preload the whole tree or a generated summary into every prompt. The index
> is the routing layer; focused pages provide progressive disclosure.

## Query

Read the index, relevant pages and their named sources. Attribute the answer to those
pages. If the answer is absent, report the likely owning path and the documentation gap
instead of presenting an improvised answer as maintained knowledge.

## Check integrity

```bash
python3 -B .claude/scripts/docs-lint.py --root . --docs-dir docs/project-docs
```

The script checks frontmatter, `[[links]]`, orphans, source paths, code citations,
staleness and unverifiable freshness. Its location is retained for compatibility with
existing hooks; it is ordinary Python and can be run by every agent.

> ⚠️ The `.claude/` path does not make the linter or documentation Claude-specific.
> Authority lives in repository `AGENTS.md` files and this maintained layer.

## Check freshness

Take the oldest `verified_at` value from maintained pages, then separate product-code
changes from documentation and adapter changes:

```bash
git log <baseline>..HEAD --oneline -- . \
  ':(exclude)docs' ':(exclude).agents' ':(exclude).claude' \
  ':(exclude).codex' ':(exclude)AGENTS.md' ':(exclude)CLAUDE.md'
```

If this is empty, no documentation sync is needed. Then inspect stale and unverified
pages with the linter and read the last entries in `docs/project-docs/log.md`.

## Synchronise after source changes

1. Find the oldest relevant `verified_at` commit.
2. Collect only product files changed since that commit.
3. Match them to each page's `sources[].code` paths.
4. Read the actual source diff for affected pages.
5. Correct prose and `updated`, or move only `verified_at` if the body remains true.
6. Propose one focused page for an uncovered area actually touched by the task.
7. Update `index.md` only for added pages, append `log.md`, then run the linter.

> ⚠️ Perform synchronisation in one session at a time. Multiple writers can safely work
> elsewhere, but `index.md` and `log.md` are single-writer coordination surfaces.

## Relationship to shared memory

Graphify or another memory service may accelerate recall. Repository documentation
remains portable when that service or a particular model is unavailable. Memory results
must be checked against the current owning source before they affect consequential work.

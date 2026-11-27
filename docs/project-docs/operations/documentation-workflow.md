---
title: Model-independent documentation workflow
type: operation
created: 2026-11-27
updated: 2026-11-27
sources:
  - code: AGENTS.md
  - code: docs/AGENTS.md
  - code: .claude/scripts/docs-lint.py
verified_at: 934e140b
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
existing hooks; it is ordinary Python and can be run by every agent. A stale-page result
points directly to the synchronisation section below.

The linter treats root-level `AGENTS.md`, `CLAUDE.md`, `index.md`, `log.md` and
`README.md` files in this layer as metadata rather than maintained content pages.

> ⚠️ The `.claude/` path does not make the linter or documentation Claude-specific.
> Authority lives in repository `AGENTS.md` files and this maintained layer.

## Check freshness

First run the linter. It compares every page against all of its named `sources[].code`,
including sources under documentation and client-adapter directories:

```bash
python3 -B .claude/scripts/docs-lint.py --root . --docs-dir docs/project-docs
```

Separately, take the oldest `verified_at` value and discover product-code changes that
may not yet belong to a maintained page:

```bash
git log <baseline>..HEAD --oneline -- . \
  ':(exclude)docs' ':(exclude).agents' ':(exclude).claude' \
  ':(exclude).codex' ':(exclude)AGENTS.md' ':(exclude)CLAUDE.md'
```

An automatic no-sync conclusion is allowed only when this product-code query is empty
and the linter reports neither stale nor unverified pages. An unverified result requires
judgement first: determine why freshness cannot be computed, then either re-verify the
page or record that its non-code sources make the result intentionally undecidable. Read
the last entries in `docs/project-docs/log.md` for recent work.

> ⚠️ Product-change discovery may exclude documentation and adapters, but named page
> sources may not. Treating one filter as both signals can hide a stale page.

## Synchronise after source changes

1. Find the oldest relevant `verified_at` commit.
2. Collect changes to every named `sources[].code` path, including documentation and
   adapters, and match them to affected pages.
3. Separately collect product files changed since that commit; unmatched files may need
   a new focused page.
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

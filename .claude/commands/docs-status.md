# /docs-status -- docs wiki status

Compatibility alias. The model-independent procedure is in
`docs/project-docs/operations/documentation-workflow.md`.

## 1. Commits not yet reflected (first)

Against the oldest `verified_at` across all pages. **Count code and docs separately** --
a docs-only commit is not something `/docs-sync` acts on.

```bash
git log <baseline>..HEAD --oneline -- . \
  ':(exclude)docs' ':(exclude).agents' ':(exclude).claude' \
  ':(exclude).codex' ':(exclude)AGENTS.md' ':(exclude)CLAUDE.md' | wc -l
git log <baseline>..HEAD --oneline -- docs .agents .claude .codex AGENTS.md CLAUDE.md | wc -l
```

```
<baseline>..HEAD   code 0 commits - docs 1 commit    -> /docs-sync not needed
<baseline>..HEAD   code 4 commits - docs 0 commits   -> /docs-sync recommended
```

**If code is 0, do not recommend `/docs-sync`.** Recommending work that does not exist is
how people stop trusting `/docs-status`.

## 2-5

Stale and unverified pages, unprocessed `raw/` material, unfinished `- [ ]` items, the
last 3 entries of `docs/project-docs/log.md`.

```bash
python3 .claude/scripts/docs-lint.py --root . --docs-dir docs/project-docs --quiet
```

## 6. Size

Page count per category.

## Reporting

Fit it on one screen. If something needs doing, recommend it in one line at the end.

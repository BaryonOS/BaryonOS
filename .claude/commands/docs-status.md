# /docs-status -- docs wiki status

Compatibility alias. The model-independent procedure is in
`docs/project-docs/operations/documentation-workflow.md`.

## 1. Commits not yet reflected (first)

Against the oldest `verified_at` across all pages, count product code and docs separately.
This count discovers unclassified work; the linter below independently checks every
named page source, including sources inside docs and adapter directories.

```bash
git log <baseline>..HEAD --oneline -- . \
  ':(exclude)docs' ':(exclude).agents' ':(exclude).claude' \
  ':(exclude).codex' ':(exclude)AGENTS.md' ':(exclude)CLAUDE.md' | wc -l
git log <baseline>..HEAD --oneline -- docs .agents .claude .codex AGENTS.md CLAUDE.md | wc -l
```

```
<baseline>..HEAD   code 0 commits - docs 1 commit    -> check named-source staleness
<baseline>..HEAD   code 4 commits - docs 0 commits   -> /docs-sync recommended
```

Recommend `/docs-sync` when product code changed or the linter reports a stale page.
Recommend no sync automatically only when product changes, stale pages and unverified
pages are all absent. An unverified page remains open for judgement; do not treat it as
stale or clean. A docs-only commit that does not touch any named source is not work for
`/docs-sync`.

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

# /docs-sync -- reflect code changes into the docs wiki

Compatibility alias. Follow the model-independent procedure in
`docs/project-docs/operations/documentation-workflow.md` and the maintenance contract in
`docs/project-docs/AGENTS.md`.

## Procedure

> ⚠️ One syncer at a time, and pull first. Two sessions syncing at once both append
> `docs/project-docs/log.md` and edit `docs/project-docs/index.md`, and the second one
> conflicts. In a multi-session setup, let one session sync and the others wait.

### 1. Baseline

The **oldest** `verified_at` sha across `docs/project-docs/**/*.md` frontmatter.

### 2. Check named sources and product changes separately

First run the linter. It checks every page's explicit `sources[].code`, including named
documentation and adapter sources:

```bash
python3 -B .claude/scripts/docs-lint.py --root . --docs-dir docs/project-docs
```

Then discover product changes that may not have a maintained page. Do not count general
docs commits here: editing only unrelated docs still moves HEAD.

```bash
git log <baseline>..HEAD --oneline -- . \
  ':(exclude)docs' ':(exclude).agents' ':(exclude).claude' \
  ':(exclude).codex' ':(exclude)AGENTS.md' ':(exclude)CLAUDE.md'
```

Stop automatically only when this product query is empty and the linter reports neither
stale nor unverified pages. A stale page needs synchronisation; an unverified page needs
judgement before any no-sync result. Otherwise process the named stale pages, judge the
unverified pages and inspect any changed product files.

### 2-1. Collect changed files

```bash
git log <baseline>..HEAD --name-only --pretty=format: -- . \
  ':(exclude)docs' ':(exclude).agents' ':(exclude).claude' \
  ':(exclude).codex' ':(exclude)AGENTS.md' ':(exclude)CLAUDE.md' \
  | sort -u | grep -v '^$'
```

Also collect changes to every stale page's named `sources[].code` paths, even when those
paths are excluded by the product query above.

> Git quotes non-ASCII paths by default (`core.quotepath`), so a path with
> non-Latin characters comes back escaped and a naive `grep '^docs/'` misses it.
> Use `-z`, or `git -c core.quotepath=false`, when a filename might not be ASCII.

### 3. Select affected pages

Use the linter result and match named-source paths by **prefix**. Keep unmatched product
files as the separate unclassified set. Report N changed product files plus N changed
named sources -> M affected pages.

### 4. Update

For each page, **read the actual diff, not the commit message**
(`git diff <baseline>..HEAD -- <path>`).

- Prose still correct -> move `verified_at` only
- Prose wrong -> fix the body, and move both `updated` and `verified_at`
- A cited `file:line` that shifted -> new line number

> ⚠️ `verified_at` is written *before* you commit, so it names the commit before yours.
> A docs-only commit does not make anything stale: staleness counts only commits that
> touch a page's `code:` sources. If one commit changes both code and the page that
> describes it, that page shows "stale by 1" -- commit code and docs separately.

### 5. Unclassified changes

List files that matched no page and **propose new pages**. Do not create them without
approval.

### 6. Finish

- Update `docs/project-docs/index.md` (if pages were added)
- Append to `docs/project-docs/log.md`
- Verify with `/docs-lint`

## Principles

- **Only read what changed.** Do not re-read the whole codebase every time
- State lives in `verified_at` and nowhere else. Do not add a status file

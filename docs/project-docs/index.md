# BaryonOS -- maintained project documentation

This lightweight index grows only where current work touches the project. The existing
[public Wiki source](../WIKI.md), canonical contracts, current status, decisions and
version-bound evidence keep their existing authority and are linked rather than copied.

This directory is a wiki maintained by an LLM: linked markdown pages, nothing outside the
repository. The rules are in [`CLAUDE.md`](CLAUDE.md).

## Start here

1. [Public explanatory Wiki source](../WIKI.md)
2. [Current project status](../PROJECT-STATE.md)
3. [CI contract](../CI-CONTRACT.md)
4. [[ci-verification-route]]

---

## concepts

No maintained pages yet; use the existing canonical documentation until work touches
this category.

## architecture

No maintained pages yet.

## subsystems

No maintained pages yet.

## decisions

Decisions remain canonical in [DECISIONS.md](../DECISIONS.md); focused maintained pages
are added only when related work requires one.

## reference

No maintained pages yet.

## operations

- [[ci-verification-route]] -- find the canonical CI contract, operating guidance,
  evidence rules and implementation sources without creating a second gate register.

## history

Historical and verification records remain in their existing directories; this index
does not migrate them.

---

## Using this wiki

```
/docs-sync     right after a deploy or release -- reads only what changed since verified_at
/docs-lint     integrity check (orphans, broken links, broken sources, stale, unverified)
/docs-query    ask the docs. If it is not there, the answer says so
/docs-status   commits not yet reflected, stale pages, recent work
```

Every page carries `verified_at`, the commit it was checked against. Change code, run
`/docs-sync`.

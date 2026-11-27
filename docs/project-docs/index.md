# BaryonOS -- maintained project documentation

This lightweight index grows only where current work touches the project. The existing
[public Wiki source](../WIKI.md), canonical contracts, current status, decisions and
version-bound evidence keep their existing authority and are linked rather than copied.

This directory is a model-independent, linked documentation layer stored with the
repository. The rules are in [`AGENTS.md`](AGENTS.md). Client-specific files such as
`CLAUDE.md` are compatibility adapters only.

## Start here

1. [[documentation-workflow]] -- how any AI reads, searches and maintains this layer.
2. [Public explanatory Wiki source](../WIKI.md)
3. [Current project status](../PROJECT-STATE.md)
4. [CI contract](../CI-CONTRACT.md)
5. [[ci-verification-route]]

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

- [[documentation-workflow]] -- model-independent, token-efficient reading and
  maintenance procedure.
- [[ci-verification-route]] -- find the canonical CI contract, operating guidance,
  evidence rules and implementation sources without creating a second gate register.

## history

Historical and verification records remain in their existing directories; this index
does not migrate them.

---

## Using this layer

These commands are optional compatibility aliases. Any agent can follow
[[documentation-workflow]] directly.

```
/docs-sync     right after a deploy or release -- reads only what changed since verified_at
/docs-lint     integrity check (orphans, broken links, broken sources, stale, unverified)
/docs-query    ask the docs. If it is not there, the answer says so
/docs-status   commits not yet reflected, stale pages, recent work
```

Every maintained page carries `verified_at`, the commit against which it was checked.

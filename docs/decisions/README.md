# Architecture decision records, one file each

From 27 November 2026, each new ADR is a separate file in this directory
(ADR-ADR-FILES-20261127). [`../DECISIONS.md`](../DECISIONS.md) is the
historical register up to that date. Its ADRs keep their IDs and anchors and are
not migrated.

## Convention

- File name: `ADR-<TOPIC>-<YYYYMMDD>.md`, equal to the ADR ID in the heading.
- First line: `# ADR-<TOPIC>-<YYYYMMDD> — <title>`.
- Required fields: `**Status:**`, `**Decision:**` and `**Rollback:**`. An
  optional note block (for example `> Ported …`) may come before `**Status:**`.
- An ID is unique across this directory and `../DECISIONS.md`.
- An amendment to an existing ADR is made in that ADR's own file or section,
  dated and with its source.
- There is no hand-edited index. Each new ADR adds only its own file, so ADR
  pull requests do not conflict with each other. List the ADRs with
  `ls docs/decisions/ADR-*.md`; the date is in the file name.

Ownership is unchanged: ADRs remain lead-owned governance files. They are
protected in the same way as `docs/DECISIONS.md` and need the same independent
review.

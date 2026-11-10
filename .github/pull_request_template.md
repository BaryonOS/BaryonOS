# Change proposal

<!--
Open the PR as a draft first. Complete the relevant sections and remove
instructions/placeholders before finalisation. Retain concrete open points.
Use 'Closes #...' only if this change resolves that issue completely.
No secrets, production keys, real user data or unsanitised log dumps.
-->

## Purpose and change

What concrete problem is resolved, what changes, and what behaviour results?
Link the issue/discussion and relevant ADR or FR/rubric ID. Identify remaining
work when this PR implements only part of an issue.

## Scope and boundaries

Identify relevant source files, interfaces and feature profiles. Describe
changes to capabilities, IPC, process/driver boundaries or storage formats,
including the invariant to preserve and negative checks. The independent kernel
and Security Floor remain authoritative.

- [ ] Relevant `AGENTS.md` files and contract specifications have been read.
- [ ] New `unsafe`, dependencies or reused code/assets: rationale, provenance, version and licence checked; or explicitly not applicable.

## Verification

Mark only checks that were actually run as passed. Distinguish host tests,
target builds, native VM experiments, physical tests and human review.
Include appropriate formatting/linting and positive and negative tests.

| Command or gate | Tested source / profile | Result and exit status | Evidence / run URL |
|---|---|---|---|
| | | | |

**Not run, blocked or not applicable (with reason):**

For native evidence, record the image/tool versions used and the report containing
hashes. Earlier evidence remains tied to its original source. A successful cache,
build or AI review does not replace required runtime acceptance.

## Evidence limits and rollback

What does this change not yet demonstrate? Which concrete risks or unresolved
review findings remain, and how can this change be safely reverted?

## Progress

Affected criterion/FR ID (or none), previous → new score and justification:

The integrator updates the assessment in `docs/progress.json` and the associated
progress displays **for every task, even when the score is unchanged**.
Record `python3 tools/progress.py --write` and `python3 tools/progress.py --check`
and their results in the verification table. Only fully satisfied criteria with
appropriately stored evidence justify new points; record the actual handover
and remaining limits in `docs/PROJECT-STATE.md`.

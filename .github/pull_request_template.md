# Change proposal

<!--
Open the PR as a draft. Complete the relevant sections and remove instructions
or placeholders before marking it ready. Preserve concrete unresolved items.
Use "Closes #..." only when the issue is fully resolved by the PR.
Do not include secrets, production keys, real user data, or unsanitised logs.
-->

## Purpose and changes

What specific problem is addressed? What changes and what observable behaviour does it provide?
Link the relevant issue/discussion, ADR, and FR/rubric ID where applicable. Identify remaining work when this PR delivers only part of an issue.

## Scope and boundaries

Identify relevant source files, interfaces, and feature profiles. Describe changes to
capabilities, IPC, process/driver boundaries, or storage formats, along with the
invariant to preserve and the relevant negative tests. The independent kernel
and Security Floor remain mandatory.

- [ ] Relevant `AGENTS.md` files and contracts were reviewed.
- [ ] New `unsafe` code, dependencies, or reused code/assets: justification, provenance, version, and licence reviewed, or explicitly not applicable.

## Verification

Mark checks as passing only if they actually ran successfully. Distinguish host tests,
target builds, native VM tests, physical-hardware tests, and human review.
Include appropriate formatting/lint checks and both positive and negative tests.

| Command or gate | Tested revision / profile | Result and exit code | Evidence / run URL |
|---|---|---|---|
| | | | |

**Not run, blocked, or not applicable (with explanation):**

For native evidence, provide the image/tool versions and report with hashes.
Previous evidence remains tied to its original revision. A successful cache,
build, or AI review does not replace required runtime acceptance.

## Evidence limitations and rollback

What does this change *not* establish? Which risks or open review findings
remain, and how can the changes be reverted safely?

## Progress assessment

Affected criterion/FR ID (or none), previous → new score, and justification:

The integrator updates the assessment in `docs/progress.json` and associated
progress displays **after every task, even when the score does not change**.
Record the results of `python3 tools/progress.py --write` and
`python3 tools/progress.py --check` in the verification table.
Only fully accepted criteria with adequate evidence justify additional points.
Record the actual handoff and open limitations in `docs/PROJECT-STATE.md`.

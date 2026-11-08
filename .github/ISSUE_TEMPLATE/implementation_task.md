---
name: Implementation task
about: A scoped implementation task with explicit acceptance and evidence limitations
title: "task: "
---

<!--
Use this template once the goal is sufficiently defined. Discuss unresolved
product or architecture decisions in Discussions before creating an actionable issue.
Read AGENTS.md, relevant local instructions, and docs/TASK-TEMPLATE.md.

The metadata block below is only for tasks accepted into the autonomous pipeline.
Remove it for manual maintenance tasks with no open rubric criterion, and
describe the checks under Acceptance and evidence instead.

Pipeline parser keys MUST remain exactly "Criterium:", "Gate:", and optionally
"Afhankelijk van:". Set Criterium to one open ID from docs/progress.json.
Gate contains one or more existing workflow.yml#job references, separated by
spaces. These are planned checks, not results. Include "Afhankelijk van: #12 #34"
only when real dependencies exist. Remove this guidance before submitting.
-->

## Pipeline metadata

```text
Criterium: INSERT_OPEN_ID
Gate: INSERT_WORKFLOW.yml#INSERT_JOB
```

## Goal and context

What specific problem is solved by this step? Link the relevant discussion,
ADR, FR acceptance ID, and existing issues/PRs. Avoid duplicating implementation work.

## Scope and interfaces

- Base commit and relevant source files:
- Allowed files to modify and owner of shared interfaces:
- Invariant to preserve, capability/privacy boundary, and dependency or format implications:
- Out of scope:

## Acceptance and evidence

- Observable result and exact commands/gates to run:
- Appropriate positive and negative cases, including failure/recovery where applicable:
- Required evidence level: host, native VM, physical, or human review:
- Recorded source/image revision, exit codes, and accessible log/report/hash references:
- What this step does not prove and how to revert it:

## Execution

A maintainer reviews the scope, dependencies, and gates and, where appropriate,
assigns exactly one label: `pipeline:serial` or `pipeline:parallel`.
This template does not start an agent, assign a label/owner, or award progress points.
Changes to pipeline infrastructure, workflows, or other protected files require
a separately reviewed PR, not the autonomous pipeline.
Use synthetic test data only. For potential vulnerabilities, follow
[SECURITY.md](https://github.com/simonvanieperen/BaryonOS/blob/main/SECURITY.md).

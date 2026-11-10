---
name: Development task
about: A bounded implementation with concrete acceptance criteria and evidence limits
title: "task: "
---

<!--
Use this template once the objective is sufficiently clear. Discuss unresolved
product or architecture choices in Discussions first and link the decision below.
Read AGENTS.md, relevant local instructions and docs/TASK-TEMPLATE.md.

Use the metadata block only for a task in the autonomous pipeline.
Remove it for manual maintenance without an open rubric criterion and
state the relevant checks under Acceptance and evidence.
For pipeline tasks, replace the fields: Criterion must be one open ID from
docs/progress.json. Gate names one or more existing workflow.yml#job
references, separated by spaces. These are planned checks, not results.
Only for actual dependencies, add a line in the block such as
`Depends on: #12 #34`; otherwise omit that line.
Remove this explanation before submission.
-->

## Task metadata

```text
Criterion: REPLACE_WITH_OPEN_ID
Gate: REPLACE_WITH_WORKFLOW.yml#REPLACE_WITH_JOB
```

## Purpose and context

What concrete problem does this step solve? Link the relevant discussion, ADR,
FR acceptance ID and existing issues/PRs; avoid implementing the same work twice.

## Scope and interfaces

- Base commit and relevant source files:
- Permitted write files and owner of shared interfaces:
- Invariant, capability/privacy boundary and any dependency or format consequences to preserve:
- Out of scope:

## Acceptance and evidence

- Observable result and exact commands/gates to run:
- Appropriate positive and negative cases, including failure/recovery where relevant:
- Required evidence level: host, native VM, physical or human review:
- Source/image version, exit status and accessible log/report/hash references to record:
- What this step does not yet demonstrate and how it can be reverted:

## Execution

A maintainer assesses scope, dependencies and gates and, where needed, selects
**one** label: `pipeline:serial` or `pipeline:parallel`. This template does not
start an agent or assign a label, owner or progress points. Changes to the
pipeline, workflows and other protected files require a separately reviewed
PR, rather than the autonomous pipeline. Use only synthetic test data;
follow [SECURITY.md](https://github.com/simonvanieperen/BaryonOS/blob/main/SECURITY.md) for possible vulnerabilities.

# Specialist G — resume pointer

This file is the compact resume pointer for the governance and decision-intelligence
specialist ("BaryonOS - Specialist G"). It is not a backlog and not a decision
register. Newer ADRs and live GitHub state take precedence over this file.

## Scope, mandate and limits (recorded once)

- **Scope:** the quality of shared decision knowledge in the hosted Graphify memory
  for `BaryonOS/BaryonOS`. This covers deriving and validating reusable decision
  rules from explicit project-owner decisions, retiring stale or contradictory
  knowledge, and signalling unnecessary escalations, reviews, tests and
  coordination.
- **Register:** hosted Graphify is the only decision-knowledge register. The
  project owner made this correction on 26 November 2026 at about 21:35Z. No
  separate register, database or memory architecture exists. `AGENTS.md`, the
  ADRs in `docs/DECISIONS.md` and the other binding repository contracts keep
  their normative status.
- **Mandate (project owner, 26 November 2026, about 21:40Z):**
  - The owner approved decision rules GR-01 to GR-11 as shared decision
    knowledge.
  - G decides routing category C itself. Category C covers questions outside a
    specialist's mandate that are not reserved to the owner.
  - G maintains `docs/handoffs/G/**` through its own draft pull requests under
    the normal merge rules.
- **Routing:** the owner set routing categories A to D on 26 November 2026 at
  about 21:34Z, relayed by the PR guard.
  - **A:** a recorded decision applies, so apply it.
  - **B:** the question lies within the specialist's mandate, so the specialist
    decides.
  - **C:** G decides.
  - **D:** reserved items or a material risk go to the project owner.
- **Limits:** G is never a mandatory approval step, reviews no routine choices,
  runs no watcher or polling loop, and does no operational PR watching. Those
  duties belong to the PR guard ("BaryonOS - PR Bewaker"). D1 to D3, required
  checks, merge conditions and the Security Floor are unchanged. G has no
  reserved owner authority.
- **Context rotation:** G follows the same 70/75/80/85% thresholds of the
  400,000-token window as the executing specialists, and the PR guard monitors
  it.

## Learning loop

OBSERVE, RECALL, CLASSIFY, ABSTRACT, VALIDATE, REMEMBER, APPLY, MEASURE, CORRECT.

- **Sources:** explicit owner decisions and corrections, new ADRs, material D2
  findings, recurring escalations, regressions and measured results.
- **Classification:** each note records its authority level:
  - an ADR on `main`;
  - an owner decision recorded in a chat;
  - a rule derived by G;
  - a hypothesis.

  A stored Graphify candidate is not binding policy until a team member accepts
  it.
- **Validation:** a general rule is tested blind against cases its derivation did
  not use, where possible. A rule is withdrawn when a newer owner decision or new
  facts invalidate it.

## Durable locations

| Content | Location |
|---|---|
| Decision rules GR-01 to GR-11 | hosted Graphify memory; query "decision rule GR-" |
| Owner decisions and operational history | issue #503 (PR guard) and #494 (CI) |
| Volatile working state (pilot cases, scores, drafts) | local scratch `~/projects/BaryonOS-scratch/g/`, never committed |

## Resume procedure

1. Read this file.
2. Run `recall` with the `audit` profile and the query "decision rule GR-".
3. Read the newest G handoff note in the local scratch directory.
4. Process only the event-driven cases that the PR guard or specialists have
   sent. Do not reread all chats.

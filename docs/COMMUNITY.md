# Discussions, reports, and contributions

Use [Discussions](https://github.com/simonvanieperen/BaryonOS/discussions) for questions, proposals, and experiences. Use [Issues](https://github.com/simonvanieperen/BaryonOS/issues/new/choose) for reproducible defects and bounded implementation work. Search for an existing topic before opening a new one.

## Choose the appropriate channel

| You want to… | Place | Helpful information |
|---|---|---|
| Ask a question or get help | [Questions and help](https://github.com/simonvanieperen/BaryonOS/discussions/categories/vragen-en-hulp) | Goal, context, and what has already been tried |
| Evaluate a technical choice | [Architecture and design](https://github.com/simonvanieperen/BaryonOS/discussions/categories/architectuur-en-ontwerp) | Current contract, proposal, alternatives, interfaces, and testable consequences |
| Improve a user task or accessibility | [Product and user experience](https://github.com/simonvanieperen/BaryonOS/discussions/categories/product-en-gebruikservaring) | Intended task, observed friction, and recognizable examples |
| Share an experiment, demo, or test | [Test results and demos](https://github.com/simonvanieperen/BaryonOS/discussions/categories/testresultaten-en-demos) | Exact source revision, setup, observation, evidence, and remaining boundaries |
| Collect preferences between concrete options | [Polls](https://github.com/simonvanieperen/BaryonOS/discussions/categories/peilingen) | A clear question and comparable options; voting does not replace a technical decision or acceptance evidence |
| Share project news or an accepted decision | [Announcements](https://github.com/simonvanieperen/BaryonOS/discussions/categories/aankondigingen) | Maintainer post with links to the authoritative source |
| Report a defect, CI problem, or concrete task | [Issue forms](https://github.com/simonvanieperen/BaryonOS/issues/new/choose) | Reproduction or bounded result, source revision, and appropriate verification |

Stay on topic, support technical statements, and distinguish proposals, code, executed tests, and assumptions. **English is the project language for active public documentation and community communication.** Technical identifiers remain unchanged.

## From discussion to implementation

1. Discuss the problem, options, and boundaries in an existing or new discussion.
2. Record an accepted architecture or contract change in [DECISIONS.md](DECISIONS.md) and the relevant specification. A proposal or poll is not a decision by itself.
3. Create or link one executable issue with result, scope, dependencies, acceptance commands, and relevant existing FR/rubric IDs. Discussions are not a second backlog or roadmap.
4. Work on a small branch and open a draft pull request according to [CONTRIBUTING.md](../CONTRIBUTING.md) and [AGENTS.md](../AGENTS.md).
5. Bind evidence to the exact tested source revision. Record failed and not-run checks as well as successful ones.

Host tests, bare-metal builds, native QEMU tests, physical-hardware tests, and human review prove different things. A screenshot shows rendered output but does not prove storage durability or enforced permissions. Partial tests and demos earn rubric points only when the complete existing criterion is accepted with the required evidence.

## Authoritative project sources

| Topic | Source |
|---|---|
| Current state and next step | [PROJECT-STATE.md](PROJECT-STATE.md) |
| Complete release task and acceptance | [FULL-TEST-RELEASE.md](FULL-TEST-RELEASE.md) |
| Fixed development criteria and progress | [PROGRESS.md](PROGRESS.md) and [progress.json](progress.json) |
| Architecture and security boundaries | [KERNEL-AND-SECURITY-CONTRACT.md](implementation/KERNEL-AND-SECURITY-CONTRACT.md) and [DECISIONS.md](DECISIONS.md) |
| Privacy, NAS, internet, and local AI | [LOCAL-PRIVACY-CONTRACT.md](LOCAL-PRIVACY-CONTRACT.md) |
| Product experience and desktop | [DESIGN-AMBITION.md](DESIGN-AMBITION.md) and [DESKTOP-SHELL.md](implementation/DESKTOP-SHELL.md) |
| Tests, builds, and source-bound evidence | [TEST-STRATEGY.md](TEST-STRATEGY.md) and [DEVELOPMENT.md](DEVELOPMENT.md) |
| Provenance and licensing | [LICENSING.md](../LICENSING.md), [THIRD-PARTY.md](../THIRD-PARTY.md), and [DEPENDENCIES.md](DEPENDENCIES.md) |

The repository remains authoritative. Discussions and the wiki should point to these sources rather than duplicate version-bound contracts or evidence reports.

## Vulnerabilities and sensitive data

Follow [SECURITY.md](../SECURITY.md). Do not put secrets, production keys, personal data, or exploitable vulnerability details in public issues, discussions, or logs. General architecture discussions about security are appropriate for Discussions.

## Maintainer workflow

Issue forms never add `pipeline:serial` or `pipeline:parallel` automatically. These labels are inputs to the existing [agent pipeline](../tools/agent_pipeline.py) and are assigned only after maintainer review. A message or form submission is not an automatic execution request.

A pipeline issue has exactly one pipeline label, one open rubric criterion, and existing workflow jobs. Its body uses:

```text
Criterium: <open ID from docs/progress.json>
Gate: <workflow>.yml#<job>
Afhankelijk van: #<issue-number>
```

The dependency line is optional; `Gate:` may contain multiple job references separated by spaces. Pipeline labels and technical change boundaries remain under maintainer control.

The four discussion forms are stored under `.github/DISCUSSION_TEMPLATE/`. Their filenames follow the existing category slugs. Renaming a category therefore also requires checking the matching form and links above.

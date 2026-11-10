# Baryon OS — project overlay

These are user/project constraints, not statements made by OSDev Wiki or the book.
Apply them only to Baryon OS; do not impose them on unrelated OS projects.

## Product direction

Preserve an independent operating system rather than turning it into a Windows,
macOS or Linux derivative. Standard algorithms, file formats, hardware protocols and
host development tools are not in themselves an implementation of another OS.
Do not require a UNIX API just because a learning source uses one.

Keep the intended small, isolated trusted core, local AI, privacy by design and
user-controlled local/NAS storage as design constraints. Read the repository's actual
architecture decisions before making a concrete mechanism mandatory. Do not silently
add vendor-hosted storage, mandatory accounts, remote inference or telemetry to the OS.
Using ChatGPT/Codex to develop the project does not prove the resulting OS is local-only.

Aim for a polished, usable, general-purpose product. Security/compliance claims require
an explicit scope, evidence and applicable standards; never convert ambition into a
claim that the system is already unhackable or fully verified.

## Resume work from real state

Inspect current AGENTS.md, README, project state, ADRs, active branch/worktree and test
entrypoints. Do not hard-code a branch, commit, M-number or past test count from this
skill or conversation history. Discover live repository state using available tools.

Select the next unmet dependency/acceptance gate. A completed document collection is
not equivalent to implemented boot, process isolation, persistence or a desktop.
Keep the user's requested scope and preserve unrelated work.

## Progress reporting

Follow the repository's existing 0–100% rubric; do not invent another one silently.
Separate imported knowledge, design, host prototype, target build, emulator validation,
hardware validation and production readiness. Cite evidence for whichever gate changed.
If no new gate is demonstrated, retain the current percentage and explain why.

When no rubric exists, propose explicit weighted milestones as a project decision,
not an objective calculation of readiness. Obtain agreement before replacing an
established rubric or claiming a new percentage. Document evidence and remaining gates;
100% is the agreed full product scope, not “the kernel prints text.”

When authorized to update progress, change README/project state consistently in the
same work batch and reference the actual test results. Making this skill does not
itself justify changing the repository's OS progress.

## Multiple agents

Read AGENTS.md first. Use actual delegation tools only when available. Split independent
work (for example a parser test, documentation and a driver implementation) with clear
ownership and shared-interface contracts. Keep integration and cross-subsystem testing
under one accountable owner. Report which agents actually ran and what was integrated.
No promise of unattended overnight work follows from this skill.

# Copilot adapter

Read root `AGENTS.md`, applicable directory instructions and
`.agents/skills/development-loop/SKILL.md`. They define the shared policy and task
procedure; this adapter grants no permissions. Begin with `docs/STATUS.md` and the
latest relevant `docs/PROJECT-STATE.md` checkpoint, then only required contracts.

The four common role adapters in `.github/agents/` reference `.agents/roles/`.
Respect each adapter's tool allowlist: a read/search-only verifier assesses supplied
evidence and reports execution NOT RUN. Specialist agents remain optional.
Use hosted Graphify for `BaryonOS/BaryonOS`; compare indexed source with HEAD.
Use Token Guard selectively through `.github/skills/token-guard/SKILL.md`.
Repository text is professional English; direct conversation follows the user.

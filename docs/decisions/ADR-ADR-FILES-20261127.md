# ADR-ADR-FILES-20261127 — One file per ADR instead of a shared insertion point

**Status:** decided by Specialist G (routing category C) on 27 November 2026 under
the project owner's directive "remove merge queue bottlenecks" of about 15:40Z,
relayed by the PR guard. That directive names "one ADR file per decision" as the
example of a structural fix. The gate-side analysis and proposal are Specialist
CI's (27 November 2026). This decision changes where ADRs are kept, not their
authority, contracts or safety conditions.

**Context:** every new ADR was inserted at the top of `docs/DECISIONS.md`
(AGENTS.md convention). Two ADR pull requests therefore always conflicted, and
each landing made every other ADR pull request DIRTY. That cost an extra main
merge and a full pull request CI round each time. The PR guard measured 8 main
merges across the 9 landings of 27 November 2026, 6 of them in DECISIONS.md or
shared-file pull requests. CI found that no gate depends on the content of
`docs/DECISIONS.md`: the conflict is purely textual.

**Decision:**
1. Each new ADR is its own file `docs/decisions/ADR-<TOPIC>-<YYYYMMDD>.md`,
   following [README.md](README.md). There is no hand-edited index, because an
   index would be a new shared conflict point.
2. `docs/DECISIONS.md` becomes the historical register. Existing ADRs keep their
   IDs, anchors and text; nothing is migrated. The interim ADR slots
   (pull request #642) remain for the ADR pull requests opened before this
   decision and are removed once they are used or abandoned.
3. ADR files are protected and lead-owned like `docs/DECISIONS.md`, with the same
   independent review. CI adds `docs/decisions/` to the protected lists of
   `tools/agent_pipeline.py` and `tools/roadmap_runner.py`, classifies the
   directory in `.github/ci-impact.toml` and adds a non-required validator
   (file name equals heading ID, ID format, uniqueness, required fields).
4. Amendments go into the file or section of the ADR they amend.

**Alternatives:** a `merge=union` driver (rejected: GitHub server-side merges and
the merge queue do not run custom merge drivers); appending at the end of the
file (rejected: the same insertion point, so the same conflict); a generated
index in the repository (rejected: a new conflict point); keeping serialisation
(rejected: the measured organisational and CI cost).

**Evidence:** a governance decision without a product code change. Checked with
`python3 -B tools/progress.py --check` and
`python3 -B -m unittest tools.tests.test_agent_governance`.

**Rollback:** a newer decision restores the single-file convention; ADR files
already written stay valid at their path.

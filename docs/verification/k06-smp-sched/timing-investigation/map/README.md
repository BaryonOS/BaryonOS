# K06 timing map: working notes, 12 November 2026

Not acceptance evidence. Read-only AI agents produced these files against commit
`ec9a8b8` (kernel and tool sources of `2543861`) to prepare the structural timing repair
assigned by the project owner. Code citations are `path:line` at that commit;
`specs/...` refers to the files on branch `docs/k06-step-specs`. A design may rely on a
claim only after it has been checked against the source again.

| File | Content | Status |
|---|---|---|
| `K06A-SPEC.md` | Normative map: which timing limits and acceptance rules are binding, where they are written and which code implements them | Complete; verified by its author agent only |
| `K06A-SITES.tsv` | Inventory of wall-clock and time-proxy limits in the kernel fixture, 92 rows | Table only: the agent stopped at a usage limit before its report and completeness check, so treat it as incomplete |
| `hosted-margins.tsv`, `hosted-markers.tsv` | Duration over watchdog and marker extremes per artifact, from all 76 artifacts of rejected run 43187501026 (3 301 boots) | Derived with `hosted-margins.py`; not recomputed independently |
| `hosted-margins.py`, `hosted-artifacts.tsv` | Extraction script and the artifact list it was run on | Supporting material |

Not produced, because the mapping run stopped at a usage limit: the oracle and workflow
map, the clock path and emulator semantics, the lock handoff and liveness analysis, the
quantitative analysis and the completeness review.

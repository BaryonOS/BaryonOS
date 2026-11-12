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

A second read-only run on 12 November 2026 against commit `67e592c` (the same kernel and
tool sources) added three reports. Where they mention a scratchpad `specs` tree, that is a
session-local copy of branch `docs/k06-step-specs`. They predate the margin logging and
the clock edge accounting, so their line numbers and their "does not exist" statements
describe the source before those changes.

| File | Content | Status |
|---|---|---|
| `K06B-ORACLE.md` | Serial oracle, workflow, verifier and evidence chain: what each layer checks and what a new line or case must touch | Complete; verified by its author agent only |
| `K06B-CLOCK.md` | Clock path in the kernel and in the QEMU 8.2.2 and 10.2.1 sources, and an attack on the first edge-accounting design | Complete; its objections led to the `smp_lock::clock` design |
| `K06B-LOCK.md` | Liveness map of the contended phases, where the 400 ms lock bound can fire, and an attack on the progress-window candidate | Complete; verified by its author agent only |

Not produced: the quantitative analysis and the completeness review. The review agent
stopped at a usage limit twice.

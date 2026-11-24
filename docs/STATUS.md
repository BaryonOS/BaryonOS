# Current overview

Updated 24 November 2026. Verified prior-main baseline:
`c3afc04ac2bc92228f2201af02ab69148c47e596` ([#478](https://github.com/BaryonOS/BaryonOS/pull/478)).
The final UI/input contribution is included in this source. The acceptance score
remains **41/100**; criterion states, weights and accepted evidence are unchanged.
Read the [current handoff](PROJECT-STATE.md) for source mappings and continuation.

| Area | Incorporated source PRs | Evidence boundary |
|---|---|---|
| Security and boot — #476 | #427, #457, #470, #456, #434 | B01/B04 evidence remains source-bound; ordinary B03 CI does not establish B04 trusted-boot acceptance. |
| Governance — #479 | #452, #473, #465 | Delegated PR completion and manual-session repair; merge-group support is preparation, not an operational queue. |
| Volume and restore — #477 | #407, #408, #411, #459, #462, #413, #414 | Explicit recovery across INSTALL; uncertain COMMIT/collection replies can hide durable changes. The final batch passed its native volume and NAS gates. |
| Network — #478 | #426, #440, #428, #431, #444, #468, #474, #445 | Final native extension and baseline gates passed. #380's failed prototype is replaced by the separate bounded full-port-sweep successor. |
| UI, settings and input — included here | #436, #441, #455, #463, #469, #472, #461 | Exact-head native acceptance is recorded on the containing integration PR. Search S3 and the read-only service-refusal walkthrough remain separate work. |

The issue-driven controller remains disabled under `EXECUTION_HOLD`. Queue
activation still requires verified predecessor-rerun handling, ruleset/check
assumptions and actual operation. Manual-session repair does not activate either.

Five UI cases remain explicitly ignored: four new defect specifications from
#472 and the inherited History reading-order case. #472 delivers its test matrix;
it does not implement those production fixes. Physical hardware, human task
studies and accessibility acceptance also remain open.

Start from freshly fetched main, AGENTS.md and the current handoff. Compare old
branches with current files, preserve child deltas across squash merges, and use
one writer per branch or shared file. Continue independent work while PR checks
run, or construct a deliberate dependent branch. Historical handoffs describe
their original sources and do not reserve ownership. Proposals remain proposals.

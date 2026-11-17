# Phase 0 cloud handoff — 18 November 2026

**Ownership: PREPARED, NOT RELEASED.** A cloud writer must wait for the verified
release commit on `codex/phase0-baseline`. This handoff transfers Codex work only;
it does not transfer the independent Claude UI/UX or S09 workstreams.

## Source and authority

- Verified integration main: `38d4cc17687156379677993be9945447591bd4c1`.
- Primary security source: `aec1fb5f75e6165da82ba81f3f43d7a0d912ab1d`,
  branch `codex/phase0-pipeline-isolation`, existing draft PR **#355**.
- This checkpoint's parent: `3747ba2b2496c89c02fd835161919589b36a5cc9`.
- Score remains **41/100**. Baseline B is not established. Governance 1–6 have
  not started. Only **0D security remediation** is the primary active milestone;
  0A is complete, 0B waits on security, and the earlier 0C consistency review is
  historical evidence rather than final integrated acceptance.

The owner authorizes necessary investigation, scoped fixes, new/update draft PRs
and independently verified **manual squash merges** throughout Phase 0. Separate
routine merge permission is unnecessary. No auto-merge, protection/hook bypass,
false closure, suppressed scanner coverage or weakened acceptance is authorized.
Close the Daybreak finding only after the repaired main and evidence are verified.

The current finite B gate is [owner-objective-refinement.json](owner-objective-refinement.json),
superseding the historical zero-count prerequisite. Every residual needs evidence,
owner, priority and a concrete next step. Long-term product work and nonblocking
quality do not delay governance by count alone. Security/integration blockers do.
The [latest-stable policy](owner-latest-stable-policy.json), its accepted
`ADR-VERSION-POLICY-20261117` and procedures in `docs/DEPENDENCIES.md` remain binding.
Do not repeat the completed stack inventory or restart the approved planning.

## Primary security work

PR355 contains pre-L0 owner-manifest authority, disposable credential-free worker
copies, immutable gate inputs, restricted Linux execution and separate trusted
publication. Its committed manifest remains **disabled**. No vulnerable issue
pipeline was used to develop the remediation.

Independent review statically verified three source repairs. It still blocks
merge/enablement on **resource bounds and complete selected client/gate
compatibility**. This is an unfinished fix, not an accepted production runtime.
The normal host-quality job needs network preparation, Git metadata and explicit
cache writes that the current offline evaluation profile does not provide.
Do not omit those checks or add ambient credentials/network as a fallback.

Evidence at the security checkpoint:

| Observation | Result and limit |
| --- | --- |
| Complete local tools suite at the initial checkpoint | 406 tests, exit 0, including ten actual offline/isolation fixtures; not real authenticated model-client parity. |
| Final metadata compatibility regressions | 78 targeted tests, exit 0, at the current source. Fixed absolute tools and explicit no-lazy-fetch handling remain in place. |
| Fresh sandboxed QEMU TEST-policy replay | Existing 26-case oracle passes; raw QEMU exit 33; immutable inputs and store preserved. Cached main guest, no guest rebuild or hardware acceptance. |
| First hosted head `2c78fb48` | host-quality failed in a metadata fixture; preserved and repaired, not hidden. |
| Current hosted head `aec1fb5f` | Running at handoff preparation; re-query before acting. No pending check is a pass. |
| Daybreak | Finding remains open. The owner's latest completed run scanned unchanged main38, not the repair. |

The sanitized receipt is on the security branch at
`docs/verification/phase0-pipeline-isolation-20261117/checks.json`.
Complete private reviews and 55 retained evidence artifacts are in owner-managed
Codex Security storage. Evidence-manifest SHA256:
`1e7f7fdaf9956f1e003293accc9f397067d5b186697601d9c3d9ec5f6d68d42a`.
Private finding content and adversarial material must not be published.

Daybreak's usage metadata reported unavailable/credits-exhausted, but **no launch
rejection was established**. Do not treat it as a proven permission/billing block.
The service exposes a launch function; reuse the latest run's discovered settings
and confirm the actual scanned SHA. Avoid another duplicate unchanged-main run.
After eligible integration, run a source-bound check and use the supported finding
resolution mechanism; verify status rather than assuming that a merge closes it.

## Existing branches and workstreams

| PR / purpose | Exact published checkpoint | Disposition |
| --- | --- | --- |
| #355 security | `codex/phase0-pipeline-isolation` — `aec1fb5f75e6165da82ba81f3f43d7a0d912ab1d` | Primary; blocked as above. |
| #354 Rust 1.100 | `codex/phase0-rust-1-100` — `aa5015378e78783fe6486480be45fdfd44656cfc` | Four compiler-WIP files preserved and formatter checked. Full Clippy/native/current-main acceptance open. Temporary base is `codex/phase0-pr340-preintegration` at `f5a976f17a3167a08ad513f0328cba8de3dd8a62`; retarget/reconcile main before any merge. |
| #333 native input | `feat/issue42-native-input` — `db4581c11333448ad70e757b41329ef1f67b7b60` | Prior source-bound native/independent evidence retained. Re-query current checks and reconcile resulting main after security. Issue42 remains open. |
| #340 limited fix | Canonical `feat/s09-ustar-name-check` — `90816de39460b76e35cd626721d4dac9bde905e2`; preview `codex/phase0-pr340-preintegration` — `f5a976f17a3167a08ad513f0328cba8de3dd8a62` | Limited Clippy/name-claim release only. Broader S09/D1–D5 and runners remain Claude-owned. Retain preview because PR354 uses it. |
| #339 claim correction | Canonical Claude head `1a2758368bf416205e6e4b2787d7304397ff4daa`; Codex preview `codex/phase0-pr339-preintegration` — `bba804996b93d93e82b6fb323cdeea9779f6fe99` | Preview merge completed. Every non-checkpoint file equals latest Claude; only status/assessment/receipt differ. Do not overwrite a live Claude writer. No duplicate PR. |
| #346 consolidated English docs | `codex/phase0-ci-guide-english` — `04e29181ef7e755eea2db0054c463b9216184671` | Main38 reconciled, 39 preserved source/evidence blobs. Source PRs347–349/351–353 stay open until merged inclusion is proved. |
| #350 OSS review config | `codex/coderabbit-oss-config` — `7b4b618e7ef0eee6b2a67ce693afcb9f2f20809c` | Configuration/schema evidence retained; current-head gates/reviews still apply. No paid features/account changes. |
| #345 inventory/current checkpoint | `codex/phase0-baseline` | This handoff; reconcile latest main without losing accepted policy/decisions. |

The obsolete #346/#350 preview refs are absent; their work is in canonical
branches. The #339 and #340 previews remain purposeful, associated with existing
PRs. Do not create duplicates or delete them before inclusion/dependency proof.

Open issues at preparation: **#41, #42, #43, #274, #283**; open PRs: **14**.
#343/#344/#336 were integrated with their recorded gates; #277 was actually
completed and closed. Preserve the [residual dispositions](README.md): production
update custody, general input/hardware acceptance, real 50/100 criteria, complete
translation/publication and unresolved rights evidence are not complete merely
because a slice or associated PR is merged. Current final security-category counts
are NOT VERIFIED; old API zeros are not final-source acceptance. Keep Project5
synchronized through its existing automation.

## Local-only work and ownership

The read-only inventory covered 37 worktrees. Original shared main remains at
`c07506c85ce30ad9177e2e2db4118fe48e867a5c` with a local merge of main38 and 924
status paths; this includes incoming tracked changes and does not mean 924 new
unpublished files. Original UI/UX has seven dirty paths; agent-test has two.
These user/Claude-owned files, local index/merge state, locks and runners are
**not transferred**. Do not reset, clean, stash, rebase or publish them.

Private evidence, compiled sandbox/runtime images, native caches, installed
toolchains/plugins, home settings and the full previous chat are not in GitHub.
Codex Cloud cannot assume filesystem or MCP access to them. Source/test recipes
and safe receipts are published; unavailable private evidence or execution
surfaces remain NOT RUN/NOT VERIFIED. No credentials are transferred. Local
inventory SHA256: `42db5222712e977c5b57a401058ab443f415ede74edc7d3004bc70f56af44504`.

All bounded helper agents have stopped writing. This alone does not complete a
project milestone or release their parent workstream. Only the verified release
commit changes the Codex ownership status above; Claude/user ownership persists.

## Next executable action in Codex Cloud

1. Fetch this release commit, verify the exact branch/head, read root and relevant
   directory instructions plus this handoff, then record one exclusive claim for
   PR355. Do not start a second writer or invoke either autonomous controller.
2. Check out the existing `codex/phase0-pipeline-isolation` source above, confirm
   clean state and current remote/CI/review results. Resolve the documented
   resource and complete workflow/client blockers using supported isolation;
   never turn the disabled manifest on as a shortcut.
3. Reuse existing test entries and receipts. Actual Linux fixture tests use
   `BARYON_SANDBOX_TEST_CONFIG=<reviewed-runtime-json>`; absence is NOT RUN.
   Provision credentials-free supported tooling only when needed and verified.
   Require independent review and current-head CI, then manually squash only if
   all conditions pass. Verify new main and Daybreak status before closure.
4. Resume the existing integration queue, issue dispositions, urgent supported-
   version fixes and final integrated assessment. Freeze B only after its real
   gate passes; then continue the approved governance phases. Do not reconstruct
   missing historical evidence, invent the approved plan or restart research.

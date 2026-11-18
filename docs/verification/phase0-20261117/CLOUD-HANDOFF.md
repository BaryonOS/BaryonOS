# Cloud-to-Local handoff — 18 November 2026

**Execution: PAUSED. Cloud development has ended. The next executor is a new
Local Codex session on the owner's Ubuntu workstation.** No successor task was
scheduled or launched. This transfer does not resume implementation, merge any
PR, close any issue/finding, enable the issue pipeline or establish baseline B.

Recorded: `2026-11-18T11:29:00+01:00`. Current main:
`de9b99047138c4fd0015ffc3f874b273b3537484`. Inventory: **8 open PRs, 5 open issues,
3 unresolved Daybreak findings**. Score remains **41/100** with the fixed rubric.
The handoff checkpoint branch is `codex/cloud-to-local-handoff-20261118`; its immutable SHA
and URL are supplied in the final verified receipt. Existing PR #356 separately
references the transferred source commit `0617b07ccf5f48a7c40c72c87b365c436bfea775`
(tree `a99dd67359f8dd485e4779fe3018e40bea5f3a28`). This separate handoff-only
branch preserves that candidate without turning transfer metadata into merge acceptance. Machine-readable evidence is
in [cloud-to-local-20261118](cloud-to-local-20261118/inventory.json).

## Transferred source and acceptance boundary

The content candidate before this handoff is `06a6230c2765be2a0125d800d39681f2eddf8d3f`,
tree `b8d133c930000af6b6fb24d62302bef5014c0da7`. It contains the original
published preview and all reviewed Codex integration history; normal pushes
preserve those original commits. The last executed source is
`d16fa0862a42d83f7de7fda8dd2de54357d3e533`, fingerprint v3
`42163e2d99138eecda0977feab534c5eebb9f77bc25a2e04abd5f2d28752be92`.
Subsequent differences are documentation, audit evidence and handoff packaging;
all non-documentation runtime/test blobs remain identical. The CPUID cleanup
only removes two obsolete unsafe block markers and comment prefixes; all
maximum-leaf, MAXPHYADDR, firmware-read and PAT/MSR contracts remain intact.
The later quarantine-test cleanup only qualifies unittest.mock imports and
references; no guard, assertion or test vector changes.

| PR | Existing branch | Source head before the handoff publication | Disposition |
|---|---|---|---|
| #340 | `feat/s09-ustar-name-check` | `8609a4d1cd91223202b1043f5486ff8746eef583` | Included in candidate; approved D6(a) ustar subset only; full S09/D1–D5 unaccepted |
| #350 | `codex/coderabbit-oss-config` | `04e464963cd0d969f8687208404f480f289d54a4` | Exact reviewed free-tier configuration; no billing change |
| #354 | `codex/phase0-rust-1-100` | `f0b4b6ed1f93012ff4bcfbeec7787de6eaa736da` | Included with reviewed current-main reconciliation `f2a6ad4970433b0da8fd648c6eaba45ff919f517`; PR itself still has historical stacked base |
| #355 | `codex/phase0-pipeline-isolation` | `5b946202c9f7323044913a26d76c96e490f5e0e6` | Preserved, excluded from combined candidate, security acceptance blocked |
| #356 | `codex/phase0-pr339-preintegration` | `0617b07ccf5f48a7c40c72c87b365c436bfea775` | Safe source transfer verified; draft, unaccepted; no Cloud merge |
| #357 | `codex/phase0-pr355-resource-wip-untested` | `8a32bc582790fa16234dbadf37a0a176de620900` | Untested recovery-only source, excluded, not superseded or accepted |
| #358 | `codex/phase0-english-coverage` | `4fae439d8492aff4769c1c208332e3dafdf38f4b` | Exact original commit normally pushed and included; helper repairs, English routes and versioned inventory |
| #359 | `codex/security-pipeline-quarantine` | `b5f756a233bbfef0ecc143f41834899b399e3805` | Included; unconditional legacy-controller hold plus import cleanup; not full isolation remediation |

Main already contains genuine verified squash merges #345 at
`55bc7b06d20a3bfdffdc2eb7d8fe42a39f6e59c5`, #346 at
`67d5859d735836331b6fb971eae89162b7fa8465`, and #339 at current main above.
Actual trees were compared with their reviewed candidates. Source PRs
#347–#349/#351–#353 were closed as incorporated after actual-main proof, not
reported as separate merges. Original closed-PR heads remain accessible through
`refs/pull/345/head` (`f14f7c946e10120ad10a34885551f2656f84b059`),
`refs/pull/346/head` (`59c07e88abdd1b3a6d06a44ea1c8fe27fdbb5ce4`) and
`refs/pull/339/head` (`7213e22416e6df39b64d6b53e9b3c53ce9b116f1`).
The old #345/#346 head branch names are absent; do not invent their existence.
The original requested handoff source `e7752fe0` remains historical source,
not the current execution checkpoint.

## Completed verification, exact scope and remaining checks

On source `d16fa0862a42d83f7de7fda8dd2de54357d3e533`:

- All 18 local executable host-quality steps passed: raw wrapper exit 0,
  300.233 seconds, transcript and receipt hashes preserved.
- All 15 real QEMU volume stages passed: raw wrapper exit 0, 303.022 seconds;
  twelve raw guest exits 33 and three intentional live host power cuts -9.
- Both real DocumentSession normal/cold-reopen cases passed: wrapper exit 0,
  19.009 seconds, both guests raw33, source/artifact/disk/serial bindings retained.
- Six H02 cases passed: twelve build/run commands exit0, six real VMs raw33,
  no timeout, 26.202 seconds, 25 loaded artifacts independently rehashed.
  Actual USB transport and synthetic protected-HID IPC remain separate proofs.
- The exact content tree at `06a6230c2765be2a0125d800d39681f2eddf8d3f` received independent source,
  artifact, disk/serial oracle and language-inventory review with no blocking
  finding in that scope. Raw review logs are preserved in the subordinate receipt.

[Combined results](../phase0-combined-20261118/README.md) and
`checks-d16fa086.json` preserve raw exits and lossless logs; earlier 44f/2a
receipts remain source-bound rather than relabelled. H02 results are in
`../issue42-combined-20261118-d16fa086/`. Selected generated-prose audit
`../issue274-generated-prose-audit-20261118/` preserves 25 source hashes,
11 successful checks/60 tests, English rendered progress/community fields and
intentional parser/anchor exceptions. Its bounded scope is not a whole-product
language or browser/pixel acceptance claim.

The versioned English inventory at frozen source
`2073e4affd8a46a4bd0a1b95bcb06f7c2b0062c2` reproduces 398 first-party
documents, 179 navigation nodes, 11 compatibility anchors, 34 English routes
and 33 retained original dossiers. Its live check passed before handoff-only
Markdown changed. Both Git-path/symlink helper defects passed real negative
fixtures and independent review. This handoff and PROJECT-STATE are new English
transfer prose; the old snapshot must not be falsely presented as their hash
approval. Its immutable `--ref 2073...` check remains the source-bound command.
The actual Wiki publication is `4a01b8b2ad2695da509a512ea688007c964f56da`;
15 source blobs and 13 prior pages were preserved. Discussion category display
names were checked in English; no stable slug was blindly changed.

At the inventory cutoff, older published #356 head `8c511e67` had **59 successful
checks, 9 intentional skips, 0 pending and 0 failed**, with auto-merge disabled.
Those results do not approve this new transfer head. **Current-head hosted CI,
final candidate review, actual-main integration and post-merge verification
remain pending/not verified.** No pending/missing/skipped mandatory gate is a pass.
The new handoff-only packaging was not independently approved for merging.

## Security, issues and private material

All three canonical Daybreak findings remain open/new: one High, one Medium,
one Low, re-queried for the authorised repository. IDs and private details stay
in Daybreak. No new scan or resolution was performed. CodeQL/Code Quality,
Dependabot and secret-alert inventories remain **NOT VERIFIED** after HTTP403;
successful scan jobs do not prove zero alerts.

#355 has verified limited pre-L0 revocation and failed-model-candidate fixes,
real Linux isolation/offline tests and host/smoke receipts, but lacks complete
kernel-enforced resource/scratch/process-tree limits and selected authenticated
model-client/CI-gate compatibility. Cloud has read-only cgroup-v2, no required
private user-bus runtime and older systemd. Bubblewrap >=0.13.1 remains required;
the verified local build alone is not full runtime acceptance. #357 is untested:
scratch polling is not kernel quota, several paths/output/lifecycle boundaries
remain outside scope, and three actionable recovery review threads remain open.
The main manifest stays disabled; the unconditional controller hold is in
unmerged #359/#356. **Do not invoke or enable the issue pipeline, or use an
unisolated fallback.** Do not declare repaired main or close the High finding.

Two private fixture workstreams received platform content-risk blocks and never
implemented their proposed kernel fixes. Their three source fixtures and private
diagnostics are **NOT TRANSFERRED**, deliberately excluded from public GitHub.
[Sanitised inventory](cloud-to-local-20261118/private-inventory.json) records
logical identifiers, hashes, verification status and fresh-local-test requirements.
An encrypted Cloud-only archive passed integrity/decryption checks; this is not
evidence of delivery. An external upload was rejected and was not retried or
rerouted. No confidential payload, credential or key is in this repository.
Do not rely on Cloud paths or private original evidence being available locally.
Recreate permitted synthetic verification from public contracts and authorised
private findings; executor transfer does not authorise bypassing platform blocks.

All five issues remain open against their entire definitions of done:

- #41: persistent policy/anchor transitions, replay resistance and production
  custody/trust/time/human requirements; existing TEST vectors are not production.
- #42: long-lived USB-to-protected-input completion/coexistence contract,
  IRQ/reset/general hotplug/malformed-device and authorised physical evidence.
- #43: actual fixed-rubric target50 remains unmet; score41 is unchanged.
- #274: English document routes and bounded audits are preserved, while genuine
  public native/SDK/prototype localisation gaps remain actionable; no waiver.
- #283: original historical licence grant and missing historical Git/expired
  Actions audit bytes remain unresolved; no source/evidence is fabricated.

## Ownership, plan and first Local action

Cloud implementation/review agents are stopped and all Codex write claims are
released for the new Local executor after remote source verification. No Cloud
task resumes after this receipt. Independent Claude UI/UX, broader S09/D1–D5,
human research and product ownership are not transferred or overwritten.
Original working clones are retained; 106 duplicated captures were compared
with the candidate, and generated caches/binaries are distinct from source.

Project #5 last verified synchronisation: run `44226887984`, current main,
20 rule tests passed;90 items scanned,0 updates needed,0 missing fields. No new
classification/token/billing changes or paid features were made during transfer.
The approved Phase0 plan, Security Floor, ADRs, ownership releases and latest-
stable policy remain authoritative. Use the existing checkpoint README and
owner-policy records; do not restart completed research or invent a new plan.
**Baseline B is not established; governance phases1–6 have not started.**

The first safe Local action is to fetch the existing eight source branches,
`codex/cloud-to-local-handoff-20261118`, and the exact final handoff SHA, read this file and repository instructions, and
compare actual GitHub heads/main with the receipt. Verify `d16fa086` source and
its archived native evidence before claiming a workstream. Re-query current-head
checks and unresolved reviews; do not merge solely from old8c results. Keep the
pipeline disabled and private material NOT TRANSFERRED until permitted fresh
local evidence exists. No production keys, signing, release or physical hardware
action is implied by this transfer.

---

## Retained prior Cloud handoff — verbatim historical record

The following bytes are preserved from the previous handoff (SHA-256
`43697dd7d4a07e48c3d68ca5831f41c9a187278d24c2630c8e3e0e669eceafd7`).
Its old task state and source heads are historical; current transfer status above
supersedes their operational instructions.

# Phase 0 cloud handoff — 18 November 2026

**Execution status: PAUSED.** This one-time amendment transfers recoverable WIP
only. It does not authorize a cloud task to resume Phase 0 until the project
owner explicitly resumes it.

**Ownership: RELEASED FOR RECOVERY; EXECUTION PAUSED.** The prepared handoff
commit `c64d81d6bde1a0e649350325db15d87cde11e1cd` and listed source commits
were verified on GitHub before this release. Local Codex writers have stopped.
After an explicit owner resumption, a new cloud task must claim one workstream
against the current remote SHA.
This handoff transfers Codex work only;
it does not transfer the independent Claude UI/UX or S09 workstreams.

## One-time WIP transfer amendment

The following remote state was reverified after the original handoff:

- `main`: `366b82b353485207a9b0d6b1ae63dafbbe265bae`.
- PR **#355**, branch `codex/phase0-pipeline-isolation`:
  `7e406c7c3ec71691939acca6db0c594bb8ce03a9`. The PR remains open and was not
  merged or otherwise changed by this transfer.
- Recovery-only draft PR **#357**, branch
  `codex/phase0-pr355-resource-wip-untested`:
  `8a32bc582790fa16234dbadf37a0a176de620900`, whose parent is the PR #355 head.
  This commit preserves exactly `tools/agent_gate.py`,
  `tools/agent_pipeline.py`, `tools/agent_sandbox.py` and
  `tools/tests/test_agent_sandbox.py` as found in the paused worktree.

The recovery commit is **UNTESTED / NOT ACCEPTED**. No tests, runtime probes or
acceptance gates were run for it. A bounded publication review found no token or
key material, private scan identifiers, local user paths or exploit procedure;
`git diff --check` passed. These checks do not establish correctness or security.
Do not merge or treat this branch as an accepted PR #355 continuation.

Remaining security blockers are unchanged:

1. Resource accounting, bounded output and complete process-tree termination
   must hold for the full scratch lifecycle; the recovery WIP does not prove or
   complete those guarantees.
2. Authenticated model-client interaction must remain outside the credential-free,
   offline execution and gate boundary, with an owner-controlled engine allowlist
   and model output treated as untrusted input.
3. Applicable targeted, native, hosted CI and independent security verification
   remain required on a coherent current head.
4. The Daybreak finding remains open pending source-bound verification and the
   supported resolution workflow.

No unpublished changes remain in the security or handoff transfer worktrees at
this amendment checkpoint. Private evidence and the unrelated local work listed
later in this document remain local-only. The inventory below records the
original handoff context and must be re-queried after an explicit resumption.

## Source and authority

- Verified integration main at this amendment: `366b82b353485207a9b0d6b1ae63dafbbe265bae`.
- Primary accepted security-source checkpoint: `7e406c7c3ec71691939acca6db0c594bb8ce03a9`,
  branch `codex/phase0-pipeline-isolation`, existing open PR **#355**.
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
| #355 security / #357 recovery WIP | `codex/phase0-pipeline-isolation` — `7e406c7c3ec71691939acca6db0c594bb8ce03a9`; stacked draft #357 `codex/phase0-pr355-resource-wip-untested` — `8a32bc582790fa16234dbadf37a0a176de620900` | Primary source remains blocked as above. Recovery WIP is untested and not accepted. |
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

Do not execute these steps while the goal remains paused. They apply only after
the project owner explicitly resumes the work.

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

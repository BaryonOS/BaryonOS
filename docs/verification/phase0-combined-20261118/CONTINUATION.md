# Combined integration checkpoint — 18 November 2026

This subordinate receipt supports the authoritative next-agent handoff in
`docs/PROJECT-STATE.md`. It does not establish project acceptance or baseline B.
The owner explicitly resumed the paused Cloud execution mission; the original
17 November handoff remains an immutable historical record.

## Published and accepted work

Actual `main` is `de9b99047138c4fd0015ffc3f874b273b3537484`.
PRs #345, #346 and #339 were manually squash-merged after their applicable
checks and independent review passed. Each resulting tree was compared with
the reviewed candidate. PRs #347–#349 and #351–#353 were closed as incorporated
only after the translation manifest proved actual-main inclusion. Source
branches remain preserved. Both required post-merge main checks passed.

The first attempt to merge #339 was rejected by automatic approval review
while native checks were pending. The same candidate was merged only after
all 57 successful checks, both native window jobs and resolved review threads
were verified. No protection or gate was bypassed.

## Current candidate and ownership

Existing PR #356 is the sole combined integration PR. Its original preview
`bba804996b93d93e82b6fb323cdeea9779f6fe99` remains in the ancestry, including
its unique historical receipt. The frozen runtime source checkpoint is
`2a08f4a33eff63b354821f6892e1be28f7602c9b`, source fingerprint v3
`25dd6d69621f1d9d4c8ed353633b3db62818562fa6ce50efa4445207505adb44`.
`inputs.json` identifies the incorporated source heads. Source PRs remain open
until actual-main inclusion and acceptance are proved.

Root owns #356, shared project state, progress and integration evidence.
The documentation worker owns #358 and its separate English/Wiki worktree.
The Rust worker owns an isolated read-only-source host validation worktree.
Independent review has no source-write ownership. Original Claude UI/S09 work,
other source branches and private evidence remain preserved.

The combined source contains the approved D6(a) ustar export subset, Rust
1.100.0 compatibility and current-main conflict resolutions, free-tier
CodeRabbit configuration, reviewed English/Wiki receipts, and unconditional
containment of the legacy issue controller. The issue pipeline remains
disabled. Unfinished #355/#357 and unpublished kernel-security fixtures are
excluded.

## Verification in progress

At source `705b5cb9`, 403 tools tests, strict preflight, the M1 build and smoke
test passed; the guest emitted raw exit 33. These are preliminary receipts,
not acceptance of the final combined source.

Source `44f74050` passed all 18 local executable host-quality steps, the
15-VM volume gate and both DocumentSession VMs. After the two reviewed
obsolete CPUID unsafe blocks were removed at
`2a08f4a33eff63b354821f6892e1be28f7602c9b`, all three gates passed again:
18 steps (303.390 s), 15 volume stages (347.294 s) and two DocumentSession
VMs (19.403 s), each raw wrapper exit 0. The runtime fingerprint is v3
`25dd6d69621f1d9d4c8ed353633b3db62818562fa6ce50efa4445207505adb44`.
Separate original-source receipts and raw logs remain preserved. Final
language inventory, exact-candidate independent review and current-head hosted
CI remain required; historical proofs are not relabeled as final-source evidence.

The prepared Wiki was independently reviewed and published by a normal push
at `4a01b8b2ad2695da509a512ea688007c964f56da`; the repository receipt preserves
the exact publication and original pages. #358 continues with English routes
for dated source records. Proper native UI localisation remains incomplete.

## Open conditions

All five issues (#41, #42, #43, #274, #283) remain open against their complete
acceptance criteria. The score remains 41/100 with the existing rubric.
Production update policy/key custody and rollback boundaries, general and
physical USB/hotplug acceptance, the actual 50/100 criteria, full English
coverage/localisation and historical provenance/audit coverage remain open.

Three Daybreak findings remain unresolved. The #355 high-severity boundary
still lacks complete kernel-enforced resource/scratch/process lifecycle and
selected authenticated-client/CI-gate compatibility. Cloud lacks the required
writable cgroup-v2 and private systemd-user runtime. Never use unisolated
execution as a fallback. Two independent kernel-security implementation tasks
were blocked by the platform content-risk filter; only unverified fixture
WIP exists in isolated worktrees. No fix or finding resolution is claimed,
and those blocked operations must not be rerouted through another executor.

CodeQL alert inventory, Dependabot alerts and secret scanning are **NOT
VERIFIED**: integration access returns HTTP 403. Successful CodeQL workflow
jobs do not establish zero alerts. No paid scan, runner or review feature was
enabled. Private finding contents and reproduction evidence remain outside
public Git.

The latest existing Project #5 synchronisation run, `44226887984` on main `de9b9904`, succeeded: 20 rule tests passed;
90 items were scanned, no updates were needed and no fields were missing. Synchronisation must be repeated after further
integrations and closures.

## Resume operations

1. Re-query live main, open PRs/issues and exact source heads. Preserve active
   writers and use the existing PRs rather than duplicates.
2. Collect ongoing source-bound host/native receipts; finish #358 and merge
   only its reviewed coherent checkpoint into #356. Refresh the versioned
   language inventory explicitly for changed active guidance.
3. Complete the applicable final-source host/native gates, independent review
   and all current-head hosted checks. Retain failed attempts and raw exits.
4. Manually squash-merge #356 only when every applicable requirement passes.
   Verify actual-main tree equality, source inclusion and post-merge checks.
   Close source PRs only after exact inclusion or reviewed equivalence proof.
5. Continue feasible independent issue/security work. Preserve explicit
   runtime, platform, hardware, access and provenance blockers. Do not close
   issues or findings by numerical target, and do not establish baseline B
   while integration/security blockers remain unresolved.

No successor task has been scheduled or launched by this checkpoint.

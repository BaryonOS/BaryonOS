# Design decisions

**Project owner:** Simon van Ieperen.
This document distinguishes the decisions and authority of the project owner
from those of the executing roles. Source quotations retain their original
wording and speaker.

## ADR-CI-MERGE-GROUP-SELECT-20261126 — merge-hold judges a merge group by the gates selected from its trusted main base

**Status:** accepted (binding acceptance contract). The project owner
approved the design on 26 November 2026 in the CI specialist chat at about
20:20 UTC. At about 21:10 UTC the owner approved leaving out workflows that are
not applicable, on the condition that only workflows whose non-applicability
is established are left out and any doubt stays fail-closed (the two-source
rule under Judging), with a renewed independent D2 review. Proposed by the CI specialist for issue #494 on 26 November 2026,
after the owner's P0 request that a merge-queue group whose
combined change is, for example, documentation only no longer waits for every
native suite. It amends ADR-CI-SELECTION-20261124 and
ADR-CI-MERGE-GROUP-GATES-20261126 in the sentences listed below; those
sentences now point here. It changes no required check name, no workflow
trigger, no ruleset and no `host-quality` or `qemu-smoke` behaviour, and it
does not enable the merge queue.

**Source:** branch `ci/merge-group-select-20261126` on base `main` `e6547bce`
("PR A"). Skipping the unselected jobs inside each workflow on `merge_group`
is a later change ("PR B"); until then those jobs still run but no longer
decide.

**Problem:** on `merge_group` the selector planned in full (fallback
`merge-group`) and merge-hold expected every applicable gate of the queue
commit, so a documentation-only group waited for, and could be ejected by,
every native suite. Narrowing from `merge_group.base_sha` would be unsafe. In
a stacked queue that base is the queue commit of the entry ahead, which has
not landed: its selector and catalogue are not protected, and the diff from
it omits the entries ahead. This was observed on queue ref
`gh-readonly-queue/main/pr-571-9508ebfa…`, whose base `9508ebfa` is the queue
commit of #572.

**Decision:**

- **Trusted base T.** T is `git merge-base <merge_group.head_sha>
  refs/remotes/origin/main`, with `refs/heads/main` fetched from origin in the
  same merge-hold job. `merge-hold.yml` loads `tools/merge_hold.py` and
  `tools/ci_impact.py` from T, never from `merge_group.base_sha`. Before it
  loads them it requires T to be a well-formed commit, an ancestor of the
  fetched main, and an ancestor of (or equal to) `merge_group.base_sha`, and
  that base to be an ancestor of the queue commit. `merge_hold.py` repeats
  these checks against the same fetched ref, without fetching again, and
  requires `BASE_SHA` to equal T and `TESTED_SHA` the queue commit.
- **Plan.** `policy.events.merge_group` may be `select` or `full`; `manual`
  stays invalid, and `push` and `schedule` remain forced full. Both catalogues
  set `select` (revision 21). Under `select` the `merge_group` plan narrows
  from the diff T..queue commit exactly as a `pull_request` plan does, with
  the same fail-closed rules: an `always_full` path anywhere in that diff (for
  example `.github/**`, `tools/ci_impact.py`, `tools/merge_hold.py`) or an
  unknown path makes it full. Because T is an ancestor of the queue commit,
  the diff covers every entry ahead in the stack as well as the group's own
  pull request. `full` remains the rollback and keeps the fallback code
  `merge-group`. The `select` jobs of the three narrowing workflows still
  refuse to narrow on any event but `pull_request`.
- **Binding.** The plan must name T as its base and merge base, the queue
  commit as head and tested commit, `main` as target, and the catalogue and
  selector of T (trusted). The wait budget is read from T's catalogue. The
  report line records T, `merge_group.base_sha` and the queue commit.
- **Judging.** The expected gates are the selected `core` and `rwa` gates of
  that plan. A selected gate that is skipped, missing, failed or still running
  blocks as before. Under a `select` plan a workflow is neither awaited nor
  judged on the queue commit only when its non-applicability is established
  by two sources: a `core` or `rwa` gate of it is excluded as
  `no-input-match` and none is selected (catalogue), and its own
  `pull_request` path filter, as T's catalogue records it, certainly keeps
  it from starting for the diff T..queue commit. The selector of T lists the
  latter workflows in the plan as `pull_request_filter_blocked`. Any doubt
  leaves a workflow judged: no `pull_request` trigger, no path filter, a
  negated pattern, both `paths` and `paths-ignore`, no or more than 300
  changed paths, or a plan without the list. The queue therefore never
  ignores a workflow that the same change would start, and be judged by, on
  `pull_request`. Its runs, including a failed or still running one, then do
  not affect the decision, and the report names these workflows. A workflow with a selected gate is judged whole, as before, with
  its other jobs and unregistered jobs, and so is a workflow without any
  `core` or `rwa` gate (advisory or trigger-only). A `full` plan is judged as
  before. `WAIT_CHECKS` (`host-quality`, `qemu-smoke`), the walk
  over the entries ahead, the member rules and `entry_hold_passed` are
  unchanged.
- **Fail-closed rules.** Where no trusted copy of the tools can be identified,
  or the inputs contradict the queue, merge-hold fails; where T is valid but
  the diff from it is indeterminate, the plan is full:

  | Condition | Result |
  |---|---|
  | main cannot be fetched, no merge base exists, or T is malformed | the job fails before loading any tool |
  | T is not on main or not an ancestor of `merge_group.base_sha`, or that base is not an ancestor of the queue commit | the job fails before loading any tool; `merge_hold.py` fails before any wait |
  | `BASE_SHA` is not T, `TESTED_SHA` is not the queue commit, or the plan's recorded merge base is not T (a `select` plan must record one) | merge-hold fails before any wait |
  | shallow or missing history, an ambiguous merge base, a diff error or more than 5,000 changed paths from a valid T | full plan with that fallback code; every applicable gate is expected |
  | an `always_full` or unknown path in T..queue commit | full plan (`always-full`, `unknown-path`) |
  | an unusable fallback (`fetch-failed`, an untrusted selector, an invalid trusted catalogue) | merge-hold fails, as before |

**Amended sentences:**

- ADR-CI-SELECTION-20261124: "`push`, `merge_group`, `schedule` and
  `workflow_dispatch` always run in full." becomes "`push`, `merge_group`,
  `schedule` and `workflow_dispatch` still run every job; on `merge_group`
  merge-hold requires only the gates that the plan from the trusted base
  selects (ADR-CI-MERGE-GROUP-SELECT-20261126)."
- ADR-CI-MERGE-GROUP-GATES-20261126, Problem: "also runs on `merge_group`, at
  full scope." becomes "also runs on `merge_group`, every job of it (which of
  those gates merge-hold requires is decided by
  ADR-CI-MERGE-GROUP-SELECT-20261126)."
- ADR-CI-MERGE-GROUP-GATES-20261126, Decision: "`BASE_SHA` must equal
  `merge_group.base_sha` … the base must be an ancestor of the queue commit.
  The base selector recomputes the plan with `--event=merge_group` (mode
  full)." becomes "`BASE_SHA` must equal the trusted base T and `TESTED_SHA`
  `merge_group.head_sha`, with the ancestry checks of
  ADR-CI-MERGE-GROUP-SELECT-20261126. The selector of T recomputes the plan
  with `--event=merge_group` from the diff T..queue commit (mode `select` or
  `full`)."

**Consequences:**

- With the catalogue at revision 21, the selector run locally on a scratch
  clone gives a documentation-only queue commit a `select` plan that leaves
  19 workflows out, every integrated native suite among them; the group no
  longer waits for them or depends on them. For a change to `LICENSE`,
  `.coveragerc`, `.c8rc.json` or a non-Markdown file under `.agents/`, the
  path filters of `native-hid-input`, `native-xhci` and `resource-pressure`
  would start them on `pull_request` although no catalogue input matches, so
  they stay judged and 16 are left out. That mismatch between catalogue
  inputs and historical path filters is resolved fail-closed here; narrowing
  such filters is separate work. It still waits for
  the workflows that have a selected core gate and are therefore judged
  whole: `ci.yml`, CodeQL and the three workflows with a `select` job
  (component-leaves, app-management-quality, net-tls-quality), all of whose
  jobs run on `merge_group` until PR B. A `kernel/**` change selects 23
  workflows. Until PR B the unselected suites also still start and use
  runners on the queue commit.
- A failure in a workflow left out this way no longer ejects the group, as
  it would not have run on `pull_request` either. The `push` run on `main`,
  which stays full, remains the post-merge backstop. The two-source rule is a
  transitional safety mechanism until PR B skips the unselected jobs on
  `merge_group` and the selection logic is consolidated.
- **Landing constraint.** merge-hold runs the copy of `tools/merge_hold.py`
  at T. While this change is itself queued, T still holds the former copy,
  which requires `BASE_SHA` to equal `merge_group.base_sha`. The change
  therefore passes only as an unstacked group, and entries stacked behind it
  fail closed until it has landed.
- The three `select` jobs still plan `merge_group` from
  `merge_group.base_sha`; they never narrow on it, so this is harmless here.
  PR B must plan them from T before any job is skipped on `merge_group`.

**Revise when:** the first queue trial shows a stacked group's base or run
payloads that differ from the above; PR B lands; or GitHub documents the
queue ref and base of stacked groups.

**Primary sources:** none retrieved for this decision; `git merge-base` and
`git merge-base --is-ancestor` behave as documented in git-merge-base(1). The
stacked base is a repository observation of 26 November 2026, not GitHub
documentation.

## ADR-MERGE-QUEUE-AUTONOMY-20261126 — mandated workstreams enqueue their own pull requests

**Status:** decided by the project owner on 26 November 2026 (CI specialist chat)
and recorded by the CI specialist. It amends ADR-MERGE-DELEGATION-20261123 and
ADR-SPECIALIST-TEAM-20261125 for the merge queue that is active on `main`
(ruleset 25967588). ADR-MERGE-HOLD-20261113, ADR-CI-MERGE-GROUP-GATES-20261126
and D1–D3 stay in force.

**Problem:** with the merge queue active, workstreams still asked for a separate
approval or lane signal before each queue entry. That approval step slows the
queue down and conflicts with the delegated merge mandate.

**Decision:**

- Workstreams with a merge mandate under ADR-MERGE-DELEGATION-20261123, as
  extended by ADR-SPECIALIST-TEAM-20261125, add their own merge-ready pull
  requests to the merge queue themselves. They ask no approval per pull request,
  push, queue entry or regular merge. The command is
  `gh pr merge <N> --squash --match-head-commit <sha>`, run only after no
  workflow run on that head is still open and every merge condition in
  AGENTS.md holds.
- The CI specialist enqueues its own regular CI pull requests once the
  applicable D2 review and the other merge conditions hold.
- The PR guard (integration coordinator) coordinates dependencies, priorities and
  exceptions. It is not an approval layer. Its dependency and priority calls are
  still followed.
- Explicit owner approval remains necessary only for protected policy changes,
  changes to binding acceptance contracts, reserved architecture decisions,
  security exceptions, costs that are not authorised, and ruleset changes outside
  an earlier mandate. Once the owner has approved such a change in substance, no
  second approval is needed to enqueue it.
- Unchanged: the required checks, the applicable acceptance gates and holds,
  dependency order, protected-file review, the Security Floor, D1–D3,
  `--match-head-commit`, and the prohibitions on `--admin`, bypasses and
  auto-merge outside a delegation.

**Consequence:** only real exceptions and substantive owner decisions reach the
project owner; regular integration within the mandate is autonomous.

## ADR-KEYRING-LOCK-20261126 — Key erasure before S3 by kernel teardown of the Key Service instance

**Status:** this ADR records decisions at two levels of authority.
- **Project owner, 26 November 2026, question O1:** "Yes, with a label". In
  TEST profiles, SUSPEND may pass on the kernel-verified fact "instance
  erased" alone, with a mandatory label; every non-TEST profile also
  requires "no unwrap path".
- **Workstreams B and H, technical decision:** the LOCK mechanism, its codes
  and the work split. They act within the project-owner mandate Q3 of
  25 November 2026 (amendment H05-T5c of ADR-H05-S3-SYSTEM-20261123), which
  requires key erasure before S3 through the B keyring architecture.

UNLOCK and questions O2, O5 and O8 remain open. Nothing is implemented yet.
Specification: [B01-KEYRING-LOCK](implementation/B01-KEYRING-LOCK.md).

**Context.** RAM stays powered during S3. The SUSPEND storage gate proves an
idle, closed storage path but observes no key material. The ring-3
Key Service holds the master key, the DRBG key and the RANDOM pool for its
whole lifetime. Kernel service-stack buffers keep copies of IPC and entropy
payloads.

**Decision.**
1. The kernel keeps a keyring lock state, LIVE or LOCKED. Only the kernel's
   own LOCK path sets LOCKED. An exit or fault of the Key Service outside
   LOCK stays `UNEXPECTED_EXIT` and never counts as LOCKED.
2. Syscall `0x92` LOCK(power handle, 0, 0, 0, 0, 0) is init-only under the
   non-delegable POWER object, with the validation order of `0x90` and
   `0x91`.
   - Preconditions are checked without side effects: an idle entropy broker
     with no pending or completed request, and no queued message from the
     instance in any endpoint it can send to.
   - A failed precondition returns -8 (retry). A LOCK while LOCKED returns
     -14.
   - LOCK ends the instance through kernel process teardown, which zeroes its
     frames. It also wipes or scrubs the kernel copies of Key Service
     payloads and entropy, and never passes through `observe_exit`.
3. While the state is not LOCKED, SUSPEND returns -25. The check runs after
   `sleep::ready()` and before `storage_closed()` and any I/O. In TEST
   profiles a pass carries the label "instance erased; unwrap secret is a
   public TEST constant". Non-TEST profiles also require that no unwrap path
   remains.
4. B lands LOCK first, without changing SUSPEND. H then lands the SUSPEND
   check, init's LOCK-then-SUSPEND sequence and the post-wake re-init target
   in one change. Each step takes its own merge lane.
5. UNLOCK is a fresh instance. It serves keys only after fresh startup
   entropy and a restore with a session-authenticated secret, and after the
   key-free read-only remount. Its mechanism is a separate decision.

**Consequences.**
- TEST profiles show the mechanism, not confidentiality.
- The system runs without key service between LOCK and UNLOCK.
- Key state that is not durable at LOCK is lost (O5).
- The volume after a wake is checked for integrity, not authenticity (O7).

**Falsified if:** SUSPEND passes while the state is not LOCKED; a frame of the
ended instance is not zero after LOCK; a Key Service exit or fault yields
LOCKED; or a TEST-profile S3 pass lacks the label.

## ADR-CI-MERGE-GROUP-GATES-20261126 — merge-hold judges the applicable gates on the merge-queue commit

**Status:** proposed by the CI specialist for issue #494 on 26 November 2026 (D1
option B step 2 of ADR-PROGRESS-ASSESSMENTS-20261125). The project owner must
review it. It amends three sentences of ADR-CI-GATE-ENFORCEMENT-20261124, which
are listed below. It also extends ADR-MERGE-HOLD-20261113, whose acceptance
hold, labels, release comments and queue walk stay in force. It does not enable
the merge queue. Enabling the queue still needs a passing trial with at least
two entries (D1 step 3) and the owner's explicit confirmation at that moment.

**Budget (project owner, 26 November 2026):** the `merge_group` wait is 80 minutes
(`MAX_WAIT_SECONDS`, `merge_group_wait_minutes`) and the merge-hold job timeout 90
minutes, because `native-window-session` reached p90 59 and max 72 minutes.

**Source:** branch `ci/d1s2-merge-group-gates-20261126` on base `main`
`2289a271`. Implementation commits: `524ea570` (round 1), `1ae6b57e` and
`9e10d7ec` (round 2), `f7743f06` (stacked on CI-SETTLE), `7d387a80` to
`18f4e375` (fix round 3), `5901b29d` to `a533f91e` (round 4), and `7dc6c53c`
to the round 6 head `763f25d9`. `7671b20d` merges `main` `60b1aac7` (#558)
and changes none of the merge-hold files.

**Problem:** since D1 step 1, every workflow with an automatic `core` or `rwa`
job also runs on `merge_group`, every job of it (which of those gates
merge-hold requires is decided by ADR-CI-MERGE-GROUP-SELECT-20261126). On `merge_group`, however,
merge-hold waited only for `host-quality` and `qemu-smoke`. It judged each
member's gates on that member's own `pull_request` head and required its queue
entry to land a tree equal to that head. The tree that actually lands was never
judged as a whole, and a queue with more than one entry ahead failed closed.

**Decision:**

- **Inputs, bound before any wait.** On `merge_group`, `BASE_SHA` must equal
  the trusted base T and `TESTED_SHA` `merge_group.head_sha`, with the
  ancestry checks of ADR-CI-MERGE-GROUP-SELECT-20261126. The selector of T
  recomputes the plan with `--event=merge_group` from the diff T..queue
  commit (mode `select` or `full`; amended by ADR-CI-MERGE-GROUP-SELECT-20261126).
  The plan must describe that event, that base, the queue commit and `main`.
  Any doubt fails at once.
- **Run binding.** A run counts only with:
  - event `merge_group`;
  - head SHA equal to the queue commit;
  - head branch equal to the queue ref without `refs/heads/`;
  - this repository as both repository and head repository.

  The run's `pull_requests` list is not used.
- **Budget clock.** The catalogue budget (45 minutes) is counted from before
  the plan. The plan, the catalogue read, the wait and the chain of entries
  ahead share it. A 3-minute reserve is kept back for the decision after the
  wait, so the gate wait gets the budget less the plan time less the reserve.
  The deadline still fails closed; a plan that uses up the budget gets the
  single poll of a 0 budget. Up to the wait's last poll the script, whose
  clock starts before the plan, takes at most 43 minutes; the workflow steps
  before the script are outside that clock. The last poll, the decision and
  those steps must fit the remaining 7 minutes of the 50-minute job timeout.
  This is an assumption, not a proof. Counted one page each, the decision
  makes 2 + 2N + 4M requests for N counted runs and M members with no member
  lifted, 2 + 2N + 8M - 8 (M >= 2) in an ordinary queue, where every member
  ahead but the first is lifted, and at most 4N + 8M - 1. A poll makes three
  plus one per entry walked, and the last poll
  may itself add a full gate report (1 + 2N to 1 + 4N) when a new failure
  trigger appears that the report does not count. For N = 24 and M = 5 that is
  about 86 requests in an ordinary queue and about 240 at worst. Each blocking socket read
  times out after 20 s, so a single request has no hard ceiling. The job
  timeout stays the fail-closed backstop.
- **Wait.** After `host-quality` and `qemu-smoke` have completed, every poll reads the
  queue first. Two consecutive well-formed reads that lack the queue commit
  (the entry was removed or rebuilt) fail the group, also while gate runs are
  open. Under an enforcing plan the poll then reads the run listing of the queue
  commit. It waits while a gate run is open or an expected gate has no run. A
  gate that has already failed ends the wait early:
  - The trigger is a newest gate run that completed without success, or a
    completed check run with a blocking conclusion. For an expected gate, any
    conclusion but success is a trigger.
  - A trigger causes one full gate report.
  - A Failed line in that report fails the group, and that report decides.
  - Each trigger is judged once, and only after its report was read.

  The walk over the entries ahead is unchanged.
- **Decision.** One full gate report on the queue commit decides with the
  `pull_request` rules, including the settle step and the job attempt binding.
  Any Pending or Failed line fails the group, and the queue removes it. A
  runtime-matrix row whose run is still running is Pending, never Failed, while
  its variants so far have succeeded.
- **Members.** Each member needs:
  - its hold decision;
  - for a released hold, a queue entry tree equal to the released head tree.
    This rule is unconditional;
  - a successful newest `pull_request` merge-hold on its own head (defense in
    depth).

  An unheld member may land a tree other than its head's only when the
  queue-commit report was enforced and clean. Under `shadow`, or with any
  Pending or Failed line, its entry tree must equal its head tree, as before.

  That report judges only this group's own entry tree, the queue commit. A
  member ahead lands the tree of its own entry E instead. When that member is
  lifted (its entry tree differs from its head tree), it also needs the newest
  merge-hold run on E to have concluded `success`. That run must have event
  `merge_group`, head SHA E, a queue branch of that pull request, and this
  repository as repository and head repository. That run judged the gates on
  E. It is read as strictly as the queue-commit runs: its jobs (all of its
  attempt and successful), a re-read by id (same id, attempt and status, still
  bound and successful) and E's `merge-hold` check run read again at the
  decision (success) must all agree. A missing, running or failed run, or any
  disagreement between these reads, fails the member at the decision. The
  ruleset's "Only merge non-failing pull requests" setting, which the
  merge-queue rule must enable, also keeps E from merging without it.
- **Shadow.** Under `shadow`, gates neither hold the wait, nor block the
  decision, nor end the wait early.

**Consequences:**

- The tree that lands is judged as a whole. An unheld member is no longer
  ejected merely because the entries ahead change its landing tree, provided
  the enforced report is clean.
- **Budget.** With the current runner capacity, one `native-window-session` run
  took more than 46 minutes of wall time (measured; see the #494 record). With
  the former 45-minute budget and 50-minute job timeout, a queue entry running
  that workflow could fail at the deadline. The project owner raised them to 80
  and 90 minutes (see Budget above); the queue status check timeout must be at
  least 100 minutes.
- **Unobserved payload fields.** The `head_branch` and `head_repository` fields
  of a live `merge_group` run have not been observed yet. If GitHub fills them
  differently, no run counts and the group fails closed at the deadline. The
  first queue trial must therefore capture one real run payload.
- **API cost.** Each gate poll adds one run listing of the queue commit. A full
  report costs about two requests per counted run. It is made only for a
  failure that has not been judged before. A lifted member ahead adds up to
  four reads at the decision: the run listing of its entry commit, that run's
  jobs, a re-read of it, and the entry's check runs.
- **Entry queue ref.** The queue read does not return a member's queue ref,
  and GitHub does not document the ref's SHA suffix. The run on a member's
  entry is therefore bound to a queue ref of that pull request, not to one
  exact ref; its head SHA ties it to the entry.
- A hold added after the final read is still not seen (ADR-MERGE-HOLD-20261113).
- **Stale failure text (resolved).** The failure text for an unheld member's
  unequal tree was reworded in `f7743f06`, stacked on CI-SETTLE before fix round 3. It now says that the
  queue-commit gate report is not an enforced clean pass and so does not cover
  that tree.

**Revise when:**

- the first queue trial shows run payloads that differ from the binding;
- measured gate wall time on the queue commit stays above the budget;
- GitHub documents how a rebuilt group's queue ref and runs behave;
- the merge queue is enabled.

**Primary sources:** the sources of ADR-CI-GATE-ENFORCEMENT-20261124, retrieved
24 November 2026. No new source was retrieved for this amendment. The
`merge_group` run fields named above are bound fail-closed and remain to be
confirmed on a live run.

## ADR-N03-PROVIDER-20261125 — Native TLS provider route, test peers, certificate time and randomness

**Status:** accepted by the project owner on 25 November 2026 in the N-track
session. Owner wording in chat: "N03 B1 met latere B2-migratie, H3, P3, T3,
R1, ml-kem en geverifieerde ed25519-dalek." The fuller owner text relayed the
same day by the PR-watcher session adds the conditions reflected below:
integration code conforming to RFC 10024 only after a separate security
review, no self-written cryptographic primitives, the host randomness
exception only for the test oracle, and ml-kem and ed25519-dalek preferred
subject to verified compatibility, stability and dependency policy. The
options are those of
[N03-PROVIDER-DECISION-PROPOSAL.md](implementation/N03-PROVIDER-DECISION-PROPOSAL.md).
This decision supersedes the 23 November "host oracle first" gating for the
items below.

**Decision:**

- **Provider route (P1 = B1, later B2).** The native provider is project glue
  that implements the rustls 0.23.46 `CryptoProvider` traits over released
  `no_std` primitives; the project writes no cryptographic primitives. Once
  rustls 0.24 is stable and suitable, the provider migrates to B2 (project primitive
  adapters, with the hybrid share concatenation in rustls).
- **Hybrid share handling.** B1 places the RFC 10024 concatenation and
  fixed-length split of the X25519MLKEM768 shares in project code. That code is
  permitted only in the RFC 10024 byte layout, mirrors the upstream rustls
  implementation, is covered by known-answer and negative tests and requires a
  separate security review before merge. It does not relax ADR-028 for any
  other construction.
- **ML-KEM primitive.** `ml-kem` is the preferred ML-KEM crate, subject to
  verified compatibility, stability and dependency policy. The FIPS 203 section 7.2 encapsulation-key check
  is unverified for `ml-kem`; route B is not accepted until a test shows it
  (or project code performs it).
- **Signatures.** `ed25519-dalek` is preferred and admissible only after
  verification of compatibility and stability under
  ADR-VERSION-POLICY-20261117 and `docs/DEPENDENCIES.md` (provenance, licence,
  maintenance, `no_std`, build script, pinned version).
- **Host peers (P2 = H3).** Host loopback evidence uses both rustls 0.23.46
  with aws-lc-rs (host-only test dependency) and the existing Node.js/OpenSSL
  peer. The host randomness exception applies only to these test oracles,
  never to the native provider.
- **Test peer pinning (P3).** A repository-public test CA, named as such and
  never in a product trust store; exact SPKI SHA-256 pins bound to the
  peer-pin generation; Ed25519 leaf certificates; fixed test DNS name and
  ALPN, with wrong name, ALPN, pin, CA, suite or group as mandatory negatives.
- **Certificate time (P4 = T3).** Development probes may use pin-only
  acceptance for pinned test peers (T2), labelled as such and without an
  "expired certificate" claim. N03 acceptance requires the trusted calendar
  authority (T1).
- **Randomness (P5 = R1).** Every random request in the TLS service is a
  separate `KeyServiceRandom` draw of at most 128 bytes. A process-global
  getrandom backend and batched per-handshake draws are not admissible in the
  native provider.

**Consequences:** N03-T2 (host loopback with H3) and the B1 provider work may
start. New dependencies still pass the dependency procedure before they are
added. No N03 points are awarded by this decision; N03 acceptance remains
native, with T1 time.

**Rollback:** a later decision may select another route; until native
acceptance nothing in the product depends on B1.

## ADR-NET-ISSUER-20261125 — Network grant issuer, helper subgrants, regrant and durable denial

**Status:** accepted by the project owner on 25 November 2026 in the N-track
session. Owner wording in chat: "N04 issuer-ADR goedkeuren, cold rollback
anchor afstemmen met B02." The fuller owner text relayed by the PR-watcher
session names kernel-bound identity, a separate policy authority, bounded
helper delegation and durable revocation, a cold-rollback trust anchor agreed
jointly with B02, and fail-closed behaviour until reliable freshness can be
guaranteed. The adopted text is
[N04-ISSUER-ADR-PROPOSAL.md](implementation/N04-ISSUER-ADR-PROPOSAL.md); its
recommendations become decisions as listed here.

**Decision:**

- **Decision 1 = 1A.** Kernel IPC metadata identifies callers in the trusted
  service loop; one dedicated policy process holds PolicyAuthority through a
  kernel-delegated private control endpoint and is the single durable owner of
  grant and denial state. The network service holds no storage capability.
- **Decision 2 = 2A.** Only PolicyAuthority authorises helper subgrants, with
  depth one and the attenuation rules of the proposal; a parent holder may only
  request.
- **Decision 3 = 3A.** Regrant within a running service issues a strictly
  higher persisted owner generation, under the proposal's rules.
- **Decision 4.** The acknowledgement states `LiveDenied`, `DurablePending`,
  `Durable` and the reported failure state, and the rule that no time-bounded
  grant survives restart, are adopted.
- **Decision 5 (cold freshness).** The cold rollback anchor is aligned with
  workstream B (B02 secure records and the ADR-B03-B04-TARGET anchor backend);
  no network-specific trust root is introduced. Agreed split (N and B, 25 November
  2026): B owns the anchor and its contract in
  `docs/implementation/COLD-FRESHNESS-ANCHOR.md` (in preparation); the N04
  policy process is a consumer that reads one store-wide generation through
  an attenuated capability, advances it only after a durable store commit and
  fails closed on a missing or corrupt anchor. Until reliable freshness can be
  guaranteed, behaviour stays fail-closed and cold reinstatement unavailable
  (5A): after a cold
  boot every network grant starts denied, and persisted grants are only
  suggestions for explicit re-confirmation.
- **Decision 5B (TPM anchor use, approved 25 November 2026).** The project
  owner explicitly approved option 5B in the N-track session: N may use the
  TPM 2.0 infrastructure owned by workstream B (ADR-B03-REL05-TPM-20261125,
  `docs/implementation/COLD-FRESHNESS-ANCHOR.md`) for durable network grants
  and rollback protection. This is an approved, bounded extension of the trust
  boundary. Binding conditions from the owner:
  1. One shared TPM infrastructure with logically separate anchor domains for
     network grants, storage and system updates.
  2. B remains owner of the TPM infrastructure, trust anchors and their
     lifecycle.
  3. N gets access only through a bounded, capability-checked interface: no
     direct TPM access and no authority over other security domains.
  4. The generation is bound to the exact authenticated grant and tombstone
     data; rollback, replay and reactivation of revoked network rights are
     prevented.
  5. Crash consistency, atomic commit semantics, recovery procedures and
     protection against counter exhaustion are required.
  6. TPM writes are limited responsibly, considering performance and hardware
     endurance.
  7. 5A fail-closed behaviour stays in force while the TPM anchor is missing,
     unreliable or not yet fully validated.
  8. No cloud, NAS or software fallback that weakens rollback protection.
  9. Positive and negative native tests are run jointly with B, including cold
     restarts, replay, interrupted writes, TPM failure and domain isolation.
  Agreed ordering with B: commit-then-advance. A grant, subgrant, revocation or
  durable denial is acknowledged only after the anchor advance succeeds. With
  store generation S and anchor generation A at startup: S == A is accepted
  only when the digest of the authenticated store state (grants and tombstones)
  equals the binding stored with the anchor generation. This rejects a
  same-generation fork after a crash between commit and advance only while an
  attacker cannot rewrite the slot binding; with an unprotected slot a hostile
  rewrite can select a sibling store (M1 below, unresolved without the
  protected slot);
  S == A + 1 rolls forward (the entry at S must authenticate and was never
  acknowledged); S < A and S > A + 1 are rejected fail-closed. The advance is a
  monotonic compare-and-swap from A to A + 1 that also stores the new binding. Reserve-first was rejected because
  every ordinary crash would leave the store looking rolled back.

**Open:** proposal questions 6 (whether N04 acceptance requires durable
revocation across restart), 7 (the component holding boot-lifetime denial
entries) and 8 (the per-boot incarnation source) are not decided by this
entry. Review of the anchor contract on #524 (25 November 2026) found that the
digest binding does not by itself reject a same-generation fork: with the
unauthenticated slot write (T3), a crash before the slot write, a replayed
store at `A` and a hostile slot rewrite can revive a stale authentic store at
`A + 1` over an acknowledged revocation (M1; red host test
`hostile_slot_write_cannot_revive_a_stale_store_at_the_same_generation`,
ignored until fixed). A missing store with binding `0^32` can likewise be
forged as genesis (L2); agreed with B: a missing store is always denied and
genesis happens only through authorised provisioning or recovery, and after a
TPM replacement N04 rejects its store and re-anchors from genesis (C665).

The project owner approved a generation-advancing reseal at every cold boot
for M1 on 25 November 2026, conditional on, among others, preventing stale or
leftover slot contents from reactivating revoked permissions. Host modelling
before implementation found that the reseal does not meet that condition
across several boots: crashes induced during a reseal leave sibling
authentic stores at one generation, and a hostile slot write then selects a
sibling without an acknowledged revocation (red tests
`reseal_multi_boot_crashed_reseal_revives_over_rolled_forward_change` and
`reseal_multi_boot_sibling_reseal_revives_over_acknowledged_change`). With an
unauthenticated slot, the counter is the only anchor state an attacker cannot
choose, so no binding rule separates two authentic stores at one generation.
The reseal is therefore not implemented.

**M1 decision (project owner, 25 November 2026), superseding the reseal
approval:** the protected-slot route through B. B owns the protected slot,
its authorisation policy, NV names and lifecycle, and is authorised to make
the bounded slot-policy changes this requires; no other trust-root change and
no direct TPM access for N. N reaches the anchor only through the
capability-controlled interface; network, storage and update domains stay
separate. Stale slot writes, replay and unauthorised generation changes must
not reactivate revoked network grants across multiple cold boots. The threat
model must state explicitly whether the protection holds against a
compromised kernel; no protection beyond demonstrated guarantees is claimed.
Crash consistency, TPM endurance, authenticated recovery and monotonicity are
preserved. Both M1 regression tests are retained and extended with multi-boot
replay, unauthorised slot writes, interrupted commits, TPM replacement and
recovery; native verification is required in addition to host tests. 5A stays
in force until the protected slot and the complete 5B integration are
demonstrably safe.

**Amendment (25 November 2026): the protected slot is an extend chain.** B
implements the M1 route as a per-domain `TPM_NT_EXTEND` hash chain (contract
`docs/implementation/COLD-FRESHNESS-ANCHOR.md` on #524, basis `5db011cc`;
P1–P3 agreed with N). The generation counter, the rewritable slot binding and
the `Pending` state above are replaced: a store records `prev`, the chain value
it was committed on top of; the kernel derives `E(S)` from the domain and
`D(S)`; startup accepts `chain == H(S.prev ‖ E(S))`, rolls forward once on
`chain == S.prev` (a never-acknowledged store) and otherwise denies the store
as `Stale`; `Foreign` covers an instance mismatch. `services/net-broker/src/durable.rs`
follows this contract in a host model whose fake computes the real SHA-256
chain and reproduces B's golden vectors. The M1 regression tests are kept
under B's names and now pass against a hostile extend:
`hostile_extend_cannot_revive_a_stale_store_at_the_same_position` (formerly
`hostile_slot_write_cannot_revive_a_stale_store_at_the_same_generation`),
`crashed_sibling_cannot_displace_a_rolled_forward_store` (the two
`reseal_multi_boot_*` cases and the multi-boot protected-slot case),
`every_crash_prefix_of_a_commit` and
`replaced_tpm_rejects_the_store_until_authorised_genesis`. A stray extend now
denies every store until authorised genesis (fail closed). Host evidence only;
native S5 verification on swtpm, torn-A1 store recovery (C143, #529) and the
residuals listed in B's contract remain open, and 5A stays in force. One
residual passes N04's own checks: if both copies of the accepted pair are
lost (TPM clear plus UEFI NVRAM reset, open U13c item), the old `ENROLL`
authorises a second genesis under the old instance, so the instance check
does not reject that instance's revived first store.

**Consequences:** issuer, subgrant and regrant code (N04-T3) is no longer held.
N04 stays open until its native evidence exists; the score is unchanged.

**Rollback:** reverting this entry restores the 23 November hold; no shipped
grant format depends on it yet.

## ADR-B03-REL05-TPM-20261125 — Trial boot with TPM 2.0 health confirmation (REL-05 option A)

**Status:** the requirements (option A, items 1–9 on issue #41) were decided
by the project owner on 25 November 2026. The mechanism below (PCR 13, NV
layout, kernel confirmation grant, amendment of ADR-049 decision 4) is
**adopted by the B workstream under the owner's execution mandate of the same
day, for the TEST profile only**; the specification
is [B03-REL05-TRIAL-BOOT-TPM.md](implementation/B03-REL05-TRIAL-BOOT-TPM.md).
Design only: nothing is implemented or tested. It would amend ADR-049
decisions 2 and 4 for a new loader feature `signed-boot-trial`; `signed-boot`
keeps ADR-049 semantics, including the REL-05 hazard, until gate migration.
On 26 November 2026 the project owner approved this ADR in substance for the
strictly bounded TEST/QEMU/swtpm profile, including C665 and M1-HAUTH (last
amendment below).

**Problem:** the signed loader raises `NieuwOsMinVersion` when a slot
verifies, before the kernel runs, so a signed but broken release makes the
previous good release a permanent `DOWNGRADE` (REL-05).

**Decision:**

1. The floor is raised only by the loader, before ExitBootServices, on the
   boot **after** a durable TPM-held health confirmation that binds the exact
   generation, version, slot and manifest digest of the trial (owner items
   5–6). It is never raised at selection time and never lowered.
2. A candidate (verified slot above the effective floor and above every
   rejected version) runs at most three times; each attempt is made durable in
   TPM NV before the candidate executes. After three unconfirmed attempts it is
   recorded as rejected and the known-good release boots; user data is not
   touched (items 3, 7).
3. TPM objects (TEST profile, disposable swtpm): loader-only `TRIAL` and
   `FLOOR` records and a `GEN` counter, writable only while PCR 13 is at its
   reset value; a `CONFIRM` record writable only while PCR 13 holds the fixed
   trial-open value. The loader always extends PCR 13 before handover. The
   effective floor is `max(UEFI copy, TPM FLOOR)`.
4. The kernel calls no UEFI runtime service (item 2). In profile
   `native-update-trial` it performs, as narrowly scoped privileged policy
   recorded by this ADR (`kernel/AGENTS.md`), one confirmation write and one PCR seal per trial boot through a
   one-shot capability held by a ring-3 health service. The kernel copies the
   loader-supplied binding; ring-3 supplies only a verdict for the versioned
   list `HEALTH-LIST-1`. The kernel still contains no cryptography.
5. While a trial is pending the update service does not write slots (amends
   ADR-049 decision 4).
6. *(Superseded in full: C665 takes precedence, and the REL-05 §7.9
   amendment of 26 November 2026 below removes `DEGRADED`.)* A missing, faulty, cleared, replaced or
   inconsistent TPM gives a degraded
   boot (known-good only, no trial, no floor write, explicit markers), except
   that a reachable TPM whose phase PCR cannot be extended halts the boot
   (`BOOT:FAIL:TPM_EXTEND`): handing over with PCR 13 at its reset value would
   grant the kernel loader-phase write rights, so halting is the fail-closed
   choice. This TEST-profile grant is not a product kernel TPM grant
   (`docs/contracts/TPM2-CRB.md`). TPM state
   behind the UEFI copy is treated as rollback. A TPM counter alone is not
   full rollback protection: clearing the TPM together with the firmware
   variables, or a boot-path attacker before B04, defeats it (item 9).

**Not authorized:** production keys, signing or activation; physical TPM or
firmware changes; a production TPM provisioning or owner-authorization
procedure; CI workflow changes without a separate owner-reviewed request. TPM 2.0 is the
intended D3 backend of ADR-B03-B04-TARGET for this TEST profile only; its
production approval remains open. Use of the anchor for N04 is authorized
separately by ADR-NET-ISSUER-20261125 Decision 5B (owner, 25 November 2026)
under its conditions 1–9; the contract is
[COLD-FRESHNESS-ANCHOR.md](implementation/COLD-FRESHNESS-ANCHOR.md).

**Amendment, 25 November 2026 (owner decision C665, PR #524):** the owner
authorises a strictly bounded extension of the trust architecture: TPM
identity binding (U13) through an authorised enrollment record signed under
the compiled-in update trust anchors (an anchor independent of TPM, UEFI
variables and disk), with an EK-salted session per boot. Until that binding
is demonstrably reliable, an unknown, replaced, reset or unverifiable TPM
blocks the normal boot route (`BOOT:FAIL:TPM_IDENTITY`; an unenrolled TPM halts
with `BOOT:FAIL:TPM_UNENROLLED`, since a deleted record cannot be told apart from
no enrollment; an NVRAM reset halts with `BOOT:FAIL:UEFI_FLOOR_MISSING` instead
of healing automatically) and only an authenticated recovery proceeds. The UEFI floor is
never raised from an unverified TPM and the anti-rollback floor is never
lowered silently. Legitimate TPM or motherboard replacement and NVRAM reset
go through authorised recovery without touching user data. The anchor domains
B02, B03/B04 and N04 stay separate; N04 re-anchors from genesis after
recovery (agreed with workstream N). Native tests C23–C30 are required before
acceptance. Specification section 10.5; TEST profile only.

**Amendment M1-HAUTH, 25 November 2026 (project-owner decision, option 1,
PR #524):** relayed by the PR guard session at about 11:58Z and confirmed by
the project owner in the B session. Quoted decision:

> I explicitly approve option 1: changing ownerAuth and lockoutAuth within
> the isolated TEST/QEMU/swtpm profile to address the M1
> undefine/redefine/replay vulnerability. This is a strictly bounded
> extension of the previously approved TPM trust architecture. The following
> requirements are binding: Preserve the per-domain TPM_NT_EXTEND hash-chain
> architecture and logical separation between B02, B03/B04 and N04. Protect
> TPM hierarchy authorization against unauthorized runtime access. Generate
> appropriate authorization secrets using approved entropy. Never hardcode,
> log or expose them to ordinary services. Preserve authorized loader-phase
> recovery, including undefine and re-genesis with a fresh, authenticated
> anchor instance. Ensure that recovery cannot revive previously revoked
> grants or silently lower anti-rollback protection. Verify that the
> PCR13-based loader-phase policy provides the intended protection. Do not
> treat PCR phase separation alone as proof of trusted boot, TPM identity or
> protection against a compromised kernel. Preserve the previously approved
> U13 TPM identity binding. Test unauthorized undefine/redefine/replay, TPM
> clear, TPM replacement, foreign boot environments, interrupted
> provisioning and authorized recovery. Retain the existing failing M1
> regression cases and demonstrate that the revised implementation prevents
> the targeted revival scenario. Coordinate P1–P3 with N and finalize the
> shared anchor contract. Keep N04 operating under 5A until the complete 5B
> integration passes native security acceptance. Explicitly document
> remaining threat-model limitations. Do not accept the demonstrated revival
> vulnerability as a residual risk merely to complete the implementation.
> This authorization applies exclusively to disposable TEST/QEMU/swtpm
> environments. It does not authorize modification of physical TPMs,
> production credentials or host security settings.

Mechanism: a loader-generated secret (`EFI_RNG_PROTOCOL`, U25) is stored in
the policy-read NV index `HAUTH` (`0x0100B304`, readable only under
`PolicyPCR(13 = 0³²) + CC_NV_Read`) and set as `ownerAuth ‖ lockoutAuth` by
`HierarchyChangeAuth`; `endorsementAuth` and platformAuth are not changed.
Evidence so far: swtpm 0.10.1 feasibility probe
(`docs/verification/b03-rel05-hauth-probe-20261125/`) and host model; the
native loader path and S5 are open. Limitation, reported to the project
owner and not accepted as a residual by B: code that runs at PCR 13 = 0³²
(boot path before B04, foreign OS) can read `HAUTH` and revive an older
store one reboot later; the revival is refused at run time within a boot,
not across a hostile boot path. 5A stays in force. Contract:
[COLD-FRESHNESS-ANCHOR.md](implementation/COLD-FRESHNESS-ANCHOR.md).

**Amendment, 26 November 2026 (project-owner decision on PR #524: approval
and C665 precedence):** the project owner approved this ADR in substance for
the strictly bounded TEST/QEMU/swtpm profile, including C665 and M1-HAUTH, and
decided (translated from Dutch) that C665 takes precedence over decision 6: an
unknown, replaced, reset or unverifiable TPM must not bypass the identity
binding through `DEGRADED`; the controlled halt and authenticated recovery
stay. REL-05 §7.9 was decided separately the same day (amendment below). The
boot-path revival at PCR 13 = 0³² before B04 stays an open security problem,
not an accepted residual risk. N04 stays under 5A until the full 5B
integration passes native security acceptance. All open implementation and
acceptance conditions (C23–C30, native S5, U13c) remain; no production
credentials, no physical TPM change and no progress score change.

Verbatim decision (project owner, Dutch, 26 November 2026 19:55 UTC, pasted
into the workstream B chat, session `ea7388`):

> EIGENAARBESLUIT — #524
>
> Ik keur ADR-B03-REL05-TPM-20261125 inhoudelijk goed voor het strikt afgebakende TEST/QEMU/swtpm-profiel, inclusief C665 en M1-HAUTH.
>
> Verduidelijk vóór de eerstvolgende push dat C665 voorrang heeft op het oorspronkelijke besluit 6: een onbekende, vervangen, geresette of onverifieerbare TPM mag niet via DEGRADED alsnog de identiteitsbinding omzeilen. Behoud gecontroleerde halt en geauthenticeerd herstel. Stem de toepasselijke testverwachtingen daarop af, zonder de nog openstaande §7.9-beslissing stilzwijgend in te vullen.
>
> De bekende boot-path-revival vóór B04 blijft een open beveiligingsprobleem en wordt niet geaccepteerd als definitief restrisico. N04 blijft onder 5A totdat volledige 5B-acceptatie is aangetoond.
>
> Behoud alle openstaande implementatie- en acceptatievoorwaarden. Geen productiecredentials, fysieke TPM-wijzigingen of verhoging van de voortgangsscore.
>
> Verwerk deze verduidelijking in de reeds voorbereide lokale wijzigingen. Stem één gebundelde push af met de PR-bewaker en integreer #524 via de merge queue zodra alle geldende voorwaarden zijn vervuld.
>
> Geen nieuwe algemene onderzoeks- of reviewronde. Ga daarna zelfstandig verder met de resterende implementatie.

Application by workstream B (the mapping of TPM classes below is B's reading
of the owner's precedence decision, recorded for the owner to confirm or
reverse; it is not itself an owner decision), TEST profile:

- A TPM that is absent, fails before or during the
  identity proof, or shows PCR 13 dirty at loader start has no verified
  identity: the loader prints the class marker, halts with
  `BOOT:FAIL:TPM_IDENTITY`, writes nothing and runs no kernel. A failed TPM
  write or read-back after a verified identity re-steps as a fault and halts
  the same way. An absent TPM therefore means no boot in this profile. An unenrolled
  TPM (`BOOT:FAIL:TPM_UNENROLLED`), another or unproven endorsement key,
  including a replaced TPM or one reset to new seeds (`BOOT:FAIL:TPM_IDENTITY`),
  and an NVRAM reset (`BOOT:FAIL:UEFI_FLOOR_MISSING`) halt as before. A failed
  extend still halts with `BOOT:FAIL:TPM_EXTEND`. Recovery is authenticated
  only (section 10.5).
- *(Superseded by the REL-05 §7.9 amendment below.)* Decision 6 kept
  `DEGRADED` only for a verified TPM whose anchor is lost or behind
  (`ANCHOR_LOST`, `ANCHOR_BEHIND`), with the cleared-TPM case left open.
  "Reset" is read as a TPM with new seeds (a new
  endorsement key); that `TPM2_Clear` keeps the endorsement key and a fresh
  swtpm state does not is TPM 2.0 Part 1 hierarchy semantics, not probed on
  swtpm *[unverified]*.

**Amendment, 26 November 2026 (project-owner decision REL-05 §7.9: TPM clear
and anchor loss):** the project owner chose fail closed (translated from
Dutch): a TPM that was cleared but keeps its original endorsement key gets a
controlled halt with authenticated recovery; a matching endorsement key
proves identity, not continuity of the protected state. Binding conditions:
no regular or `DEGRADED` boot once a TPM clear or the loss of required anchor
state is detected; no automatic re-provisioning based only on a matching
endorsement key; recovery only through an authenticated, demonstrably current
recovery procedure; keep the highest reliably established anti-rollback
floor; prevent reactivation of revoked keys, grants and older anchor
instances; when every reliable freshness record is lost, independently
authorised recovery is mandatory; preserve user data; give a clear recovery
message. The permission holds only within the existing TEST/QEMU/swtpm
mandate. REL-05 stays open until every acceptance condition is met; the
progress score stays 43/100.

Verbatim decision (project owner, Dutch, 26 November 2026 20:07 UTC, pasted
into the workstream B chat, session `ea7388`):

> EIGENAARBESLUIT — REL-05 §7.9
> Kies fail-closed: gecontroleerde halt met geauthenticeerd herstel wanneer een TPM is gewist maar zijn oorspronkelijke EK behoudt.
> Een overeenkomende EK bewijst identiteit, niet continuïteit van de beveiligde toestand.
> Bindende voorwaarden:
>
> * Geen reguliere of DEGRADED-boot wanneer een TPM-clear of verlies van noodzakelijke anchorstatus is vastgesteld.
> * Geen automatische herprovisioning op basis van uitsluitend een overeenkomende EK.
> * Herstel uitsluitend via een geauthenticeerde, aantoonbaar actuele recoveryprocedure.
> * Behoud de hoogste betrouwbaar vastgestelde anti-rollback-floor.
> * Voorkom heractivering van ingetrokken sleutels, grants en oudere anchor-instances.
> * Wanneer alle betrouwbare freshness-records verloren zijn, is onafhankelijk geautoriseerd herstel verplicht.
> * Behoud gebruikersgegevens.
> * Geef een duidelijke herstelmelding.
>
> Deze toestemming geldt uitsluitend binnen het bestaande TEST/QEMU/swtpm-mandaat.
> Leg dit besluit vast in §7.9 en de toepasselijke ADR. Implementeer en verifieer gericht, zonder nieuwe algemene onderzoeksronde.
> Laat REL-05 open totdat alle acceptatievoorwaarden zijn vervuld. De voortgangsscore blijft 43/100.

Consequence for this ADR: decision 6 is replaced in full. `DEGRADED` no
longer exists. Every `ANCHOR_LOST` and `ANCHOR_BEHIND` class with a verified
identity halts with `BOOT:FAIL:TPM_ANCHOR`; every halt that only
authenticated recovery resolves (identity mismatch, `TPM_UNENROLLED`,
`UEFI_FLOOR_MISSING`, `TPM_ANCHOR`) also prints `BOOT:RECOVERY:REQUIRED`, and
an absent, faulty or dirty-PCR TPM halts without it. The
host model and its tests implement this; the loader does not yet (the trial
path, S6, is not built). Section 7.9 of the specification records each
condition with its status. Since 27 November 2026 the TEST loader's
verify-only M1-HAUTH build no longer provisions when `ownerAuth` is unset,
which is also the state after a clear: it halts with
`LOADER-HAUTH:FAIL:TPM_UNENROLLED` and `LOADER-HAUTH:RECOVERY:REQUIRED`
without an NV or hierarchy-auth write, and only a separate TEST enrollment
build provisions. Open binding conditions include: that stand-in for the
signed `ENROLL` step (S3b) is chosen by the build and the harness, not by
the device, and is not authenticated, so the second condition is not met;
the currency of a recovery record; the recovery authority for the case where
both copies of the accepted pair are lost; and the user-visible recovery text.

**Consequences:** new TPM codec commands, loader TCG2 path, BootInfo
extension, kernel profile and health service; the characterization of PR #514
must fail under the trial profile. Issue #41, B03 and REL-05 remain open;
scores are unchanged.

## ADR-SPECIALIST-TEAM-20261125 — Permanent specialists A, P, L and R; handoffs, naming and context rotation

**Status:** decided by the project owner on 25 November 2026. The owner gave the
decision to the PR guard session at about 11:25Z. The owner then confirmed the
wording for recording in the CI specialist session at about 11:45Z ("Yes,
confirmed"). The CI checkpoint is #494, comment 7161215540. Item 8 was decided
separately at about 12:35Z and given to the PR guard. At about 13:30Z, in the
CI session while #527 was under review, the owner clarified that the context
thresholds in item 5 apply only to the executing specialists and that the PR
guard rotates only at the owner's explicit request. These chat-only
sources are recorded in the CI specialist's record on #494, comment
7162652798. The owner's formal review of
this pull request, on the current text, is the confirmation that D2 requires.

**Problem:** six workstream sessions handled their own pull requests under
`ADR-MERGE-DELEGATION-20261123`: S09, U, N, UI/UX design, B and H. Applications,
personal NAS, local AI and quantum runtime, and system-wide reliability had no
permanent owner. Handoff locations, session names and the rule for when a
session's context must rotate were only in issue comments.

**Decision:**

1. **New permanent specialists.** The project owner starts each session; agents
   never start, fork or replace specialist chats.
   - **A:** A01–A04, applications, SDK and ecosystem.
   - **P:** P01–P04, personal NAS and multiple devices.
   - **L:** L01–L02, local AI and quantum runtime. L also holds the research
     line for future physical QPU and hybrid hardware.
   - **R:** R01–R03, reliability, performance and release. R is responsible
     for performance and reliability across the whole system.
2. **Merge delegation.** `ADR-MERGE-DELEGATION-20261123`, with its amendments
   (including D2 of `ADR-PROGRESS-ASSESSMENTS-20261125`), also applies to A, P,
   L and R. Each owns its pull requests up to a protected squash merge with the
   expected head and verification on `main`. No additional central gate is
   added. Protected files still follow the review rules of that ADR and D2.
3. **Session names.** The sessions are named "BaryonOS - Specialist <X>", where
   X is R, L, A, S, N, CI, P, B, H or U. "U" stands for UI/UX and is displayed
   as "Specialist U (UI/UX)". The coordination session is "BaryonOS - PR
   Bewaker" (PR guard).
4. **Handoffs.** Each specialist keeps one compact resume pointer in
   `docs/handoffs/<letter>/CURRENT.md`. CI uses `docs/handoffs/CI/`, UI/UX
   uses the existing `docs/handoffs/UI-U/` and the PR guard uses
   `docs/handoffs/PR-WATCHER/`. A resume pointer records the scope
   and mandate once. It holds no live head SHAs, locks, PIDs or cursors. Live checkpoints
   stay in the pull request description and the workstream's handoff issue
   comment.
5. **Context rotation.** These thresholds apply only to the executing
   specialists (A, B, CI, H, L, N, P, R, S and U):
   - 65% context use is a moment to assess, not a stop;
   - from about 60%, the session updates its durable checkpoint and finishes
     the current phase;
   - above about 75%, it starts no new large phase;
   - by 80–85% at the latest, a verified handoff is ready.

   The PR guard does not follow these thresholds. It rotates its own context
   only at the project owner's explicit request. For the executing
   specialists, ownership of a subtask continues across a rotation. For every
   session, including the PR guard, no replacement chat is started.
6. **Product priority.** The product is a general-purpose OS with European data
   sovereignty. Conventional PCs and laptops come first, then server and
   business use. It aims at broad consumer and professional compatibility.
   Market potential is a research question, not an assumption. The design
   ambition and product specification remain in `docs/DESIGN-AMBITION.md` and
   `docs/implementation/PRODUCT-SPEC.md`.
7. **Quantum.**
   - From this decision on, post-quantum cryptography is a current obligation.
     It builds on ADR-007, ADR-012, ADR-017 and ADR-028; those decisions alone
     scheduled it for later integration.
   - B owns the PQC content: `docs/implementation/VERIFIABLE-SECURITY-AND-PQC.md`
     and the PQC policy in `docs/implementation/QUANTUM-SPEC.md`. That content
     must address harvest-now-decrypt-later exposure and the classical
     dependencies that remain.
   - L keeps the local quantum-execution part of `QUANTUM-SPEC.md` (L02).
   - Physical quantum hardware is a future research line for L and does not
     block the consumer release.
   - No claim of full quantum resistance is made without bounded evidence.

8. **Tracking issues (project owner, 25 November 2026, about 12:35Z, given to
   the PR guard).** Owner wording: "Alle specialisten mogen voortaan
   zelfstandig noodzakelijke, inhoudelijk gerechtvaardigde trackingissues en
   subtaken binnen hun eigen werkstroom aanmaken, zonder afzonderlijke
   toestemming te vragen. Controleer vooraf op duplicaten. Leg deze
   bevoegdheid duurzaam vast en communiceer haar naar alle specialisten. Het
   aanmaken van issues is ondersteunend aan de ontwikkeling, geen vervanging
   voor implementatie." (All specialists may from now on create necessary,
   substantively justified tracking issues and subtasks within their own
   workstream without asking separate permission. Check for duplicates first.
   Record this authority durably and communicate it to all specialists.
   Creating issues supports development and does not replace implementation.)
   - The authority belongs to the executing specialists listed in item 5. The
     PR guard's role is unchanged.
   - A specialist checks for duplicates before creating an issue.
   - An issue stays within the specialist's own workstream.

**Alternatives:** none recorded. The owner recorded no alternatives that were
considered, and the record does not establish why four new specialists were
chosen rather than another split.

**Rationale/sources:**

- **Rationale:** the gaps described under Problem. No further
  rationale is recorded.
- **Sources:**
  - the owner's decision given to the PR guard at about 11:25Z;
  - the owner's confirmation in the CI session at about 11:45Z, recorded in
    the CI checkpoint (#494, comment 7161215540);
  - the tracking-issue decision of about 12:35Z, given to the PR guard;
  - the clarification of item 5 during the review of #527;
  - the CI specialist's record of these chat-only sources on #494, comment
    7162652798;
  - the PR guard's operational history in #503.

**Consequences:**

- `AGENTS.md` lists the ten specialists and the naming rule in the merge
  delegation paragraph.
- The PR guard routes signals, metadata checks and CLEAR_READY verification
  (CLEAR_READY is the signal that a session's context may be cleared safely) to
  A, P, L and R in the same way as to the existing workstreams.
- Issues #494 (CI) and #503 (PR guard) stay the places for decisions,
  operational notices and history.
- Item 8 replaces the earlier practice of asking the owner before creating an
  issue. The PR guard communicates it to all specialists.

**Security invariant:**

- Apart from two expansions, no authority is widened beyond
  `ADR-MERGE-DELEGATION-20261123` as amended by D2. Item 2 extends that merge
  delegation to A, P, L and R. Item 8 lets the executing specialists create
  necessary, substantively justified tracking issues and subtasks in their own
  workstream, after a duplicate check, without separate permission.
- Protected-file review, `ADR-MERGE-HOLD-20261113`, required checks and branch
  protection are unchanged.
- The PR guard has no commit, push, merge, label or review authority.
- Issue creation is limited to a specialist's own workstream.

**Test evidence:** `progress.py --check`, `check_english_docs.py` and
`tools.tests.test_agent_governance` pass. They check the documents only; they
do not prove that sessions follow the rules.

**Review trigger:** added by the CI specialist when recording this decision;
it is not part of the owner's wording. Reassess this decision when any of the
following occurs:

- a delegated specialist merges a change that `ADR-MERGE-DELEGATION-20261123`
  as amended by D2 would block;
- a CLEAR_READY handoff fails to resume its subtask;
- the PR guard needs write authority to do its job;
- issue creation produces duplicates or backlog without implementation.

**Rollback:** a new project-owner decision.

## ADR-PROGRESS-ASSESSMENTS-20261125 — Per-task assessment files; CI owner decisions D1 and D2

**Status:** decided by the project owner on 25 November 2026 (#494, comments
7155783661 and 7155944783). Owner wording: "D3 goedkeuren met behoud van alle
assessmentvalidatie" (approve D3 while keeping all assessment validation);
"D1 optie B na geverifieerde merge_group-integratie" (D1 option B after a
verified merge_group integration); D2: "Yes, option C".

**Problem:** every pull request rewrote the single `assessment` object in
`docs/progress.json`, so any two open pull requests conflicted on the same
lines. Since 23 November, 24 of 54 commits on `main` touched the file; #500
needed four extra CI rounds for it.

**Decision (D3):** a task or pull request that leaves the score unchanged adds
its own file `docs/progress/assessments/<YYYY-MM-DD>-<slug>.json` with the same
fields as the `assessment` object (`date`, `source_commit`,
`assessment_basis`, `branch`, `task`, `previous_score`, `summary`). A score
change still updates the `assessment` object, the criteria and the generated
views in `docs/progress.json`, `README.md`, `docs/PROGRESS.md` and the SVG.
The fixed criteria register and the rubric stay central and unchanged.

`tools/progress.py --check` applies every existing assessment rule to the
header and to each file, unchanged, and checks `assessment_basis` wherever it
is present. It adds checks for each file:

- an exact field set, with duplicate keys rejected;
- a plain, non-symlinked file of at most 16 KiB;
- a file name whose date matches the recorded date;
- an integer score.

Files never change the score or the generated views. Like the header's
`previous_score`, a file's score is the recorded value and is not recomputed.
Tying it to the current score would turn `main` red whenever a score change
merges after a task file with a newer date.

**D2 (amends ADR-MERGE-DELEGATION-20261123):**

- For CI files (`.github/**`, `tools/merge_hold.py`, `tools/ci_impact.py`), an independent AI review satisfies "reviewed by someone other than the merging workstream". The review is done by a separate agent without write access, and bot findings must be resolved.
- Policy, register and security files (any `AGENTS.md` or `CLAUDE.md`, `tools/progress.py`, `docs/progress.json`, `docs/DECISIONS.md`, keys) still need the project owner's review.

**D1 (extends ADR-MERGE-HOLD-20261113):** the merge queue is enabled only after a verified `merge_group` integration:

1. the selected workflows gain `merge_group` triggers at full scope;
2. merge-hold judges the queue commit;
3. a queue trial with at least two entries passes.

Enabling the queue in the ruleset is a separate step that needs the project owner's explicit confirmation at that moment. It must not weaken any existing security or acceptance gate. The strict up-to-date rule and merge-hold stay until the queue has proved itself.

**Rollback:** a new project-owner decision restores the previous format in one PR:

1. revert the AGENTS.md progress rule so that an unchanged score again updates the `assessment` header in `docs/progress.json`, and restore the header-only description in `docs/PROGRESS.md`;
2. convert every open pull request that adds a task file into a header update, or close it, before the loader goes, so that no record merges unvalidated;
3. replace the task-file loader in `tools/progress.py` with a check that fails while `docs/progress/assessments/` exists, so that a task file merged after step 2 turns the check red instead of staying unvalidated (the header validation is unchanged and stays);
4. remove `docs/progress/assessments/` after the lead copies any record that must be kept into the header or `docs/history/`.

D1 and D2 are rolled back by a new project-owner decision.

## ADR-UI-NOTIFICATIONS-20261125 — Alt+N panel as the single notification centre

**Status:** accepted; project-owner decision of 25 November 2026 for UI-U
cycle C5, forwarded verbatim by the PR watcher session:
"Alt+N-paneel als primair notificatiecentrum, met reguliere toegankelijke
navigatie via toetsenbord, muis en systeemklok. Eén samenhangend
notificatiesysteem, geen dubbele interfaces."

**Decision.** The Veld Alt+N panel is the one notification surface of the
window session. It opens by Alt+N, by a click on the header status band and by
a click on the header clock (DESKTOP-SHELL.md: notification centre from
clock/date). Inside it, Tab and Shift+Tab move focus and wrap, Enter and Space
activate the focused control exactly once per physical press, Delete removes
the notice and Escape closes without changing window focus.

**Consequence.** The pointer-only shell page `Page::Notifications` and its
header bands are removed. The user icon keeps the Session page. Evidence is
host tests in `sdk/examples/window-session/tests/keyboard_only.rs`; no native
run with real input, screen reader or human trial is claimed.

## ADR-CI-SELECTION-20261124 — Targeted pull request selection for component gates

**Status:** accepted under the project owner's CI mandate for issue #494 (phase
4, 24 November 2026). It builds on ADR-CI-GATE-ENFORCEMENT-20261124.

**Problem:** `component-leaves.yml` ran all 15 leaf variants on every non-metadata
pull request. App-management and net-tls ran on every such pull request too,
even when none of their inputs changed.

**Decision:** on `pull_request`, under `policy.mode = "select"`, these three
workflows run only what the trusted base plan selects:

- component leaves per variant;
- app-management and net-tls per job.

Each workflow gets a `select` job that renders the plan into job conditions
and, for the leaves, into the matrix. The catalogue becomes the single source
of the leaves matrix.

The integrated native suites, CodeQL and `ci.yml` stay unselected. `push`,
`merge_group`, `schedule` and `workflow_dispatch` still run every job; on
`merge_group` merge-hold requires only the gates that the plan from the trusted
base selects (ADR-CI-MERGE-GROUP-SELECT-20261126). merge-hold
remains the decision point: it does not trust the `select` jobs and blocks when
an expected gate did not pass.

**Consequences:** a pull request that touches no leaf, app-management or net-tls
input skips those jobs. A pull request that touches one leaf runs that leaf
only.

**Rollback:** set `policy.mode = "full"` on `main`. All jobs and variants then
run again, and enforcement continues. Measured results are recorded on #494.

## ADR-CI-GATE-ENFORCEMENT-20261124 — merge-hold enforces the applicable CI gates

**Status:** accepted under the project owner's CI mandate for issue #494 (phases 3
and 4, 24 November 2026). It extends ADR-MERGE-HOLD-20261113 and does not replace
it: the acceptance hold, its labels, release comments and merge-queue walk stay
in force.

**Problem:** the required checks were `host-quality`, `qemu-smoke` and
`merge-hold`, but merge-hold did not verify that the other applicable gates had
run and passed. A change could therefore merge while a native or component gate
was still running, had failed, was cancelled or had been skipped, and a pull
request could change the scripts that judge it.

**Decision:**

- The required `merge-hold` check decides; no new optional check is added. It
  runs the base commit's `tools/merge_hold.py` and `tools/ci_impact.py`,
  extracted from the merge commit's first parent, never the pull request's
  copies.
- On `pull_request` it recomputes the impact plan for the exact base tip, head,
  merge commit and target branch. It requires a successful result for every
  selected `core` or `rwa` gate that GitHub starts for the change. Results are
  bound to Actions runs by workflow path, event, head SHA, head branch,
  repository and head repository. The newest run and the latest attempt decide,
  and every listing is read through all its pages.
- Missing, running, failed, cancelled, timed-out and skipped expected gates
  block, and so does an unexpected failed gate. "Not applicable" is reported
  separately and is never a pass.
- The pull request wait budget is 0 (decision Q1 of the handoff): merge-hold
  lists the pending gates and is re-run after they finish. Job timeouts are
  unchanged. On `merge_group` every applicable gate is judged on the queue
  commit (ADR-CI-MERGE-GROUP-GATES-20261126); each member still needs a
  successful pull_request merge-hold on its own head, a released hold an equal
  tree, and an unheld member an equal tree unless the enforced queue-commit
  report is clean; a member ahead whose entry tree differs from its head tree
  also needs a successful merge_group merge-hold on its own entry commit.
- `host-quality` fails on an invalid catalogue or on drift between the
  catalogue, the workflows and the sources.
- `policy.mode` gains its enforcement meaning: `shadow` reports without
  blocking; `enforce` blocks; `select` blocks and lets workflows skip
  unselected executions (phase 4); `full` is the selection rollback and keeps
  enforcement.
- `tools/ci_impact.py` and `tools/merge_hold.py` join the Protected files list.

**Consequences:** merges wait for every applicable gate, not only for the three
required checks. A pull request's first merge-hold run normally fails with a
pending list and needs one re-run. The selected workflows have `merge_group`
triggers (D1 step 1) and merge-hold judges their gates on the queue commit
(D1 step 2, ADR-CI-MERGE-GROUP-GATES-20261126); the queue stays disabled until
a trial with at least two entries passes (D1 step 3) and the project owner
confirms enabling it. A started gate that the plan did not select must still finish
before merge-hold passes. The introducing pull request is judged by the older
base copy, so its gates must be verified by hand before its merge. Runs of a fork head
list no pull request and cannot be bound to a base, so a fork pull request fails
closed until its commits are pushed to a repository branch; relaxing that is a
project-owner decision. Under `pull_request`, GitHub still reads workflow files from the
pull request side, so review of `.github/**` remains the control for workflow
edits. Details and the remaining limits are in docs/CI-CONTRACT.md
("Enforcement", "Known limits").

**Revise when:** the merge queue is enabled (the `merge_group` triggers and the
queue-commit judgement are in place, ADR-CI-MERGE-GROUP-GATES-20261126); GitHub
changes which workflow files a `pull_request` run reads; or
`policy.mode = "shadow"` is needed to let a selected gate pass.

**Primary sources:**

- GitHub. (n.d.). [*REST API endpoints for workflow runs*](https://docs.github.com/en/rest/actions/workflow-runs).
  Retrieved 24 November 2026.
- GitHub. (n.d.). [*REST API endpoints for workflow jobs*](https://docs.github.com/en/rest/actions/workflow-jobs).
  Retrieved 24 November 2026.
- GitHub. (n.d.). [*Events that trigger workflows*](https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows).
  Retrieved 24 November 2026.

## ADR-PROJECT-LIFECYCLE-20261124 — Preserve specialist status and truthful closure

**Status:** accepted on the project owner's instruction of 24 November 2026.
This revises the Status and Priority rules of ADR-PROJECT-SYNC-20261111.

**Problem:** recurring classification could overwrite a specialist's active
workflow status from a PR draft flag or label change. Unmerged PR closures and
non-completed issue closures could remain in an active column, while a missing
issue closure reason was incorrectly treated as completed. The old Priority
mapping also described project options no longer used by Project #5.

**Decision:** retain every existing nonterminal Status, including Backlog,
Ready, In Progress, In Review and Blocked. Draft flags and explicit blocked
labels initialise empty statuses or reset Done/Closed when an item reopens;
they do not continuously manage a specialist's chosen status. Reopened issues
default to Backlog, with Todo/To do retained as compatibility alternatives.

A merged PR or an issue explicitly closed as COMPLETED becomes Done regardless
of its prior project status. An unmerged closed PR or any other closed issue
becomes Closed, meaning closed without confirmed completion. An absent or
unknown closure reason never implies completion. The generic native
Item closed → Done rule must remain disabled; merge-specific automation may
still record Done.

**Primary sources:**

- GitHub. (n.d.). [*GraphQL API reference: Issues (IssueStateReason)*](https://docs.github.com/en/graphql/reference/issues).
  Retrieved 24 November 2026.
- GitHub. (n.d.). [*GraphQL API reference: Pull requests (PullRequestState)*](https://docs.github.com/en/graphql/reference/pulls).
  Retrieved 24 November 2026.
- GitHub. (n.d.). [*Using the built-in automations*](https://docs.github.com/en/issues/planning-and-tracking-with-projects/automating-your-project/using-the-built-in-automations).
  Retrieved 24 November 2026.

Work Type and Subsystem remain fill-only and require unambiguous recognised
signals. Exact scopes hw/hardware/drivers map to Hardware, and
net/network/networking to Networking. Priority and planning dates are
organisation issue fields; Effort is a numeric project field. Specialists
maintain these values; this single-select project writer does not infer or
write them. If no supported target or compatibility option exists, the update
is skipped. The writer never creates schema options or substitutes Done for
Closed. Existing scan/addition bounds, credentials and daily/manual workflow
entry points remain unchanged.

**Consequences:** project lifecycle is organisational metadata. Done does not
establish an OS criterion, required-check result or release acceptance.
Specialists own ongoing state and estimates; the existing sync reconciles
observable terminal facts and fills supported empty metadata.

**Review when:** project field types/options, lifecycle ownership or GitHub
closure semantics change.

## ADR-HWINV-20261124 — Native read-only HW-INV inventory profile

**Status:** implemented by the H specialist (H06-T3); QEMU evidence only.

**Problem:** protocol `HW-INV` v2 needed a kernel profile that reports the
hardware facts required by `tools/hw_evidence.py` without changing device state
and without ending through QEMU's `isa-debug-exit` port, which is undefined on
physical PCs. UEFI defines `SecureBoot` as a boot-service and runtime variable
(UEFI Forum, *UEFI Specification* 2.10, section 3.3 "Globally Defined
Variables", https://uefi.org/specs/UEFI/2.10/03_Boot_Manager.html), but the
kernel calls no UEFI runtime services, so this profile captures it in the
loader and passes the observation to the kernel.

**Decision:**

- A paired build: kernel feature `native-hw-inventory` (IF0 M2 profile, one
  CPU, mutually exclusive with every other native subsystem profile) and loader
  feature `hw-inventory`. `tools/build.py` refuses either half alone.
- Only the `hw-inventory` loader writes a 16-byte firmware-facts extension at
  BootInfo offset 256 (magic `NOSFWF`, version 1, size 16, one state byte,
  zero reserved bytes). The BootInfo flags and ABI are unchanged; every other
  loader leaves the bytes zero, which never validates. `EFI_NOT_FOUND` becomes
  `ABSENT`; any other non-boolean result becomes `Unreadable`, which the kernel
  reports as `HWINV:FAIL:SECUREBOOT`. The value is an observation, never
  evidence of a verified boot chain.
- PCI identity is read through legacy mechanism 1 from configuration offsets
  `0x00`, `0x08` and `0x0C` only, for every present segment-0 function of any
  header type. No configuration write, BAR mapping, command-register change or
  driver start occurs.
- The DMAR observation restates the kernel's ACPI discovery result; it is table
  presence and DRHD count, not evidence of DMA containment.
- Pass and `HWINV:FAIL:*` paths end in a `CLI`/`HLT` loop. Panic and M1/M2
  failure paths still use port `0xF4`; this remains a protocol blocker.

**Evidence limit:** the QEMU runner compares the block with the configured
CPUID vendor/hypervisor bit, the emulated VT-d unit and QMP `query-pci`, and
`hw_evidence` rejects the result as a virtual capture. No physical PC, image,
signed record or H06 acceptance is claimed; `SECUREBOOT=0`/`1` is host-tested
only.

## ADR-PROGRESS-EVIDENCE-20261124 — Show delivered work alongside fixed acceptance

**Status:** scoped implementation of the project owner's 24 November request to
reassess progress and expose implementation and verification separately.

**Problem:** the binary score can stay constant while substantial parts of open
criteria are implemented and tested. Old task blockers and repeated score-only
updates can also conceal a criterion whose original evidence requirements are
now satisfied. Work volume itself does not justify acceptance points.

**Decision:** preserve rubric v1, all criterion IDs, weights and acceptance text.
Add a versioned, dated, source-bound `delivery_review` inventory to the existing
`docs/progress.json`. The existing generator renders its scoped requirements,
implementation states, distinct evidence levels and remaining acceptance gaps in
README/PROGRESS. This is a reporting view of the same criteria, not a second
queue, execution controller or weighted completion score.

Requirement counts are descriptive, unweighted and unequal in size. A host core,
target build or native fixture is credited only within its recorded scope. No
conversion to an OS implementation percentage, release probability or remaining
time is defined. The snapshot date/source stay independent of later task updates;
changing them requires reassessment of the covered rows. Historic evidence and
task events remain intact; superseding facts are recorded explicitly.

**Acceptance consequence:** the 30 previously open criteria were reassessed on
main `2eb1c63d2afdf658f0e71d31f3542e320a7dd041`. B01 earns its existing two
points because the exact-source seven-case native gate now demonstrates all
literal entropy, lifecycle and management/ordinary-app authorization clauses.
The original artifact and clause mapping are retained under
`verification/progress-reassessment-20261124/`. No criterion is broadened or
weakened, and the other open criteria receive no partial points.

**Validation and review:** generator negative/round-trip tests, preservation of
all original rubric fields except B01 status/evidence, independent review of
the protected-file diff, and required hosted checks. Revise the reporting method
if its requirement boundaries or evidence vocabulary change; never treat a
reference-presence check as an evidence-content audit.

## ADR-MERGE-DELEGATION-20261123 — Workstreams handle their own pull requests end to end

**Status:** decided by the project owner on 23 November 2026. It supersedes the
merge-approval part of the decision of 13 November 2026 for the six named
workstream sessions only. `ADR-MERGE-HOLD-20261113` stays in force.

**Context:** routine repair and merging of every pull request ran through one
integration coordinator. That made the coordinator an approval bottleneck and
produced conflicting writers on the same branches.

**Decision:** the six workstream sessions (S09, U, N, UI/UX design, B and H)
handle their own pull requests end to end, including pull requests that were
already open, without further approval. Each may:

- address review feedback;
- resolve merge conflicts and failing checks;
- verify, commit and push;
- squash-merge;
- confirm the merge result and update its own progress assessment.

**Clarification of 24 November 2026:** the owner's autonomous backlog-drain
assignment permits necessary repository repair, review handling, integration,
auto-merge and guarded squash merges. An applicable explicit mandate already
supplies permission; it does not prove tests or acceptance. The following procedure
also preserves continuous progress by future workstream sessions.

A workstream may merge only when all of these hold:

- Required checks and acceptance gates applicable to the changed behaviour pass
  for the current head or validated merge group. Respect the CodeQL ruleset.
- Blocking feedback is handled; threads are resolved with an explanation and
  evidence. Optional checks are assessed for real defects, but an unrelated check
  or review-bot quota failure is not an independent reason to stop integration.
- Dependencies are integrated in order. A source PR merged only into an integration
  branch remains unfinished until the combined work is verified on `main`.
- Acceptance outside the required checks carries `acceptance-pending` until a
  trusted `acceptance-released: <sha>` comment links passing evidence on the exact
  head. The label is not removed to bypass acceptance. Reused content evidence
  requires matching relevant source, dependencies, inputs, configuration and
  environment; required hosted status reporting still applies to the current head.

Before every mutation, re-check the head, base and ownership. Direct merges use
`gh pr merge <N> --squash --match-head-commit <sha>` or the equivalent expected-head
API. Auto-merge is permitted after content review, dependency integration, feedback
handling and applicable non-required acceptance are complete, while required checks
or the controlled merge order finish. Disable it before pushing changes or changing
the base; a changed head requires delta review and renewed applicable evidence before
re-enabling it. This relies on single-writer ownership, branch protection and the
exact-head acceptance hold; it does not give an unreviewed later push approval.

Waiting for a PR merge or CI does not suspend the entire workstream. Continue
independent work in a separate branch, or create a correctly stacked dependent
branch. Record its original delta before a parent squash/rebase to avoid losing
child changes or reintroducing parent changes. Return to pending integrations and
verify actual main inclusion; a draft, armed auto-merge or queue entry is unfinished.

**Limits:**

- One active writer per branch or worktree, with explicit handover before a
  different writer starts.
- No direct push to `main`, no force-push, no `--admin`, no bypass of
  protections, and no weakening of tests or evidence to make a pull request
  pass.
- Files on the Protected files list in `AGENTS.md` still change only through
  a lead-scoped pull request reviewed by someone other than the merging
  workstream. The exceptions are a new child directory for new evidence and
  the workstream's own progress assessment, which this decision delegates.
- Unattended pipeline workers keep the draft-only rule and execution hold. Other
  sessions need an applicable explicit owner mandate, which may include integration.

**Coordinator role:** the integration coordinator handles only cross-stream
dependencies, ownership conflicts, orphaned pull requests and escalations that
a workstream cannot resolve itself. It is not an approval step.

**Alternatives:** keep one integration coordinator as the only merger (rejected:
it serialised about 35 open pull requests and invited competing writers on the
same branches); allow every session and pipeline worker to merge (rejected:
too broad for unattended workers that must not change their own evaluators).

**Rationale:** the workstream that wrote and verified a change has the best
context to repair and merge it. Branch protection (required checks, strict
up-to-date base, squash only, resolved threads) and `merge-hold` remain the
technical boundary, so the delegation moves only the decision, not the gate.

**Consequences:** each workstream carries its own review, CI and merge
workload. The coordinator stops being a queue and keeps only exceptions.
Writer conflicts shrink because one session owns each branch; stacked pull
requests across workstreams still need explicit coordination.

**Security invariant:** no merge without the required checks, the applicable
acceptance hold and resolved threads on the exact merged head; no bypass of
branch protection; files on the Protected files list stay lead-scoped.

**Test evidence:** none applies to a governance text change. The governance
and documentation checks (`tools/tests/test_agent_governance.py`,
`tools/check_english_docs.py`, `tools/progress.py --check`) pass on this
change; they do not prove that sessions follow the rule.

**Migration and rollback:** to restore the previous model, the project owner
announces the rollback; from then on workstreams and their pull-request
workers issue no further `gh pr merge` calls, stop their merge drivers and
hand their open pull requests back to a single coordinator in draft or
"final" state. As a safety check, workstreams and the coordinator (for
orphaned pull requests) disable any auto-merge found armed
(`gh pr merge <N> --disable-auto`) and confirm that none remains; only then
does the project owner revoke this ADR.
Pull requests already merged under this ADR stay merged and are judged on
their recorded checks.

**Refutation and review criteria:** review this decision if a pull request
merges with a failing or pending required check or applicable acceptance gate, an unresolved finding, a
bypassed protection or a merge-hold released without its acceptance; if two
writers collide on one branch; or if a merge reaches `main` from a head that
was not reassessed after a push. Any such case is recorded and may restore
owner approval per merge.

**Owner:** project owner (decision); each workstream for its own pull requests;
the lead agent for integration and decision management.

## ADR-B02-KEY1-RELAY-20261123 — Write-ahead key state committed by init on behalf of the Key Service

**Status:** project-owner decision of 23 November 2026, given in the B-track
session as two answers: "Init relays commits" (who writes the key-state
snapshot) and "Service holds it (Recommended)" (who keeps the external
secret). The option "Supersede ADR-051" (a block/volume right for the Key
Service) was offered and not chosen. The owner did not decide rollback
anchoring, a trusted generation floor, product secret custody or production
use; they stay open. This ADR authorises a TEST-profile implementation only.

**Context.** B02-T2 added a host-only write-ahead contract:
`Service::generate_durable`/`revoke_durable` and `wire::dispatch_domain_durable`
reply `Ok` to GENERATE/REVOKE only after a `KeyStateStore` committed the
sealed 120-byte snapshot in the same call (`services/keyring`, tests in
`services/keyring/tests/write_ahead.rs`). No native store exists, because
ADR-051 gives the Key Service no volume right and keeps the external secret
outside the service. Until now the init/app protocol (`KEYS_READY` →
commit → `PERSISTED`) orders key state before ciphertext, and an
uncooperative app can bypass it.

**Decision.**
1. **Commit ingress.** A new kernel endpoint, the key-state commit ingress,
   is received only by init. The Key Service holds the only SEND handle; no
   app, worker or other service holds it. The FR-I01 manifest checks this as
   it checks `APP_ADMIN_HANDLE` (INT3-FU-116): object comparison per handle
   across every non-init handle table plus an exact count. The Key Service
   still has no block or volume right, so alternative (a) of ADR-051 stays
   rejected.
2. **Secret custody (amends ADR-051).** Init supplies the external secret to
   the Key Service once per instance through administrative ingress: with the
   existing RESTORE_PERSISTENT on a cold start, or with one new admin-only
   opcode on a fresh instance. The opcode number is allocated in the
   implementation change. Domain and RANDOM ingress reject it before any
   processing, as they reject opcodes 11/12. The service keeps the secret in
   zeroizing memory for the lifetime of the instance. It never returns,
   logs or re-exports it. A second supply with any secret is refused, and
   the secret is wiped when the instance ends. This replaces the sentence of
   ADR-051 that the secret "is not retained by the service"; the remainder of
   ADR-051 stands.
3. **Commit protocol.** For GENERATE or REVOKE on a write-ahead domain
   ingress, the service seals generation n+1 bound to the volume identity.
   It then sends `(generation, blob)` to the commit ingress in a synchronous,
   bounded call. Init writes the blob to key-state object 8 in one
   crash-consistent volume transaction in domain 1, flushes, and replies
   `COMMITTED(generation)` or `NOT_COMMITTED`. Mapping to
   `StoreError`/`WriteAheadError`:
   - `COMMITTED` with the same generation: commit succeeded; the domain caller receives `Ok`.
   - `NOT_COMMITTED`: `Failed`, which replies `NotReady` and allows an explicit retry. Init may send it only when it has not begun the transaction.
   - Any other outcome: `Unknown`, which replies `Degraded` and latches the instance. This covers a timeout, a restart of init, a mismatched generation, a malformed reply and a failure after the transaction began.
4. **Authority and ordering.** Init accepts a commit by the kernel endpoint
   object, never by a payload field or PID. The service trusts only the reply
   to its own call. Init completes RESTORE or the secret supply before the
   Key Service accepts domain traffic. After that, init makes no blocking
   call into the Key Service while a commit request can be outstanding, so
   the synchronous commit cannot deadlock against administrative ingress.
   Every wait is bounded; expiry is `Unknown`. The `KEYS_READY`/`PERSISTED`
   cooperation may stay in the test profile as additional evidence. The
   enforcement point becomes the Key Service's `Ok`.
5. **Limits.** The secret remains the compile-time test constant in the
   init ELF of the disposable image (ADR-051). There is one writer and one
   instance, and the volume identity remains the profile constant
   `NOS-SYSTEM-VOL01`. Cold rollback anchoring and a trusted generation
   floor stay out of scope (B02-T3, which needs a separate owner decision).

**Consequences and residual risk.** The service process now holds both the
master and the wrapping secret. Someone who can read that process memory
already has every key. Retaining the secret adds the ability to produce
snapshots that init accepts for the next cold start, which forges lineage.
The cost was accepted to keep the KEY1 ABI unchanged. The other option
offered, init keeping the secret, needs a pending status or deferred domain
reply and per-operation exports; the owner did not choose it. B02 earns no
credit from this ADR. Acceptance needs native QEMU evidence of:
- a normal commit
- `NOT_COMMITTED` before any write, mapped to `NotReady`
- a commit timeout that latches `Degraded`
- an app without commit SEND (manifest)
- the existing host tests

## ADR-H05-S3-SYSTEM-20261123 — Test-profile S3 from the system profile: syscall 0x91 behind POWER and the storage gate

**Status:** technical decision of the executing hardware-track lead on
23 November 2026, inside the H05 implementation mandate (H05-T4). It is not a
project-owner approval and not a product power API. It exists only behind the
opt-in test feature `test-system-s3`; no published ABI version contains it.
Physical sleep, product suspend UX and device re-initialisation stay open.

**Context.** H05-T3 (ADR-H05-S3-WAKE-20261119) suspends and resumes only the
single-CPU IF0 bootstrap. H05 requires sleep/wake that is recoverable for a
running system: ring-3 processes must continue after a wake. QEMU resets every
device on wake (the CPU and all PCI functions restart; OVMF then transfers to
the waking vector), so no virtio device state survives. The M3 scheduler counts
PIT ticks; the TSC is used only for saturating diagnostics.

**Decision.**
1. **Syscall `0x91` SUSPEND(power handle, 0, 0, 0, 0, 0)**, the next number
   after `0x90` (POWER_OFF), allocated like `0x74` in ADR-H04-LINK-20261119.
   Validation order equals POWER_OFF: zero reserved arguments (-1), the
   non-delegable init-only POWER object (`check_power`), an S3 plan that
   discovery accepted with the FACS proven ACPI NVS (-14 otherwise), then the
   same storage gate as S5 (`storage_closed`: volume shut down, no block
   object, entropy broker idle; -8 retry, -16 never). Amendment H05-R1
   (23 November 2026): after that gate, SUSPEND also returns -8 while the
   volume broker is not quiescent (`volume::sleep_quiescent()`: a request held
   by the Policy, including a completed result its owner has not consumed, a
   non-Ready phase, a stored error or a reset in progress); the refusal precedes
   any I/O and is counted. One granted suspend per boot; a second request
   returns -14.
2. **Synchronous sleep in the caller's syscall.** IF stays 0. The kernel reuses
   the H05-T3 wake page, token, FACS arm/disarm, canary and post-wake checks.
   The handler runs under the kernel root CR3 (amendment H05-T6b: the one the
   32-bit wake stage loads from the static `NOS_S3_CR3` in the identity-mapped
   kernel image, written once by `prepare`; wake-page offset 8 is reserved and
   unread);
   the wake path verifies that root, and the normal scheduler path reloads
   the caller's process CR3 on return. It reloads the GDT/IDT/TR through the
   shared loader, reprograms the PIC and PIT exactly as at boot and audits
   every live process address space. Only after every check passes does the syscall return 0 to the
   caller; any failed check stops the VM (fail closed, no partial resume).
3. **Devices are lost, not resumed.** Because the storage gate already closed
   every block authority and the entropy broker is idle, no DMA can be in
   flight. After a wake the kernel marks the system as resumed: every later
   block or entropy request returns -14 before any ring, doorbell or counter
   update (the profile has no network device). Amendment H05-T5a (23 November
   2026): the refusal precedes the authority lookup, after the argument-shape
   checks, so any well-formed post-wake request takes it. Init's 0x60 and 0x40
   requests after the wake each return -14 natively, and the kernel requires
   the exact tagged log `[(0x60,-14),(0x40,-14)]`
   (docs/verification/h05-s3-wake-refusal-20261123/). An authorised post-wake
   request is not exercised. The only way forward that the
   profile exercises is POWER_OFF (S5), which re-runs the storage gate.
   Device re-initialisation after a wake is a separate slice.
   Amendment H05-T5b (23 November 2026): under the opt-in profile
   `test-system-s3-reinit` only, after a validated wake and with an idle broker,
   the kernel re-validates virtio-rng (same BDF and IDs, no BAR sizing) before it
   lifts the entropy -14. It restores BARs only from the all-cleared state and
   requires the readback and the capability regions to equal discovery. It then
   resets with acknowledgement, checks pool ownership, initialises (bus mastering
   only there) and runs a discarded 32-byte self-test. `next_ticket` stays
   monotonic, so a pre-sleep ticket is refused (-1). Block devices stay lost
   (-14). Any failure exits 0x11. The post-wake delivery is driven by the kernel
   through the Key Service's own entropy capability, copied into kernel state at
   manifest time, into an unused heap page that is wiped with IF=0. Init's own
   post-wake request is refused by authority (-6), so no ring-3 process receives
   post-wake entropy in this fixture. On local QEMU 10.2.1 the BARs are cleared
   by the wake (`BARS=RESTORED`). The oracle also accepts `BARS=KEPT` for the CI
   QEMU 8.2.2 path, which is unmeasured
   (docs/verification/h05-s3-reinit-20261123/). In `test-system-s3`, devices
   remain lost.
   Amendment H05-R1 (23 November 2026): the refusal also covers the kernel's
   own broker paths, not only syscalls. Volume `reset` (owner teardown) and
   `advance` (broker tick) return before any device access after a wake, and
   keyring `call` (0x60), `advance` and `quarantine_pending` do so through
   `device_lost()` = resumed and not yet re-initialised, so only the validated
   re-init of `test-system-s3-reinit` lifts the keyring latch. Every broker
   device call site is preceded by an access counter; after CLEANUP the kernel
   requires zero post-wake device accesses outside the accounted re-init path,
   and live (non-zero) counters at the wake.
   Amendment H05-T5c (25 November 2026, slice 1): under the opt-in profile
   `test-system-s3-blk-reinit` (built on `test-system-s3-reinit`, mutually
   exclusive with `test-system-s3-held-completion` by `compile_error!`), after a
   validated wake and the virtio-rng re-init, the kernel re-validates and
   re-initialises the virtio-blk primary lane (00:05.0, `Function::Primary`
   only) and runs a kernel-driven READ of reserved canary sector 0 whose bytes
   are discarded. No block capability is issued, `block_objects == 0` and the
   SUSPEND storage gate are unchanged, and init's post-wake 0x40 with its POWER
   handle now reaches the authority check (-6) instead of -14. The volume latch
   (four `resumed()` call sites in three functions: `device_call` twice,
   `reset`, `advance_lane`) now uses `device_lost()` = resumed and not
   re-initialised; in every other
   S3 profile block devices stay lost (-14), and the rng path is unchanged.
   Project-owner decisions of 25 November 2026, as relayed in the H05-T5c task
   brief (Q4 quoted verbatim from it; Q1-Q3, Q5 and Q6 summarised in full):
   - **Q1** (ring-3 block re-grant), **Q2** (mandatory remount after a wake) and
     **Q3** (key erasure before SUSPEND) are later slices; this slice
     implements none of them and relaxes no gate.
   - **Q4** "a refused blk re-init must NOT stop the system. It quarantines
     lane 0 (offline, every later device request gets -14 before any device
     access, no automatic retry or re-init, no false acknowledgement, no
     replay) and the system continues to the existing S5 gate and recovery."
     This overrides the review's I1 "exit 0x11". The lead decision of
     25 November 2026 reserves one distinct refusal code, -24, for a block
     request on a quarantined lane (-13/-14 keep their offline/missing
     meaning); it replaces the latch's -14 in the volume syscall result once
     lane 0 is quarantined. The quarantine stops the device when `rediscover`
     had validated it (status 0, bus mastering off), quarantines the same lane
     Policy (`reset_completed(false)`) and logs exactly one
     `VOLUME:S3:REINIT:QUARANTINE:<REASON>`. `lane.offline` is left unchanged:
     it feeds the S5 durability gate (-16 "never this boot"), and the data
     was proven durable by the storage gate before SUSPEND, so S5 still passes
     on unchanged gate code and a cold recovery follows.
   - **Q5** full identity: dword 0, revision/class (0x08), subsystem (0x2c)
     and the capability pointer are compared with config reads before any
     config write; the parsed regions after the BARs; offered and negotiated
     features, `queue_size_max`, the notify offset and the capacity after
     `initialize_profile`. No BAR is sized after the wake (`rediscover`, a copy
     of the rng `classify_bars` rule). Reset with acknowledgement, the five
     pool pages still kernel-owned, zeroed rings through `initialize_profile`,
     MSI/MSI-X re-disabled and INTx-disable set. The epoch advances through
     `start_reset` -> `reset_completed(true)` on the same Policy object; the
     ticket floor is kept and a pre-sleep ticket or epoch is refused (-1).
     `Policy::new()` is never used. The profile has no IOMMU, so DMA isolation
     against a misbehaving device is not claimable.
   - **Q6** deterministic, labelled, test-only injections
     (`test-system-s3-blk-reinit-busy`, `-identity`, `-capacity`,
     `-no-reset-ack`) corrupt one input of a real check (lane state, the saved
     subsystem dword, the expected capacity, the acknowledgement) so that only
     that check refuses and the real quarantine path runs; none fabricates
     device success. Evidence is tagged injected, emulator or physical.

   Slice order: 1 kernel re-init and quarantine (this amendment); 2 the Q3
   key-erasure SUSPEND gate (with B); 3 the Q2 remount (with S); 4 the Q1
   fresh-generation ring-3 re-grant. Invariant: no ring-3 block re-grant and
   no relaxation of `block_objects == 0` before slices 2 and 3 exist.
   Invariants as adapted from the H05-T5c security review: I1 re-init only
   after a validated wake with lane 0 quiescent, otherwise quarantine (not
   exit); I2 00:05.0 only; I3 no BAR sizing, BARs restored only from the
   all-cleared state; I4 identity equality before any config write; I5 no MMIO
   before `rediscover` succeeded; I6 acknowledged reset and pool ownership
   before reuse; I7 negotiated-state and capacity equality; I8 fresh zeroed
   rings; I9 same Policy, epoch + 1, monotonic tickets; I10 the latch lifts only
   after I1-I9 and the self-test; I11 every re-init device access is counted in
   an accounted volume subset and `io_after == 0` holds over the keyring and
   volume accounted totals; I12 no new authority, the canary READ never reaches
   a process; I13 interrupts quiesced at the function, and the post-wake
   `timer_init` masks every 8259 line but IRQ0 (the INTx line is read from
   config 0x3c and must be masked, else quarantine); I14 the final S5 gate
   re-check still passes (the self-test is a READ outside the broker counters).
   Negotiated-state checks can only run after `initialize_profile` set
   DRIVER_OK and bus mastering; a refusal there resets the device first.
   Evidence boundary: QEMU `pc-q35-8.2`/OVMF/TCG; the success path and the
   four injections are native on local QEMU 10.2.1 (`BARS=RESTORED`); the
   injections are labelled kernel-side injections, not device faults; no BAR
   or capacity change is provoked by the emulator; no physical S3; the
   ticket floor is checked natively on a Policy copy and by a host test
   (docs/verification/h05-s3-blk-reinit-20261125/).
4. **Time.** Sleep time does not advance the tick clock; deadlines therefore
   pause across S3. This is recorded, not hidden.

**Evidence boundary.** QEMU `pc-q35-8.2`/OVMF/TCG only, wake only through QMP
`system_wakeup`; no AML `_PTS`/`_WAK`, no SMP, no device survival, no
physical firmware. The oracle is raw QEMU exit 0 (the guest's own S5
write after the wake), the kernel markers in order (plan at bootstrap, one
`ENTER`/`RESUMED` pair after `SYSTEM:MANIFEST:PASS` and before
`SYSTEM:BROKERS:PASS`), the kernel's exact SUSPEND result log
`[-8, -6, -4, -1, 0, -14]`, a timer interrupt after the wake, and the
harness's QMP `SUSPEND`/`WAKEUP` events. A rejected wake token exits 0x12
(raw 37) in every S3 profile and can never satisfy that oracle. The amendment of
23 November 2026 aligns the CR3, device-refusal and exit wording with the
implementation (H05-T4). Amendment H05-T6 (23 November 2026): the wake path
validates the ARMED token before it reads `SAVED` or restores any register, a
rejected wake exits the dedicated code 0x12 instead of 0x11 (or 0x10 in
`native-acpi-s3`), and FACS stores outside the extent recorded from the
validated plan are refused with `Permission` before any mapping
(docs/verification/h05-hardening-20261123/). Amendment H05-T6b (23 November
2026): in `native-acpi-s3`, a refused S3 plan and a cleanly disarmed FACS store
failure exit the dedicated code 0x13 (raw 39), so no rejection reaches the pass
code 0x10; `test-system-s3` is unchanged (SUSPEND returns -14 without a plan).
The fixtures `test-acpi-s3-unarmed-wake` (raw 37, page token 0 so only the
`ARMED == 0` clause refuses) and `test-acpi-s3-facs-end` (raw 39) exercise the
refusal order natively (docs/verification/h05-s3-rejections-20261123/); hosted CI
runs them only after `native-power.yml` gains their steps. Loading CR3 from the
kernel image moves one field off the wake page; the page's 16-bit code, GDTR and
far pointer still decide execution before the ARMED check, so the wake page's
trust is unchanged. Amendment H05-R1 (23 November 2026): the opt-in profile
`test-system-s3-held-completion` (mutually exclusive with
`test-system-s3-reinit`) leaves init's completed SHUTDOWN flush held, expects
one quiescence -8 and then applies a labelled gate-defect injection
(`held_check_skipped`, compiled only into that profile) so the sleep happens
with the held request. Its exact SUSPEND log is `[-8, -6, -4, -1, -8, 0, -14]`
with `HELD_REFUSALS=1`, zero post-wake device accesses and one suppressed reset
(docs/verification/h05-s3-held-completion-20261123/). Natively only the volume
`reset` latch is provoked; the other latches are checked by source shape and
the access counter only, not by a provoking fixture.

## ADR-H04-LINK-20261119 — Test-profile NIC link query 0x74 and link-down refusal -23

**Status:** technical decision of the executing hardware-track lead on
19 November 2026, inside the H04 implementation mandate. It is not a
project-owner approval and not a product network API. It exists only behind
the kernel features `test-system-net-link` and `test-system-net-link-storm`.
`abi_info` stays 1.5, and applications must not rely on either number. It
accepts no criterion.

**Decision.**

1. **Syscall `0x74(handle, 0, 0, 0, 0, 0)`.** A read-only link query for the
   owner of the NetworkSource capability.
   - Authorisation uses the same `authorize` path as `0x70`–`0x72`:
     capability type plus object generation key.
   - Any nonzero argument 1–5 returns -1.
   - On success it returns only `epoch << 1 | up`.
   - It returns -8 before the first kernel sample, -15 when the device did
     not negotiate `VIRTIO_NET_F_STATUS`, and the broker's offline code -14
     after any stop or latch.
   - `0x74` is the next number after `0x73` (RTC read). It was unused in
     `kernel/src` and `sdk/runtime` at the merge.
2. **Error -23 `LINK_DOWN`.** `0x70` and `0x72` return -23 while the link is
   down or not yet observed.
   - The check runs after the argument, capability and offline checks.
   - It runs before any descriptor publication, bounce-page (DMA) write,
     doorbell or copy-out.
   - Precedence: argument errors, then capability errors, then the
     broker's existing offline code, then -23, then -8. The offline code is
     -14 after budget exhaustion or owner exit and otherwise the mapped stop
     cause (for example -12 after an I/O stop). That path is shared with
     feature-off builds and is deliberately unchanged.
   - -23 was the first free value: -1 to -22 are in use, -22 for invalid
     clock arguments.
3. **Kernel-owned link state.** The kernel owns a strictly increasing
   observation serial and link epoch.
   - It takes one coherent sample per PIT tick, with at most 16 reads.
   - The device's u8 configuration generation is only a change hint. It
     yields at most one conservative synthetic down/up pair (epoch + 2).
   - The budget is eight transitions per device lifetime. Exhausting it, or
     any device reset, latches through the existing `stop()` and reset
     acknowledgement. There is no reinitialisation.
4. **Epoch isolation.** At every observed edge, down or up, real or
   synthetic, the kernel first takes a completion that is already in the RX
   used ring, then drops it and reposts its pinned page. No frame observed
   before an edge is delivered after it. `0x74` returns -14 after any stop
   or latch, so -15 means only that `VIRTIO_NET_F_STATUS` is absent. (Added
   20 November 2026 after an AI security review; see
   `docs/verification/h04-link-review-20261120/`.)
5. **Without `VIRTIO_NET_F_STATUS`** the link is treated as active
   (virtio 1.2 §5.1.4.2).

**Consequences.**

- A product link API, its capability shape, Wi-Fi and Bluetooth need a
  separate decision.
- The QEMU harness drives link state only through `set_link` on a private
  QMP socket.
- Revisit this decision if any of these occur:
  - a feature-off build changes `.text` or `.data`;
  - any broker or device counter changes during a down window;
  - an event appears after a latch;
  - a stale RX frame crosses an epoch.
- Evidence is QEMU 8.2.2/TCG only. There is no physical NIC, and QEMU has no
  802.11 or Bluetooth model.
- Link state is sampled once per PIT tick. A frame that completes during
  a hidden down window, which opens and closes between two samples, is
  first seen in the current epoch. The u8 configuration generation narrows
  but cannot remove this sampling limit. Exactly 256 changes between two
  samples are undetectable. The edge harvests are covered by host model
  tests and source inspection; no native case drives two or three
  `set_link` changes within one tick.

## ADR-H03-SND-FIXTURE-20261119 — Kernel-only virtio-snd PCM output fixture for the fixed QEMU 8.2 profile

**Status:** technical decision of the executing hardware-track lead on
19 November 2026, inside the H03 implementation mandate. It is not a
project-owner approval. It grants no capability, syscall, IPC route or app
path, and it accepts no criterion. Owner decisions B1 (AUDIO right), B2
(syscall number) and B4 (reserved VM-FR device address) stay open.

**Decision.** `native-virtio-snd-probe` is an IF0, single-CPU, polling M2
bootstrap profile. It drives QEMU 8.2 `virtio-sound-pci` at the fixture
address 00:08.0 as a kernel-only transport. The profile is exclusive with
M3, framebuffer, SMP, RTC, ACPI S3/S5 and xHCI (`kernel/src/main.rs`).

- **DMA:** only six pinned, identity-mapped RW/NX kernel pages, kept for the
  whole boot, including after a fence or quarantine. PCM pages are
  device-read-only by protocol. The device may write only used rings, the
  control response, event buffers and TX status words.
- **No DMA isolation:** there is no IOMMU and no DMA-isolation claim in this
  profile.
- **MMIO:** only the validated common, notify and device capability pages,
  mapped UC/NX in the audited `0x3f200000` window. The window's cfg gates
  move to the internal umbrella feature `if0-device-window`, shared with the
  xHCI fixture.
- **Refusals:** `ACCESS_PLATFORM` is refused. The capture queue is never
  enabled. A second RELEASE is refused by the stream state machine.
- **Timing and oracle:** the runner uses `-icount shift=0,sleep=off`, so the
  guest TSC, PIT and QEMU's audio/WAV timer share one virtual clock. The
  oracle is the frozen `drivers/virtio-snd/TRANSPORT.md` §7: exact frames,
  with at most 4800 silent frames before and after.

**Recorded QEMU 8.2.2 deviations.**

1. *PCM_INFO used length.* QEMU 8.2.2 writes the PCM_INFO records but pushes
   `sizeof(virtio_snd_hdr)` (4) as the used length for every control
   response. The lead checked this in QEMU tag `v8.2.2`,
   `hw/audio/virtio-snd.c`
   (<https://gitlab.com/qemu-project/qemu/-/blob/v8.2.2/hw/audio/virtio-snd.c>,
   SHA-256 `99a13ef25d0729dd46e9947dea12986598b5b9242422a052fb05214f22e7794d`,
   identical in the release tarball): `virtio_snd_handle_pcm_info` copies
   the records after the header (lines 247-251), and `process_cmd` pushes
   used length `sizeof(virtio_snd_hdr)` (line 752).
   - For PCM_INFO only, the driver accepts 4 as well as the exact
     `4 + 32n`. Every other message keeps exact lengths.
   - The response buffer is zeroed before publication and parsed strictly,
     so a genuinely header-only answer fails (`channels_min` 0).
   - The used length is a hint inside a device-writable page that the kernel
     already treats as untrusted. Accepting it does not widen what the
     device can write.
2. *Drain before STOP.* QEMU disables the voice on STOP and drops buffered
   samples. The driver drains while RUNNING with nothing outstanding, then
   sends STOP, then requires a quiet window.
3. *Quiescent fence for re-initialisation.* `virtio_snd_reset` keeps TX
   elements it has already popped and replays them after re-initialisation.
   The accepted `fence-reinit` case therefore fences with no TX outstanding.
   An in-flight fence must quarantine (the `quarantine` case) and cannot be
   recovered on this QEMU.

**Consequences.** The evidence is QEMU/TCG emulator WAV output only. It is
not H03 acceptance, audible output, latency, physical playback or an app
audio path. The runner refuses QEMU versions outside 8.2. The ring-3 audio
service, its capability and its syscall wait for B1, B2 and B4.

## ADR-H05-S3-WAKE-20261119 — Own-kernel ACPI S3 wake contract for the fixed QEMU/OVMF profile

**Status:** technical decision of the executing hardware-track lead on
19 November 2026, inside the H05 implementation mandate. It is not a
project-owner approval, grants no new capability to any process and accepts
no criterion. H05 stays open.

**Context.** The H05 S3 slice (`native-acpi-s3`) puts the independent kernel
to sleep through ACPI S3 and resumes it. Three facts were measured in this
session on QEMU 8.2.2 `pc-q35-8.2` with Ubuntu `OVMF_CODE_4M.fd` under TCG:
- QEMU builds an ACPI 1.0b FACS: version byte 0, length 64, every other byte
  zero. The parser merged in #371 accepted only versions 2 and 3, so
  `origin/main` rejects the real tables with `UnsupportedFacsFields`.
- With only the ACPI 1.0b real-mode waking vector armed, edk2's own
  `AsmTransferControl` far-returns into a 16-bit code segment and QEMU TCG
  then truncates EIP to 16 bits. The firmware executes OS memory (at
  `.0 & 0xffff`) and stops with `#BP` before reaching any OS code. This is a
  firmware/emulator interaction, observed with the transfer stub located at
  `0xf743ad5` and the fault at `0x93e1`.
- edk2's `S3ResumeBootOs` reads `X_Firmware_Waking_Vector` (offset 24) for
  every FACS version and, when it is nonzero without 64-bit wake, enters it
  in 32-bit protected mode with paging disabled (`AsmDisablePaging64`).
  OVMF also leaves `SCI_EN` clear after the wake.

**Decision.**
1. `parse_s3` accepts FACS versions 0 and 1 with exact per-version reserved
   fields (version 0: offsets 24..64 zero; version 1: offsets 33..64 zero and
   a 32-bit extended vector), alongside the existing versions 2 and 3.
2. A `WakePlan` describes every FACS store: the 16-bit real-mode vector
   always, offset 24 when the FACS version defines it, and OSPM flags with
   64BIT_WAKE_F cleared for versions 2 and later. No firmware long-mode entry
   is ever requested.
3. `ExtendedVectorPolicy::Edk2Version0Extended`, used only by the fixed
   QEMU/OVMF kernel profile, additionally stores the kernel's 32-bit entry
   into offset 24 of a version-0 FACS. Discovery has proven those bytes zero,
   and the disarm plan restores zero right after every wake. The default
   policy `Specification` refuses a 32-bit entry for a version-0 FACS. `Edk2Version0Extended`
   is valid only for the QEMU/OVMF profile; physical firmware must not use it
   without its own evidence, and this restriction does not apply to the default
   `Specification` policy (integration review of #409, 23 November 2026).
4. Both entries converge on one 32-bit stage and one 64-bit token check: only
   the armed generation resumes the saved context; any other wake is reported
   and stops the VM (exit 0x12, raw 37, since H05-T6). Amendment H05-T6b
   (23 November 2026): a plan that discovery or `resolve` refuses, or a FACS
   store that fails and disarms, exits 0x13 (raw 39) instead of the pass code
   (ADR-H05-S3-SYSTEM-20261123, evidence boundary).
5. After a wake the kernel restores `SCI_EN` when it was set before sleep,
   matching Linux's documented "bug compatibility" behaviour, and reports
   whether it had to.

**Alternatives considered.**
- Real-mode vector only (strict ACPI 1.0b). It cannot resume under QEMU TCG,
  the project's CI and cloud environment, because of the firmware transfer
  stub. The real-mode stage stays armed for firmware that ignores offset 24.
- Rewrite the FACS version byte. That changes firmware-owned structure
  semantics; rejected.
- Use KVM, where the stub runs. KVM is unavailable in the cloud environment
  and the hosted runners; the TCG behaviour would remain untested.

**Rationale.** Offset 24 is the same field ACPICA already writes (to zero)
in a version-0 FACS longer than 32 bytes. Confining the nonzero store to an
explicit, named policy, a zero-validated field and an immediate restore keeps
the deviation bounded and visible in the serial plan line.

**Primary sources (read 19 November 2026).**
- ACPICA, Intel Corporation, tag `R2025_04_04`,
  `source/components/hardware/hwxfsleep.c`, `AcpiHwSetFirmwareWakingVector`:
  sets `FirmwareWakingVector`; when `Length > 32` it sets
  `XFirmwareWakingVector` for `Version >= 1` and clears it otherwise.
  <https://github.com/acpica/acpica/blob/R2025_04_04/source/components/hardware/hwxfsleep.c>
- TianoCore EDK II, tag `edk2-stable202402`,
  `UefiCpuPkg/Universal/Acpi/S3Resume2Pei/S3Resume.c`, `S3ResumeBootOs`:
  takes the extended path whenever `Facs->XFirmwareWakingVector != 0`
  (without 64-bit wake: `AsmDisablePaging64` into 32-bit protected mode),
  otherwise `AsmTransferControl` to the 16-bit vector; and
  `MdeModulePkg/Universal/Acpi/BootScriptExecutorDxe/X64/S3Asm.nasm`,
  `AsmTransferControl` (far return into a 16-bit code segment).
  <https://github.com/tianocore/edk2/blob/edk2-stable202402/UefiCpuPkg/Universal/Acpi/S3Resume2Pei/S3Resume.c>
- Linux, tag `v6.12`, `drivers/acpi/sleep.c`, `acpi_suspend_enter`: after the
  low-level resume, "This violates the spec but is required for bug
  compatibility." followed by setting `ACPI_BITREG_SCI_ENABLE`.
  <https://github.com/torvalds/linux/blob/v6.12/drivers/acpi/sleep.c>
- QEMU 8.2.2 behaviour (version-0 FACS, `SCI_EN` after wake, TCG transfer
  fault) is measured in this session, not taken from a source reading; see
  the evidence README.

**Consequences.** The S3 evidence is QEMU/OVMF/TCG evidence only. The
real-mode entry path is armed but not natively exercised; it reuses the
pattern of the natively verified SMP SIPI trampoline. Physical firmware may
use a version-1+ FACS, where the 32-bit extended vector is the specified
path. AML `_PTS`/`_WAK`, device and service lifecycle across sleep, other
wake sources, SMP and physical hardware remain open under H05.

**Rollback.** Revert the implementing commit; S5 profiles and their parser
paths are unchanged (regression-tested in the same session).
## ADR-S09-RESTORE-FIDELITY-20261119 — L1 history replay first, then a separately reviewed L2 provenance level

Status: accepted by the project owner on 19 November 2026 (decisions R1–R4 and source identity of
[S09-RESTORE-FIDELITY.md](implementation/S09-RESTORE-FIDELITY.md)).

**Decision:**

- **L1 first:** native restore replays the retained versions, names and
  metadata of each object into a destination domain; source revisions and
  logical timestamps are kept in the archive and in the restore report only.
- **R2 (decided 19 November 2026):** tombstone content (deleted, unpurged) is
  excluded by default. Content-free accounting records are included only where
  they are demonstrably required for a consistency check; they never restore
  deleted content.
  History that may predate a delete is not restored either (owner answer of
  19 November 2026: "Nee, weigeren"): L1 replays only the newest versions of
  an object whose revisions are consecutive, because DELETE, like META and
  RENAME, raises the object revision without adding a version.
- **R3 (decided 19 November 2026):** native restore runs only into a
  demonstrably never-used domain. Existing domains are never overwritten or
  merged automatically; merging into a used domain remains the v1 restore.
- **R4 (decided 19 November 2026):** the destination range is sized from the
  derived maximum v2 archive of 494 416 bytes, with separate, separately
  verified capacity for destination metadata.
- **Source identity (decided 19 November 2026):** archive v2 records no source
  volume or domain identifier, also not behind a feature flag. Source
  revisions and logical timestamps are kept as separate historical values; no
  stable domain identity is invented.
  Consequences: header bytes 40..56 of archive v2 stay reserved and zero and
  a reader refuses any other value; the L2 `origin_*` fields carry no source
  identity. Revisited only by a new owner decision.
- **L2 as the long-term target, separately reviewed:** original source
  revisions and logical timestamps are kept as immutable provenance, separate
  from the destination-assigned live revisions and clocks. L2 begins only
  after L1 is completed and independently verified.
- **L3 rejected:** installing source values as live destination revisions and
  clocks would change the crash-recovery invariants (per-domain clock sum
  equal to the generation, `g ± 1` adjacency of the A/B table copies) and the
  cross-domain stamp property tested by
  `stamps_do_not_reveal_other_domain_commits`.

**L2 review criteria (before implementation or activation):** table v3 format
and invariants; crash-recovery tests; compatibility with v1 and v2 archives
and v2 tables; the security boundary of the new wire reads; negative tests.

**Consequences:** the v1 restore remains the only implemented recovery path
and is unchanged. The POSIX ustar/pax export stays a lossy interchange format,
not native recovery. No historical revision or timestamp is fabricated. S09
stays open; this decision changes no score.

## ADR-S09-DUALDISK-20261119 — Test-only second virtio-blk device for the S09 archive fixture

**Status:** approved by the project owner on 19 November 2026 (verbatim: "Yes,
approve B1–B6"). It covers the request in
[S09-DUAL-DISK-FIXTURE-REQUEST.md](implementation/S09-DUAL-DISK-FIXTURE-REQUEST.md).

**Context.** The S09 assignment asks for an independent recovery copy on a
second virtual disk and a cold restore without the source disk. The
virtio-blk driver serves exactly one fixed PCI function, and the volume
profile refuses every block capability on a device other than 0. A second
disk is therefore a device-ownership change in a shared driver and kernel
profile.

**Decision.** A second virtio-blk device is allowed only as a test fixture,
under the request's bounds:
- B1: a closed set of two fixed PCI functions, with `discover()` unchanged for
  its existing callers;
- B2: a feature-gated second device with disjoint DMA pages, and
  `device != 0` still refused outside that feature;
- B3: a portal-only capability on device 1, ending below the disk end, and
  nothing on device 1 for the volume service or slot 2;
- B4: disposable images only;
- B5: no product authority;
- B6: QEMU evidence only.

**Alternatives considered.**
- Keep the archive range on the volume's own disk (the current native slice).
  That cannot show a restore after the source disk is lost.
- Host model only (`services/volume/tests/archive.rs`, five dual-device
  tests). That gives no native evidence of a second device.
- A general multi-device virtio-blk driver that probes the bus. That is a
  wider ownership change than the fixture needs, and it could claim the boot
  ESP.
- A second device in every profile. The fixture needs it in the volume
  profile only.

**Rationale.** A closed enum of two fixed functions keeps the driver's
`unsafe` port-I/O justification bounded, because no caller can supply an
arbitrary bus/device/function number. A dedicated test feature keeps every
existing profile unchanged. A portal-only `BlockScope { device: 1, .. }`
reuses the capability engine as it is; routing stays in the kernel broker.
Source: the request's "Why a decision is needed" facts, read at main
`8f970866`.

**Migration and rollback.** There is no data or format migration.
`discover()` keeps its `00:05.0` behaviour, so existing callers need no
change. Rollback: revert the implementing commits and remove the
second-device stages from `PLAN` in `tools/test_volume.py`. Profiles without
the test feature keep refusing `device != 0`.

**Consequences.** The released write set is listed in the request. This
ADR decided nothing about D1–D5; those are decided separately by
[ADR-S09-DESTINATION-20261119](#adr-s09-destination-20261119--backup-destination-service-trusted-dialog-and-os-backup-portal)
(D1 (b), D2 (a), D3, D4 (a), D5 (a)), which keeps this grant a fixture. S09
stays open.

## Project-owner decision package of 19 November 2026 — scope and limits

The six ADRs below record one decision package issued by the project owner on
19 November 2026. They authorize scoped technical planning, implementation,
isolated testing and reviewable draft PRs within existing project permissions.
They do not override the Security Floor, current workstream ownership, branch
protection, merge authorization, production-signing boundaries, physical-device
restrictions or the unattended-execution hold. Each ADR separates the approved
architecture from unfinished implementation, acceptance and release authority.
An approved direction is not an accepted criterion and changes no score.

## ADR-S09-DESTINATION-20261119 — Backup destination service, trusted dialog and OS backup portal

Status: accepted by the project owner on 19 November 2026. It decides D1–D5 of
[S09-BACKUP-DESTINATION-PROPOSAL.md](implementation/S09-BACKUP-DESTINATION-PROPOSAL.md)
and confirms D6, superseding that proposal's "D1–D5 remain undecided" status and
the corresponding limitation in [ADR-S09-USTAR-20261114](#adr-s09-ustar-20261114--posix-ustar-with-pax-path-records-as-the-lossy-s09-user-export-format-d6-a).

**Decision:**

- **D1 (b):** a dedicated backup destination service owns the target device or
  partition and exposes named, append-only archive entries.
- **D2 (a):** a trusted system dialog authorizes the explicitly selected
  destination and operation. Remembered permissions are revocable and follow the
  existing permission model.
- **D3:** rights are least-privilege and operation-specific for export,
  verification and restore. The backup portal never acquires unrestricted volume
  access.
- **D4 (a):** a trusted OS backup portal holds destination capabilities.
  Third-party applications receive no direct raw backup-device authority.
- **D5 (a):** initially only local, user-owned destinations. Encrypted NAS and
  external-provider destinations require the existing key-management and
  client-side encryption contracts
  ([ENCRYPTED-OBJECT-STORAGE.md](implementation/ENCRYPTED-OBJECT-STORAGE.md)).
- **D6:** the approved POSIX ustar/pax interchange format is retained.

**Implementation boundary:** the first independent recovery test runs on
isolated, disposable QEMU disks. A separate destination service and any new
production capability boundary require a documented interface, a security
review, negative tests and verified enforcement before activation. The
existing test-fixture grant in [S09-BACKUP-ARCHIVE.md](implementation/S09-BACKUP-ARCHIVE.md)
remains a fixture. S09 stays open until its full acceptance passes; Claude S09
ownership of the implementation is unchanged.

## ADR-B03-B04-TARGET-20261119 — Target architecture for update authenticity and recovery

Status: accepted by the project owner on 19 November 2026 as the target security
architecture. It adopts D1–D7 of
[B03-PRODUCTION-POLICY-PROPOSAL.md](implementation/B03-PRODUCTION-POLICY-PROPOSAL.md),
superseding its "unapproved" status for the architecture only, and extends
[ADR-B03K3](#adr-b03k3--authorised-synthetic-policy-transition-experiment).

**Decision:**

- separate offline policy-root and release-signing roles (D1);
- bounded, canonical, authenticated policy envelopes and target-bound image
  manifests (D2,
  [B03-POLICY-TRANSITION-PROPOSAL.md](implementation/B03-POLICY-TRANSITION-PROPOSAL.md));
- demonstrable persistent rollback resistance against the declared threat
  model (D3);
- revocation and recovery that never bypass signatures or lower security floors
  (D4);
- the initial loader and root authenticated through a separately verified boot
  chain (D5);
- authenticated release-version windows; ordinary RTC time does not establish
  trusted expiry (D6);
- authorization covers specifications, synthetic-key implementation and
  disposable-QEMU verification only (D7).

**Still subject to separate verification and approval:** the production
custodians, signing environment, persistent anchor backend, firmware trust
chain and recovery procedures.

**Not authorized:** real production-key generation, production signing, release
publication, physical firmware modification or activation of an unverified trust
mechanism. Issue #41 and B03/B04 remain open; scores are unchanged.

## ADR-DESIGN-TOKENS-20261119 — Deterministic generator for design tokens

Status: accepted by the project owner on 19 November 2026 for the token work in
draft PR #360. It refines the generated-token rule in `ui/AGENTS.md` and
[DESIGN-SYSTEM.md](design/DESIGN-SYSTEM.md) without changing it.

**Decision:** approve a deterministic, version-controlled token generator with
validated inputs, reproducibility checks and regression tests.
`ui/prototype/tokens.css` and `ui/design/system/tokens.css` stay frozen until the
generator reproduces the intended output and the change passes independent
review. Unresolved review threads and required checks are not bypassed.
Accessibility requirements and the existing design identity are preserved. PR
#360 remains owned by the Claude UI/UX workstream.

## ADR-P-NAMES-1-20261119 — Functional, localizable user-facing names

Status: accepted by the project owner on 19 November 2026. It decides proposal
P-NAMES-1 in [NAMING-AND-IA.md](design/NAMING-AND-IA.md) §8 and supersedes the
user-facing names of 5 November 2026 in
[DESKTOP-SHELL.md](implementation/DESKTOP-SHELL.md) §7.

| Previous name | User-facing name (EN / NL) |
|---|---|
| Horizon | Desktop / Bureaublad |
| Zenit | Menu |
| Signaal | Search / Zoeken |
| Nebula | Files / Bestanden |
| Kern | Settings / Instellingen |
| Orbit | Orbit, with a descriptive dock label |
| Haven | Haven, with a descriptive application-store label |

Materiaal remains the internal object and permission model. Existing code
identifiers, stable APIs, storage formats and historical references stay
unchanged unless a change is separately justified. User-visible strings follow
these names through localization, not through renaming internal identifiers.

## ADR-PROGRESS-ROUTE-20261119 — Route from 41 to at least 50 points (issue #43)

Status: accepted by the project owner on 19 November 2026. It sets priority only
and does not change the rubric, weights or acceptance conditions.

**Decision:** the fixed 100-point rubric and the accepted baseline of 41/100 are
retained. Completing existing functional criteria takes priority over new
prototypes or wider claims. The preferred route is S09 (+2), N02 (+3), U05 (+2)
and H02 (+3), which reaches 51/100 only if all four pass their full acceptance
independently. H02 physical-device acceptance depends on a separately selected
and authorized reference machine; if that blocks progress, U03 is assessed as an
alternative three-point criterion under its unchanged conditions. Claude's
S09/UI ownership, Codex's roadmap ownership and the Integration Manager's
integration authority remain separate.

## ADR-W001473-PROVENANCE-20261119 — Disposition route for historical source record W001473 (issue #283)

Status: accepted by the project owner on 19 November 2026.

**Decision:** first establish the original author, source revision and valid
reuse permission for W001473 (referenced from
`.agents/skills/os-dev/references/`). If sufficient primary evidence cannot be
obtained, prepare a narrowly scoped exclusion or an independently authored
replacement, supported by a source comparison, a licensing review and a
publication-impact assessment. Permission is never inferred from missing
evidence. Git history is not rewritten, unrelated reference sources are not
deleted and no legal clearance is claimed without proper verification. Issue
#283 remains open until the selected disposition is substantiated and reviewed.

**Follow-up decision (project owner, 19 November 2026), after the provenance
search:** the primary search found W001473 to be a 2006 port from the old OS
Wiki with no verifiable original grant
(`docs/verification/w001473-provenance-20261119/`). The owner therefore chose,
and extended to the whole OSDev corpus, a risk-based review:

1. Exclude W001473 through a separate, targeted and reviewed change that
   rebuilds the indexes and keeps the provenance record (PR #397).
2. Examine the other 27 explicitly imported pages individually; keep a page
   only with demonstrably sufficient rights, otherwise exclude it in a targeted
   change (PR #399).
3. Examine the 641 pages that predate the CC0 statement of 6 June 2011, without
   blind deletion, and screen the other pages for older copied text (PR #398).
4. Retain text only on a verifiable licence, original permission or an
   independently written replacement; claim no new MPL-2.0 licence for copied
   text. Third-party text without such a basis is excluded (PR #400).
5. Report which texts are reusable, uncertain or excluded, and the residual
   Git-history risk.

*Consequences:* protected skill files change only through separate, scoped and
reviewed PRs; there is no mass deletion, no history rewrite and no licence
claim without evidence. Records with an owner decision still pending (licences
with notice or share-alike obligations, quotations) stay in the corpus as
uncertain until decided. Legal clearance and acceptance of these ADRs remain
separate steps. The product score is unchanged.
*Review criterion:* #283 closes only when W001473, the 27 imports and the
third-party records are excluded or retained on a documented verifiable basis,
the pre-CC0 review and older-text screening are recorded, and every remaining
uncertain record (including the pre-CC0 pages) has a recorded disposition.

## ADR-AGENT-GOVERNANCE-20261118 — Shared contracts and risk-based execution

Status: accepted project-owner direction, 18 November 2026, fresh-session execution
request. This supersedes earlier prerequisites requiring completed Phase 0/baseline B
or zero open issues, PRs and findings before governance. Historical evidence remains
unchanged. It does not declare baseline B complete or resolve security findings.

**Decision:** root `AGENTS.md` is canonical policy; directory instructions refine
subsystems. `.agents/skills/development-loop/SKILL.md` holds the common procedure;
`.agents/roles/` holds the four common role contracts. Claude Code, Codex and Copilot
adapters reference those contracts and retain surface-specific tool restrictions.
Use `os_verifier` or `os_security_reviewer` where appropriate; no optional
`agents/independent.md` is required. No active reference to that missing file was
found during reconciliation, so no replacement role is invented.

Default to one primary agent, with at most one additional reviewer when risk warrants
it. Claim bounded write paths in existing task state and isolate writers by worktree.
`docs/STATUS.md` is the compact current overview; `docs/PROJECT-STATE.md` keeps the
single lead continuation checkpoint while independent workstreams retain their own
handoffs. Do not create a second backlog. New sessions read current state, then only
missing contracts, relevant decisions and changed source.

Verification follows changed functionality and risk. Preserve meaningful negative
security tests, mandatory hosted checks and merge protections. Reuse valid
source-bound evidence with its applicability stated. Do not repeatedly run full native
matrices for unchanged code, assemble large trivial-fix evidence packages, poll CI
continuously or repeat optional reviews after quota failures. Manual squash is
explicitly authorized for this assignment only when mandatory checks pass; auto-merge
and protection bypass remain prohibited.

**Rationale and alternatives:** a shared contract prevents duplicated client policy
from drifting while allowing tool capabilities to differ. Separate client policies
and a new orchestration framework add maintenance without requested functionality.
Exhaustive repeated verification consumed excessive resources; targeted regression
checks retain the security invariant without conflating development feedback with
full product acceptance. Existing role definitions and checkpoints are reused.

**Consequences and limits:** governance acceptance is distinct from client runtime
parity, Daybreak acceptance, physical hardware and human audit. The #359 unconditional
execution hold and #355 disabled manifest remain in force. Offline gate/model-client
work and unfinished local WIP remain separately owned; this ADR grants no activation.
The new canonical policy paths are protected from unattended pipeline edits. Validate
adapter targets, retained tool restrictions and protected-path denial through host
regressions. No OS criterion or acceptance weight changes.

## ADR-VERSION-POLICY-20261117 — Latest stable components with verified exceptions

Status: accepted by the project owner on 17 November 2026. Authority and execution
scope: [owner amendment](verification/phase0-20261117/owner-latest-stable-policy.json).
This applies to Claude Code, Codex, GitHub Copilot and other project executors.

**Decision:** existing and new project dependencies, compilers, runtimes, libraries,
frameworks, SDKs, GitHub Actions, firmware, build tools, plugins, MCP tooling and
vendored components target the latest officially released, stable, actively
maintained and security-supported version by default. Verify the current release
and support/advisory status against authoritative upstream sources at execution
time. Pin accepted exact versions and immutable commits/checksums for reproducible
inputs; floating `latest` references are not accepted.

The newest release is not automatically the safest compatible selection. A
technical exception requires verified constraints, exact affected versions,
evidence, risks, an owner and reassessment conditions. Beta, nightly, release
candidates and known-vulnerable versions require an explicitly justified exception;
a consequential security-policy or trust-boundary exception still requires the
project owner's authority. Unknown support or inaccessible version data is
`NOT VERIFIED`, not an approved exception. The Security Floor remains mandatory.

**Rationale and alternatives:** ad hoc old pins provide no current support claim;
unconditional upgrades provide no compatibility or safety claim. The selected
policy combines current upstream selection with source-bound regression evidence
and explicit, reviewable exceptions. It preserves accepted architecture,
functionality, licensing and release criteria rather than selecting versions by
age alone.

**Procedure and consequences:** use the existing [dependency maintenance and
exception records](DEPENDENCIES.md#latest-stable-selection-and-exceptions).
Material updates assess provenance, licensing, advisories, MSRV, API/ABI and compiler
behavior, and run affected host/native/security regressions with independent review
before a gated manual squash. Preserve a tested rollback. Do not rewrite historical
source/test versions as current evidence or disturb another workstream's environment.

One targeted existing-stack inventory starts in Phase 0. Resolve urgent confirmed
security, unsupported-component and compatibility blockers before baseline B;
process larger noncritical migrations in bounded owned tasks without delaying
baseline/governance solely by count. Extend existing security/Dependabot automation
only where justified, without duplicate systems or unnecessary CI work. The
approved governance migration adds a concise inherited root `AGENTS.md` rule;
this record grants no completed upgrade, accepted exception or runtime parity.

## ADR-DESIGN-OWNER-20261115 — Owner decisions of 15 November 2026 on accessibility tiers, permission prompts and flashing media

**Date:** decided by the project owner on 15 November 2026; recorded in this
register on 17 November 2026. **Scope:** the UI/UX design pipeline (Orbitaal
design system, accessibility research and matrix, permission prompts) on
branch `claude/baryon-os-design-pipeline-20261115` (PR #339). **Source:** chat
messages of the project owner in Dutch. The verbatim texts are kept in the
design pipeline checkpoint (`build/design-pipeline-checkpoint/`, local and not
versioned); the statements below are an English rendering. The decisions are
design requirements; the implementation and verification status of each one is
stated separately.

### D1 — Accessibility in two tiers

**Decision.** Accessibility is a fundamental part of the BaryonOS design
identity, not a separate accessibility theme. Two tiers apply:

1. *Universal baseline:* every regular theme meets the essential accessibility
   and safety requirements (legibility, usable contrast, keyboard access,
   avoidance of hazardous visual effects) without configuration. The baseline
   is not an optional setting.
2. *Additional settings:* users adapt the regular themes to their needs (for
   example colour perception, light intensity, contrast, motion, typography,
   reading support, stimulus load and input methods), independently or in
   combination, as variants of the same design identity rather than rigid
   profiles per diagnosis.

The regular themes remain the primary experience and are not designed only
from the most restrictive needs. Device and user preferences such as reduced
motion are respected where possible. Essential safety never depends on a
manual setting. A systematic, targeted literature review precedes further
design elaboration, and unproven medical claims (for example a dyslexia font as
a proven remedy) are avoided. The owner's clarification of 00:15 CET takes
precedence over the first message of 00:10 CET where they differ.

**Supersedes:** nothing earlier in this register.

**Consequences.** A checkable accessibility matrix links needs, evidence
quality, adaptations, configuration, implementation status and verification
method. The design system expresses settings as combinable axes over the
regular themes. Effectiveness is not claimed on the basis of AI review alone;
the parts that need user tests are named. Existing capability, security, font
and architecture contracts are preserved, and required extensions are recorded
separately.

**Review criteria.** Every regular theme passes the baseline checks with no
configuration. Settings combine without unintentionally disabling one another.
No safety property depends on a setting. The decision is revisited where the
systematic review, user tests or a human audit contradict a baseline or setting
assignment.

**Status (separate from the decision).** UX-A11Y-R is an intermediate research
checkpoint (`8cca1167`, AI-verified only); the owner's systematic-review
requirement remains open. UX-A11Y-DS run 1 ended `blocked_after_2_repairs`; its
design-system changes are uncommitted and unaccepted, and run 2 is queued. No
native implementation, user test or human audit exists.

### D2 — Simplified permission prompts

**Decision.** A permission prompt offers two primary choices, Allow and Deny
("Toestaan", "Weigeren"), with an optional remember control ("Keuze onthouden"
or a better-fitting wording within the BaryonOS identity). Without remembering, a grant for an ongoing app
permission lasts until that app closes, and a grant for a single action covers
only that action, so no separate "allow once" button exists. A denial can also
be remembered. The remember control is off by default. A remembered choice
applies only to the clearly described app, right, data, destination and scope;
a new right requires a new decision. Valid grants cause no repeated prompts, and
unnecessary repeated prompts after a temporary denial are avoided. Remembered
choices are visible, changeable and revocable in the privacy settings.
Irreversible or risky actions may still require additional safety
confirmation.

Coordinator readings R2–R5, approved by the project owner on 15 November 2026,
refine the decision. A denial without remembering is held only in volatile
session state for the identical request until the app closes and is never
written to lasting storage; a new prompt for that request follows only a new
deliberate user gesture. The remember control starts unchecked at every prompt.
The prompt-conformance part of UX-D moves to UX-C. An ADR covers persistent
grants and remembered denials, the five-field key (app, right, data,
destination, scope), the until-app-close lifetime of temporary grants and
denials, storage protection, change and revocation, and the relation to
capability enforcement and the Security Floor.

**Supersedes:** the prompt-choice rule of the owner decision of 14 November
2026 ("Toestaan tot afsluiten", "Altijd toestaan", "Weigeren", and "Eenmalig
toestaan" for single actions: allow until closing, always allow, deny, allow
once) where they differ. Evidence for the earlier model
(`docs/verification/ds-1.0-c1-20261115/`, see its `SUPERSEDED.md`) is not
evidence for this model.

**Consequences.** UX-C researches, judges, builds and verifies the prompt.
Native security contracts are unchanged. No native code change precedes an
accepted ADR, and functionality that is not yet enforced is presented only as a
prototype.

**Review criteria.** Users understand what a choice and remembering mean (UX-C
research and user tests). Remembered choices are listed, changeable and
revocable. A remembered choice never widens beyond its five-field key. Temporary
denials never reach lasting storage. The decision is revisited where UX-C
research shows the two-choice model to be misunderstood or to weaken informed
consent.

**Status (separate from the decision).** `docs/design/DESIGN-SYSTEM.md` §11.1
records the supersession. UX-C has not started, and the ADR on persistent
authorisation is not yet written. Nothing is enforced natively.

### D3 — Flashing media, photosensitivity and games

**Decision.** The decision of 15 November 2026 on third-party flashing media
and its clarification on photosensitivity and video games apply, with the
clarification taking precedence. The owner's correction of the coordinator
readings F1, F2 and F5 applies; F3 and F4 are confirmed.

- *Per-item bypass:* the option to bypass the protection against hazardous
  flashes per item stays disabled. Known hazardous flashes are not shown after
  a simple confirmation.
- *Three contexts.* (1) The system interface meets the actual flash criteria
  (WCAG 2.2 SC 2.3.1; the ITU-R BT.1702-3 flash and pattern limits); this
  baseline is never switchable. The five-second remark of ITU-R BT.1702-3 is an
  additional risk consideration, not a universal hard limit, and safe animation
  is not restricted needlessly. (2) Unsolicited content (autoplay, previews,
  advertisements, notifications, backgrounds) is protected where technically
  possible; unassessed moving media is not presumed safe, and a static or
  otherwise safe rendering is used where needed. (3) Deliberately started games
  and media play normally and are not blocked categorically; optional
  photosensitivity settings, per-app preferences and appropriate warnings are
  researched.
- *Starting versus bypassing:* starting media deliberately remains allowed and
  is distinct from bypassing a safety block. One click never switches off all
  protective measures.
- *Separate preferences:* reduced motion and photosensitivity settings are
  separate personal preferences; neither weakens the system-interface baseline.
- *Honest claims:* a warning is not a safety guarantee, and no claim is made
  that BaryonOS detects or prevents all hazardous flashes without a demonstrably
  working technique.
- *Privacy:* apps need not receive medical or raw personal settings where
  BaryonOS can enforce the presentation itself or pass it through a limited
  interface.

**Supersedes:** the coordinator's original readings F1, F2 and F5, in
particular any treatment of the five-second remark as a hard limit.

**Consequences.** UX-A11Y-A1 amends the research document, the matrix and the
design criteria accordingly. The detection and attenuation mechanism for
unsolicited content is a research result and is not decided here. The research
covers the Xbox Accessibility Guidelines, WCAG and the relevant scientific
literature.

**Review criteria.** The combined rendered system interface stays within the
WCAG 2.2 SC 2.3.1 and ITU-R BT.1702-3 flash and pattern limits. Flashes on
separate system-drawn surfaces that occur concurrently are assessed by their
combined area and flash rate, not only surface by surface. Blocked unsolicited content has no
"show anyway" control. No document or interface text claims complete automatic
detection or presents a warning as a guarantee. The decision is revisited when
validated detection techniques or new evidence change what can be guaranteed.

**Status (separate from the decision).** UX-A11Y-A1 has not started. UX-A11Y-DS
run 1 documents the five-second remark as a hard limit, and A1 corrects that
wording. No automatic flash analysis is implemented.

## ADR-B03K3 — Authorised synthetic policy-transition experiment

**Date:** 14 November 2026. **Scope:** issue #41; TEST keys and disposable
QEMU state only. The project owner authorised publication of the reviewed
transition work and implementation of the explicitly bounded experiment after
the proposal and its evidence limits were presented. PR #328 had already been
squash-merged; missing work therefore continues on `feat/issue41-policy-test`.

**Approved experimental design:** fixed, distinct root/release roles; the
canonical policy and image-binding envelopes in
[B03-POLICY-TRANSITION-PROPOSAL.md](implementation/B03-POLICY-TRANSITION-PROPOSAL.md);
complete operation identity; monotonically enforced policy/image floors;
permanent revoked-pair tombstones; eligible, policy-bound fallback; and
fail-closed handling of uncertain durable state. The synthetic experiment may
use deliberately public TEST seeds. Both signature algorithms remain required.

**Execution order:** first implement and negatively test the bounded `no_std`
codec with the existing reviewed crypto dependencies, behind an explicit
`experimental-policy-test` feature. Then execute the same verifier on the
project's own native path with TEST fixtures. Freeze the exact native ingress,
anchor encoding, write/flush/readback semantics and fault oracle before wiring
a persistent policy transition. Retain all 18 existing update acceptance stages
and the missing-production-policy compiler guards. No new ordinary-app rights,
implicit kernel policy parser or production-profile activation follows.

**D1–D7 disposition:** the experiment's roles, format, transition rules, fixed
root, release-version semantics and TEST-only activation scope are approved.
Production custody/ceremony, a concrete rollback-resistant backend, production
ingress, B04 enrolment/boot/root recovery, trusted freshness/calendar expiry
and independent release-chain review remain unresolved. An emulator variable
or host-controlled anchor can expose behaviour and counterexamples; it cannot
prove physical persistence or whole-state rollback resistance.

**Evidence:** the preceding 92-check symbolic model is a finite specification
exercise, not native crypto/storage evidence. Each implementation checkpoint
records its actual source and executed checks separately. No acceptance point
is awarded by this approval. Real key generation, production signing, release
publication and physical-device changes remain outside this authorisation.

**Revision criteria:** reassess after native interruption/replay results, a
concrete backend or ingress contract, a proposed production trust change, or
independent review. Never reinterpret experimental success as production
acceptance. Reverting the experimental feature and its fixtures leaves the
existing TEST update path and missing-policy guards intact.

## ADR-S09-USTAR-20261114 — POSIX ustar with pax `path` records as the lossy S09 user-export format (D6 (a))

Status: accepted by the project owner on 14 November 2026; recorded on
15 November 2026. It decides D6 of
[S09-BACKUP-DESTINATION-PROPOSAL.md](implementation/S09-BACKUP-DESTINATION-PROPOSAL.md)
only. D1–D5 were decided later by
[ADR-S09-DESTINATION-20261119](#adr-s09-destination-20261119--backup-destination-service-trusted-dialog-and-os-backup-portal).
Implementation status (15 November 2026): the name check and the test-only
native stage `archive-ustar` are on draft PR #340; local gate run 14 passed
([USTAR-NATIVE-20261115.md](verification/s09-archive/USTAR-NATIVE-20261115.md)).
The decision is unchanged and S09 remains open.

**Context:**
- S09 requires that a user can create, verify, restore and export independent
  recovery copies in documented formats. The destination proposal put six
  decisions to the project owner; D6 asks which common format a user export
  uses.
- `ustar::from_archive` in `services/volume/src/ustar.rs` converts a verified
  format-v1 archive into a POSIX.1-1988 ustar stream with POSIX.1-2001 pax
  `path` records. It is host-tested only
  ([S09-BACKUP-ARCHIVE.md](implementation/S09-BACKUP-ARCHIVE.md), section
  "User export to ustar"). It refuses colliding paths, a file/collection clash
  and the drive form, but still exports names that extract differently on
  Windows or macOS.

**Approval by the project owner (14 November 2026, verbatim):**

> I approve D6(a): POSIX ustar with pax path records as the initial interoperable S09 user-export format.
> Conditions:
> * This is a lossy interchange format, not the native BaryonOS/Materiaal format and not a recovery backup.
> * Preserve the existing verified archive format for recovery.
> * Reject unsafe or ambiguous cross-platform paths, including Windows alternate-data-stream syntax, reserved device names, invalid characters, trailing dots/spaces, and case-folding or Unicode-normalization collisions.
> * Never silently rename files or permit unintended overwrites.
> * Report rejected names clearly. A future explicit, reversible name-mapping mechanism may be proposed separately.
> * Document metadata loss, empty-collection limitations, and compatibility constraints.
> * Keep D1–D5 undecided; this approval does not authorize new destination capabilities, trust boundaries, or encryption architecture.
> After recording D6, autonomously implement the bounded native ustar-export test stage using the existing test-only permissions and storage fixtures.
> Update the relevant test orchestration and workflow expectations to reflect the actual stage count, coordinating with the maintenance session to avoid conflicting changes.
> Run targeted checks and the native volume gate, preserve the evidence, and update draft PR #325.
> Do not merge automatically. Continue autonomously within the approved S09 scope.

The project owner restated the same terms in Dutch on 15 November 2026.
PR #325 was merged before this record, so "update draft PR #325" can no longer
be done; the follow-up goes to one new draft PR, with no merge and no
auto-merge without the project owner's explicit approval
(ADR-MERGE-HOLD-20261113 and AGENTS.md).

**Decision:**
- D6 is decided as option (a): POSIX ustar with pax `path` records is the
  initial interoperable S09 user-export format.
- The export is a lossy interchange format.
- The export is not the native Baryon OS/Materiaal format.
- The export is not a recovery backup; nothing restores from it.
- The verified archive format v1 (checked by `archive::verify`) stays the only
  recovery format and is not changed by this decision.
- The export must refuse unsafe or ambiguous cross-platform names: Windows
  alternate-data-stream syntax, reserved device names, invalid characters,
  trailing dots or spaces, and names that collide under case folding or
  Unicode normalisation. It reports every refused name clearly, never renames
  a file silently and never permits an unintended overwrite.
- Metadata loss, the empty-collection limit and the compatibility constraints
  are documented in S09-BACKUP-ARCHIVE.md, section "User export to ustar".

**Not decided and not authorised:**
- At the time of this decision, D1–D5 remained undecided; ADR-S09-DESTINATION-20261119 later decided them. This decision does not choose a destination, a
  grant path, per-operation rights, the component that holds the authority or
  the encrypted NAS/provider variant.
- No new destination capability, trust boundary or encryption architecture is
  authorised.
- A reversible name-mapping mechanism is not part of this decision; it needs a
  separate proposal.
- The authorised native ustar-export stage is a test stage only. It uses the
  existing test-only permissions and storage fixtures and adds no production
  authority.

**Consequences and limits:**
- Approval is not implementation. On the recorded source, the converter does
  not yet refuse the names listed above; the name check, its tests and the
  native test stage are later, separate tasks. Until they pass, the export
  does not meet this decision and S09 stays open.
- The character set that the name check allows is not yet defined, and the
  reserved-name list includes conservative additions beyond the primary
  Microsoft list. Both are listed as implementation limits in
  S09-BACKUP-ARCHIVE.md, section "User export to ustar".
- The only readers tested are Python's `tarfile` module and GNU tar 1.35.
  Extraction on Windows or macOS has not been tested.
- Plaintext archives and exports are written only on disposable test images.

**Revise when:** a reversible name mapping is proposed, a D1–D5 decision
changes where an export may be written, or a supported reader is shown to
mishandle pax `path` records.

## ADR-MERGE-HOLD-20261113 — SHA-bound required merge hold for out-of-band acceptance

Status: accepted, 13 November 2026, on the project owner's instruction after
#296 merged before its K06 acceptance matrix had finished.

**Context:**
- #296 was squash-merged at 2026-11-12 00:15:40 UTC by auto-merge, which had
  been enabled on 10 November. At that time only the required checks
  `host-quality` and `qemu-smoke` had passed on head `95bdf4cb`.
- The SMP matrix that was dispatched on that head later reported failures:
  `SMP 50/50 (eight)`, `SMP 50/50 (resched-deaf)` and the 65x50 aggregate.
- Requiring the dispatch-only `SMP final repository gate` directly would not
  help. On pull request events that job is skipped, and GitHub documents that
  a job skipped by a conditional reports success.
- The same day the project owner decided that sessions may create and update
  draft PRs autonomously, but never merge, or enable auto-merge, without
  explicit approval.

**Decision:**
- `.github/workflows/merge-hold.yml` runs `tools/merge_hold.py` on every pull
  request opened, synchronize, reopened, labeled and unlabeled event. It has
  no path filter and no `if`, and runs with read-only permissions.
- The script reads labels and comments live through REST and fails closed on
  any API or input error.
- With the `acceptance-pending` label present, the check passes only when a
  participant whose author association is OWNER, MEMBER or COLLABORATOR has
  commented the exact line `acceptance-released: <head SHA>`. A new push
  changes the SHA and re-arms the hold.
- `merge-hold` is added to the `main` ruleset's required checks, pinned to the
  GitHub Actions app (integration 15368). This happens only after the workflow
  is on `main`, its behaviour has been verified on a pull request, and the
  open pull requests have been notified.

**Consequences and limits:**
- The hold is opt-in. Owners must set the label; the check does not detect
  out-of-band acceptance by itself.
- Between setting the label and the failing run being recorded, an earlier
  success on the same head still counts. The label is therefore set when the
  pull request is opened or before any merge request.
- Like every `pull_request` gate, a pull request can change its own workflow
  and script. The gate protects against omission, not against a malicious
  writer; review of `.github/` and `tools/` changes remains necessary.
- A label change made with a workflow's `GITHUB_TOKEN` starts no new run.
- Once the check is required, disabling or renaming the workflow or job
  blocks every pull request to `main`. Roll back by first removing the
  required check from the ruleset, then reverting the workflow.
- Unit tests cover the decision, pagination and fail-closed paths. A
  contract test pins the trigger, the permissions and the absence of `if`,
  `defaults` and filters.

**Amendment, 23 November 2026 (merge queue preparation, not yet enabled):**
`merge-hold` and the required `ci.yml` checks also run on `merge_group`. The
merge-group run evaluates the group's pull request and every unmerged entry ahead,
each on its own head SHA; a released hold also requires that entry's queue
commit to land the released tree. It decides last: it waits until the group's
`host-quality` and `qemu-smoke` have completed, then walks the entries ahead
from the nearest outward: each must have left the queue or completed
`host-quality`, `qemu-smoke` and `merge-hold` on its own commit. The walk ends
at the first entry whose `merge-hold` concluded `success`, because only a
successful run waited the same way; a `merge-hold` that failed early, was
cancelled or lost its runner proves nothing about the entries ahead of it.
This shortcut does not recheck an earlier predecessor whose check is rerun
after that successful entry. Before enabling a live queue, either invalidate
and rebuild all affected successor groups when a predecessor check is rerun,
or implement and verify a full predecessor reevaluation. Until one of those
controls is in place, the shortcut is preparation only and the queue must
remain disabled.
Queue entries behind the group are not evaluated, and one without a commit
does not delay the run; an entry anywhere in the queue without a pull request
(or without an integer pull request number) disables the fast stale-group
detection below, so the run is retried until the deadline. Ahead of the group,
an entry without a commit keeps the run waiting, and fails closed at the
45-minute deadline, only while the wait walk reaches it; beyond the nearest
entry ahead whose `merge-hold` concluded `success` the walk stops, and the
final evaluation fails closed at once on that entry unless its pull request
has already merged. An entry ahead with a commit but no pull request fails
closed at once when the group is evaluated. The wait walk does not skip an
entry ahead whose pull request has already merged; only the final evaluation
skips it, because it is on `main`. Fast stale-group detection applies only
after the group's own `host-quality` and `qemu-smoke` have completed: a group
whose own commit two consecutive well-formed, non-empty queue reads, one poll
interval apart, both no longer list (ejected or rebuilt) fails at once. Any
other poll in between restarts that count, and a failed, malformed or empty
read is retried until the deadline. This holds
only while `host-quality`, `qemu-smoke` and `merge-hold` are the only required
checks; a later-finishing required check reopens the window. The queue
ruleset must also have "Only merge non-failing pull requests" set to Yes, so
an entry whose required checks failed cannot merge as part of a later group
that passed; a minimum group size of 1 with no merge-limit wait, so the
window after the final read is not lengthened; each pull request built
separately (no batching), so every entry has its own commit and its own
`merge-hold` run; and a merge queue status check timeout of at least 100
minutes, so GitHub does not remove a group while `merge-hold` still waits (up
to 80 minutes, inside a job `timeout-minutes` of 90). Verify these five
settings and the predecessor-rerun control above before enabling the queue. On
`merge_group` the workflow and `tools/merge_hold.py` come from the group
commit, which includes the entries ahead, so a queued pull request ahead can
change the script that decides a pull request behind it; this is the same
trust boundary as a pull request changing its own script, and review of
`tools/merge_hold.py` and `.github/**` remains the control. The behaviour rests on GitHub's merge queue documentation
and unit tests with a mocked API; it has not yet run against a live queue.
- Enforced: a hold set, or a release comment removed, before the final read
  blocks every group containing that pull request, including a later group that
  merges the entries ahead with it.
- Not enforced: a hold set after the final read, while GitHub concludes the job
  and merges the group. With the predecessor-rerun prerequisite satisfied,
  the window does not depend on other entries' check durations, but it is
  not bounded by the project; a merge-limit wait time or
  a minimum group size above 1 in the queue ruleset would lengthen it, which
  is why both are preconditions above. Owners set the label before queueing and
  remove a pull request from the queue to apply a late hold.
- Automatic removal from the queue on a late label (GraphQL
  `dequeuePullRequest` from the `pull_request` run) was not adopted: its
  required token permission is not stated in the GraphQL pull request
  reference or the GITHUB_TOKEN permissions page, it would give a write token to
  a job that runs the pull request's own script, it cannot act for fork pull
  requests, and it races the merge on runner start-up latency. It needs a
  separate owner decision.

**Revise when:** a reliable, always-reporting aggregate of out-of-band
acceptance becomes available, or the merge queue is enabled.

## ADR-WINDOW-SCALE2-DEADLINE-20261112 — Per-scale harness deadline for the native window gate

Status: accepted, 12 November 2026, to repair `main` after #210.

**Context:** after #210 the 200% `native-window-session (2)` gate failed twice
on `main` `49f80b1e` with "G0 QMP exceeded the shared deadline before send".
The tree `17de3cba` was byte-identical to PR head `682b2eb`, where the same
profile passed. The QEMU bundle, runner image and command were also
identical. Every logged phase was uniformly about 1.45x slower on the failing
runners: boot 93 s against 65.5 s, `document-menu-high-contrast` 1797 s
against 1229 s. The 132-phase route needs about 1480 s on a fast standard
hosted runner and about 2150 s on a slower one, so the shared 1800 s deadline
cannot hold on all of them.

**Decision:** the runner bounds its harness deadline per scale.

- Scale 1 keeps 1800 s, the bound preregistered for the 1280×720 route in
  `docs/implementation/NATIVE-CLIPBOARD.md`.
- Scale 2 may use up to 2700 s, and the CI gate uses that value only for the
  scale-2 clipboard/workspace run.
- The switcher part gates and the library walkthroughs keep 1800 s.
- The runner enforces this per mode, not only the workflow: `run()` accepts
  up to 2700 s only when scale 2 is combined with both `--clipboard` and
  `--workspace`, and rejects any other mode above 1800 s, including a manual
  scale-2 `--switcher` run (follow-up of 13 November 2026).

**Consequences:** this is a harness deadline, not a latency budget. No phase,
pixel comparison or assertion is removed, and the per-phase timings stay in
the evidence. Runner tests pin both bounds, and mutations of either bound fail
them. A slow runner can now spend up to 15 more minutes on scale 2, still
within the 110-minute job limit.

**Revise when:** measured interaction budgets for the 200% route are
preregistered, or the route is split into shards that each fit the 1800 s bound.

## ADR-CI-SECURITY-ALERTS-20261111 — Public-safe summary of open security alerts

Status: accepted, 11 November 2026, on the project owner's instruction to make
Dependabot and code-scanning alerts usable for backlog work.

**Context:** agent sessions cannot read these alerts. On 11 November 2026 the
session's GitHub integration returned "Resource not accessible by integration"
for Dependabot and code-scanning alerts. The session proxy blocks
secret-scanning alerts and GraphQL. GitHub documents that a workflow token
reads code-scanning alerts with `security-events: read` and Dependabot alerts
with `vulnerability-alerts: read`. Actions logs and step summaries of this
public repository are public, while GitHub restricts the alert lists to
maintainers, and SECURITY.md forbids publishing non-public vulnerability
details.

**Decision:** `.github/workflows/security-alerts.yml` runs
`tools/security_alerts_summary.py` daily, on manual dispatch and on same-repository
pull requests that change its own files. The job token has only `contents`,
`security-events` and `vulnerability-alerts` read access, and no repository
secret is used.

- Dependabot: one row per open alert, limited to publicly derivable fields
  (ecosystem, package, manifest path, GHSA and CVE identifiers, severity,
  vulnerable range and first patched version). Anyone can derive these from the
  public lockfiles and the public GitHub Advisory Database.
- Code scanning: aggregate counts per tool and severity only. Rule identifiers,
  paths, lines and messages stay in the Security tab.
- Secret scanning: not automated. Push protection remains enabled, and alerts
  are reviewed by maintainers in the Security tab.
- Every printed value must match an allowlist pattern; API free text is never
  printed. Pagination and redirects cannot leave `https://api.github.com/`. An
  unreadable source fails the run and is reported as "not read", never as zero.
- actionlint 1.7.12, pinned in `ci.yml`, does not yet know the
  `vulnerability-alerts` scope. `.github/actionlint.yaml` ignores only that one
  diagnostic in that one workflow, and a contract test keeps the exception
  that narrow. The file is removed once a pinned actionlint knows the scope.

**Alternatives:** a personal access token would add a long-lived secret for
data the workflow token already reads. An uploaded artifact is as public as the
log. Leaving the alerts unread keeps dependency fixes outside the backlog.

**Evidence boundary:** 18 local unit and contract tests, the CI-workflow
contract tests and actionlint 1.7.12 pass. Hosted PR run 43118954606 showed
that GitHub accepts the `vulnerability-alerts` scope and returns both alert
lists to the workflow token. **Review criteria:** revisit when GitHub changes alert visibility
on public repositories, when a code-scanning detail is needed for a fix (it is
then shared through a private channel), or when the pinned actionlint changes.
## ADR-PROJECT-SYNC-20261111 — Fact-derived Status and backfill for Project #5

Status: accepted, 11 November 2026, on the project owner's instruction to
integrate the existing organisation Project #5 with Issues, Pull Requests and
Actions. It extends the conservative classifier from #303; no second project
and no new workflow file are introduced.

**Context:** the first dry-run (run 43115306190) showed that the Actions secret
`BARYON_PROJECTS_TOKEN` is not configured. `GITHUB_TOKEN` cannot write an
organisation project, and GraphQL is unavailable from agent sessions, so the
project itself could not be inspected. Without a fix the daily schedule would
fail on `main` every day.

**Decision:**

- Real-time transitions use the project's built-in workflows (auto-add, item
  closed, pull request merged, item reopened, review approved/changes
  requested). The scheduled tool complements them and does not replace them.
- Status is derived from GitHub facts only: a merged PR or an issue closed as
  completed becomes Done; an open draft PR In Progress; an open ready PR In
  Review (fallback In Progress); an explicit `blocked`/`status:blocked` label
  Blocked; an empty, Blocked or reopened-Done open issue Todo. A PR closed
  without merge, an issue closed as not planned and any status outside the
  managed set are left to humans.
- Open issues and PRs missing from the project are added, at most 50 per run,
  and only after a complete project scan.
- Work Type, Subsystem and Priority keep the fill-only rule of #303.
- Scheduled runs without the secret skip with a warning; manual runs fail.
- The 17 non-required workflows with `paths-ignore` (33 triggers) share one list that skips
  documentation, all Markdown, issue/discussion templates and this project
  tooling. A contract test fixes that list and proves that code, lockfile,
  build-tool and workflow paths still trigger them; `ci.yml` stays unfiltered.

**Consequences:** Project Status is organisational. It never overrides a
GitHub Actions result; Done follows a merge, never a check. Live behaviour is
unverified until the owner adds the secret: a fine-grained token or GitHub App
limited to organisation Projects read/write plus read access to issues,
pull requests and metadata of this repository. Unit tests use an in-memory
GraphQL fake.

**Revise when:** the project's field options differ from the assumed names,
or a GitHub App replaces the personal token.

## ADR-CI-QEMU-BUNDLE-20261111 — Pinned QEMU/OVMF from a signed package bundle

Status: accepted, 11 November 2026, on the project owner's instruction to stop
depending on Ubuntu DNS and mirrors on every CI run. This decision supplements
ADR-CI-PINS-20261108; package versions and checks stay the same.

**Problem:** on 11 November 2026, the "Pinned QEMU and OVMF" step failed on
hosted runners. snapshot.ubuntu.com timed out over HTTPS and runner DNS for
the Ubuntu mirrors failed. One external host blocked every QEMU workflow.

**Decision:** `.github/actions/setup-qemu` first downloads a bundle from the
GitHub release `ci-qemu-apt-20261111-r2` and checks its pinned sha256. Each bundle
is a partial Ubuntu mirror: the Ubuntu-signed `InRelease` files, the original
`Packages.xz` indexes and the dependency-closure `.deb` files. APT installs
from it as a `file:` source. It still verifies the Ubuntu signature
(`Signed-By`), the index hashes and every package hash. The exact pins, the
dpkg version check, the machine/TCG checks and the firmware hashes stay
fail-closed. If the bundle is missing or rejected, the action falls back with a
warning to the signed snapshot, then to the signed archive, with the same pins.
`tools/ci/build_qemu_apt_bundle.sh` rebuilds a bundle deterministically. The
Ubuntu 26.04 profile from #296 is included.

**Alternatives:**
- The live archive, over HTTP or HTTPS, is signed and safe for integrity. It
  is not reproducible: each pocket index lists only the newest version, so exact
  pins stop resolving after the next update. It also needs Ubuntu DNS on every
  run.
- Actions cache entries can be evicted and are scoped per branch.
- Workflow artifacts expire.
- Git or LFS would add about 134 MB of binaries.
A release asset is durable and independent of Ubuntu's availability. Its
integrity rests on the Ubuntu signature, not on the host.

**Licences:** the packages keep their Ubuntu licences. The `*-SOURCES.txt`
release assets list the matching source packages with their sha256. Hosting
the source tarballs next to the binaries is an open follow-up.

**Evidence boundary:** both bundles were built twice from the archive with
identical results and passed a download-only APT install from the bundle alone.
Hosted canary run 43057156214 installed every pin from the bundle on
ubuntu-24.04 (with swtpm) and ubuntu-26.04, with the Ubuntu mirrors blocked and
no fallback warning; see the
[bundle report](verification/ci-qemu-bundle-20261111.json). The first release
(`ci-qemu-apt-20261111`) stored `.deb` files under percent-encoded names and is
superseded; its assets are unchanged. The canary covers the two runner images
of that date. A later image that preinstalls other versions can still miss a
package; the action then warns and uses the signed network path. A pin update
requires a new bundle, a new tag and new sha256 values in the action. Score
39/100 is unchanged.

## ADR-081 — M3 ring-3 processes on every online CPU (K06 step 5d-2a)

> Ported to `main` on 13 November 2026 from PR #246 (head `273b2eb`); written on the K06 branches on
> 6 November 2026 and not previously on `main`. English translation; the Dutch original is archived in
> `docs/history/k06-branch-prs-2026-11-06/`. Evidence paths, scores and branch names below refer to
> those branch commits; evidence directories named `docs/verification/k06-*` are archived under
> `docs/history/k06-branch-prs-2026-11-06/verification/`. Later amendments: the `main` entries
> "K06 step 5f — remote readiness wake runs next (ADR-081 R3-5 addendum)" (11 November 2026),
> "K06 step 5f — interruptible spin window survives QEMU 8.2 PAUSE (ADR-081 addendum)"
> (11 November 2026), "K06 step 5f — IPC watchdog recalibrated for hosted runners (ADR-081 R3-5
> addendum 2)" (11 November 2026) and "K06 step 5f — review findings closed before acceptance
> (ADR-079/ADR-081 addendum)" (12 November 2026), and ADR-082 (D5, host watchdog of the `m3-load`
> outcome; 13 November 2026).

6 November 2026; effect only in the new profile `native-smp-m3` (and its negative variant
`test-smp-m3-fault`). Task specification: `docs/implementation/k06-steps/08-5d-2-ring3-on-aps.md`.
Builds on ADR-074, ADR-075, ADR-077 to ADR-080 and ADR-083. **Problem:** after the self-tests, all
APs parked; `native-smp` excluded M3 (`compile_error`). No AP performed production work.

**Decision (R3-1 to R3-4, sub-step a):**
1. **Profile.** `native-smp-m3 = native-smp + milestone-m3`; the `compile_error` is dropped only
   for that profile. In this sub-step, the profile carries only the base M3 manifest (16 processes);
   `fpu-context`, `ipc-wait`, M4 and the supervisor are still excluded from it with a `compile_error`.
   All other profiles remain single-CPU.
2. **Handover.** `smp::start` runs all self-tests unchanged and then leaves the APs in their
   home loop (`SMP:M3:WAITING:APS=n`) instead of parking them. `m3::start` builds the process table
   under `KERNEL`, after which `smp::m3_go` masks the BSP LAPIC timer, gives every online CPU an
   M3 service stack (BSP: MAIN; AP: its MAIN stack), sets gate 0xd0 to `m3_ap_timer` and
   releases `M3_GO`. An AP then starts its LAPIC tick (~10 ms) and enters `m3::ap_main`.
3. **Placement (R3-2, spec §6.2).** On spawn, `home` = the online CPU with the fewest live
   processes, the lowest index in a tie. `select` chooses only processes whose `home` = this
   CPU. No migration, no load balancing. A process therefore runs on at most one CPU
   (INV-3).
4. **Entry (R3-4).** Every M3 entry takes `KERNEL` (ADR-075). Within `KERNEL`, `State.current`
   always belongs to the holder: `load_current` puts this CPU's value into it, and `kernel_leave`
   puts it back into `CPU_CURRENT`. After its `swapgs`, the stub fetches the service stack through
   the GS anchor (`nos_smp_m3_stack`); TSS.RSP0 is set per CPU (`smp::rsp0`). From CPL3, the AP
   tick 0xd0 performs the full M3 entry. From CPL0 (idle, or a spin window of `KERNEL`) it only
   sends EOI and returns through the reviewed `iretq` of the spurious stub. On the BSP,
   PIT IRQ0 remains the M3 tick and the clock. An IRQ0 during a CPL0 wait is counted and
   incorporated into `ticks` under `KERNEL`.
5. **Address space (INV-4).** `Process.active` receives this CPU's bit before the CR3 load and
   loses it after the load of the kernel CR3 on entry. `cleanup` refuses a process that is still
   active on another CPU (`SMP:M3:REMOTE_ACTIVE`).
6. **Release (INV-1, spec §5 point 6).** On SMP, `cleanup` first clears all PTEs (user half
   and kernel aliases), then performs one kernel round to all online CPUs (`shootdown_kernel`,
   CR3 reload), and only then releases the frames.
7. **Idle and end (INV-7).** A CPU without a ready process releases `KERNEL` before `sti; hlt`.
   The end of the fixture is a stop-the-world: the stopping CPU releases `KERNEL`, every other
   CPU parks at its next entry with the kernel CR3 and IF=0 (without touching STATE) and
   leaves `ONLINE_CPUS`. The cleanup and the balance check follow.

**Shortcuts (recorded):** (a) a spawn after `M3_GO` is refused (an assert), because it requires
kernel-half rounds; that comes with K05 on SMP. (b) A kernel round without an ack stops the VM
(`SMP:M3:SHOOTDOWN_TIMEOUT`) instead of quarantine plus migration to the BSP (spec §6.2(a));
that follows in 09. (c) The M3 tick of an AP is a fixed LAPIC count (625 000, ~10 ms at the
ADR-080 ratio), not calibrated per CPU.

**Incidental finding:** `build_spaces` (5e) returned the two test spaces by value. That cost
~13 KiB of the 16 KiB MAIN stack. In the larger M3 build, this caused a #DF (stack overflow in the
self-test). Both spaces are now static.

**Falsified if:** a CPU finishes without user ticks or with fewer than 4 switches, a process runs on
two CPUs at the same time, a frame is released before every CPU has acknowledged the kernel round,
or an app fault on AP2 stops more than that one process.

**Addendum 5d-2b (R3-3, FPU in the switch):** `native-smp-m3` includes `fpu-context`
(`arch::protection` sets CR0/CR4/EFER before the AP bring-up; the APs copy them exactly, ADR-070).
The image of the running process is held per CPU in `CpuLocal.m3_fpu` (`fpu::select` writes
it). After the `swapgs`, the entry fetches it through `nos_smp_m3_fpu` and performs the FXSAVE before
any Rust code. The exit performs the FXRSTOR directly before the GPR pops through `nos_smp_m3_fxrstor`:
RSP is then the saved context in STATE and not a stack, so the helper is entered and left with `jmp`
(ADR-045). The diagnostic counters use `lock inc`. Evidence: every SMP process is an FPU canary
(probe mode 19: `st0`, `xmm0` and `xmm15` with their own values, checked on every loop iteration,
`int3` on a mismatch). The end condition requires 1000 switches per CPU. The kernel then checks
ST0/XMM0/XMM15 in every saved image (`SMP:M3:FPU:PASS`). The control
`test-smp-m3-fpu-skip` omits the FXRSTOR and must produce `SMP:M3:FPU:CORRUPT:CONTROL`. An
`int3` from CPL3 arrives through the DPL0 gate as #GP with error code 0x1a; both forms count
as a canary alarm.

**Open (5d-2c and beyond):** IPC across CPUs (R3-5) requires M4 (capability engine) plus
`ipc-wait` on SMP. Concretely: on Waiting→Ready for a process with a different `home`,
`wait::refresh` must send 0xe8; `smp_idle` must run `refresh` first; the copy between
address spaces must take `lock_pair(ASPACE[a], ASPACE[b])`; and a ping-pong program with
10 000 rounds is needed (no new syscall, INV-6). Further: usercopy under `ASPACE` (R3-6), per-process
quota and K05 on SMP with spawn after GO (R3-7), and ring-3 GS N1-N3 with M3 processes.

## ADR-080 — BSP timer, PIT clock, idle and reschedule flag on the BSP (K06 step 5c-2, audit findings F6/F9/F11/F13)

> Ported to `main` on 13 November 2026 from PR #246 (head `273b2eb`); written on the K06 branches on
> 6 November 2026 and not previously on `main`. English translation; the Dutch original is archived in
> `docs/history/k06-branch-prs-2026-11-06/`. Evidence paths, scores and branch names below refer to
> those branch commits; evidence directories named `docs/verification/k06-*` are archived under
> `docs/history/k06-branch-prs-2026-11-06/verification/`. Later amendments: the `main` entries
> "K06 step 5f — interruptible spin window survives QEMU 8.2 PAUSE (ADR-081 addendum)"
> (11 November 2026; the IF=1 spin window named in BT-6 becomes `sti; nop; pause; cli`) and
> "K06 step 5f — clock check split into idle precision and load progress (spec 06 §4 revision)"
> (11 November 2026; revises the clock clause of this decision's task specification), and ADR-082
> (D3, idle clock judging; 13 November 2026).

6 November 2026; effect only in `native-smp`. Task specification:
`docs/implementation/k06-steps/06-5c-2-bsp-timer-idle.md`. Supplements ADR-073 (preemption on the
APs) and ADR-074 (spin window S1/S3). **Problem:** the BSP had no preemption, no idle with
interrupts and no spin window. If the BSP spins on `KERNEL` while an AP starts a shootdown, it
does not accept 0xe0, and a false quarantine of all APs follows (F6). 0xe8 jumped to the
timer path and counted as a tick (F11). The LAPIC timer was uncalibrated (F9).

**Established (spec §0 "to be verified"):** `native-smp` excludes M3 (`compile_error` in
`main.rs`), so the PIC programming in `m3.rs` and Sys3 do not exist in this profile. Until now, the BSP
ran there entirely with IF=0 and did not program the 8259. This decision therefore introduces the
PIT IRQ0 clock in `native-smp` itself; the M3 path (`m3.rs`, Sys3) is unchanged. In QEMU, the
8259 delivers only to the LINT0 of the first CPU (virtual wire); the APs have masked LINT0 since step 2.

**Decision (normative):**
- **BT-1 One clock.** `bsp_timer_init` (BSP, IF=0, after the bring-up and before `GO`) remaps the 8259 to
  0x20/0x28, masks everything except IRQ0 and sets PIT channel 0 as a rate generator with divisor 1193
  (1000.15 Hz). The IRQ0 stub (vector 0x20, no GS, no lock) counts `CLOCK_TICKS` and sends a
  PIC EOI. It checks with the LAPIC ID that it runs on the BSP; an AP that receives IRQ0 stops
  the VM with `SMP:CLOCK:AP_IRQ0`. Spurious IRQ7 (0x27) is given the spurious stub. The LAPIC timer
  delivers only preemption ticks.
- **BT-2 BSP equal to an AP.** The BSP receives 0xd0 with divisor 16 (`start_timer`), the same
  INV-5c deferral and the same switch rules (`bsp_switch`; on the BSP, the home context is the
  orchestrator and therefore takes part in the rotation). `CpuLocal.irq_ready` is set per CPU as soon as
  every unmasked source has an IDT gate: on an AP after masking the LVTs in
  `ipi_phase`, on the BSP at the end of `bsp_timer_init` (LINT0 = ExtINT for the PIC, LINT1 =
  NMI to vector 2, all other LVTs masked; gates 0x20, 0x27, 0xd0, 0xe0, 0xe8, 0xf0,
  0xff). The window predicate in `lock.rs` becomes `level <= ASPACE && irq_depth == 0 && irq_ready`
  (was `on_ap()`).
- **BT-3 Idle.** `idle_halt` is the only `sti; hlt; cli`. It checks for an empty lockdep set and
  the kernel CR3 (`KERNEL_CR3`, now already set in `bsp_timer_init`) and otherwise stops with
  `SMP_IDLE_STATE`. Used by the AP home loop and by the BSP (`idle_until`).
- **BT-4 Reschedule flag.** The 0xe8 stub counts `resched_count` before it jumps into the timer body.
  `smp_preempt` compares that with `resched_seen`: a difference = 0xe8 (`need_resched = true`, no
  tick), otherwise 0xd0 (`tick_count += 1`). Fields renamed: `ticks` → `tick_count`, `resched` →
  `resched_count`. A 0xe8 deferred by INV-5c remains pending as `need_resched`. An
  `IrqGuard` that releases the last lock and restores IF=1 outside a handler calls
  `resched_on_release`: it converts a pending flag into a self-0xe8, which arrives directly after the `sti`
  and switches. Every other switch point clears the flag. The ICR fixture never switches and clears
  the flag itself.
- **BT-5 Calibrated band.** Over 100 ms of PIT channel 2, `bsp_timer_init` measures both the LAPIC
  timer (divisor 16) and the TSC and logs `SMP:TIMER:RATIO:<LAPIC-ticks per PIT-IRQ0-tick>`. **The band
  is fixed in advance at [50 000, 75 000]**, for QEMU's APIC timer clock of 1 GHz (expected
  62 500); the oracle (`tools/run_smp.py`, `RATIO_BAND`) rejects everything outside it, and the band is
  not widened afterwards. A different band (another hypervisor or real hardware) requires a new
  decision. `TSC_PER_MS` serves as the time base for bounded waits on APs, which do not touch the
  PIT (ADR-070).
- **BT-6 Kernel-thread contract (F13).** Fixture kernel threads (step 4, 5d-1, F16 and the BSP threads
  of this step) run with IF=1 as "pseudo-user": their return to IF=1 counts as the safe
  point of BT-4. Real kernel code remains IF=0 (spec §4.3); IF=1 only in user mode, in the idle
  context and in the bounded `sti; pause; cli` window of a KERNEL/ASPACE waiter.

**Evidence:** native fixtures in `native-smp` (after `SMP:SPIN:PASS`, before `SMP:SCHED:PASS`):
`SMP:CLOCK:PASS` (all APs loaded, the BSP idles through 500 IRQ0 ticks; duration on the PIT-calibrated
TSC within 1% of 499 923 µs; the clock never goes back), `SMP:BSP_PREEMPT:PASS` (two BSP threads
with register canaries, ≥8 ticks and ≥4 switches, both threads make progress, frozen after stop),
`SMP:BSP_SPIN:PASS` (AP1 holds KERNEL, waits until the BSP spins on it, sends a shootdown to the BSP
only; ack within 100 ms, no quarantine) and `SMP:IDLE:WAKE_US=<n>` + `SMP:IDLE:PASS` (idle wake
by AP1 with 0xe8, latency logged and < 100 ms; then a BSP thread that waits for a lock of AP1
while the timer is masked: AP1's 0xe8 is deferred and only the release switches). Without an AP:
`SMP:BSP_SPIN:SKIPPED`/`SMP:IDLE:SKIPPED`. Control `test-smp-bsp-spin-irq-off`: the BSP window
stays closed → `SMP:SPIN:SHOOTDOWN_TIMEOUT:APIC=0`, exit 0x10. Mutant without the release hook
(local, not committed): `SMP:IDLE:RESCHED_LOST`. See `docs/verification/k06-5c2-bsp-timer/`.

**Limits and shortcuts:** QEMU/TCG, no hardware evidence. The TSC is assumed to be constant and equal on
all CPUs (QEMU/KVM); real hardware first requires an invariant-TSC check. The
ratio is only logged; `TIMER_INITIAL` remains 20 000 (unchanged tick period). The BSP does not yet
have a run queue: `ADOPTED` (ADR-079) is not yet drained; the step that gives the BSP a queue
(5d-2 or 5f) must do that. An IRQ0 that waits longer than one period (BSP with IF=0) is
lost; the clock counts only ticks that the BSP receives in an IF=1 context. This is inherent to a
tick clock and is measured by the clock check during idle, not during IF=0 work. Out of
scope: TSC deadline, tickless idle, IOAPIC routing, power management (H05).

## ADR-079 — ICR lock per CPU, and quarantine stops at a held lock (K06, audit findings F19/F15)

> Ported to `main` on 13 November 2026 from PR #246 (head `273b2eb`); written on the K06 branches on
> 6 November 2026 and not previously on `main`. English translation; the Dutch original is archived in
> `docs/history/k06-branch-prs-2026-11-06/`. Evidence paths, scores and branch names below refer to
> those branch commits; evidence directories named `docs/verification/k06-*` are archived under
> `docs/history/k06-branch-prs-2026-11-06/verification/`. Later amendments: the `main` entries
> "K06 step 5f — review findings closed before acceptance (ADR-079/ADR-081 addendum)"
> (12 November 2026; IQ-3 transitional lock states) and "K06 step 5f — responsive APs go home before
> the IQ-3 checks (ADR-079 addendum)" (12 November 2026), and ADR-082 (TSC bound on each ICR
> acknowledgement, audit F19 defect repair; 13 November 2026).

6 November 2026; effect only in `native-smp` (the owner field exists in every build, but
single-CPU profiles write only CPU 0 into it). Task specification:
`docs/implementation/k06-steps/05-F19-F15-icr-quarantine.md`. Supplements ADR-071 (quarantine
of a deaf AP) and ADR-072 (shootdown deadline). **Problem:** F19: `send_ipi` wrote ICR high
(0x310) and ICR low (0x300) without a lock. A handler on the same CPU that itself sends an IPI
between those two writes replaces the destination. F15: `quarantine_all` resets a deaf
AP with INIT, even if that AP holds `KERNEL` or a `RUNQ`. That lock then remains closed
forever, and the BSP blocks at the next acquire.

**Decision (normative):**
- **IQ-1 ICR atomic per CPU.** `CpuLocal.icr` is an `IrqLock` at level 60 (ICR). `send_ipi`
  takes that lock (irqsave) around the idle wait, ICR high, ICR low and the concluding idle wait.
  It is the only place that writes 0x310/0x300. One lock per CPU suffices: every xAPIC has its
  own ICR, so only a handler on the same CPU can intervene, and irqsave excludes that.
  Handlers may take level 60 (LD-1 rejects only KERNEL/ASPACE).
- **IQ-2 Every lock knows its owner.** `IrqLock` has an `AtomicU8 owner`: the CPU index of
  the holder (BSP 0, AP slot + 1) directly after acquisition, `FREE` (0xff) before release. The
  ticket logic does not change. `IrqLock::owner()` and `level()` are the read interface.
- **IQ-3 Policy for a dead CPU.** `quarantine_all` first checks all APs
  (`stop_if_held`) and then calls `quarantine_cpu(slot)` per AP. The check comes first
  because the RUNQ lock of one AP can be held by another AP.
  `quarantine_cpu`: on the BSP only; check, INIT, wait 1 ms, check again (a deaf
  AP that was still running may have taken a lock after the first check). The
  READY threads from its queue then go to the BSP (`ADOPTED`), and the thread it was running (or on whose
  stack it was still standing) counts as crashed (`kill_request`, `CRASHED`). Nothing is migrated.
  The owner fields of `lock::KERNEL` and all `RUNQ` are checked; ASPACE, FRAMES and
  SHOOTDOWN do not yet exist as `IrqLock` and must be added to `held_by` as soon as they
  exist. If the AP holds anything, `SMP:QUARANTINE:LOCK_HELD:<laagste niveau>:CPU=<n>` and
  a system stop follow (`serial::fail`, or exit 0x10 in `test-smp-quarantine-lock-held`). Breaking a
  lock open is not permitted.
- **IQ-4 An AP never initiates quarantine.** `quarantine_cpu` on an AP stops with
  `SMP_QUARANTINE_ON_AP`. An AP that receives no ack in the ICR fixture reports this only through
  `icr_done`; the BSP decides. A 0xe8 to the BSP follows only with 5c-2 (the BSP currently accepts no
  interrupts). Verified: no AP path calls `quarantine_all`. Whether an AP itself starts a
  shootdown in 08 remains an open question for 08.

**Evidence:** native fixture `SMP:ICR:PASS:ROUNDS=10000` (AP1↔AP2, each 10 000 0xe8 with an ack before
the next, while every 0xd0/0xe8 entry sends a self-IPI 0xf0 from the handler). Control
`test-smp-icr-race`: without a lock and with one 0xd0 between high and low on AP1, the oracle sees
`SMP:ICR:LOST` (AP1's 0xe8 lands on AP1 and merges there with the 0xe8 of AP2) or
`SMP:ICR:MISROUTED`. `test-smp-quarantine-lock-held`: AP2 takes KERNEL and becomes deaf →
`SMP:QUARANTINE:LOCK_HELD:10:CPU=2`, exit 0x10, no hang. See `docs/verification/k06-f19f15-icr/`.

**Limits and shortcuts:** before 5c-2 the BSP has no run queue, so `ADOPTED` is maintained
but not yet run; 5c-2 must drain it. Kernel threads have no process, so the
K05 fault route has nothing to terminate on a crash; ring-3 threads on APs (5d-2) must take that
route. An AP that dies as a *waiter* in a ticket lock retains its ticket. That lock then
also jams, but IQ-3 (owner only) does not see it. The test lock
`SPIN_LOCK` in `test-smp-spin-irq-off` is exactly such a case (deliberately unusable after the control).
For KERNEL/RUNQ this is an open item; the lockdep mirror (held *or awaited*) could
cover it. Out of scope: recovery after a dead owner, x2APIC (ICR in one MSR write),
broadcast IPIs.

## ADR-078 — Scheduler protocol: two-phase blocking, queued bit and bounded kill (K06, audit findings F16/F17)

> Ported to `main` on 13 November 2026 from PR #246 (head `273b2eb`); written on the K06 branches on
> 6 November 2026 and not previously on `main`. English translation; the Dutch original is archived in
> `docs/history/k06-branch-prs-2026-11-06/`. Evidence paths, scores and branch names below refer to
> those branch commits; evidence directories named `docs/verification/k06-*` are archived under
> `docs/history/k06-branch-prs-2026-11-06/verification/`. Later amendments: none recorded.

6 November 2026; effect only in `native-smp`. Task specification:
`docs/implementation/k06-steps/04-F16-F17-block-wake-kill.md`; spec `K06-SMP.md` §6.1, §6.3
and §6.4 updated. **Problem:** F16: `smp_lock::sched` had no RUNNING→BLOCKED, no
DEAD(_PENDING) and no kill. A thread that decides to block while a waker on another
CPU is already waking it lost that wake (`wake` returned `Err(RUNNING)`). F17: a double enqueue was only
seen at a later `dispatch`, so the double-run test depended on timing.

**Decision (normative):**
- **States** (`AtomicU32`): READY 0, BLOCKED 1, BLOCKING 2, READY_PENDING 3, DEAD_PENDING 4,
  DEAD 5, RUNNING(cpu) = 8 + cpu. Every transition is one atomic step (CAS or `fetch_update`
  with a pure step function): `dispatch`, `yield_cpu`, `block_prepare`, `block_commit`, `wake`,
  `kill_request`, `kill_commit`.
- **BW-1 No lost wake.** The thread performs `block_prepare` (RUNNING(c)→BLOCKING) under the
  lock of its waker or before checking its condition (SeqCst), releases the lock
  and requests a switch. `wake` sets BLOCKED→READY (the waker puts it in the queue and sends 0xe8)
  or BLOCKING→READY_PENDING (no enqueue). The switch performs `block_commit`: BLOCKING→BLOCKED or
  READY_PENDING→READY plus requeue. The thread handles a condition that is already true after
  `block_prepare` with `wake` on itself.
- **BW-2 Only once in a queue.** `queued[tid]`, shared by all queues: `RunQueue::push`
  CAS 0→1, `pop` 1→0. A second push produces `SMP:SCHED:DOUBLE_ENQUEUE` and a stop
  (`test-smp-double-run`: exit 0x10).
- **BW-3 Bounded kill.** `kill_request`: every live state → DEAD_PENDING; if it was BLOCKED,
  the killer puts it in the queue of *d*. Then 0xe8 and waiting for DEAD within
  `SHOOTDOWN_DEADLINE_MS` (100 ms, the same constant as the shootdown acks). At the switch, *d* puts
  a dying current thread back in its own queue and leaves the stack. The
  next switch that pops it performs `kill_commit`: DEAD only if `on_cpu` has been cleared
  (F1), otherwise back into the queue. A DEAD thread is in no queue, and no CPU is
  on its stack. No DEAD within the deadline → `SMP:KILL:TIMEOUT:APIC=<n>` and quarantine of
  all APs (INV-5; the policy with held locks follows in step 05).
- **BW-4 Idle per CPU.** The idle thread is the existing home loop of each AP (`sti; hlt; cli`,
  context in `CpuLocal.saved[0]`, `HOME`). It is never in a queue and runs under the
  kernel CR3. A separate idle-thread stack adds nothing here.
- **BW-5** unchanged: no lock across the switch, only RUNQ within the switch.

**Deviation from the task specification:** the specification asks for loom models. Loom 0.7.2 has not
become a dependency. That would mean 33 new crates with build scripts (`cc`, `rustversion`, `generator`),
and `generator 0.8.12` requires Rust 1.95 while the toolchain is pinned to 1.90. Under AGENTS.md,
provenance, licensing, maintenance and build scripts must be checked first. Instead,
exhaustive interleaving models run in `sched.rs` (without a dependency). They try every
order of the atomic steps of the real functions: block/wake with two threads and a waker,
and kill during RUNNING with an F1 claim and stub. Each model has a mutant that the model must refute:
the 5d-1 protocol loses a wake, and a DEAD published on the stack lets the
cleanup race. Limit: only sequentially consistent executions, no weaker orderings.
Loom remains an open item (F-T1).

**Fixture:** threads A/B on AP1/AP2 pass a turn back and forth 10 000 times under `lock::KERNEL`. They
block with `block_prepare` under KERNEL and request the switch with their own 0xe8 (IF=0 around the
ICR write). The BSP then kills thread K (only in space 1) on AP3. With fewer APs, the
threads share CPUs (`tid % AP's`); without APs, the kernel reports `SMP:WAKE/KILL:SKIPPED`.
`test-smp-resched-deaf`: K sets TPR to class 0xe, so that AP3 does not accept 0xd0/0xe8.

**Evidence:** see `docs/verification/k06-f16f17-sched/`: native QEMU/TCG 25/25 SMP cases across 20
profiles on one source (`SMP:WAKE:PASS`/`SMP:KILL:PASS` for 2/4/8/8 CPUs,
`SMP:SCHED:DOUBLE_ENQUEUE`, `SMP:KILL:TIMEOUT:APIC=3`); M2/M3, rtc, K05 and K07 unchanged and green.
No CI or hardware.

**Out of scope:** the real M3 IPC wait queues (step 08), migration (09), the INV-5 policy with
held locks (05), K05 cleanup and process-wide shootdown after DEAD (07).

## ADR-077 — Lockdep with handler context and lock pairs per level (K06, audit findings F3/F4)

> Ported to `main` on 13 November 2026 from PR #246 (head `273b2eb`); written on the K06 branches on
> 6 November 2026 and not previously on `main`. English translation; the Dutch original is archived in
> `docs/history/k06-branch-prs-2026-11-06/`. Evidence paths, scores and branch names below refer to
> those branch commits; evidence directories named `docs/verification/k06-*` are archived under
> `docs/history/k06-branch-prs-2026-11-06/verification/`. Later amendments: none recorded.

6 November 2026; supplements ADR-074, effect only in `native-smp` (M3 builds the lock code as well,
with depth 0). Task specification: `docs/implementation/k06-steps/02-F3-F4-lockdep-context.md`.
**Problem:** F3: that handlers never take `KERNEL`/`ASPACE` was stated only in comments. A
handler that took them anyway on a CPU without locks passed lockdep and opened the
`sti; pause; cli` window inside the handler (nested interrupts on the same stack). F4: lockdep
had one bit per level, so an IPC copy between two address spaces (two `ASPACE`) or a
migration (two `RUNQ`) produced a false `SMP:LOCKDEP:ORDER`.

**Decision (normative):**
- **LD-1 Context.** `CpuLocal.irq_depth` counts the nested IDT entries of this CPU. Every GS stub
  (0xf0, 0xe0, and 0xd0/0xe8 in the shared timer body) performs `inc gs:[irq_depth]` directly after
  `gs_entry!` and `dec` directly before `gs_exit!`. After a thread switch, every exit still belongs to
  one entry on the same CPU. NMI/#DF/#MC (no GS, GS-6) and the terminal exception stub are
  not counted; they do not return and take no locks. `lock_irqsave` at a level ≤ `ASPACE` with
  depth > 0 produces `SMP:LOCKDEP:IN_HANDLER:<niveau>` and stops, before lockdep and before spinning.
- **LD-2.** The window opens only at depth 0 (in addition to the existing conditions from 5b).
- **LD-3 Pairs.** Two locks of one level only through `IrqLock::lock_pair(a, b)`. It takes them
  in ascending key order, regardless of argument order. The key is fixed at construction
  (`IrqLock::keyed`): pid for `ASPACE`, CPU index (AP slot + 1) for `RUNQ`. `Lockdep` keeps a counter
  per level (0/1/2, as bits in `held` and `pairs`) and the keys taken. A
  second acquire on a held level through `acquire` (thus outside `lock_pair`), a third
  lock, or a second half with a lower or equal key produces `SMP:LOCKDEP:ORDER`. The same
  lock twice, equal keys or different levels also stop with that marker (no
  self-deadlock).
- **LD-4 LIFO remains.** `release(level, key)` requires the highest level and the most recently taken
  key; of a pair, the second goes first. Releasing a lock that is not held is
  a kernel bug (panic).
- **Interface.** `Lockdep::acquire(level, key)`, `acquire_paired(level, key)` (only the second
  half of `lock_pair`; the spec says "outside `lock_pair`", so lockdep must distinguish this),
  `release(level, key)`, `held_mask()`. `from_bits`/`bits` are removed. The complete state is kept per
  CPU in `lock::LockdepCell` (this CPU only, IF=0); the existing `AtomicU64` remains as a mirror
  of `held_mask` for `kernel_held()`, the preempt deferral check and the BSP checks.
  `lock_pair` returns `(lage sleutel, hoge sleutel)` (low key, high key); with `let (low, high) = …`, `high` is dropped first.
  Code that drops the tuple as a whole releases `low` first and stops with the LIFO marker.

**Evidence:** see `docs/verification/k06-f3f4-lockdep/`.

**Out of scope:** more than two locks of one level, a runtime owner per lock (step 05),
the SERIAL level 70 (does not yet fit in the 64-bit mask; step 03), a loom model of
`TicketLock` (F-T1; check a new dependency first, see the evidence).

**Evidence (summary):** native QEMU/TCG 24/24 SMP cases across 19 profiles on one source, with
`SMP:LOCKPAIR:PASS:CPUS=1/2/4/8/8`, `SMP:LOCKDEP:IN_HANDLER:10` and `SMP:LOCKDEP:ORDER:20->20`;
M2/M3, rtc, K05 and K07 unchanged and green; property test against a reference model. No CI or
hardware.

## ADR-083 — `swapgs` discipline for the per-CPU GS anchor (K06, audit finding F8)

> Ported to `main` on 13 November 2026 from PR #246 (head `273b2eb`); written on the K06 branches on
> 6 November 2026 and not previously on `main`. English translation; the Dutch original is archived in
> `docs/history/k06-branch-prs-2026-11-06/`. Evidence paths, scores and branch names below refer to
> those branch commits; evidence directories named `docs/verification/k06-*` are archived under
> `docs/history/k06-branch-prs-2026-11-06/verification/`. Later amendments: none recorded.
>
> Numbered ADR-076 on the K06 branches. `main` already uses ADR-076 for ACPI S5, so this decision
> takes the next free number. Code comments that cite ADR-076 together with GS-1..GS-10 (for example
> `kernel/src/m2/arch.rs`) refer to this ADR.

6 November 2026; applies to all profiles, with effect only in `native-smp`. ADR-075 remains
reserved for 5b-2. **Problem:** the per-CPU anchor (`CpuLocal`) is held in
`IA32_GS_BASE`, but the IDT stubs 0xf0, 0xe0 and 0xe8 followed `gs:[0]` without `swapgs`. Ring 3
can set GS_BASE to 0 without privileges (`mov gs, 0x1b`; the DPL3 data descriptor has base 0),
after which every IPI follows a pointer from user memory and performs a locked write to it (SMEP
and SMAP are disabled). This blocks ring 3 on APs (5d-2).

**Decision (normative: GS-1..GS-10 in `docs/implementation/K06-SMP.md` INV-8):**
- In CPL0, the anchor of this CPU is in GS_BASE and the user value in KERNEL_GS_BASE; in
  CPL3 it is the other way round. On entry, `swapgs` follows only if CS in the hardware frame of *that*
  entry has RPL3. On exit, the same applies to the frame that `iretq` restores (after a switch,
  possibly that of another thread). No earlier decision is remembered.
- One definition: `arch::gs_entry!` (`test; jz; swapgs; lfence` as the first instructions, before
  any push or `gs:` operand) and `arch::gs_exit!` (`test; jz; swapgs` directly before a labelled
  `iretq`, with IF=0). After its own prologue, 0xe8 jumps to the timer body: one `swapgs` per
  entry.
- Exceptions: the same prologue in `m2_exception_common`. An `iretq` to CPL3 that faults
  (#GP/#SS/#NP) reports CS=8 while the user GS is already loaded. A frame RIP equal to a
  `nos_iret_user_*` label therefore also swaps, and the path remains terminal. NMI, #DF and #MC go
  through `m2_exception_paranoid`: never `swapgs` or GS, terminal. The IST stack address
  identifies the AP. M3 now also sends #MC to `m3_terminal`.
- Precondition: CR4.FSGSBASE=0 and EFER.SCE=0, otherwise `SMP:GS:PRECONDITION` and a stop. FSGSBASE,
  `syscall`/`sysret`, user TLS through GS and returning NMIs each require a new ADR.

**Evidence:** red first. Commit `c2f1bed0` (fixture on the old stubs) produces
`SMP:GS:HIJACK` and `M2:FAIL:KERNEL_EXCEPTION`. Then, on the fix, N1 `SMP:GS:PASS:CPUS=4`, N2
`SMP:GS:IRET_FIXUP:PASS` and N3 `SMP:FAULT:AP_NMI_IST:PASS` (QEMU-TCG, 4 vCPUs). A mutant
without the GS-7 label comparison makes N2 fail. The static lint `tools/tests/test_gs_discipline.py`
is the only evidence for the `lfence` (TCG does not model speculation) and rejects the old stubs.
See `docs/verification/k06-f8-swapgs/`.

**Deviations from the task specification:** VA 0 is not mapped as a user page in the fixture, because it
falls within the shared kernel PD. The red evidence is therefore the supervisor #PF at cr2<4096, not a
write into a sentinel page. N2 uses SS=0x13 instead of a non-canonical RIP:
QEMU-TCG does not check that RIP on `iretq` (the #GP came from CPL3). This has not been tested on real
hardware. The GS-10 check runs on the BSP; APs copy its CR4/EFER exactly or are rejected as
MISMATCH.

**Open:** SMEP/SMAP (separate hardening), the selector leak between processes on one CPU (before 5d-2),
#MC on its own IST stack, AMD null-selector behaviour (not testable under TCG) and hardware evidence.
AI review is not a human audit (B05).

## ADR-075 — One KERNEL lock around the M3 entry and a scratch slot per CPU (K06 step 5b-2; revision of ADR-024)

> Ported to `main` on 13 November 2026 from PR #246 (head `273b2eb`); written on the K06 branches on
> 6 November 2026 and not previously on `main`. English translation; the Dutch original is archived in
> `docs/history/k06-branch-prs-2026-11-06/`. Evidence paths, scores and branch names below refer to
> those branch commits; evidence directories named `docs/verification/k06-*` are archived under
> `docs/history/k06-branch-prs-2026-11-06/verification/`. Later amendments: ADR-082 (D2, scratch
> phase of spec 01: per-CPU progress, `SMP:SCRATCH:STALLED`, 5 s cap; 13 November 2026).

6 November 2026. **Problem:** ring 3 on APs (5d-2) means that two CPUs can handle a
syscall or fault at the same time. The M3 state (`STATE`), the frame allocator, paging, IPC wait queues,
quota and the supervisor have so far been protected only by "one CPU, IF=0". The single global
scratch slot from ADR-024 lets two CPUs zero or read each other's frame: a leak or corruption between
processes (INV-2). This step makes the kernel SMP-safe **without** changing single-CPU behaviour.
M3 remains excluded from `native-smp` until 5d-2.

**Decision (normative: K-1..K-5 in `docs/implementation/k06-steps/01-5b-2-kernel-lock-scratch.md`):**
- **K-1 One kernel section.** `lock::KERNEL: IrqLock<()>` (level 10) protects every mutation of
  `STATE`, the frame allocator, paging, IPC wait queues, quota and the supervisor. `m3_event` takes
  the lock directly after the entry stub and releases it only just before the return to the stub.
  Before the lock, `m3_event` reads only the CS of its own interrupted frame. A
  kernel exception inside a held section thus keeps the marker `M3:FAIL:KERNEL_EXCEPTION` and does
  not end as a lockdep error. `m3::start` takes the lock before building `STATE`.
  `lock::kernel_held()` (this CPU's lockdep bit) is checked in `select`/`cleanup`
  with a `debug_assert`, and in the native fixture with an `assert`.
- **INV-7 and the jump to ring 3.** `complete_round` and `m3::start` resume user mode with
  `jmp m3_restore` from a deep call, without unwinding the stack of `m3_event`. A
  local guard would never be dropped there: KERNEL would then remain taken and the next event would hang. The
  guard is therefore kept in one slot (`KERNEL_GUARD`) that only the holder touches, so the lock itself
  makes access exclusive. `kernel_leave()` sits before every `return` to the stub and in
  `resume()`, the only route to `m3_restore` from Rust. No lock spans a switch.
- **Single-CPU profiles.** `lock.rs` now also compiles for M3, with one lockdep word and without a
  GS anchor. On one CPU, the lock is always taken immediately, without a waiter and without an IF window (K-3).
  Idle with `sti; hlt` remains inside the section: the IRQ0 idle route touches only raw counters. On
  SMP, idle must leave the section (5c-2/5d-2).
- **K-4.** SAFETY texts in `m3.rs` and `m3/*.rs` cite "KERNEL held" as the justification, no longer
  "single CPU".

**Revision of ADR-024 (K06-SMP §5, points 1–7):**
1. Mapping arena unchanged, 2048 mappings per process, static. Access under `KERNEL`;
   `ASPACE[pid]` follows when fine-grained locks arrive.
2. **Scratch slot per CPU (K-2).** CPU *i* uses only VA `SCRATCH + i·4096` (`SCRATCH =
   0x3fff0000`, 8 slots, supervisor RW/NX or RO/NX). All slots share the PT of `FIRMWARE_SCRATCH`
   (compile-time assert), so one CPU keeps the same number of tables. The address exists only
   through `paging::scratch_slot()`, with index `smp::cpu_index()` (BSP 0, AP slot+1, through the
   GS anchor) or always 0 without `native-smp`. Invalidation remains local (`invlpg`). An occupied
   slot PTE causes a stop: `SMP:SCRATCH:CROSS` on SMP, otherwise a panic.
   *Residual risk:* a speculatively cached foreign slot entry is never used architecturally,
   but speculative side channels on it fall outside this model.
3. The refusal of an executable alias and the RO/NX helper remain unchanged.
4. There is still no identity map of free RAM or MMIO.
5. ASID remains an internal counter and PCID remains disabled.
6. A change in the shared kernel half, other than the CPU's own scratch slot, requires a
   shootdown to all online CPUs (ADR-072).
   *Addendum step 5e:* built as `shootdown_kernel` (target set `ONLINE_CPUS & !self`). The
   kernel half is exactly PDPT entry 0 (one PD). Every space shares that PD through the pointer
   (`Paging::new_user`). Nothing is therefore copied that could fall out of step, and a
   new PML4 entry is never needed. In `native-smp`, asserts check that no kernel mapping is placed above 1 GiB
   and that the kernel root is empty outside entry 0.
7. **Revise upon:** PCID, fine-grained subsystem locks, idle outside the section (5c-2), M3 on APs
   (5d-2), DMA/IOMMU or a new mapping API. The big lock limits parallel
   syscall throughput (R02 risk, not a K06 requirement).

**Consequence for `native-smp`:** the BSP calls `begin_scratch` in `smp::start`, directly after its
GS anchor and before the first AP. It then reads and writes paging through its own slot.
`Paging::audit` skips only the active slot of the reading CPU: the walk itself maps the
table being read there. Every other slot must be empty. APs map a temporary page only in
their own slot; allocator and page-table changes remain BSP-only (ADR-070).

**Evidence:** native fixture after `SMP:SCHED:PASS`, before `SMP:GS:*` and `SMP:PARKED`. Every CPU
zeroes, fills (`0xa5 ^ index`) and checks its own frame 1024 times through its own slot,
concurrently with the other CPUs. Every CPU then takes `KERNEL` 1000 times and increments a counter
with a separate load and store. The requirement is: all patterns intact, counter = 1000 × CPUs, lockdep
empty → `SMP:SCRATCH:PASS:CPUS=n`. The control profile `test-smp-shared-scratch` (all CPUs in
slot 0, 4 vCPUs) must produce `SMP:SCRATCH:CROSS:CPU=<n>` and exit 0x10. That marker reports both
an occupied slot PTE and a foreign pattern. Native results and commands are recorded in
`docs/PROJECT-STATE.md`.

**Deviations from the task specification:** the 500 ms deadline starts only after the BSP has completed its own
share; the M2 PIT helpers do not measure total run time. The guard is kept in a slot instead
of in `let _k`, for the reason above. The SCRATCH VA moves from `0x3fffe000` to
`0x3fff0000`; the table is the same. The fixture frames are released after the test: every CPU
has cleared and invalidated its slot PTE itself, and no CPU touches a foreign slot VA.

**Open:** KERNEL on the M3 path on APs and idle outside the section (5c-2/5d-2), fine-grained locks,
hardware evidence. QEMU-TCG does not model speculation. AI review is not a human audit (B05).

## ADR-074 — Ticket lock, irqsave and lockdep per CPU (K06 step 5a)

6 November 2026; only in `native-smp`. This is the first step towards an
SMP scheduler for processes (spec K06 §4). The crate `crates/smp-lock`
(no_std, host-testable) provides a FIFO ticket lock (Mellor-Crummey & Scott)
and `Lockdep`. `kernel/src/m2/lock.rs` contains `IrqLock`: it saves
RFLAGS.IF, executes `cli`, records the lock level in this CPU's lockdep set
and then spins. On release, the lock is released first, then the level is
cleared, and only then is IF restored. The BSP receives its own GS anchor
(CPU index 0), separate from the AP anchors. AP-specific lookups (preemption,
IST markers) therefore continue to exclude the BSP.

**Lock order (outer → inner):** KERNEL 10, ASPACE 20, FRAMES 30,
RUNQ 40, SHOOTDOWN 50, ICR 60, SERIAL 70. A CPU acquires only a level
strictly higher than everything it already holds. A violation stops the VM
with `SMP:LOCKDEP:ORDER:<held>-><wanted>`. Lockdep is active in every
`native-smp` build. Bounded spinning with `SMP:LOCK:STUCK` is active
only in debug builds.

**Evidence (as of this commit):** host tests in `smp-lock`: exclusion
under 8 threads (catches a mutant without a wait loop), FIFO, wrap,
order violation, held-across-switch and a compile_fail test for a
`Send` guard. The AP stress test now uses this ticket lock. The serial oracle
requires `SMP:LOCKDEP:PASS` on every non-terminal path. The negative profile
`test-smp-lockdep-order` must produce `SMP:LOCKDEP:ORDER:30->10`. **Native
QEMU execution remains open** (workflow `native-smp.yml`).

**Open:** nothing yet uses `IrqLock` outside the self-test. The KERNEL lock
around M3 syscalls and the per-CPU scratch slot follow in step 5b; the revision
of ADR-024 will therefore become ADR-075, not ADR-074 as the spec proposed.
Lockdep has one bit per level, so two simultaneous RUNQ locks will require
a nesting counter in step 5d. Spinning with IF=1 on KERNEL/ASPACE (§4.3)
follows in 5b.

> Ported on 13 November 2026 from PR #246 (head `273b2eb`); English translation of a branch addendum, Dutch original in `docs/history/k06-branch-prs-2026-11-06/`; evidence paths refer to branch commits.

**Addendum step 5b (6 November 2026): controlled spinning with IF=1.** A CPU
waiting for a lock of level KERNEL or ASPACE opens a `sti; pause; cli` window
between two polls. A CPU waiting in this way can therefore still acknowledge a
shootdown (0xe0) from the lock holder, which itself waits for that ack with IF=0.
Without the window, a deadlock arises. Scope:
- Only for level ≤ ASPACE. All other levels spin with IF=0, because RUNQ,
  ICR and SERIAL are also taken from handlers.
- Lockdep records the level before spinning. A waiter therefore holds at most
  KERNEL, which lies below every handler level. Handlers never take
  KERNEL/ASPACE; in this step, that is a structural contract without a runtime check.
- Only on APs (`smp::on_ap`). The BSP receives the window in 5c, once its
  interrupt sources are fixed.
- The saved IF from before `lock_irqsave` does not change. Release builds spin
  without bound via `TicketLock::lock_with`.
- **Precondition for 5c/5d:** before a timer is armed on a CPU that takes
  KERNEL/ASPACE, the preemption handler must not switch while the
  lockdep set is non-empty (INV-7). In 5b, the AP timer is already masked in the spin phase.

Evidence: a native fixture after PREEMPT. The BSP holds a lock at KERNEL level
while every AP waits for it with IF=0, and requires a shootdown ack
from every AP within 100 ms (`SMP:SPIN:PASS`). The control profile `test-smp-spin-irq-off`
(without the window) must produce `SMP:SPIN:SHOOTDOWN_TIMEOUT:APIC=1`, followed by
quarantine. This proves that the fixture measures a real hazard. Locally: the
host test `lock_with`, the serial oracle and kernel builds. **Native QEMU remains
open** (`native-smp.yml`). The former 5b content (KERNEL lock around M3 syscalls,
scratch slot per CPU, ADR-075) moves to step 5b-2.

> Ported on 13 November 2026 from PR #246 (head `273b2eb`); English translation of a branch addendum, Dutch original in `docs/history/k06-branch-prs-2026-11-06/`; evidence paths refer to branch commits.

**Addendum step 5c (6 November 2026): no switch with a held or watched
lock (INV-7).** Lockdep applies per CPU, not per thread. A timer tick
(0xd0) can arrive with a lock held or watched only through the
5b window: inside a held section, IF=0. `smp_preempt` therefore defers such a tick
when this CPU's lockdep set is non-empty. The interrupted context
resumes unchanged, `deferred` is counted, and a later tick tries again. This
also applies to STOP, the switch to home. A switch that nevertheless takes place with a non-empty set
produces `SMP:LOCKDEP:HELD_ACROSS_SWITCH` and stops (spec §4.4).
- No `need_resched` or immediate rescheduling on release: the next tick
  suffices. *shortcut: a switch waits at most one timer period; revise in 5d if
  run-queue latency requires it.*
- The BSP LAPIC timer and idle with `sti; hlt` (the former 5c) are now called 5c-2. The
  BSP window from 5b follows there.

Evidence: folded into `SMP:PREEMPT:PASS`. The BSP holds a lock at KERNEL level
while the running thread on every AP waits for it. Requirement: `deferred` rises,
`switches` stays constant over at least two deferred ticks, and after the
release `switches` rises again. The negative profile
`test-smp-held-across-switch` (without deferral) must produce the marker, with exit
0x10. Locally: serial oracle and kernel builds. **Native QEMU remains open**
(`native-smp.yml`).

> Ported on 13 November 2026 from PR #246 (head `273b2eb`); English translation of a branch addendum, Dutch original in `docs/history/k06-branch-prs-2026-11-06/`; evidence paths refer to branch commits.

**Addendum step 5d-1 (6 November 2026): run queues per AP and CR3 switch (spec
§6.1–6.4).** Step 5d has been split. Ring-3 processes on APs (5d-2) first require the
KERNEL lock around M3 and a scratch slot per CPU (5b-2). Without that lock, two
CPUs mutate the M3 state and the single global scratch slot at the same time (INV-2). 5d-1 builds the
scheduler core with kernel threads:
- `smp_lock::sched`: a FIFO `RunQueue` with fixed capacity and a status word per
  thread. A thread reaches CPU *c* only through CAS READY → RUNNING(*c*) (INV-3);
  handing it back goes through RUNNING(*c*) → READY (Release, after saving RSP). A
  failed CAS or a full queue produces `SMP:SCHED:DOUBLE_RUN` and stops.
- `sched_switch` runs in the timer path, after the 5c deferral, and therefore with an empty lockdep
  set (INV-7). It holds only RUNQ (level 40), for the handover and the
  CR3 switch, and releases it before IRETQ.
- Address-space switch according to §6.3: `ACTIVE[nieuw] |= c` → `mov cr3` →
  `ACTIVE[oud] &= !c`, with SeqCst (INV-4). The test spaces are PML4 copies of the
  kernel root: the same kernel half, no user mappings. *shortcut: the two root frames
  are not released; real process spaces come in 5d-2.*
- Vector 0xe8 (wake-up): the stub counts and takes the timer path. `arch::smp_gate` admits
  0xe8 as a supervisor gate.
- No FXSAVE in this switch: kernel threads are soft-float. FPU context belongs to
  ring 3 in 5d-2 (ADR-045).

Evidence: `SMP:SCHED:PASS:CPUS=n` after `SMP:SPIN:PASS`. Two threads per AP, one per
space; the second is woken cross-CPU with 0xe8. Every AP switches CR3 at least 4×,
and every thread checks its CR3 and its `ACTIVE` bit with IF=0. After STOP,
every AP is home under the kernel CR3, `ACTIVE` is empty and progress has stopped. The
negative profile `test-smp-double-run` puts thread 0 in two queues and must produce the marker,
with exit 0x10. Native, local: see `docs/verification/k06-smp-sched/`.

> Ported on 13 November 2026 from PR #246 (head `273b2eb`); English translation of a branch addendum, Dutch original in `docs/history/k06-branch-prs-2026-11-06/`; evidence paths refer to branch commits.

**Addendum audit fixes F1/F2 (6 November 2026, K06 audit).**
- **F2, LIFO release.** Every `IrqGuard` restores its own IF. If an outer
  guard were released before an inner one, `sti` would follow while the inner lock
  (e.g. RUNQ) is still held. A tick would then deadlock on its own RUNQ.
  `Lockdep::release` now requires `level == highest()`. Otherwise, the VM stops with
  `SMP:LOCKDEP:LIFO:<held>-><released>`, before IF is restored.
- **F1, stack handover.** `yield_cpu` publishes READY while the CPU is still running
  on the thread's stack. The CPU leaves that stack only in the stub, with
  `mov rsp`. A per-thread flag, `THREAD_ON_CPU`, governs the handover.
  `smp_preempt` returns `Resume {rsp, left}` in RAX:RDX. The stub clears
  `[left]` directly after `mov rsp`; on x86, that is a release store. `sched_switch` claims
  the flag of the next thread with a CAS (Acquire). **There is never any waiting in the
  switch.** Otherwise, two CPUs exchanging threads would wait for each other with IF=0.
  If the claim fails, the thread goes back through RUNNING(*c*) → READY. It goes to the back
  of its own queue, and the CPU runs that tick at home. This is the same pattern as
  `on_cpu` in Linux, but without waiting in the switch. *Limit: the double-run test remains
  timing-dependent (F17, a `queued` bit, is still open).*

**Alternatives considered.** The existing TTAS lock: replaced because it
is not FIFO and can starve under TCG. Lockdep only in debug builds:
rejected because the native gate runs release builds.

> Ported on 13 November 2026 from PR #246 (head `273b2eb`); English translation of a branch addendum, Dutch original in `docs/history/k06-branch-prs-2026-11-06/`; evidence paths refer to branch commits.

**Addendum audit fixes F5/F7 (6 November 2026, K06 audit, L0 spec
`docs/implementation/k06-steps/03-F5-F7-stuck-serial.md`).**
- **F5, bounded waiting in release builds too (DG-1).** The bounded spin depended on
  `cfg!(debug_assertions)`, but every native gate builds `--release`: a deadlock
  then became a host timeout without a diagnosis. Now `cfg(feature = "native-smp")` selects the
  bounded variant. All `test-smp-*` profiles inherit that feature. The limit is a
  time: `LOCK_STUCK_MS = 4 × SHOOTDOWN_DEADLINE_MS` = 400 ms, counted from the first
  contended poll. Exceeding it produces `SMP:LOCK:STUCK:<niveau>:CPU=<n>` and stops
  the VM. Single-CPU profiles (M3) remain unbounded: there is no waiter there.
- **Time base, deviation from the letter of the spec.** The spec says "measured with the
  PIT clock". But the PIT belongs to the BSP (ADR-070), and an AP must not touch it. The BSP
  therefore calibrates 10 ms of PIT channel 2 against the TSC (`TSC_PER_MS`) before the first IPI. A
  waiter on any CPU reads only the TSC. The PIT thus remains the time base,
  but the measurement during spinning runs through the TSC. Assumption: the TSC runs at a
  constant rate (QEMU/TCG, KVM). *shortcut: replace this with the calibrated TSC of
  step 06.* If the calibration fails, the boot stops with `M1:FAIL:SMP_TSC_CALIBRATION`.
- **F7, one line is indivisible (DG-2).** Under `native-smp`, `serial.rs` has a
  `static SERIAL: IrqLock<()>` at level 70 (irqsave, bounded according to DG-1).
  `serial::write` collects fragments in a per-CPU line buffer (IF=0). At the
  newline, the whole line goes out at once under `SERIAL`. In this way, the
  more than two hundred existing write sites remained unchanged. Terminal paths (the shared
  exception handler with NMI/#DF, `panic`, `fail`, `lock_stuck`) write with
  `write_terminal`: a bounded `try_lock` without lockdep. If that fails, the
  line goes out anyway, prefixed with `!`. The host oracle accepts `!` only on the
  last line of a terminal profile.
- **Finding: lockdep has no level 70.** `smp_lock::Lockdep` tracks levels 0..=63
  in one `u64` (`assert!(level < 64)`). `SERIAL = 70` had never been used until now.
  *shortcut: `SERIAL` is not recorded in the lockdep word.* It is the highest
  level and a leaf (`write_raw` takes no lock), and it is held only with IF=0.
  DG-1 sees a re-entry on the same CPU as `SMP:LOCK:STUCK:70`.
  Widen `Lockdep` (crate `smp-lock`, outside this step) as soon as a level above 63
  needs ordering.
- **DG-3.** Without `native-smp`, and before `smp::start` sets the GS anchor, the bytes go
  unchanged and directly to the UART. `fail` and `panic` now assemble their line first,
  with byte-identical output.
- **Evidence.** In every `native-smp` image, after the sync test, every CPU writes 200 lines
  `SMP:SER:<cpu>:<seq>` in fragments, interleaved. `SMP:SERIAL:PASS` then follows. The
  host requires every line intact, the order 0..199 per CPU, and no line outside that phase.
  `test-smp-lock-stuck` (2 vCPUs): the BSP holds a KERNEL-level lock. Expected:
  `SMP:LOCK:STUCK:10:CPU=1`, exit 0x10. Native, local: `docs/verification/k06-f5f7-diagnostics/`.

## ADR-076 — Bounded ACPI S5 and non-delegable POWER authority (H05 steps)

5 November 2026; extended on 10 November 2026. The opt-in
`native-acpi-s5` profile remains an IF0 bootstrap fixture, mutually exclusive
with M3, framebuffer, SMP and RTC. `boot-protocol` records the address of the
sole FADT root entry. A duplicate FADT clears that address without changing the
other inventory. During one-time discovery, the kernel copies the FADT
(≤ 4 KiB) and DSDT (≤ 64 KiB) through `FirmwareReader`, which permits only
ACPI reclaim/NVS memory. `drivers/acpi-power::parse_s5` validates the copy.
Only a valid plan permits PM1a and, when present, PM1b port I/O: first
`SLP_TYP` with `SLP_EN` cleared, then `SLP_EN` set in ACPICA order.

**Contracts.** The standalone `native-acpi-s5` fixture exposes no syscall or
IPC route. A missing, unreadable or rejected table produces a `POWER:S5:*`
diagnostic and the normal debug exit without a PM1 write. Its runner uses
`-action reboot=reset`, so a reset produces a second firmware boot instead of
being mistaken for power-off.

The separate `native-system-power` fixture adds syscall `0x90`
(`POWER_OFF`). Only init receives the sole process-bound `POWER` object.
The type supports no derivation, delegation, transfer or revocation operation;
a copied numeric handle grants no authority in another process. The syscall
validates zero-valued reserved arguments, the typed handle, the S5 plan, the
volume broker's durability state, absence of block authority and an idle
keyring broker. A successful request terminates its caller as `exit(0)`.
PM1 is written only after all fixture evidence is complete and the same
storage gate succeeds again.

Volume IPC v6 adds the domain-only `SHUTDOWN` request. It aborts unacknowledged
sessions, performs one final block flush and closes the instance only after
successful completion. A flush error or timeout leaves durability unknown,
returns a fail-closed error and prevents S5. The native fault profiles inject
exactly the first shutdown flush after an early power request; each case is
followed by cold recovery of the same disposable image.

**Evidence boundary.** The historical S5 record under
`docs/verification/h05-acpi-s5/` remains bound to its recorded source. Host,
POWER/cold-recovery, S5 and current-main integration results must be reported
against their exact source fingerprint. Exit 33 and a rejection marker show
that the tested case did not power off; they are not independent PM1 bus traces.

**Status and open scope.** H05 remains open at 39/100. ACPI mode is not enabled
through `SMI_CMD`; `_PTS` and `_TTS` are not executed. Coordinated flushing
of every future service, production session UX, S3 sleep/wake, thermal and
battery management, independent security review and physical-hardware evidence
remain absent.

**Alternatives considered.**
- A general AML interpreter such as ACPICA was rejected for these bounded
  fixtures. The restricted `_S5_` subset is sufficient for fixed QEMU.
- Writing directly to the QEMU-specific port `0x604` was rejected because it
  would not demonstrate table-derived power control. The value is used only as
  an independent runner oracle.
- A delegable generic shutdown right was rejected. POWER is manifest-only,
  process-bound and deliberately excludes delegation and transfer rights.

**Primary sources.**
- UEFI Forum (2022), *ACPI Specification*, version 6.5, §§4.8.3.2 and 7.4.2: https://uefi.org/specs/ACPI/6.5/
- ACPICA (4 April 2025), release `R2025_04_04`, `hwsleep.c` (`AcpiHwLegacySleep`): https://github.com/open-acpica/acpica/releases
- QEMU Project, version `10.2.1`, `hw/acpi/ich9.c` and `hw/acpi/core.c` (`acpi_pm1_cnt_write`): https://gitlab.com/qemu-project/qemu

**Consequences.** These fixtures validate bounded S5 and one capability-gated
storage shutdown path on fixed QEMU/OVMF. They grant no general physical-device
support. Invalid-firmware and non-durable-storage tests must continue to prevent
every effective S5 transition.

**Review criteria.** Revisit this decision before physical-hardware support,
additional shutdown participants, ACPI enable, extra AML methods, different PM1
register formats or a user-facing restart/suspend policy is introduced.

## ADR-073 — Preempted kernel threads on APs (K06 step 4)

5 November 2026; only in `native-smp`, the BSP remains IF=0. Each AP
receives a periodic LAPIC timer (vector 0xd0). The stub saves all 15 GPRs
on the interrupted stack, and `smp_preempt` selects round-robin between
the home loop and two kernel threads, each on its own guarded stack. The
kernel is soft-float without a red zone; GPRs plus the CPU interrupt frame
therefore form the complete thread context. There is no FPU, XSAVE or
per-thread CR3 switch.

**Contract.** A thread runs only between RUN and STOP. At STOP, the next
tick masks the timer and returns the AP to its home loop. Only then does
it report RESUMED. The BSP requires at least 8 ticks and 4 context switches
per AP, and progress by both threads. An AP that does not achieve this is
treated as an unresponsive AP: all APs receive INIT and the BSP continues alone.

**Evidence:** `docs/verification/k06-smp-preempt/`. **Open:** these are
kernel threads without their own address space, there is no shared run queue
or migration, and there are no interrupts or preemption on the BSP. There
is no lock order for kernel subsystems and no user processes on APs;
ADR-024 is unchanged. This step changes nothing in other profiles.

**Status:** accepted in the opt-in `native-smp` profile; K06 remains open.

**Alternatives considered.**

- TSC-deadline timer: deferred because it requires CPUID detection and
  calibration; the periodic LAPIC timer suffices for this evidence.
- Cooperative threads (`yield`): rejected because they do not demonstrate preemption.
- A full XSAVE/FXSAVE context: unnecessary because the kernel is soft-float
  without a red zone; it remains mandatory for user processes with FPU.

**Primary sources.** Intel SDM Vol. 3A: APIC timer (LVT timer, divide
configuration, initial/current count, periodic mode) and interrupt handling
(interrupt frame, RSP alignment on an interrupt without a privilege change,
IRETQ). Chapter numbers differ by edition.

## ADR-072 — TLB shootdown before reusing a physical page (K06 step 3)

5 November 2026; supplements ADR-070/071, only in `native-smp`. On SMP,
the `Frames::unmap` contract (original: “PTE wissen en TLB invalideren vóór het
vrijgeven van metadata”; English: “clear the PTE and invalidate the TLB before
releasing metadata”) applies to **every** CPU that can cache the translation.
With one shared CR3, a CPU can cache any present translation, including
speculatively. This therefore applies to **all** aliases of the frame,
and releasing a replaced frame follows this order:

1. Clear the frame while its own bootstrap alias still exists.
2. Remove both PTEs locally (window → new frame, identity alias removed)
   and execute local `invlpg`. The alias metadata remains in place.
3. Shootdown IPI (vector 0xe0) to every AP: `invlpg` for both published
   addresses, then an acknowledgement of this round's generation.
4. Only after **all** acknowledgements: release both aliases, `free` and
   perform a full paging audit.

If an acknowledgement is missing, the old frame remains quarantined
(ownership and alias retained, never reused) and the BSP resets all APs
with INIT. This fails closed because a frame that another CPU can still
read must never pass to a new owner.

**Evidence:** every CPU subsequently verifiably sees the new frame. A
control profile without shootdown deterministically shows that all APs
otherwise read the released frame: the test therefore measures an actual
hazard. See `docs/verification/k06-smp-shootdown/`. **Open:** this is one
window in a bootstrap profile. General shootdown for process address spaces
(per ASID/CR3 and per range), integration with M3 unmaps, interrupts on the
BSP and the SMP scheduler are still to follow; ADR-024 remains valid unchanged
until then.

**Status:** accepted in the opt-in `native-smp` profile; K06 remains open.

> Ported on 13 November 2026 from PR #246 (head `273b2eb`); English translation of a branch addendum, Dutch original in `docs/history/k06-branch-prs-2026-11-06/`; evidence paths refer to branch commits.

**Addendum 6 November 2026 — process-wide shootdown on `active_cpus` (K06 step 5e, audit finding
F18).** Task specification `docs/implementation/k06-steps/07-5e-process-shootdown.md`. The round has
become general (`smp.rs`):
- **PS-1:** after the PTE change, `shootdown_process(space, start, pages)` takes a `fence(SeqCst)` and
  the snapshot `targets = ACTIVE[space] & !self`. Only those CPUs receive 0xe0. The switch order
  (bit set before the CR3 load, cleared after it; §6.3 step 5) makes `ACTIVE` a superset of the CPUs that can cache the old
  translation.
- **PS-2:** one published round `{root, start, pages, gen}` under the lock `SHOOTDOWN`
  (level 50). A round with `root = 0` applies to the shared kernel half. Up to 32 pages,
  the handler uses `invlpg` per page; above that, a CR3 reload. The handler (`smp_shootdown`, Rust behind
  a stub that saves the caller-saved registers) flushes only if its CR3 is the published root,
  or for the kernel half. It **always** acknowledges with the generation and counts its rounds per CPU.
  In the fixture, "pid" is the space index; the handler compares the root, not a pid.
- **PS-3:** `Paging::unmap_shot` clears the PTE and invalidates locally. The round then follows, and only after
  all acks the release of the alias metadata, so only then can `free` succeed. If an ack is missing,
  the metadata remain in place (quarantine) and the IQ-3 policy (ADR-079) follows.
- **PS-4:** `shootdown_kernel` goes to all online CPUs (`ONLINE_CPUS`, including the BSP when an AP starts the
  round). The wait runs on the calibrated TSC, so that every CPU can start a round; the
  round from AP1 to the BSP from step 5c-2 now uses the same function.
- **PS-5:** the fixture removes a space only when `ACTIVE` is 0. It unmaps with a round, performs a
  round over the whole user GiB (CR3 reload on every target, so paging-structure caches are also flushed) and then releases
  the tables. It is checked that no live space references a free frame and that
  `ACTIVE` of a dead space is 0.

**Evidence (local, QEMU/TCG):** see `docs/verification/k06-5e-proc-shootdown/`: isolation 10 000×
per space, unmap race with a controlled #PF, target set counted per CPU, and both negative
profiles. **Open:** M3 processes do not run under `native-smp` (`compile_error`), so the M3 unmap
and cleanup route and the engine `check_invariants` have not been changed. That follows with 5d-2. There is no
rights-reduction API yet, because nothing calls it (YAGNI); it will follow the same `unmap_shot` pattern.
FRAMES is not yet a lock: only the BSP has `&mut Frames`.

**Alternatives considered.**

- Invalidate only the window and drop the identity alias locally: rejected
  after review because an AP with a shared CR3 can cache that alias speculatively.
- Reload CR3 or perform a full TLB flush per AP: also correct, but coarser;
  `invlpg` per address suffices here and remains verifiable per translation.
- PCID/INVPCID: not used; PCID is disabled in this profile.
- Release the frame and perform a “lazy” shootdown afterwards: rejected
  because this is precisely the hazard demonstrated by the control profile.

**Primary sources.** Intel SDM Vol. 3A: TLB maintenance (“Invalidation of
TLBs and Paging-Structure Caches”, “Propagation of Paging-Structure Changes
to Multiple Processors”, INVLPG as a serializing instruction) and APIC (ICR
and TPR). Here too, chapter numbers differ by edition.

## ADR-071 — Own TSS/IST per AP and bounded, targeted IPIs (K06 step 2)

5 November 2026; supplements ADR-070, still only in `native-smp`.
Each AP receives its own GDT copy with its own TSS (RSP0, IST1 for #DF,
IST2 for NMI). Its stacks are in the kernel image, with unmapped guards.
A busy TSS is never shared. IA32_GS_BASE points to a per-CPU anchor;
this is the basis for later per-CPU state.

**IPI contract.** Vector 0xf0 is a supervisor interrupt gate. The stub
counts the interrupt in the per-CPU anchor, sends EOI to its own xAPIC and
returns. Vector 0xff is the spurious gate. An AP waits with `sti; hlt`
only after all LVTs are masked, TPR is 0 and SVR is enabled. The BSP sends
only unicast IPIs. Broadcast shorthands are forbidden because they would
reach CPUs owned by firmware (for example, after a capacity limit). Each
AP must show exactly one delivery per round. If an AP does not accept a
targeted interrupt, the BSP resets all APs with INIT and continues alone.
A CPU that does not respond to an IPI can never subsequently participate
in TLB shootdown. A failed INIT stops the boot.

**Evidence.** NMI and #DF on an AP natively reach that AP's own IST stack;
previously, this caused a triple fault. See
`docs/verification/k06-smp-ipi/`. **Open:** the BSP remains IF=0, and there
is no TLB shootdown, SMP scheduler or lock order for kernel subsystems yet.
APs therefore still do not run kernel subsystems.
ADR-024 remains valid unchanged. Physical hardware has not been tested.

**Status:** accepted in the opt-in `native-smp` profile; K06 remains open.

**Alternatives considered.**

- One shared TSS: impossible, because a busy TSS may be loaded by only one CPU.
- Broadcast IPI shorthands: rejected because they also reach CPUs owned by
  firmware (for example, after a capacity limit).
- Disconnect only an unresponsive AP instead of resetting all APs: rejected
  because an AP that does not accept an IPI cannot later participate in shootdown.
- x2APIC/MSR IPI: deferred; the profile uses xAPIC MMIO.

**Primary sources.** Intel SDM Vol. 3A: task management (64-bit TSS, IST
and the busy flag), interrupt and exception handling (IST stack switch,
interrupt gates) and APIC (ICR, delivery modes, LVT, TPR/PPR, spurious vector,
EOI). SDM chapter numbers differ by edition; search by section title.

## ADR-070 — Bounded AP startup as the first K06 step

5 November 2026. The kernel starts application processors (APs) exclusively
in the opt-in M2 profile `native-smp`, with IF=0. This profile excludes M3,
framebuffer, native-rtc and graphics-g0 at compile time. All existing
single-CPU invariants (ADR-024 scratch slot, frame allocator, paging, serial,
IPC, scheduler) remain valid unchanged because APs never execute that code.
The BSP is the sole owner of frames, paging, PIT, ICR and serial output.

**Design.** The MADT is untrusted firmware input. Only enabled, unique
processors count (maximum 32). A broken, duplicate or oversized MADT
invalidates only SMP, not the MCFG/DMAR inventory.
IA32_APIC_BASE must indicate xAPIC mode, enabled and BSP, and equal the
MADT base. Only that exact page receives a UC/NX mapping, which `audit`
checks separately, like the framebuffer window.
The SIPI page is the first free ConventionalMemory frame below 0xA0000:
permanently owned by Kernel, written once before the CR3 switch and never
mapped in its own page tables. It contains only 16-bit code and a CR3 field.
The 32/64-bit step, trampoline GDT and 16 KiB AP stacks with guard pages are
in the identity-mapped kernel image. W^X therefore does not change.

**Concurrency.** An AP selects its context by hardware APIC ID and claims
it with one `lock cmpxchg` STARTING→CLAIMED. On timeout, the BSP takes the
same CAS STARTING→ABANDONED and resets the CPU with INIT. A late AP therefore
never runs on an abandoned stack. Every error stops further startup without
panic: the system continues with the CPUs already started. If a failed AP
cannot be reset with INIT, boot stops with an error: an unmanaged CPU never
runs alongside the kernel. A CPU exception on an AP reaches the shared M2
handlers and terminates the VM. A MADT read error likewise invalidates only
the MADT. An AP copies CR0/CR3/CR4/EFER/PAT from the BSP exactly and checks
them. It then performs a bounded atomic-plus-spinlock test and parks with
`cli; hlt`.

**Evidence boundary.** QEMU q35/OVMF/TCG with 1–9 vCPUs plus negative cases;
see `docs/verification/k06-smp-bringup/`. This is not SMP scheduling,
IPI/TLB shootdown or per-CPU TSS/IST: an NMI or #DF on an AP stops the VM.
Physical hardware has not been tested and K06 is not accepted.
**Revise** before an AP executes kernel subsystems, user processes or
interrupts. Per-CPU state, TSS/IST, lock order, TLB shootdown and revision
of ADR-024 then become mandatory.

**Status:** accepted as an opt-in bring-up profile (`native-smp`), not as a
product profile; K06 remains open.

**Alternatives considered.**

- x2APIC mode: deferred. xAPIC MMIO covers the q35 profile, and firmware
  in x2APIC mode falls back to one CPU with fail-closed behaviour.
- A trampoline entirely in the low page (including 32/64-bit): rejected.
  That page would need an executable mapping in the kernel tables, whereas
  the current form never maps it (W^X unchanged).
- Taking the LAPIC base from the MADT: rejected. The base comes exclusively
  from IA32_APIC_BASE; the MADT value is only compared (untrusted firmware input).
- A broadcast INIT/SIPI (“all excluding self”): rejected. It also reaches
  CPUs outside the MADT or beyond capacity and provides no per-CPU timeout
  or quarantine.

**Primary sources.**

- Intel SDM Vol. 3A, §9.4 (MP initialisation protocol, INIT-SIPI-SIPI and the
  associated waits) and §11.6 (ICR, delivery modes, SIPI vector as a page number
  below 1 MiB).
- Intel SDM Vol. 3A, §11.4 (IA32_APIC_BASE: BSP, EN and EXTD flags).
- ACPI 6.5, §5.2.12 (MADT, structure types and lengths, Enabled and Online
  Capable, where Online Capable is reserved-zero when Enabled is set)
  and §5.2.12.8 (Local APIC Address Override).
- Locations: *Intel® 64 and IA-32 Architectures Software Developer's
  Manual*, Volume 3A (Intel Corporation; current edition,
  https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html),
  for INIT-SIPI-SIPI, ICR and IA32_APIC_BASE (xAPIC/x2APIC flags),
  MTRR/PAT and XCR0. *Advanced Configuration and Power Interface (ACPI)
  Specification*, version 6.5 (UEFI Forum, August 2022,
  https://uefi.org/specs/ACPI/6.5/), §5.2.12 for the MADT. *UEFI
  Specification* 2.10 (UEFI Forum, https://uefi.org/specifications) for the
  memory model at handoff. The identity map below 1 MiB is verified OVMF
  behaviour, not a UEFI guarantee; it is therefore listed as an open point
  in the evidence boundary.
- Local OSDev snapshot W003259 (MADT), solely as a route map.
- SDM chapter numbers follow recent editions. In older editions, the same
  sections are §8.4 (MP initialisation) and §10.4/§10.6 (APIC);
  search by section title if in doubt.

**Executed test evidence:** `docs/verification/k06-smp-bringup/README.md`.
## ADR-SESSION-20261102 — familiar controls and explicit VM session boundary

The current project assignment requires familiar session, search, status and
window controls around Materiaal. An independent product identity does not
prohibit familiar conventions. The new `native-session-route` remains a
public, isolated VM trial: personal credentials are not collected while a
real auth/session/keyring service is absent. Lock covers the screen and
blocks app input, but must not be presented as revocation of keys or storage
capabilities. FR-S02 and U02 remain open; the existing meter remains 41.

Shell policy remains in the userspace compositor. An input offer is
acknowledged before a system action. App queues and clipboard offers are
cancelled when the screen is covered; apps cannot draw the trusted foreground.
The desktop becomes visible only after both complete SUR frames. Diagnostic
readiness is published last, after a preceding 0, so timer preemption cannot
allow a new page to be observed with old readiness.

SUR1 retains a complete first frame and actor/epoch/bounds/COMMIT/deadline
checks; later sparse frames use strictly increasing tiles and a mandatory
final tile. Rebind invalidates the cache. An opaque rendering path and the
union of old/new panels limit work, with exact raster comparison against
the retained reference path. No new kernel ABI, crypto or device rights.

The tested code source is 703cc07; separate route and regression profiles
pass. Research, source hashes, screenshots, measurement boundaries and open
work are recorded in [the execution report](verification/session-quality-20261102/README.md).
VMware remains a separate platform trial according to
[the concrete handoff](implementation/VMWARE-WORKSTATION-TRIAL.md).
Revise for actual account/session rights, resize/rebind, GPU transport or
a changed trusted-input boundary; a host image does not replace native evidence.

<a id="adr-057--vreemde-opslag-nooit-als-leeg-volume-toelaten"></a>
## ADR-064 — Never admit foreign storage as an empty volume

3 November 2026; correction to recovery of the experimental NOSVOL02 volume.
The earlier recovery route checked only the table sectors when both table
copies were empty. An unrecognised volume with data outside that region could
therefore be treated as generation 0. Formatting or writing behaviour must
not be inferred from lack of recognition.

An unrecognised volume is empty only when the entire current volume extent
contains zeroes. For a small first write result, the service records and
flushes a version/geometry-bound `NOSFMT02` marker before content or table
writes. This keeps uncommitted first uploads recoverable as a logically empty
state without accepting foreign non-zero data. A marker provides format
identification and error detection, not authentication. Existing valid
NOSVOL02 table copies remain the primary recovery source.

The first marker write and flush are a separate initialisation boundary.
If a crash occurs before a valid durable marker, an unrecognisable non-zero
remainder must not be treated as own storage: recovery stops with `Corrupt`,
without writes or silent erasure. Only all-zero contents or a complete valid
marker yields generation 0 here. The before/after invariant for document
transactions begins after durable format identification and remains fully
applicable; separate crash trials check every byte prefix of the first marker
and the first data-write point. Explicit recovery/format controls remain
separately open.

The kernel remains a generic, bounded block broker; recognition and admission
remain in `volume-core`. No file-system driver, import path or automatic
formatter is added. An unmarked volume with non-zero data returns `Corrupt`
and receives no write/flush request. A synthetic Btrfs-like fixture tests
the zero-table/non-zero-data boundary; it is not a complete Btrfs file system.

Revise when the volume geometry changes or explicit format/import controls
are introduced. Reverting to table-only checking is not permitted. The marker
may be replaced when transitioning to a different persistent geometry,
provided full media admission and crash trials remain intact. The native QEMU
result and exact source binding are recorded in
`docs/verification/foreign-volume-admission-20261103/README.md`.

**Design rule:** without a valid NOSVOL02 table or complete `NOSFMT02` marker,
only all-zero contents across the current volume extent `[0,580)` are admissible.

**Implementation status:** implemented in `services/volume/src/table.rs`
(`choose` and `recover_with_state`) and the independent host oracle
`tools/volume_disk.py`.

**Test evidence:** host regressions and crash boundaries are recorded in
`docs/verification/pr148-slot1-review.json`,
`docs/verification/pr148-marker-crash-review.json` and
`docs/verification/pr148-volume-core-host-suite.json`.

**Unknown / open:** native metadata-only fixture acceptance and native QEMU
volume admission on this branch.

**Primary source metadata:** Nieuw OS repository (`simonvanieperen/nieuw-os`),
branch `simonvanieperen-upgraded-disco`, commit
`c6b25873c915a98f8c0b2bb7400395e01ba44741`,
https://github.com/simonvanieperen/nieuw-os/tree/c6b25873c915a98f8c0b2bb7400395e01ba44741 .

## Number preservation during desktop integration — 2 November 2026

The desktop branches used ADR numbers that were already taken. Existing IPC
and app decisions retain their numbers; the combined UI decisions receive
unique numbers. Old full title anchors remain as aliases.
Historical evidence files retain their original number references.

| Old number / subject | Unique number |
|---|---|
| 053, native window policy | ADR-059 |
| 054, temporary authority comparison | ADR-060 |
| 055, bounded GUI idle | ADR-061 |
| 056, single-use clipboard | ADR-062 |
| 056, familiar VM session route | ADR-063 |
| 058, positive volume identity (historical title) | ADR-064 |
| 057, never admit foreign storage as an empty volume | ADR-064 |

## Research decision J-20261102 — familiar foundation, explicit installation boundary

**Date:** 2 November 2026. **Status:** design/research choice, not implemented
architecture or native acceptance. Following the
[full OS journey comparison](research/2026-11-02/journey/README.md),
the preferred direction is one core interface with predictable basic actions,
local search aliases and optional guidance based on the user's previous
platform. No four shells, previous platform as authorisation or unrestricted
AI execution. The proposed installer shows one complete change plan before
every storage mutation, including Advanced; an unknown disk is not empty,
and a changed target invalidates authorisation.

**Reason:** documented similarities help transfer knowledge, while app/window
semantics, search scope, installer phases and early write boundaries actually
differ. This does not demonstrate an exact nieuw-os sequence or better usability.
**Consequence:** separate IN/ON/UX criteria and service/security gates before
execution; P2, kernel choice, privacy contract and fixed score criteria remain
intact. **Revise:** on task observation, current source/hardware differences
or a demonstrable technical security boundary. Details and bounded next steps:
[PRODUCT-SPEC.md](implementation/PRODUCT-SPEC.md).

<a id="adr-056--herkenbare-sessieroute-met-expliciete-vm-bewijsgrens"></a>
## ADR-063 — familiar session route with explicit VM evidence boundary

2 November 2026. The session route must follow the familiar steps from
macOS, Windows and Linux within the project's own architecture. Account selection,
credential entry, verification, session activation and desktop become separate
states. An app-local UI or password comparison in the renderer grants no
kernel rights. Only a future authenticated auth/session broker may release
keys/grants and enforce lock/revoke. Cancellation, switching and locking
invalidate old broker responses; both authentication and activation must succeed.

The new opt-in profile `native-system-session-preview` therefore explicitly
uses a public VM guest with exclusively synthetic test storage. The form for
a personal user is an interaction trial; the default broker rejects every
password and every hardware method not yet implemented. There will be no
shared demo password and no covert auto-login for real accounts. Existing
native Materiaal probes retain their own profiles. Taskbar and application
search grant no additional storage/network rights.

Windows may probe WHPX through a paused probe without disks/devices/network.
Auto reports TCG fallback; strict Whpx fails if the backend is missing.
This changes no host settings or security mitigations. CPU acceleration is
not GPU passthrough or physical-driver acceptance. Revise for real account
persistence, auth IPC, compositor lock, USB/CTAP or changed rendering transport.
Sources, contract and open gates: [session research](implementation/SESSION-AND-DESKTOP.md)
and [hardware research](implementation/VM-HARDWARE-AND-PERFORMANCE.md).


## OP-001 — Writers, integrator handoff and Windows delivery route

30 October 2026; record of the operational decisions from PR #75.
This describes the roles and handoffs recorded at the time, not proven
identity of current sessions. Current explicit instructions from the project
owner take precedence over routine arrangements between agents. The current
assignment to resolve issues and pull requests and squash-merge suitable work
provides the integration mandate; current source checks, repository rules
and relevant gates remain applicable. This decision announces no new merge
pause or release of a pause.

- **Queue ownership:** the recorded handoff from manager `89f84098` to
  `5d5f8513` makes the latter the sole writer of `docs/WORK-QUEUE.md`;
  central status and score remained with the manager. Consequence: one
  register for task and file claims, preservation of existing assignments
  and parked work; silence or a timeout does not release a claim.
  Revise on an explicit new assignment or demonstrably overlapping writers.
  The original ownership statements remain historical reporting. [OP1]
- **Integrator handoff e8→7f:** the queue records explicit donor stop and
  release, no ongoing/submitted mutation, receipt of the full dossier with
  claims and WIP, and an operational check by the recipient. Consequence:
  one operational integrator within the transferred scope; recheck current
  head/base/gates for every integration mutation and combine existing work
  while preserving content. An account name or elapsed time does not prove
  session identity or handoff. Revise on a new mandate from the project owner,
  a demonstrable second writer or changed substantive scope. The separate
  incident/pause record in the source is retained here as historical status,
  not reissued. [OP1] [OP2]
- **Windows route 12→21:** the mutually confirmed handoff concerns only
  `tools/package_windows_qemu.py`,
  `tools/windows-qemu/{Start-NieuwOS.ps1,Test-Launcher.ps1,README.md}`,
  `tools/tests/test_package_windows_qemu.py` and its own Windows evidence.
  Consequence: the existing route remains authoritative; no second launcher/
  packager or implicit kernel, CI or runtime claim. Route and synthetic test
  evidence do not provide a new OS bundle or guest acceptance. A development
  candidate must be bound exactly to source, manifest, artifacts, limitations
  and the slot agreement. Revise on an explicit change of route owner/file
  boundary or candidate/slot contract. [OP3]

**Primary sources:** Nieuw OS project, 30 October 2026.

- **[OP1]** *Operational work queue — mandate, source and continuity*,
  fixed PR #75 source `c26f57b60d4eaffbb57c0eb5114fa00bd7254eb6`:
  [work queue](https://github.com/simonvanieperen/nieuw-os/blob/c26f57b60d4eaffbb57c0eb5114fa00bd7254eb6/docs/WORK-QUEUE.md).
- **[OP2]** *Current project status — integrator handoff and extended
  handoff report*, the same fixed source:
  [project status](https://github.com/simonvanieperen/nieuw-os/blob/c26f57b60d4eaffbb57c0eb5114fa00bd7254eb6/docs/PROJECT-STATE.md).
- **[OP3]** S.W.J.B.M. van Ieperen, *Role 21: Windows/QEMU handoff and
  route evidence*, original handoff source
  `0915828fcd40492f873dc9098ee445306f1f0638`:
  [route report](https://github.com/simonvanieperen/nieuw-os/blob/0915828fcd40492f873dc9098ee445306f1f0638/docs/verification/windows-delivery-21/README.md).

## ADR-058 — Familiar consumer controls on the project's own kernel

**Date/status:** 30 October 2026; decision by the project owner, recorded
in PR #114. Takes precedence over earlier requirements that the main screen
must not imitate an existing desktop and that there will be no dock or
taskbar (PRODUCT-SPEC §1 and §5, the prohibition on reproducing interactions
in the PRODUCT-SPEC *P3/P4 addendum* for common layout and controls,
DESIGN-AMBITION, PRODUCT-AND-SOVEREIGNTY §2, AGENTS.md *Product ambition*).

**Decision:** consumers must not have to learn Nieuw OS from scratch. Setup,
sign-in, locking, user management, desktop, taskbar, start menu with search,
window controls, file management, settings and common keyboard shortcuts
follow the conventions shared by Windows, macOS and common Linux desktops.
Where they differ, the design chooses and records one variant. Innovation
(“futuristic”) supplements the familiar path and never replaces it.

**Consequences:**

- The own kernel, microkernel/capability architecture, own code and Security
  Floor remain unchanged. Familiar controls grant no broader rights: an app
  still receives only explicitly granted capabilities, even if the interface
  resembles familiar platforms.
- Familiar sign-in does not change security: every user has their own key
  material (no shared volume key), credentials are entered only in the
  session service through the lock screen or Ctrl+Alt+Delete, and the PIN
  never protects keys on disk (PROFILES-AND-USERS §4).
- Materiaal remains the data, history and rights model. It is exposed through
  the Bestanden (Files) app, recent items in the start menu and permission
  dialogues; it is no longer the mandatory main screen. P2 remains historical
  product evidence, not a final layout to build.
- Independent visual identity: no copied icons, images, brand names or trade
  dress from other platforms. Layout and controls are familiar, not the
  visual branding.
- The features in FR-U01 remain mandatory, but are reached from the desktop
  and start menu. The integrator will update the matrix text in FULL-TEST-RELEASE
  in a subsequent change; this decision removes no row and closes no criterion.
  The score remains 34%.
- Details: [PROFILES-AND-USERS.md](implementation/PROFILES-AND-USERS.md)
  and [DESKTOP-SHELL.md](implementation/DESKTOP-SHELL.md).

**Revise** after user trials (U06) demonstrate that a convention works less
well for the target audience, or on a new explicit decision by the project
owner. Rollback: withdraw this ADR and make the earlier texts authoritative again.

# ADR-054 — Shared immutable artifact-store contract

**Date/status/owner:** 30 October 2026; contract proposal from release
integration, coordinated by Apps and AI. Security/integration review and
native execution remain open. The first code implementation may only be a
pure contract crate with negative host tests.

**Context:** app-package verification and the model-store experiment both
need a large, immutable artifact with bounded reads. The existing document
volume is demonstrably unsuitable for this; document quotas are not promoted
to product-store capacity. Apps has a candidate descriptor and transaction
state machine; AI uses a separate reader trial with 4 KiB read-at. Message320
remains a separate wire boundary of at most 256 payload bytes.
[AS-IPC1] [AS-IPC2]

**Decision:** one provider-independent contract with an immutable descriptor,
opaque process-bound read handle and read-at of at most 4096 bytes. Stale
epoch, revocation, short/zero reads, EOF, out-of-range, overflow, corruption
and backend protocol errors are explicit. Capacity/quota are provider-reported
`u64` values with checked arithmetic. Writable install/publish is a separate
transaction: full reservation at `begin`, exact chunks, full digest/length
check before atomic publication, and explicit `Unknown`/cancel/outcome semantics.
Reservation is not a durability guarantee. SHA-256 is content-integrity metadata,
not publisher trust or model safety. Storage publication implies no dynamic launch.

**Rejected:** reusing the document volume (insufficient capacity and different
semantics); treating 4 KiB as a Message320 payload (exceeds the wire limit);
independently defining AI and Apps storage ABIs (two incompatible authorities);
using a fixed model/app trial size as a product limit; using a caller `verified`
boolean or shared B02/B03 authority; hiding `Unknown` as failure, success or
automatic retry.

**Consequences/revision:** contract document
`implementation/ARTIFACT-STORE-CONTRACT.md` is authoritative for the shared
interface after coordination with the Apps/AI owners. The descriptor layout
is not yet a shared serialised ABI. Host contract tests can check only
arithmetic/state-machine invariants. A native backend later requires its own
fault injection, flush/durability definition, cold recovery, process/capability
binding and separate publisher and model trust decisions. Revise before wire
encoding, kernel/SDK integration, persistent store, model download or dynamic launch.

**Primary sources for the IPC boundary:**

- **[AS-IPC1]** Nieuw OS project. (2026, 26 October). *Executable kernel and security contract M2–M4*, Message format and transaction section (source version `7f4f7bfe28fe9b2314bc4e4192e43942838d5dcb`). [Kernel and security contract](https://github.com/simonvanieperen/nieuw-os/blob/7f4f7bfe28fe9b2314bc4e4192e43942838d5dcb/docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md).
- **[AS-IPC2]** Nieuw OS project. (n.d.). *SDK Message320 envelope: MESSAGE_SIZE, MAX_PAYLOAD and length validation* (source version `7f4f7bfe28fe9b2314bc4e4192e43942838d5dcb`). [SDK implementation](https://github.com/simonvanieperen/nieuw-os/blob/7f4f7bfe28fe9b2314bc4e4192e43942838d5dcb/sdk/runtime/src/message.rs).

## ADR-055 — Canonical app package without implicit rights

Renumbered during integration from branch-local ADR-052 in PR #73
(`dce3b05`); ADR-052 on the CRB aperture trial retains its existing ID.

30 October 2026; A02 verification core, on top of `origin/main` after PR #67.
In version 1, a package contains one fixed manifest and exactly one ELF image.
Metadata is signed with both algorithms already used in ADR-049 (Ed25519
and ML-DSA-65); the caller binds verification to anchors of the selected
source. The signature demonstrates origin under that key, not trustworthiness.
App ID, ABI interval, version and image digest are metadata; scripts,
dependencies and capabilities are absent. Installation therefore grants
no general rights.

Package verification is an independent `no_std` crate and rejects
non-canonical padding, unknown format, ABI mismatch, unknown anchors,
either invalid signature, trailing package bytes, inconsistent `image_length`
framing and digest errors. `verify_package` does not inspect ELF structure.
Only `verify_executable_package` additionally runs `kernel_core::user::parse`
and rejects images that do not satisfy the existing user-ELF policy.
The package format reuses no kernel-update target or update policy.
Alternative: extending the kernel-update manifest code would mix application
semantics with the boot/slot trust boundary; the existing verifier accepts
only `TARGET_KERNEL`. This decision introduces no app installation or new capabilities.

The executable entry point additionally calls existing
`kernel_core::user::parse` policy instead of introducing a second user-ELF
policy. The resulting load description is preflight, not frame allocation,
process startup or `LaunchImage` authorisation. The installer/launch boundary
remains a separate open step.

The 2 MiB image boundary is a temporary parser/fixture boundary, not a product
limit. ML-DSA stack requirements must be supported by service/stack quotas
before native integration. Revise before installer integration: source and
key rotation, ABI policy, storage/update transactions, app-image loader and
recovery behaviour. Rollback can remove the new crate/workspace entry;
B03 and the existing app-launch ABI remain unchanged. No A02 point awarded.

## ADR-056 — Source revocation blocks future packages

Renumbered during integration from branch-local ADR-053 in PR #73
(`dce3b05`); ADR-053 on IPC waiting retains its existing ID.

30 October 2026; source-management policy core for A02.
The verification core receives an explicit `SourceId` and expected source
generation for every verification; the app bundle never chooses its own
trust anchor. Key rotation and revocation are compare-and-swap operations
on that generation. Revocation blocks new package verifications/updates but
does not remove or block already installed apps; this behaviour was explicitly
determined by the project owner. Removal of apps/data is a separate user action.

`SourceRegistry` is currently a fixed in-memory core of eight slots.
User consent, capability enforcement, durable source status, crash recovery
and integration with an installer remain required. The eight slots are not
a product limit and provide no A02 acceptance evidence. Rollback can remove
the registry and its tests without changing the package format or kernel ABI.

## ADR-B03K1 — Shared, pinned key authority per release window

30 October 2026; B03-KEY-POLICY-1, basis `0ade8014`. Loader and update service
use one `update-core::policy::TrustPolicy` with at most eight exact hybrid
key pairs, inclusive release-version windows and unconditional revocation.
Invalid or duplicate table entries confer no authority. The signed version
is checked against the window only after both signature checks.
No new crypto algorithm, capability or dependency.

The alternative of calendar expiry was not chosen: the RTC provides no trusted
calendar time. Nor may a manifest supply its own trusted keys or revocation
policy. The QEMU policy table retains the existing TEST pair and separately
pins next/revoked/expired fixtures; these public keys are not production
anchors. Pre-pinned rotation does not demonstrate secure distribution or
rollback protection for new policy versions.

Revision criteria: reassess this decision when a production policy for key
distribution or rollback protection of the policy table is established, or
an authorised trusted calendar-time source becomes available. Then reassess
anchor distribution, policy-version/revocation boundary and the choice between
release-version windows and any calendar policy; these triggers do not
automatically change the current policy.

Contract, negative cases, open production/time boundaries and rollback:
`implementation/B03-SIGNED-UPDATES.md`, B03-KEY-POLICY-1. Withdrawal is a
targeted revert to the earlier single TEST verification path, without a
slot/manifest-format migration; withdrawal does remove lifecycle enforcement.
Native result and source binding are recorded separately in PROJECT-STATE;
B04, B05, B06 and the additional gates of #41 remain open.

## ADR-B03K2 — TEST anchors only through explicit test features

3 November 2026; basis `d686bd54`. `update-core::test_policy` is no longer
part of the default library build and requires the `test-signing-anchors`
feature. The native fixture service and UEFI fixture loader enable this only
explicitly; the loader profile is named `signed-boot-test`. The bare
`signed-boot` feature and a native update-service build without a policy choice
fail at compile time instead of silently using the public TEST anchors.

This is a fail-closed build boundary, not a production key lifecycle. No
production anchors, release configuration, signing ceremony or revocation
distribution were added. #41 therefore remains open; no new rubric point
or release claim. Rollback restores the unconditional `test_policy` module
and old profile name but removes this build-time protection.

Revision criteria: revise this test-feature boundary when production policy
for key distribution, release configuration or distribution of revocations
is introduced. TEST anchors do not silently become production policy.

## ADR-053 — IPC waiting preserves existing rights and receives ABI 1.10

Renumbered during main integration from branch-local ADR-051;
main's ADR-051 on key state below retains its existing ID.

30 October 2026; integration `NATIVE-TEST-VERSION`, basis `ad2b981`.
Existing component `IPC-WAIT-1` is incorporated for the failing native
Materiaal user chain. Sixteen copied handle/interest pairs with an absolute
PIT deadline may park a process; readiness neither reserves nor consumes
anything. Every reassessment rechecks the complete capability and PID.
No user pointer, copy plan or object reference survives a context switch.
No new rights are granted.

The component branch still used ABI 1.9, but main has since assigned that
to the explicitly untrusted RTC clock. Therefore only `ipc-wait` uses ABI
1.10; the RTC syscall and old profiles retain their behaviour. Integration
also preserves the later stack-range check on user return. An empty runnable
queue plus waiters may execute `sti; hlt; cli` through the existing permanent
service stack; only actually counted IRQ0s advance the monotonic clock.
All Engine digest checks remain in place. Consumers may park only without
locally runnable work. The legacy keyring loop therefore does not wait when
RANDOM ingress exists; the volume adapter waits only after its synchronous
disk and reply work. Desktop-init waits for registry/controller only in a
quiet Running phase; the separate test controller continues polling its
local kernel trigger.

First, a separate native two-process trial is built, including actual queue,
revocation, deadline, pointer and idle checks. The existing GUI gate follows
without extending its deadline. Host tests or a built ELF do not demonstrate
native progress, usability or a full release. Revise before SMP or new IRQ
wake sources: the current invariant applies only to one CPU and serialised
IF0 kernel mutations. Rollback disables the opt-in feature and consumers;
ABI 1.10 remains a reserved interface version.

## ADR-051 — Persistent key state through a separate administrative ingress

30 October 2026; implementation B02-PERSIST-1, basis `ad2b981`. The existing
specification is [B02 — authenticated records and exact key state](implementation/B02-SECURE-RECORDS.md),
dated 29 October 2026 and supplemented on 30 October 2026 (FR-B02P). The
[native profile report](verification/full-release-b02-review-followup/system-gate.json)
and [final manifest](verification/full-release-b02-review-followup/final-manifest.json)
record one `native-system` QEMU normal/recover run; this is not evidence for
cold rollback anchoring, production use or full B02 acceptance. The native
sender-control test was not performed and remains open.
B02 codec `export_persistent`/`restore_persistent` receives KEY1 opcodes 11/12
(`EXPORT_PERSISTENT`, `RESTORE_PERSISTENT`). They are accessible only through
a third, separately granted kernel endpoint of the Key Service, held only by
the supervisor/recovery agent (in the FR-I01 profile: init). Domain endpoints
and RANDOM ingress reject these opcodes before cryptographic processing;
conversely, administrative ingress rejects every key, record, recovery and
RANDOM operation. The external 32-byte secret travels in the request and is
not retained by the service; there is no second service-adapter path for
“admin”, only `wire::dispatch_persistent`.

The integrated profile defines the order: the app first seals in RAM,
reports `KEYS_READY`, init exports and commits the 120-byte blob through
the crash-consistent volume transaction in volume domain 1, and only after
`PERSISTED` does the app commit the ciphertext record as an ordinary document.
In this profile, ciphertext therefore does not reach disk under a key allocation
that has not been durably recorded. That order is guaranteed by the init/app
protocol, not by the volume or Key Service: an uncooperative app can still
commit earlier. Enforcement belongs to the still-open write-ahead mechanism
in the Key Service. Specification:
[B02-SECURE-RECORDS.md](implementation/B02-SECURE-RECORDS.md);
partial native evidence (QEMU, defined profiles, no physical hardware):
`docs/verification/full-release-b02-keystate-*` and
[full-release-b02-review-followup](verification/full-release-b02-review-followup/).
On cold restart using the unchanged read-only image, init restores the exact
state before any key work; an incorrect secret, a higher generation floor
and a key opcode on administrative ingress are rejected and counted.

Rejected alternatives: (a) grant block/volume rights to the Key Service itself —
expands the key service TCB and mixes storage recovery with key management;
(b) persistent opcodes on existing domain endpoints — every app with a key
endpoint would then receive snapshot authority.
Init's recovery secret in this profile is a compile-time test constant in
the init ELF of the tested disposable image; anyone with that image knows
the secret. This is exclusively an acceptance fixture, not a product path.
Authority: SEND to administrative ingress is equivalent to the entire key
namespace, because EXPORT returns the master encrypted under a secret chosen
by the holder; it must belong exclusively to the supervisor/recovery principal
and must never be delegated. For a cold, unused instance, the external secret
is also the only lineage authentication: anyone who knows the secret **and**
can write object 8 chooses the master for the next start, not merely an old
snapshot. A product recovery agent therefore needs independently anchored
lineage/generation checking.
Boundaries: the generation floor in this profile is a demonstrated mechanism,
not a trusted anchor; cold rollback anchoring, write-ahead for every GENERATE/
REVOKE within the service itself, and an actual volume identity (currently
the profile constant `NOS-SYSTEM-VOL01`) remain open. The SDK's own Message320
envelope/staging bytes have been wiped since the B02 IPC hygiene check;
caller-owned request arrays and compiler/register copies are not covered.
Since INT3-FU-116, the manifest checks that the app has no SEND to administrative
ingress across the entire app handle table at the manifest moment:
object comparison per handle (`APP_ADMIN_HANDLE`) plus an exact count of
occupied app slots (`APP_HANDLE_COUNT`). Sender checking on the Key Service's
administrative ingress still has no executed native counterexample in this
profile (no other process holds a SEND); this is open evidence, not implicit
approval. B02 therefore remains open; the score does not change. Rollback:
remove the two opcodes, administrative ingress and init/app steps; existing
KEY1 codes, formats and the B01 profile are unchanged.

## ADR-050 — Opt-in wall-clock syscall remains visibly untrusted

30 October 2026; implementation NATIVE-CLOCK-1, on top of RTC-NATIVE-1.
`native-system-clock` adds ABI 1.9 syscall `0x73` for a direct,
read-only UTC snapshot from CMOS. The user SDK returns an explicit
`UnauthenticatedRtc` type. The clock is public process metadata; apps
receive no CMOS ports, capabilities, pointers, storage or network rights.
Invalid non-zero arguments return `-22`, failed or unusable hardware returns
`-5`, without fallback or invented time.

This scope chooses a working, testable system function in the existing
single-CPU IF0 profile, not a trusted production TimeProvider. The adapter
verifies native RTC invariants and index restoration on every call.
Unix time and boot-monotonic PIT ticks remain separate. No certificate
validation, persistent anti-rollback, synchronisation, sleep/wake correction,
physical-hardware evidence or new progress points. The actual ring-3 app and
independent QEMU base-time oracle are the acceptance gate.

Revise before SMP, other firmware/hardware or TLS use: serialised device
ownership, an authorised TimeProvider, freshness, uncertainty margin and a
rollback anchor are then required. Rollback: remove only the opt-in feature,
syscall, SDK bindings and acceptance; ABI 1.9 remains as a documented
compatibility revision, without affecting old profiles.

## ADR-057 — Exclusive native CMOS bootstrap, no trusted time

Renumbered during integration from the duplicated ADR-049 in main
(`9a6703d`). ADR-049 on hybrid kernel updates retains its existing ID;
historical references to the CMOS bootstrap as ADR-049 belong to ADR-057.

30 October 2026; implementation RTC-NATIVE-1, basis `5c334f9`.
The existing dependency-free RTC codec is connected to the project's own kernel only
for opt-in `native-rtc` diagnostics. After installing its own IDT/paging,
on one CPU with IF=0, the trial exclusively owns CMOS index/data 0x70/0x71.
An explicit D/NMI-Enabled initial selection does not replace a firmware shadow;
the read procedure restores this own selection. No RTC data writes,
register-C ACK, app I/O, new syscall or implied production feature.
Ring-3/framebuffer combinations are rejected. The small privileged codec
serves hardware acceptance only; a production TimeProvider remains an
isolated service with a separate minimal broker/rights contract.

Alternative: immediately publishing a wall-clock syscall or TLS clock would
make unauthenticated time an implicit trust anchor. The output therefore
keeps `UnauthenticatedRtc`, caller floor and volatile backward-time checking
explicit. Five independent calendar oracles plus actual budget/floor rejection
and device-index restoration form the native gate; existing host tests remain
required. Exact execution: PROJECT-STATE and RTC-WALLTIME. No physical,
persistent anti-rollback, TLS or product evidence.
Primary emulator contract: QEMU `v10.2.1`, `hw/rtc/mc146818rtc.c`;
additional source OSDev CMOS W001717 (revision 30263). No emulator code adopted.
Rollback: revert the opt-in feature/adapter/runner; existing M2 and all
production services remain unchanged. Revise for IRQ/SMP or product use.

## ADR-031 — Native I/O uncertainty without observer panic

**Date/status/owner:** 28 October 2026; implemented and accepted on
source 88e6c04. Native builder A01, integration root, AI reviews R08/R09 and A04.

**Context and decision:** the service already supported Unknown, but the M6c
observer always required Ok for a prepared mutation. Bind the first uncertain
reply to a terminal mutating syscall error actually returned to the service;
Pending is not an error. Latch the I/O counts, commit no candidate, count no
ACK and check that neither app domain causes new I/O afterwards. Validation
and authorisation remain before the closed service state. Only full cold
recovery determines the version actually stored.

**Alternatives and consequences:** removing success assertions would mask
errors; automatic write replay can duplicate an uncertain mutation. The
chosen path keeps normal WRITE+FLUSH conditions strict and changes no
capability, syscall, wire/journal format, service implementation or external
dependency. Test injections use existing driver-snapshot, deadline and reset
paths. No automatic service restart; historical M6b observer retained separately.

**Evidence and revision:** the complete 35-group/31-VM gate and M6b regression
(18 groups/14 VMs) pass, with both captures and separate evidence audits.
30 Rust/48 Python tests for M6c, 22 Rust/30 Python tests for M6b; all exit 0.
Exact scope and source: `verification/M6D-ACCEPTANCE.md`; S07 yields 13% → 14%.
Contract and primary VIRTIO source:
`implementation/M6D-IO-UNCERTAINTY.md`. No physical device failure, power loss,
human audit or formal proof. Revise for general service supervision,
parallel mutations, new I/O error models or other hardware.

## ADR-032 — Fixed 100-point roadmap with mandatory reassessment

**Date/status/owner:** 28 October 2026; accepted and implemented on the
project owner's explicit instruction. The lead agent owns the shared progress files.

**Decision:** `docs/progress.json` contains fixed, binary acceptance criteria
with weights summing to 100. The generator produces the README, criteria
table and local SVG. 0% means not yet implemented; 100% means a fully fledged,
independent daily-use OS on demonstrably supported hardware. The own kernel
remains mandatory. The initial score for the M1–M6c evidence state is 13%.

**Alternatives/consequences:** commits, lines of code, test counts or M numbers
do not measure product completion. Host prototypes or plans alone therefore
earn no native points. Every assignment updates assessment, source and date;
a score may stay the same or decrease. Weights are an explicit planning choice,
not a scientific measurement or estimate of remaining time.

**Evidence and revision:** `tools/progress.py --check` checks the sum, unique
criteria, existing evidence references and equality of generated views. It
does not assess evidence content or run native tests. Changes to final scope/
weights require a rubric version, reason and comparison with the old score.
See `PROGRESS.md`; never lower the bar to obtain a higher number.

## ADR-030 — Native labels and a catalogue per app domain

**Date/status:** 28 October 2026; implemented and native final gate passed
on source commit bc4cd104, basis bb22fdb. Exact contract:
implementation/M6C-NAMED-CATALOG.md.
On this historical M6c execution source, the native `Unknown` error path has
a known, unrepaired observer panic, also with `milestone-m6c`. The separate
repair source and its separate acceptance are recorded above in ADR-031.

**Context and decision:** completed M6b has anonymous object slots. Materiaal
needs named objects and list/open. Add a separate dependency-free catalog-core
with explicit IPC v4/journal v2. Names are UTF-8 labels, not authority or paths;
object ID and endpoint/caller remain the access boundary. LIST returns one
bounded snapshot of only the app's own live objects, with its own revision sum.
PUT/RENAME are one record with name/content under one CAS revision. RENAME
preserves content; WRITE and FLUSH completion precede ACK. The service contract
also closes LIST on `Unknown`; this error path was tested only through the host ABI.

**Alternatives/consequences:** silently reusing v3/v1 padding would mislead
old readers; a separate name table requires multiple atomic writes. Automatic
migration is unnecessary for this synthetic trial: v1 rejects before READY
and the M6b reader is retained. Duplicate labels are allowed; no directory
tree, normalisation, encryption or physical secure erase was added. Smaller
limits are trial budgets, not fixed consumer-product choices.

**Evidence and revision:** the final M6c gate/capture passes with 19 groups
and 15 VM invocations: 14 M6c plus one ordinary M1. M6b regression passes
with 18 groups and 14 VM invocations on the same fingerprint. The 30 core tests,
33 harness tests and 15 host-ABI adapter trials retain their own scope. The
native observer in `kernel/src/m2/m3/m6c.rs` can panic on WRITE/FLUSH errors
or timeouts before the client receives `Unknown`; controlled native error
handling was not implemented on that source. A separate AI audit checks all
141 source files, 130 recorded artifact hashes and eight volumes, including
two external rename kills and six cold recoveries. Details:
`verification/M6C-ACCEPTANCE.md`; no human audit or formal proof.
The new kernel module contains an explicit fixture observer plus the existing
block broker; catalogue policy runs in ring 3. Repairing the known `Unknown`
defect requires a separate native fix with WRITE/FLUSH error and timeout
injection, checking the client response actually received and new source-bound
native verification. Revise format/migration before real user data, more slots,
larger documents or compositor integration. Rollback can use the existing M6b
build and its own volumes; do not overwrite an existing image.

**Sources:** current M6B-OBJECT-STORAGE.md/PROJECT-STATE.md on bb22fdb;
Rust bare-metal target documentation in the M6c contract. This is a project decision.

## ADR-028 — Evidence per property and standard hybrid NAS transport

**Date/status:** 27 October 2026; design accepted and N02 host trial tested from `ece94a5`. **Context:** the user's blueprint calls for formal verification, a small TCB, capabilities, memory protection and PQC. The direction fits the project; the absolute product claims and custom KDF/tunnel proposal are not substantiated.

**Decision:** retain the project's own Rust kernel, develop formal properties with explicit assumptions and implementation binding, and treat CHERI as a separate platform trial. Asymmetric/PQC code remains outside `kernel/`. For the first executable NAS transport trial, use TLS 1.3 exclusively with the standardised X25519MLKEM768 group (RFC 10024), mutual certificate checking/pinning and existing peer-bound rights. No custom combiner/record crypto. The optional-PSK framing in the supplied fragment is ambiguous. A classical-only peer receives no fallback in this trial profile.

**Alternatives/consequences:** immediately adding an untested native crate does not fit the current direct-rustc build, fixed ELF limits, small IPC, missing entropy capability and disabled SIMD. Replacing the entire kernel with seL4 is not an assigned task. The bounded host trial supplies actual protocol evidence without claiming a native port. The trial uses classical Ed25519 authentication; no claim of full post-quantum authentication/boot chain. LAN/WAN labels in loopback tests are explicit harness assumptions.

**Invariant:** permission before connecting; actual peer authentication before data; current authority on every application write/response; close/error revokes the transport fact. The fact binds to app, peer-pin generation and, for internet, revocation generation; old authenticated connections do not revive after renewed permission. Revocation takes effect at the next checked action, with no proactive push to idle sockets. Client-only document keys and the existing administrator deletion right remain applicable. A validly signed package does not itself grant capabilities. Limited authority granted once may be used automatically without repeated prompts.

**Evidence/status:** integrated 124/124 tests (85 service, 39 UI), both demos exit 0, 20 syntax checks and diff check passed. Actual negative TLS handshakes, authenticated malicious server responses, resource boundaries, revocation and regrant tested. Two AI reviews and a targeted independent recheck of the generation correction. Commands/source hashes/results in `verification/pqc-transport/`. The initial 16-policy-test checkpoint is retained as history and not added to the 124. No formal proof or human audit.

**Sources:** RFC 10024; seL4 verified configurations/Proofcraft news 2026; CHERI FAQ/CHERI-D; NIST FIPS 203/204/205. Exact URLs, corrections and native tasks are recorded in `implementation/VERIFIABLE-SECURITY-AND-PQC.md`.

**Revision/rollback:** revise for incompatible standards, library/evidence problems, a native port or a new product instruction. N02 is a separate working branch above PR #7; removing it requires no disk, key or firmware migration. The new trial opens loopback only. Preserve existing N01/M6a code and evidence archives.

## ADR-027 — Internet and encrypted personal NAS space

**Date/status:** 27 October 2026; accepted on the project owner's explicit instruction, with tested N01 host trial and browser mock. Native integration remains open. **Owner:** lead agent for contract/integration; separate service and product builders.

**Context:** internet access, a NAS role within the same OS and offering storage space to friends and family are product requirements. The host must be able to reclaim its space without the guest's permission, while the content remains encrypted against the host. This revises ADR-006 and the earlier restriction to own NAS/own LAN.

**Alternatives:** retaining own-LAN-only operation does not satisfy the new assignment; a vendor cloud or administrator key for guest content violates the privacy boundary; requiring guest consent for deletion deprives the hardware owner of their administrative right.

**Decision/invariant:** client-only content keys; encryption of document name and content; authenticated namespace/object/revision. NAS administration manages quota, revocation and deletion but receives no decryption key. Revoke/evict invalidates old sessions and ongoing uploads; tombstones prevent revival. Internet is a separate app right; NAS internet is explicitly peer-bound, and local AI remains without network access. The local NAS administrator role is separate from an internet data connection; remote administration is not silently added. A route planner automatically chooses verified LAN or previously permitted direct internet to the same peer, without repeated consent questions or new authority. Privacy and ease of use are both required; temporary host-trial caps are not product choices.

**Implementation:** dependency-free Node host trial in services/nas/, without sockets or an actual NAS. Own synthetic ciphertext store with one writer and atomic snapshots. Browser flows Mijn NAS / Ruimte ontvangen / Ruimte aanbieden (My NAS / Receive space / Offer space) are mocks. These development tools do not become a mandatory runtime for the project's own OS.

**Test evidence:** combined suite 89/89 (50 service, 39 UI) and finite demo exit 0; source hashes/commands in verification/nas/report.json. Actual child-process kills after confirmed storage mutations, no physical power-cut test. Four concrete review findings were fixed: nonce budget during staging, temporary ciphertext snapshots during erasure, a race during writer-lock recovery and unwanted administrator authority through internet.

**Boundaries/consequences:** host identity and transport facts are synthetic; no TLS/native-network evidence. A recovered content key can only read, not safely continue writing on multiple devices. Browser internet in N01 is app-bound; destination grants for ordinary apps/downloaders are still to follow. No full snapshot rollback protection, safe physical erasure or plug-and-play NAT claim. Native client distribution/key lifecycle, network capabilities, administrator storage, onboarding and human audit remain separate integration requirements.

**Sources:** Node.js v24.22.1 Crypto (https://nodejs.org/docs/latest-v24.x/api/crypto.html), RFC 8446 (https://www.rfc-editor.org/rfc/rfc8446.html) and NIST SP 800-207 (https://doi.org/10.6028/NIST.SP.800-207), accessed 27 October 2026. The functional choice and administrator deletion right are project decisions. Exact contract: implementation/NAS-SHARING-AND-INTERNET.md and LOCAL-PRIVACY-CONTRACT.md revision 3.

**Migration/rollback/revision:** no existing real NAS, router, kernel ABI or M6a image is changed. Preserve older M1–M6a results. N01 can be reverted as a separate feature commit; no destructive reset of user work. Revise on evidence of key/nonce problems, multi-writer integration, another transport/storage layer or a new instruction from the project owner. Old LAN-only texts were updated on active reading routes.
The original revision-5 principles below are retained as project decisions. Implementation decisions from ADR-019 onwards describe the direction built; current M1–M5 test outcomes and boundaries are recorded in PROJECT-STATE.md.

| ID | Decision | Reason | Revise if |
|---|---|---|---|
| ADR-001 | Own kernel; no existing kernel as the basis | Explicit decision by the project owner | The project owner deliberately changes the project scope |
| ADR-002 | Rust no_std + minimal assembly | Small, explicit unsafe boundary | Target/tooling demonstrably prevents this |
| ADR-003 | x86_64 QEMU q35, UEFI, software-emulation baseline | Controllable first test environment | A practical toolchain limitation is demonstrated |
| ADR-004 | Small capability-based microkernel as the direction | Keep authority explicit and the TCB small | Prototypes demonstrate problems with complexity/latency |
| ADR-005 | Allow standards and vetted libraries | Building custom crypto/all tools increases risk and scope | Provenance/licence/maintenance is unacceptable |
| ADR-006, revised by ADR-027 | Local account, no OS telemetry, local data or E2EE to a linked own/shared personal NAS; internet explicit per peer | Explicit extension of the project assignment, 27 October 2026 | Only after an explicit change of assignment by the project owner |
| ADR-007 | Prepare for PQC from the design stage; integrate only with an actual use case | Prevent protocol and key migration | Relevant standards/implementations change |
| ADR-008, revised | Product/interaction research in parallel from M0; actual system integration follows proven core functions | Originality and high-quality UX must help guide the system | Adjust based on experiments, without silently reducing the goal to a standard desktop |
| ADR-009 | No kernel-wide claims of formal proof | Own code does not inherit seL4's proof | A bounded, actually verified result exists |
| ADR-010 | AI inference exclusively on the device; no network right or cloud fallback | Explicit correction of the project assignment | Only after an explicit change of assignment by the project owner |
| ADR-011 | Installable third-party apps with separate file, NAS and network rights | Accessible ecosystem without implicit trust | Acceptance tests reveal a missing boundary |
| ADR-012 | Quantum as a third research track from M0: PQC, local QPU execution and hybrid system port | Explicitly preserve the original ambition | Adjustment only with substantiated research; the privacy contract remains applicable |

## Selected implementation decisions — 26 October 2026

| ID | Decision | Evidence boundary / revision |
|---|---|---|
| ADR-013 | Own UEFI loader with r-efi 7.1.0; Rust 1.90.0; x86_64/QEMU TCG | Versions researched; combination not yet built. Exact boot contract in BOOT-AND-TOOLCHAIN.md |
| ADR-014 | First kernel: fixed ELF layout and shared BootInfo; then bounded processes, IPC and capabilities | Details and negative tests in KERNEL-AND-SECURITY-CONTRACT.md; no isolation evidence before execution |
| ADR-015 | Materiaal as the first product direction; P2 local HTML/CSS/JS trial | Comparison performed, no user trial. Revise based on concrete usage observations |
| ADR-016 | Q1 safe Rust state-vector core with qsim-v0, maximum 12 qubits, separate host adapter | Analytical fixtures checked; no Rust/QPU/OS-port evidence |
| ADR-017 | Later ML-KEM-768 and ML-DSA-65 through reviewed implementations/protocol binding | No custom cryptographic primitives, no full boot-chain claim |
| ADR-018 | EU-01 through EU-08 as technical acceptance requirements; legal applicability per release | EUROPE-AND-RELEASE.md contains a conditional matrix, not a compliance claim |

## Template for a new decision

ID, date, status, context, alternatives, decision, rationale/sources, consequences, security invariant, test evidence, migration/rollback and owner. State what would refute the choice; avoid decisions without testable assumptions.

## Implementation decisions — 27 October 2026

**ADR-019 — Windows development package and read-only virtio ESP.** Rust 1.90.0/MSVC uses the existing Visual Studio 2026/SDK. QEMU 11.1.0 and the matching CODE/VARS pair come from the same recorded Windows package; hashes are recorded in docs/verification/toolchain-provenance.json. This is the permitted development variant, not the byte-identical reference build. The original IDE drive produced `Block node is read-only` before loader execution; the ESP therefore uses `if=virtio,readonly=on`. Every run receives its own ESP/VARS copy, without network or host disk. Actual cold boots and failure trials are recorded in docs/verification/boot-runs.json. Revise for other firmware/platforms; removing the separate tool directory restores the host without a global PATH change.

**ADR-020 — Fixed kernel span 32–48 MiB and BootInfo 1.1.** In the actual OVMF map, the original 2–18 MiB interval overlaps ACPI NVS and BootServicesData. AllocatePages rejected the reservation with EFI_NOT_FOUND; diagnosis in local build/run/smoke-fe68c261dd/result.json. The profile now uses `[0x02000000,0x03000000)`, the same 16 MiB. Loader, parser, linker and fixtures were changed together; the header is still 128 bytes with the same offsets, but explicitly requires ABI 1.1. ABI 1.0 is rejected. No occupied firmware memory is reused. The full acceptance sequence with these values passes. Rollback requires jointly adjusting both binaries/validator/linker and a demonstrably suitable firmware map; never introduce an address fallback alone.

**ADR-021 — Static GOT in .data.** The compiler produces five statically resolved pointers. They are explicitly placed in the existing RW .data output section; no runtime relocation handler or extra segment is added. Build inspection continues to reject dynamic metadata, relocations and unknown allocatable sections. The three ELF load segments and hashes are recorded in docs/verification/boot-manifest.json. M1 checks segment intent; actual W^X enforcement remains M2.

**ADR-022 — Progress to GitHub at every step.** The project must be fully resumable when usage limits are reached. Source code, lockfiles, compact evidence reports and open points therefore go to the private working branch after every bounded step and before long checks. A checkpoint may contain unfinished checks if explicitly marked. Codespaces is not required for this preservation routine; no Codespace was started. Main remains the starting specification until the draft PR is reviewed.

**ADR-023 — Only WB-capable ordinary ConventionalMemory, combined cache bits.** The first M2 run failed before KS-02 because OVMF reports `0x0f` and the allocator accepted only exact WB. UEFI attributes describe capabilities; UC/WC/WT/WB may occur together. The allocator requires WB, permits only these four cache bits and rejects other/special attributes. Source: TianoCore/EDK II, `edk2-stable202508`, [UefiSpec.h, EFI_MEMORY_DESCRIPTOR and cache bits](https://github.com/tianocore/edk2/blob/edk2-stable202508/MdePkg/Include/Uefi/UefiSpec.h), checked 27 October 2026. This does not identify the current package as exactly that firmware version. The regression tests both `0x0f` and rejected special/unknown bits; the actual M2 gate passes. Other platform profiles require reassessment of memory attributes; no silent reclaim of Loader/BootServices/runtime/ACPI memory.

**ADR-024 — Static 2048-mapping arena and one temporary scratch slot.** Every process context has the contractual capacity of 2048 mappings; large metadata lives statically outside the 16 KiB kernel stack. PID is a monotonic u64; internal ASID is separately bounded. Allocator zeroing/table access uses a temporary supervisor/NX slot, with invalidation before use and after clear. A writable scratch alias to an executable physical frame is rejected; read-only input has a separate RO/NX helper. No arbitrary identity map of free RAM/holes/MMIO. Evidence: kernel source ed09818, full M2/M3 gate, actual 67-user-page/BSS-zero regression and per-process cleanup balance. Revise for SMP/DMA/new mapping API; serialisation applies only in the current single-CPU/single-thread profile. **Revised by ADR-075** (KERNEL lock and scratch slot per CPU; revision note ported on 13 November 2026 from PR #246, head `273b2eb`). Restore through a consistent earlier source version in a separate checkout, without erasing current changes.

**ADR-025 — Safe fixed capability engine with a separate user-copy boundary.** `capability-core` is own dependency-free no_std code without unsafe/heap; the kernel guards user pointers, syscall order and 320-byte copies. Engine receive commits only after full output validation, followed by a byte copy that is infallible in this profile. Root authority comes exclusively from the static test manifest, not from a general userspace object lookup. Revoke revokes descendants; exit also revokes owned/issued authority. Closed object tombstones persist until the last reference. Host tests compare the full Engine state; kernel checksums are diagnostics. Evidence: 21 core tests, all eight actual M4 cases and independent targeted review. Revise before threads/shared memory/DMA/dynamic application or object types; do not extrapolate to such guarantees. No boot-ABI change or host-configuration migration.

**ADR-026 — M5: isolated READ service with trusted virtio transport.** The first system service runs as its own Rust program in ring 3. The kernel guards the typed READ capability, tickets, user copy, PCI/MMIO and a fixed five-page DMA pool. The device at `00:05.0` uses one split queue of eight slots and at most one concurrent sector read. The service cannot choose physical addresses, descriptors, PCI configuration or MMIO. The protocol and 17 failure cases are fixed before execution in [M5-BLOCK-SERVICE.md](implementation/M5-BLOCK-SERVICE.md). Only local crates from the project were added; no new external dependency.

**Status:** accepted and implemented for the fixed M5 QEMU profile. **Alternatives:** all service logic in the kernel (larger trusted base), or direct userspace DMA (requires additional isolation design, outside M5). **Owner:** lead agent for integration and decision management; `kernel_m2` for transport and mapping boundary.

**Rationale/sources:**

- OASIS Open, *[Virtual I/O Device (VIRTIO) Version 1.2](https://docs.oasis-open.org/virtio/virtio/v1.2/cs01/virtio-v1.2-cs01.html)*, Committee Specification 01, 1 July 2022: §2.4 (reset acknowledgement), §2.7 and §2.7.8 (split queue and used entry without epoch), §4.1 (PCI transport) and §5.2 (block device). Queue size, one concurrent read and the pinned pool are project choices.
- Intel Corporation, *[Intel® 64 and IA-32 Architectures Software Developer’s Manual, Volume 3A: System Programming Guide, Part 1](https://www.intel.sg/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.pdf)*, Order No. 253668-060US, September 2016: §4.1.4 and §4.5 (CPUID/MAXPHYADDR and reserved physical-address bits), §§11.12.2–11.12.3 (IA32_PAT and selection with PAT/PCD/PWT). `map_mmio` checks CPU address width and requires UC in PAT entry 3 before the 4 KiB mapping with PAT=0, PCD=1 and PWT=1; firmware PAT remains unchanged. Sources checked on 27 October 2026.

The trusted base includes kernel transport and the protocol-conforming QEMU device/backend. M5 contains no IOMMU: CPU isolation and bounded descriptor construction do not demonstrate containment of malicious DMA. The pool remains pinned until VM stop; reuse requires completion or confirmed reset. Without a reset acknowledgement, it remains quarantined and the device offline. A used entry has no epoch; software tickets do not replace a hardware reset guarantee. Revise this decision before direct user-driver DMA, SMP, physical hardware or a different device backend.

The first scope is read-only on a synthetic 64 KiB disposable image. Writing, persistence and recovery remain M6. Restart uses a bounded test coordinator and an actual new service PID/epoch plus IPC capability transfer; no general spawn API. The client stays alive. The external test oracle checks sector checksums on two different images with the same kernel and checks that all image bytes remain unchanged. Executed results and open points are recorded exclusively in PROJECT-STATE.md and verification/m5/. Restore: a separate checkout of M1–M4 on `c630c59`; no host-disk or firmware migration.


## ADR-029 — Native object service with revisions and bounded journal

**Date/status/owner:** 27 October 2026, implemented and full native gate passed within the bounded QEMU profile. The native `Unknown` error path has a known, unrepaired observer panic. The lead agent integrates; separate builders and AI reviewers check the service, broker and harness.

**Context and decision:** M6a stores one fixed value. M6b adds an actual ring-3 object service over the same bounded READ/WRITE/FLUSH ABI: four objects in two endpoint-bound domains, GET/PUT/DELETE, at most 128 bytes and 32 append-only records. The limits are trial boundaries. The object core is own dependency-free safe Rust; no new external crates are added. The kernel retains the existing DMA broker plus an explicit test observer; object policy runs in the service.

**Alternatives and consequences:** adding a full file system with names, garbage collection and encryption at once greatly expands the recovery and authority boundary. An additional host store provides no native progress. This bounded native foundation makes multiple objects and persistent deletion testable now, with explicit later extensions for space reuse, names, crypto and the desktop. No migration of existing M6a images; M6b uses only its own disposable volumes.

**Invariant:** the receiving endpoint capability binds the domain. No app fields as block or domain authority. Mutations compare the current revision, and only actual WRITE completion followed by FLUSH completion may produce a successful reply. A tombstone preserves the revision so old requests cannot undo deletion. Corruption causes no silent fallback. The service contract requires uncertain mutations to return `Unknown` and block further valid service operations without automatic replay. This `Unknown` contract was tested only at host-ABI level and is not implemented for native WRITE/FLUSH errors because of the known observer panic. Full appears before mutating I/O. Deletion is logical; old payload bytes remain in the journal and are not cryptographically protected.

**Rationale and evidence:** own contract `implementation/M6B-OBJECT-STORAGE.md`, earlier virtio/capability ABI from ADR-026/M6a; wire and journal layouts are internal project decisions. The complete final gate on `1e61a20` passes: 18 groups/14 actual VM runs, 22 Rust host tests and 26 harness tests. These include six cold recoveries, exhaustion, corruption and two actual external kills. Capture and a separate audit of 121 artifact hashes/seven disk volumes pass; an additional Python parser independently checks actual disk bytes without sharing the Rust journal code. Seven host-ABI adapter trials check `Unknown` outcomes. Native WRITE/FLUSH errors can cause a kernel panic through `sent()` in `kernel/src/m2/m3/m6b.rs`: the observer first compares the reply with an expected `Ok` reply. This defect is unrepaired; controlled end-to-end handling as `Unknown` is also absent in `milestone-m6b`. The native gate contains no device-error injection for this path. Exact source, regressions and evidence boundaries are recorded in `PROJECT-STATE.md` and `verification/m6b/`.

**Revision and rollback:** revising the native `Unknown` path requires a native fix, targeted WRITE/FLUSH device-error injection, checking the actual client response and new native verification with corresponding source binding. Also revise for more objects, compaction, multiple writers, keys, native UI or other devices. M6b has its own feature/ELF/volume format; existing M1–M6a profiles remain selectable. A separate working branch permits withdrawal without changing host disks or real user data. QEMU tests do not demonstrate physical power loss, IOMMU containment, formal correctness or production readiness.

Addendum to ADR-029, 28 October 2026: the terminal WRITE/FLUSH observer error
was fixed on `592dccf` and natively tested with two status-snapshot injections;
the historical statements above apply to their old source. Further repair
of local poll LIMIT, the actual broker deadline and reset quarantine is the
active next step K08; executed results remain in PROJECT-STATE.md.

## ADR-033 — opt-in native graphics bootstrap before product integration

Date 28 October 2026, accepted as a bounded development step; root integrates,
G09 owns core/loader/adapter. The browser prototype provides no evidence of
native display and input support. Therefore, a separate G0 feature comes
first, with validated GOP metadata, supervisor framebuffer and bounded PS/2
polling. Alternatives are building a full compositor/driver/IPC broker at once
(too large a first hardware step), or presenting a host browser as native
(no progress on the project's own hardware boundary).

Invariant: opt-in ABI 1.2 with a fixed descriptor in the retained BootInfo
page, no firmware calls after EBS, no framebuffer alias to RAM/user/executable,
strictly bounded RW/NX/UC device pages and safe `no_std` render/input core.
The kernel display is temporary diagnostics, not app, object or storage
policy. The screen labels the trial. No rights for arbitrary apps and no
new externally published production ABI.

Contract/sources/acceptance: [GRAPHICS-G0.md](implementation/GRAPHICS-G0.md).
Test evidence is stored separately; this decision is not a boot test.
Revise before a ring-3 compositor, IRQ input, DMA/GPU, SMP, Unicode/IME or
real hardware. Restore through separate checkout `7b4945f`; default BootInfo
1.1 remains selectable, no real disk or firmware migration.


## ADR-034 — Native SDK and bounded quantum port as separate profiles

Date 28 October 2026; implemented as development profiles, final acceptance
on the combined source still to be performed. Root owns ABI/boot configuration;
G01–G03 build the SDK/client, I01–I04 the port and checks.

The SDK reuses the existing capability ABI and 320-byte IPC envelope.
Canonicalisation, endpoint rights, reply origin and one outstanding request
are explicitly bounded. Only EAGAIN before enqueue may be retried. After
enqueue, Unknown, an invalid reply or deadline closes the session without
automatic mutation replay. The catalogue determines revisions/names;
the kernel receives no app or document policy. The existing M6c probe can
select SDK transport, while raw negative ABI trials retain their old path.
A separate native example uses the actual catalogue client; compilation does
not yet demonstrate general app launch.

The quantum port uses the unchanged own qsim-core with at most six qubits,
64 gates, 4096 shots and a fixed 64 KiB arena per process. A monotonic
allocator avoids unproven reuse; the trial terminates the processes, after
which existing kernel cleanup reclaims the pages. Software floating point
from the pinned Rust toolchain enables the first port without a scheduler
change for FPU/SIMD context. A disassembly gate checks the concrete ELF,
including instruction prefixes; this check is not formal proof. A new opt-in
profile contains two user processes and only a fixed kernel test observer.
The profile is mutually exclusive with M4/G0.

The alternatives — new syscalls, unbounded allocation, FPU state support or
a full job service at the same time — expand this initial change boundary.
Consequence: native reference calculations and resource boundaries are now
executable, but arbitrary job submission/results/cancellation outside the
compiled trial program remain a separate next step. Control fixtures do not
demonstrate external cancellation or a clock deadline. L02 therefore earns no point yet.

Own contracts: SDK README/ABI, `implementation/QUANTUM-SPEC.md` and existing
capability/M3 contracts. Targeted native execution, exact commands, hashes,
review findings and boundaries are stored separately in `verification/parallel-*`.
Revise before dynamic apps, more qubits, allocation reuse, FPU/SIMD, SMP or
an actual job service. Both profiles are opt-in; restore by building their
earlier source in a separate checkout. No migration of real data, new external
crate or physical-hardware action.


### ADR-034: separate invariants and falsification

**SDK — security invariant:** ordinary apps obtain only already granted,
process-bound endpoint rights. Every reply is checked for canonical form and
origin. After enqueue and loss of certainty, a mutation remains without
automatic replay; an invalid reply grants no extra rights or success.
**Falsified if:** the same mutation is resent after Unknown/a lost reply,
a foreign sender is accepted or an invalid capability message passes as valid.
**Owner:** G01–G03 for SDK, root for ABI and integration. **Test evidence:**
parallel-g01/g03 and parallel-final-sdk-m6c (the latter tests the runtime bridge,
not the separate catalogue example app).
**Migration/rollback:** existing ABI and journal remain the same; SDK selection
is opt-in, withdrawal through earlier source in a separate checkout without
data migration.

**Quantum — security invariant:** each trial process has private mappings and
at most a 64 KiB arena; input stays within 6 qubits / 64 gates / 4096 shots.
SIMD/FPU and foreign syscall/hardware instructions are not allowed in the
selected scalar artifact. Simulator policy remains in userspace; cleanup
uses actual process frames. **Falsified if:** limits can be exceeded, processes
reach each other's memory, unwanted instructions pass the artifact gate or
cleanup leaves allocated frames. **Owner:** I01–I04 for port/gate, root for
the test observer and integration. **Test evidence:** parallel-i04,
parallel-final-qsim and parallel-final-m3. This is not formal proof.
**Migration/rollback:** exclusive opt-in profile; build earlier source
separately, without changing stored user data or hardware firmware.


## ADR-035 — Security Floor remains active when privacy is voluntarily reduced

**Date/status/owner:** 28 October 2026; accepted as a product and architecture
invariant, not yet fully implemented natively. The lead agent oversees integration;
security owners assess enforcement boundaries.

**Context:** Nieuw OS must have privacy-preserving defaults, but the user may
consciously permit online services, telemetry, microphone, cloud sync or other
data sharing. Such a choice must not turn into a general “trusted app” or
“less secure mode”.

**Decision:** privacy choices expand only explicit capabilities and data flows.
Kernel/process isolation, capability checking, memory and key isolation,
package/update authenticity, protection of unshared data and future DMA/IOMMU
boundaries remain outside that privacy choice.

**Security invariant:** consent expands functionality, not trust. An internet
grant alone provides no document, key, camera, microphone, NAS, process or kernel right.

**Falsified if:** a privacy toggle, provider connection, telemetry opt-in or
focus/gaming mode activates a broader privilege class or bypasses existing
isolation checks.

**Evidence/migration:** future native LP/capability tests; no new acceptance
point from this document. Existing grants retain the same semantics.
Rollback: restore this ADR and linked contracts together; no data migration
in this documentation step.

## ADR-036 — An external provider may only provide opaque ciphertext storage

**Date/status/owner:** 28 October 2026; accepted architecture goal, no native
provider adapter yet. The storage/security owner owns the format and key boundary.

**Context:** the earlier revision prohibited vendor cloud entirely. Optional
use of existing general storage, such as OneDrive and Google Drive, must be
possible without giving the provider read access to stored content.

**Alternatives:** absolute provider exclusion limits choice; provider-native
encryption/key escrow shifts trust to the provider; custom cryptography is unacceptable.

**Decision:** an explicitly selected external provider may store only opaque,
client-side encrypted Nieuw OS objects. The provider receives no content key,
document plaintext, sensitive manifest plaintext, local search index or AI
context. No silent provider fallback. The OS may define its own versionable
container/object format over maintained standard cryptography.

**Security invariant:** possession of provider state/credentials alone yields
no accepted document plaintext within the documented threat model.

**Falsified if:** provider credentials or stored bytes without a valid Nieuw OS
key suffice to obtain plaintext; a provider adapter can read other local
objects/keys; or an outage silently activates another provider.

**Evidence/migration:** [implementation/ENCRYPTED-OBJECT-STORAGE.md](implementation/ENCRYPTED-OBJECT-STORAGE.md)
defines EO acceptance. The existing NAS route remains valid. No real provider
account, production key or user data is configured by this decision.

## ADR-037 — Own premium native productivity family, no Office imitation

**Date/status/owner:** 28 October 2026; product decision for P3/P4/M7,
not yet implemented natively. The product/UI owner owns the design system;
the graphics owner later owns renderer/compositor boundaries.

**Context:** privacy-friendly alternatives must not feel like outdated system
software. At the same time, a direct copy of Office, Windows or macOS would
undermine the independent product identity.

**Decision:** Materiaal remains the object model. Own text, spreadsheet,
presentation, PDF/media and later productivity tools share one design system,
contextual tools, history, sharing, recovery, motion and accessibility. No
permanent copy of the Office ribbon as the foundation. A shared native
GPU-accelerated UI/rendering layer is the long-term goal once the graphics
foundation permits it; the exact renderer/API remains a later technical
decision. DOCX/XLSX/PPTX/PDF are interoperability goals to be tested per feature.

**Product invariant:** familiar basic actions remain predictable; distinctiveness
comes from Materiaal, object continuity, contextual tools and capability-aware
sharing, not unfamiliar controls for originality's sake.

**Falsified if:** core apps end up as inconsistent standalone webviews, the
interface structurally copies an existing suite, or privacy is usable only
through administration-style UX.

**Evidence/migration:** PX acceptance in PRODUCT-SPEC.md and later user/native
tests. The current P2 browser prototype remains a design tool and is not
declared a product runtime by this ADR.

## ADR-038 — Privacy explanations are optional; capability decisions remain real

**Date/status/owner:** 28 October 2026; accepted UX/security contract,
not yet natively enforced.

**Context:** understandable examples and critical questions can clarify
privacy choices, but repeated warnings can disrupt the experience.
Educational privacy explanations must be fully disableable.

**Decision:** the UI separates (a) necessary decisions about new capabilities
from (b) educational privacy explanations. Educational explanations can be
disabled globally or per app. An already valid grant is not requested again
on every normal action. Focus/gaming suppresses non-urgent educational
notifications; a deliberately started new sensitive feature may still show
a compact, necessary grant request.

**Security invariant:** disabling privacy explanations changes no capability
set and creates no implicit “allow all”.

**Falsified if:** the same state change both disables explanations and
disables/expands access rights, or the system treats a capability not yet
granted as permitted because “explanations are off”.

**Evidence/migration:** LP-29/PX-04 and future native UI/broker trials. Existing
grants do not migrate; only the presentation preference is stored separately.

## ADR-039 — Check DocumentSession natively through the existing catalogue

**Date:** 28 October 2026.

**Status:** implementation integrated into the audit branch; host/cross-build
evidence and a current two-VM normal/cold-recovery trial were performed,
but this changes no existing roadmap criterion.

**Owner:** app/SDK for the user trial and library, runtime for the acceptance
gate, root for selection and integration.

**Context:** the existing M6c probe used the catalogue through direct helpers,
while the actual `DocumentSession` library already exists. The integration
step must use that actual library in ring 3 without expanding the kernel
observer, capability rights, IPC envelope or storage format.

**Alternatives:** (1) continue using direct catalogue helpers, leaving the
app library untested natively; (2) build a separate user ELF with copied
catalogue code, increasing duplication and evidence surface; (3) let the
existing M6c user flow opt into the actual library and link the existing
catalogue modules. Option 3 was selected.

**Decision/implementation:** `DocumentSession<NativeTransport>` runs opt-in
in the existing M6c user processes. `NOS_DOCUMENT_SESSION=1` requires
`NOS_SDK_RUNTIME=1` and is limited to normal/recover. Open, local editing,
CAS save, reopen and LIST remain in userspace. `list()` retains one owner
of the endpoints; service and typed client share the existing catalogue modules.

**Security invariant:** the route uses only already granted, process-bound
endpoints. Replies remain canonically validated; LIST must not erase a local
draft/conflict/uncertainty or reopen a closed session. Uncertain saves are not
automatically repeated. Syscall, IPC, memory and capability boundaries are not
expanded. ADR-035 (Security Floor) remains fully applicable.

**Rationale/sources:** `docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md`,
`sdk/document-session/README.md`, `docs/verification/parallel-g11/`,
`docs/verification/parallel-i13/` and `docs/verification/parallel-i15/`.
RFC 768, RFC 791 and RFC 1071 apply to the separate network parser, as recorded
in `services/net-protocol/README.md`.

**Test evidence:** repository-wide audit CI runs DocumentSession host tests,
SDK/compiler-binding checks, native M6c build, a baseline M6c QEMU smoke test
and two separate DocumentSession VMs (normal and cold recovery).
Historical evidence bundles remain bound to their original source.

**Falsified if:** the trial needs new rights, loses a draft through LIST,
repeats an uncertain save, makes a closed endpoint usable again, must weaken
the observer/oracle, or can count a different binary as the document app.

**Migration/rollback:** default profiles retain their existing selection.
The opt-in can be disabled; there is no migration of real user data, firmware
or physical devices. The headless document trial is not a Materiaal GUI,
general app launcher, larger-document support or space reuse.

## ADR-040 — Copy-on-write volume (M6e) instead of an append-only journal

**Date:** 28 October 2026. M6b/M6c/M6d remain unchanged and separately buildable.

**Status (separated):** *design* accepted for the opt-in profile
`native-volume`. *Implementation*: `services/volume`, the native service/app
in `sdk/examples/volume-docs`, the SDK block API and kernel profile
`native-volume` on PR #47. *Test evidence*: see below; GitHub Actions must
pass again on every new PR head. *Unknown/not tested*: physical disks and
power loss, authentication/encryption, backup/export and larger volumes.

**Rationale/sources:** OASIS, *Virtual I/O Device (VIRTIO) Version 1.2*,
Committee Specification 01, 1 July 2022, §5.2 Block Device, §5.2.6.2:
a write is stable when a `VIRTIO_BLK_T_FLUSH` is sent after the completed
write and itself completes
(https://docs.oasis-open.org/virtio/virtio/v1.2/cs01/virtio-v1.2-cs01.html,
accessed 28 October 2026). The order content → FLUSH → table → FLUSH → ACK
rests on that guarantee; an emulator that does not honour it falls outside
this evidence.

**Test evidence:** host tests `services/volume/tests/` (including exhaustive
crash simulation and the test with two mixed write attempts),
`tests/test_volume_disk.py`, and native gate `tools/test_volume.py`, with
evidence in `docs/verification/m6e/` and
[M6E-ACCEPTANCE](verification/M6E-ACCEPTANCE.md). Anything not recorded there
was not performed.

**Decision:** large documents and space reuse (S04/S05) receive a new
userspace volume format with two alternating, per-sector-sealed table copies
and 4 KiB content chunks. A mutation writes only to chunks not referenced
by the durable table; then FLUSH, the next table in the older slot, FLUSH,
ACK. Released chunks are reused by later commits. There is no journal and
therefore no record limit; a separate compaction step is unnecessary because
there is no append-only log to compact. Crash-consistent release fulfils the
role described as “compaction” in criterion S04. This interpretation is
explicitly recorded here, not silently applied.

**Security invariant:** the kernel remains a generic block broker without
format or domain knowledge (kernel/AGENTS.md: no storage policy in privileged
code). Domain authority comes exclusively from the kernel-bound endpoint.
Apps receive no block right. Read-only recovery is enforced by the emulator,
virtio negotiation, capability (READ only) and service. An error during the
table write returns `Unknown` without replay. A non-canonical, fully sealed
newest copy stops recovery.

**Falsified if:** recovery after a crash point yields a state other than before
or after the interrupted step; a write occurs outside scope or to a referenced
chunk; guest and independent oracle find different digests; quota/Full are
reported only **after** I/O; or read-only recovery changes the image.

**Review correction (CodeRabbit, PR #47):** the original seal bound a sector
only to slot, index and generation. Two interrupted attempts for the same
generation could therefore combine into an apparently complete copy and revive
a rolled-back mutation. Every sector now also carries a digest over the
payload of all sectors in its copy; recovery accepts only a copy whose sectors
all have the same recalculated digest. A mixed copy is treated as torn.

**Residual risk:** one unreadable sector in the newest table copy causes
fallback to the previous generation (loss of the last commit). FNV is not
authentication (B02 open). Trial geometry: 16 objects, 64 KiB per document,
256 KiB content.

**Migration/rollback:** opt-in feature; no automatic migration of M6c volumes
and no use of real disks. Rollback = do not select the feature or specifically
revert the M6e commits.

**Owner:** lead agent for format, contract and integration
(`services/volume`, `tools/volume_disk.py`, gate and evidence); `kernel_m2`
for the generic block broker in `kernel/src/m2/m3/volume.rs`.

## ADR-041 — Volume v2: collections, metadata, bounded history and transactions (S06)

**Date/status:** 29 October 2026, accepted for the opt-in profile
`native-volume`. Format v2 (`NOSVOL02`) and IPC v6 replace v1/IPC v5 within
that profile; there is no migration because the profile uses only disposable
images in QEMU. M6b/M6c/M6d remain unchanged.

**Status:** design accepted. Implementation: `services/volume` (v2),
`sdk/examples/volume-docs` (service and app), kernel case `test-volume-cut-txn`,
oracle and gate. Test evidence: see
[S06-ACCEPTANCE](verification/S06-ACCEPTANCE.md). Unknown: physical disks and
power loss, authentication/encryption (B02), backup/export (S09), larger
history and the native Materiaal interface.

**Context:** Materiaal works with objects that have collections, type, labels
and history; restore creates a new revision (PRODUCT-SPEC §2, P2-05).
M6e had only name and current content.

**Alternatives:** (a) a journal or operation log alongside the table:
rejected because it reintroduces the record limit and compaction that ADR-040
removed; (b) unlimited history through a separate on-disk version chain:
deferred because bounded storage and quota cannot yet be demonstrated then;
(c) collections as ordinary objects: rejected because membership could not
then be checked atomically with object metadata.

**Decision:** every object entry receives a second table sector with two
historical versions; the head stores `tags`, `kind`, collection and logical
times. Those times come from a clock per domain in the table header: the
number of commits by that domain (the clocks sum to the generation).
Collections reside in their own table sector per volume (4 per domain) and
are readable with COLLECTION_STAT, including read-only. RESTORE turns a
historical version into a new revision without rewriting content (chunks
are referenced again); the restored version leaves history and the replaced
current version takes its place, so no other version is lost. DELETE retains
the last version as history; PURGE releases history. TRANSACT performs at
most four META/DELETE/PURGE steps on different objects of one domain in one
table commit. All changes use the unchanged M6e commit protocol. A writable
new instance flushes after recovery before the first request; if that fails,
it does not start.

**Rationale/sources:** ADR-040 (commit order, VIRTIO 1.2 §5.2.6.2) and
PRODUCT-SPEC §2/§6 for the task list. No new external source required.

**Security invariant:** the domain comes exclusively from the kernel-bound
endpoint; a transaction may name only object indices within its own domain,
and an object may belong only to a collection in its own domain. Quota includes
retained history, so history cannot hold space outside quota. Read replies
(STAT, LIST, COLLECTION_STAT, READ) and revisions, stamps and counters in
mutation replies contain only state from its own domain (timestamps use its
own domain clock; LIST returns a domain counter). Two channels between domains
are deliberate residual risks: `Full` depends on shared, overcommitted volume
space (2 × 40 of 64 chunks), and a failed table write by one domain closes
the instance, after which the other domain also receives `Unknown`.
The kernel remains unaware of the format.

**Falsified if:** a partially applied transaction is visible after a crash or
error; RESTORE changes content or erases other history; an object can move
to a collection of another domain; history bypasses quota; or guest and
independent oracle find different domain digests (including collections).

**Review corrections (29 October 2026, before acceptance):** an independent
review found five points incorporated into this decision, each with a
regression test: (1) the global generation in STAT let one domain count the
other's commits; (2) collections were writable only through IPC; (3) RESTORE
of the newest version with full history erased the oldest; (4) a new instance
could write while the selected copy still existed only in disk cache;
(5) the gate could not distinguish a kill before the table write from a torn
table write. The runner now requires an actual torn attempt (prefix of g+1
sectors over the previous copy).

A second review (of those corrections) found no error in production code,
but did find tests that did not catch the intended bug and two overly broad
claims. These were also corrected:

- the test for unrelated generations became vacuous because of the clocks;
- the LIST cursor test did not reach overflow;
- COLLECTION_STAT had no negative tests;
- individual clock boundaries were not tested separately;
- invariant 7 claimed full non-interference;
- a read-only response to an `Unknown` commit was presented as definitive.

Since then, mutation checking has been repeatable and recorded.
`tools/mutate_volume.py` disables exactly one line per mutant in a copy of
the source and requires at least one test to fail. The report with command,
exit status and failing tests is recorded in
`docs/verification/s06/volume-mutants.json`.

**Consequences:** v2 rejects v1 images as corrupt (different magic);
M6e evidence remains bound to its own source, and the S06 gate repeats
S04/S05 scenarios on v2.

**Residual risk:** history is bounded to two versions (the oldest is dropped
when a new version is added); no search index; FNV is not authentication.
`Full` and instance-wide `Unknown` after a failed table write are visible
across domains. A read-only response to an `Unknown` commit is provisional.

**Migration/rollback:** no migration of v1 volumes; rollback = specifically
revert the S06 commits (v1 then applies again with its own evidence).

**Owner:** lead agent for format, contract, oracle and gate; `kernel_m2`
for kernel case `test-volume-cut-txn`.

## ADR-042 — Process management as capabilities: `Launch` and `Process` objects, spawn/kill/status (K05)

**Date:** 29 October 2026. Profile `native-supervisor`, opt-in; all existing
profiles remain buildable unchanged.

**Status (separated):** *design* recorded in
[K05-PROCESS-MANAGEMENT](implementation/K05-PROCESS-MANAGEMENT.md).
*Implementation*: `crates/capability-core` (object types, rights 128/256/512),
`kernel/src/m2/m3/supervisor.rs` (syscalls 0x30, 0x50–0x52, orphan reaping),
`sdk/runtime` (ABI 1.3 wrappers), `sdk/examples/supervisor` (supervisor,
worker, client). *Test evidence*: host tests and gate `tools/test_supervisor.py`;
see [K05-ACCEPTANCE](verification/K05-ACCEPTANCE.md). Anything not recorded
there was not performed. *Unknown*: SMP, physical hardware, user-supplied
images (A02), policy for a system-wide service manager.

**Decision:**

1. Starting a process requires a `Launch` capability on a built-in image.
   Only the boot manifest creates such objects. A process without `Launch`
   receives `EBADHANDLE`, `ETYPE` or `EACCESS`, and nothing is created.
2. `proc_spawn` gives the child only the at most two explicitly supplied,
   attenuated capabilities (the same rules as IPC transfer: `TRANSFER`
   required, never `RECV`, no expansion of rights). All fallible checks
   precede every allocation.
3. The launcher receives the sole handle to a `Process` object with
   `OBSERVE|KILL`. The object type has no `DERIVE`, `TRANSFER` or `REVOKE`:
   this authority cannot be shared or stolen.
4. Death (exit, fault, kill) follows the existing `cleanup` path;
   the `Process` object retains state and code until its owner closes the
   last handle. A running child without handles continues to exist as an
   object so the kernel can keep enforcing its relationship with the launcher.
5. Children of a dead launcher are terminated in the same kernel event,
   iteratively (at most 16 rounds), never recursively on the kernel stack.
   No process remains without a responsible owner.
6. `endpoint_create` (0x30) becomes an ordinary syscall within existing
   quotas (4 own endpoints per process, 64 total). This deliberately revises
   M4's choice “alleen het manifest maakt endpoints” (“only the manifest
   creates endpoints”), because restartable services need their own ingress;
   the RECV ownership rule remains unchanged.

**Security invariant:** authority over processes resides exclusively in
kernel metadata of the calling process; a number from another process yields
at most the caller's own existing authority (KS-12). No partial state after
a rejected or failed spawn: the free-frame count is equal before and after,
otherwise the run stops with `PARTIAL_SPAWN`. `check_invariants` requires
a running `Process` object to point to a live process and every child to
be named by exactly one object.

**Falsified if:** a process without `Launch` can start a child; a `Process`
handle reaches another process through IPC or `derive`; frames, nodes,
endpoints or queued capabilities remain after exit, fault or kill; a child
continues running after its launcher dies; or a rejected spawn changes the
free-frame count.

**Residual risk:** a supervisor can misuse its own children within its rights;
there is no policy yet for who may be a supervisor. Exit codes are reported
modulo 2^56. The ABI minor increases to 1.3; older profiles report 1.0–1.2
and do not recognise the new syscalls (`ENOSYS`).

**Migration/rollback:** opt-in feature; rollback = do not select the profile
or specifically revert the K05 commits. Rights bits 128/256/512 are included
in `KNOWN_RIGHTS`; older profiles reject them on endpoint and block objects
with `EINVAL`, as before.

**Owner:** lead agent (engine, kernel profile, SDK, example, gate, evidence).

## ADR-043 — Entropy as a capability and a ring-3 Key Service with maintained crypto crates (B01)

**Date:** 29 October 2026. Profile `native-keyring`, opt-in; existing
profiles remain unchanged.

**Status (separated):** *design* in
[B01-ENTROPY-AND-KEYS](implementation/B01-ENTROPY-AND-KEYS.md). *Implementation*:
`drivers/virtio-rng`, `Entropy` object type in `capability-core` (right 1024),
kernel broker `kernel/src/m2/m3/keyring.rs` (syscalls 0x60/0x61),
`services/keyring` (`keyring-core`), SDK runtime ABI 1.4,
`sdk/examples/keyring`. *Test evidence*: [B01-ACCEPTANCE](verification/B01-ACCEPTANCE.md);
anything not recorded there was not performed. *Unknown*: actual hardware
RNG quality, TPM/hardware protection, password recovery, side channels.

**Decision:**

1. The kernel owns the virtio-rng device and supplies raw bytes only to the
   holder of an `Entropy` capability (manifest-only object type without
   `DERIVE`/`TRANSFER`/`REVOKE`). The kernel mixes, caches or reuses nothing
   and contains no cryptography (kernel/AGENTS.md).
2. Readiness and failure are explicit: 1024 bytes of startup entropy through
   SP 800-90B Repetition Count (cutoff 6) and Adaptive Proportion tests
   (window 512, cutoff 62; conservative H = 4 bits/byte, α = 2⁻²⁰).
   Absent, stalled or short-delivering device → `BLOCKED` with a reason;
   no fallback, no test seed in the product path. Later reseeds undergo
   the same tests; failure → `DEGRADED` and rejection.
3. Only maintained, pinned RustCrypto crates with portable backends:
   `chacha20poly1305` 0.11.0 (XChaCha20-Poly1305), one production
   `chacha20` 0.10.2 for the DRBG and RNG, and `cipher` 0.5.2 with explicit
   wrapper zeroization, plus the protected HKDF/SHA-256 stack and `zeroize`
   1.8/1.9. `chacha20` 0.9.1 is retained only as a dev compatibility oracle.
   The DRBG is the
   fast-key-erasure construction on the ChaCha20 keystream (Bernstein, 2017),
   not a custom primitive. Dependency check: MIT/Apache-2.0 licences,
   Rust 1.85 minimum for the 0.10.2/0.5.2 crates, `no_std`, and no build
   scripts for either. The production builders use the single
   `chacha20_backend="soft"` selector; old 0.9-only backend configuration is
   excluded from production. Exact provenance and evidence boundaries are in
   [KEYRING-CRYPTO-DEPENDENCIES](implementation/KEYRING-CRYPTO-DEPENDENCIES.md).
4. Keys never leave the Key Service; apps receive (domain, index) handles;
   the domain follows from the kernel-bound endpoint. AEAD binds format,
   domain and index in the AAD. 16 keys per domain; revocation is permanent
   and travels with the recovery blob.
5. Recovery without escrow: a random 256-bit recovery secret for the
   requesting app and a blob (master key + revocation bitmaps, 76 bytes)
   under that secret. RESTORE in another instance derives all domain keys;
   the service does not retain the secret.

**Dependency amendment (10 November 2026):** the direct DRBG and maintained
RNG were consolidated on `chacha20 0.10.2`. This changes dependency/API
plumbing, not the DRBG construction, derived-key labels, AEAD, or persisted
wire format. Root tests retain 0.9.1 only as an independently imported
byte-for-byte oracle; the three consumer lockgraphs contain 0.10.2 only.
Cold recovery and persistent restore remain part of the locked regression
suite. This amendment does not itself constitute native B01/RANDOM evidence.

**Security invariant:** no key generation before `READY`; no bytes from a
rejected, short-delivering or timed-out request; `entropy_poll` writes only
after validating the entire output buffer and wipes the broker copy; a dead
requester does not subsequently receive its response through another process.
Cross-domain and tampering attempts return `Auth`/`BadHandle` without
partial plaintext.

**Falsified if:** a process without `Entropy` receives bytes; a service creates
keys without successful startup tests; tampered ciphertext, tag or nonce
yields plaintext; a restored domain makes a revoked key usable again; or
the kernel leaves frames, nodes or objects afterwards.

**Residual risk:** QEMU supplies host `/dev/urandom`; the tests assess behaviour,
not the actual min-entropy of physical hardware. The master key resides
unprotected in service memory (no TPM). Zeroize is hygiene, not a guarantee
about copies in registers or crash memory. One broker request at a time:
services retry on `EAGAIN`.

**Migration/rollback:** opt-in; rollback = do not select the profile or revert
the B01 commit. Rights bit 1024 is rejected on other object types with `EINVAL`.

**Owner:** lead agent (driver, engine, kernel broker, core, SDK, example,
gate, evidence).

## ADR-049 — Hybrid-signed kernel updates with slot storage, version floor and fallback (B03)

Renumbered from ADR-044 to ADR-049 when integrating PR #61 with main
`6d3bbf0`: ADR-044 on main describes networking. Both decisions retain
their content; historical evidence is not rewritten.

**Date:** 29 October 2026. Loader feature `signed-boot` and kernel profile
`native-update`, both opt-in; loaders without the feature (all other profiles)
retain the existing ESP path (`kernel.elf`) unchanged. A `signed-boot`
loader never falls back to that path.

**Status (separated):** *design* in
[B03-SIGNED-UPDATES](implementation/B03-SIGNED-UPDATES.md). *Implementation*:
`crates/update-core` (manifest `NOSMAN01`, slot storage `NOSSLOT1`, hybrid
verification, selection and installation policy), `boot/loader/src/signed.rs`,
`kernel/src/m2/m3/update.rs` (generic block broker over the slot storage, one
labelled crash fixture), `sdk/examples/update` (ring-3 update service),
`tools/update-tool` (keygen/sign/pack/verify), `tools/update_disk.py`
(independent host oracle), gate `tools/test_update.py`. *Test evidence*:
[B03-ACCEPTANCE](verification/B03-ACCEPTANCE.md); anything not recorded there
was not performed. *Unknown*: firmware verification of the loader itself
(Secure Boot, B04), production key rotation and revocation, network distribution,
physical disks and power loss, behaviour with an attacker who can write OVMF_VARS.

**Decision:**

1. A kernel image is started only from a slot whose manifest verifies with
   **both** pinned anchors, Ed25519 (`ed25519-dalek` 2.2.0, serial backend)
   **and** ML-DSA-65 (RustCrypto `ml-dsa` 0.1.2, FIPS 204), over the same
   84 canonical bytes, and whose image has exactly the length and SHA-256
   (`sha2` 0.10.9, `force-soft`) from that manifest. One valid signature
   is insufficient; there is no fallback to one algorithm.
2. The loader stores the highest started version in the UEFI NV variable
   `NieuwOsMinVersion` (vendor GUID `50402e54-a387-47bc-8f75-dcd79cd0a603`,
   NV+BS) and rejects every slot with a lower version as `DOWNGRADE`, even
   if the header points to it. The floor only increases.
3. The slot-storage header is an unsigned hint for slot order. If the
   designated slot fails (format, key ID, signature, digest, floor, reading),
   the other slot is tried; if both fail, boot stops in a controlled manner
   (`BOOT:FAIL:NO_VALID_SLOT`, exit 0x11); if slot storage is absent,
   `BOOT:FAIL:NO_STORE`. Empty, invalid or missing slot storage never falls
   back to an unsigned ESP kernel, and the build places no `kernel.elf` on
   the ESP for this profile. A correctly signed but unloadable image (ELF
   parser) is a `FORMAT` rejection of that slot; the floor increases only
   for an image that the loader can actually load.
4. The update service writes only to the slot that is **not** running.
   The running slot is determined by the verified manifest version matching
   the kernel's reported own image version, not by the header.
   Order: image, FLUSH, manifest, FLUSH, header, FLUSH; an interruption leaves
   the active slot and header intact, and installation is idempotent and resumable.
5. The service verifies the inbox bundle with the same anchors, checks the
   digest before the first write and rejects `version ≤ huidige` as up-to-date
   or downgrade respectively. Loader and service check independently;
   the kernel knows no manifest or slot format and contains no cryptography
   (kernel/AGENTS.md).
6. The anchors in the repository are **TEST anchors** (`tools/update-keys/TEST-*`,
   generated with `update-tool keygen`), including seeds, so CI signs
   reproducibly. They protect nothing outside the gate. A release profile
   replaces the anchors in a separate signing step kept outside CI;
   key rotation and revocation belong to B04.
7. Dependency check (`cargo tree -p update-core`): `ed25519-dalek` 2.2.0 and
   `curve25519-dalek` 4.1.3 (BSD-3-Clause), `ml-dsa` 0.1.2, `module-lattice`
   0.2.4, `hybrid-array` 0.4.15, `keccak`/`shake`/`sponge-cursor`,
   `signature` 2.2/3.0, `sha2` 0.10.9, `digest`, `crypto-common` (MIT OR
   Apache-2.0), `subtle` 2.6.1 (BSD-3-Clause). Build scripts: only
   `curve25519-dalek` (platform detection for word width, no network) and
   `generic-array` (`version_check`); one proc macro (`curve25519-dalek-derive`,
   through `syn`/`quote`). SIMD backends are disabled
   (`curve25519_dalek_backend="serial"`, sha2 `force-soft`); `objdump` of
   loader and service shows 0 xmm/ymm and 0 x87 instructions.
   ML-DSA verification holds approximately 200 KiB of stack: the loader runs
   selection on its own 1 MiB allocation, and profile `native-update` gives
   the service 512 KiB of user stack instead of 64 KiB.

   The `ml-dsa` 0.1.2 release is RustCrypto/signatures commit
   [`25e6bf946725ace2af69b15874c29131acc7db60`](https://github.com/RustCrypto/signatures/commit/25e6bf946725ace2af69b15874c29131acc7db60)
   of 31 October 2026. Its upstream crate source declares `#![no_std]`; the
   package manifest declares MSRV 1.87 and `Apache-2.0 OR MIT`. The project
   keeps `default-features = false`, so the default `alloc`, `getrandom` and
   PKCS#8 features are not selected. The verification backend remains the
   crate's pure-Rust `module-lattice` 0.2.4
   and SHAKE path; no optional ML-DSA hardware backend is selected. The runtime
   route decodes only a public ML-DSA-65 verification key and signature and
   calls `verify_with_context` with an empty context. It neither creates nor
   retains a secret signing key or signing SHAKE state, so the optional
   `zeroize` feature is not selected for this verification-only boundary. Any
   future signing or private-key use requires this feature decision to be
   reassessed.

**Security invariant:** no byte of a kernel image is executed before format,
key IDs, both signatures, length, digest and version floor are checked;
the last validly started slot remains available unchanged through every
interruption of an installation.

**Consequences:** slot storage costs 6 MiB per device (three slots of 2 MiB);
`image_length` is bounded to 2 MiB. Package and app updates (A02), network
distribution (N track) and firmware verification of the loader (B04) remain
open. The `KERNEL:IMAGE_VERSION` marker and startup-page words 120/192–400
are profile-bound test interfaces, not an OS ABI.

### Addendum ADR-043 — monotonic recovery state and continuous monitoring (29 October 2026)

Concrete post-merge counterexamples from #52: an older recovery blob erased
local tombstones, and every reseed created a new health-test window.
The recovery change retains Health from startup and unions tombstones when
restoring the same master. A different master is allowed only in a still-unused
instance; a used instance rejects the switch without changes. No wire-format
or cryptographic-primitive change. After a cold restart, rollback detection
still depends on future trusted persistent state (B02); a recovery blob alone
does not demonstrate freshness.

The broker confirms virtio reset before reuse; if reset ACK is missing,
DMA remains pinned and the broker offline. Loss of ownership stops the device
and closes the broker; the DMA pool remains quarantined until reboot.
Two separate native fixtures demonstrate an actually silent source and a
normal reset/restart; old `stuck` is exclusively a constant-byte fixture.
No physical-entropy or hardware-recovery claim.

Owner: lead agent; changes and evidence in native-recovery-hardening.
Revision: persistent anti-rollback/recovery contract in B02, or a concrete
counterexample to the live-instance invariant. Rollback: revert recovery
commits and keep B01 open; do not accept uncertain evidence.

## ADR-044 — Native networking through one bounded capability broker

**Date:** 30 October 2026. In progress as part of the full release;
no N01/NAS/TLS acceptance. NetworkSource is granted exclusively to the configured
ring-3 network process; apps receive no MMIO/DMA or general network right.
The own kernel validates handles, full user copy and lengths, manages pinned
RX/TX buffers, completion/timeout and reset ACK/quarantine. ARP/DNS/TCP and
later TLS/configuration remain in userspace. ABI 1.5: 0x70 send/0x71 completion/
0x72 receive; closing/death terminates the source. No automatic packet replay.
Supported trial device: modern virtio-net 00:07.0, one CPU/QEMU 256 MiB.
Actual native ARP/UDP/DNS counters and independent 8/7-test peer pass on
3879c32, with the same cold volume recovery. A full app/destination broker
remains mandatorily open.

Test firmware is not an OS network client. An actual OVMF MLD reproduction
led to EDK2 runtime options VirtioNetSupport/IPv4Support/IPv6Support=no and
romfile=, with the PCI NIC intact and a strict peer. A causal old-kernel
A/B trial confirms this. Rollback: revert the network feature from the
integration branch and keep N01 open; do not ignore a pre-boot frame or
disable driver/rights checking.

## ADR-045 — Eager x87/SSE context and actual boot-monotonic time

**Date:** 30 October 2026. Opt-in mechanisms fpu-context and monotonic-clock;
preserving microkernel/process isolation requires separate CPU state per
process. The kernel manages 16 aligned 512-byte FXSAVE images; save on CPL3
entry before Rust, restore only after all Rust/CR3 selection immediately before
GPR pops/iret. A fresh PID/slot starts with wiped register payloads, fixed
FCW/MXCSR and empty x87 tags; cleanup wipes everything. CPUID check, EM/TS off,
MP/NE on, OSFXSR/OSXMMEXCPT on, AMD FFXSR off/read back; XSAVE/AVX remain off.
No GUI/crypto/network policy moves into the trusted kernel. The kernel itself
uses no FP/SIMD and disassembly is checked. One CPU; physical AMD errata/
extended FP exceptions remain open.

Sys3 returns current 100 Hz PIT ticks since boot, exclusively with six zero
arguments, with saturation. The SDK recognises ABI 1.6; older profiles retain
their earlier ABI. No wall-clock/RTC/resume contract or real-time guarantee.
The native-fpu fixture has actual syscall/timer switches, three clean lifetimes,
an actual unsupported-AVX #UD, a surviving process and slot reuse. A separate
illegal-MXCSR trial reveals TCG's missing #GP; that failing evidence remains
preserved and was not declared passing. Host artifact helpers must not
overwrite a symlink/hard link outside the checkout. Root integration is
repeated separately natively. Rollback: revert the opt-in mechanisms and keep
FP-dependent product features visibly open.

## ADR-046 — Maintained TCP library in a ring-3 app

**Date:** 30 October 2026. Design decision, implementation/native acceptance
in progress. Use smoltcp=0.14.0 (0BSD), minimal no_std Ethernet/IPv4/TCP
features, integer Reno, fixed 1 RX/1 TX 1514-byte slots and at most two TCP
sockets with 8 KiB per buffer. No std/alloc/libc/TUN, no packet parser in
the kernel and no artificially advanced time. The TX slot stays occupied
until actual broker completion. Timeout/retransmission uses Sys3 alongside
host lifetime bounds.

The current library requires Rust 1.91.1: only this app package/all its local
Rust dependencies are built with it and loaded as their own ELF. Kernel/loader/
older apps remain pinned to 1.90; no mixed rlibs or global default change.
Matching rust-lld and locked dependency chain; build scripts and eight registry
crates were checked for provenance/licence/build behaviour. 0.12 was rejected
because of later TCP/IPv4 validation fixes. Known upstream issues for large
RX windows and listeners remain visible; the bounded client trial does not
demonstrate those general server/NAS cases. The fixed trial ISN seed is only
for test data; production entropy and TLS remain open release blockers.
EOF, refused/reset, an actually lost first SYN/retransmission and black-hole
timeout are tested natively against a protocol-conforming loopback peer.


## ADR-047 — Bounded own heap and maintained user allocator

**Date:** 30 October 2026. Implementation in progress; no full memory,
engine or product acceptance. Sys4 (opt-in ABI 1.7) grows only the process's
own contiguous RW/NX heap with fresh zero-filled pages. The existing total
process quota remains 2048 frames including program and stack. No shrink,
physical mapping, executable heap memory or delegation. Fallible batch
reservation and full user **and** kernel page-table preflight precede
publication; errors preserve old mappings and clean up reservations.
The first profile remains one CPU with IF=0.

The separate userspace allocator uses exactly `linked_list_allocator` 0.10.6
(MIT/Apache-2.0, defaults off), an explicitly exclusive arena and its own
bounded bookkeeping. No host fallback or silent automatic growth.
`OwnedHeapPages` does not prove safe Rust aliasing: the initialisation check
remains explicit. Independent review, native zero filling, quota/OOM, NX,
isolation, reuse and cleanup are separate gates. The 128 MiB OOM trial changes
neither the existing 256 MiB regression profile nor the 8 GiB product target.
Rollback: revert opt-in features; allocator/engine-dependent product criteria
remain visibly open.

## ADR-048 — Native display and input through minimal transport capabilities

**Date:** 30 October 2026. Contract approved, implementation in progress.
The existing validated GOP header/mapping and bounded pixel helper are
separated from the G0 kernel rendering loop. The separate `framebuffer`
feature contains only hardware metadata and supervisor RW/NX/UC transport.
G0 retains its separate IF=0 trial; Materiaal, fonts, documents and input policy
belong in ring 3. No VRAM frame is allocated as user RAM or released by heap cleanup.

ABI 1.8 is reserved opt-in: DISPLAY=4096 and INPUT=8192 are issued only by
the manifest, PID/generation-bound and non-transferable/non-derivable.
0x80/0x81 provide public geometry and a synchronous tile blit of at most
32×8 pixels (LE `0x00RRGGBB`); no physical base/stride or vsync guarantee.
All user copy, descriptor fields, pixels and complete destination rows must
be valid before the first hardware write. Only the trusted compositor receives
display authority; untrusted apps receive controlled surfaces/portals.

0x82–0x84 form a bounded PS/2 reset state machine and raw-source reading
exclusively for the isolated input service. Actual ACK/BAT/identity and
Set2→Set1 configuration precede Ready; both channels and error state remain
visible. Validated output precedes FIFO consumption. Revocation, death,
stale tickets, time limits and generation exhaustion must yield neither access
nor false Ready. This is not a general driver/DMA/IOMMU guarantee. Details
and execution evidence follow in the transport contract and native QMP control
trial. Rollback: revert opt-in transport, preserve G0 regression and keep
product criteria open.

## ADR-052 — Separate native CRB aperture trial, no product TPM policy

**Date:** 30 October 2026. `native-tpm-crb` combines only the existing actual
IPC-wait trial with the existing safe TPM transport/command core.
The kernel owns one fixed supervisor RW/NX/UC page at VA `0x3f180000`
for QEMU locality 0 `0xfed40000`; no arbitrary physical mapping, DMA,
new user grant or public syscall. The private non-Copy adapter performs
bounded volatile copies, explicit compiler/CPU fences and sticky quarantine
that permits no new claim. Only the bootstrap fixture encodes one read-only
GetCapability(TPM_PT_FAMILY_INDICATOR); no crypto, NV mutation, keys or storage
policy in the kernel.

The existing scheduler supplies actual PIT ticks to every bounded poll.
No poll-count clock, hidden busy-wait loop, automatic replay or fallback.
Readiness, exact response, buffer wiping, Idle and relinquish ACK precede
success. A separate absent case requires rejection before any hardware write.
The full existing CPL3 wait/rights/cleanup check remains applicable.
Both actual VM runs are still open at this design checkpoint.

This is initial MMIO integration, not a kernel grant for product apps,
TPM discovery, physical support, trusted boot, persistent key lineage or
an independent rollback anchor. PR #64 retains its own keyring/system storage
writes; this experiment does not change that interface.
QEMU v10.2.1 `hw/tpm/tpm_crb.c` was read as the primary implementation:
https://github.com/qemu/qemu/blob/v10.2.1/hw/tpm/tpm_crb.c
Register/buffer/Idle/locality completion match the existing contract.
OSDev: *Memory mapped registers in C/C++*, W003198, revision 30402
(24 March 2026), is supplementary explanation only; no normative TCG check.
Revise before a product broker, other hardware or NV commits. Rollback:
remove opt-in features/fixture; legacy profiles remain unchanged.

<a id="adr-053--native-vensterbeleid-en-vervangbare-presentatiediensten"></a>
## ADR-059 — Native window policy and replaceable presentation services

**Date:** 2 November 2026. The new opt-in window session uses the project's existing
kernel, capability engine, INP1 and SUR1. Two isolated applications each
receive their own surface and input channel; the userspace compositor owns
the sole DisplaySource. Only the existing input source owns InputSource.
The compositor receives decoded input through the input service and manages
global focus, window position and trusted title bars. Caller PID comes from
kernel IPC, never from a message field. No shared framebuffer or shared app memory.

A new no_std graphics library uses pre-rasterised proportional Open Sans
glyphs with a recorded Apache licence and own vector icons. No untrusted
font parser enters the kernel. Backing stores are private, bounded to two
windows and 1,400,000 pixels within existing process quotas. The old 800×600
diagnostic profiles remain; the window session explicitly requests a suitable
UEFI GOP mode of 1280×720 or 1920×1080.

ABI 1.11 reserves syscall 0x53 exclusively for the manifest supervisor with
the corresponding Launch and deceased Process capability. The supervisor
determines recovery policy and first has apps close their old presentation
channels. The kernel only replays the fixed compositor/input-service manifest
group, after a full capacity check, with fresh PIDs/capabilities/epochs.
App, document, volume and key-service state persist. This is not general
device delegation or kernel GUI policy. See the exact build/test contract in
`docs/implementation/NATIVE-WINDOW-SESSION.md`; native acceptance is still
open at this design checkpoint. No U01 points for this build step alone.

<a id="adr-054--exacte-tijdelijke-autoriteitsvergelijking-op-de-syscallgrens"></a>
## ADR-060 — Exact, temporary authority comparison at the syscall boundary

**Date:** 2 November 2026. The complete fresh `Stats` and digest functions
remain the existing diagnostic oracles. At the M4 failure boundary, their
repeated hash scan is replaced by a complete typed copy of all eleven authority
fields and existing exact `Engine::eq`. IPC wait compares this on every result.
Separate user-output, capability, pointer, return, page-ownership and W^X
checks remain intact. The snapshot is never restored to the active engine
and grants no rights.

One pre-reserved private kernel buffer of 461,816 bytes avoids a large stack
temporary or allocator. The single-CPU/IF0 contract is checked explicitly;
a non-blocking lease rejects reentry. Exhaustive field coverage makes a future
Engine extension fail compilation until the copy is deliberately updated.
Comparison uses Rust fields, not padding. All 1024 extra message copies are
wiped with typed volatile writes before release and before any failure
assertion; no extra IPC content remains in the released buffer. Metadata
remains private kernel data only.

This is not a mutation cache or disabling an audit. Native IPC wait and
targeted host/negative tests were performed on the component branch; full
combined M4, window and network gates follow. Host microbenchmarks provide
no native latency guarantee or U02 points. SMP or reentrant kernel callbacks
require a new design. Details and evidence boundaries:
`docs/implementation/EXACT-AUTHORITY-SNAPSHOTS.md`.

<a id="adr-055--begrensde-rust-in-gui-processen-via-bestaande-ipc_wait"></a>
## ADR-061 — Bounded idle in GUI processes through existing IPC_WAIT

**Date:** 2 November 2026. Materiaal, Rekenmachine (Calculator), compositor
and window-init park only after an actual empty receive/input check and fully
completed own work. The wait route uses an existing receive right and a
checked deadline of one next PIT tick. Wake bits are hints: original receive/
PEEK/reply validation remains mandatory. Control remains reachable;
save, incomplete frames, backpressure, errors, recovery and STOP prevent
idling. No new right, kernel ABI or changed transaction deadline.

An empty pull-input reply is not a promise of a future push; waiting only
for an input reply is therefore insufficient. The short deadline prevents
new input from remaining pending indefinitely. The compositor retains its
two-PIT-tick polling threshold; no hard 20 ms/p95 guarantee follows from it.
Six complete Windows/WHPX gates and TCG CI pass; three 15 s measurements per
arm yield 88.26% lower median process CPU with the same complete image on
one PC. This is not an energy or general performance claim. Revise for an
event-driven input protocol, SMP or changed recovery states; U02 still requires
clipboard and full interaction budgets. Contract and source-bound evidence:
implementation/GUI-IDLE-WAIT.md and verification/gui-idle-20261102/INTEGRATION.md.

<a id="adr-056--klembord-als-eenmalige-focusgebonden-tekstoverdracht"></a>
## ADR-062 — Clipboard as single-use, focus-bound text transfer

**Date:** 2 November 2026. The existing trusted window-compositor process
brokers a volatile CLP1 clipboard for explicitly selected text. Existing app
input endpoints and their sole InputSession owner are reused; no new kernel
ABI, process or device/storage/network capability. Actual PID, input epoch,
offered CtrlC/CtrlV Press sequence/generation, current focus and a single-use
broker claim are jointly required. An app-written intent label grants nothing.
Pending backpressure retains only the ticket; every flush rechecks before
plaintext is constructed.

After a valid delivery response, the app applies text only on the actual
input ACKAcknowledged. Superseded/transport failure wipes the temporary copy
without editing. The broker cannot retrieve text already enqueued; revocation
stops future boundary crossings and is not an information-flow guarantee.
Copy clears old clipboard content at the trusted key event, so a locally
oversized/empty selection causes no silent old-text fallback. CtrlShiftDelete
clears and revokes tickets; compositor restart starts empty. Text never appears
in logs/Debug or on disk, and own buffers are wiped with existing pinned zeroize.

The first version has 192 UTF-8 bytes without truncation, no history,
whole-document/answer fallback or automatic execution. This remains a bounded
step: U02 and broader capacity/formats/interaction budgets remain open.
A lost DeleteRelease may conservatively swallow one Delete cycle after a clear;
the latch prevents an old clear repeat from erasing document text. Revise for
streaming, more apps, secure account fields or a real session boundary.
Contract, source and still-running native gate: implementation/NATIVE-CLIPBOARD.md
and verification/native-clipboard-20261102/.


## ADR-MATERIAL-LIBRARY-20261102 — native interaction with existing volume objects

The native workspace exposes existing volume-v7 names, collections, metadata
and two history slots; no new wire format, storage format or kernel right.
Renaming remains a draft until explicitly saved. Organising uses CAS; history
restores as a new revision. Uncertain publication is not repeated; a confirmed
restore with failed rereading blocks save until explicit reopen.
A bounded text preview first validates the complete historical version and
UTF-8 in the existing staging buffer. Labels make abbreviation and location
clear. Proportional text shares metrics/caret/selection/wrap/hit-test and
preserves bytes.

The own scalar app-specific RX budget changes from 144→160 KiB after an
actual 157161-byte linker measurement, following the earlier 128→144 KiB
change based on 131177 bytes; fixed addresses, five segments, W^X,
R/RW/stack/guard/kernel quotas and capabilities remain the same. See
`implementation/NATIVE-MATERIAL-LIBRARY.md`.
The native gate uses complete allocated app images and an independent disk
decoder; cold reopen copies only a revalidated source/profile-bound disposable
run. U03 remains open until the full route and regression actually pass.
No authentication, encryption or hardware claim.

<a id="adr-058--positieve-volume-identiteit-vóór-toelating-als-lege-opslag"></a>

## ADR-CI-RUNNERS-20261106 — Linux CI only on the Unraid runners

Status: runner choice superseded by ADR-CI-HOSTED-20261108; other agreements
on artifacts and required checks remain applicable. Originally accepted
on 6 November 2026 (project owner's instruction, PR #241).

**Decision:**

- Every workflow job runs on `[self-hosted, Linux, X64, unraid, build]`.
- Personal machines (`MSIMON-wsl`, Windows host `MSIMON`) execute no PR code.
- GitHub-hosted runners are not used without explicit permission because of the budget.
- Artifacts are short-lived (3 days). Evidence of record remains compact in `docs/verification/`.
- Required checks `host-quality` and `qemu-smoke` (ci.yml) receive no path filter.
  Otherwise a skipped workflow would leave a required check waiting indefinitely.
- Non-required gates skip PRs that change only `docs/**` or root-level `*.md`.
- A push to main runs everything.

**Reason:**

- Label `[self-hosted, Linux]` also matched a WSL runner operating as the normal user,
  with access to that user's credentials.
- About 6 GB of artifacts caused passing gates to fail at the upload step.

**Boundaries:**

- The Unraid runners and their container isolation, sudo rights and mounts
  were not assessed here.
- The two Tower runners share one physical host. At most two heavy jobs
  therefore run simultaneously, but that host's CPU, RAM and I/O were not measured.

## ADR-CI-HOSTED-20261108 — GitHub-hosted Ubuntu runners by default

Status: accepted, 8 November 2026, on the project owner's explicit instruction
to convert workflows from self-hosted to GitHub-hosted runners.

**Decision:** all 24 job definitions in the 21 existing workflows use
`runs-on: ubuntu-24.04`. Component and toolchain matrices together produce
42 jobs. New or still-open workflow changes must use the same runner choice;
an old branch must not reintroduce self-hosted selection. This replaces the
runner choice and earlier budget restriction in ADR-CI-RUNNERS-20261106.
The fixed Ubuntu image preserves the Linux/x64 environment for existing
`pc-q35-8.2` trials, including the Windows QEMU test package built on Linux.

Existing Rust pins, test commands, timeouts, rights, required checks, path
filters and three-day artifact retention remain intact. QEMU/firmware steps
now reference `.github/actions/setup-qemu` according to ADR-CI-PINS-20261108;
that shared pinned setup replaces the original separate apt installation of
missing packages. The host Rust build cache receives a new namespace with
`ubuntu-24.04` so old self-hosted build results are not restored.
Cargo download caches remain usable; native images and acceptance results
are not cached.

**Costs and publication:** standard GitHub-hosted runners are free and unlimited
for public repositories, within execution and concurrency limits. Private
repositories are subject to the included compute budget and any additional
billing. Larger runners are outside this choice. The GitHub API reported
`private: true` at this change; this assignment changes neither visibility
nor billing settings. Artifact and cache storage have separate limits.

**Evidence boundary and recovery:** configuration checking is recorded in
`verification/github-hosted-runners-20261108.json`. A runner label or valid
YAML is not native boot evidence; the existing CI gates on the new PR head
remain necessary for that. Rollback is a targeted revert of this migration,
without changing other source or branch rules.

**Primary sources, accessed 8 November 2026:**

- GitHub. (n.d.). *GitHub-hosted runners reference*.
  https://docs.github.com/en/actions/reference/runners/github-hosted-runners
- GitHub. (n.d.). *GitHub Actions billing*.
  https://docs.github.com/en/billing/concepts/product-billing/github-actions
- GitHub. (n.d.). *Actions limits*.
  https://docs.github.com/en/actions/reference/limits


## ADR-CI-PINS-20261108 — Fixed tools for regular hosted CI

Status: accepted, 8 November 2026, on a follow-up instruction from the project
owner. This decision supplements ADR-CI-HOSTED-20261108 and replaces its
separate installation of missing emulator packages. It concerns 21 regular
workflows; the private publication audit added later retains its separate scope.

**Decision:** shared local Actions install Python 3.12.12 and QEMU/firmware
from Ubuntu snapshot 20261108T000000Z. All package versions are exact;
APT verifies signed indexes and payloads. After installation, versions,
TCG, machine pc-q35-8.2 and firmware hashes are checked. PyYAML, already used
by the host gate, receives an exactly hashed 6.0.3 wheel. Rust 1.90.0,
existing 1.91.1 checks and Node 24.22.1 remain intact.

**Tests and consequences:** 99 existing runner/preflight/M6c host tests enter
host-quality. The existing six native smoke-test starts, harness conditions,
job time limits, rights and triggers remain the same. An additional compact
QEMU diagnostics upload is best effort and does not change the required test
outcome. Local agent gates skip only GitHub setup and runner diagnostics;
strict preflight and native trials remain necessary.

**Evidence boundary:** runner distribution and tools are selected, but the
complete VM and preinstalled shared libraries are not immutable. Rust/OS gates
on the new head remain separate evidence. A host test or QEMU tool probe does
not demonstrate product acceptance. Score 41/100 remains unchanged. Pin updates
require repeated checking; recovery uses a targeted revert without weakening
branch rules.

Sources, versions, execution and revision route: [CI.md](CI.md),
[check report](verification/ci-standard-runners-20261108.json) and
[package provenance](verification/ci-packages-20261108.json).

<a id="adr-public-development-20261108--openbare-ontwikkeling-met-mpl-20-als-controlebasis"></a>

## ADR-PUBLIC-DEVELOPMENT-20261108 — Public development with MPL-2.0 as the review basis

**Status:** accepted as the project direction on 8 November 2026, on the
project owner's explicit instruction. Licence checking and actual publication
remain open.

**Decision:** Nieuw OS chooses public development. Existing
[LICENSE-PLAN.md](../LICENSE-PLAN.md) remains the basis for licence review,
with MPL-2.0 as the intended licence for the project's own OS code. The assignment
confirms the direction and review framework; it does not apply a project
licence or declare the repository ready for publication.

**Reason:** development must be inspectable and suitable for collaboration.
Public availability is deliberately accepted as part of the project direction.
The plan proceeds from the existing MPL choice, without starting a new general
licence comparison now.

**Consequences:**

- Own code, contributions, documentation, assets, models and external components
  receive explicit rights and licence boundaries according to the plan.
  Existing external terms remain intact.
- Bundled OSDev/book material receives a separate publication review according
  to its own source register; it does not automatically fall under the MPL
  direction for own OS code.
- The five existing publication conditions remain open until the actual
  inventory, compatibility, notices, licence text and source delivery are
  correct. Public availability does not mean third parties cannot copy the source.
- Migration to GitHub-hosted runners is addressed separately in
  [PR #249](https://github.com/simonvanieperen/nieuw-os/pull/249). This decision
  changes no workflows, repository visibility or branch rules.
- Source publication and a fully fledged product release have different
  acceptance boundaries. No OS criterion is signed off; the score remains
  41/100 and R03 remains open.

**Implementation state at source check:** `main` at `363b486b06023d3d578f8eca2b23ff49045a1757`
was private; the root contained `LICENSE-PLAN.md` and no applied project licence
in `LICENSE` yet. This observation is not a permanent repository setting.
Separate publication checks remain necessary.

**Revision criteria:** reopen the MPL scope or a component choice if rights
holders, actual dependencies or distribution terms require it. Changing the
public project direction requires a new explicit project decision. Reverting
documentation does not automatically revoke rights already granted or copies
already distributed.

**Sources:** the project owner's instruction of 8 November 2026; the existing
licence plan; Mozilla's official MPL-2.0 text and FAQ, with full references in
[LICENSE-PLAN.md](../LICENSE-PLAN.md).

## ADR-CI-EVIDENCE-20261108 — test outcome separate from artifact storage

**Status:** decided and implemented on `ci/artifact-storage-20261108`
in PR #251; inclusion in main follows existing checks and review rules.
**Owner:** Simon van Ieperen.

**Date:** 8 November 2026. Artifact storage must not continue to block
preservation of test evidence and the workflow. On main `363b486b`, the
investigated native tests pass, but upload-artifact rejects new archives
because of quota. Three-day retention was already configured.

**Alternatives:** merely shortening retention does not immediately help
existing quota and it was already three days. Targeted cleanup can free space,
but does not structurally protect against storage filling again and first
requires a checked inventory of evidence to retain. Merely making old uploads
non-blocking leaves no compact source-bound report when they fail. External
storage introduces a new storage/access boundary and is unnecessary for this repair.

**Decision:** all fifteen evidence uploads use one local composite action.
It first writes bounded reports, source binding and missing files to the
run log/summary and a compact receipt. Only uploads are best effort; tests,
builds, validators and the collector remain required. The original error/warn
choice for entirely missing evidence remains intact. Actual upload outcomes
are separately visible.

Normal report archives are bounded and remain available for three days.
Raw images, frames and build directories are uploaded as a full archive only
on manual request, with one-day retention. Windows test ZIPs remain separately
available after successful checks if their upload succeeds, also for one day.
A full archive does not require a second ZIP upload.

A report with a hash does not replace raw bytes for independent rechecking.
Native acceptance conditions, download status and fixed progress criteria
are not weakened. This decision changes no runner choice, tool version,
check name, billing setting or storage right. No artifacts or workflow runs
are deleted indiscriminately.

**Security invariant:** the compact collector reads only the explicit selection
within the checkout, follows no symlinks and rejects hidden paths. Content
remains byte-identical in the compact package; workflow commands and control
bytes are made inert only for run-log display. File, scan and output budgets
are bounded. The Windows package must be a real file within the checkout;
both package workflows also select that ZIP explicitly or through its parent
directory in the full archive. The configuration verifier checks this,
including omitted-package cases. This is not a general secret scan;
full-evidence retains the original workflow selection and is not an archive
filtered by the compact collector.

**Test evidence:** original verification comprises 15 collector host tests
and 151 local YAML/shell checks, recorded in
`docs/verification/ci-artifact-storage-20261108.json`. The follow-up check
is recorded in `docs/verification/ci-artifact-storage-20261108/review-followup.json`.
On merge checkout `89722c5f` of PR head `5c291cff`, actual native-display-input
and B03 jobs also passed while the report upload failed because of artifact
quota; source binding and labelled report evidence remained in the run log.
This demonstrates the tested fallback on upload failure; not a successful
artifact download, Windows execution, full raw-data recheck or new product acceptance.

**Migration/rollback:** inclusion of PR #251 activates the shared action for
new runs. Existing artifacts, retention periods and run logs are not changed.
Rollback uses a targeted revert PR of this change; the original blocking upload
steps then return. Preserve changes from other branches and existing acceptance evidence.

**Revise when:** a subsequent job must download artifacts, actual release
delivery becomes mandatory, or the measurable report budget proves structurally
too small. A required artifact transfer then receives its own mandatory delivery
gate. Procedure, sources and rollback: [CI-EVIDENCE.md](CI-EVIDENCE.md).

## ADR-CI-PUBLIC-20261108 — public CI without own runners

**Status:** decided and implemented in existing PR #249; new GitHub execution
and merging are recorded separately. **Owner:** Simon van Ieperen.
**Date:** 8 November 2026. The GitHub repository is verifiably public.

**Decision:** integrate the fixed hosted toolchain from #249 with the compact
evidence route from #251. All job selectors, including the private historical
audit and Copilot setup, are `ubuntu-24.04`. No new self-hosted infrastructure.
The private audit retains its condition and does not automatically publish
historical audit content on the public repository.

Remove the duplicate `ready_for_review` trigger; test draft PRs already on
opening/changing/reopening. Mirror existing document filters to main pushes
of specialist workflows. Preserve unfiltered required checks `host-quality`
and `qemu-smoke`, which can run independently in parallel. Preserve existing
concurrency groups and time limits. The component matrix loses two duplicate
entries; their checks remain in the specialised display/input and artifact-store
workflows, including scalar build. At most four matrix jobs simultaneously.
The host build cache receives a source SHA plus a compatible restore prefix;
no cached native acceptance.

**Alternatives:** a second migration PR would make the same configuration
conflict again. Having a new central path-classification script skip every
native gate increases the risk of missing required checks. Generic self-hosted
labels or paid larger runners do not fit the explicit choice of standard
public GitHub runners.

**Security invariant:** PR code retains a read-only token, external Actions
remain pinned to commit SHA and checkout does not retain credentials. No
`pull_request_target` execution of PR code. Only artifact uploads may fail
without replacing the test outcome. YAML regression tests guard these boundaries
and preservation of required checks. Native oracles, negative cases, markers
and product criteria remain intact.

**Test evidence:** local host tests, configuration/shell checking, source
preservation and new GitHub runs are recorded with source binding in
`verification/ci-public-runners-20261108.json`. Historical test outcomes from
#249/#251 remain with their original sources. No product point: 41/100.

**Migration/rollback:** merging existing PR #249 activates the combined change;
treat #251 as superseded only after proven inclusion of its content. Targeted
revert through a PR, without removing other code or stored evidence archives.
The old Copilot WSL choice from #245 is replaced by a hosted setup; that branch
must not bring back a self-hosted selector. **Revise when:** actual runner
duration, error logs or required checks call for a demonstrable execution adjustment.

Sources and execution route: [CI.md](CI.md).

## ADR-CODERABBIT-20261108 — targeted reviews without extra automatic change requests

**Status:** implemented on follow-up branch `ci/coderabbit-review-policy-20261108`
after merging PR #249; effective CodeRabbit review and integration are
checked separately.
**Owner:** Simon van Ieperen. **Date:** 8 November 2026.

**Reason:** CodeRabbit submits a `CHANGES_REQUESTED` review even for small
documentation points. It can remain after a new review task finishes successfully.
The active main rule already requires passing `host-quality`/`qemu-smoke`,
up-to-date branches and resolved review conversations; no bot approval is
required. Older PRs also have conflicts and CI that still selects removed runners.

**Decision:** record repository settings in `.coderabbit.yaml`: `quiet`
places the most important feedback inline and groups other suggestions;
`auto_approvals.enabled: false` stops new automatic change requests and approvals.
Automatic incremental reviews remain enabled and docstring coverage remains
a warning. Other configuration is inherited to preserve existing tools,
path instructions and additional review branches.

**Alternatives:** removing the app loses review feedback. Closing all old
change requests or threads at once ignores errors not yet assessed.
Disabling GitHub's resolved-conversations rule also affects other reviewers.
Changing only the status check does not help: `CodeRabbit` is not a required
check in the inspected main rule.

**Preservation and verification:** no source path excluded, test gate changed
or branch rule bypassed. Fix concrete findings before resolving their associated
threads. A dismissal receives a substantive reason and is not new code evidence.
Schema, host and workflow checks are recorded with hashes in
`verification/coderabbit-unblock-20261108.json`. Schema validation does not
prove effective UI/global configuration; check the next bot review.
The OS score remains 41/100.

**Migration/rollback:** CodeRabbit uses YAML from the reviewed PR branch.
New branches receive the configuration after integration; older branches must
incorporate it together with the hosted workflows. Existing change requests
remain to be addressed separately. A targeted revert restores earlier review
configuration; resolved conversations remain traceable.
**Revise when:** important errors appear only in summaries, inherited/global
settings alter the intent, or required human-review arrangements change.
Primary sources: [CI.md](CI.md).

<a id="adr-baryon-name-20261108--naamverklaring-in-de-readme"></a>

## ADR-BARYON-NAME-20261108 — name explanation in the README

**Date:** 8 November 2026. **Status:** editorial decision for project presentation;
no architecture or product-acceptance change.

**Decision:** use Baryon OS in the README and explain the name through composite
particles with three valence quarks. The connection with a small own kernel,
separate system services and its own user environment is a metaphor for cohesion.
Processor rings retain their technical meaning. Fault isolation, recovery,
security and performance remain claims requiring appropriate implementation
and test evidence.

**Sources and consequences:** the [name explanation](../README.md#waarom-de-naam-baryon-os)
references CERN/CMS and ESA. The original repository name `nieuw-os`, technical
identifiers, historical evidence files and source references remain intact.
This assignment registers no domain and establishes no trademark availability.
Progress remains 41/100.

**Revise when:** primary sources, the selected project name or actual architecture
require a substantive adjustment. A targeted revert of the naming passages
restores the previous presentation without reverting code or technical contracts.


## ADR-LICENSE-APPLICATION-20261109 — Apply MPL-2.0 to explicitly scoped project software

**Date:** 9 November 2026
**Status:** accepted
**Authority:** project owner.

**Context:** the repository is public, and the decision of 8 November 2026 established MPL-2.0 as the licensing baseline without yet applying it. This left the legal scope of project-owned software unnecessarily ambiguous.

**Decision:** include the official, unmodified MPL-2.0 text as `LICENSE`. Project-owned software covered by MPL-2.0 is listed explicitly in `MPL-2.0-FILES.txt`. `LICENSING.md` defines the scope; third-party code and assets retain their own terms through `THIRD-PARTY.md` and component-specific notices.

**Consequences:**

- the software license is auditable without implying that every repository file has the same license;
- documentation, historical evidence files, third-party assets, and project marks are not placed under MPL-2.0 solely by the root license;
- new project-owned software files must be deliberately added to the MPL scope and receive an appropriate notice;
- distribution still requires a separate release/SBOM/notice review.

**Review criterion:** changing the primary license, scope, or exclusions requires a separate decision that addresses migration consequences for already covered files and contributions.

## ADR-K06-5F-20261109 — Bound and measure SMP acceptance without widening authority

**Date:** 9 November 2026. **Status:** implementation in progress; native acceptance open.

**Source:** `main` `1db01c4a1fe9e433396c58af4e6b3732034ee2e9`, with the selected kernel, scheduler and test changes from `feat/k06-5e-proc-shootdown` `f141995ce6d5a0032cb90dea1466a24dc71b2f4d`. The latter includes the step-5d-2/FPU checkpoint `4030c4f4`. Unrelated runner migrations are excluded. The normative acceptance requirements remain [L0 step 5f](https://github.com/simonvanieperen/BaryonOS/blob/docs/k06-step-specs/docs/implementation/k06-steps/09-5f-acceptance.md).

**Decision:** complete cross-CPU IPC, address-space-locked usercopy and remote process lifecycle handling before accepting migration or SMP quota evidence. Process state remains protected by KERNEL. Usercopy and mapping changes use the process address-space lock. A remote wake publishes runnable state before sending vector `0xe8`; an interrupt is a hint, and the receiver rechecks state under KERNEL. A process must leave its old CPU and address space before its storage, stack or FPU image can be reassigned or freed.

The existing KERNEL lock-stuck bound is retained at 400 ms. The maximum deferred ticks during one acquisition is bounded at 40, derived from that bound and the existing 10 ms process timer period. Per-CPU maxima and both limits are emitted before CPUs are parked. These are acceptance limits, not measured results; they will not be enlarged in response to a failing test without a separate decision. TSC timing is restricted to the existing calibrated QEMU/TCG model. Host overcommit and ticket-lock contention remain R02 risks.

One shootdown generation has one 100-ms calibrated-TSC deadline, independent
of the number of target CPUs; it is not restarted per acknowledgement. The
ten-minute four-vCPU soak uses one scheduler manifest with four real M4 quota
processes, four real qsim pressure processes, a cross-CPU IPC pair and eight
additional CPU-bound processes. M4 authority is cleaned before the IPC pair
is admitted so unrelated queued messages cannot satisfy or perturb the quota
oracle. The qsim result is accepted only after the M4 proof, while all eighteen
processes remain subject to the final frame/FPU/scheduler cleanup balance.

Each matrix case requires 50 successful repetitions. Any failed repetition leaves that case unaccepted; source or artifact drift terminates the run and preserves the partial report. A one-vCPU run is regression evidence. Host watchdog ratios and guest deadline ratios are separate measurements. Mocked harness tests never count as native runs.

**Open acceptance:** complete L0 §9 workloads, migration with GPR/x87/SSE canaries and `active_cpus` checks, remote kill and exact cleanup, real K07 pressure across four CPUs, the 30-second concurrent workload, the ten-minute soak, negative controls and unchanged regressions. The current host checks and source integration do not accept K06 or extend K07 to SMP. No physical-hardware, speculative-execution, x2APIC or human-audit claim is made.

**Recovery:** retain the original worktrees and source branches. The 5f work is isolated on `codex/k06-5f-acceptance-20261109`; revert its reviewed commits to remove the change. No host disk, firmware, NAS or production key is part of the test configuration.


### K06 step 5f measurement and gate repair — 10 November 2026

Run 42723127310 on `f808d591` is rejected because `nine-capacity` iteration 16 fails the unchanged clock-drift gate (49/50). The old evidence logs only the median and does not distinguish IRQ loss from observation jitter. An idle return can observe more ticks than its requested target. Clock samples therefore retain a coherent IRQ0 tick/TSC snapshot with IF=0 and normalize elapsed TSC against the actual tick delta. Five target segments, their median, the observation deadline and the exact 1% drift limit remain fixed. Failure logs retain calibration plus each tick/TSC delta. Missing IRQs are not synthesized and sustained drift remains a failure.

The final repository gate previously required a missing `tools/qemu-smoke-test.sh`. The executable wrapper now runs the existing M2, K05, single-CPU K07 and RTC regression gates sequentially, with fresh isolated output directories and source checks. Host oracles, Clippy and formatting checks remain required. The workflow preserves nested reports and raw logs. The entire 64-by-50 matrix must restart from zero on the corrected source; no earlier passing repetition is reused.


### K06 step 5f FPU-canary readiness — 10 November 2026

The original run rejects `m3-four-single` iteration 33 with an FPU image mismatch. Its general loop-progress word is incremented before register initialization/checks, and the readiness flag was published before initialization. The fixture stop predicate is strengthened to require completed canary readiness for every live canary; the probe publishes it only after successful GPR/x87/SSE checks. The existing non-canary M4/qsim soak slots remain governed by their own oracles. Final FPU-image comparisons, negative controls, 1000-switch minimum, quotas and deadlines are unchanged. Failure diagnostics retain the slot, expected pattern, actual registers, TOP/tag and saved RIP. The old log lacks those fields and does not establish exact historical causality.

A host artifact/mutation check validates the actual probe RX instruction order and failure branches. CI executes it before relevant M3 cases and soak runs and retains its log. The two distinct unmap executables are excluded from that probe-specific check. A complete 64-by-50 restart is required on the corrected source; the earlier pending source is not reused.


### K06 step 5f bounded observer cost — 10 November 2026

The original eight-CPU single-TCG base case is 15/50 and migration case is 17/50, including ten unchanged 60-second watchdog failures in total. Per-process/register timeout diagnostics are absent, so exact historical timeout causality remains unproven. Source review identifies an unnecessary all-process physical-pattern scan on every M3 entry: sixteen reads each acquire ASPACE and map/unmap a scratch page, adding 32 INVLPG instructions per entry under KERNEL. Only the entering process is now sampled before cleanup; all live processes are scanned after STOP before any release. Observer sampling frequency changes to each process's own entries. The actual ring-3 pattern check remains on every loop. All genuine user-return permission walks and usercopy, shootdown, lock and resource enforcement remain unchanged. No workload, 1000-switch minimum, watchdog, quota or safety limit is reduced or increased. A full restart must establish runtime results; static work reduction is not a measured speedup.

## K06 step 5f — per-AP clock-load observation — 10 November 2026

Restart 42731994218 on `59a30f10` is rejected: `resched-deaf` 46/50, with four median clock-drift failures and exactly 1000 IRQ0 ticks in every sample. This evidence excludes the previously corrected observation-edge overshoot for these failures, without proving why IRQ0 falls behind its reference.

The clock fixture's single shared progress counter forced a locked RMW from every busy AP. Progress observation now uses independent 64-byte-separated single-writer counters and requires nonzero work from every online AP. The APs retain their continuously busy loops and PAUSE; separate synchronization tests, PIT/IRQ0, five samples, exact 1% and deadlines are unchanged. The fixture loses shared observer contention, as explicitly recorded. QEMU's use of host atomics does not establish a global QEMU lock or a causal explanation of the recorded drift. Host/crossbuild and object-code checks are not native acceptance. A new full 64×50 run and executed repository gate remain mandatory.

## K06 step 5f — syscall quantum and audit boundaries — 10 November 2026

L0 section 6.3 makes timer, explicit yield, blocking and reschedule the switch causes. Treating every completed nonblocking syscall as yield needlessly forfeits the caller's remaining quantum. The shared selector may now continue that caller only when its CPU's actual deferred-timer and reschedule generations have not changed. The complete existing return/ownership path remains mandatory; continuation acknowledges no new quantum and preserves every peer's FIFO order. Wake remains FIFO. This is not proof of the IPC deadline.

Engine audits move from repeated observations of unchanged state to every completed capability syscall, the existing bootstrap/exit boundaries and final quiescent STOP. Exact authority snapshots, rollback/output checks, usercopy leases and observable erasure remain unchanged. An independent source review and host models support this candidate; full native acceptance remains mandatory.

A separate manual diagnostic mode shares the full matrix concurrency lock, rejects simultaneous matrix input and retains raw executed trace commands. It neither relaxes native limits nor supplies acceptance repetitions. HTTP 503 snapshot failures receive three APT retries with the same signed snapshot, pins and hashes; persistent setup failure remains failure and supplies no native evidence. ADR-053's counted-IRQ0 rule is unchanged; elapsed-counter compensation requires an explicit normative decision and is not implemented.

## K06 step 5f — ordinary canary CPU hint — 10 November 2026

The complete mode19 canary-check path gains one PAUSE before its normal loop back-edge. Mode20 IPC, forced migration-Yield and legacy mode0 paths bypass it. PAUSE is a guest CPU hint; it neither blocks/yields the process nor increments the actual context-switch evidence. All 1000-switch, FPU, workload, clock and deadline checks remain.

The specified TCG profiles may benefit from finer host-vCPU interleaving: QEMU 10.2.1 exits its CPU loop on PAUSE, while its ordinary round-robin kick timer is 100 ms. Infinite starvation is not established, and the five historical eight-vCPU watchdogs are not causally explained by source inspection alone. Host/crossbuild and real-ELF checks pass; native performance and MTTCG BQL overhead remain to be measured. The full 64×50 acceptance restarts on the new frozen source.

## K06 step 5f — proven capless IPC and IRQ-entry clock edge — 10 November 2026

Run 42748931069 on `22dfceaf` is rejected. Sixty of 63 ordinary cases and all ten soak shards pass, but both 10000-round IPC cases time out in every repetition and `m3-canary-control` is 49/50. The environment/K07 probe and both regular K07 cases pass; the aggregate fails and the final gate is skipped. No repetition from this run counts toward acceptance.

Each IPC pair requires at least 40000 successful capability calls. A capability-free SEND or RECV may now omit the otherwise unused exact authority snapshot only after the complete message or output plan, endpoint authority and queue state prove under KERNEL and ASPACE serialization that dispatch has no remaining error path. Capability transfers, unavailable queues, malformed calls, usercopy failures and every other possibly failing operation retain the exact snapshot, rollback comparison and observable wipe. Full structural Engine audits remain at bootstrap, process exit/cleanup and final quiescent STOP. The syscall-local authority, usercopy and output checks remain. Mode20 gains a PAUSE host-vCPU hint before every IPC step; its SEND/RECV/WAIT protocol, FIFO behavior, 10000 rounds and 60-second bound remain unchanged.

The rejected clock repetition counts exactly 1000 IRQ0 entries in each of five segments, while its old endpoint read occurs after return from the final interrupt. IRQ0 now records TSC immediately before publishing each actual tick. Measurement still uses five consecutive 1000-tick segments, actual tick normalization, their median and the exact 1% bound. It synthesizes no tick, and missing or coalesced IRQ0 entries still lengthen the measured interval. Host checks establish implementation consistency only; a complete new 64×50 run and an actually executed green repository gate remain mandatory.

## K06 step 5f — enforce the deaf fixture precondition — 10 November 2026

Run 42809499884 rejects source `3dd6194f`: process-shootdown-deaf is 48/50 because AP1 owns RUNQ at quarantine, and the BSP-spin control is 49/50 because of sustained clock drift. IQ-3 correctly stops before resetting a lock owner. L0 step 07 section 4 explicitly requires the deliberately deaf AP to hold no lock. A test-only, normally acknowledged kernel setup round now closes scheduler admission and masks AP timers while retaining their real CR3/ACTIVE/context. Pending timer/reschedule entries resume without RUNQ. All APs must acknowledge this setup, remain lock-free, and show new AP1 reader progress before the actual A unmap. The A-round still receives no flush/ack; all existing deadlines, frame quarantine and before/after-INIT owner checks remain unchanged. The serial oracle requires the new precondition marker. This does not change production quarantine policy.

The clock-load observer now performs 64 dependent integer steps before each progress store and PAUSE. APs remain continuously busy; QEMU 10.2.1 requires a TCG-loop exit and BQL round trip for each PAUSE. The changed instruction mix reduces that observer overhead per work step, without changing IRQ0 counting, five samples, median, exact 1%, deadlines or work quotas. Historical drift causality and native effectiveness remain unproved. Host/crossbuild/object-code checks are supporting evidence only. Review this choice if the full matrix still fails, and retain every rejected original artifact.

## K06 step 5f — remote readiness wake runs next (ADR-081 R3-5 addendum) — 11 November 2026

Both 10 000-round IPC cases (`m3-ipc-four`, `m3-ipc-four-single`) failed 50/50 in every full run, always by the 60-second host watchdog after `SMP:M3:GO`. A local single-boot trace on frozen source `54a9d5c1` shows the cause: zero wait timeouts and one reschedule IPI per remote wake, yet exactly one round per four PIT ticks. A woken endpoint was appended behind the CPU-bound canaries on its home CPU, so the 0xe8 selection ran a canary and the endpoint waited about two quanta per wake (about 400 s for 10 000 rounds). The earlier syscall-cost and PAUSE changes could not remove this latency.

Decision: a wake from `wait::refresh` that is a readiness wake (`primary == 0`, non-empty mask) for a thread whose home CPU differs from the waking CPU is placed at the head of that home RUNQ through `RunQueue::push_front`. Timeout, error, kill and local wakes keep FIFO `push`. Head placement happens at most once per BLOCKED→READY transition and grants no new quantum; the thread is preempted at the next tick as before. Homes, explicit-only migration (AC-1), the KERNEL→RUNQ lock order, BW-2 single queue ownership and the 0xe8 hint semantics are unchanged. Host tests cover head order, peer FIFO order, full-ring and double-enqueue refusal.

After this change the same trace runs at about 190 rounds per 100 ticks. Two clean local boots on QEMU 10.2.1 pass the unchanged serial oracle (10 000 rounds, `SMP:IPC:PASS`, FAIR waits at most 5 ms against 400 ms) in 71.8 s (multi-thread TCG) and 66.5 s (single-thread TCG). The workload of at least 40 000 capability syscalls does not fit the generic 60-second per-boot watchdog. `tools/run_smp.py` therefore gives only the `m3-ipc` outcome the named constant `IPC_TIMEOUT_S = 180` (at least 2.5 times the measured time), analogous to `SOAK_TIMEOUT_S`. The round count, the oracle and every guest deadline are unchanged; `GUEST_TIMEOUT_S` stays 60.

Residual risk (R02): a colluding cross-CPU pair can obtain about two quanta per round-robin cycle on its home CPUs instead of one. Starvation is excluded because each head placement requires a fresh block and a remote readiness event. Acceptance still requires the full 64×50 matrix on the final source; the two local boots are supporting evidence only.

## K06 step 5f — interruptible spin window survives QEMU 8.2 PAUSE (ADR-081 addendum) — 11 November 2026

The PR job `native-smp` (Ubuntu 24.04, QEMU 8.2.2 TCG) failed with `SMP:LOCK:STUCK:10:CPU=1`: a CPU spinning on `PREEMPT_LOCK` with IF=1 never took the deferred tick, so the holder's 500 ms wait lost to the 400 ms DG-1 deadline. Cause: the spin window `sti; pause; cli` relies on interrupts being taken after the STI shadow ends. In QEMU 8.2 `helper_pause` leaves the translation block through `cpu_loop_exit` without clearing `HF_INHIBIT_IRQ_MASK`, so the window is deaf; QEMU 10.2 clears it through `do_end_instruction`, which is why the K06 matrix on 26.04 never showed this.

Decision: every IF=1 spin window (`IrqLock::take`, `icr_race_window`, `icr_work`) is `sti; nop; pause; cli`. The STI shadow covers only the NOP, so a pending interrupt is taken before PAUSE or CLI on hardware and on both QEMU versions. Saved IF, lock order, the DG-1 deadline and every oracle are unchanged; the change adds one instruction per spin iteration.

Hosted clock drift is a separate matter. Run 43022743756 failed the probe with a median of 1 015 575 µs per 1000 IRQ0 (1.56%; limit 1%) on a 4-vCPU hosted runner. The check measures lost IRQ0 edges, which TCG produces when guest vCPU threads and QEMU's timer starve on an oversubscribed host. The 1% limit stays, ticks are never synthesized (ADR-053), and AP spinning stays part of the test. The workflow now stops runner background services as a best effort; if drift persists on standard hosted runners, the remaining options are a larger runner (paid, Simon's decision) or a separately designed clock measurement.

## K06 step 5f — clock check split into idle precision and load progress (spec 06 §4 revision) — 11 November 2026

**Decision (Simon, 11 November 2026).** Simon chose to redesign the clock measurement instead of moving the SMP matrix to a paid 8-vCPU runner. This revises the "Klok" clause of K06 step specification 06-5c-2 §4. The old clause read: during the existing SMP load, Sys3 stays monotonic and rises within 1% of the PIT expectation. The revised clause has two phases:

1. **Precision (all APs halted).** The BSP idles through `CLOCK_SPAN` counted IRQ0 ticks in five segments. On the PIT-calibrated TSC, the median segment must last the PIT's own period within 1%. The limit stays at 1%. Monotonicity, `SMP_CLOCK_STOPPED` and the drift diagnostics are unchanged.
2. **Load (every AP spinning).** Every AP is woken into the existing load loop. The idle BSP must then count `CLOCK_SEGMENT` further IRQ0 ticks before a 4× deadline; otherwise the check fails with `SMP_CLOCK_STOPPED`. The clock must never go back, and every AP must show work (`SMP_CLOCK_NO_LOAD`). There is no 1% limit under load.

The marker `SMP:CLOCK:PASS:CPUS=` and the fail markers are unchanged. Ticks are still never synthesized (ADR-053): only counted IRQ0s advance the clock in both phases. K06-SMP test 6 keeps a load component through the load-phase progress and work checks.

**Evidence.** Under TCG the old measurement measured host CPU capacity, not guest clock behaviour. Lost or merged IRQ0 edges come from starved vCPU and timer threads. Locally, under `taskset`, the old load-phase check passed 5/5 with 4 host CPUs, failed 0/5 with 3 (2–4% drift) and failed 0/5 with 2 (20–27% drift). Lowering QEMU's priority with `renice` did not change this. On 4-vCPU hosted runners it failed in runs 43031091677 and 43031085708 (about 1.8%) and in run 43022743756 (1.56%). PROJECT-STATE records the local `taskset` reproduction of the revised check; native acceptance still comes from the hosted matrix.

**Not claimed.** This change does not show that the guest clock stays within 1% while all CPUs are loaded. Under TCG on a shared host that property cannot be separated from host capacity, so it remains unproven. It must be measured again on hardware or KVM before a load-precision claim is made.

**Open item.** The normative step text lives on branch `docs/k06-step-specs` (`docs/implementation/k06-steps/06-5c-2-bsp-timer-idle.md`, §4 "Klok"). It must receive the same revision when that branch is integrated.

## K06 step 5f — IPC watchdog recalibrated for hosted runners (ADR-081 R3-5 addendum 2) — 11 November 2026

**Decision (Simon, 11 November 2026).** Full run 43057843393 on `e398039e` failed `m3-ipc-four` 0/50: every repetition hit the 180-second host watchdog after `SMP:M3:GO`, without a FAIL marker, on GitHub-hosted 4-vCPU runners (2 cores with SMT). `m3-ipc-four-single` passed 50/50 at 71–84 s. Locally, with QEMU 8.2.2 confined by `taskset`, multi-thread TCG completes the unchanged 10 000 rounds in 116 s on 4 host CPUs, 130 s on 3 and 327 s on 2, with zero wait timeouts; single-thread TCG takes 103–109 s. The IPC phase runs at about one round per scheduler tick, so its wall time scales with host CPU capacity. No kernel defect is demonstrated. The R3-5 value of 180 s came from 71.8 s on a faster host and therefore violated its own rule of at least 2.5 times the measured time on the acceptance runner.

`IPC_TIMEOUT_S` becomes 660 s, the existing maximum of `tools/run.py` and equal to `SOAK_TIMEOUT_S`. The round count, the oracle, the FAIR limits, every guest deadline and `GUEST_TIMEOUT_S = 60` are unchanged. The 2.5× rule stays: if the hosted per-boot time of `m3-ipc-four` in the next full run exceeds 264 s, the margin is insufficient and K06 stays open; the runner question then returns to the owner. A canary PAUSE hint on one pass in 64 instead of every pass was measured (432 s against 327 s on 2 CPUs) and rejected.

The single idle-phase `SMP_CLOCK_DRIFT` in the same run (soak shard 9, median 1.05%, limit 1%) receives no code change by the same decision; the next full run starts from zero and a recurrence rejects it.

## K06 step 5f — review findings closed before acceptance (ADR-079/ADR-081 addendum) — 12 November 2026

**Decision (Simon, 12 November 2026):** fix all four confirmed review findings on PR #296 before the full matrix, rather than defer them. Run 43118230329 on `ae3e985` was cancelled because these changes alter kernel and tool sources; its 25 completed cases (50/50 each) are retained as diagnostics only ([receipt](verification/k06-smp-sched/cancelled-43118230329.json)).

1. **IQ-3 transitional lock states (ADR-079).** `held_by()` read only the owner fields, so an AP reset between taking a ticket and publishing its owner, or between clearing the owner and releasing the ticket, left a stranded ticket undetected. Each CPU now records the `IrqLock::id()` of the lock in that window (`CpuLocal.transit`, saved and restored LIFO so a handler in the spin window cannot erase it). `held_by()` stops on an owner match or a transit match, only for the IQ-3 locks (`KERNEL`, `ASPACE`, `RUNQ`, `SHOOTDOWN`). A level-based lockdep check was tried first and rejected: the test lock `SPIN_LOCK` shares level 10 with `KERNEL`, which turned the accepted `spin-irq-off-control` quarantine into a false `LOCK_HELD`. New negative case `quarantine-lock-pending` (AP2 deaf inside KERNEL's acquire window, `SMP:QUARANTINE:LOCK_HELD:10:CPU=2`) joins the matrix, which becomes 65 cases × 50 (3 250 guests); the verifiers and the aggregate job name follow.
2. **DG-2 harness verdict.** `run_smp.py` accepted the `!`-prefixed terminal fallback in its oracle but rejected it in the exact-marker harness check. The harness verdict is now recomputed only for that fallback line, with the same exit, timeout and artifact binding.
3. **K05 on four vCPUs (spec 5d-2 §4).** The explicit integration profile `native-smp-supervisor` (= `native-smp-m3` + `native-supervisor`) and its five `test-smp-supervisor-*` variants are the only way the build guard admits the supervisor on SMP. `run_supervisor.py --smp` runs the six cases on four vCPUs (multi-thread TCG) with the unchanged evidence oracle plus `SMP:ONLINE:4/4`, `SMP:K05:PASS` and `SUPERVISOR:SMP_HOMES` (at least one spawned service on an AP). The final gate gains the stage `k05-smp` (`test_supervisor.py --smp`). On SMP, `WAIT_IDLE_IRQS` now counts, as on one CPU, the IRQ0s the BSP observed while idle; it previously counted every idle wake of every CPU, which broke the unchanged `WAIT_IDLE_IRQS <= TICKS` check.
4. **Recovery from an idle CPU.** `smp_idle()` dispatched `select()`, and with it `smp_recovery::recover()`, only when its run queue was non-empty. A recovery request reaching an idle BSP with an empty queue (reachable once the supervisor runs on SMP) therefore stalled. A pending recovery now dispatches `select()` regardless of queue occupancy.

No tolerance, deadline, repetition count, oracle or security check was relaxed. Acceptance still requires the full 65×50 matrix, all 50 soaks, K07, the aggregate and the executed final gate on the final commit.

## K06 step 5f — responsive APs go home before the IQ-3 checks (ADR-079 addendum) — 12 November 2026

**Decision (Simon, 12 November 2026).** Full run 43125206729 on `95bdf4c` (the tree PR #296 merged as `4e8e235`) failed 2 of 3 250 guests; the receipt is `docs/verification/k06-smp-sched/incident-43125206729.json`.

- `resched-deaf` r020: after the expected kill timeout on APIC 3, `quarantine_all` resets every AP while the healthy AP1 still runs the scheduler phase. AP1's tick held or was acquiring its RUNQ when IQ-3 checked it, so IQ-3 correctly stopped (`SMP:QUARANTINE:LOCK_HELD:40:CPU=1`) instead of resetting a lock holder. This is the class of run 42809499884, where the owner-only check caught a RUNQ owner; the transit tracking of the previous addendum only widens the detection window. Spec 05 IQ-3 states the check for the deaf CPU and assumes the other APs hold nothing; nothing made that true for a responsive AP.
- `eight` r050: the step 5b-2 scratch phase exceeded its 500 ms deadline after the BSP's own share (8 vCPUs on a 4-vCPU host). A local diagnostic (QEMU 8.2.2, 4 host CPUs, 20 + 20 boots) shows the same heavy tail with and without transit tracking (mean wait 30.0 ms and 29.9 ms; maximum 291 ms and 198 ms). Simon decided to keep the code and the deadline and rerun.

Fix: `quarantine_all` first calls `settle_scheduler`. If the scheduler phase is running (`SCHED == PREEMPT_RUN`), it sets `PREEMPT_STOP`, the same stop as `stop_scheduler`, and waits at most 200 ms until every AP reports `RESUMED`: home, lock-free, its timer masked. A deaf AP never reports and is checked as before. The owner and transit checks before and after INIT, the fail-closed stop and every deadline are unchanged; outside the scheduler phase nothing changes.

Regression case `quarantine-sched-busy` (4 vCPUs, new feature `test-smp-quarantine-sched-busy`): APIC 3 is deaf to the kill as in `resched-deaf`, and AP1 takes its RUNQ in a tick and holds it until the scheduler is stopped. The oracle requires the precondition marker `SMP:QUARANTINE:SCHED_BUSY:CPU=1`, then the kill timeout, `SMP:QUARANTINED:APS=3`, `SMP:ONLINE:1/4` and M1:PASS. Without `settle_scheduler`, the same image stops with `SMP:QUARANTINE:LOCK_HELD:40:CPU=1`, as checked locally. The matrix becomes 66 cases × 50 (3 300 guests); the aggregate job, verifiers and handoff follow. Acceptance again requires the full matrix, all 50 soaks, K07, the aggregate and the executed final gate on the final commit.

## ADR-082 (D1–D5 decided by the project owner on 13 November 2026 and implemented; K06 step 5f not yet accepted) — K06 fixture timing: measured margins, liveness apart from performance, PIT edge accounting — 12 November 2026

**Status/owner.** This entry replaces the proposal text of the same number written earlier on 12 November 2026. It has two parts that must not be confused. *Implemented* describes what PR #317 contains: measurement and fail-closed checks, with every acceptance limit unchanged. *Decisions requested* lists the contract changes that the evidence supports; none of them is in force, and each needs an explicit decision by the project owner. K06 step 5f remains unaccepted, and no acceptance matrix has run on this source.

**Context.** Three consecutive full matrices were each rejected by one or two guests out of about 3 300 on a wall-clock limit: idle `SMP_CLOCK_DRIFT` (run 43057843393, and twice in run 43187501026 on a two-vCPU guest), `SMP:SCRATCH:TIMEOUT` (`eight`, run 43125206729) and `SMP:SYNC:TIMEOUT` (`nine-capacity`, run 43187501026). The investigation ([TIMING-INVESTIGATION.md](verification/k06-smp-sched/TIMING-INVESTIGATION.md)) found no kernel or synchronisation defect and two host mechanisms: the FIFO ticket lock hands the lock to a vCPU whose host thread is descheduled (sync, scratch), and QEMU merges overdue PIT edges into one pending IRQ0 (clock). The repair evidence is in [TIMING-REPAIR.md](verification/k06-smp-sched/TIMING-REPAIR.md).

### Implemented in PR #317 (no acceptance limit changed)

1. **Margin records (spec 09-5f AC-3).** Every bounded wait of the fixture (`within_ms`, `within_tsc`, `idle_until`), every contended `IrqLock` wait (DG-1 limit) and every shootdown round records the largest part of its limit that a boot used. The BSP prints `SMP:MARGIN:<WAIT|LOCK|ROUND>:<id>:<used>/<limit>` after the self-tests; the oracle stores the values as `guest_waits` in the evidence. The records carry no verdict: each of these waits still fails in the guest at its unchanged limit. A full record table (64 keys; 43 are used) prints `SMP:MARGIN:OVERFLOW`, which the oracle rejects.
2. **Sync wait: liveness apart from performance.** The guest polls every 10 ms as before. It now stops with `SMP:SYNC:STALLED` when the summed progress of all CPUs did not advance for 2 s, and with `SMP:SYNC:TIMEOUT` after 20 s; otherwise it prints `SMP:SYNC:WAIT_MS=<waited>:STALL_MS=<longest run without progress>` and continues. The oracle requires that line at a fixed position in every sequence that passes the sync phase and rejects a wait of 2 000 ms or more on every profile; the limit is a constant of the oracle, not a value read from the guest. The acceptance verdict is therefore the former one (ready within 200 polls of 10 ms). What changed is the failing party for a wait that is slow but alive (the oracle instead of a guest marker), and that such a boot records how long the wait really took. A wait without progress for 2 s also failed under the former rule.
3. **PIT edge accounting in shadow mode.** After every wake of the idle precision phase the BSP reads the PIC's pending IRQ0 and latches PIT channel 0, which places the wake on the PIT's own period grid. `smp_lock::clock::Edges` (host-tested) books each period as counted, pending, late or lost. A lost period is charged to the host only when the sample has a complete witness: exactly one IRQ0 was counted, none of the BSP's other counted interrupt entries (IPI, shootdown, LAPIC timer, reschedule) advanced, and both spans around the halt (IRQ0 entry to `hlt`, IRQ0 entry to sample) stayed under a quarter period; every other loss is charged to the kernel. Each boot prints `SMP:CLOCK:EDGES:TICKS=:LOST=:HOST=:EXTRA=:SPAN_US=:US=:OWN_US=`. The verdict is unchanged: `SMP_CLOCK_DRIFT` still judges the median of five raw segments within 1 %, and only counted IRQ0s advance the clock (ADR-053).
4. **Negative fixtures**, each ending with its own marker and fail-closed otherwise: `sync-stall` (one AP dies after counting: `SMP:SYNC:STALLED`), `sync-slow` (one AP crawls: `SMP:SYNC:TIMEOUT` at the cap), `sync-lost-update` (one lost increment: `SMP:SYNC:LOST_UPDATE`), `clock-masked` (the kernel masks IRQ0 for 3 ms on every 32nd tick: the loss is charged to the kernel and `SMP:CLOCK:DRIFT:KERNEL` follows) and `clock-divisor` (PIT programmed with twice the divisor: `SMP:CLOCK:PIT_COUNT`). The matrix becomes 71 cases × 50 (3 550 guests); verifiers and the aggregate job name follow.
5. **Workflow.** `native-smp.yml` gains the dispatch input `margin_diagnostic` (chosen cases × N boots on the matrix runner, evidence labelled `DIAGNOSTIC-NOT-ACCEPTANCE`) and a rehearsal of `tools/gate.sh` on the final-gate runner. Neither produces acceptance output; every acceptance job is skipped in that mode.

New rejection paths, all in the strict direction: `SMP_CLOCK_PIT_COUNT` (a latched PIT count outside 1..=1193), a missing, repeated or malformed sync-wait, margin or edge line, and a full margin table. Unchanged: scratch 500 ms, clock 1 %, `LOCK_STUCK_MS` 400 ms, shootdown 100 ms, the `SMP:TIMER:RATIO` band, the host watchdogs, the repetition count and every existing negative case.

Considered and not adopted: rejecting a recorded lock wait above 400 ms in the oracle. Such a record without a guest `SMP:LOCK:STUCK` arises when the waiter's own vCPU thread was descheduled while the lock was handed to it; rejecting it would add a host-scheduling failure that the guest check never had.

**Cost of the instrumentation.** An interleaved comparison on one host (QEMU 8.2.2, case `four`, 40 boots per arm, one boot at a time in the order A B B A) of the base `67e592c` against the final source gave 36/40 and 38/40 passes under the unchanged rules. All six failures were `SMP_CLOCK_DRIFT`; the median boot time was 12.82 s and 12.81 s. No cost of the margin records or of the clock sampling is visible (Fisher exact p = 0.68); a sample of this size excludes only large effects. The higher local failure rate of the `cc1043f` series (34 of 90) against the `ed05150` series (4 of 80), measured hours apart, is therefore attributed to host load and not to the sampling. The bookkeeping inside contended lock sections (one timestamp and a table scan per contended acquisition) was not measured separately.

### Evidence (diagnostic; one row per boot in `timing-investigation/repair/boots.tsv`)

| Environment | Boots | Passed (unchanged rules) | Raw clock median | Median without host-lost periods | Sync wait max (limit 2 000 ms) | Scratch max (limit 500 ms) | Lock wait max (limit 400 ms) |
|---|---|---|---|---|---|---|---|
| Hosted run 43286858831, QEMU 10.2.1, matrix runner, `cc1043f`: `four`, `eight`, `nine-capacity`, `m3-load-two`, `m3-load-eight` × 50 | 250 | 250 | −0.14 … +0.55 % | −0.14 … −0.01 % | 901 ms | 263 ms | 28 ms |
| Local, QEMU 8.2.2, 4 host CPUs, `cc1043f`: `four`, `eight`, `nine-capacity` × 30 | 90 | 55 (34 `SMP_CLOCK_DRIFT`, 1 `SMP:TIMER:RATIO`) | +0.13 … +6.22 % | −0.22 … +0.16 % | 423 ms | 408 ms | 100 ms |
| Local, QEMU 8.2.2, 2 host CPUs, `cc1043f`, host watchdog raised to 600 s: `four` × 12, `eight` × 3, `nine-capacity` × 3 | 18 | 12 (`SMP_CLOCK_DRIFT`, `SMP:SYNC:TIMEOUT` at the cap, `SMP:LOCKPAIR:TIMEOUT`, `SMP_ICR`, 2 `SMP:TIMER:RATIO`) | +0.17 … +1.16 % | −0.11 … −0.02 % | 2 569 ms | 215 ms | 158 ms |

Observations that bear on the decisions below:

- Hosted, the sync and scratch waits are heavy-tailed only on the 8-vCPU guests: scratch median 15–56 ms, 90th percentile 77–177 ms, maximum 199–263 ms; sync median 1–2 ms with one boot at 901 ms. On `four` and `m3-load-two` the same waits stay at or below 5 ms. The longest run without sync progress was 1 ms hosted and 14 ms locally, against the 2 s liveness window.
- Hosted, 59 of 250 boots lost at least one PIT period (at most 29 of 5 001, +0.55 %); in the three incidents the loss exceeded 1 %. After removing host-charged periods all 397 medians measured with the accounting (250 hosted, 147 local) lie between −0.22 % and +0.16 % of nominal.
- A counted IRQ0 beyond the elapsed periods (`EXTRA`) occurred in 3 of 250 hosted boots and 9 of 147 local boots, always one tick, in boots without any other anomaly. It cannot be a fatal condition.
- On 2 host CPUs an 8- or 9-vCPU guest needs 210–270 s per boot against the 60 s watchdog; one boot was still progressing when the 20 s cap ended the sync wait, and the lock-pair and ICR limits failed as well. That profile is outside what the fixture can judge.

### Decision of 13 November 2026 and its implementation

The project owner approved D1–D5 in the direction below, on the condition that every existing correctness and security invariant stays intact; a part whose condition cannot be shown is not to be applied. The options of 12 November follow this section unchanged, for the record. Implementation: commit `c47ab74` and the ICR repair after it on PR #317; verification in [TIMING-REPAIR.md](verification/k06-smp-sched/TIMING-REPAIR.md), section 7.

- **D1 (sync).** Every AP must show its own progress (a single-writer counter per CPU, advanced by each stress round and serial line) within 2 s, or the guest stops with `SMP:SYNC:STALLED:CPU=<n>`; the wait ends at 20 s (`SMP:SYNC:TIMEOUT`). The oracle rejects a wait of 2 000 ms or more on every case except `eight`, `nine-capacity`, `m3-eight`, `m3-migrate-eight` and `m3-load-eight`, a fixed list in `tools/run_smp.py` (`OVERCOMMITTED`), so the verdict does not depend on the machine that evaluates it (review finding C3). On those five cases the time is recorded, not judged: the check of eight contending CPUs against 2 s moves to an environment with at least as many dedicated cores as vCPUs, or KVM/hardware (H06).
- **D2 (scratch, spec 01).** Every fill round and every KERNEL round advances the CPU's own counter. 500 ms without own progress stops the guest (`SMP:SCRATCH:STALLED:CPU=<n>`), 5 s is the cap (`SMP:SCRATCH:TIMEOUT`); both quarantine as before. The PASS line carries `WAIT_MS` and `STALL_MS`, and the oracle judges spec 01's 500 ms on every case except the five above. New fixtures `scratch-stall` (AP1 dies mid-fill) and `scratch-slow` (AP1 crawls to the cap). The lock-pair wait keeps its 2 s wall-clock bound: hosted maximum 4 ms, no tail observed.
- **D3 (clock, ADR-053 and 1 % unchanged).** The witness gaps named under D3 below are closed: the IRQ0 stub reads the TSC before the LAPIC MMIO access, the halt stamp is taken immediately before `sti`, and every AP wake, and every AP not in its idle `hlt`, makes the sample non-receptive (F02 (d)). A span below a quarter period cannot merge two edges, because a merge needs IF=0 for at least a whole period; the threshold is therefore a sufficient condition, not a fitted value (F02 (e)). The kernel judges the median of the five segments without the host-charged periods, within 1 % on both sides, and fails when the host credit exceeds a tenth of the segment in a majority of the five segments (`smp_lock::clock::judge`, host-tested, including doubled ticks and the bound; a bound on the median segment alone would fail a healthy boot whose single host-stalled segment lands on the median, review finding K1); kernel-charged loss stays in the judged value, `EXTRA` reduces the host credit, and the raw median is reported beside it. The oracle rechecks the judged median against the pinned PIT period. New fixture `clock-other-cpu` (AP1 masks IRQ0 in the PIC for three periods on every 32nd tick) must fail as kernel loss, like `clock-masked`. The proposed native cases `clock-doubled` and `clock-host-bound` are host tests of `judge` instead; `clock-other-irq` is replaced by `clock-other-cpu`, which exercises the same receptive clause through the AP counters.
- **D4.** `SMP:TIMER:RATIO` band unchanged.
- **D5.** Host watchdog of the `m3-load` outcome 125 s (2.5 × the hosted maximum of 48.3 s, rounded up). Guest deadlines, the 30 s load phase and its per-CPU checks are unchanged; the per-boot time stays in the evidence and in the aggregate's margin summary.
- **Same principle, outside D1–D5 (defect repair).** The ICR exchange (audit F19) bounded each acknowledgement by 65 536 PAUSEs, a loop count whose duration was the host's (inventory site N01, rated high exposure); it failed a healthy local `eight` boot. Each acknowledgement now has a 2 s TSC bound; a lost or misrouted 0xe8 still never arrives and ends the phase as before, and `icr-race-control` must still report it. No normative limit is involved.
- **Matrix.** 74 cases × 50 (3 700 guests): `clock-other-cpu`, `scratch-stall` and `scratch-slow` join; verifiers and the aggregate job name follow.

### Options as proposed on 12 November 2026 (superseded by the decision above)

**D1 — sync wait on guests the runner cannot run in parallel.** The 2 s bound measures the host scheduler once runnable vCPUs exceed runner CPUs. Options: (a) keep 2 s everywhere (as implemented); (b) on multi-thread TCG cases with more vCPUs than runner CPUs (`eight`, `nine-capacity`, `m3-eight`, `m3-migrate-eight`, `m3-load-eight`) judge liveness only (no progress for 2 s and the 20 s cap stay fatal) and keep 2 s on every other case, with the runner's CPU count measured by `run_smp.py`, stored in the evidence and replayed by the verifier; (c) run those five cases on a runner with at least eight dedicated cores; (d) single-thread TCG for them, which removes real parallel interleavings. Recommended: (b). Consequence to accept: no acceptance environment then checks eight contending CPUs against 2 s; that check moves to KVM or hardware evidence (H06).

**D2 — scratch phase (spec 01, 500 ms).** Same mechanism and the same options. Option (b) is not implemented: the scratch fill rounds advance no shared counter, so a per-CPU progress counter and the cases `scratch-stall` and `scratch-slow` come first. Recommended: (b). The lock-pair wait (2 s) has the same shape and would follow the same rule (hosted maximum 4 ms; local 422 ms).

**D3 — idle clock precision (ADR-053, 1 %).** Options: (a) keep the raw median (as implemented); (b) judge the median of the five segments after removing periods that the accounting charged to the host, with the unchanged two-sided 1 % limit, a bound of 10 % on the host-charged share of that median segment (2.5 × the worst incident segment, +3.4 %; a stall confined to one or two segments stays harmless, as today), kernel-charged loss left in the judged value, and `EXTRA` reducing the host credit instead of failing; (c) an isolated runner; (d) the elapsed-PIT clock design in `CLOCK-INVESTIGATION.md`, which changes ADR-053. Recommended: (b). Stated limits of (b): the witness covers the BSP's own interrupt entries and two measured spans; a kernel defect that loses edges through another CPU (PIC mask, PIT rewrite) would be charged to the host up to the bound, and the time before the IRQ0 entry stamp and after the halt stamp is not measured. Additional negative cases are required first (below).

**D4 — `SMP:TIMER:RATIO` band.** No change requested. Hosted values are 62 527–65 378 in 3 551 boots against 50 000–75 000; four local boots on a loaded development host left the band (77 263–78 784).

**D5 — host watchdog of the `m3-load` cases.** `m3-load-eight` uses 46.0–48.3 s of 60 s, of which 30 s is the fixed load phase; the 2.5 × rule that ADR-081 applies to the IPC watchdog is not met. Options: keep, or set the host watchdog of the `m3-load` outcome to 120 s (no guest deadline changes). Recommended: 120 s.

**Negative cases required before D1–D3 take effect** (each with its marker, a host-oracle test and 50/50): `scratch-stall`, `scratch-slow`; an oracle test that a slow sync wait is rejected on a case the runner can run in parallel and accepted only on the listed cases with a stored runner size; `clock-other-irq` (loss while another handler runs on the BSP must be charged to the kernel); `clock-doubled` (one synthesised tick per segment must fail the lower bound); `clock-host-bound` (host-charged loss above the bound must fail). The cases `sync-deadlock` and `sync-starved` of the earlier proposal are covered in part by `sync-stall`; a deadlock inside the BSP's own share still ends in the 60 s host watchdog without a marker.

**Verification order if approved.** (1) Implement the approved parts with their negative cases. (2) One hosted diagnostic over all 71 cases with a few boots each, to obtain the margin of every wait on the matrix runner. (3) Only then one full matrix. No selective retry counts as acceptance at any step.

**Statistics, stated plainly.** Five timing failures in the three full matrices give 1.7 expected failures per matrix (exact 95 % interval 0.5–3.9), a pass probability of about 19 % (2–58 %) for the unchanged contract. A pass probability of 99 % for 3 550 guests needs a per-guest false-failure rate of at most 2.8 × 10⁻⁶; showing that at 95 % confidence takes about 1.06 million failure-free boots, roughly 300 clean matrices, so no available sample can demonstrate it. After D1–D3 the known mechanisms are removed by construction, but the incident history shows a new rare class in almost every full matrix: three classes were seen exactly once in about 9 800 guests, which puts the probability of yet another class at an order of one per matrix (pass probability about 35 %), while the optimistic assumption that the mechanism list is complete gives 85–90 %. The 250 clean hosted boots do not separate these estimates.

**Risks and open items.** The liveness window, the cap, the receptive-span threshold and the host-loss bound are engineering limits, not proofs. TCG results do not transfer to KVM or hardware. The normative step texts live on branch `docs/k06-step-specs` and must receive the approved changes when that branch is integrated. A review by an AI is not an independent human audit.


## ADR-B03K3 — Authorised synthetic policy-transition experiment

**Date:** 14 November 2026. **Scope:** issue #41; TEST keys and disposable
QEMU state only. The project owner authorised publication of the reviewed
transition work and implementation of the explicitly bounded experiment after
the proposal and its evidence limits were presented. PR #328 had already been
squash-merged; missing work therefore continues on `feat/issue41-policy-test`.

**Approved experimental design:** fixed, distinct root/release roles; the
canonical policy and image-binding envelopes in
[B03-POLICY-TRANSITION-PROPOSAL.md](implementation/B03-POLICY-TRANSITION-PROPOSAL.md);
complete operation identity; monotonically enforced policy/image floors;
permanent revoked-pair tombstones; eligible, policy-bound fallback; and
fail-closed handling of uncertain durable state. The synthetic experiment may
use deliberately public TEST seeds. Both signature algorithms remain required.

**Execution order:** first implement and negatively test the bounded `no_std`
codec with the existing reviewed crypto dependencies, behind an explicit
`experimental-policy-test` feature. Then execute the same verifier on the
project's own native path with TEST fixtures. Freeze the exact native ingress,
anchor encoding, write/flush/readback semantics and fault oracle before wiring
a persistent policy transition. Retain all 18 existing update acceptance stages
and the missing-production-policy compiler guards. No new ordinary-app rights,
implicit kernel policy parser or production-profile activation follows.

**D1–D7 disposition:** the experiment's roles, format, transition rules, fixed
root, release-version semantics and TEST-only activation scope are approved.
Production custody/ceremony, a concrete rollback-resistant backend, production
ingress, B04 enrolment/boot/root recovery, trusted freshness/calendar expiry
and independent release-chain review remain unresolved. An emulator variable
or host-controlled anchor can expose behaviour and counterexamples; it cannot
prove physical persistence or whole-state rollback resistance.

**Evidence:** the preceding 92-check symbolic model is a finite specification
exercise, not native crypto/storage evidence. Each implementation checkpoint
records its actual source and executed checks separately. No acceptance point
is awarded by this approval. Real key generation, production signing, release
publication and physical-device changes remain outside this authorisation.

**Revision criteria:** reassess after native interruption/replay results, a
concrete backend or ingress contract, a proposed production trust change, or
independent review. Never reinterpret experimental success as production
acceptance. Reverting the experimental feature and its fixtures leaves the
existing TEST update path and missing-policy guards intact.

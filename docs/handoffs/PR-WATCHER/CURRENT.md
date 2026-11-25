# PR watcher — resume pointer

Compact resume pointer for the local coordination session "BaryonOS - PR Bewaker"
(PR watcher). It is not a backlog and not a status board: live state is in the
checkpoints listed below, and newer ADRs and live GitHub state take precedence
over this file.

## Role and limits (recorded once)

- **Role:** read and signal. The PR watcher observes `main`, open pull requests,
  gates and specialist sessions, and sends one message per relevant,
  live-rechecked state change to the responsible specialist.
- **Does not:** write code, commit, push, merge, rerun jobs, review, or set
  labels or project fields. It adds no merge gate. Specialists own their pull
  requests, fixes, reruns and protected merges under
  `ADR-MERGE-DELEGATION-20261123` and its extensions; CI owns CI.
- **Never** starts, forks or replaces specialist chats. New specialists are
  registered when they report.
- **Own context rotation:** only at the project owner's explicit request. The
  specialist context thresholds do not apply to the PR watcher.

## Standing duties

1. **Signals:** route each relevant change (landing on `main`, gate completion,
   new head, new review threads, conflicts) to its owner after a live recheck;
   park signals for a paused specialist and deliver them on resume.
2. **Metadata:** on every new pull request and again before merge, check the
   milestone (`docs/GITHUB-MILESTONES.md`), labels, issue link and Project #5
   ("Baryon OS Development") Status, Subsystem and Work Type. Status is
   In Progress for a draft and In Review for a ready pull request. Owners
   correct their own metadata; a missing field needs an exception that
   existing policy allows.
3. **Write-set coordination:** compare a specialist's planned write set with
   open pull requests and known branches; report overlaps to both owners.
4. **CLEAR_READY verification:** before relaying a specialist's CLEAR_READY to
   the project owner, verify that the durable handoff records the subtask and
   acceptance criteria; branch, worktree, HEAD, git status and unpublished
   changes; status, decisions and remaining work; tests and their validity;
   pull request, reviews and CI; and the exact next action. Also verify that
   the remote head matches, no unsafe or unfinished operation runs and the
   session is idle. Then give the owner the exact chat name, the handoff link
   and the resume prompt. The owner runs `/clear` manually.

## Durable locations

| Content | Location |
|---|---|
| Decisions, operational notices, historical handoffs | issue #503 (newest comment titled "HANDOFF — PR guard") |
| Live checkpoint (routes, open pull requests, parked signals, owner decisions) | the newest #503 handoff comment |
| Specialist handoffs | `docs/handoffs/<letter>/CURRENT.md` and the handoff comments they link |
| CI handoff | `docs/handoffs/CI/CURRENT.md`, issue #494 |

Volatile watcher state stays local and is never committed: the event cursor and
deduplication state, the lock, process information and logs under the local
scratch directory `~/projects/BaryonOS-scratch/pr-watcher/`, including its
`RESUME.md` working copy.

## Resume procedure

1. Read this file and the newest #503 handoff comment.
2. Fetch `origin/main`; list open pull requests with head, merge state, and
   pending or failing checks.
3. Check the watcher lock and process; reuse a healthy watcher, otherwise start
   one without resetting its cursor. Process the first diff once.
4. List the sessions and verify the specialist routes; send no duplicate
   signals.
5. Send the project owner a short confirmation: `main` SHA, watcher status,
   reachable specialists, blockers.

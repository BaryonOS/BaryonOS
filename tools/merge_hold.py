#!/usr/bin/env python3
"""Required `merge-hold` check: block merging while out-of-band acceptance is pending.

A pull request labelled `acceptance-pending` passes only when a trusted
participant (author association OWNER, MEMBER or COLLABORATOR) has commented
the exact line `acceptance-released: <head commit SHA>`. Labels and comments
are read live through the REST API, so event ordering and re-runs cannot
replay a stale decision. A new push changes the head SHA and re-arms the hold
automatically. Any API or input error fails closed. See ADR-MERGE-HOLD-20261113.

On a `merge_group` event the check decides as late as it can. It first waits
until the other required checks (`host-quality`, `qemu-smoke`) have completed on
the group commit and, under an enforcing plan, until no applicable gate run of
that commit is open or missing, then walks the queue entries ahead from the nearest outward:
each must have left the queue or completed `host-quality`, `qemu-smoke` and
`merge-hold` on its own commit. The walk ends at the first entry whose
`merge-hold` concluded `success`; that run waited the same way, so every entry
ahead of it had finished. A `merge-hold` that ended otherwise proves nothing
about the entries ahead of it, so those are checked too. The queue merges in
order, so this group can merge only after those entries; labels and comments
are therefore read close to this group's merge, not when its own checks end.
A hold added after that final read, until GitHub merges the group, is not seen
(ADR-MERGE-HOLD-20261113). The 45-minute budget is counted from before the
plan: the plan, the group's own checks and gate runs and the whole chain ahead
(each position adding up to one poll) share it, less a 3-minute reserve kept for
the decision after the wait (see merge_group_hold).
A group whose commit has left the queue (ejected or rebuilt) can never merge;
once its own checks have completed, every poll reads the queue first, also
while gate runs are open, and two consecutive well-formed reads, one poll
apart, that both lack it end the wait with a failure. Under an enforcing plan
a gate of the queue commit that has already failed ends the wait too: a newest
gate run that completed without success, or a completed check run with a
blocking conclusion (any conclusion but success for an expected gate), makes
one full gate report, and a Failed line in it fails the group with that report.
It then evaluates
every pull request whose changes the merge-queue commit contains: the queue
entry whose commit is the group head plus
every entry ahead of it that has not merged yet (a merged one is already on
`main`). Each must pass the same decision for its own head
SHA. A released hold additionally requires the tree of that pull request's own
queue entry commit (the commit `main` moves to when it merges) to equal the
released head tree, so acceptance never carries over to a tree it did not run on.

Applicable gates (phase 3 of issue #494, docs/CI-CONTRACT.md "Enforcement").
On `pull_request` the check also recomputes the CI impact plan with the copy
of `tools/ci_impact.py` that sits next to this file; merge-hold.yml extracts
both files from the base commit, so a pull request cannot change how its own
gates are chosen or judged. Every selected `core` or `rwa` gate whose workflow
GitHub starts for the change must have concluded `success` in the newest
`pull_request` run of its workflow for the pull request head, with matching
repository, head repository, branch, event and workflow path. Missing,
running, failed, cancelled, timed-out or skipped results block; a failed
`core` or `rwa` job that the plan did not expect blocks too. "Not applicable"
(filtered, other branch, condition false) is reported, never counted as a pass.
With a positive `policy.budgets.pull_request_wait_minutes` it waits for running
gates, polling with one run listing per minute, and decides once none is open or
the budget ends; with 0 (the catalogue value since #494 B9) it decides at once
and holds no runner while gates run. Gates still pending at the decision are
listed and fail the check. merge-hold-recheck.yml (tools/merge_hold_recheck.py)
re-runs this job once every gate run of the head has finished; a manual re-run
remains the fallback.
`policy.mode = "shadow"` in the base catalogue reports gates without blocking on
them.

On `merge_group` (D1 option B step 2, ADR-CI-GATE-ENFORCEMENT-20261124) every
applicable gate is judged on the queue commit itself. BASE_SHA (the commit the
two base files came from) must equal `merge_group.base_sha` and TESTED_SHA (the
checkout) `merge_group.head_sha`, both well-formed, and the base must be an
ancestor of the queue commit. The plan is recomputed with `--event=merge_group`
(mode full) and bound to that event, base, queue commit and `main` before any
wait, so a plan error fails at once. After the wait one full gate report on the
queue commit decides with the `pull_request` rules (newest run and latest
attempt, blocking conclusions, unexpected failed gates, hidden inputs, shadow
policy); any pending or failed line fails the group and the queue removes it.
A run counts only with event `merge_group`, head SHA the queue commit, head
branch the queue ref without `refs/heads/`, and repository and head repository
this repository; its `pull_requests` list is not used. GitHub's
`head_branch` and `head_repository` of a `merge_group` run are bound fail-closed
but have not been observed live yet: the first queue trial must capture one real
run payload. Every member still needs its hold decision, a released hold an
entry tree equal to the released head tree, and a successful `pull_request`
merge-hold on its own head (defense in depth). An unheld member may land a tree
other than its head's only when the queue-commit report was enforced and clean
(no Pending or Failed line), because only then does it cover that tree; under
`shadow`, or with any Pending or Failed line, its entry tree must equal its head
tree as before step 2. Under `shadow` the group neither waits for gate runs nor
blocks on them, as before step 2.
"""

import json
import os
import re
import subprocess
import sys
import tempfile
import time
import tomllib
from urllib import request

LABEL = "acceptance-pending"
RELEASE_PREFIX = "acceptance-released: "
TRUSTED = {"OWNER", "MEMBER", "COLLABORATOR"}
SHA = re.compile(r"[0-9a-f]{40}")
MAX_PAGES = 50
# Check runs of one commit (#494). Only Actions runs count (app_id filter): a queue commit gets
# the jobs of every merge_group workflow, 77 on a comparable main push (25 Nov 2026) and at
# most 158 with every matrix expanded. A commit that becomes main's head adds its push runs
# (at most 157) and workflow_run "recheck" runs. Five pages keep that headroom and bound one
# read at 5 requests, not 50, against the 1,000 requests/hour GITHUB_TOKEN limit. Past the
# cap the read fails closed.
CHECK_RUN_MAX_PAGES = 5
API = "https://api.github.com"
# Required checks merge-hold waits for on merge_group (ci.yml job names, run by
# GitHub Actions, app id 15368). The wait budget stays below the job timeout.
WAIT_CHECKS = ("host-quality", "qemu-smoke")
# An entry ahead has finished once these ran on its own commit; "merge-hold" is
# this check (merge-hold.yml job name); a successful run waited for the entries ahead.
AHEAD_CHECKS = WAIT_CHECKS + ("merge-hold",)
ACTIONS_APP_ID = 15368
POLL_SECONDS = 60
WAIT_SECONDS = 45 * 60
# Kept back from the merge_group budget for the work after the wait (merge_group_hold); at
# least one poll interval, because the wait may end one sleep past its deadline.
DECISION_RESERVE_SECONDS = 3 * POLL_SECONDS
# shortcut: the pr-<N>-<sha> suffix is observed GitHub practice, not documented;
# an unexpected ref fails closed and stalls the queue instead of merging.
QUEUE_REF = re.compile(r"refs/heads/gh-readonly-queue/main/pr-([1-9][0-9]*)-[0-9a-f]{40}")
QUEUE_QUERY = """query($owner: String!, $name: String!) {
  repository(owner: $owner, name: $name) {
    mergeQueue(branch: "main") {
      entries(first: 100) {
        pageInfo { hasNextPage }
        nodes {
          position
          headCommit { oid }
          pullRequest { number state baseRefName headRefOid }
        }
      }
    }
  }
}"""


GATE_WORKFLOW = ".github/workflows/merge-hold.yml"
CATALOGUE = ".github/ci-impact.toml"
SELECTABLE = ("core", "rwa")
# Fallbacks whose listing can be incomplete or that do not describe this pull request.
UNUSABLE_FALLBACKS = ("unknown-event", "invalid-sha", "fetch-failed", "merge-ref-mismatch",
                      "bootstrap-no-base-catalogue", "invalid-trusted-catalogue", "no-trusted-base")
# A gate that ended like this did not pass; "skipped" also blocks an expected gate.
BLOCKING_CONCLUSIONS = ("failure", "cancelled", "timed_out", "startup_failure", "action_required", "stale")
SELECTOR_TIMEOUT = 600
MAX_WAIT_SECONDS = 45 * 60
# As tools/ci_impact.py FILTER_FILE_LIMIT: GitHub evaluates a path filter on the first 300 files only.
FILTER_FILE_LIMIT = 300
NEVER_STARTS = "so GitHub may never start it"
PLAN_ID = re.compile(r"sha256:[0-9a-f]{64}")
EXPRESSION = re.compile(r"\$\{\{.*?\}\}")


def decide(labels, comments, head_sha):
    """Return (passed, message) for one pull request head."""
    if not SHA.fullmatch(head_sha or ""):
        return False, "Head SHA is missing or malformed; failing closed."
    if LABEL not in labels:
        return True, "No acceptance hold is set."
    release = RELEASE_PREFIX + head_sha
    for comment in comments:
        if comment.get("author_association") not in TRUSTED:
            continue
        lines = (comment.get("body") or "").replace("\r", "").split("\n")
        if any(line.strip() == release for line in lines):
            return True, f"Acceptance released for {head_sha}."
    return False, (
        f"The '{LABEL}' label is set and no trusted '{release}' comment exists. "
        "Post it only after the required acceptance has passed on this head commit."
    )


def call(token, url, payload=None):
    """One authenticated API request (GET, or POST with a JSON body)."""
    req = request.Request(
        url,
        data=None if payload is None else json.dumps(payload).encode(),
        headers={
            "Authorization": "Bearer " + token,
            "Accept": "application/vnd.github+json",
            "X-GitHub-Api-Version": "2022-11-28",
            "User-Agent": "baryon-merge-hold",
        },
    )
    with request.urlopen(req, timeout=20) as response:
        return json.load(response)


def fetch_all(token, url):
    """Read every page of a REST list endpoint; raise on any failure."""
    items, page = [], 1
    while True:
        batch = call(token, f"{url}?per_page=100&page={page}")
        if not isinstance(batch, list):
            raise TypeError("Unexpected API response")
        items.extend(batch)
        if len(batch) < 100:
            return items
        page += 1
        if page > MAX_PAGES:
            raise ValueError("Too many pages")


def pull_request_hold(token, repo, number, head_sha):
    """Read one pull request live; return (passed, message, labels)."""
    base = f"{API}/repos/{repo}/issues/{number}"
    labels = [item["name"] for item in fetch_all(token, base + "/labels")]
    comments = fetch_all(token, base + "/comments") if LABEL in labels else []
    passed, message = decide(labels, comments, head_sha)
    return passed, message, labels


def commit_oid(node):
    """The entry's commit SHA, or None while it has no commit (nullable in GraphQL)."""
    return (node.get("headCommit") or {}).get("oid")


def queue_nodes(token, repo):
    """Read every entry of the main merge queue live; raise on any doubt."""
    owner, name = repo.split("/")
    answer = call(token, API + "/graphql",
                  {"query": QUEUE_QUERY, "variables": {"owner": owner, "name": name}})
    if answer.get("errors"):
        raise ValueError(f"GraphQL errors: {answer['errors']}")
    entries = answer["data"]["repository"]["mergeQueue"]["entries"]
    if entries["pageInfo"]["hasNextPage"]:
        raise ValueError("merge queue has more entries than one page")
    return entries["nodes"]


class LeftQueue(ValueError):
    """This group's commit is no longer in the merge queue, so the group can never merge."""


class GateFailed(Exception):
    """A gate of the queue commit has already failed; report is the full gate report that shows it."""

    def __init__(self, report):
        super().__init__("an applicable gate failed on the merge queue commit")
        self.report = report


def left_queue(nodes, head_sha, number):
    """True only if this well-formed queue read positively shows the group has left it.

    The list is not empty, no entry has head_sha, every entry names its pull
    request by an integer number, and the group's pull request is absent
    (ejected or merged) or has another built commit (rebuilt). An entry of that
    pull request without a commit may still be this group, so it is not proof of
    absence; unbuilt entries of other pull requests do not matter. An empty list
    may be a lagging listing, so it is not proof either.
    """
    if not nodes or any(commit_oid(node) == head_sha for node in nodes):
        return False
    pulls = [node.get("pullRequest") for node in nodes]
    if not all(isinstance(pull, dict) and isinstance(pull.get("number"), int) for pull in pulls):
        return False
    return all(SHA.fullmatch(commit_oid(node) or "")
               for node, pull in zip(nodes, pulls) if pull["number"] == number)


def own_position(nodes, head_sha, number):
    """Position of the single entry whose commit is head_sha (other entries may be unbuilt)."""
    if left_queue(nodes, head_sha, number):
        raise LeftQueue(f"merge group commit {head_sha} is no longer in the merge queue "
                        "(ejected or rebuilt), so this group can never merge")
    own = [node for node in nodes if commit_oid(node) == head_sha]
    if len(own) != 1:
        raise ValueError(f"no single queue entry has commit {head_sha}")
    position = own[0]["position"]
    if not isinstance(position, int):
        raise TypeError("queue position is not an integer")
    return own[0], position


def group_members(token, repo, head_ref, head_sha):
    """Return the open queue entries whose changes the group commit contains.

    An entry ahead whose pull request has already MERGED into main is skipped:
    it is on main, so no decision here can hold it back, and failing on it
    would only eject this group when GitHub merges the entry ahead just as its
    merge-hold succeeds. Every unmerged entry is still evaluated.
    """
    match = QUEUE_REF.fullmatch(head_ref)
    if not match or not SHA.fullmatch(head_sha):
        raise ValueError(f"unexpected merge group ref {head_ref!r} or SHA {head_sha!r}")
    nodes = queue_nodes(token, repo)
    own, position = own_position(nodes, head_sha, int(match.group(1)))
    if (own.get("pullRequest") or {}).get("number") != int(match.group(1)):
        raise ValueError(f"no single queue entry matches {head_ref} at {head_sha}")
    members = []
    for node in nodes:
        if node["position"] > position:
            continue
        pull = node.get("pullRequest")
        if not pull:
            raise ValueError(f"queue entry at position {node['position']} has no pull request")
        if pull["baseRefName"] != "main":
            raise ValueError(f"queued pull request #{pull['number']} is not an open PR to main")
        if pull["state"] == "MERGED" and node is not own:
            continue
        if pull["state"] != "OPEN":
            raise ValueError(f"queued pull request #{pull['number']} is not an open PR to main")
        if not SHA.fullmatch(commit_oid(node) or ""):
            raise ValueError(f"queue entry for #{pull['number']} has no commit")
        members.append(node)
    return members


def tree(token, repo, sha):
    """Read an identified tree; malformed values cannot prove equal content."""
    tree_sha = call(token, f"{API}/repos/{repo}/git/commits/{sha}")["tree"]["sha"]
    if not isinstance(tree_sha, str) or not SHA.fullmatch(tree_sha):
        raise ValueError(f"commit {sha} has a missing or malformed tree SHA")
    return tree_sha


def completed_runs(token, repo, sha):
    """The completed check runs on sha by name (absent or running counts as not done).

    Every page is read, up to CHECK_RUN_MAX_PAGES: a queue commit runs every gate workflow.
    A run that shifts between pages while paging only counts as not done, so the wait never
    ends early; more pages than the cap raise, and the wait fails closed at its deadline.
    """
    url = (f"{API}/repos/{repo}/commits/{sha}/check-runs"
           f"?app_id={ACTIONS_APP_ID}&filter=latest&per_page=100")
    done, page = {}, 1
    while True:
        batch = call(token, f"{url}&page={page}")["check_runs"]
        done.update({run["name"]: run for run in batch if run["status"] == "completed"})
        if len(batch) < 100:
            return done
        page += 1
        if page > CHECK_RUN_MAX_PAGES:
            raise ValueError("Too many pages")


def end_on_failed_gate(token, repo, sha, plan, queue_branch, checks, newest, judged):
    """Raise GateFailed when the full gate report of the queue commit already has a Failed line.

    A full report costs about two requests per counted run, so it is made only when this
    poll shows a failure not judged before: a newest gate run that completed without
    success, or a completed check run of the commit (checks, by name) with a blocking
    conclusion or, for an expected gate, any conclusion but success. Only the report
    decides, so its Failed lines are those the final report gives; a trigger it does not
    count (an advisory job, a superseded run) is not judged again. judged collects the
    triggers already judged, and only after a report was read.
    """
    expected = expected_gates(plan)
    triggers = {("run", run["id"], run.get("run_attempt")) for run in newest.values()
                if run.get("status") == "completed" and run.get("conclusion") != "success"}
    triggers |= {("check", check.get("id"), name) for name, check in checks.items()
                 if check.get("conclusion") in BLOCKING_CONCLUSIONS
                 or (check.get("conclusion") != "success" and any(job_matches(row, check) for row in expected))}
    if triggers <= judged:
        return
    report = gate_report(token, repo, None, sha, plan, queue_branch=queue_branch)
    judged |= triggers
    if report[2]:
        raise GateFailed(report)


def still_running(token, repo, sha, number, plan, queue_branch, judged):
    """What this group still waits for: its own checks and gate runs, then the entries ahead.

    Once host-quality and qemu-smoke have completed, every poll reads the queue
    first, so a group that has left it raises LeftQueue also while its gate runs
    are open. Under an enforcing plan every gated workflow run of this group's own
    commit must then be closed and every expected gate must have a run (the
    merge_group binding of run_matches) before the walk, and a gate that has
    already failed raises GateFailed (end_on_failed_gate); under `shadow` the gates
    do not block, so they are neither awaited nor judged early.
    Entries ahead are walked from the nearest outward. The walk stops at the
    first entry whose `merge-hold` concluded `success`: only a successful run
    went through this same wait, so every entry ahead of it had finished. A
    `merge-hold` that ended otherwise (failed early, cancelled, runner lost)
    proves nothing about the entries ahead of it, so the walk continues. Each
    poll makes one REST request per check-run page of this commit and of each entry
    walked, one GraphQL query and, under an enforcing plan, one run listing of this
    commit; usually that is one page each. A full gate report is added only for a
    failure not judged before (end_on_failed_gate). "The rate limit for
    GITHUB_TOKEN is 1,000 requests per hour per repository"
    (https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api);
    beyond it reads fail, are retried and, at the deadline, fail closed.
    """
    done = completed_runs(token, repo, sha)
    pending = [f"{name} on {sha}" for name in WAIT_CHECKS if name not in done]
    if pending:
        return pending
    nodes = queue_nodes(token, repo)
    _, position = own_position(nodes, sha, number)
    if plan.get("policy_mode") != "shadow":
        newest = newest_gate_runs(token, repo, None, sha, plan, queue_branch)
        end_on_failed_gate(token, repo, sha, plan, queue_branch, done, newest, judged)
        if gates_open(newest, plan):
            return [f"applicable gate runs on {sha} are still open or have not started"]
    ahead = [node for node in nodes if node["position"] < position]
    for node in sorted(ahead, key=lambda node: node["position"], reverse=True):
        oid = commit_oid(node)
        if not SHA.fullmatch(oid or ""):
            return [f"queue entry at position {node['position']} has no commit yet"]
        done = completed_runs(token, repo, oid)
        pending = [f"{name} on {oid}" for name in AHEAD_CHECKS if name not in done]
        if pending:
            return pending
        if done["merge-hold"].get("conclusion") == "success":
            return []
    return []


def wait_for_checks(token, repo, sha, number, plan, queue_branch, budget=WAIT_SECONDS):
    """Block until this group's checks, its gate runs and the entries ahead have finished; raise at the deadline.

    Return None, or the full gate report (passed, pending, failed) of the queue
    commit when an already failed gate ended the wait early; that report then
    decides, and its Failed lines fail the group.
    Only the timing depends on this: the decision afterwards reads the queue,
    labels and comments live and fails closed. API errors and unbuilt entries
    are retried, because waiting longer never weakens the hold. An entry ahead
    that leaves the queue (merged or removed) no longer counts. Two consecutive
    polls whose well-formed reads both show that this group itself has left the
    queue fail at once; any other poll in between resets that count, because a
    single read may lag behind the queue. The deadline is checked after each poll,
    so a budget of 0 or less still makes one poll: anything still open then fails
    closed without a sleep.
    """
    deadline = time.monotonic() + budget
    absent, judged = 0, set()
    while True:
        try:
            pending = still_running(token, repo, sha, number, plan, queue_branch, judged)
            absent = 0
        except GateFailed as failed:
            return failed.report
        except LeftQueue:
            absent += 1
            if absent >= 2:
                raise
            pending = ["merge group commit not listed in the queue once; reading again"]
        except (KeyError, IndexError, ValueError, TypeError, AttributeError, OSError) as exc:
            absent = 0
            pending = [f"queue or check-run read failed: {exc}"]
        if not pending:
            return None
        if time.monotonic() >= deadline:
            raise ValueError(f"required checks did not complete in time: {pending}")
        time.sleep(POLL_SECONDS)


def git(*args):
    """Run git in the checkout; return (returncode, stripped stdout)."""
    result = subprocess.run(["git", *args], capture_output=True, text=True, timeout=120, check=False)
    return result.returncode, result.stdout.strip()


def recompute_plan(base, head, tested, ref, repo_dir=".", event="pull_request"):
    """Run the selector next to this file (the base copy in CI) for event and return its plan."""
    selector = os.path.join(os.path.dirname(os.path.abspath(__file__)), "ci_impact.py")
    if not os.path.isfile(selector):
        raise ValueError("no tools/ci_impact.py next to merge_hold.py; the applicable gates are unknown")
    with tempfile.TemporaryDirectory() as tmp:
        out = os.path.join(tmp, "plan.json")
        result = subprocess.run(
            [sys.executable, "-B", "-I", selector, "plan", f"--event={event}", f"--base-sha={base}",
             f"--head-sha={head}", f"--tested-sha={tested}", f"--base-ref={ref}", f"--repo={repo_dir}",
             f"--out={out}"],
            capture_output=True, text=True, timeout=SELECTOR_TIMEOUT, check=False)
        if result.returncode != 0:
            raise ValueError(f"the selector exited {result.returncode}: {result.stderr.strip()[-400:]}")
        with open(out, encoding="ascii") as handle:
            return json.load(handle)


def check_plan(plan, base, head, tested, ref, event="pull_request"):
    """Raise unless the plan is a trusted, complete plan for exactly this event, base, head and tested commit."""
    shas = plan.get("shas") or {}
    if (plan.get("event"), plan.get("ref"), shas.get("base"), shas.get("head"), shas.get("tested")) != (
            event, ref, base, head, tested):
        raise ValueError(f"the recomputed plan does not describe this {event} base, head and tested commit")
    catalogue = plan.get("catalogue") or {}
    if not (plan.get("selector") or {}).get("trusted") or catalogue.get("source_sha") != base \
            or not catalogue.get("blob"):
        raise ValueError("the plan was not computed by the base selector from the base catalogue")
    code = (plan.get("fallback") or {}).get("code") or ""
    if code in UNUSABLE_FALLBACKS or code.startswith(("selector-error:", "output-error:")):
        raise ValueError(f"the plan fell back with {code}, so its gate listing may be incomplete")
    if plan.get("mode") not in ("select", "full") or not PLAN_ID.fullmatch(plan.get("plan_id") or ""):
        raise ValueError("the plan has no usable mode or plan_id")
    if not any(row.get("class") == "core" for row in plan.get("selected", [])):
        raise ValueError("the plan selects no core gate")


def expected_gates(plan):
    """Selected core and rwa gates that GitHub starts for this change (merge-hold itself excluded)."""
    return [row for row in plan["selected"]
            if row["class"] in SELECTABLE and row["workflow"] != GATE_WORKFLOW
            and row.get("blocked_by_filter") is not True]


def hidden_input_rows(plan):
    """Selected core and rwa gates whose own path filter hides a registered, non-metadata input."""
    return [row for row in plan["selected"]
            if row["class"] in SELECTABLE and row["workflow"] != GATE_WORKFLOW
            and row.get("blocked_by_filter") is True and row.get("filter_hidden_inputs")]


def unknown_filter_note(plan):
    """Why a gate with an unknown path-filter result may never start; it stays expected (K10)."""
    count = (plan.get("changes") or {}).get("count", 0)
    code = (plan.get("fallback") or {}).get("code")
    if count > FILTER_FILE_LIMIT:
        cause = (f"undefined above {FILTER_FILE_LIMIT} changed files (this change has {count}; "
                 f"GitHub evaluates only the first {FILTER_FILE_LIMIT})")
        remedy = "a smaller pull request or a new head"
    elif code:
        cause = f"unknown: the changed-file list is incomplete (fallback {code})"
        remedy = "a re-run with a usable diff or a new head"
    else:
        cause = "unknown for a change without changed files"
        remedy = "a new head"
    return (f"its path-filter result is {cause}, {NEVER_STARTS}. It stays expected (fail-closed, "
            f"docs/CI-CONTRACT.md K10); if GitHub skipped it, only {remedy} clears it")


def hidden_listing(paths, limit=5):
    # Paths come from the pull request: ASCII-escape them so no control character reaches the log,
    # and escape % because the runner decodes %0A in ::error:: data (the step summary shows %25).
    shown = ", ".join(json.dumps(path, ensure_ascii=True).replace("%", "%25") for path in paths[:limit])
    return shown + (f" and {len(paths) - limit} more" if len(paths) > limit else "")


def fetch_listing(token, url, key):
    """Every item of a paginated Actions listing ({total_count, <key>: [...]}), by id."""
    items, page, total = {}, 1, None
    while True:
        answer = call(token, f"{url}&per_page=100&page={page}")
        batch = answer[key]
        if not isinstance(batch, list) or not isinstance(answer["total_count"], int):
            raise TypeError("Unexpected API response")
        total = answer["total_count"]
        for item in batch:
            items[item["id"]] = item
        if len(batch) < 100:
            break
        page += 1
        if page > MAX_PAGES:
            raise ValueError("Too many pages")
    if len(items) < total:
        raise ValueError(f"listed {len(items)} of {total} items; the listing changed while paging")
    return list(items.values())


def run_matches(run, repo, pull, head_sha, queue_branch=None):
    """Provenance of a workflow run: this repository and this pull request head or merge-queue commit.

    Without queue_branch: the pull_request event, and the run must list this pull
    request (number and base branch) among its pull requests. GitHub fills that
    list for same-repository heads; a fork run has an empty list, cannot be bound
    to a base, and therefore never counts (fail closed: push the fork's commits
    to a repository branch, docs/CI-CONTRACT.md).
    With queue_branch (the queue ref without "refs/heads/"): the merge_group
    event on head_sha, that branch, and this repository as repository and head
    repository. Its pull request list is not used: it is not observed on merge_group.
    """
    if queue_branch is not None:
        return (run.get("event") == "merge_group" and run.get("head_sha") == head_sha
                and run.get("head_branch") == queue_branch
                and (run.get("repository") or {}).get("full_name") == repo
                and (run.get("head_repository") or {}).get("full_name") == repo)
    head = pull["head"]
    head_repo = (head.get("repo") or {}).get("full_name")
    pulls = run.get("pull_requests") or []
    return (run.get("event") == "pull_request" and run.get("head_sha") == head_sha
            and run.get("head_branch") == head["ref"]
            and (run.get("repository") or {}).get("full_name") == repo
            and (run.get("head_repository") or {}).get("full_name") == head_repo
            and lists_pull(pulls, pull.get("number"), pull["base"]["ref"]))


def lists_pull(pulls, number, base_ref):
    """Does a run's pull request list name this pull request and base? An empty list never does."""
    return any(item.get("number") == number and (item.get("base") or {}).get("ref") == base_ref
               for item in pulls)


def run_order(run):
    """Newest first by the start of the latest attempt, then id: a later re-run of an older run counts.

    Without a start time the creation time is used; without both the run cannot be ordered.
    """
    started = run.get("run_started_at") or run.get("created_at")
    if not isinstance(started, str) or not started:
        raise ValueError(f"run {run.get('id')} has no start or creation time")
    return (started, run["id"])


def read_state(run):
    """What two reads of one run must share for the jobs read between them to belong to it."""
    return run.get("id"), run.get("run_attempt"), run.get("status")


def job_matches(row, job):
    # Runtime matrix: the check name is a template (excluded rows carry no variant field).
    if row.get("variant") == "*" or EXPRESSION.search(row.get("check_name") or ""):
        pattern = "".join(".+" if EXPRESSION.fullmatch(part) else re.escape(part)
                          for part in re.split(r"(\$\{\{.*?\}\})", row["check_name"]))
        return re.fullmatch(pattern, job.get("name") or "") is not None
    return job.get("name") == row["check_name"]


def counted_workflows(plan):
    """The workflows whose runs on the head gate_report reads (merge-hold itself never counts)."""
    rows = plan["selected"] + plan["excluded"]
    return sorted({row["workflow"] for row in rows if row.get("workflow")} - {GATE_WORKFLOW})


def gated_workflows(plan):
    """Workflows with a gate GitHub may start: a running run of one must finish, because jobs
    waiting on `needs` or a runtime matrix may not be listed yet."""
    return {row["workflow"] for row in plan["selected"] + plan["excluded"]
            if row.get("class") in SELECTABLE
            and row.get("reason") not in ("event-not-applicable", "condition-false")}


def runs_open(token, repo, pull, head_sha, plan, queue_branch=None):
    """True while a gated workflow still runs for this head or an expected one has no run yet.

    One run listing (usually one page) per poll instead of a full gate_report, so a
    positive wait budget stays far below the GITHUB_TOKEN rate limit. Only the wait
    uses it; the decision is always a full gate_report. A failed read counts as open:
    the wait goes on and the next full report decides, so a transient API error
    neither ends the wait nor fails the job. queue_branch binds runs as in run_matches.
    """
    try:
        newest = newest_gate_runs(token, repo, pull, head_sha, plan, queue_branch)
    except (OSError, ValueError, TypeError) as error:
        # A run without a start or creation time also lands here: the wait goes on and the
        # next full report (at the latest at the deadline) fails closed on it.
        print(f"::warning::merge-hold wait poll failed ({error}); still waiting", file=sys.stderr)
        return True
    return gates_open(newest, plan)


def newest_gate_runs(token, repo, pull, head_sha, plan, queue_branch=None):
    """The newest matching run of each gated workflow for this head, by path (one run listing); raise on doubt."""
    workflows = gated_workflows(plan) - {GATE_WORKFLOW}
    event = "pull_request" if queue_branch is None else "merge_group"
    url = f"{API}/repos/{repo}/actions/runs?head_sha={head_sha}&event={event}"
    runs = [run for run in fetch_listing(token, url, "workflow_runs")
            if run.get("path") in workflows and run_matches(run, repo, pull, head_sha, queue_branch)]
    newest = {}
    for run in runs:  # the newest run decides, as in gate_report; a superseded open run does not count
        order = run_order(run)  # every counted run must be orderable, the first one too
        if run["path"] not in newest or order > run_order(newest[run["path"]]):
            newest[run["path"]] = run
    return newest


def gates_open(newest, plan):
    """True while one of these newest gate runs is open or an expected gate's workflow has no run."""
    return (any(run.get("status") != "completed" for run in newest.values())
            or any(row["workflow"] not in newest for row in expected_gates(plan)))


def gate_report(token, repo, pull, head_sha, plan, queue_branch=None):
    """Return (passed, pending, failed) lines for the applicable gates of this head or queue commit.

    queue_branch binds the runs to a merge-queue commit instead of pull, as in run_matches.
    """
    expected = expected_gates(plan)
    rows = plan["selected"] + plan["excluded"]
    gated = gated_workflows(plan)
    workflows = counted_workflows(plan)
    event = "pull_request" if queue_branch is None else "merge_group"
    url = f"{API}/repos/{repo}/actions/runs?head_sha={head_sha}&event={event}"
    newest = {}
    for run in fetch_listing(token, url, "workflow_runs"):
        path = run.get("path")
        if path in workflows and run_matches(run, repo, pull, head_sha, queue_branch):
            # The newest run decides: an older success never masks a newer run.
            order = run_order(run)  # every counted run must be orderable
            if path not in newest or order > run_order(newest[path]):
                newest[path] = run
    passed, pending = [], []
    # Fail closed: a path filter may hide only metadata, never a registered input.
    failed = [f"{row['check_name']}: its path filter hides registered input(s) "
              f"{hidden_listing(row['filter_hidden_inputs'])}, so the gate did not run"
              for row in hidden_input_rows(plan)]
    for workflow in workflows:
        wanted = [row for row in expected if row["workflow"] == workflow]
        run = newest.get(workflow)
        if run is None:
            pending += [f"{row['check_name']}: no {workflow} run for {head_sha} yet"
                        + (f"; {unknown_filter_note(plan)}" if row.get("blocked_by_filter", False) is None else "")
                        for row in wanted]
            continue
        where = f"run {run['id']} attempt {run.get('run_attempt')}"
        running = run.get("status") != "completed"
        # Every run's jobs are read, also a successful one without an expected gate: a job
        # with job-level continue-on-error can fail inside a run that concludes success.
        # A running run is listed too: a gate GitHub started must finish even when
        # the plan did not select it, or its later failure would go unseen.
        jobs = [job for job in fetch_listing(
            token, f"{API}/repos/{repo}/actions/runs/{run['id']}/jobs?filter=latest", "jobs")
            if job.get("run_id") == run["id"]]
        # A re-run between the two reads would pair this run state with another attempt's jobs.
        again = call(token, f"{API}/repos/{repo}/actions/runs/{run['id']}")
        if read_state(again) != read_state(run):
            # The run moved on (a skipped run that completed, a new attempt). Rather than leave it
            # pending for a later merge-hold run, read its jobs once more and judge that newer state
            # only if the run held it across that read, under the requested id, and is no older than
            # the listing. A run that keeps changing or lags behind the listing stays pending.
            jobs = [job for job in fetch_listing(
                token, f"{API}/repos/{repo}/actions/runs/{run['id']}/jobs?filter=latest", "jobs")
                if job.get("run_id") == run["id"]]
            settled = call(token, f"{API}/repos/{repo}/actions/runs/{run['id']}")
            attempt, listed = settled.get("run_attempt"), run.get("run_attempt")
            if (read_state(settled) != read_state(again) or settled.get("id") != run["id"]
                    or not (isinstance(attempt, int) and isinstance(listed, int) and attempt >= listed)):
                pending.append(f"{workflow}: {where} changed while it was read (now attempt "
                               f"{again.get('run_attempt')}, {again.get('status')}); re-run merge-hold")
                continue
            # Judge the settled record as a whole, never fields of the stale listing.
            run = settled
            where = f"run {run['id']} attempt {run.get('run_attempt')}"
            # The settled record replaces the listed one, so it must keep the listing's binding
            # (run_matches: pull request, or merge_group queue commit); otherwise it never counts.
            # Deliberately unconditional, on pull_request and merge_group alike, and on top of the
            # id and attempt checks above: stricter than either alone, it can only turn a count
            # into a Pending line (fail-closed).
            if not run_matches(run, repo, pull, head_sha, queue_branch):
                pending.append(f"{workflow}: {where} no longer matches {head_sha} when read again; "
                               "re-run merge-hold")
                continue
            running = run.get("status") != "completed"
        if running and workflow in gated:
            pending.append(f"{workflow}: {where} is still {run.get('status')}; its gates must finish, "
                           "including jobs GitHub has not created yet")
        claimed = set()
        for row in wanted:
            found = [job for job in jobs if job_matches(row, job)]
            claimed.update(job["id"] for job in found)
            if not found and running:
                pending.append(f"{row['check_name']}: not started yet in {where}, which is {run.get('status')}")
            elif not found:
                failed.append(f"{row['check_name']}: missing from {where} ({workflow})")
            elif any(job.get("status") != "completed" for job in found):
                pending.append(f"{row['check_name']}: still running in {where}")
            elif all(job.get("conclusion") == "success" for job in found) and (
                    len(found) == 1 or (row.get("variant") == "*" and not running)):
                passed.append(f"{row['check_name']}: success in {where}")
            elif row.get("variant") == "*" and running and all(job.get("conclusion") == "success" for job in found):
                # A runtime matrix may still create variants: neither a pass nor a failure yet.
                pending.append(f"{row['check_name']}: {len(found)} variant(s) succeeded so far in {where}, "
                               f"which is {run.get('status')}")
            else:
                outcome = ", ".join(sorted({str(job.get("conclusion")) for job in found}))
                failed.append(f"{row['check_name']}: {outcome} in {where} ({workflow})")
        for job in jobs:
            # A rendered runtime-matrix name inherits its template row's class.
            klass = next((row["class"] for row in rows if row.get("workflow") == workflow
                          and row.get("check_name") and job_matches(row, job)), "unregistered")
            if job["id"] in claimed or klass not in SELECTABLE + ("unregistered",):
                continue
            if job.get("status") != "completed":
                pending.append(f"{job.get('name')}: {job.get('status')} in {where} ({workflow}); "
                               "not expected by the plan, but a started gate must finish")
            elif job.get("conclusion") in BLOCKING_CONCLUSIONS:
                failed.append(f"{job.get('name')}: {job.get('conclusion')} in {where} ({workflow}); "
                              "not expected by the plan, but a failed gate is never ignored")
    return passed, pending, failed


def pull_request_gates(token, repo, number, head_sha):
    """Return (passed, lines) for the applicable gates of one pull request head."""
    pull = call(token, f"{API}/repos/{repo}/pulls/{number}")
    if pull["head"]["sha"] != head_sha:
        raise ValueError(f"the pull request head moved to {pull['head']['sha']}; its own run decides")
    base, tested, ref = os.environ["BASE_SHA"], os.environ["TESTED_SHA"], pull["base"]["ref"]
    for label, sha in (("base", base), ("tested", tested)):
        if not SHA.fullmatch(sha):
            raise ValueError(f"{label} SHA is missing or malformed")
    plan = recompute_plan(base, head_sha, tested, ref)
    check_plan(plan, base, head_sha, tested, ref)
    enforce = plan.get("policy_mode") != "shadow"
    lines = [f"Plan {plan['plan_id']}: mode {plan['mode']}"
             + (f", fallback {plan['fallback']['code']}" if plan.get("fallback") else "")
             + f", base {base}, head {head_sha}, merge commit {tested}, target {ref}."]
    if git("merge-base", "--is-ancestor", base, head_sha)[0] != 0:
        return False, lines + [
            f"::error::The head {head_sha} does not contain the base tip {base}, so the gate runs did not "
            "test the tree that would merge. Update the branch; the new head is checked again."]
    budget = min(int((plan.get("budgets") or {}).get("pull_request_wait_minutes", 0)) * 60, MAX_WAIT_SECONDS)
    deadline = time.monotonic() + budget
    fresh = False  # the last report was made after the runs were seen closed
    while True:
        passed, pending, failed = gate_report(token, repo, pull, head_sha, plan)
        if failed or not pending or time.monotonic() >= deadline:
            break
        if not runs_open(token, repo, pull, head_sha, plan):
            # A gate may have finished during the report: decide once more. Pending again
            # with nothing left running (a run that kept changing while it was read) cannot
            # resolve by waiting, so decide now.
            if fresh:
                break
            fresh = True
            continue
        while time.monotonic() < deadline:
            time.sleep(POLL_SECONDS)
            if not runs_open(token, repo, pull, head_sha, plan):
                break
        # A new attempt may start and finish during the next report: allow one refresh again.
        fresh = False
    prefix, report, tail = gate_lines(plan, passed, pending, failed)
    lines += report
    if pending and not failed:
        run = os.environ.get("GITHUB_RUN_ID", "<run id>")
        lines.append(f"{prefix}For a pull request from this repository the merge-hold re-check "
                     "(merge-hold-recheck.yml) usually re-runs this job when the pending gates "
                     "finish; a failure with only Pending lines needs no fix. If it is not re-run, "
                     "re-run it once the gates have finished "
                     f"(gh run rerun {run} --failed).")
        if any(NEVER_STARTS in line for line in pending):
            lines.append(f"{prefix}Exception: a gate with an unknown path-filter result may never start, "
                         "so waiting for it does not help; its Pending line names the remedy.")
    lines += tail
    return (not failed and not pending) or not enforce, lines


def gate_lines(plan, passed, pending, failed):
    """(prefix, summary with Failed and Pending lines, Passed and Not-a-pass lines); shadow only warns."""
    skipped = [f"{row['check_name']}: not applicable (path filter)" for row in plan["selected"]
               if row["class"] in SELECTABLE and row.get("blocked_by_filter") is True
               and not row.get("filter_hidden_inputs")]
    prefix = "::error::" if plan.get("policy_mode") != "shadow" else "::warning::(shadow, not blocking) "
    report = [f"{len(passed)} applicable gate(s) passed; {len(skipped)} filtered out; "
              f"{len(pending)} pending; {len(failed)} failed."]
    report += [prefix + "Failed: " + line for line in failed]
    report += [prefix + "Pending: " + line for line in pending]
    return prefix, report, ["Passed: " + line for line in passed] + ["Not a pass: " + line for line in skipped]


def base_wait_seconds(base_sha):
    """merge_group wait budget from the base catalogue; the fixed maximum when it is unreadable."""
    code, text = git("cat-file", "blob", f"{base_sha}:{CATALOGUE}")
    try:
        minutes = tomllib.loads(text)["policy"]["budgets"]["merge_group_wait_minutes"] if code == 0 else None
    except (tomllib.TOMLDecodeError, KeyError, TypeError):
        minutes = None
    if not isinstance(minutes, int) or isinstance(minutes, bool) or not 0 <= minutes * 60 <= MAX_WAIT_SECONDS:
        return WAIT_SECONDS
    return minutes * 60


def pull_request_gates_passed(token, repo, pr_head, number, base_ref="main"):
    """True when the newest pull_request merge-hold run of this pull request on pr_head succeeded.

    Used for merge_group members. The run must list this pull request's number
    and base branch, as in run_matches; an empty list (a fork head) never counts.
    """
    url = f"{API}/repos/{repo}/actions/runs?head_sha={pr_head}&event=pull_request"
    runs = [run for run in fetch_listing(token, url, "workflow_runs")
            if run.get("path") == GATE_WORKFLOW and run.get("head_sha") == pr_head
            and (run.get("repository") or {}).get("full_name") == repo
            and lists_pull(run.get("pull_requests") or [], number, base_ref)]
    if not runs:
        return False
    newest = max(runs, key=run_order)
    return newest.get("status") == "completed" and newest.get("conclusion") == "success"


def merge_group_plan(base_sha, head_sha):
    """The checked plan of the queue commit, made before any wait; raise on any doubt."""
    base, tested = os.environ["BASE_SHA"], os.environ["TESTED_SHA"]
    for label, sha in (("base", base), ("tested", tested)):
        if not SHA.fullmatch(sha):
            raise ValueError(f"{label} SHA is missing or malformed")
    # The base files ran from BASE_SHA and the checkout is TESTED_SHA: both must be this group's.
    if (base, tested) != (base_sha, head_sha):
        raise ValueError(f"BASE_SHA {base} and TESTED_SHA {tested} are not the merge group base "
                         f"{base_sha} and queue commit {head_sha}")
    if git("merge-base", "--is-ancestor", base, head_sha)[0] != 0:
        raise ValueError(f"the queue commit {head_sha} does not contain the base {base}")
    plan = recompute_plan(base, head_sha, head_sha, "main", event="merge_group")
    check_plan(plan, base, head_sha, head_sha, "main", event="merge_group")
    return plan


def merge_group_hold(token, repo):
    """Return (passed, lines) for the gates of the queue commit and every pull request in the group.

    Time budget. The clock starts here, before the plan. B is the base catalogue's
    merge_group budget (0 to MAX_WAIT_SECONDS, 45 min), P the time up to the wait
    (the plan: at most SELECTOR_TIMEOUT plus two git calls of at most 120 s each,
    about 14 min), R = DECISION_RESERVE_SECONDS (3 min). The wait gets B - P - R.
    It checks its deadline after each poll, so it ends at most one POLL_SECONDS sleep
    plus one poll past it, and makes one poll even when B - P - R <= 0: the plan and
    the wait together take at most max(B - R + POLL_SECONDS, P) plus one poll, which
    is at most max(B, P + one poll) while a poll takes no more than R - POLL_SECONDS.
    R is an allowance, not a proof, for the decision after the wait (the final gate
    report, about two requests per counted run, and about five per member; usually
    well under a minute). So plan + wait + reserve <= max(B, P + one poll) + R
    <= 45 + 3 = 48 min, below the 50-minute job timeout (merge-hold.yml
    timeout-minutes), for every catalogue budget. The workflow steps before this
    script are outside this clock; the job timeout stays the fail-closed backstop.
    """
    start = time.monotonic()
    head_sha = os.environ["MERGE_GROUP_HEAD_SHA"]
    if os.environ["MERGE_GROUP_BASE_REF"] != "refs/heads/main":
        raise ValueError("merge group does not target main")
    head_ref = os.environ["MERGE_GROUP_HEAD_REF"]
    match = QUEUE_REF.fullmatch(head_ref)
    if not match or not SHA.fullmatch(head_sha):
        raise ValueError(f"unexpected merge group ref {head_ref!r} or SHA {head_sha!r}")
    base_sha = os.environ["MERGE_GROUP_BASE_SHA"]
    plan = merge_group_plan(base_sha, head_sha)
    budget = base_wait_seconds(base_sha)
    branch = head_ref.removeprefix("refs/heads/")
    # Decide last: the gates, labels and comments are read only after this group's
    # checks and gate runs and every entry ahead (which merge first) have finished.
    early = wait_for_checks(token, repo, head_sha, int(match.group(1)), plan, branch,
                            budget - (time.monotonic() - start) - DECISION_RESERVE_SECONDS)
    # A gate that already failed ended the wait: that full report decides, as the final one would.
    passed_gates, pending, failed = early or gate_report(token, repo, None, head_sha, plan, queue_branch=branch)
    _, report, tail = gate_lines(plan, passed_gates, pending, failed)
    enforce, clean = plan.get("policy_mode") != "shadow", not failed and not pending
    ok = clean or not enforce
    members = group_members(token, repo, head_ref, head_sha)
    lines = []
    for node in members:
        pull, entry_commit = node["pullRequest"], node["headCommit"]["oid"]
        number, pr_head = pull["number"], pull["headRefOid"]
        passed, message, labels = pull_request_hold(token, repo, number, pr_head)
        # A released hold covers only the tree it ran on, so this PR's own queue
        # entry (the commit main becomes when it merges) must land that tree. An
        # unheld member may land another tree only when the enforced queue-commit
        # report above is clean, because only then do its gates cover that tree.
        same_tree = tree(token, repo, entry_commit) == tree(token, repo, pr_head)
        if passed and LABEL in labels and not same_tree:
            passed, message = False, (
                f"Acceptance for {pr_head} covers its own tree, but its queue entry "
                f"{entry_commit} lands a different tree. Update the branch and re-release, "
                "or let the pull requests ahead merge first.")
        elif passed and not same_tree and not (enforce and clean):
            passed, message = False, (
                f"The applicable gates ran on {pr_head}, but its queue entry {entry_commit} lands a "
                "different tree, and the queue-commit gate report is not an enforced clean pass, so it does "
                "not cover that tree. Update the branch.")
        elif passed and not pull_request_gates_passed(token, repo, pr_head, number):
            passed, message = False, (
                f"No successful pull_request merge-hold verified the applicable gates on {pr_head}.")
        ok = ok and passed
        lines.append(f"#{number}: " + (message if passed else "::error::" + message))
    lines.append(f"Plan {plan['plan_id']}: mode {plan['mode']}"
                 + (f", fallback {plan['fallback']['code']}" if plan.get("fallback") else "")
                 + f", base {base_sha}, queue commit {head_sha} ({branch}), target main.")
    return ok, lines + report + tail


def main():
    try:
        token = os.environ["GH_TOKEN"]
        repo = os.environ["REPO"]
        event = os.environ["EVENT_NAME"]
        if event == "pull_request":
            number = int(os.environ["PR_NUMBER"])
            passed, message, _ = pull_request_hold(token, repo, number, os.environ["HEAD_SHA"])
            lines = [message if passed else "::error::" + message]
            gates_passed, gate_lines = pull_request_gates(token, repo, number, os.environ["HEAD_SHA"])
            passed = passed and gates_passed
            lines += gate_lines
        elif event == "merge_group":
            passed, lines = merge_group_hold(token, repo)
        else:
            raise ValueError(f"unsupported event {event!r}")
    except (KeyError, IndexError, ValueError, TypeError, AttributeError, OSError,
            subprocess.SubprocessError, json.JSONDecodeError) as exc:
        print(f"::error::merge-hold could not decide ({exc}); failing closed.")
        return 1
    print("\n".join(lines))
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a", encoding="utf-8") as handle:
            handle.write("## merge-hold\n\n" + "".join(f"- {line}\n" for line in lines))
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())

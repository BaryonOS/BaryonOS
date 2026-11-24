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
the group commit, then walks the queue entries ahead from the nearest outward:
each must have left the queue or completed `host-quality`, `qemu-smoke` and
`merge-hold` on its own commit. The walk ends at the first entry whose
`merge-hold` concluded `success`; that run waited the same way, so every entry
ahead of it had finished. A `merge-hold` that ended otherwise proves nothing
about the entries ahead of it, so those are checked too. The queue merges in
order, so this group can merge only after those entries; labels and comments
are therefore read close to this group's merge, not when its own checks end.
A hold added after that final read, until GitHub merges the group, is not seen
(ADR-MERGE-HOLD-20261113). The 45-minute deadline covers the group's
own checks and the whole chain ahead, each position adding up to one poll.
A group whose commit has left the queue (ejected or rebuilt) can never merge;
once its own checks have completed, two consecutive well-formed reads, one
poll apart, that both lack it end the wait with a failure. It then evaluates
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
With `policy.budgets.pull_request_wait_minutes = 0` it decides at once and
lists what it still waits for; re-run the merge-hold job once those gates have
finished. `policy.mode = "shadow"` in the base catalogue reports gates without
blocking on them. On `merge_group`, where the selected workflows do not run,
each member needs a successful `pull_request` merge-hold on its own head and
a queue entry tree equal to that head tree.
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
    """The completed check runs on sha by name (absent or running counts as not done)."""
    # shortcut: one page of 100 check runs; a commit with more runs waits until
    # the deadline and fails closed. Page through if the workflows ever grow that far.
    url = (f"{API}/repos/{repo}/commits/{sha}/check-runs"
           f"?app_id={ACTIONS_APP_ID}&filter=latest&per_page=100")
    return {run["name"]: run for run in call(token, url)["check_runs"]
            if run["status"] == "completed"}


def unfinished(token, repo, sha, names):
    """Names of the check runs on sha that have not completed."""
    done = completed_runs(token, repo, sha)
    return [f"{name} on {sha}" for name in names if name not in done]


def still_running(token, repo, sha, number):
    """What this group still waits for: its own checks, then the entries ahead.

    Entries ahead are walked from the nearest outward. The walk stops at the
    first entry whose `merge-hold` concluded `success`: only a successful run
    went through this same wait, so every entry ahead of it had finished. A
    `merge-hold` that ended otherwise (failed early, cancelled, runner lost)
    proves nothing about the entries ahead of it, so the walk continues. Each
    run makes one REST request per entry walked plus one GraphQL query per
    poll; usually that is the nearest entry only. "The rate limit for
    GITHUB_TOKEN is 1,000 requests per hour per repository"
    (https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api);
    beyond it reads fail, are retried and, at the deadline, fail closed.
    """
    pending = unfinished(token, repo, sha, WAIT_CHECKS)
    if pending:
        return pending
    nodes = queue_nodes(token, repo)
    _, position = own_position(nodes, sha, number)
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


def wait_for_checks(token, repo, sha, number, budget=WAIT_SECONDS):
    """Block until this group's checks and the entries ahead have finished; raise at the deadline.

    Only the timing depends on this: the decision afterwards reads the queue,
    labels and comments live and fails closed. API errors and unbuilt entries
    are retried, because waiting longer never weakens the hold. An entry ahead
    that leaves the queue (merged or removed) no longer counts. Two consecutive
    polls whose well-formed reads both show that this group itself has left the
    queue fail at once; any other poll in between resets that count, because a
    single read may lag behind the queue.
    """
    deadline = time.monotonic() + budget
    absent = 0
    while True:
        try:
            pending = still_running(token, repo, sha, number)
            absent = 0
        except LeftQueue:
            absent += 1
            if absent >= 2:
                raise
            pending = ["merge group commit not listed in the queue once; reading again"]
        except (KeyError, IndexError, ValueError, TypeError, AttributeError, OSError) as exc:
            absent = 0
            pending = [f"queue or check-run read failed: {exc}"]
        if not pending:
            return
        if time.monotonic() >= deadline:
            raise ValueError(f"required checks did not complete in time: {pending}")
        time.sleep(POLL_SECONDS)


def git(*args):
    """Run git in the checkout; return (returncode, stripped stdout)."""
    result = subprocess.run(["git", *args], capture_output=True, text=True, timeout=120, check=False)
    return result.returncode, result.stdout.strip()


def recompute_plan(base, head, tested, ref, repo_dir="."):
    """Run the selector next to this file (the base copy in CI) and return its plan."""
    selector = os.path.join(os.path.dirname(os.path.abspath(__file__)), "ci_impact.py")
    if not os.path.isfile(selector):
        raise ValueError("no tools/ci_impact.py next to merge_hold.py; the applicable gates are unknown")
    with tempfile.TemporaryDirectory() as tmp:
        out = os.path.join(tmp, "plan.json")
        result = subprocess.run(
            [sys.executable, "-B", "-I", selector, "plan", "--event=pull_request", f"--base-sha={base}",
             f"--head-sha={head}", f"--tested-sha={tested}", f"--base-ref={ref}", f"--repo={repo_dir}",
             f"--out={out}"],
            capture_output=True, text=True, timeout=SELECTOR_TIMEOUT, check=False)
        if result.returncode != 0:
            raise ValueError(f"the selector exited {result.returncode}: {result.stderr.strip()[-400:]}")
        with open(out, encoding="ascii") as handle:
            return json.load(handle)


def check_plan(plan, base, head, tested, ref):
    """Raise unless the plan is a trusted, complete plan for exactly this pull request."""
    shas = plan.get("shas") or {}
    if (plan.get("event"), plan.get("ref"), shas.get("base"), shas.get("head"), shas.get("tested")) != (
            "pull_request", ref, base, head, tested):
        raise ValueError("the recomputed plan does not describe this pull request's base, head and merge commit")
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


def run_matches(run, repo, pull, head_sha):
    """Provenance of a workflow run: this repository, this pull request head, the pull_request event.

    GitHub lists the run's pull requests only for same-repository heads; when it
    does, this pull request (number and base branch) must be among them. A fork
    run has an empty list and is bound by head SHA, branch and head repository.
    """
    head = pull["head"]
    pulls = run.get("pull_requests") or []
    return (run.get("event") == "pull_request" and run.get("head_sha") == head_sha
            and run.get("head_branch") == head["ref"]
            and (run.get("repository") or {}).get("full_name") == repo
            and (run.get("head_repository") or {}).get("full_name") == (head.get("repo") or {}).get("full_name")
            and (not pulls or any(item.get("number") == pull.get("number")
                                  and ((item.get("base") or {}).get("ref")) == pull["base"]["ref"]
                                  for item in pulls)))


def run_order(run):
    """Newest first by the start of the latest attempt, then id: a later re-run of an older run counts.

    Without a start time the creation time is used; without both the run cannot be ordered.
    """
    started = run.get("run_started_at") or run.get("created_at")
    if not isinstance(started, str) or not started:
        raise ValueError(f"run {run.get('id')} has no start or creation time")
    return (started, run["id"])


def job_matches(row, job):
    if row.get("variant") == "*":  # runtime matrix: the check name is a template
        pattern = "".join(".+" if EXPRESSION.fullmatch(part) else re.escape(part)
                          for part in re.split(r"(\$\{\{.*?\}\})", row["check_name"]))
        return re.fullmatch(pattern, job.get("name") or "") is not None
    return job.get("name") == row["check_name"]


def gate_report(token, repo, pull, head_sha, plan):
    """Return (passed, pending, failed) lines for the applicable gates of this head."""
    expected = expected_gates(plan)
    rows = plan["selected"] + plan["excluded"]
    # Workflows with a gate GitHub may start: a running run of one must finish, because jobs
    # waiting on `needs` or a runtime matrix may not be listed yet.
    gated = {row["workflow"] for row in rows if row.get("class") in SELECTABLE
             and row.get("reason") not in ("event-not-applicable", "condition-false")}
    classes = {row["check_name"]: row["class"] for row in rows if row.get("check_name")}
    workflows = sorted({row["workflow"] for row in rows if row.get("workflow")} - {GATE_WORKFLOW})
    url = f"{API}/repos/{repo}/actions/runs?head_sha={head_sha}&event=pull_request"
    newest = {}
    for run in fetch_listing(token, url, "workflow_runs"):
        path = run.get("path")
        if path in workflows and run_matches(run, repo, pull, head_sha):
            # The newest run decides: an older success never masks a newer run.
            order = run_order(run)  # every counted run must be orderable
            if path not in newest or order > run_order(newest[path]):
                newest[path] = run
    passed, pending, failed = [], [], []
    for workflow in workflows:
        wanted = [row for row in expected if row["workflow"] == workflow]
        run = newest.get(workflow)
        if run is None:
            pending += [f"{row['check_name']}: no {workflow} run for {head_sha} yet" for row in wanted]
            continue
        where = f"run {run['id']} attempt {run.get('run_attempt')}"
        running = run.get("status") != "completed"
        if not running and not wanted and run.get("conclusion") == "success":
            continue  # no expected gate and nothing failed in it
        # A running run is listed too: a gate GitHub started must finish even when
        # the plan did not select it, or its later failure would go unseen.
        jobs = [job for job in fetch_listing(
            token, f"{API}/repos/{repo}/actions/runs/{run['id']}/jobs?filter=latest", "jobs")
            if job.get("run_id") == run["id"]]
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
            else:
                outcome = ", ".join(sorted({str(job.get("conclusion")) for job in found}))
                failed.append(f"{row['check_name']}: {outcome} in {where} ({workflow})")
        for job in jobs:
            if job["id"] in claimed or classes.get(job.get("name"), "unregistered") not in SELECTABLE + (
                    "unregistered",):
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
    while True:
        passed, pending, failed = gate_report(token, repo, pull, head_sha, plan)
        if failed or not pending or time.monotonic() >= deadline:
            break
        time.sleep(POLL_SECONDS)
    skipped = [f"{row['check_name']}: not applicable (path filter)" for row in plan["selected"]
               if row["class"] in SELECTABLE and row.get("blocked_by_filter") is True]
    lines.append(f"{len(passed)} applicable gate(s) passed; {len(skipped)} filtered out; "
                 f"{len(pending)} pending; {len(failed)} failed.")
    prefix = "::error::" if enforce else "::warning::(shadow, not blocking) "
    lines += [prefix + "Failed: " + line for line in failed]
    lines += [prefix + "Pending: " + line for line in pending]
    if pending and not failed:
        run = os.environ.get("GITHUB_RUN_ID", "<run id>")
        lines.append(f"{prefix}Re-run this merge-hold job after the pending gates finish "
                     f"(gh run rerun {run} --failed).")
    lines += ["Passed: " + line for line in passed] + ["Not a pass: " + line for line in skipped]
    return (not failed and not pending) or not enforce, lines


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


def pull_request_gates_passed(token, repo, pr_head):
    """True when the newest pull_request merge-hold run on pr_head succeeded (merge_group members)."""
    url = f"{API}/repos/{repo}/actions/runs?head_sha={pr_head}&event=pull_request"
    runs = [run for run in fetch_listing(token, url, "workflow_runs")
            if run.get("path") == GATE_WORKFLOW and run.get("head_sha") == pr_head
            and (run.get("repository") or {}).get("full_name") == repo]
    if not runs:
        return False
    newest = max(runs, key=run_order)
    return newest.get("status") == "completed" and newest.get("conclusion") == "success"


def merge_group_hold(token, repo):
    """Return (passed, lines) for every pull request in the merge group."""
    head_sha = os.environ["MERGE_GROUP_HEAD_SHA"]
    if os.environ["MERGE_GROUP_BASE_REF"] != "refs/heads/main":
        raise ValueError("merge group does not target main")
    head_ref = os.environ["MERGE_GROUP_HEAD_REF"]
    match = QUEUE_REF.fullmatch(head_ref)
    if not match or not SHA.fullmatch(head_sha):
        raise ValueError(f"unexpected merge group ref {head_ref!r} or SHA {head_sha!r}")
    # Decide last: labels and comments are read only after this group's checks
    # and every entry ahead (which merge first) have finished.
    base_sha = os.environ.get("MERGE_GROUP_BASE_SHA", "")
    budget = base_wait_seconds(base_sha) if SHA.fullmatch(base_sha) else WAIT_SECONDS
    wait_for_checks(token, repo, head_sha, int(match.group(1)), budget)
    members = group_members(token, repo, head_ref, head_sha)
    ok, lines = True, []
    for node in members:
        pull, entry_commit = node["pullRequest"], node["headCommit"]["oid"]
        number, pr_head = pull["number"], pull["headRefOid"]
        passed, message, labels = pull_request_hold(token, repo, number, pr_head)
        # Compare with this PR's own queue entry: that commit is what main
        # becomes when it merges, and its own group checks that tree. The
        # selected gates run only on pull_request, so their verdict on the PR
        # head (its merge-hold) covers the entry only when the trees are equal.
        same_tree = tree(token, repo, entry_commit) == tree(token, repo, pr_head)
        if passed and LABEL in labels and not same_tree:
            passed, message = False, (
                f"Acceptance for {pr_head} covers its own tree, but its queue entry "
                f"{entry_commit} lands a different tree. Update the branch and re-release, "
                "or let the pull requests ahead merge first.")
        elif passed and not same_tree:
            passed, message = False, (
                f"The applicable gates ran on {pr_head}, but its queue entry {entry_commit} lands a "
                "different tree, and the selected workflows do not run on merge_group. Update the branch.")
        elif passed and not pull_request_gates_passed(token, repo, pr_head):
            passed, message = False, (
                f"No successful pull_request merge-hold verified the applicable gates on {pr_head}.")
        ok = ok and passed
        lines.append(f"#{number}: " + (message if passed else "::error::" + message))
    return ok, lines


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

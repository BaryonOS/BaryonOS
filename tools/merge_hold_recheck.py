#!/usr/bin/env python3
"""Re-run a pending merge-hold once the gate runs of its pull request head have finished.

Issue #494, item B9. On `pull_request` merge-hold decides at once
(`policy.budgets.pull_request_wait_minutes = 0` in .github/ci-impact.toml) and
fails with "Pending" while gates still run, so it no longer holds a runner while
it waits. `.github/workflows/merge-hold-recheck.yml` runs this script from the
default branch whenever a catalogue workflow with a `pull_request` trigger
completes: every gate workflow (one that owns an automatic `core` or `rwa` job in
the catalogue) and, as a trigger only, every other one, because a run of those
can still leave merge-hold at Pending (a run that changed while it was read, or
a job the base catalogue does not know). For a `pull_request` run from this
repository it re-runs the newest merge-hold run of each listed pull request
whose current head is still the run's head, but only when

- no run of a gate workflow is still open for that head (each open one normally
  starts another re-check when it completes; open runs of other workflows do
  not hold merge-hold, so the re-check does not wait for them),
- that merge-hold run ended `failure`, `timed_out` or `cancelled`, and
- its latest attempt started before the triggering run finished (a later
  attempt already saw that result).

A merge-hold run that is still open but started before the triggering run
finished may have read the gate state too early: the re-check reads the run
listing again, at most WAIT_POLLS times POLL_SECONDS apart, and then applies the
same rules. The pull request head is read again just before a re-run request.

It never passes or fails a check itself: the re-run merge-hold decides again on the
same head and push-time merge commit, with live run, job, label and comment data,
and fails closed as before. Every doubt is a no-op; an unexpected API answer exits
non-zero, which shows on the re-check run and changes no pull request check.
"""

import json
import os
import re
import sys
import time
import tomllib
from datetime import datetime
from urllib import error, request

API = "https://api.github.com"
GATE_WORKFLOW = ".github/workflows/merge-hold.yml"
CATALOGUE = ".github/ci-impact.toml"
SELECTABLE = ("core", "rwa")
SHA = re.compile(r"[0-9a-f]{40}")
MAX_PAGES = 50
# Conclusions worth one more decision, and the endpoint that re-runs them.
RERUN = {"failure": "rerun-failed-jobs", "timed_out": "rerun-failed-jobs", "cancelled": "rerun"}
# Wait for a merge-hold that may have reported too early: at most 2 minutes for all
# pull requests together, far below the job's 5-minute timeout (tests/test_ci_workflows.py).
WAIT_POLLS = 8
POLL_SECONDS = 15


def gate_workflows(catalogue):
    """Paths of the workflows that own an automatic core or rwa job, merge-hold excluded.

    merge-hold-recheck.yml lists their names plus those of the other catalogue
    workflows with a pull_request trigger (tests/test_ci_workflows.py).
    """
    return {workflow["path"] for workflow in catalogue["workflow"]
            if workflow["path"] != GATE_WORKFLOW
            and any(job["class"] in SELECTABLE and job["activation"] == "automatic"
                    for job in workflow["job"])}


def client(token):
    """Return get(path) and post(path) for the REST API; an HTTP error raises."""
    def send(path, method):
        req = request.Request(API + path, method=method,
                              data=b"{}" if method == "POST" else None,
                              headers={"Authorization": "Bearer " + token,
                                       "Accept": "application/vnd.github+json",
                                       "Content-Type": "application/json",
                                       "X-GitHub-Api-Version": "2022-11-28",
                                       "User-Agent": "baryon-merge-hold-recheck"})
        with request.urlopen(req, timeout=20) as response:
            body = response.read()
        return json.loads(body) if body.strip() else {}  # a re-run answers 201 with an empty body
    return (lambda path: send(path, "GET")), (lambda path: send(path, "POST"))


def when(text):
    """An API timestamp as a datetime; a missing one raises."""
    if not isinstance(text, str) or not text:
        raise ValueError("a run has no timestamp")
    return datetime.fromisoformat(text)


def run_order(run):
    """Newest first by the start of the latest attempt, then id, as merge_hold.run_order."""
    return (when(run.get("run_started_at") or run.get("created_at")), run["id"])


def head_runs(get, repo, sha):
    """Every pull_request workflow run of this head, read across all pages."""
    runs, page = {}, 1
    while True:
        answer = get(f"/repos/{repo}/actions/runs?head_sha={sha}&event=pull_request&per_page=100&page={page}")
        batch, total = answer["workflow_runs"], answer["total_count"]
        if not isinstance(batch, list) or not isinstance(total, int):
            raise TypeError("unexpected run listing")
        runs.update((run["id"], run) for run in batch)
        if len(batch) < 100:
            break
        page += 1
        if page > MAX_PAGES:
            raise ValueError("too many pages of runs")
    if len(runs) < total:
        raise ValueError(f"listed {len(runs)} of {total} runs; the listing changed while paging")
    return list(runs.values())


def newest_hold(runs, number):
    """The newest merge-hold run of one pull request, or None."""
    holds = [run for run in runs if run.get("path") == GATE_WORKFLOW
             and any(item.get("number") == number for item in run.get("pull_requests") or [])]
    return max(holds, key=run_order) if holds else None


def rerun(post, repo, run):
    """Ask GitHub to re-run one merge-hold run; an attempt that already runs is enough."""
    endpoint = RERUN[run["conclusion"]]
    try:
        post(f"/repos/{repo}/actions/runs/{run['id']}/{endpoint}")
    except error.HTTPError as exc:
        with exc:
            body = exc.read().decode("utf-8", "replace").lower()
        if exc.code == 409 or (exc.code == 403 and "already running" in body):
            return f"merge-hold run {run['id']} is already running again"
        raise
    return f"re-ran merge-hold run {run['id']} ({run['conclusion']}, {endpoint})"


def recheck(event, repo, watched, get, post, sleep=time.sleep):
    """Re-run stale merge-hold runs for the head of a finished watched run; return report lines."""
    trigger = event["workflow_run"]
    if (trigger.get("event") != "pull_request"
            or (trigger.get("repository") or {}).get("full_name") != repo
            or (trigger.get("head_repository") or {}).get("full_name") != repo):
        return ["Not a pull_request run from this repository; nothing to re-check."]
    sha, finished = trigger["head_sha"], when(trigger["updated_at"])
    if not isinstance(sha, str) or not SHA.fullmatch(sha):
        raise ValueError("the triggering run has a missing or malformed head SHA")
    if not watched:
        raise ValueError("the catalogue names no gate workflow")
    numbers = [pull.get("number") for pull in trigger.get("pull_requests") or []]
    if not all(type(number) is int and number > 0 for number in numbers):
        raise ValueError("the triggering run lists a malformed pull request number")
    # Workflow names come from the pull request branch: JSON-escape them so no line of the
    # log starts with a workflow command or carries a control character.
    lines = [f"Run {trigger['id']} ({json.dumps(trigger.get('name'))}) finished for {sha}."]
    current = []
    for number in numbers:
        head = get(f"/repos/{repo}/pulls/{number}")["head"]["sha"]
        if head == sha:
            current.append(number)
        else:
            lines.append(f"#{number}: its head moved to {head}; the runs of that head decide.")
    if not current:
        return lines + ["No listed pull request is still at this head; nothing to re-check."]
    runs = head_runs(get, repo, sha)
    # The triggering run has finished even if the listing still lags behind.
    still_open = sorted(f"{json.dumps(run.get('name'))} (run {run['id']}, {run.get('status')})" for run in runs
                        if run.get("path") in watched and run["id"] != trigger["id"]
                        and (run.get("head_repository") or {}).get("full_name") == repo
                        and run.get("status") != "completed")
    if still_open:
        return lines + ["Gate runs still open; their completion re-checks again: " + ", ".join(still_open)]
    polls = WAIT_POLLS  # one budget for all listed pull requests
    for number in current:
        newest = newest_hold(runs, number)
        # An open merge-hold that started before this run finished may have reported Pending
        # just before it: wait for its verdict, then apply the rules below to the fresh listing.
        while (newest and newest.get("status") != "completed"
               and run_order(newest)[0] < finished and polls):
            sleep(POLL_SECONDS)
            polls -= 1
            runs = head_runs(get, repo, sha)
            newest = newest_hold(runs, number)
        if not newest:
            lines.append(f"#{number}: no merge-hold run yet; it decides when it runs.")
            continue
        where = f"#{number}: merge-hold run {newest['id']} attempt {newest.get('run_attempt')}"
        if newest.get("status") != "completed" and run_order(newest)[0] >= finished:
            lines.append(f"{where} is {newest.get('status')} and started after this run finished; "
                         "it decides itself.")
        elif newest.get("status") != "completed":
            lines.append(f"{where} is still {newest.get('status')} after a wait of up to "
                         f"{WAIT_POLLS * POLL_SECONDS} seconds; it decides itself. If it ends "
                         "with only Pending lines, re-run it once the gates have finished "
                         f"(gh run rerun {newest['id']} --failed).")
        elif newest.get("conclusion") not in RERUN:
            lines.append(f"{where} concluded {newest.get('conclusion')}; nothing to re-run.")
        elif run_order(newest)[0] >= finished:
            lines.append(f"{where} started after this run finished; its verdict already covers it.")
        else:
            # Narrow the window in which a push makes this re-run cancel the new head's merge-hold.
            head = get(f"/repos/{repo}/pulls/{number}")["head"]["sha"]
            if head != sha:
                lines.append(f"#{number}: its head moved to {head} before the re-run; "
                             "the runs of that head decide.")
            else:
                lines.append(f"#{number}: " + rerun(post, repo, newest))
    return lines


def main():
    try:
        with open(os.environ["GITHUB_EVENT_PATH"], encoding="utf-8") as handle:
            event = json.load(handle)
        with open(CATALOGUE, "rb") as handle:
            watched = gate_workflows(tomllib.load(handle))
        get, post = client(os.environ["GH_TOKEN"])
        lines = recheck(event, os.environ["REPO"], watched, get, post)
    except (KeyError, IndexError, TypeError, ValueError, AttributeError, OSError,
            tomllib.TOMLDecodeError) as exc:
        print(f"::error::merge-hold re-check failed ({exc}); no check was changed.")
        return 1
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    sys.exit(main())

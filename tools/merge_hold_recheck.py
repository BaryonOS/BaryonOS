#!/usr/bin/env python3
"""Keep merge-hold bound to the latest gate attempts of its pull request head.

Issue #494, item B9. On `pull_request` merge-hold decides at once
(`policy.budgets.pull_request_wait_minutes = 0` in .github/ci-impact.toml) and
fails with "Pending" while gates still run, so it no longer holds a runner while
it waits. `.github/workflows/merge-hold-recheck.yml` runs this script from the
default branch whenever a catalogue workflow with a `pull_request` trigger
completes. It also runs when a watched gate re-run enters `in_progress`: GitHub
does not emit `workflow_run.requested` for re-runs, so that activity invalidates
an older passing or still-active merge-hold before the gate can finish. Every
gate workflow owns an automatic `core` or `rwa` job in the catalogue; the other listed
workflows are completion triggers only, because a run of those can still leave
merge-hold at Pending (docs/CI-CONTRACT.md lists the cases, such as a run that
changed while it was read or a job the base catalogue does not know).

For a completed `pull_request` run from this repository it re-runs the newest
merge-hold run of each listed pull request whose current head is still the run's
head. Older `success`, `neutral` or `skipped` evidence is invalidated immediately
after a watched gate re-run, even if another gate remains open. A blocking hold
is re-run only when no gate workflow is still open for that head
(each open one normally starts another re-check when it completes; open runs of
other workflows do not hold merge-hold, so the re-check does not wait for them).
In both cases:

- the trigger must establish the applicable conclusion and boundary, and
- the hold's latest attempt started before the triggering run finished (or at
  the same timestamp for stale passing evidence); a later attempt already saw that result.

A merge-hold run that is still open but started before the triggering run
finished may have read the gate state too early: the re-check reads the run
listing again, at most WAIT_POLLS times POLL_SECONDS apart, and then applies the
same rules. For an `in_progress` watched gate attempt greater than one, an older
active hold is cancelled and followed to an exact terminal state; a passing
race is re-run. Force-cancel is a bounded fallback. The pull request head is
read again just before every cancel, force-cancel and re-run request.

It never synthesizes or passes a check itself: the re-run merge-hold decides
again on the same head and push-time merge commit, with live run, job, label
and comment data, and fails closed as before. An unexpected identity, state or
API answer exits non-zero and is never accepted as safe.
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
# GitHub treats success, neutral and skipped as passing required checks. All
# three therefore need a whole-workflow re-run when their evidence is stale.
PASSING_CONCLUSIONS = {"success", "neutral", "skipped"}
BLOCKING_CONCLUSIONS = {
    "action_required", "cancelled", "failure", "stale", "startup_failure", "timed_out",
}
FAILED_RERUN = {"failure": "rerun-failed-jobs", "timed_out": "rerun-failed-jobs",
                "action_required": "rerun", "cancelled": "rerun", "stale": "rerun",
                "startup_failure": "rerun"}
RERUN = {**FAILED_RERUN, **dict.fromkeys(PASSING_CONCLUSIONS, "rerun")}
# A duplicate lifecycle event may race the same POST. Accept its conflict only
# after the exact run proves that another attempt exists.
RERUN_CONFIRM_POLLS = 6
RERUN_CONFIRM_SECONDS = 5
RERUN_POST_LIMIT = 2
CANCEL_CONFIRM_POLLS = 6
FORCE_CANCEL_CONFIRM_POLLS = 12
KNOWN_RUN_STATUS = {"queued", "in_progress", "completed", "waiting", "pending", "requested"}
ACTIVE_RUN_STATUS = KNOWN_RUN_STATUS - {"completed"}
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


def hold_state(run, repo, number, sha, expected_id=None):
    """Validate the identity and decision state of one exact merge-hold run."""
    run_id = run.get("id")
    if (type(run_id) is not int or run_id < 1
            or (expected_id is not None and run_id != expected_id)):
        raise ValueError("the merge-hold run has a malformed or unexpected id")
    if (run.get("path") != GATE_WORKFLOW or run.get("head_sha") != sha
            or (run.get("head_repository") or {}).get("full_name") != repo
            or not any(type(item) is dict and item.get("number") == number
                       for item in run.get("pull_requests") or [])):
        raise ValueError("the merge-hold run is not bound to this pull request head")
    attempt, status = run.get("run_attempt"), run.get("status")
    if type(attempt) is not int or attempt < 1:
        raise ValueError("the merge-hold run has a malformed attempt number")
    if status not in KNOWN_RUN_STATUS:
        raise ValueError("the merge-hold run has an unexpected status")
    started = when(run.get("run_started_at"))
    conclusion = run.get("conclusion")
    if status == "completed":
        if conclusion not in PASSING_CONCLUSIONS | BLOCKING_CONCLUSIONS:
            raise ValueError("the merge-hold run has an unexpected conclusion")
    elif conclusion is not None:
        raise ValueError("the active merge-hold run already has a conclusion")
    return attempt, status, started, conclusion


def rerun(get, post, repo, run, boundary, number, sha, sleep=time.sleep):
    """Re-run one hold; a racing attempt must start after the gate boundary."""
    candidate = run
    for request_number in range(RERUN_POST_LIMIT):
        previous, status, _, conclusion = hold_state(candidate, repo, number, sha)
        if status != "completed":
            raise ValueError("an active merge-hold run cannot be re-run")
        endpoint = RERUN[conclusion]
        head = get(f"/repos/{repo}/pulls/{number}")["head"]["sha"]
        if head != sha:
            return (f"its head moved to {head} before the re-run; "
                    "the runs of that head decide.")
        try:
            post(f"/repos/{repo}/actions/runs/{candidate['id']}/{endpoint}")
        except error.HTTPError as exc:
            with exc:
                body = exc.read().decode("utf-8", "replace").lower()
            racing = exc.code == 409 or (exc.code == 403 and "already running" in body)
            if not racing:
                raise
            replacement = None
            for poll in range(RERUN_CONFIRM_POLLS):
                current = get(f"/repos/{repo}/actions/runs/{candidate['id']}")
                attempt, current_status, started, _ = hold_state(
                    current, repo, number, sha, candidate["id"])
                if attempt > previous:
                    if started > boundary:
                        return (f"merge-hold run {candidate['id']} already has attempt "
                                f"{attempt} after the gate boundary")
                    if current_status == "completed":
                        replacement = current
                        break
                if poll + 1 < RERUN_CONFIRM_POLLS:
                    sleep(RERUN_CONFIRM_SECONDS)
            if replacement is None:
                raise
            if request_number + 1 >= RERUN_POST_LIMIT:
                raise
            candidate = replacement
            continue
        return (f"re-ran merge-hold run {candidate['id']} "
                f"({candidate['conclusion']}, {endpoint})")
    raise RuntimeError("unreachable rerun loop")


def cancel_stale_hold(get, post, repo, run, boundary, number, sha, sleep=time.sleep):
    """Cancel an older active hold; only an exact terminal state is evidence."""
    previous, status, started, _ = hold_state(run, repo, number, sha)
    if status not in ACTIVE_RUN_STATUS:
        raise ValueError("only an active merge-hold run can be cancelled")
    starts = {previous: started}
    head = get(f"/repos/{repo}/pulls/{number}")["head"]["sha"]
    if head != sha:
        return (f"its head moved to {head} before the cancellation; "
                "the runs of that head decide.")
    try:
        post(f"/repos/{repo}/actions/runs/{run['id']}/cancel")
    except error.HTTPError as exc:
        # A completion can win this race. A conflict is never proof by itself;
        # the exact-run reads below must establish a non-green or newer state.
        with exc:
            exc.read()
        if exc.code != 409:
            raise

    def confirmed(current, phase):
        attempt, current_status, current_started, conclusion = hold_state(
            current, repo, number, sha, run["id"])
        if attempt < previous:
            raise ValueError("the cancelled merge-hold run moved to an older attempt")
        known_start = starts.setdefault(attempt, current_started)
        if current_started != known_start:
            raise ValueError("the merge-hold attempt changed its start timestamp")
        if attempt > previous and current_started > boundary:
            return (f"merge-hold run {run['id']} already has attempt {attempt} "
                    "after the gate boundary")
        if current_status != "completed":
            return None
        if conclusion in PASSING_CONCLUSIONS:
            return rerun(get, post, repo, current, boundary, number, sha, sleep)
        return (f"merge-hold run {run['id']} ended {conclusion} after {phase}; "
                "no passing verdict remains")

    for poll in range(CANCEL_CONFIRM_POLLS):
        current = get(f"/repos/{repo}/actions/runs/{run['id']}")
        if result := confirmed(current, "cancellation"):
            return result
        if poll + 1 < CANCEL_CONFIRM_POLLS:
            sleep(RERUN_CONFIRM_SECONDS)

    head = get(f"/repos/{repo}/pulls/{number}")["head"]["sha"]
    if head != sha:
        return (f"its head moved to {head} before force-cancellation; "
                "the runs of that head decide.")
    try:
        post(f"/repos/{repo}/actions/runs/{run['id']}/force-cancel")
    except error.HTTPError as exc:
        with exc:
            exc.read()
        if exc.code != 409:
            raise
    for poll in range(FORCE_CANCEL_CONFIRM_POLLS):
        current = get(f"/repos/{repo}/actions/runs/{run['id']}")
        if result := confirmed(current, "force-cancellation"):
            return result
        if poll + 1 < FORCE_CANCEL_CONFIRM_POLLS:
            sleep(RERUN_CONFIRM_SECONDS)
    raise TimeoutError("the force-cancelled merge-hold run did not reach a verifiable state")


def stale_passing(run, trigger, watched, not_after):
    """Whether passing evidence is no newer than this gate re-run boundary."""
    return (trigger["run_attempt"] > 1
            and trigger["path"] in watched
            and run.get("status") == "completed"
            and run.get("conclusion") in PASSING_CONCLUSIONS
            and run_order(run)[0] <= not_after)


def open_gates(runs, trigger, watched, repo):
    """Gate runs of this head, other than the triggering attempt, that have not completed."""
    # The listing may still show the finished triggering attempt as open; a re-run keeps the
    # run id with a new attempt, which must still count. Workflow names come from the pull
    # request branch: JSON-escape them so no log line starts with a workflow command.
    finished = (trigger["id"], trigger.get("run_attempt"))
    return sorted(f"{json.dumps(run.get('name'))} (run {run['id']}, {run.get('status')})" for run in runs
                  if run.get("path") in watched and (run["id"], run.get("run_attempt")) != finished
                  and (run.get("head_repository") or {}).get("full_name") == repo
                  and run.get("status") != "completed")


def recheck(event, repo, watched, get, post, sleep=time.sleep):
    """Re-run stale merge-hold runs for one watched activity; return report lines."""
    trigger = event["workflow_run"]
    if (trigger.get("event") != "pull_request"
            or (trigger.get("repository") or {}).get("full_name") != repo
            or (trigger.get("head_repository") or {}).get("full_name") != repo):
        return ["Not a pull_request run from this repository; nothing to re-check."]
    action = event.get("action")
    if action not in {"completed", "in_progress"}:
        raise ValueError(f"unexpected workflow_run activity {action!r}")
    attempt, path = trigger.get("run_attempt"), trigger.get("path")
    if type(attempt) is not int or attempt < 1:
        raise ValueError("the triggering run has a malformed attempt number")
    if not isinstance(path, str) or not path:
        raise ValueError("the triggering run has no workflow path")
    if not watched:
        raise ValueError("the catalogue names no gate workflow")
    if action == "in_progress" and (attempt == 1 or path not in watched):
        return ["Not a watched gate re-run attempt; nothing to re-check."]
    sha = trigger["head_sha"]
    if not isinstance(sha, str) or not SHA.fullmatch(sha):
        raise ValueError("the triggering run has a missing or malformed head SHA")
    trigger_started = when(trigger["run_started_at"])
    finished = when(trigger["updated_at"]) if action == "completed" else None
    numbers = [pull.get("number") for pull in trigger.get("pull_requests") or []]
    if not all(type(number) is int and number > 0 for number in numbers):
        raise ValueError("the triggering run lists a malformed pull request number")
    # Workflow names come from the pull request branch: JSON-escape them so no line of the
    # log starts with a workflow command or carries a control character.
    activity = "finished" if action == "completed" else f"started attempt {attempt}"
    lines = [f"Run {trigger['id']} ({json.dumps(trigger.get('name'))}) {activity} for {sha}."]
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
    if action == "in_progress":
        for number in current:
            newest = newest_hold(runs, number)
            if not newest:
                lines.append(f"#{number}: no merge-hold run yet; it decides when it runs.")
                continue
            where = f"#{number}: merge-hold run {newest['id']} attempt {newest.get('run_attempt')}"
            _, status, _, conclusion = hold_state(newest, repo, number, sha)
            if status in ACTIVE_RUN_STATUS:
                if run_order(newest)[0] <= trigger_started:
                    lines.append(f"#{number}: "
                                 + cancel_stale_hold(get, post, repo, newest, trigger_started,
                                                     number, sha, sleep))
                else:
                    lines.append(f"{where} is {status} and started after this gate re-run "
                                 "started; it decides itself.")
                continue
            if not stale_passing(newest, trigger, watched, trigger_started):
                if (conclusion in PASSING_CONCLUSIONS
                        and run_order(newest)[0] > trigger_started):
                    lines.append(f"{where} started after this gate re-run started; "
                                 "its verdict already covers it.")
                else:
                    lines.append(f"{where} is not older passing evidence; nothing to re-run.")
                continue
            lines.append(f"#{number}: "
                         + rerun(get, post, repo, newest, trigger_started,
                                 number, sha, sleep))
        return lines
    # The triggering run has finished even if the listing still lags behind.
    still_open = open_gates(runs, trigger, watched, repo)
    if still_open and not (attempt > 1 and path in watched):
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
        _, _, _, conclusion = hold_state(newest, repo, number, sha)
        stale = stale_passing(newest, trigger, watched, finished)
        if newest.get("status") != "completed" and run_order(newest)[0] >= finished:
            lines.append(f"{where} is {newest.get('status')} and started after this run finished; "
                         "it decides itself.")
            continue
        elif newest.get("status") != "completed":
            lines.append(f"{where} is still {newest.get('status')} after a wait of up to "
                         f"{WAIT_POLLS * POLL_SECONDS} seconds; it decides itself. If it ends "
                         "with only Pending lines, re-run it once the gates have finished "
                         f"(gh run rerun {newest['id']} --failed).")
            continue
        if not stale and conclusion not in FAILED_RERUN:
            if (attempt > 1 and path in watched
                    and conclusion in PASSING_CONCLUSIONS
                    and run_order(newest)[0] > finished):
                lines.append(f"{where} started after this gate re-run finished; "
                             "its verdict already covers it.")
            else:
                lines.append(f"{where} concluded {conclusion}; nothing to re-run.")
            continue
        if not stale and run_order(newest)[0] >= finished:
            lines.append(f"{where} started after this run finished; its verdict already covers it.")
            continue
        if not stale and (reopened := open_gates(runs, trigger, watched, repo)):
            # A gate can start a new attempt during the wait; its completion re-checks again.
            lines.append(f"#{number}: gate runs open again after the wait; their completion "
                         "re-checks again: " + ", ".join(reopened))
            continue
        lines.append(f"#{number}: "
                     + rerun(get, post, repo, newest, finished, number, sha, sleep))
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
        print(f"::error::merge-hold re-check failed ({exc}); no state was accepted as safe.")
        return 1
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    sys.exit(main())

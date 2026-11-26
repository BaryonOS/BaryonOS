"""Unit tests for the merge-hold re-check (#494 B9); no network access."""

import contextlib
import io
import json
import os
import sys
import tempfile
import tomllib
from pathlib import Path
from unittest import TestCase, main, mock
from urllib import error
from urllib.parse import parse_qs, urlsplit

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import merge_hold_recheck as recheck_module

REPO = "o/r"
HEAD = "a" * 40
OTHER = "b" * 40
CI = ".github/workflows/ci.yml"
LEAVES = ".github/workflows/component-leaves.yml"
WINDOWS = ".github/workflows/windows-qemu-test-version.yml"
HOLD = recheck_module.GATE_WORKFLOW
WATCHED = {CI, LEAVES}
FINISHED = "2026-11-25T12:00:00Z"
BEFORE = "2026-11-25T11:50:00Z"
AFTER = "2026-11-25T12:00:30Z"


def run(run_id, path, status="completed", conclusion="success", started=BEFORE, number=7,
        head_repo=REPO, attempt=1):
    return {"id": run_id, "path": path, "name": path.rsplit("/", 1)[-1], "status": status,
            "conclusion": conclusion if status == "completed" else None, "run_started_at": started,
            "created_at": started, "run_attempt": attempt, "head_sha": HEAD,
            "head_repository": {"full_name": head_repo}, "pull_requests": [{"number": number}]}


def event(trigger_id=1, kind="pull_request", head_repo=REPO, numbers=(7,), sha=HEAD,
          action="completed", attempt=1, path=CI, started=BEFORE):
    return {"action": action,
            "workflow_run": {"id": trigger_id, "run_attempt": attempt, "name": "CI",
                             "path": path, "event": kind, "head_sha": sha,
                             "run_started_at": started,
                             "updated_at": FINISHED, "repository": {"full_name": REPO},
                             "head_repository": {"full_name": head_repo},
                             "pull_requests": [{"number": number} for number in numbers]}}


class FakeApi:
    """Answers pull request reads and paged run listings; records every POST."""

    def __init__(self, runs, heads=None, post_error=None, total=None, later=(), details=()):
        self.runs, self.heads = runs, heads or {7: HEAD}
        self.post_errors = (list(post_error) if isinstance(post_error, list)
                            else ([post_error] if post_error else []))
        self.total = total
        self.later, self.sleeps = list(later), []  # run listings served after each sleep
        self.details = list(details)
        self.gets, self.posts = [], []

    def sleep(self, seconds):
        self.sleeps.append(seconds)
        if self.later:
            self.runs = self.later.pop(0)

    def get(self, path):
        self.gets.append(path)
        if path.startswith(f"/repos/{REPO}/pulls/"):
            head = self.heads[int(path.rsplit("/", 1)[1])]
            if isinstance(head, list):  # the head of each successive read; the last one stays
                head = head.pop(0) if len(head) > 1 else head[0]
            return {"head": {"sha": head}}
        if path.startswith(f"/repos/{REPO}/actions/runs/") and "?" not in path:
            assert self.details, path
            return self.details.pop(0) if len(self.details) > 1 else self.details[0]
        url = urlsplit(path)
        assert url.path == f"/repos/{REPO}/actions/runs", path
        query = parse_qs(url.query)
        assert query["head_sha"] == [HEAD] and query["event"] == ["pull_request"], path
        size, page = int(query["per_page"][0]), int(query["page"][0])
        total = len(self.runs) if self.total is None else self.total
        return {"total_count": total, "workflow_runs": self.runs[(page - 1) * size:page * size]}

    def post(self, path):
        self.posts.append(path)
        post_error = self.post_errors.pop(0) if self.post_errors else None
        if post_error:
            code, body = post_error
            raise error.HTTPError(path, code, "error", {}, io.BytesIO(body))
        return {}


def decide(runs, payload=None, **kwargs):
    api = FakeApi(runs, **kwargs)
    lines = recheck_module.recheck(payload or event(), REPO, WATCHED, api.get, api.post, api.sleep)
    return api, lines


class RecheckTests(TestCase):
    def test_in_progress_gate_rerun_invalidates_an_older_successful_hold(self):
        payload = event(action="in_progress", attempt=2)
        api, lines = decide([run(1, CI, status="in_progress", attempt=2), run(3, HOLD)],
                            payload=payload)
        self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/rerun"])
        self.assertIn("re-ran merge-hold run 3", lines[-1])

    def test_completed_gate_rerun_rechecks_an_older_successful_hold(self):
        payload = event(attempt=2)
        for started in (BEFORE, "2026-11-25T11:55:00Z", FINISHED):
            with self.subTest(started=started):
                api, lines = decide([run(1, CI, attempt=2), run(3, HOLD, started=started)],
                                    payload=payload)
                self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/rerun"])
                self.assertIn("re-ran merge-hold run 3", lines[-1])

    def test_gate_rerun_invalidates_every_older_passing_hold(self):
        for action in ("in_progress", "completed"):
            for conclusion in ("neutral", "skipped"):
                with self.subTest(action=action, conclusion=conclusion):
                    payload = event(action=action, attempt=2)
                    api, lines = decide(
                        [run(1, CI, status=action, attempt=2),
                         run(3, HOLD, conclusion=conclusion)],
                        payload=payload,
                    )
                    self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/rerun"])
                    self.assertIn(f"re-ran merge-hold run 3 ({conclusion}, rerun)", lines[-1])

    def test_initial_or_non_gate_in_progress_runs_do_not_invalidate_a_green_hold(self):
        for payload in (event(action="in_progress"),
                        event(action="in_progress", attempt=2, path=WINDOWS)):
            with self.subTest(payload=payload["workflow_run"]):
                api, lines = decide([run(1, CI, status="in_progress"), run(3, HOLD)],
                                    payload=payload)
                self.assertEqual((api.gets, api.posts), ([], []))
                self.assertIn("nothing to re-check", lines[-1])

    def test_completed_non_gate_rerun_does_not_invalidate_a_green_hold(self):
        payload = event(attempt=2, path=WINDOWS)
        api, lines = decide([run(1, WINDOWS, attempt=2), run(3, HOLD)], payload=payload)
        self.assertEqual(api.posts, [])
        self.assertIn("concluded success", lines[-1])

    def test_newer_successful_hold_already_covers_the_gate_rerun(self):
        for action in ("in_progress", "completed"):
            with self.subTest(action=action):
                payload = event(action=action, attempt=2)
                api, lines = decide([run(1, CI, attempt=2),
                                     run(3, HOLD, started=AFTER)], payload=payload)
                self.assertEqual(api.posts, [])
                self.assertIn("already covers it", lines[-1])

    def test_in_progress_invalidation_rechecks_the_head_before_posting(self):
        payload = event(action="in_progress", attempt=2)
        api, lines = decide([run(1, CI, status="in_progress", attempt=2), run(3, HOLD)],
                            payload=payload, heads={7: [HEAD, OTHER]})
        self.assertEqual(api.posts, [])
        self.assertIn("head moved to " + OTHER + " before the re-run", lines[-1])

    def test_in_progress_cancels_an_active_hold_at_or_before_the_boundary(self):
        for started in ("2026-11-25T11:40:00Z", BEFORE):
            with self.subTest(started=started):
                payload = event(action="in_progress", attempt=2)
                api, lines = decide(
                    [run(1, CI, status="in_progress", attempt=2),
                     run(3, HOLD, status="in_progress", started=started)],
                    payload=payload,
                    details=[run(3, HOLD, conclusion="cancelled", started=started)],
                )
                self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/cancel"])
                self.assertIn("ended cancelled after cancellation", lines[-1])

    def test_in_progress_reruns_an_active_hold_that_races_cancel_with_green(self):
        payload = event(action="in_progress", attempt=2)
        api, lines = decide(
            [run(1, CI, status="in_progress", attempt=2),
             run(3, HOLD, status="in_progress")],
            payload=payload,
            details=[run(3, HOLD)],
        )
        self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/cancel",
                                     f"/repos/{REPO}/actions/runs/3/rerun"])
        self.assertIn("re-ran merge-hold run 3 (success, rerun)", lines[-1])

    def test_in_progress_reruns_every_passing_cancel_race(self):
        payload = event(action="in_progress", attempt=2)
        for conclusion in ("success", "neutral", "skipped"):
            with self.subTest(conclusion=conclusion):
                api, lines = decide(
                    [run(1, CI, status="in_progress", attempt=2),
                     run(3, HOLD, status="in_progress")],
                    payload=payload,
                    details=[run(3, HOLD, conclusion=conclusion)],
                )
                self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/cancel",
                                             f"/repos/{REPO}/actions/runs/3/rerun"])
                self.assertIn(f"re-ran merge-hold run 3 ({conclusion}, rerun)", lines[-1])

    def test_in_progress_leaves_a_confirmed_blocking_cancel_result_red(self):
        payload = event(action="in_progress", attempt=2)
        for conclusion in ("failure", "cancelled", "timed_out", "action_required",
                           "startup_failure", "stale"):
            with self.subTest(conclusion=conclusion):
                api, lines = decide(
                    [run(1, CI, status="in_progress", attempt=2),
                     run(3, HOLD, status="in_progress")],
                    payload=payload,
                    details=[run(3, HOLD, conclusion=conclusion)],
                )
                self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/cancel"])
                self.assertIn(f"ended {conclusion} after cancellation", lines[-1])

    def test_a_cancel_conflict_requires_exact_terminal_confirmation(self):
        payload = event(action="in_progress", attempt=2)
        api, lines = decide(
            [run(1, CI, status="in_progress", attempt=2),
             run(3, HOLD, status="in_progress")],
            payload=payload,
            post_error=[(409, b'{"message": "conflict"}'), None],
            details=[run(3, HOLD, conclusion="success")],
        )
        self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/cancel",
                                     f"/repos/{REPO}/actions/runs/3/rerun"])
        self.assertIn("re-ran merge-hold run 3", lines[-1])

    def test_a_refused_cancel_raises(self):
        payload = event(action="in_progress", attempt=2)
        with self.assertRaises(error.HTTPError):
            decide(
                [run(1, CI, status="in_progress", attempt=2),
                 run(3, HOLD, status="in_progress")],
                payload=payload,
                post_error=(403, b'{"message": "Resource not accessible by integration"}'),
            )

    def test_in_progress_rechecks_the_head_before_cancel_and_rerun(self):
        payload = event(action="in_progress", attempt=2)
        api, lines = decide(
            [run(1, CI, status="in_progress", attempt=2),
             run(3, HOLD, status="in_progress")],
            payload=payload,
            heads={7: [HEAD, OTHER]},
        )
        self.assertEqual(api.posts, [])
        self.assertIn("head moved to " + OTHER + " before the cancellation", lines[-1])

        api, lines = decide(
            [run(1, CI, status="in_progress", attempt=2),
             run(3, HOLD, status="in_progress")],
            payload=payload,
            heads={7: [HEAD, HEAD, OTHER]},
            details=[run(3, HOLD)],
        )
        self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/cancel"])
        self.assertIn("head moved to " + OTHER + " before the re-run", lines[-1])

    def test_a_newer_attempt_only_covers_the_gate_after_the_boundary(self):
        payload = event(action="in_progress", attempt=2)
        api, lines = decide(
            [run(1, CI, status="in_progress", attempt=2),
             run(3, HOLD, status="in_progress")],
            payload=payload,
            details=[run(3, HOLD, status="in_progress", started=AFTER, attempt=2)],
        )
        self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/cancel"])
        self.assertIn("already has attempt 2 after the gate boundary", lines[-1])

        api, lines = decide(
            [run(1, CI, status="in_progress", attempt=2),
             run(3, HOLD, status="in_progress")],
            payload=payload,
            details=[run(3, HOLD, started=BEFORE, attempt=2)],
        )
        self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/cancel",
                                     f"/repos/{REPO}/actions/runs/3/rerun"])
        self.assertIn("re-ran merge-hold run 3", lines[-1])

    def test_in_progress_does_not_cancel_an_active_hold_after_the_boundary(self):
        payload = event(action="in_progress", attempt=2)
        api, lines = decide([run(1, CI, status="in_progress", attempt=2),
                             run(3, HOLD, status="in_progress", started=AFTER)],
                            payload=payload)
        self.assertEqual(api.posts, [])
        self.assertIn("started after this gate re-run started", lines[-1])

    def test_in_progress_cancel_confirmation_is_bounded_and_fail_closed(self):
        payload = event(action="in_progress", attempt=2)
        api = FakeApi(
            [run(1, CI, status="in_progress", attempt=2),
             run(3, HOLD, status="in_progress")],
            details=[run(3, HOLD, status="in_progress")],
        )
        with self.assertRaises(TimeoutError):
            recheck_module.recheck(payload, REPO, WATCHED, api.get, api.post, api.sleep)
        self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/cancel",
                                     f"/repos/{REPO}/actions/runs/3/force-cancel"])
        self.assertEqual(api.sleeps,
                         [recheck_module.RERUN_CONFIRM_SECONDS]
                         * (recheck_module.CANCEL_CONFIRM_POLLS
                            + recheck_module.FORCE_CANCEL_CONFIRM_POLLS - 2))

    def test_force_cancel_must_reach_a_confirmed_terminal_state(self):
        payload = event(action="in_progress", attempt=2)
        active = run(3, HOLD, status="in_progress")
        api, lines = decide(
            [run(1, CI, status="in_progress", attempt=2), active],
            payload=payload,
            details=[active] * recheck_module.CANCEL_CONFIRM_POLLS
                    + [run(3, HOLD, conclusion="cancelled")],
        )
        self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/cancel",
                                     f"/repos/{REPO}/actions/runs/3/force-cancel"])
        self.assertIn("ended cancelled after force-cancellation", lines[-1])

    def test_force_cancel_conflict_still_requires_exact_confirmation(self):
        payload = event(action="in_progress", attempt=2)
        active = run(3, HOLD, status="in_progress")
        api, lines = decide(
            [run(1, CI, status="in_progress", attempt=2), active],
            payload=payload,
            post_error=[None, (409, b'{"message": "conflict"}')],
            details=[active] * recheck_module.CANCEL_CONFIRM_POLLS
                    + [run(3, HOLD, conclusion="cancelled")],
        )
        self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/cancel",
                                     f"/repos/{REPO}/actions/runs/3/force-cancel"])
        self.assertIn("ended cancelled after force-cancellation", lines[-1])

    def test_head_move_after_cancel_prevents_force_cancel(self):
        payload = event(action="in_progress", attempt=2)
        active = run(3, HOLD, status="in_progress")
        api, lines = decide(
            [run(1, CI, status="in_progress", attempt=2), active],
            payload=payload,
            heads={7: [HEAD, HEAD, OTHER]},
            details=[active],
        )
        self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/cancel"])
        self.assertEqual(api.sleeps,
                         [recheck_module.RERUN_CONFIRM_SECONDS]
                         * (recheck_module.CANCEL_CONFIRM_POLLS - 1))
        self.assertIn("head moved to " + OTHER + " before force-cancellation", lines[-1])

    def test_a_refused_force_cancel_raises(self):
        payload = event(action="in_progress", attempt=2)
        active = run(3, HOLD, status="in_progress")
        with self.assertRaises(error.HTTPError):
            decide(
                [run(1, CI, status="in_progress", attempt=2), active],
                payload=payload,
                post_error=[None, (403, b'{"message": "Resource not accessible"}')],
                details=[active],
            )

    def test_malformed_cancel_confirmation_never_becomes_evidence(self):
        payload = event(action="in_progress", attempt=2)
        fields = {
            "id": 4,
            "path": CI,
            "head_sha": OTHER,
            "head_repository": {"full_name": "fork/r"},
            "pull_requests": [{"number": 8}],
            "run_attempt": 0,
            "status": "mystery",
            "run_started_at": None,
            "conclusion": "mystery",
        }
        for field, value in fields.items():
            with self.subTest(field=field):
                malformed = run(3, HOLD, conclusion="cancelled")
                malformed[field] = value
                api = FakeApi(
                    [run(1, CI, status="in_progress", attempt=2),
                     run(3, HOLD, status="in_progress")],
                    details=[malformed],
                )
                with self.assertRaises((TypeError, ValueError)):
                    recheck_module.recheck(payload, REPO, WATCHED, api.get, api.post, api.sleep)
                self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/cancel"])

    def test_no_rerun_while_a_gate_run_is_still_open(self):
        api, lines = decide([run(1, CI), run(2, LEAVES, status="in_progress"),
                             run(3, HOLD, conclusion="failure")])
        self.assertEqual(api.posts, [])
        self.assertIn("still open", lines[-1])

    def test_completed_rerun_invalidates_stale_green_while_another_gate_is_open(self):
        api, lines = decide([run(1, CI, attempt=2), run(2, LEAVES, status="in_progress"),
                             run(3, HOLD)], payload=event(attempt=2))
        self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/rerun"])
        self.assertIn("re-ran merge-hold run 3", lines[-1])

    def test_workflow_names_from_the_branch_are_escaped_in_the_log(self):
        payload = event()
        payload["workflow_run"]["name"] = "::error::forged\nline"
        gate = run(2, LEAVES, status="in_progress")
        gate["name"] = "::warning::forged"
        _, lines = decide([run(1, CI), gate], payload=payload)
        self.assertFalse(any(line.startswith("::") or "\n" in line for line in lines), lines)
        self.assertIn('"::error::forged\\nline"', lines[0])

    def test_failed_merge_hold_is_rerun_once_every_gate_run_is_closed(self):
        api, lines = decide([run(1, CI), run(2, LEAVES, conclusion="failure"),
                             run(3, HOLD, conclusion="failure")])
        self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/rerun-failed-jobs"])
        self.assertIn("re-ran merge-hold run 3", lines[-1])

    def test_cancelled_and_timed_out_merge_hold_use_their_endpoints(self):
        for conclusion, endpoint in (("cancelled", "rerun"), ("timed_out", "rerun-failed-jobs")):
            with self.subTest(conclusion=conclusion):
                api, _ = decide([run(1, CI), run(3, HOLD, conclusion=conclusion)])
                self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/{endpoint}"])

    def test_no_rerun_for_success_running_skipped_or_missing_merge_hold(self):
        cases = {"success": [run(3, HOLD)], "running": [run(3, HOLD, status="in_progress")],
                 "skipped": [run(3, HOLD, conclusion="skipped")], "missing": []}
        for name, holds in cases.items():
            with self.subTest(case=name):
                api, _ = decide([run(1, CI)] + holds)
                self.assertEqual(api.posts, [])

    def test_no_rerun_when_merge_hold_started_after_the_trigger_finished(self):
        for started in (AFTER, FINISHED):
            with self.subTest(started=started):
                api, lines = decide([run(1, CI), run(3, HOLD, conclusion="failure", started=started)])
                self.assertEqual(api.posts, [])
                self.assertIn("already covers it", lines[-1])

    def test_the_newest_merge_hold_attempt_decides(self):
        # A later attempt of an older run id is newer than a run created in between.
        rerun_old = run(3, HOLD, conclusion="success", started="2026-11-25T11:55:00Z", attempt=2)
        api, _ = decide([run(1, CI), rerun_old, run(4, HOLD, conclusion="failure")])
        self.assertEqual(api.posts, [])
        api, _ = decide([run(1, CI), run(3, HOLD, conclusion="success"),
                         run(4, HOLD, conclusion="failure", started="2026-11-25T11:55:00Z")])
        self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/4/rerun-failed-jobs"])

    def test_merge_hold_of_another_pull_request_is_not_touched(self):
        api, lines = decide([run(1, CI), run(3, HOLD, conclusion="failure", number=8)])
        self.assertEqual(api.posts, [])
        self.assertIn("no merge-hold run yet", lines[-1])

    def test_moved_head_is_left_to_the_runs_of_the_new_head(self):
        api, lines = decide([run(1, CI), run(3, HOLD, conclusion="failure")], heads={7: OTHER})
        self.assertEqual(api.posts, [])
        self.assertFalse(any("actions/runs?" in path for path in api.gets))
        self.assertIn("head moved", lines[1])

    def test_fork_and_non_pull_request_runs_are_ignored_without_api_calls(self):
        for payload in (event(head_repo="fork/r"), event(kind="merge_group"), event(kind="push"),
                        event(kind="workflow_dispatch")):
            with self.subTest(payload=payload["workflow_run"]["event"]):
                api, lines = decide([run(3, HOLD, conclusion="failure")], payload=payload)
                self.assertEqual((api.gets, api.posts), ([], []))
                self.assertIn("nothing to re-check", lines[0])

    def test_runs_of_non_gate_workflows_and_fork_heads_do_not_hold_the_recheck(self):
        # They do not hold merge-hold; a non-gate run that ends later starts its own re-check.
        api, _ = decide([run(1, CI), run(2, WINDOWS, status="in_progress"),
                         run(5, LEAVES, status="queued", head_repo="fork/r"),
                         run(3, HOLD, conclusion="failure")])
        self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/rerun-failed-jobs"])

    def test_a_running_merge_hold_that_started_before_the_gate_finished_is_waited_for(self):
        running = [run(1, CI), run(3, HOLD, status="in_progress")]
        api, lines = decide(running, later=[running, [run(1, CI), run(3, HOLD, conclusion="failure")]])
        self.assertEqual(api.sleeps, [recheck_module.POLL_SECONDS] * 2)
        self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/rerun-failed-jobs"])
        self.assertIn("re-ran merge-hold run 3", lines[-1])
        api, lines = decide(running, later=[[run(1, CI), run(3, HOLD)]])
        self.assertEqual((len(api.sleeps), api.posts), (1, []))
        self.assertIn("concluded success", lines[-1])

    def test_no_rerun_when_a_gate_reopens_during_the_wait(self):
        # CodeRabbit on #550: the open-gate rule applies to the refreshed listing too.
        reopened = [run(1, CI), run(2, LEAVES, status="in_progress", attempt=2),
                    run(3, HOLD, conclusion="failure")]
        api, lines = decide([run(1, CI), run(2, LEAVES), run(3, HOLD, status="in_progress")],
                            later=[reopened])
        self.assertEqual((len(api.sleeps), api.posts), (1, []))
        self.assertIn("gate runs open again after the wait", lines[-1])
        self.assertIn("(run 2, in_progress)", lines[-1])

    def test_no_rerun_when_the_triggering_run_is_rerun_during_the_wait(self):
        # CodeRabbit on #550: a re-run keeps the run id, so only the finished attempt is exempt.
        reopened = [run(1, CI, status="queued", attempt=2), run(3, HOLD, conclusion="failure")]
        api, lines = decide([run(1, CI), run(3, HOLD, status="in_progress")], later=[reopened])
        self.assertEqual((len(api.sleeps), api.posts), (1, []))
        self.assertIn("gate runs open again after the wait", lines[-1])
        self.assertIn("(run 1, queued)", lines[-1])

    def test_the_wait_for_a_running_merge_hold_is_bounded(self):
        api, lines = decide([run(1, CI), run(3, HOLD, status="in_progress")])
        self.assertEqual(api.sleeps, [recheck_module.POLL_SECONDS] * recheck_module.WAIT_POLLS)
        self.assertEqual(api.posts, [])
        self.assertIn("still in_progress after a wait", lines[-1])
        self.assertIn("gh run rerun 3 --failed", lines[-1])

    def test_all_listed_pull_requests_share_one_wait_budget(self):
        api, lines = decide([run(1, CI), run(3, HOLD, status="in_progress"),
                             run(4, HOLD, status="in_progress", number=8)],
                            payload=event(numbers=(7, 8)), heads={7: HEAD, 8: HEAD})
        self.assertEqual(api.sleeps, [recheck_module.POLL_SECONDS] * recheck_module.WAIT_POLLS)
        self.assertEqual(api.posts, [])
        self.assertTrue(all("still in_progress after a wait" in line for line in lines[-2:]), lines)

    def test_a_running_merge_hold_that_started_after_the_gate_finished_is_not_waited_for(self):
        api, lines = decide([run(1, CI), run(3, HOLD, status="in_progress", started=AFTER)])
        self.assertEqual((api.sleeps, api.posts), ([], []))
        self.assertIn("started after this run finished", lines[-1])

    def test_no_rerun_when_the_head_moves_before_the_request(self):
        api, lines = decide([run(1, CI), run(3, HOLD, conclusion="failure")], heads={7: [HEAD, OTHER]})
        self.assertEqual(api.posts, [])
        self.assertIn("head moved to " + OTHER + " before the re-run", lines[-1])
        self.assertEqual(sum(path == f"/repos/{REPO}/pulls/7" for path in api.gets), 2)

    def test_the_triggering_run_counts_as_finished_while_the_listing_lags(self):
        api, _ = decide([run(1, CI, status="in_progress"), run(3, HOLD, conclusion="failure")])
        self.assertEqual(len(api.posts), 1)

    def test_every_page_of_runs_is_read(self):
        filler = [run(100 + index, CI) for index in range(150)]
        api, _ = decide(filler + [run(2, LEAVES, status="in_progress"), run(3, HOLD, conclusion="failure")])
        self.assertEqual(api.posts, [])
        self.assertEqual(sum("actions/runs?" in path for path in api.gets), 2)
        api, _ = decide(filler + [run(3, HOLD, conclusion="failure")])
        self.assertEqual(len(api.posts), 1)

    def test_a_listing_that_changes_while_paging_raises(self):
        with self.assertRaises(ValueError):
            decide([run(1, CI), run(3, HOLD, conclusion="failure")], total=5)

    def test_an_attempt_that_already_runs_is_tolerated(self):
        for code, body in ((409, b'{"message": "conflict"}'),
                           (403, b'{"message": "This workflow is already running"}')):
            with self.subTest(code=code):
                api, lines = decide(
                    [run(1, CI), run(3, HOLD, conclusion="failure")],
                    post_error=(code, body),
                    details=[run(3, HOLD, conclusion="failure"),
                             run(3, HOLD, status="in_progress", started=AFTER, attempt=2)],
                )
                self.assertEqual(len(api.posts), 1)
                self.assertIn("already has attempt 2 after the gate boundary", lines[-1])
                self.assertEqual(api.sleeps, [recheck_module.RERUN_CONFIRM_SECONDS])

    def test_a_racing_attempt_at_the_boundary_is_followed_then_rerun(self):
        api, lines = decide(
            [run(1, CI), run(3, HOLD, conclusion="failure")],
            post_error=[(409, b'{"message": "conflict"}'), None],
            details=[run(3, HOLD, status="in_progress", started=FINISHED, attempt=2),
                     run(3, HOLD, started=FINISHED, attempt=2)],
        )
        self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/rerun-failed-jobs",
                                     f"/repos/{REPO}/actions/runs/3/rerun"])
        self.assertEqual(api.sleeps, [recheck_module.RERUN_CONFIRM_SECONDS])
        self.assertIn("re-ran merge-hold run 3 (success, rerun)", lines[-1])

    def test_the_head_is_rechecked_before_a_second_rerun_request(self):
        api, lines = decide(
            [run(1, CI), run(3, HOLD, conclusion="failure")],
            heads={7: [HEAD, HEAD, OTHER]},
            post_error=[(409, b'{"message": "conflict"}')],
            details=[run(3, HOLD, status="in_progress", started=FINISHED, attempt=2),
                     run(3, HOLD, started=FINISHED, attempt=2)],
        )
        self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/rerun-failed-jobs"])
        self.assertIn("head moved to " + OTHER + " before the re-run", lines[-1])
        self.assertEqual(sum(path == f"/repos/{REPO}/pulls/7" for path in api.gets), 3)

    def test_a_generic_conflict_without_a_new_attempt_raises(self):
        api = FakeApi(
            [run(1, CI), run(3, HOLD, conclusion="failure")],
            post_error=(409, b'{"message": "conflict"}'),
            details=[run(3, HOLD, conclusion="failure")],
        )
        with self.assertRaises(error.HTTPError):
            recheck_module.recheck(event(), REPO, WATCHED, api.get, api.post, api.sleep)
        self.assertEqual(api.sleeps,
                         [recheck_module.RERUN_CONFIRM_SECONDS]
                         * (recheck_module.RERUN_CONFIRM_POLLS - 1))
        self.assertEqual(sum(path == f"/repos/{REPO}/actions/runs/3" for path in api.gets),
                         recheck_module.RERUN_CONFIRM_POLLS)

    def test_a_refused_rerun_raises(self):
        with self.assertRaises(error.HTTPError):
            decide([run(1, CI), run(3, HOLD, conclusion="failure")],
                   post_error=(403, b'{"message": "Resource not accessible by integration"}'))

    def test_malformed_trigger_values_raise_before_any_request(self):
        for payload in (event(sha="HEAD"), event(numbers=("7",)), event(numbers=(0,)),
                        event(attempt=0), event(attempt="2"), event(path=""),
                        event(action="requested")):
            with self.subTest(payload=payload["workflow_run"]):
                api = FakeApi([])
                with self.assertRaises(ValueError):
                    recheck_module.recheck(payload, REPO, WATCHED, api.get, api.post)
                self.assertEqual(api.gets, [])
        with self.assertRaises(ValueError):
            recheck_module.recheck(event(), REPO, set(), FakeApi([]).get, FakeApi([]).post)

    def test_gate_workflows_come_from_the_catalogue(self):
        with open(ROOT / recheck_module.CATALOGUE, "rb") as handle:
            watched = recheck_module.gate_workflows(tomllib.load(handle))
        self.assertIn(CI, watched)
        self.assertIn(LEAVES, watched)
        for path in (HOLD, WINDOWS, ".github/workflows/security-alerts.yml",
                     ".github/workflows/merge-hold-recheck.yml"):
            with self.subTest(path=path):
                self.assertNotIn(path, watched)

    def test_main_reports_api_errors_and_changes_nothing(self):
        with tempfile.TemporaryDirectory() as tmp:
            event_path = os.path.join(tmp, "event.json")
            with open(event_path, "w", encoding="utf-8") as handle:
                json.dump(event(), handle)
            api = FakeApi([run(1, CI), run(3, HOLD, conclusion="failure")], post_error=(500, b"{}"))
            env = {"GITHUB_EVENT_PATH": event_path, "GH_TOKEN": "x", "REPO": REPO}
            out = io.StringIO()
            with mock.patch.dict(os.environ, env), contextlib.redirect_stdout(out), \
                    mock.patch.object(recheck_module, "client", return_value=(api.get, api.post)), \
                    mock.patch.object(recheck_module, "CATALOGUE", str(ROOT / recheck_module.CATALOGUE)):
                self.assertEqual(recheck_module.main(), 1)
            self.assertIn("::error::merge-hold re-check failed", out.getvalue())
            api = FakeApi([run(1, CI), run(3, HOLD, conclusion="failure")])
            with mock.patch.dict(os.environ, env), contextlib.redirect_stdout(io.StringIO()), \
                    mock.patch.object(recheck_module, "client", return_value=(api.get, api.post)), \
                    mock.patch.object(recheck_module, "CATALOGUE", str(ROOT / recheck_module.CATALOGUE)):
                self.assertEqual(recheck_module.main(), 0)
            self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/rerun-failed-jobs"])


if __name__ == "__main__":
    main()

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


def event(trigger_id=1, kind="pull_request", head_repo=REPO, numbers=(7,), sha=HEAD):
    return {"workflow_run": {"id": trigger_id, "name": "CI", "event": kind, "head_sha": sha,
                             "updated_at": FINISHED, "repository": {"full_name": REPO},
                             "head_repository": {"full_name": head_repo},
                             "pull_requests": [{"number": number} for number in numbers]}}


class FakeApi:
    """Answers pull request reads and paged run listings; records every POST."""

    def __init__(self, runs, heads=None, post_error=None, total=None):
        self.runs, self.heads = runs, heads or {7: HEAD}
        self.post_error, self.total = post_error, total
        self.gets, self.posts = [], []

    def get(self, path):
        self.gets.append(path)
        if path.startswith(f"/repos/{REPO}/pulls/"):
            return {"head": {"sha": self.heads[int(path.rsplit("/", 1)[1])]}}
        url = urlsplit(path)
        assert url.path == f"/repos/{REPO}/actions/runs", path
        query = parse_qs(url.query)
        assert query["head_sha"] == [HEAD] and query["event"] == ["pull_request"], path
        size, page = int(query["per_page"][0]), int(query["page"][0])
        total = len(self.runs) if self.total is None else self.total
        return {"total_count": total, "workflow_runs": self.runs[(page - 1) * size:page * size]}

    def post(self, path):
        self.posts.append(path)
        if self.post_error:
            code, body = self.post_error
            raise error.HTTPError(path, code, "error", {}, io.BytesIO(body))
        return {}


def decide(runs, payload=None, **kwargs):
    api = FakeApi(runs, **kwargs)
    lines = recheck_module.recheck(payload or event(), REPO, WATCHED, api.get, api.post)
    return api, lines


class RecheckTests(TestCase):
    def test_no_rerun_while_a_gate_run_is_still_open(self):
        api, lines = decide([run(1, CI), run(2, LEAVES, status="in_progress"),
                             run(3, HOLD, conclusion="failure")])
        self.assertEqual(api.posts, [])
        self.assertIn("still open", lines[-1])

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
        # They never start a re-check when they finish, so waiting for them would stall.
        api, _ = decide([run(1, CI), run(2, WINDOWS, status="in_progress"),
                         run(5, LEAVES, status="queued", head_repo="fork/r"),
                         run(3, HOLD, conclusion="failure")])
        self.assertEqual(api.posts, [f"/repos/{REPO}/actions/runs/3/rerun-failed-jobs"])

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
                api, lines = decide([run(1, CI), run(3, HOLD, conclusion="failure")], post_error=(code, body))
                self.assertEqual(len(api.posts), 1)
                self.assertIn("already running again", lines[-1])

    def test_a_refused_rerun_raises(self):
        with self.assertRaises(error.HTTPError):
            decide([run(1, CI), run(3, HOLD, conclusion="failure")],
                   post_error=(403, b'{"message": "Resource not accessible by integration"}'))

    def test_malformed_trigger_values_raise_before_any_request(self):
        for payload in (event(sha="HEAD"), event(numbers=("7",)), event(numbers=(0,))):
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

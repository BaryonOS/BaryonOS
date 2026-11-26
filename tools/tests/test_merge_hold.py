"""Unit tests for the merge-hold decision; no network access."""

import contextlib
import io
import os
import re
import sys
import tempfile
from pathlib import Path
from unittest import TestCase, main, mock
from urllib.error import HTTPError

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
sys.path.insert(0, str(Path(__file__).resolve().parent))
import merge_hold

HEAD = "a" * 40
OTHER = "b" * 40
ENV = {"GH_TOKEN": "x", "REPO": "o/r", "EVENT_NAME": "pull_request",
       "PR_NUMBER": "7", "HEAD_SHA": HEAD}
GROUP = "c" * 40
BASE = "d" * 40
QUEUE_BRANCH = f"gh-readonly-queue/main/pr-8-{BASE}"
AHEAD = "5" * 40  # an unlanded queue commit of the entry ahead: merge_group.base_sha of a stacked group
GROUP_ENV = {"GH_TOKEN": "x", "REPO": "o/r", "EVENT_NAME": "merge_group",
             "PR_NUMBER": "", "HEAD_SHA": "",
             "BASE_SHA": BASE, "TESTED_SHA": GROUP,
             "MERGE_GROUP_HEAD_SHA": GROUP,
             "MERGE_GROUP_HEAD_REF": "refs/heads/" + QUEUE_BRANCH,
             "MERGE_GROUP_BASE_REF": "refs/heads/main",
             "MERGE_GROUP_BASE_SHA": BASE}


def comment(body, association="MEMBER"):
    return {"body": body, "author_association": association}


class DecisionTests(TestCase):
    def test_no_label_passes(self):
        self.assertTrue(merge_hold.decide(["bug"], [], HEAD)[0])

    def test_label_without_release_fails(self):
        self.assertFalse(merge_hold.decide([merge_hold.LABEL], [], HEAD)[0])

    def test_release_for_the_current_head_passes(self):
        for association in ("OWNER", "MEMBER", "COLLABORATOR"):
            with self.subTest(association=association):
                passed, _ = merge_hold.decide(
                    [merge_hold.LABEL],
                    [comment("Matrix green.\r\nacceptance-released: " + HEAD + "\n", association)],
                    HEAD)
                self.assertTrue(passed)

    def test_release_for_another_head_does_not_carry_over(self):
        comments = [comment("acceptance-released: " + OTHER)]
        self.assertFalse(merge_hold.decide([merge_hold.LABEL], comments, HEAD)[0])

    def test_untrusted_or_inexact_release_is_ignored(self):
        for body, association in (
            ("acceptance-released: " + HEAD, "CONTRIBUTOR"),
            ("acceptance-released: " + HEAD, "NONE"),
            ("acceptance-released: " + HEAD[:12], "MEMBER"),
            ("not acceptance-released: " + HEAD, "MEMBER"),
            ("acceptance-released: " + HEAD.upper(), "MEMBER"),
        ):
            with self.subTest(body=body, association=association):
                self.assertFalse(merge_hold.decide(
                    [merge_hold.LABEL], [comment(body, association)], HEAD)[0])

    def test_malformed_head_fails_closed(self):
        for head in ("", "abc", None, "g" * 40):
            with self.subTest(head=head):
                self.assertFalse(merge_hold.decide([], [], head)[0])


class MainTests(TestCase):
    def run_main(self, env, fetch, gates=(True, ["gates verified"])):
        out = io.StringIO()
        with mock.patch.dict(os.environ, env, clear=True), \
                mock.patch.object(merge_hold, "fetch_all", fetch), \
                mock.patch.object(merge_hold, "pull_request_gates", lambda *args: gates), \
                contextlib.redirect_stdout(out):
            return merge_hold.main(), out.getvalue()

    def test_gate_verdict_joins_the_hold_verdict(self):
        no_label = lambda token, url: []  # noqa: E731
        code, out = self.run_main(ENV, no_label, gates=(False, ["::error::Failed: x"]))
        self.assertEqual(code, 1)
        self.assertIn("No acceptance hold is set.", out)
        self.assertIn("::error::Failed: x", out)
        self.assertEqual(self.run_main(ENV, no_label)[0], 0)

        def broken(*args):
            raise merge_hold.subprocess.TimeoutExpired("selector", 600)

        out = io.StringIO()
        with mock.patch.dict(os.environ, ENV, clear=True), \
                mock.patch.object(merge_hold, "fetch_all", no_label), \
                mock.patch.object(merge_hold, "pull_request_gates", broken), \
                contextlib.redirect_stdout(out):
            self.assertEqual(merge_hold.main(), 1)
        self.assertIn("failing closed", out.getvalue())

    def test_reads_live_labels_and_comments(self):
        calls = []

        def fetch(token, url):
            calls.append(url)
            if url.endswith("/labels"):
                return [{"name": merge_hold.LABEL}]
            return [comment("acceptance-released: " + HEAD)]

        code, out = self.run_main(ENV, fetch)
        self.assertEqual(code, 0, out)
        self.assertEqual(calls, ["https://api.github.com/repos/o/r/issues/7/labels",
                                 "https://api.github.com/repos/o/r/issues/7/comments"])

    def test_pending_label_fails(self):
        def fetch(token, url):
            return [{"name": merge_hold.LABEL}] if url.endswith("/labels") else []

        code, out = self.run_main(ENV, fetch)
        self.assertEqual(code, 1)
        self.assertIn("::error::", out)

    def test_api_failure_and_missing_input_fail_closed(self):
        def broken(token, url):
            raise OSError("HTTP 502")

        self.assertEqual(self.run_main(ENV, broken)[0], 1)
        env = dict(ENV)
        del env["GH_TOKEN"]
        self.assertEqual(self.run_main(env, lambda t, u: [])[0], 1)
        env = dict(ENV, PR_NUMBER="seven")
        self.assertEqual(self.run_main(env, lambda t, u: [])[0], 1)
        for event in (None, "push", "pull_request_target"):
            with self.subTest(event=event):
                env = {k: v for k, v in ENV.items() if k != "EVENT_NAME"}
                if event:
                    env["EVENT_NAME"] = event
                self.assertEqual(self.run_main(env, lambda t, u: [])[0], 1)

    def test_unexpected_payload_fails_closed(self):
        def odd(token, url):
            raise TypeError("Unexpected API response")

        self.assertEqual(self.run_main(ENV, odd)[0], 1)


def entry(position, number, commit, head, state="OPEN", base="main"):
    return {"position": position, "headCommit": {"oid": commit},
            "pullRequest": {"number": number, "state": state,
                            "baseRefName": base, "headRefOid": head}}


def runs(*states):
    """Check-run list answer: one run per AHEAD_CHECKS name with the given status."""
    return {"check_runs": [{"name": name, "status": state,
                            "conclusion": "success" if state == "completed" else None}
                           for name, state in zip(merge_hold.AHEAD_CHECKS, states)]}


DONE = runs("completed", "completed", "completed")


def gate_run(head, conclusion="success", run_id=1, status="completed", path=".github/workflows/merge-hold.yml"):
    """A pull_request merge-hold run on a member's head (the merge_group gate rule)."""
    number = {HEAD: 7, OTHER: 8}.get(head, 9)
    return {"id": run_id, "path": path, "head_sha": head, "status": status, "conclusion": conclusion,
            "event": "pull_request", "repository": {"full_name": "o/r"},
            "pull_requests": [{"number": number, "base": {"ref": "main"}}],
            "created_at": f"2026-11-24T20:{run_id % 60:02d}:00Z"}
E = "e" * 40


def per_read(*answers):
    """One answer per read, in turn; the last one holds (run_group rereads and group_jobs)."""
    reads = []

    def answer():
        reads.append(1)
        return answers[min(len(reads), len(answers)) - 1]
    return answer


class MergeGroupTests(TestCase):
    """PR 7 is ahead of PR 8; group commit GROUP contains both."""

    def run_group(self, nodes, labels=None, comments=None, trees=None,
                  env=None, answer=None, checks=None, ahead=None, on_sleep=None,
                  commits=None, gate_runs=None, group_runs=None, group_jobs=None, plan=None, ancestor=0,
                  rereads=None, plan_seconds=0, decision_seconds=0, git_seconds=0, trusted=BASE):
        """checks answer polls of the group commit, commits[sha] those of sha, ahead the rest.

        Run listings are keyed by the URL's event and head_sha and returned verbatim, as
        GitHub would list them: gate_runs[sha] answers event=pull_request (default: one
        successful merge-hold run), group_runs[sha] answers event=merge_group (default:
        the passing gate runs of GROUP, and for another queue entry commit one successful
        merge-hold run of its pull request, entry_hold_run); group_jobs[run id] lists their
        jobs (default for a listed merge-hold run: its one merge-hold job, as the run shows it).
        rereads[run id] overlays the fields a re-read of that run, as last listed, shows (a
        callable: one answer per re-read, see per_read). plan is what the mocked selector
        returns (default: group_plan()); self.plans records its calls, self.gits the git calls.
        `git merge-base <queue commit> origin/main` prints trusted (None: no merge base), and
        `--is-ancestor` exits ancestor, or 1 for the (older, newer) pairs in a set ancestor.
        Fake clock: only sleeps, the selector (plan_seconds), each git call (git_seconds) and
        the decision after the wait (decision_seconds, charged at the first label read) take
        time; self.clock[0] is the time at which main() returned. self.calls is the call
        log while it runs.
        """
        labels, comments, trees = labels or {}, comments or {}, trees or {}
        polls = {sha: list(answers) for sha, answers in (commits or {}).items()}
        polls[GROUP], polls[None] = list(checks or [DONE]), list(ahead or [DONE])
        group_runs = {GROUP: [group_run(1, CI), group_run(2, LEAVES)]} if group_runs is None else group_runs
        group_jobs = GOOD_JOBS if group_jobs is None else group_jobs
        calls = self.calls = []
        self.plans, self.gits = [], []
        clock = self.clock = [0.0]
        seen = {}  # merge_group runs by id, as last listed

        def recompute(*args, **kwargs):
            self.plans.append((args, kwargs))
            clock[0] += plan_seconds
            return plan or group_plan()

        def git(*args):
            # Ancestry as given; no base catalogue, so the wait budget is WAIT_SECONDS.
            self.gits.append(args)
            clock[0] += git_seconds
            if args[:2] == ("merge-base", "--is-ancestor"):
                return (ancestor if isinstance(ancestor, int) else int(tuple(args[2:]) in ancestor), "")
            if args[0] == "merge-base":
                self.assertEqual(args[1:], (GROUP, merge_hold.MAIN_REF))
                return (0, trusted) if trusted else (1, "")
            return (128, "")

        def fetch(token, url):
            calls.append(url)
            number = int(url.split("/")[-2])
            if url.endswith("/labels"):
                if [read for read in calls if read.endswith("/labels")] == [url]:
                    clock[0] += decision_seconds
                return [{"name": name} for name in labels.get(number, [])]
            return comments.get(number, [])

        def call(token, url, payload=None):
            calls.append(url)
            if "/check-runs?" in url:
                queue = polls.get(url.split("/commits/")[1].split("/")[0], polls[None])
                answer_ = queue.pop(0) if len(queue) > 1 else queue[0]
                if callable(answer_):  # an answer that depends on the URL, e.g. its page
                    answer_ = answer_(url)
                if isinstance(answer_, Exception):
                    raise answer_
                return answer_
            if "/jobs?" in url:
                run_id = int(url.split("/runs/")[1].split("/")[0])
                run = seen.get(run_id, {})
                listed = group_jobs.get(run_id, [job(10 * run_id, run_id, "merge-hold", run.get("conclusion"),
                                                     run.get("status"), run.get("run_attempt"))]
                                        if run.get("path") == merge_hold.GATE_WORKFLOW else [])
                listed = listed() if callable(listed) else listed  # an answer per read, e.g. one that fails once
                return {"total_count": len(listed), "jobs": listed}
            if "/actions/runs?" in url:
                head = url.split("head_sha=")[1].split("&")[0]
                event = url.split("event=")[1].split("&")[0]
                if event == "merge_group":
                    number = next((node["pullRequest"]["number"] for node in nodes
                                   if (node.get("headCommit") or {}).get("oid") == head), 9)
                    listed = group_runs.get(head, [] if head == GROUP else [entry_hold_run(head, number)])
                    seen.update((run["id"], run) for run in listed)
                else:
                    self.assertEqual(event, "pull_request")
                    listed = (gate_runs or {}).get(head, [gate_run(head)])
                return {"total_count": len(listed), "workflow_runs": listed}
            if "/actions/runs/" in url:  # re-read of one listed merge_group run
                run_id = int(url.rsplit("/", 1)[1])
                change = (rereads or {}).get(run_id) or {}
                return {**seen[run_id], **(change() if callable(change) else change)}
            if url.endswith("/graphql"):
                self.assertEqual(payload["variables"], {"owner": "o", "name": "r"})
                return answer if answer is not None else {"data": {"repository": {
                    "mergeQueue": {"entries": {"pageInfo": {"hasNextPage": False},
                                               "nodes": nodes}}}}}
            sha = url.rsplit("/", 1)[1]
            return {"tree": {"sha": trees.get(sha, "1" * 40)}}

        def sleep(seconds):
            # Fake clock: each poll advances time, so the deadline ends any wait.
            clock[0] += seconds
            calls.append("sleep")
            if on_sleep:
                on_sleep(labels, comments, nodes)

        out = io.StringIO()
        with mock.patch.dict(os.environ, env or GROUP_ENV, clear=True), \
                mock.patch.object(merge_hold, "fetch_all", fetch), \
                mock.patch.object(merge_hold, "call", call), \
                mock.patch.object(merge_hold, "recompute_plan", recompute), \
                mock.patch.object(merge_hold, "git", git), \
                mock.patch.object(merge_hold.time, "sleep", sleep), \
                mock.patch.object(merge_hold.time, "monotonic", lambda: clock[0]), \
                contextlib.redirect_stdout(out):
            return merge_hold.main(), out.getvalue(), calls

    QUEUE = [entry(1, 7, "e" * 40, HEAD), entry(2, 8, GROUP, OTHER),
             entry(3, 9, "f" * 40, "9" * 40)]

    def test_unheld_group_passes_and_checks_every_contained_pr(self):
        code, out, _ = self.run_group(self.QUEUE)
        self.assertEqual(code, 0, out)
        self.assertIn("#7:", out)
        self.assertIn("#8:", out)
        self.assertNotIn("#9:", out)

    def test_member_needs_its_own_gate_verdict_and_the_landing_tree(self):
        one = [entry(1, 8, GROUP, OTHER)]
        # D1 step 2: an unheld member may land another tree; the queue-commit gate report covers it.
        code, out, _ = self.run_group(one, trees={GROUP: "1" * 40, OTHER: "2" * 40})
        self.assertEqual(code, 0, out)
        self.assertIn("2 applicable gate(s) passed; 1 filtered out; 0 pending; 0 failed.", out)
        for runs_, why in (([], "no run"), ([gate_run(OTHER, "failure")], "failed"),
                           ([gate_run(OTHER, run_id=1), gate_run(OTHER, "failure", run_id=2)], "newer failed"),
                           ([gate_run(OTHER, None, status="in_progress")], "running"),
                           ([gate_run(OTHER, path=".github/workflows/ci.yml")], "other workflow")):
            with self.subTest(why=why):
                code, out, _ = self.run_group(one, gate_runs={OTHER: runs_})
                self.assertEqual(code, 1, out)
                self.assertIn("No successful pull_request merge-hold", out)
        code, out, _ = self.run_group(one, gate_runs={OTHER: [gate_run(OTHER, "failure", run_id=1),
                                                              gate_run(OTHER, run_id=2)]})
        self.assertEqual(code, 0, out)  # a newer successful re-run decides
        # A newer successful merge-hold of another pull request or base on the same head never counts.
        mine = dict(gate_run(OTHER, "failure", run_id=1), pull_requests=[{"number": 8, "base": {"ref": "main"}}])
        for other in ([{"number": 9, "base": {"ref": "main"}}], [{"number": 8, "base": {"ref": "release"}}]):
            with self.subTest(other=other):
                code, out, _ = self.run_group(one, gate_runs={OTHER: [mine, dict(gate_run(OTHER, run_id=2),
                                                                                  pull_requests=other)]})
                self.assertEqual(code, 1, out)
        code, out, _ = self.run_group(one, gate_runs={OTHER: [dict(gate_run(OTHER, run_id=2),
                                                                   pull_requests=[{"number": 8, "base": {"ref": "main"}}])]})
        self.assertEqual(code, 0, out)
        code, out, _ = self.run_group(one, gate_runs={OTHER: [dict(gate_run(OTHER, run_id=2), pull_requests=[])]})
        self.assertEqual(code, 1, out)  # an unlinked (fork) run never counts

    def test_merge_group_budget_comes_from_the_base_catalogue(self):
        catalogue = '[policy.budgets]\nmerge_group_wait_minutes = 5\n'
        with mock.patch.object(merge_hold, "git", return_value=(0, catalogue)):
            self.assertEqual(merge_hold.base_wait_seconds(BASE), 300)
        for answer in ((0, "[policy]\n"), (128, ""), (0, "not toml ["),
                       (0, '[policy.budgets]\nmerge_group_wait_minutes = 99\n'),
                       (0, '[policy.budgets]\nmerge_group_wait_minutes = true\n')):
            with self.subTest(answer=answer), mock.patch.object(merge_hold, "git", return_value=answer):
                self.assertEqual(merge_hold.base_wait_seconds(BASE), merge_hold.WAIT_SECONDS)

    # Budget clock (#494): host-quality keeps running, so every poll is pending.
    OPEN = runs("in_progress", "completed")

    def test_a_slow_plan_shortens_the_gate_wait(self):
        """The budget is counted from before the plan and keeps the decision reserve back:
        (80 min - 3 min - plan) / 1 min polls, rounded up, so the wait ends past its deadline."""
        for plan_seconds, sleeps in ((0, 77), (7, 77), (merge_hold.SELECTOR_TIMEOUT, 67)):
            with self.subTest(plan_seconds=plan_seconds):
                code, out, calls = self.run_group(self.QUEUE, checks=[self.OPEN], plan_seconds=plan_seconds)
                self.assertEqual(code, 1, out)
                self.assertIn("did not complete in time", out)
                self.assertEqual(calls.count("sleep"), sleeps)
                self.assertEqual(self.clock[0], plan_seconds + sleeps * merge_hold.POLL_SECONDS)
                self.assertFalse([url for url in calls if url.endswith("/labels")])
        # The git calls before the wait, the trusted-base checks and the catalogue read
        # (base_wait_seconds), are inside the clock too: (80 min - 3 min - n x 120 s) / 1 min.
        # Unstacked, T is the merge group base and needs no ancestry check of its own (n = 3).
        code, out, calls = self.run_group(self.QUEUE, checks=[self.OPEN], git_seconds=120)
        self.assertEqual(code, 1, out)
        self.assertEqual([args[:2] for args in self.gits], [("merge-base", GROUP), ("merge-base", "--is-ancestor"),
                                                            ("cat-file", "blob")])
        self.assertEqual(calls.count("sleep"), 71)
        self.assertEqual(self.clock[0], 3 * 120 + 71 * merge_hold.POLL_SECONDS)
        # Stacked on an unlanded entry ahead, T is also checked against the merge group base (n = 4).
        code, out, calls = self.run_group(self.QUEUE, checks=[self.OPEN], git_seconds=120,
                                          env=dict(GROUP_ENV, MERGE_GROUP_BASE_SHA=AHEAD))
        self.assertEqual(code, 1, out)
        self.assertEqual(self.gits[1:3], [("merge-base", "--is-ancestor", BASE, AHEAD),
                                          ("merge-base", "--is-ancestor", AHEAD, GROUP)])
        self.assertEqual(calls.count("sleep"), 69)
        self.assertEqual(self.clock[0], 4 * 120 + 69 * merge_hold.POLL_SECONDS)

    def test_plan_wait_and_decision_stay_within_the_budget_plus_the_reserve(self):
        """For every catalogue budget B and plan time P (polls take no time here): the plan and
        the wait end within max(B, P), and with a decision that takes its whole reserve R the
        total stays within max(B, P) + R. B + R alone does not bound a plan longer than B."""
        reserve = merge_hold.DECISION_RESERVE_SECONDS
        for budget in (merge_hold.WAIT_SECONDS, 5 * 60, 0):
            plans = {0, 7, 61, merge_hold.SELECTOR_TIMEOUT, budget - reserve - 1, budget - reserve, budget,
                     budget + 120}
            for plan_seconds in sorted(plan for plan in plans if plan >= 0):
                with self.subTest(budget=budget, plan_seconds=plan_seconds), \
                        mock.patch.object(merge_hold, "WAIT_SECONDS", budget):
                    code, out, calls = self.run_group(self.QUEUE, checks=[self.OPEN], plan_seconds=plan_seconds)
                    self.assertEqual(code, 1, out)
                    self.assertIn("did not complete in time", out)
                    self.assertLessEqual(self.clock[0], max(budget, plan_seconds))
                    self.assertGreaterEqual(self.clock[0], max(budget - reserve, plan_seconds))
                    # The gates finish at the last poll the wait still makes; then the decision runs.
                    sleeps = calls.count("sleep")
                    code, out, calls = self.run_group(self.QUEUE, checks=[self.OPEN] * sleeps + [DONE],
                                                      plan_seconds=plan_seconds, decision_seconds=reserve)
                    self.assertEqual(code, 0, out)
                    self.assertEqual(calls.count("sleep"), sleeps)
                    self.assertLessEqual(self.clock[0], max(budget, plan_seconds) + reserve)

    def test_a_plan_that_uses_up_the_budget_fails_closed_without_waiting(self):
        budget, reserve = merge_hold.WAIT_SECONDS, merge_hold.DECISION_RESERVE_SECONDS
        for plan_seconds in (budget - reserve, budget, budget + 600):
            with self.subTest(plan_seconds=plan_seconds):
                code, out, calls = self.run_group(self.QUEUE, checks=[self.OPEN], plan_seconds=plan_seconds)
                self.assertEqual(code, 1, out)
                self.assertIn(f"::error::merge-hold could not decide (required checks did not complete in "
                              f"time: ['host-quality on {GROUP}']); failing closed.", out)
                self.assertEqual(len(self.plans), 1)
                self.assertNotIn("sleep", calls)
                self.assertEqual(len([url for url in calls if "/check-runs?" in url]), 1)  # one poll, as a 0 budget
                self.assertFalse([url for url in calls if url.endswith("/labels")])
                self.assertEqual(self.clock[0], plan_seconds)
        # As with a 0 budget, that one poll decides when nothing is open any more.
        code, out, calls = self.run_group(self.QUEUE, plan_seconds=budget)
        self.assertEqual(code, 0, out)
        self.assertNotIn("sleep", calls)

    def test_the_budget_arithmetic_fits_the_job_timeout(self):
        """plan + wait + reserve < job timeout: the docstring of merge_group_hold, against merge-hold.yml."""
        workflow = (Path(__file__).resolve().parents[2] / merge_hold.GATE_WORKFLOW).read_text(encoding="utf-8")
        timeouts = re.findall(r"^\s+timeout-minutes: (\d+)$", workflow, re.M)
        self.assertEqual(timeouts, ["90"])
        self.assertEqual((merge_hold.WAIT_SECONDS, merge_hold.MAX_WAIT_SECONDS), (80 * 60, 80 * 60))
        self.assertGreaterEqual(merge_hold.DECISION_RESERVE_SECONDS, merge_hold.POLL_SECONDS)
        longest_plan = merge_hold.SELECTOR_TIMEOUT + 2 * 120  # the selector and two git calls
        self.assertLess(max(merge_hold.MAX_WAIT_SECONDS, longest_plan + merge_hold.POLL_SECONDS)
                        + merge_hold.DECISION_RESERVE_SECONDS, int(timeouts[0]) * 60)
        # The docstring's assumption: up to the wait's last poll at most 78 minutes, so the last
        # poll, the decision and the steps before the script have about 12 minutes.
        before_last_poll = max(merge_hold.MAX_WAIT_SECONDS - merge_hold.DECISION_RESERVE_SECONDS
                               + merge_hold.POLL_SECONDS, longest_plan)
        self.assertEqual((before_last_poll, int(timeouts[0]) * 60 - before_last_poll), (78 * 60, 12 * 60))

    def test_the_decision_after_the_wait_makes_the_counted_requests(self):
        """The requests of the decision after the wait, counted from the moment the wait returns (one
        page each), as merge_group_hold's docstring states them for its 7-minute assumption: 2 + 2N + 4M
        for N counted runs and M members with no member lifted; 2 + 2N + 8M - 8 (M >= 2) in an ordinary
        queue, where every member ahead but the first is lifted; at most 4N + 8M - 1 (every run read again;
        every member ahead lifted, which reads its entry's merge-hold run, that run's jobs, a re-read of it
        and the entry's check runs, and this group's own entry labelled and released)."""
        ends = []
        real = merge_hold.wait_for_checks

        def wait(*args, **kwargs):
            early = real(*args, **kwargs)
            ends.append(len(self.calls))
            return early

        for n, m in ((1, 1), (3, 2), (5, 4)):
            workflows = [f".github/workflows/gate{i}.yml" for i in range(1, n + 1)]
            plan = group_plan(excluded=[], selected=[row("merge-hold#merge-hold", merge_hold.GATE_WORKFLOW,
                                                         "merge-hold", "core")]
                              + [row(f"gate{i}#g", path, f"g{i}", "core") for i, path in enumerate(workflows, 1)])
            listed = [group_run(i, path) for i, path in enumerate(workflows, 1)]
            commits, heads = [f"{0xc0 + i:040x}" for i in range(m - 1)], [f"{0xa0 + i:040x}" for i in range(m - 1)]
            queue = [entry(i + 1, 10 + i, commits[i], heads[i]) for i in range(m - 1)] + [entry(m, 8, GROUP, OTHER)]
            gate_runs = {head: [dict(gate_run(head), pull_requests=[{"number": 10 + i, "base": {"ref": "main"}}])]
                         for i, head in enumerate(heads)}
            expected = {"none lifted": 2 + 2 * n + 4 * m, "ordinary": 2 + 2 * n + 4 * m + 4 * max(m - 2, 0),
                        "worst": 4 * n + 8 * m - 1}
            for case, lifted in (("none lifted", []), ("ordinary", heads[1:]), ("worst", heads)):
                worst = case == "worst"
                with self.subTest(n=n, m=m, case=case), mock.patch.object(merge_hold, "wait_for_checks", wait):
                    ends.clear()
                    attempt = 2 if worst else 1
                    code, out, calls = self.run_group(
                        queue, plan=plan, gate_runs=gate_runs,
                        group_runs={GROUP: listed, **{commit: [entry_hold_run(commit, 10 + i, run_id=70 + i)]
                                                      for i, commit in enumerate(commits)}},
                        group_jobs={i: [job(10 * i, i, f"g{i}", attempt=attempt)] for i in range(1, n + 1)},
                        rereads={i: {"run_attempt": 2} for i in range(1, n + 1)} if worst else None,
                        trees={sha: "2" * 40 for sha in lifted},
                        labels={8: [merge_hold.LABEL]} if worst else None,
                        comments={8: [comment("acceptance-released: " + OTHER)]} if worst else None)
                    self.assertEqual(code, 0, out)
                    self.assertEqual(len(ends), 1)
                    after = calls[ends[0]:]
                    self.assertNotIn("sleep", after)
                    self.assertEqual(len(after), expected[case], after)
                    if m >= 2 and case == "ordinary":
                        self.assertEqual(len(after), 2 + 2 * n + 8 * m - 8)
                    self.assertEqual(len([url for url in after if "/runs/7" in url]), 2 * len(lifted))

    def test_released_pr_with_the_landing_tree_passes(self):
        code, out, _ = self.run_group(
            [entry(1, 8, GROUP, OTHER)],
            labels={8: [merge_hold.LABEL]},
            comments={8: [comment("acceptance-released: " + OTHER)]},
            trees={GROUP: "1" * 40, OTHER: "1" * 40})
        self.assertEqual(code, 0, out)

    def test_released_hold_rejects_matching_malformed_tree_ids(self):
        # Unknown trees must never count as the same accepted tree, even if
        # both REST replies contain the same malformed value.
        for invalid in (None, "", "not-a-tree-sha", "1" * 39, "1" * 41, 123):
            with self.subTest(tree_sha=invalid):
                code, out, _ = self.run_group(
                    [entry(1, 8, GROUP, OTHER)],
                    labels={8: [merge_hold.LABEL]},
                    comments={8: [comment("acceptance-released: " + OTHER)]},
                    trees={GROUP: invalid, OTHER: invalid})
                self.assertEqual(code, 1, out)
                self.assertIn("tree SHA", out)

    def test_unreleased_pending_pr_ahead_in_the_group_fails(self):
        code, out, _ = self.run_group(self.QUEUE, labels={7: [merge_hold.LABEL]})
        self.assertEqual(code, 1)
        self.assertIn("#7: ::error::", out)

    def test_release_does_not_cover_a_different_landing_tree(self):
        code, out, _ = self.run_group(
            self.QUEUE,
            labels={8: [merge_hold.LABEL]},
            comments={8: [comment("acceptance-released: " + OTHER)]},
            trees={GROUP: "1" * 40, OTHER: "2" * 40})
        self.assertEqual(code, 1)
        self.assertIn("#8: ::error::", out)

    def test_released_pr_ahead_does_not_block_an_unheld_pr_behind_it(self):
        # PR 7's own entry commit (group 1) has PR 7's tree; group 2 adds PR 8.
        queue = [entry(1, 7, "e" * 40, HEAD), entry(2, 8, GROUP, OTHER)]
        code, out, _ = self.run_group(
            queue,
            labels={7: [merge_hold.LABEL]},
            comments={7: [comment("acceptance-released: " + HEAD)]},
            trees={"e" * 40: "1" * 40, HEAD: "1" * 40, GROUP: "2" * 40, OTHER: "2" * 40})
        self.assertEqual(code, 0, out)
        # D1 step 2: PR 8's entry lands another tree (PR 7's change on top); the gates
        # judged on the queue commit cover that tree, so it passes while they pass.
        trees = {"e" * 40: "1" * 40, HEAD: "1" * 40, GROUP: "2" * 40, OTHER: "3" * 40}
        released = {7: [comment("acceptance-released: " + HEAD)]}
        code, out, _ = self.run_group(queue, labels={7: [merge_hold.LABEL]}, comments=released, trees=trees)
        self.assertEqual(code, 0, out)
        self.assertIn("#8: No acceptance hold is set.", out)
        code, out, _ = self.run_group(queue, labels={7: [merge_hold.LABEL]}, comments=released, trees=trees,
                                      group_jobs={**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", "failure")]})
        self.assertEqual(code, 1, out)
        self.assertIn("::error::Failed: quality (viewer): failure in run 2", out)
        self.assertIn("#8: ::error::The applicable gates ran on", out)  # round 2: not clean, so the base rule holds

    def test_released_pr_ahead_whose_own_entry_lands_another_tree_fails(self):
        queue = [entry(1, 7, "e" * 40, HEAD), entry(2, 8, GROUP, OTHER)]
        code, out, _ = self.run_group(
            queue,
            labels={7: [merge_hold.LABEL]},
            comments={7: [comment("acceptance-released: " + HEAD)]},
            trees={"e" * 40: "1" * 40, HEAD: "2" * 40, GROUP: "3" * 40})
        self.assertEqual(code, 1)
        self.assertIn("#7: ::error::", out)

    def test_waits_for_required_checks_before_reading_labels(self):
        def add_hold(labels, comments, nodes):
            labels[8] = [merge_hold.LABEL]

        code, out, calls = self.run_group(
            self.QUEUE,
            checks=[runs("in_progress", "completed"), runs("completed", "queued"), DONE],
            on_sleep=add_hold)
        self.assertEqual(code, 1, out)
        self.assertIn("#8: ::error::", out)
        polls = [i for i, url in enumerate(calls) if f"/commits/{GROUP}/check-runs?" in url]
        self.assertEqual(len(polls), 3)
        self.assertEqual(calls.count("sleep"), 2)
        # The queue is read only once the group's own checks are done.
        self.assertGreater(min(i for i, url in enumerate(calls) if url.endswith("/graphql")),
                           polls[-1])
        self.assertLabelsReadAfterWait(calls)
        poll = calls[polls[0]]
        self.assertIn(f"/commits/{GROUP}/check-runs?", poll)
        self.assertIn(f"app_id={merge_hold.ACTIONS_APP_ID}", poll)

    def test_required_checks_beyond_the_first_page_are_read_in_the_same_poll(self):
        # With every gate workflow on merge_group a queue commit can carry more than
        # 100 check runs; a required check on page 2 must not make the group wait.
        filler = {"check_runs": [{"name": f"filler {index}", "status": "in_progress",
                                  "conclusion": None} for index in range(100)]}
        # host-quality completes on page 1 and qemu-smoke on page 2: both pages must count.
        first = {"check_runs": filler["check_runs"][:99] + DONE["check_runs"][:1]}
        second = {"check_runs": DONE["check_runs"][1:]}
        code, out, calls = self.run_group(
            self.QUEUE, checks=[lambda url: second if url.endswith("&page=2") else first])
        self.assertEqual(code, 0, out)
        self.assertEqual(calls.count("sleep"), 0)
        polls = [url for url in calls if f"/commits/{GROUP}/check-runs?" in url]
        self.assertEqual([url.rsplit("&", 1)[1] for url in polls], ["page=1", "page=2"])
        # #494: check runs have their own cap (justified at CHECK_RUN_MAX_PAGES); the other
        # listings keep MAX_PAGES.
        cap = merge_hold.CHECK_RUN_MAX_PAGES
        self.assertEqual((cap, merge_hold.MAX_PAGES), (5, 50))
        # cap - 1 full pages and a short last page are still read in one poll.
        code, out, calls = self.run_group(
            self.QUEUE, checks=[lambda url: DONE if url.endswith(f"&page={cap}") else filler])
        self.assertEqual(code, 0, out)
        polls = [url for url in calls if f"/commits/{GROUP}/check-runs?" in url]
        self.assertEqual([url.rsplit("&", 1)[1] for url in polls], [f"page={page}" for page in range(1, cap + 1)])
        # Past cap full pages the read fails and the group fails closed at the deadline.
        code, out, calls = self.run_group(self.QUEUE, checks=[lambda url: filler])
        self.assertEqual(code, 1, out)
        self.assertIn("Too many pages", out)
        polls = [url for url in calls if f"/commits/{GROUP}/check-runs?" in url]
        self.assertEqual(len([url for url in polls if url.endswith("&page=1")]), calls.count("sleep") + 1)
        # Each poll requests exactly cap pages, never one past it.
        self.assertEqual(len(polls), cap * (calls.count("sleep") + 1))

    def assertLabelsReadAfterWait(self, calls):
        last_poll = max(i for i, url in enumerate(calls) if "/check-runs?" in url)
        first_label = min(i for i, url in enumerate(calls) if url.endswith("/labels"))
        self.assertGreater(first_label, last_poll)

    def test_waits_for_the_entry_ahead_before_reading_labels(self):
        # PR 7 ahead still runs merge-hold; a hold set on PR 8 meanwhile is seen.
        def add_hold(labels, comments, nodes):
            labels[8] = [merge_hold.LABEL]

        code, out, calls = self.run_group(
            self.QUEUE,
            ahead=[runs("completed", "completed", "in_progress"),
                   {"check_runs": []}, DONE],
            on_sleep=add_hold)
        self.assertEqual(code, 1, out)
        self.assertIn("#8: ::error::", out)
        self.assertEqual(calls.count("sleep"), 2)
        self.assertEqual(len([url for url in calls if f"/commits/{E}/check-runs?" in url]), 3)
        self.assertFalse([url for url in calls if "/commits/" + "f" * 40 in url])
        self.assertLabelsReadAfterWait(calls)

    def test_entry_ahead_whose_merge_hold_ended_early_does_not_end_the_wait(self):
        # A merge-hold ahead that did not succeed may not have waited, so the
        # entries ahead of it may still run; a hold set on PR 7 meanwhile must
        # be seen by this group, however many such entries lie in between.
        a1, a2, a3 = "1" * 40, "2" * 40, "3" * 40

        def ended(conclusion):
            run = {"name": "merge-hold", "status": "completed"}
            if conclusion != "absent":
                run["conclusion"] = conclusion
            return {"check_runs": [run] + [{"name": name, "status": "completed",
                                            "conclusion": "success"}
                                           for name in merge_hold.WAIT_CHECKS]}

        def add_hold(labels, comments, nodes):
            labels[7] = [merge_hold.LABEL]

        head = [runs("in_progress", "in_progress", "in_progress"),
                runs("completed", "completed", "in_progress"), DONE]
        cases = [(c, [entry(1, 7, a1, HEAD), entry(2, 9, a2, "9" * 40),
                      entry(3, 8, GROUP, OTHER)], {a2: [ended(c)], a1: head})
                 for c in ("failure", "cancelled", "timed_out", "skipped", "neutral",
                           "action_required", None, "absent")]
        cases.append(("two hops", [entry(1, 7, a1, HEAD), entry(2, 9, a2, "9" * 40),
                                   entry(3, 6, a3, "6" * 40), entry(4, 8, GROUP, OTHER)],
                      {a3: [ended("cancelled")], a2: [ended("failure")], a1: head}))
        for name, queue, commits in cases:
            with self.subTest(conclusion=name):
                code, out, calls = self.run_group(queue, commits=commits, on_sleep=add_hold)
                self.assertEqual(code, 1, out)
                self.assertIn("#7: ::error::", out)
                self.assertEqual(calls.count("sleep"), 2)
                self.assertLabelsReadAfterWait(calls)

    def test_successful_entry_ahead_ends_the_walk(self):
        a1, a2 = "1" * 40, "2" * 40
        code, out, calls = self.run_group(
            [entry(1, 7, a1, HEAD), entry(2, 9, a2, "9" * 40), entry(3, 8, GROUP, OTHER)],
            commits={a2: [DONE], a1: [runs("in_progress", "in_progress", "in_progress")]})
        self.assertEqual(code, 0, out)
        self.assertEqual(calls.count("sleep"), 0)
        self.assertFalse([url for url in calls if f"/commits/{a1}/" in url])

    def test_entry_ahead_that_leaves_the_queue_ends_the_wait(self):
        def merged(labels, comments, nodes):
            del nodes[0]

        code, out, calls = self.run_group(
            [entry(1, 7, E, HEAD), entry(2, 8, GROUP, OTHER)],
            ahead=[runs("completed", "completed", "in_progress")], on_sleep=merged)
        self.assertEqual(code, 0, out)
        self.assertNotIn("#7:", out)
        self.assertEqual(calls.count("sleep"), 1)

    def test_unfinished_entry_ahead_fails_closed_at_the_deadline(self):
        for ahead in ([runs("completed", "completed", "queued")], [{"check_runs": []}],
                      [OSError("HTTP 403 rate limited")]):
            with self.subTest(ahead=ahead), mock.patch.object(merge_hold, "WAIT_SECONDS", 0):
                code, out, calls = self.run_group(self.QUEUE, ahead=ahead)
                self.assertEqual(code, 1, out)
                self.assertIn("::error::", out)
                self.assertFalse([url for url in calls if url.endswith("/labels")])

    def test_unbuilt_entries_behind_the_group_are_ignored(self):
        # headCommit and pullRequest are nullable in GraphQL (MergeQueueEntry).
        queue = [entry(1, 7, E, HEAD), entry(2, 8, GROUP, OTHER),
                 {"position": 3, "headCommit": None, "pullRequest": None},
                 {"position": 4, "headCommit": None,
                  "pullRequest": {"number": 9, "state": "OPEN", "baseRefName": "main",
                                  "headRefOid": "9" * 40}},
                 {"position": 5, "headCommit": {"oid": None}, "pullRequest": None}]
        code, out, _ = self.run_group(queue)
        self.assertEqual(code, 0, out)
        self.assertIn("#7:", out)
        self.assertIn("#8:", out)
        self.assertNotIn("#9:", out)

    def test_entry_ahead_with_missing_data_fails_closed(self):
        for ahead in ({"position": 1, "headCommit": None, "pullRequest": None},
                      {"position": 1, "headCommit": None,
                       "pullRequest": entry(1, 7, E, HEAD)["pullRequest"]},
                      {"position": 1, "headCommit": {"oid": E}, "pullRequest": None}):
            with self.subTest(ahead=ahead), mock.patch.object(merge_hold, "WAIT_SECONDS", 0):
                code, out, calls = self.run_group([ahead, entry(2, 8, GROUP, OTHER)])
                self.assertEqual(code, 1, out)
                self.assertIn("::error::", out)
                self.assertFalse([url for url in calls if url.endswith("/labels")])

    def test_entry_ahead_without_a_commit_keeps_the_run_waiting(self):
        # Real wait budget: the run polls until the entry ahead is built, and a
        # hold set on that pull request meanwhile is seen.
        pr7 = entry(1, 7, E, HEAD)["pullRequest"]

        def build(labels, comments, nodes):
            nodes[0]["headCommit"] = {"oid": E}
            labels[7] = [merge_hold.LABEL]

        code, out, calls = self.run_group(
            [{"position": 1, "headCommit": None, "pullRequest": pr7}, entry(2, 8, GROUP, OTHER)],
            on_sleep=build)
        self.assertEqual(code, 1, out)
        self.assertIn("#7: ::error::", out)
        self.assertEqual(calls.count("sleep"), 1)
        self.assertLabelsReadAfterWait(calls)

    def test_merged_entry_ahead_counts_as_already_on_main(self):
        # GitHub may still list an entry ahead as MERGED right after its
        # merge-hold succeeds; it is on main, so it neither blocks nor counts.
        code, out, calls = self.run_group(
            [entry(1, 7, E, HEAD, state="MERGED"), entry(2, 8, GROUP, OTHER)],
            labels={7: [merge_hold.LABEL]})
        self.assertEqual(code, 0, out)
        self.assertNotIn("#7:", out)
        self.assertIn("#8:", out)
        self.assertEqual(calls.count("sleep"), 0)
        self.assertFalse([url for url in calls if "/issues/7/" in url])

    def test_merged_entry_skip_stays_narrow(self):
        merged_ahead = entry(1, 6, "1" * 40, "6" * 40, state="MERGED")
        for queue, error in (
            ([entry(1, 8, GROUP, OTHER, state="MERGED")], "::error::merge-hold"),
            ([entry(1, 7, E, HEAD, state="MERGED", base="release"),
              entry(2, 8, GROUP, OTHER)], "::error::merge-hold"),
            ([entry(1, 7, E, HEAD, state="CLOSED"), entry(2, 8, GROUP, OTHER)],
             "::error::merge-hold"),
            ([merged_ahead, entry(2, 7, E, HEAD), entry(3, 8, GROUP, OTHER)], "#7: ::error::"),
        ):
            with self.subTest(queue=queue), mock.patch.object(merge_hold, "WAIT_SECONDS", 0):
                code, out, _ = self.run_group(queue, labels={7: [merge_hold.LABEL]})
                self.assertEqual(code, 1, out)
                self.assertIn(error, out)

    def test_group_that_left_the_queue_fails_after_two_reads(self):
        # Real wait budget: an ejected or rebuilt group can never merge, so two
        # consecutive well-formed reads lacking it, one poll apart, end the wait
        # instead of polling to the deadline. Only the group's own pull request
        # needs a built commit; unbuilt entries of other pull requests ahead or
        # behind do not hide its absence.
        for queue in ([entry(1, 7, E, HEAD)],
                      [entry(1, 7, E, HEAD), entry(2, 8, "f" * 40, OTHER)],
                      [entry(1, 7, None, HEAD), entry(2, 9, None, "9" * 40)],
                      [entry(1, 7, None, HEAD), entry(2, 8, "f" * 40, OTHER),
                       entry(3, 9, None, "9" * 40)]):
            with self.subTest(queue=queue):
                code, out, calls = self.run_group(queue)
                self.assertEqual(code, 1, out)
                self.assertIn("no longer in the merge queue", out)
                self.assertEqual(calls.count("sleep"), 1)
                self.assertEqual(len([url for url in calls if url.endswith("/graphql")]), 2)
                self.assertFalse([url for url in calls if url.endswith("/labels")])

    def test_one_absent_read_does_not_eject(self):
        # A listing can lag: an absent read followed by one listing the group waits on.
        group = entry(2, 8, GROUP, OTHER)

        def reappear(labels, comments, nodes):
            if group not in nodes:
                nodes.insert(1, group)

        code, out, calls = self.run_group([entry(1, 7, E, HEAD), entry(3, 9, "f" * 40, "9" * 40)],
                                          on_sleep=reappear)
        self.assertEqual(code, 0, out)
        self.assertEqual(calls.count("sleep"), 1)

    def test_absence_must_be_seen_on_consecutive_reads(self):
        # Any poll that does not show absence resets the count: alternating
        # reads, or an absent read, a failed read and another absent read.
        group = entry(2, 8, GROUP, OTHER)
        polls = (merge_hold.WAIT_SECONDS - merge_hold.DECISION_RESERVE_SECONDS) // merge_hold.POLL_SECONDS

        def toggle(labels, comments, nodes):
            if group in nodes:
                nodes.remove(group)
            else:
                nodes.insert(1, group)

        code, out, calls = self.run_group([entry(1, 7, E, HEAD)], on_sleep=toggle,
                                          ahead=[runs("in_progress")])
        self.assertEqual(code, 1, out)
        self.assertIn("did not complete in time", out)
        self.assertNotIn("no longer in the merge queue", out)
        self.assertEqual(calls.count("sleep"), polls)
        code, out, calls = self.run_group([entry(1, 7, E, HEAD)],
                                          checks=[DONE, OSError("HTTP 502"), DONE])
        self.assertEqual(code, 1, out)
        self.assertIn("no longer in the merge queue", out)
        self.assertEqual(calls.count("sleep"), 3)

    def test_doubtful_queue_reads_are_not_taken_as_leaving(self):
        # Only a well-formed read that positively lacks the group ends the wait early.
        page = {"pageInfo": {"hasNextPage": True}, "nodes": [entry(1, 7, E, HEAD)]}
        polls = (merge_hold.WAIT_SECONDS - merge_hold.DECISION_RESERVE_SECONDS) // merge_hold.POLL_SECONDS
        for nodes, answer in (
            (None, {"errors": [{"message": "denied"}]}),
            (None, {"data": {"repository": {"mergeQueue": None}}}),
            (None, {"data": {"repository": {"mergeQueue": {"entries": page}}}}),
            (self.QUEUE + [entry(4, 8, GROUP, OTHER)], None),
            ([], None),
            ([entry(1, 7, E, HEAD), entry(2, "8", "f" * 40, OTHER)], None),
            ([entry(1, 7, E, HEAD), entry(2, 8, None, OTHER)], None),
            ([entry(1, 7, E, HEAD), {"position": 2, "headCommit": None, "pullRequest": None}],
             None),
        ):
            with self.subTest(nodes=nodes, answer=answer):
                code, out, calls = self.run_group(nodes, answer=answer)
                self.assertEqual(code, 1, out)
                self.assertNotIn("no longer in the merge queue", out)
                self.assertEqual(calls.count("sleep"), polls)

    def test_own_entry_without_pull_request_fails_closed(self):
        code, out, _ = self.run_group([{"position": 1, "headCommit": {"oid": GROUP},
                                        "pullRequest": None}])
        self.assertEqual(code, 1, out)
        self.assertIn("::error::", out)

    def test_absent_check_runs_and_api_errors_keep_waiting(self):
        code, out, calls = self.run_group(
            self.QUEUE,
            checks=[{"check_runs": []}, runs("completed"), OSError("HTTP 502"),
                    {"message": "odd"}, DONE])
        self.assertEqual(code, 0, out)
        self.assertEqual(calls.count("sleep"), 4)

    def test_wait_deadline_fails_closed_without_reading_labels(self):
        with mock.patch.object(merge_hold, "WAIT_SECONDS", 0):
            code, out, calls = self.run_group(self.QUEUE, checks=[runs("in_progress", "completed")])
        self.assertEqual(code, 1)
        self.assertIn("::error::", out)
        self.assertFalse([url for url in calls if url.endswith("/labels")])

    def test_malformed_or_unknown_head_ref_fails_closed(self):
        for ref in ("", "refs/heads/main", "refs/heads/gh-readonly-queue/main/pr-8",
                    f"refs/heads/gh-readonly-queue/other/pr-8-{BASE}",
                    f"refs/heads/gh-readonly-queue/main/pr-8-{BASE}x",
                    f"refs/heads/gh-readonly-queue/main/pr-9-{BASE}"):
            with self.subTest(ref=ref):
                code, out, _ = self.run_group(self.QUEUE, env=dict(GROUP_ENV, MERGE_GROUP_HEAD_REF=ref))
                self.assertEqual(code, 1, out)

    def test_unidentifiable_group_fails_closed(self):
        page = {"pageInfo": {"hasNextPage": True}, "nodes": self.QUEUE}
        for nodes, answer, env in (
            ([entry(1, 8, "e" * 40, OTHER)], None, None),
            (self.QUEUE + [entry(4, 8, GROUP, OTHER)], None, None),
            ([entry(1, 7, "e" * 40, HEAD, state="CLOSED"), entry(2, 8, GROUP, OTHER)], None, None),
            ([entry(1, 8, GROUP, OTHER, base="release")], None, None),
            ([entry("1", 8, GROUP, OTHER)], None, None),
            ([entry(1, 7, None, HEAD), entry(2, 8, GROUP, OTHER)], None, None),
            (None, {"errors": [{"message": "denied"}]}, None),
            (None, {"data": {"repository": {"mergeQueue": None}}}, None),
            (None, {"data": {"repository": {"mergeQueue": {"entries": page}}}}, None),
            (self.QUEUE, None, dict(GROUP_ENV, MERGE_GROUP_BASE_REF="refs/heads/dev")),
            (self.QUEUE, None, dict(GROUP_ENV, MERGE_GROUP_HEAD_SHA="")),
            (self.QUEUE, None, {k: v for k, v in GROUP_ENV.items() if k != "MERGE_GROUP_HEAD_REF"}),
        ):
            with self.subTest(nodes=nodes, answer=answer, env=env):
                code, out, _ = self.run_group(nodes, answer=answer, env=env)
                self.assertEqual(code, 1, out)
                self.assertIn("::error::", out)

    # D1 option B step 2 (#494): every applicable gate is judged on the queue commit itself.
    ONE = [entry(1, 8, GROUP, OTHER)]

    def assertFailsAtOnce(self, out, calls):
        self.assertIn("::error::merge-hold could not decide", out)
        self.assertFalse([url for url in calls if "/check-runs?" in url or url.endswith("/graphql")])
        self.assertNotIn("sleep", calls)

    def test_queue_commit_gates_decide_after_the_wait_with_the_merge_group_plan(self):
        code, out, calls = self.run_group(self.QUEUE)
        self.assertEqual(code, 0, out)
        self.assertEqual(self.plans, [((BASE, GROUP, GROUP, "main"), {"event": "merge_group"})])
        self.assertIn("Passed: quality (viewer): success in run 2 attempt 1", out)
        self.assertIn(f"queue commit {GROUP} ({QUEUE_BRANCH})", out)
        listings = [i for i, url in enumerate(calls) if f"/actions/runs?head_sha={GROUP}&event=merge_group&" in url]
        self.assertEqual(len(listings), 2)  # one wait poll, then the full report
        last_poll = max(i for i, url in enumerate(calls) if "/check-runs?" in url)
        jobs = [i for i, url in enumerate(calls) if "/jobs?" in url]
        self.assertLess(listings[0], last_poll)
        self.assertGreater(min(jobs), last_poll)
        self.assertGreater(min(i for i, url in enumerate(calls) if url.endswith("/labels")), max(jobs))

    def test_failed_or_missing_gate_on_the_queue_commit_fails_the_group(self):
        for conclusion in ("failure", "cancelled", "timed_out", "skipped", "action_required", "neutral", None):
            with self.subTest(conclusion=conclusion):
                code, out, _ = self.run_group(
                    self.ONE, group_jobs={**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", conclusion)]})
                self.assertEqual(code, 1, out)
                self.assertIn(f"::error::Failed: quality (viewer): {conclusion} in run 2", out)
        code, out, _ = self.run_group(self.ONE, group_jobs={**GOOD_JOBS, 2: []})
        self.assertEqual(code, 1, out)
        self.assertIn("::error::Failed: quality (viewer): missing from run 2", out)
        # A report with only Pending lines after the wait fails too: the queue removes the group.
        code, out, _ = self.run_group(self.ONE, group_jobs={
            **GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", None, "in_progress")]})
        self.assertEqual(code, 1, out)
        self.assertIn("1 pending; 0 failed.", out)
        self.assertIn("::error::Pending: quality (viewer): still running in run 2", out)
        # A failed core job that the plan did not expect is never ignored.
        code, out, _ = self.run_group(self.ONE, group_jobs={
            **GOOD_JOBS, 1: [job(11, 1, "host-quality"), job(13, 1, "renamed", "failure")]})
        self.assertEqual(code, 1, out)
        self.assertIn("renamed: failure in run 1 attempt 1", out)
        self.assertIn("not expected by the plan", out)
        # A path filter that hides a registered input fails as on pull_request.
        plan = group_plan()
        plan["selected"][-1]["filter_hidden_inputs"] = ["tests/test_volume_archive.py"]
        code, out, _ = self.run_group(self.ONE, plan=plan)
        self.assertEqual(code, 1, out)
        self.assertIn("its path filter hides registered input(s)", out)

    def test_open_or_missing_gate_runs_fail_closed_at_the_deadline(self):
        older = group_run(2, LEAVES, created_at="2026-11-24T19:00:00Z")
        for why, listed in (("no run", [group_run(1, CI)]),
                            ("running", [group_run(1, CI), group_run(2, LEAVES, status="in_progress", conclusion=None)]),
                            ("newer run still running", [group_run(1, CI), older,
                                                         group_run(3, LEAVES, status="queued", conclusion=None)])):
            with self.subTest(why=why), mock.patch.object(merge_hold, "WAIT_SECONDS", 0):
                code, out, calls = self.run_group(self.ONE, group_runs={GROUP: listed})
                self.assertEqual(code, 1, out)
                self.assertIn(f"did not complete in time: ['applicable gate runs on {GROUP}", out)
                self.assertFalse([url for url in calls if url.endswith("/labels") or "/jobs?" in url])

    def test_gate_wait_reads_holds_and_withdrawals_after_the_gates_finish(self):
        def queue_gates(labels, comments, nodes, hold=None, withdraw=None):
            listing[GROUP] = [group_run(1, CI), group_run(2, LEAVES)]
            if hold:
                labels[hold] = [merge_hold.LABEL]
            if withdraw:
                comments[withdraw] = []

        for hold, withdraw, expected in ((None, None, 0), (7, None, 1), (None, 8, 1)):
            with self.subTest(hold=hold, withdraw=withdraw):
                listing = {GROUP: [group_run(1, CI), group_run(2, LEAVES, status="in_progress", conclusion=None)]}
                code, out, calls = self.run_group(
                    self.QUEUE, group_runs=listing, labels={8: [merge_hold.LABEL]},
                    comments={8: [comment("acceptance-released: " + OTHER)]}, trees={GROUP: "1" * 40, OTHER: "1" * 40},
                    on_sleep=lambda labels, comments, nodes: queue_gates(labels, comments, nodes, hold, withdraw))
                self.assertEqual(code, expected, out)
                self.assertEqual(calls.count("sleep"), 1)
                self.assertLabelsReadAfterWait(calls)
                if expected:
                    self.assertIn(f"#{hold or withdraw}: ::error::", out)

    def test_the_newest_queue_run_and_attempt_decide(self):
        older = group_run(2, LEAVES, created_at="2026-11-24T19:00:00Z")
        code, out, _ = self.run_group(
            self.ONE, group_runs={GROUP: [group_run(1, CI), older, group_run(3, LEAVES, conclusion="failure")]},
            group_jobs={**GOOD_JOBS, 3: [job(31, 3, "quality (viewer)", "failure")]})
        self.assertEqual(code, 1, out)
        self.assertIn("Failed: quality (viewer): failure in run 3 attempt 1", out)
        rerun = group_run(2, LEAVES, conclusion="failure", attempt=2, run_started_at="2026-11-24T22:00:00Z")
        newer = group_run(3, LEAVES, run_started_at="2026-11-24T21:00:00Z")
        code, out, _ = self.run_group(
            self.ONE, group_runs={GROUP: [group_run(1, CI), rerun, newer]},
            group_jobs={**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", "failure", attempt=2)],
                        3: [job(31, 3, "quality (viewer)")]})
        self.assertEqual(code, 1, out)
        self.assertIn("Failed: quality (viewer): failure in run 2 attempt 2", out)

    def test_queue_runs_without_matching_provenance_are_ignored(self):
        # Each case differs from a counted run in one field, in the listing GitHub returns for
        # head_sha=GROUP&event=merge_group, so only the run binding can reject it.
        for why, over in (("rebuilt entry's old commit", {"head_sha": "0" * 40}),
                          ("another entry's queue commit", {"head_sha": E}),
                          ("other queue branch", {"head_branch": f"gh-readonly-queue/main/pr-9-{BASE}"}),
                          ("no head branch", {"head_branch": None}),
                          ("other repository", {"repository": {"full_name": "x/r"}}),
                          ("other head repository", {"head_repository": {"full_name": "fork/r"}}),
                          ("no head repository", {"head_repository": None}),
                          ("pull_request event", {"event": "pull_request", "pull_requests": OWN_PULL}),
                          ("push event", {"event": "push"})):
            with self.subTest(why=why), mock.patch.object(merge_hold, "WAIT_SECONDS", 0):
                code, out, calls = self.run_group(
                    self.ONE, group_runs={GROUP: [group_run(1, CI), group_run(2, LEAVES, **over)]})
                self.assertEqual(code, 1, out)
                self.assertIn(f"applicable gate runs on {GROUP} are still open or have not started", out)
                self.assertFalse([url for url in calls if url.endswith("/labels")])

    def test_concurrent_entries_count_only_this_group_commit(self):
        e_branch = f"gh-readonly-queue/main/pr-7-{BASE}"
        own_e = [group_run(5, CI, head_sha=E, head_branch=e_branch), group_run(6, LEAVES, head_sha=E, head_branch=e_branch)]
        jobs = {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", "failure")],
                5: [job(51, 5, "host-quality")], 6: [job(61, 6, "quality (viewer)")]}
        queue = [entry(1, 7, E, HEAD), entry(2, 8, GROUP, OTHER)]
        code, out, calls = self.run_group(queue, group_runs={E: own_e, GROUP: [group_run(1, CI), group_run(
            2, LEAVES, conclusion="failure")]}, group_jobs=jobs)
        self.assertEqual(code, 1, out)
        self.assertIn("Failed: quality (viewer): failure in run 2", out)
        self.assertFalse([url for url in calls if f"/actions/runs?head_sha={E}" in url])
        # The entry ahead's gate runs, even if listed for this commit, never stand in for its own.
        with mock.patch.object(merge_hold, "WAIT_SECONDS", 0):
            code, out, _ = self.run_group(queue, group_runs={GROUP: [group_run(1, CI)] + own_e[1:]}, group_jobs=jobs)
        self.assertEqual(code, 1, out)
        self.assertIn(f"applicable gate runs on {GROUP}", out)

    def test_pull_request_success_on_the_queue_commit_does_not_count(self):
        stale = wf_run(2, LEAVES, head_sha=GROUP, head_branch=QUEUE_BRANCH)
        with mock.patch.object(merge_hold, "WAIT_SECONDS", 0):
            code, out, calls = self.run_group(self.ONE, gate_runs={GROUP: [stale]},
                                              group_runs={GROUP: [group_run(1, CI)]})
        self.assertEqual(code, 1, out)
        self.assertFalse([url for url in calls if f"head_sha={GROUP}&event=pull_request" in url])
        self.assertTrue([url for url in calls if f"head_sha={GROUP}&event=merge_group" in url])

    def test_failed_predecessor_keeps_the_walk_and_this_group_gates_are_judged(self):
        a1, a2 = "1" * 40, "2" * 40
        failed_hold = {"check_runs": [{"name": "merge-hold", "status": "completed", "conclusion": "failure"}]
                       + DONE["check_runs"][:2]}
        queue = [entry(1, 7, a1, HEAD), entry(2, 9, a2, "9" * 40), entry(3, 8, GROUP, OTHER)]
        commits = {a2: [failed_hold], a1: [DONE]}
        code, out, calls = self.run_group(queue, commits=commits)
        self.assertEqual(code, 0, out)
        self.assertTrue([url for url in calls if f"/commits/{a1}/check-runs?" in url])
        code, out, _ = self.run_group(queue, commits=commits,
                                      group_jobs={**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", "failure")]})
        self.assertEqual(code, 1, out)
        self.assertIn("::error::Failed: quality (viewer): failure in run 2", out)

    def test_member_verdict_is_bound_to_its_current_head(self):
        # The only successful pull_request merge-hold is on an older head of PR 8.
        older = dict(gate_run(OTHER), head_sha="0" * 40)
        code, out, _ = self.run_group(self.ONE, gate_runs={OTHER: [older]})
        self.assertEqual(code, 1, out)
        self.assertIn("#8: ::error::No successful pull_request merge-hold", out)
        # A release for another head does not carry over, whatever the queue-commit gates say.
        code, out, _ = self.run_group(self.ONE, labels={8: [merge_hold.LABEL]},
                                      comments={8: [comment("acceptance-released: " + HEAD)]},
                                      trees={GROUP: "1" * 40, OTHER: "1" * 40})
        self.assertEqual(code, 1, out)
        self.assertIn("#8: ::error::The 'acceptance-pending' label is set", out)

    def test_shadow_policy_reports_queue_gates_without_blocking_or_waiting(self):
        plan = group_plan(policy_mode="shadow")
        jobs = {**GOOD_JOBS, 1: [job(11, 1, "host-quality", "failure")]}
        for why, listed in (("failed and running", [group_run(1, CI, conclusion="failure"),
                                                     group_run(2, LEAVES, status="in_progress", conclusion=None)]),
                            ("never started", [group_run(1, CI, conclusion="failure")])):
            with self.subTest(why=why):
                code, out, calls = self.run_group(self.ONE, plan=plan, group_runs={GROUP: listed}, group_jobs=jobs)
                self.assertEqual(code, 0, out)
                self.assertNotIn("sleep", calls)
                # Not judged early either: the only run listing is the final report's.
                self.assertEqual(len([url for url in calls if f"head_sha={GROUP}&event=merge_group" in url]), 1)
                self.assertIn("::warning::(shadow, not blocking) Failed: host-quality: failure in run 1", out)
                self.assertIn("::warning::(shadow, not blocking) Pending: ", out)

    def test_queue_inputs_and_plan_are_bound_before_any_wait(self):
        bad = "g" * 40

        def plan_on(base):
            return group_plan(shas={"base": base, "merge_base": base, "head": GROUP, "tested": GROUP},
                              catalogue={"source_sha": base, "blob": "f" * 40})

        # Each plan matches what the code passes the selector, so only the guard under test can fail it.
        for why, env, plan in (
                ("base is not the group's", dict(GROUP_ENV, BASE_SHA=OTHER), plan_on(OTHER)),
                ("checkout is not the queue commit", dict(GROUP_ENV, TESTED_SHA=OTHER), None),
                ("malformed base", dict(GROUP_ENV, BASE_SHA=bad, MERGE_GROUP_BASE_SHA=bad), plan_on(bad)),
                ("empty tested", dict(GROUP_ENV, TESTED_SHA=""), None),
                ("no event base", {k: v for k, v in GROUP_ENV.items() if k != "MERGE_GROUP_BASE_SHA"}, None),
                ("no BASE_SHA", {k: v for k, v in GROUP_ENV.items() if k != "BASE_SHA"}, None),
                ("pull_request plan", GROUP_ENV, group_plan(event="pull_request")),
                ("plan of a pull request head", GROUP_ENV, plan_fixture()),
                ("plan of another queue commit", GROUP_ENV,
                 group_plan(shas={"base": BASE, "merge_base": BASE, "head": OTHER, "tested": OTHER})),
                ("plan for another branch", GROUP_ENV, group_plan(ref="release")),
                ("untrusted selector", GROUP_ENV, group_plan(selector={"trusted": False})),
                ("unusable fallback", GROUP_ENV, group_plan(fallback={"code": "fetch-failed"}))):
            with self.subTest(why=why):
                code, out, calls = self.run_group(self.QUEUE, env=env, plan=plan)
                self.assertEqual(code, 1, out)
                self.assertFailsAtOnce(out, calls)
        code, out, calls = self.run_group(self.QUEUE, ancestor=1)
        self.assertEqual(code, 1, out)
        self.assertIn(f"does not contain the base {BASE}", out)
        self.assertFailsAtOnce(out, calls)

    # ADR-CI-MERGE-GROUP-SELECT-20261126: the trusted base T, not merge_group.base_sha.
    def test_a_stacked_group_plans_from_the_trusted_base_not_the_entry_ahead(self):
        code, out, calls = self.run_group(self.QUEUE, env=dict(GROUP_ENV, MERGE_GROUP_BASE_SHA=AHEAD))
        self.assertEqual(code, 0, out)
        self.assertEqual(self.plans, [((BASE, GROUP, GROUP, "main"), {"event": "merge_group"})])
        self.assertIn(("cat-file", "blob", f"{BASE}:{merge_hold.CATALOGUE}"), self.gits)  # the budget too
        self.assertIn(f"trusted base {BASE} (merge base with main), merge group base {AHEAD}, "
                      f"queue commit {GROUP} ({QUEUE_BRANCH})", out)
        # A full plan whose diff was indeterminate (no merge base recorded) still decides, with every gate.
        shallow = group_plan(fallback={"code": "shallow-history", "detail": [], "reasons": ["shallow-history"]},
                             shas={"base": BASE, "merge_base": None, "head": GROUP, "tested": GROUP})
        code, out, _ = self.run_group(self.QUEUE, plan=shallow)
        self.assertEqual(code, 0, out)
        self.assertIn("mode full, fallback shallow-history", out)

    def test_the_trusted_base_is_checked_before_it_is_used(self):
        stacked = dict(GROUP_ENV, MERGE_GROUP_BASE_SHA=AHEAD)
        from_ahead = group_plan(mode="select", fallback=None,
                                shas={"base": BASE, "merge_base": AHEAD, "head": GROUP, "tested": GROUP})
        for why, env, kwargs, message in (
                ("BASE_SHA is not the merge base with main", GROUP_ENV, {"trusted": OTHER},
                 f"BASE_SHA {BASE} is not the merge base ({OTHER})"),
                ("no merge base with main", GROUP_ENV, {"trusted": None}, "is not the merge base (none)"),
                ("T is not an ancestor of the merge group base", stacked, {"ancestor": {(BASE, AHEAD)}},
                 f"the trusted base {BASE} is not an ancestor of the merge group base {AHEAD}"),
                ("the queue commit lacks the merge group base", stacked, {"ancestor": {(AHEAD, GROUP)}},
                 f"does not contain the base {AHEAD}"),
                ("malformed merge group base", dict(GROUP_ENV, MERGE_GROUP_BASE_SHA="g" * 40), {},
                 "merge group base SHA is missing or malformed")):
            with self.subTest(why=why):
                code, out, calls = self.run_group(self.QUEUE, env=env, **kwargs)
                self.assertEqual(code, 1, out)
                self.assertIn(message, out)
                self.assertFailsAtOnce(out, calls)
                self.assertEqual(self.plans, [])  # no tool of an unchecked base ran
        code, out, calls = self.run_group(self.QUEUE, env=stacked, plan=from_ahead)
        self.assertEqual(code, 1, out)
        self.assertIn(f"the plan diffs from {AHEAD}, not from the trusted base {BASE}", out)
        self.assertFailsAtOnce(out, calls)
        # A select plan must record its merge base (D2 M3b); a full plan without one stays allowed.
        unbound = group_plan(mode="select", fallback=None,
                             shas={"base": BASE, "merge_base": None, "head": GROUP, "tested": GROUP})
        code, out, calls = self.run_group(self.QUEUE, plan=unbound)
        self.assertEqual(code, 1, out)
        self.assertIn(f"the plan diffs from None, not from the trusted base {BASE}", out)
        self.assertFailsAtOnce(out, calls)

    NATIVE = ".github/workflows/native-foundation.yml"

    ADVICE = ".github/workflows/advice.yml"

    def docs_only_group(self, mode="select", native_selected=False, advisory_run=False, native_mixed=False,
                        leaves_reason="no-input-match", filter_blocked=None, native_done=False):
        """host-quality and qemu-smoke pass; the leaves fail and a native suite still runs on GROUP.

        advisory_run adds an advisory-only workflow whose run has a failed unregistered job;
        native_mixed adds an excluded rwa variant to the selected native suite; filter_blocked is the
        plan's pull_request_filter_blocked list (default: every workflow; False drops the key)."""
        selected = [row("ci#host-quality", CI, "host-quality", "core"), row("ci#qemu-smoke", CI, "qemu-smoke", "core"),
                    row("merge-hold#merge-hold", merge_hold.GATE_WORKFLOW, "merge-hold", "core")]
        native = row("native-foundation#native", self.NATIVE, "native")
        excluded = [{"id": "leaves#quality/viewer", "class": "rwa", "workflow": LEAVES, "job": "quality",
                     "check_name": "quality (viewer)", "reason": "no-input-match"}]
        excluded[0]["reason"] = leaves_reason
        if native_selected:
            selected.append(native)
            if native_mixed:
                excluded.append({"id": "native-foundation#native/extra", "class": "rwa", "workflow": self.NATIVE,
                                 "job": "native", "check_name": "native (extra)", "reason": "no-input-match"})
        else:
            excluded.append({key: native[key] for key in ("id", "class", "workflow", "job", "check_name")}
                            | {"reason": "no-input-match"})
        runs = [group_run(1, CI), group_run(2, LEAVES, conclusion="failure"),
                group_run(3, self.NATIVE, conclusion="failure") if native_done
                else group_run(3, self.NATIVE, status="in_progress", conclusion=None)]
        if advisory_run:
            excluded.append({"id": "advice#advice", "class": "advisory", "workflow": self.ADVICE, "job": "advice",
                             "check_name": "advice", "reason": "advisory"})
            runs.append(group_run(4, self.ADVICE, conclusion="failure"))
        plan = group_plan(mode=mode, fallback=None if mode == "select" else group_plan()["fallback"],
                          selected=selected, excluded=excluded)
        if filter_blocked is not False:
            plan["pull_request_filter_blocked"] = (
                [LEAVES, self.NATIVE, self.ADVICE] if filter_blocked is None else filter_blocked)
        failed_check = {"name": "quality (viewer)", "status": "completed", "conclusion": "failure", "id": 99}
        native_checks = [{"name": "native", "status": "completed", "conclusion": "success", "id": 98},
                         {"name": "native (extra)", "status": "completed", "conclusion": "failure", "id": 97}]
        return self.run_group(
            self.QUEUE, plan=plan,
            checks=[{"check_runs": DONE["check_runs"] + [failed_check] + (native_checks if native_done else [])}],
            group_runs={GROUP: runs},
            group_jobs={1: [job(11, 1, "host-quality"), job(12, 1, "qemu-smoke")],
                        2: [job(21, 2, "quality (viewer)", "failure")],
                        3: [job(31, 3, "native"), job(32, 3, "native (extra)", "failure")] if native_done
                        else [job(31, 3, "native", None, "in_progress")],
                        4: [job(41, 4, "advice", "failure"), job(42, 4, "stray", "failure")]})

    def test_a_docs_only_group_passes_on_core_without_awaiting_or_judging_unselected_suites(self):
        code, out, calls = self.docs_only_group()
        self.assertEqual(code, 0, out)
        self.assertIn("2 applicable gate(s) passed; 0 filtered out; 0 pending; 0 failed.", out)
        self.assertIn("Passed: qemu-smoke: success in run 1 attempt 1", out)
        self.assertIn(f"Not applicable, neither awaited nor judged (no selected gate): {LEAVES}, {self.NATIVE}.", out)
        self.assertNotIn("sleep", calls)
        self.assertFalse([url for url in calls if "/runs/2" in url or "/runs/3" in url])
        # Under a full plan the same runs are awaited and judged: the failed leaves gate fails the group.
        code, out, _ = self.docs_only_group(mode="full")
        self.assertEqual(code, 1, out)
        self.assertIn("::error::Failed: quality (viewer): failure in run 2 attempt 1", out)
        self.assertNotIn("Not applicable, neither awaited", out)
        # A workflow without any core or rwa gate is still judged: its failed unregistered job fails the group.
        code, out, _ = self.docs_only_group(advisory_run=True)
        self.assertEqual(code, 1, out)
        self.assertIn("::error::Failed: stray: failure in run 4 attempt 1", out)
        self.assertIn(f"neither awaited nor judged (no selected gate): {LEAVES}, {self.NATIVE}.", out)
        # A selected suite that is still running is awaited and fails closed at the deadline.
        with mock.patch.object(merge_hold, "WAIT_SECONDS", 0):
            code, out, _ = self.docs_only_group(native_selected=True)
        self.assertEqual(code, 1, out)
        self.assertIn("required checks did not complete in time", out)

    def test_only_a_workflow_established_as_not_applicable_is_left_out(self):
        """Review D2 B1, M5, M6: doubt about applicability keeps a workflow awaited and judged."""
        failed_leaves = "::error::Failed: quality (viewer): failure in run 2 attempt 1"
        # Its own pull_request path filter would have started the leaves (e.g. a LICENSE or
        # .coveragerc change): judged as on pull_request, so their failure fails the group.
        code, out, _ = self.docs_only_group(filter_blocked=[self.NATIVE])
        self.assertEqual(code, 1, out)
        self.assertIn(failed_leaves, out)
        self.assertIn(f"neither awaited nor judged (no selected gate): {self.NATIVE}.", out)
        # A plan without the list (an older selector, any doubt) leaves nothing out.
        code, out, _ = self.docs_only_group(filter_blocked=False)
        self.assertEqual(code, 1, out)
        self.assertIn(failed_leaves, out)
        self.assertNotIn("Not applicable, neither awaited", out)
        # Excluded for another reason than no-input-match: not established, still judged (M6).
        code, out, _ = self.docs_only_group(leaves_reason="condition-false")
        self.assertEqual(code, 1, out)
        self.assertIn(failed_leaves, out)
        # A workflow with a selected gate is judged whole, even beside an excluded variant (M5).
        with mock.patch.object(merge_hold, "WAIT_SECONDS", 0):
            code, out, _ = self.docs_only_group(native_selected=True, native_mixed=True)
        self.assertEqual(code, 1, out)
        self.assertIn("required checks did not complete in time", out)
        left_out = [line for line in out.splitlines() if "neither awaited nor judged" in line]
        self.assertFalse([line for line in left_out if self.NATIVE in line], out)
        # Its selected gate passed but its excluded variant failed: judged whole, the group fails.
        code, out, _ = self.docs_only_group(native_selected=True, native_mixed=True, native_done=True,
                                            filter_blocked=[self.NATIVE])
        self.assertEqual(code, 1, out)
        self.assertIn("native (extra): failure in run 3 attempt 1", out)

    # Round 2 (#494): shadow keeps the base's equal-tree rule; the gate wait ends early.
    def test_unheld_member_lands_another_tree_only_after_an_enforced_clean_report(self):
        trees = {GROUP: "1" * 40, OTHER: "2" * 40}
        rule = (f"#8: ::error::The applicable gates ran on {OTHER}, but its queue entry {GROUP} lands a "
                "different tree, and the queue-commit gate report is not an enforced clean pass, so it does not "
                "cover that tree. Update the branch.")
        failed = {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", "failure")]}
        pending = {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", None, "in_progress")]}
        code, out, _ = self.run_group(self.ONE, trees=trees)
        self.assertEqual(code, 0, out)  # enforced and clean: the queue-commit report covers the landing tree
        self.assertIn("#8: No acceptance hold is set.", out)
        shadow = group_plan(policy_mode="shadow")
        for why, jobs in (("clean", GOOD_JOBS), ("failed", failed)):
            with self.subTest(policy="shadow", report=why):
                code, out, _ = self.run_group(self.ONE, trees=trees, plan=shadow, group_jobs=jobs)
                self.assertEqual(code, 1, out)
                self.assertIn(rule, out)
                code, out, _ = self.run_group(self.ONE, plan=shadow, group_jobs=jobs)
                self.assertEqual(code, 0, out)  # an equal tree still passes under shadow
        # Enforced with a Failed or only a Pending line: the group fails anyway and the member keeps the rule.
        for why, jobs in (("failed", failed), ("pending", pending)):
            with self.subTest(policy="enforce", report=why):
                code, out, _ = self.run_group(self.ONE, trees=trees, group_jobs=jobs)
                self.assertEqual(code, 1, out)
                self.assertIn(rule, out)
                code, out, _ = self.run_group(self.ONE, group_jobs=jobs)
                self.assertEqual(code, 1, out)
                self.assertIn("#8: No acceptance hold is set.", out)

    def test_unheld_member_ahead_may_land_its_own_entry_tree_after_an_enforced_clean_report(self):
        """PR 7 is ahead and unheld; its own entry E lands a tree other than its head's. This group's
        report judges GROUP's tree, not E's, and PR 7 still passes here, because E's own merge-hold
        succeeded (it judged the gates on E; the next test pins that check) and the ruleset's "Only
        merge non-failing pull requests" setting keeps E from merging without it. Under shadow, or
        with a Pending or Failed line, PR 7 keeps the equal-tree rule as before step 2, and it still
        needs its own pull_request merge-hold."""
        queue = [entry(1, 7, E, HEAD), entry(2, 8, GROUP, OTHER)]
        trees = {E: "1" * 40, HEAD: "2" * 40, GROUP: "3" * 40, OTHER: "3" * 40}
        rule = (f"#7: ::error::The applicable gates ran on {HEAD}, but its queue entry {E} lands a "
                "different tree, and the queue-commit gate report is not an enforced clean pass, so it does not "
                "cover that tree. Update the branch.")
        code, out, calls = self.run_group(queue, trees=trees)
        self.assertEqual(code, 0, out)
        self.assertIn("#7: No acceptance hold is set.", out)
        self.assertIn("#8: No acceptance hold is set.", out)
        self.assertTrue([url for url in calls if f"/commits/{E}/check-runs?" in url])  # E's own merge-hold
        failed = {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", "failure")]}
        pending = {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", None, "in_progress")]}
        for why, plan, jobs in (("shadow", group_plan(policy_mode="shadow"), GOOD_JOBS),
                                ("failed", None, failed), ("pending", None, pending)):
            with self.subTest(report=why):
                code, out, _ = self.run_group(queue, trees=trees, plan=plan, group_jobs=jobs)
                self.assertEqual(code, 1, out)
                self.assertIn(rule, out)
                self.assertIn("#8: No acceptance hold is set.", out)  # its own entry lands its head tree
        with self.subTest(member="own pull_request merge-hold failed"):
            code, out, _ = self.run_group(queue, trees=trees, gate_runs={HEAD: [gate_run(HEAD, "failure")]})
            self.assertEqual(code, 1, out)
            self.assertIn(f"#7: ::error::No successful pull_request merge-hold verified the applicable gates on "
                          f"{HEAD}.", out)
            self.assertIn("#8: No acceptance hold is set.", out)

    def test_a_lifted_member_ahead_needs_a_successful_merge_hold_on_its_own_entry(self):
        """D2 on fix round 3: this group's report does not judge tree(E), so a lifted PR 7 ahead also
        needs the newest merge_group merge-hold run on E, bound to E, a queue ref of PR 7 and this
        repository, to have concluded success. E's check runs are done in every case here, so the walk
        ends and only the run listing at the decision differs; the jobs, the re-read and the check runs
        read after it agree with the listed run (the next test varies them)."""
        queue = [entry(1, 7, E, HEAD), entry(2, 8, GROUP, OTHER)]
        trees = {E: "1" * 40, HEAD: "2" * 40, GROUP: "3" * 40, OTHER: "3" * 40}
        good = [group_run(1, CI), group_run(2, LEAVES)]
        line = (f"#7: ::error::Its queue entry {E} lands a tree other than {HEAD}'s, which this group's "
                f"queue-commit gate report does not judge, and no successful merge_group merge-hold on {E} "
                "covers it. Update the branch, or let the pull requests ahead merge first.")
        older, newer = {"created_at": "2026-11-24T19:00:00Z"}, {"created_at": "2026-11-24T21:00:00Z"}
        for why, listed in (
                ("failed", [entry_hold_run(E, 7, "failure")]),
                ("missing", []),
                ("in progress", [entry_hold_run(E, 7, None, status="in_progress")]),
                ("an older success under a newer failure", [entry_hold_run(E, 7, run_id=71, **older),
                                                            entry_hold_run(E, 7, "failure", run_id=72, **newer)]),
                ("another workflow", [entry_hold_run(E, 7, path=CI)]),
                ("pull_request event", [entry_hold_run(E, 7, event="pull_request")]),
                ("another commit", [entry_hold_run(E, 7, head_sha=GROUP)]),
                ("another pull request's queue ref", [entry_hold_run(E, 9)]),
                ("not a queue ref", [entry_hold_run(E, 7, head_branch="topic")]),
                ("another repository", [entry_hold_run(E, 7, repository={"full_name": "x/r"})]),
                ("another head repository", [entry_hold_run(E, 7, head_repository={"full_name": "fork/r"})])):
            with self.subTest(why=why):
                code, out, calls = self.run_group(queue, trees=trees, group_runs={GROUP: good, E: listed})
                self.assertEqual(code, 1, out)
                self.assertIn(line, out)
                self.assertIn("2 applicable gate(s) passed; 1 filtered out; 0 pending; 0 failed.", out)
                self.assertIn("#8: No acceptance hold is set.", out)
                self.assertNotIn("sleep", calls)  # fails closed at the decision, without another wait
        for why, listed in (("succeeded", [entry_hold_run(E, 7)]),
                            ("a newer success over an older failure",
                             [entry_hold_run(E, 7, "failure", run_id=71, **older),
                              entry_hold_run(E, 7, run_id=72, **newer)]),
                            # The listing's binding picks the run; the reads after it never see a newer
                            # run of another workflow or one bound elsewhere.
                            ("a newer run of another workflow",
                             [entry_hold_run(E, 7, run_id=71, **older),
                              entry_hold_run(E, 7, run_id=72, path=CI, **newer)]),
                            ("a newer run from another head repository",
                             [entry_hold_run(E, 7, run_id=71, **older),
                              entry_hold_run(E, 7, run_id=72, head_repository={"full_name": "fork/r"}, **newer)])):
            with self.subTest(why=why):
                code, out, calls = self.run_group(queue, trees=trees, group_runs={GROUP: good, E: listed})
                self.assertEqual(code, 0, out)
                self.assertIn("#7: No acceptance hold is set.", out)
                self.assertEqual(len([url for url in calls if f"head_sha={E}&event=merge_group" in url]), 1)
        # Only a lifted member ahead needs it: one whose entry lands its own head tree does not, nor does
        # this group's own entry, whose merge-hold is this run.
        failed = {GROUP: good, E: [entry_hold_run(E, 7, "failure")]}
        for why, trees_ in (("member ahead with an equal tree", {E: "1" * 40, HEAD: "1" * 40}),
                            ("own entry lifted", {GROUP: "1" * 40, OTHER: "2" * 40})):
            with self.subTest(why=why):
                code, out, calls = self.run_group(queue, trees=trees_, group_runs=failed)
                self.assertEqual(code, 0, out)
                self.assertFalse([url for url in calls if "&event=merge_group" in url and GROUP not in url])

    def test_a_lifted_member_ahead_fails_closed_when_the_reads_of_its_entry_merge_hold_disagree(self):
        """D2 on round 4: the entry run gets gate_report's rigour. The listing always shows a completed,
        successful merge-hold run 70 on E, attempt 2; each case contradicts it in one read only (its jobs,
        its re-read by id, or E's check runs read again at the decision), and PR 7 then fails closed."""
        queue = [entry(1, 7, E, HEAD), entry(2, 8, GROUP, OTHER)]
        trees = {E: "1" * 40, HEAD: "2" * 40, GROUP: "3" * 40, OTHER: "3" * 40}
        group_runs = {GROUP: [group_run(1, CI), group_run(2, LEAVES)], E: [entry_hold_run(E, 7, attempt=2)]}
        line = (f"#7: ::error::Its queue entry {E} lands a tree other than {HEAD}'s, which this group's "
                f"queue-commit gate report does not judge, and no successful merge_group merge-hold on {E} "
                "covers it. Update the branch, or let the pull requests ahead merge first.")

        def checks(conclusion, status="completed"):
            return {"check_runs": [{"name": name, "status": "completed", "conclusion": "success"}
                                   for name in merge_hold.WAIT_CHECKS]
                    + [{"name": "merge-hold", "status": status, "conclusion": conclusion}]}

        def jobs(*listed):
            return {**GOOD_JOBS, 70: list(listed)}

        good = job(700, 70, "merge-hold", attempt=2)
        code, out, calls = self.run_group(queue, trees=trees, group_runs=group_runs)
        self.assertEqual(code, 0, out)  # the green path: the four reads agree
        self.assertIn("#7: No acceptance hold is set.", out)
        self.assertEqual([url.split("?")[0] for url in calls if "/runs/70" in url or f"/commits/{E}/" in url
                          or f"head_sha={E}&event=merge_group" in url][-4:],
                         ["https://api.github.com/repos/o/r/actions/runs", "https://api.github.com/repos/o/r/actions"
                          "/runs/70/jobs", "https://api.github.com/repos/o/r/actions/runs/70",
                          f"https://api.github.com/repos/o/r/commits/{E}/check-runs"])
        for why, extra in (
                # The D2 probe: the listing (stale) shows success while E's merge-hold check run failed. The
                # walk goes past E, whose failed merge-hold proves nothing, and the decision reads it again.
                ("check run on E failed", {"commits": {E: [checks("failure")]}}),
                # The walk saw it succeed; a re-run of it had started when the decision read it again.
                ("check run on E running again", {"commits": {E: [DONE, checks(None, "in_progress")]}}),
                ("the listing shows a failure the re-read and jobs do not",
                 {"group_runs": {**group_runs, E: [entry_hold_run(E, 7, "failure", attempt=2)]},
                  "rereads": {70: {"conclusion": "success"}}, "group_jobs": jobs(good)}),
                ("the run moved on between the reads",
                 {"rereads": {70: {"run_attempt": 3, "status": "in_progress", "conclusion": None}}}),
                ("the re-read under another id", {"rereads": {70: {"id": 99}}}),
                ("the re-read shows a failure", {"rereads": {70: {"conclusion": "failure"}}}),
                ("the re-read is another workflow", {"rereads": {70: {"path": CI}}}),
                ("the re-read is bound to another queue ref", {"rereads": {70: {"head_branch": QUEUE_BRANCH}}}),
                ("a job of another attempt", {"group_jobs": jobs(job(700, 70, "merge-hold", attempt=1))}),
                ("a job with a float attempt", {"group_jobs": jobs(dict(good, run_attempt=2.0))}),
                ("a bool run attempt", {"group_runs": {**group_runs, E: [entry_hold_run(E, 7, attempt=True)]},
                                        "group_jobs": jobs(job(700, 70, "merge-hold", attempt=1))}),
                ("a failed job", {"group_jobs": jobs(good, job(701, 70, "x", "failure", attempt=2))}),
                ("a running job", {"group_jobs": jobs(good, job(701, 70, "x", None, "in_progress", attempt=2))}),
                ("a job both running and successful", {"group_jobs": jobs(dict(good, status="in_progress"))}),
                ("no job", {"group_jobs": jobs()}),
                ("only a job of another run", {"group_jobs": jobs(job(700, 71, "merge-hold", attempt=2))})):
            with self.subTest(why=why):
                code, out, calls = self.run_group(queue, trees=trees, **{"group_runs": group_runs, **extra})
                self.assertEqual(code, 1, out)
                self.assertIn(line, out)
                self.assertIn("2 applicable gate(s) passed; 1 filtered out; 0 pending; 0 failed.", out)
                self.assertIn("#8: No acceptance hold is set.", out)
                self.assertNotIn("sleep", calls)

    def test_a_member_ahead_whose_entry_merge_hold_has_not_finished_waits_and_fails_closed(self):
        """E's merge-hold still running, or not reported at all: the walk waits for it (E is the nearest
        entry ahead), and the deadline fails closed before any label or entry run listing is read."""
        queue = [entry(1, 7, E, HEAD), entry(2, 8, GROUP, OTHER)]
        trees = {E: "1" * 40, HEAD: "2" * 40, GROUP: "3" * 40, OTHER: "3" * 40}
        for why, checks, listed in (("in progress", runs("completed", "completed", "in_progress"),
                                     [entry_hold_run(E, 7, None, status="in_progress")]),
                                    ("missing", runs("completed", "completed"), [])):
            with self.subTest(why=why):
                code, out, calls = self.run_group(queue, trees=trees, commits={E: [checks]},
                                                  group_runs={GROUP: [group_run(1, CI), group_run(2, LEAVES)],
                                                              E: listed})
                self.assertEqual(code, 1, out)
                self.assertIn(f"required checks did not complete in time: ['merge-hold on {E}']", out)
                self.assertEqual(calls.count("sleep"), 77)
                self.assertFalse([url for url in calls if url.endswith("/labels")
                                  or f"head_sha={E}&event=merge_group" in url])

    def test_group_that_leaves_the_queue_during_the_gate_wait_fails_after_two_reads(self):
        # Gate runs stay open, so only the queue read of every gate poll can end this wait early.
        def remove(labels, comments, nodes):
            nodes[:] = [node for node in nodes if node["headCommit"]["oid"] != GROUP]

        def rebuild(labels, comments, nodes):
            for node in nodes:
                if node["headCommit"]["oid"] == GROUP:
                    node["headCommit"] = {"oid": "f" * 40}

        listed = [group_run(1, CI), group_run(2, LEAVES, status="in_progress", conclusion=None)]
        for why, change in (("removed", remove), ("rebuilt", rebuild)):
            with self.subTest(why=why):
                code, out, calls = self.run_group([entry(1, 7, E, HEAD), entry(2, 8, GROUP, OTHER)],
                                                  group_runs={GROUP: listed}, on_sleep=change)
                self.assertEqual(code, 1, out)
                self.assertIn("no longer in the merge queue", out)
                self.assertEqual(calls.count("sleep"), 2)
                self.assertFalse([url for url in calls if url.endswith("/labels") or "/jobs?" in url])

    def test_a_failed_gate_ends_the_gate_wait_with_its_full_report(self):
        def checks(name, conclusion):
            return [{"check_runs": DONE["check_runs"] + [
                {"id": 99, "name": name, "status": "completed", "conclusion": conclusion}]}]

        running = group_run(2, LEAVES, status="in_progress", conclusion=None)
        storage = group_run(3, STORAGE, status="in_progress", conclusion=None)  # keeps the runs open
        for why, answers, listed, jobs, line in (
                ("failed job in a running run", checks("quality (viewer)", "failure"), [group_run(1, CI), running],
                 {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", "failure")]},
                 f"::error::Failed: quality (viewer): failure in run 2 attempt 1 ({LEAVES})"),
                ("skipped expected gate", checks("quality (viewer)", "skipped"), [group_run(1, CI), running],
                 {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", "skipped")]},
                 f"::error::Failed: quality (viewer): skipped in run 2 attempt 1 ({LEAVES})"),
                ("unexpected failed job", checks("renamed", "failure"), [group_run(1, CI), running],
                 {**GOOD_JOBS, 1: [job(11, 1, "host-quality"), job(13, 1, "renamed", "failure")]},
                 "::error::Failed: renamed: failure in run 1 attempt 1"),
                ("cancelled run", [DONE], [group_run(1, CI), group_run(2, LEAVES, conclusion="cancelled"), storage],
                 {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", "cancelled")]},
                 f"::error::Failed: quality (viewer): cancelled in run 2 attempt 1 ({LEAVES})"),
                ("run without jobs", [DONE], [group_run(1, CI), group_run(2, LEAVES, conclusion="startup_failure"),
                                              storage],
                 {**GOOD_JOBS, 2: []}, f"::error::Failed: quality (viewer): missing from run 2 attempt 1 ({LEAVES})")):
            with self.subTest(why=why):
                code, out, calls = self.run_group(self.QUEUE, checks=answers, group_runs={GROUP: listed},
                                                  group_jobs=jobs)
                self.assertEqual(code, 1, out)
                self.assertIn(line, out)
                self.assertNotIn("sleep", calls)
                self.assertNotIn("did not complete in time", out)
                # One wait poll and the report that ended it, which also decides: no second report.
                self.assertEqual(len([url for url in calls if f"head_sha={GROUP}&event=merge_group" in url]), 2)

    def test_a_failure_the_report_does_not_count_is_judged_once_and_the_wait_goes_on(self):
        advisory = [{"check_runs": DONE["check_runs"] + [
            {"id": 97, "name": "advice", "status": "completed", "conclusion": "failure"}]}]
        listing = {GROUP: [group_run(1, CI), group_run(2, LEAVES, status="in_progress", conclusion=None)]}
        sleeps = []

        def finish(labels, comments, nodes):
            sleeps.append(1)
            if len(sleeps) == 2:
                listing[GROUP] = [group_run(1, CI), group_run(2, LEAVES)]

        code, out, calls = self.run_group(
            self.QUEUE, checks=advisory, group_runs=listing, on_sleep=finish,
            group_jobs={**GOOD_JOBS, 1: [job(11, 1, "host-quality"), job(12, 1, "advice", "failure")]})
        self.assertEqual(code, 0, out)
        self.assertEqual(calls.count("sleep"), 2)
        # The early report of the first poll reads two runs' jobs, the final report two more.
        self.assertEqual(len([url for url in calls if "/jobs?" in url]), 4)

    def test_the_last_poll_may_add_a_full_report_before_the_decision(self):
        """D2 on round 4, as merge_group_hold's docstring counts it: with the budget used up, the one poll
        is the last. A new trigger that the report does not count (an advisory check-run failure) makes it
        add a full report (1 + 2N), and the decision then makes its own (2 + 2N + 4M): for N = 2 counted
        runs, M = 2 members and one entry walked, 3 + 1 + (1 + 2N) + (2 + 2N + 4M) = 23 requests."""
        advisory = [{"check_runs": DONE["check_runs"] + [
            {"id": 97, "name": "advice", "status": "completed", "conclusion": "failure"}]}]
        code, out, calls = self.run_group(
            self.QUEUE, checks=advisory, plan_seconds=merge_hold.WAIT_SECONDS,
            group_jobs={**GOOD_JOBS, 1: [job(11, 1, "host-quality"), job(12, 1, "advice", "failure")]})
        self.assertEqual(code, 0, out)
        self.assertNotIn("sleep", calls)
        n, m = 2, 2
        self.assertEqual(len(calls), 3 + 1 + (1 + 2 * n) + (2 + 2 * n + 4 * m), calls)
        self.assertEqual(len([url for url in calls if "/jobs?" in url]), 2 * n)

    def test_an_early_report_that_could_not_be_read_is_made_again(self):
        reads = []

        def leaves_jobs():
            reads.append(1)
            if len(reads) == 1:
                raise OSError("HTTP 502")
            return [job(21, 2, "quality (viewer)", "failure")]

        code, out, calls = self.run_group(
            self.QUEUE, checks=[{"check_runs": DONE["check_runs"] + [
                {"id": 99, "name": "quality (viewer)", "status": "completed", "conclusion": "failure"}]}],
            group_runs={GROUP: [group_run(1, CI), group_run(2, LEAVES, status="in_progress", conclusion=None)]},
            group_jobs={**GOOD_JOBS, 2: leaves_jobs})
        self.assertEqual(code, 1, out)
        self.assertIn(f"::error::Failed: quality (viewer): failure in run 2 attempt 1 ({LEAVES})", out)
        self.assertEqual(calls.count("sleep"), 1)

    def test_a_running_runtime_matrix_does_not_end_the_wait_early(self):
        # Its successful variants so far are Pending in the early report, never Failed.
        dynamic = row("leaves#quality/*", LEAVES, "quality (${{ matrix.leaf }})", variant="*")
        plan = group_plan(selected=group_plan()["selected"][:2] + [dynamic])
        listing = {GROUP: [group_run(1, CI), group_run(2, LEAVES, status="in_progress", conclusion=None)]}
        legs = [job(21, 2, "quality (viewer)"), job(22, 2, "quality (crop)")]

        def finish(labels, comments, nodes):
            listing[GROUP] = [group_run(1, CI), group_run(2, LEAVES)]

        code, out, calls = self.run_group(
            self.QUEUE, plan=plan, group_runs=listing, on_sleep=finish,
            checks=[{"check_runs": DONE["check_runs"] + [
                {"id": 97, "name": "advice", "status": "completed", "conclusion": "failure"}]}],
            group_jobs={**GOOD_JOBS, 1: [job(11, 1, "host-quality"), job(12, 1, "advice", "failure")], 2: legs})
        self.assertEqual(code, 0, out)
        self.assertEqual(calls.count("sleep"), 1)
        self.assertNotIn("Failed:", out)

    def test_an_unreadable_gate_listing_in_the_wait_is_read_again(self):
        unorderable = [group_run(1, CI), group_run(2, LEAVES, created_at=None)]
        listing = {GROUP: unorderable}

        def fix(labels, comments, nodes):
            listing[GROUP] = [group_run(1, CI), group_run(2, LEAVES)]

        code, out, calls = self.run_group(self.QUEUE, group_runs=listing, on_sleep=fix)
        self.assertEqual(code, 0, out)
        self.assertEqual(calls.count("sleep"), 1)
        with mock.patch.object(merge_hold, "WAIT_SECONDS", 0):
            code, out, calls = self.run_group(self.QUEUE, group_runs={GROUP: unorderable})
        self.assertEqual(code, 1, out)
        self.assertIn("queue or check-run read failed: run 2 has no start or creation time", out)
        self.assertFalse([url for url in calls if url.endswith("/labels")])

    # The settle step (#494) runs on the queue commit too; no early report reads these runs here.
    def test_a_queue_gate_run_that_moved_on_once_is_judged_on_its_settled_state(self):
        for why, settled, later, expected, line in (
                ("settled success", {"run_attempt": 2}, job(24, 2, "quality (viewer)", attempt=2), 0,
                 "Passed: quality (viewer): success in run 2 attempt 2"),
                ("settled failure", {"run_attempt": 2, "conclusion": "failure"},
                 job(24, 2, "quality (viewer)", "failure", attempt=2), 1,
                 f"::error::Failed: quality (viewer): failure in run 2 attempt 2 ({LEAVES})")):
            with self.subTest(why=why):
                code, out, calls = self.run_group(self.QUEUE, rereads={2: settled},
                                                  group_jobs={**GOOD_JOBS, 2: per_read(GOOD_JOBS[2], [later])})
                self.assertEqual(code, expected, out)
                self.assertIn(line, out)
                self.assertNotIn("changed while it was read", out)
                self.assertNotIn("sleep", calls)

    def test_a_queue_gate_run_that_keeps_changing_stays_pending_and_fails_the_group(self):
        # Green listing and jobs: only the settle comparison keeps this run from passing.
        code, out, _ = self.run_group(self.QUEUE, rereads={2: per_read({"run_attempt": 2}, {"run_attempt": 3})})
        self.assertEqual(code, 1, out)
        self.assertIn(f"::error::Pending: {LEAVES}: run 2 attempt 1 changed while it was read "
                      "(now attempt 2, completed); re-run merge-hold", out)
        self.assertNotIn("Passed: quality (viewer)", out)
        # An early report (for a failed check run) keeps it pending as well, never Failed, so a
        # gate run that stays open fails closed at the deadline.
        with mock.patch.object(merge_hold, "WAIT_SECONDS", 0):
            code, out, calls = self.run_group(
                self.QUEUE, checks=[{"check_runs": DONE["check_runs"] + [
                    {"id": 99, "name": "quality (viewer)", "status": "completed", "conclusion": "failure"}]}],
                group_runs={GROUP: [group_run(1, CI), group_run(2, LEAVES, status="in_progress", conclusion=None)]},
                group_jobs={**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", "failure")]},
                rereads={2: per_read({"run_attempt": 2, "status": "completed", "conclusion": "failure"},
                                     {"run_attempt": 3, "status": "queued", "conclusion": None})})
        self.assertEqual(code, 1, out)
        self.assertIn(f"did not complete in time: ['applicable gate runs on {GROUP}", out)
        self.assertNotIn("Failed:", out)
        self.assertFalse([url for url in calls if url.endswith("/labels")])

    def test_a_settled_queue_run_that_no_longer_matches_the_binding_is_not_counted(self):
        # The re-read moved on to attempt 2 and held it; only the binding of that record differs.
        # Its jobs carry attempt 2, so the job attempt binding does not hide a binding line that
        # lets the jobs be judged anyway.
        jobs = {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", attempt=2)]}
        for why, over in (("pull_request event", {"event": "pull_request", "pull_requests": OWN_PULL}),
                          ("another queue commit", {"head_sha": E}),
                          ("other queue branch", {"head_branch": f"gh-readonly-queue/main/pr-9-{BASE}"}),
                          ("other repository", {"repository": {"full_name": "x/r"}}),
                          ("other head repository", {"head_repository": {"full_name": "fork/r"}})):
            with self.subTest(why=why):
                code, out, _ = self.run_group(self.QUEUE, rereads={2: {"run_attempt": 2, **over}}, group_jobs=jobs)
                self.assertEqual(code, 1, out)
                self.assertNotIn("Passed: quality (viewer)", out)
                if "head_sha" in over:
                    # CI-SETTLE (#560) already rejects a settled head that differs from the listing.
                    self.assertIn(f"::error::Pending: {LEAVES}: run 2 attempt 1 changed while it was read "
                                  "(now attempt 2, completed); re-run merge-hold", out)
                    continue
                self.assertIn(f"::error::Pending: {LEAVES}: run 2 attempt 2 no longer matches {GROUP} when "
                              "read again; re-run merge-hold", out)
                self.assertNotIn("changed while it was read", out)

    def test_a_queue_gate_run_is_not_judged_on_jobs_of_another_attempt(self):
        """CI-SETTLE's job attempt binding holds on the queue commit too: a green attempt 2 whose
        `filter=latest` jobs still carry attempt 1, listed so or settled from attempt 1, is not
        attempt 2's, so the group must not pass."""
        for why, leaves, rereads in (("unchanged", group_run(2, LEAVES, attempt=2), None),
                                     ("settled", group_run(2, LEAVES), {2: {"run_attempt": 2}})):
            with self.subTest(why=why):
                code, out, _ = self.run_group(self.QUEUE, group_runs={GROUP: [group_run(1, CI), leaves]},
                                              rereads=rereads)
                self.assertEqual(code, 1, out)
                self.assertIn(f"::error::Pending: {LEAVES}: run 2 attempt 2 listed jobs of another attempt; "
                              "re-run merge-hold", out)
                self.assertNotIn("Passed: quality (viewer)", out)

    def test_a_settled_queue_run_is_judged_only_under_its_id_and_no_older_than_the_listing(self):
        """CI-SETTLE's id and attempt checks hold on the queue commit too. The listing shows attempt 2
        failed; re-reads that lag at a green attempt 1, or name another run at a green attempt 3, never
        pass it. Their jobs carry the re-read's attempt, so only those two checks keep it pending."""
        listed = [group_run(1, CI), group_run(2, LEAVES, conclusion="failure", attempt=2)]
        for reread in ({"run_attempt": 1, "conclusion": "success"},
                       {"id": 99, "run_attempt": 3, "conclusion": "success"}):
            with self.subTest(reread=reread):
                code, out, _ = self.run_group(
                    self.QUEUE, group_runs={GROUP: listed}, rereads={2: reread},
                    group_jobs={**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", attempt=reread["run_attempt"])]})
                self.assertEqual(code, 1, out)
                self.assertIn(f"::error::Pending: {LEAVES}: run 2 attempt 2 changed while it was read (now attempt "
                              f"{reread['run_attempt']}, completed); re-run merge-hold", out)
                self.assertNotIn("Passed: quality (viewer)", out)
                self.assertNotIn("run 99", out)


class FetchTests(TestCase):
    @staticmethod
    def pages(*bodies):
        responses = [io.BytesIO(body.encode()) for body in bodies]
        calls = []

        def urlopen(req, timeout):
            calls.append(req.full_url)
            return contextlib.nullcontext(responses[len(calls) - 1])
        return urlopen, calls

    def test_reads_every_page(self):
        full = "[" + ",".join(['{"name": "x"}'] * 100) + "]"
        urlopen, calls = self.pages(full, '[{"name": "last"}]')
        with mock.patch.object(merge_hold.request, "urlopen", urlopen):
            items = merge_hold.fetch_all("t", "https://api.example/x")
        self.assertEqual(len(items), 101)
        self.assertEqual(items[-1], {"name": "last"})
        self.assertEqual(calls, ["https://api.example/x?per_page=100&page=1",
                                 "https://api.example/x?per_page=100&page=2"])

    def test_non_list_payload_is_rejected(self):
        urlopen, _ = self.pages('{"message": "Not Found"}')
        with mock.patch.object(merge_hold.request, "urlopen", urlopen), \
                self.assertRaises(TypeError):
            merge_hold.fetch_all("t", "https://api.example/x")

    def test_page_limit_is_enforced(self):
        full = "[" + ",".join(["{}"] * 100) + "]"
        urlopen, _ = self.pages(*([full] * (merge_hold.MAX_PAGES + 1)))
        with mock.patch.object(merge_hold.request, "urlopen", urlopen), \
                self.assertRaises(ValueError):
            merge_hold.fetch_all("t", "https://api.example/x")



MERGE = "9" * 40
LEAVES = ".github/workflows/leaves.yml"
CI = ".github/workflows/ci.yml"
STORAGE = ".github/workflows/storage.yml"
PULL = {"number": 7, "head": {"sha": HEAD, "ref": "topic", "repo": {"full_name": "o/r"}}, "base": {"ref": "main"}}
OWN_PULL = [{"number": 7, "base": {"ref": "main"}}]
GATE_ENV = dict(ENV, BASE_SHA=BASE, TESTED_SHA=MERGE, GITHUB_RUN_ID="123")


def row(xid, workflow, check, klass="rwa", variant=None, blocked=False):
    return {"id": xid, "workflow": workflow, "job": xid.split("#")[1].split("/")[0], "variant": variant,
            "check_name": check, "class": klass, "blocked_by_filter": blocked}


def plan_fixture(**over):
    plan = {"event": "pull_request", "ref": "main",
            "shas": {"base": BASE, "merge_base": BASE, "head": HEAD, "tested": MERGE},
            "selector": {"trusted": True}, "catalogue": {"source_sha": BASE, "blob": "f" * 40},
            "mode": "select", "policy_mode": "enforce", "fallback": None, "plan_id": "sha256:" + "0" * 64,
            "budgets": {"pull_request_wait_minutes": 0, "merge_group_wait_minutes": 45},
            "selected": [row("ci#host-quality", CI, "host-quality", "core"),
                         row("merge-hold#merge-hold", merge_hold.GATE_WORKFLOW, "merge-hold", "core"),
                         row("leaves#quality/viewer", LEAVES, "quality (viewer)", variant="viewer"),
                         row("storage#s", STORAGE, "s", blocked=True)],
            "excluded": [{"id": "leaves#quality/crop", "class": "rwa", "workflow": LEAVES, "job": "quality",
                          "check_name": "quality (crop)", "reason": "no-input-match"},
                         {"id": "ci#advice", "class": "advisory", "workflow": CI, "job": "advice",
                          "check_name": "advice", "reason": "advisory"}]}
    plan.update(over)
    return plan


def wf_run(run_id, path, status="completed", conclusion="success", attempt=1, **over):
    run = {"id": run_id, "path": path, "status": status, "conclusion": conclusion, "run_attempt": attempt,
           "event": "pull_request", "head_sha": HEAD, "head_branch": "topic", "pull_requests": OWN_PULL,
           "created_at": f"2026-11-24T20:{run_id % 60:02d}:00Z",
           "repository": {"full_name": "o/r"}, "head_repository": {"full_name": "o/r"}}
    run.update(over)
    return run


def job(job_id, run_id, name, conclusion="success", status="completed", attempt=1):
    """A listed job; like GitHub's, it names the attempt it belongs to (wf_run's default is 1 too)."""
    return {"id": job_id, "run_id": run_id, "name": name, "status": status, "conclusion": conclusion,
            "run_attempt": attempt}


GOOD_RUNS = [wf_run(1, CI), wf_run(2, LEAVES)]
GOOD_JOBS = {1: [job(11, 1, "host-quality"), job(12, 1, "advice")], 2: [job(21, 2, "quality (viewer)")]}


def group_run(run_id, path, **over):
    """A merge_group run of a gate workflow on the queue commit GROUP (MergeGroupTests)."""
    return wf_run(run_id, path, **{"event": "merge_group", "head_sha": GROUP, "head_branch": QUEUE_BRANCH,
                                   "pull_requests": [], **over})


def entry_hold_run(commit, number, conclusion="success", run_id=70, path=merge_hold.GATE_WORKFLOW, **over):
    """The merge_group merge-hold run on the queue entry commit of pull request number (a member ahead)."""
    return group_run(run_id, path, **{
        "head_sha": commit, "head_branch": f"gh-readonly-queue/main/pr-{number}-{BASE}",
        "conclusion": conclusion, **over})


def group_plan(**over):
    """The selector's merge_group plan for GROUP on BASE under merge_group = "full": mode full, fallback merge-group."""
    return plan_fixture(**{"event": "merge_group", "mode": "full",
                           "fallback": {"code": "merge-group", "detail": ["merge_group"], "reasons": ["merge-group"]},
                           "shas": {"base": BASE, "merge_base": BASE, "head": GROUP, "tested": GROUP}, **over})


class GateTests(TestCase):
    """Phase 3: the pull_request path verifies every applicable gate from base-side code."""

    def decide(self, plan=None, runs_=None, jobs=None, pull=None, ancestor=0, page_size=100, totals=None,
               polls=None, rerun_during_read=None, settle_during_read=None, later_jobs=None, via_main=False):
        """rerun_during_read: a run that keeps changing; its first re-read shows these fields, each
        later one yet another attempt. settle_during_read: a run that moved on once and then holds
        these fields (a list gives the fields of each re-read in turn, the last one held).
        later_jobs: the jobs of a run from its second jobs read on. An exception in place of fields
        or jobs is raised by that read. via_main: decide through main() with no acceptance hold set;
        returns (exit code is 0, its output, state)."""
        plan = plan or plan_fixture()
        state = {"poll": 0, "calls": [], "sleeps": 0}
        clock = [0.0]
        rereads, job_reads = {}, {}

        def call(token, url, payload=None):
            state["calls"].append(url)
            if url.endswith("/pulls/7"):
                return pull or PULL
            if "/actions/runs/" in url and "?" not in url:  # re-read of one run
                run_id = int(url.rsplit("/", 1)[1])
                listed = runs_ if runs_ is not None else GOOD_RUNS
                if polls:
                    listed = polls[min(state["poll"], len(polls) - 1)]
                current = next(dict(run) for run in listed if run["id"] == run_id)
                if run_id in (settle_during_read or {}):
                    fields = settle_during_read[run_id]
                    if isinstance(fields, list):  # one overlay per re-read; the last one holds
                        rereads[run_id] = rereads.get(run_id, 0) + 1
                        fields = fields[min(rereads[run_id], len(fields)) - 1]
                    if isinstance(fields, Exception):
                        raise fields
                    return {**current, **fields}
                change = (rerun_during_read or {}).get(run_id)
                if change is None:
                    return current
                rereads[run_id] = rereads.get(run_id, 0) + 1
                attempt = change.get("run_attempt", current["run_attempt"]) + rereads[run_id] - 1
                return {**current, **change, "run_attempt": attempt}
            page = int(url.rsplit("page=", 1)[1])
            if "/jobs?" in url:
                run_id = int(url.split("/runs/")[1].split("/")[0])
                job_reads[run_id] = job_reads.get(run_id, 0) + (page == 1)
                listed = (jobs or GOOD_JOBS).get(run_id, [])
                if job_reads[run_id] > 1 and run_id in (later_jobs or {}):
                    listed = later_jobs[run_id]
                if isinstance(listed, Exception):
                    raise listed
                key = "jobs"
            else:
                listed = runs_ if runs_ is not None else GOOD_RUNS
                if polls:
                    listed = polls[min(state["poll"], len(polls) - 1)]
                key = "workflow_runs"
            chunk = listed[(page - 1) * page_size:page * page_size]
            total = (totals or {}).get(key, len(listed))
            return {"total_count": total, key: chunk}

        def sleep(seconds):
            clock[0] += seconds
            state["poll"] += 1
            state["sleeps"] += 1

        with mock.patch.dict(os.environ, GATE_ENV, clear=True), \
                mock.patch.object(merge_hold, "call", call), \
                mock.patch.object(merge_hold, "recompute_plan", lambda *a: plan), \
                mock.patch.object(merge_hold, "git", return_value=(ancestor, "")), \
                mock.patch.object(merge_hold.time, "sleep", sleep), \
                mock.patch.object(merge_hold.time, "monotonic", lambda: clock[0]):
            if via_main:
                out = io.StringIO()
                with mock.patch.object(merge_hold, "fetch_all", lambda token, url: []), \
                        contextlib.redirect_stdout(out):
                    return merge_hold.main() == 0, out.getvalue(), state
            passed, lines = merge_hold.pull_request_gates("x", "o/r", 7, HEAD)
        return passed, "\n".join(lines), state

    def test_all_applicable_gates_passed(self):
        passed, out, _ = self.decide()
        self.assertTrue(passed, out)
        self.assertIn("2 applicable gate(s) passed; 1 filtered out; 0 pending; 0 failed.", out)
        self.assertIn("Not a pass: s: not applicable (path filter)", out)
        self.assertIn("Plan sha256:" + "0" * 64, out)

    def test_filter_hiding_a_registered_input_fails(self):
        # #494 C1: a path filter may hide only metadata; a hidden registered input fails at once.
        plan = plan_fixture()
        plan["selected"][-1]["filter_hidden_inputs"] = ["tests/test_volume_archive.py"]
        passed, out, state = self.decide(plan=plan)
        self.assertFalse(passed, out)
        self.assertIn("::error::Failed: s: its path filter hides registered input(s) "
                      '"tests/test_volume_archive.py", so the gate did not run', out)
        self.assertIn("2 applicable gate(s) passed; 0 filtered out; 0 pending; 1 failed.", out)
        self.assertNotIn("not applicable (path filter)", out)
        self.assertEqual(state["sleeps"], 0)

    def test_filter_hiding_only_metadata_is_not_a_pass_but_does_not_block(self):
        plan = plan_fixture()
        plan["selected"][-1]["filter_hidden_inputs"] = []
        passed, out, _ = self.decide(plan=plan)
        self.assertTrue(passed, out)
        self.assertIn("Not a pass: s: not applicable (path filter)", out)
        self.assertNotIn("Passed: s:", out)

    def test_hidden_input_listing_is_bounded(self):
        paths = [f"tests/t{i}.py" for i in range(7)]
        self.assertEqual(merge_hold.hidden_listing(paths),
                         '"tests/t0.py", "tests/t1.py", "tests/t2.py", "tests/t3.py", "tests/t4.py" and 2 more')
        # A path chosen by the pull request cannot inject a workflow command or break the log.
        listed = merge_hold.hidden_listing(["tests/a\n::error::spoof.py", "tests/\udcff.py"])
        self.assertNotIn("\n", listed)
        listed.encode("ascii")
        self.assertEqual(merge_hold.hidden_listing(["tests/a%0A::error::x.py"]), '"tests/a%250A::error::x.py"')

    def test_missing_or_running_runs_are_pending_and_block(self):
        for runs_, fragment in (([wf_run(1, CI)], "no .github/workflows/leaves.yml run"),
                                ([wf_run(1, CI), wf_run(3, LEAVES, "in_progress", None)], "which is in_progress"),
                                ([wf_run(1, CI), wf_run(3, LEAVES, "queued", None)], "which is queued")):
            with self.subTest(fragment=fragment):
                passed, out, _ = self.decide(runs_=runs_)
                self.assertFalse(passed)
                self.assertIn(fragment, out)
                self.assertIn("gh run rerun 123 --failed", out)

    def test_an_unknown_filter_result_keeps_the_gate_expected_and_names_the_cause(self):
        # #494 K10: an undefined filter result stays fail-closed; only the message says why and what helps.
        from ci_impact import FILTER_FILE_LIMIT
        self.assertEqual(merge_hold.FILTER_FILE_LIMIT, FILTER_FILE_LIMIT)
        cases = (({"changes": {"count": 301}},
                  "its path-filter result is undefined above 300 changed files (this change has 301; "
                  "GitHub evaluates only the first 300), so GitHub may never start it. It stays expected "
                  "(fail-closed, docs/CI-CONTRACT.md K10); if GitHub skipped it, only a smaller pull request "
                  "or a new head clears it"),
                 # Above the limit wins over a fallback code: the count, not the diff, is the cause.
                 ({"changes": {"count": 301}, "mode": "full",
                   "fallback": {"code": "shallow-history", "detail": [], "reasons": ["shallow-history"]}},
                  "its path-filter result is undefined above 300 changed files (this change has 301; "
                  "GitHub evaluates only the first 300), so GitHub may never start it"),
                 ({"changes": {"count": 0}, "mode": "full",
                   "fallback": {"code": "shallow-history", "detail": [], "reasons": ["shallow-history"]}},
                  "its path-filter result is unknown: the changed-file list is incomplete (fallback "
                  "shallow-history), so GitHub may never start it. It stays expected (fail-closed, "
                  "docs/CI-CONTRACT.md K10); if GitHub skipped it, only a re-run with a usable diff or "
                  "a new head clears it"),
                 ({}, "its path-filter result is unknown for a change without changed files, so GitHub "
                      "may never start it"))
        for over, note in cases:
            plan = plan_fixture(**over)
            plan["selected"][2]["blocked_by_filter"] = None  # leaves: no run below
            self.assertIn(plan["selected"][2], merge_hold.expected_gates(plan))
            with self.subTest(plan=over):
                passed, out, _ = self.decide(plan=plan, runs_=[wf_run(1, CI)])
                self.assertFalse(passed, out)
                self.assertIn(f"::error::Pending: quality (viewer): no {LEAVES} run for {HEAD} yet; {note}", out)
                self.assertIn("::error::Exception: a gate with an unknown path-filter result may never start, "
                              "so waiting for it does not help; its Pending line names the remedy.", out)
        # A known filter result keeps the plain line, and a started run is judged as usual.
        passed, out, _ = self.decide(runs_=[wf_run(1, CI)])
        self.assertFalse(passed, out)
        self.assertIn(f"no {LEAVES} run for {HEAD} yet\n", out + "\n")
        self.assertNotIn("Exception:", out)
        plan = plan_fixture()
        plan["selected"][2]["blocked_by_filter"] = None
        passed, out, _ = self.decide(plan=plan)
        self.assertTrue(passed, out)
        self.assertNotIn("path-filter result", out)

    def test_expected_job_outcomes_other_than_success_block(self):
        for conclusion in ("failure", "cancelled", "timed_out", "skipped", "action_required", "neutral", None):
            with self.subTest(conclusion=conclusion):
                jobs = {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", conclusion)]}
                passed, out, _ = self.decide(jobs=jobs)
                self.assertFalse(passed)
                self.assertIn(f"Failed: quality (viewer): {conclusion}", out)

    def test_missing_or_duplicate_matrix_variant_blocks(self):
        for jobs_, fragment in (([], "missing from run 2"),
                                ([job(22, 2, "quality (crop)")], "missing from run 2"),
                                ([job(21, 2, "quality (viewer)"), job(23, 2, "quality (viewer)")],
                                 "Failed: quality (viewer): success")):
            with self.subTest(jobs=jobs_):
                passed, out, _ = self.decide(jobs={**GOOD_JOBS, 2: jobs_})
                self.assertFalse(passed)
                self.assertIn(fragment, out)

    def test_the_newest_run_decides(self):
        older_success = [wf_run(1, CI), wf_run(2, LEAVES), wf_run(3, LEAVES, "completed", "failure")]
        jobs = {**GOOD_JOBS, 3: [job(31, 3, "quality (viewer)", "failure")]}
        passed, out, _ = self.decide(runs_=older_success, jobs=jobs)
        self.assertFalse(passed)
        self.assertIn("quality (viewer): failure in run 3", out)
        running = [wf_run(1, CI), wf_run(2, LEAVES), wf_run(3, LEAVES, "in_progress", None)]
        self.assertFalse(self.decide(runs_=running)[0])
        rerun = [wf_run(1, CI), wf_run(2, LEAVES, conclusion="success", attempt=2)]
        jobs = {**GOOD_JOBS, 2: [job(24, 2, "quality (viewer)", attempt=2)]}
        passed, out, _ = self.decide(runs_=rerun, jobs=jobs)
        self.assertTrue(passed, out)
        self.assertIn("run 2 attempt 2", out)

    def test_runs_without_matching_provenance_are_ignored(self):
        for over in ({"head_branch": "other"}, {"head_repository": {"full_name": "fork/r"}},
                     {"repository": {"full_name": "x/r"}}, {"event": "push"}, {"head_sha": OTHER},
                     {"path": ".github/workflows/leaves.yml@refs/heads/x"}):
            with self.subTest(over=over):
                passed, out, _ = self.decide(runs_=[wf_run(1, CI), dict(wf_run(2, LEAVES), **over)])
                self.assertFalse(passed)
                self.assertIn("no .github/workflows/leaves.yml run", out)
        fork = dict(PULL, head={"sha": HEAD, "ref": "topic", "repo": {"full_name": "fork/r"}})
        runs_ = [wf_run(1, CI, head_repository={"full_name": "fork/r"}),
                 wf_run(2, LEAVES, head_repository={"full_name": "fork/r"})]
        self.assertTrue(self.decide(runs_=runs_, pull=fork)[0])
        jobs = {**GOOD_JOBS, 2: [dict(job(21, 2, "quality (viewer)"), run_id=99)]}
        self.assertFalse(self.decide(jobs=jobs)[0])  # a job of another run is not this run's result

    def test_unexpected_gate_failures_block_and_advisory_ones_do_not(self):
        jobs = {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)"), job(22, 2, "quality (crop)", "failure")]}
        passed, out, _ = self.decide(runs_=[wf_run(1, CI), wf_run(2, LEAVES, conclusion="failure")], jobs=jobs)
        self.assertFalse(passed)
        self.assertIn("quality (crop): failure in run 2", out)
        self.assertIn("not expected by the plan", out)
        jobs = {**GOOD_JOBS, 1: [job(11, 1, "host-quality"), job(12, 1, "advice", "failure")]}
        self.assertTrue(self.decide(runs_=[wf_run(1, CI, conclusion="failure"), wf_run(2, LEAVES)], jobs=jobs)[0])
        jobs = {**GOOD_JOBS, 1: [job(11, 1, "host-quality"), job(13, 1, "renamed", "cancelled")]}
        passed, out, _ = self.decide(runs_=[wf_run(1, CI, conclusion="cancelled"), wf_run(2, LEAVES)], jobs=jobs)
        self.assertFalse(passed)
        self.assertIn("renamed: cancelled", out)
        # A merge-hold run of its own workflow never counts, whatever it concluded.
        runs_ = GOOD_RUNS + [wf_run(5, merge_hold.GATE_WORKFLOW, conclusion="failure")]
        self.assertTrue(self.decide(runs_=runs_)[0])

    def test_phase4_select_job_shapes(self):
        """Phase 4 (#494): merge-hold judges the select job's effect, never its own claim."""
        plan = plan_fixture(policy_mode="select")
        plan["selected"].insert(1, row("leaves#select", LEAVES, "leaves select", "core"))
        template = "quality (${{ matrix.component }})"
        select_ok = job(20, 2, "leaves select")
        # (a) select succeeded; it narrowed the matrix to the planned variant; the rest never ran.
        passed, out, _ = self.decide(plan=plan, jobs={**GOOD_JOBS, 2: [select_ok, job(21, 2, "quality (viewer)")]})
        self.assertTrue(passed, out)
        # (a') nothing of the job was needed: GitHub reports one skipped template job, which is ignored.
        quiet = plan_fixture(policy_mode="select")
        quiet["selected"] = [r for r in quiet["selected"] if r["id"] != "leaves#quality/viewer"]
        quiet["selected"].insert(1, row("leaves#select", LEAVES, "leaves select", "core"))
        quiet["excluded"].append({"id": "leaves#quality/viewer", "class": "rwa", "workflow": LEAVES,
                                  "job": "quality", "check_name": "quality (viewer)", "reason": "no-input-match"})
        jobs = {**GOOD_JOBS, 2: [select_ok, job(21, 2, template, "skipped")]}
        passed, out, _ = self.decide(plan=quiet, jobs=jobs)
        self.assertTrue(passed, out)
        # (b) select narrowed too far: the expected variant is absent from the completed run.
        jobs = {**GOOD_JOBS, 2: [select_ok, job(22, 2, "quality (crop)")]}
        passed, out, _ = self.decide(plan=plan, jobs=jobs)
        self.assertFalse(passed)
        self.assertIn("quality (viewer): missing from run 2", out)
        # ... or skipped the whole job: the skipped template does not stand in for the variant.
        jobs = {**GOOD_JOBS, 2: [select_ok, job(21, 2, template, "skipped")]}
        passed, out, _ = self.decide(plan=plan, jobs=jobs)
        self.assertFalse(passed)
        self.assertIn("quality (viewer): missing from run 2", out)
        # (c) select failed and GitHub skipped its dependents.
        jobs = {**GOOD_JOBS, 2: [job(20, 2, "leaves select", "failure"), job(21, 2, template, "skipped")]}
        passed, out, _ = self.decide(plan=plan, runs_=[wf_run(1, CI), wf_run(2, LEAVES, conclusion="failure")],
                                     jobs=jobs)
        self.assertFalse(passed)
        self.assertIn("Failed: leaves select: failure", out)
        self.assertIn("quality (viewer): missing from run 2", out)

    def test_failed_gate_job_inside_a_successful_run_blocks(self):
        """CodeRabbit on #504: job-level continue-on-error can hide a failure behind a success run."""
        runs_ = GOOD_RUNS + [wf_run(3, STORAGE)]
        jobs = {**GOOD_JOBS, 3: [job(31, 3, "s", "failure")]}
        passed, out, _ = self.decide(runs_=runs_, jobs=jobs)
        self.assertFalse(passed)
        self.assertIn("s: failure in run 3", out)
        self.assertTrue(self.decide(runs_=runs_, jobs={**GOOD_JOBS, 3: [job(31, 3, "s")]})[0])

    def test_paginated_listings_are_read_completely(self):
        filler = [wf_run(100 + index, ".github/workflows/other.yml") for index in range(150)]
        passed, out, state = self.decide(runs_=filler + GOOD_RUNS, page_size=100)
        self.assertTrue(passed, out)
        self.assertEqual(sum("/actions/runs?" in url for url in state["calls"]), 2)
        many = [job(1000 + index, 2, f"other {index}") for index in range(120)] + [job(21, 2, "quality (viewer)")]
        self.assertTrue(self.decide(jobs={**GOOD_JOBS, 2: many})[0])
        with self.assertRaises(ValueError):
            self.decide(totals={"workflow_runs": 500})  # fewer items than the reported total

    def test_plan_must_be_trusted_and_bound_to_this_pull_request(self):
        bad = [
            {"shas": {"base": BASE, "head": OTHER, "tested": MERGE}},
            {"shas": {"base": BASE, "head": HEAD, "tested": OTHER}},
            {"shas": {"base": OTHER, "head": HEAD, "tested": MERGE}},
            {"ref": "release"},
            {"event": "push"},
            {"selector": {"trusted": False}},
            {"catalogue": {"source_sha": HEAD, "blob": "f" * 40}},
            {"catalogue": {"source_sha": BASE, "blob": None}},
            {"mode": "full", "fallback": {"code": "merge-ref-mismatch"}},
            {"mode": "full", "fallback": {"code": "selector-error:KeyError"}},
            {"mode": "full", "fallback": {"code": "invalid-trusted-catalogue"}},
            {"mode": "manual"},
            {"plan_id": "x"},
            {"selected": [row("leaves#quality/viewer", LEAVES, "quality (viewer)")]},
        ]
        for over in bad:
            with self.subTest(over=over), self.assertRaises(ValueError):
                self.decide(plan=plan_fixture(**over))
        self.assertTrue(self.decide(plan=plan_fixture(mode="full", fallback={"code": "always-full"}))[0])

    def test_head_that_moved_or_misses_the_base_tip_fails(self):
        with self.assertRaises(ValueError):
            self.decide(pull=dict(PULL, head={"sha": OTHER, "ref": "topic", "repo": {"full_name": "o/r"}}))
        passed, out, _ = self.decide(ancestor=1)
        self.assertFalse(passed)
        self.assertIn("does not contain the base tip", out)

    def test_shadow_policy_reports_without_blocking(self):
        passed, out, _ = self.decide(plan=plan_fixture(policy_mode="shadow"), runs_=[wf_run(1, CI)])
        self.assertTrue(passed)
        self.assertIn("::warning::(shadow, not blocking) Pending:", out)

    def test_wait_budget_polls_until_the_gates_finish_or_the_deadline(self):
        plan = plan_fixture(budgets={"pull_request_wait_minutes": 3, "merge_group_wait_minutes": 45})
        polls = [[wf_run(1, CI)], [wf_run(1, CI), wf_run(3, LEAVES, "in_progress", None)], GOOD_RUNS]
        passed, out, state = self.decide(plan=plan, polls=polls)
        self.assertTrue(passed, out)
        self.assertEqual(state["sleeps"], 2)
        passed, _, state = self.decide(plan=plan, polls=[[wf_run(1, CI)]])
        self.assertFalse(passed)
        self.assertEqual(state["sleeps"], 3)  # 3 minutes at one poll per minute, then it decides
        passed, _, state = self.decide(polls=[[wf_run(1, CI)]])
        self.assertFalse(passed)
        self.assertEqual(state["sleeps"], 0)  # budget 0: decide at once

    def test_wait_polls_with_one_run_listing_and_reports_only_when_runs_finish(self):
        """#494 maintenance: a 40-minute budget must stay far below the GITHUB_TOKEN rate limit."""
        plan = plan_fixture(budgets={"pull_request_wait_minutes": 40, "merge_group_wait_minutes": 45})
        running = [wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None)]
        polls = [running] * 30 + [GOOD_RUNS]
        passed, out, state = self.decide(plan=plan, polls=polls)
        self.assertTrue(passed, out)
        self.assertEqual(state["sleeps"], 30)
        # Two full reports (start and end), each reading jobs and re-reading every counted run;
        # the 30 polls in between cost one run listing each.
        self.assertEqual(sum("/jobs?" in url for url in state["calls"]), 4)
        self.assertEqual(sum("/actions/runs?" in url for url in state["calls"]), 2 + 1 + 30)

    def test_pending_with_nothing_left_running_decides_at_once(self):
        """A run that already finished after changing between reads cannot resolve by waiting."""
        plan = plan_fixture(budgets={"pull_request_wait_minutes": 40, "merge_group_wait_minutes": 45})
        passed, out, state = self.decide(plan=plan, rerun_during_read={2: {"run_attempt": 2}})
        self.assertFalse(passed)
        self.assertIn("changed while it was read", out)
        self.assertEqual(state["sleeps"], 0)

    def test_an_open_run_without_a_selectable_gate_does_not_hold_the_wait(self):
        """Review of #517: only gated workflows keep the wait going."""
        advisory = ".github/workflows/advisory.yml"
        plan = plan_fixture(budgets={"pull_request_wait_minutes": 40, "merge_group_wait_minutes": 45},
                            excluded=plan_fixture()["excluded"] + [
                                {"id": "adv#a", "class": "advisory", "workflow": advisory, "job": "a",
                                 "check_name": "a", "reason": "advisory"}])
        runs_ = [wf_run(1, CI), wf_run(2, LEAVES), wf_run(5, advisory, "in_progress", None)]
        passed, out, state = self.decide(plan=plan, runs_=runs_, rerun_during_read={2: {"run_attempt": 2}})
        self.assertFalse(passed)
        self.assertEqual(state["sleeps"], 0)

    def test_a_failed_poll_keeps_waiting_and_the_full_report_decides(self):
        """Review of #517: a transient API error during the wait neither ends it nor fails the job."""
        plan = plan_fixture(budgets={"pull_request_wait_minutes": 40, "merge_group_wait_minutes": 45})
        running = [wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None)]
        real = merge_hold.fetch_listing
        failures = [OSError("HTTP 502"), ValueError("listing changed")]
        raised = []

        def flaky(token, url, key):
            if key == "workflow_runs":
                STATE["listings"] += 1
                if STATE["listings"] in (2, 3):  # the first two wait polls; the first listing is the report's
                    raised.append(failures[len(raised)])
                    raise raised[-1]
            return real(token, url, key)

        STATE = {"listings": 0}
        with mock.patch.object(merge_hold, "fetch_listing", flaky):
            passed, out, state = self.decide(plan=plan, polls=[running] * 3 + [GOOD_RUNS])
        self.assertTrue(passed, out)
        self.assertEqual(raised, failures)
        self.assertEqual(state["sleeps"], 3)

    def test_an_unorderable_run_in_a_wait_poll_counts_as_open(self):
        """Graphify on #517: run_order in the poll belongs inside the poll's error handling."""
        plan = plan_fixture(budgets={"pull_request_wait_minutes": 40, "merge_group_wait_minutes": 45})
        unorderable = [wf_run(1, CI), wf_run(2, LEAVES), wf_run(3, LEAVES, created_at=None)]
        with mock.patch.object(merge_hold, "fetch_listing", return_value=unorderable), \
                mock.patch("sys.stderr", new_callable=io.StringIO) as err:
            self.assertTrue(merge_hold.runs_open("t", "o/r", PULL, HEAD, plan))
        self.assertIn("still waiting", err.getvalue())

    def test_a_lone_unorderable_run_in_a_wait_poll_counts_as_open(self):
        """CodeRabbit on #542: the first (here the only) run of a workflow is validated too."""
        plan = plan_fixture(budgets={"pull_request_wait_minutes": 40, "merge_group_wait_minutes": 45})
        lone = [wf_run(1, CI), wf_run(2, LEAVES, created_at=None)]
        with mock.patch.object(merge_hold, "fetch_listing", return_value=lone), \
                mock.patch("sys.stderr", new_callable=io.StringIO) as err:
            self.assertTrue(merge_hold.runs_open("t", "o/r", PULL, HEAD, plan))
        self.assertIn("still waiting", err.getvalue())

    def test_a_gate_that_finishes_during_the_first_report_is_seen(self):
        """CodeRabbit on #517: a stale pending report is refreshed once before it decides."""
        plan = plan_fixture(budgets={"pull_request_wait_minutes": 40, "merge_group_wait_minutes": 45})
        reports = iter([[wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None)]])
        real = merge_hold.fetch_listing

        def finishing(token, url, key):
            if key == "workflow_runs" and "/jobs" not in url:
                stale = next(reports, None)
                if stale is not None:  # the first report still sees leaves running
                    return stale
            return real(token, url, key)

        with mock.patch.object(merge_hold, "fetch_listing", finishing):
            passed, out, state = self.decide(plan=plan)
        self.assertTrue(passed, out)
        self.assertEqual(state["sleeps"], 0)

    def test_a_rerun_that_finishes_during_the_report_after_polling_is_refreshed(self):
        """CodeRabbit on #517: after a poll the next stale report still gets one refresh.

        CodeRabbit on #560 (M4): the settle step can resolve the stale report by itself, so run 2's
        jobs stay at attempt 1 through the settle read of the first report after the poll (Pending:
        jobs of another attempt) and show attempt 2 only from the refresh report on."""
        plan = plan_fixture(budgets={"pull_request_wait_minutes": 40, "merge_group_wait_minutes": 45})
        running = [wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None)]
        done = [wf_run(1, CI), wf_run(2, LEAVES, conclusion="success", attempt=2)]
        reads = {"n": 0, "jobs": 0}
        real = merge_hold.fetch_listing

        def listing(token, url, key):
            if key == "workflow_runs":
                reads["n"] += 1
                if reads["n"] == 4:  # the report after the poll (read 3) still sees attempt 1 running
                    return running
            elif "/actions/runs/2/jobs?" in url:
                reads["jobs"] += 1  # reads 1-3: report 1, then the report after the poll and its settle read
                return ([job(21, 2, "quality (viewer)", None, "in_progress")] if reads["jobs"] <= 3
                        else [job(24, 2, "quality (viewer)", attempt=2)])
            return real(token, url, key)

        with mock.patch.object(merge_hold, "fetch_listing", listing):
            passed, out, state = self.decide(plan=plan, polls=[running, done])
        self.assertTrue(passed, out)
        self.assertEqual(state["sleeps"], 1)
        # The refresh report happened: a sixth run listing and a fourth jobs read of run 2.
        self.assertEqual((reads["n"], reads["jobs"]), (6, 4))

    def test_a_superseded_open_run_does_not_hold_the_wait(self):
        """CodeRabbit on #517: the wait poll applies gate_report's newest-run rule."""
        plan = plan_fixture(budgets={"pull_request_wait_minutes": 40, "merge_group_wait_minutes": 45})
        older_open = dict(wf_run(2, LEAVES, "in_progress", None), created_at="2026-11-24T19:00:00Z")
        newer_open = wf_run(3, LEAVES, "in_progress", None)
        polls = [[wf_run(1, CI), older_open, newer_open], [wf_run(1, CI), older_open, wf_run(3, LEAVES)]]
        jobs = {**GOOD_JOBS, 3: [job(31, 3, "quality (viewer)")]}
        passed, out, state = self.decide(plan=plan, polls=polls, jobs=jobs)
        self.assertTrue(passed, out)
        self.assertEqual(state["sleeps"], 1)

    def test_failure_ends_the_wait_at_once(self):
        plan = plan_fixture(budgets={"pull_request_wait_minutes": 10, "merge_group_wait_minutes": 45})
        runs_ = [wf_run(1, CI, conclusion="failure"), wf_run(2, LEAVES, "queued", None)]
        jobs = {**GOOD_JOBS, 1: [job(11, 1, "host-quality", "failure")]}
        passed, _, state = self.decide(plan=plan, runs_=runs_, jobs=jobs)
        self.assertFalse(passed)
        self.assertEqual(state["sleeps"], 0)

    def test_started_gates_the_plan_did_not_select_must_finish(self):
        """Review B1: an unselected gate that is still running is awaited, not ignored."""
        native = ".github/workflows/native.yml"
        plan = plan_fixture(excluded=plan_fixture()["excluded"] + [
            {"id": "native#n", "class": "rwa", "workflow": native, "job": "n", "check_name": "native",
             "reason": "no-input-match"},
            {"id": "native#diag", "class": "manual", "workflow": native, "job": "diag", "check_name": "diag",
             "reason": "manual"}])
        runs_ = GOOD_RUNS + [wf_run(7, native, "in_progress", None)]
        jobs = {**GOOD_JOBS, 7: [job(71, 7, "native", None, "in_progress"), job(72, 7, "diag", None, "queued")]}
        passed, out, _ = self.decide(plan=plan, runs_=runs_, jobs=jobs)
        self.assertFalse(passed)
        self.assertIn("native: in_progress in run 7", out)
        self.assertNotIn("diag:", out)  # a manual job never blocks
        jobs = {**GOOD_JOBS, 7: [job(71, 7, "native", "failure"), job(72, 7, "diag", None, "queued")]}
        passed, out, _ = self.decide(plan=plan, runs_=runs_, jobs=jobs)
        self.assertFalse(passed)
        self.assertIn("native: failure in run 7", out)
        # An expected job that already succeeded counts while the rest of its run goes on.
        jobs = {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)"), job(22, 2, "quality (crop)", None, "queued")]}
        runs_ = [wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None)]
        passed, out, _ = self.decide(runs_=runs_, jobs=jobs)
        self.assertFalse(passed)
        self.assertIn("Passed: quality (viewer): success in run 2", out)
        self.assertIn("quality (crop): queued in run 2", out)

    def test_a_running_gate_workflow_is_pending_even_when_every_listed_job_finished(self):
        """Delta review SF1: jobs waiting on `needs` or a runtime matrix may not be listed yet."""
        native = ".github/workflows/native.yml"
        plan = plan_fixture(excluded=plan_fixture()["excluded"] + [
            {"id": "native#n", "class": "rwa", "workflow": native, "job": "n", "check_name": "native",
             "reason": "no-input-match"}])
        runs_ = GOOD_RUNS + [wf_run(7, native, "in_progress", None)]
        passed, out, _ = self.decide(plan=plan, runs_=runs_, jobs={**GOOD_JOBS, 7: [job(71, 7, "native")]})
        self.assertFalse(passed)
        self.assertIn("native.yml: run 7 attempt 1 is still in_progress", out)
        # A running workflow without any gate (advisory only) does not hold merge-hold.
        advisory = ".github/workflows/advice.yml"
        plan = plan_fixture(excluded=plan_fixture()["excluded"] + [
            {"id": "advice#a", "class": "advisory", "workflow": advisory, "job": "a", "check_name": "a",
             "reason": "advisory"}])
        runs_ = GOOD_RUNS + [wf_run(8, advisory, "in_progress", None)]
        self.assertTrue(self.decide(plan=plan, runs_=runs_, jobs={**GOOD_JOBS, 8: []})[0])

    def test_job_classes_come_from_the_same_workflow_only(self):
        """CodeRabbit on #504: an unregistered job must not borrow another workflow's advisory class."""
        jobs = {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)"), job(22, 2, "advice", "failure")]}
        passed, out, _ = self.decide(runs_=[wf_run(1, CI), wf_run(2, LEAVES, conclusion="failure")], jobs=jobs)
        self.assertFalse(passed)
        self.assertIn("advice: failure in run 2", out)

    def test_a_run_that_changes_between_reads_is_pending(self):
        """CodeRabbit on #504: a re-run between the run and job reads must not pair mismatched data."""
        passed, out, _ = self.decide(rerun_during_read={2: {"run_attempt": 2, "status": "queued"}})
        self.assertFalse(passed)
        self.assertIn("changed while it was read (now attempt 2, queued)", out)

    def test_a_skipped_non_gate_run_that_completes_between_reads_is_not_pending(self):
        """#494: a label-skipped run that completes between reads must not leave merge-hold Pending."""
        probe = ".github/workflows/probe.yml"
        plan = plan_fixture(excluded=plan_fixture()["excluded"] + [
            {"id": "probe#p", "class": "manual", "workflow": probe, "job": "p", "check_name": "p",
             "reason": "manual"}])
        runs_ = GOOD_RUNS + [wf_run(5, probe, "queued", None)]
        jobs = {**GOOD_JOBS, 5: [job(51, 5, "p", "skipped")]}
        passed, out, _ = self.decide(plan=plan, runs_=runs_, jobs=jobs,
                                     settle_during_read={5: {"status": "completed", "conclusion": "skipped"}})
        self.assertTrue(passed, out)
        self.assertNotIn("changed while it was read", out)

    def test_a_gate_run_that_completes_between_reads_is_judged_by_its_new_state(self):
        running = [wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None)]
        jobs = {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", "failure")]}
        passed, out, _ = self.decide(runs_=running, jobs=jobs,
                                     settle_during_read={2: {"status": "completed", "conclusion": "failure"}})
        self.assertFalse(passed)
        self.assertIn("Failed: quality (viewer): failure in run 2 attempt 1", out)
        self.assertIn("0 pending; 1 failed.", out)

    def test_a_run_that_moved_on_is_judged_by_the_jobs_of_its_new_state(self):
        """The first jobs read belongs to the old attempt; a failed new attempt must not pass on it."""
        passed, out, _ = self.decide(
            settle_during_read={2: {"run_attempt": 2, "conclusion": "failure"}},
            later_jobs={2: [job(24, 2, "quality (viewer)", "failure", attempt=2)]})
        self.assertFalse(passed)
        self.assertIn("Failed: quality (viewer): failure in run 2 attempt 2", out)

    def test_a_running_gate_that_settles_as_failed_is_judged_on_the_settled_record(self):
        """The listing showed it running; the settled record and its second jobs read decide."""
        running = [wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None)]
        jobs = {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", None, "in_progress")]}
        passed, out, _ = self.decide(runs_=running, jobs=jobs,
                                     settle_during_read={2: {"status": "completed", "conclusion": "failure"}},
                                     later_jobs={2: [job(21, 2, "quality (viewer)", "failure")]})
        self.assertFalse(passed)
        self.assertIn("Failed: quality (viewer): failure in run 2 attempt 1", out)
        self.assertIn("0 pending; 1 failed.", out)

    def test_a_failed_gate_that_settles_in_a_new_running_attempt_is_pending(self):
        """The failure belongs to the old attempt; the new one runs and lists no job yet.

        Should `filter=latest` still list the old attempt's failed job, the jobs belong to another
        attempt and the run stays pending as well: see the P2 test below."""
        failed = [wf_run(1, CI), wf_run(2, LEAVES, conclusion="failure")]
        jobs = {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", "failure")]}
        passed, out, _ = self.decide(
            runs_=failed, jobs=jobs, later_jobs={2: []},
            settle_during_read={2: {"run_attempt": 2, "status": "in_progress", "conclusion": None}})
        self.assertFalse(passed)
        self.assertIn("Pending: .github/workflows/leaves.yml: run 2 attempt 2 is still in_progress", out)
        self.assertIn("Pending: quality (viewer): not started yet in run 2 attempt 2, which is in_progress", out)
        self.assertIn("0 failed.", out)

    def test_a_run_that_keeps_changing_stays_pending_even_when_each_read_looks_green(self):
        passed, out, _ = self.decide(rerun_during_read={2: {"run_attempt": 2, "status": "completed"}})
        self.assertFalse(passed)
        self.assertIn("Pending: .github/workflows/leaves.yml: run 2 attempt 1 changed while it was read "
                      "(now attempt 2, completed)", out)

    def test_a_run_whose_status_moves_on_during_the_jobs_re_read_stays_pending(self):
        """The second jobs read saw the new attempt in progress; a job created after it may still
        fail, so the completed record must not be judged on those jobs."""
        passed, out, _ = self.decide(settle_during_read={2: [
            {"run_attempt": 2, "status": "in_progress", "conclusion": None},
            {"run_attempt": 2, "status": "completed", "conclusion": "success"}]})
        self.assertFalse(passed)
        self.assertIn("Pending: .github/workflows/leaves.yml: run 2 attempt 1 changed while it was read "
                      "(now attempt 2, in_progress)", out)

    def test_a_re_read_that_names_another_run_stays_pending(self):
        """GitHub returns the requested run; should a re-read ever name another id, the jobs read
        beside it cannot be tied to this run, so it stays pending under the run that was requested.
        A settled record is judged under the requested run: see the moved-on test above."""
        for reads in ([{"id": 99}, {}],  # the second read, attempt and status unchanged
                      [{"id": 99, "run_attempt": 2}, {"run_attempt": 2}],
                      [{"run_attempt": 2}, {"id": 99, "run_attempt": 2}],  # the third read
                      {"id": 99, "run_attempt": 2, "conclusion": "failure"}):  # both reads
            with self.subTest(reads=reads):
                passed, out, _ = self.decide(settle_during_read={2: reads},
                                             later_jobs={2: [job(24, 2, "quality (viewer)", "failure")]})
                self.assertFalse(passed)
                self.assertIn("Pending: .github/workflows/leaves.yml: run 2 attempt 1 changed while it was read",
                              out)
                self.assertIn("1 pending; 0 failed.", out)
                self.assertNotIn("run 99", out)

    def test_a_re_read_older_than_the_listing_never_passes(self):
        """D2 on CI-SETTLE: both re-reads may lag behind the run listing. A failed attempt 2 in the
        listing must not pass on an attempt 1 that both re-reads still return green."""
        failed = [wf_run(1, CI), wf_run(2, LEAVES, conclusion="failure", attempt=2)]
        passed, out, _ = self.decide(
            runs_=failed, jobs={**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", "failure")]},
            settle_during_read={2: {"run_attempt": 1, "conclusion": "success"}},
            later_jobs={2: [job(21, 2, "quality (viewer)")]})
        self.assertFalse(passed)
        self.assertIn("Pending: .github/workflows/leaves.yml: run 2 attempt 2 changed while it was read "
                      "(now attempt 1, completed)", out)
        self.assertNotIn("Passed: quality (viewer)", out)

    def test_a_queued_attempt_is_not_judged_on_an_older_completed_re_read(self):
        """D2 on CI-SETTLE: the listing shows attempt 2 queued; the lagging re-reads show attempt 1
        completed and green, which says nothing about attempt 2."""
        queued = [wf_run(1, CI), wf_run(2, LEAVES, "queued", None, attempt=2)]
        passed, out, _ = self.decide(
            runs_=queued, jobs={**GOOD_JOBS, 2: []},
            settle_during_read={2: {"run_attempt": 1, "status": "completed", "conclusion": "success"}},
            later_jobs={2: [job(21, 2, "quality (viewer)")]})
        self.assertFalse(passed)
        self.assertIn("Pending: .github/workflows/leaves.yml: run 2 attempt 2 changed while it was read "
                      "(now attempt 1, completed)", out)
        self.assertNotIn("Passed: quality (viewer)", out)

    def test_a_run_attempt_that_is_not_an_integer_stays_pending(self):
        """Without an integer attempt in the listing and the settled record, their order is unknown."""
        for listed, settled in ((None, 1), ("1", 1), (1, "2"), (1, None), (True, 2)):
            with self.subTest(listed=listed, settled=settled):
                passed, out, _ = self.decide(runs_=[wf_run(1, CI), wf_run(2, LEAVES, attempt=listed)],
                                             settle_during_read={2: {"run_attempt": settled}})
                self.assertFalse(passed)
                self.assertIn(f"Pending: .github/workflows/leaves.yml: run 2 attempt {listed} changed while "
                              f"it was read (now attempt {settled}, completed)", out)

    def test_an_api_error_in_a_settle_read_fails_closed(self):
        """D2 on CI-SETTLE: an HTTPError is an OSError, so main() fails closed on one in the second
        jobs read or the third run read; the run is never judged on a partial settle."""
        error = HTTPError("https://api.github.com/x", 502, "Bad Gateway", None, None)
        cases = {"second jobs read": ({2: {"run_attempt": 2}}, {2: error}, "/actions/runs/2/jobs?filter=latest&"),
                 "third run read": ({2: [{"run_attempt": 2}, error]}, None, "/actions/runs/2")}
        for read, (settle, later, url) in cases.items():
            with self.subTest(read=read):
                passed, out, state = self.decide(settle_during_read=settle, later_jobs=later, via_main=True)
                self.assertFalse(passed)
                self.assertIn("::error::merge-hold could not decide (HTTP Error 502: Bad Gateway); failing closed.",
                              out)
                self.assertNotIn("Passed:", out)
                failing = state["calls"][-1]  # the read that raised, and it was the second of its kind
                self.assertIn(url, failing)
                self.assertEqual(state["calls"].count(failing), 2)

    def test_an_advisory_run_that_settles_in_progress_is_not_counted_as_passed(self):
        """A non-gated run that moved on to a running attempt adds no Passed line, and like an
        advisory run listed as running it holds nothing."""
        advisory = ".github/workflows/advisory.yml"
        plan = plan_fixture(excluded=plan_fixture()["excluded"] + [
            {"id": "adv#a", "class": "advisory", "workflow": advisory, "job": "a", "check_name": "a",
             "reason": "advisory"}])
        passed, out, _ = self.decide(
            plan=plan, runs_=GOOD_RUNS + [wf_run(5, advisory)], jobs={**GOOD_JOBS, 5: [job(51, 5, "a")]},
            settle_during_read={5: {"run_attempt": 2, "status": "in_progress", "conclusion": None}},
            later_jobs={5: [job(52, 5, "a", None, "in_progress", attempt=2)]})
        self.assertTrue(passed, out)
        self.assertIn("2 applicable gate(s) passed; 1 filtered out; 0 pending; 0 failed.", out)
        self.assertNotIn("run 5", out)

    def test_a_settled_record_that_no_longer_lists_this_pull_request_is_not_counted(self):
        """The settled record replaces the listed one, so it must keep the pull_request binding too.
        Its jobs carry attempt 2, so the job attempt binding does not hide a binding line that
        lets the jobs be judged anyway."""
        passed, out, _ = self.decide(settle_during_read={2: {"run_attempt": 2, "pull_requests": []}},
                                     later_jobs={2: [job(24, 2, "quality (viewer)", attempt=2)]})
        self.assertFalse(passed)
        self.assertIn(f"Pending: {LEAVES}: run 2 attempt 2 no longer matches {HEAD} when read again; "
                      "re-run merge-hold", out)
        self.assertNotIn("Passed: quality (viewer)", out)

    def test_the_jobs_re_read_keeps_only_jobs_of_the_run_that_was_read(self):
        """A job of another run in the second jobs read must not decide this run's gate."""
        passed, out, _ = self.decide(
            settle_during_read={2: {"run_attempt": 2}},
            later_jobs={2: [job(24, 2, "quality (viewer)", attempt=2), job(31, 3, "quality (viewer)", "failure")]})
        self.assertTrue(passed, out)
        self.assertIn("quality (viewer): success in run 2 attempt 2", out)

    def test_a_settle_that_drops_a_job_of_the_judged_attempt_stays_pending(self):
        """CodeRabbit on #560 (A): the first jobs read lists a failed unregistered job of the judged
        attempt and the second read omits it; that evidence is never dropped, so a run that settles
        green is not judged. The first read may run ahead of the listing and hold the settled
        attempt's jobs already, so the line names the settled attempt."""
        cases = {"same attempt": ([wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None)],
                                  [job(21, 2, "quality (viewer)"), job(22, 2, "late-gate", "failure")],
                                  {"status": "completed", "conclusion": "success"}, job(21, 2, "quality (viewer)"), 1),
                 "first read ahead": (GOOD_RUNS, [job(24, 2, "quality (viewer)", attempt=2),
                                                  job(25, 2, "late-gate", "failure", attempt=2)],
                                      {"run_attempt": 2}, job(24, 2, "quality (viewer)", attempt=2), 2)}
        for name, (runs_, first, settle, kept, attempt) in cases.items():
            with self.subTest(name):
                passed, out, _ = self.decide(runs_=runs_, jobs={**GOOD_JOBS, 2: first},
                                             settle_during_read={2: settle}, later_jobs={2: [kept]})
                self.assertFalse(passed)
                self.assertIn(f"Pending: {LEAVES}: run 2 attempt {attempt} dropped job(s) late-gate between "
                              "two reads; re-run merge-hold", out)
                self.assertIn("1 pending; 0 failed.", out)
                self.assertNotIn("Passed: quality (viewer)", out)

    def test_a_settle_that_drops_a_job_without_an_integer_attempt_stays_pending(self):
        """D2 round 7 (S5): a failed job of the first read whose run_attempt is missing or not an
        integer cannot be shown to belong to an older attempt; the second read omits it, so the run
        that settles green stays pending. A JSON boolean is no attempt either: read as an int it would
        pass for an older attempt and escape the comparison (False as 0 against judged attempt 1, True
        as 1 against judged attempt 2)."""
        for attempt, judged, kept in ((None, 1, 21), ("1", 1, 21), (1.5, 1, 21), (False, 1, 21), (True, 2, 24)):
            with self.subTest(attempt=attempt, judged=judged):
                late = job(22, 2, "late-gate", "failure", attempt=attempt)
                if attempt is None:
                    del late["run_attempt"]
                passed, out, _ = self.decide(
                    runs_=[wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None)],
                    jobs={**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)"), late]},
                    settle_during_read={2: {"status": "completed", "conclusion": "success", "run_attempt": judged}},
                    later_jobs={2: [job(kept, 2, "quality (viewer)", attempt=judged)]})
                self.assertFalse(passed)
                self.assertIn(f"Pending: {LEAVES}: run 2 attempt {judged} dropped job(s) late-gate between two "
                              "reads; re-run merge-hold", out)
                self.assertIn("1 pending; 0 failed.", out)
                self.assertNotIn("Passed: quality (viewer)", out)

    def test_a_settle_that_changes_a_completed_job_of_the_judged_attempt_stays_pending(self):
        """D2 round 7 (S1): the same job id is completed and failed in the first read and completed
        and green in the second, in the same attempt (also for a job without an integer attempt);
        a completed job does not change, so the green record is not judged."""
        for name, (first, second) in {
                "failure to success": (job(22, 2, "late-gate", "failure"), job(22, 2, "late-gate")),
                "completed to running": (job(22, 2, "late-gate", "failure"),
                                         job(22, 2, "late-gate", None, "in_progress")),
                "completed to running, same conclusion": (job(22, 2, "late-gate", "failure"),
                                                          job(22, 2, "late-gate", "failure", "in_progress")),
                "no integer attempt": (job(22, 2, "late-gate", "failure", attempt="1"),
                                       job(22, 2, "late-gate", attempt="1"))}.items():
            with self.subTest(name):
                passed, out, _ = self.decide(
                    runs_=[wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None)],
                    jobs={**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)"), first]},
                    settle_during_read={2: {"status": "completed", "conclusion": "success"}},
                    later_jobs={2: [job(21, 2, "quality (viewer)"), second]})
                self.assertFalse(passed)
                self.assertIn(f"Pending: {LEAVES}: run 2 attempt 1 changed the completed record of job(s) "
                              "late-gate between two reads; re-run merge-hold", out)
                self.assertIn("1 pending; 0 failed.", out)
                self.assertNotIn("Passed: quality (viewer)", out)
        # Controls: a completed job read twice the same way, and a running job that completes, settle.
        for name, first in {"unchanged": job(22, 2, "late-gate"),
                            "completes": job(22, 2, "late-gate", None, "in_progress")}.items():
            with self.subTest(name):
                passed, out, _ = self.decide(
                    runs_=[wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None)],
                    jobs={**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)"), first]},
                    settle_during_read={2: {"status": "completed", "conclusion": "success"}},
                    later_jobs={2: [job(21, 2, "quality (viewer)"), job(22, 2, "late-gate")]})
                self.assertTrue(passed, out)
                self.assertIn("Passed: quality (viewer): success in run 2 attempt 1", out)

    def test_a_settle_that_renames_a_completed_job_of_the_judged_attempt_stays_pending(self):
        """D2 round 8: the expected gate's job is completed and failed in the first read; the second
        read lists the same job id, still failed, under an advisory job's name, and a new job id with
        the expected name that succeeded. Status and conclusion match, so only the name shows that
        the completed record changed; the advisory failure would explain the failed run, and the
        new job would pass the gate."""
        plan = plan_fixture()
        plan["excluded"] = plan["excluded"] + [{"id": "leaves#lint", "class": "advisory", "workflow": LEAVES,
                                                "job": "lint", "check_name": "lint", "reason": "advisory"}]
        passed, out, _ = self.decide(
            plan=plan, runs_=[wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None)],
            jobs={**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", "failure")]},
            settle_during_read={2: {"status": "completed", "conclusion": "failure"}},
            later_jobs={2: [job(21, 2, "lint", "failure"), job(23, 2, "quality (viewer)")]})
        self.assertFalse(passed)
        self.assertIn(f"Pending: {LEAVES}: run 2 attempt 1 changed the completed record of job(s) "
                      "quality (viewer) between two reads; re-run merge-hold", out)
        self.assertIn("1 pending; 0 failed.", out)
        self.assertNotIn("Passed: quality (viewer)", out)

    def test_a_settle_back_to_an_older_attempt_than_listed_jobs_stays_pending(self):
        """D2 round 7 (S4): the first jobs read already lists attempt 2 with a failed job, and both
        re-reads and the second jobs read went back to a green attempt 1. Attempt 2's failure must
        not be lost."""
        passed, out, _ = self.decide(
            runs_=[wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None)],
            jobs={**GOOD_JOBS, 2: [job(24, 2, "quality (viewer)", attempt=2),
                                   job(25, 2, "late-gate", "failure", attempt=2)]},
            settle_during_read={2: {"status": "completed", "conclusion": "success"}},
            later_jobs={2: [job(21, 2, "quality (viewer)")]})
        self.assertFalse(passed)
        self.assertIn(f"Pending: {LEAVES}: run 2 attempt 1 first listed jobs of a newer attempt (2) than the one "
                      "it settled on; re-run merge-hold", out)
        self.assertIn("1 pending; 0 failed.", out)
        self.assertNotIn("Passed: quality (viewer)", out)

    def test_a_settle_compares_only_jobs_of_the_judged_attempt(self):
        """Controls for A: jobs of an older attempt in the first read are neither compared nor carried
        into the judged attempt, and a second read that keeps every job of the judged attempt (extra
        jobs beside them) is judged as before."""
        older = {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)"), job(22, 2, "late-gate", "failure")]}
        passed, out, _ = self.decide(jobs=older, settle_during_read={2: {"run_attempt": 2}},
                                     later_jobs={2: [job(24, 2, "quality (viewer)", attempt=2)]})
        self.assertTrue(passed, out)
        self.assertIn("Passed: quality (viewer): success in run 2 attempt 2", out)
        self.assertNotIn("late-gate", out)
        running = [wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None)]
        first = {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", None, "in_progress"),
                                  job(22, 2, "late-gate", None, "in_progress")]}
        cases = {"success": [job(21, 2, "quality (viewer)"), job(22, 2, "late-gate"), job(23, 2, "new-gate")],
                 "failure": [job(21, 2, "quality (viewer)"), job(22, 2, "late-gate", "failure")]}
        for conclusion, later in cases.items():
            with self.subTest(conclusion):
                passed, out, _ = self.decide(runs_=running, jobs=first, later_jobs={2: later},
                                             settle_during_read={2: {"status": "completed", "conclusion": conclusion}})
                self.assertNotIn("dropped job(s)", out)
                self.assertEqual(passed, conclusion == "success", out)
                if conclusion == "failure":
                    self.assertIn(f"Failed: late-gate: failure in run 2 attempt 1 ({LEAVES}); not expected", out)
                    self.assertIn("0 pending; 1 failed.", out)

    def test_a_completed_run_re_read_as_running_in_the_same_attempt_stays_pending(self):
        """CodeRabbit on #560 (B): the listing shows attempt 1 completed and both re-reads show that
        same attempt in progress. Judged as running, a failed run would skip the conclusion
        cross-check, and a workflow outside gated_workflows would pass with its failed job omitted."""
        cond = (self.cond_plan(), GOOD_RUNS + [wf_run(5, self.COND, conclusion="failure")],
                {**GOOD_JOBS, 5: [job(51, 5, "c", "skipped")]}, self.COND, 5, "failure")
        cases = {"gated": ((None, GOOD_RUNS, GOOD_JOBS, LEAVES, 2, "success"), None),
                 "not gated": (cond, None),
                 # The re-read keeps the listed conclusion and only its status says running.
                 "not gated, conclusion kept": (cond, "failure")}
        for name, ((plan, runs_, jobs, workflow, run_id, conclusion), reread) in cases.items():
            with self.subTest(name):
                passed, out, _ = self.decide(plan=plan, runs_=runs_, jobs=jobs, settle_during_read={
                    run_id: {"status": "in_progress", "conclusion": reread}})
                self.assertFalse(passed)
                self.assertIn(f"Pending: {workflow}: run {run_id} attempt 1 was listed completed ({conclusion}) but "
                              f"re-read as in_progress ({reread}) in the same attempt; re-run merge-hold", out)
                self.assertIn("1 pending; 0 failed.", out)

    def test_a_same_attempt_conclusion_change_is_not_judged(self):
        """read_state compares the conclusion, so more runs reach the settle step; a same-attempt
        conclusion change that does not come with completion is held there, never judged. A completed
        attempt does not change its conclusion, and a running one gains one only when it completes."""
        cases = {
            # Listed failed, its jobs omit the failure (Pending on main); both re-reads say success.
            "completed failure to success": (None, [wf_run(1, CI), wf_run(2, LEAVES, conclusion="failure")],
                                             GOOD_JOBS, None, {2: {"conclusion": "success"}}, LEAVES, 2,
                                             "completed (failure)", "completed (success)"),
            # Listed failed without any job (Failed on main); both re-reads say success.
            "zero-job failure to success": (self.cond_plan(), GOOD_RUNS + [wf_run(5, self.COND, conclusion="failure")],
                                            {**GOOD_JOBS, 5: []}, None, {5: {"conclusion": "success"}}, self.COND, 5,
                                            "completed (failure)", "completed (success)"),
            # A non-gated run listed running with a running job (Pending on main) gains a conclusion
            # while both re-reads still say in_progress; its job completes green in the second read.
            "running gains a conclusion": (self.cond_plan(),
                                           GOOD_RUNS + [wf_run(5, self.COND, "in_progress", None)],
                                           {**GOOD_JOBS, 5: [job(51, 5, "c", None, "in_progress")]},
                                           {5: [job(51, 5, "c")]}, {5: {"conclusion": "failure"}}, self.COND, 5,
                                           "in_progress (None)", "in_progress (failure)")}
        for name, (plan, runs_, jobs, later, settle, workflow, run_id, listed, reread) in cases.items():
            with self.subTest(name):
                passed, out, _ = self.decide(plan=plan, runs_=runs_, jobs=jobs, later_jobs=later,
                                             settle_during_read=settle)
                self.assertFalse(passed)
                self.assertIn(f"Pending: {workflow}: run {run_id} attempt 1 was listed {listed} but re-read as "
                              f"{reread} in the same attempt; re-run merge-hold", out)
                self.assertIn("1 pending; 0 failed.", out)

    def test_re_reads_that_disagree_on_conclusion_or_head_stay_pending(self):
        """D2 round 7 (S3): the second and third reads agree on id, attempt and status but not on the
        conclusion (or the head); the third record is not judged. A re-read whose head differs from
        the listing's is not judged either, even when both re-reads agree on it."""
        running = [wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None)]
        first = {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", None, "in_progress")]}
        cases = {
            "failure then success": (running, first, [{"status": "completed", "conclusion": "failure"},
                                                      {"status": "completed", "conclusion": "success"}],
                                     "run 2 attempt 1 changed while it was read (now attempt 1, completed)"),
            "failure then none": (running, first, [{"status": "completed", "conclusion": "failure"},
                                                   {"status": "completed", "conclusion": None}],
                                  "run 2 attempt 1 changed while it was read (now attempt 1, completed)"),
            "heads differ": (running, first, [{"status": "completed", "conclusion": "success", "head_sha": "b" * 40},
                                              {"status": "completed", "conclusion": "success"}],
                             "run 2 attempt 1 changed while it was read (now attempt 1, completed)"),
            "both re-reads name another head": (GOOD_RUNS, GOOD_JOBS, {"head_sha": "b" * 40},
                                                "run 2 attempt 1 changed while it was read (now attempt 1, completed)")}
        for name, (runs_, jobs, settle, line) in cases.items():
            with self.subTest(name):
                passed, out, _ = self.decide(runs_=runs_, jobs=jobs, settle_during_read={2: settle},
                                             later_jobs={2: [job(21, 2, "quality (viewer)")]})
                self.assertFalse(passed)
                self.assertIn(f"Pending: {LEAVES}: {line}; re-run merge-hold", out)
                self.assertIn("1 pending; 0 failed.", out)
                self.assertNotIn("Passed: quality (viewer)", out)

    def test_a_new_attempt_or_a_same_attempt_status_change_is_judged_as_before(self):
        """Controls for B: a newer attempt that starts in progress, a run listed queued that re-reads
        in progress, and a run listed in progress that completes in the same attempt are judged on
        the settled record as before."""
        passed, out, _ = self.decide(
            plan=self.cond_plan(), runs_=GOOD_RUNS + [wf_run(5, self.COND, conclusion="failure")],
            jobs={**GOOD_JOBS, 5: [job(51, 5, "c", "skipped")]}, later_jobs={5: []},
            settle_during_read={5: {"run_attempt": 2, "status": "in_progress", "conclusion": None}})
        self.assertTrue(passed, out)
        self.assertNotIn("run 5", out)
        passed, out, _ = self.decide(runs_=[wf_run(1, CI), wf_run(2, LEAVES, "queued", None)],
                                     jobs={**GOOD_JOBS, 2: []}, settle_during_read={2: {"status": "in_progress"}})
        self.assertFalse(passed)
        self.assertIn(f"Pending: {LEAVES}: run 2 attempt 1 is still in_progress; its gates must finish", out)
        self.assertNotIn("listed completed", out)
        passed, out, _ = self.decide(runs_=[wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None)],
                                     jobs={**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", None, "in_progress")]},
                                     settle_during_read={2: {"status": "completed", "conclusion": "success"}},
                                     later_jobs={2: [job(21, 2, "quality (viewer)")]})
        self.assertTrue(passed, out)
        self.assertIn("Passed: quality (viewer): success in run 2 attempt 1", out)

    def assert_jobs_of_another_attempt(self, passed, out, attempt, workflow=LEAVES, run_id=2):
        """D2 on CI-SETTLE: jobs listed for another attempt than the judged run hold it as a whole."""
        self.assertFalse(passed)
        self.assertIn(f"Pending: {workflow}: run {run_id} attempt {attempt} listed jobs of another attempt; "
                      "re-run merge-hold", out)
        self.assertIn("1 pending; 0 failed.", out)
        self.assertIn("a failure with only Pending lines needs no fix", out)
        self.assertNotIn('says "if it persists"', out)
        self.assertNotIn("Passed: quality (viewer)", out)

    def test_a_settled_attempt_is_not_judged_on_lagging_jobs_of_the_listed_one(self):
        """N1: the listing shows attempt 1 green, both re-reads attempt 2 failed, and both jobs reads
        still return attempt 1's green jobs; they say nothing about attempt 2."""
        passed, out, _ = self.decide(settle_during_read={2: {"run_attempt": 2, "conclusion": "failure"}})
        self.assert_jobs_of_another_attempt(passed, out, 2)

    def test_an_unchanged_run_is_not_judged_on_jobs_of_an_older_attempt(self):
        """N2: listing and re-read agree on a failed attempt 2; the jobs read lags at attempt 1."""
        runs_ = [wf_run(1, CI), wf_run(2, LEAVES, conclusion="failure", attempt=2)]
        passed, out, _ = self.decide(runs_=runs_)
        self.assert_jobs_of_another_attempt(passed, out, 2)

    def test_a_run_is_not_judged_on_jobs_of_a_newer_attempt(self):
        """N3: jobs of attempt 3 (carried-over green jobs, its re-run job not listed yet) say nothing
        about a completed attempt 2, nor does attempt 2's success cover attempt 3."""
        newer = {2: [job(25, 2, "quality (viewer)", attempt=3)]}
        cases = {"unchanged": ([wf_run(1, CI), wf_run(2, LEAVES, attempt=2)], {**GOOD_JOBS, **newer}, None, None),
                 "settled": ([wf_run(1, CI), wf_run(2, LEAVES, conclusion="failure")], None, newer,
                             {2: {"run_attempt": 2, "conclusion": "success"}})}
        for name, (runs_, jobs, later, settle) in cases.items():
            with self.subTest(name):
                passed, out, _ = self.decide(runs_=runs_, jobs=jobs, later_jobs=later, settle_during_read=settle)
                self.assert_jobs_of_another_attempt(passed, out, 2)

    def test_a_listed_running_attempt_that_settles_is_not_judged_on_older_jobs(self):
        """regress2: the listing shows attempt 2 running, the re-reads attempt 2 completed and failed,
        and the jobs read still returns attempt 1's green jobs."""
        runs_ = [wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None, attempt=2)]
        passed, out, _ = self.decide(runs_=runs_,
                                     settle_during_read={2: {"status": "completed", "conclusion": "failure"}})
        self.assert_jobs_of_another_attempt(passed, out, 2)

    def test_a_condition_false_workflow_is_not_judged_on_a_stale_skipped_job(self):
        """G4: a condition-false rwa workflow holds no gate, but its attempt 2 failed; the stale
        skipped job of attempt 1 must not let that failure go unseen."""
        cond = ".github/workflows/cond.yml"
        plan = plan_fixture(excluded=plan_fixture()["excluded"] + [
            {"id": "cond#c", "class": "rwa", "workflow": cond, "job": "c", "check_name": "c",
             "reason": "condition-false"}])
        runs_ = GOOD_RUNS + [wf_run(5, cond, conclusion="skipped")]
        passed, out, _ = self.decide(plan=plan, runs_=runs_, jobs={**GOOD_JOBS, 5: [job(51, 5, "c", "skipped")]},
                                     settle_during_read={5: {"run_attempt": 2, "conclusion": "failure"}})
        self.assertFalse(passed)
        self.assertIn(f"Pending: {cond}: run 5 attempt 2 listed jobs of another attempt; re-run merge-hold", out)
        self.assertIn("2 applicable gate(s) passed; 1 filtered out; 1 pending; 0 failed.", out)

    COND = ".github/workflows/cond.yml"

    def cond_plan(self):
        """A condition-false rwa workflow: it holds no expected gate."""
        return plan_fixture(excluded=plan_fixture()["excluded"] + [
            {"id": "cond#c", "class": "rwa", "workflow": self.COND, "job": "c", "check_name": "c",
             "reason": "condition-false"}])

    def assert_unexplained_conclusion(self, passed, out):
        """D2 round 5: a failed run whose listed jobs show no blocking conclusion is not judged."""
        self.assertFalse(passed)
        self.assertIn(f"Pending: {LEAVES}: run 2 attempt 2 concluded failure but lists no such job; "
                      "re-run merge-hold; if it persists, re-run that workflow run", out)
        self.assertIn("1 pending; 0 failed.", out)
        # The footer says a Pending-only failure needs no fix; this line may need a workflow re-run.
        self.assertIn('::error::Exception: a Pending line that says "if it persists" may need a re-run of that '
                      "workflow run when a merge-hold re-run does not clear it.", out)

    def assert_run_without_any_job(self, passed, out, workflow, run_id, attempt, conclusion="failure"):
        self.assertFalse(passed)
        self.assertIn(f"Failed: {workflow}: run {run_id} attempt {attempt} concluded {conclusion}, but GitHub "
                      "listed no job for it; re-run merge-hold if the jobs listing lagged, or re-run that workflow "
                      "run (or push a new head) if it ended before any job started", out)

    def test_s1_a_settled_failure_whose_jobs_re_read_omits_the_failed_job_is_pending(self):
        """S1: attempt 1 lists the viewer gate green and an unregistered late gate failed. A partial
        re-run of the late gate fails again as attempt 2, but the jobs re-read lists only the
        carried-over viewer copy, so nothing listed explains attempt 2's failure."""
        passed, out, _ = self.decide(
            runs_=[wf_run(1, CI), wf_run(2, LEAVES, conclusion="failure")],
            jobs={**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)"), job(22, 2, "late-gate", "failure")]},
            settle_during_read={2: {"run_attempt": 2, "conclusion": "failure"}},
            later_jobs={2: [job(24, 2, "quality (viewer)", attempt=2)]})
        self.assert_unexplained_conclusion(passed, out)

    def test_s1u_an_unchanged_failure_whose_jobs_omit_the_failed_job_is_pending(self):
        """S1u: listing and re-read agree on a failed attempt 2 whose jobs list only the green copy."""
        passed, out, _ = self.decide(runs_=[wf_run(1, CI), wf_run(2, LEAVES, conclusion="failure", attempt=2)],
                                     jobs={**GOOD_JOBS, 2: [job(24, 2, "quality (viewer)", attempt=2)]})
        self.assert_unexplained_conclusion(passed, out)

    def test_s1c_a_listed_failed_job_still_fails_the_settled_run(self):
        """S1c control: a re-read that lists the failed late gate keeps the existing Failed line."""
        passed, out, _ = self.decide(
            runs_=[wf_run(1, CI), wf_run(2, LEAVES, conclusion="failure")],
            jobs={**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)"), job(22, 2, "late-gate", "failure")]},
            settle_during_read={2: {"run_attempt": 2, "conclusion": "failure"}},
            later_jobs={2: [job(24, 2, "quality (viewer)", attempt=2), job(25, 2, "late-gate", "failure", attempt=2)]})
        self.assertFalse(passed)
        self.assertIn(f"Failed: late-gate: failure in run 2 attempt 2 ({LEAVES}); not expected by the plan", out)
        self.assertIn("0 pending; 1 failed.", out)
        self.assertNotIn("lists no such job", out)

    def test_an_advisory_failure_explains_a_failed_run(self):
        """Control: only the advisory job failed, which the run conclusion reflects; the gate still
        passes, on the unchanged path and on the settle path."""
        def advice_failed(attempt):
            return {1: [job(11, 1, "host-quality", attempt=attempt), job(12, 1, "advice", "failure", attempt=attempt)]}
        cases = {"unchanged": ([wf_run(1, CI, conclusion="failure"), wf_run(2, LEAVES)],
                               {**GOOD_JOBS, **advice_failed(1)}, None, None, 1),
                 "settled": (GOOD_RUNS, GOOD_JOBS, advice_failed(2), {1: {"run_attempt": 2, "conclusion": "failure"}}, 2)}
        for name, (runs_, jobs, later, settle, attempt) in cases.items():
            with self.subTest(name):
                passed, out, _ = self.decide(runs_=runs_, jobs=jobs, later_jobs=later, settle_during_read=settle)
                self.assertTrue(passed, out)
                self.assertIn(f"Passed: host-quality: success in run 1 attempt {attempt}", out)
                self.assertNotIn("lists no such job", out)

    def test_g4e_a_condition_false_run_that_settles_as_failed_without_jobs_fails(self):
        """G4e: a condition-false workflow goes from a skipped attempt 1 to a failed attempt 2 whose
        jobs re-read is empty; no gate of that attempt ran, so it never passes."""
        passed, out, _ = self.decide(
            plan=self.cond_plan(), runs_=GOOD_RUNS + [wf_run(5, self.COND, conclusion="skipped")],
            jobs={**GOOD_JOBS, 5: [job(51, 5, "c", "skipped")]},
            settle_during_read={5: {"run_attempt": 2, "conclusion": "failure"}}, later_jobs={5: []})
        self.assert_run_without_any_job(passed, out, self.COND, 5, 2)
        self.assertIn("2 applicable gate(s) passed; 1 filtered out; 0 pending; 1 failed.", out)

    def test_g4u_an_unchanged_condition_false_failure_without_jobs_fails(self):
        """G4u: listing and re-read agree on a failed attempt 2 with an empty jobs listing."""
        passed, out, _ = self.decide(plan=self.cond_plan(),
                                     runs_=GOOD_RUNS + [wf_run(5, self.COND, conclusion="failure", attempt=2)],
                                     jobs={**GOOD_JOBS, 5: []})
        self.assert_run_without_any_job(passed, out, self.COND, 5, 2)
        self.assertIn("2 applicable gate(s) passed; 1 filtered out; 0 pending; 1 failed.", out)

    def test_a_run_without_any_job_never_passes(self):
        """A run that ended blocking without any job (startup_failure, or cancelled before a job
        started) ran no gate: Failed, not Pending, because re-running merge-hold cannot change
        it. An expected gate keeps its "missing from" line; a skipped run without jobs passes."""
        for conclusion in ("startup_failure", "cancelled"):
            with self.subTest(conclusion, expected=True):
                passed, out, _ = self.decide(runs_=[wf_run(1, CI), wf_run(2, LEAVES, conclusion=conclusion)],
                                             jobs={**GOOD_JOBS, 2: []})
                self.assert_run_without_any_job(passed, out, LEAVES, 2, 1, conclusion)
                self.assertIn(f"Failed: quality (viewer): missing from run 2 attempt 1 ({LEAVES})", out)
                self.assertIn("1 applicable gate(s) passed; 1 filtered out; 0 pending; 2 failed.", out)
            with self.subTest(conclusion, expected=False):
                passed, out, _ = self.decide(plan=self.cond_plan(),
                                             runs_=GOOD_RUNS + [wf_run(5, self.COND, conclusion=conclusion)],
                                             jobs={**GOOD_JOBS, 5: []})
                self.assert_run_without_any_job(passed, out, self.COND, 5, 1, conclusion)
                self.assertIn("2 applicable gate(s) passed; 1 filtered out; 0 pending; 1 failed.", out)
        passed, out, _ = self.decide(plan=self.cond_plan(),
                                     runs_=GOOD_RUNS + [wf_run(5, self.COND, conclusion="skipped")],
                                     jobs={**GOOD_JOBS, 5: []})
        self.assertTrue(passed, out)

    def test_a_job_without_an_integer_attempt_holds_its_run(self):
        """A job that does not name its attempt as an integer cannot be bound to the judged run,
        also when the run names none either or both name it as the same string. A bool or a float
        equals an int in Python but is no integer attempt, on the job or on the run."""
        for listed, attempt in ((1, None), (None, None), ("1", "1"), (True, True), (1, True), (True, 1),
                                (1.0, 1), (1, 1.0)):
            with self.subTest(listed=listed, attempt=attempt):
                bare = job(21, 2, "quality (viewer)", attempt=attempt)
                if attempt is None:
                    del bare["run_attempt"]
                passed, out, _ = self.decide(runs_=[wf_run(1, CI), wf_run(2, LEAVES, attempt=listed)],
                                             jobs={**GOOD_JOBS, 2: [bare]})
                self.assert_jobs_of_another_attempt(passed, out, listed)

    def test_a_partial_re_run_whose_latest_jobs_carry_the_new_attempt_passes(self):
        """After "re-run failed jobs" GitHub copies the carried-over jobs into the new attempt, so
        every latest job names it (live check of 26 November 2026); such a re-run still passes."""
        carried = {1: [job(13, 1, "host-quality", attempt=2), job(14, 1, "advice", attempt=2)]}
        cases = {"unchanged": ([wf_run(1, CI, attempt=2), wf_run(2, LEAVES)], {**GOOD_JOBS, **carried}, None, None),
                 "settled": ([wf_run(1, CI, conclusion="failure"), wf_run(2, LEAVES)],
                             {**GOOD_JOBS, 1: [job(11, 1, "host-quality", "failure"), job(12, 1, "advice")]},
                             carried, {1: {"run_attempt": 2, "conclusion": "success"}})}
        for name, (runs_, jobs, later, settle) in cases.items():
            with self.subTest(name):
                passed, out, _ = self.decide(runs_=runs_, jobs=jobs, later_jobs=later, settle_during_read=settle)
                self.assertTrue(passed, out)
                self.assertIn("Passed: host-quality: success in run 1 attempt 2", out)

    def test_one_job_of_another_attempt_holds_the_run_even_beside_bound_gates(self):
        """Every listed job is bound, not only the first one or the expected gates: a listing that
        mixes the re-run gate of attempt 2 with a lagging job of attempt 1 is not attempt 2's."""
        mixed = {**GOOD_JOBS, 1: [job(13, 1, "host-quality", attempt=2), job(12, 1, "advice")]}
        passed, out, _ = self.decide(runs_=[wf_run(1, CI, attempt=2), wf_run(2, LEAVES)], jobs=mixed)
        self.assertFalse(passed)
        self.assertIn(f"Pending: {CI}: run 1 attempt 2 listed jobs of another attempt; re-run merge-hold", out)
        self.assertIn("1 applicable gate(s) passed; 1 filtered out; 1 pending; 0 failed.", out)
        self.assertNotIn("Passed: host-quality", out)

    def test_a_new_running_attempt_is_not_failed_on_the_old_attempts_failed_job(self):
        """P2: a new attempt runs while `filter=latest` still lists the old attempt's failed job;
        that failure is no longer the run's outcome, so the run stays pending, not failed."""
        old = {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", "failure")]}
        cases = {"listed": ([wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None, attempt=2)], None),
                 "settled": ([wf_run(1, CI), wf_run(2, LEAVES, conclusion="failure")],
                             {2: {"run_attempt": 2, "status": "in_progress", "conclusion": None}})}
        for name, (runs_, settle) in cases.items():
            with self.subTest(name):
                passed, out, _ = self.decide(runs_=runs_, jobs=old, settle_during_read=settle)
                self.assert_jobs_of_another_attempt(passed, out, 2)
                self.assertNotIn("Failed:", out)

    def test_rendered_runtime_matrix_names_inherit_their_template_class(self):
        """CodeRabbit on #504: a dynamic advisory job must not count as an unregistered gate."""
        plan = plan_fixture(excluded=plan_fixture()["excluded"] + [
            {"id": "ci#probe/*", "class": "advisory", "workflow": CI, "job": "probe",
             "check_name": "probe (${{ matrix.case }})", "reason": "advisory"}])
        jobs = {**GOOD_JOBS, 1: [job(11, 1, "host-quality"), job(14, 1, "probe (a)", "failure")]}
        runs_ = [wf_run(1, CI, conclusion="failure"), wf_run(2, LEAVES)]
        self.assertTrue(self.decide(plan=plan, runs_=runs_, jobs=jobs)[0])

    def test_runs_without_any_time_cannot_be_ordered(self):
        runs_ = [wf_run(1, CI), dict(wf_run(2, LEAVES), created_at=None)]
        with self.assertRaises(ValueError):
            self.decide(runs_=runs_)

    def test_runs_of_another_pull_request_or_base_do_not_count(self):
        """Review N1: a listed pull request must be this one, targeting this base."""
        own = [{"number": 7, "base": {"ref": "main"}}]
        # A same-repository head always lists its pull request, so an empty list does not count.
        for pulls, passes in (([], False), (own, True), ([{"number": 8, "base": {"ref": "main"}}], False),
                              ([{"number": 7, "base": {"ref": "release"}}], False)):
            with self.subTest(pulls=pulls):
                runs_ = [dict(wf_run(1, CI), pull_requests=own), dict(wf_run(2, LEAVES), pull_requests=pulls)]
                self.assertIs(self.decide(runs_=runs_, pull=dict(PULL, number=7))[0], passes)
        # A fork head's runs list no pull request, so they cannot be bound to a base: fail closed.
        fork = dict(PULL, head={"sha": HEAD, "ref": "topic", "repo": {"full_name": "fork/r"}})
        runs_ = [dict(wf_run(1, CI, head_repository={"full_name": "fork/r"}), pull_requests=[]),
                 dict(wf_run(2, LEAVES, head_repository={"full_name": "fork/r"}), pull_requests=[])]
        passed, out, _ = self.decide(runs_=runs_, pull=fork)
        self.assertFalse(passed)
        self.assertIn("no .github/workflows/leaves.yml run", out)

    def test_a_later_rerun_of_an_older_run_decides(self):
        """Review N3: newest by the start of the latest attempt, not by id alone."""
        runs_ = [wf_run(1, CI), wf_run(2, LEAVES, conclusion="failure", attempt=2, run_started_at="2026-11-24T22:00:00Z"),
                 wf_run(3, LEAVES, run_started_at="2026-11-24T21:00:00Z")]
        jobs = {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", "failure", attempt=2)],
                3: [job(31, 3, "quality (viewer)")]}
        passed, out, _ = self.decide(runs_=runs_, jobs=jobs)
        self.assertFalse(passed)
        self.assertIn("quality (viewer): failure in run 2 attempt 2", out)

    def test_runtime_matrix_rows_match_their_template(self):
        dynamic = row("smp#margin/*", LEAVES, "DIAGNOSTIC margins (${{ matrix.case }})", variant="*")
        plan = plan_fixture(selected=plan_fixture()["selected"][:2] + [dynamic])
        jobs = {**GOOD_JOBS, 2: [job(21, 2, "DIAGNOSTIC margins (a)"), job(22, 2, "DIAGNOSTIC margins (b)")]}
        self.assertTrue(self.decide(plan=plan, jobs=jobs)[0])
        jobs = {**GOOD_JOBS, 2: [job(21, 2, "DIAGNOSTIC margins (a)"),
                                 job(22, 2, "DIAGNOSTIC margins (b)", "failure")]}
        self.assertFalse(self.decide(plan=plan, jobs=jobs)[0])
        self.assertFalse(self.decide(plan=plan, jobs={**GOOD_JOBS, 2: []})[0])
        running = [wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None)]
        jobs = {**GOOD_JOBS, 2: [job(21, 2, "DIAGNOSTIC margins (a)"), job(22, 2, "DIAGNOSTIC margins (b)")]}
        passed, out, _ = self.decide(plan=plan, runs_=running, jobs=jobs)
        self.assertFalse(passed)  # more variants may still be created
        self.assertNotIn("Passed: DIAGNOSTIC", out)
        # Pending, not Failed: the variants so far succeeded (#494 D1 step 2, an early merge_group exit).
        self.assertIn("::error::Pending: DIAGNOSTIC margins (${{ matrix.case }}): 2 variant(s) succeeded so far "
                      "in run 2 attempt 1, which is in_progress", out)
        self.assertNotIn("Failed: DIAGNOSTIC", out)


class RecomputeTests(TestCase):
    """The base copy of the selector runs isolated, with the target branch, next to merge_hold.py."""

    def test_selector_next_to_merge_hold_runs_isolated_with_every_input(self):
        with tempfile.TemporaryDirectory() as tmp:
            Path(tmp, "ci_impact.py").write_text(
                "import json, sys\n"
                "out = [a.split('=', 1)[1] for a in sys.argv if a.startswith('--out=')][0]\n"
                "json.dump({'argv': sys.argv[1:], 'isolated': sys.flags.isolated}, open(out, 'w'))\n")
            with mock.patch.object(merge_hold, "__file__", str(Path(tmp, "merge_hold.py"))):
                plan = merge_hold.recompute_plan(BASE, HEAD, MERGE, "main", repo_dir=tmp)
                queue = merge_hold.recompute_plan(BASE, GROUP, GROUP, "main", repo_dir=tmp, event="merge_group")
        self.assertEqual(plan["isolated"], 1)
        self.assertEqual(plan["argv"][:7], ["plan", "--event=pull_request", f"--base-sha={BASE}",
                                            f"--head-sha={HEAD}", f"--tested-sha={MERGE}", "--base-ref=main",
                                            f"--repo={tmp}"])
        self.assertEqual(queue["argv"][:6], ["plan", "--event=merge_group", f"--base-sha={BASE}",
                                             f"--head-sha={GROUP}", f"--tested-sha={GROUP}", "--base-ref=main"])

    def test_missing_or_failing_selector_fails_closed(self):
        with tempfile.TemporaryDirectory() as tmp:
            with mock.patch.object(merge_hold, "__file__", str(Path(tmp, "merge_hold.py"))):
                with self.assertRaises(ValueError):
                    merge_hold.recompute_plan(BASE, HEAD, MERGE, "main")
                Path(tmp, "ci_impact.py").write_text("import sys\nsys.exit(2)\n")
                with self.assertRaises(ValueError):
                    merge_hold.recompute_plan(BASE, HEAD, MERGE, "main")

    def test_real_selector_of_the_same_generation_accepts_the_interface(self):
        import test_ci_impact as fixture  # noqa: PLC0415 - shared git fixture

        with tempfile.TemporaryDirectory() as tmp:
            repo = fixture.Repo(tmp)
            files = dict(fixture.BASE_FILES)
            files[".github/ci-impact.toml"] = fixture.CATALOGUE
            files["tools/ci_impact.py"] = fixture.SELECTOR.read_bytes()
            base = repo.commit("base", files)
            repo.git("checkout", "-q", "-b", "topic")
            head = repo.commit("leaf", {"apps/solo/src/lib.rs": "// solo 2\n"})
            repo.git("checkout", "-q", "--detach", base)
            repo.git("-c", "user.name=f", "-c", "user.email=f@example.invalid",
                     "merge", "-q", "--no-ff", "-m", "merge", "topic")
            tested = repo.git("rev-parse", "HEAD")
            with mock.patch.object(merge_hold, "__file__", str(fixture.SELECTOR.parent / "merge_hold.py")):
                plan = merge_hold.recompute_plan(base, head, tested, "main", repo_dir=tmp)
                # D1 step 2: the queue commit is both head and tested commit.
                queue = merge_hold.recompute_plan(base, tested, tested, "main", repo_dir=tmp, event="merge_group")
        merge_hold.check_plan(plan, base, head, tested, "main")
        self.assertEqual(plan["mode"], "select")
        self.assertEqual([row["id"] for row in merge_hold.expected_gates(plan)],
                         ["ci#host-quality", "leaves#quality/solo"])
        merge_hold.check_plan(queue, base, tested, tested, "main", event="merge_group")
        self.assertEqual((queue["mode"], queue["fallback"]["code"]), ("full", "merge-group"))
        self.assertEqual([row["id"] for row in merge_hold.expected_gates(queue)], ["ci#host-quality"])
        with self.assertRaises(ValueError):  # bound to its event
            merge_hold.check_plan(queue, base, tested, tested, "main")

    def test_real_selector_plans_a_stacked_group_from_the_trusted_base(self):
        """ADR-CI-MERGE-GROUP-SELECT-20261126 end to end: real git, the real selector, a select catalogue."""
        import test_ci_impact as fixture  # noqa: PLC0415 - shared git fixture

        with tempfile.TemporaryDirectory() as tmp:
            repo = fixture.Repo(tmp)
            files = dict(fixture.BASE_FILES)
            files[".github/ci-impact.toml"] = fixture.MERGE_GROUP_SELECT
            files["tools/ci_impact.py"] = fixture.SELECTOR.read_bytes()
            base = repo.commit("base", files)
            repo.git("update-ref", merge_hold.MAIN_REF, base)

            def stack(name, ahead_files):
                repo.git("checkout", "-q", "-B", name, base)
                return repo.commit("entry ahead", ahead_files), repo.commit("queue commit", {"docs/guide.md": "g 2\n"})

            native_ahead, native_group = stack("q1", {"kernel/main.rs": "fn main() { 2 }\n"})
            policy_ahead, policy_group = stack("q2", {"tools/ci_impact.py": fixture.SELECTOR.read_bytes() + b"# x\n"})
            repo.git("checkout", "-q", "--orphan", "unrelated")
            unrelated = repo.commit("unrelated")
            cwd = os.getcwd()
            os.chdir(tmp)
            try:
                with mock.patch.object(merge_hold, "__file__", str(fixture.SELECTOR.parent / "merge_hold.py")):
                    def plan(ahead, group, trusted=base):
                        with mock.patch.dict(os.environ, {"BASE_SHA": trusted, "TESTED_SHA": group}):
                            return merge_hold.merge_group_plan(ahead, group)

                    native = plan(native_ahead, native_group)
                    policy = plan(policy_ahead, policy_group)
                    for why, args in (("T not protected main's merge base", (native_ahead, native_group, native_ahead)),
                                      ("T not an ancestor of the merge group base", (unrelated, native_group)),
                                      ("merge group base not in the queue commit", (policy_ahead, native_group))):
                        with self.subTest(why=why), self.assertRaises(ValueError):
                            plan(*args)
                    repo.git("update-ref", merge_hold.MAIN_REF, unrelated)  # no merge base with main
                    with self.assertRaises(ValueError):
                        plan(native_ahead, native_group)
            finally:
                os.chdir(cwd)
        self.assertEqual((native["mode"], native["shas"]["base"], native["shas"]["merge_base"]), ("select", base, base))
        self.assertEqual([row["id"] for row in merge_hold.expected_gates(native)],
                         ["ci#host-quality", "net#broker", "net#native"])  # the native path ahead counts
        self.assertEqual((policy["mode"], policy["fallback"]["code"]), ("full", "always-full"))
        self.assertEqual(policy["catalogue"]["source_sha"], base)
        self.assertEqual([row["id"] for row in merge_hold.expected_gates(policy)],
                         ["ci#host-quality", "net#broker", "net#native"])


if __name__ == "__main__":
    main()

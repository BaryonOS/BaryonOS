"""Unit tests for the merge-hold decision; no network access."""

import contextlib
import io
import os
import sys
import tempfile
from pathlib import Path
from unittest import TestCase, main, mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
sys.path.insert(0, str(Path(__file__).resolve().parent))
import merge_hold

HEAD = "a" * 40
OTHER = "b" * 40
ENV = {"GH_TOKEN": "x", "REPO": "o/r", "EVENT_NAME": "pull_request",
       "PR_NUMBER": "7", "HEAD_SHA": HEAD}
GROUP = "c" * 40
BASE = "d" * 40
GROUP_ENV = {"GH_TOKEN": "x", "REPO": "o/r", "EVENT_NAME": "merge_group",
             "PR_NUMBER": "", "HEAD_SHA": "",
             "MERGE_GROUP_HEAD_SHA": GROUP,
             "MERGE_GROUP_HEAD_REF": f"refs/heads/gh-readonly-queue/main/pr-8-{BASE}",
             "MERGE_GROUP_BASE_REF": "refs/heads/main"}


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


class MergeGroupTests(TestCase):
    """PR 7 is ahead of PR 8; group commit GROUP contains both."""

    def run_group(self, nodes, labels=None, comments=None, trees=None,
                  env=None, answer=None, checks=None, ahead=None, on_sleep=None,
                  commits=None, gate_runs=None):
        """checks answer polls of the group commit, commits[sha] those of sha, ahead the rest."""
        labels, comments, trees = labels or {}, comments or {}, trees or {}
        polls = {sha: list(answers) for sha, answers in (commits or {}).items()}
        polls[GROUP], polls[None] = list(checks or [DONE]), list(ahead or [DONE])
        calls = []

        def fetch(token, url):
            calls.append(url)
            number = int(url.split("/")[-2])
            if url.endswith("/labels"):
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
            if "/actions/runs?" in url:
                head = url.split("head_sha=")[1].split("&")[0]
                listed = (gate_runs or {}).get(head, [gate_run(head)])
                return {"total_count": len(listed), "workflow_runs": listed}
            if url.endswith("/graphql"):
                self.assertEqual(payload["variables"], {"owner": "o", "name": "r"})
                return answer if answer is not None else {"data": {"repository": {
                    "mergeQueue": {"entries": {"pageInfo": {"hasNextPage": False},
                                               "nodes": nodes}}}}}
            sha = url.rsplit("/", 1)[1]
            return {"tree": {"sha": trees.get(sha, "1" * 40)}}

        clock = [0.0]

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
        code, out, _ = self.run_group(one, trees={GROUP: "1" * 40, OTHER: "2" * 40})
        self.assertEqual(code, 1)
        self.assertIn("does not yet judge the gates on merge_group", out)
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
        # Phase 3: PR 8's gates ran on its own head. If its entry lands another tree
        # (PR 7's change on top), no gate verdict that merge-hold judges covers that tree.
        code, out, _ = self.run_group(
            queue,
            labels={7: [merge_hold.LABEL]},
            comments={7: [comment("acceptance-released: " + HEAD)]},
            trees={"e" * 40: "1" * 40, HEAD: "1" * 40, GROUP: "2" * 40, OTHER: "3" * 40})
        self.assertEqual(code, 1, out)
        self.assertIn("#8: ::error::The applicable gates ran on", out)

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
        # Past MAX_PAGES full pages the read fails and the group fails closed at the deadline.
        code, out, calls = self.run_group(self.QUEUE, checks=[lambda url: filler])
        self.assertEqual(code, 1, out)
        self.assertIn("Too many pages", out)
        self.assertEqual(len([url for url in calls if f"/commits/{GROUP}/check-runs?" in url
                              and url.endswith("&page=1")]), calls.count("sleep") + 1)

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
        polls = merge_hold.WAIT_SECONDS // merge_hold.POLL_SECONDS

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
        polls = merge_hold.WAIT_SECONDS // merge_hold.POLL_SECONDS
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


def job(job_id, run_id, name, conclusion="success", status="completed"):
    return {"id": job_id, "run_id": run_id, "name": name, "status": status, "conclusion": conclusion}


GOOD_RUNS = [wf_run(1, CI), wf_run(2, LEAVES)]
GOOD_JOBS = {1: [job(11, 1, "host-quality"), job(12, 1, "advice")], 2: [job(21, 2, "quality (viewer)")]}


class GateTests(TestCase):
    """Phase 3: the pull_request path verifies every applicable gate from base-side code."""

    def decide(self, plan=None, runs_=None, jobs=None, pull=None, ancestor=0, page_size=100, totals=None,
               polls=None, rerun_during_read=None):
        plan = plan or plan_fixture()
        state = {"poll": 0, "calls": [], "sleeps": 0}
        clock = [0.0]

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
                return {**current, **(rerun_during_read or {}).get(run_id, {})}
            page = int(url.rsplit("page=", 1)[1])
            if "/jobs?" in url:
                listed = (jobs or GOOD_JOBS).get(int(url.split("/runs/")[1].split("/")[0]), [])
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
        jobs = {**GOOD_JOBS, 2: [job(24, 2, "quality (viewer)")]}
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
        """CodeRabbit on #517: after a poll the next stale report still gets one refresh."""
        plan = plan_fixture(budgets={"pull_request_wait_minutes": 40, "merge_group_wait_minutes": 45})
        running = [wf_run(1, CI), wf_run(2, LEAVES, "in_progress", None)]
        done = [wf_run(1, CI), wf_run(2, LEAVES, conclusion="success", attempt=2)]
        jobs = {**GOOD_JOBS, 2: [job(24, 2, "quality (viewer)")]}
        reads = {"n": 0}
        real = merge_hold.fetch_listing

        def listing(token, url, key):
            if key == "workflow_runs":
                reads["n"] += 1
                if reads["n"] == 4:  # the report after the poll (read 3) still sees attempt 1 running
                    return running
            return real(token, url, key)

        with mock.patch.object(merge_hold, "fetch_listing", listing):
            passed, out, state = self.decide(plan=plan, polls=[running, done], jobs=jobs)
        self.assertTrue(passed, out)
        self.assertEqual(state["sleeps"], 1)

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
        jobs = {**GOOD_JOBS, 2: [job(21, 2, "quality (viewer)", "failure")], 3: [job(31, 3, "quality (viewer)")]}
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
        self.assertEqual(plan["isolated"], 1)
        self.assertEqual(plan["argv"][:7], ["plan", "--event=pull_request", f"--base-sha={BASE}",
                                            f"--head-sha={HEAD}", f"--tested-sha={MERGE}", "--base-ref=main",
                                            f"--repo={tmp}"])

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
        merge_hold.check_plan(plan, base, head, tested, "main")
        self.assertEqual(plan["mode"], "select")
        self.assertEqual([row["id"] for row in merge_hold.expected_gates(plan)],
                         ["ci#host-quality", "leaves#quality/solo"])


if __name__ == "__main__":
    main()

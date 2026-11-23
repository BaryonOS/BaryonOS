"""Unit tests for the merge-hold decision; no network access."""

import contextlib
import io
import os
import sys
from pathlib import Path
from unittest import TestCase, main, mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
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
    def run_main(self, env, fetch):
        out = io.StringIO()
        with mock.patch.dict(os.environ, env, clear=True), \
                mock.patch.object(merge_hold, "fetch_all", fetch), \
                contextlib.redirect_stdout(out):
            return merge_hold.main(), out.getvalue()

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
    return {"check_runs": [{"name": name, "status": state}
                           for name, state in zip(merge_hold.AHEAD_CHECKS, states)]}


DONE = runs("completed", "completed", "completed")
E = "e" * 40


class MergeGroupTests(TestCase):
    """PR 7 is ahead of PR 8; group commit GROUP contains both."""

    def run_group(self, nodes, labels=None, comments=None, trees=None,
                  env=None, answer=None, checks=None, ahead=None, on_sleep=None):
        """checks answer polls of the group commit, ahead those of any other commit."""
        labels, comments, trees = labels or {}, comments or {}, trees or {}
        polls = {True: list(checks or [DONE]), False: list(ahead or [DONE])}
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
                queue = polls[f"/commits/{GROUP}/" in url]
                answer_ = queue.pop(0) if len(queue) > 1 else queue[0]
                if isinstance(answer_, Exception):
                    raise answer_
                return answer_
            if url.endswith("/graphql"):
                self.assertEqual(payload["variables"], {"owner": "o", "name": "r"})
                return answer if answer is not None else {"data": {"repository": {
                    "mergeQueue": {"entries": {"pageInfo": {"hasNextPage": False},
                                               "nodes": nodes}}}}}
            return {"tree": {"sha": trees.get(url.rsplit("/", 1)[1], "tree-" + url[-6:])}}

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

    def test_released_pr_with_the_landing_tree_passes(self):
        code, out, _ = self.run_group(
            [entry(1, 8, GROUP, OTHER)],
            labels={8: [merge_hold.LABEL]},
            comments={8: [comment("acceptance-released: " + OTHER)]},
            trees={GROUP: "t1", OTHER: "t1"})
        self.assertEqual(code, 0, out)

    def test_unreleased_pending_pr_ahead_in_the_group_fails(self):
        code, out, _ = self.run_group(self.QUEUE, labels={7: [merge_hold.LABEL]})
        self.assertEqual(code, 1)
        self.assertIn("#7: ::error::", out)

    def test_release_does_not_cover_a_different_landing_tree(self):
        code, out, _ = self.run_group(
            self.QUEUE,
            labels={8: [merge_hold.LABEL]},
            comments={8: [comment("acceptance-released: " + OTHER)]},
            trees={GROUP: "combined", OTHER: "pr-only"})
        self.assertEqual(code, 1)
        self.assertIn("#8: ::error::", out)

    def test_released_pr_ahead_does_not_block_an_unheld_pr_behind_it(self):
        # PR 7's own entry commit (group 1) has PR 7's tree; group 2 adds PR 8.
        queue = [entry(1, 7, "e" * 40, HEAD), entry(2, 8, GROUP, OTHER)]
        code, out, _ = self.run_group(
            queue,
            labels={7: [merge_hold.LABEL]},
            comments={7: [comment("acceptance-released: " + HEAD)]},
            trees={"e" * 40: "t7", HEAD: "t7", GROUP: "t7+8"})
        self.assertEqual(code, 0, out)

    def test_released_pr_ahead_whose_own_entry_lands_another_tree_fails(self):
        queue = [entry(1, 7, "e" * 40, HEAD), entry(2, 8, GROUP, OTHER)]
        code, out, _ = self.run_group(
            queue,
            labels={7: [merge_hold.LABEL]},
            comments={7: [comment("acceptance-released: " + HEAD)]},
            trees={"e" * 40: "main+7", HEAD: "t7", GROUP: "t7+8"})
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


if __name__ == "__main__":
    main()

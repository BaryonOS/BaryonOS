"""Unit tests for conservative GitHub Projects classification (no network)."""

import contextlib
import io
import os
import sys
import unittest
from pathlib import Path
import unittest.mock as mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import project_classification as pc  # noqa: E402


class ClassificationTests(unittest.TestCase):
    def test_trusted_prefixes(self):
        self.assertEqual(
            pc.classify("feat(veld): integrate workspace", []),
            {"Work Type": "Feature", "Subsystem": "UI"},
        )
        self.assertEqual(
            pc.classify("docs: update README", []),
            {"Work Type": "Documentation", "Subsystem": "Documentation"},
        )
        self.assertEqual(
            pc.classify("fix(memory): reject invalid mapping", []),
            {"Work Type": "Bug", "Subsystem": "Memory"},
        )

    def test_priority_effort_and_dates_remain_specialist_owned(self):
        for priority in ("p0", "p1", "p2", "p3", "urgent", "high", "medium", "low"):
            with self.subTest(priority=priority):
                self.assertEqual(
                    pc.classify("fix(kernel): critical issue", [f"priority:{priority}"]),
                    {"Work Type": "Bug", "Subsystem": "Kernel"},
                )
        suggestions = pc.classify("feat(kernel): complex rewrite", [])
        for field in ("Priority", "Effort", "Start date", "Target date"):
            self.assertNotIn(field, suggestions)

    def test_legacy_priority_project_field_is_not_written(self):
        entry = item(issue(labels=["priority:p0"]))
        fields = {"Priority": {"id": "fp", "options": [
            {"id": "p0", "name": "P0 Critical"},
            {"id": "urgent", "name": "Urgent"},
        ]}}
        self.assertEqual(pc.proposed_updates(entry, fields), [])

    def test_hardware_and_networking_require_exact_scopes(self):
        for expected, scopes in (
            ("Hardware", ("hw", "hardware", "drivers")),
            ("Networking", ("net", "network", "networking")),
        ):
            for scope in scopes:
                with self.subTest(scope=scope):
                    self.assertEqual(pc.classify(f"fix({scope}): work", []),
                                     {"Work Type": "Bug", "Subsystem": expected})
                    self.assertEqual(pc.classify("Untyped issue", [f"area:{scope}"]),
                                     {"Subsystem": expected})
        for scope in ("hw-ish", "drivers-extra", "networking-extra", "internet"):
            with self.subTest(scope=scope):
                self.assertNotIn("Subsystem", pc.classify(f"feat({scope}): work", []))
        self.assertNotIn("Subsystem", pc.classify("fix(hw): work", ["area:net"]))

    def test_conflicting_signals_are_not_guessed(self):
        self.assertNotIn(
            "Subsystem",
            pc.classify("feat(kernel): work", ["subsystem:storage"]),
        )
        self.assertNotIn(
            "Work Type",
            pc.classify("feat(ui): work", ["type:bug"]),
        )

    def test_do_not_overwrite_existing_values(self):
        item = {
            "content": {"title": "feat(veld): create shell", "number": 42,
                        "labels": {"nodes": []}},
            "fieldValues": {"nodes": [
                {"name": "Bug", "field": {"name": "Work Type"}}
            ]},
        }
        fields = {
            "Work Type": {"id": "f1", "options": [{"id": "o1", "name": "Feature"}]},
            "Subsystem": {"id": "f2", "options": [{"id": "o2", "name": "UI"}]},
        }
        self.assertEqual(
            pc.proposed_updates(item, fields),
            [("Subsystem", "UI", "f2", "o2")],
        )

    def test_missing_field_or_option_remains_unmodified(self):
        item = {
            "content": {"title": "feat(kernel): work", "number": 7,
                        "labels": {"nodes": []}},
            "fieldValues": {"nodes": []},
        }
        self.assertEqual(pc.proposed_updates(item, {}), [])

    def test_subsystem_updates_require_a_blank_field_and_existing_option(self):
        for scope, expected in (("hw", "Hardware"), ("net", "Networking")):
            with self.subTest(scope=scope):
                content = issue()
                content["title"] = f"feat({scope}): work"
                entry = item(content)
                field = {"id": "fsub", "options": [{"id": "osub", "name": expected}]}
                self.assertEqual(pc.proposed_updates(entry, {"Subsystem": field}),
                                 [("Subsystem", expected, "fsub", "osub")])
                self.assertEqual(pc.proposed_updates(entry, {"Subsystem": {
                    "id": "fsub", "options": [{"id": "old", "name": "Kernel"}],
                }}), [])
                entry["fieldValues"]["nodes"].append(
                    {"name": "Kernel", "field": {"name": "Subsystem"}})
                self.assertEqual(pc.proposed_updates(entry, {"Subsystem": field}), [])


STATUS = {"id": "fs", "options": [
    {"id": "s-back", "name": "Backlog"},
    {"id": "s-ready", "name": "Ready"},
    {"id": "s-prog", "name": "In Progress"},
    {"id": "s-rev", "name": "In Review"},
    {"id": "s-block", "name": "Blocked"},
    {"id": "s-done", "name": "Done"},
    {"id": "s-closed", "name": "Closed"},
]}


def pr(state="OPEN", draft=False, labels=(), number=1):
    return {"__typename": "PullRequest", "id": f"PR{number}", "number": number,
            "title": "chore: work", "state": state, "isDraft": draft,
            "repository": {"nameWithOwner": "BaryonOS/BaryonOS"},
            "labels": {"nodes": [{"name": n} for n in labels]}}


def issue(state="OPEN", reason=None, labels=(), number=2):
    return {"__typename": "Issue", "id": f"I{number}", "number": number,
            "title": "Untyped issue", "state": state, "stateReason": reason,
            "repository": {"nameWithOwner": "BaryonOS/BaryonOS"},
            "labels": {"nodes": [{"name": n} for n in labels]}}


def item(content, status=None, item_id="item"):
    values = [{"name": status, "field": {"name": "Status"}}] if status else []
    return {"id": item_id, "content": content, "fieldValues": {"nodes": values}}


def status_update(entry, fields=None):
    updates = pc.proposed_updates(entry, {"Status": STATUS} if fields is None else fields)
    return [u[1] for u in updates if u[0] == "Status"]


class StatusDerivationTests(unittest.TestCase):
    def test_empty_status_is_initialised_from_observed_facts(self):
        self.assertEqual(status_update(item(pr(draft=True))), ["In Progress"])
        self.assertEqual(status_update(item(pr())), ["In Review"])
        self.assertEqual(status_update(item(issue())), ["Backlog"])
        self.assertEqual(status_update(item(pr(labels=["blocked"]))),
                         ["Blocked"])
        self.assertEqual(status_update(item(issue(labels=["status:blocked"]))),
                         ["Blocked"])

    def test_nonterminal_statuses_are_preserved_across_draft_and_label_changes(self):
        for status in ("Backlog", "Todo", "Ready", "In Progress", "In Review",
                       "Blocked", "Parked"):
            for labels in ([], ["blocked"], ["status:blocked"]):
                for content in (pr(draft=True, labels=labels), pr(labels=labels),
                                issue(labels=labels)):
                    with self.subTest(status=status, content=content):
                        self.assertEqual(status_update(item(content, status)), [])

    def test_confirmed_completion_overrides_every_previous_status(self):
        for status in (None, "Backlog", "Ready", "In Progress", "In Review",
                       "Blocked", "Parked", "Closed"):
            for content in (pr("MERGED", labels=["blocked"]),
                            issue("CLOSED", "COMPLETED", labels=["blocked"])):
                with self.subTest(status=status, content=content):
                    self.assertEqual(status_update(item(content, status)), ["Done"])

    def test_other_closures_never_claim_completion(self):
        missing_reason = issue("CLOSED")
        del missing_reason["stateReason"]
        contents = [pr("CLOSED"), missing_reason]
        contents += [issue("CLOSED", reason) for reason in
                     (None, "", "NOT_PLANNED", "DUPLICATE", "UNKNOWN")]
        for status in (None, "Backlog", "Ready", "In Progress", "In Review",
                       "Blocked", "Parked", "Done"):
            for content in contents:
                with self.subTest(status=status, content=content):
                    self.assertEqual(status_update(item(content, status)), ["Closed"])

    def test_reopened_items_reset_only_terminal_statuses(self):
        for status in ("Done", "Closed", "done", "closed"):
            for content, expected in ((pr(), "In Review"),
                                      (pr(draft=True), "In Progress"),
                                      (issue(), "Backlog"),
                                      (pr(labels=["blocked"]), "Blocked"),
                                      (issue(labels=["status:blocked"]), "Blocked")):
                with self.subTest(status=status, content=content):
                    self.assertEqual(status_update(item(content, status)), [expected])

    def test_no_change_when_status_already_matches(self):
        self.assertEqual(status_update(item(pr("MERGED"), "Done")), [])
        self.assertEqual(status_update(item(issue("CLOSED", "COMPLETED"), "Done")), [])
        self.assertEqual(status_update(item(pr("CLOSED"), "Closed")), [])
        self.assertEqual(status_update(item(issue("CLOSED"), "closed")), [])
        self.assertEqual(status_update(item(pr(), "in review")), [])

    def test_missing_option_falls_back_or_skips(self):
        default = {"id": "fs", "options": [
            {"id": "a", "name": "Todo"}, {"id": "b", "name": "In Progress"},
            {"id": "c", "name": "Done"}]}
        self.assertEqual(status_update(item(pr()), {"Status": default}), ["In Progress"])
        self.assertEqual(status_update(item(pr(labels=["blocked"])), {"Status": default}),
                         [])
        self.assertEqual(status_update(item(issue()), {"Status": default}), ["Todo"])
        self.assertEqual(status_update(item(pr("CLOSED"), "Done"), {"Status": default}),
                         [])
        self.assertEqual(status_update(item(issue("CLOSED"), "Done"), {"Status": default}),
                         [])
        self.assertEqual(status_update(item(pr()), {}), [])

    def test_unknown_content_is_ignored(self):
        self.assertIsNone(pc.derive_status({"__typename": "DraftIssue"}, [], ""))
        self.assertEqual(pc.proposed_updates({"content": None}, {"Status": STATUS}), [])
        self.assertEqual(status_update(item(pr(state="UNKNOWN"), "Done")), [])
        self.assertEqual(status_update(item(issue(state=None), "Done")), [])


class FakeGitHub:
    """In-memory Projects V2 endpoint that counts mutations."""

    def __init__(self, items, open_issues, open_prs, page_size=2, project=True):
        self.items, self.page_size, self.project = list(items), page_size, project
        self.open = {"issues": open_issues, "pullRequests": open_prs}
        self.mutations = []

    def __call__(self, token, query, variables):
        if query is pc.PROJECT_QUERY:
            if not self.project:
                return {"organization": {"projectV2": None}}
            start = int(variables["cursor"] or 0)
            page = self.items[start:start + self.page_size]
            more = start + self.page_size < len(self.items)
            return {"organization": {"projectV2": {
                "id": "P",
                "fields": {"nodes": [{"name": "Status", **STATUS}, {}]},
                "items": {"nodes": page, "pageInfo": {
                    "hasNextPage": more,
                    "endCursor": str(start + self.page_size) if more else None}},
            }}}
        if query is pc.UPDATE_QUERY:
            self.mutations.append(("update", variables["item"], variables["option"]))
            for entry in self.items:
                if entry["id"] == variables["item"]:
                    name = next(o["name"] for o in STATUS["options"]
                                if o["id"] == variables["option"])
                    entry["fieldValues"]["nodes"] = [
                        {"name": name, "field": {"name": "Status"}}]
            return {}
        if query is pc.ADD_QUERY:
            self.mutations.append(("add", variables["content"]))
            if not any((e["content"] or {}).get("id") == variables["content"]
                       for e in self.items):
                content = next(n for group in self.open.values() for n in group
                               if n["id"] == variables["content"])
                self.items.append(item(content, item_id="new-" + content["id"]))
            return {}
        connection = "issues" if "issues(" in query else "pullRequests"
        return {"repository": {connection: {
            "nodes": self.open[connection],
            "pageInfo": {"hasNextPage": False, "endCursor": None}}}}


def run_quietly(fake, **kwargs):
    out = io.StringIO()
    with mock.patch.object(pc, "graphql", fake), contextlib.redirect_stdout(out):
        pc.run("token", "BaryonOS", "BaryonOS/BaryonOS", 5,
               kwargs.pop("limit", 500), **kwargs)
    return out.getvalue()


class SynchronisationTests(unittest.TestCase):
    def fixture(self):
        return FakeGitHub(
            items=[item(pr("MERGED", number=1), "In Review", "i1"),
                   item(issue(number=2), None, "i2"),
                   item(None, None, "redacted")],
            open_issues=[issue(number=2), issue(number=3)],
            open_prs=[pr(number=4)],
        )

    def test_dry_run_reports_without_mutating(self):
        fake = self.fixture()
        output = run_quietly(fake, apply=False)
        self.assertEqual(fake.mutations, [])
        self.assertIn("DRY-RUN #1: Status = Done", output)
        self.assertIn("DRY-RUN #3: add to project", output)
        self.assertIn("DRY-RUN #4: add to project", output)
        self.assertNotIn("#2: add", output)

    def test_apply_is_idempotent_across_repeated_runs(self):
        fake = self.fixture()
        run_quietly(fake, apply=True)
        self.assertEqual(sorted(m[1] for m in fake.mutations if m[0] == "add"),
                         ["I3", "PR4"])
        first = len(fake.mutations)
        run_quietly(fake, apply=True)  # adds statuses for the two new items
        run_quietly(fake, apply=True)
        settled = len(fake.mutations)
        run_quietly(fake, apply=True)
        self.assertEqual(len(fake.mutations), settled)
        self.assertGreater(settled, first)
        added = [e for e in fake.items if e["id"].startswith("new-")]
        self.assertEqual(len(added), 2)

    def test_recurring_sync_does_not_take_over_specialist_statuses(self):
        fake = FakeGitHub(
            items=[item(pr(), "In Progress", "i1"),
                   item(pr(draft=True), "In Review", "i2"),
                   item(issue(), "Blocked", "i3"),
                   item(issue(labels=["blocked"]), "Ready", "i4")],
            open_issues=[], open_prs=[],
        )
        run_quietly(fake, apply=True, add_missing=False)
        fake.items[0]["content"]["isDraft"] = True
        fake.items[1]["content"]["labels"]["nodes"] = [{"name": "blocked"}]
        run_quietly(fake, apply=True, add_missing=False)
        self.assertEqual(fake.mutations, [])

    def test_truncated_scan_never_adds(self):
        fake = self.fixture()
        output = run_quietly(fake, apply=True, limit=1)
        self.assertIn("truncated", output)
        self.assertFalse([m for m in fake.mutations if m[0] == "add"])

    def test_add_missing_can_be_disabled(self):
        fake = self.fixture()
        run_quietly(fake, apply=True, add_missing=False)
        self.assertFalse([m for m in fake.mutations if m[0] == "add"])

    def test_missing_project_access_fails_closed(self):
        fake = FakeGitHub([], [], [], project=False)
        with self.assertRaises(RuntimeError):
            run_quietly(fake, apply=True)
        self.assertEqual(fake.mutations, [])

    def test_missing_content_deduplicates(self):
        nodes = [{"id": "A", "number": 1}, {"id": "B", "number": 2},
                 {"id": "B", "number": 2}]
        self.assertEqual(pc.missing_content({"A"}, nodes), [{"id": "B", "number": 2}])


class TokenHandlingTests(unittest.TestCase):
    def call_main(self, *args):
        env = {k: v for k, v in os.environ.items() if k != "BARYON_PROJECTS_TOKEN"}
        out, err = io.StringIO(), io.StringIO()
        with mock.patch.dict(os.environ, env, clear=True), \
                mock.patch.object(sys, "argv", ["project_classification.py", *args]), \
                contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            try:
                return pc.main(), out.getvalue()
            except SystemExit as exc:
                return exc.code, err.getvalue()

    def test_scheduled_run_without_token_is_skipped_with_warning(self):
        code, output = self.call_main("--skip-without-token")
        self.assertEqual(code, 0)
        self.assertIn("::warning::", output)

    def test_manual_run_without_token_fails(self):
        code, output = self.call_main()
        self.assertEqual(code, 2)
        self.assertIn("BARYON_PROJECTS_TOKEN is required", output)

    def test_rejected_token_returns_failure(self):
        def denied(token, query, variables):
            raise RuntimeError("GitHub GraphQL HTTP 401")
        with mock.patch.dict(os.environ, {"BARYON_PROJECTS_TOKEN": "x"}), \
                mock.patch.object(pc, "graphql", denied), \
                mock.patch.object(sys, "argv", ["project_classification.py"]), \
                contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(pc.main(), 1)


if __name__ == "__main__":
    unittest.main()

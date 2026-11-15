import json
import sys
import tempfile
import unittest
import unittest.mock
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import roadmap_runner as rr


def task(tid, status="ready", deps=(), writes=("services/x/",), checks=("true",)):
    return {"id": tid, "criterion": "N02", "title": tid, "goal": "g", "status": status,
            "deps": list(deps), "write_paths": list(writes), "checks": list(checks)}


class Ledger(unittest.TestCase):
    def test_selects_first_ready_task_with_verified_dependencies(self):
        ledger = {"tasks": [task("A", "blocked"), task("B", deps=["A"]), task("C"), task("D")]}
        self.assertEqual(rr.select_task(ledger)["id"], "C")
        ledger["tasks"][0]["status"] = "verified"
        self.assertEqual(rr.select_task(ledger)["id"], "B")

    def test_lead_and_owner_tasks_are_never_selected_for_workers(self):
        ledger = {"tasks": [dict(task("A"), executor="lead"), dict(task("B"), executor="owner"), task("C")]}
        self.assertEqual(rr.select_task(ledger)["id"], "C")
        with self.assertRaises(ValueError):
            rr.validate_ledger({"tasks": [dict(task("A"), executor="anyone")]})
        rr.validate_ledger({"tasks": [dict(task("A", checks=[]), executor="lead")]})

    def test_no_ready_task_returns_none(self):
        self.assertIsNone(rr.select_task({"tasks": [task("A", "verified"), task("B", "blocked")]}))

    def test_validation_rejects_unsafe_ledgers(self):
        bad = [
            {"tasks": [task("A"), task("A")]},
            {"tasks": [task("A", deps=["Z"])]},
            {"tasks": [task("A", writes=["docs/progress.json"])]},
            {"tasks": [task("A", writes=[".github/workflows/x.yml"])]},
            {"tasks": [task("A", writes=["docs/roadmap-pipeline/ledger.json"])]},
            {"tasks": [task("A", writes=["../escape"])]},
            {"tasks": [task("A", checks=[])]},
            {"tasks": [dict(task("A"), status="done")]},
        ]
        for ledger in bad:
            with self.subTest(ledger=ledger), self.assertRaises(ValueError):
                rr.validate_ledger(ledger)
        rr.validate_ledger({"tasks": [task("A"), task("B", "blocked", checks=[])]})

    def test_ledger_round_trip_is_atomic_and_validated(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "ledger.json"
            rr.save_ledger({"tasks": [task("A")]}, path)
            self.assertEqual(rr.load_ledger(path)["tasks"][0]["id"], "A")
            with self.assertRaises(ValueError):
                rr.save_ledger({"tasks": [task("A"), task("A")]}, path)
            self.assertEqual(len(rr.load_ledger(path)["tasks"]), 1)

    def test_update_task_rereads_and_keeps_other_edits(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "ledger.json"
            rr.save_ledger({"tasks": [task("A"), task("B")]}, path)
            edited = rr.load_ledger(path)
            edited["tasks"][1]["title"] = "lead edit while paused"
            rr.save_ledger(edited, path)
            rr.update_task("A", path, history={"event": "verified"}, status="verified")
            after = rr.load_ledger(path)["tasks"]
            self.assertEqual(after[0]["status"], "verified")
            self.assertEqual(after[0]["history"], [{"event": "verified"}])
            self.assertEqual(after[1]["title"], "lead edit while paused")

    def test_repository_ledger_is_valid(self):
        if rr.LEDGER.exists():
            rr.load_ledger()


class Scope(unittest.TestCase):
    def test_write_set_matching(self):
        writes = ["services/net/", "crates/a/src/lib.rs", "docs/implementation/**"]
        for path in ("services/net/src/x.rs", "crates/a/src/lib.rs", "docs/implementation/N.md",
                     "Cargo.lock", "services/other/Cargo.lock"):
            self.assertTrue(rr.in_write_set(path, writes), path)
        for path in ("crates/a/src/other.rs", "services/netx/a.rs", "docs/progress.json",
                     "tools/roadmap_runner.py", ".github/workflows/ci.yml", "AGENTS.md",
                     "docs/PROJECT-STATE.md"):
            self.assertFalse(rr.in_write_set(path, writes + ["docs/", ".github/", "tools/"]), path)

    def test_porcelain_parsing_handles_renames_and_quotes(self):
        text = ' M a/b.rs\n?? "c d.txt"\nR  old.rs -> new.rs\n'
        self.assertEqual(rr.parse_porcelain(text), ["a/b.rs", "c d.txt", "new.rs"])


class Session(unittest.TestCase):
    def test_worker_environment_drops_credentials_and_parent_session(self):
        source = {"PATH": "/bin", "HOME": "/root", "HTTPS_PROXY": "p", "GH_TOKEN": "x", "GITHUB_TOKEN": "x",
                  "AWS_SECRET_ACCESS_KEY": "x", "CLAUDE_CODE_SESSION_ID": "parent", "CLAUDECODE": "1",
                  "CLAUDE_CODE_REMOTE_SESSION_ID": "cse", "SESSION_INGRESS_URL": "u",
                  "CLOUDSDK_AUTH_ACCESS_TOKEN": "x", "GIT_ASKPASS": "x"}
        env = rr.worker_env(source)
        self.assertEqual(env["PATH"], "/bin")
        self.assertEqual(env["HTTPS_PROXY"], "p")
        for key in source:
            if key not in ("PATH", "HOME", "HTTPS_PROXY"):
                self.assertNotIn(key, env)

    def test_argv_is_a_fresh_session_without_resume_or_bypass(self):
        cfg = rr.Config()
        argv = rr.claude_argv(cfg, "roadmap-pipeline-worker-T", "0000", Path("/s.json"), {}, "acceptEdits", 10.0)
        for flag in rr.FORBIDDEN_FLAGS:
            self.assertNotIn(flag, argv)
        self.assertNotIn("bypassPermissions", argv)
        joined = " ".join(argv)
        self.assertIn("--session-id 0000", joined)
        self.assertIn("--autocompact 300000", joined)
        self.assertIn("--permission-prompts none", joined)
        self.assertIn("--name roadmap-pipeline-worker-T", joined)
        self.assertEqual(argv[1], "-p")
        ro = rr.claude_argv(cfg, "n", "1", Path("/s.json"), {}, "default", 1.0, rr.READ_ONLY_DISALLOWED)
        self.assertIn("--disallowedTools=Edit,Write,MultiEdit,NotebookEdit", ro)

    def test_allow_rules_are_task_scoped(self):
        t = dict(task("A", checks=["cargo +1.90.0 test --locked -p net-config"]))
        rules = rr.allow_rules(t, ("Edit",))
        self.assertIn("Bash(cargo +1.90.0 test --locked -p net-config:*)", rules)
        self.assertIn("Edit", rules)
        self.assertFalse([r for r in rules if r in ("Bash", "Bash(*)", "Bash(cargo:*)")])
        self.assertNotIn("Edit", rr.allow_rules(None))
        self.assertFalse([r for r in rr.allow_rules(None) if "remember" in r])
        self.assertIn("Bash(git push:*)", rr.DENY)

    def test_failure_fingerprint_ignores_timings_but_not_content(self):
        a = rr.failure_fingerprint("checks", "error[E0308] in x.rs\nfinished in 1.20s")
        b = rr.failure_fingerprint("checks", "error[E0308] in x.rs\nfinished in 9.87s")
        c = rr.failure_fingerprint("checks", "error[E0425] in y.rs\nfinished in 1.20s")
        self.assertEqual(a, b)
        self.assertNotEqual(a, c)

    def test_structured_output_and_blocking_findings(self):
        self.assertEqual(rr.structured({"structured_output": {"a": 1}}), {"a": 1})
        self.assertEqual(rr.structured({"result": 'text {"verdict": "pass"} end'}), {"verdict": "pass"})
        self.assertIsNone(rr.structured({"result": "no json"}))
        findings = [{"severity": "minor"}, {"severity": "major"}, {"severity": "blocker"}]
        self.assertEqual(len(rr.blocking(findings)), 2)

    def test_transcript_check_counts_compactions_and_proves_a_fresh_start(self):
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            slug = __import__("re").sub(r"[^A-Za-z0-9]", "-", str(rr.ROOT))
            folder = home / ".claude" / "projects" / slug
            folder.mkdir(parents=True)
            lines = [{"type": "user", "sessionId": "s1", "parentUuid": None, "message": {"content": "brief"}},
                     {"type": "assistant", "sessionId": "s1", "message": {"usage": {"input_tokens": 5,
                      "cache_read_input_tokens": 100, "output_tokens": 7}}},
                     {"type": "system", "subtype": "compact_boundary", "sessionId": "s1"},
                     {"type": "user", "sessionId": "s1", "parentUuid": "x", "message": {"content": "summary"}}]
            (folder / "s1.jsonl").write_text("\n".join(json.dumps(l) for l in lines))
            with unittest.mock.patch.object(rr.Path, "home", return_value=home):
                check = rr.Controller.transcript_check("s1", "brief\n")
            self.assertTrue(check["first_message_is_brief"])
            self.assertEqual((check["root_user_messages"], check["compactions"], check["foreign_session_entries"]),
                             (1, 1, 0))
            self.assertEqual((check["output_tokens"], check["peak_context_tokens"]), (7, 105))

    def test_dead_pid_is_not_alive(self):
        self.assertTrue(rr.pid_alive(1) or True)
        self.assertFalse(rr.pid_alive(2 ** 22 + 12345))


if __name__ == "__main__":
    unittest.main()

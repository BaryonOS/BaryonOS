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


def sandbox_result(stdout=b"", stderr=b"", code=0):
    result = unittest.mock.Mock(returncode=code, stdout_bytes=stdout, stderr_bytes=stderr)
    result.resource_status = {}
    return result


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
            {"tasks": [task("A", writes=["services/AGENTS.md"])]},
            {"tasks": [task("A", writes=[".codex/config.toml"])]},
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
        rr.validate_ledger({"tasks": [task("C", writes=["docs/verification/new-20261115/"])]})

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
                     "docs/PROJECT-STATE.md", "services/net/AGENTS.md",
                     "tools/CLAUDE.md", ".codex/config.toml"):
            self.assertFalse(rr.in_write_set(path, writes + ["docs/", ".github/", "tools/"]), path)

    def test_porcelain_parsing_handles_renames_and_quotes(self):
        text = ' M a/b.rs\n?? "c d.txt"\nR  old.rs -> new.rs\n'
        self.assertEqual(rr.parse_porcelain(text), ["a/b.rs", "c d.txt", "new.rs"])


class Session(unittest.TestCase):
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

    def test_effort_is_full_for_security_and_routine_otherwise(self):
        cfg = rr.Config()
        self.assertEqual(rr.effort_for(cfg, dict(task("A"), risk="security")), "xhigh")
        self.assertEqual(rr.effort_for(cfg, dict(task("A"), risk="high")), "xhigh")
        self.assertEqual(rr.effort_for(cfg, dict(task("A"), risk="low")), "high")
        self.assertEqual(rr.effort_for(cfg, None), "xhigh")
        argv = rr.claude_argv(cfg, "n", "1", Path("/s.json"), {}, "acceptEdits", 1.0, None, "high")
        self.assertIn("--effort high", " ".join(argv))

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

    def test_dead_pid_is_not_alive(self):
        self.assertTrue(rr.pid_alive(1) or True)
        self.assertFalse(rr.pid_alive(2 ** 22 + 12345))

    def test_incomplete_protected_sandbox_policy_fails_closed(self):
        with tempfile.TemporaryDirectory() as tmp:
            policy = Path(tmp) / "policy.json"
            policy.write_text(json.dumps({"schema": 2, "sandbox": {}}))
            with self.assertRaisesRegex(rr.Stop, "reviewed isolated runtime"):
                rr.load_sandbox(policy)

    def test_session_runs_inside_private_sandbox(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            attempt = root / "attempt"
            stage = root / "stage"
            attempt.mkdir()
            stage.mkdir()
            sandbox = unittest.mock.Mock(timeout=100)
            sandbox.run.return_value = sandbox_result(
                json.dumps({"structured_output": {"state": "implemented"}}).encode())
            controller = rr.Controller(rr.Config(), root / "state", sandbox=sandbox)
            controller.sandbox_verified = True
            lease = unittest.mock.Mock(path=stage)
            with unittest.mock.patch.object(controller, "heartbeat"):
                record = controller.session(
                    "worker", "A", attempt, "one bounded prompt", rr.WORKER_SCHEMA,
                    rr.allow_rules(task("A"), ("Edit",)), "acceptEdits", 60, lease)
            argv, used_lease = sandbox.run.call_args.args
            self.assertIs(used_lease, lease)
            self.assertEqual(argv[-1], "one bounded prompt")
            self.assertEqual(sandbox.run.call_args.kwargs["profile"], "agent")
            self.assertFalse(sandbox.run.call_args.kwargs["read_only"])
            self.assertTrue(record["transcript"]["private_sandbox_home"])

    def test_candidate_delta_excludes_disposable_outputs(self):
        with tempfile.TemporaryDirectory() as tmp:
            stage = Path(tmp)
            source = stage / "services" / "x" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text("before")
            before = rr.agent_source_state(stage)
            source.write_text("after")
            (stage / "build").mkdir()
            (stage / "build" / "payload").write_text("discard")
            rr.clear_sandbox_outputs(stage)
            self.assertEqual(rr.staged_changes(before, stage), ["services/x/lib.rs"])

    def test_gate_resolution_failure_is_a_recorded_check_failure(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            attempt = root / "attempt"
            attempt.mkdir()
            stage = root / "stage"
            stage.mkdir()
            sandbox = unittest.mock.Mock(timeout=100)
            sandbox.profiles = {"gate": {"output_max_bytes": 1024}}
            sandbox.run.return_value = sandbox_result()
            controller = rr.Controller(rr.Config(), root / "state", sandbox=sandbox)
            controller.sandbox_verified = True
            checked = dict(task("A", checks=[]), gates=["ci.yml#host-quality"])
            with unittest.mock.patch.object(controller, "heartbeat"), \
                    unittest.mock.patch.object(rr, "resolve_gate", side_effect=ValueError("unsupported gate")):
                passed, results, failure = controller.run_checks(
                    checked, attempt, unittest.mock.Mock(path=stage))
            self.assertFalse(passed)
            self.assertEqual(results[-1]["exit_code"], rr.GATE_FAIL_EXIT)
            self.assertEqual(failure, "gate blocked: unsupported gate")
            self.assertEqual((attempt / "gate.log").read_text(), failure + "\n")
            self.assertTrue(all(call.kwargs["read_only"] for call in sandbox.run.call_args_list))


if __name__ == "__main__":
    unittest.main()

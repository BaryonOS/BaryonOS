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
    def test_codex_dispatch_is_fresh_and_preserves_configuration(self):
        argv = rr.codex_argv(Path("/result.json"))
        self.assertEqual(argv[:2], ["codex", "exec"])
        self.assertEqual(argv[-1], "-")
        self.assertIn("--ephemeral", argv)
        self.assertIn("--strict-config", argv)
        self.assertEqual(argv[argv.index("--sandbox") + 1], "workspace-write")
        self.assertFalse(set(argv) & {"resume", "fork", "--ignore-user-config", "--ignore-rules",
                                     "--dangerously-bypass-approvals-and-sandbox", "--full-auto"})

    def test_automatic_ingresses_remain_held_before_commands_or_writes(self):
        with tempfile.TemporaryDirectory() as tmp, \
                unittest.mock.patch.object(rr.subprocess, "Popen") as child, \
                unittest.mock.patch.object(rr.subprocess, "run") as command:
            controller = rr.Controller(rr.Config(), Path(tmp))
            calls = (
                lambda: controller.session("worker", "A", Path(tmp), "brief", {}, [], "default", 10),
                lambda: controller.run_checks(task("A"), Path(tmp)),
                lambda: controller.run_task(task("A")),
            )
            for call in calls:
                with self.subTest(call=call), self.assertRaises(rr.TaskError):
                    call()
            self.assertEqual(list(Path(tmp).iterdir()), [])
            child.assert_not_called()
            command.assert_not_called()

    def test_manual_claim_is_durable_exclusive_and_requires_lock(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            ledger = root / "ledger.json"
            rr.save_ledger({"tasks": [task("A"), task("B")]}, ledger)
            controller = rr.Controller(rr.Config(), root / "state")
            with unittest.mock.patch.object(rr, "LEDGER", ledger), \
                    unittest.mock.patch.object(controller, "preflight", return_value="a" * 40):
                with self.assertRaisesRegex(rr.Stop, "lock"):
                    controller.prepare_manual()
                controller.acquire()
                try:
                    first = controller.prepare_manual()
                    self.assertEqual(first["task_id"], "A")
                    self.assertEqual(first["brief_sha256"], rr.sha256(Path(first["stdin"]).read_text()))
                    brief = Path(first["stdin"]).read_text()
                    self.assertIn("the lead runs the task's checks and gates", brief)
                    self.assertNotIn("the controller runs the task's checks", brief)
                    self.assertFalse(first["unattended_ready"])
                    with self.assertRaisesRegex(rr.Stop, "already active"):
                        controller.prepare_manual()
                finally:
                    controller.release()
                self.assertEqual(rr.load_ledger(ledger)["tasks"][0]["status"], "active")
                self.assertFalse((root / "state/lease.json").exists())
                rr.update_task("A", path=ledger, status="verified")
                controller.acquire()
                try:
                    second = controller.prepare_manual()
                    self.assertEqual(second["task_id"], "B")
                    self.assertNotEqual(first["stdin"], second["stdin"])
                finally:
                    controller.release()

    def test_refused_controller_does_not_remove_foreign_lease(self):
        with tempfile.TemporaryDirectory() as tmp:
            state = Path(tmp)
            owner = rr.Controller(rr.Config(), state)
            contender = rr.Controller(rr.Config(), state)
            owner.acquire()
            owner.heartbeat(task="A")
            try:
                with self.assertRaisesRegex(rr.Stop, "another controller"):
                    contender.acquire()
                contender.release()
                self.assertTrue((state / "lease.json").exists())
            finally:
                owner.release()
            self.assertFalse((state / "lease.json").exists())

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

    def test_effort_is_full_for_security_and_routine_otherwise(self):
        cfg = rr.Config()
        self.assertEqual(rr.effort_for(cfg, dict(task("A"), risk="security")), "xhigh")
        self.assertEqual(rr.effort_for(cfg, dict(task("A"), risk="high")), "xhigh")
        self.assertEqual(rr.effort_for(cfg, dict(task("A"), risk="low")), "high")
        self.assertEqual(rr.effort_for(cfg, None), "xhigh")
        argv = rr.claude_argv(cfg, "n", "1", Path("/s.json"), {}, "acceptEdits", 1.0, None, "high")
        self.assertIn("--effort high", " ".join(argv))

    def test_host_sessions_get_no_check_commands(self):
        rules = rr.allow_rules(("Edit",))
        self.assertIn("Edit", rules)
        # Candidate code runs only in the isolated gate, never as a host session command.
        self.assertEqual([r for r in rules if r.startswith("Bash(") and not r.startswith("Bash(git ")], [])
        self.assertNotIn("Edit", rr.allow_rules())
        self.assertFalse([r for r in rr.allow_rules() if "remember" in r])
        self.assertIn("Bash(git push:*)", rr.DENY)
        self.assertIn("Edit(build/roadmap-pipeline/**)", rr.DENY)  # baseline, STOP and logs
        for path in ("Edit(.git/**)", "Edit(**/__pycache__/**)", "Edit(**/*.pyc)"):
            self.assertIn(path, rr.DENY)
        # git diff/log/show accept --output=<path>, an arbitrary write (e.g. .git/config.worktree).
        self.assertFalse([r for r in rules if r.startswith(("Bash(git diff", "Bash(git log", "Bash(git show"))])
        # A home-directory deny would block every edit in a checkout under $HOME.
        self.assertFalse([r for r in rr.DENY if r.startswith("Edit(~") or r.startswith("Edit(/")])
        self.assertNotIn("progress.py --check", rr.Controller(rr.Config(), Path("/nonexistent")).init_prompt())

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

    def checks(self, checked, **patches):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            attempt = root / "attempt"
            attempt.mkdir()
            controller = rr.Controller(rr.Config(), root / "state")
            with unittest.mock.patch.object(controller, "heartbeat"), \
                    unittest.mock.patch.object(rr, "require_pipeline_release"), \
                    unittest.mock.patch.object(controller, "changed", return_value=["services/x/new.rs"]), \
                    unittest.mock.patch.object(rr.subprocess, "call") as host, \
                    unittest.mock.patch.object(rr.subprocess, "run") as run, \
                    unittest.mock.patch.multiple(rr, **(patches or {"ROOT": rr.ROOT})):
                outcome = controller.run_checks(checked, attempt)
                logs = list(attempt.glob("checks-*/checks.log"))
                log = logs[0].read_text() if logs else ""
                self.assertEqual(len(list(attempt.glob("checks-*"))), 1)  # a fresh directory per run
            host.assert_not_called()
            # Only read-only Git listing may run on the host; no check command does.
            self.assertEqual([c for c in run.call_args_list if not c.args[0][0].endswith("git")], [])
            return outcome, log

    def test_checks_and_progress_run_only_through_the_isolated_gate(self):
        seen = {}

        def gate(plan, cwd, log, env, *, source_paths):
            seen.update(plan=plan, source_paths=source_paths)
            log.write_text("ok\n")
            return 0

        (passed, results, failure), _ = self.checks(task("A", checks=["cargo test -p x"]),
                                                    run_gate=unittest.mock.Mock(side_effect=gate))
        self.assertTrue(passed)
        self.assertEqual(seen["plan"].commands, (("check-1", "cargo test -p x", {}), rr.PROGRESS_CHECK))
        self.assertIn("services/x/new.rs", seen["source_paths"])
        self.assertEqual(results[0]["exit_code"], 0)

    def test_gate_commands_follow_task_checks_once(self):
        resolved = rr.GatePlan((("g", "make g", {}), rr.PROGRESS_CHECK))
        gate = unittest.mock.Mock(return_value=0)
        self.checks(dict(task("A", checks=["true"]), gates=["ci.yml#x"]),
                    resolve_gate=unittest.mock.Mock(return_value=resolved), run_gate=gate)
        self.assertEqual(gate.call_args.args[0].commands,
                         (("check-1", "true", {}), ("g", "make g", {}), rr.PROGRESS_CHECK))

    def test_disabled_policy_fails_the_checks_closed(self):
        # The real gate with the committed disabled policy: nothing executes.
        (passed, results, failure), log = self.checks(task("A", checks=["true"]))
        self.assertFalse(passed)
        self.assertEqual(results[0]["exit_code"], rr.GATE_FAIL_EXIT)
        self.assertIn("not authorized", log)

    def test_gate_resolution_failure_is_a_recorded_check_failure(self):
        gate = unittest.mock.Mock()
        (passed, results, failure), log = self.checks(
            dict(task("A", checks=[]), gates=["ci.yml#host-quality"]),
            resolve_gate=unittest.mock.Mock(side_effect=ValueError("unsupported gate")), run_gate=gate)
        self.assertFalse(passed)
        self.assertEqual(results[-1]["exit_code"], rr.GATE_FAIL_EXIT)
        self.assertEqual(failure, "gate blocked: unsupported gate")
        self.assertEqual(log, failure + "\n")
        gate.assert_not_called()

    def test_git_callback_change_during_a_session_stops_the_controller(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            env = {k: v for k, v in __import__("os").environ.items() if not k.startswith("GIT_")}
            rr.subprocess.run(["git", "init", "-q", str(repo)], check=True, env=env)
            with unittest.mock.patch.object(rr, "ROOT", repo), \
                    unittest.mock.patch.dict("os.environ", env, clear=True):
                before = rr.git_callbacks()
                self.assertEqual(before, rr.git_callbacks())
                hook = repo / ".git" / "hooks" / "post-checkout"
                hook.write_text("#!/bin/sh\n")
                self.assertNotEqual(before, rr.git_callbacks())
                hook.unlink()
                rr.subprocess.run(["git", "-C", str(repo), "config", "core.fsmonitor", "x"], check=True)
                self.assertNotEqual(before, rr.git_callbacks())

    def test_plain_checks_keep_the_configured_deadline(self):
        gate = unittest.mock.Mock(return_value=0)
        self.checks(task("A", checks=["true"]), run_gate=gate)
        self.assertEqual(gate.call_args.args[0].timeout_seconds, rr.Config().check_timeout)

    def test_resolved_gate_keeps_finalizers_and_adds_its_deadline(self):
        resolved = rr.GatePlan((("g", "make g", {}), rr.PROGRESS_CHECK), (("f", "cleanup", {}),), 600, 60)
        gate = unittest.mock.Mock(return_value=0)
        self.checks(dict(task("A", checks=["true"]), gates=["ci.yml#x"]),
                    resolve_gate=unittest.mock.Mock(return_value=resolved), run_gate=gate)
        plan = gate.call_args.args[0]
        self.assertEqual((plan.finalizers, plan.timeout_seconds, plan.finalizer_reserve_seconds),
                         (resolved.finalizers, rr.Config().check_timeout + 600, 60))

    def test_unstageable_worker_path_is_a_recorded_failure(self):
        gate = unittest.mock.Mock()
        (passed, results, failure), _ = self.checks(
            task("A"), run_gate=gate, agent_source_paths=unittest.mock.Mock(side_effect=ValueError("bad path")))
        self.assertEqual((passed, failure), (False, "gate blocked: bad path"))
        gate.assert_not_called()

    def test_failure_feedback_inlines_raw_streams_without_resource_noise(self):
        with tempfile.TemporaryDirectory() as tmp:
            log = Path(tmp) / "checks.log"

            def write(cpu, error):
                log.write_text(f"## source commit: abc\n## step: check-1\n## resource: {{\"cpu\": {cpu}}}\n"
                               "## bounded raw streams: checks.log.step-1.stderr.log\n## exit: 101\n"
                               "## FAIL: check-1 (exit 101)\n## scratch: {\"id\": %d}\n" % cpu)
                (Path(tmp) / "checks.log.step-1.stderr.log").write_text(f"error[{error}]: mismatched types\n")
                return rr.gate_report(log)

            first, again, other = write(1, "E0308"), write(2, "E0308"), write(1, "E0425")
        self.assertIn("error[E0308]: mismatched types", first[1])
        self.assertNotIn("resource", first[1])
        self.assertEqual(first[1], again[1])  # identical failures stay identical
        self.assertNotEqual(first[0], other[0])  # the receipt binds the raw streams
        self.assertNotEqual(rr.failure_fingerprint("checks", first[1]),
                            rr.failure_fingerprint("checks", other[1]))

    def test_controller_git_ignores_a_planted_fsmonitor(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp) / "repo"
            env = {k: v for k, v in __import__("os").environ.items() if not k.startswith("GIT_")}
            rr.subprocess.run(["git", "init", "-q", str(repo)], check=True, env=env)
            marker = Path(tmp) / "PWNED"
            rr.subprocess.run(["git", "-C", str(repo), "config", "core.fsmonitor",
                               f"touch {marker}; false"], check=True, env=env)
            with unittest.mock.patch.object(rr, "ROOT", repo), \
                    unittest.mock.patch.dict("os.environ", env, clear=True):
                rr.git("status", "--porcelain")
            self.assertFalse(marker.exists())

    def test_symlinked_hook_target_changes_are_detected(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp) / "repo"
            env = {k: v for k, v in __import__("os").environ.items() if not k.startswith("GIT_")}
            rr.subprocess.run(["git", "init", "-q", str(repo)], check=True, env=env)
            target = Path(tmp) / "hook.sh"
            target.write_text("#!/bin/sh\n")
            (repo / ".git" / "hooks" / "post-commit").symlink_to(target)
            with unittest.mock.patch.object(rr, "ROOT", repo), \
                    unittest.mock.patch.dict("os.environ", env, clear=True):
                before = rr.git_callbacks()
                target.write_text("#!/bin/sh\ncurl evil\n")
                self.assertNotEqual(before, rr.git_callbacks())

    def test_main_drops_checkout_paths_before_preloading_yaml(self):
        tools = str(rr.ROOT / "tools")
        seen = []
        real = __import__

        def spy(name, *args, **kwargs):
            if name == "yaml":
                seen.append(list(rr.sys.path))
            return real(name, *args, **kwargs)

        with unittest.mock.patch.object(rr.sys, "path", [tools, str(rr.ROOT), "/usr/lib/python3"]), \
                unittest.mock.patch("builtins.__import__", side_effect=spy):
            rr.isolate_imports()
            self.assertEqual(rr.sys.path, ["/usr/lib/python3"])
        self.assertEqual(seen, [["/usr/lib/python3"]])

    def test_manual_brief_names_the_lead_as_check_runner(self):
        controller = rr.Controller(rr.Config(), Path("/nonexistent"))
        manual = controller.worker_prompt(task("A"), "a" * 40, "", "the lead runs the checks")
        self.assertIn("Do\n  not build or run code: the lead runs the checks.", manual)
        self.assertNotIn("controller runs", manual)
        self.assertIn("the controller runs the task's checks", controller.worker_prompt(task("A"), "a" * 40, ""))

    def test_fingerprint_sees_which_test_failed_and_ignores_durations(self):
        stderr = "\n".join(f"   Compiling crate{i} v0.1.0" for i in range(30)) + "\nerror: test failed"
        one = f"## stdout\ntest parser::rejects_overlong ... FAILED\n{stderr}"
        two = f"## stdout\ntest parser::rejects_zero_len ... FAILED\n{stderr}"
        self.assertNotEqual(rr.failure_fingerprint("checks", one), rr.failure_fingerprint("checks", two))
        self.assertEqual(rr.failure_fingerprint("checks", "not ok 1\n# duration_ms 70.065339"),
                         rr.failure_fingerprint("checks", "not ok 1\n# duration_ms 12.5"))
        self.assertEqual(rr.failure_fingerprint("checks", "not ok 1\n  duration_ms: 2.083117"),
                         rr.failure_fingerprint("checks", "not ok 1\n  duration_ms: 1.829266"))
        self.assertNotEqual(rr.failure_fingerprint("checks", "error[E0308]"),
                            rr.failure_fingerprint("checks", "error[E0425]"))

    def test_boundary_failure_output_reaches_feedback(self):
        with tempfile.TemporaryDirectory() as tmp:
            log = Path(tmp) / "checks.log"
            log.write_text("## step: check-1\n## BLOCKED: isolated command exited 1 before boundary cleanup failed\n")
            (Path(tmp) / "checks.log.resource.stderr.log").write_text("thread panicked at overflow\n")
            (Path(tmp) / "checks.log.planted").mkdir()  # never read, never crashes
            digest, report = rr.gate_report(log)
        self.assertIn("thread panicked at overflow", report)

    def test_untracked_tools_modules_are_detected_before_sibling_imports(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            env = {k: v for k, v in __import__("os").environ.items() if not k.startswith("GIT_")}
            rr.subprocess.run(["git", "init", "-q", str(repo)], check=True, env=env)
            tools = repo / "tools"
            (tools / "__pycache__").mkdir(parents=True)
            (tools / "agent_gate.py").write_text("")
            rr.subprocess.run(["git", "-C", str(repo), "add", "tools/agent_gate.py"], check=True, env=env)
            (tools / "notes.txt").write_text("")
            (tools / "tests").mkdir()
            (tools / "tests" / "x.py").write_text("")
            rr.subprocess.run(["git", "-C", str(repo), "add", "tools/tests/x.py"], check=True, env=env)
            (tools / "tests" / "scratch.py").write_text("")  # untracked, but not a top-level name
            with unittest.mock.patch.dict("os.environ", env, clear=True):
                self.assertEqual(rr.planted_modules(str(tools)), [])
                (tools / "yaml.py").write_text("")
                (tools / "shlex").mkdir()
                (tools / "shlex" / "__init__.py").write_text("")
                self.assertEqual(rr.planted_modules(str(tools)), ["shlex/", "yaml.py"])
                for name in ("yaml.py", "shlex/__init__.py"):
                    (tools / name).unlink()
                (tools / "shlex").rmdir()
                (repo / ".gitignore").write_text("*.pyc\n*.so\n")  # ignored files still count
                (tools / "secrets.pyc").write_bytes(b"")
                (tools / "secrets.abi3.so").write_bytes(b"")
                self.assertEqual(rr.planted_modules(str(tools)), ["secrets.abi3.so", "secrets.pyc"])
                for name in ("secrets.pyc", "secrets.abi3.so"):
                    (tools / name).unlink()
                (tools / "secrets.py").write_text("")
                rr.subprocess.run(["git", "init", "-q", str(tools)], check=True, env=env)
                rr.subprocess.run(["git", "-C", str(tools), "add", "-A"], check=True, env=env)
                self.assertEqual(rr.planted_modules(str(tools)), [".git", "secrets.py"])
        self.assertEqual(rr.sys.path[-1], rr.TOOLS_DIR)  # the checkout resolves after libraries

    def test_stop_file_holds_before_the_init_check(self):
        with tempfile.TemporaryDirectory() as tmp:
            controller = rr.Controller(rr.Config(), Path(tmp))
            (Path(tmp) / "STOP").write_text("callbacks changed\n")
            with unittest.mock.patch.object(controller, "init_check") as init, \
                    unittest.mock.patch.object(controller, "preflight") as preflight:
                with self.assertRaisesRegex(rr.Stop, "stop file"):
                    controller.loop(1, True)
            init.assert_not_called()
            preflight.assert_not_called()

    def test_stop_holds_before_recovering_a_stale_lease(self):
        with tempfile.TemporaryDirectory() as tmp:
            state = Path(tmp)
            (state / "STOP").write_text("x\n")
            (state / "lease.json").write_text(json.dumps({"controller_pid": 2 ** 22 + 1, "worker_pid": 2 ** 22 + 2}))
            controller = rr.Controller(rr.Config(), state)
            with unittest.mock.patch.object(rr, "git") as git:
                with self.assertRaisesRegex(rr.Stop, "stop file"):
                    controller.acquire()
                controller.release()
            git.assert_not_called()

    def test_brief_honours_stop_before_any_git_call(self):
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / "STOP").write_text("x\n")
            with unittest.mock.patch.object(rr, "STATE", Path(tmp)), \
                    unittest.mock.patch.object(rr, "git") as git, \
                    unittest.mock.patch.object(rr, "isolate_imports"), \
                    unittest.mock.patch.object(rr.sys, "argv", ["roadmap_runner.py", "brief"]):
                self.assertEqual(rr.main(), 2)
            git.assert_not_called()

    def interrupted_session(self, root, callbacks):
        controller = rr.Controller(rr.Config(), root / "state")
        controller.callbacks = "before"
        proc = unittest.mock.Mock(pid=2 ** 22 + 12345)
        proc.wait.side_effect = KeyboardInterrupt
        with unittest.mock.patch.object(rr, "require_pipeline_release"), \
                unittest.mock.patch.object(rr.subprocess, "Popen", return_value=proc), \
                unittest.mock.patch.object(rr.os, "killpg") as kill, \
                unittest.mock.patch.object(controller, "heartbeat"), \
                unittest.mock.patch.object(rr, "git_callbacks", return_value=callbacks):
            try:
                controller.session("worker", "A", root, "brief", {}, [], "default", 10)
            except (KeyboardInterrupt, rr.Stop) as error:
                outcome = error
        kill.assert_called_with(proc.pid, rr.signal.SIGKILL)  # no live worker after an exception
        return controller, outcome

    def test_interrupted_session_still_checks_callbacks(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _, outcome = self.interrupted_session(root, "after")
            self.assertIsInstance(outcome, rr.Stop)
            self.assertTrue((root / "state" / "STOP").exists())

    def test_acquire_compares_the_saved_baseline_before_recover_runs_git(self):
        for later, stops in (("after", True), ("before", False)):
            with self.subTest(later=later), tempfile.TemporaryDirectory() as tmp:
                state = Path(tmp) / "state"
                state.mkdir()
                marker = state / rr.CALLBACKS_MARKER
                marker.write_text(json.dumps({"baseline": "before", "session": "s"}))
                (state / "lease.json").write_text(json.dumps({"task": "A"}))  # crashed owner
                controller = rr.Controller(rr.Config(), state)
                try:
                    with unittest.mock.patch.object(rr, "git_callbacks", return_value=later), \
                            unittest.mock.patch.object(rr, "git", side_effect=rr.Stop("git reached")) as git:
                        with self.assertRaisesRegex(rr.Stop, "interrupted session" if stops else "git reached"):
                            controller.acquire()
                finally:
                    controller.release()
                if stops:
                    git.assert_not_called()  # recover()'s git status never ran
                self.assertEqual((state / "STOP").exists(), stops)
                self.assertTrue(marker.exists())  # retired only by preflight, once no worker lives

    def test_acquire_refuses_a_live_worker_before_comparing_the_baseline(self):
        with tempfile.TemporaryDirectory() as tmp:
            state = Path(tmp) / "state"
            state.mkdir()
            marker = state / rr.CALLBACKS_MARKER
            marker.write_text(json.dumps({"baseline": "before", "session": "s"}))
            (state / "lease.json").write_text(json.dumps({"task": "A"}))
            controller = rr.Controller(rr.Config(), state)
            live = rr.subprocess.CompletedProcess([], 0, "424242\n", "")
            try:
                with unittest.mock.patch.object(rr.subprocess, "run", return_value=live), \
                        unittest.mock.patch.object(rr, "git_callbacks") as callbacks, \
                        unittest.mock.patch.object(rr, "git") as git:
                    with self.assertRaisesRegex(rr.Stop, "still alive"):
                        controller.acquire()
            finally:
                controller.release()
            callbacks.assert_not_called()  # nothing compared while the worker can still change it
            git.assert_not_called()
            self.assertTrue(marker.exists())

    def test_preflight_refuses_a_live_worker_before_retiring_the_baseline(self):
        with tempfile.TemporaryDirectory() as tmp:
            state = Path(tmp)
            marker = state / rr.CALLBACKS_MARKER
            marker.write_text(json.dumps({"baseline": "before", "session": "s"}))
            live = rr.subprocess.CompletedProcess([], 0, "424242\n", "")
            with unittest.mock.patch.object(rr.subprocess, "run", return_value=live), \
                    unittest.mock.patch.object(rr, "git_callbacks", return_value="before") as callbacks, \
                    unittest.mock.patch.object(rr, "git") as git:
                with self.assertRaisesRegex(rr.Stop, "still alive"):
                    rr.Controller(rr.Config(), state).preflight()
            callbacks.assert_not_called()
            git.assert_not_called()
            self.assertTrue(marker.exists())

    def test_push_ignores_inherited_repository_location(self):
        with tempfile.TemporaryDirectory() as tmp:
            controller = rr.Controller(rr.Config(push=True), Path(tmp))
            pushed = rr.subprocess.CompletedProcess([], 0, "", "")
            with unittest.mock.patch.dict("os.environ", {"GIT_DIR": "/elsewhere/.git"}), \
                    unittest.mock.patch.object(rr.subprocess, "run", return_value=pushed) as run:
                controller.push()
            self.assertNotIn("GIT_DIR", run.call_args.kwargs["env"])

    def test_interrupted_session_leaves_its_baseline_for_the_next_preflight(self):
        for later, stops in (("after", True), ("before", False)):
            with self.subTest(later=later), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                controller, outcome = self.interrupted_session(root, "before")
                self.assertIsInstance(outcome, KeyboardInterrupt)
                marker = root / "state" / rr.CALLBACKS_MARKER
                self.assertEqual(json.loads(marker.read_text())["baseline"], "before")
                fresh = rr.Controller(rr.Config(), root / "state")
                no_worker = rr.subprocess.CompletedProcess([], 1, "", "")
                with unittest.mock.patch.object(rr, "git_callbacks", return_value=later), \
                        unittest.mock.patch.object(rr.subprocess, "run", return_value=no_worker), \
                        unittest.mock.patch.object(rr, "git", side_effect=rr.Stop("git reached")) as git:
                    with self.assertRaisesRegex(rr.Stop, "interrupted session" if stops else "git reached"):
                        fresh.preflight()
                self.assertEqual((root / "state" / "STOP").exists(), stops)
                self.assertEqual(marker.exists(), stops)  # adopted only when unchanged
                if stops:
                    git.assert_not_called()  # no hook-running Git call before the comparison

    def test_controller_git_ignores_inherited_repository_location(self):
        with tempfile.TemporaryDirectory() as tmp:
            env = {k: v for k, v in __import__("os").environ.items() if not k.startswith("GIT_")}
            repos = [Path(tmp) / name for name in ("root", "other")]
            for repo in repos:
                rr.subprocess.run(["git", "init", "-q", str(repo)], check=True, env=env)
            (repos[0] / "only-in-root.txt").write_text("")
            inherited = dict(env, GIT_DIR=str(repos[1] / ".git"), GIT_WORK_TREE=str(repos[1]),
                             GIT_SSL_CAINFO="/ca.pem")
            with unittest.mock.patch.object(rr, "ROOT", repos[0]), \
                    unittest.mock.patch.dict("os.environ", inherited, clear=True):
                self.assertIn("only-in-root.txt", rr.git("status", "--porcelain", "--untracked-files=all"))
                self.assertEqual(rr.git_env().get("GIT_SSL_CAINFO"), "/ca.pem")
                self.assertNotIn("GIT_DIR", rr.git_env())

    def test_gates_resolve_before_the_worker_session(self):
        with tempfile.TemporaryDirectory() as tmp:
            controller = rr.Controller(rr.Config(), Path(tmp))
            with unittest.mock.patch.object(rr, "require_pipeline_release"), \
                    unittest.mock.patch.object(controller, "preflight", return_value="a" * 40), \
                    unittest.mock.patch.object(rr, "resolve_gate", side_effect=ImportError("No module named 'yaml'")), \
                    unittest.mock.patch.object(controller, "session") as session:
                with self.assertRaisesRegex(rr.Blocked, "gate cannot be resolved: No module named 'yaml'"):
                    controller.run_task(dict(task("A"), gates=["ci.yml#x"]))
            session.assert_not_called()

    def test_session_stops_when_git_callbacks_changed(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            controller = rr.Controller(rr.Config(), root / "state")
            controller.callbacks = "before"
            proc = unittest.mock.Mock(pid=2 ** 22 + 12345)
            proc.wait.return_value = 0
            with unittest.mock.patch.object(rr, "require_pipeline_release"), \
                    unittest.mock.patch.object(rr.subprocess, "Popen", return_value=proc), \
                    unittest.mock.patch.object(rr.os, "killpg"), \
                    unittest.mock.patch.object(controller, "heartbeat"), \
                    unittest.mock.patch.object(controller, "transcript_check", return_value={}), \
                    unittest.mock.patch.object(rr, "git_callbacks", side_effect=["before", "after"]):
                controller.session("worker", "A", root, "brief", {}, [], "default", 10)
                self.assertFalse((root / "state" / "STOP").exists())
                with self.assertRaisesRegex(rr.Stop, "Git hooks or configuration changed"):
                    controller.session("worker", "A", root, "brief", {}, [], "default", 10)
            self.assertTrue((root / "state" / "STOP").exists())  # a later run refuses to start
            self.assertIsNone(rr.Controller(rr.Config(), root).callbacks)

    def test_progress_assessment_runs_isolated_from_the_checkout(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "docs").mkdir()
            (root / "docs" / "progress.json").write_text(json.dumps({"tracks": []}))
            with unittest.mock.patch.object(rr, "ROOT", root), \
                    unittest.mock.patch.object(rr.subprocess, "run") as run, \
                    unittest.mock.patch.object(rr, "git", return_value=""):
                run.return_value = rr.subprocess.CompletedProcess([], 0, "", "")
                rr.Controller(rr.Config(), root / "state").record_assessment(dict(task("A"), title="t"), "a" * 40,
                                                                           "verified")
        argv = [c.args[0] for c in run.call_args_list]
        self.assertEqual([a[1:4] for a in argv], [["-I", "-B", "tools/progress.py"]] * 2)


if __name__ == "__main__":
    unittest.main()

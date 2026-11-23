import contextlib
import io
import os
import re
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
import unittest.mock
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import roadmap_runner as rr

REAL_GIT_CALLBACKS = rr.git_callbacks


def task(tid, status="ready", deps=(), writes=("services/x/",), checks=("true",)):
    return {"id": tid, "criterion": "N02", "title": tid, "goal": "g", "status": status,
            "deps": list(deps), "write_paths": list(writes), "checks": list(checks)}


class Ledger(unittest.TestCase):
    def test_codex_dispatch_is_fresh_and_drops_user_mcp_servers(self):
        argv = rr.codex_argv(Path("/result.json"))
        self.assertEqual(argv[:2], ["codex", "exec"])
        self.assertEqual(argv[-1], "-")
        self.assertIn("--ephemeral", argv)
        self.assertIn("--strict-config", argv)
        self.assertEqual(argv[argv.index("--sandbox") + 1], "workspace-write")
        self.assertFalse(set(argv) & {"resume", "fork", "--ignore-rules",
                                     "--dangerously-bypass-approvals-and-sandbox", "--full-auto"})
        # User MCP servers, plugins and apps run outside the shell sandbox, so they could reach the
        # baseline home; auth stays (Codex reads it from CODEX_HOME, not config.toml).
        self.assertIn("--ignore-user-config", argv)
        self.assertEqual(argv[argv.index("--cd") + 1], str(rr.ROOT))  # workspace-write covers the cwd
        disabled = {argv[i + 1] for i, a in enumerate(argv) if a == "--disable"}
        self.assertLessEqual({"plugins", "apps"}, disabled)

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

            def preflight():
                controller.callbacks = "cb"
                return "a" * 40
            with unittest.mock.patch.object(rr, "LEDGER", ledger), \
                    unittest.mock.patch.object(rr, "git_callbacks", return_value="cb"), \
                    unittest.mock.patch.object(controller, "preflight", side_effect=preflight):
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
                    # Pinned, or a launch from ~/.local/state would make the trusted home writable.
                    self.assertEqual(first["cwd"], str(rr.ROOT))
                    self.assertEqual(first["argv"][first["argv"].index("--cd") + 1], str(rr.ROOT))
                    marker = json.loads((root / "state" / rr.CALLBACKS_MARKER).read_text())
                    self.assertEqual(marker["baseline"], "cb")  # the next preflight compares, never adopts
                    self.assertEqual(marker["manual"], "A")  # the claim the in-checkout ledger cannot end
                    with self.assertRaisesRegex(rr.Stop, "already active"):
                        controller.prepare_manual()
                finally:
                    controller.release()
                self.assertEqual(rr.load_ledger(ledger)["tasks"][0]["status"], "active")
                self.assertFalse((root / "state/lease.json").exists())
                rr.update_task("A", path=ledger, status="verified")
                controller.acquire()
                try:
                    # The real preflight: a ledger the session can rewrite does not end the claim.
                    with unittest.mock.patch.object(controller, "preflight",
                                                    side_effect=lambda: rr.Controller.preflight(controller)), \
                            unittest.mock.patch.object(rr.Controller, "refuse_live_workers"), \
                            self.assertRaisesRegex(rr.Stop, "manual task A holds the session baseline"):
                        controller.prepare_manual()
                    self.assertEqual(controller.manual_claim(), "A")
                    # What `resume --accept` does once the digest matches (that path:
                    # test_a_manual_claim_outlives_a_ledger_the_session_rewrote); the claim ends.
                    controller.retire_baseline()
                    self.assertIsNone(controller.manual_claim())
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
            tools.mkdir()
            (tools / "agent_gate.py").write_text("")
            rr.subprocess.run(["git", "-C", str(repo), "add", "tools/agent_gate.py"], check=True, env=env)
            (tools / "notes.txt").write_text("")
            (tools / "tests").mkdir()
            (tools / "tests" / "x.py").write_text("")
            rr.subprocess.run(["git", "-C", str(repo), "add", "tools/tests/x.py"], check=True, env=env)
            (tools / "tests" / "notes.md").write_text("")  # untracked, but not importable
            with unittest.mock.patch.dict("os.environ", env, clear=True):
                self.assertEqual(rr.planted_modules(str(tools)), [])
                (tools / "tests" / "__pycache__").mkdir()  # a later lead run imports matching bytecode
                (tools / "tests" / "__pycache__" / "x.cpython-313.pyc").write_bytes(b"")
                self.assertEqual(rr.planted_modules(str(tools)), ["tests/__pycache__/"])
                shutil.rmtree(tools / "tests" / "__pycache__")
                (tools / "tests" / "scratch.py").write_text("")  # nested entries reach later test runs
                self.assertEqual(rr.planted_modules(str(tools)), ["tests/scratch.py"])
                (tools / "tests" / "scratch.py").unlink()
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
                unittest.mock.patch.object(controller, "ignored_files", return_value={}), \
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
                marker.write_text(json.dumps({"baseline": "before", "session": "s", "ignored": {}}))
                (state / "lease.json").write_text(json.dumps({"task": "A"}))  # crashed owner
                controller = rr.Controller(rr.Config(), state)
                try:
                    with unittest.mock.patch.object(rr, "git_callbacks", return_value=later), \
                            unittest.mock.patch.object(controller, "ignored_files", return_value={}), \
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
            self.assertEqual(run.call_args.args[0][:3], ["git", "-c", "core.fsmonitor=false"])

    def test_a_failed_worker_probe_is_not_taken_as_absence(self):
        for outcome in (rr.subprocess.CompletedProcess([], 2, "", "pgrep: unknown option"),
                        FileNotFoundError("pgrep")):
            with self.subTest(outcome=outcome), tempfile.TemporaryDirectory() as tmp:
                state = Path(tmp)
                marker = state / rr.CALLBACKS_MARKER
                marker.write_text(json.dumps({"baseline": "before", "session": "s"}))
                probe = {"side_effect": outcome} if isinstance(outcome, Exception) else {"return_value": outcome}
                with unittest.mock.patch.object(rr.subprocess, "run", **probe), \
                        unittest.mock.patch.object(rr, "git_callbacks", return_value="before") as callbacks:
                    with self.assertRaisesRegex(rr.Stop, "worker probe failed"):
                        rr.Controller(rr.Config(), state).preflight()
                callbacks.assert_not_called()
                self.assertTrue(marker.exists())  # never retired without evidence that no worker lives

    @unittest.skipUnless(shutil.which("pgrep"), "pgrep not installed")
    def test_worker_probe_finds_a_real_tagged_process(self):
        worker = rr.subprocess.Popen([sys.executable, "-c", "import time; time.sleep(30)",
                                      "--name", rr.PROCESS_TAG + "probe"])
        try:
            with tempfile.TemporaryDirectory() as tmp:
                with self.assertRaisesRegex(rr.Stop, f"still alive: .*{worker.pid}"):
                    rr.Controller(rr.Config(), Path(tmp)).refuse_live_workers()
        finally:
            worker.kill()
            worker.wait()
        with tempfile.TemporaryDirectory() as tmp:
            rr.Controller(rr.Config(), Path(tmp)).refuse_live_workers()  # exit 1: none left

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
                        unittest.mock.patch.object(fresh, "ignored_files", return_value={}), \
                        unittest.mock.patch.object(rr, "load_ledger", return_value={"tasks": []}), \
                        unittest.mock.patch.object(rr, "planted_modules", return_value=[]), \
                        unittest.mock.patch.object(rr.subprocess, "run", return_value=no_worker), \
                        unittest.mock.patch.object(rr, "git", side_effect=rr.Stop("git reached")) as git:
                    with self.assertRaisesRegex(rr.Stop, "interrupted session" if stops else "git reached"):
                        fresh.preflight()
                self.assertEqual((root / "state" / "STOP").exists(), stops)
                self.assertTrue(marker.exists())  # retired only once every preflight check passed
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
                    unittest.mock.patch.object(controller, "ignored_files", return_value={}), \
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


class RunnerResiduals(unittest.TestCase):
    """Negative tests for the roadmap-runner residuals of issue #416."""

    def repo(self, tmp, ignore=""):
        env = {k: v for k, v in __import__("os").environ.items() if not k.startswith("GIT_")}
        repo = Path(tmp) / "repo"
        repo.mkdir()
        (repo / ".gitignore").write_text(ignore)
        for argv in (["init", "-q"], ["config", "user.email", "t@example.invalid"], ["config", "user.name", "t"],
                     ["add", ".gitignore"], ["commit", "-q", "--no-verify", "-m", "base"]):
            rr.subprocess.run(["git", "-C", str(repo), *argv], check=True, env=env)
        return repo, env

    def test_controller_git_runs_no_repository_hook(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, env = self.repo(tmp)
            marker = Path(tmp) / "HOOK-RAN"
            for name in ("pre-commit", "commit-msg", "post-commit", "post-checkout", "pre-push"):
                hook = repo / ".git" / "hooks" / name
                hook.write_text(f"#!/bin/sh\necho {name} >> {marker}\n")
                hook.chmod(0o755)
            (repo / "f.txt").write_text("a\n")
            with unittest.mock.patch.object(rr, "ROOT", repo), \
                    unittest.mock.patch.dict("os.environ", env, clear=True):
                rr.git("add", "--", "f.txt")
                rr.git("commit", "--no-verify", "-q", "-m", "x")
                (repo / "f.txt").write_text("b\n")
                rr.Controller(rr.Config(), Path(tmp) / "state").revert(["f.txt"])
                before = rr.git_callbacks()
                (repo / ".git" / "hooks" / "post-commit").write_text("#!/bin/sh\ntrue\n")
                self.assertNotEqual(before, rr.git_callbacks())  # hooks stay fingerprinted
            self.assertEqual((repo / "f.txt").read_text(), "a\n")
            self.assertFalse(marker.exists())
            self.assertIn("core.hooksPath=/dev/null", rr.CONTROLLER_GIT_CONFIG)

    def test_push_and_commits_disable_hooks(self):
        controller = rr.Controller(rr.Config(push=True), Path(tempfile.gettempdir()))
        with unittest.mock.patch.object(rr.subprocess, "run") as run, unittest.mock.patch.object(rr.time, "sleep"):
            run.return_value = rr.subprocess.CompletedProcess([], 0, "", "")
            controller.push()
        argv = run.call_args.args[0]
        self.assertIn("core.hooksPath=/dev/null", argv)
        self.assertIn("--no-verify", argv)
        source = Path(rr.__file__).read_text()
        self.assertNotIn('git("commit", "-q"', source)  # every controller commit passes --no-verify

    def test_ignored_files_a_session_creates_are_out_of_scope(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, env = self.repo(tmp, "build/\n*.log\n.tools/\n")
            (repo / "build").mkdir()
            (repo / "build" / "cache.bin").write_text("old")
            state = repo / "build" / "roadmap-pipeline"
            state.mkdir()
            with unittest.mock.patch.object(rr, "ROOT", repo), \
                    unittest.mock.patch.dict("os.environ", env, clear=True):
                controller = rr.Controller(rr.Config(), state)
                before = controller.ignored_files()
                self.assertEqual(list(before), ["build/cache.bin"])  # controller state is excluded
                (state / "lease.json").write_text("{}")
                (repo / ".tools").mkdir()
                (repo / ".tools" / "actionlint").write_text("#!/bin/sh\n")
                (repo / "run.log").write_text("x")
                planted = controller.new_ignored(before)
                self.assertEqual(planted, [".tools/actionlint", "run.log"])
                self.assertEqual(controller.changed(), [])  # git status alone never saw them
                controller.revert(planted)
                self.assertEqual(controller.new_ignored(before), [])
                (repo / "build" / "cache.bin").write_text("planted")
                with self.assertRaisesRegex(rr.Stop, "changed or removed ignored files"):
                    controller.new_ignored(before)
                (repo / "build" / "cache.bin").unlink()
                with self.assertRaisesRegex(rr.Stop, "changed or removed ignored files"):
                    controller.new_ignored(before)

    def test_verifier_fingerprint_covers_ignored_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, env = self.repo(tmp, "out/\n")
            with unittest.mock.patch.object(rr, "ROOT", repo), \
                    unittest.mock.patch.dict("os.environ", env, clear=True):
                controller = rr.Controller(rr.Config(), Path(tmp) / "state")
                before = controller.tree_fingerprint()
                (repo / "out").mkdir()
                (repo / "out" / "x").write_text("x")
                self.assertNotEqual(before, controller.tree_fingerprint())

    def test_preflight_refuses_planted_modules_before_every_session(self):
        with tempfile.TemporaryDirectory() as tmp:
            no_worker = rr.subprocess.CompletedProcess([], 1, "", "")
            with unittest.mock.patch.object(rr.subprocess, "run", return_value=no_worker), \
                    unittest.mock.patch.object(rr, "load_ledger", return_value={"tasks": []}), \
                    unittest.mock.patch.object(rr, "git_callbacks", return_value="cb"), \
                    unittest.mock.patch.object(rr, "planted_modules", return_value=["tests/evil.py"]), \
                    unittest.mock.patch.object(rr, "git") as git:
                rr.atomic_write(Path(tmp) / rr.TRUSTED_RECORD, "{}")
                with self.assertRaisesRegex(rr.Stop, "tests/evil.py"):
                    rr.Controller(rr.Config(), Path(tmp)).preflight()
            git.assert_not_called()

    def test_run_refuses_git_overrides_before_a_session_is_spent(self):
        with tempfile.TemporaryDirectory() as tmp:
            controller = rr.Controller(rr.Config(), Path(tmp))
            with unittest.mock.patch.dict("os.environ", {"GIT_EDITOR": "vi"}), \
                    unittest.mock.patch.object(controller, "init_check") as init, \
                    unittest.mock.patch.object(controller, "run_task") as run:
                with self.assertRaisesRegex(rr.Stop, "controller environment refused"):
                    controller.loop(1, True)
            init.assert_not_called()  # not even the init session is spent
            run.assert_not_called()

    def test_a_path_that_survives_the_revert_stops_the_controller(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, env = self.repo(tmp, "out/\n")
            outside = Path(tmp) / "outside"
            outside.write_text("x")
            controller = rr.Controller(rr.Config(max_repairs=0), Path(tmp) / "state")

            def worker(*args, **kwargs):
                (repo / "out").mkdir()
                (repo / "out" / "link").symlink_to(outside)
                return {"exit_code": 0, "is_error": False, "output": {"state": "implemented"}}
            with unittest.mock.patch.object(rr, "ROOT", repo), \
                    unittest.mock.patch.object(rr, "require_pipeline_release"), \
                    unittest.mock.patch.dict("os.environ", env, clear=True), \
                    unittest.mock.patch.object(controller, "preflight", return_value="a" * 40), \
                    unittest.mock.patch.object(controller, "heartbeat"), \
                    unittest.mock.patch.object(controller, "session", side_effect=worker), \
                    unittest.mock.patch.object(controller, "revert"), \
                    unittest.mock.patch.object(controller, "run_checks") as checks:
                with self.assertRaisesRegex(rr.Stop, "survived the revert"):
                    controller.run_task(task("A"))
            checks.assert_not_called()
            self.assertTrue(outside.exists())

    @unittest.mock.patch.object(rr, "controller_environment", return_value={})  # independent of GIT_* in the shell
    def test_failed_init_check_oracles_stop_the_loop(self, _):
        good = {"session_id": "s", "exit_code": 0, "transcript": {}, "output": {},
                "deny_probe_file_absent": True, "checkout_clean_after": True}
        for field, value in (("deny_probe_file_absent", False), ("checkout_clean_after", False), ("exit_code", 1)):
            with self.subTest(field=field), tempfile.TemporaryDirectory() as tmp:
                controller = rr.Controller(rr.Config(), Path(tmp))
                with unittest.mock.patch.object(controller, "init_check", return_value=dict(good, **{field: value})), \
                        unittest.mock.patch.object(rr, "load_ledger") as ledger:
                    with self.assertRaisesRegex(rr.Stop, "init check failed"):
                        controller.loop(1, True)
                ledger.assert_not_called()  # no task selected after a failed check
        with tempfile.TemporaryDirectory() as tmp:
            controller = rr.Controller(rr.Config(), Path(tmp))
            with unittest.mock.patch.object(controller, "init_check", return_value=good), \
                    unittest.mock.patch.object(rr, "load_ledger", return_value={"tasks": []}):
                self.assertEqual(controller.loop(1, True), "no-ready-task")

    def test_active_manual_claim_keeps_its_baseline(self):
        with tempfile.TemporaryDirectory() as tmp:
            state = Path(tmp)
            marker = state / rr.CALLBACKS_MARKER
            marker.write_text(json.dumps({"baseline": "cb", "session": "manual A"}))
            no_worker = rr.subprocess.CompletedProcess([], 1, "", "")
            with unittest.mock.patch.object(rr.subprocess, "run", return_value=no_worker), \
                    unittest.mock.patch.object(rr, "load_ledger", return_value={"tasks": [task("A", "active")]}), \
                    unittest.mock.patch.object(rr, "git_callbacks", return_value="cb") as callbacks, \
                    unittest.mock.patch.object(rr, "git") as git:
                with self.assertRaisesRegex(rr.Stop, "manual task"):
                    rr.Controller(rr.Config(), state).preflight()
            self.assertTrue(marker.exists())  # not retired while the manual session may run
            callbacks.assert_not_called()
            git.assert_not_called()

    def test_nested_git_is_refused_before_and_after_a_session(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, env = self.repo(tmp)
            self.assertIn("Edit(**/.git/**)", rr.DENY)
            self.assertIn("Edit(**/.git)", rr.DENY)
            self.assertEqual(rr.nested_git(repo), [])
            (repo / "crates" / "foo" / ".git").mkdir(parents=True)  # invisible to git status and ls-files
            (repo / "crates" / "foo" / ".git" / "config").write_text("[core]\n\tfsmonitor = touch /tmp/x\n")
            (repo / "w").mkdir()
            (repo / "w" / ".git").write_text("gitdir: /elsewhere\n")  # a gitfile counts as well
            self.assertEqual(rr.nested_git(repo), ["crates/foo/.git", "w/.git"])
            no_worker = rr.subprocess.CompletedProcess([], 1, "", "")
            with unittest.mock.patch.object(rr, "ROOT", repo), \
                    unittest.mock.patch.object(rr.subprocess, "run", return_value=no_worker), \
                    unittest.mock.patch.object(rr, "load_ledger", return_value={"tasks": []}), \
                    unittest.mock.patch.object(rr, "planted_modules", return_value=[]), \
                    unittest.mock.patch.object(rr, "git_callbacks", return_value="cb"):
                rr.atomic_write(Path(tmp) / "state" / rr.TRUSTED_RECORD, "{}")
                with self.assertRaisesRegex(rr.Stop, "nested .git"):
                    rr.Controller(rr.Config(), Path(tmp) / "state").preflight()
            shutil.rmtree(repo / "crates")
            (repo / "w" / ".git").unlink()
            controller = rr.Controller(rr.Config(max_repairs=0), Path(tmp) / "state")

            def worker(*args, **kwargs):
                (repo / "services" / "x" / ".git").mkdir(parents=True)  # inside the write set, still refused
                return {"exit_code": 0, "is_error": False, "output": {"state": "implemented"}}
            with unittest.mock.patch.object(rr, "ROOT", repo), \
                    unittest.mock.patch.object(rr, "require_pipeline_release"), \
                    unittest.mock.patch.dict("os.environ", env, clear=True), \
                    unittest.mock.patch.object(controller, "preflight", return_value="a" * 40), \
                    unittest.mock.patch.object(controller, "heartbeat"), \
                    unittest.mock.patch.object(controller, "session", side_effect=worker), \
                    unittest.mock.patch.object(controller, "run_checks") as checks:
                before = None
                with self.assertRaisesRegex(rr.Stop, "nested .git"):
                    controller.run_task(task("A"))
                checks.assert_not_called()
                shutil.rmtree(repo / "services")
                before = controller.tree_fingerprint()
                (repo / "q" / ".git").mkdir(parents=True)
                self.assertNotEqual(before, controller.tree_fingerprint())  # the verifier check sees it

    def test_new_ignored_file_is_reverted_and_the_attempt_continues(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, env = self.repo(tmp, "build/\n")
            controller = rr.Controller(rr.Config(max_repairs=0), Path(tmp) / "state")

            def worker(*args, **kwargs):
                (repo / "services" / "x").mkdir(parents=True, exist_ok=True)
                (repo / "services" / "x" / "lib.rs").write_text("// in scope\n")
                (repo / "build").mkdir(exist_ok=True)
                (repo / "build" / "planted.sh").write_text("#!/bin/sh\n")
                return {"exit_code": 0, "is_error": False, "output": {"state": "implemented"}}
            with unittest.mock.patch.object(rr, "ROOT", repo), \
                    unittest.mock.patch.object(rr, "require_pipeline_release"), \
                    unittest.mock.patch.dict("os.environ", env, clear=True), \
                    unittest.mock.patch.object(controller, "preflight", return_value="a" * 40), \
                    unittest.mock.patch.object(controller, "heartbeat"), \
                    unittest.mock.patch.object(controller, "session", side_effect=worker), \
                    unittest.mock.patch.object(controller, "run_checks", return_value=(False, [], "red")) as checks:
                with self.assertRaisesRegex(rr.Blocked, "not verified"):
                    controller.run_task(task("A"))
            checks.assert_called_once()  # the attempt went on to the checks after the revert
            self.assertFalse((repo / "build" / "planted.sh").exists())
            attempts = json.loads(next((Path(tmp) / "state").rglob("attempts.json")).read_text())
            self.assertEqual(attempts[0]["reverted_out_of_scope"], ["build/planted.sh"])

    def test_revert_removes_a_symlink_never_its_target(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, env = self.repo(tmp, "build/\nout/\n")
            (repo / "build").mkdir()
            (repo / "out").mkdir()
            keep_file, keep_dir = Path(tmp) / "keep.txt", repo / ".git"
            keep_file.write_text("x")
            cache = repo / "build" / "cache.bin"  # a pre-existing ignored file inside the checkout
            cache.write_text("keep")
            (repo / "build" / "f").symlink_to(keep_file)
            (repo / "build" / "d").symlink_to(keep_dir)
            (repo / "out" / "link").symlink_to(cache)
            with unittest.mock.patch.object(rr, "ROOT", repo), \
                    unittest.mock.patch.dict("os.environ", env, clear=True):
                rr.Controller(rr.Config(), Path(tmp) / "state").revert(["out/link"])
                self.assertFalse(rr.os.path.lexists(repo / "out" / "link"))
                self.assertEqual(cache.read_text(), "keep")
                rr.Controller(rr.Config(), Path(tmp) / "state").revert(["build/f", "build/d"])
            for link in ("build/f", "build/d"):
                self.assertFalse(rr.os.path.lexists(repo / link))
            self.assertTrue(keep_file.exists())
            self.assertTrue((keep_dir / "HEAD").exists())

    def test_ignored_identity_sees_a_restored_mtime(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, env = self.repo(tmp, "build/\n")
            (repo / "build").mkdir()
            cache = repo / "build" / "cache.bin"
            cache.write_text("aaaa")
            with unittest.mock.patch.object(rr, "ROOT", repo), \
                    unittest.mock.patch.dict("os.environ", env, clear=True):
                controller = rr.Controller(rr.Config(), Path(tmp) / "state")
                before = controller.ignored_files()
                stat = cache.stat()
                rr.time.sleep(0.01)
                cache.write_text("bbbb")  # same size
                rr.os.utime(cache, ns=(stat.st_atime_ns, stat.st_mtime_ns))  # mtime restored, ctime not
                self.assertEqual(controller.ignored_delta(before), ([], ["build/cache.bin"]))

    @contextlib.contextmanager
    def live_state(self, tmp, active=()):
        """The real layout: controller state inside the checkout, the baseline outside it."""
        repo, env = self.repo(tmp, "build/\n")
        state, home = repo / "build" / "roadmap-pipeline", Path(tmp) / "xdg-state"
        state.mkdir(parents=True)
        ledger = {"tasks": [dict(task(t), status="active") for t in active]}
        with unittest.mock.patch.object(rr, "ROOT", repo), unittest.mock.patch.object(rr, "STATE", state), \
                unittest.mock.patch.object(rr, "baseline_home", return_value=home), \
                unittest.mock.patch.object(rr, "load_ledger", return_value=ledger), \
                unittest.mock.patch.dict("os.environ", env, clear=True), \
                unittest.mock.patch.object(rr, "planted_modules", return_value=[]), \
                unittest.mock.patch.object(rr, "git_callbacks", return_value="cb"):
            yield repo, state, rr.Controller(rr.Config(), state)

    def resume(self, *extra):
        """`resume` through the CLI: (exit code, printed message)."""
        with unittest.mock.patch.object(rr, "isolate_imports"), \
                unittest.mock.patch.object(rr.sys, "argv", ["roadmap_runner.py", "resume", *extra]), \
                contextlib.redirect_stdout(io.StringIO()) as out:
            code = rr.main()
        return code, out.getvalue()

    def accept(self):
        """The lead's review step: `resume` reports the change and its digest; accept exactly that."""
        code, message = self.resume()
        self.assertEqual(code, 0)
        digest = re.search(r"resume --accept ([0-9a-f]+)", message)
        self.assertIsNotNone(digest, message)
        return self.resume("--accept", digest.group(1))

    def test_saved_baseline_covers_ignored_files_until_resume(self):
        for session in ("manual A", "crashed worker"):
            with self.subTest(session=session), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller):
                rr.atomic_write(controller.marker(), json.dumps(
                    {"baseline": "cb", "session": session, "ignored": controller.ignored_files()}))
                controller.check_saved_baseline()  # unchanged: passes
                (repo / "build" / "dropped.py").write_text("")
                with self.assertRaisesRegex(rr.Stop, "ignored files changed"):
                    controller.check_saved_baseline()
                self.assertTrue((state / "STOP").exists())
                self.assertEqual(self.resume()[0], 0)
                self.assertFalse((state / "STOP").exists())
                self.assertTrue(controller.marker().exists())  # not retired over an unreviewed change
                self.assertEqual(self.accept()[0], 0)
                self.assertNotIn("manual", json.loads(controller.marker().read_text()))  # the reviewed state
                controller.check_saved_baseline()  # is the new baseline: not re-fired

    def test_session_baseline_lies_outside_every_session_writable_root(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp) / "repo"
            with unittest.mock.patch.object(rr, "ROOT", repo):
                for writable in (repo / "build", Path(tempfile.gettempdir()) / "x", Path("/tmp/x")):
                    with self.subTest(xdg=writable), unittest.mock.patch.dict("os.environ", {"XDG_STATE_HOME": str(writable)}):
                        with self.assertRaisesRegex(rr.Stop, "session-writable"):
                            rr.baseline_home()
                with unittest.mock.patch.dict("os.environ", {"XDG_STATE_HOME": "/var/lib/lead"}):
                    self.assertEqual(rr.baseline_home().parent, Path("/var/lib/lead/baryonos-roadmap-pipeline"))
                self.assertEqual(rr.Controller(rr.Config()).baseline, None)  # the real state uses baseline_home

    def test_a_session_cannot_delete_or_forge_its_baseline(self):
        for tamper in ("delete", "forge"):
            with self.subTest(tamper=tamper), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller):
                controller.callbacks = "cb"
                controller.preflight = lambda: "a" * 40  # the claim itself, without a real ledger
                with unittest.mock.patch.object(rr, "select_task", return_value=task("A")), \
                        unittest.mock.patch.object(rr, "update_task"):
                    controller.lock_acquired = True
                    controller.prepare_manual()
                self.assertNotIn(repo.resolve(), controller.marker().resolve().parents)
                # The manual session writes everything under the checkout, build/roadmap-pipeline included.
                (repo / "build" / "evil.sh").write_text("")
                for leftover in state.glob("*.json"):
                    leftover.unlink()
                if tamper == "forge":
                    rr.atomic_write(state / rr.CALLBACKS_MARKER, json.dumps(
                        {"baseline": "cb", "session": "manual A", "ignored": controller.ignored_files()}))
                with self.assertRaisesRegex(rr.Stop, "ignored files changed|lies inside the checkout"):
                    rr.Controller(rr.Config(), state).check_saved_baseline()
                self.assertTrue((state / "STOP").exists())

    def test_resume_keeps_the_baseline_while_a_session_may_still_run(self):
        for blocker in ("active manual claim", "running controller"):
            with self.subTest(blocker=blocker), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp, active=("A",) if blocker == "active manual claim" else ()) \
                    as (repo, state, controller):
                rr.atomic_write(controller.marker(), json.dumps(
                    {"baseline": "cb", "session": "manual A", "ignored": controller.ignored_files()}))
                (state / "STOP").write_text("x")
                holder = (state / "controller.lock").open("a+")
                if blocker == "running controller":
                    rr.fcntl.flock(holder, rr.fcntl.LOCK_EX | rr.fcntl.LOCK_NB)
                try:
                    self.assertEqual(self.resume()[0], 0)
                finally:
                    holder.close()
                self.assertFalse((state / "STOP").exists())  # a paused or stopped run may continue
                self.assertTrue(controller.marker().exists())
                (repo / "build" / "post-resume.sh").write_text("")  # the session keeps running
                with self.assertRaisesRegex(rr.Stop, "ignored files changed"):
                    controller.check_saved_baseline()

    def test_session_keeps_its_baseline_until_the_caller_validated_it(self):
        for outcome in ("returns", "rate limit"):
            with self.subTest(outcome=outcome), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                controller = rr.Controller(rr.Config(), root / "state")
                controller.callbacks = "cb"
                proc = unittest.mock.Mock(pid=2 ** 22 + 12345)
                proc.wait.return_value = 0 if outcome == "returns" else 1

                def spawn(*args, stderr, **kwargs):
                    if outcome == "rate limit":
                        stderr.write("429 too many requests")
                    return proc
                with unittest.mock.patch.object(rr, "require_pipeline_release"), \
                        unittest.mock.patch.object(rr.subprocess, "Popen", side_effect=spawn), \
                        unittest.mock.patch.object(rr.os, "killpg"), \
                        unittest.mock.patch.object(controller, "heartbeat"), \
                        unittest.mock.patch.object(controller, "transcript_check", return_value={}), \
                        unittest.mock.patch.object(controller, "ignored_files", return_value={}), \
                        unittest.mock.patch.object(rr, "git_callbacks", return_value="cb"):
                    if outcome == "returns":
                        controller.session("worker", "A", root, "brief", {}, [], "default", 10)
                    else:
                        with self.assertRaisesRegex(rr.Stop, "rate or usage limit"):
                            controller.session("worker", "A", root, "brief", {}, [], "default", 10)
                # Retired only by the caller after its ignored-file and nested .git checks.
                self.assertTrue(controller.marker().exists())

    def test_run_task_retires_the_worker_baseline_only_after_validation(self):
        for altered in (False, True):
            with self.subTest(altered=altered), tempfile.TemporaryDirectory() as tmp:
                repo, env = self.repo(tmp, "build/\n")
                (repo / "build").mkdir()
                (repo / "build" / "cache.bin").write_text("a")
                controller = rr.Controller(rr.Config(max_repairs=0), Path(tmp) / "state")

                def worker(*args, **kwargs):
                    rr.atomic_write(controller.marker(), "{}")
                    if altered:
                        (repo / "build" / "cache.bin").write_text("changed")
                    return {"exit_code": 0, "is_error": False, "output": {"state": "blocked"}}
                with unittest.mock.patch.object(rr, "ROOT", repo), \
                        unittest.mock.patch.dict("os.environ", env, clear=True), \
                        unittest.mock.patch.object(rr, "require_pipeline_release"), \
                        unittest.mock.patch.object(controller, "preflight", return_value="a" * 40), \
                        unittest.mock.patch.object(controller, "heartbeat"), \
                        unittest.mock.patch.object(controller, "check_controls"), \
                        unittest.mock.patch.object(controller, "session", side_effect=worker):
                    with self.assertRaisesRegex(rr.Stop if altered else rr.Blocked,
                                                "changed or removed ignored" if altered else "reported blocked"):
                        controller.run_task(task("A"))
                self.assertEqual(controller.marker().exists(), altered)

    def test_init_check_cleanliness_covers_ignored_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, env = self.repo(tmp, "build/\n")
            controller = rr.Controller(rr.Config(), Path(tmp) / "state")

            def init(*args, **kwargs):
                rr.atomic_write(controller.marker(), "{}")
                (repo / "build").mkdir()
                (repo / "build" / "x").write_text("x")
                return {"session_id": "s", "exit_code": 0}
            with unittest.mock.patch.object(rr, "ROOT", repo), \
                    unittest.mock.patch.dict("os.environ", env, clear=True), \
                    unittest.mock.patch.object(controller, "preflight", return_value="a" * 40), \
                    unittest.mock.patch.object(controller, "session", side_effect=init):
                self.assertFalse(controller.init_check()["checkout_clean_after"])
            self.assertTrue(controller.marker().exists())  # the next run compares with it

    def test_init_check_retires_its_baseline_only_when_the_checkout_stayed_clean(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, env = self.repo(tmp, "build/\n")
            controller = rr.Controller(rr.Config(), Path(tmp) / "state")

            def init(*args, **kwargs):
                rr.atomic_write(controller.marker(), "{}")
                return {"session_id": "s", "exit_code": 0}
            with unittest.mock.patch.object(rr, "ROOT", repo), \
                    unittest.mock.patch.dict("os.environ", env, clear=True), \
                    unittest.mock.patch.object(controller, "preflight", return_value="a" * 40), \
                    unittest.mock.patch.object(controller, "session", side_effect=init):
                self.assertTrue(controller.init_check()["checkout_clean_after"])
            self.assertFalse(controller.marker().exists())

            def hiding_init(*args, **kwargs):
                rr.atomic_write(controller.marker(), "{}")
                rr.git("update-index", "--skip-worktree", ".gitignore")
                (repo / ".gitignore").write_text("everything/\n")
                return {"session_id": "s", "exit_code": 0}
            with unittest.mock.patch.object(rr, "ROOT", repo), \
                    unittest.mock.patch.dict("os.environ", env, clear=True), \
                    unittest.mock.patch.object(controller, "preflight", return_value="a" * 40), \
                    unittest.mock.patch.object(controller, "session", side_effect=hiding_init):
                self.assertFalse(controller.init_check()["checkout_clean_after"])
            self.assertTrue(controller.marker().exists())

    def test_resume_never_retires_a_baseline_over_an_unacknowledged_change(self):
        for case in ("leftover after a Stop", "forged legacy marker", "forged STOP", "wrong digest"):
            with self.subTest(case=case), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller):
                rr.atomic_write(controller.marker(), json.dumps(
                    {"baseline": "cb", "session": "worker", "ignored": controller.ignored_files()}))
                (repo / "build" / "planted.sh").write_text("")  # what the session left behind
                if case == "forged legacy marker":
                    rr.atomic_write(state / rr.CALLBACKS_MARKER, json.dumps(
                        {"baseline": "cb", "session": "x", "ignored": controller.ignored_files()}))
                    with self.assertRaisesRegex(rr.Stop, "planted.sh"):  # the trusted delta is reported first
                        controller.check_saved_baseline()
                if case == "forged STOP":
                    (state / "STOP").write_text("x")
                code, message = self.resume("--accept", "0" * 16) if case == "wrong digest" else self.resume()
                self.assertEqual(code, 0)
                self.assertIn("planted.sh", message)
                self.assertTrue(controller.marker().exists())
                with self.assertRaisesRegex(rr.Stop, "ignored files changed"):
                    rr.Controller(rr.Config(), state).check_saved_baseline()

    def test_resume_retires_an_untrusted_legacy_baseline_only_on_acknowledgement(self):
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            rr.atomic_write(state / rr.CALLBACKS_MARKER, json.dumps(
                {"baseline": "cb", "session": "manual A", "ignored": controller.ignored_files()}))
            with self.assertRaisesRegex(rr.Stop, "lies inside the checkout"):
                controller.check_saved_baseline()
            self.assertTrue((state / "STOP").exists())
            self.assertIn('"no_trusted_baseline": true', self.resume()[1])  # no delta from session data
            self.assertTrue((state / rr.CALLBACKS_MARKER).exists())
            self.assertEqual(self.accept()[0], 0)
            self.assertFalse((state / rr.CALLBACKS_MARKER).exists())
            controller.check_saved_baseline()  # no longer re-fires

    def test_resume_refuses_while_a_tagged_worker_is_alive(self):
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            rr.atomic_write(controller.marker(), json.dumps(
                {"baseline": "cb", "session": "worker", "ignored": controller.ignored_files()}))
            with unittest.mock.patch.object(rr.Controller, "refuse_live_workers",
                                            side_effect=rr.Stop("a previous worker process is still alive")):
                code, message = self.resume()
            self.assertEqual(code, 2)
            self.assertIn("still alive", message)
            self.assertTrue(controller.marker().exists())

    def test_verifier_baseline_is_retired_only_after_its_fingerprint_check(self):
        for tampers in (False, True):
            with self.subTest(tampers=tampers), tempfile.TemporaryDirectory() as tmp:
                repo, env = self.repo(tmp, "build/\n")
                controller = rr.Controller(rr.Config(max_repairs=0), Path(tmp) / "state")

                def session(kind, *args, **kwargs):
                    rr.atomic_write(controller.marker(), "{}")
                    if kind == "worker":
                        (repo / "services" / "x").mkdir(parents=True)
                        (repo / "services" / "x" / "a.txt").write_text("a")
                        return {"exit_code": 0, "is_error": False, "output": {"state": "done"}}
                    if tampers:
                        (repo / "services" / "x" / "a.txt").write_text("verifier edit")
                    return {"exit_code": 0, "is_error": False, "output": {"verdict": "pass", "findings": []}}
                with unittest.mock.patch.object(rr, "ROOT", repo), \
                        unittest.mock.patch.dict("os.environ", env, clear=True), \
                        unittest.mock.patch.object(rr, "require_pipeline_release"), \
                        unittest.mock.patch.object(controller, "preflight", return_value="a" * 40), \
                        unittest.mock.patch.object(controller, "heartbeat"), \
                        unittest.mock.patch.object(controller, "check_controls"), \
                        unittest.mock.patch.object(controller, "run_checks", return_value=(True, [], "")), \
                        unittest.mock.patch.object(controller, "checkpoint", return_value="committed"), \
                        unittest.mock.patch.object(controller, "session", side_effect=session):
                    if tampers:
                        with self.assertRaisesRegex(rr.Stop, "verifier changed the working tree"):
                            controller.run_task(task("A"))
                    else:
                        self.assertEqual(controller.run_task(task("A")), "committed")
                self.assertEqual(controller.marker().exists(), tampers)

    def test_nested_git_reports_an_unreadable_directory(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "locked").mkdir()
            (root / "locked").chmod(0)
            try:
                if os.access(root / "locked", os.R_OK):
                    self.skipTest("running with privileges that read any directory")
                self.assertEqual(rr.nested_git(root), ["unreadable: locked"])
            finally:
                (root / "locked").chmod(0o700)

    def preflight(self, repo, controller):
        """A real preflight in a live_state checkout (no worker probe, the checkout's own branch)."""
        branch = rr.subprocess.run(["git", "-C", str(repo), "branch", "--show-current"], check=True,
                                   capture_output=True, text=True).stdout.strip()
        controller.cfg = rr.Config(branch=branch)
        with unittest.mock.patch.object(rr.Controller, "refuse_live_workers"), \
                unittest.mock.patch.object(rr, "planted_modules", return_value=[]):
            return controller.preflight()

    def manual_marker(self, controller, task_id="L01-T3"):
        rr.atomic_write(controller.marker(), json.dumps({"baseline": "cb", "manual": task_id,
                                                         "session": f"manual {task_id}",
                                                         "ignored": controller.ignored_files()}))

    def test_a_manual_claim_outlives_a_ledger_the_session_rewrote(self):
        # 3a24e2c4 read the claim from the in-checkout ledger only: the session marks its task done,
        # preflight retires the baseline, and what it plants afterwards is adopted.
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            self.manual_marker(controller)  # the ledger (patched) already says nothing is active
            with self.assertRaisesRegex(rr.Stop, "manual task L01-T3 holds the session baseline"):
                self.preflight(repo, controller)
            self.assertTrue(controller.marker().exists())
            with unittest.mock.patch.object(rr.Controller, "refuse_live_workers"):
                code, message = self.resume()  # the delta is still empty, yet the session may run on
            self.assertEqual(code, 0)
            self.assertIn("Confirm that the manual session has ended", message)
            self.assertTrue(controller.marker().exists())
            with unittest.mock.patch.object(rr.Controller, "refuse_live_workers"):
                self.assertEqual(self.accept()[0], 0)
            self.assertIsNone(controller.manual_claim())  # the acknowledgement ended the claim
            self.assertEqual(self.preflight(repo, controller), rr.git("rev-parse", "HEAD").strip())
            self.assertFalse(controller.marker().exists())  # retired by the preflight that passed

    def test_preflight_keeps_the_baseline_until_every_check_passed(self):
        # 3a24e2c4 retired before the nested .git check: the lead removes sub/.git as told, and the
        # ignored payload Git could not see inside sub/ is adopted on the next run.
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            (repo / ".gitignore").write_text("build/\ntarget/\n")
            rr.git("commit", "-q", "--no-verify", "-am", "ignore target")
            rr.atomic_write(controller.marker(), json.dumps(
                {"baseline": "cb", "session": "worker", "ignored": controller.ignored_files()}))
            rr.subprocess.run(["git", "init", "-q", str(repo / "sub")], check=True)
            (repo / "sub" / "target" / "release").mkdir(parents=True)
            (repo / "sub" / "target" / "release" / "tool").write_text("payload")
            with self.assertRaisesRegex(rr.Stop, "nested .git"):
                self.preflight(repo, controller)
            self.assertTrue(controller.marker().exists())
            with unittest.mock.patch.object(rr.Controller, "refuse_live_workers"):
                code, message = self.resume()  # nor does `resume` retire it over the invisible payload
            self.assertEqual(code, 2)
            self.assertIn("nested .git", message)
            self.assertTrue(controller.marker().exists())
            shutil.rmtree(repo / "sub" / ".git")  # exactly what the Stop asked for
            with self.assertRaisesRegex(rr.Stop, "sub/target/release/tool"):
                self.preflight(repo, controller)
            self.assertTrue(controller.marker().exists())
            shutil.rmtree(repo / "sub")
            (state / "STOP").unlink()
            (repo / "dirty.txt").write_text("x")  # a later check fails: the baseline still stays
            with self.assertRaisesRegex(rr.Stop, "not clean"):
                self.preflight(repo, controller)
            self.assertTrue(controller.marker().exists())
            (repo / "dirty.txt").unlink()
            rr.git("checkout", "-q", "-b", "elsewhere")
            with unittest.mock.patch.object(rr.Controller, "refuse_live_workers"), \
                    unittest.mock.patch.object(rr, "planted_modules", return_value=[]):
                controller.cfg = rr.Config(branch="main-only")
                with self.assertRaisesRegex(rr.Stop, "expected 'main-only'"):
                    controller.preflight()
            self.assertTrue(controller.marker().exists())

    def test_a_manual_claim_in_the_previous_marker_format_still_needs_acknowledgement(self):
        # 3a24e2c4 wrote the claim only as the session name: an empty delta must not retire it.
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller), \
                unittest.mock.patch.object(rr.Controller, "refuse_live_workers"):
            rr.atomic_write(controller.marker(), json.dumps(
                {"baseline": "cb", "session": "manual L01-T3", "ignored": controller.ignored_files()}))
            self.assertEqual(controller.manual_claim(), "L01-T3")
            code, message = self.resume()
            self.assertIn("Confirm that the manual session has ended", message)
            self.assertTrue(controller.marker().exists())

    def test_accept_digest_binds_the_reviewed_content(self):
        # 3a24e2c4 hashed only the path lists: a reviewed digest also accepted a later, different change.
        for swap in ("content", "hooks", "tracked edit", "HEAD moved"):
            with self.subTest(swap=swap), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller), \
                    unittest.mock.patch.object(rr, "git_callbacks", side_effect=REAL_GIT_CALLBACKS), \
                    unittest.mock.patch.object(rr.Controller, "refuse_live_workers"):
                # The hooks case starts from callbacks that already differ, so `callbacks_changed`
                # stays true: only binding the fingerprint itself catches the second change.
                saved = "other" if swap == "hooks" else rr.git_callbacks()
                rr.atomic_write(controller.marker(), json.dumps(
                    {"baseline": saved, "session": "worker", "ignored": controller.ignored_files()}))
                (repo / "build" / "tool.py").write_text("print('reviewed')\n")
                reviewed = re.search(r"resume --accept ([0-9a-f]+)", self.resume()[1]).group(1)
                if swap == "content":
                    (repo / "build" / "tool.py").write_text("print('swapped')\n")
                elif swap == "hooks":
                    rr.git("config", "core.hooksPath", str(repo / "build"))
                elif swap == "tracked edit":  # the delta shows ignored files only; the digest binds the tree
                    (repo / ".gitignore").write_text("build/\n# edited after the review\n")
                else:  # an unchanged worktree on another commit
                    rr.git("commit", "-q", "--allow-empty", "--no-verify", "-m", "moved")
                code, message = self.resume("--accept", reviewed)
                self.assertIn("is kept", message)
                self.assertTrue(controller.marker().exists())

    def test_a_change_after_the_reviewed_read_is_never_adopted_unseen(self):
        # 626b5b76 read the checkout again for the digest and the snapshot: a plant between the
        # reads was saved as reviewed, and a listed file rewritten after the digest was accepted.
        for window in ("before the digest", "after the digest", "accepted file rewritten"):
            with self.subTest(window=window), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller), \
                    unittest.mock.patch.object(rr.Controller, "refuse_live_workers"):
                rr.atomic_write(controller.marker(), json.dumps(
                    {"baseline": "cb", "session": "worker", "ignored": controller.ignored_files()}))
                late = repo / "build" / ("tool.py" if window == "accepted file rewritten" else "late.py")
                if window == "accepted file rewritten":
                    late.write_text("print('reviewed')\n")
                real_digest, real_retire = rr.Controller.review_digest, rr.Controller.retire_baseline

                def digest(self, review, read):
                    if window == "before the digest":
                        late.write_text("planted before the digest")
                    value = real_digest(self, review, read)
                    if window == "accepted file rewritten" and review.get("created"):
                        late.write_text("print('swapped')\n")
                    return value

                def retire(self):
                    if window == "after the digest":
                        late.write_text("planted after the digest")
                    real_retire(self)
                if window == "accepted file rewritten":
                    reviewed = re.search(r"resume --accept ([0-9a-f]+)", self.resume()[1]).group(1)
                with unittest.mock.patch.object(rr.Controller, "review_digest", digest), \
                        unittest.mock.patch.object(rr.Controller, "retire_baseline", retire):
                    extra = ("--accept", reviewed) if window == "accepted file rewritten" else ()
                    self.assertIn("is now the saved session baseline", self.resume(*extra)[1])
                saved = json.loads(controller.marker().read_text())["ignored"]
                self.assertNotEqual(saved.get(late.relative_to(repo).as_posix()), controller.ignored_files()[
                    late.relative_to(repo).as_posix()])
                self.assertIn(late.relative_to(repo).as_posix(), self.resume()[1])  # shown, not adopted

    def test_digest_exists_only_after_the_lead_ran_resume(self):
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller), \
                unittest.mock.patch.object(rr.Controller, "refuse_live_workers"):
            rr.atomic_write(state / rr.CALLBACKS_MARKER, "{}")  # a forged legacy marker
            first = re.search(r"resume --accept ([0-9a-f]+)", self.resume()[1]).group(1)
            (controller.marker().parent / rr.REVIEW_NONCE).unlink()  # another review round
            second = re.search(r"resume --accept ([0-9a-f]+)", self.resume()[1]).group(1)
            self.assertNotEqual(first, second)  # no constant, precomputable digest
            self.assertIn("is kept", self.resume("--accept", first)[1])

    def test_a_deleted_legacy_baseline_leaves_no_trusted_history(self):
        # 3a24e2c4: deleting the in-checkout marker gave no Stop, and the plants were adopted.
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            with self.assertRaisesRegex(rr.Stop, "no trusted session baseline"):
                self.preflight(repo, controller)
            with unittest.mock.patch.object(rr.Controller, "refuse_live_workers"):
                self.assertIn('"no_trusted_baseline": true', self.resume()[1])
                self.assertEqual(self.accept()[0], 0)
            self.assertTrue((controller.marker().parent / rr.TRUSTED_RECORD).exists())
            self.preflight(repo, controller)

    def test_index_flags_cannot_hide_a_tracked_edit(self):
        for flag in ("--skip-worktree", "--assume-unchanged"):
            with self.subTest(flag=flag), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller):
                rr.atomic_write(controller.marker().parent / rr.TRUSTED_RECORD, "{}")
                (repo / "gate.py").write_text("ok\n")
                rr.git("add", "gate.py")
                rr.git("commit", "-q", "--no-verify", "-m", "gate")
                before = controller.tree_fingerprint()
                rr.git("update-index", flag, "gate.py")
                (repo / "gate.py").write_text("evil\n")
                self.assertEqual(rr.git("status", "--porcelain"), "")  # invisible to status
                self.assertNotEqual(controller.tree_fingerprint(), before)
                with self.assertRaisesRegex(rr.Stop, "hide worktree edits.*gate.py"):
                    self.preflight(repo, controller)

    def test_run_task_refuses_a_worker_that_hid_a_tracked_edit(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, env = self.repo(tmp)
            controller = rr.Controller(rr.Config(max_repairs=0), Path(tmp) / "state")

            def worker(*args, **kwargs):
                rr.git("update-index", "--skip-worktree", ".gitignore")
                (repo / ".gitignore").write_text("everything/\n")  # out of scope, invisible to status
                return {"exit_code": 0, "is_error": False, "output": {"state": "implemented"}}
            with unittest.mock.patch.object(rr, "ROOT", repo), \
                    unittest.mock.patch.object(rr, "require_pipeline_release"), \
                    unittest.mock.patch.dict("os.environ", env, clear=True), \
                    unittest.mock.patch.object(controller, "preflight", return_value="a" * 40), \
                    unittest.mock.patch.object(controller, "heartbeat"), \
                    unittest.mock.patch.object(controller, "session", side_effect=worker), \
                    unittest.mock.patch.object(controller, "run_checks") as checks:
                with self.assertRaisesRegex(rr.Stop, "hid tracked edits.*gitignore"):
                    controller.run_task(task("A"))
                checks.assert_not_called()

    def test_a_symlinked_state_component_cannot_lead_the_baseline_into_the_checkout(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, state = Path(tmp) / "repo", Path(tmp) / "state"
            (repo / "build").mkdir(parents=True)
            state.mkdir()
            (state / "baryonos-roadmap-pipeline").symlink_to(repo / "build")
            with unittest.mock.patch.object(rr, "ROOT", repo), \
                    unittest.mock.patch.dict("os.environ", {"XDG_STATE_HOME": str(state)}):
                with self.assertRaisesRegex(rr.Stop, f"session-writable {re.escape(str(repo.resolve()))};"):
                    rr.baseline_home()  # named as the checkout, not just as part of /tmp

    def test_a_planted_symlink_cannot_make_the_controller_write_the_trusted_home(self):
        # ba9eab6a: `resume` opened build/roadmap-pipeline/controller.lock with open('a+'), so a link
        # to <home>/trusted-checkout.json created the trust record without any --accept.
        for planted in ("controller.lock", "hardlinked controller.lock", "fifo controller.lock", "STOP via `stop`",
                        "atomic tmp"):
            with self.subTest(planted=planted), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller), \
                    unittest.mock.patch.object(rr.Controller, "refuse_live_workers"):
                home = controller.marker().parent
                home.mkdir(parents=True)
                record = home / rr.TRUSTED_RECORD
                if planted == "controller.lock":
                    (state / "controller.lock").symlink_to(record)
                    self.assertEqual(self.resume()[0], 2)
                elif planted == "hardlinked controller.lock":
                    rr.atomic_write(controller.marker(), json.dumps({"manual": "A"}))
                    os.link(controller.marker(), state / "controller.lock")
                    self.assertEqual(self.resume()[0], 2)
                    self.assertEqual(controller.manual_claim(), "A")  # not truncated
                elif planted == "fifo controller.lock":
                    os.mkfifo(state / "controller.lock")
                    self.assertEqual(self.resume()[0], 2)
                elif planted == "STOP via `stop`":
                    (state / "STOP").symlink_to(record)
                    with unittest.mock.patch.object(rr, "isolate_imports"), \
                            unittest.mock.patch.object(rr.sys, "argv", ["roadmap_runner.py", "stop"]), \
                            contextlib.redirect_stdout(io.StringIO()):
                        rr.main()
                    self.assertFalse((state / "STOP").is_symlink())  # replaced, not followed
                else:
                    target = state / "x.json"
                    target.with_suffix(f".json.tmp{os.getpid()}").symlink_to(record)
                    with self.assertRaisesRegex(rr.Stop, "planted symlink"):
                        rr.atomic_write(target, "{}")
                self.assertFalse(record.exists())
                self.assertFalse(controller.trusted() and planted != "hardlinked controller.lock")

    def test_the_accepted_state_stays_the_baseline_until_a_preflight_passes(self):
        # ba9eab6a retired the baseline on --accept: a plant made after it was adopted unseen.
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller), \
                unittest.mock.patch.object(rr.Controller, "refuse_live_workers"):
            self.manual_marker(controller)
            self.assertEqual(self.accept()[0], 0)
            (repo / "build" / "late.py").write_text("planted after the acknowledgement")
            with self.assertRaisesRegex(rr.Stop, "build/late.py"):
                self.preflight(repo, controller)
            self.assertTrue(controller.marker().exists())

    @staticmethod
    def forge_index_stat(repo, name):
        """What write access to .git/index allows: record the edited file's stat for its old blob,
        so Git's stat cache calls a same-size edit clean (index v2, trailing SHA-1 recomputed)."""
        index = repo / ".git" / "index"
        data = bytearray(index.read_bytes()[:-20])
        assert data[:4] == b"DIRC" and int.from_bytes(data[4:8], "big") == 2
        offset = 12
        for _ in range(int.from_bytes(data[8:12], "big")):
            length = int.from_bytes(data[offset + 60:offset + 62], "big") & 0xFFF
            if data[offset + 62:offset + 62 + length].decode() == name:
                path = repo / name
                os.utime(path, ns=(path.stat().st_atime_ns, path.stat().st_mtime_ns - 10 * 10**9))  # not racy
                info = path.stat()
                fields = (info.st_ctime_ns // 10**9, info.st_ctime_ns % 10**9, info.st_mtime_ns // 10**9,
                          info.st_mtime_ns % 10**9, info.st_dev, info.st_ino)
                for i, value in enumerate(fields):
                    data[offset + 4 * i:offset + 4 * i + 4] = (value & 0xFFFFFFFF).to_bytes(4, "big")
                data[offset + 36:offset + 40] = info.st_size.to_bytes(4, "big")
            offset += (62 + length + 8) // 8 * 8
        index.write_bytes(bytes(data) + hashlib.sha1(bytes(data)).digest())

    def test_a_forged_index_stat_cache_cannot_hide_a_tracked_edit(self):
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            rr.atomic_write(controller.marker().parent / rr.TRUSTED_RECORD, "{}")
            (repo / "gate.py").write_text("ok\n")
            rr.git("add", "gate.py")
            rr.git("commit", "-q", "--no-verify", "-m", "gate")
            before = controller.tree_fingerprint()
            (repo / "gate.py").write_text("ev\n")  # same size
            self.forge_index_stat(repo, "gate.py")
            self.assertEqual(rr.git("status", "--porcelain"), "")  # the forgery works on Git itself
            self.assertEqual(rr.git("diff", "HEAD", "--name-only"), "")
            self.assertEqual(rr.hidden_tracked(), [])  # no flag involved
            self.assertIn("gate.py", controller.changed())
            self.assertNotEqual(controller.tree_fingerprint(), before)
            with self.assertRaisesRegex(rr.Stop, "differ from HEAD.*gate.py"):
                self.preflight(repo, controller)

    def test_a_relative_xdg_state_home_is_ignored(self):
        # Resolved against the cwd (here the checkout), a relative value would put the baseline in
        # the checkout. Ignored, the home falls back to ~/.local/state: the containment check then
        # names /tmp (this test's HOME lies there), never the checkout, which it checks first.
        with tempfile.TemporaryDirectory() as tmp:
            repo, cwd = Path(tmp) / "repo", os.getcwd()
            repo.mkdir()
            os.chdir(repo)
            try:
                with unittest.mock.patch.dict("os.environ", {"XDG_STATE_HOME": "state", "HOME": str(Path(tmp) / "h")}), \
                        unittest.mock.patch.object(rr, "ROOT", repo):
                    with self.assertRaises(rr.Stop) as stopped:
                        rr.baseline_home()
            finally:
                os.chdir(cwd)
            self.assertNotIn(f"session-writable {repo.resolve()};", str(stopped.exception))
            self.assertIn(str(Path(tmp).resolve() / "h" / ".local" / "state"), str(stopped.exception))

    def test_worker_settings_deny_edits_to_the_baseline_home(self):
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp) / "xdg" / "baryonos-roadmap-pipeline" / "abc"
            controller = rr.Controller(rr.Config(), Path(tmp) / "state", baseline=home)
            with unittest.mock.patch.object(rr, "require_pipeline_release"), \
                    unittest.mock.patch.object(controller, "ignored_files", return_value={}), \
                    unittest.mock.patch.object(rr.subprocess, "Popen", side_effect=OSError("stop here")):
                with contextlib.suppress(Exception):
                    controller.session("worker", "A", Path(tmp), "brief", {}, [], "default", 10)
            deny = json.loads((Path(tmp) / "worker-settings.json").read_text())["permissions"]["deny"]
            self.assertIn(f"Edit(/{home.parent}/**)", deny)

    def test_push_runs_no_pre_push_hook(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, env = self.repo(tmp)
            remote = Path(tmp) / "remote.git"
            rr.subprocess.run(["git", "init", "-q", "--bare", str(remote)], check=True, env=env)
            rr.subprocess.run(["git", "-C", str(repo), "remote", "add", "origin", str(remote)], check=True, env=env)
            marker = Path(tmp) / "HOOK-RAN"
            hook = repo / ".git" / "hooks" / "pre-push"
            hook.write_text(f"#!/bin/sh\ntouch {marker}\nexit 1\n")
            hook.chmod(0o755)
            branch = rr.subprocess.run(["git", "-C", str(repo), "branch", "--show-current"], check=True,
                                       env=env, capture_output=True, text=True).stdout.strip()
            with unittest.mock.patch.object(rr, "ROOT", repo), \
                    unittest.mock.patch.dict("os.environ", env, clear=True):
                rr.Controller(rr.Config(push=True, branch=branch), Path(tmp) / "state").push()
            self.assertFalse(marker.exists())
            heads = rr.subprocess.run(["git", "-C", str(remote), "branch"], check=True, env=env,
                                      capture_output=True, text=True).stdout
            self.assertIn(branch, heads)  # the push itself succeeded


if __name__ == "__main__":
    unittest.main()

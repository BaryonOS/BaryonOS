import contextlib
import io
import os
import re
import hashlib
import json
import shutil
import sys
import pty
import shlex
import signal
import tempfile
import time
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
            # Patched: pgrep also sees tagged processes of a concurrent run on the host.
            with unittest.mock.patch.object(rr, "LEDGER", ledger), \
                    unittest.mock.patch.object(rr.Controller, "refuse_live_workers"), \
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
                    # The lead is given the launcher, never a bare codex command.
                    launcher = first["argv"].index("manual-run")
                    self.assertEqual(first["argv"][launcher + 1:launcher + 3], ["--", "codex"])
                    self.assertTrue(first["launcher_required"])
                    marker = json.loads((root / "state" / rr.CALLBACKS_MARKER).read_text())
                    self.assertEqual(marker["baseline"], "cb")  # the next preflight compares, never adopts
                    self.assertEqual(marker["manual"], "A")  # the claim the in-checkout ledger cannot end
                    self.assertTrue(marker["launcher"])
                    # Bound to the planned command: manual-run refuses any other.
                    self.assertEqual(marker["argv_sha256"], rr.sha256(json.dumps(first["argv"][launcher + 2:])))
                    self.assertIn(marker["claim"], first["stdin"])
                    # The planned argv survives the CLI (REMAINDER and the `--` strip) byte for byte.
                    with unittest.mock.patch.object(rr, "isolate_imports"), \
                            unittest.mock.patch.object(rr, "pin_state"), \
                            unittest.mock.patch.object(rr.Controller, "manual_run", return_value=0) as run, \
                            unittest.mock.patch.object(rr.sys, "argv", ["roadmap_runner.py", *first["argv"][3:]]):
                        self.assertEqual(rr.main(), 0)
                    run.assert_called_once_with(first["argv"][launcher + 2:])
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
                            unittest.mock.patch.object(rr.Controller, "refuse_live_workers"):
                        with self.assertRaisesRegex(rr.Stop, "manual task A has no `manual-run` completion"):
                            controller.prepare_manual()
                        rr.atomic_write(root / "state" / rr.MANUAL_RECORD, json.dumps(
                            {"claim": marker["claim"], "exit": 0}))  # what the launcher writes after ECHILD
                        with self.assertRaisesRegex(rr.Stop, "manual task A holds the session baseline"):
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
        with tempfile.TemporaryDirectory() as tmp, unittest.mock.patch.object(rr.Controller, "refuse_live_workers"):
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
                            unittest.mock.patch.object(rr.Controller, "refuse_live_workers"), \
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
            try:  # other tagged processes on the host (a concurrent run) may remain; this one may not
                rr.Controller(rr.Config(), Path(tmp)).refuse_live_workers()
            except rr.Stop as error:
                self.assertNotIn(f"'{worker.pid}'", str(error))

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
                pipeline = repo / "build" / "agent-pipeline" / "t1" / "worktree"
                pipeline.mkdir(parents=True)
                (pipeline / ".git").write_text("gitdir: /elsewhere\n")
                return {"exit_code": 0, "is_error": False, "output": {"state": "implemented"}}
            with unittest.mock.patch.object(rr, "ROOT", repo), \
                    unittest.mock.patch.object(rr, "require_pipeline_release"), \
                    unittest.mock.patch.dict("os.environ", env, clear=True), \
                    unittest.mock.patch.object(controller, "preflight", return_value="a" * 40), \
                    unittest.mock.patch.object(controller, "heartbeat"), \
                    unittest.mock.patch.object(controller, "session", side_effect=worker), \
                    unittest.mock.patch.object(controller, "run_checks") as checks:
                before = None
                with self.assertRaisesRegex(rr.Stop, "nested .git") as refused:
                    controller.run_task(task("A"))
                checks.assert_not_called()
                self.assertIn(f"`git -C {shlex.quote(str(repo))} worktree remove build/agent-pipeline/t1/worktree`",
                              str(refused.exception))
                shutil.rmtree(repo / "services")
                shutil.rmtree(repo / "build" / "agent-pipeline")
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
                unittest.mock.patch.object(rr.Controller, "refuse_live_workers"), \
                unittest.mock.patch.object(rr, "git_callbacks", return_value="cb"):
            # refuse_live_workers patched: pgrep also sees tagged processes of a concurrent run on
            # the host. Tests of the probe itself patch it again with their own outcome.
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

    # --- residual A: a manual session's processes, which pgrep cannot see ---
    def launcher_marker(self, controller, argv=(), task_id="L01-T3", claim=None):
        # Fresh per test: the environment scan sees every process of this user, concurrent suites too.
        claim = claim or rr.uuid.uuid4().hex
        rr.atomic_write(controller.marker(), json.dumps(
            {"baseline": "cb", "manual": task_id, "claim": claim, "launcher": True,
             "argv_sha256": rr.sha256(json.dumps(list(argv))),  # what `brief` planned
             "session": f"manual {task_id}", "ignored": controller.ignored_files()}))
        return claim

    def launch(self, controller, argv, out=None, cli=False, prepare=None):
        """`manual-run` in a forked child that inherits the live_state patches, so the unittest
        process itself never becomes a subreaper or calls os.wait(). `prepare` runs in that child first."""
        pid = os.fork()
        if pid == 0:
            code = 99
            try:
                if prepare is not None:
                    prepare()
                if cli:
                    with unittest.mock.patch.object(rr, "isolate_imports"), \
                            unittest.mock.patch.object(rr.sys, "argv", ["roadmap_runner.py", "manual-run", "--", *argv]):
                        code = rr.main()
                else:
                    code = controller.manual_run(argv)
            except rr.Stop as error:
                code = 2
                if out is not None:
                    out.write_text(str(error))
            finally:
                os._exit(code)
        return pid

    def reap(self, pid, timeout=10.0):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            done, status = os.waitpid(pid, os.WNOHANG)
            if done:
                return os.waitstatus_to_exitcode(status)
            time.sleep(0.01)
        os.kill(pid, signal.SIGKILL)
        os.waitpid(pid, 0)
        self.fail("the launcher did not exit")

    @staticmethod
    def kill(pid):
        with contextlib.suppress(ProcessLookupError):
            os.kill(pid, signal.SIGKILL)
        return False

    def wait_for(self, predicate, timeout=10.0):
        deadline = time.monotonic() + timeout
        while not predicate():
            if time.monotonic() > deadline:
                self.fail("timed out")
            time.sleep(0.01)

    class Blocked(BaseException):
        """Not an OSError: refuse_live_manual must not turn the alarm into the Stop a test expects."""

    @contextlib.contextmanager
    def deadline(self, seconds=5):
        def expire(*_):
            raise self.Blocked("blocked on a planted launcher lock")
        previous = signal.signal(signal.SIGALRM, expire)
        signal.alarm(seconds)
        try:
            yield
        finally:
            signal.alarm(0)
            signal.signal(signal.SIGALRM, previous)

    @unittest.skipUnless(sys.platform == "linux" and shutil.which("setsid") and shutil.which("sleep"),
                         "PR_SET_CHILD_SUBREAPER and setsid are Linux-only")
    def test_a_detached_grandchild_holds_the_launcher_until_it_exits(self):
        # Residual A of #416: a double-forked, setsid'd grandchild with a cleared environment survives
        # its parent. pgrep (no tag) and the environment scan (env -i) both miss it; the launcher must not.
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            run, home = Path(tmp), controller.marker().parent
            # Bounded (about 10 s) and detached from the test's pipes, so a regression cannot hang the suite.
            script = (f"echo $$ > {run}/child.pid; ( setsid env -i /bin/sh -c 'echo $$ > {run}/grand.pid; i=0; "
                      f"while [ ! -e {run}/GO ] && [ $i -lt 500 ]; do {shutil.which('sleep')} 0.02; i=$((i+1)); "
                      "done' </dev/null >/dev/null 2>&1 & ); exit 3")
            claim = self.launcher_marker(controller, ["/bin/sh", "-c", script])
            # An earlier completed run of this claim: the new start must invalidate it.
            rr.atomic_write(home / rr.MANUAL_RECORD, json.dumps({"claim": claim, "exit": 0}))
            launcher, grand = self.launch(controller, ["/bin/sh", "-c", script]), None
            try:
                self.wait_for(lambda: (run / "grand.pid").exists() and (run / "child.pid").exists())
                self.wait_for(lambda: (run / "grand.pid").read_text().strip() != "")
                grand = int((run / "grand.pid").read_text())
                child = int((run / "child.pid").read_text())
                self.wait_for(lambda: not os.path.exists(f"/proc/{child}"))  # the direct child is reaped
                self.assertNotIn(rr.MANUAL_ENV.encode(), Path(f"/proc/{grand}/environ").read_bytes())
                self.assertEqual(os.waitpid(launcher, os.WNOHANG), (0, 0))  # still waiting for the grandchild
                self.assertNotIn("exit", json.loads((home / rr.MANUAL_RECORD).read_text()))
                with self.assertRaisesRegex(rr.Stop, "launcher is still running"):
                    controller.refuse_live_manual()
                with self.assertRaisesRegex(rr.Stop, "launcher is still running"):
                    self.preflight(repo, controller)
                code, message = self.resume()
                self.assertEqual(code, 2)
                self.assertIn("launcher is still running", message)
                # A second launcher for the same claim is refused and runs nothing.
                second = self.launch(controller, ["/bin/sh", "-c", f"touch {run}/RAN"], out=run / "second.out")
                self.assertEqual(self.reap(second), 2)
                self.assertIn("another `manual-run` launcher", (run / "second.out").read_text())
                self.assertFalse((run / "RAN").exists())
                (run / "GO").touch()
                self.assertEqual(self.reap(launcher), 3)  # the direct child's status
                launcher = None
            finally:
                (run / "GO").touch()
                if grand is not None:  # never left behind, whatever failed above
                    self.wait_for(lambda: not os.path.exists(f"/proc/{grand}") or self.kill(grand), 5)
                if launcher is not None:
                    with contextlib.suppress(ProcessLookupError, ChildProcessError):  # reaped by a failed check
                        os.kill(launcher, signal.SIGKILL)
                        os.waitpid(launcher, 0)
            record = json.loads((home / rr.MANUAL_RECORD).read_text())
            self.assertEqual((record["claim"], record["exit"]), (claim, 3))
            controller.refuse_live_manual()  # the refusal lifts
            with self.assertRaisesRegex(rr.Stop, "manual task L01-T3 holds the session baseline"):
                self.preflight(repo, controller)  # back to the claim itself
            code, message = self.resume()
            self.assertEqual(code, 0)
            self.assertIn("Confirm that the manual session has ended", message)
            self.assertEqual(self.accept()[0], 0)
            self.assertIsNone(controller.manual_claim())

    def test_a_launcher_claim_without_its_completion_record_is_refused(self):
        # A session started with a bare codex command leaves no lock: that must not read as "no process".
        for kind in ("none", "other claim", "started only"):
            with self.subTest(record=kind), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller):
                claim = self.launcher_marker(controller)
                record_path = controller.marker().parent / rr.MANUAL_RECORD
                record = {"none": None, "other claim": {"claim": "other", "exit": 0},
                          "started only": {"claim": claim, "started": "t"}}[kind]
                if record is not None:
                    rr.atomic_write(record_path, json.dumps(record))
                with self.assertRaisesRegex(rr.Stop, "no `manual-run` completion record") as refused:
                    self.preflight(repo, controller)
                # The remedy is the explicit acknowledgement, never a fresh launcher run.
                self.assertIn(f"`resume --accept-orphans {claim}`", str(refused.exception))
                self.assertNotIn("manual-run -- true", str(refused.exception))
                for extra in ((), ("--accept", "0" * 16)):
                    code, message = self.resume(*extra)
                    self.assertEqual(code, 2)
                    self.assertIn("no `manual-run` completion record", message)
                self.assertEqual(controller.manual_claim(), "L01-T3")
                rr.atomic_write(record_path, json.dumps({"claim": claim, "exit": 0}))
                with self.assertRaisesRegex(rr.Stop, "holds the session baseline"):
                    self.preflight(repo, controller)
                self.assertEqual(self.accept()[0], 0)
                self.assertIsNone(controller.manual_claim())

    def test_a_held_launcher_lock_is_refused_even_with_a_completion_record(self):
        # Also while the ledger still lists the task active, the normal state right after `brief`.
        for active in ((), ("L01-T3",)):
            with self.subTest(active=active), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp, active=active) as (repo, state, controller):
                home = controller.marker().parent
                claim = self.launcher_marker(controller)
                rr.atomic_write(home / rr.MANUAL_RECORD, json.dumps({"claim": claim, "exit": 0}))
                holder = rr.open_private(home / rr.MANUAL_LOCK)
                rr.fcntl.flock(holder, rr.fcntl.LOCK_EX | rr.fcntl.LOCK_NB)
                try:
                    with self.assertRaisesRegex(rr.Stop, "launcher is still running"):
                        controller.refuse_live_manual()
                    with self.assertRaisesRegex(rr.Stop, "launcher is still running"):
                        self.preflight(repo, controller)
                    code, message = self.resume()
                    self.assertEqual(code, 2)
                    self.assertIn("launcher is still running", message)
                finally:
                    holder.close()
                controller.refuse_live_manual()

    @unittest.skipUnless(sys.platform == "linux" and shutil.which("setsid"), "PR_SET_CHILD_SUBREAPER is Linux-only")
    def test_the_launcher_forwards_signals_and_still_waits(self):
        # SIGHUP is the lead's terminal closing: the launcher must outlive it like SIGINT/SIGTERM/SIGQUIT,
        # forward it to the session's group and keep waiting for a setsid'd grandchild outside that group.
        sleep = shutil.which("sleep")
        for sig in (signal.SIGTERM, signal.SIGHUP, signal.SIGQUIT, signal.SIGINT):
            with self.subTest(signal=sig.name), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller):
                run, home = Path(tmp), controller.marker().parent
                # The trap before the pid file; sleep as a background job (it ignores INT and QUIT, so the
                # group-wide QUIT dumps no core) that `wait` lets the trap interrupt at once.
                script = (f"trap 'exit 7' TERM HUP QUIT INT; echo $$ > {run}/child.pid; ( setsid /bin/sh -c "
                          f"'echo $$ > {run}/grand.pid; i=0; while [ ! -e {run}/GO ] && [ $i -lt 500 ]; do "
                          f"{sleep} 0.02; i=$((i+1)); done' </dev/null >/dev/null 2>&1 & ); i=0; "
                          f"while [ $i -lt 500 ]; do {sleep} 0.02 & wait $!; i=$((i+1)); done")
                claim = self.launcher_marker(controller, ["/bin/sh", "-c", script])
                launcher, pids = self.launch(controller, ["/bin/sh", "-c", script]), []
                try:
                    self.wait_for(lambda: all((run / f).exists() and (run / f).read_text().strip()
                                              for f in ("child.pid", "grand.pid")))
                    pids = [int((run / f).read_text()) for f in ("child.pid", "grand.pid")]
                    os.kill(launcher, sig)
                    self.wait_for(lambda: not os.path.exists(f"/proc/{pids[0]}"))  # the forward reached the child
                    self.assertTrue(os.path.exists(f"/proc/{pids[1]}"))  # not in the session's group
                    self.assertEqual(os.waitpid(launcher, os.WNOHANG), (0, 0))  # still waiting for it
                    record = json.loads((home / rr.MANUAL_RECORD).read_text())
                    self.assertEqual(record["claim"], claim)
                    self.assertNotIn("exit", record)
                    (run / "GO").touch()
                    self.assertEqual(self.reap(launcher), 7)
                    launcher = None
                finally:
                    (run / "GO").touch()
                    for pid in pids:
                        self.wait_for(lambda: not os.path.exists(f"/proc/{pid}") or self.kill(pid), 5)
                    if launcher is not None:
                        with contextlib.suppress(ProcessLookupError, ChildProcessError):
                            os.kill(launcher, signal.SIGKILL)
                            os.waitpid(launcher, 0)
                self.assertEqual(json.loads((home / rr.MANUAL_RECORD).read_text())["exit"], 7)

    @unittest.skipUnless(sys.platform == "linux", "PR_SET_CHILD_SUBREAPER is Linux-only")
    def test_a_child_killed_by_a_signal_exits_128_plus_the_signal(self):
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            home = controller.marker().parent
            self.launcher_marker(controller, ["/bin/sh", "-c", "kill -TERM $$"])
            self.assertEqual(self.reap(self.launch(controller, ["/bin/sh", "-c", "kill -TERM $$"])), 143)
            self.assertEqual(json.loads((home / rr.MANUAL_RECORD).read_text())["exit"], 143)

    @unittest.skipUnless(sys.platform == "linux", "PR_SET_CHILD_SUBREAPER and /proc are Linux-only")
    def test_a_launcher_that_died_keeps_its_claim_until_the_lead_acknowledges(self):
        # SIGKILL leaves the start record without "exit" and orphans the session (reparented elsewhere).
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            run, home = Path(tmp), controller.marker().parent
            record_path = home / rr.MANUAL_RECORD
            # The planned command itself, so only the sticky record can refuse a later launcher.
            script = (f"[ -e {run}/child.pid ] && touch {run}/RAN; echo $$ > {run}/child.pid; i=0; "
                      f"while [ ! -e {run}/GO ] && [ $i -lt 500 ]; do {shutil.which('sleep')} 0.02; i=$((i+1)); done")
            argv = ["/bin/sh", "-c", script]
            claim = self.launcher_marker(controller, argv)
            launcher, orphan = self.launch(controller, argv), None
            try:
                self.wait_for(lambda: (run / "child.pid").exists() and (run / "child.pid").read_text().strip())
                orphan = int((run / "child.pid").read_text())
                self.wait_for(lambda: "child_pid" in json.loads(record_path.read_text()))
                os.kill(launcher, signal.SIGKILL)
                os.waitpid(launcher, 0)
                started = record_path.read_text()
                self.assertNotIn("exit", json.loads(started))
                self.assertEqual(json.loads(started)["child_pid"], orphan)
                self.assertEqual(json.loads(started)["child_pgid"], orphan)
                # Sticky: a later launcher for the claim neither runs nor rewrites the record.
                second = self.launch(controller, argv, out=run / "second.out")
                self.assertEqual(self.reap(second), 2)
                self.assertFalse((run / "RAN").exists())
                self.assertEqual(record_path.read_text(), started)
                self.assertIn(f"`resume --accept-orphans {claim}`", (run / "second.out").read_text())
                self.assertIn(f"`pgrep -g {orphan}`", (run / "second.out").read_text())
                with self.assertRaisesRegex(rr.Stop, f"resume --accept-orphans {claim}") as refused:
                    controller.refuse_live_manual()
                # The lead is told what to look for, and what the runner cannot enumerate.
                self.assertIn(f"pid {orphan} in process group {orphan}", str(refused.exception))
                self.assertIn("cannot enumerate a survivor that left that group with setsid and cleared its "
                              "environment", str(refused.exception))
                # The acknowledgement itself refuses while the orphan still carries the claim's marker.
                code, message = self.resume("--accept-orphans", claim)
                self.assertEqual(code, 2)
                self.assertIn(f"{orphan}", message)
                self.assertEqual(record_path.read_text(), started)
            finally:
                (run / "GO").touch()
                if orphan is not None:
                    self.wait_for(lambda: not os.path.exists(f"/proc/{orphan}") or self.kill(orphan), 5)
            code, message = self.resume("--accept-orphans", "0" * 32)  # a typo is refused, not ignored
            self.assertEqual(code, 2)
            self.assertIn("not the active launcher claim", message)
            # The acknowledgement also refuses while a tagged worker of the controller is alive.
            with unittest.mock.patch.object(rr.Controller, "refuse_live_workers",
                                            side_effect=rr.Stop("a previous worker process is still alive: [1]")):
                code, message = self.resume("--accept-orphans", claim)
            self.assertEqual(code, 2)
            self.assertIn("worker process is still alive", message)
            holder = rr.open_private(home / rr.MANUAL_LOCK)
            rr.fcntl.flock(holder, rr.fcntl.LOCK_EX | rr.fcntl.LOCK_NB)
            try:
                code, message = self.resume("--accept-orphans", claim)
                self.assertEqual(code, 2)
                self.assertIn("launcher is still running", message)
            finally:
                holder.close()
            controller_lock = rr.open_private(state / "controller.lock")
            rr.fcntl.flock(controller_lock, rr.fcntl.LOCK_EX | rr.fcntl.LOCK_NB)
            try:
                self.assertEqual(self.resume("--accept-orphans", claim)[0], 2)
            finally:
                controller_lock.close()
            self.assertEqual(record_path.read_text(), started)
            self.assertEqual(self.resume("--accept-orphans", claim)[0], 0)
            self.assertEqual(json.loads(record_path.read_text())["claim"], claim)
            self.assertNotIn("exit", json.loads(record_path.read_text()))
            controller.refuse_live_manual()  # acknowledged: the refusal lifts
            self.assertEqual(self.resume("--accept-orphans", claim)[0], 2)  # nothing left to acknowledge
            # Fail closed: the acknowledged claim is retired by `resume`, never run again.
            third = self.launch(controller, argv, out=run / "third.out")
            self.assertEqual(self.reap(third), 2)
            self.assertFalse((run / "RAN").exists())
            self.assertIn("acknowledged without a completion record", (run / "third.out").read_text())
            with self.assertRaisesRegex(rr.Stop, "holds the session baseline"):
                self.preflight(repo, controller)
            self.assertEqual(self.accept()[0], 0)
            self.assertIsNone(controller.manual_claim())

    @unittest.skipUnless(sys.platform == "linux", "PR_SET_CHILD_SUBREAPER is Linux-only")
    def test_the_launcher_refuses_while_the_controller_lock_is_held(self):
        # A `resume` that is retiring the claim holds controller.lock: no session may start under it.
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            run, home = Path(tmp), controller.marker().parent
            self.launcher_marker(controller)
            holder = rr.open_private(state / "controller.lock")
            rr.fcntl.flock(holder, rr.fcntl.LOCK_EX | rr.fcntl.LOCK_NB)
            try:
                launcher = self.launch(controller, ["/bin/sh", "-c", f"touch {run}/RAN"], out=run / "out")
                self.assertEqual(self.reap(launcher), 2)
            finally:
                holder.close()
            self.assertIn("controller.lock", (run / "out").read_text())
            self.assertFalse((run / "RAN").exists())
            self.assertFalse((home / rr.MANUAL_RECORD).exists())

    def test_the_record_and_environment_are_read_under_the_probe_lock(self):
        # Released first, a launcher could start between the probe and the record read.
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            home = controller.marker().parent
            claim = self.launcher_marker(controller)
            rr.atomic_write(home / rr.MANUAL_RECORD, json.dumps({"claim": claim, "exit": 0}))
            rr.open_private(home / rr.MANUAL_LOCK).close()
            seen = []

            def held(*_):
                with open(home / rr.MANUAL_LOCK, "rb") as other:
                    try:
                        rr.fcntl.flock(other, rr.fcntl.LOCK_EX | rr.fcntl.LOCK_NB)
                        seen.append(False)
                    except BlockingIOError:
                        seen.append(True)

            read = rr.read_record
            with unittest.mock.patch.object(rr, "manual_processes", side_effect=lambda c: held() or []), \
                    unittest.mock.patch.object(rr, "read_record", side_effect=lambda path: held() or read(path)):
                controller.refuse_live_manual()
            self.assertEqual(seen, [True, True])

    def test_a_manual_run_refusal_before_the_launcher_goes_to_stderr(self):
        # The pin_state refusal precedes the manual-run branch; stdout is the session's event stream.
        with unittest.mock.patch.object(rr, "isolate_imports"), \
                unittest.mock.patch.object(rr, "pin_state", side_effect=rr.Stop("planted link")), \
                unittest.mock.patch.object(rr.sys, "argv", ["roadmap_runner.py", "manual-run", "--", "true"]), \
                contextlib.redirect_stdout(io.StringIO()) as out, contextlib.redirect_stderr(io.StringIO()) as err:
            self.assertEqual(rr.main(), 2)
        self.assertEqual(out.getvalue(), "")
        self.assertIn("planted link", err.getvalue())

    def test_a_planted_launcher_lock_is_refused_without_blocking(self):
        for kind in ("symlink", "fifo", "directory"):
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller):
                run, lock = Path(tmp), controller.marker().parent / rr.MANUAL_LOCK
                lock.parent.mkdir(parents=True, exist_ok=True)
                if kind == "symlink":  # to a private regular file: only O_NOFOLLOW refuses it
                    (run / "private").write_text("")
                    lock.symlink_to(run / "private")
                elif kind == "fifo":
                    os.mkfifo(lock)
                else:
                    lock.mkdir()
                with self.deadline(), self.assertRaisesRegex(rr.Stop, "launcher lock"):
                    controller.refuse_live_manual()
                with self.deadline(), self.assertRaisesRegex(rr.Stop, "launcher lock"):
                    self.preflight(repo, controller)
                if sys.platform == "linux":
                    self.launcher_marker(controller, ["/bin/sh", "-c", f"touch {run}/RAN"])
                    launcher = self.launch(controller, ["/bin/sh", "-c", f"touch {run}/RAN"])
                    self.assertEqual(self.reap(launcher), 2)
                    self.assertFalse((run / "RAN").exists())
                    self.assertFalse((controller.marker().parent / rr.MANUAL_RECORD).exists())

    @unittest.skipUnless(sys.platform == "linux", "/proc and PR_SET_CHILD_SUBREAPER are Linux-only")
    def test_the_launcher_runs_only_for_a_claim_and_records_its_status(self):
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            run, home = Path(tmp), controller.marker().parent
            with unittest.mock.patch.object(rr.sys, "platform", "darwin"), \
                    self.assertRaisesRegex(rr.Stop, "needs Linux"):
                controller.manual_run(["true"])
            libc = unittest.mock.Mock()
            libc.return_value.prctl.return_value = -1
            with unittest.mock.patch.object(rr.ctypes, "CDLL", libc), self.assertRaisesRegex(rr.Stop, "subreaper"):
                controller.manual_run(["true"])
            self.assertEqual(self.reap(self.launch(controller, ["/bin/sh", "-c", f"touch {run}/RAN"])), 2)
            self.assertFalse((run / "RAN").exists())  # no claim: nothing runs
            script = f'echo "${rr.MANUAL_ENV}" > {run}/env; exit 4'
            claim = self.launcher_marker(controller, ["/bin/sh", "-c", script])
            self.assertEqual(self.reap(self.launch(controller, ["/bin/sh", "-c", script], cli=True)), 4)
            self.assertEqual((run / "env").read_text().strip(), claim)
            self.assertEqual(json.loads((home / rr.MANUAL_RECORD).read_text())["exit"], 4)
            controller.refuse_live_manual()
            # Supplementary: a process that kept the claim's marker in its environment is refused.
            survivor = rr.subprocess.Popen(["sleep", "30"], env={**os.environ, rr.MANUAL_ENV: claim})
            try:
                with self.assertRaisesRegex(rr.Stop, f"{survivor.pid}.*{rr.MANUAL_ENV}"):
                    controller.refuse_live_manual()
            finally:
                survivor.kill()
                survivor.wait()
            controller.refuse_live_manual()

    @unittest.skipUnless(sys.platform == "linux", "PR_SET_CHILD_SUBREAPER is Linux-only")
    def test_the_launcher_runs_only_the_planned_command(self):
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            run, home = Path(tmp), controller.marker().parent
            self.launcher_marker(controller, ["/bin/sh", "-c", "exit 0"])
            other = self.launch(controller, ["/bin/sh", "-c", f"touch {run}/RAN"], out=run / "out")
            self.assertEqual(self.reap(other), 2)
            self.assertIn("planned", (run / "out").read_text())
            self.assertFalse((run / "RAN").exists())
            self.assertFalse((home / rr.MANUAL_RECORD).exists())

    @unittest.skipUnless(sys.platform == "linux", "PR_SET_CHILD_SUBREAPER is Linux-only")
    def test_a_session_that_could_not_start_leaves_its_start_record(self):
        # An exec failure is no proof that nothing of the session ran: no completion is written.
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            argv = [str(Path(tmp) / "missing-codex")]
            claim = self.launcher_marker(controller, argv)
            self.assertEqual(self.reap(self.launch(controller, argv)), 127)
            record = json.loads((controller.marker().parent / rr.MANUAL_RECORD).read_text())
            self.assertEqual(record["claim"], claim)
            self.assertNotIn("exit", record)
            with self.assertRaisesRegex(rr.Stop, "no `manual-run` completion record") as refused:
                controller.refuse_live_manual()
            self.assertIn("no child pid was recorded", str(refused.exception))

    @unittest.skipUnless(sys.platform == "linux", "PR_SET_CHILD_SUBREAPER and /proc are Linux-only")
    def test_the_session_gets_default_sigchld_and_an_unblocked_mask(self):
        # An inherited SIG_IGN for SIGCHLD lets the kernel reap the child: the status would read 127.
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            run, home = Path(tmp), controller.marker().parent
            # Not a shell: dash clears its signal mask at startup and would hide an inherited block.
            argv = [sys.executable, "-c", "import pathlib, sys; pathlib.Path(sys.argv[1]).write_text("
                    "pathlib.Path('/proc/self/status').read_text()); sys.exit(5)", str(run / "mask")]
            self.launcher_marker(controller, argv)
            ignore = lambda: signal.signal(signal.SIGCHLD, signal.SIG_IGN)
            self.assertEqual(self.reap(self.launch(controller, argv, prepare=ignore)), 5)
            self.assertEqual(json.loads((home / rr.MANUAL_RECORD).read_text())["exit"], 5)
            masks = dict(re.findall(r"^(Sig\w+):\s*([0-9a-f]+)$", (run / "mask").read_text(), re.M))
            self.assertEqual(int(masks["SigBlk"], 16), 0)  # the launcher's block is not inherited
            self.assertFalse(int(masks["SigIgn"], 16) & (1 << (signal.SIGCHLD - 1)))

    @unittest.skipUnless(sys.platform == "linux", "PR_SET_CHILD_SUBREAPER is Linux-only")
    def test_a_signal_during_the_start_reaches_the_session(self):
        # Before Popen returns there is no group to forward to: the signal is held, then forwarded.
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            home = controller.marker().parent
            argv = ["/bin/sh", "-c", f"i=0; while [ $i -lt 250 ]; do {shutil.which('sleep')} 0.02; "
                    "i=$((i+1)); done; exit 0"]
            self.launcher_marker(controller, argv)

            def early_signal():
                real = rr.subprocess.Popen

                def popen(*args, **kwargs):
                    os.kill(os.getpid(), signal.SIGTERM)  # arrives while the session is being started
                    return real(*args, **kwargs)
                rr.subprocess.Popen = popen  # only in the forked launcher
            self.assertEqual(self.reap(self.launch(controller, argv, prepare=early_signal)), 143)
            self.assertEqual(json.loads((home / rr.MANUAL_RECORD).read_text())["exit"], 143)

    @unittest.skipUnless(sys.platform == "linux", "PR_SET_CHILD_SUBREAPER and ptys are Linux-only here")
    def test_a_session_on_a_terminal_reads_it_in_the_foreground(self):
        # In its own process group, a background reader of the terminal would stop on SIGTTIN and the
        # launcher would wait forever. A pty makes this deterministic: the line is buffered before the read.
        # With stdin from a file too: the planned command reads its brief from stdin, not the terminal.
        for stdin in ("terminal", "file"):
            with self.subTest(stdin=stdin), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller):
                run = Path(tmp)
                argv = ["/bin/sh", "-c", f'echo $$ > {run}/child.pid; read line < /dev/tty; echo "$line" > {run}/line']
                self.launcher_marker(controller, argv)
                pid, master = pty.fork()
                if pid == 0:
                    code = 99
                    try:
                        if stdin == "file":
                            os.dup2(os.open(os.devnull, os.O_RDONLY), 0)
                        code = controller.manual_run(argv)
                        (run / "foreground").write_text(str(os.tcgetpgrp(1) == os.getpgrp()))
                    finally:
                        os._exit(code)
                try:
                    os.write(master, b"hello\n")
                    self.assertEqual(self.reap(pid), 0)
                finally:
                    with contextlib.suppress(OSError, ValueError):
                        os.killpg(int((run / "child.pid").read_text()), signal.SIGKILL)
                    os.close(master)
                self.assertEqual((run / "line").read_text().strip(), "hello")
                self.assertEqual((run / "foreground").read_text(), "True")  # the launcher took the terminal back

    def test_a_planted_launcher_record_is_refused_without_blocking(self):
        for kind in ("symlink", "fifo", "directory"):
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller):
                run, record = Path(tmp), controller.marker().parent / rr.MANUAL_RECORD
                argv = ["/bin/sh", "-c", f"touch {run}/RAN"]
                claim = self.launcher_marker(controller, argv)
                if kind == "symlink":  # to a completed record: only O_NOFOLLOW refuses it
                    (run / "done.json").write_text(json.dumps({"claim": claim, "exit": 0}))
                    record.symlink_to(run / "done.json")
                elif kind == "fifo":
                    os.mkfifo(record)
                else:
                    record.mkdir()
                with self.deadline(), self.assertRaisesRegex(rr.Stop, "launcher record"):
                    controller.refuse_live_manual()
                if sys.platform == "linux":
                    out = run / "out"  # reap() bounds the launcher and kills it on a timeout
                    self.assertEqual(self.reap(self.launch(controller, argv, out=out)), 2)
                    self.assertIn("launcher record", out.read_text())
                    self.assertFalse((run / "RAN").exists())

    def test_a_manual_run_refusal_after_pin_state_leaves_stdout_empty(self):
        # No claim: refused inside manual_run. The prctl is mocked so this process never becomes a subreaper.
        libc = unittest.mock.Mock()
        libc.return_value.prctl.return_value = 0
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp), \
                unittest.mock.patch.object(rr.ctypes, "CDLL", libc), \
                unittest.mock.patch.object(rr.sys, "platform", "linux"), \
                unittest.mock.patch.object(rr, "isolate_imports"), \
                unittest.mock.patch.object(rr.sys, "argv", ["roadmap_runner.py", "manual-run", "--", "true"]), \
                contextlib.redirect_stdout(io.StringIO()) as out, contextlib.redirect_stderr(io.StringIO()) as err:
            self.assertEqual(rr.main(), 2)
        self.assertEqual(out.getvalue(), "")
        self.assertIn("needs an active manual claim", err.getvalue())

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

    def test_a_pipeline_worktree_is_refused_with_its_cleanup(self):
        # Residual B of #416: a live agent_pipeline worktree stays refused (fail closed); the Stop
        # names the path and the cleanup instead of only listing it.
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            worktree = repo / "build" / "agent-pipeline" / "issue-1" / "worktree"
            worktree.mkdir(parents=True)
            (worktree / ".git").write_text("gitdir: /elsewhere/.git/worktrees/worktree\n")
            rr.atomic_write(controller.marker().parent / rr.TRUSTED_RECORD, "{}")
            with self.assertRaises(rr.Stop) as refused:
                self.preflight(repo, controller)
            message = str(refused.exception)
            self.assertIn("build/agent-pipeline/issue-1/worktree/.git", message)
            self.assertIn(f"`git -C {shlex.quote(str(repo))} worktree remove build/agent-pipeline/issue-1/worktree`",
                          message)

    def test_a_cleanup_hint_is_never_built_from_an_unsafe_task_name(self):
        # The session chooses the task directory: shell syntax in it must not become a printed command.
        root = Path("/srv/check out")  # the checkout root is quoted too
        with unittest.mock.patch.object(rr, "ROOT", root):
            for name in ("a;touch x", "$(id)", "`id`", "a b", "a\nb", "a'b", ".", ".."):
                with self.subTest(name=name):
                    path = f"build/agent-pipeline/{name}/worktree/.git"
                    text = rr.nested_git_cleanup([path])
                    self.assertNotIn("worktree remove", text)
                    self.assertIn(json.dumps(path), text)  # rendered as data
                    self.assertIn("remove it after review", text)
            text = rr.nested_git_cleanup(["build/agent-pipeline/issue-1.b_c/worktree/.git"])
            self.assertIn("`git -C '/srv/check out' worktree remove build/agent-pipeline/issue-1.b_c/worktree`", text)

    def test_the_cleanup_hint_is_bounded(self):
        nested = [f"build/agent-pipeline/t{i:02}/worktree/.git" for i in range(25)]
        with unittest.mock.patch.object(rr, "ROOT", Path("/repo")):
            text = rr.nested_git_cleanup(nested)
        self.assertIn("(+5 more)", text)  # the paths
        self.assertIn("(+20 more)", text)  # the commands
        self.assertEqual(text.count("worktree remove"), 5)
        self.assertLess(len(text), 2000)

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
                real_digest = rr.Controller.review_digest

                def digest(self, review, read):
                    if window == "before the digest":
                        late.write_text("planted before the digest")
                    value = real_digest(self, review, read)
                    if window == "after the digest":  # before the snapshot is built, unlike retire_baseline
                        late.write_text("planted after the digest")
                    if window == "accepted file rewritten" and review.get("created"):
                        late.write_text("print('swapped')\n")
                    return value
                if window == "accepted file rewritten":
                    reviewed = re.search(r"resume --accept ([0-9a-f]+)", self.resume()[1]).group(1)
                with unittest.mock.patch.object(rr.Controller, "review_digest", digest):
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
                        self.assertEqual(rr.main(), 0)  # the brake holds: the link is replaced, never followed
                    self.assertFalse((state / "STOP").is_symlink())
                else:
                    target = state / "x.json"
                    target.with_suffix(f".json.tmp{os.getpid()}").symlink_to(record)
                    with self.assertRaisesRegex(rr.Stop, "planted symlink"):
                        rr.atomic_write(target, "{}")
                self.assertFalse(record.exists())
                self.assertFalse(controller.trusted() and planted != "hardlinked controller.lock")

    def test_a_state_directory_linked_into_a_trusted_home_is_refused(self):
        # 56716bca: with build/roadmap-pipeline -> <home>, the "legacy" marker was the marker itself,
        # so `resume --accept` wrote the accepted snapshot and unlinked it at once. 7d699231 checked
        # only this checkout's home: a link to a sibling checkout's home unlinked that one's marker.
        targets = ("home", "its parent", "inside it", "a sibling checkout's home")
        for entry in ("resume --accept", "acquire"):
            for target in targets:
                with self.subTest(entry=entry, target=target), tempfile.TemporaryDirectory() as tmp, \
                        self.live_state(tmp) as (repo, state, controller), \
                        unittest.mock.patch.object(rr.Controller, "refuse_live_workers"):
                    home = controller.marker().parent
                    self.manual_marker(controller)
                    sibling = home.parent / ("0" * 16)
                    sibling.mkdir()
                    rr.atomic_write(sibling / rr.CALLBACKS_MARKER, json.dumps({"manual": "S", "ignored": {}}))
                    digest = re.search(r"resume --accept ([0-9a-f]+)", self.resume()[1]).group(1)
                    for name in os.listdir(state):
                        (state / name).unlink()
                    state.rmdir()
                    link = {"home": home, "its parent": home.parent, "inside it": home / "sub",
                            "a sibling checkout's home": sibling}[target]
                    link.mkdir(exist_ok=True)
                    state.symlink_to(link, target_is_directory=True)
                    if entry == "acquire":
                        with self.assertRaisesRegex(rr.Stop, "remove the link"):
                            controller.acquire()
                    else:
                        code, message = self.resume("--accept", digest)  # the reviewed digest
                        self.assertEqual(code, 2, message)
                        self.assertIn("remove the link", message)
                    self.assertEqual(controller.manual_claim(), "L01-T3")  # the claim and its marker stand
                    self.assertTrue((sibling / rr.CALLBACKS_MARKER).exists())  # and the sibling's
                    self.assertFalse((link / "controller.lock").exists())

    def test_a_linked_build_component_creates_nothing_in_a_sibling_home(self):
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            sibling = controller.marker().parent.parent / ("0" * 16)
            sibling.mkdir(parents=True)
            state.rmdir()
            (repo / "build").rmdir()
            (repo / "build").symlink_to(sibling, target_is_directory=True)
            with self.assertRaisesRegex(rr.Stop, "remove the link"):
                controller.acquire()
            self.assertEqual(os.listdir(sibling), [])  # checked before mkdir followed the link
            # 213797e9: main() made the state directory first, and stop/pause wrote through the link.
            for command in ("resume", "stop", "pause", "status", "brief", "validate"):
                with unittest.mock.patch.object(rr, "isolate_imports"), \
                        unittest.mock.patch.object(rr.sys, "argv", ["roadmap_runner.py", command]), \
                        contextlib.redirect_stdout(io.StringIO()) as out:
                    self.assertEqual(rr.main(), 2, command)
                self.assertIn("remove the link", out.getvalue())
                self.assertEqual(os.listdir(sibling), [], command)

    def test_stop_and_pause_do_not_write_through_a_linked_state_directory(self):
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            sibling = controller.marker().parent.parent / ("0" * 16)
            sibling.mkdir(parents=True)
            state.rmdir()
            state.symlink_to(sibling, target_is_directory=True)
            for command in ("stop", "pause"):
                with unittest.mock.patch.object(rr, "isolate_imports"), \
                        unittest.mock.patch.object(rr.sys, "argv", ["roadmap_runner.py", command]), \
                        contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(rr.main(), 2, command)
            self.assertEqual(os.listdir(sibling), [])

    def test_a_link_swapped_in_after_the_descriptor_check_carries_no_unlink(self):
        # The unlink itself goes through the verified descriptor, not the path walked again.
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            self.manual_marker(controller)
            sibling = controller.marker().parent.parent / ("0" * 16)
            sibling.mkdir()
            rr.atomic_write(sibling / rr.CALLBACKS_MARKER, json.dumps({"manual": "S", "ignored": {}}))
            digest = re.search(r"resume --accept ([0-9a-f]+)", self.resume()[1]).group(1)
            real_unlink, calls = os.unlink, []

            def swap_then_unlink(path, *args, **kwargs):
                if path == rr.CALLBACKS_MARKER and kwargs.get("dir_fd") is not None:
                    calls.append(1)  # the descriptor is open; swap before the legacy unlink runs
                    os.rename(state, Path(tmp) / "state-moved")
                    state.symlink_to(sibling, target_is_directory=True)
                return real_unlink(path, *args, **kwargs)
            with unittest.mock.patch.object(rr.os, "unlink", swap_then_unlink):
                self.assertEqual(self.resume("--accept", digest)[0], 0)
            self.assertEqual(len(calls), 1)
            self.assertTrue((sibling / rr.CALLBACKS_MARKER).exists())

    def test_a_link_inside_the_state_directory_carries_no_write(self):
        # c2028873 pinned only the directory itself: manual/ or runs/ linked by an exited session took
        # brief.md, dispatch.json and the run logs into another checkout's trusted home.
        for planted in ("manual", "runs", "runs/<run>/<task>"):
            with self.subTest(planted=planted), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller):
                sibling = controller.marker().parent.parent / ("0" * 16)
                sibling.mkdir(parents=True)
                link = state / planted.replace("<run>", "r1").replace("<task>", "A")
                link.parent.mkdir(parents=True, exist_ok=True)
                link.symlink_to(sibling, target_is_directory=True)
                with unittest.mock.patch.object(rr, "isolate_imports"), \
                        unittest.mock.patch.object(rr.sys, "argv", ["roadmap_runner.py", "brief"]), \
                        contextlib.redirect_stdout(io.StringIO()) as out:
                    self.assertEqual(rr.main(), 2)
                self.assertIn("links in the controller state", out.getvalue())
                with self.assertRaisesRegex(rr.Stop, "links in the controller state"):
                    controller.preflight()  # before every task of a running controller
                self.assertEqual(os.listdir(sibling), [])

    def test_a_refusal_inside_resume_reports_stop_as_still_present(self):
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller), \
                unittest.mock.patch.object(rr.Controller, "unlink_in_state", side_effect=rr.Stop("refused")):
            (state / "STOP").write_text("x")
            code, message = self.resume()
            self.assertEqual(code, 2)
            self.assertIn("['STOP'] still present", message)
            self.assertNotIn("stop/pause cleared", message)

    def test_the_default_state_is_read_when_the_controller_is_made(self):
        # Bound at class creation, `brief` under a patched STATE wrote the real checkout's lock.
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            self.assertEqual(rr.Controller(rr.Config()).state, state)

    def test_resume_verifies_the_state_directory_without_proc(self):
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            real_readlink = os.readlink

            def no_proc(path, *args, **kwargs):
                if str(path).startswith("/proc/"):
                    raise FileNotFoundError(2, "No such file or directory")
                return real_readlink(path, *args, **kwargs)
            (state / "STOP").write_text("x")
            with unittest.mock.patch.object(rr.os, "readlink", no_proc):
                code, message = self.resume()
            self.assertEqual(code, 0, message)
            self.assertFalse((state / "STOP").exists())

    def test_a_linked_legacy_marker_outside_the_checkout_is_refused(self):
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller), \
                unittest.mock.patch.object(rr.Controller, "refuse_live_workers"):
            self.manual_marker(controller)
            sibling = controller.marker().parent.parent / ("0" * 16)
            sibling.mkdir()
            rr.atomic_write(sibling / rr.CALLBACKS_MARKER, json.dumps({"manual": "S", "ignored": {}}))
            (state / rr.CALLBACKS_MARKER).symlink_to(sibling / rr.CALLBACKS_MARKER)
            code, message = self.resume()
            self.assertEqual(code, 2, message)
            self.assertIn("links in the controller state", message)
            self.assertTrue((sibling / rr.CALLBACKS_MARKER).exists())

    def test_a_link_swapped_in_after_the_review_carries_no_unlink(self):
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller), \
                unittest.mock.patch.object(rr.Controller, "refuse_live_workers"):
            self.manual_marker(controller)
            sibling = controller.marker().parent.parent / ("0" * 16)
            sibling.mkdir()
            rr.atomic_write(sibling / rr.CALLBACKS_MARKER, json.dumps({"manual": "S", "ignored": {}}))
            digest = re.search(r"resume --accept ([0-9a-f]+)", self.resume()[1]).group(1)
            real_digest = rr.Controller.review_digest

            def digest_then_swap(self, review, read):  # after resume's first own_state()
                value = real_digest(self, review, read)
                for name in os.listdir(state):
                    (state / name).unlink()
                state.rmdir()
                state.symlink_to(sibling, target_is_directory=True)
                return value
            with unittest.mock.patch.object(rr.Controller, "review_digest", digest_then_swap):
                self.assertEqual(self.resume("--accept", digest)[0], 2)
            self.assertTrue((sibling / rr.CALLBACKS_MARKER).exists())
            self.assertEqual(controller.manual_claim(), "L01-T3")  # refused before anything was retired

    def test_a_link_swapped_in_after_the_last_check_carries_no_unlink(self):
        # 213797e9 walked the path again for the unlink after own_state() passed.
        for component in ("state directory", "build"):
            with self.subTest(component=component), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller):
                self.manual_marker(controller)
                sibling = controller.marker().parent.parent / ("0" * 16)
                (sibling / "roadmap-pipeline").mkdir(parents=True)
                for target in (sibling, sibling / "roadmap-pipeline"):
                    rr.atomic_write(target / rr.CALLBACKS_MARKER, json.dumps({"manual": "S", "ignored": {}}))
                digest = re.search(r"resume --accept ([0-9a-f]+)", self.resume()[1]).group(1)
                real_own, calls = rr.Controller.own_state, []

                def own_then_swap(self):
                    real_own(self)
                    calls.append(1)
                    if len(calls) == 2:  # the check right before the legacy unlink
                        os.rename(repo / "build", Path(tmp) / "build-moved")
                        if component == "build":
                            (repo / "build").symlink_to(sibling, target_is_directory=True)
                        else:
                            (repo / "build").mkdir()
                            (repo / "build" / "roadmap-pipeline").symlink_to(sibling, target_is_directory=True)
                with unittest.mock.patch.object(rr.Controller, "own_state", own_then_swap):
                    self.assertEqual(self.resume("--accept", digest)[0], 2)
                self.assertTrue((sibling / rr.CALLBACKS_MARKER).exists())
                self.assertTrue((sibling / "roadmap-pipeline" / rr.CALLBACKS_MARKER).exists())
                self.assertEqual(controller.manual_claim(), "L01-T3")

    def test_a_refused_resume_does_not_claim_it_cleared_stop(self):
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            elsewhere = Path(tmp) / "elsewhere"
            elsewhere.mkdir()
            (elsewhere / "STOP").write_text("x")
            state.rmdir()
            state.symlink_to(elsewhere, target_is_directory=True)
            code, message = self.resume()
            self.assertEqual(code, 2)
            self.assertNotIn("stop/pause cleared", message)
            self.assertTrue((elsewhere / "STOP").exists())

    def test_the_state_descriptor_holds_when_own_state_is_bypassed(self):
        # Second layer under own_state(): unlinks go through a descriptor opened without following
        # links. 3e8a1a39's fallback without /proc compared the directory with itself and unlinked.
        real_readlink = os.readlink

        def no_proc(path, *args, **kwargs):
            if str(path).startswith("/proc/"):
                raise FileNotFoundError(2, "No such file or directory")
            return real_readlink(path, *args, **kwargs)
        for proc in (True, False):
            with self.subTest(proc=proc), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller), \
                    unittest.mock.patch.object(rr, "pin_state"), \
                    unittest.mock.patch.object(rr.Controller, "own_state"), \
                    unittest.mock.patch.object(rr.os, "readlink", real_readlink if proc else no_proc):
                home = controller.marker().parent
                self.manual_marker(controller)
                (home / "STOP").write_text("x")
                state.rmdir()
                state.symlink_to(home, target_is_directory=True)
                code, message = self.resume()
                self.assertEqual(code, 2, message)
                self.assertIn("without following a link", message)
                self.assertTrue((home / "STOP").exists())
                self.assertEqual(controller.manual_claim(), "L01-T3")

    def worker_run(self, repo, state, controller, plant):
        """run_task with a real session(): each worker runs `plant` (shell) in attempt 1, then fails."""
        controller.cfg = rr.Config(max_repairs=1)
        controller.callbacks = "cb"

        def argv(cfg, name, sid, settings, *args, **kwargs):
            adir = Path(settings).parent
            # Plant only after the heartbeat that follows the start: a write while the worker is still
            # alive is the accepted live-writer residual, not what these tests are about.
            wait = f'until grep -q worker_pid "{state / "lease.json"}" 2>/dev/null; do sleep 0.01; done; '
            script = wait + plant(adir) if adir.name == "attempt-1" else ""
            return ["sh", "-c", script + 'echo \'{"is_error": true, "result": "no"}\'; exit 1']

        def preflight():
            controller.own_state()
            return "a" * 40
        with unittest.mock.patch.object(rr, "claude_argv", argv), \
                unittest.mock.patch.object(rr, "require_pipeline_release"), \
                unittest.mock.patch.object(controller, "preflight", preflight):
            try:
                return controller.run_task(task("A"))
            finally:
                controller.release()

    def test_a_link_left_by_a_session_carries_no_later_write(self):
        # 3e8a1a39 pinned the state only in preflight: attempt 2 wrote its session files through an
        # attempt-2 link left by attempt 1, and a relinked state took the heartbeat, then release()
        # unlinked another checkout's lease.
        for planted in ("attempt-2", "state directory"):
            with self.subTest(planted=planted), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller):
                sibling = controller.marker().parent.parent / ("0" * 16)
                sibling.mkdir(parents=True)
                other = Path(tmp) / "other-checkout" / "build" / "roadmap-pipeline"
                other.mkdir(parents=True)
                lease = json.dumps({"task": "OTHER", "controller_pid": 1})
                (other / "lease.json").write_text(lease)
                if planted == "attempt-2":
                    def plant(adir):
                        return f'ln -s "{sibling}" "{adir.parent / "attempt-2"}"; '
                else:
                    def plant(adir):
                        return f'mv "{state}" "{state}.moved"; ln -s "{other}" "{state}"; '
                with self.assertRaisesRegex(rr.Stop, "links in the controller state|resolves to"):
                    self.worker_run(repo, state, controller, plant)
                self.assertEqual(os.listdir(sibling), [])
                self.assertEqual(sorted(os.listdir(other)), ["lease.json"])
                self.assertEqual((other / "lease.json").read_text(), lease)

    @unittest.skipIf(os.geteuid() == 0, "root reads any directory")
    def test_an_unreadable_state_directory_is_refused(self):
        # 3e8a1a39's walk dropped the scandir error: a link under an unreadable directory passed.
        for unreadable in ("state", "state/runs"):
            with self.subTest(unreadable=unreadable), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller):
                sibling = controller.marker().parent.parent / ("0" * 16)
                sibling.mkdir(parents=True)
                (state / "runs").mkdir()
                (state / "runs" / "r1").symlink_to(sibling, target_is_directory=True)
                locked = state if unreadable == "state" else state / "runs"
                locked.chmod(0o300)
                try:
                    with self.assertRaisesRegex(rr.Stop, "unreadable controller state"):
                        controller.acquire()
                finally:
                    locked.chmod(0o700)
                self.assertEqual(os.listdir(sibling), [])

    def test_stop_and_pause_hold_despite_a_link_deeper_in_the_state(self):
        # 3e8a1a39 refused the brake over any link a worker left: the running controller went on.
        for command in ("stop", "pause"):
            with self.subTest(command=command), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller):
                elsewhere = Path(tmp) / "elsewhere"
                elsewhere.mkdir()
                (state / "runs" / "r1" / "A").mkdir(parents=True)
                (state / "runs" / "r1" / "A" / "attempt-2").symlink_to(elsewhere, target_is_directory=True)
                with unittest.mock.patch.object(rr, "isolate_imports"), \
                        unittest.mock.patch.object(rr.sys, "argv", ["roadmap_runner.py", command]), \
                        contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(rr.main(), 0)
                self.assertTrue((state / command.upper()).exists())
                self.assertEqual(os.listdir(elsewhere), [])
                if command == "stop":
                    with self.assertRaisesRegex(rr.Stop, "stop file present"):
                        controller.check_controls()
                (state / command.upper()).unlink()
                home = controller.marker().parent
                home.mkdir(parents=True, exist_ok=True)
                (state / f"{command.upper()}.tmp{os.getpid()}").symlink_to(home / rr.TRUSTED_RECORD)
                with unittest.mock.patch.object(rr, "isolate_imports"), \
                        unittest.mock.patch.object(rr.sys, "argv", ["roadmap_runner.py", command]), \
                        contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(rr.main(), 2)  # refused cleanly, not a traceback
                self.assertFalse((home / rr.TRUSTED_RECORD).exists())

    def test_acquire_does_not_follow_a_planted_controller_lock(self):
        # The open itself, under the state pin (which refuses any link in the state directory first).
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller), \
                unittest.mock.patch.object(rr, "pin_state"):
            home = controller.marker().parent
            home.mkdir(parents=True)
            (state / "controller.lock").symlink_to(home / rr.TRUSTED_RECORD)
            with self.assertRaisesRegex(rr.Stop, "planted symlink"):
                controller.acquire()
            self.assertFalse((home / rr.TRUSTED_RECORD).exists())

    def swap_to_sibling_state(self, tmp, state):
        """A live swapper's move: the state directory becomes a link to a sibling checkout's state."""
        other = Path(tmp) / "other-checkout" / "build" / "roadmap-pipeline"
        other.mkdir(parents=True, exist_ok=True)
        lease = json.dumps({"task": "OTHER", "controller_pid": 1})
        (other / "lease.json").write_text(lease)
        os.rename(state, Path(tmp) / "state-moved")
        state.symlink_to(other, target_is_directory=True)
        return other / "lease.json", lease

    def test_lease_reads_and_unlinks_never_follow_a_swapped_state(self):
        # cabebd55 review: recover() and loop() still read and unlinked lease.json by path.
        for site in ("recover read", "recover unlink", "loop"):
            with self.subTest(site=site), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller):
                swapped = []
                if site == "recover unlink":
                    (state / "lease.json").write_text(json.dumps({"task": "A"}))  # no live owner

                    def ledger_then_swap(*args, **kwargs):  # after the lease was read, before its unlink
                        swapped.append(self.swap_to_sibling_state(tmp, state))
                        return {"tasks": []}
                    with unittest.mock.patch.object(rr, "load_ledger", side_effect=ledger_then_swap), \
                            self.assertRaisesRegex(rr.Stop, "without following a link"):
                        controller.acquire()
                elif site == "recover read":
                    (state / "lease.json").write_text(json.dumps({"task": "A", "controller_pid": 1}))
                    real_own, swap = rr.Controller.own_state, self.swap_to_sibling_state

                    def own_then_swap(ctl):  # right after acquire()'s pin, before recover()
                        real_own(ctl)
                        swapped.append(swap(tmp, state))
                    with unittest.mock.patch.object(rr.Controller, "own_state", own_then_swap), \
                            self.assertRaisesRegex(rr.Stop, "without following a link"):
                        controller.acquire()
                else:
                    controller.lease_owned = True
                    (state / "runs" / controller.run_id).mkdir(parents=True)
                    tasks = [task("A"), None]

                    def run_then_swap(task_):
                        swapped.append(self.swap_to_sibling_state(tmp, state))
                        (state / "runs" / controller.run_id).mkdir(parents=True)  # the event log's directory
                        return "c" * 40
                    with unittest.mock.patch.object(rr, "select_task", side_effect=lambda ledger: tasks.pop(0)), \
                            unittest.mock.patch.object(rr, "controller_environment"), \
                            unittest.mock.patch.object(controller, "run_task", run_then_swap), \
                            contextlib.redirect_stdout(io.StringIO()), \
                            self.assertRaisesRegex(rr.Stop, "without following a link"):
                        controller.loop(2, False)
                path, lease = swapped[0]
                self.assertEqual(path.read_text(), lease)  # the sibling's lease survives

    def test_a_planted_fifo_lease_is_refused_without_waiting(self):
        with tempfile.TemporaryDirectory() as tmp, self.live_state(tmp) as (repo, state, controller):
            os.mkfifo(state / "lease.json")
            previous = signal.signal(signal.SIGALRM, lambda *args: self.fail("blocked on the FIFO"))
            signal.alarm(5)
            try:
                with self.assertRaisesRegex(rr.Stop, "not a regular file"):
                    controller.acquire()
            finally:
                signal.alarm(0)
                signal.signal(signal.SIGALRM, previous)
                controller.release()

    def test_odd_entries_in_the_state_exit_cleanly(self):
        # A self-loop link at the state path, or a STOP directory, raised OSError instead of a refusal.
        for planted in ("self-loop state", "STOP directory"):
            with self.subTest(planted=planted), tempfile.TemporaryDirectory() as tmp, \
                    self.live_state(tmp) as (repo, state, controller):
                if planted == "self-loop state":
                    state.rmdir()
                    state.symlink_to(state)
                    command = "status"
                else:
                    (state / "STOP").mkdir()
                    command = "resume"
                with unittest.mock.patch.object(rr, "isolate_imports"), \
                        unittest.mock.patch.object(rr.sys, "argv", ["roadmap_runner.py", command]), \
                        contextlib.redirect_stdout(io.StringIO()) as out:
                    self.assertEqual(rr.main(), 2, out.getvalue())

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

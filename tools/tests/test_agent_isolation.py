"""Controller authority and file-import regressions; OS isolation is tested separately."""
import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import agent_gate
import agent_pipeline as ap
import common


class Authority(unittest.TestCase):
    def setUp(self):
        self.task = ap.Task(7, "bounded task", "serial", "K06", ["ci.yml#host-quality"], [], "body")
        self.base = "1" * 40
        self.policy = {"tasks": {"7": {
            "base": self.base, "issue_sha256": ap.issue_digest(self.task),
            "criterion": "K06", "mode": "serial", "gates": self.task.gates[:],
            "targets": ["kernel/src/example.rs"], "reads": ["Cargo.toml"],
        }}}

    def test_owner_authority_exists_before_model_specification(self):
        ap.authorize(self.task, self.policy, self.base)
        self.assertEqual(self.task.authorized_targets, ("kernel/src/example.rs",))
        self.assertEqual(self.task.base, self.base)
        self.task.targets = ["other"]
        other = copy.deepcopy(self.task)
        other.targets = ["different"]
        self.assertTrue(ap.conflicts(self.task, other))

    def test_changed_issue_source_gate_or_base_cannot_reuse_authority(self):
        for field, value in (("body", "changed"), ("title", "changed"), ("mode", "parallel"),
                             ("gates", ["other.yml#other"]), ("deps", [9])):
            with self.subTest(field=field):
                task = copy.deepcopy(self.task)
                setattr(task, field, value)
                with self.assertRaises(ap.TaskError):
                    ap.authorize(task, self.policy, self.base)
        with self.assertRaises(ap.TaskError):
            ap.authorize(self.task, self.policy, "2" * 40)

    def test_manifest_cannot_authorize_protected_or_ambiguous_targets(self):
        for path in (".github/agent-pipeline.json", "tools/agent_sandbox.py", "sub/AGENTS.md",
                     "sub/.gitattributes", "../outside", "a/./b", "a//b", "a/.git/config"):
            with self.subTest(path=path):
                policy = copy.deepcopy(self.policy)
                policy["tasks"]["7"]["targets"] = [path]
                with self.assertRaises(ap.TaskError):
                    ap.authorize(self.task, policy, self.base)

    def test_unauthorized_task_stops_before_worktree_or_l0(self):
        with tempfile.TemporaryDirectory() as temporary, mock.patch.object(ap, "RUN", Path(temporary)), \
                mock.patch.object(ap, "require_pipeline_release"), \
                mock.patch.object(ap, "sh", return_value=self.base), mock.patch.object(ap, "run_agent") as run:
            pipeline = ap.Pipeline.__new__(ap.Pipeline)
            pipeline.policy = {"tasks": {}}
            with self.assertRaisesRegex(ap.TaskError, "authorization is absent"):
                pipeline.prepare(self.task)
            self.assertFalse(self.task.dir.exists())
            run.assert_not_called()

    def test_versioned_grant_allows_only_its_manifest_commit(self):
        with mock.patch.object(ap, "sh", return_value=".github/agent-pipeline.json\0"):
            ap.authorize_source(self.task, self.policy, "2" * 40)
        self.assertEqual(self.task.base, "2" * 40)
        for delta in ("kernel/src/example.rs\0", ".github/agent-pipeline.json\0Cargo.toml\0"):
            with mock.patch.object(ap, "sh", return_value=delta), self.assertRaises(ap.TaskError):
                ap.authorize_source(self.task, self.policy, "2" * 40)

    def test_owner_revocation_on_main_stops_before_l0(self):
        current_base = "2" * 40
        revoked = copy.deepcopy(self.policy)
        revoked["tasks"].clear()

        def controller(arguments, *args, **kwargs):
            if arguments == ["git", "rev-parse", "origin/main"]:
                return current_base
            if arguments[:2] == ["git", "diff"]:
                return ".github/agent-pipeline.json\0"
            if arguments == ["git", "show", f"{current_base}:.github/agent-pipeline.json"]:
                return json.dumps(revoked)
            return ""

        with tempfile.TemporaryDirectory() as temporary, mock.patch.object(ap, "RUN", Path(temporary)), \
                mock.patch.object(ap, "require_pipeline_release"), \
                mock.patch.object(ap, "load_policy", return_value=self.policy), \
                mock.patch.object(ap, "sh", side_effect=controller), \
                mock.patch.object(ap, "run_agent") as run:
            pipeline = ap.Pipeline.__new__(ap.Pipeline)
            pipeline.policy = self.policy
            pipeline.sandbox = mock.Mock()
            pipeline.order = mock.Mock(return_value=["codex"])
            with self.assertRaisesRegex(ap.TaskError, "changed or was revoked"):
                pipeline.prepare(self.task)
            run.assert_not_called()
            self.assertFalse(self.task.dir.exists())

    def test_unapproved_controller_never_starts_configured_backend(self):
        policy = {"schema": 2, "enabled": True, "engines": ["codex"],
                  "tasks": {}, "sandbox": {}}
        with mock.patch.object(ap, "load_policy", return_value=policy), \
                mock.patch.object(ap, "verify_controller"), \
                mock.patch.object(ap, "sh", side_effect=["", "local", "main"]), \
                mock.patch.object(ap, "Sandbox") as backend:
            with self.assertRaises(ap.TaskError):
                ap.Pipeline(["codex"], False, True)
        backend.assert_not_called()


class SourceBoundary(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.source, self.stage = self.root / "source", self.root / "stage"
        self.source.mkdir()
        self.stage.mkdir()
        (self.source / "allowed").write_text("original")
        (self.source / "other").write_text("protected")

    def stage_source(self):
        common.stage_agent_source(self.source, self.stage, ["allowed", "other"])
        return common.agent_source_state(self.stage)

    def test_only_explicit_source_names_enter_disposable_tree(self):
        (self.source / "unlisted").write_text("not source")
        self.stage_source()
        self.assertEqual(set(common.agent_source_state(self.stage)), {"allowed", "other"})

    def test_complete_delta_is_checked_before_any_import(self):
        before = self.stage_source()
        (self.stage / "allowed").write_text("new")
        (self.stage / "other").write_text("changed")
        with self.assertRaises(ValueError):
            common.import_agent_changes(self.stage, self.source, before, ["allowed"])
        self.assertEqual((self.source / "allowed").read_text(), "original")

    def test_legitimate_new_update_and_delete_are_imported(self):
        before = self.stage_source()
        (self.stage / "allowed").unlink()
        (self.stage / "new").write_text("new")
        (self.stage / "new").chmod(0o644)
        changed = common.import_agent_changes(self.stage, self.source, before, ["allowed", "new"])
        self.assertEqual(changed, ["allowed", "new"])
        self.assertFalse((self.source / "allowed").exists())
        self.assertEqual((self.source / "new").read_text(), "new")
        self.assertEqual((self.source / "other").read_text(), "protected")

    def test_links_and_special_outputs_are_not_imported(self):
        before = self.stage_source()
        (self.stage / "allowed").unlink()
        for kind in ("symbolic", "hard", "fifo"):
            with self.subTest(kind=kind):
                target = self.stage / "allowed"
                if kind == "symbolic":
                    target.symlink_to(self.source / "allowed")
                elif kind == "hard":
                    os.link(self.stage / "other", target)
                else:
                    os.mkfifo(target)
                with self.assertRaises(ValueError):
                    common.import_agent_changes(self.stage, self.source, before, ["allowed"])
                target.unlink()
        self.assertEqual((self.source / "allowed").read_text(), "original")

    def test_destination_links_are_rejected(self):
        before = self.stage_source()
        (self.stage / "allowed").write_text("new")
        (self.source / "allowed").unlink()
        (self.source / "allowed").symlink_to(self.source / "other")
        with self.assertRaises(ValueError):
            common.import_agent_changes(self.stage, self.source, before, ["allowed"])
        self.assertEqual((self.source / "other").read_text(), "protected")

    def test_worker_scope_failure_does_not_modify_controller_source(self):
        sandbox = mock.Mock()
        sandbox.rootfs = self.root / "runtime"
        (sandbox.rootfs / "usr/bin").mkdir(parents=True)
        (sandbox.rootfs / "usr/bin/codex").write_text("fixture")
        def run(command, stage, *, profile):
            self.assertEqual(profile, "agent")
            (stage / "allowed").write_text("new")
            (stage / "other").write_text("out of scope")
            return subprocess.CompletedProcess(command, 0, "", "")
        sandbox.run.side_effect = run
        with mock.patch.object(ap, "agent_source_paths", return_value=["allowed", "other"]):
            with self.assertRaises(ap.TaskError):
                ap.run_agent("untrusted data", self.source, self.root / "agent.log", ["codex"],
                             sandbox=sandbox, writes=["allowed"])
        self.assertEqual((self.source / "allowed").read_text(), "original")

    def test_failed_engine_output_is_discarded_before_fallback(self):
        sandbox = mock.Mock()
        sandbox.rootfs = self.root / "runtime"
        (sandbox.rootfs / "usr/bin").mkdir(parents=True)
        for name in ("limited", "accepted", "broken"):
            (sandbox.rootfs / "usr/bin" / name).write_text("fixture")

        def run(command, stage, *, profile):
            self.assertEqual(profile, "agent")
            self.assertEqual((stage / "allowed").read_text(), "original")
            self.assertEqual((self.source / "allowed").read_text(), "original")
            if command[0] == "accepted":
                (stage / "allowed").write_text("accepted result")
                return subprocess.CompletedProcess(command, 0, "", "")
            (stage / "allowed").write_text("unfinished failed result")
            return subprocess.CompletedProcess(command, 1, "", {
                "limited": "rate limit exceeded", "broken": "client error",
            }[command[0]])

        sandbox.run.side_effect = run
        with mock.patch.dict(ap.ENGINES, {name: [name] for name in ("limited", "accepted", "broken")}, clear=True), \
                mock.patch.object(ap, "agent_source_paths", return_value=["allowed", "other"]):
            with self.assertRaises(ap.TaskError):
                ap.run_agent("untrusted data", self.source, self.root / "failed.log", ["broken"],
                             sandbox=sandbox, writes=["allowed"])
            self.assertEqual((self.source / "allowed").read_text(), "original")
            self.assertEqual(ap.run_agent("untrusted data", self.source, self.root / "retry.log",
                                         ["limited", "accepted"], sandbox=sandbox, writes=["allowed"]),
                             "accepted")
        self.assertEqual((self.source / "allowed").read_text(), "accepted result")
        self.assertIn("client error", (self.root / "failed.log").read_text())
        self.assertIn("rate limit exceeded", (self.root / "retry.log").read_text())

    def test_environment_is_explicit_and_credential_free(self):
        with mock.patch.dict(os.environ, {"EXAMPLE_CONTROLLER_CREDENTIAL": "fixture", "BASH_ENV": "fixture"}):
            env = common.isolated_environment()
        self.assertNotIn("EXAMPLE_CONTROLLER_CREDENTIAL", env)
        self.assertNotIn("BASH_ENV", env)
        for key, value in (("BASH_ENV", "fixture"), ("PATH", "/workspace"), ("CARGO_NET_OFFLINE", "false"),
                           ("EXAMPLE_CONTROLLER_CREDENTIAL", "fixture")):
            with self.subTest(key=key), self.assertRaises(ValueError):
                common.isolated_environment({key: value})

    def test_controller_uses_fixed_lookup_and_separates_authentication(self):
        with mock.patch.dict(os.environ, {"PATH": ":.:relative/bin", "GH_TOKEN": "synthetic", "BASH_ENV": "synthetic"}):
            local = common.controller_environment()
            publication = common.controller_environment(authenticated=True)
        self.assertEqual(local["PATH"], "/usr/bin:/bin")
        self.assertEqual(local["GIT_NO_LAZY_FETCH"], "1")
        self.assertNotIn("GH_TOKEN", local)
        self.assertNotIn("BASH_ENV", publication)
        self.assertEqual(publication["GH_TOKEN"], "synthetic")
        with mock.patch.dict(os.environ, {"GIT_CONFIG_GLOBAL": "unverified"}), self.assertRaises(ValueError):
            common.controller_environment()


class Publication(unittest.TestCase):
    def test_generated_git_lookalikes_are_never_controller_executables(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "git").write_text("inert non-script fixture")
            with mock.patch.dict(os.environ, {"PATH": ".:relative/bin", "GH_TOKEN": "synthetic"}), \
                    mock.patch.object(ap.subprocess, "run", side_effect=[
                        subprocess.CompletedProcess([], 0, "", ""),
                        subprocess.CompletedProcess([], 0, str(root / ".git/hooks"), ""),
                        subprocess.CompletedProcess([], 0, "", ""),
                    ]) as process:
                ap.sh(["git", "status", "--porcelain"], root)
            for call in process.call_args_list:
                self.assertEqual(call.args[0][0], "/usr/bin/git")
                self.assertEqual(call.kwargs["env"]["PATH"], "/usr/bin:/bin")
                self.assertNotIn("GH_TOKEN", call.kwargs["env"])
            with mock.patch.object(ap.subprocess, "run") as process:
                with self.assertRaises(ap.TaskError):
                    ap.sh(["gh", "pr", "list"], root)
                process.assert_not_called()

    def test_gate_rejects_unsafe_step_environment_before_execution(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            backend = mock.Mock()
            with mock.patch.object(agent_gate, "controller_source_commit", return_value="a" * 40):
                result = agent_gate.run_gate([("step", "true", {"BASH_ENV": "untrusted"})], root,
                                             root / "gate.log", sandbox=backend, source_paths=[])
            self.assertEqual(result, 2)
            backend.run.assert_not_called()

    def test_gate_retains_nontext_raw_output_separately(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            backend = mock.Mock()
            done = subprocess.CompletedProcess([], 0, "replacement", "diagnostic")
            done.stdout_bytes, done.stderr_bytes = b"\xff\0\n", b"\xfe\n"
            backend.run.return_value = done
            with mock.patch.object(agent_gate, "controller_source_commit", return_value="a" * 40):
                self.assertEqual(agent_gate.run_gate([("step", "true", {})], root, root / "gate.log",
                                                    sandbox=backend, source_paths=[]), 0)
            self.assertEqual((root / "gate.log.step-0.stdout.log").read_bytes(), b"\xff\0\n")
            self.assertEqual((root / "gate.log.step-0.stderr.log").read_bytes(), b"\xfe\n")

    def test_required_hooks_block_without_execution_or_bypass(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            marker = root / "marker"
            hook = root / ".git/hooks/pre-commit"
            hook.write_text("#!/bin/sh\ntouch marker\n")
            hook.chmod(0o755)
            with self.assertRaises(ap.TaskError):
                ap.verify_controller(root)
            self.assertFalse(marker.exists())

    def test_publication_uses_controller_and_a_verified_commit(self):
        task = ap.Task(7, "bounded", "serial", "K06", ["w.yml#job"], [], "untrusted body")
        task.base, task.verified_head, task.targets = "1" * 40, "2" * 40, ["a"]
        pipeline = ap.Pipeline.__new__(ap.Pipeline)
        pipeline.dry_run = False
        pipeline.policy = {}
        pipeline.integrate = mock.Mock()
        pipeline.changed = mock.Mock(return_value=["a"])
        def controller(args, cwd=ap.ROOT, check=True):
            if args[:2] == ["git", "rev-parse"]:
                return task.base if cwd == ap.ROOT else task.verified_head
            return ""
        with mock.patch.object(ap, "sh", side_effect=controller) as run, \
                mock.patch.object(ap, "load_policy", return_value={}), \
                mock.patch.object(ap, "require_pipeline_release"):
            pipeline.publish(task)
        publication = [c for c in run.call_args_list if c.args[0][:2] in (["git", "push"], ["gh", "pr"])]
        self.assertEqual(len(publication), 2)
        for call in publication:
            self.assertNotEqual(call.args[1] if len(call.args) > 1 else ap.ROOT, task.worktree)
        self.assertIn(task.verified_head + ":refs/heads/" + task.branch, publication[0].args[0])
        self.assertIn("--draft", publication[1].args[0])
        self.assertNotIn("untrusted body", publication[1].args[0])


if __name__ == "__main__":
    unittest.main()

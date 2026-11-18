import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
import unittest.mock as mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import agent_gate
import agent_pipeline as ap
from agent_sandbox import SandboxError, SandboxResourceExceeded

ROOT = Path(__file__).resolve().parents[2]
HAS_YAML = importlib.util.find_spec("yaml") is not None
WORKFLOW = """
jobs:
  check:
    env:
      MACHINE: q35
      OUT: ${{ github.workspace }}/build/x
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      - name: Toolchain
        run: rustup toolchain install 1.90.0
      - name: Emulator
        run: sudo apt-get install -y qemu
      - name: Test
        run: echo "$MACHINE $OUT" > out.txt
  supported:
    env:
      MACHINE: q35
      OUT: ${{ github.workspace }}/build/x
    steps:
      - name: Test
        run: echo "$MACHINE $OUT" > out.txt
  uses:
    steps:
      - name: Checkout
        uses: actions/checkout@v4
  setup:
    steps:
      - name: Toolchain
        run: rustup toolchain install 1.90.0
  conditional:
    steps:
      - name: Diagnostics
        if: ${{ always() }}
        run: echo diag
  empty:
    steps:
      - name: Missing command
  bad:
    steps:
      - run: echo ${{ matrix.x }}
"""


@unittest.skipUnless(HAS_YAML, "PyYAML not installed")
class Gate(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.workflows = self.tmp / "wf"
        self.workflows.mkdir()
        (self.workflows / "w.yml").write_text(WORKFLOW)
        self.env = {"PATH": os.environ["PATH"]}
        self.sandbox = mock.Mock()
        self.sandbox.run.return_value = subprocess.CompletedProcess([], 0, "", "")

    def gate(self, commands, env=None):
        with mock.patch.object(agent_gate, "controller_source_commit", return_value="a" * 40):
            return agent_gate.run_gate(commands, self.tmp, self.tmp / "g.log", env or self.env,
                                       sandbox=self.sandbox, source_paths=[])

    def test_supported_run_step_expands_workspace(self):
        commands = agent_gate.resolve(["w.yml#supported"], self.workflows)
        self.assertEqual([c[0] for c in commands.commands], ["Test", "progress.py --check"])
        self.assertEqual(self.gate(commands.commands[:1]), 0)
        child_env = self.sandbox.run.call_args.kwargs["env"]
        self.assertEqual(child_env["MACHINE"], "q35")
        self.assertEqual(child_env["OUT"], "/workspace/build/x")
        self.assertEqual(child_env["NOS_SOURCE_RECEIPT"],
                         "/workspace/" + agent_gate.SOURCE_RECEIPT_NAME)
        stage = self.sandbox.run.call_args.args[1]
        receipt = json.loads((stage / agent_gate.SOURCE_RECEIPT_NAME).read_text())
        self.assertEqual(receipt["source_commit"], "a" * 40)
        self.assertEqual(receipt["source_sha256"], agent_gate.source_hash(self.tmp))
        self.assertFalse((self.tmp / "out.txt").exists())

    def test_refuses_unmodelled_steps_instead_of_skipping_them(self):
        expected = {
            "uses": "uses step needs an explicit local adapter",
            "setup": "setup step needs an explicit local adapter",
            "conditional": "conditional step needs an explicit local adapter",
            "empty": "step has no runnable script",
        }
        for job, message in expected.items():
            with self.subTest(job=job), self.assertRaisesRegex(ValueError, message):
                agent_gate.resolve([f"w.yml#{job}"], self.workflows)

    def test_refuses_unknown_context_and_bad_references(self):
        for ref in ("w.yml#bad", "w.yml#missing", "../w.yml#check", "w.yml", "nope.yml#check"):
            with self.subTest(ref=ref), self.assertRaises((ValueError, KeyError)):
                agent_gate.resolve([ref], self.workflows)

    def test_failure_is_exit_two_and_stops(self):
        commands = [("one", "exit 3", {}), ("two", "touch two", {})]
        failed = subprocess.CompletedProcess([], 3, "", "failed\n")
        failed.resource_status = {
            "profile": "gate", "unit": "fixture.scope", "LoadState": "loaded",
            "Result": "oom-kill",
            "MemoryPeak": "67108864", "verified": {"memory_max": "67108864"},
        }
        self.sandbox.run.return_value = failed
        code = self.gate(commands)
        self.assertEqual(code, agent_gate.FAIL_EXIT)
        self.assertEqual(self.sandbox.run.call_count, 1)
        log = (self.tmp / "g.log").read_text()
        self.assertIn("## FAIL: one (exit 3)", log)
        self.assertIn('"Result": "oom-kill"', log)
        self.assertIn('"memory_max": "67108864"', log)

    def test_finalizer_runs_after_success_with_the_primary_status(self):
        plan = agent_gate.GatePlan(
            [("one", "true", {})], finalizers=[("collect", "true", {})],
            timeout_seconds=1800, finalizer_reserve_seconds=60)
        self.assertEqual(self.gate(plan), 0)
        self.assertEqual(self.sandbox.run.call_count, 2)
        self.assertIn("export CI_TEST_STATUS=success\n",
                      self.sandbox.run.call_args_list[1].args[0][-1])

    def test_ordinary_failure_skips_later_work_but_runs_the_finalizer(self):
        failed = subprocess.CompletedProcess([], 9, "", "failed\n")
        failed.resource_status = {"Result": "oom-kill", "MemoryPeak": "67108864"}
        passed = subprocess.CompletedProcess([], 0, "", "")
        self.sandbox.run.side_effect = [failed, passed]
        plan = agent_gate.GatePlan(
            [("one", "exit 9", {}), ("never", "true", {})],
            finalizers=[("collect", "true", {})],
            timeout_seconds=1800, finalizer_reserve_seconds=60)

        self.assertEqual(self.gate(plan), agent_gate.FAIL_EXIT)
        self.assertEqual(self.sandbox.run.call_count, 2)
        finalizer = self.sandbox.run.call_args_list[1].args[0][-1]
        self.assertIn("export CI_TEST_STATUS=failure\n", finalizer)
        self.assertTrue(finalizer.endswith("true"))

    def test_timeout_and_resource_overflow_still_run_the_finalizer(self):
        timeout = subprocess.TimeoutExpired(["bash"], 10, output="partial", stderr="")
        timeout.stdout_bytes, timeout.stderr_bytes = b"partial", b""
        overflow = SandboxResourceExceeded("captured output exceeded")
        overflow.stdout_bytes, overflow.stderr_bytes = b"bounded", b""
        overflow.resource_status = {"Result": "oom-kill"}
        for error in (timeout, overflow):
            with self.subTest(error=type(error).__name__):
                self.sandbox.run.reset_mock()
                self.sandbox.run.side_effect = [error, subprocess.CompletedProcess([], 0, "", "")]
                plan = agent_gate.GatePlan(
                    [("one", "true", {})], finalizers=[("collect", "true", {})],
                    timeout_seconds=1800, finalizer_reserve_seconds=60)
                self.assertEqual(self.gate(plan), agent_gate.FAIL_EXIT)
                self.assertEqual(self.sandbox.run.call_count, 2)
                self.assertIn("export CI_TEST_STATUS=failure\n",
                              self.sandbox.run.call_args_list[1].args[0][-1])

    def test_finalizer_failure_does_not_replace_the_primary_failure(self):
        self.sandbox.run.side_effect = [
            subprocess.CompletedProcess([], 7, "", "primary\n"),
            subprocess.CompletedProcess([], 8, "", "finalizer\n"),
        ]
        plan = agent_gate.GatePlan(
            [("one", "exit 7", {})], finalizers=[("collect", "exit 8", {})],
            timeout_seconds=1800, finalizer_reserve_seconds=60)
        self.assertEqual(self.gate(plan), agent_gate.FAIL_EXIT)
        log = (self.tmp / "g.log").read_text()
        self.assertLess(log.index("primary"), log.index("finalizer"))
        self.assertIn("primary", log)
        self.assertIn("finalizer", log)

    def test_total_deadline_reserves_time_for_the_finalizer(self):
        plan = agent_gate.GatePlan(
            [("one", "true", {})], finalizers=[("collect", "true", {})],
            timeout_seconds=1800, finalizer_reserve_seconds=60)
        with mock.patch.object(agent_gate.time, "monotonic", return_value=100.0):
            self.assertEqual(self.gate(plan), 0)
        ordinary = self.sandbox.run.call_args_list[0].kwargs["timeout"]
        finalizer = self.sandbox.run.call_args_list[1].kwargs["timeout"]
        self.assertGreater(ordinary, 0)
        self.assertLessEqual(ordinary, 1740)
        self.assertGreater(finalizer, 0)
        self.assertLessEqual(finalizer, 60)

    def test_finalizers_require_one_explicit_bounded_reserve(self):
        finalizer = [("collect", "true", {})]
        with self.assertRaisesRegex(ValueError, "finalizer reserve"):
            agent_gate.GatePlan([], finalizers=finalizer)
        with self.assertRaises(ValueError):
            agent_gate.GatePlan([], finalizers=finalizer * 2,
                                timeout_seconds=1800, finalizer_reserve_seconds=60)

    def test_gate_retains_bounded_resource_overflow_streams(self):
        error = SandboxResourceExceeded("captured output exceeded")
        error.stdout_bytes, error.stderr_bytes = b"bounded stdout", b"bounded stderr"
        error.resource_status = {"profile": "gate", "unit": "fixture.scope"}
        self.sandbox.run.side_effect = error
        self.assertEqual(self.gate([("one", "true", {})]), agent_gate.FAIL_EXIT)
        self.assertEqual((self.tmp / "g.log.step-0.resource.stdout.log").read_bytes(), b"bounded stdout")
        self.assertEqual((self.tmp / "g.log.step-0.resource.stderr.log").read_bytes(), b"bounded stderr")

    def test_primary_and_finalizer_resource_streams_are_kept_separately(self):
        primary = SandboxResourceExceeded("primary overflow")
        primary.stdout_bytes, primary.stderr_bytes = b"primary out", b"primary err"
        primary.resource_status = {"Result": "oom-kill"}
        finalizer = SandboxResourceExceeded("finalizer overflow")
        finalizer.stdout_bytes, finalizer.stderr_bytes = b"finalizer out", b"finalizer err"
        finalizer.resource_status = {"Result": "failed"}
        self.sandbox.run.side_effect = [primary, finalizer]
        plan = agent_gate.GatePlan(
            [("one", "true", {})], finalizers=[("collect", "true", {})],
            timeout_seconds=1800, finalizer_reserve_seconds=60)

        self.assertEqual(self.gate(plan), agent_gate.FAIL_EXIT)
        self.assertEqual((self.tmp / "g.log.step-0.resource.stdout.log").read_bytes(),
                         b"primary out")
        self.assertEqual((self.tmp / "g.log.step-0.resource.stderr.log").read_bytes(),
                         b"primary err")
        self.assertEqual((self.tmp / "g.log.finalizer-1.resource.stdout.log").read_bytes(),
                         b"finalizer out")
        self.assertEqual((self.tmp / "g.log.finalizer-1.resource.stderr.log").read_bytes(),
                         b"finalizer err")

    def test_generic_sandbox_failure_aborts_without_a_finalizer(self):
        self.sandbox.run.side_effect = SandboxError("isolated resource cleanup failed")
        plan = agent_gate.GatePlan(
            [("one", "true", {})], finalizers=[("collect", "true", {})],
            timeout_seconds=1800, finalizer_reserve_seconds=60)

        self.assertEqual(self.gate(plan), agent_gate.FAIL_EXIT)
        self.sandbox.run.assert_called_once()
        self.assertNotIn("## finalizer:", (self.tmp / "g.log").read_text())

    def test_disallowed_step_environment_still_runs_the_finalizer(self):
        plan = agent_gate.GatePlan(
            [("one", "true", {"UNAVAILABLE_IN_ISOLATION": "x"})],
            finalizers=[("collect", "true", {})],
            timeout_seconds=1800, finalizer_reserve_seconds=60)

        self.assertEqual(self.gate(plan), agent_gate.FAIL_EXIT)
        self.sandbox.run.assert_called_once()
        self.assertIn("export CI_TEST_STATUS=failure\n",
                      self.sandbox.run.call_args.args[0][-1])

    def test_gate_blocks_a_commit_change_while_staging(self):
        with mock.patch.object(agent_gate, "controller_source_commit",
                               side_effect=["a" * 40, "b" * 40]):
            code = agent_gate.run_gate([], self.tmp, self.tmp / "g.log", self.env,
                                       sandbox=self.sandbox, source_paths=[])
        self.assertEqual(code, agent_gate.FAIL_EXIT)
        self.assertIn("source commit changed", (self.tmp / "g.log").read_text())
        self.sandbox.run.assert_not_called()

    def test_gate_blocks_a_source_change_while_staging(self):
        source = self.tmp / "source.txt"
        source.write_text("before")
        copy = agent_gate.stage_agent_source

        def mutate(root, stage, paths):
            copy(root, stage, paths)
            source.write_text("after")

        with mock.patch.object(agent_gate, "controller_source_commit", return_value="a" * 40), \
                mock.patch.object(agent_gate, "stage_agent_source", side_effect=mutate):
            code = agent_gate.run_gate([], self.tmp, self.tmp / "g.log", self.env,
                                       sandbox=self.sandbox, source_paths=["source.txt"])
        self.assertEqual(code, agent_gate.FAIL_EXIT)
        self.assertIn("gate source changed", (self.tmp / "g.log").read_text())
        self.sandbox.run.assert_not_called()

    def test_build_dirs_come_from_the_checkout_not_the_caller(self):
        env = dict(self.env, CARGO_TARGET_DIR="/shared", OS_OVMF_CODE="/local/ovmf")
        steps = [("s", 'echo "${CARGO_TARGET_DIR:-unset} $OS_OVMF_CODE" > o', {"OS_OVMF_CODE": "/usr/share/x"})]
        self.assertEqual(self.gate(steps, env), 0)
        child_env = self.sandbox.run.call_args.kwargs["env"]
        self.assertNotIn("CARGO_TARGET_DIR", child_env)
        self.assertEqual(child_env["OS_OVMF_CODE"], "/local/ovmf")

    def test_bash_receives_pipefail_without_a_host_shell(self):
        self.assertEqual(self.gate([("p", "false | cat", {})]), 0)
        command = self.sandbox.run.call_args.args[0]
        self.assertEqual(command, ["/bin/bash", "--noprofile", "--norc", "-eo", "pipefail", "-c", "false | cat"])

    def test_condensed_without_rtk_returns_the_tail(self):
        log = self.tmp / "g.log"
        log.write_text("".join(f"line {i}\n" for i in range(300)))
        text = agent_gate.condensed(log, lines=5)
        self.assertEqual(text.splitlines(), [f"line {i}" for i in range(295, 300)])

    def test_real_ci_jobs_refuse_unmodelled_setup(self):
        for job in ("host-quality", "qemu-smoke"):
            with self.subTest(job=job), self.assertRaisesRegex(ValueError, "explicit local adapter"):
                agent_gate.resolve([f"ci.yml#{job}"], ROOT / ".github" / "workflows")

    def test_standalone_gate_rejects_invalid_engine_policy_before_sandbox(self):
        invalid = [
            {"schema": 2, "enabled": True},
            {"schema": 2, "enabled": True, "engines": []},
            {"schema": 2, "enabled": True, "engines": ["codex", "codex"]},
            {"schema": 2, "enabled": True, "engines": [7]},
            {"schema": 2, "enabled": True, "engines": ["unknown"]},
        ]
        manifest = self.tmp / ".github" / "agent-pipeline.json"
        manifest.parent.mkdir()
        for policy in invalid:
            with self.subTest(engines=policy.get("engines")):
                manifest.write_text(json.dumps(policy))
                with mock.patch.object(agent_gate, "ROOT", self.tmp), \
                        mock.patch.object(agent_gate, "Sandbox") as sandbox:
                    self.assertEqual(agent_gate.run_gate([], self.tmp, self.tmp / "g.log"),
                                     agent_gate.FAIL_EXIT)
                sandbox.assert_not_called()

    def test_pipeline_translates_gate_resolution_failure_to_task_error(self):
        pipeline = ap.Pipeline.__new__(ap.Pipeline)
        task = ap.Task(7, "t", "parallel", "K06", ["ci.yml#host-quality"], [], "")
        with mock.patch.object(ap, "require_pipeline_release"), \
                mock.patch.object(ap, "resolve", side_effect=ValueError("unsupported gate")), \
                self.assertRaisesRegex(ap.TaskError, "gate blocked: unsupported gate"):
            pipeline.execute(task)


def issue(body, labels=("pipeline:parallel",), number=7):
    return {"number": number, "title": "t", "body": body, "labels": [{"name": l} for l in labels]}


class Planning(unittest.TestCase):
    BODY = "Criterium: K06\nGate: ci.yml#host-quality native-smp.yml#native-smp\nAfhankelijk van: #3, #4\n"

    def test_parses_a_valid_issue(self):
        task = ap.parse_issue(issue(self.BODY), {"K06"})
        self.assertEqual((task.mode, task.criterion, task.deps), ("parallel", "K06", [3, 4]))
        self.assertEqual(task.gates, ["ci.yml#host-quality", "native-smp.yml#native-smp"])

    def test_rejects_closed_criteria_bad_gates_and_label_mixes(self):
        cases = [issue(self.BODY, labels=("pipeline:serial", "pipeline:parallel")),
                 issue(self.BODY.replace("K06", "K01")),
                 issue("Criterium: K06\nGate: rm -rf /\n"),
                 issue("Criterium: K06\n")]
        for case in cases:
            with self.subTest(case=case["body"]), self.assertRaises(ValueError):
                ap.parse_issue(case, {"K06"})

    def test_english_fields_match_existing_dutch_tasks(self):
        """Accept the English fields without changing legacy task semantics."""
        english = self.BODY.replace("Criterium:", "Criterion:").replace("Afhankelijk van:", "Depends on:")
        old = ap.parse_issue(issue(self.BODY), {"K06"})
        new = ap.parse_issue(issue(english), {"K06"})
        self.assertEqual((new.mode, new.criterion, new.gates, new.deps),
                         (old.mode, old.criterion, old.gates, old.deps))

    def test_duplicate_criterion_aliases_are_rejected(self):
        """Reject repeated criterion fields in either supported language."""
        for extra in (
            "Criterion: K06\n",
            "Criterion: K07\n",
            "Criterium: K06\n",
        ):
            with self.subTest(extra=extra), self.assertRaises(ValueError):
                ap.parse_issue(issue(self.BODY + extra), {"K06", "K07"})

    def test_open_criteria_come_from_progress_json(self):
        progress = {"tracks": [{"gates": [{"id": "K01", "status": "accepted"}, {"id": "K06", "status": "open"}]}]}
        self.assertEqual(ap.open_criteria(progress), {"K06"})

    def test_spec_block_is_validated(self):
        ok = 'x\n```json\n{"targets": ["kernel/src/a.rs"], "reads": ["crates/b/src/lib.rs"]}\n```'
        self.assertEqual(ap.parse_spec(ok), (["kernel/src/a.rs"], ["crates/b/src/lib.rs"]))
        for bad in ("no block", '```json\n{"targets": []}\n```',
                    '```json\n{"targets": ["../x"]}\n```', '```json\n{"targets": ["/etc/x"]}\n```',
                    '```json\n{"targets": [".github/workflows/ci.yml"]}\n```'):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                ap.parse_spec(bad)

    def test_instruction_policy_is_protected_at_every_depth(self):
        for path in ("AGENTS.md", "ui/AGENTS.md", "services/sub/AGENTS.md",
                     "CLAUDE.md", "tools/CLAUDE.md", ".codex/config.toml"):
            with self.subTest(path=path):
                self.assertTrue(ap.is_protected(path))
                self.assertIn(path, ap.out_of_scope([path], [path]))
                spec = 'spec\n' + '```json\n' + json.dumps({"targets": [path], "reads": []}) + '\n```'
                with self.assertRaises(ValueError):
                    ap.parse_spec(spec)
        for path in ("tools/roadmap_runner.py", "tools/tests/test_agent_pipeline.py",
                     "docs/PROJECT-STATE.md", "docs/history/accepted/receipt.json",
                     "docs/history/file.md", "services/keyring/vendor/locked.rs",
                     "ui/prototype/tokens.css", "ui/design/system/tokens.css",
                     "plugins/token-guard/LICENSE", "tools/update-keys/TEST-key",
                     "CODEX-WORKFLOW.md", "README.md", "build/generated.txt"):
            with self.subTest(path=path):
                self.assertTrue(ap.is_protected(path))
        # New task-owned verification children remain writable; accepted evidence is
        # protected by source-bound review and by using new receipt destinations.
        self.assertFalse(ap.is_protected("docs/verification/new-receipt/report.json"))
        self.assertFalse(ap.is_protected("ui/AGENTS.md.example"))
        self.assertFalse(ap.is_protected("docs/CODEX-PARITY.md"))

    def test_lockfiles_also_need_explicit_owner_authority(self):
        self.assertEqual(ap.out_of_scope(["a.rs", "Cargo.lock", "b.rs", "AGENTS.md"], ["a.rs", "AGENTS.md"]),
                         ["Cargo.lock", "b.rs", "AGENTS.md"])
        self.assertEqual(ap.out_of_scope(["Cargo.lock"], ["Cargo.lock"]), [])

    def test_parallel_batches_have_disjoint_access_sets(self):
        def task(n, targets, reads=()):
            t = ap.Task(n, "", "parallel", "K06", ["ci.yml#x"], [], "")
            t.targets, t.reads = list(targets), list(reads)
            return t
        a, b, c, d, e = (task(1, ["x"]), task(2, ["y"], ["x"]), task(3, ["z"]),
                         task(4, ["w"]), task(5, ["v"]))
        self.assertEqual([t.number for t in ap.pick_parallel([a, b, c, d, e])], [1, 3, 4])

    def test_stale_reports_changed_and_removed_paths(self):
        self.assertEqual(ap.stale({"a": "1", "b": "2", "c": None}, {"a": "1", "b": "3"}), ["b"])

    def test_policy_requires_a_valid_owner_authorized_engine_order(self):
        valid = {"schema": 2, "enabled": True, "engines": ["claude"], "tasks": {}}
        invalid = [
            {key: value for key, value in valid.items() if key != "engines"},
            {**valid, "engines": []},
            {**valid, "engines": ["claude", "claude"]},
            {**valid, "engines": [7]},
            {**valid, "engines": ["unknown"]},
        ]
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / ".github").mkdir()
            manifest = root / ".github" / "agent-pipeline.json"
            with mock.patch.object(ap, "ROOT", root):
                for policy in invalid:
                    with self.subTest(engines=policy.get("engines")):
                        manifest.write_text(json.dumps(policy))
                        with self.assertRaises(ap.TaskError):
                            ap.load_policy()
                manifest.write_text(json.dumps(valid))
                self.assertEqual(ap.load_policy()["engines"], ["claude"])

    def test_pipeline_binds_engine_selection_and_order_before_controller_access(self):
        policy = {"schema": 2, "enabled": True, "engines": ["claude", "codex"], "tasks": {}}
        controller = mock.Mock()
        with mock.patch.object(ap, "load_policy", return_value=policy), \
                mock.patch.object(ap, "verify_controller", controller), \
                mock.patch.object(ap, "sh") as shell, \
                mock.patch.object(ap, "Sandbox") as sandbox, \
                mock.patch.object(ap, "run_agent") as model:
            for engines in (["codex", "claude"], ["claude"],
                            ["claude", "codex", "copilot"]):
                with self.subTest(engines=engines), \
                        self.assertRaisesRegex(ap.TaskError, "match owner authorization"):
                    ap.Pipeline(engines, False, True)
                controller.assert_not_called()
                shell.assert_not_called()
                sandbox.assert_not_called()
                model.assert_not_called()

    def test_exact_engine_order_reaches_the_next_controller_check(self):
        policy = {"schema": 2, "enabled": True, "engines": ["claude"], "tasks": {}}
        with mock.patch.object(ap, "load_policy", return_value=policy), \
                mock.patch.object(ap, "verify_controller", side_effect=ap.TaskError("next check")) as controller, \
                mock.patch.object(ap, "sh") as shell, \
                self.assertRaisesRegex(ap.TaskError, "next check"):
            ap.Pipeline(["claude"], False, True)
        controller.assert_called_once_with()
        shell.assert_not_called()

    def test_engine_order_does_not_rotate_between_attempts(self):
        pipeline = object.__new__(ap.Pipeline)
        pipeline.engines = ["claude", "codex", "copilot"]
        expected = ["claude", "codex", "copilot"]
        self.assertEqual([pipeline.order(), pipeline.order()], [expected, expected])


class GitAndEngines(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())

    def git(self, *args):
        subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", *args],
                       cwd=self.tmp, check=True, capture_output=True)

    def test_snapshot_detects_a_changed_read_dependency(self):
        self.git("init", "-q")
        (self.tmp / "a").write_text("1")
        self.git("add", "a")
        self.git("commit", "-qm", "1")
        before = ap.snapshot(["a", "missing"], "HEAD", self.tmp)
        self.assertIsNone(before["missing"])
        (self.tmp / "a").write_text("2")
        self.git("commit", "-qam", "2")
        self.assertEqual(ap.stale(before, ap.snapshot(["a", "missing"], "HEAD", self.tmp)), ["a"])

    def test_object_only_snapshot_does_not_run_configured_content_filters(self):
        self.git("init", "-q")
        (self.tmp / "a").write_text("source")
        self.git("add", "a")
        self.git("commit", "-qm", "fixture")
        self.git("config", "filter.fixture.clean", "nonexistent-fixture-helper")
        self.assertIsNotNone(ap.snapshot(["a"], "HEAD", self.tmp)["a"])
        with self.assertRaises(ap.TaskError):
            ap.sh(["git", "status", "--porcelain"], self.tmp)

    def test_missing_promisor_object_cannot_start_a_lazy_fetch(self):
        self.git("init", "-q")
        (self.tmp / "a").write_text("source")
        self.git("add", "a")
        self.git("commit", "-qm", "fixture")
        tree = subprocess.run(["/usr/bin/git", "rev-parse", "HEAD^{tree}"], cwd=self.tmp,
                              check=True, capture_output=True, text=True).stdout.strip()
        (self.tmp / ".git/objects" / tree[:2] / tree[2:]).unlink()
        self.git("config", "core.repositoryformatversion", "1")
        self.git("config", "extensions.partialClone", "origin")
        self.git("config", "remote.origin.promisor", "true")
        self.git("config", "remote.origin.url", (self.tmp / "nonexistent-remote").as_uri())
        trace = self.tmp / "trace.json"
        env = ap.controller_environment()
        env.pop("GIT_NO_LAZY_FETCH")  # The explicit option must suffice independently.
        env["GIT_TRACE2_EVENT"] = str(trace)
        with mock.patch.object(ap, "controller_environment", return_value=env):
            self.assertIsNone(ap.snapshot(["a"], "HEAD", self.tmp)["a"])
        events = [json.loads(line) for line in trace.read_text().splitlines()]
        self.assertFalse(any(event["event"] == "child_start" for event in events))

    def test_fails_over_only_on_rate_limits(self):
        results = {"limited": (1, "rate limit exceeded"), "broken": (1, "syntax error"),
                   "fine": (0, "")}
        sandbox = mock.Mock()
        sandbox.rootfs = self.tmp / "runtime"
        (sandbox.rootfs / "usr/bin").mkdir(parents=True)
        for name in results:
            (sandbox.rootfs / "usr/bin" / name).write_text("fixture")
        def run(command, stage, *, profile):
            self.assertEqual(profile, "agent")
            code, error = results[command[0]]
            return subprocess.CompletedProcess(command, code, "", error)
        sandbox.run.side_effect = run
        log = self.tmp / "agent.log"
        engines = {name: [name] for name in (*results, "absent")}
        with mock.patch.dict(ap.ENGINES, engines, clear=True), \
                mock.patch.object(ap, "agent_source_paths", return_value=[]):
            self.assertEqual(ap.run_agent("p", self.tmp, log, ["absent", "limited", "fine"],
                                         sandbox=sandbox, writes=["a"]), "fine")
            with self.assertRaises(ap.TaskError):
                ap.run_agent("p", self.tmp, log, ["broken", "fine"], sandbox=sandbox, writes=["a"])
            with self.assertRaises(ap.TaskError):
                ap.run_agent("p", self.tmp, log, ["limited"], sandbox=sandbox, writes=["a"])

    def test_agent_failure_logs_oom_separately_from_ordinary_exit(self):
        sandbox = mock.Mock()
        sandbox.rootfs = self.tmp / "runtime"
        (sandbox.rootfs / "usr/bin").mkdir(parents=True)
        (sandbox.rootfs / "usr/bin/broken").write_text("fixture")
        (sandbox.rootfs / "usr/bin/fine").write_text("fixture")
        failed = subprocess.CompletedProcess(["broken"], -9, "", "rate limit exceeded\n")
        failed.resource_status = {
            "profile": "agent", "unit": "fixture.scope", "LoadState": "loaded",
            "Result": "oom-kill",
            "MemoryPeak": "67108864", "verified": {"memory_max": "67108864"},
        }
        sandbox.run.return_value = failed
        log = self.tmp / "oom-agent.log"
        with mock.patch.dict(ap.ENGINES, {"broken": ["broken"], "fine": ["fine"]}, clear=True), \
                mock.patch.object(ap, "agent_source_paths", return_value=[]), \
                self.assertRaises(ap.TaskError):
            ap.run_agent("p", self.tmp, log, ["broken", "fine"], sandbox=sandbox, writes=["a"])
        self.assertEqual(sandbox.run.call_count, 1)
        text = log.read_text()
        self.assertIn('"Result": "oom-kill"', text)
        self.assertIn('"memory_max": "67108864"', text)

    def test_agent_retains_bounded_resource_overflow_streams(self):
        sandbox = mock.Mock()
        sandbox.rootfs = self.tmp / "runtime"
        (sandbox.rootfs / "usr/bin").mkdir(parents=True)
        (sandbox.rootfs / "usr/bin/broken").write_text("fixture")
        error = SandboxResourceExceeded("captured output exceeded")
        error.stdout_bytes, error.stderr_bytes = b"bounded stdout", b"bounded stderr"
        error.resource_status = {"profile": "agent", "unit": "fixture.scope"}
        sandbox.run.side_effect = error
        log = self.tmp / "overflow-agent.log"
        with mock.patch.dict(ap.ENGINES, {"broken": ["broken"]}, clear=True), \
                mock.patch.object(ap, "agent_source_paths", return_value=[]), \
                self.assertRaises(ap.TaskError):
            ap.run_agent("p", self.tmp, log, ["broken"], sandbox=sandbox, writes=["a"])
        retained = sorted(self.tmp.glob("overflow-agent-resource-*.stdout.log"))
        self.assertEqual(len(retained), 1)
        self.assertEqual(retained[0].read_bytes(), b"bounded stdout")
        self.assertEqual(retained[0].with_name(retained[0].name.replace("stdout", "stderr")).read_bytes(),
                         b"bounded stderr")


if __name__ == "__main__":
    unittest.main()

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
      - name: Diagnostics
        if: ${{ always() }}
        run: echo diag
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
        return agent_gate.run_gate(commands, self.tmp, self.tmp / "g.log", env or self.env,
                                   sandbox=self.sandbox, source_paths=[])

    def test_keeps_only_check_steps_and_expands_workspace(self):
        commands = agent_gate.resolve(["w.yml#check"], self.workflows)
        self.assertEqual([c[0] for c in commands], ["Test", "progress.py --check"])
        self.assertEqual(self.gate(commands[:1]), 0)
        child_env = self.sandbox.run.call_args.kwargs["env"]
        self.assertEqual(child_env["MACHINE"], "q35")
        self.assertEqual(child_env["OUT"], "/workspace/build/x")
        self.assertFalse((self.tmp / "out.txt").exists())

    def test_refuses_unknown_context_and_bad_references(self):
        for ref in ("w.yml#bad", "w.yml#missing", "../w.yml#check", "w.yml", "nope.yml#check"):
            with self.subTest(ref=ref), self.assertRaises((ValueError, KeyError)):
                agent_gate.resolve([ref], self.workflows)

    def test_failure_is_exit_two_and_stops(self):
        commands = [("one", "exit 3", {}), ("two", "touch two", {})]
        self.sandbox.run.return_value = subprocess.CompletedProcess([], 3, "", "failed\n")
        code = self.gate(commands)
        self.assertEqual(code, agent_gate.FAIL_EXIT)
        self.assertEqual(self.sandbox.run.call_count, 1)
        self.assertIn("## FAIL: one (exit 3)", (self.tmp / "g.log").read_text())

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

    def test_real_ci_job_matches_the_workflow(self):
        names = [c[0] for c in agent_gate.resolve(["ci.yml#host-quality"], ROOT / ".github" / "workflows")]
        self.assertIn("Root Rust host tests", names)
        self.assertNotIn("Rust 1.90.0 and targets", names)
        self.assertEqual(names[-1], "progress.py --check")


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
        def run(command, stage):
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


if __name__ == "__main__":
    unittest.main()

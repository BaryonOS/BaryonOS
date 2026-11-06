import importlib.util
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

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

    def test_keeps_only_check_steps_and_expands_workspace(self):
        commands = agent_gate.resolve(["w.yml#check"], self.workflows)
        self.assertEqual([c[0] for c in commands], ["Test", "progress.py --check"])
        self.assertEqual(agent_gate.run_gate(commands[:1], self.tmp, self.tmp / "g.log", self.env), 0)
        self.assertEqual((self.tmp / "out.txt").read_text().strip(), f"q35 {self.tmp}/build/x")

    def test_refuses_unknown_context_and_bad_references(self):
        for ref in ("w.yml#bad", "w.yml#missing", "../w.yml#check", "w.yml", "nope.yml#check"):
            with self.subTest(ref=ref), self.assertRaises((ValueError, KeyError)):
                agent_gate.resolve([ref], self.workflows)

    def test_failure_is_exit_two_and_stops(self):
        commands = [("one", "exit 3", {}), ("two", "touch two", {})]
        code = agent_gate.run_gate(commands, self.tmp, self.tmp / "g.log", self.env)
        self.assertEqual(code, agent_gate.FAIL_EXIT)
        self.assertFalse((self.tmp / "two").exists())
        self.assertIn("## FAIL: one (exit 3)", (self.tmp / "g.log").read_text())

    def test_build_dirs_come_from_the_checkout_not_the_caller(self):
        env = dict(self.env, CARGO_TARGET_DIR="/shared", OS_OVMF_CODE="/local/ovmf")
        steps = [("s", 'echo "${CARGO_TARGET_DIR:-unset} $OS_OVMF_CODE" > o', {"OS_OVMF_CODE": "/usr/share/x"})]
        self.assertEqual(agent_gate.run_gate(steps, self.tmp, self.tmp / "g.log", env), 0)
        self.assertEqual((self.tmp / "o").read_text().strip(), "unset /local/ovmf")

    def test_pipefail_catches_a_failing_producer(self):
        code = agent_gate.run_gate([("p", "false | cat", {})], self.tmp, self.tmp / "g.log", self.env)
        self.assertEqual(code, agent_gate.FAIL_EXIT)

    def test_condensed_without_rtk_returns_the_tail(self):
        log = self.tmp / "g.log"
        log.write_text("".join(f"line {i}\n" for i in range(300)))
        with mock.patch.object(agent_gate.shutil, "which", return_value=None):
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

    def test_scope_allows_targets_and_lockfile_only(self):
        self.assertEqual(ap.out_of_scope(["a.rs", "Cargo.lock", "b.rs", "AGENTS.md"], ["a.rs", "AGENTS.md"]),
                         ["b.rs", "AGENTS.md"])

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

    def test_fails_over_only_on_rate_limits(self):
        bin_dir = self.tmp / "bin"
        bin_dir.mkdir()
        scripts = {"limited": "echo 'Error: rate limit exceeded' >&2; exit 1",
                   "broken": "echo 'syntax error' >&2; exit 1",
                   "fine": "exit 0"}
        for name, body in scripts.items():
            path = bin_dir / name
            path.write_text(f"#!/bin/sh\n{body}\n")
            path.chmod(0o755)
        log = self.tmp / "agent.log"
        engines = {name: [name] for name in scripts} | {"absent": ["no-such-agent-cli"]}
        with mock.patch.dict(ap.ENGINES, engines, clear=True), \
                mock.patch.dict(os.environ, {"PATH": f"{bin_dir}:{os.environ['PATH']}"}):
            self.assertEqual(ap.run_agent("p", self.tmp, log, ["absent", "limited", "fine"]), "fine")
            with self.assertRaises(ap.TaskError):
                ap.run_agent("p", self.tmp, log, ["broken", "fine"])
            with self.assertRaises(ap.TaskError):
                ap.run_agent("p", self.tmp, log, ["limited"])


if __name__ == "__main__":
    unittest.main()

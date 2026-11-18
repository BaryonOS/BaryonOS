"""Opt-in offline transaction with real local Git and the provisioned sandbox.

Provide BARYON_SANDBOX_TEST_CONFIG with a reviewed runtime containing Python,
bash and fixture-agent. Absence is NOT RUN; an invalid supplied runtime fails.
The deterministic fixture exercises orchestration, not model-client or hosted CI parity.
"""
import contextlib
import io
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


@unittest.skipUnless(os.environ.get("BARYON_SANDBOX_TEST_CONFIG"),
                    "NOT RUN: isolated fixture runtime not provided")
class OfflineTransaction(unittest.TestCase):
    def setUp(self):
        config = json.loads(Path(os.environ["BARYON_SANDBOX_TEST_CONFIG"]).read_text())
        temporary = tempfile.TemporaryDirectory(prefix="pipeline-offline-")
        self.addCleanup(temporary.cleanup)
        private = Path(temporary.name)
        self.root = private / "controller"
        home = private / "home"
        self.root.mkdir(mode=0o700)
        home.mkdir(mode=0o700)
        environment = mock.patch.dict(os.environ, {
            "PATH": "/usr/bin:/bin", "HOME": str(home), "LANG": "C.UTF-8",
            "EXAMPLE_CONTROLLER_CREDENTIAL": "synthetic fixture",
        }, clear=True)
        environment.start()
        self.addCleanup(environment.stop)
        for module, name, value in (
            (ap, "ROOT", self.root), (ap, "RUN", self.root / "build/agent-pipeline"),
            (common, "ROOT", self.root), (agent_gate, "ROOT", self.root),
            (ap, "ENGINES", {"fixture": ["fixture-agent"]}),
            (ap, "require_pipeline_release", lambda: None),
            (ap.verify_controller, "__defaults__", (self.root,)),
            (ap.snapshot, "__defaults__", (self.root,)),
        ):
            patch = mock.patch.object(module, name, value)
            patch.start()
            self.addCleanup(patch.stop)

        self.issue = {
            "number": 7, "title": "Update the synthetic value",
            "body": "Criterion: F01\nGate: fixture.yml#check\n",
            "labels": [{"name": "pipeline:serial"}],
        }
        self.spec = ('Update only src/value.txt; retain all other source.\n'
                     '```json\n{"targets": ["src/value.txt"], "reads": []}\n```\n')
        self.git("init", "-q", "-b", "main")
        self.git("config", "user.name", "Offline Fixture")
        self.git("config", "user.email", "fixture@example.invalid")
        for directory in (".github", ".github/workflows", "docs", "src", "tools"):
            (self.root / directory).mkdir(mode=0o700)
        self.write(".gitignore", "build/\ntarget/\n")
        self.write("AGENTS.md", "Synthetic offline fixture; authority comes from the reviewed manifest.\n")
        self.write("src/value.txt", "original\n")
        self.write("docs/progress.json", json.dumps({
            "tracks": [{"gates": [{"id": "F01", "status": "open"}]}],
        }))
        self.write("tools/fixture_check.py", "\n".join([
            "import os", "from pathlib import Path",
            "assert 'EXAMPLE_CONTROLLER_CREDENTIAL' not in os.environ",
            "assert not Path('.git').exists()",
            "assert Path('src/value.txt').read_text() == 'updated\\n'",
            "for name in ('build', 'target'):",
            "    directory = Path(name)", "    directory.mkdir(exist_ok=True)",
            "    (directory / 'fixture-output').write_text('verified')",
            "print('fixture gate verified')", "",
        ]))
        self.write("tools/progress.py", "\n".join([
            "import sys", "from pathlib import Path",
            "assert sys.argv[1:] == ['--check']",
            "assert Path('docs/progress.json').is_file()",
            "print('fixture progress checked')", "",
        ]))
        self.write(".github/workflows/fixture.yml", json.dumps({
            "jobs": {"check": {"steps": [{
                "name": "fixture", "run": "python3 -B tools/fixture_check.py",
            }]}},
        }))
        self.write(".github/agent-pipeline.json", json.dumps({
            "schema": 2, "enabled": False, "engines": ["fixture"], "tasks": {},
        }))
        self.git("add", ".")
        self.git("commit", "-q", "-m", "test: seed offline fixture")
        self.seed = self.git("rev-parse", "HEAD").strip()
        task = ap.parse_issue(self.issue, {"F01"})
        self.write(".github/agent-pipeline.json", json.dumps({
            "schema": 2, "enabled": True, "engines": ["fixture"], "sandbox": config,
            "tasks": {"7": {
                "base": self.seed, "issue_sha256": ap.issue_digest(task),
                "criterion": "F01", "mode": "serial", "gates": task.gates,
                "targets": ["src/value.txt"], "reads": [],
            }},
        }))
        self.git("add", ".github/agent-pipeline.json")
        self.git("commit", "-q", "-m", "test: authorize offline fixture")
        self.main = self.git("rev-parse", "HEAD").strip()
        self.git("update-ref", "refs/remotes/origin/main", self.main)
        self.assertEqual(self.git("diff", "--name-only", self.seed, self.main).splitlines(),
                         [".github/agent-pipeline.json"])

        real_sh = ap.sh

        def metadata(args, cwd=None, check=True):
            cwd = self.root if cwd is None else cwd
            if args == ["gh", "issue", "list", "--state", "open", "--limit", "500",
                        "--json", "number,title,body,labels"]:
                self.assertEqual(cwd, self.root)
                return json.dumps([self.issue])
            if args in (["git", "fetch", "-q", "origin", "main"],
                        ["git", "ls-remote", "--heads", "origin", "agent/issue-7"]):
                self.assertEqual(cwd, self.root)
                return ""
            if args[0] != "git" or args[1] not in {
                "show", "status", "diff", "rev-parse", "worktree", "ls-files", "add", "commit",
            }:
                raise AssertionError(f"Unexpected external operation: {args[:2]}")
            return real_sh(args, cwd, check)

        for name, value in (
            ("sh", metadata),
            ("l0_prompt", lambda task, path: json.dumps({"path": path, "text": self.spec})),
            ("exec_prompt", lambda task, feedback="": json.dumps({
                "path": "src/value.txt", "text": "updated\n",
            })),
        ):
            patch = mock.patch.object(ap, name, value)
            patch.start()
            self.addCleanup(patch.stop)

    def write(self, relative, text):
        path = self.root / relative
        path.write_text(text, encoding="utf-8")
        path.chmod(0o644)

    def git(self, *args, cwd=None):
        return subprocess.run(["/usr/bin/git", *args], cwd=self.root if cwd is None else cwd,
                              env=common.controller_environment(), check=True,
                              capture_output=True, text=True).stdout

    def test_offline_transaction_reverifies_without_an_empty_commit(self):
        pipeline = ap.Pipeline(["fixture"], False, True)
        tasks = pipeline.ready_tasks()
        self.assertEqual(len(tasks), 1)
        task = tasks[0]
        self.assertEqual(task.authorized_targets, ("src/value.txt",))
        self.assertEqual(task.base, self.main)
        pipeline.prepare(task)
        self.assertEqual(task.spec, self.spec)
        self.assertEqual(json.loads((task.dir / "read_snapshot.json").read_text())["base"], self.main)
        pipeline.execute(task)
        first_head = task.verified_head
        self.assertNotEqual(first_head, self.main)

        # An unchanged verification retry must preserve the already committed head.
        pipeline.execute(task, feedback="Repeat verification without source changes.", allow_unchanged=True)
        self.assertEqual(task.verified_head, first_head)
        self.assertEqual(self.git("rev-list", "--count", f"{self.main}..HEAD", cwd=task.worktree).strip(), "1")
        pipeline.integrate(task)
        with contextlib.redirect_stdout(io.StringIO()) as output:
            pipeline.publish(task)
        self.assertIn("no push or PR", output.getvalue())
        self.assertEqual(pipeline.changed(task), ["src/value.txt"])
        self.assertEqual((task.worktree / "src/value.txt").read_text(), "updated\n")
        self.assertEqual((self.root / "src/value.txt").read_text(), "original\n")
        self.assertEqual(self.git("status", "--porcelain", cwd=task.worktree), "")
        self.assertEqual(self.git("status", "--porcelain"), "")
        self.assertEqual((task.dir / "agent.log").read_text().count("## engine fixture exit 0;"), 3)
        gate_log = (task.dir / "gate-1.log").read_text()
        self.assertIn("fixture gate verified", gate_log)
        self.assertIn("fixture progress checked", gate_log)
        self.assertIn("## PASS", gate_log)
        self.assertTrue((task.dir / "gate-1.log.step-0.stdout.log").is_file())
        self.assertTrue((task.dir / "gate-1.log.step-1.stdout.log").is_file())


if __name__ == "__main__":
    unittest.main()

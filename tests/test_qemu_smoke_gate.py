"""Host-only regression gate orchestration; no native runner or guest is executed."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SPY = r'''import json
import os
from pathlib import Path
import sys

args = sys.argv[1:]
state = Path(os.environ["SPY_STATE"])
if "-c" in args and "source_hash" in args[args.index("-c") + 1]:
    print(("b" if state.exists() else "a") * 64)
    raise SystemExit(0)
with Path(os.environ["SPY_LOG"]).open("a") as log:
    log.write(json.dumps({"tool": Path(sys.argv[0]).name, "args": args,
        "build": os.environ.get("NOS_BUILD_DIR"),
        "target": os.environ.get("NOS_TARGET_DIR"),
        "cargo_target": os.environ.get("CARGO_TARGET_DIR")}) + "\n")
if os.environ.get("SPY_DRIFT") in args:
    state.write_text("source changed")
if os.environ.get("SPY_FAIL") in args:
    raise SystemExit(23)
print("HOST_ONLY command spy; no test or guest execution")
'''


class QemuSmokeGateTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        (self.root / "tools").mkdir()
        self.wrapper = self.root / "tools/qemu-smoke-test.sh"
        shutil.copy2(ROOT / "tools/qemu-smoke-test.sh", self.wrapper)
        binaries = self.root / "bin"
        binaries.mkdir()
        for tool in ("python3", "cargo"):
            path = binaries / tool
            path.write_text(f"#!{sys.executable}\n" + SPY)
            path.chmod(0o755)
        self.log = self.root / "commands.jsonl"
        self.build = self.root / "evidence"
        self.env = dict(os.environ, PATH=str(binaries) + os.pathsep + os.environ["PATH"],
                        NOS_BUILD_DIR=str(self.build), NOS_TARGET_DIR=str(self.root / "target"),
                        SPY_LOG=str(self.log), SPY_STATE=str(self.root / "drift"))
        self.env.pop("SPY_DRIFT", None)
        self.env.pop("SPY_FAIL", None)

    def execute(self, **changes):
        result = subprocess.run(["bash", str(self.wrapper)], env=dict(self.env, **changes),
                                text=True, capture_output=True, timeout=30)
        calls = [json.loads(line) for line in self.log.read_text().splitlines()] if self.log.exists() else []
        return result, calls

    def test_all_required_native_gates_run_sequentially_in_isolated_directories(self):
        result, calls = self.execute()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        native = [call for call in calls if any(arg.startswith("tools/") and arg.endswith(".py") for arg in call["args"])]
        self.assertEqual([call["args"] for call in native], [
            ["-B", "tools/test_milestones.py", "--through", "m2"],
            ["-B", "-u", "tools/test_supervisor.py"],
            ["-B", "-u", "tools/test_supervisor.py", "--smp"],
            ["-B", "tools/test_resource_pressure.py"],
            ["-B", "tools/build.py", "--kernel-feature", "native-rtc"],
            ["-B", "tools/run_rtc.py", "--case", "all"],
        ])
        self.assertEqual(len({call["build"] for call in native}), 5)
        self.assertEqual(native[-1]["build"], native[-2]["build"])
        for call in native:
            self.assertTrue(Path(call["build"]).is_relative_to(self.build / "regressions"))
            self.assertEqual(call["target"], call["cargo_target"])
        self.assertIn("Regression gates passed on source_sha256=" + "a" * 64, result.stdout)
        self.assertTrue((self.build / "regressions/logs/rtc.log").is_file())

    def test_failed_native_runner_stops_following_gates_and_retains_log(self):
        result, calls = self.execute(SPY_FAIL="tools/test_supervisor.py")
        self.assertEqual(result.returncode, 23, result.stdout + result.stderr)
        self.assertTrue(any("tools/test_supervisor.py" in call["args"] for call in calls))
        self.assertFalse(any("tools/test_resource_pressure.py" in call["args"] for call in calls))
        self.assertNotIn("Regression gates passed", result.stdout)
        self.assertTrue((self.build / "regressions/logs/k05.log").is_file())

    def test_source_drift_rejects_successful_runner_before_next_gate(self):
        result, calls = self.execute(SPY_DRIFT="tools/test_supervisor.py")
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertIn("Source changed during regression stage k05", result.stderr)
        self.assertFalse(any("tools/test_resource_pressure.py" in call["args"] for call in calls))
        self.assertNotIn("Regression gates passed", result.stdout)

    def test_existing_evidence_is_never_reused_or_removed(self):
        directory = self.build / "regressions"
        directory.mkdir(parents=True)
        marker = directory / "retained.txt"
        marker.write_text("original evidence")
        result, calls = self.execute()
        self.assertEqual(result.returncode, 2)
        self.assertEqual(calls, [])
        self.assertEqual(marker.read_text(), "original evidence")


if __name__ == "__main__":
    unittest.main()

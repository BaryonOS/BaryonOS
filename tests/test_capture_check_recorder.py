"""Host regressions for recorder exit status and preserved failure evidence.

Every case runs the real recorder and a tiny real unittest subprocess in a
temporary directory. These synthetic host fixtures are not native/VM evidence.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


RECORDER = (Path(__file__).resolve().parents[1]
            / "docs/verification/m6c-review/integration-followup/run_capture_check.py")


def digest(data):
    return hashlib.sha256(data).hexdigest()


class CaptureCheckRecorderTests(unittest.TestCase):
    """A receipt is saved before nonzero child status or input drift is rejected."""

    def run_scenario(self, *, child_fails, changes_input):
        with tempfile.TemporaryDirectory(prefix="capture-recorder-") as temporary:
            root = Path(temporary)
            candidate = root / "candidate"
            for directory in (candidate / "tools", candidate / "tests",
                              root / "records", root / "tmp"):
                directory.mkdir(parents=True)
            recorder = root / "run_capture_check.py"
            shutil.copyfile(RECORDER, recorder)
            source = candidate / "tools/input.bin"
            source.write_bytes(b"original synthetic host input")
            fixture = candidate / "tests/test_m6_capture.py"
            fixture.write_text(
                "import unittest\n"
                "from pathlib import Path\n"
                "class SyntheticHostCapture(unittest.TestCase):\n"
                "    def test_fixture(self):\n"
                "        print('synthetic host child output')\n"
                f"        if {changes_input!r}:\n"
                "            Path('tools/input.bin').write_bytes(b'changed by child')\n"
                f"        if {child_fails!r}:\n"
                "            self.fail('deliberate synthetic child failure')\n",
                encoding="utf-8",
            )
            expected = {path.relative_to(candidate).as_posix(): digest(path.read_bytes())
                        for path in (source, fixture)}
            provenance = {
                "scope": "Synthetic host-only recorder regression; no native build or VM.",
                "baseline_commit": "synthetic-host-fixture",
                "baseline_host_files": {name: {"sha256": value}
                                        for name, value in expected.items()},
                "overlays": [],
            }
            provenance_path = root / "provenance.json"
            provenance_path.write_text(json.dumps(provenance), encoding="utf-8")
            completed = subprocess.run(
                [sys.executable, "-B", str(recorder)], cwd=root,
                env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"),
                capture_output=True, timeout=15,
            )

            # Check the durable receipt and raw output even in failure cases.
            record = json.loads((root / "records/combined-capture.json").read_bytes())
            summary = json.loads(completed.stdout)
            self.assertEqual(completed.stderr, b"")
            self.assertEqual(record["returncode"], 1 if child_fails else 0)
            self.assertEqual(record["inputs_unchanged"], not changes_input)
            self.assertEqual(record["input_sha256_before"], expected)
            actual = {path.relative_to(candidate).as_posix(): digest(path.read_bytes())
                      for path in (source, fixture)}
            self.assertEqual(record["input_sha256_after"], actual)
            self.assertEqual(record["recorder_sha256"], digest(RECORDER.read_bytes()))
            self.assertEqual(record["provenance_sha256"], digest(provenance_path.read_bytes()))
            self.assertEqual(summary["returncode"], record["returncode"])
            self.assertEqual(summary["inputs_unchanged"], record["inputs_unchanged"])
            self.assertEqual(record["test_method_counts_by_module"], {"test_m6_capture": 1})
            for stream in ("stdout", "stderr"):
                output = record["output"][stream]
                path = root / "records" / f"combined-capture.{stream}.txt"
                raw = path.read_bytes()
                self.assertEqual(output["path"], str(path))
                self.assertEqual(output["bytes"], len(raw))
                self.assertEqual(output["sha256"], digest(raw))
            child_stderr = (root / "records/combined-capture.stderr.txt").read_text()
            self.assertIn("Ran 1 test", child_stderr)
            self.assertIn("FAILED (failures=1)" if child_fails else "OK", child_stderr)
            self.assertEqual(completed.returncode, 1 if child_fails or changes_input else 0)

    def test_success_with_unchanged_inputs_exits_zero(self):
        self.run_scenario(child_fails=False, changes_input=False)

    def test_child_failure_records_evidence_and_exits_one(self):
        self.run_scenario(child_fails=True, changes_input=False)

    def test_input_mutation_records_evidence_and_exits_one(self):
        self.run_scenario(child_fails=False, changes_input=True)

    def test_child_failure_and_mutation_record_both_failures(self):
        self.run_scenario(child_fails=True, changes_input=True)


if __name__ == "__main__":
    unittest.main()

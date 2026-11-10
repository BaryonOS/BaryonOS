"""CI-EVIDENCE: bounded storage, honest provenance and inert log fallback."""

from __future__ import annotations

import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / "tools" / "ci_evidence.py"
VOLUME_REPORT = "build/ci-m6e/volume-gate.json"
VOLUME_AUXILIARY = "build/ci-m6e/run/volume-normal/result.json"
SPEC = importlib.util.spec_from_file_location("ci_evidence", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
evidence = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(evidence)


class EvidenceTests(unittest.TestCase):
    """Exercise bounded collection and honest fallback reporting in temporary files."""

    def setUp(self):
        """Isolate the workspace, output and runner environment files for each test."""
        self.temporary = tempfile.TemporaryDirectory(prefix="ci-evidence-test-")
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)
        self.root = self.base / "workspace"
        self.root.mkdir()
        self.output = self.base / "compact"
        self.console = io.StringIO()
        self.env = {
            "GITHUB_WORKSPACE": str(self.root),
            "GITHUB_SHA": "a" * 40,
            "GITHUB_RUN_ID": "12345",
            "GITHUB_RUN_ATTEMPT": "2",
            "GITHUB_JOB": "native-test",
            "GITHUB_OUTPUT": str(self.base / "github-output"),
            "GITHUB_STEP_SUMMARY": str(self.base / "summary.md"),
        }

    def write(self, path, data):
        """Create UTF-8 or binary fixture bytes under the temporary workspace."""
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data.encode("utf-8") if isinstance(data, str) else data)
        return target

    def collect(
        self, paths="build/", status="failure", output=None, required_paths="",
    ):
        """Collect isolated fixtures with a failing pre-archive status by default."""
        return evidence.collect(
            name="native-evidence", paths=paths, required_paths=required_paths,
            output=output or self.output,
            status=status, workspace=self.root, environ=self.env, stdout=self.console,
        )

    def test_exact_json_bytes_hash_and_prearchive_status(self):
        """Preserve exact JSON bytes and hashes without turning failure into acceptance."""
        original = b'{\n  "passed": false, "count": 3, "label": "caf\xc3\xa9"\n}\n'
        source = self.write("build/run/report.json", original)
        receipt = self.collect(required_paths="build/run/report.json")
        self.assertEqual(source.read_bytes(), original)
        self.assertEqual((self.output / "build/run/report.json").read_bytes(), original)
        self.assertEqual(receipt["retained"], [{
            "path": "build/run/report.json", "size_bytes": len(original),
            "sha256": hashlib.sha256(original).hexdigest(),
        }])
        self.assertEqual(receipt["job_status_before_archive"], "failure")
        self.assertEqual(receipt["required_paths"], ["build/run/report.json"])
        self.assertEqual(receipt["unretained_required_paths"], [])
        self.assertNotIn("passed", receipt)
        self.assertIn('"passed": false', self.console.getvalue())
        self.assertIn("COMPLETE FILE", self.console.getvalue())
        stored = json.loads((self.output / "_meta/receipt.json").read_text())
        self.assertEqual(stored, receipt)
        self.assertIn("has-files=true\n", Path(self.env["GITHUB_OUTPUT"]).read_text())
        self.assertIn("required-files=true\n", Path(self.env["GITHUB_OUTPUT"]).read_text())

    def test_auxiliary_files_cannot_replace_required_report(self):
        """Fail when optional diagnostics survive but the required aggregate does not."""
        self.write(VOLUME_AUXILIARY, '{"passed":true}\n')
        with self.assertRaisesRegex(evidence.EvidenceError, "Required evidence path"):
            self.collect(
                paths=f"{VOLUME_REPORT}\n{VOLUME_AUXILIARY}",
                required_paths=VOLUME_REPORT,
                status="success",
            )
        receipt = json.loads((self.output / "_meta/receipt.json").read_text())
        self.assertEqual(
            [entry["path"] for entry in receipt["retained"]],
            [VOLUME_AUXILIARY],
        )
        self.assertEqual(receipt["unretained_required_paths"], [VOLUME_REPORT])
        outputs = Path(self.env["GITHUB_OUTPUT"]).read_text()
        self.assertIn("has-files=true\n", outputs)
        self.assertIn("required-files=false\n", outputs)

    def test_required_paths_reject_noncanonical_or_pattern_labels(self):
        """Required evidence names are literal canonical relative paths."""
        for label in (
            "../outside.json", "/absolute.json", "build/*.json",
            "./build/report.json", "build//report.json", "build/report.json ",
            "build/report\u200b.json",
        ):
            with self.subTest(label=label), self.assertRaises(evidence.EvidenceError):
                evidence._required_paths(label)

    def test_cli_required_path_failure_is_bounded_and_nonzero(self):
        """Expose required-retention failure through the actual collector CLI."""
        self.write(VOLUME_AUXILIARY, '{"passed":true}\n')
        result = subprocess.run(
            [
                sys.executable, "-B", str(SCRIPT), "collect",
                "--name", "native-evidence",
                "--paths", f"{VOLUME_REPORT}\n{VOLUME_AUXILIARY}",
                "--required-paths", VOLUME_REPORT,
                "--output", str(self.output),
                "--status", "success",
            ],
            cwd=self.root,
            env={**os.environ, **self.env},
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertIn("Required evidence path was not retained", result.stderr)
        self.assertNotIn("Traceback", result.stderr)
        receipt = json.loads((self.output / "_meta/receipt.json").read_text())
        self.assertEqual(receipt["unretained_required_paths"], [VOLUME_REPORT])
        outputs = Path(self.env["GITHUB_OUTPUT"]).read_text()
        self.assertIn("has-files=true\n", outputs)
        self.assertIn("required-files=false\n", outputs)

    def test_oversized_required_report_cannot_hide_behind_auxiliary_files(self):
        """Fail when the required aggregate exceeds the compact per-file limit."""
        raw = b'{"data":"' + b"a" * evidence.MAX_FILE_BYTES + b'"}\n'
        aggregate = self.write(VOLUME_REPORT, raw)
        self.write(VOLUME_AUXILIARY, '{"passed":true}\n')
        with self.assertRaisesRegex(evidence.EvidenceError, "Required evidence path"):
            self.collect(
                paths=f"{VOLUME_REPORT}\n{VOLUME_AUXILIARY}",
                required_paths=VOLUME_REPORT,
                status="success",
            )
        receipt = json.loads((self.output / "_meta/receipt.json").read_text())
        self.assertEqual(aggregate.read_bytes(), raw)
        self.assertEqual(receipt["unretained_required_paths"], [VOLUME_REPORT])
        omitted = next(item for item in receipt["omitted"]
                       if item["path"] == VOLUME_REPORT)
        self.assertEqual(omitted["reason"], "per-file-byte-limit")
        self.assertEqual(
            [entry["path"] for entry in receipt["retained"]],
            [VOLUME_AUXILIARY],
        )

    @unittest.skipUnless(shutil.which("git"), "git required for real checkout binding")
    def test_checkout_commit_is_independent_of_event_sha(self):
        """Bind provenance to actual HEAD even when the event SHA differs."""
        self.write("build/result.json", '{"passed":true}\n')
        subprocess.run(["git", "init", "-q", str(self.root)], check=True, capture_output=True)
        subprocess.run(["git", "-C", str(self.root), "add", "build/result.json"], check=True, capture_output=True)
        subprocess.run([
            "git", "-C", str(self.root), "-c", "user.name=Evidence Test",
            "-c", "user.email=evidence@example.invalid", "-c", "core.hooksPath=/dev/null",
            "commit", "-qm", "test source",
        ], check=True, capture_output=True)
        expected = subprocess.check_output(["git", "-C", str(self.root), "rev-parse", "HEAD"], text=True).strip()
        receipt = self.collect(status="success")
        self.assertEqual(receipt["source"]["checkout_sha"], expected)
        self.assertNotEqual(expected, self.env["GITHUB_SHA"])
        self.assertEqual(receipt["source"]["github_sha"], self.env["GITHUB_SHA"])
        self.assertEqual(receipt["source"]["run_id"], "12345")
        self.assertEqual(receipt["source"]["run_attempt"], "2")
        self.assertEqual(receipt["source"]["job"], "native-test")

    def test_missing_evidence_still_has_an_honest_receipt(self):
        """Mark missing inputs as metadata only and report no retained evidence."""
        receipt = self.collect("build/missing-*/result.json", status="success")
        self.assertEqual(receipt["evidence_state"], "no-evidence")
        self.assertEqual(receipt["missing_patterns"], ["build/missing-*/result.json"])
        self.assertEqual(receipt["retained"], [])
        self.assertIsNone(receipt["source"]["checkout_sha"])
        self.assertTrue((self.output / "_meta/receipt.json").is_file())
        self.assertIn("has-files=false", Path(self.env["GITHUB_OUTPUT"]).read_text())
        self.assertIn("not a passing test", self.console.getvalue())
        self.assertIn("metadata only", Path(self.env["GITHUB_STEP_SUMMARY"]).read_text())

    def test_explicit_patterns_preserve_paths_without_scooping_other_files(self):
        """Preserve requested paths while excluding reports outside the selection."""
        self.write("build/a/result.json", '{"value":1}')
        self.write("build/b/deep/serial.txt", "guest output")
        self.write("build/b/deep/skip.json", '{}')
        self.write("unselected/result.json", '{}')
        receipt = self.collect("build/*/result.json\nbuild/b/**/serial.txt\nbuild/absent.json")
        self.assertEqual({item["path"] for item in receipt["retained"]}, {
            "build/a/result.json", "build/b/deep/serial.txt",
        })
        self.assertEqual(receipt["missing_patterns"], ["build/absent.json"])
        self.assertFalse((self.output / "unselected").exists())

    def test_binary_hidden_secrets_symlinks_and_escape_are_excluded(self):
        """Exclude unsafe paths and non-report content without logging their payloads."""
        self.write("build/ok.json", '{"passed":false}')
        self.write("build/binary.json", b"abc\0def")
        self.write("build/nonutf8.log", b"\xff\xfe")
        self.write("build/image.img", b"huge image placeholder")
        self.write("build/.private/report.json", '{"private":true}')
        self.write("build/credentials.json", '{"private":true}')
        self.write("build/nested/token.txt", "sensitive text")
        self.write("build/config.json", '{"api_key":"not-for-any-upload"}')
        self.write("build/pem.log", "-----BEGIN PRIVATE KEY-----\nsecret bytes")
        outside = self.base / "outside"
        outside.mkdir()
        (outside / "report.json").write_text('{"outside":true}')
        (self.root / "build/file-link.json").symlink_to(outside / "report.json")
        (self.root / "linked-directory").symlink_to(outside, target_is_directory=True)
        receipt = self.collect(f"build/\nlinked-directory/*.json\n../outside/report.json\n{outside}/report.json")
        self.assertEqual([item["path"] for item in receipt["retained"]], ["build/ok.json"])
        reasons = set(receipt["omitted_reason_counts"])
        self.assertTrue({"binary-control-bytes", "not-utf8-text", "not-compact-report-type", "hidden-path", "sensitive-name", "symlink", "credential-like-content"} <= reasons)
        self.assertNotIn("not-for-any-upload", self.console.getvalue())
        self.assertNotIn("secret bytes", self.console.getvalue())
        self.assertNotIn('"outside":true', self.console.getvalue())
        self.assertFalse((self.output / "linked-directory").exists())

    def test_large_binary_is_never_opened_or_hashed(self):
        """Skip raw binary inputs before opening or hashing their contents."""
        binary = self.write("build/volume.img", b"")
        with binary.open("r+b") as handle:
            handle.truncate(64 * 1024 * 1024)
        with patch.object(evidence, "_read_report", side_effect=AssertionError("binary opened")):
            receipt = self.collect()
        self.assertEqual(receipt["retained"], [])
        self.assertEqual(receipt["omitted"][0]["size_bytes"], 64 * 1024 * 1024)
        self.assertNotIn("sha256", receipt["omitted"][0])

    def test_oversized_json_is_omitted_never_silently_truncated(self):
        """Omit an oversized JSON report explicitly and leave its source bytes intact."""
        raw = b'{"data":"' + b"a" * evidence.MAX_FILE_BYTES + b'"}\n'
        original = self.write("build/large.json", raw)
        receipt = self.collect()
        self.assertFalse((self.output / "build/large.json").exists())
        self.assertEqual(original.read_bytes(), raw)
        self.assertEqual(receipt["omitted"][0]["reason"], "per-file-byte-limit")
        self.assertEqual(receipt["retained"], [])

    def test_total_byte_budget_includes_receipt(self):
        """Keep report bytes and the receipt within the total storage budget."""
        for index in range(9):
            self.write(f"build/log-{index}.txt", b"x" * evidence.MAX_FILE_BYTES)
        receipt = self.collect()
        files = [path for path in self.output.rglob("*") if path.is_file()]
        self.assertLessEqual(sum(path.stat().st_size for path in files), evidence.MAX_TOTAL_BYTES)
        self.assertTrue(all(path.stat().st_size <= evidence.MAX_FILE_BYTES for path in files))
        self.assertIn("total-byte-limit", receipt["omitted_reason_counts"])
        self.assertEqual(sum(item["size_bytes"] for item in receipt["retained"]), receipt["retained_bytes"])

    def test_file_count_budget_includes_receipt(self):
        """Reserve a file slot for the receipt and count omitted excess reports."""
        for index in range(503):
            self.write(f"build/result-{index:04}.json", "{}\n")
        receipt = self.collect()
        files = [path for path in self.output.rglob("*") if path.is_file()]
        self.assertLessEqual(len(files), evidence.MAX_FILES)
        self.assertEqual(len(receipt["retained"]), 499)
        self.assertEqual(receipt["omitted_reason_counts"]["file-count-limit"], 4)

    def test_console_commands_cr_and_ansi_are_inert(self):
        """Neutralize runner commands and terminal controls without altering stored bytes."""
        attacks = "::error::injected\n##[error]old syntax\r::warning::cr\n##[add-mask]mask\n\x1b[31mred\n"
        self.write("build/00-attacks.log", attacks)
        self.collect()
        console = self.console.getvalue()
        self.assertIn(": :error: :injected", console)
        self.assertIn("# #[error]old syntax", console)
        self.assertIn("# #[add-mask]mask", console)
        self.assertIn("\\u000d: :warning: :cr", console)
        self.assertIn("\\u001b[31mred", console)
        self.assertNotIn("::error::", console)
        self.assertNotIn("##[error]", console)
        self.assertNotIn("\r", console)
        self.assertNotIn("\x1b", console)
        self.assertTrue(all(line.startswith("[ci-evidence] | ") for line in console.splitlines()))
        self.assertEqual((self.output / "build/00-attacks.log").read_bytes(), attacks.encode())

    def test_console_preview_budget_and_truncation_are_explicit(self):
        """Bound console output and label truncated JSON while retaining exact files."""
        for index in range(24):
            self.write(f"build/large-{index:02}.json", '{"data":"' + "x" * 20_000 + '"}\n')
        receipt = self.collect()
        console = self.console.getvalue()
        self.assertLessEqual(len(console.encode("utf-8")), evidence.MAX_CONSOLE_BYTES)
        self.assertIn("CONSOLE LIMIT", console)
        self.assertIn("TRUNCATED JSON PREVIEW", console)
        self.assertTrue(all(line.startswith("[ci-evidence] | ") for line in console.splitlines()))
        self.assertEqual(len(receipt["retained"]), 24)
        self.assertEqual(json.loads((self.output / "build/large-00.json").read_text()), {"data": "x" * 20_000})

    def test_long_log_tail_is_labeled_without_changing_stored_bytes(self):
        """Label a long log's tail excerpt while retaining the complete compact copy."""
        content = "BEGIN\n" + "x" * 20_000 + "\nEND\n"
        self.write("build/serial.txt", content)
        self.collect()
        self.assertIn("LOG/TEXT TAIL", self.console.getvalue())
        self.assertIn("display truncated", self.console.getvalue())
        self.assertNotIn("| BEGIN", self.console.getvalue())
        self.assertIn("| END", self.console.getvalue())
        self.assertEqual((self.output / "build/serial.txt").read_bytes(), content.encode())

    def test_safe_read_refuses_symlink_substituted_after_selection(self):
        """Refuse a report replaced by a symlink between selection and reading."""
        self.write("build/report.json", '{}')
        outside = self.base / "outside.json"
        outside.write_text('{"outside":true}')
        (self.root / "build/report.json").unlink()
        (self.root / "build/report.json").symlink_to(outside)
        with self.assertRaises(OSError):
            evidence._read_report(self.root, "build/report.json")

    def test_collection_errors_are_not_ignored_or_stale_files_reused(self):
        """Reject stale output directories and propagate report read failures."""
        self.write("build/result.json", '{}')
        self.output.mkdir()
        (self.output / "old.txt").write_text("old evidence")
        with self.assertRaises(evidence.EvidenceError):
            self.collect()
        self.assertEqual((self.output / "old.txt").read_text(), "old evidence")
        other = self.base / "fresh"
        with patch.object(evidence, "_read_report", side_effect=OSError("permission denied")):
            with self.assertRaises(OSError):
                self.collect(output=other)

    def test_cli_outputs_and_errors(self):
        """Expose CLI output paths and a nonzero exit when output reuse is refused."""
        self.write("build/report.json", '{"passed":false}\n')
        command = [
            sys.executable, "-B", str(SCRIPT), "collect", "--name", "cli-evidence",
            "--paths", "build/", "--output", str(self.output), "--status", "failure",
        ]
        env = dict(os.environ, **self.env)
        result = subprocess.run(command, cwd=self.root, env=env, capture_output=True, text=True, check=False)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(f"path={self.output}\n", Path(self.env["GITHUB_OUTPUT"]).read_text())
        retry = subprocess.run(command, cwd=self.root, env=env, capture_output=True, text=True, check=False)
        self.assertEqual(retry.returncode, 2)
        self.assertIn("stale evidence is refused", retry.stderr)


if __name__ == "__main__":
    unittest.main()

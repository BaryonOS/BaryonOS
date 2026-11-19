"""Synthetic reference checks; these fixtures are not native test evidence."""
import copy
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import lp_matrix


class MatrixTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / lp_matrix.CONTRACT).parent.mkdir(parents=True)
        (self.root / lp_matrix.CONTRACT).write_text("\n".join(f"| LP-{i:02} | case | required |" for i in range(1, 32)))
        self.matrix = {"schema": 1, "rows": [dict(id=f"LP-{i:02}", status="none", evidence=None,
                       remaining="Coverage remains open") for i in range(1, 32)]}
        (self.root / lp_matrix.MATRIX).parent.mkdir(parents=True)

    def run_check(self, data=None):
        (self.root / lp_matrix.MATRIX).write_text(json.dumps(self.matrix if data is None else data))
        return lp_matrix.check(self.root)

    def reference(self, name, content):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)
        return dict(path=name, sha256=hashlib.sha256(path.read_bytes()).hexdigest())

    def test_exact_ids_and_residual_list(self):
        result = self.run_check()
        self.assertEqual(result["counts"], dict(native=0, host=0, none=31))
        self.assertEqual(len(result["residual_coverage"]), 31)
        for mutate in (lambda d: d["rows"].pop(), lambda d: d["rows"].append(d["rows"][0]),
                       lambda d: d["rows"][0].update(id="LP-32"),
                       lambda d: d["rows"][0].update(remaining=""),
                       lambda d: d["rows"][0].update(status="accepted")):
            data = copy.deepcopy(self.matrix)
            mutate(data)
            with self.assertRaises(ValueError):
                self.run_check(data)

    def test_contract_duplicate_is_rejected(self):
        path = self.root / lp_matrix.CONTRACT
        path.write_text(path.read_text().replace("LP-31", "LP-30"))
        with self.assertRaises(ValueError):
            self.run_check()

    def test_native_requires_source_bound_passing_record(self):
        report = dict(passed=True, timed_out=False, raw_qemu_exit=33, source_sha256="a" * 64)
        row = self.matrix["rows"][0]
        row.update(status="native", evidence=self.reference("docs/verification/example/result.json", json.dumps(report)))
        self.assertEqual(self.run_check()["counts"]["native"], 1)
        for change in (dict(passed=False), dict(passed=1), dict(timed_out=True), dict(raw_qemu_exit=35),
                       dict(raw_qemu_exit=33.0), dict(source_sha256="unknown")):
            row["evidence"] = self.reference("docs/verification/example/result.json", json.dumps(report | change))
            with self.assertRaises(ValueError):
                self.run_check()
        for name, text in (("docs/verification/example/README.md", "Native PASS"),
                           ("build/result.json", json.dumps(report))):
            row["evidence"] = self.reference(name, text)
            with self.assertRaises(ValueError):
                self.run_check()

    def test_reference_path_digest_and_none_claim(self):
        reference = self.reference("docs/host.txt", "host result")
        row = self.matrix["rows"][0]
        row.update(status="host", evidence=reference)
        self.assertEqual(self.run_check()["counts"]["host"], 1)
        for name in ("../outside", "/etc/passwd", "docs/../docs/host.txt", "docs//host.txt", "docs/missing", "C:\\x"):
            row["evidence"] = reference | dict(path=name)
            with self.assertRaises((ValueError, OSError)):
                self.run_check()
        row["evidence"] = reference | dict(sha256="0" * 64)
        with self.assertRaises(ValueError):
            self.run_check()
        row.update(status="none", evidence=reference)
        with self.assertRaises(ValueError):
            self.run_check()
        with tempfile.TemporaryDirectory() as outside:
            target = Path(outside) / "evidence"
            target.write_text("outside")
            (self.root / "docs/link").symlink_to(target)
            row.update(status="host", evidence=reference | dict(path="docs/link"))
            with self.assertRaises(ValueError):
                self.run_check()


if __name__ == "__main__":
    unittest.main()

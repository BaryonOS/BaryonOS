"""Graphify checkpoint inventory: invariants are read-only, provenance and no overwrite."""
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from graphify_resume import inspect, main


class GraphifyResumeTests(unittest.TestCase):
    def test_partial_chunk_inventory_preserves_source_bytes(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            source = root / ".graphify_chunk_01.json"
            data = json.dumps({"nodes": [{"id": "n"}], "edges": [], "hyperedges": []})
            source.write_text(data, encoding="utf-8")
            result = inspect(root, expected_text=3, expected_images=0)
            self.assertEqual(result["text_chunks_found"], 1)
            self.assertEqual(result["missing_text_indices"], [2, 3])
            self.assertEqual(result["chunk_records"][0]["item_counts"]["nodes"], 1)
            self.assertTrue(result["json_valid"])
            self.assertFalse(result["complete"])
            self.assertEqual(source.read_text(encoding="utf-8"), data)

    def test_malformed_json_is_not_silently_accepted(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / ".graphify_chunk_01.json").write_text('{"nodes":', encoding="utf-8")
            result = inspect(root, expected_text=1, expected_images=0)
            self.assertFalse(result["json_valid"])
            self.assertEqual(main(["--output-dir", d, "--expected-text", "1", "--expected-images", "0"]), 1)

    def test_same_numeric_index_with_different_name_is_flagged(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            for name in (".graphify_chunk_1.json", ".graphify_chunk_01.json"):
                (root / name).write_text('{"nodes":[]}', encoding="utf-8")
            result = inspect(root, expected_text=1, expected_images=0)
            self.assertEqual(sorted(result["duplicate_text_indices"]["1"]),
                             [".graphify_chunk_01.json", ".graphify_chunk_1.json"])

    def test_manifest_creation_refuses_overwrite(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / ".graphify_chunk_01.json").write_text('{"nodes":[]}', encoding="utf-8")
            out = root / "resume-manifest.json"
            args = ["--output-dir", d, "--expected-text", "1", "--expected-images", "0",
                    "--write-manifest", str(out)]
            self.assertEqual(main(args), 0)
            old = out.read_bytes()
            self.assertEqual(main(args), 2)
            self.assertEqual(out.read_bytes(), old)

    def test_unknown_ast_does_not_fake_completed_graph(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / ".graphify_chunk_01.json").write_text('{"nodes":[]}', encoding="utf-8")
            for name in ("graph.json", "graph.html", "GRAPH_REPORT.md"):
                (root / name).write_text("old", encoding="utf-8")
            r = inspect(root, expected_text=1, expected_images=0)
            self.assertTrue(all(r["final_artifacts_present"].values()))
            self.assertFalse(r["complete"])


if __name__ == "__main__":
    unittest.main()

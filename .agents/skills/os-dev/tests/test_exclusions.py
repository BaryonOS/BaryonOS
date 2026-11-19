import importlib.util, json, shutil, tempfile, unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("build_corpus", ROOT / "scripts/build_corpus.py")
bc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bc)


def page(pid, title):
    return {"page_id": pid, "title": title, "namespace": 0, "namespace_name": "Main",
            "revision_id": 1, "timestamp": "2020-01-01T00:00:00Z", "redirect": None,
            "body": f"text of {title}"}


class Exclusions(unittest.TestCase):
    def test_excluded_records_are_absent_from_the_active_index(self):
        excluded = {e["source_id"] for e in json.loads((ROOT / "references/exclusions.json").read_text())}
        rows = json.loads((ROOT / "references/source-index.json").read_text())
        self.assertTrue(excluded)
        self.assertFalse(excluded & {r["source_id"] for r in rows})

    @unittest.skipUnless(shutil.which("pdftotext") and shutil.which("pdfinfo"), "poppler missing")
    def test_rebuild_keeps_excluded_records_out(self):
        excluded = sorted(e["source_id"] for e in json.loads((ROOT / "references/exclusions.json").read_text()))
        pid = int(excluded[0][1:])
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "references").mkdir()
            shutil.copy(ROOT / "references/exclusions.json", root / "references/exclusions.json")
            with self.assertRaises(SystemExit):  # synthetic input is below the 3,000-page floor
                bc.build(root, [page(pid, "Excluded"), page(pid + 1, "Kept")],
                         ROOT / "assets/book.pdf", "test")
            text = "".join(p.read_text() for p in (root / "references/corpus").glob("wiki-*.md"))
            self.assertNotIn(f'<a id="{excluded[0].lower()}">', text)
            self.assertIn(f'<a id="w{pid + 1:06d}">', text)


if __name__ == "__main__":
    unittest.main()

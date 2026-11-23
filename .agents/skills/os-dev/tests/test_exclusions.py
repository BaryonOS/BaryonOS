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
        excluded = {e["source_id"] for e in json.loads((ROOT / "references/exclusions.json").read_text(encoding="utf-8"))}
        rows = json.loads((ROOT / "references/source-index.json").read_text(encoding="utf-8"))
        self.assertTrue(excluded)
        self.assertFalse(excluded & {r["source_id"] for r in rows})

    @unittest.skipUnless(shutil.which("pdftotext") and shutil.which("pdfinfo"), "poppler missing")
    def test_rebuild_keeps_excluded_records_out(self):
        excluded = sorted(e["source_id"] for e in json.loads((ROOT / "references/exclusions.json").read_text(encoding="utf-8")))
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


class ExcludeScript(unittest.TestCase):
    def _copy(self, tmp):
        dst = Path(tmp) / "os-dev"
        shutil.copytree(ROOT, dst, ignore=shutil.ignore_patterns("__pycache__", "book.pdf"))
        return dst

    @staticmethod
    def _snapshot(root):
        # Every data file the script may write: references/ and assets/.
        return {p: p.read_bytes() for d in ("references", "assets")
                for p in (root / d).rglob("*") if p.is_file()}

    def _module(self, root):
        spec = importlib.util.spec_from_file_location("exclude_sources_copy", root / "scripts/exclude_sources.py")
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)
        return mod

    def _target(self, root):
        rows = json.loads((root / "references/source-index.json").read_text(encoding="utf-8"))
        return next(r for r in rows if r["kind"] == "wiki" and r["local_path"].endswith("wiki-02.md"))

    def test_exclusion_keeps_remaining_hashes_valid(self):
        import hashlib, sys
        with tempfile.TemporaryDirectory() as tmp:
            root = self._copy(tmp)
            target = self._target(root)
            mod = self._module(root)
            sys.argv = ["x", "--reason", "test", "--record", "none", target["source_id"]]
            self.assertEqual(mod.main(), 0)
            rows = json.loads((root / "references/source-index.json").read_text(encoding="utf-8"))
            self.assertNotIn(target["source_id"], {r["source_id"] for r in rows})
            for r in rows:
                if r["local_path"].endswith("wiki-02.md"):
                    data = (root / r["local_path"]).read_bytes()
                    block = data[r["start_byte"]:r["start_byte"] + r["length_bytes"]]
                    self.assertEqual(hashlib.sha256(block).hexdigest(), r["text_sha256"])

    def test_failed_write_restores_every_file(self):
        import sys
        with tempfile.TemporaryDirectory() as tmp:
            root = self._copy(tmp)
            target = self._target(root)
            before = self._snapshot(root)
            mod = self._module(root)
            real = Path.write_bytes

            def failing(self, data):
                if self.name == "source-manifest.json.exclude-tmp":
                    real(self, b"partial")
                    raise OSError("simulated partial write failure")
                return real(self, data)

            sys.argv = ["x", "--reason", "test", "--record", "none", target["source_id"]]
            Path.write_bytes = failing
            try:
                with self.assertRaises(OSError):
                    mod.main()
            finally:
                Path.write_bytes = real
            after = self._snapshot(root)
            self.assertEqual(before, after)

    def test_rejected_batch_leaves_every_file_unchanged(self):
        import sys
        with tempfile.TemporaryDirectory() as tmp:
            root = self._copy(tmp)
            index = root / "references/source-index.json"
            rows = json.loads(index.read_text(encoding="utf-8"))
            first = next(r for r in rows if r["kind"] == "wiki" and r["local_path"].endswith("wiki-02.md"))
            second = next(r for r in rows if r["kind"] == "wiki" and r["local_path"].endswith("wiki-04.md"))
            second["text_sha256"] = "0" * 64
            index.write_text(json.dumps(rows, separators=(",", ":"), ensure_ascii=False) + "\n", encoding="utf-8")
            before = self._snapshot(root)
            mod = self._module(root)
            sys.argv = ["x", "--reason", "test", "--record", "none", first["source_id"], second["source_id"]]
            with self.assertRaises(SystemExit):
                mod.main()
            after = self._snapshot(root)
            self.assertEqual(before, after)

    def test_rollback_continues_after_a_failed_restoration(self):
        import sys
        with tempfile.TemporaryDirectory() as tmp:
            root = self._copy(tmp)
            target = self._target(root)
            corpus = root / target["local_path"]
            original = corpus.read_bytes()
            index = root / "references/source-index.json"
            page = root / "references/pagina-index.md"
            index_before, page_before = index.read_bytes(), page.read_bytes()
            mod = self._module(root)
            real = Path.write_bytes
            calls = {}

            def failing(self, data):
                calls[self.name] = calls.get(self.name, 0) + 1
                if self.name == "source-manifest.json.exclude-tmp":
                    raise OSError("simulated write failure")
                if self.name == "source-index.json.exclude-tmp" and calls[self.name] > 1:
                    raise OSError("simulated restore failure")
                return real(self, data)

            sys.argv = ["x", "--reason", "test", "--record", "none", target["source_id"]]
            Path.write_bytes = failing
            try:
                with self.assertRaisesRegex(OSError, "source-index.json") as ctx:
                    mod.main()
            finally:
                Path.write_bytes = real
            self.assertIsInstance(ctx.exception.__cause__, OSError)
            self.assertEqual(corpus.read_bytes(), original)
            # Files written after the unrestorable one are still restored.
            self.assertEqual(page.read_bytes(), page_before)
            # The unrestored file's original is kept and named in the error.
            kept = index.with_name("source-index.json.exclude-orig")
            self.assertEqual(kept.read_bytes(), index_before)
            self.assertIn(str(kept), str(ctx.exception))
            self.assertEqual(list(root.rglob("*.exclude-tmp")), [])

    def test_unwritable_backup_aborts_before_any_change(self):
        import sys
        with tempfile.TemporaryDirectory() as tmp:
            root = self._copy(tmp)
            target = self._target(root)
            before = self._snapshot(root)
            mod = self._module(root)
            real = Path.write_bytes

            def failing(self, data):
                if self.name == "source-manifest.json.exclude-orig":
                    raise OSError("simulated full disk")
                return real(self, data)

            sys.argv = ["x", "--reason", "test", "--record", "none", target["source_id"]]
            Path.write_bytes = failing
            try:
                with self.assertRaises(OSError):
                    mod.main()
            finally:
                Path.write_bytes = real
            self.assertEqual(self._snapshot(root), before)
            self.assertEqual(list(root.rglob("*.exclude-orig")), [])

    def test_successful_exclusion_leaves_no_backups(self):
        import sys
        with tempfile.TemporaryDirectory() as tmp:
            root = self._copy(tmp)
            target = self._target(root)
            mod = self._module(root)
            sys.argv = ["x", "--reason", "test", "--record", "none", target["source_id"]]
            self.assertEqual(mod.main(), 0)
            self.assertEqual(list(root.rglob("*.exclude-orig")), [])

    def test_partial_backup_is_removed(self):
        import sys
        with tempfile.TemporaryDirectory() as tmp:
            root = self._copy(tmp)
            target = self._target(root)
            before = self._snapshot(root)
            mod = self._module(root)
            real = Path.write_bytes

            def failing(self, data):
                if self.name == "source-manifest.json.exclude-orig":
                    real(self, b"partial")
                    raise OSError("simulated full disk")
                return real(self, data)

            sys.argv = ["x", "--reason", "test", "--record", "none", target["source_id"]]
            Path.write_bytes = failing
            try:
                with self.assertRaises(OSError):
                    mod.main()
            finally:
                Path.write_bytes = real
            self.assertEqual(self._snapshot(root), before)
            self.assertEqual(list(root.rglob("*.exclude-orig")), [])

    def test_cleanup_failure_keeps_rollback_report(self):
        import sys
        with tempfile.TemporaryDirectory() as tmp:
            root = self._copy(tmp)
            target = self._target(root)
            corpus = root / target["local_path"]
            original = corpus.read_bytes()
            mod = self._module(root)
            real_write, real_unlink = Path.write_bytes, Path.unlink

            def failing_write(self, data):
                if self.name == "source-manifest.json.exclude-tmp":
                    raise OSError("simulated write failure")
                return real_write(self, data)

            def failing_unlink(self, missing_ok=False):
                if self.name.endswith(".exclude-orig"):
                    raise OSError("simulated read-only filesystem")
                return real_unlink(self, missing_ok=missing_ok)

            sys.argv = ["x", "--reason", "test", "--record", "none", target["source_id"]]
            Path.write_bytes, Path.unlink = failing_write, failing_unlink
            try:
                with self.assertRaisesRegex(OSError, "recovery copies left behind") as ctx:
                    mod.main()
            finally:
                Path.write_bytes, Path.unlink = real_write, real_unlink
            self.assertEqual(str(ctx.exception.__cause__), "simulated write failure")
            self.assertEqual(corpus.read_bytes(), original)

    def test_existing_recovery_copy_blocks_a_new_run(self):
        import sys
        with tempfile.TemporaryDirectory() as tmp:
            root = self._copy(tmp)
            target = self._target(root)
            kept = root / "references/source-index.json.exclude-orig"
            kept.write_bytes(b"original from an earlier failed run")
            before = self._snapshot(root)
            mod = self._module(root)
            sys.argv = ["x", "--reason", "test", "--record", "none", target["source_id"]]
            with self.assertRaisesRegex(OSError, "earlier failed run"):
                mod.main()
            self.assertEqual(self._snapshot(root), before)

    def test_rebuilt_manifest_without_optional_totals(self):
        import sys
        with tempfile.TemporaryDirectory() as tmp:
            root = self._copy(tmp)
            target = self._target(root)
            mf = root / "assets/source-manifest.json"
            m = json.loads(mf.read_text(encoding="utf-8"))
            for key in ("source_text_bytes", "wiki_main_namespace_pages",
                        "wiki_main_articles_excluding_redirects"):
                m.pop(key, None)
            before = m["source_records"]
            mf.write_text(json.dumps(m, indent=2) + "\n", encoding="utf-8")
            mod = self._module(root)
            sys.argv = ["x", "--reason", "test", "--record", "none", target["source_id"]]
            self.assertEqual(mod.main(), 0)
            after = json.loads(mf.read_text(encoding="utf-8"))
            self.assertEqual(after["source_records"], before - 1)
            self.assertNotIn("source_text_bytes", after)

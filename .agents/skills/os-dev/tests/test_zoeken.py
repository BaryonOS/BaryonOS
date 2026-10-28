import importlib.util, json, unittest
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location("z",ROOT/"scripts/zoeken.py")
z=importlib.util.module_from_spec(spec)
spec.loader.exec_module(z)

class T(unittest.TestCase):
    def test_count(self):
        manifest=json.loads((ROOT/"assets/source-manifest.json").read_text())
        self.assertEqual(len(z.rows()),manifest["source_records"])
        self.assertGreaterEqual(manifest["wiki_pages"],3000)
        self.assertEqual(manifest["pdf_pages"],78)
    def test_page_frame(self):
        matches=z.search("page frame allocation",title="Page Frame Allocation")
        self.assertTrue(any(r["source_id"]=="W001461" for r in matches))
    def test_uefi(self):
        self.assertEqual(z.search("UEFI")[0]["title"],"UEFI")
    def test_microkernel(self):
        self.assertEqual(z.search("microkernel")[0]["title"],"Microkernel")
    def test_pdf(self):
        self.assertTrue(all(r["kind"]=="pdf" for r in z.search("interrupt","pdf")))
    def test_book_page(self):
        r=next(r for r in z.rows() if r["source_id"]=="B053")
        self.assertIn("Paging",z.body(r))

if __name__=="__main__":
    unittest.main()

"""Evidence-boundary negatives; these controlled files are not native proof."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import window_library as library
from common import ROOT, sha256


def decoded():
    versions = [{"revision": revision, "name": library.TITLE,
                 "content_hex": content.hex()} for revision, content in
                ((4, library.ORIGINAL), (3, library.EDITED))]
    return {"objects": [{"object": 0, "present": True, "collection": 1,
                         "tags": 0, "kind": 0, "revision": 4, "versions": versions},
                        {"object": 1, "present": False, "revision": 0}],
            "collections": [{"domain": 0, "index": 1, "name": library.COLLECTION,
                             "present": True, "revision": 2}]}


class LibraryEvidenceTests(unittest.TestCase):
    def test_restore_requires_new_revision_actual_bytes_name_and_collection(self):
        valid = decoded()
        with patch.object(library.volume_disk, "analyse", return_value=valid):
            self.assertEqual(library.disk_oracle(b"controlled parser input"), valid)
        variants = []
        for path, value in (("collection", 0), ("tags", 1), ("kind", 1)):
            bad = copy.deepcopy(valid)
            bad["objects"][0][path] = value
            variants.append(bad)
        for index, field, value in ((0, "content_hex", library.EDITED.hex()),
                                     (1, "content_hex", library.ORIGINAL.hex()),
                                     (0, "revision", 3), (0, "name", "Ander materiaal")):
            bad = copy.deepcopy(valid)
            bad["objects"][0]["versions"][index][field] = value
            variants.append(bad)
        for change in ("other-domain", "extra-document", "wrong-collection"):
            bad = copy.deepcopy(valid)
            if change == "other-domain":
                bad["objects"][1]["revision"] = 1
            elif change == "extra-document":
                bad["objects"][1]["present"] = True
            else:
                bad["collections"][0]["domain"] = 1
            variants.append(bad)
        for bad in variants:
            with self.subTest(bad=bad), patch.object(library.volume_disk, "analyse", return_value=bad):
                with self.assertRaises(ValueError):
                    library.disk_oracle(b"controlled parser input")

    def test_cold_parent_rejects_stale_source_unfinished_flags_and_changed_files(self):
        parent = ROOT / "build" / "library-evidence-host"
        parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=parent) as name:
            directory = Path(name)
            volume = directory / "volume.img"
            volume.write_bytes(bytes(library.volume_disk.IMAGE_SECTORS * library.volume_disk.SECTOR))
            serial = directory / "serial.txt"
            serial.write_bytes(b"WINDOW:CLEANUP:PASS\nWINDOW:CASE:PASS\nM1:PASS\n")
            manifest = {"kernel": {"sha256": "a" * 64}}
            report = dict(passed=True, library_gate=True, library_requested=True,
                          library_cold=False, native_gate=False, raw_qemu_exit=33,
                          kernel_feature="native-window-session-probe", source_fingerprint_version=3,
                          source_sha256="b" * 64, manifest=manifest, run_directory=str(directory),
                          volume_sha256=sha256(volume), serial_sha256=sha256(serial),
                          disk_oracle=decoded())
            report_path = directory / "result.json"

            def check(value):
                report_path.write_text(json.dumps(value))
                return library.cold_parent(directory, source="b" * 64, manifest=manifest)

            with patch.object(library, "disk_oracle", return_value=decoded()):
                self.assertEqual(check(report)["image"], volume.read_bytes())
                for key, value in (("passed", False), ("library_gate", False),
                                   ("library_cold", True), ("raw_qemu_exit", 0),
                                   ("source_sha256", "c" * 64), ("passed", 1),
                                   ("harness_terminated_process", True),
                                   ("error", "TimeoutError"), ("volume_sha256", "d" * 64),
                                   ("serial_sha256", "e" * 64), ("disk_oracle", {})):
                    with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                        check({**report, key: value})
                serial.write_bytes(b"WINDOW:FAIL:bad\nWINDOW:CLEANUP:PASS\nWINDOW:CASE:PASS\nM1:PASS\n")
                with self.assertRaises(ValueError):
                    check({**report, "serial_sha256": sha256(serial)})


if __name__ == "__main__":
    unittest.main()

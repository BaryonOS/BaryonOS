"""Negative input and actual process-image checks; no native OS execution."""
import contextlib
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import types
import unittest

BASE = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(BASE))
sys.path.insert(0, str(BASE / "source/tools"))
import windows_inputs as inputs
import run_window_session as native


class FixtureTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="nieuw-os-input-test-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.directory = self.root / "fixture"
        self.directory.mkdir()
        self.frozen = "a" * 64
        self.guard_calls = []

        def private_path(path, *, regular=False):
            # A runner boundary spy, without altering any imported module.
            self.guard_calls.append((path, regular))
            if not path.is_relative_to(self.root):
                raise ValueError("runner rejected escaped fixture")
            return path

        self.runner = types.SimpleNamespace(ROOT=self.root, private_path=private_path,
            load_json=native.load_json, sha256=native.sha256, ppm=native.ppm,
            FIXTURE_PHASES=native.FIXTURE_PHASES)
        hashes = {}
        for width, height in ((880, 510), (1120, 660)):
            raw = f"P6\n{width} {height}\n255\n".encode() + b"\x01\x02\x03" * (width * height)
            for phase in native.FIXTURE_PHASES:
                name = f"{width}x{height}-{phase}.ppm"
                (self.directory / name).write_bytes(raw)
                hashes[name] = hashlib.sha256(raw).hexdigest()
        self.metadata = dict(kind=inputs.FIXTURE_KIND, source_sha256=self.frozen,
            source_fingerprint_version=3, generator_exit=0,
            generator_command=["cargo", "+1.90.0", "test", "--offline", "--locked", "--manifest-path",
                "/workspace/frozen/ui/native/Cargo.toml", "--test", "window_native_fixture", "--", "--nocapture"],
            files_sha256=hashes)
        self.save_metadata()

    def save_metadata(self):
        (self.directory / "fixture.json").write_text(json.dumps(self.metadata), encoding="utf-8")

    def load(self):
        return inputs.load_fixture(self.runner, self.directory, self.frozen)

    def test_valid_fixture_returns_complete_bound_frames_and_calls_runner_guard(self):
        result = self.load()
        self.assertEqual(set(result), {"directory", "metadata", "frames"})
        self.assertEqual(set(result["frames"]), set(native.FIXTURE_PHASES))
        self.assertEqual(result["metadata"], self.metadata)
        self.assertGreaterEqual(len(self.guard_calls), 20)

    def test_wrong_source_schema_types_and_generator_fail_closed(self):
        variants = [("source_sha256", "b" * 64), ("kind", "native pass"),
            ("source_fingerprint_version", True), ("source_fingerprint_version", 2),
            ("generator_exit", False), ("generator_exit", 1),
            ("generator_command", ["cargo", "test"]), ("unexpected", 1)]
        original = dict(self.metadata)
        for key, value in variants:
            with self.subTest(key=key, value=value):
                self.metadata = dict(original, **{key: value})
                self.save_metadata()
                with self.assertRaises(ValueError): self.load()

    def test_duplicate_metadata_key_is_rejected_by_original_json_loader(self):
        raw = json.dumps(self.metadata)
        (self.directory / "fixture.json").write_text('{"generator_exit":0,' + raw[1:], encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "Duplicate"): self.load()

    def test_runner_private_path_rejection_is_not_bypassed(self):
        def reject(*args, **kwargs): raise ValueError("original guard rejected")
        runner = types.SimpleNamespace(**vars(self.runner))
        runner.private_path = reject
        with self.assertRaisesRegex(ValueError, "original guard rejected"):
            inputs.load_fixture(runner, self.directory, self.frozen)

    def test_missing_extra_nested_and_inventory_images_are_rejected(self):
        path = self.directory / "extra.ppm"
        path.write_bytes(b"extra")
        with self.assertRaises(ValueError): self.load()
        path.unlink()
        folder = self.directory / "nested"
        folder.mkdir()
        with self.assertRaises(ValueError): self.load()
        folder.rmdir()
        name = next(iter(self.metadata["files_sha256"]))
        del self.metadata["files_sha256"][name]
        self.save_metadata()
        with self.assertRaises(ValueError): self.load()
        (self.directory / name).unlink()
        with self.assertRaises(ValueError): self.load()

    def test_tampered_pixel_hash_is_rejected(self):
        name = next(iter(self.metadata["files_sha256"]))
        path = self.directory / name
        raw = bytearray(path.read_bytes()); raw[-1] ^= 1; path.write_bytes(raw)
        with self.assertRaisesRegex(ValueError, "hash mismatch"): self.load()

    def test_wrong_geometry_rejected_even_with_matching_hash(self):
        name = next(iter(self.metadata["files_sha256"]))
        path = self.directory / name
        raw = b"P6\n1 1\n255\n\x01\x02\x03"
        path.write_bytes(raw)
        self.metadata["files_sha256"][name] = hashlib.sha256(raw).hexdigest()
        self.save_metadata()
        with self.assertRaisesRegex(ValueError, "dimensions"): self.load()

    def test_oversized_image_rejected_before_decoder(self):
        name = next(iter(self.metadata["files_sha256"]))
        with (self.directory / name).open("wb") as stream:
            stream.truncate(inputs.MAX_PPM_BYTES + 1)
        with self.assertRaisesRegex(ValueError, "Oversized"): self.load()

    def test_fixture_escape_and_hardlink_are_rejected(self):
        outside = self.root.parent / "outside-fixture"
        with self.assertRaises(ValueError): inputs.load_fixture(self.runner, outside, self.frozen)
        source = self.directory / next(iter(self.metadata["files_sha256"]))
        linked = self.root / "hardlink.ppm"
        os.link(source, linked)
        with self.assertRaisesRegex(ValueError, "single-link"): self.load()


@unittest.skipUnless(os.name == "nt", "Windows path and process APIs")
class WindowsPathTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="nieuw-os-path-test-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.file = self.root / "plain.txt"
        self.file.write_text("test", encoding="utf-8")

    def test_plain_file_and_missing_output_under_root(self):
        self.assertEqual(inputs.assert_plain_path(self.file, root=self.root, regular=True), self.file)
        self.assertEqual(inputs.assert_plain_path(self.root / "new/out", root=self.root), self.root / "new/out")

    def test_parent_unc_namespace_drive_relative_and_root_escape_rejected(self):
        paths = [self.root / ".." / self.root.name / self.file.name,
                 r"\\server\share\file", r"\\?\C:\file", "C:relative",
                 self.root.parent / "other"]
        for path in paths:
            with self.subTest(path=str(path)), self.assertRaises(ValueError):
                inputs.assert_plain_path(path, root=self.root)

    def test_regular_rejects_directory_missing_and_hardlinked_file(self):
        with self.assertRaises(ValueError): inputs.assert_plain_path(self.root, regular=True)
        with self.assertRaises(FileNotFoundError): inputs.assert_plain_path(self.root / "missing", regular=True)
        alias = self.root / "alias"
        os.link(self.file, alias)
        with self.assertRaises(ValueError): inputs.assert_plain_path(alias, regular=True)

    def test_symlink_target_and_ancestor_rejected_when_available(self):
        linked = self.root / "symlink"
        try:
            os.symlink(self.root, linked, target_is_directory=True)
        except OSError as error:
            self.skipTest("Symlink creation unavailable: " + str(error.winerror))
        try:
            for path in (linked, linked / "plain.txt"):
                with self.assertRaises(ValueError): inputs.assert_plain_path(path)
        finally:
            linked.unlink()

    def test_junction_target_ancestor_and_root_rejected_without_admin(self):
        target = self.root / "target"; target.mkdir()
        (target / "inside").write_text("test", encoding="utf-8")
        link = self.root / "junction"
        result = subprocess.run(["cmd.exe", "/d", "/c", "mklink", "/J", str(link), str(target)],
            capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW, timeout=10)
        if result.returncode:
            self.fail("Could not create disposable junction for required negative test")
        try:
            for path in (link, link / "inside"):
                with self.assertRaises(ValueError): inputs.assert_plain_path(path)
            with self.assertRaises(ValueError): inputs.assert_plain_path(link / "inside", root=link)
        finally:
            os.rmdir(link)


@unittest.skipUnless(os.name == "nt", "Windows process-image API")
class ProcessImageTests(unittest.TestCase):
    @contextlib.contextmanager
    def child(self):
        process = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(30)"],
            stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
            creationflags=subprocess.CREATE_NO_WINDOW)
        try:
            yield process
        finally:
            if process.poll() is None: process.kill()
            process.wait(timeout=5)

    def test_real_child_image_and_mismatch(self):
        with self.child() as process:
            actual = inputs.process_image(process, Path(sys.executable))
            self.assertTrue(os.path.samefile(actual, sys.executable))
            with self.assertRaisesRegex(ValueError, "differs"):
                inputs.process_image(process, Path(__file__))

    def test_missing_invalid_and_closed_handles_fail(self):
        for process in (object(), types.SimpleNamespace(_handle=0), types.SimpleNamespace(_handle=-1)):
            with self.assertRaises(ValueError): inputs.process_image(process, Path(sys.executable))
        with self.child() as process:
            process.kill(); process.wait(timeout=5)
            process._handle.Close()
            with self.assertRaises((ValueError, OSError)):
                inputs.process_image(process, Path(sys.executable))


if __name__ == "__main__":
    unittest.main()

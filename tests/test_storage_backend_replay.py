import importlib.util
import io
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("storage_replay", ROOT / "tools/storage_backend_replay.py")
replay = importlib.util.module_from_spec(spec)
spec.loader.exec_module(replay)


class StorageReplayChecks(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir=ROOT)
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)

    def workspace(self):
        return replay.fresh_workspace(self.base / "work")

    def test_fresh_workspace_rejects_reuse_and_links(self):
        work = self.workspace()
        self.assertEqual(work.stat().st_mode & 0o777, 0o700)
        with self.assertRaises(FileExistsError):
            replay.fresh_workspace(work)
        link = self.base / "link"
        link.symlink_to(work, target_is_directory=True)
        with self.assertRaises(ValueError):
            replay.checked_path(link)

    def test_copy_requires_hash_and_independent_input(self):
        source = self.base / "source"
        source.write_bytes(b"synthetic")
        with self.assertRaisesRegex(ValueError, "SHA256"):
            replay.copy_checked(source, self.base / "bad", "0" * 64)
        alias = self.base / "alias"
        alias.hardlink_to(source)
        with self.assertRaisesRegex(ValueError, "hardlink"):
            replay.private_file(source)

    def test_allowlist_rejects_extra_package_before_parsing(self):
        packages = self.base / "packages"
        packages.mkdir(mode=0o700)
        (packages / "injected.deb").write_bytes(b"not executable")
        with self.assertRaisesRegex(ValueError, "allowlist"):
            replay.prepare(self.workspace(), packages, "openzfs")

    def test_archive_paths_and_special_files_are_rejected(self):
        for name in ("../outside", "/outside"):
            with self.subTest(name=name), self.assertRaises(ValueError):
                replay.member_path(name)
        data = io.BytesIO()
        with tarfile.open(fileobj=data, mode="w") as archive:
            member = tarfile.TarInfo("device")
            member.type = tarfile.CHRTYPE
            archive.addfile(member)
        with self.assertRaisesRegex(ValueError, "unsupported"):
            replay.unpack_tar(data.getvalue(), self.workspace() / "root")

    def test_preflight_failure_cannot_be_hidden_by_later_output(self):
        work = self.workspace()
        with self.assertRaisesRegex(ValueError, "unexpected exit"):
            replay.run_logged(work, "failure", [sys.executable, "-c", "print('help');raise SystemExit(3)"])
        self.assertTrue((work / "logs/failure.json").is_file())
        with self.assertRaisesRegex(ValueError, "missing output"):
            replay.run_logged(work, "missing", [sys.executable, "-c", "print('other')"], marker=b"usage:")

    def test_timeout_reaps_only_owned_process(self):
        work = self.workspace()
        with self.assertRaises(subprocess.TimeoutExpired):
            replay.run_logged(work, "timeout", [sys.executable, "-c", "import time;time.sleep(30)"], timeout=0.05)
        import json
        record = json.loads((work / "logs/timeout.json").read_text())
        self.assertTrue(record["timed_out"])
        self.assertTrue(record["leader_reaped"])

    def test_openzfs_execution_is_explicitly_blocked(self):
        with self.assertRaisesRegex(ValueError, "execution blocked"):
            replay.main(["--workspace", str(self.base / "work"), "--packages", str(self.base),
                         "--backend", "openzfs", "--execute"])
        self.assertFalse((self.base / "work").exists())

    def test_image_copy_is_bound_to_previously_recorded_baseline(self):
        source = self.base / "baseline.img"
        source.write_bytes(b"synthetic image")
        expected = replay.sha256(source)
        destination = self.base / "copy.img"
        self.assertEqual(replay.checked_image_copy(source, destination, expected), expected)
        self.assertEqual(source.read_bytes(), destination.read_bytes())
        with self.assertRaisesRegex(ValueError, "baseline"):
            replay.checked_image_copy(source, self.base / "bad.img", "0" * 64)
        self.assertFalse((self.base / "bad.img").exists())

    def test_future_zdb_argv_uses_only_project_copy_and_pool_name(self):
        work = self.workspace()
        image_dir = work / "images/copy"
        argv = replay.zdb_check_argv(work, image_dir)
        self.assertEqual(argv[-2:], [image_dir, "nieuwos_n3"])
        with self.assertRaisesRegex(ValueError, "project image"):
            replay.zdb_check_argv(work, self.base)

    def test_future_packaging_creates_images_directory_itself(self):
        work = self.workspace()
        crash = work / "images/crash"
        crash.mkdir(mode=0o700)
        (crash / "synthetic.img").write_bytes(b"synthetic")
        result = replay.package_evidence(work)
        with tarfile.open(result / "images/crash-pool.tar.gz") as archive:
            self.assertEqual(archive.extractfile("crash/synthetic.img").read(), b"synthetic")


if __name__ == "__main__":
    unittest.main()

"""HW-INV removable-media image (H06-T6): determinism, fail-closed names and independent readers."""
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
import unittest.mock
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import hw_inventory_image as image


class ImageTest(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp)
        self.esp = self.tmp / "esp"
        (self.esp / "EFI/BOOT").mkdir(parents=True)
        (self.esp / "EFI/BOOT/BOOTX64.EFI").write_bytes(bytes(range(256)) * 1200)
        (self.esp / "kernel.elf").write_bytes(b"\x7fELF" + bytes(700_001))
        (self.esp / "EMPTY").write_bytes(b"")

    def build(self) -> Path:
        path = self.tmp / "disk.img"
        path.write_bytes(image.build_image(self.esp))
        return path

    def partition(self) -> Path:
        disk = self.build().read_bytes()
        path = self.tmp / "part.img"
        path.write_bytes(disk[image.PART_START * image.SECTOR:(image.PART_END + 1) * image.SECTOR])
        return path

    def test_same_payload_same_bytes_and_payload_change_changes_hash(self):
        first = image.build_image(self.esp)
        self.assertEqual(len(first), image.IMAGE_BYTES)
        self.assertEqual(first, image.build_image(self.esp))
        (self.esp / "kernel.elf").write_bytes(b"\x7fELF" + bytes(700_000) + b"\1")
        self.assertNotEqual(first, image.build_image(self.esp))

    def test_rejects_names_outside_8_3_and_links(self):
        for bad in ("long-name.efi", "a.b.c", "space x", "kernel.elf64"):
            with self.subTest(bad):
                path = self.esp / bad
                path.write_bytes(b"x")
                with self.assertRaises(image.ImageError):
                    image.build_image(self.esp)
                path.unlink()
        (self.esp / "LINK").symlink_to(self.esp / "kernel.elf")
        with self.assertRaises(image.ImageError):
            image.build_image(self.esp)

    def test_rejects_case_collision_and_oversize_payload(self):
        (self.esp / "KERNEL.ELF").write_bytes(b"x")
        if len(list(self.esp.iterdir())) == 3:  # case-insensitive host filesystem
            self.skipTest("case-insensitive filesystem")
        with self.assertRaises(image.ImageError):
            image.build_image(self.esp)
        (self.esp / "KERNEL.ELF").unlink()
        (self.esp / "BIG").write_bytes(bytes(image.IMAGE_BYTES))
        with self.assertRaises(image.ImageError):
            image.build_image(self.esp)

    def test_write_image_refuses_a_leftover_esp_file(self):
        (self.esp / "EMPTY").unlink()
        manifest = dict(kernel_feature="native-hw-inventory", loader_feature="hw-inventory", source_sha256="s",
                        kernel=dict(sha256=image.sha256_bytes((self.esp / "kernel.elf").read_bytes())),
                        loader=dict(sha256=image.sha256_bytes((self.esp / "EFI/BOOT/BOOTX64.EFI").read_bytes())))
        (self.tmp / "manifest.json").write_text(json.dumps(manifest))
        with unittest.mock.patch.object(image, "source_hash", return_value="s"):
            self.assertEqual(image.checked_esp(self.tmp)[0], self.esp)
            # The ESP changes after the exact-set check: the single snapshot no longer matches the manifest.
            changed = image._payload(self.esp)
            changed[image._short("kernel.elf")] = b"other kernel"
            with unittest.mock.patch.object(image, "_payload", return_value=changed):
                with self.assertRaises(image.ImageError):
                    image.write_image(self.tmp)
            (self.esp / "STALE.EFI").write_bytes(b"x")
            with self.assertRaises(image.ImageError):
                image.write_image(self.tmp)
        self.assertFalse((self.tmp / "hw-inventory.img").exists())

    def test_write_image_does_not_follow_a_link_at_the_output_path(self):
        (self.esp / "EMPTY").unlink()
        manifest = dict(kernel_feature="native-hw-inventory", loader_feature="hw-inventory", source_sha256="s",
                        kernel=dict(sha256=image.sha256_bytes((self.esp / "kernel.elf").read_bytes())),
                        loader=dict(sha256=image.sha256_bytes((self.esp / "EFI/BOOT/BOOTX64.EFI").read_bytes())))
        (self.tmp / "manifest.json").write_text(json.dumps(manifest))
        victim = self.tmp / "victim"
        victim.write_bytes(b"keep")
        for name in ("hw-inventory.img", "hw-inventory-image.json"):
            with self.subTest(name):
                link = self.tmp / name
                link.symlink_to(victim)
                with unittest.mock.patch.object(image, "source_hash", return_value="s"), \
                        unittest.mock.patch.object(image.subprocess, "check_output", return_value="x"):
                    with self.assertRaises(image.ImageError):
                        image.write_image(self.tmp)
                self.assertEqual(victim.read_bytes(), b"keep")
                link.unlink()
        with unittest.mock.patch.object(image, "source_hash", return_value="s"), \
                unittest.mock.patch.object(image.subprocess, "check_output", return_value="x"):
            record = image.write_image(self.tmp)
        self.assertEqual(json.loads((self.tmp / "hw-inventory-image.json").read_text())["image_sha256"],
                         record["image_sha256"])

    def test_failed_record_write_never_leaves_an_old_record_beside_new_image_bytes(self):
        (self.esp / "EMPTY").unlink()
        manifest = dict(kernel_feature="native-hw-inventory", loader_feature="hw-inventory", source_sha256="s",
                        kernel=dict(sha256=image.sha256_bytes((self.esp / "kernel.elf").read_bytes())),
                        loader=dict(sha256=image.sha256_bytes((self.esp / "EFI/BOOT/BOOTX64.EFI").read_bytes())))
        (self.tmp / "manifest.json").write_text(json.dumps(manifest))
        disk, record = self.tmp / "hw-inventory.img", self.tmp / "hw-inventory-image.json"
        real_replace = image.os.replace

        def fail_second(source, target):
            if Path(target) == record:
                raise OSError("disk full")
            real_replace(source, target)

        disk.write_bytes(b"old image")
        record.write_text(json.dumps(dict(image_sha256=image.sha256_bytes(b"old image"))))
        with unittest.mock.patch.object(image, "source_hash", return_value="s"), \
                unittest.mock.patch.object(image.subprocess, "check_output", return_value="x"), \
                unittest.mock.patch.object(image.os, "replace", side_effect=fail_second):
            with self.assertRaises(image.ImageError):
                image.write_image(self.tmp)
        if record.exists():
            self.assertEqual(json.loads(record.read_text())["image_sha256"], image.sha256_bytes(disk.read_bytes()))
        self.assertEqual([p.name for p in self.tmp.iterdir() if p.name.startswith(".")], [])

    @unittest.skipUnless(shutil.which("sgdisk"), "NOT RUN: sgdisk absent")
    def test_gpt_verified_by_sgdisk(self):
        disk = self.build()
        result = subprocess.run(["sgdisk", "-v", str(disk)], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("No problems found", result.stdout)
        info = subprocess.run(["sgdisk", "-i", "1", str(disk)], capture_output=True, text=True)
        self.assertIn("C12A7328-F81F-11D2-BA4B-00A0C93EC93B", info.stdout)

    @unittest.skipUnless(shutil.which("fsck.fat"), "NOT RUN: fsck.fat absent")
    def test_fat32_verified_by_fsck_fat(self):
        result = subprocess.run(["fsck.fat", "-n", "-v", str(self.partition())], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("32 bit entries", result.stdout)
        self.assertNotIn("Auto-", result.stdout)

    @unittest.skipUnless(shutil.which("mcopy"), "NOT RUN: mtools absent")
    def test_files_read_back_by_mtools(self):
        part = self.partition()
        for inside, original in (("::/KERNEL.ELF", "kernel.elf"), ("::/EFI/BOOT/BOOTX64.EFI", "EFI/BOOT/BOOTX64.EFI"),
                                 ("::/EMPTY", "EMPTY")):
            out = self.tmp / "out"
            subprocess.run(["mcopy", "-n", "-i", str(part), inside, str(out)], check=True)
            self.assertEqual(out.read_bytes(), (self.esp / original).read_bytes(), inside)
            out.unlink()


if __name__ == "__main__":
    unittest.main()

"""Host verifier counterexamples; these tests are not native system evidence."""
import copy
import hashlib
import os
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from run_system import check_disk, check_esp, check_image, evaluate, expected_markers
import volume_disk


def transcript(case="normal", **changes):
    values = dict(APP_READY=2, APP_DOCS=1, APP_REVISION=1, APP_BYTES=4096, APP_DENIED=3,
                  APP_CONTENT_SUM=volume_disk.fnv(volume_disk.fixture(0, 1, 4096)),
                  WORKER_SPAWNS=2, WORKER_FAULTS=1, WORKER_RESTARTS=1, WORKER_KILLS=1,
                  LIVE_PROCESSES_AFTER=0, FRAMES_RECOVERED=1, DISK_READS=304,
                  DISK_WRITES=42 if case == "normal" else 0,
                  DISK_FLUSHES=3 if case == "normal" else 0, ENTROPY_BYTES=1024,
                  CRYPTO_SEAL=2, CRYPTO_UNSEAL=2, CRYPTO_TAMPER_DENIED=9,
                  RECORD_SEAL=1, RECORD_UNSEAL=1, RECORD_CONTEXT_DENIED=10, RECORD_REVOKED_DENIED=2)
    values.update(changes)
    return "\n".join(expected_markers(case)) + "\n" + "\n".join(
        f"SYSTEM:{key}:{value:020d}" for key, value in values.items()) + "\n"


class SystemHarnessTests(unittest.TestCase):
    def test_stale_or_replaced_esp_cannot_use_current_manifest(self):
        with tempfile.TemporaryDirectory() as directory:
            esp = Path(directory)
            loader = esp / "EFI/BOOT/BOOTX64.EFI"
            loader.parent.mkdir(parents=True)
            loader.write_bytes(b"inspected loader")
            kernel = esp / "kernel.elf"
            kernel.write_bytes(b"inspected kernel")
            manifest = {key: {"sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
                        for key, path in (("loader", loader), ("kernel", kernel))}
            check_esp(esp, manifest)
            for path in (kernel, loader):
                original = path.read_bytes()
                path.write_bytes(b"different previously passing program")
                with self.assertRaises(ValueError):
                    check_esp(esp, manifest)
                path.write_bytes(original)

    def test_disk_guard_rejects_wrong_geometry_symlink_hardlink_and_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            image = root / "volume.img"
            image.write_bytes(bytes(volume_disk.IMAGE_SECTORS * volume_disk.SECTOR))
            check_image(image)
            link = root / "linked.img"
            link.symlink_to(image)
            with self.assertRaises(ValueError):
                check_image(link)
            os.link(image, root / "hard.img")
            with self.assertRaises(ValueError):
                check_image(image)
            (root / "hard.img").unlink()
            image.write_bytes(b"too small")
            with self.assertRaises(ValueError):
                check_image(image)
            with self.assertRaises(ValueError):
                check_image(root)

    def test_good_transcripts_and_unrecognized_case(self):
        for case in ("normal", "recover"):
            self.assertEqual(evaluate(case, transcript(case), 33, False)["APP_BYTES"], 4096)
        with self.assertRaises(ValueError):
            evaluate("unknown", "", 33, False)

    def test_wrong_exit_timeout_and_false_success(self):
        for raw, timeout in [(0, False), (None, True), (33, True), (35, False)]:
            with self.subTest(raw=raw, timeout=timeout), self.assertRaises(ValueError):
                evaluate("normal", transcript(), raw, timeout)
        with self.assertRaises(ValueError):
            evaluate("normal", transcript() + "SYSTEM:FAIL:INJECTED\n", 33, False)

    def test_omitted_duplicate_reordered_and_wrong_case_markers(self):
        good = transcript()
        bad = [good.replace("SYSTEM:CLEANUP:PASS\n", ""),
               good + "SYSTEM:CLEANUP:PASS\n",
               good.replace("SYSTEM:CLEANUP:PASS", "SYSTEM:BROKERS:PASS"),
               good.replace("CASE:NORMAL", "CASE:RECOVER")]
        for text in bad:
            with self.subTest(text=text[:80]), self.assertRaises(ValueError):
                evaluate("normal", text, 33, False)

    def test_missing_duplicate_and_malformed_numeric_evidence(self):
        good = transcript()
        line = "SYSTEM:APP_READY:00000000000000000002\n"
        for text in [good.replace(line, ""), good + line, good.replace(line, "SYSTEM:APP_READY:2\n")]:
            with self.assertRaises(ValueError):
                evaluate("normal", text, 33, False)

    def test_crypto_io_privilege_lifecycle_and_cleanup_cannot_be_omitted(self):
        for key in ("APP_READY", "APP_DOCS", "APP_REVISION", "APP_BYTES", "APP_DENIED",
                    "APP_CONTENT_SUM", "WORKER_SPAWNS", "WORKER_FAULTS", "WORKER_RESTARTS",
                    "WORKER_KILLS", "FRAMES_RECOVERED", "DISK_READS", "DISK_WRITES", "DISK_FLUSHES",
                    "ENTROPY_BYTES", "CRYPTO_SEAL", "CRYPTO_UNSEAL", "CRYPTO_TAMPER_DENIED",
                    "RECORD_SEAL", "RECORD_UNSEAL", "RECORD_CONTEXT_DENIED", "RECORD_REVOKED_DENIED"):
            with self.subTest(key=key), self.assertRaises(ValueError):
                evaluate("normal", transcript(**{key: 0}), 33, False)
        with self.assertRaises(ValueError):
            evaluate("normal", transcript(LIVE_PROCESSES_AFTER=1), 33, False)

    def test_recovery_rejects_writes_or_flushes(self):
        for key in ("DISK_WRITES", "DISK_FLUSHES"):
            with self.subTest(key=key), self.assertRaises(ValueError):
                evaluate("recover", transcript("recover", **{key: 1}), 33, False)

    def test_wrong_disk_object_domain_revision_or_usage_is_not_success(self):
        obj = dict(present=True, revision=1, collection=0, tags=0, kind=0,
                   versions=[dict(revision=1, length=4096, name="systeemproef", content_revision=1)])
        empty = dict(present=False, revision=0, versions=[])
        good = dict(objects=[obj] + [empty] * 15, generation=1, chunks_used=1, clocks=[1, 0],
                    collections=[dict(present=False, revision=0)] * 8)
        check_disk(good)
        for mutate in (lambda s: s.update(generation=2), lambda s: s.update(chunks_used=2),
                       lambda s: s.update(clocks=[0, 1]),
                       lambda s: s["objects"][0]["versions"][0].update(name="ander"),
                       lambda s: s["objects"][0]["versions"][0].update(content_revision=2),
                       lambda s: s["objects"][1].update(present=True)):
            bad = copy.deepcopy(good)
            mutate(bad)
            with self.assertRaises(ValueError):
                check_disk(bad)


if __name__ == "__main__":
    unittest.main()

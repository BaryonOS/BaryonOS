"""Host verifier counterexamples; these tests are not native system evidence."""
import copy
import hashlib
import os
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from run_system import (KEYSTATE_HEADER, KEYSTATE_NAME, KEYSTATE_OBJECT, OPAQUE_OBJECTS, PERSIST_EXACT,
                        SEALED_NAME, SEALED_OBJECT, check_disk, check_esp, check_image, check_parent,
                        evaluate, expected_markers, record_plaintext)
import volume_disk


def transcript(case="normal", **changes):
    values = dict(APP_READY=2, APP_DOCS=1, APP_REVISION=1, APP_BYTES=4096, APP_DENIED=3,
                  APP_CONTENT_SUM=volume_disk.fnv(volume_disk.fixture(0, 1, 4096)),
                  WORKER_SPAWNS=2, WORKER_FAULTS=1, WORKER_RESTARTS=1, WORKER_KILLS=1,
                  LIVE_PROCESSES_AFTER=0, FRAMES_RECOVERED=1, DISK_READS=304,
                  DISK_WRITES=42 if case == "normal" else 0,
                  DISK_FLUSHES=3 if case == "normal" else 0, ENTROPY_BYTES=1024,
                  RECORD_SEAL=1, RECORD_UNSEAL=1, RECORD_CONTEXT_DENIED=10, RECORD_REVOKED_DENIED=2,
                  **PERSIST_EXACT[case])
    values.update(changes)
    return "\n".join(expected_markers(case)) + "\n" + "\n".join(
        f"SYSTEM:{key}:{value:020d}" for key, value in values.items()) + "\n"


# Host-only opaque fixtures with the committed B02 shapes; no key is involved.
SEALED_FIXTURE = bytes(range(1, 25)) + bytes((i * 29 + 5) & 0xFF for i in range(144))
KEYSTATE_FIXTURE = KEYSTATE_HEADER + bytes((i * 31 + 7) & 0xFF for i in range(88))


def system_image() -> bytes:
    """Independent fixture of the normal run's final disk: the plaintext
    document, the sealed record (domain 0) and the key-state snapshot (domain 1)."""
    disk = volume_disk
    with tempfile.TemporaryDirectory() as directory:
        image = Path(directory) / "volume.img"
        disk.create_image(image)
        raw = bytearray(image.read_bytes())
    document = disk.fixture(0, 1, 4096)
    entries = [disk.empty_entry() for _ in range(disk.OBJECTS)]
    placed = [(0, document, b"systeemproef", 0, 1, 1), (SEALED_OBJECT, SEALED_FIXTURE, SEALED_NAME.encode(), 1, 2, 2),
              (KEYSTATE_OBJECT, KEYSTATE_FIXTURE, KEYSTATE_NAME.encode(), 2, 1, 3)]
    # Both table copies as the real commit sequence leaves them: generation 2
    # (document and sealed record) in slot 0, generation 3 (plus the key-state
    # snapshot) in slot 1.
    for obj, data, name, chunk, stamp, request in placed:
        start = (disk.FIRST + disk.DATA_START + chunk * disk.CHUNK_SECTORS) * disk.SECTOR
        raw[start:start + len(data)] = data
        entries[obj] = dict(disk.empty_entry(), revision=1, request_id=request, created=stamp, modified=stamp,
                            present=True, current=dict(revision=1, length=len(data), name=name,
                                                       content_sum=disk.fnv(data), chunks=[chunk],
                                                       sums=[disk.fnv(data)]))
        if obj == SEALED_OBJECT:
            for index, sector in enumerate(disk.encode_table(2, 2, entries, clocks=[2, 0])):
                start = (disk.FIRST + index) * disk.SECTOR
                raw[start:start + disk.SECTOR] = sector
    for index, sector in enumerate(disk.encode_table(3, 3, entries, clocks=[2, 1])):
        start = (disk.FIRST + disk.TABLE_SECTORS + index) * disk.SECTOR
        raw[start:start + disk.SECTOR] = sector
    return bytes(raw)


class SystemHarnessTests(unittest.TestCase):
    def test_partial_network_pass_cannot_authorize_cold_recovery(self):
        previous = dict(case="normal", passed=True, kernel_feature="native-system-net",
                        source_sha256="source", after_sha256="disk")
        # The peer wrapper may crash after generic success, or fail to persist
        # its error. Missing/false/boolean-like flags never authorize recovery.
        for marker in (None, False, 1, "true"):
            stage = dict(previous)
            if marker is not None:
                stage["network_gate"] = marker
            with self.subTest(marker=marker), self.assertRaises(ValueError):
                check_parent(stage, "source", "disk", network=True)
        previous["network_gate"] = True
        check_parent(previous, "source", "disk", network=True)
        for change in (dict(passed=False), dict(source_sha256="other"),
                       dict(after_sha256="other"), dict(kernel_feature="native-system")):
            with self.subTest(change=change), self.assertRaises(ValueError):
                check_parent({**previous, **change}, "source", "disk", network=True)
        base = {**previous, "kernel_feature": "native-system"}
        base.pop("network_gate")
        check_parent(base, "source", "disk", network=False)

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

    def test_unknown_or_linked_source_esp_is_rejected_before_copy(self):
        # Synthetic marker only. The whole source tree must be checked before
        # copytree or QEMU sees it, even if the two expected hashes are correct.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            esp = root / "esp"
            loader = esp / "EFI/BOOT/BOOTX64.EFI"
            loader.parent.mkdir(parents=True)
            loader.write_bytes(b"loader")
            kernel = esp / "kernel.elf"
            kernel.write_bytes(b"kernel")
            manifest = {key: {"sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
                        for key, path in (("loader", loader), ("kernel", kernel))}
            marker = root / "outside-marker"
            marker.write_bytes(b"host-only test marker")
            check_esp(esp, manifest)
            extra = esp / "extra"
            for kind in ("regular", "symlink", "fifo", "directory"):
                with self.subTest(kind=kind):
                    if kind == "regular":
                        extra.write_bytes(b"unexpected bytes")
                    elif kind == "symlink":
                        extra.symlink_to(marker)
                    elif kind == "fifo":
                        os.mkfifo(extra)
                    else:
                        extra.mkdir()
                    with self.assertRaises(ValueError):
                        check_esp(esp, manifest)
                    extra.rmdir() if kind == "directory" else extra.unlink()
                    self.assertEqual(marker.read_bytes(), b"host-only test marker")
            # The two leaf files may be regular while their ancestor is linked.
            boot = esp / "EFI/BOOT"
            actual = root / "actual-boot"
            boot.rename(actual)
            boot.symlink_to(actual, target_is_directory=True)
            with self.assertRaises(ValueError):
                check_esp(esp, manifest)

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
                    "RECORD_SEAL", "RECORD_UNSEAL", "RECORD_CONTEXT_DENIED", "RECORD_REVOKED_DENIED",
                    "CRYPTO_RESTORES", "KEYSTATE_EXPORTS", "KEYSTATE_RESTORES",
                    "KEYSTATE_WRONG_SECRET_DENIED", "KEYSTATE_FLOOR_DENIED", "KEYSTATE_ADMIN_OP_DENIED",
                    "KEYSTATE_BYTES", "KEYSTATE_GENERATION", "ADMIN_REQUESTS", "ADMIN_DENIED",
                    "RECORD_PERSISTED", "RECORD_ADMIN_DENIED", "RECORD_OBJECT_REVISION"):
            with self.subTest(key=key), self.assertRaises(ValueError):
                evaluate("normal", transcript(**{key: 0}), 33, False)
        with self.assertRaises(ValueError):
            evaluate("normal", transcript(LIVE_PROCESSES_AFTER=1), 33, False)

    def test_persistence_evidence_is_case_specific(self):
        # A normal transcript must not pass as a cold recovery and vice versa:
        # the restored unseal, the post-restart refusals and the export count differ.
        with self.assertRaises(ValueError):
            evaluate("recover", transcript("normal").replace("CASE:NORMAL", "CASE:RECOVER")
                     .replace("00000000000000000042", "00000000000000000000")
                     .replace("SYSTEM:DISK_FLUSHES:00000000000000000003", "SYSTEM:DISK_FLUSHES:00000000000000000000"),
                     33, False)
        for key in ("RECORD_RESTORED_UNSEAL", "RECORD_RESTORED_REVOKED_DENIED", "RECORD_RESTORED_TAMPER_DENIED",
                    "KEYSTATE_RESTORES"):
            with self.subTest(key=key), self.assertRaises(ValueError):
                evaluate("recover", transcript("recover", **{key: 0}), 33, False)
        for key, value in (("KEYSTATE_EXPORTS", 1), ("RECORD_PERSISTED", 1), ("CRYPTO_SEAL", 3)):
            with self.subTest(key=key), self.assertRaises(ValueError):
                evaluate("recover", transcript("recover", **{key: value}), 33, False)
        with self.assertRaises(ValueError):
            evaluate("normal", transcript("normal", KEYSTATE_EXPORTS=2, ADMIN_REQUESTS=6), 33, False)

    def test_recovery_rejects_writes_or_flushes(self):
        for key in ("DISK_WRITES", "DISK_FLUSHES"):
            with self.subTest(key=key), self.assertRaises(ValueError):
                evaluate("recover", transcript("recover", **{key: 1}), 33, False)

    def test_wrong_disk_object_domain_revision_or_usage_is_not_success(self):
        good = volume_disk.analyse(system_image(), OPAQUE_OBJECTS)
        check_disk(good)
        self.assertEqual((good["generation"], good["chunks_used"], good["clocks"]), (3, 3, [2, 1]))
        plaintext = record_plaintext()
        for mutate in (lambda s: s.update(generation=2), lambda s: s.update(chunks_used=2),
                       lambda s: s.update(clocks=[3, 0]),
                       lambda s: s["objects"][0]["versions"][0].update(name="ander"),
                       lambda s: s["objects"][0]["versions"][0].update(content_revision=2),
                       lambda s: s["objects"][2].update(present=True),
                       lambda s: s["objects"][9].update(present=True),
                       lambda s: s["objects"][SEALED_OBJECT].update(present=False),
                       lambda s: s["objects"][KEYSTATE_OBJECT].update(present=False),
                       lambda s: s["objects"][SEALED_OBJECT]["versions"][0].update(content_hex=plaintext.hex()),
                       lambda s: s["objects"][SEALED_OBJECT]["versions"][0].update(
                           content_hex=(bytes(24) + plaintext + bytes(16)).hex()),
                       lambda s: s["objects"][SEALED_OBJECT]["versions"][0].update(
                           content_hex=(bytes(24) + SEALED_FIXTURE[24:]).hex()),
                       lambda s: s["objects"][SEALED_OBJECT]["versions"][0].update(content_revision=1),
                       lambda s: s["objects"][SEALED_OBJECT]["versions"][0].update(name="ander"),
                       lambda s: s["objects"][SEALED_OBJECT]["versions"][0].pop("content_hex"),
                       lambda s: s["objects"][KEYSTATE_OBJECT]["versions"][0].update(
                           content_hex=(b"NOSKEY02" + KEYSTATE_FIXTURE[8:]).hex()),
                       lambda s: s["objects"][KEYSTATE_OBJECT]["versions"][0].update(
                           content_hex=(KEYSTATE_FIXTURE[:12] + b"other-volume-id!" + KEYSTATE_FIXTURE[28:]).hex()),
                       lambda s: s["objects"][KEYSTATE_OBJECT]["versions"][0].update(
                           content_hex=(KEYSTATE_FIXTURE[:32] + bytes(24) + KEYSTATE_FIXTURE[56:]).hex()),
                       lambda s: s["objects"][KEYSTATE_OBJECT]["versions"][0].update(
                           content_hex=KEYSTATE_FIXTURE[:119].hex(), length=119)):
            bad = copy.deepcopy(good)
            mutate(bad)
            with self.assertRaises(ValueError):
                check_disk(bad)

    def test_opaque_objects_are_explicit_in_the_oracle(self):
        image = system_image()
        with self.assertRaises(ValueError):
            volume_disk.analyse(image)
        with self.assertRaises(ValueError):
            volume_disk.analyse(image, frozenset({SEALED_OBJECT}))
        oracle = volume_disk.analyse(image, OPAQUE_OBJECTS)
        sealed = oracle["objects"][SEALED_OBJECT]["versions"][0]
        self.assertIsNone(sealed["content_revision"])
        self.assertEqual(bytes.fromhex(sealed["content_hex"]), SEALED_FIXTURE)
        self.assertNotIn("content_hex", oracle["objects"][0]["versions"][0])


if __name__ == "__main__":
    unittest.main()

"""Host verifier counterexamples; these tests are not native system evidence."""
import copy
import hashlib
import os
from pathlib import Path
import re
import sys
import tempfile
import unittest
import unittest.mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from run_system import (DEVICE_ORDER, KEYSTATE_HEADER, KEYSTATE_NAME, KEYSTATE_OBJECT, OPAQUE_OBJECTS, PERSIST_EXACT,
                        SEALED_NAME, SEALED_OBJECT, check_disk, check_esp, check_image, check_parent,
                        S5_TAIL, COLD_CASES, cold_image, evaluate_cold, evaluate, expected_markers, record_plaintext,
                        BLK_CASES, BOOT, evaluate_lock_red)
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


class ColdHarnessTests(unittest.TestCase):
    def test_mutations_reach_aead_with_valid_outer_checksums(self):
        raw = system_image()
        original = volume_disk.analyse(raw, OPAQUE_OBJECTS)
        for case in COLD_CASES:
            with self.subTest(case=case):
                changed = cold_image(raw, case)
                oracle = volume_disk.analyse(changed, OPAQUE_OBJECTS)
                self.assertEqual(oracle["generation"], original["generation"])
                obj = KEYSTATE_OBJECT if case == "keystate" else SEALED_OBJECT
                before = bytes.fromhex(original["objects"][obj]["versions"][0]["content_hex"])
                after = bytes.fromhex(oracle["objects"][obj]["versions"][0]["content_hex"])
                if case == "withheld-secret":
                    self.assertEqual(changed, raw)
                else:
                    self.assertNotEqual(changed, raw)
                    self.assertNotEqual(before, after)
                    if case == "keystate":
                        self.assertEqual(before[:56], after[:56])
                        self.assertEqual(sum(a != b for a, b in zip(before, after)), 1)
                    elif case == "record-tag":
                        self.assertEqual(before[:-1], after[:-1])
                    else:
                        self.assertEqual(after, before[:24] + before[88:152] + before[24:88] + before[152:])
        with self.assertRaises(ValueError):
            cold_image(raw, "unknown")

    def test_refusal_requires_exact_case_no_plaintext_and_no_writes(self):
        for case in COLD_CASES:
            log = "\n".join((*expected_markers("recover")[:4], f"SYSTEM:COLD:REFUSED:{case}",
                "SYSTEM:COLD_PLAINTEXT_REPLIES:00000000000000000000",
                "SYSTEM:DISK_READS:00000000000000000001", "SYSTEM:DISK_WRITES:00000000000000000000",
                "SYSTEM:DISK_FLUSHES:00000000000000000000"))
            self.assertEqual(evaluate_cold(case, log, 33, False)["DISK_WRITES"], 0)
            bad_logs = [log + "\nSYSTEM:PLAINTEXT:PASS", log + f"\nSYSTEM:COLD:REFUSED:{case}",
                        log.replace("DISK_WRITES:00000000000000000000", "DISK_WRITES:00000000000000000001"),
                        log.replace(f"REFUSED:{case}", "REFUSED:other"),
                        log.replace("SYSTEM:COLD_PLAINTEXT_REPLIES:00000000000000000000", "")]
            for bad in bad_logs:
                with self.assertRaises(ValueError):
                    evaluate_cold(case, bad, 33, False)
            for rc, timeout in ((0, False), (33, True), (35, False)):
                with self.assertRaises(ValueError):
                    evaluate_cold(case, log, rc, timeout)

    def test_cold_log_cannot_carry_success_case_or_persistence_markers(self):
        for case in COLD_CASES:
            log = "\n".join((*expected_markers("recover")[:4], f"SYSTEM:COLD:REFUSED:{case}",
                "SYSTEM:COLD_PLAINTEXT_REPLIES:00000000000000000000",
                "SYSTEM:DISK_READS:00000000000000000001", "SYSTEM:DISK_WRITES:00000000000000000000",
                "SYSTEM:DISK_FLUSHES:00000000000000000000"))
            for extra in ("SYSTEM:CASE:RECOVER:PASS", "M1:PASS", "SYSTEM:BROKERS:PASS", "SYSTEM:CLEANUP:PASS",
                          "SYSTEM:RECORD_RESTORED_UNSEAL:00000000000000000001",
                          "SYSTEM:KEYSTATE_EXPORTS:00000000000000000000", "SYSTEM:POWER:GATE:PASS"):
                with self.subTest(case=case, extra=extra), self.assertRaisesRegex(
                        ValueError, "cold log carries a success, case or persistence marker"):
                    evaluate_cold(case, log + "\n" + extra, 33, False)

    def test_replay_is_an_open_boundary_not_a_cold_case(self):
        from run_system import COLD_ANTIROLLBACK
        self.assertNotIn("replay", COLD_CASES)
        with self.assertRaises(ValueError):
            cold_image(system_image(), "replay")
        self.assertEqual(COLD_ANTIROLLBACK["status"], "UNPROTECTED")
        self.assertTrue(COLD_ANTIROLLBACK["replay"].startswith("NOT_DETECTABLE"))

    def test_cold_binding_requires_matching_fixture_and_unchanged_parent(self):
        from run_system import check_cold_binding
        same = "a" * 64
        check_cold_binding("keystate", "keystate", same, same)
        for env, before, after in (("withheld-secret", same, same), (None, same, same), ("", same, same),
                                   ("keystate", same, "b" * 64), ("keystate", None, None)):
            with self.subTest(env=env, before=before, after=after), self.assertRaises(ValueError):
                check_cold_binding("keystate", env, before, after)
        with self.assertRaises(ValueError):
            check_cold_binding("replay", "replay", same, same)


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


# Printed by the kernel between BROKERS:PASS and CLEANUP:PASS, and after it.
TEARDOWN = ("LIVE_PROCESSES_AFTER", "FRAMES_RECOVERED")
DEVICE = tuple(f"SUSPEND_{key}" for key in (*DEVICE_ORDER, "DEVICE_IO_ACCOUNTED_VOLUME"))
AT_WAKE = dict(SUSPEND_DEVICE_IO_AT_WAKE_KEYRING=9, SUSPEND_DEVICE_IO_AT_WAKE_VOLUME=700)


def power_transcript(case="power", tail=None, **changes):
    """H05 power run in the kernel's real order: the wake evidence and the
    counters, BROKERS, teardown, CLEANUP, the post-wake device evidence, then
    the gate, case marker and S5 tail."""
    ok = case in ("power", "power-lock", "power-s3", "power-s3-reinit", "power-s3-held-completion", *BLK_CASES)
    values = dict(POWER_INIT_REFUSALS=3 if ok else 4, POWER_APP_REFUSALS=2,
                  POWER_NOT_DURABLE=0 if ok else 1, DISK_FLUSHES=4)
    held = int(case == "power-s3-held-completion")
    if case in ("power-s3", "power-s3-held-completion"):
        values.update(SUSPEND_REFUSALS=5 + held, SUSPEND_GRANTED=1, SUSPEND_TICK_AFTER_WAKE=1,
                      SUSPEND_HELD_REFUSALS=held, SUSPEND_LOST_ENTROPY=14, SUSPEND_LOST_BLOCK=14,
                      **AT_WAKE, SUSPEND_DEVICE_IO_AFTER_WAKE=0, SUSPEND_DEVICE_SUPPRESSED=held)
    if case == "power-s3-reinit":
        values.update(SUSPEND_REFUSALS=5, SUSPEND_GRANTED=1, SUSPEND_TICK_AFTER_WAKE=1,
                      SUSPEND_HELD_REFUSALS=0, SUSPEND_REINIT_ENTROPY=32, SUSPEND_REINIT_STALE_TICKET=1,
                      SUSPEND_REINIT_ENTROPY_AUTH=6, SUSPEND_LOST_BLOCK=14,
                      **AT_WAKE, SUSPEND_DEVICE_IO_AFTER_WAKE=0, SUSPEND_DEVICE_SUPPRESSED=0)
    if case in BLK_CASES:
        quarantined = BLK_CASES[case] is not None
        values.update(SUSPEND_REFUSALS=5, SUSPEND_GRANTED=1, SUSPEND_TICK_AFTER_WAKE=1,
                      SUSPEND_HELD_REFUSALS=0, SUSPEND_REINIT_ENTROPY=32, SUSPEND_REINIT_STALE_TICKET=1,
                      SUSPEND_REINIT_ENTROPY_AUTH=6, SUSPEND_BLK_QUARANTINED=int(quarantined),
                      SUSPEND_BLK_STALE_TICKET=0 if quarantined else 1,
                      SUSPEND_BLK_PIC_MASTER=0 if quarantined else 0xfe,
                      SUSPEND_BLK_PIC_SLAVE=0 if quarantined else 0xff,
                      SUSPEND_BLK_INTX_LINE=0 if quarantined else 10,
                      SUSPEND_BLK_LATCH_CARRIED=1,
                      SUSPEND_LOST_BLOCK=24 if quarantined else 6,
                      **AT_WAKE, SUSPEND_DEVICE_IO_AFTER_WAKE=0, SUSPEND_DEVICE_SUPPRESSED=0,
                      SUSPEND_DEVICE_IO_ACCOUNTED_VOLUME=0 if case.endswith("-busy") else 8)
    values.update(changes)
    values = {key: value for key, value in values.items() if value is not None}
    lines = transcript("normal", **values).replace(
        "SYSTEM:BROKERS:PASS\nSYSTEM:CLEANUP:PASS\nSYSTEM:CASE:NORMAL:PASS\nM1:PASS\n", "").splitlines()
    group = [[line for line in lines if line.split(":")[1] in keys] for keys in (TEARDOWN, DEVICE)]
    body = "".join(f"{line}\n" for line in (
        *(line for line in lines if line not in group[0] + group[1]), "SYSTEM:BROKERS:PASS",
        *group[0], "SYSTEM:CLEANUP:PASS", *group[1]))
    marker = expected_markers(case)[-2]
    if ok:
        end = ["SYSTEM:POWER:GATE:PASS", marker, "M1:PASS", *S5_TAIL] if tail is None else tail
    else:
        fault = "FLUSH_ERROR" if case == "power-flush-error" else "FLUSH_TIMEOUT"
        end = [f"SYSTEM:FAULT_INJECTED:{fault}", "SYSTEM:POWER:REFUSED:NOT_DURABLE", marker,
               "M1:PASS"] if tail is None else tail
    return body + "\n".join(end) + "\n"


S3_PLAN = ("POWER:S3:PLAN:PM1A=0x0604:SLP_TYP=1:PM1A_STS=0x0600:FACS_VERSION=0:"
           "REAL=0x1000:PM32=0x2000510:POLICY=Edk2Version0Extended")
S3_ENTER = "POWER:S3:ENTER:CYCLE=1:SCI_EN=1"
S3_RESUMED = ("POWER:S3:RESUMED:CYCLE=1:ENTRY=PM32:PM1_STS=0x8101:WAK_STS=1:SCI_EN=RESTORED:"
              "HWSIG=SAME:FACS=DISARMED:CANARY=PASS:CR3=SAME")


def s3_transcript(s3=(S3_ENTER, S3_RESUMED), plan=S3_PLAN, case="power-s3", **changes):
    """H05-T4 power-s3 in the kernel's real order: plan at bootstrap, the one
    cycle inside init's run (after MANIFEST, before the broker evidence)."""
    text = power_transcript(case, **changes)
    before = (plan + "\n") if plan else ""
    return text.replace("SYSTEM:MANIFEST:PASS\n",
                        before + "SYSTEM:MANIFEST:PASS\n" + "".join(line + "\n" for line in s3), 1)


class PowerS3HarnessTests(unittest.TestCase):
    def test_passes_only_with_raw_exit_zero_and_parses_every_suspend_counter(self):
        found = evaluate("power-s3", s3_transcript(), 0, False)
        self.assertEqual((found["SUSPEND_REFUSALS"], found["SUSPEND_GRANTED"],
                          found["SUSPEND_TICK_AFTER_WAKE"]), (5, 1, 1))
        for raw, timed_out in ((33, False), (35, False), (None, True), (0, True)):
            with self.subTest(raw=raw, timed_out=timed_out), self.assertRaises(ValueError):
                evaluate("power-s3", s3_transcript(), raw, timed_out)

    def test_exactly_one_planned_entered_resumed_cycle_in_order(self):
        bad_resumes = [S3_RESUMED.replace(old, new) for old, new in (
            ("CR3=SAME", "CR3=CHANGED"), ("CANARY=PASS", "CANARY=FAIL"), ("CYCLE=1", "CYCLE=2"),
            ("FACS=DISARMED", "FACS=STORE_FAILED"), ("WAK_STS=1", "WAK_STS=0"),
            ("ENTRY=PM32", "ENTRY=REAL"), ("HWSIG=SAME", "HWSIG=CHANGED"))]
        cases = [dict(s3=(S3_ENTER,)), dict(s3=(S3_RESUMED,)), dict(s3=(S3_RESUMED, S3_ENTER)),
                 dict(s3=(S3_ENTER, S3_ENTER, S3_RESUMED)), dict(s3=(S3_ENTER, S3_RESUMED, S3_RESUMED)),
                 dict(plan=None), dict(s3=(S3_ENTER, S3_RESUMED, "POWER:S3:NO_TRANSITION:DISARMED")),
                 *[dict(s3=(S3_ENTER, line)) for line in bad_resumes]]
        for case in cases:
            with self.subTest(case=case), self.assertRaises(ValueError):
                evaluate("power-s3", s3_transcript(**case), 0, False)

    def test_s3_lines_after_the_broker_evidence_are_refused(self):
        text = s3_transcript(s3=()).replace("SYSTEM:BROKERS:PASS\n",
                                            f"SYSTEM:BROKERS:PASS\n{S3_ENTER}\n{S3_RESUMED}\n")
        with self.assertRaisesRegex(ValueError, "order"):
            evaluate("power-s3", text, 0, False)

    def test_suspend_and_power_counters_are_exact(self):
        for change in (dict(SUSPEND_GRANTED=0), dict(SUSPEND_REFUSALS=4), dict(SUSPEND_TICK_AFTER_WAKE=0),
                       dict(POWER_INIT_REFUSALS=4)):
            with self.subTest(change=change), self.assertRaises(ValueError):
                evaluate("power-s3", s3_transcript(**change), 0, False)
        plain = s3_transcript().replace("SYSTEM:CASE:POWER_S3:PASS", "SYSTEM:CASE:POWER:PASS")
        with self.assertRaises(ValueError):
            evaluate("power-s3", plain, 0, False)

    def test_post_wake_device_refusals_are_exact_and_ordered(self):
        found = evaluate("power-s3", s3_transcript(), 0, False)
        self.assertEqual((found["SUSPEND_LOST_ENTROPY"], found["SUSPEND_LOST_BLOCK"]), (14, 14))
        # Missing refusal, a 0 instead of -14, or another refusal code.
        for change in (dict(SUSPEND_LOST_ENTROPY=None), dict(SUSPEND_LOST_BLOCK=None),
                       dict(SUSPEND_LOST_ENTROPY=0), dict(SUSPEND_LOST_BLOCK=0),
                       dict(SUSPEND_LOST_ENTROPY=6), dict(SUSPEND_LOST_BLOCK=4)):
            with self.subTest(change=change), self.assertRaises(ValueError):
                evaluate("power-s3", s3_transcript(**change), 0, False)
        entropy = "SYSTEM:SUSPEND_LOST_ENTROPY:" + "14".zfill(20)
        block = "SYSTEM:SUSPEND_LOST_BLOCK:" + "14".zfill(20)
        swapped = s3_transcript().replace(entropy, "SWAP").replace(block, entropy).replace("SWAP", block)
        with self.assertRaisesRegex(ValueError, "order"):
            evaluate("power-s3", swapped, 0, False)

    def test_suspend_evidence_outside_power_s3_is_refused(self):
        text = power_transcript("power", SUSPEND_LOST_ENTROPY=14)
        with self.assertRaisesRegex(ValueError, "outside"):
            evaluate("power", text, 0, False)
        # Refused-power, normal and cold cases are checked too, not only "power".
        stray = "SYSTEM:SUSPEND_LOST_BLOCK:" + "14".zfill(20)
        with self.assertRaisesRegex(ValueError, "outside"):
            evaluate("power-flush-error",
                     power_transcript("power-flush-error") + "\n" + stray, 33, False)
        with self.assertRaisesRegex(ValueError, "outside"):
            evaluate("normal", transcript("normal") + "\n" + stray, 33, False)
        from run_system import COLD_CASES, evaluate_cold
        with self.assertRaisesRegex(ValueError, "outside"):
            evaluate_cold(next(iter(COLD_CASES)), stray, 33, False)

    def test_wrong_machine_or_accelerator_is_refused_before_launch(self):
        from run_system import run_system
        for env in ({"NOS_QEMU_MACHINE": "pc-q35-10.0"},):
            with self.subTest(env=env), unittest.mock.patch.dict(os.environ, env), \
                    self.assertRaisesRegex(ValueError, "power-s3 runs only"):
                run_system("power-s3")
        with unittest.mock.patch.dict(os.environ, {"NOS_QEMU_MACHINE": "pc-q35-8.2"}), \
                unittest.mock.patch("run_system.qemu_accel", return_value="kvm"), \
                self.assertRaisesRegex(ValueError, "power-s3 runs only"):
            run_system("power-s3")

    def test_a_passing_s3_image_of_the_same_source_may_recover(self):
        check_parent(dict(case="power-s3", passed=True, kernel_feature="test-system-s3",
                          source_sha256="source", after_sha256="disk"), "source", "disk", network=False)


S3_REINIT = "KEYRING:S3:REINIT:BARS=RESTORED:STATUS=15:SELFTEST=32"


def reinit_transcript(marker=S3_REINIT, **changes):
    """H05-T5b power-s3-reinit: the re-init marker right after RESUMED."""
    return s3_transcript(s3=(S3_ENTER, S3_RESUMED, *((marker,) if marker else ())),
                         case="power-s3-reinit", **changes)


class PowerS3ReinitHarnessTests(unittest.TestCase):
    def test_passes_with_either_bar_state_and_parses_the_post_wake_evidence(self):
        for state in ("RESTORED", "KEPT"):
            with self.subTest(state=state):
                found = evaluate("power-s3-reinit",
                                 reinit_transcript(S3_REINIT.replace("RESTORED", state)), 0, False)
                self.assertEqual((found["SUSPEND_REINIT_ENTROPY"], found["SUSPEND_REINIT_STALE_TICKET"],
                                  found["SUSPEND_REINIT_ENTROPY_AUTH"], found["SUSPEND_LOST_BLOCK"]),
                                 (32, 1, 6, 14))
        for raw, timed_out in ((33, False), (17, False), (None, True)):
            with self.subTest(raw=raw), self.assertRaises(ValueError):
                evaluate("power-s3-reinit", reinit_transcript(), raw, timed_out)

    def test_missing_duplicate_or_misplaced_reinit_marker_is_refused(self):
        with self.assertRaisesRegex(ValueError, "re-init marker"):
            evaluate("power-s3-reinit", reinit_transcript(marker=None), 0, False)
        doubled = reinit_transcript().replace(S3_REINIT, S3_REINIT + "\n" + S3_REINIT)
        with self.assertRaisesRegex(ValueError, "re-init marker"):
            evaluate("power-s3-reinit", doubled, 0, False)
        late = reinit_transcript().replace(S3_REINIT + "\n", "").replace(
            "SYSTEM:BROKERS:PASS\n", f"SYSTEM:BROKERS:PASS\n{S3_REINIT}\n")
        early = reinit_transcript().replace(S3_REINIT + "\n", "").replace(
            S3_ENTER + "\n", f"{S3_REINIT}\n{S3_ENTER}\n")
        for text in (late, early):
            with self.assertRaisesRegex(ValueError, "order"):
                evaluate("power-s3-reinit", text, 0, False)

    def test_wrong_bar_state_status_or_self_test_token_is_refused(self):
        for old, new in (("RESTORED", "CHANGED"), ("RESTORED", "NONE"), ("RESTORED", "KEPT|RESTORED"),
                         ("STATUS=15", "STATUS=11"), ("STATUS=15", "STATUS=0"),
                         ("SELFTEST=32", "SELFTEST=0"), ("SELFTEST=32", "SELFTEST=31")):
            with self.subTest(new=new), self.assertRaisesRegex(ValueError, "re-init marker"):
                evaluate("power-s3-reinit", reinit_transcript(S3_REINIT.replace(old, new)), 0, False)

    def test_entropy_refused_after_reinit_is_refused(self):
        # No delivery, a short one, the old -14 refusal, a redeemed pre-sleep
        # ticket, or the base profile's LOST_ENTROPY line in place of the delivery.
        for change in (dict(SUSPEND_REINIT_ENTROPY=0), dict(SUSPEND_REINIT_ENTROPY=None),
                       dict(SUSPEND_REINIT_ENTROPY=31), dict(SUSPEND_REINIT_ENTROPY_AUTH=14),
                       dict(SUSPEND_REINIT_ENTROPY_AUTH=None), dict(SUSPEND_REINIT_STALE_TICKET=0),
                       dict(SUSPEND_REINIT_STALE_TICKET=None),
                       dict(SUSPEND_REINIT_ENTROPY=None, SUSPEND_REINIT_STALE_TICKET=None,
                            SUSPEND_REINIT_ENTROPY_AUTH=None, SUSPEND_LOST_ENTROPY=14)):
            with self.subTest(change=change), self.assertRaises(ValueError):
                evaluate("power-s3-reinit", reinit_transcript(**change), 0, False)

    def test_post_wake_delivery_cannot_pad_the_entropy_bound(self):
        self.assertEqual(evaluate("power-s3-reinit", reinit_transcript(ENTROPY_BYTES=288),
                                  0, False)["ENTROPY_BYTES"], 288)
        with self.assertRaisesRegex(ValueError, "before the sleep"):
            evaluate("power-s3-reinit", reinit_transcript(ENTROPY_BYTES=287), 0, False)

    def test_block_served_after_the_wake_is_refused(self):
        for change in (dict(SUSPEND_LOST_BLOCK=0), dict(SUSPEND_LOST_BLOCK=None),
                       dict(SUSPEND_LOST_BLOCK=6)):
            with self.subTest(change=change), self.assertRaises(ValueError):
                evaluate("power-s3-reinit", reinit_transcript(**change), 0, False)

    def test_case_marker_and_profile_boundaries(self):
        plain = reinit_transcript().replace("SYSTEM:CASE:POWER_S3_REINIT:PASS", "SYSTEM:CASE:POWER_S3:PASS")
        with self.assertRaises(ValueError):
            evaluate("power-s3-reinit", plain, 0, False)
        # The base S3 oracle and every other case refuse a re-init marker.
        with self.assertRaisesRegex(ValueError, "outside"):
            evaluate("power-s3", s3_transcript(s3=(S3_ENTER, S3_RESUMED, S3_REINIT)), 0, False)
        with self.assertRaisesRegex(ValueError, "outside"):
            evaluate("power", power_transcript("power") + S3_REINIT + "\n", 0, False)
        check_parent(dict(case="power-s3-reinit", passed=True, kernel_feature="test-system-s3-reinit",
                          source_sha256="source", after_sha256="disk"), "source", "disk", network=False)


S3_BLK_REINIT = "VOLUME:S3:REINIT:BARS=RESTORED:STATUS=15:CAPACITY=00000000000000002048:SELFTEST=READ"


def blk_transcript(case="power-s3-blk-reinit", volume=None, injected=None, **changes):
    """H05-T5c: the rng re-init marker, then the injection label (injected
    cases) and the blk success or quarantine marker, before BROKERS."""
    expected = BLK_CASES[case]
    if volume is None:
        volume = (S3_BLK_REINIT,) if expected is None else (f"VOLUME:S3:REINIT:QUARANTINE:{expected[1]}",)
    if injected is None:
        injected = () if expected is None else (f"SYSTEM:FAULT_INJECTED:S3_BLK_{expected[0]}",)
    return s3_transcript(s3=(S3_ENTER, S3_RESUMED, S3_REINIT, *injected, *volume), case=case, **changes)


class PowerS3BlkReinitHarnessTests(unittest.TestCase):
    """H05-T5c run_system oracle: exact markers, codes and accounting."""

    def test_every_blk_case_passes_only_with_raw_exit_zero(self):
        for case in BLK_CASES:
            with self.subTest(case=case):
                found = evaluate(case, blk_transcript(case), 0, False)
                quarantined = BLK_CASES[case] is not None
                self.assertEqual(found["SUSPEND_LOST_BLOCK"], 24 if quarantined else 6)
                self.assertEqual(found["SUSPEND_BLK_QUARANTINED"], int(quarantined))
                for raw, timed_out in ((33, False), (35, False), (None, True)):
                    with self.assertRaises(ValueError):
                        evaluate(case, blk_transcript(case), raw, timed_out)
        kept = blk_transcript(volume=(S3_BLK_REINIT.replace("RESTORED", "KEPT"),))
        self.assertEqual(evaluate("power-s3-blk-reinit", kept, 0, False)["SUSPEND_BLK_STALE_TICKET"], 1)

    def test_missing_doubled_or_misplaced_blk_markers_are_refused(self):
        for case, (label, reason) in ((c, v) for c, v in BLK_CASES.items() if v):
            quarantine = f"VOLUME:S3:REINIT:QUARANTINE:{reason}"
            inject = f"SYSTEM:FAULT_INJECTED:S3_BLK_{label}"
            for change in (dict(volume=()), dict(volume=(quarantine, quarantine)),
                           dict(injected=()), dict(injected=(inject, inject)),
                           dict(volume=(S3_BLK_REINIT,), injected=()),
                           dict(volume=(quarantine.replace(reason, "OTHER"),))):
                with self.subTest(case=case, change=change), self.assertRaisesRegex(ValueError, "marker"):
                    evaluate(case, blk_transcript(case, **change), 0, False)
            swapped = blk_transcript(case).replace(f"{inject}\n{quarantine}\n", f"{quarantine}\n{inject}\n")
            with self.subTest(case=case, swapped=True), self.assertRaisesRegex(ValueError, "order"):
                evaluate(case, swapped, 0, False)
        for change in (dict(volume=()), dict(volume=(S3_BLK_REINIT, S3_BLK_REINIT)),
                       dict(volume=(S3_BLK_REINIT.replace("SELFTEST=READ", "SELFTEST=NONE"),)),
                       dict(volume=(S3_BLK_REINIT.replace("STATUS=15", "STATUS=11"),)),
                       dict(volume=("VOLUME:S3:REINIT:QUARANTINE:CAPACITY",)),
                       dict(injected=("SYSTEM:FAULT_INJECTED:S3_BLK_BUSY",))):
            with self.subTest(change=change), self.assertRaisesRegex(ValueError, "marker"):
                evaluate("power-s3-blk-reinit", blk_transcript(**change), 0, False)
        late = blk_transcript().replace(S3_BLK_REINIT + "\n", "").replace(
            "SYSTEM:BROKERS:PASS\n", f"SYSTEM:BROKERS:PASS\n{S3_BLK_REINIT}\n")
        before_rng = blk_transcript().replace(f"{S3_REINIT}\n{S3_BLK_REINIT}\n", f"{S3_BLK_REINIT}\n{S3_REINIT}\n")
        for text in (late, before_rng):
            with self.assertRaisesRegex(ValueError, "order"):
                evaluate("power-s3-blk-reinit", text, 0, False)

    def test_codes_and_accounting_are_exact(self):
        for case, change in (("power-s3-blk-reinit", dict(SUSPEND_LOST_BLOCK=14)),
                             ("power-s3-blk-reinit", dict(SUSPEND_LOST_BLOCK=24)),
                             ("power-s3-blk-reinit", dict(SUSPEND_BLK_QUARANTINED=1)),
                             ("power-s3-blk-reinit", dict(SUSPEND_BLK_STALE_TICKET=0)),
                             ("power-s3-blk-reinit", dict(SUSPEND_BLK_PIC_SLAVE=0xfb)),
                             ("power-s3-blk-reinit", dict(SUSPEND_DEVICE_IO_ACCOUNTED_VOLUME=0)),
                             ("power-s3-blk-reinit", dict(SUSPEND_DEVICE_IO_ACCOUNTED_VOLUME=None)),
                             ("power-s3-blk-reinit", dict(SUSPEND_DEVICE_IO_AFTER_WAKE=1)),
                             ("power-s3-blk-reinit", dict(SUSPEND_BLK_INTX_LINE=None)),
                             ("power-s3-blk-reinit", dict(SUSPEND_BLK_LATCH_CARRIED=0)),
                             ("power-s3-blk-reinit", dict(SUSPEND_BLK_LATCH_CARRIED=None)),
                             ("power-s3-blk-reinit-capacity", dict(SUSPEND_BLK_LATCH_CARRIED=0)),
                             ("power-s3-blk-reinit-busy", dict(SUSPEND_BLK_LATCH_CARRIED=None)),
                             ("power-s3-blk-reinit-capacity", dict(SUSPEND_LOST_BLOCK=14)),
                             ("power-s3-blk-reinit-capacity", dict(SUSPEND_LOST_BLOCK=6)),
                             ("power-s3-blk-reinit-capacity", dict(SUSPEND_BLK_QUARANTINED=0)),
                             ("power-s3-blk-reinit-busy", dict(SUSPEND_DEVICE_IO_ACCOUNTED_VOLUME=1)),
                             ("power-s3-blk-reinit-identity", dict(SUSPEND_DEVICE_IO_ACCOUNTED_VOLUME=0))):
            with self.subTest(case=case, change=change), self.assertRaises(ValueError):
                evaluate(case, blk_transcript(case, **change), 0, False)

    def test_blk_evidence_outside_its_profiles_is_refused(self):
        for text in (reinit_transcript() .replace(S3_REINIT + "\n", f"{S3_REINIT}\n{S3_BLK_REINIT}\n"),
                     reinit_transcript(SUSPEND_BLK_QUARANTINED=0)):
            with self.assertRaisesRegex(ValueError, "outside"):
                evaluate("power-s3-reinit", text, 0, False)
        with self.assertRaisesRegex(ValueError, "outside"):
            evaluate("power-s3", s3_transcript() + "SYSTEM:FAULT_INJECTED:S3_BLK_BUSY\n", 0, False)


def held_transcript(**changes):
    """Review R1 power-s3-held-completion: the base S3 cycle plus the held -8."""
    return s3_transcript(case="power-s3-held-completion", **changes)


class PowerS3HeldCompletionHarnessTests(unittest.TestCase):
    def test_passes_only_with_raw_exit_zero_and_exact_held_evidence(self):
        found = evaluate("power-s3-held-completion", held_transcript(), 0, False)
        self.assertEqual((found["SUSPEND_REFUSALS"], found["SUSPEND_HELD_REFUSALS"],
                          found["SUSPEND_DEVICE_IO_AFTER_WAKE"], found["SUSPEND_DEVICE_SUPPRESSED"]),
                         (6, 1, 0, 1))
        for raw, timed_out in ((33, False), (None, True), (37, False)):
            with self.subTest(raw=raw), self.assertRaises(ValueError):
                evaluate("power-s3-held-completion", held_transcript(), raw, timed_out)

    def test_post_wake_device_access_fails(self):
        for case, text in (("power-s3-held-completion", held_transcript(SUSPEND_DEVICE_IO_AFTER_WAKE=1)),
                           ("power-s3", s3_transcript(SUSPEND_DEVICE_IO_AFTER_WAKE=1)),
                           ("power-s3-reinit", reinit_transcript(SUSPEND_DEVICE_IO_AFTER_WAKE=1)),
                           ("power-s3-held-completion", held_transcript(SUSPEND_DEVICE_IO_AFTER_WAKE=None))):
            with self.subTest(case=case), self.assertRaisesRegex(ValueError, "DEVICE_IO_AFTER_WAKE|out of order"):
                evaluate(case, text, 0, False)

    def test_missing_held_refusal_fails(self):
        for change in (dict(SUSPEND_HELD_REFUSALS=0), dict(SUSPEND_HELD_REFUSALS=None),
                       dict(SUSPEND_HELD_REFUSALS=2), dict(SUSPEND_REFUSALS=5),
                       dict(SUSPEND_HELD_REFUSALS=0, SUSPEND_REFUSALS=5)):
            with self.subTest(change=change), self.assertRaises(ValueError):
                evaluate("power-s3-held-completion", held_transcript(**change), 0, False)
        # A held refusal in the base profile is a gate that refused without cause.
        with self.assertRaises(ValueError):
            evaluate("power-s3", s3_transcript(SUSPEND_HELD_REFUSALS=1, SUSPEND_REFUSALS=6), 0, False)

    def test_suppressed_must_equal_held(self):
        for case, text in (("power-s3-held-completion", held_transcript(SUSPEND_DEVICE_SUPPRESSED=0)),
                           ("power-s3-held-completion", held_transcript(SUSPEND_DEVICE_SUPPRESSED=2)),
                           ("power-s3-held-completion", held_transcript(SUSPEND_DEVICE_SUPPRESSED=None)),
                           ("power-s3", s3_transcript(SUSPEND_DEVICE_SUPPRESSED=1)),
                           ("power-s3-reinit", reinit_transcript(SUSPEND_DEVICE_SUPPRESSED=1))):
            with self.subTest(case=case), self.assertRaisesRegex(ValueError, "SUPPRESSED|out of order"):
                evaluate(case, text, 0, False)

    def test_device_evidence_order_and_profile_boundaries(self):
        io = "SYSTEM:SUSPEND_DEVICE_IO_AFTER_WAKE:" + "0".zfill(20)
        sup = "SYSTEM:SUSPEND_DEVICE_SUPPRESSED:" + "1".zfill(20)
        swapped = held_transcript().replace(io, "SWAP").replace(sup, io).replace("SWAP", sup)
        with self.assertRaisesRegex(ValueError, "out of order"):
            evaluate("power-s3-held-completion", swapped, 0, False)
        plain = held_transcript().replace("SYSTEM:CASE:POWER_S3_HELD_COMPLETION:PASS", "SYSTEM:CASE:POWER_S3:PASS")
        with self.assertRaises(ValueError):
            evaluate("power-s3-held-completion", plain, 0, False)
        with self.assertRaisesRegex(ValueError, "outside"):
            evaluate("power", power_transcript("power", SUSPEND_DEVICE_SUPPRESSED=1), 0, False)
        check_parent(dict(case="power-s3-held-completion", passed=True,
                          kernel_feature="test-system-s3-held-completion",
                          source_sha256="source", after_sha256="disk"), "source", "disk", network=False)

    def test_device_evidence_must_follow_cleanup_and_precede_the_case_marker(self):
        # Review follow-up A: the kernel prints the wake evidence before CLEANUP
        # and the device evidence after every teardown, before the case marker.
        device = "".join(line + "\n" for line in held_transcript().splitlines()
                         if line.split(":")[1] in DEVICE)
        cleanup = "SYSTEM:CLEANUP:PASS\n"
        case = "SYSTEM:CASE:POWER_S3_HELD_COMPLETION:PASS\n"
        good = held_transcript()
        self.assertIn(cleanup + device, good)
        lost = next(line for line in good.splitlines() if line.startswith("SYSTEM:SUSPEND_LOST_BLOCK:")) + "\n"
        for text in (good.replace(device, "").replace(cleanup, device + cleanup),  # device lines above CLEANUP
                     good.replace(device, "").replace(case, case + device),  # after the case marker
                     good.replace(lost, "").replace(cleanup, cleanup + lost)):  # wake evidence after CLEANUP
            self.assertNotEqual(text, good)
            with self.subTest(text=text[-600:]), self.assertRaisesRegex(ValueError, "order"):
                evaluate("power-s3-held-completion", text, 0, False)

    def test_device_counters_must_be_live_at_the_wake(self):
        # Review follow-up D: a dead counter (no fetch_add) reads 0 at the wake.
        found = evaluate("power-s3-held-completion", held_transcript(), 0, False)
        self.assertGreater(found["SUSPEND_DEVICE_IO_AT_WAKE_KEYRING"], 0)
        self.assertGreater(found["SUSPEND_DEVICE_IO_AT_WAKE_VOLUME"], 0)
        for key in AT_WAKE:
            for case, make in (("power-s3", s3_transcript), ("power-s3-reinit", reinit_transcript),
                               ("power-s3-held-completion", held_transcript)):
                with self.subTest(case=case, key=key), self.assertRaisesRegex(ValueError, f"{key}: expected a live"):
                    evaluate(case, make(**{key: 0}), 0, False)
                with self.subTest(case=case, key=key, missing=True), self.assertRaisesRegex(ValueError, "order"):
                    evaluate(case, make(**{key: None}), 0, False)


KERNEL = Path(__file__).resolve().parents[1] / "kernel/src/m2"
DRIVERS = Path(__file__).resolve().parents[1] / "drivers"
# Driver entry points that touch neither MMIO nor PCI config space: pool and
# feature checks, used-ring validation, BAR classification, the in-memory
# pending flag. Every other `pub fn` / `pub unsafe fn` is a device call.
PURE = {"validate", "negotiate_rng_features", "validate_used", "classify_bars", "pending",
        "identity_matches", "negotiated_matches", "negotiated"}
DRIVER_FNS = {name for crate in ("virtio-rng", "virtio-blk") for name in re.findall(
    r"^\s*pub (?:unsafe )?fn (\w+)", (DRIVERS / crate / "src/lib.rs").read_text(), re.M)}
DEVICE_METHODS = "|".join(sorted(DRIVER_FNS - PURE))
# Free functions (unindented `pub fn`) can also be imported and called bare.
FREE_FNS = "|".join(sorted({name for crate in ("virtio-rng", "virtio-blk") for name in re.findall(
    r"^pub (?:unsafe )?fn (\w+)", (DRIVERS / crate / "src/lib.rs").read_text(), re.M)} - PURE))
DEVICE_CALL = re.compile(rf"\.(?:{DEVICE_METHODS})\(|::(?:{DEVICE_METHODS})\b|(?<![\w.:])(?:{FREE_FNS})\(")
FUNCTION = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?fn\s+(\w+)")


def functions(path: Path) -> dict[str, list[tuple[int, str]]]:
    """`fn` bodies by name, first definition wins (brace matching on code)."""
    lines = path.read_text().splitlines()
    found = {}
    for index, line in enumerate(lines):
        match = FUNCTION.match(line)
        if not match:
            continue
        depth, started, body = 0, False, []
        for number in range(index, len(lines)):
            code = lines[number].split("//")[0]
            depth += code.count("{") - code.count("}")
            started = started or "{" in code
            body.append((number + 1, lines[number]))
            if started and depth == 0:
                break
        found.setdefault(match.group(1), body)
    return found


def device_calls(body):
    return [(number, line) for number, line in body
            if DEVICE_CALL.search(line.split("//")[0]) and "policy.request(" not in line]


def following(body, marker):
    """The code statements after the first code line containing `marker`
    (comments and blank lines skipped). Attributes are kept, so a `#[cfg]`
    on a guarded statement breaks the expected sequence."""
    code = [line.split("//")[0].strip() for _, line in body]
    code = [line for line in code if line]
    found = [i for i, line in enumerate(code) if marker in line]
    if not found:
        raise AssertionError(f"guard statement missing: {marker}")
    return code[found[0] + 1:]


class PostWakeLatchStructure(unittest.TestCase):
    """STRUCTURAL ONLY (review R1/R3): source shape of the post-wake device
    latch and the access counter of both system brokers. Natively only the
    volume `reset` latch is provoked (power-s3-held-completion); this test keeps
    a future edit from adding an unlatched or uncounted broker device call."""

    # Reachable after the S3 wake in the system profile (broker ticks,
    # submission syscalls, owner teardown): latched and counted.
    REACHABLE = {"keyring.rs": {"call", "advance", "quarantine_pending"},
                 "volume.rs": {"device_call", "reset", "advance_lane"}}
    LATCH = {"keyring.rs": "device_lost()", "volume.rs": "device_lost()"}
    # The validated re-init paths (test-system-s3-reinit, and for volume.rs
    # test-system-s3-blk-reinit, H05-T5c): accounted.
    REINIT = {"keyring.rs": {"reinit_poll", "reinit_fail"},
              "volume.rs": {"blk_reinit_poll", "blk_reinit_quarantine"}}
    # Boot-time setup, and standalone-profile fixtures whose `poll` is compiled
    # only without native-system (checked below).
    OTHER = {"keyring.rs": {"prepare", "restart_fixture", "finish"},
             "volume.rs": {"prepare", "prepare_archive_device", "finish"}}

    def counted(self, name, fn, body, counter):
        previous = body[0][0]
        for number, line in device_calls(body):
            window = [text for at, text in body if previous <= at < number]
            self.assertIn(f"{counter}();", [text.strip() for text in window[-8:]],
                          f"uncounted device call at {name}:{number} in {fn}: {line.strip()}")
            previous = number

    def test_device_call_list_comes_from_the_drivers(self):
        self.assertTrue(PURE <= DRIVER_FNS, PURE - DRIVER_FNS)
        for name in ("kick", "submit", "discover", "discover_function", "rediscover", "initialize",
                     "initialize_profile", "begin_reset", "reset_acknowledged", "poll", "request",
                     "prepare_operation", "publish_prepared", "status"):
            self.assertIn(name, DEVICE_METHODS.split("|"))
        for line in ("d.kick();", "virtio_blk_driver::discover()", "Device::initialize(", "x.submit(",
                     "virtio_blk_driver::discover_function(f)", "lane.device.as_ref().is_some_and(Device::reset_acknowledged)"):
            self.assertTrue(DEVICE_CALL.search(line), line)
        self.assertEqual(set(FREE_FNS.split("|")), {"discover", "discover_function", "rediscover"})
        for line in ("let b = rediscover(&info);", "discover()", "discover_function(f)"):
            self.assertTrue(device_calls([(1, line)]), line)
        for line in ("device.pending()", "pool.validate(end)", "r.policy.request()", "// d.kick();"):
            self.assertFalse(device_calls([(1, line)]), line)

    def test_every_device_call_site_is_classified(self):
        for name in self.REACHABLE:
            bodies = functions(KERNEL / "m3" / name)
            with_calls = {fn for fn, body in bodies.items() if device_calls(body)}
            self.assertEqual(with_calls, self.REACHABLE[name] | self.REINIT[name] | self.OTHER[name], name)

    def test_reachable_calls_follow_the_latch_and_are_counted(self):
        for name, reachable in self.REACHABLE.items():
            bodies = functions(KERNEL / "m3" / name)
            for fn in sorted(reachable):
                body = bodies[fn]
                with self.subTest(file=name, fn=fn):
                    latch = [number for number, line in body if self.LATCH[name] in line.split("//")[0]]
                    self.assertTrue(latch and latch[0] < device_calls(body)[0][0], "latch not before the device call")
                    self.counted(name, fn, body, "device_access")
        # The keyring latch opens only after the validated re-init; the volume
        # latches are compiled into every S3 profile and only there.
        lost = "\n".join(line for _, line in functions(KERNEL / "m3/keyring.rs")["device_lost"])
        self.assertIn('#[cfg(feature = "test-system-s3")]\n    let lost = super::system::resumed() && !reinitialised();',
                      lost)
        lines = (KERNEL / "m3/volume.rs").read_text().splitlines()
        latches = [index for index, line in enumerate(lines) if "if device_lost()" in line]
        self.assertEqual(len(latches), 3)
        for index in latches:
            self.assertEqual(lines[index - 1].strip(), '#[cfg(feature = "test-system-s3")]')
        # H05-T5c: no raw wake flag latches a volume path any more; the volume
        # latch opens only after the validated blk re-init, never globally.
        self.assertEqual([line for line in lines if "system::resumed()" in line.split("//")[0]],
                         ["    super::system::resumed() && !blk_reinitialised()"])
        bodies = functions(KERNEL / "m3/volume.rs")
        start = bodies["device_lost"][0][0]
        self.assertEqual(lines[start - 2].strip(), '#[cfg(feature = "test-system-s3")]')
        text = "\n".join(lines)
        self.assertIn('#[cfg(all(feature = "test-system-s3", not(feature = "test-system-s3-blk-reinit")))]\n'
                      'fn blk_reinitialised() -> bool {\n    false\n}', text)
        self.assertIn('#[cfg(feature = "test-system-s3-blk-reinit")]\nfn blk_reinitialised() -> bool {\n'
                      '    blk_state().stage == BLK_DONE\n}', text)

    def test_each_latch_returns_before_any_device_statement(self):
        """The guarded statement itself, not only the latch text: each latch
        returns, and each teardown binds the device only through its guard."""
        keyring = functions(KERNEL / "m3/keyring.rs")
        volume = functions(KERNEL / "m3/volume.rs")
        self.assertEqual(following(keyring["advance"], "if device_lost() {")[0], "return;")
        self.assertEqual(following(keyring["call"], "if device_lost() {")[0], "return Err(-14);")
        # The keyring latches are compiled into every profile: no attribute may gate them out.
        for fn in ("advance", "call"):
            code = [line.split("//")[0].strip() for _, line in keyring[fn]]
            code = [line for line in code if line]
            at = next(i for i, line in enumerate(code) if "if device_lost() {" in line)
            self.assertFalse(code[at - 1].startswith("#["), f"keyring {fn} latch is attribute-gated: {code[at - 1]}")
        # Every device call of `call` sits inside the 0x60 arm, after its latch.
        call = keyring["call"]
        arm = next(number for number, line in call if line.strip() == "0x60 => {")
        end = next(number for number, line in call if line.strip() == "0x61 => {")
        latch = next(number for number, line in call if "if device_lost() {" in line.split("//")[0])
        self.assertTrue(arm < latch < end)
        for number, line in device_calls(call):
            self.assertTrue(latch < number < end, f"keyring call device statement outside 0x60: {line.strip()}")
        quarantine = keyring["quarantine_pending"]
        bind = "if let Some(device) = r.device.as_mut().filter(|_| touch) {"
        self.assertEqual(following(quarantine, "let r = runtime();")[0], "let touch = !device_lost();")
        self.assertEqual(following(quarantine, bind)[:3], ["device_access();", "device.begin_reset();", "}"])
        self.assertEqual([line.split("//")[0].strip() for _, line in quarantine if "r.device" in line], [bind])
        self.assertEqual(following(volume["reset"], "if device_lost() {")[:2],
                         ["super::system::device_access_suppressed();", "return;"])
        self.assertEqual(following(volume["advance_lane"], "if device_lost() {")[0], "return;")
        self.assertEqual(following(volume["device_call"], "if device_lost() {")[0], "return Err(-14);")
        # Re-init: no MMIO before `rediscover` validated the BARs.
        fail = keyring["reinit_fail"]
        bind = "if let Some(device) = r.device.as_mut().filter(|_| validated) {"
        self.assertEqual(following(fail, "fn reinit_fail")[0], "let validated = reinit_state().bars.is_some();")
        self.assertEqual(following(fail, bind)[:2], ["reinit_access();", "device.begin_reset();"])
        self.assertEqual([line.split("//")[0].strip() for _, line in fail if "r.device" in line], [bind])
        poll = keyring["reinit_poll"]
        stage0 = [(number, line) for number, line in poll
                  if next(n for n, text in poll if text.strip() == "0 => {") <= number
                  < next(n for n, text in poll if text.strip() == "1 => {")]
        bars = next(number for number, line in stage0 if "p.bars = Some(" in line.split("//")[0])
        reset = next(number for number, line in poll if "begin_reset" in line.split("//")[0])
        self.assertLess(bars, reset)

    def test_reinit_calls_are_accounted_and_reached_only_after_the_wake(self):
        source = (KERNEL / "m3/keyring.rs").read_text().splitlines()
        bodies = functions(KERNEL / "m3/keyring.rs")
        for fn in sorted(self.REINIT["keyring.rs"]):
            with self.subTest(fn=fn):
                start = bodies[fn][0][0]
                self.assertEqual(source[start - 2].strip(), '#[cfg(feature = "test-system-s3-reinit")]')
                self.counted("keyring.rs", fn, bodies[fn], "reinit_access")
        callers = {caller for caller, body in bodies.items() if caller != "reinit_fail"
                   and any("reinit_fail(" in line for _, line in body)}
        self.assertEqual(callers, {"reinit_require", "reinit_poll"})
        poll = "\n".join(line for _, line in functions(KERNEL / "m3/system.rs")["poll"])
        self.assertIn("if resumed() {\n        let handle = suspend_state().key_entropy;\n"
                      "        keyring::reinit_poll(", poll)

    @staticmethod
    def check_blk_reinit(volume: str, system: str) -> None:
        """H05-T5c structural contract of the volume re-init (raises on a
        violation): accounted device calls, reached only after the wake and
        the rng re-init, no MMIO before `rediscover`, the same Policy object."""
        bodies = {}
        lines = volume.splitlines()
        for index, line in enumerate(lines):
            match = FUNCTION.match(line)
            if match and match.group(1) not in bodies:
                depth, started, body = 0, False, []
                for number in range(index, len(lines)):
                    code = lines[number].split("//")[0]
                    depth += code.count("{") - code.count("}")
                    started = started or "{" in code
                    body.append((number + 1, lines[number]))
                    if started and depth == 0:
                        break
                bodies[match.group(1)] = body
        for fn in ("blk_reinit_poll", "blk_reinit_quarantine"):
            body = bodies[fn]
            if lines[body[0][0] - 2].strip() != '#[cfg(feature = "test-system-s3-blk-reinit")]':
                raise AssertionError(f"{fn} is not test-system-s3-blk-reinit only")
            previous = body[0][0]
            for number, line in device_calls(body):
                window = [text.strip() for at, text in body if previous <= at < number]
                if "blk_reinit_access();" not in window[-8:]:
                    raise AssertionError(f"unaccounted device call in {fn}: {line.strip()}")
                previous = number
        callers = {fn for fn, body in bodies.items() if fn != "blk_reinit_quarantine"
                   and any("blk_reinit_quarantine(" in line for _, line in body)}
        if callers != {"blk_reinit_poll"}:
            raise AssertionError(f"quarantine reached from {callers}")
        quarantine = [line.split("//")[0].strip() for _, line in bodies["blk_reinit_quarantine"]]
        if quarantine[1] != "let validated = blk_state().bars.is_some();" or \
                "if let Some(device) = lane.device.as_mut().filter(|_| validated) {" not in quarantine:
            raise AssertionError("quarantine may touch the device before rediscover validated it")
        poll = [line.split("//")[0].strip() for _, line in bodies["blk_reinit_poll"]]
        if poll.index("let Ok(bars) = virtio_blk_driver::rediscover(&saved) else {") > \
                next(i for i, line in enumerate(poll) if "begin_reset()" in line):
            raise AssertionError("reset before rediscover")
        block = volume[volume.index("// H05-T5c slice 1"):]
        if "Policy::new" in block or re.search(r"\.policy\s*=[^=]", block):
            raise AssertionError("re-init replaces the lane Policy")
        for term in ("lane.policy.start_reset()", "lane.policy.reset_completed(true)",
                     "lane.policy.reset_completed(false)"):
            if term not in block:
                raise AssertionError(f"missing {term}")
        callers = [line for line in system.splitlines() if "blk_reinit_poll(" in line]
        guard = ('    #[cfg(feature = "test-system-s3-blk-reinit")]\n'
                 '    if resumed() && keyring::reinit_done() && !m5::blk_reinit_settled() {\n'
                 '        m5::blk_reinit_poll(s);\n    }\n')
        if callers != ["        m5::blk_reinit_poll(s);"] or guard not in system:
            raise AssertionError("blk re-init reachable outside the validated wake")

    def test_blk_reinit_is_accounted_latched_and_keeps_the_policy(self):
        volume = (KERNEL / "m3/volume.rs").read_text()
        system = (KERNEL / "m3/system.rs").read_text()
        self.check_blk_reinit(volume, system)
        guard = "if resumed() && keyring::reinit_done() && !m5::blk_reinit_settled() {"
        mutants = {
            # The re-init runs without a validated wake (unlatched re-init).
            "unlatched re-init": (volume, system.replace(guard, "if !m5::blk_reinit_settled() {")),
            "before the rng re-init": (volume, system.replace(guard, "if resumed() && !m5::blk_reinit_settled() {")),
            "uncounted device call": (volume.replace(
                "            blk_reinit_access();\n            if device.submit(", "            if device.submit("), system),
            "quarantine MMIO before rediscover": (volume.replace(
                "lane.device.as_mut().filter(|_| validated)", "lane.device.as_mut()"), system),
            "fresh Policy": (volume.replace(
                "        if lane.policy.start_reset().is_err() {",
                "        lane.policy = Policy::new();\n        if lane.policy.start_reset().is_err() {"), system),
            "global latch": (volume.replace("super::system::resumed() && !blk_reinitialised()",
                                            "super::system::resumed()"), system),
        }
        for name, (v, s) in mutants.items():
            with self.subTest(mutant=name):
                self.assertTrue(v != volume or s != system, name)
                if name == "global latch":
                    # Caught by the exact device_lost body pin above.
                    self.assertNotIn("super::system::resumed() && !blk_reinitialised()", v)
                    continue
                with self.assertRaises(AssertionError):
                    self.check_blk_reinit(v, s)

    def test_standalone_broker_fixtures_never_run_in_native_system(self):
        m3 = (KERNEL / "m3.rs").read_text()
        for call in ("keyring::poll(state)", "m5::poll(state)"):
            before = m3[:m3.index(call)].splitlines()[-2]
            self.assertIn('not(feature = "native-system")', before, call)
        for name in self.OTHER:
            bodies = functions(KERNEL / "m3" / name)
            for fn in ("restart_fixture", "finish"):
                callers = {caller for caller, body in bodies.items()
                           if caller != fn and any(f"{fn}(s)" in line for _, line in body)}
                self.assertTrue(callers <= {"poll"}, (name, fn, callers))

    def test_s3_gate_refuses_a_held_request_before_the_transition(self):
        body = "\n".join(line for _, line in functions(KERNEL / "m3/system.rs")["suspend"])
        order = [body.index(text) for text in ("storage_closed()", "m5::sleep_quiescent()",
                                               "sleep::suspend(", "p.io_at_wake = io;")]
        self.assertEqual(order, sorted(order))
        quiescent = "\n".join(line for _, line in functions(KERNEL / "m3/volume.rs")["sleep_quiescent"])
        # K5: every lane must hold no request; only a lane without a device
        # may stay Offline (it never admitted one).
        self.assertIn("r.lanes.iter().all(|lane| {", quiescent)
        self.assertIn("lane.policy.request().is_none()", quiescent)
        self.assertIn("lane.device.is_none() && lane.policy.phase() == Phase::Offline", quiescent)
        finish = "\n".join(line for _, line in functions(KERNEL / "m3/system.rs")["finish"])
        for term in ("&& keyring_at_wake > 0", "&& volume_at_wake > 0", "&& io_after == 0",
                     "&& p.suppressed == suppressed"):
            self.assertIn(term, finish)


class PowerHarnessTests(unittest.TestCase):
    def test_power_cases_pass_only_with_their_exact_exit(self):
        self.assertEqual(evaluate("power", power_transcript(), 0, False)["POWER_NOT_DURABLE"], 0)
        for case in ("power-flush-error", "power-flush-timeout"):
            self.assertEqual(evaluate(case, power_transcript(case), 33, False)["POWER_NOT_DURABLE"], 1)
        for case, raw in (("power", 33), ("power", None), ("power-flush-error", 0)):
            with self.subTest(case=case, raw=raw), self.assertRaises(ValueError):
                evaluate(case, power_transcript(case), raw, raw is None)

    def test_power_needs_gate_and_exact_s5_tail(self):
        marker = expected_markers("power")[-2]
        for tail in (["SYSTEM:POWER:GATE:PASS", marker, "M1:PASS", S5_TAIL[0]],
                     [marker, "M1:PASS", *S5_TAIL],
                     ["SYSTEM:POWER:GATE:PASS", marker, "M1:PASS", *S5_TAIL, "M1:PASS"]):
            with self.subTest(tail=tail), self.assertRaises(ValueError):
                evaluate("power", power_transcript(tail=tail), 0, False)

    def test_refused_power_must_not_write_pm1_or_skip_refusal(self):
        marker = expected_markers("power-flush-error")[-2]
        for tail in (["SYSTEM:FAULT_INJECTED:FLUSH_ERROR", "SYSTEM:POWER:REFUSED:NOT_DURABLE", marker,
                      "M1:PASS", S5_TAIL[1]],
                     ["SYSTEM:POWER:REFUSED:NOT_DURABLE", marker, "M1:PASS"],
                     ["SYSTEM:FAULT_INJECTED:FLUSH_ERROR", marker, "M1:PASS"]):
            with self.subTest(tail=tail), self.assertRaises(ValueError):
                evaluate("power-flush-error", power_transcript("power-flush-error", tail=tail), 33, False)
        for change in (dict(POWER_INIT_REFUSALS=3), dict(POWER_NOT_DURABLE=0), dict(POWER_APP_REFUSALS=1)):
            with self.subTest(change=change), self.assertRaises(ValueError):
                evaluate("power-flush-error", power_transcript("power-flush-error", **change), 33, False)

    def test_power_counters_and_extra_flush_are_exact(self):
        for change in (dict(POWER_INIT_REFUSALS=4), dict(POWER_APP_REFUSALS=1), dict(DISK_FLUSHES=2)):
            with self.subTest(change=change), self.assertRaises(ValueError):
                evaluate("power", power_transcript(**change), 0, False)

    def test_all_power_cases_require_four_published_flushes(self):
        for case, raw_exit in (("power", 0), ("power-flush-error", 33),
                               ("power-flush-timeout", 33)):
            with self.subTest(case=case), self.assertRaisesRegex(ValueError, "flush"):
                evaluate(case, power_transcript(case, DISK_FLUSHES=3), raw_exit, False)

    def test_only_a_passing_power_image_of_the_same_source_may_recover(self):
        previous = dict(case="power", passed=True, kernel_feature="native-system-power",
                        source_sha256="source", after_sha256="disk")
        check_parent(previous, "source", "disk", network=False)
        check_parent({**previous, "case": "power-flush-timeout",
                      "kernel_feature": "test-system-power-flush-timeout"}, "source", "disk", network=False)
        for change in (dict(kernel_feature="native-system"), dict(passed=False), dict(case="unknown")):
            with self.subTest(change=change), self.assertRaises(ValueError):
                check_parent({**previous, **change}, "source", "disk", network=False)
        with self.assertRaises(ValueError):
            check_parent({**previous, "network_gate": True}, "source", "disk", network=True)


from run_system import (RELAY, RELAY_EXACT, RELAY_FEATURE, RELAY_RED, RELAY_TICKS,  # noqa: E402
                        check_disk_relay, evaluate_red)

RELAY_SUM = 0x1234_5678_9ABC_DEF0


def relay_transcript(case="relay", ticks=None, **changes):
    """Relay transcript: the normal flow with relay persistence evidence and
    the case's relay counters, in the kernel's marker order."""
    values = dict(APP_READY=2, APP_DOCS=1, APP_REVISION=1, APP_BYTES=4096, APP_DENIED=3,
                  APP_CONTENT_SUM=volume_disk.fnv(volume_disk.fixture(0, 1, 4096)),
                  WORKER_SPAWNS=2, WORKER_FAULTS=1, WORKER_RESTARTS=1, WORKER_KILLS=1,
                  LIVE_PROCESSES_AFTER=0, FRAMES_RECOVERED=1, DISK_READS=2352, DISK_WRITES=183,
                  DISK_FLUSHES=12, ENTROPY_BYTES=1024, RECORD_SEAL=1, RECORD_UNSEAL=1,
                  RECORD_CONTEXT_DENIED=10, RECORD_REVOKED_DENIED=2, **PERSIST_EXACT["relay"],
                  **RELAY_EXACT[case])
    values[RELAY + "MAX_COMMIT_TICKS"] = RELAY_TICKS[case].start if ticks is None else ticks
    values[RELAY + "SUM"] = RELAY_SUM
    values.update(changes)
    return "\n".join(expected_markers(case)) + "\n" + "\n".join(
        f"SYSTEM:{key}:{value:020d}" for key, value in values.items()) + "\n"


def relay_oracle() -> dict:
    """Relay disk shape on top of the normal fixture: key-state revisions 3, 2, 1."""
    oracle = volume_disk.analyse(system_image(), OPAQUE_OBJECTS)
    blobs = [KEYSTATE_HEADER + bytes([n]) * 24 + bytes((i * 31 + n) & 0xFF for i in range(64))
             for n in (3, 2, 1)]
    item = oracle["objects"][KEYSTATE_OBJECT]
    template = item["versions"][0]
    item.update(revision=3, versions=[dict(template, revision=n, content_hex=blob.hex())
                                      for n, blob in zip((3, 2, 1), blobs)])
    oracle.update(generation=5, chunks_used=5, clocks=[2, 3])
    return oracle, volume_disk.fnv(blobs[0])


class RelayHarnessTests(unittest.TestCase):
    """ADR-B02-KEY1-RELAY host counterexamples for the relay verifier."""

    def test_good_relay_transcripts(self):
        for case in RELAY_FEATURE:
            with self.subTest(case=case):
                found = evaluate(case, relay_transcript(case), 33, False)
                self.assertEqual(found[RELAY + "OK"], 3)

    def test_each_relay_log_fails_every_other_case(self):
        for actual in RELAY_FEATURE:
            for claimed in RELAY_FEATURE:
                if claimed != actual:
                    with self.subTest(actual=actual, claimed=claimed), self.assertRaises(ValueError):
                        evaluate(claimed, relay_transcript(actual), 33, False)

    def test_relay_markers_outside_the_relay_profile_are_rejected(self):
        extra = f"SYSTEM:{RELAY}CASE:{1:020d}\n"
        with self.assertRaises(ValueError):
            evaluate("normal", transcript() + extra, 33, False)

    def test_every_relay_counter_is_exact(self):
        for case in RELAY_FEATURE:
            for key, value in RELAY_EXACT[case].items():
                for wrong in (value - 1, value + 1):
                    if wrong < 0:
                        continue
                    with self.subTest(case=case, key=key, wrong=wrong), self.assertRaises(ValueError):
                        evaluate(case, relay_transcript(case, **{key: wrong}), 33, False)
            ticks = RELAY_TICKS[case]
            for wrong in (ticks.start - 1, ticks.stop):
                with self.subTest(case=case, ticks=wrong), self.assertRaises(ValueError):
                    evaluate(case, relay_transcript(case, ticks=wrong), 33, False)
        for key in ("CRYPTO_TAMPER_DENIED", "ADMIN_REQUESTS", "KEYSTATE_EXPORTS"):
            with self.subTest(key=key), self.assertRaises(ValueError):
                evaluate("relay", relay_transcript(**{key: PERSIST_EXACT["normal"][key]}), 33, False)
        with self.assertRaises(ValueError):
            evaluate("relay", relay_transcript(**{RELAY + "SUM": 0}), 33, False)
        missing = relay_transcript().replace(f"SYSTEM:{RELAY}SUM:{RELAY_SUM:020d}\n", "")
        with self.assertRaises(ValueError):
            evaluate("relay", missing, 33, False)

    def test_relay_disk_oracle(self):
        good, checksum = relay_oracle()
        check_disk_relay(good, checksum)
        def revision_two(oracle):
            oracle["objects"][KEYSTATE_OBJECT]["revision"] = 2
            oracle["objects"][KEYSTATE_OBJECT]["versions"].pop(0)
        def duplicate_nonce(oracle):
            versions = oracle["objects"][KEYSTATE_OBJECT]["versions"]
            versions[1]["content_hex"] = versions[0]["content_hex"]
        for name, mutate, value in (
                ("revision 2", revision_two, checksum),
                ("missing version", lambda o: o["objects"][KEYSTATE_OBJECT]["versions"].pop(), checksum),
                ("sum mismatch", lambda o: None, checksum ^ 1),
                ("duplicate nonce", duplicate_nonce, checksum),
                ("wrong clocks", lambda o: o.update(clocks=[2, 4]), checksum),
                ("wrong generation", lambda o: o.update(generation=6), checksum),
                ("other object", lambda o: o["objects"][9].update(present=True), checksum),
                ("wrong header", lambda o: o["objects"][KEYSTATE_OBJECT]["versions"][2].update(
                    content_hex=(b"NOSKEY02" + bytes(112)).hex()), checksum)):
            bad = copy.deepcopy(good)
            mutate(bad)
            with self.subTest(name), self.assertRaises(ValueError):
                check_disk_relay(bad, value)

    def test_red_manifest_case(self):
        good = "\n".join(BOOT_LINES) + "\nSYSTEM:FAIL:APP_COMMIT_HANDLE\n"
        self.assertEqual(evaluate_red("relay-leak", good, 35, False, "a", "a")["raw_exit"], 35)
        for text, raw, before, after in (
                (good + "SYSTEM:MANIFEST:PASS\n", 35, "a", "a"),
                (good + "SYSTEM:BROKERS:PASS\n", 35, "a", "a"),
                (good + "SYSTEM:CLEANUP:PASS\n", 35, "a", "a"),
                (good + "SYSTEM:CASE:RELAY_LEAK:PASS\n", 35, "a", "a"),
                (good + "M1:PASS\n", 35, "a", "a"),
                (good.replace("APP_COMMIT_HANDLE", "APP_HANDLE_COUNT"), 35, "a", "a"),
                (good + f"SYSTEM:{RELAY}CASE:{1:020d}\n", 35, "a", "a"),
                (good, 33, "a", "a"),
                (good, 35, "a", "b"),
                (good.replace("BOOT:EBS_OK\n", ""), 35, "a", "a")):
            with self.subTest(text=text[-40:], raw=raw, after=after), self.assertRaises(ValueError):
                evaluate_red("relay-leak", text, raw, False, before, after)
        with self.assertRaises(ValueError):
            evaluate_red("relay-leak", good, 35, True, "a", "a")
        self.assertEqual(tuple(RELAY_RED), ("relay-leak",))


BOOT_LINES = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK")



# B01-LOCK: the values of the first passing native power-lock run.
LOCK_VALUES = dict(LOCK_COUNT=1, LOCK_REFUSALS=6, LOCK_STACK_SCRUBS=1, LOCK_STACK_RESIDUE=0,
                   LOCK_CANARY_BEFORE=6, LOCK_CANARY_AFTER=0, LOCK_ENTROPY_OBJECTS_BEFORE=1,
                   LOCK_ENTROPY_OBJECTS_AFTER=0, LOCK_FRAMES_ZEROED=65, LOCK_GRANT_HOLDS=724)


def lock_transcript(locked=("SYSTEM:LOCK:LOCKED",), admin=(6, 4), **changes):
    """power-lock in the kernel's order: LOCKED during the run, the LOCK
    evidence after BROKERS, then CLEANUP and the S5 tail."""
    values = {key: value for key, value in {**LOCK_VALUES, **changes}.items() if value is not None}
    evidence = "".join(f"SYSTEM:{key}:{value:020d}\n" for key, value in values.items())
    text = power_transcript("power-lock", ADMIN_REQUESTS=admin[0], ADMIN_DENIED=admin[1])
    text = text.replace("SYSTEM:MANIFEST:PASS\n", "SYSTEM:MANIFEST:PASS\n" + "".join(
        f"{line}\n" for line in locked), 1)
    return text.replace("SYSTEM:BROKERS:PASS\n", f"SYSTEM:BROKERS:PASS\n{evidence}SYSTEM:LOCK:PASS\n", 1)


def lock_red_transcript(reason, locked=1, extra=()):
    return "\n".join((*BOOT, "SYSTEM:MANIFEST:PASS", *["SYSTEM:LOCK:LOCKED"] * locked, *extra,
                      f"SYSTEM:FAIL:{reason}")) + "\n"


class PowerLockHarnessTests(unittest.TestCase):
    def test_power_lock_passes_with_exact_evidence(self):
        found = evaluate("power-lock", lock_transcript(), 0, False)
        self.assertEqual((found["LOCK_COUNT"], found["LOCK_CANARY_AFTER"], found["ADMIN_REQUESTS"]), (1, 0, 6))

    def test_power_lock_refuses_weaker_evidence(self):
        for change in (dict(LOCK_CANARY_AFTER=1), dict(LOCK_CANARY_BEFORE=0), dict(LOCK_STACK_SCRUBS=0),
                       dict(LOCK_STACK_RESIDUE=8), dict(LOCK_REFUSALS=5), dict(LOCK_COUNT=2),
                       dict(LOCK_ENTROPY_OBJECTS_AFTER=1), dict(LOCK_FRAMES_ZEROED=None),
                       dict(LOCK_GRANT_HOLDS=0)):
            with self.subTest(change=change), self.assertRaises(ValueError):
                evaluate("power-lock", lock_transcript(**change), 0, False)

    def test_power_lock_needs_one_locked_marker_in_order(self):
        for locked in ((), ("SYSTEM:LOCK:LOCKED",) * 2):
            with self.subTest(locked=locked), self.assertRaises(ValueError):
                evaluate("power-lock", lock_transcript(locked=locked), 0, False)
        late = lock_transcript(locked=()).replace("SYSTEM:BROKERS:PASS\n",
                                                  "SYSTEM:BROKERS:PASS\nSYSTEM:LOCK:LOCKED\n", 1)
        with self.assertRaises(ValueError):
            evaluate("power-lock", late, 0, False)

    def test_power_lock_pins_the_extra_refused_admin_request(self):
        for admin in ((5, 3), (6, 3), (7, 4)):
            with self.subTest(admin=admin), self.assertRaises(ValueError):
                evaluate("power-lock", lock_transcript(admin=admin), 0, False)

    def test_lock_evidence_outside_power_lock_is_refused(self):
        text = power_transcript().replace("SYSTEM:MANIFEST:PASS\n", "SYSTEM:MANIFEST:PASS\nSYSTEM:LOCK:LOCKED\n", 1)
        with self.assertRaises(ValueError):
            evaluate("power", text, 0, False)

    def test_red_lock_cases_need_their_exact_refusal(self):
        noscrub = evaluate_lock_red("power-lock-noscrub", lock_red_transcript("LOCK_CANARY_RESIDUE"), 35, False)
        self.assertEqual(noscrub["fail"], "SYSTEM:FAIL:LOCK_CANARY_RESIDUE")
        own_exit = evaluate_lock_red("power-lock-exit", lock_red_transcript("UNEXPECTED_EXIT", locked=0), 35, False)
        self.assertEqual(own_exit["fail"], "SYSTEM:FAIL:UNEXPECTED_EXIT")
        for case, text, raw in (
                ("power-lock-noscrub", lock_red_transcript("LOCK_CANARY_ABSENT"), 35),
                ("power-lock-noscrub", lock_red_transcript("LOCK_CANARY_RESIDUE", locked=0), 35),
                ("power-lock-noscrub", lock_red_transcript("LOCK_CANARY_RESIDUE"), 33),
                ("power-lock-noscrub", lock_red_transcript("LOCK_CANARY_RESIDUE",
                                                           extra=("SYSTEM:LOCK_COUNT:" + "0" * 19 + "1",)), 35),
                ("power-lock-exit", lock_red_transcript("UNEXPECTED_EXIT"), 35),
                ("power-lock-exit", lock_red_transcript("UNEXPECTED_EXIT", locked=0, extra=("M1:PASS",)), 35)):
            with self.subTest(case=case, text=text, raw=raw), self.assertRaises(ValueError):
                evaluate_lock_red(case, text, raw, False)

if __name__ == "__main__":
    unittest.main()

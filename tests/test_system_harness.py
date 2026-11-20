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
                        S5_TAIL, COLD_CASES, cold_image, evaluate_cold, evaluate, expected_markers, record_plaintext,
                        S3_FEATURE, SLEEP_TAIL)
import run_sleep
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


def power_transcript(case="power", tail=None, **changes):
    """H05 power run: the normal flow plus refusal counters, then the S5 tail."""
    values = dict(POWER_INIT_REFUSALS=3 if case == "power" else 4, POWER_APP_REFUSALS=2,
                  POWER_NOT_DURABLE=0 if case == "power" else 1, DISK_FLUSHES=4)
    values.update(changes)
    body = transcript("normal", **values).replace("SYSTEM:CASE:NORMAL:PASS\nM1:PASS\n", "")
    marker = expected_markers(case)[-2]
    if case == "power":
        end = ["SYSTEM:POWER:GATE:PASS", marker, "M1:PASS", *S5_TAIL] if tail is None else tail
    else:
        fault = "FLUSH_ERROR" if case == "power-flush-error" else "FLUSH_TIMEOUT"
        end = [f"SYSTEM:FAULT_INJECTED:{fault}", "SYSTEM:POWER:REFUSED:NOT_DURABLE", marker,
               "M1:PASS"] if tail is None else tail
    return body + "\n".join(end) + "\n"


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


S3_PLAN = ("POWER:S3:PLAN:PM1A=0x0604:SLP_TYP=1:PM1A_STS=0x0600:FACS_VERSION=0:"
           "REAL=0x1000:PM32=0x2000510:POLICY=Edk2Version0Extended")
S3_RESUMED = ("POWER:S3:RESUMED:CYCLE=1:ENTRY=PM32:PM1_STS=0x8101:WAK_STS=1:SCI_EN=RESTORED:"
              "HWSIG=SAME:FACS=DISARMED:CANARY=PASS:CR3=SAME")
S3_LINES = [S3_PLAN, run_sleep.ENTER.format(1), S3_RESUMED, *SLEEP_TAIL]
S3_EVENTS = {"s3": ["RTC_CHANGE", "SUSPEND", "WAKEUP", "SHUTDOWN"],
             "s3-stale-token": ["RTC_CHANGE", "SUSPEND", "WAKEUP"],
             "s3-not-durable": ["RTC_CHANGE"]}
S3_EXIT = {"s3": 0, "s3-stale-token": 33, "s3-not-durable": 33}


def s3_transcript(case="s3", sleep=None, tail=None, **changes):
    """H05-T4 run: the power flow with init's S3 cycle inside the broker phase."""
    if case == "s3-stale-token":
        lines = [*expected_markers(case), S3_PLAN, run_sleep.ENTER.format(1), run_sleep.STALE]
        return "\n".join(lines if sleep is None else [*expected_markers(case), *sleep]) + "\n"
    durable = case == "s3"
    values = dict(SLEEP_INIT_REFUSALS=3 if durable else 4, SLEEP_APP_REFUSALS=2,
                  SLEEP_GENERATION=int(durable), SLEEP_TIMER_IRQS=2 if durable else 0,
                  SLEEP_DEVICE_IO_AFTER_WAKE=0, SLEEP_DEVICE_REFUSALS=0)
    values.update(changes)
    marker = expected_markers(case)[-2]
    if tail is None:
        tail = (["SYSTEM:POWER:GATE:PASS", marker, "M1:PASS", *S5_TAIL] if durable else
                ["SYSTEM:FAULT_INJECTED:FLUSH_ERROR", "SYSTEM:POWER:REFUSED:NOT_DURABLE", marker, "M1:PASS"])
    text = power_transcript("power" if durable else "power-flush-error", tail=tail, **values)
    lines = (S3_LINES if durable else []) if sleep is None else sleep
    return text.replace("SYSTEM:MANIFEST:PASS\n", "\n".join(["SYSTEM:MANIFEST:PASS", *lines]) + "\n", 1)


class SleepHarnessTests(unittest.TestCase):
    def test_s3_cases_pass_only_with_their_exit_and_qmp_events(self):
        for case in S3_FEATURE:
            with self.subTest(case=case):
                found = evaluate(case, s3_transcript(case), S3_EXIT[case], False, S3_EVENTS[case])
                self.assertEqual(found["sleep"]["qmp_sleep_events"],
                                 ["SUSPEND", "WAKEUP"] if case != "s3-not-durable" else [])
        bad = [("s3", 33, S3_EVENTS["s3"]), ("s3", 0, None), ("s3", 0, ["SUSPEND", "WAKEUP"]),
               ("s3", 0, ["SUSPEND", "WAKEUP", "SHUTDOWN", "SHUTDOWN"]),
               ("s3", 0, ["SHUTDOWN", "SUSPEND", "WAKEUP"]),
               ("s3", 0, ["SUSPEND", "WAKEUP", "SUSPEND", "WAKEUP", "SHUTDOWN"]),
               ("s3-not-durable", 33, ["SUSPEND", "WAKEUP"]), ("s3-not-durable", 0, ["RTC_CHANGE"]),
               ("s3-not-durable", 33, ["SHUTDOWN"]), ("s3-stale-token", 0, S3_EVENTS["s3-stale-token"]),
               ("s3-stale-token", 33, ["SUSPEND"]), ("s3-stale-token", 33, S3_EVENTS["s3"])]
        for case, raw, events in bad:
            with self.subTest(case=case, raw=raw, events=events), self.assertRaises(ValueError):
                evaluate(case, s3_transcript(case), raw, False, events)

    def test_s3_sequence_is_exact_and_inside_the_broker_phase(self):
        swapped = [S3_PLAN, run_sleep.ENTER.format(1), S3_RESUMED, *reversed(SLEEP_TAIL)]
        gap = [S3_PLAN, run_sleep.ENTER.format(1), "SYSTEM:SLEEP:TIMER:WAIT", S3_RESUMED, SLEEP_TAIL[1]]
        # Any line between ENTER and the wake report breaks "directly followed".
        noise = [S3_PLAN, run_sleep.ENTER.format(1), "KEYRING:NOISE", S3_RESUMED, *SLEEP_TAIL]
        variants = [[], S3_LINES[1:], S3_LINES[:-1], swapped, gap, noise, S3_LINES + [run_sleep.ENTER.format(2)],
                    [S3_LINES[0], run_sleep.ENTER.format(1), S3_RESUMED.replace("CYCLE=1", "CYCLE=2"),
                     *SLEEP_TAIL],
                    [S3_LINES[0], run_sleep.ENTER.format(1), S3_RESUMED.replace("0x8101", "0x0101"),
                     *SLEEP_TAIL],
                    [S3_LINES[0], run_sleep.ENTER.format(1), S3_RESUMED.replace("ENTRY=PM32", "ENTRY=UNKNOWN"),
                     *SLEEP_TAIL],
                    [S3_PLAN.replace("REAL=0x1000", "REAL=0xa0000"), *S3_LINES[1:]]]
        for sleep in variants:
            with self.subTest(sleep=sleep), self.assertRaises(ValueError):
                evaluate("s3", s3_transcript(sleep=sleep), 0, False, S3_EVENTS["s3"])
        text = s3_transcript(sleep=[])
        moved = text.replace("SYSTEM:BROKERS:PASS\n", "SYSTEM:BROKERS:PASS\n" + "\n".join(S3_LINES) + "\n")
        early = text.replace("SYSTEM:MANIFEST:PASS\n", "\n".join(S3_LINES) + "\nSYSTEM:MANIFEST:PASS\n")
        for stdout in (moved, early):
            with self.subTest(stdout=stdout[:40]), self.assertRaises(ValueError):
                evaluate("s3", stdout, 0, False, S3_EVENTS["s3"])

    def test_kernel_sleep_counters_are_exact(self):
        for change in (dict(SLEEP_INIT_REFUSALS=4), dict(SLEEP_APP_REFUSALS=1), dict(SLEEP_GENERATION=2),
                       dict(SLEEP_TIMER_IRQS=1), dict(SLEEP_DEVICE_IO_AFTER_WAKE=1),
                       dict(SLEEP_DEVICE_REFUSALS=1), dict(POWER_INIT_REFUSALS=4), dict(DISK_FLUSHES=3)):
            with self.subTest(change=change), self.assertRaises(ValueError):
                evaluate("s3", s3_transcript(**change), 0, False, S3_EVENTS["s3"])
        for change in (dict(SLEEP_INIT_REFUSALS=3), dict(SLEEP_GENERATION=1), dict(SLEEP_TIMER_IRQS=2),
                       dict(POWER_NOT_DURABLE=0)):
            with self.subTest(change=change), self.assertRaises(ValueError):
                evaluate("s3-not-durable", s3_transcript("s3-not-durable", **change), 33, False,
                         S3_EVENTS["s3-not-durable"])
        text = s3_transcript()
        missing = "\n".join(line for line in text.splitlines() if "SLEEP_TIMER_IRQS" not in line) + "\n"
        with self.assertRaises(ValueError):
            evaluate("s3", missing, 0, False, S3_EVENTS["s3"])

    def test_refused_sleep_never_reaches_the_s3_path(self):
        with self.assertRaises(ValueError):
            evaluate("s3-not-durable", s3_transcript("s3-not-durable", sleep=S3_LINES), 33, False,
                     S3_EVENTS["s3-not-durable"])
        with self.assertRaises(ValueError):
            evaluate("s3-not-durable", s3_transcript("s3-not-durable", sleep=[S3_PLAN]), 33, False,
                     S3_EVENTS["s3-not-durable"])

    def test_stale_token_must_end_at_the_rejected_wake(self):
        stale = [S3_PLAN, run_sleep.ENTER.format(1), run_sleep.STALE]
        for sleep in ([S3_PLAN, run_sleep.ENTER.format(1), S3_RESUMED],
                      [S3_PLAN, run_sleep.ENTER.format(1), run_sleep.STALE, "SYSTEM:BROKERS:PASS"],
                      [S3_PLAN, run_sleep.ENTER.format(1), "SYSTEM:SLEEP:TIMER:WAIT", run_sleep.STALE],
                      [*stale, "SYSTEM:DISK_READS:00000000000000000001"],
                      [run_sleep.ENTER.format(1), run_sleep.STALE]):
            with self.subTest(sleep=sleep), self.assertRaises(ValueError):
                evaluate("s3-stale-token", s3_transcript("s3-stale-token", sleep=sleep), 33, False,
                         S3_EVENTS["s3-stale-token"])
        self.assertEqual(expected_markers("s3-stale-token")[-1], "SYSTEM:MANIFEST:PASS")

    def test_profiles_without_s3_reject_sleep_evidence_and_events(self):
        with self.assertRaises(ValueError):
            evaluate("power", power_transcript(SLEEP_GENERATION=0), 0, False)
        with self.assertRaises(ValueError):
            evaluate("power", power_transcript(), 0, False, ["SUSPEND", "WAKEUP"])
        with self.assertRaises(ValueError):
            evaluate("normal", transcript(), 33, False, [])

    def test_only_a_passing_s3_image_of_the_same_source_may_recover(self):
        for case, feature in S3_FEATURE.items():
            previous = dict(case=case, passed=True, kernel_feature=feature,
                            source_sha256="source", after_sha256="disk")
            with self.subTest(case=case):
                check_parent(previous, "source", "disk", network=False)
                for change in (dict(kernel_feature="native-system-power"), dict(passed=False),
                               dict(after_sha256="other")):
                    with self.assertRaises(ValueError):
                        check_parent({**previous, **change}, "source", "disk", network=False)


if __name__ == "__main__":
    unittest.main()

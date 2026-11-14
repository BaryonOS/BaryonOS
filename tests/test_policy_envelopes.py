"""Host-only refusal tests for the B03-P2 oracle; synthetic text is not native evidence."""
from pathlib import Path
import importlib.util
import struct
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import build as update_build  # noqa: E402
import update_disk as disk  # noqa: E402

spec = importlib.util.spec_from_file_location("policy_envelopes_gate", ROOT / "tools/test_policy_envelopes.py")
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


def transcript() -> str:
    lines = [
        "BOOT:POLICY_PROBE:COUNT:26", "BOOT:POLICY_PROBE:MASK:67108863", "BOOT:POLICY_PROBE:PASS",
        "BOOT:FLOOR:0", "BOOT:SLOT:B:REJECTED:FORMAT", "BOOT:SLOT:A:VERSION:1",
        "BOOT:SIGNED:OK", "BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK",
        "KERNEL:IMAGE_VERSION:00000000000000000001", "UPDATE:MANIFEST:PASS",
        "UPDATE:POLICY_PROBE:COUNT:00000000000000000026",
        "UPDATE:POLICY_PROBE:MASK:00000000000067108863", "UPDATE:POLICY_PROBE:PASS", "UPDATE:READY",
    ]
    for key, value in probe.EXPECTED_NUMBERS.items():
        if key not in ("POLICY_PROBE:COUNT", "POLICY_PROBE:MASK", "IMAGE_VERSION"):
            lines.append(f"UPDATE:{key}:{value:020d}")
    lines.extend(["UPDATE:TICKS:00000000000000000123", "UPDATE:NO_BUNDLE",
                  "UPDATE:RESOURCES:PASS", "UPDATE:PASS", "M1:PASS"])
    return "\n".join(lines) + "\n"


def evaluate(text: str, **kwargs):
    status = dict(rc=33, timed_out=False, killed=False, running=False)
    status.update(kwargs)
    return probe.evaluate(text, **status)


def fixture_store():
    manifest = (probe.FIXTURES / "manifest.bin").read_bytes()
    image = (probe.FIXTURES / "payload.bin").read_bytes()
    store = bytearray(disk.STORE_SIZE)
    store[:8] = disk.STORE_MAGIC
    struct.pack_into("<IIQ", store, 8, 1, 0, 1)
    struct.pack_into("<9I", store, 24, *disk.TABLE)
    start = disk.SLOT_A_MANIFEST * disk.SECTOR
    store[start:start + len(manifest)] = manifest
    start = disk.SLOT_A_IMAGE * disk.SECTOR
    store[start:start + len(image)] = image
    return bytes(store), manifest, image


class ProbeOracle(unittest.TestCase):
    def test_complete_reports_and_native_handover_are_required(self):
        self.assertEqual(len(probe.CASE_NAMES), 26)
        self.assertEqual(probe.EXPECTED_MASK, (1 << 26) - 1)
        self.assertEqual(evaluate(transcript())["RESULT"], 4)
        for marker in (*probe.LOADER_MARKERS, *probe.SERVICE_MARKERS,
                       "KERNEL:ENTRY", "UPDATE:RESOURCES:PASS", "M1:PASS"):
            with self.subTest(marker=marker), self.assertRaises(ValueError):
                evaluate(transcript().replace(marker + "\n", ""))

    def test_forged_duplicate_and_partial_probe_markers_fail(self):
        for marker in (*probe.LOADER_MARKERS, *probe.SERVICE_MARKERS):
            for replacement in ("prefix:" + marker, marker + ":extra", marker + "\n" + marker):
                with self.subTest(marker=marker, replacement=replacement), self.assertRaises(ValueError):
                    evaluate(transcript().replace(marker, replacement))
        for text in ("\n".join(probe.LOADER_MARKERS), "\n".join(probe.SERVICE_MARKERS),
                     "BOOT:POLICY_PROBE:PASS\nUPDATE:POLICY_PROBE:PASS\nM1:PASS\n"):
            with self.assertRaises(ValueError):
                evaluate(text)

    def test_each_consumer_needs_every_case_not_only_a_pass_label(self):
        for prefix in ("BOOT", "UPDATE"):
            for label, value in (("COUNT", 25), ("MASK", 67_108_862)):
                lines = transcript().splitlines()
                needle = f"{prefix}:POLICY_PROBE:{label}:"
                lines = [f"{needle}{value:020d}" if line.startswith(needle) else line for line in lines]
                with self.subTest(prefix=prefix, label=label), self.assertRaises(ValueError):
                    evaluate("\n".join(lines))

    def test_reports_cannot_be_moved_across_handover(self):
        text = transcript().replace("UPDATE:MANIFEST:PASS\n", "")
        text = text.replace("UPDATE:READY", "UPDATE:MANIFEST:PASS\nUPDATE:READY")
        with self.assertRaises(ValueError):
            evaluate(text)
        for moved in ((probe.SERVICE_MARKERS[0],), (probe.SERVICE_MARKERS[1],),
                      probe.SERVICE_MARKERS[:2]):
            for boundary in ("BOOT:FLOOR:0", "KERNEL:ENTRY", "UPDATE:MANIFEST:PASS"):
                text = transcript()
                for marker in moved:
                    text = text.replace(marker + "\n", "")
                text = text.replace(boundary, "\n".join(moved) + "\n" + boundary)
                with self.subTest(moved=moved, boundary=boundary), self.assertRaises(ValueError):
                    evaluate(text)

    def test_timeout_kill_failure_and_wrong_exit_are_rejected(self):
        for override in (dict(rc=0), dict(rc=35), dict(timed_out=True), dict(killed=True), dict(running=True)):
            with self.subTest(override=override), self.assertRaises(ValueError):
                evaluate(transcript(), **override)
        with self.assertRaises(ValueError):
            evaluate(transcript() + "PANIC\n")

    def test_missing_or_nonzero_write_observations_are_rejected(self):
        for key in ("SERVICE_WRITES", "SERVICE_FLUSHES", "BROKER:WRITES", "BROKER:FLUSHES"):
            marker = f"UPDATE:{key}:{0:020d}\n"
            for replacement in ("", f"UPDATE:{key}:{1:020d}\n", marker + marker):
                with self.subTest(key=key), self.assertRaises(ValueError):
                    evaluate(transcript().replace(marker, replacement))

    def test_nonempty_inbox_and_byte_changes_are_rejected_independently(self):
        store, manifest, image = fixture_store()
        probe.check_store(store, store, manifest, image)
        changed = bytearray(store)
        changed[disk.INBOX_IMAGE * disk.SECTOR] = 1
        with self.assertRaisesRegex(ValueError, "changed"):
            probe.check_store(store, bytes(changed), manifest, image)
        with self.assertRaisesRegex(ValueError, "empty"):
            probe.check_store(bytes(changed), bytes(changed), manifest, image)

    def test_exact_explicit_profiles_are_required(self):
        valid = {"loader_feature": "signed-policy-probe-test", "kernel_feature": "native-update-policy-probe",
                 "source_sha256": "frozen", "update": {"image_version": 1}}
        probe.validate_build(valid, "frozen")
        for key, value in (("loader_feature", "signed-boot-test"), ("kernel_feature", "native-update"),
                           ("source_sha256", "stale"), ("update", {"image_version": 2})):
            with self.subTest(key=key), self.assertRaises(ValueError):
                probe.validate_build(dict(valid, **{key: value}), "frozen")
        for loader, kernel in (("signed-policy-probe-test", "native-update"),
                               ("signed-boot-test", "native-update-policy-probe")):
            with self.assertRaises(ValueError):
                update_build.build(loader, kernel, image_version=1)


if __name__ == "__main__":
    unittest.main()

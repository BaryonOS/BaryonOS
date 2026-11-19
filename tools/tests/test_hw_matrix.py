"""Synthetic matrix fixtures; no hardware execution or acceptance."""
import copy
import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
sys.path.insert(0, str(Path(__file__).resolve().parent))
import hw_evidence
import hw_matrix
from test_hw_evidence import NO_SSH, SSH_KEYGEN, TARGET, Fixture

VM_TARGET = "pc-q35-8.2/OVMF"


class MatrixCase(unittest.TestCase):
    """Temporary repository with the committed matrix, synthetic drivers and copied evidence."""

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / "repo"
        self.root.mkdir()
        self.matrix = json.loads((hw_matrix.ROOT / hw_matrix.MATRIX).read_text())
        (self.root / hw_matrix.MATRIX).parent.mkdir(parents=True)
        for profile in self.matrix["profiles"]:
            for device in profile["devices"]:
                if device["driver_path"] is not None:
                    path = self.root / device["driver_path"]
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_text("synthetic source")
                for name in device["evidence"]:
                    path = self.root / name
                    path.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(hw_matrix.ROOT / name, path)

    def check(self, matrix=None):
        (self.root / hw_matrix.MATRIX).write_text(json.dumps(self.matrix if matrix is None else matrix))
        return hw_matrix.check(self.root)

    def row(self, profile_id, device_id, matrix=None):
        profile = next(p for p in (matrix or self.matrix)["profiles"] if p["id"] == profile_id)
        return profile, next(d for d in profile["devices"] if d["id"] == device_id)

    def rejects(self, reason, matrix=None):
        with self.assertRaisesRegex(ValueError, reason):
            self.check(matrix)

    def file(self, name, text="evidence"):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        return name

    def record(self, **changes):
        """Legacy unsigned schema-1 style record; no longer sufficient for physical support."""
        record = dict(passed=True, level="H", target="explicit-test-target", device="gop", source_sha256="a" * 64)
        record.update(changes)
        return self.file("docs/verification/hw/result.json", json.dumps(record))


class HardwareMatrixTests(MatrixCase):
    def test_current_inventory_and_missing_fields(self):
        summary = self.check()
        self.assertEqual((summary["schema"], summary["profiles"], summary["supported"]), (2, 3, 0))
        self.assertEqual(summary["owner_decisions_open"], len(hw_matrix.OWNER_DECISIONS))
        self.assertIsNone(self.row("HW-FR", "gop")[0]["target"])
        for mutate in (lambda d: d["profiles"][0]["devices"][0].pop("dma_posture"),
                       lambda d: d["profiles"][0]["devices"][0].pop("evidence_level"),
                       lambda d: d["profiles"][0]["devices"][0].update(extra=1),
                       lambda d: d.pop("owner_decisions"),
                       lambda d: d["profiles"].pop(),
                       lambda d: d["profiles"][0]["devices"].pop(2),
                       lambda d: d["profiles"][0].update(kind="hardware"),
                       lambda d: d["profiles"][0]["devices"][2].update(status="supported")):
            matrix = copy.deepcopy(self.matrix)
            mutate(matrix)
            with self.assertRaises(ValueError):
                self.check(matrix)

    def test_dangling_and_escaping_references(self):
        device = self.matrix["profiles"][0]["devices"][0]
        device["evidence_level"] = "native-qemu"
        for name in ("docs/missing", "../outside", "/etc/passwd", "docs//bad", "C:\\bad"):
            device["evidence"] = [name]
            with self.assertRaises((ValueError, OSError)):
                self.check()
        device.update(evidence=[], evidence_level="none")
        device["driver_path"] = "missing/driver.rs"
        with self.assertRaises(OSError):
            self.check()

    def test_virtual_profiles_never_reach_supported(self):
        for profile_id, device_id in (("VM-FR", "acpi"), ("VM-FR", "gop"), ("VM-VMW", "gop")):
            matrix = copy.deepcopy(self.matrix)
            self.row(profile_id, device_id, matrix)[1]["status"] = "supported"
            with self.subTest(profile=profile_id, device=device_id):
                self.rejects("never supported", matrix)

    def test_native_verified_needs_target_bound_virtual_evidence(self):
        _, gop = self.row("VM-FR", "gop")
        gop["status"] = "native-verified"
        self.rejects("native-verified lacks")
        gop.update(evidence_level="native-qemu", evidence=[self.file("docs/verification/g0/run.json",
                                                                     "machine pc-q35-10.0")])
        self.rejects("native-verified lacks")
        gop["evidence"] = [self.file("docs/notes/run.json", VM_TARGET)]
        self.rejects("docs/verification")
        gop["evidence"] = [self.file("docs/verification/g0/run.json", "machine pc-q35-8.2, OVMF")]
        self.assertEqual(self.check()["native_verified"], 3)
        _, vmw = self.row("VM-VMW", "gop")
        vmw.update(status="native-verified", evidence_level="native-qemu", evidence=gop["evidence"])
        self.rejects("cannot apply to a vmware profile")
        vmw.update(evidence_level="vmware", evidence=[self.file("docs/verification/vmw/run.txt",
                                                                "VMware Workstation Pro17 trial")])
        self.assertEqual(self.check()["native_verified"], 4)
        _, hw_gop = self.row("HW-FR", "gop")
        hw_gop["status"] = "native-verified"
        self.rejects("native-verified applies only")

    def test_evidence_level_bound_to_profile_kind_and_references(self):
        _, gop = self.row("VM-FR", "gop")
        for level, evidence, reason in (("physical", ["docs/verification/x.json"], "cannot apply to a qemu"),
                                        ("none", ["docs/verification/x.json"], "disagree"),
                                        ("host", [], "disagree"),
                                        ("unknown", [], "cannot apply")):
            self.file("docs/verification/x.json")
            gop.update(evidence_level=level, evidence=evidence)
            with self.subTest(level=level):
                self.rejects(reason)
        gop.update(evidence_level="host", evidence=[self.file("drivers/test-host.rs")])
        self.assertTrue(self.check()["structurally_valid"])
        _, hw_gop = self.row("HW-FR", "gop")
        hw_gop.update(evidence_level="native-qemu", evidence=["docs/verification/x.json"])
        self.rejects("cannot apply to a hardware")

    def test_xhci_stays_pending_outside_qemu(self):
        for profile_id, status in (("HW-FR", "implemented"), ("HW-FR", "native-verified"),
                                   ("VM-VMW", "implemented")):
            matrix = copy.deepcopy(self.matrix)
            self.row(profile_id, "xhci", matrix)[1]["status"] = status
            with self.subTest(profile=profile_id, status=status):
                self.rejects("1b36:000d", matrix)
        self.assertEqual(self.row("VM-FR", "xhci")[1]["status"], "native-verified")

    def test_owner_decisions_are_recorded(self):
        decisions = self.matrix["owner_decisions"]
        for mutate, reason in ((lambda d: d.pop(), "omitted"),
                               (lambda d: d[1].update(status="decided"), "reference"),
                               (lambda d: d[0].update(status="decided", reference="ADR-X"), "OD-HW-TARGETS"),
                               (lambda d: d[2].update(blocks=["H09-X"]), "known physical protocols"),
                               (lambda d: d.append(dict(d[0])), "duplicate")):
            changed = copy.deepcopy(decisions)
            mutate(changed)
            matrix = dict(self.matrix, owner_decisions=changed)
            with self.subTest(reason=reason):
                self.rejects(reason, matrix)

    def test_schema1_rows_remain_valid_but_cannot_be_supported(self):
        legacy = dict(schema=1, profiles=copy.deepcopy(self.matrix["profiles"]))
        for profile in legacy["profiles"]:
            for device in profile["devices"]:
                device.pop("evidence_level")
                device.pop("note", None)
                if device["status"] == "native-verified":
                    device.update(status="pending", evidence=[])
        self.assertEqual(self.check(legacy)["schema"], 1)
        profile, gop = self.row("HW-FR", "gop", legacy)
        profile["target"] = "explicit-test-target"
        gop.update(status="supported", evidence=[self.record()])
        self.rejects("schema 1 cannot express supported", legacy)
        gop.update(status="native-verified", evidence=[])
        self.rejects("native-verified applies only", legacy)

    def test_legacy_unsigned_record_cannot_make_physical_support(self):
        profile, gop = self.row("HW-FR", "gop")
        gop["status"] = "supported"
        self.rejects("supported needs a selected HW-FR target")
        profile["target"] = "explicit-test-target"
        gop.update(evidence_level="physical", evidence=[self.record()])
        self.rejects("physical evidence level lacks a validated signed record")
        gop["evidence_level"] = "host"
        self.rejects("supported device lacks")

    def test_vtd_posture_needs_its_own_device_bound_record(self):
        _, gop = self.row("VM-FR", "gop")
        gop.update(dma_posture="VT-d contained", evidence_level="native-qemu",
                   evidence=[self.record(level="native", target=VM_TARGET)])
        self.rejects("VT-d containment")
        self.record(level="native", target=VM_TARGET, dma_posture="VT-d contained")
        self.assertTrue(self.check()["structurally_valid"])
        profile, hw_gop = self.row("HW-FR", "gop")
        profile["target"] = "explicit-test-target"
        hw_gop.update(dma_posture="VT-d contained", evidence_level="host",
                      evidence=[self.record(target="explicit-test-target", dma_posture="VT-d contained")])
        self.rejects("VT-d containment")


@unittest.skipUnless(SSH_KEYGEN, NO_SSH)
class SignedPhysicalSupportTests(MatrixCase):
    """Physical `supported` rows backed by throwaway-key signed hw-evidence records."""

    def setUp(self):
        super().setUp()
        self.fixture = Fixture(self.root)
        self.key = self.fixture.keygen("owner")
        self.fixture.allow_key(self.key)
        self.matrix["profiles"] = self.fixture.matrix["profiles"]
        self.profile, self.acpi = self.row("HW-FR", "acpi")
        self.assertEqual(self.profile["target"], TARGET)

    def test_supported_requires_validated_passing_signed_record(self):
        name = self.fixture.record(key=self.key)
        self.acpi.update(status="supported", evidence_level="physical", evidence=[name])
        self.assertEqual(self.check()["supported"], 1)
        self.acpi["evidence_level"] = "host"
        self.rejects("supported device lacks")
        self.acpi["evidence_level"] = "physical"
        _, gop = self.row("HW-FR", "gop")
        gop.update(status="supported", evidence_level="physical", evidence=[name])
        self.rejects("physical evidence level lacks")
        gop.update(status="pending", evidence_level="none", evidence=[])

        def fail_step(record):
            record["steps"][4]["result"] = "fail"
        self.fixture.record(key=self.key, passed=False, mutate=fail_step)
        self.rejects("supported device lacks")
        self.acpi["status"] = "pending"
        self.assertEqual(self.check()["supported"], 0)

    def test_invalid_referenced_records_fail_the_matrix(self):
        name = self.fixture.record(key=self.key)
        self.acpi.update(status="supported", evidence_level="physical", evidence=[name])
        (self.root / (name + ".sig")).unlink()
        with self.assertRaisesRegex(hw_evidence.EvidenceError, "unsigned"):
            self.check()
        self.fixture.record(key=self.key)
        self.fixture.allow([])
        with self.assertRaisesRegex(hw_evidence.EvidenceError, "no allowed signers"):
            self.check()
        self.fixture.allow_key(self.key)
        self.profile["target"] = "a different machine"
        with self.assertRaisesRegex(hw_evidence.EvidenceError, "target differs"):
            self.check()


if __name__ == "__main__":
    unittest.main()

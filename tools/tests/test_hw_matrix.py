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
from test_hw_evidence import NO_SSH, SSH_KEYGEN, TARGET, Fixture, symlink

VM_TARGET = "pc-q35-8.2/OVMF"
SOURCE = "a" * 64
XHCI_REPORT = "docs/verification/issue42-native-input-20261114/reports/xhci-noop-report.json"


XHCI_NOOP = ("XHCI:BEGIN", "XHCI:BUS_MASTER:CONTAINED", "XHCI:NOOP_COMPLETION:PASS", "XHCI:PASS")
XHCI_FAULT = ("XHCI:BEGIN", "XHCI:BUS_MASTER:CONTAINED", "XHCI:DMA_FAULT:WRITE_DENIED", "XHCI:PASS")
XHCI_NO_DMAR = ("XHCI:BEGIN", "XHCI:NO_DMAR:BUS_MASTER_BLOCKED", "XHCI:PASS")


def guest(machine="pc-q35-8.2", markers=XHCI_NOOP, source=SOURCE, exit_code=33, **changes) -> dict:
    """One harness guest run shaped like the committed native reports (default: the positive xHCI no-op case)."""
    run = dict(passed=True, timed_out=False, raw_qemu_exit=exit_code, expected_exit=exit_code, source_sha256=source,
               required_markers=["BOOT:EBS_OK", "KERNEL:ENTRY", *markers],
               command=["qemu-system-x86_64", "-machine", machine, "-cpu", "qemu64", "-nographic"])
    run.update(changes)
    return run


def xhci_report(**changes) -> dict:
    report = dict(schema_version=1, passed=True, source_sha256=SOURCE, guest=guest(),
                  observed=dict(markers=list(XHCI_NOOP)))
    report.update(changes)
    return report


def acpi_report(runs=3, feature="native-acpi-s5", unit=None, run=None) -> dict:
    """A multi-run `profiles` report like docs/verification/h05-acpi-s5/native-report.json."""
    entries = [dict(passed=True, guest=guest(markers=("M1:PASS",), exit_code=0),
                    observed=["POWER:S5:PM1A=0x0604:SLP_TYP=0:SCI_EN=1", "POWER:S5:WRITE"]) for _ in range(runs)]
    if run is not None:
        run(entries)
    profile = dict(schema_version=1, passed=True, kernel_feature=feature, source_sha256=SOURCE, runs=entries)
    if unit is not None:
        unit(profile)
    return dict(schema_version=1, profiles={feature: profile})


class MatrixCase(unittest.TestCase):
    """Temporary repository with the committed matrix and catalogue, synthetic drivers and copied evidence."""

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / "repo"
        self.root.mkdir()
        self.matrix = json.loads((hw_matrix.ROOT / hw_matrix.MATRIX).read_text())
        (self.root / hw_matrix.MATRIX).parent.mkdir(parents=True)
        for name in (hw_evidence.PROTOCOLS, hw_evidence.KERNEL_MANIFEST, hw_evidence.SIGNERS):
            (self.root / name).parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(hw_evidence.ROOT / name, self.root / name)
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

    def report(self, name, report):
        return self.file(name, json.dumps(report))

    def record(self, **changes):
        """Legacy unsigned schema-1 style record; no longer sufficient for physical support."""
        record = dict(passed=True, level="H", target="explicit-test-target", device="gop", source_sha256="a" * 64)
        record.update(changes)
        return self.file("docs/verification/hw/result.json", json.dumps(record))


class HardwareMatrixTests(MatrixCase):
    def test_current_inventory_and_missing_fields(self):
        summary = self.check()
        self.assertEqual((summary["schema"], summary["profiles"], summary["supported"]), (2, 3, 0))
        # The committed VM-FR xhci and acpi rows still bind to their archived passing reports.
        self.assertEqual(summary["native_verified"], 2)
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

    def test_symlinked_references_are_rejected(self):
        report = self.report("services/evil/report.json", xhci_report())
        symlink(self.root / "docs/verification/alias", "../../services/evil", self)
        symlink(self.root / "docs/verification/link/report.json", "../../../" + report, self)
        _, xhci = self.row("VM-FR", "xhci")
        for name in ("docs/verification/alias/report.json", "docs/verification/link/report.json"):
            xhci["evidence"] = [name]
            with self.subTest(name=name):
                self.rejects("symlink or aliased")
        xhci["evidence"] = ["docs/verification/issue42-native-input-20261114/README.md", XHCI_REPORT]
        symlink(self.root / "drivers/linked.rs", "xhci/src/lib.rs", self)
        xhci["driver_path"] = "drivers/linked.rs"
        self.rejects("symlink or aliased")

    def test_virtual_profiles_never_reach_supported(self):
        for profile_id, device_id in (("VM-FR", "acpi"), ("VM-FR", "gop"), ("VM-VMW", "gop")):
            matrix = copy.deepcopy(self.matrix)
            self.row(profile_id, device_id, matrix)[1]["status"] = "supported"
            with self.subTest(profile=profile_id, device=device_id):
                self.rejects("never supported", matrix)

    def test_native_verified_needs_bound_virtual_evidence(self):
        _, gop = self.row("VM-FR", "gop")
        gop["status"] = "native-verified"
        self.rejects("native-verified lacks")
        gop.update(evidence_level="native-qemu", evidence=[self.file("docs/verification/g0/run.txt",
                                                                     "machine pc-q35-10.0")])
        self.rejects("native-verified lacks")
        gop["evidence"] = [self.file("docs/notes/run.json", VM_TARGET)]
        self.rejects("docs/verification")
        # A file that merely names the machine never binds (it used to).
        gop["evidence"] = [self.file("docs/verification/g0/run.txt", "machine pc-q35-8.2, OVMF")]
        self.rejects("native-verified lacks")
        gop["evidence"] = [self.file("docs/verification/g0/run.json", "machine pc-q35-8.2, OVMF")]
        self.rejects("native-verified lacks")
        # gop has no native binding, so even a fully passing report with a GOP marker cannot qualify.
        gop["evidence"] = [self.report("docs/verification/g0/report.json",
                                       xhci_report(guest=guest(markers=("GOP:PASS",)),
                                                   observed=dict(markers=["GOP:PASS"])))]
        self.rejects("native-verified lacks")
        gop.update(status="implemented")
        _, vmw = self.row("VM-VMW", "gop")
        vmw.update(status="native-verified", evidence_level="native-qemu",
                   evidence=["docs/verification/g0/report.json"])
        self.rejects("cannot apply to a vmware profile")
        vmw.update(evidence_level="vmware", evidence=[self.file("docs/verification/vmw/run.txt",
                                                                "VMware Workstation Pro17 trial")])
        self.rejects("native-verified lacks")
        vmw.update(status="pending")
        _, vmw_acpi = self.row("VM-VMW", "acpi")
        vmware = acpi_report()
        vmware["profiles"]["native-acpi-s5"].update(machine="VMware Workstation Pro17")
        vmw_acpi.update(status="native-verified", evidence_level="vmware",
                        evidence=[self.report("docs/verification/vmw/report.json", vmware)])
        self.rejects("native-verified lacks")
        vmw_acpi.update(status="pending")
        self.assertEqual(self.check()["native_verified"], 2)
        _, hw_gop = self.row("HW-FR", "gop")
        hw_gop["status"] = "native-verified"
        self.rejects("native-verified applies only")

    def test_native_verified_accepts_bound_synthetic_reports(self):
        _, xhci = self.row("VM-FR", "xhci")
        _, acpi = self.row("VM-FR", "acpi")
        for report in (xhci_report(), xhci_report(guest=guest(machine="pc-q35-8.2,accel=tcg"))):
            xhci["evidence"] = [self.report("docs/verification/x/report.json", report)]
            with self.subTest(report=report["guest"]["command"]):
                self.assertEqual(self.check()["native_verified"], 2)
        acpi["evidence"] = [self.report("docs/verification/a/report.json", acpi_report())]
        self.assertEqual(self.check()["native_verified"], 2)

    def test_native_verified_rejects_failed_runs(self):
        _, xhci = self.row("VM-FR", "xhci")
        _, acpi = self.row("VM-FR", "acpi")
        for label, row, report in (
                ("report failed", xhci, xhci_report(passed=False)),
                ("guest failed", xhci, xhci_report(guest=guest(passed=False))),
                ("timed out", xhci, xhci_report(guest=guest(timed_out=True))),
                ("exit mismatch", xhci, xhci_report(guest=guest(raw_qemu_exit=35))),
                ("exit missing", xhci, xhci_report(guest=guest(raw_qemu_exit=None))),
                ("one run failed", acpi, acpi_report(run=lambda runs: runs[1].update(passed=False))),
                ("one guest failed", acpi, acpi_report(run=lambda runs: runs[2]["guest"].update(passed=False))),
                ("profile failed", acpi, acpi_report(unit=lambda unit: unit.update(passed=False)))):
            original = list(row["evidence"])
            # Even next to genuinely passing evidence, a failed related run rejects the row.
            row["evidence"] = original + [self.report(f"docs/verification/f/{label.replace(' ', '-')}.json", report)]
            with self.subTest(case=label):
                self.rejects("native-verified (lacks|evidence includes a failed run)")
            row["evidence"] = original
        acpi["evidence"] = [self.report("docs/verification/f/empty.json", acpi_report(runs=0))]
        self.rejects("native-verified lacks")

    def test_native_verified_rejects_wrong_machine_or_source(self):
        _, xhci = self.row("VM-FR", "xhci")
        no_machine = guest()
        no_machine["command"] = ["qemu-system-x86_64", "-nographic"]
        for label, report in (
                ("other machine", xhci_report(guest=guest(machine="pc-q35-10.0"))),
                ("machine prefix", xhci_report(guest=guest(machine="pc-q35-8.20"))),
                ("machine only in prose", xhci_report(guest=dict(no_machine, profile="pc-q35-8.2/qemu64/TCG"))),
                ("two machines", xhci_report(guest=guest(command=["q", "-machine", "pc-q35-8.2",
                                                                  "-machine", "pc-q35-10.0"]))),
                ("no source", xhci_report(source_sha256=None)),
                ("upper-case source", xhci_report(source_sha256="A" * 64,
                                                  guest=guest(source="A" * 64))),
                ("source mismatch", xhci_report(guest=guest(source="b" * 64))),
                ("marker not observed", xhci_report(observed=dict(markers=["XHCI:BEGIN"]))),
                ("not a report", ["pc-q35-8.2", "XHCI:PASS"])):
            xhci["evidence"] = [self.report(f"docs/verification/m/{label.replace(' ', '-')}.json", report)]
            with self.subTest(case=label):
                self.rejects("native-verified lacks")

    def test_native_verified_rejects_other_device_report(self):
        _, acpi = self.row("VM-FR", "acpi")
        _, gop = self.row("VM-FR", "gop")
        acpi["evidence"] = ["docs/verification/issue42-native-input-20261114/README.md", XHCI_REPORT]
        self.rejects("native-verified lacks")
        acpi["evidence"] = [self.report("docs/verification/a/s3.json", acpi_report(feature="native-acpi-s3"))]
        self.rejects("native-verified lacks")
        acpi["evidence"] = ["docs/verification/h05-acpi-s5/native-report.json"]
        gop.update(status="native-verified", evidence_level="native-qemu", evidence=[XHCI_REPORT])
        self.rejects("native-verified lacks")
        gop.update(status="implemented", evidence_level="none", evidence=[])
        _, xhci = self.row("VM-FR", "xhci")
        xhci["evidence"] = ["docs/verification/h05-acpi-s5/native-report.json"]
        self.rejects("native-verified lacks")
        xhci["evidence"] = ["docs/verification/issue42-native-input-20261114/README.md"]
        self.rejects("native-verified lacks")

    def test_passing_negative_xhci_profiles_never_bind(self):
        """Fault-injection and missing-DMAR runs also end with XHCI:PASS, but demonstrate no contained DMA."""
        _, xhci = self.row("VM-FR", "xhci")
        committed = [XHCI_REPORT.replace("noop", case) for case in ("no-dmar", "fault")]
        for name in committed:
            shutil.copyfile(hw_matrix.ROOT / name, self.root / name)
            xhci["evidence"] = [name]
            with self.subTest(committed=name):
                self.rejects("native-verified lacks")
        for label, markers in (("fault", XHCI_FAULT), ("no dmar", XHCI_NO_DMAR),
                               ("no containment", ("XHCI:BEGIN", "XHCI:NOOP_COMPLETION:PASS", "XHCI:PASS")),
                               ("no completion", ("XHCI:BEGIN", "XHCI:BUS_MASTER:CONTAINED", "XHCI:PASS"))):
            xhci["evidence"] = [self.report(f"docs/verification/n/{label.replace(' ', '-')}.json",
                                            xhci_report(guest=guest(markers=markers),
                                                        observed=dict(markers=list(markers))))]
            with self.subTest(synthetic=label):
                self.rejects("native-verified lacks")
        # Passing negative profiles remain valid context next to the positive no-op report.
        xhci["evidence"] = [XHCI_REPORT, *committed]
        self.assertEqual(self.check()["native_verified"], 2)
        # A failed negative profile still contradicts the row, even next to the positive report.
        xhci["evidence"] = [XHCI_REPORT, self.report("docs/verification/n/fault-failed.json",
                                                     xhci_report(passed=False, guest=guest(markers=XHCI_FAULT),
                                                                 observed=dict(markers=list(XHCI_FAULT))))]
        self.rejects("failed run")

    def test_positive_xhci_run_with_negative_marker_fails(self):
        _, xhci = self.row("VM-FR", "xhci")
        for label, report in (
                ("observed no dmar", xhci_report(observed=dict(markers=[*XHCI_NOOP,
                                                                        "XHCI:NO_DMAR:BUS_MASTER_BLOCKED"]))),
                ("observed dma fault", xhci_report(observed=[*XHCI_NOOP, "XHCI:DMA_FAULT:WRITE_DENIED"])),
                ("required dma fault", xhci_report(guest=guest(markers=(*XHCI_NOOP, "XHCI:DMA_FAULT:WRITE_DENIED")))),
                ("observed failure", xhci_report(observed=dict(markers=[*XHCI_NOOP, "XHCI:FAIL:PREPARE:0x1"])))):
            xhci["evidence"] = [XHCI_REPORT, self.report(f"docs/verification/c/{label.replace(' ', '-')}.json", report)]
            with self.subTest(case=label):
                self.rejects("negative-case marker")

    def test_native_report_size_and_format(self):
        _, xhci = self.row("VM-FR", "xhci")
        xhci["evidence"] = [self.file("docs/verification/big/report.json", " " * (hw_matrix.MAX_REPORT + 1))]
        self.rejects("oversize")
        xhci["evidence"] = [self.file("docs/verification/bad/report.json", "{not json")]
        self.rejects("not valid JSON")

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

        def shrink_boot(changed):
            next(d for d in changed if d["id"] == "OD-HW-BOOT")["blocks"].remove("H05-S5")

        for mutate, reason in ((lambda d: d.pop(), "omitted"),
                               (lambda d: d[1].update(status="decided"), "reference"),
                               (lambda d: d[0].update(status="decided", reference="ADR-X"), "OD-HW-TARGETS"),
                               (lambda d: d[2].update(blocks=["H09-X"]), "known physical protocols"),
                               (lambda d: d.append(dict(d[0])), "duplicate"),
                               (shrink_boot, "OD-HW-BOOT blocks fewer"),
                               (lambda d: next(x for x in d if x["id"] == "OD-HW-POWER").update(blocks=["H05-S5"]),
                                "OD-HW-POWER blocks fewer")):
            changed = copy.deepcopy(decisions)
            mutate(changed)
            matrix = dict(self.matrix, owner_decisions=changed)
            with self.subTest(reason=reason):
                self.rejects(reason, matrix)

    def test_ready_protocol_with_open_blocking_decision_fails_matrix(self):
        fixture = Fixture(self.root)
        self.matrix.update(profiles=fixture.matrix["profiles"], owner_decisions=fixture.matrix["owner_decisions"])
        summary = self.check()
        self.assertEqual(summary["owner_decisions_open"], 3)
        fixture.decision("OD-HW-CAPTURE").update(status="open")
        fixture.decision("OD-HW-CAPTURE").pop("reference")
        # No record references H05-S5: the catalogue state alone fails the matrix check.
        self.rejects("H05-S5: ready while owner decision.*OD-HW-CAPTURE")

    def test_unsigned_hw_evidence_reference_fails_on_any_row(self):
        record = self.file("docs/verification/hw/hw-evidence-gop.json", "{}")
        result = self.record()
        for profile_id, changes in (("HW-FR", dict(status="pending", evidence_level="host")),
                                    ("HW-FR", dict(status="implemented", evidence_level="host")),
                                    ("HW-FR", dict(status="unsupported", driver_path=None, evidence_level="host")),
                                    ("VM-FR", dict(status="implemented", evidence_level="native-qemu")),
                                    ("VM-VMW", dict(status="pending", evidence_level="vmware"))):
            matrix = copy.deepcopy(self.matrix)
            self.row(profile_id, "gop", matrix)[1].update(evidence=[record], **changes)
            with self.subTest(profile=profile_id, changes=changes):
                self.rejects("no allowed signers", matrix)
        legacy = dict(schema=1, profiles=copy.deepcopy(self.matrix["profiles"]))
        for profile in legacy["profiles"]:
            for device in profile["devices"]:
                device.pop("evidence_level")
                device.pop("note", None)
                if device["status"] == "native-verified":
                    device.update(status="pending", evidence=[])
        self.row("HW-FR", "gop", legacy)[1]["evidence"] = [record]
        self.rejects("no allowed signers", legacy)
        # A non-record JSON reference on a host-level row keeps the legacy structural path.
        self.row("HW-FR", "gop")[1].update(evidence_level="host", evidence=[result])
        self.assertTrue(self.check()["structurally_valid"])

    def test_vm_acpi_note_matches_present_power_features(self):
        protocols = hw_evidence.load_protocols()
        features = {p["image"]["kernel_feature"] for p in protocols.values()
                    if p["devices"] == ["acpi"] and p["image"]["kernel_feature_present"]}
        self.assertEqual(features, {"native-acpi-s5", "native-acpi-s3"})
        _, acpi = self.row("VM-FR", "acpi")
        for feature in features:
            self.assertIn(feature, acpi["note"])
        self.assertNotIn("host-parsed", acpi["note"])
        self.assertIn("No physical evidence", acpi["note"])
        if all(protocols[ident]["status"] == "blocked" for ident in ("H05-S5", "H05-S3")):
            _, hw_acpi = self.row("HW-FR", "acpi")
            self.assertIn(hw_acpi["evidence_level"], {"none", "host"})
            self.assertNotEqual(hw_acpi["status"], "supported")

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
        # A hand-written legacy record naming the posture no longer binds a virtual row (it used to).
        self.record(level="native", target=VM_TARGET, dma_posture="VT-d contained")
        self.rejects("VT-d containment lacks")
        profile, hw_gop = self.row("HW-FR", "gop")
        profile["target"] = "explicit-test-target"
        hw_gop.update(dma_posture="VT-d contained", evidence_level="host",
                      evidence=[self.record(target="explicit-test-target", dma_posture="VT-d contained")])
        self.rejects("VT-d containment")

    def test_virtual_vtd_posture_needs_bound_contained_native_report(self):
        _, xhci = self.row("VM-FR", "xhci")
        committed = list(xhci["evidence"])
        # The committed positive xHCI report requires and observes XHCI:BUS_MASTER:CONTAINED, so it binds.
        xhci["dma_posture"] = "VT-d contained"
        self.assertEqual(self.check()["native_verified"], 2)
        xhci["evidence"] = [self.report("docs/verification/v/report.json", xhci_report())]
        self.assertEqual(self.check()["native_verified"], 2)
        # From here on the row is only `implemented`, so the posture check alone decides.
        xhci["status"] = "implemented"
        negative = [XHCI_REPORT.replace("noop", case) for case in ("no-dmar", "fault")]
        for name in negative:
            shutil.copyfile(hw_matrix.ROOT / name, self.root / name)
        legacy = self.record(level="native", target=VM_TARGET, device="xhci", dma_posture="VT-d contained")
        uncontained = ("XHCI:BEGIN", "XHCI:NOOP_COMPLETION:PASS", "XHCI:PASS")
        for label, changes in (
                ("hand-written record", dict(evidence=[legacy])),
                ("prose only", dict(evidence=[committed[0]])),
                ("negative profiles only", dict(evidence=negative)),
                ("no contained marker", dict(evidence=[self.report(
                    "docs/verification/v/uncontained.json",
                    xhci_report(guest=guest(markers=uncontained), observed=dict(markers=list(uncontained))))])),
                ("other machine", dict(evidence=[self.report("docs/verification/v/q35-10.json",
                                                             xhci_report(guest=guest(machine="pc-q35-10.0")))])),
                ("host level", dict(evidence_level="host", evidence=committed))):
            matrix = copy.deepcopy(self.matrix)
            self.row("VM-FR", "xhci", matrix)[1].update(changes)
            with self.subTest(case=label):
                self.rejects("VT-d containment lacks", matrix)
        # A failed related run still rejects the row outright.
        xhci["evidence"] = committed + [self.report("docs/verification/v/failed.json", xhci_report(passed=False))]
        self.rejects("failed run")
        xhci.update(status="native-verified", evidence=committed)
        self.assertEqual(self.check()["native_verified"], 2)
        # ACPI binds as native-verified, but its binding observes no bus-master containment marker.
        _, acpi = self.row("VM-FR", "acpi")
        acpi["dma_posture"] = "VT-d contained"
        self.rejects("VT-d containment lacks")
        acpi["dma_posture"] = "trusted kernel transport"
        # VMware rows cannot claim it, even with a copy of a passing QEMU report.
        _, vmw = self.row("VM-VMW", "xhci")
        vmw.update(dma_posture="VT-d contained", evidence_level="vmware", evidence=[XHCI_REPORT])
        self.rejects("VT-d containment lacks")
        vmw.update(dma_posture="unsupported", evidence_level="none", evidence=[])
        # A family without a native binding (vtd itself) cannot claim it through another family's report.
        _, vtd = self.row("VM-FR", "vtd")
        vtd.update(dma_posture="VT-d contained", evidence_level="native-qemu", evidence=[XHCI_REPORT])
        self.rejects("VT-d containment lacks")


@unittest.skipUnless(SSH_KEYGEN, NO_SSH)
class SignedPhysicalSupportTests(MatrixCase):
    """Physical `supported` rows backed by throwaway-key signed hw-evidence records."""

    def setUp(self):
        super().setUp()
        self.use("H05-S5")

    def use(self, protocol):
        self.fixture = Fixture(self.root, protocol)
        self.key = self.fixture.keygen(f"owner-{protocol.lower()}")
        self.fixture.allow_key(self.key)
        self.prerequisites = self.fixture.prerequisites(self.key)
        self.matrix["profiles"] = self.fixture.matrix["profiles"]
        self.matrix["owner_decisions"] = self.fixture.matrix["owner_decisions"]
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
        self.rejects("does not bind")
        gop.update(status="pending", evidence_level="none", evidence=[])
        # The accepted HW-INV prerequisite is part of the support claim.
        signature = self.root / (self.prerequisites[0] + ".sig")
        saved = signature.read_bytes()
        signature.unlink()
        with self.assertRaisesRegex(hw_evidence.EvidenceError, "precondition unmet.*HW-INV"):
            self.check()
        signature.write_bytes(saved)
        self.assertEqual(self.check()["supported"], 1)

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

    def test_pending_row_with_invalid_signed_record_fails(self):
        name = self.fixture.record(key=self.key)
        self.acpi.update(status="pending", evidence_level="host", evidence=[name])
        self.assertEqual(self.check()["supported"], 0)
        signature = self.root / (name + ".sig")
        saved = signature.read_bytes()
        signature.unlink()
        with self.assertRaisesRegex(hw_evidence.EvidenceError, "unsigned"):
            self.check()
        signature.write_bytes(saved)
        record = self.root / name
        record.write_bytes(record.read_bytes() + b" ")
        with self.assertRaisesRegex(hw_evidence.EvidenceError, "signature verification failed"):
            self.check()
        self.fixture.record(key=self.key)
        self.acpi.update(evidence_level="none", evidence=[])
        _, vm_acpi = self.row("VM-FR", "acpi")
        vm_acpi["evidence"] = vm_acpi["evidence"] + [name]
        self.rejects("does not bind")
        vm_acpi["evidence"] = vm_acpi["evidence"][:-1]
        _, gop = self.row("HW-FR", "gop")
        gop.update(evidence_level="host", evidence=[name])
        self.rejects("does not bind")
        gop.update(evidence_level="none", evidence=[])
        # Reverse alias: a reference outside docs/verification that resolves to a valid record.
        symlink(self.root / "services/ref.json", "../" + name, self)
        self.acpi.update(evidence_level="host", evidence=["services/ref.json"])
        self.rejects("symlink or aliased")

    def test_vtd_contained_row_rejects_hwinv_only_record(self):
        _, xhci = self.row("HW-FR", "xhci")
        xhci.update(dma_posture="VT-d contained", evidence_level="physical", evidence=[self.prerequisites[0]])
        self.rejects("does not bind|VT-d containment lacks")

    def test_install_record_cannot_promote_unexercised_storage_rows(self):
        self.use("R03-INSTALL")
        lines = self.fixture.serial_lines()
        target = lines.index("INSTALL:TARGET:1")
        _, nvme = self.row("HW-FR", "nvme")
        _, ahci = self.row("HW-FR", "ahci")
        nvme["driver_path"] = ahci["driver_path"] = self.file("drivers/storage/src/lib.rs")
        usb = self.fixture.record(key=self.key, devices=[],
                                  lines=lines[:target] + ["INSTALL:TARGET:USB-MSC:0000:00:14.0"] + lines[target + 1:])
        nvme.update(status="supported", evidence_level="physical", evidence=[usb])
        self.rejects("does not bind")
        self.fixture.record(key=self.key, lines=lines[:target] + ["INSTALL:TARGET:USB-MSC:0000:00:14.0"]
                            + lines[target + 1:])
        with self.assertRaisesRegex(hw_evidence.EvidenceError, "device nvme listed without its transport marker"):
            self.check()
        name = self.fixture.record(key=self.key, devices=["nvme"],
                                   lines=lines[:target] + ["INSTALL:TARGET:NVME:0000:01:00.0"] + lines[target + 1:])
        self.assertEqual(self.check()["supported"], 1)
        ahci.update(status="supported", evidence_level="physical", evidence=[name])
        self.rejects("does not bind|physical evidence level lacks")


if __name__ == "__main__":
    unittest.main()

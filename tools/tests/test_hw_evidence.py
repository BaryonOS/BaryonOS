"""Adversarial physical-evidence fixtures with throwaway keys; no hardware execution or acceptance."""
import base64
import copy
import hashlib
import json
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import hw_evidence

SSH_KEYGEN = shutil.which("ssh-keygen")
NO_SSH = "ssh-keygen (OpenSSH) is unavailable: signature tests NOT RUN"
TARGET = "Synthetic reference PC A (test fixture)"
SIGNER = "owner@example.test"
NAMESPACE = hw_evidence.NAMESPACE
RECORD_DIR = "docs/verification/hw-fixture"
RECORD = f"{RECORD_DIR}/hw-evidence.json"


def fake_key(keytype: str = "ssh-ed25519") -> str:
    """A syntactically valid public-key blob (not a usable key) for format-only tests."""
    name = keytype.encode()
    return base64.b64encode(struct.pack(">I", len(name)) + name + struct.pack(">I", 32) + b"\x01" * 32).decode()


def fake_signature(namespace: str = NAMESPACE) -> str:
    """An armored SSHSIG container with the given namespace and no valid signature."""
    def string(value: bytes) -> bytes:
        return struct.pack(">I", len(value)) + value
    blob = b"SSHSIG" + struct.pack(">I", 1) + string(base64.b64decode(fake_key())) + string(namespace.encode())
    blob += string(b"") + string(b"sha512") + string(b"\x00" * 16)
    return "-----BEGIN SSH SIGNATURE-----\n" + base64.b64encode(blob).decode() + "\n-----END SSH SIGNATURE-----\n"


def physical_inventory(**changes) -> dict:
    inventory = dict(schema=1, kind="baryonos-hw-inventory",
                     cpuid=dict(vendor="GenuineIntel", hypervisor_bit=False), secure_boot="unknown",
                     pci=[dict(address="0000:00:00.0", vendor="8086", device="4660", **{"class": "06"},
                               subclass="00", prog_if="00", revision="02"),
                          dict(address="0000:00:14.0", vendor="8086", device="7ae0", **{"class": "0c"},
                               subclass="03", prog_if="30", revision="11")])
    inventory.update(changes)
    return inventory


class Fixture:
    """Temporary repository root with copied catalogue, a selected HW-FR target and one ready protocol."""

    def __init__(self, root: Path, protocol: str = "H05-S5"):
        self.root, self.protocol_id = root, protocol
        for name in (hw_evidence.PROTOCOLS, hw_evidence.KERNEL_MANIFEST, hw_evidence.SIGNERS, hw_evidence.MATRIX):
            (root / name).parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(hw_evidence.ROOT / name, root / name)
        self.protocols = json.loads((root / hw_evidence.PROTOCOLS).read_text())
        self.protocol = next(p for p in self.protocols["protocols"] if p["id"] == protocol)
        self.protocol.update(status="ready", blocking_reasons=[], markers_implemented=True)
        self.protocol["image"]["physical_image"] = "available"
        self.write_protocols()
        self.matrix = json.loads((root / hw_evidence.MATRIX).read_text())
        self.hw_profile["target"] = TARGET
        self.write_matrix()
        self.keys = Path(tempfile.mkdtemp(dir=root.parent))

    @property
    def hw_profile(self) -> dict:
        return next(p for p in self.matrix["profiles"] if p["id"] == "HW-FR")

    def write_protocols(self):
        (self.root / hw_evidence.PROTOCOLS).write_text(json.dumps(self.protocols))

    def write_matrix(self):
        (self.root / hw_evidence.MATRIX).write_text(json.dumps(self.matrix))

    def keygen(self, name: str) -> Path:
        key = self.keys / name
        subprocess.run([SSH_KEYGEN, "-q", "-t", "ed25519", "-N", "", "-C", name, "-f", str(key)],
                       check=True, capture_output=True)
        return key

    def allow(self, entries: list[str]):
        (self.root / hw_evidence.SIGNERS).write_text("# test signers\n" + "".join(e + "\n" for e in entries))

    def allow_key(self, key: Path, principal: str = SIGNER, options: str = f'namespaces="{NAMESPACE}"'):
        keytype, blob = key.with_suffix(".pub").read_text().split()[:2]
        self.allow([f"{principal} {options} {keytype} {blob}".replace("  ", " ")])

    def serial_lines(self) -> list[str]:
        return [m[:-1] + "1" if m.endswith("*") else m for m in self.protocol["required_markers"]]

    def write(self, name: str, data: bytes) -> dict:
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        return dict(path=name, sha256=hashlib.sha256(data).hexdigest())

    def record(self, *, key: Path | None = None, namespace: str = NAMESPACE, name: str = RECORD,
               inventory: dict | None = None, lines: list[str] | None = None, mutate=None, **changes) -> str:
        directory = name.rsplit("/", 1)[0]
        record = dict(
            schema=1, kind=NAMESPACE, protocol=dict(id=self.protocol_id, version=self.protocol["version"]),
            profile="HW-FR", target=TARGET, devices=list(self.protocol["devices"]),
            image=dict(name="baryonos-test.img", sha256="1" * 64), source_sha256="2" * 64,
            source_commit="3" * 40,
            inventory=self.write(f"{directory}/inventory.json",
                                 json.dumps(physical_inventory() if inventory is None else inventory).encode()),
            serial_log=self.write(f"{directory}/serial.log",
                                  "\n".join(self.serial_lines() if lines is None else lines).encode() + b"\n"),
            secure_boot="disabled", operator="Test Operator", signer=SIGNER, date="2026-11-19",
            safety_confirmations={action: True for action in self.protocol["forbidden_actions"]},
            steps=[dict(id=step["id"], result="pass") for step in self.protocol["steps"]],
            dma_posture="unsupported", passed=True, notes="synthetic fixture; not physical evidence")
        record.update(changes)
        if mutate is not None:
            mutate(record)
        self.write(name, json.dumps(record, indent=2).encode())
        signature = self.root / (name + ".sig")
        signature.unlink(missing_ok=True)
        if key is not None:
            subprocess.run([SSH_KEYGEN, "-Y", "sign", "-f", str(key), "-n", namespace, str(self.root / name)],
                           check=True, capture_output=True)
        return name


class RepositoryStateTests(unittest.TestCase):
    """Runs without ssh-keygen: committed catalogue, empty signer file and format rules."""

    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name) / "repo"
        self.root.mkdir()
        self.fixture = Fixture(self.root)

    def test_repository_protocols_are_blocked_and_signers_empty(self):
        summary = hw_evidence.check()
        self.assertEqual(summary["protocols"], 7)
        self.assertEqual(summary["ready"], [])
        self.assertEqual(summary["signers"], 0)
        self.assertEqual(hw_evidence.load_signers(), set())
        for protocol in hw_evidence.load_protocols().values():
            self.assertTrue(protocol["blocking_reasons"])

    def test_committed_empty_signer_file_rejects_every_record(self):
        name = self.fixture.record()
        (self.root / (name + ".sig")).write_text(fake_signature())
        with self.assertRaisesRegex(hw_evidence.EvidenceError, "no allowed signers.*OD-HW-SIGNERS"):
            hw_evidence.validate_record(name, self.root)
        with self.assertRaisesRegex(hw_evidence.EvidenceError, "rejected physical evidence"):
            hw_evidence.check(self.root)

    def test_missing_ssh_keygen_fails_closed(self):
        self.fixture.allow([f'{SIGNER} namespaces="{NAMESPACE}" ssh-ed25519 {fake_key()}'])
        name = self.fixture.record()
        (self.root / (name + ".sig")).write_text(fake_signature())
        with mock.patch.object(hw_evidence, "ssh_keygen", return_value=None):
            with self.assertRaisesRegex(hw_evidence.EvidenceError, "ssh-keygen is unavailable"):
                hw_evidence.validate_record(name, self.root)

    def test_allowed_signer_entries_are_strict(self):
        key = fake_key()
        self.fixture.allow([f'{SIGNER} namespaces="{NAMESPACE}" ssh-ed25519 {key} comment'])
        self.assertEqual(hw_evidence.load_signers(self.root), {SIGNER})
        for entry, reason in (
                (f"{SIGNER} ssh-ed25519 {key}", "namespaces"),
                (f'{SIGNER} namespaces="{NAMESPACE},git" ssh-ed25519 {key}', "namespaces"),
                (f'{SIGNER} cert-authority,namespaces="{NAMESPACE}" ssh-ed25519 {key}', "cert-authority"),
                (f'*@example.test namespaces="{NAMESPACE}" ssh-ed25519 {key}', "exact identities"),
                (f'{SIGNER} namespaces="{NAMESPACE}" ssh-rsa {fake_key("ssh-rsa")}', "key type"),
                (f'{SIGNER} namespaces="{NAMESPACE}" ssh-ed25519 {fake_key("ssh-dss")}', "key blob"),
                (f'{SIGNER} permit-pty namespaces="{NAMESPACE}" ssh-ed25519 {key}', "format|option")):
            self.fixture.allow([entry])
            with self.subTest(entry=entry), self.assertRaisesRegex(hw_evidence.EvidenceError, reason):
                hw_evidence.load_signers(self.root)

    def test_protocol_catalogue_drift_is_rejected(self):
        hw_evidence.load_protocols(self.root)
        original = copy.deepcopy(self.fixture.protocols)

        def protocol(doc, ident):
            return next(p for p in doc["protocols"] if p["id"] == ident)

        for mutate, reason in (
                # Flip whatever the catalogue states; the real Cargo.toml decides which is drift.
                (lambda d: protocol(d, "H05-S3")["image"].update(
                    kernel_feature_present=not protocol(d, "H05-S3")["image"]["kernel_feature_present"]),
                 "disagrees"),
                (lambda d: protocol(d, "HW-INV").update(status="ready", blocking_reasons=[]), "ready requires"),
                (lambda d: protocol(d, "H05-S3").update(blocking_reasons=[]), "blocking_reasons"),
                (lambda d: protocol(d, "H02-USB")["forbidden_actions"].remove("no-bootorder-change"), "baseline"),
                (lambda d: protocol(d, "H04-NET")["evidence_fields"].remove("signer"), "evidence_fields"),
                (lambda d: protocol(d, "H03-AV").update(devices=["wifi"]), "devices"),
                (lambda d: d["protocols"].pop(), "missing"),
                (lambda d: d.update(namespace="git"), "namespace"),
                (lambda d: d["record_fields"].append("extra"), "record_fields")):
            doc = copy.deepcopy(original)
            mutate(doc)
            self.fixture.protocols = doc
            self.fixture.write_protocols()
            with self.subTest(reason=reason), self.assertRaisesRegex(hw_evidence.EvidenceError, reason):
                hw_evidence.load_protocols(self.root)


@unittest.skipUnless(SSH_KEYGEN, NO_SSH)
class SignedEvidenceTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name) / "repo"
        self.root.mkdir()
        self.fixture = Fixture(self.root)
        self.key = self.fixture.keygen("owner")
        self.fixture.allow_key(self.key)

    def signed(self, **changes) -> str:
        return self.fixture.record(key=self.key, **changes)

    def rejects(self, name: str, reason: str):
        with self.assertRaisesRegex(hw_evidence.EvidenceError, reason):
            hw_evidence.validate_record(name, self.root)

    def test_valid_signed_record_is_accepted(self):
        summary = hw_evidence.validate_record(self.signed(), self.root)
        self.assertEqual((summary["protocol"], summary["target"], summary["devices"], summary["passed"]),
                         ("H05-S5", TARGET, ["acpi"], True))
        self.assertEqual(len(hw_evidence.check(self.root)["accepted"]), 1)

    def test_signature_rules(self):
        self.rejects(self.fixture.record(), "unsigned")
        self.rejects(self.signed(signer="intruder@example.test"), "unknown signer")
        self.rejects(self.fixture.record(key=self.fixture.keygen("intruder")), "verification failed")
        self.rejects(self.fixture.record(key=self.key, namespace="git"), "wrong signature namespace")
        name = self.signed()
        path = self.root / name
        path.write_text(path.read_text().replace('"passed": true', '"passed": true ', 1))
        self.rejects(name, "verification failed")
        self.fixture.allow_key(self.key, options='namespaces="other-namespace"')
        self.rejects(self.signed(), "namespaces")

    def test_hash_bindings(self):
        self.rejects(self.signed(mutate=lambda r: r["inventory"].update(sha256="0" * 64)), "inventory SHA-256")
        self.rejects(self.signed(mutate=lambda r: r["serial_log"].update(sha256="0" * 64)), "serial log SHA-256")
        name = self.signed()
        (self.root / RECORD_DIR / "inventory.json").write_text(json.dumps(physical_inventory(pci=[])))
        self.rejects(name, "inventory SHA-256")
        self.rejects(self.signed(mutate=lambda r: r["image"].update(sha256="F" * 64)), "image")
        self.rejects(self.signed(source_sha256="unknown"), "source_sha256")
        self.rejects(self.signed(source_commit="3" * 12), "source_commit")
        other = self.fixture.write("docs/verification/elsewhere/inventory.json", b"{}")
        self.rejects(self.signed(inventory=None, mutate=lambda r: r.update(inventory=other)), "record directory")

    def test_target_protocol_and_device_bindings(self):
        self.rejects(self.signed(target="another machine"), "target differs")
        self.rejects(self.signed(protocol=dict(id="H05-S5", version=2)), "version mismatch")
        self.rejects(self.signed(protocol=dict(id="H05-S4", version=1)), "unknown protocol")
        self.rejects(self.signed(profile="VM-FR"), "profile must be HW-FR")
        self.rejects(self.signed(devices=["xhci"]), "devices are outside")
        self.rejects(self.signed(dma_posture="contained"), "DMA posture")
        self.rejects(self.signed(inventory=physical_inventory(secure_boot="enabled")), "Secure Boot state differs")
        self.fixture.protocol.update(status="blocked", blocking_reasons=["test block"])
        self.fixture.write_protocols()
        self.rejects(self.signed(), "blocked: test block")
        self.fixture.protocol.update(status="ready", blocking_reasons=[])
        self.fixture.write_protocols()
        self.fixture.hw_profile["target"] = None
        self.fixture.write_matrix()
        self.rejects(self.signed(), "OD-HW-TARGETS")

    def test_virtual_captures_are_rejected(self):
        self.rejects(self.signed(inventory=physical_inventory(cpuid=dict(vendor="GenuineIntel",
                                                                          hypervisor_bit=True))), "hypervisor bit")
        for vendor in ("1af4", "1b36", "1234", "15ad"):
            pci = [dict(address="0000:00:03.0", vendor=vendor, device="0001", **{"class": "02"},
                        subclass="00", prog_if="00", revision="00")]
            with self.subTest(vendor=vendor):
                self.rejects(self.signed(inventory=physical_inventory(pci=pci)), f"PCI vendor {vendor}")
        lines = self.fixture.serial_lines()
        self.rejects(self.signed(lines=["HWINV:CPUID:VENDOR=AuthenticAMD:HYPERVISOR=1"] + lines), "hypervisor bit")
        self.rejects(self.signed(lines=["HWINV:PCI:0000:00:02.0:1B36:000d:0c:03:30:01"] + lines), "PCI vendor 1B36")
        self.rejects(self.signed(inventory=physical_inventory(cpuid=dict(vendor="GenuineIntel"))), "hypervisor bit")

    def test_failed_runs_cannot_be_relabelled_as_passed(self):
        lines = self.fixture.serial_lines()

        def fail_step(record):
            record["steps"][4]["result"] = "fail"
        self.rejects(self.signed(mutate=fail_step), "relabelled as passed: step")
        self.rejects(self.signed(lines=lines[:-1]), "relabelled as passed: required marker 'POWER:S5:WRITE'")
        self.rejects(self.signed(lines=lines[:-2] + [lines[-1], lines[-2]]), "missing or out of order")
        self.rejects(self.signed(lines=lines + ["POWER:S5:REJECTED:InvalidChecksum"]), "failure marker")
        honest = hw_evidence.validate_record(self.signed(passed=False, mutate=fail_step), self.root)
        self.assertFalse(honest["passed"])
        self.rejects(self.signed(passed="yes"), "boolean")

    def test_safety_confirmations_are_required(self):
        def unconfirmed(record):
            record["safety_confirmations"]["no-firmware-flash"] = False
        self.rejects(self.signed(mutate=unconfirmed), "not given: no-firmware-flash")
        self.rejects(self.signed(mutate=lambda r: r["safety_confirmations"].pop("no-internal-disk-access")),
                     "exactly the protocol")
        self.rejects(self.signed(mutate=lambda r: r["safety_confirmations"].update(extra=True)),
                     "exactly the protocol")
        self.rejects(self.signed(mutate=lambda r: r.update(steps=r["steps"][:-1])), "every protocol step")
        self.rejects(self.signed(operator=""), "operator")
        self.rejects(self.signed(date="2026-13-40"), "date")
        self.rejects(self.signed(mutate=lambda r: r.update(extra=1)), "unknown or missing fields")

    def test_record_location_rules(self):
        self.rejects(self.signed(name="docs/verification/hw-evidence.json"), "records must be")
        self.rejects(self.signed(name=f"{RECORD_DIR}/evidence.json"), "records must be")
        self.rejects(self.signed(name=f"docs/other/{RECORD_DIR}/hw-evidence.json"), "records must be")


if __name__ == "__main__":
    unittest.main()

"""Adversarial physical-evidence fixtures with throwaway keys; no hardware execution or acceptance."""
import base64
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import struct
import subprocess
import sys
import tempfile
import unittest
import unittest.mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import hw_evidence

SSH_KEYGEN = shutil.which("ssh-keygen")
NO_SSH = "ssh-keygen (OpenSSH) is unavailable: signature tests NOT RUN"
TARGET = "Synthetic reference PC A (test fixture)"
SIGNER = "owner@example.test"
NAMESPACE = hw_evidence.NAMESPACE
RECORD_DIR = "docs/verification/hw-fixture"
RECORD = f"{RECORD_DIR}/hw-evidence.json"
FIXTURE_REFERENCE = "synthetic test fixture"
# Plausible serial lines for wildcard markers whose content the validator parses or cross-checks.
# The HWINV CPUID and Secure Boot lines follow the inventory (cpuid_line, secureboot_line).
WILDCARD_LINES = {
    "HWINV:ACPI:DMAR=*": "HWINV:ACPI:DMAR=PRESENT:DRHD=1",
    "HDA:PLAYBACK:START:*": "HDA:PLAYBACK:START:MS=2000",
}
# Firmware console output that can precede the loader's marker on the same serial line (from a QEMU/OVMF capture).
FIRMWARE = '\x1b[2J\x1b[001;001HBdsDxe: starting Boot0002 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x1,0x0)'


def key_blob(*fields: bytes) -> str:
    """Base64 of SSH wire-format strings (format-only test material, never a usable key)."""
    return base64.b64encode(b"".join(struct.pack(">I", len(field)) + field for field in fields)).decode()


def fake_key(keytype: str = "ssh-ed25519") -> str:
    """A syntactically valid public-key blob (not a usable key) for format-only tests."""
    return key_blob(keytype.encode(), b"\x01" * 32)


def ec_point(size: int) -> bytes:
    return b"\x04" + b"\x01" * (size - 1)


STRUCTURAL_KEYS = {
    "ssh-ed25519": key_blob(b"ssh-ed25519", b"\x01" * 32),
    "sk-ssh-ed25519@openssh.com": key_blob(b"sk-ssh-ed25519@openssh.com", b"\x01" * 32, b"ssh:"),
    "ecdsa-sha2-nistp256": key_blob(b"ecdsa-sha2-nistp256", b"nistp256", ec_point(65)),
    "ecdsa-sha2-nistp384": key_blob(b"ecdsa-sha2-nistp384", b"nistp384", ec_point(97)),
    "ecdsa-sha2-nistp521": key_blob(b"ecdsa-sha2-nistp521", b"nistp521", ec_point(133)),
    "sk-ecdsa-sha2-nistp256@openssh.com": key_blob(b"sk-ecdsa-sha2-nistp256@openssh.com", b"nistp256",
                                                   ec_point(65), b"ssh:"),
}


def fake_signature(namespace: str = NAMESPACE) -> str:
    """An armored SSHSIG container with the given namespace and no valid signature."""
    def string(value: bytes) -> bytes:
        return struct.pack(">I", len(value)) + value
    blob = b"SSHSIG" + struct.pack(">I", 1) + string(base64.b64decode(fake_key())) + string(namespace.encode())
    blob += string(b"") + string(b"sha512") + string(b"\x00" * 16)
    return "-----BEGIN SSH SIGNATURE-----\n" + base64.b64encode(blob).decode() + "\n-----END SSH SIGNATURE-----\n"


def physical_inventory(**changes) -> dict:
    inventory = dict(schema=2, kind="baryonos-hw-inventory",
                     cpuid=dict(vendor="GenuineIntel", hypervisor_bit=False), secure_boot="disabled",
                     acpi=dict(dmar="present", drhd_units=1),
                     pci=[dict(address="0000:00:00.0", vendor="8086", device="4660", **{"class": "06"},
                               subclass="00", prog_if="00", revision="02"),
                          dict(address="0000:00:14.0", vendor="8086", device="7ae0", **{"class": "0c"},
                               subclass="03", prog_if="30", revision="11")])
    inventory.update(changes)
    return inventory


def pci_lines(inventory: dict) -> list[str]:
    """The HWINV:PCI lines and count a native inventory run emits for this inventory."""
    pci = inventory.get("pci") or []
    return ["HWINV:PCI:" + ":".join(str(f.get(k, "")) for k in hw_evidence.PCI_FIELDS) for f in pci] + [
        f"HWINV:PCI:COUNT={len(pci)}"]


def dmar_line(inventory: dict) -> str:
    acpi = inventory.get("acpi") or {}
    return f"HWINV:ACPI:DMAR={str(acpi.get('dmar', '')).upper()}:DRHD={acpi.get('drhd_units', '')}"


def cpuid_line(inventory: dict) -> str:
    return f"HWINV:CPUID:VENDOR={(inventory.get('cpuid') or {}).get('vendor', '')}:HYPERVISOR=0"


def secureboot_line(inventory: dict) -> str:
    value = inventory.get("secure_boot")
    state = {"enabled": "1", "disabled": "0", "unknown": "ABSENT"}.get(value, "") if isinstance(value, str) else ""
    return f"HWINV:SECUREBOOT={state}"


def signer_state_problem(root: Path) -> str | None:
    """Repository rule: the signer file lists a key exactly when OD-HW-SIGNERS is decided."""
    signers = hw_evidence.load_signers(root)
    matrix = json.loads((root / hw_evidence.MATRIX).read_text())
    status = next(d for d in matrix["owner_decisions"] if d["id"] == "OD-HW-SIGNERS")["status"]
    if status == "open" and signers:
        return "a signer is listed while OD-HW-SIGNERS is open"
    if status == "decided" and not signers:
        return "OD-HW-SIGNERS is decided but no signer is listed"
    return None


def protocol_table(text: str) -> dict[str, list[tuple[int, str]]]:
    """Rows of the 'Physical test protocols' table in HARDWARE-TARGETS.md: id -> [(version, reasons)]."""
    section = text.split("## Physical test protocols", 1)[1].split("\n## ", 1)[0]
    rows: dict[str, list[tuple[int, str]]] = {}
    for line in section.splitlines():
        match = re.fullmatch(r"\| `([A-Z0-9-]+)` v([0-9]+) \|[^|]*\|(.*)\|", line.strip())
        if match:
            rows.setdefault(match[1], []).append((int(match[2]), match[3]))
    return rows


def protocol_table_problems(text: str, protocols: dict) -> list[str]:
    rows, problems = protocol_table(text), []
    for ident, protocol in protocols.items():
        found = rows.get(ident, [])
        if len(found) != 1 or found[0][0] != protocol["version"]:
            problems.append(f"{ident}: expected one row for v{protocol['version']}, found {found}")
            continue
        reasons, image = found[0][1], protocol["image"]
        if image["kernel_feature_present"]:
            if ("host parsing only" in reasons or "no native" in reasons
                    or re.search(rf"`{re.escape(image['kernel_feature'])}`[^;|]*absent", reasons)):
                problems.append(f"{ident}: row denies the present kernel feature {image['kernel_feature']}")
        elif "absent" not in reasons and "no " not in reasons:
            problems.append(f"{ident}: row does not state that {image['kernel_feature']} is absent")
    if set(rows) - set(protocols):
        problems.append(f"unknown protocol rows {sorted(set(rows) - set(protocols))}")
    return problems


class Fixture:
    """Temporary repository root with copied catalogue, a selected HW-FR target and one ready protocol.

    Marking a protocol ready also decides (with a synthetic reference) every owner decision that blocks it and
    adds its kernel feature to the temporary kernel manifest when absent.
    """

    def __init__(self, root: Path, protocol: str = "H05-S5"):
        self.root, self.protocol_id = root, protocol
        for name in (hw_evidence.PROTOCOLS, hw_evidence.KERNEL_MANIFEST, hw_evidence.SIGNERS, hw_evidence.MATRIX):
            (root / name).parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(hw_evidence.ROOT / name, root / name)
        self.protocols = json.loads((root / hw_evidence.PROTOCOLS).read_text())
        self.matrix = json.loads((root / hw_evidence.MATRIX).read_text())
        self.hw_profile["target"] = TARGET
        self.write_matrix()
        self.ready(protocol)
        self.keys = Path(tempfile.mkdtemp(dir=root.parent))

    @property
    def hw_profile(self) -> dict:
        return next(p for p in self.matrix["profiles"] if p["id"] == "HW-FR")

    @property
    def protocol(self) -> dict:
        return self.get(self.protocol_id)

    def get(self, ident: str) -> dict:
        return next(p for p in self.protocols["protocols"] if p["id"] == ident)

    def decision(self, ident: str) -> dict:
        return next(d for d in self.matrix["owner_decisions"] if d["id"] == ident)

    def ready(self, ident: str):
        protocol = self.get(ident)
        protocol.update(status="ready", blocking_reasons=[], markers_implemented=True)
        protocol["image"]["physical_image"] = "available"
        if not protocol["image"]["kernel_feature_present"]:
            manifest = self.root / hw_evidence.KERNEL_MANIFEST
            feature = protocol["image"]["kernel_feature"]
            manifest.write_text(manifest.read_text().replace("[features]\n", f"[features]\n{feature} = []\n", 1))
            protocol["image"]["kernel_feature_present"] = True
        for decision in self.matrix["owner_decisions"]:
            if ident in decision["blocks"] or ident in hw_evidence.REQUIRED_BLOCKS[decision["id"]]:
                decision.update(status="decided", reference=FIXTURE_REFERENCE)
        self.write_protocols()
        self.write_matrix()

    def reopen(self, ident: str):
        decision = self.decision(ident)
        decision["status"] = "open"
        decision.pop("reference", None)
        self.write_matrix()

    def write_protocols(self):
        (self.root / hw_evidence.PROTOCOLS).write_text(json.dumps(self.protocols))

    def write_matrix(self):
        (self.root / hw_evidence.MATRIX).write_text(json.dumps(self.matrix))

    def keygen(self, name: str, keytype: str = "ed25519") -> Path:
        key = self.keys / name
        subprocess.run([SSH_KEYGEN, "-q", "-t", keytype, "-N", "", "-C", name, "-f", str(key)],
                       check=True, capture_output=True)
        return key

    def allow(self, entries: list[str]):
        (self.root / hw_evidence.SIGNERS).write_text("# test signers\n" + "".join(e + "\n" for e in entries))

    def allow_key(self, key: Path, principal: str = SIGNER, options: str = f'namespaces="{NAMESPACE}"'):
        keytype, blob = key.with_suffix(".pub").read_text().split()[:2]
        self.allow([f"{principal} {options} {keytype} {blob}".replace("  ", " ")])

    def serial_lines(self, ident: str | None = None, inventory: dict | None = None) -> list[str]:
        """A passing serial log; HWINV DMAR and PCI lines follow the inventory (default physical_inventory())."""
        inventory = physical_inventory() if inventory is None else inventory
        lines = []
        for marker in self.get(ident or self.protocol_id)["required_markers"]:
            if marker == hw_evidence.HWINV_PCI_COUNT_MARKER:
                lines += pci_lines(inventory)
            elif marker == hw_evidence.HWINV_DMAR_MARKER:
                lines.append(dmar_line(inventory))
            elif marker == "HWINV:CPUID:*":
                lines.append(cpuid_line(inventory))
            elif marker == "HWINV:SECUREBOOT=*":
                lines.append(secureboot_line(inventory))
            else:
                lines.append(WILDCARD_LINES.get(marker, marker[:-1] + "1") if marker.endswith("*") else marker)
        return lines

    @staticmethod
    def record_name(ident: str) -> str:
        return f"docs/verification/{ident.lower()}-fixture/hw-evidence.json"

    def write(self, name: str, data: bytes) -> dict:
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        return dict(path=name, sha256=hashlib.sha256(data).hexdigest())

    def record(self, *, ident: str | None = None, key: Path | None = None, namespace: str = NAMESPACE,
               name: str | None = None, inventory: dict | None = None, lines: list[str] | None = None,
               mutate=None, **changes) -> str:
        ident = ident or self.protocol_id
        spec = self.get(ident)
        if name is None:
            name = RECORD if ident == self.protocol_id else self.record_name(ident)
        directory = name.rsplit("/", 1)[0]
        inventory = physical_inventory() if inventory is None else inventory
        record = dict(
            schema=1, kind=NAMESPACE, protocol=dict(id=ident, version=spec["version"]),
            profile="HW-FR", target=TARGET, devices=list(spec["devices"]),
            image=dict(name="baryonos-test.img", sha256="1" * 64), source_sha256="2" * 64,
            source_commit="3" * 40,
            inventory=self.write(f"{directory}/inventory.json", json.dumps(inventory).encode()),
            serial_log=self.write(f"{directory}/serial.log", "\n".join(
                self.serial_lines(ident, inventory) if lines is None else lines).encode() + b"\n"),
            secure_boot="disabled", operator="Test Operator", signer=SIGNER, date="2026-11-19",
            safety_confirmations={action: True for action in spec["forbidden_actions"]},
            steps=[dict(id=step["id"], result="pass") for step in spec["steps"]],
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

    def prerequisites(self, key: Path, ident: str | None = None, **changes) -> list[str]:
        """Mark ready and sign passing records of every prerequisite (same target, source and inventory)."""
        names = []
        for required in hw_evidence.PREREQUISITES[ident or self.protocol_id]:
            self.ready(required)
            names.append(self.record(ident=required, key=key, **changes))
        return names


def symlink(link: Path, target: str, testcase: unittest.TestCase):
    link.parent.mkdir(parents=True, exist_ok=True)
    try:
        os.symlink(target, link)
    except (OSError, NotImplementedError) as error:
        testcase.skipTest(f"symlinks unavailable: {error}")


class RepositoryStateTests(unittest.TestCase):
    """Runs without ssh-keygen: committed catalogue, signer file and format rules."""

    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name) / "repo"
        self.root.mkdir()
        self.fixture = Fixture(self.root)

    def test_repository_protocols_follow_owner_decisions_and_signers(self):
        """The live catalogue must agree with its recorded decisions; it is not required to stay blocked."""
        summary = hw_evidence.check()
        self.assertEqual(summary["protocols"], 7)
        protocols = hw_evidence.load_protocols()
        blockers = hw_evidence.open_blockers(json.loads((hw_evidence.ROOT / hw_evidence.MATRIX).read_text()))
        self.assertEqual(summary["ready"], sorted(i for i, p in protocols.items() if p["status"] == "ready"))
        self.assertEqual(summary["blocked"], sorted(i for i, p in protocols.items() if p["status"] == "blocked"))
        for ident, protocol in protocols.items():
            with self.subTest(ident=ident):
                if blockers[ident]:
                    self.assertEqual(protocol["status"], "blocked")
                if protocol["status"] == "blocked":
                    self.assertTrue(protocol["blocking_reasons"])
                else:
                    self.assertEqual((protocol["blocking_reasons"], blockers[ident]), ([], []))
        signers = hw_evidence.load_signers()
        self.assertIsNone(signer_state_problem(hw_evidence.ROOT))
        matrix = json.loads((hw_evidence.ROOT / hw_evidence.MATRIX).read_text())
        status = next(d for d in matrix["owner_decisions"] if d["id"] == "OD-HW-SIGNERS")["status"]
        if status == "open":
            self.assertEqual(summary["signers"], 0)
            self.assertEqual(signers, set())
            self.assertTrue(summary["signer_status"].startswith("none"))
        else:
            self.assertGreaterEqual(summary["signers"], 1)
            self.assertEqual(summary["signer_status"], "configured")
        for protocol in protocols.values():
            self.assertEqual(set(protocol["device_markers"]), set(protocol["devices"]))
            self.assertEqual(protocol["prerequisites"], list(hw_evidence.PREREQUISITES[protocol["id"]]))

    def test_all_open_decisions_keep_every_protocol_blocked(self):
        """Synthetic all-blocked state: no target, no signer, every owner decision open."""
        protocols = json.loads((self.root / hw_evidence.PROTOCOLS).read_text())
        for protocol in protocols["protocols"]:
            protocol["status"] = "blocked"
            protocol["blocking_reasons"] = protocol["blocking_reasons"] or ["synthetic block"]
        (self.root / hw_evidence.PROTOCOLS).write_text(json.dumps(protocols))
        self.fixture.hw_profile["target"] = None
        for decision in self.fixture.matrix["owner_decisions"]:
            decision["status"] = "open"
            decision.pop("reference", None)
        self.fixture.write_matrix()
        self.fixture.allow([])
        summary = hw_evidence.check(self.root)
        self.assertEqual((summary["ready"], summary["blocked"]), ([], sorted(hw_evidence.PROTOCOL_IDS)))
        self.assertEqual(summary["signers"], 0)
        self.assertTrue(summary["signer_status"].startswith("none"))
        blockers = hw_evidence.open_blockers(self.fixture.matrix)
        loaded = hw_evidence.load_protocols(self.root)
        for ident in hw_evidence.PROTOCOL_IDS:
            with self.subTest(ident=ident):
                self.assertGreaterEqual(set(blockers[ident]),
                                        {d for d, b in hw_evidence.REQUIRED_BLOCKS.items() if ident in b})
                self.assertEqual(loaded[ident]["status"], "blocked")
                self.assertTrue(loaded[ident]["blocking_reasons"])

    def test_signer_state_follows_owner_decision(self):
        self.fixture.reopen("OD-HW-SIGNERS")
        self.fixture.allow([f'{SIGNER} namespaces="{NAMESPACE}" ssh-ed25519 {fake_key()}'])
        self.assertIn("open", signer_state_problem(self.root))
        self.fixture.decision("OD-HW-SIGNERS").update(status="decided", reference=FIXTURE_REFERENCE)
        self.fixture.write_matrix()
        self.assertIsNone(signer_state_problem(self.root))
        self.fixture.allow([])
        self.assertIn("no signer", signer_state_problem(self.root))

    def test_empty_signer_file_rejects_every_record(self):
        self.fixture.allow([])
        name = self.fixture.record()
        (self.root / (name + ".sig")).write_text(fake_signature())
        with self.assertRaisesRegex(hw_evidence.EvidenceError, "no allowed signers.*OD-HW-SIGNERS"):
            hw_evidence.validate_record(name, self.root)
        with self.assertRaisesRegex(hw_evidence.EvidenceError, "rejected physical evidence"):
            hw_evidence.check(self.root)

    def test_comment_only_and_blank_signer_file_rejects(self):
        name = self.fixture.record()
        (self.root / (name + ".sig")).write_text(fake_signature())
        for text in ("", "\n\n   \n", "# only comments\n#  another\n", "\n# comment\n\t\n"):
            (self.root / hw_evidence.SIGNERS).write_text(text)
            with self.subTest(text=text):
                self.assertEqual(hw_evidence.load_signers(self.root), set())
                with self.assertRaisesRegex(hw_evidence.EvidenceError, "no allowed signers"):
                    hw_evidence.validate_record(name, self.root)

    def test_missing_ssh_keygen_fails_closed(self):
        self.fixture.allow([f'{SIGNER} namespaces="{NAMESPACE}" ssh-ed25519 {fake_key()}'])
        name = self.fixture.record()
        (self.root / (name + ".sig")).write_text(fake_signature())
        with unittest.mock.patch.object(hw_evidence, "ssh_keygen", return_value=None):
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

    def test_every_accepted_key_type_needs_its_exact_blob_layout(self):
        for keytype, blob in STRUCTURAL_KEYS.items():
            self.fixture.allow([f'{SIGNER} namespaces="{NAMESPACE}" {keytype} {blob}'])
            with self.subTest(keytype=keytype):
                self.assertEqual(hw_evidence.load_signers(self.root), {SIGNER})
        p256 = ec_point(65)
        for keytype, blob in (
                ("ssh-ed25519", key_blob(b"ssh-ed25519")),
                ("ssh-ed25519", key_blob(b"ssh-ed25519", b"\x01" * 31)),
                ("ssh-ed25519", key_blob(b"ssh-ed25519", b"\x01" * 33)),
                ("ssh-ed25519", base64.b64encode(base64.b64decode(fake_key()) + b"\x00").decode()),
                ("ssh-ed25519", key_blob(b"ssh-ed25519", b"\x01" * 32, b"")),
                ("ssh-ed25519", base64.b64encode(struct.pack(">I", 11) + b"ssh-ed25519" + b"\x00\x00").decode()),
                ("ssh-ed25519", base64.b64encode(struct.pack(">I", 11) + b"ssh-ed25519" + b"junk").decode()),
                ("ecdsa-sha2-nistp256", key_blob(b"ecdsa-sha2-nistp256")),
                ("ecdsa-sha2-nistp256", key_blob(b"ecdsa-sha2-nistp256", b"nistp384", p256)),
                ("ecdsa-sha2-nistp256", key_blob(b"ecdsa-sha2-nistp256", b"nistp256", ec_point(64))),
                ("ecdsa-sha2-nistp256", key_blob(b"ecdsa-sha2-nistp256", b"nistp256", b"\x02" + p256[1:])),
                ("ecdsa-sha2-nistp521", key_blob(b"ecdsa-sha2-nistp521", b"nistp521", ec_point(97))),
                ("sk-ssh-ed25519@openssh.com", key_blob(b"sk-ssh-ed25519@openssh.com", b"\x01" * 32)),
                ("sk-ssh-ed25519@openssh.com", key_blob(b"sk-ssh-ed25519@openssh.com", b"\x01" * 32, b"")),
                ("sk-ecdsa-sha2-nistp256@openssh.com",
                 key_blob(b"sk-ecdsa-sha2-nistp256@openssh.com", b"nistp256", p256))):
            self.fixture.allow([f'{SIGNER} namespaces="{NAMESPACE}" {keytype} {blob}'])
            with self.subTest(keytype=keytype, blob=blob), self.assertRaisesRegex(hw_evidence.EvidenceError,
                                                                                  "key blob"):
                hw_evidence.load_signers(self.root)

    def test_malformed_signer_key_is_not_reported_configured(self):
        self.fixture.allow([f'{SIGNER} namespaces="{NAMESPACE}" ssh-ed25519 {key_blob(b"ssh-ed25519")}'])
        with self.assertRaisesRegex(hw_evidence.EvidenceError, "key blob"):
            hw_evidence.check(self.root)

    @unittest.skipUnless(SSH_KEYGEN, NO_SSH)
    def test_real_openssh_public_keys_are_accepted(self):
        for keytype in ("ed25519", "ecdsa"):
            key = self.fixture.keygen(f"real-{keytype}", keytype)
            self.fixture.allow_key(key)
            with self.subTest(keytype=keytype):
                self.assertEqual(hw_evidence.load_signers(self.root), {SIGNER})

    def test_repo_file_rejects_symlink_aliases(self):
        target = self.root / "services" / "evil" / "file.json"
        target.parent.mkdir(parents=True)
        target.write_text("{}")
        symlink(self.root / "docs/verification/link/file.json", "../../../services/evil/file.json", self)
        symlink(self.root / "docs/verification/dir", "../../services/evil", self)
        self.assertEqual(hw_evidence.repo_file(self.root.resolve(), "services/evil/file.json"), target.resolve())
        for name in ("docs/verification/link/file.json", "docs/verification/dir/file.json"):
            with self.subTest(name=name), self.assertRaisesRegex(hw_evidence.EvidenceError, "symlink or aliased"):
                hw_evidence.repo_file(self.root.resolve(), name)

    def test_protocol_catalogue_drift_is_rejected(self):
        hw_evidence.load_protocols(self.root)
        original = copy.deepcopy(self.fixture.protocols)

        def protocol(doc, ident):
            return next(p for p in doc["protocols"] if p["id"] == ident)

        def share_marker(doc):
            protocol(doc, "H03-AV")["device_markers"]["hda"].append("G0:READY")

        def device_markers(ident, **markers):
            return lambda doc: protocol(doc, ident)["device_markers"].update(markers)

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
                (lambda d: d["record_fields"].append("extra"), "record_fields"),
                (lambda d: protocol(d, "H05-S3").update(prerequisites=["HW-INV"]), "prerequisites differ"),
                (lambda d: protocol(d, "R03-INSTALL").pop("prerequisites"), "unknown or missing keys"),
                (lambda d: protocol(d, "R03-INSTALL").pop("device_markers"), "unknown or missing keys"),
                (lambda d: protocol(d, "R03-INSTALL")["device_markers"].pop("nvme"), "device_markers must name"),
                (lambda d: protocol(d, "H04-NET")["device_markers"].update(e1000e=[]), "device_markers"),
                (share_marker, "shared between devices"),
                # Wildcard overlap between devices, in both directions and against exact markers.
                (device_markers("R03-INSTALL", nvme=["INSTALL:TARGET:*"]), "overlaps.*shared between devices"),
                (device_markers("R03-INSTALL", ahci=["INSTALL:TARGET:*"]), "overlaps.*shared between devices"),
                (device_markers("R03-INSTALL", ahci=["INSTALL:TARGET:NVME:0000*"]), "overlaps.*shared between"),
                (device_markers("R03-INSTALL", ahci=["INSTALL:TARGET:NVME:0000:01:00.0"]), "overlaps.*shared"),
                (device_markers("H03-AV", hda=["G0:READ*"]), "overlaps.*shared between devices"),
                (device_markers("H03-AV", hda=["G0:FRAMEBUFFER:1024X768"]), "overlaps.*shared between devices"),
                (device_markers("H03-AV", hda=["HDA:PLAYBACK:PASS", "HDA:PLAYBACK:PASS"]), "shared between"),
                (lambda d: protocol(d, "H05-S5")["device_markers"].update(acpi=["BOOT:EBS_OK"]), "generic marker"),
                (lambda d: protocol(d, "H05-S5")["device_markers"].update(acpi=["BOOT*"]), "generic marker"),
                (lambda d: protocol(d, "H02-USB")["device_markers"].update(xhci=["xhci pass"]),
                 "invalid device marker"),
                (lambda d: protocol(d, "HW-INV")["required_markers"].remove("HWINV:ACPI:DMAR=*"),
                 "HWINV:ACPI:DMAR"),
                (lambda d: d["hwinv_marker_format"].remove(hw_evidence.HWINV_DMAR_FORMAT), "hwinv_marker_format"),
                (lambda d: d["hwinv_marker_format"].remove(hw_evidence.HWINV_PCI_FORMATS[0]), "hwinv_marker_format"),
                (lambda d: d["hwinv_marker_format"].remove(hw_evidence.HWINV_PCI_FORMATS[1]), "hwinv_marker_format"),
                (lambda d: protocol(d, "HW-INV")["required_markers"].remove("HWINV:PCI:COUNT=*"),
                 "HWINV:PCI:COUNT")):
            doc = copy.deepcopy(original)
            mutate(doc)
            self.fixture.protocols = doc
            self.fixture.write_protocols()
            with self.subTest(reason=reason), self.assertRaisesRegex(hw_evidence.EvidenceError, reason):
                hw_evidence.load_protocols(self.root)

    def test_marker_overlap_is_language_intersection(self):
        for first, second, overlap in (
                ("INSTALL:TARGET:*", "INSTALL:TARGET:AHCI:*", True),
                ("INSTALL:TARGET:NVME:*", "INSTALL:TARGET:AHCI:*", False),
                ("INSTALL:TARGET:NVME:*", "INSTALL:TARGET:NVMEX", False),
                ("INSTALL:TARGET:NVME:*", "INSTALL:TARGET:NVME:", True),
                ("G0:READY", "G0:READ*", True),
                ("G0:READY", "G0:READY", True),
                ("G0:READY", "G0:READ", False),
                ("G0:READ*", "G0:READING*", True),
                ("POWER:S5:PM1A=*", "POWER:S5:WRITE", False)):
            with self.subTest(first=first, second=second):
                self.assertEqual(hw_evidence._overlap(first, second), overlap)
                self.assertEqual(hw_evidence._overlap(second, first), overlap)
        # Distinct devices with disjoint patterns stay accepted.
        protocols = copy.deepcopy(self.fixture.protocols)
        install = next(p for p in protocols["protocols"] if p["id"] == "R03-INSTALL")
        install["device_markers"].update(nvme=["INSTALL:TARGET:NVME:*", "NVME:READY"], ahci=["INSTALL:TARGET:AHCI:*"])
        self.fixture.protocols = protocols
        self.fixture.write_protocols()
        hw_evidence.load_protocols(self.root)

    def test_ready_protocol_rejected_while_blocking_decision_open(self):
        hw_evidence.load_protocols(self.root)
        committed = copy.deepcopy(self.fixture.matrix)
        blocking = sorted(d["id"] for d in committed["owner_decisions"] if "H05-S5" in d["blocks"])
        self.assertEqual(blocking, sorted(i for i, b in hw_evidence.REQUIRED_BLOCKS.items() if "H05-S5" in b))

        def reopen(ident):
            def mutate(matrix):
                decision = next(d for d in matrix["owner_decisions"] if d["id"] == ident)
                decision["status"] = "open"
                decision.pop("reference")
            return mutate

        def shrink_boot(matrix):
            reopen("OD-HW-BOOT")(matrix)
            next(d for d in matrix["owner_decisions"] if d["id"] == "OD-HW-BOOT")["blocks"].remove("H05-S5")

        def duplicate(matrix):
            matrix["owner_decisions"].append(copy.deepcopy(matrix["owner_decisions"][3]))

        cases = [(reopen(ident), ident) for ident in blocking] + [
            (shrink_boot, "OD-HW-BOOT"),
            (lambda m: m.pop("owner_decisions"), "owner decisions not recorded"),
            (lambda m: next(d for d in m["owner_decisions"] if d["id"] == "OD-HW-CAPTURE").update(reference=" "),
             "OD-HW-CAPTURE"),
            (lambda m: next(d for d in m["owner_decisions"] if d["id"] == "OD-HW-PUBLISH").pop("reference"),
             "OD-HW-PUBLISH"),
            (lambda m: next(d for d in m["owner_decisions"] if d["id"] == "OD-HW-DISKS").update(status="done"),
             "OD-HW-DISKS"),
            (duplicate, committed["owner_decisions"][3]["id"]),
            (lambda m: m["owner_decisions"].append(dict(id="OD-HW-UNKNOWN", status="decided", reference="x")),
             "OD-HW-UNKNOWN"),
            (lambda m: m["owner_decisions"].pop(1), committed["owner_decisions"][1]["id"]),
            # A decided entry clears its blockers only when it is a valid matrix entry (hw_matrix rules).
            (lambda m: next(d for d in m["owner_decisions"] if d["id"] == "OD-HW-BOOT").pop("blocks"),
             "OD-HW-BOOT"),
            (lambda m: next(d for d in m["owner_decisions"] if d["id"] == "OD-HW-BOOT")["blocks"].remove("H05-S5"),
             "OD-HW-BOOT"),
            (lambda m: next(d for d in m["owner_decisions"] if d["id"] == "OD-HW-SIGNERS").update(blocks=["H05-S5"]),
             "OD-HW-SIGNERS"),
            (lambda m: next(d for d in m["owner_decisions"] if d["id"] == "OD-HW-CAPTURE").pop("decision"),
             "OD-HW-CAPTURE"),
            (lambda m: next(d for d in m["owner_decisions"] if d["id"] == "OD-HW-TARGETS").update(extra=True),
             "OD-HW-TARGETS"),
            (lambda m: next(d for d in m["owner_decisions"] if d["id"] == "OD-HW-PUBLISH").update(
                blocks=[*hw_evidence.PROTOCOL_IDS, "H05-S5"]), "OD-HW-PUBLISH")]
        for mutate, reason in cases:
            matrix = copy.deepcopy(committed)
            mutate(matrix)
            (self.root / hw_evidence.MATRIX).write_text(json.dumps(matrix))
            with self.subTest(reason=reason), self.assertRaisesRegex(
                    hw_evidence.EvidenceError, f"H05-S5: ready while owner decision.*{re.escape(reason)}"):
                hw_evidence.load_protocols(self.root)
        # A protocol that is not ready takes no owner-decision check: the committed state loads unchanged.
        self.assertEqual(sorted(p["status"] for p in hw_evidence.load_protocols().values()), ["blocked"] * 7)

    def test_outcome_rejects_unexpected_restarts(self):
        protocols = hw_evidence.load_protocols()
        passing = {}
        for ident, protocol in protocols.items():
            lines = self.fixture.serial_lines(ident)
            steps = [dict(id=step["id"], result="pass") for step in protocol["steps"]]
            passing[ident] = (protocol, lines, steps)
            with self.subTest(ident=ident):
                self.assertIsNone(hw_evidence.outcome(protocol, lines, steps))
                self.assertRegex(hw_evidence.outcome(protocol, lines + ["BOOT:EBS_OK"], steps), "unexpected restart")
        protocol, lines, steps = passing["H05-S5"]
        for bad in (lines + ["BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK"], ["BOOT:EBS_OK"] + lines, lines + lines):
            with self.subTest(case=bad):
                self.assertRegex(hw_evidence.outcome(protocol, bad, steps), "appears 2 time.*exactly 1")
        protocol, lines, steps = passing["H05-S3"]
        enter = lines.index("POWER:S3:ENTER")
        rebooted = lines[:enter + 1] + lines
        self.assertRegex(hw_evidence.outcome(protocol, rebooted, steps), "exactly 1 .*unexpected restart")
        protocol, lines, steps = passing["R03-INSTALL"]
        self.assertEqual(lines.count("BOOT:EBS_OK"), 2)
        self.assertRegex(hw_evidence.outcome(protocol, lines + ["BOOT:EBS_OK"], steps), "exactly 2")

    def test_restart_marker_counts_after_firmware_output_on_the_same_line(self):
        """The loader writes BOOT:EBS_OK without a preceding newline, so firmware bytes can share its line."""
        protocols = hw_evidence.load_protocols()
        for ident in ("H05-S5", "R03-INSTALL", "HW-INV"):
            protocol, lines = protocols[ident], self.fixture.serial_lines(ident)
            steps = [dict(id=step["id"], result="pass") for step in protocol["steps"]]
            expected = lines.count("BOOT:EBS_OK")
            for hidden in (FIRMWARE + "BOOT:EBS_OK", "KERNEL:ENTRYBOOT:EBS_OK", FIRMWARE + "BOOT:EBS_OKBOOT:EBS_OK"):
                with self.subTest(ident=ident, hidden=hidden):
                    self.assertRegex(hw_evidence.outcome(protocol, lines + [hidden], steps),
                                     f"appears {expected + hidden.count('BOOT:EBS_OK')} time.*exactly {expected} "
                                     ".*unexpected restart")
        protocol, lines = protocols["H05-S5"], self.fixture.serial_lines("H05-S5")
        steps = [dict(id=step["id"], result="pass") for step in protocol["steps"]]
        write = lines.index("POWER:S5:WRITE")
        reset = lines[:write + 1] + [FIRMWARE + "BOOT:EBS_OK"] + lines[1:]
        self.assertRegex(hw_evidence.outcome(protocol, reset, steps), "appears 2 time.*exactly 1")

    def test_single_prefixed_boot_satisfies_the_ordered_markers(self):
        """The loader's BOOT:EBS_OK can follow firmware output on its line; only a line suffix counts as a boot."""
        protocols = hw_evidence.load_protocols()
        for ident in ("H05-S5", "R03-INSTALL", "HW-INV"):
            protocol, lines = protocols[ident], self.fixture.serial_lines(ident)
            steps = [dict(id=step["id"], result="pass") for step in protocol["steps"]]
            prefixed = [FIRMWARE + line if line == "BOOT:EBS_OK" else line for line in lines]
            with self.subTest(ident=ident):
                self.assertIsNone(hw_evidence.outcome(protocol, prefixed, steps))
                self.assertRegex(hw_evidence.outcome(protocol, prefixed + [FIRMWARE + "BOOT:EBS_OK"], steps),
                                 "unexpected restart")
                # The marker must end its line; trailing bytes are not a boot.
                trailing = [line + " x" if line == "BOOT:EBS_OK" else line for line in lines]
                self.assertRegex(hw_evidence.outcome(protocol, trailing, steps), "'BOOT:EBS_OK' missing")

    def test_inventory_requires_acpi_dmar_observation(self):
        hw_evidence._inventory(json.dumps(physical_inventory()).encode())
        no_acpi = physical_inventory()
        no_acpi.pop("acpi")
        for inventory, reason in (
                (no_acpi, "unknown format"),
                (physical_inventory(schema=1), "unknown format"),
                (physical_inventory(acpi=dict(dmar="maybe", drhd_units=1)), "inventory acpi"),
                (physical_inventory(acpi=dict(dmar="present", drhd_units=True)), "inventory acpi"),
                (physical_inventory(acpi=dict(dmar="present", drhd_units=-1)), "inventory acpi"),
                (physical_inventory(acpi=dict(dmar="present", drhd_units=65)), "inventory acpi"),
                (physical_inventory(acpi=dict(dmar="absent", drhd_units=1)), "inventory acpi"),
                (physical_inventory(acpi=dict(dmar="present")), "inventory acpi"),
                (physical_inventory(acpi=dict(dmar="present", drhd_units=1, vtd="contained")), "inventory acpi"),
                (physical_inventory(pci=physical_inventory()["pci"] + [dict(physical_inventory()["pci"][1],
                                                                            device="7ae1")]), "PCI address twice")):
            with self.subTest(inventory=inventory.get("acpi"), schema=inventory["schema"]):
                with self.assertRaisesRegex(hw_evidence.EvidenceError, reason):
                    hw_evidence._inventory(json.dumps(inventory).encode())

    def test_h03_av_listening_precedes_playback_completion(self):
        protocol = hw_evidence.load_protocols()["H03-AV"]
        self.assertEqual((protocol["version"], protocol["status"], protocol["markers_implemented"]),
                         (2, "blocked", False))
        markers = protocol["required_markers"]
        self.assertLess(markers.index("HDA:PLAYBACK:START:*"), markers.index("HDA:PLAYBACK:PASS"))
        steps = protocol["steps"]

        def index(actor, *words):
            return next(i for i, s in enumerate(steps) if s["actor"] == actor and all(w in s["action"] for w in words))
        listen = index("operator", "listen")
        start = index("serial", "for HDA:PLAYBACK:START")
        done = index("serial", "for HDA:PLAYBACK:PASS")
        self.assertLess(listen, start)
        self.assertLess(start, done)
        for step in steps[done + 1:]:
            if step["actor"] == "operator":
                self.assertNotRegex(step["action"].lower(), "listen|during")
        self.assertGreater(index("operator", "photograph"), done)

    def test_hardware_targets_protocol_table_matches_catalogue(self):
        text = (hw_evidence.ROOT / "docs/drivers/HARDWARE-TARGETS.md").read_text()
        protocols = hw_evidence.load_protocols()
        self.assertTrue(all(p["status"] == "blocked" for p in protocols.values()))
        self.assertEqual(protocol_table_problems(text, protocols), [])
        stale = re.sub(r"\| `H05-S3` v1 \|([^|]*)\|[^\n]*\|",
                       r"| `H05-S3` v1 |\1| no native S3 entry/resume path (host parsing only); no wake source |", text)
        self.assertNotEqual(stale, text)
        self.assertRegex(" ".join(protocol_table_problems(stale, protocols)), "H05-S3: row denies")
        old_version = text.replace("| `H03-AV` v2 |", "| `H03-AV` v1 |")
        self.assertRegex(" ".join(protocol_table_problems(old_version, protocols)), "H03-AV: expected one row")

    def test_pci_log_must_equal_inventory(self):
        inventory = physical_inventory()
        lines = pci_lines(inventory)
        functions, count = lines[:-1], lines[-1]
        extra = dict(address="0000:00:1f.3", vendor="8086", device="7ad0", **{"class": "04"}, subclass="03",
                     prog_if="00", revision="11")
        bigger = pci_lines(physical_inventory(pci=inventory["pci"] + [extra]))
        for required in (True, False):
            self.assertIsNone(hw_evidence.pci_log_problem(["M1:PASS", *lines], inventory["pci"], required=required))
        self.assertIsNone(hw_evidence.pci_log_problem(["M1:PASS"], inventory["pci"], required=False))
        for label, log, reason in (
                ("no lines", ["M1:PASS"], "0 HWINV:PCI:COUNT"),
                ("missing function", functions[1:] + ["HWINV:PCI:COUNT=2"], "absent from the log: 0000:00:00.0;"),
                ("missing count", functions, "0 HWINV:PCI:COUNT"),
                ("two counts", lines + [count], "2 HWINV:PCI:COUNT"),
                ("extra log entry", bigger, "absent from the log: none; absent from the inventory: 0000:00:1f.3"),
                ("count too high", functions + ["HWINV:PCI:COUNT=3"], "COUNT=3 differs from the 2"),
                ("count too low", functions + ["HWINV:PCI:COUNT=1"], "COUNT=1 differs from the 2"),
                ("duplicate line", [functions[0], *lines], "twice"),
                ("changed field", [functions[0], functions[1].replace(":7ae0:", ":7ae1:"), count], "differ"),
                ("upper-case hex", [functions[0], functions[1].upper(), count], "malformed"),
                ("firmware prefix", [functions[0], FIRMWARE + functions[1], count], "malformed"),
                ("bad count", lines[:-1] + ["HWINV:PCI:COUNT=02"], "malformed")):
            with self.subTest(case=label):
                self.assertRegex(hw_evidence.pci_log_problem(log, inventory["pci"], required=True) or "", reason)
        # An inventory entry the log never listed is reported, whether or not the protocol requires the listing.
        larger = inventory["pci"] + [extra]
        for required in (True, False):
            self.assertRegex(hw_evidence.pci_log_problem(lines, larger, required=required) or "",
                             "absent from the log: 0000:00:1f.3;")

    def test_preconditions_state_the_full_prerequisite_binding(self):
        """Catalogue text matches the validator: target, source_sha256, source_commit and inventory SHA-256."""
        binding = "with the same target, source_sha256, source_commit and inventory SHA-256 as this record"
        for ident, protocol in hw_evidence.load_protocols().items():
            required = hw_evidence.PREREQUISITES[ident]
            stated = [text for text in protocol["preconditions"]
                      if binding in text and all(prerequisite in text for prerequisite in required)]
            with self.subTest(ident=ident):
                self.assertEqual(len(stated), 1 if required else 0)
                for text in protocol["preconditions"]:
                    self.assertNotRegex(text, r"same target(,| and) source( and|$|\.)")

    def test_preconditions_name_their_blocking_decisions(self):
        for ident, protocol in hw_evidence.load_protocols().items():
            named = set(re.findall(r"OD-HW-[A-Z-]+[A-Z]", protocol["preconditions"][0]))
            required = {d for d, blocks in hw_evidence.REQUIRED_BLOCKS.items() if ident in blocks}
            with self.subTest(ident=ident):
                self.assertEqual(named, required)


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
        self.prerequisites = self.fixture.prerequisites(self.key)

    def signed(self, **changes) -> str:
        return self.fixture.record(key=self.key, **changes)

    def rejects(self, name: str, reason: str):
        with self.assertRaisesRegex(hw_evidence.EvidenceError, reason):
            hw_evidence.validate_record(name, self.root)

    def test_valid_signed_record_is_accepted(self):
        summary = hw_evidence.validate_record(self.signed(), self.root)
        self.assertEqual((summary["protocol"], summary["target"], summary["devices"], summary["passed"]),
                         ("H05-S5", TARGET, ["acpi"], True))
        self.assertEqual((summary["source_sha256"], summary["source_commit"]), ("2" * 64, "3" * 40))
        self.assertEqual(sorted(s["protocol"] for s in hw_evidence.check(self.root)["accepted"]),
                         ["H05-S5", "HW-INV"])

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

    def test_open_decision_rejects_signed_record(self):
        name = self.signed()
        hw_evidence.validate_record(name, self.root)
        self.fixture.reopen("OD-HW-BOOT")
        self.rejects(name, "ready while owner decision.*OD-HW-BOOT")
        with self.assertRaisesRegex(hw_evidence.EvidenceError, "OD-HW-BOOT"):
            hw_evidence.check(self.root)

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
        # A reset instead of a power-off shows as a second boot after POWER:S5:WRITE.
        self.rejects(self.signed(lines=lines + ["BOOT:EBS_OK"]), "relabelled as passed: BOOT:EBS_OK appears 2")
        self.rejects(self.signed(lines=lines + lines), "exactly 1")
        self.rejects(self.signed(lines=["BOOT:EBS_OK"] + lines), "exactly 1")
        # A reset whose marker follows firmware output on the same line, as the loader writes it without a newline.
        self.rejects(self.signed(lines=lines + [FIRMWARE + "BOOT:EBS_OK", "KERNEL:ENTRY"]),
                     "relabelled as passed: BOOT:EBS_OK appears 2 time.*exactly 1")
        log = self.root / RECORD_DIR / "serial.log"
        self.assertIn((FIRMWARE + "BOOT:EBS_OK\n").encode(), log.read_bytes())

        def fail_serial_step(record):
            record["steps"][3]["result"] = "fail"
        honest = hw_evidence.validate_record(
            self.signed(lines=lines + ["BOOT:EBS_OK", "KERNEL:ENTRY"], passed=False, mutate=fail_serial_step),
            self.root)
        self.assertFalse(honest["passed"])

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

    def test_symlinked_records_are_rejected(self):
        outside = "services/evil/hw-evidence.json"
        self.signed(name=outside)
        symlink(self.root / "docs/verification/alias", "../../services/evil", self)
        self.rejects("docs/verification/alias/hw-evidence.json", "symlink or aliased")
        symlink(self.root / "docs/verification/real-dir/hw-evidence.json", "../../../services/evil/hw-evidence.json",
                self)
        self.rejects("docs/verification/real-dir/hw-evidence.json", "symlink or aliased")
        with self.assertRaisesRegex(hw_evidence.EvidenceError, "rejected physical evidence.*symlink or aliased"):
            hw_evidence.check(self.root)
        (self.root / "docs/verification/real-dir/hw-evidence.json").unlink()
        (self.root / "docs/verification/alias").unlink()
        name = self.signed()
        self.assertEqual(len(hw_evidence.check(self.root)["accepted"]), 2)
        # One signed record counted twice through an alias inside docs/verification.
        symlink(self.root / "docs/verification/copy/hw-evidence.json", "../hw-fixture/hw-evidence.json", self)
        with self.assertRaisesRegex(hw_evidence.EvidenceError, "symlink or aliased"):
            hw_evidence.check(self.root)
        (self.root / "docs/verification/copy/hw-evidence.json").unlink()
        # A canonical record whose detached signature is a symlink to a valid signature elsewhere.
        signature = self.root / (name + ".sig")
        moved = self.root / "services/sig/hw-evidence.json.sig"
        moved.parent.mkdir(parents=True)
        signature.rename(moved)
        symlink(signature, "../../../services/sig/hw-evidence.json.sig", self)
        self.rejects(name, "symlink or aliased")
        # An attachment inside the record directory that is a symlink to a file elsewhere.
        name = self.signed()
        inventory = self.root / RECORD_DIR / "inventory.json"
        relocated = self.root / "services/x/inventory.json"
        relocated.parent.mkdir(parents=True)
        inventory.rename(relocated)
        symlink(inventory, "../../../services/x/inventory.json", self)
        self.rejects(name, "symlink or aliased")

    def test_s5_requires_accepted_hw_inv_record(self):
        inv = self.prerequisites[0]
        name = self.signed()
        hw_evidence.validate_record(name, self.root)
        unmet = "precondition unmet: no accepted, passing HW-INV record"

        def restore():
            self.fixture.get("HW-INV")["version"] = 2
            self.fixture.ready("HW-INV")
            self.fixture.record(ident="HW-INV", key=self.key)
            self.assertEqual(hw_evidence.validate_record(name, self.root)["passed"], True)

        def fail_step(record):
            record["steps"][3]["result"] = "fail"
        cases = (
            ("no record", lambda: (self.root / inv).unlink()),
            ("unsigned", lambda: (self.root / (inv + ".sig")).unlink()),
            ("tampered", lambda: (self.root / inv).write_text((self.root / inv).read_text() + " ")),
            ("failed", lambda: self.fixture.record(ident="HW-INV", key=self.key, passed=False, mutate=fail_step)),
            ("source hash", lambda: self.fixture.record(ident="HW-INV", key=self.key, source_sha256="4" * 64)),
            ("source commit", lambda: self.fixture.record(ident="HW-INV", key=self.key, source_commit="5" * 40)),
            ("inventory", lambda: self.fixture.record(ident="HW-INV", key=self.key, inventory=physical_inventory(
                pci=physical_inventory()["pci"][1:]))),
            ("blocked", lambda: (self.fixture.get("HW-INV").update(status="blocked", blocking_reasons=["test"]),
                                 self.fixture.write_protocols())),
            ("version", lambda: (self.fixture.get("HW-INV").update(version=3), self.fixture.write_protocols())))
        for label, mutate in cases:
            mutate()
            with self.subTest(case=label):
                self.rejects(name, unmet)
            restore()
        # The prerequisite applies whatever the dependent record claims.
        (self.root / inv).unlink()

        def fail_s5(record):
            record["steps"][4]["result"] = "fail"
        self.rejects(self.signed(passed=False, mutate=fail_s5), unmet)

    def test_s5_from_a_qemu_capture_needs_a_physical_inventory_record(self):
        report = json.loads((hw_evidence.ROOT / "docs/verification/h05-acpi-s5/native-report.json").read_text())
        stdout = report["profiles"]["native-acpi-s5"]["runs"][0]["guest"]["stdout"]
        lines = [line.strip() for line in stdout.splitlines()]
        self.assertIn("ACPI:UNAVAILABLE:NO_DMAR", lines)
        # The QEMU log reports no DMAR table, so it contradicts the default inventory (DMAR present) ...
        self.rejects(self.signed(lines=lines), "kernel ACPI marker ACPI:UNAVAILABLE:NO_DMAR .*contradicts")
        # ... and with an agreeing inventory it still lacks an accepted HW-INV record of a physical machine.
        (self.root / self.prerequisites[0]).unlink()
        name = self.signed(lines=lines, inventory=physical_inventory(acpi=dict(dmar="absent", drhd_units=0)))
        self.rejects(name, "precondition unmet.*HW-INV")

    def test_kernel_acpi_marker_must_agree_with_inventory(self):
        lines = self.fixture.serial_lines()
        absent = physical_inventory(acpi=dict(dmar="absent", drhd_units=0))
        empty = physical_inventory(acpi=dict(dmar="present", drhd_units=0))
        present = physical_inventory()
        two = physical_inventory(acpi=dict(dmar="present", drhd_units=2))
        for marker, inventory, accepted in (
                ("ACPI:UNAVAILABLE:NO_DMAR", absent, True),
                ("ACPI:UNAVAILABLE:NO_DMAR", present, False),
                ("ACPI:UNAVAILABLE:NO_DMAR", empty, False),
                ("ACPI:UNAVAILABLE:NO_DMAR_UNITS", empty, True),
                ("ACPI:UNAVAILABLE:NO_DMAR_UNITS", absent, False),
                ("ACPI:UNAVAILABLE:NO_DMAR_UNITS", present, False),
                ("ACPI:DISCOVERY:MCFG_DMAR", present, True),
                ("ACPI:DISCOVERY:MCFG_DMAR", two, True),
                ("ACPI:DISCOVERY:MCFG_DMAR", absent, False),
                ("ACPI:DISCOVERY:MCFG_DMAR", empty, False),
                # Firmware bytes before the marker do not hide it.
                (FIRMWARE + "ACPI:UNAVAILABLE:NO_DMAR", present, False)):
            # Prerequisites are re-signed with the same inventory, so only the marker decides.
            self.fixture.prerequisites(self.key, inventory=inventory)
            log = lines[:5] + [marker, "DEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT"] + lines[5:]
            name = self.signed(lines=log, inventory=inventory)
            with self.subTest(marker=marker, acpi=inventory["acpi"]):
                if accepted:
                    self.assertTrue(hw_evidence.validate_record(name, self.root)["passed"])
                else:
                    self.rejects(name, "kernel ACPI marker ACPI:.*contradicts the inventory")
        # A failed run is held to the same rule, and so is a second boot that reports another ACPI outcome.
        self.fixture.prerequisites(self.key, inventory=present)

        def fail_step(record):
            record["steps"][3]["result"] = "fail"
        self.rejects(self.signed(lines=lines + ["ACPI:UNAVAILABLE:NO_DMAR"], passed=False, mutate=fail_step),
                     "kernel ACPI marker ACPI:UNAVAILABLE:NO_DMAR .*contradicts")
        self.rejects(self.signed(lines=["ACPI:DISCOVERY:MCFG_DMAR", *lines, "ACPI:UNAVAILABLE:NO_DMAR_UNITS"],
                                 passed=False, mutate=fail_step), "NO_DMAR_UNITS .*contradicts")

    def test_hwinv_pci_lines_in_other_protocol_logs_must_match(self):
        lines = self.fixture.serial_lines()
        listing = pci_lines(physical_inventory())
        self.assertTrue(hw_evidence.validate_record(self.signed(lines=lines + listing), self.root)["passed"])
        self.rejects(self.signed(lines=lines + listing[1:]), "not bound to the serial log.*absent from the log")
        self.rejects(self.signed(lines=lines + [listing[-1]]), "not bound to the serial log.*absent from the log")

    def test_prerequisite_cycle_is_reported(self):
        self.fixture.get("HW-INV")["prerequisites"] = ["H05-S5"]
        self.fixture.write_protocols()
        name = self.signed()
        with unittest.mock.patch.dict(hw_evidence.PREREQUISITES, {"HW-INV": ("H05-S5",)}):
            self.rejects(name, "prerequisite cycle")
            with self.assertRaisesRegex(hw_evidence.EvidenceError, "prerequisite cycle"):
                hw_evidence.check(self.root)


@unittest.skipUnless(SSH_KEYGEN, NO_SSH)
class ProtocolEvidenceTests(unittest.TestCase):
    """Per-protocol rules on other ready protocols, each with signed prerequisite records."""

    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name) / "repo"
        self.root.mkdir()

    def use(self, protocol: str, prerequisites: bool = True) -> Fixture:
        self.fixture = Fixture(self.root, protocol)
        self.key = self.fixture.keygen(f"owner-{protocol.lower()}")
        self.fixture.allow_key(self.key)
        self.prerequisites = self.fixture.prerequisites(self.key) if prerequisites else []
        return self.fixture

    def signed(self, **changes) -> str:
        return self.fixture.record(key=self.key, **changes)

    def rejects(self, name: str, reason: str):
        with self.assertRaisesRegex(hw_evidence.EvidenceError, reason):
            hw_evidence.validate_record(name, self.root)

    def test_s3_requires_hw_inv_and_s5(self):
        self.use("H05-S3")
        self.assertEqual([n.split("/")[2] for n in self.prerequisites], ["hw-inv-fixture", "h05-s5-fixture"])
        name = self.signed()
        self.assertTrue(hw_evidence.validate_record(name, self.root)["passed"])
        (self.root / self.prerequisites[1]).unlink()
        self.rejects(name, "precondition unmet.*H05-S5")
        self.fixture.record(ident="H05-S5", key=self.key)
        (self.root / (self.prerequisites[0] + ".sig")).unlink()
        self.rejects(name, "precondition unmet.*HW-INV")
        lines = self.fixture.serial_lines()
        enter = lines.index("POWER:S3:ENTER")
        self.rejects(self.signed(lines=lines[:enter + 1] + lines), "relabelled as passed: BOOT:EBS_OK appears 2")

    def test_h03_av_playback_window(self):
        self.use("H03-AV")
        lines = self.fixture.serial_lines()
        self.assertTrue(hw_evidence.validate_record(self.signed(), self.root)["passed"])
        start = next(line for line in lines if line.startswith("HDA:PLAYBACK:START:"))
        without = [line for line in lines if line != start]
        self.rejects(self.signed(lines=without),
                     re.escape("relabelled as passed: required marker 'HDA:PLAYBACK:START:*' missing or out of order"))
        self.rejects(self.signed(lines=without + [start]), "missing or out of order")
        self.rejects(self.signed(protocol=dict(id="H03-AV", version=1)), "version mismatch")
        # The START marker must report the playback duration that sets the completion deadline.
        for bad in ("HDA:PLAYBACK:START:1", "HDA:PLAYBACK:START:MS=", "HDA:PLAYBACK:START:MS=0",
                    "HDA:PLAYBACK:START:MS=02000", "HDA:PLAYBACK:START:MS=2000:EXTRA", "HDA:PLAYBACK:START:MS=x"):
            with self.subTest(start=bad):
                self.rejects(self.signed(lines=[bad if line == start else line for line in lines]),
                             "malformed HDA:PLAYBACK:START marker")
        self.rejects(self.signed(lines=[*lines[:-1], "HDA:PLAYBACK:START:MS=1", lines[-1]]),
                     "malformed HDA:PLAYBACK:START marker|repeats")
        listen = next(i for i, s in enumerate(self.fixture.protocol["steps"]) if "listen" in s["action"])

        def unclear(record):
            record["steps"][listen]["result"] = "fail"
        # An honest failure still cannot name hda without its playback markers.
        self.rejects(self.signed(lines=without, passed=False, mutate=unclear),
                     "device hda listed without its transport marker HDA:PLAYBACK:START")
        self.assertEqual(hw_evidence.validate_record(self.signed(lines=without, devices=["gop"], passed=False,
                                                                 mutate=unclear), self.root)["devices"], ["gop"])
        self.rejects(self.signed(mutate=unclear), "relabelled as passed: step")
        self.assertFalse(hw_evidence.validate_record(self.signed(passed=False, mutate=unclear), self.root)["passed"])

    def test_serial_dmar_contradicts_inventory(self):
        self.use("HW-INV")
        lines = self.fixture.serial_lines()
        self.assertTrue(hw_evidence.validate_record(self.signed(), self.root)["passed"])
        dmar = lines.index(WILDCARD_LINES["HWINV:ACPI:DMAR=*"])
        for line, reason in (("HWINV:ACPI:DMAR=ABSENT:DRHD=0", "contradicts the inventory"),
                             ("HWINV:ACPI:DMAR=PRESENT:DRHD=2", "contradicts the inventory"),
                             ("HWINV:ACPI:DMAR=MAYBE:DRHD=1", "malformed DMAR"),
                             ("HWINV:ACPI:DMAR=PRESENT:DRHD=65", "malformed DMAR"),
                             ("HWINV:ACPI:DMAR=PRESENT:DRHD=01", "malformed DMAR")):
            changed = lines[:dmar] + [line] + lines[dmar + 1:]
            with self.subTest(line=line):
                self.rejects(self.signed(lines=changed), reason)
        self.rejects(self.signed(lines=lines[:dmar + 1] + lines[dmar:]), "repeats")
        absent = physical_inventory(acpi=dict(dmar="absent", drhd_units=0))
        changed = lines[:dmar] + ["HWINV:ACPI:DMAR=ABSENT:DRHD=0"] + lines[dmar + 1:]
        self.assertTrue(hw_evidence.validate_record(self.signed(lines=changed, inventory=absent), self.root)["passed"])

    def test_hw_inv_inventory_equals_serial_pci_lines(self):
        self.use("HW-INV")
        inventory = physical_inventory()
        lines = self.fixture.serial_lines()
        self.assertTrue(hw_evidence.validate_record(self.signed(), self.root)["passed"])
        first = lines.index(pci_lines(inventory)[0])
        count = lines.index("HWINV:PCI:COUNT=2")
        extra = dict(address="0000:00:1f.3", vendor="8086", device="7ad0", **{"class": "04"}, subclass="03",
                     prog_if="00", revision="11")
        bigger = physical_inventory(pci=inventory["pci"] + [extra])

        def fail_step(record):
            record["steps"][3]["result"] = "fail"
        for label, changes, reason in (
                ("missing line", dict(lines=lines[:first] + lines[first + 1:]), "absent from the log: 0000:00:00.0;"),
                ("no listing", dict(lines=lines[:first] + lines[count + 1:], passed=False, mutate=fail_step),
                 "0 HWINV:PCI:COUNT"),
                ("extra inventory entry", dict(inventory=bigger, lines=lines), "absent from the log: 0000:00:1f.3;"),
                ("extra log line", dict(lines=self.fixture.serial_lines(inventory=bigger)),
                 "absent from the log: none; absent from the inventory: 0000:00:1f.3"),
                ("count mismatch", dict(lines=lines[:count] + ["HWINV:PCI:COUNT=3"] + lines[count + 1:]),
                 "COUNT=3 differs"),
                ("duplicate line", dict(lines=lines[:first + 1] + lines[first:]), "twice"),
                ("malformed line", dict(lines=lines[:first + 1] + [lines[first + 1].upper()] + lines[first + 2:]),
                 "malformed HWINV:PCI")):
            with self.subTest(case=label):
                self.rejects(self.signed(**changes), f"inventory not bound to the serial log: .*{reason}")
        # A larger, agreeing listing is accepted.
        self.assertTrue(hw_evidence.validate_record(
            self.signed(inventory=bigger, lines=self.fixture.serial_lines(inventory=bigger)), self.root)["passed"])
        # The kernel's own ACPI outcome in the same log must agree with the DMAR observation.
        dmar = lines.index(WILDCARD_LINES["HWINV:ACPI:DMAR=*"])
        self.rejects(self.signed(lines=lines[:dmar] + ["ACPI:UNAVAILABLE:NO_DMAR"] + lines[dmar:]),
                     "kernel ACPI marker ACPI:UNAVAILABLE:NO_DMAR .*contradicts")

    def test_secure_boot_and_cpuid_bind_to_the_serial_log(self):
        self.use("HW-INV")
        lines = self.fixture.serial_lines()
        self.assertTrue(hw_evidence.validate_record(self.signed(), self.root)["passed"])
        secure = lines.index("HWINV:SECUREBOOT=0")
        cpuid = lines.index("HWINV:CPUID:VENDOR=GenuineIntel:HYPERVISOR=0")

        def at(index, *replacement):
            return lines[:index] + list(replacement) + lines[index + 1:]
        for label, changes, reason in (
                ("log enabled, record and inventory disabled", dict(lines=at(secure, "HWINV:SECUREBOOT=1")),
                 "Secure Boot"),
                ("log absent, inventory disabled", dict(lines=at(secure, "HWINV:SECUREBOOT=ABSENT")), "Secure Boot"),
                ("log disabled, inventory unknown",
                 dict(inventory=physical_inventory(secure_boot="unknown"), lines=lines), "Secure Boot"),
                ("repeated", dict(lines=at(secure, "HWINV:SECUREBOOT=0", "HWINV:SECUREBOOT=1")), "repeats"),
                ("malformed", dict(lines=at(secure, "HWINV:SECUREBOOT=OFF")), "malformed HWINV:SECUREBOOT"),
                ("lower case", dict(lines=at(secure, "HWINV:SECUREBOOT=absent")), "malformed HWINV:SECUREBOOT"),
                ("vendor differs", dict(lines=at(cpuid, "HWINV:CPUID:VENDOR=AuthenticAMD:HYPERVISOR=0")),
                 "CPUID vendor"),
                ("cpuid repeated", dict(lines=at(cpuid, lines[cpuid], lines[cpuid])), "repeats"),
                ("cpuid malformed", dict(lines=at(cpuid, "HWINV:CPUID:VENDOR=Intel:HYPERVISOR=0")),
                 "malformed HWINV:CPUID")):
            with self.subTest(case=label):
                self.rejects(self.signed(**changes), reason)
        # ABSENT maps to an unknown inventory state, never to disabled; the record keeps the operator observation.
        unknown = physical_inventory(secure_boot="unknown")
        self.assertTrue(hw_evidence.validate_record(self.signed(inventory=unknown), self.root)["passed"])
        self.assertIn("HWINV:SECUREBOOT=ABSENT", self.fixture.serial_lines(inventory=unknown))
        enabled = physical_inventory(secure_boot="enabled")
        self.assertTrue(hw_evidence.validate_record(self.signed(inventory=enabled, secure_boot="enabled"),
                                                    self.root)["passed"])
        spaced = physical_inventory(cpuid=dict(vendor="  Shanghai  ", hypervisor_bit=False))
        self.assertTrue(hw_evidence.validate_record(self.signed(inventory=spaced), self.root)["passed"])

    def test_malformed_field_types_are_rejected_without_aborting_the_search(self):
        self.use("H05-S5")
        malformed = []
        for label, changes in (
                ("record secure_boot", dict(secure_boot=["disabled"])),
                ("inventory secure_boot", dict(inventory=physical_inventory(secure_boot=["disabled"]))),
                ("inventory acpi dmar", dict(inventory=physical_inventory(acpi=dict(dmar=["present"], drhd_units=1)))),
                ("dma posture", dict(dma_posture={"unsupported": 1})),
                ("step result", dict(mutate=lambda r: r["steps"][0].update(result=["pass"])))):
            name = f"docs/verification/hw-inv-a-{label.replace(' ', '-').replace('_', '-')}/hw-evidence.json"
            with self.subTest(case=label):
                self.rejects(self.fixture.record(ident="HW-INV", key=self.key, name=name, **changes), "")
            malformed.append(name)
        # Every malformed candidate sorts before the valid HW-INV record, so the search must skip past them.
        self.assertTrue(all(name < self.prerequisites[0] for name in malformed))
        self.assertTrue(hw_evidence.validate_record(self.signed(), self.root)["passed"])

    def test_dmar_presence_is_not_containment(self):
        self.use("HW-INV")
        self.rejects(self.signed(dma_posture="VT-d contained"), "VT-d contained posture needs")

    def test_h02_usb_requires_dmar_and_xhci(self):
        self.use("H02-USB")
        self.assertTrue(hw_evidence.validate_record(self.signed(), self.root)["passed"])
        xhci = physical_inventory()["pci"][1]
        for inventory, reason in (
                (physical_inventory(acpi=dict(dmar="absent", drhd_units=0)), "DMAR table and at least one DRHD"),
                (physical_inventory(acpi=dict(dmar="present", drhd_units=0)), "DMAR table and at least one DRHD"),
                (physical_inventory(pci=[physical_inventory()["pci"][0]]), "no xhci PCI function"),
                (physical_inventory(pci=[dict(xhci, prog_if="20")]), "no xhci PCI function")):
            with self.subTest(inventory=inventory["acpi"], pci=len(inventory["pci"])):
                self.rejects(self.signed(inventory=inventory), reason)
        lines = [line for line in self.fixture.serial_lines() if line != "USB:BUS_MASTER:CONTAINED"]
        self.rejects(self.signed(lines=lines), "relabelled as passed: required marker 'USB:BUS_MASTER:CONTAINED'")
        self.rejects(self.signed(protocol=dict(id="H02-USB", version=1)), "version mismatch")
        contained = hw_evidence.validate_record(self.signed(dma_posture="VT-d contained"), self.root)
        self.assertEqual(contained["dma_posture"], "VT-d contained")

    def test_listed_device_requires_its_transport_marker(self):
        self.use("R03-INSTALL")
        lines = self.fixture.serial_lines()
        target = lines.index("INSTALL:TARGET:1")

        def with_target(*markers):
            return lines[:target] + list(markers) + lines[target + 1:]
        usb = with_target("INSTALL:TARGET:USB-MSC:0000:00:14.0")

        def fail_step(record):
            record["steps"][3]["result"] = "fail"
        self.rejects(self.signed(lines=usb), "device nvme listed without its transport marker")
        self.rejects(self.signed(lines=with_target("INSTALL:TARGET:NVME:0000:01:00.0")),
                     "device ahci listed without its transport marker")
        accepted = hw_evidence.validate_record(
            self.signed(lines=with_target("INSTALL:TARGET:NVME:0000:01:00.0"), devices=["nvme"]), self.root)
        self.assertEqual((accepted["devices"], accepted["passed"]), (["nvme"], True))
        self.assertEqual(hw_evidence.validate_record(self.signed(lines=usb, devices=[]), self.root)["devices"], [])
        self.rejects(self.signed(lines=usb, devices=["nvme"], passed=False, mutate=fail_step),
                     "device nvme listed without its transport marker")
        # A later target marker (for example during recovery) cannot rebind the verified USB install to NVMe.
        later = usb + ["INSTALL:TARGET:NVME:0000:01:00.0"]
        self.rejects(self.signed(lines=later, devices=["nvme"]), "INSTALL:TARGET")
        recovery = usb[:usb.index("RECOVERY:BOOT:PASS")] + ["INSTALL:TARGET:NVME:0000:01:00.0", "RECOVERY:BOOT:PASS"]
        self.rejects(self.signed(lines=recovery, devices=["nvme"]), "INSTALL:TARGET")
        self.rejects(self.signed(lines=recovery, devices=[]), "INSTALL:TARGET")


if __name__ == "__main__":
    unittest.main()

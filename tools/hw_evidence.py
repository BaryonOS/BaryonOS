"""Validate signed physical hardware evidence; this accepts no hardware by itself.

A record is ``docs/verification/<new-directory>/hw-evidence*.json`` with a
detached OpenSSH signature ``<record>.sig`` over its exact bytes. The record
binds a protocol id and version from ``docs/drivers/physical-protocols.json``,
the HW-FR target from ``docs/drivers/hardware-matrix.json``, the image, source,
inventory and serial-log hashes, the Secure Boot state, the operator and signer
and one confirmation per forbidden action. Signatures are checked only by the
standard ``ssh-keygen -Y verify`` tool; this module implements no cryptography.
The inventory must agree with the run's own serial log: its PCI functions with
the ``HWINV:PCI`` lines and count, its DMAR observation with the
``HWINV:ACPI:DMAR`` line and the kernel's ACPI discovery marker. A protocol may
be ``ready`` only while none of the owner decisions that block it is open, and a
record is accepted only when the accepted records its protocol requires exist
with the same target, ``source_sha256``, ``source_commit`` and inventory
SHA-256. Every failure rejects the record (fail closed).
"""
from __future__ import annotations

import argparse
import base64
import datetime
import fnmatch
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import struct
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]
PROTOCOLS = Path("docs/drivers/physical-protocols.json")
MATRIX = Path("docs/drivers/hardware-matrix.json")
SIGNERS = Path("docs/drivers/hw-allowed-signers")
KERNEL_MANIFEST = Path("kernel/Cargo.toml")
EVIDENCE = ("docs", "verification")
RECORD_GLOB = "hw-evidence*.json"
NAMESPACE = "baryonos-hw-evidence"
PHYSICAL_PROFILE = "HW-FR"
PROTOCOL_IDS = frozenset({"HW-INV", "H05-S5", "H05-S3", "H02-USB", "H03-AV", "H04-NET", "R03-INSTALL"})
OWNER_DECISIONS = frozenset({"OD-HW-TARGETS", "OD-HW-BOOT", "OD-HW-DISKS", "OD-HW-CAPTURE", "OD-HW-SIGNERS",
                             "OD-HW-FAMILIES", "OD-HW-PUBLISH", "OD-HW-INV-ROUTE", "OD-HW-POWER"})
# Minimum protocols each owner decision blocks (docs/drivers/HARDWARE-TARGETS.md). The matrix may list more,
# never fewer, so shrinking a decision's `blocks` cannot open a protocol.
REQUIRED_BLOCKS = {
    "OD-HW-TARGETS": PROTOCOL_IDS, "OD-HW-BOOT": PROTOCOL_IDS, "OD-HW-DISKS": PROTOCOL_IDS,
    "OD-HW-CAPTURE": PROTOCOL_IDS, "OD-HW-SIGNERS": PROTOCOL_IDS, "OD-HW-PUBLISH": PROTOCOL_IDS,
    "OD-HW-FAMILIES": frozenset({"H02-USB", "H03-AV", "H04-NET", "R03-INSTALL"}),
    "OD-HW-INV-ROUTE": frozenset({"HW-INV"}), "OD-HW-POWER": frozenset({"H05-S3"}),
}
# Accepted records a protocol requires with the same target, source_sha256, source_commit and inventory SHA-256
# (catalogue preconditions).
PREREQUISITES = {"HW-INV": (), "H05-S5": ("HW-INV",), "H05-S3": ("HW-INV", "H05-S5"), "H02-USB": ("HW-INV",),
                 "H03-AV": ("HW-INV",), "H04-NET": ("HW-INV",), "R03-INSTALL": ("HW-INV", "H02-USB", "H05-S5")}
# Boot and self-test markers that every profile emits; they never identify a device.
GENERIC_MARKERS = frozenset({"BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS",
                             "KS-03:PASS", "M1:PASS", "M2:PASS"})
BOOT_MARKER = "BOOT:EBS_OK"
HWINV_DMAR_FORMAT = "HWINV:ACPI:DMAR=<PRESENT|ABSENT>:DRHD=<n>"
HWINV_DMAR_MARKER = "HWINV:ACPI:DMAR=*"
HWINV_PCI_FORMATS = ("HWINV:PCI:<ssss>:<bb>:<dd>.<f>:<vendor>:<device>:<class>:<subclass>:<prog_if>:<revision>",
                     "HWINV:PCI:COUNT=<n>")
HWINV_PCI_COUNT_MARKER = "HWINV:PCI:COUNT=*"
PCI_FIELDS = ("address", "vendor", "device", "class", "subclass", "prog_if", "revision")
# Kernel ACPI discovery outcomes (kernel/src/m2/acpi.rs report(), crates/boot-protocol/src/acpi.rs) and the
# inventory DMAR observation each one allows: (acpi.dmar, minimum drhd_units, maximum drhd_units).
KERNEL_DMAR_STATES = {"ACPI:UNAVAILABLE:NO_DMAR": ("absent", 0, 0),
                      "ACPI:UNAVAILABLE:NO_DMAR_UNITS": ("present", 0, 0),
                      "ACPI:DISCOVERY:MCFG_DMAR": ("present", 1, None)}
CONTAINED_SUFFIX = ":BUS_MASTER:CONTAINED"
# PCI (class, subclass, prog_if) the bound inventory must list for a protocol device.
DEVICE_PCI_CLASS = {"xhci": ("0c", "03", "30")}
MAX_DRHD = 64
BASELINE_FORBIDDEN = ("no-firmware-flash", "no-firmware-settings-save", "no-bootorder-change",
                      "removable-media-only", "no-internal-disk-access", "no-secure-boot-key-change",
                      "no-personal-data")
RECORD_KEYS = ("schema", "kind", "protocol", "profile", "target", "devices", "image", "source_sha256",
               "source_commit", "inventory", "serial_log", "secure_boot", "operator", "signer", "date",
               "safety_confirmations", "steps", "dma_posture", "passed", "notes")
TOP_KEYS = {"schema", "namespace", "allowed_signers", "scope", "signing", "forbidden_action_catalogue",
            "baseline_forbidden_actions", "record_fields", "inventory_format", "hwinv_marker_format",
            "protocols"}
PROTOCOL_KEYS = {"id", "version", "criterion", "fr_rows", "title", "profile", "devices", "image", "status",
                 "blocking_reasons", "markers_implemented", "preconditions", "prerequisites", "steps",
                 "required_markers", "failure_markers", "device_markers", "forbidden_actions", "recovery_route",
                 "evidence_fields"}
# PCI vendors that only appear on emulated or paravirtual hardware.
HYPERVISOR_PCI_VENDORS = {"1af4": "virtio", "1b36": "QEMU", "1234": "QEMU/Bochs display",
                          "15ad": "VMware", "80ee": "VirtualBox"}
POSTURES = {"trusted kernel transport", "VT-d contained", "unsupported"}
KEY_TYPES = {"ssh-ed25519", "sk-ssh-ed25519@openssh.com", "ecdsa-sha2-nistp256", "ecdsa-sha2-nistp384",
             "ecdsa-sha2-nistp521", "sk-ecdsa-sha2-nistp256@openssh.com"}
HEX64 = re.compile(r"[0-9a-f]{64}")
HEX40 = re.compile(r"[0-9a-f]{40}")
HEX4 = re.compile(r"[0-9a-f]{4}")
HEX2 = re.compile(r"[0-9a-f]{2}")
PCI_ADDRESS = re.compile(r"[0-9a-f]{4}:[0-9a-f]{2}:[0-1][0-9a-f]\.[0-7]")
MARKER = re.compile(r"[A-Z0-9][A-Z0-9_:=.-]*\*?")
PRINCIPAL = re.compile(r"[A-Za-z0-9][A-Za-z0-9._+-]*(?:@[A-Za-z0-9][A-Za-z0-9.-]*)?")
SIGNER_LINE = re.compile(r"(?P<principals>\S+)\s+(?:(?P<options>\S+)\s+)?"
                         r"(?P<keytype>(?:ssh|ecdsa|sk)-\S+)\s+(?P<key>[A-Za-z0-9+/]+={0,2})(?:\s+\S.*)?")
OPTION = re.compile(r'(cert-authority|namespaces="[^"]*"|valid-(?:after|before)="?[0-9]{8,14}Z?"?)(?:,|$)')
HWINV_PCI = re.compile(r"HWINV:PCI:[0-9a-f]{4}:[0-9a-f]{2}:[0-9a-f]{2}\.[0-7]:([0-9a-f]{4}):", re.IGNORECASE)
HWINV_DMAR = re.compile(r"HWINV:ACPI:DMAR=(PRESENT|ABSENT):DRHD=(0|[1-9][0-9]?)")
# Lowercase hex exactly as in the inventory; any other line naming HWINV:PCI: is malformed.
HWINV_PCI_LINE = re.compile(r"HWINV:PCI:([0-9a-f]{4}:[0-9a-f]{2}:[0-1][0-9a-f]\.[0-7]):([0-9a-f]{4}):([0-9a-f]{4}):"
                            r"([0-9a-f]{2}):([0-9a-f]{2}):([0-9a-f]{2}):([0-9a-f]{2})")
HWINV_PCI_COUNT = re.compile(r"HWINV:PCI:COUNT=(0|[1-9][0-9]{0,3})")
KERNEL_DMAR = re.compile(r"ACPI:(?:UNAVAILABLE:NO_DMAR(?:_UNITS)?|DISCOVERY:MCFG_DMAR)")
HWINV_CPUID = re.compile(r"HWINV:CPUID:VENDOR=([ -~]{12}):HYPERVISOR=([01])")
HWINV_SECUREBOOT = re.compile(r"HWINV:SECUREBOOT=(0|1|ABSENT)")
# Inventory secure_boot value each HWINV:SECUREBOOT observation requires; an absent variable is unknown, not disabled.
SECUREBOOT_STATES = {"1": "enabled", "0": "disabled", "ABSENT": "unknown"}
# The START marker reports the playback duration that sets the completion deadline (H03-AV step S7).
HDA_START = re.compile(r"HDA:PLAYBACK:START:MS=([1-9][0-9]{0,5})")
INSTALL_TARGET = "INSTALL:TARGET:"
# Point sizes of uncompressed SEC1 points (RFC 5656 section 3.1).
EC_POINT_SIZES = {"nistp256": 65, "nistp384": 97, "nistp521": 133}
MAX_JSON, MAX_LOG, MAX_SIG, MAX_SIGNERS = 1 << 20, 4 << 20, 64 << 10, 64 << 10


class EvidenceError(ValueError):
    """A physical evidence input is rejected; the message states why."""


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise EvidenceError(message)


def _member(value: object, allowed) -> bool:
    """Set membership for untrusted JSON values; a list or object never raises TypeError."""
    return isinstance(value, str) and value in allowed


def repo_file(root: Path, name: str) -> Path:
    """Resolve a canonical repository-relative POSIX path to an existing file inside root."""
    _require(isinstance(name, str) and name != "", "reference must be a non-empty path")
    relative = PurePosixPath(name)
    _require(not relative.is_absolute() and relative.as_posix() == name and ".." not in relative.parts
             and "\\" not in name and ":" not in name, f"noncanonical repository path {name!r}")
    try:
        path = (root / name).resolve(strict=True)
    except OSError as error:
        raise EvidenceError(f"missing file {name}") from error
    _require(path.is_relative_to(root) and path.is_file(), f"{name} is outside the repository or not a file")
    # A symlink (or a path through a symlinked directory) would let a location rule judge a different file
    # than the one read; only the named file itself is accepted.
    _require(path.relative_to(root).as_posix() == name, f"{name} is a symlink or aliased path (fail closed)")
    return path


def _read(path: Path, limit: int) -> bytes:
    with path.open("rb") as stream:
        data = stream.read(limit + 1)
    _require(len(data) <= limit, f"{path.name} exceeds {limit} bytes")
    return data


def _json(path: Path) -> object:
    try:
        return json.loads(_read(path, MAX_JSON))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise EvidenceError(f"{path.name} is not valid JSON: {error}") from error


def _text_list(value: object, label: str, *, empty: bool = False) -> list[str]:
    _require(isinstance(value, list) and (empty or value)
             and all(isinstance(item, str) and item.strip() for item in value), f"{label} must be a list of text")
    return value


def _matrix(root: Path) -> dict:
    matrix = _json(repo_file(root, MATRIX.as_posix()))
    _require(isinstance(matrix, dict) and isinstance(matrix.get("profiles"), list), "invalid hardware matrix")
    return matrix


def matrix_profile(root: Path, matrix: dict | None = None) -> tuple[str | None, set[str]]:
    """Return the HW-FR target (None when unselected) and its device ids."""
    matrix = _matrix(root) if matrix is None else matrix
    found = [p for p in matrix["profiles"] if isinstance(p, dict) and p.get("id") == PHYSICAL_PROFILE]
    _require(len(found) == 1 and found[0].get("kind") == "hardware", "hardware matrix lacks one HW-FR profile")
    target = found[0].get("target")
    _require(target is None or (isinstance(target, str) and target.strip()), "invalid HW-FR target")
    devices = found[0].get("devices")
    _require(isinstance(devices, list), "invalid HW-FR devices")
    return target, {d.get("id") for d in devices if isinstance(d, dict)}


def decision_problem(decision: object) -> str | None:
    """Why one hardware-matrix owner_decisions entry is malformed, or None (shared with tools/hw_matrix.py)."""
    if (not isinstance(decision, dict) or not {"id", "decision", "status", "blocks"} <= set(decision)
            or not set(decision) <= {"id", "decision", "status", "blocks", "reference"}):
        return "invalid owner decision"
    ident, blocks = decision["id"], decision["blocks"]
    if not _member(ident, OWNER_DECISIONS):
        return "unknown or duplicate owner decision"
    if not isinstance(decision["decision"], str) or not decision["decision"].strip():
        return "owner decision lacks a question"
    if (not isinstance(blocks, list) or not blocks or not all(_member(block, PROTOCOL_IDS) for block in blocks)
            or len(set(blocks)) != len(blocks)):
        return "owner decision must block known physical protocols"
    if not set(blocks) >= REQUIRED_BLOCKS[ident]:
        return (f"{ident} blocks fewer protocols than the documented minimum "
                f"({', '.join(sorted(REQUIRED_BLOCKS[ident]))})")
    reference = decision.get("reference")
    if decision["status"] != "open" and (decision["status"] != "decided" or not isinstance(reference, str)
                                         or not reference.strip()):
        return "a decided owner decision needs a reference"
    return None


def open_blockers(matrix: dict) -> dict[str, list[str]]:
    """Map every protocol to the owner decisions that still block it (fail closed).

    A decision is closed only when exactly one entry carries its id, the entry is valid under
    decision_problem() (the hardware-matrix rules, including the REQUIRED_BLOCKS minimum) and its status is
    ``decided`` with a non-empty ``reference``. An open decision blocks its listed protocols plus
    REQUIRED_BLOCKS. Missing, duplicate or malformed entries stay open; an unknown entry blocks every protocol;
    a matrix without ``owner_decisions`` (schema 1) blocks every protocol.
    """
    blockers: dict[str, set[str]] = {ident: set() for ident in PROTOCOL_IDS}
    decisions = matrix.get("owner_decisions")
    if not isinstance(decisions, list):
        for found in blockers.values():
            found.add("owner decisions not recorded")
        return {ident: sorted(found) for ident, found in blockers.items()}
    entries: dict[str, list[dict]] = {}
    for entry in decisions:
        ident = entry.get("id") if isinstance(entry, dict) else None
        if isinstance(ident, str) and ident in OWNER_DECISIONS:
            entries.setdefault(ident, []).append(entry)
            continue
        for found in blockers.values():
            found.add(ident if isinstance(ident, str) else "malformed owner decision")
    for ident in OWNER_DECISIONS:
        found = entries.get(ident, [])
        if len(found) == 1 and found[0].get("status") == "decided" and decision_problem(found[0]) is None:
            continue
        listed = {block for entry in found if isinstance(entry.get("blocks"), list)
                  for block in entry["blocks"] if _member(block, PROTOCOL_IDS)}
        for protocol in listed | REQUIRED_BLOCKS[ident]:
            blockers[protocol].add(ident)
    return {ident: sorted(found) for ident, found in blockers.items()}


def _kernel_features(root: Path) -> set[str]:
    try:
        manifest = tomllib.loads(_read(repo_file(root, KERNEL_MANIFEST.as_posix()), MAX_JSON).decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        raise EvidenceError(f"invalid {KERNEL_MANIFEST}: {error}") from error
    features = manifest.get("features")
    _require(isinstance(features, dict), f"{KERNEL_MANIFEST} has no feature table")
    return set(features)


def load_protocols(root: Path = ROOT) -> dict[str, dict]:
    """Validate the protocol catalogue against the matrix, kernel features and this validator."""
    root = root.resolve(strict=True)
    doc = _json(repo_file(root, PROTOCOLS.as_posix()))
    _require(isinstance(doc, dict) and set(doc) == TOP_KEYS, "protocol catalogue has unknown or missing keys")
    _require(type(doc["schema"]) is int and doc["schema"] == 1, "unknown protocol catalogue schema")
    _require(doc["namespace"] == NAMESPACE and doc["allowed_signers"] == SIGNERS.as_posix(),
             "protocol catalogue namespace or signer file differs from the validator")
    _require(doc["record_fields"] == list(RECORD_KEYS), "record_fields differ from the validator's record schema")
    catalogue = doc["forbidden_action_catalogue"]
    _require(isinstance(catalogue, dict) and all(isinstance(v, str) and v.strip() for v in catalogue.values()),
             "invalid forbidden-action catalogue")
    _require(doc["baseline_forbidden_actions"] == list(BASELINE_FORBIDDEN)
             and set(BASELINE_FORBIDDEN) <= set(catalogue), "baseline forbidden actions differ from the validator")
    formats = doc["hwinv_marker_format"]
    _require(isinstance(formats, list) and all(isinstance(f, str) for f in formats)
             and {HWINV_DMAR_FORMAT, *HWINV_PCI_FORMATS} <= set(formats),
             f"hwinv_marker_format must list {HWINV_DMAR_FORMAT} and {' and '.join(HWINV_PCI_FORMATS)}")
    matrix = _matrix(root)
    _, devices = matrix_profile(root, matrix)
    blockers = open_blockers(matrix)
    features = _kernel_features(root)
    protocols: dict[str, dict] = {}
    _require(isinstance(doc["protocols"], list), "protocols must be a list")
    for protocol in doc["protocols"]:
        _require(isinstance(protocol, dict) and set(protocol) == PROTOCOL_KEYS, "protocol has unknown or missing keys")
        ident = protocol["id"]
        _require(ident in PROTOCOL_IDS and ident not in protocols, f"unknown or duplicate protocol {ident!r}")
        _require(type(protocol["version"]) is int and protocol["version"] >= 1, f"{ident}: invalid version")
        _require(isinstance(protocol["criterion"], str) and re.fullmatch(r"[A-Z][0-9]{2}", protocol["criterion"]),
                 f"{ident}: invalid criterion")
        _require(all(re.fullmatch(r"FR-[A-Z][0-9]{2}", row) for row in _text_list(protocol["fr_rows"], "fr_rows")),
                 f"{ident}: invalid FULL-TEST-RELEASE row")
        _require(isinstance(protocol["title"], str) and protocol["title"].strip(), f"{ident}: missing title")
        _require(protocol["profile"] == PHYSICAL_PROFILE, f"{ident}: physical protocols apply only to HW-FR")
        scope = _text_list(protocol["devices"], f"{ident} devices", empty=True)
        _require(len(set(scope)) == len(scope) and set(scope) <= devices, f"{ident}: devices absent from HW-FR")
        image = protocol["image"]
        _require(isinstance(image, dict) and set(image) == {"kernel_feature", "kernel_feature_present",
                                                            "physical_image"}
                 and isinstance(image["kernel_feature"], str) and type(image["kernel_feature_present"]) is bool
                 and image["physical_image"] in {"missing", "available"}, f"{ident}: invalid image description")
        _require(image["kernel_feature_present"] == (image["kernel_feature"] in features),
                 f"{ident}: kernel feature {image['kernel_feature']!r} presence disagrees with {KERNEL_MANIFEST}")
        _require(type(protocol["markers_implemented"]) is bool, f"{ident}: markers_implemented must be boolean")
        if protocol["status"] == "ready":
            _require(protocol["blocking_reasons"] == [] and protocol["markers_implemented"]
                     and image["kernel_feature_present"] and image["physical_image"] == "available",
                     f"{ident}: ready requires implemented markers, a present kernel feature and a physical image")
            _require(not blockers[ident],
                     f"{ident}: ready while owner decision(s) {', '.join(blockers[ident])} remain open")
        else:
            _require(protocol["status"] == "blocked", f"{ident}: status must be ready or blocked")
            _text_list(protocol["blocking_reasons"], f"{ident} blocking_reasons")
        _text_list(protocol["preconditions"], f"{ident} preconditions")
        _require(protocol["prerequisites"] == list(PREREQUISITES[ident]),
                 f"{ident}: prerequisites differ from the validator")
        _text_list(protocol["recovery_route"], f"{ident} recovery_route")
        steps = protocol["steps"]
        _require(isinstance(steps, list) and steps, f"{ident}: steps missing")
        for step in steps:
            _require(isinstance(step, dict) and set(step) == {"id", "actor", "action", "expect"}
                     and isinstance(step["id"], str) and re.fullmatch(r"S[0-9]{1,2}", step["id"])
                     and step["actor"] in {"operator", "serial", "host"}
                     and all(isinstance(step[k], str) and step[k].strip() for k in ("action", "expect")),
                     f"{ident}: invalid step")
        _require(len({s["id"] for s in steps}) == len(steps), f"{ident}: duplicate step id")
        for key in ("required_markers", "failure_markers"):
            _require(all(MARKER.fullmatch(m) for m in _text_list(protocol[key], f"{ident} {key}")),
                     f"{ident}: invalid marker in {key}")
        device_markers, claimed = protocol["device_markers"], []
        _require(isinstance(device_markers, dict) and set(device_markers) == set(scope),
                 f"{ident}: device_markers must name exactly the protocol devices")
        for device, listed in device_markers.items():
            for marker in _text_list(listed, f"{ident} device_markers {device}"):
                _require(MARKER.fullmatch(marker) is not None, f"{ident}: invalid device marker {marker!r}")
                _require(not any(_matches(marker, generic) for generic in GENERIC_MARKERS),
                         f"{ident}: generic marker {marker!r} cannot identify device {device}")
                # A line one device's pattern matches must never also satisfy another device's pattern.
                for owner, other in claimed:
                    _require(marker != other and (owner == device or not _overlap(marker, other)),
                             f"{ident}: device marker {marker!r} ({device}) overlaps {other!r} ({owner}); "
                             "markers are shared between devices")
                claimed.append((device, marker))
        forbidden = _text_list(protocol["forbidden_actions"], f"{ident} forbidden_actions")
        _require(len(set(forbidden)) == len(forbidden) and set(forbidden) <= set(catalogue)
                 and set(BASELINE_FORBIDDEN) <= set(forbidden), f"{ident}: forbidden actions omit the baseline")
        evidence_fields = protocol["evidence_fields"]
        _require(isinstance(evidence_fields, list) and sorted(evidence_fields) == sorted(RECORD_KEYS)
                 and len(evidence_fields) == len(RECORD_KEYS), f"{ident}: evidence_fields differ from the record schema")
        protocols[ident] = protocol
    _require(set(protocols) == PROTOCOL_IDS, "required physical protocol missing")
    _require(HWINV_DMAR_MARKER in protocols["HW-INV"]["required_markers"],
             f"HW-INV must require the {HWINV_DMAR_MARKER} marker")
    _require(HWINV_PCI_COUNT_MARKER in protocols["HW-INV"]["required_markers"],
             f"HW-INV must require the {HWINV_PCI_COUNT_MARKER} marker")
    return protocols


def _signer_options(text: str, number: int) -> dict[str, str]:
    _require(not text.endswith(","), f"allowed signers line {number}: malformed options")
    found: dict[str, str] = {}
    position = 0
    while position < len(text):
        match = OPTION.match(text, position)
        _require(match is not None, f"allowed signers line {number}: unsupported option")
        name = match.group(1).split("=", 1)[0]
        _require(name not in found, f"allowed signers line {number}: duplicate option {name}")
        found[name] = match.group(1)
        position = match.end()
    return found


def _ssh_strings(blob: bytes) -> list[bytes]:
    """Split an SSH wire-format blob into its uint32-length strings (format parsing only)."""
    fields, offset = [], 0
    while offset < len(blob):
        _require(offset + 4 <= len(blob), "truncated length prefix")
        (length,) = struct.unpack(">I", blob[offset:offset + 4])
        fields.append(blob[offset + 4:offset + 4 + length])
        _require(len(fields[-1]) == length, "truncated string")
        offset += 4 + length
    return fields


def _public_key_ok(keytype: str, blob: bytes) -> bool:
    """True when the blob has exactly the public-key layout of its type (RFC 8709, RFC 5656, PROTOCOL.u2f)."""
    try:
        fields = _ssh_strings(blob)
    except EvidenceError:
        return False
    if not fields or fields[0] != keytype.encode():
        return False
    if keytype == "ssh-ed25519":
        return len(fields) == 2 and len(fields[1]) == 32
    if keytype == "sk-ssh-ed25519@openssh.com":
        return len(fields) == 3 and len(fields[1]) == 32 and len(fields[2]) > 0
    curve = keytype.removeprefix("sk-").removeprefix("ecdsa-sha2-").removesuffix("@openssh.com")
    size = EC_POINT_SIZES.get(curve)
    point_ok = (size is not None and len(fields) >= 3 and fields[1] == curve.encode() and len(fields[2]) == size
                and fields[2][:1] == b"\x04")
    if keytype.startswith("ecdsa-sha2-"):
        return point_ok and len(fields) == 3
    if keytype == "sk-ecdsa-sha2-nistp256@openssh.com":
        return point_ok and len(fields) == 4 and len(fields[3]) > 0
    return False


def load_signers(root: Path = ROOT) -> set[str]:
    """Return the exact principals of the allowed-signers file; malformed entries reject the whole file."""
    path = repo_file(root.resolve(strict=True), SIGNERS.as_posix())
    try:
        text = _read(path, MAX_SIGNERS).decode("utf-8")
    except UnicodeDecodeError as error:
        raise EvidenceError("allowed signers file is not UTF-8") from error
    principals: set[str] = set()
    for number, raw in enumerate(text.splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        match = SIGNER_LINE.fullmatch(line)
        _require(match is not None, f"allowed signers line {number}: unsupported format")
        names = match["principals"].split(",")
        _require(all(PRINCIPAL.fullmatch(name) for name in names),
                 f"allowed signers line {number}: principals must be exact identities, not patterns")
        options = _signer_options(match["options"] or "", number)
        _require("cert-authority" not in options, f"allowed signers line {number}: cert-authority is not accepted")
        _require(options.get("namespaces") == f'namespaces="{NAMESPACE}"',
                 f'allowed signers line {number}: entry must carry exactly namespaces="{NAMESPACE}"')
        _require(match["keytype"] in KEY_TYPES, f"allowed signers line {number}: key type not accepted")
        try:
            blob = base64.b64decode(match["key"], validate=True)
        except ValueError as error:
            raise EvidenceError(f"allowed signers line {number}: invalid key encoding") from error
        _require(_public_key_ok(match["keytype"], blob),
                 f"allowed signers line {number}: key blob does not match its key type")
        principals.update(names)
    return principals


def ssh_keygen() -> str | None:
    return shutil.which("ssh-keygen")


def _sshsig_namespace(data: bytes) -> str:
    """Read the namespace field of an armored SSHSIG blob (format parsing only)."""
    try:
        lines = data.decode("ascii").strip().splitlines()
        _require(len(lines) >= 3 and lines[0].strip() == "-----BEGIN SSH SIGNATURE-----"
                 and lines[-1].strip() == "-----END SSH SIGNATURE-----", "signature is not an armored SSHSIG")
        blob = base64.b64decode("".join(line.strip() for line in lines[1:-1]), validate=True)
        _require(blob[:6] == b"SSHSIG" and len(blob) >= 10, "signature is not an SSHSIG blob")
        offset, fields = 10, []
        for _ in range(2):
            (length,) = struct.unpack(">I", blob[offset:offset + 4])
            fields.append(blob[offset + 4:offset + 4 + length])
            _require(len(fields[-1]) == length, "truncated SSHSIG blob")
            offset += 4 + length
        return fields[1].decode("utf-8")
    except EvidenceError:
        raise
    except (UnicodeDecodeError, ValueError, struct.error) as error:
        raise EvidenceError(f"malformed signature: {error}") from error


def verify_signature(root: Path, record: Path, data: bytes, signer: str) -> None:
    signature_name = record.relative_to(root).as_posix() + ".sig"
    _require((root / signature_name).is_file(), f"unsigned: detached signature {signature_name} is missing")
    signature = repo_file(root, signature_name)
    namespace = _sshsig_namespace(_read(signature, MAX_SIG))
    _require(namespace == NAMESPACE, f"wrong signature namespace {namespace!r}; expected {NAMESPACE!r}")
    tool = ssh_keygen()
    _require(tool is not None, "ssh-keygen is unavailable, so the signature cannot be verified (fail closed)")
    try:
        result = subprocess.run([tool, "-Y", "verify", "-f", str(repo_file(root, SIGNERS.as_posix())),
                                 "-I", signer, "-n", NAMESPACE, "-s", str(signature)],
                                input=data, capture_output=True, timeout=30, check=False,
                                env={"PATH": os.environ.get("PATH", ""), "LC_ALL": "C"})
    except (OSError, subprocess.TimeoutExpired) as error:
        raise EvidenceError(f"ssh-keygen verification did not complete: {error}") from error
    detail = (result.stderr or result.stdout).decode("utf-8", "replace").strip().splitlines()
    _require(result.returncode == 0, f"signature verification failed for signer {signer!r}"
             + (f": {detail[-1]}" if detail else ""))


def _sha256_ref(root: Path, record_dir: Path, value: object, label: str) -> bytes:
    _require(isinstance(value, dict) and set(value) == {"path", "sha256"}, f"{label} must be {{path, sha256}}")
    _require(isinstance(value["sha256"], str) and HEX64.fullmatch(value["sha256"]), f"{label} sha256 is not hex")
    path = repo_file(root, value["path"])
    _require(path.parent == record_dir, f"{label} must be in the record directory")
    data = _read(path, MAX_LOG)
    _require(hashlib.sha256(data).hexdigest() == value["sha256"], f"{label} SHA-256 mismatch")
    return data


def _inventory(data: bytes) -> dict:
    try:
        inventory = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise EvidenceError(f"inventory is not valid JSON: {error}") from error
    _require(isinstance(inventory, dict)
             and set(inventory) == {"schema", "kind", "cpuid", "secure_boot", "acpi", "pci"}
             and type(inventory["schema"]) is int and inventory["schema"] == 2
             and inventory["kind"] == "baryonos-hw-inventory",
             "inventory has an unknown format")
    cpuid = inventory["cpuid"]
    _require(isinstance(cpuid, dict) and set(cpuid) == {"vendor", "hypervisor_bit"}
             and isinstance(cpuid["vendor"], str) and type(cpuid["hypervisor_bit"]) is bool,
             "inventory lacks the CPUID vendor and hypervisor bit")
    _require(not cpuid["hypervisor_bit"], "virtual capture: inventory records the CPUID hypervisor bit")
    _require(_member(inventory["secure_boot"], {"enabled", "disabled", "unknown"}),
             "inventory Secure Boot state invalid")
    acpi = inventory["acpi"]
    _require(isinstance(acpi, dict) and set(acpi) == {"dmar", "drhd_units"}
             and _member(acpi["dmar"], {"present", "absent"})
             and type(acpi["drhd_units"]) is int and 0 <= acpi["drhd_units"] <= MAX_DRHD
             and (acpi["dmar"] == "present" or acpi["drhd_units"] == 0),
             f"inventory acpi must be {{dmar: present|absent, drhd_units: 0..{MAX_DRHD}}}; absent needs 0 units")
    pci = inventory["pci"]
    _require(isinstance(pci, list) and 0 < len(pci) <= 1024, "inventory lists no PCI functions")
    for function in pci:
        _require(isinstance(function, dict) and set(function) == {"address", "vendor", "device", "class",
                                                                  "subclass", "prog_if", "revision"}
                 and isinstance(function["address"], str) and PCI_ADDRESS.fullmatch(function["address"])
                 and all(isinstance(function[k], str) and HEX4.fullmatch(function[k]) for k in ("vendor", "device"))
                 and all(isinstance(function[k], str) and HEX2.fullmatch(function[k])
                         for k in ("class", "subclass", "prog_if", "revision")), "invalid inventory PCI function")
        vendor = function["vendor"]
        _require(vendor not in HYPERVISOR_PCI_VENDORS,
                 f"virtual capture: PCI vendor {vendor} ({HYPERVISOR_PCI_VENDORS.get(vendor)}) is not physical")
    _require(len({function["address"] for function in pci}) == len(pci), "inventory lists a PCI address twice")
    return inventory


def _matches(marker: str, line: str) -> bool:
    return line.startswith(marker[:-1]) if marker.endswith("*") else line == marker


def _overlap(first: str, second: str) -> bool:
    """True when some serial line can match both marker patterns (a trailing '*' is the only wildcard)."""
    one, two = first.removesuffix("*"), second.removesuffix("*")
    if first.endswith("*") and second.endswith("*"):
        return one.startswith(two) or two.startswith(one)
    if first.endswith("*"):
        return two.startswith(one)
    if second.endswith("*"):
        return one.startswith(two)
    return one == two


def pci_log_problem(lines: list[str], pci: list[dict], *, required: bool) -> str | None:
    """Return why the serial log's HWINV:PCI lines differ from inventory.pci, or None when they agree.

    When the log names HWINV:PCI anywhere, or the protocol requires HWINV:PCI:COUNT (HW-INV), the log must list
    every inventory PCI function exactly once as an HWINV:PCI line and nothing else, and carry exactly one
    HWINV:PCI:COUNT equal to their number.
    """
    listed, counts = [], []
    for line in lines:
        if "HWINV:PCI:" not in line.upper():
            continue
        if (count := HWINV_PCI_COUNT.fullmatch(line)) is not None:
            counts.append(int(count[1]))
        elif (function := HWINV_PCI_LINE.fullmatch(line)) is not None:
            listed.append(function.groups())
        else:
            return f"malformed HWINV:PCI line {line!r}"
    if not (required or listed or counts):
        return None
    if len(counts) != 1:
        return f"serial log carries {len(counts)} HWINV:PCI:COUNT lines; exactly one is required"
    if len(set(listed)) != len(listed):
        return "serial log lists a PCI function twice"
    inventory = {tuple(function[key] for key in PCI_FIELDS) for function in pci}
    missing, extra = sorted(inventory - set(listed)), sorted(set(listed) - inventory)
    if missing or extra:
        return ("HWINV:PCI lines differ from inventory.pci (absent from the log: "
                f"{', '.join(f[0] for f in missing) or 'none'}; absent from the inventory: "
                f"{', '.join(f[0] for f in extra) or 'none'})")
    if counts[0] != len(listed):
        return f"HWINV:PCI:COUNT={counts[0]} differs from the {len(listed)} listed PCI functions"
    return None


def _ordered_match(marker: str, line: str) -> bool:
    # The loader writes BOOT:EBS_OK without a preceding newline, so firmware bytes can precede it on its line; the
    # marker must still end the line. The restart count below counts every occurrence the same way or more.
    return line.endswith(BOOT_MARKER) if marker == BOOT_MARKER else _matches(marker, line)


def outcome(protocol: dict, lines: list[str], steps: list[dict]) -> str | None:
    """Return why a run failed, or None when every step and marker passed."""
    failed = [step["id"] for step in steps if step["result"] != "pass"]
    if failed:
        return f"step(s) {', '.join(failed)} did not pass"
    hit = next((line for line in lines for marker in protocol["failure_markers"] if _matches(marker, line)), None)
    if hit is not None:
        return f"failure marker {hit!r} in serial log"
    position = 0
    for marker in protocol["required_markers"]:
        while position < len(lines) and not _ordered_match(marker, lines[position]):
            position += 1
        if position == len(lines):
            return f"required marker {marker!r} missing or out of order"
        position += 1
    # Every occurrence counts, also after firmware output on the same line: the loader writes the marker without
    # a preceding newline, so a restart must not hide behind console bytes.
    boots = sum(line.count(BOOT_MARKER) for line in lines)
    expected = protocol["required_markers"].count(BOOT_MARKER)
    if boots != expected:
        return f"{BOOT_MARKER} appears {boots} time(s); protocol expects exactly {expected} (unexpected restart)"
    return None


def is_record_path(name: str) -> bool:
    parts = PurePosixPath(name).parts
    return len(parts) >= 4 and parts[:2] == EVIDENCE and fnmatch.fnmatchcase(parts[-1], RECORD_GLOB)


class _PrerequisiteCycle(EvidenceError):
    """The prerequisite table loops; never skipped as an ordinary rejected candidate."""


def _record_protocol(path: Path) -> object:
    """Read only a candidate record's protocol id, to pre-filter candidates before full validation."""
    try:
        record = json.loads(_read(path, MAX_JSON))
    except (EvidenceError, OSError, UnicodeDecodeError, json.JSONDecodeError):
        return None
    reference = record.get("protocol") if isinstance(record, dict) else None
    return reference.get("id") if isinstance(reference, dict) else None


def _require_prerequisites(root: Path, name: str, summary: dict, active: frozenset[str]) -> None:
    """Require an accepted, passing prerequisite record with the same target, source hash, commit and inventory.

    Every candidate is fully validated (signature, protocol status, owner decisions and its own prerequisites);
    a candidate that fails validation never counts.
    """
    chain = active | {summary["protocol"]}
    candidates: list[str] | None = None
    for required in PREREQUISITES[summary["protocol"]]:
        if required in chain:
            raise _PrerequisiteCycle(f"prerequisite cycle: {' -> '.join(sorted(chain))} -> {required}")
        if candidates is None:
            candidates = sorted(p.relative_to(root).as_posix() for p in root.joinpath(*EVIDENCE).rglob(RECORD_GLOB)
                                if p.is_file())
        found = False
        for candidate in candidates:
            if candidate == name or _record_protocol(root / candidate) != required:
                continue
            try:
                other = _validate_record(candidate, root, chain)
            except _PrerequisiteCycle:
                raise
            except (EvidenceError, OSError):
                continue
            found = (other["protocol"] == required and other["passed"] is True
                     and all(other[key] == summary[key]
                             for key in ("target", "source_sha256", "source_commit", "inventory_sha256")))
            if found:
                break
        _require(found, f"precondition unmet: no accepted, passing {required} record with the same target, "
                        "source_sha256, source_commit and inventory SHA-256")


def validate_record(name: str | Path, root: Path = ROOT) -> dict:
    """Validate one record and return its binding summary; raise EvidenceError on any defect."""
    root = root.resolve(strict=True)
    if isinstance(name, Path):
        name = (name.relative_to(root) if name.is_absolute() else name).as_posix()
    return _validate_record(name, root, frozenset())


def _validate_record(name: str, root: Path, active: frozenset[str]) -> dict:
    _require(is_record_path(name), f"{name}: records must be docs/verification/<dir>/{RECORD_GLOB}")
    path = repo_file(root, name)
    _require(is_record_path(path.relative_to(root).as_posix()),
             f"{name}: records must be docs/verification/<dir>/{RECORD_GLOB}")
    data = _read(path, MAX_JSON)
    principals = load_signers(root)
    _require(bool(principals), f"no allowed signers: {SIGNERS} lists no key; the project owner must add a signing "
             "key (open owner decision OD-HW-SIGNERS), so every physical evidence record is rejected (fail closed)")
    try:
        record = json.loads(data)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise EvidenceError(f"record is not valid JSON: {error}") from error
    _require(isinstance(record, dict), "record must be a JSON object")
    signer = record.get("signer")
    _require(isinstance(signer, str) and PRINCIPAL.fullmatch(signer) is not None, "record lacks an exact signer")
    _require(signer in principals, f"unknown signer {signer!r}: not listed in {SIGNERS}")
    verify_signature(root, path, data, signer)

    _require(set(record) == set(RECORD_KEYS), "record has unknown or missing fields")
    _require(type(record["schema"]) is int and record["schema"] == 1 and record["kind"] == NAMESPACE,
             "unknown record schema or kind")
    protocols = load_protocols(root)
    reference = record["protocol"]
    _require(isinstance(reference, dict) and set(reference) == {"id", "version"}, "protocol must be {id, version}")
    protocol = protocols.get(reference["id"]) if isinstance(reference["id"], str) else None
    _require(protocol is not None, f"unknown protocol {reference['id']!r}")
    _require(type(reference["version"]) is int and reference["version"] == protocol["version"],
             f"protocol version mismatch: record {reference['version']!r}, catalogue {protocol['version']}")
    if protocol["status"] != "ready":
        raise EvidenceError(f"protocol {protocol['id']} v{protocol['version']} is blocked: "
                            + "; ".join(protocol["blocking_reasons"]))
    target, matrix_devices = matrix_profile(root)
    _require(record["profile"] == PHYSICAL_PROFILE, f"profile must be {PHYSICAL_PROFILE}")
    _require(target is not None, "HW-FR target not selected in the hardware matrix (owner decision OD-HW-TARGETS)")
    _require(record["target"] == target, "record target differs from the hardware-matrix HW-FR target")
    devices = record["devices"]
    _require(isinstance(devices, list) and all(isinstance(d, str) for d in devices)
             and len(set(devices)) == len(devices) and set(devices) <= set(protocol["devices"])
             and set(devices) <= matrix_devices, "record devices are outside the protocol or matrix")
    image = record["image"]
    _require(isinstance(image, dict) and set(image) == {"name", "sha256"} and isinstance(image["name"], str)
             and image["name"].strip() and isinstance(image["sha256"], str) and HEX64.fullmatch(image["sha256"]),
             "image must be {name, sha256} with a lowercase SHA-256")
    _require(isinstance(record["source_sha256"], str) and HEX64.fullmatch(record["source_sha256"]),
             "source_sha256 must be a lowercase SHA-256")
    _require(isinstance(record["source_commit"], str) and HEX40.fullmatch(record["source_commit"]),
             "source_commit must be a full lowercase commit id")
    inventory = _inventory(_sha256_ref(root, path.parent, record["inventory"], "inventory"))
    log = _sha256_ref(root, path.parent, record["serial_log"], "serial log").decode("utf-8", "replace")
    lines = [line.strip() for line in log.splitlines()]
    _require(_member(record["secure_boot"], {"enabled", "disabled"}), "secure_boot must be enabled or disabled")
    _require(inventory["secure_boot"] in {"unknown", record["secure_boot"]},
             "Secure Boot state differs between record and inventory")
    _require(isinstance(record["operator"], str) and 0 < len(record["operator"].strip()) <= 128
             and record["operator"].isprintable(), "operator identity missing")
    date = record["date"]
    _require(isinstance(date, str) and re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}", date), "date must be YYYY-MM-DD")
    try:
        datetime.date.fromisoformat(date)
    except ValueError as error:
        raise EvidenceError("date must be a valid YYYY-MM-DD date") from error
    confirmations = record["safety_confirmations"]
    _require(isinstance(confirmations, dict) and set(confirmations) == set(protocol["forbidden_actions"]),
             "safety confirmations must cover exactly the protocol's forbidden actions")
    unconfirmed = sorted(k for k, v in confirmations.items() if v is not True)
    _require(not unconfirmed, f"safety confirmation(s) not given: {', '.join(unconfirmed)}")
    steps = record["steps"]
    _require(isinstance(steps, list) and all(isinstance(s, dict) and set(s) == {"id", "result"}
                                             and _member(s["result"], {"pass", "fail", "not-run"}) for s in steps)
             and [s["id"] for s in steps] == [s["id"] for s in protocol["steps"]],
             "steps must list every protocol step in order with pass, fail or not-run")
    _require(_member(record["dma_posture"], POSTURES), "unknown DMA posture")
    contained = any(marker.endswith(CONTAINED_SUFFIX) for marker in protocol["required_markers"])
    _require(record["dma_posture"] != "VT-d contained" or contained,
             f"VT-d contained posture needs a protocol that requires a *{CONTAINED_SUFFIX} marker; an ACPI DMAR "
             "table in the inventory is not containment")
    _require(type(record["passed"]) is bool, "passed must be boolean")
    _require(isinstance(record["notes"], str) and len(record["notes"]) <= 4000, "notes must be text")
    for line in lines:
        _require(not (line.startswith("HWINV:CPUID:") and "HYPERVISOR=1" in line),
                 "virtual capture: serial log records the CPUID hypervisor bit")
        vendor = HWINV_PCI.match(line)
        _require(vendor is None or vendor.group(1).lower() not in HYPERVISOR_PCI_VENDORS,
                 f"virtual capture: serial log lists PCI vendor {vendor and vendor.group(1)}")
    cpuid_lines = [line for line in lines if line.startswith("HWINV:CPUID:")]
    _require(len(cpuid_lines) <= 1, "serial log repeats the HWINV:CPUID observation")
    for line in cpuid_lines:
        cpuid = HWINV_CPUID.fullmatch(line)
        _require(cpuid is not None, f"malformed HWINV:CPUID marker {line!r}")
        _require(cpuid[1] == inventory["cpuid"]["vendor"],
                 f"serial log CPUID vendor {cpuid[1]!r} contradicts the inventory")
    secure_lines = [line for line in lines if line.startswith("HWINV:SECUREBOOT")]
    _require(len(secure_lines) <= 1, "serial log repeats the HWINV:SECUREBOOT observation")
    for line in secure_lines:
        secure = HWINV_SECUREBOOT.fullmatch(line)
        _require(secure is not None, f"malformed HWINV:SECUREBOOT marker {line!r}")
        _require(inventory["secure_boot"] == SECUREBOOT_STATES[secure[1]],
                 f"serial log Secure Boot observation {secure[1]} contradicts the inventory "
                 f"(secure_boot={inventory['secure_boot']})")
    start_lines = [line for line in lines if line.startswith("HDA:PLAYBACK:START")]
    _require(len(start_lines) <= 1, "serial log repeats HDA:PLAYBACK:START")
    for line in start_lines:
        _require(HDA_START.fullmatch(line) is not None,
                 f"malformed HDA:PLAYBACK:START marker {line!r}; expected HDA:PLAYBACK:START:MS=<n>")
    # One install target per run: a later target line (for example during recovery) cannot rebind the claim.
    _require(sum(INSTALL_TARGET in line for line in lines) <= 1, "serial log repeats INSTALL:TARGET")
    dmar_lines = [line for line in lines if line.startswith("HWINV:ACPI:DMAR=")]
    _require(len(dmar_lines) <= 1, "serial log repeats the HWINV:ACPI:DMAR observation")
    for line in dmar_lines:
        dmar = HWINV_DMAR.fullmatch(line)
        _require(dmar is not None and int(dmar[2]) <= MAX_DRHD, f"malformed DMAR marker {line!r}")
        _require((dmar[1].lower(), int(dmar[2])) == (inventory["acpi"]["dmar"], inventory["acpi"]["drhd_units"]),
                 "serial log DMAR observation contradicts the inventory")
    acpi = inventory["acpi"]
    for line in lines:
        for found in KERNEL_DMAR.finditer(line):
            state, low, high = KERNEL_DMAR_STATES[found[0]]
            _require(acpi["dmar"] == state and low <= acpi["drhd_units"] <= (MAX_DRHD if high is None else high),
                     f"kernel ACPI marker {found[0]} in the serial log contradicts the inventory "
                     f"(acpi.dmar={acpi['dmar']}, drhd_units={acpi['drhd_units']})")
    pci_problem = pci_log_problem(lines, inventory["pci"],
                                  required=HWINV_PCI_COUNT_MARKER in protocol["required_markers"])
    _require(pci_problem is None, f"inventory not bound to the serial log: {pci_problem}")
    if contained:
        _require(inventory["acpi"]["dmar"] == "present" and inventory["acpi"]["drhd_units"] >= 1,
                 f"{protocol['id']} needs an inventory with an ACPI DMAR table and at least one DRHD unit")
    for device in protocol["devices"]:
        wanted = DEVICE_PCI_CLASS.get(device)
        if wanted is not None:
            _require(any((f["class"], f["subclass"], f["prog_if"]) == wanted for f in inventory["pci"]),
                     f"inventory lists no {device} PCI function (class {'/'.join(wanted)})")
    failure = outcome(protocol, lines, steps)
    _require(not (record["passed"] and failure), f"failed run relabelled as passed: {failure}")
    # Also for failed runs: a record never names a device whose own transport markers are absent.
    for device in devices:
        for marker in protocol["device_markers"][device]:
            _require(any(_matches(marker, line) for line in lines),
                     f"device {device} listed without its transport marker {marker}")
    summary = dict(path=name, protocol=protocol["id"], version=protocol["version"], profile=record["profile"],
                   target=record["target"], devices=list(devices), passed=record["passed"],
                   dma_posture=record["dma_posture"], signer=signer, source_sha256=record["source_sha256"],
                   source_commit=record["source_commit"], inventory_sha256=record["inventory"]["sha256"])
    _require_prerequisites(root, name, summary, active)
    return summary


def check(root: Path = ROOT) -> dict:
    root = root.resolve(strict=True)
    protocols = load_protocols(root)
    principals = load_signers(root)
    records = sorted(p.relative_to(root).as_posix() for p in root.joinpath(*EVIDENCE).rglob(RECORD_GLOB)
                     if p.is_file())
    accepted, errors = [], []
    for name in records:
        try:
            accepted.append(validate_record(name, root))
        except (EvidenceError, OSError) as error:
            errors.append(f"{name}: {error}")
    if errors:
        raise EvidenceError("rejected physical evidence: " + "; ".join(errors))
    return dict(
        protocols=len(protocols),
        ready=sorted(i for i, p in protocols.items() if p["status"] == "ready"),
        blocked=sorted(i for i, p in protocols.items() if p["status"] == "blocked"),
        signers=len(principals),
        signer_status=("configured" if principals else
                       "none: every physical evidence record is rejected until the project owner adds a key"),
        records=len(records), accepted=accepted,
        limit="Signature, binding and plausibility checks only; no physical acceptance without owner-signed runs.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--check", action="store_true", required=True)
    parser.parse_args()
    try:
        print(json.dumps(check(), indent=2))
    except (OSError, ValueError, TypeError) as error:
        parser.exit(1, f"hardware evidence: {error}\n")

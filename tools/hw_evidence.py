"""Validate signed physical hardware evidence; this accepts no hardware by itself.

A record is ``docs/verification/<new-directory>/hw-evidence*.json`` with a
detached OpenSSH signature ``<record>.sig`` over its exact bytes. The record
binds a protocol id and version from ``docs/drivers/physical-protocols.json``,
the HW-FR target from ``docs/drivers/hardware-matrix.json``, the image, source,
inventory and serial-log hashes, the Secure Boot state, the operator and signer
and one confirmation per forbidden action. Signatures are checked only by the
standard ``ssh-keygen -Y verify`` tool; this module implements no cryptography.
Every failure rejects the record (fail closed).
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
                 "blocking_reasons", "markers_implemented", "preconditions", "steps", "required_markers",
                 "failure_markers", "forbidden_actions", "recovery_route", "evidence_fields"}
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
MAX_JSON, MAX_LOG, MAX_SIG, MAX_SIGNERS = 1 << 20, 4 << 20, 64 << 10, 64 << 10


class EvidenceError(ValueError):
    """A physical evidence input is rejected; the message states why."""


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise EvidenceError(message)


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


def matrix_profile(root: Path) -> tuple[str | None, set[str]]:
    """Return the HW-FR target (None when unselected) and its device ids."""
    matrix = _json(repo_file(root, MATRIX.as_posix()))
    _require(isinstance(matrix, dict) and isinstance(matrix.get("profiles"), list), "invalid hardware matrix")
    found = [p for p in matrix["profiles"] if isinstance(p, dict) and p.get("id") == PHYSICAL_PROFILE]
    _require(len(found) == 1 and found[0].get("kind") == "hardware", "hardware matrix lacks one HW-FR profile")
    target = found[0].get("target")
    _require(target is None or (isinstance(target, str) and target.strip()), "invalid HW-FR target")
    devices = found[0].get("devices")
    _require(isinstance(devices, list), "invalid HW-FR devices")
    return target, {d.get("id") for d in devices if isinstance(d, dict)}


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
    _, devices = matrix_profile(root)
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
        else:
            _require(protocol["status"] == "blocked", f"{ident}: status must be ready or blocked")
            _text_list(protocol["blocking_reasons"], f"{ident} blocking_reasons")
        _text_list(protocol["preconditions"], f"{ident} preconditions")
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
        forbidden = _text_list(protocol["forbidden_actions"], f"{ident} forbidden_actions")
        _require(len(set(forbidden)) == len(forbidden) and set(forbidden) <= set(catalogue)
                 and set(BASELINE_FORBIDDEN) <= set(forbidden), f"{ident}: forbidden actions omit the baseline")
        evidence_fields = protocol["evidence_fields"]
        _require(isinstance(evidence_fields, list) and sorted(evidence_fields) == sorted(RECORD_KEYS)
                 and len(evidence_fields) == len(RECORD_KEYS), f"{ident}: evidence_fields differ from the record schema")
        protocols[ident] = protocol
    _require(set(protocols) == PROTOCOL_IDS, "required physical protocol missing")
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
        _require(len(blob) > 4 and blob[4:4 + struct.unpack(">I", blob[:4])[0]] == match["keytype"].encode(),
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
    _require(isinstance(inventory, dict) and set(inventory) == {"schema", "kind", "cpuid", "secure_boot", "pci"}
             and type(inventory["schema"]) is int and inventory["schema"] == 1
             and inventory["kind"] == "baryonos-hw-inventory",
             "inventory has an unknown format")
    cpuid = inventory["cpuid"]
    _require(isinstance(cpuid, dict) and set(cpuid) == {"vendor", "hypervisor_bit"}
             and isinstance(cpuid["vendor"], str) and type(cpuid["hypervisor_bit"]) is bool,
             "inventory lacks the CPUID vendor and hypervisor bit")
    _require(not cpuid["hypervisor_bit"], "virtual capture: inventory records the CPUID hypervisor bit")
    _require(inventory["secure_boot"] in {"enabled", "disabled", "unknown"}, "inventory Secure Boot state invalid")
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
    return inventory


def _matches(marker: str, line: str) -> bool:
    return line.startswith(marker[:-1]) if marker.endswith("*") else line == marker


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
        while position < len(lines) and not _matches(marker, lines[position]):
            position += 1
        if position == len(lines):
            return f"required marker {marker!r} missing or out of order"
        position += 1
    return None


def is_record_path(name: str) -> bool:
    parts = PurePosixPath(name).parts
    return len(parts) >= 4 and parts[:2] == EVIDENCE and fnmatch.fnmatchcase(parts[-1], RECORD_GLOB)


def validate_record(name: str | Path, root: Path = ROOT) -> dict:
    """Validate one record and return its binding summary; raise EvidenceError on any defect."""
    root = root.resolve(strict=True)
    if isinstance(name, Path):
        name = (name.relative_to(root) if name.is_absolute() else name).as_posix()
    _require(is_record_path(name), f"{name}: records must be docs/verification/<dir>/{RECORD_GLOB}")
    path = repo_file(root, name)
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
    _require(record["secure_boot"] in {"enabled", "disabled"}, "secure_boot must be enabled or disabled")
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
                                             and s["result"] in {"pass", "fail", "not-run"} for s in steps)
             and [s["id"] for s in steps] == [s["id"] for s in protocol["steps"]],
             "steps must list every protocol step in order with pass, fail or not-run")
    _require(record["dma_posture"] in POSTURES, "unknown DMA posture")
    _require(type(record["passed"]) is bool, "passed must be boolean")
    _require(isinstance(record["notes"], str) and len(record["notes"]) <= 4000, "notes must be text")
    for line in lines:
        _require(not (line.startswith("HWINV:CPUID:") and "HYPERVISOR=1" in line),
                 "virtual capture: serial log records the CPUID hypervisor bit")
        vendor = HWINV_PCI.match(line)
        _require(vendor is None or vendor.group(1).lower() not in HYPERVISOR_PCI_VENDORS,
                 f"virtual capture: serial log lists PCI vendor {vendor and vendor.group(1)}")
    failure = outcome(protocol, lines, steps)
    _require(not (record["passed"] and failure), f"failed run relabelled as passed: {failure}")
    return dict(path=name, protocol=protocol["id"], version=protocol["version"], profile=record["profile"],
                target=record["target"], devices=list(devices), passed=record["passed"],
                dma_posture=record["dma_posture"], signer=signer)


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

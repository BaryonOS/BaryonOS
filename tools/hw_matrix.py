"""Check hardware inventory references; this does not accept hardware or DMA isolation.

Schema 2 records an explicit evidence level per device row. QEMU and VMware
profiles can reach at most `native-verified`; `supported` exists only for the
physical profile and requires a passing record that tools/hw_evidence.py
validates (owner-signed, protocol/target/hash-bound, not a virtual capture).
Schema 1 matrices remain readable, but can no longer express `supported`.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path, PurePosixPath
import re

import hw_evidence

ROOT = Path(__file__).resolve().parents[1]
MATRIX = Path("docs/drivers/hardware-matrix.json")
PROFILES = {"VM-FR": "qemu", "VM-VMW": "vmware", "HW-FR": "hardware"}
FAMILIES = {"gop", "ps2", "xhci", "nvme", "ahci", "e1000e", "hda", "acpi", "rng", "vtd"}
POSTURES = {"trusted kernel transport", "VT-d contained", "unsupported"}
STATUSES = {"implemented", "pending", "unsupported", "native-verified", "supported"}
LEVELS = {"none", "host", "native-qemu", "vmware", "physical"}
KIND_LEVELS = {"qemu": {"none", "host", "native-qemu"}, "vmware": {"none", "host", "vmware"},
               "hardware": {"none", "host", "physical"}}
VIRTUAL_LEVEL = {"qemu": "native-qemu", "vmware": "vmware"}
ROW_KEYS = {"id", "driver_path", "dma_posture", "status", "evidence"}
OWNER_DECISIONS = {"OD-HW-TARGETS", "OD-HW-BOOT", "OD-HW-DISKS", "OD-HW-CAPTURE", "OD-HW-SIGNERS",
                   "OD-HW-FAMILIES", "OD-HW-PUBLISH", "OD-HW-INV-ROUTE", "OD-HW-POWER"}
XHCI_LIMIT = ("kernel/src/m2/xhci.rs accepts only QEMU's qemu-xhci (PCI 1b36:000d) at a fixed fixture MMIO "
              "address, so xHCI stays pending outside the QEMU profile")


def local_file(root: Path, name: str) -> Path:
    if not isinstance(name, str):
        raise ValueError("reference must be a path")
    relative = PurePosixPath(name)
    if relative.is_absolute() or relative.as_posix() != name or ".." in relative.parts or "\\" in name or ":" in name:
        raise ValueError("noncanonical repository path")
    path = (root / name).resolve(strict=True)
    if not path.is_relative_to(root) or not path.is_file():
        raise ValueError("reference outside repository or not a file")
    return path


def _verification(root: Path, path: Path) -> bool:
    return path.relative_to(root).parts[:2] == ("docs", "verification")


def proof(root: Path, paths: list[Path], profile: dict, device: dict, *, dma=False) -> bool:
    """Legacy structural record check, kept for VT-d posture on virtual profiles only."""
    for path in paths:
        if not _verification(root, path) or path.suffix != ".json":
            continue
        with path.open("rb") as stream:
            raw = stream.read(1024 * 1024 + 1)
        if len(raw) > 1024 * 1024:
            raise ValueError("oversize hardware evidence")
        record = json.loads(raw)
        if (isinstance(record, dict) and record.get("passed") is True
                and record.get("level") == "native"
                and record.get("target") == profile["target"] and record.get("device") == device["id"]
                and isinstance(record.get("source_sha256"), str)
                and re.fullmatch(r"[0-9a-f]{64}", record["source_sha256"])
                and (not dma or record.get("dma_posture") == "VT-d contained")):
            return True
    return False


def physical(root: Path, paths: list[Path], profile: dict, device: dict, *, passed=False, dma=False) -> bool:
    """True when a referenced, fully validated hw-evidence record binds this profile, target and device.

    Every referenced hw-evidence record must validate; any defect raises and fails the matrix check.
    """
    found = False
    for path in paths:
        name = path.relative_to(root).as_posix()
        if not hw_evidence.is_record_path(name):
            continue
        summary = hw_evidence.validate_record(name, root)
        if (summary["profile"] == profile["id"] and summary["target"] == profile["target"]
                and device["id"] in summary["devices"] and (summary["passed"] or not passed)
                and (summary["dma_posture"] == "VT-d contained" or not dma)):
            found = True
    return found


def target_bound(root: Path, paths: list[Path], target: str | None) -> bool:
    """True when a docs/verification reference names the profile's machine (the target before '/')."""
    token = (target or "").split("/", 1)[0].strip().encode()
    for path in paths:
        if token and _verification(root, path):
            with path.open("rb") as stream:
                if token in stream.read(4 * 1024 * 1024):
                    return True
    return False


def owner_decisions(decisions: object, hw_target: str | None) -> int:
    if not isinstance(decisions, list) or not decisions:
        raise ValueError("owner decisions missing")
    seen, open_count = set(), 0
    for decision in decisions:
        if (not isinstance(decision, dict) or not {"id", "decision", "status", "blocks"} <= set(decision)
                or not set(decision) <= {"id", "decision", "status", "blocks", "reference"}):
            raise ValueError("invalid owner decision")
        ident, blocks = decision["id"], decision["blocks"]
        if ident not in OWNER_DECISIONS or ident in seen:
            raise ValueError("unknown or duplicate owner decision")
        seen.add(ident)
        if not isinstance(decision["decision"], str) or not decision["decision"].strip():
            raise ValueError("owner decision lacks a question")
        if (not isinstance(blocks, list) or not blocks or len(set(blocks)) != len(blocks)
                or not set(blocks) <= hw_evidence.PROTOCOL_IDS):
            raise ValueError("owner decision must block known physical protocols")
        if decision["status"] == "open":
            open_count += 1
        elif decision["status"] != "decided" or not isinstance(decision.get("reference"), str) \
                or not decision["reference"].strip():
            raise ValueError("a decided owner decision needs a reference")
    if seen != OWNER_DECISIONS:
        raise ValueError("required owner decision omitted")
    if hw_target is None and next(d for d in decisions if d["id"] == "OD-HW-TARGETS")["status"] != "open":
        raise ValueError("OD-HW-TARGETS cannot be decided while the HW-FR target is null")
    return open_count


def check(root: Path = ROOT) -> dict:
    root = root.resolve(strict=True)
    matrix = json.loads((root / MATRIX).read_text())
    if not isinstance(matrix, dict) or type(matrix.get("schema")) is not int or matrix["schema"] not in (1, 2):
        raise ValueError("unknown hardware matrix schema")
    schema = matrix["schema"]
    if set(matrix) != ({"schema", "profiles"} if schema == 1 else {"schema", "owner_decisions", "profiles"}):
        raise ValueError("unknown hardware matrix fields")
    profiles = matrix["profiles"]
    if not isinstance(profiles, list) or len(profiles) != len(PROFILES):
        raise ValueError("three required profiles missing")
    seen, count, totals = set(), 0, dict.fromkeys(STATUSES, 0)
    for profile in profiles:
        if not isinstance(profile, dict) or set(profile) != {"id", "kind", "target", "devices"}:
            raise ValueError("invalid profile")
        ident = profile["id"]
        if not isinstance(ident, str) or ident not in PROFILES or ident in seen or profile["kind"] != PROFILES[ident]:
            raise ValueError("duplicate/unknown/misclassified profile")
        seen.add(ident)
        kind, target = profile["kind"], profile["target"]
        if target is not None and (not isinstance(target, str) or not target.strip()):
            raise ValueError("invalid target identity")
        devices = profile["devices"]
        if not isinstance(devices, list) or not devices:
            raise ValueError("profile devices missing")
        names = set()
        for device in devices:
            if schema == 1:
                fields_ok = isinstance(device, dict) and set(device) == ROW_KEYS
            else:
                fields_ok = (isinstance(device, dict) and ROW_KEYS | {"evidence_level"} <= set(device)
                             <= ROW_KEYS | {"evidence_level", "note"})
            if not fields_ok:
                raise ValueError("invalid device/DMA/evidence fields")
            name, status, posture = device["id"], device["status"], device["dma_posture"]
            if not isinstance(name, str) or not name or name in names:
                raise ValueError("duplicate/empty device")
            names.add(name)
            if not isinstance(status, str) or status not in STATUSES or not isinstance(posture, str) or posture not in POSTURES:
                raise ValueError("unknown status/DMA posture")
            if "note" in device and (not isinstance(device["note"], str) or not device["note"].strip()):
                raise ValueError("empty device note")
            if device["driver_path"] is None:
                if status != "unsupported":
                    raise ValueError("absent driver must be unsupported")
            else:
                local_file(root, device["driver_path"])
            evidence = device["evidence"]
            if not isinstance(evidence, list) or len(evidence) > 16:
                raise ValueError("invalid evidence references")
            paths = [local_file(root, p) for p in evidence]
            level = device.get("evidence_level", "none")
            if schema == 2:
                if level not in LEVELS or level not in KIND_LEVELS[kind]:
                    raise ValueError(f"evidence level {level!r} cannot apply to a {kind} profile")
                if (level == "none") != (not evidence):
                    raise ValueError("evidence level and evidence references disagree")
                if level in VIRTUAL_LEVEL.values() and not all(_verification(root, p) for p in paths):
                    raise ValueError("native/VM evidence must reference docs/verification")
                if level == "physical" and not physical(root, paths, profile, device):
                    raise ValueError("physical evidence level lacks a validated signed record")
            if name == "xhci" and kind != "qemu" and status != "pending":
                raise ValueError(XHCI_LIMIT)
            if status == "native-verified":
                if schema == 1 or kind == "hardware":
                    raise ValueError("native-verified applies only to schema-2 QEMU/VMware rows")
                if level != VIRTUAL_LEVEL[kind] or not target_bound(root, paths, target):
                    raise ValueError("native-verified lacks target-bound native/VM evidence")
            if status == "supported":
                if kind != "hardware":
                    raise ValueError("QEMU/VM profiles may at most reach native-verified, never supported")
                if schema == 1:
                    raise ValueError("schema 1 cannot express supported; unsigned records are no longer accepted")
                if target is None:
                    raise ValueError("supported needs a selected HW-FR target (owner decision OD-HW-TARGETS)")
                if level != "physical" or not physical(root, paths, profile, device, passed=True):
                    raise ValueError("supported device lacks a validated, passing, signed physical record")
            if posture == "VT-d contained":
                bound = (physical(root, paths, profile, device, passed=True, dma=True) if kind == "hardware"
                         else proof(root, paths, profile, device, dma=True))
                if target is None or not bound:
                    raise ValueError("VT-d containment lacks target/device-bound evidence")
            totals[status] += 1
            count += 1
        if not FAMILIES <= names:
            raise ValueError("required device family omitted")
    hw_target = next(p["target"] for p in profiles if p["id"] == "HW-FR")
    open_decisions = owner_decisions(matrix["owner_decisions"], hw_target) if schema == 2 else None
    return dict(structurally_valid=True, schema=schema, profiles=len(seen), devices=count,
                native_verified=totals["native-verified"], supported=totals["supported"],
                owner_decisions_open=open_decisions,
                limit="Inventory/reference checks only; native-verified is source-bound QEMU/VM evidence and "
                      "supported needs an owner-signed physical record; no hardware acceptance claim.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--check", action="store_true", required=True)
    parser.parse_args()
    try:
        print(json.dumps(check(), indent=2))
    except (OSError, ValueError, TypeError, RuntimeError) as error:
        parser.exit(1, f"hardware matrix: {error}\n")

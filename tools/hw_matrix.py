"""Check hardware inventory references; this does not accept hardware or DMA isolation."""
from __future__ import annotations
import argparse
import json
from pathlib import Path, PurePosixPath
import re

ROOT = Path(__file__).resolve().parents[1]
MATRIX = Path("docs/drivers/hardware-matrix.json")
PROFILES = {"VM-FR": "qemu", "VM-VMW": "vmware", "HW-FR": "hardware"}
FAMILIES = {"gop", "ps2", "xhci", "nvme", "ahci", "e1000e", "hda", "acpi", "rng", "vtd"}
POSTURES = {"trusted kernel transport", "VT-d contained", "unsupported"}
STATUSES = {"implemented", "pending", "unsupported", "supported"}


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


def proof(root: Path, paths: list[Path], profile: dict, device: dict, *, dma=False) -> bool:
    for path in paths:
        if path.relative_to(root).parts[:2] != ("docs", "verification") or path.suffix != ".json":
            continue
        with path.open("rb") as stream:
            raw = stream.read(1024 * 1024 + 1)
        if len(raw) > 1024 * 1024:
            raise ValueError("oversize hardware evidence")
        record = json.loads(raw)
        if (isinstance(record, dict) and record.get("passed") is True
                and record.get("level") == ("H" if profile["kind"] == "hardware" else "native")
                and record.get("target") == profile["target"] and record.get("device") == device["id"]
                and isinstance(record.get("source_sha256"), str)
                and re.fullmatch(r"[0-9a-f]{64}", record["source_sha256"])
                and (not dma or record.get("dma_posture") == "VT-d contained")):
            return True
    return False


def check(root: Path = ROOT) -> dict:
    root = root.resolve(strict=True)
    matrix = json.loads((root / MATRIX).read_text())
    if not isinstance(matrix, dict) or set(matrix) != {"schema", "profiles"} or type(matrix["schema"]) is not int or matrix["schema"] != 1:
        raise ValueError("unknown hardware matrix schema")
    profiles = matrix["profiles"]
    if not isinstance(profiles, list) or len(profiles) != len(PROFILES):
        raise ValueError("three required profiles missing")
    seen, count = set(), 0
    for profile in profiles:
        if not isinstance(profile, dict) or set(profile) != {"id", "kind", "target", "devices"}:
            raise ValueError("invalid profile")
        ident = profile["id"]
        if not isinstance(ident, str) or ident not in PROFILES or ident in seen or profile["kind"] != PROFILES[ident]:
            raise ValueError("duplicate/unknown/misclassified profile")
        seen.add(ident)
        target = profile["target"]
        if target is not None and (not isinstance(target, str) or not target.strip()):
            raise ValueError("invalid target identity")
        devices = profile["devices"]
        if not isinstance(devices, list) or not devices:
            raise ValueError("profile devices missing")
        names = set()
        for device in devices:
            if not isinstance(device, dict) or set(device) != {"id", "driver_path", "dma_posture", "status", "evidence"}:
                raise ValueError("invalid device/DMA fields")
            name, status, posture = device["id"], device["status"], device["dma_posture"]
            if not isinstance(name, str) or not name or name in names:
                raise ValueError("duplicate/empty device")
            names.add(name)
            if not isinstance(status, str) or status not in STATUSES or not isinstance(posture, str) or posture not in POSTURES:
                raise ValueError("unknown status/DMA posture")
            if device["driver_path"] is None:
                if status != "unsupported":
                    raise ValueError("absent driver must be unsupported")
            else:
                local_file(root, device["driver_path"])
            evidence = device["evidence"]
            if not isinstance(evidence, list) or len(evidence) > 16:
                raise ValueError("invalid evidence references")
            paths = [local_file(root, p) for p in evidence]
            if name == "xhci" and status != "pending":
                raise ValueError("xHCI remains pending its separate integration/acceptance")
            if status == "supported" and (target is None or not proof(root, paths, profile, device)):
                raise ValueError("supported device lacks target-bound H/native evidence")
            if posture == "VT-d contained" and (target is None or not proof(root, paths, profile, device, dma=True)):
                raise ValueError("VT-d containment lacks target/device-bound evidence")
            count += 1
        if not FAMILIES <= names:
            raise ValueError("required device family omitted")
    return dict(structurally_valid=True, profiles=len(seen), devices=count,
                limit="Inventory/reference checks only; no hardware acceptance or evidence authenticity claim.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", required=True)
    parser.parse_args()
    try:
        print(json.dumps(check(), indent=2))
    except (OSError, ValueError, TypeError, RuntimeError) as error:
        parser.exit(1, f"hardware matrix: {error}\n")

"""Check hardware inventory references; this does not accept hardware or DMA isolation.

Schema 2 records an explicit evidence level per device row. QEMU profiles can
reach at most `native-verified`, and only with a referenced, passing native QEMU
report bound to the profile's machine and to a device family listed in
NATIVE_BINDINGS; VMware rows cannot reach it until a VMware report format exists
in code. A `VT-d contained` posture on a QEMU row needs the same bound report from
a family whose binding observes a `*:BUS_MASTER:CONTAINED` marker; hand-written
records never bind it, and VMware rows cannot claim it. `supported` exists only
for the physical profile and requires a passing record that tools/hw_evidence.py
validates (owner-signed, protocol/target/hash-bound, not a virtual capture).
Every referenced hw-evidence record must validate and bind its row. Schema 1
matrices remain readable, but can no longer express `supported`. The physical protocol catalogue is validated as well, so a `ready`
protocol with an open owner decision fails this check.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path, PurePosixPath
import re
from typing import NamedTuple

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
OWNER_DECISIONS = hw_evidence.OWNER_DECISIONS


class Binding(NamedTuple):
    """How native QEMU run sets relate to, and positively demonstrate, one device family."""
    selector: str            # "required_marker" or "kernel_feature": which run sets concern the family
    value: str               # the marker every guest requires, or the exact kernel feature of the run set
    required: tuple          # markers every guest must also require for the positive case
    observed: tuple          # markers every run must have observed
    forbidden: tuple         # negative-case marker prefixes that must not appear in a binding run set


# Device families whose native QEMU evidence can bind a `native-verified` row. Every related run set must pass, or
# the check raises. A related run set binds only as the positive case: every guest also requires `required`, every
# run observed all `observed` markers, and no required or observed marker starts with a `forbidden` prefix (such a
# run set raises). Related negative profiles (xHCI DMA fault injection, missing DMAR) therefore never bind, although
# they also end with XHCI:PASS. A family without an entry (gop, ps2, rng, virtio-*, ...) cannot be native-verified
# until a reviewed binding is added.
XHCI_POSITIVE = ("XHCI:BUS_MASTER:CONTAINED", "XHCI:NOOP_COMPLETION:PASS", "XHCI:PASS")
NATIVE_BINDINGS = {
    "xhci": Binding("required_marker", "XHCI:PASS", XHCI_POSITIVE, XHCI_POSITIVE,
                    ("XHCI:NO_DMAR:", "XHCI:DMA_FAULT:", "XHCI:FAIL")),
    "acpi": Binding("kernel_feature", "native-acpi-s5", (), ("POWER:S5:WRITE",), ()),
}
MAX_REPORT = 4 * 1024 * 1024
HEX64 = re.compile(r"[0-9a-f]{64}")
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
    if path.relative_to(root).as_posix() != name:
        raise ValueError(f"{name} is a symlink or aliased path (fail closed)")
    return path


def _verification(root: Path, path: Path) -> bool:
    return path.relative_to(root).parts[:2] == ("docs", "verification")


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


def _report(path: Path) -> object:
    with path.open("rb") as stream:
        raw = stream.read(MAX_REPORT + 1)
    if len(raw) > MAX_REPORT:
        raise ValueError(f"{path.name}: oversize native evidence report")
    try:
        return json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError(f"{path.name}: native evidence report is not valid JSON: {error}") from error


def _observed(value: object) -> list | None:
    if isinstance(value, dict):
        value = value.get("markers")
    return value if isinstance(value, list) else None


def _required(guest: dict) -> list:
    value = guest.get("required_markers")
    return value if isinstance(value, list) else []


def _run_set(unit: dict, machine: str, binding: Binding) -> str | None:
    """Classify one report unit for this machine and device.

    None (unrelated), 'fail' (a related run failed), 'other-case' (a related run set that is not the positive case,
    such as a negative profile), 'contradiction' (a positive run set carrying a negative-case marker), 'incomplete'
    or 'pass'. Only 'pass' binds.
    """
    if "runs" in unit:
        runs = unit["runs"]
        if not isinstance(runs, list) or not runs or not all(isinstance(run, dict) for run in runs):
            return None
        runs = [(run.get("guest"), run.get("observed"), run.get("passed")) for run in runs]
    elif "guest" in unit:
        runs = [(unit["guest"], unit.get("observed"), True)]
    else:
        return None
    guests = [guest for guest, _, _ in runs]
    if not all(isinstance(guest, dict) for guest in guests):
        return None
    machines = []
    for guest in guests:
        command = guest.get("command")
        if not isinstance(command, list) or not all(isinstance(arg, str) for arg in command):
            return None
        machines.append([command[i + 1].split(",", 1)[0] for i in range(len(command) - 1)
                         if command[i] == "-machine"])
    if binding.selector == "kernel_feature":
        device = unit.get("kernel_feature") == binding.value
    else:
        device = all(binding.value in _required(g) for g in guests)
    if not device or not any(machine in found for found in machines):
        return None
    if (unit.get("passed") is not True or any(passed is not True for _, _, passed in runs)
            or any(g.get("passed") is not True or g.get("timed_out") is not False
                   or type(g.get("raw_qemu_exit")) is not int or type(g.get("expected_exit")) is not int
                   or g["raw_qemu_exit"] != g["expected_exit"] for g in guests)):
        return "fail"
    if not all(marker in _required(g) for g in guests for marker in binding.required):
        return "other-case"
    observed = [_observed(found) for _, found, _ in runs]
    markers = [m for g in guests for m in _required(g)] + [m for found in observed if found for m in found]
    if any(isinstance(m, str) and m.startswith(binding.forbidden) for m in markers):
        return "contradiction"
    source = unit.get("source_sha256")
    if (not isinstance(source, str) or not HEX64.fullmatch(source)
            or any(g.get("source_sha256") != source for g in guests)
            or any(found != [machine] for found in machines)
            or any(found is None or any(m not in found for m in binding.observed) for found in observed)):
        return "incomplete"
    return "pass"


def native_bound(root: Path, paths: list[Path], profile: dict, device: dict) -> bool:
    """True when a referenced docs/verification JSON report holds a passing run set bound to this row.

    A run set is the top-level report or one entry of its `profiles` object. It binds when the report and every
    run and guest passed (no timeout, raw exit equal to the expected exit), every guest ran with
    `-machine <profile machine>` and the report's source hash, and it is the family's positive case (see
    NATIVE_BINDINGS). A related run set that failed raises: one failed run contradicts the claim. A passing related
    negative profile does not bind, and a positive run set carrying a negative-case marker raises. Prose files and
    other machines never bind.
    """
    binding = NATIVE_BINDINGS.get(device["id"])
    machine = (profile["target"] or "").split("/", 1)[0].strip()
    if profile["kind"] != "qemu" or binding is None or not machine:
        return False
    bound = False
    for path in paths:
        if not _verification(root, path) or path.suffix != ".json":
            continue
        report = _report(path)
        if not isinstance(report, dict):
            continue
        units = [report] + (list(report["profiles"].values()) if isinstance(report.get("profiles"), dict) else [])
        for unit in units:
            state = _run_set(unit, machine, binding) if isinstance(unit, dict) else None
            if state == "fail":
                raise ValueError(f"native-verified evidence includes a failed run for {device['id']}: "
                                 f"{path.relative_to(root).as_posix()}")
            if state == "contradiction":
                raise ValueError(f"native-verified evidence for {device['id']} pairs the positive case with a "
                                 f"negative-case marker: {path.relative_to(root).as_posix()}")
            bound = bound or state == "pass"
    return bound


def contained_bound(root: Path, paths: list[Path], profile: dict, device: dict) -> bool:
    """True when native QEMU evidence demonstrates a virtual row's `VT-d contained` posture.

    The row needs `native-qemu` evidence, its family's native binding must require and observe a
    `*:BUS_MASTER:CONTAINED` marker, and a referenced report must bind as for `native-verified` (passing run set,
    profile machine, source hash, positive case). Hand-written records and VMware rows never qualify.
    """
    binding = NATIVE_BINDINGS.get(device["id"])
    if (binding is None or device.get("evidence_level") != VIRTUAL_LEVEL.get(profile["kind"])
            or not any(marker.endswith(hw_evidence.CONTAINED_SUFFIX) for marker in binding.observed)
            or not any(marker.endswith(hw_evidence.CONTAINED_SUFFIX) for marker in binding.required)):
        return False
    return native_bound(root, paths, profile, device)


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
        if not set(blocks) >= hw_evidence.REQUIRED_BLOCKS[ident]:
            raise ValueError(f"{ident} blocks fewer protocols than the documented minimum "
                             f"({', '.join(sorted(hw_evidence.REQUIRED_BLOCKS[ident]))})")
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
            for reference, path in zip(evidence, paths):
                relative = path.relative_to(root).as_posix()
                if hw_evidence.is_record_path(reference) or hw_evidence.is_record_path(relative):
                    summary = hw_evidence.validate_record(relative, root)
                    if summary["profile"] != ident or summary["target"] != target or name not in summary["devices"]:
                        raise ValueError(f"referenced hw-evidence record {relative} does not bind this "
                                         f"profile/target/device ({ident}/{name})")
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
                if level != VIRTUAL_LEVEL[kind] or not native_bound(root, paths, profile, device):
                    raise ValueError("native-verified lacks a passing, machine- and device-bound native QEMU report "
                                     "(VMware rows and families without a native binding cannot qualify)")
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
                         else contained_bound(root, paths, profile, device))
                if target is None or not bound:
                    raise ValueError("VT-d containment lacks target/device-bound evidence (a validated signed "
                                     "physical record, or a passing machine- and device-bound native QEMU report "
                                     "with a *:BUS_MASTER:CONTAINED marker)")
            totals[status] += 1
            count += 1
        if not FAMILIES <= names:
            raise ValueError("required device family omitted")
    hw_target = next(p["target"] for p in profiles if p["id"] == "HW-FR")
    open_decisions = owner_decisions(matrix["owner_decisions"], hw_target) if schema == 2 else None
    # A ready protocol must not coexist with an open owner decision, even when no record references it.
    hw_evidence.load_protocols(root)
    return dict(structurally_valid=True, schema=schema, profiles=len(seen), devices=count,
                native_verified=totals["native-verified"], supported=totals["supported"],
                owner_decisions_open=open_decisions,
                limit="Inventory/reference checks only; native-verified needs a passing, machine- and "
                      "device-bound native QEMU report and supported needs an owner-signed physical record; "
                      "no hardware acceptance claim.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--check", action="store_true", required=True)
    parser.parse_args()
    try:
        print(json.dumps(check(), indent=2))
    except (OSError, ValueError, TypeError, RuntimeError) as error:
        parser.exit(1, f"hardware matrix: {error}\n")

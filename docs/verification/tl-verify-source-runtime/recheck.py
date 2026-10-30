#!/usr/bin/env python3
"""Recompute source-runtime evidence; never certify byte reproducibility."""
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools"))
import run_materiaal_preview as preview
import run_materiaal_probe as probe
from common import source_hash

DEST = Path(__file__).resolve().parent
SOURCE = "24d5f4c112924cfe45681de3fcf3a92f62087e29524f15c74e14d83620891abd"
REFERENCE = ROOT / "build/tl-verify-image-1/reference/run/materiaal-normal-a69441f209"
ORDINARY = ROOT / "build/tl-verify-image-1/interactive/preview-run/materiaal-3d940fcfdd"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def main():
    require(source_hash() == SOURCE, "Source drift")
    reference = json.loads((REFERENCE / "result.json").read_text())
    ordinary = json.loads((ORDINARY / "result.json").read_text())
    require(reference["passed"] is True and ordinary["passed"] is True, "Failed gate")
    require(reference["raw_qemu_exit"] == 33 and ordinary["raw_qemu_exit"] == 0,
            "Wrong actual QEMU exits")
    probe.replay(reference, REFERENCE, SOURCE)
    manifests = [
        json.loads((REFERENCE / "inputs/manifest.json").read_text()),
        json.loads((ROOT / "build/tl-verify-image-1/interactive/manifest.json").read_text()),
    ]
    preview.reference_binding(reference, *manifests, SOURCE, "pc-q35-10.0")
    require(ordinary["source_sha256"] == SOURCE, "Wrong ordinary source")
    require(ordinary["kernel_sha256"] == manifests[1]["kernel"]["sha256"]
            and ordinary["loader_sha256"] == manifests[1]["loader"]["sha256"],
            "Wrong ordinary executables")
    require(digest(ORDINARY / "esp/kernel.elf") == ordinary["kernel_sha256"]
            and digest(ORDINARY / "esp/EFI/BOOT/BOOTX64.EFI") == ordinary["loader_sha256"],
            "Ordinary ESP executable drift")
    for app in manifests[1]["materiaal"]["apps"]:
        require(digest(ROOT / "build/tl-verify-image-1/interactive/users/materiaal"
                       / f"{app['name']}.elf") == app["sha256"], "App ELF drift")
    frames = {}
    for phase in probe.PHASES["normal"]:
        left, right = REFERENCE / f"{phase}.ppm", ORDINARY / f"{phase}.ppm"
        require(left.read_bytes() == right.read_bytes(), f"Full frame mismatch: {phase}")
        frames[phase] = digest(right)
    disk = probe.disk_oracle((ORDINARY / "volume.after.img").read_bytes())
    require(disk == ordinary["interactive"]["disk"], "Disk oracle mismatch")
    require((ORDINARY / "volume.after.img").read_bytes() == (ORDINARY / "volume.img").read_bytes(),
            "After/current disk mismatch")
    summaries = {}
    for name, directory, report in [
        ("reference", REFERENCE, reference), ("ordinary", ORDINARY, ordinary)
    ]:
        summary = {k: v for k, v in report.items() if k != "qmp_trace"}
        summary["original_report_path"] = str(directory / "result.json")
        summary["original_report_sha256"] = digest(directory / "result.json")
        summary["serial_sha256"] = digest(directory / "serial.txt")
        if "qmp_trace" in report:
            encoded = json.dumps(report["qmp_trace"], sort_keys=True).encode()
            summary["qmp_trace_canonical_sha256"] = hashlib.sha256(encoded).hexdigest()
            summary["qmp_trace_entries"] = len(report["qmp_trace"])
        summaries[name] = summary
    require(source_hash() == SOURCE, "Source changed during recheck")
    for name, summary in summaries.items():
        (DEST / f"{name}-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    for name, directory in [("reference", REFERENCE), ("ordinary", ORDINARY)]:
        (DEST / f"{name}-serial.txt").write_bytes((directory / "serial.txt").read_bytes())
    logs = {}
    for suffix in ["reference", "reference-canonical", "interactive"]:
        path = Path(f"/tmp/nieuw-os-tl-verify-source-runtime-{suffix}.log")
        logs[str(path)] = {"sha256": digest(path), "bytes": path.stat().st_size}
    (DEST / "recheck.json").write_text(json.dumps({
        "scope": "same-source functional runtime only; byte reproducibility remains blocked",
        "source_sha256": SOURCE, "passed": True, "frames": frames, "disk": disk,
        "reference_report_sha256": summaries["reference"]["original_report_sha256"],
        "ordinary_report_sha256": summaries["ordinary"]["original_report_sha256"],
        "logs": logs,
    }, indent=2) + "\n")
    print("PASS: native replay, seven app identities, six full frames, post-exit disk, source")


if __name__ == "__main__":
    main()

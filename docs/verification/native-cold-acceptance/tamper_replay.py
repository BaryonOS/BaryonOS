#!/usr/bin/env python3
"""Replay real cold evidence; tamper only temporary copies of its parent."""
from __future__ import annotations

import argparse
import copy
import json
from pathlib import Path
import shutil
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools"))
import materiaal_cold as cold
import run_materiaal_preview as preview


def verify(build: Path) -> dict:
    build, manifest = preview.validate_build(build)
    frozen = preview.source_hash()
    report_path = preview.private_path(build / "cold-result.json", regular=True)
    report = preview.load_json(report_path, "cold report", 16 * 1024 * 1024)
    directory = preview.private_path(Path(report["run_directory"])).resolve(strict=True)
    parent_directory = Path(report["parent_run"])
    reference_directory = Path(preview.load_json(
        parent_directory / "result.json", "parent", 16 * 1024 * 1024
    )["interactive_reference"]["run_directory"])
    originals = [
        path for root in (directory, parent_directory, reference_directory)
        for path in root.rglob("*") if path.is_file()
    ]
    before = {str(path): preview.sha256(path) for path in originals}
    before[str(report_path)] = preview.sha256(report_path)
    reference = preview.load_reference(
        reference_directory, manifest, frozen, report["machine"],
        report["firmware"], report["qemu_entry_sha256"],
    )
    args = (manifest, reference, frozen, report["machine"],
            report["firmware"], report["qemu_entry_sha256"])
    parent = cold.replay_parent(parent_directory, *args)
    expected = {
        "list": cold.cold_list_frame(reference["frames"]),
        "reopened": reference["frames"]["reopened"],
    }
    if (report.get("passed") is not True or type(report.get("raw_qemu_exit")) is not int
            or report["raw_qemu_exit"] != 0 or report.get("guest_clean_shutdown") is not False
            or report.get("source_sha256") != frozen or manifest["source_sha256"] != frozen
            or report.get("kernel_sha256") != manifest["kernel"]["sha256"]
            or report.get("loader_sha256") != manifest["loader"]["sha256"]
            or report.get("apps") != reference["apps"]
            or report.get("qemu_implementation_sha256") != reference["qemu_implementation_sha256"]
            or report.get("reference_report_sha256") != reference["report_sha256"]
            or report.get("parent_report_sha256") != parent["report_sha256"]
            or report.get("pixel_masks") != [] or report.get("volume_unchanged") is not True):
        raise ValueError("Cold report binding/exit/scope differs")
    cold.replay_input_trace(report["qmp_trace"], report["stages"], directory, cold=True)
    for stage in report["stages"]:
        frame = preview.probe.frame_bytes(directory, stage["phase"])
        if (frame != expected[stage["phase"]]
                or preview.sha256(directory / (stage["phase"] + ".ppm")) != stage["screenshot_sha256"]):
            raise ValueError("Cold exact image differs")
    for name in ("volume.img", "volume.before.img", "volume.after.img"):
        if preview.private_path(directory / name, regular=True).read_bytes() != parent["disk"]:
            raise ValueError("Cold disk bytes differ")
    if (preview.probe.disk_oracle(parent["disk"]) != report["disk"]
            or preview.sha256(directory / "volume.img") != report["before_sha256"]
            or report["before_sha256"] != report["after_sha256"]
            or preview.sha256(directory / "serial.txt") != report["serial_sha256"]):
        raise ValueError("Cold disk oracle/serial binding differs")
    preview.verify_ready_transcript(preview.read_serial(directory / "serial.txt"))
    for relative, digest in (
        ("esp/kernel.elf", manifest["kernel"]["sha256"]),
        ("esp/EFI/BOOT/BOOTX64.EFI", manifest["loader"]["sha256"]),
        ("OVMF_CODE.fd", report["firmware"]["ovmf_code_sha256"]),
        ("OVMF_VARS.template.fd", report["firmware"]["ovmf_vars_template_sha256"]),
        ("bios-256k.bin", report["firmware"]["seabios_bios_sha256"]),
        ("VGA.rom", report["firmware"]["seabios_vga_sha256"]),
    ):
        if preview.sha256(preview.private_path(directory / relative, regular=True)) != digest:
            raise ValueError("Cold frozen artifact differs")

    def relocate(value, old: str, new: str):
        if isinstance(value, str):
            return new + value[len(old):] if value.startswith(old + "/") or value == old else value
        if isinstance(value, dict):
            return {key: relocate(item, old, new) for key, item in value.items()}
        if isinstance(value, list):
            return [relocate(item, old, new) for item in value]
        return value

    rejected = []
    with tempfile.TemporaryDirectory(prefix="cold-negative-", dir=build) as name:
        clone = Path(name) / "parent"
        shutil.copytree(parent_directory, clone)
        path = clone / "result.json"
        baseline = relocate(json.loads(path.read_text()), str(parent_directory), str(clone))

        def publish(value):
            path.write_text(json.dumps(value) + "\n")

        publish(baseline)
        cold.replay_parent(clone, *args)
        for case in ("boolean-exit", "wrong-source", "wrong-reference", "failed-response",
                     "extra-command", "disk-tamper", "frame-tamper", "kernel-tamper"):
            altered = copy.deepcopy(baseline)
            restore = None
            if case == "boolean-exit":
                altered["raw_qemu_exit"] = False
            elif case == "wrong-source":
                altered["source_sha256"] = "0" * 64
            elif case == "wrong-reference":
                altered["interactive_reference"]["report_sha256"] = "0" * 64
            elif case == "failed-response":
                altered["qmp_trace"][0]["response"] = {"error": {"desc": "test"}, "id": 1}
            elif case == "extra-command":
                altered["qmp_trace"][-1]["request"]["execute"] = "system_reset"
            else:
                target = clone / {
                    "disk-tamper": "volume.img",
                    "frame-tamper": "reopened.ppm",
                    "kernel-tamper": "esp/kernel.elf",
                }[case]
                raw = target.read_bytes()
                restore = (target, raw)
                target.write_bytes(raw[:-1] + bytes([raw[-1] ^ 1]))
            publish(altered)
            try:
                cold.replay_parent(clone, *args)
            except ValueError as error:
                rejected.append({"case": case, "rejected": True, "error": str(error)})
            else:
                raise AssertionError("Tampered real parent accepted: " + case)
            finally:
                if restore:
                    restore[0].write_bytes(restore[1])
        publish(baseline)
        cold.replay_parent(clone, *args)
    if (preview.source_hash() != frozen
            or any(preview.sha256(Path(path)) != digest for path, digest in before.items())):
        raise ValueError("Original source/evidence changed during negative replay")
    return {
        "passed": True, "scope": "Offline real-artifact replay and temporary-copy negatives; no VM",
        "source_sha256": frozen, "cold_report_sha256": preview.sha256(report_path),
        "parent_report_sha256": parent["report_sha256"],
        "reference_report_sha256": reference["report_sha256"],
        "cold_trace_entries": len(report["qmp_trace"]),
        "original_files_unchanged": len(before), "negatives": rejected,
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build-dir", required=True, type=Path)
    args = parser.parse_args()
    print(json.dumps(verify(args.build_dir), indent=2))

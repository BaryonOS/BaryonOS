"""Strict parent replay and complete expected pixels for the ordinary cold GUI."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import tempfile

import run_materiaal_preview as preview


def cold_list_frame(frames: dict[str, bytes]) -> bytes:
    preview.probe.validate_frames(frames, "normal")
    listed = frames["list"]
    width, height, pixels = preview.ppm(listed)
    rw, rh, ready = preview.ppm(frames["ready"])
    if (width, height, rw, rh) != (800, 600, 800, 600):
        raise ValueError("Cold reference requires the fixed scale1 native geometry")
    # Layout::status: PAD=24, height-48, width-48, 48 at scale1, translated
    # through ASSIGNED=(16,48,768,536). A fresh controller retains Ready after
    # refresh. Every expected status pixel is borrowed from native Ready;
    # every other pixel remains the populated native list. Nothing is masked.
    result = bytearray(pixels)
    for y in range(536, 584):
        start, end = (y * width + 40) * 3, (y * width + 760) * 3
        result[start:end] = ready[start:end]
    return listed[:-len(pixels)] + result


def cold_groups(directory: Path) -> list:
    normal = preview.interactive_groups(directory)
    moves = normal[1][1][:2]
    expected_moves = [
        {"execute": "input-send-event", "arguments": {"events": [
            {"type": "rel", "data": {"axis": "x", "value": 56}},
            {"type": "rel", "data": {"axis": "y", "value": 96}},
        ]}},
        {"execute": "input-send-event", "arguments": {"events": [
            {"type": "rel", "data": {"axis": "y", "value": 54}},
        ]}},
    ]
    if moves != expected_moves:
        raise ValueError("Review changed native cold cursor placement contract")
    return [
        ("list", [*moves, {"execute": "screendump", "arguments": {
            "filename": str(directory / "list.ppm"),
        }}]),
        normal[-1],
    ]


def replay_input_trace(trace: object, stages: object, directory: Path, *, cold=False) -> None:
    if not isinstance(trace, list) or not isinstance(stages, list) or not trace:
        raise ValueError("Missing interactive trace/stages")
    for index, item in enumerate(trace, 1):
        if not isinstance(item, dict) or set(item) != {"request", "response"}:
            raise ValueError("Malformed QMP trace entry")
        request = item["request"]
        if (not isinstance(request, dict) or type(request.get("id")) is not int
                or request["id"] != index
                or json.dumps(item["response"], sort_keys=True, allow_nan=False)
                != json.dumps({"return": {}, "id": index}, sort_keys=True)):
            raise ValueError("Failed or uncorrelated QMP response")

    def request_at(index: int) -> dict:
        if index >= len(trace):
            raise ValueError("Truncated interactive input trace")
        return {key: value for key, value in trace[index]["request"].items() if key != "id"}

    if request_at(0) != {"execute": "qmp_capabilities"}:
        raise ValueError("Missing QMP handshake")
    groups = cold_groups(directory) if cold else preview.interactive_groups(directory)
    if len(stages) != len(groups):
        raise ValueError("Missing or extra parent stage")
    position = 1
    for (name, group), stage in zip(groups, stages, strict=True):
        if not isinstance(stage, dict) or stage.get("phase") != name:
            raise ValueError("Wrong parent phase order")
        for expected in group:
            if request_at(position) != expected:
                raise ValueError("Altered hardware input or missing frame")
            position += 1
            if expected["execute"] == "screendump":
                while position < len(trace) and request_at(position) == expected:
                    position += 1
                if type(stage.get("qmp_index")) is not int or stage["qmp_index"] != position:
                    raise ValueError("Parent frame not bound to final phase observation")
    if position != len(trace) - 1 or request_at(position) != {"execute": "quit"}:
        raise ValueError("Missing final quit or extra interactive command")


def replay_parent(directory: Path, manifest: dict, reference: dict, frozen: str,
                  machine: str, firmware: dict, qemu_hash: str) -> dict:
    directory = preview.private_path(directory).resolve(strict=True)
    path = preview.private_path(directory / "result.json", regular=True)
    report_hash = preview.sha256(path)
    report = preview.load_json(path, "ordinary parent report", 16 * 1024 * 1024)
    if (report.get("passed") is not True or type(report.get("raw_qemu_exit")) is not int
            or report["raw_qemu_exit"] != 0
            or report.get("source_sha256") != frozen
            or type(report.get("source_fingerprint_version")) is not int
            or report["source_fingerprint_version"] != 3
            or report.get("kernel_feature") != "native-system-materiaal"
            or report.get("kernel_sha256") != manifest["kernel"]["sha256"]
            or report.get("loader_sha256") != manifest["loader"]["sha256"]
            or report.get("machine") != machine or report.get("firmware") != firmware
            or report.get("qemu_sha256") != qemu_hash
            or report.get("qemu_implementation_sha256") != reference["qemu_implementation_sha256"]):
        raise ValueError("Parent is not the same-source ordinary exited native run")
    binding = report.get("interactive_reference", {})
    if (binding.get("report_sha256") != reference["report_sha256"]
            or binding.get("source_sha256") != frozen
            or binding.get("apps") != reference["apps"]):
        raise ValueError("Parent native reference binding differs")
    interactive = report.get("interactive", {})
    if interactive.get("pixel_masks") != []:
        raise ValueError("Parent used a pixel mask")
    replay_input_trace(report.get("qmp_trace"), interactive.get("stages"), directory)
    for stage in interactive["stages"]:
        phase = stage["phase"]
        frame = preview.probe.frame_bytes(directory, phase)
        if (frame != reference["frames"][phase]
                or hashlib.sha256(frame).hexdigest() != stage.get("screenshot_sha256")):
            raise ValueError("Parent frame or recorded frame hash differs")
    preview.verify_ready_transcript(preview.read_serial(directory / "serial.txt"))
    if preview.sha256(directory / "serial.txt") != report.get("serial_sha256"):
        raise ValueError("Parent serial changed")
    for relative, expected in (
        ("esp/kernel.elf", manifest["kernel"]["sha256"]),
        ("esp/EFI/BOOT/BOOTX64.EFI", manifest["loader"]["sha256"]),
    ):
        if preview.sha256(preview.private_path(directory / relative, regular=True)) != expected:
            raise ValueError("Parent executable changed")
    for relative, field in (
        ("OVMF_CODE.fd", "ovmf_code_sha256"),
        ("OVMF_VARS.template.fd", "ovmf_vars_template_sha256"),
        ("bios-256k.bin", "seabios_bios_sha256"),
        ("VGA.rom", "seabios_vga_sha256"),
    ):
        if preview.sha256(preview.private_path(directory / relative, regular=True)) != firmware[field]:
            raise ValueError("Parent immutable firmware changed")
    before = preview.private_path(directory / "volume.before.img", regular=True).read_bytes()
    with tempfile.TemporaryDirectory(prefix="nieuw-os-cold-parent-") as name:
        empty_path = Path(name) / "empty.img"
        preview.volume_disk.create_image(empty_path, "empty")
        if before != empty_path.read_bytes():
            raise ValueError("Parent did not start with the canonical empty image")
    after_path = preview.private_path(directory / "volume.after.img", regular=True)
    after = after_path.read_bytes()
    if (after != preview.private_path(directory / "volume.img", regular=True).read_bytes()
            or hashlib.sha256(after).hexdigest() != interactive.get("after_sha256")
            or hashlib.sha256(before).hexdigest() != interactive.get("before_sha256")
            or preview.probe.disk_oracle(after) != interactive.get("disk")):
        raise ValueError("Parent disk bytes or decoded content changed")
    if preview.sha256(path) != report_hash:
        raise ValueError("Parent report changed during replay")
    return {"directory": directory, "report_sha256": report_hash, "disk": after}

#!/usr/bin/env python3
"""Recompute archived Windows inputs, exact frame matches and raw disk content."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools"))
import run_materiaal_preview as preview
import run_materiaal_probe as probe
from common import source_hash


def recheck(run: Path, reference: Path, bundle: Path) -> dict:
    report = json.loads((run / "result.json").read_text())
    trace = json.loads((run / "qmp-trace.json").read_text())
    if (report["passed"] is not True or type(report["raw_qemu_exit"]) is not int
            or report["raw_qemu_exit"] != 0 or report["qmp_quit_sent"] is not True):
        raise ValueError("Not a passing Windows QMP-quit/raw0 run")
    state = bundle / ("proof-state-" + run.name)
    for index, item in enumerate(trace, 1):
        if (type(item["request"]["id"]) is not int or item["request"]["id"] != index
                or type(item["response"]["id"]) is not int
                or item["response"] != {"return": {}, "id": index}):
            raise ValueError("Uncorrelated or failed QMP response")

    def request(index):
        return {key: value for key, value in trace[index]["request"].items() if key != "id"}

    if request(0) != {"execute": "qmp_capabilities"}:
        raise ValueError("No QMP handshake")
    position = 1
    for (name, group), stage in zip(
        preview.interactive_groups(state), report["stages"], strict=True
    ):
        if stage["phase"] != name:
            raise ValueError("Wrong captured phase order")
        for item in group:
            expected = copy.deepcopy(item)
            if expected["execute"] == "screendump":
                expected["arguments"]["filename"] = state.name + "/" + name + ".ppm"
                if request(position) != expected:
                    raise ValueError("Missing real screendump")
                while position < len(trace) and request(position) == expected:
                    position += 1
                if stage["qmp_index"] != position:
                    raise ValueError("Captured frame is not bound to its QMP command")
            else:
                if request(position) != expected:
                    raise ValueError("Hardware input differs from specified chain")
                position += 1
        raw = (run / (name + ".ppm")).read_bytes()
        if (hashlib.sha256(raw).hexdigest() != stage["sha256"]
                or raw != (reference / (name + ".ppm")).read_bytes()):
            raise ValueError("Full native reference image mismatch")
    if position != len(trace) - 1 or request(position) != {"execute": "quit"}:
        raise ValueError("Extra command or missing final QMP quit")
    if any(key.get("data") == "f12" for item in trace
           for key in item["request"].get("arguments", {}).get("keys", [])):
        raise ValueError("Forbidden diagnostic F12 input")
    if (run / "volume.before.img").read_bytes() != (bundle / "volume.empty.img").read_bytes():
        raise ValueError("Beforeimage was not canonical empty fixture")
    if (run / "volume.after.img").read_bytes() != (state / "volume.img").read_bytes():
        raise ValueError("Active Windows disk differs from afterimage")
    disk = probe.disk_oracle((run / "volume.after.img").read_bytes())
    preview.verify_ready_transcript((run / "serial.txt").read_bytes())
    if disk != report["disk"] or source_hash() != report["source_sha256"]:
        raise ValueError("Disk result or current source differs")
    adapter = Path(__file__).with_name("windows_qmp_adapter.py")
    if hashlib.sha256(adapter.read_bytes()).hexdigest() != report["adapter_sha256"]:
        raise ValueError("Executed adapter identity differs")
    return {
        "passed": True, "exact_qmp_sequence_recomputed": True, "qmp_commands": len(trace),
        "f12_absent": True, "all_six_full_frames_byte_equal": True, "post_quit_raw_exit": 0,
        "canonical_empty_before": True, "active_disk_equal_after": True,
        "disk_recomputed": disk, "source_unchanged": True, "executed_adapter_hash_verified": True,
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", type=Path, required=True)
    parser.add_argument("--reference", type=Path, required=True)
    parser.add_argument("--bundle", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(recheck(args.run, args.reference, args.bundle), indent=2))

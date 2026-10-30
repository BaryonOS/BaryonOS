#!/usr/bin/env python3
"""Exercise unchanged acceptance helpers against mutations of real evidence."""
import copy
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools"))
import run_materiaal_preview as preview
import run_materiaal_probe as probe
from common import source_hash

SOURCE = "24d5f4c112924cfe45681de3fcf3a92f62087e29524f15c74e14d83620891abd"
REFERENCE = ROOT / "build/tl-verify-image-1/reference/run/materiaal-normal-a69441f209"
ORDINARY = ROOT / "build/tl-verify-image-1/interactive/preview-run/materiaal-3d940fcfdd"


def main():
    if source_hash() != SOURCE:
        raise ValueError("Source drift")
    report = json.loads((REFERENCE / "result.json").read_text())
    reference = json.loads((REFERENCE / "inputs/manifest.json").read_text())
    ordinary = json.loads(
        (ROOT / "build/tl-verify-image-1/interactive/manifest.json").read_text()
    )
    preview.reference_binding(report, reference, ordinary, SOURCE, "pc-q35-10.0")
    cases = {}
    for name in ("source", "app_hash", "loader_hash"):
        changed = copy.deepcopy(ordinary)
        if name == "source":
            changed["source_sha256"] = "0" * 64
        elif name == "app_hash":
            changed["materiaal"]["apps"][0]["sha256"] = "0" * 64
        else:
            changed["loader"]["sha256"] = "0" * 64
        try:
            preview.reference_binding(report, reference, changed, SOURCE, "pc-q35-10.0")
        except ValueError as error:
            cases[name] = {"rejected": True, "reason": str(error)}
        else:
            raise ValueError(f"Mutation accepted: {name}")
    frame = (ORDINARY / "reopened.ppm").read_bytes()
    if not preview.matching_reference_frame(frame, frame):
        raise ValueError("Unmodified frame rejected")
    changed = bytearray(frame)
    changed[-1] ^= 1
    if preview.matching_reference_frame(bytes(changed), frame):
        raise ValueError("Changed pixel accepted")
    cases["one_pixel_bit"] = {"rejected": True, "reason": "Full frame comparison false"}
    disk = (ORDINARY / "volume.after.img").read_bytes()
    probe.disk_oracle(disk)
    try:
        probe.disk_oracle(bytes(len(disk)))
    except ValueError as error:
        cases["empty_disk"] = {"rejected": True, "reason": str(error)}
    else:
        raise ValueError("Empty disk accepted")
    if source_hash() != SOURCE:
        raise ValueError("Source changed during negatives")
    result = {
        "scope": "host integration negatives on copies of real runtime evidence; no new VM",
        "source_sha256": SOURCE,
        "passed": True,
        "unmodified_controls_passed": True,
        "mutations": cases,
    }
    Path(__file__).with_suffix(".json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()

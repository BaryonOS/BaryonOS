#!/usr/bin/env python3
"""Preserve a completed gate only after rechecking its raw guest evidence."""
from __future__ import annotations
import argparse
import json
from pathlib import Path
from common import ROOT, BUILD, save_json, sha256
from run import is_success


def capture(phase: str):
    suite_path = BUILD / f"{phase}-test-results.json"
    suite = json.loads(suite_path.read_text(encoding="utf-8"))
    if suite.get("status") != "completed" or suite.get("passed") is not True:
        raise ValueError("Only a completed, passed gate can be captured")
    destination = ROOT / "docs/verification/m2-m4" / phase
    runs = []
    for group in suite["results"]:
        if group.get("passed") is not True:
            raise ValueError(f"Failed group: {group['name']}")
        evidence = Path(group["evidence"])
        if not evidence.is_absolute():
            evidence = ROOT / evidence
        if evidence.is_dir():
            evidence.resolve().relative_to((BUILD / "run").resolve())
            result = json.loads((evidence / "result.json").read_text(encoding="utf-8"))
            if not is_success(result["raw_qemu_exit"], result["stdout"], result["timed_out"],
                              required_markers=result["required_markers"],
                              expected_exit=result["expected_exit"],
                              forbidden_markers=result["forbidden_markers"]):
                raise ValueError(f"Guest evidence failed: {evidence}")
            if (evidence / "serial.txt").read_bytes().decode("utf-8") != result["stdout"]:
                raise ValueError(f"Serial evidence changed: {evidence}")
            for name, expected in result["artifact_sha256"].items():
                artifact = evidence / name
                artifact.resolve().relative_to(evidence.resolve())
                # Older runs recorded the immutable package template's hash,
                # but retained only the firmware-mutated working VARS file.
                # Verify that original source explicitly; never hash working VARS.
                if name == "OVMF_VARS.template.fd" and not artifact.exists():
                    artifact = Path(result["firmware_source"][1])
                if sha256(artifact) != expected:
                    raise ValueError(f"Artifact changed: {artifact}")
            runs.append(result)
        elif evidence.is_file():
            evidence.resolve().relative_to(BUILD.resolve())
            destination.mkdir(parents=True, exist_ok=True)
            # Preserve textual host output; never turn a log into a test result.
            (destination / evidence.name).write_text(
                evidence.read_text(encoding="utf-8").rstrip() + "\n", encoding="utf-8")
        else:
            raise ValueError(f"Missing evidence: {evidence}")
    save_json(destination / "suite.json", suite)
    save_json(destination / "guest-runs.json", runs)
    print(f"Captured {phase}: {len(suite['results'])} groups, {len(runs)} verified guest runs")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("phase", choices=["m2", "m3", "m4"])
    capture(parser.parse_args().phase)

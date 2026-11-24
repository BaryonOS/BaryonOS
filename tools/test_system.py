#!/usr/bin/env python3
"""FR-I01 gate on one frozen source: normal, exact-image cold readonly recovery.

Keeps the joint normal build as the final artifact, never substitutes M1 for
the tested image. Legacy profiles are separate required regression commands.
"""
from __future__ import annotations

import hashlib
from pathlib import Path
import subprocess
import sys

from build import build
from common import ROOT, BUILD, environment, save_json, source_hash
from run_system import (FEATURE, RELAY, RELAY_EXACT, RELAY_FEATURE, RELAY_RED, evaluate, expected_markers,
                        run_system)


def inputs() -> dict[str, str]:
    # Capture tests reuse both independent network wire fixture families.
    files = sorted({*(ROOT / "tests").glob("test_system_*.py"),
                    *(ROOT / "tests").glob("test_net_*.py"),
                    *(ROOT / "tests").glob("test_tcp_*.py"),
                    ROOT / "sdk/examples/tcp-probe/test_peer.py"})
    return {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}


def relay_logs(summary: dict, path: Path) -> None:
    """B02-T4 (ADR-B02-KEY1-RELAY-20261123): three relay cases on fresh disks,
    the red manifest case, then each relay log against the other evaluators.
    Kept out of `stages`, whose exact [normal, recover] shape capture_system checks."""
    summary.update(relay_stages=[], relay_red=[], relay_wrong_case=[])
    serial = {}
    for case, feature in (*RELAY_FEATURE.items(), *RELAY_RED.items()):
        build(None, feature, feature)
        result, _ = run_system(case, None)
        record = {key: result[key] for key in ("case", "passed", "run_directory", "raw_qemu_exit",
                                               "before_sha256", "after_sha256", "duration_s")}
        record["relay"] = {key: value for key, value in result.get("evidence", {}).items()
                           if key.startswith(RELAY) or case in RELAY_RED}
        summary["relay_red" if case in RELAY_RED else "relay_stages"].append(record)
        save_json(path, summary)
        if not result["passed"]:
            raise RuntimeError(result.get("error", f"{case} failed"))
        serial[case] = (Path(result["run_directory"]) / "serial.txt").read_text(), result["raw_qemu_exit"]
    for actual, (text, raw) in ((case, serial[case]) for case in RELAY_FEATURE):
        for claimed in RELAY_FEATURE:
            if claimed == actual:
                continue
            try:
                evaluate(claimed, text, raw, False)
            except ValueError as error:
                summary["relay_wrong_case"].append(dict(log=actual, evaluator=claimed, error=str(error)))
            else:
                raise ValueError(f"{actual} log passed the {claimed} evaluator")
            # Relabelled: with the case marker and number rewritten, the counters alone must reject.
            relabelled = text.replace(expected_markers(actual)[-2], expected_markers(claimed)[-2]).replace(
                f"SYSTEM:{RELAY}CASE:{case_number(actual):020d}", f"SYSTEM:{RELAY}CASE:{case_number(claimed):020d}")
            try:
                evaluate(claimed, relabelled, raw, False)
            except ValueError as error:
                summary["relay_wrong_case"].append(dict(log=actual, evaluator=claimed, relabelled=True,
                                                        error=str(error)))
            else:
                raise ValueError(f"relabelled {actual} log passed the {claimed} evaluator")
    save_json(path, summary)


def case_number(case: str) -> int:
    return RELAY_EXACT[case][RELAY + "CASE"]


def main() -> int:
    frozen, test_inputs = source_hash(), inputs()
    summary = dict(schema_version=1, source_fingerprint_version=3, source_sha256=frozen,
                   test_inputs=test_inputs, status="running", passed=False, host=[], stages=[])
    path = BUILD / "system-gate.json"
    save_json(path, summary)
    try:
        commands = [
            [sys.executable, "-B", "-m", "unittest", "discover", "-s", "tests", "-p", "test_system_*.py"],
            ["cargo", "+1.100.0", "test", "--locked", "--manifest-path", "sdk/Cargo.toml", "-p", "nieuw-os-sdk-runtime"],
        ]
        for number, command in enumerate(commands):
            result = subprocess.run(command, cwd=ROOT, env=environment(), capture_output=True, text=True)
            logs = BUILD / "logs" / f"system-host-{number}"
            logs.parent.mkdir(parents=True, exist_ok=True)
            logs.with_suffix(".stdout.txt").write_text(result.stdout)
            logs.with_suffix(".stderr.txt").write_text(result.stderr)
            summary["host"].append(dict(command=command, exit=result.returncode,
                                        stdout=str(logs.with_suffix(".stdout.txt")),
                                        stderr=str(logs.with_suffix(".stderr.txt"))))
            save_json(path, summary)
            if result.returncode:
                raise RuntimeError(f"host check {number} failed; see {logs}")
        volume = None
        normal_hash = None
        for case in FEATURE:
            manifest = build(None, FEATURE[case], FEATURE[case])
            if case == "normal":
                normal_hash = manifest["kernel"]["sha256"]
            result, volume = run_system(case, volume)
            summary["stages"].append({key: result[key] for key in
                                      ("case", "passed", "run_directory", "raw_qemu_exit",
                                       "before_sha256", "after_sha256", "duration_s")})
            save_json(path, summary)
            if not result["passed"]:
                raise RuntimeError(result.get("error", f"{case} failed"))
        relay_logs(summary, path)
        final = build(None, FEATURE["normal"], FEATURE["normal"])
        if final["kernel"]["sha256"] != normal_hash:
            raise ValueError("final joint-system artifact differs from the actually tested normal kernel")
        if source_hash() != frozen or inputs() != test_inputs:
            raise ValueError("source or test inputs changed during gate")
        summary.update(status="completed", passed=True, final_kernel_sha256=normal_hash,
                       final_manifest=str(BUILD / "manifest.json"))
    except Exception as error:
        summary.update(status="failed", error=f"{type(error).__name__}: {error}")
    save_json(path, summary)
    print(f"FR-I01 gate passed={summary['passed']} stages={len(summary['stages'])} {path}")
    if "error" in summary:
        print(summary["error"])
    return 0 if summary["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())

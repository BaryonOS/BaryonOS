#!/usr/bin/env python3
"""Real kernel NV-counter commands across fully stopped QEMU/swtpm children.

This is a public disposable fixture, not a keyring rollback-anchor acceptance.
The host never executes TPM commands or supplies a fake guest counter.
"""
from __future__ import annotations

import json
from pathlib import Path
import re

from build import build
from common import BUILD, ROOT, save_json, sha256, source_hash
from run import run_guest
import run_tpm

CASES = ("provision", "read", "auth", "missing")


def markers(case):
    baseline = run_tpm.markers("normal")
    at = baseline.index("TPMCRB:PASS") + 1
    return baseline[:at] + (f"TPMNV:PASS:{case}",) + baseline[at:]


def evaluate(stdout, case):
    if case not in CASES:
        raise ValueError("Unknown NV fixture case")
    result = run_tpm.evaluate(stdout, "normal")
    lines = stdout.splitlines()
    if lines.count("TPMNV:PUBLIC:PASS") != 1:
        raise ValueError("Missing or duplicate NV public validation")
    if [line for line in lines if line.startswith("TPMNV:PASS:")] != [f"TPMNV:PASS:{case}"]:
        raise ValueError("Wrong or duplicate NV case marker")
    counts = {"BEFORE": 1, "AFTER": int(case != "read"),
              "EXPECTED_RC": int(case in ("auth", "missing"))}
    for name, count in counts.items():
        prefix = f"TPMNV:{name}:"
        values = [line[len(prefix):] for line in lines if line.startswith(prefix)]
        if len(values) != count or any(not re.fullmatch(r"[0-9]{20}", value) for value in values):
            raise ValueError(f"Invalid NV {name} evidence")
        if count:
            result[name] = int(values[0])
            if result[name] > 2**64 - 1:
                raise ValueError(f"NV {name} overflow")
    if case == "provision":
        if result["BEFORE"] == 2**64 - 1 or result["AFTER"] != result["BEFORE"] + 1:
            raise ValueError("No exact NV increment")
    elif case != "read" and result["AFTER"] != result["BEFORE"]:
        raise ValueError("Counter changed during a negative read")
    if case == "auth" and result["EXPECTED_RC"] not in (0x98e, 0x9a2):
        raise ValueError("Wrong auth rejection code")
    if case == "missing" and result["EXPECTED_RC"] != 0x18b:
        raise ValueError("Wrong missing-index rejection code")
    return result


def reconcile(stages):
    if [stage["case"] for stage in stages] != list(CASES):
        raise ValueError("Missing or reordered cold stages")
    final = stages[0]["observed"]["AFTER"]
    state = stages[0]["backend"]["state_directory"]
    children = set()
    for stage in stages:
        if not stage["guest"]["passed"] or stage["backend"]["exit_status"] not in (0, -15):
            raise ValueError("A guest or backend did not complete")
        if stage["backend"]["state_directory"] != state:
            raise ValueError("Cold stage used different TPM state")
        directory = stage["backend"]["directory"]
        if directory in children:
            raise ValueError("Cold stage reused a backend child directory")
        children.add(directory)
        if stage["case"] != "provision" and stage["observed"]["BEFORE"] != final:
            raise ValueError("Cold counter differs from the committed fixture")


def state_hashes(directory):
    return {path.relative_to(directory).as_posix(): sha256(path)
            for path in sorted(directory.rglob("*")) if path.is_file()}


def main():
    frozen = source_hash()
    report = {"schema_version": 1, "passed": False, "source_sha256": frozen,
              "source_fingerprint_version": 3, "stages": [],
              "scope": "public disposable NV fixture; no key lineage or B02 acceptance",
              "test_inputs": {"tools/tests/test_tpm_nv.py": sha256(ROOT / "tools/tests/test_tpm_nv.py")}}
    destination = BUILD / "tpm-nv-gate.json"
    save_json(destination, report)
    state = None
    try:
        for case in CASES:
            stage = {"case": case}
            report["stages"].append(stage)
            feature = f"test-tpm-nv-{case}"
            manifest = build(None, feature, feature)
            if manifest["source_sha256"] != frozen or source_hash() != frozen:
                raise ValueError("NV fixture source changed")
            stage["manifest"] = manifest
            with run_tpm.backend(stage, state_directory=state) as control:
                stage["guest"] = run_guest(
                    f"tpm-nv-{case}", tpm_socket=control, required_markers=markers(case),
                    forbidden_markers=("TPMCRB:FAIL:", "IPCWAIT:FAIL:", "PANIC", "M3:FAIL:"),
                    expected_kernel_sha256=manifest["kernel"]["sha256"],
                    expected_loader_sha256=manifest["loader"]["sha256"])
                if not stage["guest"]["passed"]:
                    raise ValueError(f"NV {case} guest failed; inspect retained run_directory")
                stage["observed"] = evaluate(stage["guest"]["stdout"], case)
            state = Path(stage["backend"]["state_directory"])
            stage["state_files_after_stop"] = state_hashes(state)
            if not stage["state_files_after_stop"]:
                raise ValueError("No persisted TPM state after full backend stop")
            save_json(destination, report)
        reconcile(report["stages"])
        if source_hash() != frozen:
            raise ValueError("NV fixture source changed during gate")
        report["passed"] = True
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
        save_json(destination, report)
        raise
    save_json(destination, report)
    print(json.dumps({"passed": True, "stages": len(report["stages"]),
                      "report": str(destination), "source_sha256": frozen}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

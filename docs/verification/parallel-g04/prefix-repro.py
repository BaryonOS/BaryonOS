#!/usr/bin/env python3
"""Reproduce G04-01 through real GNU disassembly; never execute the bytes."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile

root = Path(sys.argv[1]).resolve()
path = root / "experiments/quantum/native/build.py"
spec = importlib.util.spec_from_file_location("qsim_native_review", path)
build = importlib.util.module_from_spec(spec)
spec.loader.exec_module(build)
report = {
    "gate_source": str(path),
    "gate_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
    "scope": "Host negative-control reproduction; fixture instructions are never executed.",
    "objdump_version": subprocess.check_output(["objdump", "--version"], text=True).splitlines()[0],
    "cases": [],
}
with tempfile.TemporaryDirectory(prefix="nieuw-os-g04-") as temporary:
    for name, encoded in (("plain-fninit", "cd80dbe3"),
                          ("data16-fninit", "cd8066dbe3"),
                          ("repz-fninit", "cd80f3dbe3")):
        binary = Path(temporary) / (name + ".bin")
        binary.write_bytes(bytes.fromhex(encoded))
        command = ["objdump", "-D", "-b", "binary", "-m", "i386:x86-64",
                   "--no-show-raw-insn", "-M", "intel", str(binary)]
        result = subprocess.run(command, text=True, capture_output=True, check=True)
        case = {"name": name, "hex": encoded, "argv": command,
                "objdump_exit": result.returncode, "disassembly": result.stdout,
                "expected_gate_acceptance": False}
        try:
            case["gate_result"] = build.inspect_instructions(result.stdout)
            case["gate_accepted"] = True
        except ValueError as error:
            case["gate_accepted"] = False
            case["gate_error"] = str(error)
        report["cases"].append(case)
report["all_negative_controls_rejected"] = all(not case["gate_accepted"] for case in report["cases"])
print(json.dumps(report, indent=2) + "\n", end="")

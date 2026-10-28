#!/usr/bin/env python3
"""Check SDK selection and real embedded userspace artifact after full capture."""
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

root = Path(sys.argv[1]).resolve()
build = root / "build/parallel-final-sdk-m6c"
archive = root / "docs/verification/parallel-final-sdk-m6c"
sys.path.insert(0, str(root / "tools"))
from common import source_hash

source = "307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60"
commit = "3e41bbf93822f7044c6020259f4a752925e1966d"
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
read = lambda path: json.loads(path.read_text())
driver = read(build / "g05-driver.json")
capture = read(archive / "capture.json")
suite = read(archive / "suite.json")
runs = read(archive / "guest-runs.json")
user_path = build / "users/manifest-m6c.json"
user = read(user_path)
user_binary = (build / "users/m6c_probe.elf").read_bytes()
user_digest = hashlib.sha256(user_binary).hexdigest()
if (driver.get("passed") is not True or driver.get("source_unchanged") is not True
        or capture.get("passed") is not True or capture.get("source_sha256") != source
        or capture.get("groups") != 35 or capture.get("vm_invocations") != 31
        or suite.get("source_sha256") != source or user.get("sdk_runtime") is not True
        or user.get("binary_sha256") != user_digest or source_hash() != source
        or subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip() != commit):
    raise ValueError("G05 frozen-source, complete-capture or SDK-user-artifact binding failed")
module_checks = {name: {"expected": expected, "observed": sha(root / name)}
                 for name, expected in user["module_sha256"].items()}
if any(row["expected"] != row["observed"] for row in module_checks.values()):
    raise ValueError("A compiled SDK/bridge/module source changed")
rows = []
for run in runs:
    if run.get("case") is None:
        continue
    directory = Path(run["run_directory"])
    manifest_path = directory / "build-manifest.json"
    manifest = read(manifest_path)
    kernel = (directory / "esp/kernel.elf").read_bytes()
    row = {"label": run["label"], "case": run["case"], "run_directory": str(directory),
           "manifest_sha256": sha(manifest_path), "sdk_runtime": manifest.get("sdk_runtime"),
           "kernel_sha256": hashlib.sha256(kernel).hexdigest(),
           "actual_sdk_user_elf_embedded": user_binary in kernel,
           "source_matches": manifest.get("source_sha256") == run.get("source_sha256") == source,
           "raw_qemu_exit": run["raw_qemu_exit"], "disk_oracle_passed": run["disk_oracle"]["passed"]}
    if (row["sdk_runtime"] is not True or not row["actual_sdk_user_elf_embedded"]
            or not row["source_matches"] or row["disk_oracle_passed"] is not True
            or row["kernel_sha256"] != run["artifact_sha256"]["esp/kernel.elf"]):
        raise ValueError("A real M6c kernel did not bind the frozen SDK executable")
    rows.append(row)
if len(rows) != 30 or len(runs) != 31:
    raise ValueError("Expected 30 SDK M6c VMs plus one ordinary M1 VM")
recoveries = [group for group in suite["results"] if "recovery_checks" in group]
if len(recoveries) != 14 or any(not all(group["recovery_checks"].values()) for group in recoveries):
    raise ValueError("Missing exact-image no-write cold recovery links")
normal = [run for run in runs if run.get("case") == "normal"]
if len(normal) != 2:
    raise ValueError("Expected two independently seeded normal volumes")
report = {
    "schema_version": 1, "task": "G05", "passed": True,
    "checked_utc": datetime.now(timezone.utc).isoformat(), "source_commit": commit,
    "source_fingerprint_version": 3, "source_sha256": source,
    "scope": "Additional SDK source/artifact selection check after unchanged full M6c acceptance capture.",
    "capture_sha256": sha(archive / "capture.json"), "suite_sha256": sha(archive / "suite.json"),
    "guest_runs_sha256": sha(archive / "guest-runs.json"),
    "groups": 35, "real_vm_invocations": 31, "sdk_m6c_invocations": 30,
    "ordinary_m1_regressions": 1, "cold_recovery_pairs": 14,
    "raw_exit_counts": dict(Counter(str(run["raw_qemu_exit"]) for run in runs)),
    "case_counts": dict(Counter(run.get("case", "ordinary-m1") for run in runs)),
    "sdk_user_elf_sha256": user_digest, "sdk_user_manifest_sha256": sha(user_path),
    "compiled_module_source_checks": module_checks, "sdk_native_runs": rows,
    "normal_results": [{"label": run["label"], "oracle": run["disk_oracle"]} for run in normal],
    "recovery_checks": [{"name": group["name"], "checks": group["recovery_checks"]} for group in recoveries],
    "known_limits": ["This runs the existing synthetic M6c roles through the SDK runtime bridge.",
                     "Block I/O and deliberate malformed/negative ABI probes retain the original raw fixture path.",
                     "The separate typed B01 SDK catalog sample was compiled and host-tested earlier; this suite does not launch that sample.",
                     "No G1/Materiaal graphical editor, larger-document storage, physical disk durability, general app launcher or human audit claim."],
    "roadmap": {"delta": 0, "existing_storage_gates": "S03 and S07 retained by regression",
                "A01": "open", "U03": "open"},
}
(archive / "sdk-binding.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n")
shutil.copyfile(user_path, archive / "sdk-user-manifest.json")
shutil.copyfile(build / "g05-driver.json", archive / "driver.json")
print(json.dumps({key: report[key] for key in ("passed", "source_commit", "source_sha256", "groups",
      "real_vm_invocations", "sdk_m6c_invocations", "cold_recovery_pairs", "raw_exit_counts", "sdk_user_elf_sha256")}, indent=2))

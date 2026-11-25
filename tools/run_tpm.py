#!/usr/bin/env python3
"""Real kernel CRB MMIO over a disposable local swtpm; no keys or NV mutation."""
from __future__ import annotations

import argparse
from contextlib import contextmanager
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time
import uuid

from build import build
from common import BUILD, LOCAL, ROOT, environment, save_json, sha256, source_hash
import run_ipc_wait
from run import run_guest

FEATURES = {"normal": "native-tpm-crb", "absent": "test-tpm-crb-absent",
            "loader-probe": "native-tpm-crb", "loader-rng": "native-tpm-crb"}
# B03 REL-05 S6 step 1: firmware TCG2/RNG before EBS, then the normal kernel case.
LOADER = {case: "tpm-loader-probe-test" for case in FEATURES if case.startswith("loader-")}
# U25: the TEST profile has no firmware RNG; only `loader-rng` adds a virtio-rng device.
LOADER_RNG = {"loader-probe": "ABSENT", "loader-rng": "PASS"}
LOADER_MARKERS = ("LOADER-TPM:TCG2:PRESENT", "LOADER-TPM:PCR13:ZERO", "LOADER-TPM:PASS")
COMMON = ("M2:PASS", "TPMCRB:BOUNDS:PASS", "TPMCRB:MAPPING:PASS",
          "IPCWAIT:MANIFEST:PASS", "IPCWAIT:REVOKED_FULL_QUEUE:PASS")


def markers(case):
    final = ("TPMCRB:ABSENT:PASS",) if case == "absent" else ("TPMCRB:RELEASED:PASS", "TPMCRB:PASS")
    # run.is_success matches in serial order: the loader prints before EBS.
    return (LOADER_MARKERS if case in LOADER else ()) + COMMON + final + run_ipc_wait.MARKERS[3:]


def evaluate(stdout, case):
    if case not in FEATURES:
        raise ValueError("Unknown native TPM case")
    result = run_ipc_wait.evaluate(stdout)
    for marker in markers(case):
        if stdout.splitlines().count(marker) != 1:
            raise ValueError(f"Expected exactly one {marker}")
    if case in LOADER:
        result.update(loader_observations(stdout))
        if result["LOADER_RNG"] != LOADER_RNG[case]:
            raise ValueError(f"Loader RNG must be {LOADER_RNG[case]} in {case}")
        case = "normal"
    elif "LOADER-TPM:" in stdout:
        raise ValueError("Loader TPM probe output in a kernel-only case")
    for name in ("WRITES", "BOUNDS_REJECTED", "QUARANTINED", "FAMILY"):
        values = re.findall(rf"^TPMCRB:{name}:([0-9]{{20}})$", stdout, re.M)
        expected = 0 if case == "absent" and name == "FAMILY" else 1
        if len(values) != expected:
            raise ValueError(f"Invalid native TPM {name} observation")
        if values:
            result[name] = int(values[0])
    if result["BOUNDS_REJECTED"] != 7:
        raise ValueError("Adapter bounds were not fully exercised")
    if case == "normal":
        if result["FAMILY"] != 0x322E3000 or result["WRITES"] == 0 or result["QUARANTINED"] != 0:
            raise ValueError("No completed TPM2 property query and release")
        if "TPMCRB:ABSENT:PASS" in stdout:
            raise ValueError("Absent TPM cannot approve the normal case")
    elif result["WRITES"] != 0 or result["QUARANTINED"] != 1 or "TPMCRB:PASS" in stdout:
        raise ValueError("Missing TPM must be quarantined without any hardware write")
    return result


def loader_observations(stdout):
    """Facts the probe loader printed; RNG presence is recorded, not required (U25)."""
    found = {}
    for name, pattern in (("BANKS", "[0-9a-f]{16}"), ("PERMANENT", "[0-9a-f]{16}"),
                          ("RNG", "PASS|ABSENT")):
        values = re.findall(rf"^LOADER-TPM:{name}:({pattern})$", stdout, re.M)
        if len(values) != 1 or len(re.findall(rf"^LOADER-TPM:{name}:", stdout, re.M)) != 1:
            raise ValueError(f"Invalid loader TPM {name} observation")
        found[f"LOADER_{name}"] = values[0] if name == "RNG" else int(values[0], 16)
    if not found["LOADER_BANKS"] & 0x2:
        raise ValueError("Loader TPM probe needs an active SHA-256 bank")
    return found


@contextmanager
def backend(report):
    prefix = LOCAL / "swtpm-native"
    env = environment()
    binary = prefix / "usr/bin/swtpm"
    if not binary.is_file():
        found = shutil.which("swtpm", path=env["PATH"])
        if not found:
            raise RuntimeError("swtpm missing: install the test tool or restore the documented local prefix")
        binary = Path(found).resolve(strict=True)
    libraries = [prefix / "usr/lib/x86_64-linux-gnu/swtpm",
                 prefix / "usr/lib/x86_64-linux-gnu", prefix / "usr/lib"]
    env["LD_LIBRARY_PATH"] = os.pathsep.join(
        [str(p) for p in libraries if p.is_dir()]
        + ([env["LD_LIBRARY_PATH"]] if env.get("LD_LIBRARY_PATH") else [])
    )
    if not BUILD.resolve().is_relative_to(ROOT.resolve()):
        raise ValueError("TPM state must be generated inside the checkout")
    directory = BUILD / "tpm" / uuid.uuid4().hex[:10]
    directory.mkdir(parents=True)
    state = directory / "state"
    state.mkdir(mode=0o700)
    control = directory / "ctrl.sock"
    relative = directory.relative_to(ROOT).as_posix()
    # Each child owns a new private state directory; do not require the optional
    # `lock` flag that the CI-distributed swtpm 0.7.3 does not support.
    command = [str(binary), "socket", "--tpm2", "--tpmstate", f"dir={relative}/state",
               "--ctrl", f"type=unixio,path={relative}/ctrl.sock,mode=0600",
               "--flags", "startup-clear"]
    if len(os.fsencode(f"{relative}/ctrl.sock")) > 107:
        raise ValueError("Use a shorter NOS_BUILD_DIR for the bounded Unix socket name")
    report["backend"] = {
        "command": command, "binary_sha256": sha256(binary),
        "version": subprocess.check_output([str(binary), "--version"], env=env, text=True).strip(),
        "directory": str(directory), "log": str(directory / "backend.log"),
    }
    with (directory / "backend.log").open("wb") as stream:
        process = subprocess.Popen(command, cwd=ROOT, env=env, stdout=stream, stderr=subprocess.STDOUT)
        try:
            deadline = time.monotonic() + 5
            while not control.is_socket():
                if process.poll() is not None:
                    raise RuntimeError(f"swtpm exited during startup; see {directory / 'backend.log'}")
                if time.monotonic() >= deadline:
                    raise RuntimeError("swtpm control socket startup timed out")
                time.sleep(0.01)
            yield control
        finally:
            if process.poll() is None:
                process.terminate()
            try:
                status = process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                status = process.wait(timeout=5)
            report["backend"]["exit_status"] = status
            report["backend"]["log_sha256"] = sha256(directory / "backend.log")
            if status not in (0, -15):
                raise RuntimeError(f"swtpm failed with exit {status}; see {directory / 'backend.log'}")


def run(case):
    if case not in FEATURES:
        raise ValueError("Unknown native TPM case")
    frozen = source_hash()
    destination = BUILD / f"tpm-{case}-report.json"
    report = {"schema_version": 1, "passed": False, "case": case,
              "source_fingerprint_version": 3, "source_sha256": frozen,
              "scope": "fixed QEMU CRB MMIO, property query, release, isolation and missing-device refusal; loader-probe adds read-only firmware TCG2 SubmitCommand and EFI_RNG_PROTOCOL presence before EBS, loader-rng the same with a TEST virtio-rng device; no product keys or antirollback"}
    save_json(destination, report)
    try:
        manifest = build(LOADER.get(case), FEATURES[case], LOADER.get(case, FEATURES[case]))
        if (manifest["kernel_feature"] != FEATURES[case]
                or manifest["loader_feature"] != LOADER.get(case)
                or manifest["source_sha256"] != frozen or source_hash() != frozen):
            raise ValueError("TPM build source/profile changed")

        def guest(control=None):
            report["guest"] = run_guest(
                f"tpm-{case}", required_markers=markers(case), tpm_socket=control,
                forbidden_markers=("TPMCRB:FAIL:", "IPCWAIT:FAIL:", "PANIC", "M3:FAIL:",
                                   "LOADER-TPM:FAIL:"),
                expected_kernel_sha256=manifest["kernel"]["sha256"],
                expected_loader_sha256=manifest["loader"]["sha256"],
                virtio_rng=case == "loader-rng",
            )
            if not report["guest"]["passed"]:
                raise ValueError("Native TPM guest failed; inspect retained serial/stderr")
            report["observed"] = evaluate(report["guest"]["stdout"], case)

        if case != "absent":
            with backend(report) as control:
                guest(control)
        else:
            guest()
        if source_hash() != frozen:
            raise ValueError("Source changed during native TPM run")
        report["passed"] = True
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
        save_json(destination, report)
        raise
    save_json(destination, report)
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=FEATURES, default="normal")
    args = parser.parse_args()
    print(json.dumps(run(args.case), sort_keys=True))

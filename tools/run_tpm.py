#!/usr/bin/env python3
"""Real kernel CRB MMIO over a disposable local swtpm. The loader-hauth case
writes TEST hierarchy auth and NV (M1-HAUTH) into that disposable state only;
loader-hauth-no-rng checks that the loader halts after the CLOSED extend, and the
pre-seeded cases (tools/tpm_seed.py) that it halts or repairs on a prepared state."""
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
import tpm_seed

# Pre-seeded M1-HAUTH cases: (boot before which tools/tpm_seed.py prepares the state, seed).
SEED = {"loader-hauth-squat": (0, "squat"), "loader-hauth-foreign-owner": (0, "foreign-owner"),
        "loader-hauth-unbound": (1, "unbound"), "loader-hauth-cut": (0, "cut")}
FEATURES = {"normal": "native-tpm-crb", "absent": "test-tpm-crb-absent",
            "loader-probe": "native-tpm-crb", "loader-rng": "native-tpm-crb",
            "loader-hauth": "native-tpm-crb", "loader-hauth-no-rng": "native-tpm-crb",
            **dict.fromkeys(SEED, "native-tpm-crb")}
# B03 REL-05 S6 step 1: firmware TCG2/RNG before EBS, then the normal kernel case.
# loader-hauth (M1-HAUTH BOUND, TEST): the probe, then provisioning on boot 0 and
# verification on boot 1 of the same swtpm state.
LOADER = {"loader-probe": "tpm-loader-probe-test", "loader-rng": "tpm-loader-probe-test",
          **dict.fromkeys(("loader-hauth", "loader-hauth-no-rng", *SEED), "tpm-hauth-provision-test")}
BOOTS = {"loader-hauth": 2, "loader-hauth-unbound": 2, "loader-hauth-cut": 2}
# Cases whose non-halt boots must provision or verify exactly like loader-hauth.
PROVISIONED = ("loader-hauth", "loader-hauth-unbound", "loader-hauth-cut")
PROVISION, EXISTING = "LOADER-HAUTH:MODE:PROVISION", "LOADER-HAUTH:MODE:EXISTING"
CLOSED = "LOADER-HAUTH:HALT:PCR13:CLOSED"
# Halt boots, (case, boot) -> (TPMA_PERMANENT ownerAuthSet | lockoutAuthSet, the loader lines
# after BEGIN and PERMANENT): the loader extends CLOSED, then stops before the kernel.
HALT = {("loader-hauth-no-rng", 0): (0, (PROVISION, CLOSED, "LOADER-HAUTH:FAIL:RNG_ABSENT:0000000000000000")),
        # C21 platform squat: the owner undefine of a PLATFORMCREATE HAUTH is refused.
        ("loader-hauth-squat", 0): (0, (PROVISION, CLOSED, "LOADER-HAUTH:FAIL:TPM_RC:0000000000000149")),
        # Foreign ownerAuth, nothing provisioned: NV_ReadPublic(HAUTH) is TPM_RC_HANDLE on handle 1.
        ("loader-hauth-foreign-owner", 0): (1, (EXISTING, CLOSED, "LOADER-HAUTH:FAIL:TPM_RC:000000000000018b")),
        ("loader-hauth-unbound", 1): (5, (EXISTING, CLOSED, "LOADER-HAUTH:FAIL:UNBOUND:0000000000000000"))}
HALT_EXIT = 35  # loader serial::exit(0x11), as tools/test.py pins for M1:FAIL
# U25: the TEST profile has no firmware RNG; `loader-rng` and the HAUTH cases add a virtio-rng device.
LOADER_RNG = {"loader-probe": "ABSENT", "loader-rng": "PASS", "loader-hauth": "PASS",
              "loader-hauth-no-rng": "ABSENT", **dict.fromkeys(SEED, "PASS")}
LOADER_MARKERS = ("LOADER-TPM:TCG2:PRESENT", "LOADER-TPM:PCR13:ZERO", "LOADER-TPM:PASS")
HAUTH_MODE = (("LOADER-HAUTH:MODE:PROVISION", "LOADER-HAUTH:LOCKOUT:SET"),
              ("LOADER-HAUTH:MODE:EXISTING", "LOADER-HAUTH:LOCKOUT:KEPT"))


def hauth_markers(boot):
    mode, lockout = HAUTH_MODE[boot]
    return ("LOADER-HAUTH:BEGIN", mode, "LOADER-HAUTH:BOUND", lockout, "LOADER-HAUTH:PCR13:CLOSED",
            "LOADER-HAUTH:READ:REFUSED", "LOADER-HAUTH:OWNER:REFUSED",
            "LOADER-HAUTH:RESIDUE:NONE", "LOADER-HAUTH:PASS")
COMMON = ("M2:PASS", "TPMCRB:BOUNDS:PASS", "TPMCRB:MAPPING:PASS",
          "IPCWAIT:MANIFEST:PASS", "IPCWAIT:REVOKED_FULL_QUEUE:PASS")


def markers(case, boot=0):
    if (case, boot) in HALT:
        return LOADER_MARKERS + ("LOADER-HAUTH:BEGIN",) + HALT[case, boot][1]
    final = ("TPMCRB:ABSENT:PASS",) if case == "absent" else ("TPMCRB:RELEASED:PASS", "TPMCRB:PASS")
    hauth = hauth_markers(boot) if case in PROVISIONED else ()
    # run.is_success matches in serial order: the loader prints before EBS.
    return ((LOADER_MARKERS if case in LOADER else ()) + hauth + COMMON + final
            + run_ipc_wait.MARKERS[3:])


def evaluate(stdout, case, boot=0):
    if case not in FEATURES or not 0 <= boot < BOOTS.get(case, 1):
        raise ValueError("Unknown native TPM case")
    if (case, boot) in HALT:
        return halted(stdout, case, boot)
    result = run_ipc_wait.evaluate(stdout)
    for marker in markers(case, boot):
        if stdout.splitlines().count(marker) != 1:
            raise ValueError(f"Expected exactly one {marker}")
    if case in PROVISIONED:
        if set(HAUTH_MODE[1 - boot]) & set(stdout.splitlines()):
            raise ValueError("HAUTH boot mode does not match the swtpm state")
        permanent = re.findall(r"^LOADER-HAUTH:PERMANENT:([0-9a-f]{16})$", stdout, re.M)
        if len(permanent) != 1 or (int(permanent[0], 16) & 0x5) != (0 if boot == 0 else 0x5):
            raise ValueError("Invalid HAUTH ownerAuth/lockoutAuth state")
        result["HAUTH_PERMANENT"] = int(permanent[0], 16)
        # The refused owner command must leave the DA lockout counter (0x20E) unchanged.
        da = re.findall(r"^LOADER-HAUTH:DA:([0-9a-f]{16}):([0-9a-f]{16})$", stdout, re.M)
        if (len(da) != 1 or len(re.findall(r"^LOADER-HAUTH:DA:", stdout, re.M)) != 1
                or da[0][0] != da[0][1] or "LOADER-HAUTH:HALT:" in stdout):
            raise ValueError("Invalid HAUTH lockout-counter observation")
        result["HAUTH_DA_COUNTER"] = int(da[0][0], 16)
    elif "LOADER-HAUTH:" in stdout:
        raise ValueError("HAUTH loader output outside the loader-hauth case")
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


def halted(stdout, case, boot):
    lines = stdout.splitlines()
    permanent = re.findall(r"^LOADER-HAUTH:PERMANENT:([0-9a-f]{16})$", stdout, re.M)
    bits, steps = HALT[case, boot]
    expected = ["LOADER-HAUTH:BEGIN", *(f"LOADER-HAUTH:PERMANENT:{value}" for value in permanent[:1]),
                *steps]
    if (len(permanent) != 1 or int(permanent[0], 16) & 0x5 != bits
            or [line for line in lines if line.startswith("LOADER-HAUTH:")] != expected):
        raise ValueError("The loader must halt at exactly the expected HAUTH step")
    if any(line.startswith(("KERNEL:", "M1:PASS", "M2:", "TPMCRB:", "IPCWAIT:")) for line in lines):
        raise ValueError("The kernel ran after a loader halt")
    result = loader_observations(stdout)
    if result["LOADER_RNG"] != LOADER_RNG[case]:
        raise ValueError(f"Loader RNG must be {LOADER_RNG[case]} in {case}")
    result["HAUTH_PERMANENT"] = int(permanent[0], 16)
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


def swtpm():
    """The swtpm binary (documented local prefix, else PATH) and its environment."""
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
    return binary, env


def fresh_state():
    if not BUILD.resolve().is_relative_to(ROOT.resolve()):
        raise ValueError("TPM state must be generated inside the checkout")
    directory = BUILD / "tpm" / uuid.uuid4().hex[:10]
    directory.mkdir(parents=True)
    (directory / "state").mkdir(mode=0o700)
    return directory


@contextmanager
def backend(report, directory=None):
    """A swtpm on a fresh private state, or on `directory` from an earlier boot."""
    binary, env = swtpm()
    if not BUILD.resolve().is_relative_to(ROOT.resolve()):
        raise ValueError("TPM state must be generated inside the checkout")
    if directory is None:
        directory = fresh_state()
    elif not (directory / "state").is_dir() or not directory.resolve().is_relative_to(BUILD.resolve()):
        raise ValueError("Reused TPM state must be an existing state under the build directory")
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
    with (directory / "backend.log").open("ab") as stream:
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
              "scope": "fixed QEMU CRB MMIO, property query, release, isolation and missing-device refusal; loader-probe adds read-only firmware TCG2 SubmitCommand and EFI_RNG_PROTOCOL presence before EBS, loader-rng the same with a TEST virtio-rng device; loader-hauth adds TEST M1-HAUTH BOUND provisioning (HAUTH, LREC, ownerAuth, lockoutAuth) and the PCR 13 CLOSED extend in a disposable swtpm state, then a second boot on that state; loader-hauth-no-rng halts after that extend without a firmware RNG and never reaches the kernel; the pre-seeded cases prepare the disposable state with raw swtpm commands first (platform HAUTH squat, foreign ownerAuth, flipped LREC binding after a provisioning boot, power-cut leftovers) and record public state before and after; no product keys, genesis, ACCEPTED or antirollback"}
    save_json(destination, report)
    try:
        manifest = build(LOADER.get(case), FEATURES[case], LOADER.get(case, FEATURES[case]))
        if (manifest["kernel_feature"] != FEATURES[case]
                or manifest["loader_feature"] != LOADER.get(case)
                or manifest["source_sha256"] != frozen or source_hash() != frozen):
            raise ValueError("TPM build source/profile changed")

        def guest(control=None, boot=0):
            label = f"tpm-{case}" if boot == 0 else f"tpm-{case}-boot{boot}"
            report["guest"] = run_guest(
                label, required_markers=markers(case, boot), tpm_socket=control,
                forbidden_markers=("TPMCRB:FAIL:", "IPCWAIT:FAIL:", "PANIC", "M3:FAIL:", "LOADER-TPM:FAIL:")
                + (() if (case, boot) in HALT else ("LOADER-HAUTH:FAIL:",)),
                expected_exit=HALT_EXIT if (case, boot) in HALT else 33,
                expected_kernel_sha256=manifest["kernel"]["sha256"],
                expected_loader_sha256=manifest["loader"]["sha256"],
                virtio_rng=LOADER_RNG.get(case) == "PASS",
            )
            if not report["guest"]["passed"]:
                raise ValueError("Native TPM guest failed; inspect retained serial/stderr")
            report["observed"] = evaluate(report["guest"]["stdout"], case, boot)

        if case == "absent":
            guest()
        elif case in BOOTS or case in SEED:
            directory, report["boots"] = None, []
            for boot in range(BOOTS.get(case, 1)):
                entry = {}
                if SEED.get(case, (None,))[0] == boot:
                    directory = directory or fresh_state()
                    entry["seeded"] = tpm_seed.seed(*swtpm(), directory, SEED[case][1])
                with backend(report, directory) as control:
                    guest(control, boot)
                directory = control.parent
                if case in SEED:
                    # Public state after the boot; a halt must leave the seeded state as it was.
                    entry["after"] = tpm_seed.snapshot(*swtpm(), directory)
                    if (case, boot) in HALT and entry["after"] != entry["seeded"]:
                        raise ValueError("The loader changed the seeded TPM state before halting")
                report["boots"].append(entry | {key: report.pop(key) for key in ("backend", "guest", "observed")})
        else:
            with backend(report) as control:
                guest(control)
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

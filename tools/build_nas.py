"""Bind the existing frozen NAS leaf builder to an opt-in kernel manifest."""
from __future__ import annotations

import sys

from build_users import inspect_user
from common import ROOT, BUILD, TARGET, run_logged, sha256
from run_fpu import load_json, private_path


def build_users():
    output = private_path(BUILD / "users/nas")
    target = private_path(TARGET / "nas-native")
    for name in ("nas-probe.elf", "nas-volume.elf", "build-report.json"):
        private_path(output / name)
    for name in ("nas-probe", "nas-volume"):
        private_path(target / name)
        for suffix in ("stdout.txt", "stderr.txt", "json"):
            private_path(BUILD / f"logs/build-{name}.{suffix}")
    for suffix in ("stdout.txt", "stderr.txt", "json"):
        private_path(BUILD / f"logs/build-nas-users.{suffix}")
    run_logged(
        [sys.executable, "-B", str(ROOT / "services/nas/native-probe/build.py"),
         "--target-dir", str(target), "--output-dir", str(output)],
        "build-nas-users",
    )
    report = load_json(output / "build-report.json")
    if (report.get("gate") != "crosscompile-and-elf-inspection-only"
            or report.get("native_execution") is not False):
        raise ValueError("NAS leaf report must not claim native execution")
    for name in ("nas-probe", "nas-volume"):
        binary = private_path(output / f"{name}.elf", regular=True)
        recorded = report["binaries"][name]
        if (recorded["sha256"] != sha256(binary)
                or recorded["bytes"] != binary.stat().st_size
                or recorded["elf"] != inspect_user(binary)):
            raise ValueError(f"NAS report differs from actual ELF: {name}")
    probe = str((output / "nas-probe.elf").resolve())
    volume = str((output / "nas-volume.elf").resolve())
    # Existing slot bindings are reused; the opt-in module supplies only the
    # NAS role policy, not the default keyring or system-service policy.
    return {
        "NOS_SYSTEM_INIT_ELF": probe,
        "NOS_SYSTEM_APP_ELF": probe,
        "NOS_KEYRING_SERVICE_ELF": probe,
        "NOS_VOLUME_SERVICE_ELF": volume,
    }, report

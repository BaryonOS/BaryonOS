#!/usr/bin/env python3
"""K06 step 1: verify native AP bring-up through the own-kernel native-smp profiles."""
from __future__ import annotations
import argparse
import json
import re
import struct
from common import BUILD, save_json, sha256, source_hash, without_acpi_diagnostics
from run import run_guest, is_success

MAX_CPUS = 8  # kernel/src/m2/smp.rs MAX_CPUS
BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS", "KS-03:PASS")
END = ("M2:PASS", "M1:PASS")
FORBIDDEN = ("FAIL", "PANIC")
# q35 has MCFG but no DMAR; a MADT failure must leave exactly this inventory.
ACPI_EXPECTED = "ACPI:UNAVAILABLE:NO_DMAR"

# case -> (kernel profile, vCPUs, TCG threads, expected outcome)
CASES = {
    "one": ("native-smp", 1, "single", "online"),
    "two-single": ("native-smp", 2, "single", "online"),
    "four": ("native-smp", 4, "multi", "online"),
    "eight": ("native-smp", 8, "multi", "online"),
    "nine-capacity": ("native-smp", 9, "multi", "online"),
    "duplicate-madt": ("native-smp", 4, "multi", "invalid-madt"),
    "ap-stall": ("test-smp-ap-stall", 4, "multi", "timeout"),
    "ap-mismatch": ("test-smp-ap-mismatch", 4, "multi", "mismatch"),
}


def duplicate_madt() -> bytes:
    """A checksummed second MADT (one enabled xAPIC); firmware appends it to the XSDT."""
    body = struct.pack("<II", 0xFEE00000, 1) + bytes([0, 8, 0, 0]) + struct.pack("<I", 1)
    table = bytearray(b"APIC" + struct.pack("<I", 36 + len(body)) + bytes([3, 0])
                      + b"NOSOS " + b"DUPMADT " + struct.pack("<I", 1) + b"NOS "
                      + struct.pack("<I", 1) + body)
    table[9] = (-sum(table)) & 0xFF
    if sum(table) % 256 != 0:
        raise AssertionError("fixture checksum")
    return bytes(table)


def evaluate(case: str, stdout: str, raw_exit: int | None, timed_out: bool) -> dict:
    """Independent serial oracle: the exact SMP sequence between KS-03 and M2:PASS."""
    profile, cpus, _, outcome = CASES[case]
    if not is_success(raw_exit, stdout, timed_out, required_markers=BOOT + END,
                      forbidden_markers=FORBIDDEN):
        raise ValueError("Native SMP boot/exit assertions failed")
    acpi = [line for line in stdout.splitlines() if line.startswith("ACPI:")]
    if acpi != [ACPI_EXPECTED]:
        raise ValueError("ACPI inventory changed; MADT handling must be independent")
    lines = without_acpi_diagnostics(stdout).splitlines()
    if lines.count(BOOT[0]) != 1:
        raise ValueError("Missing or repeated EBS handoff")
    start = lines.index(BOOT[0])
    if any(re.search(r"(?:BOOT:|BOOTINFO:|KERNEL:|KS-|M[0-9]+:|SMP:)", line)
           for line in lines[:start]):
        raise ValueError("Unexpected marker before EBS")
    observed = lines[start:]
    if tuple(observed[:len(BOOT)]) != BOOT or tuple(observed[-len(END):]) != END:
        raise ValueError("Unexpected boot prefix or terminal markers")
    smp = observed[len(BOOT):-len(END)]
    if outcome == "invalid-madt":
        if smp != ["SMP:UNAVAILABLE:INVALID_MADT"]:
            raise ValueError("Duplicate MADT was not rejected exactly")
        return {"outcome": outcome, "profile": profile, "online": 1, "ap_ids": []}
    if not smp or smp[0] != f"SMP:MADT:CPUS={cpus}:DISABLED=0:HOTPLUG=0":
        raise ValueError("MADT processor inventory does not match the VM")
    online = min(cpus, MAX_CPUS) if outcome == "online" else 1
    body = smp[1:]
    ap_lines = [line for line in body if line.startswith("SMP:AP:")]
    ids = []
    for line in ap_lines:
        match = re.fullmatch(r"SMP:AP:([0-9]+):(ONLINE|TIMEOUT|MISMATCH)", line)
        if match is None:
            raise ValueError("Malformed AP status")
        ids.append(int(match[1]))
    if len(set(ids)) != len(ids):
        raise ValueError("An AP was reported twice")
    if outcome == "online":
        if len(ids) != online - 1 or ap_lines != [f"SMP:AP:{apic}:ONLINE" for apic in ids]:
            raise ValueError("Not every expected AP came online")
        tail = [f"SMP:SKIPPED:CAPACITY={cpus - MAX_CPUS}"] if cpus > MAX_CPUS else []
    else:
        status = "TIMEOUT" if outcome == "timeout" else "MISMATCH"
        if len(ids) != 1 or ap_lines != [f"SMP:AP:{ids[0]}:{status}"]:
            raise ValueError("The faulting AP was not rejected first and only")
        tail = ["SMP:BRINGUP:STOPPED"]
    expected = ap_lines + tail + [f"SMP:ONLINE:{online}/{cpus}", f"SMP:SYNC:PASS:CPUS={online}"]
    if body != expected:
        raise ValueError("Unexpected SMP sequence or duplicate output")
    return {"outcome": outcome, "profile": profile, "online": online, "ap_ids": ids}


def run(cases: tuple[str, ...]) -> dict:
    destination = BUILD / "smp-report.json"
    destination.unlink(missing_ok=True)
    frozen = source_hash()
    manifest_path = BUILD / "manifest.json"
    manifest_hash = sha256(manifest_path)
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    profiles = {CASES[case][0] for case in cases}
    if (len(profiles) != 1
            or manifest.get("kernel_feature") not in profiles
            or manifest.get("loader_feature") is not None
            or manifest.get("source_fingerprint_version") != 3
            or manifest.get("source_sha256") != frozen):
        raise ValueError("Build the current kernel profile for exactly these cases first")
    report = {"schema_version": 1, "passed": False, "source_sha256": frozen,
              "source_fingerprint_version": 3, "manifest_sha256": manifest_hash,
              "kernel_feature": manifest["kernel_feature"],
              "scope": ("IF0 M2 AP bring-up and bounded sync self-test in QEMU/TCG; "
                        "no SMP scheduler, IPI/TLB shootdown, per-CPU TSS or hardware evidence"),
              "cases": []}
    save_json(destination, report)
    for case in cases:
        _, cpus, threads, _ = CASES[case]
        if source_hash() != frozen or sha256(manifest_path) != manifest_hash:
            raise ValueError("SMP source or manifest changed before a run")
        tables = []
        if case == "duplicate-madt":
            fixture = BUILD / "acpi" / "duplicate-madt.aml"
            fixture.parent.mkdir(parents=True, exist_ok=True)
            fixture.write_bytes(duplicate_madt())
            tables = [fixture]
        guest = run_guest(
            "smp-" + case, timeout=60, required_markers=BOOT + END,
            forbidden_markers=FORBIDDEN, cpus=cpus, tcg_threads=threads,
            acpi_tables=tables,
            expected_kernel_sha256=manifest["kernel"]["sha256"],
            expected_loader_sha256=manifest["loader"]["sha256"],
        )
        item = {"case": case, "cpus": cpus, "tcg_threads": threads, "guest": guest, "passed": False}
        if tables:
            item["acpi_table_sha256"] = sha256(tables[0])
        report["cases"].append(item)
        save_json(destination, report)
        item["observed"] = evaluate(case, guest["stdout"], guest["raw_qemu_exit"],
                                    guest["timed_out"])
        if (guest.get("passed") is not True
                or guest.get("artifact_binding_passed") is not True
                or guest.get("source_sha256") != frozen
                or source_hash() != frozen or sha256(manifest_path) != manifest_hash):
            raise ValueError("SMP source, artifacts or manifest changed during a run")
        item["passed"] = True
        save_json(destination, report)
    report["passed"] = bool(cases) and all(item["passed"] for item in report["cases"])
    save_json(destination, report)
    return report


def profile_cases(profile: str) -> tuple[str, ...]:
    return tuple(case for case, spec in CASES.items() if spec[0] == profile)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=tuple(CASES))
    parser.add_argument("--profile", choices=sorted({spec[0] for spec in CASES.values()}))
    args = parser.parse_args()
    if (args.case is None) == (args.profile is None):
        parser.error("choose exactly one of --case or --profile")
    selected = (args.case,) if args.case else profile_cases(args.profile)
    raise SystemExit(0 if run(selected)["passed"] else 1)

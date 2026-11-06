#!/usr/bin/env python3
"""K06: verify native AP bring-up, per-AP IST and bounded IPIs (native-smp profiles)."""
from __future__ import annotations
import argparse
import json
import re
import struct
from common import BUILD, save_json, sha256, source_hash, without_acpi_diagnostics
from run import run_guest, is_success

MAX_CPUS = 8  # kernel/src/m2/smp.rs MAX_CPUS
IPI_ROUNDS = 3  # kernel/src/m2/smp.rs IPI_ROUNDS
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
    "ap-ipi-deaf": ("test-smp-ap-ipi-deaf", 4, "multi", "quarantine"),
    "ap-nmi-ist": ("test-smp-ap-nmi", 4, "multi", "nmi-ist"),
    "ap-double-fault-ist": ("test-smp-ap-double-fault", 4, "multi", "df-ist"),
    "ap-shootdown-deaf": ("test-smp-ap-shootdown-deaf", 4, "multi", "shootdown-quarantine"),
    "shootdown-skipped-control": ("test-smp-shootdown-skipped", 4, "multi", "stale-control"),
    "ap-timer-dead": ("test-smp-ap-timer-dead", 4, "multi", "timer-quarantine"),
    "lockdep-order": ("test-smp-lockdep-order", 2, "multi", "lockdep-order"),
    "spin-irq-off-control": ("test-smp-spin-irq-off", 4, "multi", "spin-quarantine"),
    "held-across-switch": ("test-smp-held-across-switch", 2, "multi", "held-across-switch"),
    "double-run": ("test-smp-double-run", 4, "multi", "double-run"),
    "gs-user": ("test-smp-gs-user", 4, "multi", "gs-user"),
    "gs-iret-fault": ("test-smp-gs-iret-fault", 4, "multi", "gs-iret-fault"),
    "gs-nmi-user": ("test-smp-gs-nmi-user", 4, "multi", "gs-nmi-user"),
    "shared-scratch": ("test-smp-shared-scratch", 4, "multi", "shared-scratch"),
    "lock-in-handler": ("test-smp-lock-in-handler", 2, "multi", "lock-in-handler"),
    "lockpair-order": ("test-smp-lockpair-order", 2, "multi", "lockpair-order"),
    "lock-stuck": ("test-smp-lock-stuck", 2, "multi", "lock-stuck"),
    "serial-unlocked-control": ("test-smp-serial-unlocked", 4, "multi", "serial-torn"),
}
QUARANTINE_OUTCOMES = ("invalid-madt", "timeout", "mismatch", "quarantine", "shootdown-quarantine",
                       "timer-quarantine", "spin-quarantine")
TERMINAL = {"nmi-ist": "SMP:FAULT:AP_NMI_IST:PASS", "df-ist": "SMP:FAULT:AP_DF_IST:PASS",
            "lockdep-order": "SMP:LOCKDEP:ORDER:30->10",
            "held-across-switch": "SMP:LOCKDEP:HELD_ACROSS_SWITCH",
            "double-run": "SMP:SCHED:DOUBLE_RUN",
            "gs-iret-fault": "SMP:GS:IRET_FIXUP:PASS",
            "gs-nmi-user": "SMP:FAULT:AP_NMI_IST:PASS",
            "shared-scratch": "SMP:SCRATCH:CROSS:CPU=*",
            "lock-in-handler": "SMP:LOCKDEP:IN_HANDLER:10",
            "lockpair-order": "SMP:LOCKDEP:ORDER:20->20",
            "lock-stuck": "SMP:LOCK:STUCK:10:CPU=1"}
# Audit F8: the GS fixture prints the runtime CR4 (decimal); FSGSBASE (bit 16)
# must be clear, the value itself may differ per firmware.
GS_CR4 = "SMP:GS:CR4=*"
# Step 5b-2: which CPU first sees the shared slot crossed varies per run.
SCRATCH_CROSS = re.compile(r"SMP:SCRATCH:CROSS:CPU=([0-9])")
# Step 5a: the BSP lock self-test after smp::start, on every non-terminal path.
LOCKDEP_PASS = "SMP:LOCKDEP:PASS"
# Audit F7 (DG-2): every CPU that ran the sync test writes these lines at once,
# in fragments; each must arrive whole, in order per CPU, before SERIAL_PASS.
SERIAL_LINES = 200  # kernel/src/m2/smp.rs SERIAL_LINES
SERIAL_LINE = re.compile(r"SMP:SER:([0-9]):([0-9]{1,3})")
SERIAL_PASS = "SMP:SERIAL:PASS"


def serial_lines(lines: list[str], control: bool = False) -> tuple[list[str], dict[int, list[int]]]:
    """Split the F7 fixture lines from the rest; a line that is not whole fails.

    The fixture runs between the first SMP:ONLINE line and SERIAL_PASS; a line
    that only contains a fixture line (two writers mixed) is torn. `control`
    (no SERIAL lock): drop every fixture-phase line and require a torn one.
    """
    rest, seen, phase, torn = [], {}, "before", False
    for line in lines:
        if control and phase == "open" and line != SERIAL_PASS:
            torn |= SERIAL_LINE.fullmatch(line) is None
            continue
        if "SMP:SER:" in line:
            match = SERIAL_LINE.fullmatch(line)
            if match is None:
                raise ValueError("Torn serial line: two CPUs mixed one marker")
            if phase != "open":
                raise ValueError("Serial fixture line outside its phase")
            seen.setdefault(int(match[1]), []).append(int(match[2]))
            continue
        if phase == "before" and line.startswith("SMP:ONLINE:"):
            phase = "open"
        elif phase == "open" and line == SERIAL_PASS:
            phase = "closed"
        rest.append(line)
    if control and not torn:
        raise ValueError("Unlocked control showed no mixed line")
    return rest, seen


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


def expected_smp(case: str) -> list[str]:
    """The exact SMP lines between KS-03 and the terminal markers.

    QEMU's default flat topology numbers the BSP 0 and APs 1..n-1 in MADT order.
    """
    _, cpus, _, outcome = CASES[case]
    if outcome == "invalid-madt":
        return ["SMP:UNAVAILABLE:INVALID_MADT"]
    madt = [f"SMP:MADT:CPUS={cpus}:DISABLED=0:HOTPLUG=0"]

    def settled(online: int) -> list[str]:
        """Serial fixture, sync, IPI rounds, shootdown + visibility, preemption, spin, scheduler,
        per-CPU scratch slots under KERNEL, lock_pair rounds, park."""
        return [SERIAL_PASS, f"SMP:SYNC:PASS:CPUS={online}",
                f"SMP:IPI:PASS:ROUNDS={IPI_ROUNDS}:CPUS={online}",
                f"SMP:SHOOTDOWN:PASS:CPUS={online}",
                f"SMP:SHOOTDOWN:VISIBLE:CPUS={online}",
                f"SMP:PREEMPT:PASS:CPUS={online}",
                f"SMP:SPIN:PASS:CPUS={online}",
                f"SMP:SCHED:PASS:CPUS={online}",
                f"SMP:SCRATCH:PASS:CPUS={online}",
                f"SMP:LOCKPAIR:PASS:CPUS={online}",
                f"SMP:PARKED:APS={online - 1}"]

    if outcome in ("timeout", "mismatch"):
        status = "TIMEOUT" if outcome == "timeout" else "MISMATCH"
        return madt + [f"SMP:AP:1:{status}", "SMP:BRINGUP:STOPPED", f"SMP:ONLINE:1/{cpus}"] \
            + settled(1)
    started = min(cpus, MAX_CPUS)
    lines = madt + [f"SMP:AP:{apic}:ONLINE" for apic in range(1, started)]
    if cpus > MAX_CPUS:
        lines.append(f"SMP:SKIPPED:CAPACITY={cpus - MAX_CPUS}")
    lines += [f"SMP:ONLINE:{started}/{cpus}"]
    if outcome in ("online", "serial-torn"):
        # serial-torn (control, DG-2): without SERIAL the fixture must mix lines.
        return lines + settled(started)
    if outcome == "lockdep-order":
        # KERNEL(10) taken while FRAMES(30) is held: stop before the PASS line.
        return lines + settled(started) + [TERMINAL[outcome]]
    if outcome == "gs-user":
        # Audit F8 N1: ring 3 with its own GS on every AP, after the scheduler.
        done = settled(started)
        return lines + done[:-1] + [GS_CR4, f"SMP:GS:PASS:CPUS={started}", done[-1]]
    if outcome in ("gs-iret-fault", "gs-nmi-user"):
        return lines + settled(started)[:-1] + [GS_CR4, TERMINAL[outcome]]
    if outcome == "double-run":
        # Step 5d-1: thread 0 queued on two APs; the second dispatch must refuse.
        return lines + settled(started)[:-4] + [TERMINAL[outcome]]
    if outcome == "shared-scratch":
        # Step 5b-2 control: one slot for every CPU must be caught crossing.
        return lines + settled(started)[:-3] + [TERMINAL[outcome]]
    if outcome == "lock-in-handler":
        # Audit F3 (LD-1): the first 0xe8 handler takes KERNEL in the scheduler phase.
        return lines + settled(started)[:-4] + [TERMINAL[outcome]]
    if outcome == "lockpair-order":
        # Audit F4 (LD-3): two ASPACE locks taken loose and descending, after scratch.
        return lines + settled(started)[:-2] + [TERMINAL[outcome]]
    if outcome == "lock-stuck":
        # Audit F5 (DG-1): the AP waiting for the BSP's lock reports it, release build.
        return lines + settled(started)[:3] + [TERMINAL[outcome]]
    lines += [SERIAL_PASS, f"SMP:SYNC:PASS:CPUS={started}"]
    ipi_pass = f"SMP:IPI:PASS:ROUNDS={IPI_ROUNDS}:CPUS={started}"
    if outcome == "held-across-switch":
        # Step 5c without deferral: the first tick in a lock wait switches threads.
        return lines + [ipi_pass, f"SMP:SHOOTDOWN:PASS:CPUS={started}",
                        f"SMP:SHOOTDOWN:VISIBLE:CPUS={started}", TERMINAL[outcome]]
    if outcome == "shootdown-quarantine":
        return lines + [ipi_pass, "SMP:SHOOTDOWN:TIMEOUT:APIC=1",
                        "SMP:SHOOTDOWN:FRAME_QUARANTINED",
                        f"SMP:QUARANTINED:APS={started - 1}", f"SMP:ONLINE:1/{cpus}"]
    if outcome == "timer-quarantine":
        return lines + [ipi_pass, f"SMP:SHOOTDOWN:PASS:CPUS={started}",
                        f"SMP:SHOOTDOWN:VISIBLE:CPUS={started}", "SMP:PREEMPT:TIMEOUT:APIC=1",
                        f"SMP:QUARANTINED:APS={started - 1}", f"SMP:ONLINE:1/{cpus}"]
    if outcome == "stale-control":
        # Without the shootdown every AP must still reach the reclaimed frame.
        return lines + [ipi_pass, "SMP:SHOOTDOWN:SKIPPED",
                        f"SMP:SHOOTDOWN:STALE:APS={started - 1}",
                        f"SMP:PREEMPT:PASS:CPUS={started}",
                        f"SMP:SPIN:PASS:CPUS={started}",
                        f"SMP:SCHED:PASS:CPUS={started}",
                        f"SMP:SCRATCH:PASS:CPUS={started}",
                        f"SMP:LOCKPAIR:PASS:CPUS={started}",
                        f"SMP:PARKED:APS={started - 1}"]
    if outcome == "spin-quarantine":
        # Without the interrupt window a CPU waiting for KERNEL is deaf.
        return lines + [ipi_pass, f"SMP:SHOOTDOWN:PASS:CPUS={started}",
                        f"SMP:SHOOTDOWN:VISIBLE:CPUS={started}",
                        f"SMP:PREEMPT:PASS:CPUS={started}",
                        "SMP:SPIN:SHOOTDOWN_TIMEOUT:APIC=1",
                        f"SMP:QUARANTINED:APS={started - 1}", f"SMP:ONLINE:1/{cpus}"]
    if outcome == "quarantine":
        return lines + ["SMP:IPI:TIMEOUT:APIC=1", f"SMP:QUARANTINED:APS={started - 1}",
                        f"SMP:ONLINE:1/{cpus}"]
    if outcome == "nmi-ist":
        return lines + [f"SMP:IPI:PASS:ROUNDS={IPI_ROUNDS}:CPUS={started}", TERMINAL[outcome]]
    if outcome == "df-ist":
        return lines + [TERMINAL[outcome]]
    raise ValueError("Unknown SMP outcome")


def required_end(outcome: str) -> tuple[str, ...]:
    """Exact terminal lines for is_success; wildcard markers are checked in the tail."""
    end = (TERMINAL[outcome],) if outcome in TERMINAL else END
    return tuple(marker for marker in end if not marker.endswith("*"))


def evaluate(case: str, stdout: str, raw_exit: int | None, timed_out: bool) -> dict:
    """Independent serial oracle: the exact SMP sequence after KS-03."""
    profile, cpus, _, outcome = CASES[case]
    terminal = outcome in TERMINAL
    end = required_end(outcome)
    # DG-2: `!` marks a terminal line written without SERIAL: only the last
    # line of a terminal profile may carry it (the exact sequence checks it).
    unlocked = stdout.splitlines()
    if terminal and unlocked and unlocked[-1].startswith("!"):
        stdout = "\n".join(unlocked[:-1] + [unlocked[-1][1:]]) + "\n"
    if any(line.startswith("!") for line in stdout.splitlines()):
        raise ValueError("Unlocked (!) serial line that is not the terminal marker")
    if not is_success(raw_exit, stdout, timed_out, required_markers=BOOT + end,
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
    if tuple(observed[:len(BOOT)]) != BOOT:
        raise ValueError("Unexpected boot prefix")
    expected = expected_smp(case)
    control = outcome == "serial-torn"
    body, seen = serial_lines(observed[len(BOOT):], control)
    writers = next((int(m[1]) for m in (re.fullmatch(r"SMP:ONLINE:([0-9]+)/[0-9]+", line)
                                        for line in body) if m), 0)
    if not control and seen != {cpu: list(range(SERIAL_LINES))
                                for cpu in range(writers if SERIAL_PASS in expected else 0)}:
        raise ValueError("Serial fixture lines missing, duplicated or out of order")
    tail = []
    for line in body:
        cr4 = re.fullmatch(r"SMP:GS:CR4=([0-9]{1,20})", line)
        if cr4 and int(cr4[1]) & (1 << 16) == 0:
            line = GS_CR4
        cross = SCRATCH_CROSS.fullmatch(line)
        if cross and int(cross[1]) < min(cpus, MAX_CPUS):
            line = TERMINAL["shared-scratch"]
        tail.append(line)
    if terminal:
        if tail != expected:
            raise ValueError("Unexpected SMP sequence or missing IST marker")
    elif tail != expected + [LOCKDEP_PASS] + list(END):
        raise ValueError("Unexpected SMP sequence or duplicate output")
    online = 1 if outcome in QUARANTINE_OUTCOMES \
        else min(cpus, MAX_CPUS)
    return {"outcome": outcome, "profile": profile, "online": online,
            "ap_ids": sorted({int(m[1]) for m in (re.fullmatch(r"SMP:AP:([0-9]+):.*", line)
                                                   for line in expected) if m})}


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
              "scope": ("IF0 BSP; AP bring-up, per-AP TSS/IST, bounded unicast IPIs and sync "
                        "self-test, TLB shootdown, AP preemption (deferred while a lock is held or "
                        "awaited), AP spin with IF window, per-AP run queues of kernel threads "
                        "in two address spaces (0xe8 wake, CR3 switch with active_cpus), a ring-3 "
                        "GS fixture per AP in test-smp-gs-* profiles (swapgs discipline), a "
                        "scratch slot per CPU plus the KERNEL lock under contention, ASPACE lock_pair "
                        "rounds on every CPU (KERNEL/ASPACE refused in handlers), whole serial lines "
                        "under the SERIAL lock from every CPU at once (an unlocked control must mix), "
                        "a time-bounded lock wait reported as STUCK in release builds, and BSP "
                        "ticket-lock/lockdep self-test in "
                        "QEMU/TCG; no ring-3 processes (M3) on APs, no hardware evidence"),
              "cases": []}
    save_json(destination, report)
    for case in cases:
        _, cpus, threads, outcome = CASES[case]
        if source_hash() != frozen or sha256(manifest_path) != manifest_hash:
            raise ValueError("SMP source or manifest changed before a run")
        tables = []
        if case == "duplicate-madt":
            fixture = BUILD / "acpi" / "duplicate-madt.aml"
            fixture.parent.mkdir(parents=True, exist_ok=True)
            fixture.write_bytes(duplicate_madt())
            tables = [fixture]
        end = required_end(outcome)
        guest = run_guest(
            "smp-" + case, timeout=60, required_markers=BOOT + end,
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

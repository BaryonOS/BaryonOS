#!/usr/bin/env python3
"""K06: verify native AP bring-up, per-AP IST and bounded IPIs (native-smp profiles)."""
from __future__ import annotations
import argparse
import json
import math
import re
import struct
from common import BUILD, save_json, sha256, source_hash, without_acpi_diagnostics
from run import run_guest, is_success

MAX_CPUS = 8  # kernel/src/m2/smp.rs MAX_CPUS
IPI_ROUNDS = 3  # kernel/src/m2/smp.rs IPI_ROUNDS
ICR_ROUNDS = 10000  # kernel/src/m2/smp.rs ICR_ROUNDS
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
    "resched-deaf": ("test-smp-resched-deaf", 4, "multi", "kill-quarantine"),
    "icr-race-control": ("test-smp-icr-race", 4, "multi", "icr-race"),
    "quarantine-lock-held": ("test-smp-quarantine-lock-held", 4, "multi", "lock-held"),
    "bsp-spin-irq-off-control": ("test-smp-bsp-spin-irq-off", 4, "multi", "bsp-spin-timeout"),
    "proc-shootdown-deaf": ("test-smp-proc-shootdown-deaf", 4, "multi", "proc-shootdown-quarantine"),
    "proc-no-shootdown-control": ("test-smp-proc-no-shootdown", 4, "multi", "proc-stale-read"),
    "lock-stuck": ("test-smp-lock-stuck", 2, "multi", "lock-stuck"),
    # K06 step 5d-2: the base M3 manifest on every online CPU (R3-1).
    "m3-one": ("native-smp-m3", 1, "single", "m3"),
    "m3-one-multi": ("native-smp-m3", 1, "multi", "m3"),
    "m3-two-single": ("native-smp-m3", 2, "single", "m3"),
    "m3-two": ("native-smp-m3", 2, "multi", "m3"),
    "m3-four-single": ("native-smp-m3", 4, "single", "m3"),
    "m3-four": ("native-smp-m3", 4, "multi", "m3"),
    "m3-eight-single": ("native-smp-m3", 8, "single", "m3"),
    "m3-eight": ("native-smp-m3", 8, "multi", "m3"),
    "m3-ipc-four-single": ("native-smp-m3-ipc", 4, "single", "m3-ipc"),
    "m3-ipc-four": ("native-smp-m3-ipc", 4, "multi", "m3-ipc"),
    "m3-migrate-four-single": ("native-smp-m3-migrate", 4, "single", "m3-migrate"),
    "m3-migrate-four": ("native-smp-m3-migrate", 4, "multi", "m3-migrate"),
    "m3-migrate-eight-single": ("native-smp-m3-migrate", 8, "single", "m3-migrate"),
    "m3-migrate-eight": ("native-smp-m3-migrate", 8, "multi", "m3-migrate"),
    "m3-canary-control": ("test-smp-m3-canary", 4, "multi", "m3-migrate-corrupt"),
    "m3-fault-ap2": ("test-smp-m3-fault", 4, "multi", "m3-fault"),
    "m3-fpu-skip-control": ("test-smp-m3-fpu-skip", 4, "multi", "m3-fpu-corrupt"),
    "serial-unlocked-control": ("test-smp-serial-unlocked", 4, "multi", "serial-torn"),
    "m3-load-one": ("native-smp-m3-load", 1, "single", "m3-load"),
    "m3-load-one-multi": ("native-smp-m3-load", 1, "multi", "m3-load"),
    "m3-load-two-single": ("native-smp-m3-load", 2, "single", "m3-load"),
    "m3-load-two": ("native-smp-m3-load", 2, "multi", "m3-load"),
    "m3-load-four-single": ("native-smp-m3-load", 4, "single", "m3-load"),
    "m3-load-four": ("native-smp-m3-load", 4, "multi", "m3-load"),
    "m3-load-eight-single": ("native-smp-m3-load", 8, "single", "m3-load"),
    "m3-load-eight": ("native-smp-m3-load", 8, "multi", "m3-load"),
    "m3-soak-four": ("native-smp-m3-soak", 4, "multi", "m3-soak"),
    "m4-quotas-four": ("native-smp-m4-quotas", 4, "multi", "m4-quotas"),
    "qsim-four": ("native-smp-qsim", 4, "multi", "qsim"),
    "m3-kill-four": ("native-smp-m3-kill", 4, "multi", "m3-kill"),
    "m3-unmap-four": ("native-smp-m3-unmap", 4, "multi", "m3-unmap"),
    "m3-unmap-control": ("test-smp-m3-no-shootdown", 4, "multi", "m3-unmap-control"),
    "m3-recovery-four": ("test-smp-m3-recovery", 4, "multi", "m3-recovery"),
}
QUARANTINE_OUTCOMES = ("invalid-madt", "timeout", "mismatch", "quarantine", "shootdown-quarantine",
                       "timer-quarantine", "spin-quarantine", "kill-quarantine",
                       "proc-shootdown-quarantine")
TERMINAL = {"nmi-ist": "SMP:FAULT:AP_NMI_IST:PASS", "df-ist": "SMP:FAULT:AP_DF_IST:PASS",
            "lockdep-order": "SMP:LOCKDEP:ORDER:30->10",
            "held-across-switch": "SMP:LOCKDEP:HELD_ACROSS_SWITCH",
            "double-run": "SMP:SCHED:DOUBLE_ENQUEUE",
            "gs-iret-fault": "SMP:GS:IRET_FIXUP:PASS",
            "gs-nmi-user": "SMP:FAULT:AP_NMI_IST:PASS",
            "shared-scratch": "SMP:SCRATCH:CROSS:CPU=*",
            "lock-in-handler": "SMP:LOCKDEP:IN_HANDLER:10",
            "lockpair-order": "SMP:LOCKDEP:ORDER:20->20",
            "icr-race": "SMP:ICR:*",
            "lock-held": "SMP:QUARANTINE:LOCK_HELD:10:CPU=2",
            "bsp-spin-timeout": "SMP:SPIN:SHOOTDOWN_TIMEOUT:APIC=0",
            "proc-stale-read": "SMP:SHOOTDOWN:STALE_READ",
            "lock-stuck": "SMP:LOCK:STUCK:10:CPU=1",
            "m3-fpu-corrupt": "SMP:M3:FPU:CORRUPT:CONTROL",
            "m3-migrate-corrupt": "SMP:MIGRATE:CANARY:CORRUPT:CONTROL"}
# Audit F8: the GS fixture prints the runtime CR4 (decimal); FSGSBASE (bit 16)
# must be clear, the value itself may differ per firmware.
GS_CR4 = "SMP:GS:CR4=*"
# Step 5b-2: which CPU first sees the shared slot crossed varies per run.
SCRATCH_CROSS = re.compile(r"SMP:SCRATCH:CROSS:CPU=([0-9])")
# Audit F19 control: AP1's 0xe8 lands on AP1 itself (MISROUTED), or merges
# there with AP2's pending one (LOST); either shows the replaced destination.
ICR_RACE_SEEN = ("SMP:ICR:MISROUTED", "SMP:ICR:LOST")
# Step 5a: the BSP lock self-test after smp::start, on every non-terminal path.
LOCKDEP_PASS = "SMP:LOCKDEP:PASS"
# Step 5c-2 (BT-5, ADR-080): LAPIC ticks (divide 16) per PIT IRQ0 tick. The band
# was fixed in ADR-080 before the first measurement, for QEMU's 1 GHz APIC timer
# clock (62 500 expected); it is not widened after a run.
RATIO = "SMP:TIMER:RATIO:*"
RATIO_BAND = range(50_000, 75_001)
# The BSP's 0xe8 wake latency in microseconds; the kernel stops at >= 100 ms.
IDLE_WAKE = "SMP:IDLE:WAKE_US=*"
IDLE_WAKE_LIMIT_US = 100_000
# Existing per-boot host watchdog, including firmware startup. This is distinct
# from guest deadlines and must not grow in response to a failed repetition.
GUEST_TIMEOUT_S = 60
# The soak deliberately runs ten guest minutes. Keep its startup/teardown
# allowance separate from every existing sixty-second watchdog (ADR-5F).
SOAK_TIMEOUT_S = 660

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
        # RATIO is written before GO, before any AP writes a fixture line.
        if control and phase == "open" and line != SERIAL_PASS and not line.startswith("SMP:TIMER:RATIO:"):
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


def bsp_phases(online: int) -> list[str]:
    """Step 5c-2 between the AP spin proof and the scheduler."""
    lines = [f"SMP:CLOCK:PASS:CPUS={online}", f"SMP:BSP_PREEMPT:PASS:CPUS={online}"]
    if online == 1:
        return lines + ["SMP:BSP_SPIN:SKIPPED:CPUS=1", "SMP:IDLE:SKIPPED:CPUS=1"]
    return lines + [f"SMP:BSP_SPIN:PASS:CPUS={online}", IDLE_WAKE, f"SMP:IDLE:PASS:CPUS={online}"]


def expected_smp(case: str) -> list[str]:
    """The exact SMP lines between KS-03 and the terminal markers.

    QEMU's default flat topology numbers the BSP 0 and APs 1..n-1 in MADT order.
    """
    _, cpus, _, outcome = CASES[case]
    if outcome == "invalid-madt":
        return ["SMP:UNAVAILABLE:INVALID_MADT"]
    madt = [f"SMP:MADT:CPUS={cpus}:DISABLED=0:HOTPLUG=0"]

    def settled(online: int) -> list[str]:
        """BSP timer ratio, sync, IPI rounds, shootdown + visibility, preemption,
        spin, the BSP phases of step 5c-2 (clock under load, BSP preemption,
        BSP as shootdown target and idle wake + deferred resched, both skipped
        without APs), scheduler, block/wake ping-pong and kill (skipped without
        APs), per-CPU scratch slots under KERNEL, lock_pair rounds, the AP1/AP2
        ICR exchange (skipped below two APs), step 5e isolation and unmap race
        in two real spaces (skipped without APs), park. The serial fixture
        (audit F7) ends before sync."""
        bw = "SKIPPED" if online == 1 else "PASS"
        icr = f"SMP:ICR:PASS:ROUNDS={ICR_ROUNDS}" if online >= 3 else "SMP:ICR:SKIPPED"
        return [RATIO, SERIAL_PASS,
                f"SMP:SYNC:PASS:CPUS={online}",
                f"SMP:IPI:PASS:ROUNDS={IPI_ROUNDS}:CPUS={online}",
                f"SMP:SHOOTDOWN:PASS:CPUS={online}",
                f"SMP:SHOOTDOWN:VISIBLE:CPUS={online}",
                f"SMP:PREEMPT:PASS:CPUS={online}",
                f"SMP:SPIN:PASS:CPUS={online}",
                *bsp_phases(online),
                f"SMP:SCHED:PASS:CPUS={online}",
                f"SMP:WAKE:{bw}:CPUS={online}",
                f"SMP:KILL:{bw}:CPUS={online}",
                f"SMP:SCRATCH:PASS:CPUS={online}",
                f"SMP:LOCKPAIR:PASS:CPUS={online}",
                f"{icr}:CPUS={online}",
                f"SMP:ISOLATION:{bw}:CPUS={online}",
                f"SMP:UNMAP_RACE:{bw}:CPUS={online}",
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
    if outcome in M3_OUTCOMES:
        # Step 5d-2: every self-test as usual, then the APs wait for M3_GO.
        return lines + settled(started)[:-1] + [f"SMP:M3:WAITING:APS={started - 1}"]
    if outcome == "m3-fpu-corrupt":
        # Control: without the SMP FXRSTOR a canary sees a foreign state (int3).
        return lines + settled(started)[:-1] + [f"SMP:M3:WAITING:APS={started - 1}", LOCKDEP_PASS,
                                                "M2:PASS", f"SMP:M3:GO:CPUS={started}",
                                                TERMINAL[outcome]]
    if outcome == "m3-migrate-corrupt":
        # The real user canary rejects its corrupted GPR on the first destination.
        return lines + settled(started)[:-1] + [f"SMP:M3:WAITING:APS={started - 1}", LOCKDEP_PASS,
                                                "M2:PASS", f"SMP:M3:GO:CPUS={started}",
                                                "SMP:MIGRATE:FROM=1:TO=2", TERMINAL[outcome]]
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
        # BW-2 (audit F17): thread 0 pushed onto a second AP; the push refuses.
        return lines + settled(started)[:-9] + [TERMINAL[outcome]]
    if outcome == "shared-scratch":
        # Step 5b-2 control: one slot for every CPU must be caught crossing.
        return lines + settled(started)[:-6] + [TERMINAL[outcome]]
    if outcome == "lock-in-handler":
        # Audit F3 (LD-1): the first 0xe8 handler takes KERNEL in the scheduler phase.
        return lines + settled(started)[:-9] + [TERMINAL[outcome]]
    if outcome == "kill-quarantine":
        # BW-3: the kill target (thread 2 on AP slot 2 = APIC 3) ignores 0xe8.
        return lines + settled(started)[:-7] + ["SMP:KILL:TIMEOUT:APIC=3",
                                                f"SMP:QUARANTINED:APS={started - 1}",
                                                f"SMP:ONLINE:1/{cpus}"]
    if outcome == "lockpair-order":
        # Audit F4 (LD-3): two ASPACE locks taken loose and descending, after scratch.
        return lines + settled(started)[:-5] + [TERMINAL[outcome]]
    if outcome == "bsp-spin-timeout":
        # Step 5c-2 control: with the BSP window shut, AP1's round to the BSP
        # (APIC 0) while the BSP spins on KERNEL must time out.
        done = settled(started)
        return lines + done[:done.index(f"SMP:BSP_PREEMPT:PASS:CPUS={started}") + 1] \
            + [TERMINAL[outcome]]
    if outcome == "icr-race":
        # Audit F19 control: without the ICR lock a 0xd0 between ICR high and
        # low carries AP1's 0xe8 to AP1 itself.
        return lines + settled(started)[:-4] + [TERMINAL[outcome]]
    if outcome == "proc-shootdown-quarantine":
        # Step 5e: AP1 (the reader in space A) ignores A's round; the frame
        # stays quarantined, every AP is reset and the boot continues on one CPU.
        return lines + settled(started)[:-2] + ["SMP:SHOOTDOWN:TIMEOUT:APIC=1",
                                                "SMP:SHOOTDOWN:FRAME_QUARANTINED",
                                                f"SMP:QUARANTINED:APS={started - 1}",
                                                f"SMP:ONLINE:1/{cpus}"]
    if outcome == "proc-stale-read":
        # Step 5e control: without the round AP1 reads the reused frame.
        return lines + settled(started)[:-2] + [TERMINAL[outcome]]
    if outcome == "lock-stuck":
        # Audit F5 (DG-1): the AP waiting for the BSP's lock reports it, release build.
        return lines + settled(started)[:4] + [TERMINAL[outcome]]
    lines += [RATIO, SERIAL_PASS, f"SMP:SYNC:PASS:CPUS={started}"]
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
                        *bsp_phases(started),
                        f"SMP:SCHED:PASS:CPUS={started}",
                        f"SMP:WAKE:PASS:CPUS={started}",
                        f"SMP:KILL:PASS:CPUS={started}",
                        f"SMP:SCRATCH:PASS:CPUS={started}",
                        f"SMP:LOCKPAIR:PASS:CPUS={started}",
                        f"SMP:ICR:PASS:ROUNDS={ICR_ROUNDS}:CPUS={started}",
                        f"SMP:ISOLATION:PASS:CPUS={started}",
                        f"SMP:UNMAP_RACE:PASS:CPUS={started}",
                        f"SMP:PARKED:APS={started - 1}"]
    if outcome == "spin-quarantine":
        # Without the interrupt window a CPU waiting for KERNEL is deaf.
        return lines + [ipi_pass, f"SMP:SHOOTDOWN:PASS:CPUS={started}",
                        f"SMP:SHOOTDOWN:VISIBLE:CPUS={started}",
                        f"SMP:PREEMPT:PASS:CPUS={started}",
                        "SMP:SPIN:SHOOTDOWN_TIMEOUT:APIC=1",
                        f"SMP:QUARANTINED:APS={started - 1}", f"SMP:ONLINE:1/{cpus}"]
    if outcome == "lock-held":
        # Audit F15 (IQ-3): AP2 is deaf while it holds KERNEL; stop, no INIT report.
        return lines + ["SMP:IPI:TIMEOUT:APIC=2", TERMINAL[outcome]]
    if outcome == "quarantine":
        return lines + ["SMP:IPI:TIMEOUT:APIC=1", f"SMP:QUARANTINED:APS={started - 1}",
                        f"SMP:ONLINE:1/{cpus}"]
    if outcome == "nmi-ist":
        return lines + [f"SMP:IPI:PASS:ROUNDS={IPI_ROUNDS}:CPUS={started}", TERMINAL[outcome]]
    if outcome == "df-ist":
        return lines + [TERMINAL[outcome]]
    raise ValueError("Unknown SMP outcome")


# K06 step 5d-2: the base M3 manifest on every online CPU after the self-tests.
M3_OUTCOMES = ("m3", "m3-fault", "m3-ipc", "m3-migrate", "m3-load", "m3-soak", "m4-quotas", "qsim", "m3-kill", "m3-unmap", "m3-unmap-control", "m3-recovery")
M3_FAIR_OUTCOMES = ("m3-ipc", "m3-migrate", "m3-load", "m3-soak", "m4-quotas", "qsim")
M3_CPU = re.compile(r"SMP:M3:CPU=([0-9]):USER_TICKS=([0-9]{1,20}):SWITCHES=([0-9]{1,20})")
M3_MIN_SWITCHES = 1000  # kernel/src/m2/m3.rs smp_progress (spec 08 §4 FPU canary)
M3_FAIR_WAIT_MS = 400  # existing kernel/src/m2/smp.rs LOCK_STUCK_MS
M3_FAIR_TICKS = 40  # kernel/src/m2/smp.rs M3_FAIR_TICKS (10 ms timer)
M3_FAIR = re.compile(r"SMP:FAIR:CPU=([0-9]):WAIT_MS=([0-9]{1,20}):DEFERRED=([0-9]{1,20})"
                     r":LIMIT_MS=([0-9]{1,20}):LIMIT_TICKS=([0-9]{1,20})")
IPC_REMOTE = re.compile(r"SMP:IPC:REMOTE_WAKEUPS=([0-9]{1,20})")
IPC_RESCHEDULE = re.compile(r"SMP:IPC:RESCHEDULE_IPIS=([0-9]{1,20})")


def m3_end(outcome: str, online: int) -> list[str]:
    """After M2:PASS: GO, (the AP2 fault), per CPU user ticks > 0 and >= 1000
    switches, every live canary's own FPU image, cleanup balance, (K03), pass."""
    fault = outcome == "m3-fault"
    if outcome == "m3-recovery":
        return ["M2:PASS", "SMP:M3:GO:CPUS=4", "SMP:M3:RECOVERY:REQUEST:FROM=0:TARGET=3",
                "SMP:M3:SHOOTDOWN_TIMEOUT", "SMP:QUARANTINED:APS=3", "SMP:ONLINE:1/4",
                "SMP:M3:RECOVERY:CPUS=4:MIGRATED=*:CRASHED=*:PARKED=*:RETAINED_FRAMES=*:FREE=*",
                "SMP:M3:RECOVERY:RESUMED=*:CPU=0", "SMP:M3:RECOVERY:PASS",
                "SMP:M3:CPU=0:USER_TICKS=*:SWITCHES=*", "SMP:SCHED:CPUS=1",
                "SMP:M3:FPU:PASS:PROCESSES=*", "SMP:M3:CLEANUP:PASS", "SMP:M3:PASS:CPUS=1", "M1:PASS"]
    if outcome in ("m3-unmap", "m3-unmap-control"):
        control = outcome == "m3-unmap-control"
        return (["M2:PASS", f"SMP:M3:GO:CPUS={online}", "SMP:M3:ISOLATION:PASS", "SMP:M3:USERCOPY:UNMAP:PASS"]
                + ([] if control else ["SMP:M3:UNMAP:SHOOTDOWN:PASS"])
                + ["SMP:M3:UNMAP:REUSED:PASS", "SMP:M3:UNMAP:STALE:CONTROL" if control else "SMP:M3:UNMAP:FAULT:CPU=1",
                   "SMP:M3:UNMAP:CLEANUP:PASS"]
                + ([] if control else ["SMP:M3:UNMAP:PASS"]) + ["M1:PASS"])
    pressure = outcome in ("m4-quotas", "qsim")
    if pressure:
        tail = (["M2:PASS"] + (["QSIM:MANIFEST:PASS"] if outcome == "qsim" else [])
                + [f"SMP:M3:GO:CPUS={online}"]
                + [f"SMP:FAIR:CPU={cpu}:WAIT_MS=*:DEFERRED=*:LIMIT_MS=400:LIMIT_TICKS=40" for cpu in range(online)]
                + [f"SMP:M3:CPU={cpu}:USER_TICKS=*:SWITCHES=*" for cpu in range(online)])
        if outcome == "m4-quotas":
            return tail + ["M4:CASE:QUOTAS:PASS", "M4:SURVIVOR_PROGRESS:PASS", "SMP:K07:M4:PASS:CPUS=4",
                           "M4:CLEANUP:PASS", "KS-15:PASS", "M4:PASS", "M1:PASS"]
        return (tail + [f"QSIM:{field}:*" for _ in range(4) for field in
                        ("PROCESS_PID", "PROCESS_CHECKPOINTS", "PROCESS_USEFUL_ARENA", "PROCESS_MAX_WORK_GAP")]
                + [f"QSIM:{field}:PASS" for field in ("BELL4096", "SIX_QUBIT", "CANCEL", "DEADLINE", "RESOURCE_LIMITS", "PROCESS_ISOLATION")]
                + ["QSIM:TIMER_IRQS:*", "QSIM:SYSCALL_CALLS:*", "QSIM:CLEANUP_FRAMES:*", "SMP:K07:QSIM:PASS:CPUS=4",
                   "QSIM:CLEANUP:PASS", "QSIM:PASS", "M1:PASS"])
    return (["M2:PASS", f"SMP:M3:GO:CPUS={online}"]
            + (["SMP:M3:FAULT:UD:CPU=2"] if fault else [])
            + (["SMP:M3:REMOTE_KILL:REQUEST:FROM=0:TO=3", "SMP:M3:REMOTE_KILL:ACK:CPU=3:VECTOR=*:WAIT_MS=*:LIMIT_MS=100",
                "SMP:M3:REMOTE_KILL:CLEANUP:BEFORE=*:RELEASED=*:AFTER=*:ACTIVE=0", "SMP:M3:REMOTE_KILL:PASS:CPU=3"]
               if outcome == "m3-kill" else [])
            + (["SMP:MIGRATE:FROM=1:TO=2", "SMP:MIGRATE:ACTIVE:CPU=2:MASK=4",
                "SMP:MIGRATE:FROM=2:TO=0", "SMP:MIGRATE:ACTIVE:CPU=0:MASK=1",
                "SMP:MIGRATE:CANARY:PASS"]
               if outcome == "m3-migrate" else [])
            + (["SMP:LOAD:WORKERS=*:ELAPSED_MS=*"] if outcome in ("m3-load", "m3-soak") else [])
            + ([f"SMP:FAIR:CPU={cpu}:WAIT_MS=*:DEFERRED=*:LIMIT_MS=400:LIMIT_TICKS=40"
                for cpu in range(online)] if outcome in M3_FAIR_OUTCOMES else [])
            + (["SMP:IPC:PASS", "SMP:IPC:REMOTE_WAKEUPS=*", "SMP:IPC:RESCHEDULE_IPIS=*"]
               if outcome in ("m3-ipc", "m3-load", "m3-soak") else [])
            + [f"SMP:M3:CPU={cpu}:USER_TICKS=*:SWITCHES=*" for cpu in range(online)]
            + [f"SMP:SCHED:CPUS={online}",
               f"SMP:M3:FPU:PASS:PROCESSES={15 if fault or outcome == 'm3-kill' else 18 if outcome in ('m3-load', 'm3-soak') else 16}", "SMP:M3:CLEANUP:PASS"]
            + (["SMP:M3:K03:PASS"] if fault else [])
            + [f"SMP:M3:PASS:CPUS={online}", "M1:PASS"])


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
    deadline_margins = []
    recovery_crashed = None
    for line in body:
        cr4 = re.fullmatch(r"SMP:GS:CR4=([0-9]{1,20})", line)
        if cr4 and int(cr4[1]) & (1 << 16) == 0:
            line = GS_CR4
        if line in ICR_RACE_SEEN:
            line = TERMINAL["icr-race"]
        ratio = re.fullmatch(r"SMP:TIMER:RATIO:([0-9]{1,10})", line)
        if ratio and int(ratio[1]) in RATIO_BAND:
            line = RATIO
        wake = re.fullmatch(r"SMP:IDLE:WAKE_US=([0-9]{1,20})", line)
        if wake and int(wake[1]) < IDLE_WAKE_LIMIT_US:
            measured = int(wake[1])
            deadline_margins.append({"name": "bsp_idle_wake", "clock": "guest",
                                     "measured_us": measured, "deadline_us": IDLE_WAKE_LIMIT_US,
                                     "measured_over_deadline": measured / IDLE_WAKE_LIMIT_US,
                                     "remaining_us": IDLE_WAKE_LIMIT_US - measured})
            line = IDLE_WAKE
        m3_cpu = M3_CPU.fullmatch(line)
        if m3_cpu and int(m3_cpu[2]) > 0 and int(m3_cpu[3]) >= (1 if outcome in ("m4-quotas", "qsim") else M3_MIN_SWITCHES):
            line = f"SMP:M3:CPU={m3_cpu[1]}:USER_TICKS=*:SWITCHES=*"
        load = re.fullmatch(r"SMP:LOAD:WORKERS=([0-9]+):ELAPSED_MS=([0-9]+)", line)
        recovery = re.fullmatch(r"SMP:M3:RECOVERY:CPUS=4:MIGRATED=([0-9]+):CRASHED=([0-9]+):PARKED=([01]):RETAINED_FRAMES=([0-9]+):FREE=([0-9]+)", line)
        if recovery and outcome == "m3-recovery":
            migrated, crashed, parked, retained, free = map(int, recovery.groups())
            if 0 < migrated <= 12 and 0 <= crashed < 3 and 0 < retained < free <= (1 << 64) - 1:
                recovery_crashed = crashed
                line = "SMP:M3:RECOVERY:CPUS=4:MIGRATED=*:CRASHED=*:PARKED=*:RETAINED_FRAMES=*:FREE=*"
        resumed = re.fullmatch(r"SMP:M3:RECOVERY:RESUMED=([0-9]+):CPU=0", line)
        if resumed and outcome == "m3-recovery" and 0 < int(resumed[1]) <= 12:
            line = "SMP:M3:RECOVERY:RESUMED=*:CPU=0"
        fpu_recovery = re.fullmatch(r"SMP:M3:FPU:PASS:PROCESSES=([0-9]+)", line)
        if fpu_recovery and outcome == "m3-recovery" and recovery_crashed is not None and int(fpu_recovery[1]) == 15 - recovery_crashed:
            line = "SMP:M3:FPU:PASS:PROCESSES=*"
        kill_ack = re.fullmatch(r"SMP:M3:REMOTE_KILL:ACK:CPU=3:VECTOR=([0-9]+):WAIT_MS=([0-9]+):LIMIT_MS=100", line)
        if kill_ack and outcome == "m3-kill" and int(kill_ack[1]) in (128, 0xd0, 0xe8) and int(kill_ack[2]) < 100:
            line = "SMP:M3:REMOTE_KILL:ACK:CPU=3:VECTOR=*:WAIT_MS=*:LIMIT_MS=100"
            measured = int(kill_ack[2])
            deadline_margins.append({"name": "remote_kill_ack", "clock": "guest", "measured_ms": measured,
                                     "deadline_ms": 100, "measured_over_deadline": measured / 100, "remaining_ms": 100 - measured})
        kill_free = re.fullmatch(r"SMP:M3:REMOTE_KILL:CLEANUP:BEFORE=([0-9]+):RELEASED=([0-9]+):AFTER=([0-9]+):ACTIVE=0", line)
        if kill_free and outcome == "m3-kill":
            before, released, after = map(int, kill_free.groups())
            if 0 < released and before + released == after <= (1 << 64) - 1:
                line = "SMP:M3:REMOTE_KILL:CLEANUP:BEFORE=*:RELEASED=*:AFTER=*:ACTIVE=0"
        if load and outcome in ("m3-load", "m3-soak"):
            workers, elapsed = map(int, load.groups())
            minimum = 600_000 if outcome == "m3-soak" else 30_000
            if workers == 16 and workers >= 2 * cpus and minimum <= elapsed <= (1 << 64) - 1:
                line = "SMP:LOAD:WORKERS=*:ELAPSED_MS=*"
        if outcome == "qsim":
            numeric = re.fullmatch(r"QSIM:([A-Z_]+):([0-9]{20})", line)
            if numeric:
                field, value = numeric[1], int(numeric[2])
                allowed = {"PROCESS_PID": range(1, 5), "PROCESS_USEFUL_ARENA": (7216,), "PROCESS_MAX_WORK_GAP": (448,)}
                valid = value in allowed[field] if field in allowed else field in (
                    "PROCESS_CHECKPOINTS", "TIMER_IRQS", "SYSCALL_CALLS", "CLEANUP_FRAMES") and 0 < value <= (1 << 64) - 1
                if valid:
                    line = f"QSIM:{field}:*"
        fairness = M3_FAIR.fullmatch(line)
        if fairness and outcome in M3_FAIR_OUTCOMES:
            cpu, wait_ms, deferred, limit_ms, limit_ticks = map(int, fairness.groups())
            if (cpu < min(cpus, MAX_CPUS) and limit_ms == M3_FAIR_WAIT_MS
                    and limit_ticks == M3_FAIR_TICKS and wait_ms <= limit_ms and deferred <= limit_ticks):
                deadline_margins.extend([
                    {"name": "kernel_lock_wait", "clock": "guest", "cpu": cpu,
                     "measured_ms": wait_ms, "deadline_ms": limit_ms,
                     "measured_over_deadline": wait_ms / limit_ms, "remaining_ms": limit_ms - wait_ms},
                    {"name": "kernel_lock_deferred", "clock": "guest", "cpu": cpu,
                     "measured_ticks": deferred, "deadline_ticks": limit_ticks,
                     "measured_over_deadline": deferred / limit_ticks,
                     "remaining_ticks": limit_ticks - deferred},
                ])
                line = f"SMP:FAIR:CPU={cpu}:WAIT_MS=*:DEFERRED=*:LIMIT_MS=400:LIMIT_TICKS=40"
        remote = IPC_REMOTE.fullmatch(line)
        if remote and outcome in ("m3-ipc", "m3-load", "m3-soak") and (0 if cpus == 1 else 1) <= int(remote[1]) <= (1 << 64) - 1:
            line = "SMP:IPC:REMOTE_WAKEUPS=*"
        received = IPC_RESCHEDULE.fullmatch(line)
        if received and outcome in ("m3-ipc", "m3-load", "m3-soak") and (0 if cpus == 1 else 1) <= int(received[1]) <= (1 << 64) - 1:
            line = "SMP:IPC:RESCHEDULE_IPIS=*"
        cross = SCRATCH_CROSS.fullmatch(line)
        if cross and int(cross[1]) < min(cpus, MAX_CPUS):
            line = TERMINAL["shared-scratch"]
        tail.append(line)
    if terminal:
        if tail != expected:
            raise ValueError("Unexpected SMP sequence or missing IST marker")
    elif outcome in M3_OUTCOMES:
        if tail != expected + [LOCKDEP_PASS] + m3_end(outcome, min(cpus, MAX_CPUS)):
            raise ValueError("Unexpected SMP M3 sequence, idle CPU or missing cleanup")
    elif tail != expected + [LOCKDEP_PASS] + list(END):
        raise ValueError("Unexpected SMP sequence or duplicate output")
    online = 1 if outcome in QUARANTINE_OUTCOMES \
        else min(cpus, MAX_CPUS)
    if outcome == "qsim":
        pids = re.findall(r"^QSIM:PROCESS_PID:([0-9]{20})$", stdout, re.MULTILINE)
        if list(map(int, pids)) != [1, 2, 3, 4]:
            raise ValueError("Qsim SMP process identities missing, reordered or duplicated")
    return {"outcome": outcome, "profile": profile, "online": online,
            "deadline_margins": deadline_margins,
            "ap_ids": sorted({int(m[1]) for m in (re.fullmatch(r"SMP:AP:([0-9]+):.*", line)
                                                   for line in expected) if m})}


def run(cases: tuple[str, ...], repeat: int = 1) -> dict:
    if type(repeat) is not int or repeat < 1:
        raise ValueError("repeat must be a positive integer")
    if not cases or len(set(cases)) != len(cases) or any(case not in CASES for case in cases):
        raise ValueError("select non-empty, unique, known SMP cases")
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
              "scope": ("Profile-selected native SMP normal/negative fixtures in QEMU/TCG. "
                        "M3 profiles additionally exercise ring-3 scheduling and FPU state on APs. "
                        "Reached phases and disabled protections are identified by the selected "
                        "case, observed outcome and raw serial output. No physical hardware evidence."),
              "repeat": repeat,
              "repetitions": {case: {"requested": repeat, "attempted": 0, "passed": 0,
                                     "failed": 0, "complete": False, "flaky": False,
                                     "evidence_kind": "smp" if CASES[case][1] > 1 else "single_cpu_regression"}
                              for case in cases},
              "cases": []}
    save_json(destination, report)
    for case in cases:
        _, cpus, threads, outcome = CASES[case]
        watchdog = SOAK_TIMEOUT_S if outcome == "m3-soak" else GUEST_TIMEOUT_S
        tables = []
        if case == "duplicate-madt":
            fixture = BUILD / "acpi" / "duplicate-madt.aml"
            fixture.parent.mkdir(parents=True, exist_ok=True)
            fixture.write_bytes(duplicate_madt())
            tables = [fixture]
        end = required_end(outcome)
        summary = report["repetitions"][case]
        for iteration in range(1, repeat + 1):
            if source_hash() != frozen or sha256(manifest_path) != manifest_hash:
                report["error"] = "SMP source or manifest changed before a run"
                save_json(destination, report)
                raise ValueError(report["error"])
            label = "smp-" + case + (f"-r{iteration:03d}-of-{repeat:03d}" if repeat > 1 else "")
            item = {"case": case, "iteration": iteration, "cpus": cpus,
                    "tcg_threads": threads, "passed": False}
            report["cases"].append(item)
            summary["attempted"] += 1
            save_json(destination, report)
            try:
                guest = run_guest(
                    label, timeout=watchdog, required_markers=BOOT + end,
                    forbidden_markers=FORBIDDEN, cpus=cpus, tcg_threads=threads,
                    acpi_tables=tables,
                    expected_kernel_sha256=manifest["kernel"]["sha256"],
                    expected_loader_sha256=manifest["loader"]["sha256"],
                )
                item["guest"] = guest
                if tables:
                    item["acpi_table_sha256"] = sha256(tables[0])
                if (guest.get("artifact_binding_passed") is not True
                        or guest.get("source_sha256") != frozen
                        or source_hash() != frozen or sha256(manifest_path) != manifest_hash):
                    raise ValueError("SMP source, artifacts or manifest changed during a run")
                measured = guest["duration_s"]
                if (not isinstance(measured, (int, float)) or isinstance(measured, bool)
                        or not math.isfinite(measured) or measured < 0
                        or guest.get("timeout_s") != watchdog):
                    raise ValueError("Invalid host watchdog measurement")
                item["host_watchdog"] = {"measured_s": measured, "deadline_s": watchdog,
                                         "measured_over_deadline": measured / watchdog,
                                         "remaining_s": watchdog - measured}
                try:
                    item["observed"] = evaluate(case, guest["stdout"], guest["raw_qemu_exit"],
                                                guest["timed_out"])
                    if guest.get("passed") is not True:
                        raise ValueError("Native SMP guest harness rejected the run")
                    item["passed"] = True
                except ValueError as exc:
                    item["error"] = str(exc)
            except BaseException as exc:
                item["error"] = str(exc) or type(exc).__name__
                report["error"] = item["error"]
                raise
            finally:
                summary["passed" if item["passed"] else "failed"] += 1
                summary["complete"] = summary["attempted"] == repeat and "error" not in report
                if "error" not in report:
                    summary["flaky"] = summary["passed"] > 0 and summary["failed"] > 0
                save_json(destination, report)
            if not item["passed"] and repeat == 1:
                raise ValueError(item["error"])
    report["passed"] = all(summary["complete"] and summary["passed"] == repeat
                           and summary["failed"] == 0 for summary in report["repetitions"].values())
    save_json(destination, report)
    return report


def profile_cases(profile: str) -> tuple[str, ...]:
    return tuple(case for case, spec in CASES.items() if spec[0] == profile)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=tuple(CASES))
    parser.add_argument("--profile", choices=sorted({spec[0] for spec in CASES.values()}))
    parser.add_argument("--repeat", type=int, default=1, help="boot each selected case this many times")
    args = parser.parse_args()
    if (args.case is None) == (args.profile is None):
        parser.error("choose exactly one of --case or --profile")
    if args.repeat < 1:
        parser.error("--repeat must be a positive integer")
    selected = (args.case,) if args.case else profile_cases(args.profile)
    raise SystemExit(0 if run(selected, repeat=args.repeat)["passed"] else 1)

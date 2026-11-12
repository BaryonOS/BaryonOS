import struct
import sys
import hashlib
import json
import tempfile
from pathlib import Path
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from run_smp import CASES, MAX_CPUS, SERIAL_LINES, duplicate_madt, evaluate
import run_smp

PREFIX = ("BdsDxe: starting Boot0002\nBOOT:EBS_OK\nKERNEL:ENTRY\nBOOTINFO:OK\nKS-01:PASS\n"
          "KS-02:PASS\nACPI:UNAVAILABLE:NO_DMAR\nDEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT\n"
          "KS-03:PASS\n")
END = ["M2:PASS", "M1:PASS"]


def fixture(cpus):
    """F7: each CPU's numbered lines, interleaved line by line (never torn)."""
    return [f"SMP:SER:{cpu}:{seq}" for seq in range(SERIAL_LINES) for cpu in range(cpus)]


def transcript(lines, lockdep=True, serial=True):
    """Non-terminal runs end with the step-5a lock self-test before END; the F7
    lines of every CPU that ran the sync test precede SMP:SERIAL:PASS."""
    if lockdep and lines[-2:] == END:
        lines = lines[:-2] + ["SMP:LOCKDEP:PASS"] + END
    if serial and "SMP:SERIAL:PASS" in lines:
        at = lines.index("SMP:SERIAL:PASS")
        cpus = int(next(line for line in lines if line.startswith("SMP:ONLINE:"))
                   .split(":")[2].split("/")[0])
        lines = lines[:at] + fixture(cpus) + lines[at:]
    return PREFIX + "".join(line + "\n" for line in lines)


# Hand-written expectations, deliberately independent of run_smp.expected_smp.
def started(cpus):
    count = min(cpus, MAX_CPUS)
    lines = [f"SMP:MADT:CPUS={cpus}:DISABLED=0:HOTPLUG=0"]
    lines += [f"SMP:AP:{apic}:ONLINE" for apic in range(1, count)]
    if cpus > MAX_CPUS:
        lines.append(f"SMP:SKIPPED:CAPACITY={cpus - MAX_CPUS}")
    return lines + [f"SMP:ONLINE:{count}/{cpus}", RATIO, "SMP:SERIAL:PASS",
                    f"SMP:SYNC:PASS:CPUS={count}"]


# Step 5c-2: QEMU's 1 GHz APIC timer clock / 16 gives 62 500 per PIT tick.
RATIO = "SMP:TIMER:RATIO:62500"


def bsp(count):
    """Step 5c-2 BSP phases; the shootdown target and idle wake need one AP."""
    lines = [f"SMP:CLOCK:PASS:CPUS={count}", f"SMP:BSP_PREEMPT:PASS:CPUS={count}"]
    if count == 1:
        return lines + ["SMP:BSP_SPIN:SKIPPED:CPUS=1", "SMP:IDLE:SKIPPED:CPUS=1"]
    return lines + [f"SMP:BSP_SPIN:PASS:CPUS={count}", "SMP:IDLE:WAKE_US=412",
                    f"SMP:IDLE:PASS:CPUS={count}"]


def wake_kill(count):
    """Audit F16/F17: ping-pong and kill need at least one AP."""
    status = "SKIPPED" if count == 1 else "PASS"
    return [f"SMP:WAKE:{status}:CPUS={count}", f"SMP:KILL:{status}:CPUS={count}"]


def icr(count):
    """Audit F19: the AP1/AP2 exchange needs two APs."""
    return f"SMP:ICR:PASS:ROUNDS=10000:CPUS={count}" if count >= 3 else f"SMP:ICR:SKIPPED:CPUS={count}"


def proc(count):
    """Step 5e: isolation and unmap race in two real spaces need one AP."""
    status = "SKIPPED" if count == 1 else "PASS"
    return [f"SMP:ISOLATION:{status}:CPUS={count}", f"SMP:UNMAP_RACE:{status}:CPUS={count}"]


def online(cpus):
    count = min(cpus, MAX_CPUS)
    return started(cpus) + [f"SMP:IPI:PASS:ROUNDS=3:CPUS={count}",
                            f"SMP:SHOOTDOWN:PASS:CPUS={count}",
                            f"SMP:SHOOTDOWN:VISIBLE:CPUS={count}",
                            f"SMP:PREEMPT:PASS:CPUS={count}",
                            f"SMP:SPIN:PASS:CPUS={count}"] + bsp(count) + [
                            f"SMP:SCHED:PASS:CPUS={count}"] + wake_kill(count) + [
                            f"SMP:SCRATCH:PASS:CPUS={count}",
                            f"SMP:LOCKPAIR:PASS:CPUS={count}", icr(count), *proc(count),
                            f"SMP:PARKED:APS={count - 1}"] + END


def m3(cpus, fault=False, ipc=False, migrate=False):
    """Step 5d-2: every self-test, the APs wait, then the M3 phase on each CPU."""
    count = min(cpus, MAX_CPUS)
    return (online(cpus)[:-3] + [f"SMP:M3:WAITING:APS={count - 1}", "SMP:LOCKDEP:PASS",
                                 "M2:PASS", f"SMP:M3:GO:CPUS={count}"]
            + (["SMP:M3:FAULT:UD:CPU=2"] if fault else [])
            + (["SMP:MIGRATE:FROM=1:TO=2", "SMP:MIGRATE:ACTIVE:CPU=2:MASK=4",
                "SMP:MIGRATE:FROM=2:TO=0", "SMP:MIGRATE:ACTIVE:CPU=0:MASK=1",
                "SMP:MIGRATE:CANARY:PASS"]
               if migrate else [])
            + ([f"SMP:FAIR:CPU={cpu}:WAIT_MS=1:DEFERRED=2:LIMIT_MS=400:LIMIT_TICKS=40"
                for cpu in range(count)] if ipc or migrate else [])
            + (["SMP:IPC:PASS", "SMP:IPC:REMOTE_WAKEUPS=17", "SMP:IPC:RESCHEDULE_IPIS=13"] if ipc else [])
            + [f"SMP:M3:CPU={cpu}:USER_TICKS={980 + cpu}:SWITCHES=1000" for cpu in range(count)]
            + [f"SMP:SCHED:CPUS={count}", f"SMP:M3:FPU:PASS:PROCESSES={15 if fault else 16}",
               "SMP:M3:CLEANUP:PASS"]
            + (["SMP:M3:K03:PASS"] if fault else [])
            + [f"SMP:M3:PASS:CPUS={count}", "M1:PASS"])


def failed(status):
    return ["SMP:MADT:CPUS=4:DISABLED=0:HOTPLUG=0", f"SMP:AP:1:{status}",
            "SMP:BRINGUP:STOPPED", "SMP:ONLINE:1/4", RATIO, "SMP:SERIAL:PASS", "SMP:SYNC:PASS:CPUS=1",
            "SMP:IPI:PASS:ROUNDS=3:CPUS=1", "SMP:SHOOTDOWN:PASS:CPUS=1",
            "SMP:SHOOTDOWN:VISIBLE:CPUS=1", "SMP:PREEMPT:PASS:CPUS=1",
            "SMP:SPIN:PASS:CPUS=1"] + bsp(1) + ["SMP:SCHED:PASS:CPUS=1"] + wake_kill(1) + [
            "SMP:SCRATCH:PASS:CPUS=1", "SMP:LOCKPAIR:PASS:CPUS=1", icr(1), *proc(1),
            "SMP:PARKED:APS=0"] + END


DEAF = started(4) + ["SMP:IPI:TIMEOUT:APIC=1", "SMP:QUARANTINED:APS=3", "SMP:ONLINE:1/4"] + END
NMI = started(4) + ["SMP:IPI:PASS:ROUNDS=3:CPUS=4", "SMP:FAULT:AP_NMI_IST:PASS"]
DOUBLE = started(4) + ["SMP:FAULT:AP_DF_IST:PASS"]
SHOOTDOWN_DEAF = started(4) + ["SMP:IPI:PASS:ROUNDS=3:CPUS=4", "SMP:SHOOTDOWN:TIMEOUT:APIC=1",
                               "SMP:SHOOTDOWN:FRAME_QUARANTINED", "SMP:QUARANTINED:APS=3",
                               "SMP:ONLINE:1/4"] + END
STALE_CONTROL = started(4) + ["SMP:IPI:PASS:ROUNDS=3:CPUS=4", "SMP:SHOOTDOWN:SKIPPED",
                              "SMP:SHOOTDOWN:STALE:APS=3", "SMP:PREEMPT:PASS:CPUS=4",
                              "SMP:SPIN:PASS:CPUS=4"] + bsp(4) + [
                              "SMP:SCHED:PASS:CPUS=4"] + wake_kill(4) + [
                              "SMP:SCRATCH:PASS:CPUS=4", "SMP:LOCKPAIR:PASS:CPUS=4", icr(4),
                              *proc(4), "SMP:PARKED:APS=3"] + END
LOCKDEP_ORDER = online(2)[:-2] + ["SMP:LOCKDEP:ORDER:30->10"]
TIMER_DEAD = started(4) + ["SMP:IPI:PASS:ROUNDS=3:CPUS=4", "SMP:SHOOTDOWN:PASS:CPUS=4",
                           "SMP:SHOOTDOWN:VISIBLE:CPUS=4", "SMP:PREEMPT:TIMEOUT:APIC=1",
                           "SMP:QUARANTINED:APS=3", "SMP:ONLINE:1/4"] + END
SPIN_DEAF = started(4) + ["SMP:IPI:PASS:ROUNDS=3:CPUS=4", "SMP:SHOOTDOWN:PASS:CPUS=4",
                          "SMP:SHOOTDOWN:VISIBLE:CPUS=4", "SMP:PREEMPT:PASS:CPUS=4",
                          "SMP:SPIN:SHOOTDOWN_TIMEOUT:APIC=1", "SMP:QUARANTINED:APS=3",
                          "SMP:ONLINE:1/4"] + END
HELD_ACROSS = started(2) + ["SMP:IPI:PASS:ROUNDS=3:CPUS=2", "SMP:SHOOTDOWN:PASS:CPUS=2",
                            "SMP:SHOOTDOWN:VISIBLE:CPUS=2", "SMP:LOCKDEP:HELD_ACROSS_SWITCH"]
# BW-2 (audit F17): the second push of thread 0 is refused at the push itself.
DOUBLE_RUN = online(4)[:-11] + ["SMP:SCHED:DOUBLE_ENQUEUE"]
# Step 5b-2 control: every CPU in scratch slot 0; any CPU may see the crossing.
SHARED = online(4)[:-8] + ["SMP:SCRATCH:CROSS:CPU=2"]
# Audit F3 (LD-1): the first 0xe8 handler takes KERNEL, in the scheduler phase.
LOCK_IN_HANDLER = online(2)[:-11] + ["SMP:LOCKDEP:IN_HANDLER:10"]
# Audit F4 (LD-3): two ASPACE locks taken loose and descending, after scratch.
LOCKPAIR_ORDER = online(2)[:-7] + ["SMP:LOCKDEP:ORDER:20->20"]
# Audit F5 (DG-1): the AP waits for a lock the BSP never frees, after the IPI rounds.
LOCK_STUCK = started(2) + ["SMP:IPI:PASS:ROUNDS=3:CPUS=2", "SMP:LOCK:STUCK:10:CPU=1"]


# ADR-082 fixtures: the boot stops inside the sync phase, so the serial fixture
# lines of the CPUs that finished are written out here.
def sync_stopped(writers, terminal):
    return started(4)[:-2] + [f"SMP:SER:{cpu}:{seq}" for seq in range(SERIAL_LINES)
                              for cpu in writers] + [terminal]


SYNC_STALL = sync_stopped((0, 2, 3), "SMP:SYNC:STALLED:DONE=2:TOTAL=80000")
SYNC_SLOW = sync_stopped((0, 2, 3), "SMP:SYNC:TIMEOUT:DONE=2:TOTAL=70411")
SYNC_LOST = sync_stopped((0, 1, 2, 3), "SMP:SYNC:LOST_UPDATE")
BEFORE_CLOCK = started(4) + ["SMP:IPI:PASS:ROUNDS=3:CPUS=4", "SMP:SHOOTDOWN:PASS:CPUS=4",
                             "SMP:SHOOTDOWN:VISIBLE:CPUS=4", "SMP:PREEMPT:PASS:CPUS=4",
                             "SMP:SPIN:PASS:CPUS=4"]
EDGES = "SMP:CLOCK:EDGES:TICKS=5000:LOST=312:HOST=2:EXTRA=0:SPAN_US=3051:US=1062411:OWN_US=1062008"
CLOCK_MASKED = BEFORE_CLOCK + [EDGES, "SMP:CLOCK:DRIFT:KERNEL"]
CLOCK_DIVISOR = BEFORE_CLOCK + ["SMP:CLOCK:PIT_COUNT"]
# Audit F8: the ring-3 GS fixture runs after the scheduler phase (CR4 = 104 is
# DE|PAE|MCE as printed under OVMF/TCG; only FSGSBASE, bit 16, is checked).
# BW-3: the kill target on AP3 (APIC 3) ignores 0xe8; INV-5 quarantine follows.
RESCHED_DEAF = online(4)[:-9] + ["SMP:KILL:TIMEOUT:APIC=3", "SMP:QUARANTINED:APS=3",
                                 "SMP:ONLINE:1/4"] + END
# IQ-3 precondition: AP1 holds its RUNQ in a tick until the quarantine stops the scheduler.
SCHED_BUSY = online(4)[:-9] + ["SMP:QUARANTINE:SCHED_BUSY:CPU=1", "SMP:KILL:TIMEOUT:APIC=3",
                               "SMP:QUARANTINED:APS=3", "SMP:ONLINE:1/4"] + END
# Audit F19 control: without the ICR lock AP1's 0xe8 is carried to AP1 itself.
ICR_RACE = online(4)[:-6] + ["SMP:ICR:MISROUTED"]
# Audit F15 (IQ-3): AP2 is deaf holding KERNEL; the BSP stops instead of INIT.
LOCK_HELD = started(4) + ["SMP:IPI:TIMEOUT:APIC=2", "SMP:QUARANTINE:LOCK_HELD:10:CPU=2"]
# Step 5c-2 control: the BSP window shut, AP1's round to the BSP times out.
BSP_SPIN_DEAF = online(4)[:online(4).index("SMP:BSP_PREEMPT:PASS:CPUS=4") + 1] + [
    "SMP:SPIN:SHOOTDOWN_TIMEOUT:APIC=0"]
# Step 5e: AP1 ignores space A's round; the frame stays quarantined, the boot goes on.
PROC_DEAF = online(4)[:-4] + ["SMP:PROC_DEAF:LOCK_FREE:CPUS=4",
                              "SMP:SHOOTDOWN:TIMEOUT:APIC=1", "SMP:SHOOTDOWN:FRAME_QUARANTINED",
                              "SMP:QUARANTINED:APS=3", "SMP:ONLINE:1/4"] + END
# Step 5e control: without the round the reader sees the reused frame.
PROC_STALE = online(4)[:-4] + ["SMP:SHOOTDOWN:STALE_READ"]
GS_USER = online(4)[:-3] + ["SMP:GS:CR4=104", "SMP:GS:PASS:CPUS=4"] + online(4)[-3:]
GS_IRET = online(4)[:-3] + ["SMP:GS:CR4=104", "SMP:GS:IRET_FIXUP:PASS"]
GS_NMI = online(4)[:-3] + ["SMP:GS:CR4=104", "SMP:FAULT:AP_NMI_IST:PASS"]

VALID = {
    "sync-stall": SYNC_STALL, "sync-slow": SYNC_SLOW, "sync-lost-update": SYNC_LOST,
    "clock-masked": CLOCK_MASKED, "clock-divisor": CLOCK_DIVISOR,
    "one": online(1), "two-single": online(2), "four": online(4), "eight": online(8),
    "nine-capacity": online(9), "duplicate-madt": ["SMP:UNAVAILABLE:INVALID_MADT"] + END,
    "ap-stall": failed("TIMEOUT"), "ap-mismatch": failed("MISMATCH"),
    "ap-ipi-deaf": DEAF, "ap-nmi-ist": NMI, "ap-double-fault-ist": DOUBLE,
    "ap-shootdown-deaf": SHOOTDOWN_DEAF, "shootdown-skipped-control": STALE_CONTROL,
    "ap-timer-dead": TIMER_DEAD, "lockdep-order": LOCKDEP_ORDER,
    "spin-irq-off-control": SPIN_DEAF, "held-across-switch": HELD_ACROSS,
    "double-run": DOUBLE_RUN, "gs-user": GS_USER, "gs-iret-fault": GS_IRET,
    "gs-nmi-user": GS_NMI, "shared-scratch": SHARED,
    "lock-in-handler": LOCK_IN_HANDLER, "lockpair-order": LOCKPAIR_ORDER,
    "resched-deaf": RESCHED_DEAF, "icr-race-control": ICR_RACE,
    "quarantine-lock-held": LOCK_HELD, "quarantine-lock-pending": LOCK_HELD, "quarantine-sched-busy": SCHED_BUSY, "bsp-spin-irq-off-control": BSP_SPIN_DEAF,
    "proc-shootdown-deaf": PROC_DEAF, "proc-no-shootdown-control": PROC_STALE,
    "lock-stuck": LOCK_STUCK, "serial-unlocked-control": online(4),
    "m3-one": m3(1), "m3-one-multi": m3(1), "m3-two-single": m3(2), "m3-two": m3(2),
    "m3-four-single": m3(4), "m3-four": m3(4), "m3-eight-single": m3(8), "m3-eight": m3(8),
    "m3-ipc-four-single": m3(4, ipc=True), "m3-ipc-four": m3(4, ipc=True),
    "m3-migrate-four-single": m3(4, migrate=True), "m3-migrate-four": m3(4, migrate=True),
    "m3-migrate-eight-single": m3(8, migrate=True), "m3-migrate-eight": m3(8, migrate=True),
    "m3-canary-control": m3(4)[:m3(4).index("SMP:M3:GO:CPUS=4") + 1]
    + ["SMP:MIGRATE:FROM=1:TO=2", "SMP:MIGRATE:CANARY:CORRUPT:CONTROL"],
    "m3-fault-ap2": m3(4, fault=True),
    "m3-fpu-skip-control": m3(4)[:m3(4).index("SMP:M3:GO:CPUS=4") + 1]
    + ["SMP:M3:FPU:CORRUPT:CONTROL"],
}


class SmpSerialOracle(unittest.TestCase):
    def test_migration_order_canary_and_negative_control_are_exact(self):
        good = m3(4, migrate=True)
        first = good.index("SMP:MIGRATE:FROM=1:TO=2")
        control = VALID["m3-canary-control"]
        mutations = {
            "no first migration": good[:first] + good[first + 1:],
            "no second migration": [line for line in good if line != "SMP:MIGRATE:FROM=2:TO=0"],
            "reordered": good[:first] + good[first + 2:first + 4] + good[first:first + 2] + good[first + 4:],
            "duplicate": good[:first] + [good[first]] + good[first:],
            "wrong destination": [line.replace("FROM=2:TO=0", "FROM=2:TO=1") for line in good],
            "missing active proof": [line for line in good if line != "SMP:MIGRATE:ACTIVE:CPU=2:MASK=4"],
            "missing BSP active proof": [line for line in good if line != "SMP:MIGRATE:ACTIVE:CPU=0:MASK=1"],
            "wrong active CPU": [line.replace("ACTIVE:CPU=2:MASK=4", "ACTIVE:CPU=1:MASK=4") for line in good],
            "old CPU still active": [line.replace("ACTIVE:CPU=2:MASK=4", "ACTIVE:CPU=2:MASK=6") for line in good],
            "wrong BSP mask": [line.replace("ACTIVE:CPU=0:MASK=1", "ACTIVE:CPU=0:MASK=0") for line in good],
            "premature active proof": good[:first] + good[first:first + 2][::-1] + good[first + 2:],
            "duplicate active proof": good[:first + 1] + [good[first + 1]] + good[first + 1:],
            "missing canary": [line for line in good if line != "SMP:MIGRATE:CANARY:PASS"],
            "control instead of pass": [line.replace("CANARY:PASS", "CANARY:CORRUPT:CONTROL")
                                        for line in good],
            "missing fairness": [line for line in good if not line.startswith("SMP:FAIR:CPU=2:")],
            "unbounded fairness": [line.replace("WAIT_MS=1:", "WAIT_MS=401:") for line in good],
        }
        for name, lines in mutations.items():
            with self.subTest(name=name), self.assertRaises(ValueError):
                evaluate("m3-migrate-four", transcript(lines, lockdep=False), 33, False)
        for lines in (good, control[:-2] + control[-1:], control + ["SMP:MIGRATE:FROM=2:TO=0"],
                      control + ["SMP:MIGRATE:CANARY:PASS"],
                      control + [next(line for line in good if line.startswith("SMP:FAIR:"))],
                      control[:-1] + ["SMP:MIGRATE:ACTIVE:CPU=2:MASK=4"] + control[-1:],
                      control[:-2] + control[-2:][::-1]):
            with self.subTest(control=lines[-3:]), self.assertRaises(ValueError):
                evaluate("m3-canary-control", transcript(lines, lockdep=False), 33, False)
        with self.assertRaises(ValueError):
            evaluate("m3-migrate-four", transcript(control, lockdep=False), 33, False)
        self.assertEqual({(cpus, threads) for profile, cpus, threads, _ in CASES.values()
                          if profile == "native-smp-m3-migrate"},
                         {(cpus, threads) for cpus in (4, 8) for threads in ("single", "multi")})

    def test_m3_matrix_covers_both_tcg_modes_at_each_cpu_count(self):
        self.assertEqual({(cpus, threads) for profile, cpus, threads, _ in CASES.values()
                          if profile == "native-smp-m3"},
                         {(cpus, threads) for cpus in (1, 2, 4, 8) for threads in ("single", "multi")})

    def test_ipc_requires_remote_wakeup_and_exact_bounded_fairness(self):
        good = m3(4, ipc=True)
        first_fair = good.index("SMP:FAIR:CPU=0:WAIT_MS=1:DEFERRED=2:LIMIT_MS=400:LIMIT_TICKS=40")
        bad = {
            "missing IPC": [line for line in good if line != "SMP:IPC:PASS"],
            "missing wakeup": [line for line in good if not line.startswith("SMP:IPC:REMOTE_WAKEUPS=")],
            "zero wakeups": [line.replace("REMOTE_WAKEUPS=17", "REMOTE_WAKEUPS=0") for line in good],
            "missing IPI receipt": [line for line in good if not line.startswith("SMP:IPC:RESCHEDULE_IPIS=")],
            "zero IPI receipts": [line.replace("RESCHEDULE_IPIS=13", "RESCHEDULE_IPIS=0") for line in good],
            "overflow IPI receipts": [line.replace("RESCHEDULE_IPIS=13", "RESCHEDULE_IPIS=18446744073709551616")
                                      for line in good],
            "overflow wakeups": [line.replace("REMOTE_WAKEUPS=17", "REMOTE_WAKEUPS=18446744073709551616")
                                 for line in good],
            "missing CPU": good[:first_fair] + good[first_fair + 1:],
            "duplicate CPU": good[:first_fair] + [good[first_fair]] + good[first_fair:],
            "wrong CPU": [line.replace("FAIR:CPU=3:", "FAIR:CPU=4:") for line in good],
            "wait over limit": [line.replace("WAIT_MS=1:", "WAIT_MS=401:") for line in good],
            "deferrals over limit": [line.replace("DEFERRED=2:", "DEFERRED=41:") for line in good],
            "changed ms limit": [line.replace("LIMIT_MS=400:", "LIMIT_MS=401:") for line in good],
            "changed tick limit": [line.replace("LIMIT_TICKS=40", "LIMIT_TICKS=41") for line in good],
            "wrong phase": good[:first_fair] + good[first_fair + 4:first_fair + 6]
                           + good[first_fair:first_fair + 4] + good[first_fair + 6:],
        }
        for name, lines in bad.items():
            with self.subTest(name=name), self.assertRaises(ValueError):
                evaluate("m3-ipc-four", transcript(lines, lockdep=False), 33, False)
        with self.assertRaises(ValueError):
            evaluate("m3-four", transcript(good, lockdep=False), 33, False)
        boundary = [line.replace("WAIT_MS=1:", "WAIT_MS=400:").replace("DEFERRED=2:", "DEFERRED=40:")
                    for line in good]
        observed = evaluate("m3-ipc-four", transcript(boundary, lockdep=False), 33, False)
        margins = [margin for margin in observed["deadline_margins"] if "cpu" in margin]
        self.assertEqual(len(margins), 8)
        self.assertEqual({margin["cpu"] for margin in margins}, {0, 1, 2, 3})
        self.assertTrue(all(margin["measured_over_deadline"] == 1 for margin in margins))

    def test_m3_phase_needs_ring3_work_and_switches_on_every_cpu(self):
        good = m3(4)
        cpu2 = good.index("SMP:M3:CPU=2:USER_TICKS=982:SWITCHES=1000")
        broken = {
            "idle CPU": good[:cpu2] + ["SMP:M3:CPU=2:USER_TICKS=0:SWITCHES=1000"] + good[cpu2 + 1:],
            "too few switches": good[:cpu2] + ["SMP:M3:CPU=2:USER_TICKS=982:SWITCHES=999"]
            + good[cpu2 + 1:],
            "no FPU evidence": [line for line in good if not line.startswith("SMP:M3:FPU:")],
            "canary corrupt": good[:cpu2] + ["SMP:M3:FPU:CORRUPT"] + good[cpu2:],
            "missing CPU": good[:cpu2] + good[cpu2 + 1:],
            "no cleanup balance": [line for line in good if line != "SMP:M3:CLEANUP:PASS"],
            "parked instead of waiting": [line.replace("SMP:M3:WAITING:APS=3", "SMP:PARKED:APS=3")
                                          for line in good],
            "unexpected fault": good[:good.index("SMP:M3:GO:CPUS=4") + 1]
            + ["SMP:M3:FAULT:UD:CPU=2"] + good[good.index("SMP:M3:GO:CPUS=4") + 1:],
        }
        for name, lines in broken.items():
            with self.subTest(name=name), self.assertRaises(ValueError):
                evaluate("m3-four", transcript(lines, lockdep=False), 33, False)
        fault = m3(4, fault=True)
        for name, lines in {
            "no K03": [line for line in fault if line != "SMP:M3:K03:PASS"],
            "fault on another CPU": [line.replace("FAULT:UD:CPU=2", "FAULT:UD:CPU=1")
                                     for line in fault],
            "no fault": m3(4)}.items():
            with self.subTest(name=name), self.assertRaises(ValueError):
                evaluate("m3-fault-ap2", transcript(lines, lockdep=False), 33, False)

    def test_every_case_accepts_its_exact_sequence(self):
        from test_smp_acceptance import ACCEPTANCE_CASES
        self.assertEqual(set(VALID) | ACCEPTANCE_CASES, set(CASES))
        for case, lines in VALID.items():
            with self.subTest(case=case):
                text = transcript(lines)
                if case == "serial-unlocked-control":
                    text = text.replace("SMP:SER:1:7\nSMP:SER:2:7\n", "SMP:SER:1:SMP:SER:2:7\n7\n")
                result = evaluate(case, text, 33, False)
                self.assertEqual(result["outcome"], CASES[case][3])
        self.assertEqual(evaluate("nine-capacity", transcript(online(9)), 33, False)["online"], 8)
        self.assertEqual(evaluate("ap-ipi-deaf", transcript(DEAF), 33, False)["online"], 1)
        self.assertEqual(evaluate("ap-stall", transcript(failed("TIMEOUT")), 33, False)["ap_ids"], [1])

    def test_rejects_missing_duplicate_or_overclaimed_output(self):
        four = online(4)
        # Mutate by marker, not index: the sequence grows with every K06 step.
        at = lambda prefix: next(i for i, line in enumerate(four) if line.startswith(prefix))
        swap = lambda prefix, new: four[:at(prefix)] + [new] + four[at(prefix) + 1:]
        drop = lambda prefix: four[:at(prefix)] + four[at(prefix) + 1:]
        bad = {
            "four": [
                four[:1] + four[2:],                      # one AP missing
                four[:2] + four[1:],                      # AP reported twice
                swap("SMP:ONLINE:", "SMP:ONLINE:5/4"),
                swap("SMP:SYNC:", "SMP:SYNC:PASS:CPUS=5"),
                four[:-2] + ["SMP:SYNC:PASS:CPUS=4"] + END,
                ["SMP:MADT:CPUS=3:DISABLED=0:HOTPLUG=0"] + four[1:],
                four[:1] + ["SMP:AP:1:TIMEOUT"] + four[2:],
                drop("SMP:SYNC:"),                       # sync result missing
                drop("SMP:IPI:"),                        # IPI result missing
                swap("SMP:IPI:", "SMP:IPI:PASS:ROUNDS=2:CPUS=4"),
                drop("SMP:PARKED:"),                     # park result missing
                swap("SMP:PARKED:", "SMP:PARKED:APS=2"),
                ["SMP:UNAVAILABLE:INVALID_MADT"] + END,
                [four[0], "SMP:AP:1:ONLINE", "SMP:AP:2:ONLINE", "SMP:AP:7:ONLINE"] + four[4:],
                [four[0], four[2], four[1], four[3]] + four[4:],  # unique but out of order
                DEAF,
            ],
            "nine-capacity": [online(8), online(9)[:4] + online(9)[5:]],
            "ap-stall": [failed("TIMEOUT")[:2] + failed("TIMEOUT")[3:], failed("MISMATCH"),
                         online(4)],
            "ap-mismatch": [failed("TIMEOUT"),
                            [failed("MISMATCH")[0], "SMP:AP:2:MISMATCH"] + failed("MISMATCH")[2:]],
            "ap-ipi-deaf": [online(4), DEAF[:-3] + END, DEAF[:-4] + DEAF[-3:],
                            started(4) + ["SMP:IPI:TIMEOUT:APIC=1", "SMP:QUARANTINED:APS=2",
                                          "SMP:ONLINE:1/4"] + END],
            # A terminal IST case must not report a normal finish instead of the marker,
            # and must not accept the other IST marker.
            "ap-nmi-ist": [online(4), NMI[:-1] + ["SMP:FAULT:AP_DF_IST:PASS"], NMI + END,
                           NMI[:-2] + NMI[-1:]],
            "ap-double-fault-ist": [online(4), DOUBLE[:-1] + ["SMP:FAULT:AP_NMI_IST:PASS"],
                                    DOUBLE[:-1]],
            "duplicate-madt": [online(4), END],
            # A normal run must prove the shootdown AND its visibility; a run
            # that skipped it (stale control output) must never count as normal.
            "eight": [[line for line in online(8) if "SHOOTDOWN:VISIBLE" not in line],
                      [line for line in online(8) if "SHOOTDOWN:PASS" not in line],
                      STALE_CONTROL],
            "ap-shootdown-deaf": [online(4), DEAF,
                                  [line for line in SHOOTDOWN_DEAF if "FRAME_QUARANTINED" not in line],
                                  SHOOTDOWN_DEAF[:-3] + ["SMP:SHOOTDOWN:PASS:CPUS=4"] + END],
            "shootdown-skipped-control": [
                online(4),  # the control must not silently become a passing shootdown
                STALE_CONTROL[:-6] + ["SMP:SHOOTDOWN:STALE:APS=2"] + STALE_CONTROL[-5:],
                STALE_CONTROL[:-6] + ["SMP:SHOOTDOWN:STALE:APS=0"] + STALE_CONTROL[-5:]],
            "ap-timer-dead": [online(4), TIMER_DEAD[:-4] + END,
                              [line for line in TIMER_DEAD if "QUARANTINED" not in line]],
        }
        for case, transcripts in bad.items():
            for lines in transcripts:
                with self.subTest(case=case, lines=lines), self.assertRaises(ValueError):
                    evaluate(case, transcript(lines), 33, False)

    def test_normal_runs_must_prove_preemption(self):
        # Separate from the table above: a second "four" key there would
        # silently replace the existing "four" rejection cases.
        for lines in ([line for line in online(4) if "PREEMPT" not in line], TIMER_DEAD,
                      [line.replace("PREEMPT:PASS:CPUS=4", "PREEMPT:PASS:CPUS=3")
                       for line in online(4)]):
            with self.subTest(lines=lines), self.assertRaises(ValueError):
                evaluate("four", transcript(lines), 33, False)

    def test_normal_runs_must_prove_the_spin_window(self):
        # A normal run must show the spin phase; the deaf control must never
        # pass as a normal run, report a PASS, or skip the quarantine.
        for case, lines in (
            ("four", [line for line in online(4) if "SMP:SPIN:" not in line]),
            ("four", SPIN_DEAF),
            ("four", [line.replace("SPIN:PASS:CPUS=4", "SPIN:PASS:CPUS=3") for line in online(4)]),
            ("spin-irq-off-control", online(4)),
            ("spin-irq-off-control", SPIN_DEAF[:-5] + ["SMP:SPIN:PASS:CPUS=4"] + END),
            ("spin-irq-off-control", [line for line in SPIN_DEAF if "QUARANTINED" not in line]),
            ("spin-irq-off-control", [line.replace("SPIN:SHOOTDOWN_TIMEOUT", "SHOOTDOWN:TIMEOUT")
                                      for line in SPIN_DEAF]),
        ):
            with self.subTest(case=case, lines=lines[-6:]), self.assertRaises(ValueError):
                evaluate(case, transcript(lines), 33, False)
        self.assertEqual(evaluate("spin-irq-off-control", transcript(SPIN_DEAF), 33, False)["online"], 1)

    def test_switch_with_a_held_lock_is_terminal_and_exact(self):
        # Step 5c: the no-deferral profile must stop at the marker, inside the
        # preemption phase; a PASS there, or the marker elsewhere, is rejected.
        for case, lines in (
            ("held-across-switch", online(2)),
            ("held-across-switch", HELD_ACROSS[:-1] + ["SMP:PREEMPT:PASS:CPUS=2"]),
            ("held-across-switch", HELD_ACROSS + ["SMP:PREEMPT:PASS:CPUS=2"]),
            ("held-across-switch", HELD_ACROSS[:-2] + HELD_ACROSS[-1:]),
            ("two-single", HELD_ACROSS),
            ("two-single", online(2)[:-4] + ["SMP:LOCKDEP:HELD_ACROSS_SWITCH"] + online(2)[-4:]),
        ):
            with self.subTest(case=case, lines=lines[-4:]), self.assertRaises(ValueError):
                evaluate(case, transcript(lines), 33, False)
        self.assertEqual(evaluate("held-across-switch", transcript(HELD_ACROSS), 33, False)["online"], 2)

    def test_scheduler_phase_is_required_and_double_run_is_terminal(self):
        # Step 5d-1: a normal run must show the scheduler phase for every CPU;
        # the double-run profile must stop at its marker right after SPIN.
        for case, lines in (
            ("four", [line for line in online(4) if "SMP:SCHED:" not in line]),
            ("four", [line.replace("SCHED:PASS:CPUS=4", "SCHED:PASS:CPUS=3") for line in online(4)]),
            ("four", DOUBLE_RUN),
            ("double-run", online(4)),
            ("double-run", DOUBLE_RUN[:-1] + ["SMP:SCHED:PASS:CPUS=4", "SMP:SCHED:DOUBLE_ENQUEUE"]),
            ("double-run", DOUBLE_RUN + ["SMP:PARKED:APS=3"]),
            ("double-run", DOUBLE_RUN[:-1] + ["SMP:SCHED:DOUBLE_RUN"]),  # timing, not the push
        ):
            with self.subTest(case=case, lines=lines[-3:]), self.assertRaises(ValueError):
                evaluate(case, transcript(lines), 33, False)
        self.assertEqual(evaluate("double-run", transcript(DOUBLE_RUN), 33, False)["online"], 4)

    def test_wake_and_kill_phases_are_required_and_the_deaf_kill_quarantines(self):
        # Audit F16/F17: every run with APs proves ping-pong and kill for all
        # CPUs; without APs both are reported skipped, never passed. The deaf
        # control must time out on AP3 and quarantine, never pass.
        for case, lines in (
            ("four", [line for line in online(4) if "SMP:WAKE:" not in line]),
            ("four", [line for line in online(4) if "SMP:KILL:" not in line]),
            ("four", [line.replace("WAKE:PASS:CPUS=4", "WAKE:PASS:CPUS=3") for line in online(4)]),
            ("four", [line.replace("KILL:PASS", "KILL:SKIPPED") for line in online(4)]),
            ("one", [line.replace("SKIPPED", "PASS") for line in online(1)]),
            ("four", RESCHED_DEAF),
            ("resched-deaf", online(4)),                                   # kill not deaf
            ("resched-deaf", [line for line in RESCHED_DEAF if "QUARANTINED" not in line]),
            ("resched-deaf", [line.replace("APIC=3", "APIC=1") for line in RESCHED_DEAF]),
            ("resched-deaf", online(4)[:-7] + ["SMP:KILL:PASS:CPUS=4"] + RESCHED_DEAF[-5:]),
            ("resched-deaf", SCHED_BUSY),                                  # fixture marker foreign
            ("quarantine-sched-busy", RESCHED_DEAF),                       # AP1 never held RUNQ
            ("quarantine-sched-busy", [line.replace("CPU=1", "CPU=2") for line in SCHED_BUSY]),
            # IQ-3 without the scheduler settle: AP1 is reset inside its RUNQ tick.
            ("quarantine-sched-busy", SCHED_BUSY[:-4] + ["SMP:QUARANTINE:LOCK_HELD:40:CPU=1",
                                                         "M1:FAIL:SMP_QUARANTINE_LOCK_HELD"]),
        ):
            with self.subTest(case=case, lines=lines[-6:]), self.assertRaises(ValueError):
                evaluate(case, transcript(lines), 33, False)
        self.assertEqual(evaluate("resched-deaf", transcript(RESCHED_DEAF), 33, False)["online"], 1)
        self.assertEqual(evaluate("quarantine-sched-busy", transcript(SCHED_BUSY), 33, False)["online"], 1)

    def test_gs_fixture_is_exact_and_its_markers_stay_in_their_profiles(self):
        # Audit F8: the red run (old stubs) must be rejected, FSGSBASE must be
        # clear, and no GS marker may appear in a profile that does not own it.
        red = online(4)[:-3] + ["SMP:GS:CR4=104", "SMP:GS:HIJACK", "M2:FAIL:KERNEL_EXCEPTION"]
        fsgsbase = str(104 | 1 << 16)
        for case, lines in (
            ("gs-user", red),
            ("gs-user", online(4)),
            ("gs-user", [line.replace("=104", "=" + fsgsbase) for line in GS_USER]),
            ("gs-user", [line for line in GS_USER if "CR4" not in line]),
            ("gs-user", GS_USER[:-4] + ["SMP:GS:ANCHOR_LOST"] + GS_USER[-3:]),
            ("gs-user", [line.replace("GS:PASS:CPUS=4", "GS:PASS:CPUS=3") for line in GS_USER]),
            ("gs-iret-fault", GS_USER),
            ("gs-iret-fault", GS_IRET[:-2] + GS_IRET[-1:]),
            ("gs-nmi-user", NMI),
            ("gs-nmi-user", GS_IRET),
            ("four", GS_USER),
            ("ap-nmi-ist", GS_NMI),
            ("double-run", GS_IRET),
        ):
            with self.subTest(case=case, lines=lines[-3:]), self.assertRaises(ValueError):
                evaluate(case, transcript(lines), 33, False)
        self.assertEqual(evaluate("gs-user", transcript(GS_USER), 33, False)["online"], 4)

    def test_scratch_phase_is_required_and_the_crossing_stays_in_its_control(self):
        # Step 5b-2: a normal run must prove the per-CPU slots for every CPU;
        # the shared-slot control must report a crossing and never pass.
        for cpu in range(4):
            lines = SHARED[:-1] + [f"SMP:SCRATCH:CROSS:CPU={cpu}"]
            self.assertEqual(evaluate("shared-scratch", transcript(lines), 33, False)["online"], 4)
        for case, lines in (
            ("four", [line for line in online(4) if "SMP:SCRATCH:" not in line]),
            ("four", [line.replace("SCRATCH:PASS:CPUS=4", "SCRATCH:PASS:CPUS=3")
                      for line in online(4)]),
            ("four", SHARED),
            ("four", online(4)[:-3] + ["SMP:SCRATCH:CROSS:CPU=1"] + online(4)[-3:]),
            ("shared-scratch", online(4)),                              # control passed
            ("shared-scratch", SHARED[:-1] + ["SMP:SCRATCH:CROSS:CPU=4"]),   # no such CPU
            ("shared-scratch", SHARED[:-1] + ["SMP:SCRATCH:CROSS:CPU="]),
            ("shared-scratch", SHARED[:-1] + ["SMP:SCRATCH:PASS:CPUS=4", SHARED[-1]]),
            ("shared-scratch", SHARED + SHARED[-1:]),                   # two reporters
            ("shared-scratch", SHARED + ["SMP:PARKED:APS=3"]),
            ("shared-scratch", SHARED[:-2] + SHARED[-1:]),              # before KILL:PASS
        ):
            with self.subTest(case=case, lines=lines[-3:]), self.assertRaises(ValueError):
                evaluate(case, transcript(lines), 33, False)

    def test_lock_pair_phase_is_required_and_its_controls_stay_terminal(self):
        # Audit F3/F4: every normal run proves lock_pair on all CPUs; both
        # controls must stop with their exact marker, in their own phase.
        for case, lines in (
            ("four", [line for line in online(4) if "SMP:LOCKPAIR:" not in line]),
            ("four", [line.replace("LOCKPAIR:PASS:CPUS=4", "LOCKPAIR:PASS:CPUS=3")
                      for line in online(4)]),
            ("lock-in-handler", online(2)),                              # not detected
            ("lock-in-handler", LOCK_IN_HANDLER[:-1] + ["SMP:LOCKDEP:IN_HANDLER:20"]),
            ("lock-in-handler", online(2)[:-6] + LOCK_IN_HANDLER[-1:]),  # after KILL:PASS
            ("lockpair-order", online(2)),                               # pair order missed
            ("lockpair-order", LOCKPAIR_ORDER[:-1] + ["SMP:LOCKDEP:ORDER:30->10"]),
            ("lockpair-order", LOCKPAIR_ORDER + END),
        ):
            with self.subTest(case=case, lines=lines[-3:]), self.assertRaises(ValueError):
                evaluate(case, transcript(lines), 33, False)

    def test_icr_exchange_is_required_and_its_controls_stay_terminal(self):
        # Audit F19/F15: every run with two APs proves the ICR exchange; the race
        # control must report the misrouted IPI, and a deaf AP holding a lock
        # must stop the system instead of being reset.
        for case, lines in (
            ("four", [line for line in online(4) if "SMP:ICR:" not in line]),
            ("four", [line.replace("ICR:PASS:ROUNDS=10000:CPUS=4", "ICR:SKIPPED:CPUS=4")
                      for line in online(4)]),
            ("four", [line.replace("ROUNDS=10000:CPUS=4", "ROUNDS=9999:CPUS=4") for line in online(4)]),
            ("four", ICR_RACE),
            ("two-single", [line.replace("ICR:SKIPPED:CPUS=2", "ICR:PASS:ROUNDS=10000:CPUS=2")
                            for line in online(2)]),
            ("icr-race-control", online(4)),                             # race not seen
            ("icr-race-control", ICR_RACE[:-1] + ["SMP:ICR:TIMEOUT"]),
            ("four", ICR_RACE[:-1] + ["SMP:ICR:LOST"]),
            ("icr-race-control", ICR_RACE + END),
            ("quarantine-lock-held", DEAF),                              # INIT despite KERNEL
            ("quarantine-lock-held", LOCK_HELD[:-1] + ["SMP:QUARANTINE:LOCK_HELD:40:CPU=2"]),
            ("quarantine-lock-held", LOCK_HELD[:-1] + ["SMP:QUARANTINE:LOCK_HELD:10:CPU=1"]),
            ("quarantine-lock-held", LOCK_HELD + ["SMP:QUARANTINED:APS=3", "SMP:ONLINE:1/4"] + END),
            ("ap-ipi-deaf", LOCK_HELD),
        ):
            with self.subTest(case=case, lines=lines[-3:]), self.assertRaises(ValueError):
                evaluate(case, transcript(lines), 33, False)

    def test_bsp_phases_are_required_and_the_bsp_window_control_stays_terminal(self):
        # Step 5c-2: every run proves the clock, a calibrated ratio inside the
        # ADR-080 band and BSP preemption; with an AP also the BSP as shootdown
        # target and the idle wake (latency below 100 ms). Without APs those two
        # are skipped, never passed. The shut-window control must time out on
        # the BSP (APIC 0) and stop there.
        four, one = online(4), online(1)
        mutate = lambda lines, old, new: [line.replace(old, new) for line in lines]
        for case, lines in (
            ("four", [line for line in four if "SMP:TIMER:RATIO:" not in line]),
            ("four", mutate(four, RATIO, "SMP:TIMER:RATIO:49999")),
            ("four", mutate(four, RATIO, "SMP:TIMER:RATIO:75001")),
            ("four", mutate(four, RATIO, "SMP:TIMER:RATIO:")),
            ("four", [line for line in four if "SMP:CLOCK:" not in line]),
            ("four", mutate(four, "SMP:CLOCK:PASS:CPUS=4", "SMP:CLOCK:DRIFT:US=520000")),
            ("four", [line for line in four if "SMP:BSP_PREEMPT:" not in line]),
            ("four", [line for line in four if "SMP:BSP_SPIN:" not in line]),
            ("four", mutate(four, "BSP_SPIN:PASS:CPUS=4", "BSP_SPIN:SKIPPED:CPUS=1")),
            ("four", [line for line in four if "SMP:IDLE:PASS" not in line]),
            ("four", [line for line in four if "SMP:IDLE:WAKE_US" not in line]),
            ("four", mutate(four, "WAKE_US=412", "WAKE_US=100000")),
            ("four", mutate(four, "WAKE_US=412", "WAKE_US=")),
            ("four", BSP_SPIN_DEAF),
            ("one", mutate(one, "SKIPPED:CPUS=1", "PASS:CPUS=1")),
            ("one", one[:one.index("SMP:IDLE:SKIPPED:CPUS=1")] + ["SMP:IDLE:WAKE_US=412"]
             + one[one.index("SMP:IDLE:SKIPPED:CPUS=1"):]),
            ("bsp-spin-irq-off-control", four),                       # window not shut
            ("bsp-spin-irq-off-control", mutate(BSP_SPIN_DEAF, "APIC=0", "APIC=1")),
            ("bsp-spin-irq-off-control", BSP_SPIN_DEAF[:-1] + ["SMP:BSP_SPIN:PASS:CPUS=4"]),
            ("bsp-spin-irq-off-control", BSP_SPIN_DEAF + END),
            ("bsp-spin-irq-off-control", BSP_SPIN_DEAF[:-2] + BSP_SPIN_DEAF[-1:]),
            ("spin-irq-off-control", BSP_SPIN_DEAF),
        ):
            with self.subTest(case=case, lines=lines[-3:]), self.assertRaises(ValueError):
                evaluate(case, transcript(lines), 33, False)
        for value in ("50000", "75000"):
            lines = mutate(four, RATIO, "SMP:TIMER:RATIO:" + value)
            self.assertEqual(evaluate("four", transcript(lines), 33, False)["online"], 4)
        self.assertEqual(evaluate("bsp-spin-irq-off-control", transcript(BSP_SPIN_DEAF), 33,
                                  False)["online"], 4)

    def test_icr_race_accepts_misrouted_or_lost(self):
        for marker in ("SMP:ICR:MISROUTED", "SMP:ICR:LOST"):
            lines = ICR_RACE[:-1] + [marker]
            self.assertEqual(evaluate("icr-race-control", transcript(lines), 33, False)["online"], 4)

    def test_lock_self_test_is_required_and_terminal_order_is_exact(self):
        for case, text in (
            ("four", transcript(online(4), lockdep=False)),           # self-test missing
            ("four", transcript(online(4)[:-2] + ["SMP:LOCKDEP:PASS"] * 2 + END, lockdep=False)),
            ("four", transcript(online(4)[:-3] + ["SMP:LOCKDEP:PASS"] + online(4)[-3:],
                                lockdep=False)),                      # before PARKED
            ("ap-ipi-deaf", transcript(DEAF, lockdep=False)),
            ("lockdep-order", transcript(online(2))),                  # order not detected
            ("lockdep-order", transcript(LOCKDEP_ORDER[:-1] + ["SMP:LOCKDEP:ORDER:10->30"])),
            ("lockdep-order", transcript(LOCKDEP_ORDER[:-1] + ["SMP:LOCKDEP:PASS",
                                                               LOCKDEP_ORDER[-1]])),
            ("lockdep-order", transcript(LOCKDEP_ORDER + END)),
        ):
            with self.subTest(case=case, text=text[-90:]), self.assertRaises(ValueError):
                evaluate(case, text, 33, False)
        self.assertEqual(evaluate("lockdep-order", transcript(LOCKDEP_ORDER), 33, False)["online"], 2)

    def test_rejects_fail_markers_exit_codes_and_changed_acpi_inventory(self):
        good = transcript(online(4))
        for text, raw, timeout in (
            (good, 35, False), (good, 33, True), (good, None, True),
            (good.replace("M2:PASS", "M1:FAIL:SMP_SYNC_TIMEOUT\nM2:PASS"), 33, False),
            (good.replace("ACPI:UNAVAILABLE:NO_DMAR", "ACPI:UNAVAILABLE:INVALID_TABLES"), 33, False),
            ("SMP:ONLINE:4/4\n" + good, 33, False),
        ):
            with self.subTest(text=text[-80:], raw=raw), self.assertRaises(ValueError):
                evaluate("four", text, raw, timeout)
        nmi = transcript(NMI)
        for text, raw in ((nmi, 35), (nmi + "M1:FAIL:SMP_AP_NMI_NOT_ON_IST\n", 33)):
            with self.subTest(raw=raw), self.assertRaises(ValueError):
                evaluate("ap-nmi-ist", text, raw, False)

    def test_serial_lines_are_whole_ordered_and_complete(self):
        """F7 (DG-2): one torn, missing, reordered or misplaced line fails the run."""
        good = transcript(online(4))
        line = "SMP:SER:2:7\n"
        self.assertEqual(good.count(line), 1)
        torn = good.replace("SMP:SER:1:7\n" + line, "SMP:SER:1:SMP:SER:2:7\n7\n")
        for text in (
            torn,                                                         # mixed bytes
            good.replace("SMP:SER:2:7\n", "SMP:SE\nR:2:7\n"),             # split line
            good.replace(line, ""),                                       # lost line
            good.replace(line, line * 2),                                 # repeated line
            good.replace("SMP:SER:2:8\n", "").replace("SMP:SER:2:7\n",
                                                        "SMP:SER:2:8\nSMP:SER:2:7\n"),
            good.replace("SMP:SERIAL:PASS\n", "").replace("SMP:ONLINE:4/4\n",
                                                          "SMP:ONLINE:4/4\nSMP:SERIAL:PASS\n"),
            transcript(online(4), serial=False),                          # fixture absent
            good.replace("SMP:SYNC:PASS", "SMP:SER:0:0\nSMP:SYNC:PASS"),    # after PASS
        ):
            with self.subTest(text=text[-60:]), self.assertRaises(ValueError):
                evaluate("four", text, 33, False)
        # Interleaving between whole lines is allowed (the order of CPUs is free).
        swapped = good.replace("SMP:SER:2:7\nSMP:SER:3:7\n", "SMP:SER:3:7\nSMP:SER:2:7\n")
        self.assertEqual(evaluate("four", swapped, 33, False)["online"], 4)
        # A stopped bring-up leaves only the BSP's lines.
        self.assertEqual(evaluate("ap-stall", transcript(failed("TIMEOUT")), 33, False)["online"], 1)

    def test_unlocked_control_must_show_a_mixed_line(self):
        """Spec §4 control: without SERIAL a run with only whole lines fails."""
        good = transcript(online(4))
        with self.assertRaises(ValueError):
            evaluate("serial-unlocked-control", good, 33, False)
        mixed = good.replace("SMP:SER:0:3\nSMP:SER:1:3\n", "SMSMP:SERP:SER:0:3\n:1:3\n")
        self.assertEqual(evaluate("serial-unlocked-control", mixed, 33, False)["online"], 4)
        # Outside the fixture phase the exact sequence still holds.
        with self.assertRaises(ValueError):
            evaluate("serial-unlocked-control", mixed.replace("SMP:SPIN:PASS:CPUS=4\n", ""), 33, False)
        # The locked profile rejects the same mixed transcript.
        with self.assertRaises(ValueError):
            evaluate("four", mixed, 33, False)

    def test_unlocked_line_may_only_be_the_terminal_marker(self):
        """DG-2: `!` marks a terminal line written without SERIAL."""
        stuck = transcript(LOCK_STUCK)
        bang = stuck.replace("SMP:LOCK:STUCK:10:CPU=1", "!SMP:LOCK:STUCK:10:CPU=1")
        self.assertEqual(evaluate("lock-stuck", bang, 33, False)["online"], 2)
        for case, text in (
            ("four", transcript(online(4)).replace("SMP:SYNC:PASS", "!SMP:SYNC:PASS")),
            ("four", transcript(online(4)).replace("M1:PASS", "!M1:PASS")),      # not terminal
            ("lock-stuck", bang + "!SMP:LOCK:STUCK:10:CPU=1\n"),
            ("lock-stuck", stuck.replace("SMP:IPI:PASS", "!SMP:IPI:PASS")),
        ):
            with self.subTest(case=case), self.assertRaises(ValueError):
                evaluate(case, text, 33, False)

    def test_deadline_margins_are_recorded_and_an_overrun_needs_an_oversubscribed_host(self):
        """AC-3: margin lines leave the sequence and enter the report; a wait above its
        limit passes only for multi-thread TCG with more vCPUs than host CPUs."""
        def with_margins(cpus, *margins):
            lines = online(cpus)
            return transcript(lines[:-2] + list(margins) + END)
        report = evaluate("four", with_margins(4, "SMP:MARGIN:WAIT:1360:12/2000",
                                               "SMP:MARGIN:STALL:1360:3/1000",
                                               "SMP:MARGIN:LOCK:10:6/400",
                                               "SMP:MARGIN:ROUND:0:1/100"), 33, False, host_cpus=4)
        self.assertEqual(report["guest_waits"], {"wait_1360": [12, 2000], "stall_1360": [3, 1000],
                                                 "lock_10": [6, 400], "round_0": [1, 100]})
        self.assertEqual([item["name"] for item in report["deadline_margins"]], ["bsp_idle_wake"])
        slow = "SMP:MARGIN:WAIT:1360:2600/2000"
        self.assertEqual(evaluate("eight", with_margins(8, slow), 33, False, host_cpus=4)["online"], 8)
        for case, cpus, host, margin in (
            ("eight", 8, 8, slow),                             # the host can run eight in parallel
            ("four", 4, 4, slow),
            ("two-single", 2, 1, slow),                        # single-thread TCG is never oversubscribed
            ("eight", 8, 4, "SMP:MARGIN:STALL:1360:1001/1000"),  # only a WAIT may overrun
            ("eight", 8, 4, "SMP:MARGIN:LOCK:10:401/400"),
            ("four", 4, 4, "SMP:MARGIN:WAIT:1360:1/0"),
            ("four", 4, 4, "SMP:MARGIN:OVERFLOW"),
            ("four", 4, 4, "SMP:MARGIN:WAIT:1360:1/2000:x"),
        ):
            with self.subTest(case=case, host=host, margin=margin), self.assertRaises(ValueError):
                evaluate(case, with_margins(cpus, margin), 33, False, host_cpus=host)
        with self.assertRaises(ValueError):                    # one key, one line
            evaluate("four", with_margins(4, "SMP:MARGIN:LOCK:10:6/400", "SMP:MARGIN:LOCK:10:7/400"),
                     33, False, host_cpus=4)

    def test_sync_and_clock_fixtures_stop_on_their_own_marker_only(self):
        """ADR-082: each injected defect ends its own profile with its exact marker and
        its fixture lines; the same marker fails every other profile."""
        for case, lines in (
            ("sync-stall", SYNC_SLOW), ("sync-slow", SYNC_STALL),           # stall and cap are distinct
            ("sync-stall", sync_stopped((0, 1, 2, 3), SYNC_STALL[-1])),       # AP1 must not have written
            ("sync-stall", sync_stopped((0, 2, 3), "SMP:SYNC:STALLED:DONE=3:TOTAL=80000")),
            ("sync-slow", sync_stopped((0, 2, 3), "SMP:SYNC:TIMEOUT:DONE=2:TOTAL=80000")),
            ("sync-slow", sync_stopped((0, 2, 3), "SMP:SYNC:TIMEOUT:DONE=2:TOTAL=59999")),
            ("sync-lost-update", sync_stopped((0, 2, 3), SYNC_LOST[-1])),
            ("sync-lost-update", online(4)),                                 # defect not detected
            ("four", SYNC_STALL), ("four", SYNC_LOST), ("four", CLOCK_MASKED), ("four", CLOCK_DIVISOR),
            ("clock-masked", CLOCK_DIVISOR), ("clock-divisor", CLOCK_MASKED),
            ("clock-masked", online(4)), ("clock-divisor", online(4)),
            ("clock-masked", BEFORE_CLOCK + [EDGES, EDGES, "SMP:CLOCK:DRIFT:KERNEL"]),
            ("clock-masked", BEFORE_CLOCK + [EDGES.replace("HOST=2", "HOST=313"), "SMP:CLOCK:DRIFT:KERNEL"]),
        ):
            with self.subTest(case=case, tail=lines[-1]), self.assertRaises(ValueError):
                evaluate(case, transcript(lines), 33, False)
        report = evaluate("clock-masked", transcript(CLOCK_MASKED), 33, False)
        self.assertEqual(report["clock_edges"], {"ticks": 5000, "lost": 312, "host": 2, "extra": 0,
                                                 "span_us": 3051, "median_us": 1062411,
                                                 "own_median_us": 1062008})
        four = online(4)
        at = four.index("SMP:CLOCK:PASS:CPUS=4")
        quiet = EDGES.replace("LOST=312:HOST=2", "LOST=3:HOST=3").replace("1062411", "1000450").replace("1062008", "999850")
        self.assertEqual(evaluate("four", transcript(four[:at] + [quiet] + four[at:]), 33, False)
                         ["clock_edges"]["host"], 3)

    def test_stuck_lock_is_reported_in_its_profile_only(self):
        """F5 (DG-1): the stuck marker ends its own profile and fails all others."""
        for case, text in (
            ("lock-stuck", transcript(started(2) + ["SMP:IPI:PASS:ROUNDS=3:CPUS=2"])),
            ("lock-stuck", transcript(LOCK_STUCK[:-1] + ["SMP:LOCK:STUCK:70:CPU=1"])),
            ("lock-stuck", transcript(LOCK_STUCK[:-1] + ["SMP:LOCK:STUCK:10:CPU=0"])),
            ("four", transcript(started(4) + ["SMP:IPI:PASS:ROUNDS=3:CPUS=4",
                                              "SMP:LOCK:STUCK:10:CPU=1"])),
        ):
            with self.subTest(case=case, text=text[-40:]), self.assertRaises(ValueError):
                evaluate(case, text, 33, False)

    def test_duplicate_madt_fixture_is_a_valid_checksummed_table(self):
        table = duplicate_madt()
        self.assertEqual(table[:4], b"APIC")
        self.assertEqual(struct.unpack_from("<I", table, 4)[0], len(table))
        self.assertEqual(sum(table) % 256, 0)
        self.assertEqual(table[44:52], bytes([0, 8, 0, 0, 1, 0, 0, 0]))


    def test_process_shootdown_is_required_and_its_controls_are_exact(self):
        # Step 5e: a normal run must prove isolation and the unmap race; the
        # deaf AP must quarantine the frame; the control must show the stale read.
        for case, lines in (
            ("four", [line for line in online(4) if "SMP:UNMAP_RACE:" not in line]),
            ("four", [line for line in online(4) if "SMP:ISOLATION:" not in line]),
            ("four", [line.replace("UNMAP_RACE:PASS", "UNMAP_RACE:SKIPPED") for line in online(4)]),
            ("one", [line.replace("SKIPPED", "PASS") for line in online(1)]),
            ("four", online(4)[:-3] + ["SMP:SHOOTDOWN:STALE_READ"] + online(4)[-3:]),
            ("proc-shootdown-deaf", online(4)),
            ("proc-shootdown-deaf", [line for line in PROC_DEAF if "FRAME_QUARANTINED" not in line]),
            ("proc-shootdown-deaf", [line for line in PROC_DEAF if "PROC_DEAF:LOCK_FREE" not in line]),
            ("proc-shootdown-deaf", [line.replace("PROC_DEAF:LOCK_FREE:CPUS=4", "PROC_DEAF:LOCK_FREE:CPUS=3") for line in PROC_DEAF]),
            ("proc-no-shootdown-control", online(4)),  # the control must not pass silently
            ("proc-no-shootdown-control", PROC_STALE[:-1] + ["SMP:UNMAP_RACE:PASS:CPUS=4"] + END),
        ):
            with self.subTest(case=case, lines=lines[-4:]), self.assertRaises(ValueError):
                evaluate(case, transcript(lines), 33, False)


class SmpRepetitionHarness(unittest.TestCase):
    """Host-only runner tests; injected guest responses are not native evidence."""

    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.build = Path(temporary.name)
        (self.build / "manifest.json").write_text(json.dumps({
            "kernel_feature": "native-smp", "loader_feature": None,
            "source_fingerprint_version": 3, "source_sha256": "frozen",
            "kernel": {"sha256": "kernel"}, "loader": {"sha256": "loader"},
        }), encoding="utf-8")
        for name, value in (("BUILD", self.build), ("source_hash", lambda: "frozen")):
            replacement = patch.object(run_smp, name, value)
            replacement.start()
            self.addCleanup(replacement.stop)
        guest = patch.object(run_smp, "run_guest", side_effect=self.guest)
        self.runner = guest.start()
        self.addCleanup(guest.stop)

    def guest(self, label, **kwargs):
        return {"stdout": transcript(online(kwargs["cpus"])), "raw_qemu_exit": 33,
                "timed_out": False, "passed": True, "artifact_binding_passed": True,
                "source_sha256": "frozen", "duration_s": 1.25,
                "timeout_s": kwargs["timeout"], "run_directory": label}

    def test_fifty_runs_are_required_for_each_selected_case(self):
        report = run_smp.run(("one", "two-single"), repeat=50)
        self.assertTrue(report["passed"])
        self.assertEqual(len(report["cases"]), 100)
        self.assertEqual(len({row["guest"]["run_directory"] for row in report["cases"]}), 100)
        for case, kind in (("one", "single_cpu_regression"), ("two-single", "smp")):
            self.assertEqual(report["repetitions"][case], {
                "requested": 50, "attempted": 50, "passed": 50, "failed": 0,
                "complete": True, "flaky": False, "evidence_kind": kind,
            })
            self.assertEqual([row["iteration"] for row in report["cases"] if row["case"] == case],
                             list(range(1, 51)))
        timing = report["cases"][0]["host_watchdog"]
        self.assertEqual(timing["deadline_s"], 60)
        self.assertEqual(timing["remaining_s"], 58.75)
        self.assertAlmostEqual(timing["measured_over_deadline"], 1.25 / 60)
        for margin in report["cases"][0]["observed"]["deadline_margins"]:
            self.assertEqual(margin["clock"], "guest")
            self.assertEqual(margin["deadline_us"], 100_000)
        self.assertEqual(json.loads((self.build / "smp-report.json").read_text()), report)
        # The transcript lives in result.json/serial.txt; the report binds it by hash only.
        guest = report["cases"][0]["guest"]
        self.assertNotIn("stdout", guest)
        expected = transcript(online(1)).encode("utf-8")
        self.assertEqual(guest["stdout_sha256"], hashlib.sha256(expected).hexdigest())

    def test_one_bad_iteration_is_retained_and_cannot_be_outvoted(self):
        def intermittent(label, **kwargs):
            guest = self.guest(label, **kwargs)
            if self.runner.call_count == 2:
                guest.update(raw_qemu_exit=35, passed=False)
            return guest
        self.runner.side_effect = intermittent
        report = run_smp.run(("one",), repeat=3)
        self.assertFalse(report["passed"])
        summary = report["repetitions"]["one"]
        self.assertEqual((summary["attempted"], summary["passed"], summary["failed"]), (3, 2, 1))
        self.assertTrue(summary["complete"])
        self.assertTrue(summary["flaky"])
        self.assertIn("error", report["cases"][1])
        self.assertFalse(report["cases"][1]["passed"])

    def test_harness_accepts_only_the_dg2_terminal_fallback(self):
        # Review finding: the exact-marker harness rejected `!SMP:LOCK:STUCK...`,
        # which evaluate() accepts; artifact binding and other `!` lines still fail.
        manifest = json.loads((self.build / "manifest.json").read_text())
        manifest["kernel_feature"] = "test-smp-lock-stuck"
        (self.build / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
        bang = transcript(LOCK_STUCK).replace("SMP:LOCK:STUCK", "!SMP:LOCK:STUCK")
        for binding, stdout, accepted in ((True, bang, True), (False, bang, False),
                                          (True, transcript(LOCK_STUCK).replace(
                                              "SMP:IPI:PASS", "!SMP:IPI:PASS"), False)):
            self.runner.side_effect = lambda label, **kwargs: dict(
                self.guest(label, **kwargs), stdout=stdout, passed=False,
                artifact_binding_passed=binding)
            if accepted:
                self.assertTrue(run_smp.run(("lock-stuck",))["passed"])
            else:
                with self.assertRaises(ValueError):
                    run_smp.run(("lock-stuck",))

    def test_single_run_still_raises_on_a_rejected_guest(self):
        self.runner.return_value = self.guest("one", cpus=1, timeout=60)
        self.runner.return_value["passed"] = False
        self.runner.side_effect = None
        with self.assertRaisesRegex(ValueError, "guest harness rejected"):
            run_smp.run(("one",))
        report = json.loads((self.build / "smp-report.json").read_text())
        self.assertFalse(report["passed"])
        self.assertEqual(report["repetitions"]["one"]["failed"], 1)

    def test_only_the_ipc_outcome_gets_the_ipc_watchdog(self):
        # ADR-081 R3-5 addendum 2: 10 000 rounds keep their count; only m3-ipc gets 660 s.
        manifest = json.loads((self.build / "manifest.json").read_text())
        manifest["kernel_feature"] = "native-smp-m3-ipc"
        (self.build / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
        seen = []
        def record(label, **kwargs):
            seen.append(kwargs["timeout"])
            return dict(self.guest(label, **kwargs), passed=False)
        self.runner.side_effect = record
        with self.assertRaises(ValueError):
            run_smp.run(("m3-ipc-four",))
        self.assertEqual(seen, [run_smp.IPC_TIMEOUT_S])
        self.assertEqual((run_smp.IPC_TIMEOUT_S, run_smp.GUEST_TIMEOUT_S), (660, 60))

    def test_source_drift_aborts_before_more_repetitions(self):
        with patch.object(run_smp, "source_hash", side_effect=["frozen", "frozen", "changed"]):
            with self.assertRaisesRegex(ValueError, "changed during a run"):
                run_smp.run(("one",), repeat=50)
        report = json.loads((self.build / "smp-report.json").read_text())
        self.assertEqual(self.runner.call_count, 1)
        self.assertFalse(report["passed"])
        self.assertFalse(report["repetitions"]["one"]["complete"])
        self.assertEqual(report["repetitions"]["one"]["failed"], 1)

    def test_interruption_preserves_partial_evidence(self):
        def interrupted(label, **kwargs):
            if self.runner.call_count == 2:
                raise KeyboardInterrupt()
            return self.guest(label, **kwargs)
        self.runner.side_effect = interrupted
        with self.assertRaises(KeyboardInterrupt):
            run_smp.run(("one",), repeat=2)
        report = json.loads((self.build / "smp-report.json").read_text())
        summary = report["repetitions"]["one"]
        self.assertEqual((summary["attempted"], summary["passed"], summary["failed"]), (2, 1, 1))
        self.assertFalse(report["passed"])
        self.assertFalse(summary["complete"])
        self.assertFalse(summary["flaky"])
        self.assertEqual(report["cases"][1]["error"], "KeyboardInterrupt")

    def test_invalid_binding_or_watchdog_measurement_is_not_retried(self):
        for changed in ({"artifact_binding_passed": False}, {"duration_s": float("nan")},
                        {"duration_s": -1}, {"timeout_s": 61}):
            with self.subTest(changed=changed):
                self.runner.reset_mock()
                self.runner.side_effect = None
                self.runner.return_value = self.guest("one", cpus=1, timeout=60) | changed
                with self.assertRaises(ValueError):
                    run_smp.run(("one",), repeat=50)
                report = json.loads((self.build / "smp-report.json").read_text())
                self.assertEqual(self.runner.call_count, 1)
                self.assertFalse(report["passed"])
                self.assertFalse(report["repetitions"]["one"]["complete"])

    def test_invalid_selection_does_not_erase_previous_evidence(self):
        destination = self.build / "smp-report.json"
        destination.write_text("previous evidence", encoding="utf-8")
        for repeat in (0, -1, 1.5, True, "50"):
            with self.subTest(repeat=repeat), self.assertRaises(ValueError):
                run_smp.run(("one",), repeat=repeat)
        for cases in ((), ("one", "one"), ("missing",)):
            with self.subTest(cases=cases), self.assertRaises(ValueError):
                run_smp.run(cases, repeat=50)
        self.assertEqual(destination.read_text(), "previous evidence")
        self.runner.assert_not_called()


if __name__ == "__main__":
    unittest.main()

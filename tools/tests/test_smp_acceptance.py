"""Independent L0 acceptance-oracle tests; transcripts are synthetic host data."""
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
sys.path.insert(0, str(Path(__file__).resolve().parent))
import run_smp
from test_smp_serial import online, transcript


LOAD_CASES = {
    "m3-load-one": (1, "single"),
    "m3-load-one-multi": (1, "multi"),
    "m3-load-two-single": (2, "single"),
    "m3-load-two": (2, "multi"),
    "m3-load-four-single": (4, "single"),
    "m3-load-four": (4, "multi"),
    "m3-load-eight-single": (8, "single"),
    "m3-load-eight": (8, "multi"),
}
ACCEPTANCE_CASES = frozenset(LOAD_CASES) | {
    "m3-soak-four", "m4-quotas-four", "qsim-four", "m3-kill-four",
    "m3-unmap-four", "m3-unmap-control",
}


def prelude(cpus, manifest=()):
    # Reuse only the independently hand-written, existing bring-up transcript.
    # No expected_smp/m3_end/parser constants construct acceptance observations.
    return (online(cpus)[:-3]
            + [f"SMP:M3:WAITING:APS={cpus - 1}", "SMP:LOCKDEP:PASS", "M2:PASS"]
            + list(manifest) + [f"SMP:M3:GO:CPUS={cpus}"])


def fairness(cpus):
    return [f"SMP:FAIR:CPU={cpu}:WAIT_MS=1:DEFERRED=2:LIMIT_MS=400:LIMIT_TICKS=40"
            for cpu in range(cpus)]


def cpu_work(cpus, switches=1000):
    return [f"SMP:M3:CPU={cpu}:USER_TICKS={980 + cpu}:SWITCHES={switches}"
            for cpu in range(cpus)]


def load(cpus, elapsed=30000):
    return (prelude(cpus) + [f"SMP:LOAD:WORKERS=16:ELAPSED_MS={elapsed}"]
            + fairness(cpus)
            + ["SMP:IPC:PASS", f"SMP:IPC:REMOTE_WAKEUPS={17 if cpus > 1 else 0}",
               f"SMP:IPC:RESCHEDULE_IPIS={13 if cpus > 1 else 0}"]
            + cpu_work(cpus)
            + [f"SMP:SCHED:CPUS={cpus}", "SMP:M3:FPU:PASS:PROCESSES=18",
               "SMP:M3:CLEANUP:PASS", f"SMP:M3:PASS:CPUS={cpus}", "M1:PASS"])


def quotas():
    return (prelude(4) + fairness(4) + cpu_work(4, switches=1)
            + ["M4:CASE:QUOTAS:PASS", "M4:SURVIVOR_PROGRESS:PASS",
               "SMP:K07:M4:PASS:CPUS=4", "M4:CLEANUP:PASS", "KS-15:PASS",
               "M4:PASS", "M1:PASS"])


def qsim():
    lines = prelude(4, ["QSIM:MANIFEST:PASS"]) + fairness(4) + cpu_work(4, switches=1)
    for pid in range(1, 5):
        lines += [f"QSIM:PROCESS_PID:{pid:020d}", "QSIM:PROCESS_CHECKPOINTS:00000000000000000100",
                  "QSIM:PROCESS_USEFUL_ARENA:00000000000000007216",
                  "QSIM:PROCESS_MAX_WORK_GAP:00000000000000000448"]
    return lines + ["QSIM:BELL4096:PASS", "QSIM:SIX_QUBIT:PASS", "QSIM:CANCEL:PASS",
                    "QSIM:DEADLINE:PASS", "QSIM:RESOURCE_LIMITS:PASS", "QSIM:PROCESS_ISOLATION:PASS",
                    "QSIM:TIMER_IRQS:00000000000000000100", "QSIM:SYSCALL_CALLS:00000000000000001000",
                    "QSIM:CLEANUP_FRAMES:00000000000000000200", "SMP:K07:QSIM:PASS:CPUS=4",
                    "QSIM:CLEANUP:PASS", "QSIM:PASS", "M1:PASS"]


def unmap(control=False):
    # Independently spell the required observations: a real target mapping,
    # rejection by guarded usercopy, acknowledgement before reuse, then #PF.
    lines = prelude(4) + ["SMP:M3:ISOLATION:PASS", "SMP:M3:USERCOPY:UNMAP:PASS"]
    if not control:
        lines += ["SMP:M3:UNMAP:SHOOTDOWN:PASS"]
    lines += ["SMP:M3:UNMAP:REUSED:PASS"]
    lines += ["SMP:M3:UNMAP:STALE:CONTROL" if control else "SMP:M3:UNMAP:FAULT:CPU=1"]
    lines += ["SMP:M3:UNMAP:CLEANUP:PASS"]
    if not control:
        lines += ["SMP:M3:UNMAP:PASS"]
    return lines + ["M1:PASS"]


def remote_kill(vector=208):
    return (prelude(4)
            + ["SMP:M3:REMOTE_KILL:REQUEST:FROM=0:TO=3",
               f"SMP:M3:REMOTE_KILL:ACK:CPU=3:VECTOR={vector}:WAIT_MS=1:LIMIT_MS=100",
               "SMP:M3:REMOTE_KILL:CLEANUP:BEFORE=100:RELEASED=20:AFTER=120:ACTIVE=0",
               "SMP:M3:REMOTE_KILL:PASS:CPU=3"]
            + cpu_work(4)
            + ["SMP:SCHED:CPUS=4", "SMP:M3:FPU:PASS:PROCESSES=15",
               "SMP:M3:CLEANUP:PASS", "SMP:M3:PASS:CPUS=4", "M1:PASS"])


class SmpAcceptanceOracle(unittest.TestCase):
    def check(self, case, lines):
        return run_smp.evaluate(case, transcript(lines, lockdep=False), 33, False)

    def reject_mutations(self, case, good, mutations):
        self.check(case, good)  # Each negative control starts from accepted evidence.
        for description, lines in mutations.items():
            with self.subTest(case=case, mutation=description), self.assertRaises(ValueError):
                self.check(case, lines)

    def test_remote_lifecycle_profiles_are_bound_to_four_cpu_execution(self):
        for case, profile, outcome in (
            ("m3-unmap-four", "native-smp-m3-unmap", "m3-unmap"),
            ("m3-unmap-control", "test-smp-m3-no-shootdown", "m3-unmap-control"),
            ("m3-kill-four", "native-smp-m3-kill", "m3-kill"),
        ):
            with self.subTest(case=case):
                self.assertEqual(run_smp.CASES[case], (profile, 4, "multi", outcome))

    def test_real_unmap_requires_ordered_reuse_fault_cleanup_and_negative_control(self):
        positive, control = unmap(), unmap(control=True)
        for case, good in (("m3-unmap-four", positive), ("m3-unmap-control", control)):
            evidence = [x for x in good if x.startswith(("SMP:M3:ISOLATION:", "SMP:M3:USERCOPY:", "SMP:M3:UNMAP:"))]
            mutations = {f"omitted {marker}": [x for x in good if x != marker] for marker in evidence}
            mutations.update({
                "duplicate reuse": good[:-1] + ["SMP:M3:UNMAP:REUSED:PASS", good[-1]],
                "wrong remote CPU": [x.replace("UNMAP:FAULT:CPU=1", "UNMAP:FAULT:CPU=0") for x in good]
                    if case == "m3-unmap-four" else good + ["SMP:M3:UNMAP:FAULT:CPU=1"],
                "other profile evidence": control if case == "m3-unmap-four" else positive,
                "false one CPU binding": [x.replace("SMP:M3:GO:CPUS=4", "SMP:M3:GO:CPUS=1") for x in good],
            })
            self.reject_mutations(case, good, mutations)
        self.reject_mutations("m3-unmap-four", positive, {
            "reuse before acknowledgement": [
                "SMP:M3:UNMAP:REUSED:PASS" if x == "SMP:M3:UNMAP:SHOOTDOWN:PASS" else
                "SMP:M3:UNMAP:SHOOTDOWN:PASS" if x == "SMP:M3:UNMAP:REUSED:PASS" else x
                for x in positive],
            "stale read instead of fault": [x.replace("UNMAP:FAULT:CPU=1", "UNMAP:STALE:CONTROL") for x in positive],
            "stale read in successful trace": positive[:-1] + ["SMP:M3:UNMAP:STALE:CONTROL", "M1:PASS"],
        })

    def test_remote_kill_requires_bounded_ack_inactive_target_and_balanced_reclamation(self):
        good = remote_kill()
        self.check("m3-kill-four", remote_kill(vector=232))
        self.reject_mutations("m3-kill-four", good, {
            "missing request": [x for x in good if not x.startswith("SMP:M3:REMOTE_KILL:REQUEST:")],
            "missing acknowledgement": [x for x in good if not x.startswith("SMP:M3:REMOTE_KILL:ACK:")],
            "wrong remote CPU": [x.replace("ACK:CPU=3:", "ACK:CPU=2:") for x in good],
            "wrong request destination": [x.replace("REQUEST:FROM=0:TO=3", "REQUEST:FROM=0:TO=2") for x in good],
            "unknown vector": [x.replace("VECTOR=208:", "VECTOR=32:") for x in good],
            "deadline equality": [x.replace("WAIT_MS=1:", "WAIT_MS=100:") for x in good],
            "deadline overflow": [x.replace("WAIT_MS=1:", "WAIT_MS=18446744073709551616:") for x in good],
            "widened deadline": [x.replace("LIMIT_MS=100", "LIMIT_MS=101") for x in good],
            "still active": [x.replace("AFTER=120:ACTIVE=0", "AFTER=120:ACTIVE=8") for x in good],
            "unbalanced free count": [x.replace("AFTER=120:", "AFTER=119:") for x in good],
            "nothing reclaimed": [x.replace("RELEASED=20:AFTER=120:", "RELEASED=0:AFTER=100:") for x in good],
            "overflow free count": [x.replace("BEFORE=100:RELEASED=20:AFTER=120:",
                                             "BEFORE=18446744073709551615:RELEASED=1:AFTER=18446744073709551616:") for x in good],
            "missing target cleanup": [x for x in good if not x.startswith("SMP:M3:REMOTE_KILL:CLEANUP:")],
            "missing target pass": [x for x in good if x != "SMP:M3:REMOTE_KILL:PASS:CPU=3"],
            "killed task counted live": [x.replace("FPU:PASS:PROCESSES=15", "FPU:PASS:PROCESSES=16") for x in good],
            "missing remote execution": [x for x in good if not x.startswith("SMP:M3:CPU=3:")],
            "no final exact cleanup": [x for x in good if x != "SMP:M3:CLEANUP:PASS"],
        })

    def test_load_matrix_requires_each_cpu_count_and_both_tcg_modes(self):
        actual = {case: (cpus, tcg) for case, (_, cpus, tcg, outcome) in run_smp.CASES.items()
                  if outcome == "m3-load"}
        self.assertEqual(actual, LOAD_CASES)
        for case, (cpus, _) in LOAD_CASES.items():
            with self.subTest(case=case):
                observed = self.check(case, load(cpus))
                self.assertEqual(observed["online"], cpus)
                self.assertEqual(observed["outcome"], "m3-load")
        self.assertEqual(run_smp.CASES["m3-soak-four"][1:], (4, "multi", "m3-soak"))
        self.check("m3-soak-four", load(4, elapsed=600000))

    def test_duration_worker_count_and_progress_are_measured(self):
        for case, minimum in (("m3-load-eight", 30000), ("m3-soak-four", 600000)):
            cpus = 8 if case == "m3-load-eight" else 4
            good = load(cpus, elapsed=minimum)
            mutations = {
                "short workload": [x.replace(f"ELAPSED_MS={minimum}", f"ELAPSED_MS={minimum - 1}")
                                   for x in good],
                "zero duration": [x.replace(f"ELAPSED_MS={minimum}", "ELAPSED_MS=0") for x in good],
                "insufficient busy workers": [x.replace("WORKERS=16:", f"WORKERS={2 * cpus - 1}:")
                                              for x in good],
                "missing duration": [x for x in good if not x.startswith("SMP:LOAD:")],
                "duplicate duration": good + [f"SMP:LOAD:WORKERS=16:ELAPSED_MS={minimum}"],
                "overflow duration": [x.replace(f"ELAPSED_MS={minimum}", "ELAPSED_MS=18446744073709551616")
                                      for x in good],
                "overflow workers": [x.replace("WORKERS=16:", "WORKERS=18446744073709551616:")
                                     for x in good],
                "missing CPU": [x for x in good if not x.startswith("SMP:M3:CPU=1:")],
                "idle CPU": [x.replace("CPU=1:USER_TICKS=981:", "CPU=1:USER_TICKS=0:") for x in good],
                "insufficient switching": [x.replace("SWITCHES=1000", "SWITCHES=999") for x in good],
                "missing IPC": [x for x in good if x != "SMP:IPC:PASS"],
                "no remote wake": [x.replace("REMOTE_WAKEUPS=17", "REMOTE_WAKEUPS=0") for x in good],
                "no reschedule receipt": [x.replace("RESCHEDULE_IPIS=13", "RESCHEDULE_IPIS=0") for x in good],
                "no exact cleanup": [x for x in good if x != "SMP:M3:CLEANUP:PASS"],
                "omitted process canaries": [x.replace("FPU:PASS:PROCESSES=18", "FPU:PASS:PROCESSES=16")
                                            for x in good],
                "fairness deadline widened": [x.replace("LIMIT_MS=400:", "LIMIT_MS=401:") for x in good],
            }
            self.reject_mutations(case, good, mutations)

    def test_single_cpu_load_cannot_be_presented_as_smp(self):
        single = load(1)
        self.check("m3-load-one", single)
        for case in ("m3-load-two", "m3-load-four", "m3-load-eight", "m3-soak-four"):
            with self.subTest(case=case), self.assertRaises(ValueError):
                self.check(case, single)

    def test_quota_outcome_survivor_and_exact_cleanup_are_all_required(self):
        good = quotas()
        self.reject_mutations("m4-quotas-four", good, {
            "different successful M4 case": [x.replace("CASE:QUOTAS", "CASE:COPY") for x in good],
            "quota omitted": [x for x in good if x != "M4:CASE:QUOTAS:PASS"],
            "survivor omitted": [x for x in good if x != "M4:SURVIVOR_PROGRESS:PASS"],
            "cleanup omitted": [x for x in good if x != "M4:CLEANUP:PASS"],
            "missing SMP binding": [x for x in good if x != "SMP:K07:M4:PASS:CPUS=4"],
            "one CPU claimed": [x.replace("SMP:K07:M4:PASS:CPUS=4", "SMP:K07:M4:PASS:CPUS=1") for x in good],
            "wrong K07 workload": [x.replace("SMP:K07:M4:", "SMP:K07:QSIM:") for x in good],
            "one CPU did not execute": [x.replace("CPU=2:USER_TICKS=982:", "CPU=2:USER_TICKS=0:") for x in good],
            "no initial dispatch": [x.replace("SWITCHES=1", "SWITCHES=0") for x in good],
            "missing CPU witness": [x for x in good if not x.startswith("SMP:M3:CPU=3:")],
            "missing security check": [x for x in good if x != "KS-15:PASS"],
        })

    def test_qsim_requires_four_distinct_completed_bounded_processes(self):
        good = qsim()
        self.reject_mutations("qsim-four", good, {
            "duplicate process identity": [x.replace("QSIM:PROCESS_PID:00000000000000000004",
                                                     "QSIM:PROCESS_PID:00000000000000000003") for x in good],
            "no checkpoint progress": [x.replace("QSIM:PROCESS_CHECKPOINTS:00000000000000000100",
                                                 "QSIM:PROCESS_CHECKPOINTS:00000000000000000000") for x in good],
            "arena changed": [x.replace("ARENA:00000000000000007216", "ARENA:00000000000000007217") for x in good],
            "unbounded work gap": [x.replace("GAP:00000000000000000448", "GAP:00000000000000000449") for x in good],
            "resource limit omitted": [x for x in good if x != "QSIM:RESOURCE_LIMITS:PASS"],
            "cancellation omitted": [x for x in good if x != "QSIM:CANCEL:PASS"],
            "deadline omitted": [x for x in good if x != "QSIM:DEADLINE:PASS"],
            "isolation omitted": [x for x in good if x != "QSIM:PROCESS_ISOLATION:PASS"],
            "no actual reclaimed frames": [x.replace("FRAMES:00000000000000000200", "FRAMES:00000000000000000000")
                                            for x in good],
            "cleanup omitted": [x for x in good if x != "QSIM:CLEANUP:PASS"],
            "no SMP binding": [x for x in good if x != "SMP:K07:QSIM:PASS:CPUS=4"],
            "wrong CPU count": [x.replace("SMP:K07:QSIM:PASS:CPUS=4", "SMP:K07:QSIM:PASS:CPUS=2") for x in good],
            "missing CPU execution": [x for x in good if not x.startswith("SMP:M3:CPU=3:")],
        })

    def test_one_cpu_repetition_summary_remains_regression_evidence(self):
        with tempfile.TemporaryDirectory() as folder:
            build = Path(folder)
            (build / "manifest.json").write_text(json.dumps({
                "kernel_feature": "native-smp-m3-load", "loader_feature": None,
                "source_fingerprint_version": 3, "source_sha256": "frozen",
                "kernel": {"sha256": "kernel"}, "loader": {"sha256": "loader"},
            }), encoding="utf-8")

            def guest(label, **kwargs):
                return {"stdout": transcript(load(1), lockdep=False), "raw_qemu_exit": 33,
                        "timed_out": False, "passed": True, "artifact_binding_passed": True,
                        "source_sha256": "frozen", "duration_s": 31.5,
                        "timeout_s": kwargs["timeout"], "run_directory": label}

            with patch.object(run_smp, "BUILD", build), \
                    patch.object(run_smp, "source_hash", return_value="frozen"), \
                    patch.object(run_smp, "run_guest", side_effect=guest):
                report = run_smp.run(("m3-load-one",), repeat=50)
            self.assertTrue(report["passed"])
            self.assertEqual(report["repetitions"]["m3-load-one"]["evidence_kind"], "single_cpu_regression")
            self.assertEqual(report["repetitions"]["m3-load-one"]["passed"], 50)


if __name__ == "__main__":
    unittest.main()

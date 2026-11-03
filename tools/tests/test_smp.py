"""Host-side counterexamples for SMP QEMU evidence validation."""
import unittest

import run_smp


def transcript(cpus=2):
    lines = list(run_smp.BOOT)
    lines.extend("SMP:AP:ENTRY" for _ in range(cpus - 1))
    lines.extend(f"SMP:CPU:{index}:APIC:{index}:ONLINE" for index in range(cpus))
    lines.extend((
        f"SMP:CPUS:{cpus}",
        "SMP:RENDEZVOUS:PASS",
        "SMP:WORKERS:PASS",
        "SMP:PASS",
    ))
    return "\n".join(lines) + "\n"


def m3_transcript(cpus=2):
    lines = transcript(cpus).splitlines()
    lines.extend(
        marker
        for index in range(cpus)
        for marker in (
            f"M3:SMP:CPU:{index:020d}",
            f"M3:SMP:SCHEDULES:{(index + 1) * 7:020d}",
        )
    )
    lines.append("M3:SMP:SCHEDULER:PASS")
    return "\n".join(lines) + "\n"


def m4_smp_transcript(cpus=2, send_cpus=(0, 1), receive_cpus=(1, 0)):
    lines = m3_transcript(cpus).splitlines()
    events = (
        ("SEND", 0, send_cpus[0]),
        ("RECV", 1, receive_cpus[0]),
        ("SEND", 1, send_cpus[1]),
        ("RECV", 2, receive_cpus[1]),
    )
    lines.extend(
        f"M4:SMP:{operation}:{((slot << 8) | cpu):020d}"
        for operation, slot, cpu in events
    )
    lines.extend((
        "M4:EXCHANGE:PASS",
        "M4:SURVIVOR_PROGRESS:PASS",
        "M4:CLEANUP:PASS",
        "M4:PASS",
        "M1:PASS",
    ))
    return "\n".join(lines) + "\n"


class SmpEvidenceTests(unittest.TestCase):
    def test_two_and_four_cpu_transcripts_require_unique_online_apic_ids(self):
        for cpus in (2, 4):
            with self.subTest(cpus=cpus):
                result = run_smp.evaluate(cpus, transcript(cpus), 33, False)
                self.assertEqual(len(result["logical_apic_ids"]), cpus)
                self.assertEqual(result["rendezvous_generations"], 64)
                self.assertEqual(result["workers"], 64)

    def test_missing_duplicate_reordered_and_failure_evidence_is_rejected(self):
        valid = transcript()
        cases = (
            valid.replace("SMP:WORKERS:PASS\n", ""),
            valid.replace("SMP:RENDEZVOUS:PASS\n", ""),
            valid + "SMP:PASS\n",
            valid.replace("SMP:AP:ENTRY\n", ""),
            valid.replace("SMP:CPU:1:APIC:1:ONLINE\n", ""),
            valid.replace("SMP:CPU:1:APIC:1:ONLINE", "SMP:CPU:1:APIC:0:ONLINE"),
            valid.replace("SMP:PASS\n", "PANIC\nSMP:PASS\n"),
            valid.replace("KS-02:PASS\nKS-03:PASS", "KS-03:PASS\nKS-02:PASS"),
            valid.replace("SMP:CPU:0:APIC:0:ONLINE", "SMP:CPU:00:APIC:0:ONLINE"),
        )
        for bad in cases:
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                run_smp.evaluate(2, bad, 33, False)

    def test_unexpected_cpu_counts_exit_codes_and_timeouts_are_rejected(self):
        for cpus in (0, 1, 3, 5, True):
            with self.subTest(cpus=cpus), self.assertRaises(ValueError):
                run_smp.expected_markers(cpus)
        for exit_code, timeout in ((0, False), (35, False), (None, True), (33, True)):
            with self.subTest(exit_code=exit_code, timeout=timeout), self.assertRaises(ValueError):
                run_smp.evaluate(2, transcript(), exit_code, timeout)

    def test_native_m3_scheduling_requires_each_cpu_to_select_processes(self):
        for cpus in (2, 4):
            with self.subTest(cpus=cpus):
                result = run_smp.evaluate_m3(cpus, m3_transcript(cpus), 33, False)
                self.assertEqual(len(result["logical_apic_ids"]), cpus)
                self.assertEqual(len(result["per_cpu_process_selections"]), cpus)
                self.assertTrue(all(result["per_cpu_process_selections"]))

    def test_native_m3_scheduling_rejects_missing_zero_duplicate_and_reordered_evidence(self):
        valid = m3_transcript()
        cases = (
            valid.replace("M3:SMP:SCHEDULER:PASS\n", ""),
            valid.replace("M3:SMP:SCHEDULES:00000000000000000014",
                          "M3:SMP:SCHEDULES:00000000000000000000"),
            valid.replace("M3:SMP:CPU:00000000000000000001\n", ""),
            valid.replace("M3:SMP:SCHEDULES:00000000000000000007\n"
                          "M3:SMP:CPU:00000000000000000001",
                          "M3:SMP:CPU:00000000000000000001\n"
                          "M3:SMP:SCHEDULES:00000000000000000007"),
            valid.replace("M3:SMP:SCHEDULER:PASS\n",
                          "M3:SMP:SCHEDULER:PASS\nM3:SMP:SCHEDULER:PASS\n"),
            valid.replace("SMP:PASS\n", "PANIC\nSMP:PASS\n"),
        )
        for bad in cases:
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                run_smp.evaluate_m3(2, bad, 33, False)

    def test_native_m4_ipc_requires_a_successful_cross_cpu_message_pair(self):
        result = run_smp.evaluate_m4_smp(2, m4_smp_transcript(), 33, False)
        self.assertEqual(len(result["ipc_events"]), 4)
        self.assertEqual(
            result["cross_cpu_message_pairs"],
            [
                {
                    "sender_slot": 0,
                    "receiver_slot": 1,
                    "sender_cpu": 0,
                    "receiver_cpu": 1,
                },
                {
                    "sender_slot": 1,
                    "receiver_slot": 2,
                    "sender_cpu": 1,
                    "receiver_cpu": 0,
                },
            ],
        )

    def test_native_m4_ipc_rejects_missing_exchange_or_same_cpu_pairs(self):
        valid = m4_smp_transcript()
        same_cpu = m4_smp_transcript(send_cpus=(0, 1), receive_cpus=(0, 1))
        cases = (
            valid.replace("M4:EXCHANGE:PASS\n", ""),
            valid.replace("M4:SMP:SEND:00000000000000000000\n", ""),
            same_cpu,
            valid.replace("M4:SMP:SEND:00000000000000000000",
                          "M4:SMP:SEND:not-a-number"),
            valid.replace("M4:PASS\n", "M4:FAIL\nM4:PASS\n"),
        )
        for bad in cases:
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                run_smp.evaluate_m4_smp(2, bad, 33, False)


if __name__ == "__main__":
    unittest.main()

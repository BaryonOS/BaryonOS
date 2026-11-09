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
        for index in range(1, cpus)
        for marker in (
            f"M3:SMP:AP:PROTECTION_CPU:{index:020d}",
            "M3:SMP:AP:CR0_WP_EM_TS:00000000000000000007",
        )
    )
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


def m3_x87_transcript():
    lines = m3_transcript().splitlines()
    scheduler = lines.index("M3:SMP:CPU:00000000000000000000")
    lines[scheduler:scheduler] = (
        "M3:SMP:FAULT_CPU:00000000000000000001",
        "M3:FAULT:X87:PASS",
    )
    return "\n".join(lines) + "\n"


def m3_unexpected_ud_transcript():
    lines = m3_transcript().splitlines()
    lines.remove("M3:SMP:SCHEDULER:PASS")
    lines.extend((
        "M3:SMP:FAULT_INJECTION:UNEXPECTED_UD",
        "M3:FAIL:UNEXPECTED_USER_EVENT",
    ))
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
        "M4:SMP:QUIESCED:PASS",
        "M4:SMP:ROUND_CLEANUP:PASS",
        "M4:CLEANUP:PASS",
        "M4:PASS",
        "M1:PASS",
    ))
    return "\n".join(lines) + "\n"


def m4_revoke_smp_transcript(revoke_cpus=(0, 0), denied_cpus=(1, 0)):
    lines = m3_transcript(2).splitlines()
    events = (
        ("REVOKE", 0, revoke_cpus[0]),
        ("REVOKE", 0, revoke_cpus[1]),
        ("REVOKED", 1, denied_cpus[0]),
        ("REVOKED", 2, denied_cpus[1]),
    )
    lines.extend(
        f"M4:SMP:{operation}:{((slot << 8) | cpu):020d}"
        for operation, slot, cpu in events
    )
    lines.extend((
        "M4:CASE:REVOKE:PASS",
        "M4:SURVIVOR_PROGRESS:PASS",
        "M4:SMP:QUIESCED:PASS",
        "M4:SMP:ROUND_CLEANUP:PASS",
        "M4:CLEANUP:PASS",
        "KS-14:PASS",
        "M4:PASS",
        "M1:PASS",
    ))
    return "\n".join(lines) + "\n"


def m3_exit_smp_transcript(cpus=2, exit_cpu=1):
    lines = m3_transcript(cpus).splitlines()
    lines.extend(("M3:EXIT:PASS", "M3:SURVIVOR_PROGRESS:PASS",
                  f"M3:SMP:EXIT:{exit_cpu:020d}"))
    return "\n".join(lines) + "\n"


def m3_tlb_reuse_transcript(exit_cpus=(1, 1)):
    lines = m3_transcript(2).splitlines()
    for cpu in exit_cpus:
        lines.extend(("M3:EXIT:PASS", f"M3:SMP:EXIT:{cpu:020d}"))
    lines.append("M3:SURVIVOR_PROGRESS:PASS")
    lines.append("M3:SMP:TLB_REUSE:PASS")
    return "\n".join(lines) + "\n"


def m3_lock_contention_transcript(contentions=1):
    lines = m3_transcript(2).splitlines()
    scheduler_pass = lines.index("M3:SMP:SCHEDULER:PASS")
    lines.insert(scheduler_pass, f"M3:SMP:LOCK:CONTENDED:{contentions:020d}")
    return "\n".join(lines) + "\n"


def m4_lifecycle_smp_transcript():
    lines = m3_transcript(2).splitlines()
    for round_index in range(3):
        lines.extend(("M4:SMP:QUIESCED:PASS", "M4:SMP:ROUND_CLEANUP:PASS"))
        if round_index < 2:
            lines.append("M4:SMP:ROUND_RESET:PASS")
    lines.extend((
        "M4:CASE:LIFECYCLE:PASS",
        "M4:SURVIVOR_PROGRESS:PASS",
        "M4:CLEANUP:PASS",
        "KS-16:PASS",
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

    def test_common_evidence_rejections_apply_to_both_evaluators(self):
        for evaluator, fixture in (
            (run_smp.evaluate, transcript),
            (run_smp.evaluate_m3, m3_transcript),
        ):
            valid = fixture()
            cases = (
                None,
                valid + "x" * (128 * 1024),
                valid.replace("BOOT:EBS_OK\n", ""),
                valid.replace("KS-02:PASS\nKS-03:PASS", "KS-03:PASS\nKS-02:PASS"),
                valid.replace("SMP:CPU:1:APIC:1:ONLINE", "SMP:CPU:1:APIC:0:ONLINE"),
                valid.replace("SMP:CPU:1:APIC:1:ONLINE", "SMP:CPU:01:APIC:1:ONLINE"),
                valid.replace("SMP:CPU:1:APIC:1:ONLINE", "SMP:CPU:1:APIC:x:ONLINE"),
                valid.replace("SMP:CPU:1:APIC:1:ONLINE\n", ""),
                valid.replace("SMP:AP:ENTRY\n", ""),
                valid.replace("SMP:WORKERS:PASS\n", ""),
                valid + "SMP:PASS\n",
                valid + "PANIC\n",
                valid + "M3:FAIL:UNEXPECTED_USER_EVENT\n",
            )
            for index, bad in enumerate(cases):
                with self.subTest(evaluator=evaluator.__name__, case=index):
                    with self.assertRaises(ValueError):
                        evaluator(2, bad, 33, False)
            for exit_code, timeout in ((0, False), (35, False), (None, True), (33, True)):
                with self.subTest(evaluator=evaluator.__name__, exit_code=exit_code,
                                  timeout=timeout), self.assertRaises(ValueError):
                    evaluator(2, valid, exit_code, timeout)

    def test_unexpected_cpu_counts_exit_codes_and_timeouts_are_rejected(self):
        for cpus in (0, 1, 3, 5, True):
            with self.subTest(cpus=cpus), self.assertRaises(ValueError):
                run_smp.expected_markers(cpus)
        for profile in (
            "native-smp-m3-tlb-reuse",
            "native-smp-m4",
            "native-smp-m4-lifecycle",
            "native-smp-m4-revoke",
        ):
            with self.subTest(profile=profile), self.assertRaises(ValueError):
                run_smp.expected_markers(4, profile)
        for profile in ("native-smp-m3-x87", "native-smp-m3-unexpected-ud"):
            with self.subTest(profile=profile), self.assertRaises(ValueError):
                run_smp.expected_markers(4, profile)
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
            valid.replace("M3:SMP:AP:CR0_WP_EM_TS:00000000000000000007\n", ""),
            valid.replace("M3:SMP:AP:CR0_WP_EM_TS:00000000000000000007",
                          "M3:SMP:AP:CR0_WP_EM_TS:00000000000000000003"),
            valid.replace("M3:SMP:AP:PROTECTION_CPU:00000000000000000001\n"
                          "M3:SMP:AP:CR0_WP_EM_TS:00000000000000000007\n",
                          "M3:SMP:AP:CR0_WP_EM_TS:00000000000000000007\n"
                          "M3:SMP:AP:PROTECTION_CPU:00000000000000000001\n"),
            valid.replace("M3:SMP:SCHEDULER:PASS\n", ""),
            valid.replace("M3:SMP:AP:PROTECTION_CPU:00000000000000000001",
                          "M3:SMP:AP:PROTECTION_CPU:00000000000000000000"),
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

    def test_native_smp_x87_probe_requires_ap_trap_and_protection_readback(self):
        result = run_smp.evaluate_m3_x87(2, m3_x87_transcript(), 33, False)
        self.assertEqual(result["x87_fault_cpu"], 1)
        for bad in (
            m3_x87_transcript().replace("M3:FAULT:X87:PASS\n", ""),
            m3_x87_transcript().replace(
                "M3:SMP:FAULT_CPU:00000000000000000001",
                "M3:SMP:FAULT_CPU:00000000000000000000",
            ),
            m3_x87_transcript().replace(
                "M3:SMP:AP:CR0_WP_EM_TS:00000000000000000007",
                "M3:SMP:AP:CR0_WP_EM_TS:00000000000000000003",
            ),
        ):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                run_smp.evaluate_m3_x87(2, bad, 33, False)

    def test_native_smp_unexpected_ud_fails_closed_before_scheduler_pass(self):
        result = run_smp.evaluate_m3_unexpected_ud(
            2, m3_unexpected_ud_transcript(), 35, False
        )
        self.assertFalse(result["scheduler_pass"])
        cases = (
            m3_unexpected_ud_transcript().replace(
                "M3:SMP:FAULT_INJECTION:UNEXPECTED_UD\n", ""
            ),
            m3_unexpected_ud_transcript().replace(
                "M3:FAIL:UNEXPECTED_USER_EVENT\n", ""
            ),
            m3_unexpected_ud_transcript().replace(
                "M3:FAIL:UNEXPECTED_USER_EVENT",
                "M3:FAIL:UNEXPECTED_USER_EVENT\nM3:SMP:SCHEDULER:PASS",
            ),
            m3_unexpected_ud_transcript().replace(
                "M3:SMP:AP:CR0_WP_EM_TS:00000000000000000007",
                "M3:SMP:AP:CR0_WP_EM_TS:00000000000000000003",
            ),
        )
        for bad in cases:
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                run_smp.evaluate_m3_unexpected_ud(2, bad, 35, False)
        with self.assertRaises(ValueError):
            run_smp.evaluate_m3_unexpected_ud(
                2, m3_unexpected_ud_transcript(), 33, False
            )

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
        wrong_assignment = m4_smp_transcript(
            send_cpus=(1, 0), receive_cpus=(0, 1)
        )
        reversed_pair = valid.replace(
            "M4:SMP:SEND:00000000000000000000\n"
            "M4:SMP:RECV:00000000000000000257\n",
            "M4:SMP:RECV:00000000000000000257\n"
            "M4:SMP:SEND:00000000000000000000\n",
        )
        self.assertNotEqual(reversed_pair, valid)
        cases = (
            valid.replace("M4:EXCHANGE:PASS\n", ""),
            valid.replace("M4:SMP:SEND:00000000000000000000\n", ""),
            same_cpu,
            wrong_assignment,
            reversed_pair,
            valid.replace("M4:SMP:SEND:00000000000000000000",
                          "M4:SMP:SEND:not-a-number"),
            valid.replace("M4:PASS\n", "M4:FAIL\nM4:PASS\n"),
        )
        for bad in cases:
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                run_smp.evaluate_m4_smp(2, bad, 33, False)

    def test_native_m4_cleanup_requires_completed_quiescence(self):
        valid = m4_smp_transcript()
        missing = valid.replace("M4:SMP:QUIESCED:PASS\n", "")
        after_cleanup = valid.replace(
            "M4:SMP:QUIESCED:PASS\nM4:SMP:ROUND_CLEANUP:PASS\n",
            "M4:SMP:ROUND_CLEANUP:PASS\nM4:SMP:QUIESCED:PASS\n",
        )
        for bad in (missing, after_cleanup):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                run_smp.evaluate_m4_smp(2, bad, 33, False)

    def test_native_m4_lifecycle_requires_each_quiesced_cleanup_and_reset(self):
        valid = m4_lifecycle_smp_transcript()
        result = run_smp.evaluate_m4_lifecycle_smp(2, valid, 33, False)
        self.assertEqual(result["quiescence_rounds"], 3)
        self.assertEqual(result["cleanup_rounds"], 3)
        self.assertEqual(result["process_table_resets"], 2)
        premature = valid.splitlines()
        results = ("M4:CASE:LIFECYCLE:PASS", "M4:SURVIVOR_PROGRESS:PASS",
                   "M4:CLEANUP:PASS")
        for marker in results:
            premature.remove(marker)
        final_cleanup = max(
            i for i, line in enumerate(premature)
            if line == "M4:SMP:ROUND_CLEANUP:PASS"
        )
        premature[final_cleanup:final_cleanup] = results
        premature = "\n".join(premature) + "\n"
        for bad in (
            valid.replace("M4:SMP:QUIESCED:PASS\n", "", 1),
            valid.replace("M4:SMP:ROUND_CLEANUP:PASS\n", "", 1),
            valid.replace("M4:SMP:ROUND_RESET:PASS\n", "", 1),
            premature,
            valid.replace("M4:PASS\n", "M4:FAIL:SMP:CLEANUP_REMOTE_ACTIVE\nM4:PASS\n"),
        ):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                run_smp.evaluate_m4_lifecycle_smp(2, bad, 33, False)

    def test_native_m4_revoke_requires_cross_cpu_revoked_receive(self):
        result = run_smp.evaluate_m4_revoke_smp(
            2, m4_revoke_smp_transcript(), 33, False
        )
        self.assertEqual(
            result["cross_cpu_revocation"],
            {"revoker_cpu": 0, "receiver_slot": 1, "receiver_cpu": 1},
        )

    def test_native_m4_revoke_rejects_missing_or_misordered_cross_cpu_events(self):
        valid = m4_revoke_smp_transcript()
        wrong_assignment = m4_revoke_smp_transcript(
            revoke_cpus=(1, 1), denied_cpus=(0, 1)
        )
        cases = (
            valid.replace("M4:SMP:REVOKED:00000000000000000257\n", ""),
            m4_revoke_smp_transcript(denied_cpus=(0, 0)),
            wrong_assignment,
            valid.replace("M4:SMP:REVOKE:00000000000000000000\n", "", 1),
            valid.replace("M4:SMP:REVOKE:00000000000000000000",
                          "M4:SMP:REVOKE:not-a-number", 1),
            valid.replace("KS-14:PASS\n", ""),
        )
        for bad in cases:
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                run_smp.evaluate_m4_revoke_smp(2, bad, 33, False)

    def test_native_m3_exit_requires_cleanup_on_an_ap(self):
        result = run_smp.evaluate_m3_exit_smp(2, m3_exit_smp_transcript(), 33, False)
        self.assertEqual(
            result["process_cleanup_event"],
            {"slot": 0, "cpu": 1},
        )

    def test_native_m3_exit_rejects_missing_or_bsp_cleanup(self):
        valid = m3_exit_smp_transcript()
        cases = (
            valid.replace("M3:SMP:EXIT:00000000000000000001\n", ""),
            m3_exit_smp_transcript(exit_cpu=0),
            valid.replace("M3:SMP:EXIT:00000000000000000001",
                          "M3:SMP:EXIT:not-a-number"),
            valid.replace("M3:EXIT:PASS\n", ""),
        )
        for bad in cases:
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                run_smp.evaluate_m3_exit_smp(2, bad, 33, False)

    def test_native_m3_tlb_reuse_requires_two_replacements_on_the_same_ap(self):
        result = run_smp.evaluate_m3_tlb_reuse_smp(
            2, m3_tlb_reuse_transcript(), 33, False
        )
        self.assertEqual(
            result["process_cleanup_events"],
            [{"slot": 0, "cpu": 1}, {"slot": 0, "cpu": 1}],
        )
        self.assertTrue(result["stale_mapping_target_retained_until_replacement_exit"])

    def test_native_m3_tlb_reuse_rejects_one_exit_bsp_exit_or_missing_evidence(self):
        cases = (
            m3_tlb_reuse_transcript(exit_cpus=(1,)),
            m3_tlb_reuse_transcript(exit_cpus=(1, 0)),
            m3_tlb_reuse_transcript().replace("M3:SMP:TLB_REUSE:PASS\n", ""),
            m3_tlb_reuse_transcript().replace("M3:SMP:EXIT:00000000000000000001\n", "", 1),
        )
        for bad in cases:
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                run_smp.evaluate_m3_tlb_reuse_smp(2, bad, 33, False)

    def test_native_m3_lock_contention_requires_nonzero_ordered_evidence(self):
        valid = m3_lock_contention_transcript(3)
        result = run_smp.evaluate_m3_lock_contention_smp(2, valid, 33, False)
        self.assertEqual(result["lock_contention_attempts"], 3)
        malformed = valid.replace(
            "M3:SMP:LOCK:CONTENDED:00000000000000000003",
            "M3:SMP:LOCK:CONTENDED:0000000000000000000x",
        )
        out_of_order = valid.replace(
            "M3:SMP:LOCK:CONTENDED:00000000000000000003\n",
            "",
        ).replace(
            "M3:SMP:SCHEDULER:PASS\n",
            "M3:SMP:SCHEDULER:PASS\n"
            "M3:SMP:LOCK:CONTENDED:00000000000000000003\n",
        )
        for bad in (
            m3_lock_contention_transcript(0),
            valid.replace("M3:SMP:LOCK:CONTENDED:00000000000000000003\n", ""),
            malformed,
            out_of_order,
        ):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                run_smp.evaluate_m3_lock_contention_smp(2, bad, 33, False)

    def test_native_m3_lock_contention_profile_requires_two_cpus(self):
        with self.assertRaises(ValueError):
            run_smp.expected_markers(4, "native-smp-m3-lock-contention")


if __name__ == "__main__":
    unittest.main()

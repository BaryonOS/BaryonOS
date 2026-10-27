"""Shared guest-evidence expectations for milestone gate cases."""

BOOT_MARKERS = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK")
M1_MARKERS = BOOT_MARKERS + ("M1:PASS",)
FAIL_MARKERS = ("M1:FAIL", "M2:FAIL", "M3:FAIL", "M4:FAIL")
M2_AUDIT_MARKERS = ("KS-01:PASS", "KS-02:PASS", "KS-03:PASS")
M3_CASES = ("peer-read", "peer-write", "kernel-write", "stack-exec", "heap-exec",
            "guard-low", "guard-high", "cli", "x87", "xmm", "avx", "ud", "exit",
            "b-exit", "invalid-rsp")
M3_NONFAULT_CASES = ("exit", "b-exit", "invalid-rsp")
M3_EVENT_MARKERS = {
    "exit": "M3:EXIT:PASS",
    "b-exit": "M3:EXIT_NONZERO_B:PASS",
    "invalid-rsp": "M3:INVALID_RSP:PASS",
}
M4_CASES = (("copy", "KS-09"), ("wire", "KS-10"), ("snapshot", "KS-11"),
            ("handles", "KS-12"), ("rights", "KS-13"), ("revoke", "KS-14"),
            ("quotas", "KS-15"), ("lifecycle", "KS-16"))

MILESTONE_MARKERS = {
    "m1": M1_MARKERS,
    "m2": BOOT_MARKERS + M2_AUDIT_MARKERS + ("M2:PASS", "M1:PASS"),
    "m3": BOOT_MARKERS + ("KS-01:PASS", "KS-02:PASS", "KS-03:PASS", "KS-04:PASS",
                          "KS-07:PASS", "KS-08:PASS", "M3:SYSCALL_DF_ENTRY:PASS",
                          "M3:PASS", "M1:PASS"),
    "m4": BOOT_MARKERS + ("M4:EXCHANGE:PASS", "M4:SURVIVOR_PROGRESS:PASS",
                          "M4:CLEANUP:PASS", "M4:PASS", "M1:PASS"),
}


def guest_expectation(phase, group):
    """Return required markers, raw exit, and forbidden markers for a gate guest."""
    expected_exit = 33
    forbidden = FAIL_MARKERS

    if phase == "m2":
        if group == "own-cr3-frame-ownership-wx":
            required = MILESTONE_MARKERS["m2"]
        elif group in ("test-m2-ro", "test-m2-nx", "test-m2-absent"):
            marker = {"test-m2-ro": "M2:FAULT:RO:PASS",
                      "test-m2-nx": "M2:FAULT:NX:PASS",
                      "test-m2-absent": "M2:FAULT:ABSENT:PASS"}[group]
            required = BOOT_MARKERS + M2_AUDIT_MARKERS + (marker,)
        elif group == "kernel-fault-stops-controlled":
            required = BOOT_MARKERS + M2_AUDIT_MARKERS + (
                "M2:FAULT:KERNEL_UD:PASS", "M2:FAIL:KERNEL_EXCEPTION")
            expected_exit = 35
            forbidden = ("M1:PASS", "M2:PASS", "M1:FAIL")
        elif group == "unusable-main-stack-double-fault-ist":
            required = BOOT_MARKERS + M2_AUDIT_MARKERS + ("M2:FAULT:DF_IST:PASS",)
            expected_exit = 35
            forbidden = ("M1:PASS", "M2:PASS", "M1:FAIL", "M2:FAIL")
        elif group == "m1-regression":
            required = M1_MARKERS
        else:
            return None
    elif phase == "m3":
        if group == "private-address-spaces-preemption-fairness-cleanup":
            required = MILESTONE_MARKERS["m3"]
        elif group.startswith("test-m3-") and group.removeprefix("test-m3-") in M3_CASES:
            case = group.removeprefix("test-m3-")
            observed = (M3_EVENT_MARKERS.get(
                case, f"M3:FAULT:{case.upper().replace('-', '_')}:PASS"),)
            requirement_marker = ()
            if case not in M3_NONFAULT_CASES:
                requirement = ("KS-05" if case in ("peer-read", "peer-write", "kernel-write")
                               else "KS-06")
                requirement_marker = (f"{requirement}:PASS",)
            required = BOOT_MARKERS + observed + (
                "M3:SURVIVOR_PROGRESS:PASS", "M3:CLEANUP:PASS") + requirement_marker + (
                "KS-08:PASS", "M3:SYSCALL_DF_ENTRY:PASS", "M3:PASS", "M1:PASS")
        elif group == "all-processes-exit-idle-timer-and-resources":
            required = BOOT_MARKERS + ("M3:IDLE:PASS", "M3:PASS", "M1:PASS")
        elif group == "large-bss-image-exceeds-old-64-mapping-limit":
            required = MILESTONE_MARKERS["m3"]
        elif group == "m1-regression-after-m3":
            required = M1_MARKERS
        else:
            return None
    elif phase == "m4":
        if group == "real-ring3-message-exchange":
            required = MILESTONE_MARKERS["m4"]
        elif group.startswith("test-m4-"):
            case = group.removeprefix("test-m4-")
            requirement = dict(M4_CASES).get(case)
            if requirement is None:
                return None
            required = BOOT_MARKERS + (
                f"M4:CASE:{case.upper()}:PASS", "M4:SURVIVOR_PROGRESS:PASS",
                "M4:CLEANUP:PASS", f"{requirement}:PASS", "M4:PASS", "M1:PASS")
        elif group == "m1-regression-after-m4":
            required = M1_MARKERS
        else:
            return None
    else:
        return None

    return required, expected_exit, forbidden

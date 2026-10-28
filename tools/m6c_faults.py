"""Host contracts for explicitly injected native M6c uncertainty cases."""

FAULT_CASES = ("write-error", "flush-error", "write-timeout", "flush-timeout",
               "reset-timeout", "bad-descriptor", "used-write", "used-flush")


def outcome_valid(stdout, case):
    """Reject an accidental Unknown, forged ACK, replay or incomplete fault proof.

    These are guest observations, not replacement output. The independent disk
    decoder still verifies the image, whose state may exceed the live ACK state.
    """
    prefixes = ("M6C:FAULT_INJECTED", "M6C:OUTCOME", "M6C:UNKNOWN_REPLIES",
                "M6C:NO_REPLAY", "M6C:POST_UNKNOWN", "M6C:LIVE_SEQUENCE", "M6C:QUARANTINE")
    lines = stdout.splitlines()
    fault_lines = [line for line in lines if line.startswith(prefixes)]
    if case not in FAULT_CASES:
        return not fault_lines
    name = case.upper().replace("-", "_")
    expected = {f"M6C:FAULT_INJECTED:{name}", "M6C:OUTCOME:UNKNOWN",
                "M6C:UNKNOWN_REPLIES:11", "M6C:NO_REPLAY:PASS",
                "M6C:POST_UNKNOWN:BLOCKCALLS:0", "M6C:POST_UNKNOWN:PUBLICATIONS:0",
                "M6C:POST_UNKNOWN:COMPLETIONS:0", "M6C:LIVE_SEQUENCE:1"}
    if case == "reset-timeout":
        expected.add("M6C:QUARANTINE:PASS")
    if len(fault_lines) != len(expected) or set(fault_lines) != expected:
        return False
    counts = {"M6C:ACKS": 1, "M6C:REJECTED_NO_IO": 22, "M6C:CATALOG_REPLIES": 0}
    return all([line for line in lines if line.startswith(prefix)] == [f"{prefix}:{count}"]
               for prefix, count in counts.items())

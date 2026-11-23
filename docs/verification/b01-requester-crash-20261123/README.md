# B01-T2: native requester-crash and missing reset-ACK cases

Date: 23 November 2026 (Europe/Amsterdam). Criterion B01 (entropy/keyring
broker), task B01-T2. Branch `claude/b01-requester-crash-20261123`, base
`origin/main` `297d7bfc`. Source fingerprint v3 `9dfc6062748fe6a86c630d68823fea4ba07fd74e89a723aafdc8039b9b95fddd`;
per-stage records: `keyring-gate.json`; file hashes: `SHA256SUMS` (this README is not covered).

Command: `NOS_QEMU_MACHINE=pc-q35-8.2 python3 -B -u tools/test_keyring.py`
with separate `NOS_BUILD_DIR`/`CARGO_TARGET_DIR` in the worktree. Host phase:
the three existing host commands plus `unittest discover -s tools/tests -p
'test_keyring_*.py'`. Then seven sequential QEMU guests on one frozen source
fingerprint: normal, no-device, stuck, timeout, restart, requester-crash,
no-reset-ack. Every stage: raw QEMU exit 33, the controlled marker sequence
and no `FAIL`/`PANIC` line. Captured with `tools/capture_keyring.py`.
QEMU 10.2.1 (Debian), TCG, qemu64, 1 CPU, 256 MiB.

## What the two new cases show

| Case | Native result |
|---|---|
| requester-crash | Service B submits one real 256-byte request to a silent EGD backend and executes `ud2` while it is pending. The kernel accepts only that fault (slot 1, vector 6, first crash, pending bound to B's PID, no completed answer). On the next poll, before the broker's own liveness check runs, the kernel requires that the synchronous cleanup path (`retire_authority` then `detach`) already quarantined the broker: no pending or completed request, zero bytes, offline -14, one entropy object left, all three DMA pages kernel-owned. Only then is service A released; it starts BLOCKED Source. Kernel-driven probes through the real handler: B's dead ticket redeemed with A's handle returns -1, and two fresh requests return -14 with no new ticket, submission or byte. `REQUESTER_CRASHES=1`, `CRASH_TICKET_REFUSED=1`, `REPEAT_REFUSED=2`, `DEVICE_SUBMISSIONS=1`, `TIMEOUTS=0`, `RESET_ACKS=0`, `QUARANTINED=1`, `DELIVERED_BYTES=0`. |
| no-reset-ack | The timeout flow on the silent backend, with a labelled fixture that treats the device's reset acknowledgement as never seen. At the reset deadline the broker quarantines: `TIMEOUTS=1`, `RESET_ACKS=0`, `QUARANTINED=1`, offline stays -13, no re-initialisation (`RESTARTED_BYTES=0`), 0 bytes, both services BLOCKED Source. Two fresh requests are refused (`REPEAT_REFUSED=2`, `DEVICE_SUBMISSIONS=1`). The final reset-ACK check is skipped in this case only; the DMA-pool and empty-engine checks still run. |

Both cases end with `KEYRING:RESOURCES:PASS`: exact frame baseline, empty
capability engine, and the three DMA pages still kernel-owned and mapped
NX+W. Quarantine keeps DMA pinned until reboot, which is stricter than
"until reset-ACK"; no un-quarantine path was added.

## Negative control (development only, reverted)

With the `quarantine_pending()` call removed from `detach()`, the
requester-crash guest stopped with `KEYRING:FAIL:CRASH_NOT_QUARANTINED`
and raw exit 35 (marker sequence ended after `KEYRING:MANIFEST:PASS`).
The change was reverted before the gate above. The run record is kept
outside the repository (`build/b01t2-negative-control/`, ignored).

## Silent-backend transcript

QEMU forwards one 256-byte guest buffer as EGD requests of 255 and 1
bytes. On QEMU 10.2.1 the unchanged timeout case gives `[255, 1, 255, 1]`
with one kernel ticket, so the host check accepts only whole `[255, 1]`
pairs and an unanswered backend; the number of guest submissions is pinned
kernel-side by `DEVICE_SUBMISSIONS=1`, not by the transcript.

## Evidence limits

- Service B's crash is a labelled fault injection (`ud2` in the test Key
  Service), not an organic defect.
- The missing reset-ACK is a simulated acknowledgement read: QEMU
  virtio-pci acknowledges a status-0 reset. The case proves the broker's
  deadline/quarantine branch, not device behaviour.
- PID or endpoint reuse after the crash is not exercised; this profile has
  no spawn.
- A crash after completion (completed answer pending consumption) is
  covered only by the existing `detach` code, not by a native case.
- The probes are kernel-driven calls through the real syscall handler on
  the surviving service's capability, as the existing lease and
  authority-loss fixtures.
- `CRASH_TICKET_REFUSED` is defence in depth, not independent evidence: at
  that point no pending or completed request exists and the ticket belongs
  to another binding, so `0x61` returns -1 for either reason. The
  load-bearing checks are the crash-gate state checks, which the negative
  control above shows fail without the quarantine.
- A device that answers late after the crash (stale DMA) is not exercised;
  the crash case uses only the silent backend.
- After a missing reset-ACK the broker is quarantined but still reports
  the timeout code -13, not -14. This predates B01-T2 and is a diagnostic
  gap only: restart and re-initialisation both require `!quarantined`.
- Both new features are rejected at compile time together with
  `native-smp-legacy` or `native-system`, so the fixtures cannot alter the
  shared broker path of the system profile. The two features also imply
  `test-keyring-timeout`, which only selects the three-process slot count
  in `m3.rs`; `case()` tests the B01-T2 features first.
- No physical RNG, min-entropy, TPM or side-channel claim. AI-written and
  AI-checked; no independent human audit.

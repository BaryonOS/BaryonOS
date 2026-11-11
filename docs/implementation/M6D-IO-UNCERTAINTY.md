# M6d — native storage errors without kernel panic

Date: 28 October 2026. Contract recorded in advance, original base
`b58ae2a` (M6c), working branch `codex/m6d-io-recovery-roadmap`.
The M6c/M6d implementation and version-bound native execution have since been
recorded in [M6D-ACCEPTANCE.md](../verification/M6D-ACCEPTANCE.md) and the
later [Windows closeout](../verification/M6D-CLOSEOUT.md). This design document
itself is not a test result. The historical standalone M6b observer remains
unchanged and gets no new error-handling claim from this.

## Problem and chosen boundary

The real ring-3 catalogue service already answers failed WRITE/FLUSH operations
conservatively with `Unknown`. The kernel observer of the M6c trial, however, always
expected the predicted successful mutation. As a result, precisely that correct
error reply could cause a kernel panic. This step lets the error reach the app and
checks the boundary with real QEMU execution of the own kernel.

The existing IPC v4, journal v2 and capability contracts remain in force.
M6d builds on `milestone-m6c`, without a new syscall, dependency,
unbounded driver rights or changed disk layout. The older separate
M6b trial remains selectable; the M6b observer is not changed in this step.

## Invariants

1. A first `Unknown` follows exclusively on an actually observed failed
   mutating submit/poll in the broker. Pending is not an error. An arbitrary
   Unknown reply without that cause must not bypass the success checks.
2. After uncertainty the service stays closed until a new instance recovers the
   disk. GET, LIST, PUT, DELETE and RENAME from both app domains return
   `Unknown`, empty data and revision zero. Invalid or unauthorised requests
   keep getting their own validation error.
3. The uncertain mutation does not increase any live revision, success ACK or catalogue counter.
   There is no new block call, descriptor publication or automatic replay.
   An already published device operation may be present on the disk without an ACK.
4. Normal success still requires a real WRITE completion and FLUSH completion.
   Capabilities and endpoint-bound domains are not widened.
5. The service and both apps stay scheduled; both apps demonstrably make progress. Unconfirmed
   reset leaves DMA buffers pinned and the device in quarantine until VM stop.
6. Cold recovery starts a new VM on exactly the same image bytes, validates the
   actual journal prefix and performs no writes. No deletion or
   formatting to make a test pass.

## Eight bounded native error trials

Each trial first confirms one PUT of object 0 (`Café A`, revision 1), and
then interrupts the RENAME to `Café hernoemd A` ("Café renamed A"). One earlier ACK is therefore
allowed; no success ACK is allowed for the RENAME.

| Case | Injection boundary | Possible actual final prefix |
|---|---|---|
| `write-error` | local status error after real WRITE used-ring completion | PUT + RENAME |
| `flush-error` | local status error after real FLUSH used-ring completion | PUT + RENAME |
| `write-timeout` | target WRITE stays without completion for the broker until deadline | PUT, or PUT + RENAME |
| `flush-timeout` | target FLUSH stays without completion for the broker until deadline | PUT + RENAME |
| `reset-timeout` | timeout plus locally unobserved reset confirmation | PUT, or PUT + RENAME |
| `bad-descriptor` | descriptor check refuses before publication | exclusively PUT |
| `used-write` | invalid local used-length after real WRITE completion | PUT + RENAME |
| `used-flush` | invalid local used-length after real FLUSH completion | PUT + RENAME |

The tests change no DMA address in the real device. Status/used-length
injections change exclusively the local completion snapshot. Timeout trials
hide completion from the broker; suppressed notify alone does not prove that
a device never read. This is native error-path evidence in an explicit
test profile, not evidence of a physical disk error or power loss.

## Acceptance and evidence

- An independent Python decoder checks all real image bytes, names,
  content, revisions and the untouched sectors outside the journal.
- The still-valid live state (sequence 1) and the possible disk state
  (sequence 1 or 2) are checked separately. The harness must not bend the disk
  toward the desired outcome.
- Each error case gets its own native run and its own cold recovery; no
  synthetic success logs or replacement of native tests by host mocks.
- Negative harness tests reject duplicate/missing error markers, false ACKs,
  extra journal records, wrong name/content and swapped images.
- The full M6c gate and the relevant M6b regression remain required. Existing
  historical evidence is kept under its original version.
- Checked source checkpoint before long tests; raw exit codes, commands,
  fingerprints and artifact hashes in the evidence archive. Separate AI review
  is not a human audit or formal proof.

## Primary technical source

OASIS Open. (2022, 1 July). *Virtual I/O Device (VIRTIO) Version 1.2*
(Committee Specification 01), §§2.4, 2.7.8 and 5.2.6.
https://docs.oasis-open.org/virtio/virtio/v1.2/cs01/virtio-v1.2-cs01.html
(accessed 28 October 2026).

The project choices for `Unknown`, journal prefix, test budgets and point allocation
are own contracts, not properties proven by VIRTIO.

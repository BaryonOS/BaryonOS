# N04 native unregistered-helper isolation

Date: 14 November 2026. Follow-up branch: `test/issue43-n04-helper-isolation`.
Base: merged PR #328, `4cd444e08dac820cc66f2750c43845b4139901cc`.

## Contract and implementation

The opt-in `test-net-service-helper` manifest starts one network service,
two registered clients and a fourth process that is not registered as a network
client. Only this negative fixture gives the helper SEND aliases to both real
app ingress endpoints. It receives no reply endpoint, policy channel, raw NIC,
entropy, block or launch authority. No production permission or syscall changes.

The helper first exercises a denied raw-NIC syscall, then sends six well-formed
NET1 requests: OpenTcp, SendUdp and Retire for each app, using the allowed
destinations and request numbers `u64::MAX - 2`, `u64::MAX - 1`, `u64::MAX`.
Both legitimate clients remain paused until all six requests have been rejected
on the actual kernel envelope PID, with no decoded request, socket, payload,
retirement or revocation effect. The kernel additionally checks zero TX/RX
before release. Ten observed service counters and causal process times are
emitted and checked by the host oracle.

After release, both clients run their complete normal TCP/UDP scenarios. This
also checks that the helper could not poison the replay counters. The existing
packet peer requires exactly the normal allowed traffic and rejects extra or
unexpected packets. The helper retains its aliases until rejection is observed,
then closes them. Existing process, frame, capability and NIC cleanup checks
cover all four processes. The ordinary 6,000-tick/90-second limits remain.

## Verification after the PR #328 squash

Executable source: `21a53dd9342988263f7d49e9ef559fb7a2d288cd`. Runtime source-v3:
`865b0320b2da88dda92991fdf89de17989ec3807d58fccb0691a8f735f12d7e2`. The ten-file N04 delta was integrated onto merged main;
main's Python quality fixes and the earlier H02/network/update work are retained.

Fresh execution on this source passes all six native network cases, including
helper and full port exhaustion (raw QEMU exit 33). The helper has six sender
rejections and zero decoded/backend or packet effects before release at tick 2;
both legitimate clients then complete their normal flow. Strict preflight,
standard build and QEMU smoke pass with matching artifacts. The 92 Python
tests, 24 Rust tests, strict service Clippy and both formatting checks pass.
A separate read-only AI comparison found no integration blocker or production
authority change. These checks do not constitute a human security audit.

Current receipts: [integration commands and results](integration.json),
[helper raw report](integration-helper-report.json),
[host checks](integration-host-checks.json), and
[smoke result](integration-smoke-result.json).
Hosted checks remain a separate result. Later documentation commits retain
the same executable fingerprint.

Published source: `7f693d73702072fa0db3b99ffc56df3eed6b8fa5`. The GitHub connector
recreated the source commit because native Git push lacked credentials. Its
complete Git tree `7ad12f6febb30cf66bd3f2c7c7f9bd6b5922ee82` exactly matches
the locally executed source `21a53dd9342988263f7d49e9ef559fb7a2d288cd`;
only commit metadata differs. Original run receipts retain the local commit ID.

## Earlier local verification

The following receipts predate the squash and were not included in merged
PR #328. Their original source identities are retained, including the
sender-bypass mutation and independent artifact replay.


Executable source: `97d23ff603e0932218b893df0b994a8a15f93217` (includes concurrent H02 continuation
`885e128082483ec871831fbbfc757a6e5104e6b4`). Runtime source-v3:
`a0ea5f864712a11f302d8e2f189990eff9e77d96f550c9e28348aa7ad8facb3c`.

- All six cases pass: helper, normal, client-exit, revoke, device-reset and the
  complete 12,768-port exhaustion workload. Each exits QEMU with raw status 33.
- Helper: six sends, six sender rejections, zero decoded requests/backend
  effects at release tick 2; both apps then complete their entire normal flow.
  There are 10 legitimate opened sockets, two UDP requests, 45 transmitted and
  39 received frames, and 28 final service rejections. Cleanup passes.
- A separate worktree changes only the sender guard to accept any nonzero PID.
  The native gate correctly fails: six unauthorized requests are decoded, two
  sockets open and both owners are retired; `HELPER_EARLY_EXIT` ends the guest
  with raw exit 35. Compilation succeeds and no host timeout occurs.
- Strict preflight, standard native build and QEMU smoke pass on the same
  runtime fingerprint. The smoke exits 33 with matching artifacts.
- 92 Python peer/workflow/evidence/build-path tests, 24 Rust component tests,
  warning-denying service Clippy and formatting pass. The component receipt
  records its source hashes; those inputs are unchanged by the H02 integration.
- Separate read-only AI review corrected the helper UDP payload-length field
  before native execution. A second artifact replay passed all 22 positive
  source/serial/packet/ELF/image/firmware checks and verified the intended
  mutant failure. Neither review is an independent human audit.

Receipts: [commands and results](verification.json), [helper raw report](helper-report.json),
[sender-bypass report](sender-bypass-report.json), [independent replay](independent-replay.json),
[component checks](host-checks.json), [smoke](smoke-result.json), and [environment](environment.json).
Later status/receipt commits do not change executable inputs. Current-head
hosted CI remains a separate result; these receipts report local execution.

The two raw reports retain actual serial and packet records. Binary images,
ELFs and firmware are not committed. Other regression reports are represented
by hashes/counters and remain in `build/issue43-helper-*`; this is not a complete
raw-image archive. Local command logs: `build/issue43-helper-checks/`.

## Acceptance boundary and continuation

N04 remains open. This proves no production delegation or limited subgrant,
regrant generation, durable revocation after service restart, physical device,
TLS or independent human security acceptance. Issue #43 remains below 50/100;
the fixed score remains **41/100**. The next N04 implementation requires a
reviewed production issuer/delegation contract, generation-bound regrant and a
service-restart revocation design. This bounded test follow-up introduces no
production grant API. Rollback uses an ordinary revert of the N04-only changes;
the already merged H02/network/update work remains independent.

# Letting GUI processes wait when no work is available

Implementation checkpoint, 2 November 2026. Baseline: `b1b292aadf244dacf029b7307aebdc6103756e0d`; branch `codex/window-idle-wait-20261102`.

## Goal and boundary

Materiaal, Calculator, the window compositor and window-init use the existing IPC_WAIT only when they demonstrably have no work of their own. Kernel, ABI, permissions, protocol messages, storage and transaction timeouts remain the same. No new dependency, hardware access or user pointer. A host test or ELF build does not prove low CPU usage or complete native recovery acceptance.

The baseline has a known recovery failure: the normal complete window probe on b1b292a fails after the compositor crash. The local Windows/WHPX run actually ended with raw exit 35 and APP_EXIT, without an adapter or cleanup error. This optimisation does not turn that evidence into a pass. Native A/B testing first requires incorporation of the separate central recovery fix and rechecking of the changed states.

## Contract

- Known ABI 1.8/1.9 retains the existing yield path; ABI 1.10/1.11 can use IPC_WAIT. For these display/input apps, ABI 1.0–1.7 is explicitly rejected; operation on those versions was not demonstrated. Unknown ABI and invalid clock/wait mask/permissions provide no polling fallback.
- A wait plan uses only an existing receive permission and `now.checked_add(1)`. This is an interruption bound for idling; it neither creates nor renews a transaction deadline. The kernel continues to enforce permissions and queue conditions.
- A wake is only a reason to recheck. It consumes no message, grants no permission and replaces neither actual receive/PEEK nor reply validation.
- The input route uses pull PEEK. An Empty reply promises no future push notification. App adapters perform another actual input check no later than the next PIT tick; the control queue can wake them earlier. The compositor retains the existing INPUT_POLL_TICKS=2: it wakes no later than the next tick but performs PEEK at the existing two-tick cadence. This is not a new guarantee of an input PEEK every tick.
- An unsaved but fully drawn document may idle. Document-dirty is not render-dirty. Saving, uncertainty, conflict, closed transports, rebind, a pending offer, an incomplete frame and an exhausted input batch prevent idling.
- The compositor idles only after an actually empty ingress pass, without reply/input backlog, pending SUR1 tile or partial frame, and without remaining damage. The wrapper calls the original SUR1 receiver; sender, epoch, request, length, COMMIT and blit checks remain authoritative.
- Window-init idles only in Running, with ready mask 31 and an actually empty registry, without recovery or shutdown. Pause/rebind/resume takes priority.

## Change scope

`sdk/examples/materiaal/`, `sdk/examples/calculator/` and `sdk/examples/window-session/`. The typed RPC and native IPC_WAIT interfaces are unchanged. New helpers contain only idle conditions, checked deadlines and wake-mask validation. Original data traffic and recovery order are not replaced.

## Executed component checks

Compact source-bound checkpoint metadata is under `docs/verification/gui-idle-20261102/`. On the implementation before incorporation of the subsequent central recovery source, the following actually passed:

- Materiaal: 35 host tests; Calculator: 24 host tests; window-session: 24 host tests.
- Native component builds and ELF inspection; separate Materiaal ELF-boundary check.
- Host and native Clippy with `-D warnings`, scoped rustfmt and diff check.
- Negative cases include unknown ABI, checked overflow, stale render generation, save/uncertainty/rebind, input after Empty, a full batch, partial SUR1 transactions, a complete old frame followed by a partial new frame, backpressure and invalid wait masks. Starting at tick 10 keeps Commit209 permitted and Commit210 timed out; fragments do not renew the existing deadline.

These are component results. Complete native image/recovery/storage acceptance and CPU improvement have not yet been demonstrated at this checkpoint.

## Native A/B after recovery integration

1. Record the new corrected baseline; build baseline and idle variant in different checkouts/targets with the same compiler, firmware, QEMU, backend, one vCPU and disposable media.
2. Retain the complete existing run_window_session probe: exact app images and protected borders, actual input ACKs, compositor fault, model/epoch preservation, save/reopen, disk oracle, cleanup and raw exit 33.
3. Measure CPU only at exactly matching stable images and report available app/render counters before/after. Use at least fifteen seconds of monotonic host time, process and vCPU-thread accounting. A controlled QMP pause is diagnostic only, not an idle optimisation.
4. Check input and control/STOP after idling and repeat with the same backend. Report spread and host load. The target is at least 80% lower median CPU usage at the verified static image; without measurement this remains a target.
5. No new roadmap points for this performance change. U01/U02 and other product gates have their own acceptance.

## Rollback

The change is in a separate branch and can be reverted as a bounded commit. Existing kernel and protocol implementations are not modified. No BIOS, driver, Windows-feature, security or physical-device change is part of this task.

## Integrated recovery source 337f2f2

The central successor `337f2f29f1833ef4a8d2cd640b09419f85dbb83b` was incorporated separately. Its Recovery phases and ready_bit validation are intact; recovery.rs and tests/recovery.rs are byte-identical. Init additionally rejects idling when recovery.is_some(), even if a recovering flag were inconsistent. The new actual-phase test passes through pause, ready, rebind ACK and complete. After integration, 31 window-session host tests plus host/native/fault Clippy, both native component builds/ELF, fmt and diff pass. Exact commands/log hashes are in `../verification/gui-idle-20261102/recovery-integration/commands.json`. Complete native acceptance and CPU improvement must be measured separately.

## Complete native Windows follow-up

All six formal Windows/WHPX runs pass: three baseline runs A on 337/source3919 and three idle runs B on 1f2/source8faf. Each run completes the original thirteen phases, actual crash/recovery, save/reopen, disk oracle and cleanup/raw exit 33. Median QEMU process CPU usage falls by 88.26% at exactly the same complete screen images. This does not replace local TCG, physical, energy or authentication acceptance. Raw counters, compressed native/serial originals, hashes, measurement helper and limitations: [Windows A/B](../verification/gui-idle-20261102/windows-ab/README.md).

## Central review and measurement boundary

On the same physical PC, headless WHPX/1 vCPU/256 MiB, six complete native
routes passed with unchanged source runner/pixels/ACKs/deadline/disk/cleanup.
Median process CPU usage was 99.3724%→11.6663% of one logical CPU in three
15-s intervals per arm; measured median reduction 88.2600%. This is not an
energy, no-redraw or general performance guarantee. The compositor retains
its two-tick polling threshold; an already due poll can occur one idle tick
later because of a one-tick wait. No hard 20-ms/p95 interaction guarantee or
U02 acceptance follows. See windows-ab/README.md and central-review.json
under the verification archive.

## Central integration completed

PR123 was incorporated into PR122's desktop branch as 8289d60 after eighteen
passing CI workflows; no merge to main. The final TCG window gate on head
2a7b744/source0844 passes with all thirteen phases and raw33. The test-only
Python F12 fix changes the complete source hash; compiled OS sources and
all ten normal binaries remain the same. Formal CPU measurements retain
1f2/source8faf as their source.
[Integration, artifact checks and package evidence](../verification/gui-idle-20261102/INTEGRATION.md).

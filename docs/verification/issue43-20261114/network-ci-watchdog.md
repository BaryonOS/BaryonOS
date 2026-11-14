# N01 full-quota fixture watchdog — 14 November 2026

## Retained failure and diagnosis

Hosted run [43600849004](https://github.com/BaryonOS/BaryonOS/actions/runs/43600849004),
job `132465223071`, tested PR head `438da2efd41f9da7f5dd47dfda70e5f6426f15f7`
through merge commit `9e70261f1ce981eb229c05c0d72935ba1e003028`. The native
exhaustion report binds source-v3
`50c59a5c08d78cbd8dfc4e1f409b4ef8306eb9a00108e74e35309077271b7aba`.
QEMU 8.2.2 (`1:8.2.2+ds-0ubuntu1.18`) exited 35 after 62.472205 seconds;
the host timeout did not fire.

| Retained original | SHA-256 |
|---|---|
| Actions artifact ZIP | `351676e173a0421ea91b89080f7229bf30eed55796f2ff75ce77f63b29ce9907` |
| `result.json` | `8675b6e31faeef50c98cc5fd15bacc98ed5280af014ba7e51d0b26dcdadc39f8` |
| `serial.txt` | `c2683d474532dd2264c013d0eee8d689091a82b627246f5a1d57d6507706acee` |

At tick 6,000, waiting app B failed its inherited aggregate fixture check
(code 134). App A had observed 10,867 opens and 10,866 closes; the service
had decoded 21,736 requests and completed 10,867 opens and closes. The
terminal snapshot therefore contains about 85% of the required 12,768
open/close pairs, with one completed close reply not yet observed by A.
It is evidence of substantial completed work, not a proof of continuous
progress or absence of every possible scheduling defect.

The identical source had passed locally with QEMU 10.0.13: app A exited at
tick 5,490, leaving only 510 ticks before the inherited limit. The CI
snapshot projects approximately `6000 * 12768 / 10867 = 7049.60` ticks for
the full quota before final owner-B operations and cleanup. This is a
workload estimate, not a throughput guarantee or a substitute for a new
hosted run.

The existing client performs an authorized syscall before every wait,
blocks on queue readiness after `Again`, and retries the syscall after
waking. The idle owner waits in bounded 50-tick intervals. No request is
replayed after a successful send. The service retains its existing fair
per-owner processing and actual network polling. No scheduler or RPC
shortcut is introduced to make the workload pass.

## Narrow correction and unchanged requirements

Only `test-net-service-exhaustion` receives an aggregate deadline of
**12,000 ticks** and a default host limit of **150 seconds**. The deadline
is exclusive: client and kernel require completion before tick 12,000;
the host oracle accepts observation ticks only from 1 through 11,999.
Timed waits remain clamped to the same deadline. The 12,000-tick allowance
is about 1.7 times the projected CI workload. At the fixture's 100-Hz tick
rate, 150 seconds retains 30 seconds outside the nominal 120-second guest
budget for startup and termination. Scheduling and host-load variation
remain possible; the next hosted result must verify the correction.

The normal, client-exit, revoke and device-reset profiles retain their
existing inclusive 6,000-tick observation limit and 90-second host default.
The 180-second explicit host maximum is unchanged. Every one of A's 12,768
real open/close pairs, the actual `Exhausted` result, B's later successful
allocation/state/close while A remains registered, and all authority,
packet-capture and cleanup assertions remain required. The 300-tick
`OpenTcp` connection-establishment deadline, the protocol's separate
timeout bound and the SDK's 100,000 retry budget are unchanged.

A genuinely stalled exhaustion fixture can now take up to the larger
12,000-tick/150-second bounds to fail. This is an explicit aggregate test
budget tradeoff, not a product performance claim or relaxed resource quota.

## Verification checkpoint

On the corrected source, the 25 peer/oracle host tests pass, including
acceptance at tick 11,999, refusal at 12,000 and 12,001, and preservation of
the other four profiles' 6,000-tick boundary. Rust formatting and
`git diff --check` pass. These checks do not execute the guest.

Native exhaustion and regression execution on this correction: **NOT YET
EXECUTED** at this checkpoint. The parent integration must run its final
combined-source gates and obtain fresh hosted CI; earlier component and
local passes do not verify this correction.

Local bounded host logs: `build/issue39-ci-watchdog-checks/` in worktree
`issue39-ci`. The original CI failure remains retained under
`/tmp/baryon43-ci-exhaustion/` and in the linked GitHub Actions run.

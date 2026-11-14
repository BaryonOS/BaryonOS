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

## Executed correction checks

On the corrected source, the 25 peer/oracle host tests pass, including
acceptance at tick 11,999, refusal at 12,000 and 12,001, and preservation of
the other four profiles' 6,000-tick boundary. Rust formatting and
`git diff --check` pass. These checks do not execute the guest.

The five native cases then passed on commit
`6fe4f9957ea47d75c0f18d9cd6a12b29f733f702`, source-v3
`612fde4baae272d0591419ffc7a427966bcae4d0f20c364ee342078c14e0bd4d`.
No executable source changed during this matrix. QEMU 10.0.13 ran with
explicit machine `pc-q35-8.2`, single-CPU TCG and disposable OVMF state.
This matches the CI machine profile but not CI's QEMU 8.2.2 binary.

| Case | Exit / raw QEMU | QEMU seconds | Recorded host timeout |
|---|---|---:|---:|
| exhaustion | 0 / 33 | 56.726 | 150 s |
| normal | 0 / 33 | 6.808 | 90 s |
| client-exit | 0 / 33 | 6.546 | 90 s |
| revoke | 0 / 33 | 6.684 | 90 s |
| device-reset | 0 / 33 | 2.648 | 90 s |

App A exited at tick 5,491 in the corrected exhaustion run, after B had
closed/exited at 5,453. All 12,768 pairs and the original cross-owner,
permission, raw-packet and cleanup checks passed. This local run did not
cross tick 6,000; the unit tests and source review verify the new exact
boundary, while a new hosted run must verify completion on the slower CI
environment. The local pass alone does not establish that the CI failure
is resolved.

Each case ran `python3 -B -u tools/run_network_service.py --case CASE`, with
`CARGO_BUILD_JOBS=2`, `NOS_QEMU_MACHINE=pc-q35-8.2`, and separate
`NOS_BUILD_DIR=build/issue39-ci-CASE`,
`NOS_TARGET_DIR=target/issue39-ci-CASE` and matching `CARGO_TARGET_DIR`.
Every directory retains `network-service-CASE-report.json` and the original
`run/network-service-CASE-*/{result.json,serial.txt,stderr.txt,network-peer.json}`;
the exhaustion run also retains its raw `network-discard.bin`.

| Original native `result.json` | SHA-256 |
|---|---|
| exhaustion | `17637989ff586cd9f779f715cf1a682f43a38a9f3b1883ac0f412681694d53f7` |
| normal | `185a7a5fd62bed9551a9c31c1f5e450a90aec29349618c2f73c7d55747cd3550` |
| client-exit | `8ba1ae093b96ccda6c846c6c0b707ba64a70cb995f480f605dead9fe56937323` |
| revoke | `9eada9da6c0236b763d0aaab1e9077da35df4a43936249aa26f84286a9ffd954` |
| device-reset | `941b09af2c5f26f0108fbd9611d03683328748665da4f75383e461bcdbc4c755` |

The corrected exhaustion serial hash is
`b301a7f802d136ec457f674a58b8e816419231c4c01c45501d4c3f55ee8e0f56`;
its raw stream hash remains
`7d5e22e621d28ce27dfa3f2ab12d851e07a0632681ede12343d5bc5676f035f2`.

The parent integration must run its final combined-source gates and obtain
fresh hosted CI. This component matrix is not a full OS gate, a native
performance guarantee, physical-device evidence or proof of automatic NIC
recovery. The existing two unrelated kernel warnings remain visible in the
native build logs.

Local bounded host logs: `build/issue39-ci-watchdog-checks/` in worktree
`issue39-ci`. The original CI failure remains retained under
`/tmp/baryon43-ci-exhaustion/` and in the linked GitHub Actions run.

## Integrated and hosted confirmation

The integration and all five component runs have identical runtime source-v3
`612fde4baae272d0591419ffc7a427966bcae4d0f20c364ee342078c14e0bd4d`.
Differences from the component commit are documentation only. Strict preflight,
build, smoke and both TCP loss/cold-recovery stages passed on published
`e17a32130de29c681336dc0c4dec98c204b46260`.

The pinned QEMU 8.2.2 hosted [network run 43601931863](https://github.com/BaryonOS/BaryonOS/actions/runs/43601931863)
then passed the complete five-case matrix on merge source
`9df3d7d0b6f06c1488a0dccedf9581eb1158fa76`, with the same runtime fingerprint.
The exhaustion guest ran for **100.061 seconds**, released B at tick **9,634**,
observed B exit at **9,654**, and A exit at **9,684**. All 12,768 pairs, exact
service counters, original permission checks and cleanup passed with raw exit 33.
This successful run exceeds both former aggregate limits, confirming why the
profile-specific correction was needed without claiming a performance guarantee.

The other cases passed with their original 90-second host default. The 150-second
exhaustion default and strict 12,000-tick deadline remain finite; a genuine stall
may take longer to report. [Integrated evidence](integrated-final.json) preserves
report hashes, snapshots, observed counters, the hosted artifact digest and the
separate original failure. The compact hosted artifact omits raw packet/image
files; independent local raw-packet validation is a separate evidence layer.

# K06 — Multiple processor cores: acceptance

## Accepted property

SMP scheduling, synchronization, interrupts and address-space switching work under
concurrent load on the supported QEMU profile. Accepted on branch
`ccr-5a0d9559-16gxmt` (PR #317) under the rule in [PROGRESS.md](../PROGRESS.md): the
required native gate passed on the PR head with version-specific evidence.

Tested commit: `880984bb29c316e606490c0fa235b119ed4bb553`.
Source fingerprint (source-v3): `e693759d89705970ecaa9c315b0423261fdc7924235af566beb363419eabd255`.
Receipt: [`k06-smp-sched/acceptance-43429991481.json`](k06-smp-sched/acceptance-43429991481.json).

## Executed checks

GitHub Actions **native-smp**, run
[43429991481](https://github.com/BaryonOS/BaryonOS/actions/runs/43429991481)
(`workflow_dispatch`, full matrix, dispatched once with the project owner's permission,
first attempt, not cancelled, no job retried): 91 jobs, 86 succeeded, 5 skipped by design
(three diagnostic jobs, and `native-smp` and `experimental-m3-m4-smp`, which run only outside
a full-matrix dispatch), none failed.

| Check | Result |
|---|---|
| Section 9 matrix | 74 cases × 50 on one source hash: 73 per-case jobs 50/50 and the soak case in ten shards of five; 3 700 of 3 700 guests pass |
| K07 environment probe | passed |
| Aggregate | "Require exact 50/50 evidence on one source hash" and "Complete source-bound SMP host oracles" both succeed |
| Final repository gate | `tools/gate.sh` succeeds: host tests, clippy, fmt, then 26 native guests in isolated build directories (M2 7, K05 6, K05-SMP 6 on four vCPUs, K07 2, RTC 5) |
| `verify_artifacts.py` (read-only) | exit 0: 84 reports, 3 700 runs independently re-evaluated, aggregate and environment probe verified; 87 artifact ZIPs match their API digests |
| `verify_final_gate.py` (read-only) | exit 0 on the retained final-gate artifact and the raw REST run and job responses, after the correction below |

The 74 cases combine positive profiles with negative controls that must be detected.
Positive: one to eight CPUs and a nine-CPU capacity case; M3 scheduling with IPC,
migration, load, kill, unmap, FPU state and recovery; M4 quotas; the native quantum
workload; a soak. Negative: deaf or stalled APs, missed IPIs and TLB shootdowns, lost
updates and stalls in synchronization and scratch, lock-order and lock-in-handler faults,
GS and IST fault paths, quarantine and clock faults.

## Verifier correction after the run

The first readback with `verify_final_gate.py` as tested (sha256 `c310b372…`) exited 1:
"missing or additional native guest evidence". Since #296 the verifier checks the K05-SMP
stage (six guests) but kept the pre-SMP literal of 20 raw guest results; its own requirement
that every raw result is consumed could only hold at 26. Commit `d0d2e30` corrects the
literal and the summary. The consumption check itself is unchanged. On copies of the
artifact, a removed K05-SMP result and an extra unconsumed result each still make it exit 1.
The CI final-gate job had succeeded; `docs/` lies outside the source fingerprint.

## Margins

| Case | Per-boot host time | Bound | CPU model of the job |
|---|---|---|---|
| `m3-ipc-four` | 201.3–224.1 s (median 207.6 s) | 264 s | AMD EPYC 7763 |
| `m3-ipc-four-single` | 93.8–116.2 s | 264 s | AMD EPYC 9V74 |
| `m3-load-eight` | 42.4–43.4 s | 125 s | not recorded |

In the previous full matrix `m3-ipc-four` peaked at 158.3 s on an Intel Xeon Platinum 8573C.
The hosted 4-vCPU pool is heterogeneous, and the IPC margin depends on the CPU model a job
lands on: 85 % of the bound on the slowest model observed.

## Limits

- QEMU/TCG on GitHub-hosted runners only: no KVM, no physical hardware, no other
  supported-hardware profile.
- 3 700 clean guests bound the per-guest failure rate below about 0.08 % (rule of three,
  95 %); 50 runs bound one case below about 6 %. This does not prove the absence of rare races.
- Compact artifacts omit executable and firmware bytes; their recorded hashes are
  cross-bound, not rehashed.
- Verification by the implementing AI is not an independent human audit.

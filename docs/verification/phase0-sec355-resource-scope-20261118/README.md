# PR #355 resource-scope enforcement — 18 November 2026

This checkpoint adds fail-closed cgroup-v2 resource enforcement to PR #355 source
`511936850e53d384c36254e4ad43424cf457377e`. The issue-pipeline manifest remains
disabled. No untrusted issue body, agent or gate was executed.

## Workload distinction

The workstation exposes 62,108,440 KiB of usable RAM (approximately 59.23 GiB).
The preceding measured development reference uses `MemoryHigh=6442450944`,
`MemoryMax=8589934592`, `MemorySwapMax=0`, `TasksMax=512`, a four-core CPU quota
and four Cargo jobs. Rust tests, normal and M6c builds, and M1/M6c QEMU smoke runs
all exited 0. Their aggregate cgroup peak was 621,486,080 bytes (592.70 MiB), with
zero high, max, OOM or kill events. That source-bound evidence remains in
`../phase0-sec355-memory-20261118/`.

The 64 MiB profile in `native-resource-results.json` is deliberately restricted to
negative enforcement probes. It is not a normal agent, build or QEMU preset. Its
96 MiB allocation was killed with `Result=oom-kill` and a measured 64 MiB peak.
The same profile enforced 32 tasks, bounded each captured stream to 1 MiB, stopped
a detached descendant and left no transient scope loaded. A 2 MiB workspace
artifact completed successfully, proving that captured-output limits do not impose
a payload-wide file-size limit.

The native gate probe selected the measured development profile and read back the
actual cgroup values: 6 GiB high, 8 GiB maximum, zero swap, 512 tasks and a 400%
CPU quota. A supported normal agent profile is intentionally not activated before
the authenticated model client and its complete isolated workload can be measured.
This prevents either the 64 MiB negative profile or the 64 GiB schema ceiling from
silently becoming a workstation default.

## Enforcement implemented

The sandbox now requires explicit `agent` and `gate` profiles. It pins and executes
the reviewed bubblewrap, `systemd-run` and `systemctl` inodes, enters a unique user
scope, sets memory high/max, zero swap, task and CPU limits, sets
`memory.oom.group=1`, and verifies the live cgroup before bubblewrap starts. A
private receipt descriptor closes before the payload starts, so payload output
cannot forge the measured limit receipt.

Standard output and error are drained concurrently into bounded controller buffers.
Overflow, timeout and exceptional pipe/launcher failures invoke whole-scope
termination and unconditional empty-scope verification. `RLIMIT_FSIZE` is not used;
build and QEMU artifacts therefore retain their workload-specific storage bounds.
The pipeline and gate preserve bounded raw streams and an allow-listed resource
receipt. `Result=oom-kill` takes precedence over textual rate-limit fallback and
remains a failure for ordinary work.

## Evidence and limits

`native-resource-results.json` records eight real systemd/cgroup/bubblewrap cases.
`probe.c` is the exact statically linked payload source. The local raw Token Guard
log and fixture hashes are listed in `checks.json`. The complete tools suite passed
with 453 tests and 10 documented native-fixture skips. Independent source review
found no remaining actionable issue in this write set after exception-safe cleanup,
raw overflow retention and OOM precedence were corrected.

This checkpoint proves process-tree, CPU, memory, task, descriptor, timeout and
captured-output enforcement on the named host. It does not accept PR #355 as a
whole. Kernel-bounded aggregate scratch/staging/export lifecycle, authenticated
model-client provisioning, complete offline gate compatibility and hosted
current-head review remain open. The progress score remains 41/100.

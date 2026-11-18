# PR #355 native resource-boundary checkpoint — 18 November 2026

This checkpoint binds the native sandbox evidence to tracked source
`6dbe373a84929df6c781e5bf2355d61f7b1f39c9`. The checked-in issue pipeline
remained disabled; no model client, generated source or issue body was executed.

## Measured profiles

The native controller read the following values back from the created cgroup v2
scopes before admitting the inner sandbox:

- fixture agent: `MemoryHigh=268435456` (256 MiB),
  `MemoryMax=536870912` (512 MiB), `MemorySwapMax=0`, `memory.oom.group=1`,
  `TasksMax=128`, 200% CPU and 512 file descriptors;
- normal development gate reference: `MemoryHigh=6442450944` (6 GiB),
  `MemoryMax=8589934592` (8 GiB), `MemorySwapMax=0`, `memory.oom.group=1`,
  `TasksMax=512`, 400% CPU and 1024 file descriptors;
- each tested scratch root: 64 MiB and 4096 inodes.

The 256/512 MiB profile is deliberately limited to the disposable fixture. It is
not a normal model-agent, build or QEMU workload profile. The 6/8 GiB gate profile
is the retained development reference for this 64 GiB workstation. Neither profile
disables the kernel OOM killer or relaxes namespace, capability, network, source or
filesystem isolation.

## Native verification

The canary and full native runs both exited 0. The full suite passed 14 tests in
15.137 seconds. It exercised real byte and inode `ENOSPC`, distinct parallel leases
and mount IDs, hidden sibling roots, device and shared-memory bounds, absence of
controller credentials and runtime sockets from the payload, read-only gate source,
inner network separation, descendant termination, and quarantine under injected
termination and cleanup uncertainty.

Both runs started and ended on the same clean tracked commit. Their selected source
hashes were identical before and after execution. The pinned fixture root filesystem
tree retained SHA-256
`5efab7671a3e4670bbc47a10499405b0dace5be50bf0a8041332bfeb7e590db3`.
No scope or cgroup remained, and `cleanup_errors` was empty. `checks.json` records the
exact result and log hashes. An independent AI verifier reproduced the receipt,
source, socket, unit and cgroup checks without rerunning the privileged fixture. This
is not an independent human audit.

The exact-source host checks also passed: 517 tools tests with 15 explicit skips, 28
document-session harness tests, strict preflight, Python syntax compilation and the
Git whitespace check.

## Evidence limit

The storage-exhaustion cases are intentional negative tests. No real kernel OOM kill,
representative model-agent memory load, full build or QEMU workload was run under the
fixture-agent profile, so those workloads are not accepted at 512 MiB. The disposable
rootfs and raw logs remain ignored local evidence under `build/`; their hashes make
this receipt tamper-evident but do not make the private fixture publicly reproducible.
System packages were inventoried but not independently signature-reverified.

Authenticated model-client provisioning, complete selected-gate compatibility,
current-head hosted CI/CodeQL/Code Quality and final Daybreak acceptance remain open.
The manifest must remain disabled and the source-execution hold must remain active
until those conditions pass.

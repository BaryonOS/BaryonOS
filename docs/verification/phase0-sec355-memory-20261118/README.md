# PR #355 workstation memory investigation — 18 November 2026

The workstation reports 62,108,440 KiB usable RAM (approximately 59.23 GiB). PR #355 source `5b946202c9f7323044913a26d76c96e490f5e0e6` has no active MemoryMax/default and retains disabled `sandbox:{}`. PR #357 source `8a32bc582790fa16234dbadf37a0a176de620900` requires explicit limits; its 64 MiB mocked unit fixture is not a normal development preset.

## Negative and positive evidence

The two recorded 128 MiB units deliberately attempt a 160 MiB allocation. Both report `Result=oom-kill` and exact MemoryMax/MemoryPeak of 134,217,728 bytes. They are successful negative enforcement tests, not failed normal agents, builds or QEMU workloads.

The corrected positive reference bundle uses actual MemoryMax 8,589,934,592 bytes (8 GiB), MemoryHigh 6,442,450,944 bytes (6 GiB), swap 0, 512 tasks, 400% CPU and four Cargo jobs. Tests for kernel-core/capability-core/volume-core/update-core, normal and M6c builds, and real M1/M6c guests all exit 0; both guests exit 33. Aggregate cgroup memory.peak is 621,486,080 bytes (592.70 MiB); high/max/OOM/kill counters remain zero. A fresh target directory was used; registry/toolchain and OS page caches were reused.

## Required launcher correction

The initial Snap-based reference run exited0 but did not measure all compiler descendants: `/snap/bin/cargo` moved its child into `snap.rustup.cargo-*.scope`, whose actual memory.max/high are `max`. The fixed synthetic membership assertion fails there (exit 101). Task-local pinned native Rustup proxies keep the same Rust 1.90 compiler child inside the intended scope with actual MemoryMax 1 GiB; that control exits0. The repeated positive bundle uses those native proxies. No global toolchain, PATH, cgroup or security-policy setting changes.

[Summary and limitations](summary.json), [workload observations](native-native-workload-results.json), [source/log hashes](log-hashes.json) and negative unit statuses preserve the raw measured distinctions. Public copies replace only the task-worktree prefix; raw originals remain local.

## Acceptance boundary and continuation

No global RAM limit or OOM-killer adjustment is necessary from this evidence. The 128 MiB negative profile and PR #357's 64 GiB schema ceiling must not become normal presets. A supported runtime must use verified native executables, retain compiler descendants in its resource scope, select explicit workload profiles and reserve host/IDE RAM across concurrent tasks.

The 8 GiB reference is not an unmeasured universal agent/full-CI preset. This run used disk-backed scratch; future bounded tmpfs needs additional data/inode memory budgeting. Authenticated clients, full staging/export and selected complete gates remain unaccepted. The issue pipeline remains disabled. After this bounded investigation, execution resumes at the existing PR359/355/356 and remaining-Daybreak queue; no governance begins. Score 41/100 is unchanged.

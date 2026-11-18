# PR #355 capped scratch transaction checkpoint — 18 November 2026

This checkpoint binds the public scratch and controller-output contract to source
`edd1a6a9af2d80a56848ec500bbf2b702799dced`. The checked-in issue pipeline remains
disabled and no model client or issue body was executed.

## Boundary implemented

- Each concurrent transaction needs an exclusive lease on one dedicated tmpfs whose
  opened mount ID, device, ownership, mode, total bytes and inode count match policy.
- Source and export copies fail before mutation when their byte or file budget is
  exceeded. Workspace, build outputs, temporary directories, homes and `/dev/shm`
  remain on the same hard-capped filesystem.
- `/dev` and its separate non-recursive `/dev/pts` submount are read-only. The capped
  `/dev/shm` bind is the only writable device subtree admitted by this contract.
- Uncertain process-tree or writable-mount cleanup quarantines the lease. The dirty
  root cannot be reused, while later clean verified pool roots remain eligible.
- Pipe failures, output overflow, timeout, nonzero exit and lease-body errors remain
  primary evidence when cleanup also fails; the cleanup error is chained and noted.
- Stdout and stderr share one raw-byte budget across engine fallback or all steps and
  the finalizer of one gate plan. Payload bytes are retained once in bounded raw files.

## Verification

`checks.json` records exact commands, exit codes and retained Token Guard logs. The
exact-source full tools suite passed 513 tests with eleven environment skips. Focused
tests passed 123 cases with ten private-runtime skips. The applicable Python
host-quality groups and 26 workflow-contract tests passed. Strict preflight and the
M6c, DocumentSession, native quantum-job and SDK-app normal/denied QEMU gates passed;
native guest runs returned the expected raw exit 33.

Independent AI security rereview approved the five scoped blockers after fixes for
combined primary/cleanup errors and Bubblewrap's non-recursive `/dev/pts` mount. The
reviewed pre-commit binary diff SHA-256 is
`ab3c8ccab50d0f962d106dc754d67af3a436fa25a2da26df089dfc8849c0e3bd`.
This is not an independent human audit.

## Evidence limit

No private `BARYON_SANDBOX_TEST_CONFIG`, pinned Bubblewrap 0.13.1 rootfs or real
dedicated scratch mounts were available. Native PTY denial, ENOSPC/inode exhaustion,
parallel sibling isolation, live-survivor quarantine, authenticated model clients and
complete selected-gate compatibility therefore remain `NOT RUN`. Separate self-healing
agent invocations each receive a fresh finite transaction budget; `MAX_ATTEMPTS` remains
the trusted fixed whole-task bound. The manifest must stay disabled until these native
and hosted acceptance conditions pass.

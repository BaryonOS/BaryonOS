# Isolation, user mode, scheduling and IPC

Start from [interrupts](../themas/07-interrupts.md), [processes](../themas/09-processen.md),
[synchronization](../themas/10-synchronisatie.md), [IPC/syscalls](../themas/11-ipc.md),
[executable loading](../themas/12-executables.md) and [security](../themas/21-beveiliging.md).

## Define the boundary

Specify the privileged/unprivileged transition, register/stack contract, error format,
user-memory access policy, resource limits and permission model. Match entry/return
frames to the actual CPU mode and exception class. Do not mix an interrupt gate,
software interrupt and fast-syscall mechanism without explicit contracts.

For executable loaders, validate architecture, sizes, offsets, overlaps, permissions,
entrypoint and cleanup on partial failure. Reject malformed input before mapping or
executing it. Do not equate jumping to a loaded blob in kernel mode with user isolation.

## Scheduling and interrupts

Describe task states and legal transitions, kernel/user stack ownership, saved context,
interrupt masking/preemption policy and timer accounting. Keep interrupt work bounded.
Specify whether blocking/allocation is allowed in each execution context. Verify error
code handling, interrupt acknowledgment and exceptional return paths for the target.

Exercise task creation/exit, yield, preemption, blocking/wakeup and task cleanup.
A non-yielding user task must not be mistaken for evidence of a working preemptive
scheduler. Preserve an observable heartbeat or equivalent evidence in runtime tests.

## Concurrency

Write the lock order, ownership rules, memory-order requirements and reclamation plan.
Distinguish single-core interrupt races from true parallel execution. Document whether
an operation may sleep while holding a lock, and what cancellation does to ownership.
Stress concurrent allocation, mapping, wakeups and teardown with reproducible seeds.
Do not treat `volatile` or an isolated timing workaround as a complete synchronization
argument. Use the language and architecture's documented primitives.

## IPC/capabilities

Specify message bounds, handle identity, rights, transfer/delegation, lifetimes,
revocation, cancellation, quotas, backpressure and endpoint death. Avoid raw user
pointers or numeric handles being accepted as authority without the intended checks.
Test stale handles, forged handles, wrong rights, oversized messages, sender/receiver
exit, queue exhaustion, timeout and interrupted transfers. Define which party owns
resources on every return path and verify cleanup.

## Acceptance and assurance

Demonstrate that one malformed or faulting process cannot obtain unintended kernel
access or corrupt another process in the tested scenario. Record the actual test scope.
For a proof proposal, state properties, model, assumptions, trusted components and
unproved areas; a microkernel, Rust code or a test suite does not automatically prove
an entire operating system correct, compliant or invulnerable.

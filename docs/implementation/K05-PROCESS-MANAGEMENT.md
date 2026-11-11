# K05 — general process and service management (profile `native-supervisor`)

Date: 29 October 2026. Status: **design contract, subsequently implemented
in the same assignment**; the executed checks and their outcome are recorded
exclusively in [K05-ACCEPTANCE](../verification/K05-ACCEPTANCE.md). This
document describes the contract; deviations that proved necessary during the
build are recorded in §8. Criterium (docs/progress.json): *non-fixture apps can
start, shut down and restart services via bounded APIs; dependencies
and resources are released correctly.* Decision: ADR-042.

## 1. Problem

Up to and including M6e/S06, only the built-in kernel manifest creates processes
and endpoints, before the first user instruction. A service that fails
does not come back; an app cannot start a helper service; there is no
user-space authority over lifecycle. K05 adds that authority
as capabilities, not as an ambient right.

## 2. Security invariants (first)

1. **Starting is a capability.** A process can only start an image
   for which it holds a `Launch` capability. Without that capability
   `proc_spawn` gives `EACCESS`, `ETYPE` or `EBADHANDLE` and no process,
   frame, endpoint or node is created.
2. **No inheritance.** A child receives exclusively the at most two
   capabilities that the starter explicitly passes along, with reduced rights
   (the same rules as IPC transfer: `TRANSFER` required, never `RECV`,
   never more rights than the source). The rest of the start page is zero.
3. **Child authority is not transferable.** A `Process` capability
   (observe and terminate) cannot be transferred via IPC or
   derived: `allowed_rights` has no `TRANSFER`, `DERIVE` or `REVOKE` for this
   object type. Only the starter manages its children.
4. **Death cleans up everything.** Exit, fault or `proc_kill` of a process releases
   all its frames and tables, revokes all nodes it issued,
   closes its endpoints (`ECLOSED` for peers) and leaves no queued message
   with its capabilities behind. This is the existing `cleanup` path; K05
   only adds the triggers.
5. **Dependency is explicit.** A child belongs to its starter. If the
   starter dies, the kernel terminates all children (and their children)
   iteratively in the same event, without recursion on the kernel stack. No
   process remains over which nobody has authority.
6. **No partial state.** All fallible checks of `proc_spawn`
   (handle, right, free process slot, PID, engine registration, slots for the
   passed capabilities) precede frame allocation; an error after those
   checks goes through the same `cleanup` path as a process death, so that the
   free frame count stays the same.
7. **Quotas remain kernel quotas.** At most 16 processes, 4 own endpoints
   per process, 64 endpoints, 1024 nodes, 256 slots. `EQUOTA` changes nothing.
8. **A handle is not a secret.** A `Process` or `Launch` handle of
   process B is in A at most an own slot of A (existing KS-12 rule).

## 3. Interface (syscall ABI 1.3, only in this profile)

| No. | Name / arguments | Success (RDX) | Errors |
|---|---|---|---|
| `0x30` | `endpoint_create()` | root handle with `ALL_RIGHTS` | `EQUOTA` (4 own / 64 global / slots / nodes) |
| `0x50` | `proc_spawn(launch, arg, cap0, rights0, cap1, rights1)` | `Process` handle with `OBSERVE\|KILL` | `EBADHANDLE`, `ETYPE`, `EACCESS` (no `LAUNCH`, no `TRANSFER` on cap, `RECV` requested), `EINVAL` (unknown rights, `cap0 == 0` but `rights0 != 0`), `EQUOTA` |
| `0x51` | `proc_kill(process)` | 0 | `EBADHANDLE`, `ETYPE`, `EACCESS` (no `KILL`), `ECLOSED` (child already dead) |
| `0x52` | `proc_status(process)` | `(state << 56) \| (code & (2^56 − 1))` | `EBADHANDLE`, `ETYPE`, `EACCESS` (no `OBSERVE`) |

`state`: 1 = running, 2 = exited (code = exit code mod 2^56), 3 = faulted
(code = exception vector), 4 = killed. A `cap0`/`cap1` of 0 means "no
capability"; the corresponding rights field must then be 0. Unused
arguments remain zero per ABI v1.

New rights bits: `LAUNCH = 128` (only on `Launch` objects),
`OBSERVE = 256`, `KILL = 512` (only on `Process` objects). `KNOWN_RIGHTS`
becomes 1023. `derive`, `revoke` and IPC transfer of these objects fail with
`EINVAL` because the object type does not know those operation rights.

### Start page of a child (`0x40010000`, 1024 bytes, rest zero)

| Offset | Content |
|---|---|
| 0 | `arg` of the starter (free word, no authority) |
| 8 | 0; the child uses this as a progress counter (existing convention) |
| 16 | kernel pattern (existing audit) |
| 32 | 1 (start epoch) |
| 56 | handle of `cap0` in the child's table, or 0 |
| 64 | handle of `cap1`, or 0 |
| 176 | own PID |

Empty positions are not compacted: with `cap0=0` and a valid `cap1`
offset 56 stays zero and the delegated `cap1` is at offset 64. The number of
non-empty capabilities determines only the resource preflight. This existing
contract was enforced with a native regression after the P2 review of #51.

## 4. Kernel objects

- `Launch { owner, image }`: created per manifest, `image` points to a
  built-in ELF (0 = worker, 1 = client). Cannot be created by apps.
- `Process { owner, child_pid, state, code }`: created by `proc_spawn`,
  the owner is the starter. When the child dies the kernel sets `state`/`code`;
  the object persists until the owner closes its last handle. On
  exit of the owner the kernel closes the object and the child, if it
  is still alive, is terminated (invariant 5).

The `Engine` type in `capability-core` gets both object kinds after the
existing endpoint and block objects (`index ≥ 68`). All existing rules
(generations, tombstones, `maintain`, `check_invariants`, `state_digest`)
apply unchanged; host tests cover the new transitions.

## 5. User programs (SDK example `sdk/examples/supervisor`)

- **supervisor**: receives two `Launch` capabilities from the manifest and
  creates a registration endpoint itself. Starts the worker and the client with
  a `SEND` capability on that endpoint. Worker and client register
  by sending back a `SEND` capability on their own endpoint. The
  supervisor gives the client the worker capability, monitors the worker with
  `proc_status`, restarts it (after `proc_kill` in the normal case, after an
  observed fault in the crash case) and publishes the new capability
  again. At the end it terminates the children and checks `killed`.
- **worker**: service with counter; answers requests with sequence number and
  instance number. In the crash case it executes `ud2` after eight answers.
- **client**: sends 24 requests; on `ECLOSED` on the old capability it waits
  for the new one and continues with the same sequence number. Counts answers,
  `ECLOSED` observations and instances.

All three are ordinary SDK programs without block, device or
network rights. The kernel does not know their protocol.

## 6. Acceptance cases (real QEMU runs, each its own kernel feature)

| Case | Feature | Required observation |
|---|---|---|
| normal | `native-supervisor` | start worker+client; controlled restart after 8 answers; client 24 answers, ≥1 `ECLOSED`, 2 worker instances; all children `killed`/`exited`; frames and engine at baseline |
| crash | `test-supervisor-crash` | worker fault (`#UD`) after 8 answers, `proc_status` = faulted(6); restart; client 24 answers; baseline |
| denied | `test-supervisor-denied` | client/worker attempt `proc_spawn`, `proc_kill` and `proc_status` with endpoint, zero and forged handles and `endpoint_create` beyond the quota; exactly the recorded error codes; the kernel counts refused spawns and sees exactly 2 started processes |
| parent-exit | `test-supervisor-parent-exit` | supervisor exits while worker and client are alive; both are terminated in the same event; baseline |
| quota | `test-supervisor-quota` | supervisor starts workers until `EQUOTA` (15 succeeded), free frame count unchanged by the failed attempt; all 15 terminated and `killed`; baseline |

Kernel markers: `SUPERVISOR:MANIFEST:PASS`, `SUPERVISOR:CASE:<GEVAL>:PASS`,
`SUPERVISOR:RESOURCES:PASS`, `SUPERVISOR:PASS`, `M1:PASS`, exit code 0x10
(raw 33). Any `SUPERVISOR:FAIL:*` or `PANIC` fails. Numeric evidence as
`SUPERVISOR:<NAAM>:<20 cijfers>`. The gate `tools/test_supervisor.py` first runs
the host tests (`capability-core`, SDK runtime, Python harness) and
then the five runs on one frozen source fingerprint.

## 7. What K05 does not prove

No SMP, no real hardware, no loading of user-supplied images
(A02), no persistent service configuration, no priorities or deadlines,
no protection against a malicious supervisor that misuses its own children
within the rights granted to it. The supervisor is one ordinary
process; a system-wide service manager with policy follows later.

## 8. Deviations during implementation

- **Frame baseline and kernel page tables.** The identity map of newly
  touched RAM requires permanent `PageTable(0)` frames (M2 contract). With
  16 concurrent processes (quota trial) the kernel table grew by one
  frame. The final check therefore requires `vrij + gegroeide kerneltabellen ==
  baseline` and prints the growth as `SUPERVISOR:KERNEL_TABLES_GROWN`; a
  real user frame leak still fails.
- **Denied case.** The client in check mode exits on its own after its report;
  the supervisor observes `Exited(0)`, closes the handle and checks that the
  number then gives `EBADHANDLE`. The kernel therefore expects one kill (worker)
  and one child exit (client) instead of two kills.
- **Start word 0** carries, besides the mode, also the PID of the starter (from
  bit 8), so that worker and client accept control messages only from their
  supervisor. This is app protocol, not kernel authority.

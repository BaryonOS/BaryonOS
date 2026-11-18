<a id="ipc-wait-1-begrensd-wachten-op-bestaande-ipc-rechten"></a>

# IPC-WAIT-1: bounded waiting under existing IPC rights

Working baseline: `9596abc576fceefe2b3ff2f3bd120bcd914eb04c`. This is an opt-in
mechanism and SDK contract, not signed-off GUI or release acceptance.
The component source is prepared for integration review; executed checks
and open gates are listed below. This component agent has not performed
a native test.

<a id="integratie-op-actuele-main"></a>

## Integration on current main

The integration branch `simon/native-test-version` uses baseline `ad2b981`.
The historical component described below has been incorporated, not
redesigned. The wait interface receives **ABI 1.10**: ABI 1.9 has since been
assigned to the untrusted RTC clock by ADR-050. That clock and the extended
userreturn/stack validation from main remain intact. The historical
component measurements below do not count as new native acceptance.
`native-ipc-wait` is a separate two-process test; product profiles
remain unchanged for now. The integrator adds a native fixture
before connecting waiting to the existing GUI user chain.

<a id="invariant-en-abi-110"></a>

## Invariant and ABI 1.10

All existing SEND/RECV, copy, capability, generation, IPC and
digest checks remain intact. Waiting neither reserves nor consumes anything
and sends no message. A ready notification guarantees only that the queue
has space or data, respectively, at that moment. An actual subsequent send or
receive validates everything again and may return EAGAIN, Quota or another error.
No new right, object, node, physical mapping or authority is created.

Kernel feature `ipc-wait` is off by default and requires milestone-m4 plus
monotonic-clock. Only those profiles report ABI 1.10. Existing profiles
retain their ABI, behavior and terminal idle test. ABI 1.0–1.9 remain
accepted in the SDK; an older profile does not support syscall 0x22.

```text
ipc_wait(items_pointer, count, absolute_tick, 0, 0, 0) // syscall 0x22
entry = { handle: little-endian u64, interest: little-endian u64 }
interest = 1 SEND of 2 RECV, nooit beide of andere bits
0 <= count <= 16; maximaal 256 gekopieerde bytes
count == 0 vereist items_pointer == 0
toekomstige afstand <= 1000 PIT-ticks, nominaal 10 seconden
RAX == 0, RDX == ready-mask; bit i hoort bij entry i
RDX == 0: deadline verstreken zonder ready entry
RAX < 0: bestaande kernelfout; RDX == 0
```

English reading of the preserved ABI example: `interest` is either 1 (SEND) or
2 (RECV), never both and with no other bits. At most 16 entries and 256 copied
bytes are accepted. A zero count requires a null pointer. The future deadline
distance is at most 1,000 PIT ticks, nominally 10 seconds. With `RAX == 0`,
bit `i` in `RDX` identifies ready input entry `i`; a zero mask means the deadline
expired without a ready entry. A negative `RAX` is an existing kernel error and
requires `RDX == 0`.

The entire input is validated before registration and converted into a bounded
kernel copy. No CopyPlan, user pointer, user buffer, message, object index
or node reference is retained in a wait record. The normally saved
user registers retain their existing meaning; on resumption, they are
not dereferenced again as copy input. The wait table stores only
its own PID, full handles with generation, interests and absolute time values.

All conditions undergo the existing check_endpoint validation: live PID,
slot handle/generation/node, Revoked, Closed, type, rights and RECV ownership.
An earlier ready item never hides an error in a later item. Errors make
the waiting process context runnable again with that error and zero as the
second result. The existing error order remains in force. The same handle may
occur more than once; each result bit remains bound to its input position.

After full capability validation, actual readiness takes precedence
over a deadline that has since expired. If nothing is ready, an expired
deadline returns zero immediately. Only a still-future distance greater than 1000
is rejected as Invalid; ordinary syscall delay does not turn an expired
short wait into Invalid. The deadline is never silently renewed.
A kernel clock moving backwards or an incorrect PID binding does not yield success.

The new read-only core query checks `len < 16` for SEND and
`len > 0` for RECV. The same authorization checks are repeated at every scheduler
evaluation. Close, revoke, process-exit and queue maintenance can therefore
also wake a process without send/receive. Revocation can remove queued cap messages
and free SEND space. The last SEND-close is not EOF; existing messages
without transferred caps remain valid under the existing contract.

<a id="scheduler-echte-idle-en-interruptgrens"></a>

## Scheduler, real idle and interrupt boundary

The scheduler has an optional Waiting state with its own copied
wait record. The current syscall makes the process Waiting only after registration
is complete. The iterative selection loop evaluates all waiters before
choosing a Ready process. A signal before registration is seen in the current
queue; on one CPU with IF=0, no other Engine mutation can occur between the
check and parking. There is no separate check-then-enable wakeup registration.

If only waiting processes remain, the existing
supervisor root and permanent service stack are used with one contiguous
`sti; hlt; cli`. The existing CPL0 IRQ0 route touches only raw idle counters and
the interrupt frame, never the borrowed State or Engine. After CLI, the
idle route is disabled and the actually counted PIT delta is added exactly once
to the monotonic clock, IRQ counter and idle metric. No estimation of
lost ticks, wall clock or certificate time. The same brokers and
observers then run, after which the selection loop evaluates again. No recursive
idle→select→idle loop. A wake with zero PIT delta does not automatically count as
a successful timer wait; that deadline remains in force.

This profile enables no driver IRQ. The current PIC permits only IRQ0;
PS/2 IRQ1/12 are also disabled in the controlled device profile.
No fictitious input or virtio interrupt is therefore used as a wake source.
The Source may sleep until the next PIT tick only after real RawIdle, and
must keep processing available bytes until the device is actually empty.
Pending reset or broker work retains the existing bounded progress paths.

Processes retain their own mappings, kernel stack and 22-word context while
sleeping. No user-pointer or DMA lifetime extension is added. Existing
FPU entry-save and terminal restore remain unchanged; idle-IRQ0 from CPL0
performs no user FPU restore. Kill and cleanup remain possible for
Waiting; Process::new clears the wait record on death and slot reuse.

The existing m4 error check remains exactly intact. The new wait registration
itself compares the full Engine-Stats before and after the serialized
registration segment, including on success. **No** global digest is compared
across the sleep interval: other processes may legitimately modify the Engine during it.
Readiness queries have only `&Engine` and change no Engine field.

<a id="sdk-en-integratiegrens"></a>

## SDK and integration boundary

```rust
WaitCondition::send(SendEndpoint)
WaitCondition::receive(ReceiveEndpoint)
native::ipc_wait(&[WaitCondition], absolute_deadline) -> Result<u16, Error>
```

The SDK bounds count, creates the wire copy and rejects result bits outside count.
An empty list is only a timer wait for the process itself. Consumers retain their own
absolute protocol deadline and choose, for example, the earlier of that deadline
and the next necessary poll. No automatic IPC replay,
deadline extension, guessed Received or host execution. Sixteen conditions
cover, among other things, the four Policy inputs, eleven bounded reply routes and
RegistrySEND. A daemon must not sleep while it still has local rendering,
erasure, decoding or other immediately runnable work.

Consumer changes, exact ABI checks in existing daemon ELFs, profile aliases,
builder, bootstrap, native fixture and runner belong to the integrator. This
component does not deliver already integrated or tested daemon acceleration.
For a blocking syscall, the existing max_syscall_cycles metric also includes
the intentional wait; it must not be presented as active kernel computation time.
The new internal wait metrics distinguish calls, registration,
immediate results, wake results and actually counted idle-IRQ0s.

In the integration with B02/main (`f52dab5`), the Key Service retains the
existing poll/yield path when RANDOM or the administrative ingress
is present. Only the two legacy domain inputs are part of the
current quiescent wait set. Sleeping on those two while an admin request
is pending would unnecessarily delay the existing administrative route until the wait deadline.
The normal Materiaal service without these optional inputs retains waiting;
this adds no capability or automatic administrative authority.

<a id="verplichte-negatieve-en-native-acceptatie"></a>

## Required negative and native acceptance

- Pure core and wait tests: empty/full/wrap, wrong rights/type/PID/owner,
  stale handles, revocation, last RECV-close, process-exit, reuse,
  all items validated, no queue/node/handle mutation, ready versus Quota.
- Wire/SDK: count 0/16/17, reserved args, invalid interests, high handles,
  overly distant/expired deadlines, clock rollback, excessively high result bits and
  full pointer validation including an unreadable second input page.
- Native empty RECV→send→wake and full SEND→drain→wake, with real bytes and
  actual syscall results; immediate readiness before registration.
- Native full SEND queue with a waiting sender: the owner revokes SEND derived from
  its root while the queue remains full. Requires Revoked before the deadline,
  without queue progress and without false-success/replay.
- Native blocking of all processes at once, including count0 sleep:
  real STI/HLT, PIT counting, Sys3 progress and correct scheduler resumption.
- Native kill of a waiting process, real cleanup and slot reuse without
  an old result; FPU and IPC isolation remain checked in relevant regressions.
- Then the unchanged full GUI action and legacy gates on the exact
  integrated commit, configuration and artifacts. No timeout extension.

The separate native fixture requires prior contract review and is
wired and run by root. Until that evidence exists, native wait and
performance acceptance remain open, with zero independent release points.

<a id="uitgevoerde-componentcontrole"></a>

## Executed component checks

The separate lifecycle case `test-supervisor-wait` uses only the
existing K05 Launch/Process/SEND/KILL/OBSERVE rights. It must terminate two real
Waiting children, reject Process-derive without DERIVE and an endpoint-kill,
and reject the old status/kill handle after clean
PID/slot reuse. The new wait
must remain intact beyond the upper bound of the old deadline. The observer
changes neither process status nor wake: it checks the normal spawn/kill/
cleanup outcome and requires an empty Engine and full frame restoration.
All six K05 cases and their host oracles remain required. Native execution
passes on `586c33c`: two Waiting kills, clean slot reuse, four
denials, four blocking events, two deadline wakes and 84 idle IRQs;
all six cases raw33. Process-derive requires exactly EINVAL (DERIVE is
invalid for that object class), endpoint-kill ETYPE and both stale
Process actions EBADHANDLE. Source/hash binding and three retained failed
fixture attempts: `verification/native-test-version/wait-lifecycle-evidence.json`.
This closes no full rubric or release criterion.

The integration also adds a separate `test-fpu-wait` test:
the existing real x87/SSE apps replace their yield with
a bounded count0 deadline wait only on ABI1.10. All register comparisons,
timer preemptions, AVX app fault, survivor, PID3/slot reuse and cleanup remain
required. The native oracle requires real blocking events and idle IRQs, exclusively
deadline wakes and exactly one wait call per verified loop. The normal
ABI1.6 profile remains separate. This tests context preservation around blocking;
it is not evidence for killing a process that is still Waiting. Execution status and
exact source are in `verification/native-test-version/`.

All 47 original core tests remain intact. Executed:

| Check | Actual outcome |
| --- | --- |
| Core default | 57 passed, 0 failed, 1 explicit benchmark ignored |
| Core diagnostic-bench | 58 passed, 0 failed, 1 ignored |
| SDK-runtime | 22 passed: 5 unit, 14 message and 3 process cases |
| Actual pure WaitSet decoder/evaluator | 7 host tests passed |
| Clippy core, SDK and WaitSet host harness | Exit 0, all targets, `-D warnings` |
| SDK scalar-none release build and Clippy | Exit 0 |
| Kernel native-system + ipc-wait + fpu-context | Scalar-none release build exit 0 |
| Kernel native-system + fpu-context, without ipc-wait | Scalar-none release build exit 0 |
| Kernel Clippy, both configurations | Exit 101, exactly the same 12 existing findings |

The twelve kernel findings are in unchanged keyring, volume, paging,
main and three existing m3 lines. The structured lint sets have been
compared; no new wait finding and no suppression to declare this check passing.
This general kernel check remains failing. Core and SDK Clippy
are separately confirmed passing. The small `never_loop` annotation on select
applies only when ipc-wait is off: the old terminal idle exit then remains
in place; it suppresses none of these twelve findings or any security check.

The WaitSet tests compile the actual pure decoder/evaluator from wait.rs in a
separate host harness, without a native adapter. They execute no IRQ, scheduling or
usercopy. The test cases cover all sixteen interests, scalar bounds,
absolute deadline/clock rollback, later invalid caps after earlier readiness,
copied input, real Engine queue changes, revocation while full and
multiple senders without reservation. Native adapter, idle, PID lifetime and
context switching have also actually been built for the freestanding kernel.

The kernel compilation uses five controlled, locally copied ELF inputs
from the existing cap-bench bundle. These are only link/type-check inputs; their
old ABI is not presented as successful 1.9 service execution. No
QEMU was started. The integrator builds the actual new fixture/consumer ELFs.

The source review discovered a concrete observer recursion: sdk_app and qsim_job
called select again from poll. The four narrowly approved returns
(sdk_app two, qsim_job one, m4 lifecycle one) now return None and let the
single m3 event/select loop schedule. Their M4 profiles do not enter an old
M3 fixture branch. qsim.rs remains unchanged; its combination with ipc-wait is
rejected by the existing milestone-m4 compilation condition. Targeted rereading
found no remaining concrete blocker. This is AI source review, not a human
audit or native acceptance.

The full production digest body, Stats body, m4 syscall body and arch.rs have
remained literally identical to baseline 9596abc. The existing 22-word layout,
entry-save, userreturn validation and terminal FPU restore remain intact.
In the built kernel, STI/HLT/CLI are actually directly adjacent;
the resumable idle route clears the idle flag immediately after CLI. The old terminal
idle route remains separately present. This is instruction inspection, not execution.

```text
componentbron v3 2c793f9314f8a47ca4d4b784fff2fa84509a3c860722ede8c4d301c227690289
core lib.rs SHA256 ce50c2e7838e07da07523952b2749872ea154127930d4248c3f2827e673e8546
m3.rs SHA256 7a4b67cfcfb5111d9facb5f80d9c7b2f93c0d38ed4bc0f5803d2c2fb63a534d9
wait.rs SHA256 678709d0d822f4c21a3bba76f691cb2905f00635b96c1797fe61d0a60b20e88d
compile-only kernel SHA256 78762586bab6dc7ff362afe66afcdc5470b5cab0a577f1ceb29939d3c65c9d45
```

Component logs and compact source binding: `build/ipc-wait-checks/host-report.json`,
`target-report.json`, `final-target-report.json`, `source-binding.json` and the
corresponding separate stdout/stderr. All output is under the component's own
worktree/target. Rust 1.90.0, offline/locked where a lock exists. Scalar flags:
code-model=kernel, relocation-model=static, no-redzone, SSE/SSE2/SSE3/SSSE3/
SSE4.1/SSE4.2/AVX/AVX2 off, explicit path remapping and existing linker boundaries.
The new public SDK signatures are available to independent consumers;
their own host, target and native acceptance remain separate.

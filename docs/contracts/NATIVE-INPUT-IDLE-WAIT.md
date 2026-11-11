# Input services wait for actually missing work

INPUT-IDLE-WAIT-1, implementation based on 9596abc. This change lowers the
runnable poll load of the existing separate CPL3 input source and policy.
It decodes no new input, changes no token/ACK/IPC contract and
does not enlarge any kernel or device authority. Host tests and target builds are
not native performance or GUI acceptance; root runs that gate separately.

## ABI and ownership

On integration base `ad2b981` only the exact reported ABI 1.8
and 1.10 are accepted. ABI 1.9 is taken by the existing RTC clock and must therefore
not be recognised as a wait interface. 1.8 keeps
the existing nonblocking IPC/yield loop. Only known 1.10 uses
`native::ipc_wait(&[WaitCondition], absolute_tick)`. Other minors/majors and
unknown version values fail; no numeric forward-compatibility claim.
The SDK and kernel implement that boundary elsewhere, not in the input services.

`WaitCondition::send(SendEndpoint)` and `::receive(ReceiveEndpoint)` borrow
only existing grants. At most 16 conditions, at most 1000 ticks
ahead; an empty list is a timer-only wait. All conditions are
revalidated. Closed/Revoked/BadHandle are errors, not ready bits;
they wake fail-closed and also hide any other ready bits. A bit
reserves or consumes nothing. Ordinary IPC after wake revalidates.
The caller holds no reference to another address space.

The read-only plans live in `source::wait` (the module physically lives in
`sdk/examples/input-service/src/wait.rs`). No SDK/kernel/lib/Cargo interface
is changed by this task. Exactly duplicate interests are merged,
different send/receive interests are not. Capacity/clock overflow fail;
there is no heap or unbounded collection. The native adapter performs at most
one wait call and treats its mask exclusively as a wakeup hint.

## Source: local work before waiting

| Actual state after one existing bounded step | Plan |
| --- | --- |
| Begin; reset start/poll Pending; new SYNC/Sample; real Ready/SampleAccepted/Recovering | Runnable/yield: perform next local transition, no sleep |
| Unsent request and actual send-EAGAIN | Only request-SEND; the same outgoing bytes and original exchange deadline |
| Request actually queued and receive-EAGAIN | Only response-RECV; never send again because a wake bit ACKs nothing |
| Actual raw `read()` returns Idle/None | No IPC interest; timer until original observation tick+1 |
| RegistryReady newly created | Runnable; first actually try to send |
| RegistryReady send-EAGAIN | Only registry-SEND, original ready deadline |
| Fatal | Existing cleanup/exit; never wait or retry |

IRQ1/12 does not yet wake this wait. The raw PS2 Idle wait therefore lasts at most
until the next PIT tick; no claim of interrupt-driven input. A
reset-Pending is not Idle and does not get this timer optimisation. If the
tick has already passed during a syscall/preemption, immediately run the next
bounded step and do not add a new tick.

Exchange and Ready keep their existing `now-started > 200` boundary: the
wake boundary is checked `started+200+1`. The existing reset boundary remains >500.
No wake, EAGAIN, new plan computation or preemption renews started or
this absolute deadline. An expired deadline first leads back to the real
state machine, not to a fresh waiting period.

## Policy: only usable routes

An existing pass first checks lifecycle, focus and source ingress,
then at most 2 app requests and every retained reply exactly once.
Waiting is only allowed after that complete pass. Until the four real
receive rights have been checked, the process remains runnable for the
existing Ready preflight.

The plan contains source/compositor/supervisor RECV only if the relevant
route has no retained reply and is therefore ingestible again. App RECV keeps
its existing bounded drain/reject policy. Add SENDs exclusively for
the actually retained replies (at most 11 with 8 clients), not for all
possible reply endpoints. A stale reply that needs local recomputation
keeps the process runnable. A new message that arrives after the last check
is seen directly by the atomic level-triggered wait; there is
no lost-wakeup window between host check and sleep.

The policy introduces no new RPC timeout. An idle wait has a
bounded 1000-tick horizon; existing protocol deadlines of the peers remain
leading. If its own RegistryReady actually returned EAGAIN, add its SEND
and shorten exclusively to the original ready deadline. All
plans stay within 16 conditions. A turn budget that has been reached cannot hide a
locally retained answer: all retained replies have already been tried
and all other kernel queues are in the plan as eligible receive conditions.

## Preservation and verification

Plans read status; they allocate no request ID, change no frame,
token/cancel generation or pending reply and fabricate no input/ACK. Send-EAGAIN
is not enqueue; after a real enqueue nothing is republished. Wake can
again yield EAGAIN. A source wait error in ABI 1.9 goes to existing cleanup
and visible failure. Policy keeps the existing per-app isolation: only
Closed/Revoked/BadHandle from the wait call are rechecked once via the ordinary bounded
IPC pass. Only an actually newly retired app lane
explains such an error and allows further waiting. Critical, unexplained, clock,
yield, mask or other wait errors remain fatal; no silent fallback to
poll success or ABI 1.8. 1.8 gets no new extra wait clock conditions.

Acceptance per component: real Engine envelopes plus Ipc error counterparts
test eligible RECV, full reply queue/SEND, new local work, stale/cancel,
reply loss/Closed, cleanup and original deadlines; malformed ABI,
max 16/duplicates/overflow get negative checks. Then existing
input daemon host tests, Clippy/fmt and real scalar freestanding ELFs with the
frozen SDK 1.9. Only on one integrated image does the main gate measure real
QMP keyboard/mouse, unchanged focus/Save/reopen/cleanup, the absolute 200 s boundary
and poll/wait load. No host test or claim of fewer syscalls replaces that
native execution; all full release criteria remain open.
## Historical component check, not native integration evidence

Source freeze `2b286d2837b1a8760865c6c2c39bd2e1ea0389a5`, dependency exactly
`8a8dbe53acf5c8c7882244ddf7f753a8645a1d0a` (locally as unchanged
dependency cherry-pick `ae3f022`). Existing root/SDK/kernel files were
taken exclusively from that dependency; the consumer commit does not write them.

| Check on this consumer source | Outcome |
| --- | --- |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/input-service/Cargo.toml` | Exit 0,21 scenarios: 8 new wait plan cases and 13 existing boot/daemon cases |
| `cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/input-service/Cargo.toml --all-targets -- -D warnings` | Exit 0 |
| Same Clippy for `--release --target x86_64-unknown-none --features native --bins` | Exit 0, both mains actually typechecked |
| `cargo +1.90.0 fmt --manifest-path sdk/examples/input-service/Cargo.toml -- --check` | Exit 0 |
| `python3 -B sdk/examples/input-service/build.py` with its own target directory | Exit 0, both real scalar ELFs plus existing `inspect_user` check |
| Disassembly of those two ELF files | Source 2936/policy 5299 instructions, no FP/SIMD instructions found |

The source ELF has SHA256
`129556644b7edb2862bd02920799d637072da1e7cd110dcecae111c65a25ff6a`;
policy `0caeab494a672cb628da87c134566aa0df6a26ff585df8269778c6ea085d6fdc`.
The builder keeps the existing 64KiB stack; the bounded plan value is 272 bytes
and motivates the local `large_enum_variant` lint exception without an
allocator or general disabling of checks.

Exact command logs, source/ELF hashes and inspection are in the separate
`build/input-idle-wait/logs/{final-checks.json,component.json}`; binaries
exclusively in `target/input-wait-native`, not in Git. Two early
host attempts had an invalid fixture registration order (Engine requires
monotonic PIDs); the fixture was corrected, not the production engine.
Independent consumer review and native wait/GUI execution remain open.

## Targeted correction: error isolation of app replies

Our own re-reading of the 2b286 source found that a lifetime error from the composite
wait would make all policy clients fatal, while the old IPC flush only
retires an actually defective app reply lane. The follow-up source keeps that
isolation with one bounded recheck. Clock, yield and mask errors have
separate error types and cannot use that recovery route.

The retest counts 23 host scenarios: the 13 existing and 10 wait plan cases. The two
new cases test real closed, revoked and stale-handle app reply leaves
and still-working second-app IPC; an unexplained lifetime error, another error type
or broken critical ingress does not become poll success. Host and both native
target Clippy checks pass. New executable hashes follow separately;
the 2b286 hashes above remain exclusively with their original source.

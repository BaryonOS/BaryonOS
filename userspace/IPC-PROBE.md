<a id="eigen-ipc-userfixture-u04"></a>

# Independent IPC user fixture U04

`ipc_probe.rs` is a separate synthetic ring-3 program for M4. It does not replace the M3 probe. The fixture executes actual `int 0x80` syscalls; only the kernel observer determines the KS-09–16 results. This text describes the agreed fixture interface and planned actual cases, not QEMU tests that have already passed.

<a id="bouwen-en-controle"></a>

## Build and checks

The root agent manages the existing build tool:

```powershell
python tools/build_users.py --kind ipc
python -B -X utf8 userspace/tests/test_ipc_probe.py --elf build/users/ipc_probe.elf
```

Input: `userspace/ipc_probe.rs`, `userspace/ipc_linker.ld`. Output: `build/users/ipc_probe.elf`, `build/users/manifest-ipc.json`. Embed variable: `NOS_IPC_ELF`. The direct Rust1.90.0 flags match M3: x86_64-unknown-none, no_std, panic=abort, opt-level=2, code-model=small, relocation-model=static, no-redzone=yes, rust-lld, its own absolute `-T` linker path, --no-pie, --build-id=none, max-page-size=0x1000. No dependencies, allocator, host libc, network, hostcalls or port I/O.

<a id="geheugeninterface"></a>

## Memory interface

| Virtual range | ELF permissions | Contents |
|---|---|---|
| `0x40000000` | RX | `_start` and interpreter |
| `0x40010000–0x40011000` | RW | Control words, rows and results |
| `0x40011000–0x40012000` | RW | `msg`, 4096 bytes |
| `0x40012000–0x40013000` | RW | `out`, 4096 bytes |
| `0x40013000–0x40014000` | R | Zero-filled RO test page, to be mapped NX by the kernel |

All ranges are half-open. The ELF has three PT_LOAD segments: RX, 12288 bytes RW, 4096 bytes R. ELF flags do not prove that paging permissions are actually set. Each process receives its own physical frames; barriers are also in separate private frames.

Control words are little-endian u64 at `0x40010000`:

| Offset | Contents |
|---|---|
| 0 | Mode: 0 worker; 1 copy; 2 wire; 3 snapshot; 4 handles; 5 rights; 6 revoke; 7 quotas; 8 lifecycle |
| 8 | Private counter; incremented at each syscall and explicit crash operation |
| 16 | Private pattern; the initial copy remains in r13 and is checked during execution |
| 24 | Target/scratch, defined by the manifest |
| 32 | Role: A=0, B=1, C=2 |
| 40 | Number of fully completed rows |
| 48 | Status: 0 active, 1 rows completed, `0x100+rowindex` fixture error |
| 56 | Main: own endpoint root, rights 31 |
| 64 | Aux: initial local capability to the next role, rights 29 (no RECV) |
| 72 | Foreign: numeric peer handle; grants no access to the peer table |
| 80 | Last RAX/diagnostic |
| 88 | Row count, at most 32 |
| 96 | Own barrier word; the kernel observer modifies it only after observing actual events |
| 104 | Reserved, not used by the interpreter |
| 112 | Attempt counter, at most 32768 |

The interpreter first executes `abi_info(0)` and checks RAX=0/RDX=`1<<32`. After status1, the process continues as a counter/yield worker; other processes can therefore demonstrate actual progress after a peer completes, exits or faults. On a fixture error, the interpreter retains the diagnostic/status and performs a normal `exit(0xe4)`. No user code writes kernel markers or determines the kernel test result.

<a id="rows-en-werkelijke-resultaten"></a>

## Rows and actual results

32 rows start at `0x40010100`; each row contains ten u64 values, exactly 80 bytes:

```text
op @0
a0,a1,a2,a3,a4,a5 @8,16,24,32,40,48
expected_RAX @56
expected_RDX @64
flags @72
```

Expected negative error codes are u64 two's-complement values. Result slot i is at `0x40010c00 + 16*i`: actual RAX at +0 and actual RDX at +8. User code fills slots from the actual syscall registers or an ordinary `load64` from its own received data. The kernel does not fabricate syscall results.

| Flags | Meaning |
|---|---|
| Bits0–5 | The corresponding argument is a result index; load the actual RDX from that slot |
| Bit6 | Do not compare the dynamic RDX with a fixed value; retain it and check RAX |
| Bit7 | `expected_RDX` is itself a result index |
| Bit8 | Only on EAGAIN: perform an actual yield and execute the same row again |
| Bits32–47 | Repetitions of the same row, 0 means 1; at most 1024 |
| Other | Invalid; accepted mask `0x0000ffff000001ff` |

Each repetition resolves the arguments again and overwrites only its own result slot with the latest actual outcome. A reference to its own result slot can continue a derive chain after an ordinary user write has initialized that slot with an actual previously received/derived handle. Row progress increases only after all repetitions succeed. EAGAIN and barrier retries count toward the bounded attempt counter.

Example of a follow-up action using an actual handle: `ipc_recv(main,out,320)` → `load64(out+32)` with flag bit6 → write that loaded RDX via bit1 to `msg+32` → `ipc_send(aux,msg,320)`. Also close derived handles via their actual RDX result slot, rather than assumed slot/generation numbers.

<a id="interpreteroperaties"></a>

## Interpreter operations

`op < 0x100` is the syscall number directly. Arguments go to RDI/RSI/RDX/R10/R8/R9; RAX/RDX are then recorded. This includes ordinary syscalls 0/1/2, cap_close10h/cap_derive11h/cap_revoke12h, ipc_send20h/ipc_recv21h and actual unknown numbers for ENOSYS tests.

| Fixture operation | Arguments and effect |
|---|---|
| `0x100` WRITE64 | a0=address, a1=value; ordinary private 8-byte store |
| `0x101` FILL | a0=address, a1=byte0..255, a2=length0..4096 |
| `0x102` ASSERT64 | a0=address, a1=expected value; ordinary private read/comparison |
| `0x103` LOAD64 | a0=address; result RDX from own memory, RAX=0 |
| `0x104` WAIT64 | a0=own address, a1=expected value; otherwise actual yield and retry |
| `0x105` UD2 | Explicit receiver fault for the lifecycle test; no normal return |

Writes/FILL may touch only result slots `0x40010c00–0x40010e00` or RW buffers `0x40011000–0x40013000`. They cannot change rows, status, barrier or mode. Read operations are limited to `0x40010000–0x40014000`. All local additions check carry; an incorrect fixture script produces a fixture error. An arbitrary syscall pointer remains an integer so that the kernel must check its actual range/permissions. Misalignment is permitted for these byte/x86 memory operations.

## Message320

The OS wire contract remains unchanged: sender_pid u64@0 (send0), tag u64@8, len u32@16, cap_count u32@20, reserved u64@24 (0), two `{handle:u64,rights:u64}` pairs@32/48, data[256]@64. Total: 320 bytes. A WRITE64 at +16 therefore combines `len | (cap_count<<32)`. Unused descriptors/payload bytes remain zero. The RO page initially contains a valid empty canonical message; reception there must fail because of write permissions.

Confirmed codes: EMSGSIZE(-11) for size≠320/len257/capcount3; EINVAL(-1) for sender/reserved/unused descriptor/tail or unknown rightsbit32; EACCESS(-5) for noSEND/noDERIVE/rights escalation/transfer without TRANSFER/transferRECV. Revoked(-10) takes precedence over Closed(-9). Header and capability validation before the user buffer is part of the kernel observer test.

<a id="afgesproken-case-map-voor-kernelmanifest-en-observer"></a>

## Agreed case map for the kernel manifest and observer

| Mode / KS | Actual operations | To be proved by the kernel |
|---|---|---|
| 1 / KS09 | Send/recv with null, supervisor, noncanonical, overflow, stack/heap end crossings; out→RO-cross at `0x40012f80`; misaligned valid buffers | EFAULT or earlier EMSGSIZE; full output/queue/captable identical before/after on failure; normal delivery after valid recv |
| 2 / KS10 | Sizes0/319/321/MAX; len257/capcount3; nonzero sender/reserved/unusedcap/tail; buffers+1 | Exact error order/codes, no mutation; valid misalignment remains permitted |
| 3 / KS11 | Send known bytes; hook changes original len/payload only after snapshot; recv; unknown thread/shared/unmap numbers | Only the original snapshot is used; missing APIs return ENOSYS; no claim about actual SMP races |
| 4 / KS12 | Numeric peer handle, slot0/invalid gen; derive→close→stale; deliberate local collision | Lookup only in own table; EBADHANDLE when invalid; collision grants at most existing own authority |
| 5 / KS13 | Reduced/zero rights derive; extra32/escalation/noDERIVE/noSEND; transfer without 8/with RECV | EINVAL/EACCESS according to the contract, no new node/slot/queue on error |
| 6 / KS14 | A maincap13→B via aux; queued revoke before Brecv; separate delivered A→B→C using actual received handles; A revoke | Queued message fully cancelled; delivered B/C handles EREVOKED; earlier payload remains readable |
| 7 / KS15 | Eight depthderives plus a ninth; close actual outputs; 253 derives with init2slots; final close/rederive; 16 sends/17th send/drain/send | Depth/capslots/queue quotas and capacity recovery through actual calls; no partial transactions |
| 8 / KS16 | Three rounds: ownerclose; senderexit after queued delegation; receiverUD2 after queued delegation | Last RECV closes the endpoint; issuer-exit revoke/cancel; refs/resources are cleaned up, other processes continue running |

For a receiver-exit lifecycle case, A.aux can return EREVOKED if that bootstrap node was issued by B; this takes precedence over ECLOSED. A normal owner-close without exit gives peers ECLOSED. The kernel manifest and Engine metadata determine the specific expected case; the user fixture does not change this precedence.

The kernel observer checks actual syscall numbers, arguments and results, all relevant output bytes, FIFO, captables/nodes and resources. Status1 is only user feedback. The global 1024-node limit and generation exhaustion are additionally tested in the host engine model; without an additional executed QEMU case, they do not count as evidence of actual enforcement in the guest.

<a id="uitgevoerd-en-herstel"></a>

## Execution status and recovery

The root build and separate host artifact tests are recorded in `verification/U04-HOST-2026-10-27.md` and `verification/ipc-host-evidence.json`. Opcode and ELF checks do not prove an executed syscall. Actual EXCHANGE/case boots and M2/M3 regressions are run by the root/kernel agent and recorded separately.

No M3 probe, M3 test, root tool or kernel file was changed as part of U04. Recovery: revert only these separate IPC fixture files and the optional NOS_IPC_ELF build connection managed by the root agent; the M3 fixture remains independently usable.

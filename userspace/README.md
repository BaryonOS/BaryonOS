# Synthetic own user probe for M3

This small static ELF64 probe is the source for the real ring-3 trials KS-04–08. The host build and artifact tests below prove the file interface; only the kernel/QEMU harness can demonstrate isolation, preemption and fault handling.

No dependencies, host libc, allocator, TLS, dynamic linker or port I/O. The probe uses exclusively `abi_info` (0), `yield` (1) and `exit` (2). It cannot write a serial success marker. `probe.rs` deliberately contains separate forbidden instructions for the negative tests; these enable no compiler target features.

## Build interface

The main agent owns `tools/build_users.py` and builds with the existing local Rust 1.90.0 environment:

```powershell
python tools/build_users.py
python -B -X utf8 userspace/tests/test_probe.py --elf build/users/probe.elf
```

The first command calls rustc directly, so that `.cargo/config.toml` does not accidentally apply the kernel linker script to the user probe. Exact flags, alongside absolute source/output paths:

```text
rustc +1.90.0 --edition=2021 --crate-type=bin --crate-name=os_user_probe
--target=x86_64-unknown-none -C panic=abort -C opt-level=2
-C code-model=small -C relocation-model=static -C no-redzone=yes
-C linker=rust-lld -C link-arg=-T<absolute-pad>/userspace/linker.ld
-C link-arg=--no-pie -C link-arg=--build-id=none
-C link-arg=-zmax-page-size=0x1000 --emit=link
-o <absolute-pad>/build/users/probe.elf <absolute-pad>/userspace/probe.rs
```

The kernel receives this artifact via `NOS_USER_ELF`; root owns the embed/build order. No root Cargo build or change to root config is needed from this directory.

## Fixed interface with the kernel

ELF: `ET_EXEC`, AMD64 little-endian, entry/text at `0x40000000` with RO/X; data at `0x40010000` with RW/NX. The linker script bounds the text to before data and keeps the data area exactly 32 bytes. Kernel mapping grants the actual page rights; ELF flags on their own do not.

Four little-endian `u64` fields, filled in by the kernel before process start:

| Offset | Field | Use |
|---|---|---|
| +0 | mode | Selects one of the branches below |
| +8 | counter | Private progress; incremented before every selected action |
| +16 | pattern | Private pattern; probe keeps the initial value in r13 and compares it every loop |
| +24 | target | Only for modes 3–5; test harness chooses the protected target address |

Every process instance must get its own physical data/text frames. The same virtual address or the same ELF file does not mean shared frames. The kernel reads, for measurement purposes, only validated own process frames; there is no shared user counter.

| Mode | Action after counter++ |
|---|---|
| 0 | Unbounded busy worker without yield |
| 1 | Yield and count again |
| 2 | Exit with code 0 once counter ≥ 8 |
| 3 | Read `u64` at target |
| 4 | Write `u64=1` at target; harness chooses B's supervisor identity address |
| 5 | Write `u64=1` at target; harness chooses kernel text |
| 6 | Jump to stack base `0x7ffef000` |
| 7 | Jump to heap base `0x60000000` |
| 8 | Write to lower stack guard `0x7ffee000` |
| 9 | Write to upper stack guard `0x7ffff000` |
| 10 | `cli` |
| 11 | `fninit` |
| 12 | `pxor xmm0,xmm0` |
| 13 | `vxorps ymm0,ymm0,ymm0` |
| 14 | `ud2` |
| 15 | Immediate exit with code 7 after the first counter increment; no eight-iteration threshold |
| 16 | ABI info with zero arguments via `int 0x80` after ESP/RSP has been zeroed; no normal return |

Before the first loop every instance performs three real syscall checks:

1. ABI info: RAX=0 and RDX=`1<<32`. Directly before `int 0x80`, `std` sets the direction flag (DF). After return the probe reads RFLAGS via `pushfq` and checks that DF is zero again; its own error path always executes `cld`.
2. Unknown number `0xffff`: RAX=-2 (`ENOSYS`), RDX=0.
3. ABI info with one deliberately invalid unused argument RSI=1: RAX=-1 (`EINVAL`), RDX=0.

Every prelude check compares RBX/RBP/R14/R15/RCX/R11 with sentinels, R12 with the fixed data area, R13 with the private pattern, RSP with the agreed initial stack position and all non-result arguments with their expected value. Outside the deliberate RSI=1 case, unused arguments are zero. These are not host calls; the kernel/QEMU trial must actually execute these instructions. There is no TF probe because a debug exception can occur before a reliable check; TF sanitising remains to be tested separately.

Error codes via ordinary `exit`: `0xe0` ABI deviation, `0xe1` changed pattern, `0xe2` unknown mode, `0xe3` forbidden action unexpectedly continued. A returning exit executes `ud2`. None of these exits counts as a successful expected user fault.

The separate kernel test `test-m3-b-exit` uses mode 15 for B and mode 0 for A. Desired evidence: B ends with code 7 and counter 1, the kernel cleans up B in a controlled way and A makes progress afterwards. Exit code 7 is ordinary process metadata here and no reason for a kernel panic. The syscall implementation processes the exit; the test observer separately decides whether the expected state has been reached.

The kernel test `test-m3-invalid-rsp` uses mode 16 for the test process and mode 0 for the other worker. Counter is first incremented once. Then the probe clears all syscall arguments and RAX for ABI info, zeroes ESP/RSP and executes `int 0x80`. This must reach the software validation of the saved return context: only the process is terminated, the other worker makes progress. The `ud2` after `int 0x80` is an unreachable error trap; executing it is not a successful trial. This is not a hardware fault claim.

Modes 6/7 require, in the real harness, a page fault at exactly the intended target with the instruction-fetch and present bits. Without NX, executing zero bytes can still fault later at a different address; merely observing that A faults somewhere does not prove NX. For modes 3–5, CR2 and the user/read/write fault bits are likewise part of the evidence. In all fault modes B must continue.

## Evidence and limitations

See `verification/R03-HOST-2026-10-27.md` and `verification/host-evidence.json` for the current executed build and tests. The artifact tests cover real ELF segments, hashes, relocation/TLS absence, syscall argument/DF encoding and negative mutated ELF files. Opcode presence does not prove an executed instruction. The tests use the build inspector owned by root; they replace the kernel parser and are not an independent implementation of it.

All temporary mutation files reside exclusively under `userspace/tests/` and are removed after the tests. Artifacts stay under `build/` outside Git. This directory contains no claims that KS-04–08 have already passed.

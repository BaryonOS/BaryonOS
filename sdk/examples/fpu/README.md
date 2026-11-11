# Native x87/SSE context test

This opt-in profile validates one kernel mechanism, not a complete test release.
`fpu-context` provides eager FXSAVE64/FXRSTOR64 for every process slot;
`native-fpu` adds two separate ring-3 ELFs and a bounded observer.
The 22-word interrupt frame, process address spaces, W^X and capability/IPC contracts
are preserved. This profile has no network, personal data or writable
external disk.

The kernel checks CPUID.FPU/FXSR/SSE/SSE2, sets CR0.MP/NE, clears EM/TS,
sets CR4.OSFXSR/OSXMMEXCPT and clears OSXSAVE. AMD EFER.FFXSR is cleared and
read back so that CPL0 FXSAVE may not skip the XMM registers. Every entry
from ring 3 saves hardware state before Rust. The final return assembly
restores the selected state after all Rust, directly before the GPR pops and iret.
CPL0 idle timer returns do not use this user restore. There is no lazy TS sharing,
AVX/XSAVE, SMP or floating-point numbers in kernel Rust.

Each of the sixteen process slots owns a permanent, supervisor-only,
16-byte-aligned 512-byte image: 8192 bytes in total. Spawn and reuse clear
the complete image and set only FCW=037f and MXCSR=1f80. FTW, payloads,
FIP/FDP and XMM0–15 start at zero. Only the kernel and real FXSAVE can write these
restore images. Cleanup clears all bytes; no user pointer is held across
a scheduling point. FNINIT alone would retain physical register data.

The apps deliberately use direct, checked int80 syscalls. That is an
architecture test, not a second SDK product implementation. Before any FPU/SIMD
data instruction, every new instance checks the hardware via FXSAVE64:
FCW, FSW, FTW, MXCSR, FIP/FDP, eight 80-bit payloads and all sixteen XMM registers.
After that every register gets a process-specific value and every x87 slot a
different number. B additionally has different virtual x87 instruction and
operand addresses, so that the FIP/FDP check can detect a swapped pointer
state. FCW/MXCSR have different rounding modes. The apps
compare this real register state repeatedly after syscalls and busy loops with
PIT preemption. Reserved/padding bytes are not tested as architectural data.

The separate profile `test-fpu-wait` combines the same test with
`ipc-wait`/ABI1.10. Only there do the apps replace every yield with syscall0x22
with zero conditions and an absolute deadline two ticks after the previously measured
clock. The whole register bank is also compared after returning from that wait.
The kernel requires at least three real blockings, matching
deadline wakes, actually counted idle IRQs and no ready/error wakes.
The number of wait calls must exactly equal the number of verified
loops; any deadlines already expired are counted separately.
Timer preemption per app, the AVX fault, survivor, fresh PID and complete cleanup
remain mandatory. A survivor that is still waiting is alive, not wrongly
a failure; after its deadline it must demonstrably continue working before reuse.
The original ABI1.6/yield profile continues to exist separately.
All IRQ0s count towards the monotonic clock; only CPL3 entries save/restore
a user FPU bank. Therefore the total IRQ count is exactly user-A + user-B +
idle IRQs, while saves must be exactly user-A + user-B + syscalls + the real fault.
CPL0 idle returns deliberately skip the FXSAVE/FXRSTOR route.
Kernel and host both verify these relations; idle may not represent user preemption
or an extra FPU save.

After at least twelve PIT ticks, A triggers a real unsupported-AVX #UD.
CPUID.AVX must be absent on this qemu64 profile. B must then demonstrably
continue working. Only then is slot0 rebuilt with fresh PID3;
that instance first checks clean hardware state and again verifies
at least twelve ticks. After the normal exits all process frames have been independently
counted back and all 8192 context bytes are zero. The observer supplies no app results.

The apps also test syscall3: ABI1.6, six separately refused nonzero args,
non-decreasing PIT ticks and real progress across yield/timer switches. The public
SDK provides `native::monotonic_ticks()` and `clock::MONOTONIC_HZ=100`. It is a
boot-monotonic IRQ counter that saturates at u64::MAX; not wall time, a certificate clock,
a real-time guarantee or accounting across suspend. Old profiles without
`monotonic-clock` keep their ABI and ENOSYS for syscall3.

Build via the shared `tools/build.py --kernel-feature native-fpu` once that
profile has been integrated; the helper `tools/run_fpu.py::build_users()` builds the two
standalone workspace bins and returns `NOS_FPU_A_ELF`/`NOS_FPU_B_ELF` plus
the inspection report. `python3 tools/run_fpu.py` validates the boot sequence,
process results, independent counter relations, exact ELFs in the kernel,
source fingerprint, loader/kernel/firmware hashes, actual QEMU command line
and raw serial output. Host tests: `python3 -m unittest discover -s tools -p test_fpu.py`.
For the combination use `--kernel-feature test-fpu-wait` and the same
runner with a separate `NOS_BUILD_DIR`. The manifest choice determines which
strict oracle applies: normal evidence cannot approve the wait variant.

An earlier test with `LDMXCSR(80000000)` reached the intentional failure105:
this QEMU/TCG profile did not isolate a #GP in that case and is not evidence for it.
The emitted instruction and operand were checked with disassembly/rodata bytes.
The raw failed test is kept in `verification/mxcsr-tcg-gap/`. The chosen
negative acceptance is therefore a real unsupported-AVX #UD. There is no #GP claim.

Normative design references: Intel SDM vol.1 “Managing State Using the FXSAVE
and FXRSTOR Instructions”, vol.2 CPUID/FXSAVE/FXRSTOR/LDMXCSR and vol.3 CR0/CR4;
AMD APM vol.2 EFER.FFXSR and x87 pointer restoration conditions. The official
Intel PDF was refused by the network proxy with HTTP403 during this work; no
newly fetched primary specification check is claimed. The local
secondary source has been read: OSDev: FPU, W002427, revision29122; the independent
review also checked OSDev: FXSAVE, W001452, revision30264. Physical AMD
processors in particular require a separate CPU/errata policy for conditional
FIP/FDP/FOP restoration. QEMU evidence does not apply to physical hardware.

Open: product profile integration, engines, broader CPU/FPU exception cases,
missing-FXSR boot test and physical hardware validation. The test release and the
complete browser/media/AI acceptance remain open.

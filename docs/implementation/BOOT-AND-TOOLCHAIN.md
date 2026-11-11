# M1: decided boot contract and build profile

Status: implementation specification, 26 October 2026. This document closes the boot choices for M1; the next development chat implements them. It contains no compiled loader, kernel or successful VM test. The current research environment has no `rustc`, `cargo`, `rustup`, QEMU, Clang or `ld.lld` in PATH. There is therefore no boot evidence. The source check below was, however, carried out. For M2–M4 also read [KERNEL-AND-SECURITY-CONTRACT.md](KERNEL-AND-SECURITY-CONTRACT.md).

## Execution corrections — 27 October 2026

The actual Windows run uses QEMU 11.1.0 from the recorded development package with matching firmware. The original 2–18MiB reservation overlaps ACPI NVS and BootServicesData there; the loader rightly rejected it. The fixed profile was therefore changed to 32–48MiB and BootInfo ABI 1.1. The header layout is unchanged; ABI 1.0 is rejected. The read-only ESP is attached via virtio, because IDE refuses the read-only block node. A compiler-generated static GOT is explicitly included in .data; runtime relocations remain forbidden. See ADR-019 to ADR-021 and PROJECT-STATE.md for real test evidence. Original research claims below are historical context, not a new test result.

## 1. Fixed profile

| Component | Decision for M1 | Evidence status |
|---|---|---|
| Architecture | x86_64, little endian, 4 KiB pages, one CPU | Design choice |
| VM | `pc-q35-10.0`, `qemu64`, TCG, 256 MiB RAM | Design choice; no boot yet |
| Rust | exactly **1.90.0**, edition 2021, `panic=abort` | Official release of 18 September 2025 confirmed [B1]; deliberately a fixed known baseline, not a claim that this is the newest compiler |
| Loader target | `x86_64-unknown-uefi`; own PE32+ UEFI app | Target and calling convention confirmed [B2] |
| Kernel target | `x86_64-unknown-none`; own static ELF64 `ET_EXEC` | Target confirmed [B3] |
| Only external loader crate | `r-efi = { version = "=7.1.0", default-features = false }` | Published version 10 August 2026 and MSRV 1.68 confirmed [B4–B5]; combination with the project code not yet compiled |
| Kernel dependencies | only own `boot-protocol`; `core` of the pinned target | No external kernel implementation |
| QEMU reference | **10.0.14** | Confirmed on the official download page [B6] |
| Firmware reference | EDK II **edk2-stable202508**, `OvmfPkg/OvmfPkgX64.dsc`, X64/RELEASE/GCC5 | Release and official build route confirmed [B7–B8]; own binaries and hashes not yet produced |
| Storage | generated ESP directory via QEMU's read-only FAT disk | Only synthetic project files |
| Network | `-nic none`; no network device, host share, agent, clipboard or guest network stack | Test profile; no evidence of later app isolation |

**Build the own code straight away.** An existing or distribution QEMU may be used for development if `pc-q35-10.0` is available. In that case use OVMF_CODE and OVMF_VARS from exactly the same package/build; record version, provenance and SHA-256 of both. This variant must pass the same tests, but is not byte-identical to the reference. Do not first build QEMU and EDK II from source if this development variant is available. A separate throwaway copy of the VARS template is made for every test; CODE stays read-only. Secure Boot is off in M1; M1 proves no authenticated boot chain.

`r-efi` provides only specification types and constants, no bootloader, allocator, driver or OS kernel. The project writes the loading logic itself. The consulted Cargo manifest source shows no dependency enabled by default; `rustc-std-workspace-core` is optional. Licence choice for this dependency: MIT (upstream offers MIT OR Apache-2.0 OR LGPL-2.1-or-later). Keep the notice and registry checksum in the dependency administration. Do not add `uefi-services`, the `bootloader` crate or an adopted kernel. The high-level `uefi` crate is deliberately not selected here: a small local wrapper around raw bindings makes the final allocations and the handoff explicitly checkable.

## 2. Files to create and fixed commands

| Path | Responsibility |
|---|---|
| `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/config.toml` | Integration by the main agent; the workspace contains exclusively the Rust packages named below |
| `crates/boot-protocol/src/lib.rs` | `#![no_std]`; BootInfo, offsets, safe header/map validation |
| `boot/loader/Cargo.toml`, `boot/loader/src/main.rs` | Package `os-loader`; UEFI entry, limited firmware wrapper, file reading, handoff |
| `boot/loader/src/handoff.S` | EFIAPI to SysV, own stack, interrupts off, empty IDT |
| `crates/elf-loader-core/src/lib.rs` | Own `no_std` byte parser; host-testable ELF validation and load plan without physical writes |
| `kernel/Cargo.toml`, `kernel/src/main.rs` | Package `os-kernel`; entry, own COM1 output, BootInfo check and test exit |
| `kernel/linker.ld` | Explicit ELF segments and fixed addresses |
| `tools/build.py` | Build both targets, check the ELF and create the ESP directory |
| `tools/run.py` | QEMU arguments, private VARS copy, logs, timeout and smoke test |
| `tools/test.py` | Host tests, corrupt ELF images, EBS retry trial and VM smoke test |
| `build/esp/EFI/BOOT/BOOTX64.EFI`, `build/esp/kernel.elf` | Generated throwaway artifacts, do not edit by hand |
| `build/logs/`, `build/manifest.json` | Exact commands, hashes, versions, stdout/stderr and raw exit codes |

Do not set a general target for the whole workspace: host tests must run on the host target. Product and quantum prototypes are not root workspace members. Explicitly add any nested Rust workspaces to `exclude`.

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.90.0"
profile = "minimal"
components = ["rustfmt", "clippy"]
targets = ["x86_64-unknown-uefi", "x86_64-unknown-none"]
```

```toml
# .cargo/config.toml — only the kernel target gets ELF link options.
[target.x86_64-unknown-none]
linker = "rust-lld"
rustflags = [
  "-C", "code-model=small",
  "-C", "relocation-model=static",
  "-C", "no-redzone=yes",
  "-C", "link-arg=-Tkernel/linker.ld",
  "-C", "link-arg=--no-pie",
  "-C", "link-arg=--build-id=none",
  "-C", "link-arg=-zmax-page-size=0x1000",
]
```

The linker is part of the Rust toolchain. `tools/build.py` finds it via `rustc +1.90.0 --print sysroot` plus the host triple from `rustc +1.90.0 -vV`, and adds `<sysroot>/lib/rustlib/<host>/bin` to the subprocess PATH. That way the name `rust-lld` does not require a separately installed host LLD. The built-in target selects the ELF linker flavour. For UEFI the built-in PE/COFF linker settings are kept; never send the ELF script options to the loader. The kernel target has code model `kernel` by default; the chosen low placement requires the deliberate override to `small` above [B3]. Both crates use `#![no_std]`, `#![no_main]`; no nightly, custom-target JSON or `-Zbuild-std` is needed for M1.

Root profiles: `panic = "abort"` in dev/release; release `codegen-units = 1`, `lto = false`. Use no unwind path, global allocator, TLS, floats/SIMD, C runtime or host syscalls. If the linker turns out to need memory intrinsics, provide minimal own `memcpy`/`memmove`/`memset`/`memcmp` in a separately reviewed `unsafe` module; prevent optimisation into recursive calls and check the result. Do not silently add host libc.

After installing rustup via the official procedure, these are the operational entry points:

```sh
rustup toolchain install 1.90.0 --profile minimal --component rustfmt --component clippy
rustup target add --toolchain 1.90.0 x86_64-unknown-uefi x86_64-unknown-none
python3 tools/preflight.py
python3 tools/build.py
python3 tools/run.py --smoke
python3 tools/test.py
```

These last three tools are created by the implementation task; this document does not claim that they already exist or have been run. Once, if the lockfile is missing, `build.py` creates the `Cargo.lock` with explicitly pinned dependencies; after that all builds use `--locked`. Downloading public development dependencies belongs to the development environment, not to the installed OS. After that a populated/vendored dependency cache can be used offline.

Underneath, the tool builds from the repository root:

```sh
cargo +1.90.0 build --locked --release -p os-loader --target x86_64-unknown-uefi
cargo +1.90.0 build --locked --release -p os-kernel --target x86_64-unknown-none
```

## 3. ELF and linker layout

The kernel uses physical addresses equal to virtual addresses. Reserve **`[0x02000000, 0x03000000)`**, exactly 16 MiB, via UEFI `AllocatePages(AllocateAddress, EfiLoaderCode, 4096, &base)` before other large buffers are allocated. M1 is specific to the recorded VM profile. If this reservation fails: explicit error; no arbitrary other load address. The linker bounds the used kernel to this interval.

Starting point for `kernel/linker.ld`:

```ld
OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)
PHDRS {
  text   PT_LOAD FLAGS(5);
  rodata PT_LOAD FLAGS(4);
  data   PT_LOAD FLAGS(6);
}
SECTIONS {
  . = 0x02000000;
  __kernel_start = .;
  .text : ALIGN(4096) { KEEP(*(.text.entry)) *(.text .text.*) } :text
  . = ALIGN(4096);
  .rodata : { *(.rodata .rodata.*) } :rodata
  . = ALIGN(4096);
  .data : { *(.data .data.*) *(.got .got.*) } :data
  .bss (NOLOAD) : ALIGN(16) { *(.bss .bss.*) *(COMMON) } :data
  __kernel_end = ALIGN(4096);
  /DISCARD/ : { *(.eh_frame*) *(.comment) *(.note*) }
  ASSERT(__kernel_end <= 0x03000000, "kernel exceeds reserved span")
}
```

`build.py` inspects the real ELF output; the script alone is not evidence for the segments. Reject unexpected allocatable orphan sections, dynamic metadata or relocations that require runtime processing. Keep a machine-readable segment overview. `p_paddr == p_vaddr` is mandatory. RX, R and RW get separate page intervals aligned to 4 KiB; do not allow page overlap through rounding.

The loader treats the entire ELF file as untrusted bytes. A pure parser first produces a bounded load plan; physical writes only follow after complete validation. Accept exclusively ELF64, little-endian, current ELF version, `EM_X86_64=62`, `ET_EXEC=2`, header size 64 and program header size 56. File at most 32 MiB, `1 <= e_phnum <= 64`. All additions/multiplications and conversions to `usize` checked. Complete program header table within the file; no extended program header numbering.

Per `PT_LOAD`: `filesz <= memsz`; `offset + filesz` within the file; non-empty memory range fully within the reserved 16 MiB; `p_align` a power of two and at least 4096 for this profile; `offset % p_align == vaddr % p_align`; `paddr == vaddr`; only R/RX/RW flag combinations, never W+X. Non-empty load ranges do not overlap, not even after rounding to pages. Entry lies within the bytes actually loaded from the file of exactly one RX segment. Zero-length segments do not count as entry or valid load content.

For this small profile: reject `PT_INTERP`, `PT_DYNAMIC`, `PT_TLS`, and unknown program header types. `PT_NULL`, `PT_NOTE`, `PT_PHDR` and a non-executable `PT_GNU_STACK` may be accepted as metadata, but are not executed. At least one RX segment and one non-empty `PT_LOAD` required. Section headers are not a load source; optional section information is only a build-time check. First zero the reserved region, then copy exclusively approved `filesz`; that makes `.bss` zero. These project restrictions are deliberately stricter than general ELF capabilities [B11].

**W^X status:** in M1 ELF flags are a checked intention, not yet enforced memory protection. M1 temporarily inherits UEFI page tables. M2 creates its own tables, supervisor-only mappings, read-only code and NX data and tests the real faults. This M1 profile does not yet have userspace, ASLR, guard pages, DMA isolation or Secure Boot. Do not claim security for those on the basis of a successful boot message.

## 4. BootInfo v1.1: exact handoff format

All multi-byte fields are little-endian. Physical addresses are stored as `u64`; no Rust references, `usize`, `bool`, enums, packed structs or slice fat pointers in the ABI. `#[repr(C)]`, alignment 8, total size **128 bytes**. The structure is at the start of its own zero-filled 4096-byte `EfiLoaderData` allocation. All pointers are only used after bounds/overlap checks.

```rust
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BootInfo {
    pub magic: [u8; 8],
    pub abi_major: u16,
    pub abi_minor: u16,
    pub header_size: u32,
    pub flags: u64,
    pub memory_map_addr: u64,
    pub memory_map_len: u64,
    pub descriptor_size: u32,
    pub descriptor_version: u32,
    pub kernel_reserved_start: u64,
    pub kernel_reserved_len: u64,
    pub stack_start: u64,
    pub stack_len: u64,
    pub bootinfo_start: u64,
    pub bootinfo_len: u64,
    pub loader_start: u64,
    pub loader_len: u64,
    pub memory_map_capacity: u64,
    pub reserved0: u64,
}
```

| Byte offset | Field | Value/semantics |
|---:|---|---|
| 0 | `magic`, 8 bytes | `b"NOSBOOT\0"` |
| 8 | `abi_major`, u16 | 1 |
| 10 | `abi_minor`, u16 | 1 |
| 12 | `header_size`, u32 | 128 |
| 16 | `flags`, u64 | bit0: EBS succeeded; bit1: firmware paging still active; bit2: M1 QEMU test profile; other bits 0 |
| 24 | `memory_map_addr`, u64 | Physical start of a separate, preserved page allocation |
| 32 | `memory_map_len`, u64 | Actual bytes of the last successful `GetMemoryMap` |
| 40 | `descriptor_size`, u32 | Firmware stride, at least 40, multiple of 8, at most 4096 in this profile |
| 44 | `descriptor_version`, u32 | 1; an unknown version is rejected for M1 |
| 48 | `kernel_reserved_start`, u64 | 0x02000000 |
| 56 | `kernel_reserved_len`, u64 | 0x01000000 |
| 64 | `stack_start`, u64 | Own page-aligned stack base, not a firmware stack |
| 72 | `stack_len`, u64 | 65536 bytes |
| 80 | `bootinfo_start`, u64 | Physical address of this structure and page allocation |
| 88 | `bootinfo_len`, u64 | 4096 bytes |
| 96 | `loader_start`, u64 | `LoadedImage.ImageBase`, rounded down to 4 KiB |
| 104 | `loader_len`, u64 | Whole loaded PE image up to rounded-up `ImageBase + ImageSize` |
| 112 | `memory_map_capacity`, u64 | Full reserved map buffer in bytes; may be larger than len |
| 120 | `reserved0`, u64 | 0 |

M1 accepts exactly major 1/minor 1/header 128 and flags 7. Change semantics or required fields only with an agreed ABI version; let loader and kernel use the same crate. Compile-time assert for `size_of`, `align_of` and **every** `offset_of!`; a host test checks an explicit byte fixture. This way the implementation cannot introduce a hidden padding shift.

Limited check performed on 26 October 2026: a Python `ctypes.Structure` with exactly the fixed-width fields above gives, on the research host, size 128, alignment 8 and offsets `[0,8,10,12,16,24,32,40,44,48,56,64,72,80,88,96,104,112,120]`; asserts passed, process exit 0. This checks the prescribed C layout, not the not-yet-built Rust binaries or the boot.

Kernel reservation, stack, BootInfo, loaded loader and map buffer are separate non-overlapping preserved intervals. All starts are non-zero and page-aligned. All ends with checked arithmetic, below 1 GiB in this profile; stack/map/BootInfo are allocated with `AllocateMaxAddress` below that limit. The 256 MiB guest effectively sets a lower available RAM limit. Confirm that ranges in the firmware map are covered by LoaderCode/LoaderData; take multiple adjacent descriptors into account. BootInfo does not point to data on an abandoned stack. `bootinfo_start` must equal the received pointer address.

### Memory map contract

The format is the raw, preserved UEFI map from the successful EBS transition. Iterate with `descriptor_size`, **never** with `size_of::<MemoryDescriptor>()`. Require `0 < len <= capacity <= 1 MiB`, `len % stride == 0`. The used length has no padding at the end; bytes beyond it are not descriptors. Each record has the UEFI v1 prefix: `type:u32@0`, reserved padding@4, `physical_start:u64@8`, `virtual_start:u64@16`, `number_of_pages:u64@24`, `attribute:u64@32`. Read prefix fields as bytes/unaligned integer values; do not form a typed slice with the wrong stride.

`physical_start` is page-aligned; length is `number_of_pages * 4096` with overflow checking. Zero-page records are rejected. Descriptor order is not trusted; detect overlap without using a firmware allocator. Unknown types/attributes are treated as reserved. `virtual_start` is no licence to follow that pointer. MMIO ranges may lie outside the 256 MiB of RAM; do not map or use them automatically.

**M1 reuses no firmware memory whatsoever.** The loader image, stack, map, BootInfo, runtime/ACPI reservations and existing page tables therefore remain. M2 starts only with suitable `EfiConventionalMemory` (type 7), with explicit subtraction of preserved ranges and exclusion of runtime/nonvolatile/special attributes. EfiBootServicesCode/Data are not released automatically while the firmware CR3 is still active. First own tables/GDT/IDT/stack and a checked reclamation decision. There are no UEFI service pointers in BootInfo. The raw map follows the UEFI definition; the chosen allocation limits and reclamation rules are project policy [B9].

## 5. UEFI loader and ExitBootServices

Use `use r_efi::efi;` and the upstream protocol types; do not define your own partial `BootServices` struct with possibly wrong offsets. Entry has `extern "efiapi"`, `efi::Handle` and `*mut efi::SystemTable`, and returns `efi::Status` before the handoff. Every firmware call sits behind one small wrapper with a validity/lifetime contract. `extern "C"` is not used as a universal UEFI ABI [B2].

The order is fixed:

1. Check image/system/boot-services pointers and table revision; stay at `TPL_APPLICATION`. Disable the watchdog with `SetWatchdogTimer(0, 0, 0, null)`. Create no own UEFI events. Keep the image handle; `ExitBootServices` must get that original handle.
2. Read the image base/size and the boot volume via the Loaded Image Protocol. Open the Simple File System on **that same** device and read `\kernel.elf` with the File Protocol. No network boot or searching all volumes.
3. Reserve the fixed kernel span before large file buffers. Read the file into its own bounded loader data buffer of at most 32 MiB; accept short reads, EOF and explicit error status, never an unchecked file size. Validate fully and load according to §3. Close file/root handles and any opened protocol relationships. Return temporary file/info buffers while boot services are still fully available.
4. Allocate own stack (16 pages) and BootInfo (1 page) as LoaderData. The final map buffer follows in step 5. Use no `Vec`, logger with allocations or implicit `Drop` that may later call firmware. Keep loaded-image data as plain integers; no dependent protocol objects after this phase.
5. Query the required map size with `GetMemoryMap` and zero capacity. Allocate `round_up(max(256 KiB, needed + 8 * descriptor_size), 4096)`, with checked arithmetic and a hard maximum of 1 MiB. This is capacity, not the final map. If the first real map does not yet fit, it may be enlarged in a controlled way **before the very first EBS attempt**. At most three preparation attempts; otherwise error.
6. Initialise BootInfo with all reservations, bit0 still zero. Finish all firmware diagnostics and resource drops. From now on no allocation/free, protocol call, console call, callback registration or firmware log is allowed. The last memory map buffer allocation itself must therefore already be in the map queried afterwards.
7. On **every** `GetMemoryMap`, reset the in/out `MemoryMapSize` to the buffer capacity; fetch map, stride, version and map key. Check metadata in own code without allocations. Pass **this map key** directly to `ExitBootServices(image_handle, map_key)`. CPU writes to already reserved BootInfo do not change the firmware map.
8. `SUCCESS`: immediately own `cli`/`cld` and empty-IDT bootstrap; mark the boot services wrapper as consumed. No destructor/return path with firmware calls. Keep exactly this last map len/stride/version. Set the success bit, log `BOOT:EBS_OK` with own COM1 and go to the handoff. The code path between success status and shim may also no longer use any firmware interrupt handler.
9. `INVALID_PARAMETER`: the map key is no longer usable. Only repeat `GetMemoryMap` in **the same existing buffer** and then EBS. At most three EBS attempts in total. This design uses no new allocation during retry either. If the buffer is then too small, metadata is wrong or another status follows: final test failure; no return to the firmware menu or reuse of an old key.

After a first failed EBS call, firmware may have been partially torn down. Therefore the retry area is the small loop above; no file I/O or logging via firmware. After success Runtime Services are not used either. Failures are logged via the own fixed QEMU COM1 driver and test exit; this direct preboot I/O is exclusively the terminal error diagnostics in the recorded QEMU profile. General portable UEFI I/O goes via firmware before finalisation. The EBS state machine follows the current-map rule from UEFI §7.4.6; the retry limit and pre-reserved buffer are own policy [B9].

For a reproducible retry test: a **test-only build** fetches a map key before the first EBS, then allocates one extra test page and after that freezes all allocations. The first attempt with the old key must get `INVALID_PARAMETER`; the retry fetches the fresh map. That test page stays reserved in the last map. The production code gets no fault-injection flag or stale-key path.

## 6. CPU handoff: two different calling conventions

The loader is a UEFI app with the x64 EFIAPI convention. The ELF kernel explicitly uses `extern "sysv64" fn _start(info: *const BootInfo) -> !`. Do not merely change the function type with a Rust `transmute` and assume the registers are then correct.

The separate assembly shim is declared from Rust as `extern "efiapi" fn handoff(entry: u64, info: u64, stack_top: u64) -> !`. It therefore receives RCX=entry, RDX=info, R8=stack_top. It confirms the state set directly after EBS with `cli` and `cld`, loads an IDT descriptor with limit 0/base 0, takes the own stack (`RSP=R8`, aligned to 16 bytes), clears RBP, sets RDI=RDX and does `call RCX`. As a result RSP at the SysV entry is congruent to 8 modulo 16. No Windows shadow space is needed for that final SysV call. The shim does not return; an unexpected kernel return gives failure code 0x11 and then `cli; hlt` in a loop.

The empty IDT is exclusively an M1 fail-closed bootstrap: an exception/NMI leads to shutdown/triple fault and therefore a failed smoke test, not to an old firmware handler. M2 replaces this with own exception handlers, GDT/TSS and IDT before interrupts or userspace are allowed. M1 does not reload firmware segments; existing segment caches and CR3 are temporarily kept as bootstrap state. No firmware function is called anymore and no underlying memory is recycled. This is a deliberate short boot phase, not the final architecture.

Do not make this stack switch in an ordinary Rust function where the compiler still executes an epilogue. Use a separate assembly symbol via `global_asm!`/`include_str!`, with COFF-compatible directives in the loader. Kernel entry gets `#[no_mangle]` and `.text.entry`. No SIMD/FPU activation is needed for M1: both targets by default generate no use of those registers; introduce no target feature or floating-point dependency [B2–B3].

## 7. Serial output and machine test contract

COM1 base 0x3f8, polling, 115200 baud, 8N1. Own minimal port I/O module: interrupts off on the UART, DLAB on, divisor 1/0, LCR 0x03, FIFO 0xc7, MCR 0x0b; poll LSR bit 5 before every byte. At most 1.000.000 polls per byte; then failure exit, no endless wait loop. Use `out dx, al`/`in al, dx` for the UART and `out dx, eax` to 0xf4 for debug exit. A disabled/missing UART must not let a negative test hang indefinitely.

After a successful EBS, own code logs, in this order:

```text
BOOT:EBS_OK
KERNEL:ENTRY
BOOTINFO:OK
M1:PASS
```

The first line is produced in the loader after EBS; the next three in the real ELF kernel. Only the kernel may send `M1:PASS` and the success exit. A panic or validation error writes `M1:FAIL:<vaste_code>` where possible and exits 0x11; format no secrets or unbounded received text. The kernel also checks `.bss` zero initialisation and a known `.data` value before success.

`isa-debug-exit` uses the project values **0x10 = pass**, **0x11 = fail**. The host status is `(value << 1) | 1`: so **33** and **35** respectively; an ordinary host exit 0 is not a test pass [B10]. Only `run.py` translates (33 and all expected markers in order and no fail marker) to harness exit 0. A failure with status 35, signal, firmware prompt, triple fault, QEMU launch error or missing marker stays a fail. The raw QEMU status always stays in the log.

`run.py --smoke` semantically runs this command (concrete absolute paths via a subprocess argument list, no shell interpolation):

```sh
qemu-system-x86_64 \
  -machine pc-q35-10.0 \
  -accel tcg,thread=single \
  -cpu qemu64 -smp 1 -m 256M \
  -nodefaults -display none -monitor none -serial stdio \
  -nic none -no-reboot \
  -drive if=pflash,unit=0,format=raw,readonly=on,file=build/firmware/OVMF_CODE.fd \
  -drive if=pflash,unit=1,format=raw,file=build/run/OVMF_VARS.fd \
  -drive if=virtio,index=0,format=raw,readonly=on,file=fat:build/esp \
  -device isa-debug-exit,iobase=0xf4,iosize=0x04
```

The tool sets the working directory to the repository root; `fat:build/esp` is QEMU's read-only virtual FAT disk, not a real host disk. The ESP directory is a stable copy for the duration of the VM run; no builds that mutate it during execution. FAT contains the standard boot name `EFI/BOOT/BOOTX64.EFI` and `kernel.elf`; no disk partitioning tool is needed yet. Later, a real installation medium gets its own tested GPT/ESP builder. No `-no-shutdown`: this can disturb the expected test exit with QEMU implementations that use shutdown requests.

Hard timeout of **60 seconds** from QEMU start. `subprocess.run(..., timeout=60)` or equivalent terminates a hung process and keeps available stdout/stderr; a timeout is explicitly failed, never skipped. Keep start time, duration, version, arguments, return code, all artifact hashes and the development/reference profile used. The time limit is a CI budget, not a performance promise for the OS.

## 8. Acceptance gate: when M1 is done

| Test | Required evidence |
|---|---|
| Toolchain/target | Both binaries build with Rust 1.90.0 and `--locked`; loader is PE32+, kernel ELF64 ET_EXEC; no host OS imports |
| ABI | Compiler asserts for all offsets/size/alignment; host byte fixture; wrong magic/major/flags/range/stride rejected |
| ELF parser | Truncations, overflow, wrong machine/type, filesz>memsz, invalid entry, overlapping pages, RWX and DYNAMIC/INTERP/TLS rejected before writes |
| Smoke test | Own ELF entry after EBS, BootInfo/BSS/data correct, markers in order, QEMU 33 and harness 0 within 60 s |
| Image rejection | Mutated `kernel.elf` refuses before kernel entry, fail marker and QEMU 35; no `M1:PASS` |
| Retry | Test with deliberately old map key fails the first EBS, succeeds with a new map without new allocation; the normal build does not contain the injection |
| Error path | Explicit kernel panic reaches the failure code; a hang and deviating exit are treated as fail by the harness |
| Independence | After the first EBS no protocol/console/file calls; after success not a single UEFI call, no `Drop` with firmware effects |
| Repeatability | Three consecutive cold boots, each time a fresh VARS copy; hashes and raw logs kept |

Pure parser/protocol tests run on the host and prove exclusively that logic. QEMU tests must actually run the built loader and kernel; a Python model or simulated serial log does not count. After this document **no open architecture choice is needed to write M1**. Remaining work is concrete host preflight, implementation, compiler diagnostics, actually obtaining/hashing firmware and measuring the tests above. A failed test becomes a small registered implementation issue, not a reason to re-investigate the whole OS direction without evidence.

## 9. Sources and verification boundaries (APA 7)

Accessed on 26 October 2026. Design numbers, load address, schema, limits and test markers are own project decisions. Sources confirm the mechanisms used; they do not prove that the not-yet-written project code is correct.

- **B1.** The Rust Project. (2025, 18 September). *Announcing Rust 1.90.0*. https://blog.rust-lang.org/2025/09/18/Rust-1.90.0/
- **B2.** The Rust Project. (n.d.). *\*-unknown-uefi*. The rustc book. https://doc.rust-lang.org/rustc/platform-support/unknown-uefi.html
- **B3.** The Rust Project. (n.d.). *x86_64-unknown-none*. The rustc book. https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html
- **B4.** r-efi Contributors. (2026, 10 August). *r-efi 7.1.0*. Docs.rs. https://docs.rs/crate/r-efi/latest (the accessed version page explicitly mentions 7.1.0 and release history; the exact-version URL could not be fetched via the research tool).
- **B5.** r-efi Contributors. (2026). *Cargo.toml* [Source code; version field 7.1.0, rust-version 1.68]. https://github.com/r-efi/r-efi/blob/main/Cargo.toml — the release is also at https://github.com/r-efi/r-efi . `main` is mutable; the implementation downloads exactly 7.1.0, checks the packaged manifest and keeps the Cargo registry checksum. That download/check was not performed here.
- **B6.** QEMU Project. (2026). *Download QEMU*. https://www.qemu.org/download/ — mentions version 10.0.14; distribution overview https://download.qemu.org/ . Invocation: https://www.qemu.org/docs/master/system/invocation.html . The CLI was examined; the exact local machine/firmware combination has not yet been run.
- **B7.** TianoCore. (2025, 22 August). *edk2-stable202508* [Release]. https://github.com/tianocore/edk2/releases/tag/edk2-stable202508
- **B8.** TianoCore. (n.d.). *How to build OVMF*; *How to run OVMF*. https://www.tianocore.org/tianocore-wiki.github.io/development/tutorials-howto/how_to_build_ovmf.html ; https://www.tianocore.org/tianocore-wiki.github.io/development/tutorials-howto/how_to_run_ovmf.html
- **B9.** UEFI Forum. (2022). *UEFI specification 2.10: 7. Services—Boot Services*, §§7.2.1, 7.2.3, 7.4.4 and 7.4.6. https://uefi.org/specs/UEFI/2.10/07_Services_Boot_Services.html — chapter content consulted, including EBS retry and descriptor stride. The 2.11 URL returned an access block; therefore the reachable 2.10 text is explicitly used here, not invented 2.11 content.
- **B10.** QEMU Contributors. (n.d.). *hw/misc/debugexit.c* [Source code]. https://github.com/qemu/qemu/blob/master/hw/misc/debugexit.c — the accessed source contains `(val << 1) | 1`. Verification of the chosen local binary is additionally done by the smoke test; this is not a claim that master is the same source file as tag 10.0.14.
- **B11.** Xinuos. (n.d.). *ELF object file format: 7. Program loading* (System V gABI 4.2). https://gabi.xinuos.com/v42/elf/07-pheader.html — primary ELF64 PHDR layout, segment types, filesz/memsz, alignment and permissions consulted in substance. Using a public file format does not make the own kernel a derivative of an existing OS.

# Comparison of OS repositories and driver strategy

**Date:** 3 November 2026
**Baryon OS baseline examined:** `99d132768fd72276caa4a3b3e9a09f8f1909f0ed`
**Status:** documentation research and proposal; no architectural decision, code port or hardware test.

Addition after two further repositories were supplied: Baryon OS source
`9e13e7be17845e03256dd451de57419eb242e376` (3 November 2026).
Addition after the `rust-osdev` organisation reference: Baryon OS source
`47cc895b4de6bfde02f294399ebc777c67575d24` (3 November 2026).

## Conclusion

The examined repositories do not provide a ready-made broad collection of Linux
kernel drivers that work in Baryon OS without porting. ODI does contain several
standalone C driver implementations; `device-driver` is an active Rust toolkit
for generating device-specific register/command code. Both are possible sources
for targeted follow-up work, not a solution for the entire hardware matrix.
The proposed appropriate route for Baryon OS is to retain the independent
kernel, complete the fixed QEMU targets through VirtIO and use GOP (UEFI's
framebuffer interface) for basic display output. Then select a concrete physical
reference profile and port only suitable drivers or protocol code for that
specific target. A driver in a separate process fits the capability
architecture, but is genuinely isolated only if memory, MMIO, interrupts and
DMA are bounded; a correctly configured IOMMU is relevant to DMA.

This aims to keep the test matrix and number of device-specific implementations
manageable; no acceleration has been measured. It promises no broad support for
arbitrary PCs, Wi-Fi adapters or GPUs.

## What the five repositories actually show

| Repository and source examined | Structure and driver approach | What Baryon OS can adopt | Evidence boundary |
|---|---|---|---|
| [Ginux](https://github.com/gmkurtzer/ginux/tree/c86945b1ae246a7c9e4bd831a66e5179d55de95e), commit `c86945b` (3 November 2026) | Independent Rust x86-64 kernel, UEFI image, process/domain model and Linux userspace ABI. The README names VirtIO block/network on QEMU Q35. ADR 0002 defers Linux `.ko` compatibility; the README states that there is no stable third-party driver ABI and that general hardware coverage is future work. | Clear distinctions between an independent kernel, Linux program compatibility and drivers; real distribution workloads as integration tests; record limitations per supported VM profile. | The README names a Rocky Linux 10 systemd path, among other examples, but that is Ginux's own reported test evidence. No independent reproduction in this research. The “11 days” describe an agentic development method, not a universal driver solution. |
| [VibeOS](https://github.com/kaansenol5/VibeOS/tree/866bfb9b7fc554b5b8e0939b4626bcf0416b8b19), commit `866bfb9` (27 January 2026) | Independently built AArch64 hobby OS for QEMU and, according to the README, Raspberry Pi Zero 2W, with its own USB DWC2/HID and eMMC paths. The README lists no network or audio driver on the Pi and describes USB limitations, including no hotplug. | Select a concrete board and build a vertical slice with real device tests; document missing functions and failure behaviour as explicitly as the demonstration. | README claims were not rerun on QEMU or a Pi. The accompanying `CLAUDE.md` states that its status dates from session 40; those outdated technical notes were not used as current verification. |
| [vib-OS](https://github.com/viralcode/vib-OS/tree/73b64299bf6e591e738c58c9d1845f0babdd6f58), commit `73b6429` (29 January 2026) | Independent C/assembly kernel with its own virtio, video, network, NVMe, USB and other driver directories; primarily targets ARM64 and claims several architectures. The README contains contradictory x86-64 status: the architecture section names an x86-64 build, while the same source says it “will be released soon”; the latest commit removes x86-64 build instructions. | Maintain a concrete architecture/device matrix and distinguish “code exists”, “builds”, “boots in VM” and “tested on hardware”. | The README, screenshots and driver filenames do not establish that all named devices or platforms work. No builds or hardware tests performed. |
| [Open Driver Interface (ODI)](https://github.com/Open-Driver-Interface/odi/tree/ef58952c4a7b27338cb47bdb9401d4b6f970143c), commit `ef58952` (1 December 2023) | C framework with PCI/ACPI discovery and several driver implementations, including AHCI, XHCI, E1000 and serial. The quickstart requires porting OS-dependent functions and describes callbacks with generic pointers. README names MIT. | A specific driver, such as E1000 or AHCI, may later be a reading/porting candidate if the selected physical profile contains that device. Compare protocol choices and hardware sequencing with Baryon OS's bounded driver contracts. | This is neither a stable universal driver ABI nor a Linux `.ko` wrapper. The repository head has not been updated since the stated 2023 commit. Documentation names `source/` and a test environment, while the examined tree places implementation in `src/`. No build, code audit or compatibility test performed; an MIT statement does not establish quality or security. |
| [diondokter/device-driver](https://github.com/diondokter/device-driver/tree/036fb189f382d1ced62fa2695d43b3a69dcdaa73), commit `036fb18` (24 October 2026) | Active Rust toolkit with a `no_std` runtime and DDSL: a device specification can generate typed register, field, buffer and command access/APIs. The main crate is dual MIT/Apache-2.0; macros and `embedded-io` integration are optional features. | Possible later register generator, not directly compatible with the project pin: the upstream workspace manifest requires Rust 1.95 and the runtime inherits that requirement. | Does not generate a complete PCI/USB driver, bus enumeration, interrupt service, DMA management, reset policy or capability boundary. It does not automatically implement VirtIO queues or network protocols. Required Rust 1.95 differs from Baryon OS's 1.90.0; dependency policy, build chain and concrete `no_std` target have also not been tested. No dependency added. |

Primary toolchain sources, checked source versions on 3 November 2026:
diondokter/device-driver, *workspace Cargo.toml*, version `036fb189`,
[upstream rust-version](https://github.com/diondokter/device-driver/blob/036fb189f382d1ced62fa2695d43b3a69dcdaa73/Cargo.toml);
Baryon OS, *rust-toolchain.toml*, version `1c687206`,
[project pin Rust 1.90.0](https://github.com/simonvanieperen/nieuw-os/blob/1c6872063fab055211eff48fce11496db39728be/rust-toolchain.toml).
This is a manifest comparison, not a performed port or compatibility trial.

The principal correction to the cited Google summary: the current Ginux source
describes Linux *userspace* compatibility (Linux programs running above its own
kernel), not a working wrapper for thousands of Linux kernel drivers. Ginux's
own ADR explicitly states that binary Linux modules remain deferred. Linux also
documents that the kernel promises no stable internal module/driver API; this
differs from the much more stable syscall ABI for user programs. Making a Linux
driver run against another kernel therefore requires more than a thin syscall
layer: bus, memory, DMA, interrupt, device and subsystem contracts must fit.

## What the rust-osdev organisation contributes

The organisation is a collection of separate Rust libraries, not an integrated
driver stack. The most useful projects for the current x86-64/UEFI route are:

| Project | What the source actually provides | Relevance and limitation for Baryon OS |
|---|---|---|
| [acpi](https://github.com/rust-osdev/acpi/tree/386035c1badb74e8bd5f3a63b385e77c60d79b1f) `386035c` | Rust parser/API for ACPI tables and AML; a limited API works without an allocator, richer functions/AML require alloc. MIT/Apache-2.0 licence; current manifest is version `7.0.0-pre1`, Rust 2024 edition. | Promising component for platform/interrupt/power inventory, but not a PCI or device-specific driver. Prerelease status, dependency tree and firmware deviations require a concrete test before adoption. |
| [pci_types](https://github.com/rust-osdev/pci_types/tree/eff856b6bab81adbe1bda0ab750009f3786cb8d3) `eff856b` | `pci_types` 0.10.1: `no_std` types and helpers for PCI addresses, configuration registers, capabilities and device classes; no PCI configuration-access backend or driver binding. MIT/Apache-2.0. | Potentially useful for future generic PCI enumeration. It does not replace safe configuration-space access, BAR allocation, interrupt or DMA policy. |
| [xhci](https://github.com/rust-osdev/xhci/tree/f2254c86d5ba12754259bf0ae483c6c8dc821aad) `f2254c8` | `xhci` 0.9.2: `no_std` register, context, ring and extended-capability types for xHCI. The README explicitly states that an OS can use it to implement its own USB device driver. MIT/Apache-2.0; the latest commit in the examined history dates from 2024. | Useful as a layer beneath a future project xHCI host-controller driver, not a complete controller, DMA, interrupt, enumeration or USB HID driver. Activity and API require reassessment before use. |
| [usb](https://github.com/rust-osdev/usb/tree/75e219179df7e0961aa42e18774beb8878765341) `75e2191` | Separate 0.1.0 helper packages, including `usb_request` and `usb_hid_item`, for USB requests and HID item/usage data; no host controller. The latest commit found dates from 2022. | May accelerate parser/descriptor work after the USB transport boundary exists. First test older crates and parser behaviour against malformed/truncated input; not a replacement for Baryon OS USB enumeration or xHCI. |
| [uart_16550](https://github.com/rust-osdev/uart_16550/tree/770cbd28330de1cc461d62d8fc8ac5814c673304) `770cbd2` | Real `no_std`, allocation-free 16550 driver for x86 port I/O, MMIO and custom backends, including configuration, RX/TX and interrupts; MIT/Apache-2.0. | Technically more concrete than most repositories, but manifest `0.9.0` requires `rust-version = 1.91.0`; Baryon OS is pinned to Rust 1.90.0. Current upstream therefore cannot be used directly with the pinned project toolchain without an explicit toolchain/version decision and compatibility tests. |
| [virtio-spec-rs](https://github.com/rust-osdev/virtio-spec-rs/tree/ad565bd701e93fa47bc288d45701e1ae53ac3c12) `ad565bd` | `virtio-spec` 0.4.0: Rust definitions from the VirtIO specification; README explicitly states that the crate neither prescribes nor implements the drivers above it. MIT/Apache-2.0. | Possible reference/type source, but not a VirtIO driver or faster route to physical devices. Baryon OS already has its own `virtio-core` and targeted VirtIO drivers; do not introduce a second protocol source without a concrete need and contract comparison. |

The organisation also includes `bootloader`, `uefi-rs`, `x86_64`, `pic8259` and
`apic`, among others. These provide boot, firmware, architecture and interrupt
components respectively, not a general collection of modern GPU, Wi-Fi, NVMe
or laptop drivers. An `xhci` register crate or `virtio-spec` definition must not
be counted as a working driver.

**Practical assessment:** this is the most valuable source examined so far for
reusable Rust components aligned with the chosen programming language and
`no_std` direction. It may later reduce parser, type and register work. The best
candidate depends on the next concrete gate: PCI/ACPI for platform inventory,
xHCI types for a USB host-controller port, or a serial driver once the toolchain
boundary has been resolved. None of these crates alone provides an integrated,
capability-bounded driver stack. Add nothing before checking origin, exact
version, licence, `no_std`/MSRV, build scripts, `unsafe` boundaries and negative
native QEMU tests.

## Assessment of the proposed shortcuts

| Proposal | Assessment for Baryon OS |
|---|---|
| Wrapping Linux drivers | **Not a quick route.** A Linux userspace ABI helps applications, not kernel drivers. A compatibility layer for `.ko` modules would introduce a substantial Linux kernel/subsystem model and a new trust boundary. Investigate this, at most, later as a bounded feasibility trial; do not adopt an existing kernel as the foundation. Every code port requires separate licence, origin, API and security review. |
| rust-osdev libraries | **Targeted accelerator for components.** `acpi`, `pci_types`, `xhci` and USB helper packages may reduce some parsing/type/register work, but are not ready-made drivers. `uart_16550` is a more concrete driver, but upstream requires Rust 1.91 and therefore does not fit the current Rust 1.90 pin. `virtio-spec-rs` contains specification definitions, not a VirtIO driver. |
| Using ODI | **Possible per driver, after port and security review.** The C code may provide concrete controller logic, but expects OS bindings; the generic pointer/callback interface is not a Baryon OS capability model. Select a driver only after choosing the device and test it on the exact QEMU/hardware profile. |
| Using `device-driver` | **Possible as a later code generator, not a driver library for all hardware.** The examined version requires Rust 1.95 versus project pin 1.90.0. First resolve this incompatibility explicitly; do not silently raise the project toolchain. DDSL does not solve buses, interrupts, DMA, device lifecycle or protocol stacks. An isolated `no_std` trial and dependency/feature/build-script review remain necessary before any adoption. |
| VirtIO and UEFI GOP | **Directly useful within their boundaries.** VirtIO provides standardised devices to VMs and fits QEMU/CI; it is not a physical NIC/NVMe/GPU driver for ordinary PCs. GOP lets the loader pass on a firmware mode and framebuffer, useful for initial pixels without GPU acceleration. It does not replace a native GPU driver, modesetting, audio, Wi-Fi or post-boot USB stack. |
| AI as a driver-port factory | **Only as assistance with one port at a time.** An agent can prepare specifications, driver code and tests, but must pass the same source, licence, API, DMA and negative-test gates as handwritten code. Automatic rewriting does not establish correctness or compatibility. |
| Userspace drivers | **Architecturally appropriate, but neither free nor automatically secure.** A process crash can be restarted locally; DMA previously executed by the device may still corrupt memory. Use only limited capabilities, pinned/protected buffers, reset/quiescence contracts and, where needed, a correctly configured IOMMU. A process boundary alone does not prevent DMA. |
| Host/browser-based WASM solution as the OS foundation | **Not appropriate.** When an existing host OS or browser supplies hardware drivers, this is not an independent Baryon OS with its own kernel. This does not rule out a subsequently and explicitly designed WASM app runtime above the project's own kernel. |

## Appropriate route for Baryon OS

1. **Complete the VM hardware slice first.** Retain x86-64/UEFI/QEMU as the development target and build on the existing VirtIO block, RNG, network and sound components. Follow the binding `VM-FR` matrix; report separately for each device whether only a parser, host model, cross-compilation or real QEMU I/O was tested.
2. **Use firmware only for startup.** GOP can provide the first screen; the existing G0 specification already bounds framebuffer, geometry and post-`ExitBootServices` behaviour. Add no firmware dependency for later rendering or input without an explicit contract.
3. **Choose one reference device before porting physical drivers.** Inventory UEFI/ACPI, PCI IDs, storage, wired network, USB controller and IOMMU. Begin with a limited set of standard interfaces. Select a Wi-Fi chip or GPU only once the exact component, firmware path, licence and recovery/test profile are known. No physical installation or flashing follows from this proposal.
4. **Isolate device services through explicit authority.** Begin with a virtual device reproducible in QEMU; also test malformed descriptors, missing devices, timeouts, reset and restart. Add userspace drivers only once capability and DMA ownership are genuinely enforced in the kernel.
5. **Port selectively; do not copy uncritically.** First reuse standard specifications and small portable algorithms. For driver code: establish licence and origin, replace Linux-specific connections with a narrow Baryon OS interface and test the resulting implementation against real VM I/O. A driver name or successful build is not hardware acceptance.

This route changes neither the independent kernel nor the Security Floor,
privacy requirements or fixed acceptance matrix. In the snapshot
[progress tracker on main78ff7c03, 3 November 2026](https://github.com/simonvanieperen/nieuw-os/blob/78ff7c03b015ad16fe665ae7df9fe239d078e08d/docs/progress.json),
H01 is accepted, but full release criteria FR-H01 through FR-H03 remain
independent acceptance gates; partial driver work earns no new points.

## References

### Primary repository sources

- Ginux, `README.md`, commit [`c86945b`](https://github.com/gmkurtzer/ginux/blob/c86945b1ae246a7c9e4bd831a66e5179d55de95e/README.md), and [ADR 0002: Unified Boot Image and Driver Boundary](https://github.com/gmkurtzer/ginux/blob/c86945b1ae246a7c9e4bd831a66e5179d55de95e/docs/decisions/0002-unified-image-and-driver-boundary.md).
- VibeOS, `README.md`, commit [`866bfb9`](https://github.com/kaansenol5/VibeOS/blob/866bfb9b7fc554b5b8e0939b4626bcf0416b8b19/README.md).
- vib-OS, `README.md` and change history, commit [`73b6429`](https://github.com/viralcode/vib-OS/commit/73b64299bf6e591e738c58c9d1845f0babdd6f58).
- Open Driver Interface, [README](https://github.com/Open-Driver-Interface/odi/blob/ef58952c4a7b27338cb47bdb9401d4b6f970143c/README.md), [quickstart/port contract](https://github.com/Open-Driver-Interface/odi/blob/ef58952c4a7b27338cb47bdb9401d4b6f970143c/docs/quickstart.md) and [dependency/port notes](https://github.com/Open-Driver-Interface/odi/blob/ef58952c4a7b27338cb47bdb9401d4b6f970143c/docs/dependencies.md).
- Dion Dokter, [device-driver toolkit README](https://github.com/diondokter/device-driver/blob/036fb189f382d1ced62fa2695d43b3a69dcdaa73/README.md), [runtime manifest](https://github.com/diondokter/device-driver/blob/036fb189f382d1ced62fa2695d43b3a69dcdaa73/device-driver/Cargo.toml) and [repository licence](https://github.com/diondokter/device-driver/blob/036fb189f382d1ced62fa2695d43b3a69dcdaa73/LICENSE-MIT).
- rust-osdev, pinned [ACPI README](https://github.com/rust-osdev/acpi/blob/386035c1badb74e8bd5f3a63b385e77c60d79b1f/README.md) and [Cargo.toml](https://github.com/rust-osdev/acpi/blob/386035c1badb74e8bd5f3a63b385e77c60d79b1f/Cargo.toml), [PCI Cargo.toml](https://github.com/rust-osdev/pci_types/blob/eff856b6bab81adbe1bda0ab750009f3786cb8d3/Cargo.toml), [xHCI README](https://github.com/rust-osdev/xhci/blob/f2254c86d5ba12754259bf0ae483c6c8dc821aad/README.md) and [Cargo.toml](https://github.com/rust-osdev/xhci/blob/f2254c86d5ba12754259bf0ae483c6c8dc821aad/Cargo.toml), [USB repository](https://github.com/rust-osdev/usb/tree/75e219179df7e0961aa42e18774beb8878765341), [uart_16550 Cargo.toml](https://github.com/rust-osdev/uart_16550/blob/770cbd28330de1cc461d62d8fc8ac5814c673304/Cargo.toml), [virtio-spec-rs README](https://github.com/rust-osdev/virtio-spec-rs/blob/ad565bd701e93fa47bc288d45701e1ae53ac3c12/README.md) and [Cargo.toml](https://github.com/rust-osdev/virtio-spec-rs/blob/ad565bd701e93fa47bc288d45701e1ae53ac3c12/Cargo.toml), and [organisation repositories](https://github.com/orgs/rust-osdev/repositories).

### Technical specifications and documentation

- Linux Kernel Documentation, version 6.12, [The Linux Kernel Driver Interface](https://www.kernel.org/doc/html/v6.12/process/stable-api-nonsense.html) and [Driver implementer's API guide](https://www.kernel.org/doc/html/v6.12/driver-api/index.html). Internal kernel/driver APIs and the syscall ABI are different interfaces.
- OASIS, *Virtual I/O Device (VIRTIO) Version 1.2*, [specification](https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html).
- UEFI Forum, *UEFI Specification 2.11*, [chapter 12: Console Support, including Graphics Output Protocol](https://uefi.org/specs/UEFI/2.11/12_Protocols_Console_Support.html).
- Baryon OS, [G0 native graphics foundation](../../implementation/GRAPHICS-G0.md), [VirtIO-net task contract](../../implementation/VIRTIO-NET.md), [full release and driver acceptance](../../FULL-TEST-RELEASE.md), [virtio-blk](../../../drivers/virtio-blk/README.md) and [virtio-snd](../../../drivers/virtio-snd/README.md).
- Baryon OS project, main version `78ff7c03`, 3 November 2026:
  [VirtIO RNG implementation](https://github.com/simonvanieperen/nieuw-os/blob/78ff7c03b015ad16fe665ae7df9fe239d078e08d/drivers/virtio-rng/src/lib.rs)
  and [B01 RNG/keyring contract](https://github.com/simonvanieperen/nieuw-os/blob/78ff7c03b015ad16fe665ae7df9fe239d078e08d/docs/implementation/KEYRING-RANDOM.md).
  This substantiates existing components, not physical RNG support.

External repository claims were read on 3 November 2026; the commit IDs above
pin additional source code. Linux, OASIS and UEFI sources were consulted as
technical references; the UEFI host refused direct page retrieval in the earlier
session, so the specification's table of contents and the existing Baryon OS G0
specification were used for the GOP boundary. No external repositories were
cloned or built, no code incorporated, no toolkit added as a dependency, and no
QEMU or physical hardware tests performed. The current toolchain is Rust 1.90.0;
in particular, `uart_16550` 0.9.0 with Rust 1.91.0 was therefore not treated as
compatible.

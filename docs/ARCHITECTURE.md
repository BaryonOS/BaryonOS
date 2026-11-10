# Architecture and required knowledge

> Revision 5 update: initial implementation choices are fixed in [implementation/KERNEL-AND-SECURITY-CONTRACT.md](implementation/KERNEL-AND-SECURITY-CONTRACT.md). Use that document for direct implementation; this file preserves the ambition and background for subsequent phases.

Historical status of this background file: research proposal of 26 October 2026. The project's own kernel has since been implemented through M4 and tested in the documented QEMU profile; see PROJECT-STATE.md and verification/m2-m4/. The subsequent-phase choices below remain proposals unless current evidence explicitly covers them.

## 1. What an independent OS means here

The goal is an independent kernel, system services and user experience, without a Linux, BSD, XNU, Zircon or seL4 kernel underneath. It will therefore not be a Linux distribution with a different desktop.

An independent OS need not reinvent every tool. Rust/LLVM, Git, QEMU, UEFI firmware, public hardware standards and carefully selected libraries are permitted. A development computer running Windows or Linux likewise does not make the target OS a derivative of its host OS. Track components, versions, licences and provenance. Do not silently adopt existing kernel code.

Principles such as processes, virtual memory, microkernels and capabilities are established technical knowledge. Originality should primarily arise from a coherent product, an independent implementation and its own interaction design. A novel, untested security mechanism is not an advantage in itself.

There is a tension between implementing everything independently and achieving high assurance quickly. A new kernel starts without the proven properties and years of defect correction of mature projects. seL4 can serve as a research reference; its proofs do not apply to Baryon OS code [A7]. If high assurance ultimately becomes more important than an independent kernel, that scope choice warrants explicit reconsideration.

## 2. Recommended initial platform

| Component | Initial choice | Rationale and limit |
|---|---|---|
| Processor | x86_64, one virtual CPU | A bounded initial implementation. A separate opt-in profile starts APs within defined bounds (ADR-070); SMP scheduling follows later. |
| Machine | QEMU, a recorded q35 machine version | Repeatable virtual hardware; no dependency on the particular development laptop. |
| Accelerator | TCG as the CI baseline; local acceleration optional | TCG requires no available hardware virtualisation. Do not use TCG results as evidence of real-hardware performance. |
| Firmware | OVMF/UEFI, with versions and hashes recorded | Standard boot interface. Firmware is a dependency and part of the threat model. |
| Loader | Small independent UEFI loader in Rust | Loads the independent kernel and passes a validated memory map and boot information. |
| Loader target | `x86_64-unknown-uefi` | Produces a UEFI program; it is not itself an independent kernel. |
| Kernel target | `x86_64-unknown-none`, `#![no_std]` | Independent bare-metal ELF kernel, without host system calls. |
| Initial output | Serial log with fixed success/error codes | Automatically verifiable without a graphical desktop. |
| Devices | Timer/interrupt controller first; then limited virtio devices | Smaller surface than immediately supporting real Wi-Fi, GPU and USB hardware. |
| Storage | Generated test images only | No real disks, partitions, USB flashing or phone unlocking in the initial phases. |
| Network | Disabled during the initial boot and isolation phases | Add networking only with its own threat model and tests. |

Source basis: Rust documents separate UEFI and bare-metal targets. `x86_64-unknown-none` lacks `std` and a default allocator; `alloc` requires a custom allocator [A1–A2]. QEMU provides system emulation, different accelerators and virtio devices; machine and architecture choices are explicit and not interchangeable [A3–A4]. Virtio is a standardised interface for virtual devices [A5].

The loader must correctly handle the transition after `ExitBootServices`. Memory previously managed by firmware may be reused only under the handover contract. A screenshot showing 'Hello world' while firmware services still do all the work must not be reported as an independently operating kernel. The exact boot ABI, ELF segment validation and memory-map semantics require an ADR before implementation.

## 3. Architectural direction: a small capability microkernel

Keep the privileged core small: address spaces, threads/scheduling, interrupts, bounded IPC and enforceable access to kernel objects. Filesystems, network services, package management, graphical composition and as many drivers as possible run as isolated processes. This is a research direction; IPC complexity and performance must be measured.

A capability here is a kernel-checked right to a specific object, with explicit operations. For example, the user releases one selected file to an app; the app does not automatically gain access to the entire home directory. Guessing a value or copying it from another process must not grant access. Fuchsia illustrates process-bound handles with separate rights; it is a reference, not an ABI to adopt [A6].

Design rules:

1. Applications have no global file, network, camera or microphone access by default.
2. Rights can be transferred only explicitly and never with greater authority than the sender possesses.
3. Define object lifetimes, attenuation, handle reuse and revocation. Revocation can stop future access but cannot recover data already read.
4. IPC has bounded message sizes, validated lengths, quotas, time limits and clear ownership transfer.
5. A system service is not automatically trusted with all data. Analyse which components must be trusted for each security claim.
6. A userspace driver is not fully isolated if DMA can reach outside its memory. IOMMU management and permitted buffers must be part of real-hardware support.
7. No general root-like authority in an ordinary app. Administrative actions use a bounded management interface; physical owner control is designed separately.
8. Rust reduces certain memory errors but does not prove a correct privilege policy. Every `unsafe` block describes its invariant, assumptions, call conditions and validation evidence.

The complete security treatment is in [SECURITY-AND-QUANTUM.md](SECURITY-AND-QUANTUM.md).

## 4. A product direction beyond a different desktop

Revised privacy boundary (27 October 2026): storage is local or end-to-end encrypted on an explicitly paired personal NAS or a NAS belonging to friends/family. Internet is supported product functionality with separate permissions; remote NAS traffic requires consent bound to the verified peer. NAS administration grants capacity/deletion rights, not a content key. AI inference remains exclusively on the device. Vendor cloud, silent relays and cloud fallback are not OS options. See [LOCAL-PRIVACY-CONTRACT.md](LOCAL-PRIVACY-CONTRACT.md). Third-party apps may use the internet through bounded permissions; that does not grant automatic access to OS or NAS data.

Recommended initial product hypothesis: a locally operating personal system with understandable, temporary access for each task. One consistent permission model for apps and future AI assistants, with a visible overview of which app can currently access which data. This is a design proposal, not a claim of worldwide novelty.

| Goal | Concrete product behaviour | Evidence to investigate |
|---|---|---|
| Usable | Works without an online account; ordinary tasks through consistent settings; sound recovery route | User research measuring task completion, errors and recoverability |
| Privacy-respecting | No mandatory vendor cloud or OS telemetry; E2EE to a personal NAS and optional opaque client-side encrypted objects at an explicitly selected external provider; administrator/provider receives no content key; AI without network permission | Test traffic measurements, NAS/provider outages, false endpoints, provider credentials without a key, tampering/rollback and silent fallback attempts |
| Configurable | Replaceable shell, shortcuts and services; exportable configuration | Safe defaults remain intact during ordinary personalisation |
| European autonomy | Open formats, local language/locale, independent update keys and interchangeable services | Works without a mandatory Apple, Google or Microsoft account or central service |
| Accessible | Keyboard operation, scalable text, high contrast, later a semantic accessibility tree and screen reader | Tests with users and assistive technology |
| Lightweight and fast | Limited background work, bounded memory use, responsive input | Recorded RAM, CPU, latency and energy profiles on reference hardware |
| Broadly applicable | Shared core and API, separate desktop/touch/embedded profiles | The same contract tests for each supported platform port |

Copying window decorations is not necessary to support familiar actions. Let tasks guide the design. A complete browser engine, office suite and app store are not initial kernel milestones. Their quality and compatibility may later matter more for adoption than the kernel.

## 5. Translating every device into verifiable support

One source tree can support multiple architectures; one unchanged kernel file does not automatically run on every processor. Hardware also differs within the same ISA. An ARM64 kernel is therefore not yet a usable phone OS.

| Platform | Plan | Conditions |
|---|---|---|
| QEMU x86_64 | First supported target | Recorded machine, firmware, CPU features and devices |
| One real x86_64 PC | After VM isolation and recovery tests | Documented hardware, separate test disk, safe boot/recovery route, DMA policy |
| QEMU AArch64 | First portability experiment after core contracts | Independent interrupt, MMU, timer, context-switch and boot code |
| RISC-V 64-bit | Possible later port | Explicit ISA extensions, privilege environment, board interface and test set |
| Smartphones/tablets | Only specifically selected models | Unlockability, boot chain, SoC/GPU/touch/camera/modem drivers, firmware and power management |
| Microcontrollers without an MMU | Different security profile or outside scope | Do not promise identical process isolation; potentially a separate MPU/RTOS approach |
| Quantum hardware | Investigate from design stages Q0–Q4: local execution and an independent OS on a hybrid quantum computer | Exact execution model and local hardware required; see QUANTUM-RESEARCH.md |

Android documents that many devices ship locked and that unlocking follows device policy and erases data [A8]. HAL documentation explicitly requires vendors to implement device-specific functionality [A9]. **Support for the OnePlus 13 has not been established.** A model-specific port, driver inventory and tested boot/recovery route are missing. Do not flash the device at this stage.

Compatibility with existing apps is a separate product project. Native apps require an SDK, API/ABI and distribution path. Web apps require a port of a large browser engine. Windows, Android and Linux apps require compatibility layers or guest VMs, licence checks and additional isolation; support does not follow from choosing Rust or a microkernel. An optional Linux guest differs from using a Linux kernel as the foundation and is added only after an explicit scope decision.

## 6. Knowledge Codex requires

Codex does not need a large knowledge dump replacing every book. It needs the correct version of primary documentation, repository contracts, measurable requirements and the ability to test each assumption. Every task brief identifies the relevant source and chapter.

| Domain | Required knowledge | Project knowledge to retain | Demonstrable result |
|---|---|---|---|
| Computer architecture | Privilege levels, MMU, interrupts, TLB, caches, atomics, DMA, memory ordering | CPU/board profile, feature baseline, manual references | Positive and negative hardware tests |
| Boot/linking | UEFI, ELF/PE, linker layout, calling conventions, boot-services transition | Boot ABI, segment rules, memory-map contract | Boot after firmware handover; corrupted image rejected |
| Rust/systems | Ownership, lifetimes, `no_std`, allocators, FFI, inline assembly, UB | Toolchain pin, dependency policy, `unsafe` register | Host tests for pure logic plus target tests |
| Kernel memory | Page frames, mappings, W^X, user/kernel boundary, object lifetimes | Invariants and address-space layout | No overlap; access faults isolated |
| Scheduling | Threads, context switches, timers, priorities, fairness, later SMP | Scheduler contract and quotas | Progress and recovery under fault load |
| Authority/IPC | Capabilities, delegation, revocation, confused deputy, TOCTOU | Syscall/IPC/permission contracts | Unauthorised access demonstrably fails |
| Drivers | PCI, virtio, DMA, interrupts, buffers, reset/power | Supported device IDs and features | Malformed input and driver crashes contained |
| Storage | Block I/O, metadata, crash consistency, flush, recovery | On-disk format and migration policy | Power-cut/fault injection with consistent outcomes |
| Network | Ethernet/IP, TCP/UDP, DNS, TLS, resource exhaustion | Stack choice, port policy, parser contracts | Protocol tests, fuzzing, network disabled by default |
| Cryptography | Entropy, key management, current standards, PQC, replay/downgrade | Crypto inventory and library/parameter decisions | Known test vectors and failure paths |
| Updates | Verification, key rotation, A/B slots, rollback versus anti-rollback | Update manifest and recovery design | Interrupted/malicious update permits recovery |
| Graphics/input | Compositor, fonts, Unicode, touch, accessibility, GPU | UI/input contracts and threat model | Responsiveness, input isolation, accessible tasks |
| Apps/ecosystem | Executable loader, SDK, ABI versions, packages, signing | App manifest and compatibility policy | Small native app without hidden host dependency |
| Quality | Fuzzing, model checking, property tests, trace analysis, reproducible builds | Acceptance criteria, test logs, regression baselines | Another builder reproduces the outcome |
| Governance | Licences, provenance, maintainer roles, vulnerability disclosure | Licensing policy, contribution policy, decision log | Verifiable provenance and release procedure |

Rust's bare-metal target does not support ordinary execution of the Rust test suite [A1]. Design two test layers: host-executable pure algorithms and integration tests that boot the actual kernel in QEMU. A passing host test does not prove that kernel boot works.

## 7. Primary sources (APA 7)

Retrieval date for changing documentation: 26 October 2026. This file contains project design choices based on these sources; they do not endorse the project.

- **A1.** The Rust Project. (n.d.). *x86_64-unknown-none*. The rustc book. https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html
- **A2.** The Rust Project. (n.d.). *\*-unknown-uefi*. The rustc book. https://doc.rust-lang.org/rustc/platform-support/unknown-uefi.html
- **A3.** QEMU Project. (n.d.). *Introduction*. QEMU documentation. https://www.qemu.org/docs/master/system/introduction.html
- **A4.** QEMU Project. (n.d.). *Invocation*. QEMU documentation. https://www.qemu.org/docs/master/system/invocation.html
- **A5.** OASIS. (2022, July 1). *Virtual I/O Device (VIRTIO) version 1.2: Committee specification 01*. https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html
- **A6.** Fuchsia Authors. (n.d.). *Zircon handles*. https://fuchsia.dev/fuchsia-src/concepts/kernel/handles
- **A7.** seL4 Foundation. (n.d.). *Verification*. https://sel4.systems/Verification/
- **A8.** Android Open Source Project. (n.d.). *Lock and unlock the bootloader*. https://source.android.com/docs/core/architecture/bootloader/locking_unlocking
- **A9.** Android Open Source Project. (n.d.). *Hardware abstraction layer (HAL) overview*. https://source.android.com/docs/core/architecture/hal
- **A10.** UEFI Forum. (2024). *UEFI specification 2.11*. https://uefi.org/specs/UEFI/2.11/

The UEFI table of contents was consulted; the specific boot-services chapter could not be retrieved in full during this research. The implementation agent must actually read that chapter before writing boot code.

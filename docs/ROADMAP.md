# Roadmap — from research system to maintainable operating system

This roadmap defines the long-term technical and product phases for Baryon OS. The fixed acceptance score is maintained in [PROGRESS.md](PROGRESS.md) and [progress.json](progress.json). Current execution status belongs in [PROJECT-STATE.md](PROJECT-STATE.md); detailed release acceptance belongs in [FULL-TEST-RELEASE.md](FULL-TEST-RELEASE.md).

A booting prototype, a useful experimental OS, and a maintainable daily-use operating system are different outcomes. Milestones therefore describe **results and exit evidence**, not percentages of elapsed work.

## Fixed 100-point product roadmap

| Product track | Points |
|---|---:|
| Independent kernel and process isolation | 15 |
| Storage and data recovery | 15 |
| Hardware, drivers, and power | 12 |
| Native Material and accessibility | 15 |
| Internet and network services | 10 |
| Security, privacy, and updates | 12 |
| Applications, SDK, and installation | 8 |
| Personal NAS and multiple devices | 5 |
| Local AI and quantum runtime | 3 |
| Daily use and maintainable release | 5 |
| **Total** | **100** |

The tracks can progress in parallel. Milestone numbers are technical phases, not percentage steps: M6 does not mean 60% complete, and a successful demo does not imply release readiness.

## Technical milestones

| Milestone | Required outcome | Acceptance evidence | Decision gate |
|---|---|---|---|
| **M0 — Preparation** | Reproducible development environment, pinned QEMU/firmware/Rust versions, architecture decisions, and threat model | Tool inventory, build instructions, checksums, and no hidden dependency on a personal host configuration | Continue only when a second clean environment can reproduce the setup or missing evidence is explicitly open |
| **M1 — Independent boot** | Custom UEFI loader loads the custom Rust kernel and reaches a fixed success state | Automated QEMU run, serial output, exit status, and controlled missing/corrupt-kernel failures | Do not extend the boot slice while basic boot remains unreliable |
| **M2 — Memory and exceptions** | Custom frame management, page tables, and reliable exception handling | Allocation boundaries, invalid mapping, page-fault behavior, reserved-memory protection | Do not introduce user processes before the memory invariants are sufficiently established |
| **M3 — Processes and isolation** | Multiple user-mode processes with separate address spaces, basic scheduling, and syscall boundary | Process A cannot read/write B; one process can fail while another continues; invalid syscall memory does not crash the kernel | Broken isolation blocks progression regardless of demo quality |
| **M4 — Capabilities and IPC** | Bounded kernel objects and a versionable IPC contract | Invalid/stale handles, rights reduction, revocation, message fuzzing, and quota tests | Security review should actively attempt to break the stated guarantees |
| **M5 — First system service** | Userspace service plus first virtio driver on test images | Service/driver restart, malformed descriptors, boundary lengths, error codes, and buffer tests | Virtual DMA evidence is not physical-hardware evidence |
| **M6 — Storage and Recovery** | Persistent user storage with scalable documents, catalog/history, fault handling, backup/export, and controlled update/recovery design | Fault injection, cold recovery, full-storage cases, corruption handling, rollback/downgrade tests, independent disk checks | Daily-use claims remain blocked without recovery and independent review |
| **M7 — Native Product Integration** | Desktop/session, input, Material workflows, settings, accessibility, and at least one useful native application integrated on the independent OS | Native task tests, accessibility checks, visible/revocable application authority, recovery behavior, measured interaction budgets | Product direction follows user/task evidence; do not scale the app ecosystem before the core experience is usable |
| **M8 — Networking and Security Hardening** | Bounded network stack/services, maintained cryptography, key management, privacy contract, update trust chain, and targeted security review | Protocol/fuzz tests, negative handshakes, traffic capture, no unintended listeners, key/update chain evidence, independent findings resolved | No sensitive-data safety claim without scoped audit and published residual risks |
| **M9 — First Reference PC** | One documented physical reference system works end to end | Boot, drivers, input, graphics/audio as applicable, DMA boundaries, sleep/wake, power, network, storage, and recovery on that exact hardware | Support matrix must state exactly what is and is not supported |
| **M10 — Second Architecture** | The core process/capability/IPC model runs on a second ISA, initially for example AArch64 under QEMU | The same core contract tests pass with architecture-specific differences documented | Generalize abstractions only after real architecture differences are observed |
| **M11 — Experimental Community Release** | Reproducible experimental release, SDK, contributor path, support policy, security reporting, licensing/SBOM, and known-issues documentation | Independent build reproduction, release artifacts, documented supported scope, responsible reporting channel, maintenance process | Production classification follows evidence, not project ambition |

## Parallel product track

Product research begins before the kernel is complete and must not be mistaken for native implementation.

| Stage | Result | Decision evidence |
|---|---|---|
| **P0 — Identity and principles** | Clear product promise, initial audience, European orientation, and design principles | The product value can be explained without reducing Baryon OS to a different desktop theme |
| **P1 — Interaction concepts** | Competing interaction concepts evaluated against the same representative tasks | Differences, risks, and hypotheses are explicit rather than chosen by preference alone |
| **P2 — Experiential prototype** | High-quality interactive prototype of the strongest direction | Users understand navigation, permissions, and data location; measured friction feeds revisions |
| **P3 — Design system and application contract** | Coherent typography, motion, accessibility, privacy explanation, SDK/UI principles, and task/object interfaces | Requirements for kernel, graphics, services, and developer tooling are concrete |
| **P4 — Native translation** | Selected flows operate on the independent OS | The same tasks work natively and differences from the prototype are tracked explicitly |

## Parallel research tracks

Post-quantum protection, local quantum execution, local AI, privacy/NAS behavior, and additional hardware architectures remain explicit research tracks. A host prototype or simulator proves only its own scope. Native integration and physical-device support require separate acceptance evidence.

Key sources:

- [Security and post-quantum design](SECURITY-AND-QUANTUM.md)
- [Quantum research](QUANTUM-RESEARCH.md)
- [Local privacy contract](LOCAL-PRIVACY-CONTRACT.md)
- [Product and sovereignty](PRODUCT-AND-SOVEREIGNTY.md)
- [Architecture](ARCHITECTURE.md)

## GitHub Milestones

The [M6–M11 GitHub Milestone plan](GITHUB-MILESTONES.md) contains copy-ready descriptions, the single migration route from legacy rubric objects, and proposed issue assignments. Merging that document does not change GitHub metadata.

The [existing GitHub Milestones](https://github.com/simonvanieperen/BaryonOS/milestones) currently group work by criterion families. Their ten titles and descriptions were translated on 9 November 2026 without changing scope, state or due dates. Reorganizing these objects into the M6–M11 phases below remains the separate governance task in [#276](https://github.com/simonvanieperen/BaryonOS/issues/276).

GitHub Milestones should mirror the result-oriented phases **M6 through M11** while active work is underway. Individual rubric IDs such as `S09`, `U04`, or `N03` remain issues/criteria rather than separate milestones.

Historical M0–M5 work may be represented as closed milestones if that improves repository navigation, but it should not be recreated as a second status history.

## Evidence boundaries

- Host tests are not native-runtime evidence.
- QEMU evidence is not physical-hardware evidence.
- A successful build is not a successful runtime test.
- A screenshot is not evidence of durable storage or enforced permissions.
- Automated AI review is not an independent human security audit.
- A public repository and an applied software license do not make the system release-ready.

Historical planning detail removed from the active roadmap is preserved in [ROADMAP-through-2026-11-09.md](history/ROADMAP-through-2026-11-09.md).

# Progress measurement — from 0% to an independent daily-use OS

The score is calculated from the fixed criteria register in [progress.json](progress.json). This document defines the meaning of the score, acceptance rules, and the current criteria; task-by-task status updates do not belong here.

Historical progress updates through 9 November 2026 are preserved in [PROGRESS-updates-through-2026-11-09.md](history/PROGRESS-updates-through-2026-11-09.md). Current execution context is recorded in [PROJECT-STATE.md](PROJECT-STATE.md).

## Meaning of the percentage

The denominator is a fixed set of **100 weighted acceptance points**. A point is awarded only when the corresponding criterion is fully accepted with the required evidence.

The percentage is **not** elapsed development time, a security score, a probability of release success, or a claim of parity with Windows, macOS, Linux, or another operating system.

**0%** means that none of the fixed criteria are accepted. **100%** means that all criteria required for a maintainable independent daily-use OS on documented supported hardware are accepted.

## Evidence and acceptance

Evidence levels remain separate. Host tests and prototypes can support a design or implementation claim, but they do not replace a required native QEMU run, physical-hardware test, user study, or independent review.

A criterion is either **Accepted** or **Open**. Partial implementation does not receive partial points. A regression can return an accepted criterion to Open.

The machine-readable source of truth is [progress.json](progress.json). `tools/progress.py --check` validates the score, evidence references, and generated README/rubric/SVG views; it does not determine whether the evidence itself is technically sufficient.

## Fixed criteria register

<!-- OS-RUBRIC:BEGIN -->
**Current score: 41/100.**

### K — Independent kernel and process isolation (13/15)

| Criterion | Points | Status | Acceptance and evidence |
|---|---:|---|---|
| **K01 — Independent UEFI boot** | 2 | Accepted | The custom loader transfers control to the custom kernel; normal boot, missing/corrupt kernel, and timeout cases are checked natively. [evidence 1](../docs/verification/regression-m6b/FINAL-ACCEPTANCE.md) |
| **K02 — Independent memory management and fault boundaries** | 2 | Accepted | Custom page tables and frame management, W^X, guard stacks, and invalid mappings are checked in the fixed QEMU profile. [evidence 1](../docs/verification/m2-m4/ACCEPTANCE.md) · [evidence 2](../docs/verification/regression-m6b/FINAL-ACCEPTANCE.md) |
| **K03 — Isolated preempted processes** | 2 | Accepted | Ring-3 processes have separate address spaces; an application fault does not stop the other test processes. [evidence 1](../docs/verification/m2-m4/ACCEPTANCE.md) · [evidence 2](../docs/verification/regression-m6b/FINAL-ACCEPTANCE.md) |
| **K04 — Capabilities and bounded IPC** | 3 | Accepted | Rights reduction, generations, revocation, user-copy handling, and bounded Message320 IPC have native negative and lifecycle tests. [evidence 1](../docs/verification/m2-m4/ACCEPTANCE.md) · [evidence 2](../docs/verification/regression-m6b/FINAL-ACCEPTANCE.md) |
| **K05 — General process and service management** | 2 | Accepted | Non-fixture applications can start and stop through bounded APIs and services can restart; dependencies and resources are released correctly. [evidence 1](../docs/verification/K05-ACCEPTANCE.md) · [evidence 2](../docs/verification/k05-startup-slots/gate/supervisor-gate.json) · [evidence 3](../docs/verification/k05-startup-slots/integration.json) |
| **K06 — Multiple processor cores** | 2 | Open | SMP scheduling, synchronization, interrupts, and address-space switching work under concurrent load on the supported profile. [evidence 1](../docs/verification/k06-smp-sched/README.md) |
| **K07 — Resource management under pressure** | 2 | Accepted | Memory, CPU, IPC, and handle exhaustion remain bounded per application; the usable system recovers without kernel failure. [evidence 1](../docs/verification/K07-ACCEPTANCE.md) |

### S — Storage and data recovery (13/15)

| Criterion | Points | Status | Acceptance and evidence |
|---|---:|---|---|
| **S01 — Native read, write, and flush** | 1 | Accepted | Service-bound READ/WRITE/FLUSH operations use real virtio I/O; success is reported only after controlled completion. [evidence 1](../docs/verification/m6a-linux/README.md) · [evidence 2](../docs/verification/m6b/README.md) |
| **S02 — Journal and cold recovery** | 1 | Accepted | A bounded object journal survives native interruptions, revision checks, full storage, and cold recovery with an independent disk oracle. [evidence 1](../docs/verification/m6b/README.md) |
| **S03 — Names and domain-bound catalog** | 1 | Accepted | Native GET/PUT/DELETE/RENAME/LIST preserve names, contents, and domain boundaries, including interrupted renames. [evidence 1](../docs/verification/M6C-ACCEPTANCE.md) |
| **S04 — Space reuse and scalable volumes** | 3 | Accepted | Storage safely reuses deleted space and continues beyond the former 32-record limit; crash-consistent compaction and quotas are tested. [evidence 1](../docs/verification/M6E-ACCEPTANCE.md) · [evidence 2](../docs/verification/m6e/volume-gate.json) |
| **S05 — Documents of useful size** | 3 | Accepted | Large documents and streams exceed the former 128-byte probe limit; bounded buffers, full volumes, and partial writes are tested. [evidence 1](../docs/verification/M6E-ACCEPTANCE.md) · [evidence 2](../docs/verification/m6e/volume-gate.json) |
| **S06 — Collections, metadata, and edit history** | 2 | Accepted | Native object collections, persistent metadata, and recoverable revisions support daily Material tasks with consistent transactions. [evidence 1](../docs/verification/S06-ACCEPTANCE.md) · [evidence 2](../docs/verification/s06/volume-gate.json) |
| **S07 — Uncertain I/O without kernel panic** | 1 | Accepted | The current M6c catalog service returns controlled Unknown results for native WRITE/FLUSH failures, closes further access, and does not replay uncertain writes; there is no false ACK or kernel panic and other processes remain alive. Fault injection, cold recovery, disk-oracle checks, and normal M6c/M6b regressions are stored. This does not extend the historical M6b observer with a new error-handling claim. [evidence 1](../docs/verification/M6D-ACCEPTANCE.md) · [evidence 2](../docs/verification/m6d/capture.json) · [evidence 3](../docs/verification/m6d-m6b-regression/capture.json) · [evidence 4](../docs/verification/m6d-review/m6d-final-native-audit.json) |
| **S08 — General storage recovery after failures** | 1 | Accepted | A usable system remains responsive during disk failure, timeouts, corruption, and resource exhaustion; explicit recovery choices prevent silent data overwrite. [evidence 1](../docs/verification/S08-ACCEPTANCE.md) |
| **S09 — Backup, recovery, and data export** | 2 | Open | A user can create, verify, restore, and export independent recovery copies in documented formats; loss and conflict scenarios are tested. |

### H — Hardware, drivers, and power (1/12)

| Criterion | Points | Status | Acceptance and evidence |
|---|---:|---|---|
| **H01 — First independent device path** | 1 | Accepted | Custom virtio block transport, bounded DMA/MMIO access, reset behavior, and driver/service crashes are tested natively in QEMU. [evidence 1](../docs/verification/m5/ACCEPTANCE.md) · [evidence 2](../docs/verification/m5/review-repairs.json) |
| **H02 — Keyboard, mouse, and USB** | 3 | Open | Common input and USB devices work on the documented reference profile; hotplug and invalid device responses remain bounded. |
| **H03 — Graphics output and audio** | 3 | Open | Native graphics output, required GPU functionality, and audio meet documented usability and latency budgets on physical reference hardware. |
| **H04 — Network hardware and connections** | 2 | Open | Ethernet, Wi-Fi, and the agreed Bluetooth functions work on supported hardware, including disconnect, resume, and driver-failure paths. |
| **H05 — Power management and sleep/wake** | 2 | Open | ACPI/power management, sleep/wake, shutdown, and thermal/battery functions are measured and recoverable on supported hardware. |
| **H06 — Physical support matrix** | 1 | Open | Several predefined representative PC configurations pass the documented hardware matrix; DMA/IOMMU boundaries and limitations are explicit. |

### U — Native Material and accessibility (4/15)

| Criterion | Points | Status | Acceptance and evidence |
|---|---:|---|---|
| **U01 — Native compositor and session** | 4 | Accepted | The custom kernel runs an interactive graphical session with render and input chains, window/surface isolation, and controlled compositor recovery. [evidence 1](../docs/verification/mature-session-20261102/U01-ACCEPTANCE.md) · [evidence 2](../docs/verification/mature-session-20261102/U01-ACCEPTANCE.json) · [evidence 3](../docs/verification/mature-session-20261102/window-337-ci-verification.json) |
| **U02 — Responsive daily interaction** | 2 | Open | Keyboard, pointer, focus, clipboard, and application switching are usable natively and meet measured interaction budgets. [evidence 1](../docs/implementation/GUI-IDLE-WAIT.md) · [evidence 2](../docs/verification/gui-idle-20261102/windows-ab/README.md) · [evidence 3](../docs/verification/gui-idle-20261102/central-review.json) · [evidence 4](../docs/verification/gui-idle-20261102/INTEGRATION.md) · [evidence 5](../docs/implementation/NATIVE-CLIPBOARD.md) · [evidence 6](../docs/verification/native-clipboard-20261102/README.md) |
| **U03 — Material from open to save** | 3 | Open | Users open, edit, rename, and organize real native documents; permissions, history, and location remain understandable and correct. |
| **U04 — Native accessibility** | 3 | Open | Keyboard-only use, screen reader support, scaling/zoom, contrast, and reduced motion work in the native chain; user task tests are recorded. |
| **U05 — Settings, search, and recovery** | 2 | Open | Native system settings, local search, account/session management, and understandable error/recovery paths work consistently. |
| **U06 — Polish validated with users** | 1 | Open | Representative users complete agreed core tasks; friction, errors, and visual inconsistencies are measured and resolved within product budgets. |

### N — Internet and network services (3/10)

| Criterion | Points | Status | Acceptance and evidence |
|---|---:|---|---|
| **N01 — Native packet and transport stack** | 3 | Accepted | The operating system's network service processes real packets and bounded IP/TCP/UDP connections; malformed input, backpressure, and shutdown are tested. [evidence 1](../docs/verification/mature-session-20261102/README.md) · [evidence 2](../docs/verification/mature-session-20261102/n01-integrated.json) · [evidence 3](../docs/verification/n01-native-service-20261102/host-verification.json) |
| **N02 — Configuration and everyday internet** | 3 | Open | Address configuration, name resolution, routes, network transitions, and required internet protocols work on real authorized connections. |
| **N03 — Native authenticated transport** | 2 | Open | Maintained cryptography/TLS with the documented hybrid PQC choice works natively, including peer verification, negative handshakes, and no silent insecure fallback. |
| **N04 — Enforceable network permissions** | 2 | Open | Process and destination permissions, revocation, and helper routes are enforced natively and observed in traffic; no unintended listening ports exist. |

### B — Security, privacy, and updates (4/12)

| Criterion | Points | Status | Acceptance and evidence |
|---|---:|---|---|
| **B01 — Entropy and key management** | 2 | Accepted | Native cryptographic entropy, key lifecycle, protection, and recovery use reviewed standards and tested failure paths. [evidence 1](../docs/verification/B01-ACCEPTANCE.md) · [evidence 2](../docs/verification/native-recovery-hardening/b01/keyring-gate.json) · [evidence 3](../docs/verification/native-recovery-hardening/integration.json) |
| **B02 — Authenticated personal storage** | 2 | Open | Content and sensitive metadata are protected natively; tampering, replay/rollback within the threat model, and key loss are tested. |
| **B03 — Signed updates and rollback** | 2 | Accepted | Authentic updates, interruption, downgrade attacks, and recovery to a valid previous version are tested end to end. [evidence 1](../docs/verification/B03-ACCEPTANCE.md) · [evidence 2](../docs/verification/b03/update-gate.json) |
| **B04 — Trusted boot and distribution chain** | 2 | Open | Image/boot verification, key rotation, and dependency/build provenance are bounded for the release profile and independently reviewed. |
| **B05 — Targeted independent security review** | 2 | Open | External experts examine kernel, service, DMA, parser, and capability boundaries; blocking findings are resolved and selected invariants are machine-checked where appropriate. |
| **B06 — Native privacy contract** | 2 | Open | Applicable LP tests pass on the independent OS, including no telemetry/cloud fallback, local AI, and separated app/download permissions; remaining risks are published. |

### A — Applications, SDK, and installation (2/8)

| Criterion | Points | Status | Acceptance and evidence |
|---|---:|---|---|
| **A01 — Publicly documented application API and SDK** | 2 | Accepted | A reproducible SDK builds ordinary applications for the independent OS; API/ABI versions, sandbox behavior, and errors are documented and tested. [evidence 1](../docs/verification/A01-ACCEPTANCE.md) · [evidence 2](../sdk/README.md) |
| **A02 — Install and update applications** | 2 | Open | Packages from multiple sources can be installed, removed, and updated with provenance checks; installation does not grant broad capabilities. |
| **A03 — Application permissions and tool integration** | 2 | Open | Native applications use Material objects through visible, revocable task permissions; extensions and helper processes cannot bypass those boundaries. |
| **A04 — Useful everyday applications** | 2 | Open | At minimum, internet use, document editing, media, and file exchange work through suitable native applications or controlled ports; realistic user tasks are tested. |

### P — Personal NAS and multiple devices (0/5)

| Criterion | Points | Status | Acceptance and evidence |
|---|---:|---|---|
| **P01 — Native end-to-end encrypted NAS storage** | 2 | Open | Client and NAS roles on the independent OS pair with verified identity; names and content are unreadable to the NAS administrator, including failure and attack paths. |
| **P02 — Automatic synchronization and conflicts** | 1 | Open | Previously authorized peers resume automatically; offline edits, conflicts, and LAN/internet transitions preserve permissions and do not silently lose data. |
| **P03 — Multiple devices and key lifecycle** | 1 | Open | Devices can be securely added and revoked; key rotation, concurrent writers, and recovery work without vendor key escrow. |
| **P04 — Offer, revoke, and remove storage** | 1 | Open | The native NAS administrator manages quotas and revokes guest authority/removes local copies without guest consent or decryption keys; old sessions do not revive. |

### L — Local AI and quantum runtime (1/3)

| Criterion | Points | Status | Acceptance and evidence |
|---|---:|---|---|
| **L01 — Useful local system AI** | 2 | Open | At least the agreed core AI tasks run on the local device with bounded context, separate model downloads, and tested network denial; limited resources never cause silent cloud fallback. |
| **L02 — Local quantum execution on the independent OS** | 1 | Accepted | The local simulator/job interface runs natively with quotas, cancellation, and reference circuits; host simulation and physical QPU support remain separately labeled. [evidence 1](../docs/verification/L02-ACCEPTANCE.md) |

### R — Daily use and maintainable release (0/5)

| Criterion | Points | Status | Acceptance and evidence |
|---|---:|---|---|
| **R01 — Installation and accessible recovery** | 1 | Open | A user can install, upgrade, and independently recover the OS on supported hardware using controlled preserve/wipe options; a practical test passes. |
| **R02 — Long-term stability and performance budgets** | 2 | Open | Predefined duration, workload, energy, memory, latency, and data-integrity budgets are met on physical hardware with no known blocking usability defects. |
| **R03 — Reproducible release and maintenance** | 2 | Open | Independent reproduction, licenses/SBOM, release and patch policy, responsible reporting channel, support, and applicable accessibility/privacy obligations are concretely arranged. |

<!-- OS-RUBRIC:END -->

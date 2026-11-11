# Build decisions — Codex can now implement
Revision 5, 26 October 2026. The first investigations have been carried out and choices have been recorded. This is the short entry point for the next development chat. Use [FIRST-BATCH.md](FIRST-BATCH.md) as the execution assignment.

## What has been decided?

| Track | Chosen build basis | Exact contract |
|---|---|---|
| Boot | Own Rust kernel plus own small UEFI loader; Rust 1.90.0, r-efi 7.1.0, x86_64/QEMU TCG | [BOOT-AND-TOOLCHAIN.md](BOOT-AND-TOOLCHAIN.md): toolchain, ELF, BootInfo, entry, run command and error tests |
| Core security | Small capability-based core; first one CPU and isolated user processes, bounded IPC | [KERNEL-AND-SECURITY-CONTRACT.md](KERNEL-AND-SECURITY-CONTRACT.md): memory, syscalls, rights, quotas and negative tests |
| Product | **Materiaal**: your documents/collections are central, apps are interchangeable tools with bounded access | [PRODUCT-SPEC.md](PRODUCT-SPEC.md): comparison of three concepts, seven screens, states, design and accessibility |
| Quantum | Local Rust statevector simulator, qsim-v0, at most 12 qubits; core separate from host adapter | [QUANTUM-SPEC.md](QUANTUM-SPEC.md): gates, JSON, calculation rules, limits, fixtures and 17 tests; in addition PQC choices |
| Europe and publication | Concrete privacy, accessibility, export, supply-chain and recovery conditions | [EUROPE-AND-RELEASE.md](EUROPE-AND-RELEASE.md): applicability matrix and EU-01 to EU-08 |

This is a **chosen and investigated basis**, not an already tested OS implementation. The new OS does not take over an existing kernel. The development host, QEMU, UEFI standards and supporting libraries are tools. A UI prototype in a browser and a quantum trial on the development host are separate aids; their eventual OS port remains a real follow-up step.

## The first action

Open the project root and run:

```bash
python3 tools/preflight.py
```

This script exists and only takes inventory. Existing build/run files are checked first; if they are missing, the main agent creates these shared root/tools files for B01 according to the boot specification. Then start directly with B01, P02 and Q01. The main agent owns shared root configuration; builders have exclusive paths from FIRST-BATCH.md. No unbounded new architecture exploration.

Reference profile: QEMU 10.0.14, machine pc-q35-10.0, CPU qemu64, TCG, one CPU and 256 MiB. The boot specification document describes the practical route with available compatible QEMU/OVMF packages and the exact reference profile separately. Log deviations and real firmware hashes. Missing hardware acceleration is not a blocker for the TCG trial.

## Precedence and evidence

Explicit user requirements and [../LOCAL-PRIVACY-CONTRACT.md](../LOCAL-PRIVACY-CONTRACT.md) remain leading: internet with separate rights, E2EE storage on an explicitly paired personal NAS, optionally opaque client-side encrypted objects at an explicitly chosen external provider, remote access only with appropriate peer/provider rights, admin deletion right without content key, AI on the device and no mandatory vendor cloud or OS telemetry. A provider is not a silent fallback or key holder. The Security Floor remains active when the user deliberately chooses less privacy; privacy explanations may be turned off without widening capabilities. See also [ENCRYPTED-OBJECT-STORAGE.md](ENCRYPTED-OBJECT-STORAGE.md). The implementation specifications make older research proposals concrete; they do not revoke any privacy or product ambition.

All sources, chosen boundaries and still unproven properties are listed per specification. [VERIFICATION.md](VERIFICATION.md) records what was actually checked in this research. Reopen a decision only on a concrete technical contradiction, a demonstrated error or a new user instruction. Repair it in a targeted way, record it and continue building.

## Still to be realised first

- The loader/kernel and real boot tests; the presence of this documentation is not boot evidence.
- The interactive Materiaal prototype and first usability/accessibility trials.
- The Rust simulator and CLI tests; the analytical fixtures have been checked.
- After that the next small kernel, native product and quantum hardware steps from [../ROADMAP.md](../ROADMAP.md).

PQC, running quantum programs and running the OS on a hybrid classical/QPU system are three different goals. The first simulator does not erase that further ambition, but does not prove it either. The same applies to broad device support: the first x86_64 boot is one test platform, not a limitation of the end goal.

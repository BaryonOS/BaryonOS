# Glossary

These short definitions are reading aids; the linked repository contracts determine their project-specific meaning.

| Term | Meaning and canonical context |
|---|---|
| Capability | Bounded authority over a resource, represented by a kernel-validated handle; [kernel contract](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md) |
| IPC | Interprocess communication: requests/messages crossing process boundaries; [kernel contract](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md) |
| Ring 3 | The x86 user privilege level used for isolated user processes; [kernel contract](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md) |
| UEFI | The firmware interface used by the project's loader; [boot contract](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/implementation/BOOT-AND-TOOLCHAIN.md) |
| OVMF | UEFI firmware used for virtual-machine experiments; [development guide](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/DEVELOPMENT.md) |
| ABI | Application binary interface: the calling and data-layout agreement between compiled components; [SDK runtime](https://github.com/simonvanieperen/BaryonOS/blob/main/sdk/runtime/README.md) |
| QEMU | Emulator/virtualizer used for scoped guest tests; [development guide](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/DEVELOPMENT.md) |
| Native | Execution under the project kernel, distinct from a host prototype; [test strategy](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/TEST-STRATEGY.md) |
| NAS | Network-attached storage; hosting data does not automatically confer decryption rights; [privacy contract](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/LOCAL-PRIVACY-CONTRACT.md) |
| Security Floor | Protections that privacy preferences must not disable; [privacy contract](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/LOCAL-PRIVACY-CONTRACT.md) |
| ISA | Instruction-set architecture, the machine instruction model targeted by compiled code; [roadmap](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/ROADMAP.md) |
| QPU | Quantum processing unit; simulator evidence is a separate category; [quantum research](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/QUANTUM-RESEARCH.md) |

---

[Home](https://github.com/simonvanieperen/BaryonOS/wiki/Home) · [Canonical project status](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROJECT-STATE.md)

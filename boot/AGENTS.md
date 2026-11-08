# Boot rules

Read the root `AGENTS.md` and `docs/ARCHITECTURE.md`. A UEFI loader alone does not constitute a working independent OS kernel.

Define the boot ABI, including memory map and ownership, accessible mappings, entry point, stack, CPU state, and error path. Verify against the UEFI specification and the correct `ExitBootServices` sequence; do not call firmware boot services after control has been transferred. UEFI and kernel calling conventions are not interchangeable by default.

Validate ELF/image bounds and arithmetic overflow. An ordinary unsigned emulator boot does not establish a secure boot chain. Do not modify real firmware or devices.

Test successful transfer and missing/corrupt kernel cases. Success markers must be emitted by the kernel after transfer. Document firmware and emulator versions.

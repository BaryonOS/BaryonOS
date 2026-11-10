# Boot rules
Read the root AGENTS.md and docs/ARCHITECTURE.md. A UEFI loader alone is not a functioning independent OS kernel.
Define the boot ABI: memory map and ownership, accessible mappings, entry point, stack, CPU state and failure path. Check the UEFI specification and the correct ExitBootServices sequence; do not use firmware boot services after handover. The UEFI calling convention and the kernel ABI are not implicitly interchangeable.
Validate ELF/image bounds and arithmetic overflows. Do not infer a secure boot chain from an ordinary unsigned emulator boot. Do not modify real firmware or devices.
Test both successful handover and a corrupt or missing kernel. The marker must be produced by kernel code after handover. Document firmware and emulator versions.

# Boot, ABI and memory engineering

Read relevant sources through [boot](../themas/05-boot.md),
[toolchains](../themas/04-toolchain.md), [x86](../themas/06-x86.md) and
[memory](../themas/08-geheugen.md). For non-x86 targets use
[the architecture route](../themas/23-andere-architecturen.md), not x86 assumptions.

## Boot contract

Record who transfers control, the exact entry ABI, CPU state, address interpretation,
firmware-services lifetime, stack/descriptor requirements and memory-map ownership.
Validate boot records with their documented sizes, alignment, flags and lifetime.
Keep serial/diagnostic output available before complex allocations or GUI services.

Inspect actual linker sections, entrypoint and load/virtual addresses. Check zeroed
storage, stack alignment, address relocation and the handoff into C or Rust. Do not
copy a 32-bit compiler/linker command into an x86-64 or UEFI build without justification.
Do not presume UEFI services remain usable after the project's handoff terminates them.

Acceptance: reproducible entry, identifiable boot milestones, correct failure output,
and a test that an incomplete or malformed boot record fails safely rather than being
followed blindly. Record both positive and negative outcomes.

## Physical memory

Separate usable firmware memory from reserved/device memory and from ranges already
owned by the kernel, modules, stacks, tables, framebuffer and allocator metadata.
Check arithmetic overflow, range overlaps, alignment and address-width constraints.
Treat physical and virtual addresses as different types/concepts in interfaces.

Define allocation ownership, exhaustion behavior, zeroing/scrubbing policy, free/reuse
rules and synchronization. Include tests for empty input, tiny memory, holes, boundary
ranges, allocation exhaustion, double-free protection and bookkeeping recovery.
Never validate an allocator only by a single successful allocation.

## Virtual memory

Record page sizes, target page-table format, address canonicality, permission policy,
cache attributes, ownership and invalidation protocol. Test that the kernel remains
mapped across any transition that relies on those mappings. Separate early identity
mappings from the eventual address-space design; document when they can be removed.

Determine user/supervisor, write/execute and guard-page behavior from the actual target.
Check aliases and sharing, page-table cleanup on failure, mapping replacement and
permission reduction. On SMP, design invalidation and reclamation as a protocol;
local success does not establish safety on every CPU.

Acceptance examples: map/read/write/unmap behavior, deliberate access faults,
permission-denied writes, invalid addresses, out-of-memory rollback and remapping
without stale access. Use emulator evidence for actual privilege faults, not host tests.

## Book adaptation checkpoints

The book's 32-bit segmentation, higher-half addresses, paging dimensions, GRUB Legacy
flow and example compiler flags are historical context. Check source layouts,
instruction operands, serial/VGA examples and descriptor-table pointers against the
applicable hardware/assembler documentation before reuse. Do not assert that all
examples are correct because their text was faithfully imported.

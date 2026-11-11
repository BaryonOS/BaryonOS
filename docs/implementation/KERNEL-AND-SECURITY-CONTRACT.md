# Executable kernel and security contract M2–M4

Date: 26 October 2026. Status: **chosen implementation basis, not yet implemented or tested in QEMU**. This document settles the architecture choices for the first isolation and capability trials. The builder carries them out; doing general research again is not a precondition. Revision requires a concrete counterexample test, incompatibility or measured problem, plus a decision with a migration path.

This is an own Rust microkernel. seL4 serves as a public knowledge source for capabilities, not as an underlying kernel, adopted ABI or security proof. The source describes capabilities as rights to kernel objects; the precise tables, limits and protocols below are **own project choices** [K3]. The seL4 proofs too have explicit assumptions and apply exclusively to the configurations described there [K4].

## 1. Fixed first configuration

| Component | Decision |
|---|---|
| Machine | The M1 profile: QEMU q35, TCG, x86_64, one CPU, 256 MiB RAM, no network or host disk |
| Kernel | Rust `no_std`, `panic=abort`, `x86_64-unknown-none`, `code-model=small`; minimal assembly |
| Boot compatibility | Existing M1 `BootInfo` v1.1, 128 bytes; physical pointers; kernel VA=PA=`0x02000000`, reserved range 32–48 MiB |
| Page size | Own four-level paging, exclusively 4 KiB pages; no huge pages, PCID, demand paging or userspace MMIO |
| Processes | At most 16 processes; exactly one thread per address space; no shared user memory |
| Executables | Own small static ELF64 test programs, System V calling convention, no host libc, TLS or dynamic linker |
| Kernel objects M4 | Only endpoints and capability nodes; no general object, frame or process creation by apps |
| Creation | A bounded, built-in test manifest creates processes, memory and endpoints; later a separate management service replaces this |
| API | Own ABI version 1, register-based syscalls via `int 0x80`; bounded, non-blocking IPC |

The Rust target documentation confirms that this target has no `std`/standard allocator, by default uses no vector or floating-point registers and by default chooses the `kernel` code model. Therefore `small` is an explicit override here for the low addresses [K2]. Kernel and first user programs use the same scalar code generation; extra target features are not permitted in this phase.

M2 copies and validates BootInfo and the complete UEFI memory map before reuse of any memory. Use the descriptor stride from BootInfo, not `size_of` of a self-devised UEFI struct. All additions, counts and address ranges are checked for overflow. Version, flags and reservations follow [BOOT-AND-TOOLCHAIN.md](BOOT-AND-TOOLCHAIN.md); do not change the boot ABI here.

## 2. Memory, ownership and exceptions

### Address space

All ranges are half-open: the end address is not included. Each address space has its own PML4 and PDPT. Under PML4[0], PDPT[0] points to shared supervisor tables for the area below 1 GiB; PDPT[1] to own user tables for 1–2 GiB. Never share a table with process-specific user entries between processes. The kernel mappings are identical and exclusively supervisor; user mappings differ per process.

| Virtual range | Use and access policy |
|---|---|
| `0x00000000–0x00001000` | Unmapped; null page |
| `0x00001000–0x40000000` | Supervisor-only identity area; map exclusively actually needed RAM pages and explicit boot reservations, never gaps or devices automatically |
| `0x02000000–0x03000000` | M1 kernel reservation within this supervisor area; map image pages according to ELF segment rights |
| `0x40000000–0x60000000` | User ELF segments; first text at `0x40000000`; only actually loaded segments present |
| `0x60000000–0x60010000` | First fixed user heap, 64 KiB, read/write, NX |
| `0x7ffee000–0x7ffef000` | Unmapped lower stack guard |
| `0x7ffef000–0x7ffff000` | User stack, 64 KiB, read/write, NX |
| `0x7ffff000–0x80000000` | Unmapped upper stack guard |
| Other | Unmapped for userspace |

A user ELF may only supply non-overlapping `PT_LOAD` segments within the ELF range. Reject writable+executable segments, unequal page offsets of file/VA, overflow, missing executable entry, segment overlap at page level and unknown mandatory features. Zero the entire allocated memory before copying; `p_memsz - p_filesz` remains zero. Start `RIP=e_entry`, `RSP=0x7ffff000-8`, with zero at the dummy return position and other general registers zero. `_start` does not return.

Intel describes that U/S permission applies across the paging levels and that CR0.WP also binds supervisor writes to write permissions; NX prevents execution on pages marked for it. Interrupt/exception gates and privileges form the other half of the boundary [K1, chapters 4–6]. Chosen configuration: NX mandatory, `EFER.NXE=1`, `CR0.WP=1`; user PTEs have U/S=1 at all involved levels. Kernel leaves stay U/S=0, even when they share a higher table with user pages.

Text is RO/X, rodata RO/NX, data/heap/stacks RW/NX. **No writable alias of an executable physical frame:** the supervisor identity alias of user text is also RO/NX. Frame classification and mapping changes check all aliases. An executable frame only returns to the allocator after all mappings have been removed and TLB invalidations completed. Initially no JIT or runtime permission switching by apps.

### Frame management

Each 4 KiB frame has exactly one owner/status: `Reserved`, `Free`, `Kernel`, `PageTable(asid)` or `User(pid, kind)`. No shared user frames in M3/M4. A fixed bitmap plus metadata array suffices; large/growing allocations are not needed at the syscall boundary.

1. Initially take only `EfiConventionalMemory` as a candidate for `Free`, without runtime, nonvolatile or special attributes; explicitly subtract all boot reservations, kernel, map buffer, BootInfo, loader and stack. Overlapping or contradictory descriptors are a boot error.
2. Build own GDT, TSS, IDT and page tables, including current code/stack/map, and switch to own CR3 before interrupts or user mode are enabled. Firmware handlers and services are not a recovery path.
3. M2–M4 do not reclaim `Loader*`, `BootServices*`, ACPI or runtime areas. That temporarily costs RAM and avoids an implicit lifetime contract; later reclamation gets separate tests.
4. Zero a frame before assignment to another process and before return to `Free`. Physical addresses never leave as unrestricted app rights.
5. CR4.PCIDE and PGE stay off. A process switch reloads CR3; every mapping change gets the correct local invalidation before reuse. No remote TLB protocol as long as there is one CPU.

### CPU context and fault policy

GDT selectors: kernel code `0x08`, kernel data `0x10`, user data `0x1b`, user code `0x23`, 64-bit TSS `0x28`. TSS.RSP0 points on every switch to an own 16 KiB kernel stack with unmapped guards; double fault and NMI have separate IST stacks. Save all general registers. DS/ES/FS/GS are initialised in a controlled way; FS/GS bases are zero; user setting of these and FSGSBASE are off. Debug registers are not shared.

First user mode supports no x87/MMX/SIMD register state: `CR0.EM=1`, `TS=1`, `MP=1`; `CR4.OSFXSR=0`, `OSXSAVE=0`. An attempt that uses this register state ends in a user fault; the handler does not enable the feature after all. Scalar instructions such as fences are not automatically excluded by this. This is a temporary execution profile choice, not a limitation of the product ambition. SIMD/AI comes later with full context saving and separate leak tests [K1, chapter 13].

Normal interrupt gates are DPL0; only `0x80` is DPL3. Use `iretq`, not `sysret`. On return: checked user selectors, user RIP in executable page, user RSP in permitted stack, reserved RFLAGS bits correct; preserve only arithmetic flags CF/PF/AF/ZF/SF/OF and set fixed bit1+IF. IOPL/NT/VM/DF/TF/AC are zero. The entry stub executes `cld` before Rust. NMI performs no scheduler, allocator or capability mutations.

A user fault (`#PF`, `#GP`, `#UD`, `#NM`, etc.) terminates that process, records bounded local metadata and lets other processes continue. A kernel fault or double fault stops in a controlled way; do not try to continue a possibly damaged kernel. Fault logs contain no page content or secrets. On process exit: mark `Dying`, stop scheduling, close own endpoints, clean up cap rights, switch to a safe address space/stack and only then release frames/tables. Never release the active stack or active CR3 tables.

## 3. Scheduler with demonstrable progress

Use the legacy PIC/PIT path of this QEMU profile: remap IRQs to `0x20/0x28`, mask everything except PIT IRQ0, PIT channel 0 periodic with divisor 11932 (approximately 100 Hz), no concurrent APIC timer route. Initialisation and EOI code remain in the small arch layer; physical hardware support does not follow from this.

One FIFO round-robin queue, equal priority, quantum one timer tick. States: `Ready`, `Running`, `Dying`, `Dead`. `yield` places the process at the back. Empty run queue: `sti; hlt` idle. Any context switch takes place at the end of an IRQ or syscall, after kernel mutations have completed. Kernel work during M3/M4 is not preemptive; interrupts are masked during short, bounded critical sections. No waiting for I/O, lock spin, recursive fault handling or unbounded allocation in a syscall.

**Fairness contract:** with at most 16 permanently runnable processes, every process reaches the CPU within 16 scheduler selections, provided the bounded kernel operations terminate. This is not a real-time guarantee in milliseconds. Measure maximum syscall duration and delayed ticks separately; do not explain away timer loss as a “fast kernel”. No user priorities or unbounded busy-wait within the kernel.

## 4. Syscall ABI v1 and safe copies

`RAX`=number; arguments in `RDI, RSI, RDX, R10, R8, R9`. On return: `RAX`=0 or negative error code, `RDX`=result or zero on error. Other general registers are preserved; RFLAGS follows the mask above. Unused arguments must be zero. All ABI fields are little-endian, fixed integer widths; no Rust `usize`, references, `bool` or enums in untrusted bytes.

| No. | Name / arguments | Result on success |
|---|---|---|
| `0x00` | `abi_info()` | RDX=`(1 << 32)` for major 1/minor 0 |
| `0x01` | `yield()` | RDX=0 |
| `0x02` | `exit(code)` | Does not return; code is local process metadata |
| `0x10` | `cap_close(handle)` | Closes local slot, also if object closed/node revoked |
| `0x11` | `cap_derive(handle, rights)` | New local handle with reduced rights |
| `0x12` | `cap_revoke(handle)` | Revocation of all descendants; own node remains valid |
| `0x20` | `ipc_send(endpoint, message_ptr, size)` | One message queued, RDX=0 |
| `0x21` | `ipc_recv(endpoint, message_ptr, size)` | One message received atomically, RDX=payload length |

M3 implements only `0x00–0x02`. M4 adds the rest. Unknown syscall gives `ENOSYS`. There is no userspace debug port, general kernel pointer, arbitrary physical mapping or global object lookup function.

**Copy rule:** first treat a user pointer as `u64`. Check canonical address, user range, `checked_add(len)`, maximum length, presence/ownership of every touched page and effective U/S and R/W rights. Output requires writable pages; a read-only input may be read-only. Only then, via a small documented unsafe routine, make a byte copy into a fully initialised kernel buffer. Do not construct a `&UserStruct`, borrowed user slice or unvalidated enum. Rust does not make misalignments, invalid values and aliasing violations valid, even within `unsafe` [K5].

The kernel reads input exactly once, validates the copy and never stores the original pointer in a queue. In this profile the same mapping and bytes cannot change during copying through a second user thread, shared frame, DMA or user unmap: those possibilities do not exist, and no process switch takes place within the copy. With future threads/SMP/shared memory this contract must first be replaced by pinned mappings and synchronisation; “pointer checked in advance” alone is then insufficient.

For IPC, `size` is exactly 320; otherwise `EMSGSIZE` before copying. On receive, first copy into a kernel buffer, validate all output pages, reserve all handles and then commit in one non-interruptible operation. A failed receive does not change the user buffer, queue or cap table. All unused output bytes are zero. The fixed 320-byte format prevents lengths that change during a second user read.

| Code | Meaning |
|---|---|
| `-1 EINVAL` | Unknown flags/rights, non-zero reserved, invalid combination |
| `-2 ENOSYS` | Syscall unknown or not in this milestone |
| `-3 EFAULT` | User range/page/effective rights invalid |
| `-4 EBADHANDLE` | Slot outside table, empty or generation mismatch |
| `-5 EACCESS` | Required right missing or delegation attempts to enlarge rights |
| `-6 ETYPE` | Wrong kernel object type |
| `-7 EQUOTA` | Process or global resource limit reached |
| `-8 EAGAIN` | Endpoint queue full on send or empty on receive |
| `-9 ECLOSED` | Endpoint has been closed/stopped by owner |
| `-10 EREVOKED` | Existing local slot refers to revoked node |
| `-11 EMSGSIZE` | Wrong ABI size, payload >256 or cap_count >2 |

Validation order: syscall/unused args → ABI size → endpoint handle/generation/node/object/right → user buffer → message fields → resources → commit. Return values leak no object data before the rights check. An error changes no rights or resource ownership.

## 5. Capability and IPC model

### Handles and rights

A handle is `u64 = (generation:u32 << 32) | slot:u32`. Every process table has 256 slots; slot 0 is always invalid. Generation starts at 1, increases on close and never becomes 0. A slot with maximum generation is permanently retired on close. Never wraparound/ABA. Lookup uses **only the cap table of the calling process context**, never a PID supplied by userspace.

A handle is not a cryptographic secret. A number from process B gives A no access to B's table. If that number happens to coincide with a valid own slot of A, A obtains exclusively the right that A already held. The authority lies in kernel metadata, not in hard-to-guess digits; quantum computers do not change that lookup contract.

Rights bits for `Endpoint`: `SEND=1`, `RECV=2`, `DERIVE=4`, `TRANSFER=8`, `REVOKE=16`; all other bits invalid. `derive` requires DERIVE, and `new_rights & !old_rights == 0`. Mask 0 is permitted but gives no operation right. RECV stays with the endpoint owner: transfer with RECV is refused; an own local derivation may keep RECV. SEND says nothing about authority to use files, network or devices.

Each cap slot refers to a kernel node with object ID/generation, rights, parent node ID/generation, issuer PID and status. Nodes form a tree with maximum depth 8, where a root has depth 0; at most 1024 nodes in total. Object and node IDs also use generation counters internally without wraparound. Parent nodes remain as long as children exist, even after their last local handle is closed. Refcounts include local slots, children and queue references; reclaim only at zero.

`cap_revoke` requires REVOKE and marks all descendants revoked, also elsewhere in other processes or still in a queue. Scan at most 1024 nodes with at most 8 parent steps per node, and at most 1024 queue places; no dependence on table order or unbounded tree recursion. Revoked slots remain recognisable until `cap_close`, but give no access. `cap_close` alone does not revoke descendants. Process exit revokes all nodes in its slots and the nodes/descendants issued by that process; issuer PID is retained for this. No lasting rights arise from a dead delegator in this first profile.

### Message format and transaction

This is a format description, not an already existing implementation:

```rust
#[repr(C)]
struct IpcCapV1 { handle: u64, rights: u64 }
#[repr(C)]
struct IpcMessageV1 {
    sender_pid: u64,  // offset 0; send moet 0 zijn; receive vult kernel in
    tag: u64,         // offset 8; vrij appgegeven, geen autorisatie
    len: u32,         // offset 16; 0..=256
    cap_count: u32,   // offset 20; 0..=2
    reserved: u64,    // offset 24; 0
    caps: [IpcCapV1; 2], // offset 32; 32 bytes
    data: [u8; 256],  // offset 64; totaal 320 bytes, alignment 8
}
```

Unused cap descriptors and bytes after `len` must be zero on send; this yields a canonical, checkable representation. `sender_pid` comes from the kernel context and is never proof of substantive authority for a service. PIDs are monotonic `u64` without reuse/wrap within the same boot.

Send makes a snapshot; validates endpoint SEND, all caps to be transferred, TRANSFER and rights reduction; then reserves all node and queue places. Transfer is a **copy via a new derived node**, not a move. The source handle remains valid. A failed send leaves nothing behind. At most 16 messages per endpoint, FIFO, no waiting kernel threads.

Receive requires RECV and the owner PID. Reserve all required local slots before delivery. Replace message handles with handles from the receiver's own table; report the actual delegated rights. If the receiver has no slots, return EQUOTA and leave the message in place. For a revoked cap in a not yet delivered message the **whole message is cancelled** and the reservation returned; never deliver partial payload/rights. Revocation cleans up such messages within the fixed global scan limits. Bytes already delivered cannot be retrieved.

An endpoint belongs to exactly one owner. Closing or revoking that owner's last active RECV handle, or its exit, closes the endpoint; pending messages are cancelled. All other valid cap handles to the object give ECLOSED until closed; an already revoked node keeps EREVOKED according to the validation order. Object metadata remains as a tombstone as long as references exist, so that reuse does not make another endpoint reachable.

### Fixed quotas

| Resource | Limit M3/M4 |
|---|---|
| Processes / threads | 16 / one per process |
| User frames per process | 2048 = 8 MiB, including image/heap/stack |
| User virtual mappings | At most 2048; no automatic extension |
| Cap slots | 256 per process, of which slot 0 is unusable |
| Cap nodes / tree depth | 1024 global / 8 |
| Endpoints | 64 global, at most 4 owned per process |
| Queue | 16 messages per endpoint; fixed 320-byte message struct plus bounded kernel metadata |
| IPC transfer | At most 2 caps per message |

Reserve tables/queue places in advance; no allocator based on a user length. A manifest that exceeds limits is refused before process start. Arithmetic quota overrun and real allocator exhaustion share the same controlled error path; never panic as normal resource handling. Close/exit returns capacity, except retired generation slots.

## 6. Acceptance cases the builder executes

Make pure host tests for interval arithmetic, frame bookkeeping, cap transitions and IPC transactions. In addition, make QEMU tests that execute the real privilege transitions. The following IDs are assignments, **not passed test results**. The kernel harness writes `KS-xx:PASS` only after the described state has been established; user programs cannot print a success marker directly. Keep the real log, commit, profile, tool versions, timeout and exit status according to the boot contract.

| ID | Executable setup | Required assertion |
|---|---|---|
| KS-01, M2 | Parse map with overlapping descriptor, overflow in pages×4096 and reserved frame in ConventionalMemory | First two refused; reservation never allocated; all successful allocations unique |
| KS-02, M2 | Write pattern into frame, free, allocate to another owner; repeat until quota | New user frame fully zero; no double owner; exhaustion controlled |
| KS-03, M2 | Walk all page tables plus physical alias index | No user path to kernel page; no writable alias of executable frame; NX on heap/stack |
| KS-04, M3 | A/B get the same user VA with different physical frames and different patterns; switch repeatedly | A sees only A, B only B; PTE frame sets disjoint; no stale TLB |
| KS-05, M3 | A reads/writes B's physical frame via the supervisor identity address and tries to write kernel text | A gets user fault; B's bytes unchanged; B keeps counting |
| KS-06, M3 | A jumps to stack/heap, hits stack guard, executes `cli`, x87/XMM/AVX state instructions in separate boots | Each forbidden case stops only A; kernel remains controllable; no FPU state accessible |
| KS-07, M3 | 16 workers run without yield; measure a private progress counter per worker from the test harness | Everyone selected in every round of 16 selections; no shared user frame as measurement shortcut |
| KS-08, M3 | A crashes/exits under load; check tables before/after and let B continue | No active-stack free; frames/caps reclaimed according to contract; B makes progress |
| KS-09, M4 | IPC pointer null, supervisor address, noncanonical, last page+overflow, cross-page with second page absent/RO | EFAULT or EMSGSIZE beforehand; no kernel crash, partial output or queue change |
| KS-10, M4 | Fuzz len=`257`, cap_count=`3`, size=`0/319/321/u64::MAX`, reserved bits and misaligned valid buffer | Recorded error codes; misalignment of byte buffer permitted in itself; no UB through struct cast |
| KS-11, M4 | Test hook changes original input length/payload after snapshot; try to request shared mapping/second thread/user unmap | Operation uses only snapshot; unavailable API ENOSYS; build invariant one thread remains true |
| KS-12, M4 | Copy B's numeric handle to A; vary slot/gen; close and reuse own slot | No access to B; stale handle EBADHANDLE; collision yields at most own existing authority |
| KS-13, M4 | DERIVE/SEND without required right; derive with extra bit; transfer without TRANSFER or with RECV | EACCESS/EINVAL according to order; no node, handle or message created |
| KS-14, M4 | A→B→C delegation; revoke at A before and after delivery; hold on to old handles | Not yet delivered: whole message cancelled; delivered: cap EREVOKED, already copied data remain |
| KS-15, M4 | Fill 16 queue places; fill receiver's 255 slots; force cap node limit/depth 8; simulate maximum generation in host model | EAGAIN/EQUOTA without partial transactions; capacity returns after close; generation wrap impossible |
| KS-16, M4 | Owner closes last RECV; sender dies with queued delegated caps; receiver crashes | ECLOSED or cancellation according to lifecycle; no orphaned rights, refcount underflow or reused-object access |

KS-11 tests the concrete single-thread assumption and snapshot semantics, **not real SMP race resistance**. The later introduction of concurrent threads requires a new test case in which unmap/rewrite actually happens in parallel. The same applies to future DMA.

## 7. What each milestone does and does not deliver

| Security control | M1 | M2 | M3 | M4 / follow-up |
|---|---|---|---|---|
| Own MMU/frame management | Missing; firmware paging inherited | Implement and prove KS-01–03 | Used for isolation | Retained |
| Separate user processes | Missing | Missing | KS-04–08 | Retained |
| Controlled syscall boundary | Missing | Missing | ABI info/yield/exit | Copy rules and IPC negative tests |
| Cap rights/delegation/revocation | Missing | Missing | Missing | KS-09–16 |
| CPU preemption and quotas | Missing | Only fixed kernel resources | Scheduler/user frames | Caps/queues bounded |
| W^X | No claim | Kernel mapping check | Also user frames/aliases | Retained |
| ASLR, SMEP/SMAP, exploit mitigations | Not present | Not present | Not present in this profile | Separate extension; fixed addresses remain visible |
| DMA/IOMMU, SMP, side-channel protection | Not present | Not present | Not present | Not proven by these tests |
| Secure Boot, signed updates, disk crypto/PQC | Not present | Not present | Not present | M6/M8 contracts and quantum research |
| NAS and local AI | Not present | Not present | Not present | Service integration after relevant drivers/network; privacy gates remain binding |

“Not present” means that an M1 boot or M4 test does not justify such a product claim. The chosen protection first targets direct CPU memory access and explicit authority. Speculative/timing channels, firmware compromise and physical attacks are not solved by this.

## 8. Connection to local private use

The capability layer is the enforcement mechanism on which the later services build. It does not itself decide whether an IP address is the own NAS. For that, create separate objects with concrete scope: chosen file, named NAS dataset, approved LAN context, separate internet right and a local AI task. A general endpoint to a powerful helper must not bypass these boundaries.

The NAS service gets rights exclusively to paired datasets and an authenticated channel to the selected NAS identity; LAN by default, internet separately explicitly permitted (privacy contract revision 3). The transport service sees only ciphertext; the client manages the content keys. A NAS admin capability may block and delete space without user consent, but grants no decryption or document read right. Revocation must stop running uploads and repeated old requests at the commit boundary. The inference service gets task context and CPU/GPU/NPU resources, **no network endpoint**. The model downloader gets model storage plus download right, no documents, prompts, NAS credentials or inference history. Also restrict delegation and helper routes: the absence of a socket right alone is insufficient if IPC still makes a powerful upload helper reachable.

Later the policy service must re-check the capability, object scope and valid authorisation per request; user content or model text grants no rights. Network loss, a missing model or revocation causes a local error, no cloud fallback. The LP-01–14 tests in [LOCAL-PRIVACY-CONTRACT.md](../LOCAL-PRIVACY-CONTRACT.md) remain separate acceptance requirements; M4 cannot yet prove them. PQC will later protect keys/protocols, but does not replace this isolation and authorisation.

## 9. Primary sources (APA 7)

Accessed 26 October 2026. The consulted Intel PDF is explicitly the 2016 edition; the paging/privilege basis used is not a claim that modern processors have no errata or additional security requirements. The numeric limits, ABI, layout and tests above are project decisions.

- **K1.** Intel Corporation. (2016, September). *Intel® 64 and IA-32 architectures software developer’s manual: Volume 3A, system programming guide, part 1* (Order No. 253668-060US; chapters 4–6 and 13). https://www.intel.sg/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.pdf
- **K2.** The Rust Project. (n.d.). *x86_64-unknown-none*. The rustc book. Accessed 26 October 2026, from https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html
- **K3.** seL4 Project. (n.d.). *Capabilities*. seL4 Documentation. Accessed 26 October 2026, from https://docs.sel4.systems/Tutorials/capabilities.html
- **K4.** seL4 Project. (n.d.). *What the proofs assume*. Accessed 26 October 2026, from https://sel4.systems/Verification/assumptions.html
- **K5.** The Rust Project. (n.d.). *Behavior considered undefined*. The Rust Reference. Accessed 26 October 2026, from https://doc.rust-lang.org/reference/behavior-considered-undefined.html

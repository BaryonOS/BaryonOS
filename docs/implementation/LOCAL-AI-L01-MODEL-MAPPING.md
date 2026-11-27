# L01 model mapping: read-only verified weights for the native AI process

**Status:** design accepted by the project owner on 27 November 2026 (answers to Q0–Q6 in
section 7; decision record `ADR-L01-ARTIFACT-MAP-20261127` in `docs/DECISIONS.md`). Nothing is
implemented, built or run by this document. **Baseline:** `origin/main` `af7415df` (drafted at `7c32e6c7`; no file
cited below changed in between except `docs/DECISIONS.md`, whose references were
re-anchored). **Owner of L01:** Specialist L; decisions marked as owner questions belong
to the project owner. Interface owners for the first slice: K (kernel paging, process
lifecycle, syscall numbers) and B (Security Floor review of the new right).
**Review:** drafted by a delegated agent; reviewed by Specialist L on 27 November 2026,
who re-read at `af7415df` the limits L1–L9, the paging walk, `Paging::new_user`,
`Paging::destroy`, `Paging::audit`, `audit_process`, every syscall path that translates
a user pointer, the capability-right pattern and the toolchain pin, and corrected
sections 2, 4.1, 4.2, 4.4, 6 and 7 accordingly.
**Evidence boundary:** every code statement below carries a `path:line` reference
read at the baseline. Byte counts derived from the pinned configuration are
arithmetic, marked *derived*; no memory, latency or throughput was measured.

## 1. Problem statement

### 1.1 The model

The pinned Qwen2.5-0.5B-Instruct `model.safetensors` is 988,097,824 bytes with
SHA-256 `fdf756fa…88fb7fe` (`docs/implementation/LOCAL-AI-L01-PROPOSAL.md:14-17`),
290 BF16 tensors with 494,032,768 elements, 24 layers, hidden 896, 2 KV heads,
head dimension 64, intermediate 4,864, vocabulary 151,936, tied embeddings
(`experiments/ai/reference-forward/README.md:77-82`). The engine reads BF16 in
place; an f32 copy (about 1.84 GiB) would not fit the 2 GiB VM
(`experiments/ai/reference-forward/README.md:65-68`).

*Derived* from that configuration: one decoder layer has 12 tensors and
14,912,384 elements (29,824,768 bytes, 28.44 MiB); the embedding/output head is
151,936 × 896 BF16 = 272,269,312 bytes (259.66 MiB); a single MLP matrix
(4,864 × 896) is 8,716,288 bytes (8.31 MiB). Tensor data is 988,065,536 bytes; the
remaining 32,288 bytes are the safetensors header region (the README lists a
32,280-byte header; the 8-byte difference is consistent with the safetensors
length prefix, not verified here). The file is 942.32 MiB = 241,235 pages of 4 KiB.
Generating one token is estimated to read nearly all weights
(`experiments/ai/reference-forward/README.md:100-104`, an estimate, not a measurement).

### 1.2 Current native AI job

The `native-ai-job` profile runs one capability-free ring-3 SSE2 job twice beside
an FPU canary (`kernel/src/m2/m3/ai_job.rs:1-6`; `experiments/ai/native-job/README.md:28`)
on a tiny two-layer fixture, not Qwen (`tools/run_ai_job.py:32-36`). Each process has
the same layout: ELF segments in `0x40000000..0x60000000`
(`crates/kernel-core/src/user.rs:100-101`), a report page at `DATA = 0x40010000`, a
64 KiB heap at `HEAP = 0x60000000` and a 64 KiB stack below `0x7ffff000`
(`kernel/src/m2/m3.rs:370-380`, `kernel/src/m2/m3.rs:1186-1195`). The profile excludes
`milestone-m4` (the capability engine) and `process-memory` (heap growth)
(`kernel/src/m2/m3.rs:76-99`; `kernel/Cargo.toml:185`). Acceptance is raw QEMU exit 33
(`tools/run_ai_job.py:184`) with a frame-recovery and FPU-context-wipe check
(`kernel/src/m2/m3/ai_job.rs:265-266`).

### 1.3 Hard limits (USER_QUOTA is only one of them)

| # | Limit | Value and enforcement |
|---|---|---|
| L1 | Per-process frame quota | `USER_QUOTA = 2048` pages, 8 MiB (`crates/kernel-core/src/lib.rs:5`); counted per PID in `allocate` (`crates/kernel-core/src/lib.rs:180-193`) and `allocate_many` (`crates/kernel-core/src/lib.rs:228-238`); ELF images at most `USER_QUOTA - 32` pages (`crates/kernel-core/src/user.rs:123`); heap ends at `HEAP + USER_QUOTA` pages (`kernel/src/m2/m3/heap.rs:13`, `kernel/src/m2/m3/heap.rs:87-92`); ADR-047 keeps 2048 frames including program and stack (`docs/DECISIONS.md`, ADR-047, line 5601 at the baseline) |
| L2 | Physical frame pool | `FRAME_COUNT = 65536`, 256 MiB (`kernel/src/m2.rs:34`); frames above it are never admitted (`crates/kernel-core/src/lib.rs:124-125`); QEMU runs accept only 128 or 256 MiB (`tools/run.py:106-107`); kernel scratch access asserts `pa` inside the pool (`kernel/src/m2/paging.rs:1102-1104`) |
| L3 | User virtual window | user leaves only in `0x40000000..0x80000000`, 1 GiB, shared with ELF, heap and stack (`kernel/src/m2/paging.rs:632`, `kernel/src/m2/paging.rs:644`) |
| L4 | Page tables | 4-KiB leaves only (`kernel/src/m2/paging.rs:1`); large-page bit rejected on walks (`kernel/src/m2/paging.rs:213`, `kernel/src/m2/paging.rs:658`); at most `MAX_TABLES = 256` tables per address space (`kernel/src/m2/paging.rs:14`). 241,235 leaves need 472 page tables plus one directory (*derived*) |
| L5 | Per-process mapping record | `MAPS = 2048` entries per slot in a static arena (`kernel/src/m2/m3.rs:417`, `kernel/src/m2/m3.rs:448`) |
| L6 | Alias and owner policy | a user alias requires an `Owner::User` frame (`crates/kernel-core/src/lib.rs:336-339`); every user frame gets a kernel identity alias (`kernel/src/m2/m3.rs:1304`), writable except for text frames (`kernel/src/m2/m3.rs:1174-1176`); the audit forbids two live processes sharing a frame (`kernel/src/m2/m3.rs:1358-1363`); no shared-memory API exists (`kernel/src/m2/m3/m4.rs:229`) |
| L7 | Private RW memory | `KvCache` stores f32 (`experiments/ai/reference-forward/src/lib.rs:340-342`): at the proposal's 1,024 tokens, 24 × 2 × 2 × 64 × 1,024 × 4 B = 24 MiB (*derived*), above L1 before any scratch; the proposal's ceilings are 128 MiB KV and 512 MiB scratch (`docs/implementation/LOCAL-AI-L01-PROPOSAL.md:54-55`) |
| L8 | Byte delivery | `BootInfo` has no module/ramdisk field (`crates/boot-protocol/src/stable.rs:15-35`); virtio-blk reads one 512-byte sector per request into DMA below `RAM_END = 256 MiB` (`drivers/virtio-blk/src/lib.rs:15`, `drivers/virtio-blk/src/lib.rs:510-518`); store reads are at most 4,096 bytes (`crates/artifact-store-core/src/lib.rs:8`) and IPC payloads 256 bytes (`experiments/ai/model-store-contract/README.md:44-49`) |
| L9 | TLB | the kernel loads its own CR3 on every entry (`kernel/src/m2/m3.rs:2172`) and the process CR3 on every return (`kernel/src/m2/m3.rs:1629`); CR4.PGE and CR4.PCIDE are cleared (`kernel/src/m2/arch.rs:404`; `kernel/src/m2/paging.rs:1041`; `docs/DECISIONS.md`, ADR-075 item 5, line 3517 at the baseline), so each entry flushes all user translations |

L1 alone blocks the weights, but L2–L5 block any design that simply maps
942 MiB of private 4-KiB pages, and L7 blocks the real engine even after the
weights are solved. L8 is open for every option.

## 2. Security invariants (stated before any interface)

The Security Floor (`AGENTS.md`, "Security Floor") is not negotiable. For this
feature it means:

- **I1 Isolation.** No process sees the weights unless it holds the capability
  in I5; no process gains any other process's private frames. The audit rule of
  `kernel/src/m2/m3.rs:1358-1363` stays true for private memory.
- **I2 Read-only.** Every user-visible weight translation is not writable. The
  hardware must enforce it at the leaf and at the shared intermediate entry
  (CR0.WP is set, `kernel/src/m2/arch.rs:393`). After sealing no writable alias
  exists anywhere, kernel aliases included (`Alias.writable == 0`,
  `crates/kernel-core/src/lib.rs:93-97`). The RW scratch window installs a
  writable leaf that alias counts do not see (`kernel/src/m2/paging.rs:1121`), so
  after sealing the kernel reads weights only through the RO/NX scratch reader
  (`kernel/src/m2/paging.rs:1129-1146`, today feature-gated); the RW window is
  allowed only while filling and for release zeroing.
- **I2a User pointers.** A syscall given a user pointer into the weight window
  fails without copying and without a kernel fault. Existing M4 copy plans accept
  only `Owner::User` frames (`kernel/src/m2/m3/m4.rs:192-201`); `Process::physical`
  would panic on such a page (`kernel/src/m2/m3.rs:506-514`) and must never be
  reached with it.
- **I3 Non-executable.** Every weight translation has NX (EFER.NXE is enabled,
  `kernel/src/m2/arch.rs:377`); weight frames never enter the executable bitmap.
- **I4 Integrity before use.** No `ARTIFACT_MAP` holder (section 4.3) gets a
  translation to weight bytes before the complete 988,097,824-byte stream matched
  the pinned SHA-256 (a verifier, if any, holds a distinct right), and the bytes
  cannot change afterwards (I2). This mirrors the verify-before-parse rule of
  `experiments/ai/model-store-contract/README.md:25-31`. The expected digest comes
  from the pin, never from the artifact or the requesting process.
- **I5 No ambient authority.** Mapping requires an explicit, kernel-validated,
  PID-bound capability; an address, artifact ID or digest grants nothing
  (`kernel/AGENTS.md:5`; `docs/implementation/ARTIFACT-STORE-CONTRACT.md:17-20`).
- **I6 No network.** The AI process holds no network or document capability; the
  weight capability carries none (`docs/LOCAL-PRIVACY-CONTRACT.md:102-105`).
- **I7 Bounded resources.** Weight frames come from a fixed, separately accounted
  artifact budget reserved atomically before use; exhaustion fails closed with an
  explicit local limitation (`docs/LOCAL-PRIVACY-CONTRACT.md:103`). Raising one
  bound may not silently raise another (USER_QUOTA, `MAX_TABLES`, `MAPS`).
- **I8 Teardown.** Process exit removes its weight translation before any table
  is freed; the private KV/scratch frames are zeroed by the existing release path
  (`kernel/src/m2/m3.rs:1785-1787`) and the FPU image is wiped
  (`kernel/src/m2/m3/ai_job.rs:266`). Weight frames are zeroed when the artifact
  itself is released, never while any translation remains.
- **I9 Single CPU.** All TLB arguments below assume one CPU with the CR3 flush of
  L9; SMP requires a separate shootdown design.

## 3. Options

### Option A: sealed artifact region, shared read-only subtree, capability grant

A kernel-owned artifact object holds the weight frames under a new owner class
outside every PID's quota (L1). It is filled once, sealed (all writable aliases
removed), verified, and then exposed through one prebuilt, immutable page-table
subtree: one page directory plus 472 page tables whose leaves are `USER|NX`
without `WRITE`. A holder of the capability gets that directory linked into
PDPT entry 2 of its own address space, i.e. VA `0x80000000..0xC0000000`. This
reuses the existing pattern in which every user space has its own PDPT whose entry 0
points at the kernel's page directory (`kernel/src/m2/paging.rs:689-703`; under
`native-smp` the kernel half is asserted to be PDPT entry 0 alone,
`kernel/src/m2/paging.rs:697-701`), and avoids L3 because
the 1 GiB window holds the whole 942.32 MiB file. The subtree is not in the
process's own table list, so `Paging::destroy` never frees it
(`kernel/src/m2/paging.rs:814-828`).

### Option B: raise USER_QUOTA for a named AI profile

The AI process gets a private quota of at least 241,235 + ELF/stack/KV pages and
loads its own copy of the weights. This collides with L3 (the user window is
already partly used), L4 (472+ tables > 256), L5 (241k mapping records per slot
versus 2,048; the static arena is sized for `MAX = 16` slots,
`kernel/src/m2/m3.rs:383-387`), and L6 (each data frame gets a
writable kernel identity alias, so I2 would hold only for the user alias). Every
slot reuse copies and zeroes 942 MiB again. `Frames::allocate` scans the whole
pool per page (`crates/kernel-core/src/lib.rs:180-193`), so per-page allocation of
241k frames over a 2 GiB pool is quadratic. It also widens a global bound that
ADR-047 fixed for every process (`docs/DECISIONS.md`, ADR-047).

### Option C: stream through a bounded window with re-verification

The process keeps a window within its 8 MiB quota and the weights are streamed
through it per token. A window cannot hold even one MLP matrix (8.31 MiB,
*derived*), so streaming must split tensors by rows. Re-verification is not
possible against the pin: it is one whole-file SHA-256, so a per-chunk check needs
a derived chunk-digest table whose own trust root does not exist yet. Each token
re-reads nearly all 942 MiB (`experiments/ai/reference-forward/README.md:100-104`),
which through 4,096-byte store reads or 256-byte IPC payloads (L8) is about
241k store reads or 3.9 million IPC messages per token (*derived*).

### Comparison

| Criterion | A: sealed shared subtree | B: larger USER_QUOTA | C: streaming window |
|---|---|---|---|
| I1 isolation | link only for the holder; private audit unchanged | private copy; isolation unchanged | unchanged |
| I2/I3 RO+NX | hardware RO/NX at leaf and directory entry; no writable alias after seal | user leaf RO possible, kernel identity alias stays writable (L6) | window is writable private memory |
| I4 integrity | one full-file check before the first link | full-file check per process start | not checkable against the whole-file pin |
| I7 bounds | separate artifact budget; USER_QUOTA unchanged | global quota widened | within quota |
| 2 GiB budget (*derived*) | 942.32 MiB + 1.85 MiB tables once; 1,584.18 MiB including the 128 MiB KV and 512 MiB scratch ceilings, leaving 463.82 MiB for kernel, frame metadata, firmware and other processes | 942.32 MiB per live AI process; slot reuse needs a full reload | small memory, unbounded I/O per token |
| Page-table cost | 473 tables once; one PDPT entry per process | 473+ tables per process, above `MAX_TABLES` | small |
| TLB (L9) | a forward pass touches about 241k 4-KiB translations, refilled after every kernel entry; same in B | same as A | same per streamed page, plus I/O |
| Implementation size | new owner class, seal state, link/unlink, one capability right, audit extension | touches L1–L6 in shared code | chunk-digest format, trust root, streaming protocol |

A 2-MiB-leaf variant of A would reduce translations per pass from about 241k to
472, but it requires 2-MiB-aligned contiguous physical frames, which the
first-free allocator (`crates/kernel-core/src/lib.rs:197`) does not provide, and every walk currently rejects the
large-page bit (L4). It is a later, measured optimisation, not part of this design.

## 4. Recommendation and interface sketch

**Option A.** It keeps every existing per-process bound, enforces RO/NX in
hardware at two levels, verifies once and costs one PDPT entry per process. It
does not by itself unblock the real engine: L2 (frame pool) and L7 (private
KV/scratch) need their own owner decisions (section 7).

### 4.1 Kernel-core (`no_std`, host-testable)

- New owner class `Owner::Artifact(u16)`. `allocate_many` already applies the
  USER_QUOTA count only to `Owner::User` (`crates/kernel-core/src/lib.rs:228-238`);
  the artifact budget is a separate, fixed page count checked in the same
  all-or-nothing preflight.
- `Frames::map(user = true)` additionally accepts an `Artifact` frame only with
  `writable = false, executable = false` (today: `crates/kernel-core/src/lib.rs:336-339`).
  Each frame has exactly one user PTE (the shared leaf), recorded once when the
  subtree is built, so its alias is exactly `{writable 0, executable 0, total 1}`
  however many processes link the subtree; linking and unlinking never change alias
  counts. No existing audit reconciles alias counts with user tables:
  `Paging::audit` walks only the kernel root (`kernel/src/m2/paging.rs:874-890`,
  `kernel/src/m2/paging.rs:912-914`) and `audit_process` checks only W^X on aliases
  (`kernel/src/m2/m3.rs:1349-1350`). The artifact check above is therefore a new
  audit rule, and the subtree must never be linked into the kernel root, whose walk
  asserts that every table it reaches is its own (`kernel/src/m2/paging.rs:912`).
- Artifact state machine: `Empty → Filling → Sealed → Verified → Released`, plus
  the terminal failure state `Sealed → Rejected → Released`. `Sealed` requires
  `writable == 0` for every frame. Nothing links in `Sealed`: the verifier never
  maps the artifact; it receives the sealed bytes through a bounded kernel
  copy-out (section 4.4). `Sealed → Verified` requires that the verifier with the
  distinct `ARTIFACT_VERIFY` right has read every byte sequentially in the
  current generation and reports a 32-byte digest equal to the pinned descriptor.
  Any mismatch moves the artifact to `Rejected`, which never links. Links are
  allowed only in `Verified` (decision agreed with K, #534 issuecomment-7219369139).
  `Released` requires zero links and is reachable from every other state, so a
  failed fill, an unverified seal or a rejected artifact is aborted and its budget
  recovered; release frees every frame through the zeroing free path before reuse.
  Every illegal transition is an error.

### 4.2 Paging and process lifecycle

- Artifact frames are never identity-mapped into the kernel (contrast
  `kernel/src/m2/m3.rs:1304`); the kernel touches them only through the per-CPU
  scratch window: RW (`kernel/src/m2/paging.rs:1102-1104`) while `Filling` and for
  release zeroing, RO/NX (`kernel/src/m2/paging.rs:1129-1146`) otherwise (I2). This
  keeps the kernel identity map and its 256-table bound (L4) unchanged.
- The window `0x80000000..0xC0000000` is unused today: user and kernel leaves are
  asserted below `0x80000000` (`kernel/src/m2/paging.rs:632`,
  `kernel/src/m2/paging.rs:644`), and a grep of `kernel/src/m2` for VA constants at
  or above `0x80000000` found only these bound checks (the framebuffer window is
  `0x38000000`, `crates/boot-protocol/src/graphics.rs:9`).
- `link_artifact(root)`: writes PDPT entry 2 as `pd | PRESENT | USER | NX`
  without `WRITE`; refuses if the entry is non-zero. `unlink_artifact(root)`:
  clears it. Both run in kernel context after the entry-time CR3 load
  (`kernel/src/m2/m3.rs:2172`), so the next return CR3 load flushes stale user
  translations on one CPU (L9). Link and unlink act only on the calling process's
  own address space, so no cross-CPU shootdown is needed while PCID and PGE are
  off; adopting PCID/PGE later (K-2) must add explicit invalidation for this window.
- The general mapper never reaches the window: the `va < 0x80000000` asserts in
  `map_kind` and the private preflight stay. Every leaf under PDPT entry 2 is
  `PRESENT | USER | NX` without `WRITE` and points to an `Owner::Artifact` frame of
  a `Verified` artifact.
- Process cleanup unlinks before `Paging::destroy` and decrements the link count;
  under the artifact feature `Paging::destroy` asserts that PDPT entry 2 is zero,
  so a live link can never be dropped silently with the process PDPT frame.
  `retire_authority` (`kernel/src/m2/m3.rs:2871`) also revokes the capability.
  The exit and terminate paths unlink a remaining link themselves (PDPT entry 2
  cleared, link count decremented by one) before `Paging::destroy`; a native
  negative test proves that a process exiting while linked leaves a link count of
  0 and an empty entry 2 (K, #534 issuecomment-7220081163). The release order is:
  unlink everywhere (link count 0), then free the shared artifact tables, then
  free the artifact frames, a step that refuses frames still mapped
  (K, #534 issuecomment-7220170046).
- `audit_process` (`kernel/src/m2/m3.rs:1322`) is extended: PDPT entry 2 is zero,
  or exactly the shared artifact directory with the flags above while the process
  holds `ARTIFACT_MAP` and the artifact is `Verified`. The shared directory and its
  tables are not counted in the process's `tables` or `owned_table_count`, whose
  values existing M3 tests compare.
- User pointers (I2a). Today every syscall that translates a user pointer does so
  through `bounded_copy_plan`, which rejects any range ending above `0x80000000`
  before any lookup (`crates/kernel-core/src/user.rs:205-216`), and each lookup also
  requires an `Owner::User` frame: `m4.rs:194-198`, `m6.rs:373-375`,
  `m6b.rs:253-255`, `m6c.rs:259-261`, `update.rs:267-269`, `volume.rs:1204-1206`
  and `smp_unmap.rs:116-117` under `kernel/src/m2/m3/`. `Process::physical` is
  called only with kernel constants (`DATA`, `HEAP`, `STACK_END`). S1 keeps both
  barriers: the range bound stays `0x80000000` and every new user-pointer path uses
  these helpers.

### 4.3 Capability and syscall shape

Follow the manifest-only, non-delegable rights in `capability-core`
(`crates/capability-core/src/lib.rs:31-47`, checked as in
`crates/capability-core/src/lib.rs:940`): a new right `ARTIFACT_MAP` naming one
artifact object, issued by the manifest, without DERIVE or TRANSFER, bound to PID
and generation. A second right `ARTIFACT_VERIFY` has the same properties and
names the same object. Syscall numbers were assigned by K in the `0xA0` block
(#534 issuecomment-7219369139), and K assigned `0xA3` to `artifact_unmap`
(#534 issuecomment-7220081163); `0xA4–0xAF` stay reserved for later artifact
calls, and outside the `native-ai-artifact` profile every `0xA*` number returns
`-2` (ENOSYS) as contract §4 requires. The shape is:

```text
0xA0 artifact_map(handle)             -> Ok(base, length) | Err(Denied | Stale | NotVerified | Busy)
0xA1 artifact_verify(handle, digest)  -> Ok | Err(Denied | Stale | NotSealed | Incomplete | Mismatch)
0xA2 artifact_read(handle, buf, len)  -> Ok(copied, 0 at end) | Err(Denied | Stale | NotSealed | Fault)
0xA3 artifact_unmap(handle)           -> Ok | Err(Denied | Stale | NotMapped)
```

`artifact_map` needs `ARTIFACT_MAP`; `artifact_read` and `artifact_verify` need
`ARTIFACT_VERIFY`. No address, length, offset or digest of the artifact itself is
accepted from user space: the kernel returns base and exact length, keeps the read
cursor in the artifact state, and `artifact_read` copies at most one fixed chunk
(64 KiB) per call into the caller's private buffer. It needs a separate bounded
output plan with 17 page chunks: an unaligned 64 KiB destination can span 17
4-KiB pages. The plan checks the full destination range and every page for
caller ownership, user access, writable permission and valid frame alignment
before copying any byte. The existing two-chunk `CopyPlan` and its 320-byte IPC
and 512-byte sector limits remain unchanged; merely raising its byte maximum
would overrun its chunk array. A length above 64 KiB or a failed page check
returns an error without copying. This keeps each call bounded (contract §3)
and the user-space loop resumable.
`artifact_read` with a buffer in the artifact window returns `-3` (Fault) without
writing, because the copy plan requires writable, caller-owned user pages below
`0x80000000` (K, #534 issuecomment-7220170046).
`artifact_verify` is an O(1) 32-byte comparison and returns `Incomplete` until the
cursor equals the length. `Mismatch` moves the artifact to `Rejected`. Bytes after
`length` within the last page are zero. The 32-byte `digest` argument is the
verifier's result; the reference digest comes only from the manifest-pinned
descriptor.

### 4.4 Who verifies the hash

The kernel has no SHA-256 today (no `sha2` in `kernel/Cargo.toml`, no `Sha256`
or `sha2::` use under `kernel/src`), and `kernel/AGENTS.md:6` requires an ADR for a
cryptographic protocol or storage policy in privileged code. The ordering rule is
fixed: seal before the final hash pass, link to an `ARTIFACT_MAP` holder only
after a match. Placements: (V1) the kernel hashes through the RO/NX scratch
reader; (V2) a trusted ring-3 verifier holding a distinct, manifest-only
`ARTIFACT_VERIFY` right reads the sealed frames through the bounded kernel
copy-out `artifact_read` (section 4.3; the verifier never maps the artifact, so
nothing links before `Verified`), hashes them with the existing verifier (`experiments/ai/model-store-contract/README.md:25-31`) and
reports the digest, which the kernel compares byte-for-byte with the pinned
descriptor. Because the pinned digest is public, the kernel compare does not detect a
verifier that reports it without hashing: under V2 the verifier joins the trusted
computing base for I4; (V3) the AI process hashes its own mapping before first use, which
violates I4 as stated (a holder gets a translation before the match) and repeats
the hash on every start. **Lean: V2**, because it satisfies I4 without crypto in
privileged code, matching the precedent of ML-DSA-65 signature verification in the
ring-3 update service (`kernel/src/m2/m3.rs:372-373`, B03). It is still an owner
decision (Q1).

### 4.5 Byte delivery

Option A is independent of the source. Candidates: a ring-3 loader using the
scoped block broker (`kernel/src/m2/m3/volume.rs:1-6`) with 512-byte sector reads
(L8), or a new boot-protocol module field (boot ABI change). Filling uses the
same seal/verify rule. Throughput is unmeasured. Owner decision (Q3).

### 4.6 Out of scope

SMP and TLB shootdown, sleep/wake, AVX/FMA (only SSE2 is approved,
`docs/implementation/LOCAL-AI-L01-PROPOSAL.md:82-94`), 2-MiB pages, eviction or
paging of weights, model download and installation, more than one artifact, the
tokenizer tables, and the native forward pass itself.

## 5. Negative tests the implementation must have

Host (`kernel-core`, `capability-core`):

1. User alias of an `Artifact` frame with `writable` or `executable` is refused.
2. Seal is refused while any writable alias exists; any write alias after seal is refused.
3. Link in `Filling`, `Sealed`, `Rejected` or `Released` state is refused;
   `Verified` with a one-bit digest difference or before the read cursor reaches the
   length is unreachable; a mismatch moves the artifact to `Rejected`.
4. Artifact reservation beyond the budget fails with no owner change; USER_QUOTA
   counts are unchanged by artifact frames.
5. Release with a non-zero link count is refused; release zeroes every frame.
6. `ARTIFACT_MAP` and `ARTIFACT_VERIFY` cannot be derived, transferred, or used by
   another PID or after PID reuse (stale handle); neither right implies the other.

Native (QEMU, one CPU):

7. A user write to the mapping faults (#PF, write to read-only); the observer
   records the fault vector and address.
8. A jump into the mapping faults (#PF, instruction fetch on NX).
9. `artifact_map` without the right, with a revoked handle, or with a stale handle
   after slot reuse returns an error and leaves PDPT entry 2 zero.
10. A wrong expected digest leaves the artifact unverified and every map request refused.
11. A peer process without the right reads `0x80000000` and faults; its PDPT
    entry 2 is zero in the audit.
12. Reservation of budget + 1 pages fails closed; the free-frame count is unchanged.
13. After the holder exits, its PDPT entry 2 is zero, private KV/scratch frames
    are free and zeroed, and the frame-recovery count returns to baseline; after
    artifact release, every artifact frame is free and zero.
14. A synthetic artifact larger than USER_QUOTA (for example 3,072 pages, 12 MiB)
    maps successfully, proving it is not charged to the process quota.
15. A syscall that takes a user buffer, given a pointer into `0x80000000..`,
    returns an error; no bytes are copied and the kernel does not fault (I2a)
    (including `artifact_read` into the artifact window). An unaligned 64 KiB
    `artifact_read` destination spanning 17 pages is validated in full; a bad
    last page or a 64 KiB + 1 request is refused before any byte is copied.
16. Access after unlink faults (#PF); a second link is refused; `Paging::destroy`
    with a linked window asserts; `artifact_read` or `artifact_verify` without
    `ARTIFACT_VERIFY` is refused before any state change (K's list,
    #534 issuecomment-7219369139).

## 6. Acceptance commands, evidence limits and first slice

Existing gates (toolchain `1.100.0` per `rust-toolchain.toml:2` and root `AGENTS.md`):

```sh
cargo +1.100.0 test --locked -p kernel-core
cargo +1.100.0 test --locked -p capability-core
python3 -B tools/build.py --kernel-feature native-ai-job
python3 -B tools/run_ai_job.py          # raw QEMU exit 33, AIJOB:PASS
```

`native-ai-job` stays the unchanged regression gate. Evidence limits: QEMU TCG,
`qemu64`, one CPU, 256 MiB for every slice before Q2 is decided; no physical
hardware, SMP, sleep/wake, performance or numerical-quality claim; a synthetic
artifact proves the mapping mechanism only, not Qwen loading or inference.

**Smallest first slice (S1).** Host-tested `Owner::Artifact`, the RO/NX alias
rule, the state machine and the `ARTIFACT_MAP` right (tests 1–6); paging
link/unlink and the audit extension; one native fixture in the existing 256 MiB
profile with a 3,072-page synthetic artifact filled by the kernel with a
deterministic pattern, a holder, a peer without the right and one slot reuse
(tests 7–16). The digest reported in S1 is a build-time host digest, labelled as a
test stub until S2 adds the ring-3 verifier. The S1 pull request adds the `0xA0`–`0xA3`
rows to `docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md` §4, and K reviews the
paging and audit hunks as interface owner. Because `native-ai-job` excludes
`milestone-m4`, S1 needs its own profile; the name `native-ai-artifact` and a
runner `tools/run_ai_artifact.py` are proposals only (Q4).

**Later slices.** S2: the verifier chosen in Q1 and the byte source chosen in Q3,
still synthetic. S3: the frame-pool and VM change of Q2 plus the private-memory
bound of Q5; then the full pinned file, local evidence only, because the 988 MB
file is not committed and must be read through `NOS_QWEN_DIR` as in
`experiments/ai/reference-forward/README.md:115-117`.

## 7. Owner questions and ADR need

**Owner answers, 27 November 2026** (binding; recorded in `ADR-L01-ARTIFACT-MAP-20261127`):
Q0 yes, a focused ADR, for a bounded reusable *immutable* shared-memory mechanism without
generic writable shared memory. Q1 V2: a trusted ring-3 verifier; the kernel enforces rights,
identity, state and the digest comparison; frames are sealed before verification, the verified
frames are the mapped frames, the verdict binds identity, length, generation and digest, and
every deviation fails closed. Q2 a logically separate weight pool from kernel-validated
available memory, reusing the frame allocator, with separate accounting and a hard quota, no
hardcoded physical addresses, and a larger QEMU profile only for the AI tests. Q3 a ring-3
loader through the existing block service; no boot-protocol change; bounded loading without
unnecessary full copies. Q4 an own native profile; `native-ai-job` and the `milestone-m4`
profiles stay as they are; the minimum native coverage adds revocation and SMP/TLB correctness
where applicable to tests 7–15. Q5 a profile-bound, kernel-enforced private cap of at most
64 MiB for the authorised AI process, not reserved up front, `USER_QUOTA` unchanged; the
scratch, stack and image needs beyond the 24 MiB KV cache must be justified before S3. Q6 the
window is approved for phase 1 and stays a kernel-returned base, not a public ABI constant.

The questions as put to the owner:

- **Q0.** Does the owner agree that an ADR is required before S1 merges, since S1
  adds an owner class, the first shared user mapping class (none exists today,
  `kernel/src/m2/m3/m4.rs:229`), a capability right and a VA region?
- **Q1.** Where is the SHA-256 computed: kernel (V1, needs an ADR under
  `kernel/AGENTS.md:6`), a trusted ring-3 verifier with a distinct right and a
  kernel compare (V2, the lean; the verifier is then part of the trusted base for
  I4, because the compared digest is public), or the AI process itself (V3, which
  violates I4 as stated)?
- **Q2.** How does the 2 GiB budget reach the frame pool: grow `FRAME_COUNT` for
  the AI profile with frames above the current pool admitted only for artifacts
  (so ordinary allocations, identity aliases and the 256-table bound stay as
  they are), or a separate fixed artifact extent? Both also need a new QEMU
  memory profile beyond `tools/run.py:106-107`. Frame metadata grows with the
  pool (about 37 bytes per frame from the field types, about 18.5 MiB at 2 GiB;
  an estimate, `size_of` not checked). Two related constraints: the kernel half,
  including identity aliases, is PDPT entry 0 (first 1 GiB), so frames above 1 GiB
  can only be artifact frames reached through the scratch reader; and device setup
  asserts PCI BARs at or above `0x80000000` (for example
  `kernel/src/m2/m3/volume.rs:859`), which holds for a 2 GiB q35 guest only if QEMU
  keeps all RAM below the 32-bit PCI hole (not verified).
- **Q3.** Which byte source: a ring-3 loader through the block broker, or a
  boot-protocol module (boot ABI change under `boot/AGENTS.md`)?
- **Q4.** Does S1 run in a new milestone-m4-based profile, or should
  `native-ai-job` drop its `milestone-m4` exclusion (`kernel/src/m2/m3.rs:76-99`)?
- **Q5.** Which bounded private-memory allowance does the AI profile get for KV
  and scratch (24 MiB f32 KV at 1,024 tokens, *derived*; proposal ceilings 128 MiB
  and 512 MiB), given that USER_QUOTA is 8 MiB and `process-memory` is excluded?
- **Q6.** Is VA `0x80000000..0xC0000000` (PDPT entry 2) acceptable as the fixed
  artifact window, limiting one artifact to 1 GiB?

## 8. Not verified here

The safetensors header split (section 1.1), the frame-metadata size (Q2), QEMU's
physical layout at 2 GiB, all throughput and TLB costs, and the behaviour of any
proposed code. Graphify MCP was not used; the draft's references were read
directly from the worktree files at `7c32e6c7` and the review re-read the sections
listed in the status line at `af7415df`.

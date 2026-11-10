# TASK-USER-ALLOC — bounded allocator for user processes

Task contract, 30 October 2026, baseline `cda42f2`. Owner: `native_ui_apps`;
the lead agent owns SDK/kernel/app integration. Only `sdk/heap-allocator/**`
and this document. No global workspace change or fixed heap-growth ABI
assumption.

## Selected engine and source review

Reuse `linked_list_allocator = 0.10.6`, with default features disabled. The
latest non-yanked sparse-index version was published on 14 April 2026. The
actual crate tarball and index checksum were compared before inclusion:
`2b23ac50abb8261cb38c6e2a7192d3302e0836dac1628f6a93b82b4fad185897`.
The published manifests declare Rust ≥1.61, Apache-2.0/MIT and `build=false`.
Without defaults there are no transitive runtime dependencies; spin/nightly/
alloc_ref are disabled. The wrapper uses the existing first-fit/coalescing
engine, not a custom allocation algorithm.

Maintenance/security review: upstream 0.10.6 fixes a possible panic with a very
large Layout. The changelog lists the older OOB advisory GHSA-xg8p-34w2-j49j,
resolved since 0.10.2, and the padding fix in 0.10.4. This is a targeted source
review, not a complete audit or guarantee that all errors have been found.
The MIT notice is included with the package.

Sources: [upstream](https://github.com/rust-osdev/linked-list-allocator),
[crate index](https://index.crates.io/li/nk/linked_list_allocator),
[pinned tarball](https://static.crates.io/crates/linked_list_allocator/linked_list_allocator-0.10.6.crate),
[`Heap` API](https://docs.rs/linked_list_allocator/0.10.6/linked_list_allocator/struct.Heap.html).
Sources and actual version checked on 30 October 2026.

## Ownership and execution

- Only contiguous, permanently valid RW+NX memory assigned exclusively to this
  process may become an arena. BSS or confirmed growth of its own heap can
  provide that space; a pointer/number does not itself prove a kernel
  permission or NX.
- `Arena::checked` checks only representation, length, alignment and overflow
  without dereferencing. `initialize_raw` is unsafe: the caller guarantees
  exclusively valid `'static` storage. The safe initializer takes ownership
  of an exclusive `'static mut` byte buffer. A failed shape check writes nothing.
- Initialisation occurs at most once. The engine receives pre-normalised
  alignment and at least two machine words; unsuitable/undersized arenas and
  reinitialisation return errors before allocator state or previous work changes.
- No automatic growth, host malloc, virtual-memory backend or syscall fallback.
  The kernel agent is working on fallible growth of the process's own heap;
  v1 uses only an already confirmed arena. The development quota below 8 MiB
  is not a product limit; the library fixes no such capacity.
- The profile has one user thread per process, with pre-emption permitted.
  An atomic lock protects the engine/metrics; no syscalls, yields, callbacks
  or logging occur under that lock. Other address spaces use their own
  instance. Re-entrant allocator calls from user interrupts/signals are
  not permitted.
- First-fit allocation and deallocation cost O(number of free blocks). The
  fixed arena size bounds that list; this is neither constant time nor a
  measured real-time guarantee. Larger working sets require measurements
  on the profile.
- `GlobalAlloc` is unsafe under Rust's contract: deallocation uses exactly the
  previously returned pointer/Layout, exactly once. The wrapper does not
  promise detection of arbitrary pointers/double-free and does not test such UB.

## Errors and visibility

Fallible allocation distinguishes uninitialised state, zero/invalid requests
and OOM. Alignment, extent and Layout overflow are checked before calling the
engine. A conservative checked-arithmetic bound prevents a temporary search
address from wrapping in the engine at extremely high addresses/alignment.
Such requests return OOM before an engine change, even if a particular block
would fit. The ordinary `GlobalAlloc` API returns null on failure; infallible
Vec/Box calls can consequently terminate the process. User flows use
`try_allocate` and `Vec/String::try_reserve` for visible OOM without an
incorrect OS success status. Failed reallocation preserves the old valid
allocation according to Rust's contract.

Usage/high-water figures concern only the process's own arena: usable
capacity, engine used/free, payload usage, live allocations and totals/failures.
Engine usage counts the block rounded up to at least two machine words; the
difference from payload usage shows that overhead. Free-list headers reside
in free blocks and are part of `engine_free`, not additional external heap
space. Total free space does not guarantee a large contiguous block.
Historical counters visibly saturate and never wrap. No secrets are logged;
the deallocator does not automatically erase all plaintext or released pages.

Normal integration installs one `static HEAP: UserAllocator` as
`#[global_allocator]` and initialises it before the first `alloc` request.
Ordinary `alloc::Vec`, `alloc::boxed::Box` and `alloc::String` then operate
through that instance. `try_allocate(Layout)` returns a raw, uninitialised
allocation; the corresponding unsafe
`deallocate(pointer, exact_dezelfde_layout)` (the exact same layout) remains
the caller's responsibility. `usage()` copies only counters and returns no
reference to the lock or arena. Using a fallible API does not itself guarantee
secret erasure or a kernel permission.

Rust contract: [`GlobalAlloc`, Rust 1.90.0](https://github.com/rust-lang/rust/blob/1.90.0/library/core/src/alloc/global.rs).
No unwinding from allocator functions; `GlobalAlloc::alloc` receives a valid,
non-empty Layout. Only the safe wrapper explicitly offers a `ZeroSize` error;
the tests never call the unsafe global API with zero size. Rust's default
`alloc_zeroed` and `realloc` provide zero bytes and preservation of the old
allocation when growth fails, respectively.

## Acceptance and remaining native gate

Host tests: initialisation errors without writes, alignment/prefix/suffix
canaries, extents, small/large allocations, reuse/coalescing/fragmentation,
OOM with preservation, counters and actual alloc::Vec/Box/String over only
a dedicated static global test arena. No System/host fallback in that
collection test. Build the engine and wrapper as actual scalar code for
`x86_64-unknown-none` and record source/artifact hashes. This does not prove
native allocation or everyday browser/AI operation.

Native follow-up: root integrates an ordinary user app, grants demonstrably
owned RW+NX arenas, checks alloc/OOM/isolation and process cleanup in QEMU,
and connects any heap-growth descriptor only after separate ABI acceptance.
All full product criteria and the 8-GiB/32-GiB test profile remain in place.

## Executed evidence

[`sdk/heap-allocator/verification/README.md`](../../sdk/heap-allocator/verification/README.md)
contains reproducible commands, source/artifact binding and the executed
host/cross-compilation evidence. On this source, 15 targeted tests, the
separate Vec/Box/String executable, Clippy and fmt pass. The wrapper and pinned
engine were built as scalar code and their actual object code was checked.
QEMU, heap growth, process cleanup and isolation acceptance have not yet been
executed for this library; the full release and the relevant product criteria
remain open.

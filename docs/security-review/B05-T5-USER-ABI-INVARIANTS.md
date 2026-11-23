# B05-T5: bounded invariant checks for the ring-3 user ABI

Status: host-only bounded enumeration. It is not a proof, not a formal
verification and not an independent human audit. It does not accept B05.
Baseline: `origin/main` a91d5ade, toolchain `1.100.0` (from
`rust-toolchain.toml` and `ci.yml#host-quality`).

Test file: `crates/kernel-core/tests/bounded_user_abi_invariants.rs`, a
std-only integration test with no new dependency. It is listed under the
`syscalls-usercopy` boundary in `docs/security-review/scope.json`.

Entry points, all in `crates/kernel-core/src/user.rs` and all used by the live
kernel (`kernel/src/m2/m3.rs` and `kernel/src/m2/m3/` including `m4.rs`, the
`m6*` profiles, `supervisor.rs`, `update.rs`, `volume.rs` and
`window_session.rs`):

- `copy_plan` and `sector_input_plan`, which both delegate to the private
  `bounded_copy_plan` (maximum 320 and 512 bytes);
- `parse`, the user ELF parser. It is separate from `elf-loader-core::parse`, so
  B05-T4 does not cover it;
- `return_flags` and `valid_return_layout`, the saved ring-3 frame checks.

The oracles are written from the rules in that source file at a91d5ade. They
are checked against the inputs and the synthetic page-table model, not against
the function's own output.

## 1. Usercopy planning

A synthetic lookup model stands in for the kernel page-table walk. Every page
gets a distinct, page-aligned frame (`0x0008_0000_0000_0000 | page`), so a
swapped or reused frame is detected. The eleven models are `Good` and, for each
of missing page, foreign owner (`pid + 1`), read-only, non-user and unaligned
frame (`physical + 8`), an `*All` variant (every page broken) and a `*Second`
variant (only pages after the first page of the range are broken).

| Test | Enumeration | Cases |
| --- | --- | --- |
| `copy_plan_pointer_length_and_lookup_model_product` | 12 lengths {0, 1, 2, 255, 256, 319, 320, 321, 511, 512, 513, `u64::MAX`} × pointer set × output {false, true} × 11 models | 1,087,174 |
| `sector_input_plan_is_exactly_512_input_bytes` | pointer set for length 512 × 11 models | 90,288 |

The pointer set has these edges: 0, 1, 0x3fffffff, 0x40000000, 0x40000001, the
page edges around 0x40013000, 0x40013ec0/0x40013f00/0x40013f01 (end exactly at
or one past a page), 0x5fffffff, 0x60000000, 0x7ffff000, 0x7ffffec0,
0x7fffffff, 0x80000000, `1 << 63`, 0x0001000040000000 (non-canonical alias),
0xffff800000000000, `u64::MAX - 255`, `u64::MAX`, and, per length,
`0x80000000 - len ± 1`, `u64::MAX - len + 1` and `+ 2` (overflow edge) and
`0x40014000 - len` and `+ 1` (page edge). For lengths 1..=320 and for the
sector plan it also has every byte offset of page 0x40013000 and of the top
window page 0x7ffff000.

Checked invariants:

- The result is `Ok` exactly when the reference predicate holds:
  `1 <= len <= maximum`, `pointer + len` does not overflow,
  `pointer >= 0x40000000`, `pointer + len <= 0x80000000`, and every touched
  page is present, user, owned by the caller, writable when `output` is set and
  has a page-aligned frame.
- A range reject is `Error::Range` and calls the lookup zero times, so the
  whole range is checked before any page is looked up.
- A page reject is `Error::Permission` and stops at the first broken page:
  exactly one lookup for `*All` models and two for `*Second` models.
- Every lookup address lies in `[pointer, pointer + len)`, and successive
  lookups are on consecutive pages.
- Every accepted plan has `count` 1 or 2, equal to the number of touched pages.
  Each chunk lies within one page (`offset + length <= 4096`, `length > 0`).
  Its frame equals the model's frame for its page and its offset equals the
  page offset of its start. The chunks are contiguous and their lengths sum to
  `len`. Unused chunk slots stay zero.
- `sector_input_plan` never requires a writable page.

Observed outcome counts (local run, exact and deterministic):

| Test | Accepted | Range causes | Permission causes |
| --- | --- | --- | --- |
| `copy_plan` | 614,055 | zero length 572, too long 3,212, overflow 374, below window 396, above window 25,828 | missing 98,386, non-user 98,386, foreign 98,386, read-only output 49,193, unaligned 98,386 |
| `sector_input_plan` | 51,750 | overflow 44, below window 33, above window 5,665 | missing, non-user, foreign and unaligned 8,199 each |

Each test asserts its case total and that every listed cause occurs at least
once.

## 2. User ELF parsing

The base input is a 0x3010-byte ELF64 built in code: one `PT_GNU_STACK` (RW,
skipped) and three `PT_LOAD` headers, RX, R and RW, at 0x40000000, 0x40001000 and
0x40002000. Each has 16 file bytes; the RW segment has 0x2000 memory bytes.

| Test | Enumeration | Cases |
| --- | --- | --- |
| `parse_fixture_is_accepted` | the unmodified fixture (3 segments, 4 pages) | 1 |
| `parse_every_header_and_program_header_byte` | each byte 0..288 set to {0x00, 0x01, 0x7f, 0x80, 0xff, b^0x01}, skipping duplicates and the original | 1,235 |
| `parse_every_truncation_rejects` | every prefix length 0..0x3010 | 12,304 |
| `parse_segment_field_edge_pairs` | for each `PT_LOAD`, every pair of offset, vaddr, filesz, memsz and align over a 26-value edge set | 20,280 |
| `parse_header_entry_phoff_phnum_edges` | 8 entries × 11 e_phoff values × 10 e_phnum values {0, 1, 2, 3, 4, 5, 31, 32, 33, 0xffff} | 880 |
| `parse_segment_count_edges` | 1..=12 one-page `PT_LOAD` segments | 12 |

The edge set is 0, 1, 16, 0xfff, 0x1000, 0x1001, 0x2000, 0x3000, the file
length −16, length and length+1, 0x3fffffff, the window start, +0x1000 and +0x2000,
0x5ffff000, 0x5fffffff, 0x60000000, 2014, 2016 and 2017 pages (the quota edge),
0x0001000040000000, `1 << 63`, `u64::MAX - 4094` (page round-up overflow),
`u64::MAX - 15` and `u64::MAX`.

Checked invariants for every accepted image, against the input bytes:

- The ELF identification and header fields are the required values (ET_EXEC,
  x86-64, version 1, e_flags 0, 64-byte header, 56-byte program headers).
- `1 <= count <= MAX_SEGMENTS` (8). The table has 1..=32 headers and lies inside
  the input.
- The segments equal, in order, the input's `PT_LOAD` headers. Every other
  header is `PT_NULL`, `PT_NOTE`, `PT_PHDR` or a non-executable `PT_GNU_STACK`.
- Flags are R, RX or RW: never W+X.
- `memsz > 0`, `filesz <= memsz`, and `offset + filesz` does not overflow and
  is no greater than the input length.
- `[va, va + memsz)` lies inside `[0x40000000, 0x60000000)`.
- `align` is a power of two of at least one page, and
  `offset ≡ va (mod align)`.
- Page-rounded ranges are pairwise disjoint.
- `pages` equals the sum of page-rounded sizes and is at most
  `USER_QUOTA - 32` (2016).
- The entry equals e_entry and lies inside the file-backed bytes of an
  executable segment.
- Unused segment slots stay empty, and `parse` is deterministic.
- `parse` never panics within the enumerated inputs (each call runs under
  `catch_unwind` in the debug profile with overflow checks on).
- No truncation is accepted. Exactly 1..=8 segments are accepted and 9..=12 are
  rejected with `Range`.

Observed outcome counts:

| Test | Ok | Permission | Range | Overlap | Quota |
| --- | --- | --- | --- | --- | --- |
| Single byte | 449 | 284 | 480 | 10 | 12 |
| Truncation | 0 | 7 | 12,297 | 0 | 0 |
| Segment edges | 293 | 24 | 19,569 | 329 | 65 |
| Header | 26 | 366 | 488 | 0 | 0 |
| Segment count | 8 | 0 | 4 | 0 | 0 |

Each test asserts its case total and that its targeted reject classes occur.

## 3. Ring-3 return checks

| Test | Enumeration | Cases |
| --- | --- | --- |
| `return_flags_keep_only_arithmetic_flags_and_force_if` | all 65,536 low-16-bit patterns, each high bit 16..63 alone and cleared from `u64::MAX`, and `u64::MAX` | 65,633 |
| `valid_return_layout_selector_rip_rsp_product` | 12 CS × 12 SS selectors × 10 RIP × 8 RSP edges × 3 stack ranges (default, deep B03 profile, empty) | 34,560 |

- `return_flags`: every output bit is checked one by one. CF, PF, AF, ZF, SF and
  OF are copied from the input. Reserved bit 1 and IF are set. TF, DF, IOPL, NT,
  RF, VM, AC, VIF, VIP, ID and every reserved high bit are cleared.
- `valid_return_layout`: the result is true exactly when `cs == 0x23`,
  `ss == 0x1b`, `0x40000000 <= rip < 0x60000000` and `rsp` lies in the given
  stack range. Selectors with extra high bits (0x10023) and the non-canonical
  RIP alias 0x0001000040000000 are rejected. Accepted frames use ring-3
  selectors only. 24 cases are accepted (none with the empty stack range).

## Mutation controls

Each control was a temporary edit of `crates/kernel-core/src/user.rs`. The file
was restored from a saved copy afterwards (SHA-256 `4abcf36e…5b6f60` before and
after), and `git diff --exit-code` was clean (exit 0).

| Control (temporary source edit) | Result (test exit 101) |
| --- | --- |
| `bounded_copy_plan`: `\|\| end > 0x80000000` removed | `copy_plan` and `sector_input_plan` tests fail: `copy 0x80000000+1 … Good: got Ok(…), expected Err(AboveWindow)` and `sector 0x7ffffe01 Good` |
| `bounded_copy_plan`: `\|\| page.owner != pid` removed | both copy tests fail: `copy 0x40000000+1 … ForeignAll: got Ok(…), expected Err(Foreign)` |
| `return_flags`: mask widened from 0x8d5 to 0x9d5 (TF) | flags test fails: `flags 0x100 bit 8` |
| `parse`: flags check widened from `4..=6` to `4..=7` | single-byte test fails: `byte 236=0x07: flags 7` |

## Evidence

The commands below ran in the worktree, with `CARGO_TARGET_DIR` and
`NOS_BUILD_DIR` set inside it. Raw logs are in the ignored `build/b05-t5/`
directory and are not committed.

| Command | Exit |
| --- | --- |
| `cargo +1.100.0 test --locked -p kernel-core --test bounded_user_abi_invariants` | 0 (10 tests) |
| `cargo +1.100.0 test --locked -p kernel-core` | 0 (59 unit and 10 integration tests) |
| `cargo +1.100.0 clippy --locked -p kernel-core --all-targets -- -D warnings` | 0 |
| `cargo +1.100.0 fmt --all -- --check` | 0 |
| `python3 -B tools/b05_scope_check.py` | 0 |
| `python3 -B -m unittest discover -s tools/tests -p test_b05_scope_check.py -v` | 0 |

Runtime: 0.34 s for the test binary in the debug profile on a 32-thread host.
It took 0.50 s when built with `RUSTFLAGS="-C instrument-coverage"`, which is
the instrumentation `cargo llvm-cov` uses. `cargo-llvm-cov` itself was not
installed locally, so the `ci.yml#host-quality` coverage step was not run.

## What this does not prove

- Inputs outside the enumerated sets are untested. These include other pointer
  values, multi-byte ELF mutations other than the listed pairs, other
  program-header placements and other selector values.
- The lookup model is synthetic. The kernel's real page-table walk closures in
  `kernel/src/m2/m3.rs` and `kernel/src/m2/m3/`, the physical copy, TLB invalidation, and races with
  unmapping on other CPUs are not exercised.
- The oracles are derived from the same source file, so they are not an
  independent specification. They catch regressions against the current rules,
  not errors in those rules.
- `parse` ignores `p_paddr` and trailing bytes, and it does not require file
  ranges to be disjoint from the headers. The test does not treat these as
  defects.
- The overflow results come from the debug profile. Release builds wrap
  integer overflow instead of panicking.

## Findings

None. No rule derived from the source was broken within these bounds.

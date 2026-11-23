# B05-T4: bounded invariant checks for ELF and policy-envelope parsing

Status: host-only bounded enumeration. It is not a proof, not a formal
verification and not an independent human audit. It does not accept B05.
Baseline: `origin/main` 2f4ac6ae814fac18bca52ff80d4b9f774d12bec2, toolchain
`1.100.0` (from `rust-toolchain.toml` and `ci.yml#host-quality`).

## 1. `crates/elf-loader-core/tests/bounded_parser_invariants.rs`

Entry point: `elf_loader_core::parse`. The starting input is a minimal valid
three-segment ELF64 (RX, R and RW; 0x3010 bytes). The test builds it in code.

### What it enumerates (debug profile, overflow checks on)

| Test | Enumeration | Cases |
| --- | --- | --- |
| `every_header_and_program_header_byte` | Each byte 0..232 (ELF header and all program headers) set to {0x00, 0x01, 0x7f, 0x80, 0xff, b^0x01}, skipping duplicates and the original value | 1,001 |
| `every_truncation_rejects` | Every prefix length 0..len | 12,304 |
| `segment_field_edge_products` | For each segment: every pair of offset, address (p_vaddr = p_paddr), p_vaddr alone, file size, memory size and alignment over a 26-value edge set. The last segment also gets the complete 4-way product of offset × address × file size × memory size. | 499,564 |
| `header_entry_phoff_phnum_edges` | e_entry × e_phoff over the edge set × 9 e_phnum values (0..4, 63, 64, 65, 0xffff) | 6,084 |
| `two_segment_placement_grid` | Segment 1 placed at 0x40 steps over four pages × 7 memory sizes × 2 file sizes, with the other segments fixed | 3,598 |

The edge set contains:

- 0, 1, 16 and 64;
- the page edges 4095, 4096, 4097 and 8192;
- the kernel-window edges: start-1, start, start+0x40, start+page, end-page, end-1 and end;
- the file-length edges len-16, len-1, len and len+1;
- `u64::MAX`, `u64::MAX-4094` (the page round-up overflows), `u64::MAX-len+1`, `1<<63`, `i64::MAX`, 0xffff_8000_0000_0000 and 0xffff_ffff_8000_0000.

### Checked invariants

The oracle is written from the rules in the crate source, which is its primary source: `crates/elf-loader-core/src/lib.rs` (BaryonOS project, `parse` and its `Error` variants) at commit `2f4ac6ae814fac18bca52ff80d4b9f774d12bec2`, <https://github.com/BaryonOS/BaryonOS/blob/2f4ac6ae814fac18bca52ff80d4b9f774d12bec2/crates/elf-loader-core/src/lib.rs>. The policy-envelope oracle likewise derives from `crates/update-core/src/policy_envelope.rs` at the same commit, <https://github.com/BaryonOS/BaryonOS/blob/2f4ac6ae814fac18bca52ff80d4b9f774d12bec2/crates/update-core/src/policy_envelope.rs>. The oracle is checked against the (mutated) input bytes.

- `parse` never panics within the enumerated inputs. Every call runs under `catch_unwind`, which reports the case that panicked.
- A rejection is a typed `Error` value; a partial plan is never returned.
- Each test asserts its exact case total (the counts in the table above) and checks that its expected outcome classes actually occur:
  - accepted plans in the byte, edge, header and grid tests (the truncation test expects none);
  - `Size`, `Header`, `Permissions`, `Arithmetic`, `Segment`, `Overlap` and `Entry`, each in the tests that target it.
- In every accepted plan:
  - The plan has between 1 and `MAX_SEGMENTS` segments.
  - For every segment, `offset + file_size` does not overflow and is no greater than the input length.
  - `file_size <= memory_size` and `memory_size > 0`.
  - The flags are R, RX or RW. This is W^X: there is never W+X and never a non-readable segment.
  - `[paddr, paddr + memsz)` does not overflow and lies inside `[KERNEL_START, KERNEL_START + KERNEL_LEN)`.
  - `offset ≡ paddr (mod PAGE_SIZE)`.
  - Page-rounded ranges are pairwise disjoint, so no page carries mixed permissions.
  - `entry` equals e_entry and lies in the file-backed bytes of exactly one RX segment.
  - The plan equals, in order, the input's PT_LOAD headers with memsz > 0, and each of those has p_vaddr = p_paddr.
  - `parse` is deterministic: the same bytes give the same plan.

### Observed outcome counts

These are from a local run.

| Enumeration | Accepted | Rejected |
| --- | --- | --- |
| Single byte | 146 | Size 29, Header 101, ProgramType 48, Segment 573, Permissions 52, Overlap 14, Entry 38 |
| Header | 6 | Size 3,120, Header 2,028, Arithmetic 156, Entry 774 |
| Grid | 1,171 | Segment 257, Overlap 2,170 |
| Truncation | 0 | Size 232, Segment 12,072 |
| Segment edges | 1,723 | Arithmetic 162,334, Segment 334,212, Overlap 1,214, Entry 81 |

Runtime: about 0.7 s for all six tests on a 32-thread host.

### What this does not prove

- Inputs outside the enumerated sets are untested. These include:
  - multi-byte mutations other than the listed products;
  - more than three PT_LOAD segments;
  - other program-header placements.
- `parse` does not promise these properties, and the test does not check them:
  - that file ranges are disjoint from each other or from the headers;
  - that trailing bytes are rejected.
- Physical loading, page-table mapping and the UEFI loader's use of the plan are not exercised.
- The no-panic and overflow results come from the debug profile. Release builds wrap integer overflow instead of panicking, so the checked arithmetic in the source is what matters there.

## 2. `crates/update-core/tests/bounded_policy_envelope_invariants.rs`

Entry points: `policy_envelope::verify_policy` and `validate_successor`.

The file is compiled only with `experimental-policy-test`:

- Run it with `--features update-core/test-policy-envelope-probe`, as `signed-updates.yml` does.
- Without the feature it compiles to zero tests, so the `ci.yml#host-quality` job does not run it.

Inputs:

- The committed public TEST vectors in `src/policy_envelope/probe-fixtures/`: `policy.bin` (one signer, 5,493 bytes), `policy-premature.bin`, `policy-expired.bin`, `policy-revoked.bin` and the pinned TEST root.
- These are included with `include_bytes!`. No key is generated or embedded.

### What it enumerates

| Test | Enumeration |
| --- | --- |
| `fixture_layout_and_original_verifies` | The unmodified envelope verifies (version 1, one signer). |
| `single_byte_mutations_get_the_exact_reject` | Every byte position × up to four values {0x00, 0xff, b^0x01, b^0x80}, skipping duplicates and the original value. This makes 21,760 cases with real Ed25519 and ML-DSA-65 verification. The work is split over at most eight threads. |
| `every_truncation_and_append_is_a_format_reject` | Every length 0..5,493, plus 1..=16 appended bytes for each of four byte values. |
| `any_other_root_is_rejected_before_signature_work` | A flip of the least significant bit (`^= 1`) at each of the 1,984 root public-key bytes. |
| `successor_accepts_only_strictly_newer_linked_policies` | All 16 ordered pairs of the four committed policies passed to `validate_successor`, compared with an oracle for its root, version/predecessor and revocation-retention conditions. |

### Checked invariants

- Among the tested single-byte mutations, only the unmodified original verifies; this does not establish behaviour for multi-byte changes. Every tested single-byte change gets the exact typed reject for its region:
  - magic, format version, signer count and body length: `Format`;
  - target: `Target`;
  - a zero version or a zero minimum image version: `Format`;
  - root identifier: `Root`;
  - signer validity window, revocation byte and padding: `SignerTable` when the encoding becomes invalid, otherwise a signature reject;
  - version, floor, predecessor, signer keys and both signature legs: `Core(Signature)`.
- Every truncation and every append is `Format`, so non-canonical lengths are rejected before any signature work.
- Each reject class is reached at least once. The local run gave:
  - `Format` 60, `Target` 63, `Root` 128 and `SignerTable` 48;
  - `Core(Signature)` 13,399 in the signature region and 8,062 in the body.
- A wrong root gives `Root` for the tested change: the least significant bit (`^= 1`) of each of the 1,984 root public-key bytes. Other bit positions are not enumerated.
- `verify_policy` itself has no version floor. Stale, replayed and unlinked versions are rejected by `validate_successor`, which accepts a pair only when all of these hold, checked in this order:
  1. the root identifier (body bytes 80..112) is equal, else `Root`;
  2. the next version is strictly greater and its predecessor digest equals the current digest, else `Successor`;
  3. every revoked signer pair in the current table is present and still revoked in the next table, else `Revocation`.
- The test oracle models all three conditions, but it is bounded to the 16 ordered pairs of the four committed fixtures. These share one root, and the only linked successors follow a policy without revoked signers, so only `Ok` (1 → premature, 1 → expired) and `Successor` (for example premature → 1, and 1 → revoked with its zero predecessor) are reached. `Root` and `Revocation` rejects are covered by the existing `tests/policy_envelope.rs`, not by this enumeration.

Runtime: about 33 s for the debug build (dependencies at opt-level 0) on 8 threads.

### What this does not prove

- Multi-byte mutations are not enumerated.
- Envelopes with more than one signer, and therefore the signer-ordering, duplicate and revocation-conflict rules, are covered only by the existing `tests/policy_envelope.rs`.
- No cryptographic property is proven: there is no forgery or unforgeability claim.
- The test does not cover:
  - `verify_bound_image` and binding envelopes;
  - native loader or ring-3 probe behaviour;
  - anchor persistence and rollback resistance.

## Mutation controls

Each control was a temporary source edit, made and then restored; `git diff --exit-code` was clean afterwards.

| Control (temporary source edit) | Result |
| --- | --- |
| `elf-loader-core`: the `Error::Overlap` return replaced by a no-op | The grid, single-byte and edge tests fail: "page overlap". |
| `elf-loader-core`: the flags check widened from `4..=6` to `4..=7` | The single-byte test fails: "byte 180=0x07: flags 7". |
| `update-core`, `verify_hybrid` in `src/lib.rs`: the condition `if !pq_key.verify_with_context(message, b"", &pq_sig)` changed to `if !pq_key.verify_with_context(message, b"", &pq_sig) && false`, so an invalid ML-DSA signature is never rejected | The mutation test fails at ML-DSA-region positions such as 2748, 3435 and 4122. |

## Findings

None. No promise derived from the source was broken within these bounds.

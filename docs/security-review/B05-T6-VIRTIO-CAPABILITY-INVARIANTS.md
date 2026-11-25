# B05-T6: bounded invariant checks for the virtio PCI capability walk

Status: host-only bounded enumeration. It is not a proof, not a formal
verification and not an independent human audit. It does not accept B05.
Baseline: `origin/main` a91d5ade, toolchain `1.100.0` (`rustc 1.100.0
(0ef1bc386 2026-11-09)`). Scope boundary: `virtio-dma` in
[scope.json](scope.json); the test is listed under that boundary.

## Target

`virtio_core::parse_capabilities_for` (`crates/virtio-core/src/lib.rs`) walks
the device-controlled 256-byte PCI configuration snapshot. It decides the
common, notify, ISR and device-configuration MMIO windows that the kernel
drivers (`virtio-blk`, `virtio-rng` and `kernel/src/m2/snd.rs`) map. Before this
task, the only enumeration (`src/tests.rs`,
`pci_bounded_single_byte_mutations_never_index_outside_snapshot`) tried six
values per byte and discarded the result.

The new test is `crates/virtio-core/tests/bounded_capability_invariants.rs`. It
uses only the public API and the standard library, and it changes no `Cargo`
file. The parser source is not modified.

## Snapshot

The test builds a modern virtio-pci snapshot in code. Its layout follows QEMU:

- Header: vendor `0x1af4`, the requested device ID, status capability-list bit
  set, revision ID 1, header type 0, and subsystem fields.
- BAR0/BAR1: a 64-bit memory BAR at `0x8_0000_0000` with size `0x1000`. It
  holds all four regions.
- BAR2: a 32-bit memory BAR at `0xfebd_1000` with size `0x1000`.
- BAR3: an I/O BAR.
- Capability chain: MSI-X at `0x40`, then vendor capabilities common `0x4c`,
  notify `0x5c` (multiplier 4), ISR `0x70`, device `0x80` and PCI-config access
  (type 5, ignored by the parser) at `0x90`.
- Regions in BAR0: common `[0x000, 0x038)`, notify `[0x038, 0x048)`, ISR
  `[0x048, 0x049)`, device `[0x050, 0x058)`.

Device IDs: block `0x1042`, entropy `0x1044` and sound `0x1059`. The sound ID
is `0x1040 + 25` from `drivers/virtio-snd/src/lib.rs:16`. It is written as a
literal because `virtio-snd` is not a dependency of `virtio-core`.

## Enumeration and exact totals

Every case calls the parser under `catch_unwind` and calls it a second time to
check determinism. It then classifies the result. Each test asserts its exact
case total and the exact count of each outcome class from the run below.
Identity mutations (a byte set to its original value, a capability "moved" to
its own offset, and duplicate `wide_before` cases at BAR index 0) are counted.

| Test | Enumeration | Cases | Ok | Invalid | Unsupported | Range | Protocol |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `every_config_byte_takes_every_value` | 3 device IDs × 256 byte positions × all 256 values | 196,608 | 147,390 | 432 | 10,650 | 28,434 | 9,702 |
| `every_capability_at_every_aligned_offset` | each of the 6 capabilities moved to every 4-aligned offset `0x00..=0xfc`, chain rewired, moved capability written last | 384 | 157 | 5 | 50 | 88 | 84 |
| `bar_grid` | region BAR index 0..=5 × 7 register types (32-bit, 32-bit prefetchable, 64-bit, 64-bit prefetchable, legacy below-1M, reserved, I/O) × 8 sizes {0, 3, 0x40, 0x80, 0x1000, 2^32, 2^51, 2^52} × 9 bases × `is_memory` × register agrees/disagrees × previous register 64-bit or not | 24,192 | 188 | 21,672 | 960 | 1,372 | 0 |
| `capability_offset_length_and_multiplier_edges` | kinds 1..=4 × 18 offset edges × 14 length edges, × 7 multiplier edges for notify | 2,520 | 213 | 0 | 0 | 1,440 | 867 |
| `device_id_must_match_the_requested_profile` | 3 built IDs × 8 requested IDs (the three, `0x1041`, `0x1001`, `0x1000`, `0`, `0xffff`) | 24 | 3 | 0 | 21 | 0 | 0 |
| `fixture_is_accepted_with_exact_regions` | the snapshot for each of the 3 IDs | 3 | 3 | — | — | — | — |

The nine BAR bases are 0, `0x1000`, `0x1010` (misaligned),
`0xfeb0_0000`, 2^32, `PHYSICAL_LIMIT - size` (ends exactly at the limit),
`PHYSICAL_LIMIT - size + 0x1000`, `PHYSICAL_LIMIT` (ends past the limit) and
`2^64 - size` (the end wraps). The grid includes a region in the upper half of
a 64-bit BAR (`wide_before`) and a 64-bit BAR at index 5, which has no upper
register.

The offset edges are 0, 1, 2, 4, 7, 8, `0x38`, `0x48`, `0x50`, `0x58`, `0x60`,
`0xff8`, `0xffe`, `0xfff`, `0x1000`, `0x8000_0000`, `u32::MAX - 7` and
`u32::MAX`. The length edges are 0, 1, 2, 7, 8, 55, 56, 57, `0x10`, `0xfa8`,
`0xfff`, `0x1000`, `0x1001` and `u32::MAX`. The notify multiplier edges are
0, 1, 2, 3, 4, `0x8000_0000` and `u32::MAX`.

Capability length bytes, pointers, BAR index bytes and `cfg_type` bytes take all
256 values through the byte enumeration.

`Error` has twelve variants. The walk can return only `Invalid`, `Unsupported`,
`Range` and `Protocol`. Any other variant fails the test. All four are reached:
`Invalid`, `Unsupported` and `Range` in the BAR grid, and `Protocol` in the byte
enumeration. The other eight variants are reached zero times.

Runtime: about 0.3 s for all six tests (debug profile, 32-thread host).

## Oracle for every accepted snapshot

The oracle is written from the rules in the parser source and from the PCI
BAR encoding. It is computed from the mutated bytes and the `Bar` array, not
from parser internals. Address arithmetic uses `u128`.

- The header holds vendor `0x1af4`, the requested device ID, header type
  `& 0x7f == 0`, and the status capability-list bit (`0x10`).
- An independent chain walker follows the chain from `config[0x34]`. Every
  pointer it visits is in `0x40..=0xfc`, 4-aligned and not repeated. No
  capability ID is `0x00` or `0xff`. Every vendor capability has length at
  least 4 and fits in the snapshot.
- Each kind 1 to 4 appears exactly once in the chain. Its capability length is
  at least 16, or 20 for notify. The returned `Region` equals that
  capability's BAR, offset and length bytes.
- `notify_multiplier` equals the kind-2 field at capability offset `+16`.
- Minimum lengths are 56, 2, 1 and 8. Offset alignments are 8, 2, 1 and 8.
- The region BAR index is below 6. Its `Bar` is memory, with a power-of-two
  size, a nonzero base aligned to its size, and `base + size <= PHYSICAL_LIMIT`.
- An independent decoder of the configuration BAR registers gives exactly
  `Bar.base`. The decoder returns no base for I/O BARs, reserved types, a 64-bit
  BAR at index 5, or the upper half of a 64-bit BAR.
- `[base + offset, base + offset + length)` lies inside `[base, base + size)`,
  and `Region::address` returns `base + offset`.
- The four absolute intervals are pairwise disjoint.
- For queue indices 0, 1, 2, 3, 4, `0x100` and `u16::MAX`, `notify_address`
  is `Ok(start + q × multiplier)` exactly when that address is even and
  `address + 2` stays within the notify region. Otherwise it is an error.
- The parser is deterministic: a second call returns the same result.

`device_id_must_match_the_requested_profile` also checks that a snapshot parsed
with any other requested ID is rejected as `Unsupported`.

## Mutation controls

Each control was a temporary edit to `crates/virtio-core/src/lib.rs`, applied
from a pristine copy (SHA-256
`24dc240458ff4558eac959c0512b11e73ca9b09da28ddd5179bf2b4361cbd293`) and then
restored. `git diff --exit-code -- crates/virtio-core/src` returned 0
afterwards, and the test passed again on the restored source.

| Control (temporary source edit) | Raw exit | Failing tests and first failure |
| --- | ---: | --- |
| M1: overlap loop disabled (`if start < other_end && other < end && false`) | 101 | Byte enumeration: "id 0x1042 byte 0x54=0x8: kinds 1 and 2 overlap". Edge test: "kind 1 offset 0x0 length 0x39: kinds 1 and 2 overlap". |
| M2: pointer bound relaxed from `0x40..=0xfc` to `0x00..=0xfc` | 101 | Byte enumeration: "byte 0x81=0x8: accepted chain pointer 0x8", where the chain reaches revision ID 1 and prog-IF 0. Relocation: "cap 0 at 0x18: accepted chain pointer 0x18". |
| M3: `checked_bar(...)` call removed | 101 | BAR grid: "bar 0 ... base 0x0 size 0x80: bar 0 shape". Byte enumeration: "byte 0x10=0x0: register". The relocation tally also changed. |
| M4: minimum-length check disabled (`region.length < minimum && false`) | 101 | Byte enumeration: "byte 0x58=0x1: min length". Edge test: "kind 1 offset 0x0 length 0x1: min length". |

The upper bound `0xfc` cannot be removed observably on its own, because a 4-aligned
`u8` pointer cannot exceed `0xfc`. Only the lower bound is a meaningful
mutation.

## Acceptance commands

Run in the worktree with `CARGO_TARGET_DIR=<worktree>/target`, on unmodified
parser source.

| Command | Raw exit |
| --- | ---: |
| `cargo +1.100.0 test --locked -p virtio-core --test bounded_capability_invariants` | 0 |
| `cargo +1.100.0 test --locked -p virtio-core` (21 unit + 6 new tests) | 0 |
| `cargo +1.100.0 clippy --locked -p virtio-core --all-targets -- -D warnings` | 0 |
| `cargo +1.100.0 fmt --all -- --check` | 0 |

## What this does not prove

- Inputs outside the enumerated sets are untested. These include:
  - multi-byte mutations other than the listed products and relocations;
  - other capability orders and chain lengths;
  - BAR layouts with regions spread over several BARs, except where a
    single-byte BAR index change produces one.
- Only capability discovery is covered. The test does not cover:
  - virtqueue or descriptor validation;
  - DMA pool planning;
  - IOMMU or ACCESS_PLATFORM behaviour;
  - real DMA, MMIO ordering, reset or quiescence;
  - hardware.
- The kernel's BAR sizing and the agreement of the `Bar` array with real
  hardware are inputs here, not checked properties.
- The no-panic and overflow results come from the debug profile.
- No native QEMU run was made. This host test alone does not establish a
  native or hardware claim, and it earns no B05 acceptance.

## Findings

None. No accepted snapshot broke an oracle within these bounds.

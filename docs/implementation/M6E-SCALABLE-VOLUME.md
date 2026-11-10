# M6e — scalable document storage with space reuse (S04/S05)

Date: 28 October 2026. Baseline: `main` `6e12dcb` (20/100). Working branch
`ccr-d3fe1610-gl4r8s`. Evidence status: [M6E-ACCEPTANCE](../verification/M6E-ACCEPTANCE.md).

> **Superseded by volume v2 (S06, ADR-041).** Format `NOSVOL01` and IPC v5
> below describe the source on which S04/S05 were accepted. Profile
> `native-volume` now uses `NOSVOL02`/IPC v6 with collections, metadata,
> history and transactions: [S06-COLLECTIONS-HISTORY](S06-COLLECTIONS-HISTORY.md).
> Commit order, invariants 1–7 and crash/recovery rules apply unchanged;
> the S06 gate repeats the S04/S05 scenarios on v2.

## Objective and boundaries

M6c stored four objects of at most 128 bytes in an append-only journal of
32 records; the volume then remained permanently full. M6e replaces that
trial limit with a **copy-on-write volume** supporting large documents, space
reuse without a record limit, per-domain quotas and crash-consistent release.
M6c/M6d remain unchanged and separately buildable; no automatic migration.

Trial budget (project choice, not a product limit): 16 objects (8 per domain),
documents up to 64 KiB, 64 chunks of 4 KiB (256 KiB content), quota of 40
chunks per domain. Quotas deliberately overcommit the volume (2 × 40 > 64),
so both `Quota` (own domain) and `Full` (shared volume) are actually reachable.

Outside this step: encryption/authentication (B02), backup/export (S09),
collections/metadata/revision history (S06), physical disks, larger memory
or multiple volumes, and automatic migration of M6c volumes.

## Architecture

| Layer | Component | Authority |
|---|---|---|
| Ring 3 | `services/volume` (`volume-core`, `no_std`, `forbid(unsafe_code)`) | All format, domain, quota and revision policy |
| Ring 3 | `sdk/examples/volume-docs` `volume-service` | One scoped block capability + two kernel-bound ingress endpoints |
| Ring 3 | `volume-app` (two instances) | Only a send/reply endpoint pair; no block right |
| Kernel | `kernel/src/m2/m3/volume.rs` (`native-volume`) | Generic block broker: scope, rights, tickets, bounded copies |
| Host | `tools/volume_disk.py` | Independent Python disk oracle (separate implementation) |

The kernel knows no objects, names, revisions, domains or volume format.
It counts only sectors/publications and sets test barriers (without authority).
The SDK runtime gained a small block API (`block_read/write/flush/wait`) plus
pure, host-tested completion-envelope validation.

## Disk format v1

Scoped sectors `[4, 534)` of a 1 MiB image. Sectors outside that range contain
canaries; the oracle rejects any change to them.

- Table copy A: sectors 0–8; copy B: 9–17. Generation `g` always resides in
  copy `g % 2`. Sector 0 = header (magic `NOSVOL01`, version, geometry,
  generation, commit request ID, quota); sectors 1–8 = two entries of 240 bytes.
- Each table sector ends with `u64 generatie`, `u64 pogingdigest` (generation
  and attempt digest; FNV-64 over payload bytes 0–487 of all nine sectors of
  that copy), and an FNV-64 over slot, index and the first 504 bytes. A torn
  or displaced sector is thus detectable per sector, and sectors from two
  different write attempts for the same generation can never combine into
  a valid copy (finding from CodeRabbit review of PR #47).
- Entry: revision, request ID of last mutation, length, presence, name (at
  most 32 UTF-8 bytes, M6c name rules), chunk list (≤ 16) with per-chunk FNV
  and a content checksum. Tombstones retain revision + request ID. All unused
  fields are canonically zero / `0xFFFF`.
- Content: chunk `c` = sectors `18 + 8c … 18 + 8c + 7`. The final sector is
  zero-padded; oracle and service check that padding.

An entirely zero volume is empty generation 0; nothing is formatted.

## Durability order and invariants

1. New content goes **only** to chunks not referenced by the durable table
   and not reserved by another session.
2. Commit: final content sector → FLUSH → next table copy in the **older**
   slot → FLUSH → live state → ACK. The slot containing the current generation
   is never overwritten.
3. An error before the table write is a **definite** abort (`Io`): durable
   state is unchanged and the service remains usable. An error during or
   after the table write is `Unknown`: the instance then rejects every valid
   request (validation first, then `Unknown`) and never repeats the write.
4. Recovery selects the newest completely valid copy, but only if the other
   copy is the preceding generation or an interrupted attempt at the next
   (per sector: generation `g−1`, `g+1`, zero only when `g = 1`, or torn).
   A copy whose sectors are all validly sealed but whose content is
   non-canonical is corruption and stops recovery (no silent fallback).
5. Space from overwritten or deleted objects becomes free once the next
   table is durable; a subsequent commit reuses it (lowest free chunk first,
   deterministically). There is no journal, therefore no record limit or
   separate compaction step.
6. Quotas count **committed plus staged** chunks per domain; an overwrite
   therefore temporarily requires space for old + new (copy-on-write).
7. Content is verified against its checksum per chunk on read. Corruption
   returns `Corrupt`, never silently different bytes.

**Known residual risk** (fixed in a test): one unreadable sector in the newest
table copy cannot be distinguished from an interrupted, unflushed rewrite of
that slot. Recovery then falls back to the previous generation (loss of the
last commit); content remains checked per chunk. FNV is not authentication;
an attacker with disk access can make consistent modifications. This falls
under B02.

## IPC v5 (224 payload bytes in Message320)

TAG `0x4d36454c4b000005`, version 5. Header: version/operation, request ID,
epoch, object, revision, length, status, name length, count, aux; data 64–223
(160 bytes). Operations: STAT, BEGIN, APPEND, COMMIT, ABORT, READ, DELETE,
RENAME, LIST (paginated, 3 entries per page with domain generation).

- Streaming upload: `BEGIN(object, verwachte revisie, totale lengte, naam)` →
  `APPEND(offset == ontvangen, ≤ 160 bytes)` … → `COMMIT`. The Dutch arguments
  mean expected revision, total length, name and received count. One session
  per domain; a new BEGIN from the same kernel-bound domain replaces an
  abandoned session (for example after app restart).
- `READ(object, actuele revisie, offset, max ≤ 160)` (current revision);
  never across a chunk boundary; an older revision returns `Stale`.
- Domain derives exclusively from the endpoint. An object from the other
  domain returns `Access`, an object ≥ 16 returns `Range`. All unused bytes
  must be zero; deviations return `Invalid` without I/O.
- The client strictly validates every response (echo, padding, per-operation
  shape) and treats deviations as `Protocol`.

## Acceptance

1. Host tests `volume-core` (17): large documents, reuse (60 commits,
   > 4 × capacity allocated), Quota/Full before I/O, domain isolation,
   malformed requests, streaming rules, tombstones, read-only, corruption,
   definite I/O errors, `Unknown` at every table-write position, and an
   **exhaustive crash simulation** over 10 mixed steps: every mutating disk
   operation interrupted, with prefix, torn, reordered and suffix persistence
   of the write cache. Mutation tests confirmed that omitting the flush
   before the table write and writing to the active slot are caught.
2. Oracle tests (`tests/test_volume_disk.py`, 8) and SDK completion tests.
3. Native gate `tools/test_volume.py`, seven actual QEMU invocations on one
   frozen source: normal → cold read-only recovery of exactly the same image;
   kill during content writes → cold recovery; kill during table write → cold
   recovery; corrupt image rejected before READY.
4. Every successful run requires exact marker order, raw exit 33 (or an
   actual host kill after the cut marker), image-hash binding, unchanged
   image when read-only, untouched canaries outside scope, and matching
   domain digests between guest report and independent oracle.

An emulator kill is not physical power loss; FNV is not authentication.
No production, encryption or physical-disk claim is made.

# M6c — native document names and a protected catalogue

Date: 28 October 2026. Status: current M6c contract with existing implementation.
Original baseline: `bb22fdbe15919edc23baa158a46236195ce314b2` (completed M6b).
Working branch: `codex/m6c-named-catalog`. Evidence status and acceptance boundaries:
[M6C-ACCEPTANCE.md](../verification/M6C-ACCEPTANCE.md).
Historical evidence boundary at `bc4cd104` and the subsequent M6c review state:
the native observer could panic on WRITE/FLUSH errors. That earlier execution
had only host ABI evidence for Unknown. M6d fixes this on new source and adds
native fault injection; see [M6D-ACCEPTANCE.md](../verification/M6D-ACCEPTANCE.md).
The original M6c result below remains bound to its own source.

## Objective and boundaries

An actual ring-3 service lets two isolated native apps create, retrieve,
update, rename, delete and list named documents. This provides the first native
catalogue interface for Materiaal, not a graphical shell. Object ID remains
the identity and authorisation basis. Names are labels, not paths; duplicate
names are allowed. There is no global search or directory service.

Retain tested limits: four objects (0–3), two per endpoint domain, at most
128 content bytes, 32 append-only records and only sectors 4–35. A name has
at most 32 UTF-8 bytes. These are trial budgets, not product limits. The M6b
crate and M6b format remain unchanged and separately buildable.

## Invariants

- Endpoint and kernel-populated sender PID determine the domain. App data
  grants no authority. LIST sees only present objects owned by that domain.
- Names are non-empty, valid UTF-8 and contain no NUL/control characters,
  slash, backslash or bidi override/isolate characters (U+202A–202E, U+2066–2069).
  `.` and `..` are rejected. No normalisation or case conversion; allowed
  labels are retained and compared byte-for-byte.
- PUT stores name and content together. RENAME requires an existing object
  and current revision and preserves all content bytes. DELETE retains the
  incremented revision but removes name/content from live state and catalogue.
- Mutations: validate/prepare → WRITE completion → FLUSH completion → commit
  live state → ACK. On uncertainty, LIST also fails closed with Unknown; no
  automatic replay or data from potentially stale state.
- Full returns Full before I/O, without wraparound/overwrite. Recovery
  requires a complete valid prefix followed only by empty records. Corruption
  closes the service.
- LIST is one atomic view of at most two present objects in one domain, in
  ascending object-ID order. No content, tombstone names, global count or
  global journal sequence. Catalogue revision is the sum of only the two
  owned object revisions (including tombstones).
- Catalogue responses, including for an empty domain, expand no rights. A
  subsequent GET or mutation is reauthorised; revisions are not grants.

## IPC v4 (224 payload bytes in Message320)

Existing M6b v3 padding is not reused under the same version.
TAG = `0x4d36434c4b000004`, version = 4. GET=1, PUT=2, DELETE=3,
RENAME=4 and LIST=5. Status codes remain the same as M6b.

| Offset | Content |
|---|---|
| 0 / 2 | u16 version / operation |
| 4 | Four zero bytes |
| 8 / 16 / 24 / 32 | u64 request ID / service epoch / object ID / revision |
| 40 / 44 | u32 content length / i32 status |
| 48 / 52 | u32 name length / u32 catalogue count |
| 56–63 | Zero |
| 64–191 | Content, or at most two catalogue entries for LIST |
| 192–223 | Name in PUT/RENAME request and successful GET reply |

All unused bytes are zero. Constructor overflow becomes an explicitly invalid
bounded message, not a panic or silent truncation. GET/DELETE contain no name
or content. PUT requires a name and accepts empty content. RENAME requires a
name and no supplied content. Mutation ACKs contain only the new revision,
no name/content. NotFound may return only the authorised tombstone revision;
other errors have revision zero and empty data.

LIST request: object ID exactly the own-domain start (0 or 2), revision/
lengths/count zero. Cross-domain selectors return Access; an odd selector
within the own domain returns Invalid. LIST reply echoes selector/request
ID/epoch, supplies domain revision, count 0–2 and length `56 * aantal`
(56 × count). A 56-byte entry: object ID u64 at 0, object revision u64 at 8,
content length u32 at 16, name length u32 at 20, name at 24–55 with zero
padding. Remaining payload is zero. The client validates domain, order,
unique IDs, lengths, name, revisions and padding. An empty catalogue returns
Ok, count/length zero and the current own-domain revision.

## Journal v2

512 bytes, magic `NOSM6C02`, version 2, header size 64. Sequence/request
ID/object ID/object revision remain at 16/24/32/40; operation u16 at 48,
content length u16 at 50 and name length u16 at 52. Other header bytes zero.
Content at 64–191, name at 192–223, remainder through 504 zero; diagnostic
FNV64 at 504 as in M6b. PUT contains the complete new name/content. RENAME
record contains the new name plus exactly the existing content; recovery
checks predecessor, revision and content preservation. DELETE has empty
name/content and requires an existing object. Every transition increments
object revision and global sequence by one.

Legacy policy: an occupied M6b v1 record is explicitly rejected as unsupported/
Corrupt before READY; no automatic migration or formatting. The M6b reader
remains available for earlier synthetic volumes. This step changes no real
user volumes. An explicit migrator is later work.

## Implementation boundaries

`services/catalog` is a new dependency-free `no_std` crate `catalog-core`
with `forbid(unsafe_code)`. The native adapter uses the existing scalar
runtime; no new syscall, capability type, block rights or hardware driver.
The kernel contains only the existing block broker and an explicit acceptance
observer for this fixture, not app name/catalogue policy. Native probe:
`userspace/m6c_probe.rs`; selector: `milestone-m6c`.

## Known native observer limitation

This section describes the **historical M6c execution source `bc4cd104`**.
The actual service adapter had been checked through a host ABI shim, but the
kernel observer retained the expected Ok reply for a prepared mutation.
On WRITE/FLUSH errors or timeouts, the correct Unknown reply could therefore
cause a panic before the client received it, including with `milestone-m6c`.

Original native acceptance included no device fault injection, and the
capture at that time rejected `M6C:OUTCOME`. The fifteen host ABI scenarios
did not demonstrate a working native error path. Requirements remained:
no commit without WRITE+FLUSH, no replay and no valid access with uncertain
live state.

**Current extension:** M6d on source `88e6c04` binds Unknown to the actually
observed syscall error and tests eight native error cases plus cold recovery.
The expanded capture accepts only the exact required Unknown evidence form
for those cases. Normal/cut/recovery runs may not claim it. See
[the separate contract](M6D-IO-UNCERTAINTY.md) and
[the new acceptance](../verification/M6D-ACCEPTANCE.md). Automatic service
restart and the historical M6b error-path limitation remain outside M6d.

## Acceptance

1. Host tests for wire/UTF-8/canonicalisation, catalogue filtering, CAS,
   tombstones, renaming, full, corruption, legacy rejection and recovery.
   no_std build/fmt/Clippy.
2. Native normal flow per domain: empty catalogue/GET; create two documents;
   LIST; GET; RENAME first; reject stale rename; GET/LIST; DELETE second;
   LIST/GET tombstone; PUT first; GET/LIST. Final sequence 10: even objects
   live/revision 3, odd objects tombstone/revision 2. Each app performs 16 RPCs.
3. Cold second VM: exact names/content/revisions/catalogue, no disk mutation.
4. Negative native requests: domain boundary for GET/PUT/DELETE/RENAME/LIST,
   foreign version/epoch, invalid UTF-8, name limit, padding, invalid LIST,
   stale revision and incorrect/closed capabilities. No I/O or metadata leak.
5. Full: 32 writes, additional PUT/RENAME rejected before I/O; last name retained.
6. Two actual external kills during RENAME: before WRITE and after FLUSH before
   ACK. Only app A writes in this fixture: first create (sequence 1), then
   rename. New VM sees old name/revision 1 or complete new name/revision 2
   respectively, with exactly the same content. Independent Python disk oracle.
7. Corruption and a valid legacy v1 volume rejected before READY without disk
   mutation.
8. M6b regression and relevant build/boot checks. Source, binaries, exact
   commands, raw output, exit codes, image scope and hashes remain bound.
   New host groups record hashes of stdout, command metadata and stderr
   directly in the gate record. Capture compares source files and copied
   archive bytes against this fixed map; missing binding is rejected. The
   historical suite before this tightening lacks that binding; scope and
   separate host tests appear in
   `../verification/m6c-review/host-log-binding/README.md`.

An emulator process kill is not physical power loss. FNV is not authentication;
DELETE does not erase historical disk bytes. No encryption, native NAS,
graphical desktop, formal security proof or production suitability is claimed.

## Sources and version policy

- Baryon OS. (2026, October 28). *M6B-OBJECT-STORAGE.md; PROJECT-STATE.md*
  (commit bb22fdbe15919edc23baa158a46236195ce314b2). Existing native baseline.
- The Rust Project. (n.d.). *x86_64-unknown-none*. The rustc book.
  https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html
  (accessed 28 October 2026). No std/default allocator; host tests and actual
  guest execution are separate checks. Toolchain remains 1.90.0.

The new formats and trial budgets above are explicit project choices.

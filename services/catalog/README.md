# Native document names and catalog (M6c)

`catalog-core` contains allocator-free, dependency-free Rust `no_std` logic for
the dedicated ring-3 document service. The core forbids `unsafe`; the separately built
`user_service.rs` forms the native IPC/block-I/O boundary.

Four objects, two per endpoint domain, store a UTF-8 label (at most 32
bytes) and at most 128 content bytes. Names are labels; the object ID remains
identity and the basis of authorization. Duplicate names may exist. GET/PUT/DELETE,
CAS-checked RENAME and an atomic LIST of the caller's own domain form
IPC v4 (224 bytes). LIST returns only metadata, never content or global
activity; the catalog revision counts only the domain's own object revisions.

Journal v2 (`NOSM6C02`) contains 32 append-only records of 512 bytes. A RENAME
preserves the existing content exactly, increments the revision and writes one record.
Recovery checks predecessor, revision and content preservation. PUT/DELETE/RENAME
may only be confirmed after both WRITE and FLUSH completion. The service contract
blocks all valid operations, including LIST, after an uncertain mutation until a new
full recovery.

The historical M6c execution source had an observer defect: on WRITE/FLUSH
errors the kernel could panic before the client received `Unknown`. The M6d
extension binds the reply to the syscall error actually returned and
guards that no new block call, publication or completion follows afterwards.
Eight native fault features exercise status errors, timeouts, reset quarantine,
descriptor rejection and invalid used-length. The full native
acceptance status is in [PROJECT-STATE.md](../../docs/PROJECT-STATE.md);
the [M6d contract](../../docs/implementation/M6D-IO-UNCERTAINTY.md) defines the
exact fault/recovery boundary. A process does not yet restart automatically, and physical
disk faults or power loss are not simulated by this.

This core cannot observe physical durability; native adapter evidence
is therefore needed separately. FNV is diagnostics, not cryptographic authentication.
DELETE erases the live name/content, but not historical disk bytes. There is no
space reuse, encryption, migrator, NAS or graphical desktop. Existing
M6b-v1 records are explicitly rejected without formatting; the old M6b route
remains available separately.

Exact contract and acceptance:
[`M6C-NAMED-CATALOG.md`](../../docs/implementation/M6C-NAMED-CATALOG.md).
Executed checks and still-open evidence are in `docs/PROJECT-STATE.md`;
this README does not declare any not-yet-executed test passed.

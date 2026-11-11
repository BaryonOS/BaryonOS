# FR-A03 — streaming document session with volume-upload binding

Implementation above the existing `volume-core` service, without a new kernel
ABI, storage engine, capabilities or disk format. The new SDK route uses the
opt-in v7 envelope with service-bound upload tokens; v6 storage operations
and the legacy client remain available. The old `DocumentSession` remains
byte/API compatible with the M6c catalogue protocol. The new module is
`nieuw_os_document_session::volume`.

## Interface and ownership

`VolumeSession<T>::new(transport, epoch, service_pid, domain, send, receive,
poll_budget)` takes ownership of one previously granted endpoint pair. Use
native `nieuw_os_catalog_client::NativeTransport`; the same transport trait
enables targeted host tests. Only the kernel and service determine access.
`ReadWrite`, object numbers and metadata grant no rights.

`volume::DocumentSession::new(volume, EditMode, Buffers)` manages one document.
The caller supplies three separate, equally sized buffers: draft, last
confirmed content and read staging. Capacity is 1 through the existing
`volume_core::MAX_LENGTH` (64 KiB). The session creates no document-sized
arrays on the stack and uses no allocator. The native app must manage these
buffers within its assigned memory. An oversized document returns a visible
`ContentTooLarge`; previous content is retained. The 64 KiB limit comes from
the existing volume and remains an open product limit for the full test release.

| Call | Actual behaviour |
|---|---|
| `open(object)` | STAT and bounded READ requests; all parts have the same revision. Draft and baseline are replaced only after complete checksum verification. |
| `create(object, name)` | STAT must confirm absence. Tombstone revision remains the CAS baseline; only save creates the object. An empty document is valid. |
| `draft()` | Immutable borrowed view with object, loaded revision, name and bytes. Preserve required bytes before releasing caller buffers. |
| `replace_contents`, `replace_range`, `rename_draft` | Local changes only; validate range, capacity and name before mutation. Content consists of bytes; text boundaries belong to the editor. |
| `save()` | Unchanged draft: no IPC. Otherwise BEGIN, APPEND chunks of at most 160 bytes and COMMIT with loaded revision. Only a validated COMMIT ACK marks the local baseline clean. |
| `begin_save`, `save_next` | Same path with at most one APPEND or COMMIT per `save_next`. App handles input, progress and cancellation between these steps. Every exchange has a bounded poll budget. |
| `cancel_save()` | ABORT on the same upload ID. No COMMIT, no draft loss. Saving after cancellation is a new explicit user action. |
| `metadata()` | Returns current STAT including type, labels, collection and version history. Never silently replaces the loaded CAS baseline. |
| `list(out)` | Paged LIST of own domain. Only a complete list with the same domain clock is published into `out`. |
| `discard_changes()` | Restores only the last loaded/confirmed local baseline; a new unsaved object is released locally. Does not silently resolve uncertainty or conflict. |
| `close()` | Attempts both endpoint closes once; draft remains readable and locally editable. Sends no COMMIT. No hidden syscalls or retries in Drop. |

During upload, content changes and other document operations return `Busy`,
so APPEND cannot mix draft versions. Normal cancellation releases the existing
volume reservation. If the connection is lost, ABORT cannot be confirmed;
an active v7 upload then remains `Busy` until controlled cleanup. Close does
not claim that staging has already been released.

## R1 — distinguishing independent uploads

The first version started request IDs at 1 per SDK session, while v6 matched
an active upload only by domain, object, expected revision and request ID.
Two independent editors could therefore use the same upload ID: B's BEGIN
replaced A's name, after which A could commit under that name and incorrectly
mark its own name as saved. The concrete regression with two actual capability
endpoint pairs fails before the fix.

V7 uses tag `0x4d36454c4b000007`, version 7 and a u32 upload token at offset 4
of the existing 224-byte envelope. Other operations and fields retain their
v6 meaning. BEGIN requests token 0 and, on success, receives a non-zero token
issued by the service. APPEND, COMMIT and ABORT must echo that token; ordinary
read/metadata operations use token 0. Responses echo the token, except for
successful issuance at BEGIN. The client validates this before accepting an ACK.

The service binds an upload to token, actual kernel sender PID and kernel-bound
domain. `Service::handle_for` receives that PID only from the trusted adapter.
The native adapter retains its check that the actual sender is the client
assigned to that ingress. A payload field or predictable token grants no
additional document/domain right. The token distinguishes uploads, including
two editors within the same PID.

One active upload per domain remains: a new BEGIN returns `Busy` and does not
replace active work. The counter is per domain so token issuance reveals no
additional activity in the other domain. It does not wrap during this service
instance; exhaustion returns `Quota` before staging or I/O. After ABORT, commit
or cleanup, a new upload receives a new token. An old token, incorrect PID or
another domain cannot finish or cancel the new upload. V6
BEGIN/APPEND/COMMIT/ABORT return `Busy` if a v7 upload is active; v7 likewise
cannot take over an active legacy upload. In both protocols, `Unknown` remains
the terminal service state and takes precedence over this Busy check.

`Service::detach(principal)` is exclusively a trusted notification that the
principal is dead or **all** its grants to this service have been revoked.
A delayed close message for one old endpoint must not call this function:
it could cancel new work by the same live PID. The function cleans up staging
while retaining already confirmed data and token monotonicity. It is not a
remotely callable administrative operation.

Native notification of principal exit or full grant retirement remains
**open**. No host notification or timer is represented as native cleanup.
A lost BEGIN response can leave an active reservation without a token known
to the client; until that lifecycle integration exists, this requires proven
principal termination or controlled service recovery with new grants/epoch.
A new SDK session or arbitrary other BEGIN must not silently remove that block.

## Errors and security boundary

- The module uses `volume_core::upload` above existing v6 validators.
  Version, epoch, request ID, object, revision, upload token, lengths,
  padding and operations are validated. The outer Message320 must originate
  from the expected service, have the correct tag and contain no capabilities.
  Unexpected received handles are closed; both session endpoints are then closed.
- A sent request is never resent. EAGAIN may be retried only before enqueue.
  Request IDs do not wrap.
- `Stale` retains draft and old revision and blocks subsequent saves with
  `Conflict`. Only explicit discard and reopen obtain a new baseline. No
  automatic merge or overwrite.
- On loss or tampering of the COMMIT ACK, `Unknown` or another uncertain COMMIT
  outcome, the draft remains dirty and the session becomes permanently
  `Uncertain`. Local discard does not remove this state either. Retain desired
  draft bytes and use a new session only after independent service recovery
  and new grants; read actual storage before a deliberate subsequent change.
- A canonical `Io` during data/precommit I/O is a definite abort under the
  existing volume contract. It retains draft and previous revision; they
  are not incorrectly marked saved. Revocation through the actual capability
  engine blocks subsequent requests.
- FNV content checks detect accidental corruption. They provide no encryption,
  authenticity or anti-rollback; B02 remains a separate mandatory integration.
  This library expands no key/block rights.

## Verification and outstanding acceptance

`sdk/document-session/tests/volume.rs` uses the actual capability engine and
existing `volume_core::service::Service` with an explicitly labelled memory
disk. Tests cover 64 KiB streaming, actual core recovery, cancellation,
missing/empty documents, renaming, edits, metadata and LIST, revision conflicts,
revocation, invalid envelopes, partially read documents, definite data I/O
errors and uncertain table writes/lost COMMIT responses. Existing legacy tests
are retained separately.

These are host integration trials. A successful `x86_64-unknown-none` library
build demonstrates target compatibility only. Main integration must call the
same session natively in the actual user app, with checked image/source hashes,
interaction evidence, cold restart and an independent disk oracle. This
sub-step signs off neither native Materiaal, other programs, larger product
storage nor the full test release.

All new dependencies are existing local crates: `volume-core` already depends
on `catalog-core`. There are no new external packages or build scripts. The
integrator updates only SDK lock registration for the added local dependency;
no protocol or lock-version conversion is necessary.

### Initial partial check before R1 (`b34ff68`)

On the FR-A03 app branch, 27 tests of this crate pass (14 legacy, 12 volume
integrations and one request-ID boundary), as do Clippy with `-D warnings`,
formatting and the release library build for `x86_64-unknown-none`. Checks run
offline with Rust 1.90.0 and a private target directory. Because the lead agent
owns the SDK lockfile, the SDK tree was first copied to
`/tmp/nieuw-os-apps-check/sdk` and local dependency registration generated only
there. Tested library/test files are byte-identical to this worktree. Exact
commands, exit statuses and output appear in `/tmp/nieuw-os-a03-verification/`.

```sh
cargo +1.90.0 test --manifest-path /tmp/nieuw-os-apps-check/sdk/Cargo.toml --locked --offline -p nieuw-os-document-session
cargo +1.90.0 clippy --manifest-path /tmp/nieuw-os-apps-check/sdk/Cargo.toml --locked --offline -p nieuw-os-document-session --all-targets -- -D warnings
cargo +1.90.0 build --manifest-path /tmp/nieuw-os-apps-check/sdk/Cargo.toml --locked --offline -p nieuw-os-document-session --lib --release --target x86_64-unknown-none
cargo +1.90.0 fmt --manifest-path /tmp/nieuw-os-apps-check/sdk/Cargo.toml --all -- --check
```

All four use `CARGO_TARGET_DIR=/tmp/nieuw-os-apps-check-target`; existing local
`CARGO_HOME`/`RUSTUP_HOME` select the pinned project toolchain. Native app
invocation, graphical interaction and cold QEMU integration were **not
performed** in this partial check and remain open with the lead integrator.

### R1 recheck

The regression with two independent SDK sessions fails before the fix with
exit 101: the second BEGIN is incorrectly accepted. After the fix, all 28 SDK
tests and 37 `volume-core` tests pass, including the existing v6 crash suite,
eight new upload-identity trials and counter exhaustion. SDK and core pass
Clippy `-D warnings`; root/SDK format checks, the scalar
`x86_64-unknown-none` SDK library and both native volume programs build offline.
Logs and exact commands: `/tmp/nieuw-os-a03-r1-final/`; the failing check is in
`/tmp/nieuw-os-a03-verification/r1-before.*`.

A separate agent repeated the 28 SDK tests and nine targeted core trials on a
byte-identical source copy: all passed, none skipped. This is a second AI
check of host/wire behaviour, not a human audit or native upload acceptance.
Actual v7 app invocation and native principal-exit/lost-BEGIN cleanup remain
explicitly open.

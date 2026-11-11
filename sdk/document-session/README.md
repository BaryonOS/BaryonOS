# Bounded document editing session

The current volume path is in `nieuw_os_document_session::volume`. It uses
existing streaming volume operations with the v7 upload token for documents
up to 64 KiB, with app-managed buffers, incremental saving and cancellation.
See [contract and integration](../../docs/implementation/VOLUME-DOCUMENT-SESSION.md).
The catalogue path below remains available for existing M6c clients;
its 128-byte limit is not a new product decision for Materiaal.

## Native library over volume v7

The volume `DocumentSession` also provides the following bounded operations,
without an additional endpoint, wire operation, storage format or authority:

| Call | Behaviour |
|---|---|
| `list_filtered(ListFilter, out)` | Reads all pages within one domain clock. `All`, `Uncollected` and `Collection(1..=4)` only select the list. Errors preserve the previous output list. |
| `collection_stat(index)` | Reads the name, presence, revision and request ID of one collection in the granted domain. |
| `update_collection(index, expected, name)` | Creates/renames using revision CAS; an empty name deletes only an empty collection. |
| `set_metadata(expected_revision, tags, kind, collection)` | Changes metadata of the loaded document; `0` means no collection. A valid ACK updates the loaded revision and preserves name and contents. |
| `read_history(expected_revision, selector, out)` | Reads history `1..=2` bound to the current object revision. Verifies all parts and the checksum before changing `out`; also preserves a dirty draft. |
| `read_history_prefix(expected_revision, selector, out)` | Verifies the **entire** historical text in the existing staging buffer, then copies a UTF-8-safe prefix. `HistoryPrefix { copied, total, truncated }` makes the presentation limit explicit. No additional document buffer. |
| `restore_history(expected_revision, selector)` | Restores as a **new** revision, retains the replaced version in history and loads exactly the confirmed new revision. |
| `rename_draft(name)` + explicit `save()` | Changes the local title; only saving publishes the new name together with the contents. |

Mutations require `ReadWrite`, a clean draft, no active upload and no conflict
or uncertain outcome. Reading remains possible with `ReadOnly`; that local
mode grants no capability. Selection, list filters and history previews write
nothing. The existing service enforces CAS, domains, quotas, collection
membership and deletion of empty collections only, among other constraints.
A truncated text preview is therefore not a partially verified version.
The UI must visibly explain `truncated`; the separate restore action still
uses the full version and current revision precondition. Non-UTF-8 history
returns `InvalidUtf8` for text previews, including invalid bytes after the
shown prefix. Existing `read_history` remains an opaque-byte API.

A lost, malformed or untrusted mutation reply closes the connection and may
set `is_uncertain()`; the client never retries automatically.
`CommittedRefreshFailed { revision }` instead means restoration **was**
confirmed but the subsequent read failed. Old draft bytes remain available;
save and repeat restoration are blocked until an explicit successful `open`.
The UI must explain the known commit and read error separately and must not
repeat the restore mutation. A later new connection requires the existing
independent service/grant recovery path.

Integration tests use the actual volume Service and capability engine with a
synthetic host disk. They check filters, metadata, new restore revisions,
dirty/async-save blocking, read-only mode, stale CAS, wrong domain/reply,
timeouts, service crashes, uncertain publication and preservation of draft
and output after a partially failed read, among other cases. This is host
evidence; native UI interaction and disk oracles require a separate QEMU gate.

`nieuw-os-document-session` is an allocation-free `no_std` library with
`forbid(unsafe_code)` over the existing `nieuw_os_catalog_client::Session<T>`.
It manages one existing document and a local draft buffer of at most
**128 bytes**. Object ID, name, existing catalogue protocols and storage
format are preserved. Contents are opaque bytes: this layer introduces no
UTF-8, text, file or disk policy.

## API for G1/Materiaal and other apps

| Call | Behaviour |
|---|---|
| `DocumentSession::new(catalog, EditMode)` | Takes over one existing catalogue session; grants no new authority. |
| `open(object)` | Loads an existing document through actual GET. Failure preserves previous contents. A dirty draft blocks switching. |
| `draft()` | Returns a bounded immutable view. `Draft` may be copied explicitly; getters are `object`, `loaded_revision`, `name`, `contents`. |
| `replace_contents(bytes)` | Replaces local contents, at most 128 bytes; no IPC or storage action. |
| `replace_range(start..end, bytes)` | Inserts, replaces or deletes within byte boundaries. All checks precede mutation. Text/grapheme boundaries are the calling app's responsibility. |
| `is_dirty()` | Compares with the last loaded or confirmed bytes; restoring those bytes makes the draft clean again. |
| `save()` | For a changed draft, sends one PUT with exactly the loaded CAS revision. A valid ACK increments the local revision and makes the draft clean. With no change, nothing is sent. |
| `is_conflicted()` | A Stale reply has superseded the local baseline. The draft remains intact; another save sends no new request until this conflict is explicitly resolved. |
| `is_uncertain()` | A save may already have executed. This state is permanent for this session object; nothing is resent. |
| `discard_changes()` | Restores only local bytes to the last loaded/ACKed state. No GET, PUT, revision refresh or resolution of uncertainty. |
| `reopen()` | Explicit GET for the current object, provided there are no undiscarded changes or uncertain save. |
| `list()` | Reads the catalogue of the granted domain through the same endpoint owner. Preserves draft, loaded revision and conflict status; closed or uncertain sessions send nothing. |
| `close()` | Closes both endpoints through G03; the draft remains readable. No hidden syscall in `Drop`. |

`EditMode::ReadOnly` is purely local app policy that rejects editing/saving.
`ReadWrite` grants no write permission. The kernel-bound endpoint capability
and actual service remain authoritative. A canonical `Access` reply remains
an error and preserves the draft. Revocation of an actual endpoint still
works when the app uses `ReadWrite`.

Keep the `DocumentSession` object in app state so errors from an action
handler do not destroy the draft. For example, a handler can call
`replace_contents` followed by `save` through `&mut DocumentSession<T>`.
An error returns a decision to the app; there is no automatic reload,
merge, retry or overwrite.

### Conflict

On `Error::Conflict`, draft and loaded revision remain unchanged. If desired,
first retain `let retained = *editor.draft().unwrap();`. Let the app explicitly
choose between keeping/comparing and discarding. Only explicit
`discard_changes()` followed by `reopen()` retrieves a new baseline. The app
may then deliberately reapply chosen bytes; the next save uses the newly
loaded revision. This library does not automatically decide to overwrite the
new version with the old draft.

### Uncertainty and disconnected transport

A lost reply after request publication, an invalid response or `Unknown`
must not become an ordinary retry. Local bytes and the old revision remain
preserved; G03 closes the session and G07 blocks further saves.
`discard_changes` does not clear that uncertainty. Retain the draft, close
the old session and use a new catalogue session with newly established grants
only after independent recovery. Load actual storage state before any new
user choice.

A demonstrated EAGAIN timeout before enqueue retains the same draft but does
not claim that a mutation executed. That transport connection is also closed;
there is no automatic reconnect. Local draft editing remains possible while
storage is unavailable.

Missing objects return `NotFound` with the existing revision; this layer does
not silently create an empty document. Creation, renaming, deletion, search
and GUI interaction remain separate app actions through existing catalogue
interfaces.

## Checks and limits

Host tests use the actual capability engine for Message320 IPC, G03 transport,
B01 correlation, catalogue validation, `State::prepare/commit` and complete
`Recovery` over the 32 existing journal records. The host model orders append
WRITE, successful FLUSH, commit and ACK. A lost reply is injected only after
the actual core commit; a new service model reconstructs state solely from
the stored record bytes. Corruption of such a record is rejected by actual
Recovery.

These are **host-model tests**. The library is also built for
`x86_64-unknown-none`. G11 adds a separate opt-in native experiment: existing
M6c user processes open an existing document, edit its 128 content bytes,
save with the loaded revision and reopen. A separate cold VM reads the actual
image back through the same library. The gate requires both
`NOS_DOCUMENT_SESSION=1` and `NOS_SDK_RUNTIME=1`; only the normal M6c case
and recovery are allowed. The existing kernel observer, capability boundaries,
16/3 requests per client and external disk oracle are preserved. Current
executed results and their exact source binding are in `docs/PROJECT-STATE.md`;
compilation or this description is not VM evidence. This deterministic
experiment has no graphical interaction or general app startup. G1/Materiaal
retains separate implementation and acceptance.

```sh
CARGO_TARGET_DIR=/tmp/nieuw-os-g07-target cargo test --manifest-path sdk/Cargo.toml --workspace --locked --offline
CARGO_TARGET_DIR=/tmp/nieuw-os-g07-target cargo clippy --manifest-path sdk/Cargo.toml --workspace --all-targets --locked --offline -- -D warnings
CARGO_TARGET_DIR=/tmp/nieuw-os-g07-target cargo build --manifest-path sdk/Cargo.toml -p nieuw-os-document-session --lib --release --target x86_64-unknown-none --locked --offline
```

No external crate was added: all dependencies are the existing local SDK,
catalogue and, for tests, capability crates. Source/test evidence and the
original library boundary are in `docs/verification/parallel-g07/`.

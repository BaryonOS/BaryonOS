# N01/N02 — encrypted NAS storage and hybrid transport probe

**Executable Node host prototype:** N01 provides real AES-256-GCM encryption, local ciphertext storage and authorization checks within the service API. The original storage demo opens no sockets. N02 adds a separate real TLS 1.3/X25519MLKEM768 probe, on loopback only: see [TRANSPORT.md](TRANSPORT.md). This is not a native Nieuw OS service or a configured home NAS; the transport echo is not yet connected to the store.

This implementation follows the amended assignment of 27 October 2026: a NAS may be at home or with friends/family, with separately permitted direct internet access. The NAS administrator may independently delete their stored copies or withdraw the storage space, without a user key or signature. That administrative authority grants no decryption key. Older texts limited to an own LAN only have been revised for this scoped assignment on the basis of the new project decision; the main agent manages the shared contract documents.

## Rust building blocks for native integration

[NAS-CORE-1](native-core/README.md) adds an allocator-free `no_std` core
for recipient-bound invitations, sessions, quota reservation,
revision conflicts, durable revocation and logical deletion of ciphertext.
[NAS-VOLUME-1](native-volume/README.md) connects that core to the existing
Rust volume client and also cleans up its recoverable version history before
success. The temporary Rust capacity limits are documented with the crate; the
N01 limits below belong exclusively to the Node host prototype.

[NAS-IPC-PROBE-1](native-probe/CONTRACT.md) now provides the local 256-byte
IPC adapter and three separately startable process roles as freestanding
ELF. The contract fixes startup words, endpoints, memory and normal/cold
probe assertions. This package does not itself configure a kernel manifest and
has no QEMU execution evidence yet.

The Rust host probes and bare-metal build are **not a running native NAS**.
Transport identity, kernel IPC, client encryption, automatic sync and
multi-device key lifecycle remain open. [Verification evidence](native-volume/verification/README.md):
P01–P04 **0/5 → 0/5**, no new rubric points.

## Running

From the project root, with Node.js 24:

```sh
node services/nas/demo.mjs
node services/nas/transport-demo.mjs
node --test --test-isolation=none services/nas/tests/*.test.mjs
```

The finite demo creates its own `nieuw-os-n01-*` disposable directory under the system temp directory, writes and reads one synthetic encrypted document, checks the policy boundary and then revokes the storage. It deletes only that self-created directory. The JSON output contains results, not document content, key or capability. The tests likewise use only self-created synthetic directories. They include three real process kills after a confirmed commit, revoke and eviction.

Checked on Node.js **24.22.1**, Linux x86_64. In this development environment `--test-isolation=none` shows the individual `node:test` results; without that option the host presents the test files as groups. The kill probes start separate child processes, also with this test option.

## Modules and interfaces

| Module | Interface and boundary |
|---|---|
| `crypto.mjs` | `createDocumentCipher(namespace)` creates a new client-only 256-bit key. `encrypt({objectId, revision, name, content})` returns a `Buffer` with a ciphertext envelope. `decrypt({objectId, revision}, bytes)` verifies the full binding and tag before plaintext is returned. |
| `crypto.mjs` | `exportRecoveryKey()` is an explicit local export to the caller. `restoreDocumentReader(namespace, key)` restores reading only; the result cannot re-encrypt under a reset nonce history. The NAS does not receive these interfaces or exports. |
| `store.mjs` | `NasStore.createSynthetic()` returns `{store, adminCapability}`; `openSynthetic({directory})` opens only a marked synthetic store. `close()` terminates all temporary handles. |
| `store.mjs` | Administration: `createInvitation(admin, {quotaBytes, lifetimeMs})`, `inspectUsage(admin)`, `revokeNamespace(admin, namespace)`, `evictNamespace(admin, namespace)` and `deleteObject(admin, namespace, objectId)`. No administrative method decrypts data. |
| `store.mjs` | User: `acceptInvitation({namespace, invitation, confirmedPeerIdentity})`, `openSession({namespace, accessCapability})`, `stageUpload(session, envelope)`, `commitUpload(session, upload)`, `cancelUpload`, `readObject`, `listObjects`, `closeSession`. Read results are ciphertext, not document names or content. |
| `policy.mjs` | `createPolicyEngine()` returns `authority`, `check`, `planNasRoute` and `authorizeNasConnection`. The preflight checks permission before socket creation and grants no transport trust. Only the trusted adapter holds `authority`; apps get their own opaque handle/decision interface. No kernel enforcement. |
| `transport.mjs` | Loopback TLS with hybrid key exchange, mTLS/pins and live policy checks; the precise interfaces and evidence boundaries are in TRANSPORT.md. |
| `policy.mjs` | `planNasRoute({app, peerIdentity, transports})` automatically selects an available permitted LAN route, otherwise direct internet to exactly the same peer under existing permission. Result: `{status: 'ready', route: 'lan' \| 'internet', transport, reason}`, or `{status: 'paused', route: null, transport: null, reason}`. |

The NAS storage knows no user name or file name. Namespace IDs, object IDs, version numbers, lengths, status and quotas remain visible as operational metadata. A content name and the content itself are together in the encrypted JSON payload. Lengths and access rhythm are not hidden; traffic padding is not implemented.

## Cryptography and key boundary

AES-256-GCM comes from Node's built-in `node:crypto`/OpenSSL; no external packages or custom crypto primitives have been added. Each new client cipher gets 32 bytes from `randomBytes`. Each encryption gets a 12-byte random nonce. A ledger in the cipher prevents reuse within the full lifetime of that key and limits it to 8.192 encryptions. There is no API that re-imports an existing key as a writable cipher. The NAS also rejects reused nonces, including reserved uploads and after deletion of a single object.

The authenticated context is exactly the UTF-8 JSON array `['nieuw-os/nas-object', 1, 'aes-256-gcm', namespace, objectId, revision]`. A changed namespace, object ID, revision, nonce, ciphertext or 16-byte tag is rejected on decryption. The client must supply the expected object ID and revision itself. The NAS can only check the envelope format; without the user key it cannot determine whether a tag supplied by a malicious client is authentic.

Only the client module holds the decryption key. The store keeps SHA-256 digests of random 256-bit administration/access capabilities; raw capabilities are not in the disk snapshot. Administrator and user session are different authorities. The demo/test process contains both roles to test the interfaces; this proves no process isolation against a debugger, root access or a compromised client. The production setup must keep the client cipher on the user's device, separate from the NAS host.

**Still open:** secure local key storage, offline recovery UX, multi-device key transfer, key rotation and a persistent client ledger for expected revisions. The export is raw secret key data: the caller must store it securely on their own device themselves, never at the NAS, in a log or in Git. Without an export the in-memory key disappears after process loss. Read-only recovery can decrypt existing data; new write access after client loss requires, in N01, an explicit new namespace with a new key. That is not a finished sync/backup experience.

## Revoking, erasing and restarting

`revokeNamespace` makes the namespace permanently inaccessible to the user; existing ciphertext is retained until the administrator deletes it. `evictNamespace` deletes all object copies managed by this store in that namespace and retains only a tombstone with status/epoch/quota. Both operations require only the administration capability. A new invitation gets a new random namespace. There is no automatic retry, restoration of old rights or reuse of a deleted namespace.

Every revoke, eviction and individual admin delete increments the epoch. Existing sessions and in-flight uploads are revoked. An old upload can therefore not still commit after revocation. An individual object delete preserves revision and nonce history; a user with a still-active namespace can upload a subsequent revision only by explicitly opening a new session. To permanently end further uploads the administrator uses revoke/evict.

The store writes one bounded snapshot: exclusive temp creation, write, file `fsync`, rename and directory `fsync`, before it reports success. On a storage error the handle becomes unusable until it is reopened. Reopening validates the existing state and never silently creates an empty authority. Sessions and staged uploads exist only in RAM; after restart they are invalid. Committed ciphertext, invitation consumption, digests, quotas, revisions, nonces and tombstones remain on disk.

An exclusive writer lock prevents two simultaneous service handles. A dead local PID that is encountered is recovered only under a second exclusive `recovery.lock`, after re-reading the writer lock. That recovery lock itself is never automatically stolen. A crash exactly during lock recovery may therefore require a manual offline check: stop/check all involved local service processes and only then remove a leftover `recovery.lock` in the confirmed own synthetic test directory. PID reuse safely results in a blockage. Shared network filesystems, multiple hosts and administrators who manually modify lock files are outside this model.

Under the writer lock the store removes its own leftover `.state-<32 hex>.tmp` snapshots before new mutations and before serving after reopening. A cleanup error blocks the operation; eviction then reports no success. Other files are not cleaned up. This is logical deletion of active service copies and own temporary snapshots. Forensic secure erase, SSD remanence, external snapshots/backups and copies that a NAS owner made themselves outside the service are not demonstrably erased.

The executed process kills test reopening after a confirmed operation. **No physical power-loss test**, no fault injection on every write/rename instruction and no guarantee against a malicious NAS owner who restores an old full snapshot. Such an owner can refuse or delete storage, or offer old ciphertext; the content remains unreadable without the client key. Protection against rollback later requires a trusted client checkpoint/revision strategy. The read API rejects an old object when the caller has a newer expected revision.

## Internet and app policy

- Installing grants no internet, LAN, document, NAS or administration rights.
- System AI cannot receive network rights; internet/LAN/NAS are always denied.
- A browser can explicitly receive internet without general document or NAS rights. A separate document permission applies to exactly one object and read/write mode.
- A NAS service requires an explicitly paired peer identity and a selected local network. LAN is the default permitted route after pairing. Direct internet is permitted only after a separate permission for exactly that peer.
- After pairing once and granting permission for direct internet, `planNasRoute` automatically selects the best permitted available route at every network transition: LAN, otherwise direct internet to the same NAS. Back on the permitted LAN, that route takes priority again. The planner asks for no repeated permission and grants no new rights itself.
- Vendor relay and redirects are refused, even with internet permission. Missing or revoked peer facts produce no fallback.
- Model/update downloaders receive no personal document or NAS capability via this policy interface. Restriction to specific download servers and package/model verification are not yet implemented.

The trusted adapter supplies at most 64 current available transport facts. The planner makes no connection and does not itself test reachability; a vanished connection must be removed from the candidates or invalidated with `forgetTransport`. Without a usable route it explicitly returns `status: 'paused'`, so that the interface can show, for example, “Synchronisatie gepauzeerd” ("Synchronization paused") while local tasks continue. There is no diversion to another NAS, relay or unverified connection. Revocation of internet or peer rights takes effect immediately at the next decision, even when old transport facts still exist. An earlier `ready` outcome is not a persistent capability: the adapter must check current rights at use and later also terminate real active sessions on revocation.

In this host prototype the policy rights live in memory. A production version must store one-time choices securely on the device, restore them after restart and re-check new transport identity; this must not require a series of new permission prompts. The limited nonce budgets and read-only key recovery above are also temporary probe limits. Reliable background operation, usable key recovery and normal daily access are product requirements, not permanent manual workarounds to impose on the user.

`authority.attestVerifiedTransport(...)` is the trusted adapter interface. It returns an opaque handle that cannot be forged from JSON; a client object with `verified: true` is rejected. **This function does not itself perform TLS, certificate validation or identity proof.** N01 policy tests supply an explicit assumption here; the N02 loopback adapter does so only after real TLS validation. The `peerIdentity` in the synthetic store remains a random fixture ID and is not yet bound to this transport pin. Native interface, route, DNS and socket validation and integration between store and transport remain open.

## Bounded input

| Limit | N01 value |
|---|---|
| Document name / content | at most 512 / 16.384 UTF-8 bytes |
| Envelope | at most 140.000 bytes, strict schema |
| Namespace / object ID | random opaque 256-bit namespace / 128-bit hex object ID |
| Namespace quota | 256 to 2.000.000 bytes; default 1.000.000 |
| Namespaces | 16 including persistent tombstones |
| Object IDs | 32 per namespace, also after individual deletes |
| In-flight uploads | 8 per namespace, counted in quota and nonce budget |
| Sessions | 16 per namespace / 128 total |
| Invitation | single-use; default 5 minutes, at most 24 hours |
| Nonces per cipher/namespace | 8.192, after which explicit rotation is required |
| Full snapshot | at most 48.000.000 bytes |

## Evidence and remaining integration

The test set checks encryption/decryption, tag/metadata changes, wrong keys, read-only recovery, secrets outside NAS disk data, tenant boundaries, administrator rights, invitations, quotas, conflict situations, nonce limits, stale uploads, restart, corrupted/symlinked snapshots, orphan cleanup and the internet policy. The route probes cover LAN→internet→LAN with one permission, pause/resume, revocation while old facts exist and rejection of wrong peers, relays, redirects and forged facts. Test commands, current count and versions are recorded by the main agent in the project evidence. N01 unit/policy tests do not prove TLS. The separate N02 handshake probes provide real loopback TLS evidence; they are not QEMU, real-NAS or production-security evidence.

Open: native OS process/capability integration, connection to a real home NAS, authenticated pairing between devices, secure key lifecycle, crash/power-loss coverage, rollback protection, background sync/conflict handling, production transport quotas and an independent human security audit. N02 provides only the bounded real loopback connection. M1–M6/kernel files are not modified by these modules.

## Primary implementation references

Node.js. (2026). *Crypto: Node.js v24.22.1 documentation*. https://nodejs.org/docs/latest-v24.x/api/crypto.html

Node.js. (2026). *File system: Node.js v24.22.1 documentation*. https://nodejs.org/docs/latest-v24.x/api/fs.html

Accessed 27 October 2026. These references describe the standard APIs; the envelope format, authority model and the limits above are own prototype choices, not a security protocol certified by Node.js.

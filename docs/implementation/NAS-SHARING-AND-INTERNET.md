# N01 — internet and personal NAS space

## P05 addition — 28 October 2026

N01/N03 remain the **personal NAS route**. Privacy contract revision 4 also
allows a user to explicitly select a general external storage provider as
an untrusted backend solely for opaque, client-side encrypted objects. That
is a separate P05 route under
[ENCRYPTED-OBJECT-STORAGE.md](ENCRYPTED-OBJECT-STORAGE.md); it does not replace
the NAS, gives the provider no content key and is never a silent fallback.

Date: 27 October 2026. Project decision: internet access, a NAS at the user's home as a personal cloud, the same OS controls for client and NAS, and sharing space with friends/family without being able to read their content. The NAS administrator may unilaterally reclaim the space.

## Status and scope

**N02 addition, 27 October 2026:** the original N01 scope below remains identifiable. The separate [TLS host trial](../../services/nas/TRANSPORT.md) adds actually authenticated hybrid loopback transport and app/generation-bound facts. It provides no native NAS network service and does not yet connect the echo to the store. Current evidence: `docs/verification/pqc-transport/`; formal/native follow-up criteria: [VERIFIABLE-SECURITY-AND-PQC.md](VERIFIABLE-SECURITY-AND-PQC.md).

N01 runs in parallel with M6a. This change does not extend the project's kernel/network driver.

| Layer | Deliverable | Evidence boundary |
|---|---|---|
| Binding product contract | Privacy contract revision 4, ADR-027 plus ADR-035/036, updated architecture/roadmap/instructions | Requirements and selected role allocation |
| Encrypted host service | `services/nas/`: actual client encryption, local ciphertext storage, quotas, invitation, revocation and administration | Node host trial; no native OS service or network server |
| Network policy | Default-deny decision logic with separate app/AI/NAS permissions | Trusted adapter must actually verify identity and connection |
| Materiaal | My NAS, Receive space, Offer space, identity verification, explicit internet and host removal | Browser mock with fictional data; no cryptographic security in the demo |
| Native integration | Network stack, transport authentication, same-OS NAS installation, persistent keys and kernel capabilities | Still to be built and tested separately |

Actually executed test commands, figures and limitations are under `docs/verification/nas/`. No M8, physical-hardware, internet-reachability or production claim.

## 1. User scenarios

### Product standard: privacy without unnecessary interaction

Additional product requirement of 27 October 2026: the connected experience and finish must be convincing alongside Windows/macOS. Privacy is an underlying default property, not a reason to omit internet access, collaboration, automatic resumption or a modern interface. Security and ease of use are joint acceptance criteria.

A user confirms a new NAS identity and selects the permitted connection scope once. The route planner then automatically selects the verified local channel, or already permitted direct internet access to the same peer. No new prompt for every transition, file edit or normal synchronisation. Mismatches, revocation and unauthorised destinations continue to block access. Offline work and resumption must not cause hidden duplicate files or data loss; actual synchronisation/conflict handling is a service still to be built.

Current read-only key recovery, the 8,192-encryption limit, small test quotas and reconfirmation after browser refresh are **temporary trial boundaries**. They are not accepted everyday product behaviour. A product version requires implementation and testing of secure persistent key management, automated rotation, background synchronisation, multi-device access and understandable recovery. A feature does not become product-ready by presenting a security problem as a permanent user limitation.

The source trial demonstrates safe automatic route selection, but no actual internet connection or background synchronisation. Onboarding must explain access, data location and recovery in plain language; advanced network/key settings belong outside the default flow. Compare use through concrete tasks and measured friction; do not claim equivalence with existing platforms without those measurements.

### Setting up a personal NAS

Select the NAS role of the same OS on a second suitable device. Select a storage volume and maximum capacity; show which data could be removed if the volume needed to be reinitialised. This future installation requires a separate instruction before a real disk is erased. Create a local administrative identity. The client discovers a candidate or receives an invitation, compares the identity and selects the desired capacity. Discovery grants no trust.

### Receiving space from family or friends

The host creates a single-use invitation offering space. The recipient checks the identity and knowingly accepts that the host can delete the NAS copy. The client generates its own content key locally; neither the invitation nor the administrator receives that key. Pairing grants no access to other users or device administration.

The connection remains local by default. Use from another home or while travelling requires separate permission for internet access to that paired identity. A changed identity interrupts the connection even if the hostname or address remains the same.

### Offering and reclaiming space

For each guest, the host sees a recognisable name or local alias, used space, quota and status. No guest document names, previews or content. 'Revoke access and delete NAS copy' requires only clear confirmation from the administrator. An offline guest, lost guest key, encrypted object or guest objection does not block that administrative action.

After revocation, old sessions and ongoing uploads no longer work. The space remains revoked after restart. An old client must not recreate the namespace. A new invitation is a new offer with a new namespace.

## 2. Trust boundaries

For confidentiality, the NAS is treated as untrusted storage. The user's client and its verified distribution chain retain the key. The NAS must not be able to impose a client update or script that receives content keys.

The administrative capability is separate from the content key. A host can read ciphertext on disk, but that is not permission to read documents. The host can always end availability and observe necessary metadata. A local device compromise, lost key or deliberate replacement of client code is a different threat.

The browser demo uses no actual keys, NAS or network. Its 'encrypted' display describes the product contract and is marked as a simulation.

## 3. Storage and revocation invariants

1. Only authorised active namespaces may publish/read/delete their own objects.
2. NAS storage contains only the ciphertext envelope and necessary opaque metadata; document names are encrypted too.
3. Authenticated encryption binds schema, namespace, object and version. Verify fully before returning plaintext.
4. Content keys are not stored in server state, invitations, sessions, administration or logs.
5. Administrative revocation requires no user key, user signature or user consent.
6. Revocation and upload commit are evaluated under the same serialisation boundary. The durable revoked state precedes cleanup.
7. Staging/sessions/tickets bind to the current namespace generation. Old authority fails even after restart.
8. Deleted namespaces are not recreated by upload/resume/synchronisation. New offer = new random namespace.
9. Quotas count ciphertext and reservations; metadata, object count and numbers of sessions/invitations/uploads have separate limits.
10. Guest quotas must not prevent administrative revocation. An actual storage error produces no incorrect durability acknowledgement.
11. Purge is repeatable; administrative status and actual cleanup status remain distinguishable on errors.
12. Deletion on this NAS does not erase user copies on other devices and promises neither physical secure erasure nor destruction of uncontrolled backups.

The host trial must document its specific atomic-storage and single-writer assumptions. A successful process-restart test does not prove recovery from physical power loss on every disk.

## 4. Network policy

| Principal | Default | Explicit expansion |
|---|---|---|
| Local AI inference | No LAN/internet or upload helper | No cloud fallback |
| New third-party app | Own app storage, no network/documents/NAS | Separate app and destination grants |
| Browser | No general document/NAS permission | Internet use within the assigned scope |
| NAS synchronisation | No unknown peer; paired namespace only | Checked LAN, then optionally internet to the same identity |
| Update/model downloader | No personal context or NAS keys | Bounded download destination and verified artifacts |
| NAS administration | Local administrative capability | Remote administration is a separate function still to be designed |

A network grant is not a document grant. Combined file and internet permissions are made visible; an app that legitimately holds both can export permitted content. A textual promise cannot eliminate that risk.

The future adapter checks identity, TLS 1.3, destination and actual route before supplying an authorised principal/connection. No mutations through TLS 0-RTT. An external request must not fabricate a route or authentication using booleans. Reauthentication after key rotation or session revocation is mandatory. No automatically open administration port, UPnP/port forwarding or relay fallback.

## 5. Cryptographic implementation

The host trial uses Node.js's maintained `node:crypto` with AES-256-GCM, cryptographic randomness and authenticated associated data. The exact key/nonce lifecycle is documented with the implementation; random nonces must not be presented as indefinitely reusable without a bound. No hardcoded production keys or custom cipher/hash constructions.

TLS protects transport; client encryption also protects document content from the NAS host. These functions do not replace each other. Key transfer to a second owned device, usable offline recovery and protection against server rollback require separate integration. An old encrypted object can still have a valid cryptographic tag; the client must maintain authenticated version history before making freshness claims.

## 6. Native work still required

- Restricted network service with checked socket/destination capabilities, parser bounds and tests involving compromised apps.
- TLS/client authentication through a maintained implementation, key pinning, key rotation, no 0-RTT mutations.
- Actual NAS role in Baryon OS, administrative storage on a recoverable filesystem and multi-user quotas.
- Native client-key storage and recovery without key escrow; secure pairing of additional devices.
- Direct reachability, IPv4/IPv6, DNS rebinding, NAT/CGNAT, network transitions and an explicitly selected network route. 'Plug and play everywhere' is not proven.
- Persistent revocation and concurrency across multiple processes/devices; quotas and disk-full reserves; crash/fault injection on the final storage layer.
- Fuzzing, key/update-chain review and a bounded independent human security assessment before use with sensitive data.
- Task, keyboard and screen-reader trials of the same onboarding on client and NAS.

The next kernel milestone remains the ongoing M6a final verification. N01 connects to it when storage and network capabilities permit.

## Sources and decision status

- OpenJS Foundation. (2026). *Crypto: Node.js v24.22.1 documentation*. https://nodejs.org/docs/latest-v24.x/api/crypto.html
- Rescorla, E. (2018). *The Transport Layer Security (TLS) protocol version 1.3* (RFC 8446). IETF. https://www.rfc-editor.org/rfc/rfc8446.html
- Rose, S., Borchert, O., Mitchell, S., & Connelly, S. (2020). *Zero trust architecture* (NIST SP 800-207). NIST. https://doi.org/10.6028/NIST.SP.800-207

Primary sources checked 27 October 2026. The functional role decision and unilateral host deletion right follow from the explicitly recorded project instruction. These sources do not approve the implementation.

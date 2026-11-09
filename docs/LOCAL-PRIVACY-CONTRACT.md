<a id="bindend-privacycontract-internet-persoonlijke-nas-en-gedeelde-opslag"></a>

# Binding privacy contract: internet, personal NAS and shared storage
Revision 4, 28 October 2026. This explicit project decision supersedes both the earlier restriction to a personally owned NAS within the user's LAN and the absolute prohibition on vendor cloud services. The local AI requirement remains in force. External storage providers are strictly optional and may be used as an untrusted storage layer for opaque objects encrypted on the client side; they do not become key holders, OS trust anchors or silent fallbacks.

**Evidence status:** these are binding product and integration requirements. `services/nas/` contains N01 with actual host encryption/storage management and the separate N02 loopback experiment with genuinely authenticated hybrid TLS. `ui/prototype/` contains simulated product flows. The native network stack, NAS distribution, pairing between real devices, external provider adapters, key service and kernel enforcement remain open. Scope: [NAS-SHARING-AND-INTERNET.md](implementation/NAS-SHARING-AND-INTERNET.md), [ENCRYPTED-OBJECT-STORAGE.md](implementation/ENCRYPTED-OBJECT-STORAGE.md) and the [verification/PQC supplement ADR-028](implementation/VERIFIABLE-SECURITY-AND-PQC.md). Existing NAS administrator rights remain in force; an external provider receives no comparable administrative or decryption authority over local data.

<a id="1-internet-en-gegevenslocatie"></a>

## 1. Internet and data location

Baryon OS must support internet access. Network location or ownership does not create automatic trust: authentication, authorization and isolation also apply at home (Rose et al., 2020). Internet access does not authorize an app or service to read documents, NAS data or keys.

Personal data remains local or may optionally be sent, **encrypted client-side/end-to-end**, to:

- a personally owned, explicitly paired NAS;
- storage space on an explicitly paired NAS belonging to friends or family;
- an explicitly selected external storage provider that receives only opaque ciphertext objects.

A NAS may be on the same LAN or in someone else's home. LAN is the default route for personal NAS storage. Internet access requires separate permission for the paired NAS identity and an actually authenticated transport. An external provider is linked separately and may require its own account, but Baryon OS treats that provider as untrusted storage and gives it no content key. No implicit expansion of rights when the network changes, mandatory vendor cloud, provider or administrator key escrow, silent relay/fallback, or external OS telemetry is permitted.

Core local tasks continue to work without a NAS or internet. An unreachable or revoked storage location is visibly paused; files are not forwarded to another host. Local copies remain the user's property. NAS inference is not permitted as an alternative to local system AI.

Package/OS updates and model downloads receive separate, restricted download paths without personal context or NAS credentials. The destination sees necessary connection metadata such as the source IP. This is not an anonymity guarantee.

<a id="2-eenvoudige-persoonlijke-cloud-met-gescheiden-bevoegdheden"></a>

## 2. Simple personal cloud with separate authority

Privacy and ease of use are joint product requirements: pair once and select the scope, then connect and synchronize automatically within that authority. Additional confirmation is intended for a new trust relationship, expanded permission or consequential action. Everyday use must not require system administration knowledge; temporary limitations of host/browser experiments are not permanent product choices. See DESIGN-AMBITION.md for the concrete task criteria.

The same OS product is to have a client role and a NAS role. The intended interaction is: select the storage role, allocate space, pair through a one-time invitation and verify identity. No mandatory vendor account. 'Almost plug & play' is a product goal, not evidence that arbitrary routers/NAT/CGNAT are directly reachable without further configuration.

| Actor | Permitted | Not automatically permitted |
|---|---|---|
| User/client | Encrypt/decrypt own content; use own paired namespace; manage own key recovery | Other namespaces, general NAS administrator rights or networking for all apps |
| NAS administrator | Invite, set quotas, revoke access, delete guest copies and staging stored on the administrator's NAS without the guest's permission | Guest content, document names, decryption keys or key escrow |
| Sync/transport service | Transport permitted encrypted objects to the paired identity | Decrypt arbitrary documents or select a different provider |
| OS/model file vendor | Supply selected update/model files through a separate path | Personal plaintext storage, mandatory OS account linking, recovery keys or telemetry |
| Explicitly selected external storage provider | Store/transport opaque encrypted objects within the assigned provider namespace | Document plaintext, sensitive manifest content, Baryon OS content keys, local search index, AI context or rights to other providers/apps |

<a id="21-koppeling-en-netwerk"></a>

### 2.1 Pairing and networking

- Discovery, a hostname, a private IP or the same Wi-Fi network do not constitute authentication.
- Pairing binds a one-time, expiring invitation to the NAS identity, namespace and agreed capacity. A display name is not an authorization ID.
- Verify identity using QR or an already trusted channel. A short comparison code alone must not become a reusable password.
- The native transport adapter must use a maintained standard protocol for hybrid post-quantum key exchange, with actual peer/client authentication and no silent classical fallback. N02 investigates TLS 1.3/X25519MLKEM768 according to RFC 10024 on loopback; the Ed25519 test authentication remains classical. Mutations do not use TLS 0-RTT because of replay risks. Bind repeated requests to operation, identity and generation; bind existing transport authority to the app and revocation generation.
- The trusted adapter supplies network/identity facts; a request must not authenticate itself with a boolean such as `verified: true`.
- Internet permission is bound to the peer. An identity mismatch, redirect or DNS change to an unauthorized peer produces an error, not a fallback.
- No automatic UPnP/port forwarding, open administration port or dependency on a commercial relay as a side effect. Direct reachability or a separately and explicitly selected network route under the user's control must first be demonstrated.

<a id="22-inhoud-blijft-onder-de-sleutel-van-de-gebruiker"></a>

### 2.2 Content remains under the user's key

- Encrypt content, document names and sensitive manifests/metadata before upload. Use maintained authenticated-encryption implementations; no homemade cryptographic primitives.
- Cryptographically bind the schema, namespace, object and version to the ciphertext. An incorrect key, modified tag/data/header or wrong namespace must not yield accepted plaintext.
- Only user devices hold content keys; a NAS administrator reset grants no decryption right. The host holds no hidden recovery key.
- Key recovery uses the user's own offline recovery copy or trusted device. If all key copies are lost, content may be unrecoverable. A new client without trusted version history receives no unproven rollback guarantee.
- The client that handles keys must come from the trusted OS distribution chain. A NAS must not supply arbitrary client code that subsequently handles user keys.
- A NAS or external provider sees necessary storage/connection metadata within its role: for example, allocated space or account usage, opaque object IDs and activity/connection details. Encryption does not automatically hide file sizes, counts or timing.
- Encryption protects content, not availability. A compromised user device or malicious client update falls outside a guarantee that the NAS administrator cannot read the content.

<a id="23-beheerder-kan-ruimte-altijd-terugnemen"></a>

### 2.3 The administrator can always reclaim space

The administrator does **not need the guest's permission, signature, connection or decryption key** to delete the NAS copy. Quota management, blocking and deletion are separate administrative capabilities.

Revocation first durably blocks further access and publication of uploads; managed ciphertext, unfinished uploads and managed recovery copies are then cleaned up. An acknowledgement may describe only the step actually completed. An I/O error must not produce a false claim of successful deletion.

Old sessions, tickets and upload commits are rejected. Deleted namespaces remain recorded as revoked; synchronization must not silently recreate them. A new invitation receives a new namespace. Reservations for ongoing uploads count toward quotas. A full guest quota does not prevent the administrative action.

This deletion right concerns copies on the NAS in question. It grants no right to erase local files or copies on other devices. The host cannot guarantee that a malicious administrator has retained no ciphertext backup anywhere; physical secure erase and cryptographic erasure are separate properties.

Before pairing, the recipient is shown that the administrator can terminate storage, that content is encrypted against the administrator, and that a separate copy/backup remains necessary. Sync is not a backup in its own right. Protected versions may resist ordinary sync credentials, but must not permanently deprive the hardware owner of control over their space.

<a id="24-externe-provider-als-opaque-opslaglaag"></a>

## 2.4 External provider as an opaque storage layer

A general cloud storage service may be linked as an untrusted storage backend
only after an explicit user choice. This does not change the trust model:
the provider is not part of the Security Floor and receives no content key.

- Encrypt content, document names and sensitive manifests **before** the provider adapter receives them.
- Use a project-specific, versionable object/container format, but no invented cryptographic primitive; see [ENCRYPTED-OBJECT-STORAGE.md](implementation/ENCRYPTED-OBJECT-STORAGE.md).
- Provider credentials and content keys remain separate. Compromise of the provider account alone must not yield plaintext.
- The provider adapter receives only the intended ciphertext namespace and network destination, not automatic access to local documents, search indexes or AI context.
- A provider may delete, block or roll back ciphertext. Encryption is therefore no backup or availability guarantee; independent recovery copies and rollback detection remain necessary.
- The user may select multiple storage locations. Baryon OS never silently switches to another provider when the selected storage fails.
- Claims such as “Microsoft/Google can absolutely never crack this” are not permitted. The testable claim is that the provider does not possess the decryption key and that checked ciphertext without such a key does not yield accepted plaintext under the documented threat model.
- Account, IP, timing, volume and other necessary provider metadata may remain visible; client-side encryption is not an anonymity layer.

<a id="3-ai-uitsluitend-op-het-apparaat"></a>

## 3. AI exclusively on the device
Inference runs locally on a CPU/GPU/NPU supported on that device. Personal prompts, embeddings, search indexes, chat history, results and inference logs remain local during use. The user may keep a client-side encrypted backup on an explicitly paired personal NAS, space received from friends/family, or an explicitly selected external provider according to ENCRYPTED-OBJECT-STORAGE.md. A NAS or provider does not perform system inference and receives no plaintext AI context as part of this backup layer.

- The inference service receives no direct LAN/internet access.
- A missing model or insufficient RAM/compute capacity results in an error, queue, smaller local model or disabled feature; never external inference.
- A separate model downloader retrieves selected model files without access to prompts, documents, indexes or NAS credentials. Pin provenance/version/integrity and check the license.
- Treat model files and converters as untrusted: do not execute arbitrary model code/installation scripts in the inference environment; use a controlled format and bounded parser.
- AI receives only explicitly selected files/context. Actions pass through a restricted local broker with task-specific capabilities. The broker must not be a disguised general-purpose shell or upload service.
- Network access remains denied even when models or tools fail. Test subprocesses, delegated services and local sockets as possible bypasses.
- Local AI is optional and removable; the OS must not depend on it.

A third-party program using online services does not automatically become part of the built-in system AI. It receives only explicitly granted app permissions; the OS does not offer such a service as an alternative to local system AI.

<a id="4-open-app-ecosysteem-met-expliciete-grenzen"></a>

## 4. Open app ecosystem with explicit boundaries
Programs may come from multiple sources; no mandatory central app store. 'Downloadable' does not automatically mean 'compatible with this new OS'. The native API/ABI and any compatibility layers remain separate development tasks.

Installation grants no general authorization. By default: own app storage, no access to other apps, personal folders, NAS, internet, camera, microphone or administrative functions. Grant separate, visible and revocable permissions for:

- selected files and concrete actions (read, write, export);
- a named NAS dataset or a system portal;
- LAN and internet separately, with bounded destinations where feasible;
- devices and sensitive system operations.

A signature demonstrates provenance under the relevant key, not the trustworthiness of the app. Package installation must not automatically execute unrestricted administrative scripts. Compatibility layers fall within the same isolation model.

**Crucial evidence boundary:** an app allowed to read a file and contact a server may be able to send that content to the server. The OS cannot simultaneously grant general read/network freedom and promise that authorized data never leaves. Display combined permissions and, for sensitive data, prefer processing without network access, with a controlled sharing/export portal. A browser may need internet access without automatically receiving access to the entire home directory or NAS.

Revocation stops future use through the controlled boundary, but does not erase a copy the app was previously allowed to make. Also test data leaks through intermediary services, the clipboard, shared storage, logs and error messages. Complete information-flow security requires more than a few switches.


<a id="41-security-floor-en-uitzetbare-privacyuitleg"></a>

### 4.1 Security Floor and optional privacy explanations

Privacy choices and system security are different layers. For example, a user may
allow an app internet access, microphone access, cloud sync or voluntary telemetry.
This expands only the corresponding explicit capability and does not disable
kernel/process isolation, memory protection, key isolation,
package/update authenticity or protection of unshared data.

The UI distinguishes a necessary access decision from supplementary
educational privacy explanations. The user may disable these explanations
globally or per app. “Privacyuitleg uit” (“Privacy explanations off”) never means
“alles toestaan” (“allow everything”). Existing grants remain unchanged and new
permissions are granted or denied separately. A grant that is already valid need
not be requested again for every normal repetition.

<a id="5-acceptatietests"></a>

## 5. Acceptance tests

Existing LP identifiers are retained; LP-03 has been explicitly revised for peer-bound internet access.

| ID | Scenario | Required result |
|---|---|---|
| LP-01 | NAS unreachable; internet available | Local tasks work; no other provider/relay as fallback |
| LP-02 | Another device claims the NAS name/IP | No data without the paired, verified identity |
| LP-03 | Network switches from LAN to internet | No sync without explicit internet permission for the same verified peer |
| LP-04 | System AI; internet off | Supported local task works with the installed model |
| LP-05 | AI attempts a socket, subprocess or helper upload | Direct and delegated network access denied |
| LP-06 | RAM or model missing | Explicit local limitation, no cloud inference |
| LP-07 | Model downloader requests personal context | Documents, history, indexes and NAS keys denied |
| LP-08 | New app without grants | No NAS/LAN/internet or general document access |
| LP-09 | App has only internet permission | No personal document through a file, NAS or helper path |
| LP-10 | Permission revoked, old handle retained | Future operations denied; earlier external copies noted separately |
| LP-11 | Sync credentials compromised | Separate recovery copy remains usable; administrator may still erase their own NAS space |
| LP-12 | Modified model/package/update | No execution without successful authenticity/integrity verification |
| LP-13 | Clean installation and periodic services | Only documented necessary connections; no OS telemetry |
| LP-14 | Malicious NAS responses/metadata | Bounded errors, no code execution or escape from authority boundaries |
| LP-15 | NAS state and administrator credentials obtained | No document names/plaintext/user keys; necessary metadata made explicit |
| LP-16 | Administrator deletes space; guest offline/refuses/has lost key | Revoke and delete without guest consent or decryption key |
| LP-17 | Old session/upload after eviction or restart | No further publication or namespace revival; new invite = new namespace |
| LP-18 | Multiple uploads, full quota, abort/retry | Reservations bounded and correctly released; administrator not blocked by guest quota |
| LP-19 | Invite reused/expired/wrong peer | Pairing denied; no automatic grant of trust |
| LP-20 | NAS administrator password reset | No new decryption authority over guest data |
| LP-21 | Administration screen | Only quota/usage/identity/status; no guest document list or content |
| LP-22 | Ciphertext/header/object/version tampered with | Authentication error; no partially accepted plaintext |
| LP-23 | Internet enabled for browser, not for AI or NAS | Only explicit app destination allowed; no permission inheritance |
| LP-24 | Client and NAS roles on the same OS | Authenticated pairing/receiving/offering with understandable error and recovery paths |
| LP-25 | Crash during revoke/purge or actual disk error | Old authority does not revive; no unjustified durability or deletion claim |
| LP-26 | Only provider credentials stolen | No document plaintext, sensitive manifest content or Baryon OS content key available |
| LP-27 | Provider object/tag/header/chunk/revision tampered with | Authentication/rollback error according to the selected threat model; no partial plaintext |
| LP-28 | External provider offline/deletes data | Local operation and explicit recovery status; no silent provider fallback or false backup claim |
| LP-29 | Privacy explanations disabled globally/per app | No educational popup; capability set remains identical in bytes/semantics and new access remains separate |
| LP-30 | App granted internet and voluntary telemetry, no document grant | Telemetry path may use the granted scope but receives no document/NAS/key capability |
| LP-31 | App granted both document and internet access after a deliberate choice | UI shows the combined scope for exfiltration; other documents/apps/secrets remain protected |

Host tests, browser mocks and native OS/network tests are reported separately. A host test or policy boolean does not prove secure networking on the project's own kernel. Record the version, network setup, observation point and missing coverage.

<a id="6-veiligheidsdoel-en-bewijsgrenzen"></a>

## 6. Security objective and evidence boundaries

The objective is least privilege, damage containment, controlled connections and recoverability. The **Security Floor** remains in force when a user deliberately chooses less privacy: a broader data or network grant does not automatically confer more system authority. Internet access introduces a reachable attack surface; there is no absolute guarantee of being 'unhackable'. Authentication/encryption do not replace parser bounds, kernel isolation, secure updates, maintenance or physical hardware investigations.

Client-side encryption of external storage does not mean a provider can observe or delete nothing. Without a provider-held decryption key, content must remain confidential within the tested threat model; absolute unbreakability, anonymity and availability are not claimed.

<a id="7-ontwikkelmiddelen-zijn-geen-os-runtime"></a>

## 7. Development tools are not the OS runtime

Codex, the Git host and the Node development host are build/test tools. They do not become mandatory services of the installed OS. Use synthetic test data. This change configures no actual NAS, router, external account or production key.

<a id="primaire-bronnen"></a>

## Primary sources

- OpenJS Foundation. (2026). *Crypto: Node.js v24.22.1 documentation*. https://nodejs.org/docs/latest-v24.x/api/crypto.html — maintained crypto API, randomness, AAD and authenticated decryption; accessed 27 October 2026.
- Rescorla, E. (2018). *The Transport Layer Security (TLS) protocol version 1.3* (RFC 8446). IETF. https://www.rfc-editor.org/rfc/rfc8446.html — transport and 0-RTT/replay; accessed 27 October 2026.
- Rose, S., Borchert, O., Mitchell, S., & Connelly, S. (2020). *Zero trust architecture* (NIST SP 800-207). NIST. https://doi.org/10.6028/NIST.SP.800-207

Quotas, role allocation and the administrator's deletion right are project decisions based on the recorded assignment; the sources do not certify this project.

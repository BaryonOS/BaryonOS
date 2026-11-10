# Encrypted object storage and Security Floor

**Decision date:** 28 October 2026.

**Status:** binding architecture and product requirement; **not yet natively implemented or accepted**.

**Scope:** local storage, personal NAS, optionally selected external storage providers, key management, synchronisation and privacy/security boundaries.

This document supplements [LOCAL-PRIVACY-CONTRACT.md](../LOCAL-PRIVACY-CONTRACT.md),
[VERIFIABLE-SECURITY-AND-PQC.md](VERIFIABLE-SECURITY-AND-PQC.md) and
[PRODUCT-SPEC.md](PRODUCT-SPEC.md). It does not turn the custom Rust microkernel
into an existing OS kernel and introduces no custom cryptographic primitives.

## 1. Product principle: privacy is configurable, the Security Floor is not

A user may deliberately choose less privacy for functionality, for example
by giving a game internet access, connecting an external cloud service or
allowing telemetry from a third-party app. Such a choice expands only the
explicitly granted data or service authority.

It does **not** automatically disable:

- kernel/process isolation;
- capability checking and rights attenuation;
- separation between apps and services;
- memory protection and W^X/NX;
- protection of unshared files and secrets;
- key isolation;
- authenticity of packages and updates;
- DMA/IOMMU policy once implemented on the supported hardware profile;
- recovery and rollback boundaries.

**Invariant:** consent expands functionality, not trust. Internet permission
does not give an app any document, key, camera, microphone, NAS, process or
kernel right that has not been granted separately.

Privacy explanations and actual authority are different settings. Users may
disable educational privacy explanations. Doing so must never implicitly
expand rights. A capability not yet granted remains a separate decision or
preselected rule.

## 2. Storage models

At product level, Baryon OS supports three storage classes:

1. **Local:** plaintext is available only within the authorised local OS
   boundary; storage encryption and key management follow the native security
   architecture.
2. **Personal/trusted NAS:** content and sensitive metadata are end-to-end
   encrypted before transmission; the NAS administrator holds no content key.
3. **Untrusted external object storage:** an explicitly selected service such
   as a general cloud-storage provider may store only opaque ciphertext
   objects. The provider is storage transport, not the trust anchor for content.

An external provider is **never** a silent fallback. The user selects it
explicitly and sees that account, network, volume, timing and other necessary
provider metadata may fall outside content encryption.

The provider may require authentication for its own account. Baryon OS must
not present this as granting the provider decryption rights or OS-wide access.

## 3. Threat model for external storage

The design must assume that an external provider or an attacker with access
to provider storage can:

- read and copy all stored bytes;
- delete, rearrange, duplicate or roll back files/objects;
- observe size, timing, account and connection metadata insofar as these are
  not separately concealed;
- retain old ciphertext indefinitely;
- supply no trusted code or key to the Baryon OS client.

Under this model, the provider must obtain no accepted plaintext, document
name, sensitive manifest content or content key without user keys.

This is not a claim that cryptography is absolutely “unbreakable”. The
guarantee applies under the assumptions of the selected standard algorithms,
correct implementation, sufficient entropy, correct key protection and an
uncompromised user endpoint.

Encryption protects **confidentiality and authenticity**, not availability.
A provider can delete ciphertext or block access. Independent local/backup
copies, recovery and version/rollback detection are therefore separate
requirements.

## 4. Custom object/container format, no custom cryptography

Baryon OS may define a custom, versionable **Encrypted Object Format**, but
uses only maintained, publicly specified and reviewed cryptographic components.

Introduce no custom cipher, hash, KDF, KEM, signature algorithm or ad hoc
hybrid protocol.

Conceptual format (original notation retained). English translation of the
Dutch labels: minimal public header; document name; collection/relationships;
sensitive timestamps/metadata; revision.

```text
SecureObject vN
├── minimale publieke header
│   ├── format_version
│   ├── cipher_suite_id
│   ├── opaque_object_id
│   └── padded_size_class / chunking metadata
├── encrypted + authenticated manifest
│   ├── documentnaam
│   ├── type
│   ├── collectie/relaties
│   ├── gevoelige timestamps/metadata
│   ├── revisie
│   └── key/reference metadata
├── authenticated encrypted chunk 0
├── authenticated encrypted chunk 1
└── ...
```

The public header contains only what is necessary for safe parsing,
synchronisation and migration. Everything that need not be public belongs
in the encrypted manifest.

### 4.1 Binding and integrity

Authenticated encryption must cryptographically bind at least the following
context:

- format and suite version;
- namespace/account context;
- opaque object ID;
- revision/generation;
- chunk index and total length;
- relevant schema/type version.

A chunk from object A must not be silently accepted as a valid chunk of
object B. An incorrect key, tag, version, namespace, order or length yields
no partially accepted plaintext.

The concrete AEAD suite is fixed for native P04/B02 only after dependency,
platform and implementation review. AES-256-GCM-SIV (RFC 8452) is a candidate
because of nonce-misuse resistance; the architecture does not yet mandate
that algorithm irrevocably.

## 5. Key hierarchy

Target model:

- each object receives a random data-encryption key (DEK);
- ordinary apps receive no exportable master key;
- a separate Key Service performs wrap/unwrap/decrypt operations within
  capability boundaries;
- root/device keys use hardware protection such as TPM/secure hardware where
  a supported profile reliably provides it;
- per-object keys can be rotated, shared or revoked separately without
  exposing the entire storage namespace as plaintext.

An external provider holds no master key, object key or recovery key.

Password-based recovery uses a maintained password KDF. Argon2id (RFC 9106)
is the preferred candidate; exact parameters are measured per supported
hardware profile and stored in a versionable form. A password is not the
preferred means of storing a directly used master key.

Recovery may use an offline recovery secret and/or an already trusted device.
Loss of all valid keys may deliberately be unrecoverable; there will be no
hidden vendor or provider escrow.

## 6. Crypto agility and post-quantum route

The object format hard-codes no algorithm without a suite ID and migration
path. Each suite has:

- an explicit version;
- algorithms and parameters;
- canonical serialisation;
- key/nonce requirements;
- migration and deprecation policy.

Post-quantum algorithms such as ML-KEM (FIPS 203) belong in key exchange and
sharing where the threat model requires them; they do not automatically
replace symmetric authenticated encryption of bulk data. Hybrid transport
remains separately specified in VERIFIABLE-SECURITY-AND-PQC.md.

New suites are not activated silently. Migration creates new ciphertext
under the new suite and preserves an auditable recovery route until the old
version can be safely phased out.

## 7. Metadata and privacy profiles

Content encryption does not automatically hide:

- account identity;
- source IP and provider logs;
- object count;
- object size;
- upload/download timing;
- traffic volume.

Baryon OS must show this honestly and must not derive an anonymity claim from
client-side encryption.

For more sensitive use, an optional privacy profile may investigate additional
mitigations, such as:

- random opaque object names;
- size-class padding;
- encrypted directory/manifest structures;
- batched/delayed synchronisation where useful;
- separation between provider credentials and content keys.

Padding and batching cost storage, bandwidth and/or latency and are therefore
not a cost-free universal default without measurement.

## 8. Multiple providers and provider independence

The object layer sits above the concrete provider adapter. Where provider
limits permit, the same encrypted object must be capable of storage on:

- a personal NAS;
- a trusted third party's NAS;
- generic WebDAV/S3-like storage;
- explicitly connected general cloud-storage providers.

Provider adapters receive only capabilities for their assigned ciphertext
namespace and network destination. They need not see document content, the
local search index or AI context.

The design must be capable of supporting redundant storage of the same
encrypted data at multiple providers without giving any of them plaintext.
This is a future availability feature, not evidence of an already working
multi-cloud system.

## 9. Rollback, deletion and recovery

An attacker can restore old valid ciphertext. AEAD tags alone are therefore
insufficient to guarantee the current version.

The design must have trusted, monotonic or otherwise rollback-resistant
version anchoring for the relevant threat-model scope. Test at least:

- replay of an old object version;
- swapping chunks between objects;
- truncation;
- duplication;
- partial upload;
- provider deletion;
- provider rollback;
- local crash during key rotation;
- lost device;
- lost provider credentials.

“Delete from the cloud” means only that the current provider no longer offers
an accessible copy. The OS must not promise that a provider has never retained
historical ciphertext. Cryptographic erasure means destroying/revoking the
relevant keys within the guaranteed scope; this does not replace a claim about
provider retention.

## 10. App and network boundary

The sync service holds no general document capability for all apps. The
object/Key Service and provider adapter remain separate principals.

Example (original notation retained). English translation: Editor → Object
Service with capability to read/write object A; Object Service → Key Service
with a bounded crypto operation; Sync Service → Provider Adapter with
capability to ciphertext A; Provider Adapter → network with capability to
provider X and namespace Y.

```text
Bewerker ──cap: object A lezen/schrijven──> Object Service
Object Service ──begrensde crypto-operatie──> Key Service
Sync Service ──cap: ciphertext A──> Provider Adapter
Provider Adapter ──cap: provider X + namespace Y──> netwerk
```

A third-party app allowed both to read plaintext and to use unrestricted
internet can transmit permitted content itself. Baryon OS must make that
combined authority understandable, but cannot promise absolute non-exfiltration
after the user deliberately grants both rights.

## 11. Privacy explanations can be disabled; rights remain explicit

The system UI distinguishes:

1. **Security/capability prompt:** a necessary decision about access not yet
   granted;
2. **Privacy explanation:** additional accessible explanation, examples and
   a critical question about consequences.

Users can disable privacy explanations globally or per app. This changes no
existing grants and issues no new grants. An already valid grant is not
requested again for every ordinary action.

During a game or other focus mode, non-urgent educational notifications are
not imposed mid-activity. A new sensitive feature activated by the user may
still present a compact, necessary grant request.

## 12. Acceptance before a product claim

Minimum future gates for this architecture:

- **EO-01:** provider storage contains no plaintext document name, content or
  content key in a controlled fixture.
- **EO-02:** modified ciphertext/tag/header/chunk order is rejected.
- **EO-03:** object/namespace/revision swap is rejected.
- **EO-04:** provider deletion or outage produces no silent alternative
  provider or data-loss claim.
- **EO-05:** provider credentials alone grant no decryption right.
- **EO-06:** loss of a provider account does not silently affect an independent
  local/backup recovery route.
- **EO-07:** an app with only network permission cannot read a protected
  document through the controlled OS boundary.
- **EO-08:** an app with only document permission cannot use a provider
  adapter/network.
- **EO-09:** disabling privacy explanations changes no capabilities.
- **EO-10:** key rotation and crash paths do not return old, new or mixed
  plaintext as silent success.
- **EO-11:** rollback/replay within the selected threat-model scope is detected.
- **EO-12:** adding/revoking devices changes only the intended key/object scope.
- **EO-13:** a provider adapter cannot use a second unauthorised namespace or
  provider.
- **EO-14:** metadata that remains visible is documented and discoverable in
  language users understand.
- **EO-15:** independent crypto/integration review of the actual selected suite
  and implementation before a broad security claim.

Host prototypes, browser mocks and formats on paper do not count as this
native acceptance.

## 13. Product experience: security without old-fashioned UX

Security and privacy boundaries must not result in an administration console
as the everyday interface. Provider selection, key management, recovery and
grants receive a high-quality, consistent Materiaal experience with:

- ordinary human terms as the primary layer;
- technical details on request;
- one consistent interaction language;
- fast resumption within already granted rights;
- visible location and recovery status without persistent dashboard noise;
- accessible controls and a keyboard route;
- no imitation of Windows/macOS/Office as the product identity.

The exact native renderer and compositor are separate implementation decisions.
The product requirement is a modern, responsive, original interface; there is
no claim that this has already been realised.

## Primary technical sources

- National Institute of Standards and Technology. (2024). *Module-Lattice-Based
  Key-Encapsulation Mechanism Standard (FIPS 203).* https://csrc.nist.gov/pubs/fips/203/final
- Gueron, S., Langley, A., & Lindell, Y. (2019). *AES-GCM-SIV: Nonce
  Misuse-Resistant Authenticated Encryption* (RFC 8452).
  https://www.rfc-editor.org/rfc/rfc8452
- Biryukov, A., Dinu, D., Khovratovich, D., & Josefsson, S. (2021). *Argon2
  Memory-Hard Function for Password Hashing and Proof-of-Work Applications*
  (RFC 9106). https://www.rfc-editor.org/rfc/rfc9106
- National Institute of Standards and Technology. (2026). *Considerations for
  Achieving Crypto Agility: Strategies and Practices.*
  https://csrc.nist.gov/pubs/cswp/39/upd1/final

These sources specify components and migration principles; they do not
certify Baryon OS or this integration.

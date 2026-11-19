# NAS device-key lifecycle

**PROPOSED — owner decision required; no device trust, recovery escrow, key grant
or cryptographic protocol is adopted.** Baseline main `69d82075`.

## Existing bounds

[Key Service](../../services/keyring/src/lib.rs) separates domain key authority
from storage authority. Its `Service::revoke` retires an allocation; the
[persistent service](../../services/keyring/src/persistent.rs) provides
`Service::export_persistent`/`restore_persistent` with a 32-byte recovery secret.
That secret is not a password KDF. These mechanisms do not themselves implement
multi-device enrollment, device identity, rotation distribution or cold freshness.
The [encrypted-storage contract](ENCRYPTED-OBJECT-STORAGE.md),
[profile/user contract](PROFILES-AND-USERS.md) and
[B01 contract](B01-ENTROPY-AND-KEYS.md) remain authoritative.

## Pairing options requiring an owner choice

| Proposed route | Verified identity and maintained implementation route | New boundary requiring approval |
|---|---|---|
| First device and personally controlled NAS | Owner compares the exact peer fingerprint through a separate trusted channel, then approves an authenticated TLS session | How the initial fingerprint is obtained/displayed, authenticated account binding and a native certificate/calendar route |
| Additional personal device | An already authenticated device approves the exact new device identity, user/account scope and fresh enrollment request over authenticated TLS | Which existing device may enroll another, proof of user intent and loss/compromise of that approving device |

Both use the existing [rustls-based port](../../services/net-tls/README.md) as the
candidate implementation route, not a bespoke handshake. That maintained upstream
library has no accepted native provider, trusted calendar/signing or bootstrap
route in this project yet. Neither option authorizes adoption of AWS-LC, a new
certificate authority or any new dependency. No blind trust-on-first-contact,
NAS-name/IP identity, reused invitation or administrator-issued content-key grant.
The underlying provider/RNG/platform decision must be resolved separately.

## Proposed lifecycle

- Bind each device identity to the verified user/account, explicit object/key scope,
  enrollment generation and revocation state. No unbounded transitive enrollment.
- Revoke future device authority before cleanup. Retire its sessions/uploads and
  prevent enrollment or regrant through retained handles. Keys already exported to
  a formerly authorized device and copied plaintext cannot be recalled.
- Rotate future content wrapping/access material within the selected object scope;
  unrelated users/devices and NAS administrator rights do not expand. Exact key
  hierarchy, distribution and rotation authority require owner/security approval.
- Publish a new manifest/key generation only after the required ciphertext and
  metadata durability. An interrupted or unknown rotation retains an explicit
  recovery/uncertainty state; it never silently merges old and new plaintext.
- Recovery uses an explicitly stored offline high-entropy secret or an authorized
  existing device. The provider/NAS administrator receives neither. Lost recovery
  authority may mean unavailable data; no vendor escrow or silent online fallback.
- Existing snapshot allocation/revocation merge rules remain mandatory. Replaying
  old authenticated storage still requires an independent trusted freshness anchor;
  a fresh process or provider timestamp does not establish it.

Normal repeat synchronization within an approved device/scope should not prompt
again. New identity, wider scope, recovery transfer and changed trust authority are
separate consent boundaries. This proposal does not modify Claude-owned UI flows.

## Negative acceptance before release

| Requirement | Owner of future test | Required observation |
|---|---|---|
| LP-19 | Pairing/identity adapter | Wrong/expired/reused invitation, another fingerprint, another account and replayed enrollment grant no trust |
| LP-20 | NAS plus Key-Service integration | Administrator password reset cannot unwrap guest material or add a guest device |
| EO-12 | Device/key authority integration | Add/revoke affects only the approved device/object scope; old sessions/grants stay invalid |
| EO-10 | Consumer/persistence integration | Crash at each rotation boundary, lost device and unknown write yield no mixed plaintext or false completion |
| Recovery | Key-Service adapter | Wrong/missing secret, revoked allocation, stale authenticated state and hostile provider remain denied or unavailable |

**OWNER/SECURITY DECISIONS:** initial/secondary enrollment authority and identity
proof; key hierarchy/rotation distribution; recovery authorization and cold trust
anchor. No new trust root, escrow or capability is selected by this document.
Tests above are proposed, not executed; hardware/human audit and P03 acceptance
remain open. The roadmap score stays 41/100.

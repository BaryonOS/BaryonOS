# TEST anchors for B03

These keys are **test anchors** for the B03 gate (`tools/test_update.py`),
generated with `update-tool keygen --dir tools/update-keys --label TEST`.
The `.seed` files are the private seeds and are deliberately in the repository:
the loader with feature `signed-boot-test` and the ring-3 update service with
feature `test-signing-anchors` pin the corresponding `.pub` bytes, so that CI
can sign and verify reproducibly. The shared policy module is
only available with that explicit test feature. They are **not**
release keys and protect nothing outside QEMU acceptance; a
production policy or release profile has not been set up and requires a separate
design and authorisation path kept outside CI (ADR-049, invariant 7 in
docs/implementation/B03-SIGNED-UPDATES.md).

| File | Content |
|---|---|
| `TEST-ed25519.seed` | 32-byte Ed25519 seed (RFC 8032) |
| `TEST-mldsa65.seed` | 32-byte ML-DSA-65 seed (FIPS 204, `KeyGen_internal`) |
| `TEST-ed25519.pub` | 32-byte public key (anchor in loader and service) |
| `TEST-mldsa65.pub` | 1952-byte public key (anchor in loader and service) |
| `TEST-anchors.json` | key IDs (first 8 bytes of SHA-256 over the public key) |

The same five files exist with labels `TEST-next`, `TEST-revoked` and
`TEST-expired`. These are likewise deliberately public QEMU fixtures, not
secrets or production anchors. `update-core::test_policy` is the only shared
policy table for loader and service:

| Pair | Permitted release versions | State |
|---|---|---|
| `TEST` | 1 up to and including u64::MAX | existing pair, retained for regressions |
| `TEST-next` | 2 up to and including u64::MAX | successor pinned in advance |
| `TEST-revoked` | none | unconditionally revoked |
| `TEST-expired` | only 1 | refused outside this release window |

`KEY_EXPIRED` here means **outside the permitted release versions**, not
calendar expiry. The own RTC is not a trusted time source. Online key
discovery, dynamic trust store updates and production revocation distribution
have not been implemented. The manifest cannot add an anchor or policy.
`update-tool verify` checks an explicitly given key pair only
cryptographically; `verify-pinned` checks the shared QEMU policy.

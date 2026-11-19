# Native TLS provider evaluation

**PROPOSED — no provider, dependency, trust root or native TLS authority selected.**
Baseline main `69d82075`; evaluation date18November2026. The fixed probe profile
remains TLS1.3, X25519MLKEM768, TLS_AES_256_GCM_SHA384 and fixed ALPN, without
classical fallback, resumption or0-RTT. A library build is not a native handshake.

## Existing integration

The [net-tls manifest](../../services/net-tls/Cargo.toml) pins rustls0.23.46 with
only `custom-provider`, spin0.12.3 and zeroize1.9.1. The existing
[port contract](../../services/net-tls/README.md) records its provenance/licences,
feature and build-script inspection, `no_std` library evidence and remaining
allocator, trusted calendar, signing and native bootstrap gaps. That historical
evidence is reused within its recorded scope; no new compile or native result
is claimed. The [PQC contract](VERIFIABLE-SECURITY-AND-PQC.md) and
[keyring dependency contract](KEYRING-CRYPTO-DEPENDENCIES.md) remain authoritative.

## Candidate disposition

| Candidate | Provenance/licence and maintenance observation | Bare-metal/build/RNG disposition |
|---|---|---|
| rustls AWS-LC backend | [AWS primary repository](https://github.com/aws/aws-lc-rs), Apache-2.0 OR ISC; maintained upstream project, not a Baryon port acceptance | AWS-LC C/assembly and FFI/build tooling require a pinned target-specific build and RNG audit. No proved x86_64-unknown-none/Key-Service RNG route in this delivery; not selected. |
| RustCrypto provider | [RustCrypto primary repository](https://github.com/RustCrypto/rustls-rustcrypto), Apache-2.0 OR MIT; upstream warns of incomplete production suitability and states its current std requirement | Pure Rust is not proof of no_std or of this hybrid KEM. Exact release/features, transitive build scripts and all random routes remain unverified; not selected. |
| rustls-graviola0.4.0 | Existing port contract records std/TLS1.2 and AVX/AVX2 requirements as incompatible with the scalar VM profile | No CPU-feature expansion or new dependency. Current release/maintenance/build-script re-evaluation not completed; not selected. |

The [rustls primary defaults documentation](https://rustls.dev/docs/rustls/manual/_05_defaults/index.html)
connects X25519MLKEM768 to AWS-LC, but the fetched page identifies itself as
0.24.0-dev.2. This is upstream direction, not version-pinned evidence for the
project's0.23.46 port. Version-specific docs endpoints were inaccessible during
this check. No newer provider release, unsupported target or current advisory
status is inferred from that failure. Exact candidate archives/features/licences,
build scripts and a release-bound maintenance/advisory check remain prerequisites
before installing or selecting any provider.

## Required RNG and runtime seam

The existing `KeyServiceRandom` adapter is insufficient by itself: provider/KEM
key generation and signing may have separate random sources. Every route must
reach the authenticated Key-Service session, with the existing monotonic deadline,
process/account/generation binding, zero-on-failure and no uncertain replay rules.
No host getrandom, direct CPU RNG, cached seed or network fallback is an alternative.
A trusted certificate clock must fail closed when unavailable; monotonic uptime,
host/build time or arbitrary RTC does not become trusted calendar time.

## Proposed bounded next experiment and decision

Before a native provider is admitted, select a version/feature set with the
project owner/security authority; inspect its complete selected build graph and
licences. Then prove its random-route injection and scalar target build, followed
by a bounded allocator/OOM probe and a real Key-Service-backed native handshake.
Negatives must include missing/busy/revoked RNG authority, rollback/timeout,
partial/unknown delivery, unavailable calendar time, wrong identity/ALPN/suite,
classical-only peer and allocator exhaustion, with no plaintext before acceptance.
No self-written cryptographic primitive or trust-root substitution is permitted.

**Implementation remains blocked** on an accepted provider/RNG/platform route and
trusted certificate/signing/bootstrap contracts. Candidate manifest/build-script
and maintenance checks above are incomplete, not passing evidence. Native TLS,
certificate/peer binding, hardware, external audit and N03 acceptance remain open;
score41/100 is unchanged.

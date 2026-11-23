# Native TLS provider evaluation

**PROPOSED — no provider, dependency, trust root or native TLS authority selected.**
First evaluation: baseline main `69d82075`, 18 November 2026. Refresh: baseline
main `2f4ac6ae`, primary sources accessed 23 November 2026 (section
"Primary-source refresh"). The fixed probe profile remains TLS 1.3,
X25519MLKEM768, TLS_AES_256_GCM_SHA384 and fixed ALPN, without classical
fallback, resumption or 0-RTT. A library build is not a native handshake. The
owner/security decision options are in
[N03-PROVIDER-DECISION-PROPOSAL.md](N03-PROVIDER-DECISION-PROPOSAL.md).

## Existing integration

The [net-tls manifest](../../services/net-tls/Cargo.toml) pins rustls 0.23.46 with
only `custom-provider`, spin 0.12.3 and zeroize 1.9.1; its lock file resolves
rustls-webpki 0.103.16 and rustls-pki-types 1.15.1. The existing
[port contract](../../services/net-tls/README.md) records its provenance/licences,
feature and build-script inspection, `no_std` library evidence and remaining
allocator, trusted calendar, signing and native bootstrap gaps. That historical
evidence is reused within its recorded scope; no new compile or native result
is claimed. The [PQC contract](VERIFIABLE-SECURITY-AND-PQC.md) and
[keyring dependency contract](KEYRING-CRYPTO-DEPENDENCIES.md) remain authoritative.

## Candidate disposition (18 November 2026, retained as history)

| Candidate | Provenance/licence and maintenance observation | Bare-metal/build/RNG disposition |
|---|---|---|
| rustls AWS-LC backend | [AWS primary repository](https://github.com/aws/aws-lc-rs), Apache-2.0 OR ISC; maintained upstream project, not a Baryon port acceptance | AWS-LC C/assembly and FFI/build tooling require a pinned target-specific build and RNG audit. No proved x86_64-unknown-none/Key-Service RNG route in this delivery; not selected. |
| RustCrypto provider | [RustCrypto primary repository](https://github.com/RustCrypto/rustls-rustcrypto), Apache-2.0 OR MIT; upstream warns of incomplete production suitability and states its current std requirement | Pure Rust is not proof of no_std or of this hybrid KEM. Exact release/features, transitive build scripts and all random routes remain unverified; not selected. |
| rustls-graviola 0.4.0 | Existing port contract records std/TLS 1.2 and AVX/AVX2 requirements as incompatible with the scalar VM profile | No CPU-feature expansion or new dependency. Current release/maintenance/build-script re-evaluation not completed; not selected. |

The 18 November check found that the
[rustls primary defaults documentation](https://rustls.dev/docs/rustls/manual/_05_defaults/index.html)
connects X25519MLKEM768 to AWS-LC, but identified itself as 0.24.0-dev.2;
version-specific documentation was inaccessible. The refresh below replaces
that gap with version-bound source inspection.

## Primary-source refresh (23 November 2026)

### Method and evidence limit

Release data come from the crates.io API
(`https://crates.io/api/v1/crates/<name>`, generic User-Agent, accessed
23 November 2026) and from the upstream repositories linked in each row.
Source observations come from the exact published crate archives, fetched with
`cargo +1.100.0 fetch --locked` into scratch probe manifests outside the
repository and inspected as text. `cargo metadata` and
`cargo tree --target x86_64-unknown-none -e normal,build` were run per
candidate in isolated probe manifests, so that feature unification between
candidates does not attribute one candidate's build scripts to another.
**No build script was executed, no crate was compiled for any target and no
dependency was added to the repository.** A `#![no_std]` attribute is recorded
as an attribute observation only; compilation for `x86_64-unknown-none` is
NOT RUN for every candidate. Upstream changelogs were not consulted, except the
[rustls release page](https://github.com/rustls/rustls/releases) (accessed
23 November 2026) for the 0.23.46 advisory fixes.

### Hybrid group identity

X25519MLKEM768 is standardised in
[RFC 10024](https://www.rfc-editor.org/rfc/rfc10024.txt), "Post-Quantum
Traditional (PQ/T) Hybrid Key Agreement Mechanisms for TLS 1.3" (Kwiatkowski,
Kampanakis, Westerbaan, Stebila; IETF Standards Track, August 2026). The
[IETF datatracker record for RFC 10024](https://datatracker.ietf.org/doc/rfc10024/)
and the
[draft-ietf-tls-ecdhe-mlkem record](https://datatracker.ietf.org/doc/draft-ietf-tls-ecdhe-mlkem/)
identify it as published from draft-ietf-tls-ecdhe-mlkem-05 (all accessed
23 November 2026). The RFC number used by ADR-028 is therefore confirmed.
RFC 10024 applies the framework of RFC 9954 ("Hybrid Key Exchange in TLS 1.3")
and specifies, in its raw text:

- codepoint 4588 (0x11EC), DTLS-OK Y, Recommended Y (section 7.1);
- client share = ML-KEM-768 encapsulation key (1184 bytes) followed by the
  X25519 share (32 bytes), 1216 bytes in total; server share = ML-KEM
  ciphertext followed by the X25519 share;
- shared secret = ML-KEM shared secret followed by the X25519 shared secret,
  64 bytes (section 4.3);
- receiver obligations: the server performs the FIPS 203 section 7.2
  encapsulation-key check, the client checks the ciphertext length (both abort
  with `illegal_parameter`), any other ML-KEM decapsulation failure aborts with
  `internal_error`, and the X25519 part receives all validity checks of
  RFC 9846 section 4.3.8.2.

The rustls 0.23.46 and 0.24.0-dev.3 source comments still link the draft URL;
this is a documentation lag, not a different construction.

### Is X25519MLKEM768 available outside the aws-lc-rs provider?

| Source (exact version) | Observation | Consequence |
|---|---|---|
| rustls 0.23.46 (crate archive) | `X25519MLKEM768` exists only in `src/crypto/aws_lc_rs/pq/`; the generic `Hybrid` combiner there is `pub(crate)`. The `ring` module has no ML-KEM group. The `prefer-post-quantum` feature implies `aws_lc_rs`. | On the pinned release, rustls exposes the group **only** through the aws-lc-rs provider. Any other provider must carry its own concatenation/split code. |
| rustls 0.24.0-dev.3 (pre-release, published 2026-11-12) | `rustls::crypto::kx::Hybrid` and `HybridLayout` are public; `post_quantum_first` selects the order. aws-lc-rs appears only as a dev-dependency (through rcgen); providers moved to the separate crates `rustls-aws-lc-rs` and `rustls-ring` (both 0.24.0-dev.3, 2026-11-12). | Upstream direction makes the RFC 10024 concatenation a rustls-owned component usable by any provider. It is a development pre-release, not an admissible pin. |
| rustls-post-quantum 0.2.4 (2025-09-23) | Re-exports `rustls::crypto::aws_lc_rs::kx_group::{MLKEM768, X25519MLKEM768}`; depends on rustls with `aws-lc-rs` and on aws-lc-rs. Its crate documentation states that ML-KEM moved into rustls in 0.23.22. | Not an independent implementation; adds nothing for a non-AWS-LC route. |
| rustls-graviola 0.4.0 (2026-06-17) | Exports `kx::X25519MLKEM768`, built from its own `pub(crate) Hybrid` copy of the rustls glue over graviola ML-KEM and X25519. | A second provider exposing the group exists; its hybrid glue is provider-owned code, not rustls code. |
| rustls-rustcrypto 0.1.0-pre.0 (pre-release, 2026-11-09) | Key-exchange groups X25519, secp256r1 and secp384r1 only. Upstream [PR #143](https://github.com/RustCrypto/rustls-rustcrypto/pull/143), "Implement hybrid EC+MLKEM groups from draft-ietf-tls-ecdhe-mlkem", is open (opened 2026-02-24; state checked 23 November 2026 through the GitHub API). | No hybrid group in any release. |

### Release, licence and source observations

Dates are crates.io publication dates; "pre" marks a pre-release. Build
scripts are listed for the isolated `x86_64-unknown-none` normal+build graph
of each crate, including the crate itself.

| Crate | Latest (date) | Licence (crates.io) | `no_std` attribute | Build scripts in isolated graph | Random source | SIMD/backend selection |
|---|---|---|---|---|---|---|
| [rustls](https://github.com/rustls/rustls) | 0.23.46 (2026-10-14); 0.24.0-dev.3 pre (2026-11-12) | Apache-2.0 OR ISC OR MIT | `#![no_std]` | rustls only (nightly `read_buf` cfg; inert without that feature) | provider `SecureRandom` | none in core |
| [rustls-rustcrypto](https://github.com/RustCrypto/rustls-rustcrypto) | 0.1.0-pre.0 pre (2026-11-09); last default-branch commit 2026-11-09 | MIT OR Apache-2.0 | `#![no_std]` attribute, but the README states that `std` is currently required; default features `std`, `tls12` | curve25519-dalek, getrandom, num-traits, paste, proc-macro2, quote, rustls (73 nodes) | hard-wired `getrandom::SysRng` in `SecureRandom`, key exchange and signing | inherited from primitives |
| [rustls-post-quantum](https://github.com/rustls/rustls) | 0.2.4 (2025-09-23); 0.3.0-dev.2 pre (2026-07-27) | Apache-2.0 OR ISC OR MIT | none | aws-lc-rs, aws-lc-sys, libc, rustls | AWS-LC `RAND_bytes` | AWS-LC assembly |
| [rustls-graviola](https://github.com/ctz/graviola) | 0.4.0 (2026-06-17) | Apache-2.0 OR ISC OR MIT-0 | none (std crate) | getrandom, rustls | `getrandom::fill` inside graviola | graviola README requires x86_64 `aes`, `ssse3`, `avx`, `avx2`, `adx`, `bmi2` and `pclmulqdq`; `vzeroall` assumes AVX |
| [graviola](https://github.com/ctz/graviola) | 0.4.5 (2026-11-10) | Apache-2.0 OR ISC OR MIT-0 | none (std crate) | none | `getrandom::fill` | as above; optional AVX-512 with runtime detection |
| [ml-kem](https://github.com/RustCrypto/KEMs) | 0.3.4 (2026-10-31) | Apache-2.0 OR MIT | `#![no_std]` | num-traits | generic `CryptoRng`/`TryCryptoRng`; `DecapsulationKey::from_seed` public; `encapsulate_deterministic` public but `doc(hidden)` without `hazmat`; `getrandom` feature optional | portable Rust |
| [libcrux-ml-kem](https://github.com/cryspen/libcrux) | 0.0.12 (2026-11-19) | Apache-2.0 | `#![no_std]` | libc, libcrux-intrinsics, libcrux-ml-kem, libcrux-sha3 | caller-supplied `randomness: [u8; N]` arrays or `TryCryptoRng` | build scripts enable `simd256` on every x86_64 build unless `LIBCRUX_DISABLE_SIMD256` is set; hax/F* verification annotations present |
| [x25519-dalek](https://github.com/dalek-cryptography/curve25519-dalek) | 3.0.0 (2026-07-06) | BSD-3-Clause | `#![no_std]` | curve25519-dalek, proc-macro2, quote | `random_from_rng(CryptoRng)`; optional `getrandom` feature (`random()` panics on failure); `StaticSecret: From<[u8; 32]>` behind `static_secrets` | curve25519-dalek build script selects `serial`/`fiat`/`simd`/`avx512` through `curve25519_dalek_backend` |
| [aes-gcm](https://github.com/RustCrypto/AEADs) | 0.11.1 (2026-08-21) | Apache-2.0 OR MIT | `#![no_std]` | none | optional `getrandom`/`rand_core` (nonce generation only) | aes 0.9 and polyval 0.7 use `cpufeatures` runtime detection; `aes_backend="soft"` and `polyval_backend="soft"` force software |
| [sha2](https://github.com/RustCrypto/hashes) | 0.11.1 (2026-10-12) | MIT OR Apache-2.0 | `#![no_std]` | none | none | `cpufeatures` detection of SHA-NI/AVX2; `sha2_backend` cfg |
| [hkdf](https://github.com/RustCrypto/KDFs) | 0.13.0 (2026-03-30) | MIT OR Apache-2.0 | `#![no_std]` | none | none | through hmac/sha2 |
| [rustls-webpki](https://github.com/rustls/webpki) | 0.103.17 (2026-11-12) | ISC | `#![no_std]` | none (default features off) | none | through provider |
| [rustls-pki-types](https://github.com/rustls/pki-types) | 1.16.0 (2026-11-10) | MIT OR Apache-2.0 | `no_std` unless `std` | none | none | none |
| [aws-lc-rs](https://github.com/aws/aws-lc-rs) (reference) | 1.19.0 (2026-10-28); aws-lc-sys 0.47.0 (2026-10-28) | ISC AND (Apache-2.0 OR ISC); aws-lc-sys adds Apache-2.0, MIT, BSD-3-Clause and combinations | none; crate documentation states that the implementation requires `std` | aws-lc-rs and aws-lc-sys (CMake/cc C build, optional bindgen, `links` keys) | AWS-LC `RAND_bytes` | assembly unless the builder's no-asm mode is selected |

### RustSec status

Checked against the [RustSec advisory database](https://github.com/rustsec/advisory-db)
`crates/<name>` directories at advisory-db commit `52384ab5` (2026-11-23) and
the [rustsec.org](https://rustsec.org/) package pages, both accessed
23 November 2026. A missing directory is recorded as "no advisory recorded",
not as proof of absence of defects.

| Crate | Advisories (date, patched range) | Status for the versions in question |
|---|---|---|
| rustls | RUSTSEC-2024-0336, RUSTSEC-2024-0399, RUSTSEC-2026-0285 (2026-09-14, ≥0.23.45), RUSTSEC-2026-0346/0347/0348/0349 (2026-10-14, ≥0.23.46) | 0.23.46 is patched for all. RUSTSEC-2026-0349, "Unauthenticated ServerHello can downgrade later connections to classical key exchange", is directly relevant to the no-classical-fallback invariant. |
| rustls-webpki | RUSTSEC-2023-0053, RUSTSEC-2026-0049, -0098, -0099, -0104, -0382 (2026-10-20, ≥0.103.16) | The net-tls lock (0.103.16) and 0.103.17 are inside every patched range. |
| aws-lc-sys | RUSTSEC-2026-0044 to -0048 (patched ≥0.38.0 or ≥0.39.0) | 0.47.0 is patched. |
| aes-gcm | RUSTSEC-2023-0096 (patched ≥0.10.3) | 0.11.1 is patched. |
| sha2 | RUSTSEC-2021-0100 (affects 0.9.7 only) | 0.11.1 is not affected. |
| curve25519-dalek | RUSTSEC-2024-0344 (patched ≥4.1.3) | x25519-dalek 3.0.0 resolves curve25519-dalek 5.0.0. |
| rustls-rustcrypto, rustls-post-quantum, rustls-graviola, graviola, ml-kem, libcrux-ml-kem, x25519-dalek, hkdf, rustls-pki-types, aws-lc-rs, getrandom | no advisory recorded | — |

### RNG injectability

- getrandom 0.4.3 offers the `getrandom_backend = "custom"` and
  `"extern_impl"` cfgs. These are process-global link-time hooks with a plain
  byte-fill interface; they cannot carry a per-session Key-Service handle,
  deadline or generation binding by type. They are the only injection point for
  graviola, rustls-graviola and rustls-rustcrypto, which call getrandom
  directly.
- ml-kem, libcrux-ml-kem and x25519-dalek accept caller-supplied randomness
  (generic `CryptoRng`/`TryCryptoRng` or byte arrays). A fallible Key-Service
  draw into a fixed buffer, followed by deterministic constructors, avoids the
  `UnwrapErr`/`expect` panic paths. Whether `encapsulate_deterministic` may be
  used without the `hazmat` feature is a dependency-policy question.
- rustls itself takes randomness only through the provider's `SecureRandom`,
  which the existing `KeyServiceRandom` adapter already implements.

### Refreshed disposition

| Route | Disposition on 23 November 2026 |
|---|---|
| aws-lc-rs (rustls 0.23.46 built-in group) | The only released source of X25519MLKEM768 inside rustls. Requires `std`, a C/assembly build, CMake/cc tooling and `RAND_bytes`; a bare-metal port is a separate engineering project. Not selected. |
| rustls-graviola / graviola | Exposes the group, but is std-only, getrandom-bound and AVX/AVX2/ADX/BMI2-dependent, which conflicts with the scalar VM profile. Not selected. |
| rustls-rustcrypto | Pre-release, std-required per README, no hybrid group, hard-wired getrandom. Not selected. |
| Primitives (ml-kem or libcrux-ml-kem, x25519-dalek, aes-gcm, sha2, hkdf) | `no_std` attributes and injectable randomness; build scripts absent or controllable. Usable only behind provider glue that rustls 0.23.46 does not supply. Not selected. |
| rustls 0.24 public `Hybrid` | Removes the need for project-owned concatenation code once released; currently a development pre-release. Not selected. |

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
trusted certificate/signing/bootstrap contracts. The refresh above completes the
release, licence, attribute, build-script and advisory inspection for the named
candidates at text level only; target compilation, build-script execution and
any handshake are NOT RUN. Native TLS, certificate/peer binding, hardware,
external audit and N03 acceptance remain open; score 41/100 is unchanged.

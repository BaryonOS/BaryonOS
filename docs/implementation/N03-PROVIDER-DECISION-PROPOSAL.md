# N03 — TLS provider, test peer and time authority: decision proposal

**Status: DECIDED 25 November 2026 — see ADR-N03-PROVIDER-20261125 in
[DECISIONS.md](../DECISIONS.md), which takes precedence over this memo.**
Original status: PROPOSED, 23 November 2026, awaiting a decision by the project owner
and the security authority. This memo selects nothing, adds no dependency
and changes no security contract. It prepares the "Owner/SEC ADR on provider
and test peer pinning" on which roadmap task N03-T2 is blocked. The evidence
it relies on is the 23 November refresh in
[N03-NATIVE-TLS-PROVIDER.md](N03-NATIVE-TLS-PROVIDER.md); claims below that
are not in that refresh are marked as unverified.

**Reported decision (P2 host oracle only).** In the interactive N-track session
of 23 November 2026, the project owner chose "Host oracle first": aws-lc-rs may
be a host-only test dependency for the N03-T2 loopback handshake, and the native
provider choice stays open. This choice is reported here from that session and
from [PR #426](https://github.com/BaryonOS/BaryonOS/pull/426), which implements
it. This memo does not establish its formal approval status; no ADR records it
yet. The native provider route (P1), the test-peer model (P3), the
certificate-time policy (P4) and the random route (P5) remain open.

## Why a decision is needed

Criterion N03 requires that "maintained cryptography/TLS with the documented
hybrid PQC choice works natively, including peer verification, negative
handshakes, and no silent insecure fallback". The release-matrix row FR-N03 in
[FULL-TEST-RELEASE.md](../FULL-TEST-RELEASE.md) adds authentic
peer/hostname/ALPN checks and negative cases for expired or incorrect
certificates, mismatch, malformed handshakes and silent classical/insecure
fallback, at evidence levels N, T, P, H and A.

ADR-028 (in [DECISIONS.md](../DECISIONS.md)) fixes the profile: TLS 1.3 only,
X25519MLKEM768 (RFC 10024) only, mutual certificate checking/pinning,
existing peer-bound rights, "no custom combiner/record crypto" and no classical
fallback. The refresh establishes three facts that make a decision necessary:

1. Among the providers built into the pinned rustls 0.23.46, X25519MLKEM768
   is reachable **only** through aws-lc-rs; the generic `Hybrid` combiner is
   crate-private. External providers that carry their own hybrid code
   (rustls-graviola, rustls-openssl; see P2) are host-only std crates.
2. aws-lc-rs requires `std` and a C/assembly build with its own `RAND_bytes`;
   no released provider exposes the group in a `no_std` form.
3. rustls 0.24.0-dev.3 (pre-release) makes `Hybrid`/`HybridLayout` public, so
   the RFC 10024 concatenation would become rustls-owned code, but that
   release line is not yet stable.

No technical route reaches native N03 acceptance without at least one choice
that the project owner or security authority must make.

## Constraints that every option keeps

- Security Floor: no option weakens process isolation, capability enforcement,
  key isolation, memory protection or protection of unshared data.
- No self-written cryptographic primitive, no record-protection code and no
  trust-root substitution.
- **Proposed native-acceptance requirement:** every random byte used for key
  generation, encapsulation, TLS randoms and signing reaches the authenticated
  Key-Service session; no host getrandom, CPU RNG, cached seed or network
  fallback. **Host-test state (implemented in
  [PR #426](https://github.com/BaryonOS/BaryonOS/pull/426)):** the host
  oracle routes TLS randoms through `KeyServiceRandom`, but aws-lc-rs draws
  key-exchange, encapsulation and signing randomness internally and bypasses
  it. **Unknown:** no decision establishes this as an approved host-test
  exception; question 3 asks for one. Host evidence never counts towards this
  requirement.
- Monotonic uptime, host/build time or an untrusted RTC never becomes calendar
  time.
- A classical-only peer is refused; there is no second group in the offer.
- Host results are labelled host evidence; they never count as native
  acceptance.

## Decisions requested

### P1 — Native provider route

| Option | Description | Security Floor impact | Evidence it would unlock | Effort | Reversibility |
|---|---|---|---|---|---|
| A — aws-lc-rs bare-metal port | Port aws-lc-rs/aws-lc-sys (C, assembly, CMake/cc build) to `x86_64-unknown-none` with a Key-Service-backed `RAND_bytes` replacement and an allocator shim. | No change to the floor if the RNG replacement is complete; large new C trusted computing base inside the TLS service process. | Full N03 profile through rustls' own X25519MLKEM768 code; native handshake, peer verification and negatives once ported. | Large: target-specific C build, libc surface, threading/RNG hooks, ELF size limits; upstream states `std` is required. | Medium: a separate service-local build; abandoning it leaves rustls unchanged. |
| B — project-composed provider over RustCrypto/libcrux primitives | Implement the rustls `CryptoProvider` traits (`SupportedKxGroup`, AEAD, hash/HMAC, signature verification and signing, `SecureRandom`) as glue over released `no_std` primitives (ml-kem or libcrux-ml-kem, x25519-dalek, aes-gcm, sha2, hkdf, and ed25519-dalek for certificate verification and client signing, because rustls-webpki without its `ring`/`aws-lc-rs` features supplies no verification algorithm). **B1** on rustls 0.23.46 carries a project copy of the RFC 10024 concatenation/split glue, as rustls-graviola does. **B2** waits for a stable rustls 0.24 and uses its public `Hybrid`, so only primitive adapters remain project code. | No new primitive; adds project-owned glue to the TLS trusted computing base. Each adapter must enforce the RFC 10024 receiver obligations, including the X25519 all-zero shared-secret check (RFC 10024 section 4.3, RFC 9846 section 7.4.2; abort with `illegal_parameter`). | The N03 profile natively, provided the signature adapter and the receiver checks pass review and tests; RNG binding is exact because all primitives take caller-supplied randomness. | Medium (B1) or medium after an upstream wait (B2); scalar backend cfgs and the libcrux build scripts need an explicit policy. | High: provider is service-local; B1 glue can be deleted when B2 becomes available. |
| C — wait for an upstream `no_std` provider | Make no project provider; wait until a maintained provider (for example rustls-rustcrypto after PR #143, or a 0.24 split provider) releases X25519MLKEM768 with `no_std` and injectable randomness. | None. | None until upstream releases; N03 remains open. | Small now; unbounded calendar time. | Complete. |
| D — host-only loopback with aws-lc-rs as test oracle | Run rustls 0.23.46 with aws-lc-rs on the host to perform an X25519MLKEM768 loopback handshake and the negative cases, while the native route stays open. | None for the OS; host-only test dependency. | N03-T2 (host loopback) only. Host evidence earns no N03 points and closes no FR-N03 clause at level N. | Small to medium; requires a new host-only dependency and lock-file change with provenance review. | Complete: test-only crate. |

**The ADR-028 "no custom combiner" tension, stated precisely.** RFC 10024
defines no new cryptographic combiner: it concatenates the ML-KEM and X25519
shares and the two 32-byte secrets, and the unchanged TLS 1.3 key schedule
(HKDF) derives all keys from that 64-byte value. Under B1, project code would
perform that concatenation, split received shares by fixed lengths and route
the receiver checks (encapsulation-key check, ciphertext length, X25519
validity) to the primitives. This code is specified byte-for-byte by the RFC
and mirrors existing upstream code, but it is project-owned code on a
key-establishment path. Whether such RFC-specified glue is a "custom combiner"
within the meaning of ADR-028 is an interpretation that only the project owner
or security authority can give. Under B2 the concatenation lives in rustls and
the question reduces to whether trait adapters over audited primitives count as
"record crypto". Under B2, the AEAD adapters perform the encryption and
decryption of TLS records that rustls invokes through its provider interfaces,
while rustls keeps the TLS protocol logic that calls them. Whether that boundary
is compatible with ADR-028 is for the project owner and security authority to
decide. Whether ml-kem's `EncapsulationKey` decoding performs the full
FIPS 203 section 7.2 check is **unverified** and must be tested before B is
accepted.

### P2 — Host-loopback test oracle

| Option | Description | Consequence |
|---|---|---|
| H1 | rustls 0.23.46 + aws-lc-rs on both ends in a host-only test crate. | Same library family on both ends; weak as an independent oracle. |
| H2 | The rustls end against the existing N02 host peer (Node.js 24.22.1 with OpenSSL 3.5.9, see [pqc-transport evidence](../verification/pqc-transport/README.md)). | Independent peer that already negotiated TLS 1.3/X25519MLKEM768/AES-256-GCM-SHA384. H2 replaces only the peer. |
| H3 | Both H1 and H2. | Strongest host evidence; largest host test surface. |

Under every P2 option the rustls end still needs a provider that offers
X25519MLKEM768. On rustls 0.23.46 the published candidates are aws-lc-rs;
rustls-graviola 0.4.0 on x86_64 hosts with `aes`, `ssse3`, `avx`, `avx2`,
`adx`, `bmi2` and `pclmulqdq`, or aarch64 hosts with `aes`, `sha2`, `pmull`
and `neon` (graviola README); and rustls-openssl 0.4.2 (MIT, rustls 0.23,
system or vendored OpenSSL). Its build script compiles the KEM groups,
X25519MLKEM768 included, whenever the linked OpenSSL is 3.0 or later
(`ossl300` cfg), and `available_groups()` keeps a group only if OpenSSL can
fetch the algorithm at run time. OpenSSL 3.5 includes ML-KEM in its default
provider; an earlier 3.x host offers the group only if another loaded OpenSSL
provider supplies it (source: Tom Fay, *rustls-openssl*
0.4.2, crates.io release of 2 October 2026, crate documentation
`src/lib.rs`, `src/kx_group/mod.rs` and `build.rs`,
<https://docs.rs/rustls-openssl/0.4.2/rustls_openssl/>; repository
<https://github.com/tofay/rustls-openssl>). Each of these three published host candidates draws
key-exchange randomness outside `KeyServiceRandom`: aws-lc-rs through
`RAND_bytes`, rustls-graviola through graviola's unconditional
`getrandom::fill` (graviola 0.4.5 `src/mid/rng.rs`), and rustls-openssl
through OpenSSL. The rustls-openssl entry is compatibility
evidence from its published source only; the project has neither tested nor
approved it, and a host OpenSSL dependency would also make H1 and H2 share
the OpenSSL implementation. An option B provider can take that place only
after it exists. H2 therefore needs aws-lc-rs only where no other candidate
is available on the test host and option B is not yet implemented. Any host oracle is a development and test tool only. It never ships in an OS
image and never supplies native acceptance evidence.

### P3 — Test peer pinning model

| Element | Proposal for decision |
|---|---|
| Trust anchor | A dedicated, repository-public **test** CA, generated for QEMU/host tests only, marked as such in its name, never in a product trust store (analogous to the public `tools/update-keys/TEST-*` keys). |
| Pin | Exact SubjectPublicKeyInfo (SPKI) SHA-256 pin of each test peer leaf, bound to the existing peer-pin generation; chain validation to the test CA **and** pin match both required. |
| Certificate algorithm | Ed25519, consistent with the ADR-028 trial; explicitly classical authentication. According to the rustls 0.23.44 release notes, ML-DSA certificate support is enabled in the aws-lc-rs provider; other ML-DSA routes were not evaluated. ML-DSA would therefore add the P1 dependency question to authentication as well. |
| Identity | Fixed test DNS name and ALPN; wrong name, ALPN, pin, CA, suite or group are mandatory negatives. |
| Rotation | A pin-generation increment invalidates old authenticated connections, as ADR-028 already requires. |

### P4 — Certificate-time policy

| Option | Description | FR-N03 effect |
|---|---|---|
| T1 — trusted calendar authority | Validity periods are checked against a trusted calendar source through the existing CORE time port used by [net-tls](../../services/net-tls/README.md) (`CertificateClock`, which returns no time when unavailable, so validation fails closed). T1 keeps the fail-closed certificate clock. The CMOS RTC is not such a source: [RTC-WALLTIME.md](RTC-WALLTIME.md) labels it `UnauthenticatedRtc` and states that trusted certificate time is not provided. | Allows the "expired certificate" negative at level N; requires a trusted time source that does not exist natively yet. |
| T2 — explicitly scoped pin-only acceptance | **Proposed test-scoped exception that requires an owner decision:** it waives the fail-closed certificate clock. For pinned test peers only, validity periods are not evaluated; acceptance rests on CA chain shape, SPKI pin and pin generation. The scope is recorded in the peer record and never applies to non-pinned peers. | Unblocks a native handshake without calendar time, but leaves the "expired certificate" clause unmet and must not be reported as full peer verification. |
| T3 — T2 now, T1 required for acceptance | T2 for development probes, with the same proposed waiver for pinned test peers only; N03 acceptance requires T1. | Separates progress from acceptance; needs explicit labelling in every evidence record. |

### P5 — RNG route through the Key-Service

| Option | Description | Assessment |
|---|---|---|
| R1 — per-call session draws | Every random request (TLS randoms, X25519 secret, 64-byte ML-KEM key-generation seed, 32-byte encapsulation message) is a separate `KeyServiceRandom` draw of at most 128 bytes, then fed to deterministic primitive constructors. Ed25519 signing is deterministic and draws no randomness. | Keeps the existing deadline, generation binding, zero-on-failure and no-replay rules unchanged. Compatible with option B; option A needs a `RAND_bytes` replacement that preserves the same rules. |
| R2 — getrandom custom backend | Link a process-global getrandom backend that forwards to the Key-Service. | Cannot carry a session handle, deadline or generation by type; an error path may panic inside callers that use `UnwrapErr`. Not recommended. |
| R3 — per-handshake batched draw | One draw sized for a whole handshake, split locally. | Fewer IPC round trips; adds project-owned splitting of key material and a longer-lived secret buffer. |

## What a decision unlocks

| Decision | Unlocks |
|---|---|
| P1 = D (with P2, P3) | N03-T2 host loopback handshake and host negatives. No N03 points. |
| P1 = A or B, with P3, P4 = T1 and P5 = R1 | N03-T3 native probe: authentic peer/ALPN, wrong certificate/pin/ALPN/suite, malformed handshake, classical-only refusal, expired certificate. Native acceptance additionally needs a TLS principal, a native trusted time source and the network path, as the ledger records. |
| P4 = T2 or T3 | A native handshake probe before trusted time exists, explicitly without the "expired certificate" clause. |
| Any choice | Hardware (H) and external audit (A) levels remain open. |

## Evidence boundary

This memo is a proposal written from source inspection and primary documents;
nothing in it was compiled, executed or handshaken. The effort ratings are
relative judgements, not time forecasts. A second AI review is not a human
audit. The N03 score contribution stays at zero, and the overall score stays
41/100.

## Questions for the project owner

1. Which native provider route applies: A (aws-lc-rs bare-metal port), B1
   (project glue on rustls 0.23.46), B2 (project primitive adapters on a stable
   rustls 0.24), C (wait upstream), or D now with a later native choice?
2. Does RFC 10024-specified concatenation/split code written in the project,
   mirroring upstream rustls code, fall within ADR-028's "no custom combiner"
   prohibition?
3. Do you confirm or reject the reported choice of a host-only aws-lc-rs test
   dependency for the rustls end of the host loopback, including the bypass of
   `KeyServiceRandom` by its internal randomness as a host-test exception? On
   rustls 0.23.46 the alternatives are rustls-graviola 0.4.0 on x86_64 or
   aarch64 hosts with the CPU features its README requires, and the untested
   rustls-openssl 0.4.2 against an OpenSSL that provides ML-KEM at run time,
   by default from 3.5 (see P2); aws-lc-rs is
   needed only where neither alternative nor an option B provider is
   available. Each of the three published host candidates draws key-exchange
   randomness from its own host route (AWS-LC `RAND_bytes`, host getrandom for
   graviola, OpenSSL's RNG), so the host-test randomness exception applies to
   whichever of them is selected. An implemented option B provider that draws
   through Key-Service (R1) would not need the exception. Should the peer be rustls (H1), the existing Node.js/OpenSSL
   peer (H2) or both (H3)?
4. Is a repository-public test CA with exact SPKI pins and Ed25519 leaf
   certificates acceptable for QEMU and host tests (P3)?
5. Which certificate-time policy applies: T1, T2 or T3?
6. Is per-call Key-Service randomness (R1) confirmed as the only admissible
   route, excluding the getrandom custom backend?
7. Are libcrux build scripts and SIMD cfgs acceptable if the scalar profile is
   forced, or is ml-kem (no build script of its own) the only admissible ML-KEM
   crate?
8. Under option B, is ed25519-dalek 3.0.0 acceptable as the signature
   verification and signing primitive, subject to the dependency policy of
   ADR-VERSION-POLICY-20261117 and [DEPENDENCIES.md](../DEPENDENCIES.md)?

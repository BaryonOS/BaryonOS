# Bounded random bytes through the Key Service

TASK-KEYRANDOM, technical assignment based on `1b6a308`. The first step is a
protocol, service policy and client suitable for native integration. Host
trials/target builds do not demonstrate native RANDOM, TLS, certificate
identity or physical entropy. The integrator retains ownership of the kernel
manifest, shared lockfiles, build profiles and native TCP/TLS callers. The
complete assignment remains authoritative in
[FULL-TEST-RELEASE](../FULL-TEST-RELEASE.md).

## Authority and protocol

KEY1 opcodes 1–9 retain their format and key authority. New opcode 10
(`RANDOM`) uses only a separately granted RANDOM ingress with its own reply
endpoint and recorded caller PID/principal slot. Domain and clock come from
the trusted adapter, never from payload or caller assertions. The two existing
key ingresses do not offer RANDOM. The new ingress rejects all key,
decryption, recovery and export opcodes, including when an app sends them
directly outside the SDK. A valid signature or received capability grants no
additional authority.

A message has the existing 16-byte KEY1 header and exactly 16 request-body
bytes: `RND1` (4), version 1 u16LE, requested count u16LE, eight zero bytes.
Header index=0, request ID>0, status=0. Count is 1..224. A successful response
contains the same 16-byte prefix and exactly count bytes; 16 header + 16 prefix
+ 224 = 256. All trailing bytes are zero. Errors contain no body or random
bytes. Request IDs strictly increase per principal slot and are not reused.
This is a live-instance boundary, not a persistent anti-rollback anchor.

The SDK defaults to chunks of at most 128 bytes. It owns one exclusive
endpoint pair, an expected service PID and one request at a time. It validates
sender, tag, absence of delegated capabilities, header/opcode/index/request
ID, status, version, length, reserved bytes and padding before output.
Cancellation or uncertain IPC closes the lease; no silent retry or reacquisition
of authority. A response already sent cannot be cryptographically recalled:
revocation blocks future actions, not random bytes previously received lawfully.

`Session::begin(count)` publishes nothing; each `poll()` makes at most one
send or receive attempt. The same request is offered again only after an
EAGAIN that demonstrably published nothing. After enqueue, only receive
follows. The total poll budget is 1..1,000,000, without an unbounded internal
wait loop. `Outcome::Service(NotReady/Quota)` is a fully validated, **completed
response after publication**; it is not a BeforeSend state. An explicit new
request receives a new ID. Degraded closes the pair. Other status codes,
invalid responses and transport uncertainty are terminal. Cancellation before
enqueue also closes the pair; there is no hidden regrant or replay.

Request IDs start at 1. The endpoint pair is transferred only on successful
construction and must be exclusive. A new pair must not implicitly reuse the
request history of an already used principal slot: bootstrap must provide a
fresh service/principal lifecycle or a later explicitly coordinated monotonic
allocator. Incorrect binding fails closed; the client has no reset opcode.
Drop attempts to close both handles, as do explicit close and terminal errors;
a failed close is not retried later against a potentially reused handle generation.

## State, limits and errors

The existing 1024-byte startup/health test remains required. A RANDOM slot is
fed only on its first valid request, with 256 fresh broker bytes passing the
same continuous RCT/APT tests. HKDF-SHA256 extracts a 32-byte seed using the
separate RNG-v1/domain/monotonic-epoch label; no master key or recovery secret
is used for this. Each of four fixed slots has a separate ChaCha20Rng.
Restoring stored keys does not rewind the RNG.

After at most 4096 returned bytes or 64 requests, a new healthy 256-byte seed
is required before further output. Apps have no generator, seed, seek,
serialisation or reseed opcode. The epoch prevents repetition of identical
labels within the instance, even with a repeated raw block; it does not prove
source quality or cold rollback protection. The model assumes genuinely
unpredictable source bytes, not merely bytes passing a health test.

Per slot, at most 8192 bytes and 64 calls are permitted per 100 actual PIT
ticks. The adapter supplies Sys3 ticks; loop counts and host time are not used.
Backward/overflowing time, invalid payloads, replay and quotas are rejected
before entropy/generator processing and leave output untouched. Seed failure
returns Degraded, erases all RNG caches and supplies no fallback output.
Existing master, key allocations and revocations are retained. An RNG error
does not implicitly revoke or reissue a confirmed key. Starting/Blocked
return NotReady; Degraded remains closed.

Explicit raw/seed/output buffers use Zeroizing or an explicit erase operation.
The RNG feature erases core/key and precomputed blocks on Drop. The KDF patch
described below additionally handles explicit PRK, HMAC keypad, previous/output
block and HMAC hash buffers and retained SHA2 hash states. That enumeration
is the examined boundary; no guarantee covers deep compression temporaries,
Rust moves, compiler, register, SDK, kernel or crash copies. Caller-owned output
and data already granted remain the recipient's responsibility.

## Existing startup ABI and future native binding

The optional service descriptor is placed exclusively by the integrator:
word 120=magic/version `NOSRNG01`, 128=RANDOM-RECV, 136=reply-SEND, 144=caller
PID, 152=RANDOM principal slot 0..3. Zero magic means no descriptor. Current
keyring and joint manifests do not use these service words; 112 remains the
existing case word, 160/176 the existing app/service PIDs. An incomplete or
unknown descriptor fails initialisation. Old profiles without a descriptor
do not call Sys3; without available Sys3, RANDOM produces no output. This
agreement reserves startup positions and is not a general bootstrap ABI.
This step has not yet introduced kernel binding or native acceptance.
Words 272/280/288 contain only cumulative RANDOM calls/bytes/seeds. They are
not yet kernel-validated native evidence. The adapter validates the actual
kernel sender, tag, capabilities, complete canonical KEY1 envelope and RANDOM
opcode before Sys3 or entropy. The five descriptor words are not an app
message and must originate from trusted bootstrap. Exact aliasing with
existing descriptor handles is rejected; identity and rights of different
handles remain a kernel/bootstrap responsibility.

## Dependencies and TCB per examined property

The RNG is the maintained RustCrypto implementation `chacha20=0.10.2`, alias
`chacha20-rng`, with only features `rng,zeroize`, without default `cipher`.
`rand_core=0.10.1` supplies RNG traits. Existing AEAD/legacy DRBG remain on
chacha20 0.9.1. New distribution checksums:

| Crate | SHA256 | Licence/MSRV/build script |
|---|---|---|
| chacha20.10.2 | 65c35e4b699c7e15ccbe7ee35c005e4fc0a278d22238a2857e6ce2dadeda1b06 | MIT/Apache2; 1.85; none |
| rand_core.10.1 | 63b8176103e19a2643978565ca18b50549f6101881c443590420e4dc998a3c69 | MIT/Apache2; 1.85; none |
| cpufeatures.3.1 | 5ca28b0ae3115b884660db4118d803791fd6756b6e88f39c0f3f7859060d7566 | MIT/Apache2; 1.85; none |
| zeroize1.9.1 | e13084392c5e4bc371903e2935a5eaeed24905a7511356b883835e18a78f6879 | Apache2/MIT; 1.85; none |
| cfg-if1.0.5 | 4e7648175b45a9a48536d676f68d918270699102aa8dab5496df06904c914600 | MIT/Apache2; 1.32; none |

Archives/registry checksums, manifests, licences, RNG/Drop code and relevant
RustSec advisories were read in a targeted review. The known chacha20 2019
counter bug is fixed from .2.3; the rand_core 2019 alignment bug from .4.2 and
2021 seed bug from .6.2. Versions .10 fall outside those ranges. Version .10.2
also fixes an SSE4.1 intrinsic in the SSE2 backend; .10.0/.10.1 are withdrawn.
No general audit coverage is claimed for the new RNG version. Rand_chacha .10
is not added; it would introduce an unnecessary PPV/SIMD tree here.

The custom kernel and Rust 1.90 are retained. Native compilation requires
`--cfg chacha20_backend="soft"` alongside existing `chacha20_force_soft`,
`poly1305_backend="soft"`, and `sha2_backend="soft"`. A none build without the new
portable cfg explicitly fails. The integrator updates shared builders/lockfiles;
this task does not write them. No std/libc/getrandom or host fallback on the
x86_64-none target. The concrete lock graph and object code are retested.

The TCB for unpredictability includes source/virtio-rng/QEMU host in this VM
profile, kernel brokering/usercopy, health tests, HKDF and RNG crates, service
code, compiler and portable build selection. Principal separation additionally
includes capability/IPC isolation, bootstrap binding and client validation.
This TCB is larger than the kernel alone. For issuance, the authorised
recipient can retain bytes or send them through existing network rights; this
API grants no read, sign, decrypt or export right over personal keys/data.

TLS identity signing, trusted certificate time, hybrid PQC contract,
boot/update authenticity, persistent key protection and physical RNG quality
measurement remain separate open requirements. RANDOM justifies neither
classical/cloud fallback nor a time or certificate bypass.

## Evidence produced and next integration step

Compact source/command evidence appears in
[`keyrandom-host-none.json`](../../services/keyring/verification/keyrandom-host-none.json).
Against 9 exactly hashed source/manifest files, 57 core tests (46 existing and
11 RNG) and 11 client tests passed. These include: published ChaCha20 zero-key
vector, 1..224 core and 1..128 client lengths, principal binding with identical
source bytes, quota/time/replay before mutation, healthy mandatory reseeds,
source failure without master/key reset, rejection of every legacy operation
on RANDOM ingress, and incorrect sender/tag/capabilities/opcode/request ID/
status/length/padding. Transport trials check deadline, cancellation, uncertain
send, no resend after enqueue, both endpoint closes and closure of unexpected
capabilities.

Host Clippy for both crates and native Clippy for the service pass with
`-D warnings`; rustfmt and `git diff --check` pass. Native release builds for
the service, existing test app and client library succeed with Rust 1.90 for
x86_64-unknown-none. Objdump of the service contains no FP/SIMD register use.
The checked native dependency tree contains no std/libc/getrandom. A none build
without the new portable cfg was actually rejected (exit 101).

Builds use a separate exact source copy with its own Cargo.lock under
`/tmp/nieuw-os-keyrandom-integration` and target under
`/tmp/nieuw-os-keyrandom-target`; shared root/SDK locks are untouched.
This is host/cross-compilation evidence and **not an executed native RANDOM
call**. The integrator must add client workspace/locks/build flags, trusted
descriptor, RANDOM-only app grant, clock profile and independent runtime trials.
Actual issuance and rejection on revocation, incorrect ingress, missing
clock/entropy, reseed failure and quota remain required, together with legacy
regressions on the same integrated image. This step signs off no release criterion.

Primary sources consulted in the targeted review (no general audit claim):
[RustCrypto chacha20.10.2](https://crates.io/crates/chacha20/0.10.2),
[rand_core.10.1](https://crates.io/crates/rand_core/0.10.1),
[RUSTSEC-2019-0029](https://rustsec.org/advisories/RUSTSEC-2019-0029.html),
[RUSTSEC-2019-0035](https://rustsec.org/advisories/RUSTSEC-2019-0035.html),
[RUSTSEC-2021-0023](https://rustsec.org/advisories/RUSTSEC-2021-0023.html).

## KDF memory-hygiene correction

Review of `fc2e2a2` found an explicit gap: HKDF 0.12/HMAC 0.12 retained an
entropy-derived PRK/HMAC state without an erasing Drop. This was a managed
secret and is not dismissed as merely a compiler copy. An undocumented upgrade
to HKDF 0.13 would be insufficient: that source also left PRK, expand blocks
and several HMAC buffers unerased.

Therefore **all production KDF paths** use the targeted, reviewable
[HKDF 0.13/HMAC 0.13 hygiene variant](../../services/keyring/vendor/README.md)
with SHA2 0.11.1/zeroize. No custom KDF, combiner, hash or MAC; the patch changes
only buffer lifetime, erasure and feature/trait connections. This covers
existing startup and reseed derivation of the legacy DRBG,
`derive_key(master,domain,index)` for legacy/record encryption,
`recovery_key` for the existing recovery fixture, `persistent::wrapping_key`
under the external user secret, and `random_at` for RANDOM seeds.
All labels, outputs, key handles and snapshot/wire versions remain the same.
HKDF 0.12/SHA2 0.10 remain solely development oracles for byte comparison;
the native production tree contains none of those old KDF/hash paths.

Exact new registry pins alongside the two local source variants are SHA2
0.11.1, digest 0.11.3, hybrid-array 0.4.15 and zeroize 1.9.1. New active registry
versions match the previously reviewed TLS graph; HKDF .13 and the optional
KDF .1 trait received additional primary-source checking. There are no new
build scripts, std/libc/getrandom or native host fallback. The highest new
MSRV is 1.88; kernel/toolchain 1.90 remains.

Current source/test/none-build records appear in
[`hkdf-hygiene/host-none.json`](../../services/keyring/verification/hkdf-hygiene/host-none.json).
The earlier RANDOM report remains historical evidence for the source before
this correction. Additional trials check the actual Drop-marker chain,
RFC5869/4231, old/new byte parity including 8160 bytes and error boundary,
HMAC/SimpleHmac/reset variants, legacy DRBG reseed and RANDOM principal/reseed
derivation. This establishes neither native RANDOM nor complete erasure acceptance.

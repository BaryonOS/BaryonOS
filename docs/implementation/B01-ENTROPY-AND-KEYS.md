# B01 — native entropy and key management (profile `native-keyring`)

Date: 29 October 2026. Status: **design contract, subsequently implemented
in the same assignment**; executed checks and their outcome are recorded
exclusively in [B01-ACCEPTANCE](../verification/B01-ACCEPTANCE.md).
Deviations that proved necessary during the build are recorded in §7.
Criterion B01 (docs/progress.json): *native cryptographic entropy, key
lifecycle, protection and recovery use controlled standards and
tested failure paths.* Fulfils P02 (entropy) from
[VERIFIABLE-SECURITY-AND-PQC](VERIFIABLE-SECURITY-AND-PQC.md) §5 and fixes the
Key Service boundary from [ENCRYPTED-OBJECT-STORAGE](ENCRYPTED-OBJECT-STORAGE.md)
§5 natively. Decision: ADR-043.

## 1. Problem

Until now the own OS has no entropy source and no keys. Without a
tested source with readiness and an error path, no key may be
generated (SECURITY-AND-QUANTUM.md §7). B02 (authenticated storage), B03
(signed updates), N03 and P01 all need a Key Service that
never hands keys to ordinary apps.

## 2. Security invariants (first)

1. **Entropy is a capability.** Only the Key Service holds an
   `Entropy` capability (right `ENTROPY` = 1024, object type without
   `DERIVE`/`TRANSFER`/`REVOKE`). `entropy_read` without that capability gives
   `EBADHANDLE`/`ETYPE`/`EACCESS`. The kernel passes raw device bytes through and
   mixes, caches or reuses nothing.
2. **No key before readiness.** The service only becomes `READY` after 1024
   bytes of startup entropy that pass the SP 800-90B health tests
   (Repetition Count and Adaptive Proportion, §3). An absent, failing or
   stuck source leaves the service in `BLOCKED`: every key operation gives
   `NotReady`. There is no weaker fallback option and no test seed in the
   product path.
3. **Keys do not leave the service via app ingress.** Apps only get
   key handles (domain, index). SEAL/UNSEAL happen in the service; the
   domain authority comes exclusively from the kernel-bound ingress endpoint.
   A handle of domain A is invalid in domain B. Only the separate
   manifest-bound management ingress may hand out recovery material.
4. **No own cryptography.** Only maintained RustCrypto crates with
   pinned versions: `chacha20poly1305` 0.11.0 (XChaCha20-Poly1305 AEAD,
   RFC 8439/draft-irtf-cfrg-xchacha), one production `chacha20` 0.10.2
   dependency (DRBG and RNG) and `cipher` 0.5.2 with wrapper zeroization, plus
   the protected HKDF/SHA-256 stack. `chacha20` 0.9.1 remains exclusively a
   development oracle for byte-for-byte compatibility. All x86 SIMD backends
   are disabled
   (`chacha20_backend="soft"`,
   `poly1305_backend="soft"`, `sha2_backend="soft"`): the
   profile has `CR0.EM=1` and no SSE. The DRBG is the
   fast-key-erasure construction on ChaCha20 (Bernstein, 2017): key K,
   keystream → new K and output, old K erased.
   Current dependency provenance and the portable-backend evidence boundary
   are recorded in [KEYRING-CRYPTO-DEPENDENCIES.md](KEYRING-CRYPTO-DEPENDENCIES.md).
5. **Binding.** Every AEAD operation binds format version, domain,
   key index and purpose (seal/recovery) in the AAD. A ciphertext of key
   A under key B, a modified byte, tag or nonce gives `Auth` without
   partial plaintext.
6. **Recovery without escrow.** EXPORT_RECOVERY delivers a random 256-bit
   recovery secret and a blob that seals the master key plus revocation status under
   that secret exclusively via a separate management ingress to
   the manifest-named supervisor/recovery principal. Ordinary app ingress rejects
   EXPORT_RECOVERY and RESTORE, even if the app knows the valid wire opcodes.
   RESTORE in another service instance thereby re-derives all domain keys.
   Without the secret the blob is unusable; the service does not keep the secret.
7. **Revocation.** REVOKE erases the derived key and marks the index; the
   index is not reused. Recovery under the same master key merges
   the existing and imported revocation bitmaps. A different master key
   may only be imported into an unused instance without issued/imported
   indices; otherwise `Invalid`. An old blob must not roll back an
   existing revocation or make an index issuable again.
   This applies within the live instance. A cold new service cannot, without
   trusted persistent history, derive a newer revocation from an old
   recovery blob; that rollback boundary remains open for B02.
8. **Kernel stays small.** No cryptography in the kernel; the kernel broker
   knows only the virtio-rng queue, DMA ownership, timeouts and the
   `Entropy` capability (kernel/AGENTS.md).

## 3. Entropy source and health tests

- Device: virtio-rng-pci (modern, PCI 00:06.0, `-object rng-random,
  filename=/dev/urandom`), own driver `drivers/virtio-rng` on `virtio-core`
  (capability parsing, one queue of 8, one device-writable buffer
  of 256 bytes per request, pinned DMA pages). No interrupts; polling
  with deadline in ticks like the block broker.
- Syscalls `0x60 entropy_request(cap, len)` → ticket and `0x61
  entropy_poll(cap, ticket, ptr, len)` → bytes: `len` 1..=256; the broker
  holds one request at a time (`EAGAIN` for a second) and delivers exactly the
  bytes written by the device (shorter delivery is `EIO`, never
  padded); `-14 Offline` without device, `-13 Timeout` after 64 ticks.
  Errors leave the output buffer unchanged; after a successful poll the
  kernel erases its copy. Like the block broker, the kernel never waits in a
  syscall for the device.
- Startup: 4 × 256 bytes. Repetition Count Test with cutoff C = 1 +
  ⌈20 / H⌉ with a conservatively assumed H = 4 bits per byte → C = 6: six
  equal consecutive bytes fail. Adaptive Proportion Test with window
  W = 512 and cutoff 62: the smallest c with P[Bin(512, 2⁻⁴) ≥ c] ≤ 2⁻²⁰
  (exactly recomputed binomially in `tests/test_keyring_harness.py`). Failure →
  `BLOCKED` with reason; no retry without a new start.
- Continuous: every later refill (256 bytes per reseed, after every 64
  key operations) goes through the same tests with persistent state from the first startup byte.
  RCT runs across all buffer boundaries; APT windows of 512 bytes continue across
  two reseeds of 256 bytes. Failure blocks further key operations with
  `Degraded`; no new `Health` per reseed.
- Test fixture `test-keyring-stuck`: the kernel replaces the device bytes with
  a constant before the copy to the service. This is a labelled
  fault injection to prove the health test, not a product path.

## 4. Key hierarchy and operations (IPC v1, tag `KEY1`)

```text
master (32 B, uit DRBG)          ──HKDF-SHA-256(info = "nieuw-os key v1" ‖ domein ‖ index)──> DEK[domein][index]
recovery_secret (32 B, uit DRBG) ──XChaCha20-Poly1305(aad = "nieuw-os recovery v1")──> blob = nonce ‖ enc(master ‖ revoked-bitmap) ‖ tag
```

| Scope | Op | Request | Response | Errors |
|---|---|---|---|---|
| App domain | STATUS | – | state (0 starting, 1 ready, 2 blocked, 3 degraded), reason, number of keys | – |
| App domain | GENERATE | – | handle (index 0..15 within the domain) | NotReady, Quota (16 per domain) |
| App domain | SEAL | handle, plaintext ≤ 128 B | nonce ‖ ciphertext ‖ tag (≤ 168 B) | NotReady, BadHandle, Revoked |
| App domain | UNSEAL | handle, nonce ‖ ciphertext ‖ tag | plaintext | NotReady, BadHandle, Revoked, Auth |
| App domain | REVOKE | handle | – | BadHandle |
| Management | EXPORT_RECOVERY | – | secret (32 B), blob (76 B) | NotReady |
| Management | RESTORE | secret, blob | number of recovered domains | NotReady, Auth, Invalid (different master in used instance) |

The service keeps a counter and the revocation bitmap per domain; the
domain number follows from the endpoint on which the app request arrives. The
management ingress accepts only the manifest-named caller, without
delegated capabilities. Requests with a capability, wrong tag or
length are refused without effect.

## 5. Acceptance cases (real QEMU runs)

| Case | Feature | Required observation |
|---|---|---|
| normal | `native-keyring` | service A and B become READY (startup entropy passed); app in domain 0 and 1: GENERATE, SEAL/UNSEAL round trip, cross-domain handle refused, manipulated byte/tag/nonce refused without plaintext, REVOKE → Revoked, 17th key → Quota; app ingress refuses EXPORT_RECOVERY and RESTORE without handing out recovery bytes or changing state. The separate management wire test exports and restores recovery material via the management dispatcher; this is host evidence, not native caller authentication. Kernel: only the two services read entropy, app requests for entropy → refused |
| no-device | `test-keyring-no-device` (QEMU without virtio-rng) | `entropy_read` → Offline; both services BLOCKED; app: every GENERATE → NotReady; kernel: 0 successful entropy deliveries, 0 keys |
| stuck | `test-keyring-stuck` | labelled constant-byte fixture after real completion; Repetition Count fails at startup; BLOCKED with reason; no keys |
| timeout | `test-keyring-timeout` | local silent rng-egd backend receives the real device request but delivers nothing; deadline, reset ACK, 0 bytes delivered, 0 keys and retention of the pinned DMA pool |
| restart | `test-keyring-restart` | normal source; after the key tasks a controlled reset ACK, reinitialisation of the same DMA pool and a new request of 32 bytes with real completion |

Markers `KEYRING:MANIFEST:PASS`, `KEYRING:READY`, `KEYRING:CASE:<X>:PASS`,
`KEYRING:RESOURCES:PASS`, `KEYRING:PASS`, `M1:PASS`; numeric evidence
`KEYRING:<NAAM>:<20 cijfers>`. Gate `tools/test_keyring.py`: host tests
(`keyring-core` with RFC 8439/draft-xchacha and RFC 5869 test vectors, the
90B tests with synthetic sequences, wire codec), then the five runs on one
frozen source.

## 6. What B01 does not prove

No hardware protection of the master key (TPM), no password recovery
(Argon2id follows later with measured parameters), no persistent key storage
on disk (B02), no SMP, no physical RNG quality (QEMU supplies
`/dev/urandom` from the host: the health tests check behaviour, not the
actual minimum entropy of real hardware), no side-channel resistance of
the software backends beyond the constant-time implementation of the crates themselves.

## 7. Deviations during implementation

- **Asynchronous broker.** One syscall that waits for the device would block
  the kernel; therefore two syscalls (request/ticket and poll), like the
  block broker. Two services share one broker slot and retry on `EAGAIN`.
- **Count in the normal case.** The app generates 2 + 1 + 14 = 17 keys
  (after which the quota of 16 per domain gives `Quota`), seals and opens once.
  Recovery via the management wire test is a separate host test; the
  old evidence for app ingress is not used as evidence for this boundary.
- **Code budget.** The service with the portable crypto backends is 50 KiB of code;
  the example uses its own linker script with 128 KiB of code from
  `0x40020000` and a separate entry page at `0x40000000`, within the
  existing user limits of the kernel (`0x40000000–0x60000000`).

## 8. Remediation review 29 October 2026

The post-merge P1 findings on #52 reopened B01. In addition to the native cases,
regressions must test old blobs after revocation, different domains, refused
master switch while retaining existing keys, and RCT/APT across reseeds.
Owner loss with an open RNG request stops the driver and shuts down the broker;
the DMA pool stays in quarantine until reboot. Timeout checks the reset ACK
with a deadline; without a confirmed reset the pool stays pinned and the broker
offline. Requester crash and missing reset ACK are not natively tested. A timeout is a different case from the constant-
byte fixture; the separate restart trial does not prove recovery of a
stuck or physical device. Execution exclusively in the new evidence.

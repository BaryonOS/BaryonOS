# Key service cryptography dependency record

Status: active build contract for the Key Service and every workspace that
locks it.

## Pinned components and provenance

- `chacha20poly1305 = 0.11.0` provides XChaCha20-Poly1305. The manifest disables
  default features and enables `zeroize`; the crate tag is
  [`chacha20poly1305-v0.11.0`](https://github.com/RustCrypto/AEADs/tree/chacha20poly1305-v0.11.0).
  The tag has no separate GitHub Release object. Its declared minimum Rust
  version is 1.85 and its license is `Apache-2.0 OR MIT`.
- The direct DRBG dependency remains `chacha20 = 0.9.1`, with default features
  disabled and `zeroize` enabled. The reviewed upstream revision is
  [`e37a978ccf0992d9053fbc039470d6527108e393`](https://github.com/RustCrypto/stream-ciphers/commit/e37a978ccf0992d9053fbc039470d6527108e393).
- Every owned lockfile records the crates.io checksum for
  `chacha20poly1305 0.11.0` as
  `9b89e1c441e926b9c82a8d023f6e1b7ae0adcfaa7d621814e4d60789bac751cb`.

Compile-time trait assertions require both `XChaCha20Poly1305` and the direct
`ChaCha20` object to implement `ZeroizeOnDrop`. Removing either feature must
therefore fail compilation instead of silently weakening key cleanup.

## Freestanding build contract

The x86_64 freestanding profile has no SSE or AVX state. Its builders must pass
all three portable-backend selectors: `chacha20_force_soft` for direct
`chacha20 0.9.1`, `chacha20_backend="soft"` for the AEAD's newer ChaCha
dependency and the maintained RNG, and `poly1305_backend="soft"` for
`poly1305 0.9.1`. A `target_os = "none"` compile without the Poly1305 selector
is rejected by an explicit compile guard.

Host tests establish API compatibility, published vectors, authenticated
failure behavior, persistent-state invariants, and byte-exact legacy DRBG
output. Locked freestanding builds and object inspection separately establish
the selected backend and absence of SIMD instructions; neither is a claim
about physical hardware behavior.

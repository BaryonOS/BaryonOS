# Targeted RustCrypto memory-hygiene patch

These are local runtime variants of **HKDF0.13.0** and **HMAC0.13.0**, with
only buffer lifetime/erasure, trait checks and dependency
feature wiring changed. The KDF/MAC algorithms, labels, byte order and formats
remain upstream. This is not an upstream release or an independent crypto audit.

| Upstream archive | SHA256 |
|---|---|
| hkdf-0.13.0.crate | 4aaa26c720c68b866f2c96ef5c1264b3e6f473fe5d4ce61cd44bbe913e553018 |
| hmac-0.13.0.crate | 6303bc9732ae41b04cb554b844a762b4115a61bfaa81e3e83050991eeb56863f |

The original official source archives are in `upstream/`; their original
source can be read directly from the archive. Each unpacked variant
retains `Cargo.toml.upstream`, `Cargo.toml.orig`, README and the full
MIT/Apache-2.0 licences with original author attributions.
The small source diff is in
[`hygiene.diff`](../verification/hkdf-hygiene/hygiene.diff).
The local Cargo manifests select only the libraries; upstream tests and
benches remain available in the original archives. Local regressions and
old-version oracles are under `../tests/`.

The wrappers erase in normal return/drop paths:

- The discarded PRK in `Hkdf::new` and the optional KDF trait adapter.
- Each HKDF expand block and its previous block, also on replacement/end.
- The normalized HMAC key block, ipad/opad and the long-key hash.
- The internal HMAC hash, also in both reset variants.
- Retained SimpleHmac ipad/opad via `Zeroizing`.

For retained HMAC hash states, `sha2=0.11.1/zeroize` is **also** required.
The upstream SHA2 Drop erases state/block_len; digest/block-buffer erases the
pending buffers. `ZeroizeOnDrop` markers are only available when
the hash/core actually selected has the same marker. The production library
checks this concrete SHA256→HMAC→HKDF chain at compile time.
A removed SHA2 zeroize feature must make that compilation fail.

Per upstream, `extract` deliberately returns a PRK to its caller;
that caller must protect its own returned bytes. `new` erases the
PRK that this API itself discards. The production code uses `new`/`from_prk`.
HMAC/HKDF output to a caller is not unexpectedly erased. The tests
check byte parity with the original implementations.

This intervention claims no complete erasure: deep SHA compression temporaries,
Rust moves, compiler/register copies and abort/crash paths have not been fully
investigated or erased by this patch. No reading of freed memory
or unsafe object overwriting is used as erasure evidence.

On an upstream update, archive provenance/licences, patch, active
graph/features, marker chain, RFC/KAT/byte parity and scalar1.90-nonebuilds
must be rechecked. SHA2 is used directly from the registry;
its algorithm source has not been modified locally.

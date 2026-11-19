# Bounded safetensors header parser (host prototype)

This standalone, allocator-free `no_std` crate forbids unsafe code. It parses
only safetensors header/table observations from an existing
`model-store-contract::VerifiedArtifact`. It does not open an unverified
artifact, download a model, run inference, authorize access, or establish
publisher trust, model safety, hardware support or native acceptance.

## Contract and profile

The owner supplies a verified handle, store, `ReadBudget`, cancellation object,
header scratch buffer and tensor scratch table initialized with `Tensor::EMPTY`.
`parse` reads the eight-byte little-endian length followed by exactly the
header bytes using the same handle's `read_at`. Positive short reads advance
by the returned count; requests never exceed the store's 4096-byte bound.
Requested bytes, including failed/short attempts, retain the existing budget
charges. Store errors retain their exact model-store error through `Error::Store`.
No implicit reopen, new generation, network request or data-area read occurs.

Only an `Ok(Model)` exposes a complete validated table. Scratch buffers contain
unspecified partial data after an error and must not be used as a valid model.
Names borrow the header, and the result borrows the tensor table. Tensors are
sorted by `(start, end)` offsets rather than JSON order. The model records the
exact descriptor and data-area start; it does not own or extend the artifact
handle's lifetime. The owner must retain that same immutable generation and
recheck its existing access/revocation contract on later reads. Parsing is
fallible even after content-integrity verification. Cancellation is checked by
the read contract and at parser/table boundaries; parser-only cancellation does
not itself consume the owner's handle.

The fixed schema has no recursive descent and a maximum nesting depth of three:
root object, tensor object, shape/offset array. Metadata is one object containing
only string keys/values. Unexpected containers are rejected without traversal.
Unknown tensor fields and missing required fields are rejected. Top-level tensor
names, tensor fields and metadata keys must be unique; `__metadata__` is reserved
at the root and may appear once. Metadata is validated but not returned.

This deliberately limited format profile accepts:

- At most 1 MiB of header bytes, 4096 tensors, rank eight, and 64 metadata entries;
  caller scratch capacity can impose smaller limits. These are parser limits,
  not model-store or OS capacity limits.
- Strict UTF-8 strings with raw Unicode. All JSON escapes, including Unicode
  escapes, are rejected explicitly; escaped spellings cannot bypass duplicate
  detection. Unescaped control bytes are rejected. This profile does not accept
  every otherwise valid safetensors header.
- Unsigned decimal `u64` dimensions/offsets without leading zeros, signs,
  exponents or fractions. Standard JSON whitespace is allowed within objects;
  the header must start with `{` and only ASCII spaces may pad its end.
- `BOOL`, `U8`, `I8`, `U16`, `I16`, `U32`, `I32`, `U64`, `I64`, `F16`, `BF16`,
  `F32`, `F64`. Packed/sub-byte and float8 formats are unsupported. No tensor
  value validation, floating-point computation or alignment guarantee is made.

Scalars (rank zero) contain one element. Any zero dimension makes the tensor
empty, including when other dimensions would overflow a nonempty product.
Other element-count and byte-size multiplication is checked. Offsets must be
ordered, within the data area and agree exactly with dtype/shape byte size.
Sorting permits arbitrary object order. All sorted intervals must connect from
zero through the complete data area, rejecting overlap, holes and trailing bytes.
Empty intervals are accepted at data boundaries. Neither names nor offsets
confer executable or inference authority.

The format reference is the upstream
[safetensors format description](https://github.com/safetensors/safetensors#format).
The parser is independently implemented; no upstream code is copied. No new
third-party dependency is introduced: production uses existing local store
crates; fixture digest creation uses the same pinned `sha2 = 0.11.1` as
model-store-contract. The lockfile retains its existing registry versions/checksums.

## Checks and limits

Use Rust 1.90.0 with this manifest for `test --locked`, `clippy --locked
--all-targets -- -D warnings`, `fmt -- --check`, and `build --locked --release
--lib --target x86_64-unknown-none`. For a scalar freestanding build, use the
existing component workflow's disabled SIMD target features plus
`--cfg sha2_backend="soft"`; this avoids runtime CPU dispatch in the inherited
SHA-2 implementation. No cryptographic implementation or pin is changed.

Tests use integrity-verified synthetic fixtures with Qwen-style tensor names,
short reads and maximum-header chunking. Negatives cover the profile bounds,
UTF-8/JSON, duplicate/escaped keys, dtype, arithmetic, exact coverage, truncation,
store failures, generation mismatch, cancellation and budget charging/retry.
This is host parser and freestanding compilation evidence, not inference, a
real-model compatibility result, native store execution or provenance acceptance.

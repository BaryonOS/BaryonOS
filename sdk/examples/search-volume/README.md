# Local search over the volume service (U05-T3 S1)

This `no_std` library drives `sdk/search-index` from replies of
`volume_core::client::Client`. It indexes the names and collection labels of
the caller's own volume domain. Evidence level: host tests only. There is no
native binary yet; that is step S2.

It is a standalone workspace with its own `Cargo.lock`, and it is not a member
of `sdk/Cargo.toml`. The checks are the following:

```sh
M=sdk/examples/search-volume/Cargo.toml
cargo +1.100.0 test --locked --manifest-path $M --target-dir "$PWD/target"
cargo +1.100.0 clippy --locked --manifest-path $M --target-dir "$PWD/target" --all-targets -- -D warnings
cargo +1.100.0 fmt --manifest-path $M --all -- --check
python3 -B -m unittest tests/test_search_oracle.py
```

## Contract

The lead decisions B1–B4 define this contract.

- `Client::validate_reply` is the primary check on every reply. A malformed
  reply returns `Protocol`, and nothing is published. `check_entry` repeats
  those checks as defence in depth only. Collections 0 to 4 are valid, where
  0 means no collection.
- Every rebuild runs LIST, then every COLLECTION_STAT, then LIST again. The
  rebuild publishes only when both LIST generations are equal and are strictly
  newer than the last applied generation.
- RENAME and DELETE update the index with the revision that the service
  returned. WRITE, COLLECTION and META force a full rebuild.
- If the service applied a mutation but the index did not follow it, queries
  return `RebuildRequired` until a rebuild succeeds. A mutation that the
  service refused changes nothing in the index.
- A `Unknown` or `Timeout` result revokes the grant. The epoch is the volume
  epoch and is fixed for the boot, so a re-grant returns `Denied`. After a
  revoke, the adapter sends no further request.
- The bounds are `Index<E=8, B=64, T=32, P=8>`. A 32-byte name and a 32-byte
  label can each hold at most 16 tokens. With B = 64, the byte bound is always
  reached before T, so `TokensFull` cannot occur.

## Shared vectors

The Rust tests and `tests/test_search_oracle.py` both read `vectors.tsv`. The
oracle reimplements Rust `split_whitespace` and ASCII-only case folding, and
does not run Rust. Agreement between the two is therefore only transitive,
through this file.

## Limits

- The grant is issued in the same untrusted app process, so natively it
  carries no security value. Cross-domain isolation comes from the endpoint
  binding of the volume service.
- The generation is the sum of the object and collection revisions in the
  domain. The adapter never sends PURGE. If a revision could ever decrease,
  rebuilds would return `StaleGeneration` until a reboot.
- There are no content tokens, no persistence, no native revocation and no UI.

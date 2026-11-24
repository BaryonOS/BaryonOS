# Local search over the volume service (U05-T3 S1)

This `no_std` library drives `sdk/search-index` from replies of
`volume_core::client::Client`. It indexes the names and collection labels of
the caller's own volume domain. The library evidence level is host tests.
Step S2 adds the native `search-app` binary (`src/main.rs`). Its evidence level
is a cross-build only, because no kernel case runs it yet (step S3).

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
  rebuild publishes only when both LIST generations are equal. A clean index
  requires a strictly newer generation than the last applied one. A dirty index
  may rebuild at that same generation because an uncertain mutation reply need
  not have advanced it; an older generation is always refused.
- RENAME and DELETE update the index with the revision that the service
  returned. WRITE, COLLECTION and META force a full rebuild.
- If the service applied a mutation but the index did not follow it, queries
  return `RebuildRequired` until a rebuild succeeds. A mutation reply that
  fails validation (`Protocol`) counts as possibly applied and has the same
  effect. A mutation that the service refused changes nothing in the index.
- A `Unknown` or `Timeout` result revokes the grant. The epoch is the volume
  epoch and is fixed for the boot, so a re-grant returns `Denied`. After a
  revoke, the adapter sends no further request.
- The bounds are `Index<E=8, B=64, T=32, P=8>`. A 32-byte name and a 32-byte
  label can each hold at most 16 tokens. With B = 64, the byte bound is always
  reached before T, so `TokensFull` cannot occur.

## Native app (S2, cross-build only)

`search-app` runs as both app slots of the `native-volume` broker profile, one
per volume domain. It is planned as kernel case 25 (`test-volume-search`, marker
`VOLUME:CASE:SEARCH:PASS`). Case 26 (cold-restart recovery) is deferred (lead
decision m5). The app does the following:

1. Probes the block device with its own endpoint handle and with a forged
   handle. Both must be refused, as in the settings app.
2. Sends a raw STAT and a raw LIST for the other domain on the bare `Client`.
   The service must refuse both with `Access`.
3. Creates collection 1 `Projects` and three objects named
   `<token> notes|draft|budget`, where the token is `alpha` (domain 0) or
   `beta` (domain 1). Each object holds 16 bytes of volume fixture content.
4. Assigns the notes object to collection 1, renames the collection to
   `Archive` (the old label must give 0 hits), renames the draft to
   `<token> final` and deletes the budget object.
5. Runs these negative checks:
   - a stale-revision RENAME is refused by the service (`Stale`) and leaves the
     index unchanged;
   - the other domain's token gives 0 hits;
   - a 65-byte query gives `TextFull`, an empty query gives `EmptyQuery`, and
     invalid UTF-8 gives `InvalidUtf8`;
   - a foreign object gives `Foreign` without a request;
   - a rebuild without a change gives `StaleGeneration`.

`TokensFull` is unreachable for queries with B = 64. The index-level
`StaleRevision` is host-tested only, because the adapter always applies the
revision that the service returned.

The app reports these startup words:

| Word | Content |
|---|---|
| 400 | Own-token hits, which equals the live documents |
| 448 | Chained digest over the fixed query set (`tools/run_search.py` `queries`) |
| 480 | Own-token hits |
| 488 | Other-token hits |
| 496 | Negative-check mask (`0xfff` when complete) |
| 240, 40 | Failing step and status, as in the settings app |

`tools/run_search.py` recomputes words 400–488 from the after-image with
`tests/test_search_oracle.py`. It also requires the following:

- the full marker chain and a raw exit of 33;
- the exact scripted volume state;
- the complete mask;
- at least 48 requests per slot;
- exactly 4 `SERVICE:REJECTED`, which are the foreign probes.

The runner exits before booting until the S3 wiring exists.

`src/main.rs` reuses `../volume-docs/src/support.rs` through `#[path]`; that
file is byte-identical to the settings-volume copy. There is no `build.py` or
`linker.ld` in this crate yet; S3 adds them with the `tools/build.py` wiring.
Until then, the cross-build uses the flags of settings-volume `build.py` with
`sdk/examples/volume-docs/linker.ld`, which is byte-identical to that copy:

```sh
CARGO_ENCODED_RUSTFLAGS="$(printf '%s\x1f' -C code-model=small -C relocation-model=static \
  -C no-redzone=yes -C linker=rust-lld -C link-arg=-T$PWD/sdk/examples/volume-docs/linker.ld \
  -C link-arg=--no-pie -C link-arg=--build-id=none -C link-arg=-zmax-page-size=0x1000 | sed 's/\x1f$//')" \
cargo +1.100.0 build --offline --locked --release --manifest-path $M --features native \
  --bin search-app --target x86_64-unknown-none --target-dir "$PWD/target"
```

Measured at opt-level 3 (S2, 23 Nov 2026):

- `.text` is 0xb6c5 (limit 0x10000);
- `.data` is 0x438 and there is no `.bss` (limit 0x1000);
- `.rodata` is 0x10e8 (limit 0x8000).

The static worst-case stack, summed from the `sub rsp` frames along the deepest
direct-call chain in the disassembly, is about 47 KiB of the 64 KiB app stack.
That chain is `run` (32 KiB, which holds the 10.5 KiB `Search`), then
`Search::rebuild`, `Index::rebuild` (11 KiB, a second index), `Entry::hash` and
SHA-256. This is an estimate, not a measured high-water mark.

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

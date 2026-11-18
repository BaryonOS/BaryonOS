# PR340 — source-bound read-only disposition

Reviewed source: `93797f3a6075af9cc4fdd533dad288bfa7894ffe`. Claude retains this workstream; this review grants no write, runner or merge authority.

- Portal Clippy: the same `length % SECTOR as u64 == 0` remains at sdk/examples/volume-docs/src/app.rs:1056. The stored Rust1.90 check with `native,archive-portal` reports exit101/manual_is_multiple_of. The bounded authorized repair is `length.is_multiple_of(SECTOR as u64)` while retaining `length > 0`, followed by the exact Clippy variant and fresh native gate. This review did not rerun Clippy.
- A: narrow the broad Windows/macOS/Linux name-safety wording to the checked rules. The specification already excludes8.3 aliasing/path-length coverage; do not claim general platform safety.
- C: existing zero-sink host tests and the native complete-stream/write/flush/outside-sector oracle provide bounded indirect evidence; no new counting-sink architecture is required for this PR.
- D: opt-level=s covers all portal archive stages and was present in stored run14. Preserve it; old opt-level3 results do not prove this variant.
- H: checking exact pax extension name/type in the oracle is a small verification improvement, not a demonstrated vulnerability in the current fixed producer.
- Original blocker B was fixed by casefolded PaxHeader rejection and counting-sink tests; do not report that stale finding as open.

Stored run14 is source18da1eb9/fingerprint36cab793. Current head also merges tools files from main524c44b7, and source_hash includes tools recursively. Do not label run14 exact-head evidence for93797f3a. After a permitted repair, capture a fresh gate/fingerprint. D1–D5, encryption and physical/human product acceptance remain separate requirements, not automatic blockers for the bounded D6(a) slice.

Disposition: preserve the PR and all WIP. Current merge completion requires an authorized workstream writer, the bounded lint repair and current source-bound verification. No source or runner was changed by this review. No new concrete security blocker or human audit is claimed.

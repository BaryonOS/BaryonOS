# R68 renderer review fixes

Status: host-only regression evidence; native compositor and assistive-service
acceptance remain open.

## Source and scope

- Frozen review base: `e382a989d4b28a45a23e31ff4edef2426e9c2616`
- Work branch: `codex/native-render-review-fixes-20261101`
- Changed runtime: `ui/native/src/render.rs`
- Regression tests: `ui/native/tests/render_regressions.rs`
- Existing assertion updated only near line 597 in
  `ui/native/tests/interaction.rs`
- No Cargo manifests, lockfiles, kernel, SDK, compositor, shared tokens, or
  accessibility IPC changed.

## Findings and fixes

1. The semantic output buffer now has capacity for the volume-domain maximum
   plus the possible paging controls: `8 + PER_DOMAIN` nodes. `Frame::semantics`
   traverses the same bounded node builder once to count the exact result,
   rejects an oversized or undersized request before writing, and traverses
   again to fill the caller buffer. A short buffer therefore remains untouched.
   At 800×720, 100%, with eight objects, the regression asserts seven visible
   list items, both paging controls, the status node, and exactly 15 nodes.
   Fourteen output slots are rejected atomically. The existing 1920×1080
   interaction case separately asserts its actual 14 nodes; maximum capacity
   is not confused with the count for that viewport.
2. The Window semantic bounds are already in physical viewport coordinates.
   They are no longer scaled a second time; logical control bounds still receive
   exactly one scale conversion. Regressions assert 800×600 physical bounds at
   both 100% and 200%.

## Reproduction and verification

On the frozen unmodified source, the new test file failed both cases:

```text
semantics_budget_includes_paging_and_preserves_short_output
  panicked at src/render.rs:533:17: index 14 out of bounds for length 14
window_semantic_bounds_match_physical_viewport_at_100_and_200_percent
  expected width 800, received 1600 at 200%
```

Red log: `/tmp/nieuw-os-render-red.log` (test process exit 101).

The fixed source was verified with Rust 1.90.0, using the existing offline
Cargo cache and host linker; no packages were installed. Exact commands and
results:

| Command | Result | Log |
|---|---:|---|
| `python3 -B tools/dev.py cargo fmt --manifest-path ui/native/Cargo.toml -- --check` | exit 0 | `/tmp/nieuw-os-render-fmt.log` |
| `python3 -B tools/dev.py cargo test --offline --locked --manifest-path ui/native/Cargo.toml` | exit 0; 20 interaction tests + 2 renderer regressions | `/tmp/nieuw-os-render-tests.log` |
| `python3 -B tools/dev.py cargo check --offline --locked --manifest-path ui/native/Cargo.toml --tests` | exit 0 | `/tmp/nieuw-os-render-check.log` |
| `python3 -B tools/dev.py cargo clippy --offline --locked --manifest-path ui/native/Cargo.toml --all-targets -- -D warnings` | exit 0 | `/tmp/nieuw-os-render-clippy.log` |

SHA-256 of the fixed test inputs:

```text
52fce85611498d4d33f96a64d83205045ab6ccd9264c291227d3ff27f1a4e234  ui/native/src/render.rs
b2aa1a3e6629e0228cb61415e8397bd8420cc3cb72a585de00e0d5f7588f07d6  ui/native/tests/interaction.rs
27ed99f5a7cd74fdd79a21dd3461a148a46e9e05ae24e66cb520546273446a77  ui/native/tests/render_regressions.rs
```

## Evidence boundary

These are host tests of the scalar renderer and semantic-tree construction.
They do not execute the changed source in a guest, demonstrate a native
compositor, expose the app-local tree to a screen reader, or constitute a
human accessibility review. No U01–U06 criterion or progress point is closed
by this change.

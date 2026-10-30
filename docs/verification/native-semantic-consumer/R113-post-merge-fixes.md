# R113 post-merge UI fixes

Status: host-only regression evidence. This note makes no claim about the
native compositor, QEMU or the assistive service. Those acceptance steps stay
open.

## Source and scope

- Review base: `e08fb0e` (merge of #113)
- Work branch: `claude/fix-113-ui`
- Changed runtime: `ui/native/src/controller.rs`
- Regression tests: `ui/native/tests/interaction.rs`. Three new
  `scenario!` tests and one helper, `assert_single_live_focus`, were added.
  No existing test was changed or removed.
- No Cargo manifest, lockfile, dependency, `unsafe`, SDK, kernel or
  capability path changed.

## Findings and fixes

1. **Major: SaveAndBack BEGIN failure with `can_save` still true.**
   `activate(SaveAndBack)` switches to the Editor screen before it calls
   `save()`. On a quota error, `failed()` kept focus on `SaveAndBack`, but
   that control only exists in the close dialog. The result was no focused
   semantic node, Enter returning `InvalidInput`, and typed keys being
   dropped. Fix: when saving is not in progress, the screen is not
   `ConfirmClose` and focus is `SaveAndBack`, `failed()` now moves focus to
   `Save`. The existing rule still applies first: if `can_save` is false,
   focus moves to `Document`.
2. **Minor: read-only back focused the disabled New control.**
   `request_back()` now picks the same initial focus as the constructor:
   `New` when the mode is ReadWrite, otherwise `Refresh`.
3. **Minor: a pending pointer gesture survived a semantic action.**
   `activate_semantic` and `set_semantic_value` now clear `pressed` and
   `dragging` once the live semantic tree has accepted the action. This
   matches what `request_back`, `resize`, `rebind_input` and
   `cancel_pointer` already did. A mouse release can therefore no longer open
   whatever object moved into the pressed list index after a semantic
   Refresh, and a held drag can no longer extend a selection over replaced
   text.

## Before the fix (red)

The three tests were run against the unmodified controller at `e08fb0e`:

```
cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml --test interaction r113
exit=101
test r113_read_only_back_focuses_enabled_refresh ... FAILED
test r113_semantic_action_cancels_pending_pointer_gesture ... FAILED
test r113_save_and_back_begin_failure_refocuses_editor_control ... FAILED
  r113_read_only_back...:      assert_eq!(focus, Refresh)        left: New
  r113_semantic_action...:     assert_ne!(contents, Some("zero")) left: Some("zero")
  r113_save_and_back...:       assert_ne!(focus, SaveAndBack)     left: SaveAndBack
test result: FAILED. 0 passed; 3 failed; 28 filtered out
```

A separate mutation check covered the drag half of test 3. With every fix
applied except the clearing in `set_semantic_value`, the test failed with
`stale drag changed the selection  left: 0  right: 13` (exit 101). The full
fix was then restored.

## After the fix (green)

All commands below were run with
`CARGO_TARGET_DIR=/home/simon/projects/target-fix113/ui`, unless the command
shows a different directory.

| Command | Exit |
| --- | --- |
| `cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml --test interaction r113` (3 passed) | 0 |
| `cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml` (interaction 31 passed, render_regressions 4 passed) | 0 |
| `cargo +1.90.0 clippy --offline --locked --manifest-path ui/native/Cargo.toml --all-targets -- -D warnings` | 0 |
| `cargo +1.90.0 fmt --manifest-path ui/native/Cargo.toml -- --check` | 0 |
| `RUSTFLAGS="-C no-redzone=yes -C target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2" cargo +1.90.0 build --offline --locked --release --lib --target x86_64-unknown-none --manifest-path ui/native/Cargo.toml` | 0 |
| `CARGO_TARGET_DIR=/home/simon/projects/target-fix113/mat cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml` (20 + 2 passed) | 0 |

## Not proven

- No native QEMU run, compositor presentation or assistive-service
  consumption was performed for this change.
- The x86_64-unknown-none build shows only that the library compiles for
  that target. It does not show that it behaves correctly at runtime.

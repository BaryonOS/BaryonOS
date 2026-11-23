# H03-T3c virtio-snd fixture hardening (host evidence, 23 November 2026)

Base `965301b4`, branch `claude/h03-snd-hardening-20261123`. Host tests and
builds only; no native execution. The kernel-only virtio-snd fixture requires
QEMU 8.2 (`pc-q35-8.2`); the local QEMU is 10.2.1, so `tools/run_virtio_snd.py`
was **not run**. Hosted `native-xhci.yml` remains the native evidence.

## Changes

| Item | Change | Host test |
| --- | --- | --- |
| 4 (PR #409 addendum) | `Stale` PCM events are charged to a separate stale tally in `pcm::EventBudget` (bound `MAX_INVALID_EVENTS` = 8 per attached device). START resets the per-STOP stale bound, so STOP/START cycles previously allowed uncounted stale events. `invalid()` and `SND:EVENTS:INVALID=` keep their meaning. A compile-time assertion keeps one STOP (`MAX_PERIODS + 1` = 5) within the bound. | `f17_only_stale_events_exhaust_the_budget_across_stop_start_cycles`, `f17_stale_tally_fences_after_bound` |
| 5 (PR #409 addendum) | `queue::Lifecycle::check_initialize` refuses initialisation unless `Offline` with no control or TX request outstanding; the kernel `Device::initialize` calls it first (`Fault::State`). Posted event buffers are excluded: they are always outstanding on a live device and are voided by the acknowledged fence, so including them would refuse every re-initialisation. | `initialize_refuses_outstanding_control_or_tx_requests` |
| d (CodeRabbit #415) | An XRUN that SET_PARAMS did not select is `InvalidEvent` also after STOP; it no longer uses the stale bound. | `stream_events_count_only_while_running_for_this_stream`, `f17_unselected_xrun_after_stop_is_charged_as_invalid` |
| e (CodeRabbit #415) | `Ring::check_visible` validates used elements already visible at the pre-publication read (distinct outstanding heads, acceptable length) before a new head is registered; the kernel `place` calls it. | `visible_completions_are_validated_before_a_new_head_is_registered` |
| f (CodeRabbit #415) | The fence acknowledgement marker is written only after `finish_fence` succeeded; the serial order on success is unchanged. | native only (not run) |
| a, b, c, g (CodeRabbit #415) | Bounded wrap-test setup loop; `channels_max` source named; F10 split into status (detected) and configuration change (not detected, open); independent literal serial fixtures in `tools/tests/test_virtio_snd.py`. | the tests themselves |

## Commands (exit status)

| Command | Exit |
| --- | --- |
| `cargo +1.90.0 test --locked --manifest-path drivers/virtio-snd/Cargo.toml` (81 passed) | 0 |
| `cargo +1.90.0 clippy --locked --manifest-path drivers/virtio-snd/Cargo.toml --all-targets -- -D warnings` | 0 |
| `cargo +1.90.0 fmt --all -- --check` and `fmt --manifest-path drivers/virtio-snd/Cargo.toml -- --check` | 0 |
| `cargo +1.90.0 build --locked --release --lib --target x86_64-unknown-none --manifest-path drivers/virtio-snd/Cargo.toml` (component-leaves RUSTFLAGS) | 0 |
| The same test, clippy, fmt and freestanding build with `+1.100.0` (hosted toolchain) | 0 |
| `python3 -B tools/build.py --kernel-feature native-virtio-snd-probe`, `test-snd-fence-reinit`, `test-snd-quarantine` | 0 |
| `env -u GIT_EDITOR python3 -B -m unittest tools.tests.test_virtio_snd` (19 tests) | 0 |

## Not run

- `tools/run_virtio_snd.py` for all five cases: requires QEMU 8.2; local QEMU is 10.2.1.

## Remaining limits

- A well-behaved device that reports stale events across two or more STOP
  cycles now spends the stale budget; the fixture performs one STOP per stream.
- `check_visible` is a sampled check with the same window as F8.
- A configuration change after DRIVER_OK is not detected (F10a, open).

# H04-T3 security review follow-up — link epoch isolation and 0x74 codes

This directory holds the 20 November 2026 follow-up to
[h04-native-link-20261119](../h04-native-link-20261119/README.md). It is a
separate source version (see the source fingerprints below); the 19 November
results stay unchanged in their own directory.

## Round 1 — 20 November 2026

An independent AI security review of H04-T3 reported three findings. Two
adversarial refuters each confirmed F1 and F2 and refuted the third. This is
AI review, not a human audit. The 19 November results are historical and were not
rerun on the old source.

- **F1 (medium): RX completion crosses a synthetic epoch pair.**
  `broker_tick` takes the link sample (`link::poll`) before it polls RX.
  A sample with a changed configuration generation and an unchanged
  `LINK_UP` bit yields a synthetic down (epoch N+1) and the actual up
  (epoch N+2). `apply` handled both back to back. `drop_rx` only destroys a
  copy that the broker already holds, so a completion that was already in
  the RX used ring before the down edge was taken by the later RX poll and
  delivered by 0x72 in epoch N+2.
- **F2 (low): ambiguous 0x74 codes after a stop.** `link::query` returned
  the broker's stop cause, so -15 meant either "no `VIRTIO_NET_F_STATUS`"
  or a quarantine or Protocol stop. A stop from the link poll could also
  return a code other than -14 (for example -12). ADR-H04-LINK-20261119
  requires -14 after any stop or latch and -15 only without STATUS.
- **Refuted: RX repost inside an open down window.** Both refuters rejected
  this report, so that path is unchanged.

**Fix** (`kernel/src/m2/m3/network.rs`, only inside the feature-gated
`link` module):

- The down branch of `link::apply` first calls a new `harvest`. When the
  broker buffer is empty, `harvest` takes a pending RX completion into it,
  with the same `poll_rx` call as `broker_tick` (`rx_completed` + 1). An RX
  poll error stops the broker through `stop()`. `drop_rx` then zeroes the
  copy, counts `NETLINK:RX_DROPPED` and reposts the same pinned page. Both
  steps run before the down window opens, so the repost is not counted as a
  down-window publication.
- `link::query` returns -14 (`error(Offline)`) when the broker is offline or
  quarantined, or when the monitor is latched, whatever the stop cause. It
  checks this before the STATUS check, so -15 now means only "no
  `VIRTIO_NET_F_STATUS`". 0x70/0x72 keep the broker's existing offline code
  (unchanged code shared with the feature-off profiles).

**Host test.**
`rx_completion_pending_at_a_synthetic_down_edge_is_harvested_there` in
`drivers/virtio-net/src/tests.rs` covers the driver half of F1 on the
register/DMA model:

- an unpolled epoch-1 completion survives a generation-only sample;
- after the synthetic pair the descriptor is still posted, so a plain repost
  returns `Busy`;
- after the harvest and the repost the bounce page is zero, and `poll_rx`
  returns nothing in epoch 3.

The kernel ordering itself is checked by source inspection only.

**Executed** from a worktree outside the main checkout (no nested
`.cargo/config.toml`, no cargo shim). The settings were
`NOS_QEMU_MACHINE=pc-q35-8.2`, a separate
`NOS_BUILD_DIR`/`NOS_TARGET_DIR`/`CARGO_TARGET_DIR` per profile, Rust
1.100.0 and QEMU 8.2.2. The commands ran sequentially on a host shared with
other native runs. The source fingerprint (v3) was
`ce09758930938029b40e2d1abb1eb46589a3ad9e873abe30c17c69502580ab68` on base
`0a0a5f5`; every run recorded this value.

| Command | Exit | Wall time |
|---|---:|---:|
| `cargo +1.100.0 fmt --all -- --check` | 0 | — |
| `cargo +1.100.0 test --locked -p virtio-net-driver` | 0 | 30 passed (1 new) |
| `cargo +1.100.0 clippy --locked -p virtio-net-driver --all-targets -- -D warnings` | 0 | — |
| `python3 -B -m unittest tools.tests.test_net_link` | 0 | 15 passed |
| `python3 -m compileall -q tools` | 0 | — |
| `python3 -B tools/build.py --kernel-feature test-system-net-link` | 0 | 16 s |
| `python3 -B tools/run_net_link.py --case normal` | 0 | 94 s |
| `python3 -B tools/run_net_link.py --case nostatus` | 0 | 95 s |
| `python3 -B tools/build.py --kernel-feature test-system-net-link-storm` | 0 | 15 s |
| `python3 -B tools/run_net_link.py --case storm` | 0 | 97 s (first attempt, no rerun) |
| `python3 -B tools/build.py --kernel-feature native-system-net` | 0 | 16 s |
| `python3 -B tools/run_net_system.py --case normal` | 0 | 94 s |

Every run had raw QEMU exit 33, `passed=True` and no timeout. The link runs
also had `link_gate=True`, and the `native-system-net` run had
`network_gate=True`. No oracle changed: the runner's exact expectations
were met.

- `normal`: 3 events, no synthetic events, `RX_DROPPED` 1, one window with
  delta 0, and 10 RX completed with 9 delivered.
- `nostatus`: `NETLINK:ATTACH:NO_STATUS`. 0x74 still returned -15, because
  the probe took its no-STATUS path and its counters were 0.
- `storm`: 9 events, then `EXHAUSTED`, and 4 windows with delta 0. After
  the exhaustion stop, 0x74 returned -14; the probe needs that code to leave
  its storm loop. Send and receive then each returned -14
  (`NETLINK_PROBE_OFFLINE` 2). 2 resets and 2 ACKs.

Kernel ELF SHA-256 values are `53eeeb7f4b0d97f2…` for the link kernel,
`4fbdf06846ab2979…` for the storm kernel and `dc61978cb34c155b…` for
`native-system-net` (built in the fix worktree).

**Feature-off identity.** `native-system-net` was built at `0a0a5f5` in a
detached worktree (`git worktree add --detach …/work-h04/base 0a0a5f5`,
with its own `NOS_BUILD_DIR` inside that tree). This first build was
compared with the build in the fix worktree. `.data` was identical, but
`.text` and `.rodata` differed (`.rodata` 410,811 B against 410,779 B).
The difference comes from the build path, not from the fix:

- the kernel embeds absolute source paths of out-of-workspace path
  dependencies (`services/volume`, `services/keyring`);
- the crate metadata hashes in the embedded ring-3 ELFs also depend on the
  path;
- a different path length shifts `.rodata`, which changes RIP-relative
  displacements in `.text`.

A second worktree with a path of the same length still differed through the
crate hashes. The decisive check therefore used **one path** for both
builds. The fix diff was applied to the base worktree and the profile was
rebuilt with the same build directory. The source fingerprint of that tree
matched the run fingerprint above. Results:

- `.text` (178,947 B), `.data` (1,725,248 B) and `.rodata` (410,811 B) are
  byte-identical;
- the whole kernel ELF SHA-256 is equal,
  `432b4a280bac1606cc3f9e17298655a1bedb04bc89534cfcf055bb277dd4f590`.

Both worktrees were reverted and removed with `git worktree remove`.

**Still not covered.**

- No native case drives the F1 trigger: two `set_link` changes inside one
  PIT tick while a completion is pending. The synthetic pair is still only
  host-tested.
- The kernel cannot tell when a completion that it first sees at an up edge
  arrived. A device that completed RX while it reported the link down,
  between the last RX poll of the down epoch and the next sample, would have
  that frame delivered in the new epoch. In these QEMU runs no RX completion
  was taken inside a down window: such a completion would have been dropped
  and reposted there, and the window delta stayed 0. This residual is left
  for the lead to assess; it is not part of F1 or F2.
- ADR-H04-LINK-20261119 (lead-owned, not edited here) names -14 in the
  0x70/0x72 precedence. The broker returns its stop cause there, which is -14
  after exhaustion or owner exit.

## Round 2 — 20 November 2026

An independent AI review of the round-1 fix (commit `f0d36a0`) reported one
blocking defect. This is AI review, not a human audit. The 19 November results,
including round 1, are historical and were not rerun on their old source.

- **Up-edge variant of F1 (blocking).** Round 1 took a pending RX
  completion only in the down branch of `link::apply`.
  `LinkMonitor::observe` inserts a synthetic pair only when the
  configuration generation changes and the `LINK_UP` bit does not. From an
  observed down epoch, hidden changes up, down, up between two samples
  therefore end as one real up edge, with no synthetic pair and no
  down-edge harvest. QEMU accepts RX while its own link is up. Trace: the
  kernel epoch N is down and RX is posted. Inside one PIT tick a QMP client
  sends `set_link on`, a peer frame F completes into the used ring, then the
  client sends `set_link off` and `set_link on`. The next sample shows the
  bit flipped and the generation 3 higher: a single up event N+1.
  `apply(up)` closed the window without touching RX. `broker_tick` then polled RX and took F,
  `received()` saw a usable link (`NETLINK:RX_HELD`), and 0x72 delivered F
  in epoch N+1. F completed before a real, hidden down edge, so this broke
  ADR-H04-LINK-20261119 item 4 and the module claim that completions
  arriving while down are discarded. It is not a regression: base `0a0a5f5`
  had the same gap.
- The round-1 "Still not covered" bullet understated this case. It said that
  a device would have to complete RX while it reported the link down. A
  conforming QEMU device reaches the case with three `set_link` changes.

**Verification of the report.** The source confirms it: the up branch of
`apply` had no RX step, and `broker_tick` calls `link::poll` before its RX
poll. The reviewer's throwaway driver model test (not committed) showed
exactly one up event (epoch 3, not synthetic) and a pending completion that
`poll_rx` then returned. The new committed host test repeats that sequence.

**Fix** (`kernel/src/m2/m3/network.rs`, only inside the feature-gated
`link` module):

- The up branch of `link::apply` now runs the same `harvest` and `drop_rx`
  as the down branch. It runs after the window-close accounting and before
  the `NETLINK:EVENT:UP` or `NETLINK:SYNTHETIC:UP` marker.
- A pending completion is taken into the empty broker buffer
  (`rx_completed` + 1), zeroed, counted as `NETLINK:RX_DROPPED`, and the
  same pinned RX page is reposted. An RX poll error or a repost error stops
  the broker through `stop()`.
- The window has already closed, so the repost is not counted in
  `window_delta` and is not a down-window publication.
- Every completion present at an up edge arrived while the kernel epoch was
  down, or unknown before the first sample. The cost is at most one
  legitimate frame per up edge, because there is a single RX buffer.
- The module comment and `docs/implementation/VIRTIO-NET.md` now describe
  the harvest at every edge.

With both branches, every sample that yields an event (down or up, real or
synthetic) destroys a completion in the used ring and a held copy before
`broker_tick` polls RX. A frame delivered in an epoch therefore completed
after the harvest at the edge that opened that epoch.

**Host test.**
`rx_completion_pending_at_a_real_up_edge_is_harvested_there` in
`drivers/virtio-net/src/tests.rs` covers the driver half on the
register/DMA model:

- an observed up (epoch 1) and down (epoch 2), with nothing pending;
- hidden up, frame, down and up (generation + 3) yield exactly one real up
  event (epoch 3) and no synthetic pair;
- the frame is still pending and its descriptor still posted, so a plain
  repost returns `Busy`;
- after the harvest and the repost the bounce page is zero, the RX
  available index advanced by one, and `poll_rx` returns nothing.

The kernel ordering itself is checked by source inspection only.

**Executed** from a worktree outside the main checkout (no nested
`.cargo/config.toml`, no cargo shim). The settings were
`NOS_QEMU_MACHINE=pc-q35-8.2`, a separate
`NOS_BUILD_DIR`/`NOS_TARGET_DIR`/`CARGO_TARGET_DIR` per profile, Rust
1.100.0 and QEMU 8.2.2. The commands ran sequentially on a host shared with
other native runs. The source fingerprint (v3) was
`55a612aa7a1d71b8ad8ff3680579141ea243c979529d393bc99ef0688a67f08e` on top
of `f0d36a0`; every native run recorded this value, and the runner rechecked
it after collecting the evidence. A first native attempt was stopped by hand
after its build, because a comment in the kernel source was then reworded;
it produced no result, and every step below was run again on the final
source.

| Command | Exit | Result or wall time |
|---|---:|---:|
| `cargo +1.100.0 fmt --all -- --check` | 0 | — |
| `cargo +1.100.0 test --locked -p virtio-net-driver` | 0 | 31 passed (1 new) |
| `cargo +1.100.0 clippy --locked -p virtio-net-driver --all-targets -- -D warnings` | 0 | — |
| `python3 -B -m unittest tools.tests.test_net_link` | 0 | 15 passed |
| `python3 -m compileall -q tools` | 0 | — |
| `python3 -B tools/build.py --kernel-feature test-system-net-link` | 0 | 4 s |
| `python3 -B tools/run_net_link.py --case normal` | 0 | 98 s |
| `python3 -B tools/run_net_link.py --case nostatus` | 0 | 99 s |
| `python3 -B tools/build.py --kernel-feature test-system-net-link-storm` | 0 | 4 s |
| `python3 -B tools/run_net_link.py --case storm` | 0 | 107 s (first attempt, no rerun) |
| `python3 -B tools/build.py --kernel-feature native-system-net` | 0 | 3 s |
| `python3 -B tools/run_net_system.py --case normal` | 0 | 98 s |

Every native run had raw QEMU exit 33, `passed=True` and no timeout. The link
runs also had `link_gate=True`, and the `native-system-net` run had
`network_gate=True`. No oracle changed. The runner's exact
`NETLINK:RX_DROPPED` expectation (`[1]` for `normal` and `storm`) also
proves that the new up-edge harvest found no completion in these runs: the
probe sends only after it has seen the up state, and the peer only answers.

- `normal`: events UP 1, DOWN 2, UP 3, no synthetic event, 0 Busy, 9,343
  samples. `RX_DROPPED` 1 (the pre-down echo reply). One window, TX
  submissions 9→9 and available (RX, TX) (9, 9)→(9, 9), delta 0. 10 RX
  completed, 9 delivered; 11 queries.
- `nostatus`: `NETLINK:ATTACH:NO_STATUS`, no sample, no link counter; 7 RX
  completed and 7 delivered.
- `storm`: 9 events, then `EXHAUSTED:9`. 4 windows with delta 0,
  `RX_DROPPED` 1, 9,919 samples, 48 queries. After the exhaustion stop the
  probe saw -14 and left its loop; send and receive then each returned -14
  (`NETLINK_PROBE_OFFLINE` 2). 2 resets and 2 ACKs.

Kernel ELF SHA-256 values are `ea99fd7310020122…` for the link kernel,
`0724c232fd6acce2…` for the storm kernel and `dc61978cb34c155b…` for
`native-system-net`. The last one equals the round-1 value, as expected for
a change that exists only behind `test-system-net-link`.

**Feature-off identity.** `native-system-net` was built at `0a0a5f5` in a
detached worktree (`git worktree add --detach …/work-h04/base 0a0a5f5`,
with its own `NOS_BUILD_DIR` inside that tree; build exit 0). Its sections
were extracted with `llvm-objcopy -O binary --only-section` and compared
with the build in the fix worktree:

- `.data` (1,725,248 B) was identical;
- `.text` (178,947 B) differed, and `.rodata` was 410,811 B against
  410,779 B.

This is the path dependence that round 1 already explained (embedded
absolute paths of path dependencies, and path-dependent crate hashes). The
decisive check again used **one path**. The source diff of the fix
(`git diff 0a0a5f5 -- kernel drivers`) was applied to the base worktree,
whose source fingerprint then equalled the run fingerprint above. The
profile was rebuilt with the same build directory (exit 0; `os-kernel` was
recompiled). Results:

- `.text` (178,947 B), `.data` (1,725,248 B) and `.rodata` (410,811 B) are
  byte-identical;
- the whole kernel ELF is identical, SHA-256
  `432b4a280bac1606cc3f9e17298655a1bedb04bc89534cfcf055bb277dd4f590`
  (the same value as in round 1).

The patch was reverted and the worktree removed with `git worktree remove`.

**Still not covered.**

- No native case drives either trigger: two or three `set_link` changes
  inside one PIT tick while a completion is pending. Both the synthetic-pair
  harvest and the up-edge harvest are host-tested on the driver model only,
  and the kernel ordering is checked by source inspection.
- The round-1 residual about a completion first seen at an up edge is
  closed by this fix: such a completion is now destroyed.
- Sampling once per tick leaves a residual between two samples. A hidden
  change after the sample that opened epoch E is seen only at the next
  coherent sample. Until then, 0x72 can deliver in epoch E a frame held
  from before a hidden down edge, or a frame that completed after a hidden
  reconnect. At the next sample both are destroyed if they are still held.
  A `Busy` sample, which skips the link step but not the RX poll, widens
  this window by at least one tick. In both cases the frame is delivered in
  the epoch that the kernel had observed at delivery time, not after a
  kernel-observed edge. Closing the window would need a link sample before
  each delivery or a per-frame generation from the device. It is left for
  the lead to assess.
- Exactly 256 configuration changes between two samples remain
  undetectable, as documented for `LinkMonitor`.
- ADR-H04-LINK-20261119 (lead-owned, not edited here) names -14 in the
  0x70/0x72 precedence. The broker returns its stop cause there, which is -14
  after exhaustion or owner exit. This is unchanged since round 1.

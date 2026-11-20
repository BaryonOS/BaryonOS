# H04-T3 native virtio-net link loss and reconnect — 19 November 2026

**Evidence level:** native QEMU 8.2.2 (q35 `pc-q35-8.2`, OVMF, TCG, one CPU,
256 MiB) with the independently developed kernel, plus host tests. The link
was changed through QEMU's `set_link`; no physical NIC, cable or link partner
was used. QEMU 8.2.2 has no 802.11 or Bluetooth device model, so Wi-Fi and
Bluetooth are not covered. H04 remains open.

Compact machine-readable record: [`h04-native-link.json`](h04-native-link.json)
(the absolute checkout path is replaced by `<CHECKOUT>`).

## What was executed

Source fingerprint (`tools/common.py` `source_hash`, v3):
`59beb2c237b4cbd49711e92c6d9443c3cf8555ea1eed823f24f431a6467b3475`.
All native runs used the uncommitted worktree on base `9c707c8` (recorded as
`head_at_run`) whose source fingerprint equals the H04-T3 commit; the
fingerprint was recomputed after committing. Builds used separate
`NOS_BUILD_DIR`/`NOS_TARGET_DIR`/`CARGO_TARGET_DIR` inside the worktree,
Rust 1.100.0, QEMU 8.2.2 and `/usr/share/OVMF/OVMF_{CODE,VARS}_4M.fd`, with
the environment-only cargo shim described below.

Native steps (sequential, one QEMU at a time; raw exit of each command; the
guest's raw QEMU exit was 33 in every run):

| Command | Exit | Wall time |
|---|---:|---:|
| `python3 -B tools/build.py --kernel-feature test-system-net-link` | 0 | 3 s |
| `python3 -B tools/run_net_link.py --case normal` | 0 | 68 s |
| `python3 -B tools/run_net_link.py --case nostatus` | 0 | 69 s |
| `python3 -B tools/build.py --kernel-feature test-system-net-link-storm` | 0 | 3 s |
| `python3 -B tools/run_net_link.py --case storm` | 0 | 145 s |
| `python3 -B tools/build.py --kernel-feature native-system-net` | 0 | 3 s |
| `python3 -B tools/run_net_system.py --case normal` | 0 | 68 s |
| `python3 -B tools/run_network_service.py --case normal` (read-only use) | 0 | 12 s |
| `python3 -B tools/run_network_service.py --case device-reset` (read-only use) | 0 | 6 s |

Kernel ELF SHA-256: link `398d65a52ebe6c00…`, storm `f37fdb2f0ab81067…`;
link probe `0e4f1fe41a7b8751…` (full values in the JSON).

## Cases and kernel observations

| Case | Kernel feature / NIC | Kernel link events | Probe-visible states | Down-window counters | Result |
|---|---|---|---|---|---|
| `normal` | `test-system-net-link`, STATUS offered | UP 1, DOWN 2, UP 3; no synthetic pair, 0 Busy; 6,437 samples | UP 1, DOWN 2, UP 3 (11 queries) | TX submitted 9→9, available (RX, TX) (9, 9)→(9, 9); delta 0 | pass, link gate pass |
| `nostatus` | `test-system-net-link`, `status=off` | none; 0 samples (`NETLINK:ATTACH:NO_STATUS`) | 0x74 → -15 | no window | pass, link gate pass |
| `storm` | `test-system-net-link-storm`, STATUS offered | UP 1 … UP 9 (9 events, 4 down windows), then `EXHAUSTED:9`; 13,880 samples | UP/DOWN 1–9 (33 queries), then -14 | 4 windows, each (9, 9)→(9, 9); delta 0 | pass, link gate pass |

- `normal`: the kernel reported `(1, up)`; the probe finished the unchanged
  base schedule (3 ARP, UDP echo, 3 DNS, DNS timeout) and sent a pre-down
  echo. After the kernel printed `NETLINK:RX_HELD:8` (the echo reply held,
  undelivered), the runner sent `set_link net0 off`. At the down edge the
  kernel destroyed that copy (`NETLINK:RX_DROPPED:1`), reposted the same RX
  page and opened a down window. Send and receive returned -23; the kernel
  printed `NETLINK:DOWN_REFUSED:2` and the runner sent `set_link net0 on`.
  TX submissions and both available indices were identical at window open
  and close (`NETLINK:WINDOW_*`, `SYSTEM:NETLINK_WINDOW_DELTA:0`). After
  `(3, up)` a receive returned Again (no stale frame); a fresh neighbour slot
  resolved the next hop with a new ARP request whose reply was the first
  frame of the new epoch, and a new UDP echo with different bytes matched.
- `nostatus` (`virtio-net-pci,...,status=off`): no link sample was taken,
  0x74 returned -15, no event was printed, and the base ARP/UDP/DNS schedule
  passed unchanged with the link assumed active.
- `storm`: handshaked toggles (each waiting for the kernel's event marker)
  produced epochs 2–9; the ninth transition exhausted the eight-transition
  budget (`NETLINK:EXHAUSTED:9`). The kernel stopped the device (reset
  request, bus mastering off, pinned pages kept) and observed both reset
  acknowledgements (exhaustion and owner exit: `NET_RESETS == NET_RESET_ACKS
  == 2`). Three later toggles (`on, off, on`) were acknowledged by QEMU while
  the latched guest still ran (`SYSTEM:BROKERS:PASS` arrived 3.0 s later)
  and produced no event, state or window marker. 0x74, send and receive
  returned -14.

Every run also passed the existing joint-system gate (init, Key Service,
volume, app, worker, disk oracle; `SYSTEM:CASE:NORMAL:PASS`, `M1:PASS`, raw
QEMU exit 33) and the synthetic peer transcript check.

## Feature-off regression and binary comparison

| Run | Exit | Result |
|---|---:|---|
| `native-system-net` normal (`run_net_system.py`) | 0 | joint gate and network gate pass; NET_* 8/8 TX, 7/7 RX, 7 denied, 1 reset/ACK (unchanged expectations) |
| `native-net-service` normal | 0 | `passed=True`, raw 33 |
| `test-net-service-device-reset` | 0 | `passed=True`, raw 33 |

The two network-service runs used `tools/run_network_service.py` unchanged
(PR #380 owns those files).

Section comparison of feature-off ELFs (baseline `9c707c8` builds versus the
H04-T3 source, same toolchain and flags):

| ELF (feature off) | `.text` | `.data` | `.rodata` |
|---|---|---|---|
| `native-system-net` kernel | identical (178,947 B) | identical | same size; 53 u32 line-number deltas (max 35) outside the embedded probe |
| `native-net-service` kernel | identical (152,979 B) | identical | same size; 27 u32 line-number deltas (max 35) |
| `test-net-service-device-reset` kernel | identical (156,243 B) | identical | same size; 37 u32 line-number deltas (max 35) |
| net-probe without cargo feature `link` | identical (20,853 B) | identical | same size; 8 u32 line-number deltas |

Identical `.text` and `.data` show unchanged code and initialized data with
the link features off. The `.rodata` differences are confined to panic
`Location` line numbers moved by the inserted source lines and, for the
`native-system-net` kernel, to the embedded net-probe ELF, whose own `.text`
and `.data` are identical as well.

Timing: the storm case took 144–148 s in three runs, the other cases 67–69 s.
The host timeline shows the extra time inside the unchanged base DNS-timeout
receive-poll loop, before any link toggle (`NETLINK:RX_HELD:7` at 2.8 s,
`NETLINK:RX_HELD:8` at 141.6 s); the toggles themselves took about 0.1 s.
The two link kernels differ in `.text` only in `event` (+51 bytes, the
storm `finished()` hold inlined). The cause was not identified. The run stays
within the 180 s runner limit and the 20,000-tick kernel deadline (13,880
ticks before exhaustion), but the margin is smaller than for the other cases.

## Host tests

| Command | Exit | Result |
|---|---:|---|
| `cargo +1.100.0 test --locked -p virtio-net-driver` | 0 | 29 passed (7 new: coherent read bound, read-only sample, STATUS/offline errors, publication counter, monitor epochs, synthetic pair, exhaustion and reset latch) |
| `cargo +1.100.0 clippy --locked -p virtio-net-driver --all-targets -- -D warnings` | 0 | clean |
| `cargo +1.100.0 fmt --all -- --check`; SDK and net-probe `fmt --check` | 0 | clean |
| `cargo +1.100.0 test --manifest-path sdk/Cargo.toml --locked -p nieuw-os-sdk-runtime` | 0 | pass |
| SDK runtime and net-probe (base and `link`) `clippy --target x86_64-unknown-none -- -D warnings` | 0 | clean |
| `python3 -B -m unittest tools.tests.test_net_link` | 0 | 15 passed: peer schedules, QMP controller (fake transport), verifier counterexamples, `execute` hook |
| `python3 -B -m unittest discover -s tests -p 'test_net_*.py'` (also `test_system_*`, `test_tcp_*`) | 0 | pass |
| `python3 -B -m unittest discover -s tools/tests -p 'test_*.py'` | 1 | 567 run; 7 failures outside this change: 6 `test_agent_isolation`/`test_agent_pipeline` errors from the session's `GIT_CONFIG_*` environment and `test_gs_discipline` counting the GS writes added by the H05 S3 base commit `9c707c8` (`sleep.rs`, `arch.rs`) |
| `python3 -m compileall -q tools`; `git diff --check` | 0 | clean |

## Environment note

The worktree is nested inside the main checkout, so Cargo merged both
`.cargo/config.toml` files and passed `-Tkernel/linker.ld` twice; lld then
placed `__text_end`/`__rodata_end`/`__kernel_end` at the image base and the
first baseline attempt triple-faulted after `KS-02:PASS` (raw exit 0). A
PATH-first cargo shim outside the repository passed exactly this worktree's
configured target flags once through `CARGO_ENCODED_RUSTFLAGS`; no repository
file was changed for it. The rerun baseline passed. This is an environment
artifact, not a product defect.

## Integration re-verification by the lead

The builder branch (`0bea209`, base `9c707c8`) was merged into
`ccr-38238c58-g8y1l6` as `5bef53f` without conflicts. PR #369 is merged, and
open PR #380 shares no file with this change. The lead reran the gates on
`5bef53f` from a detached worktree outside the main checkout (no nested
`.cargo/config.toml`, no cargo shim), with `NOS_QEMU_MACHINE=pc-q35-8.2`.
Every build and every run exited 0:

- `test-system-net-link`: `run_net_link.py --case normal` and `--case nostatus`;
- `test-system-net-link-storm`: `--case storm`;
- `native-system-net`: `run_net_system.py --case normal`;
- `native-net-service` and `test-net-service-device-reset`:
  `run_network_service.py --case normal` and `--case device-reset`;
- `tools/test_power_system.py`, the three POWER cases plus cold recovery,
  because `run_system.py` and `run_volume.py` gained hooks.

The `test_gs_discipline` failure that the builder saw on `9c707c8` was
already fixed on this branch (`c54b9e2`) and passes on the merged tree.
ADR-H04-LINK-20261119 records the syscall and the error code.
`native-system.yml` now runs the three link cases.

## Not covered

- Physical NIC, cable, link partner or driver on real hardware (H).
- Wi-Fi and Bluetooth: no QEMU 8.2.2 model.
- An in-flight TX at a down edge (QEMU completes it with `used.len` 0, which
  is ordinary TX handling) was not exercised.
- A generation-only change (synthetic pair) and `Busy` samples did not occur
  natively; they are host-tested only. Exactly 256 generation changes between
  two samples are undetectable by design.
- Optional `flap` and `reset` cases were not implemented.
- This is not a product link API: 0x74 and -23 exist only in the test profile
  and are not part of a published ABI version.

## Security review follow-up — 20 November 2026

An independent AI security review of H04-T3 reported three findings. Two
adversarial refuters each confirmed F1 and F2 and refuted the third. This is
AI review, not a human audit. The results above are historical and were not
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

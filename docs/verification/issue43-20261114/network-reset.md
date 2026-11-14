# N01 unexpected device-reset containment — 2026-11-14

## Scope and source binding

The new `device-reset` native profile passed with normal, client-exit and
live-revocation regressions on the same source-v3 fingerprint:
`e8d2d175a3e6c3ac4f6d2a9d03bfbcfe8bb1217e5f8b54782ffb65910238f4f1`.
The worktree was uncommitted on base
`b8a7e83658bdbcd4970c26f753dda1b342fce0d4`; the fingerprint, rather than that
base commit alone, identifies the executed source. This receipt was written
after those runs. Integration requires a new final-source gate.

QEMU 10.0.13 used disposable OVMF state, virtio-net and the synthetic local
peer. No physical hardware or external network was exercised. The fixture
changes no production capability, syscall, DMA-release or restart contract.

## Native observations

At tick 56, the actual virtio status changed 15→0 after TX ticket 5 was
published. Bus mastering remained enabled at injection so the ordinary
driver health path had to stop it. Five TX publications produced four
completions; ticket 5 was consumed once with `Io` (-12). An actual RX `Io`
preceded the terminal TX observation. No further TX publication or RX
delivery occurred after the fault.

The peer observed one held TCP/HTTP connection and one uncertain final
`reset-probe` frame, with no replay. It observed five client frames and
three server frames; kernel counters independently recorded four completed
TX and three delivered RX. Two reset operations had two actual
acknowledgements (fault containment and owner cleanup). The four denials
were exactly two app authority probes plus the RX and terminal-TX errors.

The service retained its existing failure 15/exit 1. Both old app ingress
handles rejected real IPC sends with error -10 at tick 57, after the fault.
The normal capability, object, process and frame-cleanup gates passed.
This demonstrates fail-closed containment after an injected virtual-device
reset; it does not establish physical fault tolerance or service restart.

## Commands and results

Each native case ran `python3 -B -u tools/run_network_service.py --case CASE`,
with `CARGO_BUILD_JOBS=2` and separate `NOS_BUILD_DIR`, `NOS_TARGET_DIR` and
`CARGO_TARGET_DIR`. Paths below are relative to the development worktree.

| Case | Result | Raw QEMU exit | QEMU elapsed seconds | Build directory / run suffix |
|---|---|---:|---:|---|
|device-reset|pass|33|2.415|`build/issue39-device-reset` / `89a349a678fd47699759db52fd1c1c3c`|
|normal|pass|33|6.767|`build/issue39-reset-normal` / `056f342207c84be6a9ce76cad1e6b231`|
|client-exit|pass|33|6.619|`build/issue39-reset-client-exit` / `6f175044dcc94d79a3fc2491e8b74844`|
|revoke|pass|33|6.633|`build/issue39-reset-revoke` / `a18c777951f84806be7c6710a9535664`|

Each build directory retains `network-service-CASE-report.json` and
`run/network-service-CASE-SUFFIX/{result.json,serial.txt,stderr.txt,network-peer.json}`.
The successful reset `result.json` SHA-256 is
`4d8db132f417e51fa6c7ff2ea226912a4b4ccc53aa17681386b3dc0830ecfebe`;
its original serial SHA-256 is
`63c404d02b4d8480bcb082a2c24b8518591aeb719641313e18832c991c6d852d`.
The report binds the source, kernel/ELFs, firmware, QEMU, original outputs
and actual packet transcript. Independent read-only review re-evaluated
this stored reset report and found no remaining blocking finding.

Host checks passed: 18 virtio-net driver model tests; driver Clippy with
`--all-targets --features test-device-reset -- -D warnings`; 19 native-peer
and oracle tests; `rustfmt +1.90.0 --check --edition 2021` on all six changed
Rust files; and `git diff --check`. Logs remain in
`build/issue39-reset-checks/`. The native builds retained the two existing
warnings in unchanged kernel lock/serial code.

## Retained failed first run

The initial reset run at 12:43:59 UTC failed with raw exit 35 after 2.289 s on
source-v3 `70be68a21d6dbe4554b06411dd02d91cee31b619602aa4f1dad60897fdac516d`.
Its immutable run suffix is `72df6d0770464796b969ba66393fcec4` in
`build/issue39-device-reset/run/`. `result.json` SHA-256 is
`a197bede145c9e61a72b18ba906e9b654539de391edbf35023fee3b192381c1b`;
serial SHA-256 is
`9e0002534310fbf76adc283eb803dd4147d7ec22c4a063356b57c84ffb2a2c61`.

The ordinary RX path detected the reset before the next TX poll, producing
the existing service failure 15. The first fixture observer incorrectly
required the TX failure site. The correction records the actual RX error
and performs one real same-ticket TX poll before retaining failure 15; the
oracle separately binds the terminal error, RX error and exact denial
count. No successful result was substituted for the failed run.

Resource-exhaustion native evidence remains a separate open matrix item.

# Native TCP client — TASK-TCP

State: implementation in progress; native TCP and FR-N01 remain open.
Owner: network agent. Write scope: `services/net-stack/`,
`sdk/examples/tcp-probe/` and this document. Kernel, ABI, central build
profiles, generic runner and progress remain with the integrator. Baseline
`309ff53`, clock prerequisite `c4667c9` locally as `8151023`; evidence from
that old source is not evidence for the code added here.

## Decision: maintained stack with a separate app compiler

The integrator approved exactly `smoltcp = 0.14.0` with Rust **1.91.1** solely
as the app compiler. Kernel and existing apps retain Rust 1.90.0. The new
standalone Cargo workspaces rebuild all path dependencies from source with
1.91.1 in a separate target directory. Rust rlibs are not exchanged between
compiler releases; only the established syscall ABI crosses the kernel
boundary. The matching 1.91.1 rust-lld is explicitly selected. This neither
replaces a kernel nor moves the TCP stack into the trusted core.

smoltcp 0.14.0 was published on 17 August 2026, requires Rust 1.91 and has
the 0BSD licence. It is the current maintained release at the check on
29/30 October 2026. Stock 0.12 was rejected because of TCP window underflow,
invalid IPv4 IHL and SYN/SACK panic paths that have since been fixed. The
targeted RustSec/upstream check found no smoltcp advisory, but is neither
an audit nor a guarantee that no vulnerabilities exist.

Features are only Ethernet, IPv4, TCP and integer Reno, with one interface
address, four neighbours, one route and four assembler segments. Default
features are disabled: no std, alloc, libc, host socket, TUN/TAP, CUBIC,
floating point, log, async, IPv6, fragmentation or hidden host fallback.
DNS remains the existing separate native parser; TLS is still open.

The actual Cargo.lock has eight registry crates. `stable_deref_trait` is an
x86_64 target dependency of heapless; the earlier preliminary inspection
without that target table was corrected. All crates were checked for
provenance, licence, no_std code, build script and checksum:

| Crate | Version | Licence | SHA-256 of .crate |
|---|---|---|---|
| smoltcp | 0.14.0 | 0BSD | b6f8b28ad56c6e35524a37dd492af5d1a47e31e1a4d175cd12f89c075f01980f |
| heapless | 0.9.3 | MIT OR Apache-2.0 | 25ba4bd83f9415b58b4ed8dc5714c76e626a105be4646c02630ad730ad3b5aa4 |
| managed | 0.8.0 | 0BSD | 0ca88d725a0a943b096803bd34e73a4437208b6077654cc4ecb2947a5f91618d |
| byteorder | 1.5.0 | Unlicense OR MIT | 1fd0f2584146f6f2ef48085050886acf353beff7305ebd1ae69500e27c67f64b |
| bitflags | 1.3.2 | MIT OR Apache-2.0 | bef38d45163c2f1dde094a7dfd33ccf595c92905c8f8f4fdc18d06fb1037718a |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | 4e7648175b45a9a48536d676f68d918270699102aa8dab5496df06904c914600 |
| hash32 | 0.3.1 | MIT OR Apache-2.0 | 47d60b12902ba28e2730cd37e95b8c9223af2808df9e902d4df49588d1470606 |
| stable_deref_trait | 1.2.1 | MIT OR Apache-2.0 | 6ce2be8dc25455e1f91df71bfa12ad37d7af1092ae736f3a6cd0e37bc7810596 |

Selected MIT/0BSD licence texts are in
[`THIRD-PARTY.md`](../../services/net-stack/THIRD-PARTY.md) and belong with
a later binary distribution.

Graph: smoltcp → managed, byteorder, bitflags, cfg-if, heapless;
heapless → hash32, stable_deref_trait; hash32 → byteorder. No proc macros
or build dependencies. smoltcp/build.rs generates configuration in OUT_DIR
(SHA `77331ea0f4f21b60da452aec44028978deedb06a3577cf2aaa95c837886c9ac0`).
heapless/build.rs performs a bounded RUSTC arm_llsc compilation probe
(SHA `8bb1b56df2c231f086314c0e61fa995594b221a4c8b67c1bab2d743bc8e2957c`).
Neither script contains a network/downloader. Other crates have no build
script. `SMOLTCP_*` environment variables must not change the profile;
fixed configuration constants are also checked during compilation.

Rust distribution: official `channel-rust-1.91.1.toml`, 10 November 2025,
SHA `483bb8b271168ee654e3be4f111beefa23b9d0c8b7f4a563a8278978dbe982e5`.
rustc is `1.91.1 (ed61e7d7e 2025-11-07)`; the x86_64-none std distribution
has SHA `069edc1d99b45efd97f1d9746289667dc4dc38d8532da547bcc3d0d735420d16`.
Installation used the existing project rustup directory and changed no default
toolchain or global shell settings.

Reassessment is required before listening/server use or larger buffers:
upstream issue 1227 reports a window-scale panic above 64 KiB; issue 1215
reports incorrect SYN assignment with multiple listeners on the same port.
This active-client API exposes no listen, uses at most 8 KiB per buffer and
has no socket removal/reuse. This is not a general server review.

## Interface and invariants

`services/net-stack` is a no_std, allocator-free user library. `Stack::new`
borrows fixed RX/TX packet buffers of 1514 bytes each and one or two
caller-owned socket storage slots. `add_client` borrows 1..8192 RX and TX
bytes per socket; the trial uses 4096 per direction. No silent capacity
growth. `connect`, `send`, `receive`, `state` and `close` use only process-local
indices, not IPC or kernel capabilities.

A received frame is copied only when the RX slot is free; a full slot remains
intact. Device offers no new TX token while a previous transmission is held
locally. `transmission()` returns immutable bytes with a non-reused number.
The native bridge publishes those bytes with `NetworkSource`; only a matching
actual broker completion may execute `complete_transmission(id)`. A local
TxToken is not a NIC or remote ACK. An uncertain TX outcome causes an error,
not automatic republication. TCP retransmission is scheduled by the real stack.

Each `poll` processes at most one ingress packet and one bounded egress pass
over at most two sockets. The caller yields between steps. Native time comes
only from ABI 1.6 `monotonic_ticks` (syscall 3, 100 Hz, all six arguments
zero), with checked conversion to microseconds. Regressing/overflowing time
is rejected before packet/timer processing. Repeatedly supplying the same
tick does not expire timers. Host tests may use explicitly injected model
time; that is not native clock evidence.

The conversion also reserves the upper half of the i64 range for timer
additions. Two active sockets with the same four-tuple (local/remote address
and port) are rejected. Cancellation closes the selected socket and then
requires TX drain or actual transport reset.

The synthetic trial may use a visible fixed test seed. This claims neither
a secure internet ISN nor entropy integration. A real network service must
use a separately authorised entropy source. The library grants no network
destination permissions; the existing trial capability and still-open
production gate must not be confused.

## Native profile contract under coordination

A new opt-in TCP profile requires ABI 1.6/monotonic-clock and the same actual
virtio-net/NetworkSource broker. The existing ARP/UDP/DNS profile retains its
own fixed counters. The TCP trial has variable packet volumes and receives
a separate observer comparing app TX/RX with actual broker counters. No
success without complete process cleanup and NIC reset ACK.

Proposed counterparty: only localhost QEMU socketnetdev with synthetic
Ethernet, MAC `02:00:00:00:00:01`, IPv4 `192.0.2.1`; client
`02:00:00:00:00:07`, `192.0.2.2/24`. No TAP/bridge/internet, real accounts
or personal data. The host implements only the protocol counterparty.

| Port | Native user operation and required evidence |
|---|---|
| 8080 | TCP handshake, actual HTTP GET, exact fixture reply, FIN/ACK close |
| 8081 | SYN receives valid RST; client observes connection refusal |
| 8082 | Handshake/GET followed by RST; no accepted HTTP reply |
| 8083 | First SYN remains unanswered; transcript proves actual retransmission, then HTTP/close |
| 8084 | Counterparty does not reply; actual stack timeout with measured Sys3 ticks |

This also requires invalid checksums/segments, full buffers/backpressure,
capability rejection/revocation and cold repetition on the final combined
image. The counterparty transcript and kernel/app evidence are bound to exact
source, ELF, profile and image hashes. Host test, target build and native
execution remain separate evidence levels. Native execution has not yet
been performed.

The agreed startup contract uses cap48, wrong-type endpoint56, PID176,
success40 and errorstep240. Native counters: 400=HTTP complete 2,
408=refused 1, 416=reset 1, 424=retransmission case 1, 432=actual completed
TX, 440=actual delivered RX, 448=timeout 1, 456=checked body bytes 40,
464=start ticks, 472=end ticks, 480=capability closed/stale rejection 1,
488=clock reads, 496=blackhole elapsed ticks at least 300. The broker expects
exactly one stale-capability rejection for this separate profile; the old
profile retains its seven negative ABI trials. TX/RX are checked against
the real broker and causal peer transcript. HTTP requires observed EOF and
a closed connection before its own timeout; refusal/reset must also occur
before that timeout. This prevents a LastAck timeout following a lost final
ACK from counting as successful FIN/ACK closure. Retransmission requires
at least 100 elapsed PIT ticks **and** a real repeated SYN in the transcript.

The HTTP body is twenty bytes `Nieuw OS native TCP\n`, SHA-256
`239cdc415a27dd042c4b1388a29bcbb1aa6cd8fa724359706373263092640692`.
The complete reply is 78 bytes, SHA-256
`d128fa1924be70f9e1f42e2a466e1e94123182b23b11110d273ced90cefda90b`.
Two checked responses provide 40 body bytes in total. The 8084 timeout is
300 guest ticks; the guest deadline is 500 ticks for this case and 900 for
the other cases. Host wall time does not replace this clock evidence.

FR-N01, TLS/certificate clock, DHCP/configuration, destination permissions,
NAS, browser integration, secure production entropy and full-release
acceptance remain explicitly open. This step reduces no product requirement.

## Reproduction and current evidence boundary

Use the existing project environment with the separately installed Rust 1.91.1
and a dedicated target directory. For example, from the repository:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-tcp-core python3 -B tools/dev.py cargo +1.91.1 test --offline --locked --manifest-path services/net-stack/Cargo.toml
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-tcp-probe-rust1911 python3 -B sdk/examples/tcp-probe/build.py
```

The helper builds only `tcp-probe` for x86_64-unknown-none and reports
`extra_env.NOS_TCP_PROBE_ELF` plus `net_probe` ELF/hash metadata. It executes
no host client or kernel. Absolute worktree paths in app code are remapped
to `/nieuw-os`; native evidence binds the actually tested ELF hash. The
central integrator connects `native-system-tcp`/`test-system-tcp-recover`
and the runner.

Fifteen host trials cover packet-buffer bounds/stale completion,
regressing/overflowing/model time, four-tuple isolation/cancellation,
1501 malformed IPv4-IHL frames, actual ARP→SYN and six smoltcp conversation
models (HTTP/EOF, refusal, reset, SYN retransmission, blackhole timeout and
lost final ACK). The last regression runs the actual HTTP predicate used
natively: first failing with the old predicate, then passing with the time
check. In the negative conversation, complete bytes and EOF are received,
but all ACKs after LastAck are withheld; timer closure remains rejected.
The failing exit 101 and source/log hashes are in
`verification/lost-finack-red.json`. These models use injected time and no
host socket. The compact host/target-build report is under
`services/net-stack/verification/`; native integration acceptance remains
open. Tests and builds do not earn an N01 point.

## Primary sources

- smoltcp-rs, [smoltcp 0.14.0](https://crates.io/crates/smoltcp/0.14.0),
  source/manifest/CHANGELOG and 0BSD, checked 29/30 October 2026.
- smoltcp-rs, [window correction PR1079](https://github.com/smoltcp-rs/smoltcp/pull/1079),
  [issue1227](https://github.com/smoltcp-rs/smoltcp/issues/1227),
  [issue1215](https://github.com/smoltcp-rs/smoltcp/issues/1215).
- Rust Project, [official 1.91.1 distribution](https://static.rust-lang.org/dist/channel-rust-1.91.1.toml),
  10 November 2025.
- IETF, [RFC9293 TCP](https://www.rfc-editor.org/rfc/rfc9293), August 2022.
- IETF, [RFC9112 HTTP/1.1](https://www.rfc-editor.org/rfc/rfc9112), June 2022.

Root integration: native-system-tcp/test-system-tcp-recover combine the real
TCP client with all existing services, an optional 100-Hz clock and per-process
fpu-context. The separate native-fpu trial remains required for actual x87/SSE
register patterns; the integer TCP probe does not replace that evidence.
Root gate/capture uses tools/test_tcp_system.py and capture_system.py --tcp;
native acceptance remains open until these actually pass.

## First complete native pair on 1b6a308

Exact 1b6a3086176a6c0a4e6514bf4d2f7105bef3d480 / v3 source
25a523bbb8b56ee38b0ec18ab223a855e61e54c8cf454578643298c0660971b1:
normal 7.61 s and cold read-only recovery 7.49 s, both raw 33. Per VM,
21 actual TX/13 RX agree across app, broker and protocol peer;
HTTP2/refused1/reset1/retransmit1/timeout1/body40/stalecap1, resetACK1.
Sys3 advances 438 ticks; blackhole 300 ticks. Disk hash
ad40c83d6b9d0a409391113f7ed4deb5aafd7c27f6629c22d7a6ddc060c639ea
is identical for normal-after/recover-before/recover-after. Final tested
normal kernel bc6c5931777819829ed57de6a2014deece3229f337ddf84d4c61ae0b89583103;
actual TCP ELF fea2154f68682f6403a060a013a39c656b9e4c302b0a41061d2d0a31e8f6bb91.
[capture --tcp](../verification/full-release-tcp-1b6a308/capture-validation.json)
rechecks real ESPs/disks/raw serial/peer sequences/final artifacts. The first
failed ARP test-server run remains separately preserved; no unknown frames
were ignored. This is TCP integration evidence with real other services,
CPU context and clock; TLS/configuration/destination broker/NAS and complete
FR-N01/N02 remain open.

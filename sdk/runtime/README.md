# G01 — bounded native app runtime

This independent `no_std` crate builds on the existing Baryon OS ABI. It uses
no heap, libc, host I/O or networking. The external dependency `zeroize` is
pinned to version 1.9.1 without default features and clears its IPC envelopes
on cleanup. The pure message layer contains no `unsafe`; `native.rs` contains
the small `int 0x80` boundary. The host build does not expose that native
module, preventing host tests from accidentally making a Linux syscall with
Baryon OS arguments.

The runtime is a buildable SDK foundation. It adds no app installation,
compositor or storage rights. Since K05 (ADR-042), a process with a
`LaunchImage` capability can start, observe and terminate children, and any
process can create its own endpoints within quota; without that capability,
an app still depends on an authorised bootstrap. `ChildProcess` handles
cannot be derived, transferred or revoked.

## API and authority

`Handle::from_raw` checks only the existing generation/slot structure.
`SendEndpoint` and `ReceiveEndpoint` help prevent mistakes in calls.
A number or Rust type never grants rights: the kernel checks the process,
object type, generation and authority on every call. `Rights` is restricted
to existing bits; `derive` and capability transfer let the kernel reject
every attempted rights expansion. This authority boundary is unchanged by
the SDK.

| Function | Behaviour |
|---|---|
| `native::abi_info/check_abi` | Read the ABI and accept only existing compatible profiles 1.0 through 1.3 |
| `native::yield_now/exit` | Yield/terminate the existing process |
| `native::close/derive/revoke` | Existing capability operations; no new object creation |
| `native::send` | One non-blocking send; `Ok` means only that the message was queued |
| `native::receive` | One non-blocking receive into a private staging buffer, followed by envelope transfer with clearing |
| `native::close_received_caps` | Explicit cleanup of received capabilities; attempts every handle and preserves the original message bytes |
| `native::endpoint_create` | K05 (ABI 1.3, `native-supervisor` profile): create an owned endpoint within the kernel quota of 4 per process |
| `native::spawn` | K05: start a `LaunchImage` image with one argument word and at most two attenuated capabilities (never RECV); return the sole `ChildProcess` handle |
| `native::kill/status` | K05: terminate or observe an owned child; `ProcessStatus` decodes only canonical status words |

`Outgoing::new` builds a fully zero-filled Message320 with at most 256
payload bytes. `Outgoing::from_bytes` accepts an existing protocol message
only with valid lengths, handles and canonical padding. At most two
capabilities can be attached; RECV transfer remains prohibited.

`Received` retains the sender, tag, payload and all original envelope bytes.
The constructor also checks that the length equals the syscall result.
Native IPC uses one transfer with clearing into the envelope; the staging
buffer is cleared immediately on successful transfer, and both `Received`
and `Outgoing` clear their bytes on cleanup. Their `Debug` representation
contains no payload. This is memory hygiene, not a guarantee that compiler
or register copies, crash memory or bytes copied elsewhere are cleared.
The consuming service/app must still validate its expected peer, tag,
request ID, epoch, domain, contents and permitted capabilities. Successful
`Received::from_bytes` alone does not establish kernel provenance.
`native::receive` obtains the bytes directly from the checked kernel copy.

There is no RPC retry. Under the existing kernel contract, a send returning
`EAGAIN` has published nothing; a queued message may subsequently have been
processed by the service. After a lost reply or `Unknown`, the app must
therefore not silently resend a mutation. The separate catalogue client can
apply these session and uncertainty rules to the unchanged envelope.

## Building and checking

From the repository root, with the pinned Rust 1.90.0 toolchain available:

```sh
cargo test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime
cargo clippy --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime --all-targets -- -D warnings
cargo build --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime --target x86_64-unknown-none
python3 -B sdk/examples/catalog-read/build.py
```

Use a separate `CARGO_TARGET_DIR` for parallel work. The SDK is a separate
workspace with its own lockfile. The example builder overrides linker flags
only for its own subprocess, so that the kernel linker script is not used.

Host tests compare all payload lengths 0–256 with the actual
`capability-core` parser, check corrupt envelopes and syscall results, and
exercise send/receive/revocation using the existing pure capability model.
This is host evidence. Cross-compilation produces a native ELF but does not
demonstrate boot, isolation, device I/O or user interaction. Native SDK
integration in an actual guest experiment has a separate evidence bundle.

## Contract sources

- Project [kernel and syscall contract](../../docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md),
  26 October 2026, ABI 1.0 and Message320; current implementation in
  [`m4.rs`](../../kernel/src/m2/m3/m4.rs) and
  [`capability-core`](../../crates/capability-core/src/lib.rs).
- Project [M5 contract](../../docs/implementation/M5-BLOCK-SERVICE.md) and
  [M6a contract](../../docs/implementation/M6A-WRITE-RECOVERY.md): ABI minor 1/2
  retain the register convention and add block operations. This SDK does not
  expose a block interface to ordinary apps.
- [Privacy contract revision 4](../../docs/LOCAL-PRIVACY-CONTRACT.md): installing
  an app grants no general document, device or network permissions.

The tests do not establish a human audit or general security. Native execution,
source hashes and limitations are recorded separately under
[`docs/verification/parallel-g01`](../../docs/verification/parallel-g01/).

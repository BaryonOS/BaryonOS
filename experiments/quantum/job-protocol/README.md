# Native quantum job requests — user service v1 proposal

This standalone package implements the centrally agreed internal
request contract `QJ01`, version 1. It can encode one existing IPC payload of exactly
256 bytes in typed form and validate it strictly, without an allocator, `unsafe`
or external dependency. It is a first building block for jobs outside the
fixed native selftest.

**Execution is still missing.** No job dispatch, queue, service,
reply protocol or working cancel/status/result operation has been added.
The codec checks the requested work budget; it does not enforce quota
during computation. The kernel IPC, syscalls, capabilities, storage formats,
the existing qsim-core and the native selftest do not change. The existing
host JSON protocol `qsim-v0` remains a separate adapter.

## Exact payload contract

All multi-byte integers are little endian. Unused and reserved
bytes must be zero; the decoder does not ignore extra fields or trailing data.

| Bytes | Field | Rule |
|---|---|---|
| 0–3 | magic | Exactly `QJ01` |
| 4–5 | version, u16 | Exactly 1 |
| 6 | operation, u8 | 1 Submit, 2 Cancel, 3 Status, 4 ResultPage |
| 7 | qubits, u8 | Submit: 1–6; otherwise zero |
| 8–15 | job_id, u64 | Always non-zero |
| 16–23 | request_id, u64 | Always non-zero |
| 24–31 | seed, u64 | Submit only; any u64 allowed |
| 32–35 | shots, u32 | Submit: 1–4,096; otherwise zero |
| 36–37 | gate_count, u16 | Submit: 0–64; otherwise zero |
| 38–39 | reserved | Always zero |
| 40–47 | work_budget, u64 | Submit: 1–1,048,576; otherwise zero |
| 48 | result_page, u8 | ResultPage only: 0–3; otherwise zero |
| 49–63 | reserved | Always zero |
| 64–255 | at most 64 gates | Three bytes per gate; all unused bytes zero |

A gate is `[operation, target, control]`. The gate codes are 1 H, 2 X,
3 Z, 4 S and 5 CX. Only CX uses `control`; for the other gates that
byte must be zero. Every target and every CX control must be less than
`qubits`, and the CX control must differ from the target. Validation takes
place on the u8 indices before a future backend widens them.

Besides the common header, Cancel and Status contain only the
two IDs. ResultPage additionally contains the page index. For these
operations, too, all remaining bytes must stay zero.

There is **no caller PID or authority in the payload**. Both IDs are
correlation fields; knowing a job ID grants no job rights. A future
userspace service must bind the actor to the authenticated IPC envelope,
check rights and process request IDs according to its agreed lifecycle.
This codec alone does not prove that binding. The seed remains
exclusively input for repeatable simulation, not a key or entropy source.

## Rust API

`Identifiers::new` requires two non-zero u64s. `Submit::new` validates the
limits and gates before copying into a fixed array. `ResultPage::new`
checks the page index. Their fields are private so that safe callers
cannot encode an unvalidated request.

```rust
use qsim_job_protocol::{decode, encode, Gate, Identifiers, Request, Submit};

fn encode_example() -> Result<[u8; 256], qsim_job_protocol::Error> {
    let ids = Identifiers::new(1, 1)?;
    let request = Request::Submit(Submit::new(
        ids, 2, 0, 4096, 100_000,
        &[Gate::H { target: 0 }, Gate::Cx { target: 1, control: 0 }],
    )?);
    let payload: [u8; 256] = encode(&request);
    assert_eq!(decode(&payload)?, request);
    Ok(payload)
}
```

`decode(&[u8])` accepts only 256 bytes. `encode` always writes
a canonical `[u8; 256]`. Local `Error` variants describe rejection; they
do not define any reply, syscall or on-disk error encoding.

The Rust types are not wire structs: their memory layout is never
copied or used as an ABI. `Request` deliberately contains the full
fixed gate buffer; that is why only this enum has a justified local
Clippy exception for variant size. A `Box` would add precisely the forbidden
allocator dependency.

## Checks and reproduction

```bash
cargo +1.90.0 test --manifest-path experiments/quantum/job-protocol/Cargo.toml --locked --offline
cargo +1.90.0 clippy --manifest-path experiments/quantum/job-protocol/Cargo.toml --all-targets --locked --offline -- -D warnings
cargo +1.90.0 build --manifest-path experiments/quantum/job-protocol/Cargo.toml --lib --release --target x86_64-unknown-none --locked --offline
```

Use a separate `CARGO_TARGET_DIR` when building in parallel. The workspace is
independent of the kernel and of the existing quantum workspaces.

The 13 host tests check exact golden bytes, all four operations and
five gate codes, maximal arrays, minima/maxima, all 256 truncated lengths,
over-long input, header and ID errors, every reserved or unused
byte position, bounds and CX rules. An additional property within those
tests checks 6,400 single-byte mutations: every accepted mutation
must re-encode identically without normalisation.

The library compiles natively for `x86_64-unknown-none`; that is
**compilation evidence, not an executed OS job or VM run**. The exact executed
commands, exit statuses, source and artifact hashes are under
`docs/verification/parallel-i08`.

## Smallest next step

Connect this codec to a separately authorised userspace dispatcher,
an agreed reply contract and the existing qsim-core. That integration must
natively test actor binding, lifecycle, actual quota/cancel behaviour and result pages.
Add the new source dependency centrally to the OS source binding
as soon as the OS build actually uses it. The codec is not yet
linked into the existing OS image. L02 therefore remains open.

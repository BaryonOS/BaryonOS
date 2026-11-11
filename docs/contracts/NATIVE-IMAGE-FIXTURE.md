# Native image fixture: observer contract

IMAGE-NATIVE-OBS-1 on kernel base
`15e8cfced28ad46f15f5f573e5f00e8e87b74215`. This opt-in observer belongs to
codec/probe checkpoint `9ae1c738012fc73085a04d6cfca7cd7fbc3bde46`; it runs no
PNG/JPEG parser in the kernel. See also the component description
`docs/implementation/NATIVE-IMAGES.md` when integrating that source series.
FR-A03, object RPC, zoom/pan, the compositor and general format acceptance remain open.

## Ownership and hooks

This task delivers only `kernel/src/m2/m3/image_fixture.rs` and this contract.
The integrator owns Cargo, m3/system, ELF selection, bootstrap/capabilities,
process death, builds and the VM harness. Proposed profile:
`native-system-images=[native-system,process-memory,fpu-context]`, ABI1.7.
The existing four services remain active; the probe starts only at boot in
slot 4. It receives no raw network, key, entropy, storage or display capability.
The app uses ABI query/yield/exit, monotonic ticks and its own private heap growth.

Place the module and all hooks only under the new profile condition;
old profiles do not load this observer. The API is:

```rust
bind(s: &State, actual_pid: u64, linked_elf: &[u8]);
is_probe(s: &State, slot: usize) -> bool;
observe_exit(s: &State, slot: usize); // vóór State.cleanup/State.free
fault(s: &State, slot: usize, context: &[u64; 22]) -> !;
finished() -> bool;
check_and_print(s: &State);
```

`bind` follows loading/registering the actual probe and filling startup
word 176 with its actual PID. Pass exactly the same linked
`IMAGE_PROBE_ELF` slice used by the loader. Bind retains the PID,
boot slot, actual start tick, original mapping count and ELF fingerprint.
The startup page and original 16 heap pages must already exist;
the probe must not have executed yet. Rebinding is an error.

`is_probe` requires the stored PID **and** boot slot plus live mappings.
A worker that later reuses slot 4 therefore does not become the probe.
Exit/fault also compare the actual `last_event.pid`; a slot number is not
an identity. `observe_exit` reads only a validated startup page owned by
this PID and records all words before memory is freed.
Afterwards, `check_and_print` uses only the kernel snapshot, never a pointer
to cleaned-up user memory. IF=0/single-CPU callbacks are required; no observer
borrow may span a scheduling point, IPC or userspace execution.

The helper does **not** call `Engine.process_died`. Root retains generic
Engine/process handling and the existing exception tests. The `fault` hook
does not return: it reports actual CPU metadata and terminates only the
disposable VM test as failed. Any Engine bookkeeping required before that
termination therefore remains root's responsibility. This is not a new
product fault policy. Call `finished` alongside the existing joint gates
and `check_and_print` once before final SYSTEM success. A missing, duplicate
or invalid snapshot must never count as a completed image component.

## Required data and independent kernel observation

The kernel reads 15 fixed u64 words within the private startup page:

| Offset | Meaning / requirement |
|---|---|
|40,240|done=1,errorstep=0; actual exitcode0 and syscallvector128 |
|400,408|PNG 8, JPEG 3; including PNG redecoding after rejected input |
|416,424|149647 pixels, 10 negative outcomes |
|432,440|pixel-FNV64=548683896226622996 (`079d5118b890ae14`),623808 sample bytes |
|448|1024 growth pages reported by the app |
|456,464|actual app arena high-water mark > 0 and ≤4194304; live allocations 0 |
|472|1 decode cancelled in advance; no mid-decode latency claim |
|480,488|guest start/end ticks within actual kernel launch/exit ticks; duration <20000 ticks |
|496|QUALITY_OPEN=0 for the resolved specific progressive-JPEG oracle question |

The heap claim is also checked against the kernel: 1040 total heap pages
(16 existing + 1024 new), exactly 1024 additional user mappings relative
to bind, owner PID / Kind::Data, user/RW/NX and a mapping count within the
unchanged 2048-page quota. The user stack has 16 RW/NX/Kind::Stack pages,
with inaccessible lower/upper boundaries; the end of the heap suffix also
remains unmapped. Neither heap, stack nor quota is enlarged for this test.

Arena accounting and pixel checksums are app measurements from the hashed,
reviewed probe, not general attestation of an arbitrary malicious app.
The kernel does not parse the allocator or images again. It checks its own
memory facts, exit, identity and actual ticks. The 10 negative decoding
outcomes do not replace an actual native allocator OOM/restart test: that
remains open until separately executed. Successful normal guest execution
shows that this input set worked with the actual 64 KiB stack, not a
maximum-stack guarantee for all possible files.

## ELF and artifact binding

`bind` calculates FNV-1a64 and length over the **actual linked ELF bytes**;
these appear as `SYSTEM:IMAGE_ELF_FNV64` and `SYSTEM:IMAGE_ELF_BYTES`.
FNV is only a drift/parity fingerprint, not a cryptographic authenticity
or boot claim. Root must additionally compare SHA256 of this exact ELF
with the build manifest, include input and released image.
For codec checkpoint 9ae1c73, the actually built input ELF is:

`ac2a0ae22021ae6dda7a37d981b5a049209ee71b669a4644c2fc5eaeec9fbff2`.

After a relevant source/configuration change, it must be rebuilt and bound
again; an old hash field or old successful test is not evidence. Source commit,
profile, QEMU configuration, loader/kernel/ELF/image hashes and serial output
belong to the final root gate. The app contains synthetic PNG/JPEG fixture
bytes; no host decoder or personal photograph is present as an OS implementation.

Success markers are printed only after a valid snapshot and joint completion:
`SYSTEM:IMAGE_*` for PID/ELF/pixels/memory/ticks/guard and finally
`SYSTEM:IMAGES:PASS`. Every `SYSTEM:IMAGE_FAIL:*`, fault, timeout, missing
marker or unexpected actor makes the gate fail. The root runner must strictly
check all required values and retain raw output.

## Executed checks of this helper

The same pure `Observer` predicates run in the kernel and in 9 executed
host-model tests: valid snapshot, wrong/stale PID/slot, worker reuse,
corruption of every app word requiring an exact check, heap/owner/guard
contradictions, clock boundaries, nonzero exit, wrong vector, premature
sign-off, rebinding, duplicate exit/print and persistent rejection after
an error. Command:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools python3 -B tools/dev.py rustc +1.90.0 \
  --edition=2021 --test kernel/src/m2/m3/image_fixture.rs \
  -o /tmp/nieuw-os-image-fixture-model-tests
/tmp/nieuw-os-image-fixture-model-tests --nocapture
```

Outcome: 9/9, exit 0. These are models with explicitly supplied kernel facts,
not executed CPL3 tests or evidence that those facts exist on hardware.

The native adapters were also type-checked on a private copy of the actual
15e8cfc kernel/State using Rust1.90/x86_64-unknown-none and
`native-system,process-memory,fpu-context`, exit0. A module inclusion was
added only there; shared source was not changed. Unexecuted ELF inputs were
used for `include_bytes`: this was **cargo check**, not a linked or booted
joint profile. The exact type-check report is at
`/tmp/nieuw-os-image-fixture-binding-check.json`. Root integration, correct
role ELFs, the complete build and actual QEMU execution remain required
and have not been signed off by this helper check.

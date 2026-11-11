# M5 user program and startup manifest

`m5_probe.rs` is one real Rust `no_std` ELF with separate private mappings per
process. The startup role selects service, client, test supervisor or the separate
CPU-negative attacker. The service source is `services/block/src/user_service.rs`;
the shared wire contract is in `services/block/README.md`. Existing
M3/M4 probes and their linker/scripts are not replaced.

## Build and artifact check

From the root: `python tools/build_users.py --kind m5` builds
`build/users/m5_probe.elf` and `manifest-m5.json`. The kernel build receives
`NOS_M5_ELF`; the build tool determines the exact rustc 1.90 flags. This direct
build uses `x86_64-unknown-none`, static relocations, no red zone,
panic=abort, opt-level=2 and `m5_linker.ld`, without dependencies or host calls.
The manifest contains binary/main/linker hashes and hashes of the four included
modules. There are no normal I/O, debug port or success marker rights.

Artifact check:
`python -B -X utf8 userspace/tests/test_m5_probe.py --elf build/users/m5_probe.elf`.
The eleven tests check segments, buffers, symbols, module hashes and
negative ELF mutations. Opcode presence is not an executed syscall.

## Fixed memory and fields

RX starts at `0x40000000`; private RW at `0x40010000` with three pages.
Message input is at `0x40011000`, output at `0x40012000`, separate
RO/NX data at `0x40013000`. `m5_runtime.rs` uses scalar int80 and copies
output directly into its own values before a next syscall replaces output.
The offsets below are u64 fields from RW; no frame is shared between
processes.

| Offset | Meaning |
|---:|---|
| 0 | role: service 0, client 1, test supervisor 2, attacker 3 |
| 8 | private syscall counter |
| 16 / 24 | non-zero private pattern / reserved zero |
| 32 | service epoch at start |
| 40 | status: 0 active, 1 done, >=100 fixture error |
| 48 | READ-only Block cap, service only |
| 56 | service request root0; client reply root; supervisor announcement root |
| 64 | supervisor SEND to client control; other roles do not need this |
| 72 | service SEND to supervisor announcements |
| 80 | service request root1; client control RECV root |
| 88 | service SEND to fixed client reply endpoint |
| 96 / 104 | service scopes: absolute sector 4 / 6 |
| 112 | test case 0..17 |
| 120 / 128 | client FNV-1a64 of the two actually received sectors |
| 136 | last accepted/forwarded service epoch |
| 144 / 152 | client RPC/negative counter, supplementary observation only |
| 160 / 168 | bound client PID / supervisor PID |
| 176 | initial service PID; no silent rewrite in peers |
| 184 | test-only service fault gate: 2 submit→ud2 before poll, 1 idle ud2, 0 normal |
| 192 | client phase: 0 waiting, 1 first reads, 2 negatives, 4 later reads, 5 done |
| 200 | first sector of Block scope (4; count is 4) |
| 208 / 216 | attacker only: supervisor MMIO/DMA test address |
| 224 | numeric service Block handle for process-local authority negative |
| 232 | real client Block cap with zero READ rights, case2 only |
| 240 | local failure diagnosis |

Announcement TAG `0x4d35414e4e000001`: payload four u64
`(1, epoch, 0, 2)` plus two real caps with rights 9. The supervisor uses
the kernel sender PID and forwards TAG `0x4d3542494e000001`, payload
`(1, epoch, service_pid, 2)` with two SEND-only caps. The client validates the
supervisor PID, next epoch and increasing service PID. On rebinding,
old handles are tested via a real send for EREVOKED and closed by the client.

## Cases and acceptance

| Case | Trial |
|---:|---|
| 0 | two sectors via eight real 128-byte reads |
| 1 | thirteen protocol negatives, then normal reads |
| 2 | typed device cap/rights and non-consuming erroneous poll output |
| 3 | asynchronous EAGAIN with peer progress |
| 4..9 | bad descriptor; used-id/index/short/long/status |
| 10..11 | I/O error / timeout, recovery and normal reads |
| 12 | reset timeout, local error reply and peer progress, no sector digests |
| 13 | epochs1..3 pending submit→ud2; epoch4 normal reads |
| 14..16 | fourth attacker: MMIO read / DMA read / PCI out fault |
| 17 | no device, local error reply and peer progress, no sector digests |

Case1 request IDs1..13: version2→Invalid, WRITE2→NoSys, flags1→Invalid,
reserved1→Invalid, length0→Range, length129→Range, offsetMAX→Range,
offset511/length2→Range, old epoch→Stale, requestdata1→Invalid,
outerlen191→Invalid, outerlen193→Invalid, caps2→Invalid. The last case
transfers the same local source cap twice as SEND; the service must close both
actual receiver handles. The source cap is only closed by the client after the
error reply. The kernel observer must record, per real request, the error form,
zero-data error reply and zero device calls/publications/completions;
user word152 alone is not evidence.

The client assembles both complete 512-byte sectors and computes FNV
offset `14695981039346656037`, prime `1099511628211`. There is no expected
disk content or seed in ELF/startup. Root compares this metadata with two
different disposable images and checks that their hashes do not change.
On rebind the complete assembly is restarted so that epochs are not mixed.

The integrated QEMU gate, driver observer and unchanged M1–M4 regressions
are required separately. Host tests or a user status provide no guest or
device evidence. Client and supervisor remain active after done via counter/yield
so that survivor progress stays observable. Writes, filesystem and hardware
or IOMMU containment are outside this first M5 slice.

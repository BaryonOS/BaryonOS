# TASK-DOC-NATIVE — real document session in the combined system

Task contract, 29 October 2026. Owner of app integration: `native_ui_apps`;
main agent owns shared lockfiles, kernel observer and final integration.
Write area: `sdk/examples/system/src/app.rs`, the package manifest and this
document. No kernel/service/SDK protocol change or graphical product claim.

## Native scenario and ownership

The combined system app uses, for actual document work,
`sdk/document-session::volume::{DocumentSession, VolumeSession, Buffers}` and
the existing `NativeTransport`. The v7 upload tag and a service-issued token
bind BEGIN/APPEND/ABORT/COMMIT to the sender known via the kernel and
the granted domain. The document remains object 0, `systeemproef`, 4096
fixture bytes, revision 1. The existing record crypto trials remain unchanged;
this data on the test disk remains plaintext and proves no B02 encryption.

The old v6 client performs, exclusively before READY, the existing STAT and real
cross-domain refusal trials. Its lexical lifetime ends before the new
document session begins. After that only the document session owns the endpoint pair;
closing explicitly closes both endpoints. No two active sessions share
the same handles. The three caller buffers of 4096 bytes each sit in a
separate `inline(never)` function on the existing 64-KiB app stack, not in BSS.
A temporary fixture chunk is at most 160 bytes. No keys,
block rights or host file routes have been added.

### Normal start

1. CREATE checks absence and creates exclusively a local draft.
2. The public replace-range API builds the 4096 fixture bytes in the draft.
3. BEGIN + one APPEND of 160 bytes exercise real token-bound progress.
   ABORT cancels before the 512-byte sector buffer can send out a content write
   command. The draft stays dirty, revision 0 and not uncertain.
4. SAVE uses a new upload token and request ID 3; COMMIT must confirm revision 1.
   An uncertain or wrong reply yields no APP_DOCS success.
5. OPEN rereads the actually confirmed document; name, revision,
   full content and checksum are checked. Then close follows.

### Cold recovery

A completely new VM uses exactly the normal disk, with a read-only QEMU drive
and the existing read-only block right. The SDK opens the document in ReadOnly,
checks all 4096 bytes again and refuses local edit/save without
mutation. The independent disk oracle and identical before/after hashes remain
the runner conditions; no snapshot restore or restored RAM state.

## Acceptance and evidence boundary

Build the ordinary user program for `x86_64-unknown-none` with the existing
build script. Then run `tools/test_system.py` on one frozen source: two real
QEMU runs (normal/recover), real IPC, all existing raw capability refusals,
record context/revocation trials, lifecycle/cleanup and an independent disk oracle.
The normal final build must be byte-identical to the actually tested normal
kernel. The shared `sdk/Cargo.lock` change contains only the existing local
dependency links and is integrated by the main agent.

After successful execution this task proves the native SDK v7 positive chain and
the named error refusals. Negative uploads with token/sender swaps
remain separate host/native gates according to their actual evidence status.
No Materiaal GUI, login/session lifecycle, persistent keys, encrypted
document volume or complete test release acceptance is claimed by this.

## Actually executed — 29 October 2026

Base `d73d515` plus this app change and the lockfile generation locally permitted by
the main agent. Frozen source fingerprint v3:
`3f5869e80636e2127b37a268fafbcb2b2936626522912fef7aca507bf90457ae`.
The lockfile diff contains only the two existing local dependencies of the
system app; the main agent integrates that shared lockfile separately.

With Rust 1.90.0, QEMU 10.0.13, `pc-q35-8.2`, TCG, qemu64, one CPU and 256 MiB:

```sh
PATH=/workspace/.nieuw-os-env/bin:$PATH \
LOCALAPPDATA=/workspace/nieuw-os/.tools \
OS_OVMF_CODE=/workspace/.nieuw-os-env/sysroot/usr/share/OVMF/OVMF_CODE_4M.fd \
OS_OVMF_VARS=/workspace/.nieuw-os-env/sysroot/usr/share/OVMF/OVMF_VARS_4M.fd \
NOS_QEMU_MACHINE=pc-q35-8.2 python3 -B -u tools/test_system.py
```

**Exit 0:** nine system harness tests, 17 SDK runtime tests and two real VM runs.
Normal and cold recover give raw exit 33; both have APP_DOCS=1, revision 1,
4096 bytes, three real capability refusals, record seal/unseal 1/1,
context refusals 10, revocation refusals 2 and an empty final
process/capability state. Normal: 42 disk writes/3 flushes. Recover:
0 writes/0 flushes. The independent disk oracle passes for both images.

Normal-after, recover-before and recover-after are all:
`ad40c83d6b9d0a409391113f7ed4deb5aafd7c27f6629c22d7a6ddc060c639ea`.
The final normal kernel is byte-identical to the tested normal kernel:
`a6a559bf5a7f02b02cdba55ff4a6676805838981cb94df640acee5c77ed0b8f7`.
The ordinary system app ELF has SHA-256
`a414dfba36e620e8d225c351d023cd5becdff0b130a76e29135c15f745119b4f`;
the inspected RW segment is 1304 bytes and stays within one data page.

Target build via `sdk/examples/system/build.py`, targeted target Clippy with
`-D warnings`, package formatting check and diff check pass. Clippy
used the same portable crypto cfgs; no new crates were fetched.
A separate AI source review found no concrete blocker within ownership,
cancellation and error handling; this is not a human audit or formal
worst-case stack guarantee.

Full local evidence: `build/system-gate.json`,
`build/run/system-normal-e9e9d709347b46fbb9585bb111b64015/` and
`build/run/system-recover-a1589c5ca67f440f83b81c185a86d194/`, with raw serial,
manifests, real images and oracles. Command logs are under `build/logs/`.
The first attempt stopped before VM start because of a missing QEMU PATH; that
setup error is kept as `build/logs/native-document-no-qemu-gate.json` and
does not count as a native run. After selecting the existing runtime the
full gate passed. The main agent repeats the merged source before final acceptance.

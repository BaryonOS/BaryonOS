# S09 second archive disk: local native gate, 19 November 2026

Scope: the owner-approved, test-only second virtio-blk disk of the volume
fixture (bounds B1–B6 in
[S09-DUAL-DISK-FIXTURE-REQUEST.md](../../../implementation/S09-DUAL-DISK-FIXTURE-REQUEST.md)).
This is local QEMU evidence only. It is not hosted CI acceptance and it does not
close S09.

## Source

- Branch `claude/s09-independent-recovery`. First gate at commit `13674aac`
  (fingerprint `69bf6c9f…ee19`, log `build/gate-dualdisk-13674aac.log`).
  Re-run after the review fix at commit
  `e9fbfb57` (lane-1 MMIO window must be unmapped; comment fix):
  source fingerprint (v3, `tools/common.py:source_hash`)
  `9bc2dff89a7221341d26a87a8fc620c64acedf1b5cccdc0ddab6f7b64275a5b1`,
  identical at start and end, 22/22 passed, log
  `build/gate-dualdisk-e9fbfb57.log`. Re-run after the stricter power-cut
  checks at commit `7bd540bc` (expected new header, records and prefix on
  disk 1; disk-0 comparison): fingerprint
  `9302225a5d04564e57b1cacb80b479763acaf1398ce9738d3fb4555672a0f4ca`,
  22/22 passed, log `build/gate-dualdisk-7bd540bc.log`. All disk sha256
  values below were the same in all three runs.
- Re-run after the reserved-byte check on the new header and collection
  records at commit `b6bb2cab`: fingerprint
  `552db03a6f78b5b10f5afea14209a6f36317d0d38e918494bc1e106cb3c49379`,
  22/22 passed, exit 0, log `build/gate-dualdisk-reserved.log`; the cut
  stage again classified the archive disk as `corrupt`.
- Hosted: `volume-acceptance.yml` job `m6e-native-volume`, run
  [44495342426](https://github.com/BaryonOS/BaryonOS/actions/runs/44495342426)
  at `b6bb2cab`: `M6e gate passed=True stages=22`, all seven second-disk
  stages passed, job time 6 min 21 s (timeout 40 min).
- `host-quality` fails only on the stale workflow stage-count pins; see
  [S09-WORKFLOW-PIN-INTEGRATION-REQUEST.md](../../../implementation/S09-WORKFLOW-PIN-INTEGRATION-REQUEST.md).
- QEMU 10.2.1, machine `pc-q35-10.0`, `tcg,thread=single`, one CPU, 256 MiB.

## Commands and exit codes

| Command | Exit | Result |
|---|---|---|
| `python3 -B tools/test_volume.py` | 0 | `M6e gate passed=True stages=22`, `build/volume-gate.json` status `completed` |
| (gate host step) `cargo +1.100.0 test --locked -p volume-core` | 0 | |
| (gate host step) `cargo +1.100.0 test --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime` | 0 | |
| (gate host step) `python3 -B -m unittest discover -s tests -p 'test_volume_*.py'` | 0 | 66 tests, OK (13 in `tests/test_volume_disk1.py`; at `b6bb2cab`) |
| `cargo +1.100.0 test --locked -p virtio-blk-driver` | 0 | 1 new test: only `5<<11` and `6<<11` are reachable |
| `cargo +1.100.0 clippy --locked -p virtio-blk-driver --all-targets -- -D warnings` | 0 | |
| `cargo +1.100.0 fmt --all -- --check`, same for `--manifest-path sdk/Cargo.toml` | 0 | re-run at the final code commit |
| `cargo +1.100.0 clippy --manifest-path sdk/Cargo.toml --locked --all-targets -- -D warnings` (ci.yml) | 0 | |
| `cargo +1.100.0 clippy --offline --locked --release --target x86_64-unknown-none --manifest-path sdk/Cargo.toml -p nieuw-os-volume-docs-example --features native,archive-portal --bins -- -D warnings` | 0 | portal build |
| `python3 -B tools/progress.py --check` | 0 | 41/100, consistent (not rewritten) |
| `python3 -B -m unittest discover -s tests -p 'test_ci_workflows.py'` | 1 | only `test_volume_gate_stage_counts_match_plan`: the protected workflow pins say 15, `len(PLAN)` is 22 |

The raw gate log is `build/gate-dualdisk-13674aac.log` (ignored `build/`).
All 15 earlier stages passed unchanged in the same run.

## New stages (7, appended; 22 in total)

Each stage boots a new VM. Disk 0 is `volume.img` at `addr=0x5`; disk 1 is a
disposable raw `archive.img` under `build/run/volume-image-*/` at an explicit
`addr=0x6`. QEMU refuses an explicit slot that is already taken, so every
passing boot with disk 1 also shows that the auto-assigned ESP is not on
`00:06.0`. Sha256 values are shortened to 16 hex digits.

| Stage | Parent | Disk 0 before → after | Disk 1 before → after | Result |
|---|---|---|---|---|
| `archive-disk-export` | `archive-export` | `19e2fdb3081ed07d` → same | `a4cc8ae039e7215c` → `87d0166bc7334dd2` | archive written from sector 0 of disk 1: 91 801 bytes, sha256 `d75aa5c8…`, equal to the archive the earlier stage wrote to disk 0; `tools/volume_archive.py` parses it and it matches the volume oracle (objects, metadata incl. created/modified, collections, generation); zero padding, nothing past it written |
| `archive-disk-absent` | `archive-export` | `19e2fdb3081ed07d` → same | none attached | portal gets `NoDevice` from export and verify; no archive I/O; device 1 offline -14, not found |
| `archive-disk-small` | `archive-export` | `19e2fdb3081ed07d` → same | 108 sectors, `77b4de3c3d4812a4` → `98174475e096a82f` | `Full`; exactly 100 sectors written, equal to the reference prefix; bytes past the scope unchanged on the host; the reader refuses both the cut range and the whole disk |
| `archive-disk-restore` | `archive-disk-export` | blank `49c40c316234e2e4` → `aef1d5b3cd70a4a2` | `87d0166bc7334dd2` → same (read-only) | verify then restore; content, name, length, collection, tags and kind compared byte for byte with the archive; revision checked per the restore rule (1, or 2 after metadata, one version), not the source revision; guest digests equal the oracle |
| `archive-disk-wrong` | `archive-disk-export` | blank `49c40c316234e2e4` → same | `87d0166bc7334dd2` → same (read-only) | portal granted the device-0 archive range instead: verify `Corrupt`; disk 1 still holds a valid archive and saw no I/O |
| `archive-disk-cut` | `archive-disk-export` | `19e2fdb3081ed07d` → `e2c8e3c2e64bca10` (one new collection) | `87d0166bc7334dd2` → `583108aed98be8ef` | `VOLUME:CUT:ARCHIVE` after 20 completed disk-1 writes, then a host SIGKILL (raw -9) of the live guest |
| `archive-disk-verify` | `archive-disk-cut` | blank `49c40c316234e2e4` → same | `583108aed98be8ef` → same (read-only) | cold boot, verify returns `Corrupt`, nothing restored |

**Power cut, which outcome and why.** The older archive is the guest-written
export of `archive-disk-export` (sha256 `d75aa5c8…`, 180 sectors). Before the
new export the portal adds collection 3, so the new header differs (collection
count and generation). The host derives the expected new archive from the
older one (new header, collection records with 3 "Nieuw" added, then the older
object stream) and requires that sector 0 holds that well-formed v1 header
(magic, version, seal, one more collection, same objects and content bytes, the
domain-0 generation of disk 0, stored length +48) and sealed collection
records, that the changed sectors start at sector 0 and end before sector 20,
and that they equal the expected new prefix. In this run exactly sectors 0..19
differed: the new header and records over the older body. Disk 0 is compared
with its input: same objects, chunks, domain-1 digest and other collections;
generation and domain-0 clock +1; only collection 3 "Nieuw" newly present at
revision 1. No disk-0 write counters exist for this stage (the guest stops
before its summary). The host reader refuses it
("content checksum of slot 3 differs"), so neither the older nor a complete new
archive is present and `Corrupt` is the only correct verdict; the guest's
verify returned exactly that. That `verify` accepts the other two outcomes
(older unchanged, complete new) is shown by the Rust host model in
`services/volume/tests/archive.rs`, not by this native run;
`tests/test_volume_disk1.py` only shows that the host classifier does not call
them torn, and that it rejects a corrupted magic alone, a wrong generation or
collection, a changed sector that is not the new prefix, and a disk-0 change
beyond the one collection.

## Evidence lines (kernel serial, per stage)

`VOLUME:ARCHIVE:DEVICE:` (1, or 0 for the wrong-device grant),
`VOLUME:ARCHIVE:DISK_FOUND:`, `VOLUME:ARCHIVE:DISK_OFFLINE:` (14 when absent),
`VOLUME:ARCHIVE:SCOPE_REFUSALS:` (portal requests past its scope: 1, and 2 for
the too-small disk), `VOLUME:ARCHIVE:ESCAPE_DENIALS:` (2 with a device-1
grant, 0 in the wrong-device stage, which has none),
`VOLUME:BROKER:VOLUME_DISK_WRITES:`/`FLUSHES:` (0/0 for export, absent, small)
and `VOLUME:BROKER:ARCHIVE_DISK_READS:`/`WRITES:`/`FLUSHES:`, which equal the
portal's `VOLUME:ARCHIVE:*` counts (export 720/180/1, restore 1440/0/0, small
4/100/0, verify 168/0/0).

The per-device counters use the same lane index as the routing they count, so
they do not check the routing independently. The stages with a positive host
byte check (export, small, cut, restore) show that requests reached disk 1;
the stages with a negative verdict (absent, wrong, verify) rely on them.

`ESCAPE_DENIALS` is a kernel probe at manifest time: the portal's handle value,
looked up in the service's and in slot 2's handle tables, is refused. It is not
an access the service itself attempted: the acceptance row "volume-service
access to device 1 refused and counted" is NOT tested natively, because the
service makes no runtime attempt on device 1. Its evidence is this probe, the
manifest-time count below and the portal's rights, which lack DERIVE and
TRANSFER. The service and slot 2 hold no
device-1 handle (counted by the existing `ARCHIVE_ESCAPE` check), and the slot-2
app's own forged block calls are counted in `VOLUME:APP_DEVICE_DENIALS:`.

## Not shown

- No physical disk and no host block device: both disks are disposable raw
  images under `build/run/`.
- QEMU's virtio-blk is trusted to follow the protocol; no IOMMU and no
  malicious-device model.
- The power cut is `cli; hlt` plus a host SIGKILL with `cache=writeback`: it
  shows a torn sector prefix in the host file, not loss of a volatile device
  cache or a real power failure.
- Plaintext archive format v1 with an FNV-64 accidental-damage checksum only.
- Restore assigns new created/modified times, revisions and version history
  by design (S09-BACKUP-ARCHIVE.md, restore rule and "What is deliberately not
  in v1"); timestamps and source revisions are compared on export only.
- One shared Policy for both lanes: a failed device-1 reset leaves it
  quarantined and both lanes refuse further requests (fail closed). No stage
  exercises a device reset.
- No runtime volume-service access to device 1 is attempted, so its refusal
  and count are not shown natively (see the escape paragraph above).
- The wrong-device stage grants the device-0 archive range, as the acceptance
  table asks; this is the one exception to B3's "no device-0 archive range".
- `archive-disk-cut` prints no counters (the guest stops before the
  summary); its evidence is the kill and the host comparison of both disks.
- Not rebuilt or run: the other `discover()` profiles (m5, m6, m6b, m6c,
  update); `discover()` is unchanged for them. Kernel clippy is not a CI step
  and was not run. `tools/capture_volume.py` does not yet hash
  `archive.after.img`.
- Not run here: hosted CI, `actionlint`, the system/keyring harnesses (the
  feature cannot be combined with `native-system`; a compile check of that
  combination fails as intended with the `compile_error!`).

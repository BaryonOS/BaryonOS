# B03-F1 — native characterization of REL-05 (floor raised before health)

**Characterization only.** This evidence reproduces a known design hazard on
disposable QEMU. It is not a fix, accepts no criterion and earns no progress
points. Criterion context: B03 / FR-B03 (`docs/FULL-TEST-RELEASE.md`), issue #41.
Finding: REL-05 in `docs/audits/subagents/08-release-supply-chain.md`
(RR-01 in `docs/audits/OS-MULTI-AGENT-AUDIT.md`).

## Hazard

`boot/loader/src/signed.rs` `select()` writes the non-volatile UEFI variable
`NieuwOsMinVersion` (`raise_floor`) as soon as the selected slot's hybrid
signature and image digest verify, before `BOOT:SIGNED:OK`, before
ExitBootServices and before the kernel has run. A correctly signed release
that crashes therefore leaves the anti-rollback floor at its version, and the
previous known-good slot is refused as `DOWNGRADE` on every later boot
(ADR-049 semantics).

## Source

| Item | Value |
| --- | --- |
| Base commit | `a91d5ade2ab4d18120f4247263f7bd43cdefa080` (origin/main) plus the B03-F1 commit that adds `tools/characterize_update_floor.py` and `tests/test_update_floor_characterization.py` |
| Source fingerprint (`tools/common.py` `source_hash()`, v3) | `2584f3e6480865218efc5baea0484230a727eaaeb7970f49b469a00789620ac8` |
| Loader (`signed-boot-test`) SHA-256 | `0b11ed453351aebb50121ef81cd7b8fceb4a9572a911c727cc0939915c637d98` |
| v1 kernel `k1` (plain M1, `signed-boot-test` loader) | `9f5c4b12c0ce21166c7e79ba4bf6eb5e3ba9cafa59d4e2e313c2f3882bf45b9e` |
| v2 kernel `k2` (`test-panic`) | `60fa3f26305a6e7c3cac6a25fa40298f747031c7e70c49c8d6f4d6b722412d88` |
| Manifests `m1` / `m2` (public TEST keys, label `TEST`) | `f5efd36b…1716` / `14e1310f…74a5` (full values in `summary.json`) |
| QEMU | `QEMU emulator version 10.2.1 (Debian 1:10.2.1+ds-1ubuntu3.3)`, machine `pc-q35-10.0`, `-accel tcg,thread=single` |
| OVMF | `/usr/share/OVMF/OVMF_CODE_4M.fd` `874f15a7…8aa7`; template `OVMF_VARS_4M.fd` `5d2ac383…5d1e` |

## Method

`python3 -B -u tools/characterize_update_floor.py` builds the signed-boot-test
loader with a healthy M1 kernel (v1) and a `test-panic` kernel (v2), signs both
with `update-tool sign` and the public keys in `tools/update-keys/TEST-*`, packs
a slot store (A = v1, B = v2, no inbox) and reuses the B03 QEMU machine of
`tools/run_update.py`. The hazard sequence uses one persistent `OVMF_VARS.fd`
and one persistent `store.img`; only the unsigned header hint is set by the
host before each boot. It exits 0 only when every oracle below holds.

| Boot | State | Header | Observed loader/kernel markers | Raw exit |
| --- | --- | --- | --- | --- |
| `control-v1` | fresh variables, own store | A | `BOOT:FLOOR:0`, `BOOT:SLOT:A:VERSION:1`, `BOOT:SIGNED:OK`, `BOOT:EBS_OK`, `KERNEL:ENTRY`, `BOOTINFO:OK`, `M1:PASS` | 33 |
| `boot-1-v2-panics` | fresh variables | B | `BOOT:FLOOR:0`, `BOOT:SLOT:B:VERSION:2`, `BOOT:SIGNED:OK`, `BOOT:EBS_OK`, `KERNEL:ENTRY`, `BOOTINFO:OK`, `M1:FAIL:KERNEL_PANIC` | 35 |
| `boot-2-floor-kept` | same variables and store | B | `BOOT:FLOOR:2`, `BOOT:SLOT:A:REJECTED:DOWNGRADE`, `BOOT:SLOT:B:VERSION:2`, …, `M1:FAIL:KERNEL_PANIC` | 35 |
| `boot-3-rollback-refused` | same variables and store | A (host rollback) | `BOOT:FLOOR:2`, `BOOT:SLOT:A:REJECTED:DOWNGRADE`, `BOOT:SLOT:B:VERSION:2`, …, `M1:FAIL:KERNEL_PANIC` | 35 |

No boot timed out; each took about 1.6 s.

## Oracles

- Exact ordered list of `BOOT:`, `KERNEL:`, `BOOTINFO:`, `M1:` and `UPDATE:`
  lines per boot, plus the raw QEMU exit (33 healthy, 35 panic). Raw exit 35 is
  shared by a loader failure and a kernel panic, so the panic line, the
  preceding `KERNEL:ENTRY` and the absence of any `BOOT:FAIL` are required.
- `tools/update_disk.py` byte oracle: the store is byte-identical before and
  after every boot; slots A and B hold exactly the v1 and v2 bundles and each
  image matches its manifest digest; the header holds the slot the host set.
- The store that boot 3 refused is byte-identical to the store the control
  booted healthy (`9c08e29f…` in both `result.json` files); only the persistent
  floor differs.
- The UTF-16LE variable name `NieuwOsMinVersion` is absent from the OVMF
  template and present in the persistent variable store after boot 1, before
  the v2 kernel ever reported health. The floor value itself (2) is read back
  by the loader in boots 2 and 3; the host does not parse the EDK2 variable store.

## Mutations (performed, then reverted byte-exactly)

| ID | Temporary change | Result |
| --- | --- | --- |
| L1 | `signed.rs`: remove the `raise_floor` call | Runner exit 1 at the control boot (no floor variable) |
| L2 | `signed.rs`: `raise_floor(fw, manifest.version - 1)` | Runner exit 1 at boot 2: `BOOT:FLOOR:1`, `BOOT:SLOT:B:VERSION:2`, no DOWNGRADE |
| E1 | runner: drop the raw-exit check | New unit tests fail |
| E2 | runner: drop the store-unchanged check | New unit tests fail (inbox flip) |
| E3 | runner: expect fixed-loader markers for boots 2/3 | New unit tests fail |

After each revert, `git diff --exit-code` returned 0 and the SHA-256 of the
runner and `signed.rs` matched the pre-mutation values.

## Regression checks on the same source

| Command | Exit | Result |
| --- | --- | --- |
| `python3 -B -u tools/characterize_update_floor.py` | 0 | hazard observed, 4/4 boots |
| `python3 -B -m unittest discover -s tests -p 'test_update_*.py'` | 0 | 24 tests (15 existing harness + 9 new) |
| `python3 -B -u tools/test_update.py` | 0 | `passed=true`, all 18 stages and 6 compiler-guard controls; `build/update-gate.json` carries the same source fingerprint |

## Files

`summary.json` is the runner summary; each boot directory holds its complete
`result.json` (including the raw serial stream and QEMU command) and the raw
`serial.txt`. `SHA256SUMS` covers every file here except itself. The nested
`.gitattributes` keeps the raw bytes unnormalised.

## Limits

- Disposable QEMU/TCG with public TEST keys only; no hardware, no production
  keys, no human audit.
- A `test-panic` M1 kernel stands in for a signed but broken release; slot A
  holds a plain M1 kernel, not the native update service.
- A guest panic is not physical power loss. OVMF variable persistence on QEMU
  is not a rollback-resistance proof.
- The fix (deferring the floor until a health confirmation, or a trial-boot
  scheme) changes anti-rollback semantics under ADR-049 and
  ADR-B03-B04-TARGET-20261119 D3/D4 and requires a project-owner decision.
  Once fixed, this runner is expected to fail by design.

## Re-run on the main-merged source (25 November 2026, cloud session)

The characterization was repeated without changes to the tool on branch head
`4aeacbc5` (origin/main `bd31a78b` merged in), in a different environment from
the run above. Raw files stay in the ignored build directory and are not
committed; the committed evidence above remains bound to its own source.

| Item | Value |
| --- | --- |
| Source fingerprint (`source_hash()`, v3) | `b6bf7c4992b80e697e560e3dc990dd465c3341f1b82dfb4d8b0bdbe919f7511b` |
| Loader (`signed-boot-test`) SHA-256 | `0de32dc9a147837c71936a784c2225a463e00407b0a59fe847c425885a31e0c7` |
| QEMU / machine | `QEMU emulator version 8.2.2 (Debian 1:8.2.2+ds-0ubuntu1.18)`, `pc-q35-8.2` (`NOS_QEMU_MACHINE`) |
| OVMF | `/usr/share/OVMF/OVMF_CODE_4M.fd` and the `OVMF_VARS_4M.fd` template (floor variable absent from the template) |
| Command | `NOS_BUILD_DIR=build/b03f1 python3 -B -u tools/characterize_update_floor.py` |
| Result | exit 0, `observed=true`, `rollback_store_equals_control=true` |
| Boots | control-v1 raw 33; boot-1-v2-panics raw 35; boot-2-floor-kept raw 35; boot-3-rollback-refused raw 35; all oracles passed |
| `summary.json` SHA-256 (not committed) | `6fedad9d8b32c284e9f0ec8a6272dde7e0289be1e854f06a186e224559f20490` |

The hazard therefore persists on current main with a second QEMU/OVMF
combination. This is still characterization only: no fix, no criterion change.

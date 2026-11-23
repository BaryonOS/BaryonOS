# B04-T3 evidence — OVMF Secure Boot TEST chain (23 November 2026)

Local disposable QEMU/TCG run of `tools/run_secure_boot.py`. Specification:
[B04-SECURE-BOOT-TEST-CHAIN.md](../../implementation/B04-SECURE-BOOT-TEST-CHAIN.md).
TEST anchors only. This is not production Secure Boot, not real hardware, not
hosted CI and not acceptance of B04.

## Source and environment

- Baseline commit: `76e7904f884a25c9b3eabc080e1d00256ef31a62` on branch
  `claude/b04-secure-boot-test-20261123`, plus the uncommitted CodeRabbit
  review fixes to `tools/run_secure_boot.py` and
  `tools/tests/test_run_secure_boot.py` that are committed together with this
  capture.
- Source fingerprint v3 (`common.source_hash`): `5995b6460b4d4fa1bafe30ccb6c827e9a2206a8af65d5eb78b5021dfb4553d00`
  (equal to `source_sha256` in every `result.json`).
- QEMU: `QEMU emulator version 10.2.1 (Debian 1:10.2.1+ds-1ubuntu3.3)`, machine `pc-q35-10.0,smm=on`, TCG single-thread, secure pflash.
- Firmware package: Ubuntu `ovmf 2025.11-3ubuntu7.3`
  - `OVMF_CODE_4M.secboot.fd` sha256 `33c413a04e9edf512c504e9b0accbcec4d009d406bca761cc0b8eacc93da60b6`
  - `OVMF_VARS_4M.snakeoil.fd` sha256 `a6b8fface5ce0b5e945717036b0044a5bf4cd89b5ce7468fbd7c971dd0bd20e9` (copied per run; never written in place)
  - `PkKek-1-snakeoil.pem` sha256 `d1ead44fc7748b8c26e0cba2141b0b54a379c0f11c4e593fbc5d7dd995f90265`; host parse of the VARS template: `PK`, `KEK` and `db` each hold 1031 bytes = 28-byte signature-list header + 16-byte owner GUID + this 987-byte DER certificate, so `db` contains exactly this one certificate.
- Tools: sbsign/sbverify 0.9.4, OpenSSL 3.5.5.
- Unsigned `signed-boot-test` loader sha256: `5841d0273d7ec41117562e1563cf46af35f75351d6fd6700d9c7fa17c20fe086`
- B03 TEST kernels: k1 `2abf03c7b55d5c725e525706ffb5cecbc6baa17fc90c69e3edfae9d3e5146c1c`, k2 `3e339a4072ffba7178d40e39e08d654581ed4b0ef305a460ce2aef45c0ee1022`; manifests m1 `03b08e0b6ef7eaa1652615dc2d22d5cca7c5e96630c453e0b7384304586e9d2c`, m2 `6d678a7fcc199166dda5ccdb4a9c14409dace3ce852034fd6ff6e7bc71894ac1`.

## Commands and raw exit codes

```
export NOS_BUILD_DIR=$PWD/build NOS_TARGET_DIR=$PWD/target CARGO_TARGET_DIR=$PWD/target
python3 -B tools/run_secure_boot.py --case all --evidence docs/verification/b04-secure-boot-20261123   # exit 0, 5/5
python3 -B tools/run_secure_boot.py --case signed                                                     # exit 1 (unproven, by design)
python3 -B -m unittest discover -s tools/tests -p 'test_run_secure_boot.py' -v                        # exit 0, 31 tests
python3 -B -m compileall -q tools/run_secure_boot.py tools/tests/test_run_secure_boot.py              # exit 0
```

The runner created this directory itself: `--evidence` now accepts only a new
direct child of `docs/verification/` and reserves it before building. The
previous capture of this directory was removed first (it was this pull
request's own unmerged evidence). NOT RUN for this source: the full
`tools/tests` suite and `tools/check_english_docs.py` (no path-taking CLI).
An earlier full-suite run on the previous source fingerprint does not count
for this one.

## Results

| Case | Loader sha256 | Host sbverify (snakeoil) | Raw QEMU exit | Host timeout | Duration | Observed | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- |
| unsigned | `5841d027…` | fail | -9 (host kill) | yes (120 s) | 120.0 s | BdsDxe `Access Denied -- rejected probably by Secure Boot`; no loader/kernel line; store unchanged | refused, as expected |
| signed | `63a72fcc…` | pass | 33 | no | 3.7 s | `BOOT:FLOOR:0`, `BOOT:SLOT:B:REJECTED:FORMAT`, `BOOT:SLOT:A:VERSION:1`, `BOOT:SIGNED:OK`, `BOOT:EBS_OK` … `M1:PASS`; B03 `install` store oracle | booted, as expected |
| wrong-key | `d8398c52…` | fail (own ephemeral cert: pass, required) | -9 (host kill) | yes (120 s) | 120.0 s | same denial line; no loader/kernel line; store unchanged | refused, as expected |
| tampered | `a8199d83…` | fail | -9 (host kill) | yes (120 s) | 120.0 s | same denial line; COFF TimeDateStamp byte at file offset 128 flipped after signing; store unchanged | refused, as expected |
| kernel-digest | `52713811…` | pass | 33 | no | 3.7 s | `BOOT:SLOT:B:REJECTED:DIGEST`, fallback `BOOT:SLOT:A:VERSION:1`, `BOOT:SIGNED:OK` … `M1:PASS`; B03 `bad-digest` store oracle | loader-side refusal under SB, as expected |

Firmware identity is pinned in the runner and recorded per case: CODE
`33c413a0…`, VARS template `a6b8ffac…`, per-run VARS before boot `a6b8ffac…`
(identical in all five cases). `signed` and `kernel-digest` record
`secure_boot_proof: ["tampered", "unsigned", "wrong-key"]`: the passing refusal
cases of the same invocation with identical firmware hashes. Without such a
case a boot verdict is reported as unproven and fails. Every denial line reads
`BdsDxe: failed to load Boot0002 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x1,0x0): Access Denied -- rejected probably by Secure Boot`
and names the ESP boot option.

Stricter refusal rule in this capture: a refusal case passes only when QEMU
is still running at the 120 s host timeout. An earlier exit, even after the
denial line, fails the case and cannot serve as Secure Boot proof. `wrong-key`
additionally requires that its image verifies against its own ephemeral
certificate, so it shows refusal of a validly signed foreign-key image.

The two snakeoil-signed loaders differ in sha256 because the Authenticode
signature embeds a signing time; both verify against the snakeoil certificate.
Full per-case reports: `<case>/result.json`; raw serial: `<case>/serial.txt`.
`SHA256SUMS` covers every file in this directory except itself and this README.

## Limits and open items

- The refusal cases end with a host kill after 120 s (about 30 times the
  signed boot time). The firmware does not exit by itself after a refusal.
- The denial check depends on the BdsDxe wording of this firmware build.
- One guest at a time: before the capture, `pgrep -a qemu-system` showed a
  guest of another worktree (`n02-native-dhcp`); the capture waited until no
  `qemu-system` process remained and started at 2026-11-23T17:02:34Z. The
  `--case signed` check ran afterwards, also with no other guest.
- The snakeoil key passphrase (`snakeoil`) remains an assumption for the
  Ubuntu build: the host documentation does not state it. Public source for the
  Debian package: edk2 2025.02-8+deb13u1 `debian/tests/shell.py` passes it as
  `password` to `debian/python/UEFI/SignedBinary.py` (`openssl -passin`).
- `BUILD/esp` stays untouched only when the B03 images are fresh; when the
  source fingerprint changed, `prepare()` rebuilds them through
  `test_update.prepare_images`, which rewrites `BUILD/esp` and `BUILD/update`
  (this happened at the start of this run).
- NOT RUN: hosted CI, real hardware, dbx/revocation, PK/KEK/db rotation,
  rollback resistance, measured boot, independent review. B04 remains open.

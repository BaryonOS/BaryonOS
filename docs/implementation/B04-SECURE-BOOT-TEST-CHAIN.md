# B04-T3 — OVMF Secure Boot TEST chain

Status: TEST-only implementation under ADR-B03-B04-TARGET-20261119 D7
(synthetic keys, disposable QEMU). It is a first experiment for the D5
firmware-to-loader link and does not accept criterion B04.

## Invariant

With UEFI Secure Boot enforcing and only the TEST certificate in `db`, the
firmware starts the BaryonOS loader only when the loader's Authenticode
signature chains to that certificate and its image digest matches the
loader. The Authenticode PE image hash does not cover every image byte: it
excludes the optional-header `CheckSum` field, the Certificate Table
data-directory entry and the attribute certificate table itself (Microsoft PE
format, Appendix A, "Process for Generating the Authenticode PE Image Hash",
https://learn.microsoft.com/windows/win32/debug/pe-format#appendix-a-calculating-authenticode-pe-image-hash).
The runner does not exercise modifications in excluded ranges.
After a firmware-verified start, the loader's existing B03 TEST manifest
verification of the kernel slot store still decides which kernel runs.

## Chain under test

| Link | Verifier | Trust anchor (TEST) |
| --- | --- | --- |
| firmware → loader | OVMF `OVMF_CODE_4M.secboot.fd` image verification | `db` of `OVMF_VARS_4M.snakeoil.fd`: one distro snakeoil certificate (also PK and KEK) |
| loader → kernel | `boot/loader/src/signed.rs` (`signed-boot-test`) | public TEST anchors in `tools/update-keys/TEST-*` |

The firmware files and the snakeoil certificate/key come from the host
package `ovmf` (Ubuntu 2025.11-3ubuntu7.3). The snakeoil key is a public
distribution test key. The host documentation does not state its passphrase.
Assumption with a public source: Debian edk2 `2025.02-8+deb13u1`
`debian/tests/shell.py` passes `"snakeoil"` as the `password` argument of
`debian/python/UEFI/SignedBinary.py`, which hands it to `openssl -passin`
(https://sources.debian.org/src/edk2/2025.02-8%2Bdeb13u1/debian/tests/shell.py/).
That is the Debian package, not the Ubuntu build on the host; the value
decrypts the host key. The key is
not committed and is decrypted only into a 0600 file inside the per-run
directory, which is deleted after signing. No new long-lived key is created.

## Cases (`tools/run_secure_boot.py`)

All cases use the same `signed-boot-test` loader binary; only its signature
state changes. Each case runs in a fresh `$NOS_BUILD_DIR/run/secure-boot-*`
directory with its own copy of the variable store and slot store.

| Case | Loader | Expected |
| --- | --- | --- |
| `unsigned` | as built | firmware denial line, no loader/kernel line, host timeout, store unchanged |
| `signed` | signed with the snakeoil key | B03 `install` marker sequence and store oracle, raw exit 33 |
| `wrong-key` | signed with an ephemeral RSA-2048 key (generated and deleted per run) | as `unsigned` |
| `tampered` | snakeoil-signed, then one bit of the COFF `TimeDateStamp` flipped | as `unsigned` |
| `kernel-digest` | snakeoil-signed; slot-B kernel image corrupted | B03 `bad-digest` sequence: `BOOT:SLOT:B:REJECTED:DIGEST`, fallback to slot A, raw exit 33 |

The firmware is pinned: the runner refuses to start unless the secboot CODE and
snakeoil VARS template match the recorded sha256 of ovmf 2025.11-3ubuntu7.3.
A boot verdict (`signed`, `kernel-digest`) counts only when the same invocation
also contains a passing refusal case with identical CODE, VARS template and
per-run VARS hashes; otherwise it is reported as unproven and fails. Running
`--case signed` alone therefore exits non-zero by design.

The tamper offset (COFF `TimeDateStamp`, file offset 128 in this loader) lies
inside the Authenticode image hash but is never executed, so a refusal cannot
come from a loader crash. Host `sbverify` must agree with each construction
before QEMU starts; for `wrong-key` the image must also verify against its own
ephemeral certificate, so the case shows refusal of a validly signed image
from a key outside `db`. A refusal requires the observed BdsDxe
line containing `Access Denied` and `rejected probably by Secure Boot`;
silence or a hang alone does not count. QEMU must also still be running at
the host timeout: an earlier exit, even after the denial line, fails the case
and cannot serve as Secure Boot proof. The line must also name the ESP boot
option, `Boot0002 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x1,0x0)` (virtio
FAT drive, index 0), so a denial of another image does not satisfy the oracle. The machine is the shared B03 machine
(`run_update.qemu_command`) plus `smm=on` and secure pflash, which the
secboot firmware build requires.

## Run

```
export NOS_BUILD_DIR=$PWD/build NOS_TARGET_DIR=$PWD/target CARGO_TARGET_DIR=$PWD/target
python3 -B tools/run_secure_boot.py --case all
python3 -B -m unittest discover -s tools/tests -p 'test_run_secure_boot.py' -v
```

Every run writes `result.json`, `serial.txt` and `stderr.txt` to
`$NOS_BUILD_DIR/run/secure-boot-<case>-<id>/`. `result.json` is written only
after the cross-case Secure Boot proof check, so an interrupted run leaves no
unchecked verdict behind. To also capture repository
evidence, add `--evidence docs/verification/<new-directory>`, for example:

```
python3 -B tools/run_secure_boot.py --case all --evidence docs/verification/b04-secure-boot-YYYYMMDD
```

The `--evidence` path must resolve to a direct child of `docs/verification/`
that does not exist yet. The runner creates it before building anything and
copies each case's `result.json` and `serial.txt` into it. An existing
directory or any other path is refused with exit 1 and left unchanged. If the
run stops before any report is written (for example while building), the
runner removes the still-empty directory it created. A case that raises
instead of completing does not stop the other cases: their reports are still
written, the incomplete case is recorded as `passed: false` with the error,
and the run exits 1. Remove a failed capture before retrying.

The runner rebuilds and re-signs the B03 TEST images (`test_update.prepare_images`)
when the source fingerprint changed. Such a rebuild rewrites `$NOS_BUILD_DIR/esp`
and `$NOS_BUILD_DIR/update`; only with fresh images does the runner leave
`BUILD/esp` untouched (each case then copies it and replaces only its copy of
`BOOTX64.EFI`).

The runner is coupled to B03 internals: `run_update.qemu_command`, `execute`,
`pack_store`, `mutate`, `evaluate`, `check_store`, `load_images`, `STAGES`
(`install`, `bad-digest`), `IMAGES`, `MAX_TIMEOUT` and
`test_update.prepare_images`. `InterfaceTests` in
`tools/tests/test_run_secure_boot.py` fails when their names or signatures
change. It needs `sbsign`, `sbverify`, `openssl`
and the Ubuntu/Debian `ovmf` package paths listed above.

## What this shows and what it does not

Shown, for this firmware build in disposable QEMU/TCG: refusal of an unsigned,
foreign-key-signed and modified loader; start of a `db`-signed loader; and
continued loader-side kernel verification under Secure Boot.

Not shown: a production trust root or enrolment procedure, PK/KEK/db/dbx
update and rotation, dbx revocation, loader or policy rollback resistance,
measured boot/TPM, real hardware firmware, build provenance, hosted CI, or an
independent review. The `signed` profile is the B03 update test kernel, not
the default `tools/run.py --smoke` profile, because only `signed-boot-test`
performs loader-side kernel verification.

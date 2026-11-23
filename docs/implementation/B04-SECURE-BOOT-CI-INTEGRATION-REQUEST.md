# B04-T3 — CI integration request (not applied)

Workstreams may not edit `.github/**`. This request describes what a hosted
job for `tools/run_secure_boot.py` would need; the lead or project owner
decides whether and where to add it.

## Proposed job

- Workflow: `signed-updates.yml` (the existing B03 job already builds and
  signs the same TEST images) or a new `secure-boot-test.yml`.
- Runner: `ubuntu-24.04` or later. The runner depends on the package paths
  `/usr/share/OVMF/OVMF_CODE_4M.secboot.fd`,
  `/usr/share/OVMF/OVMF_VARS_4M.snakeoil.fd` and
  `/usr/share/ovmf/PkKek-1-snakeoil.{pem,key}`; confirm them for the runner
  image's `ovmf` version before relying on them.
- Packages: `ovmf`, `sbsigntool`, `openssl`, `qemu-system-x86` (in addition
  to the existing `setup-qemu` / Rust setup actions).
- Steps:
  1. existing Rust and Python setup;
  2. `python3 -B -m unittest discover -s tools/tests -p 'test_run_secure_boot.py' -v`;
  3. `python3 -B tools/run_secure_boot.py --case all` without `--evidence`
     (`--evidence` accepts only a new directory under `docs/verification/` and
     is for committed local captures, not CI). `NOS_BUILD_DIR` is not set in
     `signed-updates.yml`, so the runner uses the default `build/` in the
     checkout and writes one directory per case,
     `build/run/secure-boot-<case>-<id>/`, holding the final `result.json`
     (including the Secure Boot proof verdict), `serial.txt` and `stderr.txt`;
  4. upload them with `preserve-evidence`, one checkout-relative pattern per
     line, in the format of the existing B03 step:

     ```yaml
     path: |
       build/run/secure-boot-*/result.json
       build/run/secure-boot-*/serial.txt
       build/run/secure-boot-*/stderr.txt
     ```
- Duration: about 6–8 minutes for the three refusal cases (120 s host timeout
  each) plus the image build.

## Risks for the reviewer

- The denial string is the BdsDxe text of the tested `ovmf` build; another
  firmware version may word it differently and fail closed.
- The snakeoil key passphrase is an assumed public value hard-coded in the
  runner (source: Debian edk2 packaging tests, not host documentation); it
  must never become a pattern for real keys.
- The runner pins the CODE/VARS sha256 of ovmf 2025.11-3ubuntu7.3 and fails
  closed on any other build; a CI job needs the same package version or a
  reviewed pin update.
- The Debian changelog records a correction of the enrolled snakeoil
  certificate (edk2 2025.08 series). An older `ovmf` package on a runner image
  may enrol a `db` that does not match the shipped key; the `signed` case would
  then fail closed.
- A CI pass would show the TEST chain on hosted QEMU only; it would not accept
  B04.

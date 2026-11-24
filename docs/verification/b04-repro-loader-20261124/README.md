# B04: deterministic UEFI loader link (24 November 2026)

Scope: diagnosis and removal of the `loader`/`image_payload` divergence reported by
`tools/repro_check.py` (B04-T2). Same host, same checkout path, two isolated
`NOS_BUILD_DIR`/`NOS_TARGET_DIR` pairs, default build profile only. This is not an
independent-environment reproducibility result and grants no B04 acceptance.

- Source commit: `0c590f13` (branch `claude/b04-repro-divergence-20261124`), source-v3 fingerprint
  `60bed151512c905debcdb0bca68ec1616c220cbfba09519b407718de58cd5728`.
- Host: QEMU 10.2.1 (Debian 1:10.2.1+ds-1ubuntu3.3), toolchain `cargo +1.100.0` (rust-lld, LLVM 23.1.5).

## Diagnosis

| Run | Change | `differences` | Raw report |
|---|---|---|---|
| 1 | main `93dabb65` | loader, image_payload | `repro/1-main-red.json`, byte diff `repro/loader-cmp-run-7b3nuvo3.txt` |
| 2 | `/Brepro` only | loader, image_payload | `repro/2-brepro-only-red.json`, byte diff `repro/loader-cmp-run-t821x91j.txt` |
| 3 | `/Brepro` + `/DEBUG:NONE` (uncommitted tree, same flags as `73f80b64`) | none | `repro/3-both-flags-green-dirty-tree.json` |
| 4 | commit `0c590f13` | none (exit 0) | `repro/4-head-green.json` |

Run 1: the two 30 208-byte loaders differ in exactly 10 bytes: the PE COFF `TimeDateStamp`
(file offset 128), the debug-directory `TimeDateStamp` and the CodeView RSDS GUID. lld-link writes
wall-clock time and a GUID for the linked PDB `os_loader.pdb`. Sections `.text`, `.eh_fram`, `.reloc` and all
other bytes are identical; the ESP payload differs only because `EFI/BOOT/BOOTX64.EFI` is the loader.

Run 2: `/Brepro` replaces the timestamps with a content hash, but the hashes still differ.
The PDB contains each run's absolute target-dir path and a random rustc temporary directory
name, and its hash feeds the build identifier. All 20 differing bytes are timestamp/REPRO or RSDS fields.

Primary source (toolchain `rustc +1.100.0 -vV` reports LLVM 23.1.5; tag `llvmorg-23.1.5`, commit
`604d7ddfacefc074968a9ba0d88c667ab31b42bd`):

- `lld/COFF/Driver.cpp` lines 2003 and 2021: `config->timestamp` is `0` or `time(nullptr)`; lines 2604–2628
  select `BuildIDHash::PDB` when a PDB is linked.
- `lld/COFF/PDB.cpp` lines 1618–1620: after the PDB is committed, its GUID is copied into the CodeView
  record (`PDB70.Signature`), so PDB content (including run paths) determines the GUID.
- `lld/COFF/Writer.cpp` `Writer::writeBuildId` lines 2692–2716: with `config->repro` the COFF header and
  debug-directory timestamps become `xxh3_64bits` of the output, which already contains that GUID;
  otherwise they are `config->timestamp`. Lines 108–114: `/Brepro` adds a data-less
  `IMAGE_DEBUG_TYPE_REPRO` debug-directory entry.

Fix (`.cargo/config.toml`, `[target.x86_64-unknown-uefi]`): `-C link-arg=/Brepro` and
`-C link-arg=/DEBUG:NONE`. No PDB and no CodeView record are produced; one data-less REPRO debug
entry remains. Both green loaders hash `2b49fedd1f42…`; the kernel was already identical
(`71c3d9e3…`). `tools/repro_check.py` is unchanged: artifacts are hashed exactly as built and
nothing is normalised. Every run compiled from fresh target directories (no wrapper or cache).

## Regression checks at `0c590f13` (or identical flags)

- `python3 -B -m unittest tools.tests.test_repro_check`: 8 tests OK.
- `python3 -B tools/test.py`: exit 0 (10 acceptance groups).
- `python3 -B tools/run.py --smoke`: raw 33, `M1:PASS`.
- `python3 -B tools/run_secure_boot.py --case all --evidence docs/verification/b04-repro-loader-20261124`:
  5/5 as expected (per-case `result.json`/`serial.txt` in this directory). `tampered` still flips the
  COFF `TimeDateStamp` at offset 128 and is refused by the firmware.
  An earlier attempt was invalid because the tree was edited during the run (source freeze); it was discarded and re-run.
- Independent read-only verifier (fresh agent): MERGE; it confirmed the byte classification, the absence
  of any PDB/debug-directory consumer, fresh compilation and coverage of every UEFI feature build.

## NOT RUN / limits

Hosted CI; other build profiles in `repro_check` (default only); a different host, user, path or
toolchain build; FAT disk image bytes; hardware; human audit. Loss of the loader PDB removes
source-level loader debugging unless a local override is used. A caller-supplied `RUSTFLAGS`
replaces these target flags (pre-existing; it also drops the crypto backend pins). No CI job runs
`repro_check`; adding one needs a protected `.github/**` change.

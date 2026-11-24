# S09 slice E — native power cut inside an L2 restore (24 November 2026)

Evidence for `docs/implementation/S09-L2-NATIVE-EXPORT.md` section 4.1: kernel
cases 29 (`l2-restore-cut`) and 30 (`recover-l2-cut`) in the native M6e volume
gate. Task S-20261124-3-B1, branch `claude/s09-l2-restore-cut`, based on
`main` at `dfca280a`.

## Tested source

- Commit: `ea59c6fc430fb2481dd5d331ac2763235535f110`.
- Source fingerprint (v3, `tools/common.py` `source_hash`):
  `89b7a330b79a70d2a4f219a23ac723f1652b97b2af2b4153f31ee1f2240ceae4`. The
  fingerprint was re-computed after the gate and equals the one in
  `volume-gate.json`. This README and the spec edit are outside the
  fingerprint (`docs/`).
- Host: Linux, QEMU 10.2.1 (Debian 1:10.2.1+ds-1ubuntu3.3), local run. This
  is local native evidence; hosted `m6e-native-volume` has not run on this
  head.

## Cut point and recovered-state invariant

- The restore on the fresh image writes the format marker (1 table sector),
  then one 34-sector table copy per commit: collections 1–3, then objects 1–5
  in slot order. Case 29 stops the guest (`cli; hlt`, label
  `VOLUME:CUT:L2_RESTORE`) when the table-sector write count reaches
  `1 + 6·34 + 4 = 209`: the fourth sector of commit 7, object 4's
  INSTALL_COMMIT. Object 4's content has been appended but not published.
  The harness then kills the live QEMU process.
- Recovered state, required by the host oracle on the cut image (29) and on
  the cold boot (30): table generation 6, domain-0 clock 6, format 3;
  collections 1–3 and objects 1–3 equal to `expected_l2(archive, 6)` (live
  values, provenance, content bytes); objects 4 and 5, every other slot and
  domain 1 exactly never-used; the seventh table copy torn after exactly 4
  sectors over the generation-5 copy.
- Guest (case 30): every planned object and collection is fully installed
  with provenance equal to the verified plan or exactly never-used;
  observed masks `restored = 0b0000_1110`, `collections = 0b111`, 0
  mismatches; a second `restore_native_l2` returns `Stale` (-10) with
  `committed = 0`; the volume WRITE to the read-only service is refused.
  The kernel requires these values, and the host requires them to equal the
  disk oracle.
- Deterministic: the cut point (a table-sector count on a fresh image, pinned
  for the full restore by case 26) and therefore the generation, the
  installed set and the torn-sector count. Measured on both runs below.

## Commands and results

| Command | Exit | Result |
|---|---|---|
| `NOS_BUILD_DIR=build/s09cut-gate python3 -B tools/test_volume.py` | not captured (background run); summary `status=completed`, `passed=true`, and `main()` returns 0 only then | `M6e gate passed=True stages=32`; 32 of 32 stages passed; total QEMU time 157.0 s |
| — host step `cargo +1.100.0 test --locked -p volume-core` | 0 | inside the gate |
| — host step `cargo +1.100.0 test --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime` | 0 | inside the gate |
| — host step `python3 -B -m unittest discover -s tests -p 'test_volume_*.py'` | 0 | 199 tests, OK (1 skipped) |
| `python3 -B -m unittest discover -s tests -p 'test_ci_workflows.py'` | 0 | 39 tests, OK (both pins at 32) |
| `python3 -B -m unittest discover -s tools/tests -p 'test_*.py'` | 1 | 996 tests; 2 failures and 9 errors in `test_agent_isolation`, `test_agent_pipeline` and `test_roadmap_runner`, identical to the unchanged base (`git stash`), unrelated to this change |
| `python3 -m compileall -q tools` | 0 | |
| `cargo +1.100.0 fmt --all -- --check`; `cargo +1.100.0 fmt --manifest-path sdk/Cargo.toml --all -- --check` | 0; 0 | |
| `cargo +1.100.0 clippy --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-volume-docs-example --features native,l2-portal --bins --target x86_64-unknown-none -- -D warnings` | 0 | |
| `cargo +1.100.0 clippy --offline --locked --manifest-path sdk/Cargo.toml --all-targets -- -D warnings` | 0 | |

Fast loop before the gate, on a copy of a passing `l2-export` image
(`s09-l2-slice-cd/build/gate2/run/volume-image-05494b59ca`, `source_sha256`
set to the diagnostic build's fingerprint, the same `89b7a330…`):
`l2-restore-cut` passed (raw -9 after host kill), then `recover-l2-cut` on
its image passed (raw 33).

Portal size (`l2-portal` build of `volume-app`): `.text` 48 501 bytes,
`.data` 1 088, `.rodata` 4 296, within the 64 KiB / one page / 32 KiB bounds.

## Stage list (gate order)

normal, recover, archive-export, archive-restore, archive-corrupt,
archive-conflict, archive-full, archive-ustar, archive-disk-export,
archive-disk-absent, archive-disk-small, archive-disk-restore,
archive-disk-wrong, archive-disk-cut, archive-disk-verify, cut-data,
recover-cut, cut-table, recover-cut, cut-txn, recover-cut, corrupt, install,
install-recover, install-cut, recover-cut, l2-export, l2-restore,
l2-corrupt, l2-used, **l2-restore-cut** (raw -9, host kill of a live guest),
**recover-l2-cut** (raw 33). All 32 passed.

## Logs (local, ignored `build/`)

- Gate summary: `build/s09cut-gate/volume-gate.json`; console log
  `build/s09cut-gate.log`.
- `build/s09cut-gate/run/volume-l2-restore-cut-59c76dc0df/` and
  `build/s09cut-gate/run/volume-recover-l2-cut-b070073b7e/` (`serial.txt`,
  `result.json` with the oracle, `volume.after.img`, `archive.after.img`).

## Not run

- Hosted `m6e-native-volume` and `kvm-pilot` on this head: no push was made
  (builder task).
- Kernel clippy: the kernel is built only through `tools/build.py`; its
  build passed for both new features without new warnings.
- Independent review of the slice: pending. Human audit: not done.
- Cut points other than inside an INSTALL_COMMIT copy (inside
  INSTALL_COLLECTION, between INSTALL_APPEND writes) are not native; they
  remain host-tested.

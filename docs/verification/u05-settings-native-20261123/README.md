# U05-N1 native settings persistence evidence (23 November 2026)

**Scope.** This directory holds native QEMU evidence for one bounded path: per-domain
settings persistence through the M6e volume service. It is **not U05
acceptance**. It covers no UI, user/account scope, per-user encryption,
rollback resistance, physical power-loss behaviour or human review. SHA-256 in
the settings records detects accidental damage only; it is not authentication.

- Branch `claude/u05-settings-native-20261123` (base `origin/main` 2f4ac6ae).
  The commit that adds this directory is the source of these runs.
- Source fingerprint (`tools/common.py source_hash`, v3):
  `be030a90ac63b9da0eb7684698c45dc0f40792b5b91737881d03e8640cd6312f`.
  It was rechecked after the runs.
- QEMU `pc-q35-8.2`, TCG, 1 vCPU. The exact command and version are in each `result.json`.
- Kernel SHA-256 of the `test-volume-settings` build:
  `57cde59c6d2571273025e601e5dfdf13a9a409f8efe0bd39d5a09d84004e6fbe`.
- `settings-app` ELF SHA-256: `3dccc51957ddd43cd5355f992f7a7d3122a0cb2e670935667671f70661eb067d`.
  Sizes: .text 27717 B (bound 0x10000), .data 1088 B, .rodata 1664 B.
- volume-service ELF SHA-256 (unchanged volume-docs build):
  `a85788aab87921329524f09a8563937206e5e63c4ceeb82a00799f740a26db10`.

## Settings sequence (`tools/run_settings.py`, exit 0)

Each step is one cold boot. Each passed step has raw QEMU exit 33, the marker
sequence ending in `VOLUME:CASE:SETTINGS[_RECOVER]:PASS`, `VOLUME:PASS` and
`M1:PASS`, report words equal to the expected values, and a matching
independent host oracle decision. Report words are (generation, encoded value,
status) for domain A (slot 1) and domain B (slot 2). Status 5 = Corrupt and
6 = UnsupportedVersion.

| Step | Case | Image input | A report | B report | Image SHA-256 before → after |
|---|---|---|---|---|---|
| 1 | settings | fresh blank | 2, 9830658 (S2), 0 | 2, 13107457 (S2), 0 | 49c40c31…c266 → 53f3e394…3d1b |
| 2 | settings-recover (RO) | step 1 | 2, S2, 0 | 2, S2, 0 | 53f3e394…3d1b → unchanged |
| 3 | settings | step 2 | 3, 6553601 (S1), 0 | 3, 8192000 (S1), 0 | 53f3e394…3d1b → 3ddcd0b2…0dcc |
| 4 | settings-recover (RO) | step 3, A's newest slot damaged | 2, S2, 0 | 3, S1, 0 | 42d8192b…be8c → unchanged |
| 5 | settings-recover (RO) | step 3, both A slots damaged | 0, 0, 5 | 3, S1, 0 | ede93f8f…e9c7 → unchanged |
| 6 | settings-recover (RO) | step 3, A's older slot = schema 2 with valid SHA-256 | 0, 0, 6 | 3, S1, 0 | c224d6bc…8f8b → unchanged |

The full SHA-256 values are in `settings/summary.json` and in each step's
`result.json`. In steps 4–6 the host edits settings bytes and then repairs the
volume FNV sums in both table copies, so the refusal comes from settings-core,
not from the volume. After writable steps 1 and 3 the oracle also checks the
exact on-disk records per slot, the volume generation and domain clocks
(4 with [2,2], then 6 with [3,3]), and that no other object or collection
changed.

Deviation from the brief: step 4 was specified as "gen 1 = S1". That value is
unreachable, because step 3's revert overwrites gen 1. The checked fallback
result is gen 2 = S2.

## Regression (`tools/run_volume.py`, exit 0)

The default volume-docs cases ran on the same source fingerprint: `normal`
(`native-volume`) and then cold `recover` (`test-volume-recover`) on its image.
Both passed with raw exit 33 (`regression/`).

## Host checks (`host/`)

- `cargo +1.100.0 test`: 7 passed, exit 0.
- `clippy --all-targets -D warnings`: exit 0.
- `fmt --check`: exit 0, on `sdk/examples/settings-volume`.
- `rustfmt +1.90.0 --check kernel/src/m2/m3/volume.rs`: exit 0.
- `python3 -B -m unittest tests/test_settings_oracle.py`: 9 tests, exit 0.

`SHA256SUMS` covers every file here. `.gitattributes` keeps the raw serial
logs byte-exact.

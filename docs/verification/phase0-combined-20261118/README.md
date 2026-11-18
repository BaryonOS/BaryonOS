# Combined source verification — 18 November 2026

The authoritative execution checkpoint is `docs/PROJECT-STATE.md`.
[CONTINUATION.md](CONTINUATION.md) records dispositions, ownership and remaining
conditions. This receipt does not close an issue or security finding, change
the fixed 41/100 score, or establish baseline B.

## Exact source and scope

The final runtime checkpoint at this cutoff is
`2a08f4a33eff63b354821f6892e1be28f7602c9b`, fingerprint v3
`25dd6d69621f1d9d4c8ed353633b3db62818562fa6ce50efa4445207505adb44`.
It combines the reviewed inputs in [inputs.json](inputs.json) against actual
main `de9b99047138c4fd0015ffc3f874b273b3537484`. Final English coverage and
documentation-helper repairs are still in progress in #358; final candidate
review and hosted checks remain separate conditions.

The predecessor `44f7405003977dbf7f33eb5bca77f8a8f82c6f1c` passed the same
three local gates. Its separate receipts remain unchanged. The only runtime
source difference is the removal of two obsolete unsafe block markers around
safe Rust 1.100 CPUID intrinsics, with three corresponding comment prefixes.
All maximum-leaf, MAXPHYADDR, range, unsafe firmware-read and PAT/MSR contracts
remain intact. Rustfmt passed, and independent review found no change to those
guards or operations. Actual installed-compiler probes establish the API
difference; a blocked documentation fetch is not presented as verification.

## Local results

| Gate | Source 44f74050 | Source 2a08f4a3 | Scope |
|---|---:|---:|---|
| Executable `ci.yml#host-quality` steps | 18 steps, exit 0, 356.202 s | 18 steps, exit 0, 303.390 s | Includes pinned local tooling setup and executable host tests/lints; hosted Actions setup and integration are separate |
| `tools/test_volume.py` | 15 stages, exit 0, 329.326 s | 15 stages, exit 0, 347.294 s | Real QEMU volume/archive/recovery/negative cases |
| `tools/test_document_session.py` | 2 VMs, exit 0, 20.488 s | 2 VMs, exit 0, 19.403 s | Real normal edit/save and cold reopen; not GUI or complete M6c acceptance |

The volume gate records twelve successful raw guest exits 33 and three
intentional host power cuts with raw exit -9. Those expected cut cases are
distinct from timeouts or failed cases. The DocumentSession guests both exit
33. Captured reports bind their actual firmware, QEMU, artifacts, source,
disk state and serial output. Local QEMU is 10.0.13 with the corresponding
recorded machine profile; the separately pinned hosted profile must also pass.

The CPUID cleanup removes the two unnecessary-unsafe warnings. Other existing
profile-dependent dead-code and dependency warnings remain visible. Required
conditional dependencies were not removed, and no lint or gate was weakened.

## Reproduction and evidence

Use the repository's pinned Rust 1.100.0 toolchain, installed targets and
paired OVMF firmware. Isolate each gate's `NOS_BUILD_DIR`, `NOS_TARGET_DIR` and
`CARGO_TARGET_DIR` inside its own checkout. Invoke commands through the existing
Token Guard wrapper with raw logs retained:

```sh
python3 -B tools/agent_gate.py --job 'ci.yml#host-quality' --log build/host-quality.log
python3 -B -u tools/test_volume.py
NOS_DOCUMENT_SESSION=1 NOS_SDK_RUNTIME=1 python3 -B tools/test_document_session.py
```

The local host-tool setup and exact artifact hashes are recorded in
`host-44f74050.json` and `host-2a08f4a3.json`. Adjacent `checks-*.json` binds the
raw Token Guard receipts, losslessly compressed logs and captured native
results. `native-volume-*/SHA256SUMS` covers all 46 captured files per source.
The `document-session-*` directories retain complete gate reports and both
per-run result/serial/stderr sets. Git attributes preserve raw evidence bytes,
including serial line endings; CR is declared part of those captured line endings for diff checks. Generated ELF/rlib/disk artifacts remain in the
isolated local validation worktrees; their hashes and reproduction commands
are preserved rather than committing generated binaries.

Known credential values and standard token/private-key patterns were checked
before these artifacts were copied. This bounded check does not replace
GitHub secret scanning, whose alert inventory remains NOT VERIFIED (HTTP 403).

Current-head hosted CI, the final source/evidence comparison, independent
exact-candidate review, actual-main inclusion and post-merge checks remain
mandatory before integration. Historical or local successes do not substitute
for any pending, failed, skipped or inaccessible mandatory condition.

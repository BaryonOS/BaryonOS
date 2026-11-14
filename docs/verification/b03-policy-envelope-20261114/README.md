# B03 TEST policy envelopes — verified native checkpoint

## Source and result

PR [#336](https://github.com/BaryonOS/BaryonOS/pull/336) continues issue #41 after
the squash of #328. Local verification executed published commit
`f34e96a6a52299097dc09baf285699c6f906d301`, including main's N04 follow-up
`ce47359fcd9c62abca50da87ac62183026aac91c`. Its Git tree is
`66416e79f4c7ed282e02d2d9ef9bf701205334a1`; source-v3 is
`31b8b8851db624e311836e1d01b9b98c8612802cc662e133118f1054ea410718`.
Subsequent receipt/status changes affect documentation only.

The explicit TEST feature authenticates complete policy tables and exact
policy/manifest bindings using Ed25519 and ML-DSA-65. Both the UEFI loader and
native ring-3 service execute the 26-case vector suite independently. Each
reports count 26, mask 67,108,863 and PASS in the required boot/service phase.

| Verification | Observed result |
| --- | --- |
| Integrated Rust host tests | 32 passed: 9 library, 6 pinned-policy, 13 envelope, 4 CLI |
| New Python probe oracle | 8 passed, including moved/forged/incomplete report rejection |
| Existing update harness | 15 passed within the full gate |
| Build-path / CI contract tests | 3 / 26 passed |
| Strict affected-crate Clippy and root/SDK formatting | Exit 0 |
| Native policy probe | Both consumers 26/26; raw QEMU exit 33; no timeout/kill |
| Native probe block effects | Service/broker writes and flushes all zero; complete store unchanged |
| Existing signed-update gate | All 18 stages and all six compiler controls passed |
| Strict preflight, standard build and smoke | Exit 0; smoke `M1:PASS`, raw exit 33, no timeout |
| Earlier transition model, rerun on this source | 92 finite symbolic checks; no native persistence claim |

The complete probe report, full per-stage update reports, standard artifacts,
commands, digests and local QEMU provenance are retained beside this file.
[integration.json](integration.json) is the local receipt index.
[native-update-gate.json](native-update-gate.json) explicitly replaces verbose
compiler streams with their hashes and exact error messages; it is not a full
raw-log copy. [p1-host.json](p1-host.json) retains the earlier local P1 checkpoint.

## Hosted verification and review

The [B03 workflow](https://github.com/BaryonOS/BaryonOS/actions/runs/43614613093)
passed the host, full 18-stage and new native-policy steps. GitHub checked out
synthetic PR merge `b1f611ccf70c728c8902df0a51258ee72b1c34b3`, whose complete
tree equals the verified head above. [hosted.json](hosted.json) records that
identity, step outcomes and compact-artifact checks. Hosted QEMU 8.2.2 and local
QEMU 10.0.13 / `pc-q35-10.0` are distinct executions.

Separate AI source review found and fixed an oracle defect that allowed service
COUNT/MASK before kernel handover. A separate read-only replay recomputed source,
fixture and frozen-artifact hashes, re-evaluated the actual serial bytes and
reconstructed the entire starting store for byte comparison. AI review is not
an independent human security audit.

## Limits and next step

This is authentication of immutable public TEST vectors. It does not implement
policy ingress, persistent policy commits, cut/replay recovery or replacement
ELF activation. The existing firmware image floor may advance, so zero block
writes do not imply zero firmware writes. Ordinary OVMF storage does not prove
whole-state rollback resistance. The probe kernel is 2,096,008 bytes, leaving
1,144 bytes under the unchanged 2 MiB slot limit.

The [implementation contract](../../implementation/B03-TEST-POLICY-ENVELOPES.md)
specifies the wire format and next required storage/ingress, complete-anchor and
fault-oracle contracts. ADR-B03K3 already authorises the bounded TEST/QEMU work.
Production custody, B04 trusted boot and human review remain open. Progress stays
41/100; B03 retains its existing two points and issue #41 remains open.

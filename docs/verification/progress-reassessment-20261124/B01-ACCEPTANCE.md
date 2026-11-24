# B01 — entropy and key management acceptance

**Decision:** accepted on 24 November 2026 against unchanged rubric v1, on main
`2eb1c63d2afdf658f0e71d31f3542e320a7dd041`. The two existing B01 points raise
the roadmap from 41/100 to 43/100. This is a retrospective evidence assessment;
the assessment session did not modify or rerun the OS implementation.

## Unchanged acceptance condition

> Native cryptographic entropy and key lifecycle, with recovery restricted to
> manifest-bound management ingress. Native positive management recovery and
> negative ordinary-app authorization tests remain required.

## Source and retained evidence

The [actual main job](https://github.com/BaryonOS/BaryonOS/actions/runs/45328884511/job/138224957782)
completed successfully. Its collector receipt records checkout and GitHub SHA
equal to the main commit above. The original
[gate report](b01/build/keyring-gate.json), [collector receipt](b01/_meta/receipt.json),
all seven per-run JSON results and serial logs are retained without modification.
The [artifact provenance](b01/source-artifact.json) records the original download.

- Workflow run: `45328884511`, job: `138224957782`, artifact: `14890773076`.
- ZIP size: 31,934 bytes.
- ZIP SHA-256: `8f7626b019383b480cb5fac83b2db3d6c3efde711380c98c2a852633b5dfcebd`.
- Source fingerprint v3: `2c6ff5ea3ba23b1b76605598095266039358993ac00e93354871caa7aa6d0c6d`.
- All 15 report/log hashes and sizes match the original collector receipt;
  the receipt itself is the sixteenth original extracted file.

The native gate actually ran **seven** cases: normal, no-device, stuck, timeout,
restart, requester-crash and no-reset-ack. Every case reports `passed: true`, raw
exit 33 and no timeout; the aggregate reports `passed: true`. The workflow's
historical step label still says five runs; the executed plan and reports control
this assessment. Host stages, including their explicit ignored microbenchmark,
remain recorded separately in the original report.

## Clause-by-clause decision

| Original requirement | Implemented boundary and observed result |
|---|---|
| Native cryptographic entropy | The virtio-rng broker supplies 1,024 startup bytes to each isolated key service. Missing device, stuck input, timeout, restart and requester/reset failure cases fail closed under the native seven-case gate. The ordinary app cannot obtain direct entropy authority. |
| Native key lifecycle | The normal case generates 17 keys and performs sealing/unsealing. It records four tamper, three cross-domain, two revoked-key and one quota refusal. The retained host stage separately covers crypto and recovery/revocation merging. |
| Manifest-bound management ingress | The kernel provisions a separate management process, private management ingress/reply handles and caller identity. The SDK service validates startup identity and compares the actual IPC sender before decoding or dispatching management commands. Ordinary keyring ingress rejects recovery opcodes. |
| Positive native management recovery | `MGMT_EXPORTS=1`, `MGMT_RESTORES=1` and `B_UNSEAL_OK=1`: service B restores service A's exported key state and decrypts A's ciphertext. The kernel requires management-process completion and actual service counters. |
| Negative ordinary-app authorization | `APP_RECOVERY_EXPORT_DENIED=1` and `APP_RECOVERY_RESTORE_DENIED=1`. An ordinary app's SEND alias to the management ingress yields `ADMIN_BAD_CALLER=1`; it gains no management reply. Wrong recovery secret and an unsupported management GENERATE request are denied. |

The decisive counters are visible in the original
[normal serial log](b01/build/run/keyring-normal-a67cc4cb92/serial.txt) and
[normal result](b01/build/run/keyring-normal-a67cc4cb92/result.json).
Source inspection covers `kernel/src/m2/m3/keyring.rs`,
`sdk/examples/keyring/src/service.rs`, `services/keyring/src/lib.rs` and
`services/keyring/src/wire.rs` at the assessed commit. An earlier deliberately
weakened sender check produced a failing native gate; that separate historical
negative control remains in
[the 18 November checks](../b01-management-recovery-20261118/checks.json).

## Scope and remaining limits

This accepts the literal B01 roadmap condition in the documented native QEMU/TCG
profile. It does not assert physical entropy quality, a TPM, resistance to side
channels, native durable personal-storage rollback protection, external human
security review, or acceptance of the broader FR-B01 release row. Those remain
explicitly governed by their own requirements. No missing evidence is converted
into a pass by a successful build, green badge or AI review.

The previous repeated B01-open labels were administrative state, not additional
acceptance requirements. They are superseded by this clause-level assessment;
the historical records themselves remain unchanged. A regression in any accepted
boundary reopens B01 and removes its two points.

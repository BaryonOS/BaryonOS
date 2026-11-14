# Issue #41 — proposed policy-transition verification

Date: 14 November 2026. Existing PR: [#328](https://github.com/BaryonOS/BaryonOS/pull/328).
Verified integrated source: `a0599501a624822d93ca60718df24fa70a2582e8`. Initial baseline:
`1260de8b5b0bb5330422ab9d5a63a64f5e2603b7`; concurrent issue #42 source
`885e128082483ec871831fbbfc757a6e5104e6b4` is preserved. Later receipt/status edits are documentation only.
Status at this historical verification checkpoint: **REVIEWABLE PROPOSAL;
D1–D7 were unapproved.** Subsequent experimental approval is recorded in
[ADR-B03K3](../../DECISIONS.md#adr-b03k3--authorised-synthetic-policy-transition-experiment);
these earlier results retain their original source and evidence scope.

## Result

The [transition proposal](../../implementation/B03-POLICY-TRANSITION-PROPOSAL.md)
specifies policy/image binding, exact operation identity, atomic anchor commit,
readback, monotonic floors, permanent revocation and eligible fallback. The
proposed formats and state machine do not activate a native or production policy.
The existing ADRs, native code, TEST anchors, workflows and 18-stage update gate
are unchanged by this contribution. The parallel USB work and its prior evidence
are byte-identical to the concurrent head named above.

`model.py` checks a finite symbolic abstraction: **92 checks pass**. This covers
six durable staging cuts, five anchor observations, 32 policy/image-loss cases,
five floor-write observations, exact resume, predecessor-policy and slot identity,
role separation, revocation, binding and fallback cases. After fallback, the
selected running image determines the protected slot; the latest transition
hint alone must not decide it. A permitted old image needs a new-policy binding.

The model's authenticated atoms are assumed verifier results, not cryptographic
verification. Atomic rollback-resistant anchor storage is also an assumption.
Restoring both anchor and disk deliberately boots the old image: this expected
counterexample exposes the D3 boundary instead of claiming it is implemented.

## Executed checks

All rows below ran on the exact integrated source above. Full commands, timing,
source/file hashes and log hashes are in [verification.json](verification.json).

| Check | Exit | Result and limit |
|---|---:|---|
| Symbolic transition model | 0 | 92 finite specification checks; no native or cryptographic evidence. |
| Existing Python update harness | 0 | 15 host oracle tests; does not run the Rust compiler controls or native stages. |
| `tools/progress.py --check` | 0 | 41/100 consistent; criterion register unchanged. |
| `tools/build.py` | 0 | Default loader/kernel build; three ELF load segments inspected. Not the proposed policy implementation. |
| `tools/preflight.py --strict` | 2 | QEMU and OVMF missing. Rust 1.90.0 and both freestanding targets available. |
| `tools/run.py --smoke` | 1 | Stops before guest launch: `qemu-system-x86_64 not found`. No runtime pass. |

The build retains one existing dead-code warning for the kernel serial `hex`
helper. No warning policy or test boundary was relaxed. Earlier runs and failures
remain in the local ignored build logs; the receipt records the final source.

A separate read-only AI reviewed the proposal. Its concrete findings were fixed:
root/release component separation, retained revoked components, image version and
floor rules, fallback policy bindings, precise handover and idempotency language.
The lead's model review additionally corrected policy lineage/slot identity and
running-slot selection after fallback. These AI checks are not an independent
human audit. No native matrix result from earlier work is relabelled as a result
for this specification.

## Reproduction and remaining work

Run `python3 -B docs/verification/b03-policy-transition-20261114/model.py`.
Run the existing harness with
`python3 -B -m unittest discover -s tests -p 'test_update_harness.py'`.
Keep normal preflight/build/smoke and all 18 native update stages.

The finite model does not implement wire parsing, actual signatures, release
windows, physical slot placement, concurrent native processes or all possible
histories/fallbacks. Image/manifest digest and length are represented by one
symbolic image identity. Native backend/encoding/ingress and interruption tests,
physical persistence, B04 boot trust, production custody and independent review
remain open. The next decision is the proposed synthetic experiment's exact
scope; real keys, signing, publication and physical devices need separate
permission. Overall **41/100 → 41/100**; B03 retains its existing two points and
issue #41 remains open.

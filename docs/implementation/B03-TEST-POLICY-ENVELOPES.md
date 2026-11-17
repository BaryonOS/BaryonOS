# B03 TEST policy envelopes and native authentication probe

Date: 14 November 2026. Implementation: P1 codec and P2 immutable native probe.
Contract source: `dafd8fe` (codec), `edfa907` (probe); execution evidence is recorded
in [the source-bound report](../verification/b03-policy-envelope-20261114/README.md).
Scope follows **ADR-B03K3** in [DECISIONS](../DECISIONS.md) and the
[transition proposal](B03-POLICY-TRANSITION-PROPOSAL.md). Production decisions,
persistent transitions and the proposal's abstract anchor are not implemented here.

## 1. Canonical policy envelope

The implemented parser is [policy_envelope.rs](../../crates/update-core/src/policy_envelope.rs).
Integers are unsigned little-endian. The body has exactly `112 + 2008*n` bytes,
where `1 <= n <= 8`; exactly 3,373 signature bytes follow. Optional fields,
alternate encodings, trailing bytes and nonzero reserved bytes are rejected.

| Body offset | Bytes | Field |
|---|---:|---|
| 0 | 8 | Magic `B03POL01`. |
| 8 | 2 | Format version, exactly 1. |
| 10 | 2 | Signer count `n`. |
| 12 | 4 | Exact body length. |
| 16 | 16 | Exact target `B03-QEMU-TEST01\0`. |
| 32 | 8 | Positive policy version. |
| 40 | 8 | Positive minimum image version. |
| 48 | 32 | Predecessor policy-body SHA-256. |
| 80 | 32 | Root identifier: SHA-256 of `B03ROOT1`, root Ed25519 key, root ML-DSA-65 key, concatenated in that order. |
| 112 | `2008*n` | Complete signer table. |

Each entry contains Ed25519 public key (32 bytes), ML-DSA-65 public key (1,952),
first release version (8), last release version (8), revoked flag (1; 0 or 1),
and seven zero bytes. Windows are inclusive, positive and nonempty. Entries are
strictly ordered by full Ed25519-plus-ML-DSA public-key bytes. Duplicate full pairs
or duplicate pairs of the existing eight-byte SHA-256 key IDs are rejected.
No release entry may reuse either root public-key component. An active entry
may not share either component with a revoked entry in the same table.

The body is followed by Ed25519 signature (64 bytes), then ML-DSA-65 signature
(3,309 bytes). Both authenticate the entire identical body against a separately
supplied root; the envelope does not select its own root. The maximum envelope
is **19,549 bytes**. Policy identity is SHA-256 of the body, excluding signatures.
`verify_policy` returns a `VerifiedPolicy` with private authenticated state.
Zero predecessor bytes do not create an enrolment capability.

## 2. Binding an image to a policy

The existing signed **3,500-byte `NOSMAN01` manifest remains unchanged**.
The additional binding envelope is exactly **3,461 bytes**:

| Offset | Bytes | Field |
|---|---:|---|
| 0 | 8 | Magic `B03BND01`. |
| 8 | 16 | Exact experimental target. |
| 24 | 32 | Authenticated policy-body SHA-256. |
| 56 | 32 | SHA-256 of the complete 3,500-byte signed manifest. |
| 88 | 64 | Ed25519 signature over bytes 0–87. |
| 152 | 3,309 | ML-DSA-65 signature over the same bytes 0–87. |

`VerifiedPolicy::verify_bound_image` selects the release pair from the manifest's
key IDs, rejects revoked authority, authenticates both manifest signatures,
enforces the release window and `version >= max(caller_floor, policy_minimum)`,
authenticates both binding signatures with that same pair, and checks exact
image length and SHA-256. It does **not** establish ELF loadability or persistence.

`validate_successor` checks the same root lineage, strictly increasing policy
version, exact predecessor body digest and retention of every revoked pair as
a revoked tombstone. This is an in-memory check, not a committed transition,
idempotence protocol or independently durable monotonic floor.

## 3. Explicit features and native paths

| Component | Explicit feature/profile | Path and responsibility |
|---|---|---|
| Codec | `update-core/experimental-policy-test` | Enables the experimental parser and verifier. |
| Shared probe | `update-core/test-policy-envelope-probe` | Enables the codec and TEST anchors; `policy_envelope/probe.rs::run` verifies immutable vectors without I/O. |
| Loader | `signed-policy-probe-test` | `boot/loader/src/signed.rs::select` runs the probe before ordinary signed-slot selection. |
| Service | `test-policy-envelope-probe` | `sdk/examples/update/src/service.rs::_start` independently runs it after ABI validation, before block I/O. |
| Kernel | `native-update-policy-probe` | `kernel/src/m2/m3/update.rs::finish` checks and reports service evidence and broker write counts. |
| Host gate | `tools/test_policy_envelopes.py` | Builds the paired profile, boots a disposable image and checks transcript, artifact hashes and disk bytes. |

These features are absent from defaults. The existing `signed-boot-test` and
`native-update` paths retain their meaning; generic signed boot/service paths
without an explicit signing policy retain their compile guards. The new probe
does not grant another capability or change `NOSSLOT1`, its sector scope or its
**2,097,152-byte image limit**. The actual probe kernel is separately signed and
ELF-validated through the existing boot path.

## 4. Probe cases and observations

The bit positions below are fixed. Each consumer must execute **26 cases** and
produce mask **67,108,863 (`0x03ffffff`)**. Unexpected baseline failure yields
an incomplete report; individual bits are set only after the expected result.

| Bit | Case name | Required result |
|---:|---|---|
| 0 | `policy-authentic` | Accept both root signatures on the canonical policy. |
| 1 | `binding-image-authentic` | Accept binding, signed manifest and exact opaque payload. |
| 2 | `successor-authentic` | Accept authenticated successor lineage; no activation is performed. |
| 3 | `policy-ed-tampered` | Reject altered policy Ed25519 signature. |
| 4 | `policy-pq-tampered` | Reject altered policy ML-DSA signature despite valid Ed25519. |
| 5 | `policy-target` | Reject altered policy target. |
| 6 | `policy-truncated` | Reject truncated policy envelope. |
| 7 | `policy-length` | Reject inconsistent encoded body length. |
| 8 | `policy-root` | Reject identifier differing from the independently pinned root. |
| 9 | `root-ed-role` | Reject a release entry reusing the root Ed25519 component. |
| 10 | `root-pq-role` | Reject a release entry reusing the root ML-DSA component. |
| 11 | `successor-lineage` | Reject an authenticated candidate with the wrong predecessor. |
| 12 | `binding-ed-tampered` | Reject altered binding Ed25519 signature. |
| 13 | `binding-pq-tampered` | Reject altered binding ML-DSA signature despite valid Ed25519. |
| 14 | `binding-policy-digest` | Reject altered policy-digest field. |
| 15 | `binding-manifest-digest` | Reject altered exact-manifest-digest field. |
| 16 | `binding-target` | Reject altered binding target. |
| 17 | `binding-truncated` | Reject truncated binding envelope. |
| 18 | `image-digest` | Reject altered payload bytes. |
| 19 | `image-floor` | Reject image version 2 against caller floor 3. |
| 20 | `signer-revoked` | Reject a policy-selected revoked release pair. |
| 21 | `signer-premature` | Reject a signed release below its inclusive window. |
| 22 | `signer-expired` | Reject a signed release above its inclusive window. |
| 23 | `policy-trailing` | Reject a policy with an appended byte. |
| 24 | `policy-domain-confusion` | Reject manifest bytes presented as a policy. |
| 25 | `binding-domain-confusion` | Reject policy bytes presented as a binding. |

Loader output is `BOOT:POLICY_PROBE:COUNT:26`, `BOOT:POLICY_PROBE:MASK:67108863`,
then `BOOT:POLICY_PROBE:PASS`. Service observations occupy startup-page byte
offsets **408** (count) and **416** (mask); they confer no authority. The kernel
prints corresponding `UPDATE:POLICY_PROBE:COUNT/MASK` values as 20-digit decimals,
then `UPDATE:POLICY_PROBE:PASS`, after `UPDATE:MANIFEST:PASS` and before `UPDATE:READY`.
The host rejects incomplete, duplicate, altered or misplaced reports, missing
handover/resource markers, abnormal exit, timeout, kill, writes and flushes.
It requires raw QEMU exit 33, `M1:PASS`, an empty inbox and byte-identical slot disk.

## 5. Reproducible public fixtures and evidence limits

The host-only generator `crates/update-core/examples/policy_probe_fixtures.rs`
uses publicly disclosed TEST root seeds `0x71/0x72` and release seeds `0x81/0x82`,
each repeated to 32 bytes. Native consumers embed public keys and signed vectors,
not these private signing seeds. Files and `SHA256SUMS` reside in
`crates/update-core/src/policy_envelope/probe-fixtures/`. Reproduce and compare with:

```sh
cargo +1.90.0 run --offline --locked -p update-core --example policy_probe_fixtures --features experimental-policy-test,test-signing-anchors -- --check
```

`--write` regenerates those public source vectors. The probe payload is 64 opaque
bytes, not an ELF image. Embedded vectors establish native cryptographic execution,
not attacker-controlled bundle ingress, persistent policy commit or recovery.
The regular loader may advance its existing image floor in disposable OVMF variables;
zero service/broker writes and unchanged disk bytes do not mean zero firmware writes.
Ordinary OVMF persistence supplies no whole-state rollback-resistance proof.
The existing 18-stage `tools/test_update.py` gate and standard integration gates
remain separately required. B04, production custody and independent human review remain open.

## 6. Required contracts before persistent transitions

1. **Storage/ingress:** freeze bounded policy/binding/staging positions, lengths,
   permissions and resource budgets without reinterpreting existing slot fields;
   preserve the running image and all committed/fallback bindings. Specify the
   trusted current-policy information supplied to the service and validate ELF
   loadability, flushes and independent byte readback before any commit.
2. **Anchor/operation identity:** select a TEST backend and exact complete-anchor
   encoding, revision, policy digest/version, image floor and transition identity.
   Specify serialized loader-only writes, predecessor comparison, durable readback,
   missing/torn/conflicting state refusal, lost acknowledgements and explicit bootstrap.
   Ordinary firmware `SetVariable` must not be labelled atomic CAS or rollback-resistant.
3. **Native cuts and replay:** retain the existing gate; test every actual staging,
   anchor and floor boundary. Inspect disk and anchor independently, including
   disk-only replay and a whole-disk-plus-OVMF replay counterexample. Successful
   immutable-vector tests do not satisfy these transition acceptance conditions.

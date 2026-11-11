# B03 — signed updates and rollback (test profile `native-update`, loader `signed-boot-test`)

Date: 29 October 2026. Status: **design contract, then implemented in
the same assignment**; checks performed and their outcome are recorded exclusively
in [B03-ACCEPTANCE](../verification/B03-ACCEPTANCE.md). Criterion B03
(docs/progress.json): *authentic updates, interruption, downgrade attacks
and recovery to a valid previous version are tested end to end.* Fulfils
P03 (system image) from [VERIFIABLE-SECURITY-AND-PQC](VERIFIABLE-SECURITY-AND-PQC.md)
§5 and EU-07 from [EUROPE-AND-RELEASE](EUROPE-AND-RELEASE.md). Decision: ADR-049.

## 1. Problem

Until now the loader loads `kernel.elf` from the read-only ESP without any
provenance check; there is no second slot, no version policy and no way to
install a new kernel. B03 adds a **slot store** on a
block device, a **hybrid signed manifest** per slot, verification in the
loader with pinned anchors, a firmware-bound **version floor** against
downgrades, and a ring-3 **update service** that installs a bundle in two phases
so that an interruption never affects the last valid slot.

## 2. Security invariants (first)

1. **Both signatures or nothing.** A manifest is valid if both the
   Ed25519 signature and the ML-DSA-65 signature over exactly the same
   canonical bytes verify against the pinned anchors. One valid
   signature is insufficient; a missing, swapped or
   signature over different bytes is rejected. No downgrade to a single
   algorithm.
2. **Digest binds image to manifest.** The loader hashes the complete image
   it reads from the slot and compares it with `image_sha256` in the
   manifest before a single byte is executed. Length, target ID, format version and
   slot position are in the canonical bytes.
3. **Version floor.** The loader stores the highest booted version in the
   UEFI NV variable `NieuwOsMinVersion` (own vendor GUID). A slot with
   `version < vloer` is rejected as `DOWNGRADE`, even if the pointer in the
   header points to it. The floor only rises; it never decreases automatically.
4. **Fallback to the previous valid version.** The header pointer is a hint,
   not an authority. If the designated slot fails (signature, digest, floor,
   length, read), the loader tries the other slot. Only if both
   fail does the boot end in a controlled way with `BOOT:FAIL:NO_VALID_SLOT`.
5. **Interruption leaves the active slot intact.** The update service writes
   exclusively to the inactive slot: first image, FLUSH, then manifest,
   FLUSH, then the header with the new pointer, FLUSH. An interruption before
   the last FLUSH leaves the old pointer and the old slot unchanged; the
   installation is idempotent and is repeated on the next boot.
6. **The service verifies by itself.** The update service accepts a bundle
   only after the same hybrid verification, digest check and version check
   (`version > huidige` and `version ≥ vloer`); the kernel knows no formats
   and the loader does not trust the service: both check independently.
7. **No production keys in the repository.** TEST anchors from
   `tools/update-keys/TEST-*` are only available via the explicit
   feature `test-signing-anchors`; the fixture loader uses
   `signed-boot-test`. The generic feature `signed-boot` and the native
   update service without an explicit policy fail at compile time. No
   production policy or release profile has been set up; future
   production anchors require a separately designed and authorized
   release path outside CI (ADR-049).
8. **Firmware boundary named.** OVMF starts the loader without Secure Boot; the
   loader itself is not verified by the firmware (B04 open). B03
   protects the chain from the loader onward: slot choice, manifest, image, floor.

## 3. Formats (`crates/update-core`, `no_std`, host tests)

### Manifest (`NOSMAN01`, 3 500 bytes, canonical little-endian)

| Offset | Field |
|---|---|
| 0 | magic `NOSMAN01` |
| 8 | `format_version` u32 = 1 |
| 12 | `target` u32 (1 = kernel) |
| 16 | `image_length` u32 (≤ 2 MiB) |
| 20 | `version` u64 |
| 28 | `min_version` u64 (lower bound asserted by the signer) |
| 36 | `image_sha256` [32] |
| 68 | `ed25519_key_id` [8] (first 8 bytes of SHA-256 over the public key) |
| 76 | `mldsa_key_id` [8] |
| 84 | `ed25519_signature` [64] |
| 148 | `mldsa65_signature` [3309] |
| 3457 | zero up to 3500 |

The canonical message is `bytes[0..84]`; both signatures are over
exactly those 84 bytes. `key_id`'s must match the anchors; an
unknown key ID is rejected as `UNKNOWN_KEY` without verifying a
signature.

### Slot store (`NOSSLOT1`, 512 B sectors)

| Sector | Content |
|---|---|
| 0 | header: magic, `format_version` u32, `active` u32 (0/1), `generation` u64, for slot A and B each `manifest_sector` u32, `image_sector` u32, `image_capacity_sectors` u32; `inbox_manifest_sector` u32, `inbox_image_sector` u32, `inbox_capacity_sectors` u32; rest zero |
| 1–7 | slot A manifest (3 500 B in 7 sectors) |
| 8–4103 | slot A image (2 MiB; a kernel image with embedded ring-3 programs is approx. 1.5 MiB) |
| 4104–4110 | slot B manifest |
| 4111–8206 | slot B image |
| 8207–8213 | inbox manifest |
| 8214–12309 | inbox image |

Total 12 310 sectors (approx. 6 MiB).

The header carries no signature; it is a hint. Fixed positions are
checked by loader and service against this table (no trust in
header offsets outside the range of the device).

## 4. Loader (`boot/loader`, test feature `signed-boot-test`)

1. Via `LocateHandleBuffer(BLOCK_IO)`, find the first block device whose
   sector 0 carries the slot magic (≤ 32 handles). No device → `BOOT:FAIL:NO_STORE`:
   a `signed-boot-test` loader never falls back to an unsigned
   ESP kernel. Only loaders without the feature (all other profiles)
   use the existing M1 path; for `signed-boot-test` the build places no
   `kernel.elf` on the ESP.
2. Read the header, read the floor from `NieuwOsMinVersion` (missing → 0).
3. For the designated slot and then the other: read the manifest, check
   magic/version/target/length, `key_id`'s, both signatures, read the image
   (`image_length` bytes), SHA-256, compare, and check that the image is a
   loadable ELF (the same parser as the M1 path; a correctly signed
   but unloadable image is `REJECTED:FORMAT`); reject `version < vloer`.
   Serial: `BOOT:SLOT:<A|B>:VERSION:<n>` or
   `BOOT:SLOT:<A|B>:REJECTED:<SIGNATURE|DIGEST|DOWNGRADE|FORMAT|READ>`.
4. Only then raise the floor to the `version` of the chosen slot
   (`SetVariable`, NV+BS), print `BOOT:SIGNED:OK` and continue with the
   existing handover of the already validated image.

Cryptography in the loader: `ed25519-dalek` 2.2.0 (serial backend) and
`ml-dsa` 0.1.1 (RustCrypto, pure Rust), `sha2` 0.10.9. No allocations
beyond the existing page allocations; the image is read into the existing
`MAX_FILE_SIZE` buffer.

## 5. Kernel profile `native-update` and update service

- Kernel: generic block broker (like `native-volume`, same syscalls
  0x40–0x43) with a scope over the entire slot store and a ring-3
  update service (slot 0) plus an observer app (slot 1) that reports the
  outcome. The kernel prints its own image version
  (`KERNEL:IMAGE_VERSION:<n>`, build env `NOS_IMAGE_VERSION`) and knows no
  manifest or slot format.
- Service: reads the header, determines the active slot and the inbox; verifies the
  inbox bundle (invariant 6); if `inbox.version > actieve versie`:
  installs into the inactive slot in the order of invariant 5 and
  reports `UPDATE:INSTALLED:<n>`; otherwise `UPDATE:UP_TO_DATE`. Rejected bundle:
  `UPDATE:REJECTED:<reden>` without a single write.
- Fixture `test-update-cut`: the kernel terminates the VM (real host kill) after
  a fixed number of image writes into the inactive slot, before the header
  write. The next boot must start the old slot and finish the
  installation.

## 6. Acceptance cases (real QEMU runs, one slot store image, one OVMF_VARS)

| # | Run | Required observation |
|---|---|---|
| 1 | install | slot A = v1, inbox = v2; loader `SLOT:A:VERSION:1`, `SIGNED:OK`; service `UPDATE:INSTALLED:2`; host oracle: slot B = v2 bytes + manifest, header `active=B`, slot A byte-identical |
| 2 | boot-v2 | same image/vars: loader `SLOT:B:VERSION:2`; `KERNEL:IMAGE_VERSION:2`; service `UP_TO_DATE`; floor = 2 |
| 3 | downgrade | host sets header `active=A`; loader `SLOT:A:REJECTED:DOWNGRADE`, `SLOT:B:VERSION:2`; kernel v2 |
| 4 | bad-signature | fresh vars; host flips one byte in the ML-DSA signature of slot B and sets `active=B`; loader `SLOT:B:REJECTED:SIGNATURE`, `SLOT:A:VERSION:1`; service reinstalls (`INSTALLED:2`); oracle: slot B restored |
| 5 | bad-digest | fresh image; host flips one image byte in slot B (manifest intact); `SLOT:B:REJECTED:DIGEST`, fallback to A |
| 6 | cut | fresh image, feature `test-update-cut`: real kill during image writes; reboot: `SLOT:A:VERSION:1`, header unchanged, service completes `INSTALLED:2`; third boot: `SLOT:B:VERSION:2` |
| 7 | no-valid-slot | both manifests corrupted: `BOOT:FAIL:NO_VALID_SLOT`, exit 0x11; no kernel marker |
| 8 | tampered-inbox | inbox with a single signature or with `version` lowered below the floor: service `UPDATE:REJECTED:*`, no write (oracle: slot B empty, header unchanged) |

Host oracle `tools/update_disk.py` reads the slot store independently (Python,
without cryptography: byte comparison with the known v1/v2 images and
manifests). Gate `tools/test_update.py`: host tests (`update-core` with
recorded test vectors for Ed25519 (RFC 8032) and ML-DSA-65 (fixed seed
vector from the crate), manifest canonical form, selection policy, planner; `update-tool`
keygen/sign/pack round trip) and then the runs above on one frozen source.

## 7. What B03 does not prove

No firmware/Secure Boot verification of the loader itself (B04), no
key rotation/revocation policy in production (anchors are TEST anchors),
no protection against an attacker with write access to OVMF_VARS, no
updates of user programs or packages (A02), no network distribution
(N track), no physical disk or power loss (an emulator kill is something
else).

## Follow-up B03-KEY-POLICY-1 — version-bound signing authority

30 October 2026, base `0ade8014`. Goal: reject known but revoked,
prematurely used or outside-their-release-window key pairs
in both loader and update service, and install/restart a pre-pinned successor.
Write scope: `crates/update-core`, signed loader,
update service, only update error names in the kernel observer, signing tool,
update harness/fixtures, CI stage name and associated documentation. No
keyring/TPM/NAS interface or new dependency.

Invariant: exactly one Ed25519/ML-DSA-65 pair must be in the `TrustPolicy`
supplied by the trusted code, must not be revoked, both signatures
must verify, and the authenticated release version must lie within the
inclusive window. The table contains 1–8 pairs; empty tables, invalid
windows and duplicate ID pairs fail completely with `KEY_POLICY`. No first-match
bypass to readmit a revoked pair. Modified version bytes
fail on signature before window acceptance. Revocation is unconditional.

`policy::Signer` contains `anchors`, `first_version`, `last_version`, `revoked`;
`TrustPolicy::new(&[Signer])` validates the table; `verify_manifest` returns
only fully authorized manifests. The old top-level verifier remains
an explicit cryptographic primitive for the signing tool/tests, not
the native authorization path. Loader and service call exactly the same
`test_policy::verify_manifest`, with the visibly public fixtures from
`tools/update-keys`. New error names/codes: `KEY_REVOKED`/7,
`KEY_EXPIRED`/8, `KEY_NOT_YET_VALID`/9 and `KEY_POLICY`/10.

Acceptance: 6 new real cryptographic host tests, the existing tool round trip,
harness negatives and the full `tools/test_update.py` gate (18 stages).
The five extra stages are rotation install, cold boot with the successor pair
on the same disk/OVMF state, and three cases in which loader and service
reject revoked/expired/not-yet-valid respectively. The three rejections
require 0 writes/flushes and a byte-identical slot disk; the rotation requires
exactly the successor manifest on disk and preservation of the original slot.

Evidence boundary: version windows are not calendar expiry. There is no trusted
calendar clock, online revocation list, authenticated policy update, automatic
removal of the existing TEST pair or production recovery/rotation ceremony.
An attacker with a still-permitted key can still sign a permitted
release. Rollback of the loader/policy table itself remains B04;
the UEFI floor assumptions of B03 continue to apply. No closing of #41 or new
rubric points on this implementation alone. Execution follows in its own
source-bound evidence report; this section is the contract, not the verdict.

## 8. Deviations during implementation

- **Slot size 2 MiB.** A kernel image with embedded ring-3 programs is
  approx. 1.5 MiB (mostly initialized kernel data); the slots are
  therefore 4096 sectors and `image_length` ≤ 2 MiB. The table in §3 has been
  updated.
- **No separate observer app.** The profile `native-update` starts only the
  update service (one ring-3 process). The three independent parties of the
  acceptance are the loader (verification on the next boot), the service
  (verification before installation) and the host oracle `tools/update_disk.py`
  (bytes on disk). An app without block rights was already proven in M6e.
- **Running slot by version, not by header.** The service determines the
  slot it must not write to as the slot whose verified
  manifest version equals the kernel's reported own image version
  (start word 120); the header is only the ordering hint. Without such a slot
  the service refuses to install (error code -19). Thus a
  manipulated header can never make the service overwrite the active slot
  (acceptance case bad-signature).
- **Resumption visible.** Before the digest check the service checks whether the
  target slot already carries the first image sector of this bundle but no manifest yet,
  and reports that as `RESUMED` (start word 368). The labelled
  crash fixture `test-update-cut` cuts off only a first attempt (64
  completed non-header writes), so that the reboot can finish the installation.
- **Loader stack.** ML-DSA-65 verification holds approx. 200 KiB of stack frames;
  the loader performs slot selection on its own 1 MiB page allocation
  (asm stack switch, efiapi) and in this profile the kernel gives the service 512 KiB
  of user stack.
- **Loader markers.** Besides `BOOT:SLOT:<X>:VERSION:<n>` and
  `BOOT:SLOT:<X>:REJECTED:<reden>` the loader prints `BOOT:FLOOR:<n>` (floor
  read) and reports an empty slot as `REJECTED:FORMAT`. Without a slot store
  the `signed-boot-test` loader stops with `BOOT:FAIL:NO_STORE` (review finding:
  a fallback to the ESP would bypass signatures and floor; §4 has been
  adjusted accordingly).
- **ELF validity is part of slot acceptance.** A correctly signed
  image that the ELF parser rejects is `REJECTED:FORMAT`, so that the other
  slot is tried and the floor never rises for an image that cannot
  run (review finding; stage `bad-elf`).
- **Thirteen stages in nine sequences** instead of eight runs: the sequence
  install → boot-v2 → downgrade shares slot store and OVMF_VARS; bad-signature,
  bad-digest, bad-elf and no-valid-slot are fresh sequences; cut → cut-resume →
  cut-final shares the slot store; the inbox cases (signature, downgrade,
  foreign key) are each a fresh sequence. The gate script builds three
  kernels (v1, v2, v1 with cut fixture), signs with the TEST anchors, creates
  one signed non-ELF image and one ad-hoc foreign key pair, and
  checks the disk with the Python oracle after every stage.
- **User stack limit as a parameter.** `kernel-core::user::valid_return_layout`
  hard-pinned the stack to 0x7ffef000–0x7ffff000; the kernel now passes its own
  `STACK..STACK_END`, so that the 512 KiB profile does not reject valid frames
  (that made the first run hang in `idle()`).
- **Direct device poll.** The broker polls the device once
  non-blocking on `block_wait` (0x41) before returning EAGAIN; the
  installation of 3 048 sectors thus takes approx. 10 s in TCG.

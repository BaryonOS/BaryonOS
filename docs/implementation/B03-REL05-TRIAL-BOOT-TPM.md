# B03 REL-05 — trial boot with TPM 2.0 health confirmation (option A)

**Status:** specification only (task B03-REL05-D1, 25 November 2026). Nothing
in this document is implemented or tested. Decision record:
[ADR-B03-REL05-TPM-20261125](../DECISIONS.md#adr-b03-rel05-tpm-20261125--trial-boot-with-tpm-20-health-confirmation-rel-05-option-a).
Owner decision: issue #41, 25 November 2026 (items O1–O9 below).
Scope: public TEST keys, disposable QEMU with swtpm only. No production keys,
signing, physical TPM, firmware change, production activation or new trust root.

The document separates four things throughout: **design** (this text),
**implementation** (none yet), **executed tests** (none) and **unverified
assumptions** (marked *[unverified]* and collected in section 14).

## 1. Problem (REL-05)

`boot/loader/src/signed.rs` `select()` raises the UEFI floor variable
`NieuwOsMinVersion` to the selected slot's version (`raise_floor`, called
before `BOOT:SIGNED:OK`) as soon as signatures and digest verify, before the
kernel has run. A correctly signed but broken release therefore permanently
makes the previous good release a `DOWNGRADE`. The characterization runner
`tools/characterize_update_floor.py` (branch
`origin/claude/b03-floor-crash-20261123`, PR #514; **not on `main` at
`d436a595`**) reproduces this: a signed v2 that panics raises the floor 0→2 and
v1 is refused on every later boot.

## 2. Owner decision (binding input)

| ID | Requirement |
|---|---|
| O1 | Trial boot with health confirmation; TPM 2.0 is the intended hardware-bound persistent anchor; reuse `drivers/tpm-crb`, `kernel/src/m2/m3/tpm_fixture.rs` (`native-tpm-crb`) where sound. |
| O2 | No UEFI runtime `SetVariable` from the kernel; the kernel calls no UEFI runtime services. |
| O3 | At most 3 trial boots per candidate release; durably recorded attempt count. |
| O4 | Confirm only when a pre-specified set of critical services demonstrably works. |
| O5 | Confirmation bound to exact authenticated release, image identity, slot and trial generation. |
| O6 | Floor raised only after a durable, authorised confirmation. |
| O7 | Failed trial → automatic fallback to the previous permitted known-good release; user data preserved. |
| O8 | Floor never decreases; no unsigned fallback, unauthorised downgrade or silent weakening. |
| O9 | Explicit behaviour for a missing, faulty, cleared or replaced TPM; a TPM counter alone is not proof of full rollback protection. |

## 3. Existing components (read on `d436a595`)

- **Loader** (`signed.rs`, feature `signed-boot`): two slots A/B in a
  `NOSSLOT1` store, hybrid Ed25519 + ML-DSA-65 manifest verification,
  `update_core::choose(header, slots, floor)`, floor read/write through boot
  services before ExitBootServices (EBS). The loader already links `sha2`
  through `update-core`. It contains no TCG2 code; `r-efi` is pinned at
  `=7.2.0` and whether it defines `EFI_TCG2_PROTOCOL` is *[unverified]*.
- **TPM driver** (`drivers/tpm-crb`, `#![no_std]`, `forbid(unsafe_code)`):
  CRB transport plus a codec for Startup, SelfTest, GetCapability,
  NV_ReadPublic, NV_DefineSpace (8-byte counter, password auth), NV_Read of a
  counter and NV_Increment. `MAX_COMMAND` is 128 bytes. It has **no** policy
  sessions, NV_Write, PCR_Extend or PCR_Read. Contract
  `docs/contracts/TPM2-CRB.md`: no product kernel grant, no rollback anchor;
  the normative TCG documents were not read.
- **Kernel fixture** (`tpm_fixture.rs`, `native-tpm-crb`): one supervisor UC
  mapping of the QEMU CRB window, copies ≤ 128 bytes, a read-only property
  query and release. It is an experiment, not a product grant.
- **Update service** (`sdk/examples/update`): installs into the non-running
  slot (ADR-049 decision 4), identifies the running slot from the kernel's
  `KERNEL:IMAGE_VERSION`, writes header `generation` (an unsigned disk hint).
- **BootInfo**: base structure, ACPI extension at offset 192 (64 bytes),
  firmware facts `NOSFWF` at offset 256 (16 bytes, `hw-inventory` loader only).
- **Gate**: `.github/workflows/signed-updates.yml` job `b03-native-update` runs
  `tools/test_update.py` (18 stages in `tools/run_update.py`) without swtpm.
  `native-system.yml` is the precedent for swtpm in CI (`run_tpm.py`).

## 4. Threat model

| Actor | Capability | In scope? | Result under this design |
|---|---|---|---|
| T1 disk-only | Rewrites any byte of the slot store, header, inbox and user data while the machine is off; replays older disk images. | Yes | Cannot create or delete TPM NV state; cannot forge a confirmation; cannot lower the floor; cannot obtain more than 3 trial boots per candidate or re-trial a rejected candidate (section 6.4). Can cause denial of service (corrupt slots). |
| T2 runtime ring-3 compromise | Arbitrary CPL3 code in any process, including a malicious app. | Yes | No TPM access (no capability). A compromise of the **health service** can confirm an unhealthy but authenticated candidate (section 9.3). |
| T3 kernel compromise of a *non-trial* boot | Arbitrary CPL0 code after EBS. | Partially | PCR phase is `CLOSED`: cannot satisfy any write policy, so cannot write trial, floor or confirmation state. With empty TEST owner auth it can undefine indices → detected as anchor loss (denial of service, never a lower floor). |
| T4 kernel compromise of the *trial* boot | Arbitrary CPL0 code in the candidate while the window is open. | Out of scope | The candidate is authenticated signed code; it can confirm itself. Same limit as any measured-boot confirmation. |
| T5 boot-path attacker | Boots a foreign EFI application (no Secure Boot) or modifies the loader. | Out of scope until B04 | Reaches the loader phase (PCR at reset value) and can write every loader-only index, raise the floor (brick) or reset UEFI variables. Needs the authenticated loader chain of B04 (ADR-B03-B04-TARGET D5). |
| T6 TPM cleared / replaced / faulty | Owner or lockout clear, a different TPM, swtpm state deleted, command failures. | Yes (O9) | Detected as anchor loss or fault (section 10); floor kept from UEFI NV; no trials, no floor raise. |
| T7 combined T1 + T6 + reset of `OVMF_VARS` | Every persistent anchor wiped together. | Not defended | Both floors return to 0; an older signed release still present on disk can boot. Mitigation is a compiled-in minimum version in the authenticated loader plus B04 (section 10.4). **This is why a TPM counter alone is not full rollback protection.** |
| T8 power loss | Cut between any two persistent writes, or during a TPM command. | Yes | Every cut point has a defined successor state (section 7). |

The host running QEMU/swtpm and its administrator are trusted in the TEST
profile; snapshotting swtpm state voids the independence assumption.

## 5. Design overview

1. **Phase PCR.** The loader owns one non-resettable PCR, `PCR_PHASE = 13`
   (PCRs 16 and 23 are resettable from locality 0 and therefore unusable). At
   loader start it must read all zero. Before EBS the loader **always**
   extends it with exactly one constant: `OPEN` when it hands over to a trial
   candidate, `CLOSED` otherwise. The kernel extends `SEAL` after it has
   written the confirmation or abandoned it.
2. **Fixed policies.** Every product NV index is written only through a policy
   session: loader-only indices require `PCR_PHASE == 0`; the confirmation
   index requires `PCR_PHASE == H(0³² ‖ H(OPEN))`. The policy digests are
   compile-time constants computed by a host tool; the kernel computes no hash.
3. **Binding by content, not by PCR.** The release, image, slot and generation
   binding lives in the loader-written `TRIAL` record. The kernel copies the
   loader-supplied binding into the confirmation unchanged; ring-3 supplies only
   a health verdict. The next boot accepts the confirmation only if it equals
   `TRIAL` and the slot still verifies to the same manifest.
4. **Generation from a TPM counter.** `GEN` is an NV counter incremented by the
   loader once per trial boot; its values never repeat (section 14, U6).
5. **Floor in two places.** The authoritative floor record `FLOOR` is an
   ordinary loader-only NV index; the UEFI variable remains a second copy.
   Effective floor = `max(UEFI, FLOOR.floor)`. Neither is ever lowered.
6. **Floor raised only at finalization** on the boot after a valid
   confirmation, in the loader, pre-EBS (O2 respected: boot-service variable
   access by the loader, never by the kernel).
7. **Three attempts.** The attempt counter in `TRIAL` is incremented and made
   durable **before** the candidate's first byte runs, so a crash counts.
8. **Fallback.** After 3 unconfirmed attempts the candidate is recorded as
   rejected and the known-good slot (version == effective floor) boots. The
   user-data partition is never written by the loader.
9. **TPM loss.** Absent, faulty, cleared or inconsistent anchor → `DEGRADED`:
   known-good only, no trial, no floor change, explicit marker, TEST-only
   re-provisioning.
10. **Legacy profile untouched until migration.** A new loader feature
    `signed-boot-trial` carries the behaviour; `signed-boot` keeps ADR-049
    semantics until step S10 migrates the gate (section 12).

### Refinement of the brief's direction (justification)

The brief suggested a PolicyPCR on the measured *release digest*. An NV index's
`authPolicy` is fixed at `NV_DefineSpace`, so a per-release PCR value would
require either redefining the confirmation index per trial (owner
authorization inside the loader on every trial) or `PolicyAuthorize`/
`PolicyAuthorizeNV` (a signing key or extra policy index — effectively a new
trust root). The fixed phase value gives the same guarantee under T1–T3: while
the window is open only the candidate's kernel holds the TPM, and the content
binding is verified against loader-only state. The residual (T4) is identical
in both variants. A release measurement for attestation may later be extended
into a separate PCR; it is not needed for O5.

## 6. Persistence layout

### 6.1 TPM objects

All handles are TEST candidates in the owner NV range and must be checked
against the TCG handle registry *[unverified, U9]*. All indices: `nameAlg`
SHA-256, `TPMA_NV_POLICYWRITE`, `TPMA_NV_AUTHREAD` with empty `authValue`,
`TPMA_NV_NO_DA`; **not** `AUTHWRITE`, `ORDERLY`, `POLICY_DELETE`,
`WRITEDEFINE`, `READ_STCLEAR` or `WRITE_STCLEAR`. Ordinary indices also set
`TPMA_NV_WRITEALL` so no partial write is possible.

| Handle | Name | Type, size | Write policy (all include `PolicyCommandCode`) | Writer |
|---|---|---|---|---|
| `0x0100B300` | `TRIAL` | ordinary, 64 B | `PolicyPCR(13 = 0³²)` + `CC_NV_Write` | loader |
| `0x0100B301` | `CONFIRM` | ordinary, 64 B | `PolicyPCR(13 = P_OPEN)` + `CC_NV_Write` | kernel (trial boot) |
| `0x0100B302` | `FLOOR` | ordinary, 64 B | `PolicyPCR(13 = 0³²)` + `CC_NV_Write` | loader |
| `0x0100B303` | `GEN` | counter, 8 B | `PolicyPCR(13 = 0³²)` + `CC_NV_Increment` | loader |
| `0x0100B310`–`0x0100B31F` | cold-freshness anchor domains (counter + 64 B slot per domain) | see [COLD-FRESHNESS-ANCHOR.md](COLD-FRESHNESS-ANCHOR.md) | exception to the attributes above: `AUTHWRITE`, not phase-bound | kernel on behalf of the domain holder (N04 only) |

`P_OPEN = SHA-256(0³² ‖ SHA-256("BARYON-TRIAL-OPEN-V1"))` in the SHA-256 bank,
assuming the loader extends through `HashLogExtendEvent`, which hashes the
event data *[unverified, U2]*. `CLOSED` and `SEAL` are the analogous strings
`BARYON-TRIAL-CLOSED-V1` and `BARYON-TRIAL-SEAL-V1`. Policy digests:
`d1 = H(0³² ‖ TPM_CC_PolicyPCR ‖ pcrSelect{SHA256,13} ‖ H(pcrValue))`,
`authPolicy = H(d1 ‖ TPM_CC_PolicyCommandCode ‖ commandCode)`. A host tool
computes them; host tests freeze them; native tests prove that swtpm accepts
exactly them.

On every boot the loader issues `NV_ReadPublic` for each index and compares
type, attributes, size, `nameAlg` and `authPolicy` with the constants. Any
difference, absence, or `TPMA_NV_WRITTEN` clear on `TRIAL`/`FLOOR` is
**anchor loss** (section 10). The name excludes the data, so this check
detects a substituted policy, not substituted content; content integrity
comes from the write policies.

### 6.2 Records (little-endian, fixed width, no optional fields)

`TRIAL` (64 B): magic `NOSTRL01` (8) · state (1: 0 IDLE, 1 PENDING,
2 REJECTED) · slot (1) · attempts (1, 0..=3) · reason (1: 0 none, 1 exhausted,
2 abandoned) · health list id (2) · reserved 0 (2) · generation (8) ·
candidate version (8) · max rejected version (8, monotonic) · manifest digest
(24 = first 24 bytes of SHA-256 over the 84 signed manifest bytes;
*[design choice, see U10]*).

`CONFIRM` (64 B): magic `NOSCNF01` (8) · format 1 (1) · slot (1) · health list
id (2) · verdict (4: `0x50415353` "PASS" only) · generation (8) · version (8) ·
manifest digest (32).

`FLOOR` (64 B): magic `NOSFLR01` (8) · revision (8, +1 per write, no wrap) ·
floor (8) · last finalized generation (8) · last finalized manifest digest (32).
The 120-byte `B03ANC01` anchor of
[B03-POLICY-TRANSITION-PROPOSAL.md](B03-POLICY-TRANSITION-PROPOSAL.md) §3 may
later replace this record; the write discipline (single loader writer,
revision, read-back) is chosen to be compatible.

The full manifest digest is in `CONFIRM`; `TRIAL` stores 24 bytes to fit 64 B.
The loader compares `CONFIRM.digest[0..24]` with `TRIAL` **and** the full
32 bytes with a fresh hash of the slot's verified manifest.

### 6.3 UEFI variables

`NieuwOsMinVersion` (existing GUID, NV+BS, 8 bytes) keeps its meaning as the
second floor copy. It is written only by the loader during finalization or
upward healing (section 7.3). No new UEFI variable is introduced.

### 6.4 Candidate identity, rejection and alternation

Attempts are counted per candidate `(version, manifest digest)`. A candidate is
eligible for a new trial only if `version > max(effective floor, TRIAL.max_rejected)`.
If a different candidate appears while one is PENDING (only possible through
disk tampering, because the update service refuses to install during a trial,
section 9.4), the pending candidate is recorded as REJECTED/abandoned and
`max_rejected` rises. Alternating between two signed candidates therefore cannot
reset attempt counts: each distinct version gets at most 3 boots, and versions
at or below a rejected one are never trialled again. Release versions must
therefore never be reused.

## 7. State machine

Per-boot loader states (all pre-EBS):

| State | Meaning |
|---|---|
| `INIT` | TPM probed, PCR 13 read, public areas checked. |
| `DEGRADED` | Anchor lost or TPM unusable (section 10). Terminal for this boot. |
| `FINALIZE` | `TRIAL` PENDING with a valid matching `CONFIRM` for its generation. |
| `IDLE_SELECT` | No candidate eligible: boot known-good. |
| `TRIAL_START` | Eligible candidate, attempts < 3: start attempt n+1. |
| `EXHAUSTED` | Pending candidate with attempts == 3 and no valid confirmation. |
| `ABANDON` | Pending candidate no longer present; another one is. |
| `HANDOVER_TRIAL` / `HANDOVER_KNOWN_GOOD` | PCR extended, BootInfo written, EBS. |

Durable cross-boot state is the tuple `(TRIAL.state, TRIAL.gen, TRIAL.attempts,
CONFIRM.gen, FLOOR.floor, FLOOR.last_gen, UEFI floor, GEN)`.

### 7.1 Boot algorithm (normative order)

1. `INIT`: locate TCG2; `PCR_Read(13)` must be zero, otherwise `DEGRADED`
   (`PCR_DIRTY`). Check all public areas; read `TRIAL`, `FLOOR`, `CONFIRM`,
   `GEN`, UEFI floor.
   `TRIAL`, `FLOOR` and `CONFIRM` are read only if `NV_ReadPublic` reports
   `TPMA_NV_WRITTEN`; an unwritten `CONFIRM` means "no confirmation" (normal
   after provisioning), an unwritten `TRIAL`/`FLOOR` is anchor loss.
   Provisioning increments `GEN` once so that it is readable (reading a
   never-incremented counter is expected to fail, U16).
2. Consistency: if `FLOOR.floor > UEFI` → heal: SetVariable(UEFI :=
   FLOOR.floor). If `UEFI > FLOOR.floor`, this is the interrupted
   finalization of 7.3 when `TRIAL` is PENDING, `CONFIRM` is valid for
   `TRIAL.gen` (step 4 checks) and `TRIAL.version == UEFI` → resume at F2.
   Otherwise → `DEGRADED` (`ANCHOR_BEHIND`). Effective floor
   `f = max(UEFI, FLOOR.floor)`.
3. Verify both slot manifests (existing `verify_manifest`), independent of
   the header hint. Hash each verified manifest's 84 signed bytes.
4. **Confirmation processing first** (before the header hint is consulted):
   if `TRIAL.state == PENDING` and `CONFIRM` is well-formed with
   `CONFIRM.gen == TRIAL.gen`, slot/version/digest equal to `TRIAL`, verdict
   PASS, and slot `TRIAL.slot` currently verifies to that manifest → `FINALIZE`
   (7.3). A `CONFIRM` that fails any check is ignored with a marker; it is
   never an error that stops boot. If `UEFI == TRIAL.version` the UEFI copy
   was already written: resume at F2; if also `FLOOR.last_gen == TRIAL.gen`:
   resume at F3.
5. Classify slots against `f`: known-good `K` = verified slot with
   `version == f`; candidate `C` = verified slot with
   `version > max(f, TRIAL.max_rejected)` (highest if both qualify).
6. If PENDING and `C` differs from `TRIAL` in version or digest → `ABANDON`
   (7.4), then continue. If the same `(version, digest)` now sits in the other
   slot, it is the same candidate: attempts carry over, the next T2 records
   the new slot, and a confirmation bound to the old slot is rejected
   (conservative; costs at most one attempt).
7. If PENDING for `C` and `TRIAL.attempts == 3` → `EXHAUSTED` (7.4).
8. If `C` exists → `TRIAL_START` (7.2), even when `K` is absent (a trial
   without fallback; exhaustion then ends in step 8's failure). Else →
   `IDLE_SELECT`: boot `K`; if `K` is absent (for example corrupted on disk),
   stop with `BOOT:FAIL:NO_VALID_SLOT`. Never an unsigned kernel; a slot
   `≤ max_rejected` above the floor is never selected as a substitute.
9. Before EBS: extend PCR 13 with `OPEN` (trial) or `CLOSED` (otherwise). If
   TCG2 is present but the extend fails → `BOOT:FAIL:TPM_EXTEND` (controlled
   halt), because handing over with PCR 13 at zero would grant loader-phase
   authority to the kernel. Without TCG2 no extend is attempted (section
   10.2).
10. Write the BootInfo trial extension (section 8), print markers, EBS.

### 7.2 Trial start — write order and crash outcomes

| Step | Write | Crash after this step → next boot |
|---|---|---|
| T1 | `NV_Increment(GEN)`, then `NV_Read(GEN)` → g (increment returns no value) | `TRIAL` unchanged; generation g skipped (harmless); attempt not counted because the candidate did not run. |
| T2 | `NV_Write(TRIAL = PENDING, C, gen g, attempts n+1)` | Attempt n+1 counted (O3); candidate may not have run; next boot starts attempt n+2 or exhausts. |
| T3 | `HashLogExtendEvent(13, OPEN)` (volatile) | Same as T2. |
| T4 | EBS, kernel runs | Same as T2 unless the kernel wrote `CONFIRM` (7.5). |

The first attempt (n = 0) writes the full candidate binding; later attempts
rewrite the record with the new generation and count. The loader reads `TRIAL`
back after T2; a mismatch → `DEGRADED` (`TRIAL_READBACK`).

### 7.3 Finalization — write order and crash outcomes

| Step | Write | Crash after this step → next boot |
|---|---|---|
| F1 | `SetVariable(NieuwOsMinVersion := TRIAL.version)` — **commit point** | UEFI > FLOOR with a valid confirmation → step 2 resume rule → F2; effective floor already raised. |
| F2 | `NV_Write(FLOOR = {rev+1, floor = TRIAL.version, last_gen = g, digest})`, read back | Resume at F3 (idempotent). |
| F3 | `NV_Write(TRIAL = IDLE, max_rejected unchanged)` | Done; `CONFIRM` for g remains but is inert (generation never recurs). |

The UEFI copy is written first so that a TPM loss between F1 and F2 still
leaves the raised floor in UEFI NV (`DEGRADED` keeps it); writing `FLOOR`
first would lose the raise in that compound fault. After F3 the loader
continues at step 5: the confirmed release is now `K`. A crash before F1
leaves the confirmation durable; finalization is retried. A `FLOOR` read-back
mismatch → `DEGRADED`. UEFI variable write atomicity is assumed (U17).

### 7.4 Exhaustion and abandonment

| Step | Write | Crash outcome |
|---|---|---|
| E1 | `NV_Write(TRIAL = REJECTED, reason, max_rejected = max(old, C.version))` | Next boot sees REJECTED: candidate ineligible, known-good boots. |
| E2 | Boot `K` with `CLOSED`; floor, slots and user data untouched | — |

A crash between T2 of attempt 3 and E1 is the same state as "attempts == 3,
PENDING": the next boot performs E1. This is the automatic rollback of O7.

### 7.5 Kernel confirmation — write order and crash outcomes

| Step | Action | Crash outcome |
|---|---|---|
| K1 | `StartAuthSession(policy)`, `PolicyPCR(13)`, `PolicyCommandCode(NV_Write)` | Nothing durable. |
| K2 | `NV_Write(CONFIRM)` | Confirmation durable → next boot finalizes. A lost response is not retried; K4 decides. |
| K3 | `PCR_Extend(13, SEAL)` | Window closed for the rest of this boot. |
| K4 | `NV_Read(CONFIRM)` and compare | Mismatch/unreadable → `UPDATE:TRIAL:CONFIRM_UNKNOWN`; the loader decides next boot. |

If health fails or the deadline (section 9.2) expires, the kernel performs
only K3, prints `UPDATE:TRIAL:UNHEALTHY` and requests a reset; the next boot
counts or exhausts. NV_Write atomicity for one command is assumed *[U4]*.

### 7.6 Invariants (to be checked by the pure model, step S1)

- I1 The effective floor never decreases across any sequence of boots, crashes
  and T1/T2/T3/T6 actions, including a TPM loss at any point. It is not
  claimed under T5 or T7.
- I2 The floor rises to v only if a `CONFIRM` with PASS existed for a
  generation whose `TRIAL` bound exactly version v, that slot and that digest.
- I3 A candidate executes at most 3 times; each execution is preceded by a
  durable attempt increment.
- I4 After exhaustion or abandonment, a version ≤ `max_rejected` never runs as
  a trial again.
- I5 With a valid known-good slot, a failed candidate never prevents booting
  the known-good release (no floor raise without confirmation).
- I6 No path boots an image that fails manifest/digest verification or is
  below the effective floor; the loader never writes the user-data partition.
- I8 (O7, user data) A candidate performs no irreversible user-data format
  change or migration before finalization: migrations run only in a boot whose
  BootInfo mode is known-good with `version == floor`. This is a rule for every
  service of a release, enforced by review and by the test in C3/C4; the
  current `native-update` profile has no user-data volume, so that part of
  C3/C4 is not testable yet.
- I7 Every loss or inconsistency of TPM state leads to `DEGRADED`, never to a
  lower floor or a trial.

## 8. Authorization boundaries and BootInfo

| Party | May | May not |
|---|---|---|
| Loader (pre-EBS) | Read all indices; write `TRIAL`, `FLOOR`, increment `GEN` (phase 0); UEFI floor writes; extend PCR 13. | Hand over with PCR 13 at zero; lower any floor. |
| Firmware TCG2 | Transport and event log for the loader. | (in TCB, unchanged) |
| Kernel (`native-update-trial` profile) | Claim CRB after EBS (existing fixture); one-shot `CONFIRM` write and `SEAL` extend; cold-freshness counters (separate contract). | Parse manifests, hash, choose slots, write loader-only indices (policy prevents it), accept binding bytes from ring-3. |
| Health service (ring-3) | Hold capability `TrialConfirm` (one use, this boot only); submit `{list_id, verdict}`. | Supply generation, version, slot or digest. |
| Update service (ring-3) | Read trial status. | Install while a trial is pending. |
| Other processes | — | Any TPM access. |

The kernel role is narrow privileged policy (one-shot gate, window seal) and
needs the ADR's explicit authorization under `kernel/AGENTS.md`; the kernel
still contains no cryptography (policy digests are constants; the kernel
copies opaque binding bytes).

**BootInfo trial extension** (new, loader feature `signed-boot-trial` only):
offset 320, 96 bytes, magic `NOSTRI\0\0`, version 1: flags (TPM state: 0
ready, 1 absent, 2 fault, 3 anchor lost; mode: 0 known-good, 1 trial), health
list id, generation, version, slot, attempt number, 32-byte manifest digest,
`CONFIRM` handle. Zero bytes never validate (as with `NOSFWF`). The base
structure's end and the free range 272–416 must be confirmed with a const
assertion in `crates/boot-protocol` *[U11]*. Any `boot-protocol` change must be
booted in an M3 profile, not only M2 (Graphify memory: M2 BSP stack overflow
incident, 5 November 2026).

## 9. Health confirmation

### 9.1 Critical service list `HEALTH-LIST-1` (versioned, compiled into the candidate)

| ID | Check (native-update QEMU profile) | Evidence source |
|---|---|---|
| H1 | Kernel handover invariants: `KERNEL:ENTRY`, `BOOTINFO:OK`, valid trial extension. | kernel |
| H2 | Update service started at CPL3, verified both slot manifests and identified the running slot (`UPDATE:MANIFEST:PASS`, `UPDATE:READY`). | update service |
| H3 | Update service completed its run with an expected outcome (`UPDATE:UP_TO_DATE` or `UPDATE:INSTALL_DEFERRED`) and the kernel's resource reclamation checks passed (the conditions behind `UPDATE:RESOURCES:PASS`). | kernel + update service |
| H4 | Health service itself scheduled at CPL3 with a completed IPC round-trip to the update service. | health service |

`M1:PASS` is the final marker in `kernel/src/m2/m3/update.rs` (line 765, after
`UPDATE:RESOURCES:PASS`), so it cannot be a precondition; under the trial
profile it is printed after K4. How the kernel exposes H1/H3 results to the
health service is not designed yet (U18).

The list is deliberately limited to services that exist in the
`native-update` profile; it does not claim that storage, network or display
work. Later list versions (`HEALTH-LIST-2` …) add user-data volume read,
network and display per profile; the list id is bound into `TRIAL`/`CONFIRM`.
Adding a check never requires a key; removing one requires a new list version
and review.

### 9.2 Procedure

The health service collects H1–H4 within a deadline (TEST: 60 s of PIT time).
All PASS → it invokes `TrialConfirm(list_id, PASS)`. The kernel checks the
list id equals its compiled-in id, that the trial flag is set and the
capability unused, then runs K1–K4 with the binding from BootInfo. Any FAIL or
timeout → K3 only and reset. In QEMU a reset is a guest exit that the harness
treats as a power cycle; on hardware a watchdog/reset path is open *[U12]*.

### 9.3 Residual risk

A compromised health service (T2) can confirm an authenticated but broken
candidate, recreating the REL-05 symptom for that release. This does not admit
unsigned code or lower the floor. Mitigation is limiting the service's
attack surface (no other capabilities, no network) and review; it is recorded,
not solved.

### 9.4 Update service rule during a trial (amends ADR-049 decision 4)

While the BootInfo trial flag is set, the update service performs no slot
write (`UPDATE:TRIAL:INSTALL_DEFERRED`); the non-running slot is the
known-good fallback. Installation resumes after finalization.

## 10. TPM missing, faulty, cleared or replaced (O9)

### 10.1 Detection

| Observation | Classification |
|---|---|
| No TCG2 protocol, or `GetCapability` fails | `ABSENT` |
| Command error, timeout, self-test failure, `PCR_Read(13) ≠ 0` | `FAULT` |
| Any index missing, public area ≠ constants, `TRIAL`/`FLOOR` unwritten, bad magic, read-back mismatch | `ANCHOR_LOST` |
| `UEFI floor > FLOOR.floor` | `ANCHOR_BEHIND` (rollback or replacement of TPM state) |

### 10.2 Behaviour per class

| Class | Behaviour |
|---|---|
| No TCG2 protocol (no TPM, or firmware without TPM support) | `DEGRADED` boot. No extend is possible. Residual: if a TPM device nevertheless exists, a *compromised* kernel (T3) could reach it with PCR 13 at zero and write loader-only state (for example raise `FLOOR` and brick older releases); a benign kernel in this mode does not claim the CRB. |
| TCG2 present, any NV/read error, but the `CLOSED` extend succeeds | `DEGRADED` boot. |
| TCG2 present and the extend fails | Controlled halt `BOOT:FAIL:TPM_EXTEND` (no kernel runs). Recovery: power cycle; persistent failure needs TPM service or the TEST re-provisioning of 10.3. Booting degraded instead is rejected: it would hand over with PCR 13 at its reset value (see the state machine). |

In `DEGRADED`: effective floor = UEFI floor (never lowered). Boot the verified
slot with `version == floor`; if absent, the lowest verified slot ≥ floor (most
conservative). No trial, no attempt write, no floor write of any kind. BootInfo reports the
TPM state; the kernel then does not claim the CRB; the update service defers
installation. Markers `BOOT:TPM:<ABSENT|FAULT|ANCHOR_LOST|ANCHOR_BEHIND>` and
`BOOT:MODE:DEGRADED`. Updates stall until recovery: availability is traded for
O8.

### 10.3 Recovery

TEST only: a host provisioning tool (new, `tools/`) defines the indices on a
disposable swtpm with empty owner auth and writes `FLOOR.floor = max(UEFI
floor, version of the image to be marked known-good)`, `TRIAL = IDLE`. It
never sets a value below the UEFI floor. A production procedure (owner/lockout
auth custody, physical presence, EK binding) is **not** specified and needs an
owner decision.

### 10.4 Why a TPM counter alone is insufficient

A TPM can be cleared (owner/lockout auth, physical presence) or replaced; a
replaced TPM is fully attacker-controlled state. The UEFI copy survives that,
but an attacker who also resets `OVMF_VARS` (T7) removes both. Neither the
TPM nor the UEFI variable authenticates the code that enforces the floor;
that is B04. Additional mitigation (step S9): the authenticated loader carries
a compiled-in minimum image version, below which no slot boots even in
`DEGRADED`. Binding the provisioned TPM identity (EK name) is an open option
(U13). A replaced TPM can at most skip a health check for an authenticated
release; while the UEFI copy is intact it cannot lower the floor.

## 11. Test matrix (planned native QEMU + swtpm cases; none executed)

Each case uses one persistent swtpm state directory, one `OVMF_VARS` copy and
one slot store across its boots, with a new QEMU and swtpm process per cold
boot (`tools/run_tpm.py` pattern). "Cut" means the harness kills QEMU at the
named marker (TEST hook markers emitted before each write in a test feature).

| # | Owner category | Case | Expected loader / kernel markers |
|---|---|---|---|
| C1 | baseline | provisioned v1, no candidate | `BOOT:FLOOR:1`, `BOOT:MODE:KNOWN_GOOD`, `M1:PASS` |
| C2 | success | v2 installed, trial 1 healthy, next boot | boot 1: `BOOT:TRIAL:START:<g>:B:2:1`, `UPDATE:TRIAL:CONFIRMED:<g>`; boot 2: `BOOT:CONFIRM:FINALIZED:<g>:2`, `BOOT:FLOOR:2` |
| C3 | kernel crash | v2 panics ×3 | boots 1–3 `BOOT:TRIAL:START:…:1/2/3`, panic; boot 4 `BOOT:TRIAL:EXHAUSTED:2`, `BOOT:FLOOR:1`, `BOOT:SLOT:A:VERSION:1`, `M1:PASS` |
| C4 | successful rollback | after C3, header hint set to B | `BOOT:TRIAL:REFUSED:2`, A boots, floor 1 |
| C5 | power loss | cut after T1, T2, T3 | attempt counted only after T2; never more than 3 executions |
| C6 | power loss | cut after K2 before K3 | next boot finalizes |
| C7 | power loss | cut after F1, after F2 | resume; `BOOT:FLOOR:2` exactly once raised, UEFI healed |
| C8 | forged confirmation | host writes `CONFIRM`-shaped bytes to disk/inbox; ring-3 test app calls `TrialConfirm` without capability | no TPM write; `BOOT:CONFIRM:` absent; floor 1 |
| C9 | forged confirmation | non-trial boot kernel test hook attempts `NV_Write(CONFIRM)` | TPM policy failure code recorded; nothing changes |
| C10 | replayed confirmation | confirmation for g, then new candidate v3 trial g' | `BOOT:CONFIRM:REJECTED:GENERATION`; no raise to 3 without new confirmation |
| C11 | swapped slots | after confirmation in B, host swaps A/B contents before next boot | `BOOT:CONFIRM:REJECTED:BINDING`, no raise |
| C12 | exhausted attempts | health fails (H4 forced FAIL) ×3 | as C3 with `UPDATE:TRIAL:UNHEALTHY` |
| C13 | alternation | host alternates v2 and v3 candidates | `BOOT:TRIAL:ABANDONED:2`; v2 never trialled again |
| C14 | disk replay | replay a full pre-update disk image after C2 | v1 `REJECTED:DOWNGRADE`, floor 2 from TPM and UEFI |
| C15 | downgrade | old signed v1 in inbox and slot after C2 | `UPDATE:REJECTED:DOWNGRADE`, `BOOT:SLOT:A:REJECTED:DOWNGRADE` |
| C16 | TPM absent | no swtpm device (OVMF finds no TPM) | `BOOT:TPM:ABSENT`, `BOOT:MODE:DEGRADED`, known-good, floor unchanged |
| C17a | TPM fault | swtpm returns errors on NV reads, extend works (test hook) | `BOOT:TPM:FAULT`, `BOOT:MODE:DEGRADED`; no floor write |
| C17b | TPM fault | swtpm killed before the extend | `BOOT:FAIL:TPM_EXTEND`, no kernel markers; floors unchanged on the next healthy boot |
| C3/C4 add | user data | hash of all non-slot disk regions before and after | byte-identical (not testable in the current profile, I8) |
| C18 | TPM cleared | fresh swtpm state after C2, `OVMF_VARS` kept | `BOOT:TPM:ANCHOR_LOST`, `BOOT:FLOOR:2` (UEFI), degraded |
| C19 | TPM rolled back | swtpm state snapshot from before C2 restored | `BOOT:TPM:ANCHOR_BEHIND`, floor 2 kept |
| C20 | combined T7 | fresh swtpm + fresh `OVMF_VARS` + v1 on disk | documents the limit: v1 boots unless the compiled-in minimum (S9) forbids it |
| C21 | index squatting | indices predefined with weaker policy | `BOOT:TPM:ANCHOR_LOST` |
| C22 | dirty PCR | PCR 13 pre-extended (test firmware hook or EFI shell) | `BOOT:TPM:FAULT:PCR_DIRTY` |

## 12. Impact on existing gates and tools

- `tools/test_update.py` / `run_update.py` (18 stages): unchanged while
  `signed-boot` keeps ADR-049 semantics (steps S1–S9). Migration (S10) runs
  every stage with swtpm and a provisioning matched to each fresh sequence's
  known-good version; `BOOT:FLOOR` then prints the effective floor after
  confirmation processing. Consequences: every fresh-sequence `BOOT:FLOOR:0`
  expectation becomes the provisioned floor (1 for sequences starting on v1,
  2 for `inbox-downgrade`, which starts on v2); `boot-v2`, `cut-final` and
  `rotate-boot` become trial boots (`BOOT:TRIAL:START`, floor unchanged) each
  followed by a new finalization stage (`BOOT:CONFIRM:FINALIZED`, raised
  floor); `downgrade` follows the finalization stage; `cut-resume` keeps the
  provisioned floor instead of a floor raised by the interrupted boot; stages
  whose higher slot is corrupt (`bad-*`, `key-*`) stay known-good boots.
  Exact lists are fixed in S10 against real transcripts, not here.
- `tools/characterize_update_floor.py` (PR #514, not on `main`): with the
  trial profile, boot 1 must no longer raise the floor and boot 4 must boot v1;
  the runner must then **fail** (hazard not observed). S10 adds the inverse
  acceptance (C3/C4) as the regression test and records the characterization's
  expected failure; the characterization itself stays historical evidence.
- `tools/run_tpm.py`: reused pattern for swtpm lifecycle; its "no NV mutation"
  scope stays for the existing fixture cases.
- **CI:** adding swtpm to `signed-updates.yml#b03-native-update` (or a new
  job) is a protected `.github/**` change and needs a separate,
  owner-reviewed request; until then the trial cases run locally and in a
  workflow change PR only. A merge does not prove a criterion.

## 13. Implementation steps (each independently verifiable)

| Step | Content | Verification |
|---|---|---|
| S1 | New pure `no_std` module `update-core::trial` (or crate `crates/trial-core`, `forbid(unsafe_code)`): record codecs, boot decision function `decide(observations) -> plan of writes`, finalization/exhaustion logic, a model of durable state with injected crash after every write. | Host: exhaustive enumeration over small version/attempt ranges and every crash point; property tests for I1–I7; denial tests for forged/replayed/swapped records. |
| S2 | Host tool + test computing `P_OPEN`, policy digests and NV public areas; constants committed and frozen. | Host test recomputes with `sha2`; later cross-checked natively in S5. |
| S3 | `drivers/tpm-crb` codec: StartAuthSession (policy, unsalted, unbound), PolicyPCR, PolicyCommandCode, NV_DefineSpace (ordinary/counter with policy), NV_Write, NV_Read (ordinary), PCR_Read, PCR_Extend (SHA-256 only); NV_ReadPublic must decode type, attributes, size and `authPolicy` (today it returns an opaque name). Keep the 128-byte bound; prove each command size. | Host codec tests including truncated/malformed responses; existing transport tests unchanged. |
| S4 | Primary-source check of TPM Part 1–3 semantics U1–U8 and swtpm behaviour; record in `drivers/tpm-crb/verification/`. | Document review; native probe in S5. |
| S5 | TEST provisioning tool and a native probe (kernel fixture extension, test feature) that defines the indices on disposable swtpm and shows the policies accept/refuse as designed. | Native QEMU+swtpm, raw exit 33, markers. |
| S6 | Loader: TCG2 binding (own `repr(C)` if `r-efi` lacks it), PCR 13 read/extend, index checks, S1 decision function, BootInfo extension, new loader feature `signed-boot-trial`. | Native C1, C16, C17, C21, C22; M3-profile boot for the `boot-protocol` change. |
| S7 | Kernel profile `native-update-trial`: CRB claim reuse, `TrialConfirm` capability, K1–K4, deadline, reset. | Native C2, C6, C8, C9. |
| S8 | Health service (ring-3) for `HEALTH-LIST-1`; update service trial deferral. | Native C2, C3, C12; update-service unit tests. |
| S9 | Compiled-in minimum image version in the trial loader; remaining matrix C4–C20. | Native matrix. |
| S10 | Gate migration: run `test_update.py` stages under the trial profile with swtpm; inverse of the characterization; protected CI request, which also adds `tests/test_update_floor_characterization.py` to the `ci.yml` host unittest patterns (not run in CI today; Graphify finding on #514). | Owner-reviewed workflow PR; hosted job. |

## 14. Open questions and unverified claims

- U1 That a policy session satisfied by PolicyPCR + PolicyCommandCode
  authorizes NV_Write with an empty HMAC and empty `nonceCaller` in the
  command authorization (no `PolicyAuthValue`/`PolicyPassword`), so the kernel
  needs no HMAC or hash. TPM Part 1 §19 and Part 3 §23 not yet read.
- U2 `EFI_TCG2_PROTOCOL` availability in the pinned Ubuntu `OVMF_CODE_4M.fd`
  with swtpm, its GUID, `SubmitCommand` and `HashLogExtendEvent` semantics,
  extension of all active banks, and that OVMF extends nothing into PCR 13 or
  leaves the CRB idle at EBS.
- U3 `StartAuthSession` minimum `nonceCaller` size (believed 16 bytes) and
  that a constant, non-secret nonce is acceptable for an unbound unsalted
  policy session.
- U4 Atomicity of a single NV_Write of 64 bytes across power loss (swtpm and
  physical TPMs).
- U5 That PCR_Extend may list only the SHA-256 bank when several banks are
  active, and that PolicyPCR with an empty `pcrDigest` uses current values.
- U6 That a newly defined NV counter starts at the TPM's highest-ever counter
  value, including after `TPM2_Clear` (believed, not checked).
- U7 PC Client profile: PCR 13 non-resettable at locality 0; PCRs 16/23
  resettable.
- U8 Whether OVMF sets a random platform hierarchy auth; impact on T3 when
  owner auth is empty in TEST.
- U9 Owner-range NV handle choice `0x0100B300`–`0x0100B31F` against the TCG
  handle registry.
- U10 Whether a 24-byte digest prefix in `TRIAL` is acceptable (it is
  combined with a full 32-byte comparison); alternative: 96-byte `TRIAL`.
- U11 Free BootInfo range for the trial extension.
- U12 Reset/watchdog behaviour on non-QEMU hardware after a failed health check.
- U13 Binding the provisioned TPM identity (EK) to detect replacement earlier.
- U14 Release versioning: versions must never be reused (6.4); to be added to
  the production release policy (B03-PRODUCTION-POLICY-PROPOSAL).
- U15 The legacy `signed-boot` profile retains the REL-05 hazard until S10.
- U16 TPM responses for reading a never-written ordinary index or a
  never-incremented counter (believed `TPM_RC_NV_UNINITIALIZED`), and the
  semantics of `TPMA_NV_WRITTEN` and `TPMA_NV_WRITEALL`.
- U17 Atomicity of an OVMF NV variable write across power loss.
- U18 Kernel-to-health-service interface for H1/H3 observations.
- U19 Whether the pinned `r-efi` 7.2.0 defines `EFI_TCG2_PROTOCOL`; otherwise
  the loader needs its own `repr(C)` definition from the TCG EFI protocol
  specification.
- U20 Whether TPM 2.0 is accepted as the D3 backend beyond this TEST profile
  (owner decision open).

### Feasibility probe results (25 November 2026, swtpm 0.10.1)

Feasibility probe, not acceptance evidence:
[b03-rel05-probe-20261125](../verification/b03-rel05-probe-20261125/README.md).
U1 and U7 proven on swtpm; U2 presence of TCG2 proven, extend/ExitBootServices
behaviour deferred to S6. Constraints for S6/S7 derived from the probe:
`nonceCaller` must be 16 bytes (U3: 0 or 15 bytes give `TPM_RC_SIZE`), the
policy-session HMAC must be empty (non-empty gives `TPM_RC_BAD_AUTH`), every
active PCR bank must be extended consistently (U5: four banks active under
OVMF), OVMF sets a random platform auth before handover (U8), and a session
whose authorisation failed stays loaded and must be flushed.

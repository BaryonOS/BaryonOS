# B01-LOCK — keyring LOCK/UNLOCK contract for key erasure before S3

Date: 26 November 2026. Status: **proposed design contract** of workstream B.
Nothing in this document is implemented and no check has been executed for it.
Proposed decision: ADR-KEYRING-LOCK-20261126; its text is in §12 and moves to
[DECISIONS](../DECISIONS.md) only after project-owner approval, because that
file has no precedent for an unapproved entry. Basis: `origin/main`
`fb4a40fa`. Statements about current behaviour come from reading that source.
Lines marked † were re-read for this document; the others come from a
read-only source survey of the same commit and are to be re-read by whoever
implements against them.

Consumers: H05-T5c slice 2 (the Q3 key-erasure SUSPEND gate, H05-T5d) and,
through the unlock ordering, slice 4 (the Q1 fresh-generation block re-grant,
T5f). Related decisions: ADR-043 (Key Service), ADR-051 and
ADR-B02-KEY1-RELAY-20261123 (secret custody), ADR-H05-S3-SYSTEM-20261123
(SUSPEND and the storage gate).

## 1. Problem

The project-owner decision Q3 of 25 November 2026, recorded in amendment
H05-T5c of ADR-H05-S3-SYSTEM-20261123, requires that plaintext volume
content and derived keys are erased before S3 through the B TPM/keyring
architecture. If that cannot be guaranteed, S3 is refused gracefully with a
reason. Session authentication is required before access is restored, and no
claim may exceed what is shown.

RAM stays powered during S3. Whoever can read the memory of a suspended
machine (memory imaging, a cold-boot transfer, DMA from a device attached
during sleep) obtains every secret that is still in RAM. The current SUSPEND
storage gate proves that the volume is shut down, that no block object exists
and that the brokers are idle. It observes no key material, so a successful
SUSPEND today erases nothing.

## 2. Current state (source reading)

### 2.1 Key material and copies that are live at SUSPEND

| Holder | Item | Erased by the current gate |
|---|---|---|
| Key Service (`services/keyring/src/lib.rs`, `Service`) | `master`, the 32-byte HKDF PRK from which every domain key is derived | no |
| Key Service | `Drbg.key` (ChaCha20 fast-key-erasure) and the `random::Pool` state | no |
| Key Service | per-call domain keys from `derive_key` and `Zeroizing` request buffers | wiped per call; register and stack copies are hygiene only (ADR-043 residual risk) |
| Key Service, relay mode | `HeldSecret` from `SUPPLY_SECRET` | not applicable: relay cannot be built with `native-system-power` (†`kernel/src/m2/m3/system.rs:21-37`), so it is never live at a SUSPEND |
| Init, system profile | `RECOVERY_SECRET`, a public compile-time TEST constant in the init ELF (†`sdk/examples/system/src/init.rs:67`) | no; it cannot be erased while init runs |
| Volume, domain 1 | the key-state blob (master and revocation state) that init exports and commits in every normal system run (†`init.rs:382-387`), sealed under `RECOVERY_SECRET` | not applicable (on disk); with the public secret it is an unwrap path |
| Endpoints the Key Service sends to | replies already queued for apps or init, including UNSEAL plaintext, RANDOM bytes and EXPORT blobs; they are not purged when the sender dies and are zeroed only when dequeued (`crates/capability-core/src/lib.rs:1583, 1957-1960`) | no |
| Kernel service stack | the IPC send `snapshot` and receive `bytes` arrays (†`kernel/src/m2/m3/m4.rs:283-305`), the entropy delivery copy `out` (†`kernel/src/m2/m3/keyring.rs:635-637`) and the device read `buffer` (†`keyring.rs:694-715`); kernel handlers run on the permanent service stack (†`kernel/src/m2/arch.rs`, `mov rsp,[rip+m3_service_stack]`) and none of these arrays is wiped | no |
| Kernel broker | `r.bytes`, wiped after delivery (†`keyring.rs:638`) | yes |
| Volume service (`services/volume/src`) | no key fields found | — |
| Apps | any plaintext content an app keeps | no; outside the keyring (OQ1) |

The system app does not use the ADR-043 recovery export in this profile;
where an app does, it holds a recovery secret that is another unwrap path.

### 2.2 Key Service lifecycle

- The kernel spawns the Key Service at boot as slot 1 from
  `KEYRING_SERVICE_ELF` (`kernel/src/m2/m3.rs:748-749, 951-975`).
  `system::manifest` writes its capabilities:
  - Entropy (word 48);
  - two domain RECV endpoints (words 56 and 80) and the app reply SEND
    (word 88);
  - the administrative RECV (word 96) and init's admin reply SEND (word 104).

  Source: `system.rs:592-669`.
- Init cannot launch or restart it: init's Launch capability covers image 0
  only (`system.rs:400-406, 592`).
- No restart path exists. An exit of slot 1 fails the run with
  `UNEXPECTED_EXIT` (†`system.rs:926-929`), and a fault is not a managed
  image. The Key Service is torn down only in `finish`, after every check
  (†`system.rs:1489-1502`). The B01 RESTART case resets the device, not the
  service (`keyring.rs:927-1000`).
- A Key Service that dies with an entropy request pending latches the broker
  offline at -14 until reboot (`keyring.rs:237-255`). `broker_summary().2`
  then stays false, so `storage_closed` returns -8 for the rest of the boot.

### 2.3 Process teardown

`cleanup` frees and zeroes the process's user frames, its four kernel-stack
frames and its page tables (`m3.rs:1733, 1752, 1769, 1777, 1781`;
`paging.rs:821-823`). `Frames::free` zeroes before marking a frame free
(`crates/kernel-core/src/lib.rs:270-293`). An assert then requires that no
frame is still owned by that process or by its page tables
(`m3.rs:1785-1789`). The saved registers are reset; the FPU image is wiped
only with the `fpu-context` feature (`m3.rs:1700-1704`). `Engine::exit`
closes the process's own endpoints and zeroes their queued messages
(`capability-core` `lib.rs:1588-1665, 1914-1925`).

Exception: under `native-smp-m3`, a failed `kernel_round()` leaves the frames
held, unzeroed and quarantined (†`m3.rs:1758-1765`).

### 2.4 SUSPEND order (H05)

The syscall 0x91 handler (`system.rs:247-262`) checks, in order:
1. the reserved arguments (-1);
2. `check_power` (-4 or -6);
3. `sleep::ready()` (-14);
4. `storage_closed()`: `volume::power_gate` (-8 or -16), then
   `block_objects != 0 || !broker_summary().2` (-8);
5. `sleep_quiescent()` (-8).

Only then does it sleep. `broker_summary()` reports broker idleness and says
nothing about whether the Key Service is alive.

### 2.5 Session authentication

There is no native sign-in. The window session never authenticates
(`kernel/src/m2/m3/window_session.rs:3`). `ui/native/src/session.rs` models
login and unlock, but its only broker returns `Unavailable`.

## 3. Threat model and scope

In scope: recovery of key material from the RAM of a suspended machine, on
the fixed QEMU/OVMF profile of ADR-H05-S3-SYSTEM-20261123.

Out of scope, so that no claim covers them:
- A Key Service or kernel compromised before LOCK already holds every key.
- Plaintext content held by apps or by the volume service (OQ1). Q3 names it,
  but it is not key material.
- CPU caches, side channels and physical memory remanence, none of which QEMU
  shows.
- Physical S3 and physical hardware.

## 4. Two facts, not one

"Keys erased" needs two separate facts:

- **F1 — instance erased.** The kernel can verify this fact itself. It holds
  when all of the following hold:
  - No Key Service instance exists.
  - Every frame the last instance owned was returned through the zeroing free
    path, and the owner assert held.
  - Its endpoints are closed and their queues zeroed, and no message it sent
    is still queued anywhere.
  - Every kernel buffer that carried its payloads or its entropy has been
    wiped.
  - The entropy broker is idle, with no pending or completed request.
- **F2 — no unwrap path.** No holder outside the dead instance keeps material
  that unwraps the durable key state, in RAM or as a public value: the
  external secret, a recovery secret, or an exported blob together with its
  secret. This is a custody property. It follows from the secret-custody
  design, and the kernel cannot establish it by inspecting memory.

F2 is false in every current profile: init's ELF carries the public
`RECOVERY_SECRET`, and the volume holds the blob sealed under it. A TEST-profile
S3 can therefore show the erasure mechanism (F1) but claims no
confidentiality.

**OWNER question O1.** May SUSPEND pass on F1 alone in TEST profiles? The
markers and evidence would carry the label "instance erased; unwrap secret is
a public TEST constant". The alternative is that SUSPEND refuses until F2
also holds, which means no S3 until a product secret source exists.
Recommendation: allow F1 alone only in TEST profiles, with that label, and
require F1 and F2 in every non-TEST profile. Q3's "refuse if not guaranteed"
makes this the project owner's decision.

## 5. Security invariants

- **KL1 Kernel-owned state.** The kernel keeps a keyring lock state, LIVE or
  LOCKED. Only the kernel's own LOCK path (§6) sets LOCKED, and only after
  every F1 condition holds. No ring-3 message, flag, exit status or fault sets
  it. An exit or fault of the Key Service outside LOCK stays a failure
  (`UNEXPECTED_EXIT`) and never counts as LOCKED.
- **KL2 SUSPEND precondition.** While the state is not LOCKED, SUSPEND returns
  a distinct, counted refusal code. The check runs after `sleep::ready()` and
  before `storage_closed()`, the quiescence check and any I/O. H allocates
  the code, which is none of -8, -14 or -16. In a non-TEST profile, F2 must
  also hold (O1).
- **KL3 Erasure is teardown.** LOCK ends the instance through the kernel's
  process teardown. The instance cannot defer or veto it. LOCK is
  irreversible: nothing re-attaches to the old instance.
- **KL4 No residue.** LOCK either wipes the kernel buffers that carry
  Key Service payloads or entropy (§2.1), or scrubs the unused region of the
  service stack. It also refuses while a message from the instance is still
  queued.
- **KL5 Fail closed for clients.** After LOCK, every KEY1 and RANDOM request
  fails without partial output, and no domain caller receives `Ok` for an
  operation that is not durable.
- **KL6 Unlock is a new instance.** Only durable sealed key state survives a
  LOCK.
- **KL7 Authenticated unlock.** A new instance serves keys only after fresh
  startup entropy passes the health tests and the durable state is restored
  with a secret obtained through session authentication.
- **KL8 Ordering after a wake.** Unlock precedes any block re-grant to a
  service that handles key-protected data (T5f). The order relative to the Q2
  remount is open (O4).

## 6. LOCK (required by H05-T5d)

Proposed shape; the operation number and the split of the work are agreed
with H (O3).

- **Authority.** Init only, authorised by the non-delegable POWER object, like
  SUSPEND. A wrong handle type gives -6 and a forged generation gives -4,
  before any state change.
- **Preconditions.** Each is checked without side effects, and a failure
  returns a retry code:
  - the entropy broker is idle, with no pending or completed request (this
    avoids the permanent -14 latch of §2.2);
  - no endpoint to which the instance holds SEND has a queued message;
  - the state is LIVE. A second LOCK returns a defined code and changes
    nothing.
- **Effect.**
  1. The kernel marks slot 1 Dying and runs `cleanup`, the path `finish`
     uses today.
  2. It requires the owner assert, applies KL4, and then sets LOCKED,
     increments a counter and writes one marker.
  3. Under `native-smp-m3`, a quarantined teardown (§2.3) leaves the state
     not LOCKED, so SUSPEND keeps refusing.
- **Durability.** Without the write-ahead relay, which the S3 profiles cannot
  build, key state is durable only through init's export and commit. A key
  allocation or revocation between that export and LOCK is lost. A lost
  revocation would make a revoked key usable again after UNLOCK, which
  falsifies ADR-043. Init must therefore export and commit immediately before
  LOCK, and the window between the two is open question O5.
- **LOCKED-AWAKE.** If SUSPEND refuses after a successful LOCK (-8 or -16),
  or the system does not sleep, the system runs without a Key Service:
  - every KEY1 and RANDOM request fails;
  - S5 stays reachable;
  - UNLOCK is the only way back to key service.

  This state needs its own acceptance case (A7).

### 6.1 Collisions with existing H05 code (questions for H)

- **C1.** The T5b rng re-init delivers its post-wake self-test bytes into the
  Key Service heap (†`keyring.rs:1519-1526, 1694-1701`), through the
  Key Service's entropy capability copied into kernel state
  (`system.rs:156-159, 624-627`). After LOCK, neither the slot nor that
  capability exists, so the re-init needs a kernel-owned target. Slice 1's
  blk re-init runs after it.
- **C2.** `finish` tears down slot 1 and requires `entropy_objects == 0`
  (†`system.rs:1489-1502`). It must accept a slot 1 that LOCK already ended.
- **C3.** `UNEXPECTED_EXIT` (†`system.rs:926-929`) must keep refusing every
  exit outside LOCK.
- **C4.** The SUSPEND check (KL2) and init's suspend sequence belong to H.
  Proposal: B implements the LOCK path (teardown, wipes, state, counters) and
  its host and native tests; H implements the SUSPEND check and the init
  sequence on top of it.

## 7. UNLOCK (required before T5f): requirements only

No restart path, native session authentication or endpoint re-grant
mechanism exists today (§2.2, §2.5). This contract therefore fixes
requirements and leaves the mechanism to a later decision:

- U1: UNLOCK creates a fresh instance; it never revives the old one (KL6).
- U2: the instance reaches READY only after 1024 bytes of fresh startup
  entropy pass the health tests, which needs the post-wake rng re-init.
  Otherwise it stays BLOCKED and fails closed.
- U3: it restores the durable state only with a secret obtained through
  session authentication (KL7). A wrong or absent secret leaves it not READY:
  no domain traffic and no re-grant.
- U4: handles to the old instance fail. Clients receive fresh-generation
  handles only after U2 and U3, in the manner of Q1.
- U5: UNLOCK precedes T5f (KL8).

## 8. Proposed acceptance cases (native QEMU, labelled)

| Case | Expected |
|---|---|
| A1 SUSPEND while LIVE | KL2 code; no I/O; counted |
| A2 LOCK with an entropy request pending | retry refusal; no teardown; no -14 latch |
| A3 LOCK with a Key Service reply still queued | retry refusal; no teardown |
| A4 LOCK success | slot 1 dead; owner assert holds; a test-only scan finds every frame the instance owned zeroed; `entropy_objects` reclaimed; one marker |
| A5 Key Service exits on its own before LOCK | `UNEXPECTED_EXIT`; never LOCKED (forgery control) |
| A6 app KEY1 call after LOCK | error; no partial output |
| A7 LOCK, then SUSPEND refused by -8 | LOCKED-AWAKE; S5 still passes |
| A8 LOCK, SUSPEND 0, wake | no keyring instance; the re-init path resolved per C1 |
| A9 LOCK with an image handle or a forged handle | -6 or -4 before any state change |
| A10 residue | a labelled canary sent through the Key Service before LOCK is absent from the wiped buffers or the scrubbed stack region |

## 9. What this contract does not prove

It does not prove:
- confidentiality in TEST profiles (F2 is false there);
- plaintext content erasure (OQ1);
- CPU cache or register remanence;
- physical S3, firmware or hardware behaviour;
- any UNLOCK mechanism;
- a human audit.

A review by a second AI is not an independent audit.

## 10. Open questions

| ID | Owner | Question |
|---|---|---|
| O1 | project owner | F1 alone in TEST profiles with a label, or no S3 until F2 holds (§4) |
| O2 | project owner | product unwrap-secret source for UNLOCK: a sign-in-derived secret, a TPM-sealed factor with user authorisation (B03/B04), or both; which component holds it and for how long |
| O3 | H | LOCK operation number, the KL2 code, the split in C4, and the answers to C1–C3 |
| O4 | S | whether the Q2 remount needs keys, which fixes the order of remount and UNLOCK |
| O5 | project owner and B | the durability window without write-ahead: a Key Service freeze opcode before export, or write-ahead relay in S3 profiles |
| OQ1 | project owner, H and U | erasure of plaintext content held by apps and the volume service before S3 |

## 11. Work split proposal

- **B:** the LOCK path in the kernel keyring broker and teardown (KL1, KL3,
  KL4), host tests, acceptance cases A2–A6 and A9–A10, and the UNLOCK design
  once O2 and O4 are answered.
- **H:** KL2 in SUSPEND, init's suspend sequence, C1–C3, and acceptance cases
  A1, A7 and A8.

The two slices touch `kernel/src/m2/m3/system.rs` and are therefore
sequential, never parallel writers.

## 12. Proposed ADR text (to be inserted after approval)

> **ADR-KEYRING-LOCK-20261126 — Key erasure before S3 by kernel teardown of
> the Key Service instance**
>
> **Status:** proposed by workstream B on 26 November 2026; not approved.
>
> **Decision.**
> 1. The kernel keeps a keyring lock state that only its own LOCK path sets
>    (KL1).
> 2. LOCK is init-only under the POWER object. It ends the Key Service
>    instance through process teardown after side-effect-free preconditions,
>    and it wipes or scrubs the kernel copies of Key Service payloads (KL3,
>    KL4).
> 3. SUSPEND refuses with a distinct code while the state is not LOCKED
>    (KL2). In non-TEST profiles it also requires that no unwrap path remains
>    (F2).
> 4. UNLOCK is a fresh instance that serves keys only after fresh entropy and
>    a restore with a session-authenticated secret (KL6, KL7). Its mechanism
>    is a separate decision.
>
> **Consequences.**
> - TEST profiles show the mechanism, not confidentiality, because the unwrap
>   secret is a public constant.
> - The system runs without key service between LOCK and UNLOCK.
> - Key state that is not durable at LOCK is lost (O5).
>
> **Specification:** docs/implementation/B01-KEYRING-LOCK.md.

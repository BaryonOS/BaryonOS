# S09 — Backup destination service, trusted portal and per-operation rights (design specification)

**Status: design specification. Route steps R2 and R3 (section 8.2) have
host-only implementations on draft PR #413, which is not merged; nothing in
this document is native, active or a production security boundary.** It specifies
D1 (b), D2 (a), D3, D4 (a) and D5 (a) of
[ADR-S09-DESTINATION-20261119](../DECISIONS.md#adr-s09-destination-20261119--backup-destination-service-trusted-dialog-and-os-backup-portal).
The ADR's implementation boundary applies unchanged:

> A separate destination service and any new production capability boundary
> require a documented interface, a security review, negative tests and
> verified enforcement before activation.

This document is the documented interface. The negative tests of section 6 and
verified enforcement remain open. The existing
test-fixture grant in [S09-BACKUP-ARCHIVE.md](S09-BACKUP-ARCHIVE.md) ("Native
slice") remains a fixture and is not changed by this text. S09 stays open.

**Revision 2 (23 November 2026).** Revision 1 answered the independent AI
review [SPEC-REVIEW-1.md](../verification/s09-destination-service-20261123/SPEC-REVIEW-1.md)
(REQUEST_CHANGES, 13 required items). Revision 2 answers the independent AI
re-review [SPEC-REVIEW-2.md](../verification/s09-destination-service-20261123/SPEC-REVIEW-2.md)
(REQUEST_CHANGES, required findings F1–F6, optional F7–F15). Section 9 records
the resolution of each item of both reviews. Rules that need a project-owner
decision are marked *[proposed; owner decision Oxx]* and are not decided
(O4 (b), O4 (c), O11, O12, O13). The independent AI re-review of revision 2,
[SPEC-REVIEW-3.md](../verification/s09-destination-service-20261123/SPEC-REVIEW-3.md),
returned APPROVE with five optional findings, all applied (section 9). An AI
review is not a human audit; route step R1 (section 8.2) is complete as an AI
design review only, and the owner decisions above stay open.

**Revision 3 (23 November 2026).** Revision 3 answers the 29 items (L1–L29;
three of them by a proposed rule that awaits an owner decision) that the host
implementations of R2 (`services/destination`) and R3
(`tools/destination_disk.py`) on draft PR #413 and their independent AI reviews
found in revision 2: ambiguities, internal inconsistencies, unstated rules and
verification gaps. It adds one device-result mapping (2.5), completes the 2.6
copy-validity list, defines the sealing sequence, states the broker contract
required of K5 (2.7), adds negative tests N33–N36 and three owner questions,
O14–O16 (8.1), and records each item in section 9. Where the PR #413 code
differs from the text chosen here, the text was not adapted to the code; the
difference is a change required in the code. The APPROVE of review 3 covers
revision 2 only. The independent AI review of revision 3, SPEC-REVIEW-4
(REQUEST_CHANGES, required findings RQ-1–RQ-3, optional OP-1–OP-10), has been
answered in this text (section 9). The re-review SPEC-REVIEW-5
(REQUEST_CHANGES, required RQ5-1 and RQ5-2, optional OPT-1–OPT-8) found every
review-4 finding resolved; its required findings are answered in this text by
the lead and have not been re-reviewed, and nothing in revision 3 is approved. Rules marked
*[proposed; owner decision Oxx]* with O4 (b), O4 (c), O11, O12 and O13 are not
decided, and the other items of 8.1 remain open as stated there. The project
owner accepted the proposals of O14, O15 and O16 on 23 November 2026; those
rules are marked *[owner decision Oxx, 23 November 2026]*.

**Revision 4 (23 November 2026).** Revision 4 answers the 13 items (P1–P13)
that the code changes of R2 and R3 for revision 3 and for the owner decisions
O14 and O16 (draft PR #413, commits `4f1edb9c` and `e52cb865`) and their
independent AI code reviews found in revision 3. It states that a sealed entry
can today hold a different valid archive after a device fault, and proposes a
SEAL content digest for that case (owner question O17); it proposes that a
remount resolve *sealed* only on the exact catalog copy encoded for the SEAL
(owner question O18, stricter than the decided O16 wording, which is
unchanged); it names the control-character set of `valid_name`; it removes the
conditional wording that the decisions of 23 November 2026 settled; it
corrects the reachability statements and kill claims of N26, N29, N33, N34 and
N35 and lists their equivalent mutants; it adds negative test N37
(conditional on O17 when revision 4 was written; O17 and O18 have since been
decided, see revision 5); and it adds verification-plan items to R2 and R3.
Section 9 records each item ("Revision 4 record") and the changes that the
text requires in the PR #413 code ("CODE-DELTA"). O14, O15 and O16 are decided
and their decided rules are unchanged. Review 6 (SPEC-REVIEW-6) approved
revision 4 with no required finding, confirmed the review-5 answers RQ5-1 and
RQ5-2 (P13), and made nine optional findings, which the lead applied (section
9, "Review 6"; checked by one read-only verifier, whose two wording
corrections were applied and not re-reviewed). An AI review is not a
human audit.

**Revision 5 (23 November 2026).** Revision 5 answers the 13 items (Q1–Q13)
from the code round on draft PR #413 that implemented the owner decisions
O17 (a) and O18 (commit `f993a1a3`, host only, unmerged), from the
independent AI code review of that round
(`docs/verification/s09-destination-core-20261123/CODE-REVIEW-O17-O18.md` on
PR #413: APPROVE, optional findings OPT-1–OPT-5) and from the owner-decision
commit `a769bc65`, which relabelled the O17 and O18 markers; reviews 7 and 8
covered it and returned REQUEST_CHANGES on other findings, and review 9 is the
first review to approve it. The project owner decided O17 and O18 on 23
November 2026 ("Yes, accept both"): O17 as recommendation (c), the service
content check (a) and the portal trailer check (b), where a digest mismatch
replies `Corrupt` -18 and does not start the safe path, pending the O9
evidence; O18 as proposed,
so that a remount resolves *sealed* only on equality with the copy encoded
and checked in SEAL step 4. Revision 5 removes the conditional O17 and O18
wording outside the section 9 history, states the O17 latching
sub-question as decided ("no latch, pending O9") with the evidence that
would reopen it, states which poll results the broker contract defines
after a delivered result, keeps both forms of the content check with the
precondition of the trailer form, aligns N35 and N37 with the tests and
surviving mutants of `f993a1a3`, and adds an open R2 item for the parts of
the broker contract that the simulated disk does not model yet. Section 9
records each item ("Revision 5 record") and the status of every code change
("CODE-DELTA (revision 5)"). O14–O18 are decided and their decided rules are
unchanged. Review status: review 7 (SPEC-REVIEW-7, an independent AI
review) returned REQUEST_CHANGES with three required and six optional
findings; the lead applied all nine (section 9, "Review 7"). Review 8
(SPEC-REVIEW-8, an independent AI review) returned REQUEST_CHANGES on that
result with four required and six optional findings; the lead applied the
nine findings that change this text, and RQ8-2 (storing SPEC-REVIEW-7
verbatim) is an action of the revision-5 commit (section 9, "Review 8").
Review 9 (SPEC-REVIEW-9, an independent AI review of revision 5 after the
review-7 and review-8 fixes, and of `a769bc65`) returned APPROVE with no
required finding and nine optional findings (OPT9-1–OPT9-9). Reviews 7 and 8
covered `a769bc65` and the review-6 corrections that revision 4 applied
without re-review (Q2, Q13) and returned REQUEST_CHANGES on other findings;
review 9 is the first review to approve them. The lead dispositioned the
optional findings of review 9 in a separate fix pass (section 9, "Review
9"). Review 10 (SPEC-REVIEW-10, an independent AI review of that pass)
returned REQUEST_CHANGES with two required and eight optional findings; the
lead applied all ten in a further fix pass (section 9, "Review 10"). Review
11 (SPEC-REVIEW-11) approved that pass with no required finding and nine
optional findings, which are deferred to the next revision (section 9,
"Review 11"). The status record of review 11 itself was added by the lead
after it and has not been reviewed. An AI review is not a human audit.

**Status note (24 November 2026, task S-20261124-5).** The R2 core
(`services/destination`) and the R3 oracle (`tools/destination_disk.py`) are
on `main` since the integration PR #477 (`ab9414c`); the labels *[PR #413]*
below record where each claim was first made. The R5 precondition "the service
links `volume_core::archive::verify`" (2.3, "Archive verifier") now has a host
implementation: the `volume-verifier` feature of `destination-core` provides
`archive_verifier::VolumeVerifier`, which runs `archive::verify` and returns a
source error unchanged. `services/destination/tests/real_verifier.rs` runs
N7 and N10 through the service with that adapter on real v1 archives written
by `volume_core::archive::export`, and checks acceptance, the summary,
refusal of flipped, truncated and extended archives and of the stand-in
layout, and source-error passthrough for every status. The stand-in rows
remain for the other tests. This is host evidence only: no native service
links the adapter yet, and the other R5 preconditions (O4 (a)–(c), the
hosted R3 cross-check and reverse differential, the K5 broker) are unchanged.

**Owner decisions of 24 November 2026 (task S-20261124-6).** The project
owner decided O4 (a)–(c), O11, O12 and O13 on 24 November 2026. The decided
rules are recorded at each former *[proposed]* marker and in 8.1; the owner's
words are summarised there in English. A design approval is not test
evidence and does not close S09. In short:
- **O4 (a)** K4 and K5 are approved for targeted implementation. K5 separates
  all relevant broker state and buffers per device, including the recorded
  handling of a further completion poll after a read ended by a reset
  (property 3 of 2.7). K5 is an R5 precondition, K4 a product-profile
  precondition; K1–K3 do not block R5. Attaching a backup device while the
  system runs remains part of the later product design.
- **O4 (b)** ADR-S09-DUALDISK-20261119 B3 and B5 are amended for R5 only: the
  destination service may hold the second virtual disk exclusively with READ,
  WRITE and FLUSH, and a scripted test dialog is permitted. B4 and B6 are
  unchanged: isolated disposable images only, no product activation and no
  access to real host disks.
- **O4 (c)** The deterministic destination ID (section 7, option (b)) is
  permitted only in the explicit test profile, labelled `DEST:ID:TEST-ONLY`;
  product builds must not use it. The real entropy route must have native
  evidence before R8.
- **O11** A v1 destination is a separate storage device. A partition on the
  source device does not count as an independent recovery copy.
- **O12** The product binds an owner per backup registration. Backups must
  not be disabled in general once a second profile exists. The catalog
  version, trusted identity, access checks and recovery route are to be
  designed and checked against the profile, key and encryption contracts. R5
  and R6 may use one profile for now. An owner field is not evidence of
  encryption.
- **O13** A kernel-enforced write latch on the destination scope after the
  specified trigger is required. A restart, rebind or newly granted
  capability must not bypass it. It is implemented and tested before R8; the
  once-per-boot grant rule and N32 are kept as well.

The same task adds the R3 reverse differential on the host (8.2 R3, status of
24 November 2026); its hosted run remains open with the forward cross-check.

**Owner decision package 2 (24 November 2026).** The project owner then
decided O1–O3 and O5–O10, added detail to O4 (a), (b), O12 and O13, and
confirmed O14–O18, D1–D6, the restore-fidelity decisions, the independent
kernel and the Security Floor unchanged. Each decision is recorded in 8.1
under "Decided on 24 November 2026 (package 2)" or "Package 2 additions".
Ownership for the work that follows: S holds the storage contract, the
destination service, backup, restore and the integration of the S09 chain; B
holds user keys, metadata, encryption, authenticity and its security review;
the owner of the user and session facilities holds trusted profile identity,
session limits and recovery of owner rights; UI/UX holds the shared system
screens; the integration coordinator holds shared kernel interfaces,
protected policy files, ADR integration and workflow changes.

The ADR text that records these decisions in `docs/DECISIONS.md`, which is
lead-owned, is proposed to the integration coordinator in the pull request
that carries this note; until it lands, this note is the S-owned record.

**Evidence labels used below.** *[main]* means present on `main` at `035b44e5`.
Most such claims were checked by review 1 or by the verification of it for this
revision; the `sdk/task-grant-core` and `services/net-broker` claims of 3.3 and
the virtio-rng slot of `tools/run_keyring.py` were spot-checked only by
identifier, and the volume-service claims of 4.2 and the entropy flow after
PR #405 were not rechecked for this revision. PR #396
(the test-only second virtio-blk disk at PCI `00:06.0`, the portal-only
`BlockScope { device: 1, .. }` grant, the 22-stage volume gate, the `SimDisk`
host tests, [S09-DUAL-DISK-FIXTURE-REQUEST.md](S09-DUAL-DISK-FIXTURE-REQUEST.md)
and `docs/verification/s09-archive/dual-disk-native-20261119/`) is merged as
`8a7a3594`, an ancestor of `035b44e5`, and its content is labelled *[main]*.
Kernel PCI discovery at a slot other than `00:06.0` remains unchecked (section
7). *[PR #413]* (new in revision 3) means present on draft PR #413 at
`37b7bd0c`, or, for a claim made in revision 4, at `e52cb865`, or, for a
claim made in revision 5, at `f993a1a3` (host code and
tests only, not on `main`); `services/destination`
and `tools/destination_disk.py` are never *[main]*. *[design]* means specified
here and not implemented natively. *[proposed]* means a
design rule that awaits a project-owner decision. *[unproven]* means an
assumption without a test.

## 1. Scope and non-goals

In scope: the destination service interface, the trusted backup portal and its
destination-selection flow, per-operation rights, source/destination isolation,
a negative test matrix, a disposable-QEMU verification plan and a staged route.

Not in scope and not changed:
- **Archive format v1** ([S09-BACKUP-ARCHIVE.md](S09-BACKUP-ARCHIVE.md), "Bytes")
  is the entry payload, byte for byte. No field is added.
- **Restore semantics** stay as specified: restore never overwrites, assigns new
  revisions and times, and requires `verify` on the same bytes first. Faithful
  native recovery of revision history and timestamps is a separate task that
  must first identify the required format and contract changes; this document
  neither assumes nor claims it.
- **ustar/pax export (D6 (a))** remains the lossy interchange export. It is not
  a destination entry type and nothing restores from it.
- **Encrypted NAS and provider destinations (D5).** Per the ADR they require
  the existing key-management and client-side encryption contracts
  ([ENCRYPTED-OBJECT-STORAGE.md](ENCRYPTED-OBJECT-STORAGE.md)). This design
  covers only local, user-owned block devices; the service refuses any other
  destination kind by construction, because it only accepts a block
  capability. A partition on the source device is not a destination: section
  2.1 requires a separate device *[decided: O11, 24 November 2026]*.

## 2. Destination service interface (D1 (b))

### 2.1 Ownership

The destination service is a userspace system service *[design]*. At boot the
manifest gives it exactly one block capability on the whole destination device
(O11): `create_block_device_with_rights(service, scope, BLOCK_READ |
BLOCK_WRITE | BLOCK_FLUSH)`. The legacy `create_block_device` constructor must
not be used, because its default `BLOCK_READ_RIGHTS` includes DERIVE, TRANSFER
and REVOKE *[main]*; with them the service could pass raw device authority on.
No other process holds any capability on the destination device. The service
never writes outside the region rules of section 2.6, and it never holds a
capability on the source volume's block object (section 5).

**Manifest-only, non-transferable grant.** The grant is created only by the
manifest (K1) and carries no TRANSFER. On `main`, spawn (syscall 0x50) is
type-blind: it delegates any handle whose node holds TRANSFER, within a rights
subset (`check_delegation`, `bootstrap_delegate`) *[main]*. Every block grant in
a profile that has spawn is created without TRANSFER *[main]*, and that keeps a
process from passing block authority to a spawned or restarted process; the
kernel does not enforce it by object type. This document therefore requires the
destination grant to carry exactly READ|WRITE|FLUSH, and N18 probes those exact
rights, independent of the owner decisions below.

The absence of TRANSFER does not stop kernel profile code from creating a new
grant. On `main`, the M5 profile's `bind_service` creates a new block object
with `create_block_device` on every kernel-driven restart of its service
(`kernel/src/m2/m3/m5.rs:260-279`, called from `poll` at `:784-794`, at most
three restarts) *[main]*. **Once-per-boot grant rule** *[decided: kept under
O13, 24 November 2026, in addition to the kernel write latch]*: the destination profile creates the destination block object
exactly once per boot, at profile start, and after profile start no kernel
path (restart, rebind or poll) creates a block object whose scope intersects
the destination scope or delegates a handle on one. The rule is stated by
scope, not by object key, because a re-created object has a new key over the
same sectors. It is a profile code rule: the kernel does not enforce it, and
N32 is its test. Consequence, under this rule only: no instance in the same
boot can regain destination block authority after the service exits, and
recovery from the failed-mount state (2.6) and from the read-only safe path
(2.7) is a reboot. No in-boot restart route is specified (section 8, O8). The
owner decided (O13) that this profile rule alone is not the carrier of the
safe path: the kernel write latch of 2.7 is required in addition, so that a
grant re-created despite the rule still cannot write the destination scope.

**Device rule** *[decided: O11, 24 November 2026]*. In v1 the destination scope is
on a block device of its own: `scope.device` differs from that of every other
block scope in the profile. `archive_grant` compares scopes with `check_block`
for the fixture (`kernel/src/m2/m3/volume.rs:749-753`) *[main]*; this is a
precedent for the comparison, not an existing different-device rule (those
lines check that the volume service's own block handle is a different object on
device 0). A partition on the source device is not a v1 destination: it
shares the device's reset and FLUSH with the source, so a source fault can
stop the destination and the reverse, and it is not an independent copy
against loss of that device. Every stage uses a separate device (disk 1), and
N18 asserts that the destination scope is disjoint from every other block
scope and on another device.

### 2.2 Message framing

Requests and replies use the volume service's framing *[main,
`services/volume/src/wire.rs`]* with a distinct version and operation set: a
224-byte body inside `Message320`, a 64-byte header and at most 160 data bytes.

| Offset | Field | Request rule | Reply rule |
|---|---|---|---|
| 0 | version u16 | = 1 (destination wire v1) | = 1 |
| 2 | operation u16 | 1..=11 (table 2.3), else `NoSys` | echoed |
| 4 | 4 × 0 | must be 0, else `Invalid` | 0 |
| 8 | request_id u64 | ≠ 0, strictly increasing per lane binding (control lane: per service start), else `Stale` | echoed |
| 16 | epoch u64 | = grant generation of the lane binding (control lane: 0), else `Stale` | echoed |
| 24 | entry u64 | entry ID (0 only where the table allows) | entry ID |
| 32 | offset u64 | byte offset (APPEND, READ) or total length (SEAL), else 0 | next offset or total length |
| 40 | length u32 | ≤ 160 and = data length, else `Invalid` | reply data length ≤ 160 |
| 44 | status i32 | must be 0 | service status (2.4) |
| 48 | name_length u32 | CREATE: label length 1..=32 and = `length`, else `Invalid`; BIND: operation 1..=3; else 0 | per operation |
| 52 | count u32 | LIST: cursor < 16; BIND, UNBIND: lane 1..=3; else 0 | per operation |
| 56 | aux u64 | READ: byte count 1..=160; BIND: grant generation; else 0 | per operation |
| 64 | data[length] | bytes after `length` must be 0, else `Invalid` | zero-padded |

**Field rules per operation (check 2).** "any" means not checked at check 2;
"≠ 0" means `Invalid` -1 if 0.

| Op | entry | offset | length | name_length | count | aux |
|---|---|---|---|---|---|---|
| INFO, FORMAT | 0 | 0 | 0 | 0 | 0 | 0 |
| LIST | 0 | 0 | 0 | 0 | < 16 | 0 |
| CREATE | 0 | 0 | 1..=32 | = `length` | 0 | 0 |
| APPEND | ≠ 0 | any | 1..=160 | 0 | 0 | 0 |
| SEAL | ≠ 0 | any | 0 | 0 | 0 | 0 |
| ABORT, VERIFY | ≠ 0 | 0 | 0 | 0 | 0 | 0 |
| READ | ≠ 0 | any | 0 | 0 | 0 | 1..=160 |
| BIND | 0 if `name_length` = 1, else ≠ 0 | 0 | 16 | 1..=3 | 1..=3 | any |
| UNBIND | 0 | 0 | 0 | 0 | 1..=3 | 0 |

**Check order.** Every request passes the following checks in this order; the
first that fails gives the reply status, and no later check, no device I/O and
no state change follows, except the high-water rule below:

1. **Transferred capabilities.** If the kernel message header's `cap_count` is
   not 0, the service closes (syscall 0x10) every handle that `recv` attached
   and replies `Invalid` -1. It never keeps a received handle, so it never
   holds a volume endpoint or lane root that boot-time counting cannot see.
2. **Header validity** (table above, as `validate_request` does for the
   volume). Check 2 validates exactly these fields, none of which depends on
   service state: version; operation (`NoSys`); bytes 4..8 zero; `status` 0;
   `length` ≤ 160 and data bytes after `length` zero; and `entry`, `offset`,
   `length`, `name_length`, `count` and `aux` per the field-rule table above
   (`Invalid` -1 for every failure other than the operation). Within check 2 the
   order is version, then operation, then the remaining fields. `epoch` and
   `request_id` are check 5. Whether `entry` is the binding's entry is check
   6. Values that depend on the catalog or the entry (whether a control-lane
   ABORT names the writing entry, whether a BIND entry is sealed, whether an
   APPEND `offset` equals the appended total, whether a READ range lies inside
   the entry) are checked by the operation after check 7.
3. **Lane bound** (operation lanes only): an unbound operation lane returns
   `Access` -5.
4. **Sender** (operation lanes only): a kernel-stamped `sender_pid` other than
   the bound portal (4.3) returns `Access` -5.
5. **Epoch and request ID**: a wrong `epoch` or a `request_id` that is 0 or not
   above the lane's high-water mark returns `Stale` -10.
6. **Operation permission**: an operation that the lane or its binding does
   not permit (2.3, "Permitted on"; 4.1) returns `Access` -5. All binding-level
   limits are part of this check. On a verify or restore lane, VERIFY and READ
   must name the bound entry; on an export lane, APPEND, SEAL and ABORT must
   name the binding's writing entry; otherwise `Access` -5. An export binding
   permits CREATE only if it has issued no CREATE; APPEND, SEAL and ABORT only while its own entry is
   `writing`; and, once its entry is no longer `writing` (sealed, aborted,
   discarded, or SEAL answered `Unknown`, whatever the later resolution of
   2.7), only INFO and LIST. An export binding whose outcome is *unknown* is
   therefore limited to INFO and LIST by this check, not by check 7.
   *[owner decision O15, 23 November 2026]* Only a CREATE answered `Ok` counts as
   issued: a CREATE refused by check 7 or by the operation (`Invalid` label,
   `Busy`, `Full`, or `Access` in the safe path) allocates nothing and leaves
   the binding's CREATE unused. The binding authorizes one new entry; a refused
   CREATE creates none, so a retry on the same binding cannot create a second
   entry, and a transient refusal (`Busy`) needs no new BIND. The owner
   declined the alternative, under which every CREATE that passes checks 1–6
   would use the binding's CREATE.
7. **State gates.** The mount state of the instance (2.6, 2.7) decides the
   reply to a request that passed checks 1–6, by the table below. The
   operation itself runs only where the table says so.

| Mount state | Entered by | INFO | Other requests that passed checks 1–6 |
|---|---|---|---|
| mounted | mount or remount with a valid catalog; FORMAT `Ok` | answered, status `Ok` | the operation runs |
| mounted, safe path | the 2.7 trigger in a mounted instance | answered, status `Ok` | CREATE, APPEND, SEAL and FORMAT `Access` -5; all others run |
| remount pending | a device failure in SEAL steps 4–5 (2.7) | not received | no request is received; requests queued meanwhile are answered afterwards under the resulting state |
| failed mount | failed mount or remount FLUSH, or a failed mount or remount read of sectors 0–15 (not the FORMAT scan, which leaves a blank candidate) (2.6); a FORMAT write or flush failure (2.6); *[owner decision O16, 23 November 2026]* a remount whose reads would give a blank candidate, or a valid catalog other than the committed copy or that copy plus exactly the provisional record (2.7) | answered, status `NoDevice` -14 if the broker returned -14, else `Io` -12 | LIST on an export lane whose outcome is *unknown*: `Unknown` -16 with zero records; all others `Io` -12 |
| `Corrupt` | the mount rule (2.6) | answered, status `Corrupt` -18 | `Corrupt` -18 |
| blank candidate | the mount rule (2.6) | answered, status `NotFound` -19 | FORMAT and UNBIND run (FORMAT `Access` -5 if the safe-path latch is set; unreachable since O16, see the defensive rule below); all others `NotFound` -19 |

In every state other than "mounted" and "mounted, safe path", INFO's reply
has `aux` 0, `count` 0 and 48 data bytes with destination ID and formatted
sectors zero and the grant-generation maxima as in 2.3. The safe path and the
blank candidate state never coexist: the safe path is triggered only in a
mounted instance, a FORMAT failure leads to the failed-mount state, and
*[owner decision O16, 23 November 2026]* a remount never ends as a blank candidate
(2.7). The service receives no request before its boot mount has
ended in one of these states.

**Defensive rule, unreachable since O16** (revision 4). The safe-path rule of
2.7 forbids every catalog and entry write for the rest of the boot, in every
mount state. Should an instance ever be a blank candidate with the safe-path
latch set, FORMAT therefore returns `Access` -5 and writes nothing, instead of
running as the blank-candidate row says. Under the decided O16 rule this
combination cannot arise, as stated above, so no test can reach the rule and a
mutant that removes it is equivalent (N35). It is kept as defence in depth: it
costs one condition, and a later change to the remount rule would otherwise
reopen a write path in a boot in which the safe path forbids writes.

A request refused by checks 1–6 leaves the lane's `request_id` high-water mark
unchanged; a request that passes checks 1–6 advances it, including one refused
by check 7 or by the operation itself. The control lane has
no binding: checks 3 and 4 do not apply, its `epoch` is always 0 and its
`request_id` sequence starts afresh when the service starts. The kernel's
`receive` attaches transferred capabilities before the service sees the
message and fails before dequeuing when the receiver has no free handle slot
*[main, `capability-core` `receive`]*; check 1 does not prevent that stall.
Only the trusted dialog holds TRANSFER on a lane (4.3), and no portal handle
carries TRANSFER (4.1), so the stall is a residual risk of a dialog fault.

**Replies.** Replies to control-lane requests go to the dialog's control-reply
endpoint and replies to operation-lane requests go to the portal's reply
endpoint (4.3). An operation-lane request whose kernel-stamped `sender_pid` is
not the portal's process ID (fixed for the boot by the manifest, 4.3) gets no
reply at all, whatever check refused it: the portal never receives a reply
with a `request_id` it did not send, and refused requests from another sender
cannot fill its reply queue. The service still closes any transferred handle
(check 1) and counts the refusal in `DEST:DENIALS=` (section 7); a request
from the portal on an unbound lane is still answered `Access` -5. The service
never blocks on a reply send: if `send` returns the
kernel's queue-full result (`Again`), the reply is dropped and the request's
effect stands. The requester checks the echoed `operation`, `request_id` and
`epoch` of every reply and treats a mismatch as `Protocol`, ending the
operation.

### 2.3 Operations

Entry IDs are u64, assigned by the service from a monotonic counter stored in
the catalog (2.6); 0 is invalid. Only a *sealed* entry's ID is an entry
identity, and a sealed entry's ID is never reused. The ID that CREATE returns
is provisional: the catalog's next entry ID advances only in the SEAL commit
(2.7, step 4), so the ID of an entry that is aborted, discarded or lost on
reboot is issued again by a later CREATE. A provisional ID has no meaning
outside its export binding and none after a reboot; BIND, VERIFY, READ and
LIST accept only sealed IDs. Labels follow
`valid_name` as defined by `catalog_core::wire::valid_name` *[main]*, with a
limit of 32 bytes: valid UTF-8 of 1–32 bytes, not `.` or `..`, and no control
character, `/`, `\`, or bidirectional override or isolate (U+202A–U+202E,
U+2066–U+2069). (Revision 2 named only the first two rules; the full rule
matters because the trusted dialog displays labels, 3.2.) A **control
character** is a code point of Unicode General_Category Cc: U+0000–U+001F and
U+007F–U+009F (revision 4). This is the set that the referenced
`catalog_core::wire::valid_name` refuses (`char::is_control`,
`services/catalog/src/wire.rs:134-149` *[main]*), and it includes the C1
controls, which are as unsuitable on the trusted display surface as the C0
controls; naming the set lets an independent implementation such as the R3
oracle apply the rule without reading the Rust code. Other format characters
(General_Category Cf outside the two bidirectional ranges, for example U+200E,
U+200F or U+061C) are not refused; the dialog must not rely on a label in any
case (2.6, 5.4). Labels need not be
unique, because the entry ID identifies the entry. Check 2 checks only the
label length (`name_length` 1..=32 and = `length`); the content rules are
checked by CREATE (order below) and, under the decided O14, also at mount
(2.6).

| Op | Name | Request | Reply | Permitted on |
|---|---|---|---|---|
| 1 | INFO | entry 0 | aux = free sectors, defined as formatted sector count − committed `next_free` (a writing entry's sectors stay counted as free until its SEAL commits, because only the commit allocates them), count = sealed entries, length 48: data = 16-byte destination ID, u64 formatted sectors, then the highest accepted grant generation of lanes 1, 2 and 3 (u64 each, 0 if never bound); in the failed-mount, `Corrupt` and blank candidate states the reply status and fields follow the table of 2.2 check 7 | every lane, control |
| 2 | LIST | entry 0, count = cursor 0..16 | up to 2 entry records of 80 bytes (2.6 layout), length = 80 × n; count = next cursor or 0. The cursor is a record index (0-based) in the mounted catalog: LIST returns the first two records visible to the lane whose index is ≥ the cursor, and the next cursor is the index of the next visible record after them, or 0 if there is none (0 cannot be a next index once a record was returned, so it means end) | control (all entries); export lane sees only its own entry once sealed; verify/restore lanes only their bound entry |
| 3 | CREATE | entry 0, label in data | entry = provisional ID (2.3), state `writing` | export lane, once per binding |
| 4 | APPEND | entry = writing ID, offset = bytes appended so far, ≤ 160 bytes | offset = new total; `Range` -3 if the new total would exceed 525 328 bytes (below) | export lane |
| 5 | SEAL | entry = writing ID, offset = total length | entry state `sealed`, aux = archive trailer FNV | export lane |
| 6 | ABORT | entry = writing ID | state discarded; on the control lane `NotFound` -19 if `entry` is not the writing entry (on an export lane that is check 6) | export lane, control |
| 7 | VERIFY | entry = bound ID | aux = source generation, count = objects; or `Corrupt`; a failed device read replies per "Device results" (2.5), never `Corrupt`, because a read failure is no evidence about the stored bytes | verify lane |
| 8 | READ | entry = bound ID, offset, aux = byte count 1..=160 | data = bytes; `Range` -3 with no data if `offset + aux` overflows (checked arithmetic) or exceeds the entry length, including a read that straddles the end; a failed device read replies per "Device results" (2.5) with no data | restore lane |
| 9 | BIND | entry = entry ID (0 for export), aux = new grant generation, count = lane 1..=3, name_length = operation (1 export, 2 verify, 3 restore), length 16: data = destination ID the grant names | Ok; `Stale` -10 with no state change if aux ≤ the lane's highest accepted generation; `NoDevice` if the mounted ID differs; `NotFound` if the entry is not sealed; `Busy` if the lane is bound | control |
| 10 | UNBIND | count = lane 1..=3 | Ok; any writing entry on that lane is discarded; the lane is unbound, so later requests on it get `Access` until a new BIND; UNBIND of an unbound lane is `Ok` with no state change, so the dialog may repeat it | control |
| 11 | FORMAT | entry 0, length 0 | Ok, reply data = new destination ID; `Access` unless the scope is blank (2.6) | control |

The request offset in APPEND must equal the service's own count of appended
bytes; a mismatch returns `Stale` and appends nothing. This makes a duplicated
or replayed APPEND harmless. An APPEND whose new total (`offset` + `length`)
would exceed 525 328 bytes, the largest v1 archive (2.8), returns `Range` -3,
appends nothing and changes no state: the entry stays `writing`, and the
portal may ABORT. The order is: offset (`Stale`), then this bound (`Range`),
then the scope end (`Full`, 2.7 step 2). No v1 archive is longer, so such a
stream can never seal; `Full` would report a destination condition ("destination
full", 3.2, and the terminal `Full` of O3) for what is a portal or format
defect, and a bound that does not depend on the device size gives the same
refusal on every destination. SEAL with an offset that differs from the appended
total returns `Invalid` and discards nothing; the portal may then ABORT. An
APPEND or SEAL whose `entry` is not the binding's writing entry returns
`Access` -5 at check 6 and changes nothing. A second CREATE on the same export binding
returns `Access` -5 at check 6, whatever the state of the first entry (the
first CREATE being the first answered `Ok` under the O15 rule, 2.2
check 6). A CREATE
while an entry is `writing` on another lane returns `Busy`: one writing entry
per destination. After check 7, CREATE applies its refusals in this order:
label content (`valid_name`, `Invalid` -1), then `Busy` -21, then `Full` -17.
A refusal that depends only on the request precedes those that depend on
service state, as in 2.2. `Busy` and `Full` never apply together: `Full`
is judged against the committed catalog, a writing entry exists only if its
CREATE passed that test against the same committed catalog, and only that
entry's SEAL commits a new one, so while any lane is writing fewer than 16
entries are sealed and at least one sector is free. The order of the two is
therefore not observable; it is stated only to fix the code path (revision 3,
review 4 RQ-1).

**Grant generations.** The service keeps, for each operation lane, the highest
grant generation it has accepted in a BIND, for the life of the instance; INFO
reports these maxima. A BIND whose generation is not above that maximum returns
`Stale` and changes nothing. BIND applies its refusals in the order of its
row: `Stale`, then `NoDevice`, `NotFound` and `Busy`. Revocation does not remove plain data messages
already queued on a lane (`maintain` cancels only messages whose endpoint is
closed or whose carried capability is revoked) *[main]*, and `request_id`
restarts with each binding; without this rule a re-BIND with an earlier
generation would let a queued APPEND or SEAL of the old binding pass checks 5
and 6 of 2.2. CREATE when 16 entries are sealed, or when
fewer than one sector is free, returns `Full`. VERIFY runs `archive::verify` inside the service
over exactly the entry's `length` bytes from the start of its extent, not over
the zero padding of the last sector; it is a media check that moves no content
over IPC.

**Archive verifier** (revision 3). In SEAL step 3 and in VERIFY the verifier is
`volume_core::archive::verify` *[main; `services/volume` is `no_std` and
depends only on `catalog-core`]*, linked into the service, so that the
service accepts exactly the bytes that the portal's own `archive::verify`
accepts before a restore. No other archive acceptance is permitted in a native
or product build. `archive::verify` refuses any byte after the archive trailer
(`services/volume/src/archive.rs:366-377` *[main]*), so an accepted entry ends
with its archive trailer and the record's archive trailer FNV equals the last
8 bytes of the entry. A host test may replace the verifier with a stand-in
only in test code; a stand-in must refuse trailing bytes in the same way, and
rows that "verify" an entry count at R2 only for the verifier actually linked
(8.2). The
user-facing acceptance check before a restore stays `archive::verify` in the
portal over READ, on the same bytes it then restores, as the restore rule
requires.

### 2.4 Status codes

Service statuses reuse `volume_core::Status` *[main]* values: `Ok` 0, `Invalid`
-1, `NoSys` -2, `Range` -3, `Access` -5, `Stale` -10, `Io` -12, `Timeout` -13,
`NoDevice` -14, `Protocol` -15, `Unknown` -16, `Full` -17, `Corrupt` -18,
`NotFound` -19, `Busy` -21. No new code is introduced.

**Layer ambiguity.** Kernel `capability_core::Error` codes overlap numerically
with service statuses: `Revoked` and `Stale` are both -10, `Access` is -5 in
both, `Invalid` is -1 in both *[main]*. A kernel error is the syscall return of
`send`/`recv`; a service status is the reply header's `status` field. Callers
and tests must record which layer produced a code; the matrix in section 6
always names the layer.

### 2.5 Entry state machine

```
(none) --CREATE--> writing --SEAL ok--> sealed
                     |  \--ABORT / UNBIND / revoke / reboot--> (discarded)
                     |  \--entry write or flush error (APPEND, SEAL last sector, barrier 1)--> (discarded; safe path, 2.7)
                     \--SEAL fails (readback Corrupt, Io or Timeout, Full, pre-write check)--> (discarded)
                     \--SEAL catalog write or flush error (steps 4-5)--> (decided by the catalog after remount; safe path)
```

Reply codes on these edges: an entry write or barrier 1 replies the status
that "Device results" below gives for a write or flush (`Io` -12 or `Unknown`
-16; revision 2 also allowed `Timeout` -13 here, which revision 3 removes); a
failed step-3 readback replies `Timeout`
-13 (a read past its deadline, which the broker reports as -13) or `Io` -12;
an extent that fails `archive::verify`, an extent whose content digest differs
from that of the appended bytes (*[owner decision O17 (a), 23 November 2026]*, 2.7 step 3)
or a catalog copy that fails the
pre-write check of step 4 replies `Corrupt` -18; a failure in steps 4–5 replies
`Unknown` -16 (2.7). Every device error on a destination write or flush of an
export (APPEND, SEAL) starts the read-only safe path of 2.7; a failed mount or
remount FLUSH and a failed FORMAT write or flush lead instead to the stricter
failed-mount state (2.6); a failed readback does neither, because no write
is outstanding after barrier 1 has completed. A `Corrupt` reply to SEAL
concerns that entry only and does not put the service into the `Corrupt`
mount state of 2.6.

**Device results** *[design; revision 3]*. A request's reply status that comes
from the broker's result for one device request follows this table, whatever
the operation. It governs replies to requests; the failed-mount state still
records whether the FLUSH or read that failed a mount or remount, or the
FORMAT write or flush that failed, met kernel -14, and INFO reports
`NoDevice` -14 in that case (2.6). SEAL steps 4–5 reply `Unknown` -16 for every
device result (2.7).

| Broker result | Read (FORMAT scan, step-3 readback, VERIFY, READ) | Write or flush |
|---|---|---|
| submission still -8 after the submission bound (2.7) | `Io` -12 | `Io` -12 |
| any other submission result than 0 or -8: -13 (quarantined after a reset timeout), -14 (absent, or offline after a failed re-initialisation), or another error (for example a refused `prepare`) | `Io` -12 | `Io` -12 |
| completion -12 | `Io` -12 | `Io` -12 |
| completion -13, or completion poll still -8 after the poll bound (2.7) | `Timeout` -13 | `Unknown` -16 |
| any other completion result other than success (-16, -14 or an undefined code) | `Io` -12 | `Unknown` -16 |

Reasons. A submission refused by the broker was not admitted, so no data was
transferred through it: `Io`. A write or flush that was admitted and did not
complete with a definite result may or may not have reached the medium;
`Unknown` is the only reply that claims neither, while `Timeout` would suggest
a request without effect. The broker answers a timed-out mutating request with
-16 (2.7), so a -13 completion of a write is outside the broker contract and is
treated like a timeout. A read changes nothing on the device, so a read past
its deadline is `Timeout` and every other read failure `Io`. A submission
refused with kernel -14 after the boot mount can only mean that
re-initialisation after an acknowledged reset failed, because v1 has no runtime
attachment or removal of the destination (K1); that is a device fault and is
answered `Io`, so that `NoDevice` keeps one meaning for the portal: the named
destination is not the one mounted (BIND), or a mount, remount or FORMAT met
kernel -14 and the instance is in the failed-mount state (INFO, 5.3). A
completion result, including a completion -14 (which the K5 broker contract of
2.7 does not produce), follows the table above. A completion poll that
returns -1 (no such request; property 3 of the broker contract, 2.7) carries
no device result. Under the poll rule of 2.7 ("Meaning of kernel -8") the
service never polls without a request of its own pending, so -1 does not
occur; if it does, it shows a service defect and is answered as "any other
completion result" of the table, `Io` -12 for a read and `Unknown` -16 for a
write or flush, which starts the safe path; on a mount or remount read or
FLUSH, or a FORMAT write or flush, it leads to the failed-mount state
instead (2.6) (fail closed;
revision 5). No row
is added for it. This mapping rests on v1 having no runtime
attachment; if K1 is approved (O4 (a)), the reply to -14 after the boot mount
is to be decided again. Every failure of a destination write or flush of an
export, whatever its reply, starts the safe path of 2.7 (a mount or remount
FLUSH and a FORMAT write or flush lead to the failed-mount state instead,
2.6); no read failure does.

`writing` exists only in service memory. A `writing` entry is never returned by
LIST, never readable and never verifiable. After any reboot it does not exist;
its sectors are free space. `sealed` entries are immutable: no operation writes
their sectors, and v1 has no operation that deletes them (section 8, O3).

### 2.6 On-device layout

All integers little-endian. Sector size 512. Every sector number in this
document is relative to the first sector of the granted destination scope;
sector 0 is the scope's first sector, and the scope's `sector_count` is the
formatted sector count. Sectors outside the scope are not part of the
destination: the service cannot reach them (2.1) and an image oracle neither
reads nor judges them (section 7).

| Sectors | Content |
|---|---|
| 0–3 | catalog copy A (2048 bytes) |
| 4–7 | catalog copy B (2048 bytes) |
| 8–15 | reserved: zero after FORMAT, never written by the service, read only for the blank test (below) |
| 16 .. `next_free` | sealed entry extents, ascending and contiguous (CREATE allocates at the committed `next_free` and SEAL advances it past the extent, so a discarded write leaves no gap) |
| `next_free` .. end of scope | free; may hold stale bytes of discarded writes |

**Catalog copy (2048 bytes).** Header 64 bytes: magic `NOSDST\0\x01` (0..8),
version u16 = 1 (8..10), sealed-entry count u8 ≤ 16 (10), 5 × 0 (11..16),
destination ID 16 bytes (16..32), sequence u64 ≠ 0 (32..40), next entry ID u64
(40..48), `next_free` sector u64 (48..56), formatted sector count u64 (56..64).
Then 16 entry records of 80 bytes (64..1344): entry ID u64, first sector u64,
length u64 (bytes, ≥ 80 and ≤ the largest v1 archive), archive trailer FNV-64
u64, sealing sequence u64, label length u8, 7 × 0, label[32] zero-padded. Unused
records and bytes 1344..2040 are zero. Bytes 2040..2048 hold FNV-64 over
0..2040. **FNV-64** in this document, for the catalog checksum and for the
archive trailer, is 64-bit FNV-1a: offset basis `0xcbf29ce484222325`, prime
`0x100000001b3`, as `catalog_core::wire::checksum_update` and the v1 archive
format use *[main]*. The archive trailer FNV-64 of a record is the FNV-64
value stored in bytes 8..16 of the archive's 16-byte trailer, that is the last
8 bytes of the entry (2.3, "Archive verifier"). The **sealing sequence** of a record is the sequence of
the catalog copy that first lists it: the new copy's sequence in 2.7 step 4.

A copy is valid only if every rule holds (rules marked † are new in revision
3: the entry-ID and label-length rules were stated only in 2.3, the length
and copy-sequence rules only in the field descriptions above, and the
sealing-sequence rules are new):
- magic, version and checksum;
- count ≤ 16;
- copy sequence ≥ 1 †;
- for every record: entry ID ≥ 1 (2.3) †; length 80..=525 328 bytes †; label
  length 1..=32 (2.3) †; sealing sequence ≥ 2 and ≤ the copy's sequence †;
- records strictly ascending by entry ID, by first sector and by sealing
  sequence †;
- extents contiguous (the first record starts at sector 16, each further
  record starts where the previous extent of ⌈length/512⌉ sectors ends, and
  `next_free` equals the end of the last extent, or 16 with zero entries);
- 16 ≤ `next_free` ≤ formatted count (also for a catalog with zero entries);
- formatted count = granted scope's `sector_count`;
- every entry ID < next entry ID, and next entry ID ≥ 1 (also for a catalog
  with zero entries);
- all padding zero: header bytes 11..16, record bytes 41..48, label bytes
  after the label length, unused records and bytes 1344..2040.

The sealing-sequence rules follow from the commit order of 2.7: FORMAT commits
sequence 1 with no record, and each SEAL commit adds exactly one record whose
sealing sequence is the new copy's sequence, and no v1 operation changes a
record. A record that breaks them cannot result from that order and shows
damage, a crafted copy or an encoder defect. The validity rules alone catch a
wrong sealing sequence on a new record only when the committed copy's highest
sealing sequence (1 if it has no record) equals its sequence, which is not so for every
valid copy (for example a crafted one, or one after an O3 deletion); the
pre-write check of 2.7 step 4 therefore also requires that the new record's
sealing sequence equal the new copy's sequence. That requirement belongs to
the pre-write check only, not to the mount rule. The rules
stay true if a later deletion operation (O3) commits without adding a record.
The entry-ID and label-length rules make the copy validator agree with 2.3,
which already forbade entry ID 0 and labels outside 1–32 bytes.

**Label content at mount** *[owner decision O14, 23 November 2026]*. A copy is also
invalid if any record's label fails `valid_name` (2.3) in full. Reasons: CREATE
admits only labels that pass `valid_name`, so a conforming service never writes
another; the pre-write check of 2.7 step 4 then also catches an encoder defect
in a label; and the trusted dialog displays labels (3.2), so a label with
control characters or bidirectional overrides from a damaged or crafted image
would otherwise reach a trusted surface. Cost: one bad label makes the whole
destination `Corrupt`, with no read-only fallback for the other entries. The
owner declined the alternative, the length and padding rules only with the
dialog rendering every label as untrusted text. The dialog must still not
rely on a label: a crafted image can carry any valid label (5.4). Under this
rule the label-length rule "1..=32" of the validity list is partly implied:
`valid_name` refuses an empty label, so a validator that omits only the
"≥ 1" bound reaches the same mount state for every copy (equivalent mutant,
N34). The bound is kept as a stated rule, because it is the length rule of
2.3 and does not depend on the order in which an implementation applies the
two rules (revision 4).

The destination ID is 16 bytes drawn once at format time from the existing
entropy capability (an `Entropy` object and handle held by the destination
service; section 5.1). It is an identifier, not a secret and not authentication.
FNV-64 detects accidental damage only; a deliberately altered, swapped or
rolled-back image is not detected (section 5.4).

**Mount rule.** Every mount, including a remount (2.7), first issues a FLUSH
and reads sectors 0–15 only after it completes, so that the copies read are
not merely held in a volatile write cache. **A read error is not an invalid
copy:** if any read of sectors 0–15 at mount or remount fails (device `Io`,
`Timeout` or `Unknown`), the service enters the failed-mount state below and
writes nothing; a sector whose read failed also never counts as zero. Treating
a failed read of the newer copy as *invalid* would select the older copy, roll
`next_free` back to the first sector of the newest sealed extent and let the
next CREATE write into it (N8). The cost is a stated limit: a persistent read
error on a catalog sector leaves every entry on the destination unreachable
until the image is repaired outside the service; no read-only fallback is
specified.

With all reads successful: if exactly one copy is valid, use it. If both are
valid and byte-identical, use either. If both are valid and differ, the copy
with the higher sequence must *extend* the other: the same destination ID and
formatted count, the lower copy's records an exact prefix of the higher copy's
records, and `next_free` and next entry ID not lower; then the higher copy is
used. Every legitimate state satisfies this (a completed commit leaves
identical copies; a crash between barriers 2 and 3 leaves the newer copy one
record ahead). Equal sequences with different bytes, or a higher copy that does
not extend the lower, give `Corrupt`. For two valid copies, the record-prefix rule with contiguous
extents already implies that `next_free` is not lower, and both formatted
counts equal the scope's `sector_count`; these two clauses are kept as
defensive checks, cannot fail on their own, and mutants that drop them are
equivalent (N30). Sectors 8–15 are not a validity rule: no write path reaches
them (2.7 step 2) and no decision except the blank test depends on them, so a
valid copy mounts beside non-zero sectors 8–15. A rule on them would make a
valid destination `Corrupt` over bytes that nothing uses, and would not detect
a foreign writer in general, because sectors at or beyond `next_free` are not
checked either; FORMAT's full-scope scan still refuses them. If neither copy is valid and all sectors
0–15 read as zero, the device is a *blank candidate*; zero in sectors 0–15 is
only a heuristic and does not show that the rest of the scope holds no data.
In that state INFO reports `NotFound` -19, FORMAT and UNBIND run, and every
other request that passes checks 1–6 of 2.2 returns `NotFound` -19 (table of
2.2 check 7). Otherwise the destination is `Corrupt`: INFO is answered and
reports `Corrupt` -18 in its status field, every other request that passes
checks 1–6 of 2.2 is answered `Corrupt` -18, and the service never formats
automatically. `Corrupt` is terminal for the service instance: no request
clears it and the service never repairs or formats, and the instance of a
later boot that mounts the same bytes reaches `Corrupt` again. For a disposable test image the
only recovery is external: its owner (for the gates, the harness that created
it) re-zeroes the whole granted scope, not only sectors 0–15, because FORMAT
refuses a scope with data at sector ≥ 16; the next boot's instance then mounts a
blank candidate and FORMAT is permitted (N25). How a product distinguishes a FORMAT
interrupted before its first copy was durable from a damaged existing
destination, and whether any in-product recovery is offered, is open
(section 8, O10).

**Failed-mount state.** The service enters it when a mount or remount FLUSH
fails (any device error, a kernel -8 that persists past the bound of 2.7, or
kernel -14 for an absent or offline device, `volume.rs:967` *[main]*), when a
read of sectors 0–15 fails (mount rule above), when a FORMAT write or flush
fails (Format below), or, *[owner decision O16, 23 November 2026]*, when a remount
(2.7) would end as a blank candidate, or would mount a valid catalog that is
neither the committed copy kept for this comparison (2.7) nor that copy plus
exactly the provisional record (for example another destination ID, or other
records). A remount follows a mount that read a
valid, flushed catalog in the same boot, and 2.7 writes only one slot at a
time, so every legitimate sequence leaves one of those two catalogs at a
remount, never sectors 0–15 all zero or another catalog; such a result shows a
changed or misbehaving device, not a blank destination.
The export binding then stays *unknown*, because a read that contradicts the
catalog mounted earlier is no evidence about its entry, bound verify and
restore lanes keep entry IDs that the other catalog may give to other
entries, and FORMAT is refused with `Io` -12 like every other request. The
owner declined the alternative, the state that the mount rule gives (a blank
candidate with the safe-path latch kept, or mounted with that catalog and the
latch kept). The FORMAT refusal with `Access` -5 in a blank candidate with the
latch set, which that alternative needed, is kept as the defensive rule of
2.2 and is unreachable under this rule. The *sealed* resolution requires
the exact copy encoded for the SEAL (2.7,
*[owner decision O18, 23 November 2026]*). In the failed-mount state the service writes nothing and issues no further device request. INFO is
answered and reports `NoDevice` -14 in its status field if the broker returned
-14, otherwise `Io` -12; LIST on an export lane whose binding is still
*unknown* (2.7) returns status `Unknown` -16 with zero records; every other
request that passes checks 1–6 of 2.2 returns `Io` -12. This state is terminal
for the service instance: the service does not retry the FLUSH or the read and
no request clears the state, because a failed FLUSH gives no evidence about
earlier writes, and in the broker pattern of 2.7 a device reset that is not
acknowledged quarantines the kernel `Policy` until reboot
(`crates/virtio-core/src/lib.rs:560-569`, `kernel/src/m2/m3/volume.rs:1075-1079`
*[main]*). **Recovery is a reboot.** The destination block grant is
manifest-only and non-transferable, and under the once-per-boot grant rule
(2.1, O13) no other instance in the same boot can obtain it; normal operation
resumes only in the next boot's instance, after
its own mount FLUSH completes. Until then the replies above apply unchanged,
and the portal reports the destination as unavailable and any pending export as
not confirmed. An export binding whose outcome is *unknown* stays *unknown* for
the life of the instance; it never resolves as *sealed* without a completed
FLUSH. In the next boot, bindings and provisional IDs are void; the entry is
judged only from the catalog that that instance reads after its own
completed mount FLUSH, like any other sealed entry (control LIST, VERIFY), and
the earlier export is never reported as confirmed on the basis of its
provisional ID.

**Format.** Only the trusted control path may format, and only a *blank*
scope: before any write, FORMAT reads the whole granted scope and returns
`Access` unless every sector is zero; a read error during that scan returns
the read's reply status under "Device results" (2.5: `Io` -12, or `Timeout`
-13 for a read past its deadline), writes nothing and leaves the instance a
blank candidate. After the scan and before its first write, FORMAT draws the
destination ID; if the draw fails, FORMAT replies `Io` -12, writes nothing and
leaves the instance a blank candidate, because no write was issued and the
device is as before (the failed-mount state exists for the uncertainty that a
failed write leaves). Format writes copy A with sequence 1,
zero entries, next entry ID 1 and `next_free` = 16, flushes, writes the identical copy B, and flushes.
After the last flush completes, FORMAT replies `Ok` and the instance is
mounted with that catalog. If any FORMAT write or flush gives a device result
other than success, FORMAT replies the write-or-flush status of "Device
results" (2.5: `Io` -12, or `Unknown` -16 for a write or flush without a
definite result), the instance enters the failed-mount state above
and does not remount; the next boot's instance mounts whatever reached the
device (blank candidate, `Corrupt`, or one valid copy; N25). The cost of the full-scope
read on large devices is not measured *[unproven]*.

### 2.7 Durability and flush order

The `Disk` contract *[main]* applies: a completed write is not durable until a
later flush. The primary source for the barrier is *Virtual I/O Device
(VIRTIO) Version 1.2*, OASIS Committee Specification 01
(<https://docs.oasis-open.org/virtio/virtio/v1.2/cs01/virtio-v1.2-cs01.html>):
section 5.2.3 defines feature bit 9, `VIRTIO_BLK_F_FLUSH`; section 5.2.6
defines request type `VIRTIO_BLK_T_FLUSH`; section 5.2.6.2 treats a completed
write as stable once a flush request sent after it has completed. The driver
negotiates bit 9 unless a test profile clears it (`no_flush`,
`drivers/virtio-blk/src/lib.rs:243-244` *[main]*). Three claims are kept
apart: the specification states the requirement; that QEMU and its host
storage stack honour it is assumed *[unproven]*; durability on physical media
is *[unproven]* and not tested here.

1. CREATE reserves `first = next_free` in memory only.
2. APPEND writes whole sectors in `[first, first + ⌈len/512⌉)`; a partial last
   sector is buffered and zero-padded at SEAL. A write at or beyond the scope end
   returns `Full`. There are exactly two write paths: the entry path accepts
   only `sector >= max(16, next_free)` (`next_free` the committed value) and
   below the scope end, so it never relies on catalog validation to keep
   sectors 0–15 out of reach; the
   catalog path accepts only catalog sectors 0–7 (order below). Any other
   write is refused before it reaches the device.
3. SEAL: write the padded last sector; **flush (barrier 1)**; read the extent
   back and run `archive::verify` on its first `length` bytes (2.3, "Archive
   verifier"); on failure reply `Corrupt` and discard.
   *[owner decision O17 (a), 23 November 2026]* Content digest: the service also keeps,
   for the writing entry, the FNV-64 (2.6) of the bytes of every APPEND
   answered `Ok`, in append order, and compares it with the FNV-64 of the
   first `length` readback bytes; on a mismatch it replies `Corrupt` -18 and
   discards, as for a failed `archive::verify`. With the real
   `volume_core::archive::verify` linked (below), an equivalent form compares
   the readback's trailer FNV-64 (its last 8 bytes, which `archive::verify`
   requires to equal the FNV-64 of every byte before the 16-byte trailer)
   with the last 8 bytes of the concatenated APPENDs answered `Ok`; for a
   portal archive that is itself a valid v1 archive, this is at least as
   strong against accidental faults as the running digest (review 6
   OPT6-9). The running form above is the preferred form (revision 5): it
   covers every appended byte whatever the archive format and whatever
   verifier is linked. The trailer form is relied on only with the real
   `volume_core::archive::verify` linked (2.3, "Archive verifier"; an R5
   precondition), because its strength rests on that verifier's trailer
   rule, and a portal v1 archive need not pass a test stand-in with another
   trailer rule (review 7, OPT7-1); a host build with a stand-in verifier
   therefore uses the running form. At `f993a1a3` the R2 core uses
   the running form *[PR #413]*. Both forms keep the decided meaning of
   O17 (a). `archive::verify` and the content check, in either form, must
   both pass; both failures reply `Corrupt`, so their order is not
   observable.
4. Encode the new catalog copy (sequence + 1, the entry appended with its
   sealing sequence equal to the new sequence, `next_free` advanced past the
   extent, next entry ID + 1). Sequence + 1 and next entry ID + 1 use checked
   arithmetic: both values come from disk bytes, and a copy with either at
   the u64 maximum passes the validator, so an overflow is a failed encoding
   and is handled like a failed check below (revision 3). The committed copy
   keeps the maximum, so with such a copy every later SEAL of the instance
   fails in the same way and the destination accepts no new entry until it
   is repaired outside the service (N29). **Before any write**, run the
   mount validator of 2.6 on the encoded copy and check that it extends the
   committed copy: the same records plus exactly one, sequence + 1, `next_free`
   not lower, and the new record's sealing sequence equal to the new sequence
   (review 4 OP-3: the validity rules alone do not force that value, 2.6).
   For an encoded copy that passes the validator, the sealing-sequence clause
   already implies "exactly one": with no new record the clause has no
   record to hold for, and a second new record would need a sealing
   sequence above the new copy's sequence, which the validity rules refuse.
   The record-count clause is kept as a defensive check, and a mutant that
   drops it is equivalent (N29, N35; revision 4). The equivalence relies on
   the 16-entry bound: CREATE refuses with `Full` when 16 entries are sealed
   (2.3), so step 4 never starts from a committed copy with 16 records; in an
   implementation the clause also keeps the index of the new record in range
   (review 6 OPT6-8). If
   either check fails, reply `Corrupt` -18, discard the writing
   entry and write nothing; this guards against an encoder defect and does not
   enter the `Corrupt` mount state. Then write the copy into the *target*
   slot: the slot holding an invalid copy (one that read successfully but
   failed validation; a read error never reaches this step, 2.6), otherwise
   the slot with the lower sequence,
   otherwise (both copies valid and byte-identical) copy A; **flush
   (barrier 2)**.
5. Mirror the identical copy into the other slot; **flush (barrier 3)**. Reply
   `Ok` only after barrier 3 completes.

**Stale valid content at the readback** (revision 4). The step-3 readback with
`archive::verify` shows that the extent holds *a* valid archive, not that it
holds the archive the portal appended. Free space may hold stale bytes of a
discarded write (2.6 layout). If a device fault loses completed, unflushed
APPEND writes before barrier 1 (for example a device reset that drops the
volatile write cache, N33, or the device's own reset during the barrier-1
submission, which drops every unflushed write of the entry, N26 (d)) or
silently fails to store a write, and the extent
then holds a stale valid archive of the same length, `archive::verify` alone
accepts it; without the content check of step 3, SEAL would commit an entry
with another backup's content, only the SEAL reply's `aux` (the stored
trailer FNV-64) would differ from the portal's own trailer, and the portal's
restore-time `archive::verify` would accept that content. A sealed entry is
immutable and v1 cannot delete it (O3), so it would keep its slot and its
new label (revision 5: stated as the case that the decided service check
below prevents before the commit, and that the decided portal check would
report as a failed export if the service check did not; review 9, OPT9-5;
review 10, OPT10-4). Without the service check, the commit itself would be
a limit of the design, and the portal check alone would keep only the
portal from reporting it as a success; the owner therefore decided both
(O17 (c)).
*[owner decision O17 (a), 23 November 2026]* The content digest of step 3 refuses such
an extent before the commit. *[owner decision O17 (b), 23 November 2026]* The portal
compares the trailer FNV-64 that the service reports to it with the trailer
of the archive it exported, and reports the export as failed, never as
sealed or confirmed, on a mismatch (below). **Threat model:** both checks detect
an accidental mismatch caused by a device or cache fault, not a hostile
device. FNV-64 is not collision-resistant, and a device that misbehaves on
purpose can return the appended bytes on the readback and store others; a
malicious device stays out of scope (5.4). A defence against a hostile device
would need a maintained standard cryptographic hash and is a separate
decision, which this document does not make. A digest mismatch does not start
the safe path: no write or flush is outstanding after barrier 1, the extent
lies at or beyond the committed `next_free` and is not allocated, and the
mismatch is evidence about this entry only, as a failed `archive::verify` at
step 3 is (2.5). This follows the revision-3 rule for a failed step-3
readback and depends on O9: the mismatch itself shows that the device did not
keep writes it reported as completed, and if such writes could land late, the
next CREATE of the same boot would reuse that extent. Whether a digest
mismatch, or any device reset while unflushed entry writes exist, must also
start the safe path was the latching sub-question of O17 (review 6 OPT6-1).
It is decided with O17: **no latch, pending O9** *[owner decision O17,
23 November 2026]*. A mismatch replies `Corrupt` -18, discards the writing
entry, writes nothing and leaves the safe-path latch clear, so a later CREATE
of the same boot runs. The question reopens only on O9 evidence that a write
the device reported as completed, or accepted before a reset, can reach the
medium later; that evidence would require a new owner decision, and N26 (d),
N33 and N37 would then change with it (revision 5).

If the write of step 4, barrier 2, the write of step 5 or barrier 3 gives any
device result other than success (`Io`, `Timeout` or `Unknown`; the broker
pattern below reports a timed-out write or flush as `Unknown` -16), the
service replies `Unknown`, discards its in-memory writing-entry state and its
in-memory catalog as the source of truth, keeping the committed copy and
the copy encoded in step 4 (O18) only to judge the remount (O16 below),
and remounts from the device before
it receives another request (state
"remount pending" of 2.2 check 7; this interval includes the completion fence
below and is limited by the -8 and poll bounds of "Time source" below). Lane
bindings, including the export binding's provisional entry ID, are kept across
this remount; no CREATE is accepted on any lane before the outcome below is
resolved, nor, under the safe path below, for the rest of the boot, so that
ID cannot be issued again in the meantime.

A catalog read after a failed write or barrier can return bytes still held in
the device's volatile write cache, which a power loss may discard. The
remount therefore begins with a FLUSH (2.6, mount rule), and only a remount
whose FLUSH completed and that mounted a catalog resolves the export binding,
once: *sealed* if the remounted catalog lists the provisional entry ID,
otherwise *not sealed*; the outcome is fixed in the binding and does not
depend on later catalog changes. A remount that ends `Corrupt` mounts no
catalog and leaves the outcome *unknown* (the `Corrupt` row of 2.2 check 7
applies). *[owner decision O16, 23 November 2026]* "That copy plus exactly the provisional
record" means a catalog that passes the step-4 extension check against the
committed copy (one more record, sequence + 1, sealing-sequence clause
included) and whose new record carries the provisional entry ID. A remount
whose reads would give a blank candidate, or a valid catalog that is neither
the committed copy nor that copy plus exactly the provisional record, does
not resolve the binding:
the instance enters the failed-mount state (2.6) and the outcome stays
*unknown*.

Equality with the committed copy is equality of its 2048 bytes; for valid
copies this is the same as equality of every field, because the validity
rules fix every padding byte and the checksum (2.6; clarification, revision
4). *[owner decision O18, 23 November 2026]* "That copy plus exactly the provisional
record" is met only by a remounted catalog whose 2048 bytes equal the copy
that the service encoded and checked in step 4 of this SEAL, which it still
holds at the remount. That copy passed the step-4 extension check, so this
rule implies the O16 definition above and refuses, in addition, a catalog
with the provisional entry ID whose new record differs in length, archive
trailer FNV-64 or label, or whose next entry ID differs; such a remount enters the failed-mount state with the
outcome *unknown*. Reason: the writes of this boot can leave at a remount only
the committed copy or the encoded copy in either slot (2.6 failed-mount
state), so any other catalog that names the provisional ID shows a changed or
misbehaving device, which O16 already refuses to resolve from; and a record
whose content differs from the encoded one would make the export lane's LIST
report a record that the service never wrote. O18 refines the *sealed* arm
of the decided O16 rule and leaves its other arms unchanged. The owner
declined the alternative, the O16 definition alone (the extension check plus
the provisional entry ID). Because the encoded copy itself passed the step-4
extension check, an implementation that tests the extension check beside the
equality is equivalent to one that tests the equality alone (N35); the
extension check stays required only as the step-4 pre-write check.

The lane accepts only INFO and LIST (check 6 of 2.2, from the `Unknown`
reply on); its LIST returns the entry record if the outcome is *sealed* and
zero records otherwise. If the remount's FLUSH or
catalog read fails, the outcome stays *unknown* and the failed-mount state of
2.6 applies (INFO answered with `Io` or `NoDevice`; the export lane's LIST
returns `Unknown` with zero records; every other request that passes checks
1–6 of 2.2 `Io`). Bindings do not survive the reboot that ends that state; the
next boot's instance mounts only after its own mount FLUSH completes, and the
export is then not confirmed to the portal.

**Completion fence.** The remount FLUSH is issued and the export binding is
resolved only after the request that failed is quiescent: completed and
consumed, or ended by an acknowledged device reset. Until that fence the
binding stays *unknown* and the service receives no request (state "remount
pending", 2.2 check 7); after it, CREATE stays refused for the rest of the
boot under the safe path below. On
`main` the M3 volume broker implements this fence for its device *[main]*:
one global `Policy` admits at most one request
(`kernel/src/m2/m3/volume.rs:124-126`); a request past its deadline is answered
`Unknown` -16 if mutating (WRITE and FLUSH are mutating,
`crates/virtio-core/src/lib.rs:56-58`) or `Timeout` -13 otherwise, and the
device is reset (`volume.rs:1087-1099`, `:1006-1016`); while the reset runs the
broker admits no request (`:964`, kernel return -8); the device is reused only
after it reads back status 0 with bus mastering disabled
(`drivers/virtio-blk/src/lib.rs:511-524`, `volume.rs:1043-1074`), otherwise the
lane is quarantined and offline (-13 on reset timeout, `volume.rs:1075-1079`;
-14 if re-initialisation after an acknowledged reset fails, `:1068-1072`)
until reboot.
The per-device destination broker (K4/K5) must provide the same fence for
disk 1; this is a requirement, not existing behaviour.

*Meaning of kernel -8.* On `main` a submission (syscalls 0x40, 0x42, 0x43)
returns -8 when `r.resetting || r.error.is_some() || r.policy.phase() !=
Phase::Ready` (`volume.rs:964`), and these are global broker fields
(`volume.rs:122-131`) *[main]*: while a source-lane request is outstanding or
its result unconsumed, the destination would legitimately see -8 for up to
`TIMEOUT` = 500 ticks (`volume.rs:24`) plus a reset. The completion poll
(0x41) returns -8 while the caller's own request is still pending
(`volume.rs:911-912`) *[main]*. This document therefore defines K5 (4.3) as
**per-device broker state**: the `Policy`, `resetting`, `reset_deadline`,
`error`, `active` and the data buffer are kept per block device. Under K5 the
destination service is disk 1's only client, and it consumes the result of
each request (polls until the poll returns something other than -8) before it
submits the next one. It follows that the service polls only a request that
it submitted, that the broker admitted (the submission returned 0), and
whose result it has not yet consumed; it never polls with no request of its
own pending *[design; revision 5]*. At `f993a1a3` the single request loop
of the R2 core (`device::run`) meets this by construction: it polls only
after a submission returned 0 and stops at the first poll result other than
-8 or at the poll bound below *[PR #413]*. A stray-poll counter in the
simulated disk, asserted zero in every service-driven host test, is an open
R2 item (8.2 R2; CODE-DELTA (revision 5), Q6; wording aligned after review 9,
OPT9-4). A -8 on a disk-1 submission can then mean only disk 1's
own reset, which ends within `RESET_TIMEOUT` = 10 ticks (`volume.rs:25`)
either ready or offline; offline counts as a failed FLUSH (2.6). A -8 on a
submission is not a device result: the service resubmits. The service bounds
the wait itself *[design]*: a -8 on a submission that persists for longer
than 2 × `RESET_TIMEOUT` (twice the longest legitimate reset under K5, the
factor being a margin for tick granularity) counts as a device `Io` for that
request, and a completion poll that still returns -8 after `TIMEOUT` + 2 ×
`RESET_TIMEOUT` from the admission of the request counts as `Unknown` -16 for a write or
flush and `Timeout` -13 for a read. The submission bound is measured from the
first submission attempt of the request; the poll bound is measured from the
tick at which the submission was admitted (returned 0), because the broker's
own deadline starts at admission (the M3 broker passes the current tick to
`Policy::prepare` when it admits a request, `volume.rs:970`, and `prepare`
adds `TIMEOUT` to it, `crates/virtio-core/src/lib.rs` *[main]*; property 4 of
the broker contract below), and a bound measured from the first attempt
would let a legitimate reset wait shorten the poll margin and turn a request
that the broker still answers within `TIMEOUT` into a false `Unknown`. The
longest wait for one request is therefore the submission bound plus the poll
bound. For a mount or remount FLUSH an exceeded bound is a
failed FLUSH and gives the failed-mount state (2.6); for any other destination
write or flush it starts the safe path below; a read fails with `Io` or
`Timeout`. The bound is not valid against `main`'s shared broker state, where
it would turn ordinary source latency into a destination `Io`, and where a
quarantine leaves the shared phase `Quarantined`
(`crates/virtio-core/src/lib.rs:562-568`) so that the other lane receives -8
indefinitely (`volume.rs:961-966`) *[main]*; the destination service is
therefore never run on shared broker state, and full per-device state is an R5
precondition.

**Broker contract required of K5** *[design; revision 3; R5 precondition]*.
The service's rules above rely on these properties of the per-device broker
for disk 1, which K5 must provide and which every host model of the broker
(R2) must reproduce:
1. At most one request of the device is admitted at a time.
2. A submission returns -8, and changes nothing, while a request of the device
   is pending or its result unconsumed, or while the device is being reset.
   This includes a request whose poll bound the service has already given up
   on (above): the broker keeps that request until it answers it or resets
   the device, so the service's next submission, such as the remount FLUSH,
   receives -8 and is limited by the submission bound.
3. Each admitted request is answered exactly once: a completion poll returns
   -8 while it is pending and then its result once. A further poll, after
   the result has been delivered or with no request admitted, returns -1
   (no such request) and changes nothing: no device state, no buffer
   content (revision 5). On `main` the M3 broker answers a poll with no
   pending request with -1 (`kernel/src/m2/m3/volume.rs:886-891`;
   `Policy::consume` in `crates/virtio-core/src/lib.rs` clears the request
   after a delivered result) *[main]*. For a further poll after a request
   ended by a reset, the code of `035b44e5` and of `origin/main` at
   `b9aa517c` (identical `volume.rs` and `crates/virtio-core/src/lib.rs`)
   was read, not executed, for the review-9 fix pass (OPT9-3). A mutating
   request ended by a reset is kept with its terminal result and consumed by
   its first poll, so a further poll returns -1. A read ended by a reset (its
   deadline or a device error) is not consumed: the first poll returns the
   read's error code, `Policy::peek` fails with `Offline` while the phase is
   `Resetting` (`crates/virtio-core/src/lib.rs:734-748`), and a further poll
   returns -14 (`volume.rs:900-906`, `err`); `reset_completed(true)` then
   drops the read (`crates/virtio-core/src/lib.rs:577-579`) and a later poll
   returns -1, while a reset timeout or a failed re-initialisation
   quarantines the policy with the read still pending (`volume.rs:1068-1079`),
   so -14 lasts until reboot or until the owning process exits, which drops
   the pending read (`Policy::exit`, `crates/virtio-core/src/lib.rs:788-790`)
   and clears `error` (`detach`, `volume.rs:1017-1026`) *[main]* (read, not
   executed). This is a K5 delta (4.3) and an R5 precondition (8.2 R5): under
   K5 a further poll after a read ended by a reset returns -1 (property 3).
   The host model returns -1 in both cases (`p12_3_answered_exactly_once`)
   *[PR #413]*. The service does not rely on -1 (2.5).
4. A request's deadline starts when it is admitted. A request past its
   deadline is answered -16 if it is mutating (WRITE, FLUSH) and -13
   otherwise, and the device is reset; the device is reused only after the
   reset is acknowledged (completion fence above).
5. After a reset timeout the device is quarantined and a submission returns
   -13; after a failed re-initialisation it is offline and a submission
   returns -14; both last until reboot.

A broker or host model that asserts or panics on a submission while a request
is pending does not meet property 2.

The contract is not relaxed for host models (revision 5): a host model must
reproduce each of properties 1–5, and the R2 host test of each property runs
the model in a configuration that does so, with one exception. The "given
up on" clause of property 2 describes broker behaviour that can occur only
when property 4 is not met: `POLL_BOUND` is `TIMEOUT` + 2 × `RESET_TIMEOUT`
*[PR #413]*, so a request that is still pending at the service's poll bound
is already past the broker's deadline. The R2 test of that clause therefore
breaks property 4 on purpose (review 8, RQ8-3). Properties 2 and 4 are not
relaxed. A broker stub or fault-injecting
simulated disk may return a broker result, or answer with a timing, outside
this contract on purpose only where a section 6 row or the P12 test of
property 2 requires it (review 7, RQ7-3; review 8, RQ8-3). These cases
are: the P12 test of property 2 (`p12_2_refused_submission_changes_nothing`
*[PR #413]*), whose "given up on" case keeps a write pending for
`POLL_BOUND` + `SUBMIT_BOUND` + 50 ticks, past the poll bound and beyond the
property-4 deadline, and then delivers its result; N13, where a step-4 or
step-5 write or a barrier flush ends with completion -13; N26 (a)–(c), where
a submission returns -8 for good, as for a reset that is never
acknowledged; N26 (e) and (f) (host tests
`n26e_poll_beyond_bound_is_unknown` and
`n26f_remount_after_expired_poll_bound`), where a completion poll stays
pending past the deadline of property 4 to exercise the service's own poll
bound; N8 (`n8_read_error_on_newer_copy_is_failed_mount`), where a catalog
read ends with completion -16; and N33, where a read ends with completion
-16, a request ends with completion -14 (2.5: not produced by the contract)
or a write or flush ends with completion -13.
The completion codes that the contract gives are per request kind
(review 8, OPT8-2). Property 4 gives -16 to a write or flush and -13 to a
read; a completion code that the contract gives only to the other kind is
outside the contract: -13 on a write or flush (2.5, revision 3) and -16 on
a read. A read completion -13 before the read's deadline is inside the
contract: -13 is the read's own code, and property 4 states when it must
come at the latest, not that it cannot come earlier. The same holds for a
write or flush completion -16 before its deadline. 2.5 answers every one of
these cases fail closed (`Io` -12 for a read -16, `Timeout` -13 for a read
-13, `Unknown` -16 for a write or flush -13 or -16). The M3 broker on
`main` answers a request with -16 only if it is mutating
(`kernel/src/m2/m3/volume.rs`; `Policy` in `crates/virtio-core/src/lib.rs`
records `Completion::Unknown` only for a mutating request) *[main]*. What the
medium stores, and when, is device behaviour, not a broker result, and this
contract does not govern it: damaged, stale or unstored sectors and power
cuts (for example N8, N11, N12, N25, N37 (b)) and writes that reach the
medium after their request was answered (N24; whether that can happen after
the fence is the O9 question, and N24 (b) is recorded, not a pass). Disk
bytes are untrusted in every case. The broker's own
deadline of property 4 (a request past its deadline answered -16 or -13 by
the model itself, followed by the reset) is therefore an opt-in mode of the
host model, because a deadline that is always on would conflict with those
stubs; quarantine and offline (property 5) are device state of the model
that lasts until a simulated reboot. At `f993a1a3` the simulated disk
of PR #413 models neither: results and pending times are injected per
request, and quarantine and offline per submission *[PR #413]*. Its claim to
meet the contract is therefore partial for properties 4 and 5, and the
missing parts are an open R2 item (8.2).

*Time source.* The bound needs a boot-monotonic tick count in the service. On
`main`, generic syscall 3 returns the boot-monotonic PIT tick count (nominally
100 Hz, the unit of `TIMEOUT` and `RESET_TIMEOUT`) only under the feature
`monotonic-clock` (`kernel/src/m2/m3.rs:2366-2370`) *[main]*, which
`native-volume` does not enable (`kernel/Cargo.toml`) *[main]*. Whether
`monotonic-clock` combines with the volume-fixture and
`test-volume-archive-device` features without a `compile_error!` is unchecked;
the R5 test profile must enable it, and this is an R5 precondition. The bound
keeps the single loop from stalling indefinitely: UNBIND waits only for the
device requests already in progress, each limited by these bounds; while a
SEAL catalog failure is being remounted that is at most two submission bounds
(fence, then remount FLUSH) plus the remount's own polls (4.3, N26).
Because the `Policy`
admits one request and barrier 1 has completed, the only request that can be
outstanding at a failure in steps 4–5 is one catalog-sector write or one
flush. The same fence follows a timed-out entry write during APPEND or SEAL
step 3: the writing entry is discarded (2.5) and no request, and so no later
CREATE that reuses its sectors, is admitted before the reset is
acknowledged; under the safe path below, no CREATE is admitted for the rest
of the boot. The reset fences guest DMA and the used ring; whether a write the
device accepted before the reset can still reach the backing medium after the
reset, and so after the remount FLUSH, is not settled by the VIRTIO sections
cited above (5.2.6.2 covers completed writes only) and is *[unproven]* for
QEMU and its host storage stack (section 8, O9). The resolution above holds
only under that assumption; N24 (section 6) states the expected outcome if it
fails. **Safe path while O9 is unproven:** the trigger is any device result
other than success (`Io`, `Timeout` or `Unknown`, or a kernel -8 past the
bound above) on any destination write or flush, catalog or entry: an APPEND
write, the SEAL last-sector write, barrier 1, the step-4 or step-5 write,
barrier 2 or 3 (a FORMAT write or flush failure leads to the stricter
failed-mount state instead, 2.6). A device `Io` is included because a
failed write or flush, like a timed-out one, gives no evidence about which
earlier or in-flight data reached the medium, and a later FLUSH that succeeds
does not show it either. After the trigger the destination accepts no further
catalog or entry write for the rest of the boot (read-only: INFO, LIST,
VERIFY, READ); CREATE, APPEND, SEAL and FORMAT on any lane return `Access` -5
with no device write. These `Access` replies are the state "mounted, safe
path" of the table of 2.2 check 7: they come after checks 1–6, and they apply
only in a mounted instance (for a blank candidate with the latch set, the
defensive rule of 2.2 applies); while a remount is pending no request is received,
and if the remount ends in the failed-mount or `Corrupt` state, that state's
row applies. None of these states writes to the device. A late write therefore cannot overwrite a newer
catalog copy or a newly allocated extent: no extent is reused in that
boot. In particular, if the remount resolves E2 as *not sealed*, E2's sectors
lie at or beyond the committed `next_free` and would be the next CREATE's
extent, but no CREATE, and so no APPEND, is admitted in that boot, so a late
E2 catalog write cannot make E2 listed over another entry's data. Across a
cold boot the late write cannot occur only under the power-cycle assumption
below; reuse of E2's sectors by an export in the next boot is safe only
under that assumption, and O9 remains a blocking open assumption.

**Carrier of the read-only state** *[decided: O13, 24 November 2026; kernel
latch not yet implemented]*. The
read-only state is a latch in the memory of the single service instance. That
latch is sufficient only if no other instance in the same boot can write. The
missing TRANSFER right (2.1, N18) prevents a user process from passing the
grant on, but kernel profile code can create a new grant on a restart, as the
M5 profile does on `main` (2.1). The latch is therefore a sufficient carrier
only under the once-per-boot grant rule of 2.1, which N32 tests. The owner
decided that this is not sufficient: a **kernel write latch** in the
per-device broker (an extension of K5) is required. After the trigger, the
broker refuses every write to the destination scope for the rest of the boot,
whatever process or handle submits it; a service restart, a rebind or a newly
granted block capability on the scope does not clear it, because the latch is
held per device scope in the broker and not per object or handle. The
service's in-memory latch and N32 remain as additional layers. The trigger and
the interface by which the service sets the latch are specified before
implementation; the latch and its own native test are R8 preconditions (8.2).
The latch ends only with the boot. Every destination runner
must pass `-no-reboot` to QEMU, as `tools/run_volume.py:309` already does
*[main]*, so that each guest reboot ends the QEMU process and every boot in the
gates is a new QEMU process; warm reboot of a product, which a guest cannot
distinguish from a cold boot, stays open under O9.

The in-boot resolution, and any in-boot outcome after the safe-path trigger,
is reported to the portal only as "provisional"; the export counts as
confirmed only after a cold boot, whose mount FLUSH and catalog read follow a
device power cycle that no request of the previous boot survives *[design; a
new QEMU process assumed equivalent, unproven]*, and after VERIFY of the
entry. The portal treats `Unknown` as "not sealed" until the export lane's LIST
returns the entry ID; it must not repeat the export blindly and must not report
success. If the remount yields `Corrupt`, the service fails closed as in 2.6.
*[owner decision O17 (b), 23 November 2026]* Before the portal reports an export as
sealed in the same boot, provisionally or not, it also compares the trailer
FNV-64 that the service reports on its own lanes (SEAL `aux`, or the record
that the export lane's LIST returns) with the trailer of the archive it
exported, and treats a mismatch as a failed export ("Stale valid content",
above). The portal holds no control lane (4.1). For a confirmation after a
cold boot the same comparison uses the record of a LIST on a verify lane
bound to that entry; it needs the portal to keep the exported trailer across
the reboot, which is a detail of R7.

**Append-only meaning.** No operation writes a sealed extent, and catalog
copies are written only in the order above. An interrupted new export therefore cannot damage
any previously sealed entry. This resolves the host finding of PR #396 (merged
as `8a7a3594`) that on a
single raw range an interrupted re-export destroys the previous archive as soon
as one differing sector lands.

**Why step 5 mirrors the catalog.** Without step 5, decay of the newer copy
(bytes that read successfully but fail validation; a read error is a failed
mount, 2.6, and never selects the older copy) while the older one stays valid
would make the mount rule fall back to the older copy; the newest sealed entry would then no longer be listed, and its
extent would lie at or beyond the older `next_free`, so a later export could
overwrite it. Silent media decay is outside the crash model; the mirror is a
mitigation subject to the R1 review. After a completed commit both copies are
byte-identical, so decay of one copy loses no entry; decay of both gives
`Corrupt` and fails closed, unless sectors 0–15 all read as zero, which the
mount rule treats as a blank candidate that FORMAT still refuses while entry
data remains (N8). A crash between barrier 2 and barrier 3 leaves
one newer and one older valid copy, which the mount rule handles (the newer
copy extends the older), and the next commit targets the lower-sequence slot
and then mirrors. A read error on the newer copy is a failed mount, not a
fallback to the older one. Decay of the newer copy, however, is not covered
in that window: until the next commit, which under the safe path may not come
in that boot, decay of the newer copy alone would select the older copy and
leave E2 unlisted and overwritable. This window is an **accepted residual** of
the decay mitigation; a mount-time mirror is not specified, because it would
add a third catalog write path at mount, which the safe path and the
failed-mount state forbid in exactly the boots where the window is longest. The catalog write path accepts only the eight
catalog sectors, in the order target slot, barrier, other slot, barrier.

### 2.8 Bounds

16 sealed entries per destination, one writing entry, 160 bytes per APPEND or
READ, at most 4 lanes (one control and three operation lanes; section 4.3).
The largest v1 archive is 525 328 bytes, so APPEND needs at most 3 284
messages; this throughput is acceptable for the bounded v1 volume and is not
measured *[unproven]*.

## 3. Trusted backup portal and destination selection (D2 (a), D4 (a))

### 3.1 Components

- **Backup portal** *[design]*: a system component shipped and signed with the
  OS image. It is the only component whose requests the service accepts on
  an operation lane. The trusted dialog holds usable issuing roots (SEND,
  TRANSFER and REVOKE on each operation lane, section 4.3); the service refuses
  the dialog's own requests on those lanes by the kernel-stamped sender check
  of section 4.3, not by a behavioural promise. It runs `archive::export`, `archive::verify` and `archive::restore` against
  the volume and the destination service.
- **Trusted dialog** *[design]*: a system surface rendered by the shell on a
  trusted surface, not drawable by applications. It holds the control lane of
  the destination service and the lane root handles (section 4.3).
- **Third-party applications** hold no lane, no control handle and no block
  capability on any destination. A later export of a copy to an app goes through
  the export portal as a different permission, not through this service.

### 3.2 What the user sees and confirms (functional)

For export: the source (the user's own document domain, with document and
collection counts), the destination (device label, capacity, free space, the
number of existing backups, and whether it is already a Baryon OS destination),
the statement that the backup is not encrypted and is suitable only for the
user's own local device, and an action button "Back up". An option "Remember
this destination" is shown unchecked; it stores a user preference only and
grants no export right (O1, decided 24 November 2026). Errors are reported in
plain words (destination full, destination not found, destination damaged).

For verify: the selected backup (label, entry ID, size) and the result.

For restore: the list of backups on the destination (label, entry ID, object
count, size), the target domain, and the statement that existing documents are
never overwritten and that conflicts are reported by name. Times are shown only
if a trusted source exists; the catalog stores no wall-clock time and the
dialog never fabricates one (section 8, O5).

For format: only a blank device is offered, with its label and capacity and the
statement that it becomes a dedicated backup destination. Non-blank devices are
not offered (section 8, O2).

### 3.3 Grant representation

The existing permission models on `main` are per-session host models:
`sdk/task-grant-core` binds read, write and export rights to an app, a process
incarnation and an object revision, with durations `UntilClose` and `Session`
only; `services/net-broker` binds grants with an instance, slot and generation
*[main]*. Neither has a persisted grant. This design therefore uses two levels:

1. **Remembered permission** *[design; O1 decided on 24 November 2026]*: a
   persisted record `{owner, principal = backup portal, destination ID,
   operations ⊆ {verify}, revocation generation}` held in protected,
   profile-bound storage of the trusted consent facility. For the unencrypted
   v1 route only VERIFY may be remembered, the option is unchecked by default,
   and export, restore, format and destructive management are never
   remembered; a remembered destination is a preference without rights. It authorizes the
   dialog to issue a new per-operation grant for normal repetition without
   asking again. Revoking it increments the revocation generation and revokes
   every live lane bound to that destination, together with the portal's
   volume-side binding (4.1, 4.2) for each operation on those lanes. The record
   keys on the destination ID, which is not authenticated (5.4): an attacker
   with physical access who presents a clone of the catalog would receive
   automatic plaintext exports under a remembered export permission, which is
   why O1 excludes export from remembered rights. Persisting this record needs a new
   `Duration` and storage in `task-grant-core` or a new store, which is an SDK
   change, not a kernel change. It extends, rather than follows, the existing
   permission model named in D2 (a) (section 8, O1).
2. **Per-operation grant**: a kernel capability node with SEND only on one
   operation lane, plus the service's lane binding `{operation, entry ID, grant
   generation, portal process ID}` (section 4.3). It lives until the operation
   ends or is revoked; ending or revoking it also ends the portal's volume-side
   binding for that operation (4.1).

**Owner principal** *[decided: O12, 24 November 2026; design open]*. Neither the catalog record
(2.6) nor the archive header ([S09-BACKUP-ARCHIVE.md](S09-BACKUP-ARCHIVE.md),
"Bytes") names an owner, yet [PROFILES-AND-USERS.md](PROFILES-AND-USERS.md)
requires that profile A cannot open profile B's catalogue. With the dialog's
LIST of all entries and a restore into "the target domain" (3.2), user B could
restore user A's plaintext backup. The owner decided against disabling
backups in general once a second profile exists: the product binds an owner
per backup registration. Still to be designed, and checked against
[PROFILES-AND-USERS.md](PROFILES-AND-USERS.md), the key contracts and
[ENCRYPTED-OBJECT-STORAGE.md](ENCRYPTED-OBJECT-STORAGE.md) before R7: the
catalog version that carries the binding, the trusted identity it names and
who attests it, the access checks of LIST, RESTORE and VERIFY per profile, and
the recovery route when the owner profile is lost. An owner field in the
catalog is an access-control input, not evidence of encryption or of
confidentiality against a party that reads the device directly. R7 and the R8
activation wait for that design; the test-only stages of R5 and R6 run in a
single-profile fixture and do not depend on it.

## 4. Operation-specific capabilities (D3)

### 4.1 Rights per operation

| Operation | Destination service (service-enforced lane binding) | Kernel node held by the portal | Source volume |
|---|---|---|---|
| Export | INFO, CREATE once, APPEND, SEAL, ABORT on one new entry; LIST restricted to its own entry (zero records until sealed, 2.3; after an `Unknown` reply, the resolution of 2.7) | SEND on one lane; no RECV, DERIVE, TRANSFER, REVOKE | domain-bound volume endpoint, read-only binding *[needs volume change, 4.2]* |
| Verify | INFO, LIST (bound entry), VERIFY on one sealed entry | SEND on one lane | none |
| Restore | INFO, LIST (bound entry), READ on one sealed entry | SEND on one lane | domain-bound volume endpoint, create-only binding *[needs volume change, 4.2]* |
| Selection (dialog) | INFO, LIST (all), ABORT, BIND, UNBIND, format of a blank scope | control lane SEND | none |

The portal never holds a block capability on the destination device or on the
source volume. No portal handle carries DERIVE or TRANSFER, so it cannot pass a
lane on *[main mechanism: `capability-core` `subset`/`require`]*; on `main` the
portal's existing volume handles are SEND on the domain ingress and RECV on its
reply endpoint, and its archive block handle carries no TRANSFER
(`volume.rs:594-603`, `:713-724`) *[main]*. The read-only and create-only
volume bindings exist only for the duration of one operation: when its lane is
unbound or revoked, or the remembered permission behind it is revoked (3.3),
the volume-side binding for that operation ends with it (mechanism specified
with R4).

### 4.2 Source volume side

Export reads the domain through the normal volume client. Today the whole
volume service is made read-only during the fixture export
(`service_read_only()` in `kernel/src/m2/m3/volume.rs`) *[main, fixture only]*.
A product needs a per-endpoint **read-only binding** in the volume service that
refuses every mutating operation with `Access` before state is consulted.

Restore writes through the normal volume client. The non-overwrite rule is
enforced today by `archive::restore` in the caller, which is library code linked
into the portal; the volume service itself accepts any valid BEGIN/COMMIT, META,
DELETE, RENAME or PURGE from the domain's endpoint *[main]*. Least privilege
therefore needs a **create-only binding** in the volume service: commits only
into an empty or tombstoned slot, META and COLLECTION only on objects and
collections created through that binding, and DELETE, RENAME, PURGE, RESTORE
and TRANSACT refused with `Access`. Both bindings are volume-service changes
behind the existing kernel-bound endpoint; they change no kernel ABI, but they
change the volume wire's authorization behaviour and need their own
specification and host tests (route step R4). Until then, restore's
non-overwrite guarantee rests on trusted portal code, and this document does not
claim service enforcement.

### 4.3 Mapping onto capability-core and the kernel

Facts on `main`: user syscalls exist for close (0x10), derive (0x11), revoke
(0x12) and send/recv with up to two transferred capabilities (0x20/0x21) in
`kernel/src/m2/m3/m4.rs`; endpoint and block-object creation exist only in
kernel manifest code; a process owns at most `MAX_OWNED_ENDPOINTS` = 4
endpoints; `revoke` marks every strict descendant of the caller's node revoked
and keeps the caller's own node; a revoked node resolves to `Error::Revoked`;
revoking a node also removes queued messages that carry capabilities derived
from it, but nothing shows that it removes plain data messages already queued
by the revoked holder *[main, `capability-core` tests]*.

Design with existing primitives only:
- The manifest creates four endpoints owned by the destination service: one
  control lane and three operation lanes. The service keeps RECV on each. The
  dialog receives SEND on the control lane and SEND|TRANSFER|REVOKE on each
  operation lane. Because `send` requires the parent node to hold TRANSFER and
  the transferred rights (`subset`), these roots are usable: the dialog can
  itself send on an operation lane *[main, `Engine::send`]*. The service
  therefore binds the caller by lane, as the volume service binds a domain by
  its ingress endpoint *[main pattern]*, **and** by sender: the kernel stamps
  `sender_pid` into every queued message (message offset 0) *[main]*, and BIND
  records in the lane binding the portal's process ID, fixed for the boot by
  the manifest that starts the portal *[design]*. The same manifest writes that
  process ID into the destination service's start context before the service
  runs, so the service knows it before any BIND, including for reply
  suppression on unbound lanes; N18 checks the value against the portal's
  actual process ID *[design]*. A request on a bound
  operation lane whose `sender_pid` differs is refused with `Access` with no
  state change and no device I/O, and gets no reply (check 4 and Replies of
  2.2; N22). The service refuses any
  well-formed request on an unbound lane with `Access` (check 3). That process
  IDs are not reused within a boot is assumed
  *[unproven]*. Placing the issuing roots in the dialog
  rather than in the portal is this document's interpretation of D4 (a)
  ("a trusted OS backup portal holds destination capabilities"): the portal
  holds every capability that performs an operation, the dialog only the
  authority to issue and revoke them. The project owner may require the roots
  in the portal instead (section 8, O7).
- Replies: as for the volume, the portal owns its own reply endpoint, keeps
  RECV on it and the manifest delegates SEND on it to the destination service;
  operation-lane replies go there. The dialog owns a control-reply endpoint,
  keeps RECV on it and the manifest delegates SEND on it to the destination
  service; control-lane replies go there, so the dialog receives the BIND and
  UNBIND replies it waits for. The dialog owns no other endpoint, so this uses
  one of its `MAX_OWNED_ENDPOINTS` = 4 *[main limit]*. The dialog reaches the
  portal through one further portal-owned endpoint, on which it transfers lane
  handles. All these handles are included in the manifest counting of section
  5.1. With an IPC transfer the new node is a child of the
  sender's node *[main, `Engine::send`]*, which is what makes the dialog's
  `revoke` reach the portal's lane handle.
- To grant: the dialog sends BIND (lane, operation, entry, new grant generation,
  above the maximum that INFO reports for that lane, 2.3) on the control lane,
  waits for `Ok` on its control-reply endpoint, then transfers SEND on that lane to the
  portal, and tells the portal the grant generation, which the portal places in
  every request's `epoch` field.
- To revoke: the dialog first sends UNBIND on the control lane and waits for
  `Ok` on its control-reply endpoint, so that any request already queued on the
  lane (including a SEAL) is answered with `Access` and cannot commit; if the
  reply does not arrive (for example because it was dropped, 2.2), the dialog
  repeats UNBIND, which is idempotent (2.3), and does not revoke before a
  reply. A reply of `Io` (failed-mount state) or `Corrupt` (2.6) also permits
  the revoke, because in those states the service writes nothing and no queued
  request can commit. A kernel `Revoked` -10 or `Closed` -9 returned by the UNBIND
  `send` itself also permits it. When the service exits, `Engine::exit`
  revokes every node that descends from the service's nodes, including the
  dialog's delegated control and lane nodes, so `resolve` returns `Revoked`
  before it reaches the endpoint; `Closed` covers an endpoint that `maintain`
  has already reclaimed. In both cases no queued request can commit and no
  reply will come *[main, `capability-core` `Engine::exit` lib.rs:1588-1628,
  `resolve` lib.rs:1690-1701]*. N6 includes this case. It then calls `revoke`
  on its lane node, which revokes the portal's child node and keeps the
  dialog's node for reuse. The service serves the control lane and the
  operation lanes from one loop, one request at a time, so UNBIND is ordered
  against every request it has not yet served. A SEAL already served before
  UNBIND has completed and stays sealed; revocation never undoes a completed
  operation.

Needs explicit approval, because each is a kernel or shared-ABI change, and is
not assumed here:
- **K1** Runtime device ownership (hot-plugged or user-attached destination).
  Block objects are manifest-only, so v1 supports only a destination present at
  boot.
- **K2** More than three concurrent operation lanes, which would need endpoint
  badges or a higher owned-endpoint quota.
- **K3** A kernel-enforced rule that the portal can never hold the volume's or
  the destination's block object. Today this is shown by manifest counting
  (`ARCHIVE_ESCAPE`) only.
- **K4** A destination block device in a production profile at all: today
  `device_call` refuses `scope.device != 0` with `Access` *[main]*, and device 1
  exists only under the test feature `test-volume-archive-device`
  (`volume.rs:877-881`) *[main]*. K4 covers production profiles only; the
  test-only R5 profile additionally needs the owner to amend
  ADR-S09-DUALDISK-20261119 B3 and B5 (O4 (b)).
- **K5** Per-device broker state: the virtio `Policy`, `resetting`,
  `reset_deadline`, `error`, `active` and the data buffer kept per block
  device (today global, `volume.rs:122-131` *[main]*), so that a destination
  fault cannot fail the source volume closed (section 5.2) and a kernel -8 on
  disk 1 means only disk 1's own reset (2.7). A per-device `Policy` alone does
  not give this, because `resetting`, `error` and `active` gate every
  submission (`volume.rs:964`). If the owner chooses it under O13, K5 also
  carries a kernel write latch for the destination scope. K5 must meet the
  broker contract of 2.7; one delta beyond per-device state (review 9,
  OPT9-3): a further completion poll after a read ended by a reset returns
  -1 (property 3), whereas the M3 broker on `main` returns -14 while the
  reset runs and, after a quarantine, until reboot or until the owning
  process exits (2.7, property 3; read, not executed).

## 5. Isolation between source and backup devices

### 5.1 Ownership rule

The volume service holds a block capability only on the source volume scope; the
destination service holds one only on the destination scope; the portal holds
neither. The volume service never holds a destination lane or block handle, and
the destination service never holds a volume endpoint or block handle. At
manifest time this is checked by counting, as `archive_grant` does today
*[main]*: one new block object, node and handle per grant, in the intended
process, with every other process's handle count unchanged and distinct object
keys. The same counting covers the endpoint handles of section 4.3. The
destination service holds exactly: one block handle on the destination scope
(READ|WRITE|FLUSH, 2.1), four lane RECV handles, one SEND handle on the
portal's reply endpoint, one SEND handle on the dialog's control-reply endpoint
and one handle on its own `Entropy` object (one of `MAX_ENTROPY_OBJECTS` = 4)
for FORMAT *[main limit]*; in the R5 test profile with the deterministic test
destination ID (section 7, O4 (c)) it holds no `Entropy` handle, and N18
asserts that. The dialog holds exactly: one control-lane SEND,
three lane roots (SEND|TRANSFER|REVOKE), one SEND on the portal's intake
endpoint and RECV on its own control-reply endpoint. The portal holds its reply
and intake RECV handles, its volume handles (4.1) and, at most, three lane SEND
handles at run time. Counting alone does not detect excess rights, so N18 also
probes the exact rights of each handle.

### 5.2 DMA and lanes

The kernel virtio-blk driver performs DMA; userspace never does. PR #396
(merged as `8a7a3594`) gives the second device its own DMA pages *[main]*, disjoint from device 0, and an
unmapped lane-1 MMIO window after review. It also records a fixture limit: both
lanes share one virtio `Policy`, so a failed device-1 reset fails both lanes
closed (the faulting lane goes offline, the other receives kernel -8
indefinitely, 2.7). That is safe but couples availability: a faulty backup device stops the
source volume. The product requires per-device broker state (K5: the
`Policy` and the `resetting`, `reset_deadline`, `error`, `active` and data
fields), so that a destination fault yields `Io` or `NoDevice` on the
destination only. This is a kernel change (K5) inside the virtio-blk path, not
an ABI change.

### 5.3 Device identity

A destination is identified by its catalog destination ID. A grant names the
destination ID; if the mounted catalog holds another ID, BIND returns
`NoDevice` -14 and performs no I/O. If no device is present, the mount FLUSH
receives kernel -14 and the service is in the failed-mount state (2.6): INFO
reports `NoDevice` -14, every other request that passes checks 1–6 of 2.2
returns `Io` -12. A submission refused with kernel -14 (offline) or -13
(quarantined) on any destination request after the boot mount is a device
failure: the request gets `Io` -12, and a completion result follows the
"Device results" table of 2.5 (revision 3; revision 2 gave `NoDevice` -14 for a -14 here, which
contradicted the rest of this section), and on a write or flush of an export
the 2.7 safe-path trigger applies (a FORMAT write or flush leads to the
failed-mount state instead, 2.5). A non-blank layout that is not a valid catalog is `Corrupt`
(2.6), and every request except INFO returns `Corrupt` -18. A blank candidate
has no destination ID; BIND there returns `NotFound` -19 at check 7 (2.2).
`NoDevice` is thus a reply status only of BIND in the mounted state and a
status reported by INFO. The current
fixture's "wrong device" check (portal granted the device-0 range, verify gives
`Corrupt`) *[main]* becomes a `NoDevice` refusal at BIND, before any lane is
bound.

### 5.4 Limits

No IOMMU model: QEMU's virtio-blk device is trusted to follow the protocol and
DMA only into granted pages. A malicious device is out of scope. The destination
ID and FNV-64 are not authentication: an attacker with physical access can swap
or roll back the image undetected. The step-3 content digest and the portal's
trailer comparison (2.7, *[owner decision O17 (a), (b), 23 November 2026]*) detect accidental
stale content only, not a device that misbehaves on purpose. v1 archives are plaintext personal data and
belong only on the user's own local device (D5 (a)).

## 6. Negative test matrix

Layers: **K** kernel capability or block broker, **S** destination service,
**V** volume service, **P** portal code, **D** trusted dialog, **H** host
oracle. Levels: **host** = host Rust test with a simulated disk that models a
volatile write cache (as the `SimDisk` of PR #396, *[main]*); **QEMU** = native
stage on disposable images. A native "cut" kills the QEMU process on the host;
with `cache=writeback` (`tools/run_volume.py:314`, `:321`) *[main]* data that
QEMU has handed to the host page cache survives the kill, so a QEMU cut is a
**process kill without cache loss** and cannot detect a missing barrier.
Flush-order and cache-loss acceptance is therefore host-only (`SimDisk`, R2);
the native cut stages check recovery and the recorded flush order (section 7).
Whether a device reset keeps or drops the volatile write cache is not known
for QEMU (O9), so the host model must not fix one answer (revision 3): rows
whose outcome passes through a device reset (N13, N24, N26, the N33 case
of a read timeout while another lane holds unflushed APPEND writes, and
N37 (a)) run under
both reset models, the volatile cache kept across the reset and the volatile
cache dropped at the reset, and the expected outcome holds under both (where
a row states one outcome per model, the invariant it names holds under both). The
host simulated disk must also meet the broker contract of 2.7 (K5).
Rows added or changed in revisions 1–5 name, after "Kills:", the mutation
that must make the test fail and how it is observed. In host tests the status
of a request is the value returned by the service's request handler, also
where 2.2 sends no reply. Equivalent mutants, which change no observable
outcome, are listed in their row and not tested. The equivalences added in
revision 4 come from the O14/O16 code round on PR #413: R14 and X6 survived as
equivalent mutants in its code reviews; B6 rests on this text's reading of
the call sites of `mount` and of the function that sets the safe-path latch at
`e52cb865` (2.2, confirmed by review 6); A10 and A11 are reported by the
round's implementer and reasoned in this text. The equivalences added in
revision 5 come from the O17 (a)/O18 code round on PR #413 (`f993a1a3`):
M8 survived as an equivalent mutant in its code review
(CODE-REVIEW-O17-O18), and M7 survived there as a mutant that is equivalent
in every build without `test-hooks` (N35); the M names in rows N35 and N37 are those of that
review, not items of this document.

| # | Case | Precondition | Action | Expected (layer: code) | Enforced by | Level |
|---|---|---|---|---|---|---|
| N1 | Unauthorized device | app or portal without a destination block handle | block read/write with own or forged handle | K: `BadHandle` -4 or `Access` -5; counted in `app_device_denials` | K | QEMU |
| N2 | Scope escape, device | destination service | block write at scope end | K: `Access` -5; host sees no byte past the scope | K | QEMU |
| N3 | Scope escape, entry | restore lane bound to entry E | READ entry ≠ E; READ past E's length; READ straddling the end; READ with `offset + aux` overflowing u64; APPEND on a verify lane | S: `Access` -5; `Range` -3; `Range` -3 with no data; `Range` -3 with no data; `Access` -5. Kills: a READ bound check that compares only `offset` with the length, or uses wrapping addition | S | host + QEMU |
| N4 | Scope escape, sealed extent | export writing after sealed entries; separately, a crafted catalog with zero entries and `next_free` < 16; a crafted catalog with zero entries and next entry ID 0 | crafted write path below `next_free` and below sector 16 (mutation test); mount of each crafted catalog | write refused; sealed extents and sectors 0–15 byte-identical; each crafted copy is invalid | S | host |
| N5 | Revoked before operation | dialog revoked the lane | portal `send` | K: `Revoked` -10 (syscall return) | K | host + QEMU |
| N6 | Revoked during export | 50 % appended; (b) the service exits instead | dialog sends UNBIND, then revokes; (b) the dialog's UNBIND `send` returns `Revoked` -10 (4.3) | K: `Revoked` -10 on next send; S discards writing entry; LIST unchanged; earlier entries verify; (b) the dialog takes its revoke step and the revoke succeeds or is a no-op (host `Engine` case). Kills (b): a dialog that permits the revoke only on `Closed` -9 and repeats UNBIND forever | K + S | host + QEMU |
| N7 | Corrupted entry | one byte flipped in a sealed extent on the host image | VERIFY; restore flow | S: `Corrupt` -18; P: `archive::verify` `Corrupt`, nothing restored | S + P | host + QEMU |
| N8 | Corrupted catalog; catalog read error | one copy damaged; then both damaged; then both copies zeroed with sectors 8–15 zero and sealed entries at sector ≥ 16; separately, A = sequence n+1 {E1, E2} and B = sequence n {E1} (crash between barriers 2 and 3) with the simulated disk failing the read of one sector of A (`Io`, then `Timeout`, then `Unknown`); and the same with a read error on one of sectors 8–15 while 0–7 read as zero | mount; control FORMAT; for the read-error cases also INFO, control LIST and control BIND | one copy: the other, identical copy used, LIST unchanged; both damaged (a non-zero sector in 0–15): S `Corrupt`, INFO answered with `Corrupt`, every other request `Corrupt` -18, no write, no format; both zeroed: blank candidate per the mount rule (INFO status `NotFound` -19, destination ID zero), control LIST and BIND `NotFound` -19, no write, FORMAT `Access` -5 after the full-scope read finds the entry data (as N23); read error on A: failed-mount state (2.6), INFO reports `Io`, LIST, BIND and FORMAT `Io` -12, no device write, B never selected, E2's sectors never written; read error in 8–15: failed-mount state, never a blank candidate. Kills: a mount that treats a copy with a read error as invalid (selects B), and one that treats an unreadable sector as zero; (revision 2) a blank state that answers INFO with `Ok` or BIND with anything but `NotFound` -19 | S | host (read-error cases) + QEMU |
| N9 | Incomplete write | appended ≠ declared | SEAL | S: `Invalid` -1; nothing listed | S | host |
| N10 | Non-archive bytes | appended bytes fail `verify` | SEAL | S: `Corrupt` -18; nothing listed; `next_free` unchanged after reboot | S | host + QEMU |
| N11 | Power cut during APPEND | one sealed entry E1; export of E2 running | host: cut after k destination writes with cache loss; QEMU: host process kill after k destination writes | after cold boot: LIST = {E1}; E1 verifies byte-identical | S (2.7) + H | host (cache loss) + QEMU (process kill, no cache loss) |
| N12 | Power cut during SEAL | cut after barrier 1, during the catalog write, after barrier 2 | host: every write prefix with cache loss; QEMU: host process kill at each point | LIST = {E1} or {E1, E2}; a listed E2 verifies; never a listed E2 that fails; QEMU additionally: the `DEST:IO:FLUSHES` count and flush order recorded at the cut marker match 2.7; both are counted by the kernel broker per destination object key, and the cut marker is emitted by the kernel after k destination writes, independent of the flush count (section 7). Kills (revision 1, QEMU): a SEAL without barrier 1 or barrier 2 (detected by the flush count and order, not by recovery); (revision 2) a service that skips the flush call, which a service-side counter could hide but the kernel count cannot | S + H | host (cache loss) + QEMU (process kill, no cache loss) |
| N13 | `Unknown` at SEAL | the step-4 or step-5 write, or the flush at barrier 2 or 3, gives a device `Io`, `Timeout` or `Unknown` (broker timeout, 2.7), with and without the new copy durable; bound lanes: export lane X (the SEAL), a second export lane Y bound before the SEAL with no CREATE yet, and a third lane bound for verify in one run and for restore in a second run (2.8 allows three operation lanes; revision 2 named four); then, after the completion fence (N24), the remount FLUSH succeeds, or fails | SEAL on X; then on X: LIST, CREATE, APPEND, SEAL, ABORT; on Y: CREATE; on the verify and restore lanes: CREATE, and LIST, VERIFY or READ; on the control lane: INFO, LIST, BIND | S: SEAL `Unknown` -16 for every device result, remount, bindings kept. Remount FLUSH succeeds: X's LIST returns E2's record if the remounted catalog lists E2 (P reports provisionally sealed, confirmed only after cold boot and VERIFY, 2.7), else zero records (P reports not sealed); X's CREATE, APPEND, SEAL and ABORT `Access` -5 (check 6: binding limited to INFO and LIST); Y's CREATE `Access` -5 (check 7, safe path) for the rest of the boot; the verify and restore lanes' CREATE `Access` -5 (check 6), their LIST, VERIFY and READ run; control INFO, LIST and BIND run. Remount FLUSH fails (failed-mount state): X's LIST `Unknown` -16 with zero records; X's CREATE, APPEND, SEAL and ABORT `Access` -5 (check 6); Y's CREATE `Io` -12; the verify and restore lanes' CREATE `Access` -5 (check 6), their LIST, VERIFY and READ `Io` -12; control INFO status `Io` -12, control LIST and BIND `Io` -12; no device write, no FLUSH retry, state unchanged for the rest of the boot, binding never resolved as sealed (P reports not confirmed); in the next boot the provisional ID is void and only that boot's catalog decides. No repeated export in either case. Kills (revision 1): a safe-path trigger that fires only on `Unknown`, so that a device `Io` at barrier 2 or 3 readmits Y's CREATE after a successful remount (Y's CREATE runs). Kills (revision 2): a validator that applies check 7 before check 6 (X's CREATE and the verify lane's CREATE return `Io` -12 in the failed-mount state); a binding limit applied only after resolution (X's APPEND reaches check 7) | S + P | host |
| N14 | Wrong, foreign or absent device | grant names ID X; device holds valid catalog Y (natively an image whose catalog the harness writes with an ID other than the deterministic test ID, section 7); separately, a non-Baryon, non-blank layout; separately, no device (broker -14 to the mount FLUSH) | BIND naming X; INFO; a well-formed request on an unbound lane; control FORMAT | catalog Y: BIND `NoDevice` -14, no I/O, lane stays unbound, its requests `Access` -5; foreign layout: `Corrupt` mount, INFO reports `Corrupt`, BIND and FORMAT `Corrupt` -18, format not offered by D, no write; no device: failed-mount state, INFO reports `NoDevice` -14, BIND and FORMAT `Io` -12. Kills (revision 1): a check order that returns `NoDevice` from a request other than BIND, or answers FORMAT in the `Corrupt` state with anything but `Corrupt` | S + D | host + QEMU |
| N15 | Destination full | free space < archive | export | S: `Full` -17 at APPEND; nothing sealed; earlier entries intact; catalog sequence unchanged | S | host + QEMU |
| N16 | Third-party raw access | app process in the profile | forged lane, control and block handles; IPC to service without a handle | K: `BadHandle` -4; counted; service receives nothing | K | QEMU |
| N17 | Stale grant replay and header rules | lane rebound with generation g+1; lane unbound; control lane | request with epoch g; duplicated APPEND offset; second SEAL; any well-formed request on an unbound lane; control request with epoch ≠ 0 or non-increasing `request_id`; CREATE with `length` ≠ `name_length`; re-BIND of a lane with a generation equal to (and separately lower than) its highest accepted one while an APPEND of the old binding is still queued; a request rejected by checks 1–6 followed by one with the next `request_id`; in the safe path a CREATE followed by a request with the same `request_id` | S: `Stale` -10 (reply status); no extra bytes; second SEAL `Access` -5 (check 6), no second entry; unbound lane `Access` -5; control `Stale` -10; CREATE `Invalid` -1; re-BIND `Stale` -10 with no state change, lane stays unbound, the queued APPEND gets `Access` -5 and writes nothing, INFO reports the unchanged maximum; the request after one rejected by checks 1–6 is accepted (such a rejection does not advance the high-water mark); in the safe path, a CREATE refused by check 7 (`Access` -5) advances it, so a request that reuses its `request_id` returns `Stale` -10. Kills (revision 1): a BIND that accepts a generation equal to the lane maximum, and a validator that advances the high-water mark on a request rejected by checks 1–6. Kills (revision 2): a validator that does not advance the mark on a check-7 refusal (the reused `request_id` is accepted) | S | host + QEMU |
| N18 | Authority escape at manifest | profile start | count objects, nodes and handles; probe object keys; probe exact rights with `check_block` and `check_endpoint`, as `ARCHIVE_READONLY_RIGHTS` does (`volume.rs:743-748`); compare the destination scope with every other block scope by `check_block` (precedent: `volume.rs:749-753`, which compares the volume service's own handle, 2.1) | the destination service, dialog and portal hold exactly the 5.1 handles (in the R5 profile with the deterministic test destination ID, no `Entropy` handle); the destination block handle passes READ, WRITE and FLUSH and refuses DERIVE, TRANSFER and REVOKE; the dialog's control-lane handle is SEND only (the portal holds no lane handle at profile start; R5 checks at run time that each lane handle transferred to it is SEND only); the destination scope is disjoint from every other block scope, and under the proposed O11 rule on another device; any extra handle or right in volume service, destination service, dialog, portal or app: manifest fails (`ARCHIVE_ESCAPE`-style). Kills (revision 1): a manifest that grants `BLOCK_RIGHTS` (with DERIVE, TRANSFER, REVOKE) at an unchanged count; a destination scope overlapping the device-0 source scope; under O11, a disjoint scope on device 0; (revision 2) an `Entropy` handle granted to the destination service in the deterministic-ID R5 profile (count grows) | K manifest | QEMU |
| N19 | Volume read-only binding | export lane plus RO volume endpoint | portal sends BEGIN/META/DELETE to the volume | V: `Access` -5; volume unchanged | V | host (after R4) |
| N20 | Volume create-only binding | restore into a domain with occupied slots | portal sends a commit into an occupied slot, DELETE, RENAME | V: `Access` -5; existing objects keep their revision | V | host (after R4) |
| N21 | Second writer | export lane writing | CREATE on another lane | S: `Busy` -21 | S | host |
| N22 | Dialog uses a bound lane | lane bound to the portal | dialog sends INFO, APPEND or READ on that lane with its own root | S: refused at check 4 (handler status `Access` -5, `sender_pid` ≠ bound portal); no reply reaches the portal's reply endpoint (2.2 Replies; natively the kernel-reported `DEST:PORTAL:REPLYQ=` is unchanged); `DEST:DENIALS=` grows by one (service report, supporting only); no I/O. Kills: a service that skips the sender check (the request runs or is answered `Ok`); (revision 2) a service that replies to the portal for a non-portal sender (the portal's reply queue grows, observed by the portal or the host `Engine`) | S | host + QEMU |
| N23 | Data beyond the catalog area | catalog sectors 0–15 zero, non-zero data at sector ≥ 16 | control FORMAT | S: `Access` -5; no sector written | S | host |
| N24 | Late-completing write (2.7 completion fence) | simulated disk that completes a timed-out catalog-sector write late: (a) before the fence (reset acknowledgement), (b) after the remount FLUSH, (c) after a remount that resolved E2 as *not sealed*; (d) an entry-sector write during APPEND that times out (`Unknown`) and completes late, and separately one that fails with a device `Io` while its data still reaches the medium | SEAL with the step-4 write timing out (`Unknown`); CREATE on another lane and LIST on the export lane sent before the fence; (c) CREATE and APPEND on another lane after the resolution, before the late write lands; (d) CREATE and APPEND on another lane after the acknowledged reset; then remount and cold boot | (a) before the fence no remount FLUSH, no resolution, and neither request is received (remount pending, 2.2 check 7); after the remount CREATE `Access` -5 (safe path) and the export-lane LIST follows the resolution, which matches the cold-boot catalog; (b) outside the assumed device model (O9), recorded, not a pass: no sealed extent written, never a listed entry that fails VERIFY, LIST after cold boot {E1} or {E1, E2} (E2 listed although P reported not sealed: safe direction) or S `Corrupt` fail closed with E1 byte-identical; mutation that issues the remount FLUSH or receives a request before the fence must fail (a) (observed in the simulated disk's event order: remount FLUSH or a reply before the late completion); (c) CREATE `Access` -5 and APPEND `Access` -5 in that boot, no device write, E2's sectors not allocated again, so the late E2 write lands on no other entry's data; mutation that admits CREATE after the resolution in the same boot must fail (c); (d) writing entry discarded (APPEND replies `Unknown` -16, respectively `Io` -12), CREATE and APPEND `Access` -5 for the rest of the boot, the late entry write lands only in unallocated sectors; mutation that admits CREATE after the reset in the same boot must fail (d); Kills (revision 1): a safe-path trigger that ignores a device `Io` on an entry write | S + K fence | host |
| N25 | Power cut during FORMAT | blank scope | FORMAT with a cut at every write prefix of copy A, with cache loss (host), before copy B | no non-zero sector of copy A durable (a zero-entry copy A is non-zero only in sectors 0 and 3, so a durable subset of sectors 1–2 alone counts here): blank candidate, FORMAT `Ok` again; some but not all non-zero sectors of copy A durable, or copy A otherwise invalid: S `Corrupt`, INFO reports `Corrupt`, every other request `Corrupt` -18, no write, no format; after the owner re-zeroes the whole scope (2.6), the next boot's instance mounts a blank candidate and FORMAT returns `Ok`; copy A valid (sectors 0 and 3 durable): mounted with one valid copy (sequence 1, zero entries); (revision 2) separately, the simulated disk fails a FORMAT write or flush with `Io` and, in a second run, `Unknown`: FORMAT replies that code, the instance enters the failed-mount state (INFO reports `Io`, a repeated FORMAT `Io` -12), and no further device request follows. Kills (revision 2): a FORMAT failure that leaves the instance a blank candidate, so that a repeated FORMAT writes again (observed by the simulated disk's write counter) | S | host |
| N26 | Kernel -8 (2.7) | broker stub that returns kernel -8 on submission forever: (a) to the mount FLUSH; (b) to the barrier-2 flush and every later request; (c) to an APPEND write; separately (d) -8 on submission for `RESET_TIMEOUT` ticks (disk 1's own reset under K5) and then success, and a completion poll that returns -8 (pending) for `TIMEOUT` − 1 ticks and then success; (e) a completion poll that returns -8 for longer than `TIMEOUT` + 2 × `RESET_TIMEOUT` on a write; (f) (revision 5, review 7 RQ7-3) a completion poll that returns -8 for longer than the poll bound on the barrier-2 flush, after which the stub keeps that request (property 2 of 2.7), so that the remount FLUSH receives -8 on submission | mount; SEAL; APPEND; UNBIND on the control lane during the wait | (a) after one bound (2 × `RESET_TIMEOUT`): failed-mount state, INFO reports `Io`; (b) after one bound SEAL `Unknown` -16, then the remount FLUSH also exceeds the bound, failed-mount state; (c) after one bound APPEND `Io` -12, writing entry discarded, safe path (CREATE `Access` -5 for the boot); (d) both requests complete normally, no `Io`, no safe path; per reset model (revision 4), where the -8 interval is disk 1's own reset during the barrier-1 submission of a SEAL: cache kept, SEAL `Ok`; cache dropped, the reset loses every completed, unflushed write of the entry, so SEAL replies `Corrupt` -18 at the step-3 readback, nothing listed, no catalog write, provided the free sectors at the extent do not hold a valid archive of the same length (a freshly formatted image; with such a stale archive the step-3 content check refuses the extent, 2.7 "Stale valid content", as N37 (a)); under both, never a listed entry that fails VERIFY; (e) the write counts as `Unknown` -16 and starts the safe path; (f) SEAL `Unknown` -16 within the poll bound plus the submission bound, the remount FLUSH refused with -8 until the submission bound, failed-mount state with the safe-path latch set, INFO reports `Io`, export-lane LIST `Unknown` -16 with zero records, no device write follows, under both reset models (host test `n26f_remount_after_expired_poll_bound` *[PR #413]*). UNBIND is answered after at most one bound in (a) and (c) and at most two bounds in (b) (the stub admits no request, so no poll occurs; with real polls the limit of 2.7 "Time source" applies), `Io` -12 in (a) and (b) (failed-mount state) and `Ok` in (c), and no device write follows. Kills: a resubmission loop without the bound (the test times out and UNBIND is never served); (revision 2) a bound shorter than one legitimate reset, or the submission bound applied to the pending poll ((d) reports `Io`); a poll loop without a bound ((e) never ends); (revision 5) a remount that treats the refused FLUSH as done, or a submission loop without a bound after an expired poll ((f)) | S | host |
| N27 | Transferred capability on a request (2.2 check 1) | dialog double sends one request with one transferred handle on the control lane, and one on a bound operation lane | any request with `cap_count` 1 | S: handler status `Invalid` -1 (control lane: replied to the dialog; operation lane: no reply, because the sender is not the portal, 2.2 Replies); the service's handle count after the request equals the count before; no state change; `request_id` high-water mark unchanged. Kills: a service that keeps the received handle (count grows) or parses the request before closing it | S + K | host (capability-core `Engine`) + QEMU (`DEST:HANDLES=`) |
| N28 | Check order (2.2) | requests that violate two checks of 2.2 at once, for every pair of checks 1–7 (table-driven) | send each request | S: the handler status of the earlier check; no state change; no device I/O; high-water mark unchanged. Kills (narrowed in revision 2): a swap of two checks whose outcomes differ in status, reply, handle count or high-water mark: check 1 with any later check (handle count, and the status for checks 3–7), check 2 with checks 3–7 (`Invalid` against `Access`, `Stale` or a state code), check 5 with checks 3, 4, 6 and 7 (`Stale` against `Access` or a state code), and check 6 with check 7 in the failed-mount, `Corrupt` and blank states (as in N13). Equivalent mutants, which no test can detect and which are listed rather than tested: swaps among checks 3, 4 and 6 and the safe-path row of check 7, which all return `Access` -5 with no state change (checks 3 and 4 cannot fail together, because an unbound lane has no bound sender) | S | host |
| N29 | Pre-write catalog check (2.7 step 4) | test-only hook that alters the encoded copy, present only under a test-only build feature (8.2, R5 precondition): wrong checksum; `next_free` lower (refused by the validator's contiguity rule, 2.6); sequence + 2; one record dropped; (revision 3) a wrong sealing sequence on the new record, including one that the 2.6 validity rules accept (committed copy with sealing sequences 2 and 5 under sequence 9, new record sealed at 7); separately, without the hook, a committed catalog with sequence or next entry ID at the u64 maximum (checked arithmetic, 2.7 step 4) | SEAL; for the u64-maximum case, SEAL repeated with a second export on a new export binding (the first binding's CREATE was answered `Ok` and is used up) | hook cases: S: `Corrupt` -18 for that SEAL; no catalog sector written (write counter unchanged after barrier 1); writing entry discarded; service not in the `Corrupt` mount state (INFO without `Corrupt`, LIST unchanged, a later export seals). u64-maximum case (review 4 RQ-2): every SEAL `Corrupt` -18; no catalog sector written; each writing entry discarded; service not in the `Corrupt` mount state; INFO and LIST unchanged; no later export seals. Kills: a step 4 that writes without running the validator and the extension check; an extension check without the sealing-sequence clause (the wrong-sealing-sequence case, with a crafted committed copy whose last sealing sequence is below its sequence, writes); an increment that wraps or panics. Equivalent mutant (revision 4; B4 in the O14/O16 code round, R14 in its code review, where it survived as equivalent), listed, not tested: an extension check without its record-count clause ("exactly one"), which the sealing-sequence clause implies for every copy that passes the validator (2.7 step 4) | S | host |
| N30 | Mixed catalog copies (2.6) | two valid copies with different sequences where the higher does not extend the lower: a record missing; a record changed; next entry ID lower (revision 3: replaces "`next_free` lower", which two valid copies with a common record prefix cannot show, 2.6); another destination ID; separately the legitimate case (higher one record ahead); and two valid copies with equal sequences, one a record ahead of the other (`Corrupt`) | mount; INFO | non-extending pair: S `Corrupt`, INFO reports `Corrupt`, no write; legitimate pair: the higher copy is used. Kills: a mount that takes the higher sequence without the extension check | S | host |
| N31 | Operation × lane table (2.3) | control lane; unbound lane; export, verify and restore lanes bound | every operation 1..=11 on every lane kind, table-driven from the "Permitted on" column; a second CREATE on the same export binding after SEAL and after ABORT; LIST on each lane before and after a seal; VERIFY, READ, APPEND, SEAL and ABORT naming an entry other than the binding's; control ABORT naming an entry that is not writing | permitted operations pass check 6, all others `Access` -5; second CREATE `Access` -5; LIST scoping as in 2.3 (export lane: zero records until its entry is sealed, then that entry only; verify and restore lanes: the bound entry only; control: all); a wrong entry on an operation lane `Access` -5 (check 6, also in the failed-mount state); control ABORT of a non-writing entry `NotFound` -19. Kills: a permission table with any entry flipped; CREATE permitted twice per binding; a LIST that returns an entry outside the lane's scope; (revision 2) an entry check placed after check 7 (a wrong-entry READ in the failed-mount state returns `Io` -12) | S | host |
| N32 | Once-per-boot destination grant (2.1, O13) | R5 profile; a test feature makes the destination service exit (syscall 2) after its mount, and the profile then runs for at least as many kernel poll rounds as the M5 restart path needs (`m5.rs:784-794`) | kernel counts from the `Engine`'s object, node and handle tables before and after the exit: block objects whose scope intersects the destination scope, their nodes and their handles in every process, emitted by the kernel as `DEST:GRANT:OBJECTS=`, `DEST:GRANT:NODES=`, `DEST:GRANT:HANDLES=` | K: after the exit all three counts are 0 (`Engine::exit` reclaims the exited service's node and then the object, lib.rs:2003-2012) and stay 0 at every poll round until the profile ends; no process holds a handle on such an object; no process is started with one. Kills: a profile that re-creates the destination grant on a kernel-driven restart in the M5 pattern (`m5.rs:260-279`), observed as `DEST:GRANT:OBJECTS=` or `DEST:GRANT:HANDLES=` above 0 after the exit | K profile | QEMU (R5); no host model, because the rule is about kernel profile code |
| N33 | Device results (2.5, revision 3) | mounted instance; a broker stub that returns, per request kind (read for READ, VERIFY and the FORMAT scan on a blank instance; write and flush for APPEND and SEAL step 3): submission -13, submission -14, another submission error, completion -12, completion -13, completion -16, completion -14; separately (review 4 OP-6), a READ or VERIFY on a bound lane whose completion -13 resets the device while an export lane holds completed, unflushed APPEND writes in sectors that held stale bytes before (revision 4): (i) non-zero bytes that fail `archive::verify` (required), (ii) zero bytes (optional; a special case of (i)), and (iii) a valid archive of the same sector-aligned length (the N37 (a) case), under both reset models | each request once per result; in the separate case, then SEAL on the export lane | read: `Io` -12 except completion -13 `Timeout` -13, no safe path, VERIFY never `Corrupt`; write or flush: `Io` -12 for the submission results and completion -12, `Unknown` -16 for completion -13, -16 and -14, safe path in every case; `NoDevice` -14 from none of these requests; with -14 to the mount FLUSH, INFO still reports `NoDevice` -14 (as N14). Separate case, (i) and (ii): cache kept, SEAL `Ok` and the entry verifies; cache dropped, SEAL `Corrupt` -18 at the step-3 readback, nothing listed, no catalog write; under both, never a listed entry that fails VERIFY. Variant (i) is required because non-zero stale bytes also defeat a readback check that refuses only zero sectors. (iii): as N37 (a), a negative test under both reset models (O17 (a); revision 5); the owner declined the alternative of recording it as a stated limit. Kills: a write whose completion -13 replies `Timeout`; a SEAL that seals without the step-3 readback (cache-dropped model lists an entry that fails VERIFY); a -14 that replies `NoDevice` from a request other than BIND; a read failure that starts the safe path or that VERIFY answers `Corrupt` | S | host (R2) |
| N34 | Record and copy validity (2.6, revision 3) | crafted copies, each valid except for one rule: a zero-record copy with copy sequence 0 (review 4 RQ-3; with records, the sealing-sequence rule would already refuse it); entry ID 0; label length 0, and 33; length 79, and 525 329; sealing sequence 1, sealing sequence above the copy's sequence, and two records with equal sealing sequences; under the decided O14, labels that fail only the content rules of `valid_name` (2.3): `.`, `..`, invalid UTF-8 (including an encoded UTF-16 surrogate), a C0 control character, U+007F, a C1 control character (U+0085), `/`, `\`, and each end of both bidirectional ranges (U+202A, U+202E, U+2066, U+2069), and, as valid, the code points just outside those ranges (U+2029, U+202F, U+2065, U+206A); separately, a valid copy beside non-zero sectors 8–15 | mount in the Rust core and in the oracle | each crafted copy invalid in both, with the same mount state (one copy invalid beside a valid one: the valid one is used; both: `Corrupt`); the valid copy beside non-zero sectors 8–15 mounts in both. Kills: a validator that drops any one of these rules, in either implementation, except the equivalent mutants below; a label-content check that narrows or widens either bidirectional range; a sectors 8–15 rule added to either. Equivalent mutants at the level of the mount state (revision 4; A10 and A11 from the O14/O16 code round, A11 also as X6, which survived as equivalent in its second code review), listed, not tested: in either implementation, a validator without the label-length bound "≥ 1", because under O14 `valid_name` refuses an empty label (2.6; A10 in the Rust core; in the oracle the case is told apart only by its refusal reason, which is no mount-state difference, section 7); in the oracle, the length bound inside its label-content check, because its record check refuses lengths outside 1..=32 first (A11/X6). Each crafted copy breaks exactly one stated rule, so every other rule is killed by a mount-state difference in each implementation's own tests | S + H | host (R2, R3; reverse differential, 8.2) |
| N35 | Remount contradicting the committed catalog (2.6, 2.7; O16; revision 4: O18) | SEAL step-4 write fails; the device then reads at the remount: (a) all zero in sectors 0–15; (b) a valid catalog with another destination ID that lists the provisional entry ID; (c) a valid catalog with the same ID whose records differ from the committed ones, and (revision 4) the committed records under another destination ID, and the committed records at copy sequence + 1; (d) a non-blank layout that is not a valid catalog (`Corrupt`); (e) *[owner decision O18, 23 November 2026]* a valid catalog that passes the step-4 extension check against the committed copy and carries the provisional entry ID, but differs from the copy encoded in step 4 in the new record's length, archive trailer FNV-64 or label, or in next entry ID; a verify lane bound to a sealed entry | SEAL; then INFO, export-lane LIST, control FORMAT, VERIFY on the verify lane | (a)–(c): failed-mount state (O16), INFO reports `Io`, export-lane LIST `Unknown` -16 with zero records, VERIFY and FORMAT `Io` -12, no device write, binding never resolved as *sealed*; (d) `Corrupt` state, INFO reports `Corrupt`, export-lane LIST and every other request `Corrupt` -18, never a record, no device write; (e) as (a)–(c) (O18). Kills: a remount that ends as a blank candidate (INFO reports `NotFound` -19 instead of `Io`); a remount that resolves (b) as *sealed* (export-lane LIST returns a record); (revision 4) a *not sealed* resolution that compares only the records with the committed copy, or ignores its destination ID or copy sequence (the added (c) cases); (revision 5) the pre-O18 *sealed* resolution, which accepts any catalog that passes the extension check and carries the provisional entry ID ((e) resolves *sealed*; M6 in CODE-REVIEW-O17-O18). Equivalent mutants, listed, not tested: in (d), holding the binding *not sealed* instead of *unknown*, which no request in the `Corrupt` state can observe; (revision 4, from the O14/O16 code round) the *sealed* resolution's extension check without its record-count clause (2.7 step 4; B4/R14, which survived as equivalent in its code review), and the removal of the defensive FORMAT refusal in a blank candidate with the safe-path latch set, which O16 makes unreachable (2.2; B6, unreachable by this text's reading of the call sites, which review 6 confirmed); (revision 5, from the O17 (a)/O18 code round) the extension check added back beside the equality of the *sealed* resolution, which byte equality with the encoded copy implies (M8, which survived as equivalent in CODE-REVIEW-O17-O18), and a comparison with the catalog as encoded before its step-4 decode instead of the decoded, checked copy (M7, which survived there), equivalent in every build without `test-hooks` because decoding a copy that passed the validator returns it unchanged; under the test-only N29 tamper hook M7 is observable, and an optional sub-case pins it: a tamper that passes step 4, then a failed step-4 write and a remount of the tampered copy, which must resolve *sealed* (CODE-DELTA (revision 5)) | S | host (R2) |
| N36 | CREATE and APPEND refusal order (2.3, revision 3; O15) | export lane bound; (a1) another lane writing; (a2) nothing writing, and no sector free, and separately 16 entries sealed (`Busy` and `Full` cannot apply together, 2.3); (b) an entry with 525 280 bytes appended; (c) a CREATE refused with `Busy` | (a1), (a2) CREATE with an invalid label, then with a valid label; (b) APPEND of 160 bytes with the right offset, then with a wrong offset; (c) after the other entry is sealed, CREATE again on the same binding | (a1) `Invalid` -1, then `Busy` -21; (a2) `Invalid` -1, then `Full` -17 (the second CREATE reaches the operation because the refused first one left the binding's CREATE unused, O15); (b) `Range` -3 with nothing appended and the entry still `writing`, then `Stale` -10; (c) CREATE `Ok` (O15). Kills: `Full` instead of `Range` for the archive bound; a CREATE order that reports `Busy` or `Full` before a bad label; a refused CREATE that uses up the binding (the second CREATE of (a1) or (a2), or the CREATE of (c), returns `Access` -5) | S | host (R2) |
| N37 | SEAL content digest and portal trailer check (2.7 step 3 and "Stale valid content"; *[owner decision O17, 23 November 2026]*: cases (a), (b), (d) test the service check O17 (a), case (c) the portal check O17 (b); revision 4, revision 5) | one sealed entry E1; export of E2 on lane X; the free sectors at E2's extent hold a valid v1 archive of exactly E2's sector-aligned length that differs from E2 (for the barrier-1 variant of (a) and for (b), where every write of the entry, the SEAL tail included, is lost or not stored, a stale archive of the same length need not be sector-aligned; review 8, RQ8-4): (a) a READ or VERIFY on another lane whose completion -13 resets the device while X holds completed, unflushed APPEND writes (as N33), and separately the device's own reset during the barrier-1 submission (as N26 (d)), under both reset models; (b) without a reset, a simulated disk that reports success for every write of the entry and for barrier 1 but stores none of them, so that the stale valid archive stays whole, under both reset models; (c) portal code against a service stub whose SEAL reply `aux`, and separately whose LIST record trailer FNV-64, differs from the trailer of the exported archive; (d) (revision 5) unlike the shared precondition above, a freshly formatted destination with no sealed entry, no stale archive and no fault, with APPENDs refused with `Stale` (wrong offset) and with `Full` (past the scope end) between APPENDs answered `Ok` | (a), (b) SEAL on X; then control LIST, INFO and a CREATE on a new export binding; (d) SEAL on X only; (c) the portal's export flow | (a) cache dropped and (b): S `Corrupt` -18, nothing listed, no catalog write, writing entry discarded, no safe path, so the later CREATE runs (decided: no latch, pending O9, 2.7); (a) cache kept: SEAL `Ok`, `aux` equals the sent trailer, E1 and E2 listed; (b) no reset occurs, so both reset models give the same outcome; (d) SEAL `Ok` and `aux` equals the trailer of the bytes answered `Ok` (refused APPENDs are not hashed); in (a) and (b), under both reset models, every listed entry verifies; (c) P reports the export as failed, never as sealed or confirmed, and does not repeat it blindly. Since the latching sub-question is decided (2.7), a latching implementation is a defect (revision 5; this replaces the review-6 OPT6-1 note, under which it was not counted as one). Host tests at `f993a1a3` *[PR #413]*: (a) `n37a_read_timeout_over_stale_valid_archive` (READ and VERIFY resets, sector-aligned stale archive; N33 (iii)) and `n37a_own_reset_at_barrier1_over_stale_valid_archive` (stale archive not sector-aligned); (b) `n37b_unstored_entry_writes_over_stale_valid_archive` (stale archive not sector-aligned; asserts the stale extent whole and no reset); (d) `n37_digest_covers_only_appends_answered_ok`. Kills (mutant names of CODE-REVIEW-O17-O18): a SEAL without the content comparison ((a) cache dropped and (b) list the stale archive; M1); a mismatch that sets the safe-path latch (the latch is set, and the later CREATE would return `Access` -5; M4); a mismatch that replies another status than `Corrupt` (M5: `Io`); a mismatch that writes a catalog copy (catalog write count above 0; M9); a digest updated for an APPEND refused with `Stale` or `Full` ((d) SEAL `Corrupt`; M2b, M3b); a digest that skips full-size APPENDs or is never updated ((a) cache kept and (d) SEAL `Corrupt`; M12, M13); a portal that does not compare the reported trailer ((c) reports sealed) | S + P | host (R2) for (a), (b), (d); (c) with the portal (R7) |

Every row except N19, N20, N24, N26 and N32 needs only the destination service and
existing primitives; N24 also needs a simulated disk with late completion and,
natively, the K4/K5 fence of 2.7; N26 needs a broker stub; N32 needs the R5
profile with a test feature that ends the service and kernel-emitted counts.
N33 needs a broker stub; N34 needs the oracle of R3 and the reverse
differential of 8.2; N33–N36 are host rows of R2 (N34 also of R3). N37
(revision 4; O17 decided on 23 November 2026): its cases (a), (b) and (d),
for the service check O17 (a), are host rows of R2 with the simulated disk,
and its case (c), for the portal check O17 (b), needs portal code and belongs
to R7. Each host test is
paired with a mutation that disables its check and must make the test fail, as for the archive and name-check tests *[main
practice]*.

## 7. Isolated disposable-QEMU verification plan

**Images.** All images are created fresh under `$NOS_BUILD_DIR/run/` by the
harness: the source volume image as today, and a destination image attached as
the second virtio-blk device at `00:06.0` *[main]*. No host disk, no physical
device, and no image survives between gate runs except as recorded evidence.
Every destination runner passes `-no-reboot` to QEMU (2.7), as
`tools/run_volume.py:309` does *[main]*, so a guest reboot ends the QEMU
process and each boot is a new process.

**PCI slot and entropy conflict (R5 precondition).** Every runner that
provides virtio-rng, the entropy source FORMAT needs, places it at `00:06.0`
(`tools/run.py:156`, `tools/run_keyring.py:32`) *[main]*, and the volume
runner places the second virtio-blk disk at the same slot without virtio-rng
(`tools/run_volume.py:322`) *[main]*. In addition, no volume-fixture profile
creates an `Entropy` object (`create_entropy` occurs only in the desktop,
keyring, nas, network_service, system and window_session profiles), and
`test-volume-archive-device` cannot be combined with `native-system`
(`volume.rs:15-18`) *[main]*. As `main` stands, `dest-format` cannot draw a
destination ID. Two ways out, both test-profile matters and not K4:
(a) a test-profile kernel change: virtio-rng at another PCI slot, accepted by
the kernel's device discovery (unchecked), plus an `Entropy` object and handle
for the destination service in the test-only destination profile; or (b)
*[decided: O4 (c), 24 November 2026; test profile only]* a **deterministic test destination ID**:
under a test-only service feature, FORMAT writes a fixed 16-byte ID instead of
drawing one, the service holds no `Entropy` handle, and every FORMAT evidence
line carries the label `DEST:ID:TEST-ONLY`. Option (b) is non-product: it
never builds without the test feature, and it leaves FORMAT's entropy draw
without native evidence, which then comes only with the first product profile
under K4 or earlier with option (a); the owner requires that native evidence
before R8. Stages that need a
second destination ID (N14 `dest-foreign`) use an image whose catalog the
harness writes with another ID. R5 does not start before one option is in
place.

**Reuse.** `tools/run_volume.py` and the `PLAN` of `tools/test_volume.py`
*[main]*, the disk-1 attachment and the seven stages `archive-disk-*` *[main]*,
`tools/volume_archive.py` as the independent v1 reader, `tools/volume_disk.py`
for the source image, the `ARCHIVE_ESCAPE` counting and the
`VOLUME:ARCHIVE:READS|WRITES|FLUSHES` accounting. New host oracle:
`tools/destination_disk.py`, an independent Python parser of the catalog layout
(2.6) that shares no code with the Rust service and refuses every invalid copy.
Revision 3 fixes what the oracle decides:
- **Scope.** It takes the scope's first sector and sector count as inputs,
  which the harness reads from the profile's grant (2.6: all sectors are
  scope-relative); bytes of the image outside the scope are not judged.
- **Image read errors.** An image that ends before sector 16 of the scope, or
  that cannot be read, gives the failed-mount state, never a blank candidate
  or an invalid copy (2.6 mount rule); an extent that ends past the image end
  is reported unreadable, never as zero bytes.
- **Extents.** Its own extent check is structural: the record's archive
  trailer FNV equals the last 8 bytes of the entry, and the padding after
  `length` in the last sector is zero. It is not an archive acceptance. Every
  native stage that asserts that an entry verifies also parses the entry's
  first `length` bytes with `tools/volume_archive.py`, the independent v1
  reader, as `dest-export` already requires.
- **Cross-check limit** (revision 4). The cross-check between the R2 core and
  the oracle, in both directions (8.2 R3), compares the mount state and, for a
  mounted copy, the header and the records. It does not compare which rule
  refused a copy, and this is its stated limit: the interface of 2.2–2.6
  reports mount states and reply statuses, never a refusal reason, so
  requiring the core to report one would add an interface that only tests
  use. Each rule is instead killed in each implementation's own tests by a
  crafted copy that breaks that rule alone (N34). A mutant that changes the
  refusing rule but no mount state is equivalent at this level and is listed
  in N34; the oracle's own tests may also assert its refusal reasons.

**New stages (proposed; names indicative).**

| Stage | Parent | Checks |
|---|---|---|
| `dest-format` | blank disk 1 | two byte-identical catalog copies with sequence 1, no other sector written |
| `dest-export` | `dest-format` + recovered volume | one sealed entry; extent parses with `volume_archive.py`; disk 0 unchanged |
| `dest-export-2` | `dest-export` | second entry appended; first extent and its catalog record byte-identical |
| `dest-cut-append` | `dest-export` | process kill, no cache loss: `VOLUME:CUT:` marker after k writes, host kill of the QEMU process; `DEST:IO:FLUSHES` and the flush order at the marker match 2.7 |
| `dest-cut-seal` | `dest-export` | process kill, no cache loss: cut between barrier 1 and barrier 2; flush count and order at the marker as above |
| `dest-cut-mirror` | `dest-export` | process kill, no cache loss: cut after barrier 2 completes, before barrier 3 completes (N12, third point); flush count and order at the marker as above |
| `dest-after-cut` | each cut stage | cold boot; LIST and VERIFY per N11/N12, for `dest-cut-mirror` LIST = {E1, E2} and E2 verifies; oracle agrees |
| `dest-restore` | `dest-export-2` + blank disk 0 | verify then restore entry 2; byte-exact per the existing restore oracle |
| `dest-revoke` | `dest-export` | revoke mid-export (N6) |
| `dest-foreign`, `dest-full`, `dest-replay`, `dest-corrupt` | as N14, N15, N17, N7/N8 | as in the matrix |
| `dest-scope`, `dest-sender`, `dest-nonarchive` | `dest-export` | N3, N22 and N10 at QEMU level |
| `dest-grant-once` | `dest-format` | N32 |

**Evidence lines.** `DEST:CATALOG:SEQ=`, `DEST:ENTRIES=`, `DEST:NEXT_FREE=`,
`DEST:IO:READS|WRITES|FLUSHES` (with the sequence of writes and flushes up to
each cut marker), `DEST:HANDLES=` per process and stage, `DEST:GRANT:*` (N32),
`DEST:DENIALS=` and markers `DEST:CASE:<stage>:PASS`. `DEST:IO:*` and the
order of writes and flushes are counted by the kernel broker, keyed by the
destination object key, as `archive_ops` is for `VOLUME:ARCHIVE` today
*[main]*; `DEST:HANDLES=` and `DEST:GRANT:*` come from the kernel `Engine`;
the cut marker is emitted by the kernel after k destination writes,
independent of the flush count. None of these is a service self-report, so a
service that skips a flush cannot keep a counter consistent with the order of
2.7. `DEST:CATALOG:SEQ=`, `DEST:ENTRIES=` and `DEST:NEXT_FREE=` are service
reports and are cross-checked by the oracle's parse of the final image.
`DEST:DENIALS=` is a service report that no image parser can check; it
supports N22 but does not decide it. It counts every request refused by checks
1–6 of 2.2 on any lane, whether or not a reply is sent, and nothing else:
refusals by check 7 or by the operation are state outcomes, not denials of
authority or framing (revision 3). N22's native evidence is
`DEST:PORTAL:REPLYQ=`, the length of the portal's reply-endpoint queue,
emitted by the kernel from the `Engine`. Only these markers plus the raw QEMU exit
code 33 count as a pass, as for the existing gates.

**Cut stages are not power-cut evidence.** The `dest-cut-*` stages kill the
QEMU process with the host page cache intact (section 6), so they show
recovery after a process kill and the recorded flush order, not durability
under cache loss. Their recovery checks alone cannot detect a missing barrier
1 or 2; the flush count and order asserted at each cut marker can. Behaviour
under cache loss is accepted host-only (`SimDisk`, R2). A replay
oracle over QEMU's `blklogwrites` log, which would rebuild every image the
cache model allows and mount each, is an optional later research item, not a
requirement of this plan.

**Stage-count pins.** Adding stages to `PLAN` changes `len(PLAN)`, which two
protected workflow values must equal (`.github/workflows/kvm-pilot.yml` and
`.github/workflows/volume-acceptance.yml`, checked by
`tests/test_ci_workflows.py`). On `main` both pins and `len(PLAN)` are 22
(`volume-acceptance.yml:62`, `kvm-pilot.yml:124`, `tools/test_volume.py`)
*[main]*. Adding destination stages to `PLAN` therefore needs a lead-owned pin
update, or the stages go into a separate gate (section 8, O6).

**Remains unproven after this plan.** No IOMMU and no malicious-device model;
QEMU's virtio-blk is trusted. No physical disk, partition table or USB device.
No authenticity of the destination or archives. No hosted acceptance until the
hosted gate runs. No human audit.

## 8. Open items and staged implementation route

### 8.1 Items needing a project-owner decision

- **O1** Remembered permission: which operations may be remembered (proposed:
  export and verify, not restore), where the record is persisted, and whether
  the "remember" option is unchecked by default. A persisted remembered
  permission is a new extension of the existing per-session permission model,
  not the model D2 (a) refers to as existing, and needs owner confirmation
  under D2. Input for the decision: the record keys on the unauthenticated
  destination ID (3.3, 5.4), so a cloned catalog could receive automatic
  plaintext exports under a remembered export permission.
  **Decided on 24 November 2026 (package 2):** for the unencrypted v1 route only VERIFY may be
  remembered, and the option is unchecked by default. A chosen destination may
  be remembered as a user preference that grants no export, restore, erase or
  format right. Every new plaintext backup needs explicit consent for that
  operation; a copied destination ID receives no automatic export. A
  remembered right lives in protected, profile-bound storage of the trusted
  consent facility, bound to owner, portal, destination, permitted operation
  and revocation generation; revoking it also ends the live bindings and
  volume-side rights (4.1). RESTORE, FORMAT and destructive management are
  never remembered. Automatic backups remain a product goal, to be designed
  with B under the key and encryption contracts with a proven recipient and
  owner binding; a destination ID or FNV check is not authentication, and no
  unattended plaintext export is activated as a stand-in.
- **O2** Formatting: whether a non-blank device may ever be erased into a
  destination (proposed: no in v1; blank devices only).
  **Decided on 24 November 2026 (package 2):** FORMAT accepts only a scope that passes the full
  blank check of 2.6; a zero header or some zero sectors do not prove a blank
  scope. A non-blank device is never erased automatically or taken as a new
  medium. For the product, a separate trusted preparation flow is approved as a
  development direction (trusted target selection, visible device identity and
  capacity, a check against the source and system disk, explicit confirmation
  of data loss), designed and tested separately. This grants no permission to
  erase real disks during development.
- **O3** Retention: v1 has no deletion, so `Full` is terminal for a destination.
  Deleting or rotating old entries needs a decision, a catalog operation and its
  own crash tests.
  **Decided on 24 November 2026 (package 2):** v1 stays append-only: no deletion, no reuse of
  freed space, no automatic rotation; at `Full` a new backup stops with a clear
  message and existing backups stay intact. For the product, controlled
  deletion of a user's own backups and optional, explicitly configured
  retention are decided; automatic deletion is off by default, and no default
  period or count is invented without product grounds. It is a separate,
  versioned extension with catalog, mount, restore, ownership and revocation
  rules and crash tests, and a retention policy must never silently delete a
  user's last proven restorable backup.
- **O4** (a) Approval of kernel changes K1–K5 as needed (K4 and K5 are required
  for any product profile; K1–K3 can wait). Revision 2 widens K5 from one
  `Policy` per device to per-device broker state (4.3), because only that
  confines kernel -8 on disk 1 to disk 1's own reset (2.7); the approval asked
  for is the wider one. Revision 5 adds one stated delta beyond per-device
  state: a further completion poll after a read ended by a reset returns -1
  (4.3 K5; 2.7, property 3). It is a stricter kernel obligation, not a
  widened capability, and the approval asked for covers it (review 10,
  OPT10-6). (b) **Owner question:** does the
  project owner amend ADR-S09-DUALDISK-20261119 before R5, so that B3 permits device 1
  to be held by the destination service with a whole-device scope (today B3
  grants device 1 only to the slot-1 portal, with a scope that ends below the
  disk end, `volume.rs:705-711` *[main]*), and B5 permits a scripted test-only
  dialog driver (today B5 states that there is no dialog and no persistent
  grant)? See [S09-DUAL-DISK-FIXTURE-REQUEST.md](S09-DUAL-DISK-FIXTURE-REQUEST.md).
  R5 does not start before this amendment.
  (c) (new in revision 2) **Owner question:** may R5 use the deterministic,
  test-only destination ID of section 7 (proposed), or must the test-profile
  kernel change (virtio-rng at another PCI slot plus an `Entropy` object in the
  test-only destination profile) be made before R5? R5 does not start before
  this is decided; either way FORMAT's entropy draw needs native evidence
  before R8.
  **Decided on 24 November 2026:** (a) K4 and K5 approved for targeted
  implementation; K5 keeps all relevant broker state and buffers per device,
  including the further-poll rule of property 3; K5 is an R5 precondition, K4
  a product-profile precondition; K1–K3 do not block R5; attaching a backup
  device during use stays product work. (b) B3 and B5 amended for R5: the
  destination service holds disk 1 exclusively with READ, WRITE and FLUSH,
  and a scripted test dialog is permitted; disposable images only, no product
  activation, no real host disks. (c) The deterministic ID is permitted only
  in the explicit test profile with `DEST:ID:TEST-ONLY`; product builds must
  not use it; the real entropy route needs native evidence before R8.
  **Package 2 additions (24 November 2026):** K5 separates per device at
  least the `Policy`, the reset state, the reset deadline, the error state, the
  active request and the data buffer, so that a fault on the destination disk
  does not needlessly block the source disk; K5 comes first for R5, and the O13
  kernel write latch belongs to the further K5 work. K4 is built for the
  product profile. "Present at boot" must not become a permanent product
  limit; hot-plug stays product work. Quotas and concurrency limits are not
  raised without shown need. Under (b) the block handle carries no DERIVE,
  TRANSFER or REVOKE and the portal gets no raw disk rights; bounded devices
  and DMA remain.
- **O5** Time display: whether any wall-clock time is shown for an entry. The
  catalog stores none, and v1 object times are a per-domain logical clock,
  not wall-clock time ([S09-BACKUP-ARCHIVE.md](S09-BACKUP-ARCHIVE.md)).
  **Decided on 24 November 2026 (package 2):** no date or time is invented, and a logical
  counter is never shown as wall-clock time. Existing v1 entries show label,
  order, size and, where needed, "date unknown". New product metadata carries
  a usable backup date and time in the versioned catalog extension, taken from
  the trusted system clock with a stated meaning and shown in the local time
  zone; uncertain or missing clock time is shown as such. Wall-clock time
  replaces no monotonic deadline, generation or other security or recovery
  rule.
- **O6** (lead-owned) workflow stage-count pins for new destination stages, or a
  separate gate.
  **Decided on 24 November 2026 (package 2):** a separately recognisable destination acceptance
  gate for the new native destination stages, beside the existing applicable
  volume regression tests. The CI owner changes the protected workflows and any
  stage-count pins, with expectations derived from the actual test plans, not
  copied historical counts. The hosted R3 cross-check, the reverse
  differential and the P10 lint of the `no_std` library must actually run as
  specified; which gate belongs to which change and milestone is recorded. The
  acceptance-hold and exact-head procedures apply; a skipped, running or
  unexecuted relevant test is not a passed acceptance.
- **O7** Whether the dialog may hold the lane issuing roots (section 4.3) or
  D4 (a) requires them in the portal. With the sender check of 4.3 the dialog's
  residual authority on an operation lane is to bind, transfer and revoke, not
  to perform an operation.
  **Decided on 24 November 2026 (package 2):** the proposed separation is kept: the trusted
  dialog holds the issuing roots and the control lane, the portal receives only
  the temporary SEND rights for the permitted operation, and the destination
  service checks the kernel-stamped sender, binding and generation. Negative
  tests must show that the dialog cannot perform backup operations as a
  portal with its remaining rights. Identity must not rest on an unproven
  assumption about process-ID reuse: the needed invariant is proven, or a
  reviewed incarnation or generation contract is used.
- **O8** (withdrawn in revision 1) In-boot restart after the failed-mount
  state. Revision 0 named a service restart as a recovery route. Under the
  proposed once-per-boot grant rule (2.1, O13), no instance in the same boot
  can regain the manifest-only, non-transferable block grant, so recovery from
  the failed-mount state and from the safe path is a reboot, and the restart
  route is withdrawn. If the owner later wants an
  in-boot restart, a re-grant mechanism and a kernel write latch in the
  per-device broker become owner decisions together; neither is assumed here.
  How the portal reports "reboot required" to the user remains a UI detail of
  R7.
  **Decided on 24 November 2026 (package 2):** the withdrawal of the in-boot restart route is
  kept; a service restart never clears the safe state or the write latch or
  regains write rights. A safe automatic restart may be designed later, keeping
  kernel state, revocation and recovery rules; until it is shown, the current
  recovery route applies and the UI states honestly when further recovery is
  needed.
- **O9** Completion fence (2.7): whether a write that the device accepted
  before an acknowledged reset can still reach the backing medium after the
  reset and the remount FLUSH, for QEMU and its host storage stack; whether the
  K4/K5 per-device broker implements the reset fence of the M3 volume broker
  for disk 1; whether a new QEMU process is equivalent to a device power cycle
  that no request of the previous boot survives; and how a product handles a
  warm reboot, which a guest cannot distinguish from a cold boot. All are
  *[unproven]*; N24 (b) records the outcome without counting it as a pass.
  Until they are shown, the 2.7 safe path applies: read-only for the rest of
  the boot after any device error on a destination write or flush (catalog or
  entry, `Io` included), carried by the single instance's in-memory latch
  under the once-per-boot grant rule (2.1, O13), with confirmation only after a
  cold boot and VERIFY. The gates avoid warm
  reboots by `-no-reboot` (2.7, section 7).
  - *Source-level evidence only.* A reading of the QEMU 10.2.0 source made for
    this revision indicates that `virtio_blk_reset` drains in-flight requests
    synchronously (`blk_drain`) and that a guest write of device status 0 resets
    the device synchronously after stopping ioeventfd. This is not empirical,
    it covers neither the host kernel's page cache nor asynchronous I/O
    teardown, and the review's own "not evidence" list includes reading QEMU
    source; it does not count towards O9.
  - *Planned empirical route* (the review's proposal; not to be run without
    owner scheduling, and only after the destination service, its
    `DEST:O9:*` markers and `tools/destination_disk.py` exist). Common setup:
    the runner's exact QEMU version and machine (`pc-q35-8.2`), a raw image on a
    local ext4 or xfs file system (not tmpfs), `aio=threads` and `io_uring`,
    `cache.direct=off` and `on`, `werror=report,rerror=report`, `-no-reboot`,
    and host-side QEMU trace events for virtio status writes and virtio-blk
    submit and completion. **E1 (in boot):** a test feature lowers the request
    timeout and emits markers for arming, reset begin, reset acknowledgement
    and remount FLUSH completion; at arming, the harness delays the targeted
    host `pwrite`/`fdatasync` below QEMU's block layer (for example with
    `strace` fault injection, if the installed version supports it) for longer
    than the timeout. Cells: step-4 sector 0, step-4 sector 3, step-5, the
    barrier-2 FLUSH and an APPEND entry write, each in both aio modes and both
    `cache.direct` settings, at least 20 runs per cell, each ended by QMP
    `quit` and parsed by `tools/destination_disk.py`. E1 supports O9 for that
    exact configuration only if every run shows the -16 reply and the reset
    path for the targeted request, the delayed host call returning before the
    status-0 write completes (or never issued), no write of pre-reset content
    to the targeted range after the remount `fdatasync` returns, and the
    guest's resolution equal to the oracle's parse of the final image; any
    pre-reset write returning later, or a resolution mismatch, refutes it.
    **E2 (across boots):** hold a host write as in E1, then kill the QEMU
    process, and separately kill it during a write flood with `io_uring` and
    `cache.direct=on`; after `waitpid` the image must stay byte-stable (hash
    and file-change watch) until the next QEMU process's first write, and a
    lock failure on restart is a failed start, not a pass. E2 shows only that
    no request survives a process restart; it shows nothing about cache loss.
  - *Not evidence for O9:* a delay shorter than the timeout; no host-side
    trace; guest readback or markers alone; `SimDisk`; reading QEMU source; a
    single aio mode; tmpfs, `cache=unsafe` or `no-flush`; a VM stop; a QEMU
    version, backend or options other than the runner's; any claim about
    physical media. The K4/K5 broker fence still needs its own native timeout
    fixture (R5). N24 and E1/E2 are planned tests, not evidence.
  **Decided on 24 November 2026 (package 2):** the empirical E1 and E2 trials and the matching
  native broker trials are approved for scheduling and execution once the
  implementation and markers exist, with the exact recorded QEMU and backend
  configurations and isolated disposable images only; host and guest
  observations are kept and conclusions are reported per tested configuration.
  The approval proves no completion fence, safe warm reboot or physical-media
  behaviour. Until fitting evidence exists the fail-closed rules and the
  bounded safe path stay, and O17 is not changed on assumptions. If a result
  refutes a security contract, the affected route stops and its correction is
  worked out.
- **O10** `Corrupt` after an interrupted FORMAT (2.6, N25): the policy by
  which a product distinguishes a FORMAT interrupted before its first copy
  was durable from a damaged existing destination, and whether any recovery
  other than external re-zeroing of a disposable image is offered. Not
  specified; until decided the service never formats a `Corrupt` scope.
  **Decided on 24 November 2026 (package 2):** a `Corrupt` destination is never reformatted,
  repaired or treated as "probably blank" automatically. In v1 the operation
  stays refused and the medium is kept; a test harness may only recreate its
  own disposable image. A product diagnosis and recovery route is to be
  designed: recovery rests only on provably valid data and the current owner
  rights; erase and re-preparation fall under the explicit O2 flow and are
  never presented as data recovery.
- **O11** (new in revision 1) **Owner question:** must a v1 destination be a
  block device of its own (proposed, 2.1), or may it be a partition whose
  scope is disjoint from every other block scope, accepting that it shares
  reset and FLUSH with the source device and is no independent copy against
  loss of that device? Requiring a separate device narrows the product scope
  that section 1 stated in revision 0 ("device or partition"). The exact-rights
  part of N18 does not depend on this decision. R8 activation is blocked until
  O11 is decided; the test stages use disk 1 and satisfy both options.
  **Decided on 24 November 2026:** a separate storage device; a partition on
  the source device is not an independent recovery copy.
- **O12** (new in revision 1) **Owner question:** is the v1 destination
  service single-profile, with the dialog refusing every destination operation
  while more than one profile exists (proposed, 3.3), or is an owner principal
  bound per destination or per entry now, with the catalog and format change
  that needs? Without either, profile B could list and restore profile A's
  plaintext backup, contrary to
  [PROFILES-AND-USERS.md](PROFILES-AND-USERS.md). R7 and the R8 activation are
  blocked until O12 is decided; the test-only R5 and R6 stages run in a
  single-profile fixture.
  **Decided on 24 November 2026:** owner binding per backup registration;
  no general disabling of backups with a second profile; catalog version,
  trusted identity, access checks and recovery route to be designed and
  checked against the profile, key and encryption contracts (3.3); R5 and R6
  may use one profile; an owner field is not evidence of encryption.
  **Package 2 additions (24 November 2026):** the owner identity comes
  from the trusted user and session facility, never from an app-supplied name
  or ID. Showing backups and metadata, reading, VERIFY, export, restore and
  management are checked against owner rights. A reissued account ID does not
  inherit old backup rights. Recovery without the original source disk or
  computer is designed so that the rightful owner regains access through the
  current recovery facility and no other user does. Product privacy, including
  protection of names and metadata, follows the key and encryption contracts
  with B. The archive-v1 payload is not changed silently; a new format gets an
  explicit version, compatibility rules and tests.
- **O13** (new in revision 2) **Owner question:** is the in-memory latch of the
  single service instance accepted as the carrier of the read-only safe path,
  on the condition of the proposed once-per-boot grant rule (2.1: the
  destination block object is created exactly once per boot, and no kernel
  restart, rebind or poll path creates a block object intersecting the
  destination scope or delegates a handle on one; tested by N32 only, not
  enforced by the kernel), or does the project owner require a kernel write
  latch in the per-device broker (an extension of K5) that refuses every write
  to the destination scope for the rest of the boot after the trigger? On
  `main`, kernel profile code can re-create a block grant within one boot
  (`m5.rs:260-279`, `:784-794`), so the absence of TRANSFER alone does not
  carry the latch. R5 implements the proposed rule and N32, which are
  compatible with either answer; R8 activation is blocked until O13 is decided,
  and if the owner chooses the kernel latch, it and its own native test are
  added before R8.
  **Decided on 24 November 2026:** the kernel write latch, which a restart,
  rebind or newly granted capability does not bypass; implemented and tested
  natively before R8; the once-per-boot rule and N32 are kept (2.1, 2.7).
  **Package 2 additions (24 November 2026):** after the trigger the
  kernel refuses new WRITE requests to that destination scope for the rest of
  the boot, whatever service restart, rebind, new block object or new handle;
  it is limited to that destination and keeps READ and the remount FLUSH as the
  recovery protocol needs them, so it does not silently become a ban on all
  I/O. N32 stays and the latch gets its own native test. O17 stays: a digest
  mismatch alone does not set the latch without a new decision based on O9
  evidence.
- **O14** (new in revision 3) **Owner question:** does the copy validator of
  2.6 apply `valid_name` (2.3) in full to every record's label at mount
  (proposed), so that one label with invalid UTF-8, a control character, `/`,
  `\` or a bidirectional override makes its copy invalid, or only the length
  and padding rules, with the trusted dialog rendering every label as
  untrusted text? The proposal fails closed and keeps such labels off the
  trusted surface; its cost is that one bad label makes the whole destination
  `Corrupt`. At `37b7bd0c` neither the R2 core nor the R3 oracle checks label
  content at mount *[PR #413]* (the oracle refuses a label length above 32
  through its padding rule but accepts 0, and accepts entry ID 0; that is the
  L5 change of section 9). **Decided by the project owner on 23 November 2026:
  the proposal is accepted** (`valid_name` in full at mount). R7 (label
  display) and the R8 activation no longer wait on O14; R5 and R6 do not
  depend on it. The R2 core and the R3 oracle must apply the rule (section 9).
  Status (revision 4): both apply it at `e52cb865` *[PR #413]*, with the
  N34 label cases; the control-character set is named in 2.3 (P3).
- **O15** (new in revision 3) **Owner question:** does a CREATE refused by
  check 7 or by the operation use up the export binding's one CREATE (2.2
  check 6)? Proposed: no, only a CREATE answered `Ok` counts, because a
  refused CREATE allocates nothing and a transient `Busy` then needs no new
  BIND. The alternative is that every CREATE that passes checks 1–6 counts.
  The R2 core follows the proposal *[PR #413]*. **Decided by the project owner
  on 23 November 2026: the proposal is accepted** (only a CREATE answered `Ok`
  counts). R7 (portal retry behaviour) and the R8 activation no longer wait on
  O15.
- **O16** (new in revision 3) **Owner question:** after the 2.7 trigger, does a
  remount whose reads would give a blank candidate, or a valid catalog that
  is neither the committed copy nor that copy plus exactly the provisional
  record (another destination ID, or other records; revision 3 after
  review 4 OP-5), enter the failed-mount state with the export binding
  *unknown* (proposed, 2.6), or the state that the mount rule gives (blank
  candidate with the safe-path latch kept and FORMAT refused with `Access` -5,
  or mounted with that catalog)? Both write nothing. The proposal keeps the
  state table of 2.2 without a new cell, does not resolve the binding from a
  read that contradicts the catalog mounted earlier in the same boot, and
  keeps bound verify and restore lanes from naming another catalog's entries;
  it requires the service to keep the committed copy across the remount for
  the comparison. A remount that ends `Corrupt` leaves the binding *unknown*
  under either answer (2.7). At `37b7bd0c` the R2 core implemented the
  alternative *[PR #413]*. **Decided by the project owner on 23 November 2026: the
  proposal is accepted** (failed-mount state, binding *unknown*). The R2 core
  must change from the alternative to this rule, and N35 asserts the
  failed-mount state as well as the absence of any device write (section 9).
  The R8 activation no longer waits on O16. Status (revision 4): the R2 core
  follows this rule at `e52cb865` and N35 asserts it *[PR #413]*. Revision 4
  proposed a stricter reading of "that copy plus exactly the provisional
  record" as O18, which the owner accepted on 23 November 2026; it refines
  the *sealed* arm of this rule and leaves its other arms unchanged (2.7).
- **O17** (new in revision 4) **Owner question:** must a SEAL refuse an
  extent that passes `archive::verify` but does not hold the bytes that were
  appended (2.7, "Stale valid content")? (Question text as asked in
  revision 4, with three edits of revision 5 that do not change its
  meaning: the conditional sentences were removed (Q3; review 8, OPT8-3),
  "is recommended" in the sub-question became "was recommended", and the
  marking sentence names N37 (d) beside N37 (a) and (b) (Q9); review 9,
  OPT9-2. "Today" describes the state before the decision, and the trailer
  comparison of option (a) is relied on only under the precondition of 2.7
  step 3; review 7, OPT7-5.) Today a device fault that loses
  completed, unflushed APPEND writes, or silently fails to store one, can
  leave a stale valid archive of the same length in the extent; SEAL then
  commits another backup's content under the new label, the entry stays for
  good (no deletion in v1, O3), and a restore accepts it. Options: (a) the
  service keeps an FNV-64 digest of the appended bytes and compares it with
  the readback at step 3 (or uses the equivalent trailer comparison of 2.7
  step 3), replying `Corrupt` -18 before any catalog write;
  (b) the portal compares the trailer FNV-64 that the service reports (SEAL
  `aux`, LIST record) with its own archive's trailer before it reports the
  export as sealed or confirmed; (c) both. Trade-offs: (a) stops the wrong
  entry before the commit, so nothing permanent is written, at the cost of
  8 bytes of state per writing entry, one digest pass over the readback that
  step 3 already reads, and a failed export (to be retried) in the fault
  case; (b) needs no service change, but the wrong entry is already
  committed, keeps one of the 16 slots and appears in the dialog's restore
  list with the new label; it relies on portal code alone. **Threat model:**
  both detect an accidental mismatch from a device or cache fault. FNV-64 is
  not collision-resistant and a device that misbehaves on purpose can return
  the appended bytes on the readback; a defence against a hostile device
  would need a maintained standard cryptographic hash, is out of scope under
  5.4 and would be a separate decision. **Recommendation: (c)**, with (a) as
  the normative service rule and (b) as a cheap second check in the portal
  against a service defect; a mismatch replies `Corrupt` and does not start
  the safe path (2.7). Sub-question (review 6 OPT6-1): whether a digest
  mismatch, or any device reset while unflushed entry writes exist, starts
  the safe path instead. Not latching follows the revision-3 readback rule
  and was recommended until O9 shows whether late writes can land; latching
  costs the rest of the boot's exports after a single fault. The service
  rule of 2.5 and 2.7 step 3 and rows N33 (iii) and N37 (a), (b), (d) are
  marked O17 (a); the portal rule of 2.7 and row N37 (c) are marked
  O17 (b); 5.4 names both. **Decided by the project
  owner on 23 November 2026: the recommendation is accepted** ((c): the
  service rule (a) and the portal check (b); a mismatch does not start the
  safe path, pending the O9 evidence). The R2 core must implement (a) with
  N37 (a), (b) and N33 (iii) as negative tests; (b) and N37 (c) belong to R7
  (section 9). Consequences (revision 5): options (a) alone and (b) alone,
  and leaving the case as a stated limit of 2.7, are declined. The latching
  sub-question is decided as **no latch, pending O9**: a mismatch replies
  `Corrupt` -18 and leaves the safe-path latch clear (2.7, N37). Only O9
  evidence that a write the device reported as completed, or accepted before
  a reset, can reach the medium later reopens it, through a new owner
  decision; N26 (d), N33 (the separate case and its read-failure kill) and
  N37 would then change with it. R7 still needs the portal check (b) and
  N37 (c); the R8 activation no longer waits on O17. Status (revision 5):
  the R2 core implements (a) in the running form at `f993a1a3`, and N33 (iii)
  and N37 (a), (b) are negative tests under both reset models *[PR #413]*;
  (b) and N37 (c) are open (R7).
- **O18** (new in revision 4) **Owner question:** after the 2.7 trigger, does
  a remount resolve the export binding as *sealed* only if the remounted
  catalog equals, byte for byte, the copy that the service encoded and
  checked in step 4 of that SEAL (proposed, 2.7), rather than any catalog
  that passes the step-4 extension check and carries the provisional entry
  ID (the decided O16 definition)? The proposal is stricter: a remount that
  lists the provisional ID with another length, trailer FNV-64 or label, or
  another next entry ID, enters the failed-mount state with the binding
  *unknown* instead of resolving *sealed*; it adds no write and no new state.
  Reason: this boot's writes can leave only the committed or the encoded copy
  at a remount, so any other catalog shows a changed or misbehaving device,
  which O16 already refuses to resolve from, and a resolved record that
  differs from the encoded one would let the export lane's LIST report a
  record the service never wrote. Cost: the service keeps the encoded copy
  (2048 bytes) across the remount, as it already keeps the committed copy;
  a device fault in that window leaves the export *unknown* rather than
  *sealed*, which is the safe direction. It answers review finding O-2 of the
  PR #413 code review and review 5 OPT-2. **Recommendation: accept.**
  O18 is independent of O17: O17 compares the extent with the appended
  bytes, O18 the remounted catalog with the encoded one. **Decided by the project owner on 23 November 2026: the
  proposal is accepted** (byte equality with the encoded copy). It refines
  the *sealed* arm of the decided O16 rule; the R2 core must change from the
  extension check plus the provisional entry ID to this rule, and N35 (e)
  asserts the failed-mount state (section 9). Consequences (revision 5): the
  alternative, the O16 definition alone for the *sealed* arm, is declined;
  the extension check stays required only as the pre-write check of 2.7
  step 4. The R8 activation no longer waits on O18. Status (revision 5): the
  R2 core resolves *sealed* only on full equality with the decoded, checked
  step-4 copy at `f993a1a3`, and N35 (e) asserts the failed-mount state for
  the four cases (length, trailer, label, next entry ID) *[PR #413]*.

### 8.2 Staged route

Each step is one reviewable draft PR; none activates a production boundary.

| Step | Content | Acceptance check |
|---|---|---|
| R1 | This specification; independent security review of it | review findings recorded and resolved in this file (reviews 1–3 and the revision-3 ledger: section 9). **Status (revision 3):** review 3 approved revision 2; review 4 (SPEC-REVIEW-4) returned REQUEST_CHANGES on revision 3; review 5 (SPEC-REVIEW-5) found its findings resolved and returned REQUEST_CHANGES with two required findings, answered in this text (section 9) and not re-reviewed. **Status (revision 4):** owner items O14, O15 and O16 decided on 23 November 2026; O4 (b), O4 (c) and O11–O13 open; O17 and O18 proposed in revision 4 and open; review 6 (SPEC-REVIEW-6) approved revision 4 (P1–P13, section 9) with no required finding and confirmed RQ5-1 and RQ5-2 (P13); its nine optional findings were applied by the lead and checked by one read-only verifier, whose two wording corrections were applied and not re-reviewed. **Status (revision 5):** owner items O17 and O18 decided on 23 November 2026 (O14–O18 decided; O4 (a)–(c) and O11–O13 open; the other items of 8.1 remain open as stated there); review 6 approved revision 4; the owner-decision commit `a769bc65`, which relabelled the O17 and O18 markers, and the review-6 corrections of revision 4 were covered by reviews 7 and 8, which returned REQUEST_CHANGES on other findings, and first approved by review 9; review 7 (SPEC-REVIEW-7) returned REQUEST_CHANGES on revision 5 (Q1–Q13, section 9), and the lead applied its nine findings; review 8 (SPEC-REVIEW-8) returned REQUEST_CHANGES on the result with four required and six optional findings, which the lead applied (RQ8-2 is a commit action; section 9, "Review 8"); review 9 (SPEC-REVIEW-9) approved revision 5 (with those fixes), `a769bc65` and the review-6 corrections, with no required finding and nine optional findings (OPT9-1–OPT9-9), which the lead dispositioned in a separate fix pass (section 9, "Review 9"); review 10 (SPEC-REVIEW-10) returned REQUEST_CHANGES on that pass with two required and eight optional findings (RQ10-1, RQ10-2, OPT10-1–OPT10-8), which the lead applied in a further fix pass (section 9, "Review 10"), to be checked by review 11 (SPEC-REVIEW-11), which approved it with no required finding; its nine optional findings are deferred to the next revision |
| R2 | Host crate for the service core: catalog codec, mount rule, state machine, wire codec, write path; simulated disk with cache loss | host tests for N3–N15, N17 and N21–N31 (simulated disk with late completion for N24, broker stub for N26; N5 and N6 with the capability-core `Engine`), each with a killing mutation; flush-order and cache-loss acceptance for N11, N12 and N25 is established here, not natively; Clippy, fmt, `cargo test --locked` per the CI workflow commands. Revision 3 adds: N33–N36; the reset-dependent rows under both reset models (section 6); a simulated disk that meets the K5 broker contract (2.7). **Status (revision 3):** implemented on draft PR #413, unmerged *[PR #413]* (host only; AI-reviewed, not human-audited). Open against this revision: the archive verifier is a test stand-in, so N7, N10, N11 and N12 are partial; N24 (b) is recorded, not a pass; at `37b7bd0c` the revision-3 rows and the code changes they imply (section 9) were not yet made. Revision 4 adds: Clippy with `-D warnings` on the library as built without any test-only feature, for `x86_64-unknown-none`, run by the hosted gate (a lead-owned workflow change), because that is the build a native profile links and so far it is only type-checked (P10); and a host test of the simulated disk alone, under both reset models, for each property of the broker contract of 2.7, including for property 3 that a result already returned is not returned by a further poll, also for a request ended by a reset (P12); N37 (a) and (b) (P1; O17 (a) decided on 23 November 2026). **Status (revision 4):** the revision-3 rows and the O14 and O16 changes are implemented on draft PR #413 (`4f1edb9c`, `e52cb865`), unmerged, host only, AI-reviewed, not human-audited. Still open: the archive verifier is a test stand-in, so N7, N10, N11 and N12 are partial; N24 (b) is recorded, not a pass; the CODE-DELTA of section 9 (revision 4); the two items above. Revision 5 adds: for properties 4 and 5 of the broker contract, an opt-in deadline mode of the simulated disk (a request past its deadline answered -16 if mutating and -13 otherwise by the model itself, then the reset) and quarantine and offline held as device state until a simulated reboot, each with a host test under both reset models, without relaxing the contract or this check (2.7; Q4); a stray-poll counter in the simulated disk, asserted zero in every service-driven host test (2.7 poll rule; Q6); N37 (d) (Q9); optionally, the N35 sub-case that pins the checked step-4 copy (Q5). **Status (revision 5):** at `f993a1a3` *[PR #413]* (unmerged, host only, AI-reviewed in CODE-REVIEW-O17-O18, not human-audited) the R2 core implements O17 (a) in the running form and O18; N26 (d) asserts that no catalog sector is written in the cache-dropped model; N33 (iii) and N37 (a), (b) are negative tests under both reset models; N37 (d) pins that only APPENDs answered `Ok` are hashed (corrected after review 7, OPT7-3); N35 (e) asserts the failed-mount state; one host test per broker-contract property exists (`p12_1`–`p12_5`), and the claim that the simulated disk meets the contract is partial for properties 4 and 5. The code review reports 76 tests passing with Rust 1.100.0 (all suites also pass with 1.90.0), Clippy and fmt, a check for `x86_64-unknown-none`, and 14 of 16 mutants killed, with M8 surviving as equivalent and M7 as equivalent in every build without `test-hooks`. These gates and mutants ran on the uncommitted diff over `8f387ea1`; the lead then applied the review's OPT-1, OPT-2 and OPT-4 as comment-only edits (the review's provenance note; `service.rs` at `f993a1a3` differs from the reviewed hash) before committing `f993a1a3`, and no review or gate run covers those edits. Nothing was re-run for this revision; the test count was checked here only by counting test attributes. The P10 lint of the non-test build ran locally with exit 0 (Rust 1.90.0) and is still not a hosted gate. Still open: the archive verifier is a test stand-in, so N7, N10, N11 and N12 are partial; N24 (b) is recorded, not a pass; the P10 hosted gate; the revision-5 items above; for property 3, a device-state snapshot before and after each further poll, because `p12_3_answered_exactly_once` checks "changes nothing" only on the caller's buffer after the further read poll (review 7, RQ7-2); the open rows of the CODE-DELTA (revision 5) **Status (24 November 2026, read at main `9dfff70c`, task S-20261124-6):** the revision-5 items are implemented on `main`: the property-4 deadline mode and property-5 quarantine and offline state of `SimDisk` (`p12_4_deadline_answers_and_resets`, `p12_4_deadline_decided_by_ready_time`, `p12_5_quarantine_and_offline_until_reboot`); the stray-poll counter with its tripwire (`stray_poll_tripwire`, `q6_extra_poll_after_result_is_counted`); the device-state snapshot before and after each further poll (`further_poll` in `tests/sim_contract.rs`, RQ7-2); the optional U+009F and U+00A0 label cases (`tests/catalog.rs`); the optional N35 sub-case that pins the checked step-4 copy (`n35_sealed_resolution_uses_checked_copy`); the production archive verifier for N7 and N10 (`tests/real_verifier.rs`, PR #505). `sim_contract` (16 tests) and `broker` (11 tests) pass locally with Rust 1.100.0. Still open: N33 (ii) (optional; no zero-byte variant found), the hosted P10 lint (local run exit 0; coordinator request C3), N24 (b) as recorded, not a pass; the CODE-DELTA (revision 5) table below is the historical record at `f993a1a3` |
| R3 | `tools/destination_disk.py` independent oracle | Python unit tests: parses valid copies, refuses each invalid rule; a test-only encoder for crafted images (N14's second destination ID, N4, N30), whose output the oracle's own parser accepts or refuses as intended. Revision 3 adds: the oracle decisions of section 7 (scope, image read errors, extents); the record rules of 2.6 (N34); a cross-check of images written by the R2 core, run by a hosted gate (a lead-owned workflow change); and a reverse differential, in which the oracle's crafted invalid images are mounted by the R2 core and the two verdicts compared; both are required before R5. **Status (revision 3):** implemented on draft PR #413, unmerged *[PR #413]* (host only; AI-reviewed, not human-audited); at `37b7bd0c` the cross-check had run only locally (7 images, agreement) and was skipped in hosted CI, and the reverse differential did not exist. Revision 4 states the limit of both comparisons: mount state, header and records, not the refusing rule (section 7, "Cross-check limit"; P11). **Status (revision 4):** at `e52cb865` *[PR #413]* the cross-check agrees on 30 images, locally only; it is still not run in hosted CI; the reverse differential still does not exist. **Status (revision 5):** at `f993a1a3` the oracle is unchanged by the O17 (a)/O18 round (the content check and the remount rule are protocol rules, not image rules); CODE-REVIEW-O17-O18 reports the cross-check agreeing on 30 images, run locally only *[PR #413]*; there is still no hosted cross-check and no reverse differential **Status (24 November 2026, task S-20261124-6):** the reverse differential exists, host only: `ReverseDifferential` in `tests/test_volume_destination_disk.py` builds 182 crafted images (each single-rule invalid copy of 2.6, N4 and N34 twice, and beside a valid copy in each slot order; the blank, reserved-sector, N8, N14, N25 and N30 mount cases) and, with `DEST_REVERSE_OUT` set, writes each with the oracle's verdict; `services/destination/tests/reverse_differential.rs` (`DEST_REVERSE_IN`) cold-boots the R2 core on every image and requires the same state, header and records (section 7, "Cross-check limit"). Locally all 182 agree and the 30-image forward cross-check agrees; a changed oracle verdict and a core mutation that drops the header-padding rule are both detected. Neither direction runs in a hosted gate yet: that is a lead-owned workflow change and stays an R5 precondition |
| R4 | Volume service read-only and create-only endpoint bindings (separate spec) | N19, N20 host tests; existing volume and archive host tests unchanged |
| R5 | Preconditions: O4 (a) approves K5 as per-device broker state (K4 is not needed for R5); the owner amends ADR-S09-DUALDISK-20261119 B3 and B5 (O4 (b)); O4 (c) is decided and its option is in place (section 7); the test profile enables `monotonic-clock` for the -8 bound (2.7; feature combination unchecked); (revision 3) the service links `volume_core::archive::verify` (2.3, "Archive verifier"); test hooks such as N29's exist only behind a test-only build feature that no native or product profile enables; the R3 reverse differential exists and the R3 cross-check runs in a hosted gate (review 4 OP-7); the per-device broker meets the broker contract of 2.7, including the K5 delta of property 3: a further poll after a read ended by a reset returns -1 (4.3 K5; review 9, OPT9-3; review 10, RQ10-2). Content: per-device broker state, destination service binary in a test-only profile on disk 1 that follows the once-per-boot grant rule (2.1), and a scripted test-only dialog driver for the control lane that sends FORMAT and BIND and transfers lane SEND to the portal (4.3) | native stages `dest-format`, `dest-export`, `dest-export-2`, `dest-restore` and `dest-grant-once`, each lane bound by BIND with the destination ID that INFO reports after `dest-format`; N1, N2, N16, N18 counted and rights-probed (the driver's handles, including its control-reply RECV, as the dialog's in 5.1); N27 natively with `DEST:HANDLES=`; N32 natively with `DEST:GRANT:*`; at run time each lane handle transferred to the portal is SEND only (`check_endpoint`); every runner with `-no-reboot`; `dest-restore` is test-only and does not wait for R4; the per-device broker admits no disk-1 request between a disk-1 timeout and the acknowledged reset (2.7 completion fence), and a disk-1 submission is admitted while a disk-0 request is in flight, both shown by native timeout fixtures **Status (24 November 2026):** O4 (a)–(c) decided by the owner (section 8.1); the production archive verifier is linked on the host (PR #505); still open: the hosted run of the R3 cross-check and reverse differential (both exist on the host, PR #509), native K5 timeout fixtures, the `monotonic-clock` test profile and every native destination stage. **K5 step 1 (24 November 2026, PR #509, `04b06de2`):** per-device `Policy`, error, reset flag and deadline, completion buffer and publication time in the volume broker, and the property-3 further-poll rule; independent AI security review K5-REVIEW-1 returned REQUEST_CHANGES with one required finding (S3 quiescence with an absent lane), fixed in `04b06de2`; a later CodeRabbit finding (property 3 after a failed reset) fixed in `1a3924de`; with `1a3924de` the native volume gate passed 32 of 32 stages and the H05 power gate 12 of 12 stages (S3, S3 re-initialisation and held completion included), run locally. Not yet shown natively: a disk-1 request admitted while disk 0 is in flight, and no disk-1 request admitted between a disk-1 timeout and the acknowledged reset; these need a new stage and a pin change by the CI owner (C4, C5) |
| R6 | UNBIND, revocation and rebinding through the scripted test-only dialog driver of R5 | QEMU levels of N3, N5, N6, N7, N8, N10, N14, N15, N17 and N22 with stages `dest-revoke`, `dest-foreign`, `dest-full`, `dest-replay`, `dest-corrupt`, `dest-scope`, `dest-sender` and `dest-nonarchive`; crash stages `dest-cut-append`, `dest-cut-seal`, `dest-cut-mirror` and `dest-after-cut` (N11, N12) at QEMU level as process kills without cache loss, with the kernel-counted flush count and order asserted at each cut marker (section 7) |
| R7 | Trusted dialog surface and remembered permission (after O1 and O12; O14, O15 and O17 decided on 23 November 2026) | UI checks per `ui/AGENTS.md`; revocation of a remembered permission revokes live lanes and their volume-side bindings; the portal's trailer comparison of the decided O17 (b), including how it keeps the exported trailer across a reboot (2.7; N37 (c)); every label rendered in an isolated bidirectional context (first-strong isolate) and visibly delimited from trusted text, because 2.3 allows format characters such as U+200E, U+200F and U+061C (review 6 OPT6-2). **Status (revision 5):** not started; PR #413 has no portal, so the O17 (b) check and N37 (c) are open here, not on PR #413 |
| R8 | After R4 and O11–O13 (O14–O18 decided on 23 November 2026; the O4 items are R5 preconditions): independent security review of the implementation, then an owner activation decision | all matrix rows pass natively where marked QEMU; N19 and N20 pass natively, not only on the host; FORMAT's entropy draw has native evidence (section 7); under O13 (decided on 24 November 2026), both the once-per-boot rule passes N32 and the kernel write latch passes its own native test; the decided O17 (b) portal check is implemented (R7) and N37 (c) passes with the portal, so that activation cannot skip the second check (revision 5, review 7 OPT7-6); review closed. **Status (revision 5):** not started; the activation no longer waits on the O17 or O18 decision, but it requires the R7 implementation of O17 (b) above **Status (24 November 2026):** O11–O13 decided; R8 now also requires the O12 owner-binding design checked against the profile, key and encryption contracts, the O13 kernel write latch with its own native test, and native evidence of the real entropy route (O4 (c)) |

Until R8, every grant remains test-only and S09 remains open; this document
changes no score.

## 9. Review record

Review 1: [SPEC-REVIEW-1.md](../verification/s09-destination-service-20261123/SPEC-REVIEW-1.md)
(independent AI review of revision 0 at `bc3f63a1`, checked against `main`
`035b44e5`; REQUEST_CHANGES, 13 required items). Each item was checked against
the primary sources before it was resolved; where that check narrowed an item,
the narrowed fix was applied and the remainder is recorded here as refuted or
not required. An AI review and its verification are not a human audit.

| # | Review item | Resolution | Where |
|---|---|---|---|
| 1 | Failed catalog read at mount | Resolved. A read error on sectors 0–15 is a failed mount, never an invalid or zero sector. Not required: the review's mirror before the next CREATE; with a read error terminal, the next commit already targets the lower-sequence slot and then mirrors | 2.6 mount rule and failed-mount state; 2.7 step 4 and "Why step 5"; N8 |
| 2 | Safe path covers only `Unknown` | Resolved. Trigger is any device `Io`, `Timeout` or `Unknown` (or persistent -8) on any destination write or flush; same provisional-until-cold-boot rule; reply codes per edge; readback `Timeout` edge added | 2.5; 2.7 failure paragraph and safe path; N13; N24 (d); O9 |
| 3 | Safe path not implementable as written | Resolved in revision 2 (review 2, F1). The grant is manifest-only and non-transferable; the single instance's latch carries read-only for the boot only under the proposed once-per-boot grant rule (2.1, owner decision O13, test N32); recovery is a reboot; `-no-reboot` in every destination runner; warm reboot stays under O9. Corrected: revision 1 called the kernel write latch (3a) refuted because "on `main` no profile can re-grant block authority in the same boot". That claim is false: `m5.rs:260-279` re-creates a block grant on every kernel-driven restart (`:784-794`). The absence of TRANSFER blocks only user-space delegation. The kernel latch is therefore not refuted; it is not required only on the condition of the proposed rule, and it remains the owner's alternative under O13. O8 restart route withdrawn on the same condition | 2.1; 2.6; 2.7 "Carrier of the read-only state"; section 7; O8; O9; O13; N32 |
| 4 | Kernel -8 wedge | Resolved; corrected in revision 2 (review 2, F5): K5 is per-device broker state, not only a per-device `Policy`; the service consumes each result before the next submission, so -8 on a disk-1 submission means only disk 1's own reset; the service treats -8 beyond 2 × `RESET_TIMEOUT` as a device `Io` and a pending poll beyond `TIMEOUT` + 2 × `RESET_TIMEOUT` as `Unknown` or `Timeout` | 2.7 completion fence; 4.3 K5; 5.2; N26 |
| 5 | Transferred capabilities on requests | Resolved. Check 1 of 2.2 closes every received handle and replies `Invalid`; the slot-exhaustion stall is recorded as a residual dialog-fault risk; `DEST:HANDLES=` added. Refuted: the portal-TRANSFER half; on `main` the portal's volume and archive handles carry no TRANSFER, and 4.1 already excludes it | 2.2; 4.1; section 7 evidence lines; N27 |
| 6 | No endpoint for control-lane replies | Resolved. Dialog-owned control-reply endpoint (dialog RECV, service SEND); handle lists corrected (block handle added, `Entropy` no longer counted twice) | 2.2 replies; 4.3; 5.1; N18 |
| 7 | N18 checks counts only | Resolved: N18 probes exact rights and scope disjointness, with mutations. Proposed, owner decision O11: v1 requires a separate device | 1; 2.1 device rule; N18; O11; R8 |
| 8 | Rebind replay | Resolved. Per-lane maximum generation for the instance; BIND with a generation not above it returns `Stale`; INFO reports the maxima | 2.3 BIND and INFO rows, "Grant generations"; 4.3; N17 |
| 9 | Conflicting status codes | Resolved; completed in revision 2 (review 2, F2–F4, F14): binding limits all in check 6, one state table in check 7, explicit check-2 fields. One total check order; requests rejected by checks 1–6 do not advance the high-water mark; `Corrupt` refusal code `Corrupt` -18; absent device is the failed-mount state with INFO reporting `NoDevice`; `NoDevice` only from BIND or INFO | 2.2 check order; 2.6; 5.3; N14; N28 |
| 10 | Catalog not validated before write | Resolved. Step 4 runs the mount validator and the extension check before any write; failure replies `Corrupt` for that SEAL only. Refuted: the review's three examples are unreachable under the spec's own rules (archive verify bounds the length; the entry-path scope rule bounds `next_free`; CREATE bounds the label length); the guard remains against encoder defects. The optional APPEND cap at 525 328 bytes was not added (superseded in revision 3, L8: the cap returns `Range` -3) | 2.5; 2.7 step 4; N29 |
| 11 | Native cuts cannot detect a missing barrier | Resolved. QEMU cuts relabelled "process kill, no cache loss"; flush count and order asserted at each cut marker; flush-order and cache-loss acceptance host-only at R2; `blklogwrites` replay oracle optional later research | section 6 levels; N11; N12; section 7; R2; R6 |
| 12 | D3 gaps | Resolved: R8 requires R4 and native N19/N20; ending or revoking a grant or remembered permission ends the portal's volume-side binding. Proposed, owner decision O12: single-profile v1 or an owner principal; R7 and R8 blocked until decided. Refuted: gating the test-only R5 restore stage on R4; the trusted-portal dependency is already disclosed in 4.2 | 3.3; 4.1; O12; R5; R7; R8 |
| 13 | Fixture boundary and stale labels | Resolved: labels rebased on `main` `035b44e5` (PR #396 merged as `8a7a3594`); pins and R5 precondition rewritten. Proposed, owner decision O4 (b): amend ADR-S09-DUALDISK-20261119 B3 and B5 before R5 | header; 2.7; 4.3 K4; 5.2; 5.3; section 7; O4; R5 |

Review 1, optional items taken: READ bound and straddling read, reply echo checks (2.2,
2.3, N3); no blocking reply send (2.2) with idempotent UNBIND (2.3, 4.3); mount
extension rule (2.6, N30); contiguous extents (2.6); op × lane table, CREATE
once per binding and LIST scoping (2.3, N31); the cloned-catalog input to O1
(3.3, O1). Not taken: the dialog revoking after an UNBIND timeout together with
a kernel write latch, because under the proposed once-per-boot grant rule
(2.1, O13) no in-boot re-grant path exists; the kernel latch remains the
owner's alternative under O13; a sequence increase of 2 on a boot's
first commit, because it conflicts with the sequence + 1 extension check and
targets only a cross-boot late write that the O9 assumption already covers.

Review 2: [SPEC-REVIEW-2.md](../verification/s09-destination-service-20261123/SPEC-REVIEW-2.md)
(independent AI re-review of revision 1, checked against `main` `035b44e5`;
REQUEST_CHANGES, required findings F1–F6, optional F7–F15). Each finding was
checked against the spec and `main` before it was resolved: F1 against
`kernel/src/m2/m3/m5.rs:260-279` and `:784-794`; F5 against `volume.rs:24-25`,
`:122-131`, `:900-912` and `:964` and the virtio-core `Phase` enum; F6
against `create_entropy` in `kernel/src/m2/m3/`, `tools/run.py:156`,
`tools/run_keyring.py:32` and `tools/run_volume.py:322`; F10 against
`capability-core` `send`, `resolve` and `maintain`; F13 against
`volume.rs:745-753`; F15 against `kernel/src/m2/m3.rs:2366-2370` and
`kernel/Cargo.toml`. All fifteen findings hold. Revision 2 adds owner questions
O4 (c) and O13 and does not decide them. An AI review and its resolution are
not a human audit.

| # | Finding | Resolution | Where |
|---|---|---|---|
| F1 | Latch argument rests on a false claim about `main` | Resolved. Once-per-boot grant rule stated by scope as *[proposed; owner decision O13]*, with the kernel write latch as the owner's alternative; new N32 (native, kernel-counted) kills the M5 restart pattern; section 9 item 3 corrected to the conditional basis | 2.1; 2.6 failed-mount state; 2.7 "Carrier of the read-only state"; 4.3 K5; O8; O9; O13; N32; R5; R8; section 9 item 3 |
| F2 | N13 conflicts with the check order | Resolved. All binding limits are in check 6, including the *unknown* export binding (INFO and LIST only, by check 6) and a wrong entry on an operation lane (N31); N13 rewritten per lane and binding for both remount outcomes, with a check-6/7 swap mutation | 2.2 check 6; 2.7 failure paragraph; N13 |
| F3 | Header table contradicts BIND/UNBIND `count` and SEAL `offset` | Resolved. `offset`, `count` and `aux` rows corrected; a field-rule table assigns every header field of every operation to check 2, and check 2 names the entry check (check 6) and the catalog-dependent checks left to the operation | 2.2 header table, field-rule table and check 2 |
| F4 | Reachable states without a gate | Resolved. One state table in check 7 with INFO encoding (blank: `NotFound` -19, destination ID zero); the pre-fence interval is "remount pending", in which no request is received; a FORMAT write or flush failure enters the failed-mount state; blank-state replies defined; N8, N24 (a), 5.3 aligned | 2.2 check 7; 2.3 INFO row; 2.6 blank, failed-mount state, Format; 2.7; 5.3; N8; N13; N24 |
| F5 | -8 bound false-triggers on shared broker state | Resolved by option (a): K5 is per-device broker state (`Policy`, `resetting`, `reset_deadline`, `error`, `active`, data buffer); the service consumes each result before the next submission; poll -8 (pending) is bounded separately; N26 (d) and (e) added; R5 also shows that disk 0 in flight does not block disk 1 | 2.7 "Meaning of kernel -8"; 4.3 K5; 5.2; O4 (a); N26; R5 |
| F6 | PCI-slot/entropy conflict without a home | Resolved as an R5 precondition, classified as a test-profile matter, not K4: option (a) test-profile kernel change, or option (b) a deterministic test-only destination ID *[proposed; owner decision O4 (c)]*; 5.1 and N18 adjusted; N14 uses a harness-written second ID; native entropy evidence becomes an R8 precondition | section 7; 5.1; N14; N18; O4 (c); R5; R8 |
| F7 | N28 overclaims | Taken. Kill claim narrowed to distinguishable swaps; equivalent mutants listed | N28 |
| F8 | Emitting layer of flush evidence | Taken. `DEST:IO:*`, flush order and the cut marker from the kernel broker keyed by object key; `DEST:HANDLES=` and `DEST:GRANT:*` from the kernel `Engine` | section 7 evidence lines; N12 |
| F9 | Route coverage gaps | Taken. Every host level assigned to R2 (or R4), every QEMU level and stage to R5 or R6; three QEMU stages added for N3, N22 and N10 | section 7 stages; R2; R5; R6 |
| F10 | Dialog cannot revoke after the service exits | Taken. A kernel `Revoked` -10 or `Closed` -9 on the UNBIND send permits the revoke (corrected after review 3: `Engine::exit` revokes the dialog's delegated nodes, so -10 is the normal case); N6 covers it | 4.3 "To revoke", N6 |
| F11 | Spurious replies to the portal | Taken. An operation-lane request from a sender other than the portal gets no reply; N22 and N27 observe that, natively through the kernel-reported length of the portal's reply queue (`DEST:PORTAL:REPLYQ=`), not through the service's `DEST:DENIALS=` | 2.2 Replies; N22; N27 |
| F12 | Decay window after a crash between barriers 2 and 3 | Taken as an accepted residual; a mount-time mirror is declined, because it adds a third catalog write path exactly where the safe path forbids writes | 2.7 "Why step 5" |
| F13 | Inaccurate `archive_grant` citation | Taken. Cited as a precedent for comparing scopes, in 2.1 and N18 | 2.1 device rule; N18 |
| F14 | Self-contradictory high-water rule | Taken. A request refused by checks 1–6 leaves the mark unchanged; one that passes checks 1–6 advances it, also when check 7 refuses it; N17 tests both | 2.2; N17 |
| F15 | Hidden time-source dependency; UNBIND latency | Taken. Generic syscall 3 under `monotonic-clock` named, its combination with the volume-fixture features unchecked, listed as an R5 precondition; UNBIND latency stated as at most two bounds in N26 (b) | 2.7 "Time source"; N26; R5 |

Review 2, optional items declined: none in full. Partly declined: in F12,
the mount-time mirror (reason above); in F5, option (b), widening the bound to
`TIMEOUT` + `RESET_TIMEOUT`, because it would only mask shared broker state
that the product must not have and would delay every genuine reset failure by
five seconds.

Review 3 ([SPEC-REVIEW-3.md](../verification/s09-destination-service-20261123/SPEC-REVIEW-3.md),
APPROVE, optional findings only):

| Finding | Subject | Resolution | Where |
|---|---|---|---|
| R3-1 | F10 named the wrong kernel error | Taken: `Revoked` -10 or `Closed` -9 permits the revoke; N6 case (b) added | 4.3 "To revoke", N6, F10 row |
| R3-2 | N32 kill observation did not hold | Taken: counts are 0 after the exit and stay 0; the kill is a count above 0 | N32 |
| R3-3 | Leftover unconditional "no in-boot re-grant" claim | Taken: made conditional on O13 | this section, review 1 optional items |
| R3-4 | Source of the portal's process ID unspecified | Taken: the manifest writes it into the service's start context; N18 checks it | 4.3 |
| R3-5 | Consistency nits | Taken: (a) table reference and the order within check 2; (b) failed-mount row excludes the FORMAT scan; (c) kernel -14 and -13 on writes mapped in 5.3 (superseded in revision 3, L1 and L2: on submission both reply `Io` -12; a completion follows the 2.5 table); (d) K4 removed from the R5 preconditions. Not taken: (e) native N19/N20 are already named in R8; (f) the bound wording is left as revision 2 states it | 2.2, 5.3, 8.2 |

**Revision 3 record.** The inputs are the R2 implementation of
`services/destination`, its independent AI review and fix pass, and the R3
oracle `tools/destination_disk.py`, its independent AI review and fix pass,
all on draft PR #413 at `37b7bd0c` (host only, unmerged). Their findings were
deduplicated into the items L1–L29 below. Review-finding names of those passes
(for example the R2 review's O7–O9) are not owner items of 8.1. Each choice
below is reasoned from the design, the threat model or a fail-closed argument,
not taken from the code; where the PR #413 code differs from the chosen text,
the code must change, and the specification was not adapted to it. No
resolution widens a capability, weakens the check order or adds trust in disk
bytes. These resolutions are an AI revision, not a human audit. Review 4
(below) reviewed them; the rows for L2, L5, L10, L22 and L29 were corrected
after it.

| ID | Resolution type | Section changed | Reason |
|---|---|---|---|
| L1 | normative fix | 2.5 "Device results"; 5.3 | 2.5 allowed `Timeout` for a write, 5.3 `Io`. One mapping: `Timeout` is a reply only to a read; a write or flush that was admitted without a definite result is `Unknown`, which claims neither outcome; a refused submission (-13, -14, -8 past the bound) is `Io` |
| L2 | normative fix | 2.5 "Device results"; 5.3 | 5.3 contradicted itself and the N14 kill. A submission refused with kernel -14 after the boot mount can only be a failed re-initialisation (no runtime attachment, K1), a device fault: `Io`; a completion -14 follows the 2.5 table (`Unknown` -16 on a write or flush, review 5 RQ5-2). `NoDevice` stays a reply of BIND and a status reported by INFO in the failed-mount state, so it keeps one meaning for the portal. Chosen after reading the R2 mapping and `n14_wrong_foreign_absent_device`, which tests only the absent-at-mount case. Code changes required: the -14 arms of `submit_error` and `poll_error` (`device.rs`) and the R2 test `kernel_offline_on_write` (`tests/broker.rs:144-148`), which asserts `NoDevice` for -14 on a write submission *[PR #413]* (review 4 OP-9) |
| L3 | clarification | 6 (N13) | 2.8 allows three operation lanes; the third lane is bound for verify in one run and for restore in another |
| L4 | clarification | 2.6 mount rule; 6 (N29, N30) | For two valid copies the record prefix with contiguity implies `next_free` not lower, and both formatted counts equal the scope; the clauses stay as defensive, equivalent checks. N30 uses next entry ID lower; N29's `next_free` tamper is refused by contiguity |
| L5 | normative fix | 2.6 validity list; 6 (N34) | 2.3 already forbade entry ID 0 and labels outside 1–32 bytes; the copy validator now states both, so 2.3 and 2.6 agree. After review 4 (RQ-3), the list also states copy sequence ≥ 1, which was only in the header field description; both implementations already refuse sequence 0, and N34 gains a zero-record sequence-0 case |
| L6 | proposed owner decision | 2.3 (`valid_name` defined in full); 2.6 "Label content at mount"; 8.1 O14 | A policy choice between failing closed on any label that CREATE could not have written (and keeping such labels off the trusted dialog) and the availability of the other entries. Proposed: full `valid_name` at mount |
| L7 | normative fix | 2.6 (sealing sequence defined, validity list); 2.7 step 4 | The sealing sequence was undefined. Defined as the sequence of the first copy that lists the record; FORMAT is sequence 1 without records and each SEAL adds one record at the new sequence, so ≥ 2, ≤ the copy's sequence and strictly ascending follow; a record outside them shows damage, a crafted copy or an encoder defect. The bounds survive a later deletion (O3) |
| L8 | normative fix | 2.3 (APPEND row and rules) | An APPEND beyond 525 328 bytes can never seal. `Range` -3, nothing appended, entry stays `writing`; `Full` already means "destination full" (3.2, O3). Supersedes review 1 item 10, where the APPEND cap "was not added" |
| L9 | proposed owner decision | 2.2 check 6; 2.3; 8.1 O15 | Whether a refused CREATE uses up the binding is a permission-policy choice. Proposed: only a CREATE answered `Ok` counts; a refused CREATE allocates nothing, so a retry cannot create a second entry |
| L10 | clarification | 2.3; 6 (N36) | Order after check 7: label content `Invalid`, then `Busy`, then `Full`; request-only refusals precede state-dependent ones as in 2.2; label content is an operation check, not check 2. `Busy` and `Full` are mutually exclusive (corrected after review 4 RQ-1: while any lane is writing, the committed catalog is the one its CREATE passed, so neither 16 sealed entries nor zero free sectors can hold), so their order is not observable and fixes only the code path; N36 tests each after an invalid label separately |
| L11 | clarification | 2.6 Format | The ID is drawn after the scan and before the first write; a failed draw replies `Io`, writes nothing and leaves a blank candidate, because no write was issued and no device state is uncertain |
| L12 | clarification | 2.3 (VERIFY, READ rows); 2.5 "Device results" | Read failures follow the mapping (`Io` or `Timeout`); VERIFY never answers `Corrupt` for a read failure, which is no evidence about stored bytes; no read failure starts the safe path |
| L13 | clarification | 2.3 (LIST row) | The cursor is a 0-based record index of the mounted catalog; the next cursor is the index of the next visible record, or 0 for end, which is unambiguous because 0 cannot follow a returned record |
| L14 | clarification | 2.7 "Meaning of kernel -8" | The poll bound runs from admission, where the broker's own deadline starts; measuring from the first attempt would turn a legitimate reset wait into a false `Unknown`. The submission bound runs from the first attempt |
| L15 | clarification | 7 (evidence lines) | `DEST:DENIALS=` counts refusals by checks 1–6 only, replied or not; check-7 and operation refusals are state outcomes, not denials of authority or framing |
| L16 | clarification | 2.3 (INFO row) | Free sectors = formatted count − committed `next_free`; a writing entry's sectors are allocated only by the SEAL commit |
| L17 | clarification | 2.3 "Archive verifier"; 2.7 step 3 | Not an extra rule: `archive::verify` over exactly `length` bytes refuses bytes after the trailer (`archive.rs:366-377` *[main]*), so an accepted entry ends with its trailer. The text now says "first `length` bytes", not "the extent", which includes padding |
| L18 | normative fix | 2.7 "Broker contract required of K5"; 6 preamble | The service's bounds and the completion fence rely on broker properties (one request, -8 while pending including after the service's poll bound, one answer per request, reset and offline codes) that were implicit; K5 and every host model must provide them |
| L19 | route or verification-plan item | 6 preamble | The reset behaviour of the volatile cache is unknown (O9); host rows that pass through a reset must hold under "kept" and "dropped", not only the model R2 chose |
| L20 | route or verification-plan item | 2.3 "Archive verifier"; 8.2 R2, R5 | The service links `volume_core::archive::verify` (`no_std`, depends only on `catalog-core`) so that it accepts what the portal accepts; a stand-in is test-only; rows that verify count only for the linked verifier; real verifier is an R5 precondition |
| L21 | route or verification-plan item | 6 (N29); 8.2 R5 | Test hooks that alter encoded copies must not exist in a native or product build: behind a test-only build feature, an R5 precondition |
| L22 | proposed owner decision | 2.2 check 7 table and text; 2.6 failed-mount state; 2.7; 6 (N35); 8.1 O16 | A remount that reads all zero contradicts a catalog mounted earlier in the boot; whether that is the failed-mount state (proposed; binding stays *unknown*, no new table cell) or a latched blank state is a policy choice. Both write nothing. After review 4 (OP-5), the proposed rule also covers a remount that reads a valid catalog other than the committed copy or that copy plus exactly the provisional record, and a remount ending `Corrupt` leaves the binding *unknown* |
| L23 | clarification | 2.6 (layout table, mount rule) | Sectors 8–15 are never written and only the blank test reads them; a validity rule would make a valid destination `Corrupt` over unused bytes and would not detect a foreign writer in general |
| L24 | normative fix | 2.6 validity list | The length bound 80..=525 328 was in the field description only; it is now a stated validity rule, as both implementations already apply |
| L25 | clarification | 2.6 | "FNV-64" is 64-bit FNV-1a with the standard offset basis and prime, as `catalog_core::wire::checksum_update` and the v1 archive use *[main]*; the trailer FNV is the last 8 bytes of the entry |
| L26 | route or verification-plan item | 7 (oracle decisions) | The oracle's extent check is structural (trailer, padding) and not an archive acceptance; native stages that assert verification parse the first `length` bytes with `tools/volume_archive.py` |
| L27 | clarification | 2.6 (preamble); 7 (oracle decisions) | Sector numbers are scope-relative; bytes outside the scope are not part of the destination; the harness passes the scope's first sector and count to the oracle |
| L28 | clarification | 7 (oracle decisions) | Consistent with the 2.6 rule that a read error is not an invalid copy: a short or unreadable image is the failed-mount state, and an extent past the image end is unreadable, never zero |
| L29 | route or verification-plan item | 8.2 R3, R5; 6 (N34) | Agreement of the two implementations rests on local runs in one direction only. A hosted cross-check (lead-owned workflow change) and a reverse differential (oracle-crafted invalid images mounted by the Rust core), both before R5 (the cross-check since review 4 OP-7) |

Outside the ledger, revision 3 also states the checked arithmetic of 2.7
step 4 (sequence and next entry ID at the u64 maximum), which the R2 review
found as a required defect and the R2 fix pass already implements, and adds
its case to N29 (with its own expected result after review 4 RQ-2).

**Owner decisions of 23 November 2026.** The project owner accepted the
proposals of O14, O15 and O16 ("Yes, accept all three", in the S09 lead
session). Consequences for PR #413: O14 — the R2 copy validator and the R3
oracle apply `valid_name` (2.3) in full to every record's label at mount, and
the oracle's pinned "label content unchecked" test is replaced by refusal
tests (N34); O15 — no code change, the R2 core already follows it, and N36
(c) asserts it; O16 — the R2 remount after the 2.7 trigger keeps the
committed copy and enters the failed-mount state, export binding *unknown*,
when the remount gives a blank candidate or a valid catalog that is neither
the committed copy nor that copy plus exactly the provisional record, and
N35 asserts that state. Revision 3 text is otherwise unchanged by these
decisions.

**Review 4.** SPEC-REVIEW-4 is an independent AI security review of revision 3
(diff base `5d97cd76`, implementation compared at `37b7bd0c`; REQUEST_CHANGES,
3 required and 10 optional findings). It found the ledger coverage complete
and the security floor intact. Each finding was checked against this text and
the PR #413 code before it was applied; none was refuted. The review file is
stored verbatim as `docs/verification/s09-destination-service-20261123/SPEC-REVIEW-4.md`.
This record is an AI revision and not a human audit. Review 5 re-reviewed the
changes below and found each finding resolved.

| Finding | Disposition | Where |
|---|---|---|
| RQ-1 | Fixed: `Busy` and `Full` stated as mutually exclusive, with the reason; the determinism argument removed; N36 (a) split into (a1) another lane writing and (a2) nothing writing with no free sector or 16 sealed entries; L10 reason corrected | 2.3; N36; L10 row |
| RQ-2 | Fixed: the u64-maximum case has its own expected result (every SEAL `Corrupt` -18, no catalog write, entry discarded, no `Corrupt` mount state, INFO and LIST unchanged); "a later export seals" kept for the hook cases; 2.7 step 4 states that such a catalog never commits again | N29; 2.7 step 4 |
| RQ-3 | Fixed: "copy sequence ≥ 1 †" added to the validity list; zero-record sequence-0 case added to N34; L5 row extended. The remaining list was checked against `catalog::decode` and the oracle's `decode_copy`; no other rule lives only in a field description | 2.6; N34; L5 row |
| OP-1 | Fixed: the submission row covers every submission result other than 0 and -8 | 2.5 "Device results" |
| OP-2 | Fixed: the meaning of `NoDevice` restated as "a mount, remount or FORMAT met kernel -14" (INFO in the failed-mount state) or BIND; behaviour unchanged | 2.5 "Reasons" |
| OP-3 | Fixed: the step-4 pre-write check also requires the new record's sealing sequence to equal the new sequence; not added to the mount rule, where it would refuse valid copies after an O3 deletion; the 2.6 claim corrected; N29 case uses a fixture that the validity rules accept | 2.6; 2.7 step 4; N29 |
| OP-4 | Fixed: the O14 parenthetical states what the oracle does at `37b7bd0c`. The test encoder's default sealing sequence of 1 (`tools/destination_disk.py:219`) must change with the R3 code change, since every crafted copy with records is otherwise invalid under revision 3 | 8.1 O14 |
| OP-5 | Fixed: a remount ending `Corrupt` leaves the binding *unknown* (normative; matches the R2 code); a remount that reads another valid catalog than the committed copy or that copy plus exactly the provisional record is folded into the O16 rule (failed-mount state, binding *unknown*), which requires keeping the committed copy for the comparison; N35 extended with (b)–(d). O16 stays undecided (later decided on 23 November 2026, 8.1) | 2.2 check 7 table; 2.6 failed-mount state; 2.7; 8.1 O16; N35 |
| OP-6 | Fixed: the N33 case of a read timeout while another lane holds unflushed APPEND writes is added and run under both reset models; the outcome differs per model (SEAL `Ok`, or `Corrupt` -18 at the readback), the invariant "never a listed entry that fails VERIFY" holds under both | 6 preamble; N33 |
| OP-7 | Fixed: the hosted cross-check is an R5 precondition, beside the reverse differential | 8.2 R3, R5 |
| OP-8 | Fixed: the -14 → `Io` mapping is to be decided again if K1 is approved | 2.5 "Reasons" |
| OP-9 | Fixed: `kernel_offline_on_write` (`tests/broker.rs:144-148`) and the -14 arms of `submit_error` and `poll_error` named as required code changes | L2 row |
| OP-10 | Fixed: the safe-path sentences of 2.5 except mount and remount FLUSH and FORMAT writes and flushes (failed-mount state); readback `Io` added to the diagram; the poll bound measured "from the admission of the request"; the trailer FNV worded as the value stored in trailer bytes 8..16; *[PR #413]* applied to the code claims of O14–O16 and the R2 and R3 status; the check-7 table uses the long form of the O16 label | 2.5; 2.6; 2.7; 2.2; 8.1; 8.2 |

**Review 5.** SPEC-REVIEW-5 is an independent AI security re-review of
revision 3 after the review-4 fix pass (diff base `5d97cd76`, implementation
compared at `37b7bd0c`), stored verbatim as
`docs/verification/s09-destination-service-20261123/SPEC-REVIEW-5.md`. It found
every review-4 finding resolved, the ledger coverage complete (one row per
L1–L29) and the security floor intact, and returned REQUEST_CHANGES with two
required findings. The lead answered them as below; these answers are not
re-reviewed.

| Finding | Disposition | Sections |
|---|---|---|
| RQ5-1 | Fixed: N36 (a1) and (a2) are marked as holding under the O15 rule, with the alternative's result for the second CREATE (`Access` -5 at check 6), as case (c) already did. Superseded in revision 4 (P4): N36 states the O15 outcomes only | N36 |
| RQ5-2 | Fixed: 5.3 and the 2.5 "Reasons" paragraph now apply `Io` to a submission refused with kernel -14 or -13 after the boot mount, and say that a completion result follows the "Device results" table | 2.5; 5.3 |
| OPT-1 | Fixed: the 2.6 sentence now reads "the committed copy's highest sealing sequence (1 if it has no record)" | 2.6 |
| OPT-7 | Fixed: the section 6 preamble covers revisions 1–3 | 6 |
| Lead check | A read-only verification of the RQ5-1 and RQ5-2 answers found both resolved and four low optional follow-ups (the L2 and R3-5 rows, the 2.5 "Reasons" sentence order, the FORMAT exception in 5.3); all four are applied | 2.5; 5.3; 9 |
| OPT-8 | Partly fixed: reviews 4 and 5 are stored and linked; the N36 (b) scope remains implicit | 9 |
| OPT-2, OPT-3, OPT-4, OPT-5, OPT-6 | Not taken in revision 3: OPT-2 (byte comparison for the O16 match) concerns a misbehaving device, out of scope by 5.4, and goes with the O16 decision (superseded in revision 4, P2: proposed as O18); OPT-3 is harmless because the operative condition is "any result other than success"; OPT-4 is covered by the general rule that the code follows the table, and N33 detects each difference; OPT-5 is recorded by the review-4 table; OPT-6 (refusing CREATE at the u64 maximum) is a design option that fails closed either way and is left to the R2 code change | 2.5; 2.6; 2.7; N29 |

**Revision 4 record.** The inputs are the code changes of R2 and R3 for
revision 3 and for the owner decisions O14 and O16 on draft PR #413
(`4f1edb9c`, `e52cb865`; host only, unmerged) and their independent AI code
reviews: the review of the revision-3 code delta (findings F1–F7) and the two
reviews of the O14/O16 change (findings R-1, O-1, O-2, N-1), stored on PR #413
under `docs/verification/s09-destination-core-20261123/` as
`CODE-REVIEW-REV3.md`, `CODE-REVIEW-O14-O16-1.md` and
`CODE-REVIEW-O14-O16-2.md`. Their open points were deduplicated into P1–P13.
Mutant names in the table are those of the O14/O16 code round on PR #413 (B4,
B6, A10, A11, named in its ledger) and of its code reviews (R14, X6, which
survived there as equivalent mutants); they are not items of this document. As in revision 3, each choice is
reasoned from the design, the threat model or a fail-closed argument, not
taken from the code; where the PR #413 code differs from the chosen text, the
difference is listed in the CODE-DELTA table below. No resolution widens a
capability, weakens the check order or adds trust in disk bytes; the decided
text of O14, O15 and O16 is unchanged. These resolutions are an AI revision,
not a human audit, and are not yet reviewed.

| ID | Resolution type | Section changed | Reason |
|---|---|---|---|
| P1 | proposed owner decision | 2.5 (reply codes); 2.7 step 3, "Stale valid content" and the portal paragraph; 5.4; 6 (N33 (iii), new N37); 8.1 O17; 8.2 R7, R8 | A SEAL can commit a stale valid archive after a device fault (review F1). Integrity of a sealed entry is a security property, but where it is checked, in the service before the commit or in the portal after it, trades permanence of a wrong entry against a service change, so it is a policy choice. The limit is stated unconditionally; the fix, its reply code (`Corrupt`, no safe path, because nothing is outstanding and the extent is unallocated) and the N rows are conditional on O17. Threat model stated: accidental faults only; FNV-64 is not collision-resistant; a hostile-device defence needs a maintained standard hash and a separate decision |
| P2 | proposed owner decision | 2.6 failed-mount state (cross-reference); 2.7 (committed-copy equality clarified; O18 text); 6 (N35 (e)); 8.1 O16 status, O18; 8.2 R8 | Byte equality with the encoded copy changes which remounts resolve *sealed* under the decided O16, so it is proposed as O18 although stricter, with the decided text left unchanged. The committed-copy arm needs no decision: equality of a 2048-byte copy is byte equality, which for valid copies equals field equality. Answers code-review O-2 and review 5 OPT-2. Related to P1 only by subject: O17 compares extent bytes, O18 catalog bytes; each is consistent if the other is declined |
| P3 | clarification | 2.3; 6 (N34); 8.1 O14 status | "Control character" is the set of the referenced `catalog_core::wire::valid_name` *[main]*: General_Category Cc, U+0000–U+001F and U+007F–U+009F. The C1 controls are as unsuitable on the trusted display surface as the C0 controls, and naming the set lets the independent oracle apply it. Other Cf characters stay allowed; the dialog does not rely on labels (2.6, 5.4) |
| P4 | clarification | header; evidence labels; 2.2 check 6; 2.3; 2.6 (label content, failed-mount state); 6 (N34, N35, N36); 8.1 O14, O16 status; 8.2 R1, R2, R3, R7, R8 | The decisions of 23 November 2026 settled O14–O16; conditional "if accepted" and "under the alternative" wording is removed, the declined alternatives are named as declined, unconditional rules in those sentences are kept (the dialog does not rely on a label; a `Corrupt` remount leaves the binding *unknown*; the blank-plus-latch FORMAT refusal, P6), and the route status is brought to `e52cb865` (30 images locally; no hosted cross-check; no reverse differential; verifier stand-in) |
| P5 | clarification | 2.7 step 4; 6 (N29, N35) | For a copy that passes the validator, the sealing-sequence clause implies "exactly one new record"; the count clause stays as a defensive check and its removal is an equivalent mutant (B4/R14), listed in N29 and N35 |
| P6 | clarification | 2.2 (defensive rule); 2.6 failed-mount state; 6 (N35) | The rule is kept: it follows from the 2.7 rule that no write follows the trigger in that boot, costs one condition and keeps a later remount change from reopening a write path. Only its reachability is corrected: unreachable since O16; its removal is an equivalent mutant (B6), listed in N35 |
| P7 | clarification | 2.6 (label content); 6 (N34) | Under O14 `valid_name` refuses an empty label, so the "≥ 1" length bound is implied at the mount-state level (A10), and the oracle's length bound inside its content check is shadowed by its record check (A11/X6). Both bounds are kept; N34's kill claim excludes them and lists them as equivalent, consistent with the cross-check limit of P11 |
| P8 | clarification | 6 (N26 (d)) | The cache-dropping reset during the barrier-1 submission loses the tail write and SEAL replies `Corrupt` -18 at the readback, as N33 states for the same mechanism; the row now gives the outcome per reset model with the invariant that holds under both (review F5) |
| P9 | route or verification-plan item | 6 (N33) | Non-zero stale bytes that fail `archive::verify` are required, because they also defeat a readback check that refuses only zero sectors; zero bytes are an optional special case; a stale valid archive is the O17 case (N37 (a)) and stays a recorded limit if O17 is declined (review F4) |
| P10 | route or verification-plan item | 8.2 R2 | The library as built without test-only features is what a native profile links (L21) and was only type-checked; R2 acceptance adds Clippy `-D warnings` on that build for `x86_64-unknown-none` in a hosted gate, a lead-owned workflow change, not a core code change (review F7) |
| P11 | route or verification-plan item | 7 ("Cross-check limit"); 8.2 R3; 6 (N34) | Mount state, header and records are what the interface defines; refusal reasons are reported by no request, so the core is not required to expose one. State agreement is the stated limit of both comparisons; per-rule kills come from single-rule crafted copies in each implementation's own tests, with the equivalent mutants of P7 listed |
| P12 | route or verification-plan item | 8.2 R2 | The host model must meet the broker contract of 2.7 (L18), and that was checked by reading only. R2 acceptance adds a host test of the simulated disk alone for each property, under both reset models; the native broker stays an R5 precondition |
| P13 | route or verification-plan item | header; 8.2 R1; 9 | The review-5 answers RQ5-1 (N36 under O15) and RQ5-2 (`Io` for a refused submission with -14 or -13 after the boot mount; completions per the 2.5 table) were checked read-only by the lead only. The review of revision 4 must confirm 2.5, 5.3 and N36 against the decided O15; N36 now states the O15 outcome only (P4) |

**CODE-DELTA (revision 4).** Changes that this text requires on PR #413
relative to `e52cb865`. "Gated" means that the change applies only if the
owner accepts the named proposal; ungated rows apply now. No row changes the
check order, a capability or the treatment of disk bytes as untrusted.

| Spec clause | Rust core (`services/destination`) | Oracle (`tools/destination_disk.py`) | Tests | Gated |
|---|---|---|---|---|
| 2.3 control-character set (P3) | none | none | none required (at `e52cb865` the Rust and Python suites each have the 13 content cases of N34, including U+007F, U+0085 and the surrogate); optional: U+009F refused and U+00A0 accepted, to pin the upper C1 bound in both | no |
| 2.2 defensive FORMAT refusal (P6) | none (the guard is kept, with its comment) | none | none; the mutant is listed as equivalent in N35 | no |
| 2.7 step 4 count clause (P5) | none | none | none; equivalent mutant listed in N29 and N35 (test comments already note it) | no |
| 6 N26 (d) per-model outcome (P8) | none | none | at `e52cb865` `n26d_legitimate_waits_complete` asserts SEAL `Ok` and the entry listed (cache kept), SEAL `Corrupt` with nothing listed and a non-zero dropped count (cache dropped), and no safe path; add an assertion that no catalog sector is written in the cache-dropped model | no |
| 6 N33 stale-bytes variants (P9) | none | none | (i) is `n33_read_timeout_with_unflushed_append` (non-zero stale bytes); (ii) zero variant optional; (iii) `n33_read_timeout_over_stale_valid_archive_recorded` stays a recorded case, not a pass, unless O17 (a) is accepted | (iii) only: O17 (a) |
| 6 N34 label cases and kill claim (P3, P7) | none | none | none; the A10 and A11/X6 equivalents are listed in N34 | no |
| 2.7 step 3 content digest (P1, O17 (a)) | keep, per writing entry, the FNV-64 of the bytes of every APPEND answered `Ok`, in order (or use the equivalent trailer comparison of 2.7 step 3); at step 3, after the readback, compare it with the FNV-64 of the first `length` bytes (running form) or compare the readback trailer with the last 8 appended bytes (trailer form); on a mismatch reply `Corrupt` -18, discard the writing entry, write no catalog sector, do not set the safe-path latch (per the O17 recommendation; follows the answer to its safe-path sub-question) | none (it judges images, not the SEAL protocol) | N37 (a), with both the read-timeout reset and the device's own reset during the barrier-1 submission, under both reset models (the recorded N33 (iii) test becomes a negative test); N37 (b) with a simulated-disk mode that reports success for every write of the entry and for barrier 1 and stores none; kill: comparison removed | O17 (a) |
| 2.7 portal trailer comparison (P1, O17 (b)) | none (no portal on PR #413) | none | N37 (c) with the portal, at R7 | O17 (b) |
| 2.7 *sealed* resolution by byte equality (P2, O18) | after the trigger, resolve *sealed* only if the remounted catalog equals the copy encoded and checked in step 4 (byte or full-field equality, equivalent for valid copies, 2.6), instead of the extension check plus the provisional entry ID; otherwise the failed-mount state with the binding *unknown* | none | N35 (e): extension-passing catalogs with the provisional ID and another length, trailer FNV-64, label or next entry ID give the failed-mount state; kill: the current extension-based resolution | O18 |
| 2.7 committed-copy equality (P2 clarification) | none (full `Catalog` equality is byte equality for valid copies) | none | none (N35 (c) committed-copy cases added in the O14/O16 fix pass) | no |
| 7 cross-check limit (P11) | none (no refusal reason exposed) | none | none | no |
| 8.2 R2 lint of the non-test build (P10) | none | none | hosted gate, command indicative and not run: `cargo +1.90.0 clippy --locked -p destination-core --lib --target x86_64-unknown-none -- -D warnings`, without test-only features; lead-owned workflow change, not part of the PR #413 code | no |
| 8.2 R2 broker-contract test (P12) | none | none | a host test per property 1–5 of the 2.7 broker contract for the simulated disk alone, under both reset models, including a second poll after a delivered result and a request ended by a reset | no |
| 6 N36 under O15 (P4, P13) | none | none | none (the tests already assert the O15 outcomes) | no |

The status of each row above at `f993a1a3`, and the changes that revision 5
adds, are in "CODE-DELTA (revision 5)" below; this table is not edited.

**Review 6.** SPEC-REVIEW-6 is an independent AI security review of
revision 4 together with the owner-decision commit `1735ab06`, which no
earlier review covered (diff base `dd2bb19d`, implementation compared at
`e52cb865`), stored verbatim as
`docs/verification/s09-destination-service-20261123/SPEC-REVIEW-6.md`. It
found one resolution row per P1–P13, the decided meaning of O14–O16
unchanged, both stricter changes written as proposed owner items (O17, O18),
the review-5 answers RQ5-1 and RQ5-2 confirmed, and the security floor
intact. It returned APPROVE with no required finding and nine optional
findings. The lead applied them by hand as below. One read-only AI verifier then
checked these edits: it found the decided meaning of O14–O16, the security
floor and the IDs intact, and two wording faults in the edits (N37 asserted
"no safe path" although its note left latching open; the trailer form of 2.7
step 3 needed its precondition), which the lead corrected with the
verifier's optional wording points. The corrections were not re-reviewed.

| Finding | Disposition | Sections |
|---|---|---|
| OPT6-1 | Applied: not latching after a digest mismatch is stated as following the revision-3 readback rule, with its O9 dependency; the latching question is an O17 sub-question; N37 no longer lists a latching implementation as a killed mutant | 2.7 "Stale valid content"; 8.1 O17; N37 |
| OPT6-2 | Applied: R7 requires every label to be rendered in an isolated bidirectional context and visibly delimited from trusted text | 8.2 R7 |
| OPT6-3 | Applied: the B6 unreachability is attributed to this text's reading of the call sites, which review 6 confirmed | 6 preamble; N35 |
| OPT6-4 | Applied: N26 (d) states the free-space precondition and that every unflushed write of the entry is lost; the barrier-1 reset is named in "Stale valid content" and added to N37 (a) | 2.7; N26; N37; CODE-DELTA |
| OPT6-5 | Applied: under O18 the encoded copy is kept with the committed copy to judge the remount | 2.7 failure paragraph |
| OPT6-6 | Applied: the blank-candidate row of the check-7 table names the defensive FORMAT refusal | 2.2 |
| OPT6-7 | Applied: the review-4 OP-5 row notes the later O16 decision; the review-5 RQ5-1 and OPT-2 rows are marked superseded in revision 4 (P4, P2) | 9 |
| OPT6-8 | Applied: the count-clause equivalence is stated to rely on the 16-entry bound | 2.7 step 4 |
| OPT6-9 | Applied: "bytes appended so far" means every APPEND answered `Ok`; the equivalent trailer comparison is allowed | 2.7 step 3; CODE-DELTA |

**Owner decisions of 23 November 2026 (O17, O18).** The project owner
accepted the recommendations of O17 and O18 ("Yes, accept both", in the S09
lead session). Consequences for PR #413: O17 — the R2 core implements the
SEAL content check of 2.7 step 3 (running digest or the equivalent trailer
form) with `Corrupt` -18 and no safe path, and N37 (a), (b) and N33 (iii)
become negative tests under both reset models; the portal check (b) and
N37 (c) belong to R7, because PR #413 has no portal. O18 — the R2 remount
resolves the export binding *sealed* only on byte equality with the copy
encoded in step 4, and N35 (e) asserts the failed-mount state. The
conditional wording of revision 4 ("if declined", "lapses") is now historical
and is left in place until the next revision; the decided labels above
govern. These relabelled markers and this paragraph were not reviewed.

Note (revision 5): the conditional wording that the paragraph above names was
removed or restated as decided in revision 5 (Q3); the paragraph itself is
kept as written. Its "running digest or the equivalent trailer form" is
qualified by 2.7 step 3 (Q8): the trailer form is relied on only with the
real `volume_core::archive::verify` linked (R5), and a host build uses the
running form (review 9, OPT9-8).

**Revision 5 record.** The inputs are the code round on draft PR #413 that
implemented the owner decisions O17 (a) and O18 (`f993a1a3`, host only,
unmerged), its independent AI code review CODE-REVIEW-O17-O18 (APPROVE, no
required finding, optional OPT-1–OPT-5; 16 mutants, of which M8 survived as equivalent and M7
as equivalent in every build without `test-hooks`), and the owner-decision
commit `a769bc65`, which reviews 7 and 8 covered (REQUEST_CHANGES on other
findings) and review 9 is the first review to approve. Their open points
were deduplicated into Q1–Q13. Mutant names are those of that code review;
they are not items of this document.
As in revisions 3 and 4, each choice is reasoned from the design, the threat
model or a fail-closed argument, not taken from the code; where the code at
`f993a1a3` differs from the chosen text, the difference is listed in the
CODE-DELTA (revision 5) table below. No resolution widens a capability,
changes the check order of 2.2, adds trust in disk bytes or changes the
threat model of the content check (accidental faults only; FNV-64 is not
collision-resistant); the decided meaning of O14–O18 is unchanged. These
resolutions are an AI revision, not a human audit. They were reviewed by
review 7 (REQUEST_CHANGES) and review 8 (REQUEST_CHANGES); review 9
approved the fixed text with no required finding; review 10
(REQUEST_CHANGES) checked the lead's pass over its optional findings, and
review 11 approved the further fix pass with no required finding; its
nine optional findings are deferred to the next revision.

| ID | Resolution type | Section changed | Reason |
|---|---|---|---|
| Q1 | route or verification-plan item | 9 (pointer after the CODE-DELTA (revision 4) table; new CODE-DELTA (revision 5)) | The revision-4 row for N33 (iii) still named the recorded test `n33_read_timeout_over_stale_valid_archive_recorded`; at `f993a1a3` it is the negative test `n37a_read_timeout_over_stale_valid_archive` (review OPT-5). The historical table is left as written; the new table gives the status of every revision-4 row at `f993a1a3` |
| Q2 | clarification | header (revision 5); 8.1 O17, O18 (consequences); 8.2 R1; 9 (note after the `a769bc65` paragraph) | The relabelling of `a769bc65` is checked against the owner's answer "Yes, accept both" to O17 and O18 as written at `0643aa09`: O17 is recommendation (c), the service check (a) and the portal check (b), with `Corrupt` -18 and no safe path on a mismatch, pending O9; O18 is equality with the copy encoded and checked in step 4. The text states both meanings explicitly; the review of revision 5 must confirm them, because `a769bc65` itself was not reviewed (confirmed by reviews 7 and 8 and approved by review 9; annotated after review 10, OPT10-5) |
| Q3 | clarification | header (revision 4 parenthesis); 2.6 failed-mount state; 2.7 ("Stale valid content", failure paragraph, O18 paragraph); 6 (preamble, N26 (d), N33 (iii), N35 (e), N37, closing paragraph); 8.1 O16, O17, O18; 8.2 R2, R7, R8 | As P4 did for O14–O16: conditional O17 and O18 wording ("if declined", "lapses", "under O17 (a)", "waits on O17/O18") is removed or restated as decided; the declined alternatives are named as declined ((a) alone, (b) alone, the stated limit; the O16 definition alone for the *sealed* arm); unconditional rules are kept (threat model, `Corrupt` -18, the O9 dependency); the stale-content case of 2.7 and N26 (d) is stated as the case the decided checks prevent, no longer as a current limit (2.7 refined after review 9, OPT9-5: the service check prevents the commit, the portal check reports the export as failed). The latching sub-question is stated as decided, "no latch, pending O9", and only O9 evidence that late writes can land reopens it, through a new owner decision. Section 9 history is not rewritten |
| Q4 | route or verification-plan item | 2.7 (broker contract, host models); 8.2 R2; CODE-DELTA (revision 5) | The simulated disk does not model the broker's own deadline (property 4) or the persistence of quarantine and offline (property 5). The contract and the R2 check are not relaxed: a host model must reproduce every property. The deadline becomes an opt-in mode, because N26 (e) and `n26f_remount_after_expired_poll_bound` break property 4 on purpose and an always-on deadline would conflict with them (so does the "given up on" case of `p12_2_refused_submission_changes_nothing`; added after review 8, RQ8-3); quarantine and offline become device state until a simulated reboot. Both are open R2 items with tests. No owner question: the evidence is a host-model property that R2 already requires, not native broker behaviour, which stays an R5 precondition |
| Q5 | clarification | 6 (N35 equivalent mutants); CODE-DELTA (revision 5) | M7 (the pre-decode catalog instead of the checked copy) is listed as equivalent in every build without `test-hooks`, where decoding a copy that passed the validator returns it unchanged, with the qualifier that it is observable under the test-only tamper hook. The sub-case that pins the checked copy is optional: the difference exists only in test builds, and no product build can reach it |
| Q6 | normative fix | 2.7 ("Meaning of kernel -8"); 8.2 R2; CODE-DELTA (revision 5) | The rule that the service consumes each result before the next submission implies a poll invariant, now stated: the service polls only a request it submitted, that was admitted, and whose result it has not consumed. Because the simulated disk now answers a stray poll with -1 instead of panicking (review OPT-3), a service defect would show only as `Io` or `Unknown`; a stray-poll counter in the simulated disk, asserted zero in every service-driven host test, restores the tripwire (wording aligned with 8.2 R2 after review 9, OPT9-4) |
| Q7 | clarification | 2.7 (broker contract, property 3); 2.5 (device results) | Property 3 now states that a further poll returns -1 and changes nothing. On `main` this was checked for a poll after a delivered result (`volume.rs:886-891`, `Policy::consume`) *[main]*, not for a request ended by a reset; the host model returns -1 in both cases (`p12_3_answered_exactly_once`). After review 9, OPT9-3, property 3 records the reading of `main` for a request ended by a reset (read, not executed: -14 for a further poll after a read while the reset runs) as a K5 delta and R5 precondition (4.3 K5; 8.2 R5). 2.5 adds no row: -1 does not occur under the Q6 invariant and, if it does, falls under "any other completion result", which fails closed (`Io` for a read; `Unknown` and the safe path for a write or flush, or the failed-mount state for a mount or remount FLUSH and a FORMAT write or flush, 2.6; parenthesis added after review 7, OPT7-2; restated in 2.5 after review 8, OPT8-5, to name the mount or remount read as well) |
| Q8 | clarification | 2.7 step 3 | Both forms are kept and the decided meaning of O17 (a) is unchanged. The running form is preferred because it covers every appended byte whatever the format and the verifier; the trailer form is relied on only with the real `volume_core::archive::verify` linked (R5), because its strength rests on that verifier's trailer rule (wording corrected after review 7, OPT7-1). The R2 test stand-in checks a different trailer rule (it hashes every byte before the last 8, the end marker included, and checks only a length floor and the two magic values besides that sum), so a host build uses the running form, as `f993a1a3` does (review OPT-4) |
| Q9 | normative fix | 6 (N37: precondition, actions, expected outcome, test names, kills) | N37 is aligned with the tests at `f993a1a3`: (a) two tests, the barrier-1 variant with a stale archive that is not sector-aligned; (b) `Fault::silent` with no reset, so both models agree; new case (d) for "only APPENDs answered `Ok` are hashed", whose kills (M2b, M3b) the row lacked; per-model outcomes stated (cache kept: SEAL `Ok`, `aux` equals the sent trailer). Kills added for M4, M5, M9, M12 and M13, which the code review observed |
| Q10 | route or verification-plan item | header (revision 5; evidence labels); 8.2 R1, R2, R3, R7, R8 | Status clauses for revision 5 are added and earlier clauses kept as history; *[PR #413]* for a revision-5 claim means `f993a1a3`. Gate and mutation results are those reported by CODE-REVIEW-O17-O18 for the reviewed diff over `8f387ea1`; the comment-only edits made after it in `f993a1a3` are not reviewed, and nothing was re-run for this revision |
| Q11 | clarification | 2.7 (O18 paragraph); 6 (N35); 8.1 O18 | M8 (the extension check added back beside the equality) survived as equivalent and is listed in N35 as observed, no longer "expected, not run"; `extends_by_one` stays required only for the step-4 pre-write check |
| Q12 | route or verification-plan item | header (revision 5); 8.2 R1; 9 (reviews 7 and 8) | The review of revision 5 (reviews 7 and 8) must cover `a769bc65` (Q2), confirm that no decided rule of O14–O18 changed meaning, and confirm the security floor: no capability widened, check order unchanged, disk bytes untrusted, FNV-64 threat model limited to accidental faults |
| Q13 | route or verification-plan item | 2.7 step 3 and "Stale valid content"; 6 (N37); 8.1 O17; 9 (reviews 7 and 8) | The review-6 corrections that were applied without re-review (N37 "no safe path" against the open latching note; the trailer-form precondition) are rewritten by Q3 and Q8 in their final form; the review of revision 5 (reviews 7 and 8) must cover the N37 row, the trailer form of 2.7 step 3 and the O17 latching sentence in that form |

**CODE-DELTA (revision 5).** First, the status at `f993a1a3` of each row of
the CODE-DELTA (revision 4) table: *done* (with the test or code that
carries it), *open*, or *not part of PR #413*. Then the code and test
obligations that revision 5 adds for PR #413. Test names and code are as
read at `f993a1a3` *[PR #413]*; the gate and mutation results behind them are
those reported by CODE-REVIEW-O17-O18 for the reviewed diff over `8f387ea1`,
before the lead's later comment-only edits (OPT-1, OPT-2, OPT-4), and were
not re-run for this revision.
"Gated" keeps its revision-4 meaning; since O17 and O18 are decided, no row
is gated. No row changes the check order, a capability or the treatment of
disk bytes as untrusted.

| Spec clause | Rust core (`services/destination`) | Oracle (`tools/destination_disk.py`) | Tests | Gated |
|---|---|---|---|---|
| Rev. 4 row: 2.3 control-character set (P3) | done (no change required) | done (no change required) | required part done (the 13 N34 content cases in both suites, `n34_record_and_copy_validity` in the Rust core); optional U+009F and U+00A0 cases open (not found at `f993a1a3`) | no |
| Rev. 4 row: 2.2 defensive FORMAT refusal (P6) | done (the guard is kept) | none | done (equivalent mutant listed in N35) | no |
| Rev. 4 row: 2.7 step 4 count clause (P5) | done (no change) | none | done (equivalent mutant listed in N29 and N35 and in the comment of `n35_remount_contradicting_committed_catalog`) | no |
| Rev. 4 row: N26 (d) per-model outcome (P8) | none | none | done: `n26d_legitimate_waits_complete` asserts no catalog sector written in the cache-dropped model (killed M9 in the code review) | no |
| Rev. 4 row: N33 stale-bytes variants (P9) | none | none | (i) done: `n33_read_timeout_with_unflushed_append`; (ii) optional, open (no zero-byte variant found); (iii) done as the negative test `n37a_read_timeout_over_stale_valid_archive` (renamed from the recorded test the revision-4 row names) | no |
| Rev. 4 row: N34 label cases and kill claim (P3, P7) | none | none | done (no change required) | no |
| Rev. 4 row: 2.7 step 3 content digest (P1, O17 (a)) | done: running form (`Writing::digest`, compared after the readback in SEAL; mismatch replies `Corrupt`, discards, writes no catalog sector, leaves the latch clear) | none | done: `n37a_read_timeout_over_stale_valid_archive`, `n37a_own_reset_at_barrier1_over_stale_valid_archive`, `n37b_unstored_entry_writes_over_stale_valid_archive` (with `Fault::silent`), `n37_digest_covers_only_appends_answered_ok`; kill of the removed comparison observed (M1) | no (O17 decided) |
| Rev. 4 row: 2.7 portal trailer comparison (P1, O17 (b)) | not part of PR #413 (no portal) | not part of PR #413 | not part of PR #413: N37 (c) at R7 | no (O17 decided) |
| Rev. 4 row: 2.7 *sealed* resolution by byte equality (P2, O18) | done: the remount compares with the decoded, checked step-4 copy by full `Catalog` equality | none | done: N35 (e) cases in `n35_remount_contradicting_committed_catalog`; kill of the extension-based resolution observed (M6) | no (O18 decided) |
| Rev. 4 row: 2.7 committed-copy equality (P2 clarification) | done (no change) | none | done (no change) | no |
| Rev. 4 row: 7 cross-check limit (P11) | done (no change) | done (no change) | done (no change) | no |
| Rev. 4 row: 8.2 R2 lint of the non-test build (P10) | none | none | open, and not part of the PR #413 code: the hosted gate is a lead-owned workflow change; the command ran locally with exit 0 in the code review (indicative only) | no |
| Rev. 4 row: 8.2 R2 broker-contract test (P12) | none | none | partly done: `p12_1_one_request_admitted`, `p12_2_refused_submission_changes_nothing`, `p12_3_answered_exactly_once`, `p12_4_admission_time_and_reset_fence`, `p12_5_quarantined_or_offline_not_admitted`, under both reset models; properties 4 (deadline) and 5 (persistence) not modelled, see the Q4 row below | no |
| Rev. 4 row: N36 under O15 (P4, P13) | none | none | done (no change required) | no |
| 2.7 broker contract, properties 4 and 5 in host models (Q4) | none | none | open: an opt-in deadline mode of `SimDisk` that answers a request past its deadline -16 (mutating) or -13 (read) by itself and then resets, off by default so that N26 (e), `n26f_remount_after_expired_poll_bound` and the "given up on" case of `p12_2_refused_submission_changes_nothing` (added after review 8, RQ8-3) keep their stubs; quarantine (-13) and offline (-14) held as `SimDisk` device state until a simulated reboot; `p12_4` and `p12_5` extended to both, under both reset models; kills: a model whose deadline does not answer, and one that admits a submission after quarantine or offline before the simulated reboot; optional, open (review 9, OPT9-7): a default-mode tripwire in `SimDisk` that flags a request still pending past `TIMEOUT` from its admission, with explicit opt-outs for N26 (e), N26 (f) and the "given up on" case of `p12_2`, so that the equivalence of the default-off deadline with an always-on one rests on a test rather than on a manual sweep | no |
| 2.7 poll invariant (Q6) | none expected (`device::run` polls only an admitted request and stops at the first result other than -8 or at the poll bound) | none | open: a stray-poll counter in `SimDisk::poll` for a poll with no request pending, asserted zero in every service-driven host test (wording aligned with 8.2 R2 after review 9, OPT9-4, and review 10, OPT10-2); the P12 tests may still observe -1; kill: a service loop that polls once more after a delivered result | no |
| 2.7 property 3, further poll (Q7) | none | none | partly done (corrected after review 7, RQ7-2): `p12_3_answered_exactly_once` asserts -1 at each further poll, after a delivered write and read result and after requests ended by a reset, and an unchanged caller buffer after the further read poll only; open: a device-state snapshot (durable sectors, cache, write and flush totals; the `snapshot` helper of `sim_contract.rs`) taken before and after each further poll and asserted equal, so that "changes nothing" of property 3 is tested | no |
| 2.7 step 3, preferred running form (Q8) | done: running form, with its reason in the comment of `Writing::digest` | none | none | no |
| 6 N37 (d), APPENDs answered `Ok` only (Q9) | none | none | done: `n37_digest_covers_only_appends_answered_ok` (kills M2b, M3b observed) | no |
| 6 N35 equivalent mutants M7 and M8 (Q5, Q11) | none | none | done: both described (not by name) in the comment of `n35_remount_contradicting_committed_catalog` (wording corrected after review 8, OPT8-6); optional, open: the sub-case that pins the checked copy (a tamper under `test-hooks` that passes step 4, a failed step-4 write, a remount of the tampered copy, expected *sealed*) | no |

**Review 7.**
SPEC-REVIEW-7 is an independent AI review of revision 5, stored verbatim as
`docs/verification/s09-destination-service-20261123/SPEC-REVIEW-7.md`. Its
verdict is REQUEST_CHANGES, with three required findings (RQ7-1–RQ7-3) and
six optional findings (OPT7-1–OPT7-6). The lead applied all nine; the
evidence claims were checked against `f993a1a3` of PR #413; no new
*[main]* claim is made. No resolution widens a capability,
changes the check order of 2.2, trusts disk bytes, extends the FNV-64 threat
model beyond accidental faults or changes the meaning of a decided O14–O18
rule; no owner question is added. The fixes await re-review. An AI review is
not a human audit.

| Finding | Disposition | Sections |
|---|---|---|
| RQ7-1 | Applied. At `f993a1a3` `SimDisk::poll` returns -1 with no event or counter, and no test counts stray polls. The present-tense claim is replaced by "a host test that counts stray polls and asserts zero is an open R2 item", outside the *[PR #413]* label, consistent with 8.2 R2 and the CODE-DELTA (revision 5) row (Q6) | 2.7 ("Meaning of kernel -8") |
| RQ7-2 | Applied. `p12_3_answered_exactly_once` asserts -1 at the four further polls and an unchanged caller buffer after the further read poll; it takes no device-state snapshot. The row is marked "partly done", and a snapshot before and after each further poll (the `snapshot` helper of `sim_contract.rs`) is added as open | 9 (CODE-DELTA (revision 5), property 3 row) |
| RQ7-3 | Applied, both parts. The sentence now lists every case in which a stub or fault-injecting disk returns a broker result or timing outside the contract on purpose, each tied to its row: N13 (completion -13 on a write or flush; the N13 host test injects -12, -13 and -16), N26 (a)–(c) (-8 on submission for good), N26 (e) and (f) (a poll pending past the property-4 deadline) and N33 (completion -14; completion -13 on a write or flush). The list was checked against the fault injections of the host tests at `f993a1a3` (the other injected write and flush results are -12 and -16, inside the contract). Medium behaviour is stated as outside the contract's scope rather than as a departure: stored-byte faults (N8, N11, N12, N25, N37 (b)) and late completion (N24, the O9 question), so N24 and N37 (b) are not listed. The test `n26f_remount_after_expired_poll_bound`, which calls itself N26 (f), is added to the N26 row as case (f) with its asserted outcome and kills. Completion -16 on a read (N33) is not listed: properties 1–5 restrict a read's result only at its deadline, and 2.5 does not call it outside the contract | 2.7 (broker contract, host models); 6 (N26) |
| OPT7-1 | Applied. "Equivalent only when" becomes "relied on only with the real `volume_core::archive::verify` linked (R5)", with the reason that a portal v1 archive need not pass a stand-in with another trailer rule; the conclusion (running form preferred; host builds use it) is unchanged | 2.7 step 3; 9 (Q8 row) |
| OPT7-2 | Applied. The failed-mount state for a mount or remount FLUSH and a FORMAT write or flush is named beside the safe path for a -1 poll result | 2.5; 9 (Q7 row) |
| OPT7-3 | Applied. R2 now says that N33 (iii) and N37 (a), (b) are negative tests under both reset models and that N37 (d) pins that only APPENDs answered `Ok` are hashed; 8.1 O17 is unchanged | 8.2 R2 |
| OPT7-4 | Applied. The M4 kill names the latch first, as `refused_without_latch` observes it, and the later CREATE `Access` -5 as the consequence | 6 (N37) |
| OPT7-5 | Applied as a note only; the question text is kept as asked in revision 4. The note states that "Today" describes revision 4 and points option (a)'s trailer comparison to the precondition of 2.7 step 3 | 8.1 O17 |
| OPT7-6 | Applied. R8 acceptance now requires the decided O17 (b) portal check implemented in R7 and N37 (c) passing with the portal; R8 still no longer waits on the O17 decision itself | 8.2 R8 |

Note (review 8, RQ8-3). The RQ7-3 disposition above states that the list
was checked against the fault injections of the host tests at `f993a1a3`.
That check missed `p12_2_refused_submission_changes_nothing`, whose "given
up on" case keeps a write pending past the poll bound and beyond the
property-4 deadline. 2.7 now lists it. Its last sentence, "Completion -16
on a read (N33) is not listed", is superseded by review 8, OPT8-2: 2.7 now
places a read completion -16 outside the contract and lists N8 and N33 for
it. The disposition above is left as written.

**Review 8.**
SPEC-REVIEW-8 is an independent AI review of revision 5 (with the review-7
fixes), stored verbatim as
`docs/verification/s09-destination-service-20261123/SPEC-REVIEW-8.md`. Its
verdict is REQUEST_CHANGES, with four required findings (RQ8-1–RQ8-4) and
six optional findings (OPT8-1–OPT8-6). The lead applied the nine findings
that change this text; RQ8-2 needs no text change and is an action of the
revision-5 commit. The code claims were checked against `f993a1a3` of PR #413; one
*[main]* claim is added (2.7, the M3 broker's -16), checked against
`035b44e5` and `origin/main`. No resolution widens a capability, changes the check order
of 2.2, trusts disk bytes, extends the FNV-64 threat model beyond
accidental faults, relaxes a property of the broker contract or changes the
meaning of a decided O14–O18 rule; no owner question is added. The fixes
await re-review. An AI review is not a human audit.

| Finding | Disposition | Sections |
|---|---|---|
| RQ8-1 | Applied. R1 now states that review 7 returned REQUEST_CHANGES on revision 5 and its nine findings were applied, that review 8 returned REQUEST_CHANGES on the result, and that the fixes await re-review, which must also cover `a769bc65` and the review-6 corrections. The Revision 5 record intro and the Q12 and Q13 rows name the review of revision 5 (reviews 7 and 8) instead of review 7 | header (revision 5); 8.2 R1; 9 (Revision 5 record, Q12, Q13) |
| RQ8-2 | Accepted; no text change. SPEC-REVIEW-7.md and SPEC-REVIEW-8.md are not yet present in `docs/verification/s09-destination-service-20261123/`. Both must be added verbatim in the revision-5 commit, as SPEC-REVIEW-6.md was added with `0643aa09`; until then the "Review 7" and "Review 8" paragraphs point to files that do not exist | 9 (reviews 7 and 8) |
| RQ8-3 | Applied. At `f993a1a3` `p12_2_refused_submission_changes_nothing` keeps a write pending for `POLL_BOUND` + `SUBMIT_BOUND` + 50 = 590 ticks, with `TIMEOUT` 500 and `POLL_BOUND` = `TIMEOUT` + 2 × `RESET_TIMEOUT` = 520 (`device.rs`), and then delivers 0. 2.7 now states that the "given up on" clause of property 2 can occur only when property 4 is not met, that its R2 test breaks property 4 on purpose, and lists the P12 test of property 2 among the cases; "only where the case of a section 6 row requires it" becomes "only where a section 6 row or the P12 test of property 2 requires it". A repeated sweep of the pending-time injections in `services/destination/tests` and `src` at `f993a1a3` found no other request held pending past `TIMEOUT` beyond N26 (e), N26 (f) and `p12_2`. The Q4 row and the CODE-DELTA (revision 5) Q4 row name `p12_2` beside N26 (e) and (f); the RQ7-3 disposition keeps its text and receives a note. Properties 2 and 4 are not relaxed | 2.7 (broker contract, host models); 9 (Q4 row; CODE-DELTA (revision 5); note after the Review 7 table) |
| RQ8-4 | Applied. The N37 precondition now allows a stale archive that is not sector-aligned for (b) as well, because `Fault::silent` also leaves the SEAL tail write unstored; the (b) test name is annotated "stale archive not sector-aligned" (`archive(2, 1, 1000, 5)`, 1080 bytes, as asserted in `n37a_own_reset_at_barrier1_over_stale_valid_archive`) | 6 (N37) |
| OPT8-1 | Applied. The sentence now begins "With the real `volume_core::archive::verify` linked (below), an equivalent form compares ..." | 2.7 step 3 |
| OPT8-2 | Applied, with a per-kind rule. 2.7 now states that the contract's completion codes are per request kind (property 4: -16 for a write or flush, -13 for a read), so a code it gives only to the other kind is outside the contract: -13 on a write or flush, as 2.5 already said (revision 3), and -16 on a read. A read completion -13 before its deadline is inside (the read's own code; property 4 bounds when it must come, not how early), as is a write or flush completion -16 before its deadline. The M3 broker on `main` answers -16 only for a mutating request *[main]*, which supports this reading. The exception list therefore gains N8 (`n8_read_error_on_newer_copy_is_failed_mount`, read -16; the N8 row requires the `Unknown` read case) and N33 (read -16). This supersedes the last sentence of the RQ7-3 disposition, which receives a note. The P12 tests inject only -16 on writes and -13 on reads, so the single P12 exception of RQ8-3 is unchanged. 2.5 answers every case fail closed; no service rule changes | 2.7 (broker contract, host models); 9 (note after the Review 7 table) |
| OPT8-3 | Applied. The note reads "Question text as asked in revision 4, without the conditional sentences that revision 5 removed (Q3 ...)" | 8.1 O17 |
| OPT8-4 | Applied. "; the other items of 8.1 remain open as stated there" added to the bracket | 8.2 R1 |
| OPT8-5 | Applied. The -1 paragraph now says that on a mount or remount read or FLUSH, or a FORMAT write or flush, -1 leads to the failed-mount state instead of the safe path (2.6), consistent with `n8_read_error_on_newer_copy_is_failed_mount`; the Q7 row is annotated | 2.5; 9 (Q7 row) |
| OPT8-6 | Applied. The row now says that both mutants are "described (not by name)" in the comment of `n35_remount_contradicting_committed_catalog` | 9 (CODE-DELTA (revision 5), N35 row) |

**Review 9.**
SPEC-REVIEW-9 is an independent AI review of revision 5 (after the review-7
and review-8 fixes) and of the owner-decision commit `a769bc65`, stored
verbatim as
`docs/verification/s09-destination-service-20261123/SPEC-REVIEW-9.md`. Its
verdict is APPROVE, with no required finding and nine optional findings
(OPT9-1–OPT9-9). It covered `a769bc65` (Q2), the review-6 corrections in
their final form (Q13) and the security floor (Q12). The lead applied the
optional findings in a separate fix pass. The one new PR #413 claim (the
N26 (e) test name) was checked against `f993a1a3`; the one *[main]* reading that is added (2.7,
property 3) was read from the code of `035b44e5` and `origin/main`
(`b9aa517c`), whose `kernel/src/m2/m3/volume.rs` and
`crates/virtio-core/src/lib.rs` are identical, and was not executed. No
resolution widens a capability, changes the check order of 2.2, trusts disk
bytes, extends the FNV-64 threat model beyond accidental faults, relaxes a
property of the broker contract or changes the meaning of a decided O14–O18
rule; no owner question is added. The Review 7 and Review 8 paragraphs and
tables above are left as written. An AI review is not a human audit.

| Finding | Disposition | Sections |
|---|---|---|
| OPT9-1 | Accepted; no text change. SPEC-REVIEW-7.md, SPEC-REVIEW-8.md and SPEC-REVIEW-9.md are now present in `docs/verification/s09-destination-service-20261123/` (in the working tree, to be added by the revision-5 commit), so the "stored verbatim as" sentences of the Review 7 and Review 8 paragraphs point to existing files; this resolves RQ8-2 | 9 (reviews 7, 8 and 9) |
| OPT9-2 | Applied. The note now names the three edits of revision 5 to the question text (the removed conditional sentences, "is recommended" to "was recommended", and N37 (d) in the marking sentence), states that the meaning is unchanged, and its continuation lines are indented; "was recommended" is kept, not restored | 8.1 O17 |
| OPT9-3 | Applied as a K5 delta and R5 precondition. Property 3 now records what the code of `main` does for a further poll after a request ended by a reset (read, not executed): a mutating request is consumed by its first poll and a further poll returns -1; a read is not consumed while the phase is `Resetting`, so a further poll returns -14 until an acknowledged reset drops it, and, after a quarantine, until reboot or until the owning process exits (corrected after review 10, OPT10-1). K5 must return -1 for such a poll (corrected after review 10, RQ10-2); the service still does not rely on -1. The Q7 row is annotated | 2.7 (broker contract, property 3); 4.3 K5; 8.2 R5; 9 (Q7 row) |
| OPT9-4 | Applied. The stray-poll obligation is phrased as in 8.2 R2 ("a stray-poll counter in the simulated disk, asserted zero in every service-driven host test") in 2.7, the Q6 row and the CODE-DELTA (revision 5) Q6 row; the RQ7-1 disposition keeps its quoted text | 2.7 ("Meaning of kernel -8"); 9 (Q6 row; CODE-DELTA (revision 5), Q6 row) |
| OPT9-5 | Applied. The parenthesis now says that the decided service check prevents the commit and the decided portal check reports the export as failed; the Q3 row is annotated. The N26 (d) row already attributes the refusal to the step-3 content check and is unchanged | 2.7 ("Stale valid content"); 9 (Q3 row) |
| OPT9-6 | Applied. The contract paragraph names `n26e_poll_beyond_bound_is_unknown` beside `n26f_remount_after_expired_poll_bound` (both in `services/destination/tests/broker.rs` at `f993a1a3`) | 2.7 (broker contract, host models) |
| OPT9-7 | Applied as an optional, open item: a default-mode tripwire in `SimDisk` for a request still pending past `TIMEOUT` from admission, with opt-outs for N26 (e), N26 (f) and the "given up on" case of `p12_2` | 9 (CODE-DELTA (revision 5), Q4 row) |
| OPT9-8 | Applied. The revision-5 note after the `a769bc65` paragraph now points its "running digest or the equivalent trailer form" to the precondition of 2.7 step 3 (Q8); the paragraph itself is kept as written | 9 (note after the `a769bc65` paragraph) |
| OPT9-9 | Applied. The header and 8.2 R1 Status (revision 5) record reviews 7 and 8 (REQUEST_CHANGES, fixed) and review 9 (APPROVE, no required finding), and state that the lead's pass over its optional findings is to be checked by review 10 (pending); the statements that `a769bc65` was not reviewed are updated in the header, 8.2 R1 and the Revision 5 record: reviews 7 and 8 covered `a769bc65` and the review-6 corrections and returned REQUEST_CHANGES on other findings, and review 9 is the first review to approve them (corrected after review 10, RQ10-1) | header (revision 5); 8.2 R1; 9 (Revision 5 record) |

These edits were checked by review 10 (below).

**Review 10.**
SPEC-REVIEW-10 is an independent AI review of the lead's fix pass over the
optional findings of review 9 (revision 5). It is stored verbatim as
`docs/verification/s09-destination-service-20261123/SPEC-REVIEW-10.md`.
Its verdict is REQUEST_CHANGES, with two required findings (RQ10-1,
RQ10-2) and eight optional findings (OPT10-1–OPT10-8). The lead applied all
ten in a further fix pass. RQ10-1 corrects review-status statements against
the stored SPEC-REVIEW-7 and SPEC-REVIEW-8, whose objects include
`git diff 0643aa09 a769bc65` and the review-6 corrections in their final
form. RQ10-2 removes a weaker alternative that the review-9 pass had added
to the K5 delta, so that no property of the broker contract is relaxed. The
*[main]* readings that are corrected or added (2.7, property 3: OPT10-1,
OPT10-3) were read from `origin/main` at `b9aa517c`, whose
`kernel/src/m2/m3/volume.rs` and `crates/virtio-core/src/lib.rs` are
identical to those of `035b44e5`, and were not executed. No resolution
widens a capability, changes the check order of 2.2, trusts disk bytes,
extends the FNV-64 threat model beyond accidental faults, relaxes a property
of the broker contract or changes the meaning of a decided O14–O18 rule; no
owner question is added (OPT10-6 adds a pointer within O4 (a)). The
historical `a769bc65` paragraph and the Review 7, Review 8 and Review 9
paragraphs are left as written; the OPT9-3 and OPT9-9 rows of the Review 9
table are corrected as RQ10-1, RQ10-2 and OPT10-1 require. An AI review is
not a human audit.

| Finding | Disposition | Sections |
|---|---|---|
| RQ10-1 | Applied. The header, 8.2 R1 Status (revision 5) and the Revision 5 record intro state that reviews 7 and 8 covered `a769bc65` and the review-6 corrections and returned REQUEST_CHANGES on other findings, and that review 9 is the first review to approve them; the OPT9-9 disposition is corrected to match. The historical `a769bc65` paragraph ("were not reviewed") and the Review 7 and Review 8 paragraphs are left as written | header (revision 5); 8.2 R1; 9 (Revision 5 record; Review 9 table, OPT9-9 row) |
| RQ10-2 | Applied. ", or at least no device result" is deleted in all four places; the K5 delta reads "returns -1 (property 3)" in 2.7 and 4.3, and R5 and the OPT9-3 disposition require -1. No weaker K5 target is intended, so the contract is unchanged | 2.7 (property 3); 4.3 K5; 8.2 R5; 9 (Review 9 table, OPT9-3 row) |
| OPT10-1 | Applied. After a quarantine, -14 lasts until reboot or until the owning process exits: `Policy::exit` drops the pending read (`crates/virtio-core/src/lib.rs:788-790`) and `detach` clears `error` (`volume.rs:1017-1026`) *[main]* (read, not executed); the OPT9-3 disposition uses the same wording | 2.7 (property 3); 4.3 K5; 9 (Review 9 table, OPT9-3 row) |
| OPT10-2 | Applied. "(or event)" is dropped from the CODE-DELTA (revision 5) Q6 row, which now matches 2.7, the Q6 ledger row and 8.2 R2 | 9 (CODE-DELTA (revision 5), Q6 row) |
| OPT10-3 | Applied. The citations are `volume.rs:1068-1079` (the two quarantine paths) and `crates/virtio-core/src/lib.rs:577-579` (the filter), checked at `b9aa517c` | 2.7 (property 3) |
| OPT10-4 | Applied. The parenthesis states that the decided portal check would report the case as a failed export if the service check did not; the trailer comparison (b) remains the backstop for a service defect, as the following sentences and 8.1 O17 state | 2.7 ("Stale valid content") |
| OPT10-5 | Applied. The Q2 row is annotated "(confirmed by reviews 7 and 8 and approved by review 9; ...)", in the style of the Q3 and Q7 annotations | 9 (Revision 5 record, Q2 row) |
| OPT10-6 | Applied. O4 (a) points to the K5 delta of 4.3 and states that the approval asked for covers it; the delta is a stricter kernel obligation, not a widened capability | 8.1 O4 (a) |
| OPT10-7 | Applied. The final clause of R1 Status (revision 5) is reworded as proposed ("approved revision 5 (with those fixes), `a769bc65` and the review-6 corrections ... which the lead dispositioned in a separate fix pass") and records review 10 and the pending review 11 | 8.2 R1 |
| OPT10-8 | Applied. The edited prose lines of the header, 2.7 and the Revision 5 record intro are rewrapped; table rows stay single lines by format | header (revision 5); 2.7; 9 (Revision 5 record) |

These edits were checked by review 11 (below).

**Review 11.**
SPEC-REVIEW-11 is an independent AI review of the fix pass after review 10,
stored verbatim as
`docs/verification/s09-destination-service-20261123/SPEC-REVIEW-11.md`. It
confirmed RQ10-1 and RQ10-2 resolved and returned APPROVE with no required
finding and nine optional findings (OPT11-1–OPT11-9). None was applied, so
the body of revision 5 is the text that review 11 approved. OPT11-5 (store
SPEC-REVIEW-7 to SPEC-REVIEW-9 verbatim) is met by the revision-5 commit,
which adds SPEC-REVIEW-7 to SPEC-REVIEW-11. The other findings are deferred
to the next revision. This paragraph, the table below and the status
sentences that name review 11 (header, 8.2 R1, the Revision 5 record
introduction) were added by the lead after review 11 and have not been
reviewed.

| Finding | Disposition | Location named by the review |
|---|---|---|
| OPT11-1 | Deferred to the next revision (not applied; the text above is the text review 11 approved) | 9, the historical `a769bc65` paragraph ("These relabelled markers and this paragraph were not reviewed.") and the revision-5 note after it |
| OPT11-2 | Deferred to the next revision (not applied; the text above is the text review 11 approved) | 9, Review 9 table, OPT9-9 row |
| OPT11-3 | Deferred to the next revision (not applied; the text above is the text review 11 approved) | 9, Revision 5 record, Q12 and Q13 rows |
| OPT11-4 | Deferred to the next revision (not applied; the text above is the text review 11 approved) | 9, Q3 row annotation ("the portal check reports the export as failed") and Review 9 table, OPT9-5 row |
| OPT11-5 | Met without a text change: the revision-5 commit adds SPEC-REVIEW-7 to SPEC-REVIEW-11 verbatim | Header (revision 5) and 8.2 R1 Status (revision 5): review 9 "returned APPROVE" |
| OPT11-6 | Deferred to the next revision (not applied; the text above is the text review 11 approved) | 4.3 K5 ("one delta beyond per-device state"); 8.1 O4 (a) ("one stated delta") |
| OPT11-7 | Deferred to the next revision (not applied; the text above is the text review 11 approved) | 9, Revision 5 record, Q9 row, "Section changed" |
| OPT11-8 | Deferred to the next revision (not applied; the text above is the text review 11 approved) | 2.7 property 3: "and a further poll returns -14 (`volume.rs:900-906`, `err`)" |
| OPT11-9 | Deferred to the next revision (not applied; the text above is the text review 11 approved) | Header ("evidence; O18 as proposed," short line); 9 Revision 5 record intro ("review 9 is the first review to approve.") |

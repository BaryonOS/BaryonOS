# A03-T2c specification (prepared 2026-11-25, read-only analysis at e8f38b99)

Status: prepared, not implemented. Citations refer to e8f38b99; recheck after the
next main merge.

## Current volume authorisation

- The unit of authority is a volume *domain* (2 domains x 8 objects,
  `services/volume/src/lib.rs:33-34`, `domain_of` :117), bound to the
  kernel-supplied ingress endpoint, never a request field
  (`services/volume/src/wire.rs:1-3`, `service.rs:276-279`). The native adapter
  also drops messages whose kernel sender is not the ingress's bootstrap client
  (`sdk/examples/volume-docs/src/service.rs:97-106,158-174`).
- No per-object capability, ACL or handle exists: a domain client may read,
  write and delete all 8 objects. READ requires the current revision
  (`service.rs:1157-1166`). Client API: `volume_core::client::Transport`
  (`client.rs:14`), `Client::stat` (:476), `Client::read` (:647).

## Invariant

A task's IPC causes a volume READ of object O only while the broker holds a live
task-grant-core grant with actor = kernel-supplied sender PID, object (O, rev)
where rev is O's current volume revision, and right READ. Apps hold no volume
ingress handle and no block right. The broker constructs every volume request
itself (STAT at startup and after Stale, READ selector 0; nothing else) and
never relays app bytes; denial happens before
any volume message is sent.

## Design

The broker is the sole client of volume domain 0 (the `nas.rs` pattern,
`kernel/src/m2/m3/nas.rs:177-186,289-299`). At start it registers objects from
`Client::stat` via `Grants::register_object`; READ goes through the existing
`Broker::permitted` then `Client::read(id, granted.revision, ..)`; a Stale reply
calls `Grants::advance_revision` (retires the grant) and answers DENIED. CLOSE
and exit retirement stay broker-side; apps never held volume authority, so the
kernel has nothing to revoke. Object addressing matches document-session
`(object, loaded_revision)` (`sdk/document-session/src/lib.rs:270`).

Risk: the broker holds full domain-0 write authority (confused deputy if its
request construction is wrong); tests must pin READ-only request shapes.

**Agreed with S (25 Nov ~18:30Z):** broker-side READ/STAT restraint is not
enforcement, so it is not the product route. S builds a read-only ingress in
volume-core (fixed at profile construction; every op except STAT/READ refused
with Access before dispatch) and a deterministic read-only seed variant in
`tools/volume_disk.py`. T2c-2 runs natively ONLY over that ingress and image
(no broker seeding, no full-authority native fixture), creates no block
grants on device 1 and stays clear of the O13 latch. The broker's host tests
that pin request shapes remain as defence in depth.

## Slices

| Slice | Content | Kernel | Owner decision | Depends on |
|---|---|---|---|---|
| T2c-1 | Broker over `client::Transport`; stat registration; Stale retires; host tests against an in-process `Service` on a RAM disk (pattern `settings-volume/src/tests.rs:36`) | none | no | #535 |
| T2c-2 | Native `native-system` variant: new `kernel/src/m2/m3/task_grant_volume.rs` via path-swapped `mod system` (`m3.rs` ~98-120, 2-12 lines), 2 features in `kernel/Cargo.toml`, `tools/build.py` (:53,:99,:245,:477-491), new `tools/build_task_grant_volume.py`, `run_task_grant.py` cases `volume`/`volume-denied`; reuses m5 runtime and volume-docs ELF unchanged; **no `volume.rs` edit** | shared files (report to PR guard; S/H notice) | no | T2c-1, S read-only ingress + seed image, #539/N38 on main |
| T2c-3 | Product exit notification replacing the `observe_exit` fixture | yes | **yes** (A/B/C below) | T2c-2 |

T2c-2 native cases: VOLUME_READ_OK, UNGRANTED_OBJECT_DENIED (0 volume
requests), FORGED_ACTOR_DENIED, FORGED_GRANT_DENIED, NO_VOLUME_HANDLE,
CLOSE_THEN_READ_DENIED, EXIT_RETIRES + READ_AFTER_RETIREMENT_DENIED, optional
STALE_REVISION_DENIED; counters: volume READ requests > 0 only after an authorised READ and 0 for
every denied case (startup STATs, the post-Stale re-STAT and multi-READ objects
over 160 B make 'requests == authorised READs' false), volume `rejected` == 0; mutations (skip authorise, skip retire) must fail the gate.
Regression: `native-task-grant` normal/denied and `native-system-nas-probe`.
CI job addition is a `.github` change for the CI owner.

## Owner decision for T2c-3 (exit notification)

No process-exit wait interest or kernel-to-user event queue exists today
(`wait.rs:62` accepts SEND/RECV only); exits flow through `retire_authority`
(`m3.rs:1705,2788-2807`) -> `m4::process_exit` -> `Engine::exit`, which closes
endpoints the exiting PID owns (capability-core `lib.rs:1588-1640`).

- **A (recommended): reply-endpoint closure.** The broker's reply SEND targets a
  client-owned endpoint; on exit it resolves `Closed` (`lib.rs:1697-1699`,
  test :3116) and the broker retires that client's grants. No new ABI. A client
  dropping its own RECV only retires its own grants (fail-closed). Requires the
  owner to accept "an owned endpoint closes no later than its owner's exit" as a
  product contract with a pinned capability-core test. Detection is polled.
- **B: K05 Process object with OBSERVE for the broker.** Also grants KILL
  (not separable), conflicts with spawner ownership, and the orphan rule would
  end clients if the broker dies.
- **C: kernel exit-event channel** (new capability/ABI, ADR, kernel owner).
  Cleanest semantics, also covers fault/kill; largest change.

Unverified: exact `ipc_wait` error on a closed endpoint; whether fault/kill
paths reach the same cleanup (looks shared; only normal exit is tested).

## Evidence limits

QEMU x86_64 only; content from S's pre-seeded read-only image; trusted dialog remains a fixture;
one domain; no WRITE/EXPORT; bytes already read cannot be revoked.

## T2c-2 read-only interface (from S, 25 Nov ~18:40Z; verify against code before use)

- No volume-core or ELF change: the read-only mount mode already exists.
- Kernel (`task_grant_volume.rs`): create the volume block grant with
  `BLOCK_READ` only, as `nas.rs` RECOVER mode does (~197-203,
  `create_block_device_with_rights(.., BLOCK_READ)`), and set bootstrap word
  120 = 1 for the volume-docs ELF; WRITE/FLUSH become kernel capability refusals.
- Service: `Service::recover(epoch, read_only=true)` (`services/volume/src/service.rs`
  ~408-420) refuses every op outside {STAT, READ, LIST, COLLECTION_STAT,
  DOMAIN_STAT, VERSION_STAT, PROVENANCE_STAT} with `Status::Access` (-5) on both
  ingresses after the wire check, before dispatch (token upload path included).
- ELF needs two distinct non-zero client PIDs (words 160/168) and two ingresses
  (56/80): domain 1 needs its own (silent) client process; SHUTDOWN (word 104)
  need not be wired.
- Seed (S slice, `tools/volume_disk.py` variant `seeded-read`): domain 0 object 0
  `seed-a.txt` 4196 B (2 chunks) and object 1 `seed-b.txt` 300 B, revision 1,
  content `volume_disk.fixture(obj, 1, length)`; objects 2-15 absent; domain 1
  empty. Fixture use: grant object 0, leave object 1 ungranted.
- Runner assertions: image sha256 unchanged after the run; `volume_disk.analyse()`
  objects 0/1 unchanged. S reports commit and exact sums when green.
- Seed ready (S, 25 Nov ~18:55Z): local commit 6206b8c8 on
  `claude/s09-volume-seed-read-20261125` (worktree `BaryonOS-worktrees/s-seed`,
  base c04ea98d, not pushed; lands after #539/N38). `volume_disk.create_image(path,
  "seeded-read")`; fresh image sha256
  `204ae9cd6e9dea28dceedc8e6a97923d5595b6de06dbe3211ec243d11ed54420`; object 0
  fnv64 0x06a40a9cece84c71, object 1 fnv64 0x5716e11ae1dca304; analyse() domain
  digests [3816305023448471146, 10252960164691450465]. S evidence is host-only
  (volume-core `tests/seeded_read.rs`, `tests/test_volume_seeded_read.py`);
  native is A's T2c-2 stage. Stack T2c-2 locally on 6206b8c8 (cherry-pick) until
  it is on main.

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
itself (READ only, selector 0) and never relays app bytes; denial happens before
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

## Slices

| Slice | Content | Kernel | Owner decision | Depends on |
|---|---|---|---|---|
| T2c-1 | Broker over `client::Transport`; stat registration; Stale retires; host tests against an in-process `Service` on a RAM disk (pattern `settings-volume/src/tests.rs:36`) | none | no | #535 |
| T2c-2 | Native `native-system` variant: new `kernel/src/m2/m3/task_grant_volume.rs` via path-swapped `mod system` (`m3.rs` ~98-120, 2-12 lines), 2 features in `kernel/Cargo.toml`, `tools/build.py` (:53,:99,:245,:477-491), new `tools/build_task_grant_volume.py`, `run_task_grant.py` cases `volume`/`volume-denied`; reuses m5 runtime and volume-docs ELF unchanged; **no `volume.rs` edit** | shared files (report to PR guard; S/H notice) | no | T2c-1 |
| T2c-3 | Product exit notification replacing the `observe_exit` fixture | yes | **yes** (A/B/C below) | T2c-2 |

T2c-2 native cases: VOLUME_READ_OK, UNGRANTED_OBJECT_DENIED (0 volume
requests), FORGED_ACTOR_DENIED, FORGED_GRANT_DENIED, NO_VOLUME_HANDLE,
CLOSE_THEN_READ_DENIED, EXIT_RETIRES + READ_AFTER_RETIREMENT_DENIED, optional
STALE_REVISION_DENIED; counters: volume requests == authorised READs, volume
`rejected` == 0; mutations (skip authorise, skip retire) must fail the gate.
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

QEMU x86_64 only; broker-seeded content (a pre-seeded read-only image would need
an S-owned `tools/volume_disk.py` variant); trusted dialog remains a fixture;
one domain; no WRITE/EXPORT; bytes already read cannot be revoked.

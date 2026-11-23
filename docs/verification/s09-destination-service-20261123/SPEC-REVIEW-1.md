# S09 destination service: independent AI spec review 1

- **Subject:** `docs/implementation/S09-DESTINATION-SERVICE.md` at PR #403 head
  `bc3f63a172cefbe6a404ae9130b20e6eb7e3e8d5`, checked against `origin/main`
  `035b44e5`.
- **Reviewer:** an independent AI security-review agent (task S09-DEST-R1),
  23 November 2026. It ran read-only (`git show`, `grep`); it did not edit,
  build or test anything.
- **Status:** an AI review, not a human audit. The QEMU statements in its O9
  section come from the reviewer's recall of QEMU source. They are hypotheses
  to test, not evidence.
- **Verdict:** REQUEST_CHANGES, with 13 required items.

The review text follows verbatim. How each item is resolved is recorded in the
review record section of the specification.

---

S09-DEST-R1: independent security review of docs/implementation/S09-DESTINATION-SERVICE.md, PR #403 head bc3f63a1, checked against origin/main 035b44e5.
This is an AI review, not a human audit. Nothing was edited, committed or run apart from read-only `git show` and `grep`.

VERDICT: REQUEST_CHANGES

## REQUIRED

1. **Mount rule (2.6, lines 197-203) has no rule for a failed catalog read.** If a read Io/Timeout on a copy counts as "invalid", a sealed entry can be overwritten.
   - Scenario: a crash between barrier 2 and barrier 3 leaves A=n+1 {E1,E2} and B=n {E1} (lines 385-387). At the next mount, the read of A fails and A is treated as invalid. B wins, and `next_free` rolls back to E2's first sector. The next CREATE then writes over sealed E2, and step 4 targets "invalid" A.
   - Fix: any catalog read error at mount or remount is terminal `Io` (failed-mount state), never "invalid". When both copies are valid but differ, write the mirror before admitting any CREATE. Add an N row with a mutation.
2. **Safe-path trigger (lines 338-339) and confirmation (361-366) cover only `Unknown`.** Lines 280-284 also send a device `Io` at barrier 2/3 into a remount, and a later successful FLUSH then resolves the binding and readmits CREATE in the same boot.
   - Line 221 itself says a failed FLUSH proves nothing about earlier writes. On Linux hosts, a later fdatasync can succeed after a write-back error even though the data was dropped (general knowledge, not verified here).
   - Fix: trigger read-only-for-boot and "provisional until cold boot + VERIFY" on Io, Timeout or Unknown of any destination write or flush. Extend N13 to `Io`. Add the step-3 readback `Timeout` to the 2.5 failure edges.
3. **The safe path cannot be implemented as written.**
   - Lines 355-361 and O9 require the read-only state to bind a restarted instance and to survive "any reboot that is not a cold boot", with no mechanism. A guest cannot tell a warm boot from a cold one, and a new instance has no memory of the old one.
   - Lines 225-228 also say a "service restart" resumes normal operation.
   - A spawn path exists (kernel/src/m2/m3/supervisor.rs:290). I did not check whether it can re-grant block authority.
   - Fix:
     - (a) Put the latch in the per-device kernel broker (K5). Any mutating timeout or unacknowledged reset makes WRITE return -5 for every process until reboot; READ and FLUSH stay allowed. Add a native N row with a mutation.
     - (b) Require `-no-reboot` in every destination runner (already at tools/run_volume.py:309), so every reboot is a new QEMU process.
     - (c) Leave product warm-reboot behaviour open under O9 and remove the restart wording that conflicts with it.
4. **-8 wedge (lines 322-325).** The claim that the reset "ends within RESET_TIMEOUT either ready or offline" holds only for the lane that faulted.
   - With main's single Policy (kernel/src/m2/m3/volume.rs:124-126), a quarantine marks only that lane offline (1075-1079). The other lane gets -8 indefinitely (961-966; virtio-core lib.rs:566-568).
   - Effect: the service keeps resubmitting its FLUSH in its single loop, never serves UNBIND, and the revoke protocol stalls.
   - Fix: -8 for longer than RESET_TIMEOUT plus a margin counts as a failed FLUSH (`Io`). Make K5 a precondition for any profile where both lanes run. Add an N row.
5. **Transferred capabilities on requests are not handled (2.2, 4.3, 5.1).**
   - capability-core `receive` attaches every transferred cap into the receiver's slots unconditionally (crates/capability-core/src/lib.rs:1568-1580). If the receiver has no free slot it fails before dequeuing (1570-1571), so the message blocks the head of the lane.
   - Effects:
     - A portal whose domain volume handle carries TRANSFER can hand it to the destination service. That breaks the rule that the service "never holds a volume endpoint", and boot-time counting cannot see it.
     - The dialog's TRANSFER roots can exhaust the service's slots and kill a lane.
   - Fix:
     - Every lane requires `cap_count == 0`; the service closes (0x10) any received handle before parsing and replies `Invalid`.
     - Lane and volume handles held by the portal carry no TRANSFER.
     - Emit `DEST:HANDLES=` per stage.
     - Add an N row whose mutation keeps the received handle.
6. **Control-lane replies have no endpoint (4.3, lines 530-536; 5.1, lines 578-583).**
   - The service holds SEND only on the portal's reply endpoint, so the dialog cannot receive UNBIND `Ok`, which the revoke protocol waits for. Any added handle would fail N18's "exactly" count.
   - Fix: a dialog-owned reply endpoint with SEND delegated to the service; control replies go to the dialog, operation replies to the portal; update 5.1 and N18.
   - The handle lists also disagree: N18 counts Entropy twice ("5.1 handles plus one Entropy") and 5.1 leaves out the block handle.
7. **N18 checks counts only.**
   - Granting `BLOCK_RIGHTS` (lib.rs:22, which includes DERIVE, TRANSFER and REVOKE) leaves the counts unchanged and passes N18.
   - "Device or partition" (2.1) also allows a partition on the source device, which shares reset and FLUSH with the source and is not an independent backup.
   - Fix: N18 probes for exact rights, as ARCHIVE_READONLY_RIGHTS does (volume.rs:743-748): block handle READ|WRITE|FLUSH only, dialog control lane SEND only, portal lane handles SEND only. It asserts the destination device differs from every other block scope, as volume.rs:749-753 already checks `scope().device`. Add mutations for `BLOCK_RIGHTS` and a device-0 scope. v1 requires a separate device.
8. **Rebind replay (BIND row, line 119; N17).**
   - The rule "aux > every earlier one" has no storage rule and no refusal status. Revoke purges only queued messages that carry revoked caps (lib.rs:1957-1963), not plain data messages.
   - Scenario: re-BIND with the same g, for example after a dialog restart resets its counter. Queued old APPEND/SEAL messages then pass the epoch check, and the per-binding `request_id` restarts, so stale bytes can head or seal a new entry.
   - Fix: keep a per-lane maximum generation for the instance; BIND with aux ≤ max returns `Stale`; INFO reports the maximum; add this to N17 with a mutation.
9. **Status codes conflict, so tests can pass either way.**
   - Line 87 says validate every field "before any state", yet an unbound lane returns `Access` "for every request" (90).
   - N14 expects `NoDevice` on "any lane op", but the ID check happens at BIND, unbound lanes return `Access`, and a foreign non-blank layout is `Corrupt` under the mount rule.
   - The reply status of an operation refused in the `Corrupt` state is undefined, and so is the absent-device state (5.3 says `NoDevice`; 2.6 implies `Io`).
   - Fix: one total order, tested pairwise: cap_count, unbound, sender, header, epoch/request_id, operation permission, then the failed-FLUSH, `Corrupt` and safe-path states. State whether a rejected request advances the `request_id` high-water mark.
10. **Catalog is not validated before it is written (2.7 step 4).**
    - If the service commits a copy its own validator rejects (for example length > 525 328, `next_free` > formatted count, label length 0), step 5 mirrors it. Both copies are then invalid, the state is terminal `Corrupt`, and every backup becomes unreachable (O10 offers no recovery).
    - Fix: encode, run the mount validator, check the new copy extends the old one (same records plus one, seq+1), and only then write. APPEND refuses totals above 525 328. Add an N row with a mutation.
11. **Native "power cut" stages cannot fail on a missing barrier (N11, N12; §7 dest-cut-*).**
    - SIGKILL with `cache=writeback` (run_volume.py:314,321) keeps the host page cache, so a mutation that removes barrier 1 or 2 passes every native cut stage.
    - Fix:
      - Relabel these stages "process kill, no cache loss".
      - Assert `DEST:IO:FLUSHES` and the flush order at each cut marker.
      - Treat flush-order evidence as host-only (SimDisk) at R2, or add a replay oracle over QEMU's in-tree `blklogwrites` log. The oracle rebuilds every image the cache model allows and mounts each one.
12. **D3 gaps (4.2, 8.2).**
    - R5/R6 run `dest-export` and `dest-restore` before R4, so the portal holds a domain endpoint that is not read-only or create-only. R8 accepts N19/N20 as host tests only.
    - Revoking a remembered permission (3.3) revokes lanes but not the volume-side binding.
    - Neither the catalog record nor the archive header (S09-BACKUP-ARCHIVE.md:43) names an owner. With the planned per-user isolation (PROFILES-AND-USERS.md:232), the dialog's LIST-all would let user B restore user A's plaintext.
    - Fix: gate the R5 restore stage and R8 on R4; revoke the volume binding together with the lanes; declare v1 single-profile (the dialog refuses when more than one profile exists) until an owner principal is bound per destination or entry.
13. **Fixture boundary and stale labels.**
    - ADR-S09-DUALDISK B3 gives device 1 only to the slot-1 portal, with a scope that ends below the disk end (volume.rs:708-711). B5 says there is no dialog.
    - R5 moves device 1 to a destination service with a whole-device scope and adds a dialog driver. K4 covers production only, so R5 needs an explicit owner amendment of B3/B5.
    - The header (lines 18-26), §7 pins and K4 still call PR #396 unmerged. It merged as 8a7a3594: device-1 routing is in volume.rs:877-881, and the 22-stage pins (volume-acceptance.yml:62, kvm-pilot.yml:124) and archive-disk stages are on main. Rebase the labels on current main.

## OPTIONAL
- READ: require checked `offset+aux` and define a read that straddles the end (`Range`, no data). The portal checks the echoed epoch, request_id and op on every reply.
- The service never blocks on a reply send; on `Again` it drops the reply. The dialog revokes after an UNBIND timeout only together with a kernel write latch.
- Mount rule: when both copies are valid, the higher copy must extend the lower one (records a prefix, `next_free` not lower), else `Corrupt`.
- Increase the sequence by 2 on a boot's first commit, so a late catalog write from an earlier boot always loses.
- Layout row "gaps left by discarded writes" contradicts allocation at the committed `next_free`.
- Add table-driven op × lane tests from the 2.3 "Permitted on" column, plus CREATE twice per binding and LIST scoping.
- O1 input: the remembered permission keys on an unauthenticated ID. A physical attacker with a cloned catalog could receive automatic plaintext exports.

## O9: gap analysis
- **Restart in the same boot.** No mechanism exists. The broker resets the device when the owner exits with a request outstanding (volume.rs:1017-1026), but nothing stops a new instance from writing. See REQUIRED 3.
- **Warm reboot.** The guest cannot detect it, so `-no-reboot` is required for test profiles. In QEMU, a guest reset also resets the virtio device, so it is the same event as an in-boot acknowledged reset.
- **Failed mount FLUSH.** The "service restart" route must not apply after a mutating timeout or a barrier `Io` (REQUIRED 2 and 3).
- **Quarantined lane.** See the -8 wedge (REQUIRED 4). Under QEMU the reset is synchronous (below), so a non-acknowledged reset can only be produced by guest fault injection; that path has guest-side evidence only.
- **SIGKILL is not a power cut**, because the host page cache survives it (REQUIRED 11). With aio=io_uring, ring teardown may finish after the process exits (from recall, unmeasured).

The QEMU behaviour below is from my recall of the source and was not verified here; the experiment exists to test it. The guest writes status 0, which runs `virtio_blk_reset` synchronously inside that MMIO write, and that function drains in-flight I/O. A throttle filter is released during drain, so throttling alone only shows the benign path.

## O9: proposed experiment (do not run without owner scheduling)

**Common setup**
- Exact runner QEMU 10.x, pc-q35-8.2.
- Raw image on a local ext4 or xfs filesystem (not tmpfs).
- `-blockdev file,aio=threads|io_uring,cache.direct=off|on,locking=on`, `werror=report,rerror=report`, `-no-reboot`.
- QEMU trace events (log backend, timestamps) for virtio status writes and virtio-blk submit/complete, with names taken from `-trace help`.

**E1 (in-boot)**
- A test feature lowers TIMEOUT and emits `DEST:O9:ARM:<op>:<sector>`, `RESET:BEGIN`, `RESET:ACK` and `REMOUNT:FLUSH:DONE`.
- At ARM, the harness attaches `strace -f -tt -P <img> -e inject=pwritev,pwrite64:delay_exit=<µs>:when=1`. Use `fdatasync` instead for the FLUSH cell; check the strace version supports this. The delay must exceed TIMEOUT. This delays the write below QEMU's block layer, where drain cannot hurry it.
- Cells: step-4 sector 0, step-4 sector 3, step-5, barrier-2 FLUSH, APPEND entry write. Each crossed with both aio modes and both cache.direct settings. At least 20 runs per cell.
- End each run with QMP quit, then parse with `tools/destination_disk.py`.
- **Supports O9, for that exact configuration only, if every run shows:**
  - serial evidence of -16 and the reset path for the targeted request;
  - the delayed syscall entering before the status-0 write and returning before that write completes, or never issued;
  - no pwrite carrying pre-reset content to the targeted range after the remount fdatasync returns;
  - the guest's resolution equal to the oracle's parse of the final image.
- **Refutes O9:** any pre-reset write returning after the status-0 write completes or after the remount fdatasync, or a resolution mismatch.

**E2 (cross-boot)**
- Hold a pwrite as in E1, then SIGKILL; also SIGKILL during a write flood with io_uring and cache.direct=on.
- After `waitpid`, the image must stay byte-stable (hash and inotify) until the next QEMU's first write. A lock failure on restart is a failed start, not a pass.
- This shows only that "no request survives a process restart". It shows nothing about cache loss.

**Not evidence:** a delay shorter than TIMEOUT; no host-side trace; guest readback or markers alone; SimDisk; reading QEMU source; a single aio mode; tmpfs, `cache=unsafe` or `no-flush`; a VM stop; a different QEMU version, backend or options than the runner; any claim about physical media. The K4/K5 broker fence still needs its own native timeout fixture (R5).

## VERIFIED on origin/main 035b44e5
- crates/capability-core/src/lib.rs:
  - BLOCK_READ_RIGHTS includes DERIVE, TRANSFER and REVOKE (18-22); the legacy constructor uses it (1361-1363); unknown rights are rejected (1366-1376).
  - MAX_ENTROPY_OBJECTS=4 (59), MAX_OWNED_ENDPOINTS=4 (77), QUEUE_CAPACITY=16 (78); error codes Invalid -1, BadHandle -4, Access -5, Revoked -10 (146-155).
  - `send` requires TRANSFER and subset per transferred cap and stamps `sender_pid` (1505-1551); the new node is a child of the sender's node; `validate_send` requires `sender_pid` 0 (239).
  - `revoke` marks strict descendants only (1488-1503, 1896-1908); `maintain` purges only queued messages whose caps are dead or revoked (1953-1966); `receive` attaches caps unconditionally (1568-1580); RECV requires the owner (1302-1306).
- crates/virtio-core/src/lib.rs: `is_mutating` (56-58); writable negotiation requires FLUSH (290-297); a reset that is not acknowledged quarantines the Policy (560-569).
- drivers/virtio-blk/src/lib.rs: `no_flush` clears bit 9 (243-244); `begin_reset` and `reset_acknowledged` (511-525).
- kernel/src/m2/m3/volume.rs:
  - Test feature cannot build with native-system (15-18); RESET_TIMEOUT (25); single Policy for both lanes (124-126).
  - -8 while resetting, holding an error, or not Ready (964-966); timeout gives -16 for mutating, -13 otherwise (1087-1099); reset (1006-1016); acknowledgement path (1043-1074); quarantine -13 (1075-1079) or -14 (1068-1072); `detach` (1017-1026).
  - Device-1 routing only under the test feature (877-881); `archive_grant` counting, device check and scope-below-disk check (693-768).
- kernel/src/m2/m3/m4.rs: syscalls 0x10, 0x11, 0x12, 0x20, 0x21 (256-259).
- Volume crate: `Status` values in services/volume/src/lib.rs:52-69; wire body 224 bytes and data 160 bytes in services/volume/src/wire.rs:16,19; `archive::export` returns `Stale` if the source changes (archive.rs:113-116); 525 328-byte maximum stated in S09-BACKUP-ARCHIVE.md:155.
- Runners and pins: virtio-rng at addr 0x6 (tools/run.py:156); the archive disk is also at 0x6 (run_volume.py:322); `-no-reboot` (309); pins of 22 stages (volume-acceptance.yml:62, kvm-pilot.yml:124).
- Monotonic `next_pid` (kernel/src/m2/m3.rs:1059-1060).

## NOT CHECKED
- QEMU source and behaviour: the reset drain, `blklogwrites`, throttle, io_uring teardown, and strace's inject options.
- tools/run_keyring.py and kernel PCI discovery at other slots (O4).
- The `sdk/task-grant-core` and `services/net-broker` claims in 3.3, and the entropy flow after #405.
- Whether `supervisor` spawn can re-grant block objects, and whether the portal's existing volume handle carries TRANSFER.
- The full text of S09-BACKUP-DESTINATION-PROPOSAL.md and S09-BACKUP-ARCHIVE.md.
- Hosted Graphify was not used. No tests or gates were run.

Risk: the QEMU statements in the O9 section are hypotheses from recall and must not be cited as evidence. This review is an AI review, not a human audit.

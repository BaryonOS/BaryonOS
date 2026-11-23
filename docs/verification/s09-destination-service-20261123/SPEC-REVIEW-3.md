# S09 destination service: independent AI spec review 3

- **Subject:** revision 2 of `docs/implementation/S09-DESTINATION-SERVICE.md` (working tree on `22226cde`), checked against `origin/main` `035b44e5` and reviews 1 and 2.
- **Reviewer:** a fresh independent AI review agent (task S09-DEST-R3), 23 November 2026. It ran read-only.
- **Status:** an AI review, not a human audit.
- **Verdict:** APPROVE.

The structured output follows verbatim. The optional findings were applied after this review; section 9 of the specification records how.

## Per-finding judgement of review 2

- **F1:** gated-ok. The once-per-boot grant rule is stated by scope and marked [proposed; owner decision O13], with the kernel write latch as the alternative. N32 is added and runs natively at R5. R8 is blocked on O13, and section 9 item 3 now states the conditional basis. m5.rs:260-279/784-794 was confirmed. Residue: R3-2 (N32's observation channel) and R3-3 (stale unconditional wording in section 9).
- **F2:** resolved. All binding limits, the unknown export binding (INFO/LIST only) and the wrong-entry rule are in check 6. N13 is rewritten per lane for both remount outcomes and matches the check order and the check-7 table. Its kill mutation (check 7 before 6 gives Io) is observable. N31 adds the wrong-entry case in the failed-mount state.
- **F3:** resolved. The header rows for offset (SEAL total), count (BIND/UNBIND lane) and aux are corrected. The field-rule table matches every 2.3 row I checked (BIND length 16, entry 0 iff export, aux any; UNBIND count 1..=3; READ aux 1..=160). Check 2 lists its fields and defers catalog-dependent checks to the operation.
- **F4:** resolved. The check-7 table defines mounted, safe path, remount pending (no request received, which is consistent with N24(a) and the fence paragraph), failed mount (now including a FORMAT write or flush failure), Corrupt, and blank (INFO NotFound -19, FORMAT and UNBIND run). The INFO encoding for non-mounted states is defined. N8, N25, 5.3 and N24(a) agree.
- **F5:** resolved, by review 2's option (a). K5 is per-device broker state (Policy, resetting, reset_deadline, error, active, data buffer), confirmed against volume.rs:122-131 and :964. The service consumes each result before the next submission. The pending poll (-8 at volume.rs:911-912, confirmed) has its own bound, TIMEOUT + 2 x RESET_TIMEOUT. N26(d)/(e) and an R5 native fixture showing that disk 0 in flight does not block disk 1 are added. The widened K5 rides on the owner approval O4 (a), which is still open.
- **F6:** gated-ok. Reclassified as a test-profile R5 precondition, not K4. Option (a) is a test-profile kernel change. Option (b) is a deterministic test-only ID labelled DEST:ID:TEST-ONLY under O4 (c). 5.1 and N18 are adjusted, and native entropy evidence is an R8 precondition. run_keyring.py:32 addr=0x6 was confirmed. create_entropy was not re-grepped.
- **F7:** resolved. N28's kill claim is narrowed to distinguishable swaps. The equivalent mutants (checks 3/4/6 and the safe-path row, all Access) are listed. I also confirmed that reply suppression is sender-based, not check-based, so swaps 4 versus 6 stay equivalent.
- **F8:** resolved. DEST:IO:*, the flush order and the cut marker come from the kernel broker, keyed by the destination object key. DEST:HANDLES, DEST:GRANT and DEST:PORTAL:REPLYQ come from the kernel Engine. The service-reported lines are marked supporting only or are oracle-cross-checked.
- **F9:** resolved. R2 now covers N3–N15, N17 and N21–N31. R5 covers N1, N2, N16, N18, N27 and N32 with dest-format/export/export-2/restore/grant-once. R6 covers the QEMU levels of N3, N5, N6, N7, N8, N10, N14, N15, N17 and N22 and every remaining stage, including the cut stages. Residual nit: native N19/N20 required by R8 have no R step (R3-5 e, pre-existing).
- **F10:** not resolved. The spec permits the revoke on a kernel Closed -9, but on main the service's exit revokes the dialog's delegated nodes (Engine::exit lib.rs:1594-1627), and resolve returns Revoked -10 before Closed (lib.rs:1695-1700). The dialog therefore never obtains a permit. Review 2 asked for 'Closed or Revoked'. Optional, see R3-1.
- **F11:** resolved. An operation-lane request from any sender other than the portal gets no reply, whatever check refused it. N22 and N27 observe this through the kernel-emitted portal reply-queue length. Depends on the unspecified source of the portal's process ID (R3-4).
- **F12:** resolved, by review 2's second alternative. The window after a crash between barriers 2 and 3 is stated explicitly as an accepted residual of the decay mitigation, with a reason for not adding a mount-time mirror. The decline of the mirror holds.
- **F13:** resolved. 2.1 and N18 cite volume.rs:749-753 as a precedent for comparing scopes by check_block, not as a different-device rule.
- **F14:** resolved. A refusal by checks 1–6 leaves the mark unchanged. Passing checks 1–6, including a later refusal by check 7 or by the operation, advances it. N17 tests both directions with observable mutations.
- **F15:** resolved. Syscall 3 under monotonic-clock (m3.rs:2366-2370) was confirmed. monotonic-clock = [milestone-m3], and native-volume = milestone-m6a, which chains through m5, m4 and m3, so the feature dependency is satisfiable. Whether any compile_error! blocks the combination stays unchecked and is correctly listed as an R5 precondition. The UNBIND latency in N26(b) is at most two bounds (latency wording nit in R3-5 f).

## Findings

### R3-1 (optional)

- **Location:** S09-DESTINATION-SERVICE.md 4.3 'To revoke' (lines 875-879); section 9 F10 row (1359)
- **Finding:** The F10 fix names the wrong kernel error, so it never fires. On 035b44e5, `Engine::exit` (capability-core lib.rs:1588-1628) marks every live node that descends from a node issued by, or held in a slot of, the exiting process as `revoked = true`. The dialog's control-lane SEND and lane roots are created by `bootstrap_delegate` from the service's endpoint (the volume.rs:594-603 pattern), with the parent = the service's node and issuer = the service, so exit revokes them. `resolve` (lib.rs:1690-1701) checks `revoked` before `object_open`, so the dialog's UNBIND `send` returns `Revoked` -10, not `Closed` -9. The cited '[main, maintain, resolve]' basis is therefore inaccurate. Under the rule as written the dialog never receives a permit: it repeats UNBIND forever and never takes its revoke step. The portal's lane handles are already revoked by the exit, so this is a liveness defect, not a capability leak. It also leaves the trigger for ending the coupled volume-side binding (3.3, 4.1) without an event.
- **Fix:** Permit the revoke on a kernel `Revoked` -10 or `Closed` -9 from the UNBIND `send` (review 2's wording). Cite `Engine::exit` (lib.rs:1594-1627) for the revocation of descendants on owner exit, and add a host `Engine` case (service exit, then dialog UNBIND send returns -10, then dialog revoke succeeds or is a no-op) to N6 or a new row.

### R3-2 (optional)

- **Location:** N32 Expected and Kills (line 1025)
- **Finding:** The stated kill observation does not hold on main. `Engine::exit` ends by calling `maintain()`. That reclaims the exited service's node and then, at nodes == 0, the block object itself (lib.rs:2003-2012), so DEST:GRANT:OBJECTS/NODES/HANDLES fall to 0 inside the exit call. An M5-pattern re-creation afterwards returns them to 1, which equals the pre-exit sample. Comparing 'before and after the exit' with 'stay equal or fall' therefore passes the mutant, and the row's claim 'observed as an increase of DEST:GRANT:OBJECTS=' is false against that baseline. The mutant is still caught by the separate clause 'no process holds a handle on such an object', but only if that clause is sampled after the restart window, and the row does not say when it is sampled.
- **Fix:** State the expectation exactly: after the exit all three counts are 0 and remain 0 at every poll round until the profile ends. Alternatively emit (object key, generation) and require that no key or generation absent before the exit appears. Change the Kills text to 'observed as DEST:GRANT:HANDLES= > 0 or a new object key/generation after the exit'.

### R3-3 (optional)

- **Location:** Section 9, 'Review 1, optional items ... Not taken' (lines 1328-1330)
- **Finding:** Leftover of the false claim behind F1. The text still declines the dialog-revoke-plus-kernel-latch item 'because no in-boot re-grant path exists', unconditionally. That contradicts 2.1, 2.7 'Carrier', O13 and the corrected section 9 item 3, which all state that m5.rs:260-279 shows kernel profile code can re-grant, and that the kernel latch remains the owner's alternative.
- **Fix:** Reword it as 'not taken because, under the proposed once-per-boot grant rule (2.1, O13), no in-boot re-grant path exists; the kernel latch remains the owner alternative under O13'.

### R3-4 (optional)

- **Location:** 2.2 Replies (230-236), 4.3 (837-840), 2.3 BIND row
- **Finding:** Reply suppression and check 4 both depend on the portal's process ID ('fixed for the boot by the manifest'). The spec gives no channel by which the service learns it. BIND carries only the destination ID in its data, and suppression on unbound lanes needs the ID before any BIND. The design is implementable, but the source of this value, which is security-relevant, is unspecified.
- **Fix:** State how the portal's process ID reaches the service. For example: the manifest writes it into the service's start context; or BIND carries it, but then only the dialog asserts it, and on unbound lanes a manifest-provided value is still needed. Add it to N18's manifest checks.

### R3-5 (optional)

- **Location:** 2.2 check 2 text (line 174), check-7 table failed-mount row (206) vs 2.6 Format (443-445), 2.5 reply codes, 8.2 R5 vs 4.3 K4, R8
- **Finding:** Minor consistency nits. (a) Check 2 refers to 'the field-rule table below', but the table is above. Precedence between a bad version and a bad operation (`Invalid` or `NoSys`) is unstated. (b) The failed-mount row's 'read of sectors 0–15' could be read to include a read error during the FORMAT scan, which 2.6 says leaves a blank candidate. (c) The 2.5 reply code for a kernel -14 or offline -13 on an entry write is unstated (by 5.3 presumably `Io`). (d) R5's precondition 'O4 (a) approves K4' conflicts with 4.3 'K4 covers production profiles only'. (e) R8 requires native N19/N20, but no R step or stage produces them (pre-existing). (f) The 'Time source' UNBIND latency counts the fence as a submission bound; a timed-out failed request is limited by the poll bound (TIMEOUT + 2 x RESET_TIMEOUT).
- **Fix:** (a) Say 'above' and give the order within check 2 (version, then operation, then the rest). (b) Write 'mount or remount read of sectors 0–15'. (c) Map kernel -14 and offline -13 on writes to reply codes. (d) Drop K4 from R5 or say 'K4 not needed for R5'. (e) Name the R step or stage for native N19/N20. (f) Restate the bound as one poll bound plus the remount's submission and poll bounds.

## Not checked

- Hosted Graphify (recall, impact, tests_for) was not used. Code claims were checked with git show/grep at origin/main 035b44e5 only.
- create_entropy occurrences in kernel/src/m2/m3/ were not re-grepped (F6 claim taken from review 2).
- Whether monotonic-clock combines with native-volume and test-volume-archive-device without a compile_error!, and kernel PCI discovery at a slot other than 00:06.0.
- QEMU behaviour of any kind (reset drain, blklogwrites, io_uring teardown, cache semantics). The spec's QEMU 10.2.0 source reading is correctly labelled not evidence; I did not reproduce it.
- The 4.2 volume-service claims, and the sdk/task-grant-core and net-broker claims of 3.3.
- ADR-S09-DUALDISK-20261119 B3/B5 text and S09-DUAL-DISK-FIXTURE-REQUEST.md.
- The diff was reviewed by its removed lines plus a full read of revision 2. No line-by-line diff of the added lines against revision 1 beyond that.
- Whether m5.rs restart logic is reachable in a native-volume profile (native-volume chains to milestone-m5); this matters only to the plausibility of the N32 mutant.
- No builds, tests, gates or QEMU runs were executed. This is a read-only AI review, not a human audit.

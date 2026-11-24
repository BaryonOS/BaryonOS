# S09 L2 spec review 1 (independent AI review, not a human audit)

The reviewed text is revision 0 of `docs/implementation/S09-L2-PROVENANCE.md`, sha256
`6b8d4fbd5e2646433ebd34908e82715eb36e7d81c30b8b4ab2ac3fac6f508978`, at
`837619ef`. Reviewer: a read-only AI subagent (`spec-review`) of the 19 November 2026
workflow run `wf_e109afb9-253`. That run ended before its result was applied.
The text below was recovered verbatim from the run journal on 23 November 2026.
Revision 1 of the spec (section 10) records how each required item is resolved.

---

VERDICT: REQUEST_CHANGES

Scope: a read-only review of docs/implementation/S09-L2-PROVENANCE.md, checked against /home/simon/projects/BaryonOS-worktrees/s09-l2-provenance at 837619ef (services/volume/src/{table,service,wire,client,upload,archive,lib}.rs, tools/volume_disk.py, the anchors cited in tests/history.rs:391 and tests/service.rs:840 and :1079) and against S09-RESTORE-FIDELITY.md 3.1-3.4 and S09-ARCHIVE-V2.md section 7. This was an independent AI review, not a human audit. No code was built or run.

## Required changes

1. **Resume contradicts the R3 precondition (spec :195-197 vs :214-219).** `restore_native_l2` checks `never_used` and returns `Stale` before the first write. After the first INSTALL_COLLECTION or INSTALL the domain is used, so the per-slot resume in section 5 can never run.
   - Failure scenario: the restore is cut after INSTALL k of m. Every rerun fails with `Stale`, and the domain is stuck partly restored.
   - Fix: either drop resume (an interrupted L2 restore goes to a fresh domain, consistent with L1 today), or define a separate resume entry point with a per-slot and per-collection precondition and get an explicit OWNER ruling. R3 says "never-used domain", and nothing can prove that only this restore used the domain, because the ownership control is still open.
   - Note: object reuse by equality only does not write, so it is harmless. Collection reuse does write objects into a collection this restore may not have created; the spec already makes that a conflict.

2. **The session rules for v6 contradict each other (spec :129-133 vs :154-157; service.rs:385).** One passage says v6 `handle` keeps "same-domain replace of an untokened session". Another says "BEGIN while an install is staged … is `Busy`". Today `begin()` calls `self.sessions[domain].take()` before any check, so a v6 BEGIN, even a stale one, would silently drop an untokened install session.
   - Fix: state one matrix, (v6 or v7) × (BEGIN or INSTALL_BEGIN) × (staged upload or install, tokened or untokened). Then say explicitly that `begin()` must check the session kind before `take()`.

3. **Name every code path the token and principal binding must reach (spec :129-133, :160-162).** The spec only says the upload.rs rule is "extended". Four places hard-code upload opcodes:
   - service.rs:178-187: the legacy `Busy` guard covers only BEGIN, APPEND, COMMIT and ABORT. If 20 and 21 are not added, a v6 caller can append to or commit a token-bound install session. That breaks the invariant tested by "v6 cannot touch a token-bound install".
   - service.rs:224-252: `handle_for` issues a token and binds the principal only for `BEGIN`, and checks principal and token only for APPEND, COMMIT and ABORT. A v7 INSTALL_BEGIN would otherwise be staged with principal 0 and token 0, and a v6 peer in the same domain could then take it over.
   - upload.rs:43: the set of operations that require a token.
   - upload.rs:59: a token is required only on a successful `BEGIN` reply.

   Also state that install tokens use the same per-domain `next_upload_token` counter, so a token is never reused.

4. **Specify how install sessions store state and handle partial sectors (spec :161-176; service.rs:19-36).** `Session` holds `chunks`/`sums[MAX_CHUNKS]` (16 entries), one `content_sum`, one name and one 512-byte `sector` buffer. An install needs up to 3 × 16 = 48 chunks, three sums, names and lengths. INSTALL_COMMIT "writes the last partial sectors" (plural), which implies up to three buffered sectors.
   - Fix: define the session layout. Recommended: write version i's final partial sector at the APPEND that completes it. This is safe because the chunks are unreferenced and reserved, and an `Io` there is a certain abort, so one buffer is enough.
   - Fix: state the size and native stack effect of the tagged session enum. Both variants occupy `sessions[DOMAINS]`.
   - Fix: define how a zero-length version (`chunks_for(0) = 0`) is sequenced and tested.

5. **The permanent domain-wide loss of v2 export is a same-domain backup denial-of-service (spec :221-226).** Any app with write access to the domain can INSTALL one object into a free slot, for example with n = 1 and arbitrary provenance. Because `installed` is never cleared, `export_v2` is then disabled for the whole domain forever, even after DELETE and PURGE. Before this change, no ordinary write could disable backup.
   - Two further facts: the excess is transient (each later commit that does not touch the object closes it by one, so `revision ≤ clock` eventually holds again), and the refusal is client-side only. Already-deployed `export_v2` clients will still export an installed domain whenever `revision ≤ clock`, and will otherwise return `Stale`, which callers may retry forever.
   - Fix: replace the rule with a bounded one. For example: refuse only while some object has `revision > clock`, with a distinct, non-retryable status; or refuse or omit per slot. Then state that the refusal is not a security boundary.
   - Fix: specify how `export_v2` learns `installed` (PROVENANCE_STAT, where `NoSys` from an older service means nothing is installed), and how that read stays inside the snapshot that the final `domain_stat` comparison checks.
   - If the owner keeps the permanent refusal, record it as an accepted availability trade-off.

6. **Define "re-create" per operation (spec :93-96).** The spec says object origins are "zeroed by … COMMIT that makes an absent entry present". RESTORE of a tombstone (service.rs:658-671) also makes an absent entry present, and the spec says origins survive it. TRANSACT cannot re-create an entry.
   - Fix: list, for each operation (upload COMMIT, RESTORE on an absent entry, RESTORE on a present entry, DELETE, PURGE on a present or absent entry, TRANSACT DELETE, TRANSACT PURGE, META, RENAME), the exact effect on the object origins, the version origins and `installed`.
   - Fix: add one test per row. Cover the case INSTALL → DELETE → COMMIT (re-create) → RESTORE of an installed-era history version: the version origin travels while the object origins are zero.

7. **INSTALL_COLLECTION has no rule for a zero origin, and invariant 6 is weaker than the lifecycle (spec :141, :178-179, :82, :98).**
   - Fix: require `origin_revision ≠ 0` in `validate_request` (`Invalid`). A zero origin would otherwise write a v3 copy with no provenance, and the collection would look exactly like one created by an ordinary COLLECTION.
   - Fix: tighten invariant 6 to "collection origin ≠ 0 ⇒ revision ≠ 0 and present", since the origin is zeroed on delete.
   - Fix: give the exact data layout and how the v6 `named()` helper is bypassed. Today `named()` requires `name_length == length`, but INSTALL_COLLECTION sends `length = k + 8`.

8. **Specify the remaining wire shape checks (spec :137-139, :144-152).**
   - INSTALL_APPEND: `count ≤ HISTORY` in `validate_request`, and `count` must equal the target version (otherwise `Range`).
   - INSTALL_BEGIN: header `name_length = 0`; each 45-byte record's name padding is zero; the name is non-empty; no Σ overflow.
   - INSTALL_BEGIN should also check that `collection` is 0 or present (`Range`), so it fails before chunks are reserved. COMMIT re-checks this anyway.
   - Each new reply needs its own `client::validate_reply` arm. Today the arm for COMMIT and similar operations expects `revision == request.revision + 1`, but an install reply carries `revision = n`. PROVENANCE_STAT reuses `name_length` as a count.

9. **The downgrade tests cannot be done as written (spec :116-117, :281-283).** "Current code and volume_disk.py on a v3 pair → `Corrupt`" stops being testable once both have been changed to read v3.
   - Fix: keep a frozen v2-only decoder. Options: a pinned copy of the pre-change `decode_sector` and `choose` in the tests; the v2 branch of `volume_disk.py` kept selectable; or v3 fixture bytes run against the merge-base binary.
   - The tests must show that a v2-only reader returns `Corrupt` on each of these, and never falls back to `g − 1`:
     - (v2 g−1, v3 g)
     - marker + v3 at generation 1 (INSTALL as the first commit)
     - (v3 g, torn v3 g+1)

10. **Missing negative and crash tests (section 8).** Add:
    - `Io` on an INSTALL_APPEND content write and on the INSTALL_COMMIT partial-sector write or FLUSH: certain abort, chunks released, no table write.
    - INSTALL n = 3 as the very first commit at generation 1, so `revision = generation + 2` lies exactly on the bound. Add mutants: excess computed as `n` instead of `n − 1`, and excess applied to an entry with `installed = 0`.
    - Corrupt `installed` values: 4, or 0 together with `revision > clock`.
    - Cross-domain reservation: domain B's staged chunks are never handed to domain A's INSTALL. Cover `Full` versus `Quota` with Σ > 16 chunks for one object (for example 3 × 13 = 39 passes, 3 × 14 = 42 gives `Quota`).
    - SHUTDOWN and `detach` with a staged install, both tokened and untokened.
    - The read cache after INSTALL, RESTORE and READ of each history slot.
    - PROVENANCE_STAT in read-only mode, and its gap behaviour after INSTALL → DELETE → RESTORE (undelete).
    - v1 export of an installed domain.
    - The `export_v2` behaviour decided under item 5.

## Optional

- Rule R versus the 3.2 rule (OWNER). The R2 break of the 3.2 rule is confirmed: INSTALL 3 → DELETE → WRITE gives history 4/3/2 with consecutive revisions, so pre-delete versions become nameable. Rule R keeps `Entry::validate`, STAT `version ≤ revision` and the deletion gap intact. I recommend rule R. Two things should be stated plainly:
  - It changes the meaning of "revision": one commit can raise a revision by n, and version revisions 1..n−1 never matched any committed state of this destination. They are destination-assigned, not source values, so 3.4 still holds.
  - The bound proof holds because each commit raises a given object at most once, TRANSACT included (validated distinct indices, wire.rs).
- Add decode invariants: `installed ≥ 1 ⇒ revision ≥ installed`, and `origin_revision ≠ 0 ⇒ installed ≥ 1` is already listed.
- R3 is enforced at domain level by the client only. The service enforces only "slot revision 0", so any app in the domain can INSTALL into a free slot of a used domain. Say this, mirroring the L1 restore.
- Downgrade consequence: after the first INSTALL, rolling back to a binary that does not read v3 makes the volume unmountable (fail-closed, `Corrupt`). Link this to the update and rollback policy as an owner-accepted consequence.
- `SlotScan` needs to carry `format` so that `choose` can apply `format(newer) ≥ format(older)`. Both copies decode into the same scratch table today. The admission check in service.rs:961 must compare `format` and the new `Entry` fields.

## Verified (holds as specified)

- The layout uses only bytes that v2 requires to be zero:
  - header 56..120 within 56..488 (table.rs:607-610), with DOMAINS × COLLECTIONS = 8 u64;
  - head 280..320 within 280..488 (table.rs:256-259);
  - history 432..448 within 432..488 (table.rs:272-276);
  - TABLE_SECTORS = 34 and DATA_START are unchanged.
- An unmodified v2 reader on a uniform v3 copy fails the version check. `compatible_with` is then false (table.rs:825-827), so the mount is `Corrupt`, including for marker + v3 at generation 1, because `first_attempt` requires a non-uniform copy. A torn v3 attempt next to v2 `g` recovers `g`.
- The R2 failure of the 3.2 rule and the STAT incompatibility (client.rs:78, `version <= h.revision`) are both real.
- The rule R bound, `revision ≤ clock + n − 1`, is sound. At INSTALL, `n ≤ c + n − 1` for `c ≥ 1`. After that, each commit raises the clock by one and a given object by at most one; `touch` (service.rs:609-626) and TRANSACT's distinct indices guarantee this.
- RESTORE uses `take_history` then `push_history` (service.rs:658-671), so a version's origin moves rather than being copied, and pairwise-distinct origins hold.
- `version_stat`'s prefix rule (service.rs:784-794) still hides versions behind a delete under rule R.
- A purged slot keeps revision ≠ 0 (`purge` service.rs:693-696, then `touch`), so INSTALL into a used, tombstoned or purged slot gets `Stale`.
- No path before state can reach a foreign slot: wire.rs checks `NoSys` first, then range, then domain → `Access`.
- `GUARANTEED_FREE` reasoning still holds, because there is one session per domain and `reserved()` covers sessions across domains.
- `volume_disk.py` hard-codes version 2, `revision ≤ clock` and the zero-padding rules (:218-262). The format marker records version 2 (:86-97).
- The anchors cited from tests (history.rs:391, service.rs:840 and :1079) exist.

Not done: no tests, mutants or builds were run (read-only review), and the native adapter dispatch was not inspected beyond a grep showing no other user of the opcodes.
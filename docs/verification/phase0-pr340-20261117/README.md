# PR340 — limited portal lint and wording correction

The project owner released only this bounded repair in a separate clone. The portal now checks `length > 0 && length.is_multiple_of(SECTOR as u64)`. The module wording limits name rejection to the checked rules and explicitly excludes8.3 aliases/path-length coverage. Existing owner worktrees, WIP, runners, recovery format and D1–D5 are unchanged.

The exact original release/unknown-none/native,archive-portal Clippy variant reproduces manual_is_multiple_of with exit101 before the fix and passes with exit0 after it. The first host-target invocation was invalid for native-only bins and is separately retained. Existing volume-core93 host tests and53 Python volume tests pass; independent read-only review confirms both source hashes and unchanged predicates/authority. [Commands, hashes and limits](checks.json).

The complete15-stage native gate and new fingerprint are pending after this source checkpoint. Historical run14 is preserved as historical evidence, not claimed as acceptance of this head. Existing kernel-profile Clippy debt is not repaired or hidden by this change. No full S09 acceptance, target-platform extraction, physical/user/human audit or new points are claimed. No merge or auto-merge approval is implied.

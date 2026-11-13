# Repository maintenance state

Branch-local state of the maintenance and security session (Session C). It records
evidence and the next step so the work can resume without a repository-wide
re-triage. It is not a roadmap and does not change any progress criterion.

## Session boundaries

- Owned elsewhere, read-only here: PR #318 and branch `feat/s09-native-archive`
  (S09, OS-development session); branch `claude/baryon-os-ui-ux-design-152adb`
  (UI/UX session). PR #317 is merged; its evidence is not edited.
- No merge without explicit approval of the project owner. This session does not
  enable auto-merge and does not push to a branch whose PR has auto-merge armed.
- Pull requests are updated by merging `origin/main` into the head branch; no
  rebase or force-push.

## Snapshot — 13 November 2026, 21:00 (Europe/Amsterdam)

`origin/main` at `bf4aa716` (#319). Latest run per workflow on `main` is green,
including `native-window-session` on `c76daf7b` (run 43482464397). The only red
run on `bf4aa716` is the GitHub-managed `Copilot` reviewer (run 43501362868): its
agent firewall blocked a download. This is reviewer infrastructure, not a
repository workflow, and is not a merge gate.

Dependabot: 0 alerts in any state. Code scanning: 26 open alerts (CodeQL).

## Security findings

| Alert | Rule | Location | Classification | Action |
|---|---|---|---|---|
| #1, #2 | js/xss-through-dom | `ui/prototype/app.js:315`, `:453` | Confirmed missing escape, one root cause. SARIF flow: `element.dataset.value` (line 379) sets `collection`, which `materialResults()` interpolates unescaped into an `<h2>` (line 154) that reaches both `innerHTML` sinks. Values currently come from the constant `COLLECTIONS` list, so not exploitable today; the `data-value` attribute (line 147) is also unescaped. Host prototype only; not in `source_hash()`. | Fix prepared (see below). |
| #3 | js/file-system-race | `services/nas/store.mjs:26-28` | Confirmed check-then-use: `lstatSync` validates type and size, then `readFileSync` reopens the path, so a swap between the two calls bypasses the symlink and size checks. Host NAS prototype; inside `source_hash()`. | Fix prepared (see below). |
| #4–#12, #20, #21, #24, #25, #33 | rust/hard-coded-cryptographic-value | `ui/native/tests/session.rs`, `services/keyring/tests/{kdf_hygiene,secure_records}.rs` | Test fixtures: synthetic password bytes (`b"synthetic-only"`, `b'x'`, `b"123456"`), zero-initialised output buffers and public domain-separation labels in a KDF regression test. No production secret. | Proposed dismissal "used in tests"; owner decision. |
| #14, #15, #17 | same | `services/keyring/src/lib.rs:124,132,141` | False positive: zero-initialised output buffers that `Hkdf::expand` or the ChaCha20 keystream overwrite before use. | Proposed dismissal "false positive"; owner decision. |
| #16 | same | `services/keyring/src/lib.rs:140` | By design: fast-key-erasure DRBG (Bernstein, 2017) uses a fixed all-zero nonce, sound because every key is used for exactly one keystream call and then replaced (`fill`, `reseed`). | Proposed dismissal "false positive" with this invariant; owner decision. |
| #23 | same | `ui/native/src/session.rs:278` | False positive: zero initialisation of the password input buffer, which is zeroised on clear and drop. | Proposed dismissal "false positive"; owner decision. |
| #26, #27, #31, #32 | rust/access-invalid-pointer | `boot/loader/src/{graphics,signed,main}.rs` (#32 is in `main_default.rs` through `include!`) | False positive: every SARIF flow starts at a `ptr::null_mut()` out-slot that firmware fills, and every dereference is preceded by `aligned()`, which rejects null and misaligned pointers (`main_default.rs:45`, `protocol()` at 217–229, `root` at 264, `information` at `graphics.rs:43`). CodeQL does not model `aligned()` as a sanitiser. | Proposed dismissal "false positive"; owner decision. |

No exposed credential was found, so no rotation is required. No Rust code is
changed to silence the scanner; no alert has been dismissed by this session.

## Pull request triage

| PR | State | Classification | Next action |
|---|---|---|---|
| #318 | Draft, S09 | Active work owned by the OS-development session | None. |
| #313 | Draft, conflict in `docs/progress.json` | Documentation cleanup; still needed: the hosted Graphify index lists only `BaryonOS/BaryonOS` | Merge `origin/main`, resolve, verify. |
| #316 | Draft, conflict in `docs/progress.json` | CI flake hardening for the 200 % window gate (main is green again on `c76daf7b`) | Merge `origin/main`, resolve, verify. |
| #315 | Draft, conflicts in `actionlint.yaml`, `DECISIONS.md`, `PROJECT-STATE.md`, `progress.json` | CI/security maintenance; adds a workflow that reads alerts | Review token permissions and published output, then resolve. |
| #304 | Ready, behind, auto-merge armed | Feature (A02); all checks pass on its head | Hold: updating the branch would trigger an unapproved auto-merge. Approval queue. |
| #302 | Ready, bot-authored, conflicts, auto-merge armed, `host-quality` and `experimental-m3-m4-smp` fail | CI/integration blocker; overlaps #315 in `ci.yml` and `actionlint.yaml` | Diagnose read-only; blocked. |

## Issues

Not yet reviewed in this session: #283, #277, #274, #43, #42, #41, #39.

## Approval queue

Empty.

## Next step

Prepare the escape fix for alerts #1/#2 and the file-descriptor read for #3 on a
branch from `origin/main`, run `node --test --test-isolation=none
ui/prototype/tests/*.test.mjs services/nas/tests/*.test.mjs`, and open a draft PR.

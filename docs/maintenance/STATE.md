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
| #1, #2 | js/xss-through-dom | `ui/prototype/app.js:315`, `:453` | Confirmed missing escape, one root cause. SARIF flow: `element.dataset.value` (line 379) sets `collection`, which `materialResults()` interpolates unescaped into an `<h2>` (line 154) that reaches both `innerHTML` sinks. Values currently come from the constant `COLLECTIONS` list, so not exploitable today; the `data-value` attribute (line 147) is also unescaped. Host prototype only; not in `source_hash()`. | **Fixed by #320**, merged as `f9c3bb7d`. CodeQL JavaScript analysis of `refs/pull/320/merge` (`cc55e4f2`) reported 0 results (3 on the previous `main`). The alerts close when CodeQL on `main` analyses `f9c3bb7d` (run 43509417219). |
| #3 | js/file-system-race | `services/nas/store.mjs:26-28` | Confirmed check-then-use: `lstatSync` validates type and size, then `readFileSync` reopens the path, so a swap between the two calls bypasses the symlink and size checks. Host NAS prototype; inside `source_hash()`. | **Fixed by #320** (descriptor-based read, `O_NOFOLLOW`/`O_NONBLOCK`, two regression tests); same CodeQL evidence. |
| #4–#12, #20, #21, #24, #25, #33 | rust/hard-coded-cryptographic-value | `ui/native/tests/session.rs`, `services/keyring/tests/{kdf_hygiene,secure_records}.rs` | Test fixtures: synthetic password bytes (`b"synthetic-only"`, `b'x'`, `b"123456"`), zero-initialised output buffers and public domain-separation labels in a KDF regression test. No production secret. | Proposed dismissal "used in tests"; owner decision. |
| #14, #15, #17 | same | `services/keyring/src/lib.rs:124,132,141` | False positive: zero-initialised output buffers that `Hkdf::expand` or the ChaCha20 keystream overwrite before use. | Proposed dismissal "false positive"; owner decision. |
| #16 | same | `services/keyring/src/lib.rs:140` | By design: fast-key-erasure DRBG (Bernstein, 2017) uses a fixed all-zero nonce, sound because every key is used for exactly one keystream call and then replaced (`fill`, `reseed`). | Proposed dismissal "false positive" with this invariant; owner decision. |
| #23 | same | `ui/native/src/session.rs:278` | False positive: zero initialisation of the password input buffer, which is zeroised on clear and drop. | Proposed dismissal "false positive"; owner decision. |
| #26, #27, #31, #32 | rust/access-invalid-pointer | `boot/loader/src/{graphics,signed,main}.rs` (#32 is in `main_default.rs` through `include!`) | False positive: every SARIF flow starts at a `ptr::null_mut()` out-slot that firmware fills, and every dereference is preceded by `aligned()`, which rejects null and misaligned pointers (`main_default.rs:45`, `protocol()` at 217–229, `root` at 264, `information` at `graphics.rs:43`). CodeQL does not model `aligned()` as a sanitiser. | Proposed dismissal "false positive"; owner decision. |

No exposed credential was found, so no rotation is required. No Rust code is
changed to silence the scanner; no alert has been dismissed by this session.

## Pull request triage

| PR | State | Classification | Done in this session |
|---|---|---|---|
| #318 | Draft, S09 | Active work owned by the OS-development session | None (read-only). |
| #320 | **Merged** `f9c3bb7d` (21:17 UTC) | Security fix for alerts #1–#3 | Opened from `main`; Node suites: prototype 41/41, NAS 75/87 (the 12 failures are N02 real-TLS tests that fail identically on unmodified `main` with local Node 22; CI pins Node 24.22.1). Merged at head `77f59101` with required checks green; `native-window-session` (1)/(2) were still running and are not merge gates. |
| #313 | Draft | Documentation cleanup; still needed: the hosted Graphify index lists only `BaryonOS/BaryonOS` | Merged `main` (`aed017fc`); diff to `main` is the 3 intended files; `progress.py --check` passes. |
| #316 | Draft | CI flake hardening for the 200 % window gate | Merged `main` (`8e7800b7`); `tools/test_window_session.py` 28 OK; CI workflow contract tests 54 OK. Its description still says `main` is red; `main` passed the window gate again on `c76daf7b`. |
| #315 | Draft | CI/security tooling; read-only token scopes, same-repo PRs only, aggregate code-scanning counts only | Merged `main` (`f09f6275`): combined `actionlint.yaml`, both ADRs kept, its handoff on top of PROJECT-STATE with triage status; config test updated to pin both entries; 18 + 54 tests OK. |
| #304 | Ready, behind, auto-merge armed | Feature (A02). Owner's local review of 13 Nov reports two Medium findings (silent one-generation rollback after an acknowledged commit; torn first `format` returns `Corrupt` with no safe rule) | Not touched: updating the branch would trigger auto-merge with open findings. |
| #302 | Ready, bot-authored, conflicts, auto-merge armed | CI blocker: `host-quality` fails in "Preserve Rust coverage artifact" ("No readable test evidence was produced"); `experimental-m3-m4-smp` failed on 11 Nov; conflicts in `actionlint.yaml` and `tests/test_ci_evidence_workflow_verifier.py`; overlaps #315 in `ci.yml` | Not touched (auto-merge armed). |

Merges into PR branches are non-fast-forward merges of `origin/main`; no rebase or force-push. Each `progress.json` conflict was resolved to main's criteria plus a fresh assessment for that PR (41/100 stays 41/100).

## Issues

- #283: disposition comment posted (issuecomment-6864877559). 25 of 33 PRs merged; #227, #229, #232, #238, #246 superseded by #296/#319; #245 rejected with rationale; #270 superseded by #269. Only #260 lacks a recorded decision; issue left open.
- #39, #41, #42 (feature criteria), #43 (execution route), #274 (English coverage, ongoing), #277 (wiki publication): roadmap or publication work, not maintenance; left unchanged.
- Main's only red run (`Copilot` reviewer, run 43501362868) is a GitHub-managed agent blocked by its firewall; not a repository workflow.

## Approval queue (project owner)

Status read at 22:30 (Europe/Amsterdam). All four updated PRs are drafts with
no review decision and 0 unresolved review threads; `host-quality` passed on
each head. Remaining pending checks were native QEMU gates (`native-smp`,
`native-window-session`, `native-system`) and CodeQL Rust on #315.

| Order | PR | Head | Risk | Recommendation |
|---|---|---|---|---|
| 1 | #320 security fix (3 files) | `77f59101` | Low; no `progress.json` change | Approve; mark ready when the native gates finish. |
| 2 | #313 docs (3 files) | `aed017fc` | Low | Approve. |
| 3 | #316 CI harness deadline | `8e7800b7` | Low; harness bound only | Approve after `native-window-session` passes on its head. |
| 4 | #315 CI/security tooling | `82a1747d` | Low; read-only scheduled workflow | Approve last: it edits the top of `PROJECT-STATE.md` and cites #320. |

The order is not a batch. #313, #316 and #315 each rewrite the same
`progress.json` assessment against `bf4aa716`, and the strict up-to-date rule
puts each remaining PR behind after a merge. After every merge, the next PR
needs a fresh merge of `main` (re-resolving its assessment) and one CI cycle
before it can merge. No branch is updated ahead of approval.

Decisions without a merge:

5. **#304**: decide the two Medium review findings. Recommendation: disable
   auto-merge until they are resolved; any branch update would merge it.
6. **#302**: decide whether coverage belongs in the required gate.
   Recommendation: disable auto-merge, then close or rework (coverage artifact
   step and conflicts).
7. **23 Rust code-scanning alerts**: approve the per-alert classification, then
   dismiss each with its reason and a link to this record.
8. **#260**: retire, or select specific changes for a new small PR.

## Next step

- On resume, first read the checks of #313, #316 and #315 (only #320 is
  watched by Auto-fix) and fix any failure on those branches.
- Fix any CI failure that Auto-fix reports on #320 (bound to this session).
- Low-priority finding, deferred: 154 links in 34 files still point to
  `github.com/simonvanieperen/BaryonOS` (GitHub redirects them after the
  transfer). `tests/test_ci_workflows.py` pins two of them as canonical
  milestone links that must match the GitHub milestone descriptions, so a
  migration also changes milestone text on GitHub. Historical records
  (`DECISIONS.md`, `PROJECT-STATE.md`, `docs/history/`) should keep their links.
  `tools/publication_audit.py` already uses `BaryonOS/BaryonOS` (id 1435549142,
  matches the live repository). Do it when no other session is editing
  `README.md`, `docs/ROADMAP.md` or the wiki pages.
- Act on approval-queue answers; recheck head commit, required checks, review
  state and mergeability immediately before any approved merge.

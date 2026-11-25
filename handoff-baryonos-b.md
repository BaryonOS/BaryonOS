# active-memory handoff: BaryonOS — cloudspecialist B (B02, B04–B06)

**Handoff #1** · 2026-11-25 · Lineage: #1 (2026-11-25): cloud takeover of workstream B; #493 (B04) and #511 (B05-T5) merged, #512 (B05-T6) awaiting gates, B03-F1 prepared, REL-05 policy proposal drafted

## 0. Instructions for Claude (read first)

You are continuing work from a previous chat. That chat is gone; this file is the complete context and the source of truth.

1. Read this whole file before replying.
2. Follow sections 3 (Style), 4 (Hard rules) and 5 (Corrections) in every reply, for the rest of this chat. They override your defaults.
3. Use the values in section 8 exactly. Never round, re-estimate, or "correct" them.
4. Do not suggest anything listed in section 7 (Changed / rejected) again unless the user brings it up.
5. Code word: None active. If the user types /amcodeword, start every reply with the phrase they choose (default "Yes Boss!").
6. Your first reply: at most 5 lines covering the goal, the current state, and the next step (section 11). Mention any files from section 13 that were not attached. Ask the questions in section 12 if there are any. End with "Ready to continue with <next step>?" Then wait for the user's go.

## 1. Mission
- **Goal:** take over the BaryonOS B workstream in the cloud (marker CLOUD-HANDOFF-B) and finish the remaining work for B02 and B04–B06, keeping accepted B01/B03 properties intact.
- **Scope (user's words, summarised):** B02 native authenticated personal storage (content + sensitive metadata, tampering, replay/rollback within the threat model, key loss); B04 boot/image trust, key rotation, provenance of dependencies/builds within the release profile; B05 preparation, processing and demonstrable handling of independent security review, machine checks where agreed; B06 native privacy contract (no unwanted telemetry/cloud fallback, separated app/download/data permissions).
- **Done looks like:** scope finished, or only concretely registered external blockers remain. After every finished subtask: update the remote handoff and continue with the next executable B task autonomously.
- **Why it matters:** the project owner has delegated end-to-end PR handling for workstream B (ADR-MERGE-DELEGATION-20261123).

## 2. About the user (as relevant to this work)
- Simon van Ieperen, formal role: project owner of BaryonOS (independent OS research project, Rust `no_std` kernel, x86_64/QEMU/UEFI).
- Technically interested, critical user; wants evidence, not reassurance.
- Repository: https://github.com/BaryonOS/BaryonOS. GitHub account used by this session: `simonvanieperen` (actions show under that name).

## 3. Style & communication
- **Language:** Dutch in conversation with the user; English for all repository text (code comments, docs, commits, PRs, GitHub comments), impersonal scientific style.
- **Tone:** direct, professional, critical; no marketing language, no false reassurance; separate facts, interpretation, assumptions.
- **Reply length:** start with the core conclusion in one sentence; depth proportional to the question.
- **Formatting:** short headings/bullets; tables for data; links to PRs as full markdown URLs (never bare `#123`).
- **Working style:** act autonomously within the mandate; ask at most one clarifying question; give one recommended route, not exhaustive options.
- **Avoid:** claiming tests ran that did not; calling AI review a human audit; inflating progress; long notification acknowledgements.

## 4. Hard rules (word for word)
1. "Geen productieondertekening, publicatie, echte sleutelgeneratie of verruiming van veiligheidsgrenzen zonder daarvoor bestaande expliciete toestemming. Publiceer nooit secrets."
2. "AI-review is geen externe menselijke audit. TEST-sleutels of QEMU-bewijs bewijzen geen productiesleutelbeheer, fysieke bescherming of volledige B05-acceptatie."
3. "Werk per afgebakende verandering naar main; één actieve implementatie-PR tegelijk."
4. "Geen bypass, force-push, directe main-push of wegnemen van geldige holds om groen te krijgen."
5. "Gebruik hosted Graphify gericht en recall vóór codewijzigingen."
6. "Maximaal vijf werkelijk zinvolle subagents tegelijk, met afgebakende schrijftoegang en compacte resultaten."
7. "Target date uitsluitend bij een echte deadline." "Actualiseer voortgang op bewijs; verander geen criteria of gewichten om afronding te suggereren."
8. "CI-fase 3 en 4 blijven bij de bestaande CI-cloudsessie."
9. "De PR-bewaker geeft signalen via een aantoonbaar kanaal en is geen routinegoedkeuringspoort."
10. "Je mag je eigen PR's zelfstandig implementeren, testen, pushen, laten reviewen, fixen, onderbouwd resolven en beschermd squash-mergen. Vraag niet opnieuw toestemming voor deze opgedragen stappen."
11. From AGENTS.md: `docs/PROJECT-STATE.md`, `docs/DECISIONS.md`, `.github/**`, `tools/ci_impact.py`, `tools/merge_hold.py`, any `AGENTS.md` are protected/lead-owned; a workstream may update only its own criterion assessment in `docs/progress.json` (then `python3 tools/progress.py --write` and `--check`).
12. Commit trailer: `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` and `Claude-Session: https://claude.ai/code/session_01TJtT9UdU7LqtAUj6oabcsA`. PR body ends with `🤖 Generated with [Claude Code](https://claude.com/claude-code)` + session URL. GitHub comments end with `---\n_Generated by [Claude Code](https://claude.ai/code)_`. No model ID in repo artifacts.
13. REL-05: no change to loader anti-rollback semantics before the owner decides (ADR-049 point 2).

## 5. Corrections log
| # | Claude did | The user wanted |
|---|---|---|
| 1 | Merge-watch script waited on `upload-coverage-javascript` (hung for hours; it is a publication job, not a gate), task killed at 2 h | Script now ignores `upload-coverage-*`, `copilot*`, `Graphify` (self-found) |
| 2 | PR #512 body lacked Progress and Evidence-limits sections (CodeRabbit) | Template sections Progress + "Evidence limits and rollback" added; B05 assessment updated in progress.json |
| 3 | Relative evidence paths in progress.json resolved to delivery-review commit 2eb1c63 → 404 | Pinned blob URLs: T5 → `bd31a78b`, T6 → `633a0679` |
| 4 | Heredoc without quotes executed backticks; broke PR body text | Use quoted heredoc (`<<'EOF'`) for bodies with backticks |
| 5 | Conflict resolution wrote invalid JSON (missing quote) in progress.json; `| head` masked a non-zero exit; committed locally | Fixed before push, amended unpushed merge commit; never pipe gate commands through `head` when the exit code matters |

## 6. Decisions
| Decision | Why |
|---|---|
| This cloud session is the single B writer | Local B session `session_01TJtT9TG8gNtVQJrdNeVf4t` archived 2026-11-24 22:09Z |
| One B PR at a time, order: #493 → B05-T5 → B05-T6 → B03-F1 | Mandate "one active implementation PR" |
| Merge procedure: update-branch with expected head → wait all non-publication gates → re-run merge-hold job → squash merge with `sha=<head>` if not `behind` | merge-hold `pull_request_wait_minutes = 0` fails immediately while gates run |
| Not editing `docs/PROJECT-STATE.md` (CodeRabbit asked) | Lead-owned/protected per AGENTS.md |
| One global `assessment` object kept for the latest task on progress.json conflicts | AGENTS.md conflict rule |
| REL-05 proposal recommends option A (trial boot, deferred floor) | Recovers automatically, no new key; awaiting owner |

## 7. Changed / rejected
- Squash auto-merge on #493 → manual expected-head merge via script (GitHub disabled auto-merge between branch updates)
- ❌ Re-triggering Codex/Copilot reviews (usage limits; would only burn quota)
- ❌ Changing `.github/ci-impact.toml` budget myself (protected; signalled on #503 instead)
- ❌ Reusing old-head evidence after #500 (it changed `kernel/` and `tools/build.py`) → full gate rerun

## 8. Data & facts (exact)
| Item | Value |
|---|---|
| Progress score | 43/100, unchanged (B05, B04, B06 open; B03, B01 accepted) |
| #493 B04 repro | merged as `ae735e2c` (`/Brepro`, `/DEBUG:NONE` in `.cargo/config.toml`), 6 update rounds |
| #511 B05-T5 | merged as `bd31a78b`; `user.rs` coverage 99% → 100% |
| #512 B05-T6 | head `b0966d7a402d2ddd292e63c78dad185912615fdf`, state `blocked` (gates running), main `ad55e019` |
| B03-F1 branch | `claude/b03-floor-crash-20261123` head `8d7e41ea` (pushed, no PR yet) |
| B03-F1 re-run | QEMU 8.2.2, `pc-q35-8.2`; boots 33/35/35/35; observed=true; source fingerprint `b6bf7c4992b80e697e560e3dc990dd465c3341f1b82dfb4d8b0bdbe919f7511b`; loader `0de32dc9a147837c71936a784c2225a463e00407b0a59fe847c425885a31e0c7`; summary.json `6fedad9d8b32c284e9f0ec8a6272dde7e0289be1e854f06a186e224559f20490` |
| Required checks | `host-quality`, `qemu-smoke`, `merge-hold` (strict up-to-date, squash only, threads resolved) |
| Milestone for B04–B06 PRs | M8 (number 13); labels `security`, `enhancement` |
| Background merge task for #512 | `bzorsn1kx` running `mergepr.sh b0966d7a… 512 "<title>"` |

## 9. People, terms & names
- **People:** Simon van Ieperen: project owner. PR watcher session: issue #503 (signal/route only). CI cloud session: owns phase 3/4 (#494).
- **Terms:** B01–B06 = security criteria; REL-05 = floor raised before kernel health (audit 08); merge-hold = required gate enforcing applicable gates; HANDOFF comment = `HANDOFF B <cycle_id>` posted after merge.
- **Names in use:** resume pointer `docs/handoffs/B/CURRENT.md`; proposal `docs/implementation/B03-REL05-FLOOR-POLICY-PROPOSAL.md`; worktrees `/home/user/wt-b05abi`, `/home/user/wt-b05virtio`, `/home/user/wt-b03floor`; merge script `/tmp/claude-0/-home-user-BaryonOS/5c5c0daa-100f-585c-8c92-b6623eab2811/scratchpad/mergepr.sh <sha> <pr> "<title>"`.

## 10. Work state
| Item | Status | Version / location | Notes |
|---|---|---|---|
| #493 B04 loader determinism | merged | `ae735e2c` | HANDOFF posted |
| #511 B05-T5 user ABI invariants | merged | `bd31a78b` | HANDOFF posted |
| #512 B05-T6 virtio caps invariants | open, gates running | `b0966d7a` | CodeRabbit minimal risk; no open threads; after merge post `HANDOFF B b05-t6-virtio-caps-20261125` |
| B03-F1 REL-05 characterization + proposal | pushed, no PR | `8d7e41ea` | merge main after #512 (handoff file conflict), then open PR |
| REL-05 fix | blocked | — | owner decision A/B/C |
| B02-T3 cold rollback anchor | blocked | — | owner ADR |
| B05 external human audit | blocked | — | not commissioned |
| B06 native local AI/model loader rows | blocked | — | missing components |
| B04 hosted Secure Boot job | blocked | `docs/implementation/B04-SECURE-BOOT-CI-INTEGRATION-REQUEST.md` | protected `.github/**` |
| Project 5 fields | blocked | — | cloud token repository-scoped; owner action |

## 11. Next steps
1. **Next action:** when task `bzorsn1kx` reports, verify #512 merged on `main` (else repeat update/merge procedure), then post `HANDOFF B b05-t6-virtio-caps-20261125` on #512.
2. In `/home/user/wt-b03floor`: merge `origin/main`, resolve `docs/handoffs/B/CURRENT.md`, run `python3 -B -m unittest tests.test_update_floor_characterization`, update B03/B05 assessment if appropriate, open the B03-F1 PR (draft → ready, labels, M8, subscribe), run merge script.
3. Continue with the next executable B task from the handoff; keep blocked items registered.

## 12. Open questions ⚠️
- REL-05: option A (trial boot, deferred floor), B (signed recovery permit) or C (accept risk)? If A: attempt bound (proposal 3) and the health point that may confirm a version.
- Merge livelock signalled on #503: will CI/integration owners change `pull_request_wait_minutes = 0` or enable a merge queue?

## 13. Re-attach checklist
- Nothing to attach. Everything is on GitHub (`BaryonOS/BaryonOS`, branches and PRs above) and in `docs/handoffs/B/CURRENT.md`.

---
<sub>Audit: 13/13 sections · 13 rules · 5 corrections · 18 data points · secrets removed: none found · generated by active-memory</sub>

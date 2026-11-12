# Copilot handoff — finish K06 step 5f evidence

Continue the existing authorized task until the complete evidence is green and committed. A partial matrix, a single failed/missing repetition, an aggregate without raw verification, or a skipped gate is not completion. Report briefly in Dutch; write repository documentation and GitHub communication in English.

## Repository and frozen source

- Repository: `BaryonOS/BaryonOS`, stable repository ID `1435549142`. It was transferred from `simonvanieperen/BaryonOS`; old URLs redirect.
- Existing work branch: `codex/k06-5f-complete-20261109`. Fetch and check out this exact remote branch. Do not create a replacement branch, reset history, merge, or change unrelated code.
- Frozen candidate tested by the current pending workflow: `54a9d5c17291b862e30240e49ed4cde0d479315b`.
- Source fingerprint version 3: `727676734c47e4efabedb49831763d71c1cd08c5698704de4e1c12a26e57232b`.
- Documentation checkpoint before this handoff: `b107e1c8b85485792d066cc3989888a681560570`. Subsequent documentation does not change the tested native source or its fingerprint. Read the latest remote branch, not an old checkout.

Read applicable `AGENTS.md`, `CODEX-WORKFLOW.md`, `PROMPT-VERVOLG.md`, the top of `docs/PROJECT-STATE.md`, relevant L0 specifications and the repository OS-development skill. Use hosted Graphify and RTK where available; Graphify describes indexed main, so compare against the actual branch. The L0 specs are on branch `docs/k06-step-specs`, under `docs/implementation/k06-steps/`, especially `05-F19-F15-icr-quarantine.md`, `06-5c-2-bsp-timer-idle.md`, `07-5e-process-shootdown.md`, and `09-5f-acceptance.md`.

## First action: inspect both existing runs

1. **Candidate full run [42818153989](https://github.com/BaryonOS/BaryonOS/actions/runs/42818153989)**, head `54a9d5c17291b862e30240e49ed4cde0d479315b`. Dispatched with `full_matrix=true`, `clock_diagnostic=false`, `full_evidence=false`. At handoff it is **pending, with zero jobs**, behind the existing branch concurrency group. This is the intended complete proof run. Do not dispatch a duplicate.
2. **Rejected older run [42809499884](https://github.com/BaryonOS/BaryonOS/actions/runs/42809499884)**, head `3dd6194f02231e8f1db61940242d8d90318daf1d`, source-v3 `05f680b768def10a2495e5958fa3af2fb95b50be19c755cce728f8c850ac887a`. It is still running (the API can say `queued` while matrix jobs are active). At handoff: 29 ordinary jobs report success, two ordinary jobs fail, all ten soak jobs succeed, and both IPC jobs are running. It cannot be accepted. Let it finish and preserve its artifacts; no repetition is reused.

The unchanged `native-smp-${github.ref}` concurrency group prevents these full matrices from overlapping. Do not cancel either run and do not run a competing local/cloud QEMU matrix. Refresh the live state; the counts above are a checkpoint, not a prediction.

```bash
git fetch --no-tags https://github.com/BaryonOS/BaryonOS.git \
  refs/heads/codex/k06-5f-complete-20261109
git checkout codex/k06-5f-complete-20261109
git status --short --branch
gh run list --repo BaryonOS/BaryonOS --workflow native-smp.yml \
  --branch codex/k06-5f-complete-20261109 --limit 5
gh api 'repos/BaryonOS/BaryonOS/actions/runs/42818153989/jobs?per_page=100'
```

Fetch/read, push dry-run, actual checkpoint push and Actions dispatch all succeeded after the ChatGPT Codex Connector was installed for organization BaryonOS, installation `180498948`, selected repository only. A new environment must verify its own credentials. On a genuine access blocker, report the exact command and error; do not infer write access from a successful public read.

## Implemented corrections and evidence limits

Existing source `3dd6194f` already contains K06 migration/acceptance integration, exact fail-closed matrix aggregation and gate orchestration, FPU readiness repairs, bounded observer work, syscall quantum continuation, proven capless IPC snapshot elision, mode20 PAUSE, and IRQ-entry TSC measurement. Do not redo these changes blindly. The capless success proof is serialized by KERNEL/ASPACE; possibly failing/cap-bearing operations retain full snapshots, exact rollback comparisons and wipes. Native IPC recovery is still unproved until both current jobs and the candidate matrix complete.

The rejected old run has three independently verified failures:

- `proc-shootdown-deaf`: 48/50; iterations 18 and 21 report `SMP:QUARANTINE:LOCK_HELD:40:CPU=1`, then `M1:FAIL:SMP_QUARANTINE_LOCK_HELD`. Clock passes. This is the correct IQ-3 stop, exposing a violated test precondition: L0 step 07 requires the deliberately deaf AP to hold no lock.
- `bsp-spin-irq-off-control`: 49/50; iteration 45 fails clock drift before the control phase. Five samples each count exactly 1000 IRQ0 entries; normalized microseconds are `[1013723,1014537,1023535,1027186,1014857]`, median `1014857`. Later observer lags are about 2 microseconds and do not explain the sustained drift.

Candidate `54a9d5c` adds only scoped fixture/oracle repairs:

- A real, normally acknowledged kernel setup round closes test-only scheduler admission before masking AP timers. Pending timer/reschedule entries resume the same context without RUNQ. All AP setup acks, lock-free ownership, fresh AP1 reader progress and the exact ACTIVE bit are required before the A unmap. The actual A-round still performs no flush/ack. IQ-3, before/after-INIT checks, frame retention and the 100-ms deadline remain unchanged. The oracle additionally requires `SMP:PROC_DEAF:LOCK_FREE:CPUS=4`.
- Every continuously busy clock-load AP performs 64 dependent integer steps before its separate progress increment/store and PAUSE. QEMU 10.2.1 exits the TCG loop on each PAUSE and reacquires BQL. This changes fixture instruction mix/exit frequency; historical clock causality and native effectiveness remain unproved. Real IRQ0 counting, five segments, median, exact 1% and deadlines are unchanged.

Four matching crossbuilds, 275 host tests without skips, 48 focused SMP oracles, formatting, diff checks and actual ELF instructions are retained in `fixture-repair-host-checks.json` and linked files in this directory. Independent AI source review found no remaining blocker after a compile issue was corrected. No local native guest ran. None of this substitutes for the new full native matrix.

For the old source, the retained partial raw readback verifies 1547/1550 attempts, including all 50 complete soaks. All 50 soaks independently replay successfully with guest durations 600000–600007 ms. Two failure ZIPs and three raw failing repetitions are committed as compact evidence. At handoff 42 original ZIPs are also verified locally; those ignored Cloud files will not automatically exist in Copilot's workspace. Download originals from GitHub as needed. Never delete original GitHub artifacts.

## Acceptance: every condition is mandatory

- Exactly 63 ordinary cases, each 50/50, plus the combined soak case 50/50 across ten shards of five: **64 cases, 3200 unique guest runs** on one frozen commit/source hash.
- Regular `m4-quotas-four` and `qsim-four` both 50/50. The separate environment probe does not replace them.
- All 50 complete combined ten-minute soaks, including their quota/qsim/IPC/FPU/fairness/cleanup proofs.
- Aggregate artifact: `passed=true`, every case exactly 50/50, total 3200, one source hash/head; aggregate job and host oracles successful.
- Subsequent `SMP final repository gate` actually executes `tools/gate.sh`, succeeds and retains `smp-section9-final-gate` with `gate.log` and nested regression evidence. A configured, skipped or missing gate is not a pass.

Download full raw REST run/jobs/artifact metadata (paginate; wrappers can truncate pages or omit timestamps), preserve original ZIPs, verify API SHA-256/size, ZIP CRC, safe canonical extraction and per-run head binding. The direct Cloud `gh api .../artifacts/ID/zip` download previously reached an Azure `403`; the GitHub connector's artifact-download operation worked. This is separate from working Actions-dispatch permission. Use an available authorized download route, then verify the same original bytes.

The two existing read-only verifiers are now tracked in this directory. They never start guests:

```bash
R=build/k06-handoff/run-42818153989
C=54a9d5c17291b862e30240e49ed4cde0d479315b
S=727676734c47e4efabedb49831763d71c1cd08c5698704de4e1c12a26e57232b
python3 -B docs/verification/k06-smp-sched/verify_artifacts.py \
  --repository "$PWD" --input-dir "$R/artifacts" \
  --aggregate "$R/artifacts/smp-section9-matrix-report/matrix-report.json" \
  --commit "$C" --source-sha256 "$S"
python3 -B docs/verification/k06-smp-sched/verify_final_gate.py \
  "$R/artifacts/smp-section9-final-gate" --repo "$PWD" \
  --expected-commit "$C" --expected-source "$S" \
  --run-json "$R/latest-run.json" --jobs-json "$R/latest-jobs.json"
```

Use one extraction subdirectory per artifact, preserving its original name. Both verifiers must exit 0. Also require `environment_probe_checked=true` explicitly; the matrix verifier does not separately make a missing probe fatal. Check original ZIP digests/run binding and aggregate `host-oracles.log` separately. Keep the exact tested oracle files byte-identical; use an isolated frozen oracle checkout when replaying older evidence after source changes. Partial diagnostic mode intentionally rejects acceptance. Compact artifacts omit executable/firmware bytes: recorded hashes are cross-bound to manifests, not independently rehashed from absent binaries. No physical hardware or human audit is proved.

## If a candidate case is red

Inspect the exact original case ZIP and job log, identify the actual product/fixture/runner/measurement cause and fix only that cause under L0. Never reduce repetitions, quotas, soak duration, workload requirements, timeouts or safety criteria to obtain green. Run appropriate light host/fmt/diff checks, commit/push to this same branch, then dispatch the entire `full_matrix=true` workflow from zero on the new source. A rerun of failed jobs cannot replace the required proof. Preserve all rejected evidence and use the existing concurrency lock.

## Finish and commit the result

Only after matrix, K07, soaks, aggregate and actual gate all verify green, update at least `docs/PROJECT-STATE.md`, `docs/PROGRESS.md` and this verification directory with run URL, exact tested commit/source hash, QEMU/runner profile, all counts, gate outcome and explicit proof limits. Reassess K06 through the existing fixed progress rubric; current score is 41/100 and K06 remains open. Do not invent acceptance or change criterion weights. Read back the edits, run appropriate docs/host checks, `python3 -B tools/progress.py --write`, `--check`, evidence checksums and `git diff --check`; commit/push the documentation. Documentation after the tested commit need not trigger a new native source run, but must identify the exact tested commit. Keep all original artifacts.

# K06B-ORACLE: host oracle, host tests, case recipe, host time limits and workflow wiring

Task ID: K06B-ORACLE. Read-only. Source: frozen worktree `snap-67e592c` (commit `67e592c`;
kernel and tool sources equal `2543861`). All `path:line` citations are relative to that
worktree root; `specs/...` is `scratchpad/specs/docs/implementation/...`.
Date of analysis: 12 November 2026.

Labels: **[S]** verified in source, **[D]** verified in data, **[I]** inferred, **[U]** unverified.

## 0. Method and bounds

| Item | Statement |
|---|---|
| Fully read | `tools/run_smp.py`, `tools/run.py`, `tools/aggregate_smp_ci.py`, `tools/run_smp_matrix.py`, `tools/tests/test_smp_serial.py`, `tools/tests/test_smp_acceptance.py`, `tools/tests/test_smp_ci.py`, `.github/workflows/native-smp.yml`, `.github/actions/{setup-qemu,setup-python,preserve-evidence}/action.yml`, `docs/verification/k06-smp-sched/verify_artifacts.py`, `timing-investigation/{timing,analyse,prog}.py`, `map/K06A-SPEC.md`, ADR-082 (`docs/DECISIONS.md:2680-2741`) |
| Read in part | `tools/common.py`, `tools/build.py` (1-140, 340-372, 420-426), `tools/ci_evidence.py` (limits, `collect`), `tests/test_ci_workflows.py` (all test bodies except milestone/dependabot/actionlint details), `verify_final_gate.py` (name pins, CLI), `tools/run_supervisor.py` (1-120), `kernel/src/serial.rs`, `kernel/src/m2.rs` (8-20, 108-175), `kernel/src/main.rs` (9-76), `kernel/src/m2/smp.rs` (fixture sites of the two newest cases, 1318-1403, 3080-3090), trigger blocks of all 28 workflow files |
| Git history | `git show 2543861` (stat and tool/workflow/verifier/test hunks) was read through the frozen worktree's git metadata to obtain the worked example in section 3. No working-tree file of `/home/user/BaryonOS` was read. |
| Not done | No cargo, build, QEMU or test run. Hosted Graphify MCP not used (it indexes `main`); every fact comes from the worktree. Two static Python snippets (AST count of `CASES`, JSON size estimate from one archived report) were run with `python3 -I`. |
| Not opened | `tools/test_milestones.py`, `tools/test_resource_pressure.py`, `tools/run_rtc.py` bodies; kernel waits other than the sites cited; `K06A-SITES.tsv` beyond its header and id column (all 92 rows are kernel sites, none host-side **[S]**). |

## 1. (a) Serial evaluation in `tools/run_smp.py`

### 1.1 Pipeline of `evaluate(case, stdout, raw_exit, timed_out)` (`tools/run_smp.py:456-593`) [S]

| # | Step | Code | Effect on a line |
|---|---|---|---|
| 1 | `unlock_terminal` | `:447-453`, call `:461` | Only for outcomes in `TERMINAL`: a leading `!` is stripped from the **last** raw line. |
| 2 | Any other `!` line | `:462-463` | `ValueError`. |
| 3 | `is_success(raw_exit, stdout, timed_out, required_markers=BOOT + end, forbidden_markers=FORBIDDEN)` | `:464-466`; `tools/run.py:33-45` | Required markers must occur as whole lines, in order (subsequence); exit must be 33; no timeout; no forbidden substring anywhere in `stdout`. |
| 4 | ACPI inventory | `:467-469` | All lines starting `ACPI:` must equal `["ACPI:UNAVAILABLE:NO_DMAR"]`. |
| 5 | `without_acpi_diagnostics` | `:470`; `tools/common.py:24-43` | **Removes** exactly the pair `ACPI:UNAVAILABLE:NO_DMAR` + `DEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT` between `KS-02:PASS` and `KS-03:PASS`. |
| 6 | EBS anchor | `:471-476` | Exactly one `BOOT:EBS_OK`; no line before it may contain `BOOT:`, `BOOTINFO:`, `KERNEL:`, `KS-`, `M<n>:` or `SMP:`. |
| 7 | Boot prefix | `:477-479` | The first six lines from `BOOT:EBS_OK` must equal `BOOT` (`:15`). |
| 8 | `expected_smp(case)` | `:480`, `:199-363` | Expected list; wildcard tokens are literal strings containing `*`. |
| 9 | `serial_lines(observed[6:], control)` | `:482`, `:148-176` | **Removes** every `SMP:SER:<cpu>:<seq>` line (`SERIAL_LINE`, `:144`) and records `seen[cpu]`. A line containing `SMP:SER:` that is not a full match raises (torn). Phase: `before` → `open` at the first `SMP:ONLINE:` line → `closed` at `SMP:SERIAL:PASS`. In control mode (`outcome == "serial-torn"`) every `open`-phase line except `SMP:SERIAL:PASS` and lines starting `SMP:TIMER:RATIO:` is **dropped** and counts as torn when it is not a whole `SMP:SER` line (`:158-160`); no torn line raises (`:174-175`). |
| 10 | Fixture completeness | `:483-487` | `seen` must be `{cpu: [0..199]}` for the CPU count of the first `SMP:ONLINE:n/m` line (skipped in control mode). |
| 11 | Per-line normalisation loop | `:491-574` | A line whose value passes validation is **replaced in place** by its wildcard token; otherwise it stays raw. Nothing is removed here. |
| 12 | Exact comparison | `:575-582` | terminal: `tail == expected`; M3: `tail == expected + ["SMP:LOCKDEP:PASS"] + m3_end(...)`; else `tail == expected + ["SMP:LOCKDEP:PASS", "M2:PASS", "M1:PASS"]`. Mismatch → `ValueError`. |
| 13 | Qsim PIDs | `:585-589` | Regex over raw `stdout`. |
| 14 | Return | `:590-593` | `{"outcome", "profile", "online", "deadline_margins", "ap_ids"}`. |

### 1.2 Removed versus normalised lines [S]

| Line family | Treatment | Code | Position-bound in the exact comparison |
|---|---|---|---|
| `ACPI:` / `DEVICE-EXECUTION:` pair | removed | `tools/common.py:24-43` | n/a (fixed place enforced) |
| `SMP:SER:c:n` | removed, counted per CPU | `tools/run_smp.py:161-168` | no (phase-bound only) |
| control-phase lines (`serial-unlocked-control`) | removed | `:158-160` | no |
| `SMP:TIMER:RATIO:<r>` | normalised to `SMP:TIMER:RATIO:*` if `r` in 50 000..75 000 | `:497-499`, `:123-124` | **yes** |
| `SMP:IDLE:WAKE_US=<n>` | normalised to `SMP:IDLE:WAKE_US=*` if `n < 100000`; margin recorded | `:500-507`, `:126-127` | **yes** |
| `SMP:FAIR:CPU=…` | normalised if cpu, limits and values valid; two margins recorded | `:550-564`, `:371-374` | **yes** |
| `SMP:GS:CR4=<n>` | normalised if bit 16 clear | `:492-494` | yes |
| `SMP:ICR:MISROUTED` / `LOST` | normalised to `SMP:ICR:*` | `:495-496` | yes |
| `SMP:M3:CPU=…` | normalised if ticks > 0 and switches ≥ 1000 (≥ 1 for quotas/qsim) | `:508-510` | yes |
| `SMP:LOAD:…`, recovery, resumed, FPU-recovery, kill ack (margin), kill cleanup, `QSIM:<F>:<20 digits>`, `SMP:IPC:REMOTE_WAKEUPS`, `RESCHEDULE_IPIS`, `SMP:SCRATCH:CROSS:CPU=` | normalised when valid | `:511-573` | yes |

Consequence: the existing pattern for a measured line is "validate, replace by a wildcard token,
keep its place". An invalid value is not rejected explicitly; it stays raw and fails step 12.
`SMP:FAIR`, `SMP:TIMER:RATIO` and `SMP:IDLE:WAKE_US` are **not** removed before the comparison.
The only existing "parse and remove at any position inside a phase" pattern is `serial_lines`.

### 1.3 `FORBIDDEN` [S]

| Fact | Citation |
|---|---|
| `FORBIDDEN = ("FAIL", "PANIC")` | `tools/run_smp.py:17` |
| Applied as a **substring test over the entire raw stdout** (`any(marker in stdout ...)`), case-sensitive | `tools/run.py:44-45` |
| Call sites: guest harness verdict (`run_guest(..., forbidden_markers=FORBIDDEN)` → `result.json["passed"]`) | `tools/run_smp.py:651-653`; `tools/run.py:155` |
| Oracle | `tools/run_smp.py:464-466` |
| DG-2 recheck after `!` stripping | `tools/run_smp.py:681-687` |
| Stored contract checked by the verifier | `docs/verification/k06-smp-sched/verify_artifacts.py:208-211` |

A margin name containing `FAIL` or `PANIC` rejects the boot at step 3 with the generic message
"Native SMP boot/exit assertions failed" **[S]**.

### 1.4 `deadline_margins` [S]

Built at `tools/run_smp.py:489`; returned inside `observed` (`:590-591`); stored per boot at
`item["observed"]` (`:677`). Nothing aggregates it: `tools/aggregate_smp_ci.py:19-78` and
`verify_artifacts.py` do not read margin values (the verifier only requires
`evaluate(...) == item["observed"]`, `verify_artifacts.py:246-247`).

| `name` | Keys | Source line | Accept rule |
|---|---|---|---|
| `bsp_idle_wake` | `name, clock="guest", measured_us, deadline_us=100000, measured_over_deadline, remaining_us` | `:503-506` | `measured < 100000` (`:501`) |
| `remote_kill_ack` | `name, clock, measured_ms, deadline_ms=100, measured_over_deadline, remaining_ms` | `:528-529` | outcome `m3-kill`, vector in {128, 0xd0, 0xe8}, `measured < 100` (`:525`) |
| `kernel_lock_wait` | `name, clock, cpu, measured_ms, deadline_ms, measured_over_deadline, remaining_ms` | `:556-558` | `wait_ms <= 400`, limits equal 400/40 (`:553-554`) |
| `kernel_lock_deferred` | `name, clock, cpu, measured_ticks, deadline_ticks, measured_over_deadline, remaining_ticks` | `:559-562` | `deferred <= 40` |

`host_watchdog` is separate (`item["host_watchdog"]`, `:673-675`): `measured_s, deadline_s,
measured_over_deadline, remaining_s`.

Host tests that pin these shapes: `tools/tests/test_smp_serial.py:283-286` (eight entries with
`cpu`, ratio 1 at the boundary) and `:777-779` (every margin of `report["cases"][0]` must have
`clock == "guest"` and `deadline_us == 100000`; vacuous today because case `one` has no wake line).

### 1.5 What is written per boot [S]

| File | Writer | Content |
|---|---|---|
| `<NOS_BUILD_DIR>/run/<label>-<10 hex>/result.json` | `tools/run.py:155-162` | `label, started_utc, duration_s, timeout_s, timed_out, raw_qemu_exit, expected_exit, required_markers, forbidden_markers, passed, command, qemu_version, profile, base_commit, source_fingerprint_version, source_sha256, artifact_sha256, firmware_source, run_directory, stdout, stderr, expected_artifact_sha256, artifact_binding_passed` |
| `.../serial.txt`, `.../stderr.txt` | `tools/run.py:143-144` | raw bytes |
| `<NOS_BUILD_DIR>/smp-report.json` | `tools/run_smp.py:601-627`, rewritten after every boot (`:649`, `:701`, `:706`) | top: `schema_version=1, passed, source_sha256, source_fingerprint_version=3, manifest_sha256, kernel_feature, scope, repeat, repetitions{case:{requested, attempted, passed, failed, complete, flaky, evidence_kind}}, cases[], error?`; per boot: `case, iteration, cpus, tcg_threads, passed, guest{result.json without stdout + stdout_sha256} (:660-661), acpi_table_sha256?, host_watchdog, observed, error?` |

Label: `smp-<case>-rNNN-of-MMM` when `repeat > 1`, else `smp-<case>` (`:644`).

A boot that fails `evaluate` has `error` and **no `observed`** (`:676-691`): its margins are not
in the report **[S]**. Confirmed in data: the archived
`docs/verification/k06-smp-sched/m3-eight-single-original-full-report.json.gz` holds 35 boots
with `error` and without `observed` **[D]**.

### 1.6 Minimal change pattern for `SMP:MARGIN:<NAME>:<measured>/<limit>` at arbitrary positions

Recipe (not applied; names and limits in the table are placeholders) **[I: design; every
referenced line is [S]]**:

```python
# tools/run_smp.py, next to SERIAL_LINE (:144)
# AC-3 (spec 09): each named guest deadline logs measured/limit once per boot.
MARGIN_LINE = re.compile(r"SMP:MARGIN:([A-Z][A-Z0-9_]{0,31}):(0|[1-9][0-9]{0,19})/([1-9][0-9]{0,19})")
MARGIN_LIMITS = {"SYNC_MS": 2_000, "SCRATCH_MS": 500}   # name -> frozen limit; unit is in the name


def margin_lines(lines: list[str]) -> tuple[list[str], list[dict]]:
    """Split SMP:MARGIN lines (any position) from the rest; a malformed, unknown,
    repeated, re-limited or exceeded one fails the run."""
    rest, margins, names = [], [], set()
    for line in lines:
        if "SMP:MARGIN:" not in line:          # substring, as for SMP:SER: (:161)
            rest.append(line)
            continue
        match = MARGIN_LINE.fullmatch(line)
        if match is None:
            raise ValueError("Malformed SMP margin line")
        name, measured, limit = match[1], int(match[2]), int(match[3])
        if name in names:
            raise ValueError("Duplicate SMP margin line")
        if MARGIN_LIMITS.get(name) != limit or measured > limit:
            raise ValueError("Unknown SMP margin, changed limit or exceeded deadline")
        names.add(name)
        margins.append({"name": name.lower(), "clock": "guest", "measured": measured,
                        "deadline": limit, "measured_over_deadline": measured / limit})
    return rest, margins
```

In `evaluate`, replace `:482` and `:489` only:

```python
    unmargined, margins = margin_lines(observed[len(BOOT):])
    if terminal and unmargined and observed[-1] != unmargined[-1]:
        raise ValueError("Output after the terminal marker")
    body, seen = serial_lines(unmargined, control)
    ...
    deadline_margins = margins
```

| Point | Reason | Basis |
|---|---|---|
| Strip **before** `serial_lines` | In `serial-unlocked-control` an `open`-phase margin line would otherwise be dropped and counted as the required torn line (`:158-160`), so the control could pass without a real tear. The sync wait ends before `SMP:SERIAL:PASS` (`kernel/src/m2/smp.rs:1352-1357`, `:1379`), so a sync margin naturally falls in that phase. | [S] code, [I] consequence |
| Explicit `raise` | Removed lines never reach the exact comparison; the existing "leave raw and let step 12 fail" pattern does not protect them. | [S] `:574-582` |
| Substring test, `fullmatch` | A glued or torn line is rejected, as for `SMP:SER:` (`:161-164`). | [S] |
| Canonical digits, limit ≥ 1 | Avoids division by zero and leading-zero aliases. `[0-9]{1,20}` admits values above 2^64−1; existing code bounds them with `<= (1 << 64) - 1` (`:515`, `:533`, `:538`, `:547`, `:566`, `:569`) and tests use `18446744073709551616` (`tools/tests/test_smp_serial.py:261-264`). Add the same bound. | [S] |
| Host table with frozen limits | Precedent: the oracle pins `LIMIT_MS=400`, `LIMIT_TICKS=40`, `LIMIT_MS=100` (`:553-554`, `:524`) and tests reject a changed limit (`test_smp_serial.py:270-271`, `test_smp_acceptance.py:168,218`). The line format carries no unit. | [S] |
| `<` or `<=` per site | `within_ms(limit, ready)` succeeds after at most `limit − 1` elapsed polls (`kernel/src/m2/smp.rs:1396-1403`); existing rules differ (`<` wake and kill, `<=` fairness). | [S] code, [I] choice |
| One line per name per boot | A deadline executed several times per boot (for example per IPI round) must be reduced to one line by the kernel (maximum), or the duplicate rule rejects the boot. | [I] |
| Trailing-line guard for terminal outcomes | Without it a margin line after an un-banged terminal marker is hidden by the strip. | [I] |
| Nothing else changes | Steps 3–7 and 12 never see a margin line; `expected_smp`, `m3_end` and every `settled()[:-k]` slice (`:249-315`) stay as they are. A position-bound design would shift every slice. | [S] |

Optional second tier (ADR-082 "margin-oracle: line missing … host oracle rejects",
`docs/DECISIONS.md:2735`): a required-name set per case. It can be derived from the expected
sequence instead of a per-case table, for example `REQUIRED_AFTER = {"SMP:SCRATCH:PASS": {"SCRATCH_MS"}}`
and "every name whose phase marker is in `expected` must be present, no other name may be"
**[I]**. This tier forces updates to the hand-written fixtures (`started`, `online`, `bsp`,
`m3`, `failed` and the acceptance builders) and to `SmpRepetitionHarness.guest`
(`test_smp_serial.py:755-759`), which replays `online(cpus)` for every case.

### 1.7 Hazards for the margin lines

| # | Hazard | Basis |
|---|---|---|
| H1 | Name must not contain `FAIL` or `PANIC` (section 1.3). Also avoid `TIMEOUT`, `DRIFT`, `STUCK`: `map/hosted-margins.py:42` and `timing-investigation/analyse.py:21` classify lines by those substrings. | [S] |
| H2 | In `test-smp-serial-unlocked` **every** line of the whole boot is written without `SERIAL` (`kernel/src/serial.rs:122-126`). A margin line written by one CPU while another writes can tear outside the fixture phase and fail the exact sequence. Emit margin lines from the BSP only, at points where no AP writes. | [S] code, [I] consequence |
| H3 | Line length is capped at 160 bytes; overlong input is cut and flushed (`kernel/src/serial.rs:54`, `:152-157`). | [S] |
| H4 | A terminal marker written without the lock gets `!` (`serial.rs:131-140`); it must be the last raw line (`run_smp.py:447-453`). More BSP output raises the chance that an AP's terminal line meets a held `SERIAL`. | [S] code, [I] exposure |
| H5 | Shard report size. `smp-report.json` is a required evidence path (`.github/workflows/native-smp.yml:435-437`); a file above 1 MiB is omitted (`tools/ci_evidence.py:28`, `:408-409`) and an omitted required path fails the evidence step with exit 2 (`:507-511`, `:534-536`). Today: 203 843 bytes for 50 boots, about 4.1 KB per boot (re-serialised from the archived report) **[D]**. Estimate with 42 margin entries per boot on an 8-CPU load-like case (16 fairness entries): 894 743 bytes with six keys per entry, 768 743 with four keys, 497 843 as plain strings (limit 1 048 576) **[D on synthetic entries]**. With 42 four-key entries the report passes 1 MiB between 75 and 100 boots (837 693 / 1 116 643). The number of names per boot is not known **[U]**. | as marked |
| H6 | Margins of failed boots are not recorded (section 1.5). A diagnostic that needs the worst case must parse `serial.txt`, or `evaluate` must expose margins before raising. | [S] |
| H7 | `verify_artifacts.py:246-247` recomputes `evaluate` and requires equality with the stored `observed`; entries must be deterministic and JSON-stable (ints and `a / b` floats, as today). | [S], JSON float round trip [I] |
| H8 | `timing.py:63-65` records every `SMP:` line as a mark and `analyse.py:25` uses the delta to the **previous** mark as the phase duration (for example `SMP:KILL:PASS` → `SMP:SCRATCH:PASS`). A margin line printed before a phase marker makes that delta near zero. | [S] code, [I] consequence |
| H9 | `tools/run_supervisor.py` evaluates `native-smp-supervisor` and five `test-smp-supervisor-*` profiles (`:23-24`), which run `smp::start`. Its oracle keeps only named lines (`:74-80`, `:98`) and rejects `PANIC`/`FAIL` substrings per line (`:100`), so extra `SMP:` lines pass. These six cases run only in the final gate (`tools/qemu-smoke-test.sh:59-60`, `.github/workflows/native-smp.yml:591-595`); no pull-request job runs `test_supervisor.py --smp` (`process-management.yml:50` runs it without `--smp`). | [S] |
| H10 | Serial output inside a timed window consumes that window (K06A-SPEC section 5, item "Adding diagnostic serial lines"). | [I] |

### 1.8 Profiles whose serial output must stay unchanged, and enforcement

Normative text **[S]**: `specs/K06-SMP.md:40` ("Andere profielen (`milestone-m2`, M3–M6, GUI)
blijven één CPU en ongewijzigd"), `:43-44`; `specs/k06-steps/01-5b-2-kernel-lock-scratch.md:34-35`
(K-3: "De volgorde van markers, quota en geheugengebruik van alle bestaande profielen blijft
byte-gelijk"), `:74-76` (mandatory native one-CPU regression: `milestone-m2`, K05
`run_supervisor.py` 6/6, K07 `resource-pressure.yml`, `native-rtc` 5/5, `run_ipc_wait.py`,
`run_fpu.py`, `run_memory.py`; "Uitvoer en markers ongewijzigd"); `specs/k06-steps/README.md:46`;
`08-…:70-71`; `09-…:48-49` (adds "alle bestaande `run_smp.py`-gevallen").

| Group | Members | Why a margin line cannot reach it | Enforcement |
|---|---|---|---|
| Non-SMP kernel features | every `KERNEL_FEATURES` entry (`tools/build.py:50-70`) that does not enable `native-smp`: default M1, `test-panic`, `test-hang`, `milestone-m2`, `test-m2-*`, `native-rtc`, `native-acpi-s5`, `milestone-m3`…`m6c` and tests, supervisor, keyring, volume, update, system, memory, FPU, qsim, SDK, display and window profiles | `mod smp` exists only under `#[cfg(feature = "native-smp")]` (`kernel/src/m2.rs:14-15`, calls `:168-171`); the SMP line writer only under `native-smp` without legacy (`kernel/src/serial.rs:97`, `:166-172`) | Compile-time gating; `compile_error!` on forbidden feature mixes (`kernel/src/main.rs:49-76`); `tools/tests/test_gs_discipline.py:233` pins the `mod smp` gate; each runner's own oracle on every pull request (section 6) |
| Legacy SMP | `LEGACY_SMP_FEATURES` (`tools/build.py:48-49`) | separate module tree `kernel/src/m2_legacy` (`kernel/src/main.rs:9-13`); raw serial path (`serial.rs:170-171`) | `tools/test_smp.py` via job `experimental-m3-m4-smp` (`native-smp.yml:870-911`) |
| SMP family, other oracle | `native-smp-supervisor`, `test-smp-supervisor-*` (`kernel/Cargo.toml:139-144`) | **does** run `smp::start`; extra `SMP:` lines are filtered | `run_supervisor.py:74-113`; final gate only (H9) |
| One-CPU `run_smp.py` cases | `one`, `m3-one`, `m3-one-multi`, `m3-load-one`, `m3-load-one-multi` (5 cases, AST count) and the one-CPU continuations `ap-stall`, `ap-mismatch` | governed by the exact oracle; unchanged output is required only if a wait that logs a margin is skipped on one CPU | `tools/run_smp.py:575-582`; counted as `single_cpu_regression` (`:624`) |

`tools/run.py:53` ("Defaults keep every existing profile byte-identical (-smp 1, single TCG)")
refers to the QEMU command defaults of `run_guest` (`:51`), not to a serial comparison **[S]**.
`is_success` checks an ordered subsequence, the exit code and forbidden substrings; for an
M1-only expectation it also rejects lines starting `M2:`…`QSIM:` (`tools/run.py:21`, `:33-45`);
`SMP:` is not in that list. No tool compares a transcript byte for byte against a stored
reference (grep of `tools/`, `tests/` for `golden`, `byte-identical`, `7ffdacfa`: only the
comment at `run.py:53`) **[S]**. Byte identity of non-SMP profiles therefore rests on feature
gating: margin output placed in shared files (`kernel/src/m2/m3.rs`, `lock.rs`, `serial.rs`)
must be under a `native-smp` cfg **[I]**.

## 2. (b) Host tests

### 2.1 Fixture construction [S]

| Element | Location | Role |
|---|---|---|
| `PREFIX` | `tools/tests/test_smp_serial.py:14-16` | firmware line, boot markers, ACPI pair, `KS-03:PASS` |
| `fixture(cpus)` | `:20-22` | 200 `SMP:SER` lines per CPU, interleaved |
| `transcript(lines, lockdep=True, serial=True)` | `:25-35` | inserts `SMP:LOCKDEP:PASS` before `["M2:PASS", "M1:PASS"]` when the list ends with them; inserts the fixture before `SMP:SERIAL:PASS`; returns `PREFIX + lines` |
| Hand-written sequences, "deliberately independent of run_smp.expected_smp" (`:38`) | `started` `:39-46`, `RATIO` `:50`, `bsp` `:53-59`, `wake_kill` `:62-65`, `icr` `:68-70`, `proc` `:73-76`, `online` `:79-89`, `m3` `:92-109`, `failed` `:112-119`, constants `:122-177` | one list per outcome |
| `VALID` | `:179-204` | case → passing line list (51 keys) |
| Acceptance builders | `tools/tests/test_smp_acceptance.py:31-113` (`prelude`, `fairness`, `cpu_work`, `load`, `quotas`, `qsim`, `unmap`, `remote_kill`), `ACCEPTANCE_CASES` `:25-28` | 15 remaining cases; use `transcript(lines, lockdep=False)` (`:117-118`) |
| Coverage guard | `test_smp_serial.py:317-318` | `set(VALID) | ACCEPTANCE_CASES == set(CASES)` |

Passing transcript: `transcript(VALID[case])` (`:316-328`) or
`transcript(<builder>(...), lockdep=False)` for acceptance cases. Imports from the oracle are
limited to `CASES, MAX_CPUS, SERIAL_LINES, duplicate_madt, evaluate` (`:11`).

### 2.2 Precedents for a new line type and a new negative case [S]

| Subject | Positive | Negative mutations | Margin assertion |
|---|---|---|---|
| `SMP:IDLE:WAKE_US` | in `bsp()` `:58` | `:581-584`, `:587-588` (missing, `=100000`, empty, present on one CPU) | `:777-779` |
| `SMP:FAIR` | in `m3()` `:102-103` | `:265-273` (missing, duplicate, wrong CPU, over limit, changed limit, wrong phase) | `:280-286` |
| `SMP:TIMER:RATIO` | `:50` | `:572-575`; band edges accepted `:598-600` | — |
| Remote kill ack | `test_smp_acceptance.py:105-113` | `:160-179` (deadline equality, overflow, widened limit) | — |
| Case `quarantine-sched-busy` | constant `:159-161`, `VALID` `:191`, `online == 1` `:472` | `:462-467` (foreign marker in `resched-deaf`, missing marker, wrong CPU, pre-fix verdict with `M1:FAIL`) | — |
| Case `quarantine-lock-pending` | `VALID` `:191` reuses `LOCK_HELD` (`:165`) | none of its own | — |
| Runner behaviour | `SmpRepetitionHarness` `:735-897` (patched `BUILD`, `source_hash`, `run_guest`) | `:787-896` | `:773-779` |

Suggested margin test (new method in `SmpSerialOracle`, hand-written lines) **[I]**: accept one
valid line inserted at three positions of `online(4)` (after `SMP:ONLINE`, mid-sequence, before
`M1:PASS`) and assert the recorded entries; reject: missing ratio (`:12`), lower-case name,
`1/0`, sign, `18446744073709551616`, leading zero, trailing text, glued to another marker,
duplicate name, unknown name, changed limit, measured above limit, line after a terminal marker;
for `serial-unlocked-control`, a transcript with whole fixture lines plus an open-phase margin
line must still raise ("Unlocked control showed no mixed line"); one assertion that no name in the
host table contains a `FORBIDDEN` substring; one size assertion for a 50-boot report (H5).

### 2.3 Commands from the repository root

| Command | Source |
|---|---|
| `python3 -B -m unittest discover -s tools/tests -p test_smp_serial.py -v` | `.github/workflows/native-smp.yml:66` [S] |
| `python3 -B -m unittest discover -s tools/tests -p test_smp_acceptance.py -v` | same form [I]; covered in CI by `-p 'test_*.py'` (`.github/workflows/ci.yml:100`, `native-smp.yml:546`) [S] |
| `python3 -B -m unittest discover -s tools/tests -p test_smp_ci.py -v` | three tests are skipped without PyYAML (`tools/tests/test_smp_ci.py:15,62,94,162`) [S] |
| `python3 -B -m unittest discover -s tests -p 'test_ci_workflows.py'` | `ci.yml:96`; needs PyYAML 6.0.3 (`.github/actions/setup-python/requirements.txt`) [S] |
| `python3 -m unittest discover -s tools/tests -p 'test_*.py'` | `ci.yml:100` (job `host-quality`, every pull request) [S] |

`test_smp_serial.py:317` imports `test_smp_acceptance` inside a test; this relies on the
discovery start directory being on `sys.path` **[I]**.

## 3. (c) Recipe: one new negative regression case

Worked example: commit `2543861` added `quarantine-sched-busy` in 14 files (`git show --stat`)
**[S]**. `quarantine-lock-pending` is the shorter variant: it reuses outcome `lock-held`, so it
needed no oracle branch.

| # | Place | Precedent (file:line) | Change | Enforced by |
|---|---|---|---|---|
| 1 | Cargo feature | `kernel/Cargo.toml:43-44` | `test-smp-<x> = ["native-smp"]` (or an M3 parent, `:54-69`) | cargo |
| 2 | Kernel fixture | `kernel/src/m2/smp.rs:936-943` (lock-pending); `:3152-3154`, `:3311-3319`, `:3560-3565`, `:3678-3684` (sched-busy); controlled stop `:1473-1478` (`serial::exit(0x10)`) | branches under `cfg!(feature = "test-smp-<x>")` | — |
| 3 | Build choices | `tools/build.py:58` (`KERNEL_FEATURES`), used as argparse choices `:424` | add the feature name | argparse |
| 3a | Name traps | `tools/build.py:340-342` (a feature name containing `m3`…`m6c` builds a user ELF); `native-smp.yml:412-417` and `tools/run_smp_matrix.py:27,75` (profile names matching `*m3*` run the FPU probe) | choose the name accordingly | — |
| 4 | Oracle case | `tools/run_smp.py:50-51` | `"<case>": ("test-smp-<x>", cpus, "single"|"multi", "<outcome>")` | `aggregate`, verifier |
| 4a | New outcome only | `expected_smp` branch `:281-288`; `QUARANTINE_OUTCOMES` `:91-93` (boot continues on one CPU → `online = 1`, `:583-584`); `TERMINAL` `:94-109` (stop at a marker → `required_end`, `:441-444`); M3: `M3_OUTCOMES` `:367`, `m3_end` `:379-438`; watchdog map `:630` | as needed; unknown outcome raises (`:363`) | host tests |
| 5 | Host fixture | `tools/tests/test_smp_serial.py:159-161`, `:191` | transcript constant and `VALID` entry (or `ACCEPTANCE_CASES`, `test_smp_acceptance.py:25-28`) | `test_smp_serial.py:317-318` |
| 5a | Host negatives | `test_smp_serial.py:462-472` | rejected mutations, including the pre-fix verdict, and one accepted assertion | — |
| 6 | Pull-request step | `.github/workflows/native-smp.yml:171-176`, `:177-182` | build + `run_smp.py --profile test-smp-<x>` in job `native-smp` (job limit 20 min, `:41`) | none (omission is silent) |
| 7 | Matrix row | `native-smp.yml:329-330` (list `:302-366`) | `- <case>` | `tools/tests/test_smp_ci.py:71-74` (`set(matrix) == set(CASES) - {soak}`) |
| 8 | Aggregate job name | `native-smp.yml:509` `SMP section 9 aggregate 66x50` | `67x50` | `verify_final_gate.py:197,217` |
| 9 | Verifier counts | `docs/verification/k06-smp-sched/verify_artifacts.py:62` (`len(CASES) == 66` and message), `:298-299` (`3300` twice), `:308` (`case_count == 66`), `:309` (`guest_run_count == 3300`) | 67 / 3350 | — |
| 9a | Stale text | `verify_artifacts.py:4` still says "64 x 50" | optional | — |
| 10 | Final-gate verifier | `verify_final_gate.py:42`, `:197`, `:217` (string `66x50`) | `67x50` | — |
| 11 | Checksums | `docs/verification/k06-smp-sched/SHA256SUMS:126-127` (verifiers), `:3` (README), plus any other changed file in that directory | recompute; current values match the files (`53fee3ec…`, `150b51f5…`) | no test reads this list (grep of `tests/`, `tools/`) |
| 12 | Decision record | `docs/DECISIONS.md:2662`, `:2678` | addendum with the new count | — |
| 13 | State and evidence docs | `docs/PROJECT-STATE.md:48,53` (older `:62,69`); `docs/verification/k06-smp-sched/README.md:137,143`; `docs/progress.json` assessment (changed in `2543861`), then `tools/progress.py --write` and `--check` | update | `ci.yml:165-172` |
| — | No change needed | `tools/aggregate_smp_ci.py:72-73` and `tools/run_smp_matrix.py:68,82` derive counts from `CASES`; `tools/tests/test_smp_ci.py:49-50` likewise; `tests/test_ci_workflows.py` contains no SMP count, job name or matrix assertion (grep: `66` occurs only inside a hash at `:28`) | — | — |

Derived numbers after one addition **[S, arithmetic]**: 67 cases, 3 350 guests, 66 matrix rows,
10 soak shards. A `full_matrix` dispatch today starts 65 + 10 + probe + aggregate + final gate =
78 jobs (ADR-082 says 79, `docs/DECISIONS.md:2701`; the difference was not resolved **[U]**).
Other stale counts not touched by earlier additions: `COPILOT-HANDOFF.md:54` (64 cases, 3200),
`map/K06A-SPEC.md:79`.

## 4. (d) Host-side time limits

| Limit | Value | Location | Applies to |
|---|---|---|---|
| Default guest watchdog | 60 s | `tools/run_smp.py:130`; selection `:630` | 63 of 66 cases |
| Soak watchdog | 660 s | `tools/run_smp.py:133` | `m3-soak-four` |
| IPC watchdog | 660 s | `tools/run_smp.py:139` | `m3-ipc-four`, `m3-ipc-four-single` |
| Harness maximum | 660 s; range check | `tools/run.py:30`, `:67-68`; default 60 `:48` | `run_guest` |
| QEMU kill | `subprocess.run(..., timeout=timeout)`; on expiry `timed_out=True`, exit `None`, partial output kept | `tools/run.py:138-141` | each boot |
| `qemu --version` | no timeout | `tools/run.py:133` | each boot |
| Build step | 180 s per `run_logged` call (default) | `tools/common.py:153`; cargo builds `tools/build.py:362` | each build command |
| Job `native-smp` | 20 min | `native-smp.yml:41` | pull request / plain dispatch |
| Job `smp-environment-probe` | 30 min; one boot (`--repeat 1`) | `:236`, `:271` | full matrix |
| Job `smp-case-50` | 330 min; `--repeat 50`; `max-parallel: 10`; `fail-fast: false` | `:296-299`, `:418` | 65 jobs |
| Job `smp-soak-50` | 90 min; 10 shards × `--repeat 5`; `max-parallel: 10` | `:444-449`, `:487` | 10 jobs |
| Job `smp-matrix-aggregate` | 30 min | `:513` | 1 |
| Job `smp-final-gate` | 360 min | `:569` | 1 |
| Job `diagnostic-clock` | 90 min; trace-help 15 s; asserts watchdog 60 | `:619`, `:755`, `:778`, `:696` | diagnostic |
| Job `experimental-m3-m4-smp` | 45 min; per profile 600 s | `:875`; `tools/test_smp.py:77` | legacy |
| Aggregate shape | 50 per case; soak 10 × 5 | `tools/aggregate_smp_ci.py:13-16`, `:40-45`, `:54-60` | acceptance |
| Supervisor watchdog | 180 s | `tools/run_supervisor.py:28` | final gate K05 |
| Local matrix | 24-vCPU budget, 12 workers, no per-task timeout | `tools/run_smp_matrix.py:26`, `:107`, `:46-51` | local only |
| Evidence retention | compact 3 days; full raw 1 day | `.github/actions/preserve-evidence/action.yml:85`, `:96` | artifacts |
| Evidence size | 1 MiB per file, 8 MiB total (7 MiB usable), 500 files; JSON files first, then by path | `tools/ci_evidence.py:28-30`, `:399`, `:408-412` | compact artifact |
| Diagnostic script | 120 s kill timer | `timing-investigation/timing.py:55` | local diagnostics |

Duration measurement **[S]**: `start = time.monotonic()` after the version query
(`tools/run.py:134`); `duration_s = time.monotonic() - start` is evaluated when the report dict
is built (`:155`), after serial and stderr are written (`:143-144`), the ESP and firmware are
hashed (`:145-146`) and `git rev-parse HEAD` ran (`:154`). It therefore includes host
post-processing. `run_smp.py:668-675` requires a finite, non-negative number and
`timeout_s == watchdog`, then records `host_watchdog`; it applies **no threshold**.
`verify_artifacts.py:235-244` requires `0 <= duration <= deadline` and recomputes the three
derived fields exactly. A boot that ends just below the watchdog could exceed it in
`duration_s` without `timed_out` and would then pass `run_smp.py` but fail the verifier **[I]**.
The 2.5× / 264 s IPC rule (`run_smp.py:134-139`) is checked by no tool (grep `264`: no hit) **[S]**.

Worst-case sums against job limits **[S, arithmetic]**: 60 s cases 50 × 60 s = 50 min (limit 330);
IPC cases 50 × 660 s = 550 min, above the 330 min job limit (hosted 198–209 s per boot,
K06A-SPEC section 6 **[D, not recomputed]**); soak shard 5 × 660 s = 55 min (limit 90).
With `repeat > 1` a failed boot does not stop the series (`run_smp.py:702-703`); the process
exits 1 at the end (`:725`).

## 5. (e) Dispatch inputs of `native-smp.yml`

### 5.1 Existing wiring [S]

| Input | Type / default | Line |
|---|---|---|
| `full_evidence` | boolean / false | `:6-9` |
| `full_matrix` | boolean / false | `:10-13` |
| `clock_diagnostic` | boolean / false | `:14-17` |

Concurrency group `native-smp-${{ github.ref }}`, cancel-in-progress only for pull requests (`:28-31`).

| Job | `if` | Runner | QEMU machine | `needs` | Evidence name |
|---|---|---|---|---|---|
| `native-smp` `:38` | `!inputs.clock_diagnostic && (github.event_name != 'workflow_dispatch' \|\| !inputs.full_matrix)` `:39` | ubuntu-24.04 `:40` | pc-q35-8.2 `:43` | — | `native-smp-evidence` `:223` |
| `smp-environment-probe` `:232` | `github.event_name == 'workflow_dispatch' && inputs.full_matrix && !inputs.clock_diagnostic` `:234` | ubuntu-26.04 `:235` | pc-q35-10.0 `:238` | — | `runner-environment-probe` `:277` |
| `smp-case-50` `:291` | same `:293` | ubuntu-26.04 `:295` | pc-q35-10.0 `:368` | probe `:294` | `smp-case-<case>` `:424` |
| `smp-soak-50` `:439` | same `:441` | ubuntu-26.04 `:443` | pc-q35-10.0 `:451` | probe `:442` | `smp-soak-<shard>` `:493` |
| `smp-matrix-aggregate` `:508` | `always() && …` `:510` | ubuntu-24.04 `:512` | none | case + soak `:511` | `smp-section9-matrix-report` `:554`; downloads `pattern: smp-*` into `build/k06-smp-artifacts` `:527-531` |
| `smp-final-gate` `:564` | as probe `:566` | ubuntu-26.04 `:568` | pc-q35-10.0 `:571` | aggregate `:567` | `smp-section9-final-gate` `:601` |
| `diagnostic-clock` `:615` | `github.event_name == 'workflow_dispatch' && inputs.clock_diagnostic` `:617` | ubuntu-26.04 `:618` | pc-q35-10.0 `:621` | — | `DIAGNOSTIC-NOT-ACCEPTANCE-smp-clock-<run_id>-<attempt>` `:850`, `full-evidence: 'true'` `:865` |
| `experimental-m3-m4-smp` `:870` | as `native-smp` `:873` | ubuntu-24.04 `:874` | pc-q35-8.2 `:877` | — | `experimental-smp-evidence` `:905` |

Uploaded artifact names: compact `<name>-reports`, full `<name>` only on dispatch with
full evidence (`preserve-evidence/action.yml:82`, `:89-93`).

QEMU setup (`.github/actions/setup-qemu/action.yml`): on 26.04 `qemu-system-x86=1:10.2.1+ds-1ubuntu3.3`,
`ovmf=2025.11-3ubuntu7.3`, machine `pc-q35-10.0`, and the pinned `python3.14=3.14.4-1ubuntu0.2`
packages (`:52-74`); on 24.04 QEMU `1:8.2.2+ds-0ubuntu1.18`, `ovmf=2024.02-2ubuntu0.10`,
`pc-q35-8.2` (`:33-47`). The runner image version and package list are written to
`$RUNNER_TEMP/nieuw-os-qemu-environment.txt` (`:150-165`); only `diagnostic-clock` copies it into
its evidence (`native-smp.yml:661`). Because the pinned Python comes from that bundle, 26.04 jobs
run `setup-qemu` before `setup-python` with `system-python: 'true'` (`setup-python/action.yml:31-38`);
`tools/tests/test_smp_ci.py:82-88` pins this order for four jobs.

Clock diagnostic specifics: checkout at `github.sha` (`:628-632`); scope file `NOT-ACCEPTANCE.txt`
(`:633-640`); guard step exits 2 unless `full_matrix` is `false`, before QEMU setup (`:641-649`);
in-process `run_smp.run(('m3-one-multi',), repeat=50)` with a patched `subprocess.run` (`:668-839`).

### 5.2 Minimal design for a new diagnostic input [I: design; cited lines are S]

Goal: run chosen cases N times with the commands, runner label and QEMU of `smp-case-50`; label
everything non-acceptance; make it unusable by the aggregate, the verifier and the final gate.

Inputs:

```yaml
      margin_diagnostic:
        description: 'DIAGNOSTIC ONLY: run margin_cases margin_repeat times on the full-matrix profile; requires full_matrix=false and clock_diagnostic=false'
        type: boolean
        default: false
      margin_cases:
        description: 'DIAGNOSTIC ONLY: space-separated tools/run_smp.py case names'
        type: string
        default: 'eight nine-capacity m3-load-two'
      margin_repeat:
        description: 'DIAGNOSTIC ONLY: boots per case (1-50)'
        type: string
        default: '50'
```

A boolean switch keeps every `if:` in the form the existing test evaluator handles
(`tools/tests/test_smp_ci.py:106-135`: attribute lookups, `!`, `&&`, `||`, `==`, `!=`, `always()`).

| Element | Design | Why |
|---|---|---|
| Existing jobs | append `&& !inputs.margin_diagnostic` to all eight `if:` expressions, including `diagnostic-clock` | one mode per run; on pull requests the input is absent and the term is true, as for `!inputs.clock_diagnostic` today (`:39`) |
| `diagnostic-margin-plan` | `if: github.event_name == 'workflow_dispatch' && inputs.margin_diagnostic`; ubuntu-24.04 with default `setup-python` (as the aggregate job, `:512`, `:519-522`); checkout at `github.sha`; first step: exit 2 unless `full_matrix` and `clock_diagnostic` are `false` (pattern `:641-649`); second step: Python heredoc importing `CASES` (pattern `:402-409`), requiring non-empty, unique, known names and `1 <= N <= 50`, and `N × watchdog <= 300 min` using `GUEST_TIMEOUT_S`, `SOAK_TIMEOUT_S`, `IPC_TIMEOUT_S`; outputs `cases` (JSON list) and `repeat` | input validation before any guest; no workflow-side copy of the case list |
| `diagnostic-margin` | `needs: diagnostic-margin-plan`; same `if`; `runs-on: ubuntu-26.04`; `timeout-minutes: 330`; `strategy: {fail-fast: false, max-parallel: 10, matrix: {case: ${{ fromJSON(needs.diagnostic-margin-plan.outputs.cases) }}}}`; `env` as `:368-373` with `NOS_BUILD_DIR: …/build/k06-margin-diagnostic-${{ matrix.case }}`; steps `:375-397` unchanged (checkout, QEMU, Python, Rust, quiet services) | same image label, package pins, machine type and one case per runner as the matrix |
| Run step | the commands of `:402-417`, then `run_smp.py --case "$SMP_CASE" --repeat "$SMP_REPEAT"` with the exit status kept; write `diagnostic/NOT-ACCEPTANCE.txt` and `diagnostic/scope.json` (`classification: DIAGNOSTIC_NOT_ACCEPTANCE`, pattern `:739-751`); copy `$RUNNER_TEMP/nieuw-os-qemu-environment.txt` (pattern `:661`); **move** `smp-report.json` to `diagnostic/margin-diagnostic-report.json` when it exists; `git diff --exit-code`; exit with the kept status | a red boot stays red; the renamed report cannot be found by `rglob("smp-report.json")` (`tools/aggregate_smp_ci.py:21`, `verify_artifacts.py:114`) |
| Evidence | `name: DIAGNOSTIC-NOT-ACCEPTANCE-smp-margin-${{ matrix.case }}-${{ github.run_id }}-${{ github.run_attempt }}`; paths: manifest, `fpu-fixture-host.log`, `diagnostic/**`, `run/smp-*/{result.json,serial.txt,stderr.txt}`; `required-paths`: manifest and the renamed report; `if-no-files-found: error`; `full-evidence: 'true'` | name is valid for `ci_evidence.py:375`; it does not match the aggregate's `smp-*` download pattern (`native-smp.yml:530`) nor the verifier's name rule (`verify_artifacts.py:132-133`) |

Separation from acceptance, by layer:

| Layer | Mechanism | Basis |
|---|---|---|
| Job eligibility | no acceptance job is eligible when `margin_diagnostic` is true | `if:` design; test table |
| Artifact name | prefix `DIAGNOSTIC-NOT-ACCEPTANCE-` | `native-smp.yml:530`; `verify_artifacts.py:132-133` [S] |
| Report file name | not `smp-report.json` | `aggregate_smp_ci.py:21`; `verify_artifacts.py:114` [S] |
| Aggregate shape | needs one passing 50/50 report for each of the 66 cases and 10 soak reports | `aggregate_smp_ci.py:40-60` [S] |
| Final-gate readback | needs successful jobs named `SMP section 9 aggregate 66x50` and `SMP final repository gate` in the same run | `verify_final_gate.py:184-202` [S] |
| Cross-run | the aggregate downloads artifacts of its own run only | external action behaviour [U] |

Constraints to state in the job: compact evidence keeps at most 499 files and 7 MiB per artifact
and 1 MiB per file (section 4), so N ≤ 50 per job with the present report format and fewer if
the report grows (H5); full raw evidence expires after one day; the dispatch shares the
concurrency group with a full matrix on the same ref (`:28-31`; queueing behaviour **[U]**);
hosted hardware beyond the image label cannot be pinned ("4 vCPUs = 2 cores with SMT",
`TIMING-INVESTIGATION.md:11`).

### 5.3 Tests that pin the workflow structure [S]

| Test | Pins | Change needed for 5.2 |
|---|---|---|
| `tools/tests/test_smp_ci.py:95-160` `test_clock_diagnostics_cannot_launch_acceptance_jobs` | default of `clock_diagnostic`; evaluates **every** job's `if` (`job["if"]`, `:147-148`) with a value table that knows only `github.event_name`, `inputs.full_matrix`, `inputs.clock_diagnostic` (`:109-110`); expected job sets per input combination (`:137-145`); guard step of `diagnostic-clock` before QEMU setup and its exit codes (`:150-160`) | **Yes.** A new input in any `if:` raises `KeyError` at `:116` until `values` and the `eligible` signature are extended; add rows for `margin_diagnostic` (alone, with `full_matrix`, with `clock_diagnostic`); add the same guard check for the plan job; every new job needs an `if` |
| `tools/tests/test_smp_ci.py:63-92` `test_workflow_partitions_every_case_and_gates_after_aggregate` | matrix rows equal `CASES` minus soak, no duplicates; shards `0..9`; `needs` of case, soak, aggregate, final gate; `runs-on: ubuntu-26.04` for case and soak; probe machine `pc-q35-10.0`; `native-smp` step 2 has no `with`; QEMU-before-Python and `system-python` for four named jobs | No, if those jobs keep their names, `needs` and steps. Recommended addition: assert that `diagnostic-margin` has the same `runs-on`, `NOS_QEMU_MACHINE` and first five steps as `smp-case-50`, and an evidence name starting `DIAGNOSTIC-NOT-ACCEPTANCE-` |
| `tools/tests/test_smp_ci.py:162-180` | pinned package strings in `setup-qemu` | No |
| `tests/test_ci_workflows.py:350-356` | every job `runs-on` in {ubuntu-24.04, ubuntu-26.04} | No (must hold) |
| `:400-431` | no `pull_request_target`; `contents: read`; job permissions read/none | No (must hold) |
| `:433-442` | `uses:` pinned to full commits or local paths | No (must hold) |
| `:640-645` | checkout `persist-credentials: 'false'` | No (must hold) |
| `:665-675` | every `paths-ignore` equals `METADATA_PATHS_IGNORE` (`:34-42`) | No, triggers unchanged |
| `:697-707` | `continue-on-error` only on upload steps of the evidence action | No (must hold) |
| `:731-742` | pull-request types; no `draft` in a job `if` | No |
| `:391-398` | `NOS_QEMU_ACCEL` appears in no workflow except `kvm-pilot.yml` | No (must hold) |
| `.github/workflows/ci.yml:32-50` | actionlint 1.7.12 on all workflows | must pass |

`tests/test_ci_workflows.py` contains no assertion specific to the job structure, inputs, matrix
or counts of `native-smp.yml`; its only mention is the path-filter sample at `:690`.

## 6. (f) Jobs started by a push to a pull-request branch

Trigger blocks were read from all 28 workflow files **[S]**.

| Filter | Workflows (jobs) |
|---|---|
| None (always) | `ci.yml` (`host-quality`, `qemu-smoke`; `:10-13`) |
| Pull requests targeting `main` | `codeql.yml` (`analyze`) |
| `paths-ignore` = docs, `**.md`, issue/discussion templates, project-classification files | `app-management-quality`, `component-leaves`, `entropy-keys` (`b01-native-supervisor`), `nas-core` (2 jobs), `native-display-input`, `native-foundation`, `native-power` (2 jobs), `native-rtc`, **`native-smp`** (`:18-20`), `native-system`, `native-window-session`, `net-broker-quality` (2 jobs), `net-tls-quality`, `process-management` (`k05-native-supervisor`), `resource-pressure` (`k07-native`), `signed-updates`, `windows-qemu-test-version` |
| `paths` (positive list) | `storage-resilience` (`s08`) and `volume-acceptance` (`m6e`): `tools/**`, `kernel/**`, `boot/**`, `userspace/**`, `services/**`, `drivers/**`, `crates/**`, `.cargo/**`, root `Cargo.*`, `rust-toolchain.toml`, own workflow, evidence action (plus `sdk/**` for volume); `artifact-store-quality`: root `Cargo.*`, `rust-toolchain.toml`, `.cargo/**`, `.github/actions/**`, `crates/artifact-store-core/**`; `copilot-setup-steps`: own file, setup actions, `rust-toolchain.toml` |
| Not started by a pull request | `kvm-pilot`, `materiaal-probe`, `project-classification`, `clear-stale-actions`, `private-publication-audit` |

A change to `kernel/src/**`, `tools/**` or `.github/workflows/native-smp.yml` starts every
`paths-ignore` workflow, `ci.yml`, `codeql.yml`, `storage-resilience` and `volume-acceptance`;
a docs-only change starts `ci.yml` and `codeql.yml` only. Every job reachable from a pull
request runs on `ubuntu-24.04`; those with a guest use `pc-q35-8.2` (line table extracted from
the workflows: `runs-on` and `NOS_QEMU_MACHINE` of each job) **[S]**. `ubuntu-26.04` with QEMU
10.2.1 and `pc-q35-10.0` is used only by dispatch-only jobs of `native-smp.yml`.

Pull-request run of `native-smp.yml` **[S]**:

| Job | Content |
|---|---|
| `native-smp` (ubuntu-24.04, QEMU 8.2.2, `pc-q35-8.2`, Python 3.12.12, 20 min) | host checks `:60-70` (`boot-protocol`, `smp-lock`, `test_smp_serial.py`, `test_gs_discipline.py`, `test_acpi_serial.py`, `cargo fmt`); then build + `run_smp.py --profile <p>` once for 31 profiles (`:71-214`) = 43 of 66 cases, one boot each (AST count); evidence `native-smp-evidence` |
| `experimental-m3-m4-smp` (ubuntu-24.04, `pc-q35-8.2`, 45 min) | `tools/test_smp.py` (15 legacy profiles) |

Not run on a pull request: 23 cases in 12 profiles — `native-smp-m3-ipc` (2), `-migrate` (4),
`-load` (8), `-soak` (1), `native-smp-m4-quotas`, `native-smp-qsim`, `native-smp-m3-kill`,
`native-smp-m3-unmap`, `test-smp-m3-canary`, `test-smp-m3-fpu-skip`,
`test-smp-m3-no-shootdown`, `test-smp-m3-recovery` (1 each) — and no repetition above one.
`test_smp_acceptance.py` and `test_smp_ci.py` run in `ci.yml` `host-quality` (`:100`), not in
the `native-smp` job.

## 7. (g) Timing-investigation scripts

All three are diagnostic; none binds to a source hash, a manifest or the oracle **[S]**.

| Script | Arguments | Behaviour | Output |
|---|---|---|---|
| `timing-investigation/timing.py` (82 lines) | `esp cpus hostcpus repeat out [accel]` (`:6-7`): ESP directory, guest vCPUs, `taskset -c` list, boots, JSONL path (append), TCG thread mode (default `multi`) | Per boot: copies the OVMF variables to a temp directory, starts `taskset … qemu-system-x86_64 -machine pc-q35-8.2 … -name diag,debug-threads=on` with the device set of `run.py` (`:43-50`; machine and firmware paths are hard-coded, `:8`, `:43`); kills after 120 s (`:55`); timestamps each serial line on the host; samples `/proc/<pid>/task/*/{comm,schedstat,status}` every 50 ms (`:17-36`); reads steal ticks from `/proc/stat` (`:11-14`) | One JSON object per boot (`:73-80`): `i, cpus, hostcpus, accel, exit, wall, steal_ticks, total_ticks, marks, vcpu, other`. `marks` = `[line, seconds since start, seconds since previous mark]` for lines matching `(SMP\|M1\|M2):` except `SMP:SER:` (`:63-65`), plus a synthetic `DIAG:SYNC_WAIT_AFTER_BSP` (last `SMP:SER:0:` line → `SMP:SERIAL:PASS`, `:61-62`, `:66-67`). `vcpu`/`other` = `[comm, run_ns, wait_ns, slices, involuntary switches]` per thread |
| `analyse.py` (40 lines) | one or more JSONL files (`:15`) | Per file: non-pass boots (`exit != 33`) with their `FAIL`/`TIMEOUT` marks (`:21`); for the phases in `PH` (`:5-7`: sync total, sync wait, scratch, wake, lockpair, ICR, isolation, preempt) the distribution of the mark's delta to the previous mark (`:25-27`); clock deviation from 999 847 µs when `SMP:DIAG:CLOCK_US` marks exist (`:28-31`, diagnostic image only) | Text: header line, table `phase n min med p90 max (s)`, optional clock line, one summary line (wall median/max, vCPU run and run-queue wait, wait/run, involuntary switches, host steal %, boots without a vCPU sample; `:32-40`) |
| `prog.py` (31 lines) | one or more JSONL files (`:4`) | Needs the `diag-progress.patch` image, which prints `SMP:DIAG:P=<total>` every 50 guest polls (`:2`, `:10`). Per boot: sync duration (`SMP:TIMER:RATIO` → `SMP:SERIAL:PASS`, absolute times, `:11-12`, `:27`), number of progress lines, final count, tail after the final count (`:15-16`), longest span without progress (`:17-22`), largest line gap (`:23`), rate per second min/median/max (`:24`), vCPU run and wait seconds, `FAIL` marks | One text line per boot (`:28-31`) under a header per file (`:8`) |

Related: `map/hosted-margins.py` (`root out1 out2 names-file`) reads `result.json` files of
downloaded artifacts and writes `hosted-margins.tsv` (durations over watchdog) and
`hosted-markers.tsv` (ratio, fairness, wake, load extremes and lines matching
`TIMEOUT|DRIFT|STUCK|FAIL`) **[S]**. The meaning of the `schedstat` and `/proc/stat` fields is
Linux behaviour not checked here **[I]**.

## 8. Corrections to assumptions in the task text

| # | Assumption | Finding |
|---|---|---|
| C1 | `SMP:FAIR`, `SMP:TIMER:RATIO`, `SMP:IDLE:WAKE_US` are parsed and removed before the comparison | They are normalised in place and stay position-bound; only `SMP:SER` lines, the ACPI pair and control-phase lines are removed (section 1.2) |
| C2 | `tests/test_ci_workflows.py` holds count or name assertions for the matrix | It holds none; the pins are `tools/tests/test_smp_ci.py:63-92` and `:95-160`, `verify_artifacts.py:62,298-299,308-309`, `verify_final_gate.py:42,197,217`, `native-smp.yml:509` |
| C3 | Margin lines can be accepted "at arbitrary positions" by a filter alone | The filter must run before `serial_lines` (control case), must raise explicitly, and needs a trailing-line guard for terminal outcomes; emission must avoid concurrent writers in the unlocked-serial profile (sections 1.6, 1.7) |
| C4 | The aggregate can report margins | It reads no margin today; its test fixture reports have no `observed` (`tools/tests/test_smp_ci.py:19-31`) |
| C5 | Checks on the pull request cover the change | 43 of 66 cases run once on QEMU 8.2.2 / `pc-q35-8.2`; the SMP supervisor cases run only in the final gate |

## 9. Unknowns and unverified items

| # | Item | Status |
|---|---|---|
| U1 | Number and names of margin lines per boot and per case; whether any deadline repeats within one boot | [U] kernel design not in the snapshot |
| U2 | GitHub behaviour: `download-artifact` scope per run, queueing in a shared concurrency group, expression value of an absent input, input-count and job-duration limits | [U] external, not checked in this session |
| U3 | Uncompressed size of hosted compact artifacts (the table `map/hosted-artifacts.tsv` holds ZIP sizes, for example 555 496 bytes for `smp-case-eight-reports`) | [U] |
| U4 | Whether `duration_s` has ever exceeded a watchdog without `timed_out` | [U] |
| U5 | Effect of extra serial lines on guest timing inside timed windows | [U] needs measurement |
| U6 | Job count 78 (workflow) versus 79 (ADR-082) | [U] |
| U7 | Oracles of the other one-CPU regressions (`test_milestones.py`, `test_resource_pressure.py`, `run_rtc.py`, `run_ipc_wait.py`, `run_fpu.py`, `run_memory.py`) were not read line by line; the statement that non-SMP output is unaffected rests on feature gating | [I] |
| U8 | Host tests were not executed; statements about test outcomes after a change are read from the code | [I] |

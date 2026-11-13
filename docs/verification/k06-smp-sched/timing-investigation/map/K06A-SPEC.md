# K06A-SPEC: normative map of K06 step 5f timing limits and acceptance rules

Task ID: K06A-SPEC. Read-only. Source: frozen snapshot of commit `ec9a8b8`
(`scratchpad/snap-ec9a8b8`, cited as `path:line`) and the L0 specifications
(`scratchpad/specs/docs/implementation`, cited as `specs/<file>:line`). Hosted data:
run 43187501026 via `scratchpad/hosted-margins.tsv` and `scratchpad/hosted-markers.tsv`.
Date of analysis: 12 November 2026.

Status labels: **[S]** verified in source, **[D]** verified in data, **[I]** inferred,
**[U]** unverified.

## 0. Coverage bounds

- Fully read: `specs/K06-SMP.md`, `specs/k06-steps/01..09` and `README.md`,
  `tools/run_smp.py`, `tools/run.py`, `tools/aggregate_smp_ci.py`,
  `docs/verification/k06-smp-sched/{README,CLOCK-INVESTIGATION,TIMING-INVESTIGATION}.md`,
  the K06 parts of `docs/DECISIONS.md` (ADR-045/053/070..074, ADR-CI-*, ADR-K06-5F-20261109,
  all dated 5f addenda, ADR-082), `docs/PROGRESS.md`, `docs/progress.json` (K06 entry).
- Read at limit sites, not line by line: `kernel/src/m2/smp.rs` (4998 lines; every
  `within_ms`, `within_tsc`, `idle_until`, `wait_us` call and every named constant was
  enumerated by grep and read in context), `kernel/src/m2/m3.rs`, `kernel/src/m2/lock.rs`,
  `kernel/src/serial.rs`, `kernel/src/m2/m3/smp_lifecycle.rs`, `m3/smp_unmap.rs`.
  `m3/smp_recovery.rs` was not opened for limits (dropped for time; its only observed
  marker, `SMP:M3:SHOOTDOWN_TIMEOUT`, is an expected negative-case marker).
- `docs/FULL-TEST-RELEASE.md`: only the K06, evidence-code and sign-off passages.
- `.github/workflows/native-smp.yml`: job structure, matrix list, timeouts; not every step.
- Hosted Graphify MCP was not used (it indexes `main`; all facts were taken from the
  snapshot directly). No QEMU, cargo or build command was run.
- Hosted data limited to the two TSV extracts; raw serial logs were not re-read.

## 1. Normative hierarchy

| Level | Document | Role |
|---|---|---|
| L0 spec (Dutch) | `specs/K06-SMP.md`, `specs/k06-steps/01..09` | Normative acceptance requirements. `docs/DECISIONS.md:2543`: "The normative acceptance requirements remain L0 step 5f". |
| ADR / owner decisions | `docs/DECISIONS.md` (ADR-K06-5F-20261109 `:2539-2562`, dated addenda `:2567-2678`) | Fix numeric limits the spec delegates ("vooraf in de ADR") and record owner-approved changes. |
| Proposed, not in force | ADR-082 `docs/DECISIONS.md:2680-2741` | "Nothing in this entry is in force." Each part needs an explicit owner decision. |
| Register | `docs/PROGRESS.md:17-19,37`, `docs/progress.json:75-80` | K06 is `open`, evidence `[]`. |
| Release contract | `docs/FULL-TEST-RELEASE.md:74,101-104,131,146-148` | Config frozen before acceptance runs; no limit removed; failed run stays failed. |

Missing ADR bodies **[S]**: ADR-075, 077, 078, 079, 080 and 081 have no heading or body
in `docs/DECISIONS.md` at this commit (grep). They exist only as references in addendum
titles and code comments (for example `tools/run_smp.py:120-122` "The band was fixed in
ADR-080 before the first measurement").

## 2. Normative timing and acceptance requirements

Quotes are in the original language. "Impl" is the implementing code or oracle site.

### 2.1 Requirements with a numeric value

| # | Requirement (quote) | Source | Value | Impl (snapshot) | Measured by |
|---|---|---|---|---|---|
| N1 | "Eis binnen 500 ms: alle patronen kloppen, teller = 1000 × (online+1), lockdep leeg." | `specs/k06-steps/01-5b-2-kernel-lock-scratch.md:53-57` | 500 ms; 1024 fills; 1000 KERNEL rounds | `kernel/src/m2/smp.rs:3711-3712` (rounds), `:3785` `within_ms(500, …)` → `SMP:SCRATCH:TIMEOUT` + `quarantine_all` `:3790-3792` | PIT ch2 polling (`within_ms`, `:1396-1403`); clock starts after the BSP's own share (`:3784`, comment `:3763`) — an implementation interpretation of "binnen 500 ms" **[I]** |
| N2 | "DG-1 … De grens is een tijd en geen iteratietelling: `LOCK_STUCK_MS = 4 × SHOOTDOWN_DEADLINE_MS` (400 ms), gemeten met de PIT-klok. … *shortcut: PIT-polling loopt onder TCG mee met de hostklok; vervang dit door een gekalibreerde TSC zodra 06 de verhouding meet.*" | `specs/k06-steps/03-F5-F7-stuck-serial.md:17-21`; retained by `docs/DECISIONS.md:2547` | 400 ms | `smp.rs:334`; `smp.rs:353-359` (`stuck_deadline`/`past`, TSC); `kernel/src/m2/lock.rs:239-261` (every contended `IrqLock`, including SERIAL) → `SMP:LOCK:STUCK:<level>:CPU=` + fail (`smp.rs:2086-2100`) | Calibrated TSC |
| N3 | "lock-stuck … binnen 1 s en exit 0x10, geen host-timeout" | `specs/k06-steps/03-…:40-41` | 1 s (negative case) | `smp.rs:1593` `within_ms(1_000, \|\| false)` hold; oracle outcome in `tools/run_smp.py` CASES | PIT |
| N4 | "Eis: ack binnen 100 ms, geen quarantaine. Marker `SMP:BSP_SPIN:PASS`." | `specs/k06-steps/06-5c-2-bsp-timer-idle.md:52-53` | 100 ms | `smp.rs:312` `SHOOTDOWN_DEADLINE_MS = 100`; `:2920-2923` `within_tsc` + `round`; `:2944` `within_ms(100)` | TSC (ack) and PIT (HELD wait) |
| N5 | "One shootdown generation has one 100-ms calibrated-TSC deadline, independent of the number of target CPUs; it is not restarted per acknowledgement." | `docs/DECISIONS.md:2549-2550`; spec `K06-SMP.md:311` "tot de deadline (dezelfde als ADR-072)"; ADR-072 body `:246-293` states no number | 100 ms per generation | `smp.rs:1796-1826` (`round`), `:1716-1722` (`SMP:SHOOTDOWN:TIMEOUT`, `FRAME_QUARANTINED`) | TSC |
| N6 | "wacht begrensd (deadline gelijk aan de shootdown-deadline) … Geen ack binnen de deadline → INV-5." / "BW-3 Kill is begrensd … wacht op `DEAD` binnen `SHOOTDOWN_DEADLINE_MS`" | `K06-SMP.md:279-282`; `specs/k06-steps/04-F16-F17-block-wake-kill.md:26-29,52-53` | = shootdown deadline (100 ms) | `smp.rs:3686` `within_ms(SHOOTDOWN_DEADLINE_MS, DEAD)` → `SMP:KILL:TIMEOUT`; `kernel/src/m2/m3.rs:2969` `KILL_ACK_MS = 100`; `m3/smp_lifecycle.rs:11` `ACK_LIMIT_MS = 100`; oracle `run_smp.py:524-529` (WAIT_MS < 100) | PIT in `smp.rs:3686`; TSC in M3 |
| N7 | "AC-2 Eerlijkheid begrensd (F10): … het maximum van `deferred` en de wachttijd op `KERNEL` onder een grens die vooraf in de ADR staat. Overschrijding → `SMP:FAIR:FAIL:CPU=<n>`." | `specs/k06-steps/09-5f-acceptance.md:17-19`; values in `docs/DECISIONS.md:2547` ("retained at 400 ms … bounded at 40 … will not be enlarged in response to a failing test without a separate decision") | 400 ms wait; 40 deferred ticks | `smp.rs:4737`, `:4766-4784` (`m3_wait_end`), `:4788-4810` (report line); `lock.rs:242-247,264-267`; oracle `run_smp.py:371-372,550-564` | TSC (`m3_time_ms`, `smp.rs:4926`) |
| N8 | "**Klok:** tijdens de bestaande SMP-belasting blijft Sys3 monotoon en stijgt hij binnen 1% van de PIT-verwachting. Marker `SMP:CLOCK:PASS`." — revised by owner decision of 11 Nov: "Precision (all APs halted) … the median segment must last the PIT's own period within 1%. The limit stays at 1%." / "Load … count `CLOCK_SEGMENT` further IRQ0 ticks before a 4× deadline … There is no 1% limit under load." | `specs/k06-steps/06-…:48-49` (text not yet revised; open item `docs/DECISIONS.md:2648`); `docs/DECISIONS.md:2635-2648`; `:2567` ("Five target segments, their median, the observation deadline and the exact 1% drift limit remain fixed … Missing IRQs are not synthesized"); ADR-053 `:843` (counted-IRQ0 rule) | 1 % on the median of 5 × 1000-tick segments (idle); 4× progress deadline (load) | `smp.rs:2501-2504`, `:2700-2702`, `:2748-2849`; test `:2789` `us.abs_diff(expected) * 100 > expected` → `SMP_CLOCK_DRIFT` (`:2790-2811`); load bound `:2829`; `SMP_CLOCK_STOPPED` `:2782-2783,2834-2835`; `SMP_CLOCK_NO_LOAD` `:2843` | PIT-calibrated TSC vs counted IRQ0 |
| N9 | "BT-5 Gekalibreerde band. Bij de boot telt de BSP LAPIC-ticks per PIT-tick over 100 ms en logt `SMP:TIMER:RATIO:<r>`. De gate eist een band [min, max] die vooraf in de ADR staat en niet achteraf wordt aangepast." | `specs/k06-steps/06-…:34-36`; `K06-SMP.md:286-289` | Band value is not in any ADR text of this commit; code uses 50 000–75 000 | `smp.rs:2585-2599`; `run_smp.py:124` `RATIO_BAND = range(50_000, 75_001)` | — |
| N10 | "AC-3 … Een geval telt alleen als groen bij **50/50** herhalingen." / "`tools/run_smp.py --repeat 50` … Eén flake = niet groen, en de oorzaak wordt onderzocht. De deadline wordt niet verhoogd zonder ADR." | `specs/k06-steps/09-…:20-21,52-53`; `docs/DECISIONS.md:2558` | 50/50 per case; no deadline increase without ADR | `run_smp.py:704-705`; `tools/aggregate_smp_ci.py` (50/50, flaky false, shard counts); `.github/workflows/native-smp.yml:418,487` | — |
| N11 | "Een soakprofiel van 10 min op 4 vCPU's (multi-thread-TCG) met de belasting van §3.1 en §3.7." | `specs/k06-steps/09-…:54`; `docs/DECISIONS.md:2551-2556` | 600 s guest time, 4 vCPU, MTTCG | `m3.rs:3274-3279` (600 000 ms by TSC); `run_smp.py:535-539`; `SOAK_TIMEOUT_S` `:133` | TSC |
| N12 | "≥ 2× zoveel CPU-gebonden ring-3-processen als CPU's, plus een IPC-ping-pong tussen processen op **verschillende** CPU's, 30 s. Eis per CPU: user-ticks > 0, contextwissels ≥ 4, geen auditfout, en alle K07-quota blijven per app begrensd." | `K06-SMP.md:363-365`; `specs/k06-steps/09-…:28-30` | 30 s; ≥ 4 switches per CPU | `m3.rs:3278` (30 000 ms), `:3296` (`ticks >= 64`), `:3304` (`selected >= 4`); `run_smp.py:535-539` (`workers == 16`, `elapsed >= 30_000`) | TSC |
| N13 | "per CPU user-ticks > 0 en ≥ 4 wissels"; IPC "10.000 rondes, zonder verloren antwoord"; FPU canary "na 1000 wissels" | `specs/k06-steps/08-5d-2-ring3-on-aps.md:48-54` | 10 000 rounds; 1000 switches | `m3.rs:3255` `MIN_SWITCHES = 1000`, `:3149,3193-3194`; `run_smp.py:370` | count |
| N14 | "at least 8 ticks and 4 context switches per AP" | ADR-073 `docs/DECISIONS.md:221-223` | 8 ticks, 4 switches | `smp.rs:1973-1974` | count |
| N15 | "200 regels per CPU"; serial-unlocked control "in minstens één van tien runs" | `specs/k06-steps/03-…:38-39,42-44` | 200 lines | `smp.rs:363-373`; `run_smp.py:143` | count |
| N16 | "`settle_scheduler` waits at most 200 ms" (owner decision, 12 Nov) | `docs/DECISIONS.md:2669-2678` | 200 ms | `smp.rs:1426-1437` (`within_ms(200, …)` `:1432`) | PIT |
| N17 | "`IPC_TIMEOUT_S` becomes 660 s … The 2.5× rule stays: if the hosted per-boot time of `m3-ipc-four` in the next full run exceeds 264 s, the margin is insufficient and K06 stays open; the runner question then returns to the owner." | `docs/DECISIONS.md:2650-2656` (earlier `:2617-2625`: "`GUEST_TIMEOUT_S` stays 60") | 660 s watchdog; 264 s acceptance bound | `run_smp.py:134-139,630`. The 264 s bound is not enforced by any tool (grep "264" in `tools/` and the workflow: no hit) | host wall clock |
| N18 | "Existing per-boot host watchdog … This is distinct from guest deadlines and must not grow in response to a failed repetition." | `run_smp.py:128-130` (code comment); `docs/DECISIONS.md:2623` | 60 s | `run_smp.py:130`; `tools/run.py:138` (`subprocess.run(…, timeout=timeout)`) | host wall clock |
| N19 | "PS-2 … ≤ 32 pagina's per invlpg"; isolation 10.000×; ICR "10.000 0xe8 … Eis: ontvangen = verzonden per paar, geen timeout"; ping-pong 10.000× "geen timeout"; lock_pair 1000× "Er mag geen deadlock en geen marker optreden" | `specs/k06-steps/07-…:26-30,49-59`; `05-…:47-49`; `04-…:50-51`; `02-…:46-48` | Round counts only; **no time value** | `smp.rs:3506` (`WAKE_ROUNDS`), `:3910` (`ICR_ROUNDS`), `:4041` (`ISO_ROUNDS`), `:3834` (`PAIR_ROUNDS`) | count |

### 2.2 Requirements without a number (qualitative, binding)

| # | Requirement (quote) | Source | Impl |
|---|---|---|---|
| Q1 | "Bewijsniveau: native QEMU q35/OVMF/TCG; fysieke hardware valt buiten K06." | `K06-SMP.md:12-14` | `tools/run.py:104` |
| Q2 | "Profielen: 1, 2, 4 en 8 vCPU's; zowel single- als multi-thread-TCG (`-accel tcg,thread=multi`)." | `K06-SMP.md:360`; `09-…:26`; `06-…:71`; `07-…:71`; `08-…:68` | `run_smp.py:22-90` (CASES, 66 entries); workflow matrix `native-smp.yml:302-366` (65 entries + soak shards) |
| Q3 | INV-5: "Een CPU die een shootdown of reschedule-IPI niet binnen de deadline bevestigt, wordt nooit genegeerd: frames blijven in quarantaine en de CPU wordt gereset met INIT… Een mislukte INIT stopt de boot." | `K06-SMP.md:67-69` | `smp.rs:1408-1418` (`quarantine_all`) |
| Q4 | Self-tests "blijven verplichte poorten: als er één faalt, worden alle AP's gequarantaineerd (INIT) en draait de boot verder als één CPU" | `K06-SMP.md:88-90` | every `quarantine_all` path; host oracle then rejects by exact sequence (`run_smp.py:581-582`) |
| Q5 | "AC-3 Een betrouwbare gate (F12): elke deadline is een benoemde constante met motivatie. De gate meet en logt de marge (gemeten / deadline)." | `09-…:20-21` | Only partly implemented, see section 3.3 |
| Q6 | "AC-4 … een geval met `CPUS=1` telt als regressie, niet als SMP-bewijs." | `09-…:22`; `docs/DECISIONS.md:2558` | `run_smp.py` report |
| Q7 | "Regressie (ongewijzigd groen): … alle bestaande `run_smp.py`-gevallen" / "alle stap-1–4-gevallen (14/14)" | `09-…:48-49`; `K06-SMP.md:390-391` | makes `nine-capacity` (9 vCPU) mandatory although 9 is not a spec profile; ADR-070 `docs/DECISIONS.md:370` "1–9 vCPUs" |
| Q8 | BT-1 one clock: "*Weerlegd als* Sys3 sneller of trager loopt dan de PIT, of als een AP een IRQ0 ontvangt" | `06-…:23-25`; ADR-053 `docs/DECISIONS.md:843` "only actually counted IRQ0s advance the monotonic clock" | `smp.rs:2748-2849`; `CLOCK-INVESTIGATION.md:7` |
| Q9 | "Idle-wek: … de wake-latentie wordt gelogd" (no limit) | `06-…:54-55` | `smp.rs:3084-3090` adds a fatal 100 ms limit, see E12 |
| Q10 | "Ticketlock (… FIFO, geen starvation onder TCG)"; big kernel lock and ticket locks under host overcommit are "R02-risico" | `K06-SMP.md:130,424-427`; `09-…:19,86-88`; `docs/DECISIONS.md:2547` | `lock.rs` |
| Q11 | "IQ-4 Een AP initieert nooit quarantaine"; IQ-3 lock-held → `SMP:QUARANTINE:LOCK_HELD:<niveau>:CPU=<n>` | `05-…:23-39` | `smp.rs:1442-1480` |
| Q12 | "Volgorde: eerst het invariant, dan de interface, dan de implementatie, dan de negatieve test." / "'Niet uitgevoerd' is geen groen resultaat" | `specs/k06-steps/README.md:41-52` | process rule |
| Q13 | "De exacte configuratie en hashes worden vóór acceptatieruns bevroren." / "Een mislukte run blijft mislukt bewaard; een timeout telt alleen als geslaagd wanneer juist dat negatieve scenario aantoonbaar wordt verwacht en gecontroleerd." / "Geen veiligheidscontrole of acceptatiegrens wordt verwijderd." | `docs/FULL-TEST-RELEASE.md:74,101-104,131` | process rule |
| Q14 | "The entire 64-by-50 matrix must restart from zero on the corrected source; no earlier passing repetition is reused." | `docs/DECISIONS.md:2569`; `:2654` ("the next full run starts from zero and a recurrence rejects it") | process rule |
| Q15 | "Host watchdog ratios and guest deadline ratios are separate measurements. Mocked harness tests never count as native runs." | `docs/DECISIONS.md:2558` | `run_smp.py:673-675` records `host_watchdog`, not gated |

## 3. Classification of every limit found in code

### 3.1 Class 1 — normative, with source (change requires owner approval / ADR)

| Limit | Value | Code | Normative source |
|---|---|---|---|
| Idle clock precision | 1 % of 999 847 µs, median of 5 | `smp.rs:2789` | N8 |
| Clock load progress | 4 × `CLOCK_SEGMENT` | `smp.rs:2829` | N8 (`docs/DECISIONS.md:2640`) |
| Clock observation bound | 4 × `CLOCK_SPAN` (20 s) | `smp.rs:2750` | Frozen by "the observation deadline … remain fixed" (`docs/DECISIONS.md:2567`); no spec number **[I: class 1 by decision, not by spec]** |
| Shootdown generation | 100 ms | `smp.rs:312`, `:1796-1826`, `:2920-2923`; `m3.rs:3033-3040` | N4, N5 |
| Kill ack | 100 ms | `smp.rs:3686`; `m3.rs:2969`; `smp_lifecycle.rs:11` | N6 |
| Lock stuck | 400 ms | `smp.rs:334`; `lock.rs:239-261` | N2 |
| Fairness | 400 ms / 40 ticks | `smp.rs:4737,4778`; `run_smp.py:371-372` | N7 |
| Scratch | 500 ms | `smp.rs:3785` | N1 |
| Lock-stuck negative | 1 s | `smp.rs:1593` | N3 |
| Scheduler settle | 200 ms | `smp.rs:1432` | N16 |
| Repetitions | 50/50 | `run_smp.py:704-705` | N10 |
| Load / soak duration | 30 s / 600 s | `m3.rs:3274-3279` | N11, N12 |
| Round and count minima | 1024/1000 scratch, 1000 lockpair, 10 000 wake/ICR/isolation/IPC, 200 serial lines, 8 ticks/4 switches, 1000 switches | `smp.rs:3711-3712,3834,3506,3910,4041,363-373,1973-1974`; `m3.rs:3255` | N1, N13, N14, N15, N19; "No workload, 1000-switch minimum, watchdog, quota or safety limit is reduced or increased." (`docs/DECISIONS.md:2581`) |
| IPC host watchdog | 660 s, 2.5× / 264 s rule | `run_smp.py:139` | N17 |
| Default host watchdog | 60 s | `run_smp.py:130` | N18 |
| Timer ratio band | 50 000–75 000 | `run_smp.py:124` | N9 requires an ADR band; the value has no ADR text at this commit (also listed in 3.3) |

### 3.2 Class 2 — engineering values without a normative numeric source (changeable by ADR)

| # | Limit | Value | Code | Failure mode | Note |
|---|---|---|---|---|---|
| E1 | **Sync stress wait** | 200 × `wait_us(10_000)` = 2 s, unnamed literal | `smp.rs:1352-1357` | `SMP:SYNC:TIMEOUT:DONE=…:TOTAL=…` + `serial::fail(b"SMP_SYNC_TIMEOUT")` (`:1360-1370`) | **No normative source.** ADR-070 says only "bounded atomic-plus-spinlock test" (`docs/DECISIONS.md:367`); no spec gives a value. Correctness is checked separately (`SMP_SYNC_LOST_UPDATE`, `SMP_SYNC_EXCLUSION`, `:1372-1377`). |
| E2 | Lock pair | 2 000 ms | `smp.rs:3877` | `SMP:LOCKPAIR:TIMEOUT` | Spec 02 `:46-48`: "geen deadlock en geen marker", no time. |
| E3 | Wake ping-pong | `WAKE_DEADLINE_MS = 20_000` | `smp.rs:3507,3642` | `SMP:WAKE:TIMEOUT:A=…:B=…` | Spec 04 `:50-51`: "geen timeout", no value; no motivation comment. |
| E4 | ICR outer wait | `WAKE_DEADLINE_MS` | `smp.rs:3995` | `SMP:ICR:TIMEOUT` | Spec 05 `:47-49`: "geen timeout". |
| E5 | **ICR ack bound** | `1 << 16` loop iterations (`sti; nop; pause; cli`) | `smp.rs:3955` | `ICR_TIMEOUT` → `SMP:ICR:LOST` / fail `SMP_ICR` (`:4012-4020`) | Iteration count, not time; DG-1 "een tijd en geen iteratietelling" is stated for the lock spin only **[I: same defect class]**. |
| E6 | Isolation | `ISO_DEADLINE_MS = 20_000` | `smp.rs:4042,4301` | `SMP:ISOLATION:TIMEOUT` | Spec 07: rounds only. |
| E7 | IPI ready / per-IPI round | 1 000 ms / 100 ms | `smp.rs:1545,1556` | `SMP:IPI:NOT_READY`, `SMP:IPI:TIMEOUT:APIC=` (quarantine) | ADR-071: no value found. |
| E8 | Park | 100 ms | `smp.rs:1638` | `SMP:PARK:TIMEOUT` | |
| E9 | Shootdown probe | 100 ms | `smp.rs:1902` | `SMP:SHOOTDOWN:PROBE_TIMEOUT` | Distinct from the normative ack deadline. |
| E10 | Preemption phases | 500 / 500 / 200 ms; lock variant 3 × 500 ms | `smp.rs:2339,2345,2369,2418,2425,2439` | `SMP:PREEMPT:TIMEOUT`, `…:STOP_TIMEOUT`, fail `SMP_PREEMPT_LOCK_*` | |
| E11 | Spin window | 100 ms, 100 ms | `smp.rs:2462,2482`; BSP `:2944` | `SMP:SPIN:TIMEOUT`, fail `SMP_SPIN_NOT_GRANTED`, `SMP:BSP_SPIN:TIMEOUT` | |
| E12 | **Idle-wake latency** | fatal at ≥ 100 000 µs | `smp.rs:3084-3090`; `run_smp.py:127,500-507` | fail `SMP_IDLE_WAKE` | Spec 06 `:54-55` only requires the latency to be logged. |
| E13 | Idle phases | 100 ms (TSC) ×3, 200 ms, 100 ms, 500 ms | `smp.rs:2991,3013,3018,3073,3106,3120` | `SMP_IDLE_*` | |
| E14 | BSP preemption | 2 × `idle_until(500)` | `smp.rs:2880,2882` | fail `SMP_BSP_PREEMPT_TIMEOUT` | |
| E15 | Scheduler start/stop | 500 ms ×3, 200 ms; isolation 500/500/1 000 ms | `smp.rs:3426,3442,3449,3476,4315,4329,4364` | `SMP:SCHED:TIMEOUT:APIC=`, `SMP:SCHED:STOP_TIMEOUT` | |
| E16 | Block/wake/kill auxiliary waits | 100, 500, 100 ms | `smp.rs:3644,3666,3680` | `SMP:WAKE:TIMEOUT` etc. | |
| E17 | GS user | 500, 100, 500, 200 ms | `smp.rs:4576,4599,4612,4622` | `SMP:GS:TIMEOUT`, `…:STOP_TIMEOUT` | |
| E18 | AP start | INIT 10 ms, 100 × 10 ms = 1 s | `smp.rs:1193-1209` | `SMP:AP:<n>:TIMEOUT` | |
| E19 | M3 stop | `M3_STOP_MS = 1_000` (TSC) | `smp.rs:4730,4991`; `m3.rs:2790,3340-3341` | `SMP_M3_STOP_TIMEOUT` | |
| E20 | M3 unmap fixture | `CONTROL_MS = 50`, `FIXTURE_MS = 1000` | `m3/smp_unmap.rs:13-14,83-84` | fail `SMP_M3_UNMAP_DEADLINE` | |
| E21 | `wait_us` busy-wait cap | 2 000 000 PAUSE/`inb` iterations; return value ignored by `within_ms` | `smp.rs:751-772,1396-1403` | silent shortening of a wait **[I]** | |
| E22 | Serial THR poll cap | 1 000 000 polls → `exit(0x11)`; `TERMINAL_TRIES = 100_000` | `kernel/src/serial.rs:38-52` | exit 0x11 | |
| E23 | Soak host watchdog | `SOAK_TIMEOUT_S = 660`; `MAX_GUEST_TIMEOUT_S = 660` | `run_smp.py:133`; `tools/run.py:27-30` | host timeout | "reserves 60 seconds for measured boot and teardown" (`tools/run.py:27-29`). No 2.5× rule stated for soak. |
| E24 | Workflow job timeouts | 330 min (case), 90 min (soak shard), 360 min (gate), 30 min (probe) | `.github/workflows/native-smp.yml:291-437,439-506,564-,232-289` | job cancel | |
| E25 | TSC calibration | minimum of ten 10 ms PIT samples | `smp.rs:340-350` | scales every TSC deadline | |
| E26 | Negative-case hold windows | `within_ms(1_000, \|\| false)` ×several; `0..1 << 24` race window | `smp.rs:1567,1600,4582,4590,3930` | fixture duration | |

`within_ms` census **[S]**: 40 call sites in `smp.rs` (grep; 10 × 100, 7 × 1 000,
4 × 200, 14 × 500, 1 × 2 000, 2 × `WAKE_DEADLINE_MS`, 1 × `ISO_DEADLINE_MS`,
1 × `SHOOTDOWN_DEADLINE_MS`). ADR-082 states "At least 36 of the 41 `within_ms`
deadlines and the sync deadline are unnamed literals" (`docs/DECISIONS.md:2689`); the
grep count of 41 includes the function definition at `:1396`, so the exact figure is
40 call sites of which 36 are numeric literals.

### 3.3 Class 3 — normative requirement not (fully) implemented

| # | Requirement | Gap | Evidence |
|---|---|---|---|
| G1 | AC-3: every deadline a named constant with motivation; the gate measures and logs the margin (measured / deadline) | 36 of 40 `within_ms` sites and the sync wait are unnamed literals; margins are recorded only for `bsp_idle_wake`, `remote_kill_ack`, `kernel_lock_wait`, `kernel_lock_deferred` and `host_watchdog` | `specs/k06-steps/09-…:20-21`; `run_smp.py:500-507,524-529,550-564,673-675`; `docs/DECISIONS.md:2689` |
| G2 | BT-5 / AC-2: band and bounds "vooraf in de ADR" | ADR-080 (ratio band) has no body in `docs/DECISIONS.md`; the band exists only in `run_smp.py:124`. Fairness bounds are in ADR-K06-5F (`:2547`) and are covered. | grep of `docs/DECISIONS.md` |
| G3 | 2.5× / 264 s IPC rule | Documentary only; no tool or workflow step checks it | `docs/DECISIONS.md:2654`; grep "264" |
| G4 | Spec 06 §4 "Klok" text | Still states 1 % under load; owner decision of 11 Nov not yet transcribed to the spec branch | `specs/k06-steps/06-…:48-49`; `docs/DECISIONS.md:2648` |
| G5 | ADR-074 text | Still says the stuck bound is "active only in debug builds", contradicted by DG-1 and `lock.rs:239-261` | `docs/DECISIONS.md:116-118`; `specs/k06-steps/03-…:17-21` |
| G6 | Profiles "1, 2, 4 en 8 vCPU's; zowel single- als multi-thread-TCG" for all `native-smp` cases | Base self-test cases are `one` (single), `two-single`, `four` (multi), `eight` (multi), `nine-capacity` (multi): no `two` multi, no `four`/`eight` single. M3, load and migrate cases cover both modes. | `K06-SMP.md:360`; `06-…:71`; `run_smp.py:22-90` **[I: whether the spec demands the full cross product per case is an interpretation]** |
| G7 | Evidence "herhalingstellingen, marges en hashes" in the native report | Margins exist for five quantities only (see G1) | `specs/k06-steps/09-…:57-59` |
| G8 | Kill deadline "gelijk aan de shootdown-deadline" | Same value, different clock: kill at `smp.rs:3686` is PIT-polled (`within_ms`), the shootdown is TSC-bounded (`:1812-1813`) | N5, N6 **[I: value conforms; method diverges]** |

## 4. Explicit statement on the 2 s sync wait

The 2 s wait at `kernel/src/m2/smp.rs:1352-1357` (`(0..200).any(… wait_us(10_000) …)`)
**has no normative source [S]**. It is an unnamed literal. No L0 spec names a time for
the synchronisation stress; ADR-070 requires only a "bounded atomic-plus-spinlock test"
(`docs/DECISIONS.md:367`). The normative content of the check is correctness
(`TOTAL == ITERATIONS × CPUs`, exclusion), which is tested separately at `:1372-1377`.
Two properties make the wait fragile:

- The wait covers more than the counter: after the counter completes each AP prints
  200 serial lines before reporting DONE; at 9-on-4 this takes 0.38–0.51 s
  (`docs/verification/k06-smp-sched/TIMING-INVESTIGATION.md:96`) **[D, local]**.
- The stress uses a raw FIFO `TicketLock` without the stuck bound (`smp.rs:776-796`),
  so a descheduled lock holder or next-in-queue vCPU stalls all CPUs (R02).

Changing the value is not a change to a normative limit, but it is a deadline change in
the sense of "De deadline wordt niet verhoogd zonder ADR" (`specs/k06-steps/09-…:53`)
**[I]**; an accepted ADR is required, and AC-3 requires the replacement to be a named,
motivated constant with a logged margin.

## 5. What the specifications allow and forbid

| Question | Finding | Basis |
|---|---|---|
| **Changing QEMU flags in the harness for acceptance** | Fixed by spec: evidence level "native QEMU q35/OVMF/TCG", profiles 1/2/4/8 vCPUs, both single- and multi-thread TCG; the only flag named is `-accel tcg,thread=multi`. Nothing in the specs mentions host capacity, runner type, vCPU oversubscription, `-icount`, `-rtc` or HPET. Any flag change alters the acceptance configuration, which must be frozen before acceptance runs, and a runner or environment correction "must preserve the specified environment/profile and be followed by a fresh complete … matrix". Not forbidden outright, but requires a recorded decision and a restart from zero. | `K06-SMP.md:12-14,360`; `docs/FULL-TEST-RELEASE.md:74`; `CLOCK-INVESTIGATION.md:41`; current flags `tools/run.py:104` |
| — `-icount` specifically | Cannot serve the mandatory multi-thread profiles: QEMU does not combine `-icount` with multi-thread TCG. **[I: external QEMU knowledge, not verified in this session]** | — |
| — Runner size | A larger runner is an owner decision ("Larger runners are outside this choice"; "a larger runner (paid, Simon's decision)"). | ADR-CI-HOSTED `docs/DECISIONS.md:2206`; `:2633` |
| **Adding diagnostic serial lines** | Not forbidden. Precedents: `SMP:CLOCK:CALIBRATION/SAMPLE` lines, `SMP:FAIR:CPU=…` report lines, the required `SMP:PROC_DEAF:LOCK_FREE:CPUS=4` precondition marker (`docs/DECISIONS.md:2613`). Constraints: (a) the oracle is an exact sequence — an unknown line fails the case (`run_smp.py:581-582`), so the oracle and its host tests must be updated together; (b) no line may contain `FAIL` or `PANIC` (`run_smp.py:17`); (c) DG-2 line atomicity (`specs/k06-steps/03-…:22-25`); (d) one-CPU output of non-SMP profiles must stay byte-identical (`tools/run.py:53`; spec 01 K-3 **[I: K-3 line not re-cited]**); (e) serial output inside a timed window consumes the deadline (section 4). A separate manual diagnostic mode "neither relaxes native limits nor supplies acceptance repetitions" (`docs/DECISIONS.md:2595`). | as cited |
| **Adding negative cases** | Encouraged: "eerst het invariant … dan de negatieve test" (`specs/k06-steps/README.md:41-52`); the matrix grew 64 → 65 → 66 by owner decisions (`docs/DECISIONS.md:2658-2678`). Each new case needs a `CASES` entry, a workflow matrix entry, the aggregate count, a host-oracle test, and 50/50. No row may be removed or marked optional (`docs/FULL-TEST-RELEASE.md:146-148`). | as cited |
| **Changing the measurement method of a limit while keeping its value** | Allowed with a recorded decision; precedents: DG-1 shortcut (PIT → calibrated TSC, value kept; `specs/k06-steps/03-…:20-21`), the 10 Nov clock sampling repair ("the exact 1% drift limit remain fixed", `docs/DECISIONS.md:2567`), one shootdown deadline per generation (`:2549-2550`). Hard boundaries: ticks are never synthesised and the counted-IRQ0 rule stands ("elapsed-counter compensation requires an explicit normative decision and is not implemented", `:2595`; `CLOCK-INVESTIGATION.md:7`); a method that accepts more than the current one is a weakening and needs owner approval — ADR-082 part 3 states this itself for median → minimum ("Weaker than today in one respect, stated plainly"). | as cited |
| **Splitting a check into liveness and performance** | Precedent exists and was owner-approved: the clock was split into precision (idle, 1 %) and load progress (4× deadline, no 1 %) on 11 Nov (`docs/DECISIONS.md:2635-2648`). ADR-K06-5F also separates "Host watchdog ratios and guest deadline ratios". For scratch, "Eis binnen 500 ms" is a spec requirement (N1): making 500 ms non-fatal on any profile changes a normative limit and needs an owner-accepted ADR plus a spec revision. For sync (E1) and lock pair (E2) no normative time exists, so a split needs an ADR but no spec change. ADR-082 part 2 proposes exactly this and is **not in force**. | as cited |
| **Raising a deadline after a failure** | Forbidden without an ADR: "De deadline wordt niet verhoogd zonder ADR" (`09-…:53`); acceptance limits "will not be enlarged in response to a failing test without a separate decision" (`docs/DECISIONS.md:2547`); host watchdog "must not grow in response to a failed repetition" (`run_smp.py:128-130`); `m3/smp_lifecycle.rs:10-11` "never a reason to enlarge the deadline". | as cited |
| **Reusing earlier passing repetitions** | Forbidden: restart from zero on changed source (`docs/DECISIONS.md:2569,2654`). | as cited |

## 6. Hosted margins, run 43187501026 [D]

3301 boots, 3298 passed. Not 50/50: `m3-load-two` 48/50 (`SMP_CLOCK_DRIFT`, medians
1 010 679 and 1 012 547 µs), `nine-capacity` 49/50 (`SMP:SYNC:TIMEOUT:DONE=0:TOTAL=157489`).

| Quantity | Limit | Hosted observation | Margin used |
|---|---|---|---|
| KERNEL wait (fairness) | 400 ms | max 23 ms (`m3-load-eight`); soak 15–19 ms | 6 % |
| Deferred ticks | 40 | max 3 (`m3-load-eight`) | 8 % |
| BSP idle wake | 100 000 µs | max 262 µs (`gs-user`) | 0.3 % |
| Timer ratio | 50 000–75 000 | 62 527–65 138; 0 out of band | centred |
| Load elapsed | ≥ 30 000 / 600 000 ms | 30 000–30 010; 600 000–600 008 | exact by construction |
| Soak host watchdog | 660 s | max 610.4–612.5 s per shard | 92.5–92.8 % |
| `m3-load-eight` host watchdog | 60 s | max 47.3 s | 79 % |
| Other `m3-load-*` host watchdog | 60 s | max 38.3–40.8 s | 64–68 % |
| `m3-ipc-four` / `-single` | 264 s rule (660 s watchdog) | 198–209 s / 124–144 s (`README.md:155`) | 79 % of 264 s |
| `m3-migrate-eight`, `m3-eight`, `m3-four`, `m3-fault-ap2` | 60 s | max 22.8–24.4 s | 38–41 % |
| `m3-kill-four` | 60 s | max 19.5 s | 33 % |

No guest margin is logged for sync, scratch, lock pair, wake, ICR, isolation or any of
the 100–500 ms phase waits (gap G1); their hosted headroom is unknown. Local diagnostic
figures (QEMU 8.2.2, not acceptance; `TIMING-INVESTIGATION.md:62,70,96,104-108`): 8-on-4
scratch max 0.558 s (above 500 ms), lock pair max 0.717 s, wake 0.70 s, ICR 1.30 s,
isolation 1.13 s; 4-on-2 sync tail 5.6 s, scratch 2.65 s, lock pair max 2.71 s;
20 of 134 local boots fail the idle 1 % limit.

## 7. Risks for the next full matrix

See the structured result for the complete list. Ordering by likelihood:

1. Idle clock 1 % median (`smp.rs:2789`) — high. Fails in every matrix so far at roughly
   one per 1 000–3 000 guests; unchanged by owner decision.
2. Sync 2 s on `nine-capacity` and `eight` (`smp.rs:1352-1357`) — high/medium.
3. Scratch 500 ms on 8/9-vCPU profiles (`smp.rs:3785`) — medium (failed once on `eight`).
4. Host watchdogs with thin headroom: soak 612/660 s, `m3-load-eight` 47/60 s, IPC
   209/264 s — medium, driven by runner speed variance.
5. Never-failed limits with unknown hosted headroom: lock pair 2 s, ICR 2^16 iterations,
   100 ms phase waits, kill 100 ms (PIT-polled), shootdown 100 ms with 7–8 targets,
   LOCK_STUCK 400 ms on SERIAL convoys — low to unknown.
6. Implementation risks of the fix itself: exact-sequence oracle, forbidden substrings,
   serial output inside timed windows, new negative cases needing 50/50.

## 8. Open questions

1. Does the project owner accept ADR-082 in whole or in part (it is the only vehicle
   that would authorise relaxing fatal scratch 500 ms on 8/9-vCPU profiles)?
2. Is `nine-capacity` (9 vCPUs, outside the 1/2/4/8 spec profiles) required to meet the
   same timing limits, or only its capacity invariant?
3. Where is the ratio band of BT-5 recorded normatively, given that ADR-080 has no body?
4. Should the 2.5× host-watchdog rule apply to soak and `m3-load-*` (ratios 0.93 and
   0.79), or only to IPC as written?
5. Does the spec require the full single/multi cross product for base `native-smp` cases
   (gap G6)?

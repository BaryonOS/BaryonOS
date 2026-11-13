# L0-specificatie — K06 stap 5f: migratie, concurrente stress en K06-acceptatie (met K07 onder SMP)

**Datum:** 6 november 2026 · **Basis:** de head van stap 08 (5d-2) [nog niet gebouwd]
**Branch/worktree:** `feat/k06-5f-acceptance` in `/home/simon/projects/nieuw-os-k06-5f`
**Spec:** `K06-SMP.md` §6.2b, §7.3, §9 (geheel), §11; `docs/PROGRESS.md` (acceptatieregels);
`docs/verification/K07-ACCEPTANCE.md` (grens "geen SMP"); auditbevindingen F10, F12 en F21.

## 1. Doel
K06 accepteren: *"SMP-scheduling, synchronisatie, interrupts en adresruimtewisseling werken onder
concurrente belasting op het ondersteunde profiel"* (progress.json), op native QEMU q35/OVMF/TCG.
Tegelijk wordt de grens van het K07-bewijs opgeheven: de K07-quota blijven begrensd onder SMP.

## 2. Invarianten (normatief, aanvullend)
- **AC-1 Migratie alleen expliciet** (spec §6.2): bij quarantaine (naar de BSP) en in een
  testfixture. De `on_cpu`-overdracht (F1) en de RUNQ-release→acquire geven de happens-before naar
  de FXRSTOR op de nieuwe CPU (§7.3).
- **AC-2 Eerlijkheid begrensd** (F10): onder de grote kernellock blijft per CPU het maximum van
  `deferred` en de wachttijd op `KERNEL` onder een grens die vooraf in de ADR staat. Overschrijding
  → `SMP:FAIR:FAIL:CPU=<n>`. Ticketlocks onder host-overcommit staan als R02-risico in de ADR.
- **AC-3 Een betrouwbare gate** (F12): elke deadline is een benoemde constante met motivatie. De gate
  meet en logt de marge (gemeten / deadline). Een geval telt alleen als groen bij **50/50** herhalingen.
- **AC-4 Geen 1-vCPU-schijnbewijs:** een geval met `CPUS=1` telt als regressie, niet als SMP-bewijs.
- **AC-5 Eigen stacks** (F21): de schedulerthreads van de fixtures krijgen eigen stacks, of een
  assertie op de koppeling met de stap-4-stacks.

## 3. De §9-matrix (profielen 1/2/4/8 vCPU's, single- en multi-thread-TCG)
**Positief:**
1. **Concurrente belasting, 30 s:** ≥ 2× zoveel CPU-gebonden ring-3-processen als CPU's, plus een
   IPC-ping-pong tussen processen op verschillende CPU's. Per CPU: user-ticks > 0, ≥ 4 wissels,
   geen auditfout.
2. **Isolatie onder SMP** (uit 07, nu met M3-processen).
3. **Unmap-race** (uit 07, nu met M3-processen).
4. **Migratie met canary:** T migreert 1→2→0 met unieke GPR- en x87/SSE-waarden; na elke stap zijn ze
   gelijk en klopt `active_cpus`.
5. **Kill op een remote CPU:** een proces op CPU 3 doden; de K05-cleanupbalans klopt (vrije frames
   vóór = ná) en `active_cpus == 0`.
6. **BSP-interrupts:** Sys3 stijgt tijdens de belasting; de idle-BSP wordt aantoonbaar gewekt.
7. **K07 onder SMP (nieuw ten opzichte van spec §9):** de M4-quotaproef en de qsim-druk uit
   `resource-pressure.yml`, maar met de processen verspreid over 4 vCPU's. Eis: dezelfde
   quota-/`EAGAIN`-/`ResourceExhausted`-uitkomsten, survivorprogressie en exacte cleanup als op 1 CPU,
   en geen kerneluitval.

**Negatief (juiste marker en fail-closed):** `lockdep-order`, `held-across-switch`,
`proc-shootdown-deaf`, `proc-no-shootdown` (controle), `double-run` (nu `DOUBLE_ENQUEUE`),
`resched-deaf`, canary-injectie (corrumpeer bewust een bewaard register; de canary moet afgaan),
`spin-irq-off`, `lock-stuck`, `lock-in-handler`, `quarantine-lock-held`, `gs-user`, `gs-iret-fault`.

**Regressie (ongewijzigd groen):** `milestone-m2`, K05 6/6, K07 (`resource-pressure.yml`),
`native-rtc` 5/5, alle bestaande `run_smp.py`-gevallen, hosttests, Clippy en fmt.

## 4. Herhalingsharnas en soak
- `tools/run_smp.py --repeat 50` (of een kleine wrapper) per geval; het rapport telt flakes. Eén flake
  = niet groen, en de oorzaak wordt onderzocht. De deadline wordt niet verhoogd zonder ADR.
- Een soakprofiel van 10 min op 4 vCPU's (multi-thread-TCG) met de belasting van §3.1 en §3.7.

## 5. Bewijs en voortgang
- `docs/verification/k06-smp-sched/{README.md,native-report.json,SHA256SUMS}`: broncommit, exacte
  commando's, exitcodes, markers per geval, herhalingstellingen, marges en hashes. Volg de opbouw van
  de bestaande `docs/verification/k06-smp-*/`-mappen.
- `docs/verification/K06-ACCEPTANCE.md` in de vorm van `K07-ACCEPTANCE.md`: geaccepteerde eigenschap,
  uitgevoerde controles (runs met ID) en **grenzen** (TCG modelleert geen TLB-timing of speculatie;
  geen fysieke hardware, x2APIC, IOAPIC naar AP's of energie).
- `K07-ACCEPTANCE.md`: een korte aanvulling dat de quota ook onder SMP (4 vCPU's) zijn bewezen, met
  een verwijzing naar het bewijs. K07 blijft `accepted`; het puntental verandert niet.
- `docs/progress.json`: K06 → `accepted` (+2, **41 → 43**) alleen als de native gate op deze
  branch-head aantoonbaar groen is (PROGRESS.md). Daarna `python3 tools/progress.py --write` en `--check`.

## 6. Schrijfbestanden
Testfixtures in `kernel/src/m2/smp.rs` en `kernel/src/m2/m3.rs`, `tools/run_smp.py`,
`tools/tests/test_smp_serial.py`, `.github/workflows/native-smp.yml`,
`docs/verification/k06-smp-sched/`, `docs/verification/K06-ACCEPTANCE.md`,
`docs/verification/K07-ACCEPTANCE.md` (aanvulling), `docs/DECISIONS.md` (eerlijkheidsgrens, deadlines),
`docs/implementation/K06-SMP.md` (status → geïmplementeerd), `docs/PROJECT-STATE.md`,
`docs/progress.json`, `README.md`, `docs/PROGRESS.md`, `docs/assets/progress.svg`.

## 7. Acceptatiecommando's (de runner bouwt niet zelf)
```
export NOS_BUILD_DIR="$PWD/build/k06-native-smp-m3"
python3 -B tools/build.py --kernel-feature native-smp-m3
python3 -B -u tools/run_smp.py --profile native-smp-m3 --repeat 50
python3 tools/progress.py --write && python3 tools/progress.py --check
```
[te verifiëren: of `--repeat` en `native-smp-m3` zo heten na stap 08; pas de namen aan op wat daar is gebouwd.]

## 8. Buiten scope en restrisico
Work-stealing en load balancing (R02), fysieke hardware (H06), menselijke audit (B05). De grote
kernellock begrenst de parallelle syscall-doorvoer: dat is acceptabel voor K06 (correctheid), maar het
blijft een R02-risico. Een AI-review is geen onafhankelijke menselijke audit.

# L0-specificatie — K06 stap 5d-2: M3-ring-3-processen op AP's

**Datum:** 6 november 2026 · **Basis:** de head van stap 07 (5e) [nog niet gebouwd]
**Branch/worktree:** `feat/k06-5d2-ring3-smp` in `/home/simon/projects/nieuw-os-k06-5d2`
**Spec:** `K06-SMP.md` §2 (INV-1 t/m INV-7), §3.1 stap 5, §3.2, §6.1–6.4, §7.3, §7.4, §8 (5d), §9.1;
ADR-045 (FXSAVE), -053, -060, -074, -075, -076 en de ADR's van stap 02–07.

## 0. Voorwaarden (alle eerder samengevoegd of gestapeld, en native groen)
F8 (swapgs), 01 (KERNEL-lock en scratch per CPU), 02 (lockdep met context en paren), 03 (stuck en
serial), 04 (block/wake/kill), 05 (ICR en quarantaine), 06 (BSP-timer en idle), 07 (procesbrede
shootdown). Ontbreekt er één, dan **start deze stap niet**: noteer dat als blocker in PROJECT-STATE.

## 1. Probleem (waarom nu)
`kernel/src/main.rs:29` sluit M3 uit van `native-smp`. Na de zelftests worden alle AP's geparkeerd.
Geen enkele AP doet productiewerk. K06 vraagt dat processen echt op meerdere CPU's draaien.

## 2. Invarianten (normatief)
- **R3-1 Alleen het bekende pad.** M3 draait in een nieuw profiel `native-smp-m3`. `native-smp` blijft
  het bringupprofiel; de `compile_error!` op `main.rs:29` wordt alleen voor `native-smp-m3` opgeheven.
  Alle andere profielen blijven één CPU.
- **R3-2 Eén thread per proces.** Zolang M3 geen threads heeft, is het M3-proces precies één
  schedulerthread met `home_cpu` = de online CPU met de kortste queue bij spawn (laagste index bij
  gelijkspel). Geen automatische migratie (spec §6.2).
- **R3-3 FPU in de wissel** (ADR-045, spec §6.3 stap 3 en 8): FXSAVE na de laatste Rust-code van de
  oude context, FXRSTOR direct vóór de GPR-pops/IRETQ. De bestaande code staat in
  `kernel/src/m2/arch.rs:658/674` en `kernel/src/m2/m3/fpu.rs`; die wordt hergebruikt, niet herschreven.
- **R3-4 Entry onder de lock.** Elke M3-entry (syscall, fault, IRQ0-pad met procesimpact) neemt
  `KERNEL` (01). Een AP gebruikt `swapgs` volgens ADR-076, en de TSS.RSP0 van die CPU wijst naar de
  kernelstack van de lopende thread (spec §6.3 stap 6).
- **R3-5 IPC over CPU's.** De ADR-053-wait gebruikt het block/wake-protocol uit 04. Een antwoord aan
  een thread op een andere CPU stuurt 0xe8. Een kopie tussen twee adresruimten neemt
  `lock_pair(ASPACE[a], ASPACE[b])` (02).
- **R3-6 Usercopy onder `ASPACE`** (INV-6, spec §7.4): validatie en kopie gebeuren onder de lock van
  de doelruimte.
- **R3-7 Quota ongewijzigd.** Alle K07-quota (handles, delegatiediepte, IPC-queue, heap, CPU-deadline)
  gelden per proces, niet per CPU. *Weerlegd als* een proces op twee CPU's samen meer krijgt dan zijn
  quotum.

## 3. Interface
- Profiel `native-smp-m3` in `kernel/Cargo.toml` en `tools/build.py`; runner `tools/run_smp.py` krijgt
  M3-gevallen, of een nieuwe `tools/run_smp_m3.py` [kies wat de bestaande runnerconventie volgt].
- `m3.rs`: `State.current` wordt per CPU (`this_cpu().current`); de M3-scheduler legt threads in
  `RUNQ[home_cpu]`. [te verifiëren: alle plekken die `state.current` lezen; `graphify_callers` op
  `m3_event` en de schedulerfuncties.]
- AP-opstart: na `SMP_SCHED_GO` kiest een AP werk uit zijn eigen queue in plaats van te parkeren.

## 4. Native fixture en tests
- **K05 op SMP:** de supervisorproef (K05, 6/6) in `native-smp-m3` op 4 vCPU's, met diensten verspreid
  over de CPU's. Eis: dezelfde uitkomsten en cleanupbalans als op 1 CPU.
- **User-werk op elke CPU:** `SMP:SCHED:CPUS=<n>` met per CPU user-ticks > 0 en ≥ 4 wissels.
- **IPC-ping-pong** tussen twee processen op verschillende CPU's, 10.000 rondes, zonder verloren
  antwoord. Marker `SMP:IPC:PASS`.
- **FPU-canary:** twee processen op dezelfde CPU met verschillende x87/SSE-waarden; na 1000 wissels
  zijn de waarden intact.
- **Ring-3-GS** (F8 N1–N3): opnieuw in `native-smp-m3`, nu met echte M3-processen.
- **Negatief:** een appfault op AP2 stopt alleen dat proces; de andere processen op alle CPU's lopen
  door (K03 onder SMP).

## 5. Schrijfbestanden
`kernel/src/main.rs` (alleen de cfg voor het nieuwe profiel), `kernel/src/m2/m3.rs`,
`kernel/src/m2/smp.rs`, `kernel/src/m2/arch.rs` (alleen als TSS/RSP0 daar staat),
`kernel/Cargo.toml`, `tools/build.py`, `tools/run_smp.py` (of een nieuwe runner),
`tools/tests/test_smp_serial.py`, `.github/workflows/native-smp.yml`, `docs/DECISIONS.md` (nieuwe ADR:
M3 op SMP), `docs/implementation/K06-SMP.md` (§8 bijwerken), `docs/PROJECT-STATE.md`, voortgangsbestanden.

## 6. Acceptatie
- `tools/gate.sh` exit 0; fmt; clippy; de serial-oracle.
- Native `native-smp-m3` op 2/4/8 vCPU's (single- en multi-thread-TCG) en `native-smp` (alle bestaande
  gevallen).
- **Eén-CPU-regressie ongewijzigd:** `milestone-m2`, K05, K07 (`resource-pressure.yml`), `native-rtc`,
  `run_ipc_wait.py`, `run_fpu.py`.
- K06 blijft open (41/100): de §9-matrix en de stress volgen in 09.

## 7. Buiten scope
Migratie en load balancing (09), meerdere threads per proces, device-IRQ's naar AP's, de GUI- en
M6-profielen op SMP.

# L0-specificatie — K06 stap 5b-2: KERNEL-lock om de M3-entry en scratchslot per CPU (ADR-075)

**Datum:** 6 november 2026 · **Basis:** `7ffdacfa` (`fix/k06-f8-swapgs`, PR #231, top van de K06-stack)
**Branch/worktree:** `feat/k06-5b2-kernel-lock` in `/home/simon/projects/nieuw-os-k06-5b2`
**Spec:** `docs/implementation/K06-SMP.md` §2 (INV-2, INV-6), §4.2, §5 (herziening ADR-024), §8;
ADR-024, -053, -060, -074; auditbevinding F20.

## 0. Vastgestelde feiten (op `7ffdacfa`)
- `kernel/src/m2/paging.rs:14` heeft één globaal `static mut SCRATCH_ENTRY`. Het commentaar op regel 809
  noemt het een *"single-CPU scratch-entry address"*. Het wordt gebruikt in de paden op regel 774–860
  (frame nullen en kopiëren via een tijdelijke mapping).
- `kernel/src/m2/m3.rs:1564` `m3_event` is het enige Rust-entrypunt voor M3-events. Het pakt
  `static mut STATE` (`m3.rs:458`) met de SAFETY-aanname *"No scheduler can run concurrently"*.
- `kernel/src/main.rs:29` sluit M3 uit van `native-smp` met `compile_error!`. Dat blijft in deze stap zo.
- [te verifiëren] Of er buiten `m3_event` nog paden zijn die `STATE`, de frame-allocator of paging
  muteren (IRQ0-stub `m3.rs:1159-1203` telt alleen ruwe tellers; controleer de M4/M6-diensten).

## 1. Probleem (waarom nu)
Ring 3 op AP's (stap 08) betekent dat twee CPU's tegelijk een syscall of fault kunnen afhandelen.
Zonder één lock muteren ze tegelijk `STATE`, de frame-allocator en de mappingarena (INV-2). Twee
CPU's die tegelijk een frame via dezelfde `SCRATCH_ENTRY` nullen of kopiëren, lezen of schrijven
elkaars frame: dat is een datalek of corruptie tussen processen. Deze stap maakt de kernel
SMP-veilig **zonder** het single-CPU-gedrag te veranderen.

## 2. Invarianten (normatief)
- **K-1 Eén kernelsectie.** Elke mutatie van `STATE`, frame-allocator, paging, IPC-wachtrijen,
  quota en supervisor gebeurt met `KERNEL` (niveau 10) vastgehouden. `m3_event` neemt `KERNEL` als
  eerste actie na de entry-stub en geeft hem pas vrij vlak vóór de terugkeer naar de stub.
  *Weerlegd als* een debugcheck `kernel_held()` faalt in een muterende functie.
- **K-2 Scratch per CPU.** CPU *i* gebruikt alleen VA `SCRATCH_BASE + i·4096` en zijn eigen PTE-adres.
  Het adres bestaat alleen via `this_cpu().scratch_*()`. Er is geen globale scratch-entry meer.
  Invalidatie blijft lokaal (`invlpg`). *Weerlegd als* een CPU een andere scratch-VA aanraakt
  (assert in de helper), of als de fixture in §4 een vreemd patroon leest.
- **K-3 Geen verandering op één CPU.** Op één CPU neemt de lock altijd direct (geen wachter). De
  volgorde van markers, quota en geheugengebruik van alle bestaande profielen blijft byte-gelijk.
- **K-4 Bestaande single-CPU-SAFETY-notities** (ADR-060-buffer, ADR-053-wait, `STATE`) worden
  herschreven van "single-CPU/IF0" naar "houdt `KERNEL`". Geen SAFETY-tekst noemt nog "single-CPU"
  als grond.
- **K-5 Venster op de BSP pas in 5c-2.** De BSP spint in deze stap nog met IF=0 (ADR-074 aanvulling
  5b). Op één CPU wordt er nooit gespind.

## 3. Interface (minimaal)
- `lock.rs`: één statische `KERNEL: IrqLock<()>` op niveau KERNEL, en `fn kernel_held() -> bool`
  (lockdepbit van deze CPU). Een guard-type zonder `Send` (bestaat al uit 5a).
- `m3_event`: `let _k = KERNEL.lock_irqsave();` als eerste regel. [te verifiëren: of de
  contextwissel in `m3_event` zelf gebeurt; zo ja, dan moet de guard vóór de wissel vallen (INV-7),
  zie spec §6.3 stap 2.]
- `paging.rs`: `SCRATCH_ENTRY` → `[u64; MAX_CPUS]`, ingevuld bij de setup voor iedere CPU-index.
  `scratch_slot()` geeft (VA, PTE-ptr) van `this_cpu().index`. De BSP is index 0 (GS-anker uit 5a).
- ADR-075: de herziening van ADR-024 volgens spec §5 (punt 1–7), plus de K-1-regel.

## 4. Native fixture (`native-smp`, na `SMP:SCHED:PASS`, vóór `SMP:PARKED`)
Elke AP en de BSP nullen en vullen tegelijk een **eigen** frame via hun eigen scratchslot met een
patroon `0xA5 ^ cpu_index`. Dat doen ze 1024 keer, en ze controleren na elke vulling het hele frame.
Daarna neemt elke AP 1000× `KERNEL` en verhoogt een gedeelde teller (niet atomair, alleen
onder de lock). Eis binnen 500 ms: alle patronen kloppen, teller = 1000 × (online+1), lockdep leeg.
Marker `SMP:SCRATCH:PASS:CPUS=<n>`.

## 5. Negatieve test
`test-smp-shared-scratch` (4 vCPU's): alle CPU's gebruiken scratchslot 0 (de oude toestand). De
fixture moet een vreemd patroon zien → `SMP:SCRATCH:CROSS:CPU=<n>` en exit 0x10. Dat bewijst dat de
positieve test een echt gevaar meet. De oracle weigert een PASS in dit profiel en de marker in een
normaal profiel.

## 6. Schrijfbestanden
`kernel/src/m2/lock.rs`, `kernel/src/m2/paging.rs`, `kernel/src/m2/m3.rs` (lock en SAFETY-teksten),
`kernel/src/m2/smp.rs` (fixture), `kernel/Cargo.toml`, `tools/build.py`, `tools/run_smp.py`,
`tools/tests/test_smp_serial.py`, `.github/workflows/native-smp.yml`, `docs/DECISIONS.md` (ADR-075),
`docs/implementation/K06-SMP.md` (§8), `docs/PROJECT-STATE.md`, voortgangsbestanden.

## 7. Acceptatie
- `tools/gate.sh` exit 0; `cargo fmt --check`; clippy `-D warnings` op `smp-lock`; `test_smp_serial.py`.
- Kernelbuilds `native-smp`, `test-smp-shared-scratch`, `milestone-m2` en alle M3-/M4-/M6-profielen.
- **Eén-CPU-regressie, verplicht native:** `milestone-m2`, K05 (`tools/run_supervisor.py`, 6/6), K07
  (`resource-pressure.yml`: M4-quota en qsim), `native-rtc` 5/5, `run_ipc_wait.py`, `run_fpu.py`,
  `run_memory.py`. Uitvoer en markers ongewijzigd ten opzichte van `7ffdacfa`.
- Native `native-smp` (alle gevallen) en `test-smp-shared-scratch` via `run_smp.py` of `native-smp.yml`.
- Voortgang: K06 blijft open (41/100).

## 8. Buiten scope
M3 in `native-smp` (blijft uitgesloten tot 08), het venster op de BSP (06), fijnmazige locks per
subsysteem (ADR-075 noemt het herzieningscriterium), PCID.

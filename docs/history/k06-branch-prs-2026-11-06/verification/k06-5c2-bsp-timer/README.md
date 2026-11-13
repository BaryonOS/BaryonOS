# K06 stap 5c-2: BSP-timer, PIT-klok, idle en reschedule-vlag op de BSP (ADR-080)

**Datum:** 6 november 2026. **Branch:** `feat/k06-5c2-bsp-timer`, gestapeld op
`feat/k06-5b2-kernel-lock` (`068fd857`, met F3/F4, F16/F17 en F19/F15). Taakspec:
`docs/implementation/k06-steps/06-5c-2-bsp-timer-idle.md`. **Broncommit:** `5020b904`.
**Bron:** vingerafdruk (v3) `412e6740…` (volledig in `native-report.json`). **Bewijsniveau:**
native QEMU 10.2.1/OVMF/TCG (machine `pc-q35-8.2`) op de WSL-host. Geen CI, geen fysieke hardware,
review door AI. **Score:** 41 → 41/100. K06 blijft open.

## Vastgesteld vóór de bouw (spec §0)
`native-smp` sluit M3 uit (`compile_error` in `kernel/src/main.rs`). De PIC-programmering in
`m3.rs` en Sys3 bestaan in dit profiel dus niet, en de BSP draaide tot nu toe volledig met IF=0.
De PIT-IRQ0-klok is daarom in `native-smp` zelf gebouwd (BT-1); het M3-pad is ongewijzigd.

## Native SMP-matrix (`native-report.json`)
28/28 gevallen over 23 profielen, allemaal op dezelfde bron, per profiel:
`python3 -B tools/build.py --kernel-feature <p> && python3 -B -u tools/run_smp.py --profile <p>`.
- `native-smp` 6/6 (1, 2 single-thread TCG, 4, 8 en 9 vCPU's, dubbele MADT). Nieuw in de reeks:
  `SMP:TIMER:RATIO:<r>` na `SMP:ONLINE`, en na `SMP:SPIN:PASS`: `SMP:CLOCK:PASS`,
  `SMP:BSP_PREEMPT:PASS`, `SMP:BSP_SPIN:PASS`, `SMP:IDLE:WAKE_US=<n>` en `SMP:IDLE:PASS`. Met
  één CPU staan BSP_SPIN en IDLE op `SKIPPED:CPUS=1` (geen AP, dus niets te bewijzen en geen PASS).
  - **Ratio (BT-5):** gemeten 62 559–62 871 LAPIC-ticks (deler 16) per PIT-tick over alle runs;
    verwacht 62 500 bij QEMU's 1 GHz APIC-timerklok. De band [50 000, 75 000] stond in ADR-080
    vóór de eerste meting.
  - **Klok (BT-1):** alle AP's spinnen, de BSP idle't door 500 IRQ0-ticks; de duur op de
    PIT-gekalibreerde TSC moet binnen 1% van 499 923 µs liggen en de klok mag nooit teruggaan.
  - **BSP-preëmptie (BT-2):** twee BSP-threads met r12–r15-canaries en het home-context in
    rotatie; ≥8 ticks, ≥4 wissels, beide threads lopen tussen twee steekproeven door, alles
    bevroren na het maskeren, geen corrupte context.
  - **BSP als shootdowndoel (F6):** AP1 houdt `KERNEL`, wacht tot de lockdep van de BSP KERNEL
    toont (de BSP spint), stuurt 0xe0 naar alleen de BSP; ack binnen 100 ms via het BSP-venster,
    geen quarantaine.
  - **Idle-wek en uitgestelde resched (BT-3/BT-4):** wake-latentie van AP1's 0xe8 tot de handler
    op de BSP 39–248 µs over alle runs met AP's (grens 100 ms). Daarna wacht een BSP-thread met
    gemaskeerde timer op een lock van AP1; AP1's 0xe8 wordt uitgesteld (`deferred` stijgt) en
    pas de release wisselt (precies 2 wissels, `need_resched` leeg).
- `test-smp-bsp-spin-irq-off` (4 vCPU's, controle): het BSP-venster blijft dicht. Reeks t/m
  `SMP:BSP_PREEMPT:PASS`, dan `SMP:SPIN:SHOOTDOWN_TIMEOUT:APIC=0`, exit 0x10, geen hang.
- Alle 22 bestaande profielen geven hun marker of quarantainereeks met de nieuwe regels waar
  die fasen al draaiden (`lock-in-handler` stopt nog steeds in de schedulerfase: de BSP-0xe8 van
  de idle-fase valt buiten die testvoorwaarde).

## Rode controle (lokaal, niet gecommit)
Mutant zonder `smp::resched_on_release()` in `IrqGuard::drop`, `native-smp` 4 vCPU's: de reeks
loopt t/m `SMP:IDLE:WAKE_US`, daarna `SMP:IDLE:RESCHED_LOST` en `M1:FAIL:SMP_IDLE_RESCHED_LOST`.
De idle-fixture faalt dus zonder BT-4 aan de release-kant.

## Een-CPU-regressie en host (op dezelfde bron)
- `tools/test_milestones.py --through m3`: M2 8/8 groepen, M3 22/22 groepen (93 s). M3 bouwt
  `lock.rs` mee, met de één-CPU-stub (`irq_ready() = false`, `resched_on_release()` leeg).
- `tools/build.py --kernel-feature native-rtc` + `tools/run_rtc.py --case all`: 5/5 (millennium,
  leap, current, y2038, future); CMOS-eigendom ongewijzigd. `tools/test_supervisor.py` (K05): 6/6
  stappen.
- `cargo test -p smp-lock` 24 + 1 doctest, `cargo test -p boot-protocol` 28; `cargo clippy --locked
  -p smp-lock` en `-p boot-protocol --all-targets -- -D warnings` en `cargo fmt --all -- --check`
  schoon. `tools/tests/test_smp_serial.py` 16/16 (nieuw: ratio buiten de band of ontbrekend,
  ontbrekende klok/BSP-fasen, `SKIPPED` als `PASS`, wake ≥ 100 ms en de BSP-venstercontrole die
  als normale run of met `APIC=1` slaagt worden geweigerd), `test_gs_discipline.py` 8/8 (de
  IRQ0-stub is een gereviewde stub zonder GS die via de spurious-`iretq` terugkeert),
  `test_acpi_serial.py` 3/3, `test_rtc.py` 10/10.
- `tools/gate.sh` exit 0 (36 s): cargo check, cargo test 518 geslaagd en 1 genegeerd. Het script
  `tools/qemu-smoke-test.sh` bestaat niet, dus die stap is niet uitgevoerd.

## Niet uitgevoerd en open
- CI op de gestapelde PR (native-smp draait niet op een PR met een featurebranch als base),
  fysieke hardware, menselijke audit, M4 (pad-bug, #235).
- Kernel-clippy is geen CI-gate en faalt al op de basis (`serial::hex`, `paging.rs`,
  `pair_work`, `main.rs`); deze stap voegt geen meldingen toe.
- De gemeten klokafwijking wordt bij een PASS niet gelogd (alleen bij een afwijking > 1%).
- TSC als constant en gelijk op alle CPU's aangenomen (QEMU/KVM); hardware vraagt een
  invariant-TSC-controle. `ADOPTED` (ADR-079) wordt nog niet gedraaid: de BSP heeft nog geen
  run queue (5d-2 of 5f).

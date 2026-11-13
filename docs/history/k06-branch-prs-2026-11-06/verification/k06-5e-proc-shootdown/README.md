# K06 stap 5e: procesbrede TLB-shootdown op `active_cpus` (aanvulling ADR-072/-075)

**Datum:** 6 november 2026. **Branch:** `feat/k06-5e-proc-shootdown`, gestapeld op
`feat/k06-5c2-bsp-timer` (`0d197dc1`). Taakspec: `docs/implementation/k06-steps/07-5e-process-shootdown.md`.
**Broncommit:** `730ce80a`. **Bron:** vingerafdruk (v3) `fdc7ab04…` (volledig in `native-report.json`).
**Bewijsniveau:** native QEMU 10.2.1/OVMF/TCG op de WSL-host. Geen CI, geen fysieke hardware,
review door AI. **Score:** 41 → 41/100. K06 blijft open.

## Native SMP-matrix (`native-report.json`)
30/30 gevallen over 25 profielen, allemaal op dezelfde bron, per profiel:
`python3 -B tools/build.py --kernel-feature <p> && python3 -B -u tools/run_smp.py --profile <p>`.
- `native-smp` 6/6 (1, 2 single-thread TCG, 4, 8 en 9 vCPU's, dubbele MADT). Nieuw na
  `SMP:ICR`: `SMP:ISOLATION:PASS` en `SMP:UNMAP_RACE:PASS` (bij één CPU `SKIPPED:CPUS=1`).
  - **Isolatie:** thread 0 (ruimte A) op AP1 en thread 1 (ruimte B) op AP2 (bij één AP beide op
    AP1, met CR3-wissels) schrijven en lezen elk 10 000× hun eigen patroon op `0x4000_0000`.
    Twee echte ruimten met een ander frame; nooit het patroon van de ander.
  - **Unmap-race:** de lezer in A leest de pagina in een lus. De BSP (KERNEL → ASPACE[A] →
    SHOOTDOWN) wist de PTE, doet de ronde op `ACTIVE[A]`, geeft het frame vrij en hergebruikt het
    (first-fit, hetzelfde frame) met `0xb2`. De lezer krijgt een gecontroleerde #PF
    (`nos_smp_race_read` geeft via een fixup `RACE_FAULT` terug) en ziet nooit `0xb2`.
  - **Gerichte doelset:** de handlerteller per AP stijgt precies voor de CPU's in de doelset. Met
    ≥ 2 AP's moet de doelset exact AP1 zijn; AP2 (ruimte B) en de idle AP's krijgen niets.
  - **Exit (PS-5):** A en B worden verwijderd met `ACTIVE` = 0 en een ronde over de hele user-GiB.
    Daarbij wordt gecontroleerd dat geen levende ruimte een vrij frame noemt; daarna volgt de
    volledige kernel-paging-audit.
- `test-smp-proc-shootdown-deaf` (4 vCPU's): AP1 negeert de rondes voor A. Reeks t/m
  `SMP:ISOLATION:PASS`, dan `SMP:SHOOTDOWN:TIMEOUT:APIC=1`, `SMP:SHOOTDOWN:FRAME_QUARANTINED`,
  `SMP:QUARANTINED:APS=3`, `SMP:ONLINE:1/4` en de boot loopt door tot `M2:PASS`/`M1:PASS`.
- `test-smp-proc-no-shootdown` (4 vCPU's, controle): zonder ronde leest de lezer het hergebruikte
  frame, `SMP:SHOOTDOWN:STALE_READ`, exit 0x10. De race meet dus een echt gevaar.
- Alle 23 bestaande profielen geven hun marker of quarantainereeks, nu met de nieuwe ronde
  (ook `ap-shootdown-deaf`, `shootdown-skipped-control`, `spin-irq-off-control` en de ronde van
  AP1 naar de BSP in `bsp-spin-irq-off-control`).

## Host en een-CPU-regressie (op dezelfde bron)
- `cargo test -p smp-lock`: 26 + 1 doctest. Nieuw: `switch_space_sets_before_and_clears_after_the_load`
  en `model_active_snapshot_covers_every_stale_cpu`, het hostmodel van de `ACTIVE`-supersetregel
  met een nep-CR3. **Rode controle (lokaal, niet gecommit):** met de bit pas ná de CR3-load falen
  beide tests.
- `tools/tests/test_smp_serial.py` + `test_gs_discipline.py`: 25/25 (nieuw: de procesfasen zijn
  verplicht, `SKIPPED` als `PASS` en een stale read in een normale run worden geweigerd, en beide
  controles slagen niet stil).
- `tools/test_milestones.py --through m3`: M2 8/8, M3 22/22 groepen (77 s). M3 gebruikt de
  gesplitste `unmap` en `destroy(&mut self)`.
- `cargo fmt --all -- --check` en `cargo clippy -p smp-lock --all-targets -- -D warnings` schoon.
  Kernel-clippy (`native-smp`): exact dezelfde meldingen als op de basis.
- `tools/gate.sh` exit 0 (34 s): cargo check, cargo test 520 geslaagd en 1 genegeerd. Het script
  `tools/qemu-smoke-test.sh` bestaat niet, dus die stap is niet uitgevoerd.

## Niet uitgevoerd en open
- CI op de gestapelde PR, fysieke hardware, menselijke audit, `native-rtc`/K05/K07 (niet geraakt
  door deze diff), M4 (pad-bug, #235).
- Het M3-pad en de engine-`check_invariants` zijn niet gewijzigd: M3 draait niet onder
  `native-smp`. Er is geen rechtenverlagings-API (er is geen aanroeper). FRAMES is nog geen lock.
- De gecontroleerde #PF hervat via de beoordeelde spurious-`iretq`, alleen voor CPL0 en alleen voor
  `nos_smp_race_read`. Het is fixturecode, geen algemene exception-fixup.

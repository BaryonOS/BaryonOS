# K06 F19/F15: ICR-lock per CPU en quarantainebeleid bij gehouden locks (ADR-079)

**Datum:** 6 november 2026. **Branch:** `fix/k06-f19-f15-icr-quarantine`, gestapeld op
`feat/k06-5b2-kernel-lock` (`81b648ba`, met F16/F17 uit #239). Taakspec:
`docs/implementation/k06-steps/05-F19-F15-icr-quarantine.md`. **Bron:** vingerafdruk (v3)
`dc44d588…` (volledig in `native-report.json`). **Bewijsniveau:** native QEMU 10.2.1/OVMF/TCG
op de WSL-host. Geen CI, geen fysieke hardware, review door AI. **Score:** 41 → 41/100. K06 blijft open.

## Native SMP-matrix (`native-report.json`)
27/27 gevallen over 22 profielen, allemaal op dezelfde bron, per profiel:
`python3 -B tools/build.py --kernel-feature <p> && python3 -B -u tools/run_smp.py --profile <p>`.
- `native-smp` 6/6: nieuwe regel `SMP:ICR:PASS:ROUNDS=10000:CPUS=4/8/8` (na `SMP:LOCKPAIR:PASS`).
  Met 1 of 2 CPU's staat er `SMP:ICR:SKIPPED:CPUS=1/2`: er zijn geen twee AP's, dus niets te
  bewijzen en geen PASS. AP1 en AP2 sturen elkaar elk 10.000 0xe8 vanuit threadcontext. Elke
  send wacht op de ack (de teller van de partner) vóór de volgende, zodat een IRR-bit nooit twee
  IPI's samenvoegt. Hun periodieke timer (0xd0) loopt intussen door, en elke 0xd0- en 0xe8-entry
  stuurt vanuit de handler een self-IPI 0xf0. Eis: ontvangen = verzonden door de partner, per
  richting precies 10.000, geen timeout en geen fout van een handler-send.
- `test-smp-icr-race` (4 vCPU's, controle): zonder ICR-lock. AP1 wacht tussen ICR-hoog en
  ICR-laag tot er een 0xd0 is binnengekomen. De handler schrijft zijn eigen bestemming (AP1 zelf),
  dus de 0xe8 van de thread komt niet bij AP2 aan. De oracle ziet `SMP:ICR:LOST`: AP1's eigen 0xe8
  smelt in AP1's IRR samen met die van AP2. `SMP:ICR:MISROUTED` (een overschot op AP1) wordt ook
  geaccepteerd. Exit 0x10; een PASS faalt met `SMP_ICR_RACE_MISSED`. Los gedraaid: 3/3 LOST.
- `test-smp-quarantine-lock-held` (4 vCPU's): AP2 neemt `KERNEL` en wordt doof (`cli; hlt`).
  `SMP:IPI:TIMEOUT:APIC=2` → `SMP:QUARANTINE:LOCK_HELD:10:CPU=2` → exit 0x10, geen hang en geen
  `SMP:QUARANTINED`.
- Bestaande dove-AP-profielen (`ap-ipi-deaf`, `ap-shootdown-deaf`, `ap-timer-dead`,
  `spin-irq-off`, `resched-deaf`) geven ongewijzigd hun quarantainereeks: die AP's houden geen
  KERNEL of RUNQ vast. De overige profielen geven hun marker, met de ICR-regel waar die fase al
  draaide.

## Een-CPU-regressie (op dezelfde bron)
- `tools/test_milestones.py --through m3`: M2 8/8 groepen, M3 22/22 groepen. M3 bouwt `lock.rs`
  mee, inclusief het nieuwe eigenaarsveld (één-CPU-stub `cpu_index() = 0`).
- `tools/build.py --kernel-feature native-rtc` + `tools/run_rtc.py --case all` geslaagd,
  `tools/test_supervisor.py` (K05, 6 stappen), `tools/test_resource_pressure.py` (K07 en qsim
  geaccepteerd).

## Host en statisch
- `cargo test -p smp-lock` 24 + 1 doctest, `cargo test -p boot-protocol` 28,
  `cargo clippy --locked -p smp-lock --all-targets -- -D warnings` en `cargo fmt --all -- --check`
  schoon. `tools/tests/test_smp_serial.py` 15/15 (nieuw: ICR-regel verplicht met 3+ CPU's,
  SKIPPED nooit als PASS, de race-controle accepteert alleen LOST/MISROUTED, en een dove AP met
  KERNEL mag niet als gewone quarantaine slagen), `test_gs_discipline.py` 8/8,
  `test_acpi_serial.py` 3/3.
- `tools/gate.sh` exit 0 (35 s): cargo check, cargo test 518 geslaagd en 1 genegeerd. Het script
  `tools/qemu-smoke-test.sh` bestaat niet, dus die stap is niet uitgevoerd.

## Bevinding tijdens de bouw
Een eerste race-window met een one-shot LAPIC-timer met beginwaarde 1 liet QEMU/TCG vastlopen:
ook de BSP schreef binnen 60 s niets meer. Halveren gaf: zonder window groen, met window en de
natuurlijke periodieke tick wel een race en geen hang. Het window wacht nu op één echte tick
(begrensd op 2^24 iteraties). De oorzaak in QEMU is niet verder onderzocht.

## Niet uitgevoerd en open
- CI op de gestapelde PR, fysieke hardware, menselijke audit.
- Een AP die als *wachter* in een ticketlock sterft, houdt zijn ticket vast. IQ-3 controleert
  alleen eigenaren, dus die vastloper wordt niet gezien (ADR-079, open punt).
- `ADOPTED` wordt nog niet gedraaid (BSP-runqueue volgt in 5c-2). Bij een crash geldt de K05-route
  pas voor ring-3-threads op AP's (5d-2).

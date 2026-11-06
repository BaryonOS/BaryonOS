# K06 F3/F4: lockdep met handlercontext en lockparen (ADR-077)

**Datum:** 6 november 2026. **Branch:** `fix/k06-f3-f4-lockdep`, gestapeld op
`feat/k06-5b2-kernel-lock` (`de7a07b5`, PR #234). **Bron:** `394bba37`, vingerafdruk (v3) `5ea3759b…`.
**Bewijsniveau:** native QEMU 10.2.1/OVMF/TCG op de WSL-host. Geen CI, geen fysieke hardware,
review door AI. **Score:** 41 → 41/100. K06 blijft open.

## Native SMP-matrix (`native-report.json`)
24/24 gevallen over 19 profielen, allemaal op dezelfde bron, per profiel:
`python3 -B tools/build.py --kernel-feature <p> && python3 -B -u tools/run_smp.py --profile <p>`.
- `native-smp` 6/6: `SMP:LOCKPAIR:PASS:CPUS=1/2/4/8/8`. Elke online CPU neemt 1000 keer
  `lock_pair` op twee ASPACE-locks (sleutels 1 en 2). Oneven CPU-indexen geven de argumenten
  omgekeerd, dus vanaf 3 CPU's vragen minstens twee AP's in tegengestelde volgorde. Geen deadlock,
  geen marker, beide tellers tellen elke ronde en lockdep is daarna overal leeg.
- `test-smp-lock-in-handler` (2 vCPU's): de 0xe8-handler neemt KERNEL →
  `SMP:LOCKDEP:IN_HANDLER:10`, exit 0x10. Dat bewijst ook dat de stubs `irq_depth` echt ophogen.
- `test-smp-lockpair-order` (2 vCPU's): twee ASPACE-locks los en dalend genomen →
  `SMP:LOCKDEP:ORDER:20->20`, exit 0x10.
- Alle 17 bestaande profielen geven ongewijzigd hun marker (de normale reeks heeft één regel
  `SMP:LOCKPAIR:PASS` meer, na `SMP:SCRATCH:PASS`).

## Eén-CPU-regressie (op dezelfde bron)
- `tools/test_milestones.py --through m3`: M2 8/8 groepen, M3 22/22 groepen. M3 bouwt de nieuwe
  lockcode mee (diepte 0).
- `tools/run_rtc.py --case all` geslaagd, `tools/test_supervisor.py` (K05, 6 stappen),
  `tools/test_resource_pressure.py` (K07 en qsim geaccepteerd).

## Host en statisch
- `cargo test -p smp-lock`: 18 tests, waaronder de propertytest (2000 willekeurige reeksen van
  24 stappen acquire/paar/release tegen een eenvoudig stapelmodel; beide modellen moeten het bij
  elke stap eens zijn) en de gevallen paar in verkeerde volgorde, derde lock, losse tweede lock en
  release van de eerste van een paar.
- `cargo clippy --locked -p smp-lock --all-targets -- -D warnings` en `cargo fmt --all -- --check`
  schoon. `tools/tests/test_smp_serial.py` 13/13, `tools/tests/test_gs_discipline.py` 8/8 (nieuw:
  elke GS-stub telt na de proloog en telt terug vóór de epiloog; 0xe8 één keer).
- `tools/gate.sh` exit 0: cargo check, cargo test 511 geslaagd en 1 genegeerd. Het script
  `tools/qemu-smoke-test.sh` bestaat niet, dus die stap is niet uitgevoerd.

## Niet uitgevoerd en grenzen
- **Loom-model van `TicketLock`** (spec §4, F-T1): niet gedaan. Loom is een nieuwe
  dev-dependency met transitieve afhankelijkheden; volgens AGENTS.md eerst herkomst, licentie,
  onderhoud en build scripts toetsen (niet uitgevoerd). `TicketLock` zelf is in
  deze stap niet veranderd; de bestaande threadstresstests blijven. Open punt.
- `tools/test_milestones.py --through m4` niet gedraaid (hardgecodeerd `build/users`-pad, PR #235).
- CI: een gestapelde base start `native-smp.yml` niet. Geen hardware. TCG modelleert geen
  speculatie of echte geheugenordening.
- Het IN_HANDLER-negatief dekt KERNEL in 0xe8; ASPACE in een handler volgt dezelfde code (`≤ ASPACE`).

# K06 stap 4 — gepreëmpteerde kernelthreads op AP's

**Bron:** `c308b1ee9d8deedcfacd923c407392222ff7a12f` op
`k06/smp-step4-preempt`. Gestapeld op #203 (`e49e7819`), dan #202, dan
#201, dan #191 (`0cfcdd4a`). **Datum:** 5 november 2026. **Besluit:**
ADR-073. **Bewijsniveau:** native QEMU/TCG; geen fysieke hardware, review
door AI. **Score:** 41 → 41/100. K06 blijft open.

## Gerealiseerd

- Iedere AP heeft een periodieke LAPIC-timer (vector 0xd0). De stub bewaart
  alle 15 GPR's; `smp_preempt` wisselt round-robin tussen de thuislus en
  twee kernelthreads, elk op een eigen stack met guards. Threads draaien
  met IF=1.
- Elke slice van een thread zet unieke waarden in r12–r15 (plus registers
  die de compiler kiest), laat zich ongeveer 2000 PAUSE's lang meermaals
  preëmpten, en controleert daarna die waarden. Een fout in het
  opslaan of herstellen van de context geeft een harde fout.
- De BSP eist per AP minstens 8 ticks, minstens 4 contextwissels, en dat
  beide threads tussen twee metingen verder komen. Na STOP keert iedere AP
  terug naar zijn thuislus, en blijven tellers en ticks 10 ms bevroren.
- Een AP waarvan de timer nooit afgaat, leidt tot quarantaine van alle AP's
  (INIT), waarna de BSP alleen verder werkt.

## Native gevallen (14/14 geslaagd, exit 33, geen time-out)

Alle eerdere gevallen (1, 2, 4, 8 en 9 vCPU's; dubbele MADT, stall,
mismatch, dove IPI, NMI/#DF-IST, dove shootdown, controle zonder
shootdown) eisen nu ook `SMP:PREEMPT:PASS:CPUS=n` vóór het parkeren. Nieuw
is `test-smp-ap-timer-dead`: dat geeft `SMP:PREEMPT:TIMEOUT:APIC=1`,
`SMP:QUARANTINED:APS=3`, `SMP:ONLINE:1/4` en daarna een normale boot.
Details en hashes staan in [native-report.json](native-report.json).

## Commando's en regressie

Dezelfde commando's als in [stap 1](../k06-smp-bringup/README.md), nu voor
negen profielen. Host-tests: `boot-protocol` 28, alle 197 `tools/tests`
(met de `cc`-wrapper van de geïsoleerde toolchain), `test_common` 8, plus
Clippy en fmt. Regressie: K05 6/6, `milestone-m2` met ongewijzigd
`TCG/1CPU`, en `native-rtc` 5/5.

## Review

Een onafhankelijke AI-review vond geen kritieke of hoge punten. Verwerkt in
`c308b1ee`:
- **Medium:** er was geen bewijs dat registers een contextwissel
  overleven; dat levert nu de canary in r12–r15.
- **Medium:** er was geen bewijs dat na STOP niets meer draait; dat levert
  nu de bevriezingscontrole.
- De moduleheader is geactualiseerd.

Bij het bijwerken van de oracle-tests is aanvullend een fout vastgesteld: een dubbele
`"four"`-sleutel had bestaande afwijzingsgevallen stil overschreven. Die is
hersteld, en een AST-controle bevestigt dat er geen dubbele sleutels meer
zijn.

## Open

- Dit zijn kernelthreads zonder eigen adresruimte, met een vaste
  round-robin per AP en een ongekalibreerde tickfrequentie.
- Er is geen gedeelde runqueue, geen migratie en geen preëmptie of
  interrupts op de BSP.
- Gebruikersprocessen draaien niet op AP's, en er is geen lockvolgorde voor
  kernelsubsystemen; ADR-024 is ongewijzigd.
- Er is geen negatieve injectie die de canary bewust laat afgaan.
- Er is geen fysieke hardware getest.

# K06 stap 3 — TLB-shootdown vóór hergebruik van een fysieke pagina

**Bron:** `9c64c1dbe85290501934331a28a9ec7f61f39399` op
`k06/smp-step3-shootdown`. Gestapeld op #202 (`8c694ef0`), dan #201
(`cd78ea74`), dan #191 (`5263a5ef`). **Datum:** 5 november 2026.
**Besluit:** ADR-072. **Bewijsniveau:** native QEMU/TCG; geen fysieke
hardware, review door AI. **Score:** 41 → 41/100. K06 blijft open.

## Gerealiseerd

Op SMP geldt het `Frames::unmap`-contract voor iedere CPU. Het protocol
voor een vervangen kernelframe A (zie ADR-072):
1. Elke AP heeft `SHOOTDOWN_WINDOW` → A gecached; dat wordt gecontroleerd
   via `before`.
2. De BSP wist A. Daarna wijst het venster naar B en wordt de identity-alias
   van A gewist, beide alleen lokaal. De metadata blijft staan.
3. Vector 0xe0: elke AP voert `invlpg` uit op beide adressen en bevestigt
   precies de generatie van deze ronde.
4. Pas na alle acks: beide aliassen vrijgeven, A `free`n en een volledige
   paging-audit.
5. Een probe controleert dat elke CPU nu B ziet.

## Native gevallen (13/13 geslaagd, exit 33, geen time-out)

Alle gevallen uit stap 1 en 2 krijgen nu `SHOOTDOWN:PASS` en `VISIBLE`
vóór het parkeren (1, 2, 4, 8 en 9 vCPU's, single- en multi-TCG; stall,
mismatch en dubbele MADT). Nieuw:

| Profiel | Uitkomst |
|---|---|
| test-smp-ap-shootdown-deaf | TPR-klasse 0xe houdt alleen 0xe0 tegen: `SHOOTDOWN:TIMEOUT:APIC=1`, `FRAME_QUARANTINED` (A nooit vrijgegeven), alle AP's krijgen INIT, normale boot op één CPU |
| test-smp-shootdown-skipped (controle) | Zonder shootdown-IPI's lezen alle 3 AP's via hun verouderde TLB het vrijgegeven frame (`STALE:APS=3`), in multi- en single-thread TCG |

Het controlegeval bewijst dat de test een echt gevaar meet: zonder dit
protocol kan een CPU een al vrijgegeven, dus herbruikbaar, frame blijven
lezen. Hashes, SMP-regels en duur per geval staan in
[native-report.json](native-report.json).

## Commando's en regressie

Dezelfde commando's als in [stap 1](../k06-smp-bringup/README.md), nu voor
acht profielen. Host-tests: `boot-protocol` 25, `test_smp_serial` 4
(handgeschreven transcripts), `test_acpi_serial` 2, `test_rtc` 10,
`test_common` 8, plus Clippy en fmt; alle exit 0.
Regressie op deze head: K05 `test_supervisor.py` 6/6, `milestone-m2` met
ongewijzigd `TCG/1CPU`, en `native-rtc` 5/5.

## Review

Een onafhankelijke AI-review vond geen kritieke of hoge punten. Verwerkt in
`4fdf4e38`:
- **Medium:** de identity-alias van A werd alleen op de BSP verwijderd. Met
  een gedeelde CR3 kan een AP die alias speculatief cachen, dus nu gaan
  beide adressen in de shootdown.
- De generatie wordt per ronde opgehoogd.
- Er draait geen audit na quarantaine; dat is gedocumenteerd.
- Het SAFETY-commentaar van `invalidate` is gecorrigeerd.
- De functie is opgesplitst.

## Open

- Dit is één kernelvenster in een bootstrapprofiel. Nog niet gedaan: een
  algemene shootdown per ASID/CR3 en per range voor procesadresruimten, en
  integratie met de unmaps van M3.
- Er zijn nog geen interrupts op de BSP en er is geen SMP-scheduler;
  ADR-024 blijft ongewijzigd.
- Het controlegeval vereist exact `STALE:APS=3`. Een spontane TLB-uitzetting
  in een andere emulator zou het luid laten falen, niet stil laten slagen.
- Er is geen fysieke hardware getest.

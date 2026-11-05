# K06 stap 2 — eigen TSS/IST per AP en begrensde gerichte IPI's

**Bron:** `1993ade830ba9fbbba1d3c0913c12977599fe0e7` op
`k06/smp-step2-percpu-ipi`, gestapeld op #201 (`047acf39`), dat zelf op
#191 staat. **Datum:** 5 november 2026. **Besluit:** ADR-071.
**Bewijsniveau:** native QEMU/TCG; geen fysieke hardware, review door AI.
**Score:** 41 → 41/100. K06 blijft open.

## Gerealiseerd

- Iedere AP krijgt een eigen GDT-kopie en TSS. Die bevat RSP0, IST1 voor
  #DF en IST2 voor NMI, elk op een eigen stack met guards. Een GS-basisanker
  identificeert de CPU.
- Interruptvector 0xf0 telt per CPU en geeft EOI aan de eigen xAPIC.
  Vector 0xff is de spurious-gate. De AP maskeert alle LVT's die zijn
  xAPIC heeft en wacht met `sti; hlt`.
- De BSP verstuurt 3 unicast-rondes en eist per IPI precies één levering.
  Daarna parkeert hij iedere AP met een IPI. Broadcast wordt nooit
  gebruikt.
- Reageert een AP niet op een gerichte interrupt, dan krijgen alle AP's
  INIT en draait de kernel verder op één CPU. Een mislukte INIT stopt de
  boot.
- Een AP moet dezelfde IA32_APIC_BASE als de BSP melden (op de BSP-vlag
  na), anders volgt `MISMATCH`.

## Native gevallen (11/11 geslaagd, exit 33, geen time-out)

| Profiel | Geval | Uitkomst |
|---|---|---|
| native-smp | one, two-single, four, eight | alle CPU's online, sync PASS, `IPI:PASS:ROUNDS=3`, `PARKED` |
| native-smp | nine-capacity | 8/9 online; IPI's alleen naar eigen AP's |
| native-smp | duplicate-madt | `SMP:UNAVAILABLE:INVALID_MADT` |
| test-smp-ap-stall / -mismatch | 4 vCPU's | opstart gestopt, daarna 1-CPU IPI/park PASS |
| test-smp-ap-ipi-deaf | 4 vCPU's | `IPI:TIMEOUT:APIC=1`, `QUARANTINED:APS=3`, `ONLINE:1/4`, normale boot |
| test-smp-ap-nmi | 4 vCPU's | NMI op AP 1 → `SMP:FAULT:AP_NMI_IST:PASS` (op de eigen IST2-stack) |
| test-smp-ap-double-fault | 4 vCPU's | echte stack-overloop op AP 1 → `SMP:FAULT:AP_DF_IST:PASS` (eigen IST1) |

Vóór deze stap leidde een NMI of #DF op een AP tot een triple fault.
Hashes, SMP-regels en duur per geval staan in [native-report.json](native-report.json).

## Commando's en regressie

Dezelfde commando's als in [stap 1](../k06-smp-bringup/README.md), nu voor
zes profielen. Host-tests: `boot-protocol` 24, `test_smp_serial` 4
(handgeschreven transcripts, onafhankelijk van de oracle), `test_acpi_serial`
2, `test_rtc` 10, `test_common` 8, plus Clippy en fmt; alle exit 0.
Regressie: `milestone-m2` slaagt met ongewijzigd `TCG/1CPU`, en
`native-rtc` slaagt in 5/5 boots.

## Review

Een onafhankelijke AI-review (`unsafe`/asm plus security) vond geen
kritieke of hoge punten. Alle bevindingen zijn verwerkt in `1993ade8`:
- IA32_APIC_BASE van de AP werd niet vergeleken (medium).
- Het masken van CMCI en de LVT's volgde het werkelijke aantal LVT's niet.
- Na de quarantaine-INIT werd niet gewacht.
- De #DF-test had een race.
- `CpuTables` kon een paginagrens overschrijden.
- Commentaren claimden te veel.

## Open

- De BSP draait nog met IF=0, en er is geen TLB-shootdown en geen
  SMP-scheduler.
- AP's voeren nog geen kernelsubsystemen of gebruikersprocessen uit, dus
  ADR-024 is ongewijzigd.
- Een externe IOAPIC- of MSI-bron gericht op een AP stopt de VM terminaal;
  die faalt dus niet open.
- Er is geen x2APIC-modus en geen fysieke hardware getest.

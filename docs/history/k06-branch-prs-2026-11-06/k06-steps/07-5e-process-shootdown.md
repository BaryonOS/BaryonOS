# L0-specificatie — K06 stap 5e: procesbrede TLB-shootdown op `active_cpus`

**Datum:** 6 november 2026 · **Basis:** de head van stap 06 (5c-2) [nog niet gebouwd]
**Branch/worktree:** `feat/k06-5e-proc-shootdown` in `/home/simon/projects/nieuw-os-k06-5e`
**Spec:** `K06-SMP.md` §2 (INV-1, INV-4, INV-5), §5 punt 6, §7.1, §7.2, §9.3, §9.9–9.10; ADR-072, -075;
auditbevinding F18.

## 0. Vastgestelde feiten (op `7ffdacfa`, uit de audit; controleer ze bij de start)
- `ACTIVE[space]` wordt bijgehouden in `sched_switch` (5d-1), maar `shootdown_round` mikt op **alle**
  AP's.
- De 0xe0-handler flusht 2 vaste VA's zonder pid- of CR3-controle. Een ronde bevat maximaal 2 VA's.
- De twee testruimten zijn byte-identieke PML4-kopieën van de kernelroot. Ze bewijzen de
  wisselmechaniek, **geen isolatie**. Een PML4-kopie loopt uit de pas met latere wijzigingen in de
  kernelhelft op PML4-niveau.

## 1. Probleem (waarom nu)
Zodra M3-processen op AP's draaien (08), moet elke unmap, rechtenverlaging of procesexit alle CPU's
bereiken die vertalingen van dat proces kunnen cachen, en alleen die CPU's. Gebeurt dat niet, dan
leest of schrijft een CPU via een oude TLB-entry een frame dat al een nieuwe eigenaar heeft
(INV-1). Daarmee lekt data tussen processen.

## 2. Invarianten (normatief)
- **PS-1 Doelset = `active_cpus`.** Voor proces P: `targets = P.active_cpus.load(SeqCst) & !self`, na
  het wijzigen van de PTE en een `fence(SeqCst)` (spec §7.1 stap 3). De volgorde in §6.3 stap 5a–c
  garandeert dat geen CPU buiten `targets` een oude vertaling heeft.
- **PS-2 Gepubliceerde ronde.** `{pid, start, pages, gen}` met Release, onder `SHOOTDOWN`. Bereik
  begrensd: ≤ 32 pagina's per `invlpg`, anders een CR3-reload. De handler vergelijkt zijn huidige
  CR3 met die van P; bij gelijkheid flusht hij, anders niet. Hij ackt **altijd** met de generatie.
- **PS-3 Vrijgeven na alle acks.** Frames gaan pas naar `FRAMES.free` na alle acks. Een ontbrekende
  ack → de frames blijven in quarantaine en het beleid uit 05 (IQ-3) volgt.
- **PS-4 Kernelhelft.** Een wijziging in de gedeelde kernelhelft (behalve de eigen scratchslot uit
  01) gaat naar **alle online CPU's**, inclusief de BSP (kan sinds 06). De kernelhelft-PML4-entries
  worden bij de boot volledig voorgevuld, zodat er later nooit een nieuwe PML4-entry nodig is die
  kopieën zou missen.
- **PS-5 Procesexit** (spec §7.2): alle threads `DEAD` (04) → de exiterende CPU wisselt naar de
  kernel-CR3 → `active_cpus` moet 0 zijn → shootdown (meestal een lege doelset, maar het protocol
  loopt altijd) → K05-cleanup → frames vrij. `check_invariants` krijgt twee regels: *geen vrij frame
  wordt nog door een PTE in een levende ruimte genoemd* en *`active_cpus` van een dood proces is 0*.

## 3. Interface
- `smp.rs`: `shootdown_process(pid, range) -> Result<(), Quarantined>` en
  `shootdown_kernel(range)`; de oude `shootdown_round` wordt een interne helper.
- `paging.rs`: unmap en rechtenverlaging roepen `shootdown_process` aan onder `ASPACE[pid]` (+`FRAMES`).
- Testruimten: twee **echte** user-ruimten met een verschillend frame op hetzelfde VA (geen
  PML4-kopie), alleen in de fixture.
- Kernelfeatures `test-smp-proc-shootdown-deaf` en `test-smp-proc-no-shootdown`.

## 4. Native fixture en negatieve tests (spec §9.2, §9.3, §9.9, §9.10)
- **Isolatie:** ruimte A op AP1 en ruimte B op AP2 schrijven en lezen hetzelfde VA met verschillende
  patronen, 10.000×. Eis: nooit het patroon van de ander. Marker `SMP:ISOLATION:PASS`.
- **Unmap-race:** een thread in ruimte A op AP1 leest pagina X in een lus. De BSP unmapt X, geeft het
  frame vrij en hergebruikt het met een nieuw patroon. Eis: de lezer krijgt een gecontroleerde #PF
  (fixture-handler) en ziet nooit het nieuwe patroon. Marker `SMP:UNMAP_RACE:PASS`.
- **Gerichte doelset:** een ronde voor A raakt aantoonbaar alleen de CPU's in `ACTIVE[A]` (teller per
  CPU in de handler). Marker opgevouwen in `SMP:UNMAP_RACE:PASS`.
- `test-smp-proc-no-shootdown` (controleprofiel): zonder shootdown leest de lezer het hergebruikte
  frame → `SMP:SHOOTDOWN:STALE_READ` (bewijst dat de unmap-race een echt gevaar meet, zoals ADR-072).
- `test-smp-proc-shootdown-deaf`: AP1 negeert 0xe0 voor ruimte A → frames in quarantaine en het
  IQ-3-beleid. AP1 houdt geen lock vast, dus volgt INIT en draait de boot verder.

## 5. Schrijfbestanden
`kernel/src/m2/smp.rs`, `kernel/src/m2/paging.rs`, `kernel/src/m2/m3.rs` (alleen de exit- en
cleanuproute, als die al hier loopt), `kernel/Cargo.toml`, `tools/build.py`, `tools/run_smp.py`,
`tools/tests/test_smp_serial.py`, `.github/workflows/native-smp.yml`, `docs/DECISIONS.md` (aanvulling
ADR-072 en ADR-075 punt 6), `docs/implementation/K06-SMP.md` (§7), `docs/PROJECT-STATE.md`,
voortgangsbestanden.

## 6. Acceptatie
- `tools/gate.sh` exit 0; fmt; clippy; `test_smp_serial.py`; een hostmodel voor de
  `ACTIVE`-supersetregel met een nep-CR3 (audit §5).
- Native: `native-smp` (alle gevallen, 2/4/8 vCPU's, single- en multi-thread-TCG), beide negatieve
  profielen, de bestaande `ap-shootdown-deaf`/`shootdown-skipped-control`, regressie `milestone-m2`.
- K06 blijft open (41/100).

## 7. Buiten scope
PCID/INVPCID, een lazy TLB (een CPU die een ruimte "leent"), grote pagina's, IOMMU/DMA.

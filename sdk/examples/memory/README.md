# Native geheugengroei-proef

Deze architectuurproef voert werkelijke ABI1.7-geheugengroei uit in ring3 op
de eigen kernel. De normale route gebruikt `native::heap_grow`; alleen de
zeven opzettelijk ongeldige argumenten gebruiken een expliciet test-`int80`.
`Vec`, `Box` en `String` gebruiken de onderhouden `sdk/heap-allocator`, uitsluitend
boven een nieuwe eigen RW/NX-arena. Er is geen hostallocator of automatische
geheugengroei. Elke nieuwe byte wordt voor overdracht aan de allocator gelezen
en op nul gecontroleerd.

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools \
CARGO_TARGET_DIR=build/sdk-memory-probe-rust190 \
python3 -B sdk/examples/memory/build.py
```

De bouw gebruikt expliciet Rust1.90.0, een eigen vastgezette lockfile en
`x86_64-unknown-none`. Het JSON-resultaat geeft `NOS_MEMORY_ELF` en de werkelijk
geïnspecteerde ELF/hash. Alle processen gebruiken dezelfde executable met eigen
frames en een afzonderlijke private startpagina. Het protocol staat in
`src/contract.rs`; het is testmetadata, geen nieuw publiek opstartcontract.

De hoofdagent integreert featurekeuze, ELF-embedding en
`kernel/src/m2/m3/memory_fixture.rs` via de gedeelde bouw- en VM-procedures.
De features zijn `native-memory` en de afgeleide `native-memory-nx`,
`native-memory-guard`, `native-memory-peer`, `native-memory-oom`.
`native-memory` vereist `process-memory` en `fpu-context`. Legacyprofielen
blijven afzonderlijk. Geen andere M3/M4-demofeature hoort in deze fixture.

Normaal, NX, guard en peer gebruiken twee processen op het bestaande
pc-q35-8.2/qemu64/TCG/1CPU/256MiB-regressieprofiel. Drie levens omvatten een
nieuwe PID in het hergebruikte eerste processlot. Ieder leven controleert
513 pagina's groei over een pagetablegrens, argumentfouten, een geweigerde
te grote groei met daarna exact aansluitende succesvolle groei, de bestaande
totale 2048-framequota, eigen inhoud en behouden oude heap. In negatieve
gevallen voert uitsluitend het eerste proces daarna een echte foute
instructie uit: NX `#PF` met error21, onbeschreven suffix schrijven met error6,
of een fysiek frame van het levende andere proces lezen met error5.
De observer controleert vector14, foutcode, CR2 en RIP en laat het andere
proces aantoonbaar verder werken voordat het slot wordt hergebruikt.

OOM is een afzonderlijk negatief 128MiB-profiel met zestien processen.
Elk vraagt begrensde porties van64 pagina's tot de echte kernel Quota of
OutOfMemory meldt en houdt die frames bezet. Geen fixture onttrekt kunstmatig
frames aan de pool. Minstens één proces moet echte fysieke uitputting melden.
Pas wanneer alle zestien zijn geblokkeerd, mogen vijftien eindigen. Het
overgebleven OOM-proces moet daarna één nieuwe nulgevulde pagina precies op
de oude suffixgrens ontvangen. Dit verruimt de bestaande quota niet.

Vóór procesopruiming controleert de kernelobserver werkelijke eigenaar,
RW/NX/user-PTE, supervisoralias, aliasaantallen en de onbeschreven suffixgrens.
Direct ná opruiming en vóór hergebruik controleert hij alle vrijgegeven
heapframes onafhankelijk: eigenaarFree, geen aliases/PTE en alle bytes nul.
Permanente nieuwe kernelpagetables worden expliciet van de teruggewonnen
framebalans afgetrokken en apart geteld.

De negentien `MEMORY:`-tellers zijn in volgorde vastgelegd in de observer.
Voor elk geval geldt:

- `ZERO_BYTES = GROWN_PAGES * 4096`;
- `CLEANUP_ZERO_BYTES = ZERO_BYTES + CLEAN_STARTS * 65536`;
- `RETRY_ZERO_BYTES` is al in `ZERO_BYTES` opgenomen;
- `ARG_DENIED = CLEAN_STARTS * 7`, alle collectie-/privatecontroles eenmaal
  per leven, ten minste twee echte PIT-interrupts;
- geen levende processen en `FRAMES_RECOVERED = 1`.

Normale en foutproeven vereisen drie schone levens,21 argumentweigeringen,
6 quotaweigeringen en één slotreuse. Normaal eindigen drie processen;
een foutproef telt één fout en twee normale exits. OOM vereist zestien levens,
zestien exits,112 argumentweigeringen, minstens één fysieke OOM, één herstel,
4096 herstelde nulbytes en geen slotreuse. Het aantal fysieke OOM's plus
quotaweigeringen moet zestien zijn; pakketlayout of VM-allocatiedetails mogen
dit onderscheid niet worden voorgekookt.

Een brongebonden geslaagde VM-proef bewijst alleen deze concrete eigenschappen
op dit regressie-/negatieve profiel. De volledige 8GiB-productrelease,
willekeurige applicatiecompatibiliteit, fysieke hardware, SMP, heapkrimp en
gedeelde geheugenbuffers blijven afzonderlijke acceptatiecriteria. De lokale
ontwikkelproeven met tijdelijke integratiehooks zijn bovendien geen bewijs
voor een andere samengestelde hoofdbranchcommit.

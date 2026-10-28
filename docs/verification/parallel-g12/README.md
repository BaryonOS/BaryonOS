# G12 — gerichte review van de native DocumentSession-poort

## Bron en voorlopige uitkomst

Gereviewde immutable I13-commit:
`4ce6ba2f5bf7201848842fb10d54c7726ab975be`.
De review beperkt zich tot de nieuwe poort en bijbehorende gerichte
harnastests: selectie van user-ELF/libraries, compilerrecords, bronfreeze en
koude imagebinding. Er zijn geen VM, brede suite of wijzigingen aan de
implementatie uitgevoerd.

**Twee concrete afwijzingsgaten in compilerrecords zijn gevonden.** De echte
G11-compilerrecords zijn correct; hun tijdelijke gewijzigde kopieën worden
door de oorspronkelijke poort te ruim geaccepteerd. De hoofdagent heeft
reparatie I15 aan de poorteigenaar toegewezen.

## Bevindingen

| ID | Reproductie | Kleinste correctie |
| --- | --- | --- |
| G12-01 | In de echte catalog-client-rlib-record wordt `--extern catalog_core=...` vervangen door een vreemd rlibpad. `compiler_binding()` accepteert dit; alleen de finale bin-externset wordt gecontroleerd. | Controleer de exacte dependencyset voor iedere rlib: runtime/core geen externs, catalog-client runtime+core, document-session runtime+core+client. |
| G12-02 | Dezelfde echte compilerrecord krijgt achteraan `-C no-redzone=no`, naast de eerdere vereiste `no-redzone=yes`. De subsetcontrole accepteert de tegenstrijdige selectie. | Controleer voor de vier ABI-keys precies één verwachte waarde en wijs eerdere/latere duplicaten of overrides af, inclusief ondersteunde equivalente argumentvormen. |

`compiler_binding_probe.py` kopieert de vijf echte G11-compilerlogtrio's naar
een eigen TemporaryDirectory, bevestigt eerst dat de ongewijzigde records
worden geaccepteerd en voert uitsluitend daar de twee mutaties uit. Zij start
geen compiler of VM en wijzigt geen oorspronkelijke log of artifact.
`checks.json` bewaart argv, exitstatus, ruwe loghashes en het oorspronkelijke
resultaat. Een exit 0 van dit diagnostische script betekent dat de twee
proeven zijn uitgevoerd; de expliciete `rejected: false`-velden blijven de
bevindingen, geen groen acceptatieresultaat.

## Overige beoordeelde binding

Geen aanvullende concrete blokkade gevonden binnen de overige toegewezen
grenzen:

- Beide manifests moeten expliciet `document_session: true` en
  `sdk_runtime: true` selecteren, met de juiste normale/recovery-kernelcase.
- De vaste modulehashset, vier rlibs, user-ELF en bestaande ELF-inspectie worden
  werkelijk hergecontroleerd. De exacte user-ELF moet in de gestarte kernel
  voorkomen; kernel en loader moeten bij het fasegebonden manifest horen.
- De eerste build wordt vóór overschrijving door de recovery-build bewaard.
  Beide snapshots worden na afloop opnieuw gehasht; beide user- en
  library-artifacts moeten gelijk blijven.
- HEAD/source worden vóór en na iedere build en VM gecontroleerd. De bestaande
  volledige `capture_m6c.check_run` blijft voor beide runs verplicht.
- Cold recovery vereist een afzonderlijke invocation, de juiste vorige run,
  hetzelfde volume-ID/pad en exacte gelijkheid van normale after-image,
  recovery before/after en de actuele 64-KiB-image. Sequence 10 en ACK10→0
  blijven verplicht.
- De exacte 16/3 requests per client komen van de bestaande kernelobserver;
  de poort labelt ze terecht niet als nieuw geprinte serialcounters.

Dit blijft lokale integriteitscontrole met vertrouwde build/runnercode, geen
cryptografische producerattestatie of menselijke audit. De twee bevindingen
gaan over de nieuwe compilerrecordcontrole, niet over aangetoonde fouten in
de reeds gecompileerde G11-app.

## Gerichte default-ELFvergelijking

De oudere G10-default-SDK-ELF op bron `dbd35b04...` heeft hash `2ef58d24...`;
de nieuw gecompileerde default op G11-bron `91bf66c...` heeft hash `cfb96c4e...`.
Ze zijn **niet bytegelijk**. `default-binary-comparison.json` bewaart de volledige
hashes en vergelijking van hun echte load-segmenten en disassembly.

Beide bevatten 8605 instructies op dezelfde adressen en dezelfde RX/RW-
segmentgeometrie. Van 134 verschillen in de tekstuele disassembly zijn elf
uitsluitend LLVM-symboolannotaties. De andere 123 veranderen uitsluitend een
readonly-adres of RIP-verwijzing met acht bytes; er bleven geen ongeclassificeerde
instructiewijzigingen over. Ook de zes gewijzigde datawoorden zijn pointers
naar dezelfde readonly-regio met een verschuiving van acht bytes. De readonly-
sectie verschilt van 3055 naar 3047 bytes en bevat de daadwerkelijk verschillende
checkoutbronpaden en bronlocatiemetadata.

Dat verklaart de waargenomen code-/layoutverschillen, maar bewijst niet alle
data- of gedragsequivalentie. De door de hoofdagent geplande G13-regressie voert
daarom opnieuw de echte default-SDK normal/recovery-paar op de komende gezamenlijke
bronfreeze uit. Deze review heeft die VM's nog niet gestart.

# B02 — echte grafische helpertests en QMP-grenzen

Basis voor de grafische code: `41bac3c173d464a286017d31558335d70899c434`.
Datum: 28 oktober 2026. Afgebakende hostcontrole; geen nieuwe VM-uitvoering.

## Uitvoerbaar resultaat

`tests/graphics_helpers.rs` neemt `kernel/src/graphics.rs` **ongewijzigd** op.
De echte `boot-protocol`, `ui-core` en `kernel-core` worden met de gepinde
Rust 1.90.0-compiler gebouwd. Alleen de hardwaregebonden M2-/serial-entrypoints
zijn stubs: zij panikeren bij aanroep en doen nooit alsof een mapping of
gastlog is geslaagd. De harnasrunner controleert zowel de aanwezigheid als de
werkelijke succesvolle uitvoering van alle zes bestaande kerneltests.

**7/7 Rust-hosttests slagen:** de zes kerneltests plus één aanvullende test
van de echte decoderreset. Zij controleren het laatste zichtbare pixel,
stride/rijgrenzen, lege en te grote spans, alle pixels vóór de eerste store,
ongeldige metadata, RGB/BGR-kleurvolgorde, PS/2-pariteit-/timeout-/AUX-selectie,
gebruik vóór initialisatie en herstel van beide modifierzijden, Caps Lock
en gedeeltelijke extended-/pauseprefixes.

De spanvalidatie is echt uitgevoerd; de volgorde vóór de eerste volatile
store is daarnaast in de ongewijzigde bron beoordeeld. Er is **geen MMIO-store
of echte paginamapping op de host nagebootst**. De coretests bewijzen geen
PAT/PTE-instellingen, native invoertiming of fysieke hardwarewerking.

## Concrete bevindingen en hercontrole

Dezelfde definitieve Python-testbron is uitgevoerd vóór en na de reparaties
van de hoofdagent aan `tools/run_graphics.py`:

| Grens | Oude bron | Gerepareerde bron |
|---|---|---|
| QMP-correlation en succesvorm | Ontbrekende `return`, event als reply en bool/float-ID werden geaccepteerd | Exact integer-ID; `return` of `error`; geen event als commandresultaat |
| Verzenddeadline | Verlopen command werd verstuurd; pipe-write kon blokkeren; socket-timeout niet opnieuw begrensd | Controle vóór verzenden; socket-timeout en afzonderlijke pipe-chunks begrensd |
| Leesdeadline | Volledige reply kon na de gedeelde deadline worden geaccepteerd | Incrementele `read1` met deadlinecontrole rond chunks en acceptatie |

Vóór herstel: **9 testmethoden, 10 falende controles inclusief subtests**, gate
exit **1**. Na herstel: **9/9 testmethoden geslaagd**, volledige hostgate exit
**0**. De volle-pipeproef gebruikt een echte lokale pipe in een afzonderlijk
childproces; de oude implementatie overschrijdt de child-timeout van twee
seconden, de nieuwe stopt op haar eigen deadline van 50 ms. De late-readproef
gebruikt een modelklok en transportdouble; dat is geen echte socket-/VM-proef.

De projectkeuze voor integer-request-ID's en een gedeelde deadline is strikter
afgebakend dan het algemene QMP-protocol. Primaire referentie:
[QEMU Project — QEMU Machine Protocol Specification](https://www.qemu.org/docs/master/interop/qmp-spec.html),
secties Commands Responses, Success, Error en Asynchronous events;
geraadpleegd 28 oktober 2026. Het protocol onderscheidt succes, fout en event;
deze review kent daar geen extra native beveiligingsgarantie aan toe.

## Bronbinding en bewijs

- [review.json](review.json): exacte bronhashes, drie gesloten bevindingen en
  de beoordeelde unsafe-/mapping-/eigenaarschapsgrenzen.
- [before/result.json](before/result.json): rode run tegen de oorspronkelijke
  runner op de basiscommit, met afzonderlijke ruwe logs.
- [after/result.json](after/result.json): groene run tegen de gerepareerde
  rootrunner, met afzonderlijke ruwe logs.
- [initial-harness-compile-failure/result.json](initial-harness-compile-failure/result.json):
  eerste harnascompile faalde op ontbrekende accolades rond een Rust-constargument;
  deze harnasfout is hersteld en de mislukte uitvoer blijft zichtbaar.

De groene QMP-bronhash is
`279fa48d24ea615bf9031eca9fa3b0e2dedbd55717c7449b8ede7f8b8fd4cc00`.
De rootrunner stond tijdens die controle in de werkboom van de hoofdagent;
dit bewijs labelt haar daarom op inhoudshash en absoluut bronpad. De zes
grafische kerneltests blijven op de genoemde basisbron gebonden. De agentbranch
wijzigt de runner niet; integratie moet de reeds geteste rootreparaties behouden.

Binaries en tijdelijke buildoutput blijven buiten Git. De rapporten bewaren
hun hashes en exacte bouwcommando's; ruwe tekstoutput wordt niet genormaliseerd.

## Reproduceren

Met de gepinde Rust 1.90.0 op PATH, vanuit de samengevoegde projectroot:

```sh
python3 -B tools/test_graphics_helpers.py --output-root /tmp/nieuw-os-g0-helpers
```

Iedere aanroep maakt een nieuwe UUID-map met eigen libraries, testbinary,
commando's, stdout/stderr, tijdlimieten, hashes en `result.json`. Een ontbrekende
of genegeerde kerneltest, compilatiefout, bronwijziging, time-out of falende
QMP-grens maakt de gate rood. Met `--qmp-source /absoluut/pad/run_graphics.py`
kan een afzonderlijke runner worden gecontroleerd; dat pad en diens inhoud
worden expliciet in het bewijs vastgelegd.

## Aannames die blijven gelden

De huidige renderer heeft één CPU, IF=0, onveranderlijke framebuffermetadata
en geen herintredende grafische aanroep. Atomic-readyflags vormen geen SMP-
of preëmptieprotocol. In de huidige M2-volgorde wordt de software-readyflag
vóór CR3-activatie gezet, maar schrijft geen enkele caller pixels vóór
activatie en audit. Die volgorde moet behouden of opnieuw ontworpen worden
voordat andere processen, interrupts of CPUs deze helpers gebruiken.

B02 voegt controleerbaar hostbewijs en concrete harnasreparaties toe. De
native acceptatie op de gecombineerde bron blijft bij de hoofdagent.
Geen zelfstandig volledig roadmapcriterium gesloten: voorstel **14% → 14%**.

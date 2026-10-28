# G11 — native documentsessie gereed voor geïntegreerde VM-controle

## Bron en huidige bewijsgrens

Broncommit `91bf66ca2cdc00b7d2e220cd5afefd270d8fe062`, gebaseerd op
`dbd35b04d742b18d66b40fdd65d8a9b900656cac`. De opt-in gebruikers-ELF is echt
gecompileerd en door de bestaande ELF-inspectie geaccepteerd. **In G11 is nog
geen VM gestart.** De hoofdagent integreert de buildselectie en geeft een
afzonderlijke bronfreeze voor de echte normal/cold-recovery-poort.

De implementatie blijft een deterministische, headless native acceptatieproef.
Zij neemt geen G1/Materiaal-GUI, algemene appstarter, kernelinterface of
opslagformaat over.

## Gedrag en eigendom

Met `document_session=True` gebruikt de bestaande M6c-client de echte
`DocumentSession` boven G03/B01 en het bestaande `NativeTransport`. Een dunne
fixturewrapper telt iedere daadwerkelijke transportcall in het bestaande
private reportword 8; zij geeft resultaten en envelopes ongewijzigd door.

De normale client voert eerst de bestaande bevoegdheidsproeven en
catalogusbewerkingen uit. Daarna verhuist de bestaande GET9 naar de typed
`open`: het document is op revisie 2 met de oorspronkelijke inhoud na RENAME.
De client bewerkt de 128 opaque bytes, bewaart met exact die geladen
CAS-revisie, verwacht revisie 3, heropent en controleert inhoud/naam/revisie.
Een typed LIST vervangt de laatste bestaande cataloguscontrole. Daarmee blijven
**zestien requests per normale client**, sequence 10, tien mutation-ACKs en de
oorspronkelijke eindtoestand behouden.

De recovery-client opent het aanwezige document op revisie 3, controleert de
tombstone via typed `NotFound` en voert LIST uit: **drie requests per client**,
zonder mutatie. ReadOnly is daarbij uitsluitend lokaal appbeleid; de echte
autoriteit komt nog steeds van de bestaande endpointcapabilities en dienst.

Na overname gebruikt uitsluitend de typed Session de twee bestaande endpoints;
er is nooit meer dan één uitstaande aanvraag. Haar IDs lopen binnen die sessie
monotoon op. Aan het einde sluit zij send en receive elk eenmaal. De bestaande
echte stale-send-proef blijft staan en verwacht de kernelafwijzing voor de
gesloten handle. Het oude `finish()` wordt op dit opt-in pad niet nogmaals
aangeroepen. De oorspronkelijke authorityprobes blijven vóór de overname
volledig uitgevoerd; er worden geen extra capabilities gemaakt.

## Kleine gedeelde app-API-uitbreiding

De centraal goedgekeurde `DocumentSession::list(&mut self)` gebruikt uitsluitend
de bestaande `Operation::List` op de eigen Session. Zij controleert eerst
`available()`, verandert geen draft, geladen revisie, dirty/conflict/uncertain-
status en bewaart de draft bij servicefouten. Een onverwacht replytype sluit
de sessie via de bestaande foutafhandeling. Er is geen mutable-catalog-bypass
of reconnect/replay-pad toegevoegd.

Twee nieuwe tests gebruiken de echte capability-engine, G03/B01 en catalog-core:
LIST tijdens dirty en conflict behoudt de lokale draft; een echte Access-reply
behoudt eveneens de draft. Na Uncertain of Closed wordt LIST zonder verzending
geweigerd. De bestaande twaalf documentsessietests blijven groen.

## Buildselectie, grenzen en bronbinding

`build_users("m6c", sdk_runtime=True, document_session=True)` is de enige
toegelaten combinatie. Verkeerde userkind, ontbrekende SDK-bridge en short-poll
geven vóór buildbijwerkingen een fout. De hoofdagent beheert daarnaast de
centrale `NOS_DOCUMENT_SESSION`-selectie in `tools/build.py`; die centrale diff
zit niet in deze broncommit.

De entry blijft `userspace/m6c_probe.rs`. Het manifest bevat expliciete
`sdk_runtime`- en `document_session`-velden, alle betrokken Rust-/Cargo-/lock-
inputhashes en vier werkelijk gecompileerde rlibs. De compilercommando's
selecteren beide cfg's en koppelen diezelfde libraries aan de user-ELF. Er zijn
geen nieuwe externe dependencies.

De eerste crosslink vond de bestaande grens van één readonly-pagina. De
oplossing is het eenmalig delen van de bestaande `catalog_core::wire` en
`catalog_core::journal` tussen de servicefixture en typed client, uitsluitend
op het opt-in pad. De readonly-sectie past nu binnen **4096 bytes** zonder de
linker-, kernel- of capabilitygrenzen te verruimen. Een onderzochte LTO-poging
werkte niet met de aanwezige sysrootbitcode en wordt niet gebruikt; de
geslaagde build gebruikt de bestaande gewone `opt-level=2`. De default cfg
behoudt de oorspronkelijke lokale wire/journalmodules en selecteert geen
documentsessiecode of extra libraries.

## Uitgevoerde controles

Alle zes commando's in `checks.json` eindigen met exit 0:

- **41 SDK-hosttests:** 13 runtime, 14 transport en 14 documentsessie.
- **Drie gerichte userselectietests**, inclusief de verboden combinaties vóór
  enig compilercommando of outputcreatie.
- SDK-workspace Clippy voor alle targets, met `-D warnings`.
- Werkelijke native opt-in userbuild en bestaande ELF-inspectie.
- Afzonderlijke native default-SDK-userbuild; manifestflag false en geen
  documentsessie-cfg of extra libraries geselecteerd.
- Rustfmt-controle van de gewijzigde Rustbestanden.

`document-user-manifest.json` en `default-user-manifest.json` binden de echte
ELF's, segmenten en bronnen; de bijbehorende build-commandobestanden bewaren
de concrete rustc-argv's en exits. Alle modulehashes zijn ook tegen de inhoud
van de genoemde immutable commit gecontroleerd. De binaries blijven in de
eigen buildmappen buiten Git. Ruwe host-/compilerlogs en hun hashes staan naast
deze toelichting.

## Reproduceren en resterende controle

Gebruik Rust 1.90.0 met `x86_64-unknown-none` en de bestaande toolchainomgeving.
Vanuit de checkout van de genoemde bron, met een eigen buildmap:

```sh
export NOS_BUILD_DIR="$PWD/build/parallel-g11-document"
export PYTHONPATH=tools
python3 -B -c 'from build_users import build_users; build_users("m6c", sdk_runtime=True, document_session=True)'
cargo +1.90.0 test --manifest-path sdk/Cargo.toml --locked --offline --workspace
python3 -B -m unittest discover -s tests -p test_document_session_users.py -v
```

Gebruik voor parallelle Cargo-controles een eigen `CARGO_TARGET_DIR`; deze run
gebruikte aparte `/tmp/nieuw-os-par-g11-*`-mappen en `CARGO_BUILD_JOBS=2`.
De volgende gate moet op één centraal bevroren bron werkelijk booten, alle
bestaande kernelobserver-/diskorakelchecks uitvoeren, dezelfde image in een
nieuwe VM teruglezen en de ingebouwde user-ELF plus alle dependencies opnieuw
binden. Daarmee worden ook de nog niet native gecontroleerde stack-, IPC- en
timinggrenzen van deze combinatie getoetst. Deze bundel geeft daarvoor geen
vooruitlopende vrijgave en wijzigt geen roadmapscore.

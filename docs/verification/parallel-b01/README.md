# B01 — begrensde native catalogusclient

Basis: `1acf6e8cde2a63849dc49c948ad97ba7f2c39cad`, 28 oktober 2026.
Werkbranch: `codex/parallel-catalog-client-20261028`.

## Resultaat

`catalog_core::client` biedt een allocatorvrije `no_std`-client voor het
bestaande catalogusprotocol v4 en de bestaande Message320-ABI. GET, PUT,
DELETE, RENAME en LIST zijn getypeerde operaties. De client bewaart maximaal
één uitstaand verzoek en geeft request-ID's zonder hergebruik of wrap uit.
Een ongeldige bewerking wordt vóór transport afgewezen. Namen, inhoud,
objectdomein en CAS-revisies behouden de bestaande v4-proefbudgetten.

Antwoorden worden gebonden aan service-PID, tag, volledige berichtlengte,
payloadlengte, lege capabilityvelden, nulpadding, request-ID, epoch, operatie
en object. Bestaande `wire`-validatie controleert de inhoud en catalogus.
Een onzekere, verloren of beschadigde uitkomst sluit de sessie. Bij een
transport-/protocolfout vermeldt de fout expliciet of een mutatie al uitgevoerd
kan zijn. De client herhaalt geen verzoek en accepteert daarna geen late ACK.
Een geldige `Unknown`-reply blijft een fout en sluit ook verdere leesoperaties.

Dit is bibliotheekcode voor userspace. De client doet geen syscalls en verleent
geen rechten. De native adapter moet de oorspronkelijk ontvangen Message320
doorgeven, onverwachte ontvangen capabilities sluiten zonder hun velden voor
validatie te wissen, en transport-/tijdslimieten handhaven. Een nieuwe client
is geen serviceherstel en maakt een onzekere write niet veilig herhaalbaar.
Epoch/PID/domein moeten uit gecontroleerde startup-/herstelgegevens komen.

## Uitgevoerde controles

De dependencyvrije echte cataloguscore en integratietests zijn rechtstreeks
met Rust **1.90.0** gebouwd en uitgevoerd. **45 tests slagen**: 30 bestaande
core-/wiretests, 14 nieuwe clienttests en één test voor request-ID-uitputting.
De nieuwe controles omvatten open/bewerk/bewaar/heropen, hernoemen,
tombstones, herstel vanuit echte journalrecords, CAS-conflicten, volle journal,
beide domeinen, maximale/lege inhoud, alle afgekorte berichtlengtes,
capabilities, verkeerde herkomst/correlatie, time-outs en late replies.

De hostfixture gebruikt de echte `journal::State`, `Recovery` en `wire`.
Haar commit is een geheugenhandeling; zij levert **geen nieuw native
WRITE/FLUSH-, IPC-, QEMU- of duurzaamheidbewijs**. Dat hoort bij de aparte
SDK/appintegratie en haar VM-controles.

Daarnaast slagen de `x86_64-unknown-none`-corebuild, gerichte rustfmt,
Clippy voor de library en nieuwe tests. Na herstel van de dependencycache
slagen ook de normale Cargo-tests (dezelfde 45 tests), Cargo-Clippy voor alle
catalogustargets en de Cargo-build voor `x86_64-unknown-none`, alle op
broncommit `df7ac1af9fea8650ca0264d2fbd94e310ee0f67d`. De broncodecontrole
`git diff --check HEAD^ HEAD -- services/catalog` slaagt eveneens. De exacte
opdrachten, exitcodes, bronhashes, artifacthashes en ruwe uitvoer staan in:

- [direct-host-report.json](direct-host-report.json)
- [static-report.json](static-report.json)
- [cargo-report.json](cargo-report.json)

De eerste, geblokkeerde workspace-Cargo-aanroep is afzonderlijk bewaard in
[cargo-attempt.json](cargo-attempt.json): de lokale `r-efi`-index ontbrak,
waardoor geen catalogustest startte. Dit omgevingsprobleem is opgelost vóór
de geslaagde Cargo-reeks. Directe en Cargo-runs worden niet bij elkaar opgeteld.
Geen dependencies, syscall, wireformaat, opslagformaat of rechten gewijzigd.
De ruwe Rust-testuitvoer houdt haar oorspronkelijke afsluitende lege regels;
een diff-whitespacecontrole van alle bewijsbestanden meldt die regels. De
hierboven geslaagde diffcontrole is daarom expliciet tot broncode beperkt.

## Integratie en vervolgstap

De SDK-adapter gebruikt `Client::begin`, `Request::message` en
`Client::receive`; bij externe fout `Client::transport_failed`. Een succesvolle
IPC-enqueue mag niet opnieuw worden verstuurd. Alleen een werkelijk
niet-geënqueued `WouldBlock` kan dezelfde nog uitstaande message opnieuw
aanbieden. `Reply::ServiceError` is nooit een lege catalogus of succes-ACK.

Native eindgebruikersfunctionaliteit en roadmapacceptatie worden pas op de
samengevoegde bron met de adapter vastgesteld. B01 sluit op zichzelf geen
volledig roadmapcriterium. Voorgestelde scorewijziging: **14% → 14%**.

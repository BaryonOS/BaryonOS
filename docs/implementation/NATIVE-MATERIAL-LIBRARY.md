# Native Materiaal: naam, collectie en geschiedenis

Basis `159ad799d2a94fb6542d60e26ed8bbd22bdbd752`, 2 november 2026.
Uitvoering voor U03; het criterium blijft open tot native acceptatie.
Het bestaande sessie-/privacy-/opslagcontract blijft leidend.

## Gebruikersroute en visuele regels

Materiaal blijft een object met eigen inhoud, naam en revisies. De bibliotheek
toont werkelijke lokale documenten, een zoekveld en de vier begrensde collecties
die de dienst ondersteunt. Een verse disk toont een nuttige lege toestand.
Geen voorbeeldinhoud wordt als persoonlijk werk voorgesteld.

Het documentvenster krijgt proportionele tekst met één layout voor tekst,
selectie, caret, wrapping, pointer en verticale navigatie. Titel, lokale locatie,
bewaarstatus, rechten en geschiedenis krijgen vaste herkenbare plaatsen.
Neutrale oppervlakken, rustige hiërarchie en bestaande systeemtypografie volgen
QUIET-SESSION-SPEC; geen nieuw palet, kaartgrid of decoratieve animaties.
UTF-8-bytes veranderen nooit om de weergave passend te maken. Volledige shaping,
IME en een native schermlezer blijven afzonderlijk onbewezen.

- F2 bewerkt de naam; Enter wijzigt de draft, Ctrl+S bewaart expliciet.
- Ctrl+F zoekt lokaal op naam; Escape verlaat het veld zonder draftverlies.
- Ctrl+G opent ordenen en collectiebeheer; pas een bevestigde dienstbewerking
  mag een blijvende wijziging als geslaagd tonen.
- Alt+H opent geschiedenis. Herstellen leest een bestaande versie en maakt
  een nieuwe revisie; het vervangt geen bewijs van de voorgaande revisie.

## Dienstgrens

Geen nieuw opslagformaat, kernelrecht of IPC-ABI. De bestaande volume-v7-dienst
handhaaft domain, object, verwachte revisie en read-only-rechten. SDK-wrappers
weigeren mutatie tijdens bewaren, bij dirty/conflict/uncertain en bij verkeerde
revisie. Iedere gepubliceerde mutatie wordt maximaal één keer verzonden. Een
onzekere uitkomst sluit de verbinding; de UI mag geen blind opnieuw-proberen
of succes tonen. Een bevestigde restore met mislukte herlezing wordt apart
gemeld en mag de oude draft niet als nieuwe actuele inhoud bewaren.

## Acceptatie

Hosttests toetsen tekstmetrics, UTF-8-grenzen, pointer/caret, schaal, focus,
stale acties, read-only, dirty, conflict, timeout en onzekere mutaties.
Native QMP voert maken, typen, hernoemen, bewaren, ordenen, zoeken, openen,
geschiedenis en restore uit op een nieuwe wegwerpdisk. Hele toegewezen
appbeelden worden exact met dezelfde Rust-renderer vergeleken, zonder masks.
De bestaande onafhankelijke diskdecoder controleert daarna echte bytes,
naam, collectie, revisies, geschiedenis en canaries. Een tweede verse VM
op een gecontroleerde kopie van uitsluitend dat testimage bewijst koude
heropening; lezen moet de disk ongewijzigd laten.

De bestaande volledige clipboard-/compositorherstelgate blijft apart verplicht.
Nieuwe hostbeelden vervangen geen native uitvoering, menselijke taakproef,
accountauthenticatie of hardwarebewijs. Score en bewijs worden pas na die
controles herbeoordeeld.

## Gemeten native codebudget

De eerste gekoppelde bibliotheekversie meet in de echte scalar-linkermap
131.177 bytes RX-code: 105 bytes boven het eerdere 128-KiB-demobudget.
Een afzonderlijke `opt-level=z`-proef overschrijdt dat budget eveneens;
thin LTO is met de gepinde prebuilt-core/code-modelcombinatie niet bruikbaar.
Beide mislukte proeven blijven diagnostiek, geen geaccepteerde binaries.

Het appspecifieke linkerscript reserveert daarom vast **144 KiB RX** vanaf
hetzelfde adres `0x400f0000`. Read-only blijft 512 KiB; data/BSS blijft 256 KiB;
de drie 64-KiB-documentbuffers, entry/startup-ABI, stack/guards en kernelquota
blijven gelijk. Geen W+X, capability-, apparaat- of kernel-ABI-uitbreiding.
De echte ELF wordt opnieuw met dezelfde kernelparser op alle vijf segmenten,
non-overlap, rechten en totale framequota gecontroleerd. Het is geen reden om
een onverwachte verdere groei of mislukte native acceptatie te negeren.

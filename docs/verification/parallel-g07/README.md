# G07 — documentbewerklaag boven de bestaande catalogusclient

## Opgeleverd

Broncommit: `ccb2112072574d45b9c94c6f7181de23828bf79a`.
Nieuwe crate: `sdk/document-session`, geregistreerd in de zelfstandige SDK-workspace.
De publieke API en het handoffcontract staan in
[`sdk/document-session/README.md`](../../../sdk/document-session/README.md).

De appbouwsteen opent één bestaand document, bewaart een lokale 128-byte
conceptbuffer en gebruikt bij PUT precies de geladen CAS-revisie. Een geldige
ACK verhoogt die revisie en maakt het concept schoon. Conflict, server-Access,
volle opslag, intrekking, timeout en onzekerheid behouden de lokale bytes.
Na een conflict volgt geen automatische rebase; na mogelijke publicatie en
verlies van het antwoord wordt niets opnieuw verstuurd. Heropenen en lokaal
verwerpen zijn afzonderlijke expliciete acties.

Naam en object blijven ongewijzigd; inhoud bestaat uit opaque bytes.
ReadOnly is uitsluitend appbeleid. De werkelijke dienst en kernelgebonden
endpointcapabilities behouden de autoriteitsbeslissing. De library is
`no_std`, allocatorvrij en `forbid(unsafe_code)`, met uitsluitend bestaande
lokale dependencies. Er zijn geen wijzigingen aan G1/Materiaal, userspace-
fixtures, kernel, syscalls, Message320 of journalformaat.

## Uitgevoerde controles

Alle zes vastgelegde commando's in `report.json` eindigen met exit 0:

| Controle | Uitkomst |
|---|---|
| Gecombineerde SDK-workspace | 39 hosttests: 13 runtime, 14 catalogustransport en 12 nieuwe documenttests. |
| Host-Clippy | Alle SDK-targets, `-D warnings`, geslaagd. |
| Native releasebuild | `nieuw-os-document-session --lib`, `x86_64-unknown-none`, geslaagd. |
| Native Clippy | Library op dezelfde native target, `-D warnings`, geslaagd. |
| Rustfmt | Alleen beide nieuwe Rustbestanden; dependencies niet herschreven. |
| Diffcontrole | Geslaagd op de gecontroleerde bron. |

De native rlib is 77.336 bytes en heeft SHA-256
`c539dc9de4450126ca0d45aa2d7467956799505603a8c2f4e7ce9f8e238755e3`.
Dit is een Rust-libraryartifact, geen gestart gebruikersprogramma.
De uitgevoerde native VM-teller voor G07 is **nul**.

## Betekenis van de twaalf nieuwe tests

1. Openen → bytebereik bewerken → save → reopen, met exact de oorspronkelijke
   revisie en behouden naam. De hostadapter ordent WRITE, FLUSH, corecommit en
   antwoord; er komt geen ACK vóór de gemodelleerde FLUSH-completion.
2. Twee echte capability-IPC-clients laden dezelfde revisie. De tweede writer
   krijgt Stale, behoudt het concept en probeert niet automatisch opnieuw.
   Alleen expliciete discard/reopen en bewust opnieuw aanbrengen van bytes
   leidt tot bewaren op de nieuwe revisie.
3. Een antwoord wordt pas na de echte journalcorecommit weggelaten. De oude
   sessie blijft dirty/uncertain en replayt niet. Een nieuw servicemodel bouwt
   uitsluitend uit journalrecordbytes de bevestigbare inhoud op. Corruptie
   van dat record wordt door de echte Recovery geweigerd.
4. Canoniek Unknown sluit verdere verzending; zelfs een expliciete lokale
   discard wist de onzekerheid niet.
5. Een begrensde EAGAIN-timeout vóór enqueue behoudt het concept, sluit de
   sessie en beweert niet dat de mutatie is uitgevoerd.
6. ReadOnly weigert lokale edits/save zonder transport. Een server-Access-
   antwoord blijft ook onder ReadWrite een fout met conceptbehoud.
7. Te grote inhoud en ongeldige/inverterende/extreme bytebereiken falen vóór
   wijziging of IPC.
8. Echte endpointintrekking in capability-core verslaat ReadWrite; het
   concept blijft behouden en de dienst ontvangt geen PUT.
9. Lege inhoud, precies 128 bytes en ongeldige UTF-8-bytes roundtrippen zonder
   nieuwe tekstpolicy; lege inhoud wordt ook na reconstructie teruggelezen.
10. Dirty open/reopen wordt geweigerd. NotFound en een object uit een ander
    domein vervangen de bestaande inhoud niet stilzwijgend.
11. De echte 32-recordgrens geeft Full; de geladen revisie en het concept
    blijven intact, zonder automatische herhaling.
12. Terugbewerken naar de oorspronkelijke bytes maakt clean. Clean save en
    lokale discard veroorzaken geen IPC; close behoudt het concept en sluit
    beide endpoints één keer.

De tests gebruiken de bestaande capability-engine, G03 `Session<T>`, B01-
correlatie, canonieke wirecode, `State::prepare/commit` en volledige
`Recovery`. De serviceadapter, WRITE/FLUSH-completions en opslagbuffer zijn
expliciet een **hostmodel**. Hieruit volgt geen nieuwe native duurzaamheid,
echte diskcrash, GUI, G1 of appstartclaim. De native G05-runtimeproef is
afzonderlijk bewijs van de onderliggende transportlaag.

## Bronbinding en reproduceerbaarheid

`report.json` bevat alle argv, exitstatussen, gescheiden targetmappen,
stdout-/stderrhashes, bronhashes vóór/na en de native artifacthash.
Alle gecontroleerde runtime-, catalogus- en capabilitybronbestanden zijn
bytegelijk aan rootfreeze `3e41bbf93822f7044c6020259f4a752925e1966d`.
De rootwerkboom is door G07 niet gewijzigd. Het nieuwe component wordt
afzonderlijk geïntegreerd; oudere native captures krijgen geen nieuwe
bronidentiteit.

De eerste hostfixture gebruikte onterecht niet-monotone PID-registratie;
de echte capability-engine wees dat af. Alleen de fixturevolgorde is
gecorrigeerd. Clippy wees daarnaast de letterlijke omgekeerde testrange af;
dezelfde negatieve invoer wordt nu als expliciete `Range { start, end }`
aangeboden, zonder lint uit te schakelen. De opgeslagen definitieve checks
zijn daarna op de bovenstaande bron uitgevoerd.

## Open integratiepoort en handoff

De G1/Materiaal-eigenaar kan `DocumentSession<NativeTransport>` in de
appstaat bewaren en handlers via `&mut` laten werken. GET/PUT gebruiken
uitsluitend de bestaande catalogusendpoints en 128-byte-grens. Na onzekere
save moet de app het concept bewaren en pas na onafhankelijk herstel met
nieuw gevestigde grants een nieuwe sessie openen; er is geen verborgen
reconnect- of replayroute.

Een echte native appstart met deze nieuwe laag en de gebruikershandeling
in de GUI blijven open. Aanmaken, hernoemen, verwijderen en rijkere tekst-
of documentformaten zijn afzonderlijke apphandelingen. Deze taak neemt
geen G1-bestanden of productkeuzes over en wijzigt geen scorebestand.

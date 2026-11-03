# DOCX naar gewone tekst

Nieuwe afzonderlijke importconsumer voor bestaande tekst-DOCX-bestanden.
Geen editor, viewer, Word-vervanger of renderer. Verliest expliciet opmaak;
tabellen, afbeeldingen, velden, wijzigingen, hyperlinkinhoud, directe hidden
properties, sectieheaders/-footers en onbekende inhoud worden geweigerd,
niet als onvolledige geslaagde tekst teruggegeven.
Gekoppelde stijlen worden niet visueel uitgewerkt; geen zichtbaarheidclaim.
De bron wordt alleen gelezen en niet gewijzigd.

```sh
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/document-import/Cargo.toml --features host-cli
cargo +1.90.0 run --offline --locked --manifest-path sdk/examples/document-import/Cargo.toml --features host-cli --bin docx-text -- --plain-text < bron.docx > tekst.txt
```

`--plain-text` is verplicht als expliciete keuze voor verlies van opmaak.
Stdin/stdout zijn de enige gegevensinterfaces. Geen netwerk-/bestandsopeningen,
extractie naar paden, macro's, veldcode-uitvoering, entityresolver of
automatisch volgen van relaties. CLI-fouten geven stderr/exit 1, geen
successtdout. Een schrijffout kan stdout gedeeltelijk hebben geschreven;
geen atomaire opslagclaim.

## Afgebakend formaat

OPC-pakket met ZIP32, een schijf, UTF-8/ASCII-partnamen, stored of raw-DEFLATE.
Lokale records en central directory moeten dezelfde volgorde en inhoud hebben,
zonder zelfuitpakkende prefix, gaps of overlappende records. Data descriptors
met/zonder signature en ZIP-comments zijn toegestaan; ZIP64/encryptie en
andere methodes niet. CRC-32 en gedeclareerde outputlengte worden per part
gecontroleerd; CRC is geen authenticatie. Geen algemene ZIP-API.
Case-collisions en dubbele namen worden geweigerd. Geen pad op disk aangemaakt.

Verplichte namespace- en typebinding via `[Content_Types].xml` en `_rels/.rels`
naar gewone Transitional `word/document.xml`. Geen macro-enabled main type.
Externe rootrelaties afgewezen. Andere parts worden CRC-/lengtegecontroleerd
maar niet geïnterpreteerd; geen externe relatie uit zo'n part wordt gevolgd.
Alleen de main body wordt geïmporteerd. De ondersteunde body bestaat uit
alinea's/runs/tekst/tabs/gewone linebreaks en beperkte opmaakproperties.
Lijsten/nummering en onbekende structuren worden geweigerd. Lege en
afsluitende alinea's blijven als LF behouden; geen extra LF aan het einde.
Tekstentities en XML-regelovergangen worden volgens XML geïnterpreteerd.
Namespace-URI, niet de gekozen prefix, bepaalt of een element Wordtekst is.

## Grenzen

Maximaal 16 MiB pakket, 512 parts, 1 MiB per gedecomprimeerd part,
16 MiB totaal, 32768 XML-nodes per XML-part, 4096 alinea's en 64 KiB tekst.
Eigen outputbuffers reserveren fallible. XML/DEFLATE-upstreamallocaties
blijven infallible. zune-inflate's limiet is best effort en kan beperkte
decode-/buffermarge overschrijden; de finale lengte/CRC wordt opnieuw getoetst.
Zijn API meldt geen exact verbruikte raw-DEFLATE-bits: deze slice claimt
geen detectie van padding/junk binnen een gedeclareerd compressed record.
Geen harde native heap-/tijdquota of OOM-herstelclaim zonder parserproces.
Een native parserproces/capability-grens toevoegen is General/K05-werk,
niet door deze hostconsumer omzeild.

Upstreamkeuze/licenties/pins in `THIRD-PARTY.md` en Cargo.lock.
Basisnormen: Microsoft, *Structure of a WordprocessingML document*,
12 januari 2024,
<https://learn.microsoft.com/en-us/office/open-xml/word/structure-of-a-wordprocessingml-document>;
PKWARE, APPNOTE 6.3.10, 1 november 2022, secties 4.3/4.4,
<https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT>.
Geen Office-, LibreOffice-, schemavalidatie- of volledig OOXML-profielclaim.

## Acceptatie

Gerichte Rusttests, echte Python-zipfile/ElementTree DOCX-inputs door de CLI,
Clippy/fmt en no_std-librarybuild. Bewijsreceipt wordt pas na uitgevoerde
acceptatie toegevoegd. Hostbestandsfunctionaliteit en compilatie zijn geen
native open/save-/launch-/vensteracceptatie. Geen fysieke hardware-,
menselijke gebruiks-, onafhankelijke securityaudit- of releaseclaim.

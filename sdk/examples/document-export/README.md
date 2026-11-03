# Tekst naar een echt DOCX-bestand

TASK-PLATFORM-M7-DOCX-TEXT-EXPORT, basis `6d523964`.
Een concrete interoperabiliteitsfunctie: UTF-8-tekst via stdin omzetten naar
een zelfstandig WordprocessingML/OPC-document via stdout. Geen tweede editor,
Office-parser, ZIP-importer of lege formaatfoundation.
De exporter is `no_std+alloc`; de werkelijke CLI is uitsluitend een hostprogramma.

```sh
cargo +1.90.0 run --offline --locked --manifest-path sdk/examples/document-export/Cargo.toml --features host-cli --bin text-docx < tekst.txt > document.docx
```

## Concreet exportprofiel

- Tekst blijft Unicode, inclusief accenten, niet-Latijnse tekens en emoji.
  Geen fontsubstitutie of glyphfallback in de export; de ontvangende
  documentapp bepaalt fonts/layout.
- Iedere LF wordt een alinea, inclusief lege en afsluitende alinea's.
  Tabs worden `w:tab`; spaties worden met `xml:space="preserve"` vastgelegd.
  Geen trim of interpretatie van markup, URL's of formuleachtige inhoud.
- Geen stille conversie: ongeldige UTF-8, XML-ongeldige tekens en CR/CRLF
  geven een fout met waar mogelijk de bronbytepositie. De caller kiest een
  eventuele normalisatie vooraf; deze exporter wijzigt de bron niet.
- Maximaal 64 KiB invoer, 4096 alinea's en 4 MiB uitvoer; een caller kan
  het uitvoerbudget verlagen. Limieten gelden inclusief exacte ZIP-overhead.
- Drie vaste OPC-parts: `[Content_Types].xml`, `_rels/.rels`,
  `word/document.xml`. ZIP stored, geen compressie, ZIP64 of encryption.
  Een vaste datum 1980-01-01 maakt output deterministisch; dit is geen
  werkelijke bewerkingsdatum. CRC32 is een corruptiechecksum, geen authenticatie.
- Geen macros, automatische velden, embedded objecten, externe relationships,
  hyperlinks, scripts, auteurmetadata of bestandsnamen uit de invoer.

Dit is een **tekstexportprofiel**, geen DOCX-import/roundtrip-, stylesheet-,
typografie-, paginering-, tabel-, afbeelding-, revisie-, screenreader- of
volledige Office-compatibiliteitsclaim. De output bevat plaintext; een
DOCX-export is niet end-to-end-versleuteld en moet via bewust gekozen
uitvoer/opslag behandeld worden. Geen upload of netwerkfallback.

## Implementatie- en resourcegrens

Geen unsafe of dependency. Alleen de minimale standaard ZIP-serializer voor
de drie vaste parts, geen algemene archiefbibliotheek of parser.
Vaste inputlimiet begrenst alle lengtes voordat ze naar ZIP-u16/u32 gaan:
de documentinhoud is minder dan 3 MiB; partnamen zijn vaste ASCII-constanten.
XML en ZIP-output worden ieder vooraf exact/fallible gereserveerd; werkgeheugen
is circa tweemaal de uitvoer plus invoer, geen native harde heapquota.

Alleen `sdk/examples/document-export/**` en eigen receipt zijn nieuw.
Geen General-editor-/document-session-, Work-, gedeelde workspace-, kernel-,
driver-, launch-, rechten- of scorewijziging. De no_std-API werkt op callerbytes
en verleent geen bestands-/netwerkcapability. Een native exportactie moet
door de bestaande General-/portalowner worden aangesloten op geautoriseerd
opslaan; die koppeling blokkeert deze bestandsfunctie niet.

De CLI schrijft pas stdout nadat validatie en export slagen; fouten geven
exit 1 en stderr zonder documentinhoud. Stdoutwrite is niet atomair:
I/O-fouten kunnen een gedeeltelijk bestand opleveren. Gebruik bij
shellredirectie nooit hetzelfde invoer-/uitvoerpad.

## Primaire bronnen en acceptatie

Microsoft, *Structure of a WordprocessingML document*, 12 januari 2024,
<https://learn.microsoft.com/en-us/office/open-xml/word/structure-of-a-wordprocessingml-document>.
PKWARE, *APPNOTE: ZIP File Format Specification*, 6.3.10, 1 november 2022,
secties 4.3.7, 4.3.12 en 4.3.16,
<https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT>.
Geraadpleegd 3 november 2026; geen brontekst overgenomen.
De ECMA-376-landingspagina kon niet worden opgehaald (HTTP 403);
geen volledige norm-/schemavalidatie geclaimd.

```sh
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/document-export/Cargo.toml --features host-cli
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/document-export/Cargo.toml --features host-cli --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/examples/document-export/Cargo.toml -- --check
cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/document-export/Cargo.toml --target x86_64-unknown-none
python3 -B sdk/examples/document-export/tests/interop.py target/platform-docx-host/debug/text-docx build/platform-docx-evidence/example.docx
file build/platform-docx-evidence/example.docx
```

Zes Rusttests en 133 echte CLI-exports worden onafhankelijk met Python
`zipfile`/CRC en `ElementTree` gecontroleerd op vaste packageparts,
relationships, XML-namespaces en exact teruggelezen Unicode/tab/alinea-inhoud.
`file` herkent de output als Microsoft Word 2007+. Dat bewijst **niet**
dat Word/LibreOffice die uitvoer daadwerkelijk geopend/gerenderd hebben.
Geen native executable, GUI/saveportal, QEMU, hardware-, menselijke,
schema-/Officeapp-, onafhankelijke securityaudit- of releaseacceptatie.

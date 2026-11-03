# CSV-tabelconsumer

TASK-PLATFORM-M7-CSV-TABLE, basis `d10b5cda`. Een kleine app-local
`no_std + alloc`-tabel met een **host-only** bestandsconsumer:
CSV importeren, cellen als tekst bewerken, exporteren en heropenen.
Dit is geen native spreadsheetapp, formule-engine, venster of XLSX-compatibiliteit.
Geen nieuwe registrydependency, kernel-ABI, runtime of rechtenlaag.

## Gebruik en formaat

```sh
cargo +1.90.0 run --offline --locked --manifest-path sdk/examples/table-editor/Cargo.toml --features host-cli --bin table-csv -- --set 2 3 '€ 42,50' < invoer.csv > uitvoer.csv
```

`--set RIJ KOLOM TEKST` mag herhaald worden; indices beginnen bij 1.
Zonder edit wordt het bestand gecontroleerd en canoniek geschreven.
Alle import-, argument-, edit- en exportcontroles gebeuren voordat stdout
wordt geschreven. Bij zo'n fout: exit 1, een fout op stderr, geen CSV-output.
Een I/O-fout tijdens schrijven kan wel gedeeltelijke output achterlaten.
Gebruik nooit dezelfde invoer-/uitvoerbestandsnaam bij shellredirectie:
de hostshell kan de invoer vooraf afkappen. Dit is geen atomair bestandssavepad.

Gekozen profiel op basis van Y. Shafranovich,
*[RFC 4180, Common Format and MIME Type for CSV Files](https://www.rfc-editor.org/rfc/rfc4180.html)*,
oktober 2005, paragraaf 2 (informational, geen universele CSV-standaard):
komma's, dubbele quotes, verdubbelde quotes, geciteerde newlines en gelijke
kolomaantallen. Spaties, headertekst en nummerachtige waarden blijven letterlijk.
Geen schema-/type-/headerdetectie, delimitergok, platformencoding of trim.
UTF-8, optionele UTF-8-BOM en LF-records zijn expliciete uitbreidingen.
Een lege laatste cel na een komma blijft behouden; export quote lege cellen.
Export is UTF-8 zonder BOM met CRLF-records en minimaal benodigde quotes.
Een leeg bestand is een nulrijentabel; een lege record is een lege cel.
Ongeldige UTF-8, NUL, losse CR buiten quotes, ragged rows, onverwachte of
niet afgesloten quotes en tekst na een afsluitquote worden expliciet geweigerd.

`Table::set_cell` is transactioneel: een mislukte edit behoudt de oorspronkelijke
cel en het tekstbudget. `Table::export` controleert de totale uitvoergrootte en
reserveert eerst alle uitvoerruimte. De modelallocaties zijn fallible;
dit is geen native procesarena of bewijs van herstel bij iedere host-OOM.

## Grenzen en veiligheid

Maxima: 4 MiB invoer en celtekst, 8 MiB uitvoer, 64 KiB per cel,
4096 rijen en 256 kolommen. `Limits` kan deze grenzen uitsluitend verlagen.
Dit zijn expliciete grenzen van deze eerste consumer, geen productkeuze voor
grote spreadsheets. Header-/vectoroverhead komt bovenop celtekst; een native
isolatie- en harde heapquota ontbreken.

Cellen met `=`, `+`, `-`, `@`, tabs of formules blijven **letterlijke data**:
deze consumer voert niets uit en wijzigt zulke waarden niet stil.
Andere spreadsheetprogramma's kunnen diezelfde CSV wel als formule uitvoeren.
De uitvoer is dus **niet formule-injectieveilig voor externe spreadsheets**.
Dat vraagt een aparte expliciete exportkeuze; quoting alleen lost dit niet op.

De host-CLI ontvangt alleen de stdin-/stdout-/stderrhandles van zijn caller.
Hij is geen Nieuw-OS-resourcebroker. Een native consumer moet via de bestaande
objectportal en geautoriseerde volumeclient werken; padnamen of tabelcellen
verlenen geen capability. Geen netwerk, NAS, shell-executie of cloudfallback.
Inhoud wordt niet in foutmeldingen gelogd.

## Controles

```sh
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/table-editor/Cargo.toml --features host-cli
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/table-editor/Cargo.toml --features host-cli --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/examples/table-editor/Cargo.toml -- --check
cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/table-editor/Cargo.toml --target x86_64-unknown-none
```

Hosttests omvatten echte CLI-processen, import/edit/export/heropenen,
Unicode/quotes/multiline/empty/BOM, exacte budgetgrenzen, fouttransacties,
letterlijke formules en deterministische malformed-mutaties.
De crossbuild bewijst uitsluitend targetcompilatie; geen native appstart,
renderer, volumeopslag, undo/formules, XLSX/Office-pariteit of releaseacceptatie.

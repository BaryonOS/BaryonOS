# TASK-PLATFORM-M7-CSV-TABLE

Basis `d10b5cdaf56774a39e3ef10463f674c6188a463a` (actuele main bij start),
branch `platform/csv-table-20261103`. Onafhankelijk van EXIF-viewer #165.
Code-/testcommit `9a27482672562f1d406fbc422781fb55bac24005`;
de vervolgcommit past uitsluitend deze bewijsnotitie aan.
Schrijfclaim uitsluitend `sdk/examples/table-editor/**` en dit ontvangstbewijs;
geen gedeelde SDK-workspace, General-editor, Work Suite-, package-, kernel-,
driver-, centrale status-/score- of rechtenwijziging.

## Resultaat en bewijsniveau

Een werkelijk uitgevoerde hostconsumer leest UTF-8-CSV via stdin, importeert
een begrensde rectangulaire tabel, bewerkt geselecteerde cellen en schrijft
canonieke CSV via stdout. De output wordt opnieuw geimporteerd en onafhankelijk
met Python `csv.reader` geopend. Het no_std-model heeft dus een concrete
afnemer; dit is geen losse parserfoundation.

Het model accepteert alleen expliciet beschreven comma/quote/CRLF/LF/UTF-8
invoer, behoudt Unicode, spaties, lege cellen, gequote newlines en tekst die
op getallen/formules lijkt. Geen inferentie of formule-evaluatie.
Mislukte edits zijn transactioneel; argument-, parse-, budget- en
exportfouten leveren geen succesoutput. I/O-fouten tijdens stdout kunnen
wel een gedeeltelijke write geven; dit is geen atomair bestandssavepad.

Gekozen CSV-profiel en uitbreidingen zijn per feature beschreven in
`sdk/examples/table-editor/README.md`, gebaseerd op Y. Shafranovich,
RFC 4180, oktober 2005, paragraaf 2,
<https://www.rfc-editor.org/rfc/rfc4180.html> (primaire bron geraadpleegd).
Geen claim van alle CSV-dialecten of Office/XLSX-roundtrippariteit.

Modelgrenzen: 4 MiB input/celtekst, 8 MiB uitvoer, 64 KiB/cel,
4096 rijen, 256 kolommen; caller kan uitsluitend verlagen.
Vectoroverhead komt bovenop celtekst. Fallible modelallocaties zijn geen
harde native heapquota of bewijs van herstel bij iedere hostallocatiefout.
Geen nieuwe registrydependency, unsafe, shell-executie of netwerk.
Formuleachtige waarden worden letterlijk bewaard en zijn daarom **niet**
automatisch veilig om in een externe formule-evaluerende spreadsheet te openen.
Geen misleidende claim dat CSV-quoting formule-injectie oplost.

## Uitgevoerde controles

Werkdirectory
`/home/simon/.copilot/session-state/edf11e7c8b3e4c9ba1a91b7ef0779695/files/platform-csv`.
Rust 1.90.0, bestaande geisoleerde
`/tmp/nieuw-os-c-toolchain/cc-wrapper` met bijbehorende bestaande sysrootpaden;
`CARGO_TARGET_DIR="$PWD/target/platform-csv-host"`.
Geen installatie, dependencydownload of wijziging aan bestaande lockfiles.

| Exact commando | Resultaat |
|---|---|
| `cargo +1.90.0 generate-lockfile --offline --manifest-path sdk/examples/table-editor/Cargo.toml` | exit 0; alleen deze nieuwe dependencyvrije workspace |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/table-editor/Cargo.toml --features host-cli` | exit 0; 9 modeltests + 3 echte CLI-procestests |
| `cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/table-editor/Cargo.toml --features host-cli --all-targets -- -D warnings` | exit 0 |
| `cargo +1.90.0 fmt --manifest-path sdk/examples/table-editor/Cargo.toml -- --check` | exit 0 |
| `cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/table-editor/Cargo.toml --target x86_64-unknown-none` | exit 0, targetbibliotheek, geen native uitvoering |
| `python3 -B sdk/examples/table-editor/tests/interop.py target/platform-csv-host/debug/table-csv` | exit 0; 64 werkelijke CLI-outputs onafhankelijk geimporteerd met Python csv |
| `git diff --check` | exit 0 |

De eerste tien tests waren direct groen; dit is nieuwe functionaliteit,
geen geclaimde rood-naar-groenproductregressie. Bij toevoeging van twee tests
vond Clippy geneste niet-geregistreerde testfuncties; die zijn naar moduleniveau
verplaatst. De finale run voert daadwerkelijk alle twaalf tests uit.
De malformed-proef controleert 7776 vijfbytescombinaties; 64 celcombinaties
toetsen stabiele import/export, zonder het aantal fixtures als productscore
te behandelen.

Logs onder `build/platform-csv-evidence/`, SHA-256:

```text
0bf727bedacb3da079b7e159af2453c8c66d3c66d08d4cec43644f260322f7f1 final-gates.log
d5f1d678ecbd6904e38fd078666d3a3c29f55072aa783a65abc45dbaea9866a3 python-csv-oracle.log
```

Bronhashes:

```text
3e4f0833e8040087311d63087ae738a2b3054eae0e7ecffe3d878b01332d6975 src/lib.rs
24832f18a19c177aea8681ef55cde92f5d7a77d2d4d22628137c9ec8738ee2b6 src/main.rs
9aa71454ed4db8bc0a4cab0cd15904991fa54b3dec2227a790be9acbbdf53fdd tests/cli.rs
1d64896385bee5c63dde71144527937ad0731a83f0e6745da685a2ed526a7b92 tests/table.rs
```

SHA-256 van de werkelijk gebruikte artifacts:

```text
e6e7879f88277d06ba9eef4aa17bc1b7f37d7fdb6c6653a8d8be6c68c3b3d1f8 target/platform-csv-host/debug/table-csv
6535102595f17ac737da7bcf833613d755b7840db944547a6c9206803bd497cb target/platform-csv-host/x86_64-unknown-none/release/libnieuw_os_table_editor.rlib
```

## Overdracht en open grenzen

Geen native executable, GUI, venster, objectportal, geautoriseerde volume-save,
formules, undo, typed cells, XLSX, menselijke, hardware- of releaseacceptatie.
De host-CLI erft hoststdio; dit is geen Nieuw-OS-capabilityintegratie.
Native integratie vereist de bestaande app-/launch-/object-/UI-eigenaars,
niet een nieuwe parallelle broker. Scorevoorstel **41 -> 41/100**;
centrale beoordeling/status blijft bij General. Geen centrale write.

Na push/overdracht vervalt deze actieve schrijfclaim; de integrator verwerkt
de kleine slice parallel. Vervolgwerk begint op opnieuw opgehaalde main en
neemt deze nog niet geintegreerde table-editor niet als dependency.
Rollback: deze nieuwe subtree verwijderen; geen bestaande ABI of opslagformaat
wijzigt. De volwaardige native spreadsheet- en testreleaseopdracht blijft open.

# TASK-PLATFORM-M10-RECALC-CONSUMER

Basis `d10b5cdaf56774a39e3ef10463f674c6188a463a` (gefetched main),
branch `platform/spreadsheet-recalc-20261103`.
Geteste code-/test-/README-/lockfilebron
`26f45e5495014d23822593fe58d8c99de0588761`.
Vervolgcommit voegt uitsluitend deze receipt toe.
Scope: nieuwe `sdk/examples/spreadsheet/**` plus dit bewijs;
geen General-editor-, CSV-, calculator-, gedeelde workspace-, native ABI-,
kernel-, driver-, B02-, launch- of centrale score/statuswrites.
Geen afhankelijkheid van nog niet geintegreerde #165/#169/#173.

## Resultaat en bewijsgrens

Een werkelijk uitgevoerde hostconsumer stelt cellen in, berekent een formule
met A1-verwijzingen, wijzigt een broncel en berekent de afhankelijke cel opnieuw.
Geen nieuwe rekenkern: `nieuw-os-calculator::decimal` voert alle numerieke
bewerkingen uit, met bestaande negen fractieplaatsen/ties-to-even/rangechecks.
Afronding blijft zichtbaar door verwijzingen heen. Brontekst blijft behouden;
referenties worden als geparentheseerde numerieke waarden ingevuld, niet
opnieuw als formules uitgevoerd.

`no_std+alloc`-celmodel met maximaal 64 bezette cellen op A1..Z64,
256 bytes ASCII/cel en geexpandeerde formule, 32 gelijktijdige
afhankelijkheidsframes. Nieuwe cache per opvraag voorkomt verouderde resultaten.
Cyclus, ontbrekende cel, deling door nul, ongeldige literal/verwijzing,
range/depth/expansion/admissionfout zijn expliciet, nooit impliciet nul.
De README beschrijft het bewuste beperkte numerieke profiel.
Geen functions/ranges/absolute refs/macros of Office-/IEEE-pariteitsclaim.

Geen unsafe of nieuwe registrydependency. Het nieuwe consumerlockfile
hergebruikt de bestaande calculator/runtime/UI/font/zeroize-closure.
Geen gewijzigde bestaande lockfile. Fallible bron-/formulebuffers zijn geen
harde native heapquota. Hostprocessen erven hoststdio; het model geeft geen
bestands-, netwerk-, macro- of procesrechten.

## Exacte uitgevoerde controles

Werkdirectory
`/home/simon/.copilot/session-state/edf11e7c8b3e4c9ba1a91b7ef0779695/files/platform-sheet`.
Rust 1.90.0, bestaande `/tmp/nieuw-os-c-toolchain/cc-wrapper` met sysroot;
`CARGO_TARGET_DIR="$PWD/target/platform-sheet-host"`.
Geen installatie/download.

| Commando | Resultaat |
|---|---|
| `cargo +1.90.0 generate-lockfile --offline --manifest-path sdk/examples/spreadsheet/Cargo.toml` | exit 0; nieuw consumerlockfile |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/spreadsheet/Cargo.toml --features host-cli` | exit 0; 8 modeltests + 2 echte CLI-procestests, 0,380 s |
| `cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/spreadsheet/Cargo.toml --features host-cli --all-targets -- -D warnings` | exit 0, 0,136 s |
| `cargo +1.90.0 fmt --manifest-path sdk/examples/spreadsheet/Cargo.toml -- --check` | exit 0, 0,049 s |
| `cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/spreadsheet/Cargo.toml --target x86_64-unknown-none` | exit 0, 1,516 s; targetbibliotheek, geen uitvoering |
| `python3 -B sdk/examples/spreadsheet/tests/interop.py target/platform-sheet-host/debug/sheet-calc` | exit 0, 0,027 s; 32 echte herberekeningen + 5 afgeronde dependencyresultaten tegen onafhankelijke stdlib Decimal |
| `git diff --check` | exit 0 |

De eerste build faalde exit 101 door een Result-fouttypeverschil in de CLI.
Dat is expliciet hersteld zonder casts of errorfallback. Bij toevoeging van
de DAG-test kwam deze aanvankelijk per ongeluk genest in een andere test;
Clippy faalde exit 101. Test naar moduleniveau verplaatst en de finale run
voert aantoonbaar alle tien geregistreerde tests uit. Die fouten worden
niet als geslaagde controles of als oorspronkelijke productregressie gepresenteerd.

Tests omvatten diamond-herberekening, edit/wissen/herstel, negatieve
referentiewaarden en operatorvolgorde, zelf-/meercelcycli, ontbrekende refs,
deling door nul, bereikoverschrijding, afrondingspropagatie, ongeldige
adressen/tekst/opties, transactional admission, volledige capaciteit,
256/257-inputgrens, 32/33-framegrens en expansionbudget.
Een 30-cel dependency-DAG gebruikt dezelfde voorganger tweemaal en
herberekent ook na edit. Dit is geen gekwantificeerde native performancegate.
Een latere CLI-fout laat geen eerder berekende successtdout door.

Logs onder `build/platform-sheet-evidence/`, SHA-256:

```text
a2ec0f2b0844371dc4c94977a175e628d5553b28cc071ed9136cf5488639f565 gates.log (eerste compilefout, exit 101)
5a9b7b2853a0f18d8980ef16edc6d332a1e00e1b068cc0ad44ee65a2036420ac final-gates.log (geneste test/Clippy, exit 101)
a2f441a6684bdc1babfcbbb739b2c79ae6a790d6f92b14b4766acee43100cde0 accepted-gates.log (volledige finale keten, exit 0)
```

Gebruikte artifacts:

```text
1387832ce4bd9a7715e75918c2502401c3865747bed0f86d76c7702d72bfa6dd target/platform-sheet-host/debug/sheet-calc
e91839ff9ff7a468d0efcf517b7c5058391df91239fd5e70699df9b2ed70f99f target/platform-sheet-host/x86_64-unknown-none/release/libnieuw_os_spreadsheet.rlib
```

Exacte bronhashes:

```text
e75c2f7f3af5878e21b42c5e9602bef43c2a95e01f9761a2e4588b44a144dec2 src/lib.rs
203c71479bc2e1a7da083427fa22d4d0aa43b656639bff4c92147a5236942faa src/main.rs
1595afe7f16f7704e822ec856bcc0b78fc8729ecc15969728cdb2ebec19c0a09 tests/cli.rs
b3352ed374369434d4ed63739e3d4ad39c28bc8414733dc7dbe586b0d64f9772 tests/interop.py
b0e03f7cdb872d0138769c75f17a58deaea8da6736c98f98c758ad8a4b22f097 tests/recalc.rs
```

## Integratie, open grenzen en rollback

Hostfunctionaliteit plus no_std-compilatie. Geen native executable,
tabelvenster/keyboard/surface, gedeeld Work-documentmodel, tekst-/typed cells,
CSV/XLSX/ODS-import/export, geautoriseerde volume-save, undo, QEMU,
hardheapquota/OOM-herstel, hardware-/menselijke-/releaseacceptatie.
Volwaardige M10 en FR-A14 blijven open. Arithmetic-foutposities gelden voor
de geexpandeerde formule; geen verkeerde cursorbinding geclaimd.
Stdoutwrite is niet atomair. Integratiereview/nieuwe-head-CI blijven open.

Scorevoorstel **41 -> 41/100**; centrale beoordeling/status bij General.
Na push, draft-PR en overdracht vervalt deze actieve schrijfclaim;
volgende onafhankelijke milestone gebruikt opnieuw gefetchte main.
Rollback: verwijder uitsluitend nieuwe spreadsheet-subtree en receipt;
geen bestaande rekenkern, ABI of opslagformaat wijzigt.

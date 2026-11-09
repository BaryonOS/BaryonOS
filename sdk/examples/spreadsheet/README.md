# Numerieke spreadsheetconsumer

TASK-PLATFORM-M10-RECALC-CONSUMER, basis `d10b5cda`. Een kleine concrete
hostconsumer van de **bestaande** `calculator::decimal`-rekenkern:
cel instellen, formule met celverwijzingen opvragen, invoer wijzigen,
dezelfde formule opnieuw berekenen. Geen derde rekenkern, float of macrotaal.
Het model is `no_std + alloc`; de CLI is uitsluitend een hostprogramma.

```sh
cargo +1.90.0 run --offline --locked --manifest-path sdk/examples/spreadsheet/Cargo.toml --features host-cli --bin sheet-calc -- --set A1 2 --set B1 '=A1*3' --get B1 --set A1 4 --get B1
# stdout: 6, daarna 12 (ieder op een eigen regel)
```

`--set ADRES INVOER` en `--get ADRES` worden in volgorde uitgevoerd.
Een lege invoer `--set A1 ''` wist die cel. Een `--get` is vereist.
Fouten geven exit 1/stderr; stdout wordt pas geschreven nadat **alle**
acties slagen. Geen gedeeltelijke succeswaarden bij een latere formulefout.
Een stdout-I/O-fout kan wel een gedeeltelijke write opleveren.
Gebruik quotes rond formules zodat de hostshell ze niet zelf interpreteert.
Geen opslag, CSV-import of koppeling aan een nog niet geintegreerde branch.

## Expliciet ondersteund profiel

- Canonieke adressen `A1` t/m `Z64`; maximaal 64 gelijktijdig bezette cellen.
- Numerieke literal met optioneel teken, punt of komma; geen tekstcel.
- Formule begint met `=`: bestaande `+ - * / ( )`, numerieke literals en
  canonieke celverwijzingen. Geen functies, bereiken, absolute refs, namen,
  externe bestanden, macro's, datum-/geldtypes of netwerk.
- Bestaande decimale semantiek: negen fractieplaatsen, dichtstbijzijnde
  waarde met ties-to-even (bij gelijkspel naar even). Absoluut maximaal
  `9999999999,999999999`. Geen IEEE-/Excel-/XLSX-pariteitsclaim.
- Afronding blijft expliciet in `Evaluation::rounded`, ook als zij in een
  eerdere afhankelijkheid plaatsvond. De CLI toont `[afgerond]`.
- Een ontbrekende/gewiste cel is een fout, **niet** impliciet nul.
  Een cyclus, deling door nul, bereikoverschrijding, ongeldige literal of
  verwijzing levert geen opgeslagen/verzonnen resultaat op.

Invoer maximaal 256 ASCII-bytes zonder controletekens; spaties behouden.
Begrensde ongeldige formuletekst blijft bewaard om te kunnen corrigeren;
evaluatie geeft een expliciete fout met adres van de falende cel.
Niet-numerieke tekstcellen zijn nog niet gerealiseerd.
Een ongeldige admission/allocation wijzigt de vorige cel niet.

Iedere opvraag begint een nieuwe herberekening met een tijdelijke cache per
cel en cyclusdetectie. Verwijzingen worden als geparentheseerde **waarden**
aan de bestaande evaluator doorgegeven, nooit opnieuw als broncode/formules.
Daardoor veranderen negatieve waarden de operatorvolgorde niet.
Geen blijvende cache die na edit/wissen/foutreparatie verouderd kan raken.
Maximaal 32 gelijktijdige afhankelijkheidsframes; de bestaande evaluator
houdt daarnaast zijn eigen 32-nestingsgrens. Een geexpandeerde formule is
maximaal 256 bytes en kan dus eerder worden geweigerd dan de bronformule.
Foutposities van de decimale evaluator slaan op die geexpandeerde formule,
niet op een geclaimde cursorpositie in de oorspronkelijke cel.

## Resource- en integratiegrens

Celbronnen gebruiken fallible reserves: maximaal 16 KiB bronbytes plus
String-/celoverhead; tijdelijke formulebuffers maximaal circa 8 KiB langs
een afhankelijkheidspad, naast stack en cache. De host-CLI begrenst acties
tot 256. Dit is geen native hardheapquota-/OOM-/performancebewijs.

Geen unsafe of nieuwe registrydependency. De pathdependency hergebruikt de
bestaande calculatorpackage; het lockfile neemt diens bestaande runtime-,
UI- en exact gepinde font/zeroize-closure mee. Geen calculator-, renderer-,
workspace-, kernel-, IPC-, B02-, launch- of packagewrites.
De numerieke kern geeft geen bestands-, netwerk- of procesrechten.
De host-CLI erft uitsluitend de gewone hostprocess-/stdio-omgeving.

Een native tabelvenster, gedeeld Work-documentmodel, tekst-/typed cells,
CSV/XLSX/ODS-import/export, geautoriseerd opslaan, undo, selectie en
dependency-invalidationoptimalisatie blijven open. Geen volledige
spreadsheetapp of oorspronkelijke M10-acceptatie geclaimd.

## Gerichte acceptatie

```sh
# Uitvoeren vanaf de repositoryroot op een voorbereide Rust-toolchain.
set -e
export CARGO_TARGET_DIR="$PWD/target/platform-sheet-host"
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/spreadsheet/Cargo.toml --features host-cli
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/spreadsheet/Cargo.toml --features host-cli --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/examples/spreadsheet/Cargo.toml -- --check
cargo +1.90.0 build --offline --locked --manifest-path sdk/examples/spreadsheet/Cargo.toml --features host-cli --bin sheet-calc
cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/spreadsheet/Cargo.toml --target x86_64-unknown-none
python3 -B sdk/examples/spreadsheet/tests/interop.py target/platform-sheet-host/debug/sheet-calc
```

Tien tests omvatten echte processen, herberekening na edits/wissen, cycli,
ontbrekende verwijzingen, errorrepair, hergebruikte dependency-DAG,
operatorvolgorde, afrondingspropagatie en exacte limieten.
Python `Decimal` controleert onafhankelijk 32 werkelijke CLI-herberekeningen
en vijf afgeronde dependencyresultaten. Geen native uitvoering,
GUI-/hardware-/menselijke of releaseacceptatie.

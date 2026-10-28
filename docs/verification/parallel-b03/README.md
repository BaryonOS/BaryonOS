# B03 — eerlijke standalone-Rust-inventaris

Basis `41bac3c173d464a286017d31558335d70899c434`, 28 oktober 2026.

`preflight --strict` vereist werkende Git-, Rust-, Cargo- en QEMU-probes plus
firmware en beide bare-metal targets. Rustup blijft zichtbaar als optionele
toolchainmanager; ontbreken of falen wordt gerapporteerd maar vervangt de
controle van de werkelijk geselecteerde compiler niet.

Rustc en Cargo worden met `+1.90.0` bevraagd. De compiler moet daadwerkelijk
versie 1.90.0 melden. Zijn eigen `--print sysroot` en beide
`--print target-libdir --target ...`-antwoorden moeten bestaande, correcte
directories binnen die sysroot aanwijzen. Voor ieder target worden aanwezige
`libcore`, `liballoc` en `libcompiler_builtins` gecontroleerd als echte,
niet-afgekorte reguliere ar-archiefbestanden. De controle leest de archiefkop
en de begrenzing van het eerste lid, niet de complete Rust-metadata/ABI.
Symlinks naar buiten de sysroot worden geweigerd.

Een aanwezige rustup-manager wordt met `--toolchain 1.90.0` bevraagd. Zijn
targetlijst blijft apart gerapporteerd en kan ontbrekende compilerbestanden
nooit groen maken. Het rapport heeft schemaversie 2 en behoudt de bestaande
topniveauvelden. Dit is inventaris, **geen nieuwe build-, boot- of ABI-claim**.

## Controles

- **14 hosttestmethoden geslaagd**, inclusief subgevallen voor ontbrekende/
  falende vereiste tools, strict/nonstrict-exitbeleid, werkende standalone
  zonder manager, verkeerde compiler, mislukte/time-outende compilerqueries,
  ontbrekende targetdirectories of afzonderlijke libraries, corrupte/afgekorte
  archieven, verkeerd targetpad en symlinks buiten de sysroot.
- De bestaande vijf tests voor vereist-tool-faalbeleid zijn behouden met
  Cargo als vereiste tool; de gewijzigde managerkeuze wordt afzonderlijk getest.
- Echte `python3 -B tools/preflight.py --strict` op de bestaande officiële
  standalone-installatie: **exit 0**, `rustup.available = false`, Rust 1.90.0,
  UEFI- en none-targets aanwezig, `missing = []`.
- Gerichte bron-diffcontrole: **exit 0**.

Exacte commando's, bronhashes, stdout/stderr en exitcodes:
[report.json](report.json). De afzonderlijke echte inventaris staat in
[standalone-strict.stdout.txt](standalone-strict.stdout.txt).

Reproduceren met de gekozen compiler-/QEMU-/firmwareomgeving:

```sh
python3 -B -m unittest discover -s tests -p test_preflight.py -v
python3 -B tools/preflight.py --strict
```

Deze stap verwijdert een onjuiste afhankelijkheid van de toolchainmanager;
hij slaat geen target- of native acceptatiecontrole over. De eigenlijke
native suites worden door de hoofdagent op de geïntegreerde bron uitgevoerd.
Geen zelfstandig roadmapcriterium gesloten; scorevoorstel **14% → 14%**.

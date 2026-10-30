# Preview: geen oud succes bij mislukte voorbereiding

Basis `c532ba88fbf0f53b960e9dbf77421cee29d97096`; vervolgbranch
`simon/native-interactive-cold`. PR #66-head blijft tijdens de afzonderlijke
review op c532ba8 staan.

Concrete reviewbevinding: `run_preview` kon bij ongeldige build of firmware
falen voordat een nieuw rapport was geschreven. Daardoor bleef een oud
`preview-result.json` met `passed:true` als schijnbaar actueel resultaat staan.
De runner valideert nu eerst uitsluitend het eigen rapportpad via de bestaande
`private_path`-grens en verwijdert dat oude rapport voor alle invoervalidatie.
Er wordt geen ander bestand of oude runmap verwijderd. Een onveilig symlink-,
hardlink- of buiten-checkoutpad blijft geweigerd, niet gevolgd.

Nieuwe negatieve hosttests controleren ongeldige build, ongeldige firmware,
geen VM-start in beide gevallen en een rapport-symlink waarvan het doel
onaangeraakt moet blijven. Geen oorspronkelijke native bewijsbytes gewijzigd.

Uitgevoerd:

```sh
python3 -B -m unittest tools.tests.test_materiaal_preview tools.tests.test_package_windows_qemu tools.tests.test_materiaal_probe
python3 -B -m unittest discover -s tools/tests
git diff --check
python3 tools/progress.py --write
python3 tools/progress.py --check
```

Resultaat: 52 gerichte en 135 tools-tests, exit0; diff/progress exit0.
Logs `/tmp/nieuw-os-preview-stale-targeted.log` en
`/tmp/nieuw-os-preview-stale-tools.log`.
Nieuwe native/Windows-uitvoering op deze gewijzigde v3-bron: **niet uitgevoerd**.
De eerdere native/Windows-uitkomsten blijven aan hun eigen bron gebonden.
Integratiecoördinator beheert de overige packagingreviewpunten en integreert
deze kleine opvolgcommit afzonderlijk. Geen interface-/capabilitywijziging,
geen volledig productcriterium: **34% blijft34%; minstens50% open**.

# I09 — gerichte CodeRabbit-reparaties in preflight

## Bevindingen en resultaat

Deze taak repareert uitsluitend `tools/preflight.py` en zijn bestaande tests.
De baseline is commit `68c8cb76aaf905dcdd60d1e702203269e07aab2f`. De twee
concrete reviewbevindingen zijn eerst gereproduceerd en daarna opnieuw
gecontroleerd; er zijn geen gedeelde build-/launcherbestanden gewijzigd.

| Controle | Vóór de reparatie | Na de reparatie |
|---|---|---|
| Echte gepinde standalone Rust 1.90.0, `--strict` | Exit 2: Cargo weigert de selector; rustc-sysrootquery faalt | Exit 0; bare aanroepvorm geverifieerd en hergebruikt |
| Bestaande gepinde compatibility wrappers, `--strict` | Bestaande gebruikte launcher | Exit 0; `+1.90.0` blijft geselecteerd |
| Synthetische target-libdir naar ander target binnen dezelfde sysroot | Onterecht exit 0 | Exit 2 voor het gesubstitueerde target |
| Targetdir of één rlib naar ander target binnen dezelfde sysroot | Zelfde onvoldoende padgrens | Beide exit 2 |
| Geldige links binnen hetzelfde target | Toegestane lokale bestanden | Blijven geaccepteerd als inventarisatie |

`reproduction-before.json`, `standalone-before.json` en
`cross-target-before.json` bewaren het oorspronkelijke gedrag.
`reproduction-after.json` en beide `*-after.json`-rapporten bewaren het
gecontroleerde herstel. De echte standalone-/wrapperproeven lezen bestaande
officiële binaries en targetarchives; de symlinkproeven gebruiken eigen
synthetische tijdelijke mappen. Geen van deze controles is een VM-run.

## Aanroepvorm en versie

Preflight probeert voor Rust-tools eerst de gepinde selector. Wanneer de
selector wordt geweigerd, mag uitsluitend een opnieuw geverifieerde directe
Rust/Cargo 1.90.0-aanroep slagen. Een ander versienummer, een nightly-suffix,
crash of time-out geeft geen ongetoetste fallback.

Een belangrijke daadwerkelijke nuance: standalone rustc accepteert
`+1.90.0 --version` met exit 0, maar behandelt `+1.90.0` bij `--print sysroot`
als invoerbestand en faalt. Daarom controleert de selectie zowel de exacte
versie als een echte sysrootquery. De opgeslagen `toolchain_args` bepalen
vervolgens ook alle targetqueries. De uitgevoerde pogingen en hun uitkomst
blijven in het inventarisrapport staan.

## Targetidentiteit

De targetnaam blijft behouden voordat een `lib`-symlink wordt opgelost.
Een targetdirectory mag niet naar een andere locatie worden omgeleid; de
opgeloste librarydirectory en iedere geselecteerde rlib moeten binnen het
eigen target blijven. Daardoor kan bijvoorbeeld `uefi/lib -> none/lib` niet
meer slagen doordat beide vergelijkingspaden eerst naar hetzelfde verkeerde
target worden opgelost. Een geldige alias binnen hetzelfde target blijft
mogelijk. Archiveheader-/extentcontrole is nog steeds alleen inventarisatie,
geen bewezen inhoudelijke ABI- of linkcompatibiliteit.

## Uitgevoerde controles

**21 hosttests slagen: de 14 bestaande tests plus zeven gerichte aanvullingen.**
Zij dekken directe binaries, selectorcompatibele launchers, de rustc-version-
nuance, verkeerde versies, niet-Rust-tools, crash/time-outgedrag en target-
en rlibsymlinkgrenzen. De bestaande standalonetest gebruikt nu expliciet de
directe aanroepvorm; de selectorproef controleert ook de sysrootcapability.
Pythoncompilatie en diff-whitespacecontrole slagen.

Reproduceren:

```bash
python3 -B tests/test_preflight.py
python3 -B tools/preflight.py --strict
```

Voor de vastgelegde directe proef stond
`/tmp/nieuw-os-par-tools/rust/bin` vóór de compatibility wrappers in PATH.
Voor de wrapperproef is `/tmp/nieuw-os-par-tools/env.sh` gebruikt. Exacte
opdrachten, versies, paden en bronhashes staan in de JSON-bewijzen.

## Betekenis en resterende grens

Preflight blijft een **read-only inventaris**. Een geslaagde directe probe
maakt de overige buildtools niet automatisch geschikt voor bare compiler-
aanroepen. De bestaande volledige bouwketen vereist nog steeds de gepinde
launcher of de daadwerkelijk gebruikte `+1.90.0`-compatibility wrappers;
het inventarisrapport vermeldt die beperking expliciet.

Geen rootbron, bestaande native acceptatiebundel, kernel-, syscall-, IPC- of
opslag-ABI is gewijzigd. Dit voegt geen nieuwe native VM-functionaliteit of
roadmappunten toe; de eerdere 80-VM-acceptatie blijft bij broncommit `3e41bbf9`.

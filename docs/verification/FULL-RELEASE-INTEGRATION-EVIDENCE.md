# FR-I01 — archivering van gezamenlijk native integratiebewijs

Taakcontract, 29 oktober 2026. Eigenaar captureharnas: `native_ui_apps`;
hoofdagent bezit uitvoering, integratie en releaseacceptatie. Schrijfgebied:
`tools/capture_system.py`, `tests/test_system_capture.py` en dit document.
Dit document beschrijft geen geslaagde native uitvoering.

## Gebruik en bewijsgrens

Voer eerst op één bevroren bron `python3 -B tools/test_system.py` uit. Leg de
gemeenschappelijke broncommit/configuratie vast; behoud `build/run/` en de
finale normale build. Archiveer daarna met een nieuwe, herkenbare bestemming:

```sh
python3 -B tools/capture_system.py --destination docs/verification/full-release-<uniek>
```

De capture schrijft uitsluitend een nieuw direct kind van `docs/verification/`
waarvan de naam met `full-release-` begint. Bestaande bestemmingen, symlinks,
hardlinked invoer, apparaten en paden buiten de aangewezen checkout/build/run
worden geweigerd. Het harnas start geen VM en schrijft geen schijfimage.

## Verplichte hercontrole vóór archiveren

- Gate: afgerond en geslaagd; exact normal en recover; huidige bronvingerafdruk
  v3 en exact de actuele `test_system.inputs()`-hashes; geslaagde hostcontroles.
- Beide runs: eigen gegenereerde run-/imagedirectory, oorspronkelijke rawlogs,
  normale QEMU-exit en volledige herbeoordeling door `run_system.evaluate()`.
  Nieuwe verplichte guestcounters worden daardoor niet door een oude lokale
  kopie van de evaluatieregels overgeslagen.
- Herberekende kernel-/loaderhashes uit de werkelijk gebruikte ESPs, passende
  bron-/profielmanifesten en dezelfde normale kernel in de finale build.
- Werkelijke `volume.after.img`-bytes opnieuw door `volume_disk.analyse()` en
  `run_system.check_disk()`. Normal-after = recover-before = recover-after =
  de nog aanwezige gekoppelde image; recovery heeft echte readonly-QEMU-argumenten.
- Geen gewijzigde bronnen of invoerbestanden tijdens capture. Bronversie,
  exacte bytes van gate/rapporten/logs en herberekende hashes blijven bewaard.

Het archief bevat compacte JSON-/tekstbestanden, raw serial/stderr en
`SHA256SUMS`; geen kernels, firmware of schijfimages in Git. De bronartifacten
blijven in de gegenereerde buildmap voor onafhankelijke reproductie. Het
initiële firmware-VARS-templatehash blijft runnerinformatie; de actuele
VARS-copy kan tijdens de VM-run veranderd zijn en krijgt een afzonderlijke hash.

Host-harnastests gebruiken uitdrukkelijk synthetische fixtures. Zij bewijzen
uitsluitend dat foute of verouderde registraties worden geweigerd. Een archief
bewijst binnen de vertrouwde ontwikkelhost de gecontroleerde runregistratie;
het is geen cryptografische attestatie van een onbetrouwbare host, fysieke
hardwarebewijs of volledige testreleaseacceptatie.

## Uitgevoerde harnascontrole — 29 oktober 2026

Op basis `3536c71` zijn de nieuwe captureguards daadwerkelijk gecontroleerd met:

```sh
python3 -B -m unittest discover -s tests -p 'test_system_*.py'
python3 -B tools/capture_system.py --help
git diff --check
```

Resultaat: **27 hosttests geslaagd, exit 0**: 18 nieuwe capturetests en negen
bestaande evaluator-/imageguardtests. Ook Python-AST-syntaxcontrole is geslaagd.
De negatieve scenario's omvatten verouderde bron/ESP/finalbuild, onvolledige
gate, verkeerde raw exit, ontbrekende markers, timeout, corrupte echte
schijfbytes ondanks bijgewerkte sidecarhash, gewijzigd ongebruikt schijfgebied,
ontbrekende readonly, snapshotopties, bron-/invoerwijziging tijdens capture,
symlinks/hardlinks/padontsnapping, dubbele JSON-velden en overschrijven.

Volledige lokale harnasuitvoer: `build/capture-unit/{stdout,stderr}.txt` en
`build/capture-unit/result.json`. Gecontroleerde bronhashes:

- `tools/capture_system.py`: `ebfb0f7398e8e6e95ead635b9947c7d6dc3a1ee692adb6874c87fffd28c833f2`
- `tests/test_system_capture.py`: `d1bbb954a1faa17221493bc8c10f435931c836ee7bd5f5876376439d40b16e33`

De echte FR-I01-gate en capture op de uiteindelijke geïntegreerde bron zijn
in deze agenttaak **niet uitgevoerd**. De hoofdagent moet die na integratie
opnieuw uitvoeren; oudere gates missen deze bron- en testinputhashes.

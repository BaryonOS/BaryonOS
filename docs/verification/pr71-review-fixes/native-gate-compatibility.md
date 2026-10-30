# PR71 / PR96 native-replaycompatibiliteit

Bronnen:
- PR71: 3cf2fa9b5b51fdac84c9bd0eed964b5029606a59.
- PR96: d59bbec3c8b3607f5567b5a9c71cbf89ade4f713.
- Main: d791a342cc3bdb107245692f2f542d88d5fda7f1.
- CodeRabbit review 5717811668 op PR71, CHANGES_REQUESTED, 2026-10-30T19:42:02Z.

## Vastgesteld uit code

Main bevat nog geen tools/run_nas_native.py, tools/test_nas_native.py, tools/test_nas.py of native NAS-manifest. De namen van de afzonderlijke PR96-integratie zijn tools/build_nas.py, tools/test_nas.py en kernel/src/m2/m3/nas.rs. PR96 heeft bovendien services/nas/native-probe/disk_oracle.py; deze ontbreekt in PR71.

Huidige PR71-probe:
- cold_replay_token leest 40 bytes op startupoffsets 256 en 296.
- guest(recover) controleert canonieke handles; nulinput faalt op stap 47/48.
- Daarna READ met oude sessie en COMMIT met oude sessie/upload; beide verwachten Stale.
- De normale rol publiceert geen private export van het echte sessie-/uploadpaar voor de volgende boot.
- Het contract vermeldt expliciet dat rootbinding ontbreekt en de cold replay fail-closed blijft.

PR96:
- De baseline-probe gebruikt read.session = [42; 40] en doet slechts één synthetische cold read.
- Kernelmanifest vult bij gastinit alleen offsets 32,56,64,72,80,112,120,160,168,176; geen 256..336.
- De runner kopieert image en firmware, maar kent geen private handlepayload of startupinjectie.
- Zowel kernel als runner verwachten recoverytellers [(4,0),(4,4),(0,4)] voor admin/NAS/gast.
- De extra COMMIT-Stale-reply zou, na correcte handoff, [(4,0),(4,5),(0,5)] vereisen.
- Daarom is huidige PR71-probe niet compatibel met de vastgezette PR96 nativegate.

Dit is bronanalyse; de gecombineerde native kandidaat is hier niet gebouwd of uitgevoerd. Een eventuele fout op stap47/48 en daarna countermismatch is een afleiding uit de genoemde code, geen als uitgevoerd gelabelde QEMU-run.

## Aanbevolen afhandeling

Pas de drie huidige CodeRabbit71punten toe als beperkte vier-bestandenpatch. Alle nieuwe-head checks/review blijven vereist.

Houd native exact-tokenreplay expliciet open totdat een afzonderlijk, gezamenlijk profiel beschikbaar is:
1. Exact sessie/upload uit geslaagde boot1 Stage-reply exporteren via een afgebakende private testhandoff.
2. Manifest bron/run/rol/stage/volledigheid/lengte/hash binden; geen handles in Git, serial of publieke CI-artifacts.
3. Externe private readonly input in cold guest startup256..336 injecteren en integriteit voor/na controleren.
4. Kernel-/runnercounterassertions samen uitbreiden; inputweigering, herkomstafwijking, juiste oude sessie en afzonderlijke oude-uploadweigering behouden.
5. Werkelijke normal/cold uitvoering met exact bron- en artifactbinding; bestaande onafhankelijke diskoracle behouden.

Een expliciet apart opt-in replayprofiel kan de bestaande small-scope native baseline behouden zonder de strengere assertions weg te nemen. Dat vraagt een eigen ontwerp-/bouw-/testwijziging; het is niet opgenomen in de hier geleverde patch. Alleen de workflow toevoegen of counters versoepelen lost ontbrekende tokenherkomst niet op. Componentmerge PR71 krijgt geen native NAS- of volledige productpunten.

## Lokale reviewpatch

Zie manifest.json, pr71-fixes.patch, baseline-test.log en after-test.log. Uitsluitend .github/workflows/nas-core.yml, de Python-negatieftest, clientverificatie-README en zijn evidencehash wijzigen. Geen native code, oracle, kernel, SDK, Cargo of formatteringsscope gewijzigd.


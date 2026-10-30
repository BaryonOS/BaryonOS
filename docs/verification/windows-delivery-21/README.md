# Rol 21: Windows/QEMU-overdracht en routebewijs

30 oktober 2026, Europe/Amsterdam. Taak
`TASK-WINDOWS-DELIVERY-TAKEOVER`; afnemers 4/20/Simon.
Basis `ca45a7dabe6a040921de03bdc2fad3e5045a75ba`, branch
`simonvanieperen-symmetrical-lamp`. Dit is routebewijs met synthetische
bestanden, geen OS-gaststart, bundelaflevering of releaseacceptatie.

## Overdracht en actuele grens

Manager `89f84098-bd8c-4bc8-aeb2-0695517db26f` registreerde rol 21 en gaf
eerst uitsluitend deze eigen bewijsmap vrij. Build-/testbeheerder 12
(`b72f2536-b494-4a6b-a608-b6fd785e18ef`) droeg daarna formeel onder
`TASK-RUNNER-POOL-HANDOFF` de bestaande route over:
`tools/package_windows_qemu.py`,
`tools/windows-qemu/{Start-NieuwOS.ps1,Test-Launcher.ps1,README.md}`,
`tools/tests/test_package_windows_qemu.py` en eigen Windowsbewijs.
Rol 21 heeft die exacte scope bevestigd aan 12 en de manager.
Alle vijf routebestanden zijn ongewijzigd en blobgelijk met donorcommit
`bffe11fbc8c90d0e43b784c0a4ab580a111c12cc`; de gerichte Git-diff was leeg.
Geen `build.py`, `run.py`, CI, kernel, ABI of centrale voortgang overgedragen.
De native-peer `fbf60554-e98e-40bd-bd48-ee87a6498abe` is om actuele
eigendom/overdracht gevraagd; er is nog geen overdrachtsantwoord ontvangen.
Centrale status, queue en voortgangsmeter blijven manager-owned.

Release-agent `2ab17599-85c8-4409-ba06-a93c27485328` en beheerder 12
bevestigden afzonderlijk: er is geen nieuwe geintegreerde kandidaat of
Windows-artifact vrijgegeven. De historische ZIP voor bron `eabc27b`,
SHA-256 `b42be826377afb58f96c8178aa972128abad18ef7f9b4872e102a9e4a7e9ea57`,
is geen kandidaat voor deze opdracht en is niet opnieuw verpakt of herlabeld.
De NAS-deelprofielen zijn evenmin een interactieve Windows-kandidaat.

Een expliciet vrijgegeven **ontwikkelbuild** mag volgens de manager worden
afgeleverd voordat alle 62 volledige releasecriteria zijn gesloten.
Ontbrekende volledige releaseacceptatie is dus niet op zichzelf de blocker:
de ontbrekende concrete kandidaatvrijgave is dat wel.
Er wordt geen tweede launcher of packagingroute gebouwd.

## Werkelijk gecontroleerde Windows-omgeving

Via WSL-interoperabiliteit is het echte Windows-programma aangeroepen:

| Onderdeel | Waarneming |
|---|---|
| PowerShell | 5.1.26100.9756 |
| Windows-versietekst | Microsoft Windows NT 10.0.26220.0 |
| QEMU-pad | `C:\Program Files\qemu\qemu-system-x86_64.exe` |
| QEMU | 11.1.0, `v11.1.0-12130-ge470268ff4` |
| Display/backend | SDL en `rng-builtin` aanwezig |
| Uitvoeringsbeleid | CurrentUser `RemoteSigned`; overige scopes `Undefined` |

Geen uitvoeringsbeleid of beveiligingsinstelling gewijzigd. Dit zegt niets
over Linux-QEMU 10.2.1-runtimegedrag, een werkende Nieuw-OS-gast of algemene
gelijkwaardigheid van de platformen.

## Uitgevoerde commando's en resultaten

Vanuit de eigen repositoryroot:

```sh
python3 -B -m unittest tools.tests.test_package_windows_qemu
```

Exit 0: 10 synthetische hosttests, 0,036 s. De tests controleren onder meer
verouderde bron, gewijzigde app/ESP, extra ESP-inhoud, dubbele manifestkeys,
symlinks, uitvoerpadnormalisatie, bestaande ZIP en een artifactwissel
tijdens verpakking. Er is geen echte OS-bundel gemaakt.
Dezelfde suite is voor het bewijscheckpoint opnieuw uitgevoerd:
exit 0, 10 tests, 0,039 s, uitvoer in [`host-tests.txt`](host-tests.txt).
De Windows-uitvoer is met
`diff --strip-trailing-cr /tmp/nieuw-os-windows-launcher-current.log docs/verification/windows-delivery-21/windows-launcher.txt`
vergeleken (exit 0); alleen de Windows-regeleinden zijn genormaliseerd.

De bestaande launchertest is met onderstaande opdracht naar een unieke
lokale Windows-TEMP-map gekopieerd en op echte Windows PowerShell uitgevoerd.
Spaties in zowel stagingpad als QEMU-pad zijn daarmee werkelijk gebruikt.
De test maakt synthetische bundelbestanden; `-CheckOnly` start geen VM.

```sh
powershell.exe -NoProfile -NonInteractive -Command '$ErrorActionPreference = "Stop"; $source = "\\wsl.localhost\Ubuntu\home\simon\.copilotd\worktrees\wsl-Ubuntu\nieuw-os\simonvanieperen-symmetrical-lamp\tools\windows-qemu"; $root = Join-Path $env:TEMP ("Nieuw OS launcher check " + [guid]::NewGuid()); New-Item -ItemType Directory -Path $root | Out-Null; try { foreach ($name in @("Start-NieuwOS.ps1", "Test-Launcher.ps1")) { Copy-Item -LiteralPath (Join-Path $source $name) -Destination (Join-Path $root $name) }; & (Join-Path $root "Test-Launcher.ps1") -QemuPath "C:\Program Files\qemu\qemu-system-x86_64.exe"; if (-not $?) { throw "Launcher test failed" } } finally { foreach ($name in @("Start-NieuwOS.ps1", "Test-Launcher.ps1")) { $path = Join-Path $root $name; if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path } }; if (Test-Path -LiteralPath $root) { Remove-Item -LiteralPath $root } }' > /tmp/nieuw-os-windows-launcher-current.log 2>&1
```

Exit 0: vijf tests geslaagd; expliciet QEMU-pad, standaarddetectie,
gewijzigde kernelhash, ontbrekende kernel en geweigerd probeprofiel.
De tijdelijke scripts en synthetische testbestanden zijn opgeruimd.
Geen VM of persistente VM-state aangemaakt. Ruwe uitvoer:
[`windows-launcher.txt`](windows-launcher.txt); oorspronkelijk log
`/tmp/nieuw-os-windows-launcher-current.log`.

Twee voorafgaande hostprobe-opdrachten mislukten voordat QEMU werd getoetst:
een shellquote-fout (exit 2) en een ongeldige `.Source`-property op een
`FileInfo`-object (exit 1). De daaropvolgende expliciete QEMU-padcontrole
slaagde (exit 0). Deze hostprobe-fouten zijn geen gastfalen en zijn niet als
geslaagde controles meegeteld. Geen productiecode gewijzigd om ze te omzeilen.

De werkboom was voor en na deze proeven schoon op de bovengenoemde basis.
De onderstaande gehashte bestanden zijn de ongewijzigde bestaande route:

| Bestand | SHA-256 |
|---|---|
| `tools/package_windows_qemu.py` | `14192af1da2b889466da634bbe3db6c0cbdfc3ebf6d95bd3dc627cd9de74759a` |
| `tools/windows-qemu/Start-NieuwOS.ps1` | `a59a2650f5c247c1ea23145cd6da828041a9f9e27fe746f48431ee7b12424f99` |
| `tools/windows-qemu/Test-Launcher.ps1` | `b371d5d78e975d3d09c7525d87c31e4cc0808dac2f1e252a91510a0d8650a1d8` |
| `tools/windows-qemu/README.md` | `3accfa7d958713aae01ed574f4c43b0c14a4bfd1390ddfd0dc343d4a6f929a4a` |

## Open gates en exacte hervatstap

Eerstvolgend: ontvang van 4/release-agent/integrator 8 een expliciet
vrijgegeven, onveranderlijke ontwikkelkandidaat met broncommit,
bronvingerafdruk v3, buildmanifest, kernel-/loader-/imagehashes,
firmware/deviceprofiel, start-/stoproute en bekende beperkingen. Een ZIP-hash
wordt pas na daadwerkelijk verpakken vastgelegd; een bestaande ZIP is alleen
bruikbaar als die expliciet deel van de vrijgave is.
De routeoverdracht van 12 is inmiddels ontvangen; de manager is over de
exacte claim ingelicht. Een directe melding aan integrator 8
(`e8e471ce-0601-417b-b4fa-a04fd9c7b954`) werd door de messagingtool geweigerd
met `Remote priority admission timed out; message was not sent`.
Dit is geen bevestigde berichtaflevering of kandidaatvrijgave.

Daarna uitsluitend via de bestaande route:

1. Lees/kopieer de vrijgegeven artifacts read-only naar de eigen buildmap;
   controleer bron/manifest/apps/ESP en firmware voor en na verpakken.
2. Maak een nieuwe ZIP zonder teststate, leg bestandshashes en ZIP-hash vast,
   en controleer de daadwerkelijke uitgepakte inhoud en Nederlandse instructie.
3. Draai de normale Windows-launcher en interactieve gastproef op exact die
   bundel. Leg Windows-/QEMU-versie, argumenten, screenshots, invoer, logs,
   exit en onafhankelijke diskcontrole na volledige stop vast. Gebruik nieuwe
   eigen state; wijzig geen persoonlijke disks of beveiligingsbeleid.
4. Laat 20 gedrag beoordelen: start/readiness, navigatie/focus,
   invoeren/bewerken, opslaan/status, lijst/heropenen, koud hervatten en
   zichtbare fout/deny/cancel voor daadwerkelijk aanwezige functies.
   Ontbrekende functies blijven open kandidaatgates, geen verzonnen
   Windows-testfalen of succes. Laat 4 het brongebonden bewijs beoordelen
   en integrator 8 relevante wijzigingen integreren.

**Niet uitgevoerd:** kandidaatbuild/-verpakking, kandidaatmanifest-/
imagehashcontrole, normale nieuwe Windows-gaststart/bediening, koude recovery,
menselijke UX-review, fysieke hardware, signing of publieke publicatie.
FR-R04/FR-R01 en rubric R03/R01 blijven open.
Voortgangsvoorstel aan de manager: **34% -> 34%**; routebewijs sluit geen
volledig productcriterium. De gedeelde scorebestanden zijn hier niet aangepast.
`python3 -B tools/progress.py --check` is uitgevoerd: exit 0, 34/100,
rubric v1 consistent. Dit is alleen de bestaande registercontrole.

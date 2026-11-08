# Echte Windows-launchercontrole, zonder OS-start

30 oktober 2026; basis `19f2d33fe74f6735c6f727bca0e13472dc615d5e`
met de bijbehorende `CheckOnly`- en integriteitscontrolewijziging.

De door mij gekoppelde map `C:\Program Files\qemu` is bereikbaar vanuit WSL.
Echte Windows PowerShell meldt `CurrentUser: RemoteSigned`; alle overige
policy-scopes zijn `Undefined`. Er is geen uitvoeringspolicy gewijzigd of
overgeslagen. QEMU meldt:

```text
QEMU emulator version 11.1.0 (v11.1.0-12130-ge470268ff4)
```

`-object help` toont `rng-builtin`; `-display help` toont `sdl`.
Deze drie Windows-QEMU-probes zijn geslaagd (exit 0).

## Uitvoering

Direct `-File` vanaf de WSL-UNC-share werd door de bestaande RemoteSigned-
policy geweigerd (exit 1). De eigen twee scripts zijn daarna met `Copy-Item`
naar een nieuwe lokale Windows-tempmap met spaties gekopieerd. Daar werkt
dezelfde policy zonder aanpassing. Na de tests zijn uitsluitend de benoemde
synthetische bestanden en lege testmappen verwijderd.

De daadwerkelijke testaanroep in die map:

```powershell
.\Test-Launcher.ps1 -QemuPath "C:\Program Files\qemu\qemu-system-x86_64.exe"
```

Uitkomst (Windows PowerShell, exit 0):

```text
Bundle integrity and QEMU 11.1.0 prerequisites passed; no VM started.
Bundle integrity and QEMU 11.1.0 prerequisites passed; no VM started.
PASS: 5 Windows launcher tests; synthetic files only; no VM started.
```

De vijf controles toetsen expliciete QEMU-pathdetectie, standaarddetectie,
weigering van een gewijzigde kernel met dezelfde bestandsgrootte, weigering
van een ontbrekende kernel en weigering van een automatisch probeprofiel als
interactieve bundel. De twee normale controles gebruiken `-CheckOnly`:
geen VM gestart, geen `state\` aangemaakt. Alle hashrecords komen uit
synthetische testbestanden; dit bewijst geen OS-imageacceptatie.

`python3 -B -m unittest tools.tests.test_materiaal_preview tools.tests.test_package_windows_qemu tools.tests.test_materiaal_probe`
slaagt aanvullend met 31 hosttests (exit 0).

## Bewijsgrens

Geen SDL-venster, werkelijk Windows-gastscherm, handmatige OS-bediening,
ZIP-installatie of Windows-opslagherstel getest. De launcher controleert
bestandsgrootte en SHA-256 tegen het bundelmanifest; het manifest is geen
handtekening of vertrouwde distributieketen. De interactieve integratiebron
wordt door de andere sessie nog afzonderlijk geaccepteerd. Geen ZIP gemaakt,
geen bronvervanging, geen nieuwe roadmapacceptatie: **34% → 34%**.

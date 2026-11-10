# WINDOWS-INTERACTIVE-1: echte Windows-QEMU-proef

**Geslaagd, 30 oktober 2026.** De gewone zevenprocessenvariant is werkelijk
gestart onder Windows-QEMU 11.1.0 met SDL. Openen, typen, opslaan, lijst en
heropenen zijn via echte QMP-toetsen/muis uitgevoerd. Alle zes volledige
800x600-PPM's zijn bytegelijk aan de geaccepteerde native Linux-referentie:
dit is hier afzonderlijk getoetst, niet uit het Linuxbewijs afgeleid.
Geen pixelmaskers, F12, controller of diagnostische snapshots.
De gast is na de laatste fase met QMP `quit` gestopt: raw exit **0**,
duur **102,445 s**. Pas daarna is de actieve wegwerpschijf onafhankelijk
gedecodeerd: object 0/revisie 1, `Nieuw document`, `nieuw os werkt` (14 bytes),
generatie 1, domeinklokken `[1, 0]`, geldige checksums, andere objecten/
collecties leeg en alle grensmarkeringen intact.

## Uitvoering op de betrokken Windows-desktop

De schone ZIP en een reeds uitgepakte kopie staan lokaal:

```text
C:\Users\simon\Downloads\NieuwOS-testversie-eabc27b-20261030\
  NieuwOS-Windows-Interactive-1.zip
  NieuwOS\Start-NieuwOS.ps1
  TESTBEWIJS.txt
```

Open Windows PowerShell en voer uit:

```powershell
& "C:\Users\simon\Downloads\NieuwOS-testversie-eabc27b-20261030\NieuwOS\Start-NieuwOS.ps1"
```

`C:\Program Files\qemu\qemu-system-x86_64.exe` wordt automatisch gevonden.
De eerste rendering duurde in de bewijsrun circa 22 s; ook volgende
schermovergangen waren niet onmiddellijk. Maak met **Nieuw** een document,
typ tekst en gebruik **Opslaan** of Ctrl+S. Esc gaat terug naar de lijst;
Tab/Tab/Enter heropent in deze vaste eenvoudige lay-out het opgeslagen
document. Sluit de VM via het QEMU-venster.

De geleverde map bevat **geen** teststate of geschreven proefdocument:
de gewone launcher maakt eigen `state\` aan. De hardware-invoerproef gebruikte
een aparte map onder Windows Temp; die wordt niet door de launcher hergebruikt.
Herstart/koudherstel is in deze Windows-opdracht niet geaccepteerd.
De schijf is uitsluitend een synthetische ontwikkelimage, geen persoonlijke
opslag. Geen echte schijf, hostmap, netwerkkaart, credentials of clipboard
zijn gekoppeld. De uitvoeringspolicy is niet aangepast: CurrentUser bleef
`RemoteSigned`, de andere scopes `Undefined`.

ZIP-grootte: **2.296.965 bytes**. SHA-256:

```text
b42be826377afb58f96c8178aa972128abad18ef7f9b4872e102a9e4a7e9ea57
```

Dit is een lokale testbundel, geen publicatie, installer of release.
Het onveranderde README/manifest **in** de bevroren ZIP bevat nog de
conservatieve tekst van vóór deze Windows-proef. Dit aparte bewijsdocument
legt de nieuw uitgevoerde acceptatie vast voor uitsluitend bovenstaande
hash; de productiepackager/launcher/README zijn niet aangepast.

## Bron- en artifactbinding

Geïntegreerde eigen branch `simon/windows-qemu-test-version`, checkpoint
`c532ba88fbf0f53b960e9dbf77421cee29d97096`; exacte native codebron
`eabc27ba879fd3b5960863ae8eede800c5045a57`, v3:

```text
a1998a126a3a1ccc18523925a422912d934c14bab865b3d7df346862a2c251a6
kernel f2e9cc4aad87ad2b9fbc8b498ab146db2ab0df4a77b904eecf81c3a9beb198b3
loader 7e55ab0fe6c232380f52dc5be1334b41388b70d80e0ebe2b0072de7e12373a09
```

De integrator leverde alleen-lezen
`.../simonvanieperen-studious-sniffle/build/native-test-version/interactive-current`.
Voor en na kopiëren zijn alle ESP-, manifest-, buildreport- en zeven
app-ELF-bestanden gehasht; de eigen kopie onder
`build/windows-interactive-1/build` is vervolgens met de bestaande
`package_windows_qemu.validate_build` volledig gevalideerd. Manifest,
apprecords/ELF-inspecties, embedded appbytes en bronvingerafdruk zijn gelijk.
Er is hier **niet** opnieuw gebouwd of in de andere werkboom geschreven.
`build-binding.json` bewaart de exacte paden, hashes en zeven apprecords.

De bestaande packager is ongewijzigd gebruikt met expliciete reguliere
OVMF/SeaBIOS-bestanden en copyrightnotices. Gepinde pakketten:
OVMF `2025.11-3ubuntu7.3`, SeaBIOS `1.17.0-1ubuntu1`. De vier firmwarehashes
zijn gelijk aan de vaste native acceptatie; `package-validation.json`
legt dat, de ZIP-CRC-controle en de herhaalde bron/buildcontrole vast.
`package.json` bewaart alle gebundelde bestandshashes en het afleverpad.
Hashes bieden herleidbaarheid en corruptiedetectie, geen handtekening.

## Windowsplatform en afgebakende adapter

Windows-QEMU rapporteerde werkelijk:

```text
QEMU emulator version 11.1.0 (v11.1.0-12130-ge470268ff4)
```

Machine `pc-q35-8.2`, `qemu64`, TCG single-thread, 1 CPU, 256 MiB,
SDL, PS/2, `rng-builtin`, modern virtio-blk/rng, OVMF en read-only ESP.
Linux gebruikte QEMU 10.2.1/`pc-q35-10.0`/`rng-random`/headless:
dit zijn verschillende platformgrenzen. Hier zijn de zes framebytes,
hardware-invoerketen en geschreven disk apart onder Windows gecontroleerd.
Dit bewijst geen algemene gelijkwaardigheid van beide platforms/RNG's.

`windows_qmp_adapter.py` staat uitsluitend in deze bewijsmap, buiten de
v3-bronvingerafdruk. Hij controleert eerst bundelhashes, exacte bron/kernel/
loader, de gecommitte native referentieframehashes en echte PowerShell
`-CheckOnly`. Hij leest de letterlijke argumentlijst van de normale launcher
en start het echte Windows-QEMU-programma via WSL-interoperabiliteit.
Alleen de eigen statebestandsnamen verschillen en `-qmp stdio` is expliciet
toegevoegd. Geen TCP-QMP-listener, verborgen launcheroptie, policy-bypass,
netwerk/fallback of vervangende OS-functionaliteit. De normale launcher
zelf is hier met `-CheckOnly` uitgevoerd; de SDL-gaststart gebeurde via deze
expliciete adapter met dezelfde productieargumenten plus QMP.

QMP-handshake en echte gastreadiness zijn tijdens de run gecontroleerd.
Iedere fase vereiste het volledige passende native referentiebeeld vóór
volgende input. De trace bevat 918 gecorreleerde opdrachten; herhaalde
screendumps zijn alleen observaties, geen extra toetsen of fasevervangers.
Na `quit`/exit 0 volgt het bestaande onafhankelijke Python-schijforakel.
Bron, QEMU-programma en bundelbytes zijn bij afloop opnieuw gecontroleerd.

De start- en heropenbeelden zijn aanvullend door de agent bekeken:
native Materiaal met Nieuw/Vernieuwen en de heropende documenttekst.
Dit is **geen onafhankelijke menselijke visuele/UX-audit** en voegt geen
algemene kwaliteitsacceptatie toe.

## Uitgevoerde commando's en resultaten

Onderstaande paden worden vanaf de eigen repositoryroot gebruikt.
`FIRMWARE` wees op de alleen-lezen firmwaremap van de integrator;
`BUNDLE` is de eigen lokale Windows-wegwerpmap. De kopieer/bytebindingstap,
versiecontrole via `dpkg-deb -f` en finale bestandshashcontrole zijn
eveneens daadwerkelijk uitgevoerd, exit 0; hun uitkomsten zijn hierboven
en in de JSON-records vastgelegd.

```sh
git fetch origin simon/native-test-version
git merge --ff-only c532ba88fbf0f53b960e9dbf77421cee29d97096

FIRMWARE=/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-studious-sniffle/.tools/nieuw-os/qemu/usr/share
python3 -B tools/package_windows_qemu.py \
  --build-dir "$PWD/build/windows-interactive-1/build" \
  --output "$PWD/build/windows-interactive-1/NieuwOS-Windows-Interactive-1-pinned.zip" \
  --ovmf-code "$FIRMWARE/OVMF/OVMF_CODE_4M.fd" \
  --ovmf-vars "$FIRMWARE/OVMF/OVMF_VARS_4M.fd" \
  --seabios-bios "$FIRMWARE/seabios/bios-256k.bin" \
  --vga-rom "$FIRMWARE/seabios/vgabios-stdvga.bin" \
  --ovmf-license "$FIRMWARE/doc/ovmf/copyright" \
  --seabios-license "$FIRMWARE/doc/seabios/copyright" \
  --ovmf-version 2025.11-3ubuntu7.3 --seabios-version 1.17.0-1ubuntu1

BUNDLE='/mnt/c/Users/simon/AppData/Local/Temp/Nieuw OS Interactive 1 7444ffab9ac749d59cd191c53c2dcb2f/bundle'
python3 -B docs/verification/windows-qemu-interactive-1/windows_qmp_adapter.py \
  --bundle "$BUNDLE" \
  --reference "$PWD/build/windows-interactive-1/reference" \
  --output "$PWD/build/windows-interactive-1/windows-run-2"

python3 -B docs/verification/windows-qemu-interactive-1/recheck.py \
  --run "$PWD/build/windows-interactive-1/windows-run-2" \
  --reference "$PWD/build/windows-interactive-1/reference" --bundle "$BUNDLE"

python3 -B -m unittest tools.tests.test_package_windows_qemu tools.tests.test_materiaal_preview
python3 -B -m py_compile docs/verification/windows-qemu-interactive-1/windows_qmp_adapter.py docs/verification/windows-qemu-interactive-1/recheck.py
git diff --check
```

Alle bovenstaande commando's: **exit 0**. Hosttests: **29 tests**,
1,807 s; geen nieuwe native tests door die hosttesttelling.
De adapterrun: **exit 0**, QEMU **raw 0**. De afzonderlijke hercontrole:
**exit 0**, exacte QMP-volgorde/antwoorden, zes bytegelijke frames,
geen F12, lege voordisk, actieve disk gelijk aan nadisk en inhoud opnieuw
gedecodeerd. Finale aflevering controleerde ZIP- en alle uitgepakte hashes
en het ontbreken van teststate. Vier echte PowerShell-`CheckOnly`-aanroepen
slagen: eerste bundel, beide adaptervoorbereidingen (ook de hieronder
beschreven poging die nog vóór VM-start stopte) en schone Downloads-kopie.

Een afzonderlijk lokaal Python-hostharnas heeft daarnaast zes kopieën van
de echte bewijsbestanden in tijdelijke mappen gemuteerd. `recheck` weigert
raw33, een booleaanse QMP-ID, een ontbrekende fase, een geïnjecteerde
F12-toets, een gewijzigd systeemrandpixel en een gewijzigde nadisk.
Alle zes tegenvoorbeelden slagen als **weigering** (harnas exit 0);
`recheck-negatives.json` bewaart de precieze foutmeldingen.
Dit zijn synthetische hostmutaties, geen zes nieuwe gastproeven.

De afzonderlijke PowerShell-aanroepen vanaf WSL:

```sh
/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe -NoProfile \
  -File 'C:\Users\simon\AppData\Local\Temp\Nieuw OS Interactive 1 7444ffab9ac749d59cd191c53c2dcb2f\bundle\Start-NieuwOS.ps1' -CheckOnly
```

Na kopiëren/`Expand-Archive` in de nieuwe Downloads-map is daar eveneens
`Start-NieuwOS.ps1 -CheckOnly` uitgevoerd, gevolgd door `Get-FileHash`
en `Get-ExecutionPolicy -List`; alle exit 0, uitvoer in `delivery.txt`.
Er is geen `-ExecutionPolicy`-override gebruikt.

Lokale volledige logs:
`/tmp/nieuw-os-windows-interactive-1-package-pinned.json`,
`/tmp/nieuw-os-windows-interactive-1-checkonly.log`,
`/tmp/nieuw-os-windows-interactive-1-run-2.log`,
`/tmp/nieuw-os-windows-interactive-1-recheck.json`,
`/tmp/nieuw-os-windows-interactive-1-host-tests.log`,
`/tmp/nieuw-os-windows-interactive-1-delivery.log`.

## Bewaarmethode, eerdere fouten en open grenzen

`windows-result-summary.json` behoudt het echte rapport zonder de grote
ingesloten QMP-trace, met oorspronkelijke rapport-/trace-/frame-/diskhashes,
alle daadwerkelijke hardwareinputs en exacte platform/bronbinding.
Raw serial, lege stderr en CheckOnly-uitvoer zijn hier opgenomen.
Volledige oorspronkelijke rapporten, 918-opdrachtentrace, zes PPM's,
PNG-weergaven en voor-/nadisk blijven lokaal in
`build/windows-interactive-1/windows-run-2`; de actieve Windows-proefdisk
blijft in de gerapporteerde Temp-map. Binaries/disks/ZIP's zijn niet in Git
opgenomen. `recheck.py` herrekent uit deze echte bestanden; dit is een
afzonderlijke hostcontrole, geen tweede native run of menselijke audit.

Twee hostvoorbereidingsfouten zijn niet als geslaagd bewijs gebruikt:

- De eerste shellpoging las een gebroken OVMF-changelog-symlink. Omdat de
  shell die fout niet blokkeerde, produceerde de packager exit 0 met een
  lege OVMF-versietekst. Dat archief is **afgekeurd en verwijderd**; de
  gecorrigeerde nieuwe ZIP gebruikt gecontroleerde pakketmetadata,
  heeft een andere hash en is de enige geleverde bundel. Geen productiecode
  of validatievoorwaarde is versoepeld.
- Eerste adapterpoging exit 1 vóór VM-start: de strikte argumentparser
  verwachtte ten onrechte één string per regel, maar de launcher bevat
  meerdere. De adapter accepteert nu uitsluitend reeksen letterlijke
  strings; daarna is met nieuwe state de bovengenoemde echte run uitgevoerd.
  Log `/tmp/nieuw-os-windows-interactive-1-run.log`; geen gastfalen.

Niet uitgevoerd: handmatige bediening door de projecteigenaar, koudherstel/herstart,
procesopruimingsbewijs, controller-/snapshotproeven, fysieke hardware,
installatie, internet/NAS, uitgebreide beveiligingsaudit of volledige
productreleaseacceptatie. De automatische bewijsrun is al afgesloten.
Er is niets gepubliceerd of ondertekend.

Voortgangvoorstel voor de integrator: **34% -> 34%**. Deze opdracht levert
een concreet lokaal testbaar Windows-artifact en platformbewijs, maar sluit
geen volledig nieuw gewogen productcriterium. Gedeelde status/rubric/README
blijven conform de exclusieve schrijfgrens bij de integrator.

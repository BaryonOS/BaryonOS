# N1 — native backendtoets op bestaande eigen kernel

Datum: 2 november 2026. Bron: `74a34d05ef5591b782a0da9da34e4669910d0722`.
Source fingerprint v3: `8fd739dd015b9d01908ccc0e385f5b584985cade44702f6e6662483f4be0b6d3`.

**Uitkomst:** de eigen kernel en NOSVOL02-herstelroute werken in deze proef;
Btrfs en OpenZFS zijn niet native geïmporteerd en hun inhoud is niet gelezen.
De native backend-adoptiepoort blijft gesloten.

| Proef | Waargenomen resultaat | Bewijsgrens |
|---|---|---|
| Eigen NOSVOL02 normal | PASS; raw QEMU exit 33; oracle generatie 84 | Bestaande synthetische eigen volumeproef |
| Eigen NOSVOL02 cold recover | PASS; exit 33; generatie en beide domeindigests gelijk | Nieuwe VM/firmwarevariabelenkopie, identieke bytes van de normal-eindimage; alleen lezen |
| Btrfs regular file, 256 MiB | Exit 35; APP_RESULT, step 20; nul gevulde tabelregio's | Bestaande NOSVOL02-code accepteerde de onderzochte regio als leeg; verwacht proefdocument ontbreekt. Geen Btrfs-herkenning of inhoudsherstel |
| OpenZFS mirrorlid, 256 MiB | Exit 35; SERVICE_STARTUP; status -18/Corrupt | NOSVOL02-parser wijst bytes af. Geen ZFS-import, mount of inhoudsherstel |

Alle vier runs haalden BOOT:EBS_OK, KERNEL:ENTRY, BOOTINFO:OK en
VOLUME:MANIFEST:PASS. Geen run overschreed zijn timeout. Buitenlandse images
werden alleen gekopieerd naar deze werkmap, QEMU kreeg `readonly=on`, de guest
had READ-only blokcapabilities en `-nic none`. Zowel bronimage als proefkopie
hebben vóór en na de run dezelfde volledige SHA-256.

## Concrete gate failure: lege header is geen formaatidentiteit

De kernel geeft uitsluitend sectoren 4 tot en met 583 vrij
(`kernel/src/m2/m3/volume.rs`, regels 19–20, 309–312). NOSVOL02-recovery leest
de eerste 68 sectoren van die scope. `table.rs`, regels 828–846, accepteert twee
nul gevulde tabellen als `Table::EMPTY`. De Btrfs-image had in deze regio nul
bytes, zodat de dienst startup niet als onbekend formaat weigerde. De app
faalde vervolgens bij de eerste STAT van een verwacht NOSVOL02-document,
stap 20. Dit is een waarneming plus de bijbehorende bronverklaring; er is geen
Btrfs-parser opgeroepen.

Dit betreft een vertrouwde synthetische fixture. Er is geen productinstaller,
autoformatter of fysieke schijf getest. De consequentie voor het ontwerp is
wel concreet: deze toelatingsregel mag geen generiek beleid voor externe of
bestaande volumes worden. Schrijfrecht vereist een positieve, gevalideerde
formaatidentiteit en een afzonderlijke expliciete formatteeractie. Deze
opdracht verandert de driver of parser niet.

## OpenZFS-beperking

De geteste image is `nieuwos_n3.0a`, één lid van een mirror met twee regular
file-vdevs. N3 maakte die pool met OpenZFS 2.4.1 ztest/libzpool en controleerde
hem met een nieuwe zdb-processtart. Het betreft DMU-testobjectsets, geen normale
ZPL-boom met gebruikersbestanden. Eén mirrorlid aanbieden aan de bestaande
single-device NOSVOL02-broker is alleen een toets van omgang met vreemd
formaat. Native import, meerdere vdevs, datasetlezing, snapshots en ZFS-herstel
blijven onbewezen.

## Bouw en herleidbaarheid

Er is opnieuw gebouwd uit de ongewijzigde actuele bron, met Rust 1.90.0 en
de bestaande targets `x86_64-unknown-none` en `x86_64-unknown-uefi`.
QEMU: 11.1.0, `pc-q35-10.0`, TCG, één CPU, 256 MiB.
Geen oude previewbinary is gebruikt.

Bij de oorspronkelijke e08-proef vond de eerste offline build een bestaande locked SDK-workspacedependency
niet in de cache. Daarna zijn uitsluitend bestaande locked dependencies naar
`../native/cargo-cache` opgehaald. Root- en SDK-lockhashes bleven gelijk.
`../native/dependency-fetch/*-license-provenance.json` bevat pakketnaam, versie, bron en
licentiemetadata. Er is geen nieuwe dependency gekozen en geen lockfile
gewijzigd. Native builds blijven offline.

`builds/*/build-invocation.json` en de onderliggende logs bewaren commando's en
uitkomsten. `runs/*/result.json` bevat exacte QEMU-argv, bronfingerprint,
firmware-/loader-/kernelhashes, bron-/proefimagehashes, tijden en exitstatus.
De gekopieerde loader en kernel zijn vóór boot vergeleken met het buildmanifest.
Serial en stderr zijn onbewerkt opgeslagen. `N1-result.json` bundelt de
uitkomsten en houdt backendacceptatie expliciet op false.

Uitgevoerde hoofdcommando's (vanuit PowerShell):

```powershell
python -B native_probe.py build --profile native-volume
python -B native_probe.py build --profile test-volume-recover
python -B native_probe.py control --timeout 300
python -B native_probe.py foreign --label btrfs-readonly --image ../btrfs/baseline.img --timeout 120
python -B native_probe.py foreign --label openzfs-leaf-readonly --image ../openzfs/images/nieuwos_n3.0a --timeout 120
python -B native_probe.py summarize
```

De bestaande runlabels weigeren overschrijven. Voor een herhaling hoort een
nieuwe afzonderlijke werkmap te worden gebruikt. Herstel van deze proef is
het bewaren of later opruimen van uitsluitend deze wegwerpartifacten; er zijn
geen repository-, hostschijf-, firmware- of systeemconfiguratiewijzigingen
terug te draaien.

## Herproef op huidige remote-main-bron

Deze map bevat nieuwe builds en nieuwe VM-runs op commit 74a34d05ef5591b782a0da9da34e4669910d0722. De eerdere e08-bewijzen onder ../native zijn intact behouden. De sourceworktree current-main-source is vóór en na de voltooide builds/runs schoon gecontroleerd. Alleen de bestaande locked Cargo-cache is hergebruikt; nieuwe target-, build- en runmappen staan hier. Root- en SDK-lockfiles zijn bytegelijk aan de dependency-fetchbasis; zie dependency-cache-reuse.json.

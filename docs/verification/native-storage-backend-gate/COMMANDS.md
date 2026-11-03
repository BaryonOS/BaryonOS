# Actuele controle en begrensde host-replay

Alle commando's hieronder worden vanuit de repositoryroot uitgevoerd.
Historische `*.historical.txt`-bestanden zijn alleen brondata, nooit
entrypoints. Geen installatie, acquisitie, hostdisks of gebruikerdata.

## Gecommitte archiefcontrole (standaard)

```sh
python3 -B docs/verification/native-storage-backend-gate/verify_evidence.py
python3 -B -m unittest discover -s tests -p test_storage_backend_replay.py -v
```

De eerste opdracht controleert alle 207 oorspronkelijke archive-hashes,
JSON-invarianten en twee apart gepinde originele Btrfs-manifestreceipts
(82+17 entries); geen lokale images/binaries zijn nodig.
`verification.json` vermeldt modus `committed`, `local_payloads_checked: false`
en `new_native_execution: false`. De originele, modusloze uitslag blijft
bytegelijk als `verification.json.historical.txt`. De verifier overschrijft
geen rapport: optionele `--output` vereist een nog niet bestaand bestand.

## Volledig lokaal historisch payloadbewijs (niet uitgevoerd)

```sh
python3 -B docs/verification/native-storage-backend-gate/verify_evidence.py \
  --evidence-root /EXPLICIET/GEKOZEN/LOKALE/BEWIJSMAP
```

De map bevat de oorspronkelijke relatieve layout, inclusief beide lokale
`btrfs/sha256-manifest.json` en `btrfs/metadata/sha256-manifest.json`, de
OpenZFS-artifacts uit `openzfs/artifact-sha256.json`, en per native run
`data.img`, `esp/kernel.elf`, `esp/EFI/BOOT/BOOTX64.EFI`.
Voor readonly-bronimages gebruikt de verifier uitsluitend de expliciete
portable aliassen `source-images/{native,native-current}/{btrfs-readonly,openzfs-leaf-readonly}.img`.
Hij volgt nooit de historische absolute Windows-`source_image`.
Ontbrekende of afwijkende payloads falen, links en padontsnappingen ook.
De twee volledige originele manifesten zijn nu bytegelijk bewaard als
`btrfs/sha256-manifest.json.historical.txt` en
`btrfs/metadata/sha256-manifest.json.historical.txt`, met onafhankelijk
vastgelegde SHA-256 en entryaantallen. De lokale manifesten moeten precies
dezelfde bestandsnamen en hashes bevatten; beide worden vóór payloadchecks
vergeleken. Ook een niet-leeg manifest met één geldig bestand faalt.
De gepinde metadata-alias `../metadata_probe.py` blijft na lexicale
normalisatie binnen de bewijsroot; ongebonden paden worden niet vertrouwd.
Ontbrekende payloadbytes zijn niet uit receipts of samenvattingen te
reconstrueren. De modus heet `local-payloads`; ook dit is geen nieuwe VM-run.

## Offline voorbereiding (geen foreign binary uitvoeren)

```sh
python3 -B tools/storage_backend_replay.py --backend btrfs \
  --workspace "$PWD/.storage-replay-fresh-btrfs" --packages /EIGEN/GECONTROLEERDE/BTRFS-PAKKETMAP
python3 -B tools/storage_backend_replay.py --backend openzfs \
  --workspace "$PWD/.storage-replay-fresh-zfs" --packages /EIGEN/GECONTROLEERDE/ZFS-PAKKETMAP
```

Deze voorbeelden zijn **geen uitvoering in dit reviewwerk**. Kies iedere keer
een nieuwe, niet bestaande werkmap binnen deze checkout; een bestaande
werkruimte wordt nooit hervat of hergebruikt. Alleen de eigenaar mag de map
benaderen (0700); symlinks, hardlinks, vreemde eigenaars, schrijfbare
ancestors en afwijkende packageverzamelingen worden geweigerd.
De invoermap bevat precies één Btrfs-pakket of zes OpenZFS-pakketten uit de
in-code allowlist. SHA256-pins komen uit de ongewijzigde oorspronkelijke
`btrfs/toolchain.json` en `openzfs/logs/package-sha256.txt`.
Dit controleert exacte historische bytes, **geen nieuw onafhankelijk
repositoryhandtekening-/publisherbewijs**. Version-auth alleen is onvoldoende.

Alle pakketten worden eerst naar private kopieën gekopieerd en gehasht.
Daarna controleert de hosttool `/usr/bin/dpkg-deb` de naam/versie/architectuur
en levert een tarstream. Python pakt die begrensd uit: geen speciale files,
absolute paden, `..`, cyclische links of afwijkende filecollisions. Interne
packagealiases worden reguliere kopieën, geen filesystemlinks. Er worden
geen maintainer-/installatiescripts uitgevoerd. Ontbrekende hosttools of
compressieondersteuning: stoppen, niet installeren.

## Optionele nieuwe Btrfs-baseline (niet uitgevoerd)

Voeg uitsluitend na lokale beoordeling `--execute` aan de Btrfs-opdracht toe.
Die creëert één synthetische 256-MiB-image en controleert check/restore,
payloadinhoud en ongewijzigde bronhash. Geen mount, loopdevice, `/dev/*`,
VM, corruptie-/metadataacceptatie of productpunten. Preflight vereist
aanwezige executables, ldd-exit 0, geen `not found`, exacte helpstatus en
verwachte helptekst. Geen pipeline of laatste-logcommandostatus als oracle.

Subprocessen krijgen elk een eigen process group, stdin gesloten,
exclusieve logs, timeout, specifieke groupcleanup en leader-reapreceipt.
Naamgebaseerd killen is verboden. De hosttests oefenen uitsluitend
projectfixtures/Pythonprocessen; zij vervangen geen foreign-engineacceptatie.

**OpenZFS `--execute` wordt altijd geweigerd.** De ongewijzigde upstream
`ztest` heeft een hardcoded `/tmp`-runtimefile. Een uitsluitend project-lokale
create-/crashisolatie ontbreekt; zie [de primaire bron en grens](openzfs/README.md).
De oude scripts gebruiken als omweg is nadrukkelijk geen veilige replay.

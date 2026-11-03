# Primaire bronnen en bewijsgrenzen

Bronnen hieronder zijn geraadpleegd op **3 november 2026**. Een `latest`- of
`master`-documentatiepagina is niet als vastgezette release voorgesteld.
Deze bronverwijzingen vullen het dossier aan; de historische tekstsnapshots,
logs en meetresultaten zijn niet gewijzigd.

| Titel | Auteur/organisatie | Versie/datum | Primaire URL en betekenis |
| --- | --- | --- | --- |
| mkfs.btrfs(8) | btrfs-progs-project | Documentatie `latest`, geraadpleegd 3 november 2026; gebruikte proefbinary 6.17.1-1build1 | [mkfs.btrfs](https://btrfs.readthedocs.io/en/latest/mkfs.btrfs.html): `--rootdir` vult het filesystem vanuit een directory; de documentatie noemt hardlinkdetectie. |
| btrfs-check(8) | btrfs-progs-project | Documentatie `latest`, geraadpleegd 3 november 2026; gebruikte proefbinary 6.17.1-1build1 | [btrfs-check](https://btrfs.readthedocs.io/en/latest/btrfs-check.html): standaard read-only controle; repair is een afzonderlijke ingrijpende modus, niet onze nieuwe replayroute. |
| btrfs-restore(8) | btrfs-progs-project | Documentatie `latest`, geraadpleegd 3 november 2026; gebruikte proefbinary 6.17.1-1build1 | [btrfs-restore](https://btrfs.readthedocs.io/en/latest/btrfs-restore.html): `-m` betreft owner/mode/tijden, `-x` xattrs en `-S` symlinks. |
| QEMU invocation | The QEMU Project | Documentatie `master`, geraadpleegd 3 november 2026; proefruntime QEMU 11.1.0 | [Invocation](https://www.qemu.org/docs/master/system/invocation.html): machine-, accelerator-/TCG- en blockdevice-opties. De runtimeversie volgt uit de proefrecords, niet uit deze veranderlijke URL. |
| ISA debug exit implementation | The QEMU Project | Release-tag v11.1.0 | [debugexit.c](https://raw.githubusercontent.com/qemu/qemu/v11.1.0/hw/misc/debugexit.c): primaire implementatiebron voor de guestwaarde naar procesexit `(waarde << 1) \| 1`; geen gebruikersmanual of bewijs dat een guestproef slaagde. |
| ztest(1) | OpenZFS-project | Documentatie `master`, geraadpleegd 3 november 2026; geen documentatierelease vastgezet | [ztest](https://openzfs.github.io/openzfs-docs/man/master/1/ztest.1.html): regressietool met blockfiles; de upstream standaard gebruikt `/tmp`, reden om nieuwe uitvoering zonder passende isolatie niet toe te laten. |
| zdb(8) | OpenZFS-project | Documentatie `master`, geraadpleegd 3 november 2026; geen documentatierelease vastgezet | [zdb](https://openzfs.github.io/openzfs-docs/man/master/8/zdb.8.html): offline inspectie rechtstreeks vanuit userspace; geen Nieuw-OS-port of native backendacceptatie. |
| Nieuw OS-kernel en native-volume-probe | simonvanieperen / Nieuw OS-project | Bronversies e08fb0e48c1bb0781e98357f588c2f73b6ae8500 en 74a34d05ef5591b782a0da9da34e4669910d0722 | [e08-bron](https://github.com/simonvanieperen/nieuw-os/tree/e08fb0e48c1bb0781e98357f588c2f73b6ae8500) en [74a34d05-bron](https://github.com/simonvanieperen/nieuw-os/tree/74a34d05ef5591b782a0da9da34e4669910d0722): eigen implementatie; de run-/buildrecords leggen vast welke bron werkelijk is getest. |

## Metadata: semantiek is geen gemeten roundtrip

De Btrfs-handleidingen onderbouwen de bedoeling van `mkfs --rootdir` en
`btrfs restore -m -x -S`. Zij bewijzen **niet** dat die route algemeen alle
metadata verliest, noch dat onze specifieke nanoseconde-mtime-,
directory-xattr- of hardlinkwaarnemingen voor elke Btrfs-versie gelden.
Die afgebakende conclusie komt uitsluitend uit
[metadata-evidence.json](btrfs/metadata/metadata-evidence.json), de daar
vastgelegde binary/packagehash, bron-/restorevergelijking en logs.
Voor die oorspronkelijke proef is geen volledige metadata-roundtrip
aangetoond. De huidige baseline-replay heeft deze metadataproef niet herhaald.

## Native en foreign storage

QEMU-opties en upstream tools beschrijven mechanismen, geen geslaagde
Nieuw-OS-acceptatie. De eigen bronversies, daadwerkelijke exits, beelden,
serial-/tracebytes en schijfhashes horen bij de afzonderlijke proefrecords.
Een upstream Btrfs/OpenZFS-hostproef of veilige afwijzing van een foreign
image bewijst geen native filesystemondersteuning of teruggevonden inhoud.
De huidige OpenZFS-uitvoerroute blijft geblokkeerd; de historische negatieve
resultaten zijn niet als nieuwe uitvoering herlabeld.

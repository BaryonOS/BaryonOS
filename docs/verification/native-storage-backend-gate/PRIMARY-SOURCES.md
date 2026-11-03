# Primaire bronnen en bewijsgrenzen

<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:c0e61ac819bc29e1c53d60eaeb6e96d54675d4fbee561a055b09845a7cb30486 -->

Bronnen hieronder zijn geraadpleegd op **3 november 2026**. Versiegebonden
bronnen zijn gekoppeld aan de gebruikte toolversies; de QEMU-documentatie is
niet als vastgezette release voorgesteld.

<!-- /coderabbit-ai-generated:v1 -->
Deze bronverwijzingen vullen het dossier aan; de historische tekstsnapshots,
logs en meetresultaten zijn niet gewijzigd.

<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:dbeaf24ae9bde8eafdbc4b1b475ac40a5897ddfe64398e8fa7a17d25e8b4ec55 -->

| Titel | Auteur/organisatie | Versie/datum | Primaire URL en betekenis |
| --- | --- | --- | --- |
| mkfs.btrfs(8) | btrfs-progs-project | v6.17.1; proefbinary 6.17.1-1build1 | [mkfs.btrfs](https://github.com/kdave/btrfs-progs/blob/v6.17.1/Documentation/mkfs.btrfs.rst): `--rootdir` vult het filesystem vanuit een directory. |
| btrfs-check(8) | btrfs-progs-project | v6.17.1; proefbinary 6.17.1-1build1 | [btrfs-check](https://github.com/kdave/btrfs-progs/blob/v6.17.1/Documentation/btrfs-check.rst): read-only controle en afzonderlijke repairmodus. |
| btrfs-restore(8) | btrfs-progs-project | v6.17.1; proefbinary 6.17.1-1build1 | [btrfs-restore](https://github.com/kdave/btrfs-progs/blob/v6.17.1/Documentation/btrfs-restore.rst): `-m` voor owner/mode/tijden, `-x` voor xattrs en `-S` voor symlinks. |
| QEMU invocation | The QEMU Project | `master`, geraadpleegd 3 november 2026; proefruntime 11.1.0 | [Invocation](https://www.qemu.org/docs/master/system/invocation.html): opties voor machine, TCG en blockdevices; de URL legt de runtimeversie niet vast. |
| ISA debug exit implementation | The QEMU Project | v11.1.0 | [debugexit.c](https://raw.githubusercontent.com/qemu/qemu/v11.1.0/hw/misc/debugexit.c): guestwaarde naar procesexit `(waarde << 1) \| 1`; geen bewijs van een geslaagde guestproef. |
| ztest(1) | OpenZFS-project | 2.4; proefbinary 2.4.1-1ubuntu5.1 | [ztest](https://openzfs.github.io/openzfs-docs/man/v2.4/1/ztest.1.html): userspace-regressietool met blockfiles en upstream `/tmp`-gebruik. |
| zdb(8) | OpenZFS-project | 2.4; proefbinary 2.4.1-1ubuntu5.1 | [zdb](https://openzfs.github.io/openzfs-docs/man/v2.4/8/zdb.8.html): offline inspectie, geen bewijs van native backendacceptatie. |
| Nieuw OS-kernel en native-volume-probe | simonvanieperen / Nieuw OS-project | e08fb0e48c1bb0781e98357f588c2f73b6ae8500 en 74a34d05ef5591b782a0da9da34e4669910d0722 | [e08-bron](https://github.com/simonvanieperen/nieuw-os/tree/e08fb0e48c1bb0781e98357f588c2f73b6ae8500) en [74a34d05-bron](https://github.com/simonvanieperen/nieuw-os/tree/74a34d05ef5591b782a0da9da34e4669910d0722): de run- en buildrecords verbinden de proeven aan de bron. |


<!-- /coderabbit-ai-generated:v1 -->
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

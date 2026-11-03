# Btrfs — historisch bewijs en begrensde herhaling

De [oorspronkelijke toelichting](README.md.historical.txt), zes
`*.py.historical.txt`-bronnen, logs en receipts blijven bewijs van de
oorspronkelijke hostproef. Dit zijn niet-uitvoerbare tekstsnapshots; voer ze
ook niet met `python` uit. Vaste WSL-/gedeelde werkpaden en ongecontroleerde
packagehergebruik waren ongeschikt voor een nieuwe uitvoering.

De actuele route staat in [COMMANDS.md](../COMMANDS.md). Eén expliciet gekozen,
nieuwe private werkruimte vervangt alle oude BASE/OUT-/OUT.parent-paden.
Alleen het exact gepinde pakket wordt offline gecontroleerd en uitgepakt.
De optionele nieuwe Btrfs-hostproef controleert uitsluitend een kleine,
synthetische baseline en restore; corruptie- en metadataproeven zijn niet
opnieuw geïmplementeerd of uitgevoerd. Zij blijven historische resultaten,
geen claims over de nieuwe runner.

<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:1f3a13ceb32e5feea8640e76cffaf9e067c9ca33dae0935a620b126612fa7010 -->

De historische `payload-mapping.json` is alleen lokaal beschikbaar en niet in
het archief opgenomen. Het oorspronkelijke `sha256-manifest.json` is als
[`sha256-manifest.json.historical.txt`](sha256-manifest.json.historical.txt)
bewaard; de payloads waarnaar het verwijst zijn niet allemaal gearchiveerd.

## Primaire documentatie

Btrfs-progs 6.17.1-1build1 is getest; onderstaande bronnen zijn van het
btrfs-progs-project, versie 6.17.1:

- [mkfs.btrfs(8)](https://github.com/kdave/btrfs-progs/blob/v6.17.1/Documentation/mkfs.btrfs.rst): `--rootdir` importeert de bronmap.
- [btrfs-check(8)](https://github.com/kdave/btrfs-progs/blob/v6.17.1/Documentation/btrfs-check.rst): controles en de optie voor datachecksums.
- [btrfs-restore(8)](https://github.com/kdave/btrfs-progs/blob/v6.17.1/Documentation/btrfs-restore.rst): herstel met `-S`, `-x` en `-m`.

[Overige primaire bronnen en bewijsgrenzen](../PRIMARY-SOURCES.md).

<!-- /coderabbit-ai-generated:v1 -->

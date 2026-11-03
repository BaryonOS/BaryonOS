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

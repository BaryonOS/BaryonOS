# OpenZFS-commando-inventaris — historische bron, geen uitvoerinstructie

De [oorspronkelijke inventaris](COMMANDS.md.historical.txt) bevat twee
verwisselde logkoppelingen. Onderstaande koppeling volgt de daadwerkelijk
bewaarde scriptbytes; logs worden niet hernoemd en geen nieuwe uitvoering
wordt gesuggereerd:

| Historische tekstsnapshot | Werkelijke logbestemming |
|---|---|
| `03-create-unsupported-tunable.sh.historical.txt` | `logs/create-final*` |
| `03-create-low-as.sh.historical.txt` | `logs/create-run*` |
| `03-create-initial-help-mismatch.sh.historical.txt` | `logs/create.txt`, `create-time.txt`, `create-exit.txt` |
| `03-create-invalid-dedup.sh.historical.txt` | `logs/create-retry*` |
| `03-create.sh.historical.txt` | `logs/create-bounded*` |

De overige historische commandobindingen staan ongewijzigd in de
oorspronkelijke inventaris. Die tekst beschrijft wat toen werd vastgelegd,
niet wat de oude scripts veilig bewijzen: de oude preflight kon falen
maskeren; de kopiestap berekende hashes maar vergeleek ze niet; de vaste
zdb-GUID was niet herhaalbaar; stap 11 hing voor `images/` af van stap 04.

De gedeelde `/tmp`-workspace, glob-extractie en redirects uit die bronnen
zijn buiten gebruik gesteld. Gebruik uitsluitend de [actuele route](../COMMANDS.md).
Create/crashuitvoering is daar expliciet geblokkeerd; er bestaat geen
stilzwijgende fallback naar de historische scripts.

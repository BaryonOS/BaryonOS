# B07 — herbruikbare G0-capture

## Resultaat en bronbinding

`tools/capture_graphics.py` controleert een bestaande, voltooide G0-run en
publiceert een nieuw tekst-/PNG-archief. De capture draait geen VM en herstelt
geen ontbrekende of veranderde resultaten. De hoofdagent voert de integratie,
een nieuwe G0-run en de capture daarvan uit op de samengevoegde bron.

Basiscommit: `3e41bbf93822f7044c6020259f4a752925e1966d`.
Werkbranch: `codex/parallel-g0-capture-20261028`.
Geteste source-v3:
`e5d1c07736039e087ed80d3da44e689fdf6afaecb6558a719c8af08d0b204612`.
`execution.json` bevat de exacte bestandshashes, commando's, tijden en
exitstatussen. Alleen de nieuwe capture, de bijbehorende tests en deze
bewijsmap zijn gewijzigd. Bestaande G0-runner, framechecks, kernel en oude
bewijsmappen zijn ongewijzigd.

## Reproduceren

Voer de CLI uit vanuit de repository, met de echte runmap die de bestaande
`tools/run_graphics.py` op de actuele bron heeft opgeleverd:

```sh
python3 -B tools/capture_graphics.py \
  --run-dir /absolute/path/to/completed/graphics-g0-run \
  --destination docs/verification/new-g0-capture
```

De bestemming moet een nieuwe directe submap van `docs/verification` zijn.
Een bestaande map wordt ook bij een gelijktijdige creatie niet vervangen.
De CLI gebruikt alleen de Python-standaardbibliotheek en vereist Linux met
werkende `renameat2(RENAME_NOREPLACE)`-ondersteuning op het bestandssysteem.
Ontbrekende ondersteuning geeft een fout; er is geen overschrijvende fallback.

De toegevoegde capture verandert de source-v3-hash. Een oudere G0-run kan
daarom niet als bewijs voor deze nieuwe bron worden gearchiveerd. De eerdere
bevestigde native run en de expliciet ongeldig verklaarde oorspronkelijke run
blijven ieder onder hun oorspronkelijke bronbinding bewaard.

## Gecontroleerde grenzen

De capture vereist de volledige positieve G0-checkset, ruwe QEMU-exit 33,
geen gerapporteerde fout, de actuele source-v3-hash, een bijbehorend
graphics-g0-manifest en hashes van de werkelijk gelezen kernel en loader.
Alle genoemde artifacts worden opnieuw gelezen. De bytes van `serial.txt`
moeten exact gelijk zijn aan de gerapporteerde uitvoer; een afgebroken log kan
dus niet uit `result.json` worden aangevuld.

De commandoregel moet exact overeenkomen met een toegelaten private G0-profiel
zonder netwerk of extra devices. De elf QMP-opdrachten en antwoorden moeten
hun gehele volgorde, numerieke ID's, toetsen en eigen screenshotpaden behouden.
De bestaande actuele `frame_checks`, `ppm` en `valid_sequence` controleren de
vijf echte PPM's opnieuw, inclusief de volledige backspace-gelijkheid,
wijzigingen binnen de invoerregel en de kernelmarkers na `BOOT:EBS_OK`.
Een geclaimde VGA-ROM-pin moet gelijk zijn aan de werkelijk bewaarde ROM.

Onverwachte artifacts, ontbrekende bestanden, symlinks, hardlinks, andere
bestandstypen en te grote bestanden worden geweigerd. Bestandshandles gebruiken
`O_NOFOLLOW` voor iedere padcomponent. Vlak vóór publicatie worden alle
invoerbytes, bestandsidentiteiten, de QEMU-entry, bronhash en staged output
opnieuw gecontroleerd. De geteste destination-race maakt daadwerkelijk een lege
map aan vlak vóór de rename; die map blijft behouden.

Het nieuwe archief bevat vijf originele tekst-/JSON-bestanden, vijf lossless
PNG's, `image-roundtrip.json` en `capture.json`. De beeldindex bewaart per beeld
de oorspronkelijke PPM-header en beide hashes. De PNG wordt teruggelezen en
de header plus RGB-bytes moeten de volledige oorspronkelijke PPM reconstrueren.
Kernel-, loader-, ROM-, firmware- en diskbytes worden niet naar het archief
gekopieerd; hun hashes blijven in de tekstindex staan.

Dit is integriteitscontrole van lokale, door de runner geregistreerde inputs.
Het is geen cryptografische attestatie van een QEMU-proces of bescherming tegen
een onbeperkt bevoegde aanvaller die de bron en alle bewijsbestanden vervalst.
Het vastgelegde basiscommit-ID wordt op formaat gecontroleerd; de actuele
bronbinding wordt met source-v3 en het manifest gecontroleerd.

## Uitgevoerde controles

Alle onderstaande commando's zijn op 28 oktober 2026 uitgevoerd met exitstatus
0. Ruwe stdout/stderr en metadata staan naast dit document.

| Controle | Resultaat | Bewijsgrens |
|---|---:|---|
| `python3 -B -m unittest discover -s tests -p 'test_graphics*.py' -v` | 33 methoden geslaagd: 22 nieuwe capturetests en 11 bestaande G0-harnastests | Synthetische hostfixtures; geen framebuffer-, mapping- of VM-bewijs |
| `python3 -B docs/verification/parallel-b07/check_standard_png.py` | Vier resoluties, alle RGB-bytes exact teruggelezen door Pillow 12.3.0 | Onafhankelijke standaarddecoder op synthetische pixels; Pillow is geen runtime-afhankelijkheid |
| `python3 -B tools/capture_graphics.py --help` | Exit 0 | CLI-invoer en help beschikbaar |

De negatieve gevallen omvatten gewijzigde of afgebroken serial, ROM/ELF/
firmware/PPM-wijzigingen, opnieuw gehashte foutieve markers en frames, verkeerde
manifestbinding, QMP-ID's/paden/toetsen, netwerk/extra devices, dubbele
JSON-velden, niet-eindige getallen, ontbrekende/extra artifacts, symlinks/
hardlinks/FIFO's, bestaande/ontsnappende bestemmingen, inputwijzigingen tijdens
de capture, gewijzigde bron, een vervangen staged bestand en ontbrekende
no-replace-ondersteuning.

De standaarddecoder controleerde 1×1, 3×2, 96×224 en 800×600 pixels. De fixtures
bevatten nadrukkelijk synthetische ELF-/firmwarebytes en synthetische
serialmarkers; geen daarvan wordt als bootresultaat aangeboden. Nieuwe native
uitvoering van de capture is bij deze oplevering nog niet uitgevoerd.

## Voortgang en primaire bronnen

Voorstel voor het vaste scoreblad: **0 punten wijziging**. Betere bewaring en
controle van G0-bewijs sluiten geen aanvullend OS-roadmapcriterium af. De
hoofdagent verwerkt deze beoordeling en de uiteindelijke native integratiestatus.

- W3C, *Portable Network Graphics (PNG) Specification (Third Edition)*,
  Recommendation 24 juni 2025:
  <https://www.w3.org/TR/2025/REC-png-3-20250624/>. De writer gebruikt RGB8,
  filter 0, geen interlace, IHDR/IDAT/IEND, zlib-compressie en PNG-chunk-CRC's.
- Linux man-pages project, *rename(2)*, geraadpleegd 28 oktober 2026:
  <https://man7.org/linux/man-pages/man2/rename.2.html>. Publicatie gebruikt
  `RENAME_NOREPLACE`; ondersteuning is een expliciete hostvoorwaarde.

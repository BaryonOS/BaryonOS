# Eerste G0-uitvoering — later bewaard artifact ongeldig

**Niet geaccepteerd als eindbewijs.** Zie invalidated.json: de oorspronkelijke
bewaarde serialfile wijkt af van de directe archiefkopie. Beide waarnemingen
blijven intact; nieuwe geïsoleerde runs volgen in ../parallel-final-g0-confirmed/.

Bron: `3e41bbf93822f7044c6020259f4a752925e1966d`; fingerprint v3
`307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60`.

De echte private QEMU-VM eindigt met raw33 en alle serial-/pixelchecks groen.
800×600, stride800, BGR, één CPU,256MiB, TCG/qemu64, expliciet pc-q35-7.2.
G0 gebruikt eigen post-EBS/kerneluitvoer en echte QMP-toetsen a,b,backspace,c.
Iedere overgang verandert alleen de invoerregio; backspace verandert de
opname na b en herstelt bytegelijk de eerdere opname na a. De vijf originele
PPM-hashes staan in acceptance.json/result.json. Het oorspronkelijke runpad
bevat de snapshots; zij zijn geen browser- of gegenereerde designbeelden.

De SeaBIOS-ROM wordt expliciet gekopieerd en gepind op SHA256
`cc2f735f19b6318922ac3de9506dee498f149a6b75534f7e5c176d4441a7fa4a`.
Een opzettelijk verkeerde pin wordt met ValueError vóór QEMU-start afgewezen.
Het juiste bestand, de firmware en kernel/loader zijn in het native rapport
gehasht. Officiële packageherkomst: ../parallel-i01/qemu-toolchain-source.json.

Uitvoering: execute.py met de omgevingsvariabelen in het centrale
PARALLEL-NATIVE-ACCEPTANCE.md. Ruwe serial, stderr, buildmanifest en resultaat
zijn ongewijzigd gekopieerd. Source en HEAD bleven voor/na gelijk. Dit is G0
(RAM-invoer), geen native Materiaal-documentflow, compositor of fysieke
hardwareacceptatie. De negatieve ROM-pin is een hostcontrole, geen extra VM.

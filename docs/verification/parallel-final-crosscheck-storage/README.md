# B06 — onafhankelijke controle van qsim- en G0-bewijs

**Vervolg:** de afzonderlijke controle van een nieuwe G0-run is geslaagd.
Zie [SECOND-CHECK.md](SECOND-CHECK.md) en `second-check.json`. Onderstaande
eerste bevinding en alle bijbehorende bestanden blijven als historie behouden;
de oude G0-run blijft uitgesloten van de eindacceptatie.

**Blokkade in het bewaarde G0-rawlog; qsim en de G0-pixel-/binarycontroles
slagen.** Deze controle las de daadwerkelijk bewaarde bestanden opnieuw en
voerde geen build, VM, brede testsuite of bronwijziging uit.

Verwachte uitvoeringsbron is commit
`3e41bbf93822f7044c6020259f4a752925e1966d`, fingerprint v3
`307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60`.
Die HEAD en fingerprint waren tijdens deze controle aanwezig. `verify.py`
berekent zelf SHA-256's, leest de ELF-header en P6-pixels, en vergelijkt de
ruwe serialregels met de vaste grenzen. Het schrijft alleen zijn eigen
`result.json`; het start alleen `git rev-parse HEAD` als subprocess.

## Concrete blokkade

In `build/parallel-final-g0/run/graphics-g0-5ad6321b1d/serial.txt` ontbreken
de laatste regels `G0:PASS` en `M1:PASS`. Het bestand telt 409 bytes en heeft
SHA-256 `53b8af732872d41510183073a89b610fcdaed00348ed44648819111dc55c2d68`.
Het bewaarde runresultaat en het tekstarchief bevatten wel 425 bytes, met
SHA-256 `f61350dd1041ed8736a36452f1f4e56821f05f4f11008c9036e327913bfe2596`.
Daardoor falen de actuele rawloghash, bytegelijkheid met het archief en de
volledige native eindregelreeks. Het positieve opgeslagen resultaat is
onvoldoende om dit gewijzigde rawartifact te accepteren.

De eerste controle liep op 2026-10-28 van 15:44:48.784 tot 15:44:49.222 UTC.
`first-check.json` bewaart die uitkomst: 154 van 157 controles slagen; de drie
genoemde rawlogcontroles falen. `g0-serial-observed-mismatch.txt` is een
afzonderlijke, ongewijzigde kopie van de aangetroffen 409 bytes.

`forensics.json` bewaart bestandshashes, inodes en nanosecondetijden. Het
afwijkende rawbestand heeft mtime 15:37:23.296 en ctime 15:37:25.608; de
B06-scriptfile is pas om 15:44:48.526 aangemaakt. De G0-afwijking bestond dus
al vóór de uitvoering van deze controle. Geen eigen negatieve serialproef
of herstelactie heeft het oorspronkelijke pad geschreven. De oorzaak van
de eerdere afwijking is hiermee niet vastgesteld; root onderzoekt dit.

Oorspronkelijke bewijsbestanden zijn niet teruggeschreven, vervangen of
alsnog groen verklaard. De hoofdagent is onmiddellijk over deze blokkade
geïnformeerd. Alleen bewijs uit een geldige, volledig vastgelegde run kan de
G0-eindacceptatie sluiten.

## Wat wel onafhankelijk is gecontroleerd

Qsim: alle opgegeven bron-, build-, input-, firmware- en archivehashes komen
overeen met de echte bestanden. De gehele gebruikers-ELF komt exact één keer
voor in de werkelijk geladen kernel-ELF. De drie loadsegmenten zijn RX, RW
en R op afzonderlijke paginaranges. Ruwe serial en stderr zijn bytegelijk aan
het archief; raw-exit 33, geen timeout, één CPU en geen netwerk passen bij
het opgenomen profiel. De strikte native reeks bevat PID 1 en 2, ieder 253
checkpoints, 7.216 nuttige arenabytes en werkafstand 448; er zijn vier timer-
interrupts, 505 syscalls en 96 vrijgemaakte procesframes.

De gekoppelde broncontrole bevestigt dat de private heap per proces 65.536
bytes is, de observer de complete rapportvelden controleert, en frame-eigendom,
W^X, afzonderlijke mappingframes en cleanupbalans vóór PASS worden getoetst.
De gemeten serialcijfers blijven onderscheiden van die brongecontroleerde
voorwaarden; er is geen nieuwe fysieke-geheugentest of algemene dienst-API.

G0: alle vijf echte 800×600 PPM-bestanden, de kernel/loader, firmware,
gepinde VGA-ROM en QMP-trace hebben de verwachte hashes. De vier opnieuw
berekende pixelverschillen zijn respectievelijk `[48,192,62,212]` en driemaal
`[60,192,74,212]`; het totale verschil is `[48,192,74,212]`. Iedere verandering
blijft in de invoerrij. De opname na backspace is volledig bytegelijk aan de
opname na `a`, en wijkt af van de opname na `ab`. QMP bevat de elf correct
gecorreleerde commando's, de vijf echte afbeeldingspaden en toetsen
`a`, `b`, `backspace`, `c`, `esc` in de juiste volgorde.

De ROM-pin is rechtstreeks vergeleken met de echte bron- en runbestanden.
De opgeslagen negatieve pincontrole en de codevolgorde wijzen op weigering
vóór het maken van QEMU; die hostproef is niet opnieuw uitgevoerd en wordt
niet als extra VM geteld. Er is geen PNG gebruikt als pixelorakel.

## Reproductie en vervolg

```sh
python3 -B docs/verification/parallel-final-crosscheck-storage/verify.py
```

Deze opdracht controleert opnieuw dezelfde bewaarde artifacts en faalt bij
de hierboven vastgestelde discrepantie. De oorspronkelijke eerste uitkomst
blijft apart bewaard. Het onderzoek wijzigt geen roadmapcriteria of score.
De concrete blocker is uitsluitend het G0-rawartifact; qsim heeft binnen de
onderzochte bewijsgrens geen gevonden blocker.

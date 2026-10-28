# B06 — geslaagde controle van de vervangende G0-run

**Alle 87 gerichte controles slagen.** `verify_confirmed_g0.py` leest 34
bestanden uit de nieuwe run, het bevestigde archief en de gepinde tool-/firmware-
bronnen. Hij start geen VM, build of testsuite en schrijft geen afbeelding of
oorspronkelijk artifact. Alleen `second-check.json` wordt als nieuwe eigen
controleuitkomst geschreven. De eerste mislukte controle is niet overschreven.

## Exacte bewijsbinding

- Nieuwe echte run van root:
  `/tmp/nieuw-os-g0-confirmed-pipe/graphics-g0-6c34d3a840`.
- Bevestigd archief: `docs/verification/parallel-final-g0-confirmed/pipe`.
- Uitvoeringscommit: `3e41bbf93822f7044c6020259f4a752925e1966d`.
- Bronfingerprint v3:
  `307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60`.
- QEMU-profiel: expliciet `pc-q35-7.2`, pipe-QMP, geen netwerk.
- Raw-exit: 33; volledige native serialreeks en geen fouten.

Alle vijftien verplichte artifacts zijn rechtstreeks opnieuw gehasht en
aan het runrapport gebonden, waaronder werkelijke kernel/loader, firmware,
VGA-ROM, vijf PPM's en QMP-trace. Na de volledige controle zijn alle vijftien
nogmaals gelezen; hun hashes zijn ongewijzigd. De bronfingerprint bleef gelijk.

Het echte serialbestand is nu 425 bytes met SHA-256
`f61350dd1041ed8736a36452f1f4e56821f05f4f11008c9036e327913bfe2596`.
De byte-identieke archiefkopie en het runrapport bevatten dezelfde gegevens.
Na de vier invoerregels volgen daadwerkelijk `G0:PASS` en `M1:PASS`.
De gecontroleerde QMP-trace bevat de juiste elf ID/antwoordparen en de
toetsen `a`, `b`, `backspace`, `c`, `esc` met de juiste screenshotpaden.

## PPM-bytes en lossless PNG's

Alle vijf bewaarde PNG's zijn uitsluitend gelezen. Hun RGB-bytes zijn in het
geheugen gecombineerd met de opgeslagen originele PPM-header. Voor iedere
afbeelding is het resultaat **volledig bytegelijk** aan het werkelijk bewaarde
QEMU-PPM-bestand; zowel PNG- als PPM-hash komt overeen met het archief.

| Stap | Opnieuw berekend veranderd pixelgebied |
|---|---|
| Begin → `a` | `[48,192,62,212]` |
| `a` → `ab` | `[60,192,74,212]` |
| `ab` → backspace | `[60,192,74,212]` |
| Backspace → `ac` | `[60,192,74,212]` |

Alle afbeeldingen zijn 800×600 en iedere wijziging blijft binnen de
invoerrij. De volledige PPM na backspace is bytegelijk aan de eerdere PPM
na `a`, en verschilt van de PPM na `ab`. PNG-compressie introduceert dus
geen andere pixels of een nieuw ontworpen beeld.

## Afhandeling van de eerdere afwijking

De oude run `graphics-g0-5ad6321b1d` heeft nog steeds zijn afwijkende 409-byte
rawlog. Root heeft hem expliciet uitgesloten in
`docs/verification/parallel-final-g0/invalidated.json`. Deze tweede controle
bevestigt zowel de uitsluiting als de ongewijzigde afwijkende bytes. Het oude
archief is niet teruggekopieerd om het ontbrekende einde te herstellen.
De oorspronkelijke oorzaak blijft onbekend.

Een afzonderlijke UNIX-QMP-probe is door `Operation not permitted` geblokkeerd
en heeft `passed=false`; zij telt niet als geslaagde native run. De bevestigde
pipe-route heeft daar geen afhankelijkheid van. Er is in deze controle geen
nieuwe poging gedaan om de sandboxgrens te passeren.

Qsim had in de eerste onafhankelijke controle geen blocker. De vervangende
G0-run gebruikt dezelfde bevroren bron; de eerdere qsim-artifact-, proces-,
heap- en cleanupcontrole blijft afzonderlijk herkenbaar. Deze hercontrole
verleent geen nieuw roadmap-punt en maakt geen claim over een compositor,
native Materiaal-documentopslag of fysieke hardware.

Op de uitvoeringsbron met de bewaarde artifacts:

```sh
python3 -B docs/verification/parallel-final-crosscheck-storage/verify_confirmed_g0.py
```

Primaire bronnen zijn de bewaarde ruwe runbestanden en de Nieuw OS-bronboom
op de hierboven genoemde commit, gecontroleerd op 2026-10-28:
[bronboom](https://github.com/simonvanieperen/nieuw-os/tree/3e41bbf93822f7044c6020259f4a752925e1966d).

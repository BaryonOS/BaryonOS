# B09 — vaste outputlijst voor de G0-capture

## Concrete reviewbevinding en reparatie

G09 vond in B07-commit `764cc9f84291ed3ad7b0ea3de65debcbf1cc7da6` dat de
archiefindex uit `staged.iterdir()` werd opgebouwd. Een onverwacht regulier
bestand dat vóór die indexopbouw in de tijdelijke map terechtkwam, kreeg
daardoor een hash en kon met het archief worden gepubliceerd. De oorspronkelijke
inputallowlist verhinderde dit niet: deze bevinding gaat over staged output.

De reparatie definieert `ARCHIVE_FILES` met exact vijf tekstbestanden, vijf
PNG's en `image-roundtrip.json`. Die vaste set wordt gecontroleerd vóór de
indexopbouw. De index wordt uitsluitend uit de constante opgebouwd. Direct
vóór publicatie moet de tijdelijke map exact dezelfde namen plus
`capture.json` bevatten. De bestaande input-, bestands-, hash-, source-,
PNG- en atomic-no-replace-controles blijven behouden.

Alleen `tools/capture_graphics.py`, `tests/test_graphics_capture.py` en deze
bewijsmap zijn veranderd. De branch is
`codex/parallel-g0-capture-20261028`; de basis is bovengenoemde B07-commit.

## Uitgevoerde negatieve proef

De nieuwe test gebruikt uitsluitend een synthetische tijdelijke hostfixture.
Een `encode_png`-hook schrijft `unexpected.bin` in de tijdelijke archiefmap
en laat de echte encoder verder werken. De test eist een fout, geen
gepubliceerde bestemming en volledige verwijdering van de stagingmap.

Op de oude B07-code faalde die nieuwe test aantoonbaar met exitstatus 1:
de capture wierp geen fout. `before.json` en `before.stderr.txt` bewaren
het exacte commando en resultaat. Na de reparatie slaagden alle **34**
graphics-testmethoden, waaronder **23** capturetests en de **11** bestaande
G0-harnastests:

```sh
python3 -B -m unittest discover -s tests -p 'test_graphics*.py' -v
```

`after.json` en de bijbehorende ruwe logs bevatten exitstatus 0, source-v3,
de twee gewijzigde bestandshashes en het tijdstip. `git diff --cached --check`
is eveneens zonder fouten uitgevoerd.

## Bewijsgrens en vervolg

Dit is een herstel van een capturevalidator op de host. Er is geen VM gedraaid,
geen oud native artifact gewijzigd en geen resultaat gerepareerd. De hoofdagent
integreert deze kleine reparatie voordat B08 op de nieuwe bevroren bron een
echte G0-run en capture uitvoert. Het onafhankelijke G09-script kan de
gerepareerde bron opnieuw controleren.

Roadmapvoorstel: **0 punten wijziging**. De test voorkomt dat onbekende staged
output als geaccepteerd G0-bewijs wordt meegenomen; zij sluit geen nieuw
OS-criterium af.

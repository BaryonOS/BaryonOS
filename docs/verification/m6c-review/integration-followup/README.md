# M6c: gecombineerde capturecontrole na het M6b-vervolgherstel

Datum: 28 oktober 2026.

## Broncombinatie

De geteste hostbasis is `b3d9d682f3f043ef7926f112b659ad77a5f00e1a`.
Daarop zijn uitsluitend de volgende zes bronbestanden vervangen of toegevoegd:

| Gepubliceerde herkomst | Bestanden |
|---|---|
| PR10-commit `9819d6b71efa9f13d0d68d9b75529ef5772d973f`, tree `728147e3edd64131dcbdfafe2ee85c1e57aa0110` | `tools/test_m6c.py`, `tools/capture_m6c.py`, `tests/test_m6c_capture.py` |
| PR9-tree `97bd9e2098f869397a6a8ee2ec211e5bfecbf719` | `tools/test_m6.py`, `tools/capture_m6.py`, `tests/test_m6_capture.py` |

Alle overlaybytes zijn vóór de controle tegen die Git-blobs vergeleken.
De PR9-tree is daarna als commit
`69b3eec79234d12eef50f978e0ae02c4382a4d85` gepubliceerd. De controle claimt
geen checkout van een commit die bij het uitvoeren nog niet bestond:
zij bindt aan de vooraf beschikbare tree en de vastgelegde bronbytes.

De concrete integratiecontrole is nodig omdat zowel M6b- als M6c-capture
gedeelde functies uit `capture_m6` importeert. Het nieuwe stderr-pad en de
M6c-hostloghashbinding worden zo op dezelfde gecombineerde hostbron getoetst.

## Uitvoering en uitkomst

Vanuit de vastgelegde candidate is precies één opdracht uitgevoerd:

```sh
python3 -B -m unittest discover -s tests -p 'test_m6*_capture.py' -v
```

De commandoregistratie bevat de werkelijk gebruikte absolute Python-executable,
cwd, UTC-tijd, omgeving, exitstatus en inputhashes.

**29 van 29 testmethoden slagen, exit 0:** vier M6a-, tien M6b- en vijftien
M6c-capturetests. De 44 tools-/testbestanden bleven tussen de snapshots
vóór en na de uitvoering bytegelijk. De daadwerkelijke testuitvoer staat
in `records/combined-capture.stderr.txt`; stdout is afzonderlijk bewaard.
`provenance.json` en `manifest.json` verbinden de broncombinatie en logs.

Deze 29 zijn de testselectie op deze gecombineerde bron. Zij worden niet
opgeteld bij eerdere aantallen van 123, 21 of 40, omdat die eerdere runs
andere bronmomenten en overlappende testgroepen hebben. Er zijn geen
verdere suites uitgevoerd voor deze integratiecontrole.

## Grenzen en herhalen

Gastrecords en hostcommando's in deze tests zijn synthetische fixtures of
testdoubles. Dit is geen nieuwe native build, QEMU-uitvoering, devicefoutinjectie
of native acceptatie. De oorspronkelijke native runtimebron blijft behouden;
de bekende observerpanic blijft de afzonderlijk gedocumenteerde beperking.

De bovenstaande opdracht kan vanuit de repositoryroot van de geïntegreerde
bron worden herhaald. Herhalingen horen in een eigen werkmap en logset.
De absolute paden in de ruwe records beschrijven de werkelijke uitvoering.
`run_capture_check.py` gebruikt een aparte candidate-/recordsindeling en is geen
opdracht om dit bewijs te overschrijven. De historische en huidige versie worden
hieronder afzonderlijk gebonden.

## Reviewherstel van de recorderexitstatus — 28 oktober 2026

De historische run van 29 tests gebruikte recorder-SHA-256
`68c969dac3f4a3927701da01fc23605a4df503d4152779c4ecc0a30b93380483`.
Die [oorspronkelijke recorderbytes](https://github.com/simonvanieperen/nieuw-os/blob/4c8ae3cfee8a48e508cd3cd069ea4f832b0d9e0f/docs/verification/m6c-review/integration-followup/run_capture_check.py)
zijn terug te lezen in archiefcommit `4c8ae3c`; dit is geen claim dat de eerdere
testuitvoering op die latere commit plaatsvond. De oorspronkelijke
`provenance.json`, commandoreceipt en stdout/stderr blijven bytegelijk.

De huidige recorder schrijft eerst zijn receipt en output en eindigt daarna met
status 1 wanneer het testsubproces faalt of de broninputs tijdens de controle
veranderen. Een geslaagde controle met ongewijzigde inputs houdt status 0.
De huidige recorder-SHA-256 is
`ac885c4380f927811fa3358da6c4a882ae9f8a2fbe0184bb8463e87dbb491c91`.

Vier gerichte hostregressies voeren de echte recorder en een klein
Python-testsubproces uit in eigen tijdelijke mappen: succes, falend subproces,
gewijzigde input en beide fouten samen. Op de oude recorder falen de drie
negatieve scenario's op de onterechte exitstatus 0; na de twee-regelcorrectie
slagen alle vier. De tests controleren ook dat receipts en ruwe output bij
afwijzing bewaard blijven en aan hun werkelijke bytes zijn gebonden.

Exacte opdrachten, bronhashes, uitvoer en exitstatussen staan in
[capture-check-regressions.json](capture-check-regressions.json). Dit is een
afzonderlijke hostcontrole van de recorder, geen heruitvoering van de historische
29-testsuite en geen native build, QEMU-run of nieuwe native acceptatie.
De actuele bestandsindex en historische recorderherkomst staan afzonderlijk in
`manifest.json`.

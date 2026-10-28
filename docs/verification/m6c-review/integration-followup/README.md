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
`run_capture_check.py` bewaart de destijds gebruikte recorder met zijn aparte
candidate-/recordsindeling en is geen opdracht om dit bewijs te overschrijven.

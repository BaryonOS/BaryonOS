# Bestaande M6b opnieuw getest bij de M6c-oplevering

Broncommit `bc4cd104e86e2c047b727ae96e07a03fbc810c3c`, fingerprint v3
`4468b3c18bd7fa57bf3631d634d52ee4c5fe9a76da9f125714d6db95e4407345`.
`python3 -B -u tools/test_m6b.py` eindigde met exit 0: alle 18 groepen slagen,
met 14 werkelijke VM-invocaties (13 M6b en één gewone M1). Dit omvat zes koude
recoveries en twee echte externe QEMU-kills. Ook 22 Rust-objecttests en 26
M6b-harnastests slagen. Deze scope is niet de volledige M1–M6a-regressiematrix.

[suite](m6b/suite.json), [gastrecords](m6b/guest-runs.json),
[capture](m6b/capture.json), [volledige console](console/gate.stdout.txt).
Het oorspronkelijke archief `../m6b/` is behouden en niet vervangen.

De bestaande `capture_m6b.capture()` heeft de gegevens met ongewijzigde validators
opnieuw gecontroleerd. Alleen de uiteindelijke archiefbestemming is omgeleid:

```python
import sys
sys.path.insert(0, 'tools')
import capture_m6b as c
original = c._replace_archive
destination = c.ROOT / 'docs/verification/regression-m6c/m6b'
destination.parent.mkdir(parents=True, exist_ok=True)
c._replace_archive = lambda staged, unused: original(staged, destination)
c.capture()
```

Exacte argv, tijd en exitstatus: [archive-command.json](m6b/archive-command.json).
Een eerdere poging miste de bestemmingsoudermap en stopte na validatie bij het
plaatsen van de nieuwe map. De oorspronkelijke evidence bleef daarbij behouden;
de fout staat onder `archive-diagnostic/`. Na het aanmaken van de oudermap slaagt
de capture opnieuw met exit 0. Dit heeft geen nieuwe VM-runs veroorzaakt.

De M6c-build is daarna hersteld en vergeleken met de normale M6c-eindruns;
zie [m6c-ready/build-check.json](../m6c-ready/build-check.json).
Gecombineerde acceptatie en grenzen: [M6C-ACCEPTANCE.md](../M6C-ACCEPTANCE.md).

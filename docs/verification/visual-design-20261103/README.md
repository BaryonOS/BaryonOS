# Controle van het visuele ontwerpplan

**Datum:** 3 november 2026, Europe/Amsterdam.
**Bronbasis:** `380111b68688c981eec6abb8fab29174a29f5601`.
**Branch:** `simonvanieperen-fictional-disco`.
**Type:** document-/rekencontrole, geen product-, render- of gebruikerstest.

## Uitgevoerd

| Exact commando / controle | Resultaat |
|---|---|
| `python3 tools/progress.py --write` | exit 0, 41/100; README, SVG en criteriatabel gegenereerd |
| `python3 tools/progress.py --check` | exit 0, rubric v1 consistent |
| `git diff --check` | exit 0, geen whitespacefouten |
| `python3 -` met onderstaande kleurpaarcontrole | exit 0; 33 paren, nul fouten |
| `python3 -` met onderstaande documentcontrole | exit 0; lokale links, vijf richtingen, 20 x 5 cellen, ongewijzigde criteria |
| `python3 -` met afzonderlijke HC-light-berekening | exit 0; `#002D72` tegen wit circa 12,9763:1 |

De Pythoncontroles zijn inline uitgevoerd, geen nieuwe testtool of
productmodule toegevoegd. Reproduceerbare payloads staan hieronder.
De minimale high-contrastlichtberekening gebruikt dezelfde luminantieformule.

## Kleurpaarcontrole

Dit is de uitgevoerde Python-payload van `python3 -`:

```python
import re
from pathlib import Path
p = Path('docs/design/VISUAL-ARCHITECTURE-PLAN.md')
text = p.read_text()
rows = {}
for line in text.splitlines():
    if line.startswith('|') and '`#' in line:
        parts = [v.strip() for v in line.split('|')[1:-1]]
        colors = re.findall(r'#[0-9A-Fa-f]{6}', line)
        if len(colors) == 3:
            rows[parts[0]] = colors

def luminance(color):
    channels = [int(color[i:i+2], 16) / 255 for i in (1, 3, 5)]
    linear = [v/12.92 if v <= .04045 else ((v+.055)/1.055)**2.4 for v in channels]
    return sum(v*w for v,w in zip(linear, (.2126, .7152, .0722)))

def ratio(a,b):
    x,y = sorted((luminance(a),luminance(b)))
    return (y+.05)/(x+.05)
checks=[]
for i,mode in enumerate(('light','dark','high-contrast-dark')):
    for role in ('text.primary','text.secondary','danger.text','warning.text','success.text'):
        checks.append((mode,role,'content/surface',ratio(rows[role][i],rows['content/surface'][i]),4.5))
    for surface in ('content/surface','environment'):
        for role in ('boundary.interactive','accent/focus'):
            checks.append((mode,role,surface,ratio(rows[role][i],rows[surface][i]),3.0))
    checks.append((mode,'accent.on-fill','accent/focus',ratio(rows['accent.on-fill'][i],rows['accent/focus'][i]),4.5))
    checks.append((mode,'selection.text','selection.fill',ratio(rows['selection.text'][i],rows['selection.fill'][i]),4.5))
for mode,role,surface,r,minimum in checks:
    print(f'{mode}: {role} on {surface} = {r:.3f}:1, minimum {minimum:.1f}:1, {"PASS" if r >= minimum else "FAIL"}')
failed=[c for c in checks if c[3] < c[4]]
print(f'{len(checks)} opaque sRGB color-pair checks; {len(failed)} failures. Not a rendered/alpha/native accessibility test.')
raise SystemExit(bool(failed))
```

Uitvoerlog:
`/home/simon/.copilot/session-state/7f2fb1d137004b68b9bef4b8ca72f938/files/visual-contrast-check.txt`.
SHA-256:
`79f1d9030399f87b1455869a3b918dcfeef4ba8b86e6d826ff359254e1c8ac0f`.
Minimale gewone tekstcontrastwaarde in de 33 paren: circa 6,68:1;
minimale interactieve grens: circa 3,69:1.
Geen meting van uiteindelijke composited/antialiased pixels.

## Document-/criteriacontrole

Uitgevoerd op de werkboom tegen de toenmalige `HEAD` (bovenstaande basis):

```python
import json,re,subprocess
from pathlib import Path
files=[Path('docs/design/VISUAL-ARCHITECTURE-PLAN.md'),Path('docs/research/VISUAL-RENDERING-RESEARCH.md'),Path('docs/research/README.md')]
for p in files:
    text=p.read_text()
    for link in re.findall(r'\]\(([^)]+)\)',text):
        if '://' not in link and not link.startswith('#'):
            target=(p.parent/link.split('#')[0]).resolve()
            assert target.is_relative_to(Path.cwd()),(p,link)
            assert target.is_file(),(p,link)
    print(f'{p}: relative links PASS')
plan=files[0].read_text()
state_rows=[line for line in plan.splitlines() if re.match(r'^\| \d{2} ',line)]
assert len(state_rows)==20,len(state_rows)
assert all(len(line.split('|'))==8 for line in state_rows)
assert len(re.findall(r'^### R[1-5] ',plan,re.M))==5
print('Five architecture descriptions and 20 x 5 textual state cells: PASS (no visual validation claim)')
base=json.loads(subprocess.check_output(['git','show','HEAD:docs/progress.json']))
new=json.loads(Path('docs/progress.json').read_text())
assert base['tracks']==new['tracks']
assert base['rubric_version']==new['rubric_version']
print('All criterion statuses, evidence and weights unchanged: PASS')
```

Bij herhaling na commit voor de originele vergelijking `HEAD:docs/progress.json`
vervangen door `380111b68688c981eec6abb8fab29174a29f5601:docs/progress.json`.

Uitvoerlog:
`/home/simon/.copilot/session-state/7f2fb1d137004b68b9bef4b8ca72f938/files/visual-document-check.txt`.
SHA-256:
`439b48f0dd8893877c4791690c9b63a5e330f7ccde880595f3607d6a7143a64e`.

### Uitgebreide eindcontrole

Een eerste uitbreiding naar alle README-links gebruikte ten onrechte
`Path.is_file()` voor ook geldige maplinks. De Pythoncontrole stopte met
`AssertionError` op de bestaande maplink `docs/verification/m6b-storage-errors/`
(Python exit 1; de omvattende shell eindigde door een daaropvolgend
statuscommando toch met 0). Dit is **geen** geslaagde linkcontrole.
Geen bestaande documentlink aangepast om de controle groen te maken.

De gecorrigeerde inlinecontrole gebruikte `Path.exists()` voor lokale
file-/maplinks, bekeek ook README/PROGRESS, herhaalde de 5/20x5-controle en
vergeleek criteriastaat met de expliciete broncommit. Verder controleerde
zij dat de volledige gewijzigde/ongetrackte bestandsset precies de negen
documentatie-/meterbestanden hieronder was. Uitgevoerd met
`set -euo pipefail`, `python3 -`, `python3 tools/progress.py --check` en
`git diff --check`: exit 0.

Eindlog:
`/home/simon/.copilot/session-state/7f2fb1d137004b68b9bef4b8ca72f938/files/visual-final-scope-check.txt`.
SHA-256:
`3afdcb64ecca846387aedead60f7ee79c2140bf29d9a6fc74b60ec3cf7afb734`.

Gewijzigde bestanden: `README.md`, `docs/PROJECT-STATE.md`,
`docs/PROGRESS.md`, `docs/progress.json`, `docs/assets/progress.svg`,
`docs/research/README.md`, `docs/research/VISUAL-RENDERING-RESEARCH.md`,
`docs/design/VISUAL-ARCHITECTURE-PLAN.md` en dit controleverslag.

## Grenzen en persistentie

Geen Rust-/apphosttests, nieuwe VM-, fysieke, menselijke, schermlezer- of
renderperformanceproeven uitgevoerd: productcode bleef ongewijzigd.
Geen dependencies geïnstalleerd, oude tests naar een nieuwe bron omgelabeld,
images gegenereerd of hardware aangesloten.

Onderzochte externe code is geen Nieuw-OS-implementatie; ontwerpinschattingen
zijn geen voorkeurmeting. Veld/Werkstuk zijn niet definitief gekozen.
De oorspronkelijke A/B/C-beelden ontbraken en de uitgebreide geschiedenis van
ieder OS is niet afzonderlijk volledig geverifieerd.
Nieuwe aannames: compositie- en effectwaarden zijn testbare studiewaarden,
niet gevalideerde productdefaults.

**Voortgang:** 41% -> 41%, nul nieuwe criteria. De onderzoeksdocumenten,
index, checkpoint en gekoppelde meter worden samen gecommit en naar de
werkbranch gepusht; geen PR/merge/release in deze opdracht.

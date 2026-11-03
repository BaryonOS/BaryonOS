# PR150: alleen-lezen bewijscontract

3 november 2026; basis `fc23841faf8ba4e0918245aef5c413745a790c67`
met main `95d7945110ce853a1b5827ec59f6115475691805`.

## Voorstel

Geen tooluitbreiding of nieuwe uitvoerdersrol.

## Implementatie

OS Verifier beoordeelt aangeleverd bewijs en voert geen proef uit.
OS Maturity Auditor gebruikt de bestaande repositorybewijssoorten in plaats
van een ongedefinieerde E0-E8-schaal. Geen toolrecht uitgebreid.

Primaire bronnen: Nieuw OS-project, 3 november 2026, commit
`cccb29e8daf311a9db47e3747a6fb9d05329bda3`:
[OS Verifier](https://github.com/simonvanieperen/nieuw-os/blob/cccb29e8daf311a9db47e3747a6fb9d05329bda3/.github/agents/os-verifier.agent.md)
en [OS Maturity Auditor](https://github.com/simonvanieperen/nieuw-os/blob/cccb29e8daf311a9db47e3747a6fb9d05329bda3/.github/agents/os-maturity-auditor.agent.md).

## Testbewijs

De oorspronkelijke worktreecontrole gaf exit 0. De hieronder aangescherpte
hercontrole controleert alleen dat de huidige zestien toolsets gelijk zijn aan
de hieronder hardcoded verwachte lijsten, en vergelijkt de twee historische
profielhashes met de bytes op contractfix `cccb29e8`, niet met latere
profielwijzigingen. Zij leest geen `tools`-velden uit een baselineversie en
bewijst dus niet zelfstandig dat geen toolrecht ten opzichte van een eerdere
versie is uitgebreid. Hercontrole op basis `3ee3e73a` met main `ab85d633`
([Nieuw OS-project, 3 november 2026, commit `ab85d633`: zestien
AI-deelanalyses en gezamenlijke audit (#151)](https://github.com/simonvanieperen/nieuw-os/commit/ab85d6339ce5bdddb10a2de171d0c1b04e4b62ff)):
exit 0 op 3 november 2026; herhaald op de PR-head na mainintegratie `681f1137`.

```sh
python3 -B - <<'PY'
import json,hashlib,subprocess
from pathlib import Path
profiles=sorted(Path('.github/agents').glob('os-*.agent.md'))
assert len(profiles)==16
for p in profiles:
 front=p.read_text().split('---\n',2)[1]
 fields=dict(line.split(': ',1) for line in front.splitlines() if ': ' in line)
 assert fields['target']=='github-copilot'
 assert fields['disable-model-invocation']=='true'
 tools=json.loads(fields['tools'])
 assert tools==(['read','search','edit','execute'] if p.name=='os-builder.agent.md' else ['read','search']),p
print('PASS:16 profiles: target, disable-model-invocation, current tools match expected lists; no baseline diff, not full schema or runtime confinement')
source='cccb29e8daf311a9db47e3747a6fb9d05329bda3'
expected={
 'os-verifier.agent.md':'6380991093181852e2044e0b8b0804587d62d8e0c268ac7bcc7152826286ab78',
 'os-maturity-auditor.agent.md':'755945403323331f1ff8b2634b6e2bc69d11379a2447a544ddb3eabcd8852e94',
}
for name,wanted in expected.items():
 p=Path('.github/agents')/name
 data=subprocess.check_output(['git','show',f'{source}:{p.as_posix()}'],timeout=30)
 actual=hashlib.sha256(data).hexdigest()
 assert actual==wanted,(p,source,actual,wanted)
 print(p,source,actual)
PY
```

| Gewijzigd profiel onder `.github/agents/` | SHA-256 |
|---|---|
| `os-verifier.agent.md` | `6380991093181852e2044e0b8b0804587d62d8e0c268ac7bcc7152826286ab78` |
| `os-maturity-auditor.agent.md` | `755945403323331f1ff8b2634b6e2bc69d11379a2447a544ddb3eabcd8852e94` |

## Onbekend

De tabelhashes hierboven horen bij contractfix `cccb29e8`, niet bij de latere
reviewcorrectie voor referentiebaselines en verplichte oordeellabels.
Die correctie wijzigt alleen profielinstructies, geen toolsets of runtime.

De eerdere profielhashes in `pr150-profile-review.json` blijven historische
bronbindingen, niet hashes van deze twee gewijzigde bestanden. Dit is geen
volledige YAML/cloudschemavalidatie, werkelijke agentselectie, sandbox,
native uitvoering of menselijke audit. Geen productcriterium of scorewijziging.

De latere main78ff7c03-integratie neemt de ingehaalde centrale metadata
niet over; die historie blijft volledig behouden op
`archive/pr150-pre-main78-20261103` (`4ef30bf7`).
Alleen profielen, direct gerelateerde profielinstructies en eigen receipts
blijven als inhoudsdelta. De huidige mainassessment is geen profieltest.

Primaire bronnen voor deze historische status: Nieuw OS-project,
3 november 2026, [contractfix: beperk verifier tot bewijsbeoordeling en
definieer bewijssoorten](https://github.com/simonvanieperen/nieuw-os/commit/cccb29e8daf311a9db47e3747a6fb9d05329bda3),
[reviewcorrectie: referentiebaselines en bewijsankers](https://github.com/simonvanieperen/nieuw-os/commit/3ee3e73a7e8a9422d027fe5d0268923689888472),
[main: expliciete TEST-ondertekenpolicy](https://github.com/simonvanieperen/nieuw-os/commit/78ff7c03b015ad16fe665ae7df9fe239d078e08d)
en [historische mainintegratie op de bewaarde archiefcommit](https://github.com/simonvanieperen/nieuw-os/commit/4ef30bf7d0e2d3056783a949a970e06fcbbe7a89).

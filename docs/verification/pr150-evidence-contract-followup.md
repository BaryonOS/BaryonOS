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

Uitgevoerd vanuit de worktree, exit 0:

```sh
python3 -B - <<'PY'
import json,hashlib
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
print('PASS:16 frontmatter/toolsets; not full cloud schema or runtime confinement')
for n in ('os-verifier','os-maturity-auditor'):
 p=Path('.github/agents')/(n+'.agent.md');print(p,hashlib.sha256(p.read_bytes()).hexdigest())
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

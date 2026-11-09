# Selectieve Dependabot-herintegratie — issue #283

## Bron en afbakening

- Repository: `simonvanieperen/BaryonOS`, main `f704f078213170e72292a5a491f0ee83bd588955`.
- Werkbranch: `coderabbit/issue-283-dependabot-coverage`; bewijs betreft de gewijzigde bestanden op deze basis.
- Oorspronkelijk issue: [#283](https://github.com/simonvanieperen/BaryonOS/issues/283).
- Selectief overgenomen voorstel: [PR #207](https://github.com/simonvanieperen/BaryonOS/pull/207),
  head `7da75251f20d46a9a9277931117480efba1d0183`, live geraadpleegd op 2026-11-09:
  `CLOSED`, `mergedAt: null`. De diff is gelezen met `gh pr diff 207`.

De oude configuratie dekt alleen `/` en `/experiments/quantum`. Van de
61 project-eigen zelfstandige Cargo-workspaces ontbreken er 59. Die zijn
expliciet toegevoegd zonder de bestaande twee updateblokken te wijzigen.
De 62e tracked lockfile behoort tot de vendored `zune-jpeg`-bron en blijft
uitgesloten. Workspaceleden worden niet dubbel als zelfstandige root opgenomen.
De planning-/limietwijzigingen voor bestaande blokken en de gestapelde
workflowwijzigingen uit #207 zijn niet overgenomen.

De nieuwe entry behoudt de maandelijkse cadans, het label `dependencies`
en de voorgestelde limiet 2. `group-by: dependency-name` bundelt versie-updates
voor dezelfde dependency over directories. Onverenigbare versievoorwaarden
kunnen aparte PR's opleveren; security-updates hebben een eigen limiet en vallen
buiten deze groepering. Er wordt geen repositorybrede PR-limiet geclaimd.

Primaire bron: GitHub, *Dependabot options reference*, secties `directories`,
`groups` en `open-pull-requests-limit`, geraadpleegd op 9 november 2026:
https://docs.github.com/en/code-security/dependabot/working-with-dependabot/dependabot-options-reference

## Uitgevoerde controles

Omgeving: Python 3.14.4, PyYAML 6.0.3. De bestaande `host-quality`-stap ontdekt
`tests/test_ci_workflows.py` en neemt de twee nieuwe contracttests daarmee mee.

| Commando | Resultaat | Exitstatus |
|---|---|---:|
| `python3 -B -m unittest discover -s tests -p test_ci_workflows.py` vóór configuratiefix, met nieuwe tests | 11 tests; één verwachte fout: 59 ontbrekende roots | 1 |
| `python3 -B -m unittest discover -s tests -p test_ci_workflows.py` na configuratiefix | 11 geslaagd | 0 |
| `python3 -B -m unittest discover -s tests -p 'test_ci_evidence*.py'` | 23 geslaagd | 0 |
| `python3 tools/progress.py --write` | weergaven bijgewerkt, 41/100 | 0 |
| `python3 tools/progress.py --check` | consistent, 41/100 | 0 |
| `git diff --check` | geen whitespacefouten | 0 |

De dekkingstest ontdekt expliciete `[workspace]`-grenzen uit tracked manifests
en weigert ontbrekende, dubbele of extra directories, inclusief vendorroots.
De tweede test bewaakt maandelijkse groepering en de limiet voor de extra entry.
Deze controles voeren geen Dependabot-updates uit.

SHA-256 van de geteste configuratie en tests:

- `.github/dependabot.yml`: `446f5ba50bc02fb6f3850a07aaee810d56a7b8ba54edc375383bf560f17b1944`
- `tests/test_ci_workflows.py`: `16d113f2b069cd6b0c4598fddbc8abe60373c2ecde08771b8f0c41b46885247e`

## Grenzen en vervolg

De volledige `host-quality` en `qemu-smoke` zijn lokaal **niet uitgevoerd**:
OS-code, builds en workflowgates zijn niet gewijzigd. De verplichte hosted
checks op de PR-head blijven afzonderlijk te beoordelen. QEMU-, hardware-,
Dependabot-service-uitvoering en nieuwe OS-acceptatie worden niet geclaimd.
Graphify-MCP is niet beschikbaar; de repository is rechtstreeks gelezen.
Een tweede AI heeft read-only de scope en workspacegrenzen beoordeeld;
dit is geen onafhankelijke menselijke audit.

Alle overige werkstromen uit #283 blijven onbehandeld. Er zijn geen oude PR's,
issues, branches of branchregels gewijzigd. De voortgang blijft **41 → 41/100**.
Terugdraaien kan met een gewone revert van deze wijziging.

# CI voor de publieke repository

**Configuratie:** alle 23 workflowbestanden kiezen standaard
`ubuntu-24.04`: 26 jobdefinities en maximaal 42 jobs na matrixuitbreiding,
inclusief de afzonderlijke Copilot-setup en de historisch private audit.
Die audit behoudt zijn private-only voorwaarde en draait niet op de publieke
repository. Er zijn geen self-hosted runnerselectors meer.

**Integratie:** de bestaande PR #249 combineert de runner-/toolpins met de
bewijsoplossing uit PR #251 en de stroomlijning hieronder. De repository is
op 8 november 2026 via GitHub als `public` gecontroleerd. Configuratie op een
werkbranch is pas de standaard voor nieuwe main-runs na samenvoeging.

**Bewijs:** de lokale host- en configuratiecontroles staan in
[het integratierapport](verification/ci-public-runners-20261108.json).
Op de gepubliceerde bron `338bec8c` zijn vervolgens 39 van de 40 gestarte
Actions-jobs geslaagd, waaronder beide verplichte checks, de volledige
SMP-suite en de Copilot-setup. De venstersessietest liep bij die waarneming
nog; er was geen mislukte job. De echte runnerlabels, uitslagen, tijden en
geslaagde compacte QEMU-upload staan in het
[GitHub-checkpoint](verification/ci-public-runners-20261108/hosted-checkpoint.json).
Dit checkpoint is geen einduitslag van de nog lopende test of van latere commits.
De eerder gemeten budgetblokkade op de toen nog private repository is
historisch bewijs; zij wordt niet als actuele uitvoeringsuitslag gebruikt.

## Wanneer wat draait

| Aanleiding | Uitvoering |
|---|---|
| PR geopend, gewijzigd of heropend, ook als concept | Verplichte `host-quality` en `qemu-smoke`; daarnaast toepasselijke specialistische workflows |
| Concept-PR gereedmaken voor review | Geen herhaling van dezelfde bron |
| Push naar main | Verplichte checks op de geïntegreerde bron; specialistische workflows volgen dezelfde documentfilters als PRs |
| Alleen `docs/**` of root-Markdown veranderd | Verplichte CI blijft beschikbaar; brede specialistische workflows starten niet opnieuw |
| Handmatig via Actions | Iedere reguliere workflow kan afzonderlijk worden gestart; bewijsworkflows bieden `full_evidence` |
| Copilot-setup aangepast | Gerichte setupcontrole; de configuratie is na merge de ontwikkelomgeving voor Copilot |

`ci.yml` heeft bewust geen padfilters. De bestaande main- en release-regels
vereisen `host-quality` en `qemu-smoke`; een geheel overgeslagen workflow
kan anders een verplichte check op Pending laten staan. Beide jobs hebben
hun eigen checkout en build en draaien nu parallel. Beide moeten slagen.

Achterhaalde PR-runs worden via de bestaande concurrencygroepen geannuleerd.
De SMP-workflow beschermt daarnaast de lopende main-run; dit garandeert niet
dat iedere wachtende tussencommit afzonderlijk wordt uitgevoerd.

De componentmatrix is teruggebracht van 16 naar **14 jobs**, met maximaal
vier tegelijk zodat de matrix de andere gates niet volledig bezet houdt.
`ui/native` wordt al volledig gecontroleerd in `native-display-input.yml`.
`artifact-store-core` heeft zijn eigen workflow, nu ook met de scalar
freestanding-buildvlaggen uit de voormalige matrixentry. Geen componenttest
of native testcase is hiervoor verwijderd.

## Vaste gereedschappen

| Onderdeel | Versie/selectie | Beheerplek |
|---|---|---|
| Runner | Standaard `ubuntu-24.04`, Linux x64 | `.github/workflows/` |
| Python | 3.12.12 | `.github/actions/setup-python/action.yml` |
| PyYAML voor hosttests | 6.0.3, gehasht CPython 3.12/Linux x64-wheel | `setup-python/requirements.txt` |
| Rust | 1.90.0; bestaande 1.91.1-controles behouden | Toolchainbestanden en workflowstappen |
| Node | 24.22.1 voor prototype-/NAS-tests en Copilot | CI- en Copilot-setup |
| QEMU x86/common/data | `1:8.2.2+ds-0ubuntu1.18` | `.github/actions/setup-qemu/action.yml` |
| OVMF | `2024.02-2ubuntu0.10` | Dezelfde QEMU-setup |
| SeaBIOS | `1.16.3-2` | Dezelfde QEMU-setup |
| iPXE-QEMU | `1.21.1+git-20220113.fbbdc3926-0ubuntu2` | Dezelfde QEMU-setup |
| swtpm / libtpms0 | `0.7.3-0ubuntu5.24.04.1` / `0.9.3-0ubuntu4.24.04.1` | Native-system en Copilot |

De 17 reguliere QEMU-jobdefinities plus Copilot hergebruiken één installatie
uit het ondertekende Ubuntu-snapshot **20261108T000000Z**. De setup controleert
exacte pakketversies, `pc-q35-8.2`, TCG en de SHA256 van OVMF CODE/VARS en het
VGA-ROM. Er is geen stille fallback naar andere toolversies. Firmwarepaden:

```text
OS_OVMF_CODE=/usr/share/OVMF/OVMF_CODE_4M.fd
OS_OVMF_VARS=/usr/share/OVMF/OVMF_VARS_4M.fd
```

TCG en wegwerpimages blijven de reguliere testbasis; KVM of een eigen server
is geen voorwaarde. De SMP-profielen behouden hun eigen CPU-aantallen en
inhoudelijke controles. De bestaande gast- en jobtijdslimieten blijven
behouden; eventuele capaciteitsproblemen moeten eerst op de actuele run
worden gemeten.

`ubuntu-24.04` fixeert de distributie, niet iedere byte van GitHubs VM-image.
De setup registreert daarom `ImageOS`, `ImageVersion`, pakketten en
firmwarehashes. Bestaande hostbibliotheken kunnen met het runnerimage wijzigen.

## Cache zonder overgeslagen controles

De bestaande downloadcaches blijven behouden. De host-Rust-buildcache heeft
nu een sleutel met compatibiliteitshash én bron-SHA; `restore-keys` hergebruikt
de laatste passende cache. Daardoor kan een latere bronversie zijn bijgewerkte
intermediates bewaren. GitHub-caches kunnen niet onder dezelfde sleutel
worden overschreven.

Alle test- en buildstappen blijven na herstel draaien. Native schijfimages,
acceptatieverslagen en secrets vallen buiten deze cache. De wijziging is
een uitvoeringsoptimalisatie, geen gemeten snelheidsclaim.

## Host- en QEMU-controles

`host-quality` behoudt Rust-tests, Clippy/fmt, SDK- en targetbuilds,
Python-/JavaScript-tests, quantumproeven en de voortgangs-/schone-boomcontrole.
De extra bestaande runner-, preflight-, harnas-, artifactbindings- en
M6c-hosttests uit PR #249 blijven opgenomen. Nieuwe configuratieregressietests
bewaken runnerkeuze, beperkte rechten, vaste Action-SHA's, vereiste gates en
behoud van de samengevoegde componentcontroles.

De hoofd-QEMU-gate behoudt zes starts:

| Controle | Starts bij volledig succes |
|---|---:|
| M6c normale opslag | 1 |
| DocumentSession schrijven en koude recovery | 2 |
| Native quantum-jobinterface | 1 |
| SDK-app met bevoegdheid | 1 |
| SDK-app met geweigerde bevoegdheid | 1 |

Hosttests of een QEMU-toolprobe zijn geen vervanging voor deze native gates.
Een rode test blijft rood; alleen best-effort artifactuploads tolereren
uploadfouten.

## Bewijs en testpakketten

Alle zestien bewijsworkflows, inclusief `ci.yml`, gebruiken
`preserve-evidence`: eerst brongebonden rapporten in runlog/summary, daarna
een begrensd compact artifact met drie dagen retentie. Volledige selecties
zijn handmatig op te vragen met één dag retentie. De Windows-test-ZIPs
blijven apart beschikbaar wanneer de pakketcontrole én upload slagen.

De aanvullende `qemu-smoke`-selectie bevat uitsluitend de bekende rapporten
en tekstlogs; ook de handmatige volledige selectie bevat daar geen ruwe
diskimages of binaries. Preflight schrijft naar de joblog, niet naar een
verondersteld `build/preflight.json`. Zie [CI-EVIDENCE.md](CI-EVIDENCE.md)
voor grenzen, echte uploadstatus en onafhankelijke hercontrole.

## Publieke bijdragen en onderhoud

PR-tests gebruiken `pull_request`, `contents: read` en checkout zonder
bewaarde credentials. Er is geen `pull_request_target` met uitvoerbare
PR-code. Externe Actions staan op volledige commit-SHA's. De regressietests
voorkomen dat een oude branch ongemerkt een self-hosted selector terugbrengt.

Copilot gebruikt dezelfde Python-/QEMU-pins, beide Rust-toolchains en Node.
De setup installeert geen WSL-runner en stelt geen KVM- of firewallwijziging
verplicht. De verplichte job-ID is `copilot-setup-steps`. Deze setup is geen
nieuwe reguliere native acceptatiegate.

Werk pins bewust bij met bron-/hashcontrole en echte host-/native gates.
De historische pakketprovenance staat in
[ci-packages-20261108.json](verification/ci-packages-20261108.json).
Terugdraaien: een gerichte revert via PR, met behoud van andere wijzigingen.
De productvoortgang blijft **41/100**; CI-inrichting sluit geen OS-criterium.

## CodeRabbit en mergeblokkades

De controle van 8 november 2026 bevestigt dat de actieve main-regel
`Bescherm main branch` alleen `host-quality` en `qemu-smoke` als verplichte
statuschecks noemt. Daarnaast moeten reviewgesprekken opgelost zijn en moet
de branch actueel zijn. Het vereiste aantal goedkeuringen is nul. CodeRabbit
kan toch blokkeren met een ingediende `CHANGES_REQUESTED`-review of een open
reviewgesprek; een groene `CodeRabbit`-check betekent alleen dat de reviewtaak
is afgerond.

`.coderabbit.yaml` houdt automatische incrementele reviews actief en kiest
`quiet`: belangrijke opmerkingen blijven inline, overige suggesties komen in
de samenvatting. `auto_approvals.enabled: false` voorkomt nieuwe automatische
afkeuringen en goedkeuringen van de bot. Docstringdekking blijft een
waarschuwing. De configuratie erft overige ingestelde reviewtools en
projectinstructies. Geen bronpad wordt uitgesloten en de GitHub-branchregels
worden niet gewijzigd.

Bestaande afkeuringen verdwijnen niet door deze YAML. Controleer elke
opmerking tegen de actuele bron, herstel geldige fouten en los pas daarna
het betreffende gesprek op. Trek een oude botafkeuring alleen gemotiveerd in
als de bijbehorende bevindingen zijn afgehandeld. Een bestaande open
inhoudelijke bevinding blijft open; een instellingenwijziging is geen bewijs
dat de code klopt.

CodeRabbit leest de configuratie uit de PR-branch. Werk oudere branches na
integratie bij met de actuele main-inhoud en behoud hun eigen wijzigingen.
Dat is ook nodig voor oude workflows die nog op verwijderde self-hosted
runners wachten: opnieuw starten van dezelfde oude workflow wijzigt de
runnerkeuze niet. Mergeconflicten vragen afzonderlijke inhoudsbehoudende
integratie. De tellingen en bronversies van deze controle staan in
[het CodeRabbit-verslag](verification/coderabbit-unblock-20261108.json).

## Primaire bronnen

- GitHub. (z.d.). *GitHub-hosted runners reference*. Geraadpleegd op 8 november
  2026, van https://docs.github.com/en/actions/reference/runners/github-hosted-runners
- GitHub. (z.d.). *Workflow syntax for GitHub Actions*. Geraadpleegd op
  8 november 2026, van https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax
- GitHub. (z.d.). *Dependency caching reference*. Geraadpleegd op 8 november
  2026, van https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching
- GitHub. (z.d.). *Configure the development environment for agents on GitHub
  Actions*. Geraadpleegd op 8 november 2026, van
  https://docs.github.com/en/copilot/how-tos/copilot-on-github/customize-copilot/customize-cloud-agent/customize-the-agent-environment
- Canonical. (z.d.). *Ubuntu Snapshot Service*. https://snapshot.ubuntu.com/
- CodeRabbit. (2026, 1 november). *Configuration reference*.
  https://docs.coderabbit.ai/reference/configuration
- CodeRabbit. (z.d.). *Automatic Approvals*. Geraadpleegd op 8 november 2026,
  van https://docs.coderabbit.ai/pr-reviews/automatic-approvals
- CodeRabbit. (z.d.). *Configuration inheritance*. Geraadpleegd op
  8 november 2026, van https://docs.coderabbit.ai/configuration/configuration-inheritance
- CodeRabbit. (z.d.). *Configuration via YAML file*. Geraadpleegd op
  8 november 2026, van https://docs.coderabbit.ai/getting-started/yaml-configuration

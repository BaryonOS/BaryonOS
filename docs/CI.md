# Reguliere CI op standaard GitHub-runners

**Status:** geïmplementeerde CI-configuratie; volledige uitvoering van de
gedeelde setup op een GitHub-runner is nog niet bewezen. De run op PR-head
`575a5040` stopte vóór runner-toewijzing door een Actions-budgetblokkade.

**Historisch lokaal bewijs van PR #249:** 224 geselecteerde hosttests, pakket-/firmwarecontrole,
geïsoleerde APT-resolutie en een gepauzeerd QEMU/TCG-proces zijn vastgelegd
in [het brongebonden controleverslag](verification/ci-standard-runners-20261108.json).
Daarbij is Python 3.12.14 gebruikt, niet de CI-pin 3.12.12. De volledige
sudo-installatiestap, Rust-/Node-hostsuite en eigen-OS-QEMU-starts blijven open.

Deze publicatiekandidaat integreert de setup uit PR #249, commit
`0de9d805a99de22e404cba188362950d8d0f4e39`, met de bewijsregistratie uit
PR #251, commit `dc6c6bca7fdadf078275999294f8d1c275fa6ffa`.
De bovenstaande historische resultaten blijven aan hun oorspronkelijke
bron gebonden en gelden niet als uitvoering van de samengestelde kandidaat.

De 21 reguliere workflows gebruiken `ubuntu-24.04`: 24 jobdefinities en 42
jobs na de bestaande matrixuitbreidingen. De private publicatie-audit
en de gebundelde OSDev-leercollectie zijn in deze kandidaat uitgesloten.
`ci.yml` behoudt de verplichte keten
`host-quality` → `qemu-smoke` en de bestaande time-outs.

## Vaste gereedschappen

| Onderdeel | Versie/selectie | Beheerplek |
|---|---|---|
| Runner | Standaard `ubuntu-24.04`, Linux x64 | `.github/workflows/` |
| Python | 3.12.12 | `.github/actions/setup-python/action.yml` |
| PyYAML voor bestaande host-gatetests | 6.0.3, CPython 3.12/Linux x64-wheel met SHA256 | `.github/actions/setup-python/requirements.txt` |
| Rust | 1.90.0; bestaande 1.91.1-controles behouden | Toolchainbestanden en workflowstappen |
| Node | 24.22.1 voor prototype-/NAS-hosttests | `ci.yml#host-quality` |
| QEMU x86, common en data | `1:8.2.2+ds-0ubuntu1.18` | `.github/actions/setup-qemu/action.yml` |
| OVMF | `2024.02-2ubuntu0.10` | Dezelfde QEMU-setup |
| SeaBIOS | `1.16.3-2` | Dezelfde QEMU-setup |
| iPXE-QEMU | `1.21.1+git-20220113.fbbdc3926-0ubuntu2` | Dezelfde QEMU-setup |
| Optionele swtpm / libtpms0 | `0.7.3-0ubuntu5.24.04.1` / `0.9.3-0ubuntu4.24.04.1` | Alleen `native-system` schakelt deze installatie in |

Alle 17 QEMU-jobdefinities verwijzen naar één gedeelde installatieactie.
Het geïmplementeerde installatiescript laat APT uitsluitend lezen uit
het Ubuntu-snapshot **20261108T000000Z**, met eigen tijdelijke bronnen en
indexen en de officiële Ubuntu-archivekeyring. De exacte versies worden ook
bij vooraf geïnstalleerde pakketten opnieuw geïnstalleerd en gecontroleerd.
Een onbeschikbare versie geeft een fout; er is geen fallback. Het script
verwijdert geen conflicterende pakketten.

De setup bevat controles voor `pc-q35-8.2`, TCG en SHA256 van OVMF CODE/VARS en het
VGA-ROM. Beide firmwarepaden worden expliciet ingesteld:

```text
OS_OVMF_CODE=/usr/share/OVMF/OVMF_CODE_4M.fd
OS_OVMF_VARS=/usr/share/OVMF/OVMF_VARS_4M.fd
```

De reguliere rooktest gebruikt TCG, één gast-CPU en wegwerpimages. KVM is
geen vereiste. Gespecialiseerde SMP- en overige profielen behouden hun eigen
acceptatievoorwaarden en tijdslimieten.

`ubuntu-24.04` fixeert de distributiekeuze, niet iedere byte van de
GitHub-VM. Reeds aanwezige gedeelde hostbibliotheken kunnen nog met het
runnerimage veranderen. De setup registreert daarom ook `ImageOS`,
`ImageVersion`, pakketversies en firmwarehashes. Dit is geen garantie op een
volledig identiek VM-image of bitidentieke OS-builds.

## Host- en QEMU-controles

`host-quality` behoudt alle bestaande Rust-tests, Clippy/fmt, SDK- en
targetbuilds, Python-tests, JavaScript-tests, quantumproeven en de
voortgangs-/schone-werkboomcontrole. Deze bestaande hosttestreeks komt erbij:

```sh
for pattern in test_common.py test_preflight.py test_build_paths.py test_guest_harness.py test_m1_artifact_binding.py 'test_m6c_*.py'; do
  python3 -B -m unittest discover -s tests -p "$pattern"
done
```

Deze 99 hosttests bewaken machinekeuze, onjuiste toolversies, ongeldige
buildpaden, time-outs, ontbrekende markers, onverwachte exitcodes en de
koppeling van bewijs aan gestarte bestanden. Zij starten het eigen OS niet.
De bestaande native rooktest blijft nodig:

| QEMU-controle | Verwachte starts bij volledig succes |
|---|---:|
| M6c normale opslagproef | 1 |
| DocumentSession normaal schrijven en koude recovery | 2 |
| Native quantum-jobinterface | 1 |
| SDK-app met toegangsrecht | 1 |
| SDK-app met geweigerd toegangsrecht | 1 |
| **Totaal** | **6** |

De harde 60-secondenlimiet per gast blijft gelden. Geen markers, negatieve
gevallen of exitcodecontroles zijn versoepeld. Een hosttest, toolinstallatie
of QEMU-processtart bewijst geen geslaagde native OS-test.

## Diagnostiek en lokaal gebruik

De setup schrijft een omgevingsrapport in `RUNNER_TEMP`. `qemu-smoke`
gebruikt in deze kandidaat de gedeelde `preserve-evidence`-action.
Alle 53 expliciete rapport-/logpaden uit PR #249 en de oorspronkelijke
`warn`-policy blijven behouden. De collector schrijft begrensde rapporten
en gelabelde fragmenten eerst naar runlog en job summary. Daarna volgt de
best-effort upload `qemu-smoke-evidence-reports`, met drie dagen retentie.
Een handmatige `workflow_dispatch` met `full_evidence` probeert daarnaast
de volledige oorspronkelijke padselectie als `qemu-smoke-evidence` te bewaren,
met één dag retentie. Diskimages en binaries staan niet in die selectie.
Dit vervangt de directe aanvullende upload uit PR #249; tests en collector
blijven verplicht, alleen de uploadpogingen hebben `continue-on-error`.
Zie [de bewijswerkwijze](CI-EVIDENCE.md) voor budgetten en downloadgrenzen.
De preflight schrijft zijn JSON naar de joblog; er wordt geen
`build/preflight.json` geproduceerd of als opgeslagen bewijs verondersteld.

De lokale `tools/agent_gate.py` slaat GitHub-setupacties en alleen voor
GitHub bedoelde diagnostiek over. De strikte preflight en echte tests blijven
onvoorwaardelijk. Geëxporteerde lokale firmwarepaden blijven volgens
`HOST_KEYS` bruikbaar; een lokale gate vereist geen `RUNNER_TEMP`-rapport.

## Wanneer GitHub geen runner start

Bij [CI-run 42511934820](https://github.com/simonvanieperen/nieuw-os/actions/runs/42511934820)
op PR-head `575a5040` bevestigt GitHub op 8 november 2026 dat een
Actions-budget verder gebruik blokkeert. De hostjob voert geen teststappen
uit; de afhankelijke QEMU-job wordt overgeslagen. Dit is een andere foutfase
dan een artifactquota na uitvoering van tests. Een codewijziging of herhaling
onder dezelfde budgetvoorwaarden heft deze blokkade niet op.

De openbare ontwikkelrichting is vastgelegd in
[LICENSE-PLAN.md](../LICENSE-PLAN.md). De afzonderlijke publicatiecontrole
blijft nodig voordat zichtbaarheid verandert. Standaard GitHub-hosted
runners zijn gratis voor openbare repositories; artifactopslag en andere
limieten hebben een afzonderlijk beleid. De concrete budgetlimiet en
actueel verbruik zijn in deze controle niet ingezien. Controleer na gewijzigde
uitvoeringsvoorwaarden opnieuw de actuele PR-head en haar verplichte gates.

Bronnen: GitHub, [Billing and usage](https://docs.github.com/en/actions/concepts/billing-and-usage)
en [Budgets and alerts](https://docs.github.com/en/billing/concepts/budgets-and-alerts),
geraadpleegd op 8 november 2026. Het brongebonden incidentbewijs staat in
[het hervattingsrapport](verification/ci-resume-249-20261108.json).

## Onderhoud en bewijs

Werk een pin bewust bij: controleer het nieuwe snapshot, de ondertekende
indexen, pakketversies, payload- en firmwarehashes. Werk de gedeelde setup en
dit overzicht samen bij; bij Python hoort een passend gehasht PyYAML-wheel.
Draai vervolgens de relevante hosttests en bestaande native gates op de
nieuwe PR-head. Een oudere groene run accepteert geen nieuwe toolversie.

Canonical ondersteunt snapshots van ten minste twee jaar oud; plan dus
onderhoud zonder automatische fallback. Terugdraaien: gerichte revert via
een PR. Uitvoering en grenzen staan in
[het controleverslag](verification/ci-standard-runners-20261108.json);
pakketbronnen en hashes in
[de provenance](verification/ci-packages-20261108.json).

### Primaire bronnen

- Canonical. (z.d.). *Ubuntu Snapshot Service*. Geraadpleegd op 8 november
  2026, van https://snapshot.ubuntu.com/
- GitHub. (z.d.). *GitHub-hosted runners reference*. Geraadpleegd op
  8 november 2026, van
  https://docs.github.com/en/actions/reference/runners/github-hosted-runners
- GitHub. (z.d.). *actions/setup-python* (v7.1.0, commit
  `6d4fd22b20465ccc0f7354476588e8df7c9ca0e0`) [Software].
  https://github.com/actions/setup-python/tree/6d4fd22b20465ccc0f7354476588e8df7c9ca0e0
- PyYAML. (z.d.). *PyYAML 6.0.3* [Pakketmetadata].
  https://pypi.org/pypi/PyYAML/6.0.3/json

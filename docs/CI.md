# Reguliere CI op standaard GitHub-runners

De 21 reguliere workflows gebruiken `ubuntu-24.04`: 24 jobdefinities en 42
jobs na de bestaande matrixuitbreidingen. De aparte private publicatie-audit
valt buiten deze configuratie. `ci.yml` behoudt de verplichte keten
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

Alle 17 QEMU-jobs gebruiken één gedeelde installatie. APT leest uitsluitend
het Ubuntu-snapshot **20261108T000000Z**, met eigen tijdelijke bronnen en
indexen en de officiële Ubuntu-archivekeyring. De exacte versies worden ook
bij vooraf geïnstalleerde pakketten opnieuw geïnstalleerd en gecontroleerd.
Een onbeschikbare versie geeft een fout; er is geen fallback. Het script
verwijdert geen conflicterende pakketten.

De setup controleert `pc-q35-8.2`, TCG en SHA256 van OVMF CODE/VARS en het
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
bewaart JSON-rapporten en tekst-/buildlogs als `qemu-smoke-evidence`, met
drie dagen retentie. Deze aanvullende upload is best effort: een
opslagquotafout blijft zichtbaar; de teststappen zelf blijven verplicht en
bepalend. Diskimages en binaries vallen buiten deze aanvullende upload.

De lokale `tools/agent_gate.py` slaat GitHub-setupacties en alleen voor
GitHub bedoelde diagnostiek over. De strikte preflight en echte tests blijven
onvoorwaardelijk. Geëxporteerde lokale firmwarepaden blijven volgens
`HOST_KEYS` bruikbaar; een lokale gate vereist geen `RUNNER_TEMP`-rapport.

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

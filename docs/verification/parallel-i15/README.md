# I15 — sluit twee concrete compilerbindingsgaten

## Uitkomst

De twee G12-bevindingen zijn gereproduceerd en hersteld in de bestaande
DocumentSession-poort. Het oude harnas accepteerde een verwisselde
rlib-dependency en een tweede conflicterende ABI-optie in een verder geldig
compilerrecord. De herstelcommit wijzigt alleen die twee controles, de
gerichte harnastests en deze bewijsmap.

Basiscommit: `4ce6ba2f5bf7201848842fb10d54c7726ab975be`.
Echte oorspronkelijke G11-compilerrecords komen van bron
`91bf66ca2cdc00b7d2e220cd5afefd270d8fe062`, uit
`wt-g11/build/parallel-g11-document/logs`. De proeven wijzigen uitsluitend
tijdelijke kopieën. Originele compilerrecords, libraries, ELF en gastbewijs
blijven ongemoeid.

## Herstelde grenzen

De poort eist nu voor **iedere** compile een exacte dependencyset, inclusief
het juiste BUILD-relatieve rlib-pad. Ontbrekende, extra, dubbele en verwisselde
dependencies worden afgewezen.

| Compile | Exact toegestane directe dependencies |
| --- | --- |
| `nieuw_os_sdk_runtime` | Geen |
| `catalog_core` | Geen |
| `nieuw_os_catalog_client` | Runtime en catalog-core |
| `nieuw_os_document_session` | Runtime, catalog-core en catalog-client |
| `os_user_m6c_probe` | Alle vier bovenstaande libraries |

Voor `panic=abort`, `code-model=small`, `relocation-model=static` en
`no-redzone=yes` is precies **één** vermelding met de gekozen waarde vereist.
Een extra vermelding wordt afgewezen, zowel vóór als ná de oorspronkelijke
optie en ook wanneer de waarde gelijk is. De specifieke ABI-parser herkent
gescheiden en aangehechte `-C`-vormen en `--codegen` met of zonder `=`. Een
underscore in dezelfde keynaam kan een tweede vermelding niet verbergen.

De daadwerkelijke vijf ongewijzigde G11-compilerrecords blijven geaccepteerd.
Er zijn geen build-, kernel-, observer- of schijforaclewijzigingen gemaakt.

## Rode en groene controles

| Controle | Voor herstel | Na herstel |
| --- | --- | --- |
| Oorspronkelijke echte G11-records | Geaccepteerd | Geaccepteerd |
| G12: catalog-client wijst naar vreemde catalog-core-rlib | Ten onrechte geaccepteerd | Afgewezen |
| G12: latere `-C no-redzone=no` | Ten onrechte geaccepteerd | Afgewezen |
| 29 gerichte mutaties van echte recordkopieën | 28 ten onrechte geaccepteerd; één bestaande bincontrole wees al af | Alle 29 afgewezen |
| Harnasmethoden | Bestaande 20 | Alle 23 geslaagd |

De 29 mutaties omvatten de exacte dependencygrenzen per compile, ontbrekende
en dubbele dependencies, alle vier ABI-keys met eerdere/latere conflict- en
duplicaatwaarden en zes alternatieve schrijfwijzen van dezelfde ABI-key.
Dit zijn gecontroleerde hostfixtures; er wordt geen nieuwe compileruitvoering
of VM geclaimd.

`g12-original-red.json` en `g12-original-green.json` behouden de oorspronkelijke
G12-repro. `expanded-red.json` en `expanded-green.json` bevatten de 29
afzonderlijke gevallen en hashes van originele en gemuteerde records.
`checks.json` bewaart de exacte uitgevoerde opdrachten, stdout/stderr en
herstelde bronhashes. `compiler_binding_cases.py` reproduceert de gerichte
mutaties uitsluitend in een tijdelijke map.

Uitgevoerde harnasopdracht:

```sh
python3 -B -m unittest discover -s tests -p test_document_session_harness.py -v
```

Exitstatus 0, **23 methoden geslaagd**. De gerichte repro kan opnieuw tegen
een broncheckout met de poort en de bewaarde G11-compile-artifacts worden
uitgevoerd:

```sh
python3 -B docs/verification/parallel-i15/compiler_binding_cases.py \
  /path/to/gate-checkout /path/to/g11-checkout
```

## Vervolg en grenzen

De twee daadwerkelijke DocumentSession-VMs zijn nog niet gestart. I14 wacht
op integratie van deze fix, de onafhankelijke G12-hercontrole en de centrale
bronfreeze. Dit herstel sluit de twee benoemde metadata-validatiegaten; het
is geen algemene audit van alle rustc-opties of een bewijs van een willekeurige
compiler/toolchain. Roadmapvoorstel: **0 punten wijziging**.

# CI-testbewijs zonder afhankelijkheid van artifactquota

**Besluit:** ADR-CI-EVIDENCE-20261108. De testuitkomst, het compacte
runbewijs en de beschikbaarheid van een download zijn afzonderlijke resultaten.
Een mislukte GitHub-upload maakt een geslaagde test niet rood. Een mislukte
test, build, validator of bewijscollector blijft wel een fout.

## Normale runs

De zestien workflows die bewijs bewaren gebruiken
`.github/actions/preserve-evidence/action.yml`. De action leest alleen de
expliciet opgegeven bestaande bewijsselectie en doet het volgende:

1. Leg vóór de upload de echte checkout-SHA, GitHub-SHA, run-ID, poging,
   jobstatus en een inventaris vast. `tools/ci_evidence.py` schrijft dit naar
   de runlog, de job summary en `_meta/receipt.json` in een tijdelijke map.
2. Bewaar compacte UTF-8-rapporten en tekstlogs met hun oorspronkelijke
   repositorypaden en SHA-256. JSON-uitslagen worden niet herschreven.
   Binaire builds, schijfimages, ruwe frames en ZIP-bestanden vallen buiten
   deze selectie. Ontbrekende, te grote en uitgesloten bestanden blijven
   zichtbaar in het receipt.
3. Probeer het compacte pakket als `<oude-artifactnaam>-reports` te uploaden:
   drie dagen bewaartijd, compressieniveau 9. De compacte upload, de volledige
   upload en de afzonderlijke Windows-pakketupload hebben alle drie
   `continue-on-error: true` en zijn dus best effort.
4. Toon de werkelijke uploaduitkomst via `steps.<id>.outcome`, met alleen
   bij succes een download-URL. Bij een quotafout blijft het al geschreven
   runbewijs beschikbaar.

De collector begrenst de bewaarde verzameling tot 8 MiB en 500 bestanden,
inclusief het eigen receipt; een afzonderlijk rapport is maximaal 1 MiB.
De console bevat kleine JSON-rapporten en gelabelde logfragmenten, met een
gezamenlijk uitvoerbudget van circa 256 KiB. Een afgekapt fragment is expliciet
gemarkeerd. Verborgen bestanden, symlinks en paden buiten de checkout worden
niet gevolgd. Dit is geen inhoudelijke secretscanner: de aanroepende
workflow moet uitsluitend voor CI bedoelde, niet-gevoelige bewijsbestanden
selecteren.

GitHub telt runlogs en job summaries niet mee voor het artifactopslagquotum.
Hun normale retentie blijft gelden; dit beleid verkort die niet.

## Volledig archief voor hercontrole

Open de betreffende workflow onder **Actions → Run workflow** en zet
`full_evidence` aan. De tests worden op de gekozen branch opnieuw uitgevoerd.
De action probeert daarna tevens de oorspronkelijke volledige padselectie
onder de oorspronkelijke artifactnaam te bewaren, met **één dag** retentie.
De compacte registratie blijft ook bij deze uitvoering bestaan.

Gebruik deze route wanneer ruwe schijfimages, pixels of buildbytes nodig
zijn voor onafhankelijke hercontrole, zoals bij
`docs/verification/materiaal-library-20261102/verify-ci-artifact.py`.
Download het benodigde archief vóór de vervaldatum. Compacte rapporten en
hashes vervangen de ruwe bytes niet. Een mislukte volledige upload betekent
dat dit archief niet beschikbaar is; een groene testjob bewijst het tegendeel
niet. Acceptatie die ruwe bytes vereist blijft zonder dat bewijs open.

## Downloadbare Windows-testpakketten

`windows-qemu-test-version.yml` en `native-window-session.yml` blijven na
geslaagde build- en teststappen hun ZIP apart aanbieden, onder
`<oude-artifactnaam>-bundle`, met één dag retentie en zonder extra compressie
van de al gecomprimeerde ZIP. Bij een volledige handmatige bewijsrun zit het
pakket in het volledige archief en volgt geen dubbele upload.

De job summary vermeldt afzonderlijk of een **geverifieerde download**
beschikbaar is. Bij een mislukte upload, eerdere testfout of overgeslagen
pakketstap is een Windows-testversie niet opgeleverd. De bestaande native
tests en pakketvalidatie blijven verplicht. Uitvoering van de ZIP op Windows
wordt hierdoor niet bewezen.

## Quota en bestaande artifacts

De reparatie wist geen oude bewijsarchieven en verandert geen betaal- of
repositoryrechten. De actuele accountbrede opslag kon in deze sessie niet
volledig worden geïnventariseerd. De gecontroleerde recente rungebonden
artifactlijsten waren leeg; daaruit volgt geen bewezen totale bezetting.

Een lagere retentie geldt voor nieuwe uploads. Oude artifacts behouden hun
bestaande vervaldatum. Voor gerichte opruiming: inventariseer eerst namen,
run, datum en omvang; bewaar nodig acceptatiebewijs; verwijder uitsluitend
de geselecteerde verouderde artifacts. Verwijder geen volledige runs om
opslag vrij te maken, want daarmee verdwijnt ook de terugval in de runlog.
GitHub kan 6–12 uur nodig hebben om opslaggebruik te herberekenen. De
werkflow hoeft daar voor zijn testuitkomst niet meer op te wachten.

## Workflow syntax validation

The required `host-quality` job runs actionlint v1.7.12 immediately after
checkout. It checks GitHub Actions YAML, expressions, and workflow semantics.
The invocation uses `-shellcheck= -pyflakes=` because shell and Python quality
remain owned by the existing required checks; this step does not claim those
optional integrations.

The job downloads only `actionlint_1.7.12_linux_amd64.tar.gz`, verifies SHA-256
`8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8`, and
extracts only the `actionlint` executable into the runner's temporary directory.
The upstream `v1.7.12` tag points directly to commit
`914e7df21a07ef503a81201c76d2b11c789d3fca`; the release was published on
March 30, 2026, under the MIT license. The tag commit is unsigned. The verified
release checksum therefore detects asset substitution relative to the upstream
checksum list, but it is not a signed provenance chain or an audit of the
actionlint source and build process.

## Controle en terugdraaien

`python3 -B -m unittest discover -s tests -p 'test_ci_evidence*.py'`
controleert uitsluitend de bewijshulp op de host. `host-quality` voert deze
controle voortaan ook uit. De bestaande native testcommando's en gates zijn
ongewijzigd. Lokaal gesimuleerde uploaduitkomsten zijn geen echte GitHub-upload
of nieuwe QEMU-acceptatie.

Terugdraaien: revert de CI-bewijscommit via een PR. Dat herstelt de oude
uploadstappen en hun blokkerende gedrag. Verwijder geen bron-, voortgangs-
of bewijswijzigingen uit andere commits.

## Primaire bronnen

GitHub. (z.d.). *GitHub Actions billing*. Geraadpleegd op 8 november 2026,
van https://docs.github.com/en/billing/concepts/product-billing/github-actions

GitHub. (z.d.). *Contexts reference*. Geraadpleegd op 8 november 2026,
van https://docs.github.com/en/actions/reference/workflows-and-actions/contexts

GitHub. (z.d.). *Metadata syntax reference*. Geraadpleegd op 8 november 2026,
van https://docs.github.com/en/actions/reference/workflows-and-actions/metadata-syntax

GitHub. (z.d.). *Workflow commands for GitHub Actions*. Geraadpleegd op
8 november 2026, van
https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands

GitHub Actions. (z.d.). *upload-artifact* [Broncode en documentatie].
Geraadpleegd op 8 november 2026, van https://github.com/actions/upload-artifact

rhysd. (2026). *actionlint v1.7.12 release*.
https://github.com/rhysd/actionlint/releases/tag/v1.7.12

rhysd. (2026). *actionlint v1.7.12 checksums*.
https://github.com/rhysd/actionlint/releases/download/v1.7.12/actionlint_1.7.12_checksums.txt

rhysd. (2026). *actionlint source at v1.7.12*.
https://github.com/rhysd/actionlint/tree/914e7df21a07ef503a81201c76d2b11c789d3fca

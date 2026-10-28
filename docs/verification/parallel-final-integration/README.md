# Gecombineerde artifactcontrole

`artifact-audit.json`: 80 unieke echte VM-mappen, alle direct opnieuw gehashte
artifacts en gearchiveerde oorspronkelijke resultaten komen overeen. Het
script voert geen VM uit en telt hostfixtures niet mee. Bron3e41bbf9,
fingerprint307c828e... . De scope is bron-/artifactbinding; afzonderlijke
suitecapturers en gerichte crosschecks beoordelen de inhoud van de proeven.

De eerste versie van dit controlehulpmiddel verwachtte voor ieder rapport
`timed_out=false` en `artifact_sha256`. Het bestaande G0-schema gebruikt
`error=null`, exacte checks en `artifacts`. Daardoor wees die versie uitsluitend
het groene G0-rapport onterecht af en hasht hij de G0-artifacts nog niet.
`initial-audit-schema-rejection.json` bewaart dat resultaat. Het huidige audit.py
beoordeelt het expliciete G0-schema, vereist alle tien concrete checks, raw33,
geen error en een werkelijk gemeten duur onder60s; het hasht alle15 originele
G0-artifacts. Gastresultaten, bronsuites en succescriteria zijn niet gewijzigd.

De tweede controle vond vervolgens een echte afwijking in de oorspronkelijke
bewaarde G0serialfile; retained-g0-artifact-rejection.json bewaart die afwijzing.
Dat artifact is niet gerepareerd en de oude run wordt niet meegeteld.
De laatste audit controleert de verse private pipe-run uit
../parallel-final-g0-confirmed/pipe samen met de79 andere oorspronkelijke
native runs. Alle80 unieke VM-resultaten en616 artifacthashes kloppen nu.
De afzonderlijke mislukte UNIX-omgevingsprobe telt niet als geaccepteerde VM.

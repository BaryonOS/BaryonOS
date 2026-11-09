---
name: Ontwikkeltaak
about: Een afgebakende implementatie met concrete acceptatie en bewijsgrens
title: "taak: "
---

<!--
Gebruik dit template nadat het doel voldoende duidelijk is. Bespreek onbesliste
product- of architectuurkeuzes eerst in Discussions en link het besluit hieronder.
Lees AGENTS.md, relevante lokale instructies en docs/TASK-TEMPLATE.md.

Gebruik het metadata-blok alleen voor een taak in de autonome pipeline.
Verwijder het blok voor handmatig onderhoud zonder open rubriccriterium en
beschrijf de relevante controles onder Acceptatie en bewijs.
Voor de pipeline vervang je de velden: Criterium moet één open ID uit
docs/progress.json zijn. Gate noemt één of meer bestaande workflow.yml#job
verwijzingen, gescheiden door spaties. Dit zijn geplande controles, geen uitslagen.
Voeg alleen bij echte afhankelijkheden in het blok een regel toe zoals
`Afhankelijk van: #12 #34`; laat die regel anders weg.
Verwijder deze uitleg voor het indienen.
-->

## Taakmetadata

```text
Criterium: VUL_OPEN_ID_IN
Gate: VUL_WORKFLOW.yml#VUL_JOB_IN
```

## Doel en samenhang

Welk concreet probleem lost deze stap op? Link de relevante discussie, ADR,
FR-acceptatie-ID en bestaande issues/PR's; voorkom een tweede uitvoering van hetzelfde werk.

## Scope en interfaces

- Basiscommit en relevante bronbestanden:
- Toegestane schrijfbestanden en eigenaar van gedeelde interfaces:
- Te behouden invariant, capability-/privacygrens en eventuele dependency- of formaatgevolgen:
- Buiten scope:

## Acceptatie en bewijs

- Waarneembaar resultaat en exact uit te voeren commando's/gates:
- Passende positieve én negatieve gevallen, inclusief falen/herstel waar relevant:
- Vereist bewijsniveau: host, native VM, fysiek of menselijke controle:
- Vast te leggen bron-/imageversie, exitstatus en bereikbare log-/rapport-/hashverwijzingen:
- Wat deze stap nog niet bewijst en hoe hij kan worden teruggedraaid:

## Uitvoering

Een maintainer beoordeelt scope, afhankelijkheden en gates en kiest zo nodig
**één** label: `pipeline:serial` of `pipeline:parallel`. Dit template start zelf
geen agent en kent geen label, eigenaar of voortgangspunten toe. Wijzigingen aan
pipeline, workflows en andere beschermde bestanden volgen een afzonderlijk
beoordeelde PR, niet de autonome pipeline. Gebruik uitsluitend synthetische
testgegevens; mogelijke kwetsbaarheden volgen [SECURITY.md](https://github.com/simonvanieperen/nieuw-os/blob/main/SECURITY.md).

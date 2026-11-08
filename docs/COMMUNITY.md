# Discussies, meldingen en bijdragen

Gebruik [Discussies](https://github.com/simonvanieperen/nieuw-os/discussions)
voor vragen, voorstellen en ervaringen, en
[Issues](https://github.com/simonvanieperen/nieuw-os/issues/new/choose) voor
reproduceerbare defecten en afgebakend uitvoeringswerk. Zoek eerst of het
onderwerp al bestaat en sluit daar waar mogelijk bij aan.

## Kies de passende plek

| Je wilt… | Plek | Wat helpt |
|---|---|---|
| Een vraag stellen of hulp krijgen | [Vragen en hulp](https://github.com/simonvanieperen/nieuw-os/discussions/categories/vragen-en-hulp) | Doel, context en wat je al hebt geprobeerd; een bruikbaar antwoord kan als antwoord worden gemarkeerd. |
| Een technische keuze afwegen | [Architectuur en ontwerp](https://github.com/simonvanieperen/nieuw-os/discussions/categories/architectuur-en-ontwerp) | Huidig contract, voorstel, alternatieven, interfaces en toetsbare gevolgen. |
| Een gebruikstaak of toegankelijkheid verbeteren | [Product en gebruikservaring](https://github.com/simonvanieperen/nieuw-os/discussions/categories/product-en-gebruikservaring) | Gewenste handeling, ervaren knelpunt en herkenbare voorbeelden. |
| Een experiment, demo of test delen | [Testresultaten en demos](https://github.com/simonvanieperen/nieuw-os/discussions/categories/testresultaten-en-demos) | Exacte bron, opstelling, waarneming, bewijs en resterende grenzen. |
| Voorkeuren tussen concrete opties verzamelen | [Peilingen](https://github.com/simonvanieperen/nieuw-os/discussions/categories/peilingen) | Een duidelijke vraag en vergelijkbare opties; stemmen vervangen geen technisch besluit of acceptatiebewijs. |
| Projectnieuws of een vastgesteld besluit delen | [Aankondigingen](https://github.com/simonvanieperen/nieuw-os/discussions/categories/aankondigingen) | Een maintainer plaatst het bericht met links naar de leidende bron. |
| Een defect, CI-probleem of concrete taak melden | [Issueformulieren](https://github.com/simonvanieperen/nieuw-os/issues/new/choose) | Reproductie of afgebakend resultaat, bronversie en passende controle. |

Blijf bij het onderwerp, onderbouw technische uitspraken en maak onderscheid
tussen voorstel, code, uitgevoerde test en aanname. Nederlands is de
standaardtaal; technische identifiers blijven herkenbaar.

## Van gesprek naar uitvoering

1. Bespreek probleem, opties en grenzen in een bestaande of nieuwe discussie.
2. Leg een aangenomen architectuur- of contractwijziging vast in
   [DECISIONS.md](DECISIONS.md) en de betrokken specificatie. Een idee of
   peiling is op zichzelf geen besluit.
3. Maak of koppel één uitvoerbaar issue met resultaat, scope, afhankelijkheden,
   acceptatiecommando's en relevante bestaande FR-/rubric-ID's. Discussies
   vormen geen tweede backlog of roadmap.
4. Werk op een kleine branch en open eerst een concept-PR volgens
   [CONTRIBUTING.md](../CONTRIBUTING.md) en [AGENTS.md](../AGENTS.md).
5. Koppel werkelijk uitgevoerd bewijs aan de geteste bron. Verwerk de uitkomst
   in de bestaande status-/acceptatieketen; vermeld ook open en mislukte checks.

Een hosttest, bare-metalbuild, native QEMU-proef, fysieke test en menselijke
review bewijzen verschillende zaken. Een screenshot toont wat is gerenderd,
maar bewijst geen opslagduurzaamheid of afgedwongen rechten. Deelproeven en
demo's leveren alleen rubricpunten op als het volledige bestaande criterium
met het vereiste bewijs is geaccepteerd.

## Leidende projectbronnen

| Onderwerp | Bron |
|---|---|
| Actuele stand en volgende stap | [PROJECT-STATE.md](PROJECT-STATE.md) |
| Volledige releaseopdracht en acceptatie | [FULL-TEST-RELEASE.md](FULL-TEST-RELEASE.md) |
| Vaste ontwikkelcriteria en voortgang | [PROGRESS.md](PROGRESS.md) en [progress.json](progress.json) |
| Architectuur en beveiligingsgrenzen | [KERNEL-AND-SECURITY-CONTRACT.md](implementation/KERNEL-AND-SECURITY-CONTRACT.md) en [DECISIONS.md](DECISIONS.md) |
| Privacy, NAS, internet en lokale AI | [LOCAL-PRIVACY-CONTRACT.md](LOCAL-PRIVACY-CONTRACT.md) |
| Productervaring en desktop | [DESIGN-AMBITION.md](DESIGN-AMBITION.md) en [DESKTOP-SHELL.md](implementation/DESKTOP-SHELL.md) |
| Tests, builds en brongebonden bewijs | [TEST-STRATEGY.md](TEST-STRATEGY.md) en [DEVELOPMENT.md](DEVELOPMENT.md) |
| Herkomst en licentiekeuzes | [LICENSE-PLAN.md](../LICENSE-PLAN.md) en [DEPENDENCIES.md](DEPENDENCIES.md) |

De repository blijft leidend. Discussies en de wiki verwijzen naar deze
bronnen; zij vervangen geen versiegebonden contract of bewijsrapport.

## Kwetsbaarheden en gevoelige gegevens

Volg [SECURITY.md](../SECURITY.md). Het daar beschreven meldproces en eventuele
open gaten blijven bepalend; deze wegwijzer introduceert geen meldadres of
responstermijn. Zet geen secrets, productiekeys, persoonsgegevens of
exploiteerbare kwetsbaarheidsdetails in publieke issues, discussies of logs.
Een algemeen architectuurgesprek over beveiliging hoort wel bij Discussies.

## Voor maintainers

Formulieren voegen nooit automatisch `pipeline:serial` of `pipeline:parallel`
toe. Deze labels zijn invoer voor de bestaande
[agentpipeline](../tools/agent_pipeline.py) en worden pas na beoordeling
expliciet toegekend. Een bericht of formulier is geen automatische
uitvoeringsopdracht. Een pipeline-issue heeft precies één van deze labels,
één open rubriccriterium en bestaande workflowjobs. De issuebody bevat
letterlijk deze regels, met de placeholders vervangen:

```text
Criterium: <open ID uit docs/progress.json>
Gate: <workflow>.yml#<job>
Afhankelijk van: #<issue-nummer>
```

De afhankelijkheidsregel is optioneel; `Gate:` mag meerdere jobverwijzingen
bevatten, gescheiden door spaties. GitHub-formuliervelden worden Markdown;
de kop `Criterium` alleen vervangt de letterlijke regel niet. De pipeline-
labels en technische wijzigingsgrenzen blijven onder maintainerbeheer.

De vier discussieformulieren staan onder `.github/DISCUSSION_TEMPLATE/`.
Hun bestandsnamen volgen exact de categorieslugs. Een hernoemde categorie
vraagt daarom ook controle van het bijbehorende formulier en de links
hierboven. Formulieren worden actief vanaf de standaardbranch; peilingen
gebruiken GitHubs eigen invoer en hebben geen categorieformulier.

Officiële inrichtingsdocumentatie (geraadpleegd 8 november 2026):

- GitHub. (z.d.). [Creating discussion category forms](https://docs.github.com/en/discussions/managing-discussions-for-your-community/creating-discussion-category-forms).
- GitHub. (z.d.). [Syntax for discussion category forms](https://docs.github.com/en/discussions/managing-discussions-for-your-community/syntax-for-discussion-category-forms).
- GitHub. (z.d.). [Managing categories for discussions](https://docs.github.com/en/discussions/managing-discussions-for-your-community/managing-categories-for-discussions).

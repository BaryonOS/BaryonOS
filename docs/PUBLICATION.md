# Afzonderlijke publicatiekandidaat — hervat 8 november 2026

**Deze bronmomentopname is voorbereid voor een nieuwe, onafhankelijke repository.
De huidige `simonvanieperen/nieuw-os` is nog privé; de afwijkende bestemming
`simonvanieperen/nieuw-os-public` is niet bevestigd of aangemaakt.**

De hervatting heeft het eerdere werk daadwerkelijk teruggevonden: het auditdossier
staat in PR #259 en de kandidaat in concept-PR #260. De vorige kandidaat op
`83d5ee35380d0c48ac2e1890614840461bdd2d01` is lokaal volledig teruggevonden:
6.468 bestanden met exact Git-tree `2502db4122acd92b93fc5dda972920d59b9ba507`.
Die versie is het herstelpunt, niet de uiteindelijke hash van de bijgewerkte boom.

## Waarom een afzonderlijke publicatie nodig is

De oorspronkelijke bron bevat een geïntegreerde OSDev-/boekcollectie. Voor twee
daadwerkelijk behouden prozapassages uit een wiki-import uit 2006 is geen
herpublicatiegrant vastgesteld. Dit is geen oordeel dat de gehele wiki onvrij is
of dat de eigen kernelcode dit probleem heeft. Het dossier in PR #259 vermeldt
pagina W001473, de bronblob en passagehashes zonder de betrokken tekst opnieuw
in de openbare kandidaat over te nemen.

Verwijderen uit de nieuwste commit verwijdert eerdere Git- en PR-kopieën niet.
Ook de volledige reguliere Actions-historie van de oorspronkelijke repository
is niet afgetekend. De afzonderlijke kandidaat importeert die geschiedenis,
oude PR-refs, issues, wiki, discussies, Actions-runs, artifacts en caches niet.

De praktische uitsluitingsgrens blijft `.agents/skills/os-dev/`: 78 bestanden,
16.304.389 bronbytes. De eigen OS-code blijft behouden. Het boek, eigen
bronhulpmiddelen en metadata kunnen later afzonderlijk onder hun passende
voorwaarden worden opgenomen. De huidige leerroute verwijst naar primaire
online bronnen. De oorspronkelijke repository blijft intact en privé.

## Actuele samenstelling

| Onderdeel | Vastgelegde bron |
|---|---|
| Productcode en oorspronkelijke bewijsbestanden | `363b486b06023d3d578f8eca2b23ff49045a1757` |
| Main inclusief besluit openbare ontwikkeling uit #250 | `5b2189824c32145ad5c110429517dfcea3f2720f` |
| Herstelde eerdere kandidaat | #260, `83d5ee35380d0c48ac2e1890614840461bdd2d01` |
| Actuele hosted CI en vaste tools | #249, `0de9d805a99de22e404cba188362950d8d0f4e39` |
| Actuele bewijscollector, tests en reviewreparaties | #251, `dc6c6bca7fdadf078275999294f8d1c275fa6ffa` |

De bijgewerkte kandidaat bevat Python 3.12.12, het gehashte PyYAML 6.0.3-wheel,
de vaste QEMU-/firmwarepakketten en de relevante bestaande host-/QEMU-gates uit
#249. De collector en de bijbehorende foutafhandeling uit #251 blijven behouden.
De extra compacte diagnostiek van `qemu-smoke` gebruikt eveneens de gedeelde
bewaaractie; de exacte integratie en daadwerkelijk uitgevoerde controles staan
in [het hervattingsbewijs](verification/publication-resume-20261108.json).

De officiële MPL-2.0-tekst en het exacte padregister blijven leidend voor eigen
software. Nieuwe eigen CI-configuratie is aan `MPL-2.0-FILES.txt` toegevoegd.
Externe componenten, fonts en notices behouden hun afzonderlijke voorwaarden;
documentatie en assets krijgen geen algemene MPL-toepassing. Zie
[LICENSING.md](../LICENSING.md) en [THIRD-PARTY.md](../THIRD-PARTY.md).

## Bewijs en grenzen

Het eerdere onderzoek controleerde alle 6.524 bestanden van bron `363b486b`,
112 exacte registryversies en 361 checksumvermeldingen in 62 lockfiles. De
latere main-documentatie en geselecteerde CI-overlays zijn afzonderlijk aan hun
Git-blobs verbonden. De hervatting verifieert de nieuwe combinatie en legt de
werkelijke commando's, exitcodes, aantallen en bronhashes opnieuw vast.

De eerdere 470 beleidscontroles horen bij de oude kandidaat. Ze worden niet als
herhaling op deze gewijzigde CI-configuratie gepresenteerd. Ook de historische
native uitslagen en echte artifactquota-terugval uit #251 blijven aan hun
oorspronkelijke bron en GitHub-runs gebonden.

Het actuele [eindmanifest](verification/publication-final-20261108.json) bevat
de nieuwe bestandsinventaris, wijzigingen, MPL-scopehash en inhoudsdigest.
De digest sorteert expliciet op UTF-8-bytes van het repositoryrelatieve pad;
alleen het eindmanifest zelf wordt ervan uitgesloten. De private reviewcommit
heeft nog ouders in de oude repository. Alleen de bronboom, zonder die ouders,
mag naar een eventueel nieuw publicatiedoel worden overgebracht.

Er is geen nieuwe Rust-/QEMU-acceptatie op deze samengestelde boom uitgevoerd.
De oude kandidaatrun 42513558667 had voor `host-quality` geen runner of stappen;
`qemu-smoke` was overgeslagen. De eerdere GitHub-runmelding noemt een Actions-
budgetblokkade. Dat is geen geslaagde test en wordt niet omzeild met zwakkere
branchregels of een gefingeerde groene uitslag. De score blijft **41/100**;
R03 en de volledige productrelease blijven open.

## Uitvoering na keuze van de bestemming

Het concrete voorstel is `simonvanieperen/nieuw-os-public` met een nieuwe
geschiedenis. De huidige `nieuw-os` behoudt haar naam, inhoud, branches, PR's,
wiki en logarchieven in privévorm. Er worden geen oorspronkelijke Git-objecten,
secrets of eigen runnerregistraties naar het nieuwe doel overgenomen.

De nieuwe repository gebruikt standaard GitHub-runners en de bestaande
relevante host-/QEMU-gates. Repository- en Actions-instellingen moeten op het
werkelijke nieuwe doel worden gecontroleerd. Werkelijke CI-uitvoering en
openbaarheid worden na publicatie apart vastgesteld.

De oorspronkelijke gebruikersopdracht autoriseert openbaarmaking van de
bestaande repository na afgeronde controle. Een andere repository met nieuwe
geschiedenis is een materiële wijziging van dat doel en is nog niet gekozen.
Deze concept-PR blijft daarom zonder auto-merge en is niet bedoeld als gewone
cleanup-merge in de oorspronkelijke main. Geen geschiedenis is herschreven of
verwijderd en geen zichtbaarheid of facturering is aangepast.

## Bronnen

OSDev Wiki-bijdragers. (2011). *OSDev Wiki:License*.
https://wiki.osdev.org/OSDev_Wiki:License

OSDev Wiki-bijdragers. (2011). *OSDev Wiki:Copyrights*.
https://wiki.osdev.org/OSDev_Wiki:Copyrights

GitHub. (z.d.). *Removing sensitive data from a repository*.
https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository

Mozilla. (z.d.). *Mozilla Public License, version 2.0*.
https://www.mozilla.org/en-US/MPL/2.0/

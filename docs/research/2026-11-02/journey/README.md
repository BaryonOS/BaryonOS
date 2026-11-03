# Fase 4 — installatie, eerste gebruik en herkenning

Onderzocht op **2 november 2026**. Uitbreiding van [het fase 2–4-onderzoek](../README.md), uitsluitend documentatie. Geen installer/onboarding gebouwd, installatie-images gedownload, echte schijven gebruikt, OS-sessies doorlopen of deelnemers getest.

## Onderzoeksvragen en methode

Welke installatiebeslissingen zijn gedeeld, waar verschillen daadwerkelijke routes, welke handelingen van de eerste werkdag zijn overdraagbaar en welke verwachtingen moet nieuw-os ondersteunen? Welke innovatie lost een concreet probleem op zonder bekende basisacties opnieuw te laten leren?

Zoekroute: officiële actuele downloads/releases → versiegebonden installatie-/herstelgids → desktoptaakhandleidingen → vaste broncodetag/configuratie wanneer de gids te algemeen, verouderd of tegenstrijdig is. Windows/macOS, Linux-installers en dagelijkse bediening zijn parallel onderzocht en centraal samengevoegd. Zoekclusters: Windows 26H2 media/Setup/OOBE, macOS Recovery/Setup Assistant/FileVault, Ubuntu 26.04-installatie, Fedora Live Anaconda hidden pages, Kubuntu Calamares-manifest/patches, GNOME 51- en Plasma 6.8-taakroutes, mental models en W3C cognitieve toegankelijkheid.

Voorkeur voor de ondersteunde Ubuntu/Kubuntu-LTS; Fedora actuele stabiele Workstation. Upstream GNOME 51/Plasma 6.8 in dagelijkse bediening worden niet toegeschreven aan distributies die GNOME 50/Plasma 6.6 bevatten. Dynamic supportpagina's zijn geen gepinde ISO-opname. Broncode bewijst configuratie/logica, geen end-to-end-succes.

Labels: **[BRON]** controleerbare beschrijving, **[ANALYSE]** duiding, **[HYPOTHESE]/[VOORSTEL]/[ONTWERPKEUZE]** nog te toetsen richting, **[REPOSITORY]** exact afgebakende code-/bewijsbasis. Conceptuele staptellingen zijn documentanalyse, geen gemeten klikken, tijden, foutkansen of bewezen bekendheid. Niet-vastgestelde details blijven zichtbaar onbekend.

## Leesroute en geleverde artefacten

| Document | Opbrengst |
|---|---|
| [Installatievergelijking](../../os-installation-comparison.md) | 31 aspecten × Windows/macOS/Ubuntu/Fedora/Kubuntu plus expliciet toekomstige nieuw-os-kolom; werkelijke volgorde, A/B/C, veilige automatisering |
| [Windows/macOS-brondossier](windows-macos-installation.md) | 31 aspecten, clean/in-place/Recovery/Intel-varianten, WI-bronregister |
| [Linux-brondossier](linux-installation.md) | 31 aspecten, distributieprofielen/versies/patches, LI-bronregister |
| [Dagelijkse interactie](../../os-interaction-patterns.md) | 28 taken × Windows/macOS/GNOME/KDE/nieuw-os, telmethode, termen/iconen/toetsen, UI-bronregister |
| [Overdraagbare mentale modellen](../../transferable-mental-models.md) | Gedeelde concepten, herkomstprofielen, verwachtingscompatibiliteit, familiarity budget, J-bronregister |
| [Overstapfrictie](../../switching-friction.md) | Herkomstspecifieke hypotheses, compatibiliteit en installatierisico's; verband met fase 3 |
| [Installer](../../../product/new-os-installer-spec.md) | Volledige gemotiveerde funnel, stapcontracten, grens vóór iedere mutatie, open backendgates en IN-acceptatie |
| [Onboarding](../../../product/new-os-onboarding-spec.md) | Eén UI, optionele begeleiding, lokaal inert aliasregister, eerlijke ontbrekendstatus en ON-acceptatie |
| [Interactieprincipes](../../../product/interaction-principles.md) | Meetbare voorstelcriteria, toegankelijkheid en onderzoeks-/verificatieplan |
| [Bestaand productplan](../../../implementation/PRODUCT-SPEC.md) | Begrensde aanvulling na deze vergelijking; geen P2-uitbreiding of implementatie in deze opdracht |
| [Documentcontrole](verification.json) | Matrixdekking, bron-/linkconsistentie, ongewijzigde productboom/rubric, exacte uitgevoerde controles |

De bronnenregisters staan bij het dossier dat de claim bezit, met titel, organisatie, datum/versie, raadpleegdatum, directe URL en beperkingen. De centrale matrix gebruikt dezelfde WI-/LI-ID's en URLs. Bestaande gebruikers-/adoptiebronnen EA/WM/LM/DT blijven in [het oorspronkelijke register](../sources.md). Geen dubbele telling van dezelfde leverancierstekst als onafhankelijke gebruikersstem.

## Synthese, keuzes en verworpen richtingen

[ANALYSE] Herkenbare concepten zijn breder overdraagbaar dan precieze schermposities of toetscombinaties. Vooral app versus venster, zoekbereik, statusgebied, werkruimten, pakketbronnen en account-/schijfgevolgen verschillen. Dat levert toetsvragen op, geen nieuw-os-succesclaim.

[VOORSTEL] Behoud normale werkwoorden en zichtbare gevolgen; verbeter vindbaarheid met lokale synoniemen en breng schijfmutaties achter één volledige autorisatiegrens. Vraag optioneel naar eerdere ervaring voor tijdelijke hulp, niet voor vier shells. Introduceer Materiaal geleidelijk binnen dezelfde basisbediening.

Verworpen voor deze richting: een bestaand OS visueel kopiëren, vier verschillende interfaces, gebruikersherkomst als machtiging, een zoekopdracht die direct systeemacties uitvoert, onbekende schijven als leeg behandelen, dummy-encryptie/authenticatie, AI/provideraccount als noodzakelijke toegangspoort en minder stappen als enige kwaliteitsmaat. Redenen en acceptatie staan in de gekoppelde specificaties. Deze voorstellen wijzigen geen gerealiseerde securitygaranties.

## Beperkingen en volgende bewijsstap

Geen volledige releasegebonden installatie-opname, gebruiksonderzoek, keyboard-/screenreader-/touchproef of fysieke hardwarevalidatie. Enkele macOS-paneelposities, defaults en exacte KDE-staptellingen zijn niet vastgesteld. De KDE-applet-/displayingangen zijn aanvullend op tag v6.8.0 gelezen, maar de volledige backend-/shellroute is niet uitgevoerd. De native referentie blijft exact `1f0dd9c`, los van de mainbasis van deze branch.

Voorkeursvervolg: eerst actuele code-/servicegates hercontroleren, dan een begrensde read-only plan-/aliasproef en veilige taakobservatie. Vóór echte writes: capability-/auth-/encryptie-/recoverybewijs met disposable images en onafhankelijke review. Een menselijke audit en representatieve adoptie-/retentiemeting blijven afzonderlijk nodig. De vaste productscore blijft **34/100** op deze mainbasis; extra documenten leveren geen acceptatiepunten op.

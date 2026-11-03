# Nieuw-os — onboarding vanuit bestaande kennis

Datum: 2 november 2026. **Status: ontwerpspecificatie voor later onderzoek en uitvoering; niet geïmplementeerd en niet door gebruikers gevalideerd.** Onderbouwing: [mentale modellen](../research/transferable-mental-models.md), [overstapfrictie](../research/switching-friction.md) en [28 dagelijkse taken](../research/os-interaction-patterns.md). Deze aanvulling verandert geen huidige P2-route of geaccepteerd native bewijs.

## Probleem en keuze

[ANALYSE] Mensen kunnen een bekende functie onder een andere naam zoeken of een andere betekenis van vensterbediening verwachten. De onderliggende platformverschillen zijn gedocumenteerd; hun omvang bij nieuw-os is nog niet gemeten. [VOORSTEL] Eén interface en één systeemcontract, met optionele begeleiding op basis van eerdere ervaring. Een herkomstkeuze is een aanwijzing voor uitleg, geen betrouwbare bepaling van vaardigheid.

Geen vier desktops, verplichte rondleiding, marketingcarrousel, voorkeur voor een cloudprovider of AI-chat als toegangspoort tot normale handelingen. De [interactieprincipes](interaction-principles.md) bepalen hoe het effect wordt getoetst.

## Eerste sessie en overdracht uit de installer

1. **Sessiestatus:** toon of dit een proefsessie, persoonlijke sessie of beheerde sessie is. Noem een huidige openbare VM-/gastsessie niet beveiligd aangemeld. Een toekomstig persoonlijk account vereist eerst echte authenticatie en opslagisolatie.
2. **Direct bruikbaar:** de desktop is bereikbaar zodra de noodzakelijke configuratie voltooid is. Toon hoogstens een niet-blokkerend aanbod “Wegwijs in nieuw-os”, met “Later” en “Overslaan”. Toegankelijkheidsbediening blijft vanaf het eerste scherm beschikbaar.
3. **Optionele vraag:** “Met welk systeem werk je meestal?” Antwoorden Windows, macOS, Linux, eerste computer / geen voorkeur. Bij Linux is GNOME, KDE Plasma of Ubuntu een optionele vervolgvraag; “Weet ik niet” blijft geldig. Meerdere ervaringen kunnen later in de helpinstellingen worden aangegeven; er wordt geen migratiegeschiedenis afgeleid.
4. **Korte taakhulp:** bied openen/zoeken, vensters wisselen, materiaal bewaren/vinden en instellingen/rechten als afzonderlijke, overslaanbare oefeningen aan. Gebruik herkenbaar synthetisch oefenmateriaal. Wis of overschrijf geen echt document om een les te demonstreren.
5. **Migratie apart:** apps/apparaten controleren en gegevens importeren is een eigen, hervatbare route. Toon bron, bestemming, proefresultaat en beperkingen. Vraag bronrechten pas op het moment van import; de bron blijft behouden. Een herkomstantwoord is geen toestemming voor schijftoegang.
6. **Optionele diensten later:** telefoon, NAS, cloud en lokale AI pas aanbieden bij een relevante taak, met bestaande privacy-/capabilitygates. Geen stap nodig om de lokale basis te gebruiken.

[VOORSTEL] Een ondertekend/geauthenticeerd installatie-overdrachtsrecord bevat alleen versienummer, afgeronde noodzakelijke stappen, gekozen taal/invoer/toegankelijkheid en verwijzingen naar de werkelijk gemaakte account-/opslagconfiguratie. Het bevat geen wachtwoord, ontgrendelsleutel, herstelcode of provider-token. Een renderer mag afgeronde beveiligingsstappen niet zelf fabriceren. Eerste boot valideert de echte toestand; bij onvolledigheid toont hij de concrete herstelroute en schrijft hij niet stilzwijgend opnieuw naar de schijf.

Regio, encryptie, privacy en wachtwoord worden niet opnieuw gevraagd omdat een rondleiding start. Het toekomstige state-model onderscheidt `niet aangeboden`, `uitgesteld`, `bezig`, `overgeslagen` en `afgerond`; overslaan is een volwaardige eindtoestand. “Later” blijft via Help/Instellingen bereikbaar, zonder meldingen die de gebruiker blijven onderbreken.

## Zoekcompatibiliteit zonder nieuwe naamsystemen

[VOORSTEL] Een lokale, versiegebonden synoniemenregistratie vertaalt bekende termen naar bestaande nieuw-os-doelen. Zoeken toont de eigen naam, een korte omschrijving, beschikbaarheid en eventueel “Ook bekend als …”. Het gebruikt geen internet of taalmodel. Alle ondersteunde termen zijn voor iedereen beschikbaar; herkomst beïnvloedt hoogstens de eerste uitleg, niet de vindbaarheid of toegangsrechten.

Dit is een registercontract, geen bestaande API. Per item zijn nodig: intentie-ID, eigen label, taal, aliassen, type (`app`, `bestand`, `instelling`, `actie`, `hulp`), doel-ID, beschikbaarheid, mogelijke dubbelzinnigheid en vereiste vervolgstap. IDs verlenen geen rechten.

| Bekende zoekterm | Bedoeld concept / voorgesteld nieuw-os-resultaat | Vereiste begrenzing |
|---|---|---|
| Verkenner, Explorer, File Explorer, Finder, Bestanden, Files, Nautilus, Dolphin | Materiaal — bestanden en documenten vinden | Huidige Materiaal-route is beperkt; geen algemene schijf-/netwerkbeheerclaim. Toon alleen werkelijk beschikbare bronnen en operaties |
| Task Manager, Taakbeheer, Activity Monitor, Activiteitenweergave, System Monitor, Systeemmonitor | Toekomstig systeemoverzicht | Nu niet als werkende app tonen. Bij KDE kan “Task Manager” ook het taakbalkwidget bedoelen: toon betekenissen zonder automatisch een proces te beëindigen |
| Settings, Instellingen, System Preferences, Systeemvoorkeuren, System Settings, Systeeminstellingen | Toekomstige uniforme instellingen | Alleen bestaande categorieën openen; ontbrekende instelling expliciet melden |
| Spotlight, KRunner, Start, application launcher | Zoeken / apps openen | “Start” kan ook een actie zijn. Toon context en categorie; geen visuele kopie nodig |
| Wifi, Wi-Fi, WLAN | Netwerk — draadloze verbinding | Pas aanbieden wanneer echte netwerkservice en beheer bestaan; wachtwoord niet in zoekindex of querygeschiedenis |
| Sound, geluid, audio output, geluidsuitvoer | Geluid — uitvoerapparaat | Geen verbinding wisselen zonder zichtbare keuze; geselecteerd apparaat en effect tonen |
| Display, beeldscherm, resolution, resolutie | Beeldscherm | Alleen ondersteunde standen; toekomstige bevestiging/terugval bij onleesbaar beeld vereist |
| Do Not Disturb, Niet storen, Focus | Meldingen en focus | Gewone meldingen dempen is iets anders dan systeemfouten of opname-/privacy-indicatoren verbergen |
| Windows Update, Software Update, Discover, Software | Apps/updates als onderscheiden resultaten | Store, pakketinstallatie en systeemupdate zijn verschillende intenties; geen pakketbron of update automatisch kiezen |
| Quick Look, preview, voorbeeld | Materiaal — voorbeeld waar beschikbaar | Geen ondersteuning van onbekende formaten beloven; niet uitvoerbare inhoud openen als app |

Vertaalvarianten en hoofdletters worden expliciet genormaliseerd. Diakritische varianten mogen vindbaarheid vergroten; zij mogen niet twee beveiligingsgevoelige doelen samenvoegen. Substrings, spelfouten en ambigue termen krijgen een lijst met tekstlabels. Een query is altijd inert: geen shellinterpretatie, geen URL-openactie, geen rechtenaanvraag of wijziging tijdens typen. Acties tonen gevolgen en vereisen een eigen activering; systeemprivileges blijven bij de broker/service.

Beschikbaarheid wordt bepaald door de echte service-/capabilitytoestand. Een gepland onderdeel verschijnt hoogstens als duidelijk gemarkeerde hulp met “nog niet beschikbaar”, niet tussen uitvoerbare resultaten alsof het werkt. De huidige vaste Materiaal-zoektermen zijn geen implementatie van dit OS-brede register. [Repositorybasis](../research/2026-11-02/repository-baseline.md)

## Hulp per herkomst, hetzelfde gedrag

| Context | Tijdelijke uitleg | Gelijk blijvend |
|---|---|---|
| Windows | Waar apps/vensters staan, hoe wisselen en naast elkaar werken, waar bestanden worden bewaard | Dezelfde launcher, vensterknoppen, sneltoetslabels, rechten en opslagregels |
| macOS | Verschil tussen app- en vensterstatus, locatie van acties, daadwerkelijke Command/Ctrl-mapping op dit toetsenbord | Geen globale menubalk simuleren als die er niet is; geen ongeverifieerde gesturebelofte |
| GNOME | Overzicht/zoeken, vensterbediening en werkruimtegedrag indien geleverd | Geen extensies of dynamische workspaces impliciet overnemen |
| KDE/Ubuntu | Eigen instellingen-/softwarebroncontract, synoniemen en paneelstatus | Geen pakketcompatibiliteit of onbeperkte panelconfiguratie claimen |
| Geen voorkeur | Tekstlabels, selecteren/openen, teruggaan en veilige oefenactie | Volledige toegang tot dezelfde bediening, zonder lager privilege of kinderachtige toon |

De hints zijn [HYPOTHESE]; behoud of verwijder ze op basis van taakobservatie. Een gebruiker kan hints per onderwerp sluiten. Herhaal geen gesloten hint door een reboot of software-update, tenzij een relevante bediening werkelijk verandert en de wijziging wordt uitgelegd.

## Privacy, toegankelijkheid en prestaties

Bewaar de optionele ervaring en afgeronde lessen lokaal per werkelijk ondersteund profiel. Bied bekijken, wijzigen en wissen. Geen serverprofiel, telemetrie, sync of toestemming afleiden uit de keuze. Zonder echte profielisolatie geen meergebruikersprivacy claimen. Het bestaande [lokale privacycontract](../LOCAL-PRIVACY-CONTRACT.md) blijft leidend voor netwerk, NAS en AI.

Elke oefening werkt met toetsenbord en benoemde controles; focus keert na sluiten naar de startactie terug. Geen timer, verplichte sleepbeweging of uitsluitend kleur/animatie. Toon de echte, herconfigureerbare sneltoets indien geleverd. Hulp mag tekstschaal, contrast of reduced motion niet resetten. De lokale synoniemenregistratie heeft een begrensde index en maakt geen netwerkverzoek; responstijd en geheugengebruik krijgen pas budgetten na een echte baseline.

## Betrokken onderdelen en acceptatie vóór uitvoering

Toekomstige componenten: sessiestatus, eerste-bootcoördinator, begeleidingspaneel, lokale voorkeurstore, zoekregister en help-/migratieroute. Huidige relevante plekken zijn `docs/implementation/PRODUCT-SPEC.md`, `ui/prototype/` als expliciete simulatie en de afzonderlijke native shell-/servicecode op de ontwikkelbron. Er worden in deze fase geen bestanden in die productpaden aangepast. Een concrete native bestandsindeling vereist een nieuwe actuele broninspectie; deze specificatie introduceert geen schijnbaar bestaande service.

| ID | Acceptatievoorstel | Verificatie |
|---|---|---|
| ON-01 | Eerste gebruik, overslaan en hervatten werken zonder herkomstantwoord | State-/routeproef inclusief reboot en onvolledige overdracht; geen dubbele beveiligingsvraag |
| ON-02 | Identieke kernbediening, zoekdoelen en privileges bij alle herkomstkeuzes | Vergelijk routes en capability-aanvragen; wijziging van herkomst verandert geen grants |
| ON-03 | Iedere vastgelegde alias vindt het juiste beschikbare doel of een eerlijke ontbrekend-/keuzestatus | Registertests per taal; ambiguïteit, ontbrekende service, collisions, typfouten en inert typen |
| ON-04 | Begeleiding wist zichzelf op verzoek zonder gegevens of instellingen van andere taken te veranderen | Profiel-/retentie- en wisproef met synthetische gegevens |
| ON-05 | Geen blokkade voor toetsenbord, hulpmiddel, schaal of reduced motion | Handmatige focus-/screenreader-/touchproef op werkelijk ondersteunde omgeving; ontbrekend bewijs melden |
| ON-06 | Hints verminderen verkeerde verwachtingen zonder nieuwe basisstappen toe te voegen | Verkennende taakproef per herkomst; vergelijk hulp, verkeerd doel en taakvoltooiing met een route zonder hints |

Risico's: verkeerde categorisering, ongewenste profilering, verouderde aliassen, dubbelzinnige platformtermen en een helptekst die een ontbrekende systeemfunctie verbergt. Voorkeursuitvoering: eerst een begrensde lokale register-/hulpproef, daarna echte native koppeling zodra de genoemde services en toegankelijkheid bestaan. Niet beginnen met een automatische desktopmodus of een model dat acties vrij interpreteert.

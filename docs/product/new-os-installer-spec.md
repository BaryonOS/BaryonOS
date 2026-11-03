# Nieuw-os — volledige installer vanuit herkenning

Datum: 2 november 2026. **Status: voorstel na vergelijkend onderzoek; geen installer gebouwd of vrijgegeven.** De actuele repository heeft eigen UEFI/QEMU-boot en een begrensde native sessie, geen bewezen fysieke installatie, persoonlijke authenticatie of volledige account-/encryptie-/herstelketen. [Repositorybasis](../research/2026-11-02/repository-baseline.md)

## Gebruikersprobleem en keuze

[BRON] De [31-aspectvergelijking](../research/os-installation-comparison.md) laat gedeelde concepten maar verschillende faseringen zien: Setup/OOBE, Recovery/Setup Assistant en distributiegebonden installers/eerste-startwizards. Sommige geavanceerde opslagroutes schrijven vóór een latere eindbevestiging. Een gekopieerde schermvolgorde is daarom geen veilige of bewezen universele funnel.

[ONTWERPKEUZE] Gebruik één herkenbare route: eerst kunnen waarnemen en invoeren, dan veilig proberen en geschiktheid begrijpen, daarna doel en bescherming kiezen, en pas na een compleet wijzigingsplan schrijven. First boot verifieert de feitelijke uitkomst. Optionele begeleiding en diensten komen nadat de noodzakelijke basis bruikbaar is. Dit is een te toetsen ontwerpkeuze, geen uit de bronnen bewezen optimale volgorde.

De installer bedient ervaren computergebruikers zonder verplichte handleiding en beginners zonder verplichte kennis van EFI, GPT, mount points of filesystems. Advanced geeft technische controle, met dezelfde veiligheidsgrenzen. De eigen kernel blijft de basis; Calamares/Anaconda zijn onderzoeksmateriaal, geen aangenomen platformarchitectuur.

## Voorgestelde funnel

```text
Gecontroleerd opstartmedium
  → Welkom: taal + altijd bereikbare toegankelijkheid
  → Toetsenbord/invoer proberen
  → Proberen / Installeren / Herstel (alleen beschikbare routes)
  → Geschiktheid en benodigde verbinding
  → Installatiemethode + geïdentificeerde schijf
  → Voorstel gegevens-/bootwijzigingen
  → Lokaal account + bescherming en herstelmiddel
  → Lokale voorkeuren + privacycontrole
  → Volledig wijzigingsplan + expliciete schrijfbevestiging
  → Installeren met echte fase-/foutstatus
  → Herstart + juiste mediuminstructie
  → Eerste boot: feitelijke configuratie en toegang controleren
  → Desktop + optionele, hervatbare onboarding/migratie
```

Netwerk is geen aparte verplichte poort als alle geauthenticeerde kernonderdelen op het medium staan. Download, tijdsynchronisatie of latere providerkoppeling tonen hun eigen reden en netwerkgrens. Een onvolledig medium kan niet via een nep-offlinepad verder. De gebruiker kan terug naar proberen of stoppen zonder schijfmutatie.

## Stapcontract

Iedere rij is [VOORSTEL]. Verwijzingen `Ixx` zijn de dimensies van de bronmatrix, niet schermnummers van andere systemen. De vergelijkingsdossiers geven de precieze bronnen en uitzonderingen.

| Stap | Waarom / herkenbare vergelijking | Informatie voor beginner | Advanced en veilige default | Combineren, uitstellen of weglaten |
|---|---|---|---|---|
| S00 Boot | Bron en doel scheiden; alle onderzochte routes hebben boot/recovery (I01) | Naam/versie medium, proefstatus en beperkte hardwareondersteuning | Alleen gecontroleerde payload/bootketen; geen automatische reparatie van hostopslag | Geen firmwareconfiguratie verbergen in “Volgende” |
| S01 Welkom | Taal vóór belangrijke tekst; Ubuntu zet toegankelijkheid vroeg (I03/06) | Taal, groter/contrasterend beeld, spraak/invoer waar echt geleverd | Geen netwerkdetectie van nationaliteit; behoud gekozen hulpmiddelen | Toegankelijkheid permanent bereikbaar, geen verplichte vragenlijst |
| S02 Invoer | Keyboard vóór geheimen; Windows/Ubuntu hebben vroege keuze (I05) | Voorbeeld typen, layout en zichtbaar testresultaat zonder opslag | Andere layouts/IME alleen indien werkelijk ondersteund; geen wachtwoord als testtekst | Later wijzigbaar; niet opnieuw vragen in first boot |
| S03 Intentie | Linux live-Try is een herkenbaar veilig experiment (I08) | “Proberen verandert de interne schijf niet”; installatie en herstel apart | Proefsessie krijgt geen opslagmutatierecht; herstel alleen als echt geleverd | Mag bij welkom aangeboden worden, maar gevoelige stappen pas na bruikbare invoer |
| S04 Gereedheid | Eisen/hardware zijn verschillend bij alle routes (I09/23) | Werkt beeld/invoer/opslag; bekende beperkingen en ruimte/energie | Exact getest profiel; waarschuwing ≠ garantie. Onveilig/ongekend schrijfpunt blokkeert installatie | Vóór schijfkeuze tonen; nog geen drivers/firmware stil installeren |
| S05 Verbinding indien nodig | Ubuntu optioneel; Windows/macOS hebben routeafhankelijke afhankelijkheid (I07/22) | Waarom verbinding nodig is, welke download en alternatief | Offline complete kern; geverifieerde downloads, geen externe fallback of captive-portalgeheimen loggen | Overslaan als niet nodig; normale wifiwisseling later in instellingen |
| S06 Methode/doel | Alle routes onderscheiden doel en behoud/vervangen (I10–13) | “Bestaand systeem behouden”, “Deze schijf vervangen” of bewezen herstelroute; schijfnaam/capaciteit/identiteit | Geen te wissen doel vooraf geselecteerd; onbekende layout is geen vrije ruimte | Methode en schijf samen tonen; technische editor achter Advanced |
| S07 Wijzigingsvoorstel | Automatische en handmatige layouts bestaan naast elkaar (I11/25) | Voor/na: welke gegevens/systemen blijven, verdwijnen of verplaatsen; bootgevolgen | Interne layout berekenen na expliciete keuze; geen schrijfacties tijdens preview | Normale gebruiker hoeft geen partitietype te kiezen; Advanced behoudt hetzelfde planmodel |
| S08 Persoonlijk account | Account is gedeeld concept, positie verschilt (I15/18) | Naam en echte aanmeldmethode op dit apparaat; provider is afzonderlijk | Lokale account; geen ongeauthenticeerde guest als persoonlijk. Privileges niet afleiden uit een webformulier | Apparaatnaam niet-identificerend voorstellen, bewerkbaar in detail; geen verplichte onlineaccount |
| S09 Bescherming/herstel | Encryptie- en herstelmodellen verschillen sterk (I14/17) | Wat beschermd is, hoe ontgrendelen werkt en wat verlies van herstelmiddel betekent | Persoonlijke gegevens versleuteld als productdoel; alleen aanbieden bij echte crypto/sleutel-/authketen; anders blokkade | Met account combineren als uitleg helder blijft; PIN, loginwachtwoord en encryptiesleutel niet verwarren |
| S10 Lokale voorkeuren | Tijdzone/regio niet overal dezelfde stap (I04) | Tijdzone en eventueel regio voor weergave; geen locatie nodig | Bewerkbaar voorstel zonder stil GeoIP; eigen apparaatnaam zichtbaar | Taal/keyboard niet dupliceren; tijdzone kan met voorkeuren worden gecombineerd |
| S11 Privacycontrole | OOBE/first-startkeuzes lopen uiteen (I19–21/24) | Lokale opslag, geen OS-telemetrie, optionele latere koppelingen; security floor blijft aan | Geen providerescrow, geen cloud/AI nodig; geen privacykeuze die isolatie uitzet | Korte controle, geen verzameling extra dienstenvragen; apps/AI/telefoon later |
| S12 Samenvatting/autoriseer | Duidelijke schrijfgrens ontbreekt niet overal, maar is niet uniform (I25) | Concreet doel, behouden/gewist, bootwijzigingen, bescherming/herstel en netwerkdownload | Geen autorisatie bij onzeker/stale plan; expliciete actie met gevolg in knoptekst | Eén volledige grens vóór iedere mutatie, ook vanuit Advanced |
| S13 Installeren | Alle routes tonen uitvoering en overgang (I26) | Werkelijke fase, fouten, annuleer-/herstelgrens; geen valse ETA | Installerbroker voert uitsluitend geautoriseerd plan uit; geheimen niet loggen | Geen aanbiedingen, quiz of nieuwe accountkeuze tijdens kopiëren |
| S14 Herstart/medium | Routes verschillen van automatische reboot tot live-exit (I27/28) | Gereed, welk medium verwijderen, waar volgende boot vandaan komt | Geen reboot als onafgeronde writes; fout toont herstelpad | Geen ingebouwde-recovery-USB-instructie tonen als er geen USB is |
| S15 Eerste boot | Nieuwe configuratiefase komt breed voor (I29/30) | Feitelijke accounttoegang en installatiestatus; noodzakelijke foutafhandeling | Toestand controleren, niet blind wizardvlag geloven; geen herformat bij mislukking | Geen herhaalde taal/privacyvragen; optionele begeleiding mag naar desktop worden overgeslagen |
| S16 Desktop | Succes betekent bruikbare geïnstalleerde sessie (I31) | Waar apps, zoeken, materiaal, instellingen en hulp staan, voor zover geleverd | Eerlijke status van proefsessie/persoonlijk/beheerd; geen nepnetwerk/apps | [Onboarding](new-os-onboarding-spec.md) vrijwillig; import afzonderlijk en bronbehoudend |

## Schijfveiligheid en herstelbaarheid

**Geen mutatie vóór S12.** Inventarisatie is read-only en mag geen filesystemreparatie, automatische mount met writes, journal replay op bronvolumes, resize, bootentrywijziging, encryptieconversie of “voorbereidend” formatteren uitvoeren. Ook Advanced mag geen direct schrijvende editor om de bevestigingsgrens heen aanbieden. Deze strengere grens is een eigen veiligheidsbesluit op basis van de gevonden uitzonderingen, geen claim dat alle bestaande installers zo werken.

Het wijzigingsplan bevat minimaal: stabiele device-identiteit plus menselijk herkenbare naam/capaciteit, bron-/doelrol, alle te behouden/wissen/verkleinen delen, bestaande OS-/gegevensdetectie met onzekerheidslabel, bootwijzigingen, encryptie-/herstelstatus, benodigde ruimte en eventuele download. Onbekende of versleutelde gegevens worden expliciet onbekend genoemd. Het plan noemt exacte gevolgen; een bevestigingscheckbox is geen vervanging voor begrijpelijke informatie.

Elke wijziging aan doelidentiteit, media, plan, versleutelstatus of voorwaarden maakt de autorisatie ongeldig. De broker valideert identiteit en geautoriseerd plan opnieuw vlak vóór de eerste write en bij relevante fasen. Een hotplug, corrupte planning of verkeerde doelhandle geeft een blokkade, geen automatische vervangende doelschijf. Een UI-renderer bezit geen algemeen schijfrecht en kan geen consentbewijs fabriceren. Het toekomstige broker-/capabilitycontract vereist afzonderlijk ontwerp en negatieve tests.

**Dual boot is conditioneel:** alleen aanbieden voor precies geteste boot-, ruimte-, bestandssysteem-, resize- en encryptiecombinaties met herstelroute. Anders uitleggen dat behoud op deze schijf niet ondersteund is; een expliciet gekozen andere schijf of stoppen blijft mogelijk. De installer mag niet stil de enige aangeboden wisroute kiezen. Een back-up is vooraf nodig bij wijzigingen aan bestaande gegevens; een gedetecteerde back-up is geen bewijs dat herstel werkt.

**Annuleren vóór schrijven:** nul schijfwijzigingen en tijdelijke geheimen wissen. **Na schrijven:** geen beloofde undo; stop alleen op veilige fasegrenzen en rapporteer werkelijk voltooide/onvoltooide stappen en herstelopties. Bronverlies na expliciet wissen is niet door een “Terug”-knop herstelbaar. Onderbrekingstest en herstelmedium zijn releasegates. Downloadfouten laten een geldige bron/installatie intact waar technisch bewezen; geen nieuwe schijf initialiseren om een fout te maskeren.

## Encryptie, accounts en persoonsgegevens

Persoonlijke gegevensbescherming sluit aan op [het lokale privacycontract](../LOCAL-PRIVACY-CONTRACT.md), niet op een nieuwe zelfbedachte cryptolaag. Onderhoudbare crypto, entropy, sleutelisolatie, authenticatie, opslagintegriteit, update-/pakketvertrouwen en herstel moeten daadwerkelijk bestaan. Een ontbrekende keten blokkeert de claim “veilig persoonlijk geïnstalleerd”. Geen dummycheckbox, plaintext-terugval of automatische provider-key-escrow.

De normale route maakt onderscheid tussen login, ontgrendelen en herstel. Biometrie/TPM/PIN alleen bij bewezen ondersteunde hardware en veilige fallback; geen fysieke aanwezigheid of biometrie suggereren op basis van een VM. Herstelmiddelen worden buiten het te wissen of ermee beschermde volume bewaard, met uitleg en een veilige controle van beschikbaarheid; zij worden niet naar een provider verstuurd, in telemetrie gezet of in logs/screenshotbewijs opgenomen. Geheimen uit inputbuffers/voortgangstoestand verwijderen volgens een apart te testen geheugen-/servicecontract.

## State, navigatie, techniek en uitvoeringsplan

Voorgesteld toestandsmodel: read-only inventarisatie → conceptplan → compleet gevalideerd plan → expliciet geautoriseerd plan → uitvoering → verifieerbare afronding, fout of herstel. Teruggaan vóór uitvoering verwijdert afhankelijke autorisaties; het bewaren van taal of layout bewaart geen schijfconsent. Na schrijven vervangen veilige fase-/herstelacties het gewone wizard-Terug.

Toekomstige componenten: boot-/payloadverifier, readinessprobe, read-only opslaginventaris, planbuilder, account-/sleutelservice, samenvattingsrenderer, begrensde installerbroker, voortgangs-/recoverycoördinator en first-bootvalidator. Dit zijn benodigde onderdelen, **geen bestaande services**. De exacte native bestanden en IPC-contracten worden pas vastgesteld na actuele code-inspectie en securityontwerp. Geen brede herschrijving, bestaand bewijs of QEMU-opslagmodel wijzigen om deze documentatie als werkend te laten lijken.

Voorkeursvolgorde voor een later implementatieplan:

1. Pure, niet-schrijvende plan-/stateproef met synthetische schijven; ongeldige en ambigue doelen centraal. Een UI-prototype blijft als simulatie gelabeld.
2. Echte opslaginventaris en capabilitygrens met wegwerpimages; geen onbekende-volume-initialisatie. Eerst negatieve tests en onderbrekingen.
3. Echte account-/crypto-/herstelketen en toegankelijke invoer vanaf boot. Niet alvast persoonlijke privacy claimen.
4. Geautoriseerde uitvoering en first-bootherstel op het vaste virtuele profiel; vervolgens volledige regressiegates en menselijke taakproef.
5. Alleen na afzonderlijk hardwarebewijs en expliciete opdracht ooit fysieke installatie. Deze onderzoeksopdracht verleent die toestemming niet.

Prestatie-impact: inventarisatie/download mag invoer en hulpmiddelen niet blokkeren; annuleerbare probes, begrensde geheugen-/loggroei en geen animatie afhankelijk van elke disk-read. Meet respons en fasekosten op het echte profiel; er is geen onderbouwd tijdsbudget of installatieduur vastgesteld.

## Acceptatie en regressierisico

| ID | Vereist vóór ondersteunde uitvoering | Controle |
|---|---|---|
| IN-01 | Geen mutatie vóór complete planbevestiging, ook in Advanced | Broker-/state-negatieftests, read-only bronbewaking, write-audit op disposable beelden |
| IN-02 | Juiste doelidentiteit; verandering invalideert bevestiging; onbekend ≠ leeg | Meerdere gelijke schijven, encrypted/unknown/full, hotplug, stale/corrupt plan, ongeautoriseerde handle |
| IN-03 | Beginners begrijpen behoud/wissen zonder technische layouttermen | Taakproef met synthetische gegevens: vooraf gevolgen laten uitleggen; ernstig misbegrip blokkeert release |
| IN-04 | Keyboard, taal, hulpmiddelen en focus werken vóór geheimen en schijfkeuze | Keyboard-/screenreader-/contrast-/schaal-/reduced-motionproef vanaf boot, niet alleen op desktop |
| IN-05 | Werkelijke voortgang, veilige annuleergrens, herstel na onderbreking en correcte eerste boot | Foutinspuiting per fase, mislukte payload/update/boot, herstelmiddelproef; geen nep-success |
| IN-06 | Lokaal account en bescherming zijn echt; optionele providers wijzigen geen security floor | Auth-, isolatie-, sleutel-, grant- en netwerknegatieftests; menselijke beveiligingsreview blijft apart |
| IN-07 | Bronbehoud en boot-/dual-bootclaim kloppen per aangeboden route | Voor/na-inventaris, toepasselijke hashes/metadata, boot/herstel van ieder behouden systeem |

Risico's: stale device-identiteit, schijfinventaris die al schrijft, verkeerd opgeslagen herstelmiddel, verschil tussen UI-consent en brokerrechten, misleidende first-bootstatus, ontbrekende drivers/hulpmiddelen en regressie van bestaande virtuele opslagrecovery. Bouw-, lint-/type-/testcommando's volgen de actuele betrokken code; ze zijn hier niet uitgevoerd omdat uitsluitend documentatie wordt toegevoegd. Veiligheidsbewijs, bruikbaarheid en fysieke ondersteuning blijven afzonderlijke acceptatieniveaus.

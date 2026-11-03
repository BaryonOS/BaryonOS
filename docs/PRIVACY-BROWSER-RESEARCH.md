# Privacybrowser — onderzoeksrichting en open keuzes

**Status:** onderzoeksnotitie, bijgewerkt 3 november 2026. WPE WebKit 2.54 met
WPEPlatform is gekozen als eerste upstream-portkandidaat, niet als definitieve
productkeuze. Er is **nog geen browserengine naar Nieuw OS geport**; de keuze
blijft afhankelijk van een haalbare build-, proces- en surfaceproef. Chromium/
Blink en Firefox/Gecko zijn kort vergeleken als alternatieven.

## 1. Productdoel

Nieuw OS heeft voor dagelijks gebruik een browser nodig die gewone websites,
webapps, videobellen, betalingen en andere gangbare webtaken kan uitvoeren,
zonder van gedragsprofilering of advertentietelemetrie een verdien- of
systeemmodel te maken.

Het gewenste standaardgedrag is:

- de browsermaker ontvangt geen surfgeschiedenis, advertentieprofiel of
  gebruikstelemetrie die niet strikt nodig is voor een expliciet gekozen
  diagnostische handeling;
- bekende cross-site trackers en trackingopslag worden standaard beperkt;
- fingerprinting wordt waar praktisch mogelijk verminderd zonder de gebruiker
  door exotische afwijkingen juist uniek te maken;
- gevoelige browserrechten zoals camera, microfoon, locatie, klembord en
  bestanden blijven gekoppeld aan een concrete gebruikershandeling;
- een website die door privacybescherming niet goed werkt krijgt een
  begrijpelijke herstelroute, zonder dat de browser automatisch alle
  bescherming uitschakelt;
- beveiligingsupdates en noodzakelijke achtergrondverbindingen zijn
  inventariseerbaar en uitlegbaar.

Dit sluit aan op het bestaande privacycontract: internetrecht geeft een browser
geen algemene document- of NAS-bevoegdheid. De browser blijft een gewone,
begrensde app boven de capability- en netwerkgrenzen van Nieuw OS.
Daarbij blijven drie autoriteiten gescheiden: browser-apprechten, rechten van
een weborigin en door de gebruiker verleende toegang tot een concreet middel.
Een originverzoek loopt via browserbeleid en een vertrouwde broker; alleen die
broker kan een scoped runtime-capability aanvragen. Enginepolicy zelf geeft
geen kernelrecht uit.

## 2. Twee afzonderlijke privacyproblemen

### 2.1 Wat de browser zelf verstuurt

Een privacybrowser moet eerst zijn eigen netwerkgedrag kunnen verantwoorden.
Onderzoek minimaal:

- telemetrie, experimenten, crashrapportage en unieke installatie-identifiers;
- zoeksuggesties en tekst die vóór Enter naar een zoekprovider kan gaan;
- update-, certificaat-, filterlijst-, tijd- en connectiviteitsverkeer;
- account-, synchronisatie- en aanbevelingsdiensten;
- eventuele Safe Browsing-/malwarechecks en welke metadata daarbij wordt
  gedeeld;
- extensie- en componentupdates.

Doel: een schone installatie heeft een gedocumenteerde lijst van noodzakelijke
achtergrondverbindingen. Optionele diagnostiek is uitgeschakeld tenzij de
gebruiker die bewust activeert.

### 2.2 Wat websites en derden kunnen verzamelen

De browser moet daarnaast bescherming bieden tegen tracking door bezochte
websites en ingesloten derden. Kandidaten voor het standaardprofiel:

- blokkeren van bekende cross-site trackers en advertentietrackers;
- partitioneren of blokkeren van third-party opslag;
- beperken van trackingparameters in URL's waar dat geen navigatie breekt;
- bescherming tegen bekende fingerprintingvectoren;
- beperken van speculatieve/preconnect-verzoeken wanneer die privacykosten
  hebben;
- duidelijke sitegegevens-/cookiecontrole;
- geen stilzwijgende toegang tot camera, microfoon, locatie of lokale
  documenten.

Geen van deze maatregelen kan anonimiteit garanderen. Een bezochte website ziet
bij een rechtstreekse verbinding onder meer het bron-IP en kan ingelogde
activiteit aan een account koppelen. Fingerprinting kan worden verminderd maar
niet universeel onmogelijk worden gemaakt.

## 3. Tijdelijke uitzonderingen als productfunctie

De voorgestelde gebruikersroute is niet “zet privacy uit”, maar **“deze website
werkt niet”**. De browser probeert vervolgens de kleinst mogelijke uitzondering.

### Voorgesteld gedrag

1. Bescherming staat standaard aan.
2. Bij sitebreuk toont de browser welke categorie waarschijnlijk betrokken is,
   bijvoorbeeld geblokkeerde third-party opslag of een tracker/script.
3. De eerste herstelactie versoepelt slechts die categorie voor de huidige site.
4. De standaardkeuze is **eenmalig** of **tot deze site/tab wordt gesloten**.
5. Een blijvende uitzondering is een aparte, expliciete keuze.
6. Actieve uitzonderingen zijn zichtbaar vanuit de adresbalk en in één handeling
   terug te draaien.
7. Een uitzondering voor site A geldt niet automatisch voor dezelfde derde partij
   op site B.
8. De browser kan een kapotte site rapporteren met technische diagnostiek nadat
   de gebruiker de inhoud daarvan heeft kunnen beoordelen.

### Niet versoepelen via deze route

Een privacy-uitzondering mag niet automatisch de volgende beveiligingsgrenzen
uitschakelen:

- TLS-/certificaatvalidatie;
- processandboxing en site-/rendererisolatie;
- geheugen- en capabilitygrenzen van Nieuw OS;
- bescherming tegen downloads of uitvoerbare code;
- de scheiding tussen browserinternet en persoonlijke documenten/NAS;
- de regel dat een website nooit zelfstandig extra browser- of OS-rechten mag
  verlenen.

Een toestemming kan gegevens vanaf dat moment naar de betreffende website of
derde laten gaan. Het opnieuw inschakelen van bescherming kan reeds verzonden
gegevens niet terughalen.

## 4. Relevante opensourcevoorbeelden

| Project | Basis | Bruikbaar als referentie | Aandachtspunt |
|---|---|---|---|
| Chromium | Blink/Chromium | Zeer brede webcompatibiliteit, multiprocessarchitectuur en sandboxontwerp | De sandbox leunt op beveiligingsmechanismen van het onderliggende OS; een Nieuw-OS-port vereist dus een eigen sandboxintegratie |
| ungoogled-chromium | Chromium | Verwijdert Google-webserviceafhankelijkheden en veel achtergrondverzoeken | Aanvullende privacytweaks staan grotendeels niet automatisch aan; geen complete productstrategie op zichzelf |
| Brave | Chromium | Ingebouwde trackerblokkering, cookie/storagebescherming, fingerprintingmaatregelen en site-specifieke Shields | Eigen productkeuzes/telemetrie moeten afzonderlijk worden beoordeeld; niet blind als basis overnemen |
| Vanadium | Chromium | Voorbeeld van browserhardening die nauw samenwerkt met OS-beveiliging | Sterk gekoppeld aan GrapheneOS/Android; niet rechtstreeks een Nieuw-OS-port |
| Firefox | Gecko | Enhanced Tracking Protection, Total Cookie Protection en site-specifieke uitzonderingen | Standaard Firefox heeft eigen product-/telemetriekeuzes die voor Nieuw OS opnieuw moeten worden beoordeeld |
| LibreWolf | Firefox | Telemetrie uit, uBlock Origin, strengere tracking- en fingerprintinginstellingen | Strenge defaults kunnen sitecompatibiliteit en bepaalde diensten beïnvloeden |
| Mullvad Browser | Firefox/Tor Browser | Compile-time uitschakelen van telemetrie en sterke anti-fingerprinting; publiceert achtergrondverbindingen | Uniforme fingerprintingstrategie kan UX-/compatibiliteitskosten hebben |
| Tor Browser | Firefox ESR | Sterke anti-fingerprinting en netwerkidentiteitsisolatie | Tor-routing/anonimiteit is een ander productdoel dan een algemene dagelijkse Nieuw-OS-browser |

Deze tabel kiest geen winnaar. Projecten kunnen ideeën leveren zonder dat Nieuw
OS hun branding, complete patchset of productmodel overneemt.

## 5. Waarom Chromium technisch aantrekkelijk kan zijn

Chromium is een serieuze kandidaat wanneer maximale websitecompatibiliteit en
ondersteuning van hedendaagse webapps zwaar wegen. De browser kan als
opensourcebasis worden aangepast; ungoogled-chromium laat zien dat
Google-webserviceafhankelijkheden en achtergrondverzoeken uit Chromium kunnen
worden verwijderd.

Een Chromiumkeuze betekent echter niet alleen “de UI porten”. Chromium beschrijft
zelf dat zijn sandbox gebruikmaakt van beveiligingsmechanismen van het
onderliggende besturingssysteem. Voor Nieuw OS moeten daarom minimaal processen,
IPC, shared memory, netwerk, bestandstoegang, GPU/rendering, fonts, invoer,
tijd/entropy en de sandbox op de eigen kernel- en capabilitygrenzen worden
aangesloten.

De port moet bovendien upstream beveiligingsupdates snel kunnen blijven volgen.
Een zeer grote permanente patchset vergroot het risico dat patches vertraagd of
moeilijk te integreren worden.

## 6. Waarom Firefox/Gecko eveneens onderzocht moet worden

Firefox is technisch relevant omdat Mozilla al trackingpartitionering,
site-specifieke bescherming en webcompatibiliteitsmechanismen heeft. LibreWolf
en Mullvad Browser demonstreren daarnaast dat telemetrie en aanvullende
privacyfuncties verder kunnen worden aangescherpt.

Een Gecko-keuze lost de port naar de eigen kernel evenmin vanzelf op. Ook hier
zijn grafische, netwerk-, proces-, filesystem-, sandbox- en updateinterfaces
nodig. De vergelijking moet daarom niet “welke browser vind ik mooier?” zijn,
maar welke codebasis de gewenste productgaranties onderhoudbaar kan dragen.

## 7. Onderzoekscriteria vóór een enginebesluit

Maak vóór een ADR met een enginekeuze ten minste een reproduceerbare vergelijking
van Chromium- en Gecko-gebaseerde kandidaten.

| ID | Vraag | Minimale toets |
|---|---|---|
| BR-01 | Kan alle niet-noodzakelijke producttelemetrie worden verwijderd? | Netwerkcapture van schone start, idle, zoeken, browsen en afsluiten; iedere achtergrondverbinding verklaard |
| BR-02 | Hoe snel zijn upstream securityfixes over te nemen? | Meet patchdelta en bouwproces over meerdere upstream securityupdates |
| BR-03 | Hoe vaak breekt het privacyprofiel websites? | Vast testpakket met login, betalingen, streaming, videovergaderen, webapps en embeds |
| BR-04 | Werken gerichte uitzonderingen? | Site-specifieke tijdelijke uitzondering herstelt functie zonder globale privacyverlaging |
| BR-05 | Welke fingerprintinginformatie blijft zichtbaar? | Herhaalbare testset met gedocumenteerde APIs en configuraties; geen absolute anonimiteitsclaim |
| BR-06 | Kan de sandbox op Nieuw OS worden afgedwongen? | Threat model en capabilitymapping voor renderer, netwerk, GPU, downloads en bestandportalen |
| BR-07 | Is de dependency- en licentieketen beheersbaar? | Inventaris van bron, licenties, third-party code, buildtools en update-eigenaarschap |
| BR-08 | Blijft de ervaring consumentenwaardig? | Taakproef met standaardprofiel en herstel na sitebreuk zonder technische instellingenkennis |

## 8. Open productkeuzes

Nog niet beslissen zonder proef:

- Chromium/Blink versus Firefox/Gecko als eerste engine.
- Blokkeren versus partitioneren van verschillende third-party opslagvormen.
- Anti-fingerprinting via uniformering, randomisering of een combinatie.
- Welke phishing-/malwarebescherming lokaal of privacybehoudend kan blijven
  functioneren.
- DRM/Encrypted Media Extensions: privacy, vrijheid en consumentencompatibiliteit
  moeten expliciet tegen elkaar worden afgewogen.
- Extensie-ecosysteem en welke extensie-API veilig ondersteund wordt.
- Zoekprovider, DoH/DNS-gedrag en optionele synchronisatie.
- Of browserwachtwoorden in de browser zelf horen of via een afzonderlijke
  lokale credentialsdienst van Nieuw OS.
- Welke diagnostiek een gebruiker bij “website werkt niet” lokaal kan bekijken
  en vrijwillig kan delen.

## 9. Voorgestelde ontwikkelvolgorde

**BR0 — vergelijking en privacycontract.** Voer BR-01 t/m BR-08 eerst uit op
hostbuilds. Dit levert een enginebesluit en patchbudget op, geen native
browserclaim.

**BR1 — onderhoudbare browservariant op de ontwikkelhost.** Bouw het gekozen
privacyprofiel, tijdelijke site-uitzonderingen en geautomatiseerde
netwerk-/compatibiliteitstests. Houd de patchset klein en herbaseerbaar.

**BR2 — Nieuw-OS platformlaag.** Port de minimaal benodigde platforminterfaces
naar de eigen netwerk-, graphics-, input-, storage- en procesdiensten. Dit is
afhankelijk van voldoende volwassen native netwerk- en grafische infrastructuur.

**BR3 — sandbox en capability-integratie.** Renderers, netwerkdienst, GPU-/media-
processen, downloads en bestandsportalen krijgen uitsluitend de noodzakelijke
Nieuw-OS-capabilities. Test negatieve toegang en intrekking end-to-end.

**BR4 — dagelijkse gebruiksacceptatie.** Test echte websites en taken,
beveiligingsupdates, toegankelijkheid, prestatie, sitebreuk en herstel op de
ondersteunde hardware. Pas dan kan browserfunctionaliteit bijdragen aan A04 en
relevante privacy-/netwerkcriteria van de vaste roadmap.

### BR1 — eerste upstream hostslice, 3 november 2026

De geïsoleerde crate `experiments/browser-parser-host` bouwt upstream
`scraper 0.29.0` op een vaste UTF-8-HTML-fixture: invoerbytes → HTML5-DOM →
CSS-selectorquery → begrensde titel-/linkinventaris. Dit is een parser-/DOM-
componentproef op de host, niet een browserenginekeuze of Nieuw-OS-port.
Dependencyherkomst, licenties, buildscripts en exacte tests staan in
[`browser-parser-host-verificatie`](verification/browser-parser-host/README.md)
en [`de derdenregistratie`](../experiments/browser-parser-host/THIRD-PARTY.md).

| Browserfunctie/platformgrens | Benodigde Nieuw-OS-service of autoriteit | Deze slice |
|---|---|---|
| Documentinvoer | Een expliciete bron, later via een begrensde document-/downloadpoort of netwerkresponse; geen algemene bestandsmachtiging | Caller geeft maximaal 64 KiB fixturebytes door; geen bestand- of netwerk-I/O |
| HTML5-DOM en selectorquery | Browserprocesgeheugen en parser | Hostcrate gebruikt upstream `scraper`; `no_std`-doel faalt |
| CSS-styling, layout, fonts en paint | Browserstijl-/layoutketen, fontservice en een geïsoleerd compositor-/renderendpoint | Niet aanwezig |
| Navigatie, DNS, TLS en verbindingen | Netwerkdienst met app-/bestemmingsrechten; geverifieerde peerfacts en onderhouden TLS | Niet aanwezig; `href` wordt niet opgelost of opgehaald |
| Downloads en lokale documenten | Afzonderlijk download-/documentportaal met concrete objectrechten | Niet aanwezig |
| Rendererprocessen, IPC en shared memory | Eigen kernel-launch, procesisolatie, begrensde IPC en capability-uitgifte/intrekking | Niet aanwezig |
| Input, clipboard, camera/microfoon en locatie | Native input-/toestemmingsportalen met handeling- en oorsprongsgebonden scope | Niet aanwezig |
| Browser-/engine-updates en certificaat-/tijdvertrouwen | Geauthenticeerde updateketen en expliciete tijd-/trustbron | Niet aanwezig |

De tabel is een product-/servicegrensinventaris, geen bewezen engine-ABI- of
capabilitymapping. De precieze interfaces hangen af van een latere enginekeuze.
De huidige parserlimieten begrenzen de caller-invoer en uitvoervorm, maar geen
interne heap-, rekentijd- of parserdiepteconsumptie van upstream.

De upstream `scraper`-release heeft ISC-licentie en recente repositoryactiviteit;
de exacte packageversie/checksum en dependencyboom zijn vastgezet. `scraper`,
`html5ever` en `tendril` gebruiken `std`; de check voor
`x86_64-unknown-none` faalde. Daarmee is deze component alleen geschikt voor
de hostfixtureproef. Er wordt geen CSS-layout, renderer of JavaScript-engine
op basis van deze proef gebouwd.

### M5 — korte upstream-routevergelijking

| Route | Concrete portinghaak en resterend platformwerk | Eerste keuze |
|---|---|---|
| WPE WebKit 2.54 | `WPEPlatform` is de standaardintegratielaag; een port implementeert display, toplevel, view en buffer en voert input aan. Upstream levert Wayland, DRM/KMS en headless; builds en runtimes zijn beschikbaar via `meta-webkit` voor OpenEmbedded/Yocto. De API is GObject-gebaseerd. Zij levert geen Nieuw-OS-procesmodel, sandbox, scheduler, geheugenservice of capabilitybroker. | **Ja, kandidaat.** Meest begrensde expliciete surface/input-adapter voor een eerste echte engineproef. |
| Chromium/Blink | Ozone biedt concrete platforminterfaces voor vensters, input, display en surfaces; de Chromium-buildinstructies beschrijven een omvangrijke Linux-hostbuild. Ozone lost procesisolatie, sandbox, geheugen-/threadintegratie en netwerkautoriteit niet op. | Niet eerst: bredere build- en OS-integratie voor de eerste slice. |
| Firefox/Gecko | Een volledige onderhouden engine met OS-widgetabstracties, netwerkstack, IPC en meerdere processen. De korte officiële-docscan vond geen smallere embedded platforminterface vergelijkbaar met WPEPlatform; een port raakt meerdere OS-lagen. | Niet eerst: grotere integratiescope voor het huidige platformcontract. |

WPEPlatform concretiseert graphics/buffers en input, niet alle platformdiensten.
Processtart en isolatie, threads/tijd/geheugen, fonttoegang, netwerk, downloads,
clipboard, audio/media en origin-toestemming hebben dus nog een Nieuw-OS-contract
en upstream-configuratie nodig. Vooral audio/media, fontafhandeling en de
volledige dependency-/licentie-inventaris zijn niet in een buildproef
geverifieerd. WPE/WebKit gebruikt een mix van LGPL- en BSD-licenties; de exacte
component- en codecselectie moet vóór distributie worden vastgelegd.

**Besluit en blocker:** ADR-065 kiest WPE WebKit 2.54/WPEPlatform als eerste
route, niet als engineportbewijs. De huidige host heeft geen `pkg-config`
commando; een WPE/WebKit-build en de beschikbaarheid van de benodigde
GObject-/graphics-/media-dependencies zijn daarom hier niet vastgesteld.
Nieuw-OS heeft bovendien nog geen algemene engine-launch/sandbox- en
surface-adapter aangetoond. Er is bewust geen lege adapter of eigen renderer
toegevoegd. De eerste vervolgpoort is een reproduceerbare upstream-build plus
een gekoppelde `WPEDisplay`/`WPEView`/`WPEBuffer`-surface en lokale fixture; pas
daarna kan de echte layout-/renderpipeline worden geëvalueerd.

**Primaire bronnen gecontroleerd op 3 november 2026:**

- rust-scraper, *scraper 0.29.0*, crate-/releasemetadata en dependencymanifest:
  [crates.io](https://crates.io/crates/scraper/0.29.0) en
  [manifest op VCS-revisie `149ae5c9`](https://github.com/rust-scraper/scraper/blob/149ae5c9dfbaca2e6b92470018fedc95ab254d5d/scraper/Cargo.toml).
- Servo, *html5ever 0.40.1*, upstream HTML-parserdocumentatie en manifest:
  [crates.io](https://crates.io/crates/html5ever/0.40.1) en
  [Servo/html5ever](https://github.com/servo/html5ever).
- Chromium Project, *Linux build instructions*, actuele upstream
  buildvereisten:
  [chromium.googlesource.com](https://chromium.googlesource.com/chromium/src/+/main/docs/linux/build_instructions.md).
- WPE WebKit, *WPE Architecture* en *Builds*: WPEPlatform-objecten, ingebouwde
  targets en OpenEmbedded/Yocto-routes:
  [architecture](https://wpewebkit.org/about/architecture.html),
  [builds](https://wpewebkit.org/about/builds.html).
- WPE WebKit, *Frequently Asked Questions*: WPE 2.54-route en gemengde
  LGPLv2/BSD-licenties:
  [FAQ](https://wpewebkit.org/about/faq.html); WebKit Project,
  [licenties](https://webkit.org/licensing-webkit/).
- Chromium Project, *Ozone Overview*: platforminterfaces voor graphics/input:
  [chromium.googlesource.com](https://chromium.googlesource.com/chromium/src/+/db6267fecee29ec703b51f1e1b854cdf56c53a3c/docs/ozone_overview.md).
- Mozilla, *Gecko*: engineonderdelen, OS-widgets en procesarchitectuur:
  [Firefox Source Docs](https://firefox-source-docs.mozilla.org/overview/gecko.html).

## 10. Bewijsgrens en voortgang

Deze notitie is onderzoeks- en productdocumentatie. Zij implementeert geen
browser, netwerkstack of sandbox en sluit daarom geen bestaand
acceptatiecriterium. De **historische meting op 28 oktober 2026** was
**14% → 14%**; de actuele assessment staat in
[`PROJECT-STATE.md`](PROJECT-STATE.md) en `docs/progress.json`.

## Bronnen

Brave Software, Inc. (z.d.). *Brave Shields — Blocking ads, trackers & more*.
https://brave.com/shields/ (geraadpleegd 28 oktober 2026).

Chromium Authors. (z.d.). *Sandbox*. Chromium source documentation.
https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md
(geraadpleegd 28 oktober 2026).

GrapheneOS. (z.d.). *Features overview: Vanadium*. https://grapheneos.org/features
(geraadpleegd 28 oktober 2026).

LibreWolf. (z.d.). *Features*. https://librewolf.net/docs/features/
(geraadpleegd 28 oktober 2026).

Mozilla. (2026, 24 september). *Enhanced Tracking Protection in Firefox for
desktop*. https://support.mozilla.org/en-US/kb/enhanced-tracking-protection-firefox-desktop

Mozilla. (2026, 24 juni). *Total Cookie Protection and website breakage FAQ*.
https://support.mozilla.org/en-US/kb/total-cookie-protection-and-website-breakage-faq

Mullvad VPN AB. (2026, 13 januari). *The Mullvad Browser hard facts: list of
settings and modifications*. https://mullvad.net/en/browser/hard-facts

ungoogled-software. (z.d.). *ungoogled-chromium*. GitHub.
https://github.com/ungoogled-software/ungoogled-chromium

World Wide Web Consortium. (2025, 25 september). *Mitigating Browser
Fingerprinting in Web Specifications* (W3C Group Note).
https://www.w3.org/TR/fingerprinting-guidance/

De voorgestelde uitzonderingsflow, BR-acceptatievragen en ontwikkelvolgorde zijn
projectontwerp op basis van de gebruikersopdracht en bovenstaande technische
referenties; de bronnen certificeren Nieuw OS niet.

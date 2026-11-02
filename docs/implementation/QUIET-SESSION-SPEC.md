# Quiet futurism 2.0 — sessie en werkruimte

**Datum:** 2 november 2026. **Bronbasis:**
`1f0dd9c93959e3b1335656da1e548525a288133e`. **Status:** ontwerp vóór visuele
integratie, geen nieuw test-/QEMU-bewijs. Score **41/100 blijft 41**.
Het [routeonderzoek](../research/SESSION-ROUTES-2026-11-02.md) onderbouwt
herkenbare conventies. Deze specificatie vult [PRODUCT-SPEC](PRODUCT-SPEC.md),
[het sessiecontract](SESSION-AND-DESKTOP.md) en
[de native venstersessie](NATIVE-WINDOW-SESSION.md) aan. Bestaande opslag-,
privacy- en capabilitygrenzen blijven leidend; geen nieuwe roadmap of gate.

## Productcorrectie

De negatieve referentie toont volgens de gebruikersbrief kale vlakken, weinig
hiërarchie, verwante blauwgroentinten, technische statusregels en een
weinig informatief dock. Het probleem is ontbrekende sessie-/vensterstructuur;
glas, gradients, rondingen of motion lossen dat niet op. Dit is de aangeleverde
kritiek, geen nieuwe inspectie van een niet meegeleverd screenshotbestand.

De actuele bron heeft al proportionele antialiased systeemletters, eigen
pictogrammen, twee appprocessen, focus/slepen/herstel en aparte aanmeldpreview.
Deze functionaliteit blijft. Nieuwe actuele screenshots bepalen welke
tekortkomingen nog bestaan; eerdere screenshotkritiek is geen codebewijs.

De compositie krijgt drie lagen: permanente systeemstatus; werkruimte met
Materiaal en gereedschapvensters; tijdelijke zoek-/systeemlagen met juist
focusherstel. Materiaal blijft object met revisies, verwisselbare gereedschappen
en intrekbare toegang. Een app starten geeft geen algemene datatoegang.
Rechten, geschiedenis en locatie staan bij het object, niet als beheerconsole
op het hele bureaublad.

## Visuele regels en tokens

Waarden zijn **ontwerpdoelen**, geen reeds gemeten contrast-/rendererresultaten.
Native gedeelde kleuren komen nu uit `ui/graphics/src/theme.rs`; compacte
geometrie-/appkleuren deels uit `ui/prototype/tokens.css`. De builder wijzigt
de daadwerkelijke bronnen samenhangend; geen derde palet in een renderer.

| Rol | Licht | Donker | Functie |
|---|---|---|---|
| Canvas | `#F2F1EC` | `#151C24` | rustige werkachtergrond |
| Verhoogd oppervlak | `#FFFFFF` | `#202A35` | menu/zoeken/dialoog |
| Actief venster | `#FFFFFF` | `#263442` | actieve titel/controls |
| Inactief venster | `#E6E8E7` | `#1D2732` | leesbaar, lagere nadruk |
| Primaire tekst | `#20262D` | `#F2F4F5` | inhoud/labels |
| Secundaire tekst | `#5C6570` | `#B8C1CB` | metadata/locatie |
| Accent | `#14655E` | `#79C6BB` | spaarzame primaire actie |
| Selectie | `#DCEDE8` | `#254840` | tevens vorm/tekst |
| Focus | `#275DAD` | `#A8C8FF` | ononderbroken rand |
| Succes | `#236A47` | `#8AD4A8` | tevens tekst/pictogram |
| Waarschuwing | `#88520A` | `#F0C475` | gevolg/herstel |
| Gevaar | `#A13B32` | `#FFB1A6` | weigering/onomkeerbaar |
| Offline | `#5C6570` | `#B8C1CB` | “Offline · lokaal beschikbaar” |
| Vertrouwde systeemlaag | `#15232F` | `#101922` | compositorheader/badge |
| Modale achtergrond | `#15232F` op 32% | `#000000` op 44% | dempen en input blokkeren |
| Desktop-ink | `#18242F` | `#111A23` | donkere buitenshell |

Donkere tokens definiëren rollen; een complete dark-modeport is pas klaar na
review van alle toestanden. Kleur geeft geen autoriteit: een app kan donker
tekenen; alleen compositor, inputfocus en capabilities beschermen trusted UI.

- Basiseenheid 4 logische pixels; afstanden 8/12/16/24/32/48.
- Controls minimaal 44 logisch hoog. Iconen op herkenbaar 24-eenhedenraster,
  met label als betekenis anders niet vanzelfsprekend is.
- Radius 8/12/16 voor control/venster/dialoog; geen pillvorm of kaart om alles.
- Vensterrand 1 logisch; focusrand 3. Subtiele schaduw ondersteunt stapelvolgorde,
  maar vervangt geen titel of actieve indicator.
- Bodydoel 16/24, metadata 14/20, sectie 20/28, titel 32/40; locktijd 88/96
  bij voldoende ruimte. Bodygewicht 400, titel 600. Geen algemene uppercase of
  monospace. Werkelijke fontmetrics bepalen uitlijning, niet tekenaantallen.
- Native hergebruikt de gepinde Open Sans-atlas. Nieuwe optische maten vereisen
  echte atlas-/schaaltests; vergrote pixelletters zijn geen high-DPI-acceptatie.
- Bewerkbare documenttekst behoudt tijdelijk monospace met juiste caretmetrics;
  proportionele editing vereist ook aangepaste selectie/wrapping/caret en
  verandert nooit opgeslagen UTF-8 om de vormgeving passend te maken.
- Motion verklaart oorzaak/plaats, maximaal 120–160 ms als doel. Reduced motion
  wisselt direct; geen animatie/sleep stelt readiness of input kunstmatig uit.

## Board met acht samenhangende toestanden

Het board is een **ontwerpcontract**: maak vóór definitieve integratie één
inspecteerbare compositie van onderstaande toestanden. Boardrender is geen
runtimebeeld of native bewijs. Leg daarna dezelfde toestanden met echte
interacties vast en beoordeel het geheel op samenhang.

| Toestand | Compositie | Interactie en bewijsgrens |
|---|---|---|
| 1. Vergrendeld | duidelijke tijd/datum, apparaat, beperkte energie/netwerkstatus; geen privéwerk | “Aanmelden”, Enter, zichtbare focus; klok uit adapter; vaste fixturetijd alleen gelabeld in board/tests |
| 2. Gebruiker kiezen | compacte verticale gebruikersrijen, avatar/initialen, naam, laatst gebruikt zonder werkdetails | Tab/pijlen/Enter; geen appkaartgrid; beheerderstatus alleen wanneer nodig |
| 3. Authenticeren | geselecteerde naam, methode, credential-/hardware-instructie, busy/fout/herstel | echte brokergrens, zichtbaar testadapterlabel; na succes eerst ontsluiten dan juiste sessie |
| 4. Lege werkruimte | vaste status/tijd/sessiemenu, benoemde launcher en startactie Materiaal | geen nepbestanden of decoratieve serviceclaims; offline en onbeschikbare functies eerlijk |
| 5. Twee vensters | Materiaal met gevulde lijst/revisie/locatie; Rekenmachine met echte berekening/historie | actieve/inactieve titel, focus, sluiten/verbergen/herstellen; inhoud blijft bij compositorherstart |
| 6. Zoeken | begrensd veld; groepen apps/Materiaal/instellingen/acties; labels en sneltoets | type/Enter/Escape; resultaat geeft geen ontbrekende capability; leeg/fout inbegrepen |
| 7. Snelle instellingen | aan statusgebied verankerd; apparaat/netwerk/energie/schaal, ingang instellingen | alleen echte ondersteuning actief; geen nep-batterijpercentage of schijnnetwerk |
| 8. Capabilitydialoog | systeemheader, programma/object/handeling/bestemming/duur/gevolg/intrekken | bij risico standaard annuleren; achtergrondinput geblokkeerd; compositor-gehoste variant apart native bewijzen |

Board: 1440×900. Runtimebeelden: primaire native **1280×720**, 1440×900,
1024×768 en 1920×1080, laatste tevens op 200% waar ondersteund. Een browsermaat
bewijst geen gastresolutie; niet uitgevoerde native maten blijven open.

Layout gebruikt logische pixels: 1920×1080/200% is 960×540 logisch. Authpaneel
maximaal 480 breed; zoeklaag maximaal 720 met 24 marge. Kleinere schermen
krijgen compactere compositie of scrollbaar inhoudsdeel, geen primaire actie
buiten beeld. Opzettelijke vensteroverlap blijft herkenbaar; clipping van
controls/systeemmenu is afkeur. Outputverlies houdt systeemcontrols binnen
veilige bounds; meerdere outputs zijn voorlopig alleen toekomstcontract.

## Sessieroute en adaptergrens

Bestaand `SessionFlow` bewaart correlatie en wist owned credentialbuffers;
`AuthBroker` scheidt authenticatie/activatie. `UnavailableBroker` blijft veilig
default zolang de echte dienst ontbreekt. `VmGuest` is een openbare synthetische
VM-werkruimte, geen juiste-password-succes, ontsloten gebruikerssleutel of
beveiligde compositor-lock. Testadapter alleen in expliciet testprofiel.

```text
Boot → Preparing → Locked → UserSelection → MethodSelection
→ Authenticating → Unlocking → Desktop

Desktop → Locking → Locked
Desktop → LogoutPending → UserSelection
Desktop → Switching → UserSelection
```

`Preparing` gebruikt echte readiness/herstel, geen nepservices of vaste sleeps.
`Unlocking` volgt geldig authresultaat; desktopinput begint pas na succesvolle
sessieactivatie. Bij fout blijft gevoelige appinput geblokkeerd.

De bestaande enums worden gericht uitgebreid achter dezelfde brokergrens.
Onderstaande namen beschrijven het **doelcontract**, geen bestaand wire-ABI:

```text
AuthMethod = Password | LocalPin | Fingerprint | Fido2SecurityKey | RecoveryMethod
AuthResult = Authenticated(session_assertion) | Rejected(reason)
           | TemporarilyLocked(retry_policy) | Cancelled | HardwareUnavailable
           | HardwareRemoved | Timeout | ServiceFailure
```

`session_assertion` is een service-issued eenmalig bewijs of opaque handle,
gebonden aan gebruiker, boot, service-/sessiegeneratie, doel, verse challenge en
verval. Correlatievelden/publieke Rust-constructors bewijzen geen issuer. De
sessiedienst controleert issuer, intrekking, freshness en scope opnieuw bij
activatie. Geen eigen cryptografie. Een echte dienst heeft persistente
verifiers/enrollment en monotone limitering nodig; UI-countdown is geen limiter.

Methode-/gebruikerswisseling, cancel, lock, logout en relevante servicefout
retireren de poging. Late replies en hergebruikte assertions falen gesloten.
Fouttekst onthult geen credentialdetails. Hardwaretekst mag “Sleutel verwijderd”
tonen zonder geregistreerde credentialmetadata. Geen wachtwoorden, private keys,
biometrische templates of hashes van credentialinvoer in logs/screenshots/
fixtures. Tests registreren alleen synthetische publieke toestanden.

## Desktop en toegangsbediening

Permanente laag: launcher/zoeken, open apps, klok/status, meldingencentrum,
snelle instellingen en sessiemenu. Lock, wisselen, afmelden, afsluiten en
herstarten zijn benoemde acties. Onbeschikbare powerdienst rapporteert geen
“afgesloten”. Open werk volgt bewaren/verwerpen/annuleren en echte commit/
flush volgens bestaande opslagcontracten.

Compositor bezit systeemranden, titels, z-order, bounds, focus, minimalisatie en
inputrouting. Apps tekenen alleen eigen surface-inhoud. Verplaatsen/schalen
bewaart bounds/dirty-regio's. Alt+Tab wisselt vensters; controls zijn ook met
toetsenbord bereikbaar. Sluiten met draft volgt dirty-check; Escape wist niets.
Herstart retireert oude input/presentatie-epochs; apps bewaren inhoud en
accepteren alleen opnieuw geauthenticeerde bindings.

Zoeken toont uitsluitend geautoriseerde providers; geen namen uit een andere
gebruiker. Bestaande grants worden hergebruikt; installatie verleent geen data-
rechten. Capabilitydialoog noemt app, object/apparaat, handeling, bestemming,
duur, gevolg en intrekking. Bij risicovolle lees-plus-internetcombinatie start
focus op annuleren. Intrekken verandert broker-/kernelbeleid, niet alleen UI.

| Mechanisme / dreiging | Invariant, trusted component en afdwinging | Fout/herstel en open bewijs |
|---|---|---|
| Authvervalsing/replay | auth-/sessiedienst valideert gebruiker/boot/doel/challenge/expiry; UI geeft geen rechten | geen key release/desktop; poging retired, herauthenticatie; echte dienst/keys open |
| App imiteert systeemdialoog | compositor bezit trusted surface, niet-overschrijfbare systeemidentiteit en exclusieve modale input; kernel bindt echte peer | appclaim/stale action geweigerd, geldige focusreturn; browsermodal bewijst native grens niet |
| Input naar andere sessie | inputdienst/compositor controleren sessie/owner/epoch/bounds; apps hebben geen InputSource/DisplaySource | drop/deny, verse rebind na herstel; nieuwe lockgrens vereist native tests |
| Rechtenverruiming | capabilitybroker/kernel begrenzen object/actie/bestemming/duur | weigering zichtbaar, draft behouden; nieuwe prompt alleen bij nieuwe scope/autoriteit; gaten blijven benoemd |

## Toegankelijkheid en visuele acceptatie

Tab/Shift+Tab volgen leesvolgorde; Enter activeert bedoeld control; Escape
sluit uitsluitend bovenste tijdelijke laag of annuleert poging. Focus blijft
zichtbaar en binnen modaliteit; terug naar trigger of veilige opvolger.
Icon-only controls hebben labels; kleur is niet de enige statusdrager.

Browser: landmarks, echte buttons/fields, dialognamen en begrensde live
announcements. Native bouwt voort op bestaande semantiekboom met node-ID,
parent/role/name/bounds/focus/enabled/actions. Dat is een toekomstige brokerhook,
geen OS-brede schermlezer, IME, bidi of volledige shaping. WCAG voor browser en
native ontwerp is geen conformiteitsclaim: W3C, 12 december 2024,
[WCAG 2.2](https://www.w3.org/TR/WCAG22/).

Ieder runtimebeeld krijgt broncommit/bewijsniveau/resolutie/schaal/inputroute/
artifacthash. Review vóór/na op hiërarchie, alignment, leesbare tekst, actieve
vensters, focus, contrast, clipping, objectstatus, informatiedichtheid en
ongewenste technische hoofdtekst. Beoordeel expliciet herkenbare tijd/status/
zoeken/vensters; een groene pixeltest of losse authpagina volstaat niet.
AI-beeldreview is geen menselijke gebruikersproef.

## Uitvoering en acceptatie

| Groep | Verplichte proeven | Bewijs |
|---|---|---|
| Sessie | verkeerd wachtwoord/gebruiker, succes vóór activatie, replay/stale/wrong boot/session, rate limit, cancel, key removed, timeout, servicecrash, geen fail-open | negatieve hosttests eerst; daarna echte diensten native |
| Shell | volledige route; zoeken vier doeltypen; twee vensters; status/sessiemenu | browser/native apart, adapter gelabeld |
| Compositor | ontbrekende surfacecap, trusted imitatie, achtergrondinput bij modaliteit, verkeerde sessie/epoch, focusreturn, crash/rebind | echte QEMU-tests; hostmodel voorbereidend |
| Toegankelijk | Tab/Shift+Tab/Enter/Escape, trap/return, 100/200%, reduced motion, labels/rollen, draft intact | browserchecks/native hooks afzonderlijk |
| Visueel | acht boardstates/runtimebeelden, geen clipping/miniatuurcontrols/technische hoofdlaag | expliciete visuele review naast regressietests |
| Snelheid | boot→eerste frame, boot→lock/input-ready, authsucces→desktop/input-ready, full/partial render, dirty pixels/IPC; capture/QMP apart | voor/na dezelfde configuratie, echte run/logbinding |

Ontworpen, hostgetest, browserprototype, native QEMU en fysieke hardware blijven
apart. Geen cijfers zonder run, geen U02-/FR-S02-punten voor board/adapter,
geen nieuwe claim voor al geaccepteerd U01. Integrator onderhoudt gedeelde status,
score en architectuurbesluiten; deze specificatie is geen tweede roadmap.

Open: **P0** coherente lock/login/desktop met echte adapters; **P1** compositor-
lock/trusted modality/session-input/sizing; **P2** dagelijkse instellingen,
notificaties/power/herstel met persistentie; **P3** echte FIDO2-/biometrische
devices, meerdere displays en fysieke hardware.

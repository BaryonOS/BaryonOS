# Ontwerpambitie: een eigen Europees OS met hoogwaardige productkwaliteit

## Leidende aanvulling — 27 oktober 2026

De gebruiker wil een privacyvriendelijk, hypermodern Europees alternatief dat de verbonden en verzorgde ervaring biedt die consumenten van Windows/macOS verwachten. Privacy mag niet leiden tot een omslachtig of functioneel verschraald product. Die vergelijking is een ambitie en ontwerpbrief, geen gemeten productclaim.

**Veiligheid, privacy en gebruiksgemak zijn gezamenlijke acceptatievoorwaarden.** Bouw verbinding, continuïteit en samenwerken met expliciete technische grenzen. Los technische problemen gericht op; maak tijdelijke prototypebeperkingen geen permanente verplichtingen voor de gebruiker. Normale bediening vraagt geen kennis van certificaten, nonces, tunnels of services.

| Gebruikerstaak | Gewenste normale ervaring | Veiligheidsgrens en toets |
|---|---|---|
| Beginnen | Lokaal starten zonder leveranciersaccount; internet en externe apps kunnen gebruiken | Installatie geeft geen algemene document-/NAS-rechten; toegestane functies werken zonder securityinstellingen te verzwakken |
| NAS koppelen | Eén begeleide flow voor eigen NAS of ontvangen ruimte; identiteit één keer bewust bevestigen | Discovery alleen is geen vertrouwen; herbevestigen bij nieuwe identiteit of ingetrokken autoriteit |
| Dagelijks synchroniseren | Automatisch op de achtergrond binnen de al gekozen scope; geen herhaalde toestemmingsvragen per bestand | Native encryptie, tenantisolatie, juiste revisions en geteste herstelpaden |
| Thuis/onderweg wisselen | Automatisch LAN prefereren en anders eerder toegestaan internet naar dezelfde NAS | Geen nieuwe consentprompt voor dezelfde autoriteit; andere peer/relay/identiteitsfout blijft geblokkeerd |
| Tweede eigen apparaat | Werk en geselecteerde context veilig hervatten na een begrijpelijke koppeling | Sleuteloverdracht tussen eigen geverifieerde apparaten; geen sleutel bij NAS-beheerder/leverancier |
| Ruimte delen | In dezelfde OS-ervaring capaciteit aanbieden of ontvangen, ook zonder eigen NAS-budget | Beheerder mag quota terugnemen/wissen zonder leesrecht; ontvanger kent beschikbaarheidsgrens |
| Fout of verlies | Lokale taken blijven bruikbaar, duidelijke status en een uitvoerbare herstelroute | Geen dataverlies verbergen, geen cloudfallback; verloren sleutels niet met een achterdeur oplossen |
| Ingrijpende actie | Eén duidelijke bevestiging van de bevoegde actor, op het relevante moment | Verwijdering alleen op betrokken hostkopie; geen toestemming vragen aan de gast om eigen ruimte terug te nemen |

Meet per taak voltooiing, aantal beslissingen/prompts, fout-/herstelgedrag en begrijpelijkheid met gebruikers die geen beheerders zijn. Bestaande toestemming mag dagelijkse onderbrekingen voorkomen zonder identiteit of bevoegdheid stilzwijgend uit te breiden. Visuele afwerking, responsiviteit, toetsenbordbediening en toegankelijkheid moeten in dezelfde gebruiksproef slagen.

Eenmalige koppel-/herstelstappen zijn toegestaan waar zij een reële vertrouwensgrens bevestigen. CLI-stappen, herhaald vergelijken van dezelfde identiteit, handmatige sync en telkens opnieuw toestemming geven zijn geen gewenste standaardervaring. Read-only keyrecovery en kleine capaciteiten in N01 zijn proefgrenzen; veilige persistente keys, rotatie en multi-devicegebruik zijn vereisten vóór een productclaim.

## Onderzoeksaanvulling — consumentenwaarde, 28 oktober 2026

De literatuurverkenning in [research/CONSUMER-VALUE-2026-10-28.md](research/CONSUMER-VALUE-2026-10-28.md) ondersteunt de bestaande ambitie, maar scherpt de productvraag aan: een zelfstandige kernel en sterke privacyarchitectuur zijn geen voldoende consumentenbelofte. De eerste aantoonbare waarde moet liggen in een herkenbare dagelijkse taak die merkbaar beter of duidelijker verloopt.

Te onderzoeken kernbelofte:

> **Je materiaal direct terugvinden, ermee verdergaan en controle houden over wijzigingen en toegang.**

Behandel **aandacht, proberen, overstappen en blijven gebruiken** als verschillende uitkomsten. Meet taakprestatie, begrip, herstel, hulpbehoefte en vrijwillig hergebruik afzonderlijk van visuele waardering. Behoud vertrouwde basisbediening, namen, mappen en export waar die overstapfrictie verminderen; originaliteit moet uit de samenhang van Materiaal, gereedschappen, geschiedenis, rechten en herstel komen, niet uit het onnodig opnieuw uitvinden van iedere conventie.

Dit is onderzoeksrichting, geen bewijs dat consumenten al voor Nieuw OS kiezen. De P2-productspecificatie blijft leidend voor de huidige bouwscope; nieuwe hypotheses worden eerst getest voordat zij vaste producteisen worden.

> Update revisie 5: de eerste bouwkeuzes staan vast in [implementation/PRODUCT-SPEC.md](implementation/PRODUCT-SPEC.md). Gebruik dat document voor directe implementatie; dit bestand bewaart de ambitie en achtergrond voor vervolgfases.
Revisie 3, 26 oktober 2026. Dit is een bindende productopdracht op basis van Simons aanvullende instructie. Het versmallen van de eerste technische proef mag de oorspronkelijke ambitie niet vervangen.

## Richting
Ontwerp een zelfstandig OS dat een eigen, samenhangende manier van werken biedt, visueel en in bediening het niveau van volwassen platforms nastreeft en gebruikers daadwerkelijk zeggenschap geeft. Een bekende desktop met een ander thema, launcher of logo is onvoldoende als eindresultaat. *Aanvulling 30 oktober 2026 (ADR-058):* de bediening zelf (inloggen, bureaublad, taakbalk, startmenu/zoeken, vensters, sneltoetsen) volgt bewust herkenbare conventies, zodat consumenten niets opnieuw hoeven te leren; de eigen code, huisstijl en vernieuwingen komen daarbovenop.

Europa is hier de gekozen productoriëntatie: privacy, zeggenschap over gegevens, toegankelijkheid, interoperabiliteit, herstelbaarheid en vrijheid van leverancier. Dit is geen bewering dat er één formele 'Europese OS-norm' bestaat of dat een nog ongebouwd systeem al wettelijk voldoet. Koppel concrete producteisen aan bewijs en beoordeel formele toepasselijkheid bij een concrete release/doelgroep.

## Ambitie eerst, onderzoek daarna
Neem een ongebruikelijk idee serieus, ook als het niet direct in een bestaande tutorial past. Behandel een technische beperking als onderzoeksvraag:
1. Welk gewenst gebruikersresultaat wordt geblokkeerd?
2. Is de blokkade een bewezen hardwaregrens, huidige implementatiebeperking, kostenafweging of onbewezen aanname?
3. Welke andere representatie, interface of architectuur kan het resultaat mogelijk maken?
4. Wat is het kleinste experiment dat de hypothese kan toetsen?
5. Welk bewijs bepaalt voortzetting, bijstelling of uitstel?

'Dat doen bestaande OS'en niet' en 'dat is geen Linux-patroon' zijn geen afwijzingsgronden. Ook 'AI kan alles bouwen' is geen bewijs. Bewaar de ambitie als requirement; maak de onzekerheid en experimenten zichtbaar.

## Drie parallelle sporen
**Productonderzoek begint bij M0.** Ontwerp en toets interacties terwijl de eigen kernel wordt ontwikkeld. Prototypes kunnen simuleren wat nog niet bestaat, mits duidelijk gelabeld. Een bruikbare simulatie is bewijs over een interactie, niet over kernelveiligheid of werkende hardware.

**Systeemonderzoek blijft kleine bewijsbare stappen maken.** De kern levert steeds meer functies die de gekozen productervaring werkelijk kunnen dragen. Een feedbacklus verbindt beide sporen: een productbesluit kan nieuwe IPC-, opslag- of autorisatierequirements opleveren; een technisch experiment kan alternatieven voor de bediening voorstellen.

Er is geen algemene regel 'geen UI-denken vóór geheugenisolatie'. Wel: een mooie interface mag ontbrekende isolatie nooit maskeren en geeft geen toestemming om een onveilig prototype als dagelijks OS uit te brengen.

**Quantumonderzoek begint eveneens bij het ontwerp.** Volg QUANTUM-RESEARCH.md voor postkwantumweerbaarheid, lokale quantumuitvoering en het eigen OS op een hybride quantumcomputer. Het product blijft zonder QPU bruikbaar; quantumcapaciteiten krijgen een eigen bewijs- en ontwikkelpad.

## Wat de eerste productonderzoeker oplevert
- Drie wezenlijk verschillende concepten voor interactie en informatieordening; geen drie visuele skins.
- Per concept vijf uitgeschreven gebruikersscenario's met schermtoestanden en gevolgen voor opslag/rechten.
- Een vergelijking met relevante bestaande platformen: wat is bekend, wat combineert dit anders, wat moet nog onderzocht worden? Geen onbewezen wereldprimeurclaim.
- Eén aanbevolen concept met keuzes en verworpen aannames, daarna een kleine interactieve proef.
- Een eerste design system: typografie, spacing, kleur, iconografie, focus, componentgedrag, beweging en taal.
- Een koppeling van iedere fundamentele interactie aan een benodigd systeemcontract.

Verken bijvoorbeeld verschillende antwoorden op: zijn taken, documenten/objecten of apparaten het primaire organisatieprincipe? Hoe hervat iemand werk op een tweede eigen apparaat via de eigen NAS? Hoe blijft een app verwisselbaar terwijl het document en de bevoegdheden onder gebruikerscontrole blijven? Deze vragen zijn onderzoek, geen opgelegde uiteindelijke interface.

## Hoogwaardig is toetsbaar
De ervaring moet coherent zijn van eerste installatie tot foutmelding, van appinstallatie tot NAS-herstel. Onderzoek:
- duidelijke visuele hiërarchie en verzorgde details, ook lege/laad-/fouttoestanden;
- consistente invoer, animaties, venster- of werkruimtegedrag en direct zichtbare feedback;
- toegankelijkheid via toetsenbord, schaal, contrast en hulptechnologie;
- begrijpelijke rechten zonder voortdurend onderbrekende meldingen;
- een SDK en componentbibliotheek waarmee derdenapps bij het systeem passen;
- betrouwbaar installeren, verwijderen, bijwerken en ongedaan maken;
- dezelfde productlogica op verschillende apparaatklassen met passende bediening.

Een ontwerp mag aan vertrouwde patronen herinneren waar die werken. Het moet een eigen antwoord geven op de kerntaken en eigenaarschap van gegevens; alleen uiterlijk verschil is niet genoeg.

## Aanvulling 28 oktober 2026 — privacy zonder visuele of functionele concessie

De uiteindelijke eigen apps en systeemoppervlakken moeten bewust **premium en
toekomstgericht** worden ontworpen. Privacyvriendelijkheid is geen reden voor
verouderde bediening, grove systeembeheerpanelen of inconsistente open-source-
achtige deelinterfaces. De referentiekwaliteit is het niveau van afwerking dat
gebruikers bij moderne commerciële productiviteitssoftware verwachten; het
product mag die software niet klonen.

Materiaal blijft het onderscheidende model: inhoud en objectcontinuïteit eerst,
contextuele gereedschappen daarna. Eén designsysteem moet typografie, spacing,
focus, motion, delen, herstel, geschiedenis, fouttoestanden en
toegankelijkheid over eigen apps heen dragen. Het langetermijndoel is een
gedeelde native render-/UI-laag zodra de grafische basis dit technisch draagt;
het huidige browserprototype is geen stil gekozen eindruntime.

Een gebruiker mag aanvullende privacyuitleg uitschakelen. De daadwerkelijke
Security Floor en capabilitygrenzen blijven onafhankelijk daarvan actief. Zie
[implementation/PRODUCT-SPEC.md](implementation/PRODUCT-SPEC.md) en
[implementation/ENCRYPTED-OBJECT-STORAGE.md](implementation/ENCRYPTED-OBJECT-STORAGE.md).

## Niet-onderhandelbare grenzen
Eigen OS-kernel. Geen stille overgang naar een Linux-distributie of webshell op een bestaande kernel. OS-data blijft lokaal of wordt client-side/end-to-end versleuteld op een expliciet gekoppelde eigen NAS, NAS van vrienden/familie of — na expliciete keuze — als opaque ciphertext op een externe opslagprovider gezet. Een externe provider krijgt geen Nieuw-OS-inhoudssleutel en is geen stille fallback. De NAS-beheerder beheert capaciteit en verwijdering, de gebruiker zijn inhoudssleutel. AI-inferentie op apparaat, zonder cloudfallback. Externe apps met beperkte rechten. Privacykeuzes mogen de Security Floor niet uitschakelen. Volg LOCAL-PRIVACY-CONTRACT.md.

Streef naar zeer sterke bescherming, met heldere garanties per functie en testbaar herstel. Gebruik een absolute claim als 'niet te hacken' niet als vervanging voor ontwerp of bewijs. Maak veiligheid een kwaliteit van de ervaring, geen excuus voor een onaf product.

## Eerste acceptatiepoort
De startfase is productmatig pas inhoudelijk uitgewerkt als drie verschillende concepten en één beargumenteerde richting bestaan. Een kernel die tekst logt is een technische stap; die bepaalt nog niet hoe het uiteindelijke OS eruitziet of werkt. De hoofdagent bewaakt beide resultaten afzonderlijk.


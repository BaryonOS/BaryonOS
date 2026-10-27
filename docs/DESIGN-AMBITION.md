# Ontwerpambitie: een eigen Europees OS met hoogwaardige productkwaliteit

> Update revisie 5: de eerste bouwkeuzes staan vast in [implementation/PRODUCT-SPEC.md](implementation/PRODUCT-SPEC.md). Gebruik dat document voor directe implementatie; dit bestand bewaart de ambitie en achtergrond voor vervolgfases.
Revisie 3, 26 oktober 2026. Dit is een bindende productopdracht op basis van Simons aanvullende instructie. Het versmallen van de eerste technische proef mag de oorspronkelijke ambitie niet vervangen.

## Richting
Ontwerp een zelfstandig OS dat een eigen, samenhangende manier van werken biedt, visueel en in bediening het niveau van volwassen platforms nastreeft en gebruikers daadwerkelijk zeggenschap geeft. Een bekende desktop met een ander thema, launcher of logo is onvoldoende als eindresultaat.

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

## Niet-onderhandelbare grenzen
Eigen OS-kernel. Geen stille overgang naar een Linux-distributie of webshell op een bestaande kernel. OS-data op apparaat/eigen NAS op eigen LAN; AI-inferentie op apparaat, zonder cloudfallback. Externe apps met beperkte rechten. Volg LOCAL-PRIVACY-CONTRACT.md.

Streef naar zeer sterke bescherming, met heldere garanties per functie en testbaar herstel. Gebruik een absolute claim als 'niet te hacken' niet als vervanging voor ontwerp of bewijs. Maak veiligheid een kwaliteit van de ervaring, geen excuus voor een onaf product.

## Eerste acceptatiepoort
De startfase is productmatig pas inhoudelijk uitgewerkt als drie verschillende concepten en één beargumenteerde richting bestaan. Een kernel die tekst logt is een technische stap; die bepaalt nog niet hoe het uiteindelijke OS eruitziet of werkt. De hoofdagent bewaakt beide resultaten afzonderlijk.

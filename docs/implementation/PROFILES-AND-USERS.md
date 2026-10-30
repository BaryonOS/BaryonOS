# Profielen en meerdere gebruikers — stappenplan

Datum: 30 oktober 2026. Status: **voorstel**, opgesteld op verzoek van Simon
na een handmatige proef met de Windows-QEMU-testversie. Er is voor dit plan
nog geen code geschreven en niets native getest. Het gebruikersmodel in §3
vraagt eerst een besluit van Simon (stap 1); pas daarna hoort er een ADR in
[DECISIONS.md](../DECISIONS.md). Bronbasis: main
`c0b7147b`. Voortgangsvoorstel: **34% → 34%**; dit document sluit geen
criterium.

## 1. Aanleiding en gewenst resultaat

De huidige native testversie toont één app (Materiaal): een document maken,
typen, opslaan en heropenen. Er is geen eerste-startinrichting, geen profiel,
geen inlog- of vergrendelscherm, geen instellingenscherm en geen tweede
persoon op dezelfde pc. In main staat geen code voor onboarding, profielen of
gebruikerssessies. De bijbehorende releaserijen zijn open in
[FULL-TEST-RELEASE.md](../FULL-TEST-RELEASE.md):

- **FR-S01** — eerste gebruik en lokaal profiel inrichten (rubric U01, U05, B01);
- **FR-S02** — sessie vergrendelen en ontgrendelen met echte authenticatie (U01, U05, B02);
- **FR-S03** — instellingen wijzigen en na herstart terugvinden (U05, S08).

Gewenst resultaat voor een consument: bij de eerste start in een paar
stappen een eigen profiel maken, zonder leveranciersaccount; daarna inloggen,
vergrendelen en wisselen tussen meerdere personen op dezelfde pc, terwijl
iedereen alleen zijn eigen gegevens kan lezen.

Over **meerdere personen op één apparaat** bestaat nog geen ontwerp. De
P2-simulatie (S01 `#/start` in [PRODUCT-SPEC.md](PRODUCT-SPEC.md)) kent één
lokaal profiel zonder echte wachtwoordinvoer; [DESIGN-AMBITION.md](../DESIGN-AMBITION.md)
en [LOCAL-PRIVACY-CONTRACT.md](../LOCAL-PRIVACY-CONTRACT.md) beschrijven
meerdere *apparaten* van één gebruiker en NAS-gasten, niet meerdere lokale
gebruikers.

### Observaties uit de handmatige proef (agent, geen acceptatie)

Op Simons Windows-pc, bundel `eabc27b`, Windows-QEMU 11.1.0, TCG:

- Het eerste beeld verscheen na ongeveer 30 s. Enter op het startscherm
  opende na 15 s geen document; een muisklik op **Nieuw** wel. Een
  gastmuisaanwijzer was op de schermafbeeldingen niet zichtbaar.
- Het scherm wordt zichtbaar strook voor strook ververst ("magnetisch
  tekenbord"). Dat past bij het bestaande ontwerp: Materiaal doet per
  begrensde iteratie één tiletransactie ([NATIVE-MATERIAAL.md](NATIVE-MATERIAAL.md))
  en de compositor blit iedere tegel via `DisplayClient.blit` naar het
  scherm ([NATIVE-COMPOSITOR.md](NATIVE-COMPOSITOR.md)). Of tegels in het
  zichtbare framebuffer landen of in een buffer, is niet gemeten.
- Invoer wordt gepolld: IRQ1/12 wekt de inputwacht nog niet
  ([NATIVE-INPUT-IDLE-WAIT.md](../contracts/NATIVE-INPUT-IDLE-WAIT.md)).
- Een losse proef met `-accel whpx` startte tot `MATERIAAL:FOCUS:READY` en
  tekende het startscherm; QEMU meldde `Ignoring request for interrupt
  vector 0`. Invoer onder WHPX is niet getest. WHPX is geen geaccepteerd
  profiel.

Deze punten zijn één gebruikerssessie en schermafbeeldingen, geen meting of
gebruikersonderzoek.

## 2. Bestaande bouwstenen

| Bouwsteen | Wat er is | Wat ontbreekt voor profielen |
|---|---|---|
| [B01 Key Service](B01-ENTROPY-AND-KEYS.md) | Entropy, hoofdsleutel, zestien domeinen, intrekking, herstelgeheim zonder escrow | Geen TPM-bescherming, geen wachtwoord-KDF |
| [B02 records](B02-SECURE-RECORDS.md) | Geauthenticeerde records; persistent key-state-blob onder een extern 32-bytegeheim; "een aparte gebruikers-/herstelagent levert het externe geheim" | Die agent bestaat niet; koude rollbackankering en write-ahead zijn open |
| [Versleutelde opslag](ENCRYPTED-OBJECT-STORAGE.md) §5 | Doelmodel DEK per object; Argon2id (RFC 9106) als voorkeurskandidaat voor wachtwoordherstel | Geen implementatie of gemeten parameters |
| Volume/catalogus (M6) en document-session | Native objecten, catalogus en commit | Geen namespace per gebruiker |
| Capabilities, compositor, inputrouter | Focus alleen via vertrouwde compositor; apps kunnen geen input injecteren | Geen sessie-eigenaar, slotscherm of gebruikerswissel |
| [TPM-CRB-transport](../contracts/TPM2-CRB.md) | Begrensde transportkern, opt-in fixture | Geen productgrant of sleutelbinding |
| [NAS-beheermodel](NAS-SHARING-AND-INTERNET.md) | Beheerder mag quota en wissen, niet lezen | Nog niet toegepast op lokale gebruikers |

## 3. Voorgesteld gebruikersmodel (besluit stap 1)

| Rol | Mag | Mag niet |
|---|---|---|
| Eigenaar | Gebruikers toevoegen/verwijderen, ruimte toewijzen, een gebruiker en diens lokale gegevens wissen, systeemupdates goedkeuren | Inhoud, documentnamen of sleutels van andere gebruikers lezen; een wachtwoord van een ander resetten met behoud van diens gegevens |
| Gebruiker | Eigen materiaal, instellingen, apps en rechten beheren | Andere gebruikers of systeembeheer wijzigen |
| Gast (optioneel) | Tijdelijke sessie zonder blijvende opslag | Iets bewaren na afmelden |

Uitgangspunten:

- Geen verplicht leveranciersaccount; alles werkt lokaal.
- Iedere gebruiker heeft eigen sleutelmateriaal. Verwijderen door de
  eigenaar is wissen, geen ontsleutelen: dezelfde scheiding als bij de
  NAS-beheerder.
- Herstel via een herstelcode en/of later een tweede eigen apparaat. Zonder
  geldig geheim kan herstel bewust onmogelijk zijn; geen achterdeur of escrow.
- Eerste versie: één actieve sessie tegelijk; wisselen = vergrendelen en een
  andere gebruiker laten inloggen. Gelijktijdige sessies zijn een latere stap.

Open keuzes voor Simon: pincode naast wachtwoord; blijven sleutels in het
geheugen tijdens vergrendelen (gemak) of worden ze gewist (sterker); is een
gastsessie gewenst; mag de eigenaar ook de enige gebruiker zijn zonder
afzonderlijk beheeraccount.

## 4. Dreigingsmodel en invarianten

Beschermd tegen: een andere lokale gebruiker, een kwaadwillende of
gecompromitteerde app, en het lezen van een uitgeschakelde pc of losse schijf.
Niet beschermd: een gecompromitteerde kernel of vertrouwde systeemdienst,
fysieke aanvallen op een draaiende pc, en zonder TPM-binding een offline
brute-force op het wachtwoord met de schijf in handen (Argon2id maakt dat
duurder, niet onmogelijk). Deze grenzen moeten zichtbaar in de uitleg staan.

Invarianten (eerst ontwerpen, dan interface, dan code en negatieve tests):

1. Een app krijgt uitsluitend capabilities binnen de namespace van de
   ingelogde gebruiker; een geraden object-ID, catalogusnaam of handle van
   een andere gebruiker levert niets op.
2. Het wachtwoord verlaat de sessiedienst niet, wordt niet gelogd en is niet
   rechtstreeks de hoofdsleutel; het ontgrendelt via Argon2id een
   willekeurig gebruikersgeheim dat de B02-key-state ontsluit.
3. Een fout wachtwoord, afgebroken poging of I/O-fout ontgrendelt niets en
   geeft geen gedeeltelijke toegang (fail closed); pogingen worden begrensd.
4. Tijdens vergrendeling krijgen apps geen invoer en kan geen app over het
   slotscherm tekenen; alleen de vertrouwde sessiedienst bezit dat oppervlak.
5. De eigenaar kan een gebruiker wissen zonder diens geheim, maar heeft geen
   bewerking die gebruikersinhoud ontsleutelt.
6. Een onderbroken inrichting laat geen half profiel achter.

Geen zelfbedachte cryptografie: hergebruik de bestaande
XChaCha20-Poly1305/HKDF-SHA-256 uit de Key Service. Een Argon2id-implementatie
wordt pas toegevoegd na de afhankelijkhedencontrole (herkomst, licentie,
onderhoud, `no_std`, build scripts, vaste versie); er is nog geen crate gekozen.

## 5. Stappen

Elke stap is pas klaar met native QEMU-bewijs, inclusief de foutgevallen.
Hosttests, een build of een AI-review zijn geen acceptatie.

| Stap | Resultaat | Afhankelijk van | Acceptatie (native, met negatieve gevallen) |
|---|---|---|---|
| 0. Bediening bruikbaar | Zichtbare muisaanwijzer; toetsenbordactivatie op het startscherm; een frame pas tonen als het compleet is; alleen gewijzigde gebieden hertekenen; invoer via IRQ1/12 i.p.v. alleen polling | Bestaande compositor-, input- en Materiaalcontracten | Gemeten tijd van toets tot zichtbaar pixel in het vaste TCG-profiel, vóór en na; geen half getekend frame zichtbaar; bestaande pixel-, focus- en invoergates blijven groen |
| 1. Besluit gebruikersmodel | ADR met rollen, herstel, sessiemodel en dreigingsmodel uit §3–4 | Simons keuzes | Review door Simon; securityreview van invarianten |
| 2. Fundament | B02 koude rollbackankering en write-ahead; Argon2id met gemeten, versieerbare parameters; persistente instellingenopslag | B01, B02, volume | Herstart behoudt key-state en instellingen; oude snapshot, fout wachtwoord en beschadigd record worden geweigerd |
| 3. Sessiedienst | Ring-3-dienst die gebruikers versleuteld registreert, wachtwoord controleert en per gebruiker capabilities uitgeeft | Stap 2 | Gebruiker A kan B's catalogus/objecten niet openen, ook niet met geraden ID's; begrensde pogingen; geen geheim in logs of serial |
| 4. Eerste start (FR-S01) | Native inrichting: taal/bediening, naam, wachtwoord of pincode, herstelcode, opslagkeuze | Stap 0 en 3 | Terug/annuleren, stroomonderbreking tijdens inrichting zonder half profiel, daarna lokaal beginnen |
| 5. Inloggen, vergrendelen, wisselen (FR-S02) | Inlogscherm, vergrendelen, gebruiker wisselen | Stap 3 en 4 | Herhaald lock/unlock; fout geheim; inputrace; vergrendelde apps krijgen geen invoer en tekenen niet over het slotscherm |
| 6. Instellingen en beheer (FR-S03) | Instellingen per gebruiker; eigenaar voegt gebruikers toe en verwijdert ze | Stap 2–5 | Instelling overleeft koude herstart; ongeldige waarde, volle opslag en I/O-fout zonder valse bevestiging; wissen door eigenaar zonder leesrecht |
| Later | TPM-binding op ondersteunde hardware, tweede eigen apparaat, NAS-sync, gelijktijdige sessies, eventueel ouderlijk toezicht | Hardwareprofiel, NAS-dienst | Afzonderlijke contracten |

Stap 0 kan parallel aan stap 1 en 2: die raakt andere bestanden en lost de
bruikbaarheidsklachten op die een inlogscherm anders onbruikbaar maken.

## 6. Bewijsgrens

Dit document is een plan. Het bewijst geen werkend profiel, geen veilige
authenticatie en geen gebruiksgemak. Gebruikersonderzoek (U06), een
onafhankelijke menselijke securityreview en fysieke hardware blijven
afzonderlijke eisen. Bij de afronding van iedere stap worden PROJECT-STATE,
de voortgangsmeter en zo nodig DECISIONS bijgewerkt volgens AGENTS.md.

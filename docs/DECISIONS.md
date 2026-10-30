# Ontwerpbesluiten

## ADR-B03K1 — Gedeelde, vastgepinde sleutelautoriteit per releasevenster

30 oktober 2026; B03-KEY-POLICY-1, basis `0ade8014`. Loader en update-dienst
gebruiken één `update-core::policy::TrustPolicy` met maximaal acht exacte
hybride sleutelparen, inclusieve releaseversievensters en onvoorwaardelijke
intrekking. Ongeldige of dubbele tabelentries geven geen autoriteit. De
ondertekende versie wordt pas na beide handtekeningcontroles tegen het venster
getoetst. Geen nieuw cryptoalgoritme, capability of dependency.

Alternatief kalenderverval is niet gekozen: de RTC levert geen vertrouwde
kalendertijd. Een manifest mag evenmin zijn eigen vertrouwde sleutels of
intrekkingsbeleid aanleveren. De QEMU-beleidstabel behoudt het bestaande
TEST-paar en pint apart next/revoked/expired-fixtures; deze openbare sleutels
zijn geen productieankers. Vooraf vastgepinde rotatie bewijst geen veilige
distributie/rollbackbescherming van nieuwe beleidsversies.

Contract, negatieve gevallen, open productie-/tijdgrenzen en rollback:
`implementation/B03-SIGNED-UPDATES.md`, B03-KEY-POLICY-1. Terugname is een
gerichte revert naar het eerdere enkelvoudige TEST-verificatiepad, zonder
slot-/manifestformaatmigratie; terugname verwijdert wel lifecycleafdwinging.
Native resultaat en bronbinding staan afzonderlijk in PROJECT-STATE; B04,
B05, B06 en de aanvullende gates van #41 blijven open.

## ADR-053 — Wachten op IPC behoudt bestaande rechten en krijgt ABI 1.10

Bij main-integratie hernummerd vanaf de branchlokale ADR-051; de
main-ADR-051 over sleutelstatus hieronder behoudt zijn bestaande ID.

30 oktober 2026; integratie `NATIVE-TEST-VERSION`, basis `ad2b981`.
De bestaande component `IPC-WAIT-1` wordt overgenomen voor de rode native
Materiaal-gebruikersketen. Zestien gekopieerde handle/interestparen met een
absolute PIT-deadline mogen een proces parkeren; readiness reserveert noch
consumeert iets. Iedere herbeoordeling controleert opnieuw de volledige
capability en PID. Geen userpointer, copyplan of objectreferentie blijft
over een contextwisseling bewaard. Nieuwe rechten worden niet toegekend.

De componentbranch gebruikte nog ABI 1.9, maar main heeft die inmiddels
aan de expliciet niet-vertrouwde RTC-klok toegewezen. Daarom gebruikt alleen
`ipc-wait` ABI 1.10; de RTC-syscall en oude profielen behouden hun gedrag.
De integratie behoudt ook de latere stackbereikcontrole bij userreturn.
Lege uitvoerbare runqueue plus wachtenden mag via de bestaande permanente
servicestack een `sti; hlt; cli` uitvoeren; alleen werkelijk getelde IRQ0's
verhogen de monotone klok. De volledige Engine-digestchecks blijven staan.
Consumenten mogen uitsluitend parkeren zonder lokaal uitvoerbaar werk.
De legacy-keyringlus wacht daarom niet wanneer RANDOM-ingress bestaat;
de volume-adapter wacht pas na zijn synchrone disk- en replywerk.
Desktop-init wacht alleen in rustige Running-fase op registry/controller;
de aparte testcontroller blijft zijn lokale kerneltrigger pollen.

Eerst wordt een aparte native tweeprocessenproef gebouwd, inclusief echte
queue-, intrekkings-, deadline-, pointer- en idlecontroles. Daarna volgt
de bestaande GUI-gate zonder verruimde deadline. Hosttests of een gebouwde
ELF bewijzen geen native voortgang, bruikbaarheid of volledige release.
Herzien vóór SMP of nieuwe IRQ-wakebronnen: het huidige invariant geldt
uitsluitend voor één CPU en geserialiseerde IF0-kernelmutaties.
Rollback is het uitschakelen van de opt-in feature en consumenten; ABI
1.10 blijft een gereserveerde interfaceversie.

## ADR-051 — Persistente sleutelstatus via een afzonderlijk administratief ingress

30 oktober 2026; uitvoering B02-PERSIST-1, basis `ad2b981`. De bestaande
B02-codec `export_persistent`/`restore_persistent` krijgt KEY1-opcodes 11/12
(`EXPORT_PERSISTENT`, `RESTORE_PERSISTENT`). Zij zijn uitsluitend bereikbaar via
een derde, apart gegrant kernel-endpoint van de Key Service dat alleen de
supervisor/herstelagent (in het FR-I01-profiel: init) bezit. Domein-endpoints
en het RANDOM-ingress weigeren deze opcodes vóór cryptografische verwerking;
het administratieve ingress weigert omgekeerd iedere sleutel-, record-,
recovery- en RANDOM-bewerking. Het externe 32-bytegeheim reist in de
aanvraag en wordt door de dienst niet bewaard; een tweede service-adapterpad
voor "admin" bestaat niet, alleen `wire::dispatch_persistent`.

Het geïntegreerde profiel legt de volgorde vast: de app verzegelt eerst in
RAM, meldt `KEYS_READY`, init exporteert en commit het 120-byteblob via de
crashconsistente volumetransactie in volumedomein 1, en pas na `PERSISTED`
commit de app het ciphertextrecord als gewoon document. Zo komt geen
ciphertext op schijf onder een niet-duurzaam vastgelegde sleutelallocatie.
Bij koude herstart op de ongewijzigde read-only image herstelt init de exacte
toestand vóór enig sleutelwerk; een verkeerd geheim, een hogere generatievloer
en een sleutelopcode op het admin-ingress worden geweigerd en geteld.

Afgewezen alternatieven: (a) de Key Service zelf blok-/volumerechten geven —
vergroot de TCB van de sleuteldienst en vermengt opslagherstel met
sleutelbeheer; (b) persistente opcodes op de bestaande domein-endpoints —
elke app met een sleutelendpoint zou dan snapshotbevoegdheid krijgen.
Grenzen: de generatievloer in dit profiel is een gedemonstreerd mechanisme,
geen vertrouwd anker; koude rollbackankering, write-ahead per GENERATE/REVOKE
in de dienst zelf, zeroisatie van de SDK-ontvangstbuffer en een echte
volume-identiteit (nu de profielconstante `NOS-SYSTEM-VOL01`) blijven open.
B02 blijft daarom open; de score verandert niet. Rollback: verwijder de twee
opcodes, het admin-ingress en de init-/appstappen; bestaande KEY1-codes,
formaten en het B01-profiel zijn ongewijzigd.

## ADR-050 — Opt-in wall-clock syscall blijft zichtbaar niet-vertrouwd

30 oktober 2026; uitvoering NATIVE-CLOCK-1, boven op RTC-NATIVE-1.
`native-system-clock` voegt ABI 1.9 syscall `0x73` toe voor een directe,
read-only UTC-snapshot uit CMOS. De gebruikers-SDK retourneert een expliciet
`UnauthenticatedRtc`-type. De klok is openbare procesmetadata; er worden geen
CMOS-poorten, capabilities, pointers, opslag- of netwerkrechten aan apps
gegeven. Ongeldige niet-nulargumenten geven `-22`, mislukte of onbruikbare
hardware geeft `-5`, zonder fallback of verzonnen tijd.

Deze scope kiest voor een werkende, testbare systeemfunctie in het bestaande
single-CPU IF0-profiel, niet voor een vertrouwde productie-TimeProvider. De
adapter verifieert de native RTC-invarianten en indexherstel per aanroep.
Unix-tijd en boot-monotone PIT-ticks blijven gescheiden. Geen
certificaatvalidatie, persistente antirollback, synchronisatie, slaap-/wekcorrectie,
fysiek hardwarebewijs of nieuwe voortgangspunten. De echte ring-3-app en
onafhankelijke QEMU-basistijd-orakel zijn de acceptatiepoort.

Herzien vóór SMP, andere firmware/hardware of TLS-gebruik: dan zijn
geserialiseerd apparaatbezit, geautoriseerde TimeProvider, versheid,
onzekerheidsmarge en rollbackanker nodig. Rollback: verwijder uitsluitend de
opt-in feature, syscall, SDK-bindings en acceptatie; ABI 1.9 blijft als
documenteerde compatibiliteitsrevisie, zonder invloed op oude profielen.

## ADR-049 — Exclusieve native CMOS-bootstrap, geen vertrouwde tijd

30 oktober 2026; implementatie RTC-NATIVE-1, basis `5c334f9`.
De bestaande dependencyvrije RTC-codec wordt alleen voor een opt-in
`native-rtc`-diagnostiek aan de eigen kernel gekoppeld. Na eigen IDT/paging,
op een CPU met IF=0, bezit de proef uitsluitend CMOS-index/data 0x70/0x71.
Een expliciete D/NMI-Enabled-startselectie vervangt geen firmware-shadow;
de readprocedure herstelt deze eigen selectie. Geen RTC-datawrites,
register-C-ACK, app-I/O, nieuwe syscall of productiefeature-implicatie.
Ring-3/framebuffercombinaties worden geweigerd. De kleine privileged codec
is uitsluitend hardwareacceptatie; een productie-TimeProvider blijft een
geisoleerde dienst met afzonderlijk minimaal broker-/rechtencontract.

Alternatief: direct een wandkloksyscall of TLS-klok publiceren zou
niet-geauthenticeerde tijd tot een impliciet vertrouwensanker maken.
De uitvoer houdt daarom `UnauthenticatedRtc`, callerfloor en vluchtige
terugloopcontrole expliciet. Vijf onafhankelijke kalender-oracles plus echte
budget-/floorweigering en device-indexherstel vormen de native poort; de
bestaande hosttests blijven vereist. Exacte uitvoering: PROJECT-STATE en
RTC-WALLTIME. Geen fysiek-, persistent antirollback-, TLS- of productbewijs.
Primair emulatorcontract: QEMU `v10.2.1`, `hw/rtc/mc146818rtc.c`; aanvullend
OSDev CMOS W001717 (revisie 30263). Geen emulatorcode overgenomen.
Rollback: opt-in feature/adapter/runner revert; bestaande M2 en alle
productiediensten blijven ongewijzigd. Herzien bij IRQ/SMP of productgebruik.

## ADR-031 — Native I/O-onzekerheid zonder observerpanic

**Datum/status/eigenaar:** 28 oktober 2026; geïmplementeerd en geaccepteerd op
bron 88e6c04. Native builder A01, integratie root, AI-reviews R08/R09 en A04.

**Context en besluit:** de dienst kende Unknown al, maar de M6c-observer eiste
bij een voorbereide mutatie altijd Ok. Bind de eerste onzekere reply aan een
werkelijk aan de dienst teruggegeven terminale muterende syscallfout; Pending
is geen fout. Latch de I/O-tellingen, commit geen kandidaat, tel geen ACK en
controleer dat beide appdomeinen daarna geen nieuwe I/O veroorzaken. Validatie
en autorisatie blijven vóór de gesloten diensttoestand. Alleen volledige koude
recovery bepaalt de werkelijk opgeslagen versie.

**Alternatieven en gevolgen:** de succesasserties verwijderen zou fouten maskeren;
automatische write-replay kan een onzekere mutatie verdubbelen. Het gekozen pad
houdt normale WRITE+FLUSH-voorwaarden streng en verandert geen capability,
syscall, wire-/journalformaat, dienstimplementatie of externe dependency.
Testinjecties lopen via bestaande driversnapshot-, deadline- en resetpaden.
Geen automatische dienstherstart; historische M6b-observer apart behouden.

**Bewijs en herziening:** de volledige 35-groepen/31-VM-gate en M6b-regressie
(18 groepen/14 VM) slagen, met beide captures en afzonderlijke bewijsaudits.
30 Rust/48 Python-tests voor M6c, 22 Rust/30 Python-tests voor M6b; alle exit 0.
Exacte scope en bron: `verification/M6D-ACCEPTANCE.md`; S07 levert 13% → 14%.
Contract en primaire VIRTIO-bron:
`implementation/M6D-IO-UNCERTAINTY.md`. Geen fysieke devicefout, stroomuitval,
menselijke audit of formeel bewijs. Herzie bij algemene servicesupervisie,
parallelle mutaties, nieuwe I/O-foutmodellen of andere hardware.

## ADR-032 — Vaste 100-puntenroadmap met verplichte herbeoordeling

**Datum/status/eigenaar:** 28 oktober 2026; aanvaard en uitgevoerd op expliciete
gebruikersopdracht. De hoofdagent bezit de gedeelde voortgangsbestanden.

**Besluit:** `docs/progress.json` bevat vaste, binaire acceptatiecriteria met
gewichten die samen 100 vormen. De generator maakt de README, criteriatabel
en lokale SVG. 0% is nog niet gerealiseerd; 100% is een volwaardig zelfstandig
dagelijks OS op aantoonbaar ondersteunde hardware. De eigen kernel blijft
verplicht. De beginscore van de M1–M6c-bewijsstand is 13%.

**Alternatieven/gevolgen:** commits, regels code, testaantallen of M-nummers
meten geen productvoltooiing. Daarom leveren hostprototypes of alleen plannen
geen native punten op. Iedere opdracht actualiseert beoordeling, bron en datum;
een score mag gelijk blijven of dalen. Gewichten zijn een expliciete
planningskeuze, geen wetenschappelijke meting of resterende-tijdsraming.

**Bewijs en herziening:** `tools/progress.py --check` controleert som, unieke
criteria, bestaande bewijsverwijzingen en gelijkheid van gegenereerde weergaven.
Het beoordeelt de bewijsinhoud niet en voert geen native tests uit. Wijzigingen
aan eindscope/gewichten vereisen rubricversie, reden en vergelijking met de oude
score. Zie `PROGRESS.md`; nooit de lat verlagen om een hoger getal te krijgen.

## ADR-030 — Native labels en een catalogus per appdomein

**Datum/status:** 28 oktober 2026; geïmplementeerd en native eindgate geslaagd
op broncommit bc4cd104, basis bb22fdb. Exact contract: implementation/M6C-NAMED-CATALOG.md.
Op deze historische M6c-uitvoeringsbron heeft het native `Unknown`-foutpad een
bekende, ongerepareerde observerpanic, ook met `milestone-m6c`. De afzonderlijke
reparatiebron en haar afzonderlijke acceptatie staan bovenaan in ADR-031.

**Context en besluit:** afgeronde M6b heeft anonieme objectslots. Materiaal heeft
benoemde objecten en list/open nodig. Voeg een aparte dependencyvrije catalog-core
met expliciete IPC v4/journal v2 toe. Namen zijn UTF-8-labels, geen autoriteit of
pad; object-ID en endpoint/caller blijven de toegangsgrens. LIST retourneert één
begrensde snapshot van uitsluitend eigen live objecten met eigen revisiesom.
PUT/RENAME zijn één record met naam/inhoud onder één CAS-revisie. RENAME behoudt
inhoud; WRITE- en FLUSH-completion gaan vóór ACK. Het servicecontract sluit bij
`Unknown` ook LIST af; dit foutpad is alleen via host-ABI getest.

**Alternatieven/gevolgen:** stil hergebruik van v3/v1-padding zou oude readers
misleiden; een losse naamtabel vereist meerdere atomische writes. Automatische
migratie is niet nodig voor deze synthetische proef: v1 weigert vóór READY en de
M6b-reader blijft behouden. Duplicaatlabels zijn toegestaan; geen directoryboom,
normalisatie, encryptie of fysieke secure erase toegevoegd. Kleinere grenzen zijn
proefbudgetten, geen vaste consumentenproductkeuze.

**Bewijs en herziening:** de finale M6c-gate/capture slaagt met 19 groepen en
15 VM-invocaties: 14 M6c plus één gewone M1. M6b-regressie slaagt met 18 groepen
en 14 VM-invocaties op dezelfde fingerprint. De 30 coretests, 33 harnastests en
15 host-ABI-adapterproeven behouden hun eigen scope. De native observer in
`kernel/src/m2/m3/m6c.rs` kan bij WRITE-/FLUSH-fouten of time-outs panikeren
vóór de client `Unknown` ontvangt; gecontroleerde native foutafhandeling was
op die bron niet gerealiseerd. Een afzonderlijke AI-audit
controleert alle 141 bronbestanden, 130 geregistreerde artifacthashes en acht
volumes, inclusief twee externe rename-kills en zes koude recoveries. Details:
`verification/M6C-ACCEPTANCE.md`; geen menselijke audit of formeel bewijs.
De nieuwe kernelmodule bevat een expliciete fixtureobserver plus de bestaande blokbroker; het
catalogusbeleid draait in ring 3. Herstel van het bekende `Unknown`-defect
vereist een afzonderlijke native reparatie met WRITE-/FLUSH-fout- en
time-outinjectie, controle van het werkelijk ontvangen clientantwoord en
nieuwe brongebonden native verificatie. Herzie formaat/migratie vóór echt gebruikersdata,
meer slots, grotere documenten of compositorintegratie. Terugdraaien kan via de
bestaande M6b-build en zijn eigen volumes; geen bestaande image overschrijven.

**Bronnen:** huidige M6B-OBJECT-STORAGE.md/PROJECT-STATE.md op bb22fdb; Rust
bare-metal targetdocumentatie in het M6c-contract. Dit is een projectbesluit.

## ADR-028 — Bewijs per eigenschap en standaard hybride NAS-transport

**Datum/status:** 27 oktober 2026; ontwerp aanvaard en N02-hostproef getest vanaf `ece94a5`. **Context:** de gebruikersblauwdruk vraagt formele verificatie, kleine TCB, capabilities, geheugenbescherming en PQC. De richting past bij het project; de absolute productclaims en het eigen KDF-/tunnelvoorstel zijn niet onderbouwd.

**Besluit:** eigen Rust-kern behouden, formele eigenschappen met expliciete aannames en implementatiebinding uitwerken, CHERI als aparte platformproef. Asymmetrische/PQC-code blijft buiten `kernel/`. Voor de eerste uitvoerbare NAS-transportproef TLS 1.3 met uitsluitend de gestandaardiseerde groep X25519MLKEM768 (RFC 10024), wederzijdse certificaatcontrole/pinning en bestaande peergebonden rechten. Geen eigen combiner/recordcrypto. De optionele-PSK-framing uit het aangeleverde fragment is ambigu. Een klassieke-only peer krijgt geen fallback in dit proefprofiel.

**Alternatieven/gevolgen:** onmiddellijk een ongeteste native crate toevoegen past niet bij de huidige directe-rustc-build, vaste ELF-limieten, kleine IPC, ontbrekende entropycapability en uitgeschakelde SIMD. Een hele kernel vervangen door seL4 is geen gegeven opdracht. De begrensde hostproef geeft werkelijk protocolbewijs zonder een native port te claimen. De proef gebruikt klassieke Ed25519-authenticatie; geen volledige postkwantum-authenticatie-/bootketenclaim. LAN/WAN-labels in loopbacktests zijn expliciete harnasaannames.

**Invariant:** toestemming vóór verbinden; echte peer-authenticatie vóór data; actuele bevoegdheid bij iedere applicatiewrite/response; close/error trekt transportfeit in. Fact bindt aan app, peerpin-generatie en voor internet de intrekkingsgeneratie; oude geauthenticeerde verbindingen herleven niet na hernieuwde toestemming. Intrekking werkt bij de volgende gecontroleerde handeling, geen proactieve idle-socketpush. Client-only documentkeys en het bestaande beheerderverwijderrecht blijven gelden. Een geldig ondertekend pakket verleent op zichzelf geen capabilities. Eenmalig verleende beperkte autoriteit mag automatisch gebruikt worden zonder herhaalde prompts.

**Bewijs/status:** geïntegreerd 124/124 tests (85 service, 39 UI), beide demo's exit 0, 20 syntaxchecks en diffcontrole geslaagd. Echte negatieve TLS-handshakes, geauthenticeerde kwaadaardige serverantwoorden, resourcegrenzen, intrekking en regrant getest. Twee AI-reviews en gerichte onafhankelijke hercontrole van de generatiecorrectie. Commando's/bronhashes/resultaten in `verification/pqc-transport/`. Het eerste 16-policytestcheckpoint is historisch bewaard en niet bij de 124 opgeteld. Geen formeel bewijs of menselijke audit.

**Bronnen:** RFC 10024; seL4 verified configurations/Proofcraft nieuws 2026; CHERI FAQ/CHERI-D; NIST FIPS 203/204/205. Precieze URLs, correcties en native taken staan in `implementation/VERIFIABLE-SECURITY-AND-PQC.md`.

**Herziening/rollback:** herzie bij incompatibele standaarden, library-/bewijsproblemen, native port of nieuwe productinstructie. N02 is een afzonderlijke werkbranch boven PR #7; verwijderen ervan vereist geen schijf-, sleutel- of firmwaremigratie. De nieuwe proef opent uitsluitend loopback. Bestaande N01/M6a-code en bewijsarchieven behouden.

## ADR-027 — Internet en versleutelde persoonlijke NAS-ruimte

**Datum/status:** 27 oktober 2026; aanvaard op expliciete gebruikersopdracht, met geteste hostproef N01 en browsermock. Native integratie staat open. **Eigenaar:** hoofdagent voor contract/integratie; afzonderlijke service- en productbuilders.

**Context:** de gebruiker wil internet, een NAS-rol binnen hetzelfde OS en opslagruimte aanbieden aan vrienden/familie. De host moet zijn ruimte kunnen terugnemen zonder toestemming van de gast, terwijl de inhoud voor hem versleuteld blijft. Dit herziet ADR-006 en de eerdere beperking tot eigen NAS/eigen LAN.

**Alternatieven:** alleen eigen LAN handhaven voldoet niet aan de nieuwe opdracht; een leverancierscloud of beheerderssleutel voor gastinhoud schendt de privacygrens; gastconsent als vereiste voor verwijderen ontneemt de hardware-eigenaar zijn beheerrecht.

**Besluit/invariant:** client-only inhoudssleutels; versleuteling van documentnaam en inhoud; namespace/object/revisie geauthenticeerd. NAS-beheer beheert quota, intrekking en verwijdering maar krijgt geen decryptiesleutel. Revoke/evict invalideert oude sessies en lopende uploads; tombstones verhinderen herleving. Internet is een afzonderlijk apprecht; NAS-internet is expliciet peergebonden, lokale AI blijft netwerkloos. Lokale NAS-beheerrol staat los van een internet-datakoppeling; extern beheer wordt niet stilzwijgend toegevoegd. Een routeplanner kiest automatisch geverifieerd LAN of eerder toegestaan direct internet naar dezelfde peer, zonder herhaalde consentvragen of nieuwe autoriteit. Privacy en gebruiksgemak zijn samen vereist; tijdelijke hostproefcaps zijn geen productkeuze.

**Uitvoering:** dependencyvrije Node-hostproef in services/nas/, zonder sockets of echte NAS. Eigen synthetische ciphertextstore met één schrijver en atomische snapshots. Browserflows Mijn NAS / Ruimte ontvangen / Ruimte aanbieden zijn mocks. Deze ontwikkelmiddelen worden geen verplichte runtime van het eigen OS.

**Testbewijs:** gezamenlijke suite 89/89 (50 service, 39 UI) en eindige demo exit0; bronhashes/commando's in verification/nas/report.json. Echte childproceskills na bevestigde opslagmutaties, geen fysieke stroomuitvaltest. Vier concrete reviewbevindingen zijn hersteld: noncebudget bij staging, tijdelijke ciphertextsnapshots bij wissen, race bij writerlockherstel en ongewenst beheerrecht via internet.

**Grenzen/gevolgen:** hostidentiteit en transportfacts zijn synthetisch; geen TLS-/native netwerkbewijs. Herstelde inhoudssleutel kan alleen lezen, niet veilig blijven schrijven op meerdere apparaten. Browserinternet is in N01 appgebonden; bestemmingsgrants voor gewone apps/downloaders volgen nog. Geen volledige snapshot-rollbackbescherming, veilige fysieke erase of plug-and-play-NAT-claim. Native clientdistributie/key lifecycle, netwerkcapabilities, beheeropslag, onboarding en menselijke audit blijven afzonderlijke integratievoorwaarden.

**Bronnen:** Node.js v24.22.1 Crypto (https://nodejs.org/docs/latest-v24.x/api/crypto.html), RFC 8446 (https://www.rfc-editor.org/rfc/rfc8446.html) en NIST SP 800-207 (https://doi.org/10.6028/NIST.SP.800-207), geraadpleegd 27 oktober 2026. Functionele keuze en adminverwijderrecht zijn projectbesluiten. Exact contract: implementation/NAS-SHARING-AND-INTERNET.md en LOCAL-PRIVACY-CONTRACT.md revisie 3.

**Migratie/rollback/herziening:** geen bestaande echte NAS, router, kernel-ABI of M6a-image wordt gewijzigd. Bewaar oudere M1–M6a-resultaten. N01 kan als afzonderlijke featurecommit worden teruggedraaid; geen destructieve reset van gebruikerswerk. Herzie bij bewijs van key-/nonceproblemen, multi-writerintegratie, andere transport-/opslaglaag of een nieuwe gebruikersinstructie. Oude LAN-only teksten zijn op de actieve leesroutes bijgewerkt.
De oorspronkelijke revisie-5-principes hieronder zijn behouden als projectbesluiten. Uitvoeringsbesluiten vanaf ADR-019 beschrijven de gebouwde richting; actuele M1–M5-testuitkomsten en grenzen staan in PROJECT-STATE.md.

| ID | Besluit | Reden | Herzien als |
|---|---|---|---|
| ADR-001 | Eigen kernel; geen bestaande kernel als basis | Expliciete gebruikerswens | Simon bewust de projectscope verandert |
| ADR-002 | Rust no_std + minimaal assembly | Kleine expliciete onveilige grens | Target/tooling dit aantoonbaar verhindert |
| ADR-003 | x86_64 QEMU q35, UEFI, software-emulatie baseline | Controleerbare eerste testomgeving | Praktische toolchainbeperking wordt aangetoond |
| ADR-004 | Kleine capability-gebaseerde microkernel als richting | Autoriteit expliciet en TCB klein houden | Prototypes aantoonbare problemen met complexiteit/latency tonen |
| ADR-005 | Standaarden en gecontroleerde bibliotheken toestaan | Zelf crypto/alle tools bouwen vergroot risico en scope | Herkomst/licentie/onderhoud onacceptabel is |
| ADR-006, herzien door ADR-027 | Lokaal account, geen OS-telemetrie, lokale data of E2EE naar gekoppelde eigen/gedeelde persoonlijke NAS; internet per peer expliciet | Expliciete gebruikersuitbreiding 27 oktober 2026 | Alleen na expliciete gewijzigde gebruikersopdracht |
| ADR-007 | PQC-voorbereiding vanaf ontwerp; integratie pas met echte use case | Protocol- en sleutelmigratie voorkomen | Relevante standaarden/implementaties wijzigen |
| ADR-008, herzien | Product-/interactieonderzoek parallel vanaf M0; echte systeemintegratie volgt bewezen kernfuncties | Originaliteit en hoogwaardige UX moeten het systeem mede sturen | Bijstelling op experimenten, geen stille reductie tot standaarddesktop |
| ADR-009 | Geen kernelbrede claims over formeel bewijs | Eigen code erft geen seL4-bewijs | Afgebakend daadwerkelijk geverifieerd resultaat bestaat |
| ADR-010 | AI-inferentie uitsluitend op apparaat; geen netwerkrecht of cloudfallback | Expliciete gebruikerscorrectie | Alleen na expliciete gewijzigde gebruikersopdracht |
| ADR-011 | Installeerbare derdenapps met gescheiden bestands-, NAS- en netwerkrechten | Toegankelijk ecosysteem zonder impliciet vertrouwen | Acceptatietests tonen ontbrekende grens |
| ADR-012 | Quantum als derde onderzoeksspoor vanaf M0: PQC, lokale QPU-uitvoering en hybride systeemport | Oorspronkelijke ambitie expliciet behouden | Bijstelling alleen met onderbouwd onderzoek; privacycontract blijft gelden |

## Gekozen uitvoeringsbesluiten — 26 oktober 2026

| ID | Besluit | Bewijsgrens / herziening |
|---|---|---|
| ADR-013 | Eigen UEFI-loader met r-efi 7.1.0; Rust 1.90.0; x86_64/QEMU TCG | Versies onderzocht; combinatie nog niet gebouwd. Exact bootcontract in BOOT-AND-TOOLCHAIN.md |
| ADR-014 | Eerste kernel: vaste ELF-layout en gedeelde BootInfo; daarna begrensde processen, IPC en capabilities | Details en negatieve tests in KERNEL-AND-SECURITY-CONTRACT.md; geen isolatiebewijs vóór uitvoering |
| ADR-015 | Materiaal als eerste productrichting; P2 lokale HTML/CSS/JS-proef | Vergelijking uitgevoerd, geen gebruikersproef. Herzie op concrete gebruiksobservaties |
| ADR-016 | Q1 safe Rust statevector-core met qsim-v0, maximaal 12 qubits, hostadapter apart | Analytische fixtures gecontroleerd; geen Rust-/QPU-/OS-portbewijs |
| ADR-017 | Later ML-KEM-768 en ML-DSA-65 via beoordeelde implementaties/protocolbinding | Geen eigen cryptoprimitives, geen volledige bootketenclaim |
| ADR-018 | EU-01 t/m EU-08 als technische acceptatie-eisen; toepasselijkheid wetgeving per release | EUROPE-AND-RELEASE.md bevat conditionele matrix, geen conformiteitsclaim |

## Template voor nieuw besluit
ID, datum, status, context, alternatieven, besluit, onderbouwing/bronnen, gevolgen, security-invariant, testbewijs, migratie/rollback en eigenaar. Schrijf wat de keuze zou weerleggen; vermijd besluiten zonder toetsbare aannames.

## Uitvoeringsbesluiten — 27 oktober 2026

**ADR-019 — Windows-ontwikkelpakket en alleen-lezen virtio-ESP.** Rust1.90.0/MSVC gebruikt de bestaande Visual Studio2026/SDK. QEMU11.1.0 en het bijpassende CODE/VARS-paar komen uit dezelfde vastgelegde Windows-package; hashes staan in docs/verification/toolchain-provenance.json. Dit is de toegestane ontwikkelvariant, niet de byte-identieke referentiebuild. De oorspronkelijke IDE-drive gaf vóór loaderuitvoering `Block node is read-only`; de ESP gebruikt daarom `if=virtio,readonly=on`. Iedere run krijgt een eigen ESP-/VARS-kopie, zonder netwerk of hostdisk. Werkelijke cold-boots en foutproeven staan in docs/verification/boot-runs.json. Herzie bij andere firmware/platformen; verwijdering van de afzonderlijke toolmap herstelt de host zonder globale PATH-wijziging.

**ADR-020 — Vaste kernelspan32–48MiB en BootInfo1.1.** In de echte OVMF-kaart overlapt het oorspronkelijke interval2–18MiB ACPI NVS en BootServicesData. AllocatePages wees de reservering af met EFI_NOT_FOUND; diagnose in lokaal build/run/smoke-fe68c261dd/result.json. Het profiel gebruikt nu `[0x02000000,0x03000000)`, dezelfde16MiB. Loader, parser, linker en fixtures zijn samen gewijzigd; de header is nog128bytes met dezelfde offsets, maar vereist expliciet ABI1.1. ABI1.0 wordt afgewezen. Er wordt geen bezet firmwaregeheugen hergebruikt. De volledige acceptatiereeks met deze waarden slaagt. Terugdraaien vereist gezamenlijk aanpassen van beide binaries/validator/linker en een aantoonbaar passende firmwarekaart; nooit alleen een adresfallback invoeren.

**ADR-021 — Statische GOT in .data.** De compiler produceert vijf statisch opgeloste pointers. Die staan expliciet in de bestaande RW-.data-outputsectie; er komt geen runtime-relocatieverwerker of extra segment. De buildinspectie blijft dynamische metadata, relocaties en onbekende allocatable secties weigeren. De drie ELF-laadsegmenten en hashes staan in docs/verification/boot-manifest.json. M1 controleert segmentintentie; echte W^X-afdwinging blijft M2.

**ADR-022 — Voortgang per stap naar GitHub.** De gebruiker wil het project bij verbruikslimieten volledig kunnen hervatten. Broncode, lockfiles, compacte bewijsrapporten en open punten gaan daarom na elke afgebakende stap en vóór lange controles naar de privéwerkbranch. Een checkpoint mag onvoltooide controles bevatten mits expliciet gemarkeerd. Codespaces is niet vereist voor deze bewaarroutine; een Codespace is niet gestart. Main blijft de startspecificatie totdat de concept-PR is beoordeeld.

**ADR-023 — Alleen WB-capabele normale ConventionalMemory, cachebits gecombineerd.** De eerste M2-run faalde vóór KS-02 doordat OVMF `0x0f` meldt en de allocator uitsluitend exact WB accepteerde. UEFI-attributen beschrijven mogelijkheden; UC/WC/WT/WB mogen samen voorkomen. De allocator vereist WB, staat uitsluitend deze vier cachebits toe en weigert overige/bijzondere attributen. Bron: TianoCore/EDK II, `edk2-stable202508`, [UefiSpec.h, EFI_MEMORY_DESCRIPTOR en cachebits](https://github.com/tianocore/edk2/blob/edk2-stable202508/MdePkg/Include/Uefi/UefiSpec.h), gecontroleerd 27 oktober 2026. De huidige package is daarmee niet als exact die firmwareversie geïdentificeerd. Regressie test zowel `0x0f` als geweigerde special/unknown-bits; echte M2-poort slaagt. Andere platformprofielen vereisen opnieuw beoordeling van geheugenattributen; geen stil reclaim van Loader/BootServices/runtime/ACPI-geheugen.

**ADR-024 — Statische2048-mappingarena en één tijdelijk scratchslot.** Iedere procescontext heeft de contractcapaciteit2048 mappings; grote metadata staat statisch buiten de16KiB-kernelstack. PID is monotoneu64; interne ASID is afzonderlijk begrensd. Allocatorzeroing/tabeltoegang gebruikt een tijdelijk supervisor/NX-slot, met invalidatie vóór gebruik en na clear. Een writable scratchalias naar een executable fysiek frame wordt geweigerd; readonly input heeft een aparte RO/NX-helper. Geen willekeurige identitymap van vrije RAM/gaten/MMIO. Bewijs: kernelbron ed09818, volle M2/M3-gate, echte67-userpage/BSS-zero-regressie en per-processcleanupbalans. Herzie bij SMP/DMA/nieuwe mapping-API; de serialisatie geldt alleen in het huidige één-CPU/één-threadprofiel. Herstel via een consistente eerdere bronversie in een afzonderlijke checkout, zonder huidige wijzigingen te wissen.

**ADR-025 — Safe vaste capability-engine met afzonderlijke usercopygrens.** `capability-core` is eigen dependencyvrije no_std-code zonder unsafe/heap; de kernel bewaakt userpointers, syscallvolgorde en320-byte-kopieën. Engine-receive commit pas na volledige outputvalidatie, daarna volgt de in dit profiel onfeilbare bytecopy. Rootautoriteit komt uitsluitend uit het statische testmanifest, niet uit een algemene userspace-objectlookup. Revoke trekt afstammelingen in; exit ook eigen/issued autoriteit. Gesloten objecttombstones blijven bestaan tot laatste ref. Hosttests vergelijken de volledige Engine-toestand; kernelchecksums zijn diagnostiek. Bewijs:21coretests,alle acht echte M4-cases en onafhankelijke gerichte review. Herzie vóór threads/sharedmemory/DMA/dynamische applicatie- of objecttypen; geen extrapolatie naar zulke garanties. Geen boot-ABI-wijziging of hostconfigmigratie.

**ADR-026 — M5: geïsoleerde READ-dienst met vertrouwd virtio-transport.** De eerste systeemdienst draait als eigen Rust-programma in ring 3. De kernel bewaakt de getypeerde READ-capability, tickets, usercopy, PCI/MMIO en een vaste vijfpagina-DMA-pool. Het device op `00:05.0` gebruikt één splitqueue van acht plaatsen en maximaal één gelijktijdige sectorread. De dienst kan geen fysieke adressen, descriptors, PCI-configuratie of MMIO kiezen. Het protocol en de 17 foutcases staan vóór uitvoering vast in [M5-BLOCK-SERVICE.md](implementation/M5-BLOCK-SERVICE.md). Alleen eigen lokale crates zijn toegevoegd; geen nieuwe externe dependency.

**Status:** aanvaard en geïmplementeerd voor het vaste M5-QEMU-profiel. **Alternatieven:** alle dienstlogica in de kernel (grotere vertrouwde basis), of directe userspace-DMA (vereist aanvullend isolatieontwerp, buiten M5). **Eigenaar:** hoofdagent voor integratie en besluitbeheer; `kernel_m2` voor transport en mappinggrens.

**Onderbouwing/bronnen:**

- OASIS Open, *[Virtual I/O Device (VIRTIO) Version 1.2](https://docs.oasis-open.org/virtio/virtio/v1.2/cs01/virtio-v1.2-cs01.html)*, Committee Specification 01, 1 juli 2022: §2.4 (resetbevestiging), §2.7 en §2.7.8 (splitqueue en used-entry zonder epoch), §4.1 (PCI-transport) en §5.2 (blokdevice). Queuegrootte, één gelijktijdige read en de gepinde pool zijn projectkeuzes.
- Intel Corporation, *[Intel® 64 and IA-32 Architectures Software Developer’s Manual, Volume 3A: System Programming Guide, Part 1](https://www.intel.sg/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.pdf)*, Order No. 253668-060US, september 2016: §4.1.4 en §4.5 (CPUID/MAXPHYADDR en gereserveerde fysieke adresbits), §§11.12.2–11.12.3 (IA32_PAT en selectie met PAT/PCD/PWT). `map_mmio` controleert de CPU-adresbreedte en vereist UC in PAT-entry 3 vóór de 4KiB-mapping met PAT=0, PCD=1 en PWT=1; de firmware-PAT blijft ongewijzigd. Bronnen gecontroleerd op 27 oktober 2026.
De vertrouwde basis omvat het kerneltransport en het protocolconforme QEMU-device/backend. M5 bevat geen IOMMU: CPU-isolatie en begrensde descriptoropbouw bewijzen geen containment van kwaadwillige DMA. De pool blijft gepind tot VM-stop; hergebruik vereist completion of bevestigde reset. Zonder resetack blijft zij in quarantaine en het device offline. Een used-entry heeft geen epoch; softwaretickets vervangen geen hardware-resetgarantie. Herzie dit besluit vóór directe userdriver-DMA, SMP, fysieke hardware of een andere devicebackend.

De eerste scope is alleen lezen op een synthetische 64KiB-wegwerpimage. Schrijven, persistentie en herstel blijven M6. Herstart gebruikt een begrensde testcoördinator en echte nieuwe dienst-PID/epoch plus IPC-capoverdracht; geen algemene spawn-API. De client blijft leven. Het externe testorakel controleert sectorchecksums op twee verschillende images bij dezelfde kernel en controleert dat alle imagebytes ongewijzigd blijven. Uitgevoerde resultaten en open punten staan uitsluitend in PROJECT-STATE.md en verification/m5/. Herstel: een afzonderlijke checkout van M1–M4 op `c630c59`; geen hostdisk- of firmwaremigratie.


## ADR-029 — Native objectdienst met revisies en begrensd journal

**Datum/status/eigenaar:** 27 oktober 2026, geïmplementeerd en volledige native gate geslaagd binnen het begrensde QEMU-profiel. Het native `Unknown`-foutpad heeft een bekende, ongerepareerde observerpanic. Hoofdagent integreert, afzonderlijke builders en AI-reviewers controleren service, broker en harnas.

**Context en besluit:** M6a bewaart één vaste waarde. M6b voegt een echte ring-3-objectdienst toe boven dezelfde begrensde READ/WRITE/FLUSH-ABI: vier objecten in twee endpointgebonden domeinen, GET/PUT/DELETE, maximaal 128 bytes en 32 append-only records. De limieten zijn proefgrenzen. De object-core is eigen dependencyvrije safe Rust; er worden geen nieuwe externe crates toegevoegd. De kernel houdt de bestaande DMA-broker plus een expliciete testobserver; objectpolicy draait in de dienst.

**Alternatieven en gevolgen:** een volledig bestandssysteem met namen, garbage collection en encryptie ineens toevoegen maakt de herstel- en bevoegdheidsgrens veel groter. Een extra hoststore levert geen native vooruitgang. Dit begrensde native fundament maakt meerdere objecten en blijvende verwijdering nu toetsbaar, met expliciete latere uitbreidingen voor ruimtehergebruik, namen, crypto en de desktop. Geen migratie van bestaande M6a-images; M6b gebruikt uitsluitend eigen wegwerpvolumes.

**Invariant:** de ontvangende endpointcapability bindt het domein. Geen appvelden als blok- of domeinautoriteit. Mutaties vergelijken de actuele revisie, en alleen een echte WRITE-completion gevolgd door FLUSH-completion mag een succesvolle reply opleveren. Een tombstone behoudt de revisie, zodat oude verzoeken geen verwijdering ongedaan maken. Corruptie geeft geen stilzwijgende terugval. Het servicecontract vereist dat onzekere mutaties `Unknown` opleveren en verdere geldige dienstoperaties blokkeren zonder automatische replay. Dit `Unknown`-contract is alleen op host-ABI-niveau getest en is voor native WRITE-/FLUSH-fouten niet gerealiseerd door de bekende observerpanic. Full verschijnt vóór muterende I/O. Verwijderen is logisch; oude payloadbytes blijven in het journal en zijn niet cryptografisch beschermd.

**Onderbouwing en bewijs:** eigen contract `implementation/M6B-OBJECT-STORAGE.md`, eerdere virtio/capability-ABI uit ADR-026/M6a; wire- en journalindeling zijn interne projectbesluiten. De volledige eindgate op `1e61a20` slaagt: 18 groepen/14 echte VM-runs, 22 Rust-hosttests en 26 harnastests. Daaronder vallen zes koude recoveries, uitputting, corruptie en twee echte externe kills. Capture en een aparte audit van 121 artifacthashes/zeven schijfvolumes slagen; een extra Python-parser controleert onafhankelijk de echte schijfbytes zonder de Rust-journalcode te delen. Zeven host-ABI-adapterproeven controleren `Unknown`-uitkomsten. Native WRITE-/FLUSH-fouten kunnen via `sent()` in `kernel/src/m2/m3/m6b.rs` een kernel panic veroorzaken: de observer vergelijkt de reply eerst met een verwachte `Ok`-reply. Dit defect is niet gerepareerd; gecontroleerde end-to-end-afhandeling als `Unknown` ontbreekt ook in `milestone-m6b`. De native gate bevat geen devicefoutinjectie voor dit pad. Exacte bron, regressies en bewijsgrenzen staan in `PROJECT-STATE.md` en `verification/m6b/`.

**Herziening en rollback:** een herziening van het native `Unknown`-pad vereist een native reparatie, gerichte WRITE-/FLUSH-devicefoutinjectie, controle van het daadwerkelijke clientantwoord en nieuwe native verificatie met bijbehorende bronbinding. Herzie ook bij meer objecten, compaction, meerdere schrijvers, sleutels, native UI of andere devices. M6b heeft een eigen feature/ELF/volumeformaat; bestaande M1–M6a-profielen blijven selecteerbaar. Een aparte werkbranch maakt terugname mogelijk zonder hostschijven of echte gebruikersdata te wijzigen. Tests in QEMU bewijzen geen fysieke stroomuitval, IOMMU-containment, formele correctheid of productierijpheid.

Aanvulling 28 oktober 2026 op ADR-029: de terminale WRITE-/FLUSH-observerfout
is op `592dccf` hersteld en native getest met twee statussnapshotinjecties;
de historische beweringen hierboven gelden voor hun oude bron. Verder herstel
van lokale poll-LIMIT, echte brokerdeadline en resetquarantaine is de actieve
vervolgstap K08; uitgevoerde resultaten blijven in PROJECT-STATE.md.

## ADR-033 — opt-in native grafische bootstrap vóór productintegratie

Datum 28 oktober 2026, aanvaard als begrensde ontwikkelstap; root integreert,
G09 beheert core/loader/adapter. Het browserprototype geeft geen bewijs voor
native scherm- en invoerondersteuning. Daarom komt eerst een afzonderlijke
G0-feature met gevalideerde GOP-metadata, supervisor framebuffer en begrensde
PS/2-polling. Alternatieven zijn een volledige compositor/driver/IPC-broker
ineens bouwen (te grote eerste hardwarestap), of een browser op de host als
native presenteren (geen voortgang op eigen hardwaregrens).

Invariant: opt-in ABI 1.2 met vaste descriptor in de behouden BootInfo-pagina,
geen firmwarecalls na EBS, geen framebufferalias naar RAM/user/executable,
strikt begrensde RW/NX/UC-devicepagina's en safe `no_std`-render/invoercore.
De kernelweergave is tijdelijke diagnostiek, geen app-, object- of opslagpolicy.
Het scherm labelt de proef. Geen rechten voor willekeurige apps en geen nieuwe
extern gepubliceerde productie-ABI.

Contract/bronnen/acceptatie: [GRAPHICS-G0.md](implementation/GRAPHICS-G0.md).
Testbewijs wordt afzonderlijk opgeslagen; dit besluit is geen boottest.
Herzie vóór ring-3-compositor, IRQ-invoer, DMA/GPU, SMP, Unicode/IME of echte
hardware. Herstel via aparte checkout `7b4945f`; standaard BootInfo 1.1 blijft
selecteerbaar, geen echte schijf- of firmwaremigratie.


## ADR-034 — Native SDK en begrensde quantumport als afzonderlijke profielen

Datum 28 oktober 2026; geïmplementeerd als ontwikkelprofielen, eindacceptatie
op de gecombineerde bron nog uit te voeren. Root beheert ABI/bootconfig;
G01–G03 bouwen SDK/client, I01–I04 de port en controles.

De SDK hergebruikt de bestaande capability-ABI en 320-byte IPC-envelop.
Canonicalisatie, endpointrechten, antwoordherkomst en één uitstaand verzoek
zijn expliciet begrensd. Alleen EAGAIN vóór enqueue mag opnieuw worden
geprobeerd. Na enqueue, Unknown, ongeldige reply of deadline sluit de sessie
zonder automatische mutatieherhaling. De catalogus bepaalt revisies/namen;
de kernel krijgt geen app- of documentbeleid. De bestaande M6c-probe kan het
SDK-transport selecteren, terwijl ruwe negatieve ABI-proeven hun oude pad
behouden. Een apart native voorbeeld gebruikt de echte catalogusclient;
compilatie is nog geen bewijs van algemene appstart.

De quantumport gebruikt de ongewijzigde eigen qsim-core met maximaal zes
qubits, 64 gates, 4096 shots en een vaste arena van 64 KiB per proces. Een
monotone allocator vermijdt onbewezen hergebruik; de proef beëindigt de
processen waarna de bestaande kernelcleanup de pagina's terugneemt. Software-
float uit de gepinde Rust-toolchain maakt de eerste port mogelijk zonder
schedulerwijziging voor FPU/SIMD-context. Een disassemblygate controleert het
concrete ELF inclusief instructieprefixes; deze controle is geen formeel
bewijs. Een nieuw opt-in profiel bevat twee userprocessen en uitsluitend een
vaste kerneltestobserver. Het profiel is exclusief ten opzichte van M4/G0.

De alternatieven — nieuwe syscalls, onbegrensde allocatie, FPU-state support
of een volledige jobdienst tegelijk — vergroten deze eerste wijzigingsgrens.
Gevolg: native referentieberekeningen en resourcegrenzen zijn nu uitvoerbaar,
maar willekeurige jobaanvoer/resultaten/annulering buiten het gecompileerde
proefprogramma blijven een eigen volgende stap. De Control-fixtures bewijzen
geen externe cancel of klokdeadline. L02 krijgt daarom nog geen punt.

Eigen contracten: SDK README/ABI, `implementation/QUANTUM-SPEC.md` en de
bestaande capability/M3-contracten. Gerichte native uitvoering, exacte
opdrachten, hashes, reviewbevindingen en grenzen worden afzonderlijk bewaard
in `verification/parallel-*`. Herzie vóór dynamische apps, meer qubits,
allocatiehergebruik, FPU/SIMD, SMP of een echte jobdienst. Beide profielen zijn
opt-in; herstel door hun eerdere bron in een aparte checkout te bouwen.
Geen migratie van echte data, nieuwe externe crate of fysieke hardwareactie.


### ADR-034: afzonderlijke invarianten en weerlegging

**SDK — security-invariant:** gewone apps verkrijgen alleen reeds toegekende
procesgebonden endpointrechten. Elke reply wordt canoniek en op herkomst
gecontroleerd. Na enqueue en verlies van zekerheid blijft een mutatie zonder
automatische replay; een ongeldige reply geeft geen extra rechten of succes.
**Weerlegd als:** dezelfde mutatie na Unknown/verloren reply opnieuw wordt
verstuurd, een vreemde afzender wordt geaccepteerd of een ongeldig capability-
bericht als geldig doorgaat. **Eigenaar:** G01–G03 voor SDK, root voor ABI en
integratie. **Testbewijs:** parallel-g01/g03 en parallel-final-sdk-m6c (deze
laatste toetst de runtimebridge, niet de losse catalogusvoorbeeldapp).
**Migratie/rollback:** bestaande ABI en journal blijven gelijk; SDKselectie is
opt-in, terugname via eerdere bron in aparte checkout zonder datamigratie.

**Quantum — security-invariant:** elk proefproces heeft private mappings en
maximaal 64 KiB arena; input blijft binnen 6 qubits / 64 gates / 4096 shots. SIMD/FPU en
vreemde syscall/hardware-instructies zijn niet toegestaan in het geselecteerde
scalarartifact. Simulatorbeleid blijft userspace; opruiming gebruikt echte
procesframes. **Weerlegd als:** grenzen kunnen worden overschreden, processen
elkaars geheugen bereiken, ongewenste instructies door de artifactgate komen
of cleanup toegewezen frames achterlaat. **Eigenaar:** I01–I04 voor port/gate,
root voor de testobserver en integratie. **Testbewijs:** parallel-i04,
parallel-final-qsim en parallel-final-m3. Dit is geen formeel bewijs.
**Migratie/rollback:** exclusief opt-in profiel; eerdere bron afzonderlijk
bouwen, zonder opgeslagen gebruikersdata of hardwarefirmware te wijzigen.


## ADR-035 — Security Floor blijft actief bij vrijwillig minder privacy

**Datum/status/eigenaar:** 28 oktober 2026; aanvaard als product- en
architectuurinvariant, nog niet volledig native gerealiseerd. Hoofdagent bewaakt
integratie; security-eigenaars toetsen de enforcementgrenzen.

**Context:** Nieuw OS moet privacybeschermende standaarden hebben, maar de
gebruiker mag bewust online diensten, telemetrie, microfoon, cloudsync of
andere gegevensdeling toestaan. Een dergelijke keuze mag niet veranderen in
een algemene “trusted app”- of “minder veilige modus”.

**Besluit:** privacykeuzes verruimen alleen expliciete capabilities en
gegevensstromen. Kernel-/procesisolatie, capabilitycontrole, geheugen- en
sleutelisolatie, pakket-/update-authenticiteit, bescherming van niet-gedeelde
data en toekomstige DMA/IOMMU-grenzen blijven buiten die privacykeuze.

**Security-invariant:** toestemming vergroot functionaliteit, niet vertrouwen.
Een internetgrant alleen geeft geen document-, key-, camera-, microfoon-,
NAS-, proces- of kernelrecht.

**Weerlegd als:** een privacytoggle, providerkoppeling, telemetry-opt-in of
focus/gamingmodus een bredere privilegeklasse activeert of bestaande
isolatiechecks omzeilt.

**Bewijs/migratie:** toekomstige native LP-/capabilitytests; geen nieuw
acceptatiepunt door dit document. Bestaande grants blijven semantisch gelijk.
Rollback: herstel dit ADR en de gekoppelde contracten gezamenlijk; geen
datamigratie in deze documentatiestap.

## ADR-036 — Externe provider mag alleen opaque ciphertextopslag zijn

**Datum/status/eigenaar:** 28 oktober 2026; aanvaard architectuurdoel, nog geen
native provideradapter. Opslag-/securityeigenaar beheert formaat en keygrens.

**Context:** de eerdere revisie verbood leverancierscloud volledig. De gebruiker
wil desgewenst bestaande algemene opslag zoals OneDrive/Google Drive kunnen
benutten zonder die provider leesrecht op de opgeslagen inhoud te geven.

**Alternatieven:** absolute provideruitsluiting beperkt keuze; provider-native
encryptie/key escrow verschuift vertrouwen naar de provider; zelfgemaakte
cryptografie is onaanvaardbaar.

**Besluit:** een expliciet gekozen externe provider mag uitsluitend opaque,
client-side versleutelde Nieuw-OS-objecten opslaan. De provider krijgt geen
inhoudssleutel, documentplaintext, gevoelige manifestplaintext, lokale zoekindex
of AI-context. Geen stille providerfallback. Het OS mag een eigen versieerbaar
container-/objectformaat definiëren boven onderhouden standaardcryptografie.

**Security-invariant:** bezit van alleen providerstate/credentials levert geen
geaccepteerde documentplaintext op binnen het gedocumenteerde threat model.

**Weerlegd als:** providercredentials of opgeslagen bytes zonder een geldige
Nieuw-OS-key voldoende zijn voor plaintext; een provideradapter andere lokale
objecten/keys kan lezen; of een outage stil een andere provider activeert.

**Bewijs/migratie:** [implementation/ENCRYPTED-OBJECT-STORAGE.md](implementation/ENCRYPTED-OBJECT-STORAGE.md)
definieert EO-acceptatie. De bestaande NAS-route blijft geldig. Geen echte
provideraccount, productiekey of gebruikersdata wordt door dit besluit
geconfigureerd.

## ADR-037 — Eigen premium native productiviteitsfamilie, geen Office-ripoff

**Datum/status/eigenaar:** 28 oktober 2026; productbesluit voor P3/P4/M7,
nog niet native gerealiseerd. Product-/UI-eigenaar beheert designsysteem;
graphics-eigenaar beheert later renderer/compositorgrenzen.

**Context:** privacyvriendelijke alternatieven mogen niet aanvoelen als
verouderde systeemsoftware. Tegelijk zou een directe kopie van Office,
Windows of macOS de eigen productidentiteit ondermijnen.

**Besluit:** Materiaal blijft het objectmodel. Eigen tekst-, tabel-,
presentatie-, PDF/media- en latere productiviteitstools delen één designsysteem,
contextuele gereedschappen, geschiedenis, delen, herstel, motion en
toegankelijkheid. Geen permanente Office-ribbonkopie als basis. Een gedeelde
native GPU-geaccelereerde UI-/renderlaag is het langetermijndoel zodra de
graphicsbasis dat toelaat; exacte renderer/API blijft een later technisch
besluit. DOCX/XLSX/PPTX/PDF zijn interoperabiliteitsdoelen die per feature
moeten worden getest.

**Productinvariant:** herkenbare basishandelingen blijven voorspelbaar; de
eigenheid komt uit Materiaal, objectcontinuïteit, contextuele tools en
capabilitybewust delen, niet uit onbekende bediening omwille van originaliteit.

**Weerlegd als:** kernapps eindigen als inconsistente losstaande webviews,
de interface structureel een bestaande suite kopieert, of privacy alleen via
beheerachtige UX bruikbaar is.

**Bewijs/migratie:** PX-acceptatie in PRODUCT-SPEC.md en latere gebruikers-/
native tests. Het huidige P2-browserprototype blijft ontwerpgereedschap en
wordt niet door dit ADR tot productruntime verklaard.

## ADR-038 — Privacyuitleg is optioneel; capabilitybeslissingen blijven echt

**Datum/status/eigenaar:** 28 oktober 2026; aanvaard UX-/securitycontract,
nog niet native afgedwongen.

**Context:** begrijpelijke voorbeelden en kritische vragen kunnen privacykeuzes
duidelijker maken, maar herhaalde waarschuwingen kunnen de ervaring verstoren.
De gebruiker wil die privacy-popups volledig kunnen uitschakelen.

**Besluit:** de UI scheidt (a) noodzakelijke beslissingen over nieuwe
capabilities van (b) educatieve privacyuitleg. Educatieve uitleg kan globaal
of per app uit. Een reeds geldige grant wordt niet bij iedere normale
handeling opnieuw gevraagd. Focus/gaming onderdrukt niet-urgente educatieve
meldingen; een bewust gestarte nieuwe gevoelige functie mag nog wel een
compacte noodzakelijke grantvraag tonen.

**Security-invariant:** het uitschakelen van privacyuitleg wijzigt geen
capabilityset en creëert geen impliciete “allow all”.

**Weerlegd als:** dezelfde statewijziging zowel de uitleg als toegangsrechten
uitzet/verruimt, of het systeem op basis van “uitleg uit” een nog niet verleende
capability als toegestaan behandelt.

**Bewijs/migratie:** LP-29/PX-04 en toekomstige native UI-/brokerproeven. Bestaande
grants migreren niet; alleen de presentatievoorkeur wordt afzonderlijk bewaard.

## ADR-039 — DocumentSession native controleren via de bestaande catalogus

**Datum:** 28 oktober 2026.  
**Status:** implementatie geïntegreerd in de auditbranch; host-/crossbuildbewijs en een
actuele twee-VM normal/koude-recovery-proef zijn uitgevoerd, maar dit verandert
geen bestaand roadmapcriterium.  
**Eigenaar:** app/SDK voor de gebruikersproef en library, runtime voor de
acceptatiepoort, root voor selectie en integratie.

**Context:** de bestaande M6c-probe gebruikte de catalogus via directe helpers,
terwijl de echte `DocumentSession`-library al bestaat. De integratiestap moet
die echte library in ring 3 gebruiken zonder kernelobserver, capabilityrechten,
IPC-envelop of opslagformaat te verruimen.

**Alternatieven:** (1) de directe catalogushelpers blijven gebruiken, waardoor
de appbibliotheek niet native wordt getoetst; (2) een afzonderlijke user-ELF met
gekopieerde cataloguscode bouwen, wat duplicatie en bewijsoppervlak vergroot;
(3) de bestaande M6c-userflow opt-in de echte library laten gebruiken en de
bestaande catalogusmodules linken. Gekozen is optie 3.

**Besluit/implementatie:** `DocumentSession<NativeTransport>` draait opt-in in
de bestaande M6c-gebruikersprocessen. `NOS_DOCUMENT_SESSION=1` vereist
`NOS_SDK_RUNTIME=1` en is beperkt tot normal/recover. Openen, lokaal bewerken,
CAS-save, opnieuw openen en LIST blijven userspace. `list()` houdt één eigenaar
van de endpoints; service en typed client delen de bestaande catalogusmodules.

**Security-invariant:** de route gebruikt uitsluitend reeds toegekende
procesgebonden endpoints. Replies blijven canoniek gevalideerd; LIST mag lokaal
concept/conflict/onzekerheid niet wissen en een gesloten sessie niet heropenen.
Onzekere saves worden niet automatisch herhaald. Syscall-, IPC-, geheugen- en
capabilitygrenzen worden niet verruimd. ADR-035 (Security Floor) blijft volledig
van toepassing.

**Onderbouwing/bronnen:** `docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md`,
`sdk/document-session/README.md`, `docs/verification/parallel-g11/`,
`docs/verification/parallel-i13/` en `docs/verification/parallel-i15/`.
Voor de afzonderlijke netwerkparser gelden RFC 768, RFC 791 en RFC 1071, zoals
vastgelegd in `services/net-protocol/README.md`.

**Testbewijs:** de repositorybrede audit-CI voert DocumentSession-hosttests,
SDK-/compilerbindingcontroles, native M6c-build, een baseline M6c-QEMU-smoke en
twee afzonderlijke DocumentSession-VM's (normal en koude recovery) uit.
Historische bewijsbundels blijven aan hun oorspronkelijke bron gekoppeld.

**Weerlegd als:** de proef nieuwe rechten nodig heeft, een draft via LIST
verliest, een onzekere save herhaalt, een gesloten endpoint opnieuw bruikbaar
maakt, de observer/oracle moet versoepelen of een andere binary als documentapp
kan meetellen.

**Migratie/rollback:** standaardprofielen behouden hun bestaande selectie. De
opt-in kan worden uitgezet; er is geen migratie van echte gebruikersdata,
firmware of fysieke apparaten. De headless documentproef is geen Materiaal-GUI,
algemene appstarter, grotere-documentondersteuning of ruimtehergebruik.

## ADR-040 — Copy-on-write volume (M6e) in plaats van append-only journal

**Datum:** 28 oktober 2026. M6b/M6c/M6d blijven ongewijzigd en afzonderlijk
bouwbaar.

**Status (gescheiden):** *ontwerp* geaccepteerd voor het opt-in profiel
`native-volume`. *Implementatie*: `services/volume`, de native dienst/app in
`sdk/examples/volume-docs`, de SDK-blok-API en kernelprofiel `native-volume` op
PR #47. *Testbewijs*: zie hieronder; GitHub Actions moet op iedere nieuwe
PR-head opnieuw groen zijn. *Onbekend/niet getest*: fysieke schijven en
stroomuitval, authenticatie/encryptie, back-up/export en grotere volumes.

**Onderbouwing/bronnen:** OASIS, *Virtual I/O Device (VIRTIO) Version 1.2*,
Committee Specification 01, 1 juli 2022, §5.2 Block Device, §5.2.6.2:
een write is stabiel wanneer na de voltooide write een `VIRTIO_BLK_T_FLUSH`
wordt verstuurd en zelf voltooid is
(https://docs.oasis-open.org/virtio/virtio/v1.2/cs01/virtio-v1.2-cs01.html,
geraadpleegd 28 oktober 2026). De volgorde inhoud → FLUSH → tabel → FLUSH →
ACK steunt op die garantie; een emulator die haar niet nakomt valt buiten dit
bewijs.

**Testbewijs:** hosttests `services/volume/tests/` (incl. uitputtende
crashsimulatie en de test met twee gemengde schrijfpogingen),
`tests/test_volume_disk.py`, en de native gate `tools/test_volume.py` met
bewijs in `docs/verification/m6e/` en
[M6E-ACCEPTANCE](verification/M6E-ACCEPTANCE.md). Wat daar niet staat, is
niet uitgevoerd.

**Besluit:** grote documenten en ruimtehergebruik (S04/S05) krijgen een nieuw
userspace-volumeformaat met twee alternerende, per sector verzegelde
tabelkopieën en 4 KiB-inhoudschunks. Een mutatie schrijft alleen naar chunks
die de duurzame tabel niet refereert; daarna FLUSH, de volgende tabel in de
oudere slot, FLUSH, ACK. Vrijgekomen chunks worden door latere commits
hergebruikt. Er is geen journal en dus geen recordlimiet; een aparte
compactiestap is niet nodig, omdat er geen append-only log is om te
compacteren. Crashconsistente vrijgave vervult hier de rol die "compactie" in
criterium S04 beschrijft. Deze interpretatie wordt hier expliciet vastgelegd,
niet stilzwijgend toegepast.

**Security-invariant:** de kernel blijft een generieke blokbroker zonder
formaat- of domeinkennis (kernel/AGENTS.md: geen opslagpolicy in privileged
code). Domeinautoriteit komt uitsluitend van het kernelgebonden endpoint. Apps
krijgen geen blokrecht. Read-only recovery wordt afgedwongen door emulator,
virtio-onderhandeling, capability (alleen READ) en dienst. Een fout tijdens de
tabelwrite geeft `Unknown` zonder replay. Een niet-canonieke, volledig
verzegelde nieuwste kopie stopt herstel.

**Weerlegd als:** recovery na een crashpunt een andere staat oplevert dan
vóór of na de onderbroken stap; een write buiten de scope of naar een
gerefereerde chunk plaatsvindt; guest en onafhankelijk orakel verschillende
digests vinden; quota/Full pas ná I/O worden gemeld; of een read-only
recovery de image wijzigt.

**Reviewcorrectie (CodeRabbit, PR #47):** de oorspronkelijke verzegeling bond
een sector alleen aan slot, index en generatie. Twee onderbroken pogingen voor
dezelfde generatie konden daardoor samen een schijnbaar volledige kopie vormen
en een teruggedraaide mutatie laten herleven. Iedere sector draagt nu ook een
digest over de payload van alle sectoren van zijn kopie; recovery accepteert
alleen een kopie waarvan alle sectoren dezelfde, herberekende digest hebben.
Een gemengde kopie geldt als gescheurd.

**Restrisico:** één onleesbare sector in de nieuwste tabelkopie leidt tot
terugval naar de vorige generatie (verlies van de laatste commit). FNV is geen
authenticatie (B02 open). Proefgeometrie: 16 objecten, 64 KiB per document,
256 KiB inhoud.

**Migratie/rollback:** opt-in feature; geen automatische migratie van M6c-
volumes en geen gebruik van echte schijven. Terugdraaien = feature niet
selecteren of de M6e-commits gericht reverten.

**Eigenaar:** hoofdagent voor formaat, contract en integratie
(`services/volume`, `tools/volume_disk.py`, gate en bewijs); `kernel_m2` voor de
generieke blokbroker in `kernel/src/m2/m3/volume.rs`.

## ADR-041 — Volume v2: collecties, metadata, begrensde historie en transacties (S06)

**Datum/status:** 29 oktober 2026, geaccepteerd voor het opt-in profiel
`native-volume`. Formaat v2 (`NOSVOL02`) en IPC v6 vervangen v1/IPC v5 binnen
dat profiel; er is geen migratie, omdat het profiel alleen wegwerpimages in
QEMU gebruikt. M6b/M6c/M6d blijven ongewijzigd.

**Status:** ontwerp geaccepteerd. Implementatie: `services/volume` (v2),
`sdk/examples/volume-docs` (dienst en app), kernelgeval `test-volume-cut-txn`,
orakel en gate. Testbewijs: zie
[S06-ACCEPTANCE](verification/S06-ACCEPTANCE.md). Onbekend: fysieke schijven en
stroomuitval, authenticatie/encryptie (B02), back-up/export (S09), grotere
historie en de native Materiaal-interface.

**Context:** Materiaal werkt vanuit objecten met collecties, type, labels en
geschiedenis; herstel maakt een nieuwe revisie (PRODUCT-SPEC §2, P2-05). M6e had
alleen naam en actuele inhoud.

**Alternatieven:** (a) een journal of log van bewerkingen naast de tabel:
verworpen, omdat het de recordlimiet en compactie terugbrengt die ADR-040
juist verwijderde; (b) onbeperkte historie via een aparte versieketen op schijf:
uitgesteld, omdat begrensde opslag en quota dan nog niet te bewijzen zijn;
(c) collecties als gewone objecten: verworpen, omdat lidmaatschap dan niet
atomair met objectmetadata te controleren is.

**Besluit:** iedere objectentry krijgt een tweede tabelsector met twee
historische versies; de kop bewaart `tags`, `kind`, collectie en logische
tijden. Die tijden komen uit een klok per domein in de tabelheader: het aantal
commits van dat domein (de klokken tellen op tot de generatie). Collecties
staan in een eigen tabelsector per volume (4 per domein) en zijn leesbaar met
COLLECTION_STAT, ook read-only. RESTORE maakt van een historische versie een
nieuwe revisie zonder inhoud te herschrijven (chunks worden opnieuw
gerefereerd); de teruggezette versie verlaat de historie en de vervangen
actuele versie neemt haar plaats in, zodat geen andere versie wegvalt. DELETE
bewaart de laatste versie als historie; PURGE geeft historie vrij. TRANSACT
voert maximaal vier META/DELETE/PURGE-stappen op verschillende objecten van
één domein uit in één tabelcommit. Alle wijzigingen gebruiken het
ongewijzigde M6e-commitprotocol. Een schrijfbare nieuwe instantie flusht na
recovery vóór het eerste verzoek; mislukt dat, dan start zij niet.

**Onderbouwing/bronnen:** ADR-040 (commitvolgorde, VIRTIO 1.2 §5.2.6.2) en
PRODUCT-SPEC §2/§6 voor de takenlijst. Geen nieuwe externe bron nodig.

**Security-invariant:** domein komt uitsluitend van het kernelgebonden
endpoint; een transactie kan alleen objectindices binnen het eigen domein
noemen, en een object kan alleen in een collectie van het eigen domein staan.
Quota telt behouden historie mee, dus historie kan geen ruimte buiten de quota
vasthouden. Leesantwoorden (STAT, LIST, COLLECTION_STAT, READ) en de revisies,
stempels en tellers in mutatieantwoorden bevatten alleen toestand van het eigen
domein (tijdstempels gebruiken de eigen domeinklok; LIST geeft een
domeinteller). Twee kanalen tussen domeinen zijn bewust restrisico: `Full`
hangt af van de gedeelde, overboekte volumeruimte (2 × 40 van 64 chunks), en
een mislukte tabelwrite van één domein sluit de instantie, waarna ook het
andere domein `Unknown` krijgt. De kernel blijft formaatloos.

**Weerlegd als:** een gedeeltelijk toegepaste transactie na een crash of fout
zichtbaar is; RESTORE inhoud wijzigt of andere historie wist; een object naar
een collectie van een ander domein kan; historie quota omzeilt; of guest en
onafhankelijk orakel verschillende domeindigests (inclusief collecties)
vinden.

**Reviewcorrecties (29 oktober 2026, vóór acceptatie):** een onafhankelijke
review vond vijf punten die in dit besluit zijn verwerkt, elk met een
regressietest: (1) de globale generatie in STAT
liet een domein de commits van het andere tellen; (2) collecties waren via IPC
alleen schrijfbaar; (3) RESTORE van de nieuwste versie bij volle historie wiste
de oudste; (4) een nieuwe instantie kon schrijven terwijl de gekozen kopie nog
alleen in de schijfcache stond; (5) de gate kon een kill vóór de tabelwrite
niet onderscheiden van een gescheurde tabelwrite. De runner eist nu een echt
gescheurde poging (prefix van g+1-sectoren boven de vorige kopie).

Een tweede review (van die correcties) vond geen fout in de productiecode,
maar wel tests die de bedoelde fout niet vingen en twee te ruime claims. Ook
die zijn gecorrigeerd:
- de test op ongerelateerde generaties werd vacuüm door de klokken;
- de LIST-cursortest bereikte de overloop niet;
- COLLECTION_STAT had geen negatieve tests;
- de losse klokgrenzen waren niet afzonderlijk getest;
- invariant 7 claimde volledige non-interferentie;
- een read-only antwoord op een `Unknown` commit werd als definitief
  gepresenteerd.

Sindsdien is de mutatiecontrole herhaalbaar en vastgelegd.
`tools/mutate_volume.py` schakelt per mutant precies één regel uit in een
kopie van de bron en eist dat ten minste één test faalt. Het rapport met
commando, exitstatus en falende tests staat in
`docs/verification/s06/volume-mutants.json`.

**Gevolgen:** v1-images worden door v2 als corrupt geweigerd (andere magic);
het M6e-bewijs blijft gebonden aan zijn eigen bron, en de S06-gate herhaalt de
S04/S05-scenario's op v2.

**Restrisico:** historie is begrensd tot twee versies (de oudste valt weg bij
een nieuwe versie); geen zoekindex; FNV is geen authenticatie. `Full` en de
instantiebrede `Unknown` na een mislukte tabelwrite zijn zichtbaar over
domeinen heen. Een read-only antwoord op een `Unknown` commit is voorlopig.

**Migratie/rollback:** geen migratie van v1-volumes; terugdraaien = de
S06-commits gericht reverten (dan geldt weer v1 met zijn eigen bewijs).

**Eigenaar:** hoofdagent voor formaat, contract, orakel en gate; `kernel_m2`
voor het kernelgeval `test-volume-cut-txn`.

## ADR-042 — Procesbeheer als capabilities: `Launch`- en `Process`-objecten, spawn/kill/status (K05)

**Datum:** 29 oktober 2026. Profiel `native-supervisor`, opt-in; alle
bestaande profielen blijven ongewijzigd bouwbaar.

**Status (gescheiden):** *ontwerp* vastgelegd in
[K05-PROCESS-MANAGEMENT](implementation/K05-PROCESS-MANAGEMENT.md).
*Implementatie*: `crates/capability-core` (objecttypen, rechten 128/256/512),
`kernel/src/m2/m3/supervisor.rs` (syscalls 0x30, 0x50–0x52, orphan-reaping),
`sdk/runtime` (ABI 1.3-wrappers), `sdk/examples/supervisor` (supervisor,
worker, client). *Testbewijs*: hosttests en de gate `tools/test_supervisor.py`;
zie [K05-ACCEPTANCE](verification/K05-ACCEPTANCE.md). Wat daar niet staat, is
niet uitgevoerd. *Onbekend*: SMP, fysieke hardware, gebruikersgeleverde images
(A02), beleid voor een systeembrede dienstbeheerder.

**Besluit:**
1. Starten van een proces vereist een `Launch`-capability op een ingebouwde
   image. Zulke objecten maakt alleen het bootmanifest aan. Een proces zonder
   `Launch` krijgt `EBADHANDLE`, `ETYPE` of `EACCESS` en er ontstaat niets.
2. `proc_spawn` geeft het kind alleen de maximaal twee expliciet meegegeven,
   verminderde capabilities (dezelfde regels als IPC-overdracht: `TRANSFER`
   vereist, nooit `RECV`, geen rechtenvergroting). Alle fallibele controles
   gaan vooraf aan iedere toewijzing.
3. De starter ontvangt de enige handle op een `Process`-object met
   `OBSERVE|KILL`. Het objecttype kent geen `DERIVE`, `TRANSFER` of `REVOKE`:
   deze autoriteit kan niet worden gedeeld of gestolen.
4. Dood (exit, fault, kill) loopt via het bestaande `cleanup`-pad; het
   `Process`-object bewaart toestand en code tot de eigenaar zijn laatste handle
   sluit. Een lopend kind zonder handles blijft als object bestaan, zodat de
   kernel de band met de starter kan blijven afdwingen.
5. Kinderen van een gestorven starter worden in dezelfde kernelgebeurtenis
   beëindigd, iteratief (hoogstens 16 rondes), nooit recursief op de
   kernelstack. Er blijft geen proces over zonder verantwoordelijke.
6. `endpoint_create` (0x30) wordt een gewone syscall binnen de bestaande
   quota (4 eigen endpoints per proces, 64 totaal). Dit herziet de M4-keuze
   "alleen het manifest maakt endpoints" bewust, omdat herstartbare diensten
   een eigen ingress nodig hebben; de RECV-eigendomsregel blijft onveranderd.

**Security-invariant:** autoriteit over processen zit uitsluitend in
kernelmetadata van het aanroepende proces; een getal uit een ander proces
levert hoogstens eigen bestaande autoriteit op (KS-12). Geen gedeeltelijke
staat na een geweigerde of mislukte spawn: de vrije framestand is voor en na
gelijk, anders stopt de run met `PARTIAL_SPAWN`. `check_invariants` eist dat
een lopend `Process`-object naar een levend proces wijst en dat ieder kind
door precies één object wordt genoemd.

**Weerlegd als:** een proces zonder `Launch` een kind kan starten; een
`Process`-handle via IPC of `derive` bij een ander proces belandt; na exit,
fault of kill frames, nodes, endpoints of gequeued capabilities achterblijven;
na de dood van een starter een kind blijft draaien; of een geweigerde spawn de
vrije framestand verandert.

**Restrisico:** een supervisor kan zijn eigen kinderen misbruiken binnen de
rechten die hij heeft; er is nog geen beleid voor wie supervisor mag zijn.
Exitcodes worden modulo 2^56 gerapporteerd. De ABI-minor stijgt naar 1.3;
oudere profielen melden 1.0–1.2 en kennen de nieuwe syscalls niet (`ENOSYS`).

**Migratie/rollback:** opt-in feature; terugdraaien = profiel niet selecteren
of de K05-commits gericht reverten. De rechtenbits 128/256/512 zijn in
`KNOWN_RIGHTS` opgenomen; oudere profielen wijzen ze bij endpoint- en
blokobjecten met `EINVAL` af, zoals voorheen.

**Eigenaar:** hoofdagent (engine, kernelprofiel, SDK, voorbeeld, gate, bewijs).

## ADR-043 — Entropy als capability en een ring-3 Key Service met onderhouden crypto-crates (B01)

**Datum:** 29 oktober 2026. Profiel `native-keyring`, opt-in; bestaande
profielen blijven ongewijzigd.

**Status (gescheiden):** *ontwerp* in
[B01-ENTROPY-AND-KEYS](implementation/B01-ENTROPY-AND-KEYS.md). *Implementatie*:
`drivers/virtio-rng`, `Entropy`-objecttype in `capability-core` (recht 1024),
kernelbroker `kernel/src/m2/m3/keyring.rs` (syscalls 0x60/0x61),
`services/keyring` (`keyring-core`), SDK-runtime ABI 1.4,
`sdk/examples/keyring`. *Testbewijs*: [B01-ACCEPTANCE](verification/B01-ACCEPTANCE.md);
wat daar niet staat, is niet uitgevoerd. *Onbekend*: echte hardware-RNG-
kwaliteit, TPM/hardwarebescherming, wachtwoordherstel, zijkanalen.

**Besluit:**
1. De kernel bezit het virtio-rng-apparaat en levert ruwe bytes alleen aan de
   houder van een `Entropy`-capability (manifest-only objecttype zonder
   `DERIVE`/`TRANSFER`/`REVOKE`). De kernel mengt, cachet of hergebruikt niets
   en bevat geen cryptografie (kernel/AGENTS.md).
2. Gereedheid en falen zijn expliciet: 1024 bytes opstartentropy door de
   SP 800-90B-tests Repetition Count (cutoff 6) en Adaptive Proportion
   (venster 512, cutoff 62; conservatief H = 4 bit/byte, α = 2⁻²⁰). Afwezig,
   vastgelopen of kort leverend apparaat → `BLOCKED` met reden; geen
   terugval, geen testseed in het productpad. Latere reseeds doorlopen
   dezelfde tests; falen → `DEGRADED` en weigering.
3. Alleen onderhouden, vastgepinde RustCrypto-crates met portable backends:
   `chacha20poly1305` 0.10.1 (XChaCha20-Poly1305), `chacha20` 0.9.1,
   `hkdf` 0.12.4, `sha2` 0.10.9, `zeroize` 1.8/1.9. De DRBG is de
   fast-key-erasure-constructie op de ChaCha20-keystream (Bernstein, 2017),
   geen eigen primitief. Dependencycontrole: licenties MIT/Apache-2.0,
   buildscripts alleen `generic-array` (`version_check`) en `zerocopy`-vrije
   boom; geen SIMD-instructies in de gegenereerde code (objdump-controle).
4. Sleutels verlaten de Key Service nooit; apps krijgen (domein, index)-
   handles; het domein volgt uit het kernelgebonden endpoint. AEAD bindt
   formaat, domein en index in de AAD. Per domein 16 sleutels; intrekking is
   permanent en reist mee in de herstelblob.
5. Herstel zonder escrow: een willekeurig 256-bit herstelgeheim voor de
   aanvragende app en een blob (hoofdsleutel + intrekkingsbitmaps, 76 bytes)
   onder dat geheim. RESTORE in een andere instantie herleidt alle
   domeinsleutels; de dienst bewaart het geheim niet.

**Security-invariant:** geen sleutelgeneratie vóór `READY`; geen bytes uit
een geweigerde, kort leverende of time-out-aanvraag; `entropy_poll` schrijft
pas na validatie van de hele uitvoerbuffer en wist de brokerkopie; een
gestorven aanvrager krijgt zijn antwoord niet alsnog via een ander proces.
Cross-domain- en manipulatiepogingen leveren `Auth`/`BadHandle` zonder
gedeeltelijke plaintext.

**Weerlegd als:** een proces zonder `Entropy` bytes krijgt; een dienst
sleutels maakt zonder geslaagde opstarttests; gemanipuleerde ciphertext,
tag of nonce plaintext oplevert; een hersteld domein een ingetrokken sleutel
weer bruikbaar maakt; of de kernel na afloop frames, nodes of objecten
achterlaat.

**Restrisico:** QEMU levert host-`/dev/urandom`; de tests toetsen gedrag,
niet de werkelijke min-entropie van fysieke hardware. De hoofdsleutel staat
onbeschermd in het dienstgeheugen (geen TPM). Zeroize is hygiëne, geen
garantie over kopieën in registers of crashgeheugen. Eén broker-aanvraag
tegelijk: diensten herhalen bij `EAGAIN`.

**Migratie/rollback:** opt-in; terugdraaien = profiel niet selecteren of de
B01-commit reverten. Rechtenbit 1024 wordt bij andere objecttypen met
`EINVAL` afgewezen.

**Eigenaar:** hoofdagent (driver, engine, kernelbroker, core, SDK, voorbeeld,
gate, bewijs).

## ADR-049 — Hybride ondertekende kernelupdates met slotopslag, versievloer en terugval (B03)

Bij integratie van PR #61 met main `6d3bbf0` hernummerd van ADR-044 naar
ADR-049: ADR-044 op main beschrijft het netwerk. Beide besluiten blijven
inhoudelijk behouden; historisch bewijs wordt niet herschreven.

**Datum:** 29 oktober 2026. Loaderfeature `signed-boot` en kernelprofiel
`native-update`, beide opt-in; loaders zonder de feature (alle andere
profielen) houden het bestaande ESP-pad (`kernel.elf`) ongewijzigd. Een
`signed-boot`-loader valt nooit op dat pad terug.

**Status (gescheiden):** *ontwerp* in
[B03-SIGNED-UPDATES](implementation/B03-SIGNED-UPDATES.md). *Implementatie*:
`crates/update-core` (manifest `NOSMAN01`, slotopslag `NOSSLOT1`, hybride
verificatie, selectie- en installatiebeleid), `boot/loader/src/signed.rs`,
`kernel/src/m2/m3/update.rs` (generieke blokbroker over de slotopslag, één
gelabelde crashfixture), `sdk/examples/update` (ring-3 update-dienst),
`tools/update-tool` (keygen/sign/pack/verify), `tools/update_disk.py`
(onafhankelijk hostorakel), gate `tools/test_update.py`. *Testbewijs*:
[B03-ACCEPTANCE](verification/B03-ACCEPTANCE.md); wat daar niet staat, is niet
uitgevoerd. *Onbekend*: firmwareverificatie van de loader zelf (Secure Boot,
B04), sleutelrotatie en intrekking in productie, netwerkdistributie,
fysieke schijven en stroomuitval, gedrag met een aanvaller die OVMF_VARS kan
schrijven.

**Besluit:**
1. Een kernelimage wordt alleen gestart uit een slot waarvan het manifest
   met **beide** vastgepinde ankers verifieert, Ed25519 (`ed25519-dalek`
   2.2.0, serial backend) én ML-DSA-65 (RustCrypto `ml-dsa` 0.1.1, FIPS 204),
   over dezelfde 84 canonieke bytes, en waarvan het image exact de lengte en
   SHA-256 (`sha2` 0.10.9, `force-soft`) uit dat manifest heeft. Eén geldige
   handtekening is onvoldoende; er is geen terugval naar één algoritme.
2. De loader bewaart de hoogste gestarte versie in de UEFI-NV-variabele
   `NieuwOsMinVersion` (vendor-GUID `50402e54-a387-47bc-8f75-dcd79cd0a603`,
   NV+BS) en weigert ieder slot met een lagere versie als `DOWNGRADE`, ook
   als de header ernaar wijst. De vloer stijgt alleen.
3. De header van de slotopslag is een onondertekende hint voor de slotvolgorde.
   Faalt het aangewezen slot (formaat, sleutel-ID, handtekening, digest, vloer,
   lezen), dan wordt het andere slot geprobeerd; falen beide, dan stopt de
   boot gecontroleerd (`BOOT:FAIL:NO_VALID_SLOT`, exit 0x11); ontbreekt de
   slotopslag, dan `BOOT:FAIL:NO_STORE`. Een lege, ongeldige of ontbrekende
   slotopslag valt nooit terug op een onondertekende ESP-kernel, en de build
   legt voor dit profiel geen `kernel.elf` op de ESP. Een correct ondertekend
   maar onlaadbaar image (ELF-parser) is een `FORMAT`-weigering van dat slot;
   de vloer stijgt pas voor een image dat de loader ook werkelijk kan laden.
4. De update-dienst schrijft uitsluitend naar het slot dat **niet** draait.
   Het draaiende slot wordt bepaald door de geverifieerde manifestversie die
   gelijk is aan de door de kernel gemelde eigen imageversie, niet door de
   header. Volgorde: image, FLUSH, manifest, FLUSH, header, FLUSH; een
   onderbreking laat het actieve slot en de header intact en de installatie
   is idempotent en hervatbaar.
5. De dienst verifieert de inboxbundel met dezelfde ankers, controleert de
   digest vóór de eerste schrijfactie en weigert `version ≤ huidige` als
   up-to-date respectievelijk downgrade. Loader en dienst controleren
   onafhankelijk; de kernel kent geen manifest- of slotformaat en bevat
   geen cryptografie (kernel/AGENTS.md).
6. De ankers in de repository zijn **TEST-ankers** (`tools/update-keys/TEST-*`,
   gegenereerd met `update-tool keygen`), inclusief seeds, zodat CI
   reproduceerbaar ondertekent. Zij beschermen niets buiten de gate. Een
   releaseprofiel vervangt de ankers in een afzonderlijke, buiten CI gehouden
   ondertekenstap; sleutelrotatie en intrekking horen bij B04.
7. Dependencycontrole (`cargo tree -p update-core`): `ed25519-dalek` 2.2.0 en
   `curve25519-dalek` 4.1.3 (BSD-3-Clause), `ml-dsa` 0.1.1, `module-lattice`
   0.2.3, `hybrid-array` 0.4.15, `keccak`/`shake`/`sponge-cursor`,
   `signature` 2.2/3.0, `sha2` 0.10.9, `digest`, `crypto-common` (MIT OR
   Apache-2.0), `subtle` 2.6.1 (BSD-3-Clause). Buildscripts: alleen
   `curve25519-dalek` (platformdetectie voor de woordbreedte, geen netwerk) en
   `generic-array` (`version_check`); één proc-macro (`curve25519-dalek-derive`,
   via `syn`/`quote`). De SIMD-backends zijn uitgeschakeld
   (`curve25519_dalek_backend="serial"`, sha2 `force-soft`); `objdump` van
   loader en dienst toont 0 xmm/ymm- en 0 x87-instructies.
   De ML-DSA-verificatie houdt ongeveer 200 KiB stack vast: de loader draait de
   selectie op een eigen 1 MiB-allocatie en het profiel `native-update` geeft
   de dienst 512 KiB gebruikersstack in plaats van 64 KiB.

**Security-invariant:** geen byte van een kernelimage wordt uitgevoerd
voordat formaat, sleutel-ID's, beide handtekeningen, lengte, digest en de
versievloer zijn gecontroleerd; het laatst geldig gestarte slot blijft bij
iedere onderbreking van een installatie ongewijzigd beschikbaar.

**Gevolgen:** de slotopslag kost 6 MiB per apparaat (drie slots van 2 MiB);
`image_length` is begrensd op 2 MiB. Pakket- en app-updates (A02),
netwerkdistributie (N-spoor) en de verificatie van de loader door de firmware
(B04) blijven open. De `KERNEL:IMAGE_VERSION`-marker en de startpagina-
woorden 120/192–400 zijn profielgebonden testinterfaces, geen OS-ABI.
### Aanvulling ADR-043 — monotone herstelstatus en doorlopende bewaking (29 oktober 2026)

Concrete post-merge tegenvoorbeelden uit #52: een oudere recoveryblob wist
lokale tombstones en iedere reseed maakte een nieuw gezondheidstestvenster.
De herstelwijziging behoudt Health vanaf startup en verenigt tombstones bij
herstel van dezelfde master. Een andere master mag uitsluitend naar een nog
ongebruikte instantie; een gebruikte instantie weigert de wissel zonder
wijziging. Geen wire-format- of cryptoprimitiefwijziging. Na koude herstart
blijft rollbackdetectie afhankelijk van toekomstige vertrouwde persistente
status (B02); een recoveryblob alleen bewijst geen actualiteit.

De broker bevestigt virtio-reset vóór hergebruik; bij ontbrekende reset-ACK
blijft DMA gepind en de broker offline. Eigenaarsverlies stopt het apparaat en sluit de broker af; de DMA-pool
blijft in quarantaine tot reboot. Twee aparte native fixtures bewijzen een echt stille
bron en een normale reset/herstart; de oude `stuck` is uitsluitend een
constante-bytefixture. Geen fysieke entropy- of hardwareherstelclaim.

Eigenaar: hoofdagent; wijzigingen en bewijs in native-recovery-hardening.
Herziening: persistent antirollback-/recoverycontract in B02, of een concreet
tegenvoorbeeld tegen de levende-instantie-invariant. Terugdraaien: revert
herstelcommits en B01 open houden, geen onzeker bewijs accepteren.

## ADR-044 — Native netwerk via één begrensde capabilitybroker

**Datum:**30oktober2026. In uitvoering als deel van de volledige release;
geen N01/NAS/TLSacceptatie. NetworkSource is uitsluitend aan het geconfigureerde
ring3netwerkproces toegekend; apps krijgen geen MMIO/DMA of algemeen netwerkrecht.
Eigen kernel valideert handles, volledige usercopy en lengtes, beheert gepinde
RX/TX-buffers, completion/timeout en reset-ACK/quarantaine. ARP/DNS/TCP en later
TLS/configuratie blijven in userspace. ABI1.5:0x70send/0x71completion/0x72receive;
sluiten/dood beëindigt de bron. Geen automatische pakket-replay. Ondersteund
proefapparaat: modern virtio-net00:07.0, één CPU/QEMU256MiB. Werkelijke native
ARP/UDP/DNS-counters en onafhankelijke8/7testpeer slagen op3879c32, met dezelfde
koude volumerecovery. Volledige app-/bestemmingsbroker blijft verplicht open.

Testfirmware is geen OS-netwerkclient. Een echte OVMF-MLDrepro leidde tot de
EDK2-runtimeopties VirtioNetSupport/IPv4Support/IPv6Support=no en romfile=,
met intacte PCI-NIC en strikte peer. Causale oude-kernelA/Bproef bevestigt dit.
Terugdraaien: netwerkfeature uit de integratiebranch revert, N01 open houden;
geen prebootframe negeren of driver-/rechtencontrole uitschakelen.

## ADR-045 — Eager x87/SSE-context en echte boot-monotone tijd

**Datum:**30oktober2026. Opt-inmechanismen fpu-context en monotonic-clock;
behoud van microkernel/procesisolatie vereist afzonderlijke CPU-status per
proces. Kernel beheert16uitgelijnde512byte FXSAVE-beelden; save bij CPL3-entry
vóórRust, restore pas na alleRust/CR3selectie direct vóórGPRpops/iret. Verse
PID/slot start met gewiste registerpayloads, vaste FCW/MXCSR en lege x87tags;
cleanup wist alles. CPUIDcontrole, EM/TSuit, MP/NEaan, OSFXSR/OSXMMEXCPTaan,
AMDFFXSRuit/teruggelezen; XSAVE/AVXblijven uit. Geen GUI/crypto/netpolicy verhuist
naar de vertrouwde kern. Kernel gebruikt zelf geen FP/SIMD en de disassembly
wordt gecontroleerd. Eén CPU; fysiekeAMDerrata/uitgebreideFPexcepties open.

Sys3 levert actuele100HzPITticks sinds boot, uitsluitend bij zes nulargumenten,
met saturatie. SDK kent ABI1.6; oudere profielen behouden hun eerdere ABI.
Geen wandklok/RTC/hervatcontract of realtimegarantie. De native-fpu-fixture heeft
echte syscall/timerwissels, drie schone levens, een echte unsupported-AVX#UD,
overlevend proces en slotreuse. Een afzonderlijke illegale-MXCSRproef onthult
TCG's ontbrekende#GP; dat falende bewijs blijft bewaard en is niet groen verklaard.
Hostartifacthelpers mogen geen symlink/hardlink buiten de checkout overschrijven.
Rootintegratie wordt apart native herhaald. Terugdraaien: opt-inmechanismen
revert en FP-afhankelijkeproductfuncties zichtbaar open houden.

## ADR-046 — Onderhouden TCP-library in een ring3app

**Datum:**30oktober2026. Ontwerpbesluit, implementatie/nativeacceptatie loopt.
Gebruik smoltcp=0.14.0(0BSD), minimale no_stdEthernet/IPv4/TCPfeatures,
integerReno, vaste1RX/1TX1514byte slots en maximaal twee TCP-sockets met8KiB
perbuffer. Geen std/alloc/libc/TUN, geen pakketparser in de kernel en geen
kunstmatig voortgezette tijd. TXslot blijft bezet tot echte brokercompletion.
Timeout/retransmissie gebruikt Sys3, naast hostlifetimebounds.

De actuele library vereist Rust1.91.1: alleen dit apppackage/alle lokale
Rust-afhankelijkheden worden daarmee gebouwd en als eigen ELF geladen.
Kernel/loader/oudere apps blijven gepind op1.90; geen gemengde rlibs of globale
defaultwissel. Matchende rust-lld en gelockte dependencyketen; buildscripts en
acht registrycrates zijn op herkomst/licentie/buildgedrag gecontroleerd.
0.12 is afgewezen wegens latere TCP-/IPv4-validatiefixes. Bekende upstreamissues
voor groteRXwindows en luisteraars blijven zichtbaar; de begrensdeclientproef
bewijst die algemene server/NASgevallen niet. De vaste proef-ISNseed is alleen
voor testdata; productie-entropy en TLS blijven releaseblokkerend open.
EOF, refused/reset, echt verloren eersteSYN/retransmissie en blackholetimeout
worden tegen een protocolconforme loopbacktegenpartij native getoetst.


## ADR-047 — Begrensde eigen heap en onderhouden gebruikersallocator

**Datum:** 30 oktober 2026. Implementatie in uitvoering; geen volledige geheugen-,
engine- of productacceptatie. Sys4 (opt-in ABI 1.7) groeit uitsluitend de eigen
contigue RW/NX-heap met verse nulgevulde pagina's. Bestaande totale procesquota
blijft 2048 frames inclusief programma en stack. Geen shrink, fysieke mapping,
uitvoerbaar heapgeheugen of delegatie. Fallible batchreservering en volledige
user- én kernelpaginatabelpreflight gaan vóór publicatie; fouten behouden oude
mappings en ruimen reserveringen op. Het eerste profiel blijft één CPU met IF=0.

De aparte userspace allocator gebruikt exact `linked_list_allocator` 0.10.6
(MIT/Apache-2.0, defaults uit), een expliciet exclusief arena en eigen begrensde
administratie. Geen hostfallback of stil automatisch groeien. `OwnedHeapPages`
bewijst geen veilige Rust-aliasing: de initialisatiecontrole blijft expliciet.
Onafhankelijke review, native nulvulling, quotum/OOM, NX, isolatie, hergebruik en
cleanup zijn afzonderlijke gates. De 128 MiB-OOM-proef verandert het bestaande
256 MiB-regressieprofiel of 8 GiB-productdoel niet. Terugdraaien: opt-in features
revert; allocator-/engineafhankelijke productcriteria blijven zichtbaar open.

## ADR-048 — Native display en invoer via minimale transportcapabilities

**Datum:** 30 oktober 2026. Contract goedgekeurd, implementatie in uitvoering.
De bestaande gevalideerde GOP-header/mapping en begrensde pixelhelper worden
losgemaakt van de G0-kernelrenderlus. De afzonderlijke `framebuffer`-feature
bevat alleen hardwaremetadata en supervisor-RW/NX/UC-transport. G0 blijft zijn
aparte IF=0-proef; Materiaal, fonts, documenten en invoerbeleid horen in ring 3.
Geen VRAM-frame wordt als gebruikers-RAM toegewezen of door heapcleanup vrijgegeven.

ABI 1.8 wordt opt-in gereserveerd: DISPLAY=4096 en INPUT=8192 zijn alleen door
het manifest uitgegeven, PID-/generatiegebonden en niet overdraagbaar/afleidbaar.
0x80/0x81 leveren openbare geometrie en een synchrone tileblit van maximaal
32×8 pixels (LE `0x00RRGGBB`); geen fysieke basis/stride of vsyncgarantie.
Alle usercopy, descriptorvelden, pixels en volledige doelrijen moeten vóór de
eerste hardwarewrite geldig zijn. Alleen de vertrouwde compositor krijgt display-
autoriteit; onbetrouwbare apps krijgen gecontroleerde oppervlakken/portalen.

0x82–0x84 vormen een begrensde PS/2-resetstatusmachine en ruwe bronlezing voor
uitsluitend de geïsoleerde invoerdienst. Werkelijke ACK/BAT/identiteit en
Set2→Set1-configuratie gaan vóór Ready; beide kanalen en foutstatus blijven
zichtbaar. Gevalideerde uitvoer gaat vóór FIFO-consumptie. Intrekking, dood,
stale tickets, tijdslimieten en generatie-uitputting mogen geen toegang of vals
Ready opleveren. Dit is geen algemene driver-/DMA-/IOMMU-garantie. Details en
uitvoeringsbewijs volgen in het transportcontract en native QMP-bedieningsproef.
Terugdraaien: opt-in transport revert, G0-regressie behouden en productcriteria open.

## ADR-052 — Afzonderlijke native CRB-apertureproef, geen product-TPM-beleid

**Datum:** 30 oktober 2026. `native-tpm-crb` combineert uitsluitend de bestaande
echte IPC-wachtproef met de bestaande safe TPM-transport-/commandocore.
De kernel bezit één vaste supervisor-RW/NX/UC-pagina op VA `0x3f180000`
voor QEMU locality0 `0xfed40000`; geen willekeurige fysieke mapping, DMA,
nieuwe gebruikersgrant of publieke syscall. De private niet-Copy-adapter
doet begrensde volatile-kopieën, expliciete compiler-/CPU-fences en sticky
quarantaine die geen nieuwe claim toestaat. Alleen de bootstrapfixture
codeert één read-only GetCapability(TPM_PT_FAMILY_INDICATOR); geen crypto,
NV-mutatie, sleutels of opslagpolicy in de kernel.

De bestaande scheduler levert echte PIT-ticks aan iedere begrensde poll.
Geen poll-countklok, drukke verborgen waitlus, automatische replay of
fallback. Readiness, exacte response, bufferwissen, Idle en relinquish-ACK
gaan vóór succes. Een aparte absent-case vereist weigering vóór enige
hardwarewrite. De volledige bestaande CPL3-wacht-/rechten-/cleanupcontrole
blijft gelden. Beide echte VM-runs zijn bij dit ontwerpcheckpoint nog open.

Dit is een eerste MMIO-integratie, geen kernelgrant voor productapps,
TPM-discovery, fysieke ondersteuning, vertrouwde boot, persistente
sleutel-lineage of onafhankelijk rollbackanker. PR #64 houdt zijn eigen
keyring/system-opslagwrites; dit experiment verandert die interface niet.
QEMU v10.2.1 `hw/tpm/tpm_crb.c` is als primaire implementatie gelezen:
https://github.com/qemu/qemu/blob/v10.2.1/hw/tpm/tpm_crb.c
Register/buffer/Idle/locality-completion passen bij het bestaande contract.
OSDev: *Memory mapped registers in C/C++*, W003198, revisie30402
(24 maart2026), is alleen aanvullende uitleg; geen normatieve TCG-controle.
Herzien vóór een productbroker, andere hardware of NV-commits. Rollback:
opt-in features/fixture verwijderen; legacy profielen blijven ongewijzigd.

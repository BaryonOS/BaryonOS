# Begrensde willekeurige bytes via de Key Service

TASK-KEYRANDOM, technische opdracht op basis `1b6a308`. De eerste stap is een
native integreerbaar protocol, dienstbeleid en client. Hostproeven/doelbuilds
zijn geen bewijs van native RANDOM, TLS, certificaatidentiteit of fysieke
entropy. De integrator blijft eigenaar van kernelmanifest, gedeelde lockfiles,
bouwprofielen en native TCP/TLS-callers. De volledige opdracht blijft leidend in
[FULL-TEST-RELEASE](../FULL-TEST-RELEASE.md).

## Autoriteit en protocol

KEY1-opcodes 1–9 behouden hun formaat en sleutelbevoegdheid. Nieuwe opcode10
(`RANDOM`) gebruikt uitsluitend een afzonderlijk toegekend RANDOM-ingress met
een eigen reply-endpoint en vastgelegde caller-PID/principalslot. Het domein en
de klok komen uit de vertrouwde adapter, nooit uit payload of callerclaims.
De twee bestaande sleutel-ingresses bieden RANDOM niet aan. Het nieuwe ingress
weigert alle sleutel-, decryptie-, herstel- en exportopcodes, ook wanneer een
app die rechtstreeks buiten de SDK om verzendt. Een geldige handtekening of
een ontvangencap verleent geen bijkomende bevoegdheid.

Een bericht heeft de bestaande 16-byte KEY1-header en exact16requestbodybytes:
`RND1` (4), versie1u16LE, gewenst aantalu16LE, acht nulbytes. Headerindex=0,
requestID>0, status=0. Count is1..224. Een geslaagd antwoord bevat hetzelfde
16-byte-prefix en exactcount bytes; 16header+16prefix+224=256. Alle staartbytes
zijn nul. Fouten bevatten geen body of willekeurige bytes. RequestID's zijn per
principalslot strikt oplopend en worden niet hergebruikt. Dit is een levende
instantiegrens, geen persistent antirollbackanker.

De SDK gebruikt standaard chunks van hoogstens128bytes. Hij bezit één exclusief
endpointpaar, een verwachte service-PID en één request tegelijk. Hij valideert
afzender, tag, ontbrekende gedelegeerde caps, header/opcode/index/requestID,
status, versie, lengte, gereserveerde bytes en padding vóór output. Annuleren
of onzekere IPC sluit de lease; geen stil herhalen of opnieuw verkrijgen van
bevoegdheden. Een reeds verzonden antwoord kan niet cryptografisch worden
teruggenomen: intrekking blokkeert toekomstige handelingen, niet eerder
rechtmatig ontvangen willekeurige bytes.

`Session::begin(count)` publiceert niets; iedere `poll()` doet hoogstens één
send- of receivepoging. Dezelfde request wordt uitsluitend na aantoonbaar
niet-gepubliceerde EAGAIN opnieuw aangeboden. Na enqueue volgt alleen receive.
De totale pollbegroting is1..1.000.000, zonder onbegrensde interne wachtlus.
`Outcome::Service(NotReady/Quota)` is een volledig gevalideerd, **afgerond
antwoord na publicatie**; dit is geen BeforeSend-toestand. Een expliciete nieuwe
aanvraag krijgt een nieuwID. Degraded sluit het paar. Andere statuscodes,
ongeldige antwoorden en transportonzekerheid zijn terminaal. Ook annulering
vóór enqueue sluit het paar; er is geen verborgen regrant of replay.

RequestID's beginnen bij1. Het endpointpaar wordt alleen bij een geslaagde
constructor overgedragen en moet exclusief zijn. Een nieuw paar mag de
requestgeschiedenis van een al gebruikt principalslot niet stilzwijgend
hergebruiken: bootstrap moet een verse dienst-/principallifecycle of een later
expliciet gecoördineerde monotone allocator leveren. Een verkeerde binding
faalt gesloten; de client bevat geen reset-opcode. Drop probeert beide handles
te sluiten, evenals expliciete close en terminale fouten; een mislukte close
wordt niet later op een mogelijk hergebruikte handlegeneratie herhaald.

## Toestand, grenzen en fouten

De bestaande1024-byte opstart-/gezondheidstest blijft vereist. Een RANDOM-slot
wordt pas bij zijn eerste geldige request gevoed met256nieuwe brokerbytes,
door dezelfde continue RCT/APT-tests. HKDF-SHA256 extraheert een32-byte seed met
het aparte RNG-v1/domein/monotone-epoch-label; geen masterkey of herstelgeheim
wordt daarvoor gebruikt. Elk van vier vaste slots heeft een afzonderlijke
ChaCha20Rng. Herstel van opgeslagen sleutels spoelt de RNG niet terug.

Na uiterlijk4096teruggegeven bytes of64requests volgt vóór nieuwe output een
nieuwe gezonde256-byte seed. Er is geen generator-, seed-, seek-, serialisatie-
of reseed-opcode voor apps. De epoch voorkomt identieke labelherhaling binnen
de instantie, ook bij een herhaald rawblok; hij bewijst geen bronkwaliteit of
koude rollbackbescherming. Het model veronderstelt werkelijke onvoorspelbare
bronbytes, niet alleen bytes die een gezondheidstest doorstaan.

Per slot zijn hoogstens8192bytes en64calls per100echte PIT-ticks toegestaan.
De adapter levert Sys3-ticks; loopaantallen of hosttijd worden niet gebruikt.
Achteruitgaande/overlopende tijd, foute payloads, replay en quota worden vóór
entropy/generatorverwerking geweigerd en laten output ongemoeid. Seedfalen
geeft Degraded, wist alle RNG-caches en geeft geen fallbackoutput. Bestaande
master, sleutelallocaties en intrekkingen blijven behouden. Een bevestigde
sleutel wordt door een RNG-fout niet stilzwijgend ingetrokken of opnieuw
uitgegeven. Voor Starting/Blocked geldt NotReady; Degraded blijft gesloten.

De expliciete raw-/seed-/outputbuffers gebruiken Zeroizing of een expliciete
wisoperatie. De RNG-feature wist core/key en vooraf berekende blokken bij
Drop. De hieronder beschreven KDF-patch behandelt daarnaast de expliciete
PRK-, HMAC-keypad-, vorige/uitvoerblok- en HMAC-hashbuffers en bewaarde
SHA2-hashstates. Die opsomming is de onderzochte grens; geen garantie over
diepe compressietemporaries, Rust-moves, compiler-, register-, SDK-, kernel-
of crashkopieën. Caller-owned output en reeds verleende gegevens blijven de
verantwoordelijkheid van de ontvanger.

## Bestaande startup-ABI en toekomstige native binding

De optionele service-descriptor wordt uitsluitend door de integrator geplaatst:
woord120=magic/versie `NOSRNG01`,128=RANDOM-RECV,136=reply-SEND,144=callerPID,
152=RANDOM-principalslot0..3. Nulmagic betekent geen descriptor. De huidige
keyring- en jointmanifesten gebruiken deze servicewoorden niet;112 blijft het
bestaande casewoord,160/176 de bestaande app/servicePID's. Bij onvolledige of
onbekende descriptor faalt initialisatie. Oude profielen zonder descriptor
roepen Sys3 niet aan; zonder beschikbare Sys3 geeft RANDOM geen output.
Deze afspraak reserveert startupplaatsen en is geen algemene bootstrap-ABI.
Er is nog geen door deze stap aangebrachte kernelbinding of native aftekening.
Woorden272/280/288 bevatten uitsluitend cumulatieve RANDOM-calls/bytes/seeds.
Zij zijn nog geen door de kernel gevalideerd native bewijs. De adapter valideert
de daadwerkelijke kernelafzender, tag, caps, volledige canonieke KEY1-envelope
en RANDOM-opcode vóór Sys3 of entropy. De vijf descriptorwoorden zijn geen
appbericht en moeten uit vertrouwde bootstrap komen. Exacte aliasing met de
bestaande descriptorhandles wordt geweigerd; identiteit en rechten van
verschillende handles blijven een kernel-/bootstrapverantwoordelijkheid.

## Afhankelijkheden en TCB per onderzochte eigenschap

De RNG is de onderhouden RustCrypto-implementatie `chacha20=0.10.2`, alias
`chacha20-rng`, uitsluitend features`rng,zeroize`, zonder default`cipher`.
`rand_core=0.10.1` levert de RNG-traits. Bestaande AEAD/legacyDRBG blijven op
chacha20.9.1. Nieuwe distributiechecksums:

| Crate | SHA256 | Licentie/MSRV/buildscript |
|---|---|---|
| chacha20.10.2 | 65c35e4b699c7e15ccbe7ee35c005e4fc0a278d22238a2857e6ce2dadeda1b06 | MIT/Apache2;1.85;geen |
| rand_core.10.1 | 63b8176103e19a2643978565ca18b50549f6101881c443590420e4dc998a3c69 | MIT/Apache2;1.85;geen |
| cpufeatures.3.1 | 5ca28b0ae3115b884660db4118d803791fd6756b6e88f39c0f3f7859060d7566 | MIT/Apache2;1.85;geen |
| zeroize1.9.1 | e13084392c5e4bc371903e2935a5eaeed24905a7511356b883835e18a78f6879 | Apache2/MIT;1.85;geen |
| cfg-if1.0.5 | 4e7648175b45a9a48536d676f68d918270699102aa8dab5496df06904c914600 | MIT/Apache2;1.32;geen |

Archieven/registrychecksums, manifests, licenties, RNG-/Drop-code en toepasselijke
RustSec-meldingen zijn gericht gelezen. De bekende chacha2019-counterfout is
verholpen vanaf.2.3; de rand_core2019-uitlijningsfout vanaf.4.2 en2021-seedfout
vanaf.6.2. Versies.10 vallen buiten die reeksen. .10.2 herstelt bovendien een
SSE4.1-intrinsic in de SSE2-backend; .10.0/.10.1 zijn ingetrokken. Er wordt geen
algemene auditdekking voor de nieuwe RNG-versie geclaimd. Rand_chacha.10 wordt
niet toegevoegd; dat zou hier een overbodige PPV/SIMD-boom meenemen.

De eigen kernel en Rust1.90 blijven behouden. Native compilatie vereist
`--cfg chacha20_backend="soft"` naast de bestaande chacha20_force_soft,
poly1305_force_soft en sha2_backend="soft". Een none-build zonder de nieuwe
portablecfg faalt expliciet. De integrator past de gedeelde builders/lockfiles
aan; de taak schrijft die niet. Geen std/libc/getrandom of hostfallback op het
x86_64-none-doel. De concrete lockgraaf en objectcode worden opnieuw getest.

De TCB voor onvoorspelbaarheid omvat bron/virtio-rng/QEMU-host in dit VM-profiel,
kernelbrokering/usercopy, gezondheidstests, HKDF en RNG-crates, dienstcode,
compiler en portablebouwkeuze. Voor scheiding tussen principals horen daar
capability/IPC-isolatie, de bootstrapbinding en de clientvalidatie bij. Die TCB
is groter dan alleen de kernel. Voor uitgifte kan de geautoriseerde ontvanger
bytes bewaren of via zijn bestaande netwerkrecht verzenden; deze API geeft
hem geen lees-, teken-, decryptie- of exportright op persoonlijke sleutels/data.

TLS-identiteitsondertekening, vertrouwde certificaattijd, hybride PQC-contract,
boot/updateauthenticiteit, persistente sleutelbescherming en fysieke RNG-
kwaliteitsmeting blijven afzonderlijke open eisen. RANDOM rechtvaardigt geen
klassieke/cloudfallback en geen tijd- of certificaatbypass.

## Uitgevoerd bewijs en eerstvolgende integratie

Het compacte bron-/commandobewijs staat in
[`keyrandom-host-none.json`](../../services/keyring/verification/keyrandom-host-none.json).
Op9exact gehashte bron-/manifestbestanden zijn57coretests (46bestaande en11RNG)
en11clienttests geslaagd. Daaronder: gepubliceerde ChaCha20-zero-keyvector,
1..224core- en1..128clientlengtes, principalbinding bij identieke bronbytes,
quota/tijd/replay vóór mutatie, gezonde verplichte reseeds, bronfalen zonder
master-/sleutelreset, weigering van iedere legacyop op RANDOM-ingress en
foutieve afzender/tag/caps/opcode/requestID/status/lengte/padding.
Transportproeven controleren deadline, annulering, onzekere send, geen resend
na enqueue, beide endpointcloses en gesloten onverwachte caps.

Host-Clippy voor beide crates en native-Clippy voor de dienst slagen met
`-D warnings`; rustfmt en `git diff --check` slagen. De native releasebuilds
voor dienst, bestaande testapp en clientbibliotheek slagen opRust1.90 voor
x86_64-unknown-none. Objdump van de dienst bevat geen FP/SIMD-registergebruik.
De gecontroleerde native dependencyboom bevat geen std/libc/getrandom. Een
none-build zonder de nieuwe portablecfg is daadwerkelijk geweigerd(exit101).

De builds gebruiken een aparte exacte bronkopie met eigen Cargo.lock onder
`/tmp/nieuw-os-keyrandom-integration` en target onder
`/tmp/nieuw-os-keyrandom-target`; gedeelde root-/SDK-locks zijn ongemoeid.
Dit is host-/crosscompilebewijs en **geen uitgevoerde native RANDOM-call**.
De integrator moet clientworkspace/locks/buildflags, vertrouwde descriptor,
RANDOM-only appgrant, klokprofiel en onafhankelijke runtimeproeven toevoegen.
Vereist blijven daadwerkelijke uitgifte en weigering bij intrekking, verkeerd
ingress, ontbrekende klok/entropy, reseedfalen en quota, plus legacyregressies
op dezelfde geïntegreerde image. Deze stap tekent geen releasecriterium af.

Gericht geraadpleegde primaire bronnen (geen algemene auditclaim):
[RustCrypto chacha20.10.2](https://crates.io/crates/chacha20/0.10.2),
[rand_core.10.1](https://crates.io/crates/rand_core/0.10.1),
[RUSTSEC-2019-0029](https://rustsec.org/advisories/RUSTSEC-2019-0029.html),
[RUSTSEC-2019-0035](https://rustsec.org/advisories/RUSTSEC-2019-0035.html),
[RUSTSEC-2021-0023](https://rustsec.org/advisories/RUSTSEC-2021-0023.html).

## Correctie van de KDF-geheugenhygiëne

Review van `fc2e2a2` vond een expliciete leemte: de gebruikte HKDF0.12/HMAC0.12
bewaarde een entropy-afgeleide PRK/HMAC-state zonder wis-Drop. Dit was een
beheerd geheim en wordt niet als alleen een compilerkopie afgedaan. Een
ongedocumenteerde upgrade naar HKDF0.13 zou onvoldoende zijn: ook die bron
liet PRK, expandblokken en verschillende HMAC-buffers ongewist.

Daarom gebruiken **alle productie-KDF-paden** de gerichte, reviewbare
[HKDF0.13/HMAC0.13-hygiënevariant](../../services/keyring/vendor/README.md)
met SHA2.11.1/zeroize. Geen eigen KDF, combiner, hash of MAC; de patch verandert
uitsluitend bufferlevensduur, wissen en feature-/traitkoppelingen.
Het betreft bestaande startup- en reseedafleiding van de legacy-DRBG,
`derive_key(master,domain,index)` voor legacy/recordencryptie,
`recovery_key` voor de bestaande recoveryfixture, `persistent::wrapping_key`
onder het externe gebruikergeheim en `random_at` voor RANDOM-seeds.
Alle labels, outputs, keyhandles en snapshot-/wireversies blijven gelijk.
HKDF0.12/SHA2.10 blijven uitsluitend dev-orakels voor bytevergelijking;
de native productieboom bevat deze oude KDF/hashpaden niet.

Exacte nieuwe registrypins naast de twee lokale sourcevarianten zijn
SHA2.11.1, digest.11.3, hybrid-array.4.15 en zeroize1.9.1. Nieuwe actieve
registryversies komen overeen met de eerder beoordeelde TLS-graaf; HKDF.13
en de optionele KDF.1-trait zijn aanvullend primair gecontroleerd. Er zijn
geen nieuwe buildscripts, std/libc/getrandom of native hostfallback.
De hoogste nieuwe MSRV is1.88; kernel/toolchain1.90 blijft behouden.

De actuele bron-/test-/nonebuildregistratie staat in
[`hkdf-hygiene/host-none.json`](../../services/keyring/verification/hkdf-hygiene/host-none.json).
Het eerdere RANDOM-rapport blijft historisch bewijs van de bron vóór deze
correctie. De aanvullende proeven controleren de echte drop-markerketen,
RFC5869/4231, oude/nieuwe bytepariteit inclusief8160bytes en foutgrens,
HMAC/SimpleHmac/resetvarianten, legacy-DRBGreseed en RANDOM-principal/reseed-
afleiding. Geen native RANDOM- of volledige wisaftekening volgt hieruit.

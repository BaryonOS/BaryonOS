# Actuele projectstatus

## NEXT AGENT HANDOFF — GUI IPC_WAIT-checkpoint, native bewijs nog open — 2 november 2026

Branch `codex/window-idle-wait-20261102`, geïsoleerd vanaf
`b1b292aadf244dacf029b7307aebdc6103756e0d`. De andere integrator houdt merges;
geen wijziging van main. Score **37 -> 37/100**; U01/U02 blijven open.

Materiaal, Rekenmachine, compositor en window-init parkeren alleen bij bewezen
rust op bestaande receive-capabilities, via checked now+1 en de bestaande
IPC_WAIT. Pending input/reply/damage, onvolledig huidig frame en herstel/stop
blokkeren. App-PEEK hercontroleert uiterlijk volgende tick; de compositor
behoudt bestaande tweepits-inputcadans. ABI 1.8/1.9 legacy, 1.10/1.11 wacht;
1.0-1.7 niet als werkende display/input-appconfiguratie geaccepteerd.
Geen kernel-, ABI-, capability-, opslag-, timeout- of dependencywijziging.

Gerichte hosttests: Materiaal35, Rekenmachine24, window-session31, inclusief
voltooid oud frame gevolgd door gedeeltelijk nieuw frame en echte herstelstadia.
Init blokkeert rust ook bij recovery.is_some(), naast recovering. Component-ELF-builds,
host/native-Clippy en formatterchecks zijn uitgevoerd; compacte bron-/loghashes
en exacte controlegrenzen: `verification/gui-idle-20261102/checkpoint.json`.
Twee read-only kruisreviews vonden geen concrete nieuwe false-idle/deadlinefout;
dit is geen onafhankelijke menselijke securityaudit.

Bekende blocker: b1b292a faalt werkelijk in de volledige native crash/herstelroute.
De lokale Windows/WHPX-probe toont initial/editor/dirty/calculated/moved exact,
vervolgens slot4/vector6, PRESENTATION_RETIRED1 en slot3/pid4/code1/step18,
raw exit35 APP_EXIT. Geen adapter-, cleanup- of geforceerde-terminatiefout.
De centrale chat heeft deze freeze afgekeurd en werkt aan expliciete herstelfasen.
Daarom geen CPU-A/B of acceptatie op deze bekende rode basis.

De centrale opvolger is nu `337f2f29f1833ef4a8d2cd640b09419f85dbb83b`,
source-v3 `3919403542c923bc86467db3184345159a5ce6881567a0e11baa85f2d39f1995`.
Die gebruikt expliciete herstelstadia; de korte cloud-handshake slaagde,
volledige native acceptatie volgt. Deze merge behoudt de afzonderlijke rustlogica.
Eerstvolgend: controleer recovery/idle samen en bouw baseline/idle in gescheiden mappen.
Voer daarna de ongewijzigde volledige native probe met echte Windows/WHPX uit,
vervolgens exact-beeld-gebonden CPU-A/B. Documentatie/contract:
`implementation/GUI-IDLE-WAIT.md`. Bron-/native-/CPU-bewijs blijven gescheiden.
Geen hostinstellingen, passthrough of fysieke schijven; gebruik disposable images.

## Centrale herstelcheckpoint — expliciete herstelstaten; korte native handshake eerst — 2 november 2026

Branch `codex/mature-native-session-20261102`, concept-PR #122 op #121.
Laatste functionele commit `fdc2861`, source-v3 `3919403542c923bc86467db3184345159a5ce6881567a0e11baa85f2d39f1995`.
Score **37/100**: N01 geaccepteerd; U01/U02 blijven open. Bron na checkpoint
bevriezen. De nieuwe generatie-/stagefix is host/review/crossbuild-groen,
maar moet nog werkelijk native slagen.

Vorige b1 is afgekeurd, niet meer als eindfreeze gebruiken. M4/IPCwait/N01
slagen, vensterboot110,24/editor185,47/dirty266,52/calculated308,78/moved314,23s
exact; compud2 en PRESENTATION_RETIRED1 daarna Matstep18, naturalraw35 na314,717s.
CI reproduceert hetzelfde in150,412s. Oorzaak: oude mask31/pause0 veroorzaakte
Rebind8 vóór PauseACK/restart. Resultaat/rawgzip/CI-hashes zijn bewaard onder
verification/mature-session-20261102/window-b1-* en window-ci-b1-failed.json.

Nu geïntegreerd: echte Recovery helper met AwaitPause→AwaitFreshReady→
AwaitRebindAck; mask3 bij begin/install. ACK7 alleen in Pause, ACK8 alleen in
RebindAck, READY uitsluitend in FreshReady met huidige PID/gen/epochs. Alleen
na echte beidepauseACKs, geslaagde0x53 en gevalideerde verseBoot gaat de helper
naar FreshReady. Alle drieREADY en exacteinstalledGroup zijn nodig voor8;
stage verandert vóór beide sends en voorkomt dubbele8. Zes daadwerkelijke
helperregressies plus twee gerichte read-only agentreviews vinden geen blocker.
Bestaande typed retirement, beideownerpreflight, per-handleBadHandlecleanup,
alle model-/identity-/epoch-/pixelguards blijven. Geen timeout/POLLverruiming.

Root voegde --recovery-handshake-only toe aan run_window_session: wacht op
werkelijk eerstecommit, echteF12, kernelretirement/gen2 en Rec1/nieuweCompPID;
QMPquit0. Deze diagnose is altijd smoke_only=true/native_gate=false en bewijst
geen frames, werkbehoud, opslag of cleanup. Eerst die goedkope native route,
daarnade volledige1800s-route op exactdezelfdebron met18hostoracles/13nativebeelden.

Verder geïntegreerd: launcher hashcheckt nu ook alle3 Noto-notices; root22
synthetischePowerShell en20packagertests slagen. Accelerationhelperfc506... is
onveranderd. ABI1.11-parkingkeyring/volume en complete volatile snapshotwipe blijven.
RANDOM foundation slaagt in echte48267b6CI; vroegere fout is niet gereproduceerd,
hostcounterexample64RPC/100tick blijft bekend en diagnose beschikbaar.

Geautoriseerde lokalechat01a19c13-c83d-7781-8f61-de0541ac6d95 bereidt Windows-
QMPadapter buiten bron/OneDrive voor (originele runner, geen guards versoepeld).
B1 geen meetreeks meer. Die chat bezit uitsluitend een afzonderlijke idlebranch
vanaf b1: compositor/window-init en Materiaal/Calc quiescent IPC_WAIT-onderzoek,
geen kernel/ABI/rechten/timeoutwijzigingen. Cloudroot neemt dat niet stil over;
nieuwefreeze eerst doorgeven, diff en apartebrongebondenA/B-evidence vragen.
Oud344-PCidlebewijs blijft apart gerapporteerd (geen nieuwebronclaim).

Eerstvolgend: activate omgeving, build native-window-session-probe in
build/window-final/target/window-final; kortehandshake --timeout120, daarna
volledige --timeout1800 met explicieteSeaBIOS/VGA-ROMs. Geen andereVM tegelijk.
Na echtepass U01 pas+4; normale Windowsbundle exactfreeze opnieuw bouwen,
19payloads/8ELFs/fontlicenses/PSchecks bevestigen, lokaleWHPX/TCGoverdracht.
Geen hostinstellingen/fysieke schijven/passthrough. Andere integrator houdtmerges.

## Vorige afgeronde checkpoint — native sessiepreview


**Functionele implementatie / Windows-pakket:** `344dc0afde893784503217ac676e154b13662507`,
branch `codex/session-desktop-vm-20261102`, concept-PR #121. Bronbasis
`e08fb0e48c1bb0781e98357f588c2f73b6ae8500`. De latere opmaakcommit `cfed2db9a39396e48a7bc758eb2c533f3af3f22c`
wikkelt uitsluitend één kernelarray-element om volgens rustfmt. De volledig
lokaal geteste344-bron heeft source-v3
`05b047819f1c1cddce6868034ab2208d8a35a95d9f974860afb8bbc53eeb8658`;
de opgemaakte bron heeft
`0bfd8385bbe08d3ace384977848fb42f6a65f6261b28be19805eccd90fce14f0`
en een eigen geslaagde achtstappen-cloud-QEMU-herhaling. Geen verdere
functionele wijzigingen. Beide rootfmt- en diffchecks slagen.
De bestaande andere integrator blijft eigenaar van merges; deze PR is niet gemerged.

**Geleverd:** opt-in `native-system-session-preview` met gebruikerskeuze,
herkenbaar aanmeldformulier, expliciete openbare VM-gast, bureaublad/taakbalk/
programmasearch en echte Materiaal. Eigen no_std-toestanden binden authantwoorden
maar verlenen zelf geen rechten. `UnavailableBroker` weigert alle persoonlijke
credentials; geen gedeeld demowachtwoord of nepbiometrie. Owned secrets worden
met de reeds gepinde zeroize gewist. Geen nieuwe externe crate of capability.

Windows Auto/Whpx/Tcg gebruikt een echte sequentiële QMP-initprobe met één
10 s-deadline en binaire LF-invoer. De lokale chat reproduceerde en valideerde
de Windows-HMP-racefix. Source-v3 sorteert expliciet case-sensitive padcomponenten,
waardoor gelijke Windows/Linuxbronnen dezelfde hash leveren; controles blijven intact.
De compositor verwerkt max8 berichten per beurt; de interactieve app max4
tegeltransacties per input/savepass. Geen raw framebuffer of gedeelde userpointer.

**Eindbewijs cloud:** alle 8 native stappen op `344dc0a` slagen: accounts →
persoonlijke synthetische testinvoer → geweigerd/gewist → accountkeuze →
gastlogin → desktop → zoek 'mat' → Materiaal. Iedere app-pixel wordt vergeleken,
zonder interne masks; QEMU raw exit0. TCG/QEMU 10.0.13/q35-10.0/1CPU/256 MiB:
118,79s totaal; Materiaal16,08s versus21,63s op `51d883e` en33,94s op `7648a8f`.
Dit zijn losse metingen in deze cloudomgeving, geen productlatentiegarantie.
Ook de bestaande native Materiaal normal/save/list/reopen-regressie slaagt
met onafhankelijke diskoracle en cleanup (rawexit33, runnerexit0;163,82s).
46 frontendhosttests,27 SDK-apphosttests,15 compositorhosttests,72 Pythonchecks,
39 synthetische PowerShellchecks, Clippy/fmt en scalar-none/objectscan slagen.
De oude verifier telde9 archives terwijl de bestaande SDKzeroize10 oplevert;
hersteld naar de exacte tien namen, met behoud van alle objects-/broncontroles.

**Lokale hardware:** geautoriseerde chat `01a19c13-c83d-7781-8f61-de0541ac6d95`
rapporteert Windows 11 25H2, Ryzen 9 9950X3D,64 GiB. De bytegelijke QMPhelper
slaagt 6/6 op PS 5.1/PS 7.6. De 51d-nativebaseline slaagt 6/6 (3WHPX/3TCG).
De eindmeting op `344dc0a` slaagt 6/6 (3 WHPX/3 TCG), 48 appbeelden exact.
WHPX gebruikerskeuze 6,625 s gemiddeld (6,481–6,823), gast→desktop 4,070 s
(3,932–4,231). De echte PS 5.1-starter kiest Auto→WHPX, toont SDL en sluit
met launcherexit 0. Er blijft geen QEMU-proces achter. Ruwe Windowsrapporten
staan in de lokale testmap; aangeleverde hashes en grenzen zijn vastgelegd
in local-pc-reported.json. Geen hostinstellingen gewijzigd.
Geen netwerk, gedeelde hostmappen, fysieke disks of USB/PCI/GPU-doorvoer.

**Grenzen en hervatten:** de gast gebruikt hetzelfde synthetische appdomein met
vooraf verleende VM-rechten. Het accountformulier is geen beveiligde sessiegrens.
Echte profiel-/verifierpersistentie, auth-/sessiebroker, keyring-unlock, limitering,
compositor/input-lock en revocation blijven nodig; FIDO2/vingerafdruk zijn niet
gekoppeld. Ook dirty-regiontransport, vensterbeheer, algemene appstart en
OS-brede toegankelijkheid blijven open. Score **34 → 34**, geen FR-/fysieke/
menselijke aftekening. [Contract](implementation/SESSION-AND-DESKTOP.md),
[hardware](implementation/VM-HARDWARE-AND-PERFORMANCE.md),
[eindbewijs/reproductie](verification/session-desktop-20261102/README.md).

GitHub Actions op cfed2db meldt startup_failure zonder gestarte jobs; de
oorzaak is niet vastgesteld. Dit is geen groene remote-CI en de PR blijft concept.

Eerstvolgend: onderzoek de CI-startfout en bouw daarna een echte aparte
auth-/sessiedienst vóór beveiligd aanmelden.
Gebruik bestaande tools en behoud andere dirty checkouts/branches.

## NEXT AGENT HANDOFF — PR- en issueafhandeling na integratie 1 en 2 — 30 oktober 2026

**Mandaat:** Simon heeft de Claude Code-sessie in de desktopapp op 30 oktober
(~21:10 CET) aangewezen als enige operationele integrator voor alle open PR's
en issues, inclusief mergen en sluiten. Merges naar main lopen via die
integrator; tussentijdse losse merges (#71 en #74 door Simon zelf) dwingen een
nieuwe up-to-date-CI-run. Vastgelegd in [WORK-QUEUE.md](WORK-QUEUE.md).

**Broncommits:** #96 (integratie 1) gesquash-merged als `c0b7147`.
Integratie 2 (#113) heeft implementatiebron `ed63ed259746`. De bronnen en
oplossingen staan in [pr-integration-20261030.json](verification/pr-integration-20261030.json)
en [pr-integration-2-20261030.json](verification/pr-integration-2-20261030.json).

**Afgehandeld:**
- #68, #72–#81, #83, #86, #88, #89, #92, #94 en #97–#107 zijn als overgenomen
  gesloten. Per PR is gecontroleerd dat elke commit na de vastgelegde bron,
  exclusief main, aantoonbaar in main zit.
- #111 is gesloten. Het was een parallelle, oudere subset van #96 en #113; het
  bewijs en archief ervan zijn behouden. #112 is gesloten als automatisch
  duplicaat van #113.
- Reviewfixes in #96:
  - net-stack heeft een expliciete discard-route voor een nooit aangeboden,
    ingehouden TX-frame, met regressie (rood en daarna groen);
  - het dependencyregister is aangevuld;
  - in het werkregister zijn de spaties hersteld.
- Integratiefix in #113: de Materiaal-chaintest volgt het nieuwe
  semantische buffercontract. Dat was de oorzaak van de rode CI op #107.

**Open punten:**
- #109: app-management-reader naar `artifact-store-core`.
- #110: exacte boot-1 NAS-replay native activeren. De replay-gastcode uit #93
  is bewust niet actief, omdat de native normal/cold-gate anders vast faalt.
- #39–#43: productmilestones; hun criteria zijn niet aangetoond.

**Bewijs en score:** 34/100 → 34/100. Er zijn geen nieuwe native-, FR-,
fysieke of menselijke acceptaties. Hosttests, freestanding-builds en AI-review
zijn geen productpunten. Niet lokaal uitgevoerd: de freestanding-build van
`app-package-core` (lokale rustc gaf SIGILL); CI is de gate.

**Eerstvolgend commando:** `gh pr list --state open`. Beoordeel nieuwe PR's
tegen main met `docs/verification/pr-integration-2-20261030.json` als
bronbasis. Een volgende integratie start bij #109 of #110, of bij een
DEV1-herproef op vastgelegde bron en slot.

## Vorige handoff — centrale documentatie en bronbehoud (#96) — 30 oktober 2026

**Integratieopdracht:** Simon heeft in deze sessie het oplossen, squashen en mergen
van issues en pull requests opgedragen. Dit actuele mandaat geldt voor deze
integratie; de historische queue en bronclaims blijven bewaard.

**Bronbasis:** main `d791a342cc3bdb107245692f2f542d88d5fda7f1`;
management en laatste overgenomen queue: PR #75 op
`c26f57b60d4eaffbb57c0eb5114fa00bd7254eb6`.
[Bronindex en volledige historie](history/integration-2026-10-30/README.md)
scheiden deze basis van oudere branchhandoffs. De gecombineerde implementatiebron is
[`3fedd03e`](https://github.com/simonvanieperen/nieuw-os/commit/3fedd03eecff62d6bffae038ec959c56734e2137).
Gerichte SDK- en netwerkregressies zijn groen; volledige kandidaat-CI
en de native NAS normal/cold-gate blijven vereist vóór merge.

**Eigenaars en claims:** de nieuwste vastgelegde [werkqueue](WORK-QUEUE.md)
blijft de primaire bron voor taken, fileclaims en acceptatie.
Assistent `5d5f8513-2d31-41af-96ce-81c5ef56ef80` is de enige queuewriter;
teamleider 89f houdt technische samenhang, PROJECT-STATE en scorebestanden.
De queueovername en e8-vrijgave/7f-ontvangst zijn vastgelegd; bestaande
inhoudseigenaren en opdrachten blijven behouden. Stilte of een timeout
geeft geen schrijfgebied vrij. De queuebron bewaart ook de afzonderlijke
mergepauze rond #85; de managerhandoff beschrijft 7f als operationele
integrator. Dit bronverschil is [expliciet geregistreerd](history/integration-2026-10-30/README.md#overdracht-en-bronverschil);
dit checkpoint verzint geen later stop- of hervattingsbesluit.

**Product en bewijs:** 34/100 blijft 34/100; criteria, gewichten, statussen
en geaccepteerde evidence blijven gelijk aan de main-bron.
Alle 62 FR-gates, 100 cold-starts, 24 uur, fysieke hardware en menselijke
acceptatie blijven open. Het [DEV1-resultaat](https://github.com/simonvanieperen/nieuw-os/blob/557b85e26372cf3809c0760d40b430ec5ef7cc6b/docs/verification/ordinary-dev1/reference-ac4f038e86/result.json)
op bron `ca45a7dabe6a040921de03bdc2fad3e5045a75ba` is mislukt
(`passed=false`, raw exit 0), vóór het Materiaalmanifest.
De [gecorrigeerde buildfreeze](https://github.com/simonvanieperen/nieuw-os/blob/557b85e26372cf3809c0760d40b430ec5ef7cc6b/docs/verification/ordinary-dev1/isolated-build-1/build-freeze.json)
heeft `vm_gates_started=false`: geen nieuwe guestherproef uitgevoerd.
Bron: Nieuw OS / S.W.J.B.M. van Ieperen, 30 oktober 2026;
[volledige metadata en bewijsgrens](history/integration-2026-10-30/README.md#dev1-bronrecord).
UI0818 is beperkt native deelbewijs. Store/READ, issuer,
launch, TLS en NAS-lifecycle blijven productafhankelijkheden.
B05 heeft in de managementbron geen bevestigde auditor; deze integratie
is geen menselijke audit, externe opdracht of releaseaftekening.
Het [cold77-hostverslag](verification/teamleader-cold77-host.json)
houdt sourcefix, native herhaling en mergegoedkeuring apart.

**App-management:** pakketvalidatie, SDK-identiteit, catalogus, protocol en
transactiebewaking zijn samengevoegd. De gecombineerde SDK-hostsuite heeft
26 tests; een testbackend bewijst geen duurzame journal- of native launch-keten.
De volledige integratiechecks en de native NAS normal/cold-gate moeten op de
feitelijke integratiecommit slagen. PR #93 blijft buiten deze integratie:
de export en startupbinding van de vorige NAS-boot ontbreken nog.

**Eerstvolgend:** controleer live head/base, concrete reviewpunten en
vereiste checks voor de feitelijke integratiebron. Laat het progressscript
README en het voortgangsoverzicht uit die bron genereren, met de vaste
rubric en zonder oude assessments als nieuwe testuitvoering te presenteren.
Behoud lopende domeintaken; voer een DEV1-herproef alleen op de daarvoor
vastgelegde bron en bestaande slotafspraak uit. De historische volgorde,
queued retries en oude groene runs zijn geen actuele integratiegate.

**Historie:** de volledige main-DHCP-handoff, PR #75-managementgeschiedenis,
beide PR #65-teksten plus hun 0e441a9-voorganger, en de oudere PR #73/#79-
handoffs staan bytegetrouw in het [gelinkte archief](history/integration-2026-10-30/README.md).
Daar blijven oorspronkelijke testbronnen, onzekerheden en assessments
behouden. Alleen dit bovencheckpoint is het actuele hervatpunt.

# AUDIT-12 — appplatform, ABI en SDK

**Bronmetadata:** primaire repositorybron: *Nieuw OS — “Add OS specialist
agent profiles”*, auteur/organisatie S.W.J.B.M. van Ieperen, 3 november 2026,
commit `beafdedfc188154fe7285848fbd411e651013f54`
([onveranderlijke snapshot](https://github.com/simonvanieperen/nieuw-os/tree/beafdedfc188154fe7285848fbd411e651013f54)).
Alle lokale `pad:regel`-citaten hieronder verwijzen naar die snapshot.
Externe ABI-/platformclaims zijn niet onafhankelijk geverifieerd en blijven
`ONBEKEND` of toetsvragen.

Datum: 3 november 2026 (Europe/Amsterdam). Opdracht: “Beoordeel ABI, SDK,
appmanifesten, pakketten, sandboxing, rechten, backwards compatibility en
ontwikkelaarservaring.”

## Resultaat en afbakening

De onderzochte bron heeft een bruikbare, begrensde native SDK-basis,
kernelgecontroleerde capabilities en een afzonderlijke pakketverifier.
Een downloadbaar pakket installeren, duurzaam registreren, met beperkte
rechten starten, bijwerken en verwijderen vormt echter nog geen aangesloten
native productketen. De belangrijkste blokkades zijn de persistente
trust-/artifact-/catalogusdiensten en de dynamische imageautorisatie.
Dit is een brononderzoek met historische deelbewijzen, geen uitvoeringstest,
algemene productrijpheidsverklaring of veiligheidsgoedkeuring.

| Herkomst | Vaststelling |
|---|---|
| Repository | `simonvanieperen/nieuw-os`; lokaal ingestelde origin `https://github.com/simonvanieperen/nieuw-os.git` |
| Exact onderzochte bron | `beafdedfc188154fe7285848fbd411e651013f54` |
| Gevraagde bronbranch | `copilot/add-os-specialist-agents`; de lokale `refs/remotes/origin/copilot/add-os-specialist-agents` wijst exact naar deze commit; niet via netwerk ververst |
| Beginbranch / HEAD | `simonvanieperen-urban-funicular` / `69809c2cb53dfb3bd77abe5352892810fba6e82d` — afwijkend van de gevraagde bronbranch |
| Werkplek | Eigen worktree, niet de hoofdcheckout; lokaal pad niet gepubliceerd |
| Beginboom | `git status --short`: leeg; broncommit aanwezig als Git-commit |
| Bronselectie | Toegestane `git switch --detach beafdedfc188154fe7285848fbd411e651013f54`; daarna opnieuw schone boom en exact HEAD gecontroleerd |
| Werkwijze | Gewone chat; `.github/agents/os-app-platform-reviewer.agent.md:10-32` expliciet gelezen, niet technisch als custom agent geladen; geen subagents |
| Schrijfgebied | Uitsluitend dit rapport; geen productcode, status, roadmap, score, commit, push of PR |

Gelezen instructies: `AGENTS.md`, `docs/AGENTS.md`,
`kernel/AGENTS.md`, `services/AGENTS.md`, `tests/AGENTS.md`, het genoemde
profiel en de OS-dev-skill. Verder README, het bovenste checkpoint,
privacycontract revisie 4, architectuur-/beveiligingscontext, relevante
ADR's, release-eisen en onderstaande bronbestanden. Het profiel vraagt ook
handlers, achtergrondtaken, diagnostiek en uninstall; die vallen hieronder
binnen dezelfde begrensde audit. Geen externe dienst, download, installatie,
credential, echte gebruikersdata of device gebruikt. CodeRabbit niet
uitgevoerd: code naar een externe dienst sturen valt buiten deze opdracht.
Actuele GitHub-PR-/CI-status is niet opgevraagd; lokale Git-referenties zijn
geen bewijs van actuele serverstatus.

## Bewijslegenda en bron/artefact/historie

Orchestratorlegenda: **E0** ontbrekend/ambitie; **E1** ontwerp;
**E2** hostprototype/simulatie/hosttest; **E3** native bron;
**E4** native positief; **E5** native negatief;
**E6** herhaalde CI/releasegate; **E7** fysieke hardware;
**E8** onafhankelijke menselijke audit/formele evaluatie.
Een hogere code voor een deelproef promoveert niet de gehele feature.
Deze audit levert statische observaties; geen nieuw E4–E8-bewijs.

| Onderdeel | Huidige bron / grens | Historisch bewijs, uitsluitend op eigen bron |
|---|---|---|
| SDK / syscallgrens | E3: `sdk/runtime/src/native.rs:1-38`; host kan de native module niet gebruiken: `sdk/runtime/src/lib.rs:23-24` | `docs/verification/parallel-g01/README.md:3-6,23-42,66-76`: bron `89b352f8…`, hosttests en crosscompile, expliciet geen VM (E2 plus compilatie) |
| Pakketverificatie / ELF | `crates/app-package-core/src/lib.rs:503-583`: canonieke verificatie, beide signatures, digest en gedeelde ELF-parser; geen processtart | `docs/verification/app-package-elf-policy/README.md:8-19,27-45`: bron `81b45969…`, vijf hosttests en target-librarybuild; geen native installatie (E2) |
| Bronrotatie / intrekking | In-memory policy, geen duurzame autoriteit; `docs/DECISIONS.md:252-270` | `docs/verification/app-package-source-policy/README.md:8-21,28-47`: bron `be830818…`, acht hosttests; geen installer/recovery (E2) |
| Processtart / sandbox | E3: `kernel/src/m2/m3/supervisor.rs:286-389`, gecontroleerde launch/delegatie/resourcechecks | `docs/verification/k05-startup-slots/README.md:18-36`: bron `56b576b1…`, vijf VM-gevallen normal/crash/denied/parent-exit/quota (historisch E4/E5) |
| Native apps / herstel | Geen algemene pakketstarter afleiden uit vaste apps | `docs/verification/mature-session-20261102/U01-ACCEPTANCE.md:3-26,45-56,67-70`: bron `337f2f29…`, echte apps, herstel en CI; bredere appinstallatie open |
| Klembordportaal | Afgebakend voorbeeld van taakautoriteit, niet algemene portaaldekking | `docs/verification/native-clipboard-20261102/README.md:13-23,25-56,58-85`: verschillende expliciete bron-/CI-bindings, native positief/negatief en herhaalde gate (historisch E4/E5/E6), 192-byte tekstgrens |

De historische rapporten zijn gelezen; hun volledige gecomprimeerde logs,
imagehashes en externe CI-artifacts zijn hier niet opnieuw gevalideerd.
Een beschreven historische groene gate is dus geen gate op `beafdedf`.
Geen E7/E8 afgeleid; AI-review is geen E8.

## Bevindingen en prioriteit

P1 = blokkeert een bruikbare derdenappketen of stabiele contractacceptatie;
P2 = daaropvolgende API-/ontwikkelaarsgap. Dit zijn platformbevindingen en
integratierisico's, geen gereproduceerde exploiteerbare kwetsbaarheden.

| ID | Prioriteit | Bevinding, effect en exacte bron |
|---|---|---|
| AP12-01 | P1 | **Pakket → native launch ontbreekt.** De verifier levert bytes/laadplan, geen startrecht (`crates/app-package-core/src/lib.rs:572-583`). De bestaande spawn resolveert een kernelbekende image en controleert `check_launch` (`kernel/src/m2/m3/supervisor.rs:163-171,286-309`). De system-route levert slechts een vaste worker, of geen spawnimage voor Materiaal (`kernel/src/m2/m3/system.rs:72-83`). Dynamische launcher, one-use `AuthorizedImage` en kernel-copy/preflight zijn expliciet open (`docs/implementation/APP-PACKAGES.md:217-224`). Afhankelijk: AP12-02/03 en een exact bytes→autorisatie→procescontract; FR-A11 blijft open. |
| AP12-02 | P1 | **Install/update/remove is geen duurzame transactie.** `InstallCatalog` noemt zichzelf in-memory en heeft geen autoritatieve persistente bron (`sdk/app-management/src/catalog.rs:3-12`). `restart` behoudt entries uitsluitend als model; `reconcile` vertrouwt op callerautoriteit (`sdk/app-management/src/catalog.rs:264-309`). `ArtifactWriter` legt atomiciteit/digestcontrole op aan een toekomstige adapter; de guard is geen journal (`sdk/app-management/src/store.rs:40-85`). Crash-/schijffoutbestendigheid en koude recovery van een echte installatie zijn niet bewezen. Afhankelijk: persistente store/catalogus, epochautoriteit, daarna FR-A11/A12. |
| AP12-03 | P1 | **Uitgeverstrust en bronconsent zijn nog geen native dienst.** Goede scheiding: geselecteerde bronankers en beide signatures, niet een pakketgekozen sleutel (`crates/app-package-core/src/lib.rs:504-582`). Maar duurzame bronstatus, consent en capabilitycontrole blijven vereist (`docs/DECISIONS.md:257-268`). De artifact-readadapter moet caller/epoch bij iedere read controleren; de fake controleert alleen epoch/locator (`sdk/app-management/src/store.rs:8-15`). Een digest of parsebare identiteit is dus geen issuer-/installautoriteit. Afhankelijk: gescheiden apptrustdienst, niet stil B02/B03-ankers hergebruiken. |
| AP12-04 | P1 | **ABI-versienummer is nog geen compleet feature-/compatibiliteitscontract.** SDK accepteert common ABI 1.0–1.11, maar IPC-wait heeft terecht een afzonderlijke expliciete profielcheck (`sdk/runtime/src/lib.rs:26-34,81-92`). Kernel rapporteert een minor op basis van compile-time features (`kernel/src/m2/m3.rs:1726-1751`); `monotonic_ticks`/heap beschrijven afzonderlijke opt-in beschikbaarheid (`sdk/runtime/src/native.rs:40-63`). Pakketten hebben alleen een ABI-interval (`crates/app-package-core/src/lib.rs:93-108,517-519`). Ook verschillen de encoderingen: kernel/SDK major hoog 32 bits versus pakket major hoog 16 bits. Nog geen aangesloten installeradapter aangetroffen die dit vertaalt. Risico: een toekomstige launcher behandelt een common ABI-match als bewijs voor alle benodigde diensten. Vereist: gecontroleerde conversie, feature-/serviceversies en expliciete unsupported-uitkomst; geen aangetoonde actuele bypass. |
| AP12-05 | P1 | **Publieke appbootstrap lekt nog fixturekennis als integratievereiste.** Het leesvoorbeeld vereist vaste startupvelden op `0x40010000` en is expliciet niet zelfstandig startbaar door simpel een ELF te vervangen (`sdk/examples/catalog-read/README.md:28-47`). Materiaal leest veel numerieke offsets in zijn eigen adapter (`sdk/examples/materiaal/src/bootstrap.rs:29-88`) en hangt aan service-/UI-crates in de repository (`sdk/examples/materiaal/Cargo.toml:15-28`). Dat is nuttige interne integratie, geen onafhankelijk versioned appcontract. Afhankelijk: bootstrapdescriptor, stabiele protocoltypes en appstarter; internals niet stil als publieke SDK vastzetten. |
| AP12-06 | P1 | **Rechtenmodel is aanwezig; algemene grant-/uninstalllevenscyclus niet aangesloten.** Kernel controleert callerendpoint vóór usercopy/queue-operatie (`kernel/src/m2/m3/m4.rs:245-307`); installatiecatalogus bevat geen capabilities (`sdk/app-management/src/catalog.rs:3-12`). Launchdecoder koppelt gedeclareerde rechten aan werkelijk aangehechte caps en heeft afzonderlijke epochvalidatie (`sdk/app-management/src/protocol.rs:235-302`). Zero-grant/stale tests bestaan als hostbron (`sdk/app-management/src/protocol.rs:990-1057`). Native consent, intrekken na update/remove en gebruikersdatabehoud ontbreken nog expliciet (`docs/implementation/APP-PACKAGES.md:158-177`). Alleen een record verwijderen is geen capabilityrevocatie of datawipe. Afhankelijk: AP12-01/02/03, appidentiteit en policybroker; FR-A12/A13. |
| AP12-07 | P2 | **Resourcegrenzen bestaan, maar derdenappbudget en foutdiagnostiek zijn nog geen productcontract.** Engine heeft vaste proces-/slot-/node-/endpoint-/queuequota (`crates/capability-core/src/lib.rs:12-16,53-73`); spawn preflight controleert beschikbare resources (`kernel/src/m2/m3/supervisor.rs:314-332`). ELF-parser weigert onbekende segmentsoorten, W+X, overlap en te veel pagina's (`crates/kernel-core/src/user.rs:81-124`). Dit is geen dependencyresolver, dynamic linker of schaalbaar appbudget. De circa 200 KiB ML-DSA-stack en 2 MiB fixture-imagegrens vragen native servicebudgettering (`docs/implementation/APP-PACKAGES.md:247-259`; `docs/DECISIONS.md:245-248`). CPU-/I/O-/achtergrondbudgetten niet als volledige sandboxdekking geclaimd. |
| AP12-08 | P2 | **Algemene handler-, taak- en notificatie-API ontbreekt in het onderzochte SDK-oppervlak.** Geen module voor MIME/defaults/protocolhandlers, achtergrondregistratie of notificaties aangetroffen in de geïnventariseerde SDK-bron-/README-route. FR-A13 noemt het formaatregister, open/saveportaal en defaults nog open (`docs/FULL-TEST-RELEASE.md:188-190`); de minimale formaat-/compatibiliteitsmatrix is een eis, geen implementatie (`docs/FULL-TEST-RELEASE-CONTEXT.md:86-105`). Native klembord is een echt maar smal historisch deelresultaat, niet bewijs van open/save/drag/print- of algemene appintegratie. Afhankelijk: versieerbare portals en sessie-/policybroker. |
| AP12-09 | P2 | **SDK-documentatie loopt achter op bron; compatibiliteitsbelofte is niet reproduceerbaar afgedekt.** Runtime-README zegt uitsluitend ABI 1.0–1.3 en verwijst naar privacyrevisie 3 (`sdk/runtime/README.md:26-35,89-94`); bron accepteert 1.11 met een expliciete test voor 0–11 (`sdk/runtime/src/lib.rs:286-295`). README zegt ook dat geen blokinterface wordt aangeboden, terwijl `pub mod block` en native blockimports bestaan (`sdk/runtime/src/lib.rs:9`; `sdk/runtime/src/native.rs:7-13`). Dit verruimt niet vanzelf rechten, maar maakt het publieke API-verhaal inconsistent. Versie `0.1.0`, `publish=false` (`sdk/runtime/Cargo.toml:1-8`) is geen onderhouds-/deprecationbeleid. Nog geen hier aangetroffen gate met oude appbinaries tegen nieuw image/serviceversies. |
| AP12-10 | P2 | **Ontwikkelaarsroute is repositorygebonden, geen complete externe SDK-ervaring.** Er is een heldere offline native voorbeeldbuilder met eigen linkerflags, maar deze start niet (`sdk/examples/catalog-read/build.py:1-28`). De SDK-workspace sluit meerdere libraries/apps uit; één workspace-opdracht is dus geen volledige platformcontrole (`sdk/Cargo.toml:1-4`). Pakket-CI draait hosttests/Clippy/fmt en freestanding libraries, geen install/launch/update/remove (`.github/workflows/app-management-quality.yml:16-44`). Native appdebugger/profiler en een stabiele privacybewuste diagnose-API niet aangetroffen in de begrensde SDK-inventaris. Bestaande envelope-Debug vermijdt payload en Drop wist buffers (`sdk/runtime/src/message.rs:60-72,138-152`); dat is geen bewijs dat alle applogs lekvrij zijn. |

AP12-09 is een concrete documentatie-/bronafwijking. De overige punten zijn
voornamelijk bekende maar nog blokkerende product-/contractgaps, niet een
reden om reeds aanwezige kernelisolatie of native apps als “alleen simulatie”
weg te zetten. De ABI-encodingverschillen uit AP12-04 zijn op zichzelf geen
bug: zij worden een integratierisico wanneer code de representaties verwart.

## Dreigingsmodel en te behouden invarianten

Bescherm: andere apps, documenten, sleutels, beperkte grants, bronankers,
catalogusintegriteit en beschikbaarheid. Onbetrouwbaar: app/pakket,
dependency, gekozen downloadbron, IPC-payload en gecrashte oude instantie.
Vertrouwd per grens: kernel voor PID/handle/mapping; policybroker voor consent
en scope; apptrustdienst voor uitgever/rotatie; store voor immutable bytes en
duurzame commits. De laatste diensten zijn deels nog contract, geen bewezen
product-TCB. Firmware/CPU horen bij de onderliggende aannames.

| Grens / risico | Invariant en huidig aanknopingspunt |
|---|---|
| Downloadbron → verifier | Bronselectie staat buiten pakket; beide signatures plus imagehash; ondertekening geeft geen autoriteit. `docs/DECISIONS.md:217-243`. |
| Verifier → loader | Exact geverifieerde bytes moeten de uitgevoerde bytes blijven; voorkom verwisseling tussen verificatie en gebruik. Toekomstig one-use imagecontract nodig; `docs/implementation/APP-PACKAGES.md:217-224`. |
| App → dienst | Gebruik kernelafzender en echte capabilities, geen payload-PID of “verified”-boolean. `sdk/app-management/src/protocol.rs:107-144`; `kernel/src/m2/m3/m4.rs:254-281`. |
| Metadata → toestemming | `PackageIdentity`, app-ID, scope-ID, hash en Rust-handletypes zijn geen nieuwe rechten. Installeer standaard zonder grants; test ook een bevoegde tussenpersoon die anders zijn eigen rechten namens een app inzet. `sdk/app-management/src/protocol.rs:990-1003`. |
| Grant → volgende handeling | Controleer actuele scope/generatie; update/remove moet oude autoriteit expliciet afhandelen. Reeds rechtmatig gelezen kopieën zijn niet terug te halen. `docs/LOCAL-PRIVACY-CONTRACT.md:98-108`; `docs/implementation/APP-PACKAGES.md:158-177`. |
| Commit → antwoord/herstart | Onbekende uitkomst is geen succes en geen automatische replay; catalogus moet gesloten blijven tot autoritatief herstel. `sdk/app-management/src/catalog.rs:255-309`; `sdk/app-management/src/store.rs:215-255`. |
| App → gegevens plus netwerk | Gecombineerd lees-/netwerkrecht kan toegestane inhoud laten vertrekken; een sandbox is geen algemene informatieflowgarantie. `docs/LOCAL-PRIVACY-CONTRACT.md:104-108`. |
| Diagnostiek → logs | Geen payloads/keys in standaardlog; SDK-Debug is lokaal begrensd, niet een audit van alle diagnostics. `sdk/runtime/src/message.rs:60-72,138-152`. |

## Voorstel: minimale stabiele surface

Dit is een voorstel (E1), geen nieuwe architectuurbeslissing of implementatie.
Vries eerst een kleine appgrens in; houd fixture-offsets en dienstinternals
erbuiten. Verander de bestaande security floor niet.

1. **ABI-baseline:** architectuur, register-/foutconventie, Message320,
   featurebeschikbaarheid, protocolversie en startupdescriptor met lengte en
   versie; gecontroleerde omzetting tussen kernel- en pakketversie.
2. **Autoriteit:** typed handles als hulp, kernel als enige rechter; expliciete
   attenuatie/intrekking en taakgebonden document-, netwerk-, invoer- en
   surfacegrants. Common ABI-match verleent geen dienst of capability.
3. **Pakket/lifecycle:** behoud `NOSAPP01` en de gescheiden brontrust; voeg geen
   algemene installscripts toe. Specificeer immutable artifact, cataloguscommit,
   geautoriseerde image, update/removepolicy, appdata en crashherstel als één
   aangesloten contract. Bronintrekking blokkeert toekomstige pakketten, niet
   stil reeds geïnstalleerde apps (`docs/DECISIONS.md:257-268`).
4. **Dagelijkse app-API:** versieerbare open/save-/formaat-/surface-/input- en
   beperkte deelportalen; vervolgens notificatie en achtergrondtaak met quota,
   cancel en lifecycle. Niet via directe toegang tot interne servicecrates.
5. **Onderhoud/DX:** bouw-/pakket-/startroute vanuit een schone SDK-consumer,
   actuele voorbeelden, foutregister, compatibiliteitsvenster, deprecation en
   migratiebeleid; privacybewuste lokale debug/profiling zonder algemene
   beheer-/bestandsrechten.

Statische eigen ELF-apps zijn de aangetroffen basis. Geen Linux-/Windows-/
macOS-binaire compatibiliteit, libc/POSIX-dekking, universele gamestore of
creative-suitecompatibiliteit afgeleid. COMP01–03 vragen expliciete gekozen
combinaties en bewijskaders (`docs/FULL-TEST-RELEASE-CONTEXT.md:94-105`).

## Voorgestelde conformancetests en afhankelijkheden

Alle onderstaande tests zijn **niet uitgevoerd** en voegen geen bewijs toe.
Hergebruik bestaande helpers/CI; bouw geen tweede installer of testbackend
als vervanging voor ontbrekende native componenten.

| Test-ID | Acceptatie / negatief geval | Vereist niveau en afhankelijkheid |
|---|---|---|
| C12-ABI | Bestaande 1.0–1.11 common-ABI fixtures; onbekende major/minor; exacte 32+32→16+16-conversie zonder truncatie; feature ontbreekt ondanks common match | E2 als eerste, daarna E4/E5 per werkelijk ondersteund image; AP12-04/09 |
| C12-COMPAT | Bewaar oude app-ELFs/SDK-protocollen; run op nieuw image, nieuwe app op oudere ondersteunde dienst; unsupported/foutwoord/nieuwe optionele feature expliciet afhandelen | E4/E5 en herhaalde E6; geen volledige oude-versiegarantie uit huidige enumtests |
| C12-PACK | Geldig pakket; één slechte signature, hash-/framing-/padding-/ABI-fout; signer-valid W+X/overlap/interpreter/dynamic-segment geweigerd vóór procesbouw | Bestaande E2 behouden, nieuwe E4/E5 op native verifier; AP12-01/03/07 |
| C12-LAUNCH | Installeer uit twee gekozen bronnen, start exact geverifieerde image zonder grants; verwissel image/store-epoch/caller tussen verify en launch | E4/E5 met werkelijk store-/kernelpad; AP12-01/02/03 |
| C12-GRANT | Deny/revoke; handle uit andere app; stale epoch; ongeoorloofde extra grant; helper/tussendienst; gecombineerde document/netwerkgrants; privacyuitleg uit geeft niets extra | E4/E5 met werkelijke broker; AP12-06/08 |
| C12-TRUST | Duurzame bronkeuze/keyrotatie/revoke, stale mutation, cold boot; eerder geïnstalleerde app blijft startbaar volgens ADR-056 | E4/E5 plus onafhankelijke bytes/recovery; AP12-02/03 |
| C12-LIFE | Update/downgrade/bronwissel, verwijder draaiende app, behoud/wis eigen appdata versus gebruikersdocumenten, stale autoriteit na remove; volle disk/I/O-fout en kill op iedere commitgrens | E4/E5 en koude VM met bytes-oracle; AP12-01/02/06 |
| C12-QUOTA | Geheugen/endpoints/queues/CPU-/I/O-budget uitputten; parser-/signer-stackbudget; andere apps blijven bruikbaar; schone cleanup | E4/E5; historische K05 niet als actuele productbudgettest herlabelen |
| C12-PORTAL | Default/MIME/handler na uninstall; kwaadaardige handler krijgt alleen gekozen object; notificatie-/achtergrondcancel en focus/clipboard/drag-grens | E4/E5 plus bruikbare bediening; AP12-08 |
| C12-DX | Schone consumer bouwt, verpakt, start en debugt zonder private startupoffsets of extra algemene rechten; synthetische secrets/payload ontbreken in fout-/profilinglogs | E2 + E4/E5; daarna E6 op vaste bron/artifacts; AP12-05/09/10 |

Uitvoeringsvolgorde: eerst contracten AP12-03/04/05, dan duurzame store en
catalogus AP12-02, vervolgens autorisatie/launcher AP12-01 en rechtenlifecycle
AP12-06. Werkelijke portalen, quota en ontwikkelaarservaring volgen op die
aangesloten grenzen; documentatiedrift kan afzonderlijk worden hersteld.
Geen fixes aangebracht of gevraagd.

## Uitgevoerde controles, afwijkingen en niet uitgevoerd

Alle bronverwijzingen gelden voor de vaste auditcommit. Read-only
inventarisatie gebeurde met `git ls-files`, `rg`, `nl -ba … | sed -n …`,
`head` en bestandslezingen; deze zijn geen tests. Relevante exacte
controlecommando's en uitkomsten:

| Commando | Exit / uitkomst |
|---|---|
| `pwd && git branch --show-current && git rev-parse HEAD && git status --short && git worktree list --porcelain && git cat-file -t beafdedfc188154fe7285848fbd411e651013f54 && git remote -v` | 0; juiste eigen werkplek, afwijkende beginbranch/HEAD, schone boom, commit aanwezig; alleen metadata van worktrees gelezen, geen hoofdcheckoutbestanden |
| `git switch --detach beafdedfc188154fe7285848fbd411e651013f54 && git rev-parse HEAD && git status --short && git ls-files '*AGENTS.md' 'docs/AGENTS*' '.github/agents/os-app-platform-reviewer.agent.md' && git remote get-url origin` | 0; exacte detached bron, schone boom, instructies gevonden |
| `git show-ref --verify refs/remotes/origin/copilot/add-os-specialist-agents` | 0; lokale ref exact `beafdedf…`; geen fetch |
| `git status --porcelain=v1 --untracked-files=all; git rev-parse HEAD` | 0; vóór schrijven leeg en exact audit-HEAD |
| `test ! -e docs/audits/subagents/12-app-platform-sdk.md` | 0 vóór aanmaak; geen bestaand rapport overschreven |
| `git diff --check` | 0 na schrijven; alleen whitespacecontrole, geen OS-test |
| `git status --short --untracked-files=all` | Alleen dit rapport als nieuw bestand; HEAD blijft exact auditcommit |
| `sha256sum docs/audits/subagents/12-app-platform-sdk.md` | 0; digest in overdracht, niet zelfreferentieel in dit rapport |

De bestandsnaaminventaris
`git ls-files 'sdk/*/src/*' | grep -E 'portal|notif|mime|background|debug|profil'`
gaf exit 1 (geen bestandsnaammatches); aanvullende inhoud- en README-lezing is
hierboven gebruikt.

De inhoudszoekopdracht
`rg -n 'app.package|app_package|app.management|app_management|AuthorizedImage' kernel services tools -g '*.{rs,py}'`
gaf geen matches; dit is een begrensde zoekwaarneming, geen bewijs dat iedere
mogelijke implementatie ontbreekt.

Twee verkennende paden bestonden niet: `sdk/runtime/ABI.md` (view:
pad ontbreekt) en `sdk/runtime/src/abi.rs` (`wc`/`nl`: ontbreekt).
De ABI is vervolgens daadwerkelijk onderzocht in `sdk/runtime/src/lib.rs`,
`native.rs`, kernelversierapportage en package-core; de ontbrekende
bestandsnamen zijn geen zelfstandig defect. Sommige gecombineerde leesbatches
eindigden met exit 0 terwijl een eerdere `wc`/`nl`-stap een fout meldde;
deze worden niet als geslaagde bestandslezing gerekend. Grote inventarisuitvoer
was afgekapt en is door gerichte bronlezingen vervangen.

**Niet uitgevoerd:** Cargo/build/Clippy/fmt, Python-testharnassen, preflight,
QEMU, native negatieve proeven, runtimeprofiling, fuzzing, externe CodeRabbit,
CI-opvraag/-herhaling, releasevalidatie, fysieke of menselijke tests.
Ook geen repository-output genererende progress-/statusopdracht.
De enige nieuwe repository-output is dit auditdocument. Een inline
read-only Python-controle controleerde alle 62 pad-/regelverwijzingen op
bestaand bestand en geldige regelgrenzen (exit 0); begin-/eindregels zijn
vervolgens gelezen en de verifier-/privacycitaten inhoudelijk gecorrigeerd.
Dit controleert citeerbaarheid, geen runtimecorrectheid.

Aannames: het bedoelde product is de zelfstandige x86_64/UEFI-kernel met
freestanding Rust-apps; de expliciete auditopdracht begrenst de bredere actieve
releaseopdracht. “Niet aangetroffen” betreft de genoemde zoek-/leesdekking,
niet een uitputtend bewijs over alle bestanden. Geen runtime-aanvalsproef of
afwezigheidsbewijs voor kwetsbaarheden. Resterende blockers: ontbreken van de
aangesloten installer-/trust-/store-/launchketen en actuele exacte-bron
native conformanceresultaten. Het auditonderzoek zelf was niet geblokkeerd.

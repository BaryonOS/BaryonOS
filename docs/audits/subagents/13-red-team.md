# AUDIT-13 - defensief dreigingsmodel en aanvalsbomen

Datum: 3 november 2026 (Europe/Amsterdam). Alleen repositoryonderzoek.
Opdracht: "Maak een defensief dreigingsmodel en aanvalsbomen. Gebruik geen
echte data, productiecredentials of destructieve tests."

## Resultaat en afbakening

Dit rapport levert een dreigingsmodel, acht aanvalsbomen en veilige
regressievoorstellen. **Geen exploit uitgevoerd en geen nieuwe exploiteerbare
kwetsbaarheid vastgesteld.** De concrete bronbevindingen hieronder zijn
beveiligingsgrenzen en integratiebeperkingen, geen acht nieuwe bugs.
Een aanvalstak is een hypothese totdat haar voorwaarden en uitkomst apart
zijn aangetoond. Prioriteit betreft vervolgonderzoek, niet een CVSS-score.

Uitgevoerd als gewone chat die
`.github/agents/os-red-team.agent.md:10-25` expliciet heeft gelezen; het
profiel is niet technisch als customagent geladen. De repositoryskill
`os-dev` is wel geladen. Geen andere agents gestart.

| Herkomstcontrole | Waargenomen |
|---|---|
| Repository | `simonvanieperen/nieuw-os`, lokale bron; geen remote opgevraagd |
| Eigen werkboom | `/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-laughing-carnival` |
| Beginbranch | `simonvanieperen-laughing-carnival`, dus niet de gevraagde bronbranch |
| Begin-HEAD | `69809c2cb53dfb3bd77abe5352892810fba6e82d` |
| Beginstatus | `git status --short` en `--porcelain=v1`: leeg |
| Vereiste commit | Lokaal aanwezig als commitobject |
| Lokale bronref | `refs/remotes/origin/copilot/add-os-specialist-agents` wees naar de vereiste commit; geen uitspraak over de live server |
| Audit-HEAD | `beafdedfc188154fe7285848fbd411e651013f54`, expliciet detached na toegestane switch |
| Hoofdcheckout | Niet gebruikt voor bestandslezingen of wijzigingen; alleen als metadata zichtbaar in `git worktree list` |
| Schrijfgebied | Uitsluitend dit rapport; geen productcode, status, roadmap of score |

De stopvoorwaarden waren niet geraakt: de eigen boom was schoon, niet de
hoofdcheckout en de exacte commit was aanwezig. Geen resultaten van de
begin-HEAD als bronbewijs gecombineerd met de audit-HEAD.

Gelezen instructies: `AGENTS.md`, `docs/AGENTS.md`, eigen profiel en
`boot/AGENTS.md`, `kernel/AGENTS.md`, `drivers/AGENTS.md`,
`services/AGENTS.md`, `tests/AGENTS.md`, `ui/AGENTS.md`. README, actueel
checkpoint, privacycontract en volledige releaseopdracht zijn gelezen.
De specifieke auditopdracht sluit hun gebruikelijke schrijf-/publicatiestappen
voor deze opdracht uit.

## Bewijslegenda en bronkritiek

| Niveau | Betekenis in deze audit |
|---|---|
| E0 | Ontbrekend of ambitie; afwezigheid alleen claimen binnen onderzocht bereik |
| E1 | Ontwerp of contract, geen implementatiebewijs |
| E2 | Hostprototype, simulatie of hosttest; testbron zonder uitvoering is geen geslaagde test |
| E3 | Native bron aanwezig; geen bewijs van daadwerkelijke uitvoering |
| E4 | Native positief uitvoeringsbewijs, gebonden aan exacte bron en artefact |
| E5 | Native negatief uitvoeringsbewijs, eveneens bron- en artefactgebonden |
| E6 | Herhaalde CI-/releasegate voor de betreffende configuratie |
| E7 | Fysieke hardwareproef |
| E8 | Onafhankelijke menselijke audit of formele evaluatie met afgebakende aannames |

Deze audit produceert bronanalyse en ontwerp, geen nieuw E4-E8-bewijs.
E4/E5 zijn verschillende dekkingen; een positief scenario vervangt geen
negatief scenario. Een hoog niveau voor een kleine component is geen niveau
voor het gehele OS. AI-analyse is geen E8.

Historie en artefacten blijven apart:

- `docs/verification/B03-ACCEPTANCE.md:9-21` beschrijft historische native
  ondertekende-updateproeven met TEST-ankers. Bron/configuratie/artefactbinding
  staan op regels 25-43; de historische commando's op regels 45-50 zijn
  **niet hier uitgevoerd**. Geen actuele loader-, kernel- of imagehash gemeten.
- `docs/PROJECT-STATE.md:13-16` waarschuwt dat NAS-checkpoints historisch zijn.
  Regels 103-119 onderscheiden eerdere bron/CI van actuele acceptatie.
  Hun gate-uitkomsten zijn niet opnieuw gecontroleerd of naar deze HEAD gepromoveerd.
- Het oude M2-M4-contract noemt zichzelf nog ongeimplementeerd
  (`docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md:1-5`).
  Dat is geen actuele afwezigheidsclaim: de hier gelezen syscall- en
  capabilitybron bestaat daadwerkelijk.
- `services/nas/README.md:17-25` en
  `services/nas/native-probe/CONTRACT.md:3-7` zijn lokale pakketbeschrijvingen,
  geen bewijs dat alle latere native NAS-deelproeven ontbreken.
  Wel onderbouwen ze dat deze componenten niet zelfstandig een E2EE-netwerkproduct zijn.

## Assets, aanvallers en vertrouwensgrenzen

Een vertrouwensgrens is een overgang waar identiteit, invoer en bevoegdheid
opnieuw moeten worden gecontroleerd. De TCB is de verzameling componenten
waarvan de betreffende garantie afhangt; niet alleen de kernel
(`docs/SECURITY-AND-QUANTUM.md:19-38`).

| Asset / invariant | Aanvaller en toegang | Gewenste impact | Grens / vertrouwde basis |
|---|---|---|---|
| Kernel- en procesgeheugen, captable | Onbetrouwbare ring-3-app met eigen syscalls/IPC | Andere processen lezen, schrijven of besturen | CPU/paging, entrycode, usercopy, capability-engine |
| Documenten, namen, inhoudssleutels | Lokale gebruiker of bezitter van een gekopieerde testdisk | Inhoud lezen of oude rechten herstellen | Object-/sleuteldienst, sessie, persistente authenticatie en actualiteit |
| App- en netwerkautoriteit | App of gecompromitteerde tussenservice | Bevoegde dienst als plaatsvervanger misbruiken, ongeoorloofde export | Kernelafzender, grants, broker, NIC-publicatie |
| NAS-namespace, quota en intrekking | Onbevoegde client, kwaadaardige NAS/admin/provider | Impersonatie, stale upload, rollback, beschikbaarheidsverlies | Geauthenticeerde ingress, core-epochs, durable storage, client-E2EE |
| Boot-/update-/pakketintegriteit | Partij die distributiebytes of synthetische ESP/slotdisk kan wijzigen | Eigen of oude code laten uitvoeren | Firmware -> loader -> handtekeningbeleid -> image -> appstarter |
| Invoer, focus en klembord | App met eigen venster of opgeslagen oude berichten | Toestemming nabootsen, geheim of selectie lezen | Inputrouter, vertrouwde compositor/sessiedienst, eenmalig event |
| Beschikbaarheid en herstel | Geldige maar agressieve client of falende virtuele tegenpartij | Queues/quota vullen, herstel blokkeren | Begrensde resources, scheduler, backpressure, storagefouten |
| AI-context en lokale logs | Onbetrouwbaar document, model of helper | Onbedoelde lokale actie of externe verzending | Taakbroker, modelparser, gescheiden downloader, logbeleid |

De normale app met bewust verleend leesrecht EN internetrecht kan toegestane
inhoud verzenden. Dit is geen op zichzelf nieuwe privilege-escalatie
(`docs/LOCAL-PRIVACY-CONTRACT.md:106-108`). Intrekken kan al ontvangen kopieen
niet terughalen. Het wissen van eigen NAS-kopieen door de NAS-admin is
toegestaan gedrag, geen kwetsbaarheid. Metadata en beschikbaarheid worden
niet automatisch door inhoudsencryptie beschermd
(`docs/implementation/ENCRYPTED-OBJECT-STORAGE.md:62-84`).

## Concrete bronbevindingen: controles en beperkingen

Zekerheid hieronder gaat over wat de aangehaalde bron zegt/doet, niet over
het slagen van een aanval. Geen van deze rijen is een uitgevoerde exploit.

| ID | Bronbevinding | Exacte vindplaats | Niveau en beperking |
|---|---|---|---|
| B1 | Imageverificatie is featuregebonden: zonder `signed-boot` leest de loader de ESP-kernel; met de feature uitsluitend de signed store. Defaultfeatures zijn leeg. | `boot/loader/Cargo.toml:7-15`; `boot/loader/src/main.rs:518-538` | E3, hoge bronzekerheid. Geen onvoorwaardelijke secure-bootclaim; geen uitgebracht profiel onderzocht. |
| B2 | De loader leest een firmwareversievloer; ontbrekend geeft nul. Verhogen gebeurt met NV/BS-attributes. Firmwareverificatie van de loader en een aanvaller met schrijfrecht op OVMF_VARS zijn expliciete open grenzen. | `boot/loader/src/signed.rs:123-163`; `boot/loader/src/signed.rs:293-341`; `docs/DECISIONS.md:1261-1264` | E3 plus E1-grens, hoge bronzekerheid. Geen bewezen remote manier om de variabele te wijzigen; afhankelijk van aanvallertoegang. |
| B3 | Het documentvolume gebruikt FNV voor beschadiging, niet encryptie/authenticatie/rollback. Key-state-codec vereist een onafhankelijk minimum en duurzame integratie; een blob op dezelfde terugrolbare disk is geen anker. | `services/volume/src/lib.rs:1-11`; `services/volume/src/service.rs:449-465`; `services/keyring/src/persistent.rs:1-6` | E3, hoge bronzekerheid. Geen claim dat alle opslag onversleuteld is: de aparte Key Service heeft wel crypto. Productgebruikersbescherming is afzonderlijk ontwerp: `docs/implementation/PROFILES-AND-USERS.md:140-150`. |
| B4 | Native TLS-module bevat prerequisites, geen transportdienst; huidige klokken passeren de certificaattijdgrens niet. Socketcore mist een productie-issuer/native fence; raw kernel-NIC-broker legt destination policy buiten de kernel. | `services/net-tls/src/lib.rs:1-9`; `services/net-tls/src/time.rs:9-41`; `services/net-broker/src/lib.rs:3-7`; `services/net-broker/src/lib.rs:36-53`; `kernel/src/m2/m3/network.rs:1-18` | E3 voor aanwezige bron; E0 voor de daar expliciet ontbrekende koppelingen. Geen gelijkstelling van IP/poort aan cryptografische peeridentiteit. |
| B5 | NAS-core vertrouwt een extern geauthenticeerde caller; `Principal`-bytes zijn zelf geen identiteitsbewijs. Hij controleert eigenaar/epoch en persisteert revoke voor purge. | `services/nas/native-core/src/lib.rs:1-8`; `services/nas/native-core/src/lib.rs:29-37`; `services/nas/native-core/src/lib.rs:470-515`; `services/nas/native-core/src/lib.rs:742-778` | E3. Publieke constructie van `Principal` is niet op zichzelf een wire-authenticatiebug; bereikbaarheid vanuit een onbetrouwbare ingress is niet aangetoond. |
| B6 | CPU-usercopy is begrensd en owner-/RW-gecontroleerd; serialized commit steunt op een CPU, geen concurrerende unmap/shared-memory/DMA-API. | `crates/kernel-core/src/user.rs:188-238`; `kernel/src/m2/m3/m4.rs:185-235`; `docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md:94-98` | E3 plus E1-aannames. Geen algemene SMP- of kwaadwillige-DMA-garantie. |
| B7 | Delegatie controleert rechten en diepte; IPC zet afzender-PID zelf, controleert transferrechten en reserveert voor commit. Receive weigert ingetrokken transfers. | `crates/capability-core/src/lib.rs:1312-1334`; `crates/capability-core/src/lib.rs:1384-1465` | E3, verdedigende controles; geen geslaagde regressierun in deze audit. |
| B8 | Klembord controleert focus, afzender, eenmalige sequence en geldigheid opnieuw bij reply. Pakketverificatie verleent uitdrukkelijk geen object-/netwerkautoriteit. | `sdk/examples/window-session/src/clipboard.rs:138-227`; `crates/app-package-core/src/lib.rs:4-12` | E3. Geen bewijs voor accountauthenticatie of alle appstarters. |

DMA blijft een apart platformrisico: ADR-026 omvat de protocolconforme
QEMU-backend in de TCB en sluit IOMMU-containment uit
(`docs/DECISIONS.md:636-644`). Dat is geen nieuw gevonden driverlek.
Als lokale leerbron is OSDev, *IOMMU*, W005001, revisie 30116
(3 maart 2026) gelezen in
`.agents/skills/os-dev/references/corpus/wiki-14.md:17949-17965`.
Dit is een aanvullende communitybron, niet een hardwareconformiteitsbewijs.
De in ADR-026 genoemde OASIS/Intel-specificaties zijn niet extern opgehaald
of opnieuw geverifieerd.

## Aanvalsbomen

Notatie: OR = een tak kan volstaan; AND = alle voorwaarden nodig.
Alle bladeren zijn defensieve onderzoekshypothesen, tenzij expliciet B-bronfeit.
Er wordt geen aanvalsinvoer, exploitcode of actieve aanval geleverd.

### T1 - ongeautoriseerde kernel-/procesautoriteit

Doel: een onbetrouwbare app passeert de kernelgrens.

```text
OR
  AND app heeft syscalltoegang
      pointer-/lengte-/page-ownercontrole faalt op een grensgeval
      falende operatie wijzigt geheugen of autoriteit
  AND app heeft endpointtoegang
      stale/afgeleide/queued capability passeert geldigheidscontrole
      ontvanger gebruikt rechten buiten oorspronkelijke delegatie
  AND toekomstige concurrency verandert mappings tijdens copy
      huidige single-threadserialisatie wordt niet vervangen
```

Broncontroles B6/B7 maken dit geen geconstateerde bypass.
Impact potentieel hoog; **P1**, voordat nieuwe mapping-/thread-API's worden
toegevoegd. Dependencies: pagingownership, entryserialisatie, capgeneraties.
Veilige toets V1: twee synthetische processen, grensoverschrijdende
320-byte-uitvoer naar een read-only/tweede niet-eigen pagina, oude handles na
exit/revoke, queued transfer na revoke. Oracle: fout, output en autoriteit
ongewijzigd, ander proces blijft bruikbaar. De bron bevat al een
failure-stateassertie (`kernel/src/m2/m3/m4.rs:317-327`) en outputassertie
(regels 376-382); die alleen tellen niet als nieuwe uitvoering.

### T2 - verkeerde code of downgrade in boot-/pakketketen

```text
OR
  AND distributie gebruikt onbedoeld unsigned loaderprofiel (B1)
      tegenstander kan synthetische ESP-inhoud vervangen
  AND loader/floor valt buiten de beschermde vertrouwensbasis (B2)
      aanvaller kan oude geldige code plus oude vloer aanbieden
  AND pakket is geldig onder een uitgeverssleutel
      downstream starter verwart herkomst met algemene apprechten
```

Geen bewijs dat een release deze configuratiefout heeft. **P0 voor
releaseclaims**, impact hoog. Dependencies: firmwareanker, productfeatureselectie,
sleutelbeleid en capmanifest van de appstarter.
V2: later alleen disposable images met testankers; normale signed boot,
ongeldige signature/digest, te oude versie, twee ongeldige slots en
ontbrekende vloer als apart scenario. Oracle: juiste afwijzing voor het
vastgelegde model; nooit ESP-fallback in signed mode. Apart meten wat gebeurt
als disk EN variabelen worden teruggezet; dit valt niet onder gewone
disk-only antirollback. B03-historie niet als nieuwe run gebruiken.

### T3 - offline lezen of herleven van ingetrokken sleutelstatus

```text
OR
  AND aanvaller krijgt een kopie van een niet-versleuteld documentvolume (B3)
      daarin staan documenten of gevoelige namen
  AND aanvaller bewaart oude authentieke key-state en records
      er is geen onafhankelijk vertrouwd actualiteitsanker
      koude recovery accepteert oude intrekkingsstatus
```

Eerste tak is een bekende productgrens, tweede een conditionele
rollbackhypothese; geen cryptografische breuk. **P0 voor privacy-/profielclaims**.
Dependencies: per-gebruikerkey-state, duurzame write-ahead/ankering en
betrouwbare secretinvoer. V3: vaste synthetische canary-documenten en
wegwerpvolumes; koude herstart met oude/nieuwe status, corrupte tag,
verkeerde namespace en onderbroken commit. Oracle: geen plaintext bij
authenticatiefout, geen herlevend recht, geen duurzame succesmelding voor
onzekere commit. Afzonderlijke byte-inspectie; geen echte secrets gebruiken.

### T4 - confused deputy en externe verzending via helpers

Een confused deputy is een bevoegde dienst die haar eigen rechten onbedoeld
namens een onbevoegde aanvrager inzet.

```text
OR
  AND app heeft documentrecht maar geen netwerkrecht
      netwerk-/exporthelper gebruikt eigen grant zonder callerbinding
  AND app heeft netwerkrecht maar geen documentrecht
      object-/klembordhelper verwart object-ID met machtiging
  AND oude verbinding/grant blijft bestaan na intrekking of serviceherstart
      publicatie naar NIC ontsnapt aan nieuwe generatiecontrole
```

B4 is een integratiegrens, geen aangetoonde ontsnapping.
De socketcore heeft wel destinationvergelijking, herautorisatie en een fence
bij revoke (`services/net-broker/src/lib.rs:391-425`, regels 472-509).
**P1**, dependencies: productie-issuer, procesincarnatie, device-publicatiefence.
V4: rechtensplitsing over twee synthetische apps; intrekken vlak voor en na
enqueue/publicatie; vertraagde completion na herstart. Oracle: geen nieuwe
ongeautoriseerde publicatie; eerder werkelijk verzonden bytes expliciet
onderscheiden van nog annuleerbare buffers. Observatie bij virtuele NIC plus
appantwoord, geen alleen-booleanpolicy.

### T5 - NAS-impersonatie, replay of onjuiste verwijderbevestiging

```text
OR
  AND ingress neemt callerfingerprint uit het verzoek in plaats van authenticatie
      NAS-core vertrouwt die Principal (B5)
  AND oude session/upload wordt na revoke/herstart opnieuw aangeboden
      adapter verliest boot-/epoch-/namespacebinding
  AND purge krijgt I/O-fout of commituitkomst is onzeker
      adapter meldt toch verwijderen voltooid of replayt blind
```

Niet aangetoond: een onbetrouwbare remote route naar de core.
De aanwezige core bindt sessions, CAS-revisies en persist-before-ACK
(`services/nas/native-core/src/lib.rs:617-641`).
De resumeclient waarschuwt dat een checkpoint geen durable ACK of
antirollbackanker is (`services/nas/native-client/README.md:18-43`).
**P1**, dependencies: geauthenticeerde transportadapter, betrouwbare entropy,
durable Storage-contract en reconciliatie bij Unknown.
V5: synthetische peers met verschillende identiteiten, verlopen/eenmalige
invite, oud uploadticket na revoke en koude recovery, fout tussen revoke en
purge. Oracle: stale/denied; geen namespaceherleving; purgefout blijft fout;
eigen lokale clientkopie blijft bestaan. NAS-adminverwijdering zelf niet als
misbruik tellen. TLS-echo en opslagcore niet optellen tot E2EE-productbewijs.

### T6 - vervalste gebruikersintentie en sessieverwarring

```text
OR
  AND app toont namaak-inlogvenster
      gebruiker heeft geen verifieerbare vertrouwde invoerroute
  AND eerder klembordaanbod wordt na focusverlies/ACK opnieuw gebruikt
      broker of app accepteert oude sequence/epoch
  AND compositor-/sessieherstel hergebruikt oude gebruikerstoestand
      sleutels, surfaces of selectie kruisen de gebruikersgrens
```

B8 toont aanwezige klembordverdediging; een venster is geen accountgrens.
Het profielontwerp eist fail-closed login, sessie-eigen surfaces en beveiligde
aandachtsreeks (`docs/implementation/PROFILES-AND-USERS.md:152-183`).
**P1**, dependencies: echte authenticatie-/keyringgrens en vertrouwde input.
V6: uitsluitend fictieve invoer; switch focus tijdens backpressure, replay na
ACK, compositorrestart en latere gebruikerswissel. Oracle: bodyless denial,
geen documentmutatie, geen toetsaanslagen voor apps tijdens vertrouwde login.
Hergebruik testpatroon `sdk/examples/window-session/tests/clipboard.rs:1-17`,
maar hosttests niet promoveren tot native loginbewijs.

### T7 - uitputting, parserbelasting en late DMA

```text
OR
  AND geldige client vult capnodes/endpoints/uploads of volle quota
      foutafhandeling blokkeert cleanup of voortgang van anderen
  AND onbetrouwbare document-/netwerkinvoer bereikt parser
      lengte, capaciteit of uitvoertijd is niet volledig begrensd
  AND device/reset bevestigt geen einde van oude DMA
      pool wordt toch aan nieuw proces of nieuwe generatie toegewezen
```

Vaste capaciteiten zijn aanwezig
(`crates/capability-core/src/lib.rs:13-17`,
`services/nas/native-core/src/lib.rs:22-27`).
Zij zijn geen bewijs van eerlijke scheduling of hardware-DMA-containment.
**P1**, dependencies: quota, timeout, reset-ACK en poolquarantaine.
V7: later begrensde synthetische bursts tot net boven elke limiet, fout per
allocatie-/commitgrens en gesimuleerde late completion; een onafhankelijke
heartbeat moet blijven lopen en resources moeten terugkeren. Geen echte DMA,
passthrough, hostdisk of onbeperkte stresstest. Fysieke containment blijft
E7-open; QEMU kan dat niet bewijzen.

### T8 - lokale AI en diagnostiek als datakanaal

```text
OR
  AND onbetrouwbaar document/model stuurt de assistent
      taakbroker vertaalt inhoud naar ruimere object-/exportmachtiging
  AND inference heeft geen directe socket
      downloader, subprocess of lokale helper krijgt wel context en egress
  AND foutmelding/debuglog bewaart geselecteerde inhoud of sleutelgegevens
      andere rol of latere export kan die logs lezen
```

Dit zijn **E1-productdreigingen**, geen bevinding van een werkende native
inferenceketen. Het privacycontract verbiedt directe EN gedelegeerde
AI-netwerktoegang en scheidt downloader/context
(`docs/LOCAL-PRIVACY-CONTRACT.md:82-91`). P2 beschrijft expliciet een
deterministische simulatie (`docs/implementation/PRODUCT-SPEC.md:219-221`).
**P2 tot een concrete native integratie bestaat; daarna P1**.
Dependencies: native inference/parser, taakbroker en logtoegangsmodel.
V8: fictief document met instructie buiten taak, ontbrekend model, low-memory,
helperverzoek en parserfout. Oracle: beperkte lokale fout/actie, nul nieuwe
grants, nul context in downloader/logs, geen verkeer bij virtuele observatie.
Geen download of echt model nodig voor het ontwerp; geen werking geclaimd.

## Prioriteiten, afhankelijkheden en restonzekerheid

| Volgorde | Te sluiten bewijsgrens | Afhankelijk van | Acceptatie |
|---|---|---|---|
| P0 | Juiste bootketen/featureselectie en vertrouwen in vloer (T2) | Productimage, firmwareanker en sleutelbeleid | Gecombineerd positief/negatief native bewijs met image-/bronhash; B04 blijft afzonderlijk |
| P0 | Offline inhoud, key-state en koude rollback (T3) | Per-gebruikerssleutels, duurzame opslag, vertrouwd anker | Onafhankelijke diskoracle en koude negatieve recovery; geen nieuw punt uit deze analyse |
| P1 | End-to-end caller/grant/peerbinding (T1/T4/T5) | Usercopy/capcore -> issuer -> netwerk/transport -> NAS | Afwijzingen waarneembaar bij echte grens, geen requestbool als identiteit |
| P1 | Input/sessie/clipboard en resource-/deviceherstel (T6/T7) | Vertrouwde compositor, authenticatie, scheduler, resetquarantaine | Replay/focus/crash/quota negatief plus voortgang tweede app |
| P2 | AI-/logketen (T8) | Concrete native componenten en taakinterface | Geen directe of gedelegeerde egress; foutpad zonder contextlek |

V1-V8 zijn voorstellen, **niet uitgevoerd** en geen nieuwe taakqueue.
Matrixkoppelingen: T1 -> FR-K02/K03; T2 -> FR-B02 en appinstallatie;
T3 -> FR-O05/S02; T4/T5 -> netwerk/NAS en LP-05/09/10/17/25;
T6 -> FR-S02/U02; T7 -> FR-K02/H03; T8 -> LP-04-07/12.
Voor finale acceptatie blijven de bestaande N/D/T/H/A/X-eisen gelden
(`docs/FULL-TEST-RELEASE.md:99-123`), niet alleen deze auditniveaus.

Aannames: aanvaller kan aanvankelijk geen kernelgeheugen wijzigen; voor
disk-/firmwaretakken heeft hij juist de expliciet genoemde offline
schrijftoegang. Protocolconforme QEMU-backends en single-CPU-serialisatie
zijn niet stil uitgebreid naar fysieke devices of SMP. Compromittering van
de vertrouwde client/Key Service valt niet weg door E2EE. Sidechannels,
kwaadaardige firmware, cryptografische primitiefcorrectheid, alle parsers,
alle logs, alle profielen en formele noninterference zijn niet volledig
onderzocht. Geen kanspercentage of absolute veiligheidsclaim.

## Uitvoering en controles

Alle shellinvocaties voor onderzoek eindigden met shellstatus 0; dit betekent
**niet** dat iedere onderliggende zoekopdracht slaagde. De volgende beslissende
commando's zijn werkelijk uitgevoerd; regels met semicolons zijn leesbatches.

```sh
pwd && git branch --show-current && git rev-parse HEAD && git status --short && git worktree list --porcelain && git cat-file -t beafdedfc188154fe7285848fbd411e651013f54
git status --porcelain=v1 && git cat-file -t beafdedfc188154fe7285848fbd411e651013f54 && git worktree list --porcelain | grep -A3 -Fx "worktree $PWD" && git switch --detach beafdedfc188154fe7285848fbd411e651013f54 && git rev-parse HEAD && git status --short && find . -name AGENTS.md -not -path './.git/*' -print
git rev-parse --verify beafdedfc188154fe7285848fbd411e651013f54^{commit}
git show-ref --verify refs/remotes/origin/copilot/add-os-specialist-agents
git diff --name-only
git status --porcelain=v1
```

Resultaten: schoon aan het begin; commitobject en lokale ref gevonden;
switch geslaagd; bronwijzigingslijsten voor rapportcreatie leeg.
Daarnaast zijn `view`, `find`, `rg` en `nl -ba <pad> | sed -n '<bereik>p'`
gebruikt voor de hierboven geciteerde bronbereiken; geen scripts uit de
repository uitgevoerd. De skillleesroute voor isolatie en review is gelezen
in `.agents/skills/os-dev/references/workflows/04-isolation-and-concurrency.md:7-53`
en `.agents/skills/os-dev/references/workflows/06-debugging-and-review.md:19-47`.

Mislukte verkennende lookups, geen productfouten en geen bewijs van globale
afwezigheid:

| Exacte lookup | Waarneming |
|---|---|
| `rg -n 'AI|inference|netwerk|broker' docs/implementation/LOCAL-AI*.md` | rg meldde ontbrekend pad, eigen foutstatus 2; omgevende pipeline/batch eindigde 0 |
| `rg -n 'clipboard|Clipboard|fn authorize|ticket' ui/native/window-session/src/lib.rs sdk/input-client/src/lib.rs` | Eerste pad ontbreekt, rg-fout; echte bron daarna gevonden onder `sdk/examples/window-session/` |
| `rg -n 'AI|inference|prompt|model' services/README.md docs/implementation/PRODUCT-SPEC.md` | Eerste pad ontbreekt; productspecificatie wel gelezen |
| `nl -ba services/nas/native-core/tests/core.rs` | Bestand ontbreekt; inventaris toont `services/nas/native-core/tests/authority.rs`, niet uitgevoerd |
| `nl -ba docs/README.md` | Bestand ontbreekt; root-README was reeds gelezen |

De leesbatches bewaren hun tijdelijke uitvoer via het tooltranscript; er is
geen aparte permanente commandolog aangemaakt. Geen testlog of runtimeartefact
geproduceerd. Eindcontrole gebruikt `git diff --check`, status/HEAD en SHA-256
van alleen dit rapport; de digest wordt extern aan het rapport teruggegeven
om een zelfreferentiele hash te vermijden.

Uitgevoerde eindcontrole:

| Commando / controle | Resultaat |
|---|---|
| `git diff --check` | exit 0; tracked bron ongewijzigd |
| `git diff --no-index --check /dev/null docs/audits/subagents/13-red-team.md` | exit 1, geen whitespace-diagnostiek; no-index meldt het verschil van een nieuw bestand, niet een geslaagde producttest |
| `python3 -B -` met read-only regexextractie van pad/regelcitaten en assertions voor bestand/bereik | exit 0; 49 expliciete pad/regelbereikcitaten gecontroleerd; dit controleert bestaan en bereik, niet de semantische juistheid |
| `sha256sum docs/audits/subagents/13-red-team.md` | exit 0; finale digest in overdracht, niet in eigen hashinput |
| `git rev-parse HEAD`; `git branch --show-current` | exit 0; exacte broncommit, branchoutput leeg door detached HEAD |
| `git status --short --untracked-files=all`; `git diff --name-only` | exit 0; alleen nieuw rapport, geen tracked bronwijziging |

De eerste gecombineerde eindcontrole gebruikte `&&`: de no-index-status 1
sloeg de Pythoncontrole toen over. Daarna is de Pythoncontrole afzonderlijk
daadwerkelijk uitgevoerd met exit 0; de overgeslagen poging telt niet mee.

**Niet uitgevoerd:** builds, hosttests, QEMU, fuzzing, exploits, CI, fysieke
hardwaretests, menselijke gebruikerstest, onafhankelijke audit, formele
evaluatie, releasevalidatie, netwerkobservatie en live GitHub/PR-controle.
Geen externe diensten, downloads, installaties, echte data/credentials/devices,
commits, push of PR gebruikt. Geen productscore beoordeeld of gewijzigd.

**Blockers:** geen blocker voor dit defensieve rapport. Native/hardware- en
menselijke gates, productie-issuer/peerbinding, daadwerkelijke sessieauthenticatie
en koude antirollback blijven bewijs-/integratiegrenzen; dit rapport sluit ze
niet. Rollback van de audit vergt alleen verwijderen van dit nieuwe rapport
door de integrator; de werkboom is bewust detached op de gevraagde bron gelaten.

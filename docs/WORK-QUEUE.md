# Operationele werkqueue — 30 oktober 2026

## Mandaat, bron en continuiteit

**Enige schrijver:** Teamleider werkqueue, app-ID
`5d5f8513-2d31-41af-96ce-81c5ef56ef80`, branch
`simonvanieperen-legendary-eureka`. Hostcontext-ID
`dcb34641-e6d7-483a-8b8b-21598205381f` is geen extra uitvoerder.
Donor `89f84098` gaf op 30 oktober expliciet exclusief vrij vanaf
`6e3bc3ff9922d947df8f5aa89a7bcd962da601fa`; remote-SHA gecontroleerd,
queueblob `7c011de653103dc6614c90be50b61102c8d55af8` eerst bytegelijk
overgenomen op schone `ca45a7dabe6a040921de03bdc2fad3e5045a75ba`.
Overname en eerste toewijzingen zijn gepusht als `16151b9c`; donor schrijft
de queue niet meer. Alleen queueinhoud overgenomen, geen donorstatus/runtime.

Dit is het compacte actieve register, geen tweede roadmap of bewijsarchief.
Volledige eerdere contracten, twintig SEC-lockpaden en uitvoeringshistorie
blijven onveranderd raadpleegbaar in
[`b58925f9:docs/WORK-QUEUE.md`](https://github.com/simonvanieperen/nieuw-os/blob/b58925f9ce1a9a4d045ba0753f007e7d070be7c2/docs/WORK-QUEUE.md).
Die bron bewaart donorhistorie plus de hieronder toegewezen NET/UI/NAS/HW-scopes.
Geen eerdere opdracht, claim of bewijs wordt door compactie ingetrokken;
bij een niet hieronder gewijzigde filegrens geldt dat gepinde contract.
Uitgebreide voortgang blijft bij eigen taak/PR, niet in deze queue.

**Prioriteit:** gewone interactieve DEV1 voor Windows/QEMU op vaste bron,
bediening/save/reopen/herstel; daarna shared store, object-only READ,
issuer/devicefence/TCP-CR3, launch en NAS-lifecycle. Onafhankelijk werk gaat door.
ROADMAP, FULL-TEST-RELEASE en CONTEXT blijven volledig bindend:
**34/100 → 34/100**, alle 62 FR-rijen open, honderd koude starts/24 uur,
fysieke en menselijke gates niet vervangen door DEV- of hostbewijs.

Assistent wijst gewone taken en daadwerkelijk vrije gebieden zelfstandig toe.
Voor nieuwe writes: actuele eigenaar, wijzigingen en afhankelijkheden controleren;
geen claim zien of een timeout maakt een gebied niet vrij. Bestaande opdrachten
gaan zonder nieuw ACK door. Frozen werk niet stil hervatten; parked WIP behouden.
Technische ABI-/trustconflicten met betrokken eigenaar en teamleider89; alleen
onoplosbare eigenaars-/prioriteitsconflicten naar directeur. Geen Simon-routinelus.
Manager89 blijft enige status-/scorewriter. Integratorovergang hieronder is
nog niet geactiveerd; er worden geen twee mergers vrijgegeven.

**Besloten opvolging, veilige overdracht PENDING:** directeur bevestigt de
nieuwe letterlijke Simonopdracht aan7f, die de opvolgende ENIGE integrator is.
Mergewrites pas activeren na e8→7f: laatste remote main/heads, lopende/verzonden
operatie, conflictclaims/WIP en expliciete donorvrijgave+ontvangst registreren.
7 bevestigt nul bron-/GitHubmutaties, schone0ade en read-only tot activatie.
Directiestop/handoff aan8 kreeg eerst NOTSENT, bewuste enqueue daarna GELUKT;
dat bewijst nog geen stop. Geen automatische timeout/leaseverval of dubbelmerger.
Assistent coördineert deze ene overdracht zonder nieuwe directie-/Simonpoort;
alle overige domeinwerkclaims en onafhankelijke reviews blijven geldig.

## Werkelijke sessies en primaire opdracht

24 geregistreerde rollen zijn geen claim van 24 actieve agents.
Bronnen/uitslagen hieronder zijn eigenaarrapportages, tenzij anders vermeld.
`unknown` activiteit of berichtacceptatie bewijst geen draaiende build.

| Rol / werkelijke sessienaam | App-ID | Primaire taak / toestand | Bewijs / eerstvolgend |
|---|---|---|---|
| Directeur nieuw OS | `ef7a7596-8c3d-4705-ab69-36f8a2a1a1b6` | Richting/kwaliteit; read-only | Mandaat bevestigd; alleen echte uitzonderingsbesluiten |
| Teamleider voortgangsbewaking | `89f84098-bd8c-4bc8-aeb2-0695517db26f` | Technische samenhang, status/score | Donor6e3 vrijgegeven; queuewrites gestopt; assessment34 behouden |
| Teamleider werkqueue | `5d5f8513-2d31-41af-96ce-81c5ef56ef80` | TASK-ASSISTANT-QUEUE; enige queuewriter | Overname16151b9; gewone verzoeken afhandelen; zeven afdelingscontacten bericht |
| Pull requests binnenhalen | `f2afe26e-5d6f-4948-89b8-0e2aeeb96225` | PR-intake/routering; geen queuewrites | #76/8a14770 frozen; NAS19c85f34 juiste draftleafhead/base routeren |
| Issues en pull requests oplossen (donor integratie) | `e8e471ce-0601-417b-b4fa-a04fd9c7b954` | Stop/handoffqueued; donorvrijgave nog NIET ontvangen | Laatste main/heads/operatie/conflictclaims/WIP leveren; geen stop uit stilte afleiden |
| Issues en pull requests oplossen (opvolger integratie) | `7f895683-fc79-48d0-a98a-aa0a6ccfe8b9` | Door Simon/directeur aangewezen ENIGE opvolger; activatie PENDING | Nul mutaties/schoon0ade/read-only bevestigd; na donorvrijgave ontvangst/exacteclaims. #65nietsluiten vóórdocsovernamebewijs |
| Nieuw OS release | `2ab17599-85c8-4409-ba06-a93c27485328` | TASK-REL-CANDIDATE-DEV1 | Herstelde offlinefreezec969de2 opdezelfdeca45; twee builds/preflight0 buiten ancestorconfig gemeld, wacht fresh12-slot. R77coldtools apart intakepunt |
| Build- en testinfrastructuur | `b72f2536-b494-4a6b-a608-b6fd785e18ef` | Toolidentiteit/headroom/slots; begrensde TASK-CI74-FLUSH-TRACE read-only | Geen nieuweVM/code; bestaande rodeSTEP12-artifacts duiden, niet blind rerunnen |
| Beveiliging, privacy en updates | `081d6f31-7de1-4713-b466-d15adf51b2ef` | TASK-SEC-PUBLISHER-RECOVERY-CONTRACT-1 af/frozen05784243, claim gesloten | Apps+REL gerichte interfacelezingen geen concrete fout; intake3. #74STEP12/flush ROOD; SEC72/7f67471 en authority/c5fbb21a frozen |
| Internet en netwerkdiensten | `039dbef8-bc79-4480-b003-55b8eccba736` | TASK-N01-NATIVE-CONFIG vrijgegeven | #87reviewfix f9b5988 binnen bestaande brokerclaim, gates/freeze volgen; korte TLS-CI-blokker apart vrij |
| Nieuw OS voor persoonlijke NAS | `d7636e9c-fe84-4e63-8f37-4d08a5f50d0d` | TASK-NAS-EXACT-REPLAY-PROBE payloadleaf vrij | Client19c85f34 frozen; native replay wacht REL-root/privatehandoff |
| Nieuw os: apps en SDK | `9db24a72-2ae7-4807-8e68-2fc88cb235d3` | TASK-APP-SOURCE-SNAPSHOT-CONTRACT-1; alleen nieuwe docleaf | #73/dce3:vier native SDK/DocumentSession-VMs gemeld; [exact bewijs](https://github.com/simonvanieperen/nieuw-os/pull/73#issuecomment-6530544604). Lang pad rodatafail/0VM blijft open; Apps2/8, geen install/launchacceptatie;73/80/84 frozen |
| Lokale AI en quantum-runtime | `381d080e-be3d-4c78-ac72-b99b1de2b4e5` | Modeladapter af; providerafhankelijkheid bijREL | #82/b51 frozen; followup#89/f55f40c5 apart gestapeld; geen native inference; qsim-WIP behouden |
| Native material en toegankelijkheid | `764bc9fb-2036-4b13-bbff-368037f9b2d7` | TASK-UI-SR05-REGRESSIONS; U04 af/frozen70be43d6 | U04 gerichte bronreviews7/11 geen concrete blocker, niet opnieuw getest; alleen app-localSaveadmissie, geen externe consumer/nativeacceptatie. SR05 aparte leaf, geenVM |
| Keuze native os-functionaliteit | `fbf60554-e98e-40bd-bd48-ee87a6498abe` | TASK-KS-CR3-DIAG read-only na verifierhandoff | Trace79b956e; geen kernelwriter; oorzaak/repro/kleinste vervolgclaim leveren |
| Nieuw OS en hardware | `f5b20312-b6c5-46b0-bbf6-76970d3470a2` | TASK-H03-TRANSPORT-MODEL-1 af/frozen, claim gesloten | #91/ae9b3996;32tests/lint/fmt/none en7/7mutanten gemeld; geen nativefence/DMA/punten, intake3 |
| Ontwerppakket materiaal | `6117dd21-50d2-4418-b3b6-17048afd1180` | Productontwerp/gerichte UX-review | UX-V01 reviewa3dd1c7 op29b; UX-V02/03/native/human open |
| Iconen en assets nieuw OS | `1bf8f308-d645-4632-b210-2258428f86b2` | Assetpakket af/frozen | d3547639; geen native assetconsumptie |
| Onderzoek os-rol 17 registratie | `ac0a0e5e-505e-4b77-b933-bcec2c636696` | Concrete R&D-afstemming, geen algemene audit | TLS-time4ffdde9f frozen; geen nieuwe tijdtrustbeslissing |
| Invariantverificatie nieuw OS | `018d2f82-0c7f-4f13-874e-5651c374ad50` | TLS-time-model af/frozen | 60bab473,2380histories/6942calls gemeld; geen Rust-refinement/nativebewijs |
| Prestatie-meetplan nieuw OS | `a398bc66-58a8-4ebe-9aa4-ed8ee2134fdb` | CAPBENCH geblokkeerd op rootguard | b715e362, raw35/runner1 voor metingen; geen timings/herhaling zonder nieuwebron/slot |
| Qemu-producttest nieuw OS | `1963283b-73f4-4520-83f3-e9cfb93b3693` | TASK-QEMU-PRODUCT-DEV1 voorbereid | Protocola49774be;0818replay97b73e4 is geen gewoneDEV; wacht exacte kandidaat/slot |
| Windows/qemu-testpakket nieuw OS | `d7cd7dda-ca14-4f03-8adb-275b5884acfb` | TASK-WINDOWS-DELIVERY-TAKEOVER | Route0915828f:10host+5Windows-synthetisch; nieuwe kandidaat/ZIP/OS-run nog open |
| Native appbouwer nieuw OS | `29d8407c-b3c5-4c7c-ba6b-c37ca39da332` | Viewerleaf af/frozen, objectREAD/launch blocked | #86/f2c875a7,15host/lint/fmt/none gemeld; geen provider/nativeviewerclaim |

## Vrijgegeven schrijfgebieden en acceptatie

Elke rij is de volledige nieuwe fileclaim; geen impliciete domeinrechten.
Eigenaar mag binnen de rij zelfstandig uitvoeren/testen/opleveren.
Alle runners gebruiken afzonderlijke source/build/targets/images/VARS/swtpm-state.
Geen nieuwe dependencies zonder bestaande dependencycontrole.
Rapportages in deze tabel en het sessieregister verwijzen naar de
**compacte bronindex** hieronder; een gelinkte bron is niet automatisch
een door de assistent herhaalde test of een afgeronde releasegate.

| Taak / eigenaar | Exclusief schrijfgebied | Afhankelijkheid / bewijsgrens / eerstvolgende stap |
|---|---|---|
| TASK-ASSISTANT-QUEUE /5d5 | Alleen `docs/WORK-QUEUE.md`, eigen sessieartifacts | Donor6e3 overgenomen; status/score niet aanraken. Compacte besluiten commit/push/remote-SHA; integratie pas volgens exclusieve overgang8→7 |
| TASK-TL-HANDOFF /89 | `docs/PROJECT-STATE.md`, `docs/progress.json`, `README.md`, `docs/PROGRESS.md`, `docs/assets/progress.svg`, eigen managerbewijs | Enige score-/statuswriter; assessment inclusief deze taak34→34; geen queuewrites |
| TASK-INTEGRATOR-HANDOFF /5d,8→7f | Alleen registratie door5d; donors/ontvanger eigen artifacts/remote read-only tot veilige overdracht | Directiebesluit ontvangen, opvolger7f aangewezen maar geen mergewrites tot stop/vrijgave/ontvangst. Main/heads/operatie/WIP/conflictclaims vastleggen; geen tweede bron-/conflictwriter. Alleen deze overgang wacht, domeinwerk/reviews gaan door |
| TASK-CI74-FLUSH-TRACE /BUILD12 | Bron/CI-artifacts read-only, alleen eigen sessieartifacts; GEEN kernel-/runnerwrites of VMslot | SEC meldt4db90e1 ADR-onlyCI raw35 bij install/STEP12 eerste Disk.flush/ERROR-16,v3df5. -16 is Unknown voor timeout/pollErr/completionIO, geen rootcauseclaim. Bestaande logs/driverbranches naar een onderscheidende observatie of exact ontbrekend datapunt herleiden; boundedread-onlytrace, geen brede audit/blindrerun. Als instrumentatie nodig is: minimale voorstelclaim voor m6.rs naarREL/TL, niet zelf implementeren |
| TASK-SEC-PUBLISHER-RECOVERY-CONTRACT-1 /SEC2 | Af/frozen05784243; alleen `docs/contracts/APP-SOURCE-RECOVERY.md`, claim gesloten | Basisca45, exact211regels. Apps+REL gerichte interfacelezingen zonder concrete fout; geen nieuwe verifier/catalog/crypto/issuer/SDK/rootwrites. Backend/anker/actor/reconstructie blijven OPEN; doc is geen implementatie/nativepunten. Intake3 ontvangt bron/body; Apps ontwerpt ontbrekende snapshotinterface apart |
| TASK-APP-SOURCE-SNAPSHOT-CONTRACT-1 /Apps11 | Alleen nieuw `docs/contracts/APP-SOURCE-SNAPSHOT.md`, eigen sessieartifacts | Apps expliciet geen lopende writes73/80/84; SEC05784243§1 laat private reconstructie/exportinterface expliciet bijApps, door assistent gelezen. Bestaande/donorclaims en nieuw pad gecontroleerd; SEC bevestigt GEEN actieve/gedelegeerde snapshot/core/backendwrites en trekt eigen onbevestigd voorstel in. Pins SEC05784243 + core73/dce3; nauwkeurig ontwerpvoorstel voor export/import van bronmetadata, publieke keysets, checked generaties en tombstones. Bestaande valide toestanden verliesloos behouden; dubbele IDs/ongeldige keysets/stale/overflow/Unknown/partiele restore/koude rollback expliciet behandelen zonder nieuwe trustpolicy. Geen private layout kopieren, tweede verifier/issuer/backend/anker/code/grant; ontbrekende authoritatieve actualiteit niet door snapshot vervangen. Exacte APIsemantiek/negatieven beschrijven, scopedSECinterfacelezing, links/diffcheck/progresscheck; geen implementatie/nativepunten. Na docfreeze alleen gerichte review, geen automatische corewrites |
| TASK-U04-APP-CONSUMER /UI13 | Af/frozen70be43d6: `sdk/examples/materiaal/src/lib.rs`, `sdk/examples/materiaal/tests/semantic_actions.rs`, `docs/verification/native-semantic-consumer/U04-app-consumer.md` | WIP integraal bewaard, lib ongewijzigd; staleSave weigeren, nieuw snapshot/actueleSave en errorcount4. Owner2semantic/22suite/fmt/check/Clippy/none0; Apps11+7 exacte bronreview geen concrete blocker, geen eigen testexecutie. Geen externe consumer/ELF/IPC/voltooide save/nativeacceptatie. Freeze behouden; volgende aparte SR05leaf hieronder |
| TASK-UI-SR05-REGRESSIONS /UI13 | Bestaand `ui/native/tests/interaction.rs`, nieuw `docs/verification/native-semantic-consumer/SR05-dirty-close.md`; uitsluitend bij echte rode repro minimale fix `ui/native/src/controller.rs` | Aparte leaf op70be43d63b50294a2f40bc2ba431aab557498019; dezelfde controller/testowner, eerdere wrapper/hoverclaims frozen, andere schrijvers niet vrijgegeven. SaveAndBack/KeepEditing/Discard-focus elk +Escape behoudt draft/originele disk, geen saving/COMMIT; Tab/ShiftTab wrap; precies drie actieve dialogkinderen/een focus/achtergrondacties niet beschikbaar; stale generatie weigeren. Verwachte positieve matrix, geen kunstmatig rood. Eerst repro vastleggen bij echte fout, geen API-/semantiekverruiming; gerichte+bestaande UItests/fmt/Clippy/none op eigen target. Geen render/hover/root/manifest/VM of wijziging70head. Nativeconsumer IPC/protocol/bootstrap blijft REL-afhankelijk en niet actief naastDEV1 |
| TASK-N01-NATIVE-CONFIG /NET6 | `services/net-stack/**`, eventueel nieuw `services/net-config/**`, eigen bewijs daar | REL expliciet geen actieve/gereserveerde/gedelegeerde writes; bestaande NETtransportclaim behouden. Leaf Stack4a0+DHCP16adfe5+codecmainca45 inhoudsbehoudend, geen legacyrootmerge/nieuwe protocolwrites. Adres/default-route/DNS,T1/T2/expiry/linkdown/reconnect; seconds↔100Hz checked/overflow, een PacketDevice-RX/TX/completionpad. CheckingAddress→Bound uitsluitend na echte RFC5227-probe. Eerst rode lifecyclecases incl malformed/stale/NAK/expiry/conflict/overflow/backpressure, dan locked tests/Clippy/fmt/none. Root/SDK/HW/issuer/fence/TCP-CR3 apart; geen nativeclaim zonder eigen-kernelproef |
| TASK-N81-TLS-CI /NET6 | Nieuw `.github/workflows/net-tls-quality.yml`, nieuw `services/net-tls/verification/ci-only/**` | Build12 bevestigt geen writer/claim/delegatie; net-tls op4a0 zelf via Git bevestigd. Korte #81opleverblokker naast N01; alleen lockedtest/Clippy/fmt/none/dedicatedCI, correcte triggers/paden, geen skip. Frozen4a0-code/provider/grants ongewijzigd; CI-delta via intake naar actieve integrator na veilige overdracht |
| TASK-N04-NET-BROKER /NET6 | `services/net-broker/**`, `.github/workflows/net-broker-quality.yml` | Bestaande claim; reviewfixf9b5988 voor ongevraagde ingress→ongeautoriseerde ARP/RST, eerst rood101 gemeld. Definitieve gates/bewijs/freeze naar intake; authentieke caller/grant/epoch, denyfence zonder fakecompletion. Geen issuer/nativeafdwinging veronderstellen |
| TASK-UI-PRIMARY-HOVER /UI13 | `ui/native/src/render.rs`, bestaande `ui/native/tests/render_regressions.rs`, `docs/verification/native-semantic-consumer/UX-V01-primary-hover.md` | Af/frozen29b79eae op0818, exact drie files. Primary ACCENT/SURFACE/hoverrandTEXT; rood101→4pixel/24suite groen gemeld,7/15bronreviews geen blocker. Geen token/API/runner/kernel/manifest/VM of oudePR68/DEV1mutatie. Intake heeft exacte bron; integratie na veilige owneroverdracht; aparte wrapper70be43d6 inmiddels af |
| TASK-NAS-EXACT-REPLAY-PROBE /NAS10 | `services/nas/native-probe/src/main.rs`, `CONTRACT.md`, `tests/**`, `verification/replay/**` binnen dezelfde crate | REL expliciet geen actieve/gereserveerde/gedelegeerde writes; apartece52leaf, #83source1219/evidence032 frozen. Nu payload/startuprequirements+hostnegatieven voor werkelijk boot1-issued Session40+Upload40coldreplay. Native exactreplay BLOCKED tot REL na profilefreeze root/runner/private80Bhandoff levert. Originrun/role/stage/complete/lengte/hash/immutablecoldinput vereist; tokens nooit serial/gewonelog/Git, geen fictieve tokenbron/nieuwecrypto/wire/ABI/productrestore/duurzaamheidsclaim of VMslot |
| TASK-H03-TRANSPORT-MODEL-1 /HW9 | Nieuw `drivers/virtio-snd/src/{transport_model.rs,transport_model_tests.rs}`, uitsluitend `#[cfg(test)]`-modulekoppeling in `src/lib.rs`, eigen `verification/{README.md,SHA256SUMS,model-*}` binnen crate | Leaf823cba3/#88, vorige HWcodec/contract frozen. Pure bounded Absent/Probed/Ready/Streaming/Fencing/Quiesced/Quarantined/Released, revoke/checkedepoch/slot/stale/double/RXverbod/timeout. Minimaal een echt rode mutant per I1–I6 incl doublecompletion/release, exacte delta/commando/exit/restore bewaren. Status0+BME-off modelaannames, GEEN echte DMA-quiescentie. Locked tests/fmt/Clippy/none; geen productie-export/deps/Cargo/lock/unsafe/I/O/VM. B1–B9-review14/2/4 apart |
| TASK-REL-CANDIDATE-DEV1 /REL4 | Eigen releasebewijs/source/build/target/VMstate; bestaande gates uitvoeren, geen nieuwe rootbronclaim | Bronca45/herstelde offlinefreezec969de2, faalbewijsf7f01c; nestedancestor-Cargoconfigduplicatie verklaard, buitenconfigtwee builds/preflight0 gemeld.12freshheadroom/slot, maximaal twee seriele referentie/gewone7procespreviewgates,max240s/elk. EchteQMPinput/frame/diskoracle; source/v3/ELF/ESP/tools/firmware/start-stop/limits vastzetten voor20/21. Gewone gate eerst; daarna save/coldheropen vereist afzonderlijke intake drie R77hosttools opbffe11, ontbrekeninca45;12frozen/geen writer. Geen stille intake/bypass/nieuw hulpmiddel. Geen volledige FR/Windowsclaim |
| TASK-KS-CR3-DIAG /CORE14 | Kernel/loader/ABI read-only, eigen verifierevidence/artifacts | Lopende verifier eerst expliciet overdragen; instructionfetch0x200130c naCR3=1000→DF/triplefault gericht reproduceren/verklaren of ontbrekende voorwaarde leveren. Geen runtimewriter; minimale patchclaim later metREL/TL |
| TASK-QEMU-PRODUCT-DEV1 /20 | Alleen `docs/verification/product-test-20/**` | Create/edit/save/reopen,cold,deny/revoke,Full,Unknown met echteinput/provenance; vasteDEVartifact+12slot vereist, niet-uitgevoerd blijft NOT RUN/BLOCKED |
| TASK-WINDOWS-DELIVERY-TAKEOVER /21 | `tools/package_windows_qemu.py`, `tools/windows-qemu/{Start-NieuwOS.ps1,Test-Launcher.ps1,README.md}`, `tools/tests/test_package_windows_qemu.py`, `docs/verification/windows-delivery-21/**` | Donor12/21ACK; vijf routeblobs gelijk. NaREL/actieveintegrator-kandidaatvrijgave package/hash/Windowsbediening/herstel, Linux/Windowsbewijs apart. Geen tweede route, hostdisk/persoonlijkedata/policyverruiming |

## Overige behouden grenzen en concrete wachtpunten

| Werk / eigenaar | Behouden gebied / toestand | Eerstvolgend / afhankelijkheid |
|---|---|---|
| Publisher/sourcepersistentie /SEC2, snapshotinterface /Apps11 | Recoverydoc05784243 af; uitsluitend nieuwe Appsdoc hierboven vrij; code nog GEEN claim | Frozen app-package-core niet wijzigen zonderApps+actieveintegrator; adapter/backend/anker niet door hostfixture vervangen |
| NAS-resumeclient /NAS10 | `services/nas/native-client/**` en NAS-only `.github/workflows/nas-core.yml`, frozen19c85f34 | 16debug/17release incl64KiBqueue/coreCAS/Unknown/revoke en viercrategates gemeld. Intake3 om correcte drafthead/base gevraagd, niet sessiePR71muteren; typedcheckpoint geen duurzame syncqueue |
| AI-modeladapter /AI5 | `experiments/ai/model-store-contract/**`, frozen#82/b51; tests-onlybc44 apart | Store-issued opaque Ref+immutablegeneration/aparte pinmetadata, authoritativeopen/boundedread/u64quota/explicietefouten/HandleDrop/catalogusduurzaamheid aanREL+Apps geleverd. Provider blijftREL; geen inference/download/netwerkclaim |
| Viewer /22 | `sdk/examples/image-viewer/**`, frozen#86/f2c875a7 | Apps11 contract/review; DocumentSession open kiest STATrevisie, gekozen revisie nadien vergelijken. ReadOnly/word112 is clientpolicy, GEEN objectautorisatie. Kernel-authsender/object/revision/READ+LIST/mutatieweigering bijprovider; launch/focus REL/UI |
| UI-wrapper /13 | U04 af/frozen70be43d6; nieuwe SR05leaf apart | Externe native hulpmiddelenconsumer blijft OPEN; IPC/protocol/rootbootstrap vereist een afzonderlijke REL-ownerhandoff. IMAGE-0818/e37fa17b eigen bron/v326b41e55/1VM132.461s/raw33;15eb70/20replay97b73e4 geen nieuwe ordinaryDEV |
| UX /15 | `docs/implementation/UX-MATERIAAL-SAVE-RECOVERY.md`, `ui/design/save-recovery.svg` | Reviewa3dd1c7 naeb70, UX-V01bronmatig afgehandeld; native/human/UX-V02/03 blijven open |
| Assets /16 | `ui/assets/material-flow-icons.svg`, frozend3547639 | Geen native assetconsumptie; afstemming15/13, geen rendererwriter |
| TLS-time R&D/model /17,18 | Bestaande `docs/research/TLS-CERTIFICATE-TIME-BOUNDARY.md` en `experiments/verification/tls-time-gate/**` frozen |4ffdde9f/60bab473; RTC niet tot vertrouwde certificaattijd promoveren; geen nieuwe trustbootstrap zonder besluit |
| Prestatiemeting /19 | `docs/verification/performance-19/**`, geen sourceoptimalisatie | CAPBENCHguardroot bijREL; geen nieuwe VM/timings zonder nieuwebron/12slot |
| Gedeelde root/NAS /REL4, integratieovergang8→7 | #83kernelsubset/`tools/build.py` volgens gepind contract; runtime/ABI-verantwoordelijkheid behouden | Geen nieuwe actieve rootbuilder naastDEV1. BootInfo/ACPI, native80Bhandoff, provider/READ/issuer/launch/CAPBENCH zijn afzonderlijke nog te plannen rootafhankelijkheden |
| SEC72 locks /SEC2 | Exact twintig consumerlocks en SDK/keyring/randomfiles volgens gepind contract, frozen7f67471 | Geen andere writer of locked-bypass. Geen stil NV/TPM/kernelCargo/buildrecht; nieuwe runtimewijziging vraagt exacte afbakening |
| Frozen integraties /overgang8→7 | RootCargo/locks/centrale conflicten PR73/78 uitsluitend inhoudsbehoudend | #74op4dbADR-only nu rodeSTEP12/flush, geen groenclaim of blind18VMherhaling. #75neemt alleen actuele queueblob/delta van5d over; geen ownerwijziging. Daarna78→82; exclusieve activatie en branchregels eerst |

## Compacte bronindex

Projectbronnen: **Nieuw OS**, gerapporteerd door de genoemde domeineigenaar;
de versie is de volledige commit-ID in iedere URL. Titels hieronder duiden
het betreffende bron-/bewijsdossier aan. Korte hashes in statusblokken zijn
alleen leeslabels voor deze exacte bronnen. Geen nieuwe testuitvoering door
de queuewriter; broncode, eigenaarrapportage en native acceptatie blijven
verschillende bewijsniveaus. Ontbrekend of alleen lokaal bewijs blijft open.

| Statusblok / titel | Exacte primaire projectbron of rapport-URL |
|---|---|
| Centrale contracten, eerdere fileclaims en historie | [Volledige queue b58925f9](https://github.com/simonvanieperen/nieuw-os/blob/b58925f9ce1a9a4d045ba0753f007e7d070be7c2/docs/WORK-QUEUE.md); [basis ca45](https://github.com/simonvanieperen/nieuw-os/commit/ca45a7dabe6a040921de03bdc2fad3e5045a75ba) |
| REL4: DEV1 offlinefreeze en ancestorconfig-faalbewijs | [Herstelde freeze c969de2](https://github.com/simonvanieperen/nieuw-os/commit/c969de2405347881243857e7b475220b46c1b254), [faalbewijs f7f01c](https://github.com/simonvanieperen/nieuw-os/commit/f7f01cd9f63f4d37c753d9d958f56f40561cec8a); vervangt [oude freeze b4a93e43](https://github.com/simonvanieperen/nieuw-os/commit/b4a93e43a0b2f95bc7673f0c64ccb64b76f8bb35) |
| BUILD12/SEC2: #74 ADR-only bron en mislukte native gate | [Bron 4db90e1](https://github.com/simonvanieperen/nieuw-os/commit/4db90e1e64efc6063943cfa5745efc9077f8496d), [b03-native-update FAILURE, run41106589363/job124237913427](https://github.com/simonvanieperen/nieuw-os/actions/runs/41106589363/job/124237913427). Runstatus eenmalig uitgelezen; ERROR-16-diagnose eigenaarrapportage, oorzaak OPEN |
| SEC2: source recovery en bestaande securityfreezes | [APP-SOURCE-RECOVERY, 05784243](https://github.com/simonvanieperen/nieuw-os/blob/05784243d3d1ea273b588de82069596648fbde4d/docs/contracts/APP-SOURCE-RECOVERY.md), [#72 7f67471](https://github.com/simonvanieperen/nieuw-os/commit/7f674719d5aa90523c15a5a2b0472e15a923fb5f), [authority c5fbb21a](https://github.com/simonvanieperen/nieuw-os/commit/c5fbb21a0e46e010d5e9b4748ae9b9307a67cef4) |
| Apps11: #73 SourceRegistry/SDK en hostgates | [Bron dce3b05a](https://github.com/simonvanieperen/nieuw-os/commit/dce3b05aa358f4dbde6f2349b426217779b40052), [exact hostrapport](https://github.com/simonvanieperen/nieuw-os/pull/73#issuecomment-6530422253) |
| Apps11: vier native SDK-/DocumentSession-gates en lange-padgrens | [Exacte commando's, tools, hashes en foutpad op dce3b05a](https://github.com/simonvanieperen/nieuw-os/pull/73#issuecomment-6530544604); vier beperkte VMs zijn geen install-/catalogus-/launchacceptatie |
| Apps11: catalogus/protocolfreezes | [#80 bc27ed0f](https://github.com/simonvanieperen/nieuw-os/commit/bc27ed0f20097a68c9bfdfeecc8867c7e7721211), [#84 734c8fc5](https://github.com/simonvanieperen/nieuw-os/commit/734c8fc587edffba746ef470212bb751e6537b29) |
| NET6: TLS, DHCP en brokerreviewfix | [#81 4a0ce99f](https://github.com/simonvanieperen/nieuw-os/commit/4a0ce99ff6b88adcd0e2c6be0571b30390d8d1c8), [DHCP 16adfe5](https://github.com/simonvanieperen/nieuw-os/commit/16adfe54f3e6539339aad60232f7145c4212e538), [#87 f9b5988](https://github.com/simonvanieperen/nieuw-os/commit/f9b59884d7acb6b56537838969a04cd5b5e52b77) |
| NAS10: resumeclient, oorspronkelijke probe en aparte replaybasis | [Client 19c85f34](https://github.com/simonvanieperen/nieuw-os/commit/19c85f3451fb81e302a75bddd2109bf12c787720), [#83 bron 1219dcdc](https://github.com/simonvanieperen/nieuw-os/commit/1219dcdc8b7fcc6826ab64d440249f41fa2edf49), [replaybasis ce52](https://github.com/simonvanieperen/nieuw-os/commit/ce52dd6165ba7af3bdcd1533308b556d1927443d); oudere 032-evidence alleen via gepinde volledige queue, geen nieuwe replayclaim |
| AI5: modeladapter en tests-only followup | [#82 b51a809](https://github.com/simonvanieperen/nieuw-os/commit/b51a8093320e6182afce9b189e89760fc8c99116), [#89 f55f40c5](https://github.com/simonvanieperen/nieuw-os/commit/f55f40c525d906b87ab31103a4fa23d2be906d26) |
| UI13: app-local semantic consumer; SR05-basis | [U04 bewijs, 70be43d6](https://github.com/simonvanieperen/nieuw-os/blob/70be43d63b50294a2f40bc2ba431aab557498019/docs/verification/native-semantic-consumer/U04-app-consumer.md); gerichte reviews7/11 zijn AI-bronlezingen, geen onafhankelijke menselijke/native acceptatie |
| UI13/UX15: Primary-hover en gerichte ontwerpbronreview | [UX-V01 bewijs, 29b79eae](https://github.com/simonvanieperen/nieuw-os/blob/29b79eae19de616be415cfbd7b7cc313766546d6/docs/verification/native-semantic-consumer/UX-V01-primary-hover.md), [review a3dd1c7](https://github.com/simonvanieperen/nieuw-os/commit/a3dd1c7cb1b00891a337ac9dc9bd4e7d1df77b73) |
| UI13/QEMU20: historische IMAGE-0818 en replay, niet gewone DEV1 | [IMAGE-bewijs e37fa17b](https://github.com/simonvanieperen/nieuw-os/commit/e37fa17bf4896b6a67509aee62f1de7db542cc42), [replay 97b73e4](https://github.com/simonvanieperen/nieuw-os/commit/97b73e496d57c577978671d90aa392fa2c86fa7e) |
| CORE14: CR3-diagnosetrace | [79b956e](https://github.com/simonvanieperen/nieuw-os/commit/79b956e79da7a861da3cb1fc9b42d45138be73c2); oorzaak/patch/native vervolg niet afgeleid uit alleen deze trace |
| HW9: transporttoestandsmodel | [#91 ae9b3996](https://github.com/simonvanieperen/nieuw-os/commit/ae9b39962b63e7bb8b05ce956793427d2ca39275); tests/mutanten zijn modelbewijs, geen DMA-/fencebewijs |
| Assets16: pakketfreeze | [d3547639](https://github.com/simonvanieperen/nieuw-os/commit/d3547639be733aefcc1228edad37114980e5073c) |
| R&D17/verifier18: certificaattijdgrens en model | [TLS-time 4ffdde9f](https://github.com/simonvanieperen/nieuw-os/commit/4ffdde9f2762ff39de64eeca9940e8b3548d79e0), [model 60bab473](https://github.com/simonvanieperen/nieuw-os/commit/60bab47377b45c478d59c47a57b313c12e25ce2a) |
| Meting19: CAPBENCH-blokkade | [b715e362](https://github.com/simonvanieperen/nieuw-os/commit/b715e362b04fcd65de79c8bf85bfa439add037eb); geen timingsclaim |
| QEMU20: producttestprotocol | [a49774be](https://github.com/simonvanieperen/nieuw-os/commit/a49774beface0f332e9c7afc3931174eca64d2a5) |
| Windows21: bestaande overdrachtroute en ontbrekende coldtools | [Route 0915828f](https://github.com/simonvanieperen/nieuw-os/commit/0915828fcd40492f873dc9098ee445306f1f0638), [R77 corrected coldtools bffe11](https://github.com/simonvanieperen/nieuw-os/commit/bffe11fbc8c90d0e43b784c0a4ab580a111c12cc); laatste nog niet in DEV1-basis |
| Viewer22: bron/eigen gatebewijs, geen objectautorisatie | [#86 f2c875a7](https://github.com/simonvanieperen/nieuw-os/commit/f2c875a711f9e2d3d045de160163634bcbc8d80e) |
| Intake3: eerder bevroren intake | [#76 8a14770](https://github.com/simonvanieperen/nieuw-os/commit/8a14770d6a719c3a122c54cb774db58ce9b1a728) |

Normatieve NET-bronnen, geen bewijs dat Nieuw OS hier al native aan voldoet:

| Titel | Auteur / organisatie | Versie / datum | Primaire URL |
|---|---|---|---|
| IPv4 Address Conflict Detection | Stuart Cheshire / Apple, IETF | RFC5227, juli2008; echte probe voor adresgebruik | https://www.rfc-editor.org/rfc/rfc5227.html |
| Dynamic Host Configuration Protocol | Ralph Droms / Bucknell University, IETF | RFC2131, maart1997; lease/T1/T2/lifecycle | https://www.rfc-editor.org/rfc/rfc2131.html |

PR75-bronindexthread [4427109538](https://github.com/simonvanieperen/nieuw-os/pull/75#discussion_r4427109538)
is hiermee bronmatig behandeld; pas na opname van deze queueblob in de
daadwerkelijke PR-head kan de integrator de threaddispositie bepalen.

## Afdelingsroute en voltooiing

Alle zeven aanspreekpunten zijn na de overdracht gericht bericht:
14 kern/HW/NET;2 SEC/NAS;15 ontwerp/UI;11 apps/AI/22;17 R&D/18/19;
4 kwaliteit/12/20/21;8 PR/3/7. Zij blijven zelf uitvoeren, geen extra
toestemmingslaag. Geen normale voortgangsberichten naar iedereen.

Een taakoplevering bevat exacte basis/head/branch, gewijzigde files, echte
commando's/exit/log-/hashpaden en eerstvolgende stap. Host/model/target/native/
geintegreerd/fysiek/menselijk blijven gescheiden. Voor elke nieuwe claim eerst
registreren; normale bestaande uitvoering wacht niet op queueverwerking.
Middelen zijn gedeeld: twaalf bewaakt verse headroom en exclusieve VMslots,
geen capaciteit claimen op een historische CPU/RAM-momentopname.

Niet apart bemand betekent niet geschrapt: K06/SMP, S09/import/export/back-up,
volledige apps/UI/toegankelijkheid, beveiligde opslag/netwerk/NAS/AI,
boot/updates/herstel en integrale releasegates behouden hun volledige eisen.
Status en voortgang worden uitsluitend door89 herbeoordeeld; queueonderhoud
of extra hosttests verdienen geen acceptatiepunten.

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
Integrator8 blijft enige merger; manager89 blijft enige status-/scorewriter.

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
| Issues en pull requests oplossen (integratie) | `e8e471ce-0601-417b-b4fa-a04fd9c7b954` | Enige integrator/merger | Volgorde74→75→78→82; nieuwste gates/review, queue alleen exacte ownerdelta |
| Issues en pull requests oplossen (review) | `7f895683-fc79-48d0-a98a-aa0a6ccfe8b9` | Begrensde read-only review | UI29b79eae zonder concrete regressie gemeld via13; geen eigen tests/native |
| Nieuw OS release | `2ab17599-85c8-4409-ba06-a93c27485328` | TASK-REL-CANDIDATE-DEV1 | Offlinefreeze b4a93e43 opca45; wacht fresh12-slot voor twee seriele gates |
| Build- en testinfrastructuur | `b72f2536-b494-4a6b-a608-b6fd785e18ef` | Toolidentiteit/headroom/exclusieve slots | Windowsroute aan21; R77/bffe11 frozen; geen generieke runnerwriter |
| Beveiliging, privacy en updates | `081d6f31-7de1-4713-b466-d15adf51b2ef` | TASK-SEC-PUBLISHER-RECOVERY-CONTRACT-1 vrijgegeven | #74docsfix4db90e1/threadresolved gemeld, nieuweCI loopt; SEC72/7f67471 en authority/c5fbb21a frozen |
| Internet en netwerkdiensten | `039dbef8-bc79-4480-b003-55b8eccba736` | TASK-N01-NATIVE-CONFIG vrijgegeven | #87reviewfix f9b5988 binnen bestaande brokerclaim, gates/freeze volgen; korte TLS-CI-blokker apart vrij |
| Nieuw OS voor persoonlijke NAS | `d7636e9c-fe84-4e63-8f37-4d08a5f50d0d` | TASK-NAS-EXACT-REPLAY-PROBE payloadleaf vrij | Client19c85f34 frozen; native replay wacht REL-root/privatehandoff |
| Nieuw os: apps en SDK | `9db24a72-2ae7-4807-8e68-2fc88cb235d3` | Contract/reviewafnemer; #73reviewdelta opgeleverd | #73/dce3b05a:12+9host/Clippy/fmt/beide-none gemeld, reserved/provenance/appIDfixes; #80/bc27,#84/734c frozen |
| Lokale AI en quantum-runtime | `381d080e-be3d-4c78-ac72-b99b1de2b4e5` | Modeladapter af; providerafhankelijkheid bijREL | #82/b51 frozen; tests-only bc44 apart; geen native inference; qsim-WIP behouden |
| Native material en toegankelijkheid | `764bc9fb-2036-4b13-bbff-368037f9b2d7` | TASK-U04-APP-CONSUMER heractiveerd op behouden WIP89e | Hover29b79eae af/frozen,7/15bronreviews geen blocker; geen native/human-aftekening |
| Keuze native os-functionaliteit | `fbf60554-e98e-40bd-bd48-ee87a6498abe` | TASK-KS-CR3-DIAG read-only na verifierhandoff | Trace79b956e; geen kernelwriter; oorzaak/repro/kleinste vervolgclaim leveren |
| Nieuw OS en hardware | `f5b20312-b6c5-46b0-bbf6-76970d3470a2` | TASK-H03-TRANSPORT-MODEL-1 vrij | Codec#85/72f, contract#88/823c frozen; pure hostmodel/mutanten, geen device-I/O |
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

| Taak / eigenaar | Exclusief schrijfgebied | Afhankelijkheid / bewijsgrens / eerstvolgende stap |
|---|---|---|
| TASK-ASSISTANT-QUEUE /5d5 | Alleen `docs/WORK-QUEUE.md`, eigen sessieartifacts | Donor6e3 overgenomen; status/score niet aanraken. Compacte besluiten commit/push/remote-SHA, technische integratie bij8 |
| TASK-TL-HANDOFF /89 | `docs/PROJECT-STATE.md`, `docs/progress.json`, `README.md`, `docs/PROGRESS.md`, `docs/assets/progress.svg`, eigen managerbewijs | Enige score-/statuswriter; assessment inclusief deze taak34→34; geen queuewrites |
| TASK-SEC-PUBLISHER-RECOVERY-CONTRACT-1 /SEC2 | Alleen nieuw `docs/contracts/APP-SOURCE-RECOVERY.md` | Apps bevestigt geen claim; geen concurrerende contractwriter in donor of actuele registratie, nieuw pad lokaal/donor afwezig. Basisca45; API/bronpins #73actuele SourceRegistry+catalogus#80/rootstore#78/B02. Een ontbrekend bronbeheeractor→geauthenticeerd duurzaam SourceID/keyset/generatie/revoke→installercontract, durableACK/Unknown/coldrollback/reconcile/crashvensters; epoch/versie/storegeneration gescheiden. Bestaand startbaar-na-bronrevokebeleid op9d8 APP-PACKAGES:69–74 door assistent gelezen, niet opnieuw wijzigen. Een gerichte Apps/REL-interfacelezing; ontbrekend backend/anker blijft OPEN, geen nieuwe verifier/catalog/crypto/issuer/SDK/rootwrites. Pins/links/diffcheck, compacte conceptoplevering met grenzen; geen code/nativepunten |
| TASK-U04-APP-CONSUMER /UI13 | `sdk/examples/materiaal/src/lib.rs`, nieuw `sdk/examples/materiaal/tests/semantic_actions.rs`, alleen nieuw `docs/verification/native-semantic-consumer/U04-app-consumer.md` | Heractiveert dezelfde owner13/zelfde bewaarde WIP89e, geen overdracht/nieuwe goedkeuring bestaande claim. Testbestand is niet in git89e aanwezig: lokale WIP/checkpoint eerst behouden en exacte HEAD/hash vastleggen, niet resetten/namaken. Eerst staleSave met oude editor_generation weigeren; nieuw snapshot/actuele generatie vereist voor Save en passende errorcount. Generatiecheck niet versoepelen. Gerichte semantic_actiontests+bestaande adaptertests/fmt/Clippy/none; geen manifest/lock/token/API/root/runner/VM of frozen hover/PR68mutatie. Bronfreeze daarna review7/Apps11 |
| TASK-N01-NATIVE-CONFIG /NET6 | `services/net-stack/**`, eventueel nieuw `services/net-config/**`, eigen bewijs daar | REL expliciet geen actieve/gereserveerde/gedelegeerde writes; bestaande NETtransportclaim behouden. Leaf Stack4a0+DHCP16adfe5+codecmainca45 inhoudsbehoudend, geen legacyrootmerge/nieuwe protocolwrites. Adres/default-route/DNS,T1/T2/expiry/linkdown/reconnect; seconds↔100Hz checked/overflow, een PacketDevice-RX/TX/completionpad. CheckingAddress→Bound uitsluitend na echte RFC5227-probe. Eerst rode lifecyclecases incl malformed/stale/NAK/expiry/conflict/overflow/backpressure, dan locked tests/Clippy/fmt/none. Root/SDK/HW/issuer/fence/TCP-CR3 apart; geen nativeclaim zonder eigen-kernelproef |
| TASK-N81-TLS-CI /NET6 | Nieuw `.github/workflows/net-tls-quality.yml`, nieuw `services/net-tls/verification/ci-only/**` | Build12 bevestigt geen writer/claim/delegatie; net-tls op4a0 zelf via Git bevestigd. Korte #81opleverblokker naast N01; alleen lockedtest/Clippy/fmt/none/dedicatedCI, correcte triggers/paden, geen skip. Frozen4a0-code/provider/grants ongewijzigd; CI-delta naar8 |
| TASK-N04-NET-BROKER /NET6 | `services/net-broker/**`, `.github/workflows/net-broker-quality.yml` | Bestaande claim; reviewfixf9b5988 voor ongevraagde ingress→ongeautoriseerde ARP/RST, eerst rood101 gemeld. Definitieve gates/bewijs/freeze naar intake; authentieke caller/grant/epoch, denyfence zonder fakecompletion. Geen issuer/nativeafdwinging veronderstellen |
| TASK-UI-PRIMARY-HOVER /UI13 | `ui/native/src/render.rs`, bestaande `ui/native/tests/render_regressions.rs`, `docs/verification/native-semantic-consumer/UX-V01-primary-hover.md` | Af/frozen29b79eae op0818, exact drie files. Primary ACCENT/SURFACE/hoverrandTEXT; rood101→4pixel/24suite groen gemeld,7/15bronreviews geen blocker. Geen token/API/runner/kernel/manifest/VM of oudePR68/DEV1mutatie. Volgende: intake/integratie8 op exacte bron; wrapper89e geparkeerd |
| TASK-NAS-EXACT-REPLAY-PROBE /NAS10 | `services/nas/native-probe/src/main.rs`, `CONTRACT.md`, `tests/**`, `verification/replay/**` binnen dezelfde crate | REL expliciet geen actieve/gereserveerde/gedelegeerde writes; apartece52leaf, #83source1219/evidence032 frozen. Nu payload/startuprequirements+hostnegatieven voor werkelijk boot1-issued Session40+Upload40coldreplay. Native exactreplay BLOCKED tot REL na profilefreeze root/runner/private80Bhandoff levert. Originrun/role/stage/complete/lengte/hash/immutablecoldinput vereist; tokens nooit serial/gewonelog/Git, geen fictieve tokenbron/nieuwecrypto/wire/ABI/productrestore/duurzaamheidsclaim of VMslot |
| TASK-H03-TRANSPORT-MODEL-1 /HW9 | Nieuw `drivers/virtio-snd/src/{transport_model.rs,transport_model_tests.rs}`, uitsluitend `#[cfg(test)]`-modulekoppeling in `src/lib.rs`, eigen `verification/{README.md,SHA256SUMS,model-*}` binnen crate | Leaf823cba3/#88, vorige HWcodec/contract frozen. Pure bounded Absent/Probed/Ready/Streaming/Fencing/Quiesced/Quarantined/Released, revoke/checkedepoch/slot/stale/double/RXverbod/timeout. Minimaal een echt rode mutant per I1–I6 incl doublecompletion/release, exacte delta/commando/exit/restore bewaren. Status0+BME-off modelaannames, GEEN echte DMA-quiescentie. Locked tests/fmt/Clippy/none; geen productie-export/deps/Cargo/lock/unsafe/I/O/VM. B1–B9-review14/2/4 apart |
| TASK-REL-CANDIDATE-DEV1 /REL4 | Eigen releasebewijs/source/build/target/VMstate; bestaande gates uitvoeren, geen nieuwe rootbronclaim | Bronca45/offlinefreezeb4a93e43;12freshheadroom/slot, maximaal twee seriele referentie/gewone7procespreviewgates,max240s/elk. EchteQMPinput/frame/diskoracle; source/v3/ELF/ESP/tools/firmware/start-stop/limits vastzetten voor20/21. Niet UIprobe als ordinaryDEV, geen volledige FR/Windowsclaim |
| TASK-KS-CR3-DIAG /CORE14 | Kernel/loader/ABI read-only, eigen verifierevidence/artifacts | Lopende verifier eerst expliciet overdragen; instructionfetch0x200130c naCR3=1000→DF/triplefault gericht reproduceren/verklaren of ontbrekende voorwaarde leveren. Geen runtimewriter; minimale patchclaim later metREL/TL |
| TASK-QEMU-PRODUCT-DEV1 /20 | Alleen `docs/verification/product-test-20/**` | Create/edit/save/reopen,cold,deny/revoke,Full,Unknown met echteinput/provenance; vasteDEVartifact+12slot vereist, niet-uitgevoerd blijft NOT RUN/BLOCKED |
| TASK-WINDOWS-DELIVERY-TAKEOVER /21 | `tools/package_windows_qemu.py`, `tools/windows-qemu/{Start-NieuwOS.ps1,Test-Launcher.ps1,README.md}`, `tools/tests/test_package_windows_qemu.py`, `docs/verification/windows-delivery-21/**` | Donor12/21ACK; vijf routeblobs gelijk. Na4/8kandidaatvrijgave package/hash/Windowsbediening/herstel, Linux/Windowsbewijs apart. Geen tweede route, hostdisk/persoonlijkedata/policyverruiming |

## Overige behouden grenzen en concrete wachtpunten

| Werk / eigenaar | Behouden gebied / toestand | Eerstvolgend / afhankelijkheid |
|---|---|---|
| Publisher/sourcepersistentie /SEC2 | Nieuwe docleaf hierboven vrijgegeven; code nog GEEN claim | Frozen app-package-core niet wijzigen zonderApps+8; adapter/backend/anker niet door hostfixture vervangen |
| NAS-resumeclient /NAS10 | `services/nas/native-client/**` en NAS-only `.github/workflows/nas-core.yml`, frozen19c85f34 | 16debug/17release incl64KiBqueue/coreCAS/Unknown/revoke en viercrategates gemeld. Intake3 om correcte drafthead/base gevraagd, niet sessiePR71muteren; typedcheckpoint geen duurzame syncqueue |
| AI-modeladapter /AI5 | `experiments/ai/model-store-contract/**`, frozen#82/b51; tests-onlybc44 apart | Store-issued opaque Ref+immutablegeneration/aparte pinmetadata, authoritativeopen/boundedread/u64quota/explicietefouten/HandleDrop/catalogusduurzaamheid aanREL+Apps geleverd. Provider blijftREL; geen inference/download/netwerkclaim |
| Viewer /22 | `sdk/examples/image-viewer/**`, frozen#86/f2c875a7 | Apps11 contract/review; DocumentSession open kiest STATrevisie, gekozen revisie nadien vergelijken. ReadOnly/word112 is clientpolicy, GEEN objectautorisatie. Kernel-authsender/object/revision/READ+LIST/mutatieweigering bijprovider; launch/focus REL/UI |
| UI-wrapper /13 | Zelfde WIP89e expliciet heractiveerd volgens contract hierboven | IMAGE-0818/e37fa17b eigen bron/v326b41e55/1VM132.461s/raw33, slotvrij;15eb70/20replay97b73e4 geen nieuwe ordinaryDEV |
| UX /15 | `docs/implementation/UX-MATERIAAL-SAVE-RECOVERY.md`, `ui/design/save-recovery.svg` | Reviewa3dd1c7 naeb70, UX-V01bronmatig afgehandeld; native/human/UX-V02/03 blijven open |
| Assets /16 | `ui/assets/material-flow-icons.svg`, frozend3547639 | Geen native assetconsumptie; afstemming15/13, geen rendererwriter |
| TLS-time R&D/model /17,18 | Bestaande `docs/research/TLS-CERTIFICATE-TIME-BOUNDARY.md` en `experiments/verification/tls-time-gate/**` frozen |4ffdde9f/60bab473; RTC niet tot vertrouwde certificaattijd promoveren; geen nieuwe trustbootstrap zonder besluit |
| Prestatiemeting /19 | `docs/verification/performance-19/**`, geen sourceoptimalisatie | CAPBENCHguardroot bijREL; geen nieuwe VM/timings zonder nieuwebron/12slot |
| Gedeelde root/NAS /REL4, integrator8 | #83kernelsubset/`tools/build.py` volgens gepind contract; runtime/ABI-verantwoordelijkheid behouden | Geen nieuwe actieve rootbuilder naastDEV1. BootInfo/ACPI, native80Bhandoff, provider/READ/issuer/launch/CAPBENCH zijn afzonderlijke nog te plannen rootafhankelijkheden |
| SEC72 locks /SEC2 | Exact twintig consumerlocks en SDK/keyring/randomfiles volgens gepind contract, frozen7f67471 | Geen andere writer of locked-bypass. Geen stil NV/TPM/kernelCargo/buildrecht; nieuwe runtimewijziging vraagt exacte afbakening |
| Frozen integraties /8 | RootCargo/locks/centrale conflicten PR73/78 uitsluitend inhoudsbehoudend | #74ADR-B03-K1docsfix4db90e1/threadresolved doorSEC gemeld; nieuweCI afzonderlijk, geen18VMherhaling zondercodedelta. #75neemt alleen actuele queueblob/delta van5d over; geen ownerwijziging. Daarna78→82, geen branchregels omzeilen |

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

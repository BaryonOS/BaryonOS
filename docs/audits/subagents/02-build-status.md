# AUDIT-02 - Bouwbaarheid en implementatiestatus

**Bronmetadata:** primaire repositorybron: *Nieuw OS — “Add OS specialist
agent profiles”*, auteur/organisatie S.W.J.B.M. van Ieperen, 3 november 2026,
commit `beafdedfc188154fe7285848fbd411e651013f54`
([onveranderlijke snapshot](https://github.com/simonvanieperen/nieuw-os/tree/beafdedfc188154fe7285848fbd411e651013f54)).
Alle lokale `pad:regel`-citaten hieronder verwijzen naar die snapshot.
Externe normen, hardwaregedrag en vergelijkingen met andere OS'en zijn niet
onafhankelijk geverifieerd; behandel ze als `ONBEKEND` of als toetsvraag.

**Resultaat:** de repository bevat historische build- en
uitvoeringsregistraties voor eigen UEFI-/kernelimages, gebruikersprogramma's
en native VM-deelproeven. Deze registratiebestanden bewijzen niet dat de
oorspronkelijke binaries nog in het bronarchief aanwezig zijn. Nieuw OS
bestaat dus niet uitsluitend uit ontwerp of een browserprototype.
Voor de exacte auditbron is echter geen passend build-/uitvoeringsbewijs
aangetroffen. Deze audit heeft niets opnieuw gebouwd. Een volledige
geintegreerde productrelease volgt niet uit de afzonderlijke bewijzen.

## Opdracht, bron en schrijfgrens

- Opdracht: "Inventariseer de bouwbaarheid en implementatiestatus. Wijzig geen
  productcode. Rapporteer welke componenten werkelijk gebouwd kunnen worden
  en welke alleen uit ontwerp of prototypes bestaan."
- Datum: 3 november 2026, Europe/Amsterdam.
- Repository: `simonvanieperen/nieuw-os`.
- Bevroren bronbranch: `copilot/add-os-specialist-agents`.
- Exacte gelezen HEAD: `beafdedfc188154fe7285848fbd411e651013f54`.
  De lokale bronbranchref wees bij controle naar deze commit.
- Eigen werkbranch bij aanvang: `simonvanieperen-miniature-fiesta`, HEAD
  `69809c2cb53dfb3bd77abe5352892810fba6e82d`, schone werkboom.
  Na de expliciet toegestane `git switch --detach` is de audit **detached**
  uitgevoerd. De oorspronkelijke werkbranch is niet verplaatst.
- Werkmap: eigen worktree; het lokale pad is niet gepubliceerd.
  Andere worktrees zijn uitsluitend als Git-metadata gezien; hun bestanden
  zijn niet gelezen en hun actuele resultaten zijn niet gebruikt.
- Source-v3, zelf berekend via `tools/common.py:68-84`:
  `fe0fe328ef3909e2dab4386c0ba77ebf3cf6a8244301e245863b21debfde4c99`.
- Gewone chatsessie; `.github/agents/os-builder.agent.md:10-31` is als
  roltekst gelezen, niet technisch als custom agent geladen.
- Enige schrijfbestand: `docs/audits/subagents/02-build-status.md`.
  Geen productcode, status, roadmap, voortgangsmeter, commit, push of PR.
  Geen externe dienst, installatie, download, VM, build of nieuwe buildartifact.

Gelezen instructies: root-`AGENTS.md`, `docs/AGENTS.md`, builderprofiel,
`boot/AGENTS.md`, `kernel/AGENTS.md`, `drivers/AGENTS.md`,
`services/AGENTS.md`, `ui/AGENTS.md` en `tests/AGENTS.md`.
README, actuele checkpoint, privacycontract, architectuur, relevante ADR's,
releaseacceptatie en build-/CI-entrypoints vormen de context. De concrete
auditopdracht beperkt de algemene implementatie- en publicatiewerkwijze.

## Betekenis van bewijs en bouwbaarheid

| Niveau | Betekenis in deze audit |
|---|---|
| E0 | Ontbrekend of alleen ambitie |
| E1 | Ontwerp/contract, geen implementatiebewijs |
| E2 | Hostprototype, hosttest of simulatie |
| E3 | Native broncode; een `no_std`-library kan hier compilebewijs hebben, maar draait daarmee nog niet als dienst |
| E4 | Positieve native uitvoering op de genoemde historische bron |
| E5 | Negatieve native uitvoering op de genoemde historische bron |
| E6 | Herhaalde CI-/acceptatiegate binnen de expliciet genoemde historische scope |
| E7 | Gedocumenteerde fysieke hardwaretest |
| E8 | Onafhankelijke menselijke audit of formele evaluatie met afgebakende bronbinding |

De niveaus zijn geen optelsom of automatisch overerfbare productstatus.
Een op de host uitgevoerde quantumsimulator is E2; dezelfde simulator in
eigen-OS-processen kan E4/E5 hebben zonder een fysieke QPU te bewijzen.
WHPX-uitvoering is een VM op een Windows-host, geen E7-deviceacceptatie.
Een AI-review is geen E8.

**Huidige bron:** voor alle onderstaande code geldt "bron aanwezig; niet
opnieuw gebouwd". "Historisch gebouwd" betekent uitsluitend dat de bevroren
repository passend bestaand bewijs voor de genoemde oudere bron bevat.
Het is geen toezegging dat dezelfde opdracht op deze HEAD nu slaagt.
Een bronbestand of YAML-workflow zonder geslaagde uitvoeringsregistratie
wordt niet als bouwbaarheidsbewijs behandeld.

## Werkelijke buildstructuur en afhankelijkheden

`Cargo.toml:1-4` bevat twintig root-workspacemembers en expliciete uitsluitingen.
Daarnaast bestaan afzonderlijke SDK-, UI-, driver-, netwerk-, NAS- en
experimentworkspaces. De read-only TOML-controle parseerde **84 niet-vendor
Cargo-manifests**, vond **53 getrackte lockfiles**, alle twintig rootmembers en
geen ontbrekend manifest bij de expliciete lokale dependency-`path`-velden.
Dit controleert geen Cargo-resolutie, dependencycache, features, linker of ABI.
Een root-`cargo test` dekt dus niet automatisch de hele repository.

De native hoofdingang is `python3 -B tools/build.py --kernel-feature PROFIEL`.
`tools/build.py:323-330` bouwt loader en kernel apart:
`os-loader` voor `x86_64-unknown-uefi` en `os-kernel` voor
`x86_64-unknown-none`. De inspecties staan op regels 53-113; regels 333-387
binden bron, ELF/PE-hashes, gebruikersprogramma's en ESP. Alleen een
ESP-boom is nog geen verpakte, geaccepteerde productrelease.

Rust **1.90.0** en beide targets staan in `rust-toolchain.toml:1-5`;
kernel-/UEFI-linkopties staan in `.cargo/config.toml:1-17`.
Rust **1.91.1** is een bewuste afzonderlijke netwerkafhankelijkheid:
`docs/DECISIONS.md:1389-1408`, `sdk/examples/network-service/build.py:15-48`
en `sdk/examples/tcp-probe/build.py:15-48`. De netwerkbuilder kiest de
matchende linker en bouwt de appafhankelijkheden opnieuw, niet een mix van
1.90- en 1.91-rlibs. Net-config gebruikt eveneens 1.91.1
(`.github/workflows/component-leaves.yml:48-68`).

De ontwikkelroute vereist bovendien passende gelockte dependencycaches,
Rust-linker en voor hosttests een C-linker; voor VM-bewijs QEMU plus een
bijpassend OVMF CODE/VARS-paar en de voorgeschreven ROMs.
Zie `docs/DEVELOPMENT.md:41-70`, `tools/common.py:25-51,113-143`.
De normale Materiaal-build bundelt zeven echte executables; window-session
vervangt init/compositor en voegt Calculator toe
(`tools/build_materiaal.py:13-28`). Probeprofielen voegen testbediening of
faultinjectie toe en zijn niet uitwisselbaar met gewone productbediening.

## Bestaand bewijsregister: uitsluitend historie binnen de auditbron

De volledige bestanden zijn primaire auditverwijzingen; regelverwijzingen
geven de bronbinding en de bijbehorende beperkingen aan. Absolute buildpaden
in oude rapporten zijn niet gevolgd.

| ID | Aangetroffen bewijs, exacte bronbinding en grens |
|---|---|
| H01 | `docs/verification/full-release-baseline/README.md:3-20,22-43`: bron `73ebc1e169b893eb5aba4b4f8ac06d7798136de3`, v3 `cccf0e3e7fcec439428208c4458a0194ec23c6120e1f03d96abe65b0711631b7`; 238 hosttests geregistreerd, vijf echte K05-runs normal/crash/denied/parent-exit/quota, raw33. `supervisor-gate.json` is completed/passed. Niet alle ruwe hoststdout is gearchiveerd. |
| H02 | `docs/verification/parallel-final-qsim/README.md:5-13,30-50,53-76`: bron `3e41bbf93822f7044c6020259f4a752925e1966d`, v3 `307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60`. Werkelijke loader/kernel/user-ELF-buildreceipts, drie exit0; `docs/verification/parallel-final-qsim/build-logs/build-native-qsim-os-kernel.json:2-20` bevat exacte Cargo-opdracht en status. Een echte native selftest, geen QPU. |
| H03 | `docs/verification/L02-ACCEPTANCE.md:3-35`: uitvoeringsbron `433bab9feab74473517eb8d44f2fb4475848e0cf`, bewaarde CI-run `40743008938`, native quantum-jobstap geslaagd. Twee processen, IPC, Cancel/budget/paging/cleanup; software-simulatie. De externe run is niet opnieuw opgevraagd. |
| H04 | `docs/verification/ordinary-dev1/README.md:43-76,193-265` en `isolated-build-1/build-command-records.json`: bron `ca45a7dabe6a040921de03bdc2fad3e5045a75ba`, v3 `bda48571144d7771bff145cc17b0f0712e271cbc6ee323dec43bee099ea5e89d`; normale en referentiebuild uiteindelijk exit0, geinspecteerde ELFs en gecorrigeerde linkgrenzen. De eerdere dubbele Cargo-configuratie gaf een echte mislukte VM. Na buildisolatie geen nieuwe VM in dit bewijs; niet als blijvend actuele codefout gerapporteerd. |
| H05 | `docs/verification/session-desktop-20261102/README.md:3-30,39-64` en `docs/verification/session-desktop-20261102/final-native.json:3-20`: historische registratie voor bron `344dc0afde893784503217ac676e154b13662507`, v3 `05b047819f1c1cddce6868034ab2208d8a35a95d9f974860afb8bbc53eeb8658`; die registratie vermeldt zeven executables plus loader/kernel, acht native sessiefasen en rawQEMUexit0. Dit bewijst niet dat de oorspronkelijke binaries nu aanwezig zijn. Gast werkt volgens het historische resultaat; persoonlijke testinvoer wordt geweigerd. Geen echte accountauthenticatie. De serialhash is tijdens deze audit opnieuw gelijk bevonden. |
| H06 | `docs/verification/mature-session-20261102/U01-ACCEPTANCE.md:3-6,10-54`: bron `337f2f29f1833ef4a8d2cd640b09419f85dbb83b`, v3 `3919403542c923bc86467db3184345159a5ce6881567a0e11baa85f2d39f1995`. Volledige dertienfasenvenstergate, twee apps, echte compositorfault en herstel; root raw33 en CI raw33. CI-merge `663bb8c` is expliciet afzonderlijk gebonden aan dezelfde v3. Geen E7/E8. |
| H07 | `docs/verification/native-clipboard-20261102/README.md:13-56`: rootbron `d0ae570bd4666626096f92e57acef981c0f0d7b6`, v3 `05307fef9684d97480a77ea39050d80f6affcf522a6c09c7eb53340c789b35aa`; finale testbron `e3182f0bba72f300596dd81417fbbd371a8d5d79`, CI-merge `5754763ffb257d6420f74126cae7252e36afab1e`, v3 `038615b1b425d3f24ea2d9c52b36184ea666f81237c813303dc0fac93fb756ca`. Test-only-binaire brug staat apart; 22 native fasen root/CI raw33. Geen volledige U02-acceptatie. |
| H08 | `docs/verification/n01-native-service-20261102/README.md:3-18,20-56,60-91`: bron `f152aec0615285065689f0a7e2855e5858fdb350`, v3 `56ed10aade50703e08c8422251e323b4400bf37196320d24befe0b46883bbce8`; echte TCP/UDP/NIC/RPC- en client-exitproeven, raw33. `docs/verification/issue-43-receive-timeout.md:25-85` bewaart latere builds en twee regressieruns op `5925c915f6b8d02842fffd89a94f21c0e018689a`, v3 `9a8b521f4b2e5a24b279142d8554538c5cf618e3e55a9cac3e7070e12ad4fc39`. De specifieke ReceiveTcp-timeoutfix is daar alleen host-rood/groen bewezen. Geen TLS/DHCP/productinternetclaim. |
| H09 | `docs/verification/full-release-images-844ec40/README.md:1-5`: bron `844ec40d0717a51d76f2da3a03fa415633725f6a`, v3 `f879500906299f70bce900fba9941b70e5ee8fd8b9427d08f22563c564442363`; echte PNG/JPEG-decodes en tien native negatieven, raw33. Decoderfixture, geen complete viewer. |
| H10 | `docs/verification/b01/keyring-gate.json:1-9`: v3 `202a3e684c57c8b03ec1f0666c78d337a9c21e6e55e42851dce613ce8418108d`, completed/passed; drie fasen normal/no-device/stuck, alle raw33. `docs/verification/full-release-native-wait-b02/system-gate.json:1-14`: v3 `c03070ddbbe8d233f91512a97927adc54c06da2c480966c68983cc936d5e7341`, normal/recover passed, beide raw33, koude diskhash ongewijzigd. Geen herlabeling naar audit-HEAD. |
| H11 | `docs/verification/b03-key-policy-main748/update-gate.json:1-9`: v3 `1e38e27ba67c7c6793931cfd59380c1c66210ad25b0a2bd02828bf69252901c1`, completed/passed met achttien fasen. Inclusief signatuur/hash/ELF-/sleutel-/versieweigering en cut/resume. Raw35 bij geen geldig slot en raw-9 bij de bedoelde hostkill zijn verwachte negatieve uitkomsten, geen gewone raw33-boots. |
| H12 | `docs/verification/nas-release-intake/README.md:88-110`: bron `808f5391a604b6c03f18c712aa2ab3a8929abfa4`, v3 `f7b6f26be1a70a5a702095ee56458b888af562b1a01f579810206f194e52602e`; echte normal/cold-gate, exit0, geen netwerk. `docs/verification/nas-replay-110/README.md:3-47` bewaart exacte oude-tokenreplay, een ongecommitteerde run expliciet via v3 gebonden, en latere brongebonden CI. Geen E2EE, peeridentiteit of algemene sync. |
| H13 | `docs/verification/full-release-ci-d528821/workflow-runs.json:1-77` en `docs/verification/full-release-ci-15e8cfc/workflow-runs.json`: respectievelijk `d52882196da3f38a32bf01edea379ce5300e04e2` en `15e8cfced28ad46f15f5f573e5f00e8e87b74215`, elk zeven completed/success-workflowregistraties. Zelf alleen de opgeslagen JSON gelezen; geen actuele GitHub-verificatie of volledige artifactreadback. E6 is hier historisch pipelinebewijs, niet finale releaseacceptatie. |

## Componentmatrix

Entrycommands zijn bestaande routes of de daarmee overeenkomende
manifestselecties, **niet tijdens deze audit uitgevoerd**. `tools/build.py`
kiest passende subbuilders; afzonderlijke librarycommando's vervangen die
ELF-/linker-/scalarcontroles niet. H-verwijzingen slaan op bovenstaande
historische bronnen, nooit automatisch op `beafdedf`.

| Component | Actuele bron en buildentrypoint | Bouw-/implementatiestatus en bewijsgrens |
|---|---|---|
| Eigen bootloader/BootInfo/kernel | `boot/loader/Cargo.toml:1-26`, `kernel/Cargo.toml:1-19`, `boot/loader/src/main.rs:1-3`, `kernel/src/main.rs:1-3`; `python3 -B tools/build.py`, met expliciete profielen | E3 in huidige bron; historisch echte native artifacts en E4/E5 via H01/H02/H10/H11. Geen bestaande OS-kernel nodig. Niet alleen een UEFI-demo. |
| Gedeelde kernel-, ELF- en capabilitycores | `Cargo.toml:3`, `crates/capability-core/`, `crates/kernel-core/`; rootpakketten geselecteerd in `.github/workflows/ci.yml:70-74` | E3; hosttests en daadwerkelijke integratie in historische native gates H01/H02/H06. Hosttests alleen zouden geen kernelpointer-/hardwarebewijs zijn. |
| Paging, heap, FPU en IPC-wachten | `kernel/Cargo.toml:33-54`; `tools/build.py:35-36,162-167,186-194`; `python3 -B tools/run_memory.py`, `tools/run_fpu.py`, `tools/run_ipc_wait.py` | E3; native historische memory/FPU/waitrapporten: `docs/verification/full-release-memory-15e8cfc/README.md:1-5`, `docs/verification/full-release-fpu-61ea727/README.md:1-5` en `docs/verification/native-test-version/README.md:24-39,90-100`. Geen SMP- of complete resourcebelastingsacceptatie afgeleid. |
| Procesbeheer/supervisor | `tools/build.py:28-29,276-285`; `python3 -B tools/test_supervisor.py` | Historisch gebouwd en E4/E5 H01, met procesfault/denied/quota/cleanup. Algemene dynamische appinstallatie volgt hier niet uit. |
| Blockdriver, objecten, catalogus en volume | `Cargo.toml:3`, `services/volume/Cargo.toml:1-11`; `tools/build.py:13-27,261-275`; `python3 -B tools/test_m6c.py`, `tools/test_volume.py` | E3, historische native opslag-/recoveryketen H04/H06/H10/H12 en aparte M6-gates. Huidige bron niet herbouwd. Tijdelijke volumecaps zijn geen aangetoonde productcapaciteit. |
| SDK/runtime/DocumentSession | `sdk/Cargo.toml:1-4`, `tools/build_users.py:15-19,82-95,183-201`; `cargo +1.90.0 test --locked --manifest-path sdk/Cargo.toml` en native profielsubbuilders | Werkelijke Rustcode en native gebruikersprogramma's, niet alleen API-documentatie; historisch H01/H04-H07. SDK-librarytest is E2; native documentroute apart E4/E5. |
| Entropy/Key Service/geauthenticeerde records | `services/keyring/Cargo.toml:7-15`, `tools/build.py:30-31,296-305`; `python3 -B tools/test_keyring.py`, `tools/test_system.py` | E3 en historische E4/E5 H10. Dit bewijst niet alle persistente sleutel-, antirollback-, account- en NAS-grenzen van FR-O05/P05. |
| Ondertekende boot/updates | `boot/loader/Cargo.toml:14-21`, `tools/build.py:132-140,286-295`; `python3 -B tools/test_update.py` | Historisch gebouwd, E4/E5 H11. Vereist signed-boot en expliciete imageversion. Geen volledige systeem/app/modelupdateketen of uitgebrachte ondertekende release bewezen. |
| PS/2, display- en inputdiensten | `kernel/Cargo.toml:14-26`, `tools/build_materiaal.py:13-20`; `python3 -B tools/build.py --kernel-feature native-system-materiaal-probe` | Werkelijke native code en input/displayketen H05-H07; E4/E5 binnen VM-profielen. Geen USB-/GPU-/fysieke-deviceacceptatie. |
| Materiaal, sessiekeuze en desktop | `ui/native/Cargo.toml:11-20`, `ui/native/src/lib.rs:1-14`; `python3 -B tools/build.py --kernel-feature native-system-session-preview` | Historisch E4 H05. Openbare gast en weigering van persoonlijke testinvoer zijn echt; echte gebruikersauthenticatie, sleutelontgrendeling en veilig lock/unlock niet. |
| Vensters/compositor, Calculator en klembord | `sdk/examples/window-session/Cargo.toml:11-28`, `tools/build_materiaal.py:22-28`; `python3 -B tools/build.py --kernel-feature native-window-session-probe`, `tools/run_window_session.py` | E3; historisch E4/E5/E6 H06/H07, inclusief twee apps, echte compositorcrash en tekstcopy/paste. Niet reduceren tot het oudere Calculator-hostcheckpoint. Volledige toegankelijkheid, onbeperkt appbeheer en dagelijks gebruik blijven apart. |
| Imagecodec versus viewer | `services/image-codec/src/lib.rs:1-9`, `sdk/examples/image-viewer/README.md:3-7,50-65`; `python3 -B tools/build.py --kernel-feature native-system-images`; viewer-library via `.github/workflows/component-leaves.yml:36-38,78-81` | Decoder historisch native E4/E5 H09. Viewer is een aparte controller/library zonder native executable/launchslot in die leaf. Een decoderproef is geen volledige image-app. |
| Native Ethernet/ARP/UDP/DNS/TCP en netwerkdienst | `sdk/examples/network-service/build.py:15-58`, `tools/build.py:210-251`; `python3 -B tools/run_network_service.py --case normal` en `--case client-exit` | E3, historische E4/E5 H08. Geen host-netwerkstack die ontbrekende gast-TCP vervangt. Wel synthetische Ethernetpeer en vaste bootstrapbestemmingen; geen bewijs van productinternet/TLS/DHCP op audit-HEAD. |
| Netwerkbroker en DHCP-configuratie | `services/net-broker/Cargo.toml:1-5`, `services/net-config/verification/coordinator/README.md:3-31,68-75`; `cargo +1.91.1 build --locked --release --target x86_64-unknown-none --manifest-path services/net-config/Cargo.toml` | E3 librarycode; bewaarde host-/crosscompilegates. Coordinatorbewijs noemt bron `f16062b920a84d76599c4a089f8b79a6b4cfb925` en latere herstelgates afzonderlijk. Geen draaiende DHCP-/policy-issuerdienst aangetoond door deze librarybuild. De netwerkdienst heeft wel eigen native vaste RPC-scopeproeven H08. |
| Native TLS | `services/net-tls/Cargo.toml:8-17`, `services/net-tls/README.md:8-11,37-45,84-118`; `cargo +1.90.0 build --offline --locked --manifest-path services/net-tls/Cargo.toml --lib --release --target x86_64-unknown-none` | E3 adapter/library; historisch compilebewijs op `7af60df90468e47e2c551022b799bed37ea55b96`, v3 `e659fbc8117587fccbccfc641e6e3730cf0980a2f665607fea1480a5a1711347`. Geen native TLS-proces, cryptoprovider, certificaatvalidatie of handshake. Vertrouwde certificaattijd niet beschikbaar in deze adapter. |
| NAS-hostprototype | `services/nas/README.md:3-5,28-41`; `node services/nas/demo.mjs`, `node services/nas/transport-demo.mjs`, `node --test --test-isolation=none services/nas/tests/*.test.mjs` | E2: echte hostencryptie/opslag en aparte loopback-TLS, maar geen native client/NAS. Store en transportecho zijn daar niet de complete productketen. |
| NAS native core/volume/probe | `services/nas/native-probe/Cargo.toml:8-25`, `tools/build.py:38,153-155`, `tools/build_nas.py:11-36`; `python3 -B tools/test_nas.py` | E3; leaf-ELF-builds en historische native normal/cold/replay E4/E5 H12. Lokale admin/gastprincipals en opaquebytes bewijzen geen peeraccount, E2EE, multidevice of automatische sync. |
| Apppackageverificatie en bronpolicy | `crates/app-package-core/src/lib.rs:1-12`, `crates/app-package-core/Cargo.toml:7-11`; `cargo +1.90.0 build --offline --locked -p app-package-core --target x86_64-unknown-none` met gedocumenteerde portable cryptobackend | E3 verifier/library; host- en no_std-bewijs in `docs/verification/app-package-source-policy/README.md:8-12,24-47`, bron `be830818bd88a6670d0facb524e8f1f7da3274b6`. Ondertekend ELF wordt gevalideerd; geen native installer, duurzame bronregistry, dynamisch launchpad of appupdate/verwijderketen. |
| Artifact-storecontract | `crates/artifact-store-core/src/lib.rs:1-8`; `cargo +1.90.0 build --offline --locked --release --lib --target x86_64-unknown-none -p artifact-store-core` | E3 library, geen provider/dienst. `docs/verification/artifact-store-core/README.md:3-23,34-53`: host/crosscompile op `0816a530bda9006dbb5c96a96061d8fb29e385fd`, herhaald op `53c2f147ad299b16d8dc0136afefe7f845cd0117`; geen native store-/installeracceptatie. |
| Lokale AI | `experiments/ai/model-store-contract/README.md:54-96`; scalar-librarycommand op regels 84-88 | E3 model-readercontract met historisch host/compilebewijs, **geen inference-engine**. Modelformaatparser, catalogus, downloader/installatie en echte inference ontbreken in deze leaf; gepind model is geen geladen model. UI-samenvatting in prototype is E2. |
| Quantum | `experiments/quantum/native/job/README.md:3-24`, `tools/build.py:168-185`; `python3 -B tools/build.py --kernel-feature native-qsim-job`, `tools/run_qsim_job.py` | Hostsimulator E2; echte eigen-OS-simulator en IPC-jobroute historisch E4/E5 H02/H03. Geen fysieke QPU, acceleratie of cloudfallback bewezen. |
| Audio/virtio-snd | `drivers/virtio-snd/src/lib.rs:1-8`; `cargo +1.90.0 build --offline --locked --release --target x86_64-unknown-none --manifest-path drivers/virtio-snd/Cargo.toml` | E3 controlcodec; `drivers/virtio-snd/verification/README.md:3-18,20-47` bewaart host/compilebewijs en cfg(test)-transportmodel. Geen echte queues/MMIO/DMA, broker, speler of audio-uitvoer. Basis transportmodel `823cba3eb38786d39726ecfb98d6ce7f258754f7`; modelaannames zijn geen devicefencebewijs. |
| USB | `drivers/usb/src/lib.rs:1-4`; librarycommand in `docs/verification/usb-descriptor-core-f8e7490/README.md:15-26` | E3 descriptorparser; host/crosscompile op `f8e7490b1dd8fdeb3cd8e8eb8578535bc49c5cc6`. Geen xHCI, transfers, enumeratie, hotplug of USB-HID-driver. |
| ACPI/power | `drivers/acpi-power/src/lib.rs:1-7`; librarycommand in `drivers/acpi-power/verification/README.md:20-31` | E3 parser; bronbasis `4af90fcdf0a7136307428523657fa23a05d45a11`, host/crosscompile en aparte QEMU-firmwarefixture. Die firmwareproef is geen Nieuw-OS-shutdown, suspend/resume of fysieke energietest. |
| RTC en TPM-CRB | `kernel/Cargo.toml:13,38-40,54`; `python3 -B tools/run_rtc.py`, `tools/run_tpm.py`; `.github/workflows/native-system.yml:61-76` | E3 en historische native deelproeven onder `docs/verification/rtc-native/`, `docs/verification/system-clock/`, `docs/verification/native-test-version/README.md:12-39`. RTC-tijd is geen geauthenticeerde certificaattijd; een TPM-propertyquery is geen productkeyseal-/antirollbackketen. |
| Browser, PDF, audio/videoapps, tabellen/presentaties en brede appcompatibiliteit | `docs/FULL-TEST-RELEASE.md:174-192`, `docs/ARCHITECTURE.md:90` | E0/E1 voor de volledige native productroutes in deze inventaris; geen uitvoerbare engine-/appbuildentrypoints of passend native bewijs aangetroffen. Bestaande native Materiaal, Calculator en decoder worden hiervan nadrukkelijk uitgezonderd. Geen Linux/Windows/Android-appcompatibiliteit afgeleid uit de SDK. |
| Browser-UI/prototype | `ui/prototype/README.md:3-32`; `python -m http.server 8765 --bind 127.0.0.1 --directory ui/prototype`, tests via `.github/workflows/ci.yml:120-121` | E2, geen native build. Installatie, rechtenflows, NAS-koppeling en voorbeeld-AI zijn hier expliciet simulaties, niet het bewijs voor bovengenoemde native componenten. |

## Geprioriteerde bevindingen

| Prioriteit | Bevinding | Afhankelijkheid / betekenis |
|---|---|---|
| BLOKKEREND voor een actuele bouwclaim | Geen bestaande expliciete binding aan auditcommit of eigen v3 gevonden in de getrackte bewijsgebieden. `git grep` gaf 1; alle gecontroleerde buildreceipts hebben andere bronbinding. Geen herbouw toegestaan of uitgevoerd. | Een latere integrator moet exacte huidige bron, passende omgeving, buildmanifest en relevante native gates samen binden voordat huidige bouwbaarheid/acceptatie kan worden bevestigd. Dit is geen bewezen compilefout. |
| BLOKKEREND voor lokale uitvoering in deze werkmap | `cargo`, `rustc`, `rustup`, `rust-lld`, QEMU en `cc` ontbreken op het huidige PATH; eigen `.tools/nieuw-os/{cargo/bin,rustup,qemu}`, `build` en `target` ontbreken. | Geen conclusie over installaties elders op de host. Geen donorcheckout gebruikt, niets geinstalleerd. De audit zelf is hierdoor niet geblokkeerd; nieuw buildbewijs wel. |
| HOOG | Native veilige internet-/NAS-/providerketen is niet compleet: library-TLS zonder native provider/handshake, NAS-probe zonder E2EE/peeridentiteit/sync. | Native netwerkdienst H08 is echt, maar sluit deze andere grenzen niet. Nodig: geauthenticeerd transport, sleutel-/identiteitsketen, rechten en productintegratie; `docs/FULL-TEST-RELEASE.md:205-216`. |
| HOOG | Meerdere noodzakelijke productfuncties zijn geen uitvoerbare native keten: AI-inference, installer/appbeheer, PDF/browser/media, audio- en USB-transport. | Contractcrate, decoder of modelpin niet als volwaardige app/dienst meetellen. Zie componentmatrix en `docs/FULL-TEST-RELEASE.md:174-241`. |
| HOOG | De bruikbare native desktop is bewezen op afzonderlijke historische bronnen/profielen, niet als volledige release op deze HEAD. | H06/H07 leveren serieus native bewijs, maar sluiten accountauthenticatie, toegankelijkheid, hardware, integraal herstel en dagelijks gebruik niet. VM-FR en definitieve imageacceptatie blijven afzonderlijk. |
| MIDDEL | Reproduceerbaarheid hangt af van expliciete workspace-, compiler-, linker-, scalarbackend-, cache- en outputselectie. Alleen de roottoolchain installeren is onvoldoende voor netwerkcomponenten. | ADR-046, builddispatch en afzonderlijke locks volgen. Historische DEV1-fout door geerfde dubbele linkerflags is na buildisolatie hersteld; geen reden om die fout opnieuw als onherstelde productbug te tellen. |
| MIDDEL | Historische beschrijvingen kunnen lager bewijs noemen dan latere opgeslagen proeven. Bijvoorbeeld `services/nas/README.md:17-25` noemt nog geen QEMU, terwijl H12 wel native normal/cold vastlegt. | Bron, datum en latere registratie samen lezen. Omgekeerd mag latere acceptatie niet automatisch de audit-HEAD groen maken. |
| ONBEKEND | Huidige hosted CI-/PR-status, volledige artifactbeschikbaarheid, fysieke support, menselijke gebruikerstest en E8-evaluatie zijn niet opnieuw vastgesteld. | Geen externe diensten toegestaan. Bewaarde CI-JSON is historie; verwijzingen naar lokale binaries zijn geen aanwezige artifacts in deze werkmap. |

Geen nieuwe **KRITIEK**-codebevinding vastgesteld binnen deze bouwinventaris.
Dit is geen kwetsbaarhedenaudit en geeft geen veiligheidsgoedkeuring.
LAAG-stijlpunten zijn niet onderzocht; zij veranderen de bouwconclusie niet.

## Zelf uitgevoerde controles en exitstatus

Alleen-lezen inspecties draaiden vanuit de eigen werkmap. Geen
commandologbestanden zijn in de repository gemaakt; relevante uitkomsten
staan hieronder en in het sessietooltranscript. De Pythoncontrole in de
volgende sectie is volledig herhaalbaar zonder Cargo, netwerk of outputfiles.

| Exact commando / controle | Exit en uitkomst |
|---|---|
| `git branch --show-current` | 0; eerst `simonvanieperen-miniature-fiesta`, na switch leeg: detached |
| `git rev-parse HEAD` | 0; eerst `69809c2cb53dfb3bd77abe5352892810fba6e82d`, daarna exact `beafdedfc188154fe7285848fbd411e651013f54` |
| `git status --short` | 0; voor de switch en voor rapportschrijven geen uitvoer |
| `git cat-file -t beafdedfc188154fe7285848fbd411e651013f54` | 0; `commit` |
| `git worktree list --porcelain` | 0; metadata gelezen, geen andere checkoutinhoud gebruikt |
| `git switch --detach beafdedfc188154fe7285848fbd411e651013f54` | 0; alleen na vastgestelde schone eigen werkboom |
| `git rev-parse copilot/add-os-specialist-agents` | 0; exacte auditcommit |
| `git diff --name-only` en `git ls-files --others --exclude-standard` | elk 0; geen uitvoer voor rapportschrijven |
| `git diff --check` | 0 na rapportschrijven; controleerde alleen tracked wijzigingen. Het rapport was toen untracked, dus deze controle valideert dat rapport niet. Er is geen exact commandolog van een rapport-specifieke controle vastgelegd; die claim blijft onbewezen. |
| `python3 --version` | 0; Python 3.14.4 |
| `command -v python3`, `command -v git`, `command -v objdump`, `command -v readelf`, `command -v nm` | elk 0; beschikbaar op PATH |
| `command -v cargo`, `command -v rustc`, `command -v rustup`, `command -v rust-lld`, `command -v clang`, `command -v ld.lld`, `command -v qemu-system-x86_64`, `command -v qemu-img`, `command -v node`, `command -v cc` | elk 1; niet gevonden op PATH. Geen compiler/VM gestart |
| `command -v npm` | 0; Windows-npm gevonden. Niet uitgevoerd; zonder gevonden node geen Node-werkbaarheid afgeleid |
| `git grep -l -e fe0fe328ef3909e2dab4386c0ba77ebf3cf6a8244301e245863b21debfde4c99 -e beafdedfc188154fe7285848fbd411e651013f54 HEAD -- docs services drivers sdk ui experiments` | 1; geen getrackte expliciete commit-/v3-match in deze gebieden voor rapportschrijven |
| Onderstaande `python3 -B`-metadatacontrole | 0; 84 manifests gelezen, twintig rootmembers aanwezig, nul ontbrekende expliciete lokale dependencymanifests, 53 lockfiles; historische serialhash gelijk; twee keer zeven opgeslagen CI-successes; drie oude buildreceipts exit0 |

Overige verkenningscommando's waren `git ls-files`, `git ls-tree --name-only
HEAD`, `rg`, `nl -ba`, `head`, `sed` en read-only bestandsweergaven op de
genoemde bronbestanden. Hun samengestelde inspectieaanroepen eindigden met
exit0; een lege bewijszoekactie is apart hierboven als exit1 geregistreerd.
Zij zijn geen build-/testresultaten.

### Exact uitgevoerde reproduceerbare metadatacontrole

```sh
python3 -B - <<'PY'
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tomllib
sys.path.insert(0, 'tools')
from common import source_hash
files = subprocess.check_output(['git', 'ls-files', '*Cargo.toml'], text=True).splitlines()
owned = [Path(p) for p in files if 'vendor' not in Path(p).parts]
missing = []
for path in owned:
    data = tomllib.loads(path.read_text())
    tables = [data, *data.get('target', {}).values()]
    if 'workspace' in data:
        tables.append(data['workspace'])
    for table in tables:
        for kind in ('dependencies', 'dev-dependencies', 'build-dependencies'):
            for name, dep in table.get(kind, {}).items():
                if isinstance(dep, dict) and 'path' in dep:
                    if not (path.parent / dep['path'] / 'Cargo.toml').is_file():
                        missing.append((str(path), name, dep['path']))
root = tomllib.loads(Path('Cargo.toml').read_text())
print('source_v3', source_hash())
print('owned_manifests_parsed', len(owned))
print('missing_local_dependency_manifests', missing)
print('root_members', len(root['workspace']['members']))
print('root_member_manifests_exist', all((Path(p) / 'Cargo.toml').is_file() for p in root['workspace']['members']))
print('tracked_lockfiles', len(subprocess.check_output(['git', 'ls-files', '*Cargo.lock'], text=True).splitlines()))
base = Path('docs/verification/session-desktop-20261102')
report = json.loads((base / 'final-native.json').read_text())
print('historical_session_serial_sha_matches', hashlib.sha256((base / 'final-native-serial.txt').read_bytes()).hexdigest() == report['serial_sha256'])
for suffix in ('d528821', '15e8cfc'):
    data = json.loads(Path(f'docs/verification/full-release-ci-{suffix}/workflow-runs.json').read_text())
    runs = data['workflow_runs']
    print('historical_CI', data['source_commit'], len(runs), sum(r['status'] == 'completed' and r['conclusion'] == 'success' for r in runs))
for path in sorted(Path('docs/verification/parallel-final-qsim/build-logs').glob('*.json')):
    data = json.loads(path.read_text())
    if 'returncode' in data:
        print('historical_build_receipt', path.name, data['returncode'])
PY
```

Exit0. Kernuitvoer:

```text
source_v3 fe0fe328ef3909e2dab4386c0ba77ebf3cf6a8244301e245863b21debfde4c99
owned_manifests_parsed 84
missing_local_dependency_manifests []
root_members 20
root_member_manifests_exist True
tracked_lockfiles 53
historical_session_serial_sha_matches True
historical_CI d52882196da3f38a32bf01edea379ce5300e04e2 7 7
historical_CI 15e8cfced28ad46f15f5f573e5f00e8e87b74215 7 7
historical_build_receipt build-native-qsim-os-kernel.json 0
historical_build_receipt build-native-qsim-os-loader.json 0
historical_build_receipt build-qsim-user.json 0
```

## Niet uitgevoerd, aannames en overdracht

**Niet uitgevoerd:** Cargo metadata/fetch/build/test/clippy/fmt, native
crosscompilatie, ELF-inspectie van nieuw gebouwde binaries, preflight,
Python-/Node-tests, enige QEMU-/Windows-/hardwaretest, 100 koude starts,
duurproef, modeldownload/inference, netwerk/NAS/providerverkeer,
releasevalidatie en menselijke/securityaudit. Geen van deze controles
wordt als groen resultaat voor de auditbron voorgesteld.

De metadata-parser controleert alleen expliciete lokale paden en TOML-syntax;
workspace-inheritance, optionele dependencies, build scripts, registryherkomst
en semantische featureconflicten zijn daarmee niet volledig doorgerekend.
Een bestaande hash en `passed=true` bewijzen opgeslagen integriteit of een
historische runneruitkomst, niet de aanwezigheid van de oorspronkelijke
binaries of onafhankelijke menselijke verificatie. Alleen de genoemde
serial-/metadatahercontroles zijn zelf uitgevoerd. Geen absolute oude
artifactpaden of andere branches gebruikt om hiaten aan te vullen.

De bestaande README-meter staat op 41/100 (`README.md:32-61`) en is niet
herbeoordeeld of gewijzigd: scorebeheer en gezamenlijke integratie behoren
expliciet aan de hoofdagent. Deze audit levert geen nieuw native criteriumbewijs.

Overdracht: volledig rapport in `docs/audits/subagents/02-build-status.md`;
het oorspronkelijke lokale worktreepad is niet gepubliceerd.
De inhoud is ongecommit op detached `beafdedfc188154fe7285848fbd411e651013f54`.
De hoofdagent kan alleen dit document opnemen in de gezamenlijke auditcommit;
er is geen productimplementatie of integratiebesluit genomen.

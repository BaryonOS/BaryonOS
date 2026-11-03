# Repositorybasis voor het vervolgonderzoek — fase 1 vastgelegd

Datum: 2 november 2026. Bronfamilie **RB-01**. Dit is de compacte registratie van de eerdere alleen-lezen analyse, geen nieuwe securityaudit of native heruitvoering.

## Gecontroleerde bronstanden

| Bron | Vastgestelde toestand |
|---|---|
| Lokale hoofdkloon en live main | `e08fb0e48c1bb0781e98357f588c2f73b6ae8500`; 5.359 tracked bestanden geïnventariseerd; opgeslagen roadmap 34/100 |
| Nieuwere ontwikkelbranch, live gecontroleerd | `codex/mature-native-session-20261102`, vastgepind op `1f0dd9c93959e3b1335656da1e548525a288133e`; opgeslagen roadmap 41/100 |
| Lokale PC-preview | Oudere bron `7648a8f`; afzonderlijke QEMU-bundel, geen actuele ontwikkelbranch |
| Deze onderzoeksbranch | `codex/os-research-phases-2-4-20261102`, afgesplitst van main; uitsluitend onderzoek en documentatiestatus |

De lokale ontwikkelref `e5ffc3b` bleek tijdens fase 1 achter te lopen. Daarom zijn de relevante actuele bronnen via de GitHub-connector op exact `1f0dd9c` gelezen; er is geen checkout verwisseld of bestaand werk overschreven. Score 41 wordt niet naar deze op main gebaseerde documentatiebranch gekopieerd.

## Architectuur en product

[REPOSITORY] Eigen Rust-UEFI-loader en x86_64-`no_std`-kernel, capabilitygerichte proces-/IPC-grenzen, afzonderlijke systeemdiensten, eigen native SDK en grafische laag. Geen bestaande OS-kernel als onderlaag. Meerdere Cargo-workspaces/lockfiles, Python/QEMU-harnassen en Node-hostmodellen. Bronnen: [workspace](../../../Cargo.toml), [kernelcontract](../../implementation/KERNEL-AND-SECURITY-CONTRACT.md).

[REPOSITORY] Materiaal is de gekozen documentgerichte productlogica: objecten/collecties, verwisselbare gereedschappen, gegevenslocatie, beperkte rechten en herstel. Het browserprototype is HTML/CSS/JavaScript met reducer, mockbroker, fixtures en tijdelijke browseropslag. Navigatie: Mijn materiaal, Gereedschap, Apparaten, Assistent. NAS, installatie en AI zijn in deze laag gesimuleerd. Bron: [productspecificatie](../../implementation/PRODUCT-SPEC.md), [prototype](../../../ui/prototype/README.md).

[REPOSITORY] De vastgepinde ontwikkelbranch voegt een echte native desktop toe: Materiaal en Rekenmachine als geïsoleerde processen, twee vensters, focus, slepen, dock en compositorherstel met behoud van appwerk. U01-acceptatie en N01-netwerkdeelbewijs zijn opgeslagen; een tekstklembord van 192 UTF-8-bytes is later geïntegreerd. U02-snelheid, echte accountauthenticatie en volledige productacceptatie blijven open. [Checkpoint](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/docs/PROJECT-STATE.md), [U01-bewijs](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/docs/verification/mature-session-20261102/U01-ACCEPTANCE.md).

## Beperkingen die externe ontwerpideeën moeten meenemen

- Twee vaste vensters; geen algemene dynamische app-/windowlijst, resizen, snapping of virtuele-desktopbediening aangetroffen. [Scene](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/sdk/examples/window-session/src/scene.rs).
- Native sessiesearch zoekt een vaste Materiaal-verwijzing; het browserprototype filtert titels/collecties/types. Geen geïntegreerde lokale inhouds-/app-/actie-/instellingenindex.
- Venstersessie: schaal 1 of 2, minimaal 800×500 logische pixels en maximaal 1920×1080 fysieke pixels. Mobiele adaptiviteit en algemene HiDPI-dekking zijn niet aangetoond.
- Open Sans-atlas met beperkte tekendekking; complexe shaping, IME, bidi en algemene fontfallback ontbreken. Native semantische rollen/focus bestaan op appniveau; OS-brede hulptechnologieketen blijft open. [Graphics](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/ui/graphics/README.md), [native frontend](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/ui/native/README.md).
- Browser- en native paletten verschillen; spacing/kleuren komen deels uit CSS, deels uit Rust en vaste scenewaarden. P2 is geen definitieve huisstijl. [Tokens](../../../ui/prototype/tokens.css), [native thema](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/ui/graphics/src/theme.rs).
- Capabilitycore: 16 processen, 256 slots per proces, 320-byte IPC. Volume: 16 objecten totaal, 64 KiB/document, 32 UTF-8-naambytes, twee historische revisies, vier collecties per domein. Oude DocumentSession-catalogusroute heeft kleinere inhoudslimieten dan de nieuwe volumeroute. [Core](../../../crates/capability-core/src/lib.rs), [volume](../../../services/volume/src/lib.rs), [DocumentSession](../../../sdk/document-session/src/lib.rs).
- Native TLS/provider/vertrouwde tijd, volledige NAS-sync, brede appcompatibiliteit, lokale AI-inferentie, fysieke drivers en dagelijks beheer blijven afzonderlijke ontwikkelopgaven. Een map of hostmodel is geen geïntegreerde OS-functie.

## Ontwerpanalyse, geen gebruikersbevinding

[ANALYSE] De sterkste bestaande samenhang zit in materiaalidentiteit, beperkte rechten, opslagstatus en herstel. De nieuwere appgerichte Start/dock-route moet daarmee verenigbaar worden gemaakt. Gewone desktopconventies zijn herkenbaar; er is geen één-op-één-kopie vastgesteld. Het feit dat een patroon origineel oogt bewijst geen bruikbaarheid.

Onderzoeksrelevante gaten: uniforme instellingen, systeembrede rechteninventaris, notificatie-/focusmodel, algemene vindbaarheid, uitgebreid documentbeheer, toegankelijke schaal/taal en expliciete migratie. Dit zijn in de code aangetroffen of documentair open integraties, geen bewezen prioriteiten van gebruikers.

## Verificatiegrens

In fase 1 zijn de twee bestaande browsermodeltestsuites en `node --check ui/prototype/app.js` geslaagd, evenals `python -B tools/progress.py --check` op main. Git-diff bleef leeg, behalve het eerder aanwezige untracked `.github/github-app.yml`, dat niet is gelezen of gewijzigd. Geen verse native build, QEMU-run, screenreaderproef, contrastmeting of menselijk onderzoek. In deze documentatiestap worden alleen de relevante document-/broncontroles herhaald; historische OS-tests worden niet als nieuw uitgevoerd opgevoerd.

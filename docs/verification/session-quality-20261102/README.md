# Sessiekwaliteit — uitgevoerde eerste native slice

2 november 2026. Concept-PR [#129](https://github.com/simonvanieperen/nieuw-os/pull/129), branch `codex/session-quality-20261102`. **Dit is geen afgeronde volledige OS-/releaseopdracht.** De oorspronkelijke totaalprompt blijft leidend; VMware is een aanvullend compatibiliteitsspoor.

## Bron en nulmeting

De schone aanvang was `work/e08fb0e48c1bb0781e98357f588c2f73b6ae8500` (main, meter 34). De nieuwste relevante implementatie stond in PR 122: `1f0dd9c93959e3b1335656da1e548525a288133e`, meter 41. Daarvan is de nieuwe werkbranch afgeleid; gebruikerswerk en andere concept-PR’s zijn behouden. De functionele eindbron is **`703cc0705471062ffb1483a16f42a2e580a6fdf7`**, source-v3 **`e4ad9f922d202ee4249b6b1dd02a02d724f6775c616ad1c46dde48bf7ee2271c`**. Latere commits bevatten uitsluitend documentatie/artifacts. De laatste Git-HEAD staat in de opleverreactie en op PR 129.

Bij aanvang open: #39 N01, #40 U01, #41 updates, #42 hardware/input, #43 uitvoering en #109/#110 integratievervolg. PR 121/122 bevatten al sessiepreview respectievelijk geïsoleerde vensters/klembord/herstel; #120 en #125 betroffen andere integratie/status. Native U01/N01 waren op PR 122 al geaccepteerd, terwijl de main-tickets nog open stonden. Deze opdracht kent **geen extra punten toe:41 → 41**. Volledige FR-S02, U02 en hardware-/releasegates blijven open.

Het browser-Materiaal is een apart P2-prototype. De eerdere gebruikersscreenshot is in deze chat niet als afbeelding beschikbaar; [de reproduceerde native 1f-nulmeting](native/before-native.png) is daarom een herleidbare vervangende baseline, geen beweerde kopie van dat attachment. Dit beeld komt uit de eigen kernel/QEMU-venstersessie, niet uit een browser. Reeds bestaande functies: eigen UEFI/Rust-kernel, capability-/IPC-isolatie, opslag/keyring, twee appprocessen, compositorherstart, clipboard en QEMU-oracles.

## Onderzoek en productkeuzes

[Bronnen en zeven gerichte patroonketens](../../research/SESSION-ROUTES-2026-11-02.md) behandelen Windows 11, macOS, GNOME, KDE Plasma, SDDM, Android en ChromeOS; lokale FIDO2 afzonderlijk via pam-u2f, systemd-homed en Windows OS-sign-in. Apple-guide-opvragingen waren deels HTTP429; de officiële geïndexeerde fragmenten en die toegangsgrens zijn vermeld. Dit is gericht brononderzoek, geen uitputtende visuele audit van iedere actuele platformvariant.

Overgenomen: apart lockscreen, expliciete gebruiker/methode, servicebeslissing over authenticatie, vaste status/zoekingang, herkenbare vensters en sessieacties. Bewust anders: lokaal Materiaal en objectgeschiedenis, begrensde/intrekbare gereedschapsrechten, geen leveranciersaccount of cloudzoekingang. Niet overgenomen: loginvoorwaarden van Entra als bewijs voor lokale accounts, swipebediening op desktop, glas/neon, abstracte iconen zonder labels, nepvoortgang of UI-booleans als authority.

[Quiet-session-specificatie](../../implementation/QUIET-SESSION-SPEC.md) definieert licht/donkerrollen, typografie, spacing, focus, beweging, fouttoestanden en acht samenhangende boardtoestanden. De OSDev-leerbron *Drawing In a Linear Framebuffer*, W004597/revisie30599 (2026-05-21), is alleen aanvullende context; bestaande begrensde GOP/SUR-code is leidend, geen tutorialcode overgenomen.

## Gebouwd

| Gebied | Concrete verandering en bewijsgrens |
|---|---|
| Sessieroute | Opt-in `native-session-route`: lock→gebruikerskeuze→persoonlijke methoden weigeren→expliciete openbare VM-gast→werkruimte→zoeken/app→systeemmenu/instellingen→afschermen→terug→afsluiten. Geen persoonlijke credentials verzameld; geen biometrie/FIDO2-nativeclaim. |
| Authcontract | Password/LocalPin/Fingerprint/Fido2SecurityKey/RecoveryMethod; request/epoch/gebruiker/methode/boot/sessie/doel/deadline, aparte activatie, timeout/cancel/hardwareverlies/servicefalen/retry.21 hosttests; broker moet issuer, challenge, limiter en intrekking zelf afdwingen. UnavailableBroker blijft dicht. |
| Vertrouwde systeemlaag | Compositor-owned opaque foreground, appinput geblokkeerd, oude input/clipboardoffers ingetrokken, ACK vóór systeemactie, stale pointercancel. Annuleren/Escape behoudt appmodel. Capabilitybeeld is een expliciet niet-verlenend aanvraagvoorbeeld. |
| Desktop | Vaste status-/UTC-klok, gelabelde taakbalk, appactivatie/focus, lokale app/materiaal/instelling/actie-ingangen, sessie-/melding-/instellingenoppervlak, vensterverbergen/sluiten/heropenen en bestaande titelbalkslepen. Sluiten sluit de view; proces/model blijft resident. Resize blijft een contract voor nieuwe begrensde geometry/rebind, geen native implementatieclaim. |
| Visueel | Warm canvas/witte actieve vensters tegenover inktkleurige systeemlaag; proportionele systeem- en calculatorlabels; datumafstand,200%-displayklok, zoekveld/caret en compactere panelen na review hersteld. Geen transparantie of nieuwe decoratieve animatie. |
| Rendering | Sparse SUR1 uit #117: volledig eerste frame, daarna strikt oplopende gewijzigde tegels plus eindtegel; cache wordt ongeldig bij rebind. Opaque shortcut bewaart referentiepixels; oude én nieuwe paneelgrenzen worden hertekend. Verborgen vensters en RTC-lockklok hebben eigen damage. |
| Gereedheid | Desktop verschijnt pas na beide werkelijke complete SUR-frames. Diagnostische READY-publicatie gebruikt0 vóór veldupdates en gereedheid als laatste, tegen timerpreëmptie. |

Belangrijkste bronbestanden: `ui/native/src/session.rs`, `sdk/examples/window-session/src/{shell,scene,main}.rs`, `ui/graphics/src/{theme,text}.rs`, calculatorrenderer/linker, Materiaal-app, SUR1 client/receiver en bestaande build-/window-runner. [Volledige bestandslijst op de codefreeze](changed-files.txt). Alleen bestaande lokale crates en reeds gepinde zeroize zijn toegevoegd als dependency; geen nieuwe crypto, unsafe, kernel-ABI of appapparaatrecht. Calculatorreadonly is expliciet512 KiB voor de bestaande fontatlas; writable budget en W^X blijven begrensd.

## Werkelijk bewijs

| Niveau | Uitgevoerd |
|---|---|
| Ontworpen | Zeven OS-routes, lokale FIDO2, sessie-/trusted-/venster-/a11ycontracten. |
| Host |63 native-UI-tests incl21 auth en board;58 windowtests incl9 exacte raster/damage-negatieven;41 Materiaal,23 SUR1 met client,30 calculator,9 graphics. Strikte Clippy/fmt, native-bin-Clippy, 187 Python-tooltests,10 window-runnerchecks,6 latency-helperchecks;39 prototype-state-tests. Zie [commando’s/exitstatus/loghashes](checks.json) en [logs](checks/). |
| Browserprototype | Bestaande state-/CSP-/contrasttests groen; geen nieuwe browser-UI of browsera11yacceptatie geclaimd. |
| Native QEMU | Op703:18 sessiebeelden, routegate=true, raw33 en normale cleanup; afzonderlijk22 regressiefasen met native_gate/clipboard_gate=true, raw33, echte compositorfault/herstart, appbehoud en schijfcontrole. |
| Fysieke hardware/VMware | Niet uitgevoerd. Geen nieuwe Windows/WHPX-runtime, fysieke GPU/HID, USB/FIDO2, biometrie of menselijke audit. |

De 22-fasenproef is onafhankelijk teruggelezen:34 hostoracles,17 exacte volledige appbodyvergelijkingen en twee native calculatorpixelgelijkheden; compositorPID 5→9, app-PIDs 4/8 blijven, beide input/surfaceepochs vernieuwd, probes3→10.14 bytes/revisie1 op de werkelijke schijf; canaries intact; processen/handles/queued messages nul en framebaseline hersteld. [Native rapport](integrated-native.json) · [readback](integrated-readback.json) · [raw serial](integrated-serial.txt.gz).

De 18-toestandenroute bewijst compositorbediening/invoerblokkering, **geen authenticatie-/keyringgrens**. De app-keycounters blijven0 tijdens persoonlijke-methodetests. Schermafbeeldingen van deze route zijn echte captures met gereedheid en visuele review; ze zijn geen volledige onafhankelijke whole-screen-pixeloracle. De 22-fasenregressie vergelijkt volledige geselecteerde appbodies, zonder interne masks, en heeft een onafhankelijke diskoracle. [Sessierapport](session-route-native.json).

## Vóór/na en visuele acceptatie

[Native vóór](native/before-native.png) · [native lock](native/lock.png) · [native desktop](native/desktop.png) · [native zoeken](native/search.png) · [native heropend Materiaal](native/material-reopened.png) · [acht-statenboard](screenshots/session-board-host.png).

De oude presentatie gaf weinig navigatie-informatie: twee abstracte dockicons, nauwelijks systeemstatus, lege grote vlakken en technische/monospaced calculatorinformatie. Nieuwe systeemregels geven vaste plaatsen, labels, focus, echte sessiestappen en herkenbare actieve/inactieve vensters. Dat is meer dan een andere achtergrondkleur.

[Visuele review](visual-review.md) en [artifact-/bronhashes](visual-artifact-hashes.json) binden40 echte Rust-hostbeelden op 1280×720,1920×1080,1440×900,1024×768 en1920×1080@200%. Geen zichtbare controlclipping in de beoordeelde matrix. Nominale contrastparen: primaire tekst13,49:1, secundair5,23:1, focus6,45:1; geen volledige WCAG-/schermlezerclaim. Tab/ShiftTab/Enter/Escape, labels, focus, keyboardsearch en geen draftverlies zijn getest. Reduced motion: deze slice gebruikt geen animaties. Native screenreader/IME/shaping ontbreken; de editorbody blijft voorlopig monospace voor bestaande caret-/selectiemetrics.200%-klok vergroot bestaande glyphcoverage, zonder nieuwe optische hinting.1920@100%-vensters staan nog te ver uiteen; calculator heeft veel onderruimte. Geen menselijke gebruikstest of subjectieve eindacceptatie als premiumproduct.

## Prestaties — gemeten, geen algemene garantie

Alle native proeven: Linux x86_64, QEMU 10.0.13, `pc-q35-8.2`, `qemu64`,1 vCPU, 256 MiB, TCG single, VGA 16 MiB, displaynone/QMP-captures, eigen loader/OVMF4M, expliciete BIOS/VGA-ROMs, virtio-blk/rng en uitsluitend wegwerpmedia. `/dev/kvm` ontbreekt; KVM staat in de binary maar is hier niet beschikbaar. Geen netwerk, fysieke disk, hostdirectory als beschrijfbare dataopslag of passthrough. Gelijke externe firmwarehashes bij A/B.

| Meting | A:1f/dense | B:2ff/sparse | Betekenis |
|---|---:|---:|---|
|22 fasen totaal |1542,275s|697,463s|54,78% korter in deze afzonderlijke runs |
| Laatste letter→exact volledig appbeeld |62,962s|1,165s|Echte input en appbody-oracle; geen scanoutlatency |
| Opslaan→exact appbeeld |82,667s|3,354s|Echte opgeslagen toestand/oracle |
| Boot→beide complete appframes |98,082s|109,222s|Geen bootwinst |
| Heropenen→exact appbeeld |82,569s|85,842s|Volledige redraw blijft traag |

De cache heeft maximaal 1875 tegelhashes; grotere surfaces vallen expliciet terug op dense rendering. Native 200%-latency is niet gemeten.

Eén run per native arm, deels tegelijk met andere VM’s/hostchecks: geen gecontroleerde p50/p95 of causale claim voor ieder tijdverschil. B is een afzonderlijke beperkte sparsevariant met dezelfde UI als A; oude bronnen worden niet naar de nieuwe UI omgelabeld. Publieke commit1ed83f2 heeft dezelfde source-inputbytes als lokale B2ff. [A-rapport](baseline-native.json), [B-rapport](sparse-native.json).

Op de gecombineerde703-bron slaagt de volledige 22-fasenroute in508,802s. De aparte sessieroute toont interactief lock na8,269s en volledige desktop na111,142s; publieke gastactie→desktop82,049s. **Die laatste wachttijd is nog niet dagelijks bruikbaar.** Er wordt gewacht op echte appframes, niet op een decoratieve timer. De eerdere45s in een previewrunner was een timeout, geen aangetoonde rendererduur.

Nieuwe bootobservaties: firmware-exit/kernel-entry eerst gezien2,184s (samen waargenomen, niet afzonderlijk nauwkeurig gemeten), kernelgeheugen2,692s, procesmanifest3,351s, vereiste services8,265s, calculatorframe53,556s en Materiaalframe103,959s. Dit zijn host-monotone waarnemingsgrenzen. Exacte gast-/IPC-/opslag-/displayfaseverdeling, CPU-/energie-/scanoutkosten en p95 blijven open.

Host-release-letterproef768×536:1608→3 verzonden tegels met exact dezelfde pixels; gewijzigde pixelhoeveelheid is niet native afzonderlijk geïnstrumenteerd. Drie hostrepetities geven mediane update36,032→29,730ms; eerste frame16,666→20,210ms door cachehashkosten. [Hostappmetingen](material-host-benchmark.json).

Hostrenderer op de laatste source-inputbytes, drie afwisselende samples:1280-lock volledige referentie144,446ms→opaque19,592ms; gedeeltelijk paneel7,315ms. Zoeken181,543→59,517ms; paneel29,668ms. Dit zijn CPU-host-renderkosten, geen guestframe-/GPUclaim. [Alle samples,200% en bronbinding](renderer-benchmark.json).

Dominante aangetoonde oorzaak: hele appbeelden en ongewijzigde achtergronden opnieuw renderen/transporteren via vele seriële tegel-IPC’s. Verbeterd: sparse updates, opaque shortcut en paneeldamage. Resterend: volledig eerste appframe, volumestart/IPC, softwarematige displayconversie en observatie/capturekosten. Geen virtio-gpu of doublebufferingclaim zonder gastimplementatie; geen GPU-passthrough gebruikt.

## Reproduceren en fouten behouden

Activeer hier `/workspace/.nieuw-os-env/activate.sh`. De native commando’s hadden exit0:

```sh
NOS_BUILD_DIR=$PWD/build/session-route NOS_TARGET_DIR=$PWD/target/session-route \
  python3 -B tools/build.py --kernel-feature native-session-route
NOS_QEMU_MACHINE=pc-q35-8.2 NOS_BUILD_DIR=$PWD/build/session-route \
  python3 -B tools/run_window_session.py --session-route --timeout 1800 \
    --bios-rom /workspace/.nieuw-os-env/sysroot/usr/share/seabios/bios-256k.bin \
    --vga-rom /workspace/.nieuw-os-env/sysroot/usr/share/seabios/vgabios-stdvga.bin

# In een afzonderlijke schone worktree op dezelfde 703-bron:
NOS_BUILD_DIR=$PWD/build/final-verified NOS_TARGET_DIR=$PWD/target/final-verified \
  python3 -B tools/build.py --kernel-feature native-window-session-probe
NOS_QEMU_MACHINE=pc-q35-8.2 NOS_BUILD_DIR=$PWD/build/final-verified \
  python3 -B tools/run_window_session.py --clipboard --timeout 1800 \
    --bios-rom /workspace/.nieuw-os-env/sysroot/usr/share/seabios/bios-256k.bin \
    --vga-rom /workspace/.nieuw-os-env/sysroot/usr/share/seabios/vgabios-stdvga.bin
```

Hostcommando’s en exitstatus staan volledig in checks.json. Binaries/caches/disks blijven buiten Git; compacte rapporten, serial.gz en screenshots zijn opgenomen.

[Rode voorproeven](preliminary-failures.json) blijven rood: calculatorreadonly-overlap met fontatlas, niet-gecompileerde observaties, verkeerde QEMU-menuqcode (`menu` is XF86MenuKB; PCMenu is `compose`), deels getekende desktop en READY-publicatierace. De correcties verlagen geen deadline of oracle. Eén gecombineerde hosttest had een achterhaalde vaste paneeltegeltelling; de precieze nieuwe 792-telling en alle 58 release-hosttests zijn groen. Native-only Clippy/preflight/fmt groen. Niet uitgevoerd: volledige release-/100boots-/24uurmatrix, fysieke tests, VMware, nieuwe Windowsruntime, native screenreader, echte credentials/providers en menselijke audit; GitHub-CI is apart en niet automatisch groen verklaard.

## Beveiligingsimpact en resterend werk

| Mechanisme | Dreiging/invariant, afdwinging en resterend gat |
|---|---|
| Proces/capability/surface | Kwaadaardige app krijgt geen global framebuffer/inputrecht. Kernel-PID/handle/epoch/bounds en compositorreceiver blijven afdwingen; native isolationprobes en herstel slagen. Fysieke DMA/IOMMU niet bewezen. |
| Trusted UI/input | App mag geen systeemactie uitvoeren. Alleen compositor bezit foreground/controls; ACK precedeert actie, stale offers worden geannuleerd en achtergrondinput wordt geblokkeerd. Hostpixels/native routing bewezen; volledige anti-spoofing-gebruikersstudie en echte grantbroker ontbreken. |
| Auth | Rejection/crash/stale/replay levert geen sessie op. Trusted adapter/broker moet challenge, issuer, limiter en activatie verifiëren; UI buffers worden gewist. Nu hostcontract + gesloten native standaard, geen echte enrollment/provider. |
| Lock/logout | Viewcover/cancel beschermt invoer; dat is geen intrekking van reeds bestaande opslag-/keyringrechten. Alleen openbare testdata, geen persoonlijke keys ontsloten. Werkelijke sessiebroker/revoke blijft P0. |
| Opslag/updates | Bestaande opslag- en updatecontracten ongewijzigd. Echte save/reopen/canaries/cleanup getest; geen extra productie-encryptie-/update-/hardwareclaim. |

- **P0:** echte gebruikers-/auth-/sessiebroker, enrollment/herstel, boot/purpose-gebonden dienstasserties en keyring-/capability-revoke bij lock/logout; kortere eerste appframe-/sessiestart. Dit is nodig voor een echte persoonlijke boot-login-desktoproute.
- **P1:** native resize met nieuwe surfacegeometry/epochs, bredere input/secure-attention, scherpere presentatietimestamps en framebuffer/IPC-profiel; trusted grantbroker gekoppeld aan echte authority. U01 blijft bestaand bewijs, FR-S02 blijft open.
- **P2:** echte materiaalzoekindex, persistente instellingen/notificaties, restart/draft-afhandeling, grotere clipboard/drag/contextmenu, proportionele editor/IME/screenreader en dagelijkse budgetten. U02 niet geaccepteerd.
- **P3:** aparte VMware-platformproef, Windows/WHPX-herhaling van deze bron, GPU/virtio-gpu, USB/FIDO2/biometrie, multi-monitor en fysieke referentiehardware/gebruikerstests.

Twee echte deelagents leverden onderzoek/auth/board en performance/negatieven/native readback. Hun reviews zijn AI-review, geen onafhankelijke menselijke audit. Geen merge naar main. Rollback: revert van de eigen afgebakende commits; geen schijfformaat- of kernel-ABI-migratie en geen destructieve reset nodig.

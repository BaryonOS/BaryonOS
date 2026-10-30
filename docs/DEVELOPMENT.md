# Lokale ontwikkelomgeving en uitvoering

Vastgelegd op 27 oktober 2026. Deze handleiding beschrijft de daadwerkelijk ingerichte Windows-ontwikkelroute. De afzonderlijke build-, QEMU-, product- en quantumresultaten staan in [PROJECT-STATE.md](PROJECT-STATE.md) en hun ruwe rapporten. Beschikbare tools en deze commando's zijn op zichzelf geen geslaagde OS-test.

## Native M6c/M6d op de Linux-ontwikkelvariant

De actuele M6c/M6d-gate gebruikt de hieronder beschreven toolchain en de
kernel-feature `milestone-m6c`. Vanuit de repositoryroot:

```bash
python3 -B tools/preflight.py --strict
python3 -B tools/build.py --kernel-feature milestone-m6c
python3 -B tools/run_m6c.py --case normal --smoke
python3 -B -u tools/test_m6c.py
python3 -B tools/capture_m6c.py --destination "docs/verification/m6d-local-$(python3 -c 'import uuid; print(uuid.uuid4().hex)')"
python3 -B tools/build.py --kernel-feature milestone-m6c
```

De captureopdracht kiest met een nieuwe UUID een eigen directe submap van
`docs/verification/`. Gebruik voor iedere nieuwe uitvoering een nieuwe
bestemming; de standaardbestemming `docs/verification/m6d` bevat geaccepteerd
historisch bewijs en mag niet door een lokale herhaling worden vervangen.

De rooktest start twee geïsoleerde apps voor benoemde objecten, catalogus en
hernoemen. De volledige gate omvat ook echte externe QEMU-kills en koude
recoveries; succes vereist gastuitvoer, raw exit en het onafhankelijke
schijforakel. Iedere nieuwe proef maakt eigen synthetische volumes. Bezet
M6b-v1 wordt geweigerd en niet gemigreerd. Suites delen buildoutputs: voer ze
achtereenvolgens uit. De laatste opdracht herstelt de M6c-build nadat de gate
de gewone M1-boot heeft gecontroleerd; dit is geen extra VM-run.

De actuele M6c/M6d-gate omvat **35 groepen / 31 VM-invocaties**; de afzonderlijke
M6b-regressie omvat **18 groepen / 14 VM-invocaties**. Vastgelegde resultaten,
bronbinding en archieven: [M6D-ACCEPTANCE.md](verification/M6D-ACCEPTANCE.md)
voor de oorspronkelijke Linuxuitvoering en
[M6D-CLOSEOUT.md](verification/M6D-CLOSEOUT.md) voor de aanvullende
Windowsuitvoering op `326d2f6`. De opdrachten hierboven zijn een uitvoerroute,
geen nieuwe testresultaten. De historische M6c-gate van 19 groepen / 15 VM staat
in [M6C-ACCEPTANCE.md](verification/M6C-ACCEPTANCE.md).

## Toolchain en bestaande native M6b-route

Op de M6b-werkbranch is ook een lokale Linux-toolchain ingericht: Rust 1.90.0,
beide bestaande targets, QEMU 10.0.2 uit Debian bookworm-backports en OVMF
2025.02 uit één matching CODE/VARS-pakket. De machine blijft `pc-q35-10.0`,
TCG, `qemu64`, één CPU en 256 MiB. Hashes en officiële herkomst:
`verification/native-linux-toolchain/`. Dit is een afzonderlijk getest
hostprofiel; bestaande Windowsresultaten blijven historisch behouden.

`tools/common.py` ontdekt de toolmap `.tools/nieuw-os/` onder de checkout.
Op andere machines kan de bestaande toolchainroute worden gebruikt. Beide
firmwarevariabelen `OS_OVMF_CODE` en `OS_OVMF_VARS` moeten naar één matching
build wijzen wanneer automatische discovery geen paar vindt. Er zijn geen
globale PATH-/pakket-/shellwijzigingen nodig voor de vastgelegde lokale route.

```bash
python3 -B tools/dev.py cargo +1.90.0 fetch --locked
python3 -B tools/preflight.py --strict
python3 -B tools/build.py --kernel-feature milestone-m6b
python3 -B tools/run_m6b.py --case normal --smoke
python3 -B -u tools/test_m6b.py
python3 -B tools/capture_m6b.py --destination "docs/verification/m6b-local-$(python3 -c 'import uuid; print(uuid.uuid4().hex)')"
```

Op Linux is daarnaast de hostlinker `cc` nodig voor Rust-hosttests; de
kernelcrossbuild gebruikt de linker uit Rust zelf. Controleer die
afhankelijkheid apart met `python3 -B tools/preflight.py --strict-host-tests`.
De bestaande `--strict`-controle blijft uitsluitend de bootgereedschappen en
firmware afdwingen. Op Debian/Ubuntu levert het pakket `build-essential` de
standaard `cc`-driver.

Ook deze capture gebruikt per uitvoering een eigen nieuwe map; het bestaande
`docs/verification/m6b` blijft behouden. In PowerShell zijn de equivalente
captureopdrachten hieronder bruikbaar, telkens direct na de bijbehorende
geslaagde gate en vóór een volgende suite de gedeelde buildoutputs vervangt:

```powershell
python -X utf8 -B tools/capture_m6c.py --destination "docs/verification/m6d-local-$(python -c 'import uuid; print(uuid.uuid4().hex)')"
python -X utf8 -B tools/capture_m6b.py --destination "docs/verification/m6b-local-$(python -c 'import uuid; print(uuid.uuid4().hex)')"
```

De rooktest maakt zelf een nieuw wegwerpvolume en laat twee native apps
objecten aanmaken, wijzigen en verwijderen. Hij sluit de VM na de assertions
af. De complete gate start ook afzonderlijke recovery-VM's op exact de
voorgaande images; de CLI accepteert geen willekeurige bestaande disk.
`M6B:PASS` is alleen geldig samen met de verwachte raw exit en schijforacle.
Na de gate staat de normale M1-build klaar; bouw M6b opnieuw voor een demo.

Voer suites per checkout achtereenvolgens uit: ESP, manifests en outputlocaties
worden gedeeld. Afzonderlijke, bevroren Git-worktrees hebben eigen buildoutputs.
Behoud `build/run/` totdat capture de raw logs, hashes en images heeft nagekeken.

## Werkmap en gereedschappen

Projectroot: `C:\Users\simon\OneDrive\Documenten\ChatGPT\Operating System`.

De aanvullende gereedschappen staan apart onder `C:\Users\simon\AppData\Local\nieuw-os`:

| Onderdeel | Werkelijke inrichting |
|---|---|
| Rust | Exact 1.90.0, host `x86_64-pc-windows-msvc`; rustfmt en clippy |
| Cross-targets | `x86_64-unknown-uefi` en `x86_64-unknown-none` |
| Cargo/rustup | Eigen mappen `cargo/` en `rustup/` onder de toolmap |
| Hostlinker | Bestaande Visual Studio 2026 C++-gereedschappen en Windows SDK; geen extra C++-installatie voor deze route |
| Kernel-/UEFI-linker | Gebundelde Rust LLVM-linker; kernel krijgt uitsluitend zijn eigen ELF-linkopties |
| QEMU | Lokale Windows-package in `qemu/`; binary meldt `11.1.0 (v11.1.0-12130-ge470268ff4)` |
| Firmware | CODE/VARS uit diezelfde QEMU-package, gekoppeld door `share/firmware/60-edk2-x86_64.json` |
| Python, Git en Node | Reeds beschikbare hostgereedschappen |

[tools/common.py](../tools/common.py) maakt uitsluitend een subprocessomgeving: `CARGO_HOME`, `RUSTUP_HOME`, toolpaden en zo nodig de gevonden MSVC-/SDK-librarypaden. Het schrijft geen globale PATH-/shellinstellingen. `RUSTUP_AUTO_INSTALL=0` voorkomt automatische toolchaininstallatie door deze hulpmiddelen. [tools/dev.py](../tools/dev.py) gebruikt dezelfde omgeving voor een losse opdracht.

De eerste uitvoering mag publieke ontwikkeldependencies ophalen. Daarna gebruiken herhaalde builds de exacte lockfiles; `--offline` is beschikbaar zodra de cache gevuld is. Dit downloadpad is ontwikkelgereedschap en vormt geen netwerkfunctionaliteit van het nieuwe OS.

## Aanbevolen uitvoerroute

Open PowerShell in de projectroot. Controleer eerst de inventaris:

```powershell
python tools/preflight.py --strict
```

Dit inventariseert tools, targets en firmware. Het bouwt of boot niets. Exit 2 betekent ontbrekende vereisten; een gevonden tool is nog geen bewezen compatibele buildcombinatie.

Bouw daarna de eigen loader en kernel, inspecteer de geproduceerde PE-/ELF-artifacts en start de rooktest:

```powershell
python tools/build.py
python tools/run.py --smoke
```

De volledige bootacceptatieproef start met:

```powershell
python tools/test.py
```

Die opdracht omvat hosttests, drie koude VM-starts, ontbrekende/corrupte kernel, de test met een verouderde firmwaremapkey, kernelpanic en een echte guest-timeout. De hangproef mag bewust 60 seconden duren. Na de featureproeven wordt de normale build teruggezet. Alleen een actuele suite met `status: completed` en `passed: true` telt als geslaagde volledige run; `running` en `failed` zijn geen groen resultaat.

De afzonderlijke M2-poort bouwt echte foutvarianten en herstelt daarna de gewone M1-image:

```powershell
python -u tools/test_milestones.py --through m2
python tools/capture_gate.py m2
```

`capture_gate.py` controleert ruwe exit/markers, seriële output en artifacthashes voordat het compacte bewijs naar `docs/verification/m2-m4/m2/` kopieert. Het wijzigt geen testuitkomsten. Bestaande oudere runs verwijzen voor de onveranderde VARS-template nog naar de gehashte bron; nieuwe runs bewaren die template ook in hun eigen map.

De volledige M2–M3-poort is uitgevoerd op `6802b17` en start met:

```powershell
python -u tools/test_milestones.py --through m3
python tools/capture_gate.py m2
python tools/capture_gate.py m3
```

Deze reeks voert acht M2- en twintig M3-groepen uit. Na de afzonderlijke foutvarianten herstelt zij de gewone M1-image. Voor de normale M3-ontwikkelproef met eigen gebruikersprocessen bouw je expliciet het M3-profiel en eis je de bijbehorende markers:

```powershell
python tools/build.py --kernel-feature milestone-m3
python tools/run.py --smoke --milestone m3
```

De M3-CLI-poort verlangt daadwerkelijke KS-04/07/08- en M3-markers; een gewone M1-image kan daarmee niet als M3 slagen. De normale M3-proef voert de vaste synthetische programma's uit en sluit QEMU daarna af. Er is nog geen interactieve OS-shell of native Materiaal-desktop.

`userspace/` bevat de eigen statische userprobe en dertien host-artifacttests. `tools/build_users.py` gebruikt directe rustc-argumenten met het userlinkerscript; de Cargo-kernellinkopties worden daarvoor niet hergebruikt. M4 voegt een afzonderlijke IPC-probe toe via `--kind ipc`; dat spoor is pas een behaald resultaat na de eigen M4-poort.

De complete M1–M4-controle is uitgevoerd met deze opdrachten:

```powershell
python -u tools/test_milestones.py --through m4
python -u tools/test.py
python tools/capture_gate.py m1
python tools/capture_gate.py m2
python tools/capture_gate.py m3
python tools/capture_gate.py m4
```

Dat zijn 52 acceptatiegroepen en 44 afzonderlijke VM-runs. De bewuste hang in de M1-regressie duurt 60 s; de negatieve ruwe guestuitkomsten worden bij het kopiëren van bewijs niet in succes veranderd. De actuele suites en expliciete host-/QEMU-grenzen staan in verification/m2-m4/.

Start de normale M4-ontwikkelproef met:

```powershell
python tools/build.py --kernel-feature milestone-m4
python tools/run.py --smoke --milestone m4
```

De kernel voert drie synthetische processen en een normale IPC-exchange uit en beëindigt QEMU na controle. Voor afzonderlijke negatieve cases gebruik je `test_milestones.py`; de normale M4-CLI-poort verlangt de exchange-/survivor-/cleanupmarkers en accepteert een gewone M1-image niet als M4.

De latere bronvingerafdrukversie 2 sluit gegenereerde `verification/`, `build/`, `target/` en caches uit en neemt gebruikersbron mee. Buildmanifesten en guestresultaten vermelden de versie expliciet. Oudere rapporten zonder dit veld behouden hun oorspronkelijke vingerafdruk; artifacthashes blijven rechtstreeks controleerbaar.

Een losse ontwikkelopdracht in dezelfde omgeving:

```powershell
python tools/dev.py cargo +1.90.0 test --locked -p boot-protocol -p elf-loader-core
python tools/dev.py rustc +1.90.0 -vV
```

Loader en kernel gebruiken afzonderlijke targets. Instellen van één globale workspace-target zou de hosttests verstoren en hoort niet bij deze inrichting.

## M5-blokdienst starten en controleren

Bouw expliciet het M5-profiel en start de eindige ontwikkelproef:

```powershell
python tools/build.py --kernel-feature milestone-m5
python tools/run.py --smoke --milestone m5
```

De runner maakt per start een nieuwe synthetische 64KiB-image onder `build/run/`, koppelt die alleen-lezen aan modern virtio-blk op `00:05.0` en bewaart de hele commandoregel, serial, stderr, firmware-/binary-/imagehashes en het externe datacontrole-resultaat. De aparte boot-ESP blijft een ander device. Er is geen parameter voor een echte hostdisk. De dienst leest sector 4 en 6 via IPC, waarna QEMU na de assertions afsluit. Dit is nog geen interactieve OS-shell.

De M5-poort voert hosttests, ELF-controles, dezelfde kernel op twee verschillende images, 17 foutvarianten en een M1-regressiestart uit:

```powershell
python -u tools/test_m5.py
python tools/capture_gate.py m5
```

Voor de gecombineerde M2–M5-regressie en de volledige bootfoutreeks:

```powershell
python -u tools/test_milestones.py --through m5
python -u tools/test.py
```

Deze suites herstellen aan het einde de gewone M1-build. Bouw daarna opnieuw `milestone-m5` voor een losse M5-start. Gebruik voor foutfeatures de gate met bijbehorende casemarkers; `run.py --milestone m5` verlangt bewust de normale READ-case. Alleen `status: completed` en `passed: true`, plus opnieuw gecontroleerde ruwe evidence, tellen als geslaagde gate. Actueel uitgevoerd bewijs staat in PROJECT-STATE.md.

Het dienstprotocol en alle injectiegrenzen staan in [M5-BLOCK-SERVICE.md](implementation/M5-BLOCK-SERVICE.md). Het kerneltransport en het protocolconforme device zijn vertrouwd; M5 levert geen IOMMU-containment of fysieke-hardwarebewijs. Bij ontbrekende resetack blijft de vaste DMA-pool in quarantaine tot VM-stop.

## Windows: exclusieve testimages buiten synchronisatiemappen

Voer schrijvende M6-gates uit in een aparte Git-worktree buiten gesynchroniseerde
mappen. Op 28 oktober 2026 wees de single-linkguard een echte normale M6c-run
terecht af: tijdens uitvoering ontstond een tweede hardlink naar dezelfde image
onder `.tmp.driveupload`. `fsutil hardlink list` bevestigde beide namen; welk
proces die link maakte is niet vastgesteld. Het guestresultaat raw 33 en alle
succesmarkers maakten deze hostgrensfout niet groen.

Een afzonderlijke worktree behoudt dezelfde commit en bronfingerprint. Bewaar
ruwe images uitsluitend in haar eigen `build/run`; neem alleen controleerbare
tekst-/hashrapporten in Git op. De `st_nlink == 1`-voorwaarde blijft verplicht;
geen uitzondering voor syncsoftware of retry die de waargenomen tweede link
verbergt. Pas geen sync-, netwerk- of andere hostinstellingen aan voor de proef.
De mislukte run en de exacte hardlinkobservatie staan onder
`docs/verification/m6d-closeout/`.

## Vast QEMU-profiel

De lokale ontwikkelvariant gebruikt `pc-q35-10.0`, `qemu64`, TCG, één CPU, 256 MiB RAM, geen gastnetwerk en geen hostschijf. De test maakt een eigen ESP-kopie en een verse VARS-kopie in `build/run/`; CODE wordt read-only gebruikt. De concrete argumenten, ruwe exitstatus, hashes en seriële output staan per run in `result.json`, `serial.txt` en `stderr.txt`.

De normale succesobservatie vereist complete regels in volgorde: `BOOT:EBS_OK`, `KERNEL:ENTRY`, `BOOTINFO:OK`, `M1:PASS`, plus ruwe QEMU-exit 33 en geen foutmarker. Een normale hostexit 0 of alleen een firmwarebanner geldt niet als kernelboot. Een fout gebruikt in dit testprofiel ruwe QEMU-exit 35; timeout en ontbrekende markers blijven fouten.

De startdocumenten noemden QEMU 10.0.14 en zelf gebouwde EDK II `edk2-stable202508` als referentie. De huidige Windows-package en meegeleverde firmware zijn **niet byte-identiek aan die referentie**. De exacte firmwarebuildcommit is niet onafhankelijk vastgesteld; de descriptor, lokale hashes en packageherkomst zijn wel vastgelegd. Dit betreft een ontwikkelprofiel zonder Secure Boot-bewijs.

**Afgesproken profielcorrectie tijdens integratie:** de hoofdagent heeft vanwege geobserveerde firmwareoverlap de kernelreservering **32–48 MiB** (`[0x02000000, 0x03000000)`) en **BootInfo ABI 1.1** aangewezen als vervanging van de oorspronkelijke 2–18 MiB/ABI 1.0. Dit is een expliciete wijziging van de bouwbasis. Deze correctie is geïmplementeerd en de volledige bootacceptatiereeks is geslaagd. Het besluit en uitvoeringsbewijs staan in [DECISIONS.md](DECISIONS.md) en [PROJECT-STATE.md](PROJECT-STATE.md).

## Productprototype en quantumproef

Start Materiaal uitsluitend op de lokale loopbackinterface:

```powershell
python -m http.server 8765 --bind 127.0.0.1 --directory ui/prototype
```

Open [Materiaal op localhost](http://127.0.0.1:8765/). De eindcontrole na hervatten gebruikte poort 8776, omdat 8765 lege responses gaf. Gebruik zo nodig dezelfde opdracht met 8776 en open http://127.0.0.1:8776/. Stop de server met `Ctrl+C`. Dit is het browserprototype met fictieve gegevens; het draait nog niet op de eigen kernel. De uitgebreide bediening en bekende bewijsgrenzen staan in [ui/prototype/README.md](../ui/prototype/README.md).

Controleer de quantumproef en voer de Bellfixture uit:

```powershell
python experiments/quantum/verify.py
Get-Content -Raw experiments/quantum/fixtures/bell8.json | python tools/dev.py cargo +1.90.0 run --manifest-path experiments/quantum/Cargo.toml --locked --offline -p qsim-cli
```

De quantumworkspace staat buiten de kernelworkspace. `verify.py` schrijft uitvoer, bron-/binaryhashes en exacte commandoregels naar `experiments/quantum/verification/`. Het is lokale hostsimulatie; geen fysieke QPU, eigen-OS-port of cryptografische toevalsgenerator. Zie [de quantumhandleiding](../experiments/quantum/README.md).

## Herkomst en hashes

De waarden hieronder komen uit het daadwerkelijk aangemaakte [build/toolchain-provenance.json](../build/toolchain-provenance.json). Hashes identificeren de gebruikte bestanden; zij bewijzen geen foutloze of veilige software.

| Bestand | Algoritme | Vastgelegde hash |
|---|---|---|
| rustup-init.exe | SHA-256 | `00f139da614dced87d3c3d4121c43eebdac3c1bf8a5ee86b9d80dfd71c8d70fd` |
| qemu-w64-setup-20260811.exe | SHA-512 | `5bcf9eed634e8575a37b74f445af41a2fe4106da512d0c30c368301d4c105037fdfab40a5287367a28a957624cddebbc8c07e16c88ab6634f554cdf3d16bf543` |
| edk2-x86_64-code.fd | SHA-256 | `33090cc07675baa5190d9f1e84bf5176b33bcbfa9bacac522961150cdb6dbb2a` |
| edk2-i386-vars.fd, template | SHA-256 | `5d2ac383371b408398accee7ec27c8c09ea5b74a0de0ceea6513388b15be5d1e` |

Rust verwijst op zijn [officiële installatiepagina](https://rust-lang.org/tools/install/) naar [de gebruikte x64-rustupdownload](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe). De [officiële QEMU-downloadpagina](https://www.qemu.org/download/#windows) verwijst voor Windows naar Stefan Weil; de gebruikte package is [qemu-w64-setup-20260811.exe](https://qemu.weilnetz.de/w64/qemu-w64-setup-20260811.exe). Deze verwijzingen zijn op 27 oktober 2026 gecontroleerd. Dit zijn herkomstverwijzingen, geen opdracht om opnieuw te installeren.

De loaderdependency `r-efi 7.1.0` levert uitsluitend UEFI-typen/constanten. Herkomst, Cargo-checksum en de gekozen MIT-route staan in [third-party/README.md](third-party/README.md).

## Herstel en opruimen

Behoud de bronbestanden, Git-commits, lockfiles en gewenste bewijsrapporten. `build/`, de root-`target/` en `experiments/quantum/target/` bevatten gegenereerde bestanden die met bovenstaande commando's opnieuw kunnen worden gemaakt. Bewaar `build/toolchain-provenance.json` en relevante `build/run/`-rapporten als het bewijs later controleerbaar moet blijven; opnieuw bouwen reconstrueert niet de oude uitvoering.

Stop eerst QEMU, de lokale webserver en lopende Cargo-processen. De afzonderlijke toolmap `C:\Users\simon\AppData\Local\nieuw-os` kan daarna desgewenst worden verwijderd nadat het exacte pad en eventuele nog gewenste downloads/logs zijn gecontroleerd. Dat verwijdert de projectlokale compiler/cache/emulator, niet de bronrepository of bestaande Visual Studio-installatie. De ontwikkelcommando's werken vervolgens pas weer nadat de vereiste tools opnieuw beschikbaar zijn. Deze handleiding voert geen verwijdering uit.

Er is geen fysieke schijf als VM-disk gebruikt, geen firmware geflasht en geen NAS-configuratie gewijzigd. Een bronrollback gebeurt met behoud van werk en geschiedenis, bijvoorbeeld via een gerichte revert van een later gemaakte implementatiecommit; geen destructieve reset van gebruikerswerk.

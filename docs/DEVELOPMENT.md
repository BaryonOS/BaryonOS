# Lokale ontwikkelomgeving en uitvoering

Vastgelegd op 27 oktober 2026. Deze handleiding beschrijft de daadwerkelijk ingerichte Windows-ontwikkelroute. De afzonderlijke build-, QEMU-, product- en quantumresultaten staan in [PROJECT-STATE.md](PROJECT-STATE.md) en hun ruwe rapporten. Beschikbare tools en deze commando's zijn op zichzelf geen geslaagde OS-test.

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

Dat zijn52 acceptatiegroepen en44 afzonderlijke VM-runs. De bewuste hang in de M1-regressie duurt60s; de negatieve ruwe guestuitkomsten worden bij het kopiëren van bewijs niet in succes veranderd. De actuele suites en expliciete host-/QEMU-grenzen staan in verification/m2-m4/.

Start de normale M4-ontwikkelproef met:

```powershell
python tools/build.py --kernel-feature milestone-m4
python tools/run.py --smoke --milestone m4
```

De kernel voert drie synthetische processen en een normale IPC-exchange uit en beëindigt QEMU na controle. Voor afzonderlijke negatieve cases gebruik je `test_milestones.py`; de normale M4-CLI-poort verlangt de exchange-/survivor-/cleanupmarkers en accepteert een gewone M1-image niet als M4.

De latere bronvingerafdrukversie2 sluit gegenereerde `verification/`, `build/`, `target/` en caches uit en neemt gebruikersbron mee. Buildmanifesten en guestresultaten vermelden de versie expliciet. Oudere rapporten zonder dit veld behouden hun oorspronkelijke vingerafdruk; artifacthashes blijven rechtstreeks controleerbaar.

Een losse ontwikkelopdracht in dezelfde omgeving:

```powershell
python tools/dev.py cargo +1.90.0 test --locked -p boot-protocol -p elf-loader-core
python tools/dev.py rustc +1.90.0 -vV
```

Loader en kernel gebruiken afzonderlijke targets. Instellen van één globale workspace-target zou de hosttests verstoren en hoort niet bij deze inrichting.

## QEMU-profiel en bewijs

De lokale ontwikkelvariant gebruikt `pc-q35-10.0`, `qemu64`, TCG, één CPU, 256 MiB RAM, geen gastnetwerk en geen hostschijf. De test maakt een eigen ESP-kopie en een verse VARS-kopie in `build/run/`; CODE wordt read-only gebruikt. De concrete argumenten, ruwe exitstatus, hashes en seriële output staan per run in `result.json`, `serial.txt` en `stderr.txt`.

De normale succesobservatie vereist complete regels in volgorde: `BOOT:EBS_OK`, `KERNEL:ENTRY`, `BOOTINFO:OK`, `M1:PASS`, plus ruwe QEMU-exit 33 en geen foutmarker. Een normale hostexit 0 of alleen een firmwarebanner geldt niet als kernelboot. Een fout gebruikt in dit testprofiel ruwe QEMU-exit 35; timeout en ontbrekende markers blijven fouten.

De startdocumenten noemden QEMU 10.0.14 en zelf gebouwde EDK II `edk2-stable202508` als referentie. De huidige Windows-package en meegeleverde firmware zijn **niet byte-identiek aan die referentie**. De exacte firmwarebuildcommit is niet onafhankelijk vastgesteld; de descriptor, lokale hashes en packageherkomst zijn wel vastgelegd. Dit betreft een ontwikkelprofiel zonder Secure Boot-bewijs.

**Afgesproken profielcorrectie tijdens integratie:** de hoofdagent heeft vanwege geobserveerde firmwareoverlap de kernelreservering **32–48 MiB** (`[0x02000000, 0x03000000)`) en **BootInfo ABI 1.1** aangewezen als vervanging van de oorspronkelijke 2–18 MiB/ABI 1.0. Dit is een expliciete wijziging van de bouwbasis. Deze correctie is geïmplementeerd en de volledige bootacceptatiereeks is geslaagd. Het besluit en uitvoeringsbewijs staan in [DECISIONS.md](DECISIONS.md) en [PROJECT-STATE.md](PROJECT-STATE.md).

## Productprototype en quantumproef

Start Materiaal uitsluitend op de lokale loopbackinterface:

```powershell
python -m http.server 8765 --bind 127.0.0.1 --directory ui/prototype
```

Open [Materiaal op localhost](http://127.0.0.1:8765/). De eindcontrole na hervatten gebruikte poort8776, omdat8765 lege responses gaf. Gebruik zo nodig dezelfde opdracht met8776 en open http://127.0.0.1:8776/. Stop de server met `Ctrl+C`. Dit is het browserprototype met fictieve gegevens; het draait nog niet op de eigen kernel. De uitgebreide bediening en bekende bewijsgrenzen staan in [ui/prototype/README.md](../ui/prototype/README.md).

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

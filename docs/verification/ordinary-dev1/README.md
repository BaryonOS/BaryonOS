# Gewone interactieve DEV1: bron- en artifactfreeze

**Ontwikkelkandidaat geblokkeerd: eerste referentiegate mislukt, geen bundel.**
De door integrator e8 aangewezen huidige basis is main
`ca45a7dabe6a040921de03bdc2fad3e5045a75ba`. De eerste, mislukte build staat onveranderd in
de aparte detached worktree `build/release-dev1-source`; de gecorrigeerde
buildisolatie hieronder gebruikt een tweede onveranderde ca45-worktree
buiten de Cargo-configuratievoorouders. Deze bewijsbranch
bevat alleen documentatie/gegenereerde tekstrapporten, geen runtimepatch.
NAS-intake blijft afzonderlijk bewaard op `simon/native-nas-release-intake`
en concept-PR #83; die is niet in deze kandidaat opgenomen.

Bronvingerafdruk v3:
`bda48571144d7771bff145cc17b0f0712e271cbc6ee323dec43bee099ea5e89d`.
Dit is een nieuw lokaal gebouwd artifact, niet een herlabelde eabc-/R77-
of UI0818-bundel. Het ontbreken van volledige 62-rijenacceptatie verhindert
een expliciet beperkte ontwikkelbundel niet. De volledige opdracht blijft
wel open; dit document vervangt geen releasecriteria of centrale roadmap.

## Profiel en grenzen

Gewone variant: `native-system-materiaal`, zeven app-ELFs, **zonder**
desktop-controller. Referentie: apart `native-system-materiaal-probe`,
acht app-ELFs, uitsluitend pixelorakel; diens bediening geldt niet als
gebruikerpass. Beide builds gebruiken dezelfde zeven gebruikersprogramma's
en framebufferloader. De gewone gast moet de echte QMP-toets-/muisflow
maken/bewerken/opslaan/lijst/heropenen zelf uitvoeren.

Vast DEV-profiel vóór VM-uitvoering: `pc-q35-8.2`, `qemu64`,
1 vCPU, TCG, 256 MiB, 800x600, netwerk uit. Dit wijkt opzettelijk af van
het geplande VM-FR/Nehalem/8GiB/32GiB-profiel en levert daarvoor geen claim.
Het machineprofiel is gelijk aan dat van de bestaande Windowspackager.
De Linux-QEMU- en Windows-QEMU-uitvoering blijven afzonderlijke gates.
Alleen nieuw aangemaakte synthetische wegwerpdisks; geen persoonlijke data.

De twee geplande gates worden strikt serieel op hetzelfde ene runnerslot
uitgevoerd, pas na artifactfreeze, verse headroomcontrole en registrar12-ACK:
eerst één referentie, daarna één gewone preview, maximaal 240 s per gate.
Geen cold-VM, automatische herhaling, parallelle VM of extra 20-slot.
QMP-quit is **hoststop**, geen guest clean shutdown of bewijs voor
abrupte stroomuitval tijdens schrijven.

## Werkelijk uitgevoerd vóór VM

Rust 1.90.0, bestaande toolchains/caches; geen installatie of manifestwijziging.

| Commando | Resultaat |
|---|---|
| `python3 -B tools/dev.py python3 -B tools/preflight.py --strict` | exit 0; inventariscontrole, geen gast |
| `NOS_BUILD_DIR="$SOURCE/build/dev1-reference" python3 -B tools/dev.py python3 -B -u tools/build.py --kernel-feature native-system-materiaal-probe` eerste poging | exit 1 op wrapperniveau, onderliggende Cargo exit 101; bestaande fontdependency ontbrak offline |
| hetzelfde reference-buildcommando na cacheherstel | exit 0; acht ELFs en loader/kernel geïnspecteerd |
| `NOS_BUILD_DIR="$SOURCE/build/dev1-ordinary" python3 -B tools/dev.py python3 -B -u tools/build.py --kernel-feature native-system-materiaal` | exit 0; zeven ELFs en loader/kernel geïnspecteerd |
| bestaande `package_windows_qemu.validate_build` op ordinary build | geslaagd; bron/manifest, precies zeven app-ELFs, ELF-inspectie/embedding en ESP gecontroleerd; geen ZIP of VM |

De eerste fout bleef bewaard in `logs/reference-build-missing-cache.txt`.
Alleen de reeds gepinde `noto-sans-mono-bitmap` 0.3.2 werd vanuit een
bestaande read-only donorcargocache naar de eigen cache hersteld.
Cratehash vóór en na kopiëren exact gelijk aan Cargo.lock:
`f102c477fd4ec864c149803bf606ecc23bc9329602ec0f99073a728abf454abc`.
Geen netwerkdownload, nieuw versienummer, dependency of lockfilewijziging.
Daarna beide builds exit 0, samen 9 s wallclock. Volledige commandouitvoer,
preflight en beide buildmanifests zijn opgeslagen.

| Artifact | SHA-256 |
|---|---|
| Gewone kernel | `c8f7e9397df68b7432cc5093c71e4b9b6da1e63014b5d5ae9f034d4a875b9ca2` |
| Referentiekernel, niet afleveren als gewone image | `a5e85c765c32d7ded2bd21149068a40db9d236266b978bae9e3012660d56e415` |
| Loader beide | `eab80a070f6f385d1d8df06b2295c72e12369942f7f015b998fb806724dc0bef` |
| Gewoon buildmanifest | `741f28717f2e74d9b7edef8e66085603297859e436dfe237915dd2575b810982` |
| Referentiemanifest | `8bd341451db3e5811f6c2ba979a5834568ba4bc5b8293f44499daab07369d164` |

Een monolithische release-diskimage of ZIP bestaat nog niet. De build is een
gecontroleerde ESP-boom plus zeven ingesloten apps; de runner maakt een
verse synthetische 1 MiB-disk. Alle tool-/ROM-/firmwarepaden en hashes staan
in `build-freeze.json`, inclusief werkelijk QEMU-ELF
`fa552a45c337a88fe71e1e1af9ebace875f30e88685a6986d9406c92efaf44cb`.

## Exacte oorspronkelijke omgeving en commando's

Vanuit de bevroren source-worktree, met bestaande read-only hosttools:

```sh
RELEASE_ROOT=/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-refactored-pancake
SOURCE="$RELEASE_ROOT/build/release-dev1-source"
HOST_TOOLS=/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-didactic-spork/.tools/nieuw-os/host-tools
QEMU_TOOLS=/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-studious-sniffle/.tools/nieuw-os/qemu
export CARGO_HOME="$RELEASE_ROOT/.tools/nieuw-os/cargo"
export RUSTUP_HOME="$RELEASE_ROOT/.tools/nieuw-os/rustup"
export PATH="$CARGO_HOME/bin:$QEMU_TOOLS/usr/bin:$HOST_TOOLS/usr/bin:$PATH"
export LD_LIBRARY_PATH="$QEMU_TOOLS/usr/lib/x86_64-linux-gnu:$HOST_TOOLS/usr/lib/x86_64-linux-gnu:$HOST_TOOLS/lib/x86_64-linux-gnu"
export OS_OVMF_CODE="$HOST_TOOLS/usr/share/OVMF/OVMF_CODE_4M.fd"
export OS_OVMF_VARS="$HOST_TOOLS/usr/share/OVMF/OVMF_VARS_4M.fd"
export NOS_TARGET_DIR="$SOURCE/target/dev1"
export CARGO_TARGET_DIR="$SOURCE/target/dev1"
export CARGO_NET_OFFLINE=true
cd "$SOURCE"
```

Deze commando's waren bij freeze `b4a93e43` nog niet uitgevoerd.
Daarna is alleen het eerste commando eenmaal uitgevoerd; het tweede niet:

```sh
python3 -B tools/dev.py python3 -B -u tools/run_materiaal_probe.py --build-dir "$SOURCE/build/dev1-reference" --case normal --machine pc-q35-8.2 --vga-rom "$QEMU_TOOLS/usr/share/seabios/vgabios-stdvga.bin" --vga-rom-sha256 c944f5fd404a6553a32e1e0527081d40e6040d3ce1740d39766bff01871e4fde --timeout 240
python3 -B tools/dev.py python3 -B -u tools/run_materiaal_preview.py --build-dir "$SOURCE/build/dev1-ordinary" --interactive-reference-run <WERKELIJK_GESLAAGDE_REFERENTIERUN> --machine pc-q35-8.2 --timeout 240 --bios-rom "$QEMU_TOOLS/usr/share/seabios/bios-256k.bin" --vga-rom "$QEMU_TOOLS/usr/share/seabios/vgabios-stdvga.bin"
```

De tweede opdracht krijgt pas het concrete runpad na de eerste echte gate.
Niet vervangen door een historische run of UI0818: source-/app-/firmware-/
QEMU-binding moet op deze eigen artifacts slagen.

## Eerste native gate: mislukt en behouden

Registrar12 gaf na een verse hostcontrole expliciet hetzelfde ene slot vrij
voor maximaal twee seriele gates, de tweede uitsluitend na een echte PASS
van de eerste. De referentie eindigde na **3,180991606 s**, CLI exit **1**,
QEMU raw **0**, `passed:false`, `timed_out:false`, `guest_timeout:false`.
Fout: `ValueError: Guest did not complete actual UI phase ready`.
Run: `build/release-dev1-source/build/dev1-reference/run/materiaal-normal-ac4f038e86`.
Exacte CLI-uitvoer: `logs/reference-run.txt`; oorspronkelijk
`/tmp/nieuw-os-dev1-ca45-reference-run.log`.

Serial bereikt `BOOT:EBS_OK`, `KERNEL:ENTRY`, `BOOTINFO:OK`, `KS-01:PASS`,
`KS-02:PASS`; geen `KS-03`, Materiaal-manifest, screenshot of UI-fase.
QMP bevat alleen de geslaagde `qmp_capabilities`-onderhandeling, geen
gebruikersinvoer. Stderr meldt een ontbrekende `kvmvapic.bin`; dat is
vastgesteld, maar niet als oorzaak bewezen. Er is **geen gewone preview**
gestart en geen automatische herhaling, cold-VM of andere VM. Postexit waren
geen `qemu-system`-processen zichtbaar; het slot is expliciet teruggegeven
en door12 als `FREE_OWNER_REPORTED` bevestigd. De voorwaardelijke tweede
toestemming is daarmee vervallen.

`reference-ac4f038e86/` bewaart result, serial, stderr, QMP en diskmetadata
bytegelijk. De offline integriteitscontrole is exit0: alle23 gerapporteerde
artifacthashes, bevroren inputs, huidige QEMU-identiteiten en opnieuw
geinspecteerde build/ingesloten ELFs kloppen. De drie diskbeelden zijn
bytegelijk aan de canonieke lege synthetische disk, inclusief canaries.
Dit is **integriteitsbewijs van een mislukking**, geen geslaagde native gate.
Grote binaries/disks blijven uitsluitend in de oorspronkelijke buildboom;
hun hashes staan in het oorspronkelijke result. `build-freeze.json` blijft
het ongewijzigde historische pre-VM-record.

### Offline oorzaak en begrensde hervatstap

Beide nieuwe kernels hebben ongeldige bootstrapgrenzen:
`__kernel_start == __text_end == __rodata_end == __kernel_end == 0x02000000`,
terwijl hun ELF-secties wel degelijk code/data erboven bevatten.
De reference-disassembly berekent bij `0x02015f54` een kernelbereik met
lengte nul. Daardoor wordt de eigen kernel niet in zijn nieuwe pagetabel
opgenomen. Dit is een concrete buildfout die de waargenomen vroege stop
verklaart; een exacte CPU-exceptiontrace is niet opgenomen.

De bewaarde Cargo-fingerprints van beide kernels tonen de volledige
`rustflags`-reeks **tweemaal**, inclusief tweemaal `-Tkernel/linker.ld`.
De nieuwe worktree staat onder de oorspronkelijke repository en Cargo
combineert de `.cargo/config.toml` van beide voorouders. Het linkerscript
wordt zo tweemaal toegepast; de tweede lege sectiepassage overschrijft de
grenssymbolen. De eerder werkelijk geteste NAS/default-kernels hebben wel
oplopende grenssymbolen. Dit bewijst geen algemene compilerfout en zegt
niets over de afzonderlijk onderzochte TCP-CR3-fout.

Volgende onafhankelijke stap: dezelfde onveranderde bronca45 buiten deze
Cargo-configuratievoorouders uitchecken, bestaande toolchains/caches
hergebruiken en beide builds opnieuw uitvoeren. Voor een nieuwe freeze
eerst precies een kernelflagreeks en geordende ELF-grenzen aantonen.
Geen bron-/kernelpatch, Cargo-clean, nieuwe dependency of verzwakking van
de gate. Nieuwe artifacts krijgen nieuwe hashes; deze mislukte artifacts
worden niet herlabeld. Pas na die nieuwe freeze volgt een nieuwe
registrarcapaciteitsaanvraag; dit checkpoint geeft **geen VM-vrijgave**.

## Overdracht en open acceptatie

4 bezit deze kandidaatfreeze; e8 blijft enige operationele integrator.
12 regelt het ene runnerslot; 20 doet begrensde producttest; 21 bezit de
bestaande Windowspackager/launcherroute. Geen tweede kernel-/ABIbuilder.
Er is nog geen artifactvrijgave voor 20/21 zolang de gewone gate niet is
uitgevoerd. De packager moet vanaf exact de vrijgegeven bron worden
aangeroepen: hij neemt zijn `source_commit` uit zijn eigen checkout.

Niet uitgevoerd: gewone gastbediening, cold recovery op deze bron,
Windowsverpakking/-gast, faultfixtures Full/Unknown, menselijke beoordeling,
fysieke installatie, 100 koude productstarts en 24 uur productgebruik.
Geen positieve FR-rij of R01-R03-punt; scoreadvies **34 -> 34, R01-R03 0/5**.
De manager beheert centrale status/claims/progress en alle 62 criteria
blijven behouden. Voor een nieuw schrijfgebied gaat een concreet verzoek
naar de assistent-teamleider; onbereikbaarheid is geen scope-uitbreiding.

## Gecorrigeerde buildisolatie: nieuwe freeze, nog geen nieuwe VM

`isolated-build-1/build-freeze.json` vervangt alleen de kandidaat-artifacts,
niet het mislukte bewijs hierboven. De bron blijft exact ca45/v3bda485.
Nieuwe detached source:
`/home/simon/.copilot/session-state/5d1e3eef9eb44e7faae099eda4e9207d/files/dev1-ca45-source`.
Deze locatie erft niet de `.cargo/config.toml` van de oorspronkelijke
repository. Geen bronpatch, dependency-/lockfilewijziging of toolinstallatie.

Een eerste buildaanroep weigerde terecht de oude targetcache buiten deze
nieuwe checkout: `ValueError: FPU artifact escaped checkout`, exit1, nog
geen Cargo-build of VM. Het exacte log blijft in `logs/cache-path-rejected.txt`.
Daarna is alleen de eigen bestaande `target/dev1`-cache verplaatst naar de
nieuwe checkout; geen symlink, guardomzeiling, `cargo clean` of herinstallatie.
De eerste mislukte ESP, buildmanifests en run blijven op hun oude plaats.

Beide bestaande buildcommando's, met `NOS_BUILD_DIR` onder deze nieuwe
source en `NOS_TARGET_DIR/CARGO_TARGET_DIR="$SOURCE/target/dev1"`, eindigen
nu exit0. De bestaande strict preflight eindigt ook exit0.
De opgeslagen kernelfingerprints zijn exact gelijk aan **eenmaal** de
rustflags uit `.cargo/config.toml`, inclusief eenmaal het linkerscript.
De ELF-grenzen lopen correct op:
reference `02000000 < 02027000 <= 020f0000 < 0251a000`;
ordinary `02000000 < 02025000 <= 020e5000 < 0250f000`.
Dit bevestigt het herstel van de buildfout, nog niet van de gastuitvoering.

| Nieuwe artifact | SHA-256 |
|---|---|
| Reference kernel | `cf40deda4f246a60d87361894785d08467ede30b6135ca8fcb2059eb7f84734b` |
| Ordinary kernel | `98a6a06adc57daea6267fa78bf149b879385c0d37bd70d28a324dfb61877bb87` |
| Loader beide | `41c2336ec4c71489e235f10dc2feb354359d1d65b53c6a497d19a4d1e29bf5c7` |
| Reference manifest | `12a8c691aac3cbe588190953e4d85afaffbdf6444e48243dd7359286b36979c1` |
| Ordinary manifest | `b67d6a7c19715e8e60c85f83d380b628ac5689f26104b68e25fb903502adc393` |

De eerste zeven ELFs zijn exact gelijk tussen beide nieuwe builds.
Bestaande reference-inspectie en gewone packagerinspectie zijn opnieuw
uitgevoerd; firmware, ROMs en echte QEMU-ELF behouden hun eerder
vastgelegde hashes. Alle zestien bestaande commandorecords, beide
manifests, fingerprints, grenssymbolen en buildlogs zijn opgeslagen.
De eerste verwerking van inspectie-JSON als commandolog gaf een
`KeyError: 'returncode'`; deze metadatafout staat apart in de freeze.
Daarna zijn precies de acht bestaande commandologs per build geselecteerd,
alle exit0. Daarvoor is geen build of VM herhaald.

Aanvullend op dezelfde onveranderde geisoleerde bron uitgevoerd:
`python3 -B tools/dev.py python3 -B -m unittest tools.tests.test_materiaal_probe tools.tests.test_materiaal_preview`:
**exit0, 45 hosttests, 4,093 s**. Uitvoer:
`isolated-build-1/logs/runner-hosttests.txt`. Dit controleert de bestaande
runnerlogica; het is geen vervanging van de nieuwe native gates.

**Nieuw slot nog niet vrijgegeven.** De oude voorwaardelijke toestemming
is vervallen. Pas na deze gepushte freeze en een nieuwe verse
registrarcapaciteitscontrole mogen opnieuw maximaal twee seriele gates
worden gestart; geen automatische herhaling.
Met de eerder beschreven tool-/firmwareomgeving en deze nieuwe
`SOURCE`/targetpaden zijn de voorgenomen commando's:

```sh
python3 -B tools/dev.py python3 -B -u tools/run_materiaal_probe.py --build-dir "$SOURCE/build/dev1-reference" --case normal --machine pc-q35-8.2 --vga-rom "$QEMU_TOOLS/usr/share/seabios/vgabios-stdvga.bin" --vga-rom-sha256 c944f5fd404a6553a32e1e0527081d40e6040d3ce1740d39766bff01871e4fde --timeout 240
python3 -B tools/dev.py python3 -B -u tools/run_materiaal_preview.py --build-dir "$SOURCE/build/dev1-ordinary" --interactive-reference-run <NIEUWE_WERKELIJK_GESLAAGDE_REFERENTIERUN> --machine pc-q35-8.2 --timeout 240 --bios-rom "$QEMU_TOOLS/usr/share/seabios/bios-256k.bin" --vga-rom "$QEMU_TOOLS/usr/share/seabios/vgabios-stdvga.bin"
```

Geen oude reference gebruiken. Ordinary, cold, Windows, FR en R01-R03
blijven open; scoreadvies **34 -> 34, R01-R03 0/5**. De centrale eigenaar
houdt status en assessment bij; deze branch schrijft die bestanden niet.

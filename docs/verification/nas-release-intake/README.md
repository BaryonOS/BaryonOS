# NAS release-intake: begrensd opt-in profiel

**Tussenstand, geen VM-/releaseacceptatie.** Intakebron
`b6ec464b50325e90d2b949aa6f86a93eee7b24b3` behoudt main
`ca45a7dabe6a040921de03bdc2fad3e5045a75ba` en NAS-freeze
`ce52dd6165ba7af3bdcd1533308b556d1927443d`. Alle niet-NAS-bestanden zijn bij
intake exact main; de aparte NAS-workflow is de enige uitzondering.
V3-bronfingerprint bij intake:
`7490e7ea0334996297db54a171b710f474d564b3b8d0a9fd272976bd95ab1d2b`.

## Werkelijk uitgevoerd op intakebron

Bestaande read-only host-tools uit de UI-worktree, pinned Rust 1.90.0,
geen installatie of gedeelde targetwriter.

| Commando | Resultaat |
|---|---|
| `NOS_BUILD_DIR=build/nas-release-intake python3 -B tools/dev.py python3 -B services/nas/native-probe/build.py --target-dir target/nas-release-intake-native --output-dir build/nas-release-intake/users` | exit 0; twee ELF's gebouwd en geïnspecteerd |
| `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path services/nas/native-core/Cargo.toml` | exit 0; 21 tests |
| `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path services/nas/native-volume/Cargo.toml` | exit 0; 9 tests |
| `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path services/nas/native-probe/Cargo.toml` | exit 0; 5 tests |

Voor elk van die drie manifests ook daadwerkelijk uitgevoerd:
`python3 -B tools/dev.py cargo +1.90.0 clippy --offline --locked --manifest-path <manifest> --all-targets -- -D warnings`
en
`python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path <manifest> -- --check`:
alle zes exit 0. Eigen hosttarget: `target/nas-release-intake-host`.

| Nieuw gebouwd artifact | Bytes | SHA-256 |
|---|---:|---|
| `build/nas-release-intake/users/nas-probe.elf` | 91688 | `608ddcd23492e6a55f70235395d2a49cb51bb643369c16b6bc3885ed153a7344` |
| `build/nas-release-intake/users/nas-volume.elf` | 97816 | `abd6321ad3963badfe7a0f8e5eba6df457f81aa70501d0dcf490dd3bb60fff14` |

Dit zijn nieuwe lokaal gebouwde hashes, niet de eerdere ce52-ELF-hashes.
Logs: `/tmp/nieuw-os-nas-release-intake-build.log` en
`/tmp/nieuw-os-nas-release-intake-{native-core,native-volume,native-probe}-{tests,clippy,fmt}.log`.

## Implementatiegrens van de volgende slice

De manager heeft TL-REL exclusief het opt-in profiel
`native-system-nas-probe` / `test-system-nas-recover` toegewezen.
Het nieuwe kernelmodulepad vervangt alleen voor dit profiel de bestaande
system-fixturepolicy. De bestaande vier ELF-slotbindings worden hergebruikt;
`kernel/build.rs`, `system.rs`, paging/boot/loader, SDK-runtime, keyring,
volume en NAS-leafs blijven ongewijzigd.

Alleen NAS krijgt entropy, alleen volume krijgt gescopeerde blockrechten
(normaal READ/WRITE/FLUSH, koud READ). Admin/gast krijgen uitsluitend
rolgebonden IPC-endpoints. De tweede volumeingress houdt een geldige RECV
voor de bestaande wachtset maar geen actieve client-SEND. Geen nieuw
recht, syscall of wire-ABI.

Op `37b09c7` is
`NOS_BUILD_DIR=build/nas-release-profile NOS_TARGET_DIR=target/nas-release-profile python3 -B tools/dev.py python3 -B tools/build.py --kernel-feature native-system-nas-probe`
werkelijk uitgevoerd: exit 0, bestaande leaf-ELF's plus UEFI-loader en kernel
gebouwd/geïnspecteerd. Log `/tmp/nieuw-os-nas-release-profile-build.log`.
Dezelfde buildroute met `NOS_BUILD_DIR=build/nas-release-profile-cold` en
`--kernel-feature test-system-nas-recover` slaagde eveneens (exit 0;
`/tmp/nieuw-os-nas-release-profile-cold-build.log`); tijdens deze tweede build
was uitsluitend de gewijzigde JSON-reportreader in `tools/build_nas.py` nog
ongecommit, later bewaard als `4550ce3`. Dit is geen native uitvoering.

Onafhankelijke NAS-oraclebron `86a9600` is inhoudsbehoudend opgenomen als
`7890ea8`. De NAS-owner behoudt eigendom van parser en leaf-tests. Rootrunner
`tools/test_nas.py` gebruikt die oracle, bestaande ESP-/diskguards en bestaande
QEMU-uitvoering. Hij voert geen NAS-operatie namens de gast uit.

`python3 -B -m unittest discover -s tests -p 'test_nas_*.py' -v`: exit 0,
8 hosttests, log `/tmp/nieuw-os-nas-root-tests.log`. Dit zijn synthetische
artifact-/verifiernegatieven, geen native servicebewijs.
`python3 -B tools/dev.py cargo +1.90.0 fmt --all -- --check` en
`git diff --check`: exit 0.

**Nog niet uitgevoerd voor de runnercheckpointbron:** normale/koude
QEMU-run, guestcaller-/grantasserties, stackmeting en onafhankelijke
diskcontrole op echte guestbytes. De build en gates moeten op deze bron
worden herhaald. B72 heeft de read-only reviewerrol bevestigd; review
en relevante bredere native regressiegates zijn nog open.
Geen TLS, E2EE, apparaatidentiteit, stroomuitval- of volledige P04-claim.
R01-R03, P01-P04 en volledige VM-FR blijven open; scoreadvies **34% → 34%**.
De definitieve mainbasis na PR #74 en herhaalde gates blijven verplicht.
Centrale PROJECT-STATE/assessment blijven bij de manager.

## Echte normale en koude NAS-proef op 808f539

Exacte bron `808f5391a604b6c03f18c712aa2ab3a8929abfa4`, v3
`f7b6f26be1a70a5a702095ee56458b888af562b1a01f579810206f194e52602e`.
Werkboom was schoon; geen bronwijziging tijdens de volledige gate.
`808f539/gate.json`, beide volledige `result.json`-bestanden, raw serial,
stderr en commandolog zijn ongewijzigd gearchiveerd met `SHA256SUMS`.

Werkelijk uitgevoerd in de bestaande read-only hostomgeving:

```sh
HOST_TOOLS=/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-didactic-spork/.tools/nieuw-os/host-tools
QEMU_TOOLS=/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-studious-sniffle/.tools/nieuw-os/qemu
export PATH="$QEMU_TOOLS:$HOST_TOOLS/usr/bin:$PATH"
export LD_LIBRARY_PATH="$HOST_TOOLS/usr/lib/x86_64-linux-gnu:$HOST_TOOLS/lib/x86_64-linux-gnu"
export OS_OVMF_CODE="$HOST_TOOLS/usr/share/OVMF/OVMF_CODE_4M.fd"
export OS_OVMF_VARS="$HOST_TOOLS/usr/share/OVMF/OVMF_VARS_4M.fd"
export NOS_BUILD_DIR="$PWD/build/nas-release-808f539"
export NOS_TARGET_DIR="$PWD/target/nas-release-profile"
python3 -B tools/dev.py python3 -B -u tools/test_nas.py
```

Exit **0**. QEMU 10.2.1, pc-q35-10.0, qemu64, TCG, 1 CPU/256 MiB,
geen netwerk. Dit is een lokaal acceptatieprofiel, niet VM-FR of een
interactief productimage. De QEMU-launcher staat buiten deze checkout en
wordt alleen uitgevoerd/gelezen; de rapporthash bindt die launcher, geen
claim dat daarmee alle dynamische hostbibliotheken zijn geauthenticeerd.
De exacte QEMU-argumenten en firmwarehashes staan in de rapporten.

| Proef | Raw exit / duur | Block READ / WRITE / FLUSH | Entropy |
|---|---|---|---|
| Normaal | 33 / 4,497 s | 272 / 578 / 18 | 96 B |
| Koud, verse VM en vars | 33 / 3,402 s | 816 / 0 / 0 | 32 B |

Werkelijke manifest-PIDs: admin 1, gast 4. Alle drie rollen sloten met
exit 0 en hun eigen assertions af; de bestaande volumedienst bleef levend
en gereed tot gecontroleerde proefcleanup. Normale RPC-counters admin
6/1, NAS 12/6, gast 6/5; koud 4/0, 4/4, 0/4 (succes/foutreply).
De gast zag beide keren kernelweigering bij RECV op zijn SEND-only
ingress. Gemeten aangeraakte stack: admin 3672 B, gast 4760 B,
NAS normaal 30912 B en koud 25184 B. Dit zijn gemeten uitgevoerde paden,
geen formele stackbovengrens voor andere workloads.

De onafhankelijk geschreven NAS-diskoracle controleert volume-generatie 11,
NAS-generatie 6, Evicted/epoch 2/quota 128, identiteit, geen authorityhistory
en intacte scope-canaries. Na normale verwerking en vóór/na koude run:
`318cfbaaeb41a7b8cc4abc1364452ab550ab1d4a0f354090d16b5a34e67035cd`.
De koude disk bleef volledig bytegelijk.

| Artifact | SHA-256 |
|---|---|
| Normale kernel | `5c79fdcbc25c81d16d89b88b01e094505c787a8fb40ba6b040f0cf98d49db7c6` |
| Koude kernel | `057da654fc769258d1fe46b5b00ec458727ba2883c3bfd28b710f92be7800ef3` |
| Loader beide | `0746886936b64985240d0061ad03a1fb873683ec371d1d1f9986640c89d7c11c` |

De bestaande oracle-negatieven zijn hier opnieuw uitgevoerd:
`python3 -B -m unittest discover -s services/nas/native-probe/tests -p 'test_disk_oracle.py'`:
exit 0, 8 tests, `/tmp/nieuw-os-nas-oracle-intake-tests.log`.

**Open:** onafhankelijke rootprofiel-/runnerreview door B72, bredere native
regressies, definitieve mainbasis na #74, actuele CI en eventuele reviewfixes.
Dit is positief lokaal native NAS-deelbewijs, geen geaccepteerd interactief
product, volledige P04, TLS/E2EE, echte apparaatidentiteit of abrupte
stroomuitval tijdens schrijven. Honderd koude productstarts en 24 uur
gecombineerd productgebruik zijn niet uitgevoerd en niet door deze twee
smalle VM-runs vervangen. R01-R03 blijven 0/5; totaaladvies 34% naar 34%.

## Onafhankelijke review: artifact- en platformbinding

B72 meldde twee concrete MEDIUM-bevindingen op `808f539`: de runner
controleerde ESP/immutable firmware niet na gastuitvoering en bond het
koude platform niet aan het normale platform. De positieve owner-run
hierboven blijft ongewijzigd bewaard; deze bevindingen maken die run niet
achteraf mislukt, maar blokkeren onafhankelijke acceptatie.

Eerst werkelijk rood gereproduceerd met
`python3 -B -m unittest discover -s tests -p test_nas_runner.py -v`:
exit 1, zes assertion failures, opgeslagen als
`review-bindings/red-confirmed.txt`. Drie mutaties tijdens de gemockte
uitvoering (kernel, loader, CODE) lieten ten onrechte `passed=True` over;
drie platformwijzigingen (firmware, QEMU, machine) startten ten onrechte
de koude VM. Twee eerdere lokale fixture-setupfouten zijn gecorrigeerd
voordat dit assertionniveau is bereikt; de rode reproductie is geen
native VM-proef.

De gerichte fix controleert de gekopieerde ESP, immutable CODE,
oorspronkelijke CODE/VARS-template en werkelijke QEMU-executable zowel
vóór als na uitvoering. Het koude platform moet exact hetzelfde
machineprofiel, absolute QEMU-ELF-pad, binaryhash, versie en oorspronkelijke
firmwarehashes hebben. Een shellwrapper in PATH is een setupfout, nooit
skip of succes. De gastkopie van VARS blijft bewust veranderlijk.
Geen wijziging aan gedeelde common/build-tooling, kernel of service-ABI.
Hostbibliotheken worden hierdoor niet integraal geauthenticeerd.

Acceptatiecommando:
`python3 -B -m unittest discover -s tests -p 'test_nas*.py' -v`.
Werkelijk uitgevoerd: exit 0, 15 tests, `review-bindings/green.txt`.
Ook `python3 -B -m unittest discover -s tests -p 'test_*.py'`:
exit 0, 435 hosttests, `review-bindings/all-tools.txt`. Dit zijn uitsluitend
host-/harnasregressies; synthetische VM-records hierin zijn geen nieuwe
gastuitvoering. `git diff --check`: exit 0.
Nieuwe native normal/cold-pair, bredere gates en herlezing van deze twee
punten door B72 zijn nog open tot een nieuwe brongebonden freeze.
De cold-case gebruikt een synthetisch canoniek oud token `[42; 40]`,
niet een werkelijk in boot 1 uitgegeven token. Dit bewijst dus geen
echte cross-boot-tokenreplay; die verticale keten blijft apart open.
Scoreadvies ongewijzigd **34% naar 34%; R01-R03 0/5**. De manager houdt
de centrale assessment, queue en PROJECT-STATE bij.

## Nieuwe native pair op provenancefix 1219dcd

Exacte codebron `1219dcdc8b7fcc6826ab64d440249f41fa2edf49`, v3
`9c4af2f8838bd87c7c9646ebd93f3efe2e4dd552599dc3aa9a053235d2591492`.
Werkboom schoon vóór en na uitvoering. Deze sectie vervangt uitsluitend
de hierboven nog open nieuwe normal/cold-gate, niet onafhankelijke review
of de ontbrekende integrale productacceptatie.

Werkelijk uitgevoerd, exit **0**, log `/tmp/nieuw-os-nas-1219dcd-native.log`
en identieke archiefkopie `1219dcd/native-command.txt`:

```sh
HOST_TOOLS=/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-didactic-spork/.tools/nieuw-os/host-tools
QEMU_TOOLS=/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-studious-sniffle/.tools/nieuw-os/qemu
export PATH="$QEMU_TOOLS/usr/bin:$HOST_TOOLS/usr/bin:$PATH"
export LD_LIBRARY_PATH="$QEMU_TOOLS/usr/lib/x86_64-linux-gnu:$HOST_TOOLS/usr/lib/x86_64-linux-gnu:$HOST_TOOLS/lib/x86_64-linux-gnu"
export OS_OVMF_CODE="$HOST_TOOLS/usr/share/OVMF/OVMF_CODE_4M.fd"
export OS_OVMF_VARS="$HOST_TOOLS/usr/share/OVMF/OVMF_VARS_4M.fd"
export NOS_BUILD_DIR="$PWD/build/nas-release-1219dcd"
export NOS_TARGET_DIR="$PWD/target/nas-release-profile"
python3 -B tools/dev.py python3 -B -u tools/test_nas.py
```

Rechtstreeks bestaande QEMU 10.2.1-ELF, geen wrapper, executablehash
`fa552a45c337a88fe71e1e1af9ebace875f30e88685a6986d9406c92efaf44cb`.
Beide stages hebben exact gelijke `platform`-velden:
pc-q35-10.0, firmware CODE
`874f15a7c9110e78a89db159f0f68fc97e9076b74777520de1c235426f758aa7`
en oorspronkelijke VARS-template
`5d2ac383371b408398accee7ec27c8c09ea5b74a0de0ceea6513388b15be5d1e`.
CPU/TCG/geheugen/netwerk en kernel-/loaderhashes zijn gelijk aan de
begrensde 808-proef. Nieuwe bronbinding en platform-/postexitcontroles
zijn werkelijk uitgevoerd, niet alleen via een cache geclaimd.

| Proef | Raw exit / duur | Block READ / WRITE / FLUSH | Entropy |
|---|---|---|---|
| Normaal | 33 / 4,494 s | 272 / 578 / 18 | 96 B |
| Koud, verse VM en vars | 33 / 3,491 s | 816 / 0 / 0 | 32 B |

Alle rol-/stack-/cleanupasserties en de onafhankelijke diskoracle
slaagden opnieuw. Disk na normale run en vóór/na koude run:
`2970ee1518bc129cc374669b4382dbc0a4880f149b69c0850cd5b92b7ef20d95`.
De hash verschilt terecht van de oudere, afzonderlijk gemaakte schijf;
deze proef gebruikt nieuwe gastentropy. Volledige rapporten, ongewijzigde
serial/stderr, zelfstandige oracle-CLI-uitvoer en `SHA256SUMS` staan
onder `1219dcd/`. De bestaande 808-map is niet overschreven.

Ook afzonderlijk daadwerkelijk uitgevoerd, beide exit **0**:

```sh
python3 -B services/nas/native-probe/disk_oracle.py build/nas-release-1219dcd/run/nas-normal-48934263a7ac4f01a07946f0497de294/volume.img --admin-pid 1 --guest-pid 4
python3 -B services/nas/native-probe/disk_oracle.py build/nas-release-1219dcd/run/nas-recover-0ce1555ee75c48719cf4a2a2cc6c4b77/volume.img --admin-pid 1 --guest-pid 4 --normal-sha256 2970ee1518bc129cc374669b4382dbc0a4880f149b69c0850cd5b92b7ef20d95
```

**Open:** B72 herlezing van de twee reviewpunten, verdere native regressies
en definitieve mainintegratie door e8. De hosttoolsuite (435 tests) is
wel uitgevoerd; zij vervangt die native regressies niet. Dit is nog
geen VM-FR-/Windowsproductimage of werkelijk boot1-tokenreplay.
R01-R03 blijven **0/5**, totaaladvies **34% naar 34%**; honderd koude
productstarts, 24 uur productgebruik, fysieke en menselijke acceptatie
blijven open. Geen actieve tweede ABI-/BootInfoimplementatie.

## Beperkte onafhankelijke verificatie ontvangen

B72 (`b72f2536-b494-4a6b-a608-b6fd785e18ef`) heeft op code `1219dcd`
en evidence `0611d18` uitsluitend de twee eerder gemelde bindingspunten
herlezen en afgedaan, zonder aanvullende bevinding binnen die scope.
De reviewer controleerde read-only de tien nieuwe archiefhashes,
serialverificatie, gate/result/sourcecommit/v3/manifest/platform/command,
firmware- en diskbinding en de werkelijk aanwezige QEMU-ELF-hash:
exit 0 volgens diens eigen rapportage. Geen nieuwe VM of hosttest door
de reviewer uitgevoerd; de twee echte VM-runs blijven producerbewijs.
Dit vervangt uitsluitend de hierboven open tweepuntsreview, niet volledige
NAS-/productacceptatie, menselijke audit of mainintegratie.

De reviewer behield de ruwe logbytes. Een diffcheck vanuit zijn oudere
checkout miste de nieuwe lokale archiefattributen en gaf daarom exit 2;
de broncheck en herhaling met de attributen uit `0611d18` gaven exit 0.
De expliciete `-text`-attributen beschermen exact bewaarde serial/logs;
dit is geen broncode-uitzondering en geen achteraf genormaliseerde log.

Aanvullend werkelijk uitgevoerd:
`python3 -B -m unittest discover -s tools/tests -p 'test_*.py'`:
exit 0, 138 hosttests, `review-bindings/workflow-tools.txt`.
Dit is de afzonderlijke workflowselector, naast de 435 root-hosttests.
De daarin gemockte gastlogs zijn geen extra native gastuitvoering.

Volgende begrensde gate: bestaande `tools/test_system.py` normal/recover
op dezelfde bronvingerafdruk, eigen `build/nas-default-regression` en
bestaande `target/nas-release-profile`, zonder codewrites of GUI-test.
De precieze uitslag volgt pas na uitvoering; geen geïntegreerde kandidaat
of Windowsbundel is hiermee vrijgegeven. Centrale registratie en nieuwe
rol-/bestandoverdrachten blijven bij de manager.

# TL-BUILD-COLD: gewone koude herstart na geforceerde hoststop

Codebron `420fac5783fdf86ed6d6464a8f1498190fdf78ef`,
branch `simon/native-cold-acceptance`.
V3 `f6d4ca781ac058b7c6eba1d7b1bf49314e07e45828158e28a0afde07b74b2304`.
Geen writes aan de bevroren PR #66-head c096 of bestaande preview/probe,
kernel, SDK, workflows, centrale status of voortgang.

## Contract en bewijsgrens

Nieuwe achtprocessenreferentie en gewone zevenprocessenparent op dezelfde
bevroren bron; parent krijgt echte QMP-toetsen/muis voor nieuw document,
typen, opslaan, lijst en heropenen. Daarna expliciete host-QMP-quit/raw0.
De cold-runner herhaalt onafhankelijk alle parent-inputs, referentiebeelden,
executable-/firmware-/bronbindingen en het diskorakel alvorens een verse VM
met verse OVMF-vars en een exacte bytekopie van de opgeslagen disk te starten.
Alleen lijst en heropenen volgen; ieder volledig 800x600-beeld moet passen.
Na de tweede hoststop moeten de diskbytes onveranderd zijn en moet hetzelfde
onafhankelijke diskorakel slagen.

Het verwachte koude lijstbeeld neemt het populated-listbeeld over, met de
volledige Ready-statusrechthoek uit het native Ready-beeld volgens de vaste
rendererlayout; geen pixels worden genegeerd of gemaskeerd. Het heropenbeeld
is het volledige native referentiebeeld. QMP-responses/IDs en de exacte
invoervolgorde worden na exit onafhankelijk herhaald.

**Geen guest clean shutdown:** QMP-quit is een geforceerde hoststop zonder
gastafsluitprotocol, maar QEMU kan daarbij zijn host-writebackcache flushen.
Dit is dus geen bewijs van abrupte stroomuitval tijdens een write, guest
cleanup, encryptie, controller/snapshots, volledige herstelmatrix,
handmatige UX of volledige releaseacceptatie. De kernel wordt niet aangepast
om een ontbrekend gastafsluitpad te simuleren.

## Exacte uitgevoerde commandoreeks

Vanuit de repositoryroot; alle VM-images zijn geïsoleerde wegwerpimages.
De bestaande lokale tools worden hergebruikt, geen installatie.

```sh
python3 -B -m unittest tools.tests.test_materiaal_cold
python3 -B -m unittest discover -s tools/tests
python3 -B tools/run_materiaal_cold.py --help
git diff --check
python3 -B tools/progress.py --check

export LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu"
export NOS_TARGET_DIR="$PWD/target/native-cold-acceptance/gate1"
export OS_OVMF_CODE="$PWD/.tools/nieuw-os/qemu/usr/share/OVMF/OVMF_CODE_4M.fd"
export OS_OVMF_VARS="$PWD/.tools/nieuw-os/qemu/usr/share/OVMF/OVMF_VARS_4M.fd"
NOS_BUILD_DIR="$PWD/build/native-cold-acceptance/gate1-reference" python3 -B -u tools/build.py --kernel-feature native-system-materiaal-probe
python3 -B -u tools/run_materiaal_probe.py --build-dir "$PWD/build/native-cold-acceptance/gate1-reference" --machine pc-q35-10.0 --vga-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/vgabios-stdvga.bin" --vga-rom-sha256 c944f5fd404a6553a32e1e0527081d40e6040d3ce1740d39766bff01871e4fde
NOS_BUILD_DIR="$PWD/build/native-cold-acceptance/gate1-parent" python3 -B -u tools/build.py --kernel-feature native-system-materiaal
python3 -B -u tools/run_materiaal_preview.py --build-dir "$PWD/build/native-cold-acceptance/gate1-parent" --interactive-reference-run "$PWD/build/native-cold-acceptance/gate1-reference/run/materiaal-normal-bf9f735c6b" --timeout 240 --machine pc-q35-10.0 --bios-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/bios-256k.bin" --vga-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/vgabios-stdvga.bin"
python3 -B -u tools/run_materiaal_cold.py --build-dir "$PWD/build/native-cold-acceptance/gate1-parent" --parent-run "$PWD/build/native-cold-acceptance/gate1-parent/preview-run/materiaal-c767f695c6" --reference-run "$PWD/build/native-cold-acceptance/gate1-reference/run/materiaal-normal-bf9f735c6b" --timeout 240 --machine pc-q35-10.0 --bios-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/bios-256k.bin" --vga-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/vgabios-stdvga.bin"
```

14 gerichte/152 tools-hosttests exit0; CLI-help, diffcheck en voortgangcheck
exit0. Reference-build 10.58s, gewone build 2.03s, beide exit0.
Referentie werkelijk geslaagd/raw33, rapportduur 134.483s/walltime 134.64s.
Gewone parent `c767f695c6` geslaagd/raw0, rapportduur 93.036s/walltime 93.21s;
alle zes volledige referentiebeelden en post-quit diskorakel groen.
Cold-child `c4e137e8db` geslaagd/raw0, rapportduur 38.225s/walltime 38.36s;
volledig lijst- en heropenbeeld groen en alle diskbytes ongewijzigd.
Alle vijf native build/run-commando's hierboven exit0.

Logs `/tmp/nieuw-os-cold-lifecycle-tests.log`,
`/tmp/nieuw-os-cold-all-tools.log`, `/tmp/nieuw-os-cold-progress-check.log`
en `/tmp/nieuw-os-cold-gate1-{reference,parent}-{build,run}.log`,
`/tmp/nieuw-os-cold-gate1-child-run.log`.
Voor native commando's bewaart dezelfde prefix met `.time` de gemeten wall-,
user- en systijd. Gegenereerde summaries behouden originele rapporthash/pad;
grote QMP-traces, PPM's en disks blijven in de runmappen buiten Git.

## Artifactbinding en zelfstandig diskorakel

Gewone kernel
`f2e9cc4aad87ad2b9fbc8b498ab146db2ab0df4a77b904eecf81c3a9beb198b3`,
loader `f90bc8f0452e13bafcc8ed1a2afc24d8e0f20573e5a381dfb0ca90fdc0ca1025`.
Zeven app/ELF-records zijn in de parent/referencebinding vastgelegd en in
de coldsummary opgenomen; geen bitidentieke eigen-worktreebuild beloofd.

Cold-before en cold-after zijn beide
`246c8f9773533796718a729389c26d6103f45f04e7a83b4d06503e17c6d4e07e`.
Onafhankelijke Python-decoder: object0/rev1, naam `Nieuw document`,
tekst `nieuw os werkt` (14 bytes),
tekst-SHA256 `9a45a00fa9ff69c17e124f54c14bb14cf2c7650e8cf1c272f6631f618a19a5a5`,
generatie1/klokken[1,0], andere domeinen leeg en canaries intact.
Deze controle draait na procesexit; een nog levende of raw-nonzero VM
kan niet via dit orakel als geslaagd worden aangemerkt.

Volledig koude lijst-PPM:
`ebc10cc5ac84ff5ac80796e42de4261a26fcb5faaa021b8a11e8a14460555eaa`;
volledig heropen-PPM:
`dfca8c62b1e10b99a6561098b56be8749102d69dca390257717935686fb19099`.
405 cold-QMP-opdrachten (inclusief handshake/quit) zijn gecorreleerd;
extra invoer, F12/controller, ontbrekende quit of vervalste response-ID
wordt door trace-replay geweigerd.

`reports.json` bindt compacte gegenereerde summaries aan de originele
ongewijzigde lokale rapporten. De volledige originele coldreport-SHA256 is
`c3bbc81b9b2a5afc2ee166c1612c0d60cfea4e4d75755888b6d1efb9d1e4eed3`.
Raw serial/stderr zijn exact gekopieerd. Summaries zijn geen vervanging van
de oorspronkelijke grote artifacts voor een volledige replay.

## Volgende onafhankelijke infrastructuurstap: echte artifactnegatieven

Na de gepushte native mijlpaal `24e45fada614a05952cc78e11fd288fa2837c93a`
is de keten opnieuw offline uit de originele artifacts berekend:

```sh
python3 -B docs/verification/native-cold-acceptance/tamper_replay.py --build-dir "$PWD/build/native-cold-acceptance/gate1-parent"
```

Exit0, walltime 1.11s, resultaten `tamper-results.json`.
Volledige coldtrace, frames, diskorakel, executable-/firmware-/bronbinding,
referentie en parent opnieuw gecontroleerd zonder een VM te starten.
Een verse tijdelijke bytekopie van de echte parent is eerst positief
herhaald, daarna op acht manieren afzonderlijk vervalst: boolean raw-exit,
andere bron, andere referentiehash, QMP-errorresponse, extra reset in plaats
van quit, gewijzigde disk, gewijzigd frame en gewijzigde kernel.
Alle acht zijn op hun concrete fout geweigerd; daarna is de herstelde
controlekopie opnieuw geaccepteerd. Geen productieassertie versoepeld.
Alle 66 oorspronkelijke artifactbestanden/rapporten zijn vóór/na gehasht
en ongewijzigd; alleen tijdelijke kopieën zijn gemuteerd en opgeruimd.

De eerste offline scriptaanroep faalde vóór artifactwrites met
`KeyError: directory`: het bestaande rapport gebruikt `run_directory`.
Die veldnaam is in uitsluitend dit bewijshelperscript hersteld; de native
runner en zijn v3 zijn niet gewijzigd. Geen native mislukking verhuld.
Logs `/tmp/nieuw-os-cold-gate1-tamper-replay.{json,log,time}`.
Gerichte 14 tests en alle 152 tools-tests zijn daarna opnieuw exit0
uitgevoerd; voortgangcheck/diffcheck eveneens exit0.

## Open acceptatie

Deze smalle native coldketen is geslaagd. Clean shutdown,
Windowscold, productie-opslag en CORE-bitreproduceerbaarheid blijven open.
Scorevoorstel aan de centrale eigenaar **34% -> 34%**; geen extra volledig
rubriccriterium afgesloten door alleen deze smalle herstelproef.

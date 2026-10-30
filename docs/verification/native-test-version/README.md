# Native testversie — lopende integratie, nog geen releaseacceptatie

Basis `ad2b981fe1f2bcccb03cb776b2281d32529d2f00`, branch
`simon/native-test-version`; eerste componentintegratie `8becc511a08be128ce0db417f15efa180bfd2eb3`.
De bestaande IPC-WAIT-1- en inputconsumentenbron is overgenomen en aangepast
aan de inmiddels gereserveerde RTC-ABI 1.9: wachten gebruikt ABI 1.10.
Doel is de werkelijk rode native Materiaal-gebruikersketen te herstellen,
niet de 20.000-PIT-tickgrens of acceptatievoorwaarden te versoepelen.

## Nieuwste gecombineerde bron en proefversie

Op runtimebron v3 `a6c4c723fbcc6a5b50eac17b91b77f66161c8555c6d916fd10b7826815acc4b5`
slagen **16 echte VM's**. De veertien niet-GUI-runs op `37a7478` staan
in `tpm-regression-evidence.json`; normale/koude GUI op docscheckpoint
`4bdc1a2` en de bitgelijke lokale proefversie staan in
`gui-tpm-integration-evidence.json`. De GUI-runs duren 128,979 en 61,100 s:
zes echte frames, 14 opgeslagen bytes, koude heropening, zichtbare
writeweigering, nul recoverywrites/flushes en volledige cleanup.
Alle tien normale payloads zijn afzonderlijk vergeleken met de bevroren
geslaagde run; ook het normale manifest is bitgelijk.

De nieuwste lokale **automatische** QEMU-proefversie staat in
`build/native-test-version/test-version-current`, profiel
`native-system-materiaal-probe`. Dit is niet de blijvend interactieve
`native-system-materiaal`-variant. De herstelbuild onder `gui-tpm-final`
is readonly en is geen normale proefversie. Geen Windows-ZIP, hardware,
menselijke afwerkingsacceptatie of volledige CI op deze branch bewezen.
De rubric blijft **34% → 34%**; de volledige opdracht en 50% blijven open.
Oudere rapporten hieronder behouden hun eigen bronbinding.

## Eerste hostcontrole

Voor elk Cargo-commando:

```sh
export LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu"
export CARGO_TARGET_DIR="$PWD/target/native-wait-checks"
```

| Exact commando | Resultaat |
| --- | --- |
| `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked -p capability-core` | Exit 0: 57 coretests, 7 tests van echte kernel-wachtdecoder/evaluator, 1 bestaande benchmark ignored |
| `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime` | Exit 0: 23 tests, inclusief ABI 1.10 en weigering van onbekende minors |
| `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/input-service/Cargo.toml` | Exit 0: 23 tests |
| `python3 -B -m unittest tools.tests.test_ipc_wait` | Exit 0: 2 rapportverifier-tests |
| `python3 -B tools/preflight.py --strict` | Exit 0: Rust 1.90.0, beide targets, QEMU 10.2.1 en OVMF aanwezig; geen build- of bootbewijs |

Logs staan lokaal in `/tmp/nieuw-os-native-wait-{core,sdk,input,verifier}.log`.
De eerste formattingpoging faalde doordat rustfmt ontbrak. Na expliciete
toolinstallatie slaagden `cargo +1.90.0 fmt --all`,
`cargo +1.90.0 fmt --manifest-path sdk/Cargo.toml --all` en
`cargo +1.90.0 fmt --manifest-path sdk/examples/input-service/Cargo.toml --all`,
ieder via `python3 -B tools/dev.py`, exit 0.

De lokale QEMU/OVMF-pakketten komen uit officiële Ubuntu-pakketten,
zonder sudo of systeeminstallatie. Genegeerd herkomstrapport:
`.tools/nieuw-os/env-native-provenance.json`. QEMU 10.2.1
(`1:10.2.1+ds-1ubuntu3.3`), OVMF `2025.11-3ubuntu7.3`:

| Input | SHA-256 |
| --- | --- |
| QEMU implementatie | `fa552a45c337a88fe71e1e1af9ebace875f30e88685a6986d9406c92efaf44cb` |
| OVMF CODE | `874f15a7c9110e78a89db159f0f68fc97e9076b74777520de1c235426f758aa7` |
| OVMF VARS-template | `5d2ac383371b408398accee7ec27c8c09ea5b74a0de0ceea6513388b15be5d1e` |

## Open bewijsgrens

De twee daadwerkelijke GUI-pogingen blijven rood, met ongewijzigd
20.000-PIT-tickbudget: zonder wachten 202,396 s, met alleen inputwachten
202,363 s; beide drie echte frames, 14 getypte bytes, nog geen save.
Zie `gui-attempts.json` voor bron-/loghashes. De volgende versie voegt
alleen quiescente servicewachten toe; 100 keyring/volume-hosttests en
21 desktop-init-tests slagen, plus native Clippy en de samengestelde
ELF/kernelbuild. Die native runtime is nu **groen**: op `2066ae7` zes echte
frames en de volledige muis/typen/save/list/reopen/F12-keten in 131,561 s
met raw33, zonder verruimde deadline. Zie `gui-normal-evidence.json`.
De disk bevat werkelijk de 14 getypte bytes, één save-ACK en revisie1;
systeemrand/canaries blijven intact en alle processen/handles worden
vrijgegeven. De eigen-kernel/UEFI-proefversie staat lokaal onder
`build/native-test-version/test-version/esp`, met de gebonden normale run
en snapshots onder `build/native-test-version/gui-idle/run/materiaal-normal-4417ddea00`.
Alleen synthetische plaintextdata;
geen persoonlijke data, encryptie- of complete releaseclaim.

De native wachtbuild en runtime zijn inmiddels **geslaagd**:
het exacte commando en compacte hashbinding staan in `wait-evidence.json`.
Raw 33, exit 0; rolmaskers 63/60, een kernel-gecontroleerde volle queue
na intrekking, 14 blokkeringen, 7 ready-wakes, 6 deadline-wakes,
1 intrekkingswake en 199 werkelijk getelde PIT-idle-interrupts.
De eerste echte run faalde met raw 35 door audit van mappings van een reeds
overleden eerder processlot. Die audit slaat nu uitsluitend Dead slots over.
Een tweede raw-33-run bleef host-rood omdat de generieke runner de nieuwe
poort zonder M2-marker als M1-only behandelde; M2 is nu expliciet vereist.
De derde volledige build/run slaagt. Dit is geen testversoepeling.
Een daaropvolgende gerichte AI-review vond een mogelijke extra PIT-tick
vóór foutwake als de selectiescan een ongeldig userreturnframe opruimt.
De selectielus herstart nu bij haar waitrefresh na die cleanup; de wachtpoort
en legacy `test-m3-invalid-rsp` zijn op `ab98d1a` opnieuw raw33 geslaagd.
Deze review is geen menselijke audit.

**Destijds niet uitgevoerd op die integratie:** native kill/hergebruik en wachten
bij nog geblokkeerde processen (nu afzonderlijk bewezen hieronder), alle bredere native regressies, fysieke hardware en menselijke
gebruiks-/securityacceptatie. De gezamenlijke CPL3-wachtprobe is als echt
scalair ELF gebouwd en native uitgevoerd. Probe-hosttests: 3/3, exit 0.
Gerichte Clippy voor capability-core, SDK-runtime en inputconsumenten:
ieder exit 0 met `--all-targets -- -D warnings`, via dezelfde lokale
Cargo-omgeving. Die controle miste aanvankelijk de Clippy-component;
na `python3 -B tools/dev.py rustup component add clippy --toolchain 1.90.0`
zijn de drie controles daadwerkelijk herhaald.

`python3 -B -m unittest discover -s tests` slaagt met 414 tests (4,719 s);
`python3 -B -m unittest discover -s tools/tests` met 100 tests (3,496 s).
Deze zijn hostharnastests; hun gesimuleerde transcriptuitvoer is geen nieuwe
native VM-uitvoering. Logs: `/tmp/nieuw-os-native-test-{python,tools}.log`.
De score blijft **34% → 34%**; het gevraagde doel van minstens 50% is
niet bereikt. Een hosttest, geïntegreerde component of extra PASS-teller
sluit geen volledig rubriccriterium.

## Echte TPM-CRB-MMIO

`tpm-evidence.json` bevat beide echte native runs op `4b09869`, raw33/exit0:
één vaste supervisor-RW/NX/UC-aperture, een read-only TPM2-familyquery
(`0x322e3000`) en volledige Ready/response/wipe/Idle/relinquish-afhandeling.
De absent-VM zonder TPM maakt exact nul hardwarewrites en behoudt
quarantaine. Zeven echte adaptergrensweigeringen; in beide VM's blijven
de volledige CPL3-wacht-/rechten-/lifecycleproeven groen, met 199 idle-IRQ's.
De eerste eigen mapperintegratiefout blijft als raw35 bewaard.

Beide exacte commando's, toolversies, bron-, payload-, result- en loghashes
staan in het JSON-rapport. Alleen officiële lokaal uitgepakte swtpm/libtpms-
testtools, eigen Unixsocket/wegwerpstate en eigen kernel. Geen echte TPM,
nieuwe appcapability, NV-mutatie, productkeys of koud rollbackanker.
23 bestaande TPM-coretests en 4 gerichte harnastests slagen.
De bestaande native-system-CI draait voortaan beide cases zonder skip of
genegeerde fouten; CI op deze branch is nog niet uitgevoerd. Dit is een
native fundamentstap richting B02, geen afgetekend B02- of releasecriterium.

## Kill van werkelijk wachtende processen

De zes echte K05-cases slagen op `586c33c`, allemaal raw33/exit0:
normal, crash, denied, parent-exit, quota en de nieuwe waitcase.
`wait-lifecycle-evidence.json` bindt bron, gate, alle zes result/serialhashes,
payloads, het exacte gatecommando en de drie eerdere rode pogingen.
Die gate voert ook 64 core/wachttests (1 bestaande ignored), 23 SDK-tests,
1 protocoltest en 8 harnastests uit; alle exit0.

De nieuwe case meet twee geautoriseerde kills van werkelijk Waiting
kinderen, schoon hergebruik van hetzelfde slot met verse PID, vier precieze
weigeringen en geen oude wake voorbij de eerste deadline. Vier echte
blokkeringen, twee ouder-deadlinewakes, 84 idle-IRQ's/87 ticks; volledige
Engine-/framecleanup. Geen productie-, rechten- of schedulerverruiming.
De probe is geen nieuw rubriccriterium; eerdere GUI/FPU-bewijzen blijven
aan hun eigen bron gebonden.

## Koude heropening en lokale proefversie

Op `de541fa` slaagt ook een volledig nieuwe readonly-QEMU-VM, exit0/raw33,
60,769 s. De parent is de bovenstaande geslaagde normale run: exact dezelfde
diskbytes, loader, firmware, VGA-ROM en acht userspace-ELFs. Drie echte frames,
heropenen van `nieuw os werkt`, zichtbaar geweigerde write, geen save-ACK,
nul disk-writes/flushes en dezelfde diskhash na afloop. Alle processen/handles
zijn opgeruimd; canaries en systeemrand blijven intact.
Zie `gui-recover-evidence.json` voor commando, bron-, artifact- en loghashes.

De eerste herstelinvocatie met een aparte buildroot werd vóór gaststart
geweigerd. De runner verlangt bewust dat de normale parent onder de eigen
`BUILD/run` staat; die controle is niet veranderd. Bouw het herstelprofiel
daarom in dezelfde root als de normale run:

```sh
export LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu"
export NOS_BUILD_DIR="$PWD/build/native-test-version/gui-idle"
export NOS_TARGET_DIR="$PWD/target/native-test-version/gui-baseline"
python3 -B -u tools/build.py --kernel-feature test-system-materiaal-recover
```

Dit buildcommando en het exacte recover-runcommando uit het JSON-bewijs zijn
beide werkelijk uitgevoerd, exit0. Buildlog:
`/tmp/nieuw-os-native-gui-recover-build.log`; runlog:
`/tmp/nieuw-os-native-gui-recover-run.log`.

Een normale, opnieuw uitvoerbare lokale proefversie is vervolgens gebouwd:

```sh
LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu" \
NOS_BUILD_DIR="$PWD/build/native-test-version/test-version" \
NOS_TARGET_DIR="$PWD/target/native-test-version/gui-baseline" \
python3 -B -u tools/build.py --kernel-feature native-system-materiaal-probe
```

Exit0; log `/tmp/nieuw-os-native-test-version-build.log`. Afzonderlijke
SHA-256-vergelijking bevestigt dat loader, kernel en alle acht userspace-ELFs
bitgelijk zijn aan de geslaagde normale native run. De gekopieerde
`volume.empty.img` heeft de oorspronkelijke lege-fixturediskhash.
Dit is een automatische QEMU-proefversie met een begrensde testlevensduur,
geen installer of dagelijks bruikbaar OS. De oorspronkelijke `gui-idle/esp`
bevat nu het readonly-profiel; beide runs behouden hun eigen bevroren inputs.
Geen binaries in Git en geen publieke release.

## Echte FPU-context over blokkeren

Op `deb7a3d` slagen zowel `test-fpu-wait` als de aparte ABI1.6/yieldbaseline:
exit0/raw33, respectievelijk 5,687 s en 2,162 s. De combinatie verifieert de
hele x87/SSE-bank rond 41 echte blokkeringen/deadlinewakes, 36 CPL0-idle-IRQ's
en 347 echte CPL3-timerinterrupts. Exact 456 saves/restores; drie schone
levensduren, echte AVX-fault, survivor, verse PID3 in slot0 en volledig
frame-/context-/Engineherstel. Geen extra capabilities toegekend.

`fpu-wait-evidence.json` bevat beide exacte build/runcommando's, profielen,
bron-/artifact-/loghashes en de vier eerdere failures. Hun oorzaken waren
fixturecompositie, ontbrekende PID-registratie, onvoldoende echte user-
preëmptie en een tellerrelatie die idle ten onrechte als user-FPU-save telde.
Beide orakels controleren nu exact totaal=user-A+user-B+idle en
saves=user-A+user-B+syscalls+fault, plus restoregelijkheid. De bestaande
assembly, schedulerbody en IPC-afdwinging zijn daarvoor niet aangepast.
De negatieve hosttests weigeren ook het verkeerde profiel en foutief
meegetelde idle-saves; `python3 -B tools/test_fpu.py`: 12 tests, exit0.

Dit sluit de ontbrekende FPU/wachtcombinatieproef, niet kill van een nog
Waiting proces, volledige GUI- of releaseacceptatie.

## Definitieve geïntegreerde proefversie

De normale én koude GUI-keten zijn op `36cbd21` werkelijk opnieuw uitgevoerd:
exit0/raw33 in 128,857 s en 60,679 s. Zelfde zes/drie echte frames,
werkelijke edit/save/list/reopen, exacte 14 opgeslagen bytes, zichtbare
readonly-weigering, ongewijzigde koude disk en volledige cleanup.
System normaal/koud, native RTC en IPC-wacht slagen op diezelfde bron
opnieuw raw33. Core/wacht: 64 tests en 1 bestaande ignored; SDK: 23 tests,
offline/locked exit0. `integrated-evidence.json` bevat de exacte commando's,
logpaden, bron- en artifacthashes. De FPU-runs hebben dezelfde runtime-
bronvingerafdruk; hun oorspronkelijke commitbinding blijft behouden.

`build/native-test-version/test-version` is opnieuw als normaal proefprofiel
gebouwd. Een aparte hashvergelijking bevestigt dat de loader, kernel en alle
acht userspace-ELFs bitgelijk zijn aan de geslaagde normale run. De
`volume.empty.img` is de bytegelijke synthetische lege schijf.
Het profiel is automatisch opnieuw te bedienen met:

```sh
LD_LIBRARY_PATH="$PWD/.tools/nieuw-os/host-cc/usr/lib/x86_64-linux-gnu" \
NOS_BUILD_DIR="$PWD/build/native-test-version/test-version" \
python3 -B -u tools/run_materiaal_probe.py \
  --build-dir "$PWD/build/native-test-version/test-version" \
  --machine pc-q35-10.0 \
  --vga-rom "$PWD/.tools/nieuw-os/qemu/usr/share/seabios/vgabios-stdvga.bin" \
  --vga-rom-sha256 c944f5fd404a6553a32e1e0527081d40e6040d3ce1740d39766bff01871e4fde
```

Dit laatste padcommando is de herhaalprocedure, **niet nog een uitgevoerde VM**:
de werkelijk uitgevoerde normale run gebruikte `gui-final`, zoals hierboven
en in het JSON-rapport staat. De geleverde payloads zijn er bitgelijk aan.
Dit blijft een begrensde automatische QEMU-proef, geen installer/dagelijks OS.
Score **34% → 34%**; het 50%-doel en de volledige productmatrix blijven open.

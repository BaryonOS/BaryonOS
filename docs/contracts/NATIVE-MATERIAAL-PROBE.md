# GUI-QMP-1 — echte native bedieningsketen

Eigenaar native_ui_apps: uitsluitend tools/run_materiaal_probe.py,
tools/tests/test_materiaal_probe.py en dit contract. Basisd528821, eigen
worktree materiaal-qmp. Root bezit feature/ABI/manifest/build/kernelobserver;
kernel_storage bouwt de afzonderlijke desktop-init/controller. Deze runner is
een afgebakende integratieproef, geen volledige Materiaal- of releaseacceptatie.

## Uitvoering en bewijsgrens

De runner start alleen een gehashte eigen-kernel-QEMU met acht echte CPL3-images:
desktop-init, keyring-service, volume-service, materiaal-app, native-compositor,
input-source, input-policy, desktop-controller. De normale productcompositie
heeft zeven processen; het achtste proces is uitsluitend de beperkte test-
tegenpartij met twee IPC-grants voor een echte InitStopRPC. Het heeft geen
display-, invoer-, blok- of geheugenschrijfgrants naar een ander proces.

QMP levert echte PS/2-toetsen/muisbewegingen en leest echte framebufferbeelden.
De runner schrijft geen guest-RAM, draft, resultaatwoord of opslaginhoud tijdens
de proef. Hij voert geen ontbrekende OS-functie op de host uit. Protocolfixtures
in de Python-unittests zijn expliciet hosttegenproeven voor de validator.

De fixture is het werkelijk getypte ASCII/UTF-8 `nieuw os werkt` (14 bytes),
object0, naam `Nieuw document`, revision1. Het is plaintext-testdata op een
nieuwe1MiB-wegwerpimage. De bestaande Key Service is aanwezig, maar deze keten
bewijst geen persistente encrypted opslag, sleutelherstel of productbeveiliging.
FNV is hier alleen een controle op de exacte bytes; geen authenticatieclaim.

Deze eerste ontwikkelproef is expliciet800×600, qemu64/TCG/1CPU/256MiB, VGA/PS2.
Zij vervangt de grotere releaseprofielen, toetsenbord-/muisacceptatie,
200%-toegankelijkheid, schermlezer en fysieke hardwarevalidatie niet.

## Gekoppelde build

Features: native-system-materiaal-probe voor normal en
test-system-materiaal-recover voor de afzonderlijke koude readonly-VM, steeds
de werkelijk gekoppelde framebufferloader. De feature/build-helper moet door
root geïntegreerd zijn; afwezigheid of een oude bronhash geeft een expliciete
mislukte preflight, geen vooraf ingevuld PASS of terugval naar een browserproef.
native-system-materiaal en test-system-materiaal-scale200 zijn aparte rootprofielen;
deze CLI tekent die nog niet af.

`BUILD/manifest.json` bevat source_fingerprint_version3, actuele source_sha256,
kernel_feature, loader_feature=framebuffer, SDK-/document_session-proefflagsFalse
en `materiaal` exact gelijk aan `users/materiaal/build-report.json`:

```json
{
  "passed": true,
  "native_runtime_performed": false,
  "probe_controller": true,
  "apps": [
    {"name": "desktop-init", "sha256": "64 lowercase hex", "elf": {}},
    {"name": "keyring-service", "sha256": "64 lowercase hex", "elf": {}},
    {"name": "volume-service", "sha256": "64 lowercase hex", "elf": {}},
    {"name": "materiaal-app", "sha256": "64 lowercase hex", "elf": {}},
    {"name": "native-compositor", "sha256": "64 lowercase hex", "elf": {}},
    {"name": "input-source", "sha256": "64 lowercase hex", "elf": {}},
    {"name": "input-policy", "sha256": "64 lowercase hex", "elf": {}},
    {"name": "desktop-controller", "sha256": "64 lowercase hex", "elf": {}}
  ]
}
```

`elf` staat hierboven alleen ter schemaillustratie als leeg object; werkelijk
bewijs vereist exact het opnieuw berekende inspect_user-resultaat. Iedere
image moet ook letterlijk in de kernelbytes zijn ingebed. De build-PASS is
uitsluitend inspectie, geen runtimeclaim. Kernel en loader moeten exact de
ESP-kopie zijn; extra ESP-files, gewijzigde bytes, symlinks en hardlinks worden
geweigerd. Root bepaalt de NOS_DESKTOP_INIT_ELF/NOS_MATERIAAL_ELF/
NOS_COMPOSITOR_ELF/NOS_INPUT_SOURCE_ELF/NOS_INPUT_POLICY_ELF en controllergrants.

## Native seriële grammar

De bestaande zeven BOOT/EBS/kernel/KS/M2-markers moeten exact in volgorde staan,
gevolgd door MATERIAAL:MANIFEST:PASS. Dan ieder exact eenmaal:

- MATERIAAL:SOURCE:READY — werkelijk hardware-reset plus source-SYNC bevestigd.
- MATERIAAL:POLICY:READY — echte dienstloop/IPC-ingress gereed.
- MATERIAAL:COMPOSITOR:READY — echte bootstrapblits en geldige constructors.
- MATERIAAL:FOCUS:READY — echte actuele policy-FocusACK, na de drie servicegates.

Init's SYSTEM:READY1 is uitsluitend StartupReady en wordt niet als paint- of
hardwarebewijs aanvaard. App400 vereist een volledig actueel frame plus echte
INP1-response; Empty is op zichzelf geen source-/focusbewijs. Daarom zijn alle
vier afzonderlijke gates vereist voordat de runner enige gebruikersinput geeft.

De observer emit voor ieder nieuw volledig actueel frame:

```text
MATERIAAL:SNAPSHOT:BEGIN
MATERIAAL:READY:<u64 exact20 decimal digits>
MATERIAAL:PRESENTED_GEN:<u64>
MATERIAAL:FRAMES:<u64>
MATERIAAL:TILES:<u64>
MATERIAAL:KEYS:<u64>
MATERIAAL:MOUSE:<u64>
MATERIAAL:INPUT_ACKS:<u64>
MATERIAAL:SAVE_ACKS:<u64>
MATERIAAL:SAVED_REVISION:<u64>
MATERIAAL:SAVED_BYTES:<u64>
MATERIAAL:SAVED_SUM:<u64>
MATERIAAL:SCREEN:<u64>
MATERIAAL:DRAFT_DIRTY:<u64>
MATERIAAL:DRAFT_BYTES:<u64>
MATERIAAL:DRAFT_SUM:<u64>
MATERIAAL:OBJECT:<u64>
MATERIAAL:SCALE:<u64>
MATERIAAL:ERRORS:<u64>
MATERIAAL:SNAPSHOT:END
```

Ook alle overige u64's zijn exact20 cijfers en hoogstens u64::MAX. SCREEN1/2/3
is Materiaal/Editor/Confirm; geen draft geeft OBJECT=u64::MAX en length/sum0.
Een lege bestaande draft heeft FNV_OFFSET, zodat geen draft en leeg document
onderscheiden blijven. De observer publiceert geen voorgeschreven hostfase.
De host selecteert de fasen uit werkelijk waargenomen toestanden/generaties.
Volledige frames en generation moeten strikt oplopen; tiles omvatten minimaal
alle32×8-tegels van ieder frame. Verlopen/halve/duplicaatframes gelden niet.

Na de laatste echte screenshot geeft QMP één F12. App640 telt uitsluitend
werkelijk geauthenticeerde INP1 F12/Press, geen Enterrelease, F12Repeat of Release.
De kernel-probe mag alleen zijn eigen controllerbarrier zetten na werkelijk
Saved→latere List→latere Reopened met gelijk object/length/FNV en een daarna
nieuwere F12Press. Koude recovery vraagt lijst→werkelijk geopend document,
zonder synthetische save. Het is proeforchestratie, geen productshutdownknop.

Dan volgen MATERIAAL:F12_BARRIER:<1 als20digits> en
MATERIAAL:CONTROLLER:RELEASED. De controller doet de echte InitStopRPC;
Init stuurt appSTOP5, weigert dirty/saving via STOP_REFUSED6 en verwacht pas
na echte clientcleanup DONE3. Compositor-retire volgt na echte Focus0ACK en
echte wisblits. EXIT_SLOT/EXIT_CODE/EXIT_STEP-triplets voor app3/init0/controller7
moeten elk eenmaal code0/step0 melden. Hun onderlinge afloopvolgorde mag variëren.

Vervolgens precies deze20digitvelden: DISK_READS,DISK_WRITES,DISK_FLUSHES,
LIVE_PROCESSES,LIVE_HANDLES,FRAMES_RECOVERED. Normal vereist writes>0/flushes≥2;
recover schrijft/flusht0. Liveprocessen/handles0 en frameherstel1 zijn verplicht,
gevolgd door CLEANUP:PASS,CASE:PASS,M1:PASS. Raw QEMU-exit33 en geen timeout zijn
verplicht. Een host-QMPquit of kill kan deze afloop niet vervangen.

## Echte handelingen en schermcontrole

Normal: volledige ready-capture → mouseNieuw → volledige lege editor-capture →
14 echte toetsen → volledige dirty-capture → Ctrl+S → Saved-capture → Escape →
List-capture → Tab,Tab,Enter → Reopened-capture → F12 → echte clean afsluiting.
De brongecontroleerde ui-core-cursor begint op(0,0); bounded relatieve bewegingen
gaan zonder ingedrukte knop naar Nieuw, gevolgd door echte Down/Up. De app
ontvangt globale GOP-coördinaten, geen geïnjecteerde lokale UI-events.

Recover: nieuwe QEMU uit dezelfde bronversie, private kopie van een volledig
herberekende geslaagde normal-volume.after.img, QEMU-disk readonly=on plus echte
readonly-grants. Ready-capture → Tab,Enter → echte Open-capture → Ctrl+S →
begrijpelijke readonly-fout/capture met0saveACK → F12 en clean afsluiten.
Alle diskbytes vóór/na moeten exact gelijk blijven.

Ieder capturemoment bewaart de SHA256/lengte van het daadwerkelijk waargenomen
seriële prefix en het precieze screendump-commandonummer. Replay koppelt die
gegevens opnieuw aan de ruwe bytes, de volledige geordende QMP-transcriptie en
PPM. Beelden moeten echt800×600 zijn en tussen iedere handeling veranderen.
Pixels buiten assignedRect(16,48,768,536) blijven gelijk; opslaan behoudt het
daadwerkelijk gerenderde tekst-/caretgebied terwijl status/controlpixels wijzigen.
De font-/layout-/gebruiksinspectie blijft daarnaast een echte visuele beoordeling;
`visual_quality_acceptance=false` meldt dat die volledige releasepoort nog open is.

De onafhankelijke schijforakel hergebruikt volume_disk.choose/content (beide
tablekopieën, generaties, checksums, padding en chunks) en vergelijkt echte
opgeslagen naam/revision/bytes met de14 getypte bytes. Canaries buiten het grant,
andere objecten/domeinen, geschiedenis en collecties moeten onaangeraakt zijn.
De oude volume_disk.analyse accepteert uitsluitend haar binaire gegenereerde
fixture; deze proef verandert die functie niet en vermomt de getypte tekst niet.

## Rapport en padgrenzen

Alle writes blijven in een expliciet gekozen ROOT/build[/submap]/run/materiaal-*
child. Normal maakt altijd een nieuwe wegwerpimage en neemt geen diskargument.
Recover accepteert alleen een gegenereerde parent-rundir, herberekent diens
volledige bewijs en neemt uitsluitend een nieuwe kopie van diens afterimage.
Geen persoonlijke disks, NAS, accounts, shares, hostnetwerk of fysieke writes.
Private QMP-pipe is standaard; Unixsocket mag alleen binnen de eigen korte runmap.

result.json bevat schema_version1, passed, scope,case, source_commit/source_sha256,
exact command/cwd/profile, echte QEMU-entry/implementatiehashes, frozen_artifacts,
before/after SHA256, stages en artifact_sha256. `stages` is een geordende lijst
van `{phase,snapshot,serial_size,serial_sha256,qmp_index}`. `observed` komt uit
herberekening van rawserial/QMP/pixels/disk. Iedere fout houdt passedFalse;
een nieuwe mislukte preflight trekt een oud samenvattings-PASS vooraf in.
De runmap bewaart exacte ESP/ELF/build-/firmwareinputs, rawserial/stderr/QMP,
echte PPM's en before/afterdisk. Geen gegenereerde binaries in Git.

## Controlecommando's en resterend plan

Hostvalidering (geen nativeclaim):

```sh
python3 -B -m unittest discover -s tools/tests -p test_materiaal_probe.py -v
```

Na rootintegratie eerst het actuele dedicated probeimage bouwen met bestaande
tools/dev.py/tools/build.py en een eigen NOS_BUILD_DIR/NOS_TARGET_DIR. Daarna:

```sh
python3 -B tools/run_materiaal_probe.py --case normal --build-dir build/materiaal-probe --vga-rom /explicit/existing/VGA.rom --vga-rom-sha256 EXACT_SHA256
python3 -B tools/run_materiaal_probe.py --case recover --build-dir build/materiaal-probe --parent build/materiaal-probe/run/materiaal-normal-UNIQUE --vga-rom /explicit/existing/VGA.rom --vga-rom-sha256 EXACT_SHA256
```

De tweede opdracht vereist eerst een nieuwe recoverbuild met dezelfde bevroren
bron en het gekoppelde recoverfeature; hergebruik geen normal-kernel als
readonlybewijs. Bovenstaande ROM/SHA/UNIQUE zijn expliciete invulplaatsen voor
bestaande gecontroleerde artifacts, geen geleverde of geteste native uitkomst.

Nog uit te voeren na de eerste native keten, zichtbaar open:

| Scenario | Echte handeling en verwacht onafhankelijk bewijs |
|---|---|
| Koude read-view | Nieuwe VM na volledig beëindigde normal-VM; dezelfde getypte bytes werkelijk via provider geopend, hashes en readonly-I/O0. |
| Dirty-close/cancel | Typ nieuw werk, Escape toont echte bevestiging; doorwerken behoudt draft, expliciet annuleren van upload stuurt ABORT, geen extra revision. |
| Readonly | Open echte bytes, Ctrl+S wordt begrijpelijk geweigerd; geen COMMIT-ACK, geen diskmutatie; native capabilitygrens apart blijven controleren. |
| 200% | Dedicated scale200-image, echte font-/focus-/pointercontrole op minimaal ondersteund viewport; geen fallback naar100% om groen te halen. |
| Savepointer buiten | Down op Save, beweging/release buiten assignedRect en heldreentry; geen BEGIN/COMMIT. Alleen nieuwe echte Down/Up binnen mag opslaan. |
| Fout/herstel | Source-/policy-/surfaceverlies, verloren ACK, vol volume en mislukte I/O behouden draft en melden echte toestand; geen onzichtbare actie of valse Saved/Presented. |

Op dit broncheckpoint zijn de rootbootstrap/observer/controller nog externe
integratieafhankelijkheden. De runner heeft geen native VM-PASS geclaimd. Alle62
releasecriteria en de bestaande voortgangsrubric blijven onverminderd gelden.

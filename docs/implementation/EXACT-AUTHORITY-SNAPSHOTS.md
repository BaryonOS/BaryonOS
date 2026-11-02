# Exacte autoriteitssnapshots

2 november 2026; implementatiebasis `4638c7c`. Afgebakende optimalisatie van de
bestaande M4- en IPC-wait-controles. Geen nieuwe rechten, mutatiecache,
UI-acceptatie of U02-punten.

`m4::syscall` berekende vóór ieder verzoek volledige `Stats`; een fout vereiste
een tweede scan. De diagnostische digest doorloopt circa74.569 getypeerde
woorden, inclusief ongebruikte queuecellen. `State.audit()` staat juist niet
op dit gewone IPC-pad. De bestaande `stats()`, `state_digest()`, referentiedigest
en alle rapportage blijven ongewijzigde verse scans.

## Invariant en opslag

De nieuwe `AuthoritySnapshot` bevat een private, volledige `Engine`. Een
vooraf gereserveerde kernelstatic ontvangt via `snapshot_into` rechtstreeks
alle tien `Copy`-arrays en `last_pid`. De exhaustieve destructurering zonder
`..` en `deny(unused_variables)` laten veldgroei compilatie falen totdat de
kopie bewust wordt uitgebreid. De bestaande afgeleide `Engine::eq` vergelijkt
alle getypeerde velden, ook generaties, dode slots en inactieve berichtcellen.
Padding wordt niet vergeleken. Er is geen digestcollisie of afhankelijkheid van
mutatiehooks bij deze foutcontrole; de kopie wordt nooit teruggezet of als
actieve autoriteit gebruikt.

Op het gecontroleerde x86-64-profiel kost de snapshot461.816B, alignment8.
`AuthoritySnapshot::new()` staat uitsluitend in statische kernelopslag, niet
als grote returnwaarde, clone of tijdelijke `Engine` op de16KiB-servicestack.
De scalar `native-ipc-wait`-build toont voor `snapshot_into`40B stackruimte plus
48B opgeslagen registers; de gewiste snapshotlease gebruikt344B plus48B.
Dit is inspectie van deze functies, geen volledige maximale kernelstackanalyse.

Eén gedeelde `SnapshotLock` geeft een niet-blokkerende exclusieve lease. De
kernel controleert IF=0 en neemt de lease vóór een mutable bufferborrow.
Herintreding stopt; zij hergebruikt nooit een nog actieve snapshot. Geen borrow
van de live `Engine` overleeft `dispatch`. M4 vergelijkt op iedere fout;
IPC-wait vergelijkt op **ieder** resultaat, inclusief succesvol parkeren.
Alle bestaande pointer-, capability-, return- en afzonderlijke useroutputchecks
blijven aanwezig.

Een snapshot dupliceert tijdelijk IPC-inhoud. Vóór vrijgave overschrijft de
kernel daarom alle1024 gekopieerde `Message`-waarden met een geldig leeg
bericht via getypeerde volatile writes: headers, descriptors en alle payloads.
Er wordt geen Rust-structpadding geïnterpreteerd. Dit gebeurt ook vóór een
assertie over een gedetecteerde autoriteitswijziging: panic-abort voert geen
Drop uit. Daarna wordt de lease vrijgegeven, vóór observers of scheduling.
Niet-inhoudelijke node-/objectmetadata blijft private kernelopslag en wordt
bij de volgende capture overschreven. De wipe voegt320KiB aan begrensde writes
per observatie toe; die kosten mogen niet uit een prestatiemeting verdwijnen.

## Gerichte verificatie

`cargo +1.90.0 test --offline --locked -p capability-core`:66 tests geslaagd,
één bestaande benchmark genegeerd, plus7 IPC-wait-tests geslaagd. Vijf nieuwe
tests vergelijken snapshots met volledige host-clones en verse orakels,
veranderen ieder autoriteitsveld inclusief slot0 en de laatste inactieve
queuecel, controleren foutpaden en onverwachte writes, wissen alle
berichtvelden en weigeren herintreding. De clones staan alleen op een grote
hostteststack. Clippy voor capability-core met
`--all-targets -- -D warnings` slaagt. Een aanvullende kernel-Clippycontrole
met `--features native-ipc-wait --target x86_64-unknown-none -- -D warnings`
stopt op zeven bestaande meldingen in ongewijzigde `m3.rs`, `paging.rs` en
`main.rs` (useless_conversion, needless_return, manual_is_multiple_of,
too_many_arguments en not_unsafe_ptr_arg_deref). Geen melding raakt deze diff;
de native releasebuild en QEMU-gate slagen. Volledige strenge kernel-Clippy
wordt daarom niet als groen gerapporteerd; log: `kernel-clippy.txt`.

De bestaande `native-ipc-wait`-kernel en ELF-inspectie bouwen. De echte
QEMU-run slaagt: rawexit33, runnerexit0,5,22344s, beide fasemaskers63/60,
14 geblokkeerde waits,7 ready-wakes,6 time-outs,1 foutwake,181 idle-IRQs en
volledige cleanup. Profiel: QEMU10.0.13/q35-10.0/TCG/qemu64/1CPU/256MiB,
geen netwerk of fysieke disks. Bron-v3:
`2cc2e7604a669660fd6b79ec70f8c9af8b4586f060b8330a258b01f7fc7dc73f`;
kernel-SHA256:
`4104dba1c521142325d74de57ea06f6903adefadc8bc37e287d595950b89756b`.
Dit bewijst native IPC-/wait-voortgang binnen deze gate; het meet geen
vergelijkbare syscalllatentie, desktoplatentie of volledige U01-keten.

Uitvoering vanuit de eigen werkboom, met de bestaande geactiveerde toolchain:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools \
NOS_BUILD_DIR="$PWD/build/exact-snapshot/native" \
NOS_TARGET_DIR="$PWD/build/exact-snapshot/native-target" \
python3 -B tools/run_ipc_wait.py
```

Ruwe logs: `build/exact-snapshot/logs/{host,clippy,native-wait}.txt`;
bron-/artifactgebonden rapport: `build/exact-snapshot/native/ipc-wait-report.json`.
De onafhankelijke tweede-agentdiffreview vond twee herstelde punten: wissen
vóór een mogelijke panic-abort en het verwijderen van een onjuiste repr(C)-
commentaarclaim. Na hercontrole geen resterende blocker binnen deze scope;
dit is geen menselijke audit.

Een voorbereidende hostproef met9×128 herhalingen op acht processen, één
endpoint en acht berichten gaf medianen114.102ns voor één Stats-scan,
214.314ns voor twee scans,45.032ns voor snapshot+wissen en134.736ns voor
snapshot+exactEq+wissen. Zij gebruikte Rust1.90/O3, de eigen kernel-memory-
intrinsics en scalar gegenereerde proefcode; de host-target waarschuwt dat
SSE2 normaal deel is van zijn ABI. Dit is uitsluitend een onderzoekssignaal,
geen betrouwbare native latency- of hardwarevergelijking. Ruwe proefbestanden
staan tijdelijk onder `/tmp/nieuw-os-authority-snapshot-design/`; de passende
acceptatie berust op de echte native gates en brongebonden artifacts.

## Grenzen en terugname

Dit is uitsluitend het bestaande single-CPU/IF0-contract. SMP, herintredende
callbacks of nieuwe interruptpaden vereisen opnieuw ontwerp. Een falende
IF0-controle of hardware-/kernelcrash kan de VM stoppen; zij verleent geen
nieuwe autoriteit. De opslag is nooit een userspace-mapping of DMA-buffer.
Terugname herstelt uitsluitend de M4/wait-Stats-vergelijkingen; de oude verse
orakels blijven beschikbaar. Er is geen persistente formaatmigratie.

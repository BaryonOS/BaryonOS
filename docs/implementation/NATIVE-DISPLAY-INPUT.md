# Native display- en invoertransport — uitvoering

Basis `61ea7278390d5235a8bc024cbf084222a7b0c658`, 30 oktober 2026.
De hoofdagent heeft DISPLAY=4096, INPUT=8192 en opt-in-ABI 1.8 gereserveerd.
Deze taak behoudt G0 als afzonderlijke regressie en maakt geen releaseclaim.

Eigenaar van de nieuwe transporten en capability-engine: `git_build_audit`.
Root bezit boot/framebufferextractie, centrale syscall-/SDK-hooks, build,
workspaces, manifestgrants en integratie. `native_ui_apps` bezit de ring-3
renderer/controller. De memoryagent blijft eigenaar van m3/paging/Sys4.

De kernel krijgt uitsluitend vast hardwaretransport: geen fonts, GUI-parser,
documentbeleid, focuspolicy of toetsenlayout. De framebuffer blijft
supervisor-RW/NX/UC-MMIO, buiten RAM-/heap-/DMA-eigenaarschap. Eén getypeerde,
niet-delegeerbare manifestcapability geeft de compositor transporttoegang.
Een tweede bron krijgt geen autoriteit. Sluiten/exit en generatieuitputting
mogen oude handles niet opnieuw geldig maken.

Display: maximaal 32×8 pixels per operatie, packed little-endian u32
`0x00RRGGBB` (bytes B,G,R,0). Geometry geeft alleen width, height en epoch;
geen fysiek adres, vensterpointer of device-stride. Alle descriptor- en
payloadplannen worden volledig gevalideerd en naar private RAM gekopieerd;
alle rijen/kleuren worden gecontroleerd vóór de eerste VRAM-write.

Input: één beschermde hardwarebron krijgt alleen een vaste begrensde reset,
status/data-read en readinessreport. Geen raw-port-/commandinterface voor
apps. De bron configureert beide i8042-kanalen, Set 2 met controllervertaling
naar Set 1 en muis-ID 0. Ready vereist echte ACK/BAT/ID en gecontroleerde
controllermode. Outputplannen worden vóór statusread/FIFO-consumptie/commands
gevalideerd. Reset is een begrensde state machine; pending betekent nooit
Ready. AUX, parity, timeout, bekende verloren bytes en epochs blijven zichtbaar.
De ring-3-bron levert daarna echte IPC aan de bestaande input-core; een
payload of lokaal geconstrueerde `Received` verleent geen bron-PID/focusrecht.

Native acceptatie blijft open totdat root de hooks integreert en echte QMP-
toetsen/muis, gerenderde pixels, weigering, stale handles, ongeldige usercopy,
resetfouten en cleanup op het actuele geïntegreerde image uitvoert. Pure
driver-/codectests zijn geen vervanging voor dat bewijs.

## Vast wire-/syscallcontract 1.8

Alle overige registerargumenten zijn nul; errors leveren RDX=0 en veranderen
de caller-output niet. De kernel kiest de echte PID uit de syscallcontext.
De sourcehandles zijn geen process-ID's of fysieke adressen.

| Nummer | Argumenten | Succes-RDX |
| --- | --- | --- |
| 0x80 DisplayInfo | handle, outputpointer, 32, 0, 0, 0 | 32 |
| 0x81 DisplayBlit | handle, descriptorpointer, pixelpointer, pixelbytes, 0, 0 | width×height pixels |
| 0x82 InputResetBegin | handle, 0, 0, 0, 0, 0 | nieuw ticket >0; alleen gestart |
| 0x83 InputResetPoll | handle, ticket, outputpointer, 32, 0, 0 | 32, uitsluitend Ready |
| 0x84 InputRead | handle, outputpointer, 32, 0, 0, 0 | 32, één echte status/data-sample |

`EAGAIN` bij Read betekent geen FIFO-data; bij ResetPoll betekent het een nog
lopende geautoriseerde reset. ResetPoll mag na geldige outputpreflight stappen
uitvoeren en bytes drainen, maar schrijft bij Pending geen Ready of output.
Er is geen succesfallback en geen automatische tweede reset bij een onduidelijk
antwoord. Een verkeerde ticket/owner kan geen andere reset voortzetten.

De vier vaste records zijn precies 32 little-endian bytes:

- Info: width u32@0, height u32@4, epoch u64@8, maxwidth u16@16=32,
  maxheight u16@18=8, format u32@20=1, acht nulbytes vanaf24.
- Blitdescriptor: epoch u64@0, x/y u32@8/12, width/height u16@16/18,
  flags u32@20=0, bytecount u32@24, reserved u32@28=0.
- ResetReady: ticket u64@0, epoch u64@8, discarded u32@16, operations u32@20,
  profile u32@24=1, flags u32@28=0.
- Raw input: epoch u64@0, sequence u64@8, status u8@16, data u8@17,
  nulbytes vanaf18. Status bevat werkelijk OBF/AUX/parity/timeout; de kernel
  herschrijft geen keyboardfout naar een printable sentinel en gooit geen
  muisbytes weg. Na foutstatus wordt opnieuw reset vereist.

De SDK-crate deelt de vaste pure `wire`-module met de kernel. De kernel gebruikt
`default-features=false`: geen user-runtime, allocator of GUI in die dependency.
De optionele/default `client`-laag gebruikt de bestaande SDK-runtime. Alleen
root voegt de centrale native wrappers toe; de client dupliceert geen int80-asm.
De getypeerde Source-wrappers controleren representatie, verlenen geen rechten
en sluiten geen geleende/shared handles. De eigenaar sluit de manifestgrant
expliciet. Een app kan de globale source-objecten niet zelf aanmaken.

## Begrensde PS/2-reset

De pure `drivers/ps2`-core werkt boven vier vaste getypeerde portoperaties.
Iedere ResetPoll doet maximaal acht state-machineacties, dus maximaal zestien
portoperaties. De globale grenzen zijn 8.192 operaties, 256 drainbytes, maximaal
twee RESEND-herhalingen, 64 monotone PIT-ticks per stap en 1.000 per reset.
Er is geen kernel-pollspin en geen TSC-/host-/walltimefallback.

De gesloten commandosequentie schakelt interfaces uit en draint begrensd,
schakelt IRQ1/IRQ12 uit, initialiseert toetsenbord met ACK+BAT, schakelt scanning
tijdelijk uit, stelt Set2 in, initialiseert de muis met ACK+BAT+ID0 en defaults,
zet controllervertaling naar Set1 aan, controleert de commandobyte en stelt
scanning met F4 en echte ACK's vast vóór Ready. RESEND van een muiscommando herhaalt ook D4.
Onverwachte antwoorden, foutstatus, ontbrekende ACK/BAT/ID, verkeerde mode,
budget/timeout en counteroverflow leveren geen Ready. Andere-kanaalinput tijdens
reset wordt expliciet als discarded geteld en gedekt door de nieuwe epoch/SYNC.
Er is geen claim dat i8042 ieder mogelijk ongemerkt fysiek byteverlies meldt.

Tickets en epochs lopen niet rond. Raw sequences beginnen per succesvolle
reset op1; uitputting faalt vóór FIFO-read. Sluiten/exit of een nieuwe globale
objectgeneratie mag een oude Ready/reset niet erven. `detach(pid)` is alleen
voor bewezen procesdood/volledige principal-retirement, niet voor een vertraagd
oud-grant-event dat een nieuwe sessie van dezelfde levende PID zou wissen.

Native hookpunten, door root te verbinden:

- `display::prepare()` na echte framebuffer-map; `ps2::prepare()` doet geen I/O.
- `display::syscall(state,slot,number,args)` voor0x80/81;
  `ps2::syscall(...)` voor0x82–84, vóór generieke M4-fallback.
- `display::detach(pid)` en `ps2::detach(pid)` bij echte principal-cleanup.
- Manifest-only `Engine::create_display(pid)` / `create_input(pid)` en
  `check_display` / `check_input` met echte caller-PID. Statische UI-tests
  kunnen het nieuwe recht niet als raw port-I/O of RAM-framerecht gebruiken.
- Root bouwt/versiet ABI1.8 en verleent sources alleen aan compositor en
  hardware-bronproces. Die bron levert SYNC/SAMPLE via echte IPC aan input-core;
  compositor- en lifecycle-ingresses blijven afzonderlijke geautoriseerde PIDs.

Kernelcounters zijn display `[blits,pixels,denied]` en input
`[started,ready,samples,denied,discarded]`. Herhaald poll van hetzelfde succesvolle
resetticket bevestigt dezelfde Ready zonder extra hardwarewerk of extra
readycounter. Een achterblijvend beeld na compositoruitval is nog geen veilige
sessievergrendeling/herstelervaring; die productpoort blijft uitdrukkelijk open.

## Controle en bronnen

De capability-slice heeft47 hosttests, inclusief globaal exclusieve quota, PID/type,
rechteverruiming, derive/transfer/revoke, close/reuse/exit, maximumgeneratie-
retirement en atomaire slot/nodequota. Een afzonderlijke reviewer herhaalde die
en vijf aanvullende tegenproeven (52 groen), zonder concrete blocker.

PS/2 heeft acht hosttests en geslaagde Clippy/freestanding librarybuild; de SDK
heeft vijf hostcodectests en geslaagde Clippy/pure-wire targetbuild. De volledige
native SDK-laag en de nieuwe kerneladapters wachten op de door root beheerde
centrale hooks/dependencies. Deze tekst tekent die native grens niet af.

Commando's vanuit deze worktree, met `LOCALAPPDATA=/workspace/nieuw-os/.tools`
en eigen targetmappen:

```text
python3 -B tools/dev.py cargo +1.90.0 test --locked --offline -p capability-core
python3 -B tools/dev.py cargo +1.90.0 test --manifest-path drivers/ps2/Cargo.toml --locked --offline
python3 -B tools/dev.py cargo +1.90.0 clippy --manifest-path drivers/ps2/Cargo.toml --locked --offline --all-targets -- -D warnings
python3 -B tools/dev.py cargo +1.90.0 build --manifest-path drivers/ps2/Cargo.toml --locked --offline --release --target x86_64-unknown-none --lib
python3 -B tools/dev.py cargo +1.90.0 test --manifest-path sdk/display-client/Cargo.toml --locked --offline
python3 -B tools/dev.py cargo +1.90.0 clippy --manifest-path sdk/display-client/Cargo.toml --locked --offline --all-targets -- -D warnings
python3 -B tools/dev.py cargo +1.90.0 build --manifest-path sdk/display-client/Cargo.toml --no-default-features --locked --offline --release --target x86_64-unknown-none --lib
```

De gecontroleerde primaire emulatorbron is QEMU10.0.13:
[`hw/input/ps2.c`](https://github.com/qemu/qemu/blob/v10.0.13/hw/input/ps2.c),
SHA-256 `fe898d87af4c87c138767b272c8ebcb0932927fcc889b4a82ac50e5086cda715`,
en [`hw/input/pckbd.c`](https://github.com/qemu/qemu/blob/v10.0.13/hw/input/pckbd.c),
SHA-256 `e48dc36f8ddd165877ac647408dd4f10ab782dc9061e25f78c20a975abb5eaa4`,
gelezen30oktober2026. Geen QEMU-code overgenomen en geen nieuwe externe
dependency. Fysieke PS/2-/GPU-hardware en universele deviceondersteuning zijn
niet getest. Root behoudt G0- en alle bestaande releasegates naast de nieuwe
native bedieningsproef; de volledige Materiaal- en releaseacceptatie blijft open.

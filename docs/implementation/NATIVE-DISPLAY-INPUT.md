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

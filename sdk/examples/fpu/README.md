# Native x87/SSE-contextproef

Dit opt-in profiel valideert één kernelmechanisme, geen volledige testrelease.
`fpu-context` biedt eager FXSAVE64/FXRSTOR64 voor elk processlot;
`native-fpu` voegt twee afzonderlijke ring-3-ELFs en een begrensde observer toe.
Het 22-woord interruptframe, procesadresruimten, W^X en capability/IPC-contracten
blijven behouden. Dit profiel heeft geen netwerk, persoonlijke data of schrijfbare
externe schijf.

De kernel controleert CPUID.FPU/FXSR/SSE/SSE2, zet CR0.MP/NE, wist EM/TS,
zet CR4.OSFXSR/OSXMMEXCPT en wist OSXSAVE. AMD EFER.FFXSR wordt gewist en
teruggelezen zodat CPL0-FXSAVE de XMM-registers niet mag overslaan. Elke entry
uit ring3 bewaart hardwaretoestand vóór Rust. De laatste terugkeerassembly
herstelt de geselecteerde toestand na alle Rust, direct vóór GPR-pops en iret.
CPL0-idletimerreturns gebruiken deze userrestore niet. Er is geen lazy TS-sharing,
AVX/XSAVE, SMP of drijvendekommagetallen in kernel-Rust.

Elk van de zestien processlots bezit een permanente, supervisor-only,
16-byte-uitgelijnde 512-byte-image: samen 8192 bytes. Spawn en hergebruik wissen
de volledige image en zetten alleen FCW=037f en MXCSR=1f80. FTW, payloads,
FIP/FDP en XMM0–15 starten nul. Alleen de kernel en echte FXSAVE kunnen deze
restorebeelden schrijven. Cleanup wist alle bytes; geen userpointer wordt over
een planningspunt vastgehouden. FNINIT alleen zou fysieke registerdata behouden.

De apps gebruiken bewust directe, gecontroleerde int80-syscalls. Dat is een
architectuurproef, geen tweede SDK-productimplementatie. Vóór enige FPU/SIMD-
datainstructie controleert iedere nieuwe instantie de hardware via FXSAVE64:
FCW, FSW, FTW, MXCSR, FIP/FDP, acht 80-bit payloads en alle zestien XMM-registers.
Daarna krijgt ieder register een processpecifieke waarde en ieder x87-slot een
verschillend getal. B heeft bovendien andere virtuele x87-instructie- en
operandadressen, zodat de FIP/FDP-controle een verwisselde pointertoestand kan
onderscheiden. FCW/MXCSR hebben verschillende afrondingsmodi. De apps
vergelijken deze echte registertoestand herhaald na syscalls en busyloops met
PIT-preëmptie. Gereserveerde/paddingbytes worden niet als architectuurdata getest.

Het afzonderlijke profiel `test-fpu-wait` combineert dezelfde proef met
`ipc-wait`/ABI1.10. Alleen daar vervangen de apps iedere yield door syscall0x22
met nul voorwaarden en een absolute deadline twee ticks na de vorige gemeten
klok. De hele registerbank wordt ook na terugkeer uit die wacht vergeleken.
De kernel verlangt minstens drie echte blokkeringen, overeenkomende
deadline-wakes, werkelijk getelde idle-IRQ's en geen ready-/error-wakes.
Het aantal wachtcalls moet exact gelijk zijn aan het aantal geverifieerde
lussen; eventuele reeds verstreken deadlines worden afzonderlijk geteld.
Timerpreëmptie per app, de AVX-fault, survivor, verse PID en volledige cleanup
blijven verplicht. Een nog wachtende survivor is levend, niet ten onrechte
een fout; hij moet na zijn deadline aantoonbaar verder werken vóór hergebruik.
Het oorspronkelijke ABI1.6/yieldprofiel blijft afzonderlijk bestaan.
Alle IRQ0's tellen voor de monotone klok; alleen CPL3-entry's bewaren/herstellen
een user-FPU-bank. Daarom is het totale IRQ-getal exact user-A + user-B +
idle-IRQ's, terwijl saves exact user-A + user-B + syscalls + de echte fault
moeten zijn. CPL0-idle-returns slaan de FXSAVE/FXRSTOR-route bewust over.
Kernel en host verifiëren beide deze relaties; idle mag geen userpreëmptie
of extra FPU-save voorstellen.

A veroorzaakt na minstens twaalf PIT-ticks een echte unsupported-AVX #UD.
CPUID.AVX moet op dit qemu64-profiel afwezig zijn. B moet daarna aantoonbaar
verder werken. Pas vervolgens wordt slot0 opnieuw opgebouwd met verse PID3;
die instantie controleert eerst schone hardwaretoestand en verifieert opnieuw
minstens twaalf ticks. Na de normale exits zijn alle procesframes onafhankelijk
teruggeteld en alle 8192 contextbytes nul. De observer levert geen appresultaten.

De apps toetsen ook syscall3: ABI1.6, zes afzonderlijk geweigerde nonzeroargs,
nondecreasing PIT-ticks en echte voortgang over yield/timerwissels. De openbare
SDK biedt `native::monotonic_ticks()` en `clock::MONOTONIC_HZ=100`. Het is een
boot-monotone, bij u64::MAX verzadigende IRQ-teller; geen walltime, certificaatklok,
realtimegarantie of accounting over suspend. Oude profielen zonder
`monotonic-clock` houden hun ABI en ENOSYS voor syscall3.

Bouw via het gedeelde `tools/build.py --kernel-feature native-fpu` nadat dat
profiel is geïntegreerd; de helper `tools/run_fpu.py::build_users()` bouwt de twee
zelfstandige workspacebins en retourneert `NOS_FPU_A_ELF`/`NOS_FPU_B_ELF` plus
het inspectierapport. `python3 tools/run_fpu.py` valideert de bootsequentie,
procesresultaten, onafhankelijke tellerrelaties, exacte ELFs in de kernel,
bronfingerprint, loader/kernel/firmwarehashes, daadwerkelijke QEMU-commandoregel
en ruwe seriële uitvoer. Hosttests: `python3 -m unittest discover -s tools -p test_fpu.py`.
Gebruik voor de combinatie `--kernel-feature test-fpu-wait` en dezelfde
runner met een afzonderlijke `NOS_BUILD_DIR`. De manifestkeuze bepaalt welk
strikt orakel geldt: normaal bewijs kan de wachtvariant niet goedkeuren.

Een eerdere proef met `LDMXCSR(80000000)` bereikte de opzettelijke failure105:
dit QEMU/TCG-profiel isoleerde daarbij geen #GP en is daarvoor geen bewijs.
De uitgezonden instructie en operand zijn met disassembly/rodatabytes gecontroleerd.
De ruwe mislukte proef blijft in `verification/mxcsr-tcg-gap/` bewaard. De gekozen
negatieve acceptatie is daarom echte unsupported-AVX #UD. Er is geen #GP-claim.

Normatieve ontwerpverwijzingen: Intel SDM vol.1 “Managing State Using the FXSAVE
and FXRSTOR Instructions”, vol.2 CPUID/FXSAVE/FXRSTOR/LDMXCSR en vol.3 CR0/CR4;
AMD APM vol.2 EFER.FFXSR en x87 pointer-restorationvoorwaarden. De officiële
Intel-PDF is tijdens dit werk door de netwerkproxy met HTTP403 geweigerd; er
wordt geen nieuw opgehaalde primaire-specificatiecontrole geclaimd. De lokale
secundaire bron is gelezen: OSDev: FPU, W002427, revisie29122; de onafhankelijke
review controleerde ook OSDev: FXSAVE, W001452, revisie30264. Vooral fysieke AMD-
processors vereisen een afzonderlijke CPU-/erratapolicy voor conditionele
FIP/FDP/FOP-restoration. QEMU-bewijs geldt niet voor fysieke hardware.

Open: productprofielintegratie, engines, bredere CPU-/FPU-exceptiegevallen,
ontbrekende-FXSR-bootproef en fysieke hardwarevalidatie. De testrelease en de
volledige browser/media/AI-acceptatie blijven open.

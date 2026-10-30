# GUI-BOOT-INIT-1: afzonderlijke native init/supervisor

Dit contract beschrijft de nieuwe CPL3-component `sdk/examples/desktop-init`.
De eerste componentcontrole is een host-protocolproef plus echte scalar
ELF-build, geen native desktopacceptatie. Kernelbootstrap, grants, imagebouw,
werkelijke QMP-bediening en volledige releaseacceptatie zijn eigendom van de
hoofdintegratie en blijven afzonderlijk te bewijzen.

## Autoriteit en bootmetadata

Het zevenprocessenprofiel gebruikt init1, keyring2, volume3, Materiaal4,
compositor5, inputsource6 en inputpolicy7. Init heeft uitsluitend RegistryRECV,
AppControlSEND en CompositorControlSEND. Geen persoonlijk volume, keyring,
netwerk, display, rawinput, fysiek geheugen, LaunchImage of ChildProcess.
Native IPC komt rechtstreeks uit de SDK. De gedeelde inputdaemonlibrary levert
alleen de bestaande Ipc/NativeIpc-adapter en capcleanup; er wordt geen driver,
policy- of display-I/O door init uitgevoerd.

Private DATA-startpagina:1024 bytes op0x40010000, alle velden u64:

| Byte | Waarde |
| --- | --- |
|32|`u64::from_le_bytes(*b"NOSDINI1")`|
|40|BootGeneration>0|
|48|RegistryRECV|
|56|AppControlSEND|
|64|CompositorControlSEND|
|72|optionele ControllerControlRECV, anders0|
|80/88/96/104|werkelijke App/Comp/Source/Policy PID, voor dit profiel exact4/5/6/7|
|112|SurfaceEpoch>0|
|120|PolicyEpoch>0|
|128|verwachte NativeDisplayEpoch uit kernel-GOP, >0|
|136|verwachte ClientID>0, voor de manifestbinding; geen bevoegdheid uit payload|
|144|optionele ControllerReplySEND, anders0|
|152|optionele verwachte ControllerPID, apart proces boven7; anders0|

De drie controllerwaarden zijn gezamenlijk aanwezig of gezamenlijk0. Alle
handles zijn representatief geldig en onderling verschillend; de kernel
controleert type/rechten/houder. Velden160..1023 zijn aanvankelijk0. PIDs of
replyroutes worden nooit uit een request gekozen. De normale zevenprocessen-
variant heeft geen testcontroller en init stopt daar niet automatisch.

## Gereedheid en echte focus

Init ontvangt op één RegistryRECV maximaal vier berichten per ronde. Een
IPDREAD1-bericht moet de echte kernelafzender, nul ontvangen caps, exacte tag,
32B-lengte, `IPD1`, versie1, rol, epoch en BootGeneration dragen:

- SourcePID6→rol1, PolicyEpoch, werkelijk RawEpoch>0.
- PolicyPID7→rol2, PolicyEpoch, RawEpoch0.
- CompositorPID5→rol3, SurfaceEpoch en exact verwachte NativeDisplayEpoch.

De source mag dit volgens zijn eigen contract pas na echte hardware-Ready en
canonieke SYNC-ACK melden; de compositor pas na de volledige eigen chrome en
lege toegewezen apprect via echte displayblits. Init leest hun geheugen niet
en vervangt die afzonderlijke native eigenschappen niet door eigen claims.
Een gefabriceerd payloadveld of tag verleent geen afzenderautoriteit.

AppPID4 stuurt SYSTEM-tag0x53595354454d0001 met payload[1] als **StartupReady**.
Dat is geen volledig getekend frame, succesvolle document-I/O of bewijs van
focus. Pas met alle drie IPD-rollen én AppStartupReady publiceert init precies
één CMP1 FocusStart, requestID1, gebonden aan SurfaceEpoch/BootGeneration.
Send-EAGAIN bewaart dezelfde bytes. Na sendOk wacht init op de echte
compositorreply; het request wordt niet opnieuw gepubliceerd.

Alleen de canonieke, caploze reply van CompPID5 met juiste opcode|0x8000,
ID1, epoch, generatie, status0 en stateFocused1 laat SYSTEM GO[2] naar de app
gaan. GO telt pas na echte SDK-sendOk. InitReady betekent uitsluitend deze
GO-publicatie na gecontroleerde afhankelijkheden en focus. AppReady400,
PresentedGeneration, daadwerkelijke UI- en opslagscenario's blijven aparte
observer-/bedieningscriteria. Een onverwachte AppDONE buiten een actieve
STOP is geen toestemming om het systeem als afgesloten te beschouwen.

## Optionele STOP-route en weigering

De optionele native controller bezit alleen zijn vaste initControlSEND en
persoonlijke ReplyRECV. DINITCTL is tag `*b"DINITCTL"`, payload48B:
`DIN1`@0,versionu16=1@4,opcodeu16=Stop1@6,requestIDu64@8,
SurfaceEpoch@16,BootGeneration@24,statusu64@32,reservedu64@40. Requeststatus en
reserved zijn0. RequestID is niet0 en strikt monotoon. Exacte kernel-PID en de
vaste controllerroute zijn vereist. Replyopcode is0x8001; status is
Stopped0, Refused1 of Failed2, reserved0. Een reply krijgt dezelfde bindings
en uitsluitend de vaste ControllerReplySEND.

Init publiceert SYSTEM STOP[5] eenmaal en wacht op AppPID4. App[6]
STOP_REFUSED betekent onopgeslagen/bezig werk: init antwoordt Refused en blijft
Running; AppDONE/Retire/DONE worden daarmee niet aangetekend. App[3] DONE is
alleen geldig binnen deze actieve STOP na de publicatie. Dan volgt CMP1
Retire ID2. Pas de echte compositor-Retired-ACK (Focus0 plus werkelijk zichtbare
apprect-wisblits volgens het compositorcontract) laat Stopped volgen.
Een ontbrekende of ongeldige Retire-ACK geeft Failed en nooit Stopped.
Pending controllerreplies blijven bij EAGAIN ongewijzigd; geen tweede STOP,
Retire of impliciet opnieuw opslaan. Reeds gequeue'de berichten kunnen niet
uit een andere queue worden teruggehaald. Het bestaande eenbyte-Appprotocol
correleert op fase en één uitstaande STOP; het is geen algemene transactiedienst.

## Afzonderlijk testcontrollerproces

Alleen `native-system-materiaal-probe` kan een achtste CPL3-procesPID8 krijgen.
Dit is expliciete proeforchestratie, geen gebruikersfunctie of extra recht van
een gewone app. De normale desktopvariant blijft zeven processen.
Controllermetadata:32`NOSDCTL1`,40BootGeneration,48InitControlSEND,
56eigenReplyRECV,64InitPIDexact1,72SurfaceEpoch,80ownPIDexact8.
88..399 en464..1023 zijn gereserveerd0.

De controller leest uitsluitend zijn eigen barrierwoord400. Initieel0; de
kernel-testobserver mag dit op1 zetten **na** de onafhankelijk gecontroleerde
echte QMP-handelingen, Saved en Presented. De barrier injecteert geen UI-
of documentinhoud en bewijst zelf geen opgeslagen/gepresenteerd werk.
De controller gebruikt echte Sys3/PIT-tijd, publiceert Stop ID1, wacht op de
exacte canonieke Stopped-reply van InitPID1, sluit zijn twee handles en zet pas
dan eigen Done456=1 vóór exit0. Refused, Failed, extra caps, verkeerde binding
of timeout leveren geen Done. De eigen BootGeneration op40 blijft behouden.

Controllerwoorden:400barrier,408request-sendOk,416canoniekeStopped-reply,
424fase,432fatale code,440starttick,448eindtick,456Done. Geen toegang tot RAM
van andere processen, keys, display, input, documenten of hostfunctionaliteit.

## Grenzen en observatie

Alle deadlines zijn vooraf vastgelegd in echte100Hz-PIT-ticks: bootstrap5000,
CMP-uitwisseling500, AppSTOP1000, ControllerReply200, testbarrier20000 en
testcontroller-RPC2000. Iedere fase bewaart de oorspronkelijke deadline;
EAGAIN vernieuwt hem niet. Achteruitgaande of overlopende tijd is fataal.
Dit is monotone tijd, geen klok voor certificaten of wandtijd. Een ronde doet
maximaal één controllerreceive, vier registryreceives en één pending send;
alle wachtlussen staan CPU-tijd af. Onverwachte ontvangen caps worden vóór
verdere verwerking eenmaal gesloten. Pixel-/key-/documentpayloads worden niet
gelogd. Native duur en deadlines moeten nog op de daadwerkelijke images slagen.

Initwoorden:400SourceReady,408PolicyReady,416CompReady,424AppStartupReady,
432echteFocusACK,440GOsendOk,448STOPsendOk,456STOP_REFUSED,
464echteRetireACK,472ControllerReplySendOk,480fase,488fatale code,
496afwijzingen,504gesloten ontvangen caps,512AppDONE,
520registryreceives,528InitReadyTick,536StartTick. Fases zijn
Boot0/Focus1/Go2/Running3/Stop4/Retire5/Reply6/Stopped7/Failed8.
Dit zijn ondersteunende observaties, geen autoriteit en geen zelfstandig
bewijs van rendering, opslag of correcte afsluiting.

Er bestaat nog geen daadwerkelijke crashnotificatie-/herstartovereenkomst voor
de kritieke GUI-diensten. Init krijgt daarom geen verzonnen status of kill-
rechten en claimt geen automatische crashrecovery. Deadline-afwijzing is geen
herstel. Volledige sessie-/shutdownpolicy, meerdere vensters, herstartende
inputepochs, toegankelijkheid en volwaardige productacceptatie blijven open.

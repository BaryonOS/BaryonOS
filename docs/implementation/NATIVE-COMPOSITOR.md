# TASK-NATIVE-COMPOSITOR — één toegewezen native surface

Deze ring3-server bezit uitsluitend de door root verleende DisplaySource,
SurfaceRECV/AppReplySEND, INP1FocusSEND/FocusReplyRECV en
SupervisorControlRECV/RegistrySEND. Geen persoonlijk volume, sleuteldienst,
netwerk, rawInput of framebufferpointer. De eigen kernel houdt device-/copy-
en capabilitymechanismen; SUR1-parse, focuscorrelatie en tekenbeleid blijven
in deze afzonderlijke userspacedienst.

## Contract en rechten

Rootbootstrap is een private1024B-startpagina op DATA. Alle velden zijn u64:
32 `NOSCOMP1` little-endian,40 BootGeneration>0,48 DisplaySource,56 SurfaceRECV,
64 AppReplySEND,72 AppPID,80 SurfaceEpoch,88 INP1FocusSEND,96 FocusReplyRECV,
104 PolicyPID,112 PolicyEpoch,120 SupervisorControlRECV,128 RegistrySEND,
136 InitPID,144 ClientID. Ongebruikte woorden zijn aanvankelijk0. Dit is een
begrensd bootmanifest, geen nieuwe algemene product-ABI. Handles/PIDs uit
payloads verlenen nooit nieuwe autoriteit. InitPID1 is de bestaande lifecycle-
supervisor; SourcePID6, CompositorPID5 en SupervisorPID1 blijven verschillend.
PolicyPID7 staat buiten die actorconfiguratie.

De compositor controleert de werkelijk ontvangen kernelafzender, tag en nul
caps vóór SUR1 of control wordt verwerkt. Onverwachte ontvangen caps worden
op alle paden eenmaal gesloten. Replybestemmingen zijn vast gebootstrapt.
Een verkeerde app of een request kan geen eigen replycap kiezen.

NativeDisplayEpoch komt uitsluitend uit de echte DisplayClient geometrycall
en blijft intern. SUR1 INFO toont publieke GOP-breedte/hoogte, SurfaceEpoch en
één toegewezen globale rect `(16,48,W-32,H-64)`. Gecontroleerd minimum is
352×304, zodat de app minimaal320×240 krijgt. Alle subtracties/conversies en
bounds worden vooraf gecontroleerd. Inputpolicy behoudt globale GOP-geometrie;
de appadapter vertaalt alleen binnen de rect en annuleert lokale pointeracties
buiten de surface. De compositor heeft geen rawmousehook.

## Echte beeldketen

SUR1 houdt maximaal één1024B-tile. Alleen de opake CommitReady uit accept kan
via commit_with met een verplichte callback worden voltooid. Productie gebruikt
uitsluitend de echte DisplayClient.blit; geen hostbackend die positief bewijs
van device-I/O levert. Een callbackfout is ook bij Ok(Reply) een negatieve
Blit-status en wordt niet als succesvolle pixels/frame geteld.

Er is maximaal één pending surfacereply. IPC-EAGAIN bewaart exact dat antwoord
en stopt nieuwe SurfaceRECV, maar niet control/focusverwerking. Iedere ronde
expireert staging vóór accept; een timeoutreply krijgt dezelfde bewaarplicht.
Commit start alleen na een verse echte monotone tickcheck; geen token wordt
over wachtrondes of willekeurige callbacks bewaard. Eerdere bevestigde tiles
zijn zichtbare writes, geen atomische pageflip/vsync of rollbackgarantie.

Vier niet-overlappende chrome-strips liggen geheel buiten de apprect. Hun
private32×8-tegels gebruiken exact Noto Sans Mono Bitmap0.3.2 en dezelfde
compile-time `ui/prototype/tokens.css` via de bestaande pure designmodule.
Geen fontparser, documentcontroller, mediadecoder of documentdata zit in de
compositorgraph. Fontpointertabellen en tekst/bitmapdata krijgen R+NX; de
linker gebruikt vijf afzonderlijke LOAD-segmenten. De entry is directe
externC_start met de bestaande C-stackafspraak.

Native geometry, alle pure runtimeconfiguratie en het volledig tekenen van
chrome plus wissen van de toegewezen apprect moeten met echte DisplayClient-
blits slagen vóór READY. Tijdens die bootstrap wordt na maximaal32tiles
CPU-tijd afgestaan. Dit verwijdert zichtbare vorige pixels in dat gebied;
het is geen algemene VRAM-wisgarantie. READY-tag `IPDREAD1`,32B: `IPD1`@0,versionu16=1@4,roleu16=3@6,
SurfaceEpochu64@8,BootGeneration@16 en echte NativeDisplayEpoch@24. Init koppelt
kernel-PID, rol, generatie en epoch. Een queued READY is geen InitACK.

## CMP1-control

Init gebruikt tag `u64::from_le_bytes(*b"CMPCTRL1")`, precies48 bytes:
`CMP1`@0,versionu16=1@4,opcodeu16@6,requestIDu64@8,SurfaceEpoch@16,
BootGeneration@24,statusu64@32,stateu64@40. Requeststatus/state zijn0.
Op1 FocusStart,2 FocusStop,3 Retire. Request-ID is niet0 en strikt monotoon.
Geen client-ID, rect, PID of replyhandle uit de controlpayload wordt gebruikt.

Controlreplies gaan via de vaste RegistrySEND naar init, met opcode|0x8000 en
dezelfde request-/epoch-/generatiebinding. Gesloten statusset:0Ok,1Malformed,
2Access,3Binding,4Replay,5Busy,6Input,7Uncertain,8Retired,9Exhausted,10Display.
Alleen bij status0 betekent state0Inactive,1Focused of2Retired; negatieve
replies hebben state0 zonder een inactiefverklaring te doen.

Init verstuurt maximaal één onbeantwoord controlverzoek. De compositor houdt
afzonderlijk één pending controlreply naast de surfacereply. EAGAIN bewaart
dezelfde bytes zonder het commando opnieuw toe te passen. FocusStart stuurt
INP1 Focus met het gebootstrapte ClientID; FocusStop en Retire sturen Focus0,
zoals de bestaande inputcore ondersteunt. PolicyEpoch verschilt van de
surface-/native-displayepoch. Een Focused/Inactive/Retired-succes volgt pas na
een echte canonieke INP1-bevestiging van het gebonden policy-PID.

Na focus-sendOk wordt alleen op de reply gewacht; nooit automatisch opnieuw
publiceren. Verkeerde afzender/binding of time-out is onzeker en beëindigt de
epoch zonder rollback- of succesclaim. Retire wist staging, verwerpt een nog
wachtende surfacereply en blokkeert verdere surfaces. Na Focus0ACK wordt de
gehele apprect met werkelijke begrensde achtergrondblits gewist. Alleen na
voltooiing kan de retirecontrol positief worden bevestigd; een blitfout is
terminaal. De bestaande200-tick-replydeadline wordt niet verruimd. Maximale
wistijd is nog niet native gemeten en blijft een integratiecriterium. Reeds eerder geldig gequeue'de
antwoorden kunnen niet uit andermans kernelqueue worden teruggehaald.

## Uitvoering en bewijsgrens

Vaste rondebudgetten: maximaal één controlreceive, één focusreply, één surface-
message en één chrometile; uitsluitend tijdens Retire maximaal32wistiles. Appflood of een volle appreplyqueue stopt de
beschermde control-/focusroutes niet. Persoonlijke pixel-/keypayloads worden
niet gelogd. Root bezit init-/kernelhooks, manifestgrants, de uiteindelijke
beeld-/QMP-capture en testimage. Deze componentpoort verlangt eerst pure
hostfoutproeven en een echte ELF-targetbuild; zij levert nog geen positieve
fakehardware- of native desktopclaim. Werkelijke init/processdeath, native
compositor-Materiaalbediening, herstel, multiwindow en toegankelijkheid blijven
afzonderlijke open releasecriteria.


## Bouw en observatie

`LOCALAPPDATA=/workspace/nieuw-os/.tools PATH=/workspace/.nieuw-os-env/bin:$PATH python3 sdk/examples/compositor/build.py` bouwt de eigen standalone Cargo-workspace offline/locked met Rust1.90.0, scalar x86_64-unknown-none en het gecontroleerde vijfsegmentenlinkerscript. Root moet de echte ABI1.8-wrappers hebben geïntegreerd; bij componentvalidatie wordt uitsluitend de goedgekeurde tijdelijke SDK-patch gebruikt en daarna teruggezet. `NOS_COMPOSITOR_ELF` verwijst naar de uitvoer; deze ELF-build is geen uitgevoerde OS-test.

Private startpagina-observatiewoorden zijn geen autoriteit:400 werkelijke chromeblits inclusief bootstrap,408 ontvangen surfaces,416 bevestigde echte appblits,424 pixels daarvan,432 afwijzingen,440 surfacereply-EAGAIN,448 gepubliceerde focusrequests,456 canonieke focusACKs,464 voltooid frame of0,472 Ready gequeued,480 focusstate,488 fatale stapcode,496 werkelijke apprect-wistegels inclusief bootstrap. Root moet deze correleren met echte proces-/brokergebeurtenissen, bron/ELF/imagehashes en de gerenderde interface.

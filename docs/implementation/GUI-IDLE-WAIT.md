# GUI-processen laten wachten bij afwezig werk

Implementatiecheckpoint, 2 november 2026. Basis: `b1b292aadf244dacf029b7307aebdc6103756e0d`; branch `codex/window-idle-wait-20261102`.

## Doel en grens

Materiaal, Rekenmachine, venstercompositor en window-init gebruiken de bestaande IPC_WAIT alleen wanneer zij aantoonbaar geen eigen werk hebben. De kernel, ABI, rechten, protocolberichten, opslag en transactietime-outs blijven gelijk. Geen nieuwe dependency, hardwaretoegang of gebruikerspointer. Een hosttest of ELF-build bewijst geen lage CPU-belasting of complete native herstelacceptatie.

De basis heeft een bekende hersteluitval: de normale volledige vensterprobe op b1b292a faalt na de compositorcrash. De lokale Windows/WHPX-run eindigde werkelijk met raw exit 35 en APP_EXIT, zonder adapter- of cleanupfout. Dat bewijs wordt niet groen gemaakt door deze optimalisatie. Voor native A/B moet eerst de afzonderlijke centrale herstelreparatie worden opgenomen en opnieuw langs de gewijzigde toestanden worden gecontroleerd.

## Contract

- Bekende ABI 1.8/1.9 behoudt het bestaande yield-pad; ABI 1.10/1.11 kan IPC_WAIT gebruiken. Voor deze display/input-apps worden ABI 1.0-1.7 expliciet geweigerd; hun werking was niet aangetoond. Onbekende ABI en ongeldige klok/wachtmasker/rechten geven geen pollingfallback.
- Een wachtplan gebruikt uitsluitend een bestaand ontvangstrecht en `now.checked_add(1)`. Dit is een afbreekgrens voor rust; het maakt of vernieuwt geen transactiedeadline. De kernel blijft rechten en queuevoorwaarden afdwingen.
- Een wake is slechts aanleiding om opnieuw te controleren. Zij consumeert geen bericht, verleent geen recht en vervangt geen echte receive/PEEK of antwoordvalidatie.
- De inputroute gebruikt pull-PEEK. Een Empty-antwoord belooft geen toekomstige pushmelding. Bij de appadapters volgt uiterlijk de volgende PIT-tick opnieuw een echte invoercontrole; de control-queue kan eerder wakker maken. De compositor behoudt de bestaande INPUT_POLL_TICKS=2: hij wordt uiterlijk de volgende tick wakker, maar doet PEEK op de bestaande tweepitscadans. Dit is geen nieuwe garantie van een input-PEEK iedere tick.
- Een onopgeslagen maar volledig getekend document mag rusten. Document-dirty is niet render-dirty. Opslaan, onzekerheid, conflict, gesloten transports, rebind, pending aanbod, onvolledig frame en uitgeputte invoerbatch verhinderen rust.
- De compositor rust uitsluitend na een werkelijk lege ingress-pass, zonder reply/inputbacklog, pending SUR1-tegel of gedeeltelijk frame en zonder resterende damage. De wrapper roept de oorspronkelijke SUR1-ontvanger aan; afzender-, epoch-, request-, lengte-, COMMIT- en blitcontroles blijven leidend.
- Window-init rust uitsluitend in Running, gereedmasker31 en bij een werkelijk lege registry, zonder herstel of shutdown. Pause/rebind/resume heeft voorrang.

## Wijzigingsgebied

`sdk/examples/materiaal/`, `sdk/examples/calculator/` en `sdk/examples/window-session/`. De typed RPC- en native IPC_WAIT-interfaces zijn ongewijzigd. Nieuwe helpers bevatten alleen rustvoorwaarden, checked deadlines en wake-maskvalidatie. Het oorspronkelijke dataverkeer en de herstelvolgorde worden niet vervangen.

## Uitgevoerde componentcontroles

De compacte brongebonden checkpointmetadata staat onder `docs/verification/gui-idle-20261102/`. Op de implementatie vóór opname van de volgende centrale herstelbron zijn daadwerkelijk geslaagd:

- Materiaal:35 hosttests, Rekenmachine:24 hosttests, window-session:24 hosttests.
- Native componentbuilds en ELF-inspectie; afzonderlijke Materiaal ELF-grenscontrole.
- Host- en native-Clippy met `-D warnings`, scoped rustfmt en diffcontrole.
- Negatieven omvatten onbekende ABI, checked overflow, stale rendergeneratie, save/uncertainty/rebind, input na Empty, volle batch, gedeeltelijke SUR1-transacties, volledig oud frame gevolgd door gedeeltelijk nieuw frame, backpressure en ongeldige wachtmaskers. Begin op tick10 houdt Commit209 toegestaan en Commit210 timeout; fragmenten vernieuwen de bestaande deadline niet.

Dit zijn componentuitkomsten. Volledige native beeld-/herstel-/opslagacceptatie en CPU-winst zijn op dit checkpoint nog niet aangetoond.

## Native A/B na herstelintegratie

1. Leg de nieuwe, herstelde basis vast; bouw baseline en idlevariant in verschillende checkouts/targets met dezelfde compiler, firmware, QEMU, backend, een vCPU en disposable media.
2. Behoud de volledige bestaande run_window_session-probe: exacte appbeelden en beschermde randen, echte invoer-ACKs, compositorfault, model/epochbehoud, save/reopen, diskoracle, cleanup en rawexit33.
3. Meet CPU pas bij exact overeenkomstige stabiele beelden en rapporteer beschikbare app-/rendercounters vóór/na. Gebruik minimaal vijftien seconden monotone hosttijd, proces- en vCPU-threadaccounting. Een gecontroleerde QMP-pauze is uitsluitend diagnostiek en geen idleoptimalisatie.
4. Controleer invoer en control/STOP na rust en herhaal met dezelfde backend. Rapporteer spreiding en hostbelasting. Doel is minstens80% lagere mediane CPU-belasting bij het geverifieerde stille beeld; zonder meting blijft dit een doel.
5. Geen nieuwe roadmap-punten voor deze performancewijziging. U01/U02 en andere productgates hebben hun eigen acceptatie.

## Herstel

De verandering staat in een afzonderlijke branch en kan als afgebakende commit worden teruggenomen. De bestaande kernel en protocolimplementaties worden niet aangepast. Geen BIOS-, driver-, Windows-feature-, beveiligings- of fysieke-apparaatwijziging is onderdeel van deze taak.

## Geïntegreerde herstelbron337f2f2

De centrale opvolger `337f2f29f1833ef4a8d2cd640b09419f85dbb83b` is afzonderlijk opgenomen. Zijn Recovery-fases en ready_bit-validatie zijn intact; recovery.rs en tests/recovery.rs zijn byte-identiek. Init weigert bovendien rust bij recovery.is_some(), ook als een recovering-vlag inconsistent zou zijn. De nieuwe echte-fasentest doorloopt pause, ready, rebind-ACK en complete. Na integratie slagen31 window-session-hosttests plus host/native/fault-Clippy, beide native componentbuilds/ELF, fmt en diff. Exacte commando's/loghashes staan in `../verification/gui-idle-20261102/recovery-integration/commands.json`. Volledige native acceptatie en CPU-winst moeten apart worden gemeten.

## Volledig native Windows-vervolg

Op dezelfde functionele freeze1f2 slagen inmiddels alle zes formele Windows/WHPX-runs (3basis/3idle), inclusief de oorspronkelijke13fasen, echte crash/herstel, save/reopen, schijforacle en cleanup/rawexit33. QEMU-procesCPU daalt mediaan88,26% bij exact dezelfde volledige schermbeelden. Dit vervangt geen lokaleTCG-, fysieke-, energie- of authenticatieacceptatie. Ruwe tellers, gecomprimeerde native/seriële originelen, hashes, meethelper en beperkingen: [Windows A/B](../verification/gui-idle-20261102/windows-ab/README.md).

## Centrale review en meetgrens

Op dezelfde fysieke pc, headlessWHPX/1vCPU/256MiB, slaagden zes volledige
native routes met ongewijzigde bronrunner/pixels/ACKs/deadline/disk/cleanup.
ProcesCPU-mediaan99,3724%→11,6663% van éénlogischeCPU in drie15s-intervallen
per arm; gemeten medianedaling88,2600%. Dit is geen energie-, geen-redraw-
of algemene prestatiegarantie. De compositor behoudt zijn2tick-polldrempel;
een reeds verschuldigde poll kan door een1tick-wacht éénrustticklater volgen.
Een harde20ms/p95-interactiegarantie en U02-acceptatie volgen hier niet uit.
Zie windows-ab/README.md en central-review.json onder het verificatiearchief.

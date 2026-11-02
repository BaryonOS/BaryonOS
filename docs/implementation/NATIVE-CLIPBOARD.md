# Native geselecteerde tekst delen — U02 klembord

Implementatiecontract, 2 november 2026. Basis41d02693083aa5262eee435c583680b566808489,
branch codex/native-clipboard-20261102. Dit sluit U02 niet: volledige gemeten
interactiebudgetten en dagelijkse formaat-/capaciteitsdekking blijven nodig.

## Gebruik en bevoegdheid

Ctrl+C kopieert uitsluitend expliciet geselecteerde tekst uit het actieve
Materiaal- of Rekenmachinevenster. Ctrl+V vervangt de selectie of voegt bij de
cursor in. Geen hele-document-, oude-uitkomst- of achtergrondleesfallback.
Ctrl+Shift+Delete wist het vluchtige klembord via de vertrouwde compositor.
De eerste begrensde implementatie ondersteunt192 UTF-8-bytes in één bericht;
te grote selecties geven zichtbare feedback, zonder afkappen. Dit is geen
verlaging van de volledige productambitie; ruimere streamoverdracht volgt apart.

De compositor is reeds de vertrouwde eigenaar van globale invoer en focus.
Hij bemiddelt CLP1 over bestaande app-INP-endpoints. InputSession blijft de
enige eigenaar van inputreplyRX; er wordt geen tweede sessie of kopie van de
handles gemaakt. Geen nieuw kernelrecht, syscall, proces, hostklembord,
bestands-/netwerktoegang of algemene clipboardread. Gewone inputrollen blijven
ongewijzigd; CLP1 bestaat uitsluitend op de expliciete venstercompositorroute.

Een aanvraag vereist werkelijke kernel-PID, huidige app-inputepoch, exacte
sequence/generation van het nog aangeboden vertrouwde CtrlC/CtrlV-Press-event
en actuele vensterfocus. Control vereist geen Alt/Super/Shift; repeat/release
zijn geen gebruikersmachtiging. Eén aanbod kan maximaal één clipboardactie
autoriseren. Apps kunnen geen machtiging maken met een zelfgeschreven keylabel.
ACK, annulering, focusverlies, queueoverflow, bronverlies, retirement en herstel
maken tickets ongeldig. Vertrouwde CtrlC wist vooraf de oude inhoud, zodat een
mislukte kopie geen stil hergebruik van oude tekst veroorzaakt.

## Gegevens en transacties

CLP1 heeft een vaste64-byte-header plus maximaal192 tekstbytes binnen de
bestaande256-byte-IPC-payload. UTF-8 en lengtes/padding/versies worden strikt
gecontroleerd; geen capabilities in berichten. De broker bewaart alleen de
gekozen tekst, zonder historie/persistentie/logging. Eigen buffers worden met
bestaande gepinde zeroize1.9.1 gewist. Geen plaintextDebug/Clone.

Een uitgestelde plakresponse bewaart alleen een ticket; de compositor
controleert dat ticket en focus opnieuw bij iedere daadwerkelijke flush.
Geen plaintext in pendingreplies nadat toestemming is vervallen. De app past
ontvangen tekst pas toe na de oorspronkelijke inputACKAcknowledged;
Superseded wist de tijdelijke tekst en verandert het document niet.
Rekenmachine valideert de complete tekst en resterende capaciteit vóór enige
mutatie; ongeldige of te lange tekst verandert expressie/cursor/selectie of de
vorige berekening niet. Plakken voert geen berekening uit.

De compositorcrash/herstart wist zijn clipboard en trekt alle tickets in;
appdocument en expressie blijven volgens het bestaande herstelcontract bestaan.
Dit is geen account-/authenticatiegrens. Andere applicaties, passwordvelden,
mediaformaten, knippen en klembordhistorie worden hiermee niet verondersteld.

## Schrijfgebieden en acceptatie

De brokerbuilder bezit uitsluitend input-client en window-session plus hun
lockfiles in een eigen worktree. Root bezit ui/native, beide appadapters,
calculator-model, native fixtures/runner en centrale documentatie/score.
De latencyonderzoeker begint read-only; geen gedeelde gelijktijdige writer.

Gerichte Rusttests gebruiken echte protocol-/Queue-/capability-engines voor
forgedPID, epoch, generation, sequence, wrongfocus, replay, afterACK,
annulering/backpressure, invalidUTF8/oversize en atomische paste. Daarna
Clippy/fmt, freestandingbuilds en ELF-bounds. Native gate behoudt alle bestaande
13fasen/pixels/ACKs/fault/save/disk/cleanup/raw33 en voegt een echte
Materiaal→Rekenmachine→Materiaal-kopieer/plakroute, leegmaken, foutscenario en
herstel zonder clipboardinhoud toe. Geen maskers of versoepelde deadlines.
Voor latency worden meetprofiel, workload, herhalingen en budgetten vóór de
metingen vastgelegd; invoerACK is geen voltooid beeld.

Bewijs blijft per bronhash/build/profiel. Geen U02-punten voor alleen hosttests,
een extra UI of nieuwe counters. Bestaande41/100 blijft tijdens implementatie.

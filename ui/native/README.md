# Native Materiaal — eerste frontendbouwsteen

`no_std`, allocatorvrij en safe Rust. Eén echte volume-DocumentSession voor
lijst/open/create/UTF-8-edits en streaming-save; geen browserbroker of nagemaakte
opslagstatus. Renderer met private 32×8-tegels, proportionele antialiased systeemtekst,
100/200%-layout, keyboard/muisfocus en een begrensde app-lokale semantiekboom
met stabiele node-ID's, ouderrelaties en expliciete actie-beschrijvingen.

Deze library bevat geen native display-/inputtransport, compositorproces,
schermlezer of volledige Materiaalomgeving. Alleen de kerneladapter mag input-
herkomst/capabilities bevestigen. Alle volledige releasecriteria blijven open.
Contract en adaptervolgorde: [NATIVE-MATERIAAL.md](../../docs/implementation/NATIVE-MATERIAAL.md).

Een native caller construeert `Controller::new` met één `VolumeSession` en drie
exclusieve callerbuffers; geen gekopieerde actieve endpointparen. `refresh`
haalt de echte lijst; `open`/`create` openen een draft. `deliver(InputEnvelope)`
verwerkt alleen door de adapter geauthenticeerde INP1-events. `save` start;
`advance_save` doet maximaal één APPEND/COMMIT; `cancel_save` vraagt echte ABORT.
`close_clean` weigert dirty/busy en probeert expliciet beide endpoints te sluiten.
`rebind_input(new_epoch)` bewaart document/caret/selectie en annuleert oude
klik-/dragstaat. De adapter mag dit uitsluitend na nieuwe geauthenticeerde
inputdienstgrants doen; het is geen capabilitygrant of bewezen hardwarereset.
Oude PEEK/ACKs blijven bij de oude endpoints en worden nooit naar de nieuwe
dienst doorgestuurd. De eerste geldige nieuwe input heft de invoerwaarschuwing
op; een onzekere opslaguitkomst blijft zichtbaar.

`Frame::new(controller.view())` leent een onveranderlijke snapshot. Itereer
`frame.tiles()`, render elke tile naar een eigen `[u32; 256]` en blit alleen de
geretourneerde tight-packed rect. Leg generatie/epoch vast, drop frame, en roep
`controller.presented(generation, epoch)` uitsluitend na bevestigde overdracht
van **alle** tiles. Fout/onderbreking/nieuwe generatie laat repaint open. De
eerste versie tekent het volledige frame opnieuw; er is geen latencyclaim of
atomische pageflip. Root moet dit op het referentieprofiel meten/integreren.

`Frame::semantics` levert een begrensde tree met rollen, stabiele ID's,
ouderrelaties, fysieke bounds, focus, read-only/enabled, tekstwaarde,
byte-selectie en echte savevoortgang. `activate` en `set_value` beschrijven
alleen welke UI-actie bij een node past; ze roepen niets aan en verlenen geen
rechten. Dit is voorbereiding voor een native toegankelijkheidsbroker, geen
IPC, werkende schermlezer of OS-brede toegankelijkheidsinterface.
Fontfallback verandert nooit opgeslagen UTF-8. Codepointnavigatie is nog geen
grapheme shaping, IME of algemene taal-/fontondersteuning.

Vanuit de repositoryroot met de bestaande toolchainomgeving:

```sh
python3 -B ui/native/verify.py
```

Dat voert de echte hostscenario's, Clippy, fmt, scalar-none-build en objectscan
uit. `verification/checks.json` bindt bestanden, de gedeelde CSS-tokenbron en
alle overige bestaande source-v3-inputs aan de gebouwde archieven. Het root-imageharnas neemt ui/native, ui/surface-protocol en tokens.css mee
in source-v3. Dit bronverband vervangt geen native uitvoeringsacceptatie.

De hostharness gebruikt de werkelijke capability-engine en volume Service op
geordende RAM-sectoren. 18 scenario's behandelen onder meer echte PS2-bytes,
commit/reopen, ABORT, onbekende commituitkomst, deny/revoke, conflict/I/O, UTF-8,
vol/capaciteit64KiB, dialogfocus, stale input/epoch en werkelijke rasterbuffers.
De grote bestaande engine/service-hoststructuren krijgen een expliciete
16-MiB-testthreadstack; dit verruimt geen native userstack. De rasterfixturetest
schrijft uitsluitend synthetische hostbeelden onder
`build/native-materiaal-visual/`; die beelden zijn door deze agent bekeken en
blijven buiten Git. Zij zijn geen QEMU-screenshots of gebruikerstest. Compacte
lijstpaginering heeft echte pointer-/keyboardacties, zichtbare focus en
semantische namen; ook op 320 logische pixels blijft materiaal zichtbaar.
Native stack/arena-
budgetten, UI-bediening en koude versleutelde opslag zijn nog niet aangetoond.

Fontbron, gekozen features en notices staan in [THIRD-PARTY.md](THIRD-PARTY.md).
Native kleuren, vectorpictogrammen en systeemtitels delen de zelfstandige
[graphics-laag](../graphics/README.md) met de compositor. Bestaande compacte
spacing-/radiustokens blijven uit [tokens.css](../prototype/tokens.css) komen.
Het prototype wordt niet als runtime gebruikt en doet geen werk namens het OS.
Documenttekst behoudt bewust zijn bestaande monospaced raster en caretmetrics;
proportionele labels wijzigen nooit de documentbytes of selectieposities.

Het gedeelde font is een begrensde, offline gegenereerde Open Sans-atlas met
echte 100/200%-rasters. De native app laadt geen TTF-bestanden of hostfonts.
De RO-segmentgrens is expliciet 512 KiB; startup-, RW- en codesegmenten blijven
afzonderlijk begrensd en W^X. `set_compositor_cursor(true)` onderdrukt uitsluitend
de lokale pointertekening; hover/focus en appinvoer behouden hun werkelijke
posities. De compositor blijft eigenaar van de vertrouwde vensterrand.

De aanvullende test `render_desktop_foundation_at_reference_scales` tekent
1280×720 op 100% en 1920×1080 op 200% via dezelfde tile-renderer. Met
`OS_FOUNDATION_FIXTURE_DIRECTORY` schrijft die test inspecteerbare PPM-beelden
naar een bestaande lege directory. Dit zijn hostfixtures, geen native bewijs.

## Sessieroute en bureaublad (2 november 2026)

Het opt-in kernelprofiel `native-system-session-preview` geeft de native app
startupwoord648=1. `Controller::enable_vm_desktop` toont gebruikerskeuze,
een herkenbaar aanmeldformulier en een expliciete openbare VM-gastsessie,
gevolgd door taakbalk, lokale programmasearch en echte Materiaalnavigatie.
Esc in de Materiaallijst keert terug naar het bureaublad; editor/dirty-dialog
behouden hun bestaande bescherming van onopgeslagen werk. Bestaande native
Materiaalgates gebruiken dit profiel niet.

`session.rs` bindt antwoorden aan request/epoch/account/methode en scheidt
authenticatie van activatie. De native standaard `UnavailableBroker` weigert
ieder echt wachtwoord; vingerafdruk/FIDO2 zijn niet gekoppeld. Dit is geen
beveiligd accountbeheer of compositor-lock: de openbare gast blijft hetzelfde
synthetische appdomein met vooraf verleende VM-rechten. Geen echte gegevens
of productiecredentials in deze preview gebruiken. Zie het
[sessiecontract](../../docs/implementation/SESSION-AND-DESKTOP.md).

Het startoppervlak heeft een duidelijke kop, een herkenbare gaststatus en één
primaire actie naar Materiaal. De kaart toont de echte bibliotheektoestand:
een nog niet geladen of lege bibliotheek, een opslagfout of een bestaand
document.
Zij verzint geen recente documenten, netwerkstatus of energiegegevens. De
compacte lay-out houdt de kaart vrij van de navigatiebalk; de brede balk noemt
de Start-actie expliciet. Dit blijft een VM-preview boven de bestaande
Materiaal-app, geen volledige desktop of geauthenticeerde gebruikerssessie.

De compacte indeling geldt ook bij minder dan640 logische pixels breedte,
niet alleen bij minder dan420 pixels hoogte. Korte kaartteksten blijven op
de minimale viewport binnen de kaart; een volle objectlijst of andere
niet-opslagfout wordt niet als opslaguitval gepresenteerd.

Aanvullende leerbron: OSDev, *Drawing In a Linear Framebuffer*, W004597,
revisie30599 (21 mei 2026). De bestaande UEFI/GOP- en capabilitytransporten
blijven leidend; geen tutorialcode is overgenomen.

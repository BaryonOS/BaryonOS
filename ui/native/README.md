# Native Materiaal — eerste frontendbouwsteen

`no_std`, allocatorvrij en safe Rust. Eén echte volume-DocumentSession voor
lijst/open/create/UTF-8-edits en streaming-save; geen browserbroker of nagemaakte
opslagstatus. Renderer met private 32×8-tegels, echte lowercase bitmapglyphs,
100/200%-layout, keyboard/muisfocus en semantische rollen/status/selectie.

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

`Frame::new(controller.view())` leent een onveranderlijke snapshot. Itereer
`frame.tiles()`, render elke tile naar een eigen `[u32; 256]` en blit alleen de
geretourneerde tight-packed rect. Leg generatie/epoch vast, drop frame, en roep
`controller.presented(generation, epoch)` uitsluitend na bevestigde overdracht
van **alle** tiles. Fout/onderbreking/nieuwe generatie laat repaint open. De
eerste versie tekent het volledige frame opnieuw; er is geen latencyclaim of
atomische pageflip. Root moet dit op het referentieprofiel meten/integreren.

`Frame::semantics` levert een begrensde tree met rollen, namen, focus,
read-only/enabled, tekstwaarde, byte-selectie en echte savevoortgang. Dit is
voorbereiding voor een native toegankelijkheidsbroker, geen werkende schermlezer.
Fontfallback verandert nooit opgeslagen UTF-8. Codepointnavigatie is nog geen
grapheme shaping, IME of algemene taal-/fontondersteuning.

Vanuit de repositoryroot met de bestaande toolchainomgeving:

```sh
python3 -B ui/native/verify.py
```

Dat voert de echte hostscenario's, Clippy, fmt, scalar-none-build en objectscan
uit. `verification/checks.json` bindt bestanden, de gedeelde CSS-tokenbron en
alle overige bestaande source-v3-inputs aan de gebouwde archieven. Het root-
imageharnas moet ui/native en tokens.css apart gaan meenemen vóór native UI-
acceptatie; de huidige algemene source_hash volgt ui/ nog niet.

De hostharness gebruikt de werkelijke capability-engine en volume Service op
geordende RAM-sectoren. 14 scenario's behandelen onder meer echte PS2-bytes,
commit/reopen, ABORT, onbekende commituitkomst, deny/revoke, conflict/I/O, UTF-8,
vol/capaciteit64KiB, dialogfocus, stale input/epoch en werkelijke rasterbuffers.
De grote bestaande engine/service-hoststructuren krijgen een expliciete
16-MiB-testthreadstack; dit verruimt geen native userstack. Native stack/arena-
budgetten, UI-bediening en koude versleutelde opslag zijn nog niet aangetoond.

Fontbron, gekozen features en notices staan in [THIRD-PARTY.md](THIRD-PARTY.md).
Eigen native kleur-/radius-/spacingwaarden lezen compile-time uitsluitend de
bestaande [tokens.css](../prototype/tokens.css). Het prototype zelf wordt niet
als runtime gebruikt en doet geen werk namens het OS.

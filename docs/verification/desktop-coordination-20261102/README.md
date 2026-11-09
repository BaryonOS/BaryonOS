# Overdracht aan leidende desktopchat — 2 november 2026

Op expliciete aanwijzing van de projecteigenaar is **Verbeter de Nieuw OS-desktop** als leidende chat aangewezen:
`01a19dcf-1da5-7192-bab2-7349f1e4a07a`, host `durable`.
Deze chat hervat productimplementatie alleen na een concrete taaktoewijzing
vanuit die chat. Alle eigen builders zijn gepauzeerd; onafgemaakt werk is
bewaard, niet geïntegreerd. Er is geen wijziging aan main of PR122.

De leidende chat is hier via `read_thread` gelezen. Een directe
`send_message_to_thread`/`start_task_turn`-functie is in deze omgeving niet
beschikbaar; deze overdracht is daarom een ophaalbaar repositoryartefact,
geen claim dat al een bericht is ontvangen of een taak is toegewezen.

## Geaccepteerde basis

PR122: `codex/mature-native-session-20261102`, broncommit
`1f0dd9c93959e3b1335656da1e548525a288133e`.
Native vensters, compositorherstel, GUI-idle en begrensd native klembord
zijn al geïmplementeerd; zie het vorige checkpoint in PROJECT-STATE.md.
De score blijft 41/100; U02-interactiebudgetten en echte persoonlijke
authenticatie blijven open. Klembordbewijs en pakket blijven bij hun
oorspronkelijke bron en mogen niet aan deze ongeteste WIP worden toegeschreven.

## Bewaarde wijzigingen, uitsluitend ter beoordeling

Alle drie de branches beginnen bij dezelfde 1f0dd9c-basis. Ze kunnen afzonderlijk
worden opgehaald. Geen is een afgeronde desktop of native acceptatie.

| Branch | Commit | Inhoud en eigenaar bij overdracht |
| --- | --- | --- |
| `codex/native-workspace-performance-20261102` | zie head van deze overdracht | Root: `ui/graphics/src/damage.rs`, module-export, Controller- en Calculator Model-API's. Conservatieve begrensde damagejournal; alle mutaties markeren voorlopig nog FULL. Geen versnellingsclaim. |
| `codex/native-workspace-v2-20261102` | `a958f33ab7e183977f9163ac611f31e8b74a50b4` | Visual: theme/text-metrics, native design/render, Calculator-render/linker en window Scene/palettetests. Geen inputmain of protocolwijziging. |
| `codex/native-retained-frames-20261102` | `fb5810e45bd8d2ab4a73d886d6c23cf526b10cec` | Performance: SUR1-protocol/client, beide app-adapters en ketentests, window-surface en legacy-surface-adapter. Opt-in behouden van ongewijzigde tegels en uniforme tegels; native mains nog niet aangesloten. |

De performancecommit vervangt alleen zijn nog niet gepubliceerde checkpoint
`a933adb`; de eerste push daarvan is wegens de Git-committerprivacy geweigerd.
De definitieve commit gebruikt de publieke Codex-identiteit. Geen gepubliceerde
geschiedenis is herschreven.

### Gedeelde interfaces en onvoltooide integratie

- `Controller::paint_damage() -> Damage`; alleen exacte huidige
  `presented(generation, display_epoch)` wist de journal.
- `Calculator Model::paint_damage(viewport)`, `presented(generation)` en
  `force_full_repaint()`. Mutaties zijn nog conservatief FULL; precieze
  tekst-/caret-/statusgebieden ontbreken.
- Retain is standaard uit. Alleen een compositor met echte private backing
  mag `Receiver::new_retained` gebruiken. RetainRun draagt de laatste volledig
  gepresenteerde basis; de server controleert volgorde, caller, epoch, grenzen,
  ACK-binding en sinds die basis gewijzigde tegels. Partial-generatiewisseling
  mag zulke tegels niet onterecht behouden. Rebind vereist eerst een volledig frame.
- Beide app-mains en de window-compositor-main moeten nog worden aangesloten.
  `Action::Retain`, opt-in, herstel en echte tellers zijn daar nog niet verwerkt.
  Tellerstanden zijn per SurfaceSession en resetten bij rebind.
- Calculator-pointerroute moet de nieuwe proportionele
  `render::expression_caret_at` gebruiken. Oude vaste celbreedte is ongeschikt.
- Scene bevat een begrensde palette voor de twee echte apps. Native inputrouting,
  focuscancel, clipboard-intrekking bij openen, modaliteit en terugkeer zijn nog
  niet gebouwd. Een getekende palette bewijst geen werkende launcher.
- De oude compositor markeert bij focus beide volledige vensters. Alleen
  appbody-retention neemt deze compositor-/displaykosten niet weg. Bij verdere
  optimalisatie moeten overlap, z-order en echte beschadigde gebieden kloppen.

### Visueel werk: bron en beelden niet verwarren

Hostbeelden van de echte Rust-renderers, **geen QEMU-screenshots**:
`/workspace/nieuw-os-workspace-v2/build/visual-preview-v2-001/` bevat
`workspace-initial.ppm`, `workspace-dirty.ppm`, `workspace-palette.ppm`,
`workspace-palette-filtered.ppm` en `host-contact-sheet.ppm`.
Deze composities dateren van vóór de laatste kleine stylingpolish en bewijzen
dus niet de finale WIP-bron. Calculatorbeelden en componentrapport staan onder
`build/visual-calculator/` van dezelfde worktree.

Open: palettequery-clipping en caret, lange documentnamen op compact formaat,
visuele review na laatste polish, echte 100/200%-pixelorakels en native gates.
De Calculator-RO-reservering is lokaal 512KiB geworden voor bestaande AA-fonts;
geen wijziging aan W^X, capabilityrechten of de bestaande procesquota. Deze
wijziging is nog een afzonderlijk te beoordelen deel van de visuele branch.

### Uitgevoerde deelchecks en bewijsgrenzen

Root-scaffold: `cargo +1.90.0 test --manifest-path ui/graphics/Cargo.toml`
slaagt met 11 tests. Calculator-hostsuite slaagt met 30 tests;
`cargo +1.90.0 test --manifest-path ui/native/Cargo.toml` met 51 tests.
Graphics/native/calculator `cargo fmt -- --check` en `git diff --check` slagen.
Logs: `/workspace/scratch/coordination-handoff-{graphics,native,calculator}.log`.

De performancebuilder rapporteert 36 SUR-hosttests met Clippy/fmt, 41
window-hosttests en 15 legacy-adaptertests groen. De nieuwe app-retentiontests
zijn nog **niet gebouwd** zonder integratie van de gedeelde API's.
De visualbuilder rapporteert Calculator 34 hosttests plus native scalar/ELF,
en eerdere native-UI 51/graphics 7/window 43 hosttests; de laatste kleine
stylingpolish is daarna niet opnieuw getest of gerenderd.
Dit zijn afzonderlijke deelresultaten, geen test van de samengevoegde bron.

Geen nieuwe VM-, WHPX-/SDL-, causal-latency-, p95-, energie- of fysieke
hardwareacceptatie. De 16KiB-compositorstack moet na de nieuwe receiverstaat
daadwerkelijk worden gecontroleerd. Geen veiligheids- of authenticatieclaim.

## Authenticatieonderzoek: geen productwijzigingen

De authagent heeft alleen gelezen; geen bestanden, dependencies, builds of VM's
toegevoegd. `SessionFlow` is interactielogica en `UnavailableBroker` weigert;
een accountdatabase en echte passwordverifier ontbreken. Bestaande HKDF is
geen password-stretching. Een mogelijke afzonderlijke vierprocessenprobe met
onderhouden PBKDF2, echte entropy, kernel-captransfer en lock/revoke is slechts
afgebakend. Niet goedgekeurd, niet gebouwd en niet aan de GUI gekoppeld.
Persisterende verifiers, cold rollback en volledige GUI-lock blijven open.
U05/B02/B06 krijgen geen punten.

## Gevraagde taakverdeling vanuit de leidende chat

De leidende chat bepaalt welke WIP zij behoudt en wijst één afgebakend werkdeel
toe met basiscommit, schrijfbestanden, interfaces en acceptatiegate. Deze chat
kan bijvoorbeeld het bestaande retained-surfacewerk afronden en native meten,
of onafhankelijke negatieve tests uitvoeren. Dit is een beschikbaar aanbod,
geen reeds verleende taak. Styling, sessieroute en integratie worden hier
niet zelfstandig voortgezet.

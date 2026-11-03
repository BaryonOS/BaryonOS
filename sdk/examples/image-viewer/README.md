# R22-IMAGE-VIEWER-LEAF-1

Appspecifieke `no_std`-controller voor **FR-A03**, rubric **A04/U03**.
FR-A04 is PDF en blijft een ander werkpakket. Basis:
`ca45a7dabe6a040921de03bdc2fad3e5045a75ba`.
TL89f heeft uitsluitend deze nieuwe subtree vrijgegeven; Apps/SDK11 is afnemer.
Geen native executable, nieuwe SDK, wire, bootstrap, installer of gedeelde UI.

## Wat de leaf werkelijk doet

Een geselecteerd object/revisie en expliciet byte-/pixelbudget starten een
loadticket. `decode` verwerkt echte PNG/JPEG-bytes met de bestaande
`services/image-codec`; `decoded` accepteert ook het resultaat van een aparte
decoder. Beide controleren de selectie en uitvoervorm voordat pixels beschikbaar
worden. Oude tickets, een andere revisie en te grote/beschadigde resultaten
worden geweigerd. Beginnen aan een nieuw object verwijdert het vorige beeld uit
de controller; fouten leveren nooit vervangende succespixels.

`Fit`, 100% en begrensde zoom van 25–800%, pijltjestoetsen en slepen werken op
de toegewezen SUR1-geometrie. 100% gebruikt de opgegeven 100/200%-UI-schaal.
Fit past beide dimensies in de surface; extreme beeldverhoudingen houden
minimaal een zichtbare pixel. De eenvoudige nearest-neighbour-weergave
selecteert echte bronsamples, maar is geen kwaliteitsclaim voor fotoschaling.
De viewer volgt JPEG-EXIF-orientatie 1–8 bij de pixelkeuze, zonder een tweede
volledig beeld te alloceren. Fit, zoom, pan en `semantics().image_size` gebruiken
de georienteerde displayafmetingen. `decoded` weigert afmetingen die niet
overeenkomen met de orientatie voordat renderen mogelijk wordt.
RGBA16 wordt alleen voor de weergave afgerond; origineel en metadata blijven
ongewijzigd. Alpha wordt tegen de door de caller gekozen 24-bit achtergrond
gemengd. Geen eigen font-, kleur-, focus- of componentensysteem.

`render_tile` schrijft uitsluitend een tight-packed, maximaal 32x8 BGRX-tegel
in callergeheugen. Extra bufferruimte blijft onaangeraakt. De adapter verstuurt
die bytes met de bestaande SUR1-client en roept `acknowledge` **pas na de echte
positieve blit/COMMIT-reply** aan. Alleen alle opeenvolgende tiles van dezelfde
generatie zetten `presented`; veranderingen trekken deze vlag meteen in.
Deze vlag betekent niet pageflip, atomiciteit, duurzaamheid of native bewijs.

`input` verwerkt de bestaande INP1-`Event`-types. De adapter blijft eigenaar
van authenticatie, epochs, volgorde en PEEK/ACK. Bij een superseded offer moet
hij de lokale pointer/focusstaat annuleren via het bestaande Cancel-event;
deze leaf is geen inputbroker. Buiten de toegewezen rechthoek, na overflow of
focusverlies vervalt een drag; terugkomen met ingedrukte knop start geen drag.
Toetsen: `+`/`-`, `0`/Home (passen), `1` (100%), pijlen en Escape (sluiten).
Een keyrelease voert geen handeling uit; Ctrl/Alt/Super-combinaties worden
niet overgenomen van de shell.

`semantics` biedt de echte toestand, Nederlandse status, beeldgrootte, zoom,
focus en framebevestiging voor de toekomstige gedeelde chrome/a11y-adapter.
Dit is **geen** geregistreerde accessibility-tree of werkende schermlezer.
Fouten/lege/loadingstatus worden hier niet als pixels getekend; de bestaande
UI-adapter moet deze tekst werkelijk weergeven. De leaf bevat geen venster.

## Invarianten en harde bewijsgrenzen

- `Selection`, `Load` en `observed` zijn correlatiegegevens, **geen autoriteit**.
  Een native openportaal/provider moet de kernel-afzender aan uitsluitend het
  geselecteerde object, de revisie en READ binden, inclusief directe kwaadaardige
  IPC/LIST/mutaties. `domain0 SEND` of `EditMode::ReadOnly` is geen alternatief.
- De native launchslot-, object-only READ-, focus-, surface- en lifecycle-seam
  is op de basisbron **niet vrijgegeven**. Root/Materiaal bezit die integratie.
  Geen bestaande Materiaal-endpoints of bootstrapvelden worden overgenomen.
- Selectie bindt byte- en decodedpixelbudget; de bestaande codec-`Limits` bindt
  daarnaast dimensies, metadata, scans/chunks en werktoelating. Er zijn geen
  extra controller-/renderallocaties of volledige RGBA-conversiebuffers.
  De bestaande parser kan infallible alloceren: een eigen begrensde arena en
  geïsoleerd proces met OOM/faultcleanup zijn **verplichte native voorwaarden**.
  `work_bytes` is geen harde heaplimiet. Een `OutOfMemory`-fout van een worker
  kunnen tonen bewijst niet dat iedere allocatorfout herstelbaar is.
- `decode` is synchroon. Een vooraf geannuleerd `Cancellation` werkt echt;
  het ticket annuleren voorkomt late publicatie. Een UI-cancel kan niet tijdens
  een geblokkeerde singlethread-decode vanzelf draaien. Responsieve native
  job-/procesannulering blijft open; er is hier geen misleidende cancelknop.
- `close`, cancel, vervangen en revoke droppen de owned pixelbuffers en
  invalideren toekomstig renderen/ACKs. Bronbytes zijn borrowed en worden niet
  gewist. Allocator-deallocatie is geen secure erase. De adapter moet echte
  jobs stoppen, endpoints sluiten en de oude surface laten wissen/retiren;
  reeds getoonde pixels of eerder verkregen kopieën worden niet teruggehaald.
- De codec en de gelijknamige semantiekvlag melden
  `metadata_transforms_applied=false` voor de **ruwe samples**. De tegelrenderer
  past EXIF-orientatie toe op de weergave; ICC/gamma-kleurbeheer en PNG eXIf
  blijven open. Geen volledige formaat-/fotogetrouwheidsclaim.
  Dependencykeuze/notices blijven in `services/image-codec`; geen nieuwe
  registrydependency, decoder of hostuitvoering toegevoegd.

## Gerichte acceptatie

Gebruik de bestaande gepinde toolchainomgeving met een eigen absolute
`CARGO_TARGET_DIR`; geen gedeelde Cargo-targetwriter. Vanuit de repositoryroot:

```sh
cargo +1.90.0 test --locked --offline --manifest-path sdk/examples/image-viewer/Cargo.toml
cargo +1.90.0 clippy --locked --offline --manifest-path sdk/examples/image-viewer/Cargo.toml --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/examples/image-viewer/Cargo.toml -- --check
cargo +1.90.0 build --locked --offline --release --lib --manifest-path sdk/examples/image-viewer/Cargo.toml --target x86_64-unknown-none
```

Tests gebruiken echte decoderfixtures plus onafhankelijke bekende
RGBA8/RGBA16/alpha-/letterboxwaarden en echte bestaande input/surfacetypes.
Vier EXIF-regressies toetsen alle acht asymmetrische pixelmatrices, echte
JPEG-decode naar alle BGRX-tegels, ongeldige displayafmetingen en geroteerde
Fit-/zoom-/panassen bij beide UI-schalen.
Foutinjecties voor dienstweigering/I/O/OOM zijn uitsluitend controllerinputs,
geen nagebootste kernel-/serviceacceptatie. Tests dekken stale replies,
budgetten, malformed decoded buffers, decodefouten, annulering, intrekking,
sluiten, schaal/zoom/pangrenzen, pointercancel en oude/onvolledige frames.
De verificatiemap bindt uitgevoerde resultaten aan bron-/artifacthashes.

**Niet uitgevoerd:** native appstart, objectportal, QEMU-bediening,
onafhankelijk framebufferorakel, process-OOM/cancel/revoke, cold storage,
volledige native/hardware/accessibility- en release-integratiegates.
FR-A03 blijft open; voorgestelde scorewijziging **34% -> 34% (+0)**, via 11
naar de manager. Terugdraaien: uitsluitend deze nieuwe subtree verwijderen;
geen gedeelde interface of runtimeprofiel is gewijzigd.

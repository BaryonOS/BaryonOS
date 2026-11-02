# Minder input-RPC's tijdens interactieve schermopbouw

2 november 2026. Basis:
`51d883e76c226b1ddd201f0e77cb126e4e5f745e`.
Gewijzigd: `sdk/examples/materiaal/src/lib.rs` en `tests/chain.rs`.
De integrator selecteert de nieuwe methode in `src/main.rs` uitsluitend met
de bestaande kernelgepubliceerde VM-previewvlag op offset 648.

## Contract en wijziging

De oorspronkelijke `App::step()` verwerkt input, één pending save-stap en
één tegel. Bij een leeg inputkanaal betekent dat één INP1-PEEK-uitwisseling
vóór iedere 32×8-tegel, ook wanneer er geen toetsenbord-/muisinput is.

De nieuwe, afzonderlijke `step_interactive()` voert dezelfde `input_step()`
en `save_step()` elk één keer uit. Daarna volgen maximaal vier bestaande
`paint_step()`-transacties, met onmiddellijke stop bij een paintfout,
surfacefalen of een voltooid frame. `input.and(paint)` behoudt de bestaande
foutvoorrang. Na inputverlies mag beperkte schermopbouw nog de fout tonen;
de draft blijft behouden en `ready` blijft onwaar. De oorspronkelijke
`step()` en zijn één-tegelcontract blijven ongewijzigd.

Alle berichten gebruiken dezelfde gevalideerde clients, endpoints en
capabilities. `paint_step()` blijft voor iedere tegel de juiste generatie,
rechthoek, pixels en COMMIT-bevestiging vereisen. Alleen een volledig
bevestigd huidig frame kan `presented_generation`/`ready` bijwerken. Een
verloren bevestiging sluit de bestaande routes en stopt de batch direct.
Er zijn geen nieuwe buffers, afhankelijkheden, threads of rechten.

Bij stabiele input omvat een volledig 768×536-appgebied 1608 tegels. Per vier
tegels is nu één lege PEEK nodig: theoretisch 402 in plaats van 1608
uitwisselingen, 1206 minder. Dit is een protocoltelling, geen claim van
viervoudige snelheid. Invoer, STOP en savevoortgang worden opnieuw verwerkt
na maximaal vier bestaande, begrensde tegeltransacties. Scheduler/polling
geven daarmee geen vaste wandkloktijd- of responstijdgarantie.

Een afzonderlijke read-only AI-review controleerde de aanpak en uiteindelijke
lib-/test-/selectiediff en vond geen concrete blocker. Dit is geen menselijke
audit. De finale native walkthrough en snelheidsmeting op de geïntegreerde
bron blijven bij de integrator.

## Uitgevoerde controles

```sh
source /workspace/.nieuw-os-env/activate.sh
CARGO_TARGET_DIR=/workspace/nieuw-os/build/perf-interactive/host-target \
  cargo +1.90.0 test --offline --locked \
  --manifest-path sdk/examples/materiaal/Cargo.toml
CARGO_TARGET_DIR=/workspace/nieuw-os/build/perf-interactive/host-target \
  cargo +1.90.0 clippy --offline --locked \
  --manifest-path sdk/examples/materiaal/Cargo.toml --all-targets -- -D warnings
CARGO_TARGET_DIR=/workspace/nieuw-os/build/perf-interactive/native-target \
  python3 -B sdk/examples/materiaal/build.py
cargo +1.90.0 fmt --manifest-path sdk/examples/materiaal/Cargo.toml -- --check
git diff --check -- sdk/examples/materiaal/src/lib.rs sdk/examples/materiaal/tests/chain.rs
```

- **27 hosttests geslaagd**: 25 ketentests, inclusief vijf nieuwe scenario's,
  en twee semantische actietests. Ze gebruiken de echte capability-, input-,
  volume- en surface-engines met expliciete RAM-/displaytegenpartijen.
- Nieuwe scenario's controleren werkelijke INP1-requestaantallen: één PEEK
  voor vier tegels, het bestaande één-tegelcontract, maximaal vier inputoffers
  per inputpass, bevestigde huidige generatie, geen repaint van een schoon
  frame, direct stoppen bij verloren ACK met behoud van draft, en
  inputverlies met begrensde foutweergave zonder valse save/ready.
- Clippy met `-D warnings`, formattering en diffcontrole: exit 0.
- Native `x86_64-unknown-none` releasebuild en afzonderlijke ELF-inspectie:
  exit 0, entry `0x40000000`, vijf geldige segmenten. App-ELF SHA-256:
  `c0d3e99dbf1899a890f74b115c7265939e62ea02fde8fbdfb54b5ab5071a7777`.
- Native geïntegreerde VM, Windows/WHPX, fysieke hardware en werkelijke
  nieuwe schermtijden: **niet door deze deelagent uitgevoerd**.

Lokale logs staan in `build/perf-interactive/logs/host-after.txt`,
`clippy.txt` en `native-build.txt`; binaries en caches blijven buiten Git.
De integrator bindt zijn finale bronhash en VM-resultaat afzonderlijk.

Rollback: laat het previewprofiel weer `step()` aanroepen en verwijder de
afzonderlijke interactieve methode. Geen disk- of protocolmigratie nodig.

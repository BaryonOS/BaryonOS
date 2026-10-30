# TASK-NATIVE-MATERIAAL — eerste native frontendketen

30 oktober 2026; basis `61ea727`. Eigenaar `native_ui_apps`: uitsluitend
`ui/native/**` en dit contract. Root bezit kernel/ABI/startupcaps/build/integratie;
`git_build_audit` onderzoekt display-/PS2-transport; geheugenagent bezit m3.rs.
Allocatorcommit `42d4bc9` blijft afzonderlijk behouden. Alle 62 volledige
releasecriteria blijven open; deze slice is geen complete Materiaalomgeving.

## Gecontroleerde basis en grens

G0 heeft echte GOP/framebufferkernelmapping en keyboardpolling, maar is een
kernelproef met uppercase debugfont, niet een gebruikersomgeving. Graphics-
bootstrap is op deze basis incompatibel met m3. `graphics::write_pixels` is al
begrensd op 80 canonieke pixels en herbruikbaar in de hardwareadapter.
`ui-core` levert echte Set1/PS2-ID0-events en focusrouting; `input-core` levert
authenticeerbare INP1-policy maar nog geen native dienstloop. Volume
DocumentSession v7 is native getest; deze frontend gebruikt die bestaande
implementatie, geen browserbroker of eigen opslagsimulatie.

De frontend bezit precies één DocumentSession en zijn endpointpaar. Geen
block-/entropy-/key-/netwerkcapabilities, geen firmware/device-I/O en geen
framebufferpointer. Een bevoegde compositoradapter bezit de displaycap en
vertaalt alleen private RAM-tegels. De UI verleent geen bevoegdheden. Een
renderer-/controller-hosttest bewijst geen native input-/displaygrens.

## Afgestemde adaptervorm (nog geen vastgesteld kernel-ABI)

- Publieke viewport: pixelbreedte, pixelhoogte en niet-nul display-epoch;
  hoogstens de bestaande 1920×1080-basis. Geen fysieke base of hardwarestride.
- Tegels maximaal 32×8, 256 little-endian u32's / 1024 bytes. Ieder pixel is
  `0x00RRGGBB`; payloadbytes B,G,R,0. Laatste rij/kolom zijn korter. De
  kerneladapter controleert het volledige rect, omvang en bovenbytes vóór de
  eerste hardwarewrite. Geen MMIO-slice naar Rust Canvas.
- Reële inputadapter valideert INP1-afzender uit kernelmetadata, tag, epoch,
  client, request-ID, canonieke velden en focusgeneratie. Pas daarna worden
  typed input-core-events aan deze frontend doorgegeven. Apps kunnen geen
  bron-/focus-/lifecyclebericht namaken. PEEK wordt pas na verwerking met exact
  het ontvangen eventnummer geACKt; een onsuccesvolle send consumeert niets.
- Focus-/bron-/queuecancel beëindigt drag/modifierinteractie, maar bewaart draft
  en selectie. Een nieuwe focusgeneratie erft geen vastgehouden actie. Een
  service- of display-epochwisseling vereist nieuwe grants/volledige repaint;
  een oud gepresenteerd frame mag nieuwe damage niet wissen.
- Een frame leest alleen eigen, onveranderlijk geleende viewdata en maakt
  private tegels. Alleen bevestiging van dezelfde framegeneratie én viewport-
  epoch kan damage aftekenen. Een gefaalde/onderbroken blit houdt damage open.

## Eerste gebruikersketen en echte documenttoestanden

Materiaal toont resultaten van de werkelijke volumelijst, opent hetzelfde
object in de tekstbewerker en bewaart via DocumentSession begin_save/save_next.
Drie callerbuffers blijven exclusief eigendom van die sessie. De eerste native
fixture gebruikt 4096 bytes; de core voegt geen permanente kleinere grens toe
aan de bestaande 64-KiB-volumegrens. Grotere callerbuffers vereisen een
bevestigde eigen arena en kunnen niet stil op de 64-KiB-userstack worden gezet.

Caret/selectie bewegen op UTF-8-codepointgrenzen; documentbytes worden niet
genormaliseerd of vervangen door fontfallbacks. De eerste fysieke layout is
US-Set1; invoermethoden/grapheme shaping/andere layouts blijven open. Alleen
werkende acties worden getekend: openen, nieuw, terug, opslaan en echte cancel.
Geen voorgeladen objectlijst of succes op basis van alleen een klik.
Dirty sluiten vraagt bewaren/verwerpen/annuleren. Tijdens streaming-save zijn
edits geblokkeerd, voortgang komt uit echte APPEND-bevestigingen, annuleren
gebruikt ABORT. Alleen Saved{revision} mag de opslagstatus bevestigen. Conflict,
volle opslag, denial en onzekere commit behouden de draft. Onzeker werk wordt
niet automatisch opnieuw verzonden. Koude versleutelde opslag blijft open tot
de afzonderlijke persistente sleutel-/B02-keten gereed is.

## Onderhoudbaar ontwerp, tekst en toegankelijkheid

Native kleurtokens/radii/spacing lezen compile-time de ene bestaande bron
`ui/prototype/tokens.css`; geen tegenstrijdige tweede kleurentabel of font-
netwerkverzoek. Typografiematen en control-/focusregels volgen PRODUCT-SPEC.
100/200% schalen veranderen geometrie en tekst samen; focus/status krijgen
semantische rollen en zichtbare labels, dus kleur is niet de enige betekenis.
Deze slice animeert niet. Dat respecteert reduced-motion, maar bewijst nog
geen systeembrede instelling of screenreaderwerking.

De expliciet goedgekeurde backend is `noto-sans-mono-bitmap = 0.3.2`, defaults
uit; regular, size16/32, Basic Latin, Latin1 Supplement en Specials. Geen
runtime-dependencies, buildscript, allocator of floats. Rust-MSRV 1.56.1;
gepubliceerd 7 januari 2026. De daadwerkelijke tarballchecksum is
`f102c477fd4ec864c149803bf606ecc23bc9329602ec0f99073a728abf454abc`.
Library MIT; de oorspronkelijke Noto Sans Mono Regular 2.014 is OFL1.1,
copyright 2022 Noto Project Authors. Font-SHA:
`56134342e3d5137fbfcb25361039d4a98def7df75db7dd821ef47c3ae03127b6`,
herleidbaar tot upstreamgit `636fb222507237549c20e9b23dae54fc9d639ae3`.
Beide notices blijven onder ui/native/licenses behouden. Dit is een begrensde
eerste fontbackend, geen definitieve huisstijl of volledige Unicode-engine.
Ontbrekende glyphs zijn herkenbare vervangingstekens; opgeslagen UTF-8 blijft
intact. Het oude uppercase G0-font wordt niet voor gebruikersdocumenten benut.

Primaire bronnen: [crate](https://crates.io/crates/noto-sans-mono-bitmap/0.3.2),
[upstream](https://github.com/phip1611/noto-sans-mono-bitmap-rs),
[Noto-OFL](https://github.com/notofonts/latin-greek-cyrillic/blob/main/OFL.txt),
[bestaande productafspraken](PRODUCT-SPEC.md), [invoercontract](NATIVE-INPUT-SERVICE.md)
en [GOP-basis](GRAPHICS-G0.md). Broncontrole op 30 oktober 2026 is geen audit.

## Bewijsplan en open vervolg

Gerichte hosttests: UTF-8/range/selection, echte service + capability-engine
achter DocumentSession, save/ABORT/conflict/onzekere commit/I/O-fout, geen valse
Saved, guards/clipping/kleurvolgorde van werkelijke rendertegels, focus en
100/200%-layout. Daarna Clippy/fmt en daadwerkelijke scalar none-build.
Root integreert echte ring3processen/caps, QMP-toets/muis, gerenderde voor/na-
beelden én bedieningsassertions, diskoracle/coldreopen en exact imagebewijs.
Schermlezer, alle zeven routes, compositorherstel, volledige editorfuncties,
cross-appklembord, persistente keys en de rest van de release blijven open.

## Eerste uitgevoerd checkpoint

`python3 -B ui/native/verify.py` is uitgevoerd met Rust1.90.0: 14 hostscenario's,
Clippy, fmt, featuregraph en daadwerkelijke scalar `x86_64-unknown-none`-build
slagen. De negen gebouwde rlibs zijn via echte Cargo-artifactrecords gebonden
en met objdump/nm gecontroleerd; geen SIMD/FPU-instructies of hostmallocbackend
in die objecten. Generieke controllermethoden worden pas in de definitieve
native caller volledig gemonomorfiseerd; dit is geen ring3-uitvoering.
Exacte bron-/input-/artifactbinding en commando's:
[`ui/native/verification/checks.json`](../../ui/native/verification/checks.json).

De grote bestaande capability/volume-hostharness overschreed eerst de kleine
standaard testthreadstack. Dezelfde expliciete 16-MiB-hostteststack als het SDK-
harnas verhelpt dat; de native 64-KiB-stack wordt niet verruimd of afgetekend.
Een eerste compile vereiste type-/APIcorrecties in de nieuwe tests. Op het
bovenstaande checkpoint zijn de gerichte controles werkelijk opnieuw geslaagd.

Native integratie vraagt een eigen app-linker: de fontpointertabellen staan in
`.data.rel.ro` en moeten R+NX blijven. De oude systemapp-linker zet `.data.*`
in één RW-pagina en laat maar48KiB rodata toe; dat past niet. Gemeten vóór
definitieve link: circa146KiB relro +35KiB rodata +27KiB frontendcode. Een
begrensd eigen readonlysegment van bijvoorbeeld256KiB past binnen het reeds
toegestane userimagewindow en vraagt geen kernel-/capabilityverruiming. Definitieve
residentie/stack/latency blijven te meten op de echte gekoppelde UIapp.

De algemene `tools/common.py::source_hash` volgt ui/ nog niet. Eigen bewijs
bindt ui/native en ui/prototype/tokens.css daarom aanvullend. Root moet de
native image-/capture-inputbinding uitbreiden vóór UI-acceptatie; anders kan
oude UIcode onder een ongewijzigde algemene bronhash onterecht actueel lijken.

# FR-U01 — begrensde native invoerbouwstenen

Taakcontract, 29 oktober 2026. Eigenaar: `native_ui_apps`; integratie: hoofdagent.
Schrijfgebied: uitsluitend `crates/ui-core/**` en dit document. Geen nieuwe
dependency, syscall, kernelrecht of grafische productsuccessclaim.

## Resultaat en grens

De `no_std`-core decodeert een vertaalde PS/2 Set-1-toetsenstroom en standaard
driebyte-PS/2-muispackets naar fysieke events. Een aparte router bewaart de
focusgeneratie en de eigenaar van ingedrukte toetsen/knoppen. De bestaande
`Keyboard`, `KeyEvent`, `Modifiers` en ASCII-`TextLine` blijven beschikbaar voor
G0. Er komen geen apparaat-I/O, gebruikerspointers, tekstlayout of opslag in de
core. Dit is geen compositor, invoerdienst of voltooide U01/U02-acceptatie.

Profiel: QEMU q35, i8042, naar Set 1 vertaalde toetsen en PS/2-device-ID 0;
geen wiel-/vijfknopsmodus, USB, IME of fysieke hardwareclaim. De adapter moet
het profiel werkelijk configureren, controllerstatus controleren, AUX en
keyboard correct scheiden en commandoantwoorden buiten de eventstroom houden.

## Interfaces en invarianten

- `Set1Keyboard::feed`: fysieke `KeyboardEvent`s met Press/Repeat/Release en
  links/rechts onderscheid voor modifiers; tekstvertaling blijft afzonderlijk.
  Onbekende codes en gebroken prefixes geven een expliciete fout. De adapter
  reset decoder én router bij verloren/controllerfoute invoer en meldt inputreset
  aan de betrokken client; geen stil vastzittende modifier.
- `Ps2Mouse::feed`: precies drie bytes per packet; bit 3 markeert de eerste
  byte. Payloadbytes mogen elk bitpatroon hebben. Bewegingswaarden zijn signed
  negen bits; een overflowas beweegt niet, knopovergangen blijven beschikbaar.
  Na bekend byteverlies/timeout reset de adapter de packetgrens. De bitmarker
  alleen bewijst geen herstel na willekeurig ongemerkt byteverlies.
- `Cursor`: getypeerde schermgrenzen, clamping en omzetting van PS/2-Y naar
  scherm-Y; geen pointerversnelling of schaalpolicy.
- `InputRouter`: alleen de vertrouwde compositor verandert focus. Events dragen
  eigenaar en monotone generatie; aflevering controleert die opnieuw. Oude
  repeats/releases en al ingedrukte modifiers/knoppen worden niet aan een nieuwe
  focus overgedragen. Generatie-overflow sluit de route en hergebruikt geen token.
  Ook inputreset bewaart bekende oude down-eigenaars als onderdrukkingsmarkers
  tot een release is waargenomen; een reset mag geen nieuwe klik of sneltoets
  verzinnen. Als een release zelf verloren ging kan de eerste volledige nieuwe
  toetscyclus worden onderdrukt. De adapter meldt de reset begrijpelijk.
  Routertokens zijn geen kernelcapabilities: bron-, proces- en surfaceautoriteit
  moeten bij de uiteindelijke kernel-/dienstgrens worden gecontroleerd.

## Acceptatie van deze bouwsteen

Hosttests: vaste packet-/scancodevectoren, signed extremen, overflow, gesplitste
packets, foute headers/prefixes, pauze/PrintScreen, modifierrelease en repeat,
cursorclamping, focuswisseling met vastgehouden toetsen/knoppen, oude queue-events
en generatie-uitputting. Bestaande G0-tests blijven slagen. Compileer de library
met de vastgelegde Rust 1.90.0 naar `x86_64-unknown-none`; voer gerichte Clippy en
formatcontrole uit in een eigen targetmap.

Native vervolgacceptatie blijft open: ring-3-display-/inputdienst, kernel-
capabilitygrens, echte QMP-muis/toetsen, twee surface-eigenaren, focus en
compositorherstel op het uiteindelijke image. Hosttests of librarycompilatie
bewijzen deze scenario's niet.

## Uitgevoerde controle — 29 oktober 2026

Op basiscommit `73ebc1e`, met de nieuwe bronhashes uit
[`input-acceptance.json`](../../crates/ui-core/verification/input-acceptance.json),
zijn daadwerkelijk uitgevoerd (allemaal exit 0):

```text
cargo +1.90.0 test -p ui-core --locked --offline
cargo +1.90.0 clippy -p ui-core --all-targets --locked --offline -- -D warnings
cargo +1.90.0 build -p ui-core --lib --release --target x86_64-unknown-none --locked --offline
cargo +1.90.0 fmt -p ui-core --check
git diff --check
```

Resultaat: **25 hosttests geslaagd** (15 nieuwe invoertests en 10 bestaande
G0-coretests), native librarycompilatie, Clippy en formattering geslaagd.
Toolselectie via bestaande `tools/common.py::environment()`, met
`LOCALAPPDATA=/workspace/nieuw-os/.tools` en eigen
`CARGO_TARGET_DIR=/workspace/nieuw-os-worktrees/ui/build/input-target`.
Volledige lokale stdout/stderr: `build/input-acceptance/`; het compacte rapport
bevat commando's, exitstatussen, compiler, bron-/loghashes en de targetartifacthash.
Geen QEMU-/native invoerproef of menselijke bediening uitgevoerd voor deze slice.

## Bronnen

- OSDev Wiki, *PS/2 Mouse*, W003157, revisie 30841 van 2 juli 2026:
  standaardpacket, sign-/overflowbits en device-ID. Lokale tekst gecontroleerd.
- OSDev Wiki, *PS/2 Keyboard*, W001900, revisie 30840 van 2 juli 2026:
  Set-1 make/break, prefixes, commando-/antwoordgrens. Lokale tekst gecontroleerd.
- QEMU Project, `hw/input/ps2.c`, tag `v10.0.2`, functies
  `ps2_mouse_send_packet`, `ps2_mouse_event` en `ps2_keyboard_event`:
  https://github.com/qemu/qemu/blob/v10.0.2/hw/input/ps2.c.
  Primaire bron voor het emulatorprofiel, gelezen 29 oktober 2026; geen code
  overgenomen. QEMU begrenst standaardmuisdeltas daar op ±127; de core decodeert
  het bredere standaard-negenbitsformaat eveneens. Geen claim dat een complete
  IBM-hardwarehandleiding lokaal is gecontroleerd.

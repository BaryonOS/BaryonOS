# Native image-fixture: observercontract

IMAGE-NATIVE-OBS-1 op kernelbasis
`15e8cfced28ad46f15f5f573e5f00e8e87b74215`. Deze opt-in-waarnemer hoort bij
codec/probecheckpoint `9ae1c738012fc73085a04d6cfca7cd7fbc3bde46`; hij voert geen
PNG/JPEG-parser uit in de kernel. Zie ook de componentbeschrijving
`docs/implementation/NATIVE-IMAGES.md` bij integratie van die bronreeks.
FR-A03, object-RPC, zoom/pan, compositor en algemene formaatacceptatie blijven open.

## Eigenaarschap en hooks

Deze taak levert uitsluitend `kernel/src/m2/m3/image_fixture.rs` en dit contract.
De integrator bezit Cargo, m3/system, ELF-selectie, bootstrap/capabilities,
process-death, builds en VM-harnas. Voorgesteld profiel:
`native-system-images=[native-system,process-memory,fpu-context]`, ABI1.7.
De bestaande vier diensten blijven actief; de probe start alleen bij boot in
slot 4. Hij krijgt geen raw netwerk-, sleutel-, entropy-, opslag- of displaycap.
De app gebruikt ABI-query/yield/exit, monotone ticks en eigen private heapgroei.

Plaats de module en alle hooks uitsluitend onder de nieuwe profielvoorwaarde;
oude profielen laden deze waarnemer niet. De API is:

```rust
bind(s: &State, actual_pid: u64, linked_elf: &[u8]);
is_probe(s: &State, slot: usize) -> bool;
observe_exit(s: &State, slot: usize); // vóór State.cleanup/State.free
fault(s: &State, slot: usize, context: &[u64; 22]) -> !;
finished() -> bool;
check_and_print(s: &State);
```

`bind` volgt het laden/registreren van de echte probe en het vullen van
startupwoord 176 met diens werkelijke PID. Geef exact dezelfde gelinkte
`IMAGE_PROBE_ELF`-slice door die de loader gebruikt. Bind bewaart PID,
bootslot, echte starttick, oorspronkelijke mappingtelling en ELF-fingerprint.
De startpagina en oorspronkelijke 16 heappagina's moeten al aanwezig zijn;
de probe mag nog niet hebben uitgevoerd. Herbinding is een fout.

`is_probe` vereist de opgeslagen PID **en** bootslot plus levende mappings.
Een worker die later slot 4 hergebruikt wordt daardoor niet de probe. Exit/fault
vergelijken ook de echte `last_event.pid`; een slotnummer is geen identiteit.
`observe_exit` leest uitsluitend een gevalideerde, door dit PID beheerde
startpagina en legt alle woorden vast voordat geheugen wordt vrijgegeven.
Daarna gebruikt `check_and_print` alleen de kernelsnapshot, nooit een pointer
naar opgeruimd gebruikersgeheugen. IF=0/single-CPU-callbacks zijn vereist; geen
observerborrow mag een schedulingpunt, IPC of gebruikersuitvoering overspannen.

De helper roept **geen** `Engine.process_died` aan. De root houdt de generieke
Engine-/procesafhandeling en de bestaande uitzonderingsproeven intact. De
`fault`-hook retourneert niet: hij meldt echte CPU-metadata en beëindigt alleen
de wegwerp-VM-proef als mislukt. Eventuele Engine-bookkeeping die vóór die
afsluiting nodig is blijft dus bij de root. Dit is geen nieuw productfaultbeleid.
Roep `finished` op naast de bestaande joint-gates en `check_and_print` eenmaal
vóór het uiteindelijke SYSTEM-succes. Een ontbrekende, dubbele of ongeldige
snapshot mag nooit als een voltooid image-onderdeel tellen.

## Vereiste gegevens en onafhankelijke kernelwaarneming

De kern leest 15 vaste u64-woorden binnen de private startpagina:

| Offset | Betekenis / eis |
|---|---|
|40,240|done=1,errorstep=0; werkelijk exitcode0 en syscallvector128 |
|400,408|PNG 8, JPEG 3; inclusief PNG-herdecode na geweigerde invoer |
|416,424|149647 pixels, 10 negatieve uitkomsten |
|432,440|pixel-FNV64=548683896226622996 (`079d5118b890ae14`),623808samplebytes |
|448|1024 door de app opgegeven groeipagina's |
|456,464|werkelijke app-arena-highwater > 0 en≤4194304; liveallocaties 0 |
|472|1 vooraf geannuleerde decode; geen mid-decode-latentieclaim |
|480,488|gaststart/eindtick binnen echte kernellaunch-/exittick; duur<20000ticks |
|496|QUALITY_OPEN=0 voor de herstelde specifieke progressieve JPEG-orakelvraag |

De heapclaim wordt daarnaast met de kernel getoetst: 1040 totale heappagina's
(16 bestaand + 1024 nieuw), precies 1024 extra gebruikersmappings tegenover bind,
eigenaar PID / Kind::Data, user/RW/NX en mappingtelling binnen de ongewijzigde 2048-
paginaquota. De userstack heeft 16 RW/NX/Kind::Stack-pagina's, met onbeschikbare
onder-/bovengrens; ook het uiteinde van de heapsuffix blijft ongemapt. Geen
heap, stack of quota wordt voor deze proef vergroot.

De arena-accounting en pixelchecksums zijn appmetingen van de gehashte,
gecontroleerde probe, geen algemene attestation van een willekeurige kwaadwillige
app. De kernel parseert de allocator of afbeeldingen niet opnieuw. Hij controleert
de eigen geheugenfeiten, exit, identiteit en echte ticks. De 10 negatieve decode-
uitkomsten vervangen geen echte native allocator-OOM-/herstartproef: die blijft
open tot zij afzonderlijk werkelijk wordt uitgevoerd. Een succesvolle normale
gastuitvoering bewijst dat deze inputset met de echte 64 KiB-stack heeft gewerkt,
niet een maximale-stackgarantie voor alle mogelijke bestanden.

## ELF- en artifactbinding

`bind` berekent FNV-1a64 en lengte over de **werkelijke gelinkte ELF-bytes**;
deze verschijnen als `SYSTEM:IMAGE_ELF_FNV64` en `SYSTEM:IMAGE_ELF_BYTES`.
FNV is uitsluitend een drift-/parityfingerprint, geen cryptografische
authenticiteits- of bootclaim. De root moet daarnaast SHA256 van precies dit
ELF vergelijken met het buildmanifest, include-input en vrijgegeven image.
Voor codeccheckpoint9ae1c73 is het daadwerkelijk gebouwde input-ELF:

`ac2a0ae22021ae6dda7a37d981b5a049209ee71b669a4644c2fc5eaeec9fbff2`.

Na relevante bron-/configuratieverandering moet dat opnieuw worden gebouwd en
gebonden; een oud hashveld of oude succesvolle test is geen bewijs. Broncommit,
profiel, QEMU-configuratie, loader/kernel/ELF/imagehashes en seriële uitvoer
horen bij de finale rootgate. De app bevat synthetische PNG/JPEG-fixturebytes;
er is geen hostdecoder of persoonlijke foto als OS-implementatie aanwezig.

Succesmarkers worden pas na geldige snapshot en gezamenlijke afronding gedrukt:
`SYSTEM:IMAGE_*` voor PID/ELF/pixels/geheugen/ticks/guard en tenslotte
`SYSTEM:IMAGES:PASS`. Iedere `SYSTEM:IMAGE_FAIL:*`, fault, timeout, ontbrekende
marker of onverwachte actor maakt de gate rood. De rootrunner moet alle vereiste
waarden strikt toetsen en ruwe uitvoer behouden.

## Uitgevoerde controles van deze helper

Dezelfde pure `Observer`-predicates draaien in de kernel en in 9 uitgevoerde
hostmodelproeven: geldige snapshot, verkeerde/stale PID/slot, workerreuse,
corruptie van elk exact te toetsen appwoord, heap-/owner-/guardtegenspraak,
klokgrenzen, niet-nul exit, foutvector, voortijdig aftekenen, herbinding,
dubbele exit/print en blijvende weigering na een fout. Commando:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools python3 -B tools/dev.py rustc +1.90.0 \
  --edition=2021 --test kernel/src/m2/m3/image_fixture.rs \
  -o /tmp/nieuw-os-image-fixture-model-tests
/tmp/nieuw-os-image-fixture-model-tests --nocapture
```

Uitkomst: 9/9, exit 0. Dit zijn modellen met expliciet aangeboden kernel-feiten,
geen uitgevoerde CPL3-proeven of bewijs dat die feiten op hardware bestaan.

De native adapters zijn daarnaast op een private kopie van de echte 15e8cfc-
kernel/State met Rust1.90/x86_64-unknown-none en
`native-system,process-memory,fpu-context` getypecheckt, exit0. Alleen daar is
een module-inclusion toegevoegd; de gedeelde bron is niet gewijzigd. Voor
`include_bytes` zijn onuitgevoerde ELF-inputs gebruikt: dit was **cargo check**,
geen gelinkt of opgestart joint-profiel. Het exacte typecheckrapport staat op
`/tmp/nieuw-os-image-fixture-binding-check.json`. Rootintegratie, juiste rol-ELFs,
volledige build en echte QEMU-uitvoering blijven vereist en zijn nog niet
door deze helpercontrole afgetekend.

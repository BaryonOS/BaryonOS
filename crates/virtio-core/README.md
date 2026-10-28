# virtio-core — zuiver M5-beleid

Safe Rust `no_std`, nul dependencies en `forbid(unsafe_code)`. Deze crate krijgt
alleen integers en vaste arrays: geen raw pointers, MMIO, PCI-I/O, klok of heap.
Het profiel is modern virtio-blk, split queue8, één outstanding request,
512-byte READ en een gebruikersslice van 1–128 bytes. De M5-runtime biedt geen
WRITE of IOMMU-integratie. De aanvullende M6a-planhelpers hieronder publiceren
zelf geen I/O en verruimen geen capability of syscall.

## API

- `parse_capabilities(&[u8;256], &[Bar;6]) -> Regions`: PCI-snapshot met
  vendor/device `1af4:1042`, header0, capabilityketen met cycle-/lengtecontrole,
  expliciete memory-BARs en niet-overlappende common/notify/ISR/device-regio's.
  De strikte QEMU-slice vereist 8-byte common/device-offsetalignment. `Bar`
  bevat `base/size/is_memory`; de door de kernel geprobed grootte moet bij de
  config-BAR passen. `Regions::notify_address` controleert multiplier en bereik.
  Adressen boven de x86-64-PTE-limiet52 worden geweigerd; de kernel controleert
  daarnaast de werkelijk ondersteunde CPU-breedte en overlap met RAM/reserveringen.
- `negotiate_features`: VERSION_1 en readonly vereist, alleen die twee bits
  geaccepteerd. ACCESS_PLATFORM wordt geweigerd: dit profiel configureert geen
  IOMMU. Overige versnellings-/queuefeatures worden niet onderhandeld.
- `descriptor_plan(DmaPool, ram_end)` en `validate_descriptors`: vijf unieke,
  page-aligned kernelpagina's; exact header16/RO → bounce512/device-WRITE →
  status1/device-WRITE. Geen indirecte descriptor, variabele chain of useradres.
  `Descriptor::encode` en `read_header` produceren little-endian bytes.
- `validate_used(head, old_idx, new_idx, id, len, status)`: geen nieuwe entry
  geeft Pending; één entry met correcte head is vereist. Een geslaagde READ
  vereist len513. IO/unsupported met len1–513 levert een terminale foutuitkomst,
  nooit toestemming om bouncegegevens als succes te leveren. Onbekende status,
  indexsprong, verkeerde id en ongeldige lengte geven Protocol. De normale
  16-bit ringindexwrap wordt wel ondersteund.
- `Policy`: const initializer, `submit(Binding, Read, capacity, now, timeout)`,
  `complete`, `peek`, `consume`, `start_reset`, `reset_completed`, `timeout`.
  Binding bevat PID en getypeerde objectkey/generation. Elke echte poll moet
  eerst opnieuw een actuele READ-cap naar hetzelfde object autoriseren.

## Lifetime en bewijsgrens

`Policy::new` begint Offline. Alleen een door de transportlaag bevestigde reset
opent Ready met een nieuwe epoch. Submit valideert bounds en checked deadlines
vóór publicatie van een ticket. Peek consumeert niet; de kernel valideert de
volledige useroutput vóór consume en de daaropvolgende onfeilbare copy.

Timeout/cancel gaat naar Resetting en houdt de pending metadata vast. Zonder
acknowledgement wordt de policy permanent Quarantined; late `true` heft dat niet
op. Epoch en ticket wrappen nooit. De kernel houdt alle DMA-pagina's bootlang
gepind en mag exposed buffers pas na completion of geverifieerde reset wijzigen.

**Een virtio used entry bevat geen epoch of nonce.** De parameters van
`Policy::complete(epoch,ticket,...)` komen uit kernelmetadata. De hosttest voor
een oude epoch bewijst alleen softwarebinding; hij detecteert geen kwaadwillige
device-replay met dezelfde opnieuw gebruikte head-id. De echte status0-reset,
barrières en queuequiescence moeten in de transportdriver worden bewezen.

De pure poolcontrole bewijst geen echte frame-eigenaar, pinning, cachetype of
DMA-containment. De kernel moet die eigenschappen aan daadwerkelijke
Frames/pagetables/PCI-registers koppelen. Zonder VT-d blijft het QEMU-device
vertrouwd; er is geen hardware-IOMMU-claim of ondersteuning voor echte apparaten.

## Controles

12 hosttests dekken PCI-capability/bounds/cycles/overflow en 1.536 bytevarianten,
descriptoradressen/flags/chain/lengtes, completionindex/id/lengte/status, slice-
bounds, PID/objectbinding, timeout/quarantaine en epoch/ticket-no-wrap.
Maximale tellerstanden worden alleen in de `cfg(test)`-module gezet.

De aanvullende capability-core-tests bewaren alle21 M4-tests en voegen vijf
gerichte BlockDevice-tests toe: scope/type/invalidmask, adminclose, transfer/
revocation/exit, aparte quota, generationretirement en gemengde queuecancellation.

Exacte commando's vanuit de projectroot met Rust1.90.0 en eigen
`CARGO_TARGET_DIR=build/m5-core-tests`:

```text
cargo +1.90.0 fmt -p capability-core -p virtio-core -- --check
cargo +1.90.0 test -p capability-core -p virtio-core --locked --offline -- --test-threads=3
cargo +1.90.0 clippy -p capability-core -p virtio-core --all-targets --locked --offline -- -D warnings
cargo +1.90.0 check -p capability-core -p virtio-core --target x86_64-unknown-none --locked --offline
git diff --check -- crates/capability-core crates/virtio-core
```

Uitgevoerde resultaten, exacte argv en bronhashes worden lokaal bewaard in
`verification/`; root kopieert de compacte bewijsversie naar docs/verification.
Een hosttest is geen echte PCI-/DMA-/reset- of ring3-uitvoering. De kernelagent
beheert die afzonderlijke grens. Terugdraaien vereist gezamenlijk verwijderen
van deze crate en de door root beheerde workspace-/driverdependency.

## Eerste M6a-stap: zuivere WRITE/FLUSH-plannen

`writeback::Features::negotiate` vereist VERSION_1 en FLUSH (bit9), weigert
READ_ONLY en ACCESS_PLATFORM en neemt alleen de twee vereiste bits over. Dit
valideert een featurevoorstel; het bewijst geen `FEATURES_OK`-acceptatie door een
device. De bestaande M5-featurefunctie en `Policy` blijven ongewijzigd.

`writeback::Plan::new` krijgt dit gevalideerde featuretype, een `Operation`, de
vaste vijfpagina-pool, RAM-einde en devicecapaciteit in sectoren. `Write { sector }`
vereist `sector < capacity` en bouwt exact header16/RO → bounce512/RO →
status1/device-WRITE. FLUSH bouwt header16/RO → status1/device-WRITE, met opcode4,
sector0 en geen data-descriptor. De WRITE-opcode is1. Alle headerbytes en de
volledige actieve descriptorketen worden vóór publicatie vergeleken door
`Plan::validate`; een extra of ontbrekende descriptor faalt ook.

Head is altijd0. `Plan::validate_used` verlangt één nieuwe used-entry, id0,
lengte1 en een bekende status; alleen de statusbyte is device-writable. De
normale 16-bit indexwrap blijft toegestaan. IOERR en UNSUPP zijn terminale
foutuitkomsten en rechtvaardigen geen payload of aanname dat de opslag
ongewijzigd is. De zeven aanvullende hosttests controleren features,
bytevolgorde, beide ketens, capaciteit-/poolgrenzen, header-/descriptormutaties
en completionmetadata. Deze zijn zuivere hostcontroles, geen echte WRITE/FLUSH.

De toekomstige transportcaller blijft verantwoordelijk voor autorisatie,
featureacceptatie, het volledig initialiseren en kopiëren van precies512
payloadbytes vóór publicatie, barrières, deadlines en de volledige DMA-lifetime.
Een writecompletion bewijst geen duurzaamheid. Een fout, timeout of reset kan
een al gedeeltelijk uitgevoerde write niet terugdraaien; crashherstel blijft een
afzonderlijke M6a-stap. Er is nog geen schrijvende kernel-/dienstinterface.

Primaire bron: OASIS Open, *VIRTIO1.2*, Committee Specification01 (1 juli2022),
§§2.7 en5.2; tag `v1.2-cs01`, commit
`e230375adde48ff39f734ea2783ef7b9d2ba346a`,
[vast bronbestand](https://raw.githubusercontent.com/oasis-tcs/virtio-spec/e230375adde48ff39f734ea2783ef7b9d2ba346a/content.tex),
gecontroleerd op28 oktober2026. SHA256 van het bronbestand:
`08554df46753374735954a8436f8786a8cc5bd0b29f9f260f434e8c05061d7fd`.

Bron voor de geïnterpreteerde velden/protocolgrens:
[OASIS VIRTIO1.2](https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html),
met name §§2.4,2.7,3.3,4.1 en5.2. Geen externe implementatiecode overgenomen.

# M6a — begrensde WRITE/FLUSH, stap voor stap

28 oktober 2026. Basis `5a58a74`, werkbranch `codex/m6a-write-flush-core`.
Dit is het contract voor de eerste implementatiestap; uitgevoerd bewijs staat
in PROJECT-STATE.md en verification/m6a/. M6a als geheel is nog niet afgerond.

## Eerste stap: zuivere requestplannen

Voeg uitsluitend `virtio_core::writeback` toe: safe Rust, `no_std`, geen nieuwe
dependencies, heap, pointers, MMIO of I/O. De bestaande M5-functies, `Policy`,
capabilityrechten, syscalls en alleen-lezen driver blijven ongewijzigd. De
nieuwe API wordt in deze stap nog niet door de kernel/driver aangeroepen.

### Beveiligingsinvarianten

- Een afzonderlijk schrijfprofiel vereist VERSION_1 en FLUSH (bit 9), weigert
  READ_ONLY en ACCESS_PLATFORM, en selecteert geen onbekende featurebits.
  Dit softwareprofiel vervangt geen echte FEATURES_OK-readback van het device.
- WRITE beschrijft precies één volledige sector van 512 bytes binnen de
  opgegeven devicecapaciteit. Geen gedeeltelijke sector of read-modify-write.
  Header: type 1, reserved 0, gecontroleerde sector in little-endian.
- De WRITE-keten heeft drie descriptors: header16/device-readable →
  bounce512/device-readable → status1/device-writable. Geen useradres,
  indirecte keten of door de caller gekozen DMA-adres buiten de gevalideerde pool.
- FLUSH heeft type 4, reserved 0, sector 0 en twee descriptors:
  header16/device-readable → status1/device-writable. Geen bounce/data-descriptor.
- De vijf bestaande poolpagina's blijven uniek, page-aligned en binnen
  `ram_end` en de bestaande fysieke adreslimiet. Een plan controleert alleen
  integers; werkelijk eigenaarschap, pinning en CPU-adresbreedte blijven callerwerk.
- Elk plan bindt zijn operatie, header en exacte descriptorketen. Afwijkende
  descriptorvelden of een extra descriptor worden vóór publicatie geweigerd.
- Zonder nieuwe used-entry: Pending, ongeacht nog oude status/metadata.
  Verder: precies één nieuwe entry, juiste head/id en used-lengte exact 1.
  Status OK geeft Success; IOERR/UNSUPP zijn terminale fouten zonder payload.
  Verkeerde index, id, lengte of status geeft Protocol. Ringindexwrap mag.

### Interface

`Features::negotiate(offered)` maakt een niet vrij construeerbaar profiel;
`accepted_bits()` geeft het te onderhandelen masker. `Operation` is
`Write { sector }` of `Flush`. `Plan::new(features, operation, pool, ram_end,
capacity)` maakt een immutable plan, met accessors voor operatie, header en de
actieve descriptorslice. `Plan::validate(header, descriptors)` bindt ook de
headerbytes aan de draft. `Plan::validate_used(old_idx, new_idx, id, len, status)`
controleert de completion voor vaste head 0.

Geen nieuwe capability- of syscallinterface in deze stap. Een geldig plan geeft
geen app schrijfbevoegdheid. De toekomstige kernelcaller moet opnieuw autoriseren,
alle 512 bytes veilig naar de private bouncebuffer kopiëren vóór publicatie,
barrières toepassen en de buffer onveranderd/gepind houden tot completion of
bevestigde reset. Eén request tegelijk blijft het uitgangspunt.

## Acceptatie eerste stap

Hosttests controleren featureweigering en onbekende bits, headerbytes en
DMA-richting, lege/uiterste capaciteit, pooloverlap/alignment/overflow,
verkeerde descriptorketens en alle completionvelden inclusief indexwrap.
Alle bestaande M5-coretests blijven behouden. Rustfmt, Clippy met warnings als
fouten en `x86_64-unknown-none`-controle moeten slagen. Bouw en boot daarnaast
het ongewijzigde M5-profiel en controleer het bestaande schijforakel, zodra de
ontwikkelomgeving dat toelaat. Laat een andere agent de wijziging reviewen.

Hosttests van plannen zijn geen uitgevoerde WRITE/FLUSH, persistentiebewijs,
DMA-containment of crashherstelbewijs. M5-regressies blijven alleen-lezenbewijs.

## Resterend vóór een volledige M6a-mijlpaal

1. Ontwerp aparte WRITE- en FLUSH-autoriteit, scope, ABI en dienstprotocol;
   laat bestaande READ-only apps en foutcases hun rechten behouden.
2. Integreer transport en ring-3-dienst met begrensde usercopy, tickets,
   deadlines, completion, reset/quarantaine en bufferlifetime.
3. Laat het harnas uitsluitend nieuwe synthetische schrijfimages maken;
   geen hostdevices of padparameter voor een echte schijf. Boot-ESP blijft apart.
4. Bewijs extern dat uitsluitend toegestane sectoren wijzigen, overige bytes
   gelijk blijven en WRITE→completion→FLUSH→completion→herstart data teruglevert.
5. Test IOERR/volle backend, gedeeltelijk uitgevoerde writes, onderbreking vóór
   en na flush, dienstcrash en onzekere uitkomst na timeout. Geen blind opnieuw
   schrijven of rollback claimen: reset draait een uitgevoerde write niet terug.

WRITE-completion is geen duurzaamheidsbewijs. FLUSH moet na een voltooide
WRITE worden aangeboden; zelfs die volgorde is pas bewezen na echte device- en
herstartproeven binnen het gekozen backendprofiel. Bestandssysteem, journaling,
updates, cryptografie en fysieke hardware blijven verdere stappen.

## Primaire bron en toetsbaarheid

OASIS Open, *Virtual I/O Device (VIRTIO) Version 1.2*, Committee Specification 01,
1 juli 2022, §§2.7 (splitqueue/used-lengte) en 5.2 (blockdevice).
[Officiële specificatie](https://docs.oasis-open.org/virtio/virtio/v1.2/cs01/virtio-v1.2-cs01.html).
De HTML-route gaf hier HTTP 403. Op 28 oktober 2026 is daarom de officiële
[bron op tag v1.2-cs01](https://raw.githubusercontent.com/oasis-tcs/virtio-spec/e230375adde48ff39f734ea2783ef7b9d2ba346a/content.tex)
gelezen: commit `e230375adde48ff39f734ea2783ef7b9d2ba346a`, 309631 bytes,
SHA256 `08554df46753374735954a8436f8786a8cc5bd0b29f9f260f434e8c05061d7fd`.
Het exacte één-sector-/één-requestprofiel en strikte used-lengte zijn projectkeuzes.

Eigenaar gedeeld contract/docs/Git: root. Corebuilder: B06, uitsluitend
`crates/virtio-core/`. Reviewer: andere agent, eerst alleen lezen. Herstel via
aparte checkout van basis `5a58a74`; geen destructieve reset of schijfmigratie.

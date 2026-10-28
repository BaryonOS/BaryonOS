# M6b — native begrensde objectdienst

Eigen dependencyvrije Rust-core met `no_std` en `forbid(unsafe_code)`. `wire.rs`
en `journal.rs` worden op de host getest en in de native ELF opgenomen.
`user_service.rs` is de daadwerkelijke ring-3-adapter en wordt rechtstreeks
door de native gebruikersfixture gecompileerd met `crate::{rt,wire,journal}`.
De library alleen voert geen I/O uit en bewijst geen native isolatie/durabiliteit.

## Bevoegdheden en grenzen

Vier objectslots, IDs0–3. De kernel maakt twee dienstendpoints: het ontvangende
endpoint bepaalt domein0 (object0/1) of domein1 (object2/3). De echte
`Message.sender_pid` moet bovendien overeenkomen met de aan dat domein gebonden
client-PID. Een request bevat geen domein, replycapability, devicecapability,
absoluut sectoradres of pointers naar de opslag. Onverwachte ontvangen
capabilities worden gesloten vóór verdere verwerking.

De dienst bezit één Block-capability met READ|WRITE|FLUSH97, sectors4–35 van
de synthetische schijf. Clients krijgen uitsluitend SEND naar hun eigen
dienstendpoint en RECV op hun eigen antwoordendpoint. Rechten op hetzelfde
endpoint omvatten in deze eerste proef GET/PUT/DELETE binnen dat domein.

Maximaal128bytes per object,32records van512bytes, vaste tabellen, geen heap,
geen GC, geen netwerk en geen encryptie. Dit is een native opslagfundament;
geen filesystem, NAS, algemene app-API of productgeschikte opslaglimiet.

## Mutaties en herstel

- GET geeft inhoud en objectrevisie. Een nooit bestaand object geeft
  `NotFound=-19`, revisie0; een verwijderd object geeft NotFound met zijn
  behouden tombstonerevisie. Een aanwezig leeg object geeft succes, lengte0.
- PUT bevat de verwachte revisie. CAS0 maakt een nieuw object; de actuele
  tombstonerevisie maakt expliciete heraanmaak mogelijk. Succes verhoogt de
  objectrevisie met één.
- DELETE vereist de actuele revisie en een bestaand object. Een oude revisie
  geeft Stale; een al afwezig object met actuele revisie geeft NotFound zonder
  I/O. De tombstone behoudt de verhoogde revisie en bevat geen oude payload.
- `State::prepare` wijzigt niets. De adapter schrijft exact één nieuwe sector,
  wacht op WRITE-completion en daarna op echte FLUSH-completion, roept pas
  daarna `State::commit` aan en verstuurt het appantwoord. Een prepared record
  mag uitsluitend op zijn oorspronkelijke toestand worden gecommit.
- Servicecontract, alleen op host-ABI-niveau getest: iedere onzekere
  mutatieuitkomst zet de dienst op `Unknown`. Verder geldige requests krijgen
  `Unknown` totdat een nieuw proces het hele journal herstelt. WRITE/FLUSH
  worden nooit automatisch herhaald. Native is dit foutpad niet gerealiseerd:
  de acceptatieobserver kan bij WRITE-/FLUSH-fouten panikeren vóór gecontroleerde
  afhandeling van `Unknown`, ook met `milestone-m6b`. Dit defect is niet
  gerepareerd. Zie de [bewijsgrenzen](../../docs/implementation/M6B-OBJECT-STORAGE.md#bewijsgrenzen);
  herziening vereist een native reparatie met devicefoutinjectie en nieuwe
  native verificatie met bronbinding.
- `request_id` correleert verzoek en antwoord; het is geen los idempotencysysteem.
  Een oude request met oude CAS-revisie wordt geweigerd.
- Recovery scant alle32sectoren in volgorde. Alleen complete geldige prefixes
  gevolgd door uitsluitend nulsectoren worden geaccepteerd. Een beschadigd
  bezet record, gat, foutieve volgorde, revisiesprong of delete zonder bestaand
  object geeft Corrupt. Recovery gokt geen eerdere waarde en formatteert niet.
- Een leeg, volledig nul journal is een geldige lege toestand. Een vol journal
  geeft Full vóór WRITE. Deze stap claimt geen herstel bij fysieke stroomuitval.

## Wireprotocol versie3

Payload is exact224bytes in Message320; TAG=`0x4d36424c4b000003`.
Alle integers zijn little-endian; ongebruikte bytes en payloadstaart zijn nul.

| Offset | Betekenis |
|---:|---|
|0/2|u16 versie3 / operatie GET1, PUT2 of DELETE3|
|4|u32 gereserveerd0|
|8/16|u64 request-ID / service-epoch|
|24/32|u64 object-ID / verwachte of geretourneerde revisie|
|40/44|u32 lengte / i32 status|
|48–64|gereserveerd0|
|64–192|payload, maximaal128bytes met nulstaart|
|192–224|gereserveerd0|

`request(id,epoch,object,revision,op,value)` en
`reply(header,epoch,revision,status,value)` construeren bounded arrays.
`validate_request(bytes,epoch,domain)` controleert formaat, identiteitsscope en
semantiek; `validate_reply(bytes,request_header,epoch)` controleert correlatie,
revisie en canonieke antwoorden. Alle foutantwoorden bevatten lengte0 en geen
payload; alleen NotFound mag de reeds geautoriseerde objectrevisie teruggeven.

## Journalformaat

Sector512: magic `NOSM6B01`@0; version u16=1@8; header u16=64@10;
flags u32=0@12; sequence u64@16; request-ID u64@24; object u64@32;
revision u64@40; PUT2/DELETE3 u16@48; lengte u16@50; reserved0@52–64;
payload@64–192 met nulstaart; padding0@192–504; checksum u64@504.

Sequence is exact slotindex+1. Objectrevisies nemen per object met precies één
toe, inclusief deletes. FNV-1a64 over alle512bytes met het checksumveld nul is
alleen diagnostische corruptiedetectie, geen authenticatie of botsingsgarantie.

Publieke core-API: `State::{new,sequence,get,prepare,commit}`,
`Prepared::record`, `decode_record`, `Recovery::{new,push,finish}`.
`Recovery::push` heeft een blijvende fouttoestand: een eenmaal afgewezen record
kan niet worden weggepoetst door dezelfde scanner later andere bytes te geven.

## Native startup en observatie

Startupoffsets: epoch32, Block-cap48, ingress56/80, reply88/72 voor domein0/1,
client-PIDs160/168, service-PID176, case112. De runtime levert vaste
MSG/OUT/SECTOR-buffers en de bestaande scalaire int80-ABI.

Service schrijft sequence128 en status232. Stage192:1 ready (pas na volledige
recovery en FLUSH),2 vóór WRITE,3 na WRITE-completion vóór FLUSH,4 na
FLUSH-completion/commit vóór appantwoord,5 Unknown. Startupfout zet status232
en completion40=1 zonder READY. Deze velden zijn diagnostiek; de kernelobserver
moet echte devicepublicaties/completions en IPC-antwoorden onafhankelijk volgen.

## Controles

Gerichte hostcommando's vanuit de root:

```text
cargo +1.90.0 test -p object-core --locked --offline
cargo +1.90.0 check -p object-core --target x86_64-unknown-none --locked --offline
```

Uitgevoerde commando's, native QEMU-resultaten, bronhashes en beperkingen worden
door root vastgelegd onder `docs/verification/`; deze handleiding is geen
vervanging voor dat bewijs. Geen bestaande M6a-volumes of protocollen migreren:
M6b heeft een eigen profiel, magic, versie en wegwerpimage.

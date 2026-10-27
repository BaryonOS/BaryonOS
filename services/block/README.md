# Eerste blokdienst (M5)

Deze crate bevat het zuivere wireprotocol voor de eerste geïsoleerde Rust
blokdienst. `src/user_service.rs` wordt rechtstreeks opgenomen in de echte
`no_std` ring3-ELF uit `userspace/m5_probe.rs`; het is geen hostsimulatie.
De hostcrate test alleen `wire.rs`. De kernel beheert het virtio-transport,
de vijf vaste DMA-pagina's, tickets en reset/quiescence. Dit is geen volledig
userspace-hardwaredriver en geen bewijs tegen kwaadaardige DMA zonder IOMMU.

De dienst heeft één getypeerde READ-only Block-capability en twee eigen
requestendpoints. Elk endpoint is gebonden aan één sector van 512 bytes,
in de wegwerpproef sector 4 respectievelijk 6. De cliënt bezit uitsluitend
SEND op deze endpoints. De dienst kiest de sector uit het ontvangstendpoint
en antwoordt uitsluitend via het vooraf gebonden replyendpoint. Een app kan
geen absoluut LBA, devicecapability of replyhandle opgeven.

## Wirecontract versie 1

Een verzoek of antwoord is exact 192 payloadbytes in de bestaande Message320.
Alle integers zijn little-endian; er zijn geen Rust struct-layoutaannames.

| Offset | Type/bytes | Betekenis |
|---:|---|---|
| 0 | u16 | versie 1 |
| 2 | u16 | READ=1; WRITE=2 geeft NoSys |
| 4 | u32 | flags, nul |
| 8 | u64 | request-ID, door antwoord overgenomen |
| 16 | u64 | service-epoch |
| 24 | u64 | relatieve byteoffset binnen de gebonden sector |
| 32 | u32 | lengte, READ-verzoek 1..128 |
| 36 | i32 | status, nul in verzoek |
| 40 | 8 | reserved, nul |
| 48 | 128 | gegevens, nul in verzoek; alleen werkelijk gelezen bytes in antwoord |
| 176 | 16 | reserved, nul |

TAG is `0x4d35424c4b000001`. Verzoeken hebben geen overgedragen caps.
Onverwachte ontvangen caps worden direct gesloten. Daarna worden de echte
kernel-sender-PID, het outer bericht, reserved/data, epoch, opcode en het
bereik gecontroleerd. `offset + lengte` gebruikt checked arithmetic en mag
512 niet overschrijden. Een foutantwoord gebruikt de huidige server-epoch,
dezelfde ID/opcode/offset, lengte 0 en volledig nul data. Ongeldige apprequests
en WRITE bereiken het backendpad niet. De kernelobserver moet deze afwezigheid
van I/O onafhankelijk controleren; een cliëntcounter is daarvoor onvoldoende.

Appstatussen: Invalid=-1, NoSys=-2, Range=-3, Access=-5, Stale=-10,
Io=-12, Timeout=-13, NoDevice=-14, DeviceProtocol=-15. Deze appstatussen staan
in de payload en zijn gescheiden van de syscallresultaten in RAX/RDX.

## Device-interface en herstel

ABI-info is 1.1. `block_submit` (0x40) krijgt
`(cap, relative_sector, offset, length, 0, 0)` en geeft bij succes een ticket
in RDX. `block_poll` (0x41) krijgt `(cap, ticket, outptr, 320, 0, 0)`;
EAGAIN betekent dat de dienst yieldt. Alleen een succesvol gevalideerde
poll-Message levert bytes voor een appantwoord. Er staat geen diskseed of
verwachte respons in de dienst. Na een devicefout wordt uitsluitend READ
maximaal eenmaal herhaald; reset-timeout/no-device hebben geen dataclaim.

Nieuwe requestcaps gaan via echte IPC: dienst deriveert SEND|TRANSFER (9),
testsupervisor ontvangt 9 en draagt SEND (1) aan de cliënt over. Tussencaps
worden gesloten. Bij herstart leert de supervisor de nieuwe service-PID uit
Message.sender_pid en vereist hij exact de volgende epoch. De cliënt houdt
zijn eigen adresruimte, controleert oude EREVOKED-handles, sluit ze zelf en
start de sectorassemblage opnieuw. Alleen de kerneltestcoördinator mag na
fault en devicequiescence het vaste serviceslot vervangen; er is geen algemene
spawn- of beheer-API.

## Verificatiegrens

Hostcommando: `cargo +1.90.0 test -p block-protocol --locked --offline`.
Gebruik de projecttoolchain met een eigen `CARGO_TARGET_DIR` onder
`build/m5-service-tests`. De negen zuivere tests controleren het wirecontract;
ze bewijzen geen CPL3, device-I/O of herstart. De eigen ELF-handleiding en
artifacttest staan in `userspace/M5-PROBE.md`.

De geïntegreerde gastgate moet malformed requests zonder device-I/O, echte
sectorbytes, devicefoutpaden en drie pending-read-crashes met herbinding
aantonen. Twee verschillende hostgegenereerde images worden met dezelfde
kernel/userbinary vergeleken; FNV-1a64 is slechts diagnostische testmetadata.
Lokale werkinvoer onder `verification/` is genegeerd; root bewaart geselecteerde
tekstbewijzen onder `docs/verification/m5/service/`. Geen WRITE, filesystem,
M6, netwerk- of fysiekehardwareclaim.

# M5 — geïsoleerde blokdienst en virtio-transport

Bouwbasis vastgelegd op 27 oktober 2026, gestart op `c630c59`. Dit concretiseert roadmap M5. Interfaceversie 1; root coördineert wijzigingen en samenhangende tests. Dit document beschrijft acceptatiecriteria, geen reeds behaald testbewijs.

## Resultaat en bewijsgrens

Een echte Rust no_std-dienst in ring 3, een client en een usertestsupervisor gebruiken de bestaande Message320-IPC. Een klein vertrouwd kerneltransport leest via modern virtio-blk-PCI een per run gegenereerde, alleen-lezen 64KiB-image. Vier replies van 128 bytes vormen één sector van 512 bytes. Twee requestendpoints binden afzonderlijk sector 4 en 6. Alleen de dienst krijgt een getypeerde READ-devicecapability.

WRITE, FLUSH, DISCARD, bestandssystemen en netwerk zijn geen M5-interfaces. Een WRITE-dienstverzoek faalt zonder device-I/O. Schrijven en persistentie volgen in M6.

De vertrouwde basis (TCB) omvat het kerneltransport en het protocolconforme QEMU-device/backend. De userdienst kan geen DMA-adres, descriptor, BAR/MMIO, PCI-configuratie of busmasterinstelling kiezen. DMA bereikt volgens dit driverprotocol uitsluitend vijf permanent gepinde supervisor-RW/NX-kernelpagina's. Zonder IOMMU is geen containment van een kwaadwillig device bewezen. Dit is een geïsoleerde dienst met kerneltransport, geen volledig userspace-hardwaredriver.

## Driver en geheugen

- Exact testdevice `00:05.0`, vendor `1af4`, device `1042`; de bestaande boot-ESP is een ander device. Eén splitqueue met acht plaatsen, maximaal één uitstaande READ, sector 512 bytes, resultaatslice 1–128 bytes.
- PCI-capabilityketen, BAR-type/grootte, capabilityoffset/lengte en notify-multiplier worden begrensd en met overflowcontrole verwerkt. Alleen gevalideerde devicegebieden krijgen supervisor-RW/NX-UC-mappings. Geen algemene RAM-map-bypass of user-MMIO.
- Vereis VIRTIO_F_VERSION_1; onderhandel uitsluitend begrepen features. Alleen-lezen testbackend; geen willekeurige devicecommando's. Capaciteit komt uit stabiel gelezen deviceconfiguratie.
- Vijf vaste pagina's: descriptors, available-ring, used-ring, request/status en bouncebuffer. Header 16 bytes leesbaar voor device, bouncebuffer 512 bytes en status 1 byte schrijfbaar; keten 0→1→2. Valideer de volledige draft vóór publicatie. Geheugenbarrières vóór available/notify en na used-index vóór dataread. Device-interrupts blijven gemaskeerd; PIT/yield verzorgen voortgang.
- Submit, poll en reset zijn eindige stappen; geen I/O-wachtlus met interrupts uit. Succes vereist head 0, juiste indexstap, used-lengte 513 en status OK. IOERR/UNSUPP levert geen data. Ongeldige id/index/lengte/status geeft een lokaal foutpad.
- Gepubliceerde descriptors en bouncebuffer blijven ongewijzigd tot completion of bevestigde reset (status 0). Cancel/exit ontkoppelt het resultaat en vraagt reset; nooit vroeg wissen. Zonder resetack blijft het device offline en de pool in quarantaine tot VM-stop. Poolpagina's gaan niet terug naar de frameallocator.
- Het wissen van PCI-busmastering bewijst op zichzelf geen beëindiging van oude DMA. Een used-entry heeft geen epoch/nonce; softwaretickets detecteren geen kwaadwillige replay met dezelfde head. Protocolconform resetgedrag is hier een expliciete aanname.

## Autoriteit en ABI 1.1

`capability-core` behoudt 64 endpointobjecten en voegt maximaal vier BlockDevice-objecten toe. Scope bevat device-id, eerste sector en sectoraantal; M5 kent device 0, eerste sector 4 en vier sectoren toe. READ1 hoort uitsluitend bij BlockDevice; SEND1 uitsluitend bij Endpoint. Controleer objecttype vóór operatierecht. DERIVE4, TRANSFER8 en REVOKE16 vereisen het typespecifieke masker en een subset. RECV2 is ongeldig voor BlockDevice. De dienst ontvangt READ1; een ruimer manifestroot wordt vóór userentry afgezwakt en gesloten.

`abi_info` is voor M5 `(1<<32)|1` (major 1, minor 1); M1–M4 blijven 1.0. Nieuwe fouten: -12 EIO, -13 ETIMEDOUT, -14 ENODEV en -15 EPROTO. Registerconventie blijft gelijk.

| Syscall | Argumenten | Resultaat |
|---|---|---|
| 0x40 block_submit | cap, relatieve sector, offset, lengte; R8/R9 nul | RDX opaque ticket; geen userpointer |
| 0x41 block_poll | cap, ticket, outputpointer, grootte 320; R8/R9 nul | RDX lengte; Message320 met sender 0, tag ticket, data en alle overige bytes nul |

Een ticket bindt PID, objectkey/generation, transportepoch en een monotone id zonder wrap. Iedere poll herautoriseert READ op hetzelfde object. Een andere geldige READ-handle van dezelfde PID naar hetzelfde object mag pollen. Een stale/revoked handle faalt; procesexit annuleert het resultaat.

Controlevolgorde: syscall/ongebruikte argumenten → vereiste pollgrootte → cap/generation/node/object/type/READ → pollticket of submitgrenzen → volledige outputvalidatie → consumptie en kopie. Lengte 0 of groter dan 128 geeft EMSGSIZE; offsetoverflow/buiten 512 geeft EINVAL; sector buiten scope geeft EACCESS. Offline geeft ENODEV, bezet/ongereed EAGAIN, onjuist/afgehandeld ticket EINVAL. Geen userpointer blijft over scheduling of I/O bewaard. Ongeldige output consumeert geen completion.

## Dienstprotocol en herstart

Payload exact 192 bytes binnen Message320; apprequests hebben cap_count 0:

| Offset | Veld |
|---|---|
| 0 | versie u16 = 1 |
| 2 | opcode u16: READ1, WRITE2 niet ondersteund |
| 4 | flags u32 = 0 |
| 8 | request-id u64 |
| 16 | dienstepoch u64 |
| 24 | relatieve byteoffset u64 |
| 32 | lengte u32 |
| 36 | status i32, request 0 |
| 40 | gereserveerd u64 = 0 |
| 48 | data 128 bytes; request nul, reply ongebruikte bytes nul |
| 176 | gereserveerd 16 bytes = 0 |

Het ontvangstendpoint bepaalt vaste scope en replyendpoint; appvelden kiezen geen fysieke LBA of replycap. Onverwachte ontvangen capabilities worden gesloten. Andere ongeldige velden, epoch of ranges leveren een nulgevuld foutantwoord; daarna werkt een geldig verzoek. De kernelobserveerder moet foutverzoeken en replies koppelen en bewijzen dat daarbij geen blockcall/publicatie/completion plaatsvond.

Eén M5-user-ELF met private rollen: dienst 0, client 1, supervisor 2; CPU-negatieven voegen attacker 3 toe. Gemeenschappelijke virtuele data op `0x40010000`, Messagebuffers op `0x40011000`/`0x40012000`; frames blijven per proces privé. Clientdigests op offsets 120/128. Geen seed of verwachte schijfbytes in guestconfiguratie.

Een begrensde kerneltestcoördinator mag uitsluitend het vaste dienstslot herstarten na echte fault/exit en veilige devicequiescence. Geen algemene spawn-API. Client-PID, adresruimte en voortgang blijven leven. De nieuwe dienst krijgt een nieuwe PID, epoch en endpointobjecten. Zij stuurt twee echte SEND|TRANSFER-requestcaps via IPC naar de supervisor; die stuurt SEND-only door via het client-controlendpoint. Geen stil herschrijven van handles. De client ziet oude EREVOKED, sluit die en hervat. Case 13 herhaalt drie pending-read-crashes en leest succesvol in epoch 4. Dit bewijst niet dat DMA fysiek tegelijk met de fault liep.

## Acceptatiecases

Iedere case vereist kernelobservaties van calls, publicatie, completion, bytes, resources en voortgang. Alleen userstatus is onvoldoende. Alleen synthetische checksummetadata wordt gelogd. Verwachte afsluiting: `M5:CASE:NAME:PASS`, `M5:SURVIVOR_PROGRESS:PASS`, `M5:RESOURCES:PASS`, `M5:PASS`, `M1:PASS`, raw-exit 33, geen timeout of onverwachte FAIL.

| Case | Feature | Vereist bewijs |
|---:|---|---|
| 0 | milestone-m5, READ | client→dienst→virtio→reply; sector 4/6 via vier slices; twee verschillende images bij dezelfde kernelbinary |
| 1 | test-m5-service-errors | versie/op/flags/reserved/len0/129/overflow/epoch/scope/caps vóór I/O geweigerd; daarna geldig verzoek |
| 2 | test-m5-device-rights | verkeerde type/rechten/scope/ticket/cap; ongeldig polloutput onveranderd en niet geconsumeerd; daarna goede poll |
| 3 | test-m5-async | test-only notifydelay; echte EAGAIN/PIT/peerprogress en vervolgens completion |
| 4 | test-m5-bad-descriptor | draftinjectie vóór publicatie/notify geweigerd; geen onveilige DMA; herstel |
| 5–9 | test-m5-used-id/index/short/long/status | lokale snapshotinjectie na echte completion; geen corrupte usercopy; lokaal fout/reset/herstel |
| 10 | test-m5-io-error | geïnjecteerde IOERR-status geeft EIO, geen data, herstel |
| 11 | test-m5-timeout | test-only suppress-notify, tickdeadline, resetack, geen vroege poolreuse, herstel |
| 12 | test-m5-reset-timeout | test-only resetack niet waarnemen; offline/quarantaine/stabiele pool/peerprogress; geen datadigests |
| 13 | test-m5-restart | drie echte dienstcrashes, bevestigde reset, nieuwe PID/epoch, echte IPC-rebind, ingetrokken oude autoriteit, stabiele resources |
| 14–16 | test-m5-cpu-mmio/dma/pci | userread van MMIO/pool of PCI-out laat alleen attacker faulten; dienst/client werken |
| 17 | test-m5-no-device | testdevice werkelijk weggelaten; boot-ESP niet gekozen; lokaal ENODEV, peerprogress, geen submissions |

Exacte featurenamen staan in tools/build.py. Alle cases behalve 12/17 moeten herstellen en beide sectorchecksums leveren. De host genereert iedere run een andere image; de guest kent geen fixtureantwoord. `M5:SECTOR4_DIGEST:<16hex>` en `M5:SECTOR6_DIGEST:<16hex>` komen uit ontvangen bytes. De host vergelijkt FNV-1a64 en de ongewijzigde SHA256 van de volledige image; interne guestasserts vergelijken kopieën bytegewijs. FNV is diagnostiek, geen cryptografie.

Aanvullend: hosttests voor types/rechten/transfer/revoke/quota/generation, PCI-loops/BARoverflow, descriptors, completionvelden, tickets/nonwrap, reset en quarantaine. Faultinjecties blijven expliciet gelabeld en compile-time test-only. Volledige M1–M4-regressie, review door een andere agent, herstel en finale hash-/remotecontrole zijn vereist.

## Ownership en oplevering

- kernel_m2: kernel/**, crates/kernel-core/**, drivers/virtio-blk/**; transport, scheduler, manifest, kernelobserveerder.
- quantum_contract: crates/virtio-core/**, crates/capability-core/**; pure policy/parser, getypeerde caps, hosttests.
- product_acceptance: services/block/** en nieuwe M5-userspacebestanden; dienst/client/supervisor, protocol, artifacttests. Bestaande probes behouden.
- Root: workspace/lockfiles/tools/docs, images, build-/runharnas, branches/checkpoints. Maximaal drie actieve agents, één schrijver per bestand; reviews eerst alleen lezen.

Na betekenisvolle stappen en vóór lange tests privé pushen naar `codex/m5-block-service`. Bewaar bron, pins, echte logs/exits en open punten; geen binaries/caches/secrets. Herstel via een aparte consistente checkout op `c630c59`, geen destructieve reset.

## Primaire bronnen

OASIS Open, *Virtual I/O Device (VIRTIO) Version 1.2*, Committee Specification 01, 1 juli 2022: https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html — §§2.4, 2.7, 3.1, 3.3, 4.1, 5.2. Versie 1.2 is de gekozen protocolbasis, geen claim van nieuwste versie.

QEMU Project, system-manpage en lokaal geïnstalleerde 11.1-help: https://www.qemu.org/docs/master/system/qemu-manpage.html — device/BDF/readonly-opties. Gericht gecontroleerd op 27 oktober 2026. Quota, ABI en acceptatiecases hierboven zijn projectbesluiten.

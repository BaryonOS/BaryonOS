# M5 — geïsoleerde blokdienst en virtio-transport

Vastgelegde bouwbasis27 oktober2026, gestart op c630c59. Geen algemene conceptselectie: dit concretiseert roadmapM5. Root coördineert deze interfaceversie1; wijzigingen alleen met concrete fout en samenhangende tests.

## Te leveren resultaat en bewijsgrens

Eén echte no_std ring3-blokdienst, één cliënt en één usertestsupervisor gebruiken bestaande Message320-IPC. Een kleine trusted kerneltransportdriver leest via een modern virtio-blk-PCI-testdevice een per run gegenereerd readonly64KiB-image. Vier128B-replies vormen één512B-sector. Twee requestendpoints binden afzonderlijk sector4 en6; alleen de service krijgt een getypeerde READ-devicecap. WRITE/FLUSH/DISCARD/filesystem/netwerk zijn geen M5-interfaces; een WRITE-serviceverzoek faalt zonder device-I/O. Verdere schrijf-/opslagsemantiek volgt M6.

De kerneltransportdriver en het protocolconforme QEMU-device/backend behoren tot de TCB. De userdienst kan geen DMA-adres, descriptoradres, BAR/MMIO/PCI-config of busmasterinstelling opgeven. DMA bereikt uitsluitend vijf permanent gepinde supervisor-RW/NX-kernelpagina's. Zonder IOMMU is geen containment van een kwaadwillig device buiten die pool bewezen. Directe user-MMIO/queue-/busmastermacht, userpages alsDMAtarget en poolhergebruik zonder completion/quiescence zijn blockers. Dit is een geïsoleerde userdienst/front-end met kerneltransport, geen volledig userspace-hardwaredriver.

## Driver- en geheugencontract

- Exact testdevice00:05.0, vendor1af4/device1042 modern virtio-blk; bestaande boot-ESP wordt niet als testvolume geselecteerd. Eén splitqueue8, maximaal één outstanding read,512B-sector, maximaal128B-resultaatslice.
- PCI-capabilityketen, BAR-type/lengte, capabilityoffset/length en notify-multiplier worden bounded/checked verwerkt. Alleen de gevalideerde common/notify/ISR/device-subregio's krijgen een aparte supervisor-RW/NX-UC-mapping. Geen algemene RAMmap-bypass, geen user-MMIO.
- Vereis VERSION1; onderhandel alleen begrepen features. Readonlytestbackend; geen virtio-write of willekeurige devicecommando's. Capaciteit komt uit stabiel gelezen deviceconfig, niet uit uservelden.
- Vijf vaste pages: descriptors,avail,used,request/status,bounce. Header16B read-only voor device; bounce512B enstatus1B writable; chain0→1→2. De draft wordt vóór publicatie gevalideerd. Memorybarrières vóór avail/notify en ná used-index vóór dataread. Device-IRQs blijven gemaskeerd; PIT/user-yield verzorgen voortgang.
- Submit/poll/reset zijn eindige stappen; geen I/O-wachtlus met IF0. Een normale usedcompletion vereist verwachte head0, juiste indexdelta en volledige512B+status voor succes. StatusIOERR/UNSUPP geeft geen data; onbekende/ongeldige id,len,index,status geeft lokaal protocolfoutpad.
- Gepubliceerde descriptors/bounce blijven ongewijzigd tot completion of bevestigde reset(status0). Cancel/exit ontkoppelt het resultaat en vraagt reset; geen toestemming om buffers vroeg te wissen. Bij ontbrekende resetack blijft device offline en pool in quarantaine tot VM-stop, nooit terug naar Frames.
- Een gewist PCI-busmasterbit bewijst op zichzelf geen beëindigde oude DMA. Virtio-used bevat geen epoch/nonce: softwareticketbinding detecteert geen kwaadwillige hardware-replay met dezelfde head. Vertrouwd resetgedrag is hier de devicegrens.

## Getypeerde autoriteit en ABI1.1

capability-core behoudt64endpointobjecten en voegt maximaal4BlockDevice-objecten toe. Scope bevat device-id,first-sector,count; M5-grantfirst4,count4. READ1 heeft alleen betekenis bij BlockDevice, SEND1 alleen bij Endpoint. Objecttype wordt vóór operatierecht gecontroleerd. Algemene DERIVE4/TRANSFER8/REVOKE16 blijven alleen met typespecifiek masker/subset mogelijk; RECV2 is ongeldig voorBlockDevice. De service ontvangt READ1; een eventueel ruimer manifestroot wordt vóór de eerste userentry gesloten/afgezwakt.

M5abi_info is major1/minor1, `(1<<32)|1`. Bestaande M1–M4-profielen blijven1.0. Argumenten/returnregisters blijven onveranderd. Nieuwe fouten:-12EIO,-13ETIMEDOUT,-14ENODEV,-15EPROTO; bestaande fouten blijven.

| Syscall | Args | Resultaat |
|---|---|---|
| 0x40 block_submit | cap,relative_sector,offset,length; R8/R9nul | RDX opaque ticket; geen userpointer/inputbuffer |
| 0x41 block_poll | cap,ticket,output_ptr,size320; R8/R9nul | RDXlength; Message320 metsender0/tagticket/len/data/restnul |

Ticket bindt PID+getypeerde objectkey/generation+transportepoch+monotone ticket-id zonder wrap. Iedere poll herautoriseert READ op hetzelfde scopeobject. Een andere geldige READ-cap binnen dezelfde PID naar hetzelfde object mag pollen; geen binding aan één lokaal handlenummer. Revoked/stale handles zelf falen, procesexit annuleert altijd.

Foutvolgorde: syscall/unusedargs→vereiste wire-size bijpoll→cap/gen/node/object/type/READ→ticket bijpoll of scalar-scope/bounds bijsubmit→volledige outputvalidatie→resultaatconsumptie/bytecopy. Len0/>128 geeft EMSGSIZE; offset/offset+len buiten512 of overflow EINVAL; sector buiten toegekende scope EACCESS. Deviceoffline ENODEV, bezet/ongereed EAGAIN. Geen userpointer over scheduling/I/O bewaren. Ongeldige output consumeert geen completion; later geldig ophalen blijft mogelijk.

## Dienstprotocol en herstart

Payload exact192B binnen Message320; apprequests cap_count0. Versie1/opREAD1,flags0,request-id,service-epoch,relative byteoffset,length1..128,status0,reserved0,data128,reserved16. OpWRITE2 geeft ENOSYS/EACCESS zonder submission. Ontvangstendpoint kiest vaste scope/replyendpoint; appvelden kiezen geen fysiekeLBA/replycap. Onverwachte ontvangen capdescriptors worden gesloten. Andere ongeldige velden/epoch/ranges geven een nulgevuld foutantwoord; daarna werkt een geldig request.

Eén M5-userELF met private rolconfig: service0/client1/supervisor2; CPU-negatieven hebben extraattackerrole. Gemeenschappelijke data40010000,Messagebuffers40011000/40012000; rol/captabel wordt met kernelbuilder afgestemd in de userhandleiding. Clientdigests staan op private offsets120/128. Geen gedeelde userframes.

Alleen een begrensde kerneltestcoördinator mag het vastgelegde dienstslot herstarten, na echte fault/exit en veilige devicequiescence. Geen algemene spawn-/beheer-API. ClientPID/adresruimte/counter blijven leven. Nieuwe dienst krijgt nieuwePID/epoch en endpointobjecten. Zij stuurt twee echte SEND|TRANSFER-requestcaps via IPC naar de usertestsupervisor; die stuurt SEND-only via het cliënt-controlendpoint. Geen stil herschrijven van cliënthandles. Cliënt ziet oudeEREVOKED, sluit die zelf en hervat. Restartcase herhaalt drie pending-read-crashes en eindigt met succesvolle reads; geen claim dat fysieke DMA exact gelijktijdig liep met de fault.

## Eindcriteria en cases vóór uitvoering

Alle uitspraken vereisen kernel-observaties van calls/driverpublicatie/completion/bytes/resources/progress. Userstatus alleen is geen bewijs. Geen inhoudslogs; alleen synthetische checksummetadata. Iedere case: M5:CASE:NAME:PASS→M5:SURVIVOR_PROGRESS:PASS→M5:RESOURCES:PASS→M5:PASS→M1:PASS, raw33/geentimeout/onverwachteFAIL.

| Case | Feature | Vereist bewijs |
|---:|---|---|
| 0 | milestone-m5, READ | normale client→service→virtio→reply; sector4/6 via vier128B-slices, twee verschillende images bij dezelfde kernelbinary |
| 1 | test-m5-service-errors | versie/op/flags/reserved/len0/129/overflow/epoch/scope/caps geweigerd vóór device-I/O; daarna geldige read |
| 2 | test-m5-device-rights | verkeerde objecttype/rechten/scope/ticket/foreigncap; ongeldig polloutput ongewijzigd en niet geconsumeerd; daarna geldige poll |
| 3 | test-m5-async | test-only notifydelay, echte EAGAIN/PIT/peerprogress, daarna echte completion |
| 4 | test-m5-bad-descriptor | draftinjectie vóór publicatie/notify geweigerd; geen onveilige DMA; later goede read |
| 5–9 | test-m5-used-id/index/short/long/status | lokale snapshotinjectie ná echte devicecompletion; geen corrupte usercopy, lokaal fout/reset/herstel |
| 10 | test-m5-io-error | geïnjecteerde bekende IOERR-status geeft EIO/geen data, gecontroleerd herstel |
| 11 | test-m5-timeout | test-only suppress-notify, eindige tickdeadline, resetack, geen te vroege poolreuse; later goede read |
| 12 | test-m5-reset-timeout | test-only resetack niet waarnemen; offline/quarantaine/stabiele pool en peerprogress; geen datadigest vereist |
| 13 | test-m5-restart | drie echte dienstcrashes met pendingrequest, bevestigde reset, nieuwePID/epoch, echte IPC-rebind, stale oudeautoriteit en stabiele resources |
| 14–16 | test-m5-cpu-mmio/dma/pci | echte user-MMIO-/poolread of PCI-out veroorzaakt alleen attackerfault; cliënt/service blijven werken |
| 17 | test-m5-no-device | testdevice werkelijk weggelaten, boot-ESP niet verkeerd geselecteerd; lokaal ENODEV/peerprogress/geen submissions |

Features gebruiken exacte uitgeschreven namen: used-index,used-short,used-long,used-status; geen wildcardfeatures. Case12/17 vereisen geen succesvolle readchecksums. Andere cases herstellen en leveren beide sectorchecksums. Root genereert per run readonly64KiB-image met andere synthetische bytes; guest kent geen seed/verwachte fixture. Marker M5:SECTOR4_DIGEST:<16hex> /SECTOR6_DIGEST komt uit werkelijk ontvangen bytes; host vergelijkt FNV-1a64 en ongewijzigdeSHA256-imagehash. FNV is diagnostische testchecksum, geen cryptografie. Interne guestasserts vergelijken kopieën bytegewijs.

Aanvullend: hosttests voor getypeerde rechten/transfer/revoke/quota/generation, PCI-caploops/BARoverflow, descriptoradressen/flags/ketens, completionvelden, ticketepochs/nonwrap en reset/quarantaine. Faultinjecties blijven expliciet gelabeld en compile-time test-only. Volledige M1–M4-regressie na integratie. Review door een andere agent, herstel en finale hash/remotecommitcontrole vereist.

## Exclusieve ownership en oplevering

- kernel_m2: kernel/**,crates/kernel-core/**,drivers/virtio-blk/**; driver/scheduler/manifest en echte observer.
- quantum_contract: crates/virtio-core/**,crates/capability-core/**; pure policy/parser/typedcap en hosttests.
- product_acceptance: services/block/**,nieuweM5-userspacebestanden; echte dienst/client/supervisor/protocol en artifacttests. Bestaande probes behouden.
- Root: rootworkspace/lockfiles/tools/docs,images/build/run/harnas/branches/checkpoints. Maximaal drie actieve agents, geen gedeelde schrijvers. Reviews eerst read-only.

Voortgang gaat na iedere betekenisvolle stap en vóór lange tests naar de privébranch codex/m5-block-service. Bron/pins/echte logs/exits/openpunten blijven hervatbaar; geen binaries/caches/secrets. Terug naar M1–M4 via een consistente aparte checkout op c630c59, geen destructieve reset.

## Primaire broncontrole

OASIS Open, *Virtual I/O Device (VIRTIO) Version1.2*, Committee Specification01,1juli2022: https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html ; gerichte §§2.4,2.7,3.1,3.3,4.1,5.2. QEMU Project, huidige system-manpage en geïnstalleerde11.1-help: https://www.qemu.org/docs/master/system/qemu-manpage.html ; device/BDF/readonly-variant. Gecontroleerd27oktober2026. Versie1.2 is de gekozen protocolbasis, niet een claim van nieuwste versie. Bovenstaande quota/ABI/cases zijn eigen projectbesluiten.

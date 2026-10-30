# TPM-CRB-TRANSPORT-1 — begrensde driverkern

Status: afzonderlijke safe/no_std-transport- en codeclogica in
`drivers/tpm-crb`; opt-in MMIO-bootstrapfixture nu in implementatie volgens
ADR-052, beide native gevallen nu groen op `4b09869`. Geen productkernelgrant of koude
rollbackankering. Hostmodellen zijn uitsluitend protocol-/registertests.
De nog verplichte B02-/boot-/update-eisen blijven open.

`native-tpm-crb` houdt uitsluitend een vaste supervisor-RW/NX/UC-mapping
op VA `0x3f180000`, met typed registeroffsets, checked buffergrenzen,
maximaal128B-kopieën, volatile accessen en MFENCE/compilerbarrières.
De ene claim en quarantaine blijven voor de VM-levensduur behouden.
Geen app krijgt TPM-recht; de bestaande twee echte CPL3-wachtapps blijven
hun oorspronkelijke rechten en volledige orakels gebruiken. De scheduler
pollt de bestaande transportkern met echte PIT-ticks; één read-only
family-propertyquery en volledig bevestigde vrijgave worden geëist.
`test-tpm-crb-absent` vereist Unsupported/NotStarted/quarantaine en nul
writes. Zeven adaptergrensweigeringen mogen geen MMIO-write veroorzaken.
Runner: `tools/run_tpm.py --case normal|absent`; swtpm gebruikt uitsluitend
een beheerde eigen Unixsocket/state, geen echte TPM of netwerkverbinding.
De code hieronder blijft het transportcontract; oude open-statuszinnen
zijn geen nieuwe native acceptatie. Actuele uitvoering: PROJECT-STATE.
Bron-/artifact-/logbinding en de bewaarde eerste mislukking staan in
`verification/native-test-version/tpm-evidence.json`. Beide VM's raw33;
absent maakt nul writes, positief geeft family `0x322e3000` en bevestigde
Idle/relinquish. De bestaande volledige CPL3-wachtproeven blijven groen.

## Ondersteund interfacecontract

Het enige onderzochte apparaatmodel is QEMU10.0.13 `tpm-crb`, locality0,
MMIO-aperture0xFED40000..0xFED40FFF. De gebruikte machinecompatibiliteit blijft
`pc-q35-8.2`; dat is niet QEMU-versie8.2. De lokale beheerde binary meldt
10.0.13 Debian1:10.0.13+ds-0+deb13u1 en adverteert `tpm-crb`. Er is nog geen
TPM-instance met backend gestart. De configuratie hieronder is een verwacht
modelcontract op basis van upstreambron, geen native discoverybewijs.

CRB-registers zijn little-endian32bit. ControlRequest/Status/Cancel/Start staan
op0x40/44/48/4C; commandSize/address op0x58/5C/60, responseSize/address op
0x64/68/6C. Beide hardwarebuffers moeten exact hetzelfde3968B-gebied vanaf
0xFED40080 aanwijzen. Andere adressen, externe RAM/DMA-buffers, andere maten,
FIFO-interface, andere locality, idle-bypass of een niet-geselecteerde CRB
worden geweigerd. Er is geen automatische FIFO- of apparaatfallback.
InterfaceType1/version1, locality0-only, CRB-support en selector1 zijn vereist;
vendor/revision en transferbreedteflags worden niet als autoriteit behandeld.

`Io` bezit de ene adapter. Hij levert typed read32/write32(Register),
read_buffer/write_buffer met relatieve offsets binnen de vaste hardwarebuffer,
en quarantine(). Elke bufferactie is maximaal128B. Een toekomstige adapter
moet zelf de begrenzing, exact voltooide kopieën, UC/device-mapping,
compiler-/CPU-geheugenvolgorde en volatile register-/bufferaccessen afdwingen.
Bufferwrites moeten vóór START zichtbaar zijn; response-reads volgen pas na
de bevestigde completion. Deze voorwaarden zijn nog geen onderzochte unsafe
MMIO-implementatie. Er worden geen willekeurige fysieke pointers of frames
uit apparaatdata gemapt.

De buffer is in dit QEMU-model apparaat-MMIO, geen allocator-/user-RAM of
virtio-DMA-pool. Quarantaine voorkomt hergebruik of overdracht van de apparaat-
autoriteit en nieuwe writes. Het bewijst geen IOMMU-isolatie op fysieke
hardware. Een toekomstige broker blijft verantwoordelijk voor mappings,
capabilityhouder, intrekking, deadlines en processterfte.

## Transport en levensduur

`Transport` is niet Clone/Copy en geeft zijn Io niet vrij tijdens een
hardwaretransactie. API: acquire(now,deadline)→Ticket, poll_ready,
execute(&codec::Command,now,deadline,cancel_ticks)→Ticket, poll,
cancel, release(now,deadline)→Ticket, poll_released, detach.
Tickets zijn niet0, checked monotoon en mogen niet tussen bewerkingen worden
verwisseld. Verkeerde tickets veranderen de actieve transactie niet en voeren
geen I/O uit. Een voltooide response wordt eenmaal als een eigen getypeerde
waarde teruggegeven; een broker moet die zelf behouden wanneer usercopy nog
niet is bevestigd. Geen request wordt automatisch opnieuw gepubliceerd.

De caller levert echte monotone Sys3/PIT-ticks, nooit usergekozen tijd of een
poll-count-klok. De kern heeft geen eigen klok of verborgen wachtlus: iedere
poll is begrensd, en de toekomstige broker moet uiterlijk op de deadline
blijven pollen en eigenaarsverlies afhandelen. Deadline is absoluut, groter
dan now en maximaal60000ticks later; cancel_ticks ligt in1..60000. Voor het
huidige100Hz-profiel is dit maximaal600s, geen standaard producttimeout.
EAGAIN-achtige polling vernieuwt de oorspronkelijke deadline niet.
Achteruitgaande of overlopende tijd blokkeert actieve polls, nieuwe commands en release. Reeds geverifieerde Ready/Released/Cancelling-ACKs kunnen idempotent worden teruggegeven zonder nieuwe klokcontrole of I/O; zij bevestigen uitsluitend die eerdere toestand en verlenen geen nieuwe mutatie.

Acquire vraagt uitsluitend locality0, nooit seize/resetEstablishment. Na
bevestigde locality volgt COMMAND_READY; pas nadat request is gewist en het
apparaat Ready/START0 is, kan de commandbuffer worden beschreven. Buffers,
locality en status worden vóór gebruik opnieuw gecontroleerd. Reeds actief
werk of een vreemde/seized locality wordt niet overgenomen.

Vóór START wordt de volledige gedeelde buffer daadwerkelijk via Io gewist,
daartussen geen I/O buiten de vaste aperture. Daarna wordt het exact
gecodeerde commando geschreven. Dit voorkomt dat een oude succesheader als
nieuw resultaat wordt gebruikt. Het nieuwe commando kan onmiddellijk klaar
zijn; een vooraf waargenomen Busy1 is daarom niet verplicht. Een verdwenen
START-bit alleen is onvoldoende: ook locality, niet-idle status, vaste
bufferdescriptors, bounded responseheader, stabiele header en het specifieke
responseformaat moeten kloppen. Publicatie volgt pas na volledige validatie
en werkelijk wissen van de nu weer CPU-beschikbare hardwarebuffer.

`Failure {fault,effect}` onderscheidt NotStarted, Unknown en Completed.
NotStarted betekent uitsluitend dat voor deze commandbewerking geen START is
gepubliceerd; het belooft geen afwezigheid van locality-/Ready-statewijzigingen.
Een teruggegeven Ready na annulering lost de eerdere Unknown-transactie niet
op. Ook is_nv_mutation=false belooft geen afwezigheid van andere TPM- of
autorisatiestate-effecten. Effect wordt **vóór** de START-write Unknown: ook een mislukte write kan het
apparaat hebben bereikt. Iedere onzekere fout daarna blijft Unknown.
Completed bij een TPM-foutcode betekent alleen dat een canonieke TPM-respons
werd ontvangen; het betekent geen succes, geen opslagflush of algemene
no-side-effectsclaim. TPM_RC_INITIALIZE wordt bijvoorbeeld expliciet
teruggegeven en niet stil als geslaagde Startup aangemerkt.

Bij timeout/annulering wordt eenmaal CANCEL1 geschreven. Pas echte START0,
CANCEL0-write én waargenomen CANCEL0 laten bufferwissen/Ready toe. De caller
ontvangt dan alsnog Timeout/Cancelled met Unknown-effect, nooit een positieve
NV-mutatiebevestiging. Ontbreekt de begrensde completion/clear-ACK, dan volgt
permanente quarantaine. Geen bufferwrites, detach, regrant, replay of verzonnen
resetbevestiging. Er is bewust geen reset()-API. Hetzelfde geldt bij
onverwacht apparaatreset, gewijzigde descriptors, corrupte response of
partiële START-I/O. Drop roept de quarantainehook eenmaal aan tenzij nooit
gebruikt of volledig vrijgegeven; het doet geen verborgen apparaatreset.
De adapter moet die blokkade ook na eigen Drop in de broker behouden.

Release vereist Ready/START0, wist de buffer, vraagt GO_IDLE en wacht op het
gewiste request plus Idle1. Daarna pas localityRelinquish en de bevestiging
dat Assigned en Granted uit staan. Alleen daarna kan detach de adapter
teruggeven. Zonder ACK komt er geen impliciete vrijgave.

## TPM2-codecs, autorisatie en gegevens

De aparte codec encodeert alleen Startup(Clear/State), SelfTest(full),
GetCapability(TPM_PROPERTIES, maximaal16), NV_ReadPublic, NV_DefineSpace voor
één8B-counter, NV_Read van exact die8B en NV_Increment. Geen Clear,
UndefineSpace, hiërarchiewijziging, algemene raw-commandinterface, RNG-
alternatief, nieuwe cryptoprimitief of stil provisioningswachtwoord.

Commands zijn maximaal128B, responses maximaal512B. Tag, big-endian lengte,
commandcode, count, volgorde, vaste nvIndex, naamalgoritme, countertype,
authPolicy-lengte en datasize worden gecontroleerd. Iedere commandencoder
controleert invoer/uitvoerruimte vóór outputmutatie. De commandwaarde leent
alleen de geschreven callerbuffer. De caller beschermt/wist die eigen RAM;
de transportkern bewaart geen tweede RAM-kopie van het wachtwoordcommando.

NV-autorisatie is een expliciet door de caller aangeleverde TPM_RS_PW-borrow
(max32B, geen Default/Debug). Een lege waarde is alleen mogelijk door een
expliciet lege callerwaarde, nooit een verborgen fallback. De hosttests
gebruiken zichtbare synthetische waarden en voeren geen NV-write uit op een
TPM. Het huidige codecprofiel is geen uitgewerkt productie-authbeleid.
PW-sessies bieden geen versleuteling van het commandbuffer of cryptografische
response-HMAC. De TPM/MMIO-keten blijft vertrouwd; de gewone app mag er geen
recht op krijgen. Voor counters gelden AUTHWRITE/AUTHREAD, countertype en
SHA256-naamalgoritme; dictionary-attackbescherming wordt niet uitgeschakeld.
Alleen de dynamische WRITTEN-bit wordt aanvullend geaccepteerd.

Een wachtwoordrespons vereist lege nonce/HMAC en continueSession1, niet0.
Properties zijn bounded en strikt oplopend; vreemde capabilities of
niet-canonieke more/count-velden falen. NV_ReadPublic geeft een **opaque**
structureel passende naam terug: de codec berekent of authenticeert de
SHA256-digest niet en bewijst daarmee geen lineage. Dat hoort later bij de
beoordeelde key-/provisioningsdienst buiten de kernel. Foutresponses hebben
exact de passende10B-header; geen gedeeltelijke data wordt geaccepteerd.

## Rollback, boot, updates en privacy

`services/keyring/src/persistent.rs` vereist een onafhankelijk vertrouwde
minimum_generation. Hetzelfde getal naast een herstelblob of op dezelfde
terugrolbare disk is geen anker. Deze driver roept restore_persistent niet aan
en maakt die garantie nog niet waar. Een TPM-counter alleen verbindt ook nog
geen sleutel-lineage, geauthenticeerde snapshot, twee-copy-journal, flushvolgorde,
crashuitkomst of gebruikersbevestiging. Onzekere NV_Increment/Define-uitkomsten
mogen niet opnieuw worden uitgevoerd; eerst is een echt onderzocht
reconciliatie-/herstelcontract nodig.

Voor de nog open koude proef zijn afzonderlijke wegwerp-swtpm-state en
OS-disk nodig, met een nieuw QEMU/swtpm-proces per coldstart. Alleen de disk
terugrollen terwijl het onafhankelijke TPM-anker actueel blijft moet worden
geweigerd. Als een hostbeheerder óók TPM-state/snapshots terugrolt, is die
onafhankelijkheidsaanname verdwenen; daartegen wordt geen valse bescherming
geclaimd. Geen persoonlijke schijf, echte NAS, fysieke TPM of firmware wordt
hier gebruikt. Ontbrekend swtpm is geen reden om een testwachtwoord, RAMwaarde
of opgeslagen snapshot als hardwareanker voor te stellen.

TPM-aanwezigheid maakt geen geauthenticeerde boot-/updateketen of PQC-garantie.
De bestaande boot-/signatuur-/PQC-contracten en privacy-Security-Floor blijven
onverkort gelden. Geen leveranciersaccount, cloudsleutel, telemetrie, externe
opslag of extra netwerkrecht wordt geïntroduceerd. Per eigenschap blijft de
TCB benoemd: de transportgrens steunt op Io, device, klok en exclusieve broker;
koude antirollback bovendien op provisioning/auth, keycodec, crash-/diskcommit,
boot/updatevertrouwen en de werkelijk onafhankelijke persistente TPM-state.

## Broncontrole en nog open normatieve controle

De normatieve TCG-PC-Client-PTP-resource op
https://trustedcomputinggroup.org/resource/pc-client-platform-tpm-profile-ptp-specification/
was niet bereikbaar: hostproxy403. Er is **geen** gelezen normatieve PTP-PDF
of claim van TCG-compliance. FIFO/CRB worden niet vereenzelvigd. De nog
noodzakelijke primaire documentcontrole omvat PTP Chapter5 CRB en de relevante
TPM2 Library Parts2/3 commands/session/NV-counterregels.

De nu daadwerkelijk gelezen implementatiebronnen zijn:

- QEMU10.0.13, commit98f88efe8f07067667b8d9b1a5ce9551d34036c1:
  hw/tpm/tpm_crb.c, include/hw/acpi/tpm.h, tests/qtest/tpm-crb-test.c.
  Registerdefinities/layout, Ready/Start/completion/Idle/locality en de
  onmiddellijke-versus-uitgestelde completion zijn gecontroleerd.
- Tianocore edk2-stable202502, commitfbe0805b2091393406952e84724188f8c1941837:
  MdePkg/Include/IndustryStandard/TpmPtp.h en Tpm20.h, plus
  SecurityPkg/Library/Tpm2CommandLib/Tpm2NVStorage.c. Het Tpm20-header verwijst
  historisch naar Libraryrev0.96; dat is geen actuele normatieve speccontrole.
- Microsoft/TCG TPM2 reference, commitee21db0a941decd3cac67925ea3310873af60ab3:
  TPMCmd/tpm/src/main/SessionProcess.c, BuildResponseSessions: PW-respons zet
  continueSession en lege nonce/HMAC. Ook deze implementatie vervangt de
  ontbrekende normatieve PTP/Library-documenten niet.

Precieze URLs, lokale inhoudshashes, queryresultaten en beheerde QEMU-binaryhash
staan onder drivers/tpm-crb/verification. Upstreambron is gelezen, niet als
C-code overgenomen of meegebouwd: QEMU GPL2+, edk2 BSD-2-Clause-Patent,
Microsoft/TCG-reference BSD-licentie met expliciete patentdisclaimer. De
reference vermeldt zelf “This code is informative”. De notices en voorwaarden
blijven bij hun bronnen.
Er zijn geen nieuwe Cargo-dependencies. Er wordt hier geen algemene
projectlicentie gekozen of juridische audit geclaimd.

Nog open: normatieve TCG-controle, onafhankelijke bronreview, echte swtpm-
backend, unsafe/MMIO-adapter, capability-/brokergrants, native transport en
koude fout-/rollbackproeven, volledige boot/update/PQC-/B02-integratie en
productacceptatie. Hosttests en targetbuild leveren hiervoor geen punten.

# NAS-IPC-PROBE-1 — lokale native acceptatie-interface

Dit is een **afzonderlijke P04-probe**, niet de geïntegreerde release, een
interoperabel NAS-netwerkprotocol, geauthenticeerde apparaatkoppeling of
E2EE-client. De client verstuurt herkenbaar synthetische opaque bytes,
geen echte documenten of sleutels. P01–P04 blijven open tot hun vereiste
native bewijs bestaat; een ELF-build levert geen punt.

Bronwriter: NAS-spoor, uitsluitend `services/nas/**`. Kernelmanifest,
syscallselectie, rootbuild, QEMU-runner en finale bewijsbinding blijven
bij TL-REL. Zij worden niet door deze builder of binaries ingesteld.

## Artefacts en geheugen

Rust 1.90.0, `x86_64-unknown-none`, `no_std`, statisch ELF64, geen allocator
of externe package. Het ene `nas-probe.elf` bevat drie rollen. De afzonderlijke
procesadresruimten en hun kernelgrants bepalen de autoriteit, niet de code
of een rolveld uit een verzoek. `nas-volume.elf` bouwt de **ongewijzigde**
bestaande `sdk/examples/volume-docs/src/service.rs` met diens eigen linker.

```sh
NOS_BUILD_DIR=build/nas-probe python3 -B services/nas/native-probe/build.py
```

Eigen targets `target/nas-native/{nas-probe,nas-volume}`; geïnspecteerde
ELF's en `build-report.json` onder `build/nas-probe/users`. Binaries blijven
buiten Git. De builder gebruikt bestaande `inspect_user`, logging en
hashhelpers; geen tweede ELF-loader of filesystem.

Beide images behouden startupdata `0x40010000`, maximaal één RW/NX-pagina.
De probe gebruikt dezelfde 64-KiB-stack `0x7ffef000..0x7ffff000` en bestaande
64-KiB-private heap `0x60000000..0x60010000`. Alleen de NAS-rol plaatst één
geldige `Workspace` in de heap, zonder heap-grow, allocator of tweede
gebruiker. De normale Rust-constructor wordt gebruikt; geen aanname dat
alle nulbits een geldige Rust-layout vormen. Core/volume blijven `unsafe`-vrij.
De native adapter heeft uitsluitend de gedocumenteerde heapplaatsing,
startup-/intrinsicshergebruik en stackmeting als lokale unsafe.

De entry markeert de vooraf gemapte stack vóór het eerste Rust-frame.
Bij succes levert startupwoord **448** de aangeraakte stack-high-water.
Daarvoor wordt alleen geïnitialiseerd, inactief stackgeheugen onder de
huidige RSP gelezen; geen live Rust-borrow, redzone of andermans geheugen.
Dit is géén formele bovengrens voor ongebruikte stackreserveringen of nog
niet uitgevoerde foutpaden. Native waarden blijven open tot QEMU-uitvoering.
De ELF-footprint/hash en hostmetingen staan in de bijbehorende bewijsbundle.

## Processen, endpoints en rechten

Voorgestelde slots, nog door de rootintegrator te binden:

| Slot | Rol / image | Enige functionele grants |
|---|---|---|
| 0 | beheerder / `nas-probe.elf` | SEND naar NAS-adminingress, RECV op eigen NAS-reply, SEND/RECV naar gastcoördinatie |
| 1 | NAS / `nas-probe.elf` | twee RECV-ingresses, twee SEND-replies, SEND/RECV naar eigen volumedomein, entropybron |
| 2 | bestaande volumedienst / `nas-volume.elf` | gescopeerd block READ/WRITE/FLUSH (koud READ-only), twee eigen RECV-ingresses en vaste SEND-replies |
| 3 | gast / `nas-probe.elf` | SEND naar NAS-gastingress, RECV op eigen NAS-reply, SEND/RECV naar beheercoördinatie |

Alleen het vertrouwde kernelmanifest mint/delegeert deze bestaande
procesgebonden capabilities. Geen NAS-/gastverzoek kan capabilities creëren.
Reply-endpoints horen vast bij hun werkelijke ontvanger; er zijn geen
reply-capabilities of toegangsrechten in de wire. Iedere receive controleert
werkelijke `Received.sender`, tag, lengte en geen capabilitybijlagen.
Een rolveld of PID in een payload verleent geen autoriteit.

NAS bezit volumedomein 0, object 0. De tweede volumeingress blijft
geïsoleerd; wijs een andere manifest-PID toe en sluit ongebruikte gedelegeerde
SEND-handles vóór start. Geef beheerder/gast geen NAS-volume-, block-,
entropy-, launch- of netwerkgrant. Handles zijn slot/generatie **binnen één proces**; gelijke getallen
in verschillende processen zijn geen identiteits- of delegatiebewijs.

`LocalBinding` maakt voor deze probe een publieke `NOSPROC1 + PID`-principal
uit de werkelijk gecontroleerde afzender. Dit bewijst uitsluitend lokale
procesherkomst, **geen** cryptografische apparaatidentiteit of blijvende
accountidentiteit. Manifest-PIDs moeten bij de koude proef stabiel dezelfde
rollen aanduiden; productidentity/restartdelegatie wordt hier niet afgeleid.
Transport/TLS, peerpins en LAN/internetrechten blijven NET-eigenaarschap.

## Startupwords

Alle offsets zijn bytes in de bestaande private startup-pagina. Niet genoemd:
nul. Bestaande volumedienst gebruikt zijn **eigen ongewijzigde** layout.

| Offset | Beheerder (rol 0) | NAS (rol 1) | Gast (rol 3) |
|---|---|---|---|
| 32 | 1 | 1 | 1 |
| 48 | 0 | entropyhandle | 0 |
| 56 | NAS-admin SEND | admin RECV | NAS-gast SEND |
| 64 | eigen NAS-reply RECV | gast RECV | eigen NAS-reply RECV |
| 72 | gastcoördinatie SEND | adminreply SEND | admincoördinatie SEND |
| 80 | gastcoördinatie RECV | gastreply SEND | admincoördinatie RECV |
| 88 | 0 | volume SEND | 0 |
| 96 | 0 | volume-reply RECV | 0 |
| 112 | normaal 0, koud 1 | normaal 0, koud 1 | normaal 0, koud 1 |
| 120 | 0 | 1 | 3 |
| 160 | NAS-PID | admin-PID | NAS-PID |
| 168 | gast-PID | gast-PID | admin-PID |
| 176 | eigen PID | eigen PID | eigen PID |
| 184 | 0 | volume-PID | 0 |

De syscallset moet bestaande IPC, entropy en monotone klok omvatten.
Geen nieuwe syscall, algemene caprechten of betrouwbare RTC-claim.
De initieel lege synthetische disk gebruikt de bestaande
`tools/volume_disk.py::create_image`-fixture: 2.048 sectoren, waarvan alleen
de 580 sectoren vanaf sector 4 binnen de volumecapability vallen; canaries
erbuiten. Gebruik nooit een hostschijf.
Koud heropenen gebeurt op dezelfde gehashte image met READ-only blockgrant;
verwacht nul diskmutaties. Geen automatische lege initialisatie of cleanup.

## Wire v1: NASP / 256 bytes

Tag `0x4e41535000000001`, exact 256 bytes, little-endian. Coördinatie heeft
de afzonderlijke tag `TAG+1` en vaste peercontrole; dit is geen tunnel voor
NAS- of volumeautoriteit.

| Bytes | Betekenis |
|---|---|
| 0..4 | `NASP` |
| 4..6 | versie 1 |
| 6..8 | opcode 1..13; alleen reply zet bit `0x8000` |
| 8..16 | niet-nul requestcorrelatie, geen capability |
| 16..20 | status: 0 of vaste negatieve foutcode; verzoeken altijd 0 |
| 20..22 | payloadlengte 0..104 |
| 22..24 | nul |
| 24..56 | namespace waar opcode dat vereist, anders nul |
| 56..96 | 40-byte sessiehandle waar vereist, anders nul |
| 96..136 | 40-byte uploadhandle waar vereist, anders nul |
| 136..144 | verwachte/geretourneerde revisie of epoch |
| 144..152 | quota, objectindex, afloop of enumeratie-index volgens opcode |
| 152..256 | payload; alle bytes na lengte nul |

Opcodes: 1 Invite, 2 Accept, 3 Open, 4 Stage, 5 Commit, 6 Read, 7 Usage,
8 SetQuota, 9 Revoke, 10 Evict, 11 Shutdown, 12 UsageAt, 13 Close.
De parser controleert ook **ongebruikte velden per opcode**, tokenvorm,
replyvorm en lege foutreplies. Sessiedecodering verleent geen rechten:
de core controleert caller, levende handle, namespace en epoch opnieuw.
Geen principal, sleutel, mutable clientbuffer of capability in de wire.

De 104-byte-payload en één geselecteerde gast zijn zichtbare proefgrenzen,
geen productiecapaciteit. De core heeft andere expliciete proefcaps.
Staging reserveert quota en bevestigt geen duurzame publicatie; COMMIT
bevestigt pas na snapshot én historypurge. Elke opslagvervangingsfout
wordt `Unknown`, ook bij initialisatie; de originele oorzaak blijft lokale
diagnostiek. De instantie blokkeert vervolggebruik. Geen stil opnieuw
uitvoeren, succesfallback, oude snapshot of ACK vóór duurzaamheid.

De monotone klok is alleen voor aanbiedingen binnen dezelfde boot.
De koude probe accepteert geen oude aanbieding op geresette PIT-ticks.
Dit is een fail-closed proefbeperking, geen afgeronde persistent-pairing-UX
of betrouwbare tijdprovider; RTC mag dit niet stil vervangen.

## Normale en koude assertions

Normaal: admin biedt 128 bytes; gast accepteert en opent, stage/commit/leest
96 synthetische bytes; reserveert 32 vervangingsbytes. Admin ziet used 96,
reserved 32, weigert quota 127, trekt in zonder gastconsent. Gast krijgt
`Stale` op oude upload/read en `Denied` op opnieuw openen. Gastbeheer via
gastingress wordt geweigerd; ontvangen via de eigen SEND-only NAS-ingress
geeft kernel `Access`. Dat controleert concrete rechten, niet de onjuiste
aanname dat een numerieke handle globaal uniek is. Daarna verwijdert
admin de ciphertext, Usage toont
`Evicted`/used 0/reserved 0; NAS-shutdown alleen via admin.

Koud: service herstelt uitsluitend bestaande canonieke snapshot met verse
echte entropy; admin enumereert `Evicted`, epoch 2, used/reserved 0; gast
kan niet openen, geen beheer uitvoeren of een oude/synthetische
sessiehandle gebruiken. Idempotente eviction/shutdown schrijven niets.
Geen inhoudssleutel wordt aan de NAS of beheerder geleverd.

Diagnostiek: startupwoord 40 = 1 alleen na volledige rolassertions; 240 =
foutstap; 400 = geslaagde RPC's, 408 = foutreplies, 416 = verkeerde caller
(NAS) of SEND-als-RECV-weigering (gast), 424 = ongeldige envelopes, 448 =
aangeraakte stack-high-water. 192 = 1 na geslaagde NAS-recovery/initialisatie.
Elke fout stopt de betreffende proefrol met exit 1; geen groen alleen door
timeout. De kernel/rootrunner moet exit 0 van beheerder/NAS/gast, de nog
levende bestaande volumedienst, alle counters, diskbytes en de koude
herstelrun onafhankelijk valideren. De volumedienst heeft geen nieuwe
shutdownopcode en hoeft niet als vierde proces succesvol af te sluiten.

**Open native gates:** geen manifest/VM-run in dit pakket; geen claim van
stroomuitval, fysieke secure erase, antirollback, netwerk/TLS, E2EE,
achtergrondsync, apparatenrotatie/herstel, UI of volledige P04-acceptatie.
